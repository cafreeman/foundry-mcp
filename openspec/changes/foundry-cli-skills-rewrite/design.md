## Context

Foundry is currently a dual-mode Rust binary: a CLI for human use and an MCP server for AI tool integration. The MCP server model requires agents to call structured JSON-RPC tools to read/write project specs, but agents already have capable file editing tools (Read, Edit, Write) — the MCP layer adds indirection without value.

The rewrite makes foundry a pure CLI: a storage and scaffolding tool that agents invoke via shell, then edit files directly. The central `~/.foundry/` store remains the defining feature — specs live outside codebases, are accessible across projects, and can be backed up to a git remote.

## Goals / Non-Goals

**Goals:**
- Pure CLI binary: all functionality via subcommands, no server mode
- Agents write file content directly; foundry only scaffolds structure and reads
- Symlink bridge: `foundry link` makes `~/.foundry/<project>/` appear as `.foundry/` in the working directory
- Git backup: `~/.foundry/` as an optional git repo with auto-commit on scaffold and `foundry git sync` for agent edits
- Skill installer: deploys bundled `.md` skill files to Claude Code / Cursor
- Lightweight lifecycle skills: `foundry:load`, `foundry:new`, `foundry:done`

**Non-Goals:**
- Content validation or generation (agents own content)
- MCP protocol support (removed entirely)
- Multi-user or shared foundry stores
- Skill execution engine (skills are `.md` files read by the AI tool, not run by foundry)

## Decisions

### 1. Full rewrite rather than incremental simplification

The existing codebase is shaped around MCP: `FoundryResponse<T>` wrappers, `next_steps`/`workflow_hints` fields, backends abstraction (FilesystemBackend/MemoryBackend), edit_engine with `EditCommand` types, MCP macros. Adapting these to the new model produces a codebase with vestigial complexity. A ground-up rewrite targeting the new scope produces ~40% of the current code and clearer module boundaries.

**Alternatives considered:** Strip MCP in-place — rejected because the response types and backends abstraction would need gutting anyway, and the test suite would need rewriting for the new command structure.

### 2. Symlinks over absolute-path output

The bridge between `~/.foundry/` and a project's working directory could be: (a) CLI outputs absolute paths, agent uses them; (b) symlink `.foundry -> ~/.foundry/<project>/`; (c) smart project detection.

Symlinks win: agents work naturally with relative paths (`.foundry/vision.md`), the files look and behave local, and `.foundry/` in `.gitignore` keeps project repos clean. Absolute-path output would require every skill to capture and thread paths.

### 3. Auto-detect project name from codebase, explicit override

`foundry link` with no argument reads project name from (in priority order): `Cargo.toml` `[package].name`, `package.json` `name`, git remote URL (last path segment, strip `.git`), directory name. Explicit `foundry link <name>` always overrides. This covers the common case with zero friction while remaining predictable.

### 4. Git via subprocess, not library

Git integration shells out to system `git` rather than using `libgit2`/`git2` crate. Reasons: no additional compile-time dependency, no OpenSSL linking complexity, system git handles SSH/HTTPS credential management transparently, and the operations needed (init, add, commit, push, remote) are simple and stable across git versions.

### 5. Skills bundled as binary assets with `include_str!`

Skill `.md` files are embedded in the binary at compile time via `include_str!`. `foundry install` writes them to disk; `foundry update` overwrites with the bundled version. This means the skill version is always in sync with the binary version — no separate download or version mismatch.

**Alternatives considered:** Download skills from a remote URL — rejected, adds network dependency and makes offline use unreliable.

### 6. Git auto-commit on scaffold writes, `foundry git sync` for agent edits

Two commit points: (a) foundry-initiated writes (`project init`, `spec init`) auto-commit immediately — these are discrete events with clear commit messages; (b) agent edits via `.foundry/` symlink are batched and committed via `foundry git sync "<message>"` which the skill directs the agent to call at workflow checkpoints. This gives a useful local history without over-committing mid-edit.

## Risks / Trade-offs

- **Symlink gitignore discipline required** — If users don't add `.foundry` to `.gitignore`, the symlink gets committed. The `foundry link` command should print a reminder and optionally append to `.gitignore` automatically.
- **System git dependency** — `foundry git` commands fail if git is not installed. The `foundry git init` command should check for git availability and print a clear error.
- **Skill format lock-in** — Skills are `.md` files with Claude Code / Cursor frontmatter. If those tools change their skill format, foundry needs a binary release to update. Acceptable trade-off for the simplicity of `include_str!`.
- **Project name detection edge cases** — A Cargo workspace with multiple members, or a `package.json` with a scoped name (`@org/pkg`), may produce unexpected project names. The auto-detection should sanitize to kebab-case and warn when the result is ambiguous.
- **Breaking change with no migration path** — Existing MCP-based users lose their integration immediately on upgrade. Mitigation: clear changelog entry, `foundry uninstall` removes old MCP config cleanly before users run `foundry install` for the new skill-based setup.

## Migration Plan

1. Existing users run `foundry uninstall claude-code` / `foundry uninstall cursor` (old binary)
2. Install new binary (e.g. `cargo install foundry-mcp` or download release)
3. Run `foundry install claude-code` / `foundry install cursor` (deploys skill files)
4. In each project: `foundry link` to create the `.foundry` symlink, add `.foundry` to `.gitignore`
5. Optionally: `foundry git init` + `foundry git remote <url>` to enable backup

No data migration needed — `~/.foundry/` file structure is unchanged.

## Open Questions

- Should `foundry link` automatically append `.foundry` to `.gitignore` if a `.gitignore` exists, or just print a reminder? (Leaning toward auto-append with a notice, opt-out via flag)
- What is the exact skill format for Cursor? Cursor uses `.cursor/rules/` for rules files — confirm whether skills go in `.cursor/rules/` or a different location.
- Should `foundry git sync` without a message auto-generate one (e.g. "foundry: sync from <project>"), or require a message argument? Skills can provide the message, but a fallback default would make manual use friendlier.
