# BRAHM Stage 05 — Agent Integration & Hybrid Protection (`src/integration/`)

## 1. Objective

Provides authorized local and GitHub repository onboarding, vendor-agnostic inspection agent integration contracts, explicit approval protection installation, visible inspection guides, and safe rollback.

---

## 2. Subsystems

| Module | File | Responsibility |
|---|---|---|
| **Models** | `src/integration/models.rs` | `ProtectionProfile`, `RepositoryRecord`, `AgentCapabilities`, `AgentSession`, `AgentResult`, `ProtectionPlan`, `RollbackReport` |
| **Input System** | `src/integration/input.rs` | `RepositoryInputManager` (`connect_local`, `validate_github_url`, `git_clone_readonly` via `git` argv without shell) |
| **Agent Interface** | `src/integration/agent.rs` | Vendor-agnostic `InspectionAgent` trait + built-in `LocalTestAgentAdapter` |
| **Docs Generator** | `src/integration/docs_gen.rs` | Transparent `BRAHM_INSPECTION_GUIDE.md` and `BRAHM_SECURITY.md` |
| **Installer** | `src/integration/installer.rs` | `ProtectionInstaller` implementing `plan`, explicit-approval `install`, `validate_installation`, and `rollback` |

---

## 3. Protection Profiles (`ProtectionProfile`)

- `AnalysisOnly`: Stage 01 + Stage 02 + genuine baseline fingerprint only.
- `ResearchMode`: Generates Research-profile decoys + prepares isolated inspection mirror & canaries.
- `ControlledInspection`: Standard-profile decoys + inspection mirror + `LocalTestAgentAdapter` registration.
- `MonitoringMode`: Integrity monitoring and canary telemetry over existing configuration.

---

## 4. Safe Installation & Rollback

1. Running `ProtectionInstaller::install` with `approved = false` writes only `BRAHM_PROTECTION_PLAN.json` and rejects mutation (`BrahmError::Unauthorized`).
2. Approved installation backs up existing `brahm.toml` to `brahm.toml.bak` via `BrahmConfig::backup_and_save` before writing updated state.
3. Rollback verifies BLAKE3 ownership before deleting BRAHM decoys, cleans session workspaces, restores `brahm.toml.bak`, verifies genuine baseline post-rollback, preserves unknown user files, and supports `--dry-run`.
4. **GitHub Credential Safety:** Plaintext tokens embedded in URLs (`https://token@github.com/...`) and CLI token flags are strictly rejected.
