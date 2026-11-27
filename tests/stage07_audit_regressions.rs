//! Regression Test Suite verifying all Phase 1 Audit Fixes (ISSUE-001 through ISSUE-007).

use brahm::analysis::{fingerprint_files, run_repository_analysis};
use brahm::config::BrahmConfig;
use brahm::deception::decoy::{DecoyEngine, DecoyGenerationProfile};
use brahm::integration::{ProtectionInstaller, ProtectionProfile};
use brahm::models::{FileClassification, FileEntry, FileType};
use brahm::monitoring::{
    build_monitoring_router, ApiState, EventStore, IntegrityMonitor, MonitoringApiConfig,
    SqliteEventStore,
};
use brahm::parser::{ParserRegistry, SupportedLanguage};
use brahm::structure::datamodels::DataModelDetector;
use brahm::structure::routes::ApiRouteDetector;
use brahm::structure::types::{DataModelKind, HttpMethod};
use std::collections::{HashSet, VecDeque};
use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tempfile::tempdir;

#[test]
fn test_issue_001_stored_baseline_detects_post_install_modification() {
    let repo_dir = tempdir().unwrap();
    let app_file = repo_dir.path().join("app.js");
    fs::write(&app_file, "const ok = true;\n").unwrap();

    let state_dir = tempdir().unwrap();
    let outcome = ProtectionInstaller::new()
        .install(
            repo_dir.path(),
            state_dir.path(),
            ProtectionProfile::ControlledInspection,
            true,
        )
        .unwrap();

    // Baseline file MUST have been persisted separately
    let baseline_path = state_dir.path().join("BRAHM_BASELINE.json");
    assert!(baseline_path.is_file());

    // Mutate genuine source AFTER installation
    fs::write(&app_file, "const ok = false; // tampered\n").unwrap();

    // Loading stored baseline vs live directory MUST catch the modification
    let stored: brahm::deception::GenuineLayer =
        serde_json::from_str(&fs::read_to_string(&baseline_path).unwrap()).unwrap();
    let (record, evts) = IntegrityMonitor::new()
        .check_integrity(
            &outcome.record.repository_id,
            repo_dir.path(),
            &stored,
            None,
            &HashSet::new(),
            "reg_001",
        )
        .unwrap();

    assert!(!record.baseline_matches);
    assert_eq!(record.unexpected_change_count, 1);
    assert_eq!(evts.len(), 1);
}

#[test]
fn test_issue_003_install_auto_persists_lifecycle_events_in_sqlite() {
    let repo_dir = tempdir().unwrap();
    fs::write(repo_dir.path().join("index.ts"), "export const x = 1;\n").unwrap();
    let state_dir = tempdir().unwrap();

    let outcome = ProtectionInstaller::new()
        .install(
            repo_dir.path(),
            state_dir.path(),
            ProtectionProfile::ControlledInspection,
            true,
        )
        .unwrap();

    let store = SqliteEventStore::open(state_dir.path().join("brahm_monitoring.db")).unwrap();
    let events = store
        .list_events(Some(&outcome.record.repository_id), None, 100)
        .unwrap();
    assert!(
        events.len() >= 2,
        "Expected DecoyCreated + InspectionSessionStarted persisted automatically"
    );
}

#[test]
fn test_issue_005_schema_sql_tables_and_middleware_express_handler() {
    let model_detector = DataModelDetector::new();
    let sql = r#"
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY,
    email TEXT NOT NULL UNIQUE
);

CREATE TABLE orders (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL
);
"#;
    // Non-migration file inside db/ must parse all CREATE TABLE schemas cleanly
    let models = model_detector.detect_from_text(&PathBuf::from("backend/db/schema.sql"), sql);
    assert_eq!(models.len(), 2);
    assert_eq!(models[0].kind, DataModelKind::SchemaDefinition);
    assert_eq!(models[0].name, "users");
    assert_eq!(models[1].name, "orders");

    // Middleware-wrapped Express route extracts final handler name
    let reg = ParserRegistry::new();
    let ts = r#"
router.post('/api/orders', requireAuth, validateOrder, createOrderHandler);
"#;
    let parsed = reg.parse_source(ts, SupportedLanguage::TypeScript).unwrap();
    let routes = ApiRouteDetector::new().detect_routes(&parsed);
    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].method, HttpMethod::Post);
    assert_eq!(routes[0].path_pattern, "/api/orders");
    assert_eq!(
        routes[0].handler_name.as_deref(),
        Some("createOrderHandler")
    );
}

#[test]
fn test_issue_007_cleanup_prunes_empty_decoy_subdirs() {
    let repo_dir = tempdir().unwrap();
    fs::write(repo_dir.path().join("lib.rs"), "pub fn run() {}\n").unwrap();
    let stage1 = run_repository_analysis(repo_dir.path(), BrahmConfig::default()).unwrap();
    let arch = brahm::architecture::ArchitectureEngine::new()
        .analyze(repo_dir.path(), &stage1)
        .unwrap();

    let stage_out = tempdir().unwrap();
    let mut res = DecoyEngine::new()
        .generate_and_stage(
            repo_dir.path(),
            &arch.report,
            DecoyGenerationProfile::Conservative,
            stage_out.path(),
        )
        .unwrap();

    assert!(res.decoy_layer_root.join("canary").is_dir());
    let cleanup = res
        .registry
        .cleanup_owned_artifacts(&res.decoy_layer_root, false)
        .unwrap();
    assert!(!cleanup.removed_paths.is_empty());
    // Empty `canary` subdir pruned safely while decoy_layer_root itself remains intact
    assert!(!res.decoy_layer_root.join("canary").exists());
    assert!(res.decoy_layer_root.is_dir());
}

#[test]
fn test_issue_002_hasher_skips_symlinks() {
    let repo_dir = tempdir().unwrap();
    fs::write(repo_dir.path().join("legit.rs"), "pub fn ok() {}\n").unwrap();

    let mut entries = vec![
        FileEntry {
            relative_path: "legit.rs".to_string(),
            filename: "legit.rs".to_string(),
            extension: Some("rs".to_string()),
            size_bytes: 15,
            file_type: FileType::File,
            is_binary: false,
            is_hidden: false,
            modified_at: None,
            language: Some("Rust".to_string()),
            classification: FileClassification::SourceCode,
            hash_blake3: None,
        },
        FileEntry {
            relative_path: "escape_link.txt".to_string(),
            filename: "escape_link.txt".to_string(),
            extension: Some("txt".to_string()),
            size_bytes: 0,
            file_type: FileType::Symlink,
            is_binary: false,
            is_hidden: false,
            modified_at: None,
            language: None,
            classification: FileClassification::Unknown,
            hash_blake3: None,
        },
    ];

    let mut warnings = Vec::new();
    fingerprint_files(repo_dir.path(), &mut entries, &mut warnings, 1024 * 1024);

    assert!(entries[0].hash_blake3.is_some());
    assert!(
        entries[1].hash_blake3.is_none(),
        "Symlink entries must never be opened or fingerprinted"
    );
    assert!(warnings.is_empty());
}

#[tokio::test]
async fn test_issue_006_monitoring_api_endpoints_and_localhost_cors() {
    let state_dir = tempdir().unwrap();
    fs::write(
        state_dir.path().join("BRAHM_ARCHITECTURE.json"),
        r#"{"schema_version":"0.2.0","modules":[]}"#,
    )
    .unwrap();
    fs::write(
        state_dir.path().join("BRAHM_RATING.json"),
        r#"{"total_score":80.0,"band":"Strong"}"#,
    )
    .unwrap();

    let store = SqliteEventStore::open_in_memory().unwrap();
    let token = "stage07_dashboard_api_token_987654".to_string();
    let cfg = MonitoringApiConfig {
        bind_addr: "127.0.0.1:0".parse().unwrap(),
        allow_remote: false,
        auth_token: token.clone(),
        rate_limit_per_minute: 60,
        state_dir: state_dir.path().to_path_buf(),
    };
    cfg.validate().unwrap();

    let state = ApiState {
        store,
        config: cfg,
        request_timestamps: Arc::new(Mutex::new(VecDeque::new())),
    };
    let app = build_monitoring_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    let req_with_origin = move |path: &str, origin: &str, bearer: Option<&str>| -> String {
        let mut stream = TcpStream::connect(addr).unwrap();
        let auth = bearer
            .map(|t| format!("Authorization: Bearer {}\r\n", t))
            .unwrap_or_default();
        let req = format!(
            "GET {} HTTP/1.1\r\nHost: 127.0.0.1\r\nOrigin: {}\r\n{}Connection: close\r\n\r\n",
            path, origin, auth
        );
        stream.write_all(req.as_bytes()).unwrap();
        let mut resp = String::new();
        stream.read_to_string(&mut resp).unwrap();
        resp
    };

    let t1 = token.clone();
    let r_arch = tokio::task::spawn_blocking(move || {
        req_with_origin("/api/v1/architecture", "http://localhost:5173", Some(&t1))
    })
    .await
    .unwrap();
    assert!(r_arch.starts_with("HTTP/1.1 200"));
    assert!(r_arch.contains("access-control-allow-origin: http://localhost:5173"));
    assert!(r_arch.contains("\"schema_version\":\"0.2.0\""));

    let t2 = token.clone();
    let r_rating = tokio::task::spawn_blocking(move || {
        req_with_origin("/api/v1/rating", "http://evil.example", Some(&t2))
    })
    .await
    .unwrap();
    assert!(r_rating.starts_with("HTTP/1.1 200"));
    assert!(
        !r_rating
            .to_lowercase()
            .contains("access-control-allow-origin"),
        "Non-localhost Origin must never receive CORS allow header"
    );

    let t3 = token.clone();
    let r_spoof = tokio::task::spawn_blocking(move || {
        req_with_origin("/api/v1/deception", "http://localhost:evil.com", Some(&t3))
    })
    .await
    .unwrap();
    assert!(r_spoof.starts_with("HTTP/1.1 200"));
    assert!(!r_spoof
        .to_lowercase()
        .contains("access-control-allow-origin"));

    let t4 = token.clone();
    let r_exp = tokio::task::spawn_blocking(move || {
        req_with_origin("/api/v1/experiment", "http://127.0.0.1:4173", Some(&t4))
    })
    .await
    .unwrap();
    assert!(r_exp.starts_with("HTTP/1.1 200"));
    assert!(r_exp.contains("access-control-allow-origin: http://127.0.0.1:4173"));

    server.abort();
}

#[test]
fn test_apply_one_command_workflow_and_dry_run() {
    use brahm::cli::{Cli, Commands};
    use clap::Parser;

    // Verify CLI aliases `apply` and `protect` parse cleanly
    assert!(matches!(
        Cli::try_parse_from(["brahm", "apply", "./my-app"])
            .unwrap()
            .command,
        Commands::Apply(_)
    ));
    assert!(matches!(
        Cli::try_parse_from(["brahm", "protect", "./my-app", "--dry-run"])
            .unwrap()
            .command,
        Commands::Apply(_)
    ));

    let repo_dir = tempdir().unwrap();
    fs::write(
        repo_dir.path().join("server.ts"),
        "import express from 'express';\nconst app = express();\napp.get('/health', healthCheck);\nfunction healthCheck() {}\n",
    )
    .unwrap();
    let state_dir = tempdir().unwrap();
    let dry_state = state_dir.path().join("dry");
    let real_state = state_dir.path().join("applied");

    let installer = ProtectionInstaller::new();

    // 1. Dry-run preview must not create persistent state directory artifacts
    let dry = installer
        .apply(
            repo_dir.path(),
            &dry_state,
            ProtectionProfile::ControlledInspection,
            true,
            true,
        )
        .unwrap();
    assert!(dry.dry_run);
    assert!(!dry_state.exists());

    // 2. Full apply generates architecture, decoys, active workspace mirror, and verifies integrity
    let applied = installer
        .apply(
            repo_dir.path(),
            &real_state,
            ProtectionProfile::ControlledInspection,
            false,
            true,
        )
        .unwrap();
    assert!(!applied.dry_run);
    assert!(applied.decoy_count > 0);
    assert!(applied.canary_count > 0);
    assert_eq!(applied.integrity_verified, Some(true));
    assert!(applied.session_id.is_some());
    assert!(applied
        .inspection_workspace
        .as_ref()
        .is_some_and(|w| w.is_dir()));
    assert!(real_state.join("BRAHM_BASELINE.json").is_file());
    assert!(real_state.join("BRAHM_INTEGRITY_REPORT.json").is_file());
}

#[test]
fn test_blind_shield_redacts_mirror_without_mutating_genuine_repo() {
    use brahm::cli::{Cli, Commands};
    use clap::Parser;

    assert!(matches!(
        Cli::try_parse_from(["brahm", "apply", "./app", "--shield"])
            .unwrap()
            .command,
        Commands::Apply(a) if a.shield
    ));

    let repo_dir = tempdir().unwrap();
    let ts_path = repo_dir.path().join("pricing.ts");
    let ts_orig = r#"
export interface PriceResult { total: number; }
export function computeSecretDiscount(amount: number): number {
  const hiddenFactor = 17;
  return amount - hiddenFactor;
}
"#;
    fs::write(&ts_path, ts_orig).unwrap();
    fs::write(
        repo_dir.path().join("schema.sql"),
        "CREATE TABLE secrets_cfg (id TEXT PRIMARY KEY, internal_key TEXT);\n",
    )
    .unwrap();

    let state_dir = tempdir().unwrap();
    let out = ProtectionInstaller::new()
        .apply(
            repo_dir.path(),
            state_dir.path(),
            ProtectionProfile::BlindShield,
            false,
            true,
        )
        .unwrap();

    assert_eq!(out.integrity_verified, Some(true));
    let ws = out.inspection_workspace.unwrap();

    // Isolated mirror has implementation bodies & SQL DDL redacted
    let ws_ts = fs::read_to_string(ws.join("pricing.ts")).unwrap();
    assert!(ws_ts.contains("export interface PriceResult"));
    assert!(ws_ts.contains("BRAHM_SEALED"));
    assert!(!ws_ts.contains("hiddenFactor"));

    let ws_sql = fs::read_to_string(ws.join("schema.sql")).unwrap();
    assert!(ws_sql.contains("BRAHM_SHIELD"));
    assert!(!ws_sql.contains("internal_key"));

    // Genuine repo files remain 100% unmodified
    assert_eq!(fs::read_to_string(&ts_path).unwrap(), ts_orig);
}
