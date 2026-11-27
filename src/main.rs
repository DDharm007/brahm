//! BRAHM CLI Entry Point.
//!
//! Provides CLI execution, structured logging initialization, and exit code management.

use brahm::cli::{Cli, Commands, InitArgs, ScanArgs};
use brahm::errors::Result;
use clap::Parser;
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

fn run() -> Result<()> {
    let cli = Cli::parse();
    init_logging(cli.verbose);

    match cli.command {
        Commands::Init(args) => handle_init(args)?,
        Commands::Scan(args) => handle_scan(args)?,
        Commands::Arch(args) => handle_arch(args)?,
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
