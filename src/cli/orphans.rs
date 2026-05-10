use clap::Args;
use serde::Deserialize;

#[derive(Args, Deserialize, Default)]
pub struct OrphansArgs {
    /// Comma-separated directory prefixes to exclude (e.g. "Files/_archive,docs")
    /// Defaults to "Files/_archive,docs/480ai" — archived/staged content rarely worth flagging.
    #[arg(long)]
    pub exclude_dirs: Option<String>,

    /// Maximum number of results to return
    #[arg(long, short, default_value = "100")]
    #[serde(default = "default_limit_100")]
    pub limit: usize,

    /// Sort: modified (default, newest first), title, words
    #[arg(long, default_value = "modified")]
    #[serde(default = "default_sort_modified")]
    pub sort: String,
}

fn default_limit_100() -> usize {
    100
}
fn default_sort_modified() -> String {
    "modified".into()
}
