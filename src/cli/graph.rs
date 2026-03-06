use clap::Args;
use serde::Deserialize;

#[derive(Args, Deserialize, Default)]
pub struct GraphArgs {
    /// Center note for subgraph extraction (omit for full vault graph)
    #[arg(long)]
    pub center: Option<String>,

    /// Maximum BFS traversal depth from center node
    #[arg(long, default_value = "2")]
    #[serde(default = "default_depth")]
    pub depth: usize,

    /// Graph serialization format: json, dot (Graphviz), or mermaid
    #[arg(long, default_value = "json")]
    #[serde(default = "default_graph_format")]
    pub graph_format: String,

    /// Enable fuzzy matching for center note resolution
    #[arg(long)]
    #[serde(default)]
    pub fuzzy: bool,
}

fn default_depth() -> usize {
    2
}
fn default_graph_format() -> String {
    "json".to_string()
}
