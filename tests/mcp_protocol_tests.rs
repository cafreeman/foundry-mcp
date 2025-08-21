//! MCP Protocol tests for the Project Manager MCP Server

use foundry_mcp::handlers::ProjectManagerHandler;
use foundry_mcp::tools::{
    CreateSpecTool, LoadSpecTool, ProjectManagerTools, SetupProjectTool, UpdateSpecTool,
};
use rust_mcp_sdk::schema::CallToolRequestParams;
use serde_json::json;

#[tokio::test]
async fn test_mcp_tools_serialization() {
    // Test SetupProjectTool serialization using JSON
    let tool_json = json!({
        "project_name": "test-project",
        "tech_stack": {
            "languages": ["Rust", "JavaScript"],
            "frameworks": ["Actix", "React"],
            "databases": ["PostgreSQL"],
            "tools": ["Cargo", "npm"],
            "deployment": ["Docker"]
        },
        "vision": {
            "overview": "Test overview",
            "goals": ["Build fast API"],
            "target_users": ["Developers"],
            "success_criteria": ["< 100ms response time"]
        },
        "project_path": "/path/to/project"
    });

    let serialized = serde_json::to_string(&tool_json).expect("Failed to serialize tool JSON");
    let deserialized: SetupProjectTool =
        serde_json::from_str(&serialized).expect("Failed to deserialize SetupProjectTool");

    // Test that serialization/deserialization works - we can't access private fields directly
    let re_serialized = serde_json::to_value(&deserialized).expect("Failed to re-serialize");
    assert!(re_serialized.get("project_name").is_some());
    assert!(re_serialized.get("tech_stack").is_some());
    assert!(re_serialized.get("vision").is_some());
}

#[tokio::test]
async fn test_create_spec_tool_serialization() {
    let tool_json = json!({
        "project_name": "test-project",
        "spec_name": "test_feature",
        "description": "A test feature specification"
    });

    let serialized = serde_json::to_string(&tool_json).expect("Failed to serialize tool JSON");
    let deserialized: CreateSpecTool =
        serde_json::from_str(&serialized).expect("Failed to deserialize CreateSpecTool");

    let re_serialized = serde_json::to_value(&deserialized).expect("Failed to re-serialize");
    assert!(re_serialized.get("project_name").is_some());
    assert!(re_serialized.get("spec_name").is_some());
    assert!(re_serialized.get("description").is_some());
}

#[tokio::test]
async fn test_load_spec_tool_serialization() {
    let tool_json = json!({
        "project_name": "test-project",
        "spec_id": "20240101_test_feature"
    });

    let serialized = serde_json::to_string(&tool_json).expect("Failed to serialize tool JSON");
    let deserialized: LoadSpecTool =
        serde_json::from_str(&serialized).expect("Failed to deserialize LoadSpecTool");

    let re_serialized = serde_json::to_value(&deserialized).expect("Failed to re-serialize");
    assert!(re_serialized.get("project_name").is_some());
    assert!(re_serialized.get("spec_id").is_some());
}

#[tokio::test]
async fn test_update_spec_tool_serialization() {
    let tool_json = json!({
        "project_name": "test-project",
        "spec_id": "20240101_test_feature",
        "operation": "add_task",
        "task": {
            "id": "task_001",
            "title": "Test Task",
            "description": "A test task",
            "priority": "High",
            "status": "Todo",
            "dependencies": ["task_000"]
        }
    });

    let serialized = serde_json::to_string(&tool_json).expect("Failed to serialize tool JSON");
    let deserialized: UpdateSpecTool =
        serde_json::from_str(&serialized).expect("Failed to deserialize UpdateSpecTool");

    let re_serialized = serde_json::to_value(&deserialized).expect("Failed to re-serialize");
    assert!(re_serialized.get("operation").is_some());
    assert!(re_serialized.get("task").is_some());
    assert!(re_serialized.get("project_name").is_some());
    assert!(re_serialized.get("spec_id").is_some());
}

#[tokio::test]
async fn test_project_manager_tools_enum_conversion() {
    let tool_json = json!({
        "project_name": "test-project",
        "tech_stack": {
            "languages": ["Rust"],
            "frameworks": ["Actix"],
            "databases": ["PostgreSQL"],
            "tools": ["Cargo"],
            "deployment": ["Docker"]
        },
        "vision": {
            "overview": "Test overview",
            "goals": ["Build API"],
            "target_users": ["Developers"],
            "success_criteria": ["Fast response"]
        }
    });

    // Test conversion to enum
    let tool_params = CallToolRequestParams {
        name: "setup_project".to_string(),
        arguments: Some(tool_json.as_object().unwrap().clone()),
    };

    let tool_enum = ProjectManagerTools::try_from(tool_params);
    assert!(tool_enum.is_ok());

    match tool_enum.unwrap() {
        ProjectManagerTools::SetupProjectTool(_tool) => {
            // Successfully converted to SetupProjectTool variant
            assert!(true);
        }
        _ => panic!("Expected SetupProjectTool variant"),
    }
}

#[tokio::test]
async fn test_mcp_handler_creation() {
    let handler = ProjectManagerHandler::new();
    assert!(handler.is_ok(), "Failed to create ProjectManagerHandler");
}

#[tokio::test]
async fn test_tools_list() {
    // Test that the ProjectManagerTools enum has the expected tools
    let tools = ProjectManagerTools::tools();
    assert_eq!(tools.len(), 4);

    let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
    assert!(tool_names.contains(&"setup_project".to_string()));
    assert!(tool_names.contains(&"create_spec".to_string()));
    assert!(tool_names.contains(&"load_spec".to_string()));
    assert!(tool_names.contains(&"update_spec".to_string()));

    // Verify all tools have descriptions
    for tool in &tools {
        assert!(tool.description.is_some());
        assert!(!tool.description.as_ref().unwrap().is_empty());
    }
}

#[tokio::test]
async fn test_setup_project_tool_call() {
    let tool_json = json!({
        "project_name": "test-project",
        "tech_stack": {
            "languages": ["Rust"],
            "frameworks": ["Actix-Web"],
            "databases": ["PostgreSQL"],
            "tools": ["Cargo"],
            "deployment": ["Docker"]
        },
        "vision": {
            "overview": "Test project",
            "goals": ["Build API"],
            "target_users": ["Developers"],
            "success_criteria": ["Fast response"]
        }
    });

    let tool: SetupProjectTool =
        serde_json::from_value(tool_json).expect("Failed to deserialize SetupProjectTool");
    let result = tool.call_tool();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_create_spec_tool_call() {
    let tool_json = json!({
        "project_name": "test-project",
        "spec_name": "test_feature",
        "description": "A test feature"
    });

    let tool: CreateSpecTool =
        serde_json::from_value(tool_json).expect("Failed to deserialize CreateSpecTool");
    let result = tool.call_tool();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_load_spec_tool_call() {
    let tool_json = json!({
        "project_name": "test-project",
        "spec_id": "20240101_test_feature"
    });

    let tool: LoadSpecTool =
        serde_json::from_value(tool_json).expect("Failed to deserialize LoadSpecTool");
    let result = tool.call_tool();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_spec_tool_call() {
    let tool_json = json!({
        "project_name": "test-project",
        "spec_id": "20240101_test_feature",
        "operation": "add_task"
    });

    let tool: UpdateSpecTool =
        serde_json::from_value(tool_json).expect("Failed to deserialize UpdateSpecTool");
    let result = tool.call_tool();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_tool_parameter_validation() {
    // Test that invalid parameters fail to deserialize

    // Missing required field for SetupProjectTool
    let invalid_setup = json!({"project_name": "test"});
    let setup_result: Result<SetupProjectTool, _> = serde_json::from_value(invalid_setup);
    assert!(setup_result.is_err());

    // Missing required field for CreateSpecTool
    let invalid_create = json!({"project_name": "test"});
    let create_result: Result<CreateSpecTool, _> = serde_json::from_value(invalid_create);
    assert!(create_result.is_err());

    // Missing required field for LoadSpecTool
    let invalid_load = json!({"project_name": "test"});
    let load_result: Result<LoadSpecTool, _> = serde_json::from_value(invalid_load);
    assert!(load_result.is_err());

    // Missing required fields for UpdateSpecTool
    let invalid_update = json!({"project_name": "test"});
    let update_result: Result<UpdateSpecTool, _> = serde_json::from_value(invalid_update);
    assert!(update_result.is_err());
}

#[tokio::test]
async fn test_tool_schema_compliance() {
    // Test that our tools produce valid JSON schemas
    let tools = ProjectManagerTools::tools();

    assert_eq!(tools.len(), 4);

    for tool in &tools {
        // Verify required fields are present
        assert!(!tool.name.is_empty());
        assert!(tool.description.is_some());
        assert!(!tool.description.as_ref().unwrap().is_empty());

        // Verify input schema is valid JSON
        let schema_json = serde_json::to_string(&tool.input_schema);
        assert!(schema_json.is_ok());

        // Verify schema has required type field
        assert_eq!(tool.input_schema.type_(), "object");
        assert!(tool.input_schema.properties.is_some());
    }
}
