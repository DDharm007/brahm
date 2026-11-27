# BRAHM Stage 07 — End-to-End & Failure-Path Validation Report

**Validation Date:** 2026-11-27  
**Executable Verified:** `target/release/brahm.exe` (`brahm 0.1.0`, commit after `d8f4db4` + stdout/stderr & `--paths` fixes)  
**Test Environment:** Isolated temporary workspace (`brahm_phase2_bdfd21e2`) containing a representative TypeScript/Express/SQLite API (`package.json`, `src/server.ts`, `db/schema.sql`) and `.env` credential fixture.  
**Test Suite Gate:** `cargo test` passed **134 / 134** tests (21 unit + 113 integration across 16 binaries); `cargo clippy --all-targets --all-features -- -D warnings` passed with 0 warnings.

---

## 1. Twenty-Step End-to-End CLI Pipeline Validation

All 20 end-to-end steps completed with **Exit Code 0**:

| Step | Operation | Measured Time | Exit | Verified Observed Outcome |
| :--- | :--- | ---: | ---: | :--- |
| `E2E-01` | `brahm --version` | 1242 ms | 0 | Printed `brahm 0.1.0`. |
| `E2E-02` | `brahm init <repo>` | 211 ms | 0 | Created `brahm.toml` cleanly inside sample target. |
| `E2E-03` | `brahm scan <repo> --format all` | 159 ms | 0 | Detected 5 files, 3 dirs, Node.js (`package.json` confirmed), JSON + Markdown reports generated. |
| `E2E-04` | `brahm arch <repo> --format all` | 178 ms | 0 | Classified `REST API / Backend Service (High)`, parsed AST, 1 entry point, `GET /api/items`, 1 SQL table, DAG graph (5 nodes, 4 edges), exported JSON/MD artifacts. |
| `E2E-05` | `brahm apply <repo> --dry-run` | 154 ms | 0 | Previewed protection plan without writing any persistent disk artifacts to target state dir. |
| `E2E-06` | `brahm decoy generate --profile research` | 170 ms | 0 | Generated 11 staged synthetic decoys across `canary/`, `docs/`, `manifests/`, `src/`, `config/`, `db/`. |
| `E2E-07` | `brahm decoy list` | 161 ms | 0 | Listed all 11 registered decoys with IDs, types, paths, and `state=Staged`. |
| `E2E-08` | `brahm decoy validate` | 154 ms | 0 | Verified BLAKE3 digests across all 11 staged decoy files (`[PASS]`). |
| `E2E-09` | `brahm deception policy list` | 159 ms | 0 | Enumerated `pol_conservative_v1`, `pol_standard_v1`, `pol_research_v1`. |
| `E2E-10` | `brahm deception prepare` | 188 ms | 0 | Created isolated workspace mirror `sess_a064f4adf49a` (`4 genuine, 10 synthetic, 1 secrets stripped` — `.env` stripped cleanly). |
| `E2E-11` | `brahm deception session inspect` | 264 ms | 0 | Returned full session JSON including baseline digest, mirror view digest, policy ID, and 10 injected artifacts. |
| `E2E-12` | `brahm connect local` | 242 ms | 0 | Registered `sample_repo` (`repo_0df62de597648c56`) and sealed `BRAHM_BASELINE.json` (`8ce65bda...`). |
| `E2E-13` | `brahm protection plan` | 168 ms | 0 | Drafted structured `plan_6f5cff8b3bc8` for `controlled_inspection` (`will_generate_decoys=true`, `requires_explicit_approval=true`). |
| `E2E-14` | `brahm apply <repo> --json` | 299 ms | 0 | Output clean JSON on stdout (logs separated cleanly on stderr): activated `sess_51c580f650a4`, staged 8 decoys (8 canaries), `integrity_verified=true`. |
| `E2E-15` | `brahm protection status` | 159 ms | 0 | Reported active profile `controlled_inspection`, linked decoy registry, session ID, and clean baseline verification (`true`). |
| `E2E-16` | `brahm agent run <sid> --paths ...` | 180 ms | 0 | Accessed `canary/BRAHM_CANARY_ANCHOR.md` (`BRAHM-CANARY-D5157663B736`) and `package.json`; persisted correlated `canary_accessed` event into SQLite. |
| `E2E-17` | `brahm events list` & `sessions timeline` | 322 ms | 0 | Returned 10 lifecycle + canary events from `brahm_monitoring.db` and correlated session timeline (`event_count=2`, `canary_access_count=1`). |
| `E2E-18` | `brahm integrity check` (clean -> tamper -> clean) | 376 ms | 0 | Clean check returned `baseline_matches=true`; live modification to `src/server.ts` immediately flagged `unexpected_change_count=1` (`BLAKE3` drift). |
| `E2E-19` | `brahm monitor status`, `reports generate` & Axum API | 1895 ms | 0 | DB reported 11 events, emitted all 5 intelligence deliverables, localhost HTTP server returned `200` on `/api/v1/health`, `401` on unauthenticated `/api/v1/events`, and `200` with bearer auth. |
| `E2E-20` | `session stop/cleanup`, `protection rollback`, `decoy remove` | 820 ms | 0 | Safely removed 8/11 BRAHM-owned decoys & cleaned mirror sessions with `skipped_unowned_files=[]` and zero impact to genuine source files. |

---

## 2. Thirteen Failure-Path & Security-Boundary Validations

Every invalid operation, unapproved mutation, credential violation, and traversal attempt was safely blocked or recorded without corrupting repository state:

| Case | Scenario Tested | Measured Time | Exit | Enforced Safety Behavior |
| :--- | :--- | ---: | ---: | :--- |
| `FAIL-01` | Scan nonexistent directory | 156 ms | 1 | Returned `Invalid path` error cleanly without creating artifacts. |
| `FAIL-02` | Scan regular file instead of directory | 164 ms | 1 | Rejected file target requiring directory root. |
| `FAIL-03` | `brahm init` overwrite without `--force` | 167 ms | 1 | Preserved existing `brahm.toml` and aborted safely. |
| `FAIL-04` | `connect github` with non-GitHub domain | 160 ms | 1 | Rejected non-`github.com` remote URL. |
| `FAIL-05` | `connect github` with embedded URL credentials | 161 ms | 1 | Blocked `https://user:token@github.com/...` plaintext secret injection (`Unauthorized operation`). |
| `FAIL-06` | `protection install` without `--approve` | 174 ms | 1 | Refused unapproved profile activation (`Installation blocked`), saving only preview plan. |
| `FAIL-07` | `deception policy validate` malformed JSON | 156 ms | 1 | Schema deserialization failed cleanly on missing `policy_id`. |
| `FAIL-08` | `agent run` with missing session ID | 161 ms | 1 | Rejected `sess_missing_999` (`Inspection session error`). |
| `FAIL-09` | `agent run` sandbox path traversal (`../../etc/passwd`) | 394 ms | 0 | Controlled sandbox denied traversal (`blocked_attempts: ["../../etc/passwd"]`, `workspace_access_denied`, `allowed=false`). |
| `FAIL-10` | `integrity check` on uninitialized `--state-dir` | 166 ms | 1 | Refused missing repository config / baseline cleanly. |
| `FAIL-11` | `monitor start` binding `0.0.0.0` without `--allow-remote` | 155 ms | 1 | Rejected non-loopback binding (`Security policy: refusing non-loopback bind`). |
| `FAIL-12` | `monitor start` without `BRAHM_MONITOR_TOKEN` | 151 ms | 1 | Rejected startup when required >=16-char auth token was absent. |
| `FAIL-13` | `decoy validate` after staged decoy deletion | 354 ms | 1 | Flagged missing staged file `[FAIL] decoy_8ea0eee04ac95465` while verifying intact artifacts. |

---

## 3. Issues Uncovered & Resolved During E2E Execution

Running the full compiled CLI caught two real CLI ergonomics defects that unit/library tests had not previously exercised:
1. `tracing_subscriber` previously wrote `INFO` diagnostics to `stdout` instead of `stderr`, mixing log text into `--json` stdout streams (`src/main.rs` fixed via `.with_writer(std::io::stderr)`).
2. `brahm agent run --paths` required repeating `--paths` before each path rather than accepting space-separated relative paths (`src/cli/mod.rs` fixed via `num_args = 1..`).

Both fixes were compiled, re-verified across all 134 automated tests, and validated live through `E2E-14`, `E2E-16`, and `E2E-17`.
