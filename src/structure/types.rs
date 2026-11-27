//! Domain models for application landmarks: entry points, API routes, and data models.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::parser::SourceSpan;

/// Category of application entry point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryPointKind {
    /// Executable binary main entry point (e.g. `fn main()` in Rust/Go/C++).
    Executable,
    /// Root library module exposed to downstream consumers (e.g. `lib.rs`, `index.ts`).
    LibraryRoot,
    /// Web server or HTTP daemon startup file (e.g. `server.ts`, `app.py`, `wsgi.py`).
    ServerEntry,
    /// Command-line tool entry point.
    CliEntry,
    /// Background queue worker, cron, or job daemon entry point.
    WorkerEntry,
}

impl EntryPointKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Executable => "executable",
            Self::LibraryRoot => "library_root",
            Self::ServerEntry => "server_entry",
            Self::CliEntry => "cli_entry",
            Self::WorkerEntry => "worker_entry",
        }
    }
}

/// A detected application landmark entry point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntryPoint {
    /// File containing the entry point.
    pub file_path: PathBuf,

    /// Entry point category.
    pub kind: EntryPointKind,

    /// Specific function or variable acting as the entry point (e.g. `"main"`).
    pub symbol_name: Option<String>,

    /// Source coordinate span if found in parsed code.
    pub span: Option<SourceSpan>,

    /// Human-readable explanation of why this was identified as an entry point.
    pub description: String,

    /// Confidence score from 0.0 (heuristic guess) to 1.0 (verified language entry point).
    pub confidence: f32,
}

/// Supported HTTP methods for API route detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Options,
    Head,
    All,
    WebSocket,
}

impl HttpMethod {
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_uppercase().as_str() {
            "GET" => Some(Self::Get),
            "POST" => Some(Self::Post),
            "PUT" => Some(Self::Put),
            "DELETE" => Some(Self::Delete),
            "PATCH" => Some(Self::Patch),
            "OPTIONS" => Some(Self::Options),
            "HEAD" => Some(Self::Head),
            "ALL" | "USE" => Some(Self::All),
            "WS" | "WEBSOCKET" => Some(Self::WebSocket),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
            Self::Options => "OPTIONS",
            Self::Head => "HEAD",
            Self::All => "ALL",
            Self::WebSocket => "WS",
        }
    }
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A detected HTTP API endpoint or route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiRoute {
    /// HTTP method accepted by the route.
    pub method: HttpMethod,

    /// URL path pattern (e.g. `"/api/v1/users/:id"`).
    pub path_pattern: String,

    /// Name or reference of the route handler if identifiable.
    pub handler_name: Option<String>,

    /// File containing the route declaration.
    pub file_path: PathBuf,

    /// Coordinate span in the source file.
    pub span: SourceSpan,

    /// Framework responsible for routing (e.g. `"Express"`, `"FastAPI"`, `"Next.js"`).
    pub framework: String,
}

/// Category of data or database model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataModelKind {
    /// ORM entity model (e.g. Django `models.Model`, TypeORM `@Entity()`, Prisma model).
    OrmEntity,
    /// Relational database schema migration script (`.sql`).
    SqlMigration,
    /// Schema or query definition file.
    SchemaDefinition,
}

/// A detected database model or persistent data structure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataModel {
    /// Name of the model (e.g. `"User"`, `"20240101_init.sql"`).
    pub name: String,

    /// Category of model.
    pub kind: DataModelKind,

    /// File where the model is defined.
    pub file_path: PathBuf,

    /// Coordinate span if applicable.
    pub span: Option<SourceSpan>,

    /// Framework, ORM, or database technology.
    pub framework_or_orm: String,

    /// Backing database table name if detected.
    pub table_name: Option<String>,

    /// Extracted fields or columns if applicable.
    pub fields: Vec<String>,
}
