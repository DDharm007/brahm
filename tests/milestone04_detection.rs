//! Milestone 04 Language, Framework, and Configuration Detection Integration Tests

use brahm::config::BrahmConfig;
use brahm::detection::{
    detect_frameworks, detect_language, is_configuration_or_manifest, is_manifest,
    is_sensitive_credential_file,
};
use brahm::models::ConfidenceLevel;
use brahm::scanner::Scanner;

use std::fs;
use tempfile::tempdir;

#[test]
fn test_all_25_languages_detected() {
    let test_cases = vec![
        ("main.rs", Some("rs"), "Rust"),
        ("app.js", Some("js"), "JavaScript"),
        ("app.mjs", Some("mjs"), "JavaScript"),
        ("index.ts", Some("ts"), "TypeScript"),
        ("component.tsx", Some("tsx"), "TypeScript"),
        ("script.py", Some("py"), "Python"),
        ("App.java", Some("java"), "Java"),
        ("main.go", Some("go"), "Go"),
        ("driver.c", Some("c"), "C"),
        ("engine.cpp", Some("cpp"), "C++"),
        ("Program.cs", Some("cs"), "C#"),
        ("index.php", Some("php"), "PHP"),
        ("gem.rb", Some("rb"), "Ruby"),
        ("view.swift", Some("swift"), "Swift"),
        ("Service.kt", Some("kt"), "Kotlin"),
        ("widget.dart", Some("dart"), "Dart"),
        ("index.html", Some("html"), "HTML"),
        ("style.css", Some("css"), "CSS"),
        ("styles.scss", Some("scss"), "SCSS"),
        ("query.sql", Some("sql"), "SQL"),
        ("deploy.sh", Some("sh"), "Shell"),
        ("setup.ps1", Some("ps1"), "PowerShell"),
        ("data.json", Some("json"), "JSON"),
        ("ci.yaml", Some("yaml"), "YAML"),
        ("Cargo.toml", Some("toml"), "TOML"),
        ("pom.xml", Some("xml"), "XML"),
        ("README.md", Some("md"), "Markdown"),
    ];

    for (filename, ext, expected_lang) in test_cases {
        let detected = detect_language(filename, ext);
        assert_eq!(
            detected.as_deref(),
            Some(expected_lang),
            "Failed for file '{}' with extension '{:?}'",
            filename,
            ext
        );
    }
}

#[test]
fn test_framework_detection_react_and_next() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let package_json = r#"{
        "name": "my-next-app",
        "version": "1.0.0",
        "dependencies": {
            "next": "^14.0.0",
            "react": "^18.2.0",
            "react-dom": "^18.2.0"
        }
    }"#;
    fs::write(root.join("package.json"), package_json).unwrap();
    fs::write(root.join("next.config.js"), "module.exports = {};").unwrap();
    fs::create_dir_all(root.join("src").join("pages")).unwrap();
    fs::write(
        root.join("src").join("pages").join("index.tsx"),
        "export default function Home() { return <div>Hello</div>; }",
    )
    .unwrap();

    let config = BrahmConfig::default();
    let scanner = Scanner::new(root, config);
    let output = scanner.scan();

    let frameworks = detect_frameworks(root, &output.files);

    let next = frameworks.iter().find(|f| f.name == "Next.js");
    assert!(next.is_some());
    assert_eq!(
        next.unwrap().confidence,
        ConfidenceLevel::ConfirmedByManifest
    );

    let react = frameworks.iter().find(|f| f.name == "React");
    assert!(react.is_some());
    assert_eq!(
        react.unwrap().confidence,
        ConfidenceLevel::ConfirmedByManifest
    );

    let node = frameworks.iter().find(|f| f.name == "Node.js");
    assert!(node.is_some());
    assert_eq!(
        node.unwrap().confidence,
        ConfidenceLevel::ConfirmedByManifest
    );
}

#[test]
fn test_framework_detection_vite() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let package_json = r#"{
        "name": "vite-react-app",
        "devDependencies": {
            "vite": "^5.0.0"
        }
    }"#;
    fs::write(root.join("package.json"), package_json).unwrap();
    fs::write(root.join("vite.config.ts"), "export default {};").unwrap();

    let config = BrahmConfig::default();
    let scanner = Scanner::new(root, config);
    let output = scanner.scan();

    let frameworks = detect_frameworks(root, &output.files);
    let vite = frameworks.iter().find(|f| f.name == "Vite");
    assert!(vite.is_some());
    assert_eq!(
        vite.unwrap().confidence,
        ConfidenceLevel::ConfirmedByManifest
    );
}

#[test]
fn test_framework_detection_rust() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let cargo_toml = r#"[package]
name = "sample-rust"
version = "0.1.0"
edition = "2021"
"#;
    fs::write(root.join("Cargo.toml"), cargo_toml).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src").join("main.rs"), "fn main() {}").unwrap();

    let config = BrahmConfig::default();
    let scanner = Scanner::new(root, config);
    let output = scanner.scan();

    let frameworks = detect_frameworks(root, &output.files);
    let rust = frameworks.iter().find(|f| f.name == "Rust");
    assert!(rust.is_some());
    assert_eq!(
        rust.unwrap().confidence,
        ConfidenceLevel::ConfirmedByManifest
    );
}

#[test]
fn test_framework_detection_django_and_flask() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let reqs = "Django>=4.2\npsycopg2-binary>=2.9\n";
    fs::write(root.join("requirements.txt"), reqs).unwrap();
    fs::write(root.join("manage.py"), "#!/usr/bin/env python\n").unwrap();

    let config = BrahmConfig::default();
    let scanner = Scanner::new(root, config);
    let output = scanner.scan();

    let frameworks = detect_frameworks(root, &output.files);
    let django = frameworks.iter().find(|f| f.name == "Django");
    assert!(django.is_some());
    assert_eq!(
        django.unwrap().confidence,
        ConfidenceLevel::ConfirmedByManifest
    );
}

#[test]
fn test_framework_detection_go_and_java() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    fs::write(root.join("go.mod"), "module example.com/mygo\n\ngo 1.22\n").unwrap();
    fs::write(root.join("pom.xml"), "<project></project>").unwrap();

    let config = BrahmConfig::default();
    let scanner = Scanner::new(root, config);
    let output = scanner.scan();

    let frameworks = detect_frameworks(root, &output.files);
    assert!(frameworks
        .iter()
        .any(|f| f.name == "Go" && f.confidence == ConfidenceLevel::ConfirmedByManifest));
    assert!(frameworks
        .iter()
        .any(|f| f.name == "Java (Maven)" && f.confidence == ConfidenceLevel::ConfirmedByManifest));
}

#[test]
fn test_manifest_and_sensitive_file_protections() {
    assert!(is_manifest("Cargo.toml"));
    assert!(is_manifest("package.json"));
    assert!(is_manifest("go.mod"));
    assert!(is_configuration_or_manifest("tsconfig.json"));
    assert!(is_configuration_or_manifest("Dockerfile"));

    assert!(is_sensitive_credential_file(".env"));
    assert!(is_sensitive_credential_file(".env.local"));
    assert!(is_sensitive_credential_file("id_rsa"));
    assert!(is_sensitive_credential_file("database_secret.key"));
    assert!(!is_sensitive_credential_file(".env.example"));
}
