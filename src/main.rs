mod cli;
mod config;
mod error;
mod extract;
mod index;
mod model;
mod output;
mod search;
mod service;
mod vault;

use std::io::{Read, Write};
use std::process;

use clap::Parser;
use serde::de::DeserializeOwned;

use cli::{Cli, Command};
use config::paths;
use error::OvError;
use output::json::ErrorResponse;
use vault::Vault;

fn main() {
    let cli = Cli::parse();
    config::update_check::maybe_notify_update();

    if let Err(e) = run(cli) {
        let err_resp = ErrorResponse::from_error(&e);
        eprintln!("{}", err_resp.to_json_string());
        process::exit(e.exit_code());
    }
}

/// Shared context extracted from Cli for command handlers
struct Ctx {
    vault: Option<String>,
    jsonl: bool,
    fields: Option<String>,
}

impl From<&Cli> for Ctx {
    fn from(cli: &Cli) -> Self {
        Self {
            vault: cli.vault.clone(),
            jsonl: cli.jsonl,
            fields: cli.fields.clone(),
        }
    }
}

/// Parse JSON input string into the specified Args type
fn parse_json_input<T: DeserializeOwned>(json_str: &str) -> Result<T, OvError> {
    let sanitized = sanitize_input(json_str);
    serde_json::from_str(&sanitized)
        .map_err(|e| OvError::InvalidInput(format!("Invalid JSON payload: {e}")))
}

/// Strip control characters (U+0000..U+001F except \n, \r, \t) from input
fn sanitize_input(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t')
        .collect()
}

/// Require a field from an Option, returning MissingField error if None
fn require_field<T>(value: Option<T>, field_name: &str) -> Result<T, OvError> {
    value.ok_or_else(|| OvError::MissingField(field_name.to_string()))
}

fn run(cli: Cli) -> Result<(), OvError> {
    let ctx = Ctx::from(&cli);
    let json_input = cli.json.as_deref();

    match cli.command {
        Command::List(args) => {
            let args = merge_or_use(json_input, args)?;
            cmd_list(&ctx, args)
        }
        Command::Read(args) => {
            let args = merge_or_use(json_input, args)?;
            cmd_read(&ctx, args)
        }
        Command::Tags(args) => {
            let args = merge_or_use(json_input, args)?;
            cmd_tags(&ctx, args)
        }
        Command::Stats(_) => cmd_stats(&ctx),
        Command::Links(args) => {
            let args = merge_or_use(json_input, args)?;
            cmd_links(&ctx, args)
        }
        Command::Backlinks(args) => {
            let args = merge_or_use(json_input, args)?;
            cmd_backlinks(&ctx, args)
        }
        Command::Config(args) => {
            let args = merge_or_use(json_input, args)?;
            cmd_config(&ctx, args)
        }
        Command::Search(args) => {
            let args = merge_or_use(json_input, args)?;
            cmd_search(&ctx, args)
        }
        Command::Graph(args) => {
            let args = merge_or_use(json_input, args)?;
            cmd_graph(&ctx, args)
        }
        Command::Daily(args) => {
            let args = merge_or_use(json_input, args)?;
            cmd_daily(&ctx, args)
        }
        Command::Create(args) => {
            let args = merge_or_use(json_input, args)?;
            cmd_create(&ctx, args)
        }
        Command::Append(args) => {
            let args = merge_or_use(json_input, args)?;
            cmd_append(&ctx, args)
        }
        Command::Index(args) => cmd_index(&ctx, args),
        Command::Schema(args) => cmd_schema(&ctx, args),
    }
}

/// If --json is provided, parse it; otherwise use clap-parsed args
fn merge_or_use<T: DeserializeOwned>(json_input: Option<&str>, clap_args: T) -> Result<T, OvError> {
    match json_input {
        Some(json_str) => parse_json_input(json_str),
        None => Ok(clap_args),
    }
}

fn open_vault(ctx: &Ctx) -> Result<Vault, OvError> {
    let vault_path = paths::resolve_vault_path(ctx.vault.as_deref())?;
    Vault::open(vault_path)
}

// ─── list ────────────────────────────────────────────────────────────────

fn cmd_list(ctx: &Ctx, args: cli::list::ListArgs) -> Result<(), OvError> {
    let vault_path = paths::resolve_vault_path(ctx.vault.as_deref())?;
    let params = service::ListParams {
        dir: args.dir,
        tag: args.tag,
        date: args.date,
        sort: args.sort,
        reverse: args.reverse,
        limit: args.limit,
        offset: args.offset,
    };

    let result = if let Some(summaries) = index::reader::read_all_from_index(&vault_path) {
        service::list_summaries(&summaries, &params)
    } else {
        let vault = Vault::open(vault_path)?;
        service::list_notes(vault.notes(), &params)
    };

    let meta = vec![
        ("total", serde_json::json!(result.total)),
        ("offset", serde_json::json!(args.offset)),
        ("limit", serde_json::json!(args.limit)),
        (
            "has_more",
            serde_json::json!(args.offset + result.notes.len() < result.total),
        ),
    ];
    output::print_with_meta(
        &result.notes,
        result.notes.len(),
        ctx.jsonl,
        &ctx.fields,
        meta,
    );

    Ok(())
}

// ─── read ────────────────────────────────────────────────────────────────

fn cmd_read(ctx: &Ctx, args: cli::read::ReadArgs) -> Result<(), OvError> {
    let note_name = require_field(args.note.as_deref().map(String::from), "note")?;
    let vault = open_vault(ctx)?;
    let file_path = vault.resolve_note_with_mode(&note_name, args.fuzzy)?;
    let relative = vault.relative_path(&file_path);
    let note = vault.read_note(&relative)?;

    // --section: extract a specific section only
    if let Some(ref section_name) = args.section {
        let section_body = note
            .body
            .as_deref()
            .and_then(|body| vault::extract_section(body, section_name));

        if args.raw {
            if let Some(ref body) = section_body {
                print!("{body}");
            }
            return Ok(());
        }

        let data = serde_json::json!({
            "title": note.title,
            "path": note.path,
            "section": section_name,
            "body": section_body,
        });
        output::print_output(&data, 1, ctx.jsonl, &ctx.fields);
        return Ok(());
    }

    if args.raw {
        if let Some(ref body) = note.body {
            print!("{body}");
        }
        return Ok(());
    }

    let mut note_output = note;
    if args.no_body {
        note_output.body = None;
    }
    output::print_output(note_output, 1, ctx.jsonl, &ctx.fields);

    Ok(())
}

// ─── tags ────────────────────────────────────────────────────────────────

fn cmd_tags(ctx: &Ctx, args: cli::tags::TagsArgs) -> Result<(), OvError> {
    let vault_path = paths::resolve_vault_path(ctx.vault.as_deref())?;
    let params = service::TagsParams {
        sort: args.sort,
        min_count: args.min_count,
        limit: args.limit,
    };

    let summaries = if let Some(idx_summaries) = index::reader::read_all_from_index(&vault_path) {
        service::aggregate_tags_from_summaries(&idx_summaries, &params)
    } else {
        let vault = Vault::open(vault_path)?;
        service::aggregate_tags(vault.notes(), &params)
    };

    let count = summaries.len();
    output::print_output(summaries, count, ctx.jsonl, &ctx.fields);

    Ok(())
}

// ─── stats ───────────────────────────────────────────────────────────────

fn cmd_stats(ctx: &Ctx) -> Result<(), OvError> {
    let vault_path = paths::resolve_vault_path(ctx.vault.as_deref())?;

    let stats = if let Some(idx_summaries) = index::reader::read_all_from_index(&vault_path) {
        let vault = Vault::open(vault_path)?;
        service::compute_stats_from_summaries(vault.directories(), &idx_summaries)
    } else {
        let vault = Vault::open(vault_path)?;
        service::compute_stats(&vault, vault.notes())
    };

    output::print_output(&stats, 1, ctx.jsonl, &ctx.fields);

    Ok(())
}

// ─── links ───────────────────────────────────────────────────────────────

fn cmd_links(ctx: &Ctx, args: cli::links::LinksArgs) -> Result<(), OvError> {
    let note_name = require_field(args.note, "note")?;
    let vault = open_vault(ctx)?;
    let file_path = vault.resolve_note_with_mode(&note_name, args.fuzzy)?;
    let relative = vault.relative_path(&file_path);
    let note = vault.read_note(&relative)?;

    let count = note.links.len();
    output::print_output(&note.links, count, ctx.jsonl, &ctx.fields);

    Ok(())
}

// ─── backlinks ───────────────────────────────────────────────────────────

fn cmd_backlinks(ctx: &Ctx, args: cli::links::BacklinksArgs) -> Result<(), OvError> {
    let note_name = require_field(args.note, "note")?;
    let vault = open_vault(ctx)?;
    let file_path = vault.resolve_note_with_mode(&note_name, args.fuzzy)?;
    let target_stem = file_path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let backlinks = service::find_backlinks(&vault.root, &target_stem, vault.notes(), args.context);

    let count = backlinks.len();
    output::print_output(&backlinks, count, ctx.jsonl, &ctx.fields);

    Ok(())
}

// ─── config ──────────────────────────────────────────────────────────────

fn cmd_config(ctx: &Ctx, args: cli::config::ConfigArgs) -> Result<(), OvError> {
    let mut app_config = config::AppConfig::load()?;

    match (args.key.as_deref(), args.value.as_deref()) {
        (None, None) => {
            let json = serde_json::to_value(&app_config).unwrap_or_default();
            output::print_output(&json, 1, ctx.jsonl, &ctx.fields);
        }
        (Some(key), None) => match key {
            "vault_path" => {
                let value = app_config.vault_path.as_deref().unwrap_or("");
                let data = serde_json::json!({ "key": key, "value": value });
                output::print_output(&data, 1, ctx.jsonl, &ctx.fields);
            }
            "vaults" => {
                let vaults: Vec<serde_json::Value> = paths::discover_vaults()
                    .iter()
                    .map(|p| {
                        serde_json::json!({
                            "path": p.to_string_lossy(),
                            "name": p.file_name().unwrap_or_default().to_string_lossy(),
                        })
                    })
                    .collect();
                let count = vaults.len();
                output::print_output(&vaults, count, ctx.jsonl, &ctx.fields);
            }
            _ => {
                return Err(OvError::InvalidInput(format!("Unknown config key: {key}")));
            }
        },
        (Some(key), Some(value)) => {
            match key {
                "vault_path" => app_config.vault_path = Some(value.to_string()),
                _ => {
                    return Err(OvError::InvalidInput(format!("Unknown config key: {key}")));
                }
            }
            app_config.save()?;
            let data = serde_json::json!({
                "action": "updated",
                "key": key,
                "value": value,
            });
            output::print_output(&data, 1, ctx.jsonl, &ctx.fields);
        }
        (None, Some(_)) => {
            return Err(OvError::InvalidInput("value requires key".to_string()));
        }
    }

    Ok(())
}

// ─── search ──────────────────────────────────────────────────────────────

fn cmd_search(ctx: &Ctx, args: cli::search::SearchArgs) -> Result<(), OvError> {
    let query = require_field(args.query, "query")?;
    let vault_path = paths::resolve_vault_path(ctx.vault.as_deref())?;
    let search_result = search::search(&vault_path, &query, args.limit, args.offset, args.snippet)?;
    let mut results = search_result.hits;

    // search returns limit+1 to detect has_more
    let has_more = results.len() > args.limit;
    if has_more {
        results.truncate(args.limit);
    }

    let count = results.len();
    let meta = vec![
        ("offset", serde_json::json!(args.offset)),
        ("limit", serde_json::json!(args.limit)),
        ("has_more", serde_json::json!(has_more)),
        (
            "has_more_accurate",
            serde_json::json!(!search_result.window_exhausted),
        ),
    ];
    output::print_with_meta(&results, count, ctx.jsonl, &ctx.fields, meta);

    Ok(())
}

// ─── index ───────────────────────────────────────────────────────────────

fn cmd_index(ctx: &Ctx, args: cli::index::IndexArgs) -> Result<(), OvError> {
    match args.action {
        cli::index::IndexAction::Build => {
            let vault = open_vault(ctx)?;
            let result = index::writer::build_index(&vault, false)?;
            let link_idx = index::link_index::LinkIndex::build(vault.notes());
            link_idx.save(&vault.root)?;

            let data = serde_json::json!({
                "action": "built",
                "indexed": result.indexed,
                "skipped": result.skipped,
                "total": result.total,
                "elapsed_ms": result.elapsed_ms,
            });
            output::print_output(&data, 1, ctx.jsonl, &ctx.fields);
        }
        cli::index::IndexAction::Status => {
            let vault_path = paths::resolve_vault_path(ctx.vault.as_deref())?;
            let status = index::writer::index_status(&vault_path)?;
            output::print_output(&status, 1, ctx.jsonl, &ctx.fields);
        }
        cli::index::IndexAction::Clear => {
            let vault_path = paths::resolve_vault_path(ctx.vault.as_deref())?;
            index::writer::clear_index(&vault_path)?;
            let data = serde_json::json!({ "action": "cleared" });
            output::print_output(&data, 1, ctx.jsonl, &ctx.fields);
        }
    }

    Ok(())
}

// ─── graph ───────────────────────────────────────────────────────────────

fn cmd_graph(ctx: &Ctx, args: cli::graph::GraphArgs) -> Result<(), OvError> {
    let vault = open_vault(ctx)?;
    let notes = vault.notes();
    let link_idx = index::link_index::LinkIndex::build(notes);

    if let Some(ref center) = args.center {
        let resolved = vault.resolve_note_with_mode(center, args.fuzzy)?;
        let stem = resolved
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let (nodes, edges) = link_idx.subgraph(&stem, args.depth);

        match args.graph_format.as_str() {
            "dot" => {
                let dot_str = index::link_index::to_dot(&nodes, &edges);
                let data = serde_json::json!({ "format": "dot", "content": dot_str });
                output::print_output(&data, nodes.len(), ctx.jsonl, &ctx.fields);
            }
            "mermaid" => {
                let mermaid_str = index::link_index::to_mermaid(&nodes, &edges);
                let data = serde_json::json!({ "format": "mermaid", "content": mermaid_str });
                output::print_output(&data, nodes.len(), ctx.jsonl, &ctx.fields);
            }
            _ => {
                let data = serde_json::json!({
                    "center": stem,
                    "depth": args.depth,
                    "nodes": nodes,
                    "edges": edges.iter().map(|(s, t)| serde_json::json!({"source": s, "target": t})).collect::<Vec<_>>(),
                });
                output::print_output(&data, nodes.len(), ctx.jsonl, &ctx.fields);
            }
        }
    } else {
        let graph = link_idx.to_graph(notes);

        match args.graph_format.as_str() {
            "dot" => {
                let nodes: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
                let edges: Vec<(String, String)> = graph
                    .edges
                    .iter()
                    .map(|e| (e.source.clone(), e.target.clone()))
                    .collect();
                let dot_str = index::link_index::to_dot(&nodes, &edges);
                let data = serde_json::json!({ "format": "dot", "content": dot_str });
                output::print_output(&data, graph.nodes.len(), ctx.jsonl, &ctx.fields);
            }
            "mermaid" => {
                let nodes: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
                let edges: Vec<(String, String)> = graph
                    .edges
                    .iter()
                    .map(|e| (e.source.clone(), e.target.clone()))
                    .collect();
                let mermaid_str = index::link_index::to_mermaid(&nodes, &edges);
                let data = serde_json::json!({ "format": "mermaid", "content": mermaid_str });
                output::print_output(&data, graph.nodes.len(), ctx.jsonl, &ctx.fields);
            }
            _ => {
                output::print_output(&graph, graph.nodes.len(), ctx.jsonl, &ctx.fields);
            }
        }
    }

    Ok(())
}

// ─── daily ───────────────────────────────────────────────────────────────

fn cmd_daily(ctx: &Ctx, args: cli::daily::DailyArgs) -> Result<(), OvError> {
    let vault = open_vault(ctx)?;

    let date = if let Some(ref d) = args.date {
        d.clone()
    } else {
        chrono::Local::now().format("%Y-%m-%d").to_string()
    };

    let daily_dir = "Daily";
    let filename = format!("{date}.md");
    let relative = format!("{daily_dir}/{filename}");
    let full_path = vault.root.join(&relative);

    if full_path.exists() {
        let note = vault.read_note(&relative)?;
        output::print_output(note, 1, ctx.jsonl, &ctx.fields);
    } else if args.dry_run {
        let data = serde_json::json!({
            "action": "would_create",
            "path": relative,
            "date": date,
            "dry_run": true,
        });
        output::print_output(&data, 1, ctx.jsonl, &ctx.fields);
    } else {
        let dir_path = vault.root.join(daily_dir);
        std::fs::create_dir_all(&dir_path)?;

        let content = format!("# {date}\n\n## Notes\n\n");
        std::fs::write(&full_path, &content)?;

        let data = serde_json::json!({
            "action": "created",
            "path": relative,
            "date": date,
        });
        output::print_output(&data, 1, ctx.jsonl, &ctx.fields);
    }

    Ok(())
}

// ─── create ──────────────────────────────────────────────────────────────

/// Truncate content at a char boundary (safe for multibyte UTF-8)
fn truncate_content(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

fn sanitize_title(title: &str) -> Result<String, OvError> {
    if title.contains('\0') {
        return Err(OvError::InvalidInput(
            "Title contains null byte".to_string(),
        ));
    }
    if title.contains('/') || title.contains('\\') {
        return Err(OvError::InvalidInput(
            "Title cannot contain path separators (/ or \\)".to_string(),
        ));
    }
    if title == "." || title == ".." {
        return Err(OvError::InvalidInput(
            "Title cannot be '.' or '..'".to_string(),
        ));
    }
    let clean = title.strip_suffix(".md").unwrap_or(title);
    if clean.is_empty() {
        return Err(OvError::InvalidInput("Title cannot be empty".to_string()));
    }
    let filename = format!("{clean}.md");
    if filename.len() > 255 {
        return Err(OvError::InvalidInput(format!(
            "Filename too long ({} bytes, max 255): {filename}",
            filename.len()
        )));
    }
    Ok(clean.to_string())
}

fn validate_path_safety(
    vault_root: &std::path::Path,
    relative: &str,
    parent: &std::path::Path,
) -> Result<(), OvError> {
    let canonical_root = vault_root
        .canonicalize()
        .map_err(|e| OvError::General(format!("Cannot canonicalize vault root: {e}")))?;

    let mut normalized = canonical_root.clone();
    for component in std::path::Path::new(relative).components() {
        match component {
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::Normal(c) => {
                normalized.push(c);
            }
            _ => {}
        }
    }
    if !normalized.starts_with(&canonical_root) {
        return Err(OvError::InvalidInput(format!(
            "Path escapes vault boundary: {relative}"
        )));
    }

    if parent.exists() {
        let canonical_parent = parent.canonicalize().map_err(|e| {
            OvError::General(format!(
                "Cannot canonicalize parent {}: {e}",
                parent.display()
            ))
        })?;
        if !canonical_parent.starts_with(&canonical_root) {
            return Err(OvError::InvalidInput(format!(
                "Path escapes vault boundary (symlink): {relative}"
            )));
        }
    }

    Ok(())
}

fn append_sections(content: &mut String, sections_str: &str) {
    for heading in sections_str.split(',') {
        let heading = heading.trim();
        if !heading.is_empty() {
            content.push_str(&format!("\n## {heading}\n\n"));
        }
    }
}

fn append_body(content: &mut String, body: &str) {
    if body.is_empty() {
        return;
    }
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(body);
    if !body.ends_with('\n') {
        content.push('\n');
    }
}

fn atomic_write_new(path: &std::path::Path, content: &[u8], relative: &str) -> Result<(), OvError> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                OvError::AlreadyExists(relative.to_string())
            } else {
                OvError::General(format!("Cannot create {relative}: {e}"))
            }
        })?;

    if let Err(e) = file.write_all(content).and_then(|_| file.sync_all()) {
        let _ = std::fs::remove_file(path);
        return Err(OvError::General(format!("Failed to write {relative}: {e}")));
    }

    Ok(())
}

fn cmd_create(ctx: &Ctx, args: cli::create::CreateArgs) -> Result<(), OvError> {
    let title = require_field(args.title, "title")?;

    // Validate constraints that clap enforces for flag-based input but not for --json
    if args.template.is_some() && args.frontmatter.is_some() {
        return Err(OvError::InvalidInput(
            "template and frontmatter are mutually exclusive".to_string(),
        ));
    }
    if args.vars.is_some() && args.template.is_none() {
        return Err(OvError::InvalidInput("vars requires template".to_string()));
    }

    let vault = open_vault(ctx)?;

    let clean_title = sanitize_title(&title)?;

    let dir = args.dir.as_deref().unwrap_or_else(|| {
        vault
            .obsidian_config
            .new_file_location
            .as_deref()
            .unwrap_or("")
    });

    let filename = format!("{clean_title}.md");
    let relative = if dir.is_empty() {
        filename.clone()
    } else {
        format!("{dir}/{filename}")
    };
    let full_path = vault.root.join(&relative);

    // Validate path safety BEFORE any early returns (--if-not-exists, --dry-run)
    if let Some(parent) = full_path.parent() {
        validate_path_safety(&vault.root, &relative, parent)?;
    }

    // Handle --if-not-exists
    if args.if_not_exists && full_path.exists() {
        let data = serde_json::json!({
            "action": "skipped",
            "path": relative,
            "title": clean_title,
            "reason": "already_exists",
        });
        output::print_output(&data, 1, ctx.jsonl, &ctx.fields);
        return Ok(());
    }

    // Build content
    let mut content = String::new();

    if let Some(ref frontmatter_json) = args.frontmatter {
        let mut fm_map: std::collections::BTreeMap<String, serde_json::Value> =
            serde_json::from_str(frontmatter_json)
                .map_err(|e| OvError::InvalidInput(format!("Invalid frontmatter JSON: {e}")))?;

        if let Some(ref tags_str) = args.tags {
            let tag_values: Vec<serde_json::Value> = tags_str
                .split(',')
                .map(|t| {
                    let t = t.trim();
                    if t.starts_with('#') {
                        serde_json::Value::String(t.to_string())
                    } else {
                        serde_json::Value::String(format!("#{t}"))
                    }
                })
                .collect();

            match fm_map.get_mut("tags") {
                Some(serde_json::Value::Array(arr)) => {
                    arr.extend(tag_values);
                }
                Some(v) => {
                    *v = serde_json::Value::Array(tag_values);
                }
                None => {
                    fm_map.insert("tags".to_string(), serde_json::Value::Array(tag_values));
                }
            }
        }

        if !fm_map.is_empty() {
            let yaml_str =
                serde_yaml::to_string(&fm_map).map_err(|e| OvError::General(e.to_string()))?;
            content.push_str("---\n");
            content.push_str(&yaml_str);
            if !yaml_str.ends_with('\n') {
                content.push('\n');
            }
            content.push_str("---\n");
        }
    } else if let Some(ref template_name) = args.template {
        // Validate template name: reject path separators and traversal
        if template_name.contains('/')
            || template_name.contains('\\')
            || template_name.contains("..")
        {
            return Err(OvError::InvalidInput(
                "Template name cannot contain path separators or '..'".to_string(),
            ));
        }
        let template_dir = vault
            .obsidian_config
            .template_folder
            .as_deref()
            .unwrap_or("Templates");
        let template_path = vault
            .root
            .join(template_dir)
            .join(format!("{template_name}.md"));
        // Boundary check: ensure resolved template is within vault
        if let Ok(canonical_template) = template_path.canonicalize() {
            let canonical_root = vault
                .root
                .canonicalize()
                .unwrap_or_else(|_| vault.root.clone());
            if !canonical_template.starts_with(&canonical_root) {
                return Err(OvError::InvalidInput(format!(
                    "Template path escapes vault boundary: {template_name}"
                )));
            }
        }
        if !template_path.exists() {
            return Err(OvError::NoteNotFound(format!(
                "Template not found: {template_name}"
            )));
        }
        content = std::fs::read_to_string(&template_path)?;

        let now = chrono::Local::now();
        content = content.replace("{{date:YYYY-MM-DD}}", &now.format("%Y-%m-%d").to_string());
        content = content.replace("{{time:HH:mm}}", &now.format("%H:%M").to_string());
        content = content.replace("{{title}}", &clean_title);

        if let Some(ref vars_str) = args.vars {
            for pair in vars_str.split(',') {
                if let Some((k, v)) = pair.split_once('=') {
                    let k = k.trim();
                    let v = v.trim();
                    content = content.replace(&format!("{{{{{k}}}}}"), v);
                }
            }
        }

        use std::sync::OnceLock;
        static PLACEHOLDER_RE: OnceLock<regex::Regex> = OnceLock::new();
        let placeholder_re =
            PLACEHOLDER_RE.get_or_init(|| regex::Regex::new(r"\{\{[^}]+\}\}").unwrap());
        content = placeholder_re.replace_all(&content, "").to_string();
    } else {
        content.push_str(&format!("# {clean_title}\n\n"));
        if let Some(ref tags_str) = args.tags {
            let tags_formatted: Vec<String> = tags_str
                .split(',')
                .map(|t| {
                    let t = t.trim();
                    if t.starts_with('#') {
                        t.to_string()
                    } else {
                        format!("#{t}")
                    }
                })
                .collect();
            content.push_str(&format!("Tags: {}\n\n", tags_formatted.join(" ")));
        }
    }

    if let Some(ref sections_str) = args.sections {
        append_sections(&mut content, sections_str);
    }

    if let Some(ref body) = args.content {
        append_body(&mut content, body);
    }

    if args.stdin {
        let mut stdin_content = String::new();
        std::io::stdin().read_to_string(&mut stdin_content)?;
        content.push_str(&stdin_content);
    }

    // Dry-run: return what would be created
    if args.dry_run {
        let data = serde_json::json!({
            "action": "would_create",
            "path": relative,
            "title": clean_title,
            "content_preview": truncate_content(&content, 500),
            "content_length": content.len(),
            "dry_run": true,
        });
        output::print_output(&data, 1, ctx.jsonl, &ctx.fields);
        return Ok(());
    }

    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            OvError::General(format!("Cannot create directory {}: {e}", parent.display()))
        })?;
    }

    atomic_write_new(&full_path, content.as_bytes(), &relative)?;

    let data = serde_json::json!({
        "action": "created",
        "path": relative,
        "title": clean_title,
    });
    output::print_output(&data, 1, ctx.jsonl, &ctx.fields);

    Ok(())
}

// ─── append ─────────────────────────────────────────────────────────────

fn cmd_append(ctx: &Ctx, args: cli::append::AppendArgs) -> Result<(), OvError> {
    let note_name = require_field(args.note, "note")?;
    let vault = open_vault(ctx)?;
    let file_path = vault.resolve_note_with_mode(&note_name, args.fuzzy)?;
    let relative = vault.relative_path(&file_path);

    let mut new_content = String::new();
    if args.stdin {
        std::io::stdin().read_to_string(&mut new_content)?;
    } else if let Some(ref text) = args.content {
        new_content = text.clone();
    } else {
        return Err(OvError::MissingField(
            "content (either --content or --stdin is required)".to_string(),
        ));
    }

    if args.date {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        new_content = format!("### {today}\n{new_content}");
    }

    let mut file_content = std::fs::read_to_string(&file_path)?;

    if let Some(ref section) = args.section {
        let insert_pos = vault::find_section_insert_point(&file_content, section);
        let prefix = if insert_pos > 0 && !file_content[..insert_pos].ends_with("\n\n") {
            if file_content[..insert_pos].ends_with('\n') {
                "\n".to_string()
            } else {
                "\n\n".to_string()
            }
        } else {
            String::new()
        };
        let suffix =
            if insert_pos < file_content.len() && !file_content[insert_pos..].starts_with('\n') {
                "\n".to_string()
            } else {
                String::new()
            };
        file_content.insert_str(insert_pos, &format!("{prefix}{new_content}\n{suffix}"));
    } else {
        if !file_content.ends_with('\n') {
            file_content.push('\n');
        }
        file_content.push('\n');
        file_content.push_str(&new_content);
        if !new_content.ends_with('\n') {
            file_content.push('\n');
        }
    }

    // Dry-run: return what would be appended
    if args.dry_run {
        let data = serde_json::json!({
            "action": "would_append",
            "path": relative,
            "section": args.section,
            "content_length": new_content.len(),
            "dry_run": true,
        });
        output::print_output(&data, 1, ctx.jsonl, &ctx.fields);
        return Ok(());
    }

    std::fs::write(&file_path, &file_content)?;

    let data = serde_json::json!({
        "action": "appended",
        "path": relative,
        "section": args.section,
    });
    output::print_output(&data, 1, ctx.jsonl, &ctx.fields);

    Ok(())
}

// ─── schema ──────────────────────────────────────────────────────────────

fn cmd_schema(ctx: &Ctx, args: cli::schema::SchemaArgs) -> Result<(), OvError> {
    match args.action {
        cli::schema::SchemaAction::Commands => {
            let commands = schema_commands();
            let count = commands.len();
            output::print_output(&commands, count, ctx.jsonl, &ctx.fields);
        }
        cli::schema::SchemaAction::Describe(desc_args) => {
            let cmd_name = require_field(desc_args.command, "command")?;
            let desc = schema_describe(&cmd_name)?;
            output::print_output(&desc, 1, ctx.jsonl, &ctx.fields);
        }
        cli::schema::SchemaAction::Skill => {
            // Raw markdown output — intended for direct context injection, not JSON wrapping
            print!("{}", schema_skill());
        }
    }
    Ok(())
}

fn schema_commands() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "name": "list",
            "description": "List notes with filtering by dir/tag/date and sorting",
            "has_side_effects": false,
            "supports_dry_run": false,
            "supports_json_input": true
        }),
        serde_json::json!({
            "name": "read",
            "description": "Read a note by name. Default: exact match",
            "has_side_effects": false,
            "supports_dry_run": false,
            "supports_json_input": true
        }),
        serde_json::json!({
            "name": "search",
            "description": "Full-text search with tag:/in:/title:/date:/type: prefixes. Requires index",
            "has_side_effects": false,
            "supports_dry_run": false,
            "supports_json_input": true
        }),
        serde_json::json!({
            "name": "tags",
            "description": "List all tags with occurrence counts",
            "has_side_effects": false,
            "supports_dry_run": false,
            "supports_json_input": true
        }),
        serde_json::json!({
            "name": "stats",
            "description": "Show vault-wide statistics",
            "has_side_effects": false,
            "supports_dry_run": false,
            "supports_json_input": false
        }),
        serde_json::json!({
            "name": "links",
            "description": "Show outgoing [[wiki-links]] from a note",
            "has_side_effects": false,
            "supports_dry_run": false,
            "supports_json_input": true
        }),
        serde_json::json!({
            "name": "backlinks",
            "description": "Show incoming backlinks pointing to a note",
            "has_side_effects": false,
            "supports_dry_run": false,
            "supports_json_input": true
        }),
        serde_json::json!({
            "name": "graph",
            "description": "Explore the link graph (JSON, DOT, or Mermaid)",
            "has_side_effects": false,
            "supports_dry_run": false,
            "supports_json_input": true
        }),
        serde_json::json!({
            "name": "daily",
            "description": "Open or create today's daily note",
            "has_side_effects": true,
            "supports_dry_run": true,
            "supports_json_input": true
        }),
        serde_json::json!({
            "name": "create",
            "description": "Create a new note (plain, frontmatter, or template-based)",
            "has_side_effects": true,
            "supports_dry_run": true,
            "supports_json_input": true
        }),
        serde_json::json!({
            "name": "append",
            "description": "Append content to an existing note (section-aware)",
            "has_side_effects": true,
            "supports_dry_run": true,
            "supports_json_input": true
        }),
        serde_json::json!({
            "name": "index",
            "description": "Manage Tantivy search index (build/status/clear)",
            "has_side_effects": true,
            "supports_dry_run": false,
            "supports_json_input": false
        }),
        serde_json::json!({
            "name": "config",
            "description": "Get or set configuration values",
            "has_side_effects": true,
            "supports_dry_run": false,
            "supports_json_input": true
        }),
        serde_json::json!({
            "name": "schema",
            "description": "Introspect CLI schema — list commands, describe inputs/outputs",
            "has_side_effects": false,
            "supports_dry_run": false,
            "supports_json_input": false
        }),
    ]
}

fn schema_describe(cmd_name: &str) -> Result<serde_json::Value, OvError> {
    let desc = match cmd_name {
        "list" => serde_json::json!({
            "name": "list",
            "description": "List notes with filtering by dir/tag/date and sorting",
            "has_side_effects": false,
            "input": {
                "fields": [
                    {"name": "dir", "type": "string", "required": false, "description": "Filter by directory name"},
                    {"name": "tag", "type": "string", "required": false, "description": "Filter by tag (e.g., '#imweb')"},
                    {"name": "date", "type": "string", "required": false, "description": "Filter by modification date: YYYY-MM-DD or 'today'"},
                    {"name": "sort", "type": "string", "required": false, "default": "modified", "enum": ["title", "modified", "size", "words"], "description": "Sort field"},
                    {"name": "reverse", "type": "boolean", "required": false, "default": false, "description": "Reverse sort order"},
                    {"name": "limit", "type": "integer", "required": false, "default": 50, "description": "Max results to return"},
                    {"name": "offset", "type": "integer", "required": false, "default": 0, "description": "Skip first N results"}
                ]
            },
            "output": {
                "type": "array",
                "fields": ["title", "path", "dir", "tags", "modified", "word_count", "link_count", "evicted"],
                "meta": ["total", "offset", "limit", "has_more"]
            },
            "examples": [
                {"description": "Recent notes in Zettelkasten", "json": "{\"dir\":\"Zettelkasten\",\"limit\":10}"},
                {"description": "Notes tagged #k8s", "json": "{\"tag\":\"#k8s\"}"},
                {"description": "Notes modified today", "json": "{\"date\":\"today\"}"}
            ]
        }),
        "read" => serde_json::json!({
            "name": "read",
            "description": "Read a note by name. Exact match by default, use fuzzy flag for fuzzy matching",
            "has_side_effects": false,
            "input": {
                "fields": [
                    {"name": "note", "type": "string", "required": true, "description": "Note name or path"},
                    {"name": "fuzzy", "type": "boolean", "required": false, "default": false, "description": "Enable fuzzy matching"},
                    {"name": "no_body", "type": "boolean", "required": false, "default": false, "description": "Exclude body content from output"},
                    {"name": "raw", "type": "boolean", "required": false, "default": false, "description": "Output raw body text only (bypasses JSON — for piping/context injection)"},
                    {"name": "section", "type": "string", "required": false, "description": "Extract only a specific section by heading name"}
                ]
            },
            "output": {
                "type": "object",
                "fields": ["title", "path", "dir", "frontmatter", "tags", "links", "headings", "word_count", "file_meta", "body"],
                "section_mode": {
                    "description": "When --section is used, output shape changes to: {title, path, section, body}",
                    "fields": ["title", "path", "section", "body"]
                }
            },
            "examples": [
                {"description": "Read a note exactly", "json": "{\"note\":\"ElasticSearch\"}"},
                {"description": "Read with fuzzy matching", "json": "{\"note\":\"elastic\",\"fuzzy\":true}"},
                {"description": "Metadata only", "json": "{\"note\":\"ElasticSearch\",\"no_body\":true}"},
                {"description": "Read specific section", "json": "{\"note\":\"ElasticSearch\",\"section\":\"Timeline\"}"}
            ]
        }),
        "search" => serde_json::json!({
            "name": "search",
            "description": "Full-text search with prefix filters. Requires: ov index build",
            "has_side_effects": false,
            "input": {
                "fields": [
                    {"name": "query", "type": "string", "required": true, "description": "Search query. Supports prefixes: tag:#X, in:Dir, title:X, date:YYYY, type:X"},
                    {"name": "snippet", "type": "boolean", "required": false, "default": false, "description": "Show text snippet around matches"},
                    {"name": "limit", "type": "integer", "required": false, "default": 20, "description": "Max results"},
                    {"name": "offset", "type": "integer", "required": false, "default": 0, "description": "Skip first N results"}
                ]
            },
            "output": {
                "type": "array",
                "fields": ["title", "path", "tags", "score", "snippet"],
                "meta": ["offset", "limit", "has_more", "has_more_accurate"]
            },
            "examples": [
                {"description": "Keyword search", "json": "{\"query\":\"kubernetes\"}"},
                {"description": "Tag + directory filter", "json": "{\"query\":\"tag:#k8s in:Zettelkasten\",\"snippet\":true}"},
                {"description": "Type filter", "json": "{\"query\":\"type:troubleshooting\",\"limit\":5}"}
            ]
        }),
        "tags" => serde_json::json!({
            "name": "tags",
            "description": "List all tags with occurrence counts",
            "has_side_effects": false,
            "input": {
                "fields": [
                    {"name": "sort", "type": "string", "required": false, "default": "count", "enum": ["count", "name"], "description": "Sort field"},
                    {"name": "limit", "type": "integer", "required": false, "description": "Max tags to return"},
                    {"name": "min_count", "type": "integer", "required": false, "description": "Min occurrences filter"}
                ]
            },
            "output": {
                "type": "array",
                "fields": ["tag", "count", "notes"]
            },
            "examples": [
                {"description": "Top 10 tags", "json": "{\"limit\":10}"},
                {"description": "Tags with 5+ uses", "json": "{\"min_count\":5}"}
            ]
        }),
        "stats" => serde_json::json!({
            "name": "stats",
            "description": "Show vault-wide statistics",
            "has_side_effects": false,
            "input": {"fields": []},
            "output": {
                "type": "object",
                "fields": ["total_notes", "total_words", "total_links", "unique_tags", "directories", "total_size_bytes", "total_size_mb", "skipped_files", "avg_words_per_note", "avg_links_per_note", "top_tags", "directory_list", "source"]
            },
            "examples": [
                {"description": "Get vault stats", "cli": "ov stats"}
            ]
        }),
        "links" => serde_json::json!({
            "name": "links",
            "description": "Show outgoing [[wiki-links]] from a note",
            "has_side_effects": false,
            "input": {
                "fields": [
                    {"name": "note", "type": "string", "required": true, "description": "Note name or path"},
                    {"name": "fuzzy", "type": "boolean", "required": false, "default": false, "description": "Enable fuzzy matching"}
                ]
            },
            "output": {
                "type": "array",
                "fields": ["target", "alias", "is_embed", "line"]
            },
            "examples": [
                {"description": "Get outgoing links", "json": "{\"note\":\"Redis\"}"}
            ]
        }),
        "backlinks" => serde_json::json!({
            "name": "backlinks",
            "description": "Show incoming backlinks pointing to a note",
            "has_side_effects": false,
            "input": {
                "fields": [
                    {"name": "note", "type": "string", "required": true, "description": "Note name or path"},
                    {"name": "context", "type": "boolean", "required": false, "default": false, "description": "Show surrounding text context"},
                    {"name": "fuzzy", "type": "boolean", "required": false, "default": false, "description": "Enable fuzzy matching"}
                ]
            },
            "output": {
                "type": "array",
                "fields": ["source", "source_path", "context", "line"]
            },
            "examples": [
                {"description": "Find backlinks with context", "json": "{\"note\":\"Redis\",\"context\":true}"}
            ]
        }),
        "graph" => serde_json::json!({
            "name": "graph",
            "description": "Explore the link graph",
            "has_side_effects": false,
            "input": {
                "fields": [
                    {"name": "center", "type": "string", "required": false, "description": "Center note for subgraph (omit for full graph)"},
                    {"name": "depth", "type": "integer", "required": false, "default": 2, "description": "BFS traversal depth"},
                    {"name": "graph_format", "type": "string", "required": false, "default": "json", "enum": ["json", "dot", "mermaid"], "description": "Output format"},
                    {"name": "fuzzy", "type": "boolean", "required": false, "default": false, "description": "Enable fuzzy matching for center note"}
                ]
            },
            "output": {
                "type": "object",
                "fields": ["nodes", "edges", "orphans"]
            },
            "examples": [
                {"description": "Subgraph around Redis", "json": "{\"center\":\"Redis\",\"depth\":2}"},
                {"description": "Full graph as DOT", "json": "{\"graph_format\":\"dot\"}"}
            ]
        }),
        "daily" => serde_json::json!({
            "name": "daily",
            "description": "Open or create today's daily note",
            "has_side_effects": true,
            "supports_dry_run": true,
            "input": {
                "fields": [
                    {"name": "date", "type": "string", "required": false, "description": "YYYY-MM-DD (defaults to today)"},
                    {"name": "dry_run", "type": "boolean", "required": false, "default": false, "description": "Preview without creating"}
                ]
            },
            "output": {
                "type": "object",
                "fields": ["action", "path", "date"]
            },
            "examples": [
                {"description": "Open today's note", "json": "{}"},
                {"description": "Preview creation", "json": "{\"dry_run\":true}"}
            ]
        }),
        "create" => serde_json::json!({
            "name": "create",
            "description": "Create a new note",
            "has_side_effects": true,
            "supports_dry_run": true,
            "input": {
                "fields": [
                    {"name": "title", "type": "string", "required": true, "description": "Note title (becomes filename)"},
                    {"name": "dir", "type": "string", "required": false, "description": "Target directory"},
                    {"name": "tags", "type": "string", "required": false, "description": "Comma-separated tags"},
                    {"name": "template", "type": "string", "required": false, "description": "Template note name"},
                    {"name": "frontmatter", "type": "string", "required": false, "description": "YAML frontmatter as JSON string"},
                    {"name": "sections", "type": "string", "required": false, "description": "Comma-separated section headings"},
                    {"name": "content", "type": "string", "required": false, "description": "Initial body text"},
                    {"name": "vars", "type": "string", "required": false, "description": "Template variables (key=val,key=val)"},
                    {"name": "stdin", "type": "boolean", "required": false, "default": false, "description": "Read body from stdin"},
                    {"name": "dry_run", "type": "boolean", "required": false, "default": false, "description": "Preview without creating"},
                    {"name": "if_not_exists", "type": "boolean", "required": false, "default": false, "description": "Skip silently if note exists (idempotent)"}
                ],
                "constraints": ["template and frontmatter are mutually exclusive", "vars requires template"]
            },
            "output": {
                "type": "object",
                "fields": ["action", "path", "title"]
            },
            "examples": [
                {"description": "Simple note", "json": "{\"title\":\"My Note\",\"tags\":\"idea,k8s\"}"},
                {"description": "With frontmatter", "json": "{\"title\":\"Redis 장애\",\"frontmatter\":\"{\\\"type\\\":\\\"troubleshooting\\\"}\",\"tags\":\"troubleshooting\",\"sections\":\"원인,해결\"}"},
                {"description": "Idempotent create", "json": "{\"title\":\"My Note\",\"if_not_exists\":true}"},
                {"description": "Dry run", "json": "{\"title\":\"Test\",\"dry_run\":true}"}
            ]
        }),
        "append" => serde_json::json!({
            "name": "append",
            "description": "Append content to an existing note (section-aware)",
            "has_side_effects": true,
            "supports_dry_run": true,
            "input": {
                "fields": [
                    {"name": "note", "type": "string", "required": true, "description": "Note name (exact match default)"},
                    {"name": "content", "type": "string", "required": true, "description": "Content to append (or use stdin)"},
                    {"name": "section", "type": "string", "required": false, "description": "Insert under this ## section heading"},
                    {"name": "date", "type": "boolean", "required": false, "default": false, "description": "Prepend ### YYYY-MM-DD heading"},
                    {"name": "fuzzy", "type": "boolean", "required": false, "default": false, "description": "Enable fuzzy matching"},
                    {"name": "stdin", "type": "boolean", "required": false, "default": false, "description": "Read content from stdin"},
                    {"name": "dry_run", "type": "boolean", "required": false, "default": false, "description": "Preview without writing"}
                ]
            },
            "output": {
                "type": "object",
                "fields": ["action", "path", "section"]
            },
            "examples": [
                {"description": "Append to end", "json": "{\"note\":\"Meeting\",\"content\":\"New item\"}"},
                {"description": "Insert under section", "json": "{\"note\":\"Meeting\",\"section\":\"Timeline\",\"content\":\"14:30 event\",\"date\":true}"},
                {"description": "Dry run", "json": "{\"note\":\"Meeting\",\"content\":\"test\",\"dry_run\":true}"}
            ]
        }),
        "config" => serde_json::json!({
            "name": "config",
            "description": "Get or set configuration values",
            "has_side_effects": true,
            "input": {
                "fields": [
                    {"name": "key", "type": "string", "required": false, "enum": ["vault_path", "vaults"], "description": "Config key. 'vaults' lists auto-detected vaults"},
                    {"name": "value", "type": "string", "required": false, "description": "Value to set"}
                ]
            },
            "output": {
                "type": "object",
                "fields": ["key", "value", "action"]
            },
            "examples": [
                {"description": "Show all config", "json": "{}"},
                {"description": "Get vault path", "json": "{\"key\":\"vault_path\"}"},
                {"description": "Set vault path", "json": "{\"key\":\"vault_path\",\"value\":\"/path/to/vault\"}"}
            ]
        }),
        "index" => serde_json::json!({
            "name": "index",
            "description": "Manage Tantivy search index",
            "has_side_effects": true,
            "input": {
                "subcommands": ["build", "status", "clear"]
            },
            "output": {
                "type": "object",
                "fields": ["action", "indexed", "skipped", "total", "elapsed_ms"]
            },
            "examples": [
                {"description": "Build index", "cli": "ov index build"},
                {"description": "Check status", "cli": "ov index status"},
                {"description": "Clear index", "cli": "ov index clear"}
            ]
        }),
        _ => {
            return Err(OvError::InvalidInput(format!(
                "Unknown command: {cmd_name}. Use `ov schema commands` to list available commands"
            )));
        }
    };

    Ok(desc)
}

fn schema_skill() -> String {
    r#"# ov — Obsidian Vault CLI (Agent Skill File)

## Overview
`ov` is an agent-first CLI for Obsidian vaults. All output is JSON. All input supports `--json` payloads.

## Invariants
- All responses are JSON: `{"ok": true, "count": N, "data": ..., "meta": {...}}`
- Errors are JSON: `{"ok": false, "error": {"code": "...", "message": "...", "hint": "..."}}`
- Note matching is EXACT by default. Use `--fuzzy` flag to enable fuzzy matching.
- Write commands (create, append, daily) support `--dry-run` for preview.
- `create` supports `--if-not-exists` for idempotent operations.
- Exit codes: 0=success, 1=general, 2=vault_not_found, 3=index_not_built, 4=query_parse, 5=already_exists, 6=invalid_input

## Input Modes
1. Named flags: `ov create --title "Note" --tags "a,b"`
2. JSON payload: `ov create --json '{"title":"Note","tags":"a,b"}'`

## Context Window Management
- Use `--fields title,path,tags` to select only needed fields
- Use `--jsonl` for NDJSON streaming (one object per line, no wrapper)
- Use `--limit` and `--offset` for pagination. Check `meta.has_more` to continue.

## Safety
- Always use `--dry-run` before write operations to preview
- Always use `--if-not-exists` with `create` for retry safety
- Note resolution is exact match only (no fuzzy) unless `--fuzzy` is set
- Path traversal attacks are blocked (vault escape detection)
- Control characters in input are stripped automatically

## Prerequisites
- Set OV_VAULT env var or use --vault flag
- Run `ov index build` before using `search` command
- Index is incremental — safe to rebuild frequently

## Discovery
- `ov schema commands` — list all commands with side-effect flags
- `ov schema describe --command <name>` — input/output schema + examples
- `ov schema skill` — this document
"#
    .to_string()
}
