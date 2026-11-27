//! High-level Decoy Engine orchestrating generation, validation, isolated staging, and deliverables.

use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

use super::generator::DecoyGenerator;
use super::identifiers::generate_repository_id;
use super::models::{
    DecoyGenerationProfile, DecoyInstallState, DecoyManifest, DecoyManifestEntry,
    DecoyValidationReport, GeneratedDecoy,
};
use super::registry::DecoyRegistry;
use super::validation::DecoyValidator;
use crate::architecture::ArchitectureReport;
use crate::errors::{BrahmError, Result};

/// Bundle returned after generating and staging Stage 03 decoy deliverables.
#[derive(Debug, Clone)]
pub struct DecoyGenerationResult {
    pub registry: DecoyRegistry,
    pub manifest: DecoyManifest,
    pub validation: DecoyValidationReport,
    pub generated: Vec<GeneratedDecoy>,
    pub registry_path: PathBuf,
    pub manifest_path: PathBuf,
    pub report_path: PathBuf,
    pub decoy_layer_root: PathBuf,
}

/// Orchestrator for BRAHM Stage 03 Decoy Generation Engine.
#[derive(Debug, Default, Clone)]
pub struct DecoyEngine {
    generator: DecoyGenerator,
    validator: DecoyValidator,
}

impl DecoyEngine {
    /// Initialize a new `DecoyEngine`.
    pub fn new() -> Self {
        Self {
            generator: DecoyGenerator::new(),
            validator: DecoyValidator::new(),
        }
    }

    /// Generate, validate, stage into an isolated `decoy_output_dir`, and persist
    /// `BRAHM_DECOY_REGISTRY.json`, `BRAHM_DECOY_MANIFEST.json`, and `BRAHM_DECOY_REPORT.md`.
    ///
    /// Strictly never writes inside `genuine_repo_root`.
    pub fn generate_and_stage(
        &self,
        genuine_repo_root: &Path,
        architecture: &ArchitectureReport,
        profile: DecoyGenerationProfile,
        decoy_output_dir: &Path,
    ) -> Result<DecoyGenerationResult> {
        let canonical_repo =
            fs::canonicalize(genuine_repo_root).unwrap_or_else(|_| genuine_repo_root.to_path_buf());
        let repo_id = generate_repository_id(&canonical_repo, &architecture.repository.name);

        let decoy_layer_root = decoy_output_dir.join("decoy_layer");
        fs::create_dir_all(&decoy_layer_root)?;

        // Guard against accidentally staging directly inside the genuine repository root
        let canon_out =
            fs::canonicalize(decoy_output_dir).unwrap_or_else(|_| decoy_output_dir.to_path_buf());
        if canon_out == canonical_repo {
            return Err(BrahmError::Decoy(
                "Decoy staging output directory cannot be identical to the genuine repository root"
                    .to_string(),
            ));
        }

        // 1. Generate decoys deterministically from Stage 02 architecture
        let mut generated =
            self.generator
                .generate_for_architecture(&canonical_repo, architecture, profile)?;

        // 2. Validate safety, paths, non-secret, non-executing, and fingerprint invariants
        let validation = self.validator.validate(&generated, &canonical_repo);
        if !validation.valid {
            return Err(BrahmError::Decoy(format!(
                "Decoy validation failed with {} error(s)",
                validation.issues.len()
            )));
        }

        // 3. Stage files into isolated decoy_layer directory
        let mut registry = DecoyRegistry::new(
            &repo_id,
            &architecture.repository.name,
            decoy_layer_root.to_string_lossy().replace('\\', "/"),
            profile,
        );

        let mut manifest_entries = Vec::with_capacity(generated.len());

        for item in &mut generated {
            let dest = decoy_layer_root.join(&item.artifact.synthetic_relative_path);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&dest, &item.content)?;
            item.artifact.install_state = DecoyInstallState::Staged;

            manifest_entries.push(DecoyManifestEntry {
                artifact_id: item.artifact.artifact_id.clone(),
                relative_path: item.artifact.synthetic_relative_path.clone(),
                category: item.artifact.artifact_type,
                integrity_fingerprint: item.artifact.integrity_fingerprint.clone(),
                brahm_owned: item.artifact.brahm_owned,
                synthetic_status: item.artifact.synthetic_status,
                canary_token: item.artifact.canary_token.clone(),
            });

            registry.upsert(item.artifact.clone());
        }

        // 4. Write persistent Stage 03 deliverables
        let registry_path = decoy_output_dir.join("BRAHM_DECOY_REGISTRY.json");
        registry.save_to_path(&registry_path)?;

        let manifest = DecoyManifest {
            schema_version: "1.0".to_string(),
            generated_at: Utc::now(),
            repository_id: repo_id,
            total_artifacts: manifest_entries.len(),
            production_build_excluded: true,
            entries: manifest_entries,
        };
        let manifest_path = decoy_output_dir.join("BRAHM_DECOY_MANIFEST.json");
        fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;

        let report_md = render_decoy_report_markdown(&registry, &validation, &decoy_layer_root);
        let report_path = decoy_output_dir.join("BRAHM_DECOY_REPORT.md");
        fs::write(&report_path, report_md)?;

        info!(
            "Stage 03 generated {} decoy artifact(s) under '{}' (profile: {})",
            registry.artifacts().len(),
            decoy_layer_root.display(),
            profile
        );

        Ok(DecoyGenerationResult {
            registry,
            manifest,
            validation,
            generated,
            registry_path,
            manifest_path,
            report_path,
            decoy_layer_root,
        })
    }
}

/// Render Stage 03 Markdown deliverable `BRAHM_DECOY_REPORT.md`.
pub fn render_decoy_report_markdown(
    registry: &DecoyRegistry,
    validation: &DecoyValidationReport,
    decoy_layer_root: &Path,
) -> String {
    let data = registry.data();
    let mut md = String::new();
    md.push_str("# BRAHM Stage 03 — Decoy Generation & Registry Report\n\n");
    md.push_str("> **Notice:** All artifacts listed below are explicitly synthetic and isolated from genuine application builds.\n\n");
    md.push_str("## 1. Summary\n\n");
    md.push_str("| Field | Value |\n");
    md.push_str("|---|---|\n");
    md.push_str(&format!(
        "| **Repository** | `{}` (`{}`) |\n",
        data.repository_name, data.repository_id
    ));
    md.push_str(&format!(
        "| **Generation Profile** | `{}` |\n",
        data.generation_profile
    ));
    md.push_str(&format!(
        "| **Isolated Staging Root** | `{}` |\n",
        decoy_layer_root.to_string_lossy().replace('\\', "/")
    ));
    md.push_str(&format!(
        "| **Total Artifacts** | `{}` |\n",
        data.artifacts.len()
    ));
    md.push_str(&format!(
        "| **Validation Passed** | `{}` ({} checked) |\n\n",
        validation.valid, validation.checked_count
    ));

    md.push_str("## 2. Generated Synthetic Artifacts\n\n");
    if data.artifacts.is_empty() {
        md.push_str("_No decoy artifacts generated (profile may be `disabled`)._\n\n");
    } else {
        md.push_str("| Artifact ID | Category | Relative Path | Canary Marker | Related Node |\n");
        md.push_str("|---|---|---|---|---|\n");
        for art in &data.artifacts {
            let canary = art.canary_token.as_deref().unwrap_or("-");
            let rel_node = art.related_architecture_node.as_deref().unwrap_or("-");
            md.push_str(&format!(
                "| `{}` | `{}` | `{}` | `{}` | `{}` |\n",
                art.artifact_id, art.artifact_type, art.synthetic_relative_path, canary, rel_node
            ));
        }
        md.push('\n');
    }

    if !validation.issues.is_empty() {
        md.push_str("## 3. Validation Notices\n\n");
        for issue in &validation.issues {
            md.push_str(&format!(
                "- [{:?}] `{}` (`{}`): {}\n",
                issue.severity, issue.artifact_id, issue.check, issue.message
            ));
        }
        md.push('\n');
    }

    md
}
