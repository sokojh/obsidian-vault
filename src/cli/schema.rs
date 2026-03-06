use clap::{Args, Subcommand};

#[derive(Args)]
pub struct SchemaArgs {
    #[command(subcommand)]
    pub action: SchemaAction,
}

#[derive(Subcommand)]
pub enum SchemaAction {
    /// List all available commands with descriptions and side-effect flags
    Commands,
    /// Describe a specific command's input fields, output fields, and examples
    Describe(DescribeArgs),
    /// Export a complete skill file (markdown) for agent context injection
    Skill,
}

#[derive(Args)]
pub struct DescribeArgs {
    /// Command name to describe (e.g., "list", "create", "search")
    #[arg(long)]
    pub command: Option<String>,
}
