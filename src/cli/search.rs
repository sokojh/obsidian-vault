use clap::Args;
use serde::Deserialize;

#[derive(Args, Deserialize, Default)]
pub struct SearchArgs {
    /// Search query (positional). Equivalent to --query. e.g. `ov search "TMAP"`
    #[arg(value_name = "QUERY", conflicts_with = "query")]
    #[serde(default)]
    pub query_pos: Option<String>,

    /// Search query — plain keywords or prefixed filters (tag:, in:, title:, date:, type:)
    #[arg(long)]
    pub query: Option<String>,

    /// Show text snippet around each match
    #[arg(long)]
    #[serde(default)]
    pub snippet: bool,

    /// Maximum number of results to return
    #[arg(long, short, default_value = "20")]
    #[serde(default = "default_limit_20")]
    pub limit: usize,

    /// Skip first N results (for pagination)
    #[arg(long, default_value = "0")]
    #[serde(default)]
    pub offset: usize,
}

fn default_limit_20() -> usize {
    20
}
