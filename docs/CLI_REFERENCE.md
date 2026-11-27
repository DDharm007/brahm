# BRAHM CLI Command Reference (Stages 01–06)

All commands follow `brahm [--verbose] <subcommand>`.

## Global Flags
- `--help`, `-h`: Display command help tree.
- `--version`, `-V`: Print engine version.
- `--verbose`, `-v`: Enable `brahm=debug,info` structured logs.

## Stage 01 & 02 — Repository & Architecture Intelligence
- `brahm init [path] [--force]` — Initialize `brahm.toml`.
- `brahm scan <repository> [--format terminal|json|markdown|all] [--output <dir>] [--exclude <pattern>] [--config <toml>] [--no-hash] [--follow-symlinks] [--arch|--deep]`
- `brahm arch <repository> ...` / `brahm architecture analyze <repository> ...` — Execute Stage 01 + Stage 02 deep code intelligence and emit `BRAHM_ARCHITECTURE.*`, `BRAHM_DEPENDENCY_GRAPH.json`, and `BRAHM_SYMBOL_INDEX.json`.

## Stage 03 — Decoy Generation Engine
- `brahm decoy generate <repository> [--output <dir>] [--profile disabled|conservative|standard|research]`
- `brahm decoy list [--registry <json>]`
- `brahm decoy inspect <artifact-id> [--registry <json>]`
- `brahm decoy validate [--registry <json>]`
- `brahm decoy remove [--registry <json>] [--dry-run]`

## Stage 04 — Repository Deception Engine
- `brahm deception prepare <repository> [--decoy-dir <dir>] [--output <dir>] [--profile conservative|standard|research] [--ttl <secs>]`
- `brahm deception policy list`
- `brahm deception policy validate <policy-file>`
- `brahm deception session create <repository> ...`
- `brahm deception session inspect <session-id> [--sessions-dir <dir>]`
- `brahm deception session stop <session-id> [--sessions-dir <dir>]`
- `brahm deception cleanup <session-id> [--sessions-dir <dir>]`

## Stage 05 — Agent Integration & Protection Management
- `brahm connect local <path> [--state-dir <dir>]`
- `brahm connect github <url> [--dest <dir>] [--state-dir <dir>]` *(never pass tokens via CLI or URL)*
- `brahm protection plan <repository> [--state-dir <dir>] [--profile analysis-only|research-mode|controlled-inspection|monitoring-mode]`
- `brahm protection validate [--state-dir <dir>]`
- `brahm protection install <repository> [--state-dir <dir>] [--profile ...] --approve`
- `brahm protection status [--state-dir <dir>]`
- `brahm protection rollback [--state-dir <dir>] [--dry-run]`

## Stage 06 — Monitoring, Integrity & Security Intelligence
- `brahm monitor start [--state-dir <dir>] [--bind 127.0.0.1:8787] [--allow-remote]` *(requires `BRAHM_MONITOR_TOKEN` env var of $\ge 16$ chars)*
- `brahm monitor status [--state-dir <dir>]`
- `brahm monitor stop [--state-dir <dir>]`
- `brahm events list [--db <sqlite>] [--repository-id <id>] [--session-id <id>] [--limit <n>]`
- `brahm events inspect <event-id> [--db <sqlite>]`
- `brahm sessions timeline <session-id> [--state-dir <dir>]`
- `brahm integrity check [--state-dir <dir>]`
- `brahm reports generate [--state-dir <dir>] [--output <dir>]`
