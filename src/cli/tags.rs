use clap::Args;

#[derive(Args)]
pub struct TagsArgs {
    /// Sort by: count, name
    #[arg(long, short, default_value = "count")]
    pub sort: String,

    /// Maximum results
    #[arg(long, short)]
    pub limit: Option<usize>,

    /// Minimum count to show
    #[arg(long)]
    pub min_count: Option<usize>,
}
