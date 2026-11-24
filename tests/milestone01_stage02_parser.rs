//! Stage 02 Milestone 01 Verification Test Suite
//! Multi-Language Tree-sitter Parser Foundation & Registry

use std::fs;
use std::sync::Arc;
use std::thread;
use tempfile::tempdir;

use brahm::parser::{ParserConfig, ParserError, ParserRegistry, SupportedLanguage};

#[test]
fn test_01_language_enum_and_extensions() {
    assert_eq!(SupportedLanguage::all().len(), 9);

    assert_eq!(
        SupportedLanguage::from_extension("rs"),
        Some(SupportedLanguage::Rust)
    );
    assert_eq!(
        SupportedLanguage::from_extension(".RS"),
        Some(SupportedLanguage::Rust)
    );
    assert_eq!(
        SupportedLanguage::from_extension("js"),
        Some(SupportedLanguage::JavaScript)
    );
    assert_eq!(
        SupportedLanguage::from_extension("mjs"),
        Some(SupportedLanguage::JavaScript)
    );
    assert_eq!(
        SupportedLanguage::from_extension("cjs"),
        Some(SupportedLanguage::JavaScript)
    );
    assert_eq!(
        SupportedLanguage::from_extension("jsx"),
        Some(SupportedLanguage::JavaScript)
    );
    assert_eq!(
        SupportedLanguage::from_extension("ts"),
        Some(SupportedLanguage::TypeScript)
    );
    assert_eq!(
        SupportedLanguage::from_extension("mts"),
        Some(SupportedLanguage::TypeScript)
    );
    assert_eq!(
        SupportedLanguage::from_extension("tsx"),
        Some(SupportedLanguage::Tsx)
    );
    assert_eq!(
        SupportedLanguage::from_extension("py"),
        Some(SupportedLanguage::Python)
    );
    assert_eq!(
        SupportedLanguage::from_extension("go"),
        Some(SupportedLanguage::Go)
    );
    assert_eq!(
        SupportedLanguage::from_extension("java"),
        Some(SupportedLanguage::Java)
    );
    assert_eq!(
        SupportedLanguage::from_extension("c"),
        Some(SupportedLanguage::C)
    );
    assert_eq!(
        SupportedLanguage::from_extension("h"),
        Some(SupportedLanguage::C)
    );
    assert_eq!(
        SupportedLanguage::from_extension("cpp"),
        Some(SupportedLanguage::Cpp)
    );
    assert_eq!(
        SupportedLanguage::from_extension("hpp"),
        Some(SupportedLanguage::Cpp)
    );

    assert_eq!(SupportedLanguage::from_extension("unknown"), None);
}

#[test]
fn test_02_language_names_and_aliases() {
    assert_eq!(
        SupportedLanguage::from_language_name("Rust"),
        Some(SupportedLanguage::Rust)
    );
    assert_eq!(
        SupportedLanguage::from_language_name("rust"),
        Some(SupportedLanguage::Rust)
    );
    assert_eq!(
        SupportedLanguage::from_language_name("JavaScript"),
        Some(SupportedLanguage::JavaScript)
    );
    assert_eq!(
        SupportedLanguage::from_language_name("TypeScript"),
        Some(SupportedLanguage::TypeScript)
    );
    assert_eq!(
        SupportedLanguage::from_language_name("Python"),
        Some(SupportedLanguage::Python)
    );
    assert_eq!(
        SupportedLanguage::from_language_name("Go"),
        Some(SupportedLanguage::Go)
    );
    assert_eq!(
        SupportedLanguage::from_language_name("golang"),
        Some(SupportedLanguage::Go)
    );
    assert_eq!(
        SupportedLanguage::from_language_name("Java"),
        Some(SupportedLanguage::Java)
    );
    assert_eq!(
        SupportedLanguage::from_language_name("C"),
        Some(SupportedLanguage::C)
    );
    assert_eq!(
        SupportedLanguage::from_language_name("C++"),
        Some(SupportedLanguage::Cpp)
    );
    assert_eq!(
        SupportedLanguage::from_language_name("cpp"),
        Some(SupportedLanguage::Cpp)
    );
    assert_eq!(SupportedLanguage::from_language_name("Haskell"), None);
}

#[test]
fn test_03_parse_all_supported_languages() {
    let registry = ParserRegistry::new();

    let cases = [
        (
            SupportedLanguage::Rust,
            "pub struct User { pub id: u64, pub name: String }\nimpl User {\n    pub fn new(id: u64, name: String) -> Self { Self { id, name } }\n}",
            "source_file",
        ),
        (
            SupportedLanguage::JavaScript,
            "class AuthService {\n    constructor(secret) { this.secret = secret; }\n    login(token) { return true; }\n}\nmodule.exports = { AuthService };",
            "program",
        ),
        (
            SupportedLanguage::TypeScript,
            "export interface Repository {\n    id: string;\n    name: string;\n    isPrivate: boolean;\n}\nexport type RepoId = string;",
            "program",
        ),
        (
            SupportedLanguage::Tsx,
            "import React from 'react';\nexport const Banner: React.FC<{ title: string }> = ({ title }) => {\n    return <div className=\"banner\"><h1>{title}</h1></div>;\n};",
            "program",
        ),
        (
            SupportedLanguage::Python,
            "from typing import Optional\n\nclass DataPipeline:\n    def __init__(self, name: str):\n        self.name = name\n    \n    async def execute(self) -> bool:\n        return True",
            "module",
        ),
        (
            SupportedLanguage::Go,
            "package main\n\nimport \"fmt\"\n\ntype Server struct {\n    port int\n}\n\nfunc (s *Server) Start() error {\n    fmt.Println(\"Starting\")\n    return nil\n}",
            "source_file",
        ),
        (
            SupportedLanguage::Java,
            "package com.brahm.core;\n\npublic class SecurityManager {\n    private final String key;\n    public SecurityManager(String key) {\n        this.key = key;\n    }\n    public boolean verify() { return true; }\n}",
            "program",
        ),
        (
            SupportedLanguage::C,
            "#include <stdio.h>\n\ntypedef struct {\n    int code;\n    char* message;\n} ErrorResponse;\n\nint handle_request(ErrorResponse* res) {\n    return 0;\n}",
            "translation_unit",
        ),
        (
            SupportedLanguage::Cpp,
            "#include <iostream>\n#include <string>\n\nnamespace brahm {\n    class ArchitectureEngine {\n    public:\n        explicit ArchitectureEngine(std::string name);\n        virtual ~ArchitectureEngine() = default;\n    };\n}",
            "translation_unit",
        ),
    ];

    for (lang, source, expected_root_kind) in cases {
        let parsed = registry
            .parse_source(source, lang)
            .expect("Parsing must succeed");
        assert_eq!(parsed.language, lang);
        assert!(
            !parsed.has_syntax_errors,
            "Syntax error in {:?} snippet",
            lang
        );
        assert_eq!(parsed.root_node().kind(), expected_root_kind);
        assert!(parsed.line_count() >= 2);
        assert_eq!(parsed.byte_len(), source.len());
    }
}

#[test]
fn test_04_source_span_and_node_text() {
    let registry = ParserRegistry::new();
    let source = "fn compute(val: i32) -> i32 {\n    val * 2\n}";
    let parsed = registry
        .parse_source(source, SupportedLanguage::Rust)
        .unwrap();

    let root = parsed.root_node();
    let span = parsed.node_span(&root);
    assert_eq!(span.start_line, 1);
    assert_eq!(span.start_col, 1);
    assert_eq!(span.end_line, 3);
    assert_eq!(span.start_byte, 0);
    assert_eq!(span.end_byte, source.len());

    assert_eq!(parsed.node_text(&root), source);

    let fn_nodes = parsed.find_nodes_by_kind("function_item");
    assert_eq!(fn_nodes.len(), 1);
    let fn_text = parsed.node_text(&fn_nodes[0]);
    assert!(fn_text.starts_with("fn compute"));
}

#[test]
fn test_05_safe_file_size_limit() {
    let dir = tempdir().unwrap();
    let huge_file = dir.path().join("huge.rs");

    // Create a 50 KB file
    let content = "fn dummy() {}\n".repeat(3500); // ~49 KB
    fs::write(&huge_file, content).unwrap();

    // Configure registry with strict 10 KB limit
    let strict_config = ParserConfig::new().with_max_file_size(10 * 1024);
    let registry = ParserRegistry::with_config(strict_config);

    let result = registry.parse_file(&huge_file, None);
    assert!(result.is_err());
    match result.unwrap_err() {
        ParserError::FileTooLarge {
            path,
            size,
            max_size,
        } => {
            assert_eq!(path, huge_file);
            assert!(size > 10 * 1024);
            assert_eq!(max_size, 10 * 1024);
        }
        other => panic!("Expected FileTooLarge error, got {:?}", other),
    }
}

#[test]
fn test_06_error_recovery_and_partial_parsing() {
    let registry = ParserRegistry::new();
    // Syntax with syntax errors (missing braces, incomplete tokens)
    let broken_source = "fn broken_fn( { let x = ; return ";

    // Default configuration allows partial parse with error indicators
    let parsed = registry
        .parse_source(broken_source, SupportedLanguage::Rust)
        .unwrap();
    assert!(parsed.has_syntax_errors);
    let errors = parsed.collect_error_spans();
    assert!(
        !errors.is_empty(),
        "Must collect error spans from malformed code"
    );

    // Strict configuration rejecting partial parses
    let strict_config = ParserConfig::new().with_allow_partial(false);
    let strict_registry = ParserRegistry::with_config(strict_config);
    let result = strict_registry.parse_source(broken_source, SupportedLanguage::Rust);
    assert!(result.is_err());
    match result.unwrap_err() {
        ParserError::ParseFailed { reason, .. } => {
            assert!(reason.contains("allow_partial_parse"));
        }
        other => panic!("Expected ParseFailed, got {:?}", other),
    }
}

#[test]
fn test_07_lossy_utf8_resilience() {
    let dir = tempdir().unwrap();
    let non_utf8_file = dir.path().join("corrupted.py");

    // Write valid python with invalid UTF-8 bytes in a comment
    let mut bytes = b"# Invalid comment: ".to_vec();
    bytes.push(0xFE);
    bytes.push(0xFF);
    bytes.extend_from_slice(b"\ndef calculate():\n    return 42\n");
    fs::write(&non_utf8_file, bytes).unwrap();

    let registry = ParserRegistry::new();
    let parsed = registry
        .parse_file(&non_utf8_file, None)
        .expect("Lossy parsing must handle invalid UTF-8");
    assert_eq!(parsed.language, SupportedLanguage::Python);
    assert!(!parsed.has_syntax_errors);
    assert!(parsed.source.contains("def calculate"));
}

#[test]
fn test_08_registry_detection_and_supported_queries() {
    let registry = ParserRegistry::new();

    let rust_path = std::path::Path::new("src/main.rs");
    let py_path = std::path::Path::new("scripts/deploy.py");
    let ts_path = std::path::Path::new("client/app.tsx");
    let unknown_path = std::path::Path::new("data/archive.bin");

    assert!(registry.is_supported_path(rust_path));
    assert!(registry.is_supported_path(py_path));
    assert!(registry.is_supported_path(ts_path));
    assert!(!registry.is_supported_path(unknown_path));

    assert_eq!(
        registry.detect_language(rust_path, None),
        Some(SupportedLanguage::Rust)
    );
    assert_eq!(
        registry.detect_language(unknown_path, Some("Python")),
        Some(SupportedLanguage::Python)
    );
    assert_eq!(
        registry.detect_language(unknown_path, Some("Unknown")),
        None
    );

    let exts = registry.supported_extensions();
    assert!(exts.contains(&"rs"));
    assert!(exts.contains(&"js"));
    assert!(exts.contains(&"ts"));
    assert!(exts.contains(&"tsx"));
    assert!(exts.contains(&"py"));
    assert!(exts.contains(&"go"));
    assert!(exts.contains(&"java"));
    assert!(exts.contains(&"c"));
    assert!(exts.contains(&"cpp"));
}

#[test]
fn test_09_multithreaded_concurrent_parsing() {
    let registry = Arc::new(ParserRegistry::new());
    let mut handles = Vec::new();

    for i in 0..10 {
        let reg = Arc::clone(&registry);
        let handle = thread::spawn(move || {
            let rust_snippet = format!("pub fn compute_{}() -> i32 {{ {} }}", i, i * 10);
            let parsed_rust = reg
                .parse_source(&rust_snippet, SupportedLanguage::Rust)
                .unwrap();
            assert!(!parsed_rust.has_syntax_errors);

            let py_snippet = format!("def func_{}():\n    return {}", i, i);
            let parsed_py = reg
                .parse_source(&py_snippet, SupportedLanguage::Python)
                .unwrap();
            assert!(!parsed_py.has_syntax_errors);
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }
}
