# ov — Obsidian Vault CLI

Rust CLI for Obsidian vaults. Binary: `ov`, crate: `obsidian-vault`.

## Build & Test

```bash
cargo build                    # debug build
cargo build --release          # release build
cargo test                     # all tests (unit + integration)
cargo test --test cli_tests    # integration tests only
cargo install --path .         # install to ~/.cargo/bin/ov
```

Test vault: `tests/fixtures/sample_vault/`

## Project Layout

```
src/
├── main.rs           # Entry point, Ctx struct, cmd_* handlers
├── cli/              # clap derive args (one file per subcommand)
├── vault/            # Vault::open(), scanner, fuzzy matching, ObsidianConfig
├── extract/          # extract_note(), frontmatter parsing, link/tag regex
├── model/            # Note, NoteSummary, WikiLink, Backlink, Graph*, Frontmatter
├── index/            # Tantivy schema, writer (incremental), reader, tokenizer
├── search/           # parse_query() — tag:/in:/title:/date:/type: prefix parsing
├── service/          # Shared logic: list, tags, stats, backlinks (CLI + MCP)
├── config/           # AppConfig (TOML), paths::resolve_vault_path()
└── output/           # human.rs, json.rs (ApiResponse), fields.rs (field selection)
```

## Architecture Patterns

### Index-First Reads
`cmd_list`, `cmd_tags`, `cmd_stats` try `index::reader::read_all_from_index()` first (zero file I/O from Tantivy). Falls back to `Vault::notes()` if index missing.

### Service Layer
`src/service/mod.rs` contains reusable business logic. Two variants per function:
- `list_notes(notes)` — from `&[Note]` (full scan path)
- `list_summaries(summaries)` — from `&[NoteSummary]` (index path)

### Vault Caching
`Vault.notes_cache: OnceLock<Vec<Note>>` — parsed once via `rayon::par_iter()`, reused across commands in same session.

### Incremental Indexing
`index/writer.rs` tracks `file_hashes.json` (size + modified timestamp). Only re-indexes changed files. Schema migration clears both tantivy dir AND file_hashes.json.

### Schema (Tantivy)
Fields: `path`, `title` (3x boost), `body`, `tags`, `dir`, `modified`, `hash`, `note_type`, `word_count`, `file_size`, `link_count`. Schema version check: `get_field("word_count")`.

## Conventions

- **Error handling**: `OvError` enum in `error.rs`, exit codes 0-4
- **Output**: All commands support `--format human|json|jsonl`, JSON wraps in `ApiResponse { ok, count, data, meta }`
- **Path safety**: `cmd_create` uses `canonicalize()` + component walk to prevent vault escape
- **File creation**: `OpenOptions::create_new(true)` for atomic create (no TOCTOU)
- **Unicode**: `truncate_str()` in `human.rs` handles CJK/emoji width correctly

## Key Types

| Type | Location | Purpose |
|------|----------|---------|
| `Note` | `model/note.rs` | Full parsed note (body, links, headings) |
| `NoteSummary` | `model/note.rs` | Lightweight (from index, no body) |
| `Vault` | `vault/mod.rs` | Scanner + resolver + notes cache |
| `SearchHit` | `index/reader.rs` | Tantivy search result |
| `VaultStats` | `service/mod.rs` | Aggregated vault statistics |
| `VaultGraph` | `model/graph.rs` | Nodes + edges + orphans + degree map |
| `Frontmatter` | `model/frontmatter.rs` | Parsed YAML/Zettelkasten metadata |

## Adding a New Command

1. Add args struct in `src/cli/<name>.rs`
2. Add variant to `Command` enum in `src/cli/mod.rs`
3. Add `cmd_<name>()` handler in `src/main.rs`
4. If logic is reusable, extract to `src/service/mod.rs`
5. Add integration tests in `tests/cli_tests.rs`
