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
            OvError::VaultNotFound(_) => Some("Set OV_VAULT env or use --vault flag"),
            OvError::IndexNotBuilt => Some("Run `ov index build` to create the search index"),
            OvError::NoteNotFound(_) => {
                Some("Use `ov list` to find available notes, or try --fuzzy flag")
            }
            OvError::AlreadyExists(_) => Some("Use --if-not-exists to skip silently"),
            OvError::MissingField(_) => {
                Some("Use `ov schema describe <command>` to see required fields")
            }
            _ => None,
        }
    }
}

pub type OvResult<T> = Result<T, OvError>;
