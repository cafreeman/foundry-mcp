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

## Deprecations

- Project.path and Spec.path are retained for backwards compatibility but are logically deprecated. Prefer location_hint and locator for UI and external references.
