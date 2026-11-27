# BRAHM Stage 03 — Decoy Generation Engine (`src/deception/decoy/`)

## 1. Objective

Generates deterministic, architecture-aware synthetic decoy artifacts informed by Stage 02 `ArchitectureReport` telemetry while keeping all generated artifacts physically isolated inside a dedicated decoy staging layer (`decoy_layer`).

---

## 2. Module Responsibilities

| Module | File | Responsibility |
|---|---|---|
| **Models** | `src/deception/decoy/models.rs` | `DecoyArtifact`, `DecoyCategory`, `DecoyTemplate`, `DecoyRegistryData`, `DecoyManifest`, `DecoyGenerationProfile` |
| **Identifiers** | `src/deception/decoy/identifiers.rs` | Deterministic BLAKE3 IDs (`repo_...`, `decoy_...`), `BRAHM-CANARY-...` markers, content digests |
| **Templates** | `src/deception/decoy/templates.rs` | Non-executing `TemplateRegistry` with validation & `{{KEY}}` substitution |
| **Generator** | `src/deception/decoy/generator.rs` | `DecoyGenerator` selecting templates strictly from observed Stage 02 languages/frameworks/models |
| **Registry** | `src/deception/decoy/registry.rs` | `DecoyRegistry` JSON persistence, search, deactivation, integrity verification, and ownership-gated cleanup |
| **Validation** | `src/deception/decoy/validation.rs` | `DecoyValidator` enforcing unique IDs, safe paths, zero credential patterns, inert code checks |
| **Engine** | `src/deception/decoy/engine.rs` | `DecoyEngine` generating `BRAHM_DECOY_REGISTRY.json`, `BRAHM_DECOY_MANIFEST.json`, `BRAHM_DECOY_REPORT.md` |

---

## 3. Decoy Categories & Profiles

- **Categories:** `SyntheticSource`, `SyntheticDocumentation`, `SyntheticConfiguration`, `CanaryArtifact`, `SyntheticDependency`.
- **Profiles (`DecoyGenerationProfile`):**
  - `Disabled`: 0 artifacts generated.
  - `Conservative`: Canary anchor + `SYSTEM_ARCHITECTURE.md`.
  - `Standard`: Documentation + synthetic dependency catalog + language/API/ORM stubs matching Stage 02 evidence.
  - `Research`: Expanded synthetic documentation + secondary JSON canary token.

---

## 4. Safety Guarantees
- Staging output directory cannot match `genuine_repo_root`.
- Decoys never overwrite `Cargo.toml`, `package.json`, `pyproject.toml`, `go.mod`, `pom.xml`, or verified `BRAHM_ARCHITECTURE.*`.
- Safe cleanup (`cleanup_owned_artifacts`) verifies both `brahm_owned == true` AND BLAKE3 file match before deletion, supporting `--dry-run`.
