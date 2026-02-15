use codex_mcp_rs::codex::{Options, SandboxPolicy};
use std::path::PathBuf;

#[test]
fn test_options_validation() {
    // Test valid options
    let opts = Options {
        prompt: "Test prompt".to_string(),
        working_dir: PathBuf::from("/tmp"),
        sandbox: SandboxPolicy::ReadOnly,
        session_id: None,
        skip_git_repo_check: true,
        return_all_messages: false,
        return_all_messages_limit: None,
        image_paths: vec![],
        model: None,
        yolo: false,
        profile: None,
        timeout_secs: None,
        force_stdin: false,
    };

    assert!(!opts.prompt.is_empty());
    assert_eq!(opts.working_dir, PathBuf::from("/tmp"));
}

#[test]
fn test_sandbox_policies() {
    let policies = vec![
        SandboxPolicy::ReadOnly,
        SandboxPolicy::WorkspaceWrite,
        SandboxPolicy::DangerFullAccess,
    ];

    for policy in policies {
        let opts = Options {
            prompt: "test".to_string(),
            working_dir: PathBuf::from("/tmp"),
            sandbox: policy.clone(),
            session_id: None,
            skip_git_repo_check: true,
            return_all_messages: false,
            return_all_messages_limit: None,
            image_paths: vec![],
            model: None,
            yolo: false,
            profile: None,
            timeout_secs: None,
            force_stdin: false,
        };

        assert_eq!(opts.sandbox, policy);
    }
}

#[test]
fn test_sandbox_policy_strings() {
    assert_eq!(SandboxPolicy::ReadOnly.as_str(), "read-only");
    assert_eq!(SandboxPolicy::WorkspaceWrite.as_str(), "workspace-write");
    assert_eq!(
        SandboxPolicy::DangerFullAccess.as_str(),
        "danger-full-access"
    );
}

#[test]
fn test_image_paths() {
    let opts = Options {
        prompt: "Analyze this image".to_string(),
        working_dir: PathBuf::from("/tmp"),
        sandbox: SandboxPolicy::ReadOnly,
        session_id: None,
        skip_git_repo_check: true,
        return_all_messages: false,
        return_all_messages_limit: None,
        image_paths: vec![
            PathBuf::from("/path/to/image1.png"),
            PathBuf::from("/path/to/image2.jpg"),
        ],
        model: None,
        yolo: false,
        profile: None,
        timeout_secs: None,
        force_stdin: false,
    };

    assert_eq!(opts.image_paths.len(), 2);
    assert!(opts.image_paths[0].to_string_lossy().ends_with(".png"));
    assert!(opts.image_paths[1].to_string_lossy().ends_with(".jpg"));
}

#[test]
fn test_session_id_format() {
    let session_id = "550e8400-e29b-41d4-a716-446655440000";

    let opts = Options {
        prompt: "Continue task".to_string(),
        working_dir: PathBuf::from("/tmp"),
        sandbox: SandboxPolicy::ReadOnly,
        session_id: Some(session_id.to_string()),
        skip_git_repo_check: true,
        return_all_messages: false,
        return_all_messages_limit: None,
        image_paths: vec![],
        model: None,
        yolo: false,
        profile: None,
        timeout_secs: None,
        force_stdin: false,
    };

    assert!(opts.session_id.is_some());
    assert_eq!(opts.session_id.unwrap(), session_id);
}

#[test]
fn test_model_options() {
    let models = vec![
        "claude-3-opus-20240229",
        "claude-3-sonnet-20240229",
        "claude-3-haiku-20240307",
    ];

    for model in models {
        let opts = Options {
            prompt: "test".to_string(),
            working_dir: PathBuf::from("/tmp"),
            sandbox: SandboxPolicy::ReadOnly,
            session_id: None,
            skip_git_repo_check: true,
            return_all_messages: false,
            return_all_messages_limit: None,
            image_paths: vec![],
            model: Some(model.to_string()),
            yolo: false,
            profile: None,
            timeout_secs: None,
            force_stdin: false,
        };

        assert_eq!(opts.model, Some(model.to_string()));
    }
}

#[test]
fn test_return_all_messages_flag() {
    let opts_detailed = Options {
        prompt: "test".to_string(),
        working_dir: PathBuf::from("/tmp"),
        sandbox: SandboxPolicy::ReadOnly,
        session_id: None,
        skip_git_repo_check: true,
        return_all_messages: true,
        return_all_messages_limit: Some(5000),
        image_paths: vec![],
        model: None,
        yolo: false,
        profile: None,
        timeout_secs: None,
        force_stdin: false,
    };

    let opts_simple = Options {
        prompt: "test".to_string(),
        working_dir: PathBuf::from("/tmp"),
        sandbox: SandboxPolicy::ReadOnly,
        session_id: None,
        skip_git_repo_check: true,
        return_all_messages: false,
        return_all_messages_limit: None,
        image_paths: vec![],
        model: None,
        yolo: false,
        profile: None,
        timeout_secs: None,
        force_stdin: false,
    };

    assert!(opts_detailed.return_all_messages);
    assert_eq!(opts_detailed.return_all_messages_limit, Some(5000));
    assert!(!opts_simple.return_all_messages);
    assert_eq!(opts_simple.return_all_messages_limit, None);
}

#[test]
fn test_escape_prompt_integration() {
    // Removed since escape_prompt function was removed
    // Command::arg() handles platform-specific escaping automatically
    // This test is now empty as the functionality was removed
}

#[test]
fn test_working_directory_paths() {
    let paths = vec!["/tmp", "/home/user/project", ".", ".."];

    for path in paths {
        let opts = Options {
            prompt: "test".to_string(),
            working_dir: PathBuf::from(path),
            sandbox: SandboxPolicy::ReadOnly,
            session_id: None,
            skip_git_repo_check: true,
            return_all_messages: false,
            return_all_messages_limit: None,
            image_paths: vec![],
            model: None,
            yolo: false,
            profile: None,
            timeout_secs: None,
            force_stdin: false,
        };

        assert_eq!(opts.working_dir, PathBuf::from(path));
    }
}

#[test]
fn test_profile_configuration() {
    let profiles = vec!["default", "development", "production"];

    for profile in profiles {
        let opts = Options {
            prompt: "test".to_string(),
            working_dir: PathBuf::from("/tmp"),
            sandbox: SandboxPolicy::ReadOnly,
            session_id: None,
            skip_git_repo_check: true,
            return_all_messages: false,
            return_all_messages_limit: None,
            image_paths: vec![],
            model: None,
            yolo: false,
            profile: Some(profile.to_string()),
            timeout_secs: None,
            force_stdin: false,
        };

        assert_eq!(opts.profile, Some(profile.to_string()));
    }
}

#[test]
fn test_yolo_mode() {
    let opts_safe = Options {
        prompt: "test".to_string(),
        working_dir: PathBuf::from("/tmp"),
        sandbox: SandboxPolicy::ReadOnly,
        session_id: None,
        skip_git_repo_check: true,
        return_all_messages: false,
        return_all_messages_limit: None,
        image_paths: vec![],
        model: None,
        yolo: false,
        profile: None,
        timeout_secs: None,
        force_stdin: false,
    };

    let opts_yolo = Options {
        prompt: "test".to_string(),
        working_dir: PathBuf::from("/tmp"),
        sandbox: SandboxPolicy::DangerFullAccess,
        session_id: None,
        skip_git_repo_check: true,
        return_all_messages: false,
        return_all_messages_limit: None,
        image_paths: vec![],
        model: None,
        yolo: true,
        profile: None,
        timeout_secs: None,
        force_stdin: false,
    };

    assert!(!opts_safe.yolo);
    assert!(opts_yolo.yolo);
    assert_eq!(opts_yolo.sandbox, SandboxPolicy::DangerFullAccess);
}
