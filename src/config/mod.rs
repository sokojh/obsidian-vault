pub mod paths;

use std::fs;

use serde::{Deserialize, Serialize};

use crate::error::OvResult;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// Default vault path
    pub vault_path: Option<String>,

    /// Default output format
    pub default_format: Option<String>,

    /// Directories to exclude from scanning
    pub exclude_dirs: Option<Vec<String>>,
}

impl AppConfig {
    pub fn load() -> OvResult<Self> {
        let path = paths::config_path();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let config: AppConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(AppConfig::default())
        }
    }

    pub fn save(&self) -> OvResult<()> {
        let path = paths::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

}
