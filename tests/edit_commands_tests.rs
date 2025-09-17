//! Tests for update_spec with operation "edit_commands"

use foundry_mcp::cli::args::UpdateSpecArgs;
use foundry_mcp::cli::commands::{create_project, create_spec, load_spec, update_spec};
use foundry_mcp::test_utils::TestEnvironment;

fn commands_json(cmds: serde_json::Value) -> String {
    serde_json::to_string(&cmds).unwrap()
}

#[test]
fn test_set_task_status_done() {
    let env = TestEnvironment::new().unwrap();
    env.with_env_async(|| async {
        // Setup project and spec
        let project_args = env.create_project_args("ec-project");
        create_project::execute(project_args).await.unwrap();
        let spec_resp = create_spec::execute(env.create_spec_args("ec-project", "feature"))
            .await
            .unwrap();
        let spec_name = spec_resp.data.spec_name;

        // Seed a known task list
        let foundry_dir = env.foundry_dir();
        let task_file = foundry_dir
            .join("ec-project")
            .join("specs")
            .join(&spec_name)
            .join("task-list.md");
        std::fs::write(
            &task_file,
            "## Tasks\n- [ ] Implement OAuth2 integration\n- [ ] Add password validation\n",
        )
        .unwrap();

        // Build edit_commands payload: mark task done
        let cmds = serde_json::json!([
            {
                "target": "tasks",
                "command": "set_task_status",
                "selector": {"type": "task_text", "value": "Implement OAuth2 integration"},
                "status": "done"
            }
        ]);

        let args = UpdateSpecArgs {
            project_name: "ec-project".to_string(),
            spec_name: spec_name.clone(),
            commands: commands_json(cmds),
        };

        let resp = update_spec::execute(args).await.unwrap();
        assert_eq!(
            resp.data.applied_count + resp.data.skipped_idempotent_count,
            1
        );

        // Verify file content updated
        let updated = std::fs::read_to_string(&task_file).unwrap();
        assert!(updated.contains("- [x] Implement OAuth2 integration"));
    });
}

#[test]
fn test_upsert_task_idempotent() {
    let env = TestEnvironment::new().unwrap();
    env.with_env_async(|| async {
        // Setup
        let project_args = env.create_project_args("ec-upsert");
        create_project::execute(project_args).await.unwrap();
        let spec_resp = create_spec::execute(env.create_spec_args("ec-upsert", "feature"))
            .await
            .unwrap();
        let spec_name = spec_resp.data.spec_name;

        // Upsert a task twice
        let cmd = serde_json::json!([
            {
                "target": "tasks",
                "command": "upsert_task",
                "selector": {"type": "task_text", "value": "Add password validation"},
                "content": "- [ ] Add password validation"
            }
        ]);

        for _ in 0..2 {
            let args = UpdateSpecArgs {
                project_name: "ec-upsert".to_string(),
                spec_name: spec_name.clone(),
                commands: commands_json(cmd.clone()),
            };
            let _ = update_spec::execute(args).await.unwrap();
        }

        // Verify only one instance exists
        let foundry_dir = env.foundry_dir();
        let task_file = foundry_dir
            .join("ec-upsert")
            .join("specs")
            .join(&spec_name)
            .join("task-list.md");
        let content = std::fs::read_to_string(task_file).unwrap();
        let count = content.matches("Add password validation").count();
        assert_eq!(count, 1);
    });
}

#[test]
fn test_append_to_spec_section() {
    let env = TestEnvironment::new().unwrap();
    env.with_env_async(|| async {
        // Setup
        let project_args = env.create_project_args("ec-append");
        create_project::execute(project_args).await.unwrap();
        let spec_resp = create_spec::execute(env.create_spec_args("ec-append", "feature"))
            .await
            .unwrap();
        let spec_name = spec_resp.data.spec_name;

        // Seed spec section
        let spec_file = env
            .foundry_dir()
            .join("ec-append")
            .join("specs")
            .join(&spec_name)
            .join("spec.md");
        std::fs::write(&spec_file, "# Feature\n\n## Requirements\n- Item A\n").unwrap();

        // Append to Requirements
        let cmds = serde_json::json!([
            {
                "target": "spec",
                "command": "append_to_section",
                "selector": {"type": "section", "value": "## Requirements"},
                "content": "- Item B"
            }
        ]);
        let args = UpdateSpecArgs {
            project_name: "ec-append".to_string(),
            spec_name: spec_name.clone(),
            operation: "edit_commands".to_string(),
            commands: Some(commands_json(cmds)),
        };
        let _ = update_spec::execute(args).await.unwrap();

        // Verify append and idempotence
        let updated = std::fs::read_to_string(&spec_file).unwrap();
        assert!(updated.contains("- Item B"));

        // Re-run same command; should not duplicate
        let cmds2 = serde_json::json!([
            {
                "target": "spec",
                "command": "append_to_section",
                "selector": {"type": "section", "value": "## Requirements"},
                "content": "- Item B"
            }
        ]);
        let args2 = UpdateSpecArgs {
            project_name: "ec-append".to_string(),
            spec_name: spec_name.clone(),
            operation: "edit_commands".to_string(),
            commands: Some(commands_json(cmds2)),
        };
        let _ = update_spec::execute(args2).await.unwrap();
        let updated2 = std::fs::read_to_string(&spec_file).unwrap();
        assert_eq!(updated2.matches("- Item B").count(), 1);
    });
}
