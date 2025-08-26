//! Implementation of the get_foundry_help command

use crate::cli::args::GetFoundryHelpArgs;
use crate::types::responses::{
    FoundryResponse, GetFoundryHelpResponse, HelpContent, ValidationStatus,
};
use anyhow::Result;

pub async fn execute(args: GetFoundryHelpArgs) -> Result<FoundryResponse<GetFoundryHelpResponse>> {
    let topic = args.topic.as_deref().unwrap_or("overview");

    let content = match topic {
        "workflows" => create_workflows_help(),
        "content-examples" => create_content_examples_help(),
        "project-structure" => create_project_structure_help(),
        "parameter-guidance" => create_parameter_guidance_help(),
        _ => create_overview_help(),
    };

    let next_steps = vec![
        "Use specific help topics: workflows, content-examples, project-structure, parameter-guidance".to_string(),
        "Start with 'foundry create_project' or 'foundry analyze_project' to begin".to_string(),
        "Load existing projects with 'foundry load_project <name>'".to_string(),
    ];

    let workflow_hints = vec![
        "Help topics provide LLM-optimized guidance for efficient Foundry usage".to_string(),
        "All commands return JSON for programmatic consumption".to_string(),
        "Content must be provided by LLMs as arguments - Foundry manages structure only"
            .to_string(),
    ];

    Ok(FoundryResponse {
        data: GetFoundryHelpResponse {
            topic: topic.to_string(),
            content,
        },
        next_steps,
        validation_status: ValidationStatus::Complete,
        workflow_hints,
    })
}

fn create_overview_help() -> HelpContent {
    HelpContent {
        title: "Foundry - Project Management for AI Coding Assistants".to_string(),
        description: "Foundry is a CLI tool that manages project structure in ~/.foundry/ to help LLMs maintain context about software projects through structured specifications. Foundry is content-agnostic - LLMs provide ALL content as arguments, Foundry manages file structure.".to_string(),
        examples: vec![
            "foundry create_project my-app --vision '...' --tech-stack '...' --summary '...'".to_string(),
            "foundry load_project my-app  # Load complete project context".to_string(),
            "foundry create_spec my-app user_auth --spec '...' --notes '...' --tasks '...'".to_string(),
            "foundry list_projects  # Discover available projects".to_string(),
        ],
        workflow_guide: vec![
            "Core LLM Workflow: create → list → load → create_spec → work".to_string(),
            "Use 'foundry get_foundry_help workflows' for detailed guidance".to_string(),
            "All content (vision, specs, notes) must be provided by LLMs".to_string(),
            "Foundry creates structured file organization for consistent context".to_string(),
        ],
    }
}

fn create_workflows_help() -> HelpContent {
    HelpContent {
        title: "Foundry Workflows for LLM Development".to_string(),
        description: "Step-by-step workflows for common LLM development scenarios using Foundry's structured project management.".to_string(),
        examples: vec![
            "# New Project Workflow:".to_string(),
            "1. foundry create_project PROJECT_NAME --vision '...' --tech-stack '...' --summary '...'".to_string(),
            "2. foundry create_spec PROJECT_NAME FEATURE_NAME --spec '...' --notes '...' --tasks '...'".to_string(),
            "3. Work on implementation using task-list.md as checklist".to_string(),
            "".to_string(),
            "# Existing Codebase Analysis:".to_string(),
            "1. Use codebase_search/grep_search/read_file to analyze existing code".to_string(),
            "2. foundry analyze_project PROJECT_NAME --vision '...' --tech-stack '...' --summary '...'".to_string(),
            "3. foundry create_spec PROJECT_NAME FEATURE --spec '...' --notes '...' --tasks '...'".to_string(),
            "".to_string(),
            "# Context Loading Workflow:".to_string(),
            "1. foundry list_projects  # Discover available projects".to_string(),
            "2. foundry load_project PROJECT_NAME  # Load full context".to_string(),
            "3. foundry load_spec PROJECT_NAME [SPEC_NAME]  # Load specific spec".to_string(),
        ],
        workflow_guide: vec![
            "Always provide complete content in arguments - never expect Foundry to generate content".to_string(),
            "Use validate_content to check content quality before creating projects/specs".to_string(),
            "Specs use timestamp directories (YYYYMMDD_HHMMSS_feature_name) for chronological organization".to_string(),
            "Project summary should be concise for quick context loading".to_string(),
            "Task-list.md serves as implementation checklist - update as work progresses".to_string(),
        ],
    }
}

fn create_content_examples_help() -> HelpContent {
    HelpContent {
        title: "Content Examples and Templates".to_string(),
        description: "Example content formats for vision, tech-stack, summary, specs, and notes to guide LLM content creation.".to_string(),
        examples: vec![
            "# Vision Content (2-4 paragraphs):".to_string(),
            "\"This project solves [PROBLEM] for [TARGET_USERS] by providing [UNIQUE_VALUE]. Unlike existing solutions, our approach [DIFFERENTIATOR]. Key features include [FEATURES]. The roadmap prioritizes [PRIORITIES].\"".to_string(),
            "".to_string(),
            "# Tech Stack Content (comprehensive):".to_string(),
            "\"Language: Rust for performance and safety. Framework: Tokio async runtime with clap for CLI. Storage: JSON files for simplicity. Deployment: Cross-platform binaries. Rationale: [WHY_THESE_CHOICES]. Constraints: [LIMITATIONS]. Team preferences: [STANDARDS].\"".to_string(),
            "".to_string(),
            "# Summary Content (concise):".to_string(),
            "\"[PROJECT_NAME]: [ONE_LINE_DESCRIPTION]. Tech: [KEY_TECHNOLOGIES]. Focus: [MAIN_PRIORITIES]. Users: [TARGET_AUDIENCE].\"".to_string(),
            "".to_string(),
            "# Spec Content (detailed requirements):".to_string(),
            "\"## Overview\\n[FEATURE_DESCRIPTION]\\n\\n## Requirements\\n- [REQ1]\\n- [REQ2]\\n\\n## Implementation\\n[APPROACH]\\n\\n## Testing\\n[TEST_STRATEGY]\"".to_string(),
            "".to_string(),
            "# Notes Content (context/decisions):".to_string(),
            "\"## Design Decisions\\n[RATIONALE]\\n\\n## Dependencies\\n[REQUIREMENTS]\\n\\n## Considerations\\n[TRADEOFFS]\"".to_string(),
            "".to_string(),
            "# Tasks Content (implementation checklist):".to_string(),
            "\"- [ ] [TASK1]\\n- [ ] [TASK2]\\n- [ ] [TASK3]\\n\\nUpdate this checklist as work progresses.\"".to_string(),
        ],
        workflow_guide: vec![
            "Content length requirements: Vision ≥200 chars, Tech-stack ≥150 chars, Summary ≥100 chars".to_string(),
            "Use markdown formatting for structured content (headers, lists, code blocks)".to_string(),
            "Vision should answer: What problem? Who for? Why unique? What features? What priorities?".to_string(),
            "Tech-stack should include: Languages, frameworks, databases, deployment, rationale".to_string(),
            "Specs should be implementation-ready with clear requirements and approach".to_string(),
            "Use 'foundry validate_content' to check content before project/spec creation".to_string(),
        ],
    }
}

fn create_project_structure_help() -> HelpContent {
    HelpContent {
        title: "Foundry Project Structure".to_string(),
        description: "Understanding the file organization and directory structure that Foundry creates and manages.".to_string(),
        examples: vec![
            "~/.foundry/PROJECT_NAME/".to_string(),
            "├── project/".to_string(),
            "│   ├── vision.md      # High-level product vision and roadmap".to_string(),
            "│   ├── tech-stack.md  # Technology choices and architecture decisions".to_string(),
            "│   └── summary.md     # Concise summary for quick context loading".to_string(),
            "└── specs/".to_string(),
            "    ├── 20250823_143052_user_auth/".to_string(),
            "    │   ├── spec.md        # Feature specification and requirements".to_string(),
            "    │   ├── task-list.md   # Implementation checklist (updated by agents)".to_string(),
            "    │   └── notes.md       # Additional context and design decisions".to_string(),
            "    └── 20250824_091234_api_endpoints/".to_string(),
            "        ├── spec.md".to_string(),
            "        ├── task-list.md".to_string(),
            "        └── notes.md".to_string(),
        ],
        workflow_guide: vec![
            "All files use markdown format for consistent rendering across tools".to_string(),
            "Spec directories use ISO timestamp format: YYYYMMDD_HHMMSS_feature_name".to_string(),
            "Feature names must use snake_case (e.g., user_auth, api_endpoints)".to_string(),
            "Project files provide context for LLM sessions".to_string(),
            "Spec files contain feature-specific implementation guidance".to_string(),
            "Task-list.md serves as living checklist - update as work progresses".to_string(),
            "Notes.md captures design decisions and context for future reference".to_string(),
        ],
    }
}

fn create_parameter_guidance_help() -> HelpContent {
    HelpContent {
        title: "Parameter Guidelines and Schemas".to_string(),
        description: "Detailed guidance on parameter requirements, formats, and validation rules for all Foundry commands.".to_string(),
        examples: vec![
            "# create_project parameters:".to_string(),
            "--vision: High-level product vision (≥200 chars, 2-4 paragraphs)".to_string(),
            "--tech-stack: Technology decisions with rationale (≥150 chars)".to_string(),
            "--summary: Concise vision+tech summary (≥100 chars)".to_string(),
            "".to_string(),
            "# create_spec parameters:".to_string(),
            "feature_name: snake_case identifier (e.g., user_auth)".to_string(),
            "--spec: Detailed requirements and implementation approach".to_string(),
            "--notes: Design decisions, dependencies, considerations".to_string(),
            "--tasks: Implementation checklist in markdown list format".to_string(),
            "".to_string(),
            "# validate_content parameters:".to_string(),
            "content_type: vision|tech-stack|summary|spec|notes".to_string(),
            "--content: Content to validate against type-specific rules".to_string(),
            "".to_string(),
            "# Common validation rules:".to_string(),
            "- Project names: kebab-case (my-awesome-project)".to_string(),
            "- Feature names: snake_case (user_authentication)".to_string(),
            "- Content: Non-empty, minimum length requirements".to_string(),
            "- Markdown: Valid markdown formatting encouraged".to_string(),
        ],
        workflow_guide: vec![
            "All content parameters are REQUIRED - Foundry never generates content".to_string(),
            "Use specific, descriptive project and feature names".to_string(),
            "Validate content before creation to catch issues early".to_string(),
            "Minimum content lengths ensure sufficient detail for LLM context".to_string(),
            "Rich parameter descriptions guide LLM content creation".to_string(),
            "Error messages provide actionable guidance for parameter fixes".to_string(),
        ],
    }
}
