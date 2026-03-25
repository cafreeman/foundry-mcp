mod common;

use assert_fs::TempDir;
use common::{err_utf8, run_foundry, utf8};
use serde_json::Value;
use std::fs;
use std::path::Path;

fn isolated_home(temp: &TempDir) -> std::path::PathBuf {
    let home = temp.path().join("home");
    fs::create_dir_all(&home).unwrap();
    home
}

fn json_project_init_path(stdout: &str) -> String {
    let v: Value = serde_json::from_str(stdout.trim()).expect("project init JSON");
    v.get("path")
        .and_then(|x| x.as_str())
        .expect("path field")
        .to_string()
}

fn json_spec_init(stdout: &str) -> (String, String) {
    let v: Value = serde_json::from_str(stdout.trim()).expect("spec init JSON");
    let id = v
        .get("id")
        .and_then(|x| x.as_str())
        .expect("id field")
        .to_string();
    let path = v
        .get("path")
        .and_then(|x| x.as_str())
        .expect("path field")
        .to_string();
    (id, path)
}

#[test]
fn project_init_load_list_delete() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);

    let o = run_foundry(&home, &["project", "init", "my-project"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    let created = json_project_init_path(&utf8(&o));
    assert!(Path::new(&created).join("vision.md").exists());

    let o = run_foundry(&home, &["project", "load", "my-project"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    let text = utf8(&o);
    assert!(text.contains("=== vision.md ==="));
    assert!(text.contains("(empty)"));

    let o = run_foundry(&home, &["list", "projects"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    let arr: Vec<Value> = serde_json::from_str(utf8(&o).trim()).unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(
        arr[0].get("name").and_then(|x| x.as_str()),
        Some("my-project")
    );

    let o = run_foundry(&home, &["project", "delete", "my-project"]);
    assert!(!o.status.success());

    let o = run_foundry(&home, &["project", "delete", "my-project", "--confirm"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
}

#[test]
fn list_projects_empty_store() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);

    let o = run_foundry(&home, &["list", "projects"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    let arr: Vec<Value> = serde_json::from_str(utf8(&o).trim()).unwrap();
    assert!(arr.is_empty());
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
    let (id, spec_path) = json_spec_init(&utf8(&o));
    assert!(spec_path.contains("auth-flow"));
    assert!(Path::new(&spec_path).join("spec.md").exists());

    let o = run_foundry(&home, &["list", "specs", "p1"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    let entries: Vec<Value> = serde_json::from_str(utf8(&o).trim()).unwrap();
    let found = entries
        .iter()
        .find(|e| e.get("id").and_then(|x| x.as_str()) == Some(id.as_str()));
    assert!(found.is_some(), "list specs should include new spec id");

    let o = run_foundry(&home, &["spec", "load", "p1", "auth-flow"]);
    assert!(o.status.success(), "{}", err_utf8(&o));

    let o = run_foundry(&home, &["spec", "load", "p1", &id]);
    assert!(o.status.success(), "{}", err_utf8(&o));

    let o = run_foundry(&home, &["spec", "delete", "p1", "auth-flow"]);
    assert!(!o.status.success());

    let o = run_foundry(&home, &["spec", "delete", "p1", "auth-flow", "--confirm"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
}

#[test]
fn list_projects_sorts_names() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);

    for name in ["gamma", "alpha", "beta"] {
        assert!(
            run_foundry(&home, &["project", "init", name])
                .status
                .success()
        );
    }

    let o = run_foundry(&home, &["list", "projects"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    let arr: Vec<Value> = serde_json::from_str(utf8(&o).trim()).unwrap();
    let names: Vec<&str> = arr
        .iter()
        .map(|e| e.get("name").and_then(|x| x.as_str()).unwrap())
        .collect();
    assert_eq!(names, vec!["alpha", "beta", "gamma"]);
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
fn list_specs_sorted_by_timestamp_ascending() {
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

    let o = run_foundry(&home, &["list", "specs", "chrono"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    let entries: Vec<Value> = serde_json::from_str(utf8(&o).trim()).unwrap();
    assert_eq!(entries.len(), 2);
    let ids: Vec<&str> = entries
        .iter()
        .map(|e| e.get("id").and_then(|x| x.as_str()).unwrap())
        .collect();
    assert!(ids[0].contains("20240115"));
    assert!(ids[1].contains("20240615"));
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

#[test]
fn top_level_help_describes_skill_commands() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);

    let o = run_foundry(&home, &["--help"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    let help = utf8(&o);
    assert!(help.contains("install"));
    assert!(help.contains("update"));
    assert!(help.contains("uninstall"));
    assert!(
        help.contains("Install bundled skills")
            && help.contains("Update bundled skills")
            && help.contains("Remove bundled skills"),
        "expected help text for skill commands: {help}"
    );
}
