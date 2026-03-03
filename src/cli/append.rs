use clap::Args;

#[derive(Args)]
pub struct AppendArgs {
    /// Note name (fuzzy matching supported)
    pub note: String,

    /// Target section heading (e.g., "Timeline", "Notes")
    #[arg(long)]
    pub section: Option<String>,

    /// Read content from stdin
    #[arg(long)]
    pub stdin: bool,

    /// Inline content to append
    #[arg(long)]
    pub content: Option<String>,

    /// Add date subheading (### YYYY-MM-DD)
    #[arg(long)]
    pub date: bool,
}
