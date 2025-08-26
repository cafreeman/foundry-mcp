//! # Foundry MCP Macros
//!
//! Procedural macros for automatically generating MCP tools from CLI argument structs.
//! This eliminates the need for manual tool definitions and ensures compile-time
//! compatibility between CLI and MCP interfaces.

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields, Meta, parse_macro_input};

/// Derive macro that generates MCP tool definitions from CLI argument structs
///
/// # Usage
///
/// ```rust
/// #[derive(Parser, Debug, McpTool)]
/// #[mcp(
///     name = "create_project",
///     description = "Create new project structure with LLM-provided content"
/// )]
/// pub struct CreateProjectArgs {
///     #[mcp(description = "Descriptive project name using kebab-case")]
///     pub project_name: String,
///
///     #[mcp(
///         description = "High-level product vision covering problem, users, value prop",
///         min_length = 200
///     )]
///     pub vision: String,
/// }
/// ```
///
/// This generates:
/// - `impl McpToolDefinition for CreateProjectArgs`
/// - `tool_definition()` method returning `McpTool`
/// - Enhanced `from_mcp_params()` method with better error handling
#[proc_macro_derive(McpTool, attributes(mcp))]
pub fn derive_mcp_tool(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match generate_mcp_tool_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(error) => syn::Error::new(Span::call_site(), error)
            .to_compile_error()
            .into(),
    }
}

/// Generate the implementation for McpToolDefinition trait
fn generate_mcp_tool_impl(input: &DeriveInput) -> Result<proc_macro2::TokenStream, String> {
    let struct_name = &input.ident;

    // Parse struct-level attributes
    let (tool_name, tool_description) = parse_struct_attributes(&input.attrs)?;

    // Parse struct fields
    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => &fields_named.named,
            _ => {
                return Err("McpTool can only be derived for structs with named fields".to_string());
            }
        },
        _ => return Err("McpTool can only be derived for structs".to_string()),
    };

    // Generate field information
    let mut required_fields = Vec::new();
    let mut properties = Vec::new();
    let mut from_params_fields = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();

        let field_info = parse_field_attributes(&field.attrs)?;
        let is_optional = is_option_type(&field.ty);

        if !is_optional {
            required_fields.push(field_name_str.clone());
        }

        // Generate property definition
        let description = field_info
            .description
            .unwrap_or_else(|| format!("Parameter: {}", field_name_str));

        let property_gen = if let Some(min_length) = field_info.min_length {
            quote! {
                {
                    let mut property = serde_json::Map::new();
                    property.insert("type".to_string(), serde_json::json!("string"));
                    property.insert("description".to_string(), serde_json::json!(#description));
                    property.insert("minLength".to_string(), serde_json::json!(#min_length));
                    properties.insert(#field_name_str.to_string(), property);
                }
            }
        } else {
            quote! {
                {
                    let mut property = serde_json::Map::new();
                    property.insert("type".to_string(), serde_json::json!("string"));
                    property.insert("description".to_string(), serde_json::json!(#description));
                    properties.insert(#field_name_str.to_string(), property);
                }
            }
        };

        properties.push(property_gen);

        // Generate from_mcp_params field extraction
        if is_optional {
            from_params_fields.push(quote! {
                #field_name: params[#field_name_str].as_str().map(|s| s.to_string())
            });
        } else {
            from_params_fields.push(quote! {
                #field_name: params[#field_name_str]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Missing {} parameter", #field_name_str))?
                    .to_string()
            });
        }
    }

    // Generate the implementation
    let tool_def = quote! {
        impl crate::mcp::traits::McpToolDefinition for #struct_name {
            fn tool_definition() -> rust_mcp_sdk::schema::Tool {
                let mut properties: std::collections::HashMap<String, serde_json::Map<String, serde_json::Value>> = std::collections::HashMap::new();
                #(#properties)*

                rust_mcp_sdk::schema::Tool {
                    name: #tool_name.to_string(),
                    description: Some(#tool_description.to_string()),
                    title: None,
                    input_schema: rust_mcp_sdk::schema::ToolInputSchema::new(
                        vec![#(#required_fields.to_string()),*],
                        Some(properties),
                    ),
                    annotations: None,
                    meta: None,
                    output_schema: None,
                }
            }

            fn from_mcp_params(params: &serde_json::Value) -> anyhow::Result<Self> {
                Ok(Self {
                    #(#from_params_fields),*
                })
            }
        }
    };

    Ok(tool_def)
}

/// Parse struct-level MCP attributes
fn parse_struct_attributes(attrs: &[Attribute]) -> Result<(String, String), String> {
    let mut name = None;
    let mut description = None;

    for attr in attrs {
        if attr.path().is_ident("mcp") {
            match &attr.meta {
                Meta::List(meta_list) => {
                    // Parse the entire token stream as a string and extract key-value pairs
                    let tokens_str = meta_list.tokens.to_string();

                    // Extract name = "value" pairs
                    if let Some(name_match) = extract_string_value(&tokens_str, "name") {
                        name = Some(name_match);
                    }
                    if let Some(desc_match) = extract_string_value(&tokens_str, "description") {
                        description = Some(desc_match);
                    }
                }
                _ => return Err("Expected #[mcp(...)] attribute format".to_string()),
            }
        }
    }

    let name = name.ok_or("Missing 'name' attribute in #[mcp(...)]")?;
    let description = description.ok_or("Missing 'description' attribute in #[mcp(...)]")?;

    Ok((name, description))
}

/// Extract a string value from "key = \"value\"" pattern
fn extract_string_value(input: &str, key: &str) -> Option<String> {
    let pattern = format!("{} =", key);
    if let Some(start_pos) = input.find(&pattern) {
        let after_equals = &input[start_pos + pattern.len()..];

        // Find the first quote
        if let Some(first_quote) = after_equals.find('"') {
            let start_content = first_quote + 1;

            // Find the matching closing quote, being careful about escaped quotes
            let content_start = &after_equals[start_content..];
            if let Some(end_quote) = content_start.find('"') {
                return Some(content_start[..end_quote].to_string());
            }
        }
    }
    None
}

/// Information extracted from field attributes
#[derive(Default)]
struct FieldInfo {
    description: Option<String>,
    min_length: Option<u32>,
}

/// Parse field-level MCP attributes
fn parse_field_attributes(attrs: &[Attribute]) -> Result<FieldInfo, String> {
    let mut info = FieldInfo::default();

    for attr in attrs {
        if attr.path().is_ident("mcp")
            && let Meta::List(meta_list) = &attr.meta {
                let tokens_str = meta_list.tokens.to_string();

                // Extract description using improved helper
                if let Some(description) = extract_string_value(&tokens_str, "description") {
                    info.description = Some(description);
                }

                // Extract min_length
                if let Some(min_length) = extract_numeric_value(&tokens_str, "min_length") {
                    info.min_length = Some(min_length);
                }
            }
    }

    Ok(info)
}

/// Extract a numeric value from "key = value" pattern
fn extract_numeric_value(input: &str, key: &str) -> Option<u32> {
    let pattern = format!("{} =", key);
    if let Some(start_pos) = input.find(&pattern) {
        let after_equals = &input[start_pos + pattern.len()..].trim_start();

        // Extract consecutive digits
        let digits: String = after_equals
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();

        if !digits.is_empty() {
            return digits.parse().ok();
        }
    }
    None
}

/// Check if a type is Option<T>
fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    false
}

// Note: The McpToolDefinition trait is defined in the main crate
// to avoid circular dependencies with external crates
