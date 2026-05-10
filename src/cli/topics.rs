use clap::Args;
use serde::Deserialize;

#[derive(Args, Deserialize, Default)]
pub struct TopicsArgs {
    /// Comma-separated directory prefixes to exclude. Default: "Files/_archive,docs/480ai"
    #[arg(long)]
    pub exclude_dirs: Option<String>,

    /// Maximum number of topics to return
    #[arg(long, short, default_value = "50")]
    #[serde(default = "default_limit_50")]
    pub limit: usize,

    /// Skip first N results (for pagination)
    #[arg(long, default_value = "0")]
    #[serde(default)]
    pub offset: usize,

    /// Only show topics with at least this many occurrences
    #[arg(long, default_value = "1")]
    #[serde(default = "default_min_count_1")]
    pub min_count: usize,
}

fn default_limit_50() -> usize {
    50
}
fn default_min_count_1() -> usize {
    1
}
