//! Stage 05 Agent Integration & Hybrid Protection Subsystem.

pub mod agent;
pub mod docs_gen;
pub mod input;
pub mod installer;
pub mod models;

pub use agent::{InspectionAgent, LocalTestAgentAdapter};
pub use docs_gen::{generate_inspection_guide, generate_security_doc};
pub use input::{RepositoryConnectionResult, RepositoryInputManager, ValidatedGitHubUrl};
pub use installer::{ProtectionInstallOutcome, ProtectionInstaller};
pub use models::{
    AgentCapabilities, AgentManifest, AgentResult, AgentSession, AgentTelemetryEvent,
    ProtectionPlan, ProtectionProfile, RepositoryRecord, RepositorySourceKind, RollbackReport,
};
