//! Central Symbol Index and query engine.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::types::{Symbol, SymbolKind};

/// High-level summary metrics of extracted symbols in a repository.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SymbolSummary {
    /// Total count of extracted symbols.
    pub total_symbols: usize,
    /// Number of unique files containing symbols.
    pub files_with_symbols: usize,
    /// Counts broken down by symbol kind.
    pub by_kind: HashMap<String, usize>,
    /// Counts broken down by visibility.
    pub by_visibility: HashMap<String, usize>,
}

/// In-memory indexed database of symbols extracted from repository sources.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SymbolIndex {
    /// All indexed symbols.
    symbols: Vec<Symbol>,

    #[serde(skip)]
    by_id: HashMap<String, usize>,

    #[serde(skip)]
    by_file: HashMap<PathBuf, Vec<usize>>,

    #[serde(skip)]
    by_kind: HashMap<SymbolKind, Vec<usize>>,

    #[serde(skip)]
    by_name: HashMap<String, Vec<usize>>,
}

impl SymbolIndex {
    /// Create an empty `SymbolIndex`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct a `SymbolIndex` from an existing list of symbols, building all lookup tables.
    pub fn from_symbols(symbols: Vec<Symbol>) -> Self {
        let mut index = Self {
            symbols,
            by_id: HashMap::new(),
            by_file: HashMap::new(),
            by_kind: HashMap::new(),
            by_name: HashMap::new(),
        };
        index.rebuild_indices();
        index
    }

    /// Add a single symbol to the index.
    pub fn add(&mut self, symbol: Symbol) {
        let idx = self.symbols.len();
        self.by_id.insert(symbol.id.clone(), idx);

        self.by_file
            .entry(symbol.file_path.clone())
            .or_default()
            .push(idx);

        self.by_kind.entry(symbol.kind).or_default().push(idx);

        self.by_name
            .entry(symbol.name.clone())
            .or_default()
            .push(idx);

        self.symbols.push(symbol);
    }

    /// Add a batch of symbols.
    pub fn extend(&mut self, symbols: impl IntoIterator<Item = Symbol>) {
        for s in symbols {
            self.add(s);
        }
    }

    /// Rebuild index mappings from the internal symbols array.
    pub fn rebuild_indices(&mut self) {
        self.by_id.clear();
        self.by_file.clear();
        self.by_kind.clear();
        self.by_name.clear();

        for (idx, sym) in self.symbols.iter().enumerate() {
            self.by_id.insert(sym.id.clone(), idx);
            self.by_file
                .entry(sym.file_path.clone())
                .or_default()
                .push(idx);
            self.by_kind.entry(sym.kind).or_default().push(idx);
            self.by_name.entry(sym.name.clone()).or_default().push(idx);
        }
    }

    /// Look up a symbol by its unique ID.
    pub fn get_by_id(&self, id: &str) -> Option<&Symbol> {
        self.by_id.get(id).and_then(|&idx| self.symbols.get(idx))
    }

    /// Retrieve all symbols declared in a specific file.
    pub fn get_by_file(&self, path: &Path) -> Vec<&Symbol> {
        self.by_file
            .get(path)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&i| self.symbols.get(i))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Retrieve all symbols matching a specific kind.
    pub fn get_by_kind(&self, kind: SymbolKind) -> Vec<&Symbol> {
        self.by_kind
            .get(&kind)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&i| self.symbols.get(i))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Retrieve all symbols matching an unqualified name.
    pub fn get_by_name(&self, name: &str) -> Vec<&Symbol> {
        self.by_name
            .get(name)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&i| self.symbols.get(i))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Slice view of all symbols.
    pub fn all(&self) -> &[Symbol] {
        &self.symbols
    }

    /// Iterator over all indexed symbols.
    pub fn iter(&self) -> std::slice::Iter<'_, Symbol> {
        self.symbols.iter()
    }

    /// Consume index into vector of symbols.
    pub fn into_vec(self) -> Vec<Symbol> {
        self.symbols
    }

    /// Total number of symbols indexed.
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Check if the index is empty.
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    /// Generate summary metrics for the index.
    pub fn summary(&self) -> SymbolSummary {
        let mut by_kind = HashMap::new();
        let mut by_visibility = HashMap::new();

        for sym in &self.symbols {
            *by_kind.entry(sym.kind.as_str().to_string()).or_insert(0) += 1;
            *by_visibility
                .entry(sym.visibility.as_str().to_string())
                .or_insert(0) += 1;
        }

        SymbolSummary {
            total_symbols: self.symbols.len(),
            files_with_symbols: self.by_file.len(),
            by_kind,
            by_visibility,
        }
    }
}
