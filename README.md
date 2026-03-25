# Foundry

**Project context and specs in `~/.foundry/`, wired into your repo with a CLI and optional skills.**

Foundry is a small Rust CLI: it scaffolds empty markdown files for projects and timestamped specs, optionally backs up `~/.foundry/` with git, and installs lifecycle skills for Claude Code and Cursor. Agents edit content with their normal file tools—Foundry does not run an MCP server and does not take spec or vision text on the command line.

## Install

```bash
cargo install foundry-mcp
```

## Quick start

```bash
# Create a project scaffold (kebab-case name)
foundry project init my-app

# In a codebase directory, link ~/.foundry/my-app as ./.foundry
foundry link my-app

# New spec (kebab-case feature); then edit files under .foundry/specs/...
foundry spec init my-app auth-flow

# Optional: turn ~/.foundry into a git repo and sync
foundry git init
foundry git remote https://github.com/you/foundry-backup.git
foundry git sync "checkpoint message"
```

## Skills (Claude Code / Cursor)

```bash
foundry install claude-code   # ~/.claude/skills/foundry_*.md
foundry install cursor      # ./.cursor/rules/foundry_*.md (current directory)

foundry update              # refresh previously installed copies
foundry uninstall claude-code
foundry uninstall cursor
foundry status
```

Bundled skills: **foundry:load**, **foundry:new**, **foundry:done** (see `assets/skills/`).

## Commands (summary)

| Area | Commands |
|------|-----------|
| Projects | `foundry project init \| load \| list \| delete <name> --confirm` |
| Specs | `foundry spec init \| load \| list \| delete <project> <id> --confirm` |
| Link | `foundry link [project]` (auto-detect name from Cargo.toml, package.json, git remote, or directory) |
| Git | `foundry git init`, `foundry git remote <url>`, `foundry git sync <message...>` |
| Skills | `foundry install`, `foundry update`, `foundry uninstall`, `foundry status` |

## Layout

```
~/.foundry/<project>/
  vision.md
  tech-stack.md
  summary.md
  specs/<YYYYMMDD_HHMMSS>_<feature>/
    spec.md
    task-list.md
    notes.md
```

## Migrating from 0.7.x (MCP)

- The MCP server and JSON tool layer are removed; use the CLI and direct file edits.
- Uninstall old MCP-related config from your environment, then `foundry install claude-code` / `foundry install cursor` for the new skill files.
- Use `foundry link` so `.foundry/` points at the right project in each repo.

## License

MIT
