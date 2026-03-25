//! Portfolio CRDT layer — re-exports and wraps hypergrid::crdt.
//!
//! All distributed consistency primitives (VectorClock, CrdtLog, OR-Set,
//! PN-Counter) come directly from the hypergrid substrate. Portfolio-specific
//! CRDT operations extend the base set with domain-aware field names.

// Re-export the hypergrid primitives used directly by the portfolio layer.
pub use hypergrid::crdt::{
    CrdtLog, CrdtSemantics, LwwRegister, NodeId, OrSet, OrSetEntry,
    PnCounter, VectorClock,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::graph::PortfolioEdge;
use crate::types::ComponentId;

// ── Portfolio-level CRDT Operation ───────────────────────────────────────────

/// A portfolio-domain CRDT operation. Translated into hypergrid
/// `CrdtOperation::SetField` / `AddToSet` before being appended to the
/// underlying `hypergrid::crdt::CrdtLog`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrdtOperation {
    /// LWW field update on a Component row in the components Hypercube.
    SetField {
        component_id: ComponentId,
        field: String,
        value: JsonValue,
        timestamp: DateTime<Utc>,
        actor: NodeId,
    },
    /// OR-Set add on a set-valued component field (tags, owners, toolbox_ids).
    AddToSet {
        component_id: ComponentId,
        set_name: String,
        value: JsonValue,
        tag: Uuid,
        timestamp: DateTime<Utc>,
        actor: NodeId,
    },
    /// OR-Set remove by tag.
    RemoveFromSet {
        component_id: ComponentId,
        set_name: String,
        tag: Uuid,
        timestamp: DateTime<Utc>,
        actor: NodeId,
    },
    /// Add a portfolio edge into the Hypergraph.
    AddEdge {
        edge: PortfolioEdge,
        timestamp: DateTime<Utc>,
        actor: NodeId,
    },
    /// Remove a portfolio edge from the Hypergraph.
    RemoveEdge {
        edge_id: Uuid,
        timestamp: DateTime<Utc>,
        actor: NodeId,
    },
}

/// Portfolio-level CRDT log — wraps and extends `hypergrid::crdt::CrdtLog`.
#[derive(Debug, Default)]
pub struct PortfolioCrdtLog {
    pub ops: Vec<CrdtOperation>,
    pub hg_log: CrdtLog,
}

impl PortfolioCrdtLog {
    pub fn new() -> Self { Self::default() }

    pub fn append(&mut self, op: CrdtOperation) {
        self.ops.push(op);
    }

    pub fn len(&self) -> usize { self.ops.len() }
    pub fn is_empty(&self) -> bool { self.ops.is_empty() }

    pub fn delta_since(&self, peer: &str) -> &[CrdtOperation] {
        let cursor = self.hg_log.sync_cursors.get(peer).copied().unwrap_or(0);
        let cursor = cursor.min(self.ops.len());
        &self.ops[cursor..]
    }

    pub fn mark_synced(&mut self, peer: &str) {
        self.hg_log.mark_synced(peer);
    }
}
