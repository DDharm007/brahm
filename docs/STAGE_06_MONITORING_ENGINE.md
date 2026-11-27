# BRAHM Stage 06 — Monitoring & Intelligence Engine (`src/monitoring/`)

## 1. Objective

Collects structured events from sandbox inspections and integrity checks, persists records into indexed SQLite storage, correlates session timelines, applies deterministic rule-based suspicious activity scoring, optionally serves an authenticated localhost HTTP API, and generates security intelligence reports.

---

## 2. Modules & Database Schema (`src/monitoring/storage.rs`)

`SqliteEventStore` (via bundled `rusqlite v0.40.2`, behind modular `EventStore` trait for future PostgreSQL extensibility) applies version 1 migrations creating:
- `schema_migrations(version, applied_at)`
- `monitoring_events(event_id PRIMARY KEY, event_type, repository_id, session_id, artifact_id, timestamp, source, severity, correlation_id, metadata_json)` with indexes on `repository_id`, `session_id`, `artifact_id`, `timestamp`, and `event_type`.
- `session_refs`, `decoy_refs`, `integrity_records`, and `monitoring_config`.

No raw repository source code bodies or credential values are ever stored in SQLite.

---

## 3. Detection, Correlation & Classification

- **Canary Detection (`src/monitoring/canary.rs`):** `CanaryDetector` matches sandbox accesses or log excerpts against `DecoyRegistry`, suppresses burst duplicates (`is_duplicate`), distinguishes `BenignValidatorScan` vs. `ConfirmedSignal`, and ignores unregistered markers cleanly.
- **Timeline Correlator (`src/monitoring/correlator.rs`):** Aggregates events chronologically per `session_id` into `SessionTimeline` (`BRAHM_SESSION_TIMELINE.json`).
- **Rule-Based Classifier (`src/monitoring/classifier.rs`):** Evaluates documented rules `R01_REPEATED_CANARY_ACCESS` ($\ge 2$ signals), `R02_SYNTHETIC_ARTIFACT_MODIFIED`, `R03_SANDBOX_BOUNDARY_ATTEMPT`, `R04_EXPIRED_SESSION_ACTIVITY`, and `R05_GENUINE_INTEGRITY_DRIFT` using configurable `ScoringWeights`. Never flags a single benign event as malicious and always attaches an explicit attribution uncertainty disclaimer.
- **Integrity Monitor (`src/monitoring/integrity.rs`):** Diffs live BLAKE3 digests against `GenuineLayer` and `DecoyRegistry`, distinguishing expected changes (`expected_change_paths`) from unexpected drift without silently deleting or modifying user files.

---

## 4. Optional Local Monitoring API (`src/monitoring/api.rs`)

Powered by `axum` (`serve_monitoring_api`):
- Default bind `127.0.0.1:8787`; non-loopback bind requires explicit `allow_remote = true`.
- Endpoints: `GET /api/v1/health`, `/api/v1/repositories`, `/api/v1/events`, `/api/v1/sessions`, `/api/v1/decoys`, `/api/v1/integrity`, `/api/v1/reports`.
- Intelligence endpoints require `Authorization: Bearer <BRAHM_MONITOR_TOKEN>` verified via constant-time byte comparison and guarded by a 60-second sliding-window rate limiter (`401`, `429`, `400` structured JSON errors).
