# BRAHM Security Model & Ethical Engineering Guarantees

## 1. Non-Negotiable Core Guarantees

1. **Pure Deterministic Execution (Zero Runtime AI):** BRAHM uses zero runtime LLM APIs or generative cloud inference. Every parse, classification, decoy render, and telemetry correlation uses auditable Rust logic.
2. **Read-Only Original Repositories:** Target repositories scanned by Stage 01–06 are never mutated, executed, or contaminated with decoys or configuration scripts.
3. **Production Build Isolation:** `ProductionIntegrityGuard` checks both genuine trees and build manifests (`Cargo.toml`, `package.json`, `pyproject.toml`, `go.mod`, `pom.xml`, `build.gradle`) to reject decoy leaks before mirror preparation or installation.
4. **Inert Non-Secret Decoys & Canaries:** Canaries use non-routable RFC 2606 domains (`*.invalid`) and synthetic markers (`BRAHM-CANARY-<HEX>`). `TemplateRegistry` and `DecoyValidator` strictly block AWS/Stripe/GitHub credential patterns and executable OS calls (`Command`, `eval`, `unsafe`).
5. **Secret Stripping & Telemetry Data Minimization:** `.env`, `.env.*`, `*.pem`, `*.key`, and private key files are excluded from hashing, AST parsing, inspection mirrors, and monitoring SQLite metadata.
6. **GitHub & Network Discipline:** GitHub URL inputs reject embedded plaintext credentials (`token@github.com`) and shell characters; monitoring API defaults strictly to `127.0.0.1` with mandatory Bearer authentication (`BRAHM_MONITOR_TOKEN`) and sliding-window rate limiting.
7. **Ownership-Gated Cleanup & Rollback:** Removal and rollback validate `brahm_owned == true`, path confinement, and exact BLAKE3 digests so unknown or user-modified files are never deleted.

---

## 2. Explicit Scope Boundaries & Technical Honesty

- **No Prevention Claim:** Accessing decoys or canaries demonstrates interaction inside an instrumented environment; it does **not** prove prevention of reverse engineering or unauthorized copying.
- **No Actor Attribution Claim:** Canary hits record interaction with registered artifacts under current telemetry context and carry explicit disclaimers stating actor identity (human developer vs. specific AI assistant) cannot be definitively attributed without independent external verification.
- **No Public GitHub Stealth Selective Serving:** Selective inspection views are provided strictly through isolated local mirrors and authorized inspection sandboxes (`ControlledInspectionEnvironment`), not deceptive public GitHub multiplexing.
