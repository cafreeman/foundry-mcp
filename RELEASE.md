# Release Process

This document outlines the release process for Foundry MCP, following a simple manual approach using `cargo-release`.

## Overview

Foundry MCP uses a workspace configuration with two crates:

- **`foundry-mcp`** - Main CLI tool and MCP server (PUBLISHED to crates.io)
- **`foundry-mcp-macros`** - Internal procedural macros (NOT PUBLISHED - internal use only)

The release process only publishes the main `foundry-mcp` crate while keeping the macro crate internal.

## Prerequisites

1. **Install cargo-release**:

   ```bash
   cargo install cargo-release
   ```

2. **Ensure clean working directory**:

   ```bash
   git status  # Should show no uncommitted changes
   ```

3. **Verify you're on main branch**:

   ```bash
   git branch  # Should show * main
   ```

4. **Ensure crates.io authentication**:
   ```bash
   cargo login  # If not already authenticated
   ```

## Pre-Release Quality Checks

**CRITICAL**: Run these commands manually before any release due to cargo-release environment limitations:

```bash
# Format check
cargo fmt --check

# Linting with all warnings as errors
cargo clippy --all-targets --all-features -- -D warnings

# Run all tests
cargo test

# Verify release build works
cargo build --release
```

If any of these fail, fix the issues before proceeding.

## Release Process

### 1. Prepare the Release

Update `CHANGELOG.md` manually if needed (cargo-release will auto-update the Unreleased section):

```markdown
## [Unreleased]

### Added

- New feature descriptions

### Changed

- Breaking changes

### Fixed

- Bug fixes
```

### 2. Execute Release

For patch releases (bug fixes):

```bash
cargo release patch --execute
```

For minor releases (new features):

```bash
cargo release minor --execute
```

For major releases (breaking changes):

```bash
cargo release major --execute
```

**Note**: Without `--execute`, cargo-release runs in dry-run mode by default.

### 3. Verify Release

After the release:

1. **Check GitHub**: Verify the tag was created and pushed
2. **Check crates.io**: Verify only `foundry-mcp` was published (not `foundry-mcp-macros`)
3. **Check CHANGELOG.md**: Verify automatic updates were applied correctly

## What cargo-release Does Automatically

1. **Version Bumping**: Updates version in `Cargo.toml` for `foundry-mcp` only
2. **Changelog Updates**: Updates `CHANGELOG.md` with new version and date
3. **Git Operations**: Creates commit with message "Release v{version}"
4. **Tagging**: Creates and pushes git tag "v{version}"
5. **Publishing**: Publishes `foundry-mcp` to crates.io (skips `foundry-mcp-macros`)
6. **GitHub Links**: Updates changelog with proper GitHub compare links

## Internal Package Protection

The `foundry-mcp-macros` crate is protected from accidental publishing through:

1. **`publish = false`** in `foundry-mcp-macros/Cargo.toml`
2. **cargo-release** automatically respects this setting and excludes the crate

This ensures that procedural macros remain internal to the project and are never accidentally published.

## Troubleshooting

### "uncommitted changes detected"

```bash
git add .
git commit -m "Prepare for release"
```

### "unable to find file CHANGELOG.md"

Ensure `CHANGELOG.md` exists in the project root with proper format.

### "authentication required"

```bash
cargo login
# Enter your crates.io API token
```

### "package not found in registry"

For first-time publishing, ensure all metadata in `Cargo.toml` is complete.

### "foundry-mcp-macros was published accidentally"

This should not happen due to `publish = false`, but if it does:

1. Yank the version from crates.io immediately
2. Verify the `publish = false` setting is in place

## Rollback Procedures

### If release fails after version bump but before publishing:

1. Reset to previous commit: `git reset --hard HEAD~1`
2. Delete the tag: `git tag -d v{version} && git push origin :refs/tags/v{version}`
3. Fix the issue and retry

### If release fails after publishing:

1. The crate version is already published and cannot be unpublished
2. Fix the issue and release a patch version
3. Update documentation to note the issue

## Release Checklist

- [ ] All tests passing locally
- [ ] Clean working directory (no uncommitted changes)
- [ ] On main branch
- [ ] Pre-release quality checks completed
- [ ] CHANGELOG.md updated with new features/fixes
- [ ] crates.io authentication verified
- [ ] cargo release {level} --execute executed
- [ ] GitHub tag created and pushed
- [ ] Only foundry-mcp published to crates.io (foundry-mcp-macros excluded)
- [ ] CHANGELOG.md automatically updated with version and date
- [ ] GitHub compare links working correctly

## Configuration Files

- **`release.toml`**: cargo-release configuration (based on proven worktree setup)
- **`foundry-mcp-macros/Cargo.toml`**: Contains `publish = false` for internal package protection
- **`CHANGELOG.md`**: Automatically updated during release process

## Support

For issues with the release process:

1. Check this documentation first
2. Verify configuration matches the working worktree setup
3. Run dry-run mode to test changes: `cargo release {level}` (without --execute)
