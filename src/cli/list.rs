use clap::Args;

#[derive(Args)]
pub struct ListArgs {
    /// Filter by directory
    #[arg(long)]
    pub dir: Option<String>,

    /// Filter by tag (e.g., "#imweb")
    #[arg(long)]
    pub tag: Option<String>,

    /// Filter by date (YYYY-MM-DD, "today", "this-week")
    #[arg(long)]
    pub date: Option<String>,

    /// Sort by: title, modified, size, words
    #[arg(long, short, default_value = "modified")]
    pub sort: String,

    /// Reverse sort order
    #[arg(long, short)]
    pub reverse: bool,

    /// Maximum results
    #[arg(long, short, default_value = "50")]
    pub limit: usize,

    /// Skip first N results
    #[arg(long, default_value = "0")]
    pub offset: usize,
}
