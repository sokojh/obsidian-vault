use clap::{Args, Subcommand};

#[derive(Args)]
#[command(
    after_long_help = "\x1b[1mExamples:\x1b[0m
  ov index build     # Build index (incremental — only re-indexes changed files)
  ov index status    # Show index stats (doc count, size, freshness)
  ov index clear     # Delete index (next 'build' does full rebuild)"
)]
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
