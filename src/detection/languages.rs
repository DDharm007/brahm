//! Deterministic programming language detection based on extension and filename.

/// Detects the programming language for a given file name and extension.
pub fn detect_language(filename: &str, extension: Option<&str>) -> Option<String> {
    // Exact filename matches take priority
    match filename {
        "Dockerfile" => return Some("Dockerfile".to_string()),
        "Makefile" => return Some("Makefile".to_string()),
        "CMakeLists.txt" => return Some("CMake".to_string()),
        _ => {}
    }

    let ext = extension?.to_ascii_lowercase();
    let lang = match ext.as_str() {
        "rs" => "Rust",
        "js" | "mjs" | "cjs" => "JavaScript",
        "ts" | "mts" | "cts" => "TypeScript",
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
