use clap::Args;
use serde::Deserialize;

#[derive(Args, Deserialize, Default)]
pub struct AppendArgs {
    /// Note name to append to (exact match by default, use --fuzzy for fuzzy matching)
    #[arg(long)]
    pub note: Option<String>,

    /// Insert under this ## section heading instead of end of file
    #[arg(long)]
    pub section: Option<String>,

    /// Read content from stdin (piped input)
    #[arg(long)]
    #[serde(default)]
    pub stdin: bool,

    /// Inline content text to append
    #[arg(long)]
    pub content: Option<String>,

    /// Prepend a ### YYYY-MM-DD date heading before the content
    #[arg(long)]
    #[serde(default)]
    pub date: bool,

    /// Enable fuzzy matching for note name resolution
    #[arg(long)]
    #[serde(default)]
    pub fuzzy: bool,

    /// Preview what would be appended without writing the file
    #[arg(long)]
    #[serde(default)]
    pub dry_run: bool,
}
