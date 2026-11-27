# BRAHM Symbol Extraction & Indexing (`src/symbols/`)

## 1. Extracted Symbol Taxonomy (`SymbolKind`)

`src/symbols/types.rs` defines uniform cross-language classifications:
- `Function` — Top-level functions and closures bound to declarations
- `Method` — Functions associated with classes, structs, interfaces, or `impl` blocks
- `Class` — Object-oriented class definitions
- `Interface` — Interfaces, protocols, and Rust `trait` definitions
- `Struct` — Struct, record, and data class types
- `Enum` — Enumeration definitions
- `Module` — Namespaces, Rust `mod` items, and C++ `namespace` declarations
- `Variable` / `Constant` — Top-level exported constants or global bindings

Each `Symbol` records:
- Deterministic `id`
- `name`, `qualified_name`, and `parent_id` hierarchy
- `visibility` (`Public`, `Private`, `Protected`, `Internal`)
- `parameters: Vec<ParameterInfo>` and `return_type`
- `signature` and `doc_comment`
- `file_path` and exact `SourceSpan`

---

## 2. Deterministic Identity Generation (`src/symbols/id.rs`)

`generate_symbol_id` produces stable 16-character hexadecimal IDs via **BLAKE3** hashing of:
```text
<normalized_relative_path> :: <symbol_kind> :: <qualified_name> :: <start_line>
```
Platform path separators (`\` vs `/`) are normalized to `/` before hashing so Linux, macOS, and Windows produce identical symbol IDs for the same codebase.

---

## 3. Language-Specific Extraction Notes (`src/symbols/languages/`)
- **Rust:** Distinguishes `impl` methods via `impl_item` traversal and detects `visibility_modifier` child nodes (`pub`, `pub(crate)`).
- **JavaScript / TypeScript / TSX:** Unwraps parent `export_statement` blocks so JSDoc comments attached above `export class` or `export function` are accurately captured alongside `accessibility_modifier` visibility.
- **Python:** Handles `@decorator` wrappers (`decorated_definition`), extracts triple-quoted docstrings as documentation comments, and converts Python leading underscore naming (`_internal`, `__private`) into visibility rules.
- **Go:** Applies Go capitalization rules (uppercase initial identifier = `Public`, lowercase = `Internal`/package-private) across structs, interfaces, functions, and receiver methods.
- **Java:** Parses child `modifiers` nodes (`public`, `private`, `protected`) on classes, interfaces, enums, constructors, and methods.
- **C / C++:** Captures functions, structs, classes, enums, and namespaces, cleaning pointer/reference declarator chains.
