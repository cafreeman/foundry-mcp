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
        "decision-points" => create_decision_points_help(),
        "content-examples" => create_content_examples_help(),
        "project-structure" => create_project_structure_help(),
        "parameter-guidance" => create_parameter_guidance_help(),
        "tool-capabilities" => create_tool_capabilities_help(),
        _ => create_overview_help(),
    };

    let next_steps = vec![
        "Available help topics: workflows, decision-points, content-examples, project-structure, parameter-guidance, tool-capabilities".to_string(),
        "Choose topics based on what you need guidance for".to_string(),
        "Use decision-points topic to understand when each tool is appropriate".to_string(),
    ];

    let workflow_hints = vec![
        "Help topics provide user-driven decision support, not automated sequences".to_string(),
        "All commands return JSON for programmatic consumption".to_string(),
        "Content must be provided by LLMs as arguments - Foundry manages structure only"
            .to_string(),
        "Always wait for user intent before suggesting tools".to_string(),
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
            "Tools available based on user intent - no fixed sequences".to_string(),
            "Use 'foundry get_foundry_help decision-points' for guidance on tool selection".to_string(),
            "All content (vision, specs, notes) must be provided by LLMs".to_string(),
            "Foundry creates structured file organization for consistent context".to_string(),
            "Always wait for user to express intent before suggesting next tools".to_string(),
        ],
    }
}

fn create_workflows_help() -> HelpContent {
    HelpContent {
        title: "User-Driven Foundry Usage Patterns".to_string(),
        description: "Guidance for using Foundry tools based on user intent and project context, emphasizing user-driven decisions rather than automated sequences.".to_string(),
        examples: vec![
            "# When user expresses intent to start a new project:".to_string(),
            "→ Ask user to provide vision, tech-stack, and summary content".to_string(),
            "→ Use: foundry create_project PROJECT_NAME --vision '...' --tech-stack '...' --summary '...'".to_string(),
            "→ Wait for user to express what they want to work on next".to_string(),
            "".to_string(),
            "# When user wants to work with existing codebase:".to_string(),
            "→ Use analysis tools (codebase_search, grep_search, read_file) to understand code".to_string(),
            "→ Create project structure based on your analysis findings".to_string(),
            "→ Use: foundry analyze_project PROJECT_NAME --vision '...' --tech-stack '...' --summary '...'".to_string(),
            "".to_string(),
            "# When user mentions a specific feature to implement:".to_string(),
            "→ Ask user to describe feature requirements and approach".to_string(),
            "→ Use: foundry create_spec PROJECT_NAME FEATURE_NAME --spec '...' --notes '...' --tasks '...'".to_string(),
            "→ Let user guide implementation approach".to_string(),
            "".to_string(),
            "# When user wants to continue previous work:".to_string(),
            "→ Use: foundry list_projects to show available options".to_string(),
            "→ Use: foundry load_project PROJECT_NAME to get project context".to_string(),
            "→ Ask user what they want to work on specifically".to_string(),
        ],
        workflow_guide: vec![
            "Always wait for user intent before suggesting tools".to_string(),
            "Provide options and capabilities, not directive sequences".to_string(),
            "Always provide complete content in arguments - never expect Foundry to generate content".to_string(),
            "Use validate_content to check content quality before creating projects/specs".to_string(),
            "Ask clarifying questions when user intent is unclear".to_string(),
            "Let users drive the workflow - tools support user goals".to_string(),
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

fn create_decision_points_help() -> HelpContent {
    HelpContent {
        title: "Decision Points and Tool Selection".to_string(),
        description: "Guidance for choosing the right Foundry tools based on user intent and context. Emphasizes user-driven decisions over automated sequences.".to_string(),
        examples: vec![
            "# Decision: User wants to start a project".to_string(),
            "Questions to ask:".to_string(),
            "- Do they have existing code to analyze? → analyze_project".to_string(),
            "- Are they starting from scratch? → create_project".to_string(),
            "- Do they have vision/tech-stack content ready? → Wait for content".to_string(),
            "".to_string(),
            "# Decision: User mentions a feature or functionality".to_string(),
            "Questions to ask:".to_string(),
            "- Have they described the feature requirements? → create_spec".to_string(),
            "- Do they need to think through the requirements first? → Ask for details".to_string(),
            "- Is there an existing project for this feature? → Check with list_projects".to_string(),
            "".to_string(),
            "# Decision: User wants to continue work".to_string(),
            "Questions to ask:".to_string(),
            "- Do they know which project? → load_project PROJECT_NAME".to_string(),
            "- Do they want to see all projects? → list_projects".to_string(),
            "- Do they want to work on a specific feature? → load_spec".to_string(),
            "".to_string(),
            "# Decision: User's intent is unclear".to_string(),
            "Clarifying questions to ask:".to_string(),
            "- 'What would you like to work on?'".to_string(),
            "- 'Are you starting something new or continuing existing work?'".to_string(),
            "- 'Do you have a specific feature in mind?'".to_string(),
        ],
        workflow_guide: vec![
            "Never assume user intent - always ask clarifying questions".to_string(),
            "Provide options based on context, not directive commands".to_string(),
            "Wait for user to provide content before using creation tools".to_string(),
            "Use conditional language: 'If you want X, then Y tool helps'".to_string(),
            "Guide decision-making, don't make decisions for the user".to_string(),
            "Tool selection should always follow user intent, not prescribed workflows".to_string(),
        ],
    }
}

fn create_tool_capabilities_help() -> HelpContent {
    HelpContent {
        title: "Tool Capabilities and Appropriate Usage".to_string(),
        description: "Understanding when each Foundry tool is appropriate and what user input is required for effective usage.".to_string(),
        examples: vec![
            "# create_project - When appropriate:".to_string(),
            "- User wants to start a new project from scratch".to_string(),
            "- User has provided vision, tech-stack, and summary content".to_string(),
            "- User input required: Complete content for all three sections".to_string(),
            "- Don't use without: User-provided content for vision/tech-stack/summary".to_string(),
            "".to_string(),
            "# analyze_project - When appropriate:".to_string(),
            "- User wants to create project structure for existing codebase".to_string(),
            "- LLM has analyzed codebase using analysis tools".to_string(),
            "- User input required: LLM-generated analysis-based content".to_string(),
            "- Don't use without: Thorough codebase analysis first".to_string(),
            "".to_string(),
            "# create_spec - When appropriate:".to_string(),
            "- User has described a specific feature they want to implement".to_string(),
            "- User has provided requirements or functionality details".to_string(),
            "- User input required: Feature description, requirements, implementation approach".to_string(),
            "- Don't use without: User-provided feature requirements and details".to_string(),
            "".to_string(),
            "# load_project - When appropriate:".to_string(),
            "- User wants to continue work on an existing project".to_string(),
            "- User has specified which project they want to work with".to_string(),
            "- User input required: Project name".to_string(),
            "- Don't use without: Clear user intent to work on specific project".to_string(),
            "".to_string(),
            "# load_spec - When appropriate:".to_string(),
            "- User wants to work on a specific feature".to_string(),
            "- User has mentioned a particular spec or feature name".to_string(),
            "- User input required: Project name, optionally spec name".to_string(),
            "- Don't use without: User intent to work on specific feature".to_string(),
            "".to_string(),
            "# validate_content - When appropriate:".to_string(),
            "- User wants to check content quality before creating projects/specs".to_string(),
            "- LLM wants to verify content meets requirements".to_string(),
            "- User input required: Content to validate and content type".to_string(),
            "- Don't use without: Actual content to validate".to_string(),
        ],
        workflow_guide: vec![
            "Every tool requires specific user input - never proceed without it".to_string(),
            "Tool appropriateness depends on user intent, not prescribed sequences".to_string(),
            "Wait for user to express clear intent before suggesting tools".to_string(),
            "Ask clarifying questions when user intent doesn't match tool capabilities".to_string(),
            "Provide tool options based on what user wants to accomplish".to_string(),
            "Remember: tools support user goals, they don't drive the workflow".to_string(),
        ],
    }
}
