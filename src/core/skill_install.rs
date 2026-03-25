use crate::core::fsutil;
use crate::core::paths;
use crate::skills::SKILL_FILES;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const STATE_FILE: &str = ".foundry-skill-installs.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct InstallState {
    #[serde(default)]
    claude_code: bool,
    #[serde(default)]
    cursor_roots: Vec<String>,
}

fn state_path() -> Result<PathBuf> {
    Ok(paths::ensure_foundry_dir()?.join(STATE_FILE))
}

fn load_state() -> Result<InstallState> {
    let path = state_path()?;
    if !path.exists() {
        return Ok(InstallState::default());
    }
    let text =
        fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
    serde_json::from_str(&text).context("Failed to parse skill install state")
}

fn save_state(state: &InstallState) -> Result<()> {
    let path = state_path()?;
    let text =
        serde_json::to_string_pretty(state).context("Failed to serialize skill install state")?;
    fsutil::write_file_atomic(&path, &text)?;
    Ok(())
}

fn claude_skills_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".claude").join("skills"))
}

fn cursor_rules_dir(cwd: &Path) -> PathBuf {
    cwd.join(".cursor").join("rules")
}

pub fn install(target: &str, cwd: &Path) -> Result<()> {
    match target {
        "claude-code" => install_claude_code(),
        "cursor" => install_cursor(cwd),
        _ => bail!(
            "Unknown install target {:?}; supported: claude-code, cursor",
            target
        ),
    }
}

fn install_claude_code() -> Result<()> {
    let dir = claude_skills_dir()?;
    fs::create_dir_all(&dir).with_context(|| format!("Failed to create {}", dir.display()))?;

    let mut state = load_state()?;
    state.claude_code = true;
    save_state(&state)?;

    for (filename, content) in SKILL_FILES {
        let path = dir.join(filename);
        let existed = path.exists();
        fsutil::write_file_atomic(&path, content)?;
        println!(
            "Wrote {}{}",
            path.display(),
            if existed { " (overwritten)" } else { "" }
        );
    }

    Ok(())
}

fn install_cursor(cwd: &Path) -> Result<()> {
    let dir = cursor_rules_dir(cwd);
    fs::create_dir_all(&dir).with_context(|| format!("Failed to create {}", dir.display()))?;

    let root = cwd
        .canonicalize()
        .with_context(|| format!("Failed to canonicalize {}", cwd.display()))?;

    let mut state = load_state()?;
    if !state
        .cursor_roots
        .iter()
        .any(|p| Path::new(p) == root.as_path())
    {
        state.cursor_roots.push(root.to_string_lossy().into_owned());
    }
    save_state(&state)?;

    for (filename, content) in SKILL_FILES {
        let path = dir.join(filename);
        let existed = path.exists();
        fsutil::write_file_atomic(&path, content)?;
        println!(
            "Wrote {}{}",
            path.display(),
            if existed { " (overwritten)" } else { "" }
        );
    }

    Ok(())
}

pub fn update() -> Result<()> {
    let state = load_state()?;
    let mut updated_any = false;

    if state.claude_code {
        let dir = claude_skills_dir()?;
        if dir.is_dir() {
            for (filename, content) in SKILL_FILES {
                let path = dir.join(filename);
                if path.exists() {
                    fsutil::write_file_atomic(&path, content)?;
                    println!("Updated {}", path.display());
                    updated_any = true;
                }
            }
        }
    }

    for root in &state.cursor_roots {
        let p = Path::new(root);
        if !p.is_dir() {
            continue;
        }
        let dir = cursor_rules_dir(p);
        if !dir.is_dir() {
            continue;
        }

        let mut any_here = false;
        for (filename, content) in SKILL_FILES {
            let path = dir.join(filename);
            if path.exists() {
                fsutil::write_file_atomic(&path, content)?;
                println!("Updated {}", path.display());
                any_here = true;
            }
        }
        updated_any |= any_here;
    }

    if !updated_any {
        println!("No installed skills found");
    }

    Ok(())
}

pub fn uninstall(target: &str, cwd: &Path) -> Result<()> {
    match target {
        "claude-code" => uninstall_claude_code(),
        "cursor" => uninstall_cursor(cwd),
        _ => bail!(
            "Unknown uninstall target {:?}; supported: claude-code, cursor",
            target
        ),
    }
}

fn uninstall_claude_code() -> Result<()> {
    let dir = claude_skills_dir()?;
    if !dir.is_dir() {
        println!("No foundry skills found for claude-code");
        return Ok(());
    }

    let mut removed = 0usize;
    for entry in fs::read_dir(&dir).with_context(|| format!("Failed to read {}", dir.display()))? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with("foundry_") && name.ends_with(".md") {
            fs::remove_file(entry.path())
                .with_context(|| format!("Failed to remove {}", entry.path().display()))?;
            removed += 1;
        }
    }

    if removed == 0 {
        println!("No foundry skills found for claude-code");
    } else {
        println!(
            "Removed {removed} foundry skill file(s) from {}",
            dir.display()
        );
    }

    let mut state = load_state()?;
    state.claude_code = false;
    save_state(&state)?;
    Ok(())
}

fn uninstall_cursor(cwd: &Path) -> Result<()> {
    let dir = cursor_rules_dir(cwd);
    if !dir.is_dir() {
        println!("No foundry skills found for cursor");
        return Ok(());
    }

    let mut removed = 0usize;
    for entry in fs::read_dir(&dir).with_context(|| format!("Failed to read {}", dir.display()))? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with("foundry_") && name.ends_with(".md") {
            fs::remove_file(entry.path())
                .with_context(|| format!("Failed to remove {}", entry.path().display()))?;
            removed += 1;
        }
    }

    if removed == 0 {
        println!("No foundry skills found for cursor");
    } else {
        println!(
            "Removed {removed} foundry skill file(s) from {}",
            dir.display()
        );
    }

    let root = cwd.canonicalize().ok();
    if let Some(root) = root {
        let mut state = load_state()?;
        state
            .cursor_roots
            .retain(|p| Path::new(p) != root.as_path());
        save_state(&state)?;
    }

    Ok(())
}

pub fn installed_targets_summary() -> Result<String> {
    let state = load_state()?;
    let mut parts = Vec::new();
    if state.claude_code {
        parts.push("claude-code".to_string());
    }
    if !state.cursor_roots.is_empty() {
        parts.push("cursor".to_string());
    }
    Ok(parts.join(", "))
}

#[derive(Debug, Clone, Serialize)]
pub struct InstallStatus {
    pub claude_code: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cursor_roots: Vec<String>,
}

pub fn install_status() -> Result<InstallStatus> {
    let state = load_state()?;
    Ok(InstallStatus {
        claude_code: state.claude_code,
        cursor_roots: state.cursor_roots,
    })
}
