# AGENTS.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Build & Development Commands

### Build
```bash
# Build entire workspace
cargo build

# Build specific crate
cargo build -p gearclaw_core
cargo build -p gearclaw_cli
cargo build -p gearclaw_gui
```

### Run
```bash
# Run CLI (default interactive mode)
cargo run -p gearclaw_cli

# Run single command
cargo run -p gearclaw_cli -- run "your prompt here"

# Run GUI (requires macOS 12.0+, Xcode Command Line Tools)
cargo run -p gearclaw_gui

# Generate sample config
cargo run -p gearclaw_cli -- config-sample

# Test MCP integration
cargo run -p gearclaw_cli -- test-mcp
```

### Test & Lint
```bash
# Run tests
cargo test

# Format code
cargo fmt

# Check linting
cargo clippy

# Update dependencies
cargo update
```

### Configuration
- Main config: `gearclaw.toml` or `~/.openclaw/gearclaw.toml`
- Sample config generation: `cargo run -p gearclaw_cli -- config-sample`

## Architecture Overview

GearClaw is a Rust-based AI assistant framework with a multi-crate workspace:

### Core Components
- **`gearclaw_core`**: Core library containing all business logic
  - LLM client with streaming support
  - Tool execution engine with security levels (deny/allowlist/full)
  - Session management and persistence
  - Memory management with embeddings
  - MCP (Model Context Protocol) client integration
  - Skills system for extending agent capabilities

- **`gearclaw_cli`**: Command-line interface
  - Interactive chat mode
  - Single command execution
  - Session and memory management commands

- **`gearclaw_gui`**: Desktop client using GPUI framework
  - Native macOS application
  - Requires Xcode Command Line Tools for Metal shader compilation

### Key Architecture Patterns

1. **Agent-Tool System**: The `Agent` struct orchestrates LLM calls, tool execution, and session management with up to 15 tool call loops to prevent infinite recursion.

2. **Security Model**: Three-tier security system for tool execution:
   - `deny`: No tools allowed
   - `allowlist`: Only safe commands (ls, cat, git, etc.)
   - `full`: Unrestricted execution

3. **MCP Integration**: Supports Model Context Protocol for external tool integration via child processes.

4. **Memory System**: Embedding-based memory with SQLite storage for context retrieval.

5. **Skills System**: Extensible skill loading from directory-based configuration.

### Configuration Structure
- **LLM**: Model selection, API endpoints, embedding models
- **Tools**: Security levels, execution profiles
- **Session**: Persistence, token limits, auto-save intervals
- **Agent**: System prompts, workspace paths, memory settings
- **MCP**: Server configurations with command/args/env

### File Organization
```
crates/
├── core/src/           # Core business logic
│   ├── agent.rs       # Main agent orchestration
│   ├── llm.rs         # LLM API client
│   ├── tools.rs       # Tool execution engine
│   ├── config.rs      # Configuration management
│   ├── session.rs     # Session persistence
│   ├── memory.rs      # Embedding-based memory
│   ├── mcp.rs         # MCP client integration
│   └── skills.rs      # Skills system
├── cli/src/           # CLI interface
└── gui/src/           # Desktop GUI (GPUI-based)
```

## Development Notes

- Uses tokio async runtime throughout
- Streaming LLM responses with tool call handling
- Configuration supports both TOML files and environment variables
- Chinese language prompts and messages (system is bilingual)
- Extensive error handling with custom `GearClawError` types
- Tracing-based logging with configurable levels

## Special Requirements

### GUI Development
- macOS 12.0+ required
- Xcode Command Line Tools mandatory for Metal shader compilation
- GPUI framework dependency from Zed editor project

### MCP Setup
- Filesystem server example in config points to Node.js-based MCP server
- Requires proper PATH configuration for Node.js/npm execution