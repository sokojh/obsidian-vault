use clap::Args;
use serde::Deserialize;

#[derive(Args, Deserialize, Default)]
pub struct LinksArgs {
    /// Note name or path (exact match by default)
    #[arg(long)]
    pub note: Option<String>,

    /// Enable fuzzy matching for note name resolution
    #[arg(long)]
    #[serde(default)]
    pub fuzzy: bool,
}

#[derive(Args, Deserialize, Default)]
pub struct BacklinksArgs {
    /// Note name or path (exact match by default)
    #[arg(long)]
    pub note: Option<String>,

    /// Show surrounding text context around each backlink reference
    #[arg(long)]
    #[serde(default)]
    pub context: bool,

    /// Enable fuzzy matching for note name resolution
    #[arg(long)]
    #[serde(default)]
    pub fuzzy: bool,
}
