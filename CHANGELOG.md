# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/cafreeman/foundry-mcp/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/cafreeman/foundry-mcp/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/cafreeman/foundry-mcp/releases/tag/v0.1.0
