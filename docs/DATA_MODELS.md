# BRAHM Unified Data Models Reference

All models serialize deterministically with `serde` and `serde_json` and enforce versioned schemas across Stages 01–06.

## Core Cross-Stage Entities

| Entity | Crate Path | Stage | Description |
|---|---|---|---|
| `RepositoryReport` | `brahm::models` | 01 | Complete inventory, language stats, frameworks, BLAKE3 hashes |
| `ArchitectureReport` | `brahm::architecture` | 02 | Pattern, layers, entry points, API routes, ORM models, hotspots, facts vs inferences |
| `ArchitectureGraph` / `GraphExport` | `brahm::graph` | 02 | `petgraph` DiGraph & serializable vertex/edge topology |
| `Symbol` / `SymbolIndex` | `brahm::symbols` | 02 | Deterministic BLAKE3 symbol IDs, spans, visibility, signatures |
| `DecoyArtifact` / `DecoyTemplate` | `brahm::deception::decoy` | 03 | Generated synthetic file metadata, fingerprints, template specs |
| `DecoyRegistryData` / `DecoyManifest` | `brahm::deception::decoy` | 03 | Persistent registry & production isolation manifest |
| `DeceptionPolicy` / `PolicyEvaluationResult` | `brahm::deception` | 04 | Auditable mirror inclusion/exclusion policy rules |
| `InspectionSession` / `InspectionManifest` | `brahm::deception` | 04 | Isolated workspace lifecycle (`Created`..`Cleaned`) & view fingerprints |
| `RepositoryRecord` / `ProtectionPlan` | `brahm::integration` | 05 | Connected repository config & approval-gated installation plan |
| `AgentCapabilities` / `AgentSession` / `AgentResult` | `brahm::integration` | 05 | Vendor-agnostic `InspectionAgent` adapter contract & telemetry |
| `MonitoringEvent` / `CanaryEvent` | `brahm::monitoring` | 06 | Structured security event & deduplicated canary trigger record |
| `IntegrityRecord` / `SecurityFinding` | `brahm::monitoring` | 06 | BLAKE3 drift verification & rule-based security finding |
| `SessionTimeline` / `IntelligenceReport` | `brahm::monitoring` | 06 | Correlated session sequence & executive security intelligence report |
