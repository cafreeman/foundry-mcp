use std::path::PathBuf;

/// A Foundry project stored under `~/.foundry/<name>/`.
#[derive(Debug, Clone)]
pub struct Project {
    pub name: String,
    pub path: PathBuf,
}
