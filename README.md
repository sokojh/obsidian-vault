# ov — Obsidian Vault CLI

High-performance CLI for Obsidian vaults. Terminal-first.

```
ov search "kubernetes" --snippet
ov list --tag imweb --sort words --limit 10
ov read "아키텍처" --format json
ov stats
```

## Install

```bash
cargo install --path .
```

Set your vault path once:

```bash
# ~/.zshrc
export OV_VAULT="$HOME/Library/Mobile Documents/iCloud~md~obsidian/Documents/MyVault"
```

Or pass `--vault <path>` to any command.

## Quick Start

```bash
# Build search index (recommended, makes everything ~25ms)
ov index build

# Browse notes
ov list --limit 20
ov list --dir Zettelkasten --tag devops --sort title

# Full-text search
ov search "docker networking"
ov search "tag:#aws in:Zettelkasten" --snippet

# Read a note (fuzzy matching)
ov read "kube"        # matches "Kubernetes Basics"
ov read "아키텍처" --raw  # body only, no metadata

# Explore structure
ov tags --sort count --limit 20
ov stats
ov links "My Note"
ov backlinks "My Note" --context
ov graph --format mermaid

# Create & append
ov create "New Idea" --dir Zettelkasten --tags idea,devops
ov append "Meeting Notes" --section "Timeline" --content "Discussed migration plan" --date

# Daily note
ov daily
```

## Commands

| Command | Description |
|---------|-------------|
| `list` | List notes with filtering, sorting, pagination |
| `read` | Read a note by name (fuzzy matching) |
| `search` | Full-text search (Tantivy) with prefix filters |
| `tags` | Aggregate tags with counts |
| `stats` | Vault statistics |
| `links` | Outgoing links from a note |
| `backlinks` | Incoming links to a note |
| `graph` | Link graph (json, dot, mermaid) |
| `daily` | Today's daily note |
| `create` | Create a new note (templates, frontmatter) |
| `append` | Append to an existing note (section-aware) |
| `index` | Manage search index (build/status/clear) |
| `config` | Get/set configuration |

### Global Options

```
--vault <PATH>       Vault path (or OV_VAULT env)
-f, --format <FMT>   human | json | jsonl
--fields <FIELDS>    Field selection (comma-separated)
-q, --quiet          Suppress stderr
```

## Search Prefixes

Combine free-text with structured filters:

```bash
ov search "tag:#devops in:Zettelkasten kubernetes"
ov search "title:아키텍처 date:2024-01"
ov search "type:person"
```

| Prefix | Example | Filters by |
|--------|---------|-----------|
| `tag:` | `tag:#aws` | Tag (auto-adds `#`) |
| `in:` | `in:Zettelkasten` | Directory |
| `title:` | `title:kube` | Title substring |
| `date:` | `date:2024-01` | Modified date prefix |
| `type:` | `type:person` | Frontmatter type |

## Output Formats

**Human** (default) — colored terminal tables:
```
Title                  Directory       Words  Links  Tags
───────────────────────────────────────────────────────────
Kubernetes Basics      Zettelkasten    450    3      #k8s #devops
```

**JSON** — structured API response:
```json
{
  "ok": true,
  "count": 1,
  "data": [{ "title": "Kubernetes Basics", "path": "Zettelkasten/Kubernetes Basics.md", ... }],
  "meta": { "total": 23, "offset": 0, "limit": 50 }
}
```

**JSONL** — one object per line, for streaming/pipelines.

Field selection: `--fields title,path,tags` filters output to specified fields.

## Performance

| Scenario | Time |
|----------|------|
| list/tags/stats (with index) | ~25ms |
| search (with index) | ~25ms |
| index rebuild (incremental, no changes) | ~19ms |
| Full vault scan (330 notes, iCloud cold) | ~169s |

The index-first architecture reads directly from Tantivy — zero file I/O for read-heavy commands. Always run `ov index build` first.

## Architecture

```
src/
├── main.rs              # CLI entry, command dispatch
├── cli/                 # Argument definitions (clap derive)
├── vault/               # Vault scanning, fuzzy matching, config
├── extract/             # Note parsing, frontmatter, patterns
├── model/               # Note, Link, Tag, Graph structs
├── index/               # Tantivy schema, reader, writer, tokenizer
├── search/              # Query parsing with prefix support
├── service/             # Shared business logic (list, tags, stats, backlinks)
├── config/              # App config, XDG paths
└── output/              # Human, JSON, JSONL formatters
```

Key design decisions:
- **Index-first reads**: `list`, `tags`, `stats` read from Tantivy index with no file I/O, falling back to vault scan
- **Parallel I/O**: `rayon::par_iter()` for vault scanning when index unavailable
- **OnceLock caching**: Notes parsed once per session
- **Service layer**: Reusable business logic in `service/mod.rs`
- **Incremental indexing**: File hash tracking for fast rebuilds

## Note Creation

### Templates

```bash
ov create "John Doe" --template person --vars role=SRE,team=Infra
```

Templates live in Obsidian's template folder. Variables: `{{title}}`, `{{date:YYYY-MM-DD}}`, `{{time:HH:mm}}`, custom via `--vars`.

### Frontmatter

```bash
ov create "Meeting" --frontmatter '{"type":"meeting","attendees":["alice","bob"]}' --tags meeting
```

### Section-Aware Append

```bash
ov append "Project Log" --section "Timeline" --content "Deployed v2" --date
```

Inserts before the next same-or-higher level heading.

## Configuration

Stored at `~/.local/share/ov/config.toml`:

```bash
ov config vault_path "/path/to/vault"
ov config default_format json
```

Vault resolution priority: `--vault` flag > `OV_VAULT` env > config file > auto-detect (`.obsidian/` walk-up or iCloud).

## License

MIT
