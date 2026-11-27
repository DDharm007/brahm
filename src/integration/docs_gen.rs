//! Transparent Inspection Guide & Security Documentation Generator (Milestone 05.5).
//!
//! Generates human-readable repository guidance clearly distinguishing verified Stage 02 architecture
//! from synthetic inspection artifacts without hidden prompts or coercive instructions.

use std::fs;
use std::path::Path;

use super::models::RepositoryRecord;
use crate::architecture::ArchitectureReport;
use crate::deception::InspectionSession;
use crate::errors::Result;

/// Generate `BRAHM_INSPECTION_GUIDE.md` in `out_dir`.
pub fn generate_inspection_guide(
    record: &RepositoryRecord,
    arch: &ArchitectureReport,
    session: Option<&InspectionSession>,
    out_dir: &Path,
) -> Result<String> {
    let mut md = String::new();
    md.push_str("# BRAHM Controlled Repository Inspection Guide\n\n");
    md.push_str(
        "> Transparent operational instructions for authorized developers and test agents.\n\n",
    );

    md.push_str("## 1. Verified Architecture vs. Inspection Workspace\n\n");
    md.push_str(&format!(
        "- **Repository:** `{}` (`{}`)\n",
        record.name, record.repository_id
    ));
    md.push_str(&format!(
        "- **Active Protection Profile:** `{}`\n",
        record.active_profile
    ));
    md.push_str(&format!(
        "- **Verified Architecture Pattern:** {}\n",
        arch.pattern.primary_pattern
    ));
    md.push_str(&format!(
        "- **Genuine Baseline Fingerprint (BLAKE3):** `{}`\n\n",
        record.baseline_fingerprint
    ));

    md.push_str("### Clear Boundary Notice\n");
    md.push_str("- Verified codebase ground truth is captured exclusively in `BRAHM_ARCHITECTURE.json` and `BRAHM_ARCHITECTURE.md`.\n");
    md.push_str("- Files introduced under the inspection mirror workspace matching `BRAHM_DECOY_MANIFEST.json` are **controlled synthetic artifacts**.\n\n");

    if let Some(sess) = session {
        md.push_str("## 2. Active Inspection Session Scope\n\n");
        md.push_str(&format!("- **Session ID:** `{}`\n", sess.session_id));
        md.push_str(&format!(
            "- **Isolated Workspace:** `{}`\n",
            sess.workspace_path.to_string_lossy().replace('\\', "/")
        ));
        md.push_str(&format!(
            "- **Policy Applied:** `{}` (`{}`)\n",
            sess.policy_id, sess.policy_profile
        ));
        md.push_str(&format!(
            "- **Registered Synthetic Artifacts:** `{}`\n",
            sess.registered_artifacts.len()
        ));
        md.push_str(&format!(
            "- **Session Expiration (UTC):** `{}`\n\n",
            sess.expires_at
        ));
    }

    md.push_str("## 3. Operational Rules\n\n");
    md.push_str("1. Operate exclusively within the designated isolated inspection workspace.\n");
    md.push_str("2. Never mutate genuine production repositories directly.\n");
    md.push_str("3. All decoy and canary interactions inside the inspection workspace are logged transparently for telemetry analysis.\n");

    fs::create_dir_all(out_dir)?;
    fs::write(out_dir.join("BRAHM_INSPECTION_GUIDE.md"), &md)?;
    Ok(md)
}

/// Generate `BRAHM_SECURITY.md` documenting scope, honesty guarantees, and limits.
pub fn generate_security_doc(record: &RepositoryRecord, out_dir: &Path) -> Result<String> {
    let mut md = String::new();
    md.push_str("# BRAHM Security Model & Operational Limitations\n\n");
    md.push_str(&format!(
        "**Target Repository:** `{}` (`{}`)\n\n",
        record.name, record.repository_id
    ));

    md.push_str("## 1. Guarantees Enforced\n\n");
    md.push_str("- **Read-Only Genuine Source:** Genuine application files are never altered during analysis or inspection preparation.\n");
    md.push_str("- **Secret Exclusion:** Sensitive credential patterns (`.env`, `*.pem`, `*.key`, etc.) are omitted from inspection mirrors and telemetry.\n");
    md.push_str("- **Production Build Isolation:** Synthetic decoy artifacts are excluded from production build manifests and validated via `ProductionIntegrityGuard`.\n");
    md.push_str("- **Zero Runtime AI Models:** All detection, parsing, and telemetry correlation rules execute deterministically.\n\n");

    md.push_str("## 2. Explicit Scope & Technical Limitations\n\n");
    md.push_str("- BRAHM does **not** guarantee prevention of reverse engineering or unauthorized copying of public repositories.\n");
    md.push_str("- Public Git hosts (e.g., GitHub) serve identical tree views to anonymous clones; selective inspection views apply to controlled local mirrors and authorized sandbox environments.\n");
    md.push_str("- Canary interaction events prove access to a registered synthetic artifact within the telemetry boundary; they do **not** definitively attribute identity (human vs. AI model) without independent corroboration.\n");

    fs::create_dir_all(out_dir)?;
    fs::write(out_dir.join("BRAHM_SECURITY.md"), &md)?;
    Ok(md)
}
