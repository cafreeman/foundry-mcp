use std::path::PathBuf;

/// A timestamped spec directory under `~/.foundry/<project>/specs/<id>/`.
#[derive(Debug, Clone)]
pub struct Spec {
    pub id: String,
    pub project_name: String,
    pub feature: String,
    pub path: PathBuf,
}
