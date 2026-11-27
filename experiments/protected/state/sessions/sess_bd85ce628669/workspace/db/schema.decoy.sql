-- BRAHM-SYNTHETIC-ARTIFACT: decoy_f14e3db038406867
-- Canary Marker: BRAHM-CANARY-3732F08AFC7B
-- Inert synthetic schema example (never executed against genuine DBs).

CREATE TABLE IF NOT EXISTS synthetic_inspection_audit (
    id INTEGER PRIMARY KEY,
    artifact_id TEXT NOT NULL,
    canary_marker TEXT NOT NULL
);
