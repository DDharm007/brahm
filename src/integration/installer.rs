//! Safe Protection Installation & Rollback Workflow (Milestone 05.6 & 05.7).
//!
//! Coordinates connecting repositories, drafting explicit protection plans, verifying
//! production exclusions, enforcing explicit approval before installation, and performing
//! ownership-verified dry-run/executed rollback preserving genuine user assets.

use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;
use uuid::Uuid;

use super::agent::{InspectionAgent, LocalTestAgentAdapter};
use super::docs_gen::{generate_inspection_guide, generate_security_doc};
use super::input::RepositoryInputManager;
use super::models::{
    AgentManifest, ProtectionPlan, ProtectionProfile, RepositoryRecord, RollbackReport,
};
use crate::config::BrahmConfig;
use crate::deception::decoy::{DecoyEngine, DecoyGenerationProfile, DecoyManifest, DecoyRegistry};
use crate::deception::{
    DeceptionPolicy, DeceptionPolicyProfile, GenuineLayer, InspectionSession,
    InspectionSessionManager, ProductionIntegrityGuard,
};
use crate::errors::{BrahmError, Result};
use crate::monitoring::{
    EventSeverity, EventSource, EventStore, EventType, IntegrityMonitor, IntegrityRecord,
    MonitoringEvent, SqliteEventStore,
};
use crate::reporting::architecture_md::generate_architecture_markdown;

/// Bundle returned after running `ProtectionInstaller::install`.
pub struct ProtectionInstallOutcome {
    pub record: RepositoryRecord,
    pub plan: ProtectionPlan,
    pub session: Option<InspectionSession>,
    pub agent_manifest: AgentManifest,
    pub install_report_path: PathBuf,
    pub config_backup_path: Option<PathBuf>,
}

/// Summary returned by the one-command `brahm apply` / `ProtectionInstaller::apply` workflow.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApplyOutcome {
    pub dry_run: bool,
    pub repository_id: String,
    pub repository_name: String,
    pub repository_path: PathBuf,
    pub state_dir: PathBuf,
    pub profile: ProtectionProfile,
    pub baseline_fingerprint: String,
    pub session_id: Option<String>,
    pub inspection_workspace: Option<PathBuf>,
    pub decoy_count: usize,
    pub canary_count: usize,
    pub integrity_verified: Option<bool>,
    pub architecture_report_path: Option<PathBuf>,
    pub install_report_path: Option<PathBuf>,
    pub plan: ProtectionPlan,
}

/// Load the stored genuine repository BLAKE3 baseline sealed at connect/install time.
pub fn load_stored_baseline(state_dir: &Path) -> Result<GenuineLayer> {
    let baseline_path = state_dir.join("BRAHM_BASELINE.json");
    if !baseline_path.is_file() {
        return Err(BrahmError::Integration(format!(
            "Stored genuine baseline '{}' not found. Connect or install protection before running integrity verification.",
            baseline_path.display()
        )));
    }
    let raw = fs::read_to_string(&baseline_path)?;
    let baseline: GenuineLayer = serde_json::from_str(&raw)?;
    Ok(baseline)
}

/// Manages safe installation, configuration, and rollback of BRAHM protections.
#[derive(Debug, Default, Clone)]
pub struct ProtectionInstaller {
    input_mgr: RepositoryInputManager,
    decoy_engine: DecoyEngine,
    integrity_guard: ProductionIntegrityGuard,
}

impl ProtectionInstaller {
    pub fn new() -> Self {
        Self {
            input_mgr: RepositoryInputManager::new(),
            decoy_engine: DecoyEngine::new(),
            integrity_guard: ProductionIntegrityGuard::new(),
        }
    }

    /// Generate a preview plan (`BRAHM_PROTECTION_PLAN.json`) without mutating protection state.
    pub fn plan(
        &self,
        repo_path: &Path,
        state_dir: &Path,
        target_profile: ProtectionProfile,
    ) -> Result<ProtectionPlan> {
        let conn = self.input_mgr.connect_local(repo_path, state_dir, None)?;
        let current_profile = conn.record.active_profile;

        let (will_generate_decoys, will_create_mirror, steps) = match target_profile {
            ProtectionProfile::AnalysisOnly => (
                false,
                false,
                vec![
                    "Verify repository structure via Stage 01 scanner".to_string(),
                    "Generate Stage 02 architecture baseline".to_string(),
                    "Capture BLAKE3 genuine baseline fingerprint".to_string(),
                ],
            ),
            ProtectionProfile::ResearchMode => (
                true,
                true,
                vec![
                    "Run Stage 01 & Stage 02 baseline analysis".to_string(),
                    "Generate Research-tier synthetic decoys & canaries in isolated staging"
                        .to_string(),
                    "Prepare isolated controlled inspection workspace (no genuine mutations)"
                        .to_string(),
                    "Validate production build exclusion invariants".to_string(),
                ],
            ),
            ProtectionProfile::ControlledInspection => (
                true,
                true,
                vec![
                    "Run Stage 01 & Stage 02 baseline analysis".to_string(),
                    "Generate Standard-tier decoys & canary anchors in isolated staging"
                        .to_string(),
                    "Assemble isolated inspection mirror with deception policies".to_string(),
                    "Register LocalTestAgentAdapter inspection contract".to_string(),
                ],
            ),
            ProtectionProfile::MonitoringMode => (
                false,
                false,
                vec![
                    "Load existing decoy registry & baseline fingerprint".to_string(),
                    "Enable canary detection and integrity drift monitoring".to_string(),
                    "Verify zero genuine file contamination".to_string(),
                ],
            ),
        };

        // Always validate baseline isolation before approving plan
        let empty_manifest = DecoyManifest {
            schema_version: "1.0".to_string(),
            generated_at: Utc::now(),
            repository_id: conn.record.repository_id.clone(),
            total_artifacts: 0,
            production_build_excluded: true,
            entries: Vec::new(),
        };
        let integrity = self
            .integrity_guard
            .verify_production_isolation(&conn.record.local_path, &empty_manifest)?;

        let plan = ProtectionPlan {
            schema_version: "1.0".to_string(),
            plan_id: format!("plan_{}", &Uuid::new_v4().simple().to_string()[..12]),
            generated_at: Utc::now(),
            repository_id: conn.record.repository_id.clone(),
            repository_name: conn.record.name.clone(),
            current_profile,
            target_profile,
            proposed_steps: steps,
            will_generate_decoys,
            will_create_mirror,
            production_exclusions_verified: integrity.production_isolated,
            requires_explicit_approval: true,
        };

        fs::create_dir_all(state_dir)?;
        fs::write(
            state_dir.join("BRAHM_PROTECTION_PLAN.json"),
            serde_json::to_string_pretty(&plan)?,
        )?;

        Ok(plan)
    }

    /// Execute installation workflow. Fails safely if `approved == false`.
    pub fn install(
        &self,
        repo_path: &Path,
        state_dir: &Path,
        target_profile: ProtectionProfile,
        approved: bool,
    ) -> Result<ProtectionInstallOutcome> {
        let plan = self.plan(repo_path, state_dir, target_profile)?;

        if !approved {
            return Err(BrahmError::Unauthorized(format!(
                "Installation blocked: explicit approval is required before activating protection profile '{}' (plan saved to {})",
                target_profile,
                state_dir.join("BRAHM_PROTECTION_PLAN.json").display()
            )));
        }

        if !plan.production_exclusions_verified {
            return Err(BrahmError::Deception(
                "Installation blocked: target repository failed pre-flight production isolation check"
                    .to_string(),
            ));
        }

        let mut conn = self.input_mgr.connect_local(repo_path, state_dir, None)?;
        let decoys_dir = state_dir.join("decoys");
        let sessions_dir = state_dir.join("sessions");

        let mut active_session: Option<InspectionSession> = None;
        let mut agent_sessions = Vec::new();
        let adapter = LocalTestAgentAdapter::new();

        // Write verified Stage 02 Architecture doc in state_dir
        let _ = generate_architecture_markdown(
            &conn.architecture.report,
            Some(&state_dir.join("BRAHM_ARCHITECTURE.md")),
        )?;

        // Generate decoys & mirror if required by profile & persist telemetry refs
        let store = SqliteEventStore::open(state_dir.join("brahm_monitoring.db"))?;
        let corr_id = format!("install_{}", plan.plan_id);

        if plan.will_generate_decoys {
            let decoy_prof = match target_profile {
                ProtectionProfile::ResearchMode => DecoyGenerationProfile::Research,
                _ => DecoyGenerationProfile::Standard,
            };
            let decoy_res = self.decoy_engine.generate_and_stage(
                &conn.record.local_path,
                &conn.architecture.report,
                decoy_prof,
                &decoys_dir,
            )?;
            conn.record.decoy_registry_path = Some(decoy_res.registry_path.clone());

            for art in decoy_res.registry.artifacts() {
                store.record_decoy_ref(&conn.record.repository_id, art)?;
                let mut meta = std::collections::BTreeMap::new();
                meta.insert(
                    "relative_path".to_string(),
                    art.synthetic_relative_path.clone(),
                );
                store.record_event(&MonitoringEvent {
                    event_id: format!("evt_{}", &Uuid::new_v4().simple().to_string()[..12]),
                    event_type: EventType::DecoyCreated,
                    repository_id: conn.record.repository_id.clone(),
                    session_id: None,
                    artifact_id: Some(art.artifact_id.clone()),
                    timestamp: Utc::now(),
                    source: EventSource::DecoyEngine,
                    severity: EventSeverity::Informational,
                    correlation_id: corr_id.clone(),
                    metadata: meta,
                })?;
            }

            if plan.will_create_mirror {
                let pol_prof = match target_profile {
                    ProtectionProfile::ResearchMode => DeceptionPolicyProfile::Research,
                    _ => DeceptionPolicyProfile::Standard,
                };
                let policy = DeceptionPolicy::for_profile(pol_prof);
                let sess_mgr = InspectionSessionManager::new(&sessions_dir);
                let outcome = sess_mgr.create_and_prepare(
                    &conn.record.local_path,
                    &decoy_res.registry,
                    &decoy_res.decoy_layer_root,
                    &policy,
                    3600,
                )?;
                let active_outcome_sess = sess_mgr.activate_session(&outcome.session.session_id)?;
                store.record_session_ref(&active_outcome_sess)?;
                store.record_event(&MonitoringEvent {
                    event_id: format!("evt_{}", &Uuid::new_v4().simple().to_string()[..12]),
                    event_type: EventType::InspectionSessionStarted,
                    repository_id: conn.record.repository_id.clone(),
                    session_id: Some(active_outcome_sess.session_id.clone()),
                    artifact_id: None,
                    timestamp: Utc::now(),
                    source: EventSource::PolicyEngine,
                    severity: EventSeverity::Informational,
                    correlation_id: corr_id,
                    metadata: std::collections::BTreeMap::new(),
                })?;

                let agent_sess = adapter.prepare_session(&active_outcome_sess)?;
                agent_sessions.push(agent_sess);
                conn.record.active_session_id = Some(active_outcome_sess.session_id.clone());
                active_session = Some(active_outcome_sess);
            }
        }

        conn.record.active_profile = target_profile;
        conn.record.updated_at = Utc::now();

        // Update persisted BRAHM_REPOSITORY_CONFIG.json
        fs::write(
            state_dir.join("BRAHM_REPOSITORY_CONFIG.json"),
            serde_json::to_string_pretty(&conn.record)?,
        )?;

        // Update/backup state_dir config snapshot cleanly
        let mut cfg = BrahmConfig::find_or_default(&conn.record.local_path).0;
        cfg.integration.protection_profile = target_profile;
        let state_cfg_path = state_dir.join("brahm.toml");
        let config_backup_path = cfg.backup_and_save(&state_cfg_path)?;

        // Generate visible documentation inside state_dir
        let _ = generate_inspection_guide(
            &conn.record,
            &conn.architecture.report,
            active_session.as_ref(),
            state_dir,
        )?;
        let _ = generate_security_doc(&conn.record, state_dir)?;

        // Persist BRAHM_AGENT_MANIFEST.json
        let agent_manifest = AgentManifest {
            schema_version: "1.0".to_string(),
            generated_at: Utc::now(),
            repository_id: conn.record.repository_id.clone(),
            registered_adapters: vec![adapter.name().to_string()],
            sessions: agent_sessions,
        };
        fs::write(
            state_dir.join("BRAHM_AGENT_MANIFEST.json"),
            serde_json::to_string_pretty(&agent_manifest)?,
        )?;

        // Persist BRAHM_INSTALLATION_REPORT.md
        let install_report_path = state_dir.join("BRAHM_INSTALLATION_REPORT.md");
        let report_md = render_installation_report_markdown(
            &conn.record,
            &plan,
            active_session.as_ref(),
            config_backup_path.as_deref(),
        );
        fs::write(&install_report_path, report_md)?;

        info!(
            "Installed BRAHM protection profile '{}' for '{}'",
            target_profile, conn.record.name
        );

        Ok(ProtectionInstallOutcome {
            record: conn.record,
            plan,
            session: active_session,
            agent_manifest,
            install_report_path,
            config_backup_path,
        })
    }

    /// Validate installed configuration, baseline isolation, and staged artifacts.
    pub fn validate_installation(
        &self,
        state_dir: &Path,
    ) -> Result<(RepositoryRecord, bool, String)> {
        let record_path = state_dir.join("BRAHM_REPOSITORY_CONFIG.json");
        if !record_path.is_file() {
            return Err(BrahmError::Integration(format!(
                "Repository is not registered in '{}'",
                state_dir.display()
            )));
        }
        let record: RepositoryRecord = serde_json::from_str(&fs::read_to_string(&record_path)?)?;

        let current_baseline = self
            .integrity_guard
            .capture_genuine_baseline(&record.local_path)?;

        let baseline_matches = current_baseline.baseline_fingerprint == record.baseline_fingerprint;

        let manifest = if let Some(ref reg_path) = record.decoy_registry_path {
            if reg_path.is_file() {
                let reg = DecoyRegistry::load_from_path(reg_path)?;
                DecoyManifest {
                    schema_version: "1.0".to_string(),
                    generated_at: Utc::now(),
                    repository_id: record.repository_id.clone(),
                    total_artifacts: reg.artifacts().len(),
                    production_build_excluded: true,
                    entries: reg
                        .artifacts()
                        .iter()
                        .map(|a| crate::deception::decoy::DecoyManifestEntry {
                            artifact_id: a.artifact_id.clone(),
                            relative_path: a.synthetic_relative_path.clone(),
                            category: a.artifact_type,
                            integrity_fingerprint: a.integrity_fingerprint.clone(),
                            brahm_owned: a.brahm_owned,
                            synthetic_status: a.synthetic_status,
                            canary_token: a.canary_token.clone(),
                        })
                        .collect(),
                }
            } else {
                empty_decoy_manifest(&record.repository_id)
            }
        } else {
            empty_decoy_manifest(&record.repository_id)
        };

        let prod_report = self
            .integrity_guard
            .verify_production_isolation(&record.local_path, &manifest)?;

        let ok = baseline_matches && prod_report.production_isolated;
        let msg = if ok {
            format!(
                "Verified: baseline matches ({}) and genuine production repository is clean.",
                &record.baseline_fingerprint[..12]
            )
        } else if !baseline_matches {
            "Baseline mismatch: genuine repository files modified since registration.".to_string()
        } else {
            format!(
                "Production isolation failed ({} finding(s)).",
                prod_report.failures_count
            )
        };

        Ok((record, ok, msg))
    }

    /// Compare live repository & staged decoys against persisted baseline digests,
    /// recording telemetry and saving `BRAHM_INTEGRITY_REPORT.json` inside `state_dir`.
    pub fn verify_state(
        &self,
        state_dir: &Path,
        correlation_prefix: &str,
    ) -> Result<IntegrityRecord> {
        let cfg_path = state_dir.join("BRAHM_REPOSITORY_CONFIG.json");
        let rec: RepositoryRecord = serde_json::from_str(&fs::read_to_string(&cfg_path)?)?;
        let store = SqliteEventStore::open(state_dir.join("brahm_monitoring.db"))?;
        let baseline = load_stored_baseline(state_dir)?;

        let reg_and_root = rec
            .decoy_registry_path
            .as_ref()
            .filter(|p| p.is_file())
            .and_then(|p| DecoyRegistry::load_from_path(p).ok())
            .map(|r| {
                let root = PathBuf::from(&r.data().decoy_layer_root);
                (r, root)
            });

        let monitor = IntegrityMonitor::new();
        let corr_id = format!("{}_{}", correlation_prefix, Utc::now().timestamp());
        let expected_paths = std::collections::HashSet::new();
        let (record, evts) = monitor.check_integrity(
            &rec.repository_id,
            &rec.local_path,
            &baseline,
            reg_and_root.as_ref().map(|(r, p)| (r, p.as_path())),
            &expected_paths,
            &corr_id,
        )?;
        store.record_integrity(&record)?;
        store.record_events_batch(&evts)?;
        fs::write(
            state_dir.join("BRAHM_INTEGRITY_REPORT.json"),
            serde_json::to_string_pretty(&record)?,
        )?;
        Ok(record)
    }

    /// One-command anti-reverse-engineering workflow: loads a user's project directory,
    /// performs deterministic Stage 01 + Stage 02 architecture mapping, stages decoys,
    /// provisions an isolated inspection workspace, and optionally verifies baseline integrity.
    pub fn apply(
        &self,
        repo_path: &Path,
        state_dir: &Path,
        profile: ProtectionProfile,
        dry_run: bool,
        verify: bool,
    ) -> Result<ApplyOutcome> {
        if dry_run {
            let preview_dir =
                std::env::temp_dir().join(format!("brahm_dry_run_{}", Uuid::new_v4().simple()));
            let plan_res = self.plan(repo_path, &preview_dir, profile);
            let cfg_data = fs::read_to_string(preview_dir.join("BRAHM_REPOSITORY_CONFIG.json"))
                .ok()
                .and_then(|s| serde_json::from_str::<RepositoryRecord>(&s).ok());
            let _ = fs::remove_dir_all(&preview_dir);
            let plan = plan_res?;
            return Ok(ApplyOutcome {
                dry_run: true,
                repository_id: plan.repository_id.clone(),
                repository_name: cfg_data
                    .as_ref()
                    .map(|r| r.name.clone())
                    .unwrap_or_else(|| plan.repository_id.clone()),
                repository_path: cfg_data
                    .as_ref()
                    .map(|r| r.local_path.clone())
                    .unwrap_or_else(|| repo_path.to_path_buf()),
                state_dir: state_dir.to_path_buf(),
                profile,
                baseline_fingerprint: cfg_data.map(|r| r.baseline_fingerprint).unwrap_or_default(),
                session_id: None,
                inspection_workspace: None,
                decoy_count: 0,
                canary_count: 0,
                integrity_verified: None,
                architecture_report_path: None,
                install_report_path: None,
                plan,
            });
        }

        let outcome = self.install(repo_path, state_dir, profile, true)?;
        let (decoy_count, canary_count) = match outcome
            .record
            .decoy_registry_path
            .as_ref()
            .filter(|p| p.is_file())
            .and_then(|p| DecoyRegistry::load_from_path(p).ok())
        {
            Some(reg) => {
                let total = reg.artifacts().len();
                let canaries = reg
                    .artifacts()
                    .iter()
                    .filter(|a| a.canary_token.is_some())
                    .count();
                (total, canaries)
            }
            None => (0, 0),
        };

        let integrity_verified = if verify {
            let rec = self.verify_state(state_dir, "apply_verify")?;
            Some(rec.baseline_matches && rec.decoy_integrity_valid)
        } else {
            None
        };

        let arch_path = state_dir.join("BRAHM_ARCHITECTURE.json");
        Ok(ApplyOutcome {
            dry_run: false,
            repository_id: outcome.record.repository_id,
            repository_name: outcome.record.name,
            repository_path: outcome.record.local_path,
            state_dir: state_dir.to_path_buf(),
            profile,
            baseline_fingerprint: outcome.record.baseline_fingerprint,
            session_id: outcome.session.as_ref().map(|s| s.session_id.clone()),
            inspection_workspace: outcome.session.map(|s| s.workspace_path),
            decoy_count,
            canary_count,
            integrity_verified,
            architecture_report_path: arch_path.is_file().then_some(arch_path),
            install_report_path: Some(outcome.install_report_path),
            plan: outcome.plan,
        })
    }

    /// Safe ownership-verified rollback removing ONLY BRAHM-owned decoys & sessions inside `state_dir`.
    /// Never modifies genuine repository source files or unknown user files.
    pub fn rollback(&self, state_dir: &Path, dry_run: bool) -> Result<RollbackReport> {
        let record_path = state_dir.join("BRAHM_REPOSITORY_CONFIG.json");
        if !record_path.is_file() {
            return Err(BrahmError::Integration(format!(
                "Cannot rollback: missing repository config in '{}'",
                state_dir.display()
            )));
        }
        let mut record: RepositoryRecord =
            serde_json::from_str(&fs::read_to_string(&record_path)?)?;

        let mut removed_artifacts = Vec::new();
        let mut skipped_unowned = Vec::new();
        let mut cleaned_sessions = Vec::new();

        // 1. Clean BRAHM-owned staged decoys
        if let Some(ref reg_path) = record.decoy_registry_path {
            if reg_path.is_file() {
                let mut reg = DecoyRegistry::load_from_path(reg_path)?;
                let decoy_root = PathBuf::from(&reg.data().decoy_layer_root);
                if decoy_root.exists() {
                    let res = reg.cleanup_owned_artifacts(&decoy_root, dry_run)?;
                    removed_artifacts.extend(res.removed_paths);
                    skipped_unowned.extend(res.skipped_unverified_paths);
                }
                if !dry_run {
                    reg.save_to_path(reg_path)?;
                }
            }
        }

        // 2. Clean isolated session mirrors
        let sessions_dir = state_dir.join("sessions");
        if sessions_dir.is_dir() {
            let sess_mgr = InspectionSessionManager::new(&sessions_dir);
            for sess in sess_mgr.list_sessions()? {
                cleaned_sessions.push(sess.session_id.clone());
                if !dry_run {
                    let _ = sess_mgr.cleanup_session(&sess.session_id)?;
                }
            }
        }

        // 3. Restore backed up brahm.toml.bak if present
        let cfg_path = state_dir.join("brahm.toml");
        let bak_path = state_dir.join("brahm.toml.bak");
        let restored_backup = bak_path.is_file();
        if restored_backup && !dry_run {
            fs::copy(&bak_path, &cfg_path)?;
        }

        if !dry_run {
            record.active_profile = ProtectionProfile::AnalysisOnly;
            record.active_session_id = None;
            record.updated_at = Utc::now();
            fs::write(&record_path, serde_json::to_string_pretty(&record)?)?;
        }

        // 4. Verify genuine baseline remains intact afterward
        let post_baseline = self
            .integrity_guard
            .capture_genuine_baseline(&record.local_path)?;
        let integrity_verified_after =
            post_baseline.baseline_fingerprint == record.baseline_fingerprint;

        let report = RollbackReport {
            schema_version: "1.0".to_string(),
            executed_at: Utc::now(),
            dry_run,
            repository_id: record.repository_id.clone(),
            removed_artifacts,
            cleaned_sessions,
            restored_config_backup: restored_backup,
            skipped_unowned_files: skipped_unowned,
            integrity_verified_after,
        };

        fs::write(
            state_dir.join("BRAHM_ROLLBACK_REPORT.json"),
            serde_json::to_string_pretty(&report)?,
        )?;

        Ok(report)
    }
}

fn empty_decoy_manifest(repo_id: &str) -> DecoyManifest {
    DecoyManifest {
        schema_version: "1.0".to_string(),
        generated_at: Utc::now(),
        repository_id: repo_id.to_string(),
        total_artifacts: 0,
        production_build_excluded: true,
        entries: Vec::new(),
    }
}

fn render_installation_report_markdown(
    record: &RepositoryRecord,
    plan: &ProtectionPlan,
    session: Option<&InspectionSession>,
    backup_path: Option<&Path>,
) -> String {
    let mut md = String::new();
    md.push_str("# BRAHM Stage 05 — Protection Installation Report\n\n");
    md.push_str("| Property | Value |\n");
    md.push_str("|---|---|\n");
    md.push_str(&format!(
        "| **Repository** | `{}` (`{}`) |\n",
        record.name, record.repository_id
    ));
    md.push_str(&format!("| **Plan ID** | `{}` |\n", plan.plan_id));
    md.push_str(&format!(
        "| **Active Profile** | `{}` |\n",
        record.active_profile
    ));
    md.push_str(&format!(
        "| **Baseline Hash** | `{}` |\n",
        record.baseline_fingerprint
    ));
    md.push_str(&format!(
        "| **Production Excluded** | `{}` |\n",
        plan.production_exclusions_verified
    ));
    if let Some(sess) = session {
        md.push_str(&format!(
            "| **Inspection Session** | `{}` |\n",
            sess.session_id
        ));
    }
    if let Some(bak) = backup_path {
        md.push_str(&format!(
            "| **Config Backup** | `{}` |\n",
            bak.to_string_lossy().replace('\\', "/")
        ));
    }
    md.push('\n');
    md
}
