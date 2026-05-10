use clap::Args;
use serde::Deserialize;

#[derive(Args, Deserialize, Default)]
pub struct DeadLinksArgs {
    /// Maximum number of results
    #[arg(long, short, default_value = "200")]
    #[serde(default = "default_limit_200")]
    pub limit: usize,
}

fn default_limit_200() -> usize {
    200
}
