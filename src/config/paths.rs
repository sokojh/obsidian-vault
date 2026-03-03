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

/// Common iCloud Obsidian vault locations on macOS
pub fn icloud_vault_candidates() -> Vec<PathBuf> {
    let home = dirs::home_dir().unwrap_or_default();
    vec![home.join("Library/Mobile Documents/iCloud~md~obsidian/Documents")]
}

/// Resolve vault path from CLI arg, config, or auto-detect
pub fn resolve_vault_path(explicit: Option<&str>) -> OvResult<PathBuf> {
    // 1. Explicit CLI argument
    if let Some(path) = explicit {
        let p = PathBuf::from(path);
        if p.join(".obsidian").is_dir() {
            return Ok(p);
        }
        // Maybe it's a subdirectory inside iCloud vault location
        return Err(OvError::VaultNotFound(path.to_string()));
    }

    // 2. OV_VAULT environment variable
    if let Ok(path) = std::env::var("OV_VAULT") {
        let p = PathBuf::from(&path);
        if p.join(".obsidian").is_dir() {
            return Ok(p);
        }
    }

    // 3. Current directory or parents
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

    // 4. iCloud auto-detect: look for vault dirs inside iCloud container
    for candidate in icloud_vault_candidates() {
        if candidate.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&candidate) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() && path.join(".obsidian").is_dir() {
                        return Ok(path);
                    }
                }
            }
        }
    }

    Err(OvError::VaultNotFound(
        "No vault found. Use --vault PATH or set OV_VAULT.".to_string(),
    ))
}
