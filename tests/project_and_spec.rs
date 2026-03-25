mod common;

use assert_fs::TempDir;
use common::{err_utf8, run_foundry, utf8};
use std::fs;
use std::path::Path;

fn isolated_home(temp: &TempDir) -> std::path::PathBuf {
    let home = temp.path().join("home");
    fs::create_dir_all(&home).unwrap();
    home
}

#[test]
fn project_init_load_list_delete() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);

    let o = run_foundry(&home, &["project", "init", "my-project"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    let created = utf8(&o).trim().to_string();
    assert!(Path::new(&created).join("vision.md").exists());

    let o = run_foundry(&home, &["project", "load", "my-project"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    let text = utf8(&o);
    assert!(text.contains("=== vision.md ==="));
    assert!(text.contains("(empty)"));

    let o = run_foundry(&home, &["project", "list"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    assert_eq!(utf8(&o).lines().filter(|l| !l.is_empty()).count(), 1);

    let o = run_foundry(&home, &["project", "delete", "my-project"]);
    assert!(!o.status.success());

    let o = run_foundry(&home, &["project", "delete", "my-project", "--confirm"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
}

#[test]
fn project_list_empty_store() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);

    let o = run_foundry(&home, &["project", "list"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    assert!(utf8(&o).contains("No projects found"));
}

#[test]
fn spec_init_load_list_delete() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);

    assert!(
        run_foundry(&home, &["project", "init", "p1"])
            .status
            .success()
    );

    let o = run_foundry(&home, &["spec", "init", "p1", "auth-flow"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    let spec_path = utf8(&o).trim().to_string();
    assert!(spec_path.contains("auth-flow"));

    let o = run_foundry(&home, &["spec", "list", "p1"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    let list_out = utf8(&o);
    let id_line = list_out.lines().find(|l| l.contains("auth-flow")).unwrap();
    let id = id_line.trim();

    let o = run_foundry(&home, &["spec", "load", "p1", "auth-flow"]);
    assert!(o.status.success(), "{}", err_utf8(&o));

    let o = run_foundry(&home, &["spec", "load", "p1", id]);
    assert!(o.status.success(), "{}", err_utf8(&o));

    let o = run_foundry(&home, &["spec", "delete", "p1", "auth-flow"]);
    assert!(!o.status.success());

    let o = run_foundry(&home, &["spec", "delete", "p1", "auth-flow", "--confirm"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
}
