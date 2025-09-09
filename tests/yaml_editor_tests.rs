use assert_fs::prelude::*;
use assert_fs::TempDir;
use foundry_mcp::yaml_editor::{YamlEditor, YamlEditorError};
use serde_yml::Value;

fn create_test_spec() -> String {
    r#"
name: "user_auth"
status: "in_progress"
priority: 3
tasks:
  - description: "Set up OAuth2"
    completed: false
  - description: "Create user model"
    completed: true
notes: []
created_at: "2025-01-01T00:00:00Z"
last_modified: "2025-01-01T00:00:00Z"
"#
    .to_string()
}

#[test]
fn test_update_simple_field() {
    let tmp = TempDir::new().unwrap();
    let file = tmp.child("spec.yml");
    file.write_str(&create_test_spec()).unwrap();

    let mut editor = YamlEditor::load(file.path()).unwrap();

    // Update status
    editor
        .set_string("status", "ready_for_review")
        .expect("set status");
    editor.save().unwrap();

    let editor2 = YamlEditor::load(file.path()).unwrap();
    let status = editor2.get_value("status").unwrap();
    assert_eq!(status.as_str(), Some("ready_for_review"));

    // Update priority (increment)
    let mut editor3 = YamlEditor::load(file.path()).unwrap();
    editor3.increment_number("priority").unwrap();
    editor3.save().unwrap();
    let editor4 = YamlEditor::load(file.path()).unwrap();
    let priority = editor4.get_value("priority").unwrap();
    assert_eq!(priority.as_i64(), Some(4));

    // Type validation: navigating through wrong type should error
    let mut editor5 = YamlEditor::load(file.path()).unwrap();
    let err = editor5.get_value("tasks.completed").unwrap_err();
    match err {
        YamlEditorError::TypeMismatch { .. } => {}
        other => panic!("expected TypeMismatch, got {other:?}"),
    }
}

#[test]
fn test_array_operations() {
    let tmp = TempDir::new().unwrap();
    let file = tmp.child("spec.yml");
    file.write_str(&create_test_spec()).unwrap();

    let mut editor = YamlEditor::load(file.path()).unwrap();

    // Append simple string task
    editor
        .append_to_array("tasks", Value::String("Write unit tests".into()))
        .unwrap();
    editor.save().unwrap();

    // Update task status
    let mut editor2 = YamlEditor::load(file.path()).unwrap();
    editor2.set_bool("tasks.0.completed", true).unwrap();
    editor2.save().unwrap();
    let editor3 = YamlEditor::load(file.path()).unwrap();
    let completed = editor3.get_value("tasks.0.completed").unwrap();
    assert_eq!(completed.as_bool(), Some(true));

    // Remove a task
    let mut editor4 = YamlEditor::load(file.path()).unwrap();
    editor4.remove_path("tasks.1").unwrap();
    editor4.save().unwrap();
    let editor5 = YamlEditor::load(file.path()).unwrap();
    let tasks = editor5.get_value("tasks").unwrap();
    let seq = tasks.as_sequence().unwrap();
    assert_eq!(seq.len(), 2); // originally 2, removed index 1, added one -> 2
}

#[test]
fn test_nested_path_navigation() {
    let tmp = TempDir::new().unwrap();
    let file = tmp.child("spec.yml");
    file.write_str(&create_test_spec()).unwrap();

    let editor = YamlEditor::load(file.path()).unwrap();
    let val = editor.get_value("tasks.0.completed").unwrap();
    assert_eq!(val.as_bool(), Some(false));
    let val2 = editor.get_value("tasks.1.description").unwrap();
    assert_eq!(val2.as_str(), Some("Create user model"));

    let mut editor2 = YamlEditor::load(file.path()).unwrap();
    let err = editor2.get_value("tasks.10.description").unwrap_err();
    match err {
        YamlEditorError::PathNotFound(_) => {}
        other => panic!("expected PathNotFound, got {other:?}"),
    }
}

#[test]
fn test_atomic_operations() {
    let tmp = TempDir::new().unwrap();
    let file = tmp.child("spec.yml");
    let original = create_test_spec();
    file.write_str(&original).unwrap();

    // Attempt an invalid operation that should not corrupt the file
    let mut editor = YamlEditor::load(file.path()).unwrap();
    let res = editor.update_path("tasks.invalid.completed", Value::Bool(true));
    assert!(res.is_err());

    // Ensure file on disk is unchanged (since we never saved)
    let on_disk = std::fs::read_to_string(file.path()).unwrap();
    assert_eq!(on_disk, original);
}

