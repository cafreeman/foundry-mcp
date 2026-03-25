## Why

Foundry's MCP server model creates unnecessary friction: agents are better at editing files directly than through structured MCP tool calls, and the MCP protocol adds complexity (JSON-RPC, tool schemas, response wrapping) that provides no real value when the agent just needs to read and write markdown files. The centralized `~/.foundry/` storage model is genuinely valuable — it keeps project context out of codebases and accessible across tools — but it needs a simpler interface.

## What Changes

- **BREAKING**: Remove MCP server entirely (`src/mcp/` deleted, no more `foundry serve`)
- **BREAKING**: Remove all content arguments from CLI commands — agents write file content directly using their own tools
- **BREAKING**: Remove `update-spec` command and edit engine — no more `edit_commands` syntax
- Full ground-up rewrite as a pure CLI binary (~40% of current code size)
- Add `foundry link` command: creates `.foundry -> ~/.foundry/<project>/` symlink in working directory so agents can read/write foundry files using relative paths
- Add git backup: `~/.foundry/` becomes an optional git repo; foundry auto-commits on scaffold writes; `foundry sync "<message>"` commits agent changes and pushes to remote
- Replace MCP installation with skill file installation: `foundry install claude-code` deploys `.md` skill files to `~/.claude/skills/`; `foundry install cursor` deploys to `.cursor/rules/` in the current working directory
- Ship a lightweight lifecycle skill set: `foundry:load`, `foundry:new`, `foundry:done` — skills direct agents through the workflow and call `foundry sync` at appropriate points
- Remove: `analyze-project`, `validate-content`, `get-foundry-help` commands (workflow guidance now lives in skills)

## Capabilities

### New Capabilities
- `storage`: Project and spec CRUD — init scaffolding (directory + empty files), load (cat to stdout), list, delete. No content arguments anywhere.
- `symlink-bridge`: `foundry link [project]` creates `.foundry -> ~/.foundry/<project>/` symlink in CWD. Auto-detects project name from `Cargo.toml`/`package.json`/git remote/directory name; explicit name overrides.
- `git-backup`: `foundry git init` turns `~/.foundry/` into a git repo. Auto-commits on every `foundry project init` / `foundry spec init`. `foundry git sync "<message>"` stages all changes, commits, and pushes to configured remote.
- `skill-installer`: `foundry install <target>` writes skill `.md` files into the target AI tool's skills directory. `foundry update` refreshes installed skill files to the latest version bundled in the binary. Supports `claude-code` and `cursor` targets.

### Modified Capabilities

_None — existing specs directory is empty._

## Impact

- **Breaking for existing users**: All MCP installations (Claude Code, Cursor) stop working; users must run `foundry uninstall` on old version then `foundry install` on new
- **Removes**: `src/mcp/`, `edit_engine.rs`, backends abstraction (`FilesystemBackend`/`MemoryBackend` traits), `FoundryResponse` wrapper type, `next_steps`/`workflow_hints` fields, `validate_content` op, `get_foundry_help` op, `analyze_project` op
- **Adds**: `git` subcommand (shells out to system `git`), `link` subcommand (creates symlink), `skill-installer` module, bundled skill `.md` files as binary assets
- **Dependencies**: Removes `rust-mcp-sdk`; adds nothing (git via subprocess, symlinks via stdlib)
- **Rust edition/toolchain**: Unchanged (2024 edition, strict Clippy)
