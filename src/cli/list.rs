use clap::Args;

#[derive(Args)]
#[command(after_long_help = "\x1b[1mExamples:\x1b[0m
  ov list --format json                          # All notes, newest first
  ov list --dir Zettelkasten --tag \"#troubleshooting\" --limit 10
  ov list --date today                           # Notes modified today
  ov list --date this-week --sort title          # This week, alphabetical
  ov list --sort words --reverse --limit 5       # Top 5 longest notes")]
pub struct ListArgs {
    /// Filter by directory name (e.g., "Zettelkasten", "Clippings")
    #[arg(long)]
    pub dir: Option<String>,

    /// Filter by tag (include # prefix, e.g., "#imweb")
    #[arg(long)]
    pub tag: Option<String>,

    /// Filter by modification date: YYYY-MM-DD, "today", or "this-week"
    #[arg(long)]
    pub date: Option<String>,

    /// Sort field: title, modified, size, words [default: modified]
    #[arg(long, short, default_value = "modified")]
    pub sort: String,

    /// Reverse sort order (e.g., oldest first, A→Z)
    #[arg(long, short)]
    pub reverse: bool,

    /// Maximum number of results to return
    #[arg(long, short, default_value = "50")]
    pub limit: usize,

    /// Skip first N results (for pagination)
    #[arg(long, default_value = "0")]
    pub offset: usize,
}
