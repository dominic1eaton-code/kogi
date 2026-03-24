// =============================================================================
// hypergrid::crdt — HG-CRDT: Distributed Consistency Engine
//
// VectorClock, LatticeOrder, CrdtSemantics, CrdtOperation,
// CrdtMergeEngine, CrdtLog, MergeResult, ConflictRecord
// =============================================================================

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::cell::{
    ActorId, AttributeKey, CubeId, DimCoordinate, DimKey, GridId, TypedAttrValue,
};
use crate::error::{CrdtError, HypergridError};

// ─── NodeId ──────────────────────────────────────────────────────────────────

/// NodeId combines federation node identity with identity tag for per-actor causal tracking.
/// Format: "{region}:{instance_id}:{identity_tag}"
/// Example: "node-us-east-1:i-0abc123:@alice-work"
pub type NodeId = String;

// ─── VectorClock ─────────────────────────────────────────────────────────────

/// Logical clock for causal ordering of mutations across federation nodes.
/// HashMap<NodeId, u64>: per-node logical timestamp.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct VectorClock(pub HashMap<NodeId, u64>);

impl VectorClock {
    pub fn new() -> Self { Self(HashMap::new()) }

    /// Advance this node's logical clock. Call BEFORE any write operation.
    /// Returns the new timestamp for this node.
    pub fn tick(&mut self, node_id: &str) -> u64 {
        let counter = self.0.entry(node_id.to_owned()).or_insert(0);
        *counter += 1;
        *counter
    }

    /// Merge two clocks: element-wise maximum.
    /// Call on receiving remote CrdtOperations from a federation peer.
    pub fn merge(&mut self, other: &VectorClock) {
        for (node, &ts) in &other.0 {
            self.0.entry(node.clone())
                .and_modify(|v| *v = (*v).max(ts))
                .or_insert(ts);
        }
    }

    /// Does self causally dominate other?
    /// True if self has seen everything other has seen (and possibly more).
    pub fn dominates(&self, other: &VectorClock) -> bool {
        other.0.iter().all(|(node, &ts)|
            self.0.get(node).copied().unwrap_or(0) >= ts
        )
    }

    /// Are these clocks concurrent? (neither happened-before the other)
    pub fn concurrent_with(&self, other: &VectorClock) -> bool {
        !self.dominates(other) && !other.dominates(self)
    }

    /// Did self happen-before other? (self is strictly older)
    pub fn happened_before(&self, other: &VectorClock) -> bool {
        self.0.iter().all(|(n, &ts)|
            other.0.get(n).copied().unwrap_or(0) >= ts
        ) && self.0 != other.0
    }

    /// Return the subset of (node, ts) pairs in self that are newer than other.
    /// Used for computing federation delta sync payloads.
    pub fn delta_since(&self, other: &VectorClock) -> HashMap<NodeId, u64> {
        self.0.iter()
            .filter(|(node, &ts)| ts > other.0.get(*node).copied().unwrap_or(0))
            .map(|(n, &ts)| (n.clone(), ts))
            .collect()
    }

    pub fn get(&self, node_id: &str) -> u64 {
        self.0.get(node_id).copied().unwrap_or(0)
    }
}

// ─── LatticeNode ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LatticeNode {
    pub state:       String,
    pub successors:  Vec<String>,  // valid forward transitions from this state
    pub is_terminal: bool,         // no successors — final, irreversible state
}

// ─── LatticeOrder ─────────────────────────────────────────────────────────────

/// A directed acyclic graph (DAG) encoding valid lifecycle state transitions.
/// The join(a, b) operation computes the least upper bound (LUB) — the most advanced
/// state reachable from both a and b in the partial order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatticeOrder {
    pub nodes: Vec<LatticeNode>,
}

impl LatticeOrder {
    pub fn new(nodes: Vec<LatticeNode>) -> Self { Self { nodes } }

    /// Is this a valid forward transition from `from` to `to`?
    pub fn is_valid_transition(&self, from: &str, to: &str) -> bool {
        self.nodes.iter()
            .find(|n| n.state == from)
            .map(|n| n.successors.iter().any(|s| s == to))
            .unwrap_or(false)
    }

    /// Successors of a given state.
    pub fn successors(&self, state: &str) -> Vec<&str> {
        self.nodes.iter()
            .find(|n| n.state == state)
            .map(|n| n.successors.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// BFS: can we reach `to` by following successors from `from`?
    pub fn can_reach(&self, from: &str, to: &str) -> bool {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::from([from.to_owned()]);
        while let Some(curr) = queue.pop_front() {
            if curr == to { return true; }
            if visited.insert(curr.clone()) {
                for s in self.successors(&curr) {
                    queue.push_back(s.to_owned());
                }
            }
        }
        false
    }

    /// CRDT join: compute the least upper bound of states a and b.
    /// The result is always a valid state — never a regression.
    ///
    /// join cases:
    ///   a == b              → a  (same state, no conflict)
    ///   b reachable from a  → b  (b is more advanced)
    ///   a reachable from b  → a  (a is more advanced)
    ///   neither dominates   → LUB (lowest common ancestor in reverse DAG)
    ///   no LUB found        → a  (safe fallback: keep current, don't advance)
    pub fn join(&self, a: &str, b: &str) -> String {
        if a == b { return a.to_owned(); }
        if self.can_reach(a, b) { return b.to_owned(); }
        if self.can_reach(b, a) { return a.to_owned(); }
        // Neither dominates: find the LUB (closest state reachable from both)
        self.least_upper_bound(a, b).unwrap_or_else(|| a.to_owned())
    }

    fn least_upper_bound(&self, a: &str, b: &str) -> Option<String> {
        // All states reachable from a (including a itself)
        let reachable_from_a: HashSet<String> = self.all_reachable(a);
        let reachable_from_b: HashSet<String> = self.all_reachable(b);
        let common: HashSet<_> = reachable_from_a.intersection(&reachable_from_b).collect();
        if common.is_empty() { return None; }
        // Find the state in `common` with the fewest reachable states from it
        // (i.e., the one closest to both a and b in the partial order)
        common.into_iter()
            .min_by_key(|s| self.all_reachable(s).len())
            .map(|s| s.clone())
    }

    fn all_reachable(&self, start: &str) -> HashSet<String> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::from([start.to_owned()]);
        while let Some(curr) = queue.pop_front() {
            if visited.insert(curr.clone()) {
                for s in self.successors(&curr) { queue.push_back(s.to_owned()); }
            }
        }
        visited
    }

    // ── Predefined lattices ───────────────────────────────────────────────

    /// Kogi ComponentStatus: Draft → Active ⟷ Paused → Completed → Archived
    pub fn kogi_component_status() -> Self {
        Self::new(vec![
            LatticeNode { state: "Draft".into(),     successors: vec!["Active".into(), "Archived".into()], is_terminal: false },
            LatticeNode { state: "Active".into(),    successors: vec!["Paused".into(), "Completed".into()], is_terminal: false },
            LatticeNode { state: "Paused".into(),    successors: vec!["Active".into(), "Completed".into(), "Archived".into()], is_terminal: false },
            LatticeNode { state: "Completed".into(), successors: vec!["Archived".into()], is_terminal: false },
            LatticeNode { state: "Archived".into(),  successors: vec![], is_terminal: true },
            LatticeNode { state: "Deleted".into(),   successors: vec![], is_terminal: true },
        ])
    }

    /// Ume ModuleLifecycleState: Registered → Starting → Running ⟷ Degraded → Recovering → Stopped
    pub fn ume_module_lifecycle() -> Self {
        Self::new(vec![
            LatticeNode { state: "Registered".into(), successors: vec!["Starting".into()], is_terminal: false },
            LatticeNode { state: "Starting".into(),   successors: vec!["Running".into(), "Stopped".into()], is_terminal: false },
            LatticeNode { state: "Running".into(),    successors: vec!["Degraded".into(), "Stopped".into()], is_terminal: false },
            LatticeNode { state: "Degraded".into(),   successors: vec!["Running".into(), "Recovering".into(), "Stopped".into()], is_terminal: false },
            LatticeNode { state: "Recovering".into(), successors: vec!["Running".into(), "Stopped".into()], is_terminal: false },
            LatticeNode { state: "Stopped".into(),    successors: vec![], is_terminal: true },
        ])
    }

    /// Qala SolutionLifecycleState: Draft → InReview → Approved → Active → Deprecated → Retired
    pub fn qala_solution_lifecycle() -> Self {
        Self::new(vec![
            LatticeNode { state: "Draft".into(),      successors: vec!["InReview".into()], is_terminal: false },
            LatticeNode { state: "InReview".into(),   successors: vec!["Approved".into(), "Draft".into()], is_terminal: false },
            LatticeNode { state: "Approved".into(),   successors: vec!["Active".into()], is_terminal: false },
            LatticeNode { state: "Active".into(),     successors: vec!["Deprecated".into()], is_terminal: false },
            LatticeNode { state: "Deprecated".into(), successors: vec!["Retired".into()], is_terminal: false },
            LatticeNode { state: "Retired".into(),    successors: vec![], is_terminal: true },
        ])
    }

    /// Qala CCR Status lattice
    pub fn qala_ccr_status() -> Self {
        Self::new(vec![
            LatticeNode { state: "Draft".into(),       successors: vec!["Submitted".into()], is_terminal: false },
            LatticeNode { state: "Submitted".into(),   successors: vec!["UnderReview".into()], is_terminal: false },
            LatticeNode { state: "UnderReview".into(), successors: vec!["Approved".into(), "Rejected".into()], is_terminal: false },
            LatticeNode { state: "Approved".into(),    successors: vec!["Implemented".into()], is_terminal: false },
            LatticeNode { state: "Rejected".into(),    successors: vec!["Closed".into()], is_terminal: false },
            LatticeNode { state: "Implemented".into(), successors: vec!["Closed".into()], is_terminal: false },
            LatticeNode { state: "Closed".into(),      successors: vec![], is_terminal: true },
        ])
    }

    /// Qala SDE Maturity stages
    pub fn qala_sde_maturity() -> Self {
        Self::new(vec![
            LatticeNode { state: "SANDBOX".into(), successors: vec!["DEV".into()],     is_terminal: false },
            LatticeNode { state: "DEV".into(),     successors: vec!["NIGHTLY".into()], is_terminal: false },
            LatticeNode { state: "NIGHTLY".into(), successors: vec!["TEST".into()],    is_terminal: false },
            LatticeNode { state: "TEST".into(),    successors: vec!["CM".into()],      is_terminal: false },
            LatticeNode { state: "CM".into(),      successors: vec![],                 is_terminal: true  },
        ])
    }
}

// ─── CrdtSemantics ───────────────────────────────────────────────────────────

/// The CRDT merge strategy for one attribute key.
/// This is the most important design decision per field — determines how concurrent
/// writes from multiple federation nodes are resolved.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CrdtSemantics {
    /// Last-Write-Wins: highest VectorClock timestamp wins. Tiebreak: ActorId lex sort.
    LastWriteWins,

    /// OR-Set: all concurrent Adds survive (each with unique_tag).
    /// Removes target only the specific unique_tag — concurrent Add+Remove: Add wins.
    OrSet,

    /// Grow-Only Counter: only increments allowed; decrement = CrdtError.
    GrowOnlyCounter,

    /// Positive-Negative Counter: supports both increment and decrement.
    PNCounter,

    /// Max Register: highest numeric value always wins, regardless of causal order.
    MaxRegister,

    /// Min Register: lowest numeric value wins.
    MinRegister,

    /// Lattice CRDT: join = least upper bound in the defined partial order.
    Lattice(LatticeOrder),

    /// Append-Only Log: all entries from all nodes preserved; causally ordered.
    AppendLog,

    /// Deep-Merge JSON: LWW applied per leaf key in the JSON tree.
    DeepMergeJson,

    /// Custom merge logic delegated to an AttributeTypePlugin.
    Custom(String), // plugin_id
}

// ─── CrdtOperation ───────────────────────────────────────────────────────────

/// The fundamental mutation unit in Hypergrid.
/// Every state change — user write, federation sync, AI writeback, schema migration —
/// is expressed as a CrdtOperation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrdtOperation {
    // ── Cell attribute mutations ─────────────────────────────────────────

    /// LWW / Lattice / MaxRegister / MinRegister / DeepMerge
    SetAttr {
        coord:     SerializableCoord,
        attr_key:  AttributeKey,
        value:     TypedAttrValue,
        timestamp: VectorClock,
        actor:     ActorId,
    },

    /// OR-Set Add: add an element with a globally unique tag.
    AddToSet {
        coord:      SerializableCoord,
        attr_key:   AttributeKey,
        element:    TypedAttrValue,
        unique_tag: Uuid,           // unique per-add; makes the add idempotent and removable
        actor:      ActorId,
    },

    /// OR-Set Remove: remove only the element with this exact unique_tag.
    RemoveFromSet {
        coord:      SerializableCoord,
        attr_key:   AttributeKey,
        unique_tag: Uuid,
        actor:      ActorId,
    },

    /// Counter increment (positive for PNCounter; must be > 0 for GrowOnly).
    IncrCounter {
        coord:    SerializableCoord,
        attr_key: AttributeKey,
        delta:    i64,              // negative = decrement (PNCounter only)
        node_id:  NodeId,          // per-node counter tracking for PNCounter
    },

    /// Lattice state transition.
    TransitionState {
        coord:      SerializableCoord,
        attr_key:   AttributeKey,
        from_state: String,
        to_state:   String,
        actor:      ActorId,
        timestamp:  VectorClock,
    },

    /// Admin-forced lattice regression (LatticeAdminOverride in EventLog).
    LatticeAdminOverride {
        coord:      SerializableCoord,
        attr_key:   AttributeKey,
        new_state:  String,
        reason:     String,
        admin_id:   ActorId,
        timestamp:  VectorClock,
    },

    /// Append to an AppendLog attribute.
    AppendLog {
        coord:     SerializableCoord,
        attr_key:  AttributeKey,
        entry:     TypedAttrValue,
        causal_ts: VectorClock,
        actor:     ActorId,
    },

    /// Deep-merge a JSON delta into a Json attribute.
    DeepMergeJson {
        coord:     SerializableCoord,
        attr_key:  AttributeKey,
        delta:     serde_json::Value,  // only changed leaf keys
        timestamp: VectorClock,
        actor:     ActorId,
    },

    // ── Schema mutations ─────────────────────────────────────────────────

    /// Add a new DimensionAxis to a Hypercube (zero-downtime, additive).
    AddDimension {
        cube_id:   CubeId,
        axis:      crate::dim::DimensionAxis,
        schema_ts: u64,
    },

    /// Register a new AttributeKeyDef (zero-downtime, additive).
    RegisterAttrKey {
        cube_id:   CubeId,
        attr_def:  crate::cell::AttributeKeyDef,
        schema_ts: u64,
    },

    /// Tombstone an attribute key (requires governance gate).
    TombstoneAttrKey {
        cube_id:   CubeId,
        attr_key:  AttributeKey,
        schema_ts: u64,
    },

    // ── Graph mutations ──────────────────────────────────────────────────

    AddEdge    { edge: crate::graph::HypergraphEdge },
    RemoveEdge { edge_id: Uuid },

    // ── Namespace mutations ──────────────────────────────────────────────

    RegisterNamespace { path: String, entity_ref: crate::cell::EntityRef },
    AddFederationPeer { peer: crate::space::FederationPeer },
}

/// Serializable coordinate used inside CrdtOperations (avoids borrowing issues).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableCoord {
    pub grid_id: GridId,
    pub cube_id: CubeId,
    pub keys:    Vec<DimKey>,
}

impl From<&DimCoordinate> for SerializableCoord {
    fn from(c: &DimCoordinate) -> Self {
        Self { grid_id: c.grid_id, cube_id: c.cube_id, keys: c.keys.clone() }
    }
}

impl SerializableCoord {
    pub fn to_dim_coordinate(&self) -> DimCoordinate {
        DimCoordinate { grid_id: self.grid_id, cube_id: self.cube_id, keys: self.keys.clone() }
    }
}

// ─── MergeResult ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum MergeResult {
    Applied,
    ConflictRecorded { kept: String },   // "current" or "incoming"
    DiscardedStale,                       // incoming op is causally older than current
}

// ─── ConflictKind ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictKind {
    LwwLosing  { kept_actor: ActorId },
    LatticeInvalid { attempted_from: String, attempted_to: String },
    GrowOnlyViolation,
    SchemaVersionConflict,
}

// ─── ConflictRecord ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictRecord {
    pub conflict_id: Uuid,
    pub cube_id:     CubeId,
    pub coord:       SerializableCoord,
    pub attr_key:    AttributeKey,
    pub kind:        ConflictKind,
    pub winner_value: TypedAttrValue,
    pub loser_value:  TypedAttrValue,
    pub winner_actor: ActorId,
    pub loser_actor:  ActorId,
    pub detected_at:  DateTime<Utc>,
    pub resolved:     bool,
}

// ─── OrSetEntry ──────────────────────────────────────────────────────────────

/// One element in an OR-Set attribute value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrSetEntry {
    pub unique_tag: Uuid,
    pub element:    TypedAttrValue,
    pub added_by:   ActorId,
    pub added_at:   DateTime<Utc>,
    pub removed:    bool,           // soft-delete flag; OR-Set semantics preserve the record
}

// ─── PerNodeCounter ──────────────────────────────────────────────────────────

/// Per-node counter state for GrowOnly and PNCounter CRDTs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerNodeCounter {
    pub positive: HashMap<NodeId, i64>,
    pub negative: HashMap<NodeId, i64>,  // only used for PNCounter
}

impl PerNodeCounter {
    pub fn total(&self) -> i64 {
        let pos: i64 = self.positive.values().sum();
        let neg: i64 = self.negative.values().sum();
        pos - neg
    }

    pub fn incr(&mut self, node_id: &str, delta: i64, grow_only: bool) -> Result<(), CrdtError> {
        if grow_only && delta <= 0 {
            return Err(CrdtError::GrowOnlyViolation);
        }
        if delta > 0 {
            *self.positive.entry(node_id.to_owned()).or_insert(0) += delta;
        } else {
            *self.negative.entry(node_id.to_owned()).or_insert(0) += delta.unsigned_abs() as i64;
        }
        Ok(())
    }

    /// Merge: element-wise max for each node's counts.
    pub fn merge(&mut self, other: &PerNodeCounter) {
        for (n, &v) in &other.positive {
            let e = self.positive.entry(n.clone()).or_insert(0);
            *e = (*e).max(v);
        }
        for (n, &v) in &other.negative {
            let e = self.negative.entry(n.clone()).or_insert(0);
            *e = (*e).max(v);
        }
    }
}

// ─── CrdtMergeEngine ─────────────────────────────────────────────────────────

/// The heart of HG-CRDT. Applies CrdtOperations deterministically regardless of arrival order.
/// The key invariant: the same set of operations applied in any order produces the same state.
pub struct CrdtMergeEngine {
    pub conflicts: std::sync::Mutex<Vec<ConflictRecord>>,
}

impl CrdtMergeEngine {
    pub fn new() -> Self {
        Self { conflicts: std::sync::Mutex::new(Vec::new()) }
    }

    /// Apply one CrdtOperation against an in-memory CellStore.
    ///
    /// The store is a simple HashMap for this in-memory implementation.
    /// Production code uses PostgreSQL via the CellStoreBackend trait.
    pub fn apply_op(
        &self,
        op: &CrdtOperation,
        cells:    &mut HashMap<(CubeId, Vec<DimKey>, AttributeKey), TypedAttrValue>,
        or_sets:  &mut HashMap<(CubeId, Vec<DimKey>, AttributeKey), Vec<OrSetEntry>>,
        counters: &mut HashMap<(CubeId, Vec<DimKey>, AttributeKey), PerNodeCounter>,
        clocks:   &mut HashMap<(CubeId, Vec<DimKey>, AttributeKey), VectorClock>,
        actors:   &mut HashMap<(CubeId, Vec<DimKey>, AttributeKey), ActorId>,
        crdt_cfg: &HashMap<(CubeId, AttributeKey), CrdtSemantics>,
    ) -> Result<MergeResult, CrdtError> {
        match op {
            CrdtOperation::SetAttr { coord, attr_key, value, timestamp, actor } => {
                let k = (coord.cube_id, coord.keys.clone(), attr_key.clone());
                let semantics = crdt_cfg.get(&(coord.cube_id, attr_key.clone()))
                    .unwrap_or(&CrdtSemantics::LastWriteWins);

                match semantics {
                    CrdtSemantics::LastWriteWins => {
                        let current_ts = clocks.get(&k).cloned().unwrap_or_default();
                        if timestamp.dominates(&current_ts) {
                            cells.insert(k.clone(), value.clone());
                            clocks.insert(k.clone(), timestamp.clone());
                            actors.insert(k, actor.clone());
                            Ok(MergeResult::Applied)
                        } else if timestamp.concurrent_with(&current_ts) {
                            // Deterministic tiebreak: higher ActorId wins
                            let current_actor = actors.get(&k).cloned().unwrap_or_default();
                            if actor.as_str() > current_actor.as_str() {
                                cells.insert(k.clone(), value.clone());
                                clocks.insert(k.clone(), timestamp.clone());
                                actors.insert(k, actor.clone());
                                Ok(MergeResult::Applied)
                            } else {
                                self.record_conflict(ConflictRecord {
                                    conflict_id:  Uuid::new_v4(),
                                    cube_id:      coord.cube_id,
                                    coord:        coord.clone(),
                                    attr_key:     attr_key.clone(),
                                    kind:         ConflictKind::LwwLosing { kept_actor: current_actor },
                                    winner_value: cells.get(&k).cloned().unwrap_or(TypedAttrValue::Null),
                                    loser_value:  value.clone(),
                                    winner_actor: actors.get(&k).cloned().unwrap_or_default(),
                                    loser_actor:  actor.clone(),
                                    detected_at:  Utc::now(),
                                    resolved:     false,
                                });
                                Ok(MergeResult::ConflictRecorded { kept: "current".into() })
                            }
                        } else {
                            Ok(MergeResult::DiscardedStale)
                        }
                    }

                    CrdtSemantics::MaxRegister => {
                        let current = cells.get(&k).and_then(|v| v.as_f64()).unwrap_or(f64::NEG_INFINITY);
                        let incoming = value.as_f64().unwrap_or(f64::NEG_INFINITY);
                        if incoming > current {
                            cells.insert(k, value.clone());
                        }
                        Ok(MergeResult::Applied)
                    }

                    CrdtSemantics::MinRegister => {
                        let current = cells.get(&k).and_then(|v| v.as_f64()).unwrap_or(f64::INFINITY);
                        let incoming = value.as_f64().unwrap_or(f64::INFINITY);
                        if incoming < current {
                            cells.insert(k, value.clone());
                        }
                        Ok(MergeResult::Applied)
                    }

                    CrdtSemantics::Lattice(lattice) => {
                        let current_state = if let Some(TypedAttrValue::Text(s)) = cells.get(&k) {
                            s.clone()
                        } else { String::new() };
                        let new_state = value.as_str().unwrap_or("").to_owned();
                        if current_state.is_empty() {
                            cells.insert(k, TypedAttrValue::Text(new_state));
                        } else {
                            let merged = lattice.join(&current_state, &new_state);
                            cells.insert(k, TypedAttrValue::Text(merged));
                        }
                        Ok(MergeResult::Applied)
                    }

                    CrdtSemantics::DeepMergeJson => {
                        if let TypedAttrValue::Json(delta) = value {
                            let current = cells.entry(k.clone())
                                .or_insert_with(|| TypedAttrValue::Json(serde_json::Value::Object(Default::default())));
                            if let TypedAttrValue::Json(base) = current {
                                deep_merge_json(base, delta);
                            }
                        }
                        Ok(MergeResult::Applied)
                    }

                    _ => {
                        // Fallback: LWW for unhandled semantics
                        cells.insert(k, value.clone());
                        Ok(MergeResult::Applied)
                    }
                }
            }

            CrdtOperation::AddToSet { coord, attr_key, element, unique_tag, actor } => {
                let k = (coord.cube_id, coord.keys.clone(), attr_key.clone());
                let entries = or_sets.entry(k).or_default();
                // OR-Set: all Adds survive — no conflict possible
                if !entries.iter().any(|e| e.unique_tag == *unique_tag) {
                    entries.push(OrSetEntry {
                        unique_tag: *unique_tag,
                        element: element.clone(),
                        added_by: actor.clone(),
                        added_at: Utc::now(),
                        removed: false,
                    });
                }
                Ok(MergeResult::Applied)
            }

            CrdtOperation::RemoveFromSet { coord, attr_key, unique_tag, .. } => {
                let k = (coord.cube_id, coord.keys.clone(), attr_key.clone());
                if let Some(entries) = or_sets.get_mut(&k) {
                    if let Some(entry) = entries.iter_mut().find(|e| e.unique_tag == *unique_tag) {
                        entry.removed = true;
                    }
                }
                Ok(MergeResult::Applied)
            }

            CrdtOperation::IncrCounter { coord, attr_key, delta, node_id } => {
                let k = (coord.cube_id, coord.keys.clone(), attr_key.clone());
                let semantics = crdt_cfg.get(&(coord.cube_id, attr_key.clone()))
                    .unwrap_or(&CrdtSemantics::GrowOnlyCounter);
                let grow_only = matches!(semantics, CrdtSemantics::GrowOnlyCounter);
                let counter = counters.entry(k).or_default();
                counter.incr(node_id, *delta, grow_only)
                    .map(|_| MergeResult::Applied)
            }

            CrdtOperation::TransitionState { coord, attr_key, from_state, to_state, actor, timestamp } => {
                let k = (coord.cube_id, coord.keys.clone(), attr_key.clone());
                let semantics = crdt_cfg.get(&(coord.cube_id, attr_key.clone()))
                    .unwrap_or(&CrdtSemantics::LastWriteWins);
                if let CrdtSemantics::Lattice(lattice) = semantics {
                    if !lattice.is_valid_transition(from_state, to_state) {
                        return Err(CrdtError::InvalidStateTransition {
                            from: from_state.clone(), to: to_state.clone()
                        });
                    }
                    let current = if let Some(TypedAttrValue::Text(s)) = cells.get(&k) {
                        s.clone()
                    } else { from_state.clone() };
                    let merged = lattice.join(&current, to_state);
                    cells.insert(k.clone(), TypedAttrValue::Text(merged));
                    clocks.insert(k.clone(), timestamp.clone());
                    actors.insert(k, actor.clone());
                }
                Ok(MergeResult::Applied)
            }

            CrdtOperation::LatticeAdminOverride { coord, attr_key, new_state, actor, timestamp, .. } => {
                // Admin override: force-set state regardless of lattice order
                // Requires PermissionTier::Admin — enforced at write path, not here
                let k = (coord.cube_id, coord.keys.clone(), attr_key.clone());
                cells.insert(k.clone(), TypedAttrValue::Text(new_state.clone()));
                clocks.insert(k.clone(), timestamp.clone());
                actors.insert(k, actor.clone());
                Ok(MergeResult::Applied)
            }

            CrdtOperation::AppendLog { coord, attr_key, entry, causal_ts, actor } => {
                // Append-only: all entries survive; order by causal_ts
                let k = (coord.cube_id, coord.keys.clone(), attr_key.clone());
                let existing = cells.entry(k).or_insert(TypedAttrValue::Json(serde_json::json!([])));
                if let TypedAttrValue::Json(arr) = existing {
                    if let Some(vec) = arr.as_array_mut() {
                        vec.push(serde_json::json!({
                            "entry": serde_json::to_value(entry).unwrap_or_default(),
                            "actor": actor,
                            "ts":    causal_ts.0.values().max().copied().unwrap_or(0),
                        }));
                    }
                }
                Ok(MergeResult::Applied)
            }

            CrdtOperation::DeepMergeJson { coord, attr_key, delta, .. } => {
                let k = (coord.cube_id, coord.keys.clone(), attr_key.clone());
                let current = cells.entry(k)
                    .or_insert_with(|| TypedAttrValue::Json(serde_json::Value::Object(Default::default())));
                if let TypedAttrValue::Json(base) = current {
                    deep_merge_json(base, delta);
                }
                Ok(MergeResult::Applied)
            }

            // Schema and graph ops: handled at Grid level, not cell level
            _ => Ok(MergeResult::Applied),
        }
    }

    fn record_conflict(&self, record: ConflictRecord) {
        if let Ok(mut conflicts) = self.conflicts.lock() {
            conflicts.push(record);
        }
    }

    pub fn take_conflicts(&self) -> Vec<ConflictRecord> {
        self.conflicts.lock().map(|mut c| std::mem::take(&mut *c)).unwrap_or_default()
    }
}

/// Recursively merge `delta` into `base` at the leaf-key level (LWW per leaf).
fn deep_merge_json(base: &mut serde_json::Value, delta: &serde_json::Value) {
    if let (serde_json::Value::Object(b), serde_json::Value::Object(d)) = (base, delta) {
        for (k, v) in d {
            if v.is_object() {
                let entry = b.entry(k.clone()).or_insert(serde_json::Value::Object(Default::default()));
                deep_merge_json(entry, v);
            } else {
                b.insert(k.clone(), v.clone());
            }
        }
    }
}

// ─── CrdtLog ─────────────────────────────────────────────────────────────────

/// Buffer of CrdtOperations for federation delta sync.
/// Hot path: in-memory VecDeque. Background: flushed to PostgreSQL.
#[derive(Debug)]
pub struct CrdtLogEntry {
    pub op:          CrdtOperation,
    pub applied:     bool,
    pub causal_ts:   VectorClock,
    pub sync_cursors: HashMap<NodeId, u64>,  // which peers have ACKed this op
    pub created_at:  DateTime<Utc>,
}

pub struct CrdtLog {
    pub grid_id:    GridId,
    hot:            std::sync::Mutex<VecDeque<CrdtLogEntry>>,
    pub seq:        AtomicU64,
    pub max_hot:    usize,
}

impl CrdtLog {
    pub fn new(grid_id: GridId) -> Self {
        Self { grid_id, hot: Default::default(), seq: AtomicU64::new(0), max_hot: 50_000 }
    }

    pub fn append(&self, op: CrdtOperation, vc: VectorClock) {
        let entry = CrdtLogEntry {
            op, applied: true, causal_ts: vc,
            sync_cursors: HashMap::new(), created_at: Utc::now(),
        };
        if let Ok(mut log) = self.hot.lock() {
            log.push_back(entry);
            if log.len() > self.max_hot {
                log.pop_front(); // overflow: oldest entries dropped (persisted to PG in production)
            }
        }
    }

    /// Return all operations since the peer's last known VectorClock position.
    pub fn delta_since(&self, peer_vc: &VectorClock) -> Vec<CrdtOperation> {
        self.hot.lock()
            .map(|log| log.iter()
                .filter(|e| {
                    // Include if this op's causal_ts has entries newer than peer_vc
                    e.causal_ts.0.iter().any(|(node, &ts)|
                        ts > peer_vc.0.get(node).copied().unwrap_or(0)
                    )
                })
                .map(|e| e.op.clone())
                .collect())
            .unwrap_or_default()
    }
}
