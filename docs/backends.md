# Backend Architecture

This document describes the pluggable backend architecture for Foundry MCP, the core trait contracts, capabilities, invariants, and a checklist for implementing new backends.

## Overview

Foundry decouples domain logic (naming, validation, fuzzy matching, edit engine integration) from persistence using a façade (Foundry<B>) and a storage trait (FoundryBackend). The default backend is FilesystemBackend, which preserves directory layout, atomic writes, and JSON response stability.

## Key Components

- Foundry<B: FoundryBackend>
  - Centralized domain logic (spec name generation, validation, fuzzy matching)
  - Thin delegation to the selected backend for I/O
  - Implements SpecContentStore to support EditEngine
- FoundryBackend trait
  - Async trait defining project/spec operations
  - Helper methods for metadata (latest, count)
  - Capabilities introspection
- BackendCapabilities
  - Flags describing guarantees and features (atomic replace, strong consistency, etc.)
- ResourceLocator
  - Opaque, backend-specific pointer for UI/deeplinks (e.g., filesystem path)
- SpecContentStore
  - Minimal read/write interface used by EditEngine to operate on spec files

## Trait Contract (summarized)

Backends must implement:

- create_project, project_exists, list_projects, load_project
- create_spec, list_specs, load_spec, update_spec_content, delete_spec
- get_latest_spec, count_specs
- capabilities() -> BackendCapabilities

Behavioral requirements:

- Atomic replace semantics for file updates (no partial writes)
- Sorting: newest-first based on created_at (RFC3339 strings)
- Stable JSON shapes on inputs/outputs (additional optional fields allowed)
- Idempotent updates for edit commands

## Invariants

- Timestamped spec names: YYYYMMDD_HHMMSS_feature_name
- Feature names are snake_case
- created_at timestamps are RFC3339 strings
- Directory structure (filesystem backend):
  ~/.foundry/<project>/
  vision.md, tech-stack.md, summary.md
  specs/<spec_name>/{spec.md, notes.md, task-list.md}

## Implementing a New Backend

1. Create a module under src/core/backends/<name>.rs
2. Implement FoundryBackend for your type
3. Map ResourceLocator appropriately
4. Ensure capabilities() reflect your guarantees
5. Validate atomic semantics for updates
6. Run contract tests (see tests in src/core/backends/tests.rs)
7. Verify façade-based EditEngine integration via SpecContentStore

## Extension Checklist

- [ ] Implement trait methods with equivalent semantics
- [ ] Ensure RFC3339 timestamps, newest-first sorting
- [ ] Provide atomic replacement for updates
- [ ] Pass contract tests and façade + EditEngine integration tests
- [ ] Provide locator/location_hint where applicable

## Linear Backend (Experimental)

The Linear backend integrates Foundry MCP with Linear's issue tracking system, enabling project and specification management through Linear's GraphQL API.

### Setup

#### Prerequisites

1. **Linear API Token**: Obtain a personal API token from Linear's settings
2. **Team Access**: Ensure your token has access to the target Linear team
3. **Feature Flag**: Enable the `linear_backend` feature flag

#### Environment Variables

```bash
# Required
export LINEAR_API_TOKEN="lin_api_..."

# Optional team resolution (one of the following)
export LINEAR_TEAM_ID="team-id-123"           # Direct team ID
export LINEAR_TEAM_KEY="TEAM"                 # Team key (e.g., "ENG")
export LINEAR_TEAM_NAME="Engineering"         # Team name

# Optional configuration
export LINEAR_GRAPHQL_ENDPOINT="https://api.linear.app/graphql"  # Default endpoint
export LINEAR_HTTP_TIMEOUT_SECS="30"          # Request timeout
```

#### Feature Flag

The Linear backend is gated behind the `linear_backend` feature flag:

```bash
# Enable Linear backend
cargo build --features linear_backend
cargo test --features linear_backend
```

### Architecture

The Linear backend maps Foundry concepts to Linear entities:

- **Projects** → Linear Projects with Vision/Tech Stack documents
- **Specifications** → Linear Issues with "foundry" label
- **Tasks** → Linear Sub-issues with task markers
- **Notes** → Linear Documents linked to spec issues

### Key Features

#### Project Management

- **Create**: Creates Linear project with Vision and Tech Stack documents
- **Load**: Retrieves project metadata and documents
- **List**: Enumerates projects accessible to the API token

#### Specification Management

- **Create**: Creates Linear issue with spec content and notes document
- **Load**: Retrieves spec content from issue description and sub-issues
- **List**: Queries issues with "foundry" label by project
- **Update**: Supports task list updates via sub-issue reconciliation
- **Delete**: Archives spec issue and closes all sub-issues

#### Task Reconciliation

- **Phase D Integration**: Updates task lists by creating/updating/closing sub-issues
- **Marker System**: Uses hidden HTML comments for spec and task identification
- **State Management**: Maps task completion to Linear issue states

### Resource Locator

Linear backend uses `ResourceLocator::Linear` with:

- `project_id`: Linear project identifier
- `issue_id`: Linear issue identifier for specs
- `notes_document_id`: Linear document identifier for notes
- `issue_url`: Direct link to Linear issue
- `notes_url`: Direct link to Linear document

### Capabilities

```rust
BackendCapabilities {
    supports_documents: true,      // Vision/Tech Stack as Linear documents
    supports_subtasks: true,       // Tasks as Linear sub-issues
    url_deeplinks: true,           // Direct links to Linear issues
    atomic_replace: false,         // Linear API doesn't guarantee atomicity
    strong_consistency: false,     // Eventual consistency model
}
```

### Error Handling

The Linear backend implements robust error handling:

- **Rate Limiting**: Automatic retry with exponential backoff
- **Server Errors**: Retryable with configurable limits
- **GraphQL Errors**: Non-retryable, returned immediately
- **Timeout Handling**: Configurable request timeouts

### Testing

Comprehensive test coverage includes:

- **Contract Tests**: Mock-backed tests for all CRUD operations
- **Resilience Tests**: Rate limiting, retry behavior, error handling
- **Integration Tests**: End-to-end workflows with real Linear API

Run tests with:

```bash
cargo test --features linear_backend
```

### Limitations

1. **API Rate Limits**: Subject to Linear's GraphQL API rate limits
2. **Team Dependencies**: Requires proper team access and configuration
3. **Schema Dependency**: Relies on Linear's GraphQL schema introspection
4. **Network Dependency**: Requires internet connectivity for all operations

### Future Enhancements

- **Batch Operations**: Optimize multiple operations in single requests
- **Webhook Integration**: Real-time updates via Linear webhooks
- **Advanced Filtering**: Enhanced spec listing with complex filters
- **Bulk Operations**: Mass spec creation and management

## Deprecations

- Project.path and Spec.path are retained for backwards compatibility but are logically deprecated. Prefer location_hint and locator for UI and external references.
