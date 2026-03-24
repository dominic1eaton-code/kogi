// =============================================================================
// hypergrid::error — Error Types
// =============================================================================

use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum HypergridError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Permission denied: actor '{actor}' needs {required_tier:?} to write '{attr_key}'")]
    PermissionDenied { actor: String, attr_key: String, required_tier: crate::cell::PermissionTier },

    #[error("Policy denied: {reason}")]
    PolicyDenied { reason: String },

    #[error("Invalid state transition from '{from}' to '{to}'")]
    InvalidStateTransition { from: String, to: String },

    #[error("Cyclic dependency detected: adding edge from {from} to {to} would create a cycle")]
    CyclicDependency { from: Uuid, to: Uuid },

    #[error("Schema conflict: {0}")]
    SchemaConflict(String),

    #[error("Cube '{name}' does not exist in this Grid")]
    CubeNotFound { name: String },

    #[error("Invalid dimension coordinate: expected {expected} dimensions, got {got}")]
    InvalidCoordinate { expected: usize, got: usize },

    #[error("Invalid HyperQL: {0}")]
    InvalidHyperQL(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Federation error: peer '{peer}' returned error: {message}")]
    FederationError { peer: String, message: String },

    #[error("CRDT error: {0}")]
    CrdtError(#[from] CrdtError),

    #[error("Plugin error: {0}")]
    PluginError(#[from] PluginError),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Consent required for edge type '{edge_type}'")]
    ConsentRequired { edge_type: String },

    #[error("Shadow cell is read-only: attribute '{attr}' is not in write_back_attrs")]
    ShadowReadOnly { attr: String },

    #[error("Governance required: action '{action}' requires a governance proposal vote")]
    GovernanceRequired { action: String },

    #[error("Time travel error: {0}")]
    TimeTravelError(String),
}

#[derive(Debug, Error)]
pub enum CrdtError {
    #[error("GrowOnly counter violation: decrement attempted on a GrowOnlyCounter attribute")]
    GrowOnlyViolation,

    #[error("Wrong CRDT type: operation does not match attribute's declared semantics")]
    WrongCrdtType,

    #[error("Invalid state transition from '{from}' to '{to}' in Lattice CRDT")]
    InvalidStateTransition { from: String, to: String },

    #[error("Merge conflict: {0}")]
    MergeConflict(String),
}

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Plugin initialization failed: {0}")]
    InitFailed(String),

    #[error("Plugin computation failed: {0}")]
    ComputeFailed(String),

    #[error("Plugin health check failed: {0}")]
    HealthCheckFailed(String),

    #[error("Plugin not found: {0}")]
    NotFound(String),
}

impl From<serde_json::Error> for HypergridError {
    fn from(e: serde_json::Error) -> Self {
        Self::SerializationError(e.to_string())
    }
}
