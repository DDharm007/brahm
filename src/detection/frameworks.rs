//! Evidence-based framework detection from manifests and configurations.

pub use crate::models::{ConfidenceLevel, FrameworkIndicator};

/// Represents evidence discovered for a framework.
#[derive(Debug, Clone)]
pub struct FrameworkRule {
    pub name: &'static str,
    pub indicator_files: &'static [&'static str],
    pub manifest_dependencies: &'static [&'static str],
}
