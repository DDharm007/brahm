//! Command execution handlers for BRAHM CLI.

use crate::cli::{InitArgs, ScanArgs};
use crate::config::BrahmConfig;
use crate::errors::{BrahmError, Result};
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Executes the `init` command, generating a default `brahm.toml` configuration.
pub fn execute_init(args: &InitArgs) -> Result<PathBuf> {
    let target_dir = &args.path;
    let target_file: PathBuf = if target_dir.is_dir() {
        target_dir.join("brahm.toml")
    } else if target_dir.extension().is_some_and(|ext| ext == "toml") {
        target_dir.clone()
    } else {
        if !target_dir.exists() {
            fs::create_dir_all(target_dir)?;
        }
        target_dir.join("brahm.toml")
    };

    if target_file.exists() && !args.force {
        warn!(
            "Configuration file already exists at '{}'. Use --force to overwrite.",
            target_file.display()
        );
        return Err(BrahmError::Config(format!(
            "Configuration file '{}' already exists. Aborting to prevent accidental data loss. Use --force to overwrite.",
            target_file.display()
        )));
    }

    let default_config = BrahmConfig::default();
    default_config.save_to_path(&target_file)?;

    info!(
        "Successfully initialized BRAHM configuration at '{}'.",
        target_file.display()
    );
    Ok(target_file)
}

/// Validates scan inputs, locates/loads configuration, and applies command-line overrides.
pub fn prepare_scan_config(args: &ScanArgs) -> Result<(BrahmConfig, PathBuf)> {
    let repo_path = &args.repository;

    if !repo_path.exists() {
        return Err(BrahmError::InvalidPath {
            path: repo_path.clone(),
            reason: "Target path does not exist on the filesystem.".to_string(),
        });
    }

    if !repo_path.is_dir() {
        return Err(BrahmError::InvalidPath {
            path: repo_path.clone(),
            reason: "Target path is a file, but repository scanning requires a directory."
                .to_string(),
        });
    }

    let canonical_repo = match fs::canonicalize(repo_path) {
        Ok(p) => p,
        Err(e) => {
            return Err(BrahmError::InvalidPath {
                path: repo_path.clone(),
                reason: format!("Failed to resolve canonical path: {}", e),
            })
        }
    };

    // Load configuration either from explicitly passed path or discover default
    let (mut config, loaded_from) = if let Some(ref config_path) = args.config {
        if !config_path.exists() {
            return Err(BrahmError::InvalidPath {
                path: config_path.clone(),
                reason: "Specified custom configuration file does not exist.".to_string(),
            });
        }
        let cfg = BrahmConfig::load_from_path(config_path)?;
        (cfg, Some(config_path.clone()))
    } else {
        BrahmConfig::find_or_default(&canonical_repo)
    };

    if let Some(ref path) = loaded_from {
        debug!("Loaded configuration from '{}'", path.display());
    } else {
        debug!("Using default BRAHM configuration (no brahm.toml found)");
    }

    // Apply CLI overrides
    if args.no_hash {
        config.scanner.calculate_hashes = false;
    }
    if args.follow_symlinks {
        config.scanner.follow_symlinks = true;
    }
    for custom_exclude in &args.exclude {
        if !config
            .scanner
            .exclusions
            .directories
            .contains(custom_exclude)
        {
            config
                .scanner
                .exclusions
                .directories
                .push(custom_exclude.clone());
        }
    }

    // Validate output directory if specified
    if let Some(ref output_dir) = args.output {
        if !output_dir.exists() {
            fs::create_dir_all(output_dir)?;
            debug!(
                "Created report output directory at '{}'",
                output_dir.display()
            );
        }
    }

    Ok((config, canonical_repo))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::OutputFormat;
    use tempfile::tempdir;

    #[test]
    fn test_execute_init_creates_file() {
        let dir = tempdir().unwrap();
        let args = InitArgs {
            path: dir.path().to_path_buf(),
            force: false,
        };
        let created = execute_init(&args).expect("Init should succeed");
        assert!(created.is_file());
        assert!(created.ends_with("brahm.toml"));

        // Second init without force must fail safely
        let second_result = execute_init(&args);
        assert!(second_result.is_err());

        // Second init with force must succeed
        let force_args = InitArgs {
            path: dir.path().to_path_buf(),
            force: true,
        };
        assert!(execute_init(&force_args).is_ok());
    }

    #[test]
    fn test_prepare_scan_invalid_path() {
        let args = ScanArgs {
            repository: PathBuf::from("d:/non_existent_path_xyz_12345"),
            format: OutputFormat::All,
            output: None,
            exclude: vec![],
            config: None,
            no_hash: false,
            follow_symlinks: false,
        };
        assert!(prepare_scan_config(&args).is_err());
    }

    #[test]
    fn test_prepare_scan_valid_dir() {
        let dir = tempdir().unwrap();
        let args = ScanArgs {
            repository: dir.path().to_path_buf(),
            format: OutputFormat::Json,
            output: Some(dir.path().join("reports")),
            exclude: vec!["custom_folder".to_string()],
            config: None,
            no_hash: true,
            follow_symlinks: true,
        };
        let (config, canonical) = prepare_scan_config(&args).expect("Prepare should succeed");
        assert!(canonical.exists());
        assert!(!config.scanner.calculate_hashes);
        assert!(config.scanner.follow_symlinks);
        assert!(config
            .scanner
            .exclusions
            .directories
            .contains(&"custom_folder".to_string()));
        assert!(dir.path().join("reports").exists());
    }
}
