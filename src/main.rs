//! BRAHM CLI Entry Point.
//!
//! Provides CLI execution, structured logging initialization, and exit code management.

use brahm::cli::{ArchitectureCommands, Cli, Commands, DecoyCommands, InitArgs, ScanArgs};
use brahm::config::BrahmConfig;
use brahm::deception::decoy::{DecoyEngine, DecoyGenerationProfile, DecoyRegistry};
use brahm::errors::{BrahmError, Result};
use clap::Parser;
use std::path::PathBuf;
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
