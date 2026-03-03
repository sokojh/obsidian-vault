use clap::Args;

#[derive(Args)]
pub struct ConfigArgs {
    /// Config key to get/set
    pub key: Option<String>,

    /// Value to set
    pub value: Option<String>,
}
