//! Manifest file identification.

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
    "Dockerfile",
    "docker-compose.yml",
    "docker-compose.yaml",
    "tsconfig.json",
    "vite.config.ts",
    "vite.config.js",
    "next.config.js",
    "next.config.ts",
    "next.config.mjs",
    "webpack.config.js",
    ".gitignore",
    ".env.example",
    "README.md",
];

/// Checks if a filename is a known manifest or project configuration file.
pub fn is_manifest(filename: &str) -> bool {
    KNOWN_MANIFESTS.contains(&filename)
}
