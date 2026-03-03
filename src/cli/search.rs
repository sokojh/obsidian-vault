use clap::Args;

#[derive(Args)]
pub struct SearchArgs {
    /// Search query (supports tag:, in:, title:, date: prefixes)
    pub query: String,

    /// Show snippet around match
    #[arg(long)]
    pub snippet: bool,

    /// Maximum results
    #[arg(long, short, default_value = "20")]
    pub limit: usize,

    /// Skip first N results
    #[arg(long, default_value = "0")]
    pub offset: usize,
}
