use clap::Args;
use serde::Deserialize;

use super::serde_helpers::string_or_array;

#[derive(Args, Deserialize, Default)]
pub struct CreateArgs {
    /// Note title (becomes filename and # heading)
    #[arg(long)]
    pub title: Option<String>,

    /// Target directory within vault (e.g., "Zettelkasten", "Clippings")
    #[arg(long)]
    pub dir: Option<String>,

    /// Tags: comma-separated string ("a,b") or JSON array (["a","b"]). Auto-prefixed with #
    #[arg(long)]
    #[serde(default, deserialize_with = "string_or_array")]
    pub tags: Option<String>,

    /// Template note name to use as base
    #[arg(long, conflicts_with = "frontmatter")]
    pub template: Option<String>,

    /// Read body content from stdin (piped input)
    #[arg(long)]
    #[serde(default)]
    pub stdin: bool,

    /// Template variable substitutions (key1=val1,key2=val2). Requires --template
    #[arg(long, requires = "template", conflicts_with = "frontmatter")]
    pub vars: Option<String>,

    /// YAML frontmatter from JSON string. Cannot be used with --template
    #[arg(long, conflicts_with_all = ["template", "vars"])]
    pub frontmatter: Option<String>,

    /// Section headings: comma-separated string ("A,B") or JSON array (["A","B"])
    #[arg(long)]
    #[serde(default, deserialize_with = "string_or_array")]
    pub sections: Option<String>,

    /// Initial body text to include in the note
    #[arg(long)]
    pub content: Option<String>,

    /// Preview what would be created without writing the file
    #[arg(long)]
    #[serde(default)]
    pub dry_run: bool,

    /// Skip silently if note already exists (idempotent operation)
    #[arg(long)]
    #[serde(default)]
    pub if_not_exists: bool,
}
