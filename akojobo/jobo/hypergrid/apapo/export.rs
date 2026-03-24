// =============================================================================
// hypergrid::export — HG-EXPORT: Export and Snapshot System
//
// ExportFormat, ExportRequest, ExportResult, SnapshotStore, Snapshot
// =============================================================================

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::cell::{CubeId, DimKey, GridId, HyperRow, TypedAttrValue};
use crate::dim::DimSlice;
use crate::error::HypergridError;

// ─── ExportFormat ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    Csv,
    Xlsx,
    Json,
    JsonLd,     // JSON-LD with schema.org / Hypergrid schema definitions
    Parquet,    // Apache Parquet (columnar; for data lake pipelines)
    Arrow,      // Apache Arrow IPC format
    NdJson,     // Newline-delimited JSON (for streaming)
}

// ─── ExportRequest ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    pub cube_id:        CubeId,
    pub format:         ExportFormat,
    pub filter:         Option<DimSlice>,
    pub attr_keys:      Option<Vec<String>>,  // None = all attrs
    pub view_id:        Option<Uuid>,          // apply a saved view's filters/folds
    pub as_of:          Option<DateTime<Utc>>, // time-travel export
    pub include_schema: bool,
    pub include_meta:   bool,                  // include VectorClock, timestamps
    pub label:          Option<String>,
}

// ─── ExportResult ────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct ExportResult {
    pub format:      ExportFormat,
    pub bytes:       Vec<u8>,
    pub row_count:   u64,
    pub col_count:   u64,
    pub size_bytes:  u64,
    pub export_id:   Uuid,
    pub generated_at: DateTime<Utc>,
}

// ─── HypergridExporter ───────────────────────────────────────────────────────

pub struct HypergridExporter;

impl HypergridExporter {
    /// Export HyperRows to the requested format.
    /// In production: large exports are async, streamed to S3, returned as signed URL.
    pub fn export(
        &self,
        rows: &[HyperRow],
        request: &ExportRequest,
    ) -> Result<ExportResult, HypergridError> {
        let bytes = match request.format {
            ExportFormat::Json => self.to_json(rows, request)?,
            ExportFormat::Csv  => self.to_csv(rows, request)?,
            ExportFormat::NdJson => self.to_ndjson(rows, request)?,
            _ => {
                // Parquet / Arrow / XLSX: stub
                serde_json::to_vec(&self.rows_to_json_value(rows))
                    .map_err(|e| HypergridError::SerializationError(e.to_string()))?
            }
        };

        let col_count = rows.first().map(|r| r.attrs.len() as u64).unwrap_or(0);
        let size = bytes.len() as u64;

        Ok(ExportResult {
            format:      request.format.clone(),
            row_count:   rows.len() as u64,
            col_count,
            size_bytes:  size,
            bytes,
            export_id:   Uuid::new_v4(),
            generated_at: Utc::now(),
        })
    }

    fn to_json(&self, rows: &[HyperRow], _req: &ExportRequest) -> Result<Vec<u8>, HypergridError> {
        let value = self.rows_to_json_value(rows);
        serde_json::to_vec_pretty(&value)
            .map_err(|e| HypergridError::SerializationError(e.to_string()))
    }

    fn to_ndjson(&self, rows: &[HyperRow], _req: &ExportRequest) -> Result<Vec<u8>, HypergridError> {
        let mut buf = Vec::new();
        for row in rows {
            let obj = self.row_to_json_object(row);
            let mut line = serde_json::to_vec(&obj)
                .map_err(|e| HypergridError::SerializationError(e.to_string()))?;
            line.push(b'\n');
            buf.extend(line);
        }
        Ok(buf)
    }

    fn to_csv(&self, rows: &[HyperRow], _req: &ExportRequest) -> Result<Vec<u8>, HypergridError> {
        if rows.is_empty() { return Ok(Vec::new()); }

        // Collect all attr keys across all rows
        let mut all_keys: Vec<String> = rows.iter()
            .flat_map(|r| r.attrs.keys().cloned())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        all_keys.sort();

        let mut csv = String::new();

        // Header
        csv.push_str("d1_key,");
        csv.push_str(&all_keys.join(","));
        csv.push('\n');

        // Rows
        for row in rows {
            let d1 = format!("{:?}", row.d1_key);
            csv.push_str(&d1);
            for key in &all_keys {
                csv.push(',');
                if let Some(val) = row.attrs.get(key) {
                    let s = match val {
                        TypedAttrValue::Text(t)   => t.replace(',', ";").replace('\n', " "),
                        TypedAttrValue::Number(n) => format!("{n:.6}"),
                        TypedAttrValue::Integer(i) => i.to_string(),
                        TypedAttrValue::Bool(b)   => b.to_string(),
                        TypedAttrValue::Null      => String::new(),
                        other => serde_json::to_string(other).unwrap_or_default()
                            .replace(',', ";").replace('\n', " "),
                    };
                    csv.push_str(&s);
                }
            }
            csv.push('\n');
        }

        Ok(csv.into_bytes())
    }

    fn rows_to_json_value(&self, rows: &[HyperRow]) -> serde_json::Value {
        let arr: Vec<_> = rows.iter().map(|r| self.row_to_json_object(r)).collect();
        serde_json::Value::Array(arr)
    }

    fn row_to_json_object(&self, row: &HyperRow) -> serde_json::Value {
        let mut obj = serde_json::Map::new();
        obj.insert("d1_key".into(), serde_json::to_value(&row.d1_key).unwrap_or_default());
        for (k, v) in &row.attrs {
            obj.insert(k.clone(), serde_json::to_value(v).unwrap_or_default());
        }
        serde_json::Value::Object(obj)
    }
}

// =============================================================================
// Snapshot System
// =============================================================================

/// A point-in-time snapshot of a Hypercube.
/// Used for AS_OF time-travel (provides a faster starting point than replaying
/// the EventLog from genesis) and for disaster recovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub snapshot_id:    Uuid,
    pub cube_id:        CubeId,
    pub grid_id:        GridId,
    pub label:          String,
    pub checkpoint_ts:  DateTime<Utc>,  // EventLog replay starts from this point
    pub rows:           HashMap<String, HashMap<String, TypedAttrValue>>,
    // rows: { d1_key_str → { attr_key → value } }
    pub row_count:      u64,
    pub created_at:     DateTime<Utc>,
    pub retain_until:   Option<DateTime<Utc>>,
    pub status:         SnapshotStatus,
    pub size_bytes:     u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapshotStatus {
    Pending,    // creation in progress
    Complete,   // ready to use
    Failed,
    Expired,    // retain_until has passed
}

impl Snapshot {
    /// Genesis snapshot: empty — forces full EventLog replay from the beginning.
    pub fn genesis(cube_id: CubeId, grid_id: GridId) -> Self {
        Self {
            snapshot_id:   Uuid::nil(),
            cube_id, grid_id,
            label:         "genesis".into(),
            checkpoint_ts: DateTime::<Utc>::MIN_UTC,
            rows:          HashMap::new(),
            row_count:     0,
            created_at:    Utc::now(),
            retain_until:  None,
            status:        SnapshotStatus::Complete,
            size_bytes:    0,
        }
    }

    /// Create a snapshot from current HyperRows.
    pub fn create(
        cube_id: CubeId, grid_id: GridId,
        rows: &[HyperRow],
        label: impl Into<String>,
        retain_until: Option<DateTime<Utc>>,
    ) -> Self {
        let snapshot_rows: HashMap<String, HashMap<String, TypedAttrValue>> = rows.iter()
            .map(|r| {
                let key = format!("{:?}", r.d1_key);
                let attrs = r.attrs.clone();
                (key, attrs)
            })
            .collect();

        let size = serde_json::to_vec(&snapshot_rows).map(|v| v.len() as u64).unwrap_or(0);

        Self {
            snapshot_id:   Uuid::new_v4(),
            cube_id, grid_id,
            label:         label.into(),
            checkpoint_ts: Utc::now(),
            row_count:     rows.len() as u64,
            rows:          snapshot_rows,
            created_at:    Utc::now(),
            retain_until,
            status:        SnapshotStatus::Complete,
            size_bytes:    size,
        }
    }

    /// Read a row from the snapshot.
    pub fn get_row(&self, d1_key: &DimKey) -> Option<HashMap<String, TypedAttrValue>> {
        let key = format!("{:?}", d1_key);
        self.rows.get(&key).cloned()
    }

    pub fn is_expired(&self) -> bool {
        self.retain_until.map(|t| Utc::now() > t).unwrap_or(false)
    }
}

/// In-memory snapshot store. Production: S3 / MinIO (Parquet, compressed, versioned).
pub struct SnapshotStore {
    snapshots: std::sync::RwLock<HashMap<Uuid, Snapshot>>,
}

impl SnapshotStore {
    pub fn new() -> Self { Self { snapshots: Default::default() } }

    pub fn put(&self, snap: Snapshot) {
        self.snapshots.write().unwrap().insert(snap.snapshot_id, snap);
    }

    pub fn get(&self, id: Uuid) -> Option<Snapshot> {
        self.snapshots.read().unwrap().get(&id).cloned()
    }

    /// Find the most recent complete snapshot for a cube at or before `before`.
    pub fn latest_before(&self, cube_id: CubeId, before: DateTime<Utc>) -> Option<Snapshot> {
        self.snapshots.read().unwrap().values()
            .filter(|s| {
                s.cube_id == cube_id &&
                s.status == SnapshotStatus::Complete &&
                s.checkpoint_ts <= before &&
                !s.is_expired()
            })
            .max_by_key(|s| s.checkpoint_ts)
            .cloned()
    }

    pub fn list_for_cube(&self, cube_id: CubeId) -> Vec<Snapshot> {
        self.snapshots.read().unwrap().values()
            .filter(|s| s.cube_id == cube_id)
            .cloned()
            .collect()
    }
}
