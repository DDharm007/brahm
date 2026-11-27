//! Core types and representations for the BRAHM Architecture & Dependency Graph.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Categorization of vertices in the architecture graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    /// A source code or manifest file.
    File,
    /// A namespace, package or module container.
    Module,
    /// A standalone function.
    Function,
    /// A method belonging to a class, struct, or interface.
    Method,
    /// An object-oriented class declaration.
    Class,
    /// An interface, protocol or trait.
    Interface,
    /// A struct or record type.
    Struct,
    /// An enumeration type.
    Enum,
    /// An HTTP / API route or endpoint.
    ApiRoute,
    /// A persistent database table, ORM entity, or migration model.
    DataModel,
    /// A third-party or external dependency / package.
    ExternalPackage,
}

impl std::fmt::Display for NodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeKind::File => write!(f, "file"),
            NodeKind::Module => write!(f, "module"),
            NodeKind::Function => write!(f, "function"),
            NodeKind::Method => write!(f, "method"),
            NodeKind::Class => write!(f, "class"),
            NodeKind::Interface => write!(f, "interface"),
            NodeKind::Struct => write!(f, "struct"),
            NodeKind::Enum => write!(f, "enum"),
            NodeKind::ApiRoute => write!(f, "api_route"),
            NodeKind::DataModel => write!(f, "data_model"),
            NodeKind::ExternalPackage => write!(f, "external_package"),
        }
    }
}

/// A node in the architecture graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNode {
    /// Unique deterministic identifier.
    pub id: String,
    /// Human-readable name or label.
    pub name: String,
    /// Kind of architectural entity.
    pub kind: NodeKind,
    /// Source file path where this entity is declared (if applicable).
    pub file_path: Option<PathBuf>,
    /// Additional context-specific properties.
    pub metadata: HashMap<String, String>,
}

impl GraphNode {
    /// Create a new graph node.
    pub fn new(id: impl Into<String>, name: impl Into<String>, kind: NodeKind) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            kind,
            file_path: None,
            metadata: HashMap::new(),
        }
    }

    /// Set file path builder method.
    pub fn with_file_path(mut self, path: PathBuf) -> Self {
        self.file_path = Some(path);
        self
    }

    /// Add a metadata key-value pair.
    pub fn with_meta(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Categorization of relationships between nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    /// Containment / hierarchy (e.g., File contains Function, Class contains Method).
    Contains,
    /// Code dependency import (e.g., File imports File, File imports ExternalPackage).
    Imports,
    /// Execution invocation (e.g., Function calls Function).
    Calls,
    /// Routing relationship (e.g., ApiRoute routes to Handler function).
    RoutesTo,
    /// Inheritance or interface implementation (e.g., Class inherits Class, Struct implements Interface).
    InheritsFrom,
    /// General architectural dependency.
    DependsOn,
}

impl std::fmt::Display for EdgeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EdgeKind::Contains => write!(f, "contains"),
            EdgeKind::Imports => write!(f, "imports"),
            EdgeKind::Calls => write!(f, "calls"),
            EdgeKind::RoutesTo => write!(f, "routes_to"),
            EdgeKind::InheritsFrom => write!(f, "inherits_from"),
            EdgeKind::DependsOn => write!(f, "depends_on"),
        }
    }
}

/// An edge representing a relationship between two nodes in the graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Relationship kind.
    pub kind: EdgeKind,
    /// Weight or strength of relationship.
    pub weight: f64,
    /// Optional context note or description.
    pub metadata: Option<String>,
}

impl GraphEdge {
    /// Create a new graph edge.
    pub fn new(kind: EdgeKind) -> Self {
        Self {
            kind,
            weight: 1.0,
            metadata: None,
        }
    }

    /// Set edge weight.
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }

    /// Set edge metadata note.
    pub fn with_meta(mut self, note: impl Into<String>) -> Self {
        self.metadata = Some(note.into());
        self
    }
}

impl Default for GraphEdge {
    fn default() -> Self {
        Self::new(EdgeKind::DependsOn)
    }
}
