use clap::Args;

#[derive(Args)]
pub struct CreateArgs {
    /// Note title
    pub title: String,

    /// Target directory within vault
    #[arg(long)]
    pub dir: Option<String>,

    /// Tags to add (comma-separated)
    #[arg(long)]
    pub tags: Option<String>,

    /// Template to use
    #[arg(long)]
    pub template: Option<String>,

    /// Read content from stdin
    #[arg(long)]
    pub stdin: bool,

    /// Template variables (key1=val1,key2=val2)
    #[arg(long)]
    pub vars: Option<String>,
}
