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
