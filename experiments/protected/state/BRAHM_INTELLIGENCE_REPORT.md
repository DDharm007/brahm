# BRAHM Stage 06 — Security Monitoring & Intelligence Report

> Generated deterministically by **BRAHM Monitoring & Intelligence Engine**

## 1. Repository Identity

- **Name:** `brahm-commerce-demo`
- **Repository ID:** `repo_b7153cf65c6656d9`

## 2. Monitoring Period

- **Start:** `2026-11-27T06:37:56.685643400+00:00`
- **End:** `2026-11-27T06:42:06.072000400+00:00`

## 3. Total Events

- **Total Collected Events:** `23`
- **Aggregate Rule Risk Score:** `20`

## 4. Canary Events

- **Canary Trigger Events Observed:** `11`

## 5. Decoy Interactions

- **Total Decoy & Synthetic Interactions:** `11`

## 6. Session History

- **Recorded Inspection Sessions:** `1`

### Session `sess_bd85ce628669` (12 events, 11 canary hits)
- `2026-11-27T06:37:56.786719200+00:00` | `[informational]` inspection_session_started
- `2026-11-27T06:42:06.067047400+00:00` | `[medium]` canary_accessed [decoy_18c46ced59d40a22] (canary/BRAHM_CANARY_ANCHOR.md)
- `2026-11-27T06:42:06.067401900+00:00` | `[medium]` canary_accessed [decoy_75b673cbdb07ad53] (canary/inspection_canary_token.json)
- `2026-11-27T06:42:06.067549200+00:00` | `[medium]` canary_accessed [decoy_54dd6c0bd4fdf907] (config/api.decoy.json)
- `2026-11-27T06:42:06.067665300+00:00` | `[medium]` canary_accessed [decoy_325c74c5d81d5c05] (config/service.decoy.yaml)
- `2026-11-27T06:42:06.068021900+00:00` | `[medium]` canary_accessed [decoy_f14e3db038406867] (db/schema.decoy.sql)
- `2026-11-27T06:42:06.068287800+00:00` | `[medium]` canary_accessed [decoy_e82c958168b620b7] (docs/COMPONENT_OVERVIEW.md)
- `2026-11-27T06:42:06.068441500+00:00` | `[medium]` canary_accessed [decoy_d9f0b7c160f4d297] (docs/INTERNAL_DESIGN.md)
- `2026-11-27T06:42:06.068551900+00:00` | `[medium]` canary_accessed [decoy_afd0e437f6eb4c60] (docs/SERVICE_DEPENDENCIES.md)
- `2026-11-27T06:42:06.068680+00:00` | `[medium]` canary_accessed [decoy_677ffff5a968188f] (docs/SYSTEM_ARCHITECTURE.md)
- `2026-11-27T06:42:06.068794300+00:00` | `[medium]` canary_accessed [decoy_811112c2221985c8] (manifests/synthetic_dependencies.decoy.json)
- `2026-11-27T06:42:06.072000400+00:00` | `[medium]` canary_accessed [decoy_c34470c51dc68c6d] (src/syntheticTelemetryBridge.ts)

### Session `unscoped_global` (0 events, 0 canary hits)

## 7. Integrity Status

- **Genuine Baseline Matches:** `true`
- **Staged Decoy Integrity Valid:** `true`
- **Unexpected Drift Items:** `0`

## 8. Suspicious Activity Classifications

- **Repeated Registered Canary Access** (`R01_REPEATED_CANARY_ACCESS` | Severity: `high` | Confidence: `85%` | Weight: `20`)

## 9. Rule-Based Evidence

- `[R01_REPEATED_CANARY_ACCESS]` Observed 11 distinct registered canary marker interactions
  - _Note:_ Actor attribution remains explicitly uncertain: telemetry observes interaction with a registered artifact inside the instrumented boundary and does not independently identify human vs. AI agent identity.

## 10. Known Limitations

- BRAHM cannot claim prevention of reverse engineering or copying from observed decoy hits alone.
- Canary interactions establish observed artifact access inside the telemetry boundary; actor attribution (human vs. AI model) remains uncertain without external corroboration.
- File integrity checks validate cryptographic BLAKE3 match against baseline snapshots, not runtime semantic safety.

## 11. Recommended Defensive Actions

- Review supporting rule evidence and correlate session timelines before taking defensive action.

