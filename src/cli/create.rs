use clap::Args;

#[derive(Args)]
#[command(
    after_long_help = "\x1b[1mCreation Modes:\x1b[0m
  1. Plain (default):     ov create \"Title\" --tags \"tag1,tag2\"
  2. Frontmatter (JSON):  ov create \"Title\" --frontmatter '{\"type\":\"study\"}' --tags \"tag1\"
  3. Template-based:      ov create \"Title\" --template \"Core Zettel Template\"

  --frontmatter and --template are mutually exclusive.
  --sections and --content work with all modes.

\x1b[1mExamples:\x1b[0m
  # Simple note with tags
  ov create \"Kafka Consumer Groups\" --tags \"kafka,study\"

  # Structured note with YAML frontmatter + section headings
  ov create \"Redis 장애 분석\" \\
    --frontmatter '{\"type\":\"troubleshooting\",\"severity\":\"P1\"}' \\
    --tags \"troubleshooting,redis\" \\
    --sections \"문제 상황,원인 분석,해결 방법\" \\
    --content \"2024-01-15 Redis 커넥션 풀 고갈 발생\"

  # Template-based note with extra sections
  ov create \"김영수\" --template \"_사람정보_템플릿\" --sections \"면담기록\"

  # Note in a specific directory
  ov create \"논문 요약\" --dir Clippings --tags \"clippings,AI\"

  # Pipe content from stdin
  echo \"Body text\" | ov create \"My Note\" --stdin"
)]
pub struct CreateArgs {
    /// Note title (becomes filename and # heading)
    pub title: String,

    /// Target directory within vault (e.g., "Zettelkasten", "Clippings")
    #[arg(long)]
    pub dir: Option<String>,

    /// Comma-separated tags (e.g., "kafka,study"). Auto-prefixed with #
    #[arg(long)]
    pub tags: Option<String>,

    /// Template note name to use as base (fuzzy matching supported)
    #[arg(long, conflicts_with = "frontmatter")]
    pub template: Option<String>,

    /// Read body content from stdin (piped input)
    #[arg(long)]
    pub stdin: bool,

    /// Template variable substitutions (key1=val1,key2=val2). Requires --template
    #[arg(long, requires = "template", conflicts_with = "frontmatter")]
    pub vars: Option<String>,

    /// YAML frontmatter from JSON string. Cannot be used with --template
    #[arg(long, conflicts_with_all = ["template", "vars"])]
    pub frontmatter: Option<String>,

    /// Section headings to add as ## headers (comma-separated). Works with all modes
    #[arg(long)]
    pub sections: Option<String>,

    /// Initial body text to include in the note
    #[arg(long)]
    pub content: Option<String>,
}
