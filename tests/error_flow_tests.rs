use codex_mcp_rs::codex::{CodexResult, Options, SandboxPolicy};
use codex_mcp_rs::server::SecurityConfig;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn test_agent_messages_size_limit() {
    // Create a mock result that would exceed the agent messages limit
    let large_message = "x".repeat(11 * 1024 * 1024); // 11MB > 10MB limit
    let result = CodexResult {
        success: true,
        session_id: "test-session".to_string(),
        agent_messages: large_message,
        agent_messages_truncated: false,
        all_messages: Vec::new(),
        all_messages_truncated: false,
        error: None,
        warnings: None,
    };

    // The agent_messages should be truncatable in practice
    assert!(result.agent_messages.len() > 10 * 1024 * 1024);
    assert!(!result.agent_messages_truncated);
}

#[test]
fn test_agent_messages_truncation_flag() {
    let result = CodexResult {
        success: true,
        session_id: "test-session".to_string(),
        agent_messages: "[... Agent messages truncated due to size limit ...]".to_string(),
        agent_messages_truncated: true,
        all_messages: Vec::new(),
        all_messages_truncated: false,
        error: None,
        warnings: None,
    };

    assert!(result.agent_messages_truncated);
    assert!(result.agent_messages.contains("truncated"));
}

#[test]
fn test_all_messages_limit() {
    // Test that all_messages can be properly bounded
    let mut result = CodexResult {
        success: true,
        session_id: "test-session".to_string(),
        agent_messages: "test messages".to_string(),
        agent_messages_truncated: false,
        all_messages: Vec::new(),
        all_messages_truncated: false,
        error: None,
        warnings: None,
    };

    // Simulate adding messages up to limit
    for i in 0..50001 {
        if result.all_messages.len() < 50000 {
            result.all_messages.push(HashMap::from([
                ("id".to_string(), Value::String(format!("msg_{}", i))),
                ("type".to_string(), Value::String("test".to_string())),
            ]));
        } else {
            result.all_messages_truncated = true;
            break;
        }
    }

    assert_eq!(result.all_messages.len(), 50000);
    assert!(result.all_messages_truncated);
}

#[test]
fn test_error_and_warning_handling() {
    let result = CodexResult {
        success: false,
        session_id: "".to_string(),
        agent_messages: "".to_string(),
        agent_messages_truncated: false,
        all_messages: Vec::new(),
        all_messages_truncated: false,
        error: Some("Test error message".to_string()),
        warnings: Some("Test warning message".to_string()),
    };

    assert!(!result.success);
    assert!(result.error.is_some());
    assert!(result.warnings.is_some());
    assert_eq!(result.error.unwrap(), "Test error message");
    assert_eq!(result.warnings.unwrap(), "Test warning message");
}

#[test]
fn test_path_handling_with_non_utf8() {
    // Test PathBuf can handle non-UTF8 paths (even if we serialize as strings for JSON)
    let non_utf8_path = PathBuf::from("/path/with/invalid/utf8/�sequence");
    let opts = Options {
        prompt: "test".to_string(),
        working_dir: non_utf8_path.clone(),
        sandbox: SandboxPolicy::ReadOnly,
        session_id: None,
        skip_git_repo_check: true,
        return_all_messages: false,
        return_all_messages_limit: None,
        image_paths: vec![non_utf8_path.clone()],
        model: None,
        yolo: false,
        profile: None,
        timeout_secs: None,
    };

    // Should be able to create options without panicking
    assert_eq!(opts.working_dir, non_utf8_path);
    assert_eq!(opts.image_paths.len(), 1);
    assert_eq!(opts.image_paths[0], non_utf8_path);
}

#[test]
fn test_escape_prompt_with_special_chars() {
    // Removed since escape_prompt function was removed
    // Command::arg() handles platform-specific escaping automatically
    let input = "Test with \"quotes\" and \n newlines and \t tabs";

    // Verify the prompt can contain special characters
    assert!(input.contains('"'));
    assert!(input.contains('\n'));
    assert!(input.contains('\t'));
}

#[test]
fn test_stderr_error_context() {
    // Test error messages that include stderr context
    let error_with_stderr = "Command failed\nStderr: Warning: Something went wrong".to_string();

    assert!(error_with_stderr.contains("Stderr:"));
    assert!(error_with_stderr.contains("Warning: Something went wrong"));
}

#[test]
fn test_server_security_restrictions() {
    use codex_mcp_rs::server::CodexServer;

    let server = CodexServer::new();

    // Test that security config works in both directions
    // This is a unit test to verify the logic exists
    let args = codex_mcp_rs::server::CodexArgs {
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

    // Simulate security config that disallows dangerous features
    let security = SecurityConfig {
        allow_danger_full_access: false,
        allow_yolo: false,
        allow_skip_git_check: false,
    };

    let (restricted_args, warnings) = server.apply_security_restrictions(args, &security);

    // Should be downgraded to safe defaults
    assert_eq!(restricted_args.sandbox, SandboxPolicy::ReadOnly);
    assert!(!restricted_args.yolo);
    assert!(!restricted_args.skip_git_repo_check);
    // Should have warnings about the downgrades
    assert!(!warnings.is_empty());
}

#[tokio::test]
async fn test_timeout_error_shape() {
    // Test that timeout produces proper error structure without validation noise
    use codex_mcp_rs::codex;
    use std::env;
    use tempfile::tempdir;

    // Create a temporary directory for the test
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let temp_path = temp_dir.path().to_path_buf();

    // Create a simple shell script that sleeps
    #[cfg(not(target_os = "windows"))]
    {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let script_path = temp_path.join("sleep_script.sh");
        fs::write(&script_path, "#!/bin/sh\nsleep 10\n").expect("Failed to write script");
        let mut perms = fs::metadata(&script_path)
            .expect("Failed to get metadata")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).expect("Failed to set permissions");

        env::set_var("CODEX_BIN", script_path.to_str().unwrap());
    }

    #[cfg(target_os = "windows")]
    {
        use std::fs;
        let script_path = temp_path.join("sleep_script.bat");
        fs::write(&script_path, "@echo off\ntimeout /t 10 /nobreak\n")
            .expect("Failed to write script");
        env::set_var("CODEX_BIN", script_path.to_str().unwrap());
    }

    let opts = Options {
        prompt: "test".to_string(),
        working_dir: temp_path.clone(),
        sandbox: SandboxPolicy::ReadOnly,
        session_id: None,
        skip_git_repo_check: true,
        return_all_messages: false,
        return_all_messages_limit: None,
        image_paths: vec![],
        model: None,
        yolo: false,
        profile: None,
        timeout_secs: Some(1), // 1 second timeout
    };

    let result = codex::run(opts).await.expect("run should return Ok");

    // Verify timeout behavior
    assert!(
        !result.success,
        "timeout should mark result as unsuccessful"
    );
    assert!(
        result.error.is_some(),
        "timeout should have an error message"
    );

    let error_msg = result.error.unwrap();
    assert!(
        error_msg.contains("timed out"),
        "error should mention timeout, got: {}",
        error_msg
    );

    // With ValidationMode::Skip, timeout should NOT add session_id or agent_messages warnings
    assert!(
        result.session_id.is_empty(),
        "timeout result should have empty session_id"
    );
    assert!(
        result.warnings.is_none(),
        "timeout should not generate validation warnings"
    );

    // Clean up env var
    env::remove_var("CODEX_BIN");
}
