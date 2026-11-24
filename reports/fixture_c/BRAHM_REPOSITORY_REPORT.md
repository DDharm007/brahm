# BRAHM Repository Intelligence Report

> Generated deterministically by **BRAHM (Blind Reverse-engineering & Adversarial Honeypot Mechanism)**

## 1. Executive Overview

| Metric | Value |
|---|---|
| **Repository Name** | `fixture_c_multilingual` |
| **Target Path** | `//?/D:/Personal Projects/Brahm BLA/fixtures/fixture_c_multilingual` |
| **Canonical Path** | `//?/D:/Personal Projects/Brahm BLA/fixtures/fixture_c_multilingual` |
| **Scan Duration** | `2 ms` |
| **Total Scanned Files** | `14` |
| **Total Directories** | `5` |
| **Total Scanned Size** | `1.46 kB` |
| **Excluded Items** | `0` |
| **Scan Warnings** | `0` |

## 2. Repository Structure

| Directory Depth | Discovered Directories | Total Files Encountered |
|---|---|---|
| Maximum Tree Depth: `2` | `5` directories | `14` files |

## 3. Language Distribution

| Language | Files | % Files | Total Size | % Bytes |
|---|---|---|---|---|
| **TypeScript** | `1` | `7.1%` | `204 B` | `14%` |
| **TOML** | `2` | `14.3%` | `180 B` | `12.3%` |
| **YAML** | `1` | `7.1%` | `173 B` | `11.8%` |
| **Markdown** | `1` | `7.1%` | `167 B` | `11.4%` |
| **SQL** | `1` | `7.1%` | `167 B` | `11.4%` |
| **Python** | `1` | `7.1%` | `152 B` | `10.4%` |
| **Go** | `2` | `14.3%` | `134 B` | `9.2%` |
| **Shell** | `1` | `7.1%` | `81 B` | `5.5%` |
| **JSON** | `1` | `7.1%` | `79 B` | `5.4%` |
| **Rust** | `1` | `7.1%` | `57 B` | `3.9%` |
| **JavaScript** | `1` | `7.1%` | `37 B` | `2.5%` |

## 4. Framework Detection

| Framework | Confidence Level | Evidence Discovered |
|---|---|---|
| **Rust** | `Confirmed by Manifest` | Found 'Cargo.toml' manifest; Cargo.toml contains valid [package] section |
| **Flask** | `Confirmed by Manifest` | Python manifest declares flask dependency |
| **Go** | `Confirmed by Manifest` | go.mod module manifest present in repository |

## 5. File Classification

| Classification Category | File Count |
|---|---|
| **Build System** | `1` |
| **Configuration** | `4` |
| **Database** | `1` |
| **Deployment** | `1` |
| **Documentation** | `1` |
| **Scripts** | `1` |
| **Source Code** | `5` |

## 6. Important Configurations

- `Cargo.toml`
- `README.md`
- `go.mod`
- `pyproject.toml`
- `requirements.txt`

## 7. Repository Statistics

- **Total Scanned Bytes:** `1461`
- **Configuration Files Count:** `4`
- **Package Manifest Count:** `4`
- **Unknown File Format Count:** `0`

### File Extension Summary

| Extension | Count |
|---|---|
| `go` | `1` |
| `js` | `1` |
| `json` | `1` |
| `md` | `1` |
| `mod` | `1` |
| `py` | `1` |
| `rs` | `1` |
| `sh` | `1` |
| `sql` | `1` |
| `toml` | `2` |
| `ts` | `1` |
| `txt` | `1` |
| `yml` | `1` |

## 8. Excluded Paths

_No paths were excluded during scanning._

## 9. Scan Warnings

_Zero scan warnings recorded. Traversal was clean._

## 10. Technical Limitations

- Deterministic Stage 01 analysis operates without runtime AI models or abstract syntax tree (AST) semantic inferencing.
- File hashing represents cryptographic integrity fingerprints (BLAKE3), not dynamic trust evaluations or behavior validation.
- Secret values, private keys, and credential stores are masked and intentionally excluded from reports.

## 11. Scan Metadata

- **BRAHM Version:** `0.1.0`
- **Scan Started At:** `2026-11-24 06:03:43.746805800 UTC`
- **Scan Completed At:** `2026-11-24 06:03:43.749307800 UTC`
- **Elapsed Scan Time:** `2 ms`

