use std::path::{Path, PathBuf};

use walkdir::WalkDir;

/// Directories to always exclude from scanning
const EXCLUDED_DIRS: &[&str] = &[
    ".obsidian",
    ".smart-connections",
    ".smart-env",
    "smart-chats",
    "personal_account",
    ".trash",
    ".git",
    "node_modules",
];

/// File patterns to exclude
const EXCLUDED_EXTENSIONS: &[&str] = &["excalidraw.md"];

/// Scan vault directory for markdown files
pub fn scan_vault(root: &Path, extra_excludes: &[String]) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for entry in WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !is_excluded_dir(e.path(), root, extra_excludes))
    {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();

        // Only .md files
        if path.extension().is_none_or(|ext| ext != "md") {
            continue;
        }

        // Skip excalidraw files
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if EXCLUDED_EXTENSIONS.iter().any(|ext| name.ends_with(ext)) {
            continue;
        }

        // Skip iCloud evicted files (.icloud placeholder)
        if name.starts_with('.') && name.ends_with(".icloud") {
            continue;
        }

        files.push(path.to_path_buf());
    }

    files.sort();
    files
}

fn is_excluded_dir(path: &Path, root: &Path, extra_excludes: &[String]) -> bool {
    if !path.is_dir() {
        return false;
    }

    let dir_name = path.file_name().unwrap_or_default().to_string_lossy();

    // Built-in exclusions
    if EXCLUDED_DIRS.iter().any(|&d| dir_name == d) {
        return true;
    }

    // User-configured exclusions
    if let Ok(relative) = path.strip_prefix(root) {
        let rel_str = relative.to_string_lossy();
        if extra_excludes
            .iter()
            .any(|e| rel_str.starts_with(e.as_str()))
        {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scan_vault_basic() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        // Create some files
        fs::write(root.join("note1.md"), "# Note 1").unwrap();
        fs::write(root.join("note2.md"), "# Note 2").unwrap();
        fs::write(root.join("not_md.txt"), "text").unwrap();

        let files = scan_vault(root, &[]);
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_scan_excludes_dirs() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        fs::write(root.join("note.md"), "# Note").unwrap();
        fs::create_dir_all(root.join(".obsidian")).unwrap();
        fs::write(root.join(".obsidian/config.md"), "config").unwrap();
        fs::create_dir_all(root.join("personal_account")).unwrap();
        fs::write(root.join("personal_account/secret.md"), "secret").unwrap();

        let files = scan_vault(root, &[]);
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("note.md"));
    }

    #[test]
    fn test_scan_excludes_excalidraw() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        fs::write(root.join("note.md"), "# Note").unwrap();
        fs::write(root.join("diagram.excalidraw.md"), "excalidraw").unwrap();

        let files = scan_vault(root, &[]);
        assert_eq!(files.len(), 1);
    }
}
