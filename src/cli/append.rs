use clap::Args;

#[derive(Args)]
#[command(
    after_long_help = "\x1b[1mExamples:\x1b[0m
  ov append \"My Note\" --content \"추가 내용\"                       # Append to end
  ov append \"My Note\" --section \"Timeline\" --content \"14:30 이벤트\"  # Insert under section
  ov append \"My Note\" --date --content \"오늘의 기록\"               # Auto date heading
  echo \"piped text\" | ov append \"My Note\" --stdin                 # From stdin"
)]
pub struct AppendArgs {
    /// Note name to append to (fuzzy matching supported)
    pub note: String,

    /// Insert under this ## section heading instead of end of file
    #[arg(long)]
    pub section: Option<String>,

    /// Read content from stdin (piped input)
    #[arg(long)]
    pub stdin: bool,

    /// Inline content text to append
    #[arg(long)]
    pub content: Option<String>,

    /// Prepend a ### YYYY-MM-DD date heading before the content
    #[arg(long)]
    pub date: bool,
}
