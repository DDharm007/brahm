//! Milestone 02 CLI Architecture Integration Tests

use brahm::cli::{
    execute_init, prepare_scan_config, Cli, Commands, InitArgs, OutputFormat, ScanArgs,
};
use clap::Parser;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_cli_version_flag() {
    let help_cli = Cli::try_parse_from(["brahm", "--help"]);
    assert!(help_cli.is_err()); // Clap exits with Help error type when --help is passed
    let err = help_cli.unwrap_err();
    assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);

    let ver_cli = Cli::try_parse_from(["brahm", "--version"]);
    assert!(ver_cli.is_err());
    let err = ver_cli.unwrap_err();
    assert_eq!(err.kind(), clap::error::ErrorKind::DisplayVersion);
}

#[test]
fn test_init_command_creates_valid_brahm_toml() {
    let dir = tempdir().unwrap();
    let args = InitArgs {
        path: dir.path().to_path_buf(),
        force: false,
    };

    let target_file = execute_init(&args).expect("Init must succeed on clean dir");
    assert!(target_file.is_file());
    assert_eq!(target_file.file_name().unwrap(), "brahm.toml");

    let content = fs::read_to_string(&target_file).expect("Must read created config");
    assert!(content.contains("[scanner]"));
    assert!(content.contains("follow_symlinks = false"));
}

#[test]
fn test_init_safely_prevents_overwrite_without_force() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("brahm.toml");
    fs::write(&config_path, "# Original User Config\n").unwrap();

    let args = InitArgs {
        path: dir.path().to_path_buf(),
        force: false,
    };

    let result = execute_init(&args);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));

    // File content must be preserved untouched
    let content = fs::read_to_string(&config_path).unwrap();
    assert_eq!(content, "# Original User Config\n");

    // With force flag, overwrite is permitted
    let force_args = InitArgs {
        path: dir.path().to_path_buf(),
        force: true,
    };
    let force_result = execute_init(&force_args);
    assert!(force_result.is_ok());
    let new_content = fs::read_to_string(&config_path).unwrap();
    assert!(new_content.contains("[scanner]"));
}

#[test]
fn test_scan_options_parsing() {
    // Test: brahm scan ./project --format json
    let cli = Cli::try_parse_from(["brahm", "scan", "./project", "--format", "json"]).unwrap();
    if let Commands::Scan(args) = cli.command {
        assert_eq!(args.format, OutputFormat::Json);
        assert_eq!(args.repository.to_str().unwrap(), "./project");
    } else {
        panic!("Expected scan command");
    }

    // Test: brahm scan ./project --format markdown
    let cli = Cli::try_parse_from(["brahm", "scan", "./project", "--format", "markdown"]).unwrap();
    if let Commands::Scan(args) = cli.command {
        assert_eq!(args.format, OutputFormat::Markdown);
    } else {
        panic!("Expected scan command");
    }

    // Test: brahm scan ./project --output ./reports
    let cli = Cli::try_parse_from(["brahm", "scan", "./project", "--output", "./reports"]).unwrap();
    if let Commands::Scan(args) = cli.command {
        assert_eq!(args.output.unwrap().to_str().unwrap(), "./reports");
    } else {
        panic!("Expected scan command");
    }

    // Test: brahm scan ./project --exclude node_modules --verbose
    let cli = Cli::try_parse_from([
        "brahm",
        "scan",
        "./project",
        "--exclude",
        "node_modules",
        "--verbose",
    ])
    .unwrap();
    assert!(cli.verbose);
    if let Commands::Scan(args) = cli.command {
        assert_eq!(args.exclude, vec!["node_modules".to_string()]);
    } else {
        panic!("Expected scan command");
    }
}

#[test]
fn test_scan_rejects_invalid_path() {
    let args = ScanArgs {
        repository: std::path::PathBuf::from("/this/path/definitely/does/not/exist/9999"),
        format: OutputFormat::All,
        output: None,
        exclude: vec![],
        config: None,
        no_hash: false,
        follow_symlinks: false,
    };

    let result = prepare_scan_config(&args);
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(err_str.contains("Invalid path") || err_str.contains("does not exist"));
}

#[test]
fn test_scan_rejects_file_target() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("not_a_dir.txt");
    fs::write(&file_path, "sample content").unwrap();

    let args = ScanArgs {
        repository: file_path,
        format: OutputFormat::All,
        output: None,
        exclude: vec![],
        config: None,
        no_hash: false,
        follow_symlinks: false,
    };

    let result = prepare_scan_config(&args);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("requires a directory"));
}
