use crate::core::{project, spec};
use crate::yaml_editor::YamlEditor;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct UpdateFieldArgs {
    pub project: String,
    pub spec: Option<String>,
    pub field_path: String,
    pub value: serde_yml::Value,
}

#[derive(Deserialize, Serialize)]
pub struct TaskWithPriority {
    pub description: String,
    pub completed: bool,
    pub priority: u8,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct AddTaskArgs {
    pub project: String,
    pub spec: String,
    pub task_description: String,
    pub priority: Option<u8>,
}

#[derive(Deserialize)]
pub struct UpdateTaskStatusArgs {
    pub project: String,
    pub spec: String,
    pub task_index: usize,
    pub completed: bool,
}

pub fn update_project_field(args: UpdateFieldArgs) -> Result<String, Box<dyn std::error::Error>> {
    let file_path = if let Some(spec_name) = &args.spec {
        spec::get_spec_path(&args.project, spec_name)?.join("spec.yml")
    } else {
        project::get_project_path(&args.project)?.join("project.yml")
    };

    let mut editor = YamlEditor::load(&file_path)?;
    editor.update_path(&args.field_path, args.value)?;
    editor.save()?;

    Ok(format!(
        "Updated {}: {}",
        args.spec.unwrap_or("project".to_string()),
        args.field_path
    ))
}

pub fn add_task_to_spec(args: AddTaskArgs) -> Result<String, Box<dyn std::error::Error>> {
    let spec_path = spec::get_spec_path(&args.project, &args.spec)?.join("spec.yml");
    let mut editor = YamlEditor::load(&spec_path)?;

    let task_value = if let Some(priority) = args.priority {
        serde_yml::to_value(&TaskWithPriority {
            description: args.task_description.clone(),
            completed: false,
            priority,
            created_at: chrono::Utc::now().to_rfc3339(),
        })?
    } else {
        serde_yml::Value::String(args.task_description.clone())
    };

    editor.append_to_array("tasks", task_value)?;
    editor.set_string("last_modified", &chrono::Utc::now().to_rfc3339())?;
    editor.save()?;

    Ok(format!("Added task to {}: {}", args.spec, args.task_description))
}

pub fn update_task_status(
    args: UpdateTaskStatusArgs,
) -> Result<String, Box<dyn std::error::Error>> {
    let spec_path = spec::get_spec_path(&args.project, &args.spec)?.join("spec.yml");
    let mut editor = YamlEditor::load(&spec_path)?;

    let task_path = format!("tasks.{}.completed", args.task_index);
    editor.set_bool(&task_path, args.completed)?;
    editor.set_string("last_modified", &chrono::Utc::now().to_rfc3339())?;
    editor.save()?;

    Ok(format!(
        "Marked task {} as {}",
        args.task_index,
        if args.completed { "completed" } else { "incomplete" }
    ))
}

