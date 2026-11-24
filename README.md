# BRAHM

**Blind Reverse-engineering & Adversarial Honeypot Mechanism**

> **Category:** Cybersecurity / Software Deception / Repository Protection  
> **Status:** Stage 01 — Foundation & Repository Analyzer

---

## 1. Overview

BRAHM is an experimental cybersecurity framework engineered to analyze software repositories, map genuine application architectures deterministically, and establish the structural foundation for controlled repository deception and adversarial honeypots.

BRAHM operates **strictly without runtime AI models or LLM APIs**. All intelligence extraction, parsing, categorization, and integrity verification is performed using deterministic algorithms, hardcoded rules, and conventional systems programming in Rust.

---

## 2. Stage 01 Objective: Repository Analyzer

Stage 01 delivers the **BRAHM Repository Analyzer**, which performs high-assurance analysis of software repositories:

- **Recursive Traversal:** Secure scanning with path canonicalization, symlink boundary protection, and file size quotas.
- **Language & File Classification:** Deterministic identification of programming languages and structural roles (source code, configuration, tests, database, assets, infrastructure).
- **Framework & Manifest Discovery:** Evidence-backed framework indicators (React, Next.js, Vite, Node.js, Rust, Django, Flask, Go, Java) with explicit confidence ranking.
- **Integrity Fingerprinting:** Streaming cryptographic hashing (BLAKE3) without loading large files into memory.
- **Multi-Format Intelligence Reporting:** Terminal, structured JSON (`BRAHM_REPOSITORY_REPORT.json`), and comprehensive Markdown (`BRAHM_REPOSITORY_REPORT.md`).

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
│   ├── analysis/          # Repository statistics aggregation
│   ├── config/            # TOML configuration parser and validation
│   ├── reporting/         # Terminal, JSON, and Markdown report generators
│   ├── models/            # Domain data models and Serde schemas
│   └── errors/            # Centralized thiserror-based error handling
├── tests/                 # Integration and property tests
├── fixtures/              # Test fixture repositories
├── docs/                  # Architecture and technical documentation
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

# Scan a target repository
brahm scan ./my-project --format all --output ./reports
```

---

## 5. Security Principles

- **Strictly Read-Only:** Scanned repositories are never mutated, written to, or modified in any form.
- **Zero Script Execution:** No repository scripts, hooks, or package managers are executed.
- **Symlink Boundary Confinement:** Prevents directory traversal attacks escaping the repository root.
- **Credential Protection:** Secrets and sensitive environment variables are strictly masked and never stored in reports.
