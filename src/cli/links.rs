use clap::Args;

#[derive(Args)]
pub struct LinksArgs {
    /// Note name or path (fuzzy matching supported)
    pub note: String,
}

#[derive(Args)]
pub struct BacklinksArgs {
    /// Note name or path (fuzzy matching supported)
    pub note: String,

    /// Show surrounding text context around each backlink reference
    #[arg(long)]
    pub context: bool,
}
