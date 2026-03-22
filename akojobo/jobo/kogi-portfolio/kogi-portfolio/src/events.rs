//! Portfolio event system — wraps hypergrid::core::EventLog.
//!
//! `hypergrid::core::EventLog` is the append-only audit trail that stores
//! every mutation at the cell level. The Portfolio System sits on top of that
//! and adds domain-typed `PortfolioEvent` records that carry business meaning
//! (ComponentCreated, EdgeAdded, ApprovalGranted, …).
//!
//! Each `PortfolioEvent` is emitted via `hypergrid::core::EventLog::append`
//! using `EventKind::Custom(kind_tag)` so every portfolio event is
//! automatically captured in the immutable hypergrid audit trail.

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use hypergrid::core::{EventKind as HgEventKind, EventLog as HgEventLog};
use hypergrid::crdt::VectorClock;
use hypergrid::cell::GridId;

use crate::types::ComponentId;

// ── PortfolioEventKind ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortfolioEventKind {
    ComponentCreated,
    ComponentUpdated,
    ComponentRemoved,
    ComponentStatusChanged,
    ComponentStateChanged,
    EdgeAdded,
    EdgeRemoved,
    PolicyAttached,
    PolicyDetached,
    ApprovalRequested,
    ApprovalGranted,
    ApprovalRejected,
    ResourceAllocated,
    ResourceConsumed,
    SnapshotSaved,
    CheckpointCreated,
    StateRestored,
    CrdtMergeApplied,
    ActionPerformed,
    UserAdded,
    UserRemoved,
    ToolboxAttached,
    ToolboxDetached,
    Custom(String),
}

impl PortfolioEventKind {
    /// Convert to a hypergrid EventKind::Custom tag string.
    pub fn to_hg_tag(&self) -> String {
        format!("portfolio:{}", serde_json::to_string(self).unwrap_or_default())
    }
}

// ── PortfolioEvent ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioEvent {
    pub event_id:     Uuid,
    pub event_kind:   PortfolioEventKind,
    pub component_id: Option<ComponentId>,
    /// Node ID of the actor.
    pub actor:        String,
    pub timestamp:    DateTime<Utc>,
    pub payload:      JsonValue,
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
        self.payload = payload; self
    }
}

// ── Snapshot ─────────────────────────────────────────────────────────────────

/// A point-in-time snapshot of the PortfolioSystem component store.
/// Stored both in the portfolio-level snapshot map AND emitted as a
/// PortfolioEvent into the hypergrid EventLog for full audit continuity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub snapshot_id:    Uuid,
    pub label:          String,
    pub note:           Option<String>,
    pub created_at:     DateTime<Utc>,
    pub created_by:     String,
    pub is_checkpoint:  bool,
    pub component_data: JsonValue,
    pub edge_data:      JsonValue,
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

// ── EventLog (portfolio wrapper) ──────────────────────────────────────────────

/// Maximum portfolio domain events kept in the in-process ring buffer.
pub const EVENT_LOG_CAP: usize = 10_000;

/// The portfolio event log.
///
/// Dual-write strategy:
///   1. Every event is appended to the inner `hypergrid::core::EventLog`
///      as `EventKind::Custom(tag)` — making it part of the immutable
///      hypergrid audit trail.
///   2. The portfolio-typed event is kept in the local `domain_log` ring
///      buffer for fast domain queries (by kind, by component, by time).
pub struct EventLog {
    /// Hypergrid event log — the immutable source of truth.
    pub hg_log: HgEventLog,
    /// In-process ring buffer of portfolio-typed events (capped at EVENT_LOG_CAP).
    domain_log: std::collections::VecDeque<PortfolioEvent>,
    /// Named snapshots.
    pub snapshots: HashMap<Uuid, Snapshot>,
    grid_id: GridId,
}

impl EventLog {
    pub fn new(grid_id: GridId) -> Self {
        Self {
            hg_log: HgEventLog::new(),
            domain_log: std::collections::VecDeque::new(),
            snapshots: HashMap::new(),
            grid_id,
        }
    }

    /// Emit a portfolio event. Dual-writes to both logs.
    pub fn emit(&mut self, event: PortfolioEvent) {
        // 1. Write to hypergrid EventLog as an immutable audit entry.
        let vc = VectorClock::new();
        self.hg_log.append(
            HgEventKind::Custom(event.event_kind.to_hg_tag()),
            self.grid_id,
            &event.actor,
            vc,
        );

        // 2. Append to portfolio ring buffer (FIFO eviction at cap).
        if self.domain_log.len() >= EVENT_LOG_CAP {
            self.domain_log.pop_front();
        }
        self.domain_log.push_back(event);
    }

    pub fn len(&self) -> usize { self.domain_log.len() }
    pub fn is_empty(&self) -> bool { self.domain_log.is_empty() }

    pub fn events_of_kind(&self, kind: &PortfolioEventKind) -> Vec<&PortfolioEvent> {
        self.domain_log.iter().filter(|e| &e.event_kind == kind).collect()
    }

    pub fn events_for(&self, component_id: ComponentId) -> Vec<&PortfolioEvent> {
        self.domain_log.iter().filter(|e| e.component_id == Some(component_id)).collect()
    }

    pub fn events_after(&self, since: DateTime<Utc>) -> Vec<&PortfolioEvent> {
        self.domain_log.iter().filter(|e| e.timestamp > since).collect()
    }

    pub fn all_events(&self) -> Vec<&PortfolioEvent> {
        self.domain_log.iter().collect()
    }

    pub fn save_snapshot(&mut self, snap: Snapshot) -> Uuid {
        let id = snap.snapshot_id;
        self.snapshots.insert(id, snap);
        id
    }

    pub fn get_snapshot(&self, id: Uuid) -> Option<&Snapshot> {
        self.snapshots.get(&id)
    }

    pub fn checkpoints(&self) -> Vec<&Snapshot> {
        self.snapshots.values().filter(|s| s.is_checkpoint).collect()
    }

    /// AS_OF time-travel: delegate to hypergrid's EventLog.
    pub fn as_of(&self, timestamp: DateTime<Utc>) -> Vec<&hypergrid::core::EventEntry> {
        self.hg_log.as_of(timestamp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dual_write_and_query() {
        let grid_id = Uuid::new_v4();
        let mut log = EventLog::new(grid_id);
        let cid = Uuid::new_v4();
        log.emit(PortfolioEvent::new(PortfolioEventKind::ComponentCreated, Some(cid), "node-1"));
        log.emit(PortfolioEvent::new(PortfolioEventKind::ComponentUpdated, Some(cid), "node-1"));

        assert_eq!(log.len(), 2);
        assert_eq!(log.hg_log.len(), 2); // hypergrid also received both
        assert_eq!(log.events_for(cid).len(), 2);
    }

    #[test]
    fn ring_buffer_cap() {
        let grid_id = Uuid::new_v4();
        let mut log = EventLog::new(grid_id);
        for _ in 0..EVENT_LOG_CAP + 10 {
            log.emit(PortfolioEvent::new(PortfolioEventKind::ComponentCreated, None, "n"));
        }
        assert_eq!(log.len(), EVENT_LOG_CAP);
    }
}
