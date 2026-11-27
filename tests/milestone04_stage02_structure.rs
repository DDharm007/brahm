//! Stage 02 Milestone 04 Verification Test Suite
//! Application Structure, Landmark, API Route & Data Model Detection

use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

use brahm::parser::{ParserRegistry, SupportedLanguage};
use brahm::structure::{
    ApiRouteDetector, DataModelDetector, DataModelKind, EntryPointDetector, EntryPointKind,
    HttpMethod, StructureAnalyzer,
};
use brahm::symbols::extract_and_index;

#[test]
fn test_01_language_entry_points_detection() {
    let registry = ParserRegistry::new();
    let detector = EntryPointDetector::new();

    // 1. Rust main.rs
    let rust_code = "fn main() { println!(\"Hello\"); }";
    let parsed_rs = registry
        .parse_source(rust_code, SupportedLanguage::Rust)
        .unwrap();
    let symbols_rs = extract_and_index(&parsed_rs);
    let entries_rs = detector.detect_file_entrypoints(&parsed_rs, &symbols_rs);
    assert_eq!(entries_rs.len(), 1);
    assert_eq!(entries_rs[0].kind, EntryPointKind::Executable);
    assert_eq!(entries_rs[0].symbol_name.as_deref(), Some("main"));

    // 2. Python __main__ and manage.py
    let dir = tempdir().unwrap();
    let manage_py = dir.path().join("manage.py");
    fs::write(&manage_py, "if __name__ == '__main__':\n    pass\n").unwrap();
    let parsed_py = registry.parse_file(&manage_py, Some("python")).unwrap();
    let symbols_py = extract_and_index(&parsed_py);
    let entries_py = detector.detect_file_entrypoints(&parsed_py, &symbols_py);
    assert!(entries_py
        .iter()
        .any(|e| e.kind == EntryPointKind::CliEntry));
    assert!(entries_py
        .iter()
        .any(|e| e.kind == EntryPointKind::Executable));

    // 3. Node server.ts
    let server_ts = dir.path().join("server.ts");
    fs::write(
        &server_ts,
        "import express from 'express';\nconst app = express();\n",
    )
    .unwrap();
    let parsed_ts = registry.parse_file(&server_ts, Some("typescript")).unwrap();
    let symbols_ts = extract_and_index(&parsed_ts);
    let entries_ts = detector.detect_file_entrypoints(&parsed_ts, &symbols_ts);
    assert_eq!(entries_ts.len(), 1);
    assert_eq!(entries_ts[0].kind, EntryPointKind::ServerEntry);
}

#[test]
fn test_02_express_route_detection() {
    let registry = ParserRegistry::new();
    let detector = ApiRouteDetector::new();

    let express_code = r#"
const express = require('express');
const app = express();
const router = express.Router();

app.get('/api/v1/health', (req, res) => res.json({ status: 'ok' }));
router.post('/api/v1/auth/login', handleLogin);
app.delete('/api/v1/users/:id', deleteUserHandler);
"#;

    let parsed = registry
        .parse_source(express_code, SupportedLanguage::JavaScript)
        .unwrap();
    let routes = detector.detect_routes(&parsed);

    assert_eq!(routes.len(), 3);

    let get_route = routes.iter().find(|r| r.method == HttpMethod::Get).unwrap();
    assert_eq!(get_route.path_pattern, "/api/v1/health");
    assert_eq!(get_route.framework, "Express/Fastify");

    let post_route = routes
        .iter()
        .find(|r| r.method == HttpMethod::Post)
        .unwrap();
    assert_eq!(post_route.path_pattern, "/api/v1/auth/login");
    assert_eq!(post_route.handler_name.as_deref(), Some("handleLogin"));

    let delete_route = routes
        .iter()
        .find(|r| r.method == HttpMethod::Delete)
        .unwrap();
    assert_eq!(delete_route.path_pattern, "/api/v1/users/:id");
}

#[test]
fn test_03_nextjs_route_detection() {
    let registry = ParserRegistry::new();
    let detector = ApiRouteDetector::new();

    // 1. Pages router: pages/api/auth/login.ts
    let pages_path = PathBuf::from("frontend/src/pages/api/auth/login.ts");
    let mut parsed_pages = registry
        .parse_source(
            "export default function handler(req, res) {}",
            SupportedLanguage::TypeScript,
        )
        .unwrap();
    parsed_pages.path = Some(pages_path);
    let pages_routes = detector.detect_routes(&parsed_pages);
    assert_eq!(pages_routes.len(), 1);
    assert_eq!(pages_routes[0].path_pattern, "/api/auth/login");
    assert_eq!(pages_routes[0].method, HttpMethod::All);
    assert_eq!(pages_routes[0].framework, "Next.js (Pages API)");

    // 2. App router: app/api/users/[id]/route.ts
    let app_path = PathBuf::from("app/api/users/[id]/route.ts");
    let app_source = r#"
export async function GET(request: Request) { return Response.json({}); }
export async function POST(request: Request) { return Response.json({}); }
"#;
    let mut parsed_app = registry
        .parse_source(app_source, SupportedLanguage::TypeScript)
        .unwrap();
    parsed_app.path = Some(app_path);
    let app_routes = detector.detect_routes(&parsed_app);
    assert_eq!(app_routes.len(), 2);
    assert_eq!(app_routes[0].path_pattern, "/api/users/:id");
    assert!(app_routes.iter().any(|r| r.method == HttpMethod::Get));
    assert!(app_routes.iter().any(|r| r.method == HttpMethod::Post));
}

#[test]
fn test_04_python_fastapi_and_flask_routes() {
    let registry = ParserRegistry::new();
    let detector = ApiRouteDetector::new();

    let py_code = r#"
from fastapi import FastAPI, APIRouter

app = FastAPI()
router = APIRouter()

@app.get("/api/v1/status")
async def get_status():
    return {"status": "ok"}

@router.post("/api/v1/items")
async def create_item(item: dict):
    return item

@app.route("/flask/login", methods=["POST"])
def flask_login():
    return "ok"
"#;

    let parsed = registry
        .parse_source(py_code, SupportedLanguage::Python)
        .unwrap();
    let routes = detector.detect_routes(&parsed);

    assert_eq!(routes.len(), 3);

    let status_route = routes
        .iter()
        .find(|r| r.path_pattern == "/api/v1/status")
        .unwrap();
    assert_eq!(status_route.method, HttpMethod::Get);
    assert_eq!(status_route.handler_name.as_deref(), Some("get_status"));

    let items_route = routes
        .iter()
        .find(|r| r.path_pattern == "/api/v1/items")
        .unwrap();
    assert_eq!(items_route.method, HttpMethod::Post);

    let flask_route = routes
        .iter()
        .find(|r| r.path_pattern == "/flask/login")
        .unwrap();
    assert_eq!(flask_route.method, HttpMethod::Post);
}

#[test]
fn test_05_database_models_detection() {
    let registry = ParserRegistry::new();
    let detector = DataModelDetector::new();

    // 1. Django ORM
    let django_code = r#"
from django.db import models

class UserProfile(models.Model):
    username = models.CharField(max_length=100)
    created_at = models.DateTimeField(auto_now_add=True)
"#;
    let parsed_django = registry
        .parse_source(django_code, SupportedLanguage::Python)
        .unwrap();
    let models_django = detector.detect_models(&parsed_django);
    assert_eq!(models_django.len(), 1);
    assert_eq!(models_django[0].name, "UserProfile");
    assert_eq!(models_django[0].kind, DataModelKind::OrmEntity);
    assert_eq!(models_django[0].framework_or_orm, "Django ORM");

    // 2. TypeORM Entity
    let typeorm_code = r#"
import { Entity, PrimaryGeneratedColumn, Column } from "typeorm";

@Entity()
export class AccountEntity {
    @PrimaryGeneratedColumn()
    id: number;
}
"#;
    let parsed_typeorm = registry
        .parse_source(typeorm_code, SupportedLanguage::TypeScript)
        .unwrap();
    let models_typeorm = detector.detect_models(&parsed_typeorm);
    assert_eq!(models_typeorm.len(), 1);
    assert_eq!(models_typeorm[0].name, "AccountEntity");
    assert_eq!(models_typeorm[0].framework_or_orm, "TypeORM");

    // 3. SQL Migration File
    let migration_path = PathBuf::from("db/migrations/20240101_initial_schema.sql");
    let mut parsed_sql = registry
        .parse_source(
            "CREATE TABLE users (id SERIAL PRIMARY KEY);",
            SupportedLanguage::C,
        )
        .unwrap();
    parsed_sql.path = Some(migration_path);
    let models_sql = detector.detect_models(&parsed_sql);
    assert_eq!(models_sql.len(), 1);
    assert_eq!(models_sql[0].kind, DataModelKind::SqlMigration);
    assert_eq!(models_sql[0].name, "20240101_initial_schema.sql");
}

#[test]
fn test_06_structure_analyzer_end_to_end() {
    let dir = tempdir().unwrap();
    let root = dir.path().to_path_buf();

    let server_file = root.join("server.js");
    fs::write(
        &server_file,
        "const app = require('express')();\napp.get('/api/ping', ping);\n",
    )
    .unwrap();

    let registry = ParserRegistry::new();
    let parsed_server = registry
        .parse_file(&server_file, Some("javascript"))
        .unwrap();
    let symbols = extract_and_index(&parsed_server);

    let analyzer = StructureAnalyzer::new();
    let structure = analyzer.analyze(&[parsed_server], &symbols);

    assert_eq!(structure.summary.entry_point_count, 1);
    assert_eq!(structure.summary.route_count, 1);
    assert_eq!(structure.entrypoints[0].kind, EntryPointKind::ServerEntry);
    assert_eq!(structure.routes[0].path_pattern, "/api/ping");
}
