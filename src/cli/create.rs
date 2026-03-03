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
    #[arg(long, conflicts_with = "frontmatter")]
    pub template: Option<String>,

    /// Read content from stdin
    #[arg(long)]
    pub stdin: bool,

    /// Template variables (key1=val1,key2=val2)
    #[arg(long, requires = "template", conflicts_with = "frontmatter")]
    pub vars: Option<String>,

    /// Frontmatter as JSON string (e.g., '{"type":"person","role":"SRE"}')
    #[arg(long, conflicts_with_all = ["template", "vars"])]
    pub frontmatter: Option<String>,

    /// Section headings (comma-separated, e.g., "Summary,Timeline")
    #[arg(long)]
    pub sections: Option<String>,

    /// Initial body content
    #[arg(long)]
    pub content: Option<String>,
}
