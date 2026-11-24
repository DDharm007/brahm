//! Rules-based file classifier.

use crate::models::FileClassification;

/// Classifies a file into an architectural category based on relative path, filename, and extension.
pub fn classify_file(
    relative_path: &str,
    filename: &str,
    extension: Option<&str>,
) -> FileClassification {
    let lower_path = relative_path.to_ascii_lowercase().replace('\\', "/");
    let lower_name = filename.to_ascii_lowercase();

    // 1. Tests
    if lower_path.contains("/tests/")
        || lower_path.contains("/test/")
        || lower_path.contains("/__tests__/")
        || lower_name.ends_with("_test.rs")
        || lower_name.ends_with(".test.ts")
        || lower_name.ends_with(".test.js")
        || lower_name.ends_with(".spec.ts")
        || lower_name.ends_with(".spec.js")
        || lower_name.starts_with("test_")
    {
        return FileClassification::Tests;
    }

    // 2. Documentation
    if lower_name.starts_with("readme")
        || lower_name.starts_with("license")
        || lower_name.starts_with("changelog")
        || lower_name.starts_with("contributing")
        || lower_name.starts_with("code_of_conduct")
        || lower_path.starts_with("docs/")
        || lower_path.contains("/docs/")
    {
        return FileClassification::Documentation;
    }

    // 3. Infrastructure & Deployment
    if lower_name == "dockerfile"
        || lower_name.starts_with("dockerfile.")
        || lower_name.starts_with("docker-compose")
        || lower_path.contains("/k8s/")
        || lower_path.contains("/kubernetes/")
        || lower_path.contains("/terraform/")
        || lower_path.contains("/helm/")
        || lower_path.contains("/deploy/")
    {
        return FileClassification::Infrastructure;
    }

    // 4. Build System
    if lower_name == "cargo.toml"
        || lower_name == "makefile"
        || lower_name == "cmakelists.txt"
        || lower_name == "pom.xml"
        || lower_name.starts_with("build.gradle")
        || lower_name == "gemfile"
    {
        return FileClassification::BuildSystem;
    }

    // 5. Database
    if lower_path.contains("/migrations/")
        || lower_path.contains("/schema/")
        || lower_path.contains("/db/")
        || extension.is_some_and(|e| e.eq_ignore_ascii_case("sql"))
    {
        return FileClassification::Database;
    }

    // 6. Scripts
    if lower_path.contains("/scripts/")
        || extension.is_some_and(|e| {
            e.eq_ignore_ascii_case("sh")
                || e.eq_ignore_ascii_case("ps1")
                || e.eq_ignore_ascii_case("bat")
                || e.eq_ignore_ascii_case("cmd")
        })
    {
        return FileClassification::Scripts;
    }

    // 7. Assets
    if extension.is_some_and(|e| {
        matches!(
            e.to_ascii_lowercase().as_str(),
            "png"
                | "jpg"
                | "jpeg"
                | "gif"
                | "svg"
                | "ico"
                | "webp"
                | "ttf"
                | "woff"
                | "woff2"
                | "eot"
                | "mp3"
                | "wav"
                | "mp4"
        )
    }) {
        return FileClassification::Assets;
    }

    // 8. Configuration
    if lower_name.starts_with('.')
        || lower_name.contains("config")
        || lower_name == "package.json"
        || lower_name == "tsconfig.json"
        || lower_name == "pyproject.toml"
        || lower_name == "requirements.txt"
        || lower_name == "go.mod"
        || extension.is_some_and(|e| {
            matches!(
                e.to_ascii_lowercase().as_str(),
                "toml" | "json" | "yaml" | "yml" | "xml" | "ini"
            )
        })
    {
        return FileClassification::Configuration;
    }

    // 9. Source Code
    if extension.is_some_and(|e| {
        matches!(
            e.to_ascii_lowercase().as_str(),
            "rs" | "js"
                | "mjs"
                | "cjs"
                | "ts"
                | "mts"
                | "cts"
                | "py"
                | "go"
                | "java"
                | "c"
                | "cpp"
                | "cc"
                | "h"
                | "hpp"
                | "cs"
                | "php"
                | "rb"
                | "swift"
                | "kt"
                | "dart"
                | "html"
                | "css"
                | "scss"
        )
    }) {
        return FileClassification::SourceCode;
    }

    // 10. Unknown
    FileClassification::Unknown
}
