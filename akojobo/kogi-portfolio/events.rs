//! EventLog — append-only, event-sourced audit trail.

use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::types::ComponentId;

/// Maximum events retained in the in-memory EventLog before FIFO eviction.
pub const EVENT_LOG_CAP: usize = 10_000;

// ── PortfolioEventKind ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortfolioEventKind {
    // Component lifecycle
    ComponentCreated,
    ComponentUpdated,
    ComponentRemoved,
    ComponentStatusChanged,
    ComponentStateChanged,

    // Graph
    EdgeAdded,
    EdgeRemoved,

    // Governance
    PolicyAttached,
    PolicyDetached,
    ApprovalRequested,
    ApprovalGranted,
    ApprovalRejected,

    // Resource
    ResourceAllocated,
    ResourceConsumed,

    // Time-Travel
    SnapshotSaved,
    CheckpointCreated,
    StateRestored,

    // Distribution
    CrdtMergeApplied,

    // Action
    ActionPerformed,
    UserAdded,
    UserRemoved,

    // TMS
    ToolboxAttached,
    ToolboxDetached,

    // Custom
    Custom(String),
}

// ── PortfolioEvent ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioEvent {
    pub event_id: Uuid,
    pub event_kind: PortfolioEventKind,
    pub component_id: Option<ComponentId>,
    pub actor: String,          // node ID
    pub timestamp: DateTime<Utc>,
    pub payload: JsonValue,
}

impl PortfolioEvent {
    pub fn new(kind: PortfolioEventKind, component_id: Option<ComponentId>, actor: &str) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_kind: kind,
            component_id,
            actor: actor.to_owned(),
            timestamp: Utc::now(),
            payload: JsonValue::Null,
        }
    }

    pub fn with_payload(mut self, payload: JsonValue) -> Self {
        self.payload = payload;
        self
    }
}

// ── Snapshot ─────────────────────────────────────────────────────────────────

/// A point-in-time snapshot of the PortfolioSystem component store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub snapshot_id: Uuid,
    pub label: String,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub is_checkpoint: bool,
    /// Serialised component store (JSON blob in production; HashMap here).
    pub component_data: JsonValue,
    pub edge_data: JsonValue,
}

impl Snapshot {
    pub fn new(label: impl Into<String>, note: Option<String>, by: &str, is_checkpoint: bool) -> Self {
        Self {
            snapshot_id: Uuid::new_v4(),
            label: label.into(),
            note,
            created_at: Utc::now(),
            created_by: by.to_owned(),
            is_checkpoint,
            component_data: JsonValue::Null,
            edge_data: JsonValue::Null,
        }
    }
}

// ── EventLog ─────────────────────────────────────────────────────────────────

/// Append-only `VecDeque<PortfolioEvent>`.
/// Bounded at `EVENT_LOG_CAP` events; FIFO eviction when cap is reached.
#[derive(Debug, Clone, Default)]
pub struct EventLog {
    entries: VecDeque<PortfolioEvent>,
    pub snapshots: HashMap<Uuid, Snapshot>,
}

impl EventLog {
    pub fn new() -> Self { Self::default() }

    /// Append a new event; evict oldest entry if at capacity.
    pub fn emit(&mut self, event: PortfolioEvent) {
        if self.entries.len() >= EVENT_LOG_CAP {
            self.entries.pop_front();
        }
        self.entries.push_back(event);
    }

    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }

    /// Return events matching a specific kind.
    pub fn events_of_kind(&self, kind: &PortfolioEventKind) -> Vec<&PortfolioEvent> {
        self.entries.iter().filter(|e| &e.event_kind == kind).collect()
    }

    /// Return events for a specific component.
    pub fn events_for(&self, component_id: ComponentId) -> Vec<&PortfolioEvent> {
        self.entries.iter().filter(|e| e.component_id == Some(component_id)).collect()
    }

    /// Return all events after a timestamp (for replay / audit).
    pub fn events_after(&self, since: DateTime<Utc>) -> Vec<&PortfolioEvent> {
        self.entries.iter().filter(|e| e.timestamp > since).collect()
    }

    pub fn all_events(&self) -> Vec<&PortfolioEvent> {
        self.entries.iter().collect()
    }

    // ── Snapshot / Time-Travel ─────────────────────────────────────────────

    pub fn save_snapshot(&mut self, snap: Snapshot) -> Uuid {
        let id = snap.snapshot_id;
        self.snapshots.insert(id, snap);
        id
    }

    pub fn get_snapshot(&self, snapshot_id: Uuid) -> Option<&Snapshot> {
        self.snapshots.get(&snapshot_id)
    }

    pub fn checkpoints(&self) -> Vec<&Snapshot> {
        self.snapshots.values().filter(|s| s.is_checkpoint).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_log_cap_eviction() {
        let mut log = EventLog::new();
        for i in 0..EVENT_LOG_CAP + 5 {
            log.emit(PortfolioEvent::new(
                PortfolioEventKind::ComponentCreated,
                Some(Uuid::new_v4()),
                "node-1",
            ));
        }
        assert_eq!(log.len(), EVENT_LOG_CAP);
    }
}
