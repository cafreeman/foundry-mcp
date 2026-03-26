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

#[test]
fn status_emits_json_store_and_skills_shape() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);

    let o = run_foundry(&home, &["status"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    let v: Value = serde_json::from_str(utf8(&o).trim()).expect("valid json");
    assert!(v.get("store").is_some());
    assert_eq!(v.get("git_initialized"), Some(&Value::Bool(false)));
    assert!(v.get("skills").is_some());
}

#[test]
fn list_projects_returns_one_entry() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);
    assert!(
        run_foundry(&home, &["project", "init", "alpha"])
            .status
            .success()
    );

    let o = run_foundry(&home, &["list", "projects"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    let arr: Vec<Value> = serde_json::from_str(utf8(&o).trim()).unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0].get("name").and_then(|x| x.as_str()), Some("alpha"));
}

#[test]
fn spec_status_and_instructions_apply_emit_json() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);
    assert!(
        run_foundry(&home, &["project", "init", "wp"])
            .status
            .success()
    );
    let o = run_foundry(&home, &["spec", "init", "wp", "feat-a"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    let init: Value = serde_json::from_str(utf8(&o).trim()).unwrap();
    let id = init.get("id").and_then(|x| x.as_str()).unwrap().to_string();
    let spec_dir = init
        .get("path")
        .and_then(|x| x.as_str())
        .unwrap()
        .to_string();

    let spec_root = Path::new(&spec_dir);
    fs::write(spec_root.join("spec.md"), "# Feat A\n\nDo the thing.\n").unwrap();
    fs::write(
        spec_root.join("task-list.md"),
        "- [ ] step one\n- [ ] step two\n",
    )
    .unwrap();

    let o = run_foundry(&home, &["spec", "status", "wp", &id]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    let st: Value = serde_json::from_str(utf8(&o).trim()).unwrap();
    assert_eq!(
        st.get("state").and_then(|s| s.as_str()),
        Some("implementing")
    );

    let o = run_foundry(&home, &["spec", "instructions", "apply", "wp", &id]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    let app: Value = serde_json::from_str(utf8(&o).trim()).unwrap();
    assert_eq!(
        app.get("schema_name").and_then(|s| s.as_str()),
        Some("foundry-spec")
    );
    assert_eq!(
        app.get("state").and_then(|s| s.as_str()),
        Some("in_progress")
    );
}

#[test]
fn spec_collapse_prepare_finalize_moves_archive() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);
    assert!(
        run_foundry(&home, &["project", "init", "col"])
            .status
            .success()
    );
    let o = run_foundry(&home, &["spec", "init", "col", "done-feat"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    let init: Value = serde_json::from_str(utf8(&o).trim()).unwrap();
    let id = init.get("id").and_then(|x| x.as_str()).unwrap().to_string();
    let spec_dir = init
        .get("path")
        .and_then(|x| x.as_str())
        .unwrap()
        .to_string();

    fs::write(Path::new(&spec_dir).join("spec.md"), "# Done\n").unwrap();
    fs::write(
        Path::new(&spec_dir).join("task-list.md"),
        "- [x] only task\n",
    )
    .unwrap();

    let o = run_foundry(&home, &["spec", "collapse", "prepare", "col", &id]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    let summary = home
        .join(".foundry")
        .join("col")
        .join("completed")
        .join(&id)
        .join("summary.md");
    assert!(summary.exists());
    fs::write(&summary, "# Shipped\n\nWe shipped done-feat.\n").unwrap();

    let o = run_foundry(
        &home,
        &["spec", "collapse", "finalize", "col", &id, "--confirm"],
    );
    assert!(o.status.success(), "{}", err_utf8(&o));

    assert!(!Path::new(&spec_dir).exists());
    let archive = home
        .join(".foundry")
        .join("col")
        .join("completed")
        .join(&id)
        .join("archive");
    assert!(archive.join("spec.md").is_file());
    assert!(summary.is_file());

    let o = run_foundry(&home, &["list", "specs", "col"]);
    let specs: Vec<Value> = serde_json::from_str(utf8(&o).trim()).unwrap();
    assert!(
        !specs
            .iter()
            .any(|e| e.get("id").and_then(|x| x.as_str()) == Some(id.as_str())),
        "active list should not contain collapsed id"
    );

    let o = run_foundry(&home, &["list", "completed", "col"]);
    let done: Vec<Value> = serde_json::from_str(utf8(&o).trim()).unwrap();
    assert!(
        done.iter()
            .any(|e| e.get("id").and_then(|x| x.as_str()) == Some(id.as_str())),
        "completed list should include id"
    );
}

#[test]
fn spec_collapse_finalize_rejects_unexpected_subdirectories_with_name() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);
    assert!(
        run_foundry(&home, &["project", "init", "col"])
            .status
            .success()
    );
    let o = run_foundry(&home, &["spec", "init", "col", "done-feat"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    let init: Value = serde_json::from_str(utf8(&o).trim()).unwrap();
    let id = init.get("id").and_then(|x| x.as_str()).unwrap().to_string();
    let spec_dir = init
        .get("path")
        .and_then(|x| x.as_str())
        .unwrap()
        .to_string();

    fs::write(Path::new(&spec_dir).join("spec.md"), "# Done\n").unwrap();
    fs::write(
        Path::new(&spec_dir).join("task-list.md"),
        "- [x] only task\n",
    )
    .unwrap();

    let o = run_foundry(&home, &["spec", "collapse", "prepare", "col", &id]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    let summary = home
        .join(".foundry")
        .join("col")
        .join("completed")
        .join(&id)
        .join("summary.md");
    fs::write(&summary, "# Shipped\n\nWe shipped done-feat.\n").unwrap();

    let stray_dir = Path::new(&spec_dir).join("scratch");
    fs::create_dir_all(&stray_dir).unwrap();
    fs::write(stray_dir.join("extra.md"), "left behind\n").unwrap();

    let o = run_foundry(
        &home,
        &["spec", "collapse", "finalize", "col", &id, "--confirm"],
    );
    assert!(!o.status.success(), "finalize should reject stray dir");
    let err = err_utf8(&o);
    assert!(
        err.contains("Unexpected subdirector")
            || (err.contains("Unexpected subdirectories") && err.contains("scratch")),
        "stderr should name the unexpected directory: {err}"
    );
}

#[test]
fn project_arg_rejects_path_like_names() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);
    for (args, label) in [
        (&["list", "specs", ".."][..], "list specs"),
        (&["list", "completed", ".."][..], "list completed"),
        (&["project", "load", ".."][..], "project load"),
    ] {
        let o = run_foundry(&home, args);
        assert!(
            !o.status.success(),
            "{label} should reject ..: stdout={}",
            utf8(&o)
        );
        let err = err_utf8(&o).to_ascii_lowercase();
        assert!(
            err.contains("kebab") || err.contains("name cannot"),
            "{label}: {}",
            err_utf8(&o)
        );
    }
    let o = run_foundry(&home, &["list", "specs", "a/b"]);
    assert!(!o.status.success(), "slash segment: {}", utf8(&o));
    let err = err_utf8(&o).to_ascii_lowercase();
    assert!(
        err.contains("kebab") || err.contains("name cannot"),
        "{}",
        err_utf8(&o)
    );
}

#[test]
fn spec_instructions_verify_returns_json_when_all_tasks_done() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);
    assert!(
        run_foundry(&home, &["project", "init", "vp"])
            .status
            .success()
    );
    let o = run_foundry(&home, &["spec", "init", "vp", "verify-feat"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    let init: Value = serde_json::from_str(utf8(&o).trim()).unwrap();
    let id = init.get("id").and_then(|x| x.as_str()).unwrap().to_string();
    let spec_dir = init
        .get("path")
        .and_then(|x| x.as_str())
        .unwrap()
        .to_string();

    fs::write(Path::new(&spec_dir).join("spec.md"), "# Verify Feat\n\nDo the thing.\n").unwrap();
    fs::write(
        Path::new(&spec_dir).join("task-list.md"),
        "- [x] implement thing\n- [x] write tests\n",
    )
    .unwrap();

    let o = run_foundry(&home, &["spec", "instructions", "verify", "vp", &id]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    let v: Value = serde_json::from_str(utf8(&o).trim()).expect("valid json");

    assert_eq!(v.get("state").and_then(|s| s.as_str()), Some("ready_to_verify"));
    assert_eq!(v.get("project").and_then(|s| s.as_str()), Some("vp"));

    let tasks = v.get("tasks").and_then(|t| t.as_array()).unwrap();
    assert_eq!(tasks.len(), 2);
    assert_eq!(
        tasks[0].get("description").and_then(|s| s.as_str()),
        Some("implement thing")
    );
    assert_eq!(tasks[0].get("done").and_then(|b| b.as_bool()), Some(true));

    let cf = v.get("context_files").unwrap();
    assert!(cf.get("spec_md").and_then(|s| s.as_str()).is_some());
    assert!(cf.get("task_list_md").and_then(|s| s.as_str()).is_some());
    assert!(v.get("instruction").and_then(|s| s.as_str()).is_some());
}

#[test]
fn spec_instructions_verify_rejects_incomplete_spec() {
    let temp = TempDir::new().unwrap();
    let home = isolated_home(&temp);
    assert!(
        run_foundry(&home, &["project", "init", "vp2"])
            .status
            .success()
    );
    let o = run_foundry(&home, &["spec", "init", "vp2", "in-progress"]);
    assert!(o.status.success(), "{}", err_utf8(&o));
    let init: Value = serde_json::from_str(utf8(&o).trim()).unwrap();
    let id = init.get("id").and_then(|x| x.as_str()).unwrap().to_string();
    let spec_dir = init
        .get("path")
        .and_then(|x| x.as_str())
        .unwrap()
        .to_string();

    fs::write(Path::new(&spec_dir).join("spec.md"), "# In Progress\n").unwrap();
    fs::write(
        Path::new(&spec_dir).join("task-list.md"),
        "- [x] done task\n- [ ] incomplete task\n",
    )
    .unwrap();

    let o = run_foundry(&home, &["spec", "instructions", "verify", "vp2", &id]);
    assert!(!o.status.success(), "should reject spec with incomplete tasks");
    let err = err_utf8(&o);
    assert!(
        err.contains("not ready to verify") || err.contains("Complete all task-list"),
        "stderr should explain phase requirement: {err}"
    );
}
