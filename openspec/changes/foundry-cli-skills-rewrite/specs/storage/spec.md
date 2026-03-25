## ADDED Requirements

### Requirement: Project scaffolding
`foundry project init <name>` SHALL create a new project directory at `~/.foundry/<name>/` containing empty files `vision.md`, `tech-stack.md`, `summary.md`, and an empty `specs/` subdirectory. The command SHALL accept no content arguments. Project names SHALL be validated as kebab-case (lowercase letters, digits, hyphens; no leading/trailing/consecutive hyphens).

#### Scenario: Successful project init
- **WHEN** user runs `foundry project init my-project` and `~/.foundry/my-project/` does not exist
- **THEN** the directory is created with `vision.md`, `tech-stack.md`, `summary.md` (all empty), and `specs/` subdirectory; the command exits 0 and prints JSON to stdout with at least `name` and `path` fields

#### Scenario: Duplicate project name
- **WHEN** user runs `foundry project init my-project` and `~/.foundry/my-project/` already exists
- **THEN** the command exits non-zero with an error message indicating the project already exists; no files are modified

#### Scenario: Invalid project name
- **WHEN** user runs `foundry project init My_Project` (uppercase or underscore)
- **THEN** the command exits non-zero with a message explaining kebab-case requirement

### Requirement: Project loading
`foundry project load <name>` SHALL print the full contents of `vision.md`, `tech-stack.md`, and `summary.md` to stdout, separated by clear headers identifying each file. If a file is empty, the header SHALL still appear with a note that the file is empty.

#### Scenario: Load populated project
- **WHEN** user runs `foundry project load my-project` and the project files contain content
- **THEN** stdout contains all three files' content with identifying headers; exit 0

#### Scenario: Load project with empty files
- **WHEN** user runs `foundry project load my-project` and all files are empty
- **THEN** stdout contains the three headers each noting the file is empty; exit 0

#### Scenario: Load nonexistent project
- **WHEN** user runs `foundry project load no-such-project`
- **THEN** exit non-zero with a clear error; nothing written to stdout

### Requirement: Project listing
`foundry list projects` SHALL print a JSON array to stdout describing all projects in `~/.foundry/` (each entry includes at least `name` and `path`). If no projects exist, the command SHALL print an empty JSON array `[]` and exit 0.

#### Scenario: List multiple projects
- **WHEN** `~/.foundry/` contains `alpha/`, `beta/`, `gamma/`
- **THEN** stdout is JSON listing those three projects; exit 0

#### Scenario: Empty store
- **WHEN** `~/.foundry/` exists but contains no project directories
- **THEN** stdout is `[]`; exit 0

### Requirement: Project deletion
`foundry project delete <name>` SHALL remove `~/.foundry/<name>/` and all its contents after printing the path being deleted. The command SHALL require a `--confirm` flag to prevent accidental deletion.

#### Scenario: Delete with confirmation flag
- **WHEN** user runs `foundry project delete my-project --confirm`
- **THEN** `~/.foundry/my-project/` is removed; exit 0 with confirmation message

#### Scenario: Delete without confirmation flag
- **WHEN** user runs `foundry project delete my-project` (no `--confirm`)
- **THEN** exit non-zero with a message explaining `--confirm` is required; nothing deleted

#### Scenario: Delete nonexistent project
- **WHEN** user runs `foundry project delete no-such-project --confirm`
- **THEN** exit non-zero with error; nothing deleted

### Requirement: Spec scaffolding
`foundry spec init <project> <feature>` SHALL create a new spec directory at `~/.foundry/<project>/specs/<YYYYMMDD_HHMMSS>_<feature>/` containing empty files `spec.md`, `task-list.md`, and `notes.md`. The timestamp prefix SHALL use the current UTC time. The command SHALL accept no content arguments.

#### Scenario: Successful spec init
- **WHEN** user runs `foundry spec init my-project auth-flow` and `my-project` exists
- **THEN** a timestamped directory is created under `~/.foundry/my-project/specs/` with the three empty files; stdout prints JSON with at least `id`, `project`, `feature`, and `path`; exit 0

#### Scenario: Spec init for nonexistent project
- **WHEN** user runs `foundry spec init no-such-project auth-flow`
- **THEN** exit non-zero with error; no directories created

### Requirement: Spec loading
`foundry spec load <project> <spec-id>` SHALL print the contents of `spec.md`, `task-list.md`, and `notes.md` from the identified spec directory to stdout with identifying headers. `<spec-id>` SHALL match by prefix (the timestamp+feature directory name) and support partial matches if unambiguous.

#### Scenario: Load spec by full ID
- **WHEN** user runs `foundry spec load my-project 20240315_120000_auth-flow`
- **THEN** stdout contains all three files' content with headers; exit 0

#### Scenario: Load spec by unambiguous partial ID
- **WHEN** user runs `foundry spec load my-project auth-flow` and only one spec matches
- **THEN** that spec's files are printed to stdout; exit 0

#### Scenario: Ambiguous partial ID
- **WHEN** user runs `foundry spec load my-project auth` and multiple specs match
- **THEN** exit non-zero listing the matching spec IDs; user must be more specific

### Requirement: Spec listing
`foundry list specs <project>` SHALL print a JSON array to stdout for all spec directories under `~/.foundry/<project>/specs/`, sorted by timestamp ascending (each entry includes at least `id` and `path`).

#### Scenario: List specs
- **WHEN** `~/.foundry/my-project/specs/` contains multiple spec directories
- **THEN** stdout is JSON listing them in chronological order; exit 0

#### Scenario: No specs
- **WHEN** `~/.foundry/my-project/specs/` is empty
- **THEN** stdout is `[]`; exit 0

### Requirement: Spec deletion
`foundry spec delete <project> <spec-id>` SHALL remove the identified spec directory and its contents. Requires `--confirm` flag. Supports same partial-match logic as spec load.

#### Scenario: Delete spec with confirmation
- **WHEN** user runs `foundry spec delete my-project auth-flow --confirm` and the spec is unambiguous
- **THEN** the spec directory is removed; exit 0 with confirmation

#### Scenario: Delete without confirmation
- **WHEN** user runs `foundry spec delete my-project auth-flow` (no `--confirm`)
- **THEN** exit non-zero; nothing deleted
