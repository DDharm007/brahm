//! Command-line interface definition and parsing for BRAHM.
//!
//! Powered by `clap` with strict argument validation and clear documentation.

use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// BRAHM — Blind Reverse-engineering & Adversarial Honeypot Mechanism
/// Stage 01: Repository Analyzer
#[derive(Parser, Debug)]
#[command(
    name = "brahm",
    author = "BRAHM Systems Architecture Team",
    version,
    about = "Automated repository intelligence, deterministic architecture mapping, and deception foundation.",
    long_about = "BRAHM (Blind Reverse-engineering & Adversarial Honeypot Mechanism) is a cybersecurity framework \
    designed to analyze repositories, establish genuine application architecture, and provide the deterministic foundation \
    for controlled deception mechanisms without AI runtime dependencies."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose diagnostic logging output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a default brahm.toml configuration in the target directory
    Init(InitArgs),

    /// Scan a repository and generate structured intelligence reports
    Scan(ScanArgs),
}

/// Output report format options
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputFormat {
    Terminal,
    Json,
    Markdown,
    All,
}

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Target directory or path to write brahm.toml
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Force overwrite of existing configuration without prompt
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Args, Debug)]
pub struct ScanArgs {
    /// Target repository path to scan
    #[arg(required = true)]
    pub repository: PathBuf,

    /// Report output format: terminal, json, markdown, or all
    #[arg(short, long, value_enum, default_value = "all")]
    pub format: OutputFormat,

    /// Directory where generated reports will be written
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Additional directories, files, or patterns to exclude from scan
    #[arg(short, long)]
    pub exclude: Vec<String>,

    /// Path to a specific brahm.toml configuration file
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Skip BLAKE3 file fingerprint calculations
    #[arg(long)]
    pub no_hash: bool,

    /// Follow symbolic links during scan (default: false for security)
    #[arg(long)]
    pub follow_symlinks: bool,
}
