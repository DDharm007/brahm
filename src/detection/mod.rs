//! Deterministic language, framework, and configuration detection.

pub mod frameworks;
pub mod languages;
pub mod manifests;

pub use frameworks::detect_frameworks;
pub use languages::detect_language;
pub use manifests::{
    is_configuration_or_manifest, is_manifest, is_sensitive_credential_file, KNOWN_CONFIGURATIONS,
    KNOWN_MANIFESTS,
};
