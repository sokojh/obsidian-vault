use clap::Args;

#[derive(Args)]
#[command(after_long_help = "\x1b[1mExamples:\x1b[0m
  ov tags --format json                  # All tags sorted by count
  ov tags --sort name                    # Alphabetical order
  ov tags --min-count 5 --limit 10       # Top 10 tags with 5+ uses")]
pub struct TagsArgs {
    /// Sort field: count (most used first) or name (alphabetical) [default: count]
    #[arg(long, short, default_value = "count")]
    pub sort: String,

    /// Maximum number of tags to return
    #[arg(long, short)]
    pub limit: Option<usize>,

    /// Only show tags with at least this many occurrences
    #[arg(long)]
    pub min_count: Option<usize>,
}
