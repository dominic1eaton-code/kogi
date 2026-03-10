use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub postgres_dsn: String,
    pub postgres_database: String,
    pub postgres_schema_path: String,
    pub sqlite_path: String,
    pub sqlite_schema_path: String,
    pub storage_root: String,
    pub state_path_local: String,
    pub state_path_network: String,
    pub max_connections: u32,
    pub snapshot_retention: usize,
    pub backup_retention: usize,
    pub default_target: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DatabaseActor {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub scopes: Vec<String>,
}

impl DatabaseActor {
    pub fn system() -> Self {
        Self {
            id: "database-service".to_string(),
            role: "service".to_string(),
            token: String::new(),
            scopes: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DatabaseRequest {
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub actor: Option<DatabaseActor>,
    #[serde(default)]
    pub collection: Option<String>,
    #[serde(default)]
    pub record_id: Option<String>,
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub sql: Option<String>,
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
    #[serde(default)]
    pub options: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub correlation_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DatabaseResponse {
    pub request_id: String,
    pub action: String,
    pub status: String,
    pub message: String,
    pub timestamp_ms: u64,
    pub data: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access: Option<AccessDecision>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<DatabaseMetrics>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DatabaseMetrics {
    pub collection_count: usize,
    pub record_count: usize,
    pub snapshot_count: usize,
    pub checkpoint_count: usize,
    pub backup_count: usize,
    pub operation_count: u64,
    pub last_snapshot_ms: Option<u64>,
    pub last_checkpoint_ms: Option<u64>,
    pub last_backup_ms: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccessPolicy {
    pub role: String,
    pub actions: Vec<String>,
    pub collections: Vec<String>,
    pub read_only: bool,
}

impl AccessPolicy {
    pub fn allows(&self, action: &str, collection: Option<&str>) -> bool {
        let action_match = self.actions.iter().any(|allowed| {
            allowed == "*" || allowed.eq_ignore_ascii_case(action)
        });
        if !action_match {
            return false;
        }

        if self.collections.is_empty() || self.collections.iter().any(|c| c == "*") {
            return true;
        }

        if let Some(collection) = collection {
            return self
                .collections
                .iter()
                .any(|c| c.eq_ignore_ascii_case(collection));
        }

        false
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccessDecision {
    pub allowed: bool,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_role: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LockInfo {
    pub lock_id: String,
    pub collection: String,
    pub record_id: String,
    pub owner: String,
    pub acquired_ms: u64,
    pub ttl_ms: u64,
}

impl LockInfo {
    pub fn is_expired(&self, now_ms: u64) -> bool {
        now_ms.saturating_sub(self.acquired_ms) > self.ttl_ms
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConcurrencyState {
    pub max_readers: u32,
    pub max_writers: u32,
    pub active_readers: u32,
    pub active_writers: u32,
    pub lock_timeout_ms: u64,
    pub last_conflict: Option<String>,
}

impl Default for ConcurrencyState {
    fn default() -> Self {
        Self {
            max_readers: 256,
            max_writers: 64,
            active_readers: 0,
            active_writers: 0,
            lock_timeout_ms: 15000,
            last_conflict: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScalingState {
    pub read_replicas: u32,
    pub write_nodes: u32,
    pub shard_count: u32,
    pub autoscale_enabled: bool,
    pub strategy: String,
}

impl Default for ScalingState {
    fn default() -> Self {
        Self {
            read_replicas: 1,
            write_nodes: 1,
            shard_count: 1,
            autoscale_enabled: false,
            strategy: "single_primary".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OptimizationState {
    pub vacuum_enabled: bool,
    pub analyze_enabled: bool,
    pub index_rebuild_enabled: bool,
    pub compression: String,
    pub last_optimized_ms: Option<u64>,
}

impl Default for OptimizationState {
    fn default() -> Self {
        Self {
            vacuum_enabled: true,
            analyze_enabled: true,
            index_rebuild_enabled: false,
            compression: "lz4".to_string(),
            last_optimized_ms: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StorageState {
    pub capacity_mb: u64,
    pub used_mb: u64,
    pub tier: String,
    pub engine: String,
    pub format: String,
    pub collection_count: usize,
    pub record_count: usize,
}

impl Default for StorageState {
    fn default() -> Self {
        Self {
            capacity_mb: 10240,
            used_mb: 0,
            tier: "primary".to_string(),
            engine: "postgres".to_string(),
            format: "row".to_string(),
            collection_count: 0,
            record_count: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectionState {
    pub connected: bool,
    pub mode: String,
    pub dsn: String,
    pub database: String,
    pub schema_path: String,
    pub last_checked_ms: Option<u64>,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self {
            connected: false,
            mode: "disconnected".to_string(),
            dsn: String::new(),
            database: String::new(),
            schema_path: String::new(),
            last_checked_ms: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub label: String,
    pub created_at_ms: u64,
    pub collection_count: usize,
    pub record_count: usize,
    pub data: BTreeMap<String, BTreeMap<String, serde_json::Value>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub created_at_ms: u64,
    pub wal_position: u64,
    pub record_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Backup {
    pub id: String,
    pub snapshot_id: String,
    pub created_at_ms: u64,
    pub location: String,
    pub size_mb: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub action: String,
    pub actor: String,
    pub status: String,
    pub timestamp_ms: u64,
    pub details: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DatabaseState {
    pub collections: BTreeMap<String, BTreeMap<String, serde_json::Value>>,
    pub next_id: u64,
    pub access_policies: Vec<AccessPolicy>,
    pub service_tokens: Vec<String>,
    pub locks: BTreeMap<String, LockInfo>,
    pub snapshots: Vec<Snapshot>,
    pub checkpoints: Vec<Checkpoint>,
    pub backups: Vec<Backup>,
    pub scaling: ScalingState,
    pub optimization: OptimizationState,
    pub storage: StorageState,
    pub concurrency: ConcurrencyState,
    pub connection: ConnectionState,
    pub audit: Vec<AuditEntry>,
    pub operation_count: u64,
}

impl Default for DatabaseState {
    fn default() -> Self {
        Self {
            collections: BTreeMap::new(),
            next_id: 1,
            access_policies: vec![
                AccessPolicy {
                    role: "admin".to_string(),
                    actions: vec!["*".to_string()],
                    collections: vec!["*".to_string()],
                    read_only: false,
                },
                AccessPolicy {
                    role: "service".to_string(),
                    actions: vec!["*".to_string()],
                    collections: vec!["*".to_string()],
                    read_only: false,
                },
                AccessPolicy {
                    role: "operator".to_string(),
                    actions: vec![
                        "read".to_string(),
                        "query".to_string(),
                        "snapshot".to_string(),
                        "checkpoint".to_string(),
                        "backup".to_string(),
                        "restore".to_string(),
                        "optimize".to_string(),
                        "scale".to_string(),
                        "storage".to_string(),
                        "concurrency".to_string(),
                    ],
                    collections: vec!["*".to_string()],
                    read_only: false,
                },
                AccessPolicy {
                    role: "analyst".to_string(),
                    actions: vec![
                        "read".to_string(),
                        "query".to_string(),
                        "storage".to_string(),
                    ],
                    collections: vec!["*".to_string()],
                    read_only: true,
                },
                AccessPolicy {
                    role: "guest".to_string(),
                    actions: vec!["read".to_string(), "query".to_string()],
                    collections: vec!["public".to_string()],
                    read_only: true,
                },
            ],
            service_tokens: Vec::new(),
            locks: BTreeMap::new(),
            snapshots: Vec::new(),
            checkpoints: Vec::new(),
            backups: Vec::new(),
            scaling: ScalingState::default(),
            optimization: OptimizationState::default(),
            storage: StorageState::default(),
            concurrency: ConcurrencyState::default(),
            connection: ConnectionState::default(),
            audit: Vec::new(),
            operation_count: 0,
        }
    }
}

impl DatabaseMetrics {
    pub fn from_state(state: &DatabaseState) -> Self {
        let record_count = state
            .collections
            .values()
            .map(|records| records.len())
            .sum();
        let last_snapshot_ms = state.snapshots.last().map(|s| s.created_at_ms);
        let last_checkpoint_ms = state.checkpoints.last().map(|c| c.created_at_ms);
        let last_backup_ms = state.backups.last().map(|b| b.created_at_ms);
        Self {
            collection_count: state.collections.len(),
            record_count,
            snapshot_count: state.snapshots.len(),
            checkpoint_count: state.checkpoints.len(),
            backup_count: state.backups.len(),
            operation_count: state.operation_count,
            last_snapshot_ms,
            last_checkpoint_ms,
            last_backup_ms,
        }
    }
}
