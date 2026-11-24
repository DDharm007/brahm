//! BRAHM CLI Entry Point.
//!
//! Provides CLI execution, structured logging initialization, and exit code management.

use brahm::cli::{Cli, Commands, InitArgs, ScanArgs};
use brahm::config::BrahmConfig;
use brahm::errors::{BrahmError, Result};
use clap::Parser;
use std::path::PathBuf;
use std::process::ExitCode;
use tracing::{error, info, warn};
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
    let config_dir = &args.path;
    let target_file: PathBuf = if config_dir.is_dir() {
        config_dir.join("brahm.toml")
    } else {
        config_dir.clone()
    };

    if target_file.exists() && !args.force {
        warn!(
            "Configuration file already exists at '{}'. Use --force to overwrite.",
            target_file.display()
        );
        return Err(BrahmError::Config(format!(
            "Configuration file '{}' already exists. Aborting to avoid data loss.",
            target_file.display()
        )));
    }

    let default_config = BrahmConfig::default();
    default_config.save_to_path(&target_file)?;
    info!(
        "Successfully initialized BRAHM configuration at '{}'.",
        target_file.display()
    );
    Ok(())
}

fn handle_scan(args: ScanArgs) -> Result<()> {
    if !args.repository.exists() {
        return Err(BrahmError::InvalidPath {
            path: args.repository,
            reason: "Target repository path does not exist".to_string(),
        });
    }

    info!(
        "Target repository validated at '{}'. Ready for scan execution.",
        args.repository.display()
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
