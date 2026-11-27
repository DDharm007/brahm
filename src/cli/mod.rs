//! Command-line interface definition and parsing for BRAHM.
//!
//! Powered by `clap` with strict argument validation and clear documentation.

use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

pub mod handlers;

pub use handlers::{execute_init, prepare_scan_config};

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

    /// Run Stage 01 + Stage 02 deep code intelligence and architecture mapping
    Arch(ScanArgs),

    /// Architecture analysis commands (Stage 02)
    Architecture {
        #[command(subcommand)]
        action: ArchitectureCommands,
    },

    /// Decoy generation and registry management (Stage 03)
    Decoy {
        #[command(subcommand)]
        action: DecoyCommands,
    },

    /// Repository deception mirror, policy, and inspection session management (Stage 04)
    Deception {
        #[command(subcommand)]
        action: DeceptionCommands,
    },
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeceptionPolicyProfileArg {
    Conservative,
    Standard,
    Research,
}

impl From<DeceptionPolicyProfileArg> for crate::deception::policy::DeceptionPolicyProfile {
    fn from(arg: DeceptionPolicyProfileArg) -> Self {
        match arg {
            DeceptionPolicyProfileArg::Conservative => Self::Conservative,
            DeceptionPolicyProfileArg::Standard => Self::Standard,
            DeceptionPolicyProfileArg::Research => Self::Research,
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum DeceptionCommands {
    /// Prepare isolated inspection workspace & mirror for an authorized repository
    Prepare {
        /// Target genuine repository path
        repository: PathBuf,
        /// Directory containing generated decoys & BRAHM_DECOY_REGISTRY.json
        #[arg(short, long, default_value = ".brahm_decoys")]
        decoy_dir: PathBuf,
        /// Root directory where isolated inspection sessions will be created
        #[arg(short, long, default_value = ".brahm_sessions")]
        output: PathBuf,
        /// Deception policy profile (conservative, standard, research)
        #[arg(short, long, value_enum, default_value = "standard")]
        profile: DeceptionPolicyProfileArg,
        /// Session expiration TTL in seconds
        #[arg(long, default_value_t = 3600)]
        ttl: i64,
    },
    /// Deception policy listing and validation
    Policy {
        #[command(subcommand)]
        action: PolicyCommands,
    },
    /// Manage isolated inspection sessions
    Session {
        #[command(subcommand)]
        action: SessionCommands,
    },
    /// Safely clean up an inspection session workspace
    Cleanup {
        /// Session identifier (e.g., sess_...)
        session_id: String,
        /// Sessions root directory
        #[arg(long, default_value = ".brahm_sessions")]
        sessions_dir: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
pub enum PolicyCommands {
    /// List available deterministic policy presets
    List,
    /// Validate a policy JSON file against safety constraints
    Validate {
        /// Path to policy JSON file
        policy_file: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
pub enum SessionCommands {
    /// Create and prepare a new isolated inspection session
    Create {
        repository: PathBuf,
        #[arg(short, long, default_value = ".brahm_decoys")]
        decoy_dir: PathBuf,
        #[arg(short, long, default_value = ".brahm_sessions")]
        output: PathBuf,
        #[arg(short, long, value_enum, default_value = "standard")]
        profile: DeceptionPolicyProfileArg,
        #[arg(long, default_value_t = 3600)]
        ttl: i64,
    },
    /// Inspect status and deliverables for an inspection session
    Inspect {
        session_id: String,
        #[arg(long, default_value = ".brahm_sessions")]
        sessions_dir: PathBuf,
    },
    /// Stop/complete an active inspection session
    Stop {
        session_id: String,
        #[arg(long, default_value = ".brahm_sessions")]
        sessions_dir: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
pub enum ArchitectureCommands {
    /// Analyze a repository and build Stage 02 architecture intelligence baseline
    Analyze(ScanArgs),
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecoyProfileArg {
    Disabled,
    Conservative,
    Standard,
    Research,
}

impl From<DecoyProfileArg> for crate::deception::decoy::DecoyGenerationProfile {
    fn from(arg: DecoyProfileArg) -> Self {
        match arg {
            DecoyProfileArg::Disabled => Self::Disabled,
            DecoyProfileArg::Conservative => Self::Conservative,
            DecoyProfileArg::Standard => Self::Standard,
            DecoyProfileArg::Research => Self::Research,
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum DecoyCommands {
    /// Generate controlled synthetic decoys from Stage 02 architecture baseline
    Generate {
        /// Target repository path
        repository: PathBuf,
        /// Output directory for isolated decoy staging & registry files
        #[arg(short, long, default_value = ".brahm_decoys")]
        output: PathBuf,
        /// Generation profile (disabled, conservative, standard, research)
        #[arg(short, long, value_enum, default_value = "standard")]
        profile: DecoyProfileArg,
    },
    /// List registered decoys from a BRAHM decoy registry
    List {
        /// Path to BRAHM_DECOY_REGISTRY.json
        #[arg(short, long, default_value = ".brahm_decoys/BRAHM_DECOY_REGISTRY.json")]
        registry: PathBuf,
    },
    /// Inspect metadata for a specific decoy artifact ID
    Inspect {
        /// Artifact ID to inspect
        artifact_id: String,
        /// Path to BRAHM_DECOY_REGISTRY.json
        #[arg(short, long, default_value = ".brahm_decoys/BRAHM_DECOY_REGISTRY.json")]
        registry: PathBuf,
    },
    /// Validate integrity of staged decoys against registry fingerprints
    Validate {
        /// Path to BRAHM_DECOY_REGISTRY.json
        #[arg(short, long, default_value = ".brahm_decoys/BRAHM_DECOY_REGISTRY.json")]
        registry: PathBuf,
    },
    /// Safely remove ONLY BRAHM-owned decoys inside isolated staging directory
    Remove {
        /// Path to BRAHM_DECOY_REGISTRY.json
        #[arg(short, long, default_value = ".brahm_decoys/BRAHM_DECOY_REGISTRY.json")]
        registry: PathBuf,
        /// Preview removals without modifying disk
        #[arg(long)]
        dry_run: bool,
    },
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

    /// Run Stage 02 deep code intelligence and architecture mapping (AST, Symbols, Graph)
    #[arg(long, alias = "deep")]
    pub arch: bool,
}
