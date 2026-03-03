use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

fn vault_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample_vault")
}

fn ov() -> Command {
    let mut cmd = Command::cargo_bin("ov").unwrap();
    cmd.arg("--vault").arg(vault_path());
    cmd
}

#[test]
fn test_list_json() {
    ov()
        .args(["list", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ok\": true"))
        .stdout(predicate::str::contains("\"count\":"));
}

#[test]
fn test_list_human() {
    ov()
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("notes"));
}

#[test]
fn test_list_filter_dir() {
    ov()
        .args(["list", "--dir", "Zettelkasten", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("kubernetes-basics"))
        .stdout(predicate::str::contains("docker"));
}

#[test]
fn test_list_filter_tag() {
    ov()
        .args(["list", "--tag", "#TDD", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tdd-article"));
}

#[test]
fn test_list_sort_title() {
    ov()
        .args(["list", "--sort", "title", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ok\": true"));
}

#[test]
fn test_list_limit() {
    ov()
        .args(["list", "--limit", "2", "--format", "json"])
        .assert()
        .success();
}

#[test]
fn test_read_exact() {
    ov()
        .args(["read", "docker", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Docker"))
        .stdout(predicate::str::contains("#devops"));
}

#[test]
fn test_read_fuzzy() {
    ov()
        .args(["read", "kube", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Kubernetes"));
}

#[test]
fn test_read_raw() {
    ov()
        .args(["read", "docker", "--raw"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Docker is a containerization"));
}

#[test]
fn test_read_not_found() {
    ov()
        .args(["read", "nonexistent_note_xyz", "--format", "json"])
        .assert()
        .failure();
}

#[test]
fn test_tags_json() {
    ov()
        .args(["tags", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#devops"))
        .stdout(predicate::str::contains("#kubernetes"));
}

#[test]
fn test_tags_sort_name() {
    ov()
        .args(["tags", "--sort", "name", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ok\": true"));
}

#[test]
fn test_tags_min_count() {
    ov()
        .args(["tags", "--min-count", "2", "--format", "json"])
        .assert()
        .success();
}

#[test]
fn test_stats_json() {
    ov()
        .args(["stats", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("total_notes"))
        .stdout(predicate::str::contains("total_words"))
        .stdout(predicate::str::contains("unique_tags"));
}

#[test]
fn test_stats_human() {
    ov()
        .args(["stats"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Vault Statistics"));
}

#[test]
fn test_links() {
    ov()
        .args(["links", "kubernetes-basics", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Docker"))
        .stdout(predicate::str::contains("Container Networking"));
}

#[test]
fn test_backlinks() {
    ov()
        .args(["backlinks", "docker", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("kubernetes-basics").or(
            predicate::str::contains("container-networking"),
        ));
}

#[test]
fn test_backlinks_with_context() {
    ov()
        .args(["backlinks", "docker", "--context", "--format", "json"])
        .assert()
        .success();
}

#[test]
fn test_config_show() {
    ov()
        .args(["config", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ok\": true"));
}

#[test]
fn test_fields_selector() {
    ov()
        .args(["list", "--format", "json", "--fields", "title,tags"])
        .assert()
        .success()
        .stdout(predicate::str::contains("title"));
}

#[test]
fn test_vault_not_found() {
    Command::cargo_bin("ov")
        .unwrap()
        .args(["--vault", "/nonexistent/path", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Vault not found"));
}


#[test]
fn test_index_build_and_search() {
    // Build index
    ov()
        .args(["index", "build"])
        .assert()
        .success();

    // Check status
    ov()
        .args(["index", "status", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"exists\": true"));

    // Search
    ov()
        .args(["search", "kubernetes", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("kubernetes"));

    // Search with snippet
    ov()
        .args(["search", "Docker", "--snippet", "--format", "json"])
        .assert()
        .success();

    // Search with tag prefix
    ov()
        .args(["search", "tag:#devops container", "--format", "json"])
        .assert()
        .success();

    // Search with no results
    ov()
        .args(["search", "xyznonexistent12345"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No results found"));

    // Clear index
    ov()
        .args(["index", "clear"])
        .assert()
        .success();
}

#[test]
fn test_graph_json() {
    ov()
        .args(["graph", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("nodes"))
        .stdout(predicate::str::contains("edges"));
}

#[test]
fn test_graph_center() {
    ov()
        .args(["graph", "--center", "docker", "--depth", "1", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("docker"));
}

#[test]
fn test_graph_dot() {
    ov()
        .args(["graph", "--graph-format", "dot"])
        .assert()
        .success()
        .stdout(predicate::str::contains("digraph vault"));
}

#[test]
fn test_graph_mermaid() {
    ov()
        .args(["graph", "--graph-format", "mermaid"])
        .assert()
        .success()
        .stdout(predicate::str::contains("graph LR"));
}

#[test]
fn test_daily_dry_run() {
    ov()
        .args(["daily", "--dry-run", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("dry_run"));
}

#[test]
fn test_daily_existing() {
    ov()
        .args(["daily", "--date", "2024-01-15"])
        .assert()
        .success()
        .stdout(predicate::str::contains("2024-01-15"));
}

#[test]
fn test_create_and_read() {
    // Create a new note
    ov()
        .args(["create", "Test Note", "--dir", "Daily", "--tags", "test,tmp", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("created"));

    // Read it back
    ov()
        .args(["read", "Test Note", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Test Note"));

    // Clean up: remove the created file
    let path = vault_path().join("Daily/Test Note.md");
    let _ = std::fs::remove_file(path);
}

#[test]
fn test_create_duplicate() {
    ov()
        .args(["create", "docker", "--dir", "Zettelkasten"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

