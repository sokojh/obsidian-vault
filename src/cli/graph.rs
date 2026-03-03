use clap::Args;

#[derive(Args)]
#[command(
    after_long_help = "\x1b[1mExamples:\x1b[0m
  ov graph --format json                                    # Full vault graph
  ov graph --center \"ElasticSearch\" --depth 2 --format json  # Subgraph around a note
  ov graph --graph-format dot                                # Graphviz DOT output
  ov graph --graph-format mermaid                            # Mermaid diagram"
)]
pub struct GraphArgs {
    /// Center note for subgraph extraction (omit for full vault graph)
    #[arg(long)]
    pub center: Option<String>,

    /// Maximum BFS traversal depth from center node
    #[arg(long, default_value = "2")]
    pub depth: usize,

    /// Graph serialization format: json, dot (Graphviz), or mermaid
    #[arg(long, default_value = "json")]
    pub graph_format: String,
}
