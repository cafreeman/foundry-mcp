## 1. Tear Down Old Code

- [x] 1.1 Delete `src/mcp/` module entirely (server, handlers, tools, traits, macros, error)
- [x] 1.2 Delete `src/core/edit_engine.rs` and remove from `src/core/mod.rs`
- [x] 1.3 Delete `src/core/backends/` (filesystem, memory, tests, mod)
- [x] 1.4 Delete `src/core/ops/analyze_project.rs`, `validate_content.rs`, `get_foundry_help.rs`
- [x] 1.5 Delete `src/types/edit_commands.rs` and remove from `src/types/mod.rs`
- [x] 1.6 Remove `FoundryResponse<T>`, `next_steps`, `workflow_hints` from `src/types/responses.rs`
- [x] 1.7 Remove `rust-mcp-sdk`, `tokio` (if only used by MCP), and any other MCP-only deps from `Cargo.toml`
- [x] 1.8 Remove `foundry serve` subcommand and MCP dispatch from `src/main.rs` and `src/cli/`
- [x] 1.9 Remove `src/core/installation/claude_code.rs` MCP registration logic (keep file structure for skill installation reuse)
- [x] 1.10 Ensure `cargo build` compiles after deletions (fix all dangling references)

## 2. Core Types and Path Utilities

- [x] 2.1 Define lean `Project` struct: `name: String`, `path: PathBuf`
- [x] 2.2 Define lean `Spec` struct: `id: String`, `project_name: String`, `feature: String`, `path: PathBuf`
- [x] 2.3 Write `foundry_dir() -> PathBuf` resolving `~/.foundry/` (keep existing, verify works)
- [x] 2.4 Write `project_path(name) -> PathBuf` and `spec_path(project, id) -> PathBuf` helpers
- [x] 2.5 Write `timestamp_id(feature) -> String` generating `YYYYMMDD_HHMMSS_<feature>` spec IDs
- [x] 2.6 Write `parse_spec_id(dir_name) -> Option<(timestamp, feature)>` for load/delete matching
- [x] 2.7 Write `resolve_spec_id(project, partial) -> Result<String>` for unambiguous partial matching

## 3. Storage — Project Commands

- [x] 3.1 Implement `foundry project init <name>`: validate kebab-case name, create `~/.foundry/<name>/` with empty `vision.md`, `tech-stack.md`, `summary.md`, `specs/`
- [x] 3.2 Implement `foundry project load <name>`: read and print all three files with headers; handle empty files gracefully
- [x] 3.3 Implement `foundry list projects`: list project directory names in `~/.foundry/`; handle empty store
- [x] 3.4 Implement `foundry project delete <name> --confirm`: remove project directory; require `--confirm` flag
- [x] 3.5 Wire all four project subcommands into CLI dispatch in `src/main.rs`

## 4. Storage — Spec Commands

- [x] 4.1 Implement `foundry spec init <project> <feature>`: create timestamped dir with empty `spec.md`, `task-list.md`, `notes.md`
- [x] 4.2 Implement `foundry spec load <project> <id>`: partial-match ID, print all three files with headers
- [x] 4.3 Implement `foundry list specs <project>`: list spec dirs sorted by timestamp ascending
- [x] 4.4 Implement `foundry spec delete <project> <id> --confirm`: partial-match, require `--confirm`
- [x] 4.5 Wire all four spec subcommands into CLI dispatch

## 5. Symlink Bridge

- [x] 5.1 Write project name detection: read `Cargo.toml` → `package.name`
- [x] 5.2 Write project name detection: read `package.json` → `name` (strip `@scope/` prefix)
- [x] 5.3 Write project name detection: read first git remote URL, extract last path segment, strip `.git`
- [x] 5.4 Write project name detection: fall back to current directory name
- [x] 5.5 Write `normalize_to_kebab_case(s) -> Option<String>` for sanitizing detected names
- [x] 5.6 Implement `foundry link [project]`: detect or use provided name, create `.foundry` symlink in CWD, handle replace-existing-symlink and conflict-with-file cases
- [x] 5.7 Implement `.gitignore` advisory in `foundry link`: auto-append `.foundry` if `.gitignore` exists and lacks it; print reminder if no `.gitignore`
- [x] 5.8 Wire `foundry link` into CLI dispatch

## 6. Git Backup

- [x] 6.1 Write `git_available() -> bool` (check `git --version` via subprocess)
- [x] 6.2 Write `is_git_repo(path) -> bool` (check for `.git/` directory)
- [x] 6.3 Write `git_run(args, cwd) -> Result<()>` helper for subprocess git calls with clear error messages
- [x] 6.4 Implement `foundry git init`: check git available, run `git init` in `~/.foundry/`
- [x] 6.5 Implement `foundry git remote <url>`: set/update `origin` remote on `~/.foundry/` repo
- [x] 6.6 Implement `foundry git sync "<message>"`: `git add -A`, `git commit -m`, `git push origin`; handle nothing-to-commit and no-remote cases
- [x] 6.7 Add auto-commit helper: after `project init` and `spec init`, if `~/.foundry/` is a git repo, stage and commit new files with generated message
- [x] 6.8 Wire `foundry git` subcommand group into CLI dispatch

## 7. Skill Content

- [x] 7.1 Write `foundry:load` skill file (`assets/skills/foundry_load.md`): instructs agent to run `foundry project load <project>` and/or `foundry spec load` and read `.foundry/` files into context
- [x] 7.2 Write `foundry:new` skill file (`assets/skills/foundry_new.md`): instructs agent to run `foundry spec init <project> <feature>`, then read and populate the scaffolded files via the `.foundry` symlink
- [x] 7.3 Write `foundry:done` skill file (`assets/skills/foundry_done.md`): instructs agent to summarize completed spec work, then run `foundry git sync "<summary message>"`
- [x] 7.4 Embed all three skill files in the binary via `include_str!` in `src/skills/mod.rs`

## 8. Skill Installer

- [x] 8.1 Implement `foundry install claude-code`: write bundled skill files to `~/.claude/skills/`
- [x] 8.2 Implement `foundry install cursor`: write bundled skill files to `.cursor/rules/` in CWD
- [x] 8.3 Implement `foundry update`: re-write all skill files for all detected installed targets
- [x] 8.4 Implement `foundry uninstall <target>`: remove only foundry-owned skill files (track by filename prefix `foundry_`)
- [x] 8.5 Wire `foundry install`, `foundry update`, `foundry uninstall` into CLI dispatch

## 9. Status Command

- [x] 9.1 Implement `foundry status`: show foundry store path, project count, git init status, installed skill targets
- [x] 9.2 Wire `foundry status` into CLI dispatch

## 10. Tests

- [x] 10.1 Set up `TestEnvironment` (temp `HOME`, temp foundry dir) for the new codebase — adapt or port from existing `src/test_environment.rs`
- [x] 10.2 Unit tests for kebab-case validation and name normalization
- [x] 10.3 Unit tests for spec ID timestamp generation and partial matching
- [x] 10.4 Integration tests for `project init` / `project load` / `list projects` / `project delete`
- [x] 10.5 Integration tests for `spec init` / `spec load` / `list specs` / `spec delete`
- [x] 10.6 Integration tests for `foundry link` (symlink creation, auto-detect, `.gitignore` advisory)
- [x] 10.7 Integration tests for `foundry git` commands (mock git subprocess or use real git in temp dir)
- [x] 10.8 Unit tests for skill content embedding (verify `include_str!` files are non-empty)
- [x] 10.9 Integration tests for `foundry install` / `foundry update` / `foundry uninstall`
- [x] 10.10 Run `cargo clippy -- -D warnings` and `cargo fmt` — fix all issues

## 11. Cleanup and Documentation

- [x] 11.1 Update `README.md`: remove MCP references, document new CLI interface and skill workflow
- [x] 11.2 Update `CHANGELOG.md`: document breaking changes, migration steps
- [x] 11.3 Delete `docs/backends.md` (no longer relevant)
- [x] 11.4 Update `CLAUDE.md` project overview to reflect new architecture
- [x] 11.5 Bump version in `Cargo.toml` to `0.8.0` (breaking release)
