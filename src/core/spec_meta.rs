//! Optional `meta.json` in an active spec directory for workflow hints.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const META_FILE: &str = "meta.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpecPhaseHint {
    EmptyScaffold,
    Drafting,
    ReadyForImplementation,
    Implementing,
    CompletedPendingCollapse,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpecMeta {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_hint: Option<SpecPhaseHint>,
}

fn default_version() -> u32 {
    1
}

pub fn read_meta(dir: &Path) -> Result<SpecMeta> {
    let p = dir.join(META_FILE);
    if !p.exists() {
        return Ok(SpecMeta::default());
    }
    let text = fs::read_to_string(&p).with_context(|| format!("Failed to read {}", p.display()))?;
    serde_json::from_str(&text).context("Failed to parse meta.json")
}

pub fn write_meta(dir: &Path, meta: &SpecMeta) -> Result<()> {
    let p = dir.join(META_FILE);
    let text = serde_json::to_string_pretty(meta).context("Failed to serialize meta.json")?;
    fs::write(&p, text).with_context(|| format!("Failed to write {}", p.display()))
}

pub fn default_meta() -> SpecMeta {
    SpecMeta {
        version: 1,
        phase_hint: None,
    }
}
