//! # Declarative Macros for MCP Tool Generation
//!
//! ## Usage
//!
//! ```ignore
//! impl_mcp_tool! {
//!     name = "example_tool",
//!     description = "An example tool for documentation",
//!     struct ExampleArgs {
//!         name: String {
//!             description = "The name parameter"
//!         },
//!         content: String {
//!             description = "Content with minimum length validation",
//!             min_length = 100
//!         }
//!     }
//! }
//! ```

/// The macro generates:
/// - `impl McpToolDefinition for StructName`
/// - `tool_definition()` method returning `rust_mcp_sdk::schema::Tool`
/// - `from_mcp_params()` method with parameter conversion and validation
#[macro_export]
macro_rules! impl_mcp_tool {
    (
        name = $tool_name:expr,
        description = $tool_desc:expr,
        struct $struct_name:ident {
            $(
                $field:ident: $field_type:ty {
                    description = $field_desc:expr
                    $(, min_length = $min_len:expr)?
                }
            ),* $(,)?
        }
    ) => {
        impl $crate::mcp::traits::McpToolDefinition for $struct_name {
            fn tool_definition() -> rust_mcp_sdk::schema::Tool {
                let mut properties: std::collections::HashMap<String, serde_json::Map<String, serde_json::Value>> = std::collections::HashMap::new();

                $(
                    {
                        let mut property = serde_json::Map::new();
                        property.insert("type".to_string(), serde_json::json!("string"));
                        property.insert("description".to_string(), serde_json::json!($field_desc));
                        $(
                            property.insert("minLength".to_string(), serde_json::json!($min_len));
                        )?
                        properties.insert(stringify!($field).to_string(), property);
                    }
                )*

                // Determine required fields (non-Option types)
                let required_fields = vec![
                    $(stringify!($field).to_string()),*
                ];

                rust_mcp_sdk::schema::Tool {
                    name: $tool_name.to_string(),
                    description: Some($tool_desc.to_string()),
                    title: None,
                    input_schema: rust_mcp_sdk::schema::ToolInputSchema::new(
                        required_fields,
                        Some(properties),
                    ),
                    annotations: None,
                    meta: None,
                    output_schema: None,
                }
            }

            fn from_mcp_params(params: &serde_json::Value) -> anyhow::Result<Self> {
                Ok(Self {
                    $(
                        $field: params[stringify!($field)]
                            .as_str()
                            .ok_or_else(|| anyhow::anyhow!("Missing {} parameter", stringify!($field)))?
                            .to_string()
                    ),*
                })
            }
        }
    };
}

// The macros are automatically available where this module is used

#[cfg(test)]
mod tests {
    use crate::mcp::traits::McpToolDefinition;

    // Test struct that mimics CreateProjectArgs structure
    #[derive(Debug)]
    pub struct TestCreateProjectArgs {
        pub project_name: String,
        pub vision: String,
        pub tech_stack: String,
        pub summary: String,
    }

    // Generate MCP tool implementation using our macro
    impl_mcp_tool! {
        name = "create_project",
        description = "Create new project structure with LLM-provided content. Creates ~/.foundry/PROJECT_NAME/ with vision.md, tech-stack.md, and summary.md",
        struct TestCreateProjectArgs {
            project_name: String {
                description = "Descriptive project name using kebab-case (e.g., 'my-awesome-app')"
            },
            vision: String {
                description = "High-level product vision (2-4 paragraphs, 200+ chars) covering: problem being solved, target users, unique value proposition, and key roadmap priorities. Use markdown with ## headers, bullet points, and clear structure. Include specific examples. Goes into vision.md",
                min_length = 200
            },
            tech_stack: String {
                description = "Comprehensive technology decisions (150+ chars) including languages, frameworks, databases, deployment platforms, and rationale. Use markdown with ## headers for categories, bullet points for technologies, and brief explanations. Include constraints, preferences, or team standards. Goes into tech-stack.md",
                min_length = 150
            },
            summary: String {
                description = "Concise summary (100+ chars) of vision and tech-stack for quick context loading. Should capture essential project essence in 2-3 sentences using clear, professional language. Combine main value proposition with primary technology. Goes into summary.md",
                min_length = 100
            }
        }
    }

    #[test]
    fn test_tool_definition_generation() {
        let tool = TestCreateProjectArgs::tool_definition();

        // Verify basic tool properties
        assert_eq!(tool.name, "create_project");
        assert!(tool.description.is_some());
        assert_eq!(
            tool.description.unwrap(),
            "Create new project structure with LLM-provided content. Creates ~/.foundry/PROJECT_NAME/ with vision.md, tech-stack.md, and summary.md"
        );

        // Verify required fields
        let required_fields = &tool.input_schema.required;
        assert_eq!(required_fields.len(), 4);
        assert!(required_fields.contains(&"project_name".to_string()));
        assert!(required_fields.contains(&"vision".to_string()));
        assert!(required_fields.contains(&"tech_stack".to_string()));
        assert!(required_fields.contains(&"summary".to_string()));

        // Verify properties exist
        let properties = tool.input_schema.properties.as_ref().unwrap();
        assert_eq!(properties.len(), 4);

        // Check specific property details
        let vision_prop = properties.get("vision").unwrap();
        assert_eq!(
            vision_prop.get("type").unwrap(),
            &serde_json::json!("string")
        );
        assert_eq!(
            vision_prop.get("minLength").unwrap(),
            &serde_json::json!(200)
        );

        let tech_stack_prop = properties.get("tech_stack").unwrap();
        assert_eq!(
            tech_stack_prop.get("minLength").unwrap(),
            &serde_json::json!(150)
        );

        let summary_prop = properties.get("summary").unwrap();
        assert_eq!(
            summary_prop.get("minLength").unwrap(),
            &serde_json::json!(100)
        );

        let project_name_prop = properties.get("project_name").unwrap();
        assert!(!project_name_prop.contains_key("minLength"));
    }

    #[test]
    fn test_parameter_conversion() {
        let params = serde_json::json!({
            "project_name": "test-project",
            "vision": "Test vision content that meets the minimum length requirement for this field which needs to be at least 200 characters long. This is a test vision that describes the problem, target users, value proposition and roadmap priorities.",
            "tech_stack": "Test tech stack content that meets minimum requirements. This describes languages, frameworks, databases, deployment platforms with rationale for each choice.",
            "summary": "Test summary that meets minimum length requirements and captures project essence in concise format."
        });

        let result = TestCreateProjectArgs::from_mcp_params(&params).unwrap();
        assert_eq!(result.project_name, "test-project");
        assert!(result.vision.starts_with("Test vision content"));
        assert!(result.tech_stack.starts_with("Test tech stack"));
        assert!(result.summary.starts_with("Test summary"));
    }

    #[test]
    fn test_missing_required_parameter() {
        let params = serde_json::json!({
            "project_name": "test-project",
            "vision": "Test vision",
            // Missing tech_stack and summary
        });

        let result = TestCreateProjectArgs::from_mcp_params(&params);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Missing") && error_msg.contains("parameter"));
    }
}
