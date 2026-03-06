# ov — Obsidian Vault CLI (Agent-First)

Rust CLI for Obsidian vaults. Binary: `ov`, crate: `obsidian-vault`.
All output is JSON. Designed for AI agent consumption, not human use.

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
├── main.rs           # Entry point, Ctx struct, cmd_* handlers, schema definitions
├── cli/              # clap derive args (one file per subcommand), all derive Deserialize
│   └── schema.rs     # Schema introspection command (commands, describe, skill)
├── vault/            # Vault::open(), scanner, exact/fuzzy matching, ObsidianConfig
├── extract/          # extract_note(), frontmatter parsing, link/tag regex
├── model/            # Note, NoteSummary, WikiLink, Backlink, Graph*, Frontmatter
├── index/            # Tantivy schema, writer (incremental), reader, tokenizer
├── search/           # parse_query() — tag:/in:/title:/date:/type: prefix parsing
├── service/          # Shared logic: list, tags, stats, backlinks
├── config/           # AppConfig (TOML), paths::resolve_vault_path()
└── output/           # json.rs (ApiResponse, ErrorResponse), fields.rs (field selection)
```

## Agent-First Design

### All Output is JSON
- Success: `{"ok":true, "count":N, "data":..., "meta":{...}}`
- Error: `{"ok":false, "error":{"code":"...", "message":"...", "hint":"..."}}`
- No human/colored output. `colored` dependency removed.

### Input Modes
1. **Named flags**: `ov create --title "Note" --tags "a,b"`
2. **JSON payload**: `ov create --json '{"title":"Note","tags":"a,b"}'`
   - All Args structs derive both `clap::Args` and `serde::Deserialize`
   - JSON input overrides clap-parsed args when `--json` is present

### Deterministic Matching
- Note resolution is **exact match** by default
- `--fuzzy` flag opt-in for fuzzy matching
- `vault.resolve_note_with_mode(query, fuzzy)` — single method, explicit mode

### Safety & Idempotency
- `--dry-run` on all write commands (create, append, daily)
- `--if-not-exists` on create for idempotent retry
- Structured error codes: `NOTE_NOT_FOUND`, `ALREADY_EXISTS`, `INVALID_INPUT`, etc.
- Control characters stripped from all input automatically

### Schema Introspection
- `ov schema commands` — list all commands with side-effect flags
- `ov schema describe --command <name>` — input/output schema + examples
- `ov schema skill` — markdown skill file for agent context injection

### Context Window Management
- `--fields title,path,tags` — select only needed fields (works on ALL commands)
- `--jsonl` — NDJSON streaming (no wrapper, one object per line)
- `meta.has_more` + `meta.next_offset` for pagination
- All commands use unified `output::print_output()` — `--fields` is never bypassed

### Input Hardening
- Path traversal blocked before `--dry-run` and `--if-not-exists` early returns
- Control characters (U+0000..U+001F except \n, \r, \t) stripped from all input
- Title validation: rejects `/`, `\`, `\0`, `..`, empty, >255 bytes
- `config --value` without `--key` returns `INVALID_INPUT` error
- `tags` and `sections` accept both string (`"a,b"`) and array (`["a","b"]`) via `serde_helpers::string_or_array`

## Architecture Patterns

### Index-First Reads
`cmd_list`, `cmd_tags`, `cmd_stats` try `index::reader::read_all_from_index()` first (zero file I/O). Falls back to `Vault::notes()` if index missing.

### Service Layer
`src/service/mod.rs` contains reusable business logic. Two variants per function:
- `list_notes(notes)` — from `&[Note]` (full scan path)
- `list_summaries(summaries)` — from `&[NoteSummary]` (index path)

### Vault Caching
`Vault.notes_cache: OnceLock<Vec<Note>>` — parsed once via `rayon::par_iter()`.

### Incremental Indexing
`index/writer.rs` tracks `file_hashes.json`. Only re-indexes changed files.

## Error Codes

| Code | Exit | Meaning |
|------|------|---------|
| `GENERAL_ERROR` | 1 | Unclassified error |
| `VAULT_NOT_FOUND` | 2 | Vault path invalid |
| `INDEX_NOT_BUILT` | 3 | Search index missing |
| `QUERY_PARSE_ERROR` | 4 | Invalid search query |
| `ALREADY_EXISTS` | 5 | Note already exists |
| `INVALID_INPUT` | 6 | Bad input (path traversal, control chars, etc.) |
| `MISSING_FIELD` | 6 | Required field not provided |

## Key Types

| Type | Location | Purpose |
|------|----------|---------|
| `Note` | `model/note.rs` | Full parsed note (body, links, headings) |
| `NoteSummary` | `model/note.rs` | Lightweight (from index, no body) |
| `Vault` | `vault/mod.rs` | Scanner + resolver + notes cache |
| `ApiResponse<T>` | `output/json.rs` | Standard JSON success wrapper |
| `ErrorResponse` | `output/json.rs` | Structured JSON error wrapper |
| `VaultStats` | `service/mod.rs` | Aggregated vault statistics |

## Adding a New Command

1. Add args struct in `src/cli/<name>.rs` — derive `Args, Deserialize, Default`
2. Add variant to `Command` enum in `src/cli/mod.rs`
3. Add `cmd_<name>()` handler in `src/main.rs` — JSON output only
4. Add `merge_or_use(json_input, args)?` call in `run()` for `--json` support
5. Add command metadata to `schema_commands()` and `schema_describe()` in `main.rs`
6. Add integration tests in `tests/cli_tests.rs`
