use schemars::JsonSchema;
use serde::Deserialize;

/// Parameters for vault_search tool
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchParams {
    /// Search query (supports tag:, in:, title:, date: prefixes)
    pub query: String,
    /// Maximum results (default 20)
    pub limit: Option<usize>,
    /// Include snippet around match
    pub snippet: Option<bool>,
}

/// Parameters for vault_read tool
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadParams {
    /// Note name or path (fuzzy matching supported)
    pub note: String,
    /// Include body content (default true)
    pub body: Option<bool>,
}

/// Parameters for vault_list tool
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListParams {
    /// Filter by directory
    pub dir: Option<String>,
    /// Filter by tag (e.g., "#devops")
    pub tag: Option<String>,
    /// Sort by: title, modified, words
    pub sort: Option<String>,
    /// Maximum results (default 50)
    pub limit: Option<usize>,
}

/// Parameters for vault_tags tool
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TagsParams {
    /// Sort by: count, name
    pub sort: Option<String>,
    /// Minimum count to show
    pub min_count: Option<usize>,
}

/// Parameters for vault_links tool
#[derive(Debug, Deserialize, JsonSchema)]
pub struct LinksParams {
    /// Note name or path
    pub note: String,
}

/// Parameters for vault_backlinks tool
#[derive(Debug, Deserialize, JsonSchema)]
pub struct BacklinksParams {
    /// Note name or path
    pub note: String,
    /// Include context line
    pub context: Option<bool>,
}
