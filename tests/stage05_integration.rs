//! Stage 05 Agent Integration & Hybrid Protection Test Suite.

use brahm::cli::{Cli, Commands, ConnectCommands, ProtectionCommands};
use brahm::config::BrahmConfig;
use brahm::deception::decoy::DecoyRegistry;
use brahm::deception::{ControlledInspectionEnvironment, InspectionSessionManager};
use brahm::integration::{
    AgentManifest, InspectionAgent, LocalTestAgentAdapter, ProtectionInstaller, ProtectionPlan,
    ProtectionProfile, RepositoryInputManager, RepositoryRecord, RollbackReport,
};
use clap::Parser;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_01_local_repo_registration_and_idempotency() {
    let fixture_a = Path::new("fixtures/fixture_a_react");
    let state_dir = tempdir().unwrap();
    let mgr = RepositoryInputManager::new();

    let first = mgr
        .connect_local(fixture_a, state_dir.path(), None)
        .unwrap();
    assert!(!first.already_registered);
    assert_eq!(first.record.active_profile, ProtectionProfile::AnalysisOnly);
    assert!(state_dir
        .path()
        .join("BRAHM_REPOSITORY_CONFIG.json")
        .is_file());

    // Reconnecting recognizes existing registration
    let second = mgr
        .connect_local(fixture_a, state_dir.path(), None)
        .unwrap();
    assert!(second.already_registered);
    assert_eq!(first.record.repository_id, second.record.repository_id);
    assert_eq!(
        first.record.baseline_fingerprint,
        second.record.baseline_fingerprint
    );
}

#[test]
fn test_02_github_url_validation_and_local_git_clone_simulation() {
    // Valid GitHub URLs
    let https =
        RepositoryInputManager::validate_github_url("https://github.com/DDharm007/brahm").unwrap();
    assert_eq!(https.owner, "DDharm007");
    assert_eq!(https.repo_name, "brahm");
    assert_eq!(
        https.sanitized_url,
        "https://github.com/DDharm007/brahm.git"
    );

    let ssh =
        RepositoryInputManager::validate_github_url("git@github.com:DDharm007/brahm.git").unwrap();
    assert_eq!(ssh.owner, "DDharm007");

    // Reject plaintext credential embedded in URL
    assert!(
        RepositoryInputManager::validate_github_url("https://ghp_secret123@github.com/a/b")
            .is_err()
    );

    // Reject shell injection metacharacters
    assert!(
        RepositoryInputManager::validate_github_url("https://github.com/a/b;rm -rf /").is_err()
    );

    // Verify local git clone helper works via local bare repository without internet
    let origin_dir = tempdir().unwrap();
    let seed_dir = tempdir().unwrap();
    fs::write(seed_dir.path().join("main.rs"), "pub fn main() {}\n").unwrap();

    let init_bare = Command::new("git")
        .args(["init", "--bare", "-b", "main"])
        .arg(origin_dir.path())
        .output()
        .unwrap();
    assert!(init_bare.status.success());

    // Initialize seed repo, commit, push to bare repo
    assert!(Command::new("git")
        .current_dir(seed_dir.path())
        .args(["init"])
        .output()
        .unwrap()
        .status
        .success());
    assert!(Command::new("git")
        .current_dir(seed_dir.path())
        .args(["config", "user.email", "test@example.invalid"])
        .output()
        .unwrap()
        .status
        .success());
    assert!(Command::new("git")
        .current_dir(seed_dir.path())
        .args(["config", "user.name", "BRAHM Test"])
        .output()
        .unwrap()
        .status
        .success());
    assert!(Command::new("git")
        .current_dir(seed_dir.path())
        .args(["add", "."])
        .output()
        .unwrap()
        .status
        .success());
    assert!(Command::new("git")
        .current_dir(seed_dir.path())
        .args(["commit", "-m", "init"])
        .output()
        .unwrap()
        .status
        .success());
    assert!(Command::new("git")
        .current_dir(seed_dir.path())
        .args(["push"])
        .arg(origin_dir.path())
        .arg("HEAD:main")
        .output()
        .unwrap()
        .status
        .success());

    let clone_parent = tempdir().unwrap();
    let clone_dest = clone_parent.path().join("cloned_repo");
    RepositoryInputManager::git_clone_readonly(&origin_dir.path().to_string_lossy(), &clone_dest)
        .expect("Local git clone must succeed");
    assert!(clone_dest.join("main.rs").is_file());
}

#[test]
fn test_03_protection_plan_requires_explicit_approval_and_installs_deliverables() {
    let repo_dir = tempdir().unwrap();
    fs::write(repo_dir.path().join("main.rs"), "pub fn main() {}\n").unwrap();
    let state_dir = tempdir().unwrap();

    let installer = ProtectionInstaller::new();

    // Pre-write a config file to verify backup creation on migration/install
    let initial_cfg = BrahmConfig::default();
    initial_cfg
        .save_to_path(state_dir.path().join("brahm.toml"))
        .unwrap();

    // Unapproved install MUST fail safely and write only the preview plan
    let denied = installer.install(
        repo_dir.path(),
        state_dir.path(),
        ProtectionProfile::ControlledInspection,
        false,
    );
    assert!(denied.is_err());
    assert!(state_dir
        .path()
        .join("BRAHM_PROTECTION_PLAN.json")
        .is_file());

    // Approved install completes workflow and writes all Stage 05 artifacts
    let outcome = installer
        .install(
            repo_dir.path(),
            state_dir.path(),
            ProtectionProfile::ControlledInspection,
            true,
        )
        .expect("Approved installation must succeed");

    assert_eq!(
        outcome.record.active_profile,
        ProtectionProfile::ControlledInspection
    );
    assert!(outcome.session.is_some());
    assert!(outcome.config_backup_path.unwrap().is_file());

    for file in [
        "BRAHM_PROTECTION_PLAN.json",
        "BRAHM_REPOSITORY_CONFIG.json",
        "BRAHM_AGENT_MANIFEST.json",
        "BRAHM_INSTALLATION_REPORT.md",
        "BRAHM_INSPECTION_GUIDE.md",
        "BRAHM_SECURITY.md",
        "BRAHM_ARCHITECTURE.md",
    ] {
        assert!(
            state_dir.path().join(file).is_file(),
            "Missing deliverable {}",
            file
        );
    }

    let _: ProtectionPlan = serde_json::from_str(
        &fs::read_to_string(state_dir.path().join("BRAHM_PROTECTION_PLAN.json")).unwrap(),
    )
    .unwrap();
    let _: RepositoryRecord = serde_json::from_str(
        &fs::read_to_string(state_dir.path().join("BRAHM_REPOSITORY_CONFIG.json")).unwrap(),
    )
    .unwrap();
    let _: AgentManifest = serde_json::from_str(
        &fs::read_to_string(state_dir.path().join("BRAHM_AGENT_MANIFEST.json")).unwrap(),
    )
    .unwrap();
}

#[test]
fn test_04_local_agent_adapter_sandbox_telemetry() {
    let repo_dir = tempdir().unwrap();
    fs::write(repo_dir.path().join("index.js"), "console.log('app');\n").unwrap();
    let state_dir = tempdir().unwrap();

    let installer = ProtectionInstaller::new();
    let outcome = installer
        .install(
            repo_dir.path(),
            state_dir.path(),
            ProtectionProfile::ControlledInspection,
            true,
        )
        .unwrap();

    let sess_mgr = InspectionSessionManager::new(state_dir.path().join("sessions"));
    let active_sess = sess_mgr
        .activate_session(&outcome.session.unwrap().session_id)
        .unwrap();
    let decoy_reg =
        DecoyRegistry::load_from_path(outcome.record.decoy_registry_path.unwrap()).unwrap();

    let mut sandbox = ControlledInspectionEnvironment::open(active_sess, decoy_reg).unwrap();
    let agent = LocalTestAgentAdapter::new();
    assert!(agent.get_capabilities().requires_isolated_sandbox);

    let targets = vec![
        "index.js".to_string(),
        "canary/BRAHM_CANARY_ANCHOR.md".to_string(),
        "../forbidden.txt".to_string(),
    ];
    let result = agent.run_inspection(&mut sandbox, &targets).unwrap();

    assert_eq!(result.accessed_files.len(), 2);
    assert_eq!(result.blocked_attempts, vec!["../forbidden.txt"]);
    assert!(result
        .telemetry_events
        .iter()
        .any(|e| e.canary_token.is_some() && e.allowed));
}

#[test]
fn test_05_rollback_dry_run_and_safe_execution() {
    let repo_dir = tempdir().unwrap();
    let genuine_file = repo_dir.path().join("app.py");
    fs::write(&genuine_file, "print('genuine')\n").unwrap();

    let state_dir = tempdir().unwrap();
    let installer = ProtectionInstaller::new();
    installer
        .install(
            repo_dir.path(),
            state_dir.path(),
            ProtectionProfile::ResearchMode,
            true,
        )
        .unwrap();

    // Plant an unrelated user note in state_dir/decoys/decoy_layer -> must survive rollback
    let user_note = state_dir.path().join("decoys/decoy_layer/manual_note.txt");
    fs::write(&user_note, "keep me").unwrap();

    // Dry-run rollback
    let dry: RollbackReport = installer.rollback(state_dir.path(), true).unwrap();
    assert!(dry.dry_run);
    assert!(!dry.removed_artifacts.is_empty());
    assert!(dry.integrity_verified_after);

    // Executed rollback
    let exec = installer.rollback(state_dir.path(), false).unwrap();
    assert!(!exec.dry_run);
    assert!(exec.integrity_verified_after);
    assert!(user_note.is_file());
    assert!(genuine_file.is_file());
    assert!(state_dir
        .path()
        .join("BRAHM_ROLLBACK_REPORT.json")
        .is_file());

    let (rec, valid, _) = installer.validate_installation(state_dir.path()).unwrap();
    assert!(valid);
    assert_eq!(rec.active_profile, ProtectionProfile::AnalysisOnly);
}

#[test]
fn test_06_cli_connect_and_protection_parsing() {
    let c1 = Cli::try_parse_from(["brahm", "connect", "local", "./repo"]).unwrap();
    assert!(matches!(
        c1.command,
        Commands::Connect {
            action: ConnectCommands::Local { .. }
        }
    ));

    let c2 = Cli::try_parse_from([
        "brahm",
        "connect",
        "github",
        "https://github.com/DDharm007/brahm",
    ])
    .unwrap();
    assert!(matches!(
        c2.command,
        Commands::Connect {
            action: ConnectCommands::Github { .. }
        }
    ));

    let c3 = Cli::try_parse_from([
        "brahm",
        "protection",
        "install",
        "./repo",
        "--profile",
        "controlled-inspection",
        "--approve",
    ])
    .unwrap();
    match c3.command {
        Commands::Protection {
            action: ProtectionCommands::Install { approve, .. },
        } => assert!(approve),
        _ => panic!("Expected protection install"),
    }
}
