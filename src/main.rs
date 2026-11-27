//! BRAHM CLI Entry Point.
//!
//! Provides CLI execution, structured logging initialization, and exit code management.

use brahm::cli::{
    AgentCommands, ApplyArgs, ArchitectureCommands, Cli, Commands, ConnectCommands,
    DeceptionCommands, DecoyCommands, EventsCommands, InitArgs, IntegrityCommands, MonitorCommands,
    PolicyCommands, ProtectionCommands, ReportsCommands, ScanArgs, SessionCommands,
    SessionsTimelineCommands,
};
use brahm::config::BrahmConfig;
use brahm::deception::decoy::{DecoyEngine, DecoyGenerationProfile, DecoyRegistry};
use brahm::deception::{
    ControlledInspectionEnvironment, DeceptionPolicy, DeceptionPolicyEngine,
    DeceptionPolicyProfile, InspectionSessionManager,
};
use brahm::errors::{BrahmError, Result};
use brahm::integration::{
    load_stored_baseline, InspectionAgent, LocalTestAgentAdapter, ProtectionInstaller,
    RepositoryInputManager, RepositoryRecord,
};
use brahm::monitoring::{
    generate_all_monitoring_reports, serve_monitoring_api, CanaryDetector, EventSeverity,
    EventSource, EventStore, EventType, IntegrityMonitor, MonitoringApiConfig, MonitoringEvent,
    RuleBasedClassifier, ScoringWeights, SqliteEventStore, TimelineCorrelator,
};
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use tracing::error;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn init_logging(verbose: bool) {
    let filter = if verbose {
        EnvFilter::new("brahm=debug,info")
    } else {
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("brahm=info,warn,error"))
    };

    let _ = tracing_subscriber::registry()
        .with(fmt::layer().with_target(false).compact())
        .with(filter)
        .try_init();
}

fn handle_init(args: InitArgs) -> Result<()> {
    let created = brahm::cli::execute_init(&args)?;
    println!(
        "Initialized BRAHM configuration file at: {}",
        created.display()
    );
    Ok(())
}

fn handle_scan(args: ScanArgs) -> Result<()> {
    let (config, canonical_repo) = brahm::cli::prepare_scan_config(&args)?;
    let report = brahm::analysis::run_repository_analysis(&canonical_repo, config)?;
    brahm::reporting::dispatch_reports(&report, args.format, args.output.as_deref())?;

    if args.arch {
        let engine = brahm::architecture::ArchitectureEngine::new();
        let bundle = engine.analyze(&canonical_repo, &report)?;
        brahm::reporting::dispatch_architecture_reports(
            &bundle,
            args.format,
            args.output.as_deref(),
        )?;
    }

    Ok(())
}

fn handle_arch(mut args: ScanArgs) -> Result<()> {
    args.arch = true;
    handle_scan(args)
}

fn handle_decoy(action: DecoyCommands) -> Result<()> {
    match action {
        DecoyCommands::Generate {
            repository,
            output,
            profile,
        } => {
            let stage1 =
                brahm::analysis::run_repository_analysis(&repository, BrahmConfig::default())?;
            let arch_bundle =
                brahm::architecture::ArchitectureEngine::new().analyze(&repository, &stage1)?;
            let gen_profile: DecoyGenerationProfile = profile.into();
            let result = DecoyEngine::new().generate_and_stage(
                &repository,
                &arch_bundle.report,
                gen_profile,
                &output,
            )?;
            println!(
                "Generated {} decoy artifact(s) under {}",
                result.registry.artifacts().len(),
                result.decoy_layer_root.display()
            );
            println!("Registry: {}", result.registry_path.display());
            println!("Manifest: {}", result.manifest_path.display());
            println!("Report:   {}", result.report_path.display());
        }
        DecoyCommands::List { registry } => {
            let reg = DecoyRegistry::load_from_path(&registry)?;
            println!(
                "Registered Decoys for '{}' ({} total):",
                reg.data().repository_name,
                reg.artifacts().len()
            );
            for art in reg.artifacts() {
                println!(
                    "  {} | {:<24} | {:<36} | state={:?}",
                    art.artifact_id,
                    art.artifact_type.to_string(),
                    art.synthetic_relative_path,
                    art.install_state
                );
            }
        }
        DecoyCommands::Inspect {
            artifact_id,
            registry,
        } => {
            let reg = DecoyRegistry::load_from_path(&registry)?;
            let art = reg.get(&artifact_id).ok_or_else(|| {
                BrahmError::Decoy(format!("Artifact '{}' not found in registry", artifact_id))
            })?;
            println!("{}", serde_json::to_string_pretty(art)?);
        }
        DecoyCommands::Validate { registry } => {
            let reg = DecoyRegistry::load_from_path(&registry)?;
            let decoy_root = PathBuf::from(&reg.data().decoy_layer_root);
            let checks = reg.verify_staged_integrity(&decoy_root);
            let mut all_ok = true;
            for (id, ok, msg) in &checks {
                let status = if *ok { "PASS" } else { "FAIL" };
                println!("[{}] {} — {}", status, id, msg);
                if !*ok {
                    all_ok = false;
                }
            }
            if !all_ok {
                return Err(BrahmError::Decoy(
                    "One or more staged decoy artifacts failed integrity verification".to_string(),
                ));
            }
        }
        DecoyCommands::Remove { registry, dry_run } => {
            let mut reg = DecoyRegistry::load_from_path(&registry)?;
            let decoy_root = PathBuf::from(&reg.data().decoy_layer_root);
            let cleanup = reg.cleanup_owned_artifacts(&decoy_root, dry_run)?;
            if !dry_run {
                reg.save_to_path(&registry)?;
            }
            println!(
                "Cleanup ({}): removed {}, skipped {}",
                if dry_run { "DRY-RUN" } else { "EXECUTED" },
                cleanup.removed_paths.len(),
                cleanup.skipped_unverified_paths.len()
            );
            for p in &cleanup.removed_paths {
                println!("  - {}", p);
            }
        }
    }
    Ok(())
}

fn prepare_deception_session_cli(
    repository: &Path,
    decoy_dir: &Path,
    output: &Path,
    profile: DeceptionPolicyProfile,
    ttl: i64,
) -> Result<()> {
    let reg_path = decoy_dir.join("BRAHM_DECOY_REGISTRY.json");
    let (registry, decoy_root) = if reg_path.is_file() {
        let reg = DecoyRegistry::load_from_path(&reg_path)?;
        let root = PathBuf::from(&reg.data().decoy_layer_root);
        (reg, root)
    } else {
        // Generate decoys seamlessly into decoy_dir if not generated yet
        let stage1 = brahm::analysis::run_repository_analysis(repository, BrahmConfig::default())?;
        let arch = brahm::architecture::ArchitectureEngine::new().analyze(repository, &stage1)?;
        let gen_res = DecoyEngine::new().generate_and_stage(
            repository,
            &arch.report,
            DecoyGenerationProfile::Standard,
            decoy_dir,
        )?;
        (gen_res.registry, gen_res.decoy_layer_root)
    };

    let policy = DeceptionPolicy::for_profile(profile);
    let mgr = InspectionSessionManager::new(output);
    let outcome = mgr.create_and_prepare(repository, &registry, &decoy_root, &policy, ttl)?;

    println!(
        "Prepared isolated inspection session: {}",
        outcome.session.session_id
    );
    println!("  Workspace: {}", outcome.session.workspace_path.display());
    println!("  Baseline:  {}", outcome.session.baseline_fingerprint);
    println!(
        "  Injected:  {} synthetic artifact(s)",
        outcome.session.registered_artifacts.len()
    );
    Ok(())
}

fn handle_deception(action: DeceptionCommands) -> Result<()> {
    match action {
        DeceptionCommands::Prepare {
            repository,
            decoy_dir,
            output,
            profile,
            ttl,
        } => {
            prepare_deception_session_cli(&repository, &decoy_dir, &output, profile.into(), ttl)?;
        }
        DeceptionCommands::Policy { action } => match action {
            PolicyCommands::List => {
                let engine = DeceptionPolicyEngine::new();
                for pol in engine.list_presets() {
                    println!(
                        "{} (v{}, profile={}, priority={})",
                        pol.policy_id, pol.version, pol.profile, pol.priority
                    );
                }
            }
            PolicyCommands::Validate { policy_file } => {
                let raw = fs::read_to_string(&policy_file)?;
                let pol: DeceptionPolicy = serde_json::from_str(&raw)?;
                pol.validate()?;
                println!("Policy '{}' validated successfully.", pol.policy_id);
            }
        },
        DeceptionCommands::Session { action } => match action {
            SessionCommands::Create {
                repository,
                decoy_dir,
                output,
                profile,
                ttl,
            } => {
                prepare_deception_session_cli(
                    &repository,
                    &decoy_dir,
                    &output,
                    profile.into(),
                    ttl,
                )?;
            }
            SessionCommands::Inspect {
                session_id,
                sessions_dir,
            } => {
                let mgr = InspectionSessionManager::new(&sessions_dir);
                let sess = mgr.load_session(&session_id)?;
                println!("{}", serde_json::to_string_pretty(&sess)?);
            }
            SessionCommands::Stop {
                session_id,
                sessions_dir,
            } => {
                let mgr = InspectionSessionManager::new(&sessions_dir);
                let sess = mgr.stop_session(&session_id)?;
                record_session_lifecycle_if_db(
                    &sessions_dir,
                    &sess.repository_id,
                    &sess.session_id,
                    EventType::InspectionSessionEnded,
                );
                println!(
                    "Session '{}' status updated to {:?}",
                    sess.session_id, sess.status
                );
            }
        },
        DeceptionCommands::Cleanup {
            session_id,
            sessions_dir,
        } => {
            let mgr = InspectionSessionManager::new(&sessions_dir);
            let sess = mgr.cleanup_session(&session_id)?;
            record_session_lifecycle_if_db(
                &sessions_dir,
                &sess.repository_id,
                &sess.session_id,
                EventType::CleanupCompleted,
            );
            println!(
                "Session '{}' workspace cleaned ({:?})",
                sess.session_id, sess.status
            );
        }
    }
    Ok(())
}

fn handle_connect(action: ConnectCommands) -> Result<()> {
    let mgr = RepositoryInputManager::new();
    match action {
        ConnectCommands::Local { path, state_dir } => {
            let conn = mgr.connect_local(&path, &state_dir, None)?;
            println!(
                "Connected local repository '{}' ({}) baseline={}",
                conn.record.name, conn.record.repository_id, conn.record.baseline_fingerprint
            );
        }
        ConnectCommands::Github {
            url,
            dest,
            state_dir,
        } => {
            let conn = mgr.connect_github(&url, &dest, &state_dir)?;
            println!(
                "Connected GitHub repository '{}' ({}) at {}",
                conn.record.name,
                conn.record.repository_id,
                conn.record.local_path.display()
            );
        }
    }
    Ok(())
}

fn handle_protection(action: ProtectionCommands) -> Result<()> {
    let installer = ProtectionInstaller::new();
    match action {
        ProtectionCommands::Plan {
            repository,
            state_dir,
            profile,
        } => {
            let plan = installer.plan(&repository, &state_dir, profile.into())?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
        ProtectionCommands::Validate { state_dir } => {
            let (rec, ok, msg) = installer.validate_installation(&state_dir)?;
            println!("Repository '{}' ({}): {}", rec.name, rec.repository_id, msg);
            if !ok {
                return Err(BrahmError::Integration(msg));
            }
        }
        ProtectionCommands::Install {
            repository,
            state_dir,
            profile,
            approve,
        } => {
            let out = installer.install(&repository, &state_dir, profile.into(), approve)?;
            println!(
                "Installed protection profile '{}' for '{}' (report: {})",
                out.record.active_profile,
                out.record.name,
                out.install_report_path.display()
            );
        }
        ProtectionCommands::Status { state_dir } => {
            let (rec, ok, msg) = installer.validate_installation(&state_dir)?;
            println!("{}", serde_json::to_string_pretty(&rec)?);
            println!("Validation status: {} ({})", ok, msg);
        }
        ProtectionCommands::Rollback { state_dir, dry_run } => {
            let rep = installer.rollback(&state_dir, dry_run)?;
            println!("{}", serde_json::to_string_pretty(&rep)?);
        }
    }
    Ok(())
}

fn handle_monitor(action: MonitorCommands) -> Result<()> {
    match action {
        MonitorCommands::Start {
            state_dir,
            bind,
            allow_remote,
        } => {
            let token = std::env::var("BRAHM_MONITOR_TOKEN").map_err(|_| {
                BrahmError::Unauthorized(
                    "BRAHM_MONITOR_TOKEN environment variable (min 16 chars) is required before starting Monitoring API"
                        .to_string(),
                )
            })?;
            let bind_addr: std::net::SocketAddr = bind.parse().map_err(|e| {
                BrahmError::Config(format!("Invalid bind address '{}': {}", bind, e))
            })?;

            let db_path = state_dir.join("brahm_monitoring.db");
            let store = SqliteEventStore::open(&db_path)?;
            let cfg = MonitoringApiConfig {
                bind_addr,
                allow_remote,
                auth_token: token,
                rate_limit_per_minute: 120,
                state_dir,
            };
            let rt = tokio::runtime::Runtime::new().map_err(|e| {
                BrahmError::Monitoring(format!("Failed starting tokio runtime: {}", e))
            })?;
            println!(
                "Starting authenticated BRAHM monitoring API on http://{}",
                bind_addr
            );
            rt.block_on(serve_monitoring_api(cfg, store))?;
        }
        MonitorCommands::Status { state_dir } => {
            let db_path = state_dir.join("brahm_monitoring.db");
            let store = SqliteEventStore::open(&db_path)?;
            let count = store.count_events()?;
            println!(
                "Monitoring DB: {}\nRecorded Events: {}",
                db_path.display(),
                count
            );
        }
        MonitorCommands::Stop { state_dir } => {
            let db_path = state_dir.join("brahm_monitoring.db");
            if db_path.is_file() {
                let store = SqliteEventStore::open(&db_path)?;
                store.set_config("monitor_status", "stopped")?;
            }
            println!("Monitoring service state marked stopped.");
        }
    }
    Ok(())
}

fn handle_events(action: EventsCommands) -> Result<()> {
    match action {
        EventsCommands::List {
            db,
            repository_id,
            session_id,
            limit,
        } => {
            let store = SqliteEventStore::open(&db)?;
            let evts = store.list_events(repository_id.as_deref(), session_id.as_deref(), limit)?;
            println!("{}", serde_json::to_string_pretty(&evts)?);
        }
        EventsCommands::Inspect { event_id, db } => {
            let store = SqliteEventStore::open(&db)?;
            let evt = store
                .get_event(&event_id)?
                .ok_or_else(|| BrahmError::Monitoring(format!("Event '{}' not found", event_id)))?;
            println!("{}", serde_json::to_string_pretty(&evt)?);
        }
    }
    Ok(())
}

fn handle_sessions_timeline(action: SessionsTimelineCommands) -> Result<()> {
    match action {
        SessionsTimelineCommands::Timeline {
            session_id,
            state_dir,
        } => {
            let store = SqliteEventStore::open(state_dir.join("brahm_monitoring.db"))?;
            let evts = store.list_events(None, Some(&session_id), 1000)?;
            let repo_id = evts
                .first()
                .map(|e| e.repository_id.as_str())
                .unwrap_or("unknown_repo");
            let timeline =
                TimelineCorrelator::new().build_session_timeline(&session_id, repo_id, &evts);
            println!("{}", serde_json::to_string_pretty(&timeline)?);
        }
    }
    Ok(())
}

fn record_session_lifecycle_if_db(
    sessions_dir: &Path,
    repo_id: &str,
    session_id: &str,
    event_type: EventType,
) {
    if let Some(parent) = sessions_dir.parent() {
        let db_path = parent.join("brahm_monitoring.db");
        if db_path.is_file() {
            if let Ok(store) = SqliteEventStore::open(&db_path) {
                let _ = store.record_event(&MonitoringEvent {
                    event_id: format!(
                        "evt_{}",
                        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
                    ),
                    event_type,
                    repository_id: repo_id.to_string(),
                    session_id: Some(session_id.to_string()),
                    artifact_id: None,
                    timestamp: chrono::Utc::now(),
                    source: EventSource::PolicyEngine,
                    severity: EventSeverity::Informational,
                    correlation_id: format!("lifecycle_{}", session_id),
                    metadata: std::collections::BTreeMap::new(),
                });
            }
        }
    }
}

fn handle_integrity_check(action: IntegrityCommands) -> Result<()> {
    match action {
        IntegrityCommands::Check { state_dir } => {
            let record = ProtectionInstaller::new().verify_state(&state_dir, "corr")?;
            println!("{}", serde_json::to_string_pretty(&record)?);
        }
    }
    Ok(())
}

fn handle_reports_generate(action: ReportsCommands) -> Result<()> {
    match action {
        ReportsCommands::Generate { state_dir, output } => {
            let out_dir = output.unwrap_or_else(|| state_dir.clone());
            let cfg_path = state_dir.join("BRAHM_REPOSITORY_CONFIG.json");
            let rec: RepositoryRecord = serde_json::from_str(&fs::read_to_string(&cfg_path)?)?;
            let store = SqliteEventStore::open(state_dir.join("brahm_monitoring.db"))?;
            let events = store.list_events(Some(&rec.repository_id), None, 10_000)?;
            let sess_mgr = InspectionSessionManager::new(state_dir.join("sessions"));
            let sessions = sess_mgr.list_sessions()?;

            let integrity = match store.latest_integrity(&rec.repository_id)? {
                Some(r) => r,
                None => {
                    let baseline = load_stored_baseline(&state_dir)?;
                    IntegrityMonitor::new()
                        .check_integrity(
                            &rec.repository_id,
                            &rec.local_path,
                            &baseline,
                            None,
                            &std::collections::HashSet::new(),
                            "report_gen",
                        )?
                        .0
                }
            };

            let classifier = RuleBasedClassifier::new(ScoringWeights::default());
            let generated = generate_all_monitoring_reports(
                &rec.repository_id,
                &rec.name,
                &events,
                integrity,
                sessions.len(),
                &classifier,
                &out_dir,
            )?;

            println!(
                "Generated Stage 06 Intelligence Reports in {}:",
                out_dir.display()
            );
            println!("  - {}", generated.monitoring_report_path.display());
            println!("  - {}", generated.events_path.display());
            println!("  - {}", generated.timeline_path.display());
            println!("  - {}", generated.integrity_path.display());
            println!("  - {}", generated.markdown_path.display());
        }
    }
    Ok(())
}

fn handle_agent(action: AgentCommands) -> Result<()> {
    match action {
        AgentCommands::Run {
            session_id,
            state_dir,
            paths,
        } => {
            let cfg_path = state_dir.join("BRAHM_REPOSITORY_CONFIG.json");
            let rec: RepositoryRecord = serde_json::from_str(&fs::read_to_string(&cfg_path)?)?;
            let decoy_reg_path = rec.decoy_registry_path.ok_or_else(|| {
                BrahmError::Integration("No decoy registry linked to repository state".to_string())
            })?;
            let registry = DecoyRegistry::load_from_path(&decoy_reg_path)?;

            let sess_mgr = InspectionSessionManager::new(state_dir.join("sessions"));
            let session = sess_mgr.load_session(&session_id)?;
            let mut sandbox = ControlledInspectionEnvironment::open(session, registry.clone())?;

            let adapter = LocalTestAgentAdapter::new();
            let agent_res = adapter.run_inspection(&mut sandbox, &paths)?;

            let store = SqliteEventStore::open(state_dir.join("brahm_monitoring.db"))?;
            let corr_id = format!("agent_{}", agent_res.agent_session_id);
            let mut detector = CanaryDetector::new(5);
            let (_canaries, mon_events) = detector.process_sandbox_accesses(
                &rec.repository_id,
                &registry,
                sandbox.access_log(),
                &corr_id,
            );
            store.record_events_batch(&mon_events)?;

            println!("{}", serde_json::to_string_pretty(&agent_res)?);
        }
    }
    Ok(())
}

fn handle_apply(args: ApplyArgs) -> Result<()> {
    let out = ProtectionInstaller::new().apply(
        &args.repository,
        &args.state_dir,
        args.profile.into(),
        args.dry_run,
        args.verify,
    )?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    if out.dry_run {
        println!(
            "[DRY-RUN] Protection plan for '{}' ({})",
            out.repository_name, out.repository_id
        );
        println!("  Profile:          {}", out.profile);
        println!("  Target State Dir: {}", out.state_dir.display());
        println!(
            "  Will Generate Decoys: {} | Create Isolated Workspace: {}",
            out.plan.will_generate_decoys, out.plan.will_create_mirror
        );
        return Ok(());
    }

    println!(
        "BRAHM protection applied for '{}' ({})",
        out.repository_name, out.repository_id
    );
    println!("  Profile:              {}", out.profile);
    println!("  Genuine Repo (clean): {}", out.repository_path.display());
    println!("  Baseline Digest:      {}", out.baseline_fingerprint);
    println!(
        "  Staged Decoys:        {} ({} canaries)",
        out.decoy_count, out.canary_count
    );
    if let Some(ref sid) = out.session_id {
        println!("  Active Session ID:    {}", sid);
    }
    if let Some(ref ws) = out.inspection_workspace {
        println!("  Inspection Workspace: {}", ws.display());
    }
    if let Some(ok) = out.integrity_verified {
        println!(
            "  Integrity Verified:   {}",
            if ok { "PASS" } else { "FAILED" }
        );
    }
    println!("  State Directory:      {}", out.state_dir.display());
    Ok(())
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    init_logging(cli.verbose);

    match cli.command {
        Commands::Init(args) => handle_init(args)?,
        Commands::Scan(args) => handle_scan(args)?,
        Commands::Arch(args) => handle_arch(args)?,
        Commands::Architecture { action } => match action {
            ArchitectureCommands::Analyze(args) => handle_arch(args)?,
        },
        Commands::Decoy { action } => handle_decoy(action)?,
        Commands::Deception { action } => handle_deception(action)?,
        Commands::Connect { action } => handle_connect(action)?,
        Commands::Apply(args) => handle_apply(args)?,
        Commands::Protection { action } => handle_protection(action)?,
        Commands::Monitor { action } => handle_monitor(action)?,
        Commands::Events { action } => handle_events(action)?,
        Commands::Sessions { action } => handle_sessions_timeline(action)?,
        Commands::Integrity { action } => handle_integrity_check(action)?,
        Commands::Reports { action } => handle_reports_generate(action)?,
        Commands::Agent { action } => handle_agent(action)?,
    }

    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            error!("{}", err);
            ExitCode::FAILURE
        }
    }
}
