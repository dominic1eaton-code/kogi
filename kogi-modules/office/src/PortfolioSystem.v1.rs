// PortfolioSystem.rs
// Unified, complete implementation reconciling all prior versions.
//
// Sections:
//   1.  Imports & utilities
//   2.  ComponentMetadata  (version control, resource tracking, unique IDs)
//   3.  Component taxonomy (PortfolioComponentType, BookType, PortfolioComponent)
//   4.  Graph layer        (GraphEdge, GraphView – DAG / multi-relation)
//   5.  Event sourcing     (PortfolioEventKind, PortfolioEvent, EventLog)
//   6.  Snapshot / checkpoint
//   7.  CRDT               (VectorClock, LwwField, OrSet, CrdtOperation, CrdtLog)
//   8.  Governance         (Policy, PolicyEngine trait, ApprovalWorkflow, ResourceAllocation)
//   9.  Plugin traits      (PortfolioPlugin)
//  10.  Federation         (FederationPeer, PortfolioFederation)
//  11.  Query language     (PortfolioQuery / PQL)
//  12.  PortfolioSystem    (main runtime — CRUD, graph, snapshots, time-travel, federation)
//  13.  os_bridge adapter  (NewPortfolioItem, parse_entity_type — backward-compat shim)
//  14.  Computational models (per-component-type analytics & scoring engines)

#![allow(dead_code)]

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::os_bridge::{
    parse_portfolio_kind, PortfolioEntityType, PortfolioItemContainerType, PortfolioItemType,
    ResourceType,
};

// =============================================================================
// §1 — UTILITIES
// =============================================================================

/// Unix epoch seconds.
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Generate a short, unique tag for OR-Set entries.
fn unique_tag(actor: &str, ts: u64, seq: u64) -> String {
    format!("{actor}:{ts}:{seq}")
}

// =============================================================================
// §2 — COMPONENT METADATA
// =============================================================================

/// Canonical metadata carried by every `PortfolioComponent`.
///
/// Serves as the foundation for:
///  * Version control  — `version`, `vector_clock`
///  * Unique identity  — `id`, `actor_id`
///  * Resource tracking — `budget`, `budget_spent`, `resource_units`
///  * Governance       — `owner`, `policy_ids`
///  * Searchability    — `tags`, `properties`
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComponentMetadata {
    /// Globally unique component identifier (e.g. `"comp-0042"`).
    pub id: String,
    /// Monotonic integer version; incremented on every mutation.
    pub version: u64,
    /// Epoch-seconds of first creation.
    pub created_at: u64,
    /// Epoch-seconds of last mutation.
    pub updated_at: u64,
    /// Actor (peer / user) that last wrote this component.
    pub actor_id: String,
    /// Causal vector clock: actor → logical time.
    pub vector_clock: HashMap<String, u64>,
    /// Arbitrary string tags for search and governance.
    pub tags: Vec<String>,
    /// Open-ended key→value bag (description, priority, custom fields …).
    pub properties: HashMap<String, String>,
    // ── Resource management ───────────────────────────────────────────────
    /// Optional human owner of this component.
    pub owner: Option<String>,
    /// Allocated budget (arbitrary unit; interpret at application layer).
    pub budget: Option<f64>,
    /// Consumed budget so far.
    pub budget_spent: f64,
    /// Generic resource units (e.g. person-hours, story-points).
    pub resource_units: f64,
    /// IDs of governance policies that apply to this component.
    pub policy_ids: Vec<String>,
}

impl ComponentMetadata {
    pub fn new(id: impl Into<String>, actor_id: impl Into<String>) -> Self {
        let ts = now();
        Self {
            id: id.into(),
            version: 1,
            created_at: ts,
            updated_at: ts,
            actor_id: actor_id.into(),
            vector_clock: HashMap::new(),
            tags: Vec::new(),
            properties: HashMap::new(),
            owner: None,
            budget: None,
            budget_spent: 0.0,
            resource_units: 0.0,
            policy_ids: Vec::new(),
        }
    }

    pub fn touch(&mut self, actor_id: &str) {
        self.version += 1;
        self.updated_at = now();
        self.actor_id = actor_id.to_string();
        let entry = self.vector_clock.entry(actor_id.to_string()).or_insert(0);
        *entry += 1;
    }
}

// =============================================================================
// §3 — COMPONENT TAXONOMY
// =============================================================================

/// Every first-class object in the portfolio system.
///
/// Items  : `Portfolio`, `Project`, `Program`, `Resource`, `Asset`,
///          `Artifact`, `SubPortfolio`
/// Containers: `Binder`, `Book`, `Folder`, `Record`
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortfolioComponentType {
    // ── Items ─────────────────────────────────────────────────────────────
    Portfolio,
    Project,
    Program,
    Resource,
    Asset,
    Artifact,
    SubPortfolio,
    // ── Containers ────────────────────────────────────────────────────────
    /// Unordered collection of heterogeneous components.
    Binder,
    /// Typed document container; sub-type stored in `book_type`.
    Book,
    /// Hierarchical file-system-style container (ordered by insertion or name).
    Folder,
    /// Ordered, sequential list of components.
    Record,
}

/// Sub-types for `PortfolioComponentType::Book`.
///
/// * `Notebook`     — general notes / wiki pages
/// * `Playbook`     — runbooks, procedures, playbooks
/// * `Contactbook`  — people, stakeholders, organisations
/// * `Schedulebook` — timelines, milestones, calendar events
/// * `Itembook`     — complete dossier for one portfolio item:
///                    dependencies, linked items, charters, files, documents
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BookType {
    Notebook,
    Playbook,
    Contactbook,
    Schedulebook,
    Itembook,
}

/// The universal base type for every portfolio object.
///
/// # Container semantics
/// | Type      | Ordering | Deduplication |
/// |-----------|----------|---------------|
/// | Binder    | none     | yes (HashSet) |
/// | Record    | strict   | no  (Vec)     |
/// | Folder    | by-name  | yes (HashSet) |
/// | Book      | loose    | yes (HashSet) |
///
/// `unordered_members` is used by Binder / Folder / Book.
/// `ordered_members`   is used by Record (and optionally Folder when
///                     insertion-order matters).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortfolioComponent {
    pub metadata: ComponentMetadata,
    pub component_type: PortfolioComponentType,
    /// Optional sub-type for `Book` components.
    pub book_type: Option<BookType>,
    pub name: String,
    pub status: String,

    // ── Graph relationships ───────────────────────────────────────────────
    /// Directed hierarchy: IDs of direct children.
    pub children: HashSet<String>,
    /// Directed hierarchy: IDs of direct parents.
    pub parents: HashSet<String>,
    /// Bidirectional peer associations (non-hierarchical links).
    pub links: HashSet<String>,
    /// Directed dependencies: this component depends on these IDs.
    pub dependencies: HashSet<String>,
    /// Components that depend on this one (reverse index of `dependencies`).
    pub dependents: HashSet<String>,

    // ── Container membership ─────────────────────────────────────────────
    /// For Binder / Folder / Book: unordered set of member component IDs.
    pub unordered_members: HashSet<String>,
    /// For Record (and ordered Folder): strictly-ordered member component IDs.
    pub ordered_members: Vec<String>,

    // ── Legacy container-type list (from os_bridge) ───────────────────────
    /// Raw `PortfolioItemContainerType` values retained for os_bridge compat.
    pub container_refs: Vec<PortfolioItemContainerType>,
}

impl PortfolioComponent {
    /// Create a new component with sensible defaults.
    pub fn new(
        id: impl Into<String>,
        actor_id: impl Into<String>,
        component_type: PortfolioComponentType,
        name: impl Into<String>,
    ) -> Self {
        let id = id.into();
        let actor_id = actor_id.into();
        Self {
            metadata: ComponentMetadata::new(id, &actor_id),
            component_type,
            book_type: None,
            name: name.into(),
            status: "active".to_string(),
            children: HashSet::new(),
            parents: HashSet::new(),
            links: HashSet::new(),
            dependencies: HashSet::new(),
            dependents: HashSet::new(),
            unordered_members: HashSet::new(),
            ordered_members: Vec::new(),
            container_refs: Vec::new(),
        }
    }

    /// True if this component is a container type.
    pub fn is_container(&self) -> bool {
        matches!(
            self.component_type,
            PortfolioComponentType::Binder
                | PortfolioComponentType::Book
                | PortfolioComponentType::Folder
                | PortfolioComponentType::Record
        )
    }

    /// True if this component is a portfolio item type.
    pub fn is_item(&self) -> bool {
        !self.is_container()
    }

    /// All direct member IDs regardless of ordering semantics.
    pub fn all_members(&self) -> impl Iterator<Item = &str> {
        self.unordered_members
            .iter()
            .map(String::as_str)
            .chain(self.ordered_members.iter().map(String::as_str))
    }

    /// Add a member.  Uses ordered_members for `Record`, unordered for others.
    pub fn add_member(&mut self, id: impl Into<String>) {
        let id = id.into();
        if self.component_type == PortfolioComponentType::Record {
            self.ordered_members.push(id);
        } else {
            self.unordered_members.insert(id);
        }
    }

    /// Remove a member from either list.
    pub fn remove_member(&mut self, id: &str) -> bool {
        let from_unordered = self.unordered_members.remove(id);
        let before = self.ordered_members.len();
        self.ordered_members.retain(|m| m != id);
        from_unordered || self.ordered_members.len() < before
    }
}

// =============================================================================
// §4 — GRAPH LAYER
// =============================================================================

/// Typed relationship between two components (directed edge).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GraphEdgeKind {
    /// Strict parent→child ownership hierarchy.
    Hierarchy,
    /// `from` depends on `to` before it can proceed.
    Dependency,
    /// Peer association (non-hierarchical, non-dependency).
    Link,
    /// Container membership (`from` contains `to`).
    Contains,
    /// Cross-portfolio federation bridge.
    Federation,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub from: String,
    pub to: String,
    pub kind: GraphEdgeKind,
    pub label: Option<String>,
    pub created_at: u64,
    pub properties: HashMap<String, String>,
}

impl GraphEdge {
    pub fn new(
        id: impl Into<String>,
        from: impl Into<String>,
        to: impl Into<String>,
        kind: GraphEdgeKind,
    ) -> Self {
        Self {
            id: id.into(),
            from: from.into(),
            to: to.into(),
            kind,
            label: None,
            created_at: now(),
            properties: HashMap::new(),
        }
    }
}

/// Immutable read-only view over a component graph.
pub struct GraphView<'a> {
    components: &'a HashMap<String, PortfolioComponent>,
    edges: &'a Vec<GraphEdge>,
}

impl<'a> GraphView<'a> {
    pub fn new(
        components: &'a HashMap<String, PortfolioComponent>,
        edges: &'a Vec<GraphEdge>,
    ) -> Self {
        Self { components, edges }
    }

    /// All edges going *out* of `id`.
    pub fn outgoing(&self, id: &str) -> Vec<&GraphEdge> {
        self.edges.iter().filter(|e| e.from == id).collect()
    }

    /// All edges coming *into* `id`.
    pub fn incoming(&self, id: &str) -> Vec<&GraphEdge> {
        self.edges.iter().filter(|e| e.to == id).collect()
    }

    /// Transitive reachability from `start` following `kind` edges.
    pub fn reachable(&self, start: &str, kind: &GraphEdgeKind) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(start.to_string());
        while let Some(node) = queue.pop_front() {
            if visited.contains(&node) {
                continue;
            }
            visited.insert(node.clone());
            for edge in self.edges.iter().filter(|e| e.from == node && &e.kind == kind) {
                queue.push_back(edge.to.clone());
            }
        }
        visited.into_iter().filter(|id| id != start).collect()
    }

    /// Kahn's algorithm topological sort over `kind` edges.
    /// Returns `Err` if a cycle is detected.
    pub fn topological_sort(&self, kind: &GraphEdgeKind) -> Result<Vec<String>, String> {
        let relevant: Vec<&GraphEdge> =
            self.edges.iter().filter(|e| &e.kind == kind).collect();

        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        for c in self.components.keys() {
            in_degree.entry(c.as_str()).or_insert(0);
        }
        for e in &relevant {
            *in_degree.entry(e.to.as_str()).or_insert(0) += 1;
        }

        let mut queue: VecDeque<&str> = in_degree
            .iter()
            .filter(|(_, &d)| d == 0)
            .map(|(&k, _)| k)
            .collect();

        let mut sorted = Vec::new();
        while let Some(node) = queue.pop_front() {
            sorted.push(node.to_string());
            for e in relevant.iter().filter(|e| e.from == node) {
                let deg = in_degree.entry(e.to.as_str()).or_default();
                *deg = deg.saturating_sub(1);
                if *deg == 0 {
                    queue.push_back(e.to.as_str());
                }
            }
        }

        if sorted.len() == self.components.len() {
            Ok(sorted)
        } else {
            Err("Cycle detected in dependency graph".to_string())
        }
    }

    /// Detect whether adding `from→to` of `kind` would introduce a cycle.
    pub fn would_cycle(&self, from: &str, to: &str, kind: &GraphEdgeKind) -> bool {
        self.reachable(to, kind).contains(&from.to_string())
    }
}

// =============================================================================
// §5 — EVENT SOURCING
// =============================================================================

/// Every mutation the system can undergo.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PortfolioEventKind {
    // Components
    ComponentCreated,
    ComponentUpdated,
    ComponentRemoved,
    ComponentStatusChanged,
    // Graph
    EdgeAdded,
    EdgeRemoved,
    DependencyAdded,
    DependencyRemoved,
    HierarchyLinked,
    HierarchyUnlinked,
    MemberAdded,
    MemberRemoved,
    // Snapshots
    SnapshotSaved,
    CheckpointCreated,
    StateRestored,
    // Governance
    PolicyAttached,
    PolicyDetached,
    ApprovalRequested,
    ApprovalGranted,
    ApprovalRejected,
    ResourceAllocated,
    ResourceConsumed,
    /// Kept for backward-compat event-log consumers; maps to ResourceAllocated.
    BudgetAllocated,
    /// Kept for backward-compat event-log consumers; maps to ResourceConsumed.
    BudgetSpent,
    // Federation / CRDT
    PeerRegistered,
    PeerRemoved,
    CrdtMergeApplied,
    EventStreamPublished,
}

/// An immutable record of one state transition.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortfolioEvent {
    pub id: String,
    pub sequence: u64,
    pub timestamp: u64,
    pub actor_id: String,
    pub kind: PortfolioEventKind,
    /// Structured payload.  Keys/values are event-kind-specific.
    pub payload: HashMap<String, String>,
}

/// Append-only ordered event log.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EventLog {
    events: Vec<PortfolioEvent>,
    next_sequence: u64,
}

impl EventLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(
        &mut self,
        actor_id: impl Into<String>,
        kind: PortfolioEventKind,
        payload: HashMap<String, String>,
    ) -> &PortfolioEvent {
        let seq = self.next_sequence;
        self.next_sequence += 1;
        self.events.push(PortfolioEvent {
            id: format!("evt-{seq:06}"),
            sequence: seq,
            timestamp: now(),
            actor_id: actor_id.into(),
            kind,
            payload,
        });
        self.events.last().unwrap()
    }

    pub fn all(&self) -> &[PortfolioEvent] {
        &self.events
    }

    /// Events with timestamp ≤ `until` (inclusive).
    pub fn up_to(&self, until: u64) -> impl Iterator<Item = &PortfolioEvent> {
        self.events.iter().filter(move |e| e.timestamp <= until)
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

// =============================================================================
// §6 — SNAPSHOT & CHECKPOINT
// =============================================================================

/// Complete, serialisable picture of the system at an instant.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortfolioSnapshot {
    pub snapshot_id: String,
    pub label: Option<String>,
    pub created_at: u64,
    pub components: HashMap<String, PortfolioComponent>,
    pub edges: Vec<GraphEdge>,
    pub active_portfolio_id: Option<String>,
    pub next_id: u64,
    pub next_edge_id: u64,
    /// Metadata summary.
    pub component_count: usize,
    pub edge_count: usize,
    pub event_count: usize,
    pub last_checkpoint_id: Option<String>,
}

/// Named, human-memorable pointer to a stored snapshot.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortfolioCheckpoint {
    pub id: String,
    pub label: String,
    pub snapshot_id: String,
    pub created_at: u64,
    pub note: Option<String>,
}

// =============================================================================
// §7 — CRDT
// =============================================================================

/// Lamport-style vector clock: actor → logical time.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct VectorClock(pub HashMap<String, u64>);

impl VectorClock {
    pub fn tick(&mut self, actor: &str) -> u64 {
        let t = self.0.entry(actor.to_string()).or_insert(0);
        *t += 1;
        *t
    }

    /// Merge two clocks by taking the element-wise maximum.
    pub fn merge(&mut self, other: &VectorClock) {
        for (actor, &t) in &other.0 {
            let entry = self.0.entry(actor.clone()).or_insert(0);
            if t > *entry {
                *entry = t;
            }
        }
    }
}

/// Last-Write-Wins scalar field.  Ties broken by lexicographic actor_id.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LwwField {
    pub value: String,
    pub timestamp: u64,
    pub actor_id: String,
}

impl LwwField {
    pub fn new(value: impl Into<String>, actor_id: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            timestamp: now(),
            actor_id: actor_id.into(),
        }
    }

    /// Apply an incoming update, accepting it only if it is "later".
    pub fn merge(&mut self, incoming: LwwField) {
        if incoming.timestamp > self.timestamp
            || (incoming.timestamp == self.timestamp
                && incoming.actor_id > self.actor_id)
        {
            *self = incoming;
        }
    }
}

/// Observed-Remove Set.
/// Tracks (element, unique_tag) pairs; removals tombstone specific tags.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OrSet {
    /// Active entries: element → set of live tags.
    pub observed: HashMap<String, HashSet<String>>,
    /// Tombstone set of removed tags.
    pub removed_tags: HashSet<String>,
}

impl OrSet {
    pub fn add(&mut self, element: impl Into<String>, tag: impl Into<String>) {
        self.observed
            .entry(element.into())
            .or_default()
            .insert(tag.into());
    }

    pub fn remove(&mut self, element: &str) {
        if let Some(tags) = self.observed.remove(element) {
            self.removed_tags.extend(tags);
        }
    }

    pub fn contains(&self, element: &str) -> bool {
        self.observed
            .get(element)
            .map_or(false, |tags| !tags.is_empty())
    }

    pub fn members(&self) -> Vec<&str> {
        self.observed.keys().map(String::as_str).collect()
    }

    /// Merge remote OR-Set into self.
    pub fn merge(&mut self, remote: &OrSet) {
        // Remove any tags that the remote has tombstoned.
        for tombstone in &remote.removed_tags {
            for tags in self.observed.values_mut() {
                tags.remove(tombstone);
            }
        }
        self.removed_tags.extend(remote.removed_tags.iter().cloned());
        // Add all remote observations not yet tombstoned locally.
        for (elem, tags) in &remote.observed {
            let live: HashSet<String> = tags
                .iter()
                .filter(|t| !self.removed_tags.contains(*t))
                .cloned()
                .collect();
            if !live.is_empty() {
                self.observed
                    .entry(elem.clone())
                    .or_default()
                    .extend(live);
            }
        }
    }
}

/// A single portable CRDT mutation (used for peer exchange).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CrdtOperation {
    SetField {
        component_id: String,
        field: String,
        value: String,
        timestamp: u64,
        actor: String,
    },
    AddToSet {
        component_id: String,
        set_name: String,
        element: String,
        tag: String,
        timestamp: u64,
        actor: String,
    },
    RemoveFromSet {
        component_id: String,
        set_name: String,
        element: String,
        timestamp: u64,
        actor: String,
    },
    AddEdge {
        edge: GraphEdge,
        timestamp: u64,
        actor: String,
    },
    RemoveEdge {
        edge_id: String,
        timestamp: u64,
        actor: String,
    },
}

/// Append-only log of CRDT operations for distributed merge.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CrdtLog {
    pub operations: Vec<CrdtOperation>,
    pub clock: VectorClock,
}

impl CrdtLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(&mut self, actor: &str, op: CrdtOperation) {
        self.clock.tick(actor);
        self.operations.push(op);
    }

    /// Merge a remote peer's CRDT log into this log (union + clock merge).
    pub fn merge(&mut self, remote: &CrdtLog) {
        // Deduplicate by comparing operation payloads via their serialised form.
        // Production systems should use unique op IDs; this is a sound fallback.
        let existing: HashSet<String> = self
            .operations
            .iter()
            .filter_map(|o| serde_json::to_string(o).ok())
            .collect();

        for op in &remote.operations {
            if let Ok(s) = serde_json::to_string(op) {
                if !existing.contains(&s) {
                    self.operations.push(op.clone());
                }
            }
        }
        self.clock.merge(&remote.clock);
    }
}

// =============================================================================
// §8 — GOVERNANCE
// =============================================================================

// ── Resource-kind taxonomy ────────────────────────────────────────────────────

/// Discriminant for what kind of resource a `ResourceAllocation` tracks.
///
/// | Variant      | Unit          | Example use                              |
/// |--------------|---------------|------------------------------------------|
/// | Budget       | currency      | financial spend cap for a project        |
/// | PersonHours  | person-hours  | labour / capacity planning               |
/// | StoryPoints  | story-points  | agile sprint velocity                    |
/// | ComputeUnits | CPU/GPU hours | cloud / HPC scheduling                   |
/// | StorageGiB   | gibibytes     | data-lake / object-store quotas          |
/// | Custom       | free-form     | any domain-specific unit                 |
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceKind {
    Budget,
    PersonHours,
    StoryPoints,
    ComputeUnits,
    StorageGiB,
    Custom(String),
}

impl ResourceKind {
    /// Human-readable label for the unit associated with this kind.
    pub fn unit_label(&self) -> &str {
        match self {
            ResourceKind::Budget => "currency",
            ResourceKind::PersonHours => "person-hours",
            ResourceKind::StoryPoints => "story-points",
            ResourceKind::ComputeUnits => "compute-units",
            ResourceKind::StorageGiB => "GiB",
            ResourceKind::Custom(u) => u.as_str(),
        }
    }
}

/// Evaluation context passed to a policy engine.
#[derive(Clone, Debug)]
pub struct PolicyContext<'a> {
    pub component: &'a PortfolioComponent,
    pub event_kind: &'a PortfolioEventKind,
    pub actor_id: &'a str,
    pub properties: HashMap<String, String>,
}

/// Decision returned by a policy evaluation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow,
    Deny(String),
    RequireApproval(String),
}

/// Trait that governance policy engines must implement.
pub trait PolicyEngine: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn evaluate(&self, ctx: &PolicyContext<'_>) -> PolicyDecision;
}

/// Lifecycle stages of an approval request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}

/// A single approval request in the governance workflow.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: String,
    pub component_id: String,
    pub requested_by: String,
    pub reason: String,
    pub status: ApprovalStatus,
    pub created_at: u64,
    pub resolved_at: Option<u64>,
    pub resolver: Option<String>,
    pub notes: Option<String>,
}

/// Generalised resource allocation and spend / consumption tracking per component.
///
/// Replaces the former `BudgetAllocation` — now supports any `ResourceKind`,
/// from financial budgets to person-hours, story-points, compute quotas, etc.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub component_id: String,
    /// What type of resource this allocation tracks.
    pub kind: ResourceKind,
    /// Total capacity granted (in `kind` units).
    pub total: f64,
    /// Amount formally allocated (≤ total; may be reserved before spend).
    pub allocated: f64,
    /// Amount consumed / spent so far.
    pub consumed: f64,
    /// Human-readable denomination label (e.g. "USD", "EUR", "GPU-h").
    pub denomination: String,
    /// Optional time-boxing period (e.g. "2025-Q3", "Sprint-42").
    pub period: Option<String>,
}

impl ResourceAllocation {
    /// Remaining capacity = total − consumed.
    pub fn remaining(&self) -> f64 {
        self.total - self.consumed
    }

    /// Utilisation as a percentage (0–100+).
    pub fn utilisation_pct(&self) -> f64 {
        if self.total == 0.0 {
            0.0
        } else {
            (self.consumed / self.total) * 100.0
        }
    }

    /// True when consumed exceeds total (over-budget / over-capacity).
    pub fn is_overrun(&self) -> bool {
        self.consumed > self.total
    }

    /// Headroom = allocated − consumed (negative when over-allocated).
    pub fn headroom(&self) -> f64 {
        self.allocated - self.consumed
    }
}

/// Backward-compatibility type alias so existing call-sites compile unchanged.
#[allow(dead_code)]
pub type BudgetAllocation = ResourceAllocation;

// =============================================================================
// §9 — PLUGIN TRAITS
// =============================================================================

/// Lifecycle hooks for extending `PortfolioSystem` behaviour.
pub trait PortfolioPlugin: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;

    fn on_component_created(&self, _component: &PortfolioComponent) {}
    fn on_component_updated(&self, _component: &PortfolioComponent) {}
    fn on_component_removed(&self, _component_id: &str) {}
    fn on_edge_added(&self, _edge: &GraphEdge) {}
    fn on_edge_removed(&self, _edge_id: &str) {}
    fn on_event(&self, _event: &PortfolioEvent) {}
    fn on_snapshot_saved(&self, _snapshot: &PortfolioSnapshot) {}
    fn on_crdt_merge(&self, _ops: &[CrdtOperation]) {}
}

// =============================================================================
// §10 — FEDERATION
// =============================================================================

/// A registered remote peer in the portfolio federation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FederationPeer {
    pub id: String,
    pub name: String,
    pub endpoint: Option<String>,
    pub last_seen: u64,
    pub vector_clock: VectorClock,
    pub trusted: bool,
}

/// Multi-portfolio federation: a flat map of named `PortfolioSystem` instances
/// that can exchange CRDT operations and share cross-portfolio edges.
pub struct PortfolioFederation {
    pub portfolios: HashMap<String, PortfolioSystem>,
    pub peers: HashMap<String, FederationPeer>,
    pub federation_edges: Vec<GraphEdge>,
    next_edge_id: u64,
}

impl PortfolioFederation {
    pub fn new() -> Self {
        Self {
            portfolios: HashMap::new(),
            peers: HashMap::new(),
            federation_edges: Vec::new(),
            next_edge_id: 1,
        }
    }

    pub fn add_portfolio(&mut self, id: impl Into<String>, system: PortfolioSystem) {
        self.portfolios.insert(id.into(), system);
    }

    pub fn remove_portfolio(&mut self, id: &str) -> Option<PortfolioSystem> {
        self.portfolios.remove(id)
    }

    pub fn get_portfolio(&self, id: &str) -> Option<&PortfolioSystem> {
        self.portfolios.get(id)
    }

    pub fn get_portfolio_mut(&mut self, id: &str) -> Option<&mut PortfolioSystem> {
        self.portfolios.get_mut(id)
    }

    pub fn register_peer(&mut self, peer: FederationPeer) {
        self.peers.insert(peer.id.clone(), peer);
    }

    pub fn remove_peer(&mut self, peer_id: &str) -> Option<FederationPeer> {
        self.peers.remove(peer_id)
    }

    /// Add a cross-portfolio `Federation` edge.
    pub fn federate(
        &mut self,
        from_portfolio: &str,
        from_component: &str,
        to_portfolio: &str,
        to_component: &str,
    ) {
        let id = format!("fed-edge-{:04}", self.next_edge_id);
        self.next_edge_id += 1;
        self.federation_edges.push(GraphEdge {
            id,
            from: format!("{from_portfolio}/{from_component}"),
            to: format!("{to_portfolio}/{to_component}"),
            kind: GraphEdgeKind::Federation,
            label: None,
            created_at: now(),
            properties: HashMap::new(),
        });
    }

    /// Push CRDT log from `source_id` into `target_id`.
    pub fn sync_crdt(&mut self, source_id: &str, target_id: &str) -> Result<(), String> {
        let source_crdt = self
            .portfolios
            .get(source_id)
            .ok_or_else(|| format!("Portfolio '{source_id}' not found"))?
            .crdt
            .clone();

        let target = self
            .portfolios
            .get_mut(target_id)
            .ok_or_else(|| format!("Portfolio '{target_id}' not found"))?;

        target.crdt.merge(&source_crdt);
        target.event_log.append(
            "federation",
            PortfolioEventKind::CrdtMergeApplied,
            HashMap::from([
                ("source".to_string(), source_id.to_string()),
                ("ops".to_string(), source_crdt.operations.len().to_string()),
            ]),
        );

        Ok(())
    }

    /// Broadcast one portfolio's CRDT log to all other portfolios.
    pub fn broadcast_crdt(&mut self, source_id: &str) -> Result<usize, String> {
        let target_ids: Vec<String> = self
            .portfolios
            .keys()
            .filter(|k| k.as_str() != source_id)
            .cloned()
            .collect();

        let count = target_ids.len();
        for target_id in target_ids {
            self.sync_crdt(source_id, &target_id)?;
        }
        Ok(count)
    }
}

// =============================================================================
// §11 — QUERY LANGUAGE (PQL)
// =============================================================================

/// Predicate filter for `PortfolioSystem::query`.
#[derive(Clone, Debug, Default)]
pub struct PortfolioQuery {
    pub component_types: Option<Vec<PortfolioComponentType>>,
    pub statuses: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub name_contains: Option<String>,
    pub owner: Option<String>,
    pub has_dependencies: Option<bool>,
    pub has_children: Option<bool>,
    pub property_filter: Option<(String, String)>,
}

impl PortfolioQuery {
    /// Parse a minimal PQL query string.
    ///
    /// Supported tokens (space-separated, evaluated as AND):
    ///   `type=<ComponentType>`   `status=<value>`   `tag=<value>`
    ///   `name=<substring>`       `owner=<value>`
    pub fn parse(pql: &str) -> Self {
        let mut q = Self::default();
        for token in pql.split_whitespace() {
            if let Some(rest) = token.strip_prefix("type=") {
                let ct = parse_component_type(rest);
                q.component_types.get_or_insert_with(Vec::new).push(ct);
            } else if let Some(rest) = token.strip_prefix("status=") {
                q.statuses
                    .get_or_insert_with(Vec::new)
                    .push(rest.to_string());
            } else if let Some(rest) = token.strip_prefix("tag=") {
                q.tags.get_or_insert_with(Vec::new).push(rest.to_string());
            } else if let Some(rest) = token.strip_prefix("name=") {
                q.name_contains = Some(rest.to_string());
            } else if let Some(rest) = token.strip_prefix("owner=") {
                q.owner = Some(rest.to_string());
            }
        }
        q
    }
}

fn parse_component_type(s: &str) -> PortfolioComponentType {
    match s.to_lowercase().as_str() {
        "portfolio" => PortfolioComponentType::Portfolio,
        "project" => PortfolioComponentType::Project,
        "program" => PortfolioComponentType::Program,
        "resource" => PortfolioComponentType::Resource,
        "asset" => PortfolioComponentType::Asset,
        "artifact" => PortfolioComponentType::Artifact,
        "subportfolio" | "sub_portfolio" => PortfolioComponentType::SubPortfolio,
        "binder" => PortfolioComponentType::Binder,
        "book" => PortfolioComponentType::Book,
        "folder" => PortfolioComponentType::Folder,
        "record" => PortfolioComponentType::Record,
        _ => PortfolioComponentType::Project, // safe fallback
    }
}

// =============================================================================
// §12 — PORTFOLIO SYSTEM
// =============================================================================

/// The central runtime.
///
/// All state is stored in `components` (a flat `HashMap<id, PortfolioComponent>`)
/// plus `edges` (the typed relationship graph).  Snapshots, checkpoints,
/// events, CRDT operations, governance data, and plugins are maintained
/// alongside.
#[derive(Clone)]
pub struct PortfolioSystem {
    // ── Core state ────────────────────────────────────────────────────────
    pub components: HashMap<String, PortfolioComponent>,
    pub edges: Vec<GraphEdge>,
    pub active_portfolio_id: Option<String>,

    // ── Identity / sequencing ─────────────────────────────────────────────
    pub actor_id: String,
    next_id: u64,
    next_edge_id: u64,
    next_snap_id: u64,
    next_ckpt_id: u64,
    next_req_id: u64,

    // ── History / time-travel ─────────────────────────────────────────────
    pub event_log: EventLog,
    pub snapshots: HashMap<String, PortfolioSnapshot>,
    pub checkpoints: Vec<PortfolioCheckpoint>,

    // ── Distributed ───────────────────────────────────────────────────────
    pub crdt: CrdtLog,

    // ── Governance ────────────────────────────────────────────────────────
    pub policy_engines: Vec<Arc<dyn PolicyEngine>>,
    pub approval_requests: Vec<ApprovalRequest>,
    pub resource_allocations: HashMap<String, ResourceAllocation>,

    // ── Plugins ───────────────────────────────────────────────────────────
    pub plugins: Vec<Arc<dyn PortfolioPlugin>>,
}

impl std::fmt::Debug for PortfolioSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PortfolioSystem")
            .field("components", &self.components.len())
            .field("edges", &self.edges.len())
            .field("active_portfolio_id", &self.active_portfolio_id)
            .field("actor_id", &self.actor_id)
            .field("event_log", &self.event_log)
            .field("snapshots", &self.snapshots.len())
            .field("checkpoints", &self.checkpoints.len())
            .field("crdt", &self.crdt)
            .field("policy_engines", &self.policy_engines.len())
            .field("approval_requests", &self.approval_requests.len())
            .field("resource_allocations", &self.resource_allocations.len())
            .field("plugins", &self.plugins.len())
            .finish()
    }
}


// ── Constructors ──────────────────────────────────────────────────────────────

impl PortfolioSystem {
    /// Create an empty system.
    pub fn new(actor_id: impl Into<String>) -> Self {
        Self {
            components: HashMap::new(),
            edges: Vec::new(),
            active_portfolio_id: None,
            actor_id: actor_id.into(),
            next_id: 1,
            next_edge_id: 1,
            next_snap_id: 1,
            next_ckpt_id: 1,
            next_req_id: 1,
            event_log: EventLog::new(),
            snapshots: HashMap::new(),
            checkpoints: Vec::new(),
            crdt: CrdtLog::new(),
            policy_engines: Vec::new(),
            approval_requests: Vec::new(),
            resource_allocations: HashMap::new(),
            plugins: Vec::new(),
        }
    }

    /// Backwards-compatible MVP seed (mirrors the original hardcoded state).
    pub fn mvp() -> Self {
        let mut sys = Self::new("system");
        let comp = sys.create_component_raw(
            PortfolioComponentType::Project,
            "Kogi Kernel Runtime",
            vec![
                PortfolioItemContainerType::Binder,
                PortfolioItemContainerType::Book,
                PortfolioItemContainerType::Notebook,
                PortfolioItemContainerType::Playbook,
                PortfolioItemContainerType::Folder,
                PortfolioItemContainerType::FileSet,
                PortfolioItemContainerType::VersionControl,
                PortfolioItemContainerType::Metadata,
            ],
        );
        sys.active_portfolio_id = Some(comp.metadata.id.clone());
        sys
    }

    // ── Private helpers ───────────────────────────────────────────────────

    fn next_comp_id(&mut self) -> String {
        let id = format!("comp-{:06}", self.next_id);
        self.next_id += 1;
        id
    }

    fn next_edge_id_str(&mut self) -> String {
        let id = format!("edge-{:06}", self.next_edge_id);
        self.next_edge_id += 1;
        id
    }

    fn emit(
        &mut self,
        kind: PortfolioEventKind,
        payload: impl IntoIterator<Item = (&'static str, String)>,
    ) {
        let map: HashMap<String, String> =
            payload.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
        let actor = self.actor_id.clone();
        let evt = self.event_log.append(&actor, kind, map).clone();
        for plugin in &self.plugins {
            plugin.on_event(&evt);
        }
    }

    fn evaluate_policies(
        &self,
        component: &PortfolioComponent,
        event_kind: &PortfolioEventKind,
    ) -> PolicyDecision {
        for engine in &self.policy_engines {
            let ctx = PolicyContext {
                component,
                event_kind,
                actor_id: &self.actor_id,
                properties: HashMap::new(),
            };
            match engine.evaluate(&ctx) {
                PolicyDecision::Allow => continue,
                decision => return decision,
            }
        }
        PolicyDecision::Allow
    }

    /// Internal: build and register a component (used by mvp + helpers).
    fn create_component_raw(
        &mut self,
        component_type: PortfolioComponentType,
        name: &str,
        container_refs: Vec<PortfolioItemContainerType>,
    ) -> PortfolioComponent {
        let id = self.next_comp_id();
        let actor = self.actor_id.clone();
        let mut comp = PortfolioComponent::new(&id, &actor, component_type, name);
        comp.container_refs = container_refs;

        self.emit(
            PortfolioEventKind::ComponentCreated,
            [("id", id.clone()), ("name", name.to_string())],
        );
        for plugin in &self.plugins {
            plugin.on_component_created(&comp);
        }
        self.components.insert(id, comp.clone());
        comp
    }
}

// ── Component CRUD ────────────────────────────────────────────────────────────

impl PortfolioSystem {
    /// Create and register a new `PortfolioComponent`.
    /// Returns `Err` if a governance policy denies the operation.
    pub fn create_component(
        &mut self,
        component_type: PortfolioComponentType,
        name: impl Into<String>,
    ) -> Result<PortfolioComponent, String> {
        let name = name.into();
        let id = self.next_comp_id();
        let actor = self.actor_id.clone();
        let comp = PortfolioComponent::new(&id, &actor, component_type.clone(), &name);

        // Policy check
        match self.evaluate_policies(&comp, &PortfolioEventKind::ComponentCreated) {
            PolicyDecision::Allow => {}
            PolicyDecision::Deny(reason) => return Err(format!("Denied: {reason}")),
            PolicyDecision::RequireApproval(reason) => {
                return Err(format!("Requires approval: {reason}"))
            }
        }

        self.emit(
            PortfolioEventKind::ComponentCreated,
            [
                ("id", id.clone()),
                ("name", name.clone()),
                ("type", format!("{component_type:?}")),
            ],
        );
        self.crdt.append(
            &actor,
            CrdtOperation::SetField {
                component_id: id.clone(),
                field: "name".into(),
                value: name,
                timestamp: now(),
                actor: actor.clone(),
            },
        );

        for plugin in &self.plugins {
            plugin.on_component_created(&comp);
        }
        self.components.insert(id.clone(), comp.clone());
        Ok(comp)
    }

    /// Create a `Book` component with a specific `BookType`.
    pub fn create_book(
        &mut self,
        book_type: BookType,
        name: impl Into<String>,
    ) -> Result<PortfolioComponent, String> {
        let mut comp = self.create_component(PortfolioComponentType::Book, name)?;
        comp.book_type = Some(book_type.clone());

        // Itembooks are auto-linked to the system's active portfolio item.
        if book_type == BookType::Itembook {
            if let Some(ref pid) = self.active_portfolio_id.clone() {
                self.add_member(pid, &comp.metadata.id)?;
            }
        }

        self.components
            .entry(comp.metadata.id.clone())
            .and_modify(|c| c.book_type = Some(book_type));
        Ok(comp)
    }

    /// Retrieve a component by ID (immutable).
    pub fn get_component(&self, id: &str) -> Option<&PortfolioComponent> {
        self.components.get(id)
    }

    /// Retrieve a component by ID (mutable).
    pub fn get_component_mut(&mut self, id: &str) -> Option<&mut PortfolioComponent> {
        self.components.get_mut(id)
    }

    /// All components, unordered.
    pub fn all_components(&self) -> Vec<&PortfolioComponent> {
        self.components.values().collect()
    }

    /// All components of a specific type.
    pub fn components_by_type(
        &self,
        component_type: &PortfolioComponentType,
    ) -> Vec<&PortfolioComponent> {
        self.components
            .values()
            .filter(|c| &c.component_type == component_type)
            .collect()
    }

    /// A specific subset by IDs (skips missing IDs silently).
    pub fn components_subset<'a>(
        &'a self,
        ids: &[String],
    ) -> Vec<&'a PortfolioComponent> {
        ids.iter().filter_map(|id| self.components.get(id)).collect()
    }

    /// Update name / status / properties of a component.
    ///
    /// Only `Some(…)` fields are applied.
    pub fn edit_component(
        &mut self,
        id: &str,
        name: Option<String>,
        status: Option<String>,
        tags: Option<Vec<String>>,
        properties: Option<HashMap<String, String>>,
        owner: Option<String>,
    ) -> Result<PortfolioComponent, String> {
        let actor = self.actor_id.clone();
        let comp = self
            .components
            .get_mut(id)
            .ok_or_else(|| format!("Component '{id}' not found"))?;

        if let Some(n) = name {
            self.crdt.append(
                &actor,
                CrdtOperation::SetField {
                    component_id: id.to_string(),
                    field: "name".into(),
                    value: n.clone(),
                    timestamp: now(),
                    actor: actor.clone(),
                },
            );
            comp.name = n;
        }
        if let Some(s) = status {
            comp.status = s;
        }
        if let Some(t) = tags {
            comp.metadata.tags = t;
        }
        if let Some(p) = properties {
            comp.metadata.properties.extend(p);
        }
        if let Some(o) = owner {
            comp.metadata.owner = Some(o);
        }
        comp.metadata.touch(&actor);

        let result = comp.clone();
        self.emit(
            PortfolioEventKind::ComponentUpdated,
            [("id", id.to_string())],
        );
        for plugin in &self.plugins {
            plugin.on_component_updated(&result);
        }
        Ok(result)
    }

    /// Remove a component and all its incident edges.
    pub fn remove_component(&mut self, id: &str) -> Result<PortfolioComponent, String> {
        let removed = self
            .components
            .remove(id)
            .ok_or_else(|| format!("Component '{id}' not found"))?;

        // Clean up edges
        let removed_edges: Vec<String> = self
            .edges
            .iter()
            .filter(|e| e.from == id || e.to == id)
            .map(|e| e.id.clone())
            .collect();
        for eid in &removed_edges {
            self.remove_edge(eid).ok();
        }

        // Remove from parent/child/link sets of other components
        for comp in self.components.values_mut() {
            comp.children.remove(id);
            comp.parents.remove(id);
            comp.links.remove(id);
            comp.dependencies.remove(id);
            comp.dependents.remove(id);
            comp.unordered_members.remove(id);
            comp.ordered_members.retain(|m| m != id);
        }

        self.emit(
            PortfolioEventKind::ComponentRemoved,
            [("id", id.to_string())],
        );
        for plugin in &self.plugins {
            plugin.on_component_removed(id);
        }
        Ok(removed)
    }

    /// Change which component is the "active portfolio" in focus.
    pub fn set_active_portfolio(&mut self, id: &str) -> Result<(), String> {
        if self.components.contains_key(id) {
            self.active_portfolio_id = Some(id.to_string());
            Ok(())
        } else {
            Err(format!("Component '{id}' not found"))
        }
    }
}

// ── Graph operations ──────────────────────────────────────────────────────────

impl PortfolioSystem {
    /// Add a directed `Hierarchy` (parent → child) edge.
    /// Returns `Err` if this would introduce a cycle.
    pub fn add_hierarchy(&mut self, parent_id: &str, child_id: &str) -> Result<String, String> {
        let view = GraphView::new(&self.components, &self.edges);
        if view.would_cycle(parent_id, child_id, &GraphEdgeKind::Hierarchy) {
            return Err(format!(
                "Adding hierarchy {parent_id}→{child_id} would create a cycle"
            ));
        }

        let edge_id = self.next_edge_id_str();
        let edge = GraphEdge::new(&edge_id, parent_id, child_id, GraphEdgeKind::Hierarchy);
        self.edges.push(edge.clone());

        if let Some(p) = self.components.get_mut(parent_id) {
            p.children.insert(child_id.to_string());
        }
        if let Some(c) = self.components.get_mut(child_id) {
            c.parents.insert(parent_id.to_string());
        }

        self.emit(
            PortfolioEventKind::HierarchyLinked,
            [
                ("parent", parent_id.to_string()),
                ("child", child_id.to_string()),
                ("edge_id", edge_id.clone()),
            ],
        );
        self.crdt.append(
            &self.actor_id.clone(),
            CrdtOperation::AddEdge {
                edge,
                timestamp: now(),
                actor: self.actor_id.clone(),
            },
        );
        for plugin in &self.plugins {
            plugin.on_edge_added(self.edges.last().unwrap());
        }
        Ok(edge_id)
    }

    /// Remove a `Hierarchy` edge by IDs.
    pub fn remove_hierarchy(&mut self, parent_id: &str, child_id: &str) -> Result<(), String> {
        let eid = self
            .edges
            .iter()
            .find(|e| {
                e.from == parent_id
                    && e.to == child_id
                    && e.kind == GraphEdgeKind::Hierarchy
            })
            .map(|e| e.id.clone())
            .ok_or_else(|| format!("Hierarchy edge {parent_id}→{child_id} not found"))?;

        self.remove_edge(&eid)?;
        if let Some(p) = self.components.get_mut(parent_id) {
            p.children.remove(child_id);
        }
        if let Some(c) = self.components.get_mut(child_id) {
            c.parents.remove(parent_id);
        }
        self.emit(
            PortfolioEventKind::HierarchyUnlinked,
            [
                ("parent", parent_id.to_string()),
                ("child", child_id.to_string()),
            ],
        );
        Ok(())
    }

    /// Add a directed `Dependency` (from → to) edge.
    pub fn add_dependency(&mut self, from: &str, to: &str) -> Result<String, String> {
        let view = GraphView::new(&self.components, &self.edges);
        if view.would_cycle(from, to, &GraphEdgeKind::Dependency) {
            return Err(format!("Dependency {from}→{to} would create a cycle"));
        }

        let edge_id = self.next_edge_id_str();
        let edge = GraphEdge::new(&edge_id, from, to, GraphEdgeKind::Dependency);
        self.edges.push(edge.clone());

        if let Some(f) = self.components.get_mut(from) {
            f.dependencies.insert(to.to_string());
        }
        if let Some(t) = self.components.get_mut(to) {
            t.dependents.insert(from.to_string());
        }

        self.emit(
            PortfolioEventKind::DependencyAdded,
            [
                ("from", from.to_string()),
                ("to", to.to_string()),
                ("edge_id", edge_id.clone()),
            ],
        );
        self.crdt.append(
            &self.actor_id.clone(),
            CrdtOperation::AddEdge {
                edge,
                timestamp: now(),
                actor: self.actor_id.clone(),
            },
        );
        Ok(edge_id)
    }

    /// Remove a `Dependency` edge by component IDs.
    pub fn remove_dependency(&mut self, from: &str, to: &str) -> Result<(), String> {
        let eid = self
            .edges
            .iter()
            .find(|e| {
                e.from == from && e.to == to && e.kind == GraphEdgeKind::Dependency
            })
            .map(|e| e.id.clone())
            .ok_or_else(|| format!("Dependency {from}→{to} not found"))?;

        self.remove_edge(&eid)?;
        if let Some(f) = self.components.get_mut(from) {
            f.dependencies.remove(to);
        }
        if let Some(t) = self.components.get_mut(to) {
            t.dependents.remove(from);
        }
        self.emit(
            PortfolioEventKind::DependencyRemoved,
            [("from", from.to_string()), ("to", to.to_string())],
        );
        Ok(())
    }

    /// Add a bidirectional `Link` between two components.
    pub fn add_link(&mut self, a: &str, b: &str) -> Result<String, String> {
        let edge_id = self.next_edge_id_str();
        let edge = GraphEdge::new(&edge_id, a, b, GraphEdgeKind::Link);
        self.edges.push(edge.clone());

        if let Some(ca) = self.components.get_mut(a) {
            ca.links.insert(b.to_string());
        }
        if let Some(cb) = self.components.get_mut(b) {
            cb.links.insert(a.to_string());
        }

        self.emit(
            PortfolioEventKind::EdgeAdded,
            [("a", a.to_string()), ("b", b.to_string()), ("edge_id", edge_id.clone())],
        );
        self.crdt.append(
            &self.actor_id.clone(),
            CrdtOperation::AddEdge {
                edge,
                timestamp: now(),
                actor: self.actor_id.clone(),
            },
        );
        for plugin in &self.plugins {
            plugin.on_edge_added(self.edges.last().unwrap());
        }
        Ok(edge_id)
    }

    /// Remove a `Link` by component IDs.
    pub fn remove_link(&mut self, a: &str, b: &str) -> Result<(), String> {
        let eid = self
            .edges
            .iter()
            .find(|e| {
                e.kind == GraphEdgeKind::Link
                    && ((e.from == a && e.to == b) || (e.from == b && e.to == a))
            })
            .map(|e| e.id.clone())
            .ok_or_else(|| format!("Link {a}↔{b} not found"))?;

        self.remove_edge(&eid)?;
        if let Some(ca) = self.components.get_mut(a) {
            ca.links.remove(b);
        }
        if let Some(cb) = self.components.get_mut(b) {
            cb.links.remove(a);
        }
        Ok(())
    }

    /// Remove any edge by its edge ID.
    pub fn remove_edge(&mut self, edge_id: &str) -> Result<GraphEdge, String> {
        let pos = self
            .edges
            .iter()
            .position(|e| e.id == edge_id)
            .ok_or_else(|| format!("Edge '{edge_id}' not found"))?;
        let removed = self.edges.remove(pos);
        self.emit(
            PortfolioEventKind::EdgeRemoved,
            [("edge_id", edge_id.to_string())],
        );
        self.crdt.append(
            &self.actor_id.clone(),
            CrdtOperation::RemoveEdge {
                edge_id: edge_id.to_string(),
                timestamp: now(),
                actor: self.actor_id.clone(),
            },
        );
        for plugin in &self.plugins {
            plugin.on_edge_removed(edge_id);
        }
        Ok(removed)
    }

    /// Add `item_id` as a member of `container_id`.
    pub fn add_member(&mut self, container_id: &str, item_id: &str) -> Result<(), String> {
        let comp = self
            .components
            .get_mut(container_id)
            .ok_or_else(|| format!("Container '{container_id}' not found"))?;
        comp.add_member(item_id);
        comp.metadata.touch(&self.actor_id.clone());

        let edge_id = self.next_edge_id_str();
        let edge = GraphEdge::new(&edge_id, container_id, item_id, GraphEdgeKind::Contains);
        self.edges.push(edge);

        self.emit(
            PortfolioEventKind::MemberAdded,
            [
                ("container", container_id.to_string()),
                ("item", item_id.to_string()),
            ],
        );
        Ok(())
    }

    /// Remove `item_id` from the membership of `container_id`.
    pub fn remove_member(&mut self, container_id: &str, item_id: &str) -> Result<(), String> {
        let comp = self
            .components
            .get_mut(container_id)
            .ok_or_else(|| format!("Container '{container_id}' not found"))?;

        if !comp.remove_member(item_id) {
            return Err(format!("Item '{item_id}' not a member of '{container_id}'"));
        }
        comp.metadata.touch(&self.actor_id.clone());

        self.edges.retain(|e| {
            !(e.from == container_id && e.to == item_id && e.kind == GraphEdgeKind::Contains)
        });

        self.emit(
            PortfolioEventKind::MemberRemoved,
            [
                ("container", container_id.to_string()),
                ("item", item_id.to_string()),
            ],
        );
        Ok(())
    }

    /// Read-only graph view over the current state.
    pub fn graph_view(&self) -> GraphView<'_> {
        GraphView::new(&self.components, &self.edges)
    }

    /// Topological sort of the dependency graph.
    pub fn dependency_order(&self) -> Result<Vec<String>, String> {
        self.graph_view()
            .topological_sort(&GraphEdgeKind::Dependency)
    }

    /// All transitive dependencies of `id`.
    pub fn transitive_dependencies(&self, id: &str) -> Vec<String> {
        self.graph_view().reachable(id, &GraphEdgeKind::Dependency)
    }

    /// All transitive children (full subtree) of `id`.
    pub fn subtree(&self, id: &str) -> Vec<String> {
        self.graph_view().reachable(id, &GraphEdgeKind::Hierarchy)
    }
}

// ── Snapshot, Checkpoint & Time-Travel ───────────────────────────────────────

impl PortfolioSystem {
    /// Capture an in-memory snapshot of the current state (not stored).
    pub fn snapshot(&self) -> PortfolioSnapshot {
        self.build_snapshot(None)
    }

    /// Capture and store a snapshot, returning it.
    pub fn save_snapshot(&mut self, label: Option<String>) -> PortfolioSnapshot {
        let snap = self.build_snapshot(label);
        let id = snap.snapshot_id.clone();
        self.snapshots.insert(id.clone(), snap.clone());
        self.emit(
            PortfolioEventKind::SnapshotSaved,
            [("snapshot_id", id)],
        );
        for plugin in &self.plugins {
            plugin.on_snapshot_saved(&snap);
        }
        snap
    }

    /// Save a snapshot and create a named checkpoint pointing to it.
    pub fn save_checkpoint(
        &mut self,
        label: impl Into<String>,
        note: Option<String>,
    ) -> PortfolioCheckpoint {
        let label = label.into();
        let snap = self.save_snapshot(Some(label.clone()));
        let ckpt_id = format!("ckpt-{:04}", self.next_ckpt_id);
        self.next_ckpt_id += 1;

        let ckpt = PortfolioCheckpoint {
            id: ckpt_id,
            label,
            snapshot_id: snap.snapshot_id.clone(),
            created_at: now(),
            note,
        };
        self.checkpoints.push(ckpt.clone());
        self.emit(
            PortfolioEventKind::CheckpointCreated,
            [
                ("checkpoint_id", ckpt.id.clone()),
                ("snapshot_id", snap.snapshot_id),
            ],
        );
        ckpt
    }

    /// Restore state from a stored snapshot ID.
    pub fn restore_snapshot(&mut self, snapshot_id: &str) -> Result<(), String> {
        let snap = self
            .snapshots
            .get(snapshot_id)
            .ok_or_else(|| format!("Snapshot '{snapshot_id}' not found"))?
            .clone();
        self.apply_snapshot(snap);
        Ok(())
    }

    /// Restore state from a checkpoint label or ID.
    pub fn restore_checkpoint(&mut self, checkpoint_id: &str) -> Result<(), String> {
        let snap_id = self
            .checkpoints
            .iter()
            .find(|c| c.id == checkpoint_id || c.label == checkpoint_id)
            .ok_or_else(|| format!("Checkpoint '{checkpoint_id}' not found"))?
            .snapshot_id
            .clone();
        self.restore_snapshot(&snap_id)
    }

    /// Restore state from an externally-provided snapshot (e.g. from disk).
    pub fn restore_from(&mut self, snapshot: PortfolioSnapshot) {
        self.apply_snapshot(snapshot);
    }

    /// Reconstruct a read-only state image by replaying events up to `timestamp`.
    ///
    /// Note: this returns a *new* system built from event replay.  The current
    /// system is not modified.  Full fidelity requires that each stored snapshot
    /// has been taken before `timestamp`, followed by event replay.
    pub fn reconstruct_at(&self, timestamp: u64) -> PortfolioSystem {
        // Find the latest snapshot that was saved at or before `timestamp`.
        let base_snap: Option<&PortfolioSnapshot> = self
            .snapshots
            .values()
            .filter(|s| s.created_at <= timestamp)
            .max_by_key(|s| s.created_at);

        let mut rebuilt = PortfolioSystem::new(&self.actor_id);

        if let Some(snap) = base_snap {
            rebuilt.apply_snapshot(snap.clone());
        }

        // Replay events after the snapshot's timestamp.
        let snap_ts = base_snap.map_or(0, |s| s.created_at);
        for evt in self
            .event_log
            .all()
            .iter()
            .filter(|e| e.timestamp > snap_ts && e.timestamp <= timestamp)
        {
            rebuilt.apply_event_for_time_travel(evt);
        }

        rebuilt
    }

    // ── Snapshot internals ────────────────────────────────────────────────

    fn build_snapshot(&self, label: Option<String>) -> PortfolioSnapshot {
        let id = format!("snap-{:06}", self.next_snap_id);
        PortfolioSnapshot {
            snapshot_id: id,
            label,
            created_at: now(),
            components: self.components.clone(),
            edges: self.edges.clone(),
            active_portfolio_id: self.active_portfolio_id.clone(),
            next_id: self.next_id,
            next_edge_id: self.next_edge_id,
            component_count: self.components.len(),
            edge_count: self.edges.len(),
            event_count: self.event_log.len(),
            last_checkpoint_id: self.checkpoints.last().map(|c| c.id.clone()),
        }
    }

    fn apply_snapshot(&mut self, snap: PortfolioSnapshot) {
        self.components = snap.components;
        self.edges = snap.edges;
        self.active_portfolio_id = snap.active_portfolio_id;
        self.next_id = snap.next_id;
        self.next_edge_id = snap.next_edge_id;
        self.emit(
            PortfolioEventKind::StateRestored,
            [("snapshot_id", snap.snapshot_id)],
        );
    }

    /// Best-effort event application for time-travel replay.
    fn apply_event_for_time_travel(&mut self, event: &PortfolioEvent) {
        match &event.kind {
            PortfolioEventKind::ComponentCreated => {
                if let (Some(id), Some(name)) =
                    (event.payload.get("id"), event.payload.get("name"))
                {
                    let ct = event
                        .payload
                        .get("type")
                        .map(|t| parse_component_type(t))
                        .unwrap_or(PortfolioComponentType::Project);
                    let comp = PortfolioComponent::new(id, &event.actor_id, ct, name);
                    self.components.insert(id.clone(), comp);
                }
            }
            PortfolioEventKind::ComponentRemoved => {
                if let Some(id) = event.payload.get("id") {
                    self.components.remove(id);
                }
            }
            PortfolioEventKind::ComponentUpdated => {
                // Already captured in CRDT operations; skip for simplicity.
            }
            PortfolioEventKind::EdgeAdded => { /* restored via snapshot */ }
            PortfolioEventKind::EdgeRemoved => {
                if let Some(eid) = event.payload.get("edge_id") {
                    self.edges.retain(|e| &e.id != eid);
                }
            }
            _ => { /* governance/federation events: no live state change needed here */ }
        }
    }
}

// ── Queries ───────────────────────────────────────────────────────────────────

impl PortfolioSystem {
    /// Filter components by a structured `PortfolioQuery`.
    pub fn query(&self, q: &PortfolioQuery) -> Vec<&PortfolioComponent> {
        self.components
            .values()
            .filter(|c| {
                if let Some(ref types) = q.component_types {
                    if !types.contains(&c.component_type) {
                        return false;
                    }
                }
                if let Some(ref statuses) = q.statuses {
                    if !statuses.contains(&c.status) {
                        return false;
                    }
                }
                if let Some(ref tags) = q.tags {
                    if !tags.iter().all(|t| c.metadata.tags.contains(t)) {
                        return false;
                    }
                }
                if let Some(ref substr) = q.name_contains {
                    if !c.name.to_lowercase().contains(&substr.to_lowercase()) {
                        return false;
                    }
                }
                if let Some(ref owner) = q.owner {
                    if c.metadata.owner.as_deref() != Some(owner) {
                        return false;
                    }
                }
                if let Some(want_deps) = q.has_dependencies {
                    if c.dependencies.is_empty() == want_deps {
                        return false;
                    }
                }
                if let Some(want_children) = q.has_children {
                    if c.children.is_empty() == want_children {
                        return false;
                    }
                }
                if let Some((ref key, ref val)) = q.property_filter {
                    if c.metadata.properties.get(key).map(String::as_str) != Some(val) {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    /// Execute a PQL query string.
    pub fn query_pql(&self, pql: &str) -> Vec<&PortfolioComponent> {
        self.query(&PortfolioQuery::parse(pql))
    }

    /// Snapshot summary metadata (item counts, edge counts, etc.).
    pub fn portfolio_metadata(&self) -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("component_count".into(), self.components.len().to_string());
        m.insert("edge_count".into(), self.edges.len().to_string());
        m.insert("event_count".into(), self.event_log.len().to_string());
        m.insert(
            "snapshot_count".into(),
            self.snapshots.len().to_string(),
        );
        m.insert(
            "checkpoint_count".into(),
            self.checkpoints.len().to_string(),
        );
        m.insert(
            "active_portfolio_id".into(),
            self.active_portfolio_id
                .clone()
                .unwrap_or_else(|| "none".into()),
        );
        for (ct, count) in self.type_counts() {
            m.insert(format!("count_{ct:?}"), count.to_string());
        }
        m
    }

    fn type_counts(&self) -> HashMap<PortfolioComponentType, usize> {
        let mut map: HashMap<PortfolioComponentType, usize> = HashMap::new();
        for c in self.components.values() {
            *map.entry(c.component_type.clone()).or_insert(0) += 1;
        }
        map
    }
}

// ── Governance ────────────────────────────────────────────────────────────────

impl PortfolioSystem {
    /// Register a governance policy engine.
    pub fn register_policy_engine(&mut self, engine: Arc<dyn PolicyEngine>) {
        self.policy_engines.push(engine);
    }

    /// Attach a policy ID to a component's metadata.
    pub fn attach_policy(&mut self, component_id: &str, policy_id: &str) -> Result<(), String> {
        let comp = self
            .components
            .get_mut(component_id)
            .ok_or_else(|| format!("Component '{component_id}' not found"))?;
        comp.metadata.policy_ids.push(policy_id.to_string());
        self.emit(
            PortfolioEventKind::PolicyAttached,
            [
                ("component_id", component_id.to_string()),
                ("policy_id", policy_id.to_string()),
            ],
        );
        Ok(())
    }

    /// Detach a policy ID from a component's metadata.
    pub fn detach_policy(&mut self, component_id: &str, policy_id: &str) -> Result<(), String> {
        let comp = self
            .components
            .get_mut(component_id)
            .ok_or_else(|| format!("Component '{component_id}' not found"))?;
        comp.metadata.policy_ids.retain(|p| p != policy_id);
        self.emit(
            PortfolioEventKind::PolicyDetached,
            [
                ("component_id", component_id.to_string()),
                ("policy_id", policy_id.to_string()),
            ],
        );
        Ok(())
    }

    /// Open an approval request for a component operation.
    pub fn request_approval(
        &mut self,
        component_id: &str,
        reason: impl Into<String>,
    ) -> ApprovalRequest {
        let id = format!("req-{:06}", self.next_req_id);
        self.next_req_id += 1;
        let req = ApprovalRequest {
            id: id.clone(),
            component_id: component_id.to_string(),
            requested_by: self.actor_id.clone(),
            reason: reason.into(),
            status: ApprovalStatus::Pending,
            created_at: now(),
            resolved_at: None,
            resolver: None,
            notes: None,
        };
        self.approval_requests.push(req.clone());
        self.emit(
            PortfolioEventKind::ApprovalRequested,
            [("request_id", id), ("component_id", component_id.to_string())],
        );
        req
    }

    /// Resolve a pending approval request.
    pub fn resolve_approval(
        &mut self,
        request_id: &str,
        approved: bool,
        resolver: impl Into<String>,
        notes: Option<String>,
    ) -> Result<ApprovalRequest, String> {
        let req = self
            .approval_requests
            .iter_mut()
            .find(|r| r.id == request_id)
            .ok_or_else(|| format!("Approval request '{request_id}' not found"))?;

        req.status = if approved {
            ApprovalStatus::Approved
        } else {
            ApprovalStatus::Rejected
        };
        req.resolved_at = Some(now());
        req.resolver = Some(resolver.into());
        req.notes = notes;

        let result = req.clone();
        self.emit(
            if approved {
                PortfolioEventKind::ApprovalGranted
            } else {
                PortfolioEventKind::ApprovalRejected
            },
            [("request_id", request_id.to_string())],
        );
        Ok(result)
    }

    /// Allocate a resource of any `ResourceKind` to a component.
    ///
    /// Replaces / updates any previous allocation for the same component.
    pub fn allocate_resource(
        &mut self,
        component_id: &str,
        kind: ResourceKind,
        total: f64,
        denomination: impl Into<String>,
        period: Option<String>,
    ) -> Result<ResourceAllocation, String> {
        if !self.components.contains_key(component_id) {
            return Err(format!("Component '{component_id}' not found"));
        }
        let denomination = denomination.into();
        let alloc = ResourceAllocation {
            component_id: component_id.to_string(),
            kind,
            total,
            allocated: total,
            consumed: 0.0,
            denomination: denomination.clone(),
            period,
        };
        self.resource_allocations
            .insert(component_id.to_string(), alloc.clone());
        if let Some(comp) = self.components.get_mut(component_id) {
            comp.metadata.budget = Some(total);
        }
        self.emit(
            PortfolioEventKind::ResourceAllocated,
            [
                ("component_id", component_id.to_string()),
                ("total", total.to_string()),
                ("denomination", denomination),
            ],
        );
        Ok(alloc)
    }

    /// Convenience wrapper: allocate a financial `Budget` resource.
    pub fn allocate_budget(
        &mut self,
        component_id: &str,
        total: f64,
        currency: impl Into<String>,
        period: Option<String>,
    ) -> Result<ResourceAllocation, String> {
        self.allocate_resource(component_id, ResourceKind::Budget, total, currency, period)
    }

    /// Record consumption / spend against a component's resource allocation.
    pub fn record_consumption(
        &mut self,
        component_id: &str,
        amount: f64,
    ) -> Result<f64, String> {
        let alloc = self
            .resource_allocations
            .get_mut(component_id)
            .ok_or_else(|| format!("No resource allocated for '{component_id}'"))?;
        alloc.consumed += amount;
        let remaining = alloc.remaining();
        if let Some(comp) = self.components.get_mut(component_id) {
            comp.metadata.budget_spent += amount;
        }
        self.emit(
            PortfolioEventKind::ResourceConsumed,
            [
                ("component_id", component_id.to_string()),
                ("amount", amount.to_string()),
                ("remaining", remaining.to_string()),
            ],
        );
        Ok(remaining)
    }

    /// Backward-compatible wrapper for `record_consumption`.
    pub fn record_spend(&mut self, component_id: &str, amount: f64) -> Result<f64, String> {
        self.record_consumption(component_id, amount)
    }

    /// Retrieve the resource allocation for a component, if any.
    pub fn get_resource_allocation(&self, component_id: &str) -> Option<&ResourceAllocation> {
        self.resource_allocations.get(component_id)
    }

    /// All resource allocations that are currently overrun.
    pub fn overrun_allocations(&self) -> Vec<&ResourceAllocation> {
        self.resource_allocations
            .values()
            .filter(|a| a.is_overrun())
            .collect()
    }

    /// Total consumption across all allocations of a given `ResourceKind`.
    pub fn total_consumption_by_kind(&self, kind: &ResourceKind) -> f64 {
        self.resource_allocations
            .values()
            .filter(|a| &a.kind == kind)
            .map(|a| a.consumed)
            .sum()
    }
}

// ── Plugin registration ───────────────────────────────────────────────────────

impl PortfolioSystem {
    pub fn register_plugin(&mut self, plugin: Arc<dyn PortfolioPlugin>) {
        self.plugins.push(plugin);
    }

    pub fn unregister_plugin(&mut self, plugin_id: &str) {
        self.plugins.retain(|p| p.id() != plugin_id);
    }
}

// ── CRDT / distributed merge ──────────────────────────────────────────────────

impl PortfolioSystem {
    /// Apply a remote CRDT log, mutating live component state.
    pub fn apply_crdt_log(&mut self, remote: &CrdtLog) {
        let ops: Vec<CrdtOperation> = remote.operations.clone();
        for op in &ops {
            self.apply_crdt_op(op);
        }
        self.crdt.merge(remote);
        self.emit(
            PortfolioEventKind::CrdtMergeApplied,
            [("op_count", ops.len().to_string())],
        );
        for plugin in &self.plugins {
            plugin.on_crdt_merge(&ops);
        }
    }

    fn apply_crdt_op(&mut self, op: &CrdtOperation) {
        match op {
            CrdtOperation::SetField {
                component_id,
                field,
                value,
                timestamp,
                actor,
            } => {
                if let Some(comp) = self.components.get_mut(component_id) {
                    // LWW: only accept if incoming is strictly newer or same
                    // time with lexicographically larger actor.
                    let accept = comp.metadata.updated_at < *timestamp
                        || (comp.metadata.updated_at == *timestamp
                            && comp.metadata.actor_id < *actor);
                    if accept {
                        match field.as_str() {
                            "name" => comp.name = value.clone(),
                            "status" => comp.status = value.clone(),
                            "owner" => comp.metadata.owner = Some(value.clone()),
                            _ => {
                                comp.metadata.properties.insert(field.clone(), value.clone());
                            }
                        }
                        comp.metadata.updated_at = *timestamp;
                        comp.metadata.actor_id = actor.clone();
                    }
                }
            }
            CrdtOperation::AddToSet {
                component_id,
                set_name,
                element,
                ..
            } => {
                if let Some(comp) = self.components.get_mut(component_id) {
                    match set_name.as_str() {
                        "tags" => {
                            if !comp.metadata.tags.contains(element) {
                                comp.metadata.tags.push(element.clone());
                            }
                        }
                        "dependencies" => {
                            comp.dependencies.insert(element.clone());
                        }
                        "links" => {
                            comp.links.insert(element.clone());
                        }
                        "members" => {
                            comp.unordered_members.insert(element.clone());
                        }
                        _ => {}
                    }
                }
            }
            CrdtOperation::RemoveFromSet {
                component_id,
                set_name,
                element,
                ..
            } => {
                if let Some(comp) = self.components.get_mut(component_id) {
                    match set_name.as_str() {
                        "tags" => comp.metadata.tags.retain(|t| t != element),
                        "dependencies" => {
                            comp.dependencies.remove(element);
                        }
                        "links" => {
                            comp.links.remove(element);
                        }
                        "members" => {
                            comp.unordered_members.remove(element);
                        }
                        _ => {}
                    }
                }
            }
            CrdtOperation::AddEdge { edge, .. } => {
                if !self.edges.iter().any(|e| e.id == edge.id) {
                    self.edges.push(edge.clone());
                }
            }
            CrdtOperation::RemoveEdge { edge_id, .. } => {
                self.edges.retain(|e| &e.id != edge_id);
            }
        }
    }
}

// =============================================================================
// §13 — OS_BRIDGE ADAPTER  (backward-compat shim)
// =============================================================================

/// Request shape retained for backward compatibility with callers that
/// construct `NewPortfolioItem` to pass to the portfolio system.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewPortfolioItem {
    pub item_type: String,
    pub name: String,
    pub status: String,
}

impl PortfolioSystem {
    /// Backward-compatible entry-point: create a component from the old
    /// `NewPortfolioItem` shape and add it to the active portfolio's item-set.
    pub fn add_item(&mut self, req: NewPortfolioItem) -> Result<PortfolioComponent, String> {
        let ct = entity_type_to_component_type(&parse_entity_type(&req.item_type));
        let mut comp = self.create_component(ct, req.name)?;
        comp.status = req.status.clone();
        // Reflect status write-back
        if let Some(c) = self.components.get_mut(&comp.metadata.id) {
            c.status = req.status;
        }
        // Auto-link to active portfolio
        if let Some(ref pid) = self.active_portfolio_id.clone() {
            self.add_member(pid, &comp.metadata.id).ok();
        }
        Ok(comp)
    }
}

// Bridge: `PortfolioEntityType` (os_bridge) → `PortfolioComponentType`
fn entity_type_to_component_type(et: &PortfolioEntityType) -> PortfolioComponentType {
    match et {
        PortfolioEntityType::Portfolio => PortfolioComponentType::Portfolio,
        PortfolioEntityType::Project => PortfolioComponentType::Project,
        PortfolioEntityType::Program => PortfolioComponentType::Program,
        PortfolioEntityType::SubPortfolio => PortfolioComponentType::SubPortfolio,
        PortfolioEntityType::Artifact => PortfolioComponentType::Artifact,
        PortfolioEntityType::Asset => PortfolioComponentType::Asset,
        PortfolioEntityType::Document => PortfolioComponentType::Artifact,
        PortfolioEntityType::Custom => PortfolioComponentType::Resource,
        PortfolioEntityType::Resource
        | PortfolioEntityType::Capital
        | PortfolioEntityType::Investment
        | PortfolioEntityType::Account
        | PortfolioEntityType::Land
        | PortfolioEntityType::Estate
        | PortfolioEntityType::Labor => PortfolioComponentType::Resource,
    }
}

fn parse_entity_type(item_type: &str) -> PortfolioEntityType {
    let (kind, resource_type) = parse_portfolio_kind(item_type);
    match (kind, resource_type) {
        (PortfolioItemType::Project, _) => PortfolioEntityType::Project,
        (PortfolioItemType::Program, _) => PortfolioEntityType::Program,
        (PortfolioItemType::SubPortfolio, _) => PortfolioEntityType::SubPortfolio,
        (PortfolioItemType::Resource, Some(ResourceType::Artifact)) => {
            PortfolioEntityType::Artifact
        }
        (PortfolioItemType::Resource, Some(ResourceType::Asset)) => PortfolioEntityType::Asset,
        (PortfolioItemType::Resource, Some(ResourceType::Capital)) => {
            PortfolioEntityType::Capital
        }
        (PortfolioItemType::Resource, Some(ResourceType::Investment)) => {
            PortfolioEntityType::Investment
        }
        (PortfolioItemType::Resource, Some(ResourceType::Account)) => {
            PortfolioEntityType::Account
        }
        (PortfolioItemType::Resource, Some(ResourceType::Land)) => PortfolioEntityType::Land,
        (PortfolioItemType::Resource, Some(ResourceType::Estate)) => PortfolioEntityType::Estate,
        (PortfolioItemType::Resource, Some(ResourceType::Labor)) => PortfolioEntityType::Labor,
        (PortfolioItemType::Resource, None) => PortfolioEntityType::Resource,
    }
}

// =============================================================================
// §14 — COMPUTATIONAL MODELS
// =============================================================================
//
// Each portfolio component type gets a dedicated analytics / scoring model.
// Models are pure-function structs — they borrow data from `PortfolioSystem`
// (or from `PortfolioComponent` directly) and return typed result objects.
//
// Component-type → Model mapping:
//   Portfolio    → PortfolioHealthModel   (health score, resource roll-up)
//   Project      → ProjectMetricsModel    (schedule, CPI/SPI, risk score)
//   Program      → ProgramAlignmentModel  (benefit realisation, coherence)
//   SubPortfolio → SubPortfolioRollupModel (aggregated child metrics)
//   Resource     → ResourceUtilisationModel (capacity vs demand)
//   Asset        → AssetValueModel         (depreciation, ROI)
//   Artifact     → ArtifactMaturityModel   (completeness, currency, reuse)
//   Binder       → BinderCoverageModel     (membership completeness)
//   Book         → BookConsistencyModel    (page completeness by BookType)
//   Folder       → FolderOrganisationModel (depth, orphan detection)
//   Record       → RecordIntegrityModel    (sequence gaps, duplicate entries)

// =============================================================================
// 14.1 — PORTFOLIO HEALTH MODEL
// =============================================================================

/// Input data for the portfolio-level health computation.
#[derive(Clone, Debug)]
pub struct PortfolioHealthInput<'a> {
    pub portfolio: &'a PortfolioComponent,
    pub children: Vec<&'a PortfolioComponent>,
    pub allocation: Option<&'a ResourceAllocation>,
}

/// Scored output from `PortfolioHealthModel::compute`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortfolioHealthResult {
    /// Overall health score 0.0–100.0.
    pub health_score: f64,
    /// Fraction of child components with status == "active" (0.0–1.0).
    pub active_ratio: f64,
    /// Resource utilisation percentage (0.0 if no allocation).
    pub resource_utilisation_pct: f64,
    /// Total rolled-up resource units across all children.
    pub total_resource_units: f64,
    /// Human-readable summary string.
    pub summary: String,
}

pub struct PortfolioHealthModel;

impl PortfolioHealthModel {
    /// Compute a health score for a `Portfolio` component.
    ///
    /// Score formula:
    ///   * +50 pts for ≥ 80 % of children being active
    ///   * +30 pts inverse resource overrun penalty (0 if overrun > 20 %)
    ///   * +20 pts for having at least one child
    pub fn compute(input: &PortfolioHealthInput<'_>) -> PortfolioHealthResult {
        let total = input.children.len();
        let active = input
            .children
            .iter()
            .filter(|c| c.status == "active")
            .count();

        let active_ratio = if total == 0 {
            0.0
        } else {
            active as f64 / total as f64
        };

        let utilisation_pct = input
            .allocation
            .map(|a| a.utilisation_pct())
            .unwrap_or(0.0);

        let total_resource_units: f64 = input
            .children
            .iter()
            .map(|c| c.metadata.resource_units)
            .sum();

        let activity_pts = if active_ratio >= 0.8 { 50.0 } else { active_ratio * 62.5 };
        let resource_pts = if utilisation_pct > 120.0 {
            0.0
        } else {
            30.0 * (1.0 - ((utilisation_pct - 100.0).max(0.0) / 20.0))
        };
        let coverage_pts = if total > 0 { 20.0 } else { 0.0 };

        let health_score = (activity_pts + resource_pts + coverage_pts).clamp(0.0, 100.0);

        PortfolioHealthResult {
            health_score,
            active_ratio,
            resource_utilisation_pct: utilisation_pct,
            total_resource_units,
            summary: format!(
                "Health {:.1}/100 — {active}/{total} children active, \
                 resource utilisation {utilisation_pct:.1}%",
                health_score,
            ),
        }
    }
}

// =============================================================================
// 14.2 — PROJECT METRICS MODEL
// =============================================================================

/// Earned-value inputs for project-level schedule/cost analysis.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectMetricsInput {
    /// Planned value (budget authorised for work scheduled to date).
    pub planned_value: f64,
    /// Earned value (authorised budget for work actually completed).
    pub earned_value: f64,
    /// Actual cost incurred to date.
    pub actual_cost: f64,
    /// Budget at completion (total authorised budget).
    pub budget_at_completion: f64,
    /// Schedule variance weight used in risk score (0.0–1.0).
    pub schedule_risk_weight: f64,
    /// Number of open risk items logged against the project.
    pub open_risk_count: usize,
}

/// Earned-value and risk metrics for a `Project` component.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectMetricsResult {
    /// Cost Performance Index = EV / AC  (> 1 is under-budget).
    pub cpi: f64,
    /// Schedule Performance Index = EV / PV (> 1 is ahead of schedule).
    pub spi: f64,
    /// Cost Variance = EV − AC.
    pub cost_variance: f64,
    /// Schedule Variance = EV − PV.
    pub schedule_variance: f64,
    /// Estimate at Completion = BAC / CPI.
    pub estimate_at_completion: f64,
    /// Estimate to Complete = EAC − AC.
    pub estimate_to_complete: f64,
    /// Composite risk score 0.0–100.0 (higher = riskier).
    pub risk_score: f64,
    pub summary: String,
}

pub struct ProjectMetricsModel;

impl ProjectMetricsModel {
    pub fn compute(input: &ProjectMetricsInput) -> ProjectMetricsResult {
        let cpi = if input.actual_cost == 0.0 {
            1.0
        } else {
            input.earned_value / input.actual_cost
        };
        let spi = if input.planned_value == 0.0 {
            1.0
        } else {
            input.earned_value / input.planned_value
        };
        let cost_variance = input.earned_value - input.actual_cost;
        let schedule_variance = input.earned_value - input.planned_value;
        let eac = if cpi == 0.0 {
            f64::INFINITY
        } else {
            input.budget_at_completion / cpi
        };
        let etc = eac - input.actual_cost;

        // Risk score: blend schedule / cost deviation with open risk count.
        let cost_risk = (1.0 - cpi.min(1.0)) * 40.0;
        let sched_risk = (1.0 - spi.min(1.0)) * input.schedule_risk_weight * 40.0;
        let open_risk_pts = (input.open_risk_count as f64 * 5.0).min(20.0);
        let risk_score = (cost_risk + sched_risk + open_risk_pts).clamp(0.0, 100.0);

        ProjectMetricsResult {
            cpi,
            spi,
            cost_variance,
            schedule_variance,
            estimate_at_completion: eac,
            estimate_to_complete: etc,
            risk_score,
            summary: format!(
                "CPI={cpi:.2} SPI={spi:.2} EAC={eac:.0} risk={risk_score:.1}/100"
            ),
        }
    }
}

// =============================================================================
// 14.3 — PROGRAM ALIGNMENT MODEL
// =============================================================================

/// Input for measuring how well a program's projects align to strategic goals.
#[derive(Clone, Debug)]
pub struct ProgramAlignmentInput<'a> {
    pub program: &'a PortfolioComponent,
    /// Child projects/sub-programs.
    pub children: Vec<&'a PortfolioComponent>,
    /// Strategic goal IDs the program is meant to advance.
    pub strategic_goal_ids: Vec<String>,
    /// Strategic goal IDs actually covered by children (union of their tags).
    pub covered_goal_ids: Vec<String>,
    /// Planned benefit value (arbitrary unit).
    pub planned_benefit: f64,
    /// Realised benefit value to date.
    pub realised_benefit: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProgramAlignmentResult {
    /// Benefit Realisation Index = realised / planned (1.0 = on-track).
    pub benefit_realisation_index: f64,
    /// Fraction of strategic goals covered by child components (0.0–1.0).
    pub strategic_coverage: f64,
    /// Coherence score: proportion of active children (0.0–1.0).
    pub coherence: f64,
    /// Aggregate alignment score 0.0–100.0.
    pub alignment_score: f64,
    pub summary: String,
}

pub struct ProgramAlignmentModel;

impl ProgramAlignmentModel {
    pub fn compute(input: &ProgramAlignmentInput<'_>) -> ProgramAlignmentResult {
        let bri = if input.planned_benefit == 0.0 {
            1.0
        } else {
            input.realised_benefit / input.planned_benefit
        };

        let strategic_coverage = if input.strategic_goal_ids.is_empty() {
            1.0
        } else {
            let covered = input
                .covered_goal_ids
                .iter()
                .filter(|g| input.strategic_goal_ids.contains(g))
                .count();
            covered as f64 / input.strategic_goal_ids.len() as f64
        };

        let total = input.children.len();
        let active = input
            .children
            .iter()
            .filter(|c| c.status == "active")
            .count();
        let coherence = if total == 0 {
            0.0
        } else {
            active as f64 / total as f64
        };

        let alignment_score =
            (bri.min(1.0) * 40.0 + strategic_coverage * 40.0 + coherence * 20.0).clamp(0.0, 100.0);

        ProgramAlignmentResult {
            benefit_realisation_index: bri,
            strategic_coverage,
            coherence,
            alignment_score,
            summary: format!(
                "Alignment {alignment_score:.1}/100 — BRI={bri:.2} coverage={:.0}% coherence={:.0}%",
                strategic_coverage * 100.0,
                coherence * 100.0,
            ),
        }
    }
}

// =============================================================================
// 14.4 — SUB-PORTFOLIO ROLLUP MODEL
// =============================================================================

/// Aggregated metrics rolled up from all components within a `SubPortfolio`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubPortfolioRollupResult {
    pub total_components: usize,
    pub active_components: usize,
    pub total_resource_units: f64,
    pub total_budget_allocated: f64,
    pub total_budget_consumed: f64,
    /// Average health score of child portfolios/projects (0.0 if none).
    pub avg_child_health: f64,
    pub rollup_score: f64,
    pub summary: String,
}

pub struct SubPortfolioRollupModel;

impl SubPortfolioRollupModel {
    pub fn compute(
        children: &[&PortfolioComponent],
        allocations: &HashMap<String, ResourceAllocation>,
    ) -> SubPortfolioRollupResult {
        let total = children.len();
        let active = children.iter().filter(|c| c.status == "active").count();

        let total_resource_units: f64 = children.iter().map(|c| c.metadata.resource_units).sum();

        let (total_budget_allocated, total_budget_consumed) = children.iter().fold(
            (0.0_f64, 0.0_f64),
            |(alloc_acc, consumed_acc), c| {
                if let Some(a) = allocations.get(&c.metadata.id) {
                    (alloc_acc + a.allocated, consumed_acc + a.consumed)
                } else {
                    (alloc_acc, consumed_acc)
                }
            },
        );

        let active_ratio = if total == 0 {
            0.0
        } else {
            active as f64 / total as f64
        };
        let budget_health = if total_budget_allocated == 0.0 {
            1.0
        } else {
            (1.0 - ((total_budget_consumed / total_budget_allocated) - 1.0).max(0.0)).clamp(0.0, 1.0)
        };
        let rollup_score = (active_ratio * 60.0 + budget_health * 40.0).clamp(0.0, 100.0);

        SubPortfolioRollupResult {
            total_components: total,
            active_components: active,
            total_resource_units,
            total_budget_allocated,
            total_budget_consumed,
            avg_child_health: rollup_score,
            rollup_score,
            summary: format!(
                "Rollup {rollup_score:.1}/100 — {active}/{total} active, \
                 budget {total_budget_consumed:.0}/{total_budget_allocated:.0}"
            ),
        }
    }
}

// =============================================================================
// 14.5 — RESOURCE UTILISATION MODEL
// =============================================================================

/// Capacity planning inputs for a `Resource` component.
#[derive(Clone, Debug)]
pub struct ResourceUtilisationInput {
    /// Total capacity available in this resource (e.g. hours/sprint, FTEs).
    pub total_capacity: f64,
    /// Capacity already committed / allocated to work items.
    pub committed: f64,
    /// Capacity actually consumed in the last measurement period.
    pub consumed: f64,
    /// Number of work items (tasks/stories) assigned.
    pub assigned_items: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceUtilisationResult {
    /// Commitment ratio = committed / total_capacity.
    pub commitment_ratio: f64,
    /// Consumption ratio = consumed / total_capacity.
    pub consumption_ratio: f64,
    /// Slack = total_capacity − committed (may be negative when over-committed).
    pub slack: f64,
    /// Efficiency = consumed / committed (1.0 = perfect forecast).
    pub efficiency: f64,
    /// Utilisation health score 0.0–100.0 (penalises over-commitment & idleness).
    pub utilisation_score: f64,
    pub summary: String,
}

pub struct ResourceUtilisationModel;

impl ResourceUtilisationModel {
    pub fn compute(input: &ResourceUtilisationInput) -> ResourceUtilisationResult {
        let commitment_ratio = if input.total_capacity == 0.0 {
            0.0
        } else {
            input.committed / input.total_capacity
        };
        let consumption_ratio = if input.total_capacity == 0.0 {
            0.0
        } else {
            input.consumed / input.total_capacity
        };
        let slack = input.total_capacity - input.committed;
        let efficiency = if input.committed == 0.0 {
            1.0
        } else {
            (input.consumed / input.committed).min(2.0)
        };

        // Score penalises both over-commitment (> 100 %) and severe under-use (< 40 %).
        let over_commit_penalty = ((commitment_ratio - 1.0).max(0.0) * 100.0).min(60.0);
        let idle_penalty = if commitment_ratio < 0.4 {
            (0.4 - commitment_ratio) * 50.0
        } else {
            0.0
        };
        let utilisation_score = (100.0 - over_commit_penalty - idle_penalty).clamp(0.0, 100.0);

        ResourceUtilisationResult {
            commitment_ratio,
            consumption_ratio,
            slack,
            efficiency,
            utilisation_score,
            summary: format!(
                "Utilisation {utilisation_score:.1}/100 — committed={:.0}% consumed={:.0}% slack={slack:.1}",
                commitment_ratio * 100.0,
                consumption_ratio * 100.0,
            ),
        }
    }
}

// =============================================================================
// 14.6 — ASSET VALUE MODEL
// =============================================================================

/// Financial inputs for an `Asset` component.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetValueInput {
    /// Original acquisition cost.
    pub acquisition_cost: f64,
    /// Current book value after depreciation.
    pub current_book_value: f64,
    /// Estimated market / fair value.
    pub market_value: f64,
    /// Total income / return generated by the asset to date.
    pub total_return: f64,
    /// Age of the asset in periods (e.g. months or years).
    pub age_periods: f64,
    /// Expected useful-life in the same unit as `age_periods`.
    pub useful_life_periods: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetValueResult {
    /// Return on Investment = total_return / acquisition_cost.
    pub roi: f64,
    /// Straight-line depreciation rate per period.
    pub depreciation_rate: f64,
    /// Proportion of useful life consumed (0.0–1.0+).
    pub life_consumed_ratio: f64,
    /// Premium/discount: (market_value − book_value) / book_value.
    pub market_to_book_ratio: f64,
    /// Asset health score 0.0–100.0 (high ROI + young asset = healthy).
    pub asset_score: f64,
    pub summary: String,
}

pub struct AssetValueModel;

impl AssetValueModel {
    pub fn compute(input: &AssetValueInput) -> AssetValueResult {
        let roi = if input.acquisition_cost == 0.0 {
            0.0
        } else {
            input.total_return / input.acquisition_cost
        };

        let depreciation_rate = if input.useful_life_periods == 0.0 {
            0.0
        } else {
            1.0 / input.useful_life_periods
        };

        let life_consumed_ratio = if input.useful_life_periods == 0.0 {
            0.0
        } else {
            input.age_periods / input.useful_life_periods
        };

        let market_to_book_ratio = if input.current_book_value == 0.0 {
            1.0
        } else {
            input.market_value / input.current_book_value
        };

        // Score: positive ROI + good market-to-book + young asset.
        let roi_pts = (roi * 40.0).clamp(0.0, 40.0);
        let mtb_pts = ((market_to_book_ratio - 1.0).clamp(-1.0, 1.0) * 30.0 + 30.0).clamp(0.0, 30.0);
        let age_pts = ((1.0 - life_consumed_ratio) * 30.0).clamp(0.0, 30.0);
        let asset_score = (roi_pts + mtb_pts + age_pts).clamp(0.0, 100.0);

        AssetValueResult {
            roi,
            depreciation_rate,
            life_consumed_ratio,
            market_to_book_ratio,
            asset_score,
            summary: format!(
                "Asset score {asset_score:.1}/100 — ROI={:.1}% MTB={market_to_book_ratio:.2} life={:.0}%",
                roi * 100.0,
                life_consumed_ratio * 100.0,
            ),
        }
    }
}

// =============================================================================
// 14.7 — ARTIFACT MATURITY MODEL
// =============================================================================

/// Quality / reuse inputs for an `Artifact` component.
#[derive(Clone, Debug)]
pub struct ArtifactMaturityInput {
    /// Number of required metadata fields that are populated (0–`required_fields_total`).
    pub required_fields_populated: usize,
    pub required_fields_total: usize,
    /// Age of the artifact in days since last update.
    pub days_since_update: u64,
    /// Maximum tolerable staleness before a penalty is applied (days).
    pub max_fresh_days: u64,
    /// Number of components that reference / reuse this artifact.
    pub reuse_count: usize,
    /// Whether the artifact has been formally reviewed/approved.
    pub is_reviewed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtifactMaturityResult {
    /// Completeness ratio (0.0–1.0).
    pub completeness: f64,
    /// Currency ratio: 1.0 if fresh, decays toward 0.0 when stale.
    pub currency: f64,
    /// Reuse score component (logarithmic).
    pub reuse_score_component: f64,
    /// Overall maturity score 0.0–100.0.
    pub maturity_score: f64,
    pub summary: String,
}

pub struct ArtifactMaturityModel;

impl ArtifactMaturityModel {
    pub fn compute(input: &ArtifactMaturityInput) -> ArtifactMaturityResult {
        let completeness = if input.required_fields_total == 0 {
            1.0
        } else {
            input.required_fields_populated as f64 / input.required_fields_total as f64
        };

        let currency = if input.days_since_update <= input.max_fresh_days {
            1.0
        } else {
            let overage = (input.days_since_update - input.max_fresh_days) as f64;
            (1.0 - overage / input.max_fresh_days as f64).clamp(0.0, 1.0)
        };

        // Log-scale reuse: ln(reuse+1) / ln(11) gives 0.0 at 0 reuses, ~1.0 at 10.
        let reuse_score_component = ((input.reuse_count as f64 + 1.0).ln() / 11_f64.ln()).min(1.0);

        let review_pts = if input.is_reviewed { 10.0 } else { 0.0 };
        let maturity_score = (completeness * 40.0
            + currency * 30.0
            + reuse_score_component * 20.0
            + review_pts)
            .clamp(0.0, 100.0);

        ArtifactMaturityResult {
            completeness,
            currency,
            reuse_score_component,
            maturity_score,
            summary: format!(
                "Maturity {maturity_score:.1}/100 — complete={:.0}% currency={:.0}% reuse={}",
                completeness * 100.0,
                currency * 100.0,
                input.reuse_count,
            ),
        }
    }
}

// =============================================================================
// 14.8 — BINDER COVERAGE MODEL
// =============================================================================

/// How well a `Binder` covers the expected set of child IDs.
#[derive(Clone, Debug)]
pub struct BinderCoverageInput {
    /// IDs that should ideally be in the binder.
    pub expected_ids: HashSet<String>,
    /// IDs actually present in the binder.
    pub actual_ids: HashSet<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BinderCoverageResult {
    /// Fraction of expected IDs that are present.
    pub coverage_ratio: f64,
    /// IDs that are expected but absent.
    pub missing: Vec<String>,
    /// IDs present but not in the expected set (unexpected members).
    pub extra: Vec<String>,
    /// Coverage score 0.0–100.0.
    pub coverage_score: f64,
    pub summary: String,
}

pub struct BinderCoverageModel;

impl BinderCoverageModel {
    pub fn compute(input: &BinderCoverageInput) -> BinderCoverageResult {
        let mut missing: Vec<String> = input
            .expected_ids
            .difference(&input.actual_ids)
            .cloned()
            .collect();
        missing.sort();

        let mut extra: Vec<String> = input
            .actual_ids
            .difference(&input.expected_ids)
            .cloned()
            .collect();
        extra.sort();

        let coverage_ratio = if input.expected_ids.is_empty() {
            1.0
        } else {
            let found = input.expected_ids.intersection(&input.actual_ids).count();
            found as f64 / input.expected_ids.len() as f64
        };

        let extra_penalty = ((extra.len() as f64 / (input.actual_ids.len() as f64 + 1.0)) * 10.0).min(10.0);
        let coverage_score = (coverage_ratio * 100.0 - extra_penalty).clamp(0.0, 100.0);

        BinderCoverageResult {
            coverage_ratio,
            missing: missing.clone(),
            extra: extra.clone(),
            coverage_score,
            summary: format!(
                "Coverage {coverage_score:.1}/100 — {:.0}% present, {} missing, {} extra",
                coverage_ratio * 100.0,
                missing.len(),
                extra.len(),
            ),
        }
    }
}

// =============================================================================
// 14.9 — BOOK CONSISTENCY MODEL
// =============================================================================

/// Consistency inputs for a `Book` component (any `BookType`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BookConsistencyInput {
    pub book_type: BookType,
    /// Number of pages / entries present.
    pub page_count: usize,
    /// Minimum expected pages for the given `BookType`.
    pub min_expected_pages: usize,
    /// Number of pages that have been reviewed / approved.
    pub reviewed_pages: usize,
    /// Number of broken internal cross-references detected.
    pub broken_refs: usize,
    /// Whether the book has a table of contents / index.
    pub has_index: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BookConsistencyResult {
    /// Page completeness ratio (capped at 1.0 when over-supplied).
    pub page_completeness: f64,
    /// Review coverage ratio.
    pub review_coverage: f64,
    /// Reference integrity score (1.0 = zero broken refs).
    pub reference_integrity: f64,
    /// Overall consistency score 0.0–100.0.
    pub consistency_score: f64,
    pub summary: String,
}

pub struct BookConsistencyModel;

impl BookConsistencyModel {
    pub fn compute(input: &BookConsistencyInput) -> BookConsistencyResult {
        let page_completeness = if input.min_expected_pages == 0 {
            1.0
        } else {
            (input.page_count as f64 / input.min_expected_pages as f64).min(1.0)
        };

        let review_coverage = if input.page_count == 0 {
            1.0
        } else {
            input.reviewed_pages as f64 / input.page_count as f64
        };

        let reference_integrity = if input.page_count == 0 {
            1.0
        } else {
            (1.0 - input.broken_refs as f64 / input.page_count as f64).clamp(0.0, 1.0)
        };

        let index_pts = if input.has_index { 10.0 } else { 0.0 };
        let consistency_score = (page_completeness * 40.0
            + review_coverage * 30.0
            + reference_integrity * 20.0
            + index_pts)
            .clamp(0.0, 100.0);

        BookConsistencyResult {
            page_completeness,
            review_coverage,
            reference_integrity,
            consistency_score,
            summary: format!(
                "[{:?}] Consistency {consistency_score:.1}/100 — \
                 complete={:.0}% reviewed={:.0}% integrity={:.0}%",
                input.book_type,
                page_completeness * 100.0,
                review_coverage * 100.0,
                reference_integrity * 100.0,
            ),
        }
    }
}

// =============================================================================
// 14.10 — FOLDER ORGANISATION MODEL
// =============================================================================

/// Structural inputs for a `Folder` component.
#[derive(Clone, Debug)]
pub struct FolderOrganisationInput {
    /// Maximum depth of nesting observed across all sub-folders.
    pub max_depth: usize,
    /// Recommended max depth before penalising.
    pub depth_threshold: usize,
    /// Number of items (non-folder) directly in this folder.
    pub direct_item_count: usize,
    /// Number of orphaned items (items with no parent that should have one).
    pub orphan_count: usize,
    /// Number of duplicate-named entries.
    pub duplicate_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FolderOrganisationResult {
    /// Depth penalty ratio (0.0 = at/below threshold; 1.0 = far exceeds it).
    pub depth_penalty: f64,
    /// Organisation score 0.0–100.0.
    pub organisation_score: f64,
    pub summary: String,
}

pub struct FolderOrganisationModel;

impl FolderOrganisationModel {
    pub fn compute(input: &FolderOrganisationInput) -> FolderOrganisationResult {
        let depth_penalty = if input.max_depth <= input.depth_threshold {
            0.0
        } else {
            ((input.max_depth - input.depth_threshold) as f64 / input.depth_threshold as f64)
                .clamp(0.0, 1.0)
        };

        let orphan_penalty = (input.orphan_count as f64 * 5.0).min(30.0);
        let dupe_penalty = (input.duplicate_count as f64 * 5.0).min(20.0);
        let depth_pts = depth_penalty * 30.0;

        let organisation_score =
            (100.0 - depth_pts - orphan_penalty - dupe_penalty).clamp(0.0, 100.0);

        FolderOrganisationResult {
            depth_penalty,
            organisation_score,
            summary: format!(
                "Organisation {organisation_score:.1}/100 — depth={} orphans={} dupes={}",
                input.max_depth, input.orphan_count, input.duplicate_count,
            ),
        }
    }
}

// =============================================================================
// 14.11 — RECORD INTEGRITY MODEL
// =============================================================================

/// Sequence-integrity inputs for a `Record` component.
#[derive(Clone, Debug)]
pub struct RecordIntegrityInput {
    /// Total entries in the record's ordered list.
    pub entry_count: usize,
    /// Number of entries whose sequence number is out of expected order.
    pub out_of_order_count: usize,
    /// Number of detected duplicate entries.
    pub duplicate_count: usize,
    /// Number of entries missing required fields.
    pub incomplete_entries: usize,
    /// Whether the record has a hash / checksum for tamper detection.
    pub has_integrity_hash: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecordIntegrityResult {
    /// Sequencing quality ratio (1.0 = perfectly ordered).
    pub sequence_quality: f64,
    /// Uniqueness ratio (1.0 = no duplicates).
    pub uniqueness_ratio: f64,
    /// Completeness ratio (1.0 = all entries have required fields).
    pub completeness_ratio: f64,
    /// Overall integrity score 0.0–100.0.
    pub integrity_score: f64,
    pub summary: String,
}

pub struct RecordIntegrityModel;

impl RecordIntegrityModel {
    pub fn compute(input: &RecordIntegrityInput) -> RecordIntegrityResult {
        let base = input.entry_count.max(1) as f64;

        let sequence_quality =
            (1.0 - input.out_of_order_count as f64 / base).clamp(0.0, 1.0);
        let uniqueness_ratio =
            (1.0 - input.duplicate_count as f64 / base).clamp(0.0, 1.0);
        let completeness_ratio =
            (1.0 - input.incomplete_entries as f64 / base).clamp(0.0, 1.0);

        let hash_pts = if input.has_integrity_hash { 10.0 } else { 0.0 };
        let integrity_score = (sequence_quality * 35.0
            + uniqueness_ratio * 30.0
            + completeness_ratio * 25.0
            + hash_pts)
            .clamp(0.0, 100.0);

        RecordIntegrityResult {
            sequence_quality,
            uniqueness_ratio,
            completeness_ratio,
            integrity_score,
            summary: format!(
                "Integrity {integrity_score:.1}/100 — seq={:.0}% unique={:.0}% complete={:.0}%",
                sequence_quality * 100.0,
                uniqueness_ratio * 100.0,
                completeness_ratio * 100.0,
            ),
        }
    }
}

// =============================================================================
// 14.12 — COMPUTATIONAL MODEL REGISTRY  (PortfolioSystem integration)
// =============================================================================

impl PortfolioSystem {
    // ── Portfolio health ──────────────────────────────────────────────────

    /// Compute health for a `Portfolio` or `SubPortfolio` component.
    pub fn compute_portfolio_health(&self, portfolio_id: &str) -> Option<PortfolioHealthResult> {
        let portfolio = self.components.get(portfolio_id)?;
        let children: Vec<&PortfolioComponent> = portfolio
            .children
            .iter()
            .filter_map(|id| self.components.get(id))
            .collect();
        let allocation = self.resource_allocations.get(portfolio_id);
        Some(PortfolioHealthModel::compute(&PortfolioHealthInput {
            portfolio,
            children,
            allocation,
        }))
    }

    // ── Sub-portfolio rollup ──────────────────────────────────────────────

    pub fn compute_subportfolio_rollup(
        &self,
        subportfolio_id: &str,
    ) -> Option<SubPortfolioRollupResult> {
        let sp = self.components.get(subportfolio_id)?;
        let children: Vec<&PortfolioComponent> = sp
            .children
            .iter()
            .filter_map(|id| self.components.get(id))
            .collect();
        Some(SubPortfolioRollupModel::compute(&children, &self.resource_allocations))
    }

    // ── Resource utilisation ──────────────────────────────────────────────

    /// Compute utilisation for a `Resource` component using its metadata fields.
    pub fn compute_resource_utilisation(
        &self,
        resource_id: &str,
    ) -> Option<ResourceUtilisationResult> {
        let comp = self.components.get(resource_id)?;
        if comp.component_type != PortfolioComponentType::Resource {
            return None;
        }
        let alloc = self.resource_allocations.get(resource_id);
        let total_capacity = alloc.map(|a| a.total).unwrap_or(comp.metadata.resource_units);
        let committed = alloc.map(|a| a.allocated).unwrap_or(0.0);
        let consumed = alloc.map(|a| a.consumed).unwrap_or(comp.metadata.budget_spent);
        let assigned_items = comp.unordered_members.len() + comp.ordered_members.len();
        Some(ResourceUtilisationModel::compute(&ResourceUtilisationInput {
            total_capacity,
            committed,
            consumed,
            assigned_items,
        }))
    }

    // ── Artifact maturity ────────────────────────────────────────────────

    /// Compute maturity for an `Artifact` using its metadata.
    ///
    /// `required_fields` is a list of property-key names that must be present.
    pub fn compute_artifact_maturity(
        &self,
        artifact_id: &str,
        required_fields: &[&str],
        max_fresh_days: u64,
    ) -> Option<ArtifactMaturityResult> {
        let comp = self.components.get(artifact_id)?;
        if comp.component_type != PortfolioComponentType::Artifact {
            return None;
        }
        let ts_now = now();
        let days_since_update = (ts_now - comp.metadata.updated_at) / 86_400;
        let populated = required_fields
            .iter()
            .filter(|k| comp.metadata.properties.contains_key(**k))
            .count();
        let is_reviewed = comp
            .metadata
            .properties
            .get("reviewed")
            .map(|v| v == "true")
            .unwrap_or(false);
        let reuse_count = comp.dependents.len();

        Some(ArtifactMaturityModel::compute(&ArtifactMaturityInput {
            required_fields_populated: populated,
            required_fields_total: required_fields.len(),
            days_since_update,
            max_fresh_days,
            reuse_count,
            is_reviewed,
        }))
    }

    // ── Binder coverage ──────────────────────────────────────────────────

    pub fn compute_binder_coverage(
        &self,
        binder_id: &str,
        expected_ids: HashSet<String>,
    ) -> Option<BinderCoverageResult> {
        let binder = self.components.get(binder_id)?;
        if binder.component_type != PortfolioComponentType::Binder {
            return None;
        }
        Some(BinderCoverageModel::compute(&BinderCoverageInput {
            expected_ids,
            actual_ids: binder.unordered_members.clone(),
        }))
    }

    // ── Record integrity ─────────────────────────────────────────────────

    pub fn compute_record_integrity(&self, record_id: &str) -> Option<RecordIntegrityResult> {
        let record = self.components.get(record_id)?;
        if record.component_type != PortfolioComponentType::Record {
            return None;
        }
        let entry_count = record.ordered_members.len();
        // Detect duplicates via a frequency count.
        let mut seen = HashMap::new();
        let mut duplicate_count = 0usize;
        for m in &record.ordered_members {
            let cnt = seen.entry(m.clone()).or_insert(0usize);
            *cnt += 1;
            if *cnt == 2 {
                duplicate_count += 1;
            }
        }
        let has_integrity_hash = record
            .metadata
            .properties
            .contains_key("integrity_hash");

        Some(RecordIntegrityModel::compute(&RecordIntegrityInput {
            entry_count,
            out_of_order_count: 0, // ordering check requires domain-specific sequence keys
            duplicate_count,
            incomplete_entries: 0, // completeness requires schema awareness
            has_integrity_hash,
        }))
    }

    // ── Folder organisation ───────────────────────────────────────────────

    /// Compute organisation score for a `Folder` component.
    ///
    /// `depth_threshold` is the max tolerable nesting depth before a penalty.
    pub fn compute_folder_organisation(
        &self,
        folder_id: &str,
        depth_threshold: usize,
    ) -> Option<FolderOrganisationResult> {
        let folder = self.components.get(folder_id)?;
        if folder.component_type != PortfolioComponentType::Folder {
            return None;
        }
        // BFS to find max depth below this folder.
        let max_depth = self.folder_max_depth(folder_id, 0);
        let direct_item_count = folder
            .unordered_members
            .iter()
            .filter(|id| {
                self.components
                    .get(id.as_str())
                    .map(|c| !c.is_container())
                    .unwrap_or(false)
            })
            .count();
        // Orphans: members that list no parent or list a different parent.
        let orphan_count = folder
            .unordered_members
            .iter()
            .filter(|id| {
                self.components
                    .get(id.as_str())
                    .map(|c| !c.parents.contains(folder_id))
                    .unwrap_or(true)
            })
            .count();
        // Duplicate names inside this folder.
        let mut name_freq: HashMap<&str, usize> = HashMap::new();
        for id in &folder.unordered_members {
            if let Some(c) = self.components.get(id) {
                *name_freq.entry(c.name.as_str()).or_insert(0) += 1;
            }
        }
        let duplicate_count = name_freq.values().filter(|&&n| n > 1).count();

        Some(FolderOrganisationModel::compute(&FolderOrganisationInput {
            max_depth,
            depth_threshold,
            direct_item_count,
            orphan_count,
            duplicate_count,
        }))
    }

    /// Recursive DFS depth calculation for folders.
    fn folder_max_depth(&self, folder_id: &str, current_depth: usize) -> usize {
        let folder = match self.components.get(folder_id) {
            Some(f) => f,
            None => return current_depth,
        };
        let sub_folders: Vec<&str> = folder
            .unordered_members
            .iter()
            .filter(|id| {
                self.components
                    .get(id.as_str())
                    .map(|c| c.component_type == PortfolioComponentType::Folder)
                    .unwrap_or(false)
            })
            .map(String::as_str)
            .collect();

        if sub_folders.is_empty() {
            current_depth
        } else {
            sub_folders
                .iter()
                .map(|id| self.folder_max_depth(id, current_depth + 1))
                .max()
                .unwrap_or(current_depth)
        }
    }
}
