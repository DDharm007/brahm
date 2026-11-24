//! Milestone 01 Foundation Verification Tests

use brahm::cli::{Cli, Commands};
use brahm::config::BrahmConfig;
use brahm::errors::BrahmError;
use clap::Parser;

#[test]
fn test_config_defaults_and_validation() {
    let config = BrahmConfig::default();
    assert!(config.validate().is_ok());
    assert!(!config.scanner.follow_symlinks);
    assert_eq!(config.scanner.max_file_size_mb, 100);
    assert!(config.scanner.calculate_hashes);
    assert!(config
        .scanner
        .exclusions
        .directories
        .contains(&".git".to_string()));
}

#[test]
fn test_cli_parsing_version_and_help() {
    // Test parsing 'init' command
    let cli = Cli::try_parse_from(["brahm", "init"]).expect("Failed to parse init");
    match cli.command {
        Commands::Init(args) => {
            assert_eq!(args.path.to_str().unwrap(), ".");
            assert!(!args.force);
        }
        _ => panic!("Expected Init command"),
    }

    // Test parsing 'scan' command
    let cli = Cli::try_parse_from([
        "brahm",
        "scan",
        "./test_repo",
        "--format",
        "json",
        "--exclude",
        "node_modules",
        "--verbose",
    ])
    .expect("Failed to parse scan");

    assert!(cli.verbose);
    match cli.command {
        Commands::Scan(args) => {
            assert_eq!(args.repository.to_str().unwrap(), "./test_repo");
            assert_eq!(args.exclude, vec!["node_modules".to_string()]);
        }
        _ => panic!("Expected Scan command"),
    }
}

#[test]
fn test_error_handling_display() {
    let err = BrahmError::Config("Test configuration error".to_string());
    assert_eq!(
        format!("{}", err),
        "Configuration error: Test configuration error"
    );

    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let brahm_io = BrahmError::from(io_err);
    assert!(format!("{}", brahm_io).contains("I/O error"));
}
