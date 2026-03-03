use clap::Args;

#[derive(Args)]
#[command(
    after_long_help = "\x1b[1mQuery Prefixes:\x1b[0m
  tag:#imweb          Filter by tag
  in:Clippings        Filter by directory
  title:장애          Search in title only
  date:2024-01       Filter by date (YYYY, YYYY-MM, or YYYY-MM-DD)
  type:troubleshooting  Filter by frontmatter type

  Prefixes can be combined with keywords: \"tag:#k8s in:Zettelkasten pod\"

\x1b[1mExamples:\x1b[0m
  ov search \"kubernetes\"                    # Keyword search
  ov search \"tag:#imweb\" --snippet          # Tag filter with context
  ov search \"in:Clippings docker\"           # Search within directory
  ov search \"title:장애\" --format json       # Title search, JSON output

\x1b[1mNote:\x1b[0m
  Requires a search index. Run 'ov index build' first."
)]
pub struct SearchArgs {
    /// Search query — plain keywords or prefixed filters (tag:, in:, title:, date:, type:)
    pub query: String,

    /// Show text snippet around each match
    #[arg(long)]
    pub snippet: bool,

    /// Maximum number of results to return
    #[arg(long, short, default_value = "20")]
    pub limit: usize,

    /// Skip first N results (for pagination)
    #[arg(long, default_value = "0")]
    pub offset: usize,
}
