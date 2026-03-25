//! Bundled skill content embedded at compile time.

pub const FOUNDRY_LOAD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/skills/foundry_load.md"
));
pub const FOUNDRY_NEW: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/skills/foundry_new.md"
));
pub const FOUNDRY_DONE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/skills/foundry_done.md"
));
pub const FOUNDRY_WORK: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/skills/foundry_work.md"
));

pub const SKILL_FILES: [(&str, &str); 4] = [
    ("foundry_load.md", FOUNDRY_LOAD),
    ("foundry_new.md", FOUNDRY_NEW),
    ("foundry_done.md", FOUNDRY_DONE),
    ("foundry_work.md", FOUNDRY_WORK),
];
