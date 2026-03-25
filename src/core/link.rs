use crate::core::names::normalize_to_kebab_case;
use crate::core::paths::project_path;
use crate::core::project::validate_project_name;
use anyhow::{Context, Result, bail};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub fn detect_project_name(cwd: &Path) -> Result<String> {
    if let Some(name) = package_name_from_cargo_toml(&cwd.join("Cargo.toml"))
        && let Some(k) = normalize_to_kebab_case(&name)
    {
        return Ok(k);
    }

    if let Some(name) = package_name_from_package_json(&cwd.join("package.json"))
        && let Some(k) = normalize_to_kebab_case(&name)
    {
        return Ok(k);
    }

    if let Some(name) = first_git_remote_repo_name(cwd)
        && let Some(k) = normalize_to_kebab_case(&name)
    {
        return Ok(k);
    }

    let dir_name = cwd
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();

    normalize_to_kebab_case(&dir_name).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not detect a valid kebab-case project name; pass it explicitly: `foundry link <name>`"
        )
    })
}

fn package_name_from_cargo_toml(path: &Path) -> Option<String> {
    let text = fs::read_to_string(path).ok()?;
    let value: toml::Value = toml::from_str(&text).ok()?;
    value
        .get("package")?
        .get("name")?
        .as_str()
        .map(|s| s.to_string())
}

fn package_name_from_package_json(path: &Path) -> Option<String> {
    let text = fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    let name = value.get("name")?.as_str()?.to_string();
    Some(strip_npm_scope(&name).to_string())
}

fn strip_npm_scope(name: &str) -> &str {
    name.rfind('/').map_or(name, |idx| &name[idx + 1..])
}

fn first_git_remote_repo_name(cwd: &Path) -> Option<String> {
    let url = std::process::Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(cwd)
        .output()
        .ok()
        .filter(|o| o.status.success())?;

    let url = String::from_utf8_lossy(&url.stdout);
    let url = url.trim();
    repo_name_from_remote_url(url)
}

fn repo_name_from_remote_url(url: &str) -> Option<String> {
    let url = url.trim();
    if url.is_empty() {
        return None;
    }

    let without_suffix = url.strip_suffix(".git").unwrap_or(url);
    let last = without_suffix.rsplit(['/', ':']).next()?;
    if last.is_empty() {
        return None;
    }
    Some(last.to_string())
}

pub fn link(cwd: &Path, project: &str) -> Result<()> {
    validate_project_name(project)?;
    let target = project_path(project)?;
    if !target.is_dir() {
        bail!(
            "Project {:?} does not exist at {}",
            project,
            target.display()
        );
    }

    let link_path = cwd.join(".foundry");

    match link_path.symlink_metadata() {
        Ok(meta) => {
            if meta.file_type().is_symlink() {
                fs::remove_file(&link_path)
                    .context("Failed to remove existing .foundry symlink")?;
                println!("Replaced existing .foundry symlink");
            } else {
                bail!(
                    ".foundry exists and is not a symlink; remove it or pick a different location (found at {})",
                    link_path.display()
                );
            }
        }
        Err(_) if link_path.exists() => {
            bail!(
                ".foundry exists and is not a symlink; remove it before linking ({})",
                link_path.display()
            );
        }
        Err(_) => {}
    }

    create_symlink(&target, &link_path)?;
    println!("Linked {} -> {}", link_path.display(), target.display());

    gitignore_advisory(cwd)?;
    Ok(())
}

#[cfg(unix)]
fn create_symlink(target: &Path, link: &Path) -> Result<()> {
    std::os::unix::fs::symlink(target, link).with_context(|| {
        format!(
            "Failed to create symlink {} -> {}",
            link.display(),
            target.display()
        )
    })?;
    Ok(())
}

#[cfg(windows)]
fn create_symlink(target: &Path, link: &Path) -> Result<()> {
    std::os::windows::fs::symlink_dir(target, link).with_context(|| {
        format!(
            "Failed to create symlink {} -> {}",
            link.display(),
            target.display()
        )
    })?;
    Ok(())
}

fn gitignore_advisory(cwd: &Path) -> Result<()> {
    let gitignore = cwd.join(".gitignore");
    if !gitignore.exists() {
        println!("Reminder: add `.foundry` to your `.gitignore` to avoid committing the symlink.");
        return Ok(());
    }

    let text = fs::read_to_string(&gitignore).context("Failed to read .gitignore")?;
    if text.lines().any(|line| line.trim() == ".foundry") {
        return Ok(());
    }

    let mut out = text;
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(".foundry\n");
    fs::write(&gitignore, out).context("Failed to update .gitignore")?;
    println!("Added .foundry to .gitignore");
    Ok(())
}

pub fn current_dir() -> Result<PathBuf> {
    env::current_dir().context("Failed to determine current directory")
}

/// If `./.foundry` is a symlink into `~/.foundry/<project>/`, return that project name.
pub fn resolve_dot_foundry_project(cwd: &Path) -> Result<Option<String>> {
    let link = cwd.join(".foundry");
    let meta = match fs::symlink_metadata(&link) {
        Ok(m) if m.file_type().is_symlink() => m,
        _ => return Ok(None),
    };
    let _ = meta;
    let target = fs::read_link(&link).context("Failed to read .foundry symlink")?;
    let foundry = crate::core::paths::foundry_dir()?;
    let target_abs = if target.is_absolute() {
        fs::canonicalize(&target).ok()
    } else {
        fs::canonicalize(cwd.join(&target)).ok()
    };
    let Some(target_abs) = target_abs else {
        return Ok(None);
    };
    let foundry_canon = fs::canonicalize(&foundry).unwrap_or(foundry);
    let rel = match target_abs.strip_prefix(&foundry_canon) {
        Ok(r) => r,
        Err(_) => return Ok(None),
    };
    let mut components = rel.components();
    let Some(std::path::Component::Normal(first)) = components.next() else {
        return Ok(None);
    };
    Ok(Some(first.to_string_lossy().into_owned()))
}
