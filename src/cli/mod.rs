pub mod append;
pub mod config;
pub mod create;
pub mod daily;
pub mod graph;
pub mod index;
pub mod links;
pub mod list;
pub mod mcp;
pub mod read;
pub mod search;
pub mod stats;
pub mod tags;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "ov",
    about = "Obsidian Vault CLI — high-performance vault interface for terminal and AI",
    version
)]
pub struct Cli {
    /// Path to Obsidian vault
    #[arg(long, env = "OV_VAULT", global = true)]
    pub vault: Option<String>,

    /// Output format
    #[arg(long, short, default_value = "human", global = true)]
    pub format: OutputFormat,

    /// Select specific fields (comma-separated)
    #[arg(long, global = true)]
    pub fields: Option<String>,

    /// Suppress stderr output
    #[arg(long, short, global = true)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Full-text search with tag:/in:/title:/date: prefixes
    Search(search::SearchArgs),

    /// Read a note (fuzzy name matching)
    Read(read::ReadArgs),

    /// List notes with filtering and sorting
    List(list::ListArgs),

    /// List all tags with counts
    Tags(tags::TagsArgs),

    /// Show outgoing links from a note
    Links(links::LinksArgs),

    /// Show incoming links (backlinks) to a note
    Backlinks(links::BacklinksArgs),

    /// Link graph visualization
    Graph(graph::GraphArgs),

    /// Vault statistics
    Stats(stats::StatsArgs),

    /// Today's daily note
    Daily(daily::DailyArgs),

    /// Create a new note
    Create(create::CreateArgs),

    /// Search index management
    Index(index::IndexArgs),

    /// Configuration management
    Config(config::ConfigArgs),

    /// Append content to an existing note
    Append(append::AppendArgs),

    /// Start MCP server (stdio)
    Mcp(mcp::McpArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Human,
    Json,
    Jsonl,
}
