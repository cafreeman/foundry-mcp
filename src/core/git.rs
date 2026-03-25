use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn is_git_repo(path: &Path) -> bool {
    path.join(".git").is_dir()
}

pub fn git_run(args: &[&str], cwd: &Path) -> Result<()> {
    let status = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .status()
        .with_context(|| {
            format!(
                "Failed to execute `git {}` in {}",
                args.join(" "),
                cwd.display()
            )
        })?;

    if !status.success() {
        bail!(
            "`git {}` failed in {} (status: {:?})",
            args.join(" "),
            cwd.display(),
            status.code()
        );
    }

    Ok(())
}

pub fn init_foundry_repo(foundry_root: &Path) -> Result<()> {
    if !git_available() {
        bail!("git is not available on PATH; install git to use `foundry git init`");
    }

    if is_git_repo(foundry_root) {
        println!(
            "Git repository already initialized at {}",
            foundry_root.display()
        );
        return Ok(());
    }

    git_run(&["init"], foundry_root)?;
    println!("Initialized git repository at {}", foundry_root.display());
    Ok(())
}

pub fn set_origin_remote(foundry_root: &Path, url: &str) -> Result<()> {
    if !is_git_repo(foundry_root) {
        bail!(
            "{} is not a git repository; run `foundry git init` first",
            foundry_root.display()
        );
    }

    // `git remote add` fails if origin exists; fall back to set-url.
    let add = Command::new("git")
        .args(["remote", "add", "origin", url])
        .current_dir(foundry_root)
        .status()
        .context("Failed to run `git remote add`")?;

    if add.success() {
        println!("Set origin remote to {url}");
        return Ok(());
    }

    git_run(&["remote", "set-url", "origin", url], foundry_root)?;
    println!("Updated origin remote to {url}");
    Ok(())
}

pub fn sync(foundry_root: &Path, message: &str) -> Result<()> {
    if message.trim().is_empty() {
        bail!("Sync message must not be empty");
    }

    if !is_git_repo(foundry_root) {
        bail!(
            "{} is not a git repository; run `foundry git init` first",
            foundry_root.display()
        );
    }

    git_run(&["add", "-A"], foundry_root)?;

    let staged_clean = Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .current_dir(foundry_root)
        .status()
        .context("Failed to check staged changes")?
        .success();

    if staged_clean {
        println!("Nothing to sync");
        return Ok(());
    }

    git_run(&["commit", "-m", message], foundry_root)?;

    let has_origin = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(foundry_root)
        .status()
        .context("Failed to check origin remote")?
        .success();

    if !has_origin {
        eprintln!(
            "warning: no origin remote configured; committed locally only. Set one with `foundry git remote <url>`"
        );
        return Ok(());
    }

    git_run(&["push", "origin"], foundry_root)?;
    Ok(())
}

pub fn auto_commit_scaffold(foundry_root: &Path, message: String, paths: &[PathBuf]) -> Result<()> {
    if paths.is_empty() {
        return Ok(());
    }

    if !is_git_repo(foundry_root) {
        return Ok(());
    }

    for p in paths {
        let rel = path_relative_to(p, foundry_root)?;
        git_run(&["add", "--", &rel], foundry_root)?;
    }

    let has_staged = !Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .current_dir(foundry_root)
        .status()
        .context("Failed to check staged changes")?
        .success();

    if !has_staged {
        return Ok(());
    }

    git_run(&["commit", "-m", &message], foundry_root)?;
    Ok(())
}

/// Stage all changes under a project directory (including deletions) and commit if needed.
pub fn commit_project_subtree(
    foundry_root: &Path,
    project_dir: &Path,
    message: &str,
) -> Result<()> {
    if !is_git_repo(foundry_root) {
        return Ok(());
    }

    let rel = path_relative_to(project_dir, foundry_root)?;
    git_run(&["add", "-A", "--", &rel], foundry_root)?;

    let has_staged = !Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .current_dir(foundry_root)
        .status()
        .context("Failed to check staged changes")?
        .success();

    if !has_staged {
        return Ok(());
    }

    git_run(&["commit", "-m", message], foundry_root)?;
    Ok(())
}

fn path_relative_to(path: &Path, root: &Path) -> Result<String> {
    let rel = path
        .strip_prefix(root)
        .with_context(|| format!("Path {} is not within {}", path.display(), root.display()))?;
    Ok(rel.to_string_lossy().replace('\\', "/"))
}
