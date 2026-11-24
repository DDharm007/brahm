# BRAHM Repository Intelligence Report

> Generated deterministically by **BRAHM (Blind Reverse-engineering & Adversarial Honeypot Mechanism)**

## 1. Executive Overview

| Metric | Value |
|---|---|
| **Repository Name** | `fixture_b_fullstack` |
| **Target Path** | `//?/D:/Personal Projects/Brahm BLA/fixtures/fixture_b_fullstack` |
| **Canonical Path** | `//?/D:/Personal Projects/Brahm BLA/fixtures/fixture_b_fullstack` |
| **Scan Duration** | `1 ms` |
| **Total Scanned Files** | `13` |
| **Total Directories** | `10` |
| **Total Scanned Size** | `2.01 kB` |
| **Excluded Items** | `0` |
| **Scan Warnings** | `0` |

## 2. Repository Structure

| Directory Depth | Discovered Directories | Total Files Encountered |
|---|---|---|
| Maximum Tree Depth: `4` | `10` directories | `13` files |

## 3. Language Distribution

| Language | Files | % Files | Total Size | % Bytes |
|---|---|---|---|---|
| **JavaScript** | `3` | `23.1%` | `529 B` | `26.3%` |
| **JSON** | `2` | `15.4%` | `310 B` | `15.4%` |
| **YAML** | `1` | `7.7%` | `275 B` | `13.7%` |
| **TypeScript** | `2` | `15.4%` | `198 B` | `9.9%` |
| **Markdown** | `1` | `7.7%` | `166 B` | `8.3%` |
| **SQL** | `1` | `7.7%` | `160 B` | `8%` |
| **Dockerfile** | `1` | `7.7%` | `143 B` | `7.1%` |

## 4. Framework Detection

| Framework | Confidence Level | Evidence Discovered |
|---|---|---|
| **Node.js** | `Confirmed by Manifest` | Identified 'backend/package.json' manifest; Identified 'frontend/package.json' manifest |
| **Next.js** | `Confirmed by Manifest` | 'frontend/package.json' declares 'next' dependency; Next.js configuration file present |
| **React** | `Confirmed by Manifest` | 'frontend/package.json' declares 'react' dependency; React implicitly required by Next.js framework |

## 5. File Classification

| Classification Category | File Count |
|---|---|
| **Configuration** | `5` |
| **Database** | `1` |
| **Documentation** | `1` |
| **Infrastructure** | `2` |
| **Source Code** | `4` |

## 6. Important Configurations

- `.env.example`
- `Dockerfile`
- `README.md`
- `backend/package.json`
- `docker-compose.yml`
- `frontend/next.config.js`
- `frontend/package.json`

## 7. Repository Statistics

- **Total Scanned Bytes:** `2009`
- **Configuration Files Count:** `5`
- **Package Manifest Count:** `2`
- **Unknown File Format Count:** `0`

### File Extension Summary

| Extension | Count |
|---|---|
| `(no extension)` | `2` |
| `example` | `1` |
| `js` | `3` |
| `json` | `2` |
| `md` | `1` |
| `sql` | `1` |
| `ts` | `1` |
| `tsx` | `1` |
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
- **Scan Started At:** `2026-11-24 06:03:43.642715 UTC`
- **Scan Completed At:** `2026-11-24 06:03:43.644459100 UTC`
- **Elapsed Scan Time:** `1 ms`

