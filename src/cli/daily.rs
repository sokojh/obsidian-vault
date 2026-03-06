use clap::Args;
use serde::Deserialize;

#[derive(Args, Deserialize, Default)]
pub struct DailyArgs {
    /// Target date in YYYY-MM-DD format (defaults to today)
    #[arg(long)]
    pub date: Option<String>,

    /// Preview what would be created without writing the file
    #[arg(long)]
    #[serde(default)]
    pub dry_run: bool,
}
