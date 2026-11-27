//! Deterministic identifiers and non-secret canary markers for Stage 03 Decoy Engine.

use std::path::Path;

/// Generate a deterministic repository identifier from its canonical path or name.
pub fn generate_repository_id(repo_path: &Path, repo_name: &str) -> String {
    let norm = repo_path.to_string_lossy().replace('\\', "/");
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"brahm-repo-v1:");
    hasher.update(repo_name.as_bytes());
    hasher.update(b":");
    hasher.update(norm.as_bytes());
    let hex = hasher.finalize().to_hex();
    format!("repo_{}", &hex[..16])
}

/// Generate a deterministic 22-character artifact ID (`decoy_<16hex>`).
///
/// Determinism inputs: repository ID, template ID, template version, and normalized relative path.
pub fn generate_decoy_artifact_id(
    repository_id: &str,
    template_id: &str,
    template_version: &str,
    relative_path: &str,
) -> String {
    let norm_path = relative_path.replace('\\', "/");
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"brahm-decoy-v1:");
    hasher.update(repository_id.as_bytes());
    hasher.update(b":");
    hasher.update(template_id.as_bytes());
    hasher.update(b":");
    hasher.update(template_version.as_bytes());
    hasher.update(b":");
    hasher.update(norm_path.as_bytes());
    let hex = hasher.finalize().to_hex();
    format!("decoy_{}", &hex[..16])
}

/// Generate a synthetic, explicitly non-secret, non-routable canary marker.
///
/// Format: `BRAHM-CANARY-<12HEX>` so it is easily recognizable by BRAHM telemetry
/// while being unambiguously inert and incapable of granting access anywhere.
pub fn generate_canary_marker(artifact_id: &str, repository_id: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"brahm-canary-marker-v1:");
    hasher.update(repository_id.as_bytes());
    hasher.update(b":");
    hasher.update(artifact_id.as_bytes());
    let hex = hasher.finalize().to_hex();
    format!("BRAHM-CANARY-{}", hex[..12].to_ascii_uppercase())
}

/// Compute BLAKE3 hexadecimal integrity fingerprint for artifact text contents.
pub fn compute_content_fingerprint(content: &str) -> String {
    blake3::hash(content.as_bytes()).to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_ids_and_canary_markers() {
        let repo_id = generate_repository_id(Path::new("fixtures/sample"), "sample");
        let id1 =
            generate_decoy_artifact_id(&repo_id, "tpl_canary_md", "1.0.0", "canary/anchor.md");
        let id2 =
            generate_decoy_artifact_id(&repo_id, "tpl_canary_md", "1.0.0", "canary\\anchor.md");
        assert_eq!(id1, id2);
        assert!(id1.starts_with("decoy_"));

        let c1 = generate_canary_marker(&id1, &repo_id);
        assert!(c1.starts_with("BRAHM-CANARY-"));
        assert_eq!(c1.len(), "BRAHM-CANARY-".len() + 12);
    }
}
