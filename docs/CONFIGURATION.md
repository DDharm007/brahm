# BRAHM Configuration Reference (`brahm.toml`)

`BrahmConfig` (`src/config/mod.rs`) loads TOML configuration deterministically with `#[serde(default)]` so Stage 01 `brahm.toml` files remain valid across upgrades. Updates via `BrahmConfig::backup_and_save` create `brahm.toml.bak` automatically before writing.

## Full `brahm.toml` Example

```toml
schema_version = "2.0"

[scanner]
follow_symlinks = false
max_file_size_mb = 100
calculate_hashes = true

[scanner.exclusions]
directories = [".git", "node_modules", "target", "dist", "build", ".next", "__pycache__", ".venv", "venv"]
files = [".DS_Store", "Thumbs.db"]
extensions = ["tmp", "swp", "bak"]

[decoy]
profile = "standard" # disabled | conservative | standard | research
staging_dir = ".brahm/decoys"
exclude_from_production_builds = true

[deception]
default_policy_profile = "standard" # conservative | standard | research
workspace_root = ".brahm_sessions"
session_ttl_seconds = 3600

[integration]
protection_profile = "analysis_only" # analysis_only | research_mode | controlled_inspection | monitoring_mode
authorized_agent = "brahm-local-test-agent"

[monitoring]
db_path = ".brahm_state/brahm_monitoring.db"
api_bind_addr = "127.0.0.1:8787"
enable_api = false
allow_remote_api = false
rate_limit_per_minute = 120

[monitoring.scoring_weights]
repeated_canary_access = 20
synthetic_artifact_modified = 35
sandbox_boundary_violation = 40
expired_session_activity = 25
genuine_integrity_drift = 45
```

## Credentials & Secrets Handling
- Never place API tokens or credentials in `brahm.toml`.
- Set `BRAHM_MONITOR_TOKEN` in OS environment variables when launching `brahm monitor start`.
- GitHub authentication relies on existing OS `git` credential helpers with `GIT_TERMINAL_PROMPT=0` when cloning private repositories.
