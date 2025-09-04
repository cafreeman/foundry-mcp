# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Comprehensive MCP tool improvements with enhanced user experience patterns
- Option-based guidance replacing directive language across all 9 MCP tools
- Content creation acknowledgment in tool responses to recognize AI assistant's role
- Workflow efficiency improvements with smart guidance for optimal tool selection
- Consistent user experience patterns across all tools with collaborative language

### Changed

- **BREAKING**: All MCP tool descriptions enhanced with structural guidance and user experience improvements
- **BREAKING**: Response patterns updated to use "You can..." instead of directive language
- **BREAKING**: Workflow hints and next steps now provide option-based guidance
- Enhanced load_project tool with critical workflow efficiency improvements
- Improved create_spec and analyze_project tools with content creation acknowledgment
- Updated load_spec, delete_spec, and list_projects tools with consistent user experience patterns
- Enhanced update_spec, validate_content, and get_foundry_help tools with improved response patterns

### Fixed

- Eliminated all directive language ("Use when...") across all 9 MCP tools
- Improved user decision-making control with collaborative guidance patterns
- Enhanced workflow efficiency with smart tool selection guidance

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
