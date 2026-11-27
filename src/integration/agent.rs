//! Vendor-agnostic Agent Integration Adapter Interface (Milestone 05.4).
//!
//! Defines the trait contract (`InspectionAgent`) and a local test adapter (`LocalTestAgentAdapter`)
//! operating strictly inside Stage 04 `ControlledInspectionEnvironment` sandboxes.

use chrono::Utc;
use uuid::Uuid;

use super::models::{AgentCapabilities, AgentResult, AgentSession, AgentTelemetryEvent};
use crate::deception::{ControlledInspectionEnvironment, InspectionSession};
use crate::errors::Result;

/// Generic adapter interface for authorized inspection coding agents.
pub trait InspectionAgent {
    /// Human-readable identifier of this agent adapter.
    fn name(&self) -> &str;

    /// Declare supported sandbox and telemetry capabilities.
    fn get_capabilities(&self) -> AgentCapabilities;

    /// Prepare an agent-specific session wrapper around a Stage 04 `InspectionSession`.
    fn prepare_session(&self, session: &InspectionSession) -> Result<AgentSession>;

    /// Execute controlled inspection over requested paths inside a bounded sandbox environment.
    fn run_inspection(
        &self,
        sandbox: &mut ControlledInspectionEnvironment,
        paths_to_inspect: &[String],
    ) -> Result<AgentResult>;
}

/// Built-in deterministic local test-agent adapter for verification & controlled studies.
#[derive(Debug, Default, Clone)]
pub struct LocalTestAgentAdapter {
    adapter_name: String,
}

impl LocalTestAgentAdapter {
    pub fn new() -> Self {
        Self {
            adapter_name: "brahm-local-test-agent".to_string(),
        }
    }

    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            adapter_name: name.into(),
        }
    }
}

impl InspectionAgent for LocalTestAgentAdapter {
    fn name(&self) -> &str {
        &self.adapter_name
    }

    fn get_capabilities(&self) -> AgentCapabilities {
        AgentCapabilities {
            can_read_workspace: true,
            can_list_files: true,
            supports_structured_telemetry: true,
            requires_isolated_sandbox: true,
            supported_environments: vec!["local_sandbox".to_string()],
        }
    }

    fn prepare_session(&self, session: &InspectionSession) -> Result<AgentSession> {
        let agent_session_id = format!("agent_{}", &Uuid::new_v4().simple().to_string()[..12]);
        let guide = session.state_dir.join("BRAHM_INSPECTION_GUIDE.md");
        Ok(AgentSession {
            agent_session_id,
            agent_name: self.adapter_name.clone(),
            inspection_session_id: session.session_id.clone(),
            workspace_path: session.workspace_path.clone(),
            prepared_at: Utc::now(),
            instructions_path: if guide.is_file() { Some(guide) } else { None },
        })
    }

    fn run_inspection(
        &self,
        sandbox: &mut ControlledInspectionEnvironment,
        paths_to_inspect: &[String],
    ) -> Result<AgentResult> {
        let agent_session = self.prepare_session(sandbox.session())?;
        let mut accessed_files = Vec::new();
        let mut blocked_attempts = Vec::new();
        let mut notes = Vec::new();

        // If no explicit paths provided, inspect available files inside workspace
        let targets: Vec<String> = if paths_to_inspect.is_empty() {
            sandbox.list_workspace_files()?
        } else {
            paths_to_inspect.to_vec()
        };

        let log_start_idx = sandbox.access_log().len();

        for rel in targets {
            match sandbox.read_workspace_file(&rel) {
                Ok(_content) => {
                    accessed_files.push(rel);
                }
                Err(e) => {
                    blocked_attempts.push(rel.clone());
                    notes.push(format!("Blocked/failed access to '{}': {}", rel, e));
                }
            }
        }

        let telemetry_events: Vec<AgentTelemetryEvent> = sandbox.access_log()[log_start_idx..]
            .iter()
            .map(|rec| AgentTelemetryEvent {
                timestamp: rec.timestamp,
                session_id: rec.session_id.clone(),
                agent_name: self.adapter_name.clone(),
                action: if rec.allowed {
                    "workspace_read".to_string()
                } else {
                    "workspace_access_denied".to_string()
                },
                target_path: rec.requested_path.clone(),
                allowed: rec.allowed,
                artifact_id: rec.matched_artifact_id.clone(),
                canary_token: rec.matched_canary_token.clone(),
            })
            .collect();

        Ok(AgentResult {
            agent_session_id: agent_session.agent_session_id,
            agent_name: self.adapter_name.clone(),
            completed_at: Utc::now(),
            accessed_files,
            blocked_attempts,
            telemetry_events,
            notes,
        })
    }
}
