//! Stage 02 Architecture Intelligence Orchestrator.
//!
//! Coordinates AST parsing, symbol indexing, dependency analysis, application landmark detection,
//! graph construction, architectural layer inference, and hotspot metrics.

use chrono::Utc;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::{debug, info};

use super::types::{
    ArchitecturalLayer, ArchitecturalPatternSummary, ArchitectureAnalysisBundle,
    ArchitectureMetrics, ArchitectureReport, ComponentHotspot, DependencyIntelligence,
    FactVsInference, UnresolvedImport,
};
use crate::dependencies::analyzer::{DependencyAnalysisResult, DependencyAnalyzer};
use crate::dependencies::types::DependencyCycle;
use crate::detection::is_sensitive_credential_file;
use crate::errors::Result;
use crate::graph::{GraphBuilder, GraphExport};
use crate::models::{FileClassification, RepositoryReport};
use crate::parser::{ParsedSource, ParserRegistry};
use crate::structure::analyzer::StructureAnalyzer;
use crate::structure::datamodels::DataModelDetector;
use crate::symbols::extractor::{DefaultSymbolExtractor, SymbolExtractor};
use crate::symbols::SymbolIndex;

/// Main orchestrator for BRAHM Stage 02 Code Intelligence & Architecture Mapping.
#[derive(Debug, Default, Clone)]
pub struct ArchitectureEngine;

impl ArchitectureEngine {
    /// Initialize a new `ArchitectureEngine`.
    pub fn new() -> Self {
        Self
    }

    /// Execute the full Stage 02 static analysis pipeline over a repository,
    /// building on top of Stage 01's `RepositoryReport`.
    pub fn analyze(
        &self,
        repo_root: &Path,
        stage1_report: &RepositoryReport,
    ) -> Result<ArchitectureAnalysisBundle> {
        let start_instant = Instant::now();
        let canonical_root =
            fs::canonicalize(repo_root).unwrap_or_else(|_| repo_root.to_path_buf());

        info!(
            "Starting Stage 02 architecture analysis for '{}'",
            stage1_report.repository.name
        );

        let registry = ParserRegistry::new();
        let symbol_extractor = DefaultSymbolExtractor::new();
        let structure_analyzer = StructureAnalyzer::new();
        let model_detector = DataModelDetector::new();
        let graph_builder = GraphBuilder::new();

        let mut parsed_files: Vec<ParsedSource> = Vec::new();
        let mut known_abs_paths: Vec<PathBuf> = Vec::new();
        let mut extra_data_models = Vec::new();
        let mut warnings = Vec::new();
        let mut languages_parsed_set = BTreeSet::new();

        // 1. Parse AST for supported source files & extract non-AST database schemas (.sql/.prisma)
        for entry in &stage1_report.files {
            if entry.is_binary || is_sensitive_credential_file(&entry.filename) {
                continue;
            }

            let abs_path = canonical_root.join(&entry.relative_path);
            let norm_abs = fs::canonicalize(&abs_path).unwrap_or_else(|_| abs_path.clone());
            known_abs_paths.push(norm_abs.clone());

            let lower_name = entry.filename.to_ascii_lowercase();
            if lower_name.ends_with(".sql") || lower_name.ends_with(".prisma") {
                if let Ok(content) = fs::read_to_string(&norm_abs) {
                    let rel_path = PathBuf::from(&entry.relative_path);
                    extra_data_models.extend(model_detector.detect_from_text(&rel_path, &content));
                }
                continue;
            }

            if !registry.is_supported_path(&norm_abs) {
                continue;
            }

            match registry.parse_file(&norm_abs, entry.language.as_deref()) {
                Ok(parsed) => {
                    if parsed.has_syntax_errors {
                        warnings.push(format!(
                            "Syntax errors recovered during AST parse of '{}'",
                            entry.relative_path
                        ));
                    }
                    languages_parsed_set.insert(parsed.language.display_name().to_string());
                    parsed_files.push(parsed);
                }
                Err(e) => {
                    debug!("Skipped parsing '{}': {}", entry.relative_path, e);
                    warnings.push(format!(
                        "Skipped AST parse of '{}': {}",
                        entry.relative_path, e
                    ));
                }
            }
        }

        // 2. Run Dependency & Cycle Analysis using canonical paths for exact resolution
        let dep_analyzer =
            DependencyAnalyzer::new(&canonical_root).with_known_files(known_abs_paths);
        let raw_dependencies = dep_analyzer.analyze_repository(&parsed_files);

        // Relativize paths in parsed_files and dependency_results so outputs are clean & cross-platform
        for parsed in &mut parsed_files {
            if let Some(ref abs_p) = parsed.path {
                parsed.path = Some(to_relative_path(abs_p, &canonical_root, repo_root));
            }
        }
        let dependencies = relativize_dependencies(raw_dependencies, &canonical_root, repo_root);

        // 3. Extract Symbols & Build Symbol Index
        let mut symbol_index = SymbolIndex::new();
        for parsed in &parsed_files {
            let syms = symbol_extractor.extract_symbols(parsed);
            symbol_index.extend(syms);
        }

        // 4. Detect Application Landmarks (Entry Points, API Routes, ORM Models)
        let mut structure = structure_analyzer.analyze(&parsed_files, &symbol_index);
        structure.data_models.extend(extra_data_models);
        structure.summary.data_model_count = structure.data_models.len();

        // 5. Build Architecture & Dependency Graph (petgraph)
        let graph = graph_builder.build(&parsed_files, &symbol_index, &dependencies, &structure);
        let graph_export = GraphExport::from_graph(&graph);

        // 6. Infer Architectural Pattern & Layers
        let languages_parsed: Vec<String> = languages_parsed_set.into_iter().collect();
        let detected_frameworks: Vec<String> = stage1_report
            .frameworks
            .iter()
            .map(|f| f.name.clone())
            .collect();

        let pattern = infer_architectural_pattern(
            &detected_frameworks,
            &languages_parsed,
            &structure.entrypoints,
            &structure.routes,
            &structure.data_models,
            stage1_report,
        );

        let layers = build_architectural_layers(
            stage1_report,
            &symbol_index,
            &structure.routes,
            &structure.data_models,
        );

        // 7. Compute Hotspots & Blast Radius
        let hotspots = compute_hotspots(&graph);

        // 8. Collect Unresolved Imports & Dependency Intelligence
        let mut unresolved_internal_imports = Vec::new();
        for imp in &dependencies.imports {
            if imp.is_relative && imp.resolved_target.is_none() && !imp.is_external {
                unresolved_internal_imports.push(UnresolvedImport {
                    source_file: imp.source_file.to_string_lossy().replace('\\', "/"),
                    raw_import_path: imp.raw_path.clone(),
                    line: imp.span.start_line,
                });
            }
        }

        let mut external_packages: Vec<String> = dependencies
            .unique_external_packages
            .iter()
            .cloned()
            .collect();
        external_packages.sort();

        let graph_scc_cycles = graph.find_cyclic_components();

        let dep_intelligence = DependencyIntelligence {
            external_packages,
            circular_dependencies: dependencies.cycles.clone(),
            graph_scc_cycles,
            unresolved_internal_imports: unresolved_internal_imports.clone(),
        };

        // 9. Compile Metrics
        let symbol_summary = symbol_index.summary();
        let files_with_syntax_errors = parsed_files.iter().filter(|p| p.has_syntax_errors).count();

        let metrics = ArchitectureMetrics {
            total_files_scanned: stage1_report.statistics.total_files,
            files_parsed_ast: parsed_files.len(),
            files_with_syntax_errors,
            total_symbols: symbol_summary.total_symbols,
            symbols_by_kind: symbol_summary.by_kind,
            total_imports: dependencies.stats.total_imports,
            internal_imports: dependencies.stats.internal_imports,
            external_imports: dependencies.stats.external_imports,
            unique_external_packages: dependencies.stats.unique_external_packages,
            circular_dependency_count: dependencies.stats.circular_dependency_count,
            entry_point_count: structure.entrypoints.len(),
            api_route_count: structure.routes.len(),
            data_model_count: structure.data_models.len(),
            graph_nodes: graph.node_count(),
            graph_edges: graph.edge_count(),
            graph_is_dag: !graph.has_cycles(),
            graph_density: graph.density(),
        };

        // 10. Explicitly Separate Facts vs. Inferences vs. Unresolved References
        let confidence_breakdown = build_fact_vs_inference(
            &metrics,
            &pattern,
            &layers,
            &unresolved_internal_imports,
            &graph_export,
        );

        let duration_ms = start_instant.elapsed().as_millis();

        let report = ArchitectureReport {
            schema_version: "2.0".to_string(),
            brahm_version: env!("CARGO_PKG_VERSION").to_string(),
            generated_at: Utc::now(),
            analysis_duration_ms: duration_ms,
            repository: stage1_report.repository.clone(),
            pattern,
            metrics,
            layers,
            entry_points: structure.entrypoints,
            api_routes: structure.routes,
            data_models: structure.data_models,
            dependencies: dep_intelligence,
            hotspots,
            confidence_breakdown,
            warnings,
        };

        Ok(ArchitectureAnalysisBundle {
            report,
            graph,
            graph_export,
            symbol_index,
            parsed_files,
        })
    }
}

/// Convert an absolute canonical path into a clean relative path with forward slashes.
fn to_relative_path(path: &Path, canonical_root: &Path, repo_root: &Path) -> PathBuf {
    let stripped = path
        .strip_prefix(canonical_root)
        .or_else(|_| path.strip_prefix(repo_root))
        .unwrap_or(path);

    let clean_str = stripped.to_string_lossy().replace('\\', "/");
    let clean_trimmed = clean_str.trim_start_matches("./");
    PathBuf::from(clean_trimmed)
}

/// Relativize all path keys and values inside `DependencyAnalysisResult`.
fn relativize_dependencies(
    mut deps: DependencyAnalysisResult,
    canonical_root: &Path,
    repo_root: &Path,
) -> DependencyAnalysisResult {
    for imp in &mut deps.imports {
        imp.source_file = to_relative_path(&imp.source_file, canonical_root, repo_root);
        if let Some(ref target) = imp.resolved_target {
            imp.resolved_target = Some(to_relative_path(target, canonical_root, repo_root));
        }
    }

    for exp in &mut deps.exports {
        exp.source_file = to_relative_path(&exp.source_file, canonical_root, repo_root);
    }

    let mut new_internal = HashMap::new();
    for (src, targets) in deps.internal_dependencies {
        let rel_src = to_relative_path(&src, canonical_root, repo_root);
        let rel_targets: Vec<PathBuf> = targets
            .into_iter()
            .map(|t| to_relative_path(&t, canonical_root, repo_root))
            .collect();
        new_internal.insert(rel_src, rel_targets);
    }
    deps.internal_dependencies = new_internal;

    let mut new_external = HashMap::new();
    for (src, pkgs) in deps.external_dependencies {
        let rel_src = to_relative_path(&src, canonical_root, repo_root);
        new_external.insert(rel_src, pkgs);
    }
    deps.external_dependencies = new_external;

    deps.cycles = deps
        .cycles
        .into_iter()
        .map(|c| DependencyCycle {
            cycle: c
                .cycle
                .into_iter()
                .map(|p| to_relative_path(&p, canonical_root, repo_root))
                .collect(),
        })
        .collect();

    deps
}

/// Infer the dominant architectural pattern from static landmarks.
fn infer_architectural_pattern(
    frameworks: &[String],
    languages_parsed: &[String],
    entrypoints: &[crate::structure::types::EntryPoint],
    routes: &[crate::structure::types::ApiRoute],
    data_models: &[crate::structure::types::DataModel],
    stage1_report: &RepositoryReport,
) -> ArchitecturalPatternSummary {
    let mut rationale = Vec::new();
    let has_frontend = frameworks
        .iter()
        .any(|f| matches!(f.as_str(), "React" | "Next.js" | "Vue" | "Angular" | "Vite"));
    let has_backend_routes = !routes.is_empty();
    let has_db = !data_models.is_empty();
    let has_docker = stage1_report
        .files
        .iter()
        .any(|f| f.filename == "Dockerfile" || f.filename == "docker-compose.yml");

    let (primary_pattern, confidence) = if has_frontend && (has_backend_routes || has_db) {
        rationale.push(
            "Combined frontend UI framework with backend API endpoints or database persistence"
                .to_string(),
        );
        ("Full-Stack Web Application".to_string(), "High".to_string())
    } else if languages_parsed.len() >= 3 && has_docker {
        rationale.push(format!(
            "Polyglot codebase parsing {} languages with container orchestration manifests",
            languages_parsed.len()
        ));
        (
            "Polyglot Multi-Service Architecture".to_string(),
            "High".to_string(),
        )
    } else if has_backend_routes {
        rationale.push(format!(
            "Explicit HTTP routing layer with {} endpoint(s)",
            routes.len()
        ));
        if has_db {
            rationale.push(format!(
                "Backed by {} persistence schema(s)/ORM model(s)",
                data_models.len()
            ));
        }
        ("REST API / Backend Service".to_string(), "High".to_string())
    } else if has_frontend {
        rationale.push(
            "Manifest-confirmed frontend client framework without standalone backend routes"
                .to_string(),
        );
        (
            "Frontend Single-Page Application (SPA)".to_string(),
            "High".to_string(),
        )
    } else if entrypoints
        .iter()
        .any(|e| e.kind == crate::structure::types::EntryPointKind::LibraryRoot)
        && !entrypoints
            .iter()
            .any(|e| e.kind == crate::structure::types::EntryPointKind::Executable)
    {
        rationale.push(
            "Exposes library root entry point without standalone executable main".to_string(),
        );
        (
            "Modular Software Library / SDK".to_string(),
            "High".to_string(),
        )
    } else if !entrypoints.is_empty() {
        rationale.push(format!(
            "Detected {} executable/server entry point(s)",
            entrypoints.len()
        ));
        (
            "CLI / Standalone Application".to_string(),
            "Medium".to_string(),
        )
    } else {
        rationale.push(
            "General source repository without specialized web/service landmarks".to_string(),
        );
        ("Modular Software Repository".to_string(), "Low".to_string())
    };

    ArchitecturalPatternSummary {
        primary_pattern,
        confidence,
        detected_frameworks: frameworks.to_vec(),
        languages_parsed: languages_parsed.to_vec(),
        rationale,
    }
}

/// Group repository files into logical architectural layers.
fn build_architectural_layers(
    stage1_report: &RepositoryReport,
    symbol_index: &SymbolIndex,
    routes: &[crate::structure::types::ApiRoute],
    data_models: &[crate::structure::types::DataModel],
) -> Vec<ArchitecturalLayer> {
    struct LayerAccum {
        role: String,
        dirs: BTreeSet<String>,
        files: usize,
        symbols: usize,
        routes: usize,
    }

    let mut layer_map: BTreeMap<String, LayerAccum> = BTreeMap::new();

    for file in &stage1_report.files {
        let norm_rel = file.relative_path.replace('\\', "/");
        let dir_part = Path::new(&norm_rel)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| ".".to_string());

        let (layer_name, role) = classify_layer(&norm_rel, file.classification);

        let entry = layer_map.entry(layer_name).or_insert_with(|| LayerAccum {
            role,
            dirs: BTreeSet::new(),
            files: 0,
            symbols: 0,
            routes: 0,
        });

        entry.dirs.insert(dir_part);
        entry.files += 1;

        let file_rel_path = Path::new(&norm_rel);
        let syms_in_file = symbol_index.get_by_file(file_rel_path).len();
        entry.symbols += syms_in_file;

        let routes_in_file = routes
            .iter()
            .filter(|r| r.file_path.to_string_lossy().replace('\\', "/") == norm_rel)
            .count();
        entry.routes += routes_in_file;
    }

    // Ensure data models also attribute layer visibility if needed
    let _ = data_models;

    layer_map
        .into_iter()
        .map(|(name, acc)| ArchitecturalLayer {
            name,
            role: acc.role,
            directory_paths: acc.dirs.into_iter().collect(),
            file_count: acc.files,
            symbol_count: acc.symbols,
            route_count: acc.routes,
        })
        .collect()
}

/// Map path conventions and file classification into logical architectural layer names.
fn classify_layer(norm_path: &str, classification: FileClassification) -> (String, String) {
    let lower = norm_path.to_ascii_lowercase();

    if classification == FileClassification::Database
        || lower.contains("/models")
        || lower.contains("/db/")
        || lower.contains("/migrations")
        || lower.ends_with(".sql")
        || lower.ends_with(".prisma")
    {
        return (
            "Persistence & Data Layer".to_string(),
            "Database schemas, migrations, and ORM entities".to_string(),
        );
    }

    if lower.contains("/routes")
        || lower.contains("/api/")
        || lower.contains("/controllers")
        || lower.contains("/handlers")
        || lower.contains("server.")
    {
        return (
            "API & Transport Layer".to_string(),
            "HTTP route handlers, controllers, and server entrypoints".to_string(),
        );
    }

    if lower.starts_with("frontend/")
        || lower.contains("/components/")
        || lower.contains("/pages/")
        || lower.ends_with(".tsx")
        || lower.ends_with(".jsx")
        || lower.ends_with(".css")
    {
        return (
            "Presentation / UI Layer".to_string(),
            "Client UI views, components, and browser assets".to_string(),
        );
    }

    match classification {
        FileClassification::Tests => (
            "Verification & Test Suite".to_string(),
            "Automated unit, integration, and verification tests".to_string(),
        ),
        FileClassification::Infrastructure | FileClassification::Deployment => (
            "Infrastructure & CI/CD".to_string(),
            "Containers, orchestration workflows, and deployment scripts".to_string(),
        ),
        FileClassification::Configuration | FileClassification::BuildSystem => (
            "Configuration & Build Manifests".to_string(),
            "Dependency manifests, build configs, and runtime settings".to_string(),
        ),
        FileClassification::Documentation => (
            "Documentation".to_string(),
            "Architecture specifications, READMEs, and guides".to_string(),
        ),
        _ => (
            "Core Application & Domain Logic".to_string(),
            "Primary business logic, services, parsers, and utilities".to_string(),
        ),
    }
}

/// Identify top architectural hotspots ranked by fan-in centrality and blast radius.
fn compute_hotspots(graph: &crate::graph::ArchitectureGraph) -> Vec<ComponentHotspot> {
    graph
        .rank_by_fan_in(10)
        .into_iter()
        .filter(|(_, fan_in)| *fan_in > 0)
        .map(|(node, fan_in)| {
            let fan_out = graph.fan_out(&node.id);
            let blast_radius_count = graph
                .blast_radius(&node.id, Some(4))
                .map(|b| b.total_impacted_nodes)
                .unwrap_or(0);

            ComponentHotspot {
                node_id: node.id,
                name: node.name,
                kind: node.kind.to_string(),
                file_path: node
                    .file_path
                    .map(|p| p.to_string_lossy().replace('\\', "/")),
                fan_in,
                fan_out,
                blast_radius_count,
            }
        })
        .collect()
}

/// Separate verified static facts from inferred architectural conclusions.
fn build_fact_vs_inference(
    metrics: &ArchitectureMetrics,
    pattern: &ArchitecturalPatternSummary,
    layers: &[ArchitecturalLayer],
    unresolved_imports: &[UnresolvedImport],
    graph_export: &GraphExport,
) -> FactVsInference {
    let mut detected_facts = vec![
        format!(
            "Deterministically parsed {} source files into Tree-sitter concrete syntax trees ({} with syntax error recovery)",
            metrics.files_parsed_ast, metrics.files_with_syntax_errors
        ),
        format!(
            "Extracted {} unique AST symbols with deterministic BLAKE3 identifiers",
            metrics.total_symbols
        ),
        format!(
            "Observed {} import declarations ({} internal, {} external across {} unique packages)",
            metrics.total_imports,
            metrics.internal_imports,
            metrics.external_imports,
            metrics.unique_external_packages
        ),
        format!(
            "Constructed directed graph containing {} vertices and {} edges (DAG: {})",
            metrics.graph_nodes, metrics.graph_edges, metrics.graph_is_dag
        ),
    ];

    if metrics.api_route_count > 0 {
        detected_facts.push(format!(
            "Detected {} explicit API route declarations across source/framework conventions",
            metrics.api_route_count
        ));
    }

    if metrics.data_model_count > 0 {
        detected_facts.push(format!(
            "Detected {} explicit ORM entities, SQL migrations, or schema declarations",
            metrics.data_model_count
        ));
    }

    let calls_edges = graph_export
        .summary
        .edge_counts_by_kind
        .get("calls")
        .copied()
        .unwrap_or(0);

    let inferred_relationships = vec![
        format!(
            "Inferred primary architectural style '{}' (Confidence: {})",
            pattern.primary_pattern, pattern.confidence
        ),
        format!(
            "Classified repository contents into {} logical architectural layers via path and manifest heuristics",
            layers.len()
        ),
        format!(
            "Heuristically inferred {} function/method call edge(s) from identifier invocation syntax",
            calls_edges
        ),
    ];

    let unresolved_sample: Vec<String> = unresolved_imports
        .iter()
        .take(15)
        .map(|u| format!("{}:{} -> '{}'", u.source_file, u.line, u.raw_import_path))
        .collect();

    FactVsInference {
        detected_facts,
        inferred_relationships,
        unresolved_references_count: unresolved_imports.len(),
        unresolved_references_sample: unresolved_sample,
    }
}
