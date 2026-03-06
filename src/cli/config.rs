use clap::Args;
use serde::Deserialize;

#[derive(Args, Deserialize, Default)]
pub struct ConfigArgs {
    /// Config key to get or set (omit to show all)
    #[arg(long)]
    pub key: Option<String>,

    /// Value to set for the given key
    #[arg(long)]
    pub value: Option<String>,
}
