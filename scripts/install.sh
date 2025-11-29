#!/bin/bash

# Get the absolute path of the codex-mcp-rs binary
# if current os is Darwin, use $(pwd)/codex-mcp-rs
if [ "$(uname)" == "Darwin" ]; then
    CODEX_MCP_RS_PATH=$(pwd)/codex-mcp-rs
fi
if [ ! -f "$CODEX_MCP_RS_PATH" ]; then
    CODEX_MCP_RS_PATH=$(pwd)/target/release/codex-mcp-rs
    if [ ! -f "$CODEX_MCP_RS_PATH" ]; then
        echo "Error: codex-mcp-rs binary not found"
        exit 1
    fi
fi

# Add the codex-mcp-rs server to the Claude Code MCP registry
CLAUDE_PATH=$(which claude)
if [ -f "$CLAUDE_PATH" ]; then
    "$CLAUDE_PATH" mcp add codex-rs -s user --transport stdio -- "$CODEX_MCP_RS_PATH"
else
    echo "Error: claude not found"
    exit 1
fi