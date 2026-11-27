# BRAHM Import, Export & Dependency Analysis (`src/dependencies/`)

## 1. Extraction Capabilities

`src/dependencies/languages/` parses language-native import and export constructs directly from Tree-sitter CSTs:

| Language | Captured Imports | Captured Exports |
|---|---|---|
| **Rust** | `use_declaration`, `mod_item` declarations | `pub use`, `pub` visibility tracking |
| **JS / TS / TSX** | ES6 `import` (default, named, namespace, side-effect), CommonJS `require(...)` | `export default`, named `export`, re-exports (`export ... from`) |
| **Python** | `import x`, `from pkg.mod import y`, relative `.mod` / `..mod` | Top-level definitions and module exports |
| **Go** | Single and grouped `import (...)` specs | Exported capitalization convention |
| **Java** | `import pkg.Class;`, wildcard & `static` imports | `public` top-level class/interface exports |
| **C / C++** | Quoted `#include "local.h"` vs. angle `#include <sys.h>` | Header file symbols |

---

## 2. Import Path Resolution (`PathResolver`)

`src/dependencies/resolver.rs` resolves raw import strings deterministically without invoking package managers or compilers:
- Pre-populates a fast in-memory cache of canonical known repository files.
- **JS/TS:** Checks exact files plus `.ts`, `.tsx`, `.js`, `.jsx`, `.mjs` and `/index.{ts,tsx,js,jsx}` fallbacks. Resolves relative paths internally while classifying non-relative specifiers (e.g., `express`, `react`) as `External`.
- **Rust:** Maps `crate::`, `super::`, and `self::` against the enclosing `Cargo.toml` / `src/` hierarchy (`foo.rs` or `foo/mod.rs`). Treats third-party crates and `std::`/`core::`/`alloc::` as external.
- **Python:** Distinguishes relative dotted imports (`.models` -> `models.py` or `models/__init__.py`) from installed packages (`fastapi`, `sqlalchemy`).
- **Go / Java:** Matches internal package directories or `src/main/java/...` roots before marking unresolved third-party domains (`github.com/...`, `org.springframework.*`) as external.
- **C / C++:** Strictly distinguishes quoted local includes from system `<...>` headers.

---

## 3. Cycle Detection (`CycleDetector`)

`src/dependencies/cycles.rs` traverses the directed internal file graph using DFS recursion stacks (`visiting` + `visited` sets) and normalizes discovered cycles by lexicographically minimal node rotation so duplicate equivalent loops (e.g. `A->B->A` and `B->A->B`) are deduplicated into a canonical `DependencyCycle`.
