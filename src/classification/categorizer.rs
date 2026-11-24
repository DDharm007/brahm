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

    // 1. Dependencies
    if lower_path.starts_with("node_modules/")
        || lower_path.contains("/node_modules/")
        || lower_path.starts_with("vendor/")
        || lower_path.contains("/vendor/")
        || lower_path.starts_with("target/")
        || lower_path.contains("/target/")
    {
        return FileClassification::Dependencies;
    }

    // 2. Generated files
    if lower_path.starts_with("dist/")
        || lower_path.contains("/dist/")
        || lower_path.starts_with("build/")
        || lower_path.contains("/build/")
        || lower_name.ends_with(".min.js")
        || lower_name.ends_with(".min.css")
        || lower_name.ends_with(".map")
        || lower_name.contains(".generated.")
    {
        return FileClassification::Generated;
    }

    // 3. Tests
    if lower_path.starts_with("tests/")
        || lower_path.contains("/tests/")
        || lower_path.starts_with("test/")
        || lower_path.contains("/test/")
        || lower_path.contains("/__tests__/")
        || lower_name.ends_with("_test.rs")
        || lower_name.ends_with(".test.ts")
        || lower_name.ends_with(".test.js")
        || lower_name.ends_with(".test.tsx")
        || lower_name.ends_with(".test.jsx")
        || lower_name.ends_with(".spec.ts")
        || lower_name.ends_with(".spec.js")
        || lower_name.ends_with(".spec.tsx")
        || lower_name.ends_with(".spec.jsx")
        || lower_name.ends_with("_test.go")
        || lower_name.starts_with("test_")
    {
        return FileClassification::Tests;
    }

    // 4. Documentation
    if lower_name.starts_with("readme")
        || lower_name.starts_with("license")
        || lower_name.starts_with("changelog")
        || lower_name.starts_with("contributing")
        || lower_name.starts_with("code_of_conduct")
        || lower_path.starts_with("docs/")
        || lower_path.contains("/docs/")
        || extension.is_some_and(|e| e.eq_ignore_ascii_case("md"))
    {
        return FileClassification::Documentation;
    }

    // 5. Deployment
    if lower_path.contains(".github/workflows/")
        || lower_name == ".gitlab-ci.yml"
        || lower_name == "procfile"
        || lower_path.starts_with("deploy/")
        || lower_path.contains("/deploy/")
        || lower_path.starts_with("deployments/")
        || lower_path.contains("/deployments/")
    {
        return FileClassification::Deployment;
    }

    // 6. Infrastructure
    if lower_name == "dockerfile"
        || lower_name.starts_with("dockerfile.")
        || lower_name.starts_with("docker-compose")
        || lower_path.contains("/k8s/")
        || lower_path.contains("/kubernetes/")
        || lower_path.contains("/terraform/")
        || lower_path.contains("/helm/")
        || lower_path.contains("/ansible/")
        || extension.is_some_and(|e| e.eq_ignore_ascii_case("tf"))
    {
        return FileClassification::Infrastructure;
    }

    // 7. Build System
    if lower_name == "cargo.toml"
        || lower_name == "makefile"
        || lower_name == "cmakelists.txt"
        || lower_name == "pom.xml"
        || lower_name.starts_with("build.gradle")
        || lower_name == "gemfile"
        || lower_name == "webpack.config.js"
        || lower_name.starts_with("vite.config.")
    {
        return FileClassification::BuildSystem;
    }

    // 8. Database
    if lower_path.contains("/migrations/")
        || lower_path.contains("/schema/")
        || lower_path.contains("/db/")
        || lower_path.contains("/database/")
        || lower_path.contains("/prisma/")
        || lower_name == "schema.prisma"
        || extension.is_some_and(|e| e.eq_ignore_ascii_case("sql"))
    {
        return FileClassification::Database;
    }

    // 9. Scripts
    if lower_path.starts_with("scripts/")
        || lower_path.contains("/scripts/")
        || extension.is_some_and(|e| {
            matches!(
                e.to_ascii_lowercase().as_str(),
                "sh" | "bash" | "zsh" | "ps1" | "bat" | "cmd"
            )
        })
    {
        return FileClassification::Scripts;
    }

    // 10. Assets
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
                | "avi"
                | "mov"
        )
    }) {
        return FileClassification::Assets;
    }

    // 11. Configuration
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
                "toml" | "json" | "yaml" | "yml" | "xml" | "ini" | "env" | "conf"
            )
        })
    {
        return FileClassification::Configuration;
    }

    // 12. Source Code
    if extension.is_some_and(|e| {
        matches!(
            e.to_ascii_lowercase().as_str(),
            "rs" | "js"
                | "mjs"
                | "cjs"
                | "ts"
                | "mts"
                | "cts"
                | "jsx"
                | "tsx"
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

    // 13. Unknown
    FileClassification::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_classifications() {
        assert_eq!(
            classify_file("src/main.rs", "main.rs", Some("rs")),
            FileClassification::SourceCode
        );
        assert_eq!(
            classify_file(
                "tests/integration_test.rs",
                "integration_test.rs",
                Some("rs")
            ),
            FileClassification::Tests
        );
        assert_eq!(
            classify_file("README.md", "README.md", Some("md")),
            FileClassification::Documentation
        );
        assert_eq!(
            classify_file("Dockerfile", "Dockerfile", None),
            FileClassification::Infrastructure
        );
        assert_eq!(
            classify_file(".github/workflows/ci.yml", "ci.yml", Some("yml")),
            FileClassification::Deployment
        );
        assert_eq!(
            classify_file("Cargo.toml", "Cargo.toml", Some("toml")),
            FileClassification::BuildSystem
        );
        assert_eq!(
            classify_file("migrations/001_init.sql", "001_init.sql", Some("sql")),
            FileClassification::Database
        );
        assert_eq!(
            classify_file("scripts/setup.sh", "setup.sh", Some("sh")),
            FileClassification::Scripts
        );
        assert_eq!(
            classify_file("public/logo.png", "logo.png", Some("png")),
            FileClassification::Assets
        );
        assert_eq!(
            classify_file("tsconfig.json", "tsconfig.json", Some("json")),
            FileClassification::Configuration
        );
        assert_eq!(
            classify_file("dist/bundle.min.js", "bundle.min.js", Some("js")),
            FileClassification::Generated
        );
        assert_eq!(
            classify_file("node_modules/express/index.js", "index.js", Some("js")),
            FileClassification::Dependencies
        );
        assert_eq!(
            classify_file("random_unknown.xyz", "random_unknown.xyz", Some("xyz")),
            FileClassification::Unknown
        );
    }
}
