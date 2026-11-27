//! BRAHM CLI Entry Point.
//!
//! Provides CLI execution, structured logging initialization, and exit code management.

use brahm::cli::{
    ArchitectureCommands, Cli, Commands, ConnectCommands, DeceptionCommands, DecoyCommands,
    InitArgs, PolicyCommands, ProtectionCommands, ScanArgs, SessionCommands,
};
use brahm::config::BrahmConfig;
use brahm::deception::decoy::{DecoyEngine, DecoyGenerationProfile, DecoyRegistry};
use brahm::deception::{
    DeceptionPolicy, DeceptionPolicyEngine, DeceptionPolicyProfile, InspectionSessionManager,
};
use brahm::errors::{BrahmError, Result};
use brahm::integration::{ProtectionInstaller, RepositoryInputManager};
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
        Commands::Protection { action } => handle_protection(action)?,
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
