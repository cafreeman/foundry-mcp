# Project Manager MCP - Tools Documentation

This document describes all available MCP tools provided by the Project Manager server. These tools enable AI coding assistants to manage software projects, specifications, tasks, and development notes.

## Overview

The Project Manager MCP server provides four main tools:

| Tool                              | Purpose                | Use Case                          |
| --------------------------------- | ---------------------- | --------------------------------- |
| [`setup_project`](#setup_project) | Create new projects    | Starting a new software project   |
| [`create_spec`](#create_spec)     | Create specifications  | Defining features or requirements |
| [`load_spec`](#load_spec)         | Load project context   | Getting context for AI assistance |
| [`update_spec`](#update_spec)     | Update tasks and notes | Tracking implementation progress  |

## Tool Definitions

### setup_project

Creates a new software project with technology stack and business vision.

#### Parameters

| Parameter          | Type          | Required | Description                                             |
| ------------------ | ------------- | -------- | ------------------------------------------------------- |
| `name`             | string        | Yes      | Project name (alphanumeric, dashes, underscores only)   |
| `description`      | string        | Yes      | Brief project description                               |
| `languages`        | array[string] | Yes      | Programming languages (e.g., ["Rust", "TypeScript"])    |
| `frameworks`       | array[string] | No       | Frameworks and libraries (e.g., ["React", "Actix-Web"]) |
| `databases`        | array[string] | No       | Database systems (e.g., ["PostgreSQL", "Redis"])        |
| `tools`            | array[string] | No       | Development tools (e.g., ["Cargo", "npm", "Docker"])    |
| `deployment`       | array[string] | No       | Deployment platforms (e.g., ["AWS", "Vercel"])          |
| `overview`         | string        | Yes      | High-level project vision                               |
| `goals`            | array[string] | Yes      | Specific project goals                                  |
| `target_users`     | array[string] | Yes      | Intended user base                                      |
| `success_criteria` | array[string] | Yes      | Measurable success metrics                              |

#### Example Usage

```json
{
  "name": "setup_project",
  "arguments": {
    "name": "e-commerce-api",
    "description": "REST API for an e-commerce platform",
    "languages": ["Rust", "TypeScript"],
    "frameworks": ["Actix-Web", "React"],
    "databases": ["PostgreSQL", "Redis"],
    "tools": ["Cargo", "npm", "Docker"],
    "deployment": ["AWS ECS", "CloudFront"],
    "overview": "A high-performance e-commerce backend API",
    "goals": ["Handle 10,000 concurrent users", "Sub-100ms response times", "99.9% uptime"],
    "target_users": ["Online shoppers", "Store administrators", "Third-party integrators"],
    "success_criteria": ["Process 1M+ requests per day", "Zero data loss", "PCI DSS compliance"]
  }
}
```

#### Response

```json
{
  "content": [
    {
      "type": "text",
      "text": "✅ Project 'e-commerce-api' created successfully!\n\nProject Structure:\n- 📁 ~/.foundry/e-commerce-api/\n  - 📄 project/metadata.json\n  - 📄 project/tech-stack.md\n  - 📄 project/vision.md\n  - 📁 specs/ (ready for specifications)\n\nNext steps:\n1. Create your first specification with create_spec\n2. Define implementation tasks and requirements\n3. Use load_spec to get project context when coding"
    }
  ]
}
```

---

### create_spec

Creates a new specification for a feature, requirement, or component within a project.

#### Parameters

| Parameter      | Type   | Required | Description                               |
| -------------- | ------ | -------- | ----------------------------------------- |
| `project_name` | string | Yes      | Name of the existing project              |
| `spec_name`    | string | Yes      | Specification name (snake_case format)    |
| `title`        | string | Yes      | Human-readable specification title        |
| `description`  | string | Yes      | Brief description of what the spec covers |

#### Example Usage

```json
{
  "name": "create_spec",
  "arguments": {
    "project_name": "e-commerce-api",
    "spec_name": "user_authentication",
    "title": "User Authentication System",
    "description": "JWT-based authentication with OAuth2 support for social login"
  }
}
```

#### Response

```json
{
  "content": [
    {
      "type": "text",
      "text": "✅ Specification 'user_authentication' created successfully!\n\nSpecification ID: 20240115_user_authentication\nLocation: ~/.foundry/e-commerce-api/specs/20240115_user_authentication/\n\nFiles created:\n- 📄 metadata.json (specification metadata)\n- 📄 spec.md (main specification content)\n- 📄 task-list.md (implementation tasks)\n- 📄 notes.md (development notes)\n\nNext steps:\n1. Use update_spec to add implementation tasks\n2. Add development notes and decisions\n3. Use load_spec to get full context when implementing"
    }
  ]
}
```

---

### load_spec

Loads complete project and specification context for AI coding assistance.

#### Parameters

| Parameter      | Type   | Required | Description                                             |
| -------------- | ------ | -------- | ------------------------------------------------------- |
| `project_name` | string | Yes      | Name of the project                                     |
| `spec_id`      | string | Yes      | Specification ID (e.g., "20240115_user_authentication") |

#### Example Usage

```json
{
  "name": "load_spec",
  "arguments": {
    "project_name": "e-commerce-api",
    "spec_id": "20240115_user_authentication"
  }
}
```

#### Response

The response contains a comprehensive context document with:

1. **Project Information**

   - Technology stack details
   - Business vision and goals
   - Target users and success criteria

2. **Specification Content**

   - Current specification status
   - Detailed requirements and design
   - Implementation guidelines

3. **Task List**

   - Current implementation tasks
   - Task status and priorities
   - Dependencies between tasks

4. **Development Notes**
   - Implementation decisions
   - Technical observations
   - Questions and issues

```json
{
  "content": [
    {
      "type": "text",
      "text": "# Project Context: e-commerce-api\n\n## Technology Stack\n\n### Languages\n- Rust\n- TypeScript\n\n### Frameworks\n- Actix-Web\n- React\n\n[... full context continues ...]"
    }
  ]
}
```

---

### update_spec

Updates the task list and development notes for a specification.

#### Parameters

| Parameter      | Type          | Required | Description                         |
| -------------- | ------------- | -------- | ----------------------------------- |
| `project_name` | string        | Yes      | Name of the project                 |
| `spec_id`      | string        | Yes      | Specification ID                    |
| `tasks`        | array[object] | No       | Array of task objects to add/update |
| `notes`        | array[object] | No       | Array of note objects to add        |

#### Task Object Format

```json
{
  "id": "task_001",
  "title": "Implement JWT token generation",
  "description": "Create function to generate JWT tokens with proper claims",
  "status": "Todo",
  "priority": "High",
  "dependencies": ["task_database_setup"]
}
```

#### Note Object Format

```json
{
  "content": "Consider using RS256 algorithm for JWT signing",
  "category": "Implementation"
}
```

#### Example Usage

```json
{
  "name": "update_spec",
  "arguments": {
    "project_name": "e-commerce-api",
    "spec_id": "20240115_user_authentication",
    "tasks": [
      {
        "id": "task_001",
        "title": "Set up JWT dependencies",
        "description": "Add jsonwebtoken and related crates to Cargo.toml",
        "status": "Todo",
        "priority": "High",
        "dependencies": []
      },
      {
        "id": "task_002",
        "title": "Implement token generation",
        "description": "Create functions to generate access and refresh tokens",
        "status": "Todo",
        "priority": "High",
        "dependencies": ["task_001"]
      },
      {
        "id": "task_003",
        "title": "Add OAuth2 endpoints",
        "description": "Implement Google and GitHub OAuth2 integration",
        "status": "Todo",
        "priority": "Medium",
        "dependencies": ["task_002"]
      }
    ],
    "notes": [
      {
        "content": "Using RS256 algorithm for JWT signing for better security",
        "category": "Decision"
      },
      {
        "content": "Consider implementing token rotation for refresh tokens",
        "category": "Enhancement"
      },
      {
        "content": "Need to validate JWT expiration times in middleware",
        "category": "Implementation"
      }
    ]
  }
}
```

#### Response

```json
{
  "content": [
    {
      "type": "text",
      "text": "✅ Specification 'user_authentication' updated successfully!\n\nUpdates made:\n- Added 3 new tasks\n- Added 3 new notes\n- Updated task-list.md\n- Updated notes.md\n- Updated metadata timestamps\n\nCurrent status:\n- Total tasks: 3\n- Todo: 3, In Progress: 0, Completed: 0, Blocked: 0\n- High priority: 2, Medium priority: 1\n\nUse load_spec to get the updated context for implementation."
    }
  ]
}
```

## Task Status Values

| Status       | Description                             |
| ------------ | --------------------------------------- |
| `Todo`       | Task is ready to be started             |
| `InProgress` | Task is currently being worked on       |
| `Completed`  | Task has been finished                  |
| `Blocked`    | Task cannot proceed due to dependencies |

## Task Priority Values

| Priority   | Description                               |
| ---------- | ----------------------------------------- |
| `Low`      | Nice-to-have features, non-urgent         |
| `Medium`   | Standard development work                 |
| `High`     | Important features, should be prioritized |
| `Critical` | Urgent issues, bugs, blockers             |

## Note Categories

| Category         | Description                           |
| ---------------- | ------------------------------------- |
| `Implementation` | Technical details and code notes      |
| `Decision`       | Architectural decisions and rationale |
| `Question`       | Open questions needing clarification  |
| `Bug`            | Bug reports and issues                |
| `Enhancement`    | Ideas for improvements                |
| `Other`          | General notes                         |

## Best Practices

### Project Setup

- Use descriptive project names with clear scope
- Include all relevant technologies in the stack
- Write specific, measurable goals and success criteria
- Define clear target user personas

### Specification Creation

- Use snake_case for specification names
- Write clear, concise titles and descriptions
- Break large features into multiple specifications
- Start with high-level requirements before implementation details

### Task Management

- Create atomic, actionable tasks
- Set appropriate priorities based on business value
- Define clear dependencies between tasks
- Update task status regularly as work progresses

### Note Taking

- Categorize notes appropriately for easy filtering
- Document architectural decisions and rationale
- Capture questions early for later resolution
- Record implementation insights and observations

## Common Workflows

### Starting a New Project

1. **Create the project** with `setup_project`
2. **Define core specifications** for major features
3. **Break down specifications** into implementable tasks
4. **Begin implementation** using `load_spec` for context

### Feature Development

1. **Create a specification** for the feature
2. **Add initial tasks** with `update_spec`
3. **Load context** with `load_spec` before coding
4. **Update progress** and add notes as you work

### Project Maintenance

1. **Regular task updates** to track progress
2. **Documentation of decisions** in notes
3. **Creation of new specifications** as requirements evolve
4. **Context loading** to maintain continuity across sessions

## Error Handling

Common errors and their meanings:

- **Project not found**: The specified project doesn't exist
- **Specification not found**: The spec ID doesn't exist in the project
- **Invalid project name**: Project name contains invalid characters
- **Invalid spec name**: Specification name is not in snake_case format
- **Validation error**: Required fields are missing or invalid
- **File system error**: Permissions or disk space issues

All errors include helpful messages indicating how to resolve the issue.
