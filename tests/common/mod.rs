//! Shared helpers for integration tests (`foundry` binary + isolated `HOME`).
//!
//! Lives under `tests/common/mod.rs` (not `tests/common.rs`) so Cargo does not compile
//! helpers as a separate integration test crate.

use std::path::Path;
use std::process::Output;

/// Run the `foundry` binary with `HOME` set to `home`.
pub fn run_foundry(home: &Path, args: &[&str]) -> Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_foundry"))
        .args(args)
        .env("HOME", home)
        .output()
        .expect("spawn foundry")
}

pub fn utf8(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

pub fn err_utf8(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}
