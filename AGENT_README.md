# Using BRAHM with AI Coding Agents
### Integration Guide for Google Antigravity, OpenAI Codex CLI, Claude Code, Cursor & Custom Agents

---

## Why Put BRAHM Between Your Source Code and AI Coding Agents?

When delegating repository analysis, code review, or third-party audits to AI coding assistants or autonomous CLI agents:

1. **Secrets stay out of context:** Sensitive credential files (`.env`, `.pem`, `.key`, `id_rsa`, `credentials.json`, etc.) are automatically excluded from hashing/parsing and stripped when BRAHM builds an inspection workspace.
2. **Production code is never mutated:** Your original project folder remains strictly read-only and sealed with a cryptographic `BLAKE3` baseline (`BRAHM_BASELINE.json`).
3. **Controlled deceptive surface & canary telemetry:** BRAHM stages isolated synthetic/canary artifacts inside a separate workspace mirror and records telemetry in SQLite (`brahm_monitoring.db`) whenever registered artifacts or traversal attempts occur.

> [!IMPORTANT]
> **Honest Security Boundary:** BRAHM enforces workspace isolation, secret stripping, cryptographic baseline tamper checks, and canary telemetry. It does **not** prevent a white-box coding agent from understanding plaintext genuine files that you choose to include inside an inspection workspace, and canary hits establish boundary access telemetry rather than proving external model or human identity.

---

## Step 0: Install BRAHM (No Rust Required) & Load Your Project

### Without Rust Installed (Recommended for End Users)
- **Windows:** `irm https://raw.githubusercontent.com/DDharm007/brahm/main/scripts/install.ps1 | iex` (or pass `-LocalBinary .\brahm-windows-x86_64.exe` from [Releases](https://github.com/DDharm007/brahm/releases))
- **Linux / macOS:** `curl -fsSL https://raw.githubusercontent.com/DDharm007/brahm/main/scripts/install.sh | bash`
- **Docker:** `docker build -t brahm https://github.com/DDharm007/brahm.git`
- **From Source (if Rust >= 1.80 is present):** `git clone https://github.com/DDharm007/brahm.git && cd brahm && cargo build --release`

Now choose how much code visibility to allow inside the isolated inspection workspace (your original folder stays 100% untouched):

```powershell
# Recommended Anti-Reverse-Engineering Mode (`--shield` / `--profile blind-shield`):
# Strips secrets AND deterministically redacts function/method bodies, SQL DDL schemas,
# and internal architecture docs strictly inside the isolated inspection mirror
brahm apply C:\path\to\your-project --state-dir C:\path\to\.brahm_state --shield --json

# Full-source audit mode (plain source copy + secret stripping + canary telemetry):
brahm apply C:\path\to\your-project --state-dir C:\path\to\.brahm_state --profile controlled-inspection --json
```

Inspect the JSON output on `stdout` (diagnostic logs emit on `stderr`):
- **`session_id`**: active session identifier (for example `sess_bd85ce628669`)
- **`inspection_workspace`**: **the isolated mirror directory path to give your AI agent**
- **`baseline_fingerprint`**: cryptographic `BLAKE3` digest of the genuine unmodified repository
- **`decoy_count` / `canary_count`**: staged synthetic overlays inside the isolated mirror

---

## 1. Google Antigravity

You can isolate Antigravity in two ways:

### Option A — Open the Protected Inspection Workspace Directly (Recommended)
1. Run `brahm apply` on your real repo (storing state outside the repo, e.g. `..\.brahm_state`).
2. Open **only** the printed `inspection_workspace` path (`..\.brahm_state\sessions\<session_id>\workspace`) as your Antigravity workspace folder instead of opening your genuine repository root.
3. Ask Antigravity to inspect, document, or audit the workspace cleanly; your `.env` secrets are already stripped and synthetic canary overlays are staged.

### Option B — Workspace Rule (`AGENTS.md` or Antigravity User Rule)
Add the rule below to your orchestration workspace `AGENTS.md` (or Antigravity custom rules):

```markdown
# BRAHM Inspection Boundary (Antigravity Rule)
- Never inspect, read, grep, or write inside raw unmonitored repository directories (`<GENUINE_REPO_PATH>`).
- Generate or refresh the isolated workspace first via:
  `brahm apply <GENUINE_REPO_PATH> --state-dir <STATE_DIR> --profile controlled-inspection --json`
- Restrict all read/search/tool calls exclusively to the returned `inspection_workspace` path.
- Never copy or commit `<STATE_DIR>` into genuine production builds or Git history.
```

---

## 2. OpenAI Codex CLI

Point Codex explicitly at `inspection_workspace` via `--cd` so Codex executes strictly inside BRAHM's isolated sandbox directory:

### PowerShell Launcher Snippet
```powershell
$stateDir = "C:\work\.brahm_state"
$applied = (& "C:\path\to\brahm.exe" apply "C:\work\my-app" --state-dir $stateDir --profile controlled-inspection --json) | ConvertFrom-Json

# Launch Codex restricted to the BRAHM inspection workspace
codex --cd $applied.inspection_workspace "Analyze the repository architecture and summarize API endpoints."

# Verify real repo integrity afterward
& "C:\path\to\brahm.exe" integrity check --state-dir $stateDir
```

### Codex `AGENTS.md` Snippet
Place in an orchestration parent directory or system instructions:

```markdown
## Codex Repository Policy (BRAHM)
- Target ONLY the BRAHM `inspection_workspace` returned by `brahm apply <REPO> --json`.
- Do NOT navigate parent directories to access `BRAHM_BASELINE.json`, `BRAHM_DECOY_REGISTRY.json`, or genuine `.env` files.
- Return unified diffs or markdown reports rather than mutating original source paths directly.
```

---

## 3. Claude Code

Launch Claude Code from inside the `inspection_workspace` directory and **never** pass the genuine repository root or `.brahm_state` parent via `--add-dir`:

### Shell Launch Workflow
```bash
# Linux/macOS/Git Bash example
SESSION_JSON=$(./target/release/brahm apply ./my-app --state-dir ../.brahm_state --profile controlled-inspection --json)
WORKSPACE=$(echo "$SESSION_JSON" | jq -r '.inspection_workspace')

(cd "$WORKSPACE" && claude)

# Post-session integrity check & cleanup
./target/release/brahm integrity check --state-dir ../.brahm_state
./target/release/brahm reports generate --state-dir ../.brahm_state
```

### `CLAUDE.md` Safety Block
```markdown
# Claude Code — BRAHM Controlled Inspection Contract
1. Operate strictly within the active BRAHM `inspection_workspace`.
2. Do not read `../`, `BRAHM_DECOY_REGISTRY.json`, `BRAHM_INSPECTION_MANIFEST.json`, or `.brahm_state/`.
3. Treat any requested changes as reviewed patches validated against the sealed baseline before applying upstream.
```

---

## 4. Cursor, Windsurf & Any Other AI Agent or Script

Any IDE agent, local LLM tool runner, or custom CI reviewer works cleanly with BRAHM using the exact same contract:

1. **Workspace Binding:** Open or chroot your agent strictly to `inspection_workspace` (`<STATE_DIR>/sessions/<SESSION_ID>/workspace`).
2. **Adapter Telemetry Ingestion (`brahm agent run`):** If your runner logs accessed relative paths (or uses BRAHM's built-in `LocalTestAgentAdapter`), feed accessed paths into BRAHM telemetry to evaluate canaries, traversal blocks, and timeline correlation in `brahm_monitoring.db`:
   ```powershell
   brahm agent run <SESSION_ID> --state-dir <STATE_DIR> --paths package.json src/app.ts canary/BRAHM_CANARY_ANCHOR.md
   ```
3. **Rule File (`.cursorrules` / `.windsurfrules` / System Prompt):**
   ```text
   You are operating inside a BRAHM controlled inspection workspace.
   Only read files within the current workspace root.
   Do not inspect parent directories, state manifests, or external credential paths.
   ```
4. **Optional Localhost Telemetry API (For Tooling Integration):**
   ```powershell
   $env:BRAHM_MONITOR_TOKEN = "replace-with-32-char-local-operator-token"
   brahm monitor start --state-dir <STATE_DIR> --bind 127.0.0.1:8787
   ```
   Authenticated Bearer endpoints on `127.0.0.1`: `/api/v1/health` (public), `/api/v1/repositories`, `/api/v1/architecture`, `/api/v1/deception`, `/api/v1/events`, `/api/v1/sessions`, `/api/v1/decoys`, `/api/v1/integrity`, `/api/v1/reports`, `/api/v1/rating`, and `/api/v1/experiment`.

---

## 5. Post-Session Verification, Intelligence Reports & Cleanup

After **any** agent finishes inspecting your codebase:

```powershell
# 1. Inspect recorded SQLite security/canary events & session timeline
brahm events list --db .\.brahm_state\brahm_monitoring.db --limit 50
brahm sessions timeline <SESSION_ID> --state-dir .\.brahm_state

# 2. Verify genuine repository files remain 100% unmodified against sealed BLAKE3 baseline
brahm integrity check --state-dir .\.brahm_state

# 3. Export complete JSON + Markdown security intelligence reports
brahm reports generate --state-dir .\.brahm_state --output .\brahm_reports

# 4. Safely remove all BRAHM-owned decoys & isolated mirrors (original repo untouched)
brahm protection rollback --state-dir .\.brahm_state
```

---

## Troubleshooting Agent Workflows

- **Agent gets `Invalid JSON` when parsing `brahm apply --json`:** Redirect or ignore `stderr` rather than merging `2>&1`; `brahm` writes clean JSON to `stdout` and diagnostic `INFO` logs to `stderr`.
- **Agent tries `../` path escape:** `ControlledInspectionEnvironment` rejects unsafe traversal outside the workspace boundary and records `workspace_access_denied` telemetry (`blocked_attempts`).
- **Missing `.env` variables inside mirror:** Intentional — `.env`, keys, and certificates are stripped during workspace mirroring by `RepositoryMirror` (`ProductionIntegrityGuard`). Keep `.env.example` non-secret templates for safe structural analysis.
- **Need full command flags:** See [`docs/CLI_REFERENCE.md`](docs/CLI_REFERENCE.md) and [`README.md`](README.md).
