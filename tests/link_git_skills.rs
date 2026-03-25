mod common;

use assert_fs::TempDir;
use common::{err_utf8, run_foundry, utf8};
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn isolated_home(temp: &TempDir) -> std::path::PathBuf {
    let home = temp.path().join("home");
    fs::create_dir_all(&home).unwrap();
    home
}

fn run_foundry_in_dir(home: &Path, cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_foundry"))
        .args(args)
        .env("HOME", home)
        .current_dir(cwd)
        .output()
        .expect("spawn foundry")
}

fn run_foundry_with_env(home: &Path, env_pairs: &[(&str, &str)], args: &[&str]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_foundry"));
    cmd.args(args).env("HOME", home);
    for (key, value) in env_pairs {
        cmd.env(key, value);
    }
    cmd.output().expect("spawn foundry")
}

#[test]
fn link_creates_symlink_and_gitignore_append() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);
    let cwd = temp.path().join("repo");
    fs::create_dir_all(&cwd).unwrap();
    fs::write(cwd.join(".gitignore"), "node_modules\n").unwrap();

    assert!(
        run_foundry(&home, &["project", "init", "my-app"])
            .status
            .success()
    );

    let o = Command::new(env!("CARGO_BIN_EXE_foundry"))
        .args(["link", "my-app"])
        .env("HOME", &home)
        .current_dir(&cwd)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", err_utf8(&o));

    let link = cwd.join(".foundry");
    assert!(link.is_symlink());
    let gi = fs::read_to_string(cwd.join(".gitignore")).unwrap();
    assert!(gi.contains(".foundry"));
    let msg = utf8(&o);
    assert!(msg.contains("Added .foundry to .gitignore") || msg.contains("Linked"));
}

#[test]
fn link_replace_existing_symlink() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);
    let cwd = temp.path().join("repo");
    fs::create_dir_all(&cwd).unwrap();

    assert!(
        run_foundry(&home, &["project", "init", "a"])
            .status
            .success()
    );
    assert!(
        run_foundry(&home, &["project", "init", "b"])
            .status
            .success()
    );

    assert!(
        Command::new(env!("CARGO_BIN_EXE_foundry"))
            .args(["link", "a"])
            .env("HOME", &home)
            .current_dir(&cwd)
            .output()
            .unwrap()
            .status
            .success()
    );

    let o = Command::new(env!("CARGO_BIN_EXE_foundry"))
        .args(["link", "b"])
        .env("HOME", &home)
        .current_dir(&cwd)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", err_utf8(&o));
    assert!(utf8(&o).contains("Replaced") || utf8(&o).contains("Linked"));
}

#[test]
fn git_init_and_sync_smoke() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);

    assert!(
        run_foundry(&home, &["project", "init", "x"])
            .status
            .success()
    );

    let o = run_foundry(&home, &["git", "init"]);
    assert!(o.status.success(), "{}", err_utf8(&o));

    let foundry = home.join(".foundry");
    let _ = Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&foundry)
        .output();
    let _ = Command::new("git")
        .args(["config", "user.name", "test"])
        .current_dir(&foundry)
        .output();

    let o = run_foundry(&home, &["git", "sync", "first"]);
    assert!(o.status.success(), "{}", err_utf8(&o));

    let o = run_foundry(&home, &["git", "sync", "second"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    assert!(utf8(&o).contains("Nothing to sync"));
}

#[test]
fn install_uninstall_claude_code_skills() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);

    let skills = home.join(".claude").join("skills");

    let o = run_foundry(&home, &["install", "claude-code"]);
    assert!(o.status.success(), "{}", err_utf8(&o));

    assert!(skills.join("foundry_load.md").exists());
    assert!(skills.join("foundry_work.md").exists());

    let o = run_foundry(&home, &["uninstall", "claude-code"]);
    assert!(o.status.success(), "{}", err_utf8(&o));

    assert!(!skills.join("foundry_load.md").exists());
    assert!(!skills.join("foundry_work.md").exists());
}

#[test]
fn install_update_cursor_skills() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);
    let cwd = temp.path().join("proj");
    fs::create_dir_all(&cwd).unwrap();

    let o = Command::new(env!("CARGO_BIN_EXE_foundry"))
        .args(["install", "cursor"])
        .env("HOME", &home)
        .current_dir(&cwd)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", err_utf8(&o));

    let rules = cwd.join(".cursor").join("rules");
    assert!(rules.join("foundry_load.md").exists());
    assert!(rules.join("foundry_work.md").exists());

    let o = Command::new(env!("CARGO_BIN_EXE_foundry"))
        .args(["update"])
        .env("HOME", &home)
        .current_dir(&cwd)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", err_utf8(&o));

    let o = Command::new(env!("CARGO_BIN_EXE_foundry"))
        .args(["uninstall", "cursor"])
        .env("HOME", &home)
        .current_dir(&cwd)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", err_utf8(&o));
}

#[test]
fn status_shows_store_and_git() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);

    let o = run_foundry(&home, &["status"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    let t = utf8(&o);
    assert!(t.contains("store:"));
    assert!(t.contains("git:"));
}

#[test]
fn link_detects_project_from_cargo_toml() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);
    let cwd = temp.path().join("crate-root");
    fs::create_dir_all(&cwd).unwrap();
    fs::write(
        cwd.join("Cargo.toml"),
        r#"[package]
name = "detected-crate"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    assert!(
        run_foundry(&home, &["project", "init", "detected-crate"])
            .status
            .success()
    );

    let o = run_foundry_in_dir(&home, &cwd, &["link"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    assert!(cwd.join(".foundry").is_symlink());
}

#[test]
fn link_detects_project_from_package_json_scope() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);
    let cwd = temp.path().join("js-root");
    fs::create_dir_all(&cwd).unwrap();
    fs::write(
        cwd.join("package.json"),
        r#"{"name": "@org/my-app", "version": "1.0.0"}"#,
    )
    .unwrap();

    assert!(
        run_foundry(&home, &["project", "init", "my-app"])
            .status
            .success()
    );

    let o = run_foundry_in_dir(&home, &cwd, &["link"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    assert!(cwd.join(".foundry").is_symlink());
}

#[test]
fn link_detects_project_from_git_remote() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);
    let cwd = temp.path().join("git-only");
    fs::create_dir_all(&cwd).unwrap();

    assert!(
        Command::new("git")
            .args(["init"])
            .current_dir(&cwd)
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("git")
            .args([
                "remote",
                "add",
                "origin",
                "https://github.com/user/my-repo.git",
            ])
            .current_dir(&cwd)
            .status()
            .unwrap()
            .success()
    );

    assert!(
        run_foundry(&home, &["project", "init", "my-repo"])
            .status
            .success()
    );

    let o = run_foundry_in_dir(&home, &cwd, &["link"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    assert!(cwd.join(".foundry").is_symlink());
}

#[test]
fn link_detects_project_from_directory_name() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);
    let cwd = temp.path().join("linked-proj");
    fs::create_dir_all(&cwd).unwrap();

    assert!(
        run_foundry(&home, &["project", "init", "linked-proj"])
            .status
            .success()
    );

    let o = run_foundry_in_dir(&home, &cwd, &["link"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    assert!(cwd.join(".foundry").is_symlink());
}

#[test]
fn git_init_fails_when_git_not_on_path() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);
    assert!(
        run_foundry(&home, &["project", "init", "solo"])
            .status
            .success()
    );

    let o = run_foundry_with_env(&home, &[("PATH", "")], &["git", "init"]);
    assert!(
        !o.status.success(),
        "expected failure when git is not discoverable via PATH"
    );
    let combined = format!("{}{}", err_utf8(&o), utf8(&o));
    assert!(
        combined.to_lowercase().contains("git"),
        "expected git-related error, got: {combined}"
    );
}

#[test]
fn install_rejects_unknown_target() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);
    let o = run_foundry(&home, &["install", "vscode"]);
    assert!(!o.status.success());
    let msg = format!("{}{}", err_utf8(&o), utf8(&o));
    assert!(
        msg.contains("claude-code") && msg.contains("cursor"),
        "expected supported targets in error: {msg}"
    );
}
