//! HG-ERR — Unified error types for Hypergrid.

use thiserror::Error;
use uuid::Uuid;

/// The unified error type for all Hypergrid operations.
#[derive(Debug, Error)]
pub enum HypergridError {
    #[error("entity not found: {0}")]
    NotFound(String),

    #[error("permission denied for actor '{actor}': {reason}")]
    PermissionDenied { actor: String, reason: String },

    #[error("invalid operation: {0}")]
    InvalidOperation(String),

    #[error("dimension index {0} out of range (max N={1})")]
    DimensionOutOfRange(u8, u8),

    #[error("dimensionality mismatch: coordinate has {coord_n} keys, hypercube has N={cube_n}")]
    DimensionalityMismatch { coord_n: usize, cube_n: u8 },

    #[error("maximum dimensionality N=16 exceeded")]
    MaxDimensionalityExceeded,

    #[error("attribute key '{0}' is not registered in the AttributeKeyRegistry")]
    UnregisteredAttributeKey(String),

    #[error("CRDT conflict on cell {cell_id}: {reason}")]
    CrdtConflict { cell_id: Uuid, reason: String },

    #[error("cyclic dependency detected between {0} and {1}")]
    CyclicDependency(Uuid, Uuid),

    #[error("version conflict: local={local}, remote={remote}")]
    VersionConflict { local: u64, remote: u64 },

    #[error("entity already exists: {0}")]
    AlreadyExists(String),

    #[error("schema validation error: {0}")]
    SchemaValidation(String),

    #[error("formula error: {0}")]
    FormulaError(String),

    #[error("computation error: {0}")]
    ComputationError(String),

    #[error("federation error: {0}")]
    FederationError(String),

    #[error("plugin error from '{plugin_id}': {reason}")]
    PluginError { plugin_id: String, reason: String },

    #[error("storage error: {0}")]
    StorageError(String),

    #[error("namespace error: {0}")]
    NamespaceError(String),

    #[error("graph error: {0}")]
    GraphError(String),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// The unified result type for Hypergrid operations.
pub type HypergridResult<T> = Result<T, HypergridError>;
