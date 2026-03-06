use clap::Args;
use serde::Deserialize;

#[derive(Args, Deserialize, Default)]
pub struct ReadArgs {
    /// Note name or path (exact match by default, use --fuzzy for fuzzy matching)
    #[arg(long)]
    pub note: Option<String>,

    /// Enable fuzzy matching for note name resolution
    #[arg(long)]
    #[serde(default)]
    pub fuzzy: bool,

    /// Exclude body content from output (body is included by default)
    #[arg(long)]
    #[serde(default)]
    pub no_body: bool,

    /// Output only the raw body text, no JSON wrapping
    #[arg(long)]
    #[serde(default)]
    pub raw: bool,

    /// Extract only a specific section by heading name
    #[arg(long)]
    pub section: Option<String>,
}
