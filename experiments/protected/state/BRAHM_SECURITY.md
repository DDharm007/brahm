# BRAHM Security Model & Operational Limitations

**Target Repository:** `brahm-commerce-demo` (`repo_b7153cf65c6656d9`)

## 1. Guarantees Enforced

- **Read-Only Genuine Source:** Genuine application files are never altered during analysis or inspection preparation.
- **Secret Exclusion:** Sensitive credential patterns (`.env`, `*.pem`, `*.key`, etc.) are omitted from inspection mirrors and telemetry.
- **Production Build Isolation:** Synthetic decoy artifacts are excluded from production build manifests and validated via `ProductionIntegrityGuard`.
- **Zero Runtime AI Models:** All detection, parsing, and telemetry correlation rules execute deterministically.

## 2. Explicit Scope & Technical Limitations

- BRAHM does **not** guarantee prevention of reverse engineering or unauthorized copying of public repositories.
- Public Git hosts (e.g., GitHub) serve identical tree views to anonymous clones; selective inspection views apply to controlled local mirrors and authorized sandbox environments.
- Canary interaction events prove access to a registered synthetic artifact within the telemetry boundary; they do **not** definitively attribute identity (human vs. AI model) without independent corroboration.
