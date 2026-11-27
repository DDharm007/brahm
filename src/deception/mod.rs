//! BRAHM Deception Subsystems (Stage 03 Decoy Generation & Stage 04 Repository Deception).

pub mod decoy;
pub mod layers;
pub mod mirror;
pub mod policy;
pub mod sandbox;
pub mod session;

pub use layers::{
    is_ignored_repo_dir, GenuineLayer, InspectionLayer, IntegrityCheckFinding,
    ProductionIntegrityGuard, ProductionIntegrityReport, SyntheticLayer,
};
pub use mirror::{
    ControlledMirrorManager, InspectionManifest, InspectionManifestEntry, MirrorPreparationResult,
};
pub use policy::{
    DeceptionPolicy, DeceptionPolicyEngine, DeceptionPolicyProfile, PolicyDecision,
    PolicyEvaluationResult,
};
pub use sandbox::{ControlledInspectionEnvironment, WorkspaceAccessRecord};
pub use session::{
    DeceptionPreparedOutcome, InspectionSession, InspectionSessionManager, SessionStatus,
};
