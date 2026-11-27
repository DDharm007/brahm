# BRAHM

**Blind Reverse-engineering & Adversarial Honeypot Mechanism**

> **Category:** Cybersecurity / Software Deception / Repository Protection  
> **Status:** Stages 01–06 Implemented (Repository Analyzer · Code Intelligence · Decoy Engine · Deception Mirrors · Agent Integration · Monitoring & Intelligence)

---

## 1. Overview

BRAHM is a pure-Rust, deterministic cybersecurity and controlled software-deception framework engineered to analyze software repositories, reconstruct genuine application architectures, generate isolated synthetic decoy artifacts, assemble controlled inspection environments, onboard authorized workflows, and monitor canary/integrity telemetry.

BRAHM operates **strictly without runtime AI models or LLM APIs**. Every scan, AST parse, decoy synthesis, deception policy check, integrity hash verification, and security rule correlation executes deterministically in Rust.

---

## 2. Six-Stage Architecture

1. **Stage 01 — Repository Analyzer:** Recursive scanning, 25-language detection, manifest discovery, BLAKE3 integrity hashing, JSON/Markdown/terminal reports.
2. **Stage 02 — Code Intelligence Engine:** Multi-language Tree-sitter 0.25 AST parsing (8 languages), BLAKE3 symbol indexing, dependency/cycle resolution, entry/route/ORM detection, `petgraph` architecture graph & blast radius (`BRAHM_ARCHITECTURE.*`, `BRAHM_DEPENDENCY_GRAPH.json`, `BRAHM_SYMBOL_INDEX.json`).
3. **Stage 03 — Decoy Generation Engine (`src/deception/decoy/`):** Architecture-aware non-executing template generator, non-secret `BRAHM-CANARY-...` markers, persistent `BRAHM_DECOY_REGISTRY.json`, `BRAHM_DECOY_MANIFEST.json`, and ownership-gated dry-run/real cleanup.
4. **Stage 04 — Repository Deception Engine (`src/deception/`):** Layer isolation (`GenuineLayer`, `SyntheticLayer`, `InspectionLayer`), secret-stripping inspection mirrors, deterministic policy engine, isolated session lifecycle, and `ProductionIntegrityGuard`.
5. **Stage 05 — Agent Integration & Hybrid Protection (`src/integration/`):** Local & credential-safe GitHub repository onboarding, vendor-agnostic `InspectionAgent` contract + `LocalTestAgentAdapter`, explicit-approval `ProtectionInstaller`, visible guides (`BRAHM_INSPECTION_GUIDE.md`, `BRAHM_SECURITY.md`), and safe rollback.
6. **Stage 06 — Monitoring & Intelligence Engine (`src/monitoring/`):** Indexed bundled SQLite event store (`rusqlite`), deduplicating `CanaryDetector`, chronological `TimelineCorrelator`, rule-based `RuleBasedClassifier`, continuous BLAKE3 `IntegrityMonitor`, optional authenticated localhost Axum API, and intelligence reports (`BRAHM_MONITORING_REPORT.json`, `BRAHM_SECURITY_EVENTS.json`, `BRAHM_SESSION_TIMELINE.json`, `BRAHM_INTEGRITY_REPORT.json`, `BRAHM_INTELLIGENCE_REPORT.md`).

---

## 3. Repository Layout

```text
brahm/
├── src/
│   ├── main.rs            # Unified CLI entry point (Stages 01–06)
│   ├── lib.rs             # Core library module exports
│   ├── cli/               # Clap CLI command tree & argument validation
│   ├── scanner/           # Recursive filesystem traversal engine
│   ├── detection/         # Language, manifest, and framework detection
│   ├── classification/    # File role classification
│   ├── analysis/          # Statistics aggregation & BLAKE3 hashing
│   ├── parser/            # Stage 02: Multi-language Tree-sitter AST engine
│   ├── symbols/           # Stage 02: Symbol extraction & BLAKE3 symbol index
│   ├── dependencies/      # Stage 02: Import/export resolution & cycle detection
│   ├── structure/         # Stage 02: Entry points, HTTP routes & ORM detection
│   ├── graph/             # Stage 02: Petgraph dependency graph & blast radius
│   ├── architecture/      # Stage 02: Architecture intelligence synthesis
│   ├── deception/         # Stages 03 & 04: Decoys, templates, mirror, policies, sandbox & guard
│   ├── integration/       # Stage 05: Repo onboarding, InspectionAgent, installer & rollback
│   ├── monitoring/        # Stage 06: SQLite event store, canaries, rules, integrity & Axum API
│   ├── config/            # Versioned TOML configuration with .bak backup support
│   ├── reporting/         # Terminal, JSON, and Markdown generators
│   ├── models/            # Stage 01 domain schemas
│   └── errors/            # Centralized thiserror error taxonomy
├── tests/                 # Stage 01–06 unit, integration, validation & E2E suites
├── fixtures/              # Fixture repositories (A, B, C, D)
├── docs/                  # Architecture, Security, CLI, Data Models & Stage specs
├── Cargo.toml
└── README.md
```

---

## 4. Building & Running

```powershell
# Compile and verify formatting, lints, and all tests
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test

# Build optimized release binary
cargo build --release
```

### Quick Usage Examples
```powershell
brahm --help
brahm scan ./fixtures/fixture_b_fullstack --arch --output ./reports
brahm decoy generate ./fixtures/fixture_b_fullstack --profile standard --output ./.brahm_decoys
brahm deception prepare ./fixtures/fixture_b_fullstack --decoy-dir ./.brahm_decoys --output ./.brahm_sessions
brahm protection install ./fixtures/fixture_b_fullstack --profile controlled-inspection --state-dir ./.brahm_state --approve
brahm integrity check --state-dir ./.brahm_state
brahm reports generate --state-dir ./.brahm_state
```

See [`docs/CLI_REFERENCE.md`](docs/CLI_REFERENCE.md) for the complete command catalog and [`docs/SECURITY_MODEL.md`](docs/SECURITY_MODEL.md) for strict ethical and security boundaries.
