# BRAHM Stage 07 — Issue Register

**Audit Date:** 2026-11-27 | **Baseline Commit:** `1c012c9` | **Machine-Readable Ledger:** [BRAHM_ISSUE_REGISTER.json](./BRAHM_ISSUE_REGISTER.json)

| ID | Severity | Subsystem | Affected File(s) | Status | Verification |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `ISSUE-001` | Critical | Integrity Verification & CLI | `src/main.rs`, `src/integration/input.rs` | Fixed and verified | `stage07_audit_regressions::test_issue_001_stored_baseline_detects_post_install_modification` |
| `ISSUE-002` | High | Scanner & Hasher | `src/scanner/engine.rs`, `src/analysis/hasher.rs` | Fixed and verified | `stage07_audit_regressions::test_issue_002_hasher_skips_symlinks` |
| `ISSUE-003` | High | Installer & Monitoring Telemetry | `src/integration/installer.rs`, `src/main.rs`, `src/cli/mod.rs` | Fixed and verified | `stage07_audit_regressions::test_issue_003_install_auto_persists_lifecycle_events_in_sqlite` |
| `ISSUE-004` | Medium | CLI State Directory Alignment | `src/cli/mod.rs`, `src/main.rs` | Documented known workflow convention | `tests/stage04_deception.rs`, `tests/stage05_integration.rs` |
| `ISSUE-005` | Medium | Stage 02 Code Intelligence | `src/structure/datamodels.rs`, `src/structure/routes.rs` | Fixed and verified | `stage07_audit_regressions::test_issue_005_schema_sql_tables_and_middleware_express_handler` |
| `ISSUE-006` | Medium | Monitoring HTTP API & CORS | `src/monitoring/api.rs` | Fixed and verified | `stage07_audit_regressions::test_issue_006_monitoring_api_endpoints_and_localhost_cors` |
| `ISSUE-007` | Low | Decoy Artifact Cleanup | `src/deception/decoy/registry.rs` | Fixed and verified | `stage07_audit_regressions::test_issue_007_cleanup_prunes_empty_decoy_subdirs` |
| `ISSUE-008` | Low | Documentation & CLI Apply UX | `README.md`, `src/cli/mod.rs`, `src/integration/installer.rs` | Fixed and verified (GUI omitted per owner) | `stage07_audit_regressions::test_apply_one_command_workflow_and_dry_run` + `README.md` |
| `ISSUE-009` | Informational | Windows Path Formatting | `src/analysis/aggregator.rs`, `src/deception/mirror.rs` | Documented known platform behavior | `tests/milestone07_stage02_validation.rs` |

---

## Detailed Issue Entries

### ISSUE-001 — Stored Integrity Baseline Bypass in CLI Integrity Check `[Critical]`
- **Affected Files:** `src/main.rs`, `src/integration/input.rs`
- **Root Cause:** `brahm integrity check` and `brahm reports generate` captured a live baseline from the working repository right before checking integrity against it, making post-install modifications invisible. Idempotent re-connection could also overwrite baseline metadata.
- **Impact:** Tampering after `connect` or `protection install` was not detected from CLI workflows.
- **Fix Applied:** Persisted `BRAHM_BASELINE.json` (alongside `BRAHM_ARCHITECTURE.json`, `BRAHM_DEPENDENCY_GRAPH.json`, and `BRAHM_SYMBOL_INDEX.json`) on first `connect`, preserved `baseline_fingerprint` across reconnects, and updated CLI handlers to compare live state against `BRAHM_BASELINE.json`.
- **Verification:** `test_issue_001_stored_baseline_detects_post_install_modification` (PASS).

### ISSUE-002 — Symlink Traversal Risk When `follow_symlinks = false` `[High]`
- **Affected Files:** `src/scanner/engine.rs`, `src/analysis/hasher.rs`
- **Root Cause:** File symlinks still reached `inspect_file` and `File::open` during hashing when `follow_symlinks = false`; broken targets when `follow_symlinks = true` were not cleanly skipped.
- **Impact:** Symlinks targeting files outside the repository boundary could be opened/hashed.
- **Fix Applied:** Excluded symlinks before `inspect_file` when `follow_symlinks = false`, canonicalized and enforced repository-root boundary checks when `follow_symlinks = true`, and skipped `FileType::Symlink` inside `fingerprint_files`.
- **Verification:** `test_issue_002_hasher_skips_symlinks` (PASS).

### ISSUE-003 — Missing Auto-Telemetry Persistence on Install/Stop & Missing `agent run` CLI `[High]`
- **Affected Files:** `src/integration/installer.rs`, `src/main.rs`, `src/cli/mod.rs`
- **Root Cause:** `protection install`, `deception stop/cleanup`, and local inspection adapters did not automatically emit correlated events into `brahm_monitoring.db` via CLI flows.
- **Impact:** Operators running CLI-only protection workflows lacked persisted SQLite audit trails without writing Rust harness code.
- **Fix Applied:** Wired `ProtectionInstaller::install` and `Stop`/`Cleanup` CLI handlers to persist `DecoyCreated`, `InspectionSessionStarted`, `InspectionSessionEnded`, decoy references, and session metadata into `brahm_monitoring.db`, and implemented `brahm agent run <session_id> --state-dir <dir> [--paths ...]`.
- **Verification:** `test_issue_003_install_auto_persists_lifecycle_events_in_sqlite` (PASS).

### ISSUE-004 — Standalone Subcommand vs. Integrated State Directory Defaults `[Medium]`
- **Affected Files:** `src/cli/mod.rs`, `src/main.rs`
- **Root Cause:** Standalone `decoy` and `deception` subcommands default to `.brahm_decoys` and `.brahm_sessions` instead of `.brahm_state/{decoys,sessions}`.
- **Impact:** Manual subcommand chaining requires explicit flag alignment (`--state-dir`, `--decoy-dir`, `--session-dir`).
- **Status:** Preserved backward compatibility; convention documented in CLI reference and Phase 2 validation workflow.

### ISSUE-005 — `schema.sql` Early Return & Express Middleware Route Handler Extraction `[Medium]`
- **Affected Files:** `src/structure/datamodels.rs`, `src/structure/routes.rs`
- **Root Cause:** SQL files under `db/` returned early as a single `SqlMigration`; JS/TS route calls extracted the first identifier (`requireAuth`) rather than the trailing handler.
- **Impact:** Schema tables in `db/schema.sql` and final handlers behind Express middleware were missed or misattributed.
- **Fix Applied:** Preserved `/migration` handling while parsing all `CREATE TABLE` definitions in non-migration SQL files as `SchemaDefinition`, and extracted `args.named_child(named_count - 1)` for JS/TS route handlers.
- **Verification:** `test_issue_005_schema_sql_tables_and_middleware_express_handler` (PASS).

### ISSUE-006 — Missing Dashboard API Routes & Localhost CORS Policy `[Medium]`
- **Affected Files:** `src/monitoring/api.rs`
- **Root Cause:** Axum server lacked `/api/v1/{architecture,deception,rating,experiment}` routes and localhost CORS headers.
- **Impact:** React TypeScript dashboard could not query architecture, deception policy, BRAHM rating, or experiment scorecard payloads from `localhost`.
- **Fix Applied:** Added authenticated GET routes and strict numeric-port `is_localhost_origin` CORS middleware (`http://localhost[:port]`, `http://127.0.0.1[:port]`).
- **Verification:** `test_issue_006_monitoring_api_endpoints_and_localhost_cors` (PASS).

### ISSUE-007 — Empty Decoy Subdirectories Left After Cleanup `[Low]`
- **Affected Files:** `src/deception/decoy/registry.rs`
- **Root Cause:** `cleanup_owned_artifacts` unlinked decoy files without pruning empty child directories under `decoy_layer_root`.
- **Impact:** Empty `canary/`, `docs/`, etc. subdirectories remained on disk after cleanup.
- **Fix Applied:** Pruned empty descendant directories inside `decoy_layer_root` during non-dry-run cleanup.
- **Verification:** `test_issue_007_cleanup_prunes_empty_decoy_subdirs` (PASS).

### ISSUE-008 — Stage 07 Public README Overhaul & Single-Command Apply UX `[Low]`
- **Affected Files:** `README.md`, `src/cli/mod.rs`, `src/integration/installer.rs`
- **Root Cause:** Public root documentation belonged to Phase 9 after rating/experiment evidence compiled, and operators needed a single-command `brahm apply <path>` entrypoint for loading a user project and applying anti-reverse-engineering protection directly via CLI (React `dashboard/` UI intentionally omitted per owner directive).
- **Fix Applied:** Implemented `brahm apply` (`brahm protect`) + `ProtectionInstaller::apply` + `verify_state`, and completed the full public `README.md` overhaul with SVG branding badge, empirical ratings, and coding-agent boundary rules (React web UI omitted per owner directive).
- **Verification:** `test_apply_one_command_workflow_and_dry_run` + `scripts/reproduce_experiment.ps1` (PASS).

### ISSUE-009 — Windows UNC Extended Path Prefix (`\\?\`) in Display Output `[Informational]`
- **Affected Files:** `src/analysis/aggregator.rs`, `src/deception/mirror.rs`
- **Root Cause:** Windows `std::fs::canonicalize` includes extended verbatim prefix `\\?\`.
- **Impact:** Cosmetic prefix in Windows absolute path prints; normalized relative `/` paths are unaffected.
- **Status:** Documented known platform behavior.
