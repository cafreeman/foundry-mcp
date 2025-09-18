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
allowed-tools: mcp__foundry
description: Analyze an existing codebase and create Foundry project docs
---

Goal: Analyze the current codebase, draft complete artifacts (vision, tech_stack, summary), confirm with the user, then create the Foundry project context via MCP.

Plan:
- Discover repository context: languages, frameworks, build tools, service layout, deployment, and docs (README, package files, infra files).
- Draft the following based on analysis (no boilerplate):
  - vision (200+ chars): problem, users, value, roadmap priorities
  - tech_stack (150+ chars): languages, frameworks, infra, deployment with rationale
  - summary (100+ chars): 2–3 sentences capturing essence
- Show drafts to the user; ask clarifying questions to fill gaps and refine.
- When the user confirms, call MCP:
  {"name":"analyze_project","arguments":{"project_name":"<project_name>","vision":"<final_vision>","tech_stack":"<final_tech_stack>","summary":"<final_summary>"}}
- If content-length validation fails, revise with the user and retry.

Tool reference:
- analyze_project(project_name, vision, tech_stack, summary)
  - Creates ~/.foundry/PROJECT with vision.md, tech-stack.md, summary.md from LLM-provided content
  - Content minimums: vision ≥200 chars, tech_stack ≥150, summary ≥100
  - Returns next_steps and workflow_hints
"###;

const CLAUDE_CREATE_PROJECT_CMD: &str = r###"---
allowed-tools: mcp__foundry
description: Create a new Foundry project with complete context documents
---

Goal: Collaboratively define vision, tech_stack, and summary for a new project, then create the Foundry project via MCP.

Plan:
- Interview the user briefly about problem, users, constraints, and deployment preferences.
- Draft:
  - vision (200+ chars)
  - tech_stack (150+ chars) with clear rationale
  - summary (100+ chars)
- Present drafts; iterate with the user until approved.
- Call MCP:
  {"name":"create_project","arguments":{"project_name":"<project_name>","vision":"<final_vision>","tech_stack":"<final_tech_stack>","summary":"<final_summary>"}}
- If content validation fails, expand collaboratively and retry.

Tool reference:
- create_project(project_name, vision, tech_stack, summary)
  - Initializes new ~/.foundry/PROJECT with provided documents
  - Same content rules as analyze_project; no auto-summarization performed
"###;

const CLAUDE_LIST_SPECS_CMD: &str = r###"---
allowed-tools: mcp__foundry
description: List specs for a project with lightweight discovery
---

Goal: Discover and summarize existing specs to help the user pick one.

Plan:
- If $1 is empty, ask the user for project name or show list_projects and confirm.
- Call list_specs:
  {"name":"list_specs","arguments":{"project_name":"$1"}}
- Present a compact list (name, feature, date) and propose likely next steps (load_spec, create_spec).

Tool reference:
- list_specs(project_name)
  - Returns lightweight metadata for specs without loading full content
"###;

const CLAUDE_LOAD_SPEC_CMD: &str = r###"---
allowed-tools: mcp__foundry
description: Load a spec by fuzzy name with discovery fallback
---

Goal: Load the target spec using fuzzy match, resolve ambiguities with the user, then suggest next steps.

Plan:
- If $1 (project_name) is missing, ask or show list_projects to select.
- If $2 (spec_query) is missing, call list_specs and ask the user to choose.
- Call load_spec with fuzzy matching:
  {"name":"load_spec","arguments":{"project_name":"$1","spec_name":"$2"}}
- If ambiguous/low-confidence, show top matches and ask the user to confirm one.
- After loading, summarize incomplete tasks and suggest update_spec or append notes.

Tool reference:
- load_spec(project_name, spec_name)
  - Loads full spec.md, notes.md, task-list.md; supports fuzzy spec_name
  - Returns summary context and workflow hints
"###;

const CLAUDE_CREATE_SPEC_CMD: &str = r###"---
allowed-tools: mcp__foundry
description: Create a new spec with tasks and notes for a project
---

Goal: Collaboratively draft a feature spec, checklist, and notes, then create it via MCP.

Plan:
- Ask the user for the feature goal and constraints; scan existing specs to avoid duplicates and align structure.
- Draft:
  - spec.md (Overview, Requirements, Acceptance Criteria, Implementation Approach, Dependencies)
  - task-list.md with actionable checkboxes
  - notes.md capturing rationale/risks
- Review drafts with the user and refine.
- Call MCP:
  {"name":"create_spec","arguments":{"project_name":"$1","feature_name":"$2","spec":"<final_spec>","tasks":"<final_tasks>","notes":"<final_notes>"}}
- If validation fails, expand/refine and retry.

Tool reference:
- create_spec(project_name, feature_name, spec, tasks, notes)
  - Creates specs/YYYYMMDD_HHMMSS_feature_name/{spec.md, task-list.md, notes.md}
  - Use one feature per spec; provide actionable task list with checkboxes
"###;

const CLAUDE_UPDATE_SPEC_CMD: &str = r###"---
allowed-tools: mcp__foundry
description: Update a spec using Foundry edit_commands safely and idempotently
---

Goal: Apply targeted updates to a spec.

Rules:
- Use upsert_task to add tasks; set_task_status to toggle tasks; append_to_section for spec/notes.
- Do not use append on tasks; only spec/notes.
- If selector errors occur, load_spec then retry with exact text.

Plan:
1) Ensure correct spec is loaded (use load_spec with $1 and $2 if needed). Read current content.
2) Propose precise edit_commands based on the user’s intent (add task, complete task, add requirement/notes). Confirm with the user.
3) Issue a single update_spec call with a commands array. Examples:
   - Upsert a task:
     {"name":"update_spec","arguments":{"project_name":"$1","spec_name":"$2","commands":[
       {"target":"tasks","command":"upsert_task","selector":{"type":"task_text","value":"Add password validation"},"content":"- [ ] Add password validation"}
     ]}}
   - Append a requirement:
     {"name":"update_spec","arguments":{"project_name":"$1","spec_name":"$2","commands":[
       {"target":"spec","command":"append_to_section","selector":{"type":"section","value":"## Requirements"},"content":"- Two-factor authentication support"}
     ]}}

Tool reference:
- update_spec(project_name, spec_name, commands[])
  - Commands supported:
    - set_task_status: {target:"tasks", selector:{type:"task_text", value:"..."}, status:"done"|"todo"}
    - upsert_task: {target:"tasks", selector:{type:"task_text", value:"..."}, content:"- [ ] ..."}
    - append_to_section: {target:"spec"|"notes", selector:{type:"section", value:"## Header"}, content:"..."}
  - Idempotent, safe to re-run; if selector fails/ambiguous, load_spec, copy exact text, adjust selector, and retry
"###;

// Cursor-formatted (no frontmatter) variants

const CURSOR_ANALYZE_PROJECT_CMD: &str = r###"# Analyze Project With Foundry

## Overview
Analyze an existing codebase and create Foundry project documents using MCP tools.

## Steps
1. Discover repository context (languages, frameworks, build tools, deployment, docs) and infer tech stack with rationale.
2. Draft vision (200+ chars), tech_stack (150+ chars), and summary (100+ chars) from the repo; avoid boilerplate.
3. Show drafts to the user, ask clarifying questions, and refine collaboratively.
4. Call MCP with finalized drafts:
   {"name":"analyze_project","arguments":{"project_name":"$1","vision":"<final_vision>","tech_stack":"<final_tech_stack>","summary":"<final_summary>"}}
5. If validation fails, refine drafts with the user and retry.

## Instruction to Agent
- Gather repository context, draft and refine the artifacts with the user, then call the Foundry MCP "analyze_project" tool with the finalized vision, tech_stack, and summary for the chosen project.
"###;

const CURSOR_CREATE_PROJECT_CMD: &str = r###"# Create Foundry Project

## Overview
Create a new Foundry project from complete LLM-provided documents.

## Steps
1. Interview the user briefly about problem, users, constraints, and deployment preferences.
2. Draft vision (200+), tech_stack (150+ with rationale), and summary (100+).
3. Present drafts; iterate with the user until approved.
4. Call MCP:
   {"name":"create_project","arguments":{"project_name":"$1","vision":"<final_vision>","tech_stack":"<final_tech_stack>","summary":"<final_summary>"}}
5. If content validation fails, expand collaboratively and retry.

## Instruction to Agent
- Interview, draft, and confirm with the user, then call the Foundry MCP "create_project" tool with the finalized vision, tech_stack, and summary for the chosen project.
"###;

const CURSOR_LIST_SPECS_CMD: &str = r###"# List Specs With Foundry

## Overview
List specs for a project without loading full content.

## Steps
1. If project_name missing, ask the user or list projects and let the user choose.
2. Call MCP:
   {"name":"list_specs","arguments":{"project_name":"$1"}}
3. Summarize results succinctly; suggest load_spec or create_spec.

## Instruction to Agent
- Determine the project_name if missing, then call the Foundry MCP "list_specs" tool and summarize results for the user.
"###;

const CURSOR_LOAD_SPEC_CMD: &str = r###"# Load Spec With Foundry

## Overview
Load a specification by fuzzy name for focused work.

## Steps
1. If project/spec query missing, help user with discovery (list_projects / list_specs).
2. Call MCP:
   {"name":"load_spec","arguments":{"project_name":"$1","spec_name":"$2"}}
3. If multiple/low-confidence matches, present candidates and ask for confirmation.
4. After loading, propose next steps based on incomplete tasks.

## Instruction to Agent
- Determine project_name and disambiguate spec_name as needed, then call the Foundry MCP "load_spec" tool and present actionable next steps.
"###;

const CURSOR_CREATE_SPEC_CMD: &str = r###"# Create Spec With Foundry

## Overview
Create a new feature spec with tasks and notes.

## Steps
1. Ask the user for the feature goal and constraints; scan existing specs to avoid duplicates.
2. Draft spec.md (Overview, Requirements, Acceptance Criteria, Implementation Approach, Dependencies), task-list.md (checkboxes), and notes.md.
3. Review drafts with the user; refine.
4. Call MCP:
   {"name":"create_spec","arguments":{"project_name":"$1","feature_name":"$2","spec":"<final_spec>","tasks":"<final_tasks>","notes":"<final_notes>"}}
5. If validation fails, expand/refine and retry.

## Instruction to Agent
- Draft spec, tasks, and notes collaboratively; upon approval, call the Foundry MCP "create_spec" tool with the finalized content.
"###;

const CURSOR_UPDATE_SPEC_CMD: &str = r###"# Update Spec With Foundry

## Overview
Perform idempotent updates to a spec using edit_commands.

## Arguments
- project_name (optional), spec_query (optional), instruction(s)

## Steps
1. Ensure correct spec is loaded (use load_spec if needed).
2. Choose appropriate commands:
   - Add task → upsert_task
   - Complete/reopen task → set_task_status
   - Add requirements/notes → append_to_section
3. Issue a single update_spec with a commands array. Example:
   {"name":"update_spec","arguments":{"project_name":"$1","spec_name":"$2","commands":[{"target":"spec","command":"append_to_section","selector":{"type":"section","value":"## Requirements"},"content":"- Two-factor authentication support"}]}}
4. If selector error, reload spec and retry with exact text.

## Instruction to Agent
- Read current content, propose precise edit_commands, confirm with the user, then call the Foundry MCP "update_spec" tool with the agreed commands.
"###;
