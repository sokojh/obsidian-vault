pub mod tools;

use std::path::PathBuf;

use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{tool, tool_handler, tool_router, ServerHandler};

use crate::search;
use crate::service;
use crate::vault::Vault;

use tools::*;

#[derive(Clone)]
pub struct OvMcpServer {
    vault_path: PathBuf,
    tool_router: ToolRouter<Self>,
}

impl OvMcpServer {
    pub fn new(vault_path: PathBuf) -> Self {
        Self {
            vault_path,
            tool_router: Self::tool_router(),
        }
    }

    fn open_vault(&self) -> Result<Vault, String> {
        Vault::open(self.vault_path.clone()).map_err(|e| e.to_string())
    }
}

#[tool_router]
impl OvMcpServer {
    #[tool(description = "Search notes in the Obsidian vault using full-text search. Supports prefix filters: tag:#tagname, in:directory, title:keyword, date:YYYY-MM")]
    async fn vault_search(
        &self,
        Parameters(params): Parameters<SearchParams>,
    ) -> Result<String, String> {
        let limit = params.limit.unwrap_or(20);
        let snippet = params.snippet.unwrap_or(false);

        let results =
            search::search(&self.vault_path, &params.query, limit, 0, snippet)
                .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "count": results.len(),
            "results": results,
        }))
        .map_err(|e| e.to_string())
    }

    #[tool(description = "Read a note from the Obsidian vault. Supports fuzzy name matching.")]
    async fn vault_read(
        &self,
        Parameters(params): Parameters<ReadParams>,
    ) -> Result<String, String> {
        let vault = self.open_vault()?;
        let file_path = vault.resolve_note(&params.note).map_err(|e| e.to_string())?;
        let relative = vault.relative_path(&file_path);
        let mut note = vault.read_note(&relative).map_err(|e| e.to_string())?;

        if params.body == Some(false) {
            note.body = None;
        }

        serde_json::to_string_pretty(&note).map_err(|e| e.to_string())
    }

    #[tool(description = "List notes in the Obsidian vault with optional filtering by directory, tag, and sorting.")]
    async fn vault_list(
        &self,
        Parameters(params): Parameters<ListParams>,
    ) -> Result<String, String> {
        let vault = self.open_vault()?;
        let svc_params = service::ListParams {
            dir: params.dir,
            tag: params.tag,
            date: None,
            sort: params.sort.unwrap_or_else(|| "modified".to_string()),
            reverse: false,
            limit: params.limit.unwrap_or(50),
            offset: 0,
        };
        let result = service::list_notes(vault.notes(), &svc_params);

        serde_json::to_string_pretty(&serde_json::json!({
            "count": result.notes.len(),
            "notes": result.notes,
        }))
        .map_err(|e| e.to_string())
    }

    #[tool(description = "List all tags in the Obsidian vault with usage counts.")]
    async fn vault_tags(
        &self,
        Parameters(params): Parameters<TagsParams>,
    ) -> Result<String, String> {
        let vault = self.open_vault()?;
        let svc_params = service::TagsParams {
            sort: params.sort.unwrap_or_else(|| "count".to_string()),
            min_count: params.min_count,
            limit: None,
        };
        let summaries = service::aggregate_tags(vault.notes(), &svc_params);

        serde_json::to_string_pretty(&serde_json::json!({
            "count": summaries.len(),
            "tags": summaries,
        }))
        .map_err(|e| e.to_string())
    }

    #[tool(description = "Get outgoing links from a note in the Obsidian vault.")]
    async fn vault_links(
        &self,
        Parameters(params): Parameters<LinksParams>,
    ) -> Result<String, String> {
        let vault = self.open_vault()?;
        let file_path = vault
            .resolve_note(&params.note)
            .map_err(|e| e.to_string())?;
        let relative = vault.relative_path(&file_path);
        let note = vault.read_note(&relative).map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "note": note.title,
            "count": note.links.len(),
            "links": note.links,
        }))
        .map_err(|e| e.to_string())
    }

    #[tool(description = "Get backlinks (incoming links) to a note in the Obsidian vault.")]
    async fn vault_backlinks(
        &self,
        Parameters(params): Parameters<BacklinksParams>,
    ) -> Result<String, String> {
        let vault = self.open_vault()?;
        let file_path = vault
            .resolve_note(&params.note)
            .map_err(|e| e.to_string())?;
        let target_stem = file_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let with_context = params.context.unwrap_or(false);
        let backlinks =
            service::find_backlinks(&vault.root, &target_stem, vault.notes(), with_context);

        serde_json::to_string_pretty(&serde_json::json!({
            "target": target_stem,
            "count": backlinks.len(),
            "backlinks": backlinks,
        }))
        .map_err(|e| e.to_string())
    }

    #[tool(description = "Append content to an existing note. Optionally target a specific section heading.")]
    async fn vault_append(
        &self,
        Parameters(params): Parameters<AppendParams>,
    ) -> Result<String, String> {
        let vault = self.open_vault()?;
        let file_path = vault
            .resolve_note(&params.note)
            .map_err(|e| e.to_string())?;
        let relative = vault.relative_path(&file_path);

        let mut new_content = params.content;

        // Prepend date subheading if requested
        if params.date == Some(true) {
            let today = chrono::Local::now().format("%Y-%m-%d").to_string();
            new_content = format!("### {today}\n{new_content}");
        }

        // Read existing file
        let mut file_content =
            std::fs::read_to_string(&file_path).map_err(|e| e.to_string())?;

        if let Some(ref section) = params.section {
            let insert_pos = crate::vault::find_section_insert_point(&file_content, section);
            let prefix = if insert_pos > 0
                && !file_content[..insert_pos].ends_with("\n\n")
            {
                if file_content[..insert_pos].ends_with('\n') {
                    "\n".to_string()
                } else {
                    "\n\n".to_string()
                }
            } else {
                String::new()
            };
            let suffix = if insert_pos < file_content.len()
                && !file_content[insert_pos..].starts_with('\n')
            {
                "\n".to_string()
            } else {
                String::new()
            };
            file_content.insert_str(insert_pos, &format!("{prefix}{new_content}\n{suffix}"));
        } else {
            if !file_content.ends_with('\n') {
                file_content.push('\n');
            }
            file_content.push('\n');
            file_content.push_str(&new_content);
            if !new_content.ends_with('\n') {
                file_content.push('\n');
            }
        }

        std::fs::write(&file_path, &file_content).map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "action": "appended",
            "path": relative,
            "section": params.section,
        }))
        .map_err(|e| e.to_string())
    }

    #[tool(description = "Create a new note in the Obsidian vault with dynamic frontmatter, tags, and section headings. No template file needed — the structure is defined at call time.")]
    async fn vault_create(
        &self,
        Parameters(params): Parameters<CreateParams>,
    ) -> Result<String, String> {
        let vault = self.open_vault()?;

        // Determine target directory
        let dir = params
            .dir
            .as_deref()
            .or(vault.obsidian_config.new_file_location.as_deref())
            .unwrap_or("");

        let filename = format!("{}.md", params.title);
        let relative = if dir.is_empty() {
            filename.clone()
        } else {
            format!("{dir}/{filename}")
        };
        let full_path = vault.root.join(&relative);

        // Path traversal protection: check BEFORE any filesystem side effects.
        // We can't canonicalize (dir may not exist yet), so normalize the
        // logical path by stripping ".." components and verify the result
        // stays under vault root.
        {
            let canonical_root = vault.root.canonicalize().map_err(|e| e.to_string())?;
            // Build normalized path by processing each component
            let mut normalized = canonical_root.clone();
            for component in std::path::Path::new(&relative).components() {
                match component {
                    std::path::Component::ParentDir => {
                        normalized.pop();
                    }
                    std::path::Component::Normal(c) => {
                        normalized.push(c);
                    }
                    _ => {} // skip CurDir, Prefix, RootDir
                }
            }
            if !normalized.starts_with(&canonical_root) {
                return Err(format!(
                    "Path escapes vault boundary: {relative}"
                ));
            }
        }

        // Build file content
        let mut file_content = String::new();

        // Build YAML frontmatter (BTreeMap ensures deterministic key ordering for clean git diffs)
        let mut fm_map = params.frontmatter.unwrap_or_default();

        // Merge tags into frontmatter
        if let Some(tags) = params.tags {
            let tag_values: Vec<serde_json::Value> = tags
                .iter()
                .map(|t| {
                    let tag = if t.starts_with('#') {
                        t.clone()
                    } else {
                        format!("#{t}")
                    };
                    serde_json::Value::String(tag)
                })
                .collect();

            fm_map
                .entry("tags".to_string())
                .and_modify(|v| {
                    if let serde_json::Value::Array(arr) = v {
                        arr.extend(tag_values.clone());
                    } else {
                        *v = serde_json::Value::Array(tag_values.clone());
                    }
                })
                .or_insert(serde_json::Value::Array(tag_values));
        }

        if !fm_map.is_empty() {
            let yaml_str = serde_yaml::to_string(&fm_map).map_err(|e| e.to_string())?;
            file_content.push_str("---\n");
            file_content.push_str(&yaml_str);
            if !yaml_str.ends_with('\n') {
                file_content.push('\n');
            }
            file_content.push_str("---\n");
        }

        // Add sections
        if let Some(sections) = params.sections {
            for heading in &sections {
                file_content.push_str(&format!("\n## {heading}\n\n"));
            }
        }

        // Add initial content
        if let Some(content) = params.content {
            if !file_content.ends_with('\n') {
                file_content.push('\n');
            }
            file_content.push_str(&content);
            if !content.ends_with('\n') {
                file_content.push('\n');
            }
        }

        // Create parent directory if needed (after path traversal check)
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        // Atomic file creation: O_CREAT | O_EXCL via create_new(true)
        // Single syscall — no TOCTOU race between exists() and write()
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&full_path)
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::AlreadyExists {
                    format!("Note already exists: {relative}")
                } else {
                    e.to_string()
                }
            })?;
        file.write_all(file_content.as_bytes())
            .map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "action": "created",
            "path": relative,
            "title": params.title,
        }))
        .map_err(|e| e.to_string())
    }

    #[tool(description = "Get statistics about the Obsidian vault (note count, word count, top tags, etc.)")]
    async fn vault_stats(&self) -> Result<String, String> {
        let vault = self.open_vault()?;
        let stats = service::compute_stats(&vault, vault.notes());

        serde_json::to_string_pretty(&stats).map_err(|e| e.to_string())
    }
}

#[tool_handler]
impl ServerHandler for OvMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Obsidian Vault CLI - search, read, and explore your Obsidian vault".into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            ..Default::default()
        }
    }
}

/// Run the MCP server on stdio
pub async fn run_mcp_server(vault_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let server = OvMcpServer::new(vault_path);

    use rmcp::ServiceExt;
    let service = server
        .serve(rmcp::transport::io::stdio())
        .await?;

    service.waiting().await?;
    Ok(())
}
