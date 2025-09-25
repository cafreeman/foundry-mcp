# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.0] - 2025-09-25

### Added

- **Comprehensive Content Management Operations**: Extended edit operations from 3 to 9 commands for full specification lifecycle management
  - **Content Removal Operations**: `remove_list_item`, `remove_from_section`, `remove_section` for cleanup and maintenance
  - **Content Replacement Operations**: `replace_list_item`, `replace_in_section`, `replace_section_content` for updates and migrations
  - **Enhanced Selector System**: Added `text_in_section` selector for precise text targeting within markdown sections
  - **Real-world Use Cases**: Supports backward compatibility cleanup, technology stack upgrades, and specification modernization
  - **Comprehensive Documentation**: All 9 operations documented across MCP schema, help system, and user guides
  - **Production Testing**: 185 tests including real-world scenarios (technology upgrades, requirement cleanup)
- **Documentation Clarifications**: update_spec docs now explicitly require a non-empty `commands` array; added minimal valid examples, full operations list, recommended ordering, and numbered-list guidance in installed templates (Cursor rules, Claude subagent, command docs) and README
- **Backend Abstraction System**: Complete pluggable backend architecture for extensible storage systems
  - **FoundryBackend Trait**: Async trait defining storage contracts for Projects and Specs with Send + Sync bounds
  - **BackendCapabilities**: Feature introspection system with flags for documents, subtasks, URL deeplinks, atomic replace, and strong consistency
  - **ResourceLocator Enum**: Backend-agnostic resource identification system (FilesystemPath variant implemented)
  - **Foundry<B> Façade**: Centralized domain logic layer providing storage-agnostic operations with timestamp generation, validation, and fuzzy matching
  - **FilesystemBackend**: Production implementation porting all I/O logic from core modules with preserved atomic write semantics
  - **SpecContentStore Abstraction**: Interface for EditEngine I/O operations via façade delegation
  - **get_default_foundry() Factory**: Backend instantiation function providing Foundry<FilesystemBackend> instances
- Backend Architecture Documentation
  - Added docs/backends.md with trait contracts, invariants, capabilities, and extension checklist
  - Added "Architecture: Backends" section to README.md linking to the new docs

### Changed

- **BREAKING**: **Core Architecture Refactoring**: Complete migration to backend abstraction system
- **Selector Ergonomics**: Improved numbered-list matching by normalizing numeric prefixes in edit engine; selectors that omit numbers can match when the remainder is unique (output formatting preserved)
- **EditSelector**: Added optional `section_context` to `TaskText` (serde-defaulted) for future section-scoped matching; currently documented in templates but not yet enforced by engine scope rules
  - **Core Modules**: Removed direct I/O logic from `core/project.rs` and `core/spec.rs`, delegating to backend via façade
  - **CLI Commands**: Updated all CLI commands to use `Foundry<FilesystemBackend>` instances instead of direct core calls
  - **Domain Logic Centralization**: Moved spec name generation, validation, and fuzzy matching logic to façade for consistency across backends
  - **Type System Updates**: Added optional `location_hint` and `locator` fields to Project and Spec types with `#[serde(skip_serializing_if = "Option::is_none")]`
  - **Async/Sync Bridge**: Implemented `futures::executor::block_on` for compatibility without runtime conflicts
- Edit Engine routing
  - Removed legacy EditEngine::apply_edit_commands that performed direct filesystem I/O
  - All edit operations now route exclusively through Foundry::apply_edit_commands (façade), which uses SpecContentStore and backend abstraction

### Technical Implementation

- **Zero Breaking Changes**: All 146 existing tests pass unchanged, maintaining complete backward compatibility
- **Incremental Migration Strategy**: Three-phase implementation (Abstraction → Filesystem Backend → Integration)
- **Atomic Operations Preserved**: Filesystem backend maintains existing atomic write semantics using temp + rename pattern
- **Domain Logic Separation**: Storage-agnostic business logic (timestamp generation, validation, fuzzy matching) centralized in façade
- **Module Restructuring**: New `core/backends/` module hierarchy with trait definitions and filesystem implementation
- **Type Safety**: Backend abstraction uses generics (`Foundry<B: FoundryBackend>`) for compile-time backend selection
- **Async Support**: Full async/await support in backend trait for future API-based backends while maintaining sync CLI compatibility

### Deprecated

- Documented deprecation of `path` fields on Project/Spec in favor of `location_hint` and `locator` in docs/backends.md and README.md

### In Progress

- **Backend Infrastructure Completion**: Final phase implementation items
  - MCP handler updates to use façade (CLI commands already updated)
  - Unit tests for FilesystemBackend coverage
  - Test-only InMemoryBackend for contract testing
  - Complete documentation updates (README.md backend architecture section)

### Enhanced

- **Comprehensive Documentation Update**: Complete documentation overhaul to reflect new content management capabilities
  - **MCP Tool Schema**: Updated `UpdateSpecArgs` to include all 9 operations in tool introspection for AI assistant discovery
  - **Help System**: Enhanced `get_foundry_help` with categorized operations (Addition, Removal, Replacement) and comprehensive examples
  - **Template Files**: Updated Claude subagent and Cursor rules templates to showcase full content management capabilities
  - **User Documentation**: Transformed README.md from "append-only" to "comprehensive content management" messaging
  - **Command Guides**: Enhanced `.cursor/commands/foundry_update_spec.md` with removal and replacement operation examples
  - **Installation Templates**: Updated embedded templates to mention 9 operations instead of original 3
  - **Workflow Patterns**: Added real-world scenarios for specification cleanup and technology migration workflows
- **Command Template Documentation**: Improved update_spec command templates with comprehensive edit commands guidance
  - Added explicit note that `commands` parameter is required (MCP array; CLI passes JSON string)
  - Documented selector normalization rules for tasks (ignores checkbox prefix, collapses whitespace, ignores trailing periods)
  - Documented numbered-list guidance and recommended command ordering for batch operations
  - Added section header matching guidance (case-insensitive, exact header text with hashes)
  - Documented idempotence behavior for all edit commands (repeat-safe operations)
  - Added guidance about selector candidate suggestions on command failures
  - Enhanced both Claude and Cursor command templates with consistent documentation

### Fixed

- **CLI Command Reference Consistency**: Updated all remaining references to old CLI commands to use MCP tool names
  - Updated help text in `list_projects.rs` to reference `mcp_foundry_*` tools instead of `foundry mcp *` commands
  - Updated documentation in `args.rs` to use MCP tool names in parameter descriptions
  - Updated error messages across all command files to reference MCP tools instead of CLI commands
  - Updated workflow hints and next steps to use consistent MCP tool naming
  - Updated test assertions to match MCP tool names
  - Ensures complete consistency with MCP-only architecture introduced in v0.6.0
  - Updated workflow hint in `src/core/ops/load_project.rs` to reference `mcp_foundry_get_foundry_help`
  - Updated guidance in `src/cli/commands/get_foundry_help.rs` to reference `mcp_foundry_validate_content`
  - Verified no remaining `foundry <tool>` references via repo-wide search

## [0.6.1] - 2025-09-19

### Enhanced

- **Command Template Modernization**: Updated all Claude Code and Cursor command templates with 2024-2025 best practices
  - **Enhanced Context Gathering**: Added systematic repository analysis and project discovery phases to all commands
  - **Comprehensive Error Recovery**: Implemented specific error handling patterns for MCP failures, validation issues, and user interaction challenges
  - **Workflow Continuity**: Added explicit guidance linking commands together for seamless development workflows
  - **Argument Hints**: Added `argument-hint` frontmatter to all Claude commands for better user guidance
  - **Specific Tool Permissions**: Updated `allowed-tools` to specify exact MCP tools (e.g., `mcp__foundry__create_project`) instead of generic `mcp__foundry`
  - **Advanced Analysis Features**: Enhanced Cursor templates with development insights, pattern recognition, and strategic guidance
  - **Quality Validation**: Added content completeness checks and quality validation throughout all workflows
  - **Professional Structure**: Organized all templates with clear phases, error patterns, and success criteria matching industry standards

## [0.6.0] - 2025-09-18

### Changed

- Command templates redesigned for Claude and Cursor
  - Cursor: no frontmatter; structured steps with explicit agent instructions (analyze/draft → confirm → call MCP)
  - Claude: frontmatter retained; removed argument-hints; same collaborative workflow
  - Simplified command names with `foundry_` prefix (e.g., `foundry_analyze_project`)
- Installers now install client-specific command variants
  - Cursor installs Cursor-style commands; Claude installs Claude-style commands
- **BREAKING**: Disabled CLI execution of MCP tools - MCP tools now only accessible via MCP server
  - Removed `Commands::Mcp` enum and all MCP CLI subcommands
  - CLI now focused on lifecycle operations: serve, install, uninstall, status
  - All error messages and workflow hints updated to use MCP JSON format instead of CLI commands
  - Removed "MCP vs CLI Tool Confusion" section from templates

### Removed

- Anti-instruction text (e.g., "do not create/modify .cursor/commands") from command bodies
- CLI execution of MCP tools (`foundry mcp ...` commands)
- Dead code: `McpCommands` enum, unused imports, unreachable code paths

### Added

- Clear "Instruction to Agent" sections in all commands directing LLM to gather info, collaborate with user, then invoke the appropriate Foundry MCP tool

## [0.5.1] - 2025-09-18

### Fixed

- MCP tool JSON schema compliance for strict validators (Cursor GPT-5)
  - Added explicit `items` definition for the `commands` array in `update_spec` input schema
  - Resolves GPT-5 model errors in Cursor when MCP servers are enabled

## [0.5.0] - 2025-09-17

### Changed

- BREAKING: Remove context-based patching across CLI/MCP and replace with deterministic edit_commands

  - Removed `src/core/context_patch.rs` and all related types and dependencies (including rapidfuzz)
  - `update_spec` now supports only `operation: "edit_commands"` with commands array
  - New types in `src/types/edit_commands.rs` and `EditCommandsResponsePayload`
  - New core executor `src/core/edit_engine.rs` (deterministic, idempotent semantics)
  - Updated MCP `get_foundry_help` to include `edit-commands` topic and examples
  - Updated templates (`claude_subagent.rs`, `cursor_rules.rs`) to show edit_commands usage
  - Updated CLI args and tool schemas to remove replace/append/context_patch pathways
  - Migration guidance: use set_task_status, upsert_task, append_to_section with task_text/section selectors

- **Performance & Standards Alignment**: Comprehensive codebase modernization
  - **Fuzzy Matching Optimization**: Replaced `rapidfuzz` with lightweight `strsim` for equivalent functionality with reduced dependencies
  - **Logging Standardization**: Replaced `eprintln!` in core modules with `tracing::{warn, error}` for consistent logging
  - **Timestamp Consistency**: Unified all `created_at` fields to RFC3339 format across project/spec listings
  - **Functional Programming**: Refactored edit_engine and other hotspots to use iterator-based patterns over manual loops
  - **Error Handling**: Removed unwrap/expect from non-test code, ensuring proper error propagation
  - **Response Minimization**: Added `#[serde(skip_serializing_if)]` attributes to optional fields across all response structs
  - **Test Modernization**: Migrated all tests to use `TestEnvironment` + `temp-env` pattern, removing `#[tokio::test]`

### Added

- **Comprehensive Test Coverage**: Added tests for fuzzy matching thresholds, RFC3339 timestamp validation, and logging hygiene
- **Edge Case Validation**: Tests for fuzzy matching with empty strings, case sensitivity, and boundary conditions

## [0.4.0] - 2025-09-08

### Added

- **Streamlined Spec Discovery System**: Complete intelligent spec discovery and loading capabilities
  - **Fuzzy Matching for Spec Loading**: Natural language queries like "auth" automatically match "user_authentication" specs
  - **Lightweight Spec Discovery**: New `list_specs` command for efficient spec discovery without full project context loading
  - **Intelligent Workflow Selection**: Updated guidance templates to recommend optimal workflows based on task type
  - **Performance Optimization**: ~90% reduction in data transfer for focused spec work compared to full project loading
  - **Enhanced LoadSpecResponse**: Added optional `MatchInfo` struct for fuzzy match feedback and confidence scoring
  - **Priority-Based Matching Algorithm**: 5-tier matching strategy (Exact → Feature → Substring → Fuzzy Feature → Fuzzy Name)
  - **Comprehensive Error Handling**: Clear disambiguation for multiple matches, helpful suggestions for no matches
  - **Backward Compatibility**: All existing exact match workflows continue working unchanged

### Changed

- **BREAKING**: Updated guidance templates to remove "ALWAYS load project first" mandates in favor of intelligent workflow selection
- **BREAKING**: Enhanced `load_spec` command with fuzzy matching capabilities while maintaining exact match behavior
- **BREAKING**: Updated CLI argument documentation to include fuzzy matching guidance and repo name inference patterns
- **BREAKING**: Revised workflow patterns in templates to promote efficient spec discovery and loading
- Enhanced `load_spec` command with natural language query support and match confidence feedback
- Updated Cursor rules template with new efficient workflow patterns and fuzzy matching guidance
- Updated Claude subagent template with intelligent workflow selection and performance optimization recommendations
- Improved CLI argument documentation with comprehensive fuzzy matching capability descriptions

### Fixed

- **MCP Interface Documentation**: Fixed critical interface mismatch between CLI syntax and MCP tool call format
  - Updated all templates (Claude Subagent and Cursor Rules) to show correct MCP tool call JSON format
  - Replaced CLI syntax (`mcp_foundry_update_spec --operation context_patch --context-patch '{...}'`) with proper MCP format (`{"name": "update_spec", "arguments": {...}}`)
  - Fixed tool names throughout documentation (removed `mcp_foundry_` prefix, now uses `update_spec`, `create_project`, etc.)
  - Updated all error messages and workflow hints to show MCP syntax instead of CLI commands
  - Fixed help content examples to use correct JSON parameter format
  - Updated test validations to check for correct tool names in generated templates
- **Context Operations Clarification**: Documented and clarified the three context patch operations
  - **Insert**: Adds content between `before_context` and `after_context` landmarks (for new requirements/tasks)
  - **Replace**: Replaces the last line of `before_context` with new content (for task completion, updates)
  - **Delete**: Removes the last line of `before_context` (for cleanup, obsolete items)
  - Created comprehensive `CONTEXT_OPERATIONS_GUIDE.md` with detailed examples and best practices
  - Clarified when to use each operation type and how context selection affects success rates
- **Workflow Inefficiency**: Eliminated contradictory guidance that forced LLMs to load full project context before spec work
- **Spec Discovery Gap**: Resolved issue where users couldn't efficiently discover specs using natural language
- **Performance Issues**: Addressed redundant data transfer in common "work on specific feature" scenarios
- **User Experience**: Improved workflow efficiency by matching natural language patterns to technical spec names

## [0.3.1] - 2025-09-04

### Fixed

- **Install/Uninstall Output Formatting**: Fixed newline escaping issue where `\n` characters were displayed literally instead of actual line breaks
  - Install and uninstall commands now display properly formatted human-readable output
  - JSON output mode continues to work correctly with `--json` flag
  - Both output modes maintain proper formatting and emoji display
- **Claude Code Path Detection**: Replaced hardcoded paths with robust cross-platform executable discovery
  - Added `which` crate dependency for proper PATH resolution across different operating systems
  - Improved `execute_claude_command` function to handle aliases and shell functions
  - Better error handling for "already exists" scenarios in MCP server registration
  - Enhanced compatibility across different user environments and installation methods
- **Installation Detection and Messaging**: Fixed installation detection issues and improved user feedback
  - **Claude Code**: Fixed detection failure where "already exists" responses were treated as errors
  - **Claude Code**: Modified `execute_claude_command` to return output regardless of exit status for proper error handling
  - **Claude Code**: Added `RegistrationResult` enum to distinguish between new and existing MCP server registrations
  - **Claude Code**: Improved messaging: "Registered" vs "already registered with Claude Code CLI"
  - **Cursor**: Added check for existing server configuration before adding/updating
  - **Cursor**: Improved messaging: "Added" vs "Updated existing Foundry MCP server in Cursor configuration"
  - Both environments now provide clear feedback about what actually happened during installation

## [0.3.0] - 2025-09-04

### Added

- **Context-Based Patching System**: Complete engine for precise markdown updates without full file replacement
  - Multi-algorithm fuzzy matching with RapidFuzz integration (Simple, Levenshtein, JaroWinkler, TokenSort, PartialRatio)
  - JSON interface with schema validation and configurable similarity thresholds
  - Section-aware matching for markdown hierarchy disambiguation
  - Advanced features: context caching, batch operations, conflict detection, rollback system
  - Smart suggestion engine for failed matches with similarity analysis
  - Performance monitoring with real-time metrics collection
  - Comprehensive help system with 'context-patching' topic and troubleshooting guidance
  - 70-90% reduction in content generation for targeted updates
- **Enhanced Purpose Communication System**: Complete 4-phase implementation
  - 'CONTEXT FOR FUTURE IMPLEMENTATION' parameter enhancement across all document creation
  - Cold Start Test principle integration ensuring documents serve as complete implementation contexts
  - Enhanced validation system with context completeness scoring
  - Template system enhancements for improved document quality
- **Embedded Template Installation System**: Production-ready template infrastructure
  - ClientTemplate trait for extensible template system
  - Cursor rules template (.cursor/rules/foundry.mdc) with context-patching guidance
  - Claude subagent template (~/.claude/agents/foundry-mcp-agent.md) with intelligent defaults
  - Automatic template installation/removal integrated with install/uninstall commands
  - Comprehensive edge case testing and error handling
- **Testing Infrastructure Improvements**: Modern test organization and coverage
  - Reorganized installation tests into focused files (install_cursor_tests.rs, etc.)
  - Added comprehensive Claude Code uninstall test coverage (previously missing)
  - Enhanced TestEnvironment utilities with PATH isolation support
  - Test-only algorithm API for context-patching isolation testing
- Comprehensive MCP tool improvements with enhanced user experience patterns
- Option-based guidance replacing directive language across all 9 MCP tools
- Content creation acknowledgment in tool responses to recognize AI assistant's role
- Workflow efficiency improvements with smart guidance for optimal tool selection
- Consistent user experience patterns across all tools with collaborative language
- Production-grade error handling with helpful suggestions for failed context matches
- Robust boundary condition support for file start/end insertions in context-based patching

### Changed

- **BREAKING**: All MCP tool descriptions enhanced with structural guidance and user experience improvements
- **BREAKING**: Response patterns updated to use "You can..." instead of directive language
- **BREAKING**: Workflow hints and next steps now provide option-based guidance
- **BREAKING**: Algorithm selection completely hidden from LLMs in context-based patching (smart cascading internal)
- Enhanced load_project tool with critical workflow efficiency improvements
- Improved create_spec and analyze_project tools with content creation acknowledgment
- Updated load_spec, delete_spec, and list_projects tools with consistent user experience patterns
- Enhanced update_spec, validate_content, and get_foundry_help tools with improved response patterns
- Improved context-based patching reliability for edge cases and boundary conditions
- Refactored MCP command structure to use consistent `mcp_foundry_` prefix
- Enhanced InstallArgs and UninstallArgs with JSON output format support

### Fixed

- **Critical Production Bugs**: 5 major context-based patching issues resolved through comprehensive TDD
  - Empty context boundary handling for file start/end insertions
  - Replace/Delete position calculation logic (was inserting instead of replacing)
  - Search range extension for insertion operations at file boundaries
  - Content change detection with exact matching for concurrent modifications
  - Test interference issues in extreme threshold scenarios
- Case sensitivity bug in context patching operation validation
- Eliminated all directive language ("Use when...") across all 9 MCP tools
- Improved user decision-making control with collaborative guidance patterns
- Enhanced workflow efficiency with smart tool selection guidance
- Testing artifacts and temporary code cleanup from development

## [0.2.0] - 2025-09-02

### Added

- Modern testing infrastructure with assert_fs and temp-env integration
- Enhanced installation error messages with actionable guidance
- Multi-file update capability for specifications
- Iterative spec workflow commands and enhanced LLM prompting
- User-driven decision support to prevent LLM autopilot behavior
- Codanna MCP integration for enhanced code exploration
- Console dependency for improved CLI user experience
- Declarative macro system for MCP tool definitions with validation constraints

### Changed

- Refactored command responses to remove unnecessary next steps and workflow hints
- Improved configuration path handling to support project-local directories
- Updated test suite to use async testing patterns with better isolation
- Applied comprehensive functional programming refactoring throughout codebase
- Enhanced CLI polish and user experience improvements
- Simplified MCP server startup to use 'foundry serve' command
- Replaced glob reexports with selective explicit reexports
- **BREAKING**: Replaced procedural macro crate with integrated declarative macros
- Refactored MCP tool definition system to use custom `impl_mcp_tool!` macro
- Consolidated macro functionality into main crate, eliminating separate dependency

### Fixed

- Installation command binary path issues
- Test cleanup and reliability improvements
- Outdated analyze_project implementation checklist
- Various code quality improvements and linting fixes
- Doctest compilation errors in macro documentation
- Clippy warnings in macro implementations

### Removed

- Separate `foundry-mcp-macros` procedural macro crate
- Complex procedural macro dependencies (syn, quote, proc-macro2)
- Workspace configuration for internal macro crate

## [0.1.0] - 2024-12-19

### Added

- Initial release of Foundry MCP
- CLI tool for deterministic project management
- MCP server implementation for AI coding assistant integration
- Project structure management with vision, tech-stack, and summary documents
- Specification management with timestamped directories
- Support for creating, loading, updating, and validating project specifications
- Comprehensive validation system for content requirements
- Installation support for Claude Code and Cursor IDE integration
- Internal procedural macros for tool generation (foundry-mcp-macros - not published)
- JSON response format for both CLI and MCP interfaces
- Workflow guidance and next-steps recommendations
- Complete test suite with integration and unit tests

### Technical Features

- Rust 2024 edition with functional programming patterns
- Structured directory management in ~/.foundry/
- MCP protocol compliance for AI assistant integration
- Comprehensive error handling with anyhow
- Async runtime support with tokio
- CLI argument parsing with clap
- JSON serialization with serde
- Workspace support for internal macro crate

[Unreleased]: https://github.com/cafreeman/foundry-mcp/compare/v0.7.0...HEAD
[0.7.0]: https://github.com/cafreeman/foundry-mcp/compare/v0.6.1...v0.7.0
[0.6.1]: https://github.com/cafreeman/foundry-mcp/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/cafreeman/foundry-mcp/compare/v0.5.1...v0.6.0
[0.5.1]: https://github.com/cafreeman/foundry-mcp/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/cafreeman/foundry-mcp/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/cafreeman/foundry-mcp/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/cafreeman/foundry-mcp/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/cafreeman/foundry-mcp/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/cafreeman/foundry-mcp/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/cafreeman/foundry-mcp/releases/tag/v0.1.0
