//! Deterministic Deception Policy Engine (Milestone 04.3).
//!
//! Evaluates registered synthetic decoy artifacts and generates auditable inclusion decisions
//! for controlled inspection workspaces while strictly forbidding secret exposure or production leaks.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::deception::decoy::{DecoyArtifact, DecoyCategory, DecoyInstallState};
use crate::errors::{BrahmError, Result};

/// High-level policy tier controlling synthetic exposure in an inspection mirror.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DeceptionPolicyProfile {
    /// Selected documentation and registered canary anchors only.
    Conservative,
    /// Balanced synthetic documentation, inert source stubs, config examples, and canaries.
    #[default]
    Standard,
    /// Full research suite of all validated synthetic artifacts.
    Research,
}

impl std::fmt::Display for DeceptionPolicyProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Conservative => write!(f, "conservative"),
            Self::Standard => write!(f, "standard"),
            Self::Research => write!(f, "research"),
        }
    }
}

/// Auditable deception policy configuration (`BRAHM_DECEPTION_POLICY.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeceptionPolicy {
    pub policy_id: String,
    pub version: String,
    pub profile: DeceptionPolicyProfile,
    pub priority: u32,
    pub include_categories: BTreeSet<DecoyCategory>,
    pub exclude_categories: BTreeSet<DecoyCategory>,
    pub include_artifact_ids: BTreeSet<String>,
    pub exclude_artifact_ids: BTreeSet<String>,
    pub include_docs_projection: bool,
    pub include_synthetic_configs: bool,
    pub register_canaries: bool,
    /// Security invariant: MUST always be false.
    pub allow_secret_exposure: bool,
    /// Security invariant: MUST always be false.
    pub allow_production_deployment: bool,
    /// Optional cap on maximum artifacts injected into a single mirror session.
    pub max_artifacts: Option<usize>,
}

impl DeceptionPolicy {
    /// Create a canonical policy preset for a given `DeceptionPolicyProfile`.
    pub fn for_profile(profile: DeceptionPolicyProfile) -> Self {
        match profile {
            DeceptionPolicyProfile::Conservative => {
                let mut include_categories = BTreeSet::new();
                include_categories.insert(DecoyCategory::SyntheticDocumentation);
                include_categories.insert(DecoyCategory::CanaryArtifact);

                let mut exclude_categories = BTreeSet::new();
                exclude_categories.insert(DecoyCategory::SyntheticSource);
                exclude_categories.insert(DecoyCategory::SyntheticConfiguration);
                exclude_categories.insert(DecoyCategory::SyntheticDependency);

                Self {
                    policy_id: "pol_conservative_v1".to_string(),
                    version: "1.0.0".to_string(),
                    profile,
                    priority: 100,
                    include_categories,
                    exclude_categories,
                    include_artifact_ids: BTreeSet::new(),
                    exclude_artifact_ids: BTreeSet::new(),
                    include_docs_projection: true,
                    include_synthetic_configs: false,
                    register_canaries: true,
                    allow_secret_exposure: false,
                    allow_production_deployment: false,
                    max_artifacts: Some(4),
                }
            }
            DeceptionPolicyProfile::Standard => {
                let mut include_categories = BTreeSet::new();
                include_categories.insert(DecoyCategory::SyntheticDocumentation);
                include_categories.insert(DecoyCategory::SyntheticSource);
                include_categories.insert(DecoyCategory::SyntheticConfiguration);
                include_categories.insert(DecoyCategory::CanaryArtifact);

                let mut exclude_categories = BTreeSet::new();
                exclude_categories.insert(DecoyCategory::SyntheticDependency);

                Self {
                    policy_id: "pol_standard_v1".to_string(),
                    version: "1.0.0".to_string(),
                    profile,
                    priority: 200,
                    include_categories,
                    exclude_categories,
                    include_artifact_ids: BTreeSet::new(),
                    exclude_artifact_ids: BTreeSet::new(),
                    include_docs_projection: true,
                    include_synthetic_configs: true,
                    register_canaries: true,
                    allow_secret_exposure: false,
                    allow_production_deployment: false,
                    max_artifacts: Some(16),
                }
            }
            DeceptionPolicyProfile::Research => {
                let mut include_categories = BTreeSet::new();
                include_categories.insert(DecoyCategory::SyntheticDocumentation);
                include_categories.insert(DecoyCategory::SyntheticSource);
                include_categories.insert(DecoyCategory::SyntheticConfiguration);
                include_categories.insert(DecoyCategory::CanaryArtifact);
                include_categories.insert(DecoyCategory::SyntheticDependency);

                Self {
                    policy_id: "pol_research_v1".to_string(),
                    version: "1.0.0".to_string(),
                    profile,
                    priority: 300,
                    include_categories,
                    exclude_categories: BTreeSet::new(),
                    include_artifact_ids: BTreeSet::new(),
                    exclude_artifact_ids: BTreeSet::new(),
                    include_docs_projection: true,
                    include_synthetic_configs: true,
                    register_canaries: true,
                    allow_secret_exposure: false,
                    allow_production_deployment: false,
                    max_artifacts: None,
                }
            }
        }
    }

    /// Enforce mandatory ethical & safety invariants on a policy object.
    pub fn validate(&self) -> Result<()> {
        if self.policy_id.trim().is_empty() {
            return Err(BrahmError::PolicyViolation(
                "Policy ID cannot be empty".to_string(),
            ));
        }
        if self.allow_secret_exposure {
            return Err(BrahmError::PolicyViolation(
                "Deception policy violation: allow_secret_exposure must always be false"
                    .to_string(),
            ));
        }
        if self.allow_production_deployment {
            return Err(BrahmError::PolicyViolation(
                "Deception policy violation: allow_production_deployment must always be false"
                    .to_string(),
            ));
        }
        Ok(())
    }
}

/// Auditable decision made by the PolicyEngine for an artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub artifact_id: String,
    pub relative_path: String,
    pub category: DecoyCategory,
    pub included: bool,
    pub rule: String,
    pub reason: String,
}

/// Complete auditable policy evaluation outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyEvaluationResult {
    pub policy_id: String,
    pub profile: DeceptionPolicyProfile,
    pub included_ids: Vec<String>,
    pub decisions: Vec<PolicyDecision>,
}

/// Deterministic rule-based Deception Policy Engine.
#[derive(Debug, Default, Clone)]
pub struct DeceptionPolicyEngine;

impl DeceptionPolicyEngine {
    pub fn new() -> Self {
        Self
    }

    /// Return built-in policy presets.
    pub fn list_presets(&self) -> Vec<DeceptionPolicy> {
        vec![
            DeceptionPolicy::for_profile(DeceptionPolicyProfile::Conservative),
            DeceptionPolicy::for_profile(DeceptionPolicyProfile::Standard),
            DeceptionPolicy::for_profile(DeceptionPolicyProfile::Research),
        ]
    }

    /// Evaluate every artifact in `candidates` against `policy`.
    pub fn evaluate(
        &self,
        policy: &DeceptionPolicy,
        candidates: &[DecoyArtifact],
    ) -> Result<PolicyEvaluationResult> {
        policy.validate()?;

        let mut decisions = Vec::with_capacity(candidates.len());
        let mut included_ids = Vec::new();

        for art in candidates {
            let (included, rule, reason) = self.evaluate_single(policy, art, included_ids.len());
            if included {
                included_ids.push(art.artifact_id.clone());
            }
            decisions.push(PolicyDecision {
                artifact_id: art.artifact_id.clone(),
                relative_path: art.synthetic_relative_path.clone(),
                category: art.artifact_type,
                included,
                rule,
                reason,
            });
        }

        Ok(PolicyEvaluationResult {
            policy_id: policy.policy_id.clone(),
            profile: policy.profile,
            included_ids,
            decisions,
        })
    }

    fn evaluate_single(
        &self,
        policy: &DeceptionPolicy,
        art: &DecoyArtifact,
        current_count: usize,
    ) -> (bool, String, String) {
        if !art.synthetic_status || !art.brahm_owned {
            return (
                false,
                "ownership_safety_gate".to_string(),
                "Rejected non-synthetic or non-BRAHM-owned entry".to_string(),
            );
        }

        if matches!(
            art.install_state,
            DecoyInstallState::Inactive | DecoyInstallState::Removed
        ) {
            return (
                false,
                "lifecycle_active_gate".to_string(),
                format!(
                    "Artifact state is {:?}; excluded from mirror",
                    art.install_state
                ),
            );
        }

        if policy.exclude_artifact_ids.contains(&art.artifact_id) {
            return (
                false,
                "explicit_exclude_id".to_string(),
                "Explicitly excluded by artifact ID".to_string(),
            );
        }

        if policy.exclude_categories.contains(&art.artifact_type) {
            return (
                false,
                "exclude_category".to_string(),
                format!(
                    "Category '{}' excluded by policy profile '{}'",
                    art.artifact_type, policy.profile
                ),
            );
        }

        if art.artifact_type == DecoyCategory::SyntheticDocumentation
            && !policy.include_docs_projection
        {
            return (
                false,
                "docs_projection_disabled".to_string(),
                "Documentation projection disabled by policy".to_string(),
            );
        }

        if art.artifact_type == DecoyCategory::SyntheticConfiguration
            && !policy.include_synthetic_configs
        {
            return (
                false,
                "synthetic_configs_disabled".to_string(),
                "Synthetic configurations disabled by policy".to_string(),
            );
        }

        if art.artifact_type == DecoyCategory::CanaryArtifact && !policy.register_canaries {
            return (
                false,
                "canary_registration_disabled".to_string(),
                "Canary inclusion disabled by policy".to_string(),
            );
        }

        let allowed = policy.include_artifact_ids.contains(&art.artifact_id)
            || policy.include_categories.contains(&art.artifact_type);

        if !allowed {
            return (
                false,
                "not_in_allowlist".to_string(),
                format!(
                    "Category '{}' not enabled in policy allowlist",
                    art.artifact_type
                ),
            );
        }

        if let Some(cap) = policy.max_artifacts {
            if current_count >= cap {
                return (
                    false,
                    "max_artifacts_cap".to_string(),
                    format!("Reached policy cap of {} artifacts", cap),
                );
            }
        }

        (
            true,
            format!("profile_{}_allow", policy.profile),
            format!(
                "Permitted under policy '{}' ({})",
                policy.policy_id, policy.profile
            ),
        )
    }
}
