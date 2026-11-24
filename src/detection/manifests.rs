//! Manifest and configuration file identification with credential safety safeguards.

/// Standard project manifests and ecosystem package definition files.
pub const KNOWN_MANIFESTS: &[&str] = &[
    "package.json",
    "Cargo.toml",
    "requirements.txt",
    "pyproject.toml",
    "Pipfile",
    "go.mod",
    "pom.xml",
    "build.gradle",
    "build.gradle.kts",
    "Gemfile",
    "composer.json",
];

/// Known configuration, build, and deployment metadata files.
pub const KNOWN_CONFIGURATIONS: &[&str] = &[
    "Dockerfile",
    "docker-compose.yml",
    "docker-compose.yaml",
    "tsconfig.json",
    "vite.config.ts",
    "vite.config.js",
    "vite.config.mjs",
    "next.config.js",
    "next.config.ts",
    "next.config.mjs",
    "webpack.config.js",
    ".gitignore",
    ".env.example",
    "README.md",
];

/// Checks if a filename is an official project manifest.
pub fn is_manifest(filename: &str) -> bool {
    KNOWN_MANIFESTS.contains(&filename)
}

/// Checks if a file represents an important project configuration or manifest.
pub fn is_configuration_or_manifest(filename: &str) -> bool {
    is_manifest(filename) || KNOWN_CONFIGURATIONS.contains(&filename)
}

/// Identifies sensitive credential or secret files that must NEVER be read or exposed.
pub fn is_sensitive_credential_file(filename: &str) -> bool {
    let lower = filename.to_ascii_lowercase();

    // Never match .env.example as sensitive
    if lower == ".env.example" || lower.ends_with(".example") || lower.ends_with(".sample") {
        return false;
    }

    if lower.starts_with(".env")
        || lower.ends_with(".pem")
        || lower.ends_with(".key")
        || lower.ends_with(".pkcs12")
        || lower.ends_with(".pfx")
        || lower == "id_rsa"
        || lower == "id_ed25519"
        || lower == "id_ecdsa"
        || lower.contains("credentials")
        || lower.contains("service-account")
        || lower.contains("secret")
    {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_identification() {
        assert!(is_manifest("package.json"));
        assert!(is_manifest("Cargo.toml"));
        assert!(is_manifest("requirements.txt"));
        assert!(is_manifest("pyproject.toml"));
        assert!(is_manifest("go.mod"));
        assert!(is_manifest("pom.xml"));
        assert!(is_manifest("build.gradle"));
        assert!(is_manifest("Gemfile"));
        assert!(!is_manifest("unknown_file.txt"));
    }

    #[test]
    fn test_configuration_identification() {
        assert!(is_configuration_or_manifest("Dockerfile"));
        assert!(is_configuration_or_manifest("docker-compose.yml"));
        assert!(is_configuration_or_manifest("tsconfig.json"));
        assert!(is_configuration_or_manifest("vite.config.ts"));
        assert!(is_configuration_or_manifest("next.config.js"));
        assert!(is_configuration_or_manifest(".gitignore"));
        assert!(is_configuration_or_manifest(".env.example"));
        assert!(is_configuration_or_manifest("README.md"));
    }

    #[test]
    fn test_sensitive_credential_protection() {
        assert!(is_sensitive_credential_file(".env"));
        assert!(is_sensitive_credential_file(".env.local"));
        assert!(is_sensitive_credential_file(".env.production"));
        assert!(is_sensitive_credential_file("id_rsa"));
        assert!(is_sensitive_credential_file("server.key"));
        assert!(is_sensitive_credential_file("cert.pem"));
        assert!(is_sensitive_credential_file("aws_credentials.json"));
        assert!(is_sensitive_credential_file("gcp-service-account.json"));

        // Safe files must not be flagged as sensitive
        assert!(!is_sensitive_credential_file(".env.example"));
        assert!(!is_sensitive_credential_file("config.example.json"));
        assert!(!is_sensitive_credential_file("README.md"));
    }
}
