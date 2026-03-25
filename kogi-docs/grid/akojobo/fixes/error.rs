//! Error types for the Portfolio System.

use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum PortfolioError {
    #[error("component not found: {0}")]
    NotFound(Uuid),

    #[error("permission denied for user '{user_id}': {action}")]
    PermissionDenied { user_id: String, action: String },

    #[error("invalid operation: {0}")]
    InvalidOperation(String),

    #[error("cyclic dependency between {0} and {1}")]
    CyclicDependency(Uuid, Uuid),

    #[error("version conflict: local={local}, remote={remote}")]
    VersionConflict { local: String, remote: String },

    #[error("already exists: {0}")]
    AlreadyExists(Uuid),

    #[error("invalid state: current={current:?}, attempted={attempted}")]
    InvalidState { current: crate::types::ComponentState, attempted: String },

    #[error("storage error: {0}")]
    StorageError(String),

    #[error("validation error: {0}")]
    ValidationError(String),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type PortfolioResult<T> = Result<T, PortfolioError>;
