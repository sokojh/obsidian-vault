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
    // Clear any stale index first
    let _ = ov().args(["index", "clear"]).assert();

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

#[test]
fn test_append_to_note() {
    // Create a temporary note
    ov()
        .args(["create", "AppendTest", "--dir", "Daily", "--format", "json"])
        .assert()
        .success();

    // Append content
    ov()
        .args(["append", "AppendTest", "--content", "New content line", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("appended"));

    // Read back and verify
    ov()
        .args(["read", "AppendTest", "--raw"])
        .assert()
        .success()
        .stdout(predicate::str::contains("New content line"));

    // Clean up
    let path = vault_path().join("Daily/AppendTest.md");
    let _ = std::fs::remove_file(path);
}

#[test]
fn test_append_with_section() {
    // Create a note with a section via template
    ov()
        .args(["create", "AppendSectionTest", "--template", "Person", "--dir", "People", "--format", "json"])
        .assert()
        .success();

    // Append to Timeline section
    ov()
        .args(["append", "AppendSectionTest", "--section", "Timeline", "--content", "Met at conference.", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("appended"));

    // Read and verify content is in the right place
    ov()
        .args(["read", "AppendSectionTest", "--raw"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Met at conference."));

    // Clean up
    let path = vault_path().join("People/AppendSectionTest.md");
    let _ = std::fs::remove_file(path);
}

#[test]
fn test_append_with_date() {
    // Create temp note
    ov()
        .args(["create", "AppendDateTest", "--dir", "Daily", "--format", "json"])
        .assert()
        .success();

    // Append with date
    ov()
        .args(["append", "AppendDateTest", "--date", "--content", "Dated entry.", "--format", "json"])
        .assert()
        .success();

    // Verify date heading was added
    ov()
        .args(["read", "AppendDateTest", "--raw"])
        .assert()
        .success()
        .stdout(predicate::str::contains("###"))
        .stdout(predicate::str::contains("Dated entry."));

    // Clean up
    let path = vault_path().join("Daily/AppendDateTest.md");
    let _ = std::fs::remove_file(path);
}

#[test]
fn test_create_with_person_template() {
    // Clean up in case of previous failed run
    let path = vault_path().join("People/TestPerson.md");
    let _ = std::fs::remove_file(&path);

    ov()
        .args(["create", "TestPerson", "--template", "Person", "--dir", "People",
               "--vars", "org=imweb,role=SRE", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("created"));

    // Read and verify template variables were substituted
    ov()
        .args(["read", "TestPerson", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"type\": \"person\""))
        .stdout(predicate::str::contains("\"org\": \"imweb\""))
        .stdout(predicate::str::contains("\"role\": \"SRE\""))
        .stdout(predicate::str::contains("Timeline"));

    // Clean up
    let path = vault_path().join("People/TestPerson.md");
    let _ = std::fs::remove_file(path);
}

#[test]
fn test_search_type_prefix() {
    // Clear any stale index, then build fresh
    let _ = ov().args(["index", "clear"]).assert();
    ov()
        .args(["index", "build"])
        .assert()
        .success();

    // Search for type:person — should find 김철수
    ov()
        .args(["search", "type:person", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("김철수"));
}

#[test]
fn test_read_person_note() {
    ov()
        .args(["read", "김철수", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("person"))
        .stdout(predicate::str::contains("imweb"));
}

// ─── P0: frontmatter 기본 동작 ──────────────────────────────────────────

#[test]
fn test_create_with_frontmatter() {
    let path = vault_path().join("Daily/FrontmatterTest.md");
    let _ = std::fs::remove_file(&path);

    ov()
        .args(["create", "FrontmatterTest", "--dir", "Daily",
               "--frontmatter", r#"{"type":"article","status":"draft"}"#,
               "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("created"));

    // Read back and verify frontmatter fields
    ov()
        .args(["read", "FrontmatterTest", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"type\": \"article\""))
        .stdout(predicate::str::contains("\"status\": \"draft\""));

    let _ = std::fs::remove_file(&path);
}

// ─── P0: frontmatter 잘못된 JSON 에러 ───────────────────────────────────

#[test]
fn test_create_frontmatter_invalid_json() {
    ov()
        .args(["create", "BadFm", "--frontmatter", "{broken json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid frontmatter JSON"));
}

// ─── P0: path traversal 차단 ────────────────────────────────────────────

#[test]
fn test_create_path_traversal_blocked() {
    ov()
        .args(["create", "EscapeTest", "--dir", "../../tmp"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Path escapes vault boundary"));
}

// ─── P0: --frontmatter + --template 상호 배타 ──────────────────────────

#[test]
fn test_create_frontmatter_template_conflict() {
    ov()
        .args(["create", "ConflictTest",
               "--frontmatter", r#"{"type":"note"}"#,
               "--template", "Person"])
        .assert()
        .failure();
}

// ─── P1: frontmatter + tags 병합 ────────────────────────────────────────

#[test]
fn test_create_frontmatter_with_tags() {
    let path = vault_path().join("Daily/FmTagsTest.md");
    let _ = std::fs::remove_file(&path);

    ov()
        .args(["create", "FmTagsTest", "--dir", "Daily",
               "--frontmatter", r#"{"type":"note"}"#,
               "--tags", "devops,sre",
               "--format", "json"])
        .assert()
        .success();

    // Read raw file to verify tags in YAML frontmatter
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("#devops"), "should contain #devops tag");
    assert!(content.contains("#sre"), "should contain #sre tag");
    assert!(content.contains("type: note"), "should contain type field");

    let _ = std::fs::remove_file(&path);
}

// ─── P1: --sections 단독 사용 ───────────────────────────────────────────

#[test]
fn test_create_with_sections_only() {
    let path = vault_path().join("Daily/SectionsTest.md");
    let _ = std::fs::remove_file(&path);

    ov()
        .args(["create", "SectionsTest", "--dir", "Daily",
               "--sections", "Summary,References",
               "--format", "json"])
        .assert()
        .success();

    ov()
        .args(["read", "SectionsTest", "--raw"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Summary"))
        .stdout(predicate::str::contains("## References"));

    let _ = std::fs::remove_file(&path);
}

// ─── P1: --content 단독 사용 ────────────────────────────────────────────

#[test]
fn test_create_with_content_only() {
    let path = vault_path().join("Daily/ContentTest.md");
    let _ = std::fs::remove_file(&path);

    ov()
        .args(["create", "ContentTest", "--dir", "Daily",
               "--content", "This is initial content.",
               "--format", "json"])
        .assert()
        .success();

    ov()
        .args(["read", "ContentTest", "--raw"])
        .assert()
        .success()
        .stdout(predicate::str::contains("This is initial content."));

    let _ = std::fs::remove_file(&path);
}

// ─── P1: --template + --sections 조합 ──────────────────────────────────

#[test]
fn test_create_template_with_sections() {
    let path = vault_path().join("People/TemplateSectionsTest.md");
    let _ = std::fs::remove_file(&path);

    ov()
        .args(["create", "TemplateSectionsTest", "--dir", "People",
               "--template", "Person",
               "--sections", "Extra Notes,Follow-ups",
               "--format", "json"])
        .assert()
        .success();

    // Template content + extra sections should both be present
    ov()
        .args(["read", "TemplateSectionsTest", "--raw"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Timeline"))     // from template
        .stdout(predicate::str::contains("## Extra Notes"))   // from --sections
        .stdout(predicate::str::contains("## Follow-ups"));   // from --sections

    let _ = std::fs::remove_file(&path);
}

// ─── P1: --template + --content 조합 ───────────────────────────────────

#[test]
fn test_create_template_with_content() {
    let path = vault_path().join("People/TemplateContentTest.md");
    let _ = std::fs::remove_file(&path);

    ov()
        .args(["create", "TemplateContentTest", "--dir", "People",
               "--template", "Person",
               "--content", "Extra note appended after template.",
               "--format", "json"])
        .assert()
        .success();

    ov()
        .args(["read", "TemplateContentTest", "--raw"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Extra note appended after template."));

    let _ = std::fs::remove_file(&path);
}

// ─── P2: --frontmatter + --sections 조합 ───────────────────────────────

#[test]
fn test_create_frontmatter_with_sections() {
    let path = vault_path().join("Daily/FmSectionTest.md");
    let _ = std::fs::remove_file(&path);

    ov()
        .args(["create", "FmSectionTest", "--dir", "Daily",
               "--frontmatter", r#"{"type":"note"}"#,
               "--sections", "Summary,Notes",
               "--format", "json"])
        .assert()
        .success();

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("---"), "should have YAML frontmatter");
    assert!(content.contains("## Summary"), "should have Summary section");
    assert!(content.contains("## Notes"), "should have Notes section");

    let _ = std::fs::remove_file(&path);
}

// ─── P2: --sections + --content 조합 ───────────────────────────────────

#[test]
fn test_create_sections_with_content() {
    let path = vault_path().join("Daily/SectionContentTest.md");
    let _ = std::fs::remove_file(&path);

    ov()
        .args(["create", "SectionContentTest", "--dir", "Daily",
               "--sections", "Summary",
               "--content", "Body text here.",
               "--format", "json"])
        .assert()
        .success();

    ov()
        .args(["read", "SectionContentTest", "--raw"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Summary"))
        .stdout(predicate::str::contains("Body text here."));

    let _ = std::fs::remove_file(&path);
}

// ─── title sanitization 테스트 ──────────────────────────────────────────

#[test]
fn test_create_title_with_slash_rejected() {
    ov()
        .args(["create", "bad/title"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("path separators"));
}

#[test]
fn test_create_title_dotdot_rejected() {
    ov()
        .args(["create", ".."])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be '.' or '..'"));
}

#[test]
fn test_create_title_md_extension_stripped() {
    let path = vault_path().join("Daily/StripMdTest.md");
    let _ = std::fs::remove_file(&path);

    // User passes "StripMdTest.md" — should NOT create "StripMdTest.md.md"
    ov()
        .args(["create", "StripMdTest.md", "--dir", "Daily", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("StripMdTest"));

    assert!(path.exists(), "StripMdTest.md should exist");
    assert!(!vault_path().join("Daily/StripMdTest.md.md").exists(),
            "StripMdTest.md.md should NOT exist");

    let _ = std::fs::remove_file(&path);
}

// ─── template not found → error ────────────────────────────────────────

#[test]
fn test_create_template_not_found() {
    ov()
        .args(["create", "TemplateNotFoundTest", "--template", "NonExistentTemplate"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Template not found"));
}

