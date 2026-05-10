use clap::Args;
use serde::Deserialize;

#[derive(Args, Deserialize, Default)]
pub struct DeadLinksArgs {
    /// Comma-separated directory prefixes to exclude (skip notes IN these dirs).
    /// Defaults to "Files/_archive,docs/480ai" — archived content's stale links shouldn't pollute live diagnostics.
    #[arg(long)]
    pub exclude_dirs: Option<String>,

    /// Maximum number of results
    #[arg(long, short, default_value = "200")]
    #[serde(default = "default_limit_200")]
    pub limit: usize,

    /// Skip first N results (for pagination)
    #[arg(long, default_value = "0")]
    #[serde(default)]
    pub offset: usize,
}

fn default_limit_200() -> usize {
    200
}
