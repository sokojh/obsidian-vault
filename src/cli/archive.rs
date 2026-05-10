use clap::Args;
use serde::Deserialize;

#[derive(Args, Deserialize, Default)]
pub struct ArchiveArgs {
    /// Note name (filename without .md, or relative path) to archive
    #[arg(long)]
    pub note: Option<String>,

    /// Target directory inside vault. Default: `Files/_archive/<YYYY-MM-DD>/`
    #[arg(long)]
    pub to: Option<String>,

    /// Enable fuzzy matching for note name resolution
    #[arg(long)]
    #[serde(default)]
    pub fuzzy: bool,

    /// Preview without moving the file
    #[arg(long)]
    #[serde(default)]
    pub dry_run: bool,
}
