//! HG-CRDT — N-dimensional CRDT for distributed consistency.
//!
//! Hypergrid uses per-attribute CRDT semantics:
//! - **LWW** (Last-Write-Wins): Text, Number, Enum, Date, Bool, Relation
//! - **OR-Set**: MultiEnum, Tag, MultiRelation
//! - **GrowOnly Counter**: analytics counters, view counts
//! - **PN-Counter**: budget_spent, allocated_units
//! - **Max Register**: version, sequence_number
//! - **Lattice**: status lifecycle order

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::cell::{AttributeKey, DimKey, TypedAttrValue};
use crate::error::{HypergridResult};

/// Unique node identifier in the federation network.
pub type NodeId = String;

/// The CRDT semantics for a given attribute key.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CrdtSemantics {
    /// Last-Write-Wins by VectorClock timestamp; lexicographic actor tiebreak.
    LastWriteWins,
    /// Observed-Remove Set: concurrent Adds survive; Removes only remove tagged entries.
    OrSet,
    /// Grow-only counter: sum of all increments; never decrements.
    GrowOnlyCounter,
    /// Positive-Negative counter: sum of (positive_ops - negative_ops) per node.
    PnCounter,
    /// Max Register: highest numeric value wins.
    MaxRegister,
    /// Custom lattice: platform-defined partial order (e.g. lifecycle status).
    Lattice(LatticeOrder),
}

/// A custom lattice definition — encodes the partial order for status-style attributes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LatticeOrder {
    /// Ordered list of values from least to greatest.
    pub order: Vec<String>,
}

impl LatticeOrder {
    pub fn new(order: Vec<impl Into<String>>) -> Self {
        Self { order: order.into_iter().map(Into::into).collect() }
    }

    /// Returns the "higher" of two values according to this lattice order.
    pub fn merge<'a>(&self, a: &'a str, b: &'a str) -> &'a str {
        let idx_a = self.order.iter().position(|v| v == a).unwrap_or(0);
        let idx_b = self.order.iter().position(|v| v == b).unwrap_or(0);
        if idx_a >= idx_b { a } else { b }
    }
}

// ─── VectorClock ─────────────────────────────────────────────────────────────

/// Logical clock for causal ordering of mutations across federation nodes.
/// Maps each NodeId to its monotonically increasing counter.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct VectorClock {
    pub counters: HashMap<NodeId, u64>,
}

impl VectorClock {
    /// Create an empty VectorClock.
    pub fn new() -> Self {
        Self::default()
    }

    /// Increment this node's counter and return the new value.
    pub fn tick(&mut self, node_id: &str) -> u64 {
        let counter = self.counters.entry(node_id.to_owned()).or_insert(0);
        *counter += 1;
        *counter
    }

    /// Merge two VectorClocks by taking the element-wise maximum.
    pub fn merge(&mut self, other: &VectorClock) {
        for (node, &count) in &other.counters {
            let entry = self.counters.entry(node.clone()).or_insert(0);
            if count > *entry {
                *entry = count;
            }
        }
    }

    /// Returns `true` if `self` happened-before `other`
    /// (self ≤ other on all nodes and self ≠ other).
    pub fn happened_before(&self, other: &VectorClock) -> bool {
        let mut strictly_less = false;
        for (node, &self_count) in &self.counters {
            let other_count = other.counters.get(node).copied().unwrap_or(0);
            if self_count > other_count {
                return false;
            }
            if self_count < other_count {
                strictly_less = true;
            }
        }
        // Check for nodes that exist only in other
        for node in other.counters.keys() {
            if !self.counters.contains_key(node) {
                strictly_less = true;
            }
        }
        strictly_less
    }

    /// Returns `true` iff neither clock dominates the other.
    pub fn concurrent_with(&self, other: &VectorClock) -> bool {
        !self.happened_before(other) && !other.happened_before(self) && self != other
    }

    /// Returns this node's current counter value.
    pub fn get(&self, node_id: &str) -> u64 {
        self.counters.get(node_id).copied().unwrap_or(0)
    }
}

// ─── OR-Set ──────────────────────────────────────────────────────────────────

/// An observed-remove set entry with a unique tag.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrSetEntry {
    pub value: String,
    /// Unique tag per add-operation, used for precise removal.
    pub tag: Uuid,
    pub added_at: DateTime<Utc>,
    pub added_by: NodeId,
}

/// An Observed-Remove Set (OR-Set) CRDT.
///
/// Invariants:
/// - Concurrent adds always survive.
/// - Removes are tagged: only the exact tagged entry is removed.
/// - `observed` tracks all seen (tag, value) pairs for idempotent re-adds.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrSet {
    pub entries: Vec<OrSetEntry>,
    /// All tombstoned tags (removed entries).
    pub tombstones: Vec<Uuid>,
}

impl OrSet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a value with a fresh tag. Returns the tag UUID.
    pub fn add(&mut self, value: String, node_id: &str) -> Uuid {
        let tag = Uuid::new_v4();
        self.entries.push(OrSetEntry {
            value,
            tag,
            added_at: Utc::now(),
            added_by: node_id.to_owned(),
        });
        tag
    }

    /// Remove an entry by its unique tag.
    pub fn remove(&mut self, tag: Uuid) {
        self.tombstones.push(tag);
        self.entries.retain(|e| e.tag != tag);
    }

    /// Returns all live values (not tombstoned).
    pub fn values(&self) -> Vec<&str> {
        self.entries
            .iter()
            .filter(|e| !self.tombstones.contains(&e.tag))
            .map(|e| e.value.as_str())
            .collect()
    }

    /// Merge another OR-Set into this one (CRDT merge).
    pub fn merge(&mut self, other: &OrSet) {
        for entry in &other.entries {
            if !self.entries.iter().any(|e| e.tag == entry.tag) {
                self.entries.push(entry.clone());
            }
        }
        for &tag in &other.tombstones {
            if !self.tombstones.contains(&tag) {
                self.tombstones.push(tag);
                self.entries.retain(|e| e.tag != tag);
            }
        }
    }
}

// ─── PN-Counter ──────────────────────────────────────────────────────────────

/// Positive-Negative counter. Commutes across concurrent increments and decrements.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PnCounter {
    pub positive: HashMap<NodeId, u64>,
    pub negative: HashMap<NodeId, u64>,
}

impl PnCounter {
    pub fn new() -> Self { Self::default() }

    pub fn increment(&mut self, node_id: &str, amount: u64) {
        *self.positive.entry(node_id.to_owned()).or_insert(0) += amount;
    }

    pub fn decrement(&mut self, node_id: &str, amount: u64) {
        *self.negative.entry(node_id.to_owned()).or_insert(0) += amount;
    }

    pub fn value(&self) -> i64 {
        let pos: u64 = self.positive.values().sum();
        let neg: u64 = self.negative.values().sum();
        pos as i64 - neg as i64
    }

    pub fn merge(&mut self, other: &PnCounter) {
        for (node, &val) in &other.positive {
            let e = self.positive.entry(node.clone()).or_insert(0);
            if val > *e { *e = val; }
        }
        for (node, &val) in &other.negative {
            let e = self.negative.entry(node.clone()).or_insert(0);
            if val > *e { *e = val; }
        }
    }
}

// ─── CRDT Operation ──────────────────────────────────────────────────────────

/// The identifier for a cube cell at a given coordinate.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CellRef {
    pub cube_id: Uuid,
    pub coord: Vec<DimKey>,
}

/// A single CRDT operation that can be applied and merged across nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrdtOperation {
    /// LWW field update: accepted if remote timestamp is newer, or actor sorts higher.
    SetField {
        cell_ref: CellRef,
        attribute: AttributeKey,
        value: TypedAttrValue,
        timestamp: DateTime<Utc>,
        vector_clock: VectorClock,
        actor: NodeId,
    },
    /// OR-Set add: idempotent edge/tag insertion.
    AddToSet {
        cell_ref: CellRef,
        set_name: AttributeKey,
        value: String,
        tag: Uuid,
        timestamp: DateTime<Utc>,
        actor: NodeId,
    },
    /// OR-Set remove: removes by exact tag.
    RemoveFromSet {
        cell_ref: CellRef,
        set_name: AttributeKey,
        tag: Uuid,
        timestamp: DateTime<Utc>,
        actor: NodeId,
    },
    /// Counter increment.
    Increment {
        cell_ref: CellRef,
        attribute: AttributeKey,
        amount: u64,
        actor: NodeId,
    },
    /// PN-Counter decrement.
    Decrement {
        cell_ref: CellRef,
        attribute: AttributeKey,
        amount: u64,
        actor: NodeId,
    },
    /// Schema event: add a new axis or attribute key.
    SchemaAddAxis {
        cube_id: Uuid,
        axis_id: Uuid,
        timestamp: DateTime<Utc>,
        actor: NodeId,
    },
    SchemaAddAttributeKey {
        cube_id: Uuid,
        key: AttributeKey,
        timestamp: DateTime<Utc>,
        actor: NodeId,
    },
}

// ─── CrdtLog ─────────────────────────────────────────────────────────────────

/// In-flight CRDT operation buffer: operations pending application and federation sync.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CrdtLog {
    pub operations: Vec<CrdtOperation>,
    /// NodeId → last synced operation index for delta sync.
    pub sync_cursors: HashMap<NodeId, usize>,
}

impl CrdtLog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a new CRDT operation to the log.
    pub fn append(&mut self, op: CrdtOperation) {
        self.operations.push(op);
    }

    /// Return all operations not yet synced by the given peer.
    pub fn delta_since(&self, peer: &str) -> &[CrdtOperation] {
        let cursor = self.sync_cursors.get(peer).copied().unwrap_or(0);
        &self.operations[cursor..]
    }

    /// Mark that a peer has synced up to the current log tail.
    pub fn mark_synced(&mut self, peer: &str) {
        self.sync_cursors.insert(peer.to_owned(), self.operations.len());
    }

    /// Merge another node's CrdtLog into this one (union of all ops).
    pub fn merge(&mut self, other: &CrdtLog) -> HypergridResult<usize> {
        let start_len = self.operations.len();
        // In a production system you'd deduplicate by op ID; here we append new ops.
        for op in &other.operations {
            self.operations.push(op.clone());
        }
        Ok(self.operations.len() - start_len)
    }

    pub fn len(&self) -> usize {
        self.operations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }
}

// ─── Conflict Record ─────────────────────────────────────────────────────────

/// A recorded CRDT conflict for user review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictRecord {
    pub record_id: Uuid,
    pub cell_ref: CellRef,
    pub attribute: AttributeKey,
    pub winning_value: TypedAttrValue,
    pub losing_value: TypedAttrValue,
    pub winning_actor: NodeId,
    pub losing_actor: NodeId,
    pub detected_at: DateTime<Utc>,
    pub reviewed: bool,
}

// ─── LWW Register ────────────────────────────────────────────────────────────

/// A Last-Write-Wins register holding a single typed value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LwwRegister {
    pub value: TypedAttrValue,
    pub timestamp: DateTime<Utc>,
    pub actor: NodeId,
    pub vector_clock: VectorClock,
}

impl LwwRegister {
    pub fn new(value: TypedAttrValue, actor: &str) -> Self {
        let mut vc = VectorClock::new();
        vc.tick(actor);
        Self {
            value,
            timestamp: Utc::now(),
            actor: actor.to_owned(),
            vector_clock: vc,
        }
    }

    /// Attempt to merge a remote write. Returns `true` if remote wins.
    pub fn merge(&mut self, remote: LwwRegister) -> bool {
        let remote_wins = if remote.timestamp > self.timestamp {
            true
        } else if remote.timestamp == self.timestamp {
            remote.actor > self.actor // lexicographic tiebreak
        } else {
            false
        };
        if remote_wins {
            *self = remote;
        }
        remote_wins
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_clock_happened_before() {
        let mut a = VectorClock::new();
        a.tick("node-1");
        let mut b = a.clone();
        b.tick("node-2");
        assert!(a.happened_before(&b));
        assert!(!b.happened_before(&a));
    }

    #[test]
    fn vector_clock_concurrent() {
        let mut a = VectorClock::new();
        let mut b = VectorClock::new();
        a.tick("node-1");
        b.tick("node-2");
        assert!(a.concurrent_with(&b));
    }

    #[test]
    fn or_set_add_and_remove() {
        let mut s = OrSet::new();
        let tag = s.add("hello".to_owned(), "node-1");
        s.add("world".to_owned(), "node-1");
        assert_eq!(s.values().len(), 2);
        s.remove(tag);
        assert_eq!(s.values(), vec!["world"]);
    }

    #[test]
    fn pn_counter_concurrent() {
        let mut a = PnCounter::new();
        let mut b = PnCounter::new();
        a.increment("node-1", 5);
        b.increment("node-2", 3);
        a.merge(&b);
        assert_eq!(a.value(), 8);
    }
}
