use anyhow::Result;
use clap::Parser;
use codex_mcp_rs::server::CodexServer;
use rmcp::{transport::stdio, ServiceExt};

/// MCP server wrapping the Codex CLI for AI-assisted coding tasks
#[derive(Parser)]
#[command(
    name = "codex-mcp-rs",
    version,
    about = "MCP server that provides AI-assisted coding through the Codex CLI",
    long_about = None,
    after_help = "ENVIRONMENT VARIABLES:
  CODEX_BIN                    Override the codex binary path (default: 'codex')
  CODEX_DEFAULT_TIMEOUT        Default timeout in seconds when not specified per-call
                               (fallback if unset: 600, max: 3600; values > 3600 are capped)
  CODEX_ALLOW_DANGEROUS        Allow danger-full-access sandbox mode (default: false)
                               Accepts: 1/true/yes/y/on/t/enable/enabled or
                               0/false/no/n/off/f/disable/disabled
  CODEX_ALLOW_YOLO             Allow yolo mode (bypasses all approvals) (default: false)
                               Accepts: 1/true/yes/y/on/t/enable/enabled or
                               0/false/no/n/off/f/disable/disabled
  CODEX_ALLOW_SKIP_GIT_CHECK   Allow running outside git repositories (default: false)
                               Accepts: 1/true/yes/y/on/t/enable/enabled or
                               0/false/no/n/off/f/disable/disabled

USAGE:
  This server communicates via stdio using the Model Context Protocol (MCP).
  It should be configured in your MCP client (e.g., Claude Desktop) settings.

  Example MCP client configuration:
    {
      \"mcpServers\": {
        \"codex\": {
          \"command\": \"/path/to/codex-mcp-rs\"
        }
      }
    }

SUPPORTED PARAMETERS:
  The 'codex' tool accepts the following parameters:

  PROMPT (required)            Task instruction to send to Codex
  cd (required)                Working directory for the Codex session
  sandbox                      Sandbox policy: read-only (default), workspace-write,
                               or danger-full-access
  SESSION_ID                   Resume an existing session (from previous response)
  skip_git_repo_check          Allow running outside git repos (default: false)
  return_all_messages          Return all messages including reasoning (default: false)
  return_all_messages_limit    Max messages to return when enabled (default: 10000)
  image                        Array of image file paths to attach to prompt
  model                        Model to use (overrides default)
  yolo                         Run without approval prompts (default: false)
  profile                      Config profile from ~/.codex/config.toml
  timeout_secs                 Timeout in seconds (default: CODEX_DEFAULT_TIMEOUT or 600, max: 3600)

AGENTS.MD SUPPORT:
  If an AGENTS.md file exists in the working directory, its content will be
  automatically prepended to the prompt as a system prompt. This allows you to
  define project-specific instructions or context for all Codex invocations.

SECURITY:
  - By default, dangerous operations are disabled
  - Set environment variables to enable them
  - Timeouts are enforced to prevent unbounded execution
  - Sandbox modes restrict file system access
  - See environment variables above for security controls

For more information, visit: https://github.com/missdeer/codex-mcp-rs"
)]
struct Cli {}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments (this will handle -h/--help and --version)
    let _cli = Cli::parse();

    // Create an instance of our codex server
    let service = CodexServer::new().serve(stdio()).await.inspect_err(|e| {
        eprintln!("serving error: {:?}", e);
    })?;

    service.waiting().await?;
    Ok(())
}
