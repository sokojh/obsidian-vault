pub mod append;
pub mod config;
pub mod create;
pub mod daily;
pub mod graph;
pub mod index;
pub mod links;
pub mod list;
pub mod read;
pub mod schema;
pub mod search;
pub mod serde_helpers;
pub mod stats;
pub mod tags;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "ov",
    about = "Obsidian Vault CLI — agent-first vault interface. All output is JSON.",
    version
)]
pub struct Cli {
    /// Path to Obsidian vault root directory
    #[arg(long, env = "OV_VAULT", global = true)]
    pub vault: Option<String>,

    /// Output as NDJSON (one JSON object per line) instead of wrapped response
    #[arg(long, global = true)]
    pub jsonl: bool,

    /// Select specific fields in output (comma-separated, e.g., "title,tags,path")
    #[arg(long, global = true)]
    pub fields: Option<String>,

    /// JSON payload input (alternative to individual flags)
    #[arg(long, global = true)]
    pub json: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Full-text search. Supports tag:/in:/title:/date:/type: prefixes. Requires: ov index build
    Search(search::SearchArgs),

    /// Read a note by name. Default: exact match. Use --fuzzy for fuzzy matching
    Read(read::ReadArgs),

    /// List notes with filtering by dir/tag/date and sorting
    List(list::ListArgs),

    /// List all tags with occurrence counts
    Tags(tags::TagsArgs),

    /// Show outgoing [[wiki-links]] from a note
    Links(links::LinksArgs),

    /// Show incoming backlinks pointing to a note
    Backlinks(links::BacklinksArgs),

    /// Explore the link graph (JSON output, or DOT/Mermaid format)
    Graph(graph::GraphArgs),

    /// Show vault-wide statistics (note count, word count, tags, etc.)
    Stats(stats::StatsArgs),

    /// Open or create today's daily note
    Daily(daily::DailyArgs),

    /// Create a new note (plain, frontmatter, or template-based). Supports --dry-run
    Create(create::CreateArgs),

    /// Manage the Tantivy search index (build, status, clear)
    Index(index::IndexArgs),

    /// Get or set configuration values
    Config(config::ConfigArgs),

    /// Append content to an existing note (end, section, or dated entry). Supports --dry-run
    Append(append::AppendArgs),

    /// Introspect CLI schema — list commands, describe inputs/outputs, export skill file
    Schema(schema::SchemaArgs),
}
