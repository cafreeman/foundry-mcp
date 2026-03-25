# Foundry

**Project context and specs in `~/.foundry/`, with JSON-first workflow commands for agents, a `.foundry` symlink into your repo, and optional git backup of the whole store.**

Foundry is a Rust CLI: it scaffolds markdown under `~/.foundry/`, derives spec workflow state from `task-list.md` checkboxes, exposes **`--json`** for skills and automation, and can **collapse** finished specs into `completed/<spec-id>/summary.md` plus an `archive/` of the working files. Agents author all prose; the binary does not generate spec content.

## Install

```bash
cargo install foundry-mcp
```

Rust **1.85+** (see `rust-version` in `Cargo.toml`).

## Quick start

```bash
foundry project init my-app
foundry link my-app                    # in your repo; optional name if auto-detect works
foundry spec init my-app auth-flow     # then edit .foundry/specs/... or paths under ~/.foundry/

foundry git init                       # optional backup of ~/.foundry/
foundry git remote https://github.com/you/foundry-backup.git
foundry git sync "checkpoint message"
```

## Agent-oriented commands (`--json`)

```bash
foundry --json status                  # store path, git, skills, cwd .foundry -> project
foundry --json list projects
foundry --json list specs my-app
foundry --json list completed my-app

foundry --json spec status my-app <spec-id-or-partial>
foundry --json spec instructions apply my-app <spec-id-or-partial>
foundry --json spec instructions collapse my-app <spec-id-or-partial>   # JSON only
```

## Collapse a finished spec (completed-work record)

When every task checkbox is checked:

```bash
foundry spec collapse prepare my-app <spec-id-or-partial>
# Edit ~/.foundry/my-app/completed/<id>/summary.md — you write the shipped-work summary
foundry spec collapse finalize my-app <spec-id-or-partial> --confirm
```

Active files move to `completed/<id>/archive/`; the spec disappears from `specs/`.

## Skills (Claude Code / Cursor)

```bash
foundry install claude-code
foundry install cursor
foundry update
foundry uninstall claude-code
foundry status
```

Bundled skills: **foundry:load**, **foundry:new**, **foundry:work**, **foundry:done** (`assets/skills/`).

## Commands (summary)

| Area | Commands |
|------|-----------|
| Inventory | `foundry list projects`, `list specs <p>`, `list completed <p>` (+ `--json`) |
| Projects | `foundry project init \| load \| list \| delete <name> --confirm` |
| Specs | `foundry spec init \| load \| list \| delete …`, `spec status`, `spec instructions …`, `spec collapse …` |
| Link | `foundry link [project]` |
| Git | `foundry git init`, `remote <url>`, `sync <message...>` |
| Skills | `install`, `update`, `uninstall`, `status` |

## Layout

```
~/.foundry/<project>/
  vision.md
  tech-stack.md
  summary.md
  specs/<YYYYMMDD_HHMMSS>_<feature>/
    meta.json
    spec.md
    task-list.md
    notes.md
  completed/<same-spec-id>/
    summary.md              # agent-written record of shipped work
    archive/                # former spec.md, task-list.md, notes.md, meta.json
      ...
```

## Migrating from 0.8.x

- Run `foundry install …` again to pick up the fourth skill (`foundry_work.md`).
- Prefer `--json` for agent flows; human text defaults are unchanged for most commands.
- New specs get `meta.json`; old spec dirs still work (defaults apply).

## Migrating from 0.7.x (MCP)

- Use the CLI and file tools only; there is no MCP server.
- Use `foundry link` and the skills above.

## License

MIT
