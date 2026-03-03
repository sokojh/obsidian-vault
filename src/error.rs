use thiserror::Error;

#[derive(Error, Debug)]
pub enum OvError {
    #[error("General error: {0}")]
    General(String),

    #[error("Vault not found at: {0}")]
    VaultNotFound(String),

    #[error("Index not built. Run `ov index build` first.")]
    IndexNotBuilt,

    #[error("Query parse error: {0}")]
    QueryParse(String),

    #[error("Note not found: {0}")]
    NoteNotFound(String),

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

    #[error("WalkDir error: {0}")]
    WalkDir(#[from] walkdir::Error),
}

impl OvError {
    pub fn exit_code(&self) -> i32 {
        match self {
            OvError::VaultNotFound(_) => 2,
            OvError::IndexNotBuilt => 3,
            OvError::QueryParse(_) => 4,
            _ => 1,
        }
    }

}

pub type OvResult<T> = Result<T, OvError>;
