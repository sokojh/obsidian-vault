use clap::Args;

#[derive(Args)]
pub struct LinksArgs {
    /// Note name or path
    pub note: String,
}

#[derive(Args)]
pub struct BacklinksArgs {
    /// Note name or path
    pub note: String,

    /// Show context line around the link
    #[arg(long)]
    pub context: bool,
}
