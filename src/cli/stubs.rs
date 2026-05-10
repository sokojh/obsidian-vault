use clap::Args;
use serde::Deserialize;

#[derive(Args, Deserialize, Default)]
pub struct StubsArgs {
    /// Maximum word count to consider a note a stub
    #[arg(long, default_value = "100")]
    #[serde(default = "default_max_words")]
    pub max_words: usize,

    /// Restrict to a single directory (e.g. "Zettelkasten")
    #[arg(long)]
    pub dir: Option<String>,

    /// Maximum number of results
    #[arg(long, short, default_value = "100")]
    #[serde(default = "default_limit_100")]
    pub limit: usize,

    /// Sort: words (default, shortest first), modified, title
    #[arg(long, default_value = "words")]
    #[serde(default = "default_sort_words")]
    pub sort: String,
}

fn default_max_words() -> usize {
    100
}
fn default_limit_100() -> usize {
    100
}
fn default_sort_words() -> String {
    "words".into()
}
