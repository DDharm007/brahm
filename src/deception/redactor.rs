//! Deterministic AST-driven Source & Schema Redactor for `BlindShield` Mirrors (Stage 04).
//!
//! Operates strictly inside isolated inspection workspace copies and NEVER mutates
//! genuine production repository source files. Uses Stage 02's Tree-sitter parsers
//! deterministically with zero AI/LLM runtime dependencies.

use std::path::Path;

use crate::parser::{ParserRegistry, SupportedLanguage};

/// Redact proprietary implementation logic inside an isolated inspection workspace file.
///
/// Returns `(output_bytes, was_redacted)`.
pub fn redact_mirror_file(
    rel_path: &str,
    content_bytes: &[u8],
    registry: &ParserRegistry,
) -> (Vec<u8>, bool) {
    let Ok(text) = std::str::from_utf8(content_bytes) else {
        // Skip binary blobs cleanly without mutation
        return (content_bytes.to_vec(), false);
    };

    let norm = rel_path.replace('\\', "/");
    let lower = norm.to_ascii_lowercase();

    // 1. SQL schemas and migrations
    if lower.ends_with(".sql") {
        let stub = "-- [BRAHM_SHIELD] SQL schema DDL and database queries redacted for isolated inspection.\n\
                    -- Original production database schemas remain unmodified in the owner's repository.\n";
        return (stub.as_bytes().to_vec(), true);
    }

    // 2. Detailed internal documentation exposing API routes / DB tables / architecture specs
    let filename = Path::new(&lower)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if matches!(
        filename,
        "api_documentation.md" | "architecture.md" | "database_schema.md"
    ) {
        let stub = "# [BRAHM_SHIELD] Internal Architecture Specification Protected\n\n\
                    Detailed endpoint mappings, database table definitions, and internal module diagrams\n\
                    are redacted in this isolated anti-reverse-engineering inspection workspace.\n";
        return (stub.as_bytes().to_vec(), true);
    }

    // 3. Example / template env files: redact default assignment values
    if filename.starts_with(".env") {
        let mut masked = String::new();
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                masked.push_str(line);
                masked.push('\n');
            } else if let Some((k, _)) = line.split_once('=') {
                masked.push_str(k);
                masked.push_str("=<BRAHM_SHIELD_REDACTED>\n");
            } else {
                masked.push_str(line);
                masked.push('\n');
            }
        }
        return (masked.into_bytes(), true);
    }

    // 4. Supported source languages via Tree-sitter AST implementation body redaction
    let Some(lang) = Path::new(&norm)
        .extension()
        .and_then(|s| s.to_str())
        .and_then(SupportedLanguage::from_extension)
    else {
        return (content_bytes.to_vec(), false);
    };

    let Ok(parsed) = registry.parse_source(text, lang) else {
        return (content_bytes.to_vec(), false);
    };

    let mut spans = Vec::new();
    collect_top_function_bodies(parsed.tree.root_node(), lang, &mut spans);

    if spans.is_empty() {
        return (content_bytes.to_vec(), false);
    }

    // Sort spans non-overlapping descending by byte start and splice deterministically
    spans.sort_by_key(|a| std::cmp::Reverse(a.0));
    let mut out = text.to_string();
    let mut last_start = usize::MAX;
    for (start, end, stub) in spans {
        if end <= out.len() && end <= last_start && start < end {
            out.replace_range(start..end, &stub);
            last_start = start;
        }
    }

    (out.into_bytes(), true)
}

fn collect_top_function_bodies(
    node: tree_sitter::Node<'_>,
    lang: SupportedLanguage,
    out: &mut Vec<(usize, usize, String)>,
) {
    let kind = node.kind();
    let is_target_fn = match lang {
        SupportedLanguage::JavaScript | SupportedLanguage::TypeScript | SupportedLanguage::Tsx => {
            matches!(
                kind,
                "function_declaration"
                    | "method_definition"
                    | "function_expression"
                    | "arrow_function"
            )
        }
        SupportedLanguage::Python => kind == "function_definition",
        SupportedLanguage::Rust => kind == "function_item",
        SupportedLanguage::Go => matches!(kind, "function_declaration" | "method_declaration"),
        SupportedLanguage::Java => matches!(kind, "method_declaration" | "constructor_declaration"),
        SupportedLanguage::C | SupportedLanguage::Cpp => kind == "function_definition",
    };

    if is_target_fn {
        if let Some(body) = node.child_by_field_name("body") {
            let stub = stub_for_language(lang);
            out.push((body.start_byte(), body.end_byte(), stub));
            // Do not descend into children of an already-redacted function body
            return;
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_top_function_bodies(child, lang, out);
    }
}

fn stub_for_language(lang: SupportedLanguage) -> String {
    match lang {
        SupportedLanguage::JavaScript
        | SupportedLanguage::TypeScript
        | SupportedLanguage::Tsx => {
            "{ /* [BRAHM_SHIELD] Implementation redacted for anti-RE protection. */ throw new Error('BRAHM_SEALED'); }"
                .to_string()
        }
        SupportedLanguage::Python => {
            "pass  # [BRAHM_SHIELD] Implementation redacted.".to_string()
        }
        SupportedLanguage::Rust => {
            "{ /* [BRAHM_SHIELD] Implementation redacted. */ unimplemented!(\"BRAHM_SEALED\") }"
                .to_string()
        }
        SupportedLanguage::Go => {
            "{ /* [BRAHM_SHIELD] Implementation redacted. */ panic(\"BRAHM_SEALED\") }"
                .to_string()
        }
        SupportedLanguage::Java => {
            "{ /* [BRAHM_SHIELD] Implementation redacted. */ throw new UnsupportedOperationException(\"BRAHM_SEALED\"); }"
                .to_string()
        }
        SupportedLanguage::C | SupportedLanguage::Cpp => {
            "{ /* [BRAHM_SHIELD] Implementation redacted. */ }".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_typescript_service_and_arrow_routes() {
        let reg = ParserRegistry::new();
        let src = r#"
export interface OrderSummary { id: string; totalCents: number; }

export function calculateOrderTotals(subtotal: number): number {
  const tax = Math.round((subtotal * 825) / 10000);
  const shipping = subtotal >= 10000 ? 0 : 995;
  return subtotal + tax + shipping;
}

export class PaymentService {
  public verifyWebhook(body: string, sig: string): boolean {
    return body.length > 0 && sig === "secret_algo";
  }
}

export const quickCheck = (x: number) => x * 42;
"#;
        let (bytes, changed) =
            redact_mirror_file("src/services/orderService.ts", src.as_bytes(), &reg);
        assert!(changed);
        let out = String::from_utf8(bytes).unwrap();
        assert!(out.contains("export interface OrderSummary"));
        assert!(out.contains("BRAHM_SEALED"));
        assert!(!out.contains("825"));
        assert!(!out.contains("secret_algo"));
        assert!(!out.contains("42"));
    }

    #[test]
    fn test_redact_sql_and_internal_docs() {
        let reg = ParserRegistry::new();
        let (sql_bytes, sql_changed) = redact_mirror_file(
            "db/schema.sql",
            b"CREATE TABLE users (id TEXT PRIMARY KEY, password_hash TEXT NOT NULL);",
            &reg,
        );
        assert!(sql_changed);
        let sql_out = String::from_utf8(sql_bytes).unwrap();
        assert!(sql_out.contains("BRAHM_SHIELD"));
        assert!(!sql_out.contains("password_hash"));

        let (doc_bytes, doc_changed) =
            redact_mirror_file("API_DOCUMENTATION.md", b"# API\nPOST /api/orders", &reg);
        assert!(doc_changed);
        assert!(!String::from_utf8(doc_bytes)
            .unwrap()
            .contains("/api/orders"));
    }
}
