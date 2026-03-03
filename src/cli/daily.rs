use clap::Args;

#[derive(Args)]
pub struct DailyArgs {
    /// Target date in YYYY-MM-DD format (defaults to today)
    #[arg(long)]
    pub date: Option<String>,

    /// Preview what would be created without writing the file
    #[arg(long)]
    pub dry_run: bool,
}
