pub mod tools;

use std::collections::HashMap;
use std::path::PathBuf;

use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{tool, tool_handler, tool_router, ServerHandler};

use crate::model::link::Backlink;
use crate::model::note::NoteSummary;
use crate::model::tag::TagSummary;
use crate::search;
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
        let mut notes: Vec<NoteSummary> = vault
            .read_all_notes()
            .iter()
            .map(NoteSummary::from)
            .collect();

        if let Some(ref dir) = params.dir {
            notes.retain(|n| n.dir == *dir || n.dir.starts_with(&format!("{dir}/")));
        }

        if let Some(ref tag) = params.tag {
            let tag_normalized = if tag.starts_with('#') {
                tag.clone()
            } else {
                format!("#{tag}")
            };
            notes.retain(|n| n.tags.iter().any(|t| t == &tag_normalized));
        }

        match params.sort.as_deref() {
            Some("title") => {
                notes.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
            }
            Some("words") => notes.sort_by(|a, b| b.word_count.cmp(&a.word_count)),
            _ => notes.sort_by(|a, b| b.modified.cmp(&a.modified)),
        }

        let limit = params.limit.unwrap_or(50);
        notes.truncate(limit);

        serde_json::to_string_pretty(&serde_json::json!({
            "count": notes.len(),
            "notes": notes,
        }))
        .map_err(|e| e.to_string())
    }

    #[tool(description = "List all tags in the Obsidian vault with usage counts.")]
    async fn vault_tags(
        &self,
        Parameters(params): Parameters<TagsParams>,
    ) -> Result<String, String> {
        let vault = self.open_vault()?;
        let notes = vault.read_all_notes();

        let mut tag_map: HashMap<String, Vec<String>> = HashMap::new();
        for note in &notes {
            for tag in &note.tags {
                tag_map
                    .entry(tag.clone())
                    .or_default()
                    .push(note.title.clone());
            }
        }

        let mut summaries: Vec<TagSummary> = tag_map
            .into_iter()
            .map(|(tag, notes)| TagSummary {
                count: notes.len(),
                tag,
                notes,
            })
            .collect();

        if let Some(min) = params.min_count {
            summaries.retain(|s| s.count >= min);
        }

        match params.sort.as_deref() {
            Some("name") => summaries.sort_by(|a, b| a.tag.cmp(&b.tag)),
            _ => summaries.sort_by(|a, b| b.count.cmp(&a.count)),
        }

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

        let notes = vault.read_all_notes();
        let with_context = params.context.unwrap_or(false);
        let mut backlinks: Vec<Backlink> = Vec::new();

        for note in &notes {
            for link in &note.links {
                if link.target.eq_ignore_ascii_case(&target_stem) {
                    let context = if with_context {
                        note.body.as_ref().and_then(|body| {
                            body.lines()
                                .nth(link.line.saturating_sub(1))
                                .map(|l| l.to_string())
                        })
                    } else {
                        None
                    };

                    backlinks.push(Backlink {
                        source: note.title.clone(),
                        source_path: note.path.clone(),
                        context,
                        line: link.line,
                    });
                }
            }
        }

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

    #[tool(description = "Get statistics about the Obsidian vault (note count, word count, top tags, etc.)")]
    async fn vault_stats(&self) -> Result<String, String> {
        let vault = self.open_vault()?;
        let notes = vault.read_all_notes();

        let total_notes = notes.len();
        let total_words: usize = notes.iter().map(|n| n.word_count).sum();
        let total_links: usize = notes.iter().map(|n| n.links.len()).sum();

        let mut all_tags: HashMap<String, usize> = HashMap::new();
        for note in &notes {
            for tag in &note.tags {
                *all_tags.entry(tag.clone()).or_default() += 1;
            }
        }

        let dirs = vault.directories();
        let total_size: u64 = notes.iter().map(|n| n.file_meta.size).sum();

        let mut sorted_tags: Vec<_> = all_tags.iter().collect();
        sorted_tags.sort_by(|a, b| b.1.cmp(a.1));
        let top_tags: Vec<_> = sorted_tags
            .iter()
            .take(10)
            .map(|(tag, count)| serde_json::json!({"tag": tag, "count": count}))
            .collect();

        serde_json::to_string_pretty(&serde_json::json!({
            "total_notes": total_notes,
            "total_words": total_words,
            "total_links": total_links,
            "unique_tags": all_tags.len(),
            "directories": dirs.len(),
            "total_size_mb": format!("{:.1}", total_size as f64 / 1_048_576.0),
            "top_tags": top_tags,
        }))
        .map_err(|e| e.to_string())
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
