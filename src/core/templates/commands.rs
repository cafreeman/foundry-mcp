//! Installable command templates for Claude and Cursor

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// We place commands directly under the client's commands directory
/// Common set of command files we install for Claude (frontmatter) (filename -> content)
fn claude_command_files() -> Vec<(&'static str, &'static str)> {
    vec![
        ("foundry_analyze_project.md", CLAUDE_ANALYZE_PROJECT_CMD),
        ("foundry_create_project.md", CLAUDE_CREATE_PROJECT_CMD),
        ("foundry_list_specs.md", CLAUDE_LIST_SPECS_CMD),
        ("foundry_load_spec.md", CLAUDE_LOAD_SPEC_CMD),
        ("foundry_create_spec.md", CLAUDE_CREATE_SPEC_CMD),
        ("foundry_update_spec.md", CLAUDE_UPDATE_SPEC_CMD),
    ]
}

/// Common set of command files we install for Cursor (no frontmatter) (filename -> content)
fn cursor_command_files() -> Vec<(&'static str, &'static str)> {
    vec![
        ("foundry_analyze_project.md", CURSOR_ANALYZE_PROJECT_CMD),
        ("foundry_create_project.md", CURSOR_CREATE_PROJECT_CMD),
        ("foundry_list_specs.md", CURSOR_LIST_SPECS_CMD),
        ("foundry_load_spec.md", CURSOR_LOAD_SPEC_CMD),
        ("foundry_create_spec.md", CURSOR_CREATE_SPEC_CMD),
        ("foundry_update_spec.md", CURSOR_UPDATE_SPEC_CMD),
    ]
}

/// Resolve Claude commands directory: ~/.claude/commands/foundry
pub fn claude_commands_dir(config_dir: &Path) -> PathBuf {
    config_dir.join("commands")
}

/// Resolve Cursor commands directory: <project>/.cursor/commands/foundry
pub fn cursor_commands_dir(config_dir: &Path) -> PathBuf {
    config_dir.join("commands")
}

/// Install commands into the given directory, creating parent dirs as needed
/// Install both sets by default (legacy behavior)
pub fn install_commands(commands_dir: &Path) -> Result<String> {
    if let Some(parent) = commands_dir.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create commands parent dir: {:?}", parent))?;
    }
    fs::create_dir_all(commands_dir)
        .with_context(|| format!("Failed to create commands dir: {:?}", commands_dir))?;

    let mut created = 0usize;
    for (filename, content) in claude_command_files()
        .into_iter()
        .chain(cursor_command_files())
    {
        let path = commands_dir.join(filename);
        fs::write(&path, content)
            .with_context(|| format!("Failed to write command file: {:?}", path))?;
        created += 1;
    }

    Ok(format!(
        "Created Foundry commands: {} (in {})",
        created,
        commands_dir.to_string_lossy()
    ))
}

/// Install only Claude-style commands (with frontmatter)
pub fn install_claude_commands(commands_dir: &Path) -> Result<String> {
    if let Some(parent) = commands_dir.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create commands parent dir: {:?}", parent))?;
    }
    fs::create_dir_all(commands_dir)
        .with_context(|| format!("Failed to create commands dir: {:?}", commands_dir))?;

    let mut created = 0usize;
    for (filename, content) in claude_command_files() {
        let path = commands_dir.join(filename);
        fs::write(&path, content)
            .with_context(|| format!("Failed to write command file: {:?}", path))?;
        created += 1;
    }

    Ok(format!(
        "Created Foundry commands: {} (in {})",
        created,
        commands_dir.to_string_lossy()
    ))
}

/// Install only Cursor-style commands (no frontmatter)
pub fn install_cursor_commands(commands_dir: &Path) -> Result<String> {
    if let Some(parent) = commands_dir.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create commands parent dir: {:?}", parent))?;
    }
    fs::create_dir_all(commands_dir)
        .with_context(|| format!("Failed to create commands dir: {:?}", commands_dir))?;

    let mut created = 0usize;
    for (filename, content) in cursor_command_files() {
        let path = commands_dir.join(filename);
        fs::write(&path, content)
            .with_context(|| format!("Failed to write command file: {:?}", path))?;
        created += 1;
    }

    Ok(format!(
        "Created Foundry commands: {} (in {})",
        created,
        commands_dir.to_string_lossy()
    ))
}

/// Remove commands directory (non-fatal if missing)
pub fn remove_commands(commands_dir: &Path) -> Result<Option<String>> {
    if !commands_dir.exists() {
        return Ok(None);
    }

    let mut removed = 0usize;
    for (filename, _content) in claude_command_files()
        .into_iter()
        .chain(cursor_command_files())
    {
        let path = commands_dir.join(filename);
        if path.exists() {
            fs::remove_file(&path)
                .with_context(|| format!("Failed to remove command file: {:?}", path))?;
            removed += 1;
        }
    }

    // Clean up directory if empty
    if commands_dir.read_dir()?.next().is_none() {
        let _ = fs::remove_dir(commands_dir);
    }

    Ok(Some(format!(
        "Removed Foundry commands: {} (from {})",
        removed,
        commands_dir.to_string_lossy()
    )))
}

// ---- Command contents ----

const CLAUDE_ANALYZE_PROJECT_CMD: &str = r###"---
allowed-tools: mcp__foundry__analyze_project, mcp__foundry__list_projects
description: Analyze an existing codebase and create Foundry project docs
argument-hint: [project-name]
---

Goal: Analyze the current codebase, draft complete artifacts (vision, tech_stack, summary), confirm with the user, then create the Foundry project context via MCP.

## Context Gathering Phase

**Before Analysis**:
1. Scan repository structure: README, package.json/Cargo.toml/pyproject.toml, docker files, CI configs
2. Check existing Foundry projects: Use list_projects to avoid naming conflicts
3. Identify deployment context: Look for infra configs, deployment scripts, environment files

## Analysis Workflow

**Step 1: Repository Discovery**
- Languages and frameworks (check package managers, import patterns)
- Build tools and deployment infrastructure
- Service architecture and component relationships
- Documentation quality and project maturity

**Step 2: Draft Creation** (no boilerplate content)
- vision (200+ chars): Clear problem statement, target users, unique value proposition, roadmap priorities
- tech_stack (150+ chars): Languages, frameworks, infrastructure, deployment with detailed rationale
- summary (100+ chars): 2-3 sentences capturing project essence for quick context loading

**Step 3: User Collaboration**
- Present drafts with specific questions about gaps or assumptions
- Iterate based on user feedback to ensure accuracy and completeness
- Confirm all content meets minimum length and quality requirements

**Step 4: MCP Integration**
Call analyze_project with finalized content:
```json
{"name":"analyze_project","arguments":{"project_name":"$1","vision":"<final_vision>","tech_stack":"<final_tech_stack>","summary":"<final_summary>"}}
```

## Error Recovery Patterns

**Content Validation Failures:**
- Length too short → Expand specific sections with user input and retry
- Missing technical details → Ask targeted questions about architecture/deployment
- Vague descriptions → Request specific examples and concrete details

**MCP Tool Errors:**
- Project name conflicts → Suggest alternatives or ask user preference
- Invalid characters in name → Auto-sanitize to kebab-case format
- Network/permission issues → Guide user to check foundry installation status

## Workflow Continuity

**After Successful Analysis:**
- Suggest: `/foundry_create_spec` to plan first feature
- Consider: `/foundry_list_specs` if project already has specifications
- Next logical action: Define initial feature specifications for development roadmap

## Tool Reference
- analyze_project(project_name, vision, tech_stack, summary)
  - Creates ~/.foundry/PROJECT with vision.md, tech-stack.md, summary.md from LLM-provided content
  - Content minimums: vision ≥200 chars, tech_stack ≥150, summary ≥100
  - Returns next_steps and workflow_hints for AI development workflow guidance
"###;

const CLAUDE_CREATE_PROJECT_CMD: &str = r###"---
allowed-tools: mcp__foundry__create_project, mcp__foundry__list_projects
description: Create a new Foundry project with complete context documents
argument-hint: [project-name]
---

Goal: Collaboratively define vision, tech_stack, and summary for a new project, then create the Foundry project via MCP.

## Context Gathering Phase

**Project Discovery**:
1. Check existing Foundry projects: Use list_projects to avoid naming conflicts and understand existing portfolio
2. Validate project name format: Ensure kebab-case format for consistency
3. Gather comprehensive project context through targeted interview

## Interview Workflow

**Step 1: Problem & Users**
- What specific problem does this project solve?
- Who are the target users and their key use cases?
- What makes this solution unique or valuable?

**Step 2: Technical Constraints**
- What are the preferred programming languages/frameworks?
- Any existing infrastructure or platform requirements?
- Performance, security, or compliance constraints?
- Team size and technical expertise level?

**Step 3: Deployment & Scale**
- Expected deployment environment (cloud, on-premise, hybrid)?
- Anticipated scale and performance requirements?
- Integration requirements with existing systems?

## Document Creation

**Vision Draft** (200+ chars minimum):
- Clear problem statement with specific user pain points
- Target audience and their motivations
- Unique value proposition and competitive advantages
- High-level roadmap priorities and success metrics

**Tech Stack Draft** (150+ chars minimum):
- Primary languages and frameworks with specific rationale
- Infrastructure and deployment platform choices
- Database and data storage decisions
- Development tools, testing, and CI/CD approach
- External integrations and API dependencies

**Summary Draft** (100+ chars minimum):
- 2-3 sentences capturing project essence for quick context loading
- Should enable future AI assistants to understand project purpose immediately

## User Collaboration

**Review Process**:
- Present all three drafts simultaneously for holistic feedback
- Ask specific clarifying questions about technical decisions
- Validate assumptions about user needs and market positioning
- Ensure all content meets quality and length requirements

## MCP Integration

Call create_project with finalized content:
```json
{"name":"create_project","arguments":{"project_name":"$1","vision":"<final_vision>","tech_stack":"<final_tech_stack>","summary":"<final_summary>"}}
```

## Error Recovery Patterns

**Content Validation Failures:**
- Length insufficient → Guide user to expand specific sections with concrete details
- Generic content → Request specific examples, metrics, or technical details
- Inconsistent information → Highlight conflicts and ask for clarification

**MCP Tool Errors:**
- Project name exists → Suggest variations or ask user to choose alternative
- Invalid project name → Auto-suggest kebab-case conversion
- Permission/access issues → Guide user to verify foundry installation and permissions

**Interview Challenges:**
- Vague requirements → Use follow-up questions to drill down to specifics
- Technical uncertainty → Suggest research phase or prototype approach
- Scope too broad → Help break down into focused initial version

## Workflow Continuity

**After Successful Creation:**
- Immediate next step: `/foundry_create_spec` to define first feature or capability
- Consider: `/foundry_load_project` to verify created context and review documents
- Long-term workflow: Build systematic feature specifications for development roadmap

## Tool Reference
- create_project(project_name, vision, tech_stack, summary)
  - Initializes new ~/.foundry/PROJECT with provided documents
  - Content minimums: vision ≥200 chars, tech_stack ≥150, summary ≥100
  - No auto-summarization performed - all content must be LLM-provided
  - Returns next_steps and workflow_hints for continued development planning
"###;

const CLAUDE_LIST_SPECS_CMD: &str = r###"---
allowed-tools: mcp__foundry__list_specs, mcp__foundry__list_projects, mcp__foundry__load_project
description: List specs for a project with lightweight discovery
argument-hint: [project-name]
---

Goal: Discover and summarize existing specs to help the user pick one for detailed work.

## Context Gathering Phase

**Project Validation**:
1. If no project name provided, use list_projects to show available options
2. Verify project exists before attempting to list specs
3. Check project context to understand domain and scope

## Discovery Workflow

**Step 1: Project Resolution**
- If `$1` (project name) missing → call list_projects and guide user selection
- If project name provided → validate it exists in Foundry system
- Display project summary context if available for user orientation

**Step 2: Spec Listing**
Call list_specs for the target project:
```json
{"name":"list_specs","arguments":{"project_name":"$1"}}
```

**Step 3: Results Presentation**
- Present specs in chronological order (newest first) for development context
- Include: spec name, feature name, creation date, and brief purpose
- Highlight incomplete or recently modified specs for priority attention
- Group by status or category if pattern emerges

## Error Recovery Patterns

**Project Issues:**
- Project not found → Show list_projects and ask user to choose valid project
- Empty project → Guide user to create first spec with `/foundry_create_spec`
- Permission errors → Check foundry installation status and guide troubleshooting

**Display Challenges:**
- Many specs → Offer filtering options or group by status/feature area
- Long spec names → Truncate display but preserve full names for selection
- No specs found → Emphasize this is normal for new projects and suggest creating first spec

## Workflow Continuity

**After Listing Specs:**
- **If specs exist:** Suggest `/foundry_load_spec [project-name] [spec-query]` for detailed work
- **If no specs:** Recommend `/foundry_create_spec [project-name] [feature-name]` to start development
- **For planning:** Consider `/foundry_load_project [project-name]` to review overall context

**Pattern Recognition:**
- Multiple incomplete specs → Suggest prioritizing completion with `/foundry_update_spec`
- Recent spec activity → Highlight likely continuation candidates
- Feature gaps → Point out areas where new specs might be valuable

## Results Interpretation

**Spec Status Analysis:**
- Identify which specs have incomplete tasks (from task-list.md metadata)
- Highlight recently created or modified specs for development priority
- Note any patterns in feature naming or development progression

**Development Insights:**
- Project maturity level based on spec count and completeness
- Feature area coverage and potential gaps
- Development velocity and current focus areas

## Tool Reference
- list_specs(project_name)
  - Returns lightweight metadata for specs without loading full content
  - Includes: spec names, feature names, creation dates, and status indicators
  - Enables quick discovery without performance impact of loading complete specifications
"###;

const CLAUDE_LOAD_SPEC_CMD: &str = r###"---
allowed-tools: mcp__foundry__load_spec, mcp__foundry__list_projects, mcp__foundry__list_specs
description: Load a spec by fuzzy name with discovery fallback
argument-hint: [project-name] [spec-query]
---

Goal: Load the target spec using fuzzy matching, resolve ambiguities with the user, then provide actionable next steps.

## Context Gathering Phase

**Parameter Resolution**:
1. If project name (`$1`) missing → use list_projects to guide selection
2. If spec query (`$2`) missing → call list_specs and present options
3. Validate project exists and contains specifications before proceeding

## Discovery Workflow

**Step 1: Project & Spec Resolution**
- **Missing project:** Use list_projects to show available projects and guide selection
- **Missing spec query:** Call list_specs for the project and present options in user-friendly format
- **Both provided:** Proceed directly to fuzzy matching load operation

**Step 2: Spec Loading with Fuzzy Matching**
Call load_spec with intelligent fuzzy matching:
```json
{"name":"load_spec","arguments":{"project_name":"$1","spec_name":"$2"}}
```

**Step 3: Ambiguity Resolution**
- **High confidence match:** Load directly and present content summary
- **Multiple matches:** Display candidates with feature names and dates, ask user to choose
- **No matches:** Suggest alternatives or offer to create new spec with provided name

## Error Recovery Patterns

**Loading Failures:**
- **Spec not found:** Show available specs from list_specs and suggest alternatives
- **Corrupted spec files:** Report specific file issues and guide user to recovery options
- **Permission errors:** Check foundry installation and guide user through troubleshooting

**Fuzzy Matching Issues:**
- **Too many matches:** Show top 5 candidates with creation dates and let user choose
- **Low confidence matches:** Present options with similarity scores and confirmation prompts
- **Ambiguous queries:** Ask user to provide more specific search terms or use exact spec names

**Project Context Errors:**
- **Project not found:** Guide user to create project first with `/foundry_create_project`
- **Empty project:** Suggest creating first spec with `/foundry_create_spec`
- **Foundry system issues:** Direct user to check installation status

## Content Analysis & Presentation

**Spec Overview:**
- **Feature Summary:** Extract and highlight main feature purpose and scope
- **Current Status:** Analyze task completion percentage and remaining work
- **Recent Changes:** Note any recent modifications or additions to context

**Task Analysis:**
- **Incomplete Tasks:** Highlight pending tasks that need attention
- **Blocked Tasks:** Identify any dependencies or prerequisites preventing progress
- **Priority Assessment:** Suggest logical next tasks based on current state

**Documentation Quality:**
- **Completeness Check:** Identify any missing sections or incomplete documentation
- **Clarity Assessment:** Note areas that might need clarification or expansion
- **Dependency Mapping:** Highlight connections to other specs or external requirements

## Workflow Continuity

**Immediate Next Steps:**
- **For Active Development:** Suggest `/foundry_update_spec` to mark tasks complete or add new ones
- **For Planning:** Consider reviewing project context with `/foundry_load_project`
- **For Extension:** Use `/foundry_update_spec` to add new requirements or notes

**Development Workflow:**
- **Implementation Phase:** Guide user to mark tasks as complete as they progress
- **Review Phase:** Suggest adding implementation notes or lessons learned
- **Iteration Phase:** Help identify next logical features or improvements

**Quality Assurance:**
- **Task Completion:** Verify all tasks align with current feature scope
- **Documentation Updates:** Ensure notes reflect current understanding and decisions
- **Dependency Validation:** Check that requirements still match project constraints

## Tool Reference
- load_spec(project_name, spec_name)
  - Loads complete specification including spec.md, notes.md, and task-list.md
  - Supports fuzzy spec_name matching for user convenience
  - Returns full context with summary, task analysis, and workflow guidance
  - Enables immediate transition to development or planning activities
"###;

const CLAUDE_CREATE_SPEC_CMD: &str = r###"---
allowed-tools: mcp__foundry__create_spec, mcp__foundry__load_project, mcp__foundry__list_specs
description: Create a new spec with tasks and notes for a project
argument-hint: [project-name] [feature-name]
---

Goal: Collaboratively draft a comprehensive feature specification, implementation checklist, and design notes, then create it via MCP.

## Context Gathering Phase

**Project & Feature Discovery**:
1. Load project context to understand domain, tech stack, and existing features
2. List existing specs to avoid duplication and ensure naming consistency
3. Validate feature scope fits within project vision and constraints

## Feature Definition Workflow

**Step 1: Feature Scope Interview**
- **Problem Statement:** What specific user problem does this feature solve?
- **User Stories:** Who will use this feature and what are their key workflows?
- **Acceptance Criteria:** How will we know this feature is complete and successful?
- **Constraints:** Any technical, business, or timeline limitations?
- **Dependencies:** What other features or systems does this depend on?

**Step 2: Technical Planning**
- **Integration Points:** How does this connect to existing system architecture?
- **Data Requirements:** What new data models or storage needs are required?
- **API Design:** What interfaces (REST, GraphQL, internal APIs) need to be created/modified?
- **Performance Considerations:** Expected load, scalability, or optimization requirements?
- **Security & Compliance:** Any authentication, authorization, or compliance needs?

## Document Drafting

**Specification Document (spec.md)** - Must include all sections:
```markdown
# [Feature Name]

## Overview
Clear description of feature purpose and value proposition

## Requirements
### Functional Requirements
- Specific user-facing capabilities
### Non-Functional Requirements
- Performance, security, compliance needs

## Acceptance Criteria
- Measurable success criteria
- User experience expectations
- Technical validation points

## Implementation Approach
- High-level technical strategy
- Integration patterns
- Database/API changes needed

## Dependencies
- Other features or systems required
- External service integrations
- Team coordination needs
```

**Task Checklist (task-list.md)** - Actionable implementation steps:
```markdown
## Planning Phase
- [ ] Detailed technical design
- [ ] API endpoint specification
- [ ] Database schema changes

## Development Phase
- [ ] Core feature implementation
- [ ] Unit tests and validation
- [ ] Integration testing

## Deployment Phase
- [ ] Documentation updates
- [ ] Production deployment
- [ ] User acceptance validation
```

**Design Notes (notes.md)** - Context and rationale:
- Technical decision reasoning
- Alternative approaches considered
- Risk assessment and mitigation
- Future extension opportunities

## User Collaboration & Review

**Draft Review Process**:
- Present all three documents for holistic feedback
- Ask specific questions about technical assumptions
- Validate feature scope and complexity assessment
- Ensure task list covers complete implementation lifecycle

**Refinement Iteration**:
- Address gaps in requirements or acceptance criteria
- Clarify technical approach based on user feedback
- Adjust task granularity for development workflow
- Update notes with additional context or constraints

## MCP Integration

Create specification with complete content:
```json
{"name":"create_spec","arguments":{"project_name":"$1","feature_name":"$2","spec":"<final_spec>","tasks":"<final_tasks>","notes":"<final_notes>"}}
```

## Error Recovery Patterns

**Content Validation Failures:**
- **Incomplete sections:** Guide user to expand missing specification areas
- **Vague requirements:** Ask for specific, measurable criteria and examples
- **Task list gaps:** Ensure complete development lifecycle coverage
- **Length requirements:** Help user add concrete details and context

**Feature Definition Issues:**
- **Scope too broad:** Help break down into focused, manageable feature
- **Unclear value:** Work with user to define specific user benefits
- **Technical complexity:** Suggest phased approach or simplified initial version
- **Dependency conflicts:** Review existing specs and help resolve integration issues

**MCP Tool Errors:**
- **Naming conflicts:** Suggest alternative feature names or versioning approach
- **Invalid characters:** Auto-sanitize to snake_case format for consistency
- **Project context missing:** Guide user to create/load project first

## Workflow Continuity

**After Successful Creation:**
- **Immediate next:** Begin implementation using `/foundry_update_spec` to track progress
- **Planning review:** Use `/foundry_load_spec` to verify complete context was captured
- **Portfolio view:** Consider `/foundry_list_specs` to see feature in project context

**Development Progression:**
- **Implementation tracking:** Mark tasks complete as development progresses
- **Requirement changes:** Update spec sections when scope or requirements evolve
- **Knowledge capture:** Add implementation insights and lessons learned to notes

## Tool Reference
- create_spec(project_name, feature_name, spec, tasks, notes)
  - Creates timestamped directory: specs/YYYYMMDD_HHMMSS_feature_name/
  - Files: spec.md (requirements), task-list.md (checklist), notes.md (context)
  - Enforces one feature per spec for focused development workflow
  - Content must be comprehensive and LLM-provided (no auto-generation)
"###;

const CLAUDE_UPDATE_SPEC_CMD: &str = r###"---
allowed-tools: mcp__foundry__update_spec, mcp__foundry__load_spec
description: Update a spec using Foundry edit_commands safely and idempotently
argument-hint: [project-name] [spec-query] [update-description]
---

Goal: Apply precise, targeted updates to specification documents using idempotent edit commands.

## Context Gathering Phase

**Spec Validation & Loading**:
1. Ensure target spec is loaded with current content via load_spec
2. Understand user's intended changes and map to appropriate edit commands
3. Validate that proposed changes align with spec structure and project context

## Edit Command Rules & Guidelines

**Command Selection Strategy**:
- **Task Management:** Use `upsert_task` to add new tasks, `set_task_status` to mark complete/incomplete
- **Content Addition:** Use `append_to_section` for adding requirements, notes, or spec content
- **Selector Precision:** Always use exact text from current spec content for reliable targeting

**Critical Rules**:
- **Tasks:** Never use append commands on task-list.md - only `upsert_task` and `set_task_status`
- **Spec/Notes:** Only use `append_to_section` for spec.md and notes.md content
- **Text Matching:** Selectors must match existing content exactly (including whitespace and formatting)

**Required Fields & Target Restrictions**:
- **Required Fields:** `set_task_status` requires `status`; all other modifying commands require `content`
- **Target Restrictions:** Task commands (`set_task_status`, `upsert_task`) only work with `tasks`; `append_to_section` is invalid for `tasks`

**Selector normalization & idempotence**:
- **Tasks selector normalization:** Matching ignores checkbox prefix, collapses internal whitespace, and ignores trailing periods. Either of these values works: `Implement OAuth2 integration` or `- [ ] Implement OAuth2 integration`.
- **Section headers:** Case-insensitive match on the full header string including hashes (surrounding whitespace trimmed). Use exact header text like `## Requirements`.
- **Idempotence:** All commands are repeat-safe. When no change is needed, the tool reports a skipped/idempotent outcome.

## Update Workflow

**Step 1: Current State Analysis**
- Load spec with `load_spec` if not already loaded
- Review current content to understand structure and existing sections
- Identify exact text for selectors to ensure reliable command execution

**Step 2: Command Planning**
Based on user intent, select appropriate command patterns:

**Task Updates:**
```json
// Mark task as complete
{"target":"tasks","command":"set_task_status","selector":{"type":"task_text","value":"- [ ] Implement user authentication"},"status":"done"}

// Add new task
{"target":"tasks","command":"upsert_task","selector":{"type":"task_text","value":"- [ ] New task description"},"content":"- [ ] New task description"}
```

**Specification Updates:**
```json
// Add requirement to existing section
{"target":"spec","command":"append_to_section","selector":{"type":"section","value":"## Requirements"},"content":"- New functional requirement"}

// Add content to implementation section
{"target":"spec","command":"append_to_section","selector":{"type":"section","value":"## Implementation Approach"},"content":"Additional technical details"}
```

**Notes Updates:**
```json
// Add design rationale
{"target":"notes","command":"append_to_section","selector":{"type":"section","value":"## Technical Decisions"},"content":"Reasoning for technology choice"}
```

**Step 3: User Confirmation**
- Present proposed edit commands with clear description of changes
- Confirm commands align with user's intended modifications
- Verify selector text matches current spec content exactly

**Step 4: Command Execution**
Execute single update_spec call with complete commands array:
```json
{"name":"update_spec","arguments":{"project_name":"$1","spec_name":"$2","commands":[/* array of edit commands */]}}
```

Note: `commands` is required and must be a JSON array when using MCP tools. When using the CLI directly, pass the same array as a JSON string argument.

### Minimal Valid Example
```json
{"name":"update_spec","arguments":{"project_name":"$1","spec_name":"$2","commands":[
  {"target":"spec","command":"append_to_section","selector":{"type":"section","value":"## Overview"},"content":"New line"}
]}}
```

### Additional Examples
- Remove a task (no additional fields):
`{"name":"update_spec","arguments":{"project_name":"$1","spec_name":"$2","commands":[{"target":"tasks","command":"remove_list_item","selector":{"type":"task_text","value":"Outdated task"}}]}}`

- Replace text in a section (requires content field):
`{"name":"update_spec","arguments":{"project_name":"$1","spec_name":"$2","commands":[{"target":"spec","command":"replace_in_section","selector":{"type":"text_in_section","section":"## Requirements","text":"MySQL 5.7"},"content":"MySQL 8.0"}]}}`

### Supported Operations
- Task Management: `set_task_status`, `upsert_task`
- Content Addition: `append_to_section`
- Content Removal: `remove_list_item`, `remove_from_section`, `remove_section`
- Content Replacement: `replace_list_item`, `replace_in_section`, `replace_section_content`

### Recommended Ordering
1) remove_list_item → 2) replace_in_section → 3) replace_section_content → 4) append_to_section

## Error Recovery Patterns

**Selector Failures:**
- **Text not found:** Re-load spec, copy exact text including whitespace, retry with precise selector
- **Ambiguous matches:** Use longer text snippets for unique identification
- **Section missing:** Guide user to add section first or choose different target section
- **Candidate suggestions:** On failure, the tool returns selector candidates with short previews; copy one suggestion into your next attempt.

**Command Validation Errors:**
- **Invalid command structure:** Review command format and fix JSON syntax issues
- **Target mismatch:** Ensure task commands target "tasks", spec commands target "spec"/"notes"
- **Content format issues:** Validate markdown syntax and checkbox format for tasks

**Idempotency Issues:**
- **Duplicate additions:** Check if content already exists before adding
- **Conflicting status:** Verify current task status before changing
- **Content conflicts:** Review existing content for logical consistency

## Advanced Update Patterns

**Bulk Task Management:**
```json
[
  {"target":"tasks","command":"set_task_status","selector":{"type":"task_text","value":"- [x] First completed task"},"status":"done"},
  {"target":"tasks","command":"upsert_task","selector":{"type":"task_text","value":"- [ ] New follow-up task"},"content":"- [ ] New follow-up task"}
]
```

**Comprehensive Spec Updates:**
```json
[
  {"target":"spec","command":"append_to_section","selector":{"type":"section","value":"## Requirements"},"content":"- Additional requirement"},
  {"target":"notes","command":"append_to_section","selector":{"type":"section","value":"## Implementation Notes"},"content":"Key insight from development"}
]
```

## Workflow Continuity

**After Successful Updates:**
- **Progress tracking:** Use `/foundry_load_spec` to review updated content and assess progress
- **Related updates:** Consider if changes require updates to other specs or project documents
- **Development flow:** Continue implementation and use `/foundry_update_spec` iteratively as work progresses

**Quality Assurance:**
- **Content consistency:** Verify all changes maintain logical flow and accuracy
- **Task dependencies:** Ensure task updates reflect actual implementation progress
- **Documentation sync:** Keep spec, tasks, and notes aligned with current understanding

## Tool Reference
- update_spec(project_name, spec_name, commands[])
  - **Supported Commands:**
    - `set_task_status`: Change task completion status (done/todo)
    - `upsert_task`: Add new task or update existing task content
    - `append_to_section`: Add content to spec.md or notes.md sections
  - **Idempotent Design:** Safe to re-run commands without duplication
  - **Error Recovery:** If selector fails, load current spec content and retry with exact text
  - **Batch Operations:** Single call can execute multiple related commands atomically
"###;

// Addendum: The following quick reference clarifies required args and common pitfalls for update_spec.
// Appended to the installed command to reduce user error when constructing payloads.

// Cursor-formatted (no frontmatter) variants

const CURSOR_ANALYZE_PROJECT_CMD: &str = r###"# Analyze Project With Foundry

## Overview
Analyze an existing codebase and create comprehensive Foundry project documents using MCP tools.

## Context Gathering Phase

**Before Starting Analysis:**
1. **Repository Structure Scan:** Examine README, package.json/Cargo.toml/pyproject.toml, docker files, CI configs
2. **Foundry System Check:** Use list_projects to avoid naming conflicts and understand existing portfolio
3. **Deployment Context:** Look for infrastructure configs, deployment scripts, environment files

## Detailed Analysis Workflow

### Step 1: Repository Discovery
**Technical Infrastructure:**
- Languages and frameworks (check package managers, import patterns, build configs)
- Build tools and deployment infrastructure (CI/CD, containerization, cloud configs)
- Service architecture and component relationships (microservices, monolith, APIs)
- Documentation quality and project maturity indicators

### Step 2: Document Drafting (No Boilerplate Content)
**Vision Document (200+ chars minimum):**
- Clear problem statement with specific user pain points
- Target audience and their core motivations
- Unique value proposition and competitive advantages
- High-level roadmap priorities and success metrics

**Tech Stack Document (150+ chars minimum):**
- Primary languages and frameworks with specific rationale
- Infrastructure and deployment platform choices with reasoning
- Database and data storage decisions with justification
- Development tools, testing, and CI/CD approach
- External integrations and API dependencies

**Summary Document (100+ chars minimum):**
- 2-3 sentences capturing project essence for immediate context loading
- Should enable future AI assistants to understand project purpose instantly

### Step 3: User Collaboration & Refinement
**Review Process:**
- Present all three documents simultaneously for holistic feedback
- Ask specific clarifying questions about technical decisions and business context
- Validate assumptions about user needs, market positioning, and technical constraints
- Ensure all content meets quality standards and minimum length requirements

### Step 4: MCP Integration
**Execute Project Creation:**
```json
{"name":"analyze_project","arguments":{"project_name":"$1","vision":"<final_vision>","tech_stack":"<final_tech_stack>","summary":"<final_summary>"}}
```

## Error Recovery Strategies

**Content Validation Issues:**
- **Length insufficient:** Guide user to expand sections with concrete details and examples
- **Generic content:** Request specific metrics, technology versions, and architectural details
- **Inconsistent information:** Highlight conflicts between documents and ask for clarification

**Technical Analysis Challenges:**
- **Complex architecture:** Break down into focused sections and validate understanding with user
- **Missing documentation:** Ask targeted questions to fill gaps in technical understanding
- **Legacy systems:** Help identify modernization opportunities and technical debt areas

**MCP Tool Errors:**
- **Project name conflicts:** Suggest variations or ask user to choose alternative naming
- **Invalid project format:** Auto-suggest kebab-case conversion for consistency
- **Permission/access issues:** Guide user to verify Foundry installation and system permissions

## Workflow Continuity

**After Successful Analysis:**
- **Immediate next step:** Use `/foundry_create_spec` to define first feature or improvement
- **Planning phase:** Consider `/foundry_list_specs` if project already has some specifications
- **Development roadmap:** Begin systematic feature specification process for development workflow

**Quality Assurance:**
- **Content accuracy:** Verify technical details match actual codebase implementation
- **Future usability:** Ensure documents provide sufficient context for development teams
- **Completeness:** Confirm all major architectural and business aspects are captured

## Advanced Analysis Techniques

**Architectural Assessment:**
- **Scalability patterns:** Identify current approach to handling growth and load
- **Security posture:** Note authentication, authorization, and data protection patterns
- **Performance characteristics:** Understand current optimization strategies and bottlenecks
- **Maintainability factors:** Assess code organization, testing, and documentation practices

**Business Context Integration:**
- **User journey mapping:** Connect technical architecture to user experience flows
- **Competitive positioning:** Understand how technical choices support business differentiation
- **Growth planning:** Identify technical enablers and constraints for business scaling
- **Risk assessment:** Note technical debt areas and potential modernization needs

## Instructions for Agent
**Primary Workflow:** Systematically gather repository context through file analysis and codebase examination, draft comprehensive vision/tech-stack/summary documents collaboratively with the user, then call the Foundry MCP "analyze_project" tool with finalized content that meets all quality and length requirements.

**Success Criteria:** Created project documents should enable any future developer or AI assistant to understand the project's purpose, technical architecture, and development context without additional research.
"###;

const CURSOR_CREATE_PROJECT_CMD: &str = r###"# Create Foundry Project

## Overview
Create a new Foundry project with comprehensive context documents through collaborative design process.

## Context Gathering Phase

**Project Portfolio Management:**
1. **Foundry System Check:** Use list_projects to avoid naming conflicts and understand existing project portfolio
2. **Project Name Validation:** Ensure kebab-case format for consistency across Foundry system
3. **Scope Definition:** Establish clear boundaries and initial feature set for focused development

## Comprehensive Interview Workflow

### Phase 1: Problem & Market Context
**Core Problem Analysis:**
- What specific user pain point or business problem does this project address?
- How do users currently solve this problem, and what are the limitations?
- What would success look like for both users and the business?
- Are there existing solutions, and how would this be different/better?

**Target Audience Deep Dive:**
- Who are the primary users and what are their technical skill levels?
- What are their key workflows and usage patterns?
- What are their most important requirements and constraints?
- How do they currently discover and adopt new solutions?

### Phase 2: Technical Foundation
**Technology Preferences & Constraints:**
- What programming languages and frameworks are preferred by the team?
- Are there existing infrastructure or platform requirements to consider?
- What are the performance, security, or compliance constraints?
- What is the team's technical expertise and preferred development tools?

**Architecture & Integration:**
- Will this integrate with existing systems or be standalone?
- What are the expected scale and performance requirements?
- Are there specific API or data integration requirements?
- What deployment environments are planned (cloud, on-premise, hybrid)?

### Phase 3: Project Planning
**Development Approach:**
- What is the development timeline and resource availability?
- Should this be built in phases, and what would the MVP include?
- What are the key risks and how might they be mitigated?
- How will success be measured and validated?

## Document Creation Process

### Vision Document (200+ chars minimum)
**Structure & Content:**
- **Problem Statement:** Clear description of user pain points and market opportunity
- **Target Users:** Specific audience definition with user personas and use cases
- **Value Proposition:** Unique benefits and competitive advantages
- **Success Metrics:** Measurable outcomes and key performance indicators
- **Roadmap Priorities:** High-level development phases and key milestones

### Tech Stack Document (150+ chars minimum)
**Comprehensive Technical Decisions:**
- **Core Technologies:** Primary languages, frameworks, and development tools with rationale
- **Infrastructure Choices:** Hosting, database, caching, and storage decisions with reasoning
- **Development Workflow:** Testing frameworks, CI/CD, code quality, and deployment processes
- **External Dependencies:** Third-party services, APIs, and integration requirements
- **Architecture Patterns:** Design patterns, scalability approach, and security considerations

### Summary Document (100+ chars minimum)
**Quick Context Loading:**
- **Essence Capture:** 2-3 sentences that immediately convey project purpose and approach
- **AI Assistant Enablement:** Content that allows future AI assistants to understand context instantly
- **Key Differentiators:** Brief mention of what makes this project unique or valuable

## Collaborative Review & Refinement

### Multi-Round Feedback Process
**Initial Draft Review:**
- Present all three documents together for comprehensive evaluation
- Identify gaps, inconsistencies, or areas needing expansion
- Validate technical assumptions and business logic
- Ensure content meets minimum length and quality requirements

**Iterative Improvement:**
- Address specific feedback on technical approach and market positioning
- Refine language for clarity and add concrete examples where needed
- Validate that all stakeholder concerns and requirements are captured
- Confirm alignment between vision, technical choices, and success criteria

### Quality Validation Checklist
- [ ] All documents meet minimum character requirements
- [ ] Technical decisions include clear rationale and context
- [ ] Vision articulates clear value proposition and success metrics
- [ ] Content is specific enough for implementation guidance
- [ ] Documents work together as cohesive project foundation

## MCP Integration & Execution

**Create Project with Finalized Content:**
```json
{"name":"create_project","arguments":{"project_name":"$1","vision":"<final_vision>","tech_stack":"<final_tech_stack>","summary":"<final_summary>"}}
```

## Error Recovery & Problem Solving

**Content Development Issues:**
- **Vague requirements:** Use targeted follow-up questions to get specific details
- **Technical uncertainty:** Suggest research phases or prototype validation approaches
- **Scope too broad:** Help break down into focused initial version with clear expansion path
- **Conflicting priorities:** Facilitate decision-making through trade-off analysis

**Validation Failures:**
- **Length insufficient:** Guide user to add concrete examples, metrics, and detailed explanations
- **Generic content:** Request specific technology versions, architectural patterns, and business context
- **Inconsistent information:** Highlight conflicts and work with user to resolve contradictions

**System Integration Issues:**
- **Project name conflicts:** Suggest alternatives that maintain semantic meaning
- **Invalid naming format:** Provide kebab-case suggestions while preserving intent
- **MCP connectivity problems:** Guide user through Foundry installation verification

## Workflow Continuity

**After Successful Project Creation:**
- **Immediate next step:** Use `/foundry_create_spec` to define first feature or core capability
- **Context verification:** Consider `/foundry_load_project` to review created documents and ensure completeness
- **Development planning:** Begin systematic feature specification process for structured development approach

**Long-term Development Strategy:**
- **Feature prioritization:** Use project vision to guide specification creation order
- **Technical validation:** Ensure each new spec aligns with established tech stack decisions
- **Progress tracking:** Maintain project coherence as specifications and features evolve

## Instructions for Agent
**Primary Mission:** Conduct comprehensive project interview to gather complete context, collaboratively draft vision/tech-stack/summary documents that meet all quality requirements, then execute project creation through Foundry MCP tools.

**Success Standard:** Created project should provide sufficient context for any development team or AI assistant to understand the project's purpose, technical approach, and success criteria without requiring additional discovery work.
"###;

const CURSOR_LIST_SPECS_CMD: &str = r###"# List Specs With Foundry

## Overview
Discover and analyze existing specifications for a project to guide development planning and prioritization.

## Context Gathering Phase

**Project Discovery & Validation:**
1. **Parameter Resolution:** If project name missing, use list_projects to show available options and guide selection
2. **Project Context:** Load project summary if available to understand domain and development scope
3. **System Validation:** Ensure project exists in Foundry system before attempting spec listing

## Comprehensive Discovery Workflow

### Step 1: Project Resolution
**Missing Project Name Handling:**
- Execute list_projects to display available Foundry projects with metadata
- Present projects in user-friendly format with creation dates and brief descriptions
- Guide user to select appropriate project for spec exploration

**Project Validation:**
- Confirm target project exists and is accessible
- Load basic project context (vision/summary) if available for user orientation
- Set expectations about spec discovery process

### Step 2: Specification Listing & Analysis
**Execute Spec Discovery:**
```json
{"name":"list_specs","arguments":{"project_name":"$1"}}
```

**Results Processing & Presentation:**
- **Chronological Organization:** Present specs with newest first to show development progression
- **Metadata Display:** Include spec names, feature names, creation dates, and brief purpose descriptions
- **Status Assessment:** Highlight incomplete or recently modified specs for priority attention
- **Categorization:** Group by development phase, feature area, or completion status when patterns emerge

### Step 3: Development Insights & Analysis
**Spec Portfolio Assessment:**
- **Project Maturity:** Evaluate development progress based on spec count and completion patterns
- **Feature Coverage:** Identify areas with comprehensive specifications vs gaps needing attention
- **Development Velocity:** Note patterns in spec creation and modification timing
- **Priority Identification:** Highlight specs with incomplete tasks or recent activity

**Pattern Recognition:**
- **Feature Dependencies:** Identify logical groupings or prerequisite relationships
- **Development Phases:** Recognize infrastructure vs feature vs optimization specifications
- **Resource Allocation:** Understand where development effort has been focused

## Error Recovery & Problem Resolution

**Project Access Issues:**
- **Project Not Found:** Display available projects via list_projects and guide proper selection
- **Empty Project:** Emphasize this is normal for new projects and suggest creating first specification
- **Permission Problems:** Guide user through Foundry installation verification and troubleshooting

**Display & Usability Challenges:**
- **Large Spec Count:** Offer filtering options by date, status, or feature area for manageable display
- **Complex Naming:** Present specs with both technical names and user-friendly feature descriptions
- **No Specifications:** Frame as opportunity rather than problem and guide toward spec creation

## Workflow Continuity & Next Steps

### Immediate Action Recommendations
**When Specs Exist:**
- **Active Development:** Suggest `/foundry_load_spec [project-name] [spec-query]` for detailed work on specific features
- **Planning Phase:** Recommend reviewing incomplete specs to understand current development priorities
- **Quality Review:** Consider loading recently modified specs to assess progress and next steps

**When No Specs Found:**
- **First Spec Creation:** Guide toward `/foundry_create_spec [project-name] [feature-name]` to begin development planning
- **Project Planning:** Suggest reviewing project vision to identify logical first features or capabilities
- **Architecture Planning:** Consider starting with foundational or infrastructure specifications

### Strategic Development Guidance
**Portfolio Analysis:**
- **Multiple Incomplete Specs:** Suggest prioritizing completion with `/foundry_update_spec` for focused progress
- **Recent Activity Patterns:** Highlight likely continuation candidates based on modification dates
- **Feature Gap Identification:** Point out areas where new specifications might add value

**Development Workflow Optimization:**
- **Spec Sequencing:** Help identify logical order for feature implementation based on dependencies
- **Resource Planning:** Guide allocation of effort based on spec complexity and business priority
- **Quality Assurance:** Suggest regular spec reviews to maintain accuracy and relevance

## Advanced Analysis Features

### Development Metrics & Insights
**Progress Tracking:**
- **Completion Rates:** Assess percentage of specs with completed task lists
- **Development Patterns:** Identify cycles of specification creation vs implementation
- **Feature Scope:** Understand breadth vs depth of current development approach

**Quality Assessment:**
- **Documentation Standards:** Note consistency in spec format and completeness
- **Update Frequency:** Identify specs that may need attention due to staleness
- **Integration Planning:** Recognize specs that connect to or depend on others

## Instructions for Agent
**Primary Objective:** Efficiently discover project specifications, present them in actionable format with development insights, and guide user toward most logical next steps in their development workflow.

**Value Delivery:** Transform simple spec listing into comprehensive development planning assistance that helps users understand their project's current state and optimal next actions.

**Success Metrics:** User gains clear understanding of project development status, knows which specs need attention, and receives actionable guidance for continuing their development work.
"###;

const CURSOR_LOAD_SPEC_CMD: &str = r###"# Load Spec With Foundry

## Overview
Load a complete specification using intelligent fuzzy matching to enable focused development work.

## Context Gathering & Parameter Resolution

**Discovery Workflow for Missing Parameters:**
1. **Project Name Missing:** Execute list_projects to display available options with metadata and guide user selection
2. **Spec Query Missing:** Execute list_specs for the target project and present specs in user-friendly format
3. **Both Present:** Proceed directly to fuzzy matching and loading workflow

**Validation & Preparation:**
- Verify target project exists in Foundry system before attempting spec operations
- Set user expectations about fuzzy matching capabilities and potential disambiguation needs
- Prepare for potential multiple matches requiring user selection

## Comprehensive Loading Workflow

### Step 1: Intelligent Spec Discovery
**Parameter Resolution Strategy:**
- **Missing Project:** Show list_projects with creation dates and brief descriptions for informed selection
- **Missing Spec:** Display list_specs results with feature names, dates, and completion status
- **Fuzzy Matching:** Use provided spec query with intelligent pattern matching

### Step 2: Spec Loading with Fuzzy Matching
**Execute Load Operation:**
```json
{"name":"load_spec","arguments":{"project_name":"$1","spec_name":"$2"}}
```

**Match Quality Assessment:**
- **High Confidence Single Match:** Load directly and proceed to content analysis
- **Multiple Matches:** Present candidates with feature names, creation dates, and similarity scores
- **Low Confidence:** Show alternatives and ask for clarification or more specific search terms
- **No Matches:** Suggest creating new spec or provide spelling/naming guidance

### Step 3: Comprehensive Content Analysis
**Specification Overview:**
- **Feature Summary:** Extract and highlight main feature purpose, scope, and value proposition
- **Development Status:** Analyze task completion percentage and identify remaining work
- **Context Assessment:** Review requirements, acceptance criteria, and implementation approach

**Task Portfolio Analysis:**
- **Incomplete Tasks:** Identify and prioritize pending tasks requiring attention
- **Blocked Dependencies:** Note any prerequisites or external dependencies preventing progress
- **Implementation Readiness:** Assess which tasks are ready for immediate development work

**Documentation Quality Review:**
- **Completeness Evaluation:** Identify missing sections or areas needing expansion
- **Clarity Assessment:** Note areas that might benefit from additional detail or examples
- **Consistency Check:** Ensure alignment between spec, tasks, and notes documents

## Error Recovery & Problem Resolution

**Loading & Discovery Issues:**
- **Spec Not Found:** Display available specs via list_specs and guide proper selection with fuzzy alternatives
- **Corrupted Spec Files:** Report specific file issues and provide guidance for recovery or recreation
- **Permission Errors:** Guide user through Foundry installation verification and system troubleshooting

**Fuzzy Matching Challenges:**
- **Too Many Matches:** Present top 5 candidates with distinguishing features and creation dates
- **Ambiguous Results:** Ask user for more specific search terms or suggest using exact spec names
- **Similarity Confusion:** Show match confidence scores and distinctive features for better selection

**System Integration Problems:**
- **Project Not Found:** Guide user to create project first or verify correct project name
- **Empty Project State:** Emphasize normal condition for new projects and suggest spec creation
- **MCP Connectivity:** Direct user to verify Foundry installation and system configuration

## Advanced Content Analysis Features

### Development Progress Assessment
**Task Completion Analysis:**
- **Progress Metrics:** Calculate completion percentage and estimate remaining effort
- **Milestone Tracking:** Identify major development phases and current position
- **Velocity Indicators:** Note patterns in task completion and modification history

**Implementation Readiness:**
- **Dependency Resolution:** Check if prerequisite tasks or external dependencies are satisfied
- **Resource Requirements:** Identify technical skills, tools, or infrastructure needed
- **Risk Assessment:** Note potential blockers or challenges mentioned in notes

### Content Quality & Completeness
**Documentation Standards:**
- **Section Coverage:** Verify all required specification sections are present and complete
- **Detail Sufficiency:** Assess whether implementation guidance is adequate for development
- **Example Availability:** Note presence of code examples, mockups, or concrete illustrations

**Integration Context:**
- **Related Specifications:** Identify connections to other specs or project components
- **External Dependencies:** Highlight third-party services, APIs, or system requirements
- **Testing Considerations:** Review acceptance criteria and validation approaches

## Workflow Continuity & Next Steps

### Immediate Development Actions
**For Active Implementation:**
- **Task Progression:** Use `/foundry_update_spec` to mark completed tasks and add new ones as work progresses
- **Requirement Changes:** Update specification sections when scope or requirements evolve during implementation
- **Knowledge Capture:** Add implementation insights, lessons learned, and technical decisions to notes

**For Planning & Review:**
- **Context Verification:** Consider `/foundry_load_project` to review broader project context and alignment
- **Related Work:** Use `/foundry_list_specs` to see this feature in context of overall project portfolio
- **Dependency Planning:** Load related specs that this feature depends on or connects to

### Strategic Development Guidance
**Implementation Prioritization:**
- **Quick Wins:** Identify tasks that can be completed rapidly to build momentum
- **Critical Path:** Highlight tasks that unblock other work or enable major functionality
- **Risk Mitigation:** Suggest tackling uncertain or complex tasks early for learning and adaptation

**Quality Assurance Planning:**
- **Testing Strategy:** Review acceptance criteria and plan comprehensive validation approach
- **Documentation Updates:** Ensure spec remains current as implementation progresses and understanding evolves
- **Integration Testing:** Plan for testing connections to other system components or external services

## Instructions for Agent
**Primary Mission:** Efficiently load target specification with intelligent disambiguation, perform comprehensive content analysis, and provide actionable development guidance based on current spec state.

**Content Delivery:** Present spec information in digestible format with clear development priorities, highlight immediate next steps, and provide context for decision-making.

**Success Standard:** User gains complete understanding of feature requirements, knows exactly what work remains, and has clear guidance for continuing development efficiently.
"###;

const CURSOR_CREATE_SPEC_CMD: &str = r###"# Create Spec With Foundry

## Overview
Create a comprehensive feature specification with detailed implementation guidance through collaborative design process.

## Context Gathering & Project Discovery

**Project Foundation Analysis:**
1. **Load Project Context:** Review project vision, tech stack, and summary to understand domain and constraints
2. **Existing Spec Review:** List current specs to avoid duplication and ensure consistent naming patterns
3. **Feature Scope Validation:** Confirm proposed feature aligns with project vision and technical architecture

**Duplicate Prevention & Naming:**
- Scan existing spec names and feature areas to avoid conflicts or overlaps
- Ensure new feature name follows project conventions and semantic clarity
- Validate that feature scope is appropriately focused and doesn't duplicate existing functionality

## Comprehensive Feature Definition Process

### Phase 1: Feature Discovery & Requirements
**Problem Statement & User Value:**
- What specific user pain point or business need does this feature address?
- How do users currently handle this need, and what are the limitations?
- What would success look like from both user and business perspectives?
- How does this feature support the overall project vision and roadmap?

**User Stories & Workflows:**
- Who are the primary users of this feature and what are their technical skill levels?
- What are the key user workflows and interaction patterns?
- What are the most important requirements from a user experience perspective?
- How will users discover, learn, and adopt this feature?

**Technical Requirements & Constraints:**
- How does this feature integrate with existing system architecture?
- What are the performance, security, or compliance requirements?
- Are there specific technology or platform constraints to consider?
- What external dependencies or third-party integrations are needed?

### Phase 2: Technical Design & Planning
**Architecture & Integration:**
- How does this feature connect to existing components and data models?
- What new APIs, database changes, or infrastructure components are needed?
- How will this feature handle errors, edge cases, and recovery scenarios?
- What are the scalability and maintenance considerations?

**Implementation Strategy:**
- Should this be implemented in phases, and what would the MVP include?
- What are the key technical risks and how should they be mitigated?
- What testing strategies are needed for comprehensive validation?
- How will this feature be deployed and monitored in production?

## Document Creation & Structure

### Specification Document (spec.md)
**Complete Section Coverage:**
```markdown
# [Feature Name]

## Overview
- Clear feature purpose and value proposition
- Target user audience and primary use cases
- Key benefits and competitive advantages

## Requirements
### Functional Requirements
- Specific user-facing capabilities and behaviors
- Input/output specifications and data handling
- Integration points with existing system components

### Non-Functional Requirements
- Performance benchmarks and scalability targets
- Security, privacy, and compliance considerations
- Usability, accessibility, and user experience standards

## Acceptance Criteria
- Measurable success criteria for feature completion
- User experience validation points and testing scenarios
- Technical validation requirements and quality gates

## Implementation Approach
- High-level technical strategy and architecture decisions
- Integration patterns and data flow design
- Development phases and milestone planning

## Dependencies
- Prerequisites and blocking requirements
- External service integrations and API dependencies
- Cross-team coordination needs and timelines
```

### Implementation Checklist (task-list.md)
**Complete Development Lifecycle:**
```markdown
## Discovery & Planning Phase
- [ ] Detailed technical design and architecture review
- [ ] API specification and interface design
- [ ] Database schema changes and migration planning
- [ ] Security and compliance review

## Development Phase
- [ ] Core feature implementation and unit testing
- [ ] Integration testing with existing system components
- [ ] User interface development and usability testing
- [ ] Performance optimization and load testing

## Quality Assurance Phase
- [ ] Comprehensive test suite development and execution
- [ ] Security testing and vulnerability assessment
- [ ] Accessibility testing and compliance validation
- [ ] Cross-browser/platform compatibility testing

## Deployment & Launch Phase
- [ ] Documentation updates and developer guide creation
- [ ] Production deployment and infrastructure setup
- [ ] User acceptance testing and feedback collection
- [ ] Monitoring and analytics implementation
```

### Design Notes (notes.md)
**Decision Context & Rationale:**
- Technical decision reasoning and alternative approaches considered
- Risk assessment and mitigation strategies
- Dependencies on other features or external factors
- Future extension opportunities and evolution plans
- Team coordination needs and communication requirements

## Collaborative Review & Iteration

### Multi-Stage Review Process
**Initial Draft Assessment:**
- Present all three documents together for holistic evaluation
- Identify gaps, inconsistencies, or areas requiring additional detail
- Validate technical feasibility and alignment with project constraints
- Ensure comprehensive coverage of requirements and implementation approach

**Refinement & Enhancement:**
- Address feedback on technical approach, user experience, and business value
- Expand sections with concrete examples, code snippets, or mockups where helpful
- Adjust task granularity and sequencing for optimal development workflow
- Update notes with additional context, constraints, or design considerations

### Quality Validation Standards
**Content Completeness:**
- [ ] All specification sections present with sufficient detail for implementation
- [ ] Task list covers complete development lifecycle from planning to deployment
- [ ] Notes provide adequate context for future developers and decision-making
- [ ] Content is specific enough to guide implementation without ambiguity

**Technical Accuracy:**
- [ ] Implementation approach aligns with project tech stack and architecture patterns
- [ ] Dependencies are accurately identified and coordination needs are clear
- [ ] Performance and security considerations are appropriately addressed
- [ ] Testing strategy covers functional, integration, and non-functional requirements

## MCP Integration & Execution

**Create Specification with Complete Content:**
```json
{"name":"create_spec","arguments":{"project_name":"$1","feature_name":"$2","spec":"<final_spec>","tasks":"<final_tasks>","notes":"<final_notes>"}}
```

## Error Recovery & Quality Assurance

**Content Development Challenges:**
- **Scope Definition Issues:** Help break down overly broad features into focused, manageable specifications
- **Technical Complexity:** Suggest phased implementation approach or simplified initial version
- **Requirement Ambiguity:** Use targeted questions to clarify specific behaviors and expectations
- **Integration Concerns:** Review existing specs and architecture to resolve dependency conflicts

**Validation & Refinement:**
- **Incomplete Specifications:** Guide user to expand missing sections with concrete details and examples
- **Task List Gaps:** Ensure complete development lifecycle coverage from planning through deployment
- **Technical Inconsistencies:** Validate alignment with project tech stack and architectural patterns
- **Content Quality:** Help user add specific examples, metrics, and measurable criteria

## Workflow Continuity & Development Planning

**After Successful Specification Creation:**
- **Implementation Initiation:** Begin development using `/foundry_update_spec` to track task completion progress
- **Context Verification:** Use `/foundry_load_spec` to review created specification and validate completeness
- **Project Portfolio:** Consider `/foundry_list_specs` to see new feature in context of overall development roadmap

**Long-term Development Strategy:**
- **Progress Tracking:** Mark tasks complete and add implementation insights as development proceeds
- **Requirement Evolution:** Update specification sections when scope or understanding changes
- **Knowledge Capture:** Document lessons learned, technical decisions, and optimization opportunities

## Instructions for Agent
**Primary Objective:** Conduct comprehensive feature discovery interview, collaboratively draft complete specification documents that meet all quality standards, then execute specification creation through Foundry MCP tools.

**Success Criteria:** Created specification should provide sufficient guidance for any development team to implement the feature successfully, with clear requirements, comprehensive task breakdown, and complete technical context.

**Quality Standard:** All documents should work together as cohesive implementation guide that reduces ambiguity, prevents scope creep, and enables efficient development workflow.
"###;

const CURSOR_UPDATE_SPEC_CMD: &str = r###"# Update Spec With Foundry

## Overview
Perform precise, idempotent updates to specification documents using intelligent edit command strategies.

## Context Gathering & Preparation

**Specification Loading & Validation:**
1. **Current State Assessment:** Load target specification using load_spec to understand current content and structure
2. **User Intent Analysis:** Understand specific changes requested and map to appropriate edit command patterns
3. **Content Validation:** Verify current spec content to ensure edit commands target exact text for reliable execution

**Parameter Resolution:**
- **Missing Project Name:** Use list_projects to guide user selection of correct project
- **Missing Spec Query:** Execute list_specs and present options for user to choose target specification
- **Update Requirements:** Clarify specific changes needed and validate against current spec structure

## Intelligent Edit Command Strategy

### Command Selection & Planning

**Task Management Operations:**
- **Mark Task Complete:** Use `set_task_status` with exact task text and "done" status
- **Reopen Task:** Use `set_task_status` with exact task text and "todo" status
- **Add New Task:** Use `upsert_task` with new task content in proper checkbox format
- **Modify Task:** Use `upsert_task` with updated task description (replaces existing)

**Content Addition Operations:**
- **Add Requirements:** Use `append_to_section` targeting "## Requirements" or specific subsection
- **Expand Implementation:** Use `append_to_section` targeting "## Implementation Approach"
- **Update Notes:** Use `append_to_section` targeting appropriate notes.md section
- **Extend Acceptance Criteria:** Use `append_to_section` targeting "## Acceptance Criteria"

**Critical Edit Rules:**
- **Task Files:** Never use `append_to_section` on task-list.md - only `upsert_task` and `set_task_status`
- **Spec/Notes Files:** Only use `append_to_section` for adding content to spec.md and notes.md
- **Exact Text Matching:** Selectors must match existing content precisely including whitespace

**Selector normalization & idempotence**:
- **Tasks selector normalization:** Matching ignores checkbox prefix, collapses internal whitespace, and ignores trailing periods. Either of these values works: `Implement OAuth2 integration` or `- [ ] Implement OAuth2 integration`.
- **Section headers:** Case-insensitive match on the full header string including hashes (surrounding whitespace trimmed). Use exact header text like `## Requirements`.
- **Idempotence:** Commands are repeat-safe; unchanged operations may be reported as skipped/idempotent.

## Comprehensive Update Workflow

### Step 1: Current State Analysis & Content Review
**Specification Loading:**
- Execute load_spec if target specification not already loaded in current session
- Review complete current content including spec.md, task-list.md, and notes.md
- Identify exact text for selectors to ensure reliable command targeting and execution

**Content Structure Assessment:**
- Understand current section organization and naming conventions
- Identify existing tasks and their current completion status
- Note areas where content additions would be most appropriate and valuable

### Step 2: Edit Command Planning & Validation
**Command Type Selection:**
Based on user intent, choose appropriate command patterns:

**Task Status Updates:**
```json
// Mark specific task as completed
{"target":"tasks","command":"set_task_status","selector":{"type":"task_text","value":"- [ ] Implement user authentication"},"status":"done"}

// Reopen completed task for additional work
{"target":"tasks","command":"set_task_status","selector":{"type":"task_text","value":"- [x] Setup database schema"},"status":"todo"}
```

**Task Content Management:**
```json
// Add new implementation task
{"target":"tasks","command":"upsert_task","selector":{"type":"task_text","value":"- [ ] Add password reset functionality"},"content":"- [ ] Add password reset functionality"}

// Update existing task with more specific details
{"target":"tasks","command":"upsert_task","selector":{"type":"task_text","value":"- [ ] Basic auth"},"content":"- [ ] Implement JWT-based authentication with refresh tokens"}
```

**Specification Content Expansion:**
```json
// Add new functional requirement
{"target":"spec","command":"append_to_section","selector":{"type":"section","value":"## Requirements"},"content":"- Support for two-factor authentication using TOTP"}

// Extend implementation approach with technical details
{"target":"spec","command":"append_to_section","selector":{"type":"section","value":"## Implementation Approach"},"content":"Authentication will use JWT tokens with 15-minute expiration and secure refresh mechanism"}
```

**Design Notes Enhancement:**
```json
// Add implementation insight or decision rationale
{"target":"notes","command":"append_to_section","selector":{"type":"section","value":"## Technical Decisions"},"content":"Chose JWT over sessions for better scalability in distributed environment"}
```

### Step 3: User Confirmation & Command Verification
**Edit Command Review:**
- Present proposed edit commands with clear description of intended changes
- Explain which files and sections will be modified and how content will be added/updated
- Confirm that edit commands align with user's intended modifications and development goals
- Verify selector text matches current specification content exactly for reliable execution

**Content Quality Validation:**
- Ensure new content follows specification formatting conventions and quality standards
- Validate that task additions use proper markdown checkbox format and clear descriptions
- Confirm that content additions enhance rather than duplicate existing information

### Step 4: Atomic Command Execution
**Execute Complete Update Operation:**
```json
{"name":"update_spec","arguments":{"project_name":"$1","spec_name":"$2","commands":[/* array of validated edit commands */]}}
```

Note: `commands` is required and must be a JSON array when using MCP tools. When using the CLI directly, pass the same array as a JSON string argument.

### Minimal Valid Example
```json
{"name":"update_spec","arguments":{"project_name":"$1","spec_name":"$2","commands":[
  {"target":"spec","command":"append_to_section","selector":{"type":"section","value":"## Overview"},"content":"New line"}
]}}
```

### Supported Operations
- Task Management: `set_task_status`, `upsert_task`
- Content Addition: `append_to_section`
- Content Removal: `remove_list_item`, `remove_from_section`, `remove_section`
- Content Replacement: `replace_list_item`, `replace_in_section`, `replace_section_content`

### Recommended Ordering
1) remove_list_item → 2) replace_in_section → 3) replace_section_content → 4) append_to_section

## Error Recovery & Problem Resolution

### Command Execution Issues
**Selector Targeting Failures:**
- **Text Not Found:** Re-load current specification content, copy exact text including whitespace, retry with precise selector
- **Ambiguous Matches:** Use longer text snippets with surrounding context for unique identification
- **Section Missing:** Guide user to add target section first or suggest alternative existing section
- **Candidate Suggestions:** On failure, the tool returns selector candidates with previews; copy one suggestion into your next attempt.

**Command Structure Problems:**
- **Invalid JSON Format:** Review command syntax and fix structural issues with proper escaping
- **Target Mismatches:** Ensure task commands use "tasks" target, spec/notes commands use appropriate targets
- **Content Format Issues:** Validate markdown syntax, checkbox format for tasks, and section header structure

### Content & Logic Validation
**Idempotency & Consistency:**
- **Duplicate Content Detection:** Check if similar content already exists before adding new information
- **Task Status Conflicts:** Verify current task completion status before attempting status changes
- **Logical Content Flow:** Ensure new content maintains logical progression and doesn't contradict existing information

**Quality Assurance:**
- **Content Completeness:** Verify that additions provide sufficient detail for implementation guidance
- **Technical Accuracy:** Ensure updates align with project tech stack and architectural patterns
- **User Value:** Confirm that changes enhance rather than complicate the specification

## Advanced Update Patterns & Batch Operations

### Complex Multi-Command Updates
**Development Progress Tracking:**
```json
[
  {"target":"tasks","command":"set_task_status","selector":{"type":"task_text","value":"- [x] Database schema design"},"status":"done"},
  {"target":"tasks","command":"upsert_task","selector":{"type":"task_text","value":"- [ ] API endpoint implementation"},"content":"- [ ] API endpoint implementation"},
  {"target":"notes","command":"append_to_section","selector":{"type":"section","value":"## Implementation Notes"},"content":"Database migration completed successfully, ready for API development"}
]
```

**Feature Scope Evolution:**
```json
[
  {"target":"spec","command":"append_to_section","selector":{"type":"section","value":"## Requirements"},"content":"- Integration with third-party payment processor"},
  {"target":"tasks","command":"upsert_task","selector":{"type":"task_text","value":"- [ ] Payment integration research"},"content":"- [ ] Payment integration research"},
  {"target":"notes","command":"append_to_section","selector":{"type":"section","value":"## External Dependencies"},"content":"Payment processor selection impacts security compliance requirements"}
]
```

## Workflow Continuity & Development Progression

### After Successful Updates
**Progress Assessment:**
- **Load Updated Spec:** Use `/foundry_load_spec` to review updated content and assess current development status
- **Related Spec Impact:** Consider whether updates affect other specifications or project components
- **Next Development Steps:** Identify logical next tasks or areas needing attention based on completed updates

**Quality Maintenance:**
- **Content Consistency:** Verify all changes maintain logical flow and technical accuracy across documents
- **Task Dependencies:** Ensure task updates reflect actual implementation progress and dependencies
- **Documentation Sync:** Keep specification, tasks, and notes aligned with current understanding and decisions

### Strategic Development Planning
**Implementation Workflow:**
- **Task Prioritization:** Use updated task list to guide development focus and resource allocation
- **Risk Management:** Monitor notes for implementation challenges and adjust planning accordingly
- **Feature Evolution:** Track requirement changes and ensure specification remains current with development reality

**Collaboration & Communication:**
- **Team Coordination:** Use updated specifications to communicate progress and next steps to stakeholders
- **Knowledge Sharing:** Ensure implementation insights captured in notes benefit future development work
- **Quality Standards:** Maintain specification quality to support continued development efficiency

## Instructions for Agent
**Primary Mission:** Load current specification content, analyze user's intended changes, map to precise edit commands, obtain user confirmation, then execute updates through Foundry MCP tools with full error recovery support.

**Execution Standards:** All edit commands must target exact existing text, use appropriate command types for content areas, and maintain specification quality and consistency throughout the update process.

**Success Criteria:** Updates accurately reflect user intent, maintain specification integrity, and provide clear development guidance while supporting continued iterative improvement of the feature specification.
"###;
