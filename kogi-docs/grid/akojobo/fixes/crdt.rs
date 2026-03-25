//! Distributed CRDT layer: VectorClock, CrdtLog, CrdtOperation, federation.

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::graph::GraphEdge;

pub type NodeId = String;

// ── VectorClock ───────────────────────────────────────────────────────────────

/// Logical clock for causal ordering of mutations across federation nodes.
/// Every `ComponentMetadata` carries one.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct VectorClock {
    pub counters: HashMap<NodeId, u64>,
}

impl VectorClock {
    pub fn new() -> Self { Self::default() }

    /// Increment this node's counter; return new value.
    pub fn tick(&mut self, node_id: &str) -> u64 {
        let c = self.counters.entry(node_id.to_owned()).or_insert(0);
        *c += 1;
        *c
    }

    /// Element-wise max merge.
    pub fn merge(&mut self, other: &VectorClock) {
        for (node, &count) in &other.counters {
            let e = self.counters.entry(node.clone()).or_insert(0);
            if count > *e { *e = count; }
        }
    }

    /// True iff self ≤ other on all nodes and self ≠ other.
    pub fn happened_before(&self, other: &VectorClock) -> bool {
        let mut strictly_less = false;
        for (node, &sc) in &self.counters {
            let oc = other.counters.get(node).copied().unwrap_or(0);
            if sc > oc { return false; }
            if sc < oc { strictly_less = true; }
        }
        for node in other.counters.keys() {
            if !self.counters.contains_key(node) { strictly_less = true; }
        }
        strictly_less
    }

    /// True iff neither clock dominates the other.
    pub fn concurrent_with(&self, other: &VectorClock) -> bool {
        !self.happened_before(other) && !other.happened_before(self) && self != other
    }

    pub fn get(&self, node_id: &str) -> u64 {
        self.counters.get(node_id).copied().unwrap_or(0)
    }
}

// ── CrdtOperation ─────────────────────────────────────────────────────────────

/// A single CRDT operation that can be applied to a PortfolioSystem instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrdtOperation {
    /// LWW field update — accepted only if remote timestamp is newer
    /// or actor ID sorts higher on equal timestamps.
    SetField {
        component_id: Uuid,
        field: String,
        value: JsonValue,
        timestamp: DateTime<Utc>,
        actor: NodeId,
    },
    /// OR-Set add — idempotent edge insertion.
    AddEdge {
        edge: GraphEdge,
        timestamp: DateTime<Utc>,
        actor: NodeId,
    },
    /// OR-Set remove — removes by ID.
    RemoveEdge {
        edge_id: Uuid,
        timestamp: DateTime<Utc>,
        actor: NodeId,
    },
    /// OR-Set member add with unique tag.
    AddToSet {
        component_id: Uuid,
        set_name: String,
        value: JsonValue,
        tag: Uuid,
        timestamp: DateTime<Utc>,
        actor: NodeId,
    },
    /// OR-Set member remove by tag.
    RemoveFromSet {
        component_id: Uuid,
        set_name: String,
        tag: Uuid,
        timestamp: DateTime<Utc>,
        actor: NodeId,
    },
}

// ── CrdtLog ───────────────────────────────────────────────────────────────────

/// In-flight CRDT operation buffer.
/// Stores operations pending application and federation peer sync.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CrdtLog {
    pub operations: Vec<CrdtOperation>,
    pub sync_cursors: HashMap<NodeId, usize>,
}

impl CrdtLog {
    pub fn new() -> Self { Self::default() }

    pub fn append(&mut self, op: CrdtOperation) {
        self.operations.push(op);
    }

    /// Return all operations not yet seen by the given peer.
    pub fn delta_since(&self, peer: &str) -> &[CrdtOperation] {
        let cursor = self.sync_cursors.get(peer).copied().unwrap_or(0);
        &self.operations[cursor.min(self.operations.len())..]
    }

    pub fn mark_synced(&mut self, peer: &str) {
        self.sync_cursors.insert(peer.to_owned(), self.operations.len());
    }

    pub fn merge_from(&mut self, other: &CrdtLog) {
        for op in &other.operations {
            self.operations.push(op.clone());
        }
    }

    pub fn len(&self) -> usize { self.operations.len() }
    pub fn is_empty(&self) -> bool { self.operations.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_clock_merge_and_order() {
        let mut a = VectorClock::new();
        a.tick("node-a");
        let mut b = a.clone();
        b.tick("node-b");
        assert!(a.happened_before(&b));
        a.merge(&b);
        assert_eq!(a.get("node-b"), 1);
    }
}
