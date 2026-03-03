use clap::Args;

#[derive(Args)]
#[command(after_long_help = "\x1b[1mExamples:\x1b[0m
  ov config                    # Show all config
  ov config vault.path         # Get a specific key
  ov config vault.path /path   # Set a value")]
pub struct ConfigArgs {
    /// Config key to get or set (omit to show all)
    pub key: Option<String>,

    /// Value to set for the given key
    pub value: Option<String>,
}
