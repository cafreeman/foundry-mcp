# Project Manager MCP - Installation Guide

This guide walks you through installing and configuring the Project Manager MCP for use as both a CLI tool and MCP server for AI coding assistants.

## 🆕 CLI Mode Available

Project Manager MCP now includes a comprehensive command-line interface in addition to its MCP server capabilities:

- **CLI Mode**: Direct project management, client configuration, and system administration
- **MCP Server Mode**: Traditional MCP server for AI assistant integration (default when no command provided)

Both modes are **100% backward compatible** with existing installations.

## Prerequisites

### Required
- **Rust 1.70+** - Install from [rustup.rs](https://rustup.rs/)
- **Git** - For cloning the repository

### Supported Platforms
- **macOS** (Intel and Apple Silicon)
- **Linux** (x86_64 and ARM64)
- **Windows** (x86_64)

## Installation Methods

### Method 1: Install from Source (Recommended)

1. **Clone the repository:**
   ```bash
   git clone https://github.com/your-org/project-manager-mcp.git
   cd project-manager-mcp
   ```

2. **Build and install:**
   ```bash
   cargo build --release
   cargo install --path .
   ```

3. **Verify installation:**
   ```bash
   # Check version
   project-manager-mcp --version
   
   # Test CLI mode
   project-manager-mcp --help
   
   # Test MCP server mode (Ctrl+C to exit)
   project-manager-mcp serve --verbose
   ```

### Method 2: Install from Crates.io

```bash
cargo install project-manager-mcp
```

### Method 3: Download Pre-built Binary

1. Visit the [releases page](https://github.com/your-org/project-manager-mcp/releases)
2. Download the binary for your platform
3. Extract and move to a directory in your PATH:
   ```bash
   # macOS/Linux
   sudo mv project-manager-mcp /usr/local/bin/
   
   # Windows
   # Move to a directory in your PATH
   ```

## Quick Setup with CLI

🚀 **New**: Use the built-in installation commands for automatic setup!

```bash
# Install for Cursor IDE
project-manager-mcp install --client cursor

# Install for Claude Desktop
project-manager-mcp install --client claude-desktop
```

This automatically configures the client with the correct binary path and optimal settings.

## Manual MCP Client Configuration

You can also manually configure MCP clients. The Project Manager MCP server works with any MCP-compatible AI coding assistant. Below are configuration examples for popular clients.

### Claude Desktop

Add this configuration to your Claude Desktop config file:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "project-manager": {
      "command": "project-manager-mcp",
      "args": [],
      "env": {
        "LOG_LEVEL": "info"
      }
    }
  }
}
```

### Codeium

Add to your Codeium MCP configuration:

```json
{
  "servers": {
    "project-manager": {
      "command": ["project-manager-mcp"],
      "env": {
        "LOG_LEVEL": "info"
      }
    }
  }
}
```

### Continue.dev

Add to your `config.json`:

```json
{
  "mcp": {
    "servers": [
      {
        "name": "project-manager",
        "command": "project-manager-mcp",
        "args": [],
        "env": {
          "LOG_LEVEL": "info"
        }
      }
    ]
  }
}
```

### Cursor

Add to your Cursor MCP settings:

```json
{
  "mcp.servers": {
    "project-manager": {
      "command": "project-manager-mcp",
      "args": [],
      "env": {
        "LOG_LEVEL": "info"
      }
    }
  }
}
```

## Environment Variables

Configure the server behavior using these environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `LOG_LEVEL` | `info` | Logging level (`error`, `warn`, `info`, `debug`, `trace`) |
| `PROJECT_MANAGER_BASE_DIR` | `~/.project-manager-mcp` | Base directory for project storage |
| `PROJECT_MANAGER_BACKUP_RETENTION_DAYS` | `7` | Days to keep file backups |

Example with custom configuration:

```json
{
  "mcpServers": {
    "project-manager": {
      "command": "project-manager-mcp",
      "args": [],
      "env": {
        "LOG_LEVEL": "debug",
        "PROJECT_MANAGER_BASE_DIR": "/path/to/projects",
        "PROJECT_MANAGER_BACKUP_RETENTION_DAYS": "14"
      }
    }
  }
}
```

## Verification

After installation and configuration:

1. **Restart your AI coding assistant**

2. **Test the connection** by asking your AI assistant:
   ```
   "Can you list the available MCP tools for project management?"
   ```

3. **Create a test project:**
   ```
   "Create a new project called 'test-project' with Rust and PostgreSQL"
   ```

4. **Check the file system:**
   ```bash
   ls ~/.project-manager-mcp/
   # Should show: test-project/
   ```

## File System Structure

The server creates this directory structure in your home directory:

```
~/.project-manager-mcp/
├── project-1/
│   ├── project/
│   │   ├── metadata.json      # Project metadata
│   │   ├── tech-stack.md      # Technology information
│   │   └── vision.md          # Project goals and vision
│   └── specs/
│       └── 20240115_feature_name/
│           ├── metadata.json  # Specification metadata
│           ├── spec.md        # Main specification content
│           ├── task-list.md   # Implementation tasks
│           └── notes.md       # Development notes
└── project-2/
    └── ...
```

## Troubleshooting

### Server Won't Start

1. **Check Rust installation:**
   ```bash
   rustc --version
   cargo --version
   ```

2. **Verify binary exists:**
   ```bash
   which project-manager-mcp
   ```

3. **Test manual execution:**
   ```bash
   # Test CLI mode
   project-manager-mcp --help
   
   # Test MCP server mode
   project-manager-mcp serve --verbose
   # Should show: "Starting Project Manager MCP Server..."
   ```

4. **Check logs** with enhanced CLI options:
   ```bash
   # Verbose logging
   project-manager-mcp serve --verbose --log-level debug
   
   # JSON format for monitoring
   project-manager-mcp serve --log-format json
   
   # Environment variable (legacy)
   LOG_LEVEL=debug project-manager-mcp
   ```

### Client Connection Issues

1. **Verify JSON syntax** in your MCP configuration
2. **Check file permissions** on the config file
3. **Restart your AI assistant** completely
4. **Test with minimal config:**
   ```json
   {
     "mcpServers": {
       "project-manager": {
         "command": "project-manager-mcp"
       }
     }
   }
   ```

### Permission Errors

1. **Check directory permissions:**
   ```bash
   ls -la ~/.project-manager-mcp/
   ```

2. **Fix ownership if needed:**
   ```bash
   sudo chown -R $USER:$USER ~/.project-manager-mcp/
   ```

3. **Set correct permissions:**
   ```bash
   chmod 755 ~/.project-manager-mcp/
   chmod -R 644 ~/.project-manager-mcp/*
   ```

### Performance Issues

1. **Check disk space:**
   ```bash
   df -h ~/.project-manager-mcp/
   ```

2. **Clean old backups:**
   ```bash
   find ~/.project-manager-mcp/ -name "*.backup.*" -mtime +7 -delete
   ```

3. **Reduce log level:**
   ```json
   "env": {
     "LOG_LEVEL": "warn"
   }
   ```

## Updating

### From Source
```bash
cd project-manager-mcp
git pull
cargo build --release
cargo install --path .
```

### From Crates.io
```bash
cargo install project-manager-mcp --force
```

### Pre-built Binary
Download the latest release and replace your existing binary.

## Uninstallation

1. **Remove the binary:**
   ```bash
   cargo uninstall project-manager-mcp
   # or manually: rm /usr/local/bin/project-manager-mcp
   ```

2. **Remove client configuration** from your AI assistant

3. **Optionally remove data** (projects will be lost):
   ```bash
   rm -rf ~/.project-manager-mcp/
   ```

## CLI Usage Examples

Once installed, explore the CLI capabilities:

```bash
# Get help for all commands
project-manager-mcp --help

# Start MCP server with custom settings
project-manager-mcp serve --port 3001 --log-format json --verbose

# Configure clients automatically  
project-manager-mcp install --client cursor
project-manager-mcp install --client claude-desktop

# Project management (upcoming in Phase 5)
project-manager-mcp list-projects
project-manager-mcp status --verbose
project-manager-mcp doctor --fix
```

## Getting Help

- **CLI Help**: `project-manager-mcp --help` or `project-manager-mcp <command> --help`
- **Issues**: [GitHub Issues](https://github.com/your-org/project-manager-mcp/issues)
- **Documentation**: [README.md](./README.md) - Comprehensive CLI and MCP usage guide
- **API Docs**: `cargo doc --open`

## Next Steps

Once installed, check out:
- [MCP Tools Documentation](./TOOLS.md) - Learn about available tools
- [Configuration Guide](./CONFIGURATION.md) - Advanced configuration options
- [Examples](./examples/) - Sample workflows and use cases