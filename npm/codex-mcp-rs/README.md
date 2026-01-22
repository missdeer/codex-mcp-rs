# @missdeer/codex-mcp-rs

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![MCP Compatible](https://img.shields.io/badge/MCP-Compatible-green.svg)](https://modelcontextprotocol.io)

NPM package for **codex-mcp-rs** - A high-performance Rust implementation of MCP (Model Context Protocol) server that wraps the Codex CLI.

## Quick Start

Run directly with npx - no installation required:

```bash
npx @missdeer/codex-mcp-rs
```

This automatically installs the correct binary for your platform and launches the MCP server.

## Installation

### Option 1: Use via npx (Recommended)

```bash
npx @missdeer/codex-mcp-rs
```

npx handles everything automatically:
1. Installs the platform-specific binary package
2. Launches the MCP server on stdio transport

Add to Claude Code:

```bash
claude mcp add codex-rs -s user --transport stdio -- npx @missdeer/codex-mcp-rs
```

### Option 2: Global Installation

```bash
npm install -g @missdeer/codex-mcp-rs
```

This installs the binary locally for faster startup on subsequent runs.

## Usage with Claude Code

After installation, add to your Claude Code MCP configuration:

```bash
claude mcp add codex-rs -s user --transport stdio -- codex-mcp-rs
```

Or manually add to your `~/.claude/settings.json`:

```json
{
  "mcpServers": {
    "codex-rs": {
      "command": "codex-mcp-rs",
      "transport": "stdio"
    }
  }
}
```

## Features

- ✨ High-performance Rust implementation
- 🚀 Low memory footprint
- 🔒 Configurable sandbox policies
- 🔄 Session management for multi-turn conversations
- 🖼️ Image attachment support
- ⚡ Fast async I/O with Tokio

## Supported Platforms

- Linux (x86_64, arm64)
- macOS (x86_64, arm64)
- Windows (x86_64, arm64)

## Prerequisites

You must have the [Codex CLI](https://github.com/openai/codex) installed and configured on your system.

## Tool Parameters

The server provides a `codex` tool with the following parameters:

- **PROMPT** (required): Task instruction
- **cd** (required): Working directory
- **sandbox**: Security policy (read-only, workspace-write, danger-full-access)
- **SESSION_ID**: Resume previous session
- **skip_git_repo_check**: Allow running outside git repos
- **return_all_messages**: Return full reasoning trace
- **image**: Attach image files
- **model**: Override Codex model
- **yolo**: Disable all prompts
- **profile**: Load config profile

## Documentation

For detailed documentation, see the [GitHub repository](https://github.com/missdeer/codex-mcp-rs).

## License

This project is dual-licensed:

### Non-Commercial / Personal Use - GNU General Public License v3.0

Free for personal projects, educational purposes, open source projects, and non-commercial use. See [LICENSE](https://github.com/missdeer/codex-mcp-rs/blob/master/LICENSE) for the full GPLv3 license text.

### Commercial / Workplace Use - Commercial License Required

**If you use codex-mcp-rs in a commercial environment, workplace, or for any commercial purpose, you must obtain a commercial license.**

This includes but is not limited to:
- Using the software at work (any organization)
- Integrating into commercial products or services
- Using for client work or consulting
- Offering as part of a SaaS/cloud service

**Contact**: missdeer@gmail.com for commercial licensing inquiries.

See [LICENSE-COMMERCIAL](https://github.com/missdeer/codex-mcp-rs/blob/master/LICENSE-COMMERCIAL) for more details.

## Related Projects

- [codexmcp](https://github.com/GuDaStudio/codexmcp) - Python implementation
- [codex-mcp-go](https://github.com/w31r4/codex-mcp-go) - Go implementation
- [geminimcp](https://github.com/GuDaStudio/geminimcp) - Gemini CLI MCP server
