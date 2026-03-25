mod common;

use assert_fs::TempDir;
use common::{err_utf8, run_foundry, utf8};
use std::fs;
use std::process::Command;

fn isolated_home(temp: &TempDir) -> std::path::PathBuf {
    let home = temp.path().join("home");
    fs::create_dir_all(&home).unwrap();
    home
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

    let o = run_foundry(&home, &["uninstall", "claude-code"]);
    assert!(o.status.success(), "{}", err_utf8(&o));

    assert!(!skills.join("foundry_load.md").exists());
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

    let p = cwd.join(".cursor").join("rules").join("foundry_load.md");
    assert!(p.exists());

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
