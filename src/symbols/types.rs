//! Domain models and types for extracted code symbols.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::parser::SourceSpan;

/// Category of extracted code symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    Function,
    Method,
    Constructor,
    Class,
    Interface,
    Struct,
    Enum,
    Trait,
    Module,
    TypeAlias,
    Constant,
}

impl SymbolKind {
    /// Canonical lowercase string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::Method => "method",
            Self::Constructor => "constructor",
            Self::Class => "class",
            Self::Interface => "interface",
            Self::Struct => "struct",
            Self::Enum => "enum",
            Self::Trait => "trait",
            Self::Module => "module",
            Self::TypeAlias => "type_alias",
            Self::Constant => "constant",
        }
    }

    /// User-friendly display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Function => "Function",
            Self::Method => "Method",
            Self::Constructor => "Constructor",
            Self::Class => "Class",
            Self::Interface => "Interface",
            Self::Struct => "Struct",
            Self::Enum => "Enum",
            Self::Trait => "Trait",
            Self::Module => "Module",
            Self::TypeAlias => "Type Alias",
            Self::Constant => "Constant",
        }
    }
}

impl std::fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Declared visibility or access level of a symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
    Unknown,
}

impl Visibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Private => "private",
            Self::Protected => "protected",
            Self::Internal => "internal",
            Self::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Metadata describing a parameter in a function or method signature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParameterInfo {
    /// Parameter identifier name.
    pub name: String,
    /// Declared type annotation if available.
    pub type_annotation: Option<String>,
    /// Default argument value if specified.
    pub default_value: Option<String>,
}

impl ParameterInfo {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            type_annotation: None,
            default_value: None,
        }
    }

    pub fn with_type(mut self, type_ann: impl Into<String>) -> Self {
        self.type_annotation = Some(type_ann.into());
        self
    }

    pub fn with_default(mut self, default_val: impl Into<String>) -> Self {
        self.default_value = Some(default_val.into());
        self
    }
}

/// A structured, statically detected software symbol.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Symbol {
    /// Deterministic 16-character hex identifier.
    pub id: String,

    /// Unqualified bare name of the symbol (e.g. `login`).
    pub name: String,

    /// Fully qualified path/name (e.g. `crate::auth::AuthService::login`).
    pub qualified_name: String,

    /// Symbol category.
    pub kind: SymbolKind,

    /// Visibility / access level.
    pub visibility: Visibility,

    /// Source file path where the symbol is defined.
    pub file_path: PathBuf,

    /// Precise coordinate span in the source code.
    pub span: SourceSpan,

    /// Full declared signature string if applicable.
    pub signature: Option<String>,

    /// Return type if declared.
    pub return_type: Option<String>,

    /// Extracted parameters if applicable.
    pub parameters: Vec<ParameterInfo>,

    /// ID of the enclosing parent symbol if nested.
    pub parent_id: Option<String>,

    /// Extracted documentation comment if present.
    pub doc_comment: Option<String>,

    /// Whether the function/method is marked async.
    pub is_async: bool,

    /// Whether the method/field is static.
    pub is_static: bool,
}

impl Symbol {
    /// Helper to test if this symbol is callable.
    pub fn is_callable(&self) -> bool {
        matches!(
            self.kind,
            SymbolKind::Function | SymbolKind::Method | SymbolKind::Constructor
        )
    }

    /// Helper to test if this symbol defines a type.
    pub fn is_type_definition(&self) -> bool {
        matches!(
            self.kind,
            SymbolKind::Class
                | SymbolKind::Interface
                | SymbolKind::Struct
                | SymbolKind::Enum
                | SymbolKind::Trait
                | SymbolKind::TypeAlias
        )
    }
}
