use clap::{Args, Subcommand};

#[derive(Args)]
pub struct IndexArgs {
    #[command(subcommand)]
    pub action: IndexAction,
}

#[derive(Subcommand)]
pub enum IndexAction {
    /// Build or incrementally update the Tantivy search index
    Build,
    /// Show index status (document count, size, last build time)
    Status,
    /// Delete the index directory for a fresh rebuild
    Clear,
}
