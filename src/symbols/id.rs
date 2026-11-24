//! Deterministic symbol identifier generation.

use std::path::Path;

use super::types::SymbolKind;

/// Generate a deterministic, collision-resistant, 16-character hex identifier for a code symbol.
///
/// Combines the relative file path, fully-qualified name, and symbol kind.
/// Ensures that running BRAHM repeatedly on the same codebase yields identical symbol IDs.
pub fn generate_symbol_id(file_path: &Path, qualified_name: &str, kind: SymbolKind) -> String {
    let mut hasher = blake3::Hasher::new();

    // Normalize path separators to forward slashes for cross-platform reproducibility
    let normalized_path = file_path.to_string_lossy().replace('\\', "/");

    hasher.update(b"brahm:sym:v1:");
    hasher.update(normalized_path.as_bytes());
    hasher.update(b":");
    hasher.update(qualified_name.as_bytes());
    hasher.update(b":");
    hasher.update(kind.as_str().as_bytes());

    let hash = hasher.finalize();
    let hex = hash.to_hex();
    format!("sym_{}", &hex[..16])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_determinism_and_uniqueness() {
        let p1 = Path::new("src/auth/service.rs");
        let id1 = generate_symbol_id(p1, "AuthService::login", SymbolKind::Method);
        let id2 = generate_symbol_id(p1, "AuthService::login", SymbolKind::Method);
        let id3 = generate_symbol_id(p1, "AuthService::logout", SymbolKind::Method);

        assert_eq!(id1, id2, "Identical inputs must produce identical IDs");
        assert_ne!(
            id1, id3,
            "Different qualified names must produce different IDs"
        );
        assert!(id1.starts_with("sym_"));
        assert_eq!(id1.len(), 20); // "sym_" + 16 chars
    }

    #[test]
    fn test_id_cross_platform_path_normalization() {
        let p_win = Path::new("src\\auth\\service.rs");
        let p_unix = Path::new("src/auth/service.rs");

        let id_win = generate_symbol_id(p_win, "AuthService", SymbolKind::Struct);
        let id_unix = generate_symbol_id(p_unix, "AuthService", SymbolKind::Struct);

        assert_eq!(
            id_win, id_unix,
            "Path separators must be normalized so Windows and Unix produce identical IDs"
        );
    }
}
