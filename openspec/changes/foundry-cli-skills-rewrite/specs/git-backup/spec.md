## ADDED Requirements

### Requirement: Git repository initialization
`foundry git init` SHALL initialize `~/.foundry/` as a git repository (running `git init` if not already a repo). If `~/.foundry/` is already a git repository, the command SHALL report that and exit 0 without re-initializing. The command SHALL fail with a clear error if `git` is not available in `PATH`.

#### Scenario: Initialize fresh repo
- **WHEN** user runs `foundry git init` and `~/.foundry/` is not a git repo
- **THEN** `git init` is run in `~/.foundry/`; exit 0 with confirmation

#### Scenario: Already a git repo
- **WHEN** `~/.foundry/` is already a git repository
- **THEN** exit 0 with a notice that git is already initialized; no changes made

#### Scenario: Git not available
- **WHEN** `git` is not found in PATH
- **THEN** exit non-zero with a message explaining that git must be installed

### Requirement: Remote configuration
`foundry git remote <url>` SHALL set the `origin` remote of the `~/.foundry/` git repository to the given URL. If an `origin` remote already exists, it SHALL be updated (overwritten). The command SHALL require `foundry git init` to have been run first.

#### Scenario: Set remote on initialized repo
- **WHEN** user runs `foundry git remote https://github.com/user/foundry-backup.git` and `~/.foundry/` is a git repo
- **THEN** `origin` remote is set to the URL; exit 0

#### Scenario: Update existing remote
- **WHEN** an `origin` remote already exists
- **THEN** it is replaced with the new URL; exit 0 with notice of update

#### Scenario: Remote set on non-initialized repo
- **WHEN** `~/.foundry/` is not a git repo
- **THEN** exit non-zero instructing the user to run `foundry git init` first

### Requirement: Auto-commit on scaffold writes
Whenever `foundry project init` or `foundry spec init` creates files in `~/.foundry/`, and `~/.foundry/` is a git repository, foundry SHALL automatically stage and commit the newly created files with a descriptive commit message (e.g., `"foundry: init project my-project"` or `"foundry: init spec my-project/auth-flow"`). If `~/.foundry/` is not a git repo, scaffold commands SHALL succeed silently without any git operations.

#### Scenario: Auto-commit on project init
- **WHEN** user runs `foundry project init my-project` and `~/.foundry/` is a git repo
- **THEN** the new project files are staged and committed with message `"foundry: init project my-project"`; exit 0

#### Scenario: Auto-commit on spec init
- **WHEN** user runs `foundry spec init my-project auth-flow` and `~/.foundry/` is a git repo
- **THEN** the new spec files are staged and committed with an appropriate message; exit 0

#### Scenario: No git repo — scaffold succeeds silently
- **WHEN** user runs `foundry project init my-project` and `~/.foundry/` is NOT a git repo
- **THEN** files are created normally; no git operations; no error or warning; exit 0

### Requirement: On-demand sync
`foundry git sync "<message>"` SHALL stage all changes in `~/.foundry/` (`git add -A`), commit with the provided message, and push to the `origin` remote. If there are no changes to commit, the command SHALL exit 0 with a notice. If no remote is configured, the command SHALL commit locally and warn that no remote is set.

#### Scenario: Sync with changes and remote
- **WHEN** user runs `foundry git sync "updated auth spec"` and there are uncommitted changes and an origin remote
- **THEN** all changes are staged, committed with the given message, and pushed; exit 0

#### Scenario: Nothing to sync
- **WHEN** there are no uncommitted changes in `~/.foundry/`
- **THEN** exit 0 with message "Nothing to sync"

#### Scenario: Sync without remote
- **WHEN** no `origin` remote is configured
- **THEN** changes are committed locally; exit 0 with a warning that no remote is set and suggesting `foundry git remote <url>`

#### Scenario: Sync on non-initialized repo
- **WHEN** `~/.foundry/` is not a git repo
- **THEN** exit non-zero instructing the user to run `foundry git init` first

#### Scenario: Message argument is required
- **WHEN** user runs `foundry git sync` with no message
- **THEN** exit non-zero with usage error; no git operations performed
