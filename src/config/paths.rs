use std::path::{Path, PathBuf};

use crate::error::{OvError, OvResult};

/// XDG-compliant data directory: ~/.local/share/ov/
pub fn data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
        .join("ov")
}

/// Config file path: ~/.local/share/ov/config.toml
pub fn config_path() -> PathBuf {
    data_dir().join("config.toml")
}

/// Per-vault index directory: ~/.local/share/ov/vaults/<vault-hash>/
pub fn vault_index_dir(vault_path: &Path) -> PathBuf {
    let hash = blake3::hash(vault_path.to_string_lossy().as_bytes());
    let short_hash = &hash.to_hex()[..16];
    data_dir().join("vaults").join(short_hash)
}

/// Common cloud-synced Obsidian vault locations across platforms
fn cloud_vault_candidates() -> Vec<PathBuf> {
    let home = dirs::home_dir().unwrap_or_default();
    let mut candidates = Vec::new();

    // macOS iCloud
    candidates.push(home.join("Library/Mobile Documents/iCloud~md~obsidian/Documents"));

    // Windows iCloud
    candidates.push(home.join("iCloudDrive/iCloud~md~obsidian/Documents"));
    // Windows iCloud (alternative)
    if let Ok(appdata) = std::env::var("APPDATA") {
        candidates.push(
            PathBuf::from(appdata).join("Apple Computer/iCloudDrive/iCloud~md~obsidian/Documents"),
        );
    }

    // Dropbox
    candidates.push(home.join("Dropbox"));
    candidates.push(home.join("Dropbox/Obsidian"));

    // OneDrive (personal)
    candidates.push(home.join("OneDrive"));
    candidates.push(home.join("OneDrive/Obsidian"));

    // OneDrive (business) — Windows
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        let up = PathBuf::from(userprofile);
        candidates.push(up.join("OneDrive - *")); // glob-like, handled below
    }

    // Google Drive
    candidates.push(home.join("Google Drive/My Drive"));
    candidates.push(home.join("Google Drive/My Drive/Obsidian"));

    // Linux common paths
    candidates.push(home.join("Documents/Obsidian"));
    candidates.push(home.join("Obsidian"));

    candidates
}

/// Scan a directory for Obsidian vaults (dirs containing .obsidian/)
fn find_vaults_in(dir: &Path) -> Vec<PathBuf> {
    let mut vaults = Vec::new();
    if dir.is_dir() {
        // Check if the dir itself is a vault
        if dir.join(".obsidian").is_dir() {
            vaults.push(dir.to_path_buf());
            return vaults;
        }
        // Check immediate children
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.join(".obsidian").is_dir() {
                    vaults.push(path);
                }
            }
        }
    }
    vaults
}

/// List all auto-detectable vaults across cloud sync locations
pub fn discover_vaults() -> Vec<PathBuf> {
    let mut all_vaults = Vec::new();
    for candidate in cloud_vault_candidates() {
        // Handle glob-like patterns (OneDrive business)
        if candidate.to_string_lossy().contains('*') {
            if let Some(parent) = candidate.parent() {
                if let Ok(entries) = std::fs::read_dir(parent) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            all_vaults.extend(find_vaults_in(&path));
                        }
                    }
                }
            }
        } else {
            all_vaults.extend(find_vaults_in(&candidate));
        }
    }
    all_vaults.sort();
    all_vaults.dedup();
    all_vaults
}

/// Resolve vault path from CLI arg, config, env, cwd, or auto-detect
pub fn resolve_vault_path(explicit: Option<&str>) -> OvResult<PathBuf> {
    // 1. Explicit CLI argument (--vault)
    if let Some(path) = explicit {
        let p = PathBuf::from(path);
        if p.join(".obsidian").is_dir() {
            return Ok(p);
        }
        return Err(OvError::VaultNotFound(path.to_string()));
    }

    // 2. OV_VAULT environment variable
    if let Ok(path) = std::env::var("OV_VAULT") {
        let p = PathBuf::from(&path);
        if p.join(".obsidian").is_dir() {
            return Ok(p);
        }
    }

    // 3. Saved config (ov config --key vault_path --value "...")
    if let Ok(config) = super::AppConfig::load() {
        if let Some(ref path) = config.vault_path {
            let p = PathBuf::from(path);
            if p.join(".obsidian").is_dir() {
                return Ok(p);
            }
        }
    }

    // 4. Current directory or parents (.obsidian/ walk-up)
    let cwd = std::env::current_dir().unwrap_or_default();
    let mut dir = cwd.as_path();
    loop {
        if dir.join(".obsidian").is_dir() {
            return Ok(dir.to_path_buf());
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }

    // 5. Auto-detect from cloud sync locations
    let vaults = discover_vaults();
    match vaults.len() {
        0 => Err(OvError::VaultNotFound(
            "No vault found. Use --vault PATH or set OV_VAULT".to_string(),
        )),
        1 => Ok(vaults.into_iter().next().unwrap()),
        _ => {
            let vault_list: Vec<String> = vaults
                .iter()
                .map(|v| v.to_string_lossy().to_string())
                .collect();
            Err(OvError::InvalidInput(format!(
                "Multiple vaults found. Specify one with --vault or OV_VAULT: {}",
                vault_list.join(", ")
            )))
        }
    }
}
