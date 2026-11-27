# BRAHM

**Blind Reverse-engineering & Adversarial Honeypot Mechanism**

> **Category:** Cybersecurity / Software Deception / Repository Protection  
> **Status:** Stage 01 (Repository Analyzer) & Stage 02 (Code Intelligence & Architecture Mapping Engine)

---

## 1. Overview

BRAHM is an experimental cybersecurity framework engineered to analyze software repositories, map genuine application architectures deterministically, and establish the structural foundation for controlled repository deception and adversarial honeypots.

BRAHM operates **strictly without runtime AI models or LLM APIs**. All intelligence extraction, AST parsing, dependency mapping, graph analysis, categorization, and integrity verification are performed using deterministic algorithms, hardcoded rules, Tree-sitter grammars, and conventional systems programming in Rust.

---

## 2. Capabilities (Stage 01 + Stage 02)

### Stage 01 — Repository Analyzer
- **Recursive Traversal:** Secure scanning with path canonicalization, symlink boundary protection, and file size quotas.
- **Language & File Classification:** Deterministic identification of 25 programming languages and 13 structural file roles.
- **Framework & Manifest Discovery:** Evidence-backed framework indicators (React, Next.js, Vite, Node.js, Rust, Django, Flask, Go, Java) with confidence rankings.
- **Integrity Fingerprinting:** Streaming cryptographic hashing (BLAKE3) without loading large files into memory.
- **Multi-Format Reporting:** Terminal, structured JSON (`BRAHM_REPOSITORY_REPORT.json`), and Markdown (`BRAHM_REPOSITORY_REPORT.md`).

### Stage 02 — Code Intelligence & Architecture Mapping Engine
- **Multi-Language AST Parsing (`src/parser/`):** Powered by Tree-sitter 0.25 across Rust, JavaScript, TypeScript/TSX, Python, Go, Java, C, and C++ with error recovery.
- **Symbol Index (`src/symbols/`):** Extracts functions, methods, classes, interfaces, structs, enums, and modules with deterministic BLAKE3 symbol IDs (`BRAHM_SYMBOL_INDEX.json`).
- **Dependency & Cycle Resolution (`src/dependencies/`):** Resolves internal module imports vs. external ecosystem packages and detects circular dependencies via DFS.
- **Application Structure Landmarks (`src/structure/`):** Detects executable/library/server entry points, HTTP API routes (Express, Fastify, Next.js, FastAPI, Flask, Gin, Echo, Axum, Actix, Spring Boot), and ORM/SQL models.
- **Architecture Graph Engine (`src/graph/`):** Constructs a `petgraph` directed architecture graph with topological sorting, Tarjan SCC cycle detection, centrality ranking, and blast radius impact metrics (`BRAHM_DEPENDENCY_GRAPH.json`).
- **Architecture Blueprints (`src/architecture/`, `src/reporting/`):** Synthesizes architectural patterns and layers, separating deterministic static facts from heuristics/inferences (`BRAHM_ARCHITECTURE.json`, `BRAHM_ARCHITECTURE.md`).

---

## 3. Architecture & Modular Organization

```text
brahm/
├── src/
│   ├── main.rs            # CLI entry point, structured logging, exit codes
│   ├── lib.rs             # Core library module exports
│   ├── cli/               # Clap CLI definition and argument validation
│   ├── scanner/           # Recursive filesystem traversal engine
│   ├── detection/         # Language, manifest, and framework detection
│   ├── classification/    # File categorization logic
│   ├── analysis/          # Repository statistics aggregation & BLAKE3 hashing
│   ├── parser/            # Stage 02: Multi-language Tree-sitter AST engine
│   ├── symbols/           # Stage 02: Symbol extraction & BLAKE3 symbol index
│   ├── dependencies/      # Stage 02: Import/export analysis & cycle detector
│   ├── structure/         # Stage 02: Entry point, API route & ORM detection
│   ├── graph/             # Stage 02: Petgraph dependency graph & blast radius
│   ├── architecture/      # Stage 02: Architecture intelligence orchestrator
│   ├── config/            # TOML configuration parser and validation
│   ├── reporting/         # Terminal, JSON, and Markdown report generators
│   ├── models/            # Domain data models and Serde schemas
│   └── errors/            # Centralized thiserror-based error handling
├── tests/                 # Stage 01 & Stage 02 integration & validation suites
├── fixtures/              # Test fixture repositories (Fixtures A, B, C, D)
├── docs/                  # Architecture and Stage technical specifications
├── Cargo.toml
└── README.md
```

---

## 4. Building & Running

### Prerequisites
- Rust stable (1.80+)
- Cargo

### Compilation
```powershell
# Development build
cargo build

# Optimized release build
cargo build --release
```

### Usage
```powershell
# Display help and version information
brahm --help
brahm --version

# Initialize default configuration
brahm init

# Run Stage 01 repository scan
brahm scan ./my-project --format all --output ./reports

# Run Stage 01 + Stage 02 deep code intelligence & architecture mapping
brahm scan ./my-project --arch --format all --output ./reports
# Or use the dedicated subcommand:
brahm arch ./my-project --format all --output ./reports
```

---

## 5. Security Principles

- **Strictly Read-Only:** Scanned repositories are never mutated, written to, or modified in any form.
- **Zero Script Execution:** No repository scripts, hooks, or package managers are executed.
- **Symlink Boundary Confinement:** Prevents directory traversal attacks escaping the repository root.
- **Credential Protection:** Secrets and sensitive environment variables are strictly masked and never stored in reports.
