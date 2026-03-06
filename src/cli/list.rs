use clap::Args;
use serde::Deserialize;

#[derive(Args, Deserialize, Default)]
pub struct ListArgs {
    /// Filter by directory name (e.g., "Zettelkasten", "Clippings")
    #[arg(long)]
    pub dir: Option<String>,

    /// Filter by tag (e.g., "#imweb")
    #[arg(long)]
    pub tag: Option<String>,

    /// Filter by modification date: YYYY-MM-DD or "today"
    #[arg(long)]
    pub date: Option<String>,

    /// Sort field: title | modified | size | words
    #[arg(long, short, default_value = "modified")]
    #[serde(default = "default_sort_modified")]
    pub sort: String,

    /// Reverse sort order
    #[arg(long, short)]
    #[serde(default)]
    pub reverse: bool,

    /// Maximum number of results to return
    #[arg(long, short, default_value = "50")]
    #[serde(default = "default_limit_50")]
    pub limit: usize,

    /// Skip first N results (for pagination)
    #[arg(long, default_value = "0")]
    #[serde(default)]
    pub offset: usize,
}

fn default_sort_modified() -> String {
    "modified".to_string()
}
fn default_limit_50() -> usize {
    50
}
