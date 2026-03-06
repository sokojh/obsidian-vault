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

// ─── list ────────────────────────────────────────────────────────────────

#[test]
fn test_list_json() {
    ov().args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ok\":true"))
        .stdout(predicate::str::contains("\"count\":"));
}

#[test]
fn test_list_filter_dir() {
    ov().args(["list", "--dir", "Zettelkasten"])
        .assert()
        .success()
        .stdout(predicate::str::contains("kubernetes-basics"))
        .stdout(predicate::str::contains("docker"));
}

#[test]
fn test_list_filter_tag() {
    ov().args(["list", "--tag", "#TDD"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tdd-article"));
}

#[test]
fn test_list_sort_title() {
    ov().args(["list", "--sort", "title"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ok\":true"));
}

#[test]
fn test_list_limit() {
    ov().args(["list", "--limit", "2"]).assert().success();
}

#[test]
fn test_list_json_input() {
    ov().args(["list", "--json", r#"{"dir":"Zettelkasten","limit":3}"#])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ok\":true"));
}

#[test]
fn test_list_has_more() {
    // Clear stale index to ensure full scan works
    let _ = ov().args(["index", "clear"]).assert();
    ov().args(["list", "--limit", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"has_more\":true"));
}

// ─── read ────────────────────────────────────────────────────────────────

#[test]
fn test_read_exact() {
    ov().args(["read", "--note", "docker"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Docker"))
        .stdout(predicate::str::contains("#devops"));
}

#[test]
fn test_read_fuzzy() {
    ov().args(["read", "--note", "kube", "--fuzzy"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Kubernetes"));
}

#[test]
fn test_read_exact_no_fuzzy() {
    // "kube" should NOT match without --fuzzy
    ov().args(["read", "--note", "kube"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("NOTE_NOT_FOUND"));
}

#[test]
fn test_read_raw() {
    ov().args(["read", "--note", "docker", "--raw"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Docker is a containerization"));
}

#[test]
fn test_read_not_found() {
    ov().args(["read", "--note", "nonexistent_note_xyz"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("NOTE_NOT_FOUND"));
}

#[test]
fn test_read_json_input() {
    ov().args(["read", "--json", r#"{"note":"docker"}"#])
        .assert()
        .success()
        .stdout(predicate::str::contains("Docker"));
}

// ─── tags ────────────────────────────────────────────────────────────────

#[test]
fn test_tags_json() {
    ov().args(["tags"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#devops"))
        .stdout(predicate::str::contains("#kubernetes"));
}

#[test]
fn test_tags_sort_name() {
    ov().args(["tags", "--sort", "name"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ok\":true"));
}

#[test]
fn test_tags_min_count() {
    ov().args(["tags", "--min-count", "2"]).assert().success();
}

// ─── stats ───────────────────────────────────────────────────────────────

#[test]
fn test_stats_json() {
    ov().args(["stats"])
        .assert()
        .success()
        .stdout(predicate::str::contains("total_notes"))
        .stdout(predicate::str::contains("total_words"))
        .stdout(predicate::str::contains("unique_tags"));
}

// ─── links ───────────────────────────────────────────────────────────────

#[test]
fn test_links() {
    ov().args(["links", "--note", "kubernetes-basics"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Docker"))
        .stdout(predicate::str::contains("Container Networking"));
}

#[test]
fn test_backlinks() {
    ov().args(["backlinks", "--note", "docker"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("kubernetes-basics")
                .or(predicate::str::contains("container-networking")),
        );
}

#[test]
fn test_backlinks_with_context() {
    ov().args(["backlinks", "--note", "docker", "--context"])
        .assert()
        .success();
}

// ─── config ──────────────────────────────────────────────────────────────

#[test]
fn test_config_show() {
    ov().args(["config"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ok\":true"));
}

// ─── fields ──────────────────────────────────────────────────────────────

#[test]
fn test_fields_selector() {
    ov().args(["list", "--fields", "title,tags"])
        .assert()
        .success()
        .stdout(predicate::str::contains("title"));
}

// ─── vault not found ─────────────────────────────────────────────────────

#[test]
fn test_vault_not_found() {
    Command::cargo_bin("ov")
        .unwrap()
        .args(["--vault", "/nonexistent/path", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("VAULT_NOT_FOUND"));
}

// ─── index + search ─────────────────────────────────────────────────────

#[test]
fn test_index_build_and_search() {
    let _ = ov().args(["index", "clear"]).assert();

    ov().args(["index", "build"]).assert().success();

    ov().args(["index", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"exists\":true"));

    ov().args(["search", "--query", "kubernetes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("kubernetes"));

    ov().args(["search", "--query", "Docker", "--snippet"])
        .assert()
        .success();

    ov().args(["search", "--query", "tag:#devops container"])
        .assert()
        .success();

    // No results — still success, just empty data
    ov().args(["search", "--query", "xyznonexistent12345"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"count\":0"));

    let _ = ov().args(["index", "clear"]).assert();
}

// ─── graph ───────────────────────────────────────────────────────────────

#[test]
fn test_graph_json() {
    ov().args(["graph"])
        .assert()
        .success()
        .stdout(predicate::str::contains("nodes"))
        .stdout(predicate::str::contains("edges"));
}

#[test]
fn test_graph_center() {
    ov().args(["graph", "--center", "docker", "--depth", "1", "--fuzzy"])
        .assert()
        .success()
        .stdout(predicate::str::contains("docker"));
}

#[test]
fn test_graph_dot() {
    ov().args(["graph", "--graph-format", "dot"])
        .assert()
        .success()
        .stdout(predicate::str::contains("digraph vault"));
}

#[test]
fn test_graph_mermaid() {
    ov().args(["graph", "--graph-format", "mermaid"])
        .assert()
        .success()
        .stdout(predicate::str::contains("graph LR"));
}

// ─── daily ───────────────────────────────────────────────────────────────

#[test]
fn test_daily_dry_run() {
    ov().args(["daily", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("dry_run"));
}

#[test]
fn test_daily_existing() {
    ov().args(["daily", "--date", "2024-01-15"])
        .assert()
        .success()
        .stdout(predicate::str::contains("2024-01-15"));
}

// ─── create ──────────────────────────────────────────────────────────────

#[test]
fn test_create_and_read() {
    let path = vault_path().join("Daily/Test Note.md");
    let _ = std::fs::remove_file(&path);

    ov().args([
        "create",
        "--title",
        "Test Note",
        "--dir",
        "Daily",
        "--tags",
        "test,tmp",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("created"));

    ov().args(["read", "--note", "Test Note"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Test Note"));

    let _ = std::fs::remove_file(path);
}

#[test]
fn test_create_dry_run() {
    ov().args(["create", "--title", "DryRunTest", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("would_create"))
        .stdout(predicate::str::contains("dry_run"));
}

#[test]
fn test_create_if_not_exists() {
    // docker already exists in the fixture
    ov().args([
        "create",
        "--title",
        "docker",
        "--dir",
        "Zettelkasten",
        "--if-not-exists",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("skipped"));
}

#[test]
fn test_create_duplicate() {
    ov().args(["create", "--title", "docker", "--dir", "Zettelkasten"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("ALREADY_EXISTS"));
}

#[test]
fn test_create_json_input() {
    let path = vault_path().join("Daily/JsonCreateTest.md");
    let _ = std::fs::remove_file(&path);

    ov().args([
        "create",
        "--json",
        r#"{"title":"JsonCreateTest","dir":"Daily","tags":"test"}"#,
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("created"));

    let _ = std::fs::remove_file(path);
}

// ─── append ──────────────────────────────────────────────────────────────

#[test]
fn test_append_to_note() {
    let path = vault_path().join("Daily/AppendTest.md");
    let _ = std::fs::remove_file(&path);

    ov().args(["create", "--title", "AppendTest", "--dir", "Daily"])
        .assert()
        .success();

    ov().args([
        "append",
        "--note",
        "AppendTest",
        "--content",
        "New content line",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("appended"));

    ov().args(["read", "--note", "AppendTest", "--raw"])
        .assert()
        .success()
        .stdout(predicate::str::contains("New content line"));

    let _ = std::fs::remove_file(path);
}

#[test]
fn test_append_with_section() {
    let path = vault_path().join("People/AppendSectionTest.md");
    let _ = std::fs::remove_file(&path);

    ov().args([
        "create",
        "--title",
        "AppendSectionTest",
        "--template",
        "Person",
        "--dir",
        "People",
    ])
    .assert()
    .success();

    ov().args([
        "append",
        "--note",
        "AppendSectionTest",
        "--section",
        "Timeline",
        "--content",
        "Met at conference.",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("appended"));

    ov().args(["read", "--note", "AppendSectionTest", "--raw"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Met at conference."));

    let _ = std::fs::remove_file(path);
}

#[test]
fn test_append_with_date() {
    let path = vault_path().join("Daily/AppendDateTest.md");
    let _ = std::fs::remove_file(&path);

    ov().args(["create", "--title", "AppendDateTest", "--dir", "Daily"])
        .assert()
        .success();

    ov().args([
        "append",
        "--note",
        "AppendDateTest",
        "--date",
        "--content",
        "Dated entry.",
    ])
    .assert()
    .success();

    ov().args(["read", "--note", "AppendDateTest", "--raw"])
        .assert()
        .success()
        .stdout(predicate::str::contains("###"))
        .stdout(predicate::str::contains("Dated entry."));

    let _ = std::fs::remove_file(path);
}

#[test]
fn test_append_dry_run() {
    ov().args([
        "append",
        "--note",
        "docker",
        "--content",
        "test",
        "--dry-run",
        "--fuzzy",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("would_append"));
}

// ─── create with template ────────────────────────────────────────────────

#[test]
fn test_create_with_person_template() {
    let path = vault_path().join("People/TestPerson.md");
    let _ = std::fs::remove_file(&path);

    ov().args([
        "create",
        "--title",
        "TestPerson",
        "--template",
        "Person",
        "--dir",
        "People",
        "--vars",
        "org=imweb,role=SRE",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("created"));

    ov().args(["read", "--note", "TestPerson"])
        .assert()
        .success()
        .stdout(predicate::str::contains("person"))
        .stdout(predicate::str::contains("imweb"))
        .stdout(predicate::str::contains("SRE"))
        .stdout(predicate::str::contains("Timeline"));

    let _ = std::fs::remove_file(path);
}

// ─── frontmatter tests ──────────────────────────────────────────────────

#[test]
fn test_create_with_frontmatter() {
    let path = vault_path().join("Daily/FrontmatterTest.md");
    let _ = std::fs::remove_file(&path);

    ov().args([
        "create",
        "--title",
        "FrontmatterTest",
        "--dir",
        "Daily",
        "--frontmatter",
        r#"{"type":"article","status":"draft"}"#,
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("created"));

    ov().args(["read", "--note", "FrontmatterTest"])
        .assert()
        .success()
        .stdout(predicate::str::contains("article"))
        .stdout(predicate::str::contains("draft"));

    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_create_frontmatter_invalid_json() {
    ov().args([
        "create",
        "--title",
        "BadFm",
        "--frontmatter",
        "{broken json",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("INVALID_INPUT"));
}

#[test]
fn test_create_frontmatter_template_conflict() {
    ov().args([
        "create",
        "--title",
        "ConflictTest",
        "--frontmatter",
        r#"{"type":"note"}"#,
        "--template",
        "Person",
    ])
    .assert()
    .failure();
}

#[test]
fn test_create_frontmatter_with_tags() {
    let path = vault_path().join("Daily/FmTagsTest.md");
    let _ = std::fs::remove_file(&path);

    ov().args([
        "create",
        "--title",
        "FmTagsTest",
        "--dir",
        "Daily",
        "--frontmatter",
        r#"{"type":"note"}"#,
        "--tags",
        "devops,sre",
    ])
    .assert()
    .success();

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("#devops"), "should contain #devops tag");
    assert!(content.contains("#sre"), "should contain #sre tag");
    assert!(content.contains("type: note"), "should contain type field");

    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_create_frontmatter_with_sections() {
    let path = vault_path().join("Daily/FmSectionTest.md");
    let _ = std::fs::remove_file(&path);

    ov().args([
        "create",
        "--title",
        "FmSectionTest",
        "--dir",
        "Daily",
        "--frontmatter",
        r#"{"type":"note"}"#,
        "--sections",
        "Summary,Notes",
    ])
    .assert()
    .success();

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("---"), "should have YAML frontmatter");
    assert!(
        content.contains("## Summary"),
        "should have Summary section"
    );
    assert!(content.contains("## Notes"), "should have Notes section");

    let _ = std::fs::remove_file(&path);
}

// ─── sections and content ────────────────────────────────────────────────

#[test]
fn test_create_with_sections_only() {
    let path = vault_path().join("Daily/SectionsTest.md");
    let _ = std::fs::remove_file(&path);

    ov().args([
        "create",
        "--title",
        "SectionsTest",
        "--dir",
        "Daily",
        "--sections",
        "Summary,References",
    ])
    .assert()
    .success();

    ov().args(["read", "--note", "SectionsTest", "--raw"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Summary"))
        .stdout(predicate::str::contains("## References"));

    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_create_with_content_only() {
    let path = vault_path().join("Daily/ContentTest.md");
    let _ = std::fs::remove_file(&path);

    ov().args([
        "create",
        "--title",
        "ContentTest",
        "--dir",
        "Daily",
        "--content",
        "This is initial content.",
    ])
    .assert()
    .success();

    ov().args(["read", "--note", "ContentTest", "--raw"])
        .assert()
        .success()
        .stdout(predicate::str::contains("This is initial content."));

    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_create_template_with_sections() {
    let path = vault_path().join("People/TemplateSectionsTest.md");
    let _ = std::fs::remove_file(&path);

    ov().args([
        "create",
        "--title",
        "TemplateSectionsTest",
        "--dir",
        "People",
        "--template",
        "Person",
        "--sections",
        "Extra Notes,Follow-ups",
    ])
    .assert()
    .success();

    ov().args(["read", "--note", "TemplateSectionsTest", "--raw"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Timeline"))
        .stdout(predicate::str::contains("## Extra Notes"))
        .stdout(predicate::str::contains("## Follow-ups"));

    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_create_template_with_content() {
    let path = vault_path().join("People/TemplateContentTest.md");
    let _ = std::fs::remove_file(&path);

    ov().args([
        "create",
        "--title",
        "TemplateContentTest",
        "--dir",
        "People",
        "--template",
        "Person",
        "--content",
        "Extra note appended after template.",
    ])
    .assert()
    .success();

    ov().args(["read", "--note", "TemplateContentTest", "--raw"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Extra note appended after template.",
        ));

    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_create_sections_with_content() {
    let path = vault_path().join("Daily/SectionContentTest.md");
    let _ = std::fs::remove_file(&path);

    ov().args([
        "create",
        "--title",
        "SectionContentTest",
        "--dir",
        "Daily",
        "--sections",
        "Summary",
        "--content",
        "Body text here.",
    ])
    .assert()
    .success();

    ov().args(["read", "--note", "SectionContentTest", "--raw"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Summary"))
        .stdout(predicate::str::contains("Body text here."));

    let _ = std::fs::remove_file(&path);
}

// ─── title sanitization ─────────────────────────────────────────────────

#[test]
fn test_create_title_with_slash_rejected() {
    ov().args(["create", "--title", "bad/title"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("INVALID_INPUT"));
}

#[test]
fn test_create_title_dotdot_rejected() {
    ov().args(["create", "--title", ".."])
        .assert()
        .failure()
        .stderr(predicate::str::contains("INVALID_INPUT"));
}

#[test]
fn test_create_title_md_extension_stripped() {
    let path = vault_path().join("Daily/StripMdTest.md");
    let _ = std::fs::remove_file(&path);

    ov().args(["create", "--title", "StripMdTest.md", "--dir", "Daily"])
        .assert()
        .success()
        .stdout(predicate::str::contains("StripMdTest"));

    assert!(path.exists(), "StripMdTest.md should exist");
    assert!(
        !vault_path().join("Daily/StripMdTest.md.md").exists(),
        "StripMdTest.md.md should NOT exist"
    );

    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_create_path_traversal_blocked() {
    ov().args(["create", "--title", "EscapeTest", "--dir", "../../tmp"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("INVALID_INPUT"));
}

#[test]
fn test_create_template_not_found() {
    ov().args([
        "create",
        "--title",
        "TemplateNotFoundTest",
        "--template",
        "NonExistentTemplate",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("NOTE_NOT_FOUND"));
}

// ─── search type prefix ─────────────────────────────────────────────────

#[test]
fn test_search_type_prefix() {
    let _ = ov().args(["index", "clear"]).assert();
    ov().args(["index", "build"]).assert().success();

    ov().args(["search", "--query", "type:person"])
        .assert()
        .success()
        .stdout(predicate::str::contains("김철수"));
}

// ─── read person note ────────────────────────────────────────────────────

#[test]
fn test_read_person_note() {
    ov().args(["read", "--note", "김철수"])
        .assert()
        .success()
        .stdout(predicate::str::contains("person"))
        .stdout(predicate::str::contains("imweb"));
}

// ─── schema introspection ────────────────────────────────────────────────

#[test]
fn test_schema_commands() {
    ov().args(["schema", "commands"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\":\"list\""))
        .stdout(predicate::str::contains("has_side_effects"));
}

#[test]
fn test_schema_describe() {
    ov().args(["schema", "describe", "--command", "create"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\":\"create\""))
        .stdout(predicate::str::contains("input"))
        .stdout(predicate::str::contains("output"))
        .stdout(predicate::str::contains("examples"));
}

#[test]
fn test_schema_skill() {
    ov().args(["schema", "skill"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Invariants"))
        .stdout(predicate::str::contains("--dry-run"));
}

// ─── structured errors ──────────────────────────────────────────────────

#[test]
fn test_error_has_code_and_hint() {
    ov().args(["read", "--note", "totally_nonexistent_note_xyz"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("\"code\":\"NOTE_NOT_FOUND\""))
        .stderr(predicate::str::contains("\"hint\":"));
}

// ─── JSONL mode ──────────────────────────────────────────────────────────

#[test]
fn test_jsonl_output() {
    ov().args(["--jsonl", "list", "--limit", "3"])
        .assert()
        .success()
        // JSONL should NOT have the wrapper
        .stdout(predicate::str::contains("\"ok\":true").not())
        // Each line should be a valid JSON object
        .stdout(predicate::str::contains("\"title\":"));
}

// ─── missing required field ──────────────────────────────────────────────

#[test]
fn test_missing_required_field() {
    ov().args(["read"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("MISSING_FIELD"));
}

#[test]
fn test_create_missing_title() {
    ov().args(["create"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("MISSING_FIELD"));
}

// ─── P1: Clippings false positive fix ────────────────────────────────────

#[test]
fn test_clippings_false_positive_body_author() {
    // A StandardYaml note with "author:" in body should NOT be classified as Clippings
    ov().args(["read", "--note", "yaml-with-author-body"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"format\":\"standard_yaml\""))
        .stdout(predicate::str::contains("Design Patterns"));
}

// ─── P1: Stats source field ─────────────────────────────────────────────

#[test]
fn test_stats_source_field() {
    // Clear index to force full_scan path
    let _ = ov().args(["index", "clear"]).assert();
    ov().args(["stats"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"source\":\"full_scan\""));
}

#[test]
fn test_stats_skipped_files_field() {
    let _ = ov().args(["index", "clear"]).assert();
    ov().args(["stats"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"skipped_files\":"));
}

// ─── P2: Aliases parsing ────────────────────────────────────────────────

#[test]
fn test_aliases_parsed() {
    ov().args(["read", "--note", "yaml-with-author-body"])
        .assert()
        .success()
        .stdout(predicate::str::contains("GoF Patterns"))
        .stdout(predicate::str::contains("DP"));
}

// ─── P2: Read --section ─────────────────────────────────────────────────

#[test]
fn test_read_section() {
    ov().args(["read", "--note", "김철수", "--section", "Timeline"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"section\":\"Timeline\""))
        .stdout(predicate::str::contains("온보딩 미팅"));
}

#[test]
fn test_read_section_raw() {
    ov().args(["read", "--note", "김철수", "--section", "Timeline", "--raw"])
        .assert()
        .success()
        .stdout(predicate::str::contains("온보딩 미팅"))
        // raw mode should NOT output JSON wrapping
        .stdout(predicate::str::contains("\"ok\":true").not());
}

#[test]
fn test_read_section_not_found() {
    // Requesting a non-existent section should return null body
    ov().args([
        "read",
        "--note",
        "김철수",
        "--section",
        "NonExistentSection",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("\"body\":null"));
}

// ─── P2: Schema section field ───────────────────────────────────────────

#[test]
fn test_schema_describe_read_has_section() {
    ov().args(["schema", "describe", "--command", "read"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\":\"section\""));
}

// ─── P2: Search has_more_accurate ───────────────────────────────────────

#[test]
fn test_search_has_more_accurate() {
    let _ = ov().args(["index", "clear"]).assert();
    ov().args(["index", "build"]).assert().success();

    ov().args(["search", "--query", "kubernetes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("has_more_accurate"));

    let _ = ov().args(["index", "clear"]).assert();
}
