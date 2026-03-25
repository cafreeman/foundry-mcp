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

#[test]
fn project_list_sorts_names() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);

    for name in ["gamma", "alpha", "beta"] {
        assert!(
            run_foundry(&home, &["project", "init", name])
                .status
                .success()
        );
    }

    let o = run_foundry(&home, &["project", "list"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    let out = utf8(&o);
    let lines: Vec<&str> = out.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines, vec!["alpha", "beta", "gamma"]);
}

#[test]
fn project_load_nonexistent_writes_no_stdout() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);

    let o = run_foundry(&home, &["project", "load", "missing-project"]);
    assert!(!o.status.success(), "{}", utf8(&o));
    assert!(
        utf8(&o).trim().is_empty(),
        "stdout should stay empty on error; got {:?}",
        utf8(&o)
    );
}

#[test]
fn project_init_rejects_non_kebab_name() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);

    let o = run_foundry(&home, &["project", "init", "My_Project"]);
    assert!(!o.status.success());
    let msg = format!("{}{}", err_utf8(&o), utf8(&o));
    assert!(
        msg.to_lowercase().contains("kebab"),
        "expected kebab-case hint: {msg}"
    );
}

#[test]
fn spec_list_sorted_by_timestamp_ascending() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);

    assert!(
        run_foundry(&home, &["project", "init", "chrono"])
            .status
            .success()
    );

    let specs = home.join(".foundry").join("chrono").join("specs");
    for (id, label) in [
        ("20240115_120000_early", "early"),
        ("20240615_120000_late", "late"),
    ] {
        let dir = specs.join(id);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("spec.md"), format!("# {label}\n")).unwrap();
        fs::write(dir.join("task-list.md"), "").unwrap();
        fs::write(dir.join("notes.md"), "").unwrap();
    }

    let o = run_foundry(&home, &["spec", "list", "chrono"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    let out = utf8(&o);
    let lines: Vec<&str> = out.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("20240115"));
    assert!(lines[1].contains("20240615"));
}

#[test]
fn spec_load_ambiguous_partial_fails_with_matches() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);

    assert!(
        run_foundry(&home, &["project", "init", "amb"])
            .status
            .success()
    );

    let specs = home.join(".foundry").join("amb").join("specs");
    for id in ["20240315_120000_aa", "20240315_120100_bb"] {
        let dir = specs.join(id);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("spec.md"), "").unwrap();
        fs::write(dir.join("task-list.md"), "").unwrap();
        fs::write(dir.join("notes.md"), "").unwrap();
    }

    let o = run_foundry(&home, &["spec", "load", "amb", "20240315_120"]);
    assert!(!o.status.success());
    let msg = format!("{}{}", err_utf8(&o), utf8(&o));
    assert!(
        msg.contains("Ambiguous") || msg.contains("20240315"),
        "expected ambiguity listing; got: {msg}"
    );
}
