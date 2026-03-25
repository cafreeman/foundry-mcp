//! Foundry: CLI library for scaffolding `~/.foundry/` project context.

pub mod core;
pub mod skills;
pub mod types;

/// Resolve `~/.foundry` without creating it.
pub fn foundry_dir() -> anyhow::Result<std::path::PathBuf> {
    crate::core::paths::foundry_dir()
}

#[cfg(test)]
mod tests {
    #[test]
    fn bundled_skills_are_non_empty() {
        assert!(!crate::skills::FOUNDRY_LOAD.trim().is_empty());
        assert!(!crate::skills::FOUNDRY_NEW.trim().is_empty());
        assert!(!crate::skills::FOUNDRY_DONE.trim().is_empty());
    }
}
