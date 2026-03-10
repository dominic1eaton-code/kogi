#[path = "DatabaseSystem.rs"]
pub mod database_system;

pub mod model;

pub use database_system::DatabaseSystem;
pub use model::{
    AccessDecision, AccessPolicy, AuditEntry, Backup, Checkpoint, ConcurrencyState, ConnectionState,
    DatabaseActor, DatabaseConfig, DatabaseMetrics, DatabaseRequest, DatabaseResponse, DatabaseState,
    OptimizationState, ScalingState, Snapshot, StorageState,
};

pub fn to_json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value)
        .unwrap_or_else(|_| "{\"error\":\"serialization_failed\"}".to_string())
}
