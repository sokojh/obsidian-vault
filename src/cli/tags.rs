use clap::Args;
use serde::Deserialize;

#[derive(Args, Deserialize, Default)]
pub struct TagsArgs {
    /// Sort field: count (most used first) or name (alphabetical)
    #[arg(long, short, default_value = "count")]
    #[serde(default = "default_sort_count")]
    pub sort: String,

    /// Maximum number of tags to return
    #[arg(long, short)]
    pub limit: Option<usize>,

    /// Only show tags with at least this many occurrences
    #[arg(long)]
    pub min_count: Option<usize>,
}

fn default_sort_count() -> String {
    "count".to_string()
}
