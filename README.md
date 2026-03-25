# Foundry

**Project context and specs in `~/.foundry/`, with JSON workflow output for inventory and spec state, a `.foundry` symlink into your repo, and optional git backup of the whole store.**

Foundry is a Rust CLI: it scaffolds markdown under `~/.foundry/`, derives spec workflow state from `task-list.md` checkboxes, prints **JSON** for inventory/status/spec workflow commands (no flag), and can **collapse** finished specs into `completed/<spec-id>/summary.md` plus an `archive/` of the working files. Agents author all prose; the binary does not generate spec content. **Plain text** is reserved for `project load`, `spec load`, `link`, `git`, and skill install commands.

## Install

```bash
cargo install foundry-mcp
```

Rust **1.85+** (see `rust-version` in `Cargo.toml`).

## Quick start

```bash
foundry project init my-app            # stdout: JSON with path (use `path` to locate the dir)
foundry link my-app                    # in your repo; optional name if auto-detect works
foundry spec init my-app auth-flow     # stdout: JSON with id + path; then edit files under that path or `.foundry/`

foundry git init                       # optional backup of ~/.foundry/
foundry git remote https://github.com/you/foundry-backup.git
foundry git sync "checkpoint message"
```

## JSON workflow commands (stdout is always JSON)

```bash
foundry status
foundry list projects
foundry list specs my-app
foundry list completed my-app

foundry project init <name>            # { name, path }
foundry spec init <project> <feature>  # { id, project, feature, path }

foundry spec status my-app <spec-id-or-partial>
foundry spec instructions apply my-app <spec-id-or-partial>
foundry spec instructions collapse my-app <spec-id-or-partial>

foundry spec collapse prepare my-app <spec-id-or-partial>    # { summary_path, … }
foundry spec collapse finalize my-app <spec-id-or-partial> --confirm
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
| Inventory (JSON) | `list projects`, `list specs <p>`, `list completed <p>` |
| Projects | `project init` (JSON), `project load` (text), `project delete … --confirm` (text) |
| Specs | `spec init` (JSON), `spec load` (text), `spec delete …` (text), `spec status` / `instructions` / `collapse` (JSON) |
| Link / Git / Skills | plain-text CLI output; `status` is JSON |

## Layout

```
~/.foundry/<project>/
  vision.md
  tech-stack.md
  summary.md
  specs/<YYYYMMDD_HHMMSS>_<feature>/
    meta.json               # optional; phase_hint must match spec.md + task-list.md if set
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
- **No `--json` flag:** `list`, `status`, `project init`, `spec init`, `spec status`, `spec instructions …`, and `spec collapse …` **always** print JSON to stdout. Drop `--json` from scripts and skills.
- `project load`, `spec load`, `link`, `git`, installs remain plain text.
- New specs get `meta.json`; old spec dirs still work (defaults apply).
- Inventory is only `foundry list …` (not `project list` / `spec list`).

## Migrating from 0.7.x (MCP)

- Use the CLI and file tools only; there is no MCP server.
- Use `foundry link` and the skills above.

## License

MIT
