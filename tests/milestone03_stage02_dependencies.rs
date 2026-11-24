//! Stage 02 Milestone 03 Verification Test Suite
//! Import, Export & Dependency Analysis Engine

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

use brahm::dependencies::{
    CycleDetector, DependencyAnalyzer, ImportKind, PathResolver, ResolutionResult,
};
use brahm::parser::{ParserRegistry, SupportedLanguage};

#[test]
fn test_01_rust_import_extraction() {
    let registry = ParserRegistry::new();
    let code = r#"
use std::collections::HashMap;
pub use crate::parser::{ParserRegistry, SupportedLanguage};
use super::types::Symbol;
use crate::scanner::*;

mod engine;
pub mod helper;
"#;

    let parsed = registry
        .parse_source(code, SupportedLanguage::Rust)
        .unwrap();
    let analyzer = DependencyAnalyzer::new("D:/repo");
    let (imports, exports) = analyzer.extract_file_imports(&parsed);

    assert_eq!(imports.len(), 6); // 4 use statements + 2 mod declarations
    assert_eq!(exports.len(), 1); // 1 pub use re-export

    let std_import = imports
        .iter()
        .find(|i| i.raw_path == "std::collections::HashMap")
        .unwrap();
    assert_eq!(std_import.kind, ImportKind::Named);
    assert!(!std_import.is_relative);
    assert!(std_import.is_external);

    let crate_import = imports
        .iter()
        .find(|i| i.raw_path.starts_with("crate::parser"))
        .unwrap();
    assert_eq!(crate_import.kind, ImportKind::Named);
    assert!(crate_import.is_relative);
    assert!(!crate_import.is_external);
    assert_eq!(crate_import.specifiers.len(), 2);

    let wildcard_import = imports
        .iter()
        .find(|i| i.raw_path == "crate::scanner::*")
        .unwrap();
    assert_eq!(wildcard_import.kind, ImportKind::Wildcard);

    let mod_import = imports
        .iter()
        .find(|i| i.raw_path == "self::engine")
        .unwrap();
    assert!(mod_import.is_relative);
}

#[test]
fn test_02_javascript_typescript_import_extraction() {
    let registry = ParserRegistry::new();
    let code = r#"
import React, { useState, useEffect as useMount } from 'react';
import * as path from 'path';
import './styles.css';
import { AuthService } from './services/auth';

const localModule = require('./local/legacy');
export { UserService } from './services/user';
export default function App() {}
"#;

    let parsed = registry
        .parse_source(code, SupportedLanguage::TypeScript)
        .unwrap();
    let analyzer = DependencyAnalyzer::new("D:/repo");
    let (imports, exports) = analyzer.extract_file_imports(&parsed);

    // 4 top-level imports + 1 require() + 1 re-export import
    assert_eq!(imports.len(), 6);
    assert_eq!(exports.len(), 2); // 1 re-export + 1 default export

    let react_imp = imports.iter().find(|i| i.raw_path == "react").unwrap();
    assert_eq!(react_imp.kind, ImportKind::Named);
    assert!(react_imp.is_external);
    assert!(!react_imp.is_relative);
    assert_eq!(react_imp.specifiers[0].name, "React");
    assert_eq!(react_imp.specifiers[2].alias.as_deref(), Some("useMount"));

    let wildcard_imp = imports.iter().find(|i| i.raw_path == "path").unwrap();
    assert_eq!(wildcard_imp.kind, ImportKind::Wildcard);

    let side_effect_imp = imports
        .iter()
        .find(|i| i.raw_path == "./styles.css")
        .unwrap();
    assert_eq!(side_effect_imp.kind, ImportKind::SideEffect);
    assert!(side_effect_imp.is_relative);

    let dynamic_imp = imports
        .iter()
        .find(|i| i.raw_path == "./local/legacy")
        .unwrap();
    assert_eq!(dynamic_imp.kind, ImportKind::Dynamic);
    assert!(dynamic_imp.is_relative);

    let reexport_imp = imports
        .iter()
        .find(|i| i.raw_path == "./services/user")
        .unwrap();
    assert_eq!(reexport_imp.kind, ImportKind::ReExport);
}

#[test]
fn test_03_python_import_extraction() {
    let registry = ParserRegistry::new();
    let code = r#"
import os, sys
import numpy as np
from .models import User, Account as Acc
from ..utils.helpers import *
from typing import Optional
"#;

    let parsed = registry
        .parse_source(code, SupportedLanguage::Python)
        .unwrap();
    let analyzer = DependencyAnalyzer::new("D:/repo");
    let (imports, _) = analyzer.extract_file_imports(&parsed);

    assert_eq!(imports.len(), 6); // os, sys, numpy, .models, ..utils.helpers, typing

    let os_imp = imports.iter().find(|i| i.raw_path == "os").unwrap();
    assert!(os_imp.is_external);
    assert!(!os_imp.is_relative);

    let np_imp = imports.iter().find(|i| i.raw_path == "numpy").unwrap();
    assert_eq!(np_imp.specifiers[0].alias.as_deref(), Some("np"));

    let rel_models = imports.iter().find(|i| i.raw_path == ".models").unwrap();
    assert!(rel_models.is_relative);
    assert_eq!(rel_models.specifiers.len(), 2);
    assert_eq!(rel_models.specifiers[1].alias.as_deref(), Some("Acc"));

    let wildcard_helpers = imports
        .iter()
        .find(|i| i.raw_path.starts_with("..utils.helpers"))
        .unwrap();
    assert!(wildcard_helpers.is_relative);
    assert_eq!(wildcard_helpers.kind, ImportKind::Wildcard);
}

#[test]
fn test_04_go_java_c_cpp_import_extraction() {
    let registry = ParserRegistry::new();

    // Go
    let go_code = r#"
package main
import (
    "fmt"
    myos "os"
    "github.com/gin-gonic/gin"
)
"#;
    let parsed_go = registry
        .parse_source(go_code, SupportedLanguage::Go)
        .unwrap();
    let analyzer = DependencyAnalyzer::new("D:/repo");
    let (go_imports, _) = analyzer.extract_file_imports(&parsed_go);
    assert_eq!(go_imports.len(), 3);
    assert_eq!(go_imports[1].specifiers[0].alias.as_deref(), Some("myos"));

    // Java
    let java_code = r#"
package com.app;
import java.util.List;
import static org.junit.Assert.*;
"#;
    let parsed_java = registry
        .parse_source(java_code, SupportedLanguage::Java)
        .unwrap();
    let (java_imports, _) = analyzer.extract_file_imports(&parsed_java);
    assert_eq!(java_imports.len(), 2);
    assert_eq!(java_imports[1].kind, ImportKind::Wildcard);

    // C++
    let cpp_code = r#"
#include <iostream>
#include <vector>
#include "security/context.h"
"#;
    let parsed_cpp = registry
        .parse_source(cpp_code, SupportedLanguage::Cpp)
        .unwrap();
    let (cpp_imports, _) = analyzer.extract_file_imports(&parsed_cpp);
    assert_eq!(cpp_imports.len(), 3);
    let local_header = cpp_imports
        .iter()
        .find(|i| i.raw_path == "security/context.h")
        .unwrap();
    assert!(local_header.is_relative);
    let sys_header = cpp_imports
        .iter()
        .find(|i| i.raw_path == "iostream")
        .unwrap();
    assert!(!sys_header.is_relative);
}

#[test]
fn test_05_path_resolver_file_matching() {
    let dir = tempdir().unwrap();
    let root = dir.path().to_path_buf();

    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();

    let app_ts = src.join("app.ts");
    let utils_ts = src.join("utils.ts");
    let auth_index = src.join("auth").join("index.ts");
    fs::create_dir_all(src.join("auth")).unwrap();

    fs::write(&app_ts, "import { add } from './utils';").unwrap();
    fs::write(&utils_ts, "export const add = 1;").unwrap();
    fs::write(&auth_index, "export const auth = 2;").unwrap();

    let mut resolver = PathResolver::new(&root);
    resolver.add_known_file(resolver.normalize_path(&app_ts));
    resolver.add_known_file(resolver.normalize_path(&utils_ts));
    resolver.add_known_file(resolver.normalize_path(&auth_index));

    // Resolve `./utils` -> `utils.ts`
    let res1 = resolver.resolve_import(&app_ts, "./utils", SupportedLanguage::TypeScript);
    match res1 {
        ResolutionResult::Internal(target) => {
            assert_eq!(target, resolver.normalize_path(&utils_ts));
        }
        other => panic!("Expected Internal, got {:?}", other),
    }

    // Resolve `./auth` -> `auth/index.ts`
    let res2 = resolver.resolve_import(&app_ts, "./auth", SupportedLanguage::TypeScript);
    match res2 {
        ResolutionResult::Internal(target) => {
            assert_eq!(target, resolver.normalize_path(&auth_index));
        }
        other => panic!("Expected Internal, got {:?}", other),
    }

    // Resolve `react` -> External
    let res3 = resolver.resolve_import(&app_ts, "react", SupportedLanguage::TypeScript);
    assert_eq!(res3, ResolutionResult::External("react".to_string()));
}

#[test]
fn test_06_cycle_detection_algorithms() {
    let f1 = PathBuf::from("service/auth.ts");
    let f2 = PathBuf::from("service/user.ts");
    let f3 = PathBuf::from("service/session.ts");
    let f4 = PathBuf::from("utils/logger.ts");

    let mut graph = HashMap::new();
    // 3-node cycle: auth -> user -> session -> auth
    graph.insert(f1.clone(), vec![f2.clone(), f4.clone()]);
    graph.insert(f2.clone(), vec![f3.clone()]);
    graph.insert(f3.clone(), vec![f1.clone()]);
    graph.insert(f4.clone(), vec![]); // logger has no dependencies

    let cycles = CycleDetector::find_cycles(&graph);
    assert_eq!(cycles.len(), 1);
    assert_eq!(cycles[0].length(), 3);

    let cycle_str = cycles[0].display_path(None);
    assert!(cycle_str.contains("auth.ts"));
    assert!(cycle_str.contains("user.ts"));
    assert!(cycle_str.contains("session.ts"));
}

#[test]
fn test_07_full_repository_dependency_analysis() {
    let dir = tempdir().unwrap();
    let root = dir.path().to_path_buf();
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();

    let a_file = src.join("a.py");
    let b_file = src.join("b.py");

    // Mutual dependency cycle: a.py imports b, b.py imports a
    fs::write(&a_file, "import sys\nfrom .b import helper_b\n").unwrap();
    fs::write(&b_file, "import os\nfrom .a import helper_a\n").unwrap();

    let registry = ParserRegistry::new();
    let parsed_a = registry.parse_file(&a_file, Some("python")).unwrap();
    let parsed_b = registry.parse_file(&b_file, Some("python")).unwrap();

    let analyzer =
        DependencyAnalyzer::new(&root).with_known_files(vec![a_file.clone(), b_file.clone()]);

    let result = analyzer.analyze_repository(&[parsed_a, parsed_b]);

    assert_eq!(result.stats.total_imports, 4); // 2 external + 2 internal
    assert_eq!(result.stats.internal_imports, 2);
    assert_eq!(result.stats.external_imports, 2);
    assert_eq!(result.stats.unique_external_packages, 2); // sys, os
    assert!(result.unique_external_packages.contains("sys"));
    assert!(result.unique_external_packages.contains("os"));

    // Cycle detected
    assert_eq!(result.cycles.len(), 1);
    assert_eq!(result.stats.circular_dependency_count, 1);
}
