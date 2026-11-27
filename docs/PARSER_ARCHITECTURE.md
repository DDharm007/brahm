# BRAHM AST Parser Architecture (`src/parser/`)

## 1. Design Goals
- **Deterministic & Non-Executing:** Parses source files into concrete syntax trees (CST) strictly without evaluating or executing any scanned code.
- **Resilient Error Recovery:** Recovers gracefully from incomplete or malformed syntax (`ERROR` and `MISSING` AST nodes) while extracting valid surrounding nodes.
- **Memory & Encoding Safe:** Bounded file-size limits (`DEFAULT_MAX_PARSE_FILE_SIZE`) and lossy UTF-8 decoding prevent crashes on corrupted encodings.
- **ABI Version Alignment:** Built on `tree-sitter = "0.25"` to cleanly load Language ABI v15 grammar crates.

---

## 2. Core Components

| Component | File | Responsibility |
|---|---|---|
| `SupportedLanguage` | `src/parser/language.rs` | Enum mapping extensions, names, and aliases to grammar implementations |
| `LanguageParser` & `TreeSitterEngine` | `src/parser/engine.rs` | Thread-safe stateless wrapper around `tree_sitter::Parser` per invocation |
| `ParserRegistry` | `src/parser/registry.rs` | Dispatches file paths and language hints to the appropriate Tree-sitter grammar |
| `ParsedSource` | `src/parser/parsed.rs` | Immutable source buffer paired with its parsed `tree_sitter::Tree` and helper queries |
| `SourceSpan` | `src/parser/span.rs` | 1-based line/column and 0-based byte offset coordinate records |
| `ParserConfig` | `src/parser/config.rs` | Configurable parsing thresholds (maximum bytes per source file) |

---

## 3. Thread Safety & Traversal Design

1. `tree_sitter::Parser` is not `Sync`, so `TreeSitterEngine` instantiates parser state locally per parse operation while sharing compiled `tree_sitter::Language` descriptors safely across threads.
2. AST DFS traversal in `ParsedSource::find_nodes_by_kind` explicitly iterates child indices via `current.child_count()` and `current.child(i)` rather than stateful cursors inside recursive stack pops, eliminating cursor state loops.
