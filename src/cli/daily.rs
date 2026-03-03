use clap::Args;

#[derive(Args)]
pub struct DailyArgs {
    /// Date (YYYY-MM-DD), defaults to today
    #[arg(long)]
    pub date: Option<String>,

    /// Only show what would be created
    #[arg(long)]
    pub dry_run: bool,
}
