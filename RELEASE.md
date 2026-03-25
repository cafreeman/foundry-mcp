# Release process

Manual releases using [cargo-release](https://github.com/crate-ci/cargo-release). The repository is a **single package** (`foundry-mcp` on crates.io; binary name `foundry`).

## Versioning and changelog (between releases)

- **`Cargo.toml` / `Cargo.lock`**: Keep the **version that matches the latest published crates.io release** (what users already have). Do **not** bump the crate version for unreleased work—**`cargo release`** applies the next version when you ship.
- **`CHANGELOG.md`**: Put all unreleased notes under **`## [Unreleased]`** only. Do **not** add a dated `## [x.y.z] - …` section by hand; **`release.toml`** pre-release replacements create that header and fix compare links when you run **`cargo release … --execute`**.
- **Footer link**: `[Unreleased]` should compare **`v<last-published>...HEAD`** (e.g. after 0.8.0 shipped, `…/compare/v0.8.0...HEAD`). After a release, cargo-release rewrites this to `v<new>...HEAD` and adds the new version link.

## Prerequisites

1. Install cargo-release:

   ```bash
   cargo install cargo-release
   ```

2. Clean working tree and correct branch:

   ```bash
   git status
   git branch
   ```

3. crates.io auth if needed: `cargo login`

## Pre-release checks

Run locally before `cargo release` (hooks do not replace these):

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release
```

## Release

Update `CHANGELOG.md` under `## [Unreleased]` as needed. Then:

```bash
# dry run
cargo release patch

# execute (use minor/major when appropriate)
cargo release patch --execute
```

Without `--execute`, cargo-release stays in dry-run mode.

## After release

1. Confirm the git tag was pushed.
2. Confirm the crate version appears on crates.io.
3. Skim `CHANGELOG.md` for the new section and compare links.

## Troubleshooting

- **Uncommitted changes**: commit or stash before release.
- **Missing CHANGELOG.md**: add one matching the patterns in `release.toml` replacements.
- **Auth errors**: `cargo login` with a valid API token.

## Configuration

- **`release.toml`**: tag/commit messages, changelog replacement rules, `publish = true` for this crate.

## Rollback

- **Before publish**: reset the release commit and delete the local tag; remove remote tag if it was pushed.
- **After publish**: yank only if absolutely necessary; normally ship a follow-up patch.

## Checklist

- [ ] Tests and Clippy clean
- [ ] `CHANGELOG.md` updated for the release
- [ ] `cargo release {level} --execute` completed
- [ ] Tag on GitHub; crate on crates.io
