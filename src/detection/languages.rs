//! Deterministic programming language detection based on extension and filename.

/// Detects the programming language for a given file name and extension.
pub fn detect_language(filename: &str, extension: Option<&str>) -> Option<String> {
    // 1. Exact filename matches take precedence
    match filename {
        "Dockerfile" => return Some("Dockerfile".to_string()),
        "Makefile" | "makefile" | "GNUmakefile" => return Some("Makefile".to_string()),
        "CMakeLists.txt" => return Some("CMake".to_string()),
        "Cargo.toml" => return Some("TOML".to_string()),
        "Cargo.lock" => return Some("TOML".to_string()),
        "package.json" => return Some("JSON".to_string()),
        "tsconfig.json" => return Some("JSON".to_string()),
        "pyproject.toml" => return Some("TOML".to_string()),
        "go.mod" | "go.sum" => return Some("Go".to_string()),
        "pom.xml" => return Some("XML".to_string()),
        "Gemfile" => return Some("Ruby".to_string()),
        _ => {}
    }

    // 2. Extension matching
    let ext = extension?.to_ascii_lowercase();
    let lang = match ext.as_str() {
        "rs" => "Rust",
        "js" | "mjs" | "cjs" => "JavaScript",
        "ts" | "mts" | "cts" => "TypeScript",
        "jsx" => "JavaScript",
        "tsx" => "TypeScript",
        "py" | "pyw" | "pyi" => "Python",
        "java" => "Java",
        "go" => "Go",
        "c" | "h" => "C",
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => "C++",
        "cs" => "C#",
        "php" | "phtml" => "PHP",
        "rb" => "Ruby",
        "swift" => "Swift",
        "kt" | "kts" => "Kotlin",
        "dart" => "Dart",
        "html" | "htm" => "HTML",
        "css" => "CSS",
        "scss" | "sass" => "SCSS",
        "sql" => "SQL",
        "sh" | "bash" | "zsh" => "Shell",
        "ps1" | "psm1" | "psd1" => "PowerShell",
        "json" => "JSON",
        "yaml" | "yml" => "YAML",
        "toml" => "TOML",
        "xml" => "XML",
        "md" | "markdown" => "Markdown",
        _ => return None,
    };

    Some(lang.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection_by_extension() {
        assert_eq!(
            detect_language("main.rs", Some("rs")),
            Some("Rust".to_string())
        );
        assert_eq!(
            detect_language("app.ts", Some("ts")),
            Some("TypeScript".to_string())
        );
        assert_eq!(
            detect_language("index.js", Some("js")),
            Some("JavaScript".to_string())
        );
        assert_eq!(
            detect_language("component.tsx", Some("tsx")),
            Some("TypeScript".to_string())
        );
        assert_eq!(
            detect_language("script.py", Some("py")),
            Some("Python".to_string())
        );
        assert_eq!(
            detect_language("Server.java", Some("java")),
            Some("Java".to_string())
        );
        assert_eq!(
            detect_language("main.go", Some("go")),
            Some("Go".to_string())
        );
        assert_eq!(detect_language("core.c", Some("c")), Some("C".to_string()));
        assert_eq!(
            detect_language("engine.cpp", Some("cpp")),
            Some("C++".to_string())
        );
        assert_eq!(
            detect_language("Model.cs", Some("cs")),
            Some("C#".to_string())
        );
        assert_eq!(
            detect_language("index.php", Some("php")),
            Some("PHP".to_string())
        );
        assert_eq!(
            detect_language("app.rb", Some("rb")),
            Some("Ruby".to_string())
        );
        assert_eq!(
            detect_language("view.swift", Some("swift")),
            Some("Swift".to_string())
        );
        assert_eq!(
            detect_language("Main.kt", Some("kt")),
            Some("Kotlin".to_string())
        );
        assert_eq!(
            detect_language("lib.dart", Some("dart")),
            Some("Dart".to_string())
        );
        assert_eq!(
            detect_language("index.html", Some("html")),
            Some("HTML".to_string())
        );
        assert_eq!(
            detect_language("style.css", Some("css")),
            Some("CSS".to_string())
        );
        assert_eq!(
            detect_language("theme.scss", Some("scss")),
            Some("SCSS".to_string())
        );
        assert_eq!(
            detect_language("schema.sql", Some("sql")),
            Some("SQL".to_string())
        );
        assert_eq!(
            detect_language("deploy.sh", Some("sh")),
            Some("Shell".to_string())
        );
        assert_eq!(
            detect_language("build.ps1", Some("ps1")),
            Some("PowerShell".to_string())
        );
        assert_eq!(
            detect_language("data.json", Some("json")),
            Some("JSON".to_string())
        );
        assert_eq!(
            detect_language("config.yaml", Some("yaml")),
            Some("YAML".to_string())
        );
        assert_eq!(
            detect_language("config.toml", Some("toml")),
            Some("TOML".to_string())
        );
        assert_eq!(
            detect_language("manifest.xml", Some("xml")),
            Some("XML".to_string())
        );
        assert_eq!(
            detect_language("README.md", Some("md")),
            Some("Markdown".to_string())
        );
    }

    #[test]
    fn test_language_detection_by_filename() {
        assert_eq!(
            detect_language("Dockerfile", None),
            Some("Dockerfile".to_string())
        );
        assert_eq!(
            detect_language("Makefile", None),
            Some("Makefile".to_string())
        );
        assert_eq!(
            detect_language("CMakeLists.txt", Some("txt")),
            Some("CMake".to_string())
        );
        assert_eq!(detect_language("Gemfile", None), Some("Ruby".to_string()));
    }

    #[test]
    fn test_unknown_extension_returns_none() {
        assert_eq!(
            detect_language("archive.unknownext", Some("unknownext")),
            None
        );
        assert_eq!(detect_language("binary.xyz123", Some("xyz123")), None);
        assert_eq!(detect_language("raw_data", None), None);
    }
}
