//! Detection of HTTP API routes across web frameworks.

use std::path::Path;
use tree_sitter::Node;

use super::types::{ApiRoute, HttpMethod};
use crate::parser::{ParsedSource, SourceSpan, SupportedLanguage};

/// Detector responsible for locating declared HTTP routes and endpoints.
#[derive(Debug, Default, Clone)]
pub struct ApiRouteDetector;

impl ApiRouteDetector {
    pub fn new() -> Self {
        Self
    }

    /// Extract all declared API routes from a parsed source file.
    pub fn detect_routes(&self, parsed: &ParsedSource) -> Vec<ApiRoute> {
        let mut routes = Vec::new();
        let file_path = parsed.file_path().unwrap_or(Path::new("unknown"));

        // 1. Next.js file-based API route detection
        if let Some(next_route) = self.detect_nextjs_file_route(parsed, file_path) {
            routes.extend(next_route);
        }

        // 2. Syntax-based route detection per language
        match parsed.language {
            SupportedLanguage::JavaScript
            | SupportedLanguage::TypeScript
            | SupportedLanguage::Tsx => {
                self.detect_js_routes(parsed, file_path, &mut routes);
            }
            SupportedLanguage::Python => {
                self.detect_python_routes(parsed, file_path, &mut routes);
            }
            SupportedLanguage::Go => {
                self.detect_go_routes(parsed, file_path, &mut routes);
            }
            SupportedLanguage::Rust => {
                self.detect_rust_routes(parsed, file_path, &mut routes);
            }
            SupportedLanguage::Java => {
                self.detect_java_routes(parsed, file_path, &mut routes);
            }
            _ => {}
        }

        routes
    }

    /// Detect Express / Fastify / Koa routes in JS/TS.
    fn detect_js_routes(
        &self,
        parsed: &ParsedSource,
        file_path: &Path,
        routes: &mut Vec<ApiRoute>,
    ) {
        let calls = parsed.find_nodes_by_kind("call_expression");

        for call in calls {
            if let Some(fn_node) = call.child_by_field_name("function") {
                if fn_node.kind() == "member_expression" {
                    if let Some(prop) = fn_node.child_by_field_name("property") {
                        let method_name = parsed.node_text(&prop).trim();
                        if let Some(http_method) = HttpMethod::from_name(method_name) {
                            if let Some(args) = call.child_by_field_name("arguments") {
                                if let Some(first_arg) = args.child(1) {
                                    if first_arg.kind() == "string" {
                                        let path_pattern =
                                            clean_string_literal(parsed.node_text(&first_arg));
                                        if path_pattern.starts_with('/') {
                                            let span = parsed.node_span(&call);
                                            let named_count = args.named_child_count();
                                            let handler_name = if named_count >= 2 {
                                                args.named_child(named_count - 1).map(|h| {
                                                    parsed.node_text(&h).trim().to_string()
                                                })
                                            } else {
                                                None
                                            };

                                            routes.push(ApiRoute {
                                                method: http_method,
                                                path_pattern,
                                                handler_name,
                                                file_path: file_path.to_path_buf(),
                                                span,
                                                framework: "Express/Fastify".to_string(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Detect Next.js file-based API routes.
    fn detect_nextjs_file_route(
        &self,
        parsed: &ParsedSource,
        file_path: &Path,
    ) -> Option<Vec<ApiRoute>> {
        let normalized = file_path.to_string_lossy().replace('\\', "/");

        // Next.js Pages router: pages/api/...
        if let Some(api_idx) = normalized.find("pages/api/") {
            let sub = &normalized[api_idx + "pages/api/".len()..];
            let clean_sub = sub
                .trim_end_matches(".ts")
                .trim_end_matches(".tsx")
                .trim_end_matches(".js")
                .trim_end_matches(".jsx")
                .trim_end_matches("/index");

            // Convert `[id]` to `:id` parameter format
            let route_path = format!("/api/{}", clean_sub.replace('[', ":").replace(']', ""));
            let span = SourceSpan::new(1, 1, 1, 1, 0, 0);

            return Some(vec![ApiRoute {
                method: HttpMethod::All,
                path_pattern: route_path,
                handler_name: Some("default".to_string()),
                file_path: file_path.to_path_buf(),
                span,
                framework: "Next.js (Pages API)".to_string(),
            }]);
        }

        // Next.js App router: app/api/.../route.ts
        if let Some(api_idx) = normalized.find("app/api/") {
            if normalized.ends_with("/route.ts") || normalized.ends_with("/route.js") {
                let sub = &normalized[api_idx + "app/api/".len()..];
                let dir_part = sub
                    .trim_end_matches("/route.ts")
                    .trim_end_matches("/route.js");
                let route_path = format!("/api/{}", dir_part.replace('[', ":").replace(']', ""));

                let mut routes = Vec::new();
                for method in ["GET", "POST", "PUT", "DELETE", "PATCH"] {
                    if parsed
                        .source
                        .contains(&format!("export async function {}", method))
                        || parsed
                            .source
                            .contains(&format!("export function {}", method))
                    {
                        routes.push(ApiRoute {
                            method: HttpMethod::from_name(method).unwrap(),
                            path_pattern: route_path.clone(),
                            handler_name: Some(method.to_string()),
                            file_path: file_path.to_path_buf(),
                            span: SourceSpan::new(1, 1, 1, 1, 0, 0),
                            framework: "Next.js (App Router)".to_string(),
                        });
                    }
                }
                if !routes.is_empty() {
                    return Some(routes);
                }
            }
        }

        None
    }

    /// Detect FastAPI and Flask routes in Python.
    fn detect_python_routes(
        &self,
        parsed: &ParsedSource,
        file_path: &Path,
        routes: &mut Vec<ApiRoute>,
    ) {
        let decorators = parsed.find_nodes_by_kind("decorator");

        for dec in decorators {
            let dec_text = parsed.node_text(&dec).trim();

            // FastAPI / Flask style: @app.get("/path"), @router.post("/path"), @app.route("/path")
            for method in ["get", "post", "put", "delete", "patch"] {
                let pattern = format!(".{}(", method);
                if dec_text.contains(&pattern) {
                    if let Some(path_literal) = extract_first_string_arg(&dec, parsed) {
                        let span = parsed.node_span(&dec);
                        let handler = find_python_decorated_handler_name(&dec, parsed);

                        routes.push(ApiRoute {
                            method: HttpMethod::from_name(method).unwrap(),
                            path_pattern: path_literal,
                            handler_name: handler,
                            file_path: file_path.to_path_buf(),
                            span,
                            framework: "FastAPI/Flask".to_string(),
                        });
                        break;
                    }
                }
            }

            if dec_text.contains(".route(") {
                if let Some(path_literal) = extract_first_string_arg(&dec, parsed) {
                    let span = parsed.node_span(&dec);
                    let handler = find_python_decorated_handler_name(&dec, parsed);

                    let method = if dec_text.contains("POST") {
                        HttpMethod::Post
                    } else {
                        HttpMethod::Get
                    };

                    routes.push(ApiRoute {
                        method,
                        path_pattern: path_literal,
                        handler_name: handler,
                        file_path: file_path.to_path_buf(),
                        span,
                        framework: "Flask".to_string(),
                    });
                }
            }
        }
    }

    /// Detect Gin and Echo routes in Go.
    fn detect_go_routes(
        &self,
        parsed: &ParsedSource,
        file_path: &Path,
        routes: &mut Vec<ApiRoute>,
    ) {
        let calls = parsed.find_nodes_by_kind("call_expression");

        for call in calls {
            if let Some(fn_node) = call.child_by_field_name("function") {
                if fn_node.kind() == "selector_expression" {
                    if let Some(field) = fn_node.child_by_field_name("field") {
                        let method_name = parsed.node_text(&field).trim();
                        if let Some(http_method) = HttpMethod::from_name(method_name) {
                            if let Some(args) = call.child_by_field_name("arguments") {
                                if let Some(first_arg) = args.child(1) {
                                    if first_arg.kind() == "interpreted_string_literal" {
                                        let path_pattern =
                                            clean_string_literal(parsed.node_text(&first_arg));
                                        if path_pattern.starts_with('/') {
                                            let span = parsed.node_span(&call);
                                            let handler = args
                                                .child(3)
                                                .map(|h| parsed.node_text(&h).trim().to_string());

                                            routes.push(ApiRoute {
                                                method: http_method,
                                                path_pattern,
                                                handler_name: handler,
                                                file_path: file_path.to_path_buf(),
                                                span,
                                                framework: "Gin/Echo".to_string(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Detect Actix-web and Axum routes in Rust.
    fn detect_rust_routes(
        &self,
        parsed: &ParsedSource,
        file_path: &Path,
        routes: &mut Vec<ApiRoute>,
    ) {
        // Look for `.route("/path", get(handler))` or `#[get("/path")]`
        let calls = parsed.find_nodes_by_kind("call_expression");

        for call in calls {
            if let Some(fn_node) = call.child_by_field_name("function") {
                if fn_node.kind() == "field_expression" {
                    if let Some(field) = fn_node.child_by_field_name("field") {
                        if parsed.node_text(&field).trim() == "route" {
                            if let Some(args) = call.child_by_field_name("arguments") {
                                if let Some(first_arg) = args.child(1) {
                                    if first_arg.kind() == "string_literal" {
                                        let path_pattern =
                                            clean_string_literal(parsed.node_text(&first_arg));
                                        let span = parsed.node_span(&call);

                                        let method = if parsed.node_text(&args).contains("post") {
                                            HttpMethod::Post
                                        } else if parsed.node_text(&args).contains("delete") {
                                            HttpMethod::Delete
                                        } else {
                                            HttpMethod::Get
                                        };

                                        routes.push(ApiRoute {
                                            method,
                                            path_pattern,
                                            handler_name: None,
                                            file_path: file_path.to_path_buf(),
                                            span,
                                            framework: "Actix/Axum".to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Detect Spring Boot annotations in Java.
    fn detect_java_routes(
        &self,
        parsed: &ParsedSource,
        file_path: &Path,
        routes: &mut Vec<ApiRoute>,
    ) {
        let annotations = parsed.find_nodes_by_kind("annotation");

        for ann in annotations {
            let ann_text = parsed.node_text(&ann).trim();
            for (kw, method) in [
                ("GetMapping", HttpMethod::Get),
                ("PostMapping", HttpMethod::Post),
                ("PutMapping", HttpMethod::Put),
                ("DeleteMapping", HttpMethod::Delete),
                ("PatchMapping", HttpMethod::Patch),
                ("RequestMapping", HttpMethod::All),
            ] {
                if ann_text.starts_with(&format!("@{}", kw)) {
                    if let Some(path_literal) = extract_first_string_arg(&ann, parsed) {
                        let span = parsed.node_span(&ann);
                        let method_decl = if ann
                            .parent()
                            .map(|p| p.kind() == "modifiers")
                            .unwrap_or(false)
                        {
                            ann.parent().and_then(|p| p.parent())
                        } else {
                            ann.parent()
                        };
                        let handler = method_decl
                            .and_then(|m| m.child_by_field_name("name"))
                            .map(|n| parsed.node_text(&n).trim().to_string());

                        routes.push(ApiRoute {
                            method,
                            path_pattern: path_literal,
                            handler_name: handler,
                            file_path: file_path.to_path_buf(),
                            span,
                            framework: "Spring Boot".to_string(),
                        });
                        break;
                    }
                }
            }
        }
    }
}

fn find_python_decorated_handler_name(dec: &Node, parsed: &ParsedSource) -> Option<String> {
    let parent = dec.parent()?;
    // If parent is decorated_definition, look for the function_definition inside it
    if parent.kind() == "decorated_definition" {
        if let Some(def_node) = parent.child_by_field_name("definition") {
            if let Some(name_node) = def_node.child_by_field_name("name") {
                return Some(parsed.node_text(&name_node).trim().to_string());
            }
        }
        let mut cursor = parent.walk();
        for child in parent.children(&mut cursor) {
            if child.kind() == "function_definition" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    return Some(parsed.node_text(&name_node).trim().to_string());
                }
            }
        }
    }
    None
}

fn extract_first_string_arg(node: &Node, parsed: &ParsedSource) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "string"
            || child.kind() == "string_literal"
            || child.kind() == "interpreted_string_literal"
        {
            let text = clean_string_literal(parsed.node_text(&child));
            if text.starts_with('/') {
                return Some(text);
            }
        } else if let Some(nested) = extract_first_string_arg(&child, parsed) {
            return Some(nested);
        }
    }
    None
}

fn clean_string_literal(raw: &str) -> String {
    raw.trim().trim_matches(['\'', '"', '`']).to_string()
}
