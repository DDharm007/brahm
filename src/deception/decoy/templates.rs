//! Deterministic, non-executing template engine for synthetic decoy generation.
//!
//! All template substitutions operate strictly as inert string replacements using `{{KEY}}` markers.
//! Untrusted code evaluation or execution inside templates is strictly impossible.

use std::collections::{BTreeMap, HashMap};

use super::models::{DecoyCategory, DecoyTemplate, RiskClassification};
use crate::errors::{BrahmError, Result};

/// Context map passed into template rendering.
pub type TemplateContext = BTreeMap<String, String>;

/// Registry storing validated decoy templates indexed by `template_id`.
#[derive(Debug, Clone)]
pub struct TemplateRegistry {
    templates: HashMap<String, DecoyTemplate>,
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::with_builtin_templates()
    }
}

impl TemplateRegistry {
    /// Create an empty template registry.
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    /// Create a registry pre-populated with all verified Stage 03 built-in templates.
    pub fn with_builtin_templates() -> Self {
        let mut registry = Self::new();
        for tpl in builtin_templates() {
            // Builtin templates are verified statically
            let _ = registry.register(tpl);
        }
        registry
    }

    /// Register and validate a new template.
    pub fn register(&mut self, template: DecoyTemplate) -> Result<()> {
        validate_template(&template)?;
        self.templates
            .insert(template.template_id.clone(), template);
        Ok(())
    }

    /// Retrieve a template by ID.
    pub fn get(&self, template_id: &str) -> Option<&DecoyTemplate> {
        self.templates.get(template_id)
    }

    /// List all registered templates sorted deterministically by `template_id`.
    pub fn list(&self) -> Vec<&DecoyTemplate> {
        let mut items: Vec<&DecoyTemplate> = self.templates.values().collect();
        items.sort_by(|a, b| a.template_id.cmp(&b.template_id));
        items
    }

    /// Render a template with the supplied context map.
    /// Does not execute code or allow arbitrary expressions.
    pub fn render(&self, template_id: &str, ctx: &TemplateContext) -> Result<String> {
        let tpl = self.templates.get(template_id).ok_or_else(|| {
            BrahmError::Template(format!("Template '{}' is not registered", template_id))
        })?;
        render_template_body(&tpl.body_template, ctx)
    }
}

/// Validate template metadata, path safety, and content constraints.
pub fn validate_template(tpl: &DecoyTemplate) -> Result<()> {
    if tpl.template_id.trim().is_empty() {
        return Err(BrahmError::Template(
            "Template ID cannot be empty".to_string(),
        ));
    }
    if tpl.version.trim().is_empty() {
        return Err(BrahmError::Template(format!(
            "Template '{}' version cannot be empty",
            tpl.template_id
        )));
    }
    if tpl.body_template.trim().is_empty() {
        return Err(BrahmError::Template(format!(
            "Template '{}' body cannot be empty",
            tpl.template_id
        )));
    }
    let rel = tpl.default_relative_path.replace('\\', "/");
    if rel.starts_with('/') || rel.contains("..") || rel.contains(':') {
        return Err(BrahmError::Template(format!(
            "Template '{}' contains unsafe default path '{}'",
            tpl.template_id, tpl.default_relative_path
        )));
    }
    // Disallow risky secret prefixes inside templates
    for forbidden in [
        "AKIA",
        "sk_live_",
        "ghp_",
        "BEGIN RSA PRIVATE KEY",
        "BEGIN OPENSSH PRIVATE KEY",
    ] {
        if tpl.body_template.contains(forbidden) {
            return Err(BrahmError::Template(format!(
                "Template '{}' contains forbidden credential-like pattern '{}'",
                tpl.template_id, forbidden
            )));
        }
    }
    Ok(())
}

/// Deterministically substitute `{{KEY}}` placeholders cleanly without shell/eval features.
pub fn render_template_body(body: &str, ctx: &TemplateContext) -> Result<String> {
    let mut output = body.to_string();
    for (k, v) in ctx {
        let placeholder = format!("{{{{{}}}}}", k);
        output = output.replace(&placeholder, v);
    }
    Ok(output)
}

/// Construct the comprehensive library of deterministic Stage 03 templates.
fn builtin_templates() -> Vec<DecoyTemplate> {
    vec![
        // --- Synthetic Source Files (Inert stubs matching language syntax) ---
        DecoyTemplate {
            template_id: "tpl_src_rust_coordinator".to_string(),
            version: "1.0.0".to_string(),
            category: DecoyCategory::SyntheticSource,
            target_language: Some("Rust".to_string()),
            default_relative_path: "src/synthetic_dispatch_adapter.rs".to_string(),
            risk_classification: RiskClassification::InertLow,
            description: "Inert Rust coordinator stub mirroring modular crate boundaries".to_string(),
            body_template: r#"//! BRAHM-SYNTHETIC-ARTIFACT: {{ARTIFACT_ID}} (Inert Decoy Module)
//! Template: {{TEMPLATE_ID}} v{{TEMPLATE_VERSION}}
//! Target Context: {{REPO_NAME}} ({{ARCH_PATTERN}})
//! Note: This file is an isolated synthetic artifact for controlled inspection studies.

/// Synthetic inspection descriptor (non-functional, zero side effects).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntheticRouteDescriptor {
    pub marker_id: &'static str,
    pub canary_ref: &'static str,
    pub domain: &'static str,
}

/// Returns inert synthetic descriptor for inspection visibility.
pub fn synthetic_descriptor() -> SyntheticRouteDescriptor {
    SyntheticRouteDescriptor {
        marker_id: "{{ARTIFACT_ID}}",
        canary_ref: "{{CANARY_MARKER}}",
        domain: "{{ARCH_PATTERN}}",
    }
}
"#
            .to_string(),
        },
        DecoyTemplate {
            template_id: "tpl_src_ts_telemetry_bridge".to_string(),
            version: "1.0.0".to_string(),
            category: DecoyCategory::SyntheticSource,
            target_language: Some("TypeScript".to_string()),
            default_relative_path: "src/syntheticTelemetryBridge.ts".to_string(),
            risk_classification: RiskClassification::InertLow,
            description: "Inert TypeScript telemetry interface module".to_string(),
            body_template: r#"/**
 * BRAHM-SYNTHETIC-ARTIFACT: {{ARTIFACT_ID}}
 * Template: {{TEMPLATE_ID}} v{{TEMPLATE_VERSION}}
 * Controlled Inspection Target: {{REPO_NAME}}
 */

export interface SyntheticTraceEnvelope {
    readonly artifactId: string;
    readonly canaryReference: string;
    readonly syntheticLayer: true;
}

export function createSyntheticTraceEnvelope(): SyntheticTraceEnvelope {
    return {
        artifactId: "{{ARTIFACT_ID}}",
        canaryReference: "{{CANARY_MARKER}}",
        syntheticLayer: true,
    };
}
"#
            .to_string(),
        },
        DecoyTemplate {
            template_id: "tpl_src_js_legacy_router".to_string(),
            version: "1.0.0".to_string(),
            category: DecoyCategory::SyntheticSource,
            target_language: Some("JavaScript".to_string()),
            default_relative_path: "src/syntheticRouterStub.js".to_string(),
            risk_classification: RiskClassification::InertLow,
            description: "Inert CommonJS route descriptor for JS backends".to_string(),
            body_template: r#"// BRAHM-SYNTHETIC-ARTIFACT: {{ARTIFACT_ID}}
// Template: {{TEMPLATE_ID}} v{{TEMPLATE_VERSION}}

const SYNTHETIC_METADATA = Object.freeze({
    artifactId: "{{ARTIFACT_ID}}",
    canaryMarker: "{{CANARY_MARKER}}",
    synthetic: true,
    repository: "{{REPO_NAME}}"
});

function getSyntheticRouteMetadata() {
    return SYNTHETIC_METADATA;
}

module.exports = { SYNTHETIC_METADATA, getSyntheticRouteMetadata };
"#
            .to_string(),
        },
        DecoyTemplate {
            template_id: "tpl_src_py_worker_stub".to_string(),
            version: "1.0.0".to_string(),
            category: DecoyCategory::SyntheticSource,
            target_language: Some("Python".to_string()),
            default_relative_path: "services/synthetic_cache_adapter.py".to_string(),
            risk_classification: RiskClassification::InertLow,
            description: "Inert Python typed stub module".to_string(),
            body_template: r#""""BRAHM-SYNTHETIC-ARTIFACT: {{ARTIFACT_ID}}
Template: {{TEMPLATE_ID}} v{{TEMPLATE_VERSION}}
Inert inspection artifact for controlled evaluation ({{REPO_NAME}}).
"""

from dataclasses import dataclass


@dataclass(frozen=True)
class SyntheticInspectionDescriptor:
    artifact_id: str = "{{ARTIFACT_ID}}"
    canary_marker: str = "{{CANARY_MARKER}}"
    synthetic: bool = True


def get_synthetic_descriptor() -> SyntheticInspectionDescriptor:
    return SyntheticInspectionDescriptor()
"#
            .to_string(),
        },
        DecoyTemplate {
            template_id: "tpl_src_go_middleware".to_string(),
            version: "1.0.0".to_string(),
            category: DecoyCategory::SyntheticSource,
            target_language: Some("Go".to_string()),
            default_relative_path: "internal/synthetic_trace.go".to_string(),
            risk_classification: RiskClassification::InertLow,
            description: "Inert Go descriptor struct package".to_string(),
            body_template: r#"// BRAHM-SYNTHETIC-ARTIFACT: {{ARTIFACT_ID}}
package internal

// SyntheticEnvelope identifies an inert decoy inspection module.
type SyntheticEnvelope struct {
	ArtifactID   string
	CanaryMarker string
	Synthetic    bool
}

// GetSyntheticEnvelope returns static metadata with zero side effects.
func GetSyntheticEnvelope() SyntheticEnvelope {
	return SyntheticEnvelope{
		ArtifactID:   "{{ARTIFACT_ID}}",
		CanaryMarker: "{{CANARY_MARKER}}",
		Synthetic:    true,
	}
}
"#
            .to_string(),
        },
        // --- Synthetic Architecture Documentation ---
        DecoyTemplate {
            template_id: "tpl_doc_system_architecture".to_string(),
            version: "1.0.0".to_string(),
            category: DecoyCategory::SyntheticDocumentation,
            target_language: None,
            default_relative_path: "docs/SYSTEM_ARCHITECTURE.md".to_string(),
            risk_classification: RiskClassification::MonitoredMedium,
            description: "Synthetic system architecture document for controlled inspection layer".to_string(),
            body_template: r#"# System Architecture Overview (Inspection View)

> **BRAHM Synthetic Artifact ID:** `{{ARTIFACT_ID}}`  
> **Canary Marker:** `{{CANARY_MARKER}}`  
> **Template:** `{{TEMPLATE_ID}}` (`v{{TEMPLATE_VERSION}}`)

## Overview

This document describes an inspection-layer architectural projection for **{{REPO_NAME}}** (`{{ARCH_PATTERN}}`).

- **Detected Ecosystem Languages:** {{LANGUAGES}}
- **Observed Frameworks:** {{FRAMEWORKS}}
- **Synthetic Note:** This file belongs exclusively to the BRAHM synthetic inspection layer and must never replace verified `BRAHM_ARCHITECTURE.md` telemetry.
"#
            .to_string(),
        },
        DecoyTemplate {
            template_id: "tpl_doc_internal_design".to_string(),
            version: "1.0.0".to_string(),
            category: DecoyCategory::SyntheticDocumentation,
            target_language: None,
            default_relative_path: "docs/INTERNAL_DESIGN.md".to_string(),
            risk_classification: RiskClassification::MonitoredMedium,
            description: "Synthetic internal module design notes".to_string(),
            body_template: r#"# Internal Component Design Notes

> **BRAHM Synthetic Artifact ID:** `{{ARTIFACT_ID}}`  
> **Canary Reference:** `{{CANARY_MARKER}}`

## Component Layout

- **Repository:** `{{REPO_NAME}}`
- **Synthetic Coordination Anchor:** `{{ARTIFACT_ID}}`
- **Classification:** Controlled Synthetic Documentation (`synthetic_status = true`)
"#
            .to_string(),
        },
        DecoyTemplate {
            template_id: "tpl_doc_service_dependencies".to_string(),
            version: "1.0.0".to_string(),
            category: DecoyCategory::SyntheticDocumentation,
            target_language: None,
            default_relative_path: "docs/SERVICE_DEPENDENCIES.md".to_string(),
            risk_classification: RiskClassification::MonitoredMedium,
            description: "Synthetic service dependency notes".to_string(),
            body_template: r#"# Service Dependency Matrix (Synthetic Inspection Reference)

> **Artifact ID:** `{{ARTIFACT_ID}}`  
> **Canary Marker:** `{{CANARY_MARKER}}`

## Declared Layers

- **Detected Architecture:** {{ARCH_PATTERN}}
- **Synthetic Boundary Endpoint:** `https://telemetry.invalid/v1/inspection/{{CANARY_MARKER}}` (RFC 2606 non-routable)
"#
            .to_string(),
        },
        DecoyTemplate {
            template_id: "tpl_doc_component_overview".to_string(),
            version: "1.0.0".to_string(),
            category: DecoyCategory::SyntheticDocumentation,
            target_language: None,
            default_relative_path: "docs/COMPONENT_OVERVIEW.md".to_string(),
            risk_classification: RiskClassification::MonitoredMedium,
            description: "Synthetic component overview catalog".to_string(),
            body_template: r#"# Component Overview

> **Artifact ID:** `{{ARTIFACT_ID}}` | **Canary:** `{{CANARY_MARKER}}`

- **Repository:** `{{REPO_NAME}}`
- **Detected Frameworks:** {{FRAMEWORKS}}
- **Synthetic Layer Indicator:** This file is part of an isolated BRAHM inspection environment.
"#
            .to_string(),
        },
        // --- Synthetic Configuration Artifacts (Inert, zero credentials) ---
        DecoyTemplate {
            template_id: "tpl_cfg_service_yaml".to_string(),
            version: "1.0.0".to_string(),
            category: DecoyCategory::SyntheticConfiguration,
            target_language: None,
            default_relative_path: "config/service.decoy.yaml".to_string(),
            risk_classification: RiskClassification::MonitoredMedium,
            description: "Inert synthetic YAML service config without functional credentials".to_string(),
            body_template: r#"# BRAHM-SYNTHETIC-ARTIFACT: {{ARTIFACT_ID}}
service:
  name: "{{REPO_NAME}}-inspection-stub"
  synthetic: true
  canary_marker: "{{CANARY_MARKER}}"
  endpoint: "https://decoy-endpoint.invalid/api"
  timeout_ms: 5000
"#
            .to_string(),
        },
        DecoyTemplate {
            template_id: "tpl_cfg_api_json".to_string(),
            version: "1.0.0".to_string(),
            category: DecoyCategory::SyntheticConfiguration,
            target_language: None,
            default_relative_path: "config/api.decoy.json".to_string(),
            risk_classification: RiskClassification::MonitoredMedium,
            description: "Inert synthetic JSON API route descriptor".to_string(),
            body_template: r#"{
  "_brahm_synthetic": true,
  "artifact_id": "{{ARTIFACT_ID}}",
  "canary_marker": "{{CANARY_MARKER}}",
  "service": "{{REPO_NAME}}",
  "routes": [
    {
      "method": "GET",
      "path": "/_synthetic/health",
      "synthetic": true
    }
  ]
}
"#
            .to_string(),
        },
        DecoyTemplate {
            template_id: "tpl_cfg_schema_sql".to_string(),
            version: "1.0.0".to_string(),
            category: DecoyCategory::SyntheticConfiguration,
            target_language: None,
            default_relative_path: "db/schema.decoy.sql".to_string(),
            risk_classification: RiskClassification::InertLow,
            description: "Inert synthetic SQL inspection schema example".to_string(),
            body_template: r#"-- BRAHM-SYNTHETIC-ARTIFACT: {{ARTIFACT_ID}}
-- Canary Marker: {{CANARY_MARKER}}
-- Inert synthetic schema example (never executed against genuine DBs).

CREATE TABLE IF NOT EXISTS synthetic_inspection_audit (
    id INTEGER PRIMARY KEY,
    artifact_id TEXT NOT NULL,
    canary_marker TEXT NOT NULL
);
"#
            .to_string(),
        },
        // --- Canary Artifacts ---
        DecoyTemplate {
            template_id: "tpl_canary_anchor".to_string(),
            version: "1.0.0".to_string(),
            category: DecoyCategory::CanaryArtifact,
            target_language: None,
            default_relative_path: "canary/BRAHM_CANARY_ANCHOR.md".to_string(),
            risk_classification: RiskClassification::CanaryHighSignal,
            description: "Registered high-signal inert canary marker document".to_string(),
            body_template: r#"# Registered Inspection Telemetry Anchor

- **Synthetic Artifact ID:** `{{ARTIFACT_ID}}`
- **Canary Identifier:** `{{CANARY_MARKER}}`
- **Repository:** `{{REPO_NAME}}`
- **Generated:** `{{GENERATED_AT}}`
- **Notice:** This artifact contains no credentials or private keys. Accessing this file within an instrumented BRAHM inspection sandbox records an auditable canary telemetry event.
"#
            .to_string(),
        },
        DecoyTemplate {
            template_id: "tpl_canary_trace_json".to_string(),
            version: "1.0.0".to_string(),
            category: DecoyCategory::CanaryArtifact,
            target_language: None,
            default_relative_path: "canary/inspection_canary_token.json".to_string(),
            risk_classification: RiskClassification::CanaryHighSignal,
            description: "Machine-readable non-secret canary anchor".to_string(),
            body_template: r#"{
  "brahm_synthetic": true,
  "artifact_id": "{{ARTIFACT_ID}}",
  "canary_marker": "{{CANARY_MARKER}}",
  "repository_id": "{{REPO_ID}}",
  "non_credential_notice": "Inert monitoring token incapable of granting external access"
}
"#
            .to_string(),
        },
        // --- Synthetic Dependency Metadata ---
        DecoyTemplate {
            template_id: "tpl_dep_synthetic_manifest".to_string(),
            version: "1.0.0".to_string(),
            category: DecoyCategory::SyntheticDependency,
            target_language: None,
            default_relative_path: "manifests/synthetic_dependencies.decoy.json".to_string(),
            risk_classification: RiskClassification::InertLow,
            description: "Isolated synthetic dependency catalog never merged into Cargo.toml or package.json".to_string(),
            body_template: r#"{
  "brahm_synthetic": true,
  "artifact_id": "{{ARTIFACT_ID}}",
  "canary_marker": "{{CANARY_MARKER}}",
  "isolated_from_production": true,
  "observed_languages": "{{LANGUAGES}}",
  "synthetic_packages": [
    {
      "name": "brahm-synthetic-inspection-shim",
      "version": "0.0.0-synthetic",
      "purpose": "Controlled deception baseline marker"
    }
  ]
}
"#
            .to_string(),
        },
    ]
}
