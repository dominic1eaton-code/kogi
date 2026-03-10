use crate::model::*;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static ID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StorageTarget {
    Local,
    Network,
}

struct StorageProfile {
    engine: String,
    mode: String,
    dsn: String,
    database: String,
    schema_path: String,
    storage_root: String,
    state_path: String,
}

pub struct DatabaseSystem {
    config: DatabaseConfig,
}

struct ActionOutcome {
    data: Value,
    message: String,
    mutated: bool,
}

impl DatabaseSystem {
    pub fn from_env() -> Self {
        let storage_root = resolve_storage_root();
        let state_path_local = resolve_state_path(StorageTarget::Local);
        let state_path_network = resolve_state_path(StorageTarget::Network);

        let postgres_dsn = env_default("KOGI_DATABASE_DSN", "");
        let postgres_dsn = if postgres_dsn.is_empty() {
            env_default("DATABASE_URL", "postgres://kogi:kogi@127.0.0.1:5432/kogi")
        } else {
            postgres_dsn
        };

        let postgres_database = env_default("KOGI_DATABASE_NAME", "kogi");
        let postgres_schema_path = env_default(
            "KOGI_DATABASE_POSTGRES_SCHEMA",
            &env_default("KOGI_DATABASE_SCHEMA", "kogi-database/postgres/schema.sql"),
        );
        let sqlite_schema_path =
            env_default("KOGI_DATABASE_SQLITE_SCHEMA", "kogi-database/sqlite/schema.sql");
        let sqlite_path = resolve_sqlite_path(&storage_root);

        let default_target = resolve_default_target();
        let max_connections = env_default("KOGI_DATABASE_MAX_CONNECTIONS", "64")
            .parse::<u32>()
            .unwrap_or(64);
        let snapshot_retention = env_default("KOGI_DATABASE_SNAPSHOT_RETENTION", "12")
            .parse::<usize>()
            .unwrap_or(12);
        let backup_retention = env_default("KOGI_DATABASE_BACKUP_RETENTION", "6")
            .parse::<usize>()
            .unwrap_or(6);

        let config = DatabaseConfig {
            postgres_dsn,
            postgres_database,
            postgres_schema_path,
            sqlite_path,
            sqlite_schema_path,
            storage_root,
            state_path_local,
            state_path_network,
            max_connections,
            snapshot_retention,
            backup_retention,
            default_target,
        };

        ensure_storage_paths(&config);

        Self { config }
    }

    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }

    pub fn handle_request(&self, request: DatabaseRequest) -> DatabaseResponse {
        let target = resolve_request_target(&self.config, &request);
        let profile = self.storage_profile(target);
        let mut warnings = Vec::new();
        let mut state = match self.load_state(&profile.state_path) {
            Ok(state) => state,
            Err(err) => {
                warnings.push(err);
                DatabaseState::default()
            }
        };

        let now = now_ms();
        let request_id = request
            .request_id
            .clone()
            .unwrap_or_else(|| new_id("req"));
        let action = normalize_action(&request.action);
        if action.is_empty() {
            return self.error_response(
                request_id,
                "invalid_action",
                "action is required",
                now,
                warnings,
            );
        }

        self.apply_storage_profile(&mut state, &profile);

        let actor = request.actor.clone().unwrap_or_else(DatabaseActor::system);
        let access = if action == "status" {
            AccessDecision {
                allowed: true,
                reason: "status is public".to_string(),
                policy_role: Some("public".to_string()),
            }
        } else {
            self.authorize(&state, &actor, &action, request.collection.as_deref())
        };

        if !access.allowed {
            let response = DatabaseResponse {
                request_id,
                action,
                status: "denied".to_string(),
                message: access.reason.clone(),
                timestamp_ms: now,
                data: json!({"error": "access_denied"}),
                access: Some(access),
                metrics: Some(DatabaseMetrics::from_state(&state)),
                warnings,
            };
            return response;
        }

        let outcome = match action.as_str() {
            "status" => Ok(self.status(&state, &profile)),
            "connect" => Ok(self.connect(&mut state, &profile)),
            "create" => self.create_record(&mut state, &request, &actor),
            "read" => self.read_record(&mut state, &request),
            "update" => self.update_record(&mut state, &request, &actor),
            "delete" => self.delete_record(&mut state, &request, &actor),
            "query" => self.query(&mut state, &request),
            "snapshot" => self.snapshot(&mut state, &request),
            "checkpoint" => self.checkpoint(&mut state),
            "backup" => self.backup(&mut state, &request, &profile),
            "restore" => self.restore(&mut state, &request),
            "scale" => self.scale(&mut state, &request),
            "optimize" => self.optimize(&mut state, &request),
            "storage" => self.storage(&mut state),
            "access" | "access_control" => self.access_control(&mut state, &request),
            "concurrency" => self.concurrency(&mut state, &request),
            "data_management" => self.data_management(&mut state, &request),
            _ => Err(format!("unknown action: {action}")),
        };

        let (status, message, data, mutated) = match outcome {
            Ok(outcome) => ("ok".to_string(), outcome.message, outcome.data, outcome.mutated),
            Err(err) => ("error".to_string(), err, json!({}), false),
        };

        if status == "ok" {
            state.operation_count = state.operation_count.saturating_add(1);
        }

        self.update_storage(&mut state);
        self.record_audit(&mut state, &request_id, &action, &actor, &status, &message);
        if mutated || status == "ok" {
            if let Err(err) = self.save_state(&state, &profile.state_path) {
                warnings.push(err);
            }
        }

        DatabaseResponse {
            request_id,
            action,
            status,
            message,
            timestamp_ms: now,
            data,
            access: Some(access),
            metrics: Some(DatabaseMetrics::from_state(&state)),
            warnings,
        }
    }

    fn status(&self, state: &DatabaseState, profile: &StorageProfile) -> ActionOutcome {
        ActionOutcome {
            data: json!({
                "config": self.config,
                "active_target": profile.mode,
                "active_engine": profile.engine,
                "active_state_path": profile.state_path,
                "active_storage_root": profile.storage_root,
                "connection": state.connection,
                "storage": state.storage,
                "scaling": state.scaling,
                "optimization": state.optimization,
                "concurrency": state.concurrency,
                "metrics": DatabaseMetrics::from_state(state),
                "snapshots": state.snapshots.len(),
                "backups": state.backups.len(),
                "checkpoints": state.checkpoints.len(),
                "audit_entries": state.audit.len(),
            }),
            message: "status ok".to_string(),
            mutated: false,
        }
    }

    fn connect(&self, state: &mut DatabaseState, profile: &StorageProfile) -> ActionOutcome {
        self.apply_storage_profile(state, profile);
        state.connection.connected = true;
        let base_mode = if env_default("KOGI_DATABASE_LIVE", "0") == "1" {
            "configured"
        } else {
            "simulated"
        };
        state.connection.mode = format!("{base_mode}-{}", profile.mode);
        state.connection.last_checked_ms = Some(now_ms());

        ActionOutcome {
            data: json!({
                "connection": state.connection,
                "engine": profile.engine,
                "schema": profile.schema_path,
                "database": profile.database,
                "storage_root": profile.storage_root,
                "max_connections": self.config.max_connections,
            }),
            message: "connection established".to_string(),
            mutated: true,
        }
    }

    fn create_record(
        &self,
        state: &mut DatabaseState,
        request: &DatabaseRequest,
        actor: &DatabaseActor,
    ) -> Result<ActionOutcome, String> {
        begin_write(state)?;
        let collection = request
            .collection
            .clone()
            .unwrap_or_else(|| "default".to_string());
        let mut record_value = request.payload.clone().unwrap_or_else(|| json!({}));
        let mut record_id = request.record_id.clone();
        if record_id.is_none() {
            record_id = record_value
                .get("id")
                .and_then(|value| value.as_str())
                .map(|value| value.to_string());
        }

        let record_id = record_id.unwrap_or_else(|| {
            let id = format!("record-{}", state.next_id);
            state.next_id = state.next_id.saturating_add(1);
            id
        });

        if let Some(object) = record_value.as_object_mut() {
            object.insert("id".to_string(), Value::String(record_id.clone()));
            object.insert("created_by".to_string(), Value::String(actor.id.clone()));
            object.insert("created_at_ms".to_string(), Value::Number(now_ms().into()));
        }

        let records = state
            .collections
            .entry(collection.clone())
            .or_insert_with(BTreeMap::new);
        if records.contains_key(&record_id) {
            end_write(state);
            return Err(format!("record already exists: {record_id}"));
        }

        records.insert(record_id.clone(), record_value.clone());
        end_write(state);

        Ok(ActionOutcome {
            data: json!({
                "collection": collection,
                "record_id": record_id,
                "record": record_value,
            }),
            message: "record created".to_string(),
            mutated: true,
        })
    }

    fn read_record(
        &self,
        state: &mut DatabaseState,
        request: &DatabaseRequest,
    ) -> Result<ActionOutcome, String> {
        begin_read(state)?;
        let collection = request.collection.clone();
        let record_id = request.record_id.clone();

        let data = match (collection, record_id) {
            (Some(collection), Some(record_id)) => {
                let record = state
                    .collections
                    .get(&collection)
                    .and_then(|records| records.get(&record_id))
                    .cloned();
                json!({
                    "collection": collection,
                    "record_id": record_id,
                    "record": record,
                })
            }
            (Some(collection), None) => {
                let records = state
                    .collections
                    .get(&collection)
                    .cloned()
                    .unwrap_or_default();
                json!({
                    "collection": collection,
                    "records": records,
                })
            }
            (None, _) => {
                let collections: Vec<String> = state.collections.keys().cloned().collect();
                json!({
                    "collections": collections,
                })
            }
        };
        end_read(state);

        Ok(ActionOutcome {
            data,
            message: "read completed".to_string(),
            mutated: false,
        })
    }

    fn update_record(
        &self,
        state: &mut DatabaseState,
        request: &DatabaseRequest,
        actor: &DatabaseActor,
    ) -> Result<ActionOutcome, String> {
        begin_write(state)?;
        let collection = request
            .collection
            .clone()
            .unwrap_or_else(|| "default".to_string());
        let record_id = request
            .record_id
            .clone()
            .ok_or_else(|| "record_id is required".to_string())?;

        let lock_id = match self.lock_record(state, actor, &collection, &record_id) {
            Ok(lock_id) => lock_id,
            Err(err) => {
                end_write(state);
                return Err(err);
            }
        };

        let updated_record = {
            let records = state
                .collections
                .entry(collection.clone())
                .or_insert_with(BTreeMap::new);
            let record = match records.get_mut(&record_id) {
                Some(record) => record,
                None => {
                    self.unlock_record(state, &collection, &record_id, &lock_id);
                    end_write(state);
                    return Err("record not found".to_string());
                }
            };

            if let Some(payload) = &request.payload {
                match (record.as_object_mut(), payload.as_object()) {
                    (Some(record_object), Some(update_object)) => {
                        for (key, value) in update_object {
                            record_object.insert(key.clone(), value.clone());
                        }
                        record_object.insert("updated_by".to_string(), Value::String(actor.id.clone()));
                        record_object.insert("updated_at_ms".to_string(), Value::Number(now_ms().into()));
                    }
                    _ => {
                        *record = payload.clone();
                    }
                }
            }

            record.clone()
        };

        self.unlock_record(state, &collection, &record_id, &lock_id);
        end_write(state);

        Ok(ActionOutcome {
            data: json!({
                "collection": collection,
                "record_id": record_id,
                "record": updated_record,
            }),
            message: "record updated".to_string(),
            mutated: true,
        })
    }

    fn delete_record(
        &self,
        state: &mut DatabaseState,
        request: &DatabaseRequest,
        actor: &DatabaseActor,
    ) -> Result<ActionOutcome, String> {
        begin_write(state)?;
        let collection = request
            .collection
            .clone()
            .unwrap_or_else(|| "default".to_string());
        let record_id = request
            .record_id
            .clone()
            .ok_or_else(|| "record_id is required".to_string())?;

        let lock_id = match self.lock_record(state, actor, &collection, &record_id) {
            Ok(lock_id) => lock_id,
            Err(err) => {
                end_write(state);
                return Err(err);
            }
        };
        let records = state
            .collections
            .entry(collection.clone())
            .or_insert_with(BTreeMap::new);
        let removed = records.remove(&record_id);
        self.unlock_record(state, &collection, &record_id, &lock_id);
        end_write(state);

        Ok(ActionOutcome {
            data: json!({
                "collection": collection,
                "record_id": record_id,
                "deleted": removed.is_some(),
            }),
            message: "record deleted".to_string(),
            mutated: true,
        })
    }

    fn query(
        &self,
        state: &mut DatabaseState,
        request: &DatabaseRequest,
    ) -> Result<ActionOutcome, String> {
        begin_read(state)?;
        let query = request
            .query
            .clone()
            .or_else(|| request.sql.clone())
            .unwrap_or_else(|| "select 1".to_string());
        let normalized = query.to_lowercase();
        let mut records = BTreeMap::new();

        if normalized.starts_with("select") {
            if let Some(collection) = parse_select_collection(&normalized) {
                records = state
                    .collections
                    .get(&collection)
                    .cloned()
                    .unwrap_or_default();
            }
        }
        end_read(state);

        Ok(ActionOutcome {
            data: json!({
                "query": query,
                "records": records,
                "row_count": records.len(),
                "executed_at_ms": now_ms(),
            }),
            message: "query executed".to_string(),
            mutated: false,
        })
    }

    fn snapshot(
        &self,
        state: &mut DatabaseState,
        request: &DatabaseRequest,
    ) -> Result<ActionOutcome, String> {
        let label = request
            .payload
            .as_ref()
            .and_then(|payload| payload.get("label"))
            .and_then(|value| value.as_str())
            .unwrap_or("snapshot")
            .to_string();

        let snapshot = Snapshot {
            id: new_id("snap"),
            label,
            created_at_ms: now_ms(),
            collection_count: state.collections.len(),
            record_count: total_records(&state.collections),
            data: state.collections.clone(),
        };
        state.snapshots.push(snapshot.clone());
        enforce_retention(&mut state.snapshots, self.config.snapshot_retention);

        Ok(ActionOutcome {
            data: json!({
                "snapshot_id": snapshot.id,
                "collection_count": snapshot.collection_count,
                "record_count": snapshot.record_count,
            }),
            message: "snapshot captured".to_string(),
            mutated: true,
        })
    }

    fn checkpoint(&self, state: &mut DatabaseState) -> Result<ActionOutcome, String> {
        let checkpoint = Checkpoint {
            id: new_id("checkpoint"),
            created_at_ms: now_ms(),
            wal_position: state.operation_count,
            record_count: total_records(&state.collections),
        };
        state.checkpoints.push(checkpoint.clone());

        Ok(ActionOutcome {
            data: json!({
                "checkpoint_id": checkpoint.id,
                "wal_position": checkpoint.wal_position,
            }),
            message: "checkpoint created".to_string(),
            mutated: true,
        })
    }

    fn backup(
        &self,
        state: &mut DatabaseState,
        request: &DatabaseRequest,
        profile: &StorageProfile,
    ) -> Result<ActionOutcome, String> {
        let snapshot_id = request
            .payload
            .as_ref()
            .and_then(|payload| payload.get("snapshot_id"))
            .and_then(|value| value.as_str())
            .map(|value| value.to_string());

        let snapshot = if let Some(snapshot_id) = snapshot_id {
            state
                .snapshots
                .iter()
                .find(|snap| snap.id == snapshot_id)
                .cloned()
                .ok_or_else(|| "snapshot not found".to_string())?
        } else if let Some(latest) = state.snapshots.last().cloned() {
            latest
        } else {
            let snapshot = Snapshot {
                id: new_id("snap"),
                label: "backup_snapshot".to_string(),
                created_at_ms: now_ms(),
                collection_count: state.collections.len(),
                record_count: total_records(&state.collections),
                data: state.collections.clone(),
            };
            state.snapshots.push(snapshot.clone());
            snapshot
        };

        let backup_id = new_id("backup");
        let backup_root = Path::new(&profile.storage_root).join("backups");
        fs::create_dir_all(&backup_root)
            .map_err(|err| format!("backup directory error: {err}"))?;
        let backup_path = backup_root.join(format!("{}.json", backup_id));
        let payload = json!({
            "snapshot": snapshot,
            "created_at_ms": now_ms(),
            "database": profile.database,
            "engine": profile.engine,
        });
        let encoded = serde_json::to_string_pretty(&payload)
            .map_err(|err| format!("backup encode error: {err}"))?;
        fs::write(&backup_path, encoded).map_err(|err| format!("backup write error: {err}"))?;

        let size_mb = backup_path
            .metadata()
            .map(|meta| bytes_to_mb(meta.len()))
            .unwrap_or(0);
        let backup = Backup {
            id: backup_id.clone(),
            snapshot_id: snapshot.id.clone(),
            created_at_ms: now_ms(),
            location: backup_path.to_string_lossy().to_string(),
            size_mb,
        };
        state.backups.push(backup.clone());
        enforce_retention(&mut state.backups, self.config.backup_retention);

        Ok(ActionOutcome {
            data: json!({
                "backup_id": backup.id,
                "snapshot_id": backup.snapshot_id,
                "location": backup.location,
                "size_mb": backup.size_mb,
            }),
            message: "backup created".to_string(),
            mutated: true,
        })
    }

    fn restore(
        &self,
        state: &mut DatabaseState,
        request: &DatabaseRequest,
    ) -> Result<ActionOutcome, String> {
        let payload = request.payload.clone().unwrap_or_else(|| json!({}));
        let backup_id = payload.get("backup_id").and_then(|value| value.as_str());
        let snapshot_id = payload.get("snapshot_id").and_then(|value| value.as_str());

        let snapshot = if let Some(backup_id) = backup_id {
            let backup = state
                .backups
                .iter()
                .find(|backup| backup.id == backup_id)
                .cloned()
                .ok_or_else(|| "backup not found".to_string())?;
            let content = fs::read_to_string(&backup.location)
                .map_err(|err| format!("backup read error: {err}"))?;
            let payload: Value = serde_json::from_str(&content)
                .map_err(|err| format!("backup parse error: {err}"))?;
            let snapshot_value = payload
                .get("snapshot")
                .cloned()
                .ok_or_else(|| "backup missing snapshot".to_string())?;
            let snapshot: Snapshot = serde_json::from_value(snapshot_value)
                .map_err(|err| format!("snapshot decode error: {err}"))?;
            snapshot
        } else if let Some(snapshot_id) = snapshot_id {
            state
                .snapshots
                .iter()
                .find(|snap| snap.id == snapshot_id)
                .cloned()
                .ok_or_else(|| "snapshot not found".to_string())?
        } else {
            return Err("backup_id or snapshot_id required".to_string());
        };

        state.collections = snapshot.data.clone();
        state.next_id = next_id_from_collections(&state.collections);

        Ok(ActionOutcome {
            data: json!({
                "restored_from": snapshot.id,
                "collection_count": snapshot.collection_count,
                "record_count": snapshot.record_count,
            }),
            message: "restore completed".to_string(),
            mutated: true,
        })
    }

    fn scale(
        &self,
        state: &mut DatabaseState,
        request: &DatabaseRequest,
    ) -> Result<ActionOutcome, String> {
        if let Some(payload) = &request.payload {
            if let Some(read_replicas) = payload.get("read_replicas").and_then(|v| v.as_u64()) {
                state.scaling.read_replicas = read_replicas as u32;
            }
            if let Some(write_nodes) = payload.get("write_nodes").and_then(|v| v.as_u64()) {
                state.scaling.write_nodes = write_nodes as u32;
            }
            if let Some(shard_count) = payload.get("shard_count").and_then(|v| v.as_u64()) {
                state.scaling.shard_count = shard_count as u32;
            }
            if let Some(strategy) = payload.get("strategy").and_then(|v| v.as_str()) {
                state.scaling.strategy = strategy.to_string();
            }
            if let Some(autoscale) = payload.get("autoscale_enabled").and_then(|v| v.as_bool()) {
                state.scaling.autoscale_enabled = autoscale;
            }
        }

        Ok(ActionOutcome {
            data: json!({
                "scaling": state.scaling,
            }),
            message: "scaling updated".to_string(),
            mutated: true,
        })
    }

    fn optimize(
        &self,
        state: &mut DatabaseState,
        request: &DatabaseRequest,
    ) -> Result<ActionOutcome, String> {
        if let Some(payload) = &request.payload {
            if let Some(vacuum) = payload.get("vacuum_enabled").and_then(|v| v.as_bool()) {
                state.optimization.vacuum_enabled = vacuum;
            }
            if let Some(analyze) = payload.get("analyze_enabled").and_then(|v| v.as_bool()) {
                state.optimization.analyze_enabled = analyze;
            }
            if let Some(index_rebuild) = payload
                .get("index_rebuild_enabled")
                .and_then(|v| v.as_bool())
            {
                state.optimization.index_rebuild_enabled = index_rebuild;
            }
            if let Some(compression) = payload.get("compression").and_then(|v| v.as_str()) {
                state.optimization.compression = compression.to_string();
            }
        }
        state.optimization.last_optimized_ms = Some(now_ms());

        Ok(ActionOutcome {
            data: json!({
                "optimization": state.optimization,
            }),
            message: "optimization updated".to_string(),
            mutated: true,
        })
    }

    fn storage(&self, state: &mut DatabaseState) -> Result<ActionOutcome, String> {
        Ok(ActionOutcome {
            data: json!({
                "storage": state.storage,
            }),
            message: "storage status".to_string(),
            mutated: false,
        })
    }

    fn access_control(
        &self,
        state: &mut DatabaseState,
        request: &DatabaseRequest,
    ) -> Result<ActionOutcome, String> {
        if let Some(payload) = &request.payload {
            if let Some(policies) = payload.get("policies").and_then(|value| value.as_array()) {
                let mut parsed = Vec::new();
                for policy in policies {
                    let role = policy
                        .get("role")
                        .and_then(|value| value.as_str())
                        .unwrap_or("custom")
                        .to_string();
                    let actions = policy
                        .get("actions")
                        .and_then(|value| value.as_array())
                        .map(|items| {
                            items
                                .iter()
                                .filter_map(|item| item.as_str())
                                .map(|item| item.to_string())
                                .collect::<Vec<String>>()
                        })
                        .unwrap_or_else(|| vec!["read".to_string()]);
                    let collections = policy
                        .get("collections")
                        .and_then(|value| value.as_array())
                        .map(|items| {
                            items
                                .iter()
                                .filter_map(|item| item.as_str())
                                .map(|item| item.to_string())
                                .collect::<Vec<String>>()
                        })
                        .unwrap_or_else(|| vec!["*".to_string()]);
                    let read_only = policy
                        .get("read_only")
                        .and_then(|value| value.as_bool())
                        .unwrap_or(false);

                    parsed.push(AccessPolicy {
                        role,
                        actions,
                        collections,
                        read_only,
                    });
                }
                if !parsed.is_empty() {
                    state.access_policies = parsed;
                }
            }
            if let Some(tokens) = payload
                .get("service_tokens")
                .and_then(|value| value.as_array())
            {
                state.service_tokens = tokens
                    .iter()
                    .filter_map(|value| value.as_str())
                    .map(|value| value.to_string())
                    .collect();
            }
        }

        Ok(ActionOutcome {
            data: json!({
                "policies": state.access_policies,
                "service_tokens": state.service_tokens,
            }),
            message: "access control updated".to_string(),
            mutated: true,
        })
    }

    fn concurrency(
        &self,
        state: &mut DatabaseState,
        request: &DatabaseRequest,
    ) -> Result<ActionOutcome, String> {
        if let Some(payload) = &request.payload {
            if let Some(max_readers) = payload.get("max_readers").and_then(|v| v.as_u64()) {
                state.concurrency.max_readers = max_readers as u32;
            }
            if let Some(max_writers) = payload.get("max_writers").and_then(|v| v.as_u64()) {
                state.concurrency.max_writers = max_writers as u32;
            }
            if let Some(lock_timeout_ms) = payload.get("lock_timeout_ms").and_then(|v| v.as_u64()) {
                state.concurrency.lock_timeout_ms = lock_timeout_ms;
            }
        }

        Ok(ActionOutcome {
            data: json!({
                "concurrency": state.concurrency,
            }),
            message: "concurrency updated".to_string(),
            mutated: true,
        })
    }

    fn data_management(
        &self,
        state: &mut DatabaseState,
        request: &DatabaseRequest,
    ) -> Result<ActionOutcome, String> {
        let mut removed = None;
        if let Some(payload) = &request.payload {
            if let Some(collection) = payload.get("purge_collection").and_then(|v| v.as_str()) {
                let removed_records = state.collections.remove(collection).map(|c| c.len());
                removed = removed_records.map(|count| (collection.to_string(), count));
            }
            if let Some(retain) = payload.get("audit_retain").and_then(|v| v.as_u64()) {
                retain_latest_audit(&mut state.audit, retain as usize);
            }
        }

        Ok(ActionOutcome {
            data: json!({
                "purged": removed.map(|(collection, count)| json!({"collection": collection, "records": count})),
                "audit_entries": state.audit.len(),
            }),
            message: "data management completed".to_string(),
            mutated: true,
        })
    }

    fn authorize(
        &self,
        state: &DatabaseState,
        actor: &DatabaseActor,
        action: &str,
        collection: Option<&str>,
    ) -> AccessDecision {
        if !actor.token.is_empty() && state.service_tokens.contains(&actor.token) {
            return AccessDecision {
                allowed: true,
                reason: "service token matched".to_string(),
                policy_role: Some("service_token".to_string()),
            };
        }

        let role = if actor.role.is_empty() {
            "guest"
        } else {
            actor.role.as_str()
        };

        for policy in &state.access_policies {
            if policy.role.eq_ignore_ascii_case(role) && policy.allows(action, collection) {
                return AccessDecision {
                    allowed: true,
                    reason: format!("policy {role} allowed"),
                    policy_role: Some(policy.role.clone()),
                };
            }
        }

        AccessDecision {
            allowed: false,
            reason: format!("role {role} not permitted for action {action}"),
            policy_role: None,
        }
    }

    fn update_storage(&self, state: &mut DatabaseState) {
        let record_count = total_records(&state.collections);
        let encoded = serde_json::to_vec(&state.collections).unwrap_or_default();
        state.storage.used_mb = bytes_to_mb(encoded.len() as u64);
        state.storage.collection_count = state.collections.len();
        state.storage.record_count = record_count;
    }

    fn record_audit(
        &self,
        state: &mut DatabaseState,
        request_id: &str,
        action: &str,
        actor: &DatabaseActor,
        status: &str,
        message: &str,
    ) {
        let entry = AuditEntry {
            id: request_id.to_string(),
            action: action.to_string(),
            actor: actor.id.clone(),
            status: status.to_string(),
            timestamp_ms: now_ms(),
            details: message.to_string(),
        };
        state.audit.push(entry);
        retain_latest_audit(&mut state.audit, 500);
    }

    fn lock_record(
        &self,
        state: &mut DatabaseState,
        actor: &DatabaseActor,
        collection: &str,
        record_id: &str,
    ) -> Result<String, String> {
        let now = now_ms();
        let lock_key = format!("{}::{}", collection, record_id);
        if let Some(lock) = state.locks.get(&lock_key) {
            if !lock.is_expired(now) && lock.owner != actor.id {
                state.concurrency.last_conflict = Some(format!(
                    "lock held by {} for {}:{}",
                    lock.owner, collection, record_id
                ));
                return Err("record locked".to_string());
            }
        }

        let lock_id = new_id("lock");
        state.locks.insert(
            lock_key,
            LockInfo {
                lock_id: lock_id.clone(),
                collection: collection.to_string(),
                record_id: record_id.to_string(),
                owner: actor.id.clone(),
                acquired_ms: now,
                ttl_ms: state.concurrency.lock_timeout_ms,
            },
        );
        Ok(lock_id)
    }

    fn unlock_record(
        &self,
        state: &mut DatabaseState,
        collection: &str,
        record_id: &str,
        lock_id: &str,
    ) {
        let lock_key = format!("{}::{}", collection, record_id);
        if let Some(lock) = state.locks.get(&lock_key) {
            if lock.lock_id == lock_id {
                state.locks.remove(&lock_key);
            }
        }
    }

    fn load_state(&self, state_path: &str) -> Result<DatabaseState, String> {
        let path = Path::new(state_path);
        if !path.exists() {
            return Ok(DatabaseState::default());
        }
        let content = fs::read_to_string(path)
            .map_err(|err| format!("state read error: {err}"))?;
        let state: DatabaseState = serde_json::from_str(&content)
            .map_err(|err| format!("state parse error: {err}"))?;
        Ok(state)
    }

    fn save_state(&self, state: &DatabaseState, state_path: &str) -> Result<(), String> {
        let path = Path::new(state_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| format!("state dir error: {err}"))?;
        }
        let content = serde_json::to_string_pretty(state)
            .map_err(|err| format!("state encode error: {err}"))?;
        fs::write(path, content).map_err(|err| format!("state write error: {err}"))?;
        Ok(())
    }

    fn storage_profile(&self, target: StorageTarget) -> StorageProfile {
        let (engine, mode, dsn, database, schema_path) = match target {
            StorageTarget::Local => (
                "sqlite",
                "local",
                format!("sqlite:{}", self.config.sqlite_path),
                self.config.sqlite_path.clone(),
                self.config.sqlite_schema_path.clone(),
            ),
            StorageTarget::Network => (
                "postgres",
                "network",
                self.config.postgres_dsn.clone(),
                self.config.postgres_database.clone(),
                self.config.postgres_schema_path.clone(),
            ),
        };

        let storage_root = storage_root_for(&self.config.storage_root, target);
        let state_path = match target {
            StorageTarget::Local => self.config.state_path_local.clone(),
            StorageTarget::Network => self.config.state_path_network.clone(),
        };

        StorageProfile {
            engine: engine.to_string(),
            mode: mode.to_string(),
            dsn,
            database,
            schema_path,
            storage_root,
            state_path,
        }
    }

    fn apply_storage_profile(&self, state: &mut DatabaseState, profile: &StorageProfile) {
        state.connection.dsn = profile.dsn.clone();
        state.connection.database = profile.database.clone();
        state.connection.schema_path = profile.schema_path.clone();
        state.storage.engine = profile.engine.clone();
        state.storage.tier = profile.mode.clone();
    }

    fn error_response(
        &self,
        request_id: String,
        action: &str,
        message: &str,
        timestamp_ms: u64,
        warnings: Vec<String>,
    ) -> DatabaseResponse {
        DatabaseResponse {
            request_id,
            action: action.to_string(),
            status: "error".to_string(),
            message: message.to_string(),
            timestamp_ms,
            data: json!({"error": message}),
            access: None,
            metrics: None,
            warnings,
        }
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn new_id(prefix: &str) -> String {
    let counter = ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("{}-{}-{}", prefix, now_ms(), counter)
}

fn env_default(key: &str, fallback: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| fallback.to_string())
}

fn normalize_action(action: &str) -> String {
    action.trim().to_lowercase().replace(' ', "_")
}

fn parse_select_collection(query: &str) -> Option<String> {
    let tokens: Vec<&str> = query.split_whitespace().collect();
    let from_index = tokens.iter().position(|token| *token == "from")?;
    tokens
        .get(from_index + 1)
        .map(|value| value.trim_matches(';').to_string())
}

fn total_records(collections: &BTreeMap<String, BTreeMap<String, Value>>) -> usize {
    collections.values().map(|records| records.len()).sum()
}

fn bytes_to_mb(bytes: u64) -> u64 {
    if bytes == 0 {
        return 0;
    }
    let mb = bytes / (1024 * 1024);
    if mb == 0 { 1 } else { mb }
}

fn begin_read(state: &mut DatabaseState) -> Result<(), String> {
    if state.concurrency.active_readers >= state.concurrency.max_readers {
        state.concurrency.last_conflict = Some("readers limit reached".to_string());
        return Err("read concurrency limit reached".to_string());
    }
    state.concurrency.active_readers += 1;
    Ok(())
}

fn end_read(state: &mut DatabaseState) {
    if state.concurrency.active_readers > 0 {
        state.concurrency.active_readers -= 1;
    }
}

fn begin_write(state: &mut DatabaseState) -> Result<(), String> {
    if state.concurrency.active_writers >= state.concurrency.max_writers {
        state.concurrency.last_conflict = Some("writers limit reached".to_string());
        return Err("write concurrency limit reached".to_string());
    }
    state.concurrency.active_writers += 1;
    Ok(())
}

fn end_write(state: &mut DatabaseState) {
    if state.concurrency.active_writers > 0 {
        state.concurrency.active_writers -= 1;
    }
}

fn retain_latest_audit(entries: &mut Vec<AuditEntry>, retain: usize) {
    if entries.len() <= retain {
        return;
    }
    let start = entries.len().saturating_sub(retain);
    entries.drain(0..start);
}

fn enforce_retention<T>(items: &mut Vec<T>, retention: usize) {
    if retention == 0 || items.len() <= retention {
        return;
    }
    let drop_count = items.len().saturating_sub(retention);
    items.drain(0..drop_count);
}

fn next_id_from_collections(collections: &BTreeMap<String, BTreeMap<String, Value>>) -> u64 {
    let mut max_id = 0u64;
    for records in collections.values() {
        for key in records.keys() {
            if let Some(stripped) = key.strip_prefix("record-") {
                if let Ok(value) = stripped.parse::<u64>() {
                    if value > max_id {
                        max_id = value;
                    }
                }
            }
        }
    }
    max_id.saturating_add(1)
}

fn resolve_state_path(target: StorageTarget) -> String {
    let env_key = match target {
        StorageTarget::Local => "KOGI_DATABASE_STATE_PATH_LOCAL",
        StorageTarget::Network => "KOGI_DATABASE_STATE_PATH_NETWORK",
    };

    if let Ok(path) = std::env::var(env_key) {
        if !path.is_empty() {
            return path;
        }
    }

    if let Ok(path) = std::env::var("KOGI_DATABASE_STATE_PATH") {
        if !path.is_empty() {
            return path;
        }
    }

    if let Some(root) = find_repo_root() {
        let filename = match target {
            StorageTarget::Local => "database_state_local.json",
            StorageTarget::Network => "database_state_network.json",
        };
        let path = root
            .join("kogi-modules")
            .join("database")
            .join("state")
            .join(filename);
        return path.to_string_lossy().to_string();
    }

    match target {
        StorageTarget::Local => "kogi-database-state-local.json".to_string(),
        StorageTarget::Network => "kogi-database-state-network.json".to_string(),
    }
}

fn resolve_storage_root() -> String {
    if let Ok(root) = std::env::var("KOGI_DATABASE_STORAGE_ROOT") {
        if !root.is_empty() {
            return root;
        }
    }

    if let Some(root) = find_repo_root() {
        return root.join("kogi-database").to_string_lossy().to_string();
    }

    "kogi-database".to_string()
}

fn resolve_sqlite_path(storage_root: &str) -> String {
    if let Ok(path) = std::env::var("KOGI_DATABASE_SQLITE_PATH") {
        if !path.is_empty() {
            return path;
        }
    }

    Path::new(storage_root)
        .join("sqlite")
        .join("kogi.db")
        .to_string_lossy()
        .to_string()
}

fn find_repo_root() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;
    for _ in 0..8 {
        if current.join("kogi-modules").exists() {
            return Some(current);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

fn ensure_storage_paths(config: &DatabaseConfig) {
    for state_path in [&config.state_path_local, &config.state_path_network] {
        let path = Path::new(state_path);
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
    }

    for target in [StorageTarget::Local, StorageTarget::Network] {
        let root = storage_root_for(&config.storage_root, target);
        let backup_root = Path::new(&root).join("backups");
        let _ = fs::create_dir_all(backup_root);
    }
}

fn storage_root_for(storage_root: &str, target: StorageTarget) -> String {
    let segment = match target {
        StorageTarget::Local => "sqlite",
        StorageTarget::Network => "postgres",
    };
    Path::new(storage_root)
        .join(segment)
        .to_string_lossy()
        .to_string()
}

fn resolve_default_target() -> String {
    let mode = env_default("KOGI_DATABASE_MODE", "");
    if !mode.is_empty() {
        return mode;
    }
    let target = env_default("KOGI_DATABASE_TARGET", "");
    if !target.is_empty() {
        return target;
    }
    env_default("KOGI_DATABASE_ENGINE", "network")
}

fn resolve_request_target(config: &DatabaseConfig, request: &DatabaseRequest) -> StorageTarget {
    if let Some(options) = &request.options {
        for key in ["storage", "storage_mode", "storage_target", "engine"] {
            if let Some(value) = options.get(key) {
                if let Some(parsed) = parse_storage_target(value) {
                    return parsed;
                }
            }
        }
    }

    if let Some(payload) = &request.payload {
        for key in ["storage", "storage_mode", "storage_target", "engine"] {
            if let Some(value) = payload.get(key).and_then(|item| item.as_str()) {
                if let Some(parsed) = parse_storage_target(value) {
                    return parsed;
                }
            }
        }
    }

    parse_storage_target(&config.default_target).unwrap_or(StorageTarget::Network)
}

fn parse_storage_target(value: &str) -> Option<StorageTarget> {
    let normalized = value.trim().to_lowercase();
    match normalized.as_str() {
        "local" | "sqlite" | "desktop" | "client" => Some(StorageTarget::Local),
        "network" | "postgres" | "postgresql" | "remote" | "server" => {
            Some(StorageTarget::Network)
        }
        _ => None,
    }
}
