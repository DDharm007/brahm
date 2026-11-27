//! Stage 02 Markdown report generator (`BRAHM_ARCHITECTURE.md`).
//!
//! Produces a comprehensive, developer-oriented technical architectural blueprint
//! strictly separating deterministic static facts from inferred architectural relationships.

use std::fs;
use std::path::Path;

use crate::architecture::ArchitectureReport;
use crate::errors::Result;

/// Generates `BRAHM_ARCHITECTURE.md` and writes to disk if `output_path` is provided.
pub fn generate_architecture_markdown(
    report: &ArchitectureReport,
    output_path: Option<&Path>,
) -> Result<String> {
    let mut md = String::new();

    md.push_str("# BRAHM Architectural Blueprint & Code Intelligence Report\n\n");
    md.push_str("> Generated deterministically by **BRAHM Stage 02 (Code Intelligence & Architecture Mapping Engine)**\n\n");

    // 1. Executive Architecture Overview
    md.push_str("## 1. Executive Architecture Overview\n\n");
    md.push_str("| Property | Value |\n");
    md.push_str("|---|---|\n");
    md.push_str(&format!(
        "| **Repository** | `{}` |\n",
        report.repository.name
    ));
    md.push_str(&format!(
        "| **Inferred Pattern** | **{}** (Confidence: `{}`) |\n",
        report.pattern.primary_pattern, report.pattern.confidence
    ));
    md.push_str(&format!(
        "| **Analysis Duration** | `{} ms` |\n",
        report.analysis_duration_ms
    ));
    md.push_str(&format!(
        "| **AST Files Parsed** | `{}` / `{}` total files |\n",
        report.metrics.files_parsed_ast, report.metrics.total_files_scanned
    ));
    md.push_str(&format!(
        "| **Extracted Symbols** | `{}` |\n",
        report.metrics.total_symbols
    ));
    md.push_str(&format!(
        "| **API Routes Detected** | `{}` |\n",
        report.metrics.api_route_count
    ));
    md.push_str(&format!(
        "| **Data Models & Schemas** | `{}` |\n",
        report.metrics.data_model_count
    ));
    md.push_str(&format!(
        "| **Graph Topology** | `{}` nodes, `{}` edges (DAG: `{}`) |\n\n",
        report.metrics.graph_nodes, report.metrics.graph_edges, report.metrics.graph_is_dag
    ));

    if !report.pattern.rationale.is_empty() {
        md.push_str("**Architectural Rationale:**\n");
        for reason in &report.pattern.rationale {
            md.push_str(&format!("- {}\n", reason));
        }
        md.push('\n');
    }

    // 2. Detected Facts vs Inferred Architecture vs Unresolved References
    md.push_str("## 2. Verification & Epistemological Breakdown\n\n");
    md.push_str("### Verified Static Facts (Deterministic AST & Manifests)\n");
    for fact in &report.confidence_breakdown.detected_facts {
        md.push_str(&format!("- ✅ {}\n", fact));
    }
    md.push('\n');

    md.push_str("### Inferred Relationships & Heuristics\n");
    for inf in &report.confidence_breakdown.inferred_relationships {
        md.push_str(&format!("- 🔍 {}\n", inf));
    }
    md.push('\n');

    md.push_str(&format!(
        "### Unresolved References ({} total)\n",
        report.confidence_breakdown.unresolved_references_count
    ));
    if report
        .confidence_breakdown
        .unresolved_references_sample
        .is_empty()
    {
        md.push_str("_All relative internal imports resolved successfully._\n\n");
    } else {
        for unres in &report.confidence_breakdown.unresolved_references_sample {
            md.push_str(&format!("- ⚠️ `{}`\n", unres));
        }
        md.push('\n');
    }

    // 3. Application Landmarks & Entry Points
    md.push_str("## 3. Application Entry Points\n\n");
    if report.entry_points.is_empty() {
        md.push_str("_No explicit application entry points identified._\n\n");
    } else {
        md.push_str("| Kind | File Path | Symbol | Confidence | Description |\n");
        md.push_str("|---|---|---|---|---|\n");
        for ep in &report.entry_points {
            let file = ep.file_path.to_string_lossy().replace('\\', "/");
            let sym = ep.symbol_name.as_deref().unwrap_or("-");
            md.push_str(&format!(
                "| `{}` | `{}` | `{}` | `{:.0}%` | {} |\n",
                ep.kind.as_str(),
                file,
                sym,
                ep.confidence * 100.0,
                ep.description
            ));
        }
        md.push('\n');
    }

    // 4. API Surface & Route Catalog
    md.push_str("## 4. API Surface & Endpoint Catalog\n\n");
    if report.api_routes.is_empty() {
        md.push_str("_No HTTP API endpoints detected._\n\n");
    } else {
        md.push_str("| Method | Path Pattern | Handler | Framework | Source Location |\n");
        md.push_str("|---|---|---|---|---|\n");
        for route in &report.api_routes {
            let file = route.file_path.to_string_lossy().replace('\\', "/");
            let handler = route.handler_name.as_deref().unwrap_or("_anonymous_");
            md.push_str(&format!(
                "| `{}` | `{}` | `{}` | {} | `{}:{}` |\n",
                route.method,
                route.path_pattern,
                handler,
                route.framework,
                file,
                route.span.start_line
            ));
        }
        md.push('\n');
    }

    // 5. Data Architecture & Persistence Models
    md.push_str("## 5. Data Models & Persistence Layer\n\n");
    if report.data_models.is_empty() {
        md.push_str("_No ORM entities, SQL migrations, or database schemas detected._\n\n");
    } else {
        md.push_str("| Entity / Artifact | ORM / Technology | Backing Table | File Path |\n");
        md.push_str("|---|---|---|---|\n");
        for model in &report.data_models {
            let file = model.file_path.to_string_lossy().replace('\\', "/");
            let table = model.table_name.as_deref().unwrap_or("-");
            md.push_str(&format!(
                "| **{}** | {} | `{}` | `{}` |\n",
                model.name, model.framework_or_orm, table, file
            ));
        }
        md.push('\n');
    }

    // 6. Logical Architectural Layers
    md.push_str("## 6. Architectural Layers & Component Distribution\n\n");
    if report.layers.is_empty() {
        md.push_str("_No layers identified._\n\n");
    } else {
        md.push_str("| Architectural Layer | Role | Files | Symbols | Routes |\n");
        md.push_str("|---|---|---|---|---|\n");
        for layer in &report.layers {
            md.push_str(&format!(
                "| **{}** | {} | `{}` | `{}` | `{}` |\n",
                layer.name, layer.role, layer.file_count, layer.symbol_count, layer.route_count
            ));
        }
        md.push('\n');
    }

    // 7. Dependency Topology & Circular Dependencies
    md.push_str("## 7. Dependency Topology & Cycles\n\n");
    md.push_str(&format!(
        "- **Total Imports:** `{}` (`{}` internal, `{}` external)\n",
        report.metrics.total_imports,
        report.metrics.internal_imports,
        report.metrics.external_imports
    ));
    md.push_str(&format!(
        "- **Circular Dependency Cycles:** `{}`\n\n",
        report.metrics.circular_dependency_count
    ));

    if !report.dependencies.circular_dependencies.is_empty() {
        md.push_str("### Detected Dependency Cycles\n");
        for (idx, dep_cycle) in report.dependencies.circular_dependencies.iter().enumerate() {
            let path_chain: Vec<String> = dep_cycle
                .cycle
                .iter()
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .collect();
            md.push_str(&format!("{}. `{}`\n", idx + 1, path_chain.join(" -> ")));
        }
        md.push('\n');
    }

    // 8. Blast Radius & Centrality Hotspots
    md.push_str("## 8. Architectural Hotspots (Centrality & Blast Radius)\n\n");
    if report.hotspots.is_empty() {
        md.push_str("_No high-coupling hotspots identified._\n\n");
    } else {
        md.push_str("| Component | Kind | Fan-In | Fan-Out | Blast Radius |\n");
        md.push_str("|---|---|---|---|---|\n");
        for hs in &report.hotspots {
            md.push_str(&format!(
                "| `{}` | `{}` | `{}` | `{}` | `{}` node(s) |\n",
                hs.name, hs.kind, hs.fan_in, hs.fan_out, hs.blast_radius_count
            ));
        }
        md.push('\n');
    }

    // 9. External Dependencies
    md.push_str("## 9. External Ecosystem Packages\n\n");
    if report.dependencies.external_packages.is_empty() {
        md.push_str("_No external packages detected in analyzed AST imports._\n\n");
    } else {
        let pkg_list: Vec<String> = report
            .dependencies
            .external_packages
            .iter()
            .map(|p| format!("`{}`", p))
            .collect();
        md.push_str(&format!("{}\n\n", pkg_list.join(", ")));
    }

    // 10. Diagnostics & Warnings
    if !report.warnings.is_empty() {
        md.push_str("## 10. Parser Diagnostics & Warnings\n\n");
        for w in &report.warnings {
            md.push_str(&format!("- {}\n", w));
        }
        md.push('\n');
    }

    if let Some(path) = output_path {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, &md)?;
    }

    Ok(md)
}
