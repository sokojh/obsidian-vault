use clap::{Args, Subcommand};

#[derive(Args)]
pub struct IndexArgs {
    #[command(subcommand)]
    pub action: IndexAction,
}

#[derive(Subcommand)]
pub enum IndexAction {
    /// Build or update the search index
    Build,
    /// Show index status
    Status,
    /// Clear the index
    Clear,
}
