#![allow(dead_code)]

mod cli;
mod config;
mod error;
mod extract;
mod index;
mod mcp;
mod model;
mod output;
mod search;
mod vault;

use std::collections::HashMap;
use std::process;

use clap::Parser;

use cli::{Cli, Command, OutputFormat};
use config::paths;
use error::OvError;
use model::note::NoteSummary;
use model::tag::TagSummary;
use output::json::ApiResponse;
use vault::Vault;

fn main() {
    let cli = Cli::parse();
    let format = cli.format.clone();

    if let Err(e) = run(cli) {
        match format {
            OutputFormat::Json | OutputFormat::Jsonl => {
                let err_json = serde_json::json!({
                    "ok": false,
                    "error": e.to_string(),
                    "code": e.exit_code(),
                });
                eprintln!("{}", serde_json::to_string(&err_json).unwrap_or_default());
            }
            OutputFormat::Human => {
                eprintln!("error: {e}");
            }
        }
        process::exit(e.exit_code());
    }
}

/// Shared context extracted from Cli for command handlers
struct Ctx {
    vault: Option<String>,
    format: OutputFormat,
    fields: Option<String>,
    quiet: bool,
}

impl From<&Cli> for Ctx {
    fn from(cli: &Cli) -> Self {
        Self {
            vault: cli.vault.clone(),
            format: cli.format.clone(),
            fields: cli.fields.clone(),
            quiet: cli.quiet,
        }
    }
}

fn run(cli: Cli) -> Result<(), OvError> {
    let ctx = Ctx::from(&cli);
    match cli.command {
        Command::List(args) => cmd_list(&ctx, args),
        Command::Read(args) => cmd_read(&ctx, args),
        Command::Tags(args) => cmd_tags(&ctx, args),
        Command::Stats(_) => cmd_stats(&ctx),
        Command::Links(args) => cmd_links(&ctx, args),
        Command::Backlinks(args) => cmd_backlinks(&ctx, args),
        Command::Config(args) => cmd_config(&ctx, args),
        Command::Search(args) => cmd_search(&ctx, args),
        Command::Graph(args) => cmd_graph(&ctx, args),
        Command::Daily(args) => cmd_daily(&ctx, args),
        Command::Create(args) => cmd_create(&ctx, args),
        Command::Append(args) => cmd_append(&ctx, args),
        Command::Index(args) => cmd_index(&ctx, args),
        Command::Mcp(_) => cmd_mcp(&ctx),
    }
}

fn open_vault(ctx: &Ctx) -> Result<Vault, OvError> {
    let vault_path = paths::resolve_vault_path(ctx.vault.as_deref())?;
    Vault::open(vault_path)
}

// ─── list ────────────────────────────────────────────────────────────────

fn cmd_list(ctx: &Ctx, args: cli::list::ListArgs) -> Result<(), OvError> {
    let vault = open_vault(ctx)?;
    let mut notes: Vec<NoteSummary> = vault
        .read_all_notes()
        .iter()
        .map(NoteSummary::from)
        .collect();

    // Filter by directory
    if let Some(ref dir) = args.dir {
        notes.retain(|n| n.dir == *dir || n.dir.starts_with(&format!("{dir}/")));
    }

    // Filter by tag
    if let Some(ref tag) = args.tag {
        let tag_normalized = if tag.starts_with('#') {
            tag.clone()
        } else {
            format!("#{tag}")
        };
        notes.retain(|n| n.tags.iter().any(|t| t == &tag_normalized));
    }

    // Filter by date
    if let Some(ref date) = args.date {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let date_filter = match date.as_str() {
            "today" => today,
            _ => date.clone(),
        };
        notes.retain(|n| n.modified.starts_with(&date_filter));
    }

    // Sort
    match args.sort.as_str() {
        "title" => notes.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase())),
        "size" | "words" => notes.sort_by(|a, b| b.word_count.cmp(&a.word_count)),
        _ => notes.sort_by(|a, b| b.modified.cmp(&a.modified)),
    }

    if args.reverse {
        notes.reverse();
    }

    // Pagination
    let total = notes.len();
    let notes: Vec<NoteSummary> = notes
        .into_iter()
        .skip(args.offset)
        .take(args.limit)
        .collect();

    match ctx.format {
        OutputFormat::Human => {
            output::human::print_note_list(&notes);
        }
        OutputFormat::Jsonl => {
            for note in &notes {
                let json_val = serde_json::to_value(note).unwrap_or_default();
                let line = if let Some(ref fields_str) = ctx.fields {
                    let field_names = output::fields::parse_fields(fields_str);
                    let filtered = output::fields::filter_fields(&json_val, &field_names);
                    serde_json::to_string(&filtered).unwrap_or_default()
                } else {
                    serde_json::to_string(&json_val).unwrap_or_default()
                };
                println!("{line}");
            }
        }
        OutputFormat::Json => {
            let response = ApiResponse::success(&notes, notes.len())
                .with_meta("total", serde_json::json!(total))
                .with_meta("offset", serde_json::json!(args.offset))
                .with_meta("limit", serde_json::json!(args.limit));

            let json_val = serde_json::to_value(&response).unwrap_or_default();
            let output_str = if let Some(ref fields_str) = ctx.fields {
                let field_names = output::fields::parse_fields(fields_str);
                let mut filtered = json_val;
                if let Some(data) = filtered.get_mut("data") {
                    *data = output::fields::filter_fields(data, &field_names);
                }
                serde_json::to_string_pretty(&filtered).unwrap_or_default()
            } else {
                serde_json::to_string_pretty(&json_val).unwrap_or_default()
            };
            println!("{output_str}");
        }
    }

    Ok(())
}

// ─── read ────────────────────────────────────────────────────────────────

fn cmd_read(ctx: &Ctx, args: cli::read::ReadArgs) -> Result<(), OvError> {
    let vault = open_vault(ctx)?;
    let file_path = vault.resolve_note(&args.note)?;
    let relative = vault.relative_path(&file_path);
    let note = vault.read_note(&relative)?;

    if args.raw {
        if let Some(ref body) = note.body {
            print!("{body}");
        }
        return Ok(());
    }

    match ctx.format {
        OutputFormat::Human => {
            output::human::print_note_detail(
                &note.title,
                &note.path,
                &note.tags,
                note.body.as_deref().unwrap_or(""),
            );
        }
        _ => {
            let mut note_output = note;
            if !args.body {
                note_output.body = None;
            }
            output::print_output(note_output, 1, &ctx.format, &ctx.fields);
        }
    }

    Ok(())
}

// ─── tags ────────────────────────────────────────────────────────────────

fn cmd_tags(ctx: &Ctx, args: cli::tags::TagsArgs) -> Result<(), OvError> {
    let vault = open_vault(ctx)?;
    let notes = vault.read_all_notes();

    let mut tag_map: HashMap<String, Vec<String>> = HashMap::new();
    for note in &notes {
        for tag in &note.tags {
            tag_map
                .entry(tag.clone())
                .or_default()
                .push(note.title.clone());
        }
    }

    let mut summaries: Vec<TagSummary> = tag_map
        .into_iter()
        .map(|(tag, notes)| TagSummary {
            count: notes.len(),
            tag,
            notes,
        })
        .collect();

    if let Some(min) = args.min_count {
        summaries.retain(|s| s.count >= min);
    }

    match args.sort.as_str() {
        "name" => summaries.sort_by(|a, b| a.tag.cmp(&b.tag)),
        _ => summaries.sort_by(|a, b| b.count.cmp(&a.count)),
    }

    if let Some(limit) = args.limit {
        summaries.truncate(limit);
    }

    let count = summaries.len();
    match ctx.format {
        OutputFormat::Human => {
            output::human::print_tag_list(&summaries);
        }
        _ => {
            output::print_output(summaries, count, &ctx.format, &ctx.fields);
        }
    }

    Ok(())
}

// ─── stats ───────────────────────────────────────────────────────────────

fn cmd_stats(ctx: &Ctx) -> Result<(), OvError> {
    let vault = open_vault(ctx)?;
    let notes = vault.read_all_notes();

    let total_notes = notes.len();
    let total_words: usize = notes.iter().map(|n| n.word_count).sum();
    let total_links: usize = notes.iter().map(|n| n.links.len()).sum();

    let mut all_tags: HashMap<String, usize> = HashMap::new();
    for note in &notes {
        for tag in &note.tags {
            *all_tags.entry(tag.clone()).or_default() += 1;
        }
    }

    let dirs = vault.directories();
    let total_size: u64 = notes.iter().map(|n| n.file_meta.size).sum();
    let evicted: usize = notes.iter().filter(|n| n.file_meta.evicted).count();

    // Sort tags by count for top_tags
    let mut sorted_tags: Vec<_> = all_tags.iter().collect();
    sorted_tags.sort_by(|a, b| b.1.cmp(a.1));
    let top_tags: Vec<_> = sorted_tags
        .iter()
        .take(10)
        .map(|(tag, count)| serde_json::json!({"tag": tag, "count": count}))
        .collect();

    let stats = serde_json::json!({
        "total_notes": total_notes,
        "total_words": total_words,
        "total_links": total_links,
        "unique_tags": all_tags.len(),
        "directories": dirs.len(),
        "total_size_bytes": total_size,
        "total_size_mb": format!("{:.1}", total_size as f64 / 1_048_576.0),
        "evicted_files": evicted,
        "avg_words_per_note": if total_notes > 0 { total_words / total_notes } else { 0 },
        "avg_links_per_note": if total_notes > 0 { total_links / total_notes } else { 0 },
        "top_tags": top_tags,
        "directory_list": dirs,
    });

    match ctx.format {
        OutputFormat::Human => {
            output::human::print_stats(&stats);
        }
        _ => {
            let response = ApiResponse::success(&stats, 1);
            println!("{}", response.to_json_string());
        }
    }

    Ok(())
}

// ─── links ───────────────────────────────────────────────────────────────

fn cmd_links(ctx: &Ctx, args: cli::links::LinksArgs) -> Result<(), OvError> {
    let vault = open_vault(ctx)?;
    let file_path = vault.resolve_note(&args.note)?;
    let relative = vault.relative_path(&file_path);
    let note = vault.read_note(&relative)?;

    let count = note.links.len();
    match ctx.format {
        OutputFormat::Human => {
            if note.links.is_empty() {
                println!("No outgoing links.");
            } else {
                println!("Outgoing links from \"{}\":", note.title);
                for link in &note.links {
                    let kind = if link.is_embed { "embed" } else { "link" };
                    let alias = link
                        .alias
                        .as_ref()
                        .map(|a| format!(" ({a})"))
                        .unwrap_or_default();
                    println!("  [{kind}] [[{}]]{alias} (line {})", link.target, link.line);
                }
            }
        }
        _ => {
            output::print_output(&note.links, count, &ctx.format, &ctx.fields);
        }
    }

    Ok(())
}

// ─── backlinks ───────────────────────────────────────────────────────────

fn cmd_backlinks(ctx: &Ctx, args: cli::links::BacklinksArgs) -> Result<(), OvError> {
    let vault = open_vault(ctx)?;
    let file_path = vault.resolve_note(&args.note)?;
    let target_stem = file_path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let notes = vault.read_all_notes();
    let mut backlinks: Vec<model::link::Backlink> = Vec::new();

    for note in &notes {
        for link in &note.links {
            if link.target.eq_ignore_ascii_case(&target_stem) {
                let context = if args.context {
                    // Read full file content for accurate line context
                    let full_path = vault.root.join(&note.path);
                    std::fs::read_to_string(&full_path).ok().and_then(|content| {
                        content
                            .lines()
                            .nth(link.line.saturating_sub(1))
                            .map(|l| l.trim().to_string())
                    })
                } else {
                    None
                };

                backlinks.push(model::link::Backlink {
                    source: note.title.clone(),
                    source_path: note.path.clone(),
                    context,
                    line: link.line,
                });
            }
        }
    }

    let count = backlinks.len();
    match ctx.format {
        OutputFormat::Human => {
            if backlinks.is_empty() {
                println!("No backlinks found for \"{target_stem}\".");
            } else {
                println!("Backlinks to \"{target_stem}\":");
                for bl in &backlinks {
                    print!("  <- {} ({}:{})", bl.source, bl.source_path, bl.line);
                    if let Some(ref ctx) = bl.context {
                        print!("  | {}", ctx.trim());
                    }
                    println!();
                }
            }
        }
        _ => {
            output::print_output(&backlinks, count, &ctx.format, &ctx.fields);
        }
    }

    Ok(())
}

// ─── config ──────────────────────────────────────────────────────────────

fn cmd_config(ctx: &Ctx, args: cli::config::ConfigArgs) -> Result<(), OvError> {
    let mut app_config = config::AppConfig::load()?;

    match (args.key.as_deref(), args.value.as_deref()) {
        (None, None) => {
            match ctx.format {
                OutputFormat::Human => {
                    println!("Config file: {}", paths::config_path().display());
                    println!(
                        "  vault_path: {}",
                        app_config.vault_path.as_deref().unwrap_or("(auto-detect)")
                    );
                    println!(
                        "  default_format: {}",
                        app_config.default_format.as_deref().unwrap_or("human")
                    );
                }
                _ => {
                    let json = serde_json::to_value(&app_config).unwrap_or_default();
                    let response = ApiResponse::success(&json, 1);
                    println!("{}", response.to_json_string());
                }
            }
        }
        (Some(key), None) => match key {
            "vault_path" => println!(
                "{}",
                app_config.vault_path.as_deref().unwrap_or("(not set)")
            ),
            "default_format" => println!(
                "{}",
                app_config.default_format.as_deref().unwrap_or("human")
            ),
            _ => eprintln!("Unknown config key: {key}"),
        },
        (Some(key), Some(value)) => {
            match key {
                "vault_path" => app_config.vault_path = Some(value.to_string()),
                "default_format" => app_config.default_format = Some(value.to_string()),
                _ => {
                    return Err(OvError::General(format!("Unknown config key: {key}")));
                }
            }
            app_config.save()?;
            if !ctx.quiet {
                eprintln!("Config updated: {key} = {value}");
            }
        }
        _ => {}
    }

    Ok(())
}

// ─── search ──────────────────────────────────────────────────────────────

fn cmd_search(ctx: &Ctx, args: cli::search::SearchArgs) -> Result<(), OvError> {
    let vault_path = paths::resolve_vault_path(ctx.vault.as_deref())?;
    let results = search::search(&vault_path, &args.query, args.limit, args.offset, args.snippet)?;

    let count = results.len();
    match ctx.format {
        OutputFormat::Human => {
            use colored::Colorize;
            if results.is_empty() {
                println!("No results found.");
            } else {
                for hit in &results {
                    println!(
                        "{} {} {}",
                        hit.title.cyan().bold(),
                        format!("({:.2})", hit.score).dimmed(),
                        hit.path.dimmed()
                    );
                    if !hit.tags.is_empty() {
                        println!("  Tags: {}", hit.tags.join(" ").yellow());
                    }
                    if let Some(ref snippet) = hit.snippet {
                        // Convert HTML snippet to plain text with markers
                        let plain = snippet
                            .replace("<b>", "\x1b[1;33m")
                            .replace("</b>", "\x1b[0m");
                        println!("  {plain}");
                    }
                    println!();
                }
                println!("{count} results");
            }
        }
        _ => {
            output::print_output(&results, count, &ctx.format, &ctx.fields);
        }
    }

    Ok(())
}

// ─── index ───────────────────────────────────────────────────────────────

fn cmd_index(ctx: &Ctx, args: cli::index::IndexArgs) -> Result<(), OvError> {
    match args.action {
        cli::index::IndexAction::Build => {
            let vault = open_vault(ctx)?;
            if !ctx.quiet {
                eprintln!("Building index for {}...", vault.root.display());
            }
            let result = index::writer::build_index(&vault, false)?;

            // Also build link index
            let link_idx = index::link_index::LinkIndex::build(&vault);
            link_idx.save(&vault.root)?;

            match ctx.format {
                OutputFormat::Human => {
                    eprintln!(
                        "Indexed {} files ({} new, {} unchanged) in {}ms",
                        result.total, result.indexed, result.skipped, result.elapsed_ms
                    );
                }
                _ => {
                    let data = serde_json::json!({
                        "indexed": result.indexed,
                        "skipped": result.skipped,
                        "total": result.total,
                        "elapsed_ms": result.elapsed_ms,
                    });
                    let response = ApiResponse::success(&data, 1);
                    println!("{}", response.to_json_string());
                }
            }
        }
        cli::index::IndexAction::Status => {
            let vault_path = paths::resolve_vault_path(ctx.vault.as_deref())?;
            let status = index::writer::index_status(&vault_path)?;

            match ctx.format {
                OutputFormat::Human => {
                    output::human::print_stats(&status);
                }
                _ => {
                    let response = ApiResponse::success(&status, 1);
                    println!("{}", response.to_json_string());
                }
            }
        }
        cli::index::IndexAction::Clear => {
            let vault_path = paths::resolve_vault_path(ctx.vault.as_deref())?;
            index::writer::clear_index(&vault_path)?;
            if !ctx.quiet {
                eprintln!("Index cleared.");
            }
        }
    }

    Ok(())
}

// ─── graph ───────────────────────────────────────────────────────────────

fn cmd_graph(ctx: &Ctx, args: cli::graph::GraphArgs) -> Result<(), OvError> {
    let vault = open_vault(ctx)?;

    // Build link index (always fresh from vault)
    let link_idx = index::link_index::LinkIndex::build(&vault);

    if let Some(ref center) = args.center {
        // Subgraph from center
        let resolved = vault.resolve_note(center)?;
        let stem = resolved
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let (nodes, edges) = link_idx.subgraph(&stem, args.depth);

        match args.graph_format.as_str() {
            "dot" => {
                println!("{}", index::link_index::to_dot(&nodes, &edges));
            }
            "mermaid" => {
                println!("{}", index::link_index::to_mermaid(&nodes, &edges));
            }
            _ => {
                let data = serde_json::json!({
                    "center": stem,
                    "depth": args.depth,
                    "nodes": nodes,
                    "edges": edges.iter().map(|(s, t)| serde_json::json!({"source": s, "target": t})).collect::<Vec<_>>(),
                });
                let response = ApiResponse::success(&data, nodes.len());
                println!("{}", response.to_json_string());
            }
        }
    } else {
        // Full graph
        let graph = link_idx.to_graph(&vault);

        match args.graph_format.as_str() {
            "dot" => {
                let nodes: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
                let edges: Vec<(String, String)> = graph
                    .edges
                    .iter()
                    .map(|e| (e.source.clone(), e.target.clone()))
                    .collect();
                println!("{}", index::link_index::to_dot(&nodes, &edges));
            }
            "mermaid" => {
                let nodes: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
                let edges: Vec<(String, String)> = graph
                    .edges
                    .iter()
                    .map(|e| (e.source.clone(), e.target.clone()))
                    .collect();
                println!("{}", index::link_index::to_mermaid(&nodes, &edges));
            }
            _ => {
                let response = ApiResponse::success(&graph, graph.nodes.len());
                println!("{}", response.to_json_string());
            }
        }
    }

    Ok(())
}

// ─── mcp ─────────────────────────────────────────────────────────────────

fn cmd_mcp(ctx: &Ctx) -> Result<(), OvError> {
    let vault_path = paths::resolve_vault_path(ctx.vault.as_deref())?;

    let rt = tokio::runtime::Runtime::new().map_err(|e| OvError::General(e.to_string()))?;
    rt.block_on(async {
        mcp::run_mcp_server(vault_path)
            .await
            .map_err(|e| OvError::General(e.to_string()))
    })
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
        // Read existing daily note
        let note = vault.read_note(&relative)?;
        match ctx.format {
            OutputFormat::Human => {
                output::human::print_note_detail(
                    &note.title,
                    &note.path,
                    &note.tags,
                    note.body.as_deref().unwrap_or(""),
                );
            }
            _ => {
                output::print_output(note, 1, &ctx.format, &ctx.fields);
            }
        }
    } else if args.dry_run {
        match ctx.format {
            OutputFormat::Human => {
                println!("Would create: {relative}");
                println!("# {date}\n\n## Notes\n");
            }
            _ => {
                let data = serde_json::json!({
                    "action": "create",
                    "path": relative,
                    "dry_run": true,
                });
                let response = ApiResponse::success(&data, 1);
                println!("{}", response.to_json_string());
            }
        }
    } else {
        // Create daily note
        let dir_path = vault.root.join(daily_dir);
        std::fs::create_dir_all(&dir_path)?;

        let content = format!("# {date}\n\n## Notes\n\n");
        std::fs::write(&full_path, &content)?;

        if !ctx.quiet {
            eprintln!("Created daily note: {relative}");
        }

        match ctx.format {
            OutputFormat::Human => {
                println!("Created: {relative}");
            }
            _ => {
                let data = serde_json::json!({
                    "action": "created",
                    "path": relative,
                });
                let response = ApiResponse::success(&data, 1);
                println!("{}", response.to_json_string());
            }
        }
    }

    Ok(())
}

// ─── create ──────────────────────────────────────────────────────────────

fn cmd_create(ctx: &Ctx, args: cli::create::CreateArgs) -> Result<(), OvError> {
    let vault = open_vault(ctx)?;

    // Determine target directory
    let dir = args.dir.as_deref().unwrap_or_else(|| {
        vault
            .obsidian_config
            .new_file_location
            .as_deref()
            .unwrap_or("")
    });

    let filename = format!("{}.md", args.title);
    let relative = if dir.is_empty() {
        filename.clone()
    } else {
        format!("{dir}/{filename}")
    };
    let full_path = vault.root.join(&relative);

    if full_path.exists() {
        return Err(OvError::General(format!("Note already exists: {relative}")));
    }

    // Build content
    let mut content = String::new();

    // Read from template if specified
    if let Some(ref template_name) = args.template {
        let template_dir = vault
            .obsidian_config
            .template_folder
            .as_deref()
            .unwrap_or("Templates");
        let template_path = vault.root.join(template_dir).join(format!("{template_name}.md"));
        if template_path.exists() {
            content = std::fs::read_to_string(&template_path)?;
            // Replace template variables
            let now = chrono::Local::now();
            content = content.replace("{{date:YYYY-MM-DD}}", &now.format("%Y-%m-%d").to_string());
            content = content.replace("{{time:HH:mm}}", &now.format("%H:%M").to_string());
            content = content.replace("{{title}}", &args.title);

            // Apply --vars substitutions
            if let Some(ref vars_str) = args.vars {
                for pair in vars_str.split(',') {
                    if let Some((k, v)) = pair.split_once('=') {
                        let k = k.trim();
                        let v = v.trim();
                        content = content.replace(&format!("{{{{{k}}}}}"), v);
                    }
                }
            }

            // Clean remaining {{...}} placeholders (replace with empty string)
            let placeholder_re = regex::Regex::new(r"\{\{[^}]+\}\}").unwrap();
            content = placeholder_re.replace_all(&content, "").to_string();
        } else {
            eprintln!("Template not found: {template_name}");
        }
    }

    // Read from stdin if requested
    if args.stdin {
        use std::io::Read;
        let mut stdin_content = String::new();
        std::io::stdin().read_to_string(&mut stdin_content)?;
        content.push_str(&stdin_content);
    }

    // Add tags if specified and content is empty (no template)
    if content.is_empty() {
        content.push_str(&format!("# {}\n\n", args.title));
        if let Some(ref tags_str) = args.tags {
            let tags: Vec<&str> = tags_str.split(',').map(|t| t.trim()).collect();
            let tags_formatted: Vec<String> = tags
                .iter()
                .map(|t| {
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

    // Create parent directory if needed
    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(&full_path, &content)?;

    if !ctx.quiet {
        eprintln!("Created note: {relative}");
    }

    match ctx.format {
        OutputFormat::Human => {
            println!("Created: {relative}");
        }
        _ => {
            let data = serde_json::json!({
                "action": "created",
                "path": relative,
                "title": args.title,
            });
            let response = ApiResponse::success(&data, 1);
            println!("{}", response.to_json_string());
        }
    }

    Ok(())
}

// ─── append ─────────────────────────────────────────────────────────────

fn cmd_append(ctx: &Ctx, args: cli::append::AppendArgs) -> Result<(), OvError> {
    let vault = open_vault(ctx)?;
    let file_path = vault.resolve_note(&args.note)?;
    let relative = vault.relative_path(&file_path);

    // Read content to append
    let mut new_content = String::new();
    if args.stdin {
        use std::io::Read;
        std::io::stdin().read_to_string(&mut new_content)?;
    } else if let Some(ref text) = args.content {
        new_content = text.clone();
    } else {
        return Err(OvError::General(
            "Either --content or --stdin is required".to_string(),
        ));
    }

    // Prepend date subheading if requested
    if args.date {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        new_content = format!("### {today}\n{new_content}");
    }

    // Read existing file
    let mut file_content = std::fs::read_to_string(&file_path)?;

    // Find insert point
    if let Some(ref section) = args.section {
        let insert_pos = find_section_insert_point(&file_content, section);
        // Ensure proper spacing
        let prefix = if insert_pos > 0
            && !file_content[..insert_pos].ends_with("\n\n")
        {
            if file_content[..insert_pos].ends_with('\n') {
                "\n".to_string()
            } else {
                "\n\n".to_string()
            }
        } else {
            String::new()
        };
        let suffix = if insert_pos < file_content.len()
            && !file_content[insert_pos..].starts_with('\n')
        {
            "\n".to_string()
        } else {
            String::new()
        };
        file_content.insert_str(insert_pos, &format!("{prefix}{new_content}\n{suffix}"));
    } else {
        // Append to end
        if !file_content.ends_with('\n') {
            file_content.push('\n');
        }
        file_content.push('\n');
        file_content.push_str(&new_content);
        if !new_content.ends_with('\n') {
            file_content.push('\n');
        }
    }

    std::fs::write(&file_path, &file_content)?;

    if !ctx.quiet {
        eprintln!("Appended to: {relative}");
    }

    match ctx.format {
        OutputFormat::Human => {
            println!("Appended to: {relative}");
        }
        _ => {
            let data = serde_json::json!({
                "action": "appended",
                "path": relative,
                "section": args.section,
            });
            let response = ApiResponse::success(&data, 1);
            println!("{}", response.to_json_string());
        }
    }

    Ok(())
}

/// Find the insert point within a section (delegates to vault module).
fn find_section_insert_point(content: &str, section: &str) -> usize {
    vault::find_section_insert_point(content, section)
}
