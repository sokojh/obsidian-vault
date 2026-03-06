# ov — Obsidian Vault CLI

Agent-first CLI for Obsidian vaults. All output is JSON. Designed for AI agent consumption.

```bash
ov list --json '{"tag":"#devops","limit":5}' --fields title,path,tags
ov read --json '{"note":"docker","no_body":true}'
ov create --json '{"title":"K8s Guide","tags":["k8s","devops"],"sections":["Overview","Setup"],"dry_run":true}'
ov schema commands --fields name,has_side_effects
```

## Install

### Pre-built binaries (recommended)

Download from [GitHub Releases](https://github.com/sokojh/obsidian-vault/releases/latest):

```bash
# macOS (Apple Silicon)
curl -LO https://github.com/sokojh/obsidian-vault/releases/latest/download/ov-aarch64-apple-darwin.tar.gz
tar xzf ov-aarch64-apple-darwin.tar.gz
sudo mv ov /usr/local/bin/

# macOS (Intel)
curl -LO https://github.com/sokojh/obsidian-vault/releases/latest/download/ov-x86_64-apple-darwin.tar.gz
tar xzf ov-x86_64-apple-darwin.tar.gz
sudo mv ov /usr/local/bin/

# Linux (x86_64)
curl -LO https://github.com/sokojh/obsidian-vault/releases/latest/download/ov-x86_64-unknown-linux-gnu.tar.gz
tar xzf ov-x86_64-unknown-linux-gnu.tar.gz
sudo mv ov /usr/local/bin/

# Linux (ARM64)
curl -LO https://github.com/sokojh/obsidian-vault/releases/latest/download/ov-aarch64-unknown-linux-gnu.tar.gz
tar xzf ov-aarch64-unknown-linux-gnu.tar.gz
sudo mv ov /usr/local/bin/
```

### From crates.io

```bash
cargo install obsidian-vault
```

### From source

```bash
git clone https://github.com/sokojh/obsidian-vault.git
cd obsidian-vault
cargo install --path .
```

### Setup

Set your vault path once:

```bash
# ~/.zshrc or ~/.bashrc
export OV_VAULT="$HOME/Library/Mobile Documents/iCloud~md~obsidian/Documents/MyVault"
```

Or pass `--vault <path>` to any command.

## Agent-First Design

### Why agent-first?

Traditional CLIs optimize for human discoverability and forgiveness. Agent CLIs optimize for **predictability** and **defense-in-depth**. This CLI is built from the ground up for AI agents:

- **JSON-only output** — no colored/human formatting to parse
- **JSON payload input** — `--json` on every command, no flag guessing
- **Schema introspection** — CLI describes itself at runtime
- **Deterministic matching** — exact match by default, no fuzzy surprises
- **Structured errors** — machine-readable codes + actionable hints
- **Safety by default** — `--dry-run`, `--if-not-exists`, path traversal blocking

### Self-Discovery

An agent encountering `ov` for the first time can learn everything it needs:

```bash
# What commands exist? Which have side effects?
ov schema commands --fields name,has_side_effects,supports_dry_run

# How do I use "create"? What fields, what constraints?
ov schema describe --command create

# Give me a skill file to inject into my context
ov schema skill
```

### Input Modes

Every command supports two input styles:

```bash
# Named flags
ov create --title "My Note" --tags "k8s,devops" --dir Zettelkasten

# JSON payload (preferred for agents)
ov create --json '{"title":"My Note","tags":["k8s","devops"],"dir":"Zettelkasten"}'
```

JSON input accepts arrays natively — `"tags":["a","b"]` and `"tags":"a,b"` are equivalent.

### Output Contract

Every successful response:
```json
{"ok":true, "count":1, "data":{...}, "meta":{"total":8, "has_more":false}}
```

Every error response:
```json
{"ok":false, "error":{"code":"NOTE_NOT_FOUND", "message":"...", "hint":"Use --fuzzy flag"}}
```

### Context Window Management

```bash
# Select only needed fields (saves ~84% tokens)
ov read --note "docker" --no-body --fields title,tags,links

# NDJSON streaming — one object per line, no wrapper
ov list --jsonl --fields title,tags

# Pagination
ov list --json '{"limit":10,"offset":0}'
# → check meta.has_more, increment offset
```

### Safety Features

```bash
# Preview before writing (no side effects)
ov create --json '{"title":"Test","dry_run":true}'

# Idempotent create (safe to retry)
ov create --json '{"title":"Test","if_not_exists":true}'

# Exact match by default (no fuzzy hallucinations)
ov read --note "docker"        # exact match only
ov read --note "dock" --fuzzy  # opt-in fuzzy
```

## Commands

| Command | Side Effects | Dry Run | Description |
|---------|:---:|:---:|-------------|
| `list` | | | List notes with filtering by dir/tag/date and sorting |
| `read` | | | Read a note by name (exact match default) |
| `search` | | | Full-text search with prefix filters (requires index) |
| `tags` | | | List all tags with occurrence counts |
| `stats` | | | Vault-wide statistics |
| `links` | | | Outgoing `[[wiki-links]]` from a note |
| `backlinks` | | | Incoming links pointing to a note |
| `graph` | | | Link graph (JSON, DOT, or Mermaid format) |
| `daily` | Yes | Yes | Open or create today's daily note |
| `create` | Yes | Yes | Create a new note (plain, frontmatter, or template) |
| `append` | Yes | Yes | Append to an existing note (section-aware) |
| `index` | Yes | | Manage Tantivy search index (build/status/clear) |
| `config` | Yes | | Get or set configuration values |
| `schema` | | | Introspect CLI schema (commands/describe/skill) |

### Global Options

```
--vault <PATH>       Vault path (or OV_VAULT env)
--json <JSON>        JSON payload input (alternative to flags)
--jsonl              NDJSON output (one object per line)
--fields <FIELDS>    Select specific output fields (comma-separated)
```

## Quick Start

```bash
# Build search index (makes search ~25ms)
ov index build

# Browse notes
ov list --fields title,path,tags --limit 10
ov list --json '{"dir":"Zettelkasten","tag":"#devops","sort":"title"}'

# Full-text search
ov search --json '{"query":"docker networking","limit":5}'
ov search --json '{"query":"tag:#aws in:Zettelkasten"}'

# Read a note
ov read --note "docker"
ov read --note "docker" --no-body --fields title,tags,links

# Explore structure
ov tags --json '{"sort":"count","min_count":2}' --fields tag,count
ov stats --fields total_notes,unique_tags
ov links --json '{"note":"docker"}' --fields target
ov backlinks --json '{"note":"docker"}' --fields source,source_path
ov graph --json '{"center":"docker","depth":1,"graph_format":"mermaid"}'

# Create with dry-run first
ov create --json '{"title":"K8s Guide","tags":["k8s"],"sections":["Overview","Setup"],"dry_run":true}'
ov create --json '{"title":"K8s Guide","tags":["k8s"],"sections":["Overview","Setup"],"if_not_exists":true}'

# Append to a section
ov append --json '{"note":"K8s Guide","section":"Setup","content":"Install kubectl first"}'

# Daily note
ov daily --dry-run
ov daily
```

## Search Prefixes

Combine free-text with structured filters:

```bash
ov search --json '{"query":"tag:#devops in:Zettelkasten kubernetes"}'
ov search --json '{"query":"title:아키텍처 date:2024-01"}'
ov search --json '{"query":"type:person"}'
```

| Prefix | Example | Filters by |
|--------|---------|-----------|
| `tag:` | `tag:#aws` | Tag (auto-adds `#`) |
| `in:` | `in:Zettelkasten` | Directory |
| `title:` | `title:kube` | Title substring |
| `date:` | `date:2024-01` | Modified date prefix |
| `type:` | `type:person` | Frontmatter type |

## Error Codes

All errors are JSON with machine-readable codes and actionable hints:

| Code | Exit | Meaning | Hint |
|------|------|---------|------|
| `GENERAL_ERROR` | 1 | Unclassified error | — |
| `VAULT_NOT_FOUND` | 2 | Vault path invalid | Set OV_VAULT env or use --vault |
| `INDEX_NOT_BUILT` | 3 | Search index missing | Run `ov index build` |
| `QUERY_PARSE_ERROR` | 4 | Invalid search query | — |
| `ALREADY_EXISTS` | 5 | Note already exists | Use --if-not-exists |
| `INVALID_INPUT` | 6 | Bad input (path traversal, etc.) | — |
| `MISSING_FIELD` | 6 | Required field not provided | Use `ov schema describe` |

## Note Creation

### Templates

```bash
ov create --json '{"title":"John Doe","template":"person","vars":"role=SRE,team=Infra"}'
```

Templates live in Obsidian's template folder. Variables: `{{title}}`, `{{date:YYYY-MM-DD}}`, `{{time:HH:mm}}`, custom via `vars`.

### Frontmatter

```bash
ov create --json '{"title":"Meeting","frontmatter":"{\"type\":\"meeting\",\"attendees\":[\"alice\",\"bob\"]}","tags":"meeting"}'
```

Note: `template` and `frontmatter` are mutually exclusive. `vars` requires `template`.

### Section-Aware Append

```bash
ov append --json '{"note":"Project Log","section":"Timeline","content":"Deployed v2"}'
```

Inserts before the next same-or-higher level heading.

## Architecture

```
src/
├── main.rs              # Entry point, Ctx struct, cmd_* handlers, schema definitions
├── cli/                 # clap derive args (all derive Deserialize for --json support)
│   ├── schema.rs        # Schema introspection (commands, describe, skill)
│   └── serde_helpers.rs # Custom deserializers (string_or_array)
├── vault/               # Vault scanning, exact/fuzzy matching, ObsidianConfig
├── extract/             # Note parsing, frontmatter, link/tag regex
├── model/               # Note, Link, Tag, Graph structs
├── index/               # Tantivy schema, reader, writer (incremental), tokenizer
├── search/              # Query parsing with prefix support
├── service/             # Shared business logic (list, tags, stats, backlinks)
├── config/              # AppConfig (TOML), XDG paths
└── output/              # JSON output (ApiResponse, ErrorResponse), field filtering
```

Key design decisions:
- **Agent-first**: All output is JSON. No human/colored output. `colored` dependency removed.
- **Unified output**: All commands use `print_output()` — `--fields` works everywhere.
- **Index-first reads**: `list`, `tags`, `stats` read from Tantivy index (zero file I/O), falling back to vault scan.
- **Parallel I/O**: `rayon::par_iter()` for vault scanning when index unavailable.
- **Incremental indexing**: File hash tracking (`file_hashes.json`) for fast rebuilds.
- **Input hardening**: Path traversal blocking, control character stripping, constraint validation.

## Performance

| Scenario | Time |
|----------|------|
| list/tags/stats (with index) | ~25ms |
| search (with index) | ~25ms |
| index rebuild (incremental, no changes) | ~19ms |

The index-first architecture reads directly from Tantivy — zero file I/O for read-heavy commands. Always run `ov index build` first.

## Configuration

Stored at `~/.local/share/ov/config.toml`:

```bash
ov config --key vault_path
ov config --key vault_path --value "/path/to/vault"
```

Vault resolution priority: `--vault` flag > `OV_VAULT` env > config file > auto-detect (`.obsidian/` walk-up or iCloud).

## License

MIT
