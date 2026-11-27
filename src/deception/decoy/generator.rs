//! Architecture-aware deterministic decoy artifact generator.

use chrono::Utc;

use super::identifiers::{
    compute_content_fingerprint, generate_canary_marker, generate_decoy_artifact_id,
    generate_repository_id,
};
use super::models::{
    DecoyArtifact, DecoyGenerationProfile, DecoyInstallState, DecoyMonitoringState, DecoyTemplate,
    GeneratedDecoy,
};
use super::templates::{render_template_body, TemplateContext, TemplateRegistry};
use crate::architecture::ArchitectureReport;
use crate::errors::Result;

/// Generates controlled, architecture-aware synthetic artifacts from Stage 02 output.
#[derive(Debug, Clone)]
pub struct DecoyGenerator {
    templates: TemplateRegistry,
}

impl Default for DecoyGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl DecoyGenerator {
    /// Create a generator initialized with built-in validated templates.
    pub fn new() -> Self {
        Self {
            templates: TemplateRegistry::with_builtin_templates(),
        }
    }

    /// Create a generator with a custom `TemplateRegistry`.
    pub fn with_templates(templates: TemplateRegistry) -> Self {
        Self { templates }
    }

    /// Inspect registered templates.
    pub fn template_registry(&self) -> &TemplateRegistry {
        &self.templates
    }

    /// Generate synthetic decoy artifacts informed by Stage 02 `ArchitectureReport`
    /// and bounded by `DecoyGenerationProfile`.
    pub fn generate_for_architecture(
        &self,
        repo_path: &std::path::Path,
        architecture: &ArchitectureReport,
        profile: DecoyGenerationProfile,
    ) -> Result<Vec<GeneratedDecoy>> {
        if profile == DecoyGenerationProfile::Disabled {
            return Ok(Vec::new());
        }

        let repo_name = &architecture.repository.name;
        let repo_id = generate_repository_id(repo_path, repo_name);
        let selected = self.select_templates_for_architecture(architecture, profile);

        let now = Utc::now();
        let langs_str = if architecture.pattern.languages_parsed.is_empty() {
            "None detected".to_string()
        } else {
            architecture.pattern.languages_parsed.join(", ")
        };
        let frameworks_str = if architecture.pattern.detected_frameworks.is_empty() {
            "Generic".to_string()
        } else {
            architecture.pattern.detected_frameworks.join(", ")
        };

        let mut results = Vec::with_capacity(selected.len());
        for (tpl, arch_node_hint) in selected {
            let artifact_id = generate_decoy_artifact_id(
                &repo_id,
                &tpl.template_id,
                &tpl.version,
                &tpl.default_relative_path,
            );
            let canary_marker = generate_canary_marker(&artifact_id, &repo_id);

            let mut ctx = TemplateContext::new();
            ctx.insert("ARTIFACT_ID".to_string(), artifact_id.clone());
            ctx.insert("TEMPLATE_ID".to_string(), tpl.template_id.clone());
            ctx.insert("TEMPLATE_VERSION".to_string(), tpl.version.clone());
            ctx.insert("REPO_NAME".to_string(), repo_name.clone());
            ctx.insert("REPO_ID".to_string(), repo_id.clone());
            ctx.insert(
                "ARCH_PATTERN".to_string(),
                architecture.pattern.primary_pattern.clone(),
            );
            ctx.insert("LANGUAGES".to_string(), langs_str.clone());
            ctx.insert("FRAMEWORKS".to_string(), frameworks_str.clone());
            ctx.insert("CANARY_MARKER".to_string(), canary_marker.clone());
            ctx.insert("GENERATED_AT".to_string(), now.to_rfc3339());

            let content = render_template_body(&tpl.body_template, &ctx)?;
            let fingerprint = compute_content_fingerprint(&content);

            let filename = std::path::Path::new(&tpl.default_relative_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("decoy.txt")
                .to_string();

            let mut metadata = std::collections::BTreeMap::new();
            metadata.insert("profile".to_string(), profile.to_string());
            if let Some(ref lang) = tpl.target_language {
                metadata.insert("target_language".to_string(), lang.clone());
            }

            let artifact = DecoyArtifact {
                artifact_id,
                artifact_type: tpl.category,
                synthetic_filename: filename,
                synthetic_relative_path: tpl.default_relative_path.clone(),
                template_id: tpl.template_id.clone(),
                template_version: tpl.version.clone(),
                generated_at: now,
                generation_version: env!("CARGO_PKG_VERSION").to_string(),
                related_architecture_node: arch_node_hint,
                synthetic_status: true,
                brahm_owned: true,
                risk_classification: tpl.risk_classification,
                integrity_fingerprint: fingerprint,
                install_state: DecoyInstallState::Generated,
                monitoring_state: DecoyMonitoringState::Registered,
                canary_token: Some(canary_marker),
                metadata,
            };

            results.push(GeneratedDecoy { artifact, content });
        }

        Ok(results)
    }

    /// Deterministically choose templates grounded strictly in observed Stage 02 evidence.
    fn select_templates_for_architecture(
        &self,
        arch: &ArchitectureReport,
        profile: DecoyGenerationProfile,
    ) -> Vec<(&DecoyTemplate, Option<String>)> {
        let mut chosen: Vec<(&DecoyTemplate, Option<String>)> = Vec::new();
        let mut add = |id: &str, hint: Option<String>| {
            if let Some(tpl) = self.templates.get(id) {
                if !chosen
                    .iter()
                    .any(|(existing, _)| existing.template_id == id)
                {
                    chosen.push((tpl, hint));
                }
            }
        };

        // 1. Conservative baseline: 1 canary anchor + 1 synthetic system architecture doc
        add(
            "tpl_canary_anchor",
            Some(format!("pattern:{}", arch.pattern.primary_pattern)),
        );
        add(
            "tpl_doc_system_architecture",
            Some(format!("pattern:{}", arch.pattern.primary_pattern)),
        );

        if profile == DecoyGenerationProfile::Conservative {
            return chosen;
        }

        // 2. Standard profile: add design doc, synthetic dependency manifest, and evidence-backed modules/configs
        add(
            "tpl_doc_internal_design",
            Some("layer:architecture".to_string()),
        );
        add(
            "tpl_dep_synthetic_manifest",
            Some("dependencies:isolated".to_string()),
        );

        for lang in &arch.pattern.languages_parsed {
            match lang.as_str() {
                "Rust" => add("tpl_src_rust_coordinator", Some("lang:Rust".to_string())),
                "TypeScript" | "TSX" => add(
                    "tpl_src_ts_telemetry_bridge",
                    Some(format!("lang:{}", lang)),
                ),
                "JavaScript" => add(
                    "tpl_src_js_legacy_router",
                    Some("lang:JavaScript".to_string()),
                ),
                "Python" => add("tpl_src_py_worker_stub", Some("lang:Python".to_string())),
                "Go" => add("tpl_src_go_middleware", Some("lang:Go".to_string())),
                _ => {}
            }
        }

        let has_api_evidence = arch.metrics.api_route_count > 0
            || arch.pattern.detected_frameworks.iter().any(|f| {
                matches!(
                    f.as_str(),
                    "Express" | "Next.js" | "FastAPI" | "Flask" | "Spring Boot" | "Gin" | "Node.js"
                )
            });

        if has_api_evidence {
            add("tpl_cfg_api_json", Some("layer:api".to_string()));
            add("tpl_cfg_service_yaml", Some("layer:service".to_string()));
        }

        if arch.metrics.data_model_count > 0 {
            add("tpl_cfg_schema_sql", Some("layer:persistence".to_string()));
        }

        if profile == DecoyGenerationProfile::Standard {
            return chosen;
        }

        // 3. Research profile: include full synthetic suite + secondary canary token
        add(
            "tpl_doc_service_dependencies",
            Some("layer:dependencies".to_string()),
        );
        add(
            "tpl_doc_component_overview",
            Some("layer:components".to_string()),
        );
        add(
            "tpl_canary_trace_json",
            Some("canary:secondary".to_string()),
        );
        if has_api_evidence || !arch.pattern.languages_parsed.is_empty() {
            add("tpl_cfg_service_yaml", Some("layer:service".to_string()));
        }

        chosen
    }
}
