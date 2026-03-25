## ADDED Requirements

### Requirement: Skill installation
`foundry install <target>` SHALL write bundled skill `.md` files to the target AI tool's skills directory. Supported targets: `claude-code` (writes to `~/.claude/skills/`), `cursor` (writes to `.cursor/rules/` in the current working directory). Each skill file SHALL be written atomically (write to temp file, rename). If a skill file already exists, it SHALL be overwritten without prompting.

#### Scenario: Install for Claude Code
- **WHEN** user runs `foundry install claude-code`
- **THEN** skill `.md` files are written to `~/.claude/skills/`; exit 0 listing files written

#### Scenario: Install for Cursor
- **WHEN** user runs `foundry install cursor` in a project directory
- **THEN** skill `.md` files are written to `.cursor/rules/` (created if absent); exit 0 listing files written

#### Scenario: Overwrite existing skills
- **WHEN** skill files already exist in the target directory
- **THEN** they are overwritten with the bundled version; exit 0 with notice of overwrite

#### Scenario: Unknown target
- **WHEN** user runs `foundry install vscode` (unsupported target)
- **THEN** exit non-zero listing supported targets

### Requirement: Skill update
`foundry update` SHALL overwrite all previously installed skill files with the current versions bundled in the binary, for all targets where skills have been installed. Targets where foundry has never been installed are skipped silently.

#### Scenario: Update installed skills
- **WHEN** user runs `foundry update` and skills are installed for one or more targets
- **THEN** all installed skill files are overwritten; exit 0 with a summary of what was updated

#### Scenario: Nothing installed
- **WHEN** no skills have been installed for any target
- **THEN** exit 0 with message "No installed skills found"

### Requirement: Skill uninstallation
`foundry uninstall <target>` SHALL remove all skill files previously installed by foundry for the given target. Files not created by foundry SHALL NOT be removed. The command SHALL exit 0 even if no foundry skill files are found for the target.

#### Scenario: Uninstall Claude Code skills
- **WHEN** user runs `foundry uninstall claude-code` and foundry skill files exist in `~/.claude/skills/`
- **THEN** foundry skill files are removed; other files in the directory are untouched; exit 0

#### Scenario: Nothing to uninstall
- **WHEN** no foundry skill files are present for the target
- **THEN** exit 0 with message "No foundry skills found for <target>"

### Requirement: Bundled skill content
The binary SHALL bundle the following skill files as compile-time assets:
- `foundry:load` — loads project context from `.foundry/` into the agent's context window
- `foundry:new` — scaffolds a new spec (calls `foundry spec init`, opens files for agent to write)
- `foundry:done` — archives a completed spec (calls `foundry git sync` with a summary message)

Skill files SHALL be embedded via `include_str!` at compile time. The installed skill version SHALL always match the binary version.

#### Scenario: Skills reflect binary version
- **WHEN** user installs foundry v2.0.0 and runs `foundry install claude-code`
- **THEN** the installed skill files are the versions bundled in v2.0.0

#### Scenario: All three lifecycle skills installed
- **WHEN** `foundry install claude-code` completes successfully
- **THEN** `~/.claude/skills/` contains `foundry_load.md`, `foundry_new.md`, and `foundry_done.md`
