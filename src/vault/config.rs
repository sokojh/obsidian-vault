use std::fs;
use std::path::Path;

use serde::Deserialize;

/// Parsed .obsidian/app.json settings
#[derive(Debug, Clone, Default)]
pub struct ObsidianConfig {
    pub attachment_folder: Option<String>,
    pub new_file_location: Option<String>,
    pub template_folder: Option<String>,
}

#[derive(Deserialize)]
struct AppJson {
    #[serde(rename = "attachmentFolderPath")]
    attachment_folder_path: Option<String>,
    #[serde(rename = "newFileLocation")]
    new_file_location: Option<String>,
}

#[derive(Deserialize)]
struct TemplatesJson {
    folder: Option<String>,
}

impl ObsidianConfig {
    pub fn load(vault_root: &Path) -> Self {
        let obsidian_dir = vault_root.join(".obsidian");
        let mut config = ObsidianConfig::default();

        // Parse app.json
        if let Ok(content) = fs::read_to_string(obsidian_dir.join("app.json")) {
            if let Ok(app) = serde_json::from_str::<AppJson>(&content) {
                config.attachment_folder = app.attachment_folder_path;
                config.new_file_location = app.new_file_location;
            }
        }

        // Parse templates.json
        if let Ok(content) = fs::read_to_string(obsidian_dir.join("templates.json")) {
            if let Ok(templates) = serde_json::from_str::<TemplatesJson>(&content) {
                config.template_folder = templates.folder;
            }
        }

        config
    }
}
