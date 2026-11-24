//! Evidence-based framework and ecosystem detection.
//!
//! Evaluates manifests and configuration files to establish deterministic
//! architectural confidence without speculative semantic parsing.

use crate::models::{ConfidenceLevel, FileEntry, FrameworkIndicator};
use serde_json::Value;
use std::fs;
use std::path::Path;

/// Analyzes discovered repository files and parses manifests to detect frameworks.
pub fn detect_frameworks(repo_root: &Path, files: &[FileEntry]) -> Vec<FrameworkIndicator> {
    let mut detected: Vec<FrameworkIndicator> = Vec::new();

    // 1. Node.js, React, Next.js, Vite detection via package.json
    detect_node_ecosystem(repo_root, files, &mut detected);

    // 2. Rust detection via Cargo.toml
    detect_rust_ecosystem(repo_root, files, &mut detected);

    // 3. Python, Django, Flask detection
    detect_python_ecosystem(repo_root, files, &mut detected);

    // 4. Go detection via go.mod
    detect_go_ecosystem(files, &mut detected);

    // 5. Java detection via pom.xml and build.gradle
    detect_java_ecosystem(files, &mut detected);

    detected
}

fn add_or_merge_framework(
    detected: &mut Vec<FrameworkIndicator>,
    name: &str,
    confidence: ConfidenceLevel,
    evidence: Vec<String>,
) {
    if let Some(existing) = detected.iter_mut().find(|f| f.name == name) {
        // Upgrade confidence if higher
        if confidence == ConfidenceLevel::ConfirmedByManifest {
            existing.confidence = ConfidenceLevel::ConfirmedByManifest;
        }
        for ev in evidence {
            if !existing.evidence.contains(&ev) {
                existing.evidence.push(ev);
            }
        }
    } else {
        detected.push(FrameworkIndicator {
            name: name.to_string(),
            confidence,
            evidence,
        });
    }
}

fn detect_node_ecosystem(
    repo_root: &Path,
    files: &[FileEntry],
    detected: &mut Vec<FrameworkIndicator>,
) {
    let package_json_entries: Vec<&FileEntry> = files
        .iter()
        .filter(|f| f.filename == "package.json")
        .collect();

    let has_next_config = files.iter().any(|f| {
        f.filename == "next.config.js"
            || f.filename == "next.config.ts"
            || f.filename == "next.config.mjs"
    });
    let has_vite_config = files.iter().any(|f| {
        f.filename == "vite.config.ts"
            || f.filename == "vite.config.js"
            || f.filename == "vite.config.mjs"
    });

    if !package_json_entries.is_empty() {
        for entry in package_json_entries {
            let manifest_path = repo_root.join(&entry.relative_path);
            let evidence = vec![format!("Identified '{}' manifest", entry.relative_path)];

            add_or_merge_framework(
                detected,
                "Node.js",
                ConfidenceLevel::ConfirmedByManifest,
                evidence,
            );

            let mut has_react = false;
            let mut has_next = false;
            let mut has_vite = false;

            if let Ok(content) = fs::read_to_string(&manifest_path) {
                if let Ok(json) = serde_json::from_str::<Value>(&content) {
                    let check_dep = |dep_name: &str| -> bool {
                        let in_deps = json
                            .get("dependencies")
                            .and_then(|d| d.get(dep_name))
                            .is_some();
                        let in_dev = json
                            .get("devDependencies")
                            .and_then(|d| d.get(dep_name))
                            .is_some();
                        let in_peer = json
                            .get("peerDependencies")
                            .and_then(|d| d.get(dep_name))
                            .is_some();
                        in_deps || in_dev || in_peer
                    };

                    has_react = check_dep("react");
                    has_next = check_dep("next");
                    has_vite = check_dep("vite");
                }
            }

            // Next.js
            if has_next {
                let mut next_ev = vec![format!(
                    "'{}' declares 'next' dependency",
                    entry.relative_path
                )];
                if has_next_config {
                    next_ev.push("Next.js configuration file present".to_string());
                }
                add_or_merge_framework(
                    detected,
                    "Next.js",
                    ConfidenceLevel::ConfirmedByManifest,
                    next_ev,
                );
            }

            // React
            if has_react || has_next {
                let mut react_ev = vec![];
                if has_react {
                    react_ev.push(format!(
                        "'{}' declares 'react' dependency",
                        entry.relative_path
                    ));
                }
                if has_next {
                    react_ev.push("React implicitly required by Next.js framework".to_string());
                }
                add_or_merge_framework(
                    detected,
                    "React",
                    ConfidenceLevel::ConfirmedByManifest,
                    react_ev,
                );
            }

            // Vite
            if has_vite {
                let mut vite_ev = vec![format!(
                    "'{}' declares 'vite' dependency",
                    entry.relative_path
                )];
                if has_vite_config {
                    vite_ev.push("Vite configuration file present".to_string());
                }
                add_or_merge_framework(
                    detected,
                    "Vite",
                    ConfidenceLevel::ConfirmedByManifest,
                    vite_ev,
                );
            }
        }
    } else {
        // No package.json, but configs might exist
        if has_next_config {
            add_or_merge_framework(
                detected,
                "Next.js",
                ConfidenceLevel::StronglyIndicated,
                vec!["next.config.* present in repository".to_string()],
            );
        }
        if has_vite_config {
            add_or_merge_framework(
                detected,
                "Vite",
                ConfidenceLevel::StronglyIndicated,
                vec!["vite.config.* present in repository".to_string()],
            );
        }
    }
}

fn detect_rust_ecosystem(
    repo_root: &Path,
    files: &[FileEntry],
    detected: &mut Vec<FrameworkIndicator>,
) {
    let cargo_entry = files.iter().find(|f| f.filename == "Cargo.toml");
    let has_rs_files = files.iter().any(|f| f.extension.as_deref() == Some("rs"));

    if let Some(entry) = cargo_entry {
        let manifest_path = repo_root.join(&entry.relative_path);
        let mut evidence = vec![format!("Found '{}' manifest", entry.relative_path)];

        if let Ok(content) = fs::read_to_string(&manifest_path) {
            if content.contains("[package]") {
                evidence.push("Cargo.toml contains valid [package] section".to_string());
            }
        }

        add_or_merge_framework(
            detected,
            "Rust",
            ConfidenceLevel::ConfirmedByManifest,
            evidence,
        );
    } else if has_rs_files {
        add_or_merge_framework(
            detected,
            "Rust",
            ConfidenceLevel::Possible,
            vec!["Rust source files (.rs) detected without Cargo.toml manifest".to_string()],
        );
    }
}

fn detect_python_ecosystem(
    repo_root: &Path,
    files: &[FileEntry],
    detected: &mut Vec<FrameworkIndicator>,
) {
    let req_entry = files.iter().find(|f| f.filename == "requirements.txt");
    let pyproject_entry = files.iter().find(|f| f.filename == "pyproject.toml");
    let has_manage_py = files.iter().any(|f| f.filename == "manage.py");

    let mut py_content = String::new();

    if let Some(entry) = req_entry {
        if let Ok(c) = fs::read_to_string(repo_root.join(&entry.relative_path)) {
            py_content.push_str(&c);
            py_content.push('\n');
        }
    }
    if let Some(entry) = pyproject_entry {
        if let Ok(c) = fs::read_to_string(repo_root.join(&entry.relative_path)) {
            py_content.push_str(&c);
            py_content.push('\n');
        }
    }

    let lower_manifest = py_content.to_ascii_lowercase();

    // Django
    if lower_manifest.contains("django") {
        let mut evidence = vec!["Python manifest declares django dependency".to_string()];
        if has_manage_py {
            evidence.push("Found manage.py entrypoint".to_string());
        }
        add_or_merge_framework(
            detected,
            "Django",
            ConfidenceLevel::ConfirmedByManifest,
            evidence,
        );
    } else if has_manage_py {
        add_or_merge_framework(
            detected,
            "Django",
            ConfidenceLevel::StronglyIndicated,
            vec!["Found standard Django manage.py entrypoint".to_string()],
        );
    }

    // Flask
    if lower_manifest.contains("flask") {
        add_or_merge_framework(
            detected,
            "Flask",
            ConfidenceLevel::ConfirmedByManifest,
            vec!["Python manifest declares flask dependency".to_string()],
        );
    }
}

fn detect_go_ecosystem(files: &[FileEntry], detected: &mut Vec<FrameworkIndicator>) {
    let go_mod = files.iter().any(|f| f.filename == "go.mod");
    let has_go_files = files.iter().any(|f| f.extension.as_deref() == Some("go"));

    if go_mod {
        add_or_merge_framework(
            detected,
            "Go",
            ConfidenceLevel::ConfirmedByManifest,
            vec!["go.mod module manifest present in repository".to_string()],
        );
    } else if has_go_files {
        add_or_merge_framework(
            detected,
            "Go",
            ConfidenceLevel::Possible,
            vec!["Go source files (.go) discovered without go.mod".to_string()],
        );
    }
}

fn detect_java_ecosystem(files: &[FileEntry], detected: &mut Vec<FrameworkIndicator>) {
    let has_pom = files.iter().any(|f| f.filename == "pom.xml");
    let has_gradle = files
        .iter()
        .any(|f| f.filename == "build.gradle" || f.filename == "build.gradle.kts");
    let has_java_files = files.iter().any(|f| f.extension.as_deref() == Some("java"));

    if has_pom {
        add_or_merge_framework(
            detected,
            "Java (Maven)",
            ConfidenceLevel::ConfirmedByManifest,
            vec!["pom.xml Maven project definition present".to_string()],
        );
    } else if has_gradle {
        add_or_merge_framework(
            detected,
            "Java (Gradle)",
            ConfidenceLevel::ConfirmedByManifest,
            vec!["build.gradle Gradle project definition present".to_string()],
        );
    } else if has_java_files {
        add_or_merge_framework(
            detected,
            "Java",
            ConfidenceLevel::Possible,
            vec![
                "Java source files (.java) discovered without Maven/Gradle build manifests"
                    .to_string(),
            ],
        );
    }
}
