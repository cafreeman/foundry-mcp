## ADDED Requirements

### Requirement: Symlink creation
`foundry link [project]` SHALL create a symbolic link named `.foundry` in the current working directory pointing to `~/.foundry/<project>/`. If `.foundry` already exists as a symlink, it SHALL be replaced. If `.foundry` exists as a regular file or directory, the command SHALL exit non-zero with an error rather than overwrite.

#### Scenario: Successful link creation
- **WHEN** user runs `foundry link my-project` in `/home/user/code/my-app/` and `~/.foundry/my-project/` exists
- **THEN** `/home/user/code/my-app/.foundry` is created as a symlink to `~/.foundry/my-project/`; exit 0

#### Scenario: Replace existing symlink
- **WHEN** `.foundry` already exists as a symlink pointing to a different project
- **THEN** the symlink is updated to point to the new project; exit 0 with notice of replacement

#### Scenario: Conflict with existing file or directory
- **WHEN** `.foundry` exists as a regular file or non-symlink directory
- **THEN** exit non-zero with error describing the conflict; symlink not created

#### Scenario: Target project does not exist
- **WHEN** user runs `foundry link no-such-project` and `~/.foundry/no-such-project/` does not exist
- **THEN** exit non-zero with error; symlink not created

### Requirement: Project name auto-detection
When `foundry link` is run without a project name argument, it SHALL attempt to detect the project name from the current working directory in this priority order: (1) `Cargo.toml` `[package].name` field, (2) `package.json` `name` field, (3) last path segment of the first git remote URL (stripping `.git` suffix), (4) name of the current working directory. The detected name SHALL be normalized to kebab-case. If no source yields a valid kebab-case name, the command SHALL exit non-zero and prompt the user to provide the name explicitly.

#### Scenario: Detect from Cargo.toml
- **WHEN** user runs `foundry link` in a directory containing `Cargo.toml` with `[package] name = "my-crate"`
- **THEN** the link is created as if `foundry link my-crate` was run

#### Scenario: Detect from package.json
- **WHEN** no `Cargo.toml` exists but `package.json` has `"name": "@org/my-app"`
- **THEN** the name is normalized (e.g. `my-app`) and used for the link

#### Scenario: Detect from git remote
- **WHEN** no manifest files exist but the directory has a git remote `https://github.com/user/my-repo.git`
- **THEN** `my-repo` is used as the project name

#### Scenario: Detect from directory name
- **WHEN** no manifest files and no git remote are found
- **THEN** the current directory name (normalized to kebab-case) is used

#### Scenario: No valid name detected
- **WHEN** detection produces an empty or non-normalizable result
- **THEN** exit non-zero with a message instructing the user to run `foundry link <name>` explicitly

### Requirement: .gitignore advisory
After successfully creating a symlink, `foundry link` SHALL check whether a `.gitignore` file exists in the current directory. If `.foundry` is not already listed in `.gitignore`, the command SHALL automatically append `.foundry` to `.gitignore` and print a notice that it was added. If no `.gitignore` exists, the command SHALL print a reminder to add `.foundry` to version control ignore files.

#### Scenario: .gitignore exists without .foundry entry
- **WHEN** `.gitignore` exists and does not contain `.foundry`
- **THEN** `.foundry` is appended to `.gitignore`; command prints "Added .foundry to .gitignore"

#### Scenario: .gitignore already contains .foundry
- **WHEN** `.gitignore` already lists `.foundry`
- **THEN** no modification to `.gitignore`; no notice printed

#### Scenario: No .gitignore present
- **WHEN** `.gitignore` does not exist in the current directory
- **THEN** symlink is created; command prints a reminder to add `.foundry` to `.gitignore`
