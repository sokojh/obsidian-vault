use clap::Args;

#[derive(Args)]
#[command(
    after_long_help = "\x1b[1mExamples:\x1b[0m
  ov read \"ElasticSearch\"                  # Metadata + body (fuzzy match)
  ov read \"ElasticSearch\" --format json    # Full note as JSON
  ov read \"ElasticSearch\" --raw            # Body text only, no metadata"
)]
pub struct ReadArgs {
    /// Note name or path (fuzzy matching supported — partial names work)
    pub note: String,

    /// Include body content in output (default: true)
    #[arg(long, default_value = "true")]
    pub body: bool,

    /// Output only the raw body text, no metadata or formatting
    #[arg(long)]
    pub raw: bool,
}
