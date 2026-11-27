# BRAHM Troubleshooting Guide

## Common Errors & Deterministic Remedies

| Symptom / Error | Root Cause | Resolution |
|---|---|---|
| `Unauthorized operation: BRAHM_MONITOR_TOKEN environment variable (min 16 chars) is required` | `brahm monitor start` invoked without Bearer token configured | Set `BRAHM_MONITOR_TOKEN` ($\ge 16$ characters) in your shell environment before starting the API. |
| `Unauthorized operation: refusing non-loopback bind ... without explicit allow_remote=true` | Attempted to bind HTTP API to `0.0.0.0` or external IP without flag | Bind to `127.0.0.1:8787` (default) or explicitly pass `--allow-remote` if authorized. |
| `Unauthorized operation: Installation blocked: explicit approval is required` | `brahm protection install` run without `--approve` | Inspect the generated `BRAHM_PROTECTION_PLAN.json` and re-run with `--approve`. |
| `Security error: do not pass plaintext credentials/tokens inside repository URLs` | `connect github` given `https://token@github.com/...` | Pass clean `https://github.com/<owner>/<repo>` and configure system `git` credentials cleanly. |
| `Production integrity guard blocked session creation` | A synthetic decoy path or `BRAHM-SYNTHETIC-ARTIFACT` marker was found inside genuine repository files | Inspect `BRAHM_INTEGRITY_REPORT.json` findings and remove stray decoy copies from the genuine directory. |
| `Session '...' TTL exceeded` / `Expired` | Inspection session exceeded configured `--ttl` seconds | Create a fresh inspection session via `brahm deception session create` or increase `--ttl`. |
| Build failure on Windows MinGW / GNU toolchain | GCC or cargo missing from current PATH | Verify `rustup show` active toolchain and run `cargo test` inside Developer Mode / standard terminal. |
