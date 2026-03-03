pub mod append;
pub mod config;
pub mod create;
pub mod daily;
pub mod graph;
pub mod index;
pub mod links;
pub mod list;
pub mod read;
pub mod search;
pub mod stats;
pub mod tags;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "ov",
    about = "Obsidian Vault CLI — high-performance vault interface for terminal and AI",
    version,
    after_long_help = "\x1b[1mExamples:\x1b[0m
  ov list --format json                          # List recent notes as JSON
  ov search \"kubernetes\" --snippet               # Full-text search with snippets
  ov read \"ElasticSearch\" --raw                   # Read note body (fuzzy match)
  ov create \"My Note\" --tags \"idea,k8s\"           # Create a simple note
  ov create \"장애보고\" --frontmatter '{\"type\":\"troubleshooting\"}' --sections \"원인,해결\"
  ov append \"My Note\" --section \"Timeline\" --content \"14:30 이벤트\"
  ov tags --sort count --format json             # List all tags
  ov index build                                 # Build/update search index

\x1b[1mEnvironment:\x1b[0m
  OV_VAULT    Path to vault (alternative to --vault)"
)]
pub struct Cli {
    /// Path to Obsidian vault root directory
    #[arg(long, env = "OV_VAULT", global = true)]
    pub vault: Option<String>,

    /// Output format: human (table), json (wrapped), jsonl (streaming)
    #[arg(long, short, default_value = "human", global = true)]
    pub format: OutputFormat,

    /// Select specific fields in output (comma-separated, e.g., "title,tags,path")
    #[arg(long, global = true)]
    pub fields: Option<String>,

    /// Suppress informational stderr messages
    #[arg(long, short, global = true)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Full-text search (requires: ov index build). Supports tag:/in:/title:/date: prefixes
    Search(search::SearchArgs),

    /// Read a note by name with fuzzy matching
    Read(read::ReadArgs),

    /// List notes with filtering by dir/tag/date and sorting
    List(list::ListArgs),

    /// List all tags with occurrence counts
    Tags(tags::TagsArgs),

    /// Show outgoing [[wiki-links]] from a note
    Links(links::LinksArgs),

    /// Show incoming backlinks pointing to a note
    Backlinks(links::BacklinksArgs),

    /// Explore the link graph (JSON, DOT, or Mermaid output)
    Graph(graph::GraphArgs),

    /// Show vault-wide statistics (note count, word count, tags, etc.)
    Stats(stats::StatsArgs),

    /// Open or create today's daily note
    Daily(daily::DailyArgs),

    /// Create a new note (plain, frontmatter, or template-based)
    Create(create::CreateArgs),

    /// Manage the Tantivy search index (build, status, clear)
    Index(index::IndexArgs),

    /// Get or set configuration values
    Config(config::ConfigArgs),

    /// Append content to an existing note (end, section, or dated entry)
    Append(append::AppendArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Human,
    Json,
    Jsonl,
}
