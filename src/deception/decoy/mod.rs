//! Stage 03 Decoy Generation Engine subsystem.

pub mod engine;
pub mod generator;
pub mod identifiers;
pub mod models;
pub mod registry;
pub mod templates;
pub mod validation;

pub use engine::{render_decoy_report_markdown, DecoyEngine, DecoyGenerationResult};
pub use generator::DecoyGenerator;
pub use identifiers::{
    compute_content_fingerprint, generate_canary_marker, generate_decoy_artifact_id,
    generate_repository_id,
};
pub use models::{
    DecoyArtifact, DecoyCategory, DecoyCleanupResult, DecoyGenerationProfile, DecoyInstallState,
    DecoyManifest, DecoyManifestEntry, DecoyMonitoringState, DecoyRegistryData, DecoyTemplate,
    DecoyValidationIssue, DecoyValidationReport, GeneratedDecoy, RiskClassification,
    ValidationSeverity,
};
pub use registry::DecoyRegistry;
pub use templates::{render_template_body, validate_template, TemplateContext, TemplateRegistry};
pub use validation::DecoyValidator;
