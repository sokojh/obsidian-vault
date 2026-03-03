use clap::Args;

#[derive(Args)]
pub struct GraphArgs {
    /// Center node for subgraph
    #[arg(long)]
    pub center: Option<String>,

    /// BFS depth from center
    #[arg(long, default_value = "2")]
    pub depth: usize,

    /// Graph output format: json, dot, mermaid
    #[arg(long, default_value = "json")]
    pub graph_format: String,
}
