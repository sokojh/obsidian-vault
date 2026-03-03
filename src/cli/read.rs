use clap::Args;

#[derive(Args)]
pub struct ReadArgs {
    /// Note name or path (fuzzy matching supported)
    pub note: String,

    /// Include raw body content in output
    #[arg(long, default_value = "true")]
    pub body: bool,

    /// Only show body content (no metadata)
    #[arg(long)]
    pub raw: bool,
}
