use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::paths;

const GITHUB_API_URL: &str =
    "https://api.github.com/repos/sokojh/obsidian-vault/releases/latest";
const CHECK_INTERVAL_SECS: u64 = 86400; // 24 hours

#[derive(Debug, Serialize, Deserialize, Default)]
struct VersionCache {
    latest_version: String,
    checked_at: u64,
}

fn cache_path() -> std::path::PathBuf {
    paths::data_dir().join("version_check.json")
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn load_cache() -> Option<VersionCache> {
    let content = fs::read_to_string(cache_path()).ok()?;
    serde_json::from_str(&content).ok()
}

fn save_cache(cache: &VersionCache) {
    let path = cache_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(path, serde_json::to_string(cache).unwrap_or_default());
}

fn fetch_latest_version() -> Option<String> {
    let resp = ureq::get(GITHUB_API_URL)
        .set("User-Agent", "ov-cli")
        .set("Accept", "application/vnd.github.v3+json")
        .timeout(std::time::Duration::from_secs(3))
        .call()
        .ok()?;
    let body: serde_json::Value = resp.into_json().ok()?;
    let tag = body.get("tag_name")?.as_str()?;
    // Strip leading 'v' if present
    Some(tag.trim_start_matches('v').to_string())
}

/// Check for updates and print a stderr notice if a newer version is available.
/// Silently does nothing on any failure. Caches result for 24 hours.
pub fn maybe_notify_update() {
    let current = env!("CARGO_PKG_VERSION");

    // Check cache first
    if let Some(cache) = load_cache() {
        if now_secs() - cache.checked_at < CHECK_INTERVAL_SECS {
            if !cache.latest_version.is_empty() && cache.latest_version != current {
                print_notice(current, &cache.latest_version);
            }
            return;
        }
    }

    // Fetch from GitHub (with short timeout)
    let latest = match fetch_latest_version() {
        Some(v) => v,
        None => return,
    };

    save_cache(&VersionCache {
        latest_version: latest.clone(),
        checked_at: now_secs(),
    });

    if latest != current {
        print_notice(current, &latest);
    }
}

fn print_notice(current: &str, latest: &str) {
    let notice = serde_json::json!({
        "update_available": true,
        "current_version": current,
        "latest_version": latest,
        "upgrade": "cargo install --git https://github.com/sokojh/obsidian-vault.git"
    });
    eprintln!("{}", notice);
}
