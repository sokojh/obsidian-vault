use thiserror::Error;

#[derive(Error, Debug)]
pub enum OvError {
    #[error("{0}")]
    General(String),

    #[error("Vault not found at: {0}")]
    VaultNotFound(String),

    #[error("Index not built. Run `ov index build` first.")]
    IndexNotBuilt,

    #[error("Query parse error: {0}")]
    QueryParse(String),

    #[error("Note not found: {0}")]
    NoteNotFound(String),

    #[error("Note already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("Config write error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("File scan error: {0}")]
    WalkDir(#[from] walkdir::Error),
}

impl OvError {
    pub fn exit_code(&self) -> i32 {
        match self {
            OvError::VaultNotFound(_) => 2,
            OvError::IndexNotBuilt => 3,
            OvError::QueryParse(_) => 4,
            OvError::AlreadyExists(_) => 5,
            OvError::InvalidInput(_) | OvError::MissingField(_) => 6,
            _ => 1,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            OvError::General(_) => "GENERAL_ERROR",
            OvError::VaultNotFound(_) => "VAULT_NOT_FOUND",
            OvError::IndexNotBuilt => "INDEX_NOT_BUILT",
            OvError::QueryParse(_) => "QUERY_PARSE_ERROR",
            OvError::NoteNotFound(_) => "NOTE_NOT_FOUND",
            OvError::AlreadyExists(_) => "ALREADY_EXISTS",
            OvError::InvalidInput(_) => "INVALID_INPUT",
            OvError::MissingField(_) => "MISSING_FIELD",
            OvError::Io(_) => "IO_ERROR",
            OvError::Yaml(_) => "YAML_PARSE_ERROR",
            OvError::Json(_) => "JSON_PARSE_ERROR",
            OvError::Toml(_) => "TOML_PARSE_ERROR",
            OvError::TomlSerialize(_) => "TOML_SERIALIZE_ERROR",
            OvError::WalkDir(_) => "SCAN_ERROR",
        }
    }

    pub fn hint(&self) -> Option<&'static str> {
        match self {
            OvError::General(_) => None,
            OvError::VaultNotFound(_) => Some(
                "Set OV_VAULT env var or use --vault <PATH>. \
                 Vault must contain at least one .md file to be valid.",
            ),
            OvError::IndexNotBuilt => Some(
                "Run `ov index build` to create the search index. \
                 Tantivy index is required for `search`/`graph` commands. \
                 List/read/tags work without index but slower (cold scan).",
            ),
            OvError::QueryParse(_) => Some(
                "Search query syntax: plain keywords, or prefixed filters. \
                 Examples: `tag:idea`, `in:Zettelkasten`, `title:플랜`, `date:2026-05`, `type:zettel`. \
                 Combine with AND/OR. Run `ov schema describe --command search` for examples.",
            ),
            OvError::NoteNotFound(_) => Some(
                "Try one of: (1) `ov read --note <name> --fuzzy` for fuzzy match, \
                 (2) `ov list --dir <folder>` to browse, \
                 (3) `ov search --query <keyword>` to find by content.",
            ),
            OvError::AlreadyExists(_) => Some(
                "Use --if-not-exists to skip silently and treat as success. \
                 Or use `ov append --note <title>` to add to the existing note. \
                 To overwrite intentionally, delete the file first.",
            ),
            OvError::InvalidInput(_) => Some(
                "Run `ov schema describe --command <name>` to see expected input format. \
                 For --json payload, ensure valid JSON with snake_case keys matching the args struct.",
            ),
            OvError::MissingField(_) => Some(
                "Run `ov schema describe --command <name>` to see required fields and examples.",
            ),
            OvError::Io(_) => Some(
                "Check file permissions and that the vault path is accessible. \
                 If using iCloud, ensure files are downloaded (not placeholder).",
            ),
            OvError::Yaml(_) => Some(
                "Frontmatter YAML must be valid. \
                 Common issues: unquoted special chars, tab indentation (use spaces), missing closing `---`. \
                 Use `ov read --note <name>` to inspect the broken file.",
            ),
            OvError::Json(_) => Some(
                "JSON must be valid (double quotes, no trailing commas). \
                 For passing via shell, prefer single quotes around the JSON: --json '{\"key\":\"value\"}'.",
            ),
            OvError::Toml(_) => Some(
                "Config TOML invalid. Inspect with `ov config get --all` or open ~/.config/ov/config.toml.",
            ),
            OvError::TomlSerialize(_) => Some(
                "Failed to write config TOML. Check ~/.config/ov/ permissions.",
            ),
            OvError::WalkDir(_) => Some(
                "File scan failed. Check vault directory permissions and that path exists.",
            ),
        }
    }
}

pub type OvResult<T> = Result<T, OvError>;
