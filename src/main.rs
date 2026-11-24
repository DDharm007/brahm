//! BRAHM CLI Entry Point.
//!
//! Provides CLI execution, structured logging initialization, and exit code management.

use brahm::cli::{Cli, Commands, InitArgs, ScanArgs};
use brahm::errors::Result;
use clap::Parser;
use std::process::ExitCode;
use tracing::{error, info};
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
    info!(
        "Repository validated at '{}'. Configured with max file size {} MB.",
        canonical_repo.display(),
        config.scanner.max_file_size_mb
    );
    Ok(())
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    init_logging(cli.verbose);

    match cli.command {
        Commands::Init(args) => handle_init(args)?,
        Commands::Scan(args) => handle_scan(args)?,
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
