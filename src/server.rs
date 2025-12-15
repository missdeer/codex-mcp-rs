use crate::codex::{self, Options, SandboxPolicy, DEFAULT_TIMEOUT_SECS, MAX_TIMEOUT_SECS};
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    schemars, tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

mod serialize_as_os_string {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::path::{Path, PathBuf};

    #[allow(dead_code)]
    pub fn serialize<S>(path: &Path, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Use UTF-8 string representation for cross-platform compatibility
        match path.to_str() {
            Some(s) => s.serialize(serializer),
            None => Err(serde::ser::Error::custom("path contains invalid UTF-8")),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <String as Deserialize>::deserialize(deserializer)?;
        Ok(PathBuf::from(s))
    }
}

mod serialize_as_os_string_vec {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::path::PathBuf;

    #[allow(dead_code)]
    pub fn serialize<S>(paths: &Vec<PathBuf>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(paths.len()))?;
        for path in paths {
            match path.to_str() {
                Some(s) => seq.serialize_element(s)?,
                None => return Err(serde::ser::Error::custom("path contains invalid UTF-8")),
            }
        }
        seq.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<PathBuf>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec_strings = <Vec<String> as Deserialize>::deserialize(deserializer)?;
        Ok(vec_strings.into_iter().map(PathBuf::from).collect())
    }
}

/// Input parameters for codex tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CodexArgs {
    /// Instruction for task to send to codex
    #[serde(rename = "PROMPT")]
    pub prompt: String,
    /// Set the workspace root for codex before executing the task
    #[serde(
        serialize_with = "serialize_as_os_string::serialize",
        deserialize_with = "serialize_as_os_string::deserialize"
    )]
    pub cd: PathBuf,
    /// Sandbox policy for model-generated commands. Defaults to 'read-only'
    #[serde(default)]
    pub sandbox: SandboxPolicy,
    /// Resume the specified session of the codex. Defaults to None, start a new session
    #[serde(rename = "SESSION_ID", default)]
    pub session_id: Option<String>,
    /// Allow codex running outside a Git repository (useful for one-off directories)
    #[serde(default)]
    pub skip_git_repo_check: bool,
    /// Return all messages (e.g. reasoning, tool calls, etc.) from the codex session
    #[serde(default)]
    pub return_all_messages: bool,
    /// Maximum number of messages to keep when return_all_messages is true (default: 10000)
    #[serde(default)]
    pub return_all_messages_limit: Option<usize>,
    /// Attach one or more image files to the initial prompt
    #[serde(
        serialize_with = "serialize_as_os_string_vec::serialize",
        deserialize_with = "serialize_as_os_string_vec::deserialize"
    )]
    pub image: Vec<PathBuf>,
    /// The model to use for the codex session
    #[serde(default)]
    pub model: Option<String>,
    /// Run every command without approvals or sandboxing
    #[serde(default)]
    pub yolo: bool,
    /// Configuration profile name to load from '~/.codex/config.toml'
    #[serde(default)]
    pub profile: Option<String>,
    /// Timeout in seconds for codex execution. If not specified, uses CODEX_DEFAULT_TIMEOUT
    /// environment variable or falls back to 600 seconds (10 minutes). Max: 3600 seconds.
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

/// Result of parsing the default timeout from environment
struct DefaultTimeoutResult {
    value: u64,
    warning: Option<String>,
}

/// Get the default timeout, checking environment variable first.
/// Returns DEFAULT_TIMEOUT_SECS if env var is not set or invalid.
/// Clamps values to MAX_TIMEOUT_SECS if too large.
/// Returns any warning message for structured reporting.
fn get_default_timeout_with_warning() -> DefaultTimeoutResult {
    match std::env::var("CODEX_DEFAULT_TIMEOUT") {
        Ok(val) => {
            let trimmed = val.trim();
            // Treat empty string as "not set"
            if trimmed.is_empty() {
                return DefaultTimeoutResult {
                    value: DEFAULT_TIMEOUT_SECS,
                    warning: None,
                };
            }
            match trimmed.parse::<u64>() {
                Ok(0) => DefaultTimeoutResult {
                    value: DEFAULT_TIMEOUT_SECS,
                    warning: Some(format!(
                        "CODEX_DEFAULT_TIMEOUT=0 is invalid; using default of {} seconds",
                        DEFAULT_TIMEOUT_SECS
                    )),
                },
                Ok(secs) if secs > MAX_TIMEOUT_SECS => DefaultTimeoutResult {
                    value: MAX_TIMEOUT_SECS,
                    warning: Some(format!(
                        "CODEX_DEFAULT_TIMEOUT={} exceeds maximum of {} seconds; capping to maximum",
                        secs, MAX_TIMEOUT_SECS
                    )),
                },
                Ok(secs) => DefaultTimeoutResult {
                    value: secs,
                    warning: None,
                },
                Err(_) => DefaultTimeoutResult {
                    value: DEFAULT_TIMEOUT_SECS,
                    warning: Some(format!(
                        "CODEX_DEFAULT_TIMEOUT='{}' is not a valid number; using default of {} seconds",
                        trimmed, DEFAULT_TIMEOUT_SECS
                    )),
                },
            }
        }
        Err(std::env::VarError::NotUnicode(_)) => DefaultTimeoutResult {
            value: DEFAULT_TIMEOUT_SECS,
            warning: Some(format!(
                "CODEX_DEFAULT_TIMEOUT contains invalid UTF-8; using default of {} seconds",
                DEFAULT_TIMEOUT_SECS
            )),
        },
        Err(std::env::VarError::NotPresent) => DefaultTimeoutResult {
            value: DEFAULT_TIMEOUT_SECS,
            warning: None,
        },
    }
}

/// Security configuration for server-side restrictions
pub struct SecurityConfig {
    /// Allow dangerous sandbox modes
    pub allow_danger_full_access: bool,
    /// Allow yolo mode (bypasses approvals)
    pub allow_yolo: bool,
    /// Allow skipping git repo checks
    pub allow_skip_git_check: bool,
}

fn parse_env_bool(key: &str, warnings: &mut Vec<String>) -> Option<bool> {
    std::env::var(key).ok().and_then(|v| {
        let normalized = v.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "1" | "true" | "yes" | "y" | "on" | "t" | "enable" | "enabled" => Some(true),
            "0" | "false" | "no" | "n" | "off" | "f" | "disable" | "disabled" => Some(false),
            "" => None,
            _ => {
                warnings.push(format!(
                    "Environment variable {} has unrecognized boolean value '{}'; defaulting to disabled.",
                    key, v
                ));
                None
            }
        }
    })
}

/// Get security configuration from environment variables
fn get_security_config(warnings: &mut Vec<String>) -> SecurityConfig {
    SecurityConfig {
        allow_danger_full_access: parse_env_bool("CODEX_ALLOW_DANGEROUS", warnings)
            .unwrap_or(false),
        allow_yolo: parse_env_bool("CODEX_ALLOW_YOLO", warnings).unwrap_or(false),
        allow_skip_git_check: parse_env_bool("CODEX_ALLOW_SKIP_GIT_CHECK", warnings)
            .unwrap_or(false),
    }
}

fn merge_warnings(
    mut security_warnings: Vec<String>,
    result_warnings: Option<String>,
) -> Option<String> {
    if let Some(w) = result_warnings {
        security_warnings.push(w);
    }

    if security_warnings.is_empty() {
        None
    } else {
        Some(security_warnings.join("\n"))
    }
}

fn attach_warnings(mut error_msg: String, warnings: Option<String>) -> String {
    if let Some(w) = warnings {
        if !w.is_empty() {
            error_msg = format!("{error_msg}\nWarnings: {w}");
        }
    }
    error_msg
}

/// Output from the codex tool
#[derive(Debug, Serialize, schemars::JsonSchema)]
struct CodexOutput {
    success: bool,
    #[serde(rename = "SESSION_ID")]
    session_id: String,
    agent_messages: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    agent_messages_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    all_messages: Option<Vec<HashMap<String, Value>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    all_messages_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    warnings: Option<String>,
}

fn build_codex_output(
    result: &codex::CodexResult,
    return_all_messages: bool,
    warnings: Option<String>,
) -> CodexOutput {
    CodexOutput {
        success: result.success,
        session_id: result.session_id.clone(),
        agent_messages: result.agent_messages.clone(),
        agent_messages_truncated: result.agent_messages_truncated.then_some(true),
        all_messages: return_all_messages.then_some(result.all_messages.clone()),
        all_messages_truncated: (return_all_messages && result.all_messages_truncated)
            .then_some(true),
        error: result.error.clone(),
        warnings,
    }
}

#[derive(Clone)]
pub struct CodexServer {
    tool_router: ToolRouter<CodexServer>,
}

impl Default for CodexServer {
    fn default() -> Self {
        Self::new()
    }
}

impl CodexServer {
    /// Apply server-side security restrictions based on configuration
    /// Returns the modified args and any warning messages about security downgrades
    pub fn apply_security_restrictions(
        &self,
        mut args: CodexArgs,
        security: &SecurityConfig,
    ) -> (CodexArgs, Vec<String>) {
        let mut warnings = Vec::new();

        // Restrict dangerous sandbox mode unless explicitly allowed
        if !security.allow_danger_full_access && args.sandbox == SandboxPolicy::DangerFullAccess {
            warnings.push("Security warning: danger-full-access sandbox mode was downgraded to read-only. Set CODEX_ALLOW_DANGEROUS=true to enable.".to_string());
            args.sandbox = SandboxPolicy::ReadOnly;
        }

        // Restrict yolo mode unless explicitly allowed
        if !security.allow_yolo && args.yolo {
            warnings.push(
                "Security warning: yolo mode was disabled. Set CODEX_ALLOW_YOLO=true to enable."
                    .to_string(),
            );
            args.yolo = false;
        }

        // Restrict git repo skip unless explicitly allowed
        if !security.allow_skip_git_check && args.skip_git_repo_check {
            warnings.push("Security warning: skip_git_repo_check was disabled. Set CODEX_ALLOW_SKIP_GIT_CHECK=true to enable.".to_string());
            args.skip_git_repo_check = false;
        }

        (args, warnings)
    }

    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl CodexServer {
    /// Executes a non-interactive Codex session via CLI to perform AI-assisted coding tasks in a secure workspace.
    /// This tool wraps the 'codex exec' command, enabling model-driven code generation, debugging, or automation based on natural language prompts.
    /// It supports resuming ongoing sessions for continuity and enforces sandbox policies to prevent unsafe operations.
    #[tool(
        name = "codex",
        description = "Execute Codex CLI for AI-assisted coding tasks"
    )]
    async fn codex(
        &self,
        Parameters(args): Parameters<CodexArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Get security configuration
        let mut security_warnings = Vec::new();
        let security = get_security_config(&mut security_warnings);

        // Validate required parameters
        if args.prompt.is_empty() {
            return Err(McpError::invalid_params(
                "PROMPT is required and must be a non-empty string",
                None,
            ));
        }

        if args.cd.as_os_str().is_empty() {
            return Err(McpError::invalid_params(
                "cd is required and must be a non-empty string",
                None,
            ));
        }

        // Apply security restrictions
        let (mut args, restriction_warnings) = self.apply_security_restrictions(args, &security);
        security_warnings.extend(restriction_warnings);

        // Enforce timeout requirements: always set and within limits
        // Only parse env var when we actually need the default (None or Some(0))
        match args.timeout_secs {
            None => {
                // Always require a timeout to prevent unbounded execution
                let default_result = get_default_timeout_with_warning();
                args.timeout_secs = Some(default_result.value);
                if let Some(warning) = default_result.warning {
                    security_warnings.push(warning);
                }
            }
            Some(0) => {
                // Zero timeout is invalid, use default
                let default_result = get_default_timeout_with_warning();
                security_warnings.push(format!(
                    "Timeout of 0 seconds is invalid; using default of {} seconds",
                    default_result.value
                ));
                if let Some(warning) = default_result.warning {
                    security_warnings.push(warning);
                }
                args.timeout_secs = Some(default_result.value);
            }
            Some(timeout) if timeout > MAX_TIMEOUT_SECS => {
                security_warnings.push(format!(
                    "Timeout of {} seconds exceeds maximum of {} seconds; capping to maximum",
                    timeout, MAX_TIMEOUT_SECS
                ));
                args.timeout_secs = Some(MAX_TIMEOUT_SECS);
            }
            Some(_) => {
                // Valid timeout within range
            }
        }

        // Validate working directory exists and is a directory
        let working_dir = &args.cd;
        let canonical_working_dir = working_dir.canonicalize().map_err(|e| {
            McpError::invalid_params(
                format!(
                    "working directory does not exist or is not accessible: {} ({})",
                    working_dir.display(),
                    e
                ),
                None,
            )
        })?;

        if !canonical_working_dir.is_dir() {
            return Err(McpError::invalid_params(
                format!(
                    "working directory is not a directory: {}",
                    working_dir.display()
                ),
                None,
            ));
        }

        // Validate image files exist and are files
        let mut canonical_image_paths = Vec::new();
        for img_path in &args.image {
            // Resolve image path relative to working directory first, then canonicalize
            let resolved_path = if img_path.is_absolute() {
                img_path.clone()
            } else {
                // For relative paths, resolve against the working directory
                canonical_working_dir.join(img_path)
            };

            let canonical = resolved_path.canonicalize().map_err(|e| {
                McpError::invalid_params(
                    format!(
                        "image file does not exist or is not accessible: {} ({})",
                        resolved_path.display(),
                        e
                    ),
                    None,
                )
            })?;

            if !canonical.is_file() {
                return Err(McpError::invalid_params(
                    format!("image path is not a file: {}", resolved_path.display()),
                    None,
                ));
            }

            canonical_image_paths.push(canonical);
        }

        // Create options for codex client
        let opts = Options {
            prompt: args.prompt,
            working_dir: canonical_working_dir,
            sandbox: args.sandbox,
            session_id: args.session_id,
            skip_git_repo_check: args.skip_git_repo_check,
            return_all_messages: args.return_all_messages,
            return_all_messages_limit: args.return_all_messages_limit,
            image_paths: canonical_image_paths,
            model: args.model,
            yolo: args.yolo,
            profile: args.profile,
            timeout_secs: args.timeout_secs,
        };

        // Execute codex
        let result = match codex::run(opts).await {
            Ok(r) => r,
            Err(e) => {
                let warning_text = merge_warnings(security_warnings.clone(), None);
                let error_msg =
                    attach_warnings(format!("Failed to execute codex: {}", e), warning_text);
                return Err(McpError::internal_error(error_msg, None));
            }
        };

        let combined_warnings = merge_warnings(security_warnings.clone(), result.warnings.clone());

        // Prepare the response
        let output = build_codex_output(&result, args.return_all_messages, combined_warnings);

        let json_output = serde_json::to_string(&output).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize output: {}", e), None)
        })?;

        // Always return structured content so callers can inspect success, error, and warning fields.
        Ok(CallToolResult::success(vec![Content::text(json_output)]))
    }
}

#[tool_handler]
impl ServerHandler for CodexServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("This server provides a codex tool for AI-assisted coding tasks. Use the codex tool to execute coding tasks via the Codex CLI.".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn restore_env(key: &str, original: Option<String>) {
        if let Some(val) = original {
            std::env::set_var(key, val);
        } else {
            std::env::remove_var(key);
        }
    }

    #[test]
    fn parse_env_bool_accepts_numeric_and_text_values() {
        let key = "CODEX_TEST_BOOL_ACCEPT";
        let original = std::env::var(key).ok();

        std::env::set_var(key, "1");
        let mut warnings = Vec::new();
        assert_eq!(parse_env_bool(key, &mut warnings), Some(true));
        assert!(warnings.is_empty());

        std::env::set_var(key, "off");
        warnings.clear();
        assert_eq!(parse_env_bool(key, &mut warnings), Some(false));
        assert!(warnings.is_empty());

        restore_env(key, original);
    }

    #[test]
    fn parse_env_bool_warns_on_invalid_values() {
        let key = "CODEX_TEST_BOOL_INVALID";
        let original = std::env::var(key).ok();

        std::env::set_var(key, "maybe");
        let mut warnings = Vec::new();
        assert_eq!(parse_env_bool(key, &mut warnings), None);
        assert_eq!(warnings.len(), 1);

        restore_env(key, original);
    }

    #[test]
    fn merge_warnings_combines_security_and_result() {
        let combined = merge_warnings(vec!["security".into()], Some("result".into())).unwrap();
        assert!(combined.contains("security"));
        assert!(combined.contains("result"));
    }

    #[test]
    fn apply_security_restrictions_returns_warnings() {
        let server = CodexServer::new();
        let args = CodexArgs {
            prompt: "test".to_string(),
            cd: PathBuf::from("/tmp"),
            sandbox: SandboxPolicy::DangerFullAccess,
            session_id: None,
            skip_git_repo_check: true,
            return_all_messages: false,
            return_all_messages_limit: None,
            image: vec![],
            model: None,
            yolo: true,
            profile: None,
            timeout_secs: None,
        };
        let security = SecurityConfig {
            allow_danger_full_access: false,
            allow_yolo: false,
            allow_skip_git_check: false,
        };

        let (_updated, warnings) = server.apply_security_restrictions(args, &security);
        assert_eq!(warnings.len(), 3);
    }

    #[test]
    fn attach_warnings_appends_to_error_message() {
        let message = attach_warnings(
            "failure".to_string(),
            Some("warn-one\nwarn-two".to_string()),
        );
        assert!(message.contains("failure"));
        assert!(message.contains("Warnings: warn-one"));
        assert!(message.contains("warn-two"));
    }

    #[test]
    fn get_default_timeout_returns_default_when_env_not_set() {
        let key = "CODEX_DEFAULT_TIMEOUT";
        let original = std::env::var(key).ok();
        std::env::remove_var(key);

        let result = super::get_default_timeout_with_warning();
        assert_eq!(result.value, DEFAULT_TIMEOUT_SECS);
        assert!(result.warning.is_none());

        restore_env(key, original);
    }

    #[test]
    fn get_default_timeout_parses_valid_value() {
        let key = "CODEX_DEFAULT_TIMEOUT";
        let original = std::env::var(key).ok();
        std::env::set_var(key, "1800");

        let result = super::get_default_timeout_with_warning();
        assert_eq!(result.value, 1800);
        assert!(result.warning.is_none());

        restore_env(key, original);
    }

    #[test]
    fn get_default_timeout_trims_whitespace() {
        let key = "CODEX_DEFAULT_TIMEOUT";
        let original = std::env::var(key).ok();
        std::env::set_var(key, "  900  ");

        let result = super::get_default_timeout_with_warning();
        assert_eq!(result.value, 900);
        assert!(result.warning.is_none());

        restore_env(key, original);
    }

    #[test]
    fn get_default_timeout_treats_empty_as_unset() {
        let key = "CODEX_DEFAULT_TIMEOUT";
        let original = std::env::var(key).ok();
        std::env::set_var(key, "");

        let result = super::get_default_timeout_with_warning();
        assert_eq!(result.value, DEFAULT_TIMEOUT_SECS);
        assert!(result.warning.is_none());

        restore_env(key, original);
    }

    #[test]
    fn get_default_timeout_treats_whitespace_only_as_unset() {
        let key = "CODEX_DEFAULT_TIMEOUT";
        let original = std::env::var(key).ok();
        std::env::set_var(key, "   ");

        let result = super::get_default_timeout_with_warning();
        assert_eq!(result.value, DEFAULT_TIMEOUT_SECS);
        assert!(result.warning.is_none());

        restore_env(key, original);
    }

    #[test]
    fn get_default_timeout_caps_values_exceeding_max() {
        let key = "CODEX_DEFAULT_TIMEOUT";
        let original = std::env::var(key).ok();
        std::env::set_var(key, "9999");

        let result = super::get_default_timeout_with_warning();
        assert_eq!(result.value, MAX_TIMEOUT_SECS);
        assert!(result.warning.is_some());
        assert!(result.warning.unwrap().contains("exceeds maximum"));

        restore_env(key, original);
    }

    #[test]
    fn get_default_timeout_rejects_zero() {
        let key = "CODEX_DEFAULT_TIMEOUT";
        let original = std::env::var(key).ok();
        std::env::set_var(key, "0");

        let result = super::get_default_timeout_with_warning();
        assert_eq!(result.value, DEFAULT_TIMEOUT_SECS);
        assert!(result.warning.is_some());
        assert!(result.warning.unwrap().contains("invalid"));

        restore_env(key, original);
    }

    #[test]
    fn get_default_timeout_rejects_invalid_string() {
        let key = "CODEX_DEFAULT_TIMEOUT";
        let original = std::env::var(key).ok();
        std::env::set_var(key, "not-a-number");

        let result = super::get_default_timeout_with_warning();
        assert_eq!(result.value, DEFAULT_TIMEOUT_SECS);
        assert!(result.warning.is_some());
        assert!(result.warning.unwrap().contains("not a valid number"));

        restore_env(key, original);
    }
}
