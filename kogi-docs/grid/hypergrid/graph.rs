// =============================================================================
// hypergrid::graph — HG-GRAPH: Hypergraph Engine
//
// Hypergraph, HypergraphNode, HypergraphEdge, EdgeType, EdgeDirection,
// ConsentStatus, ShadowCell, graph traversal algorithms, cycle detection
// =============================================================================

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::cell::{ActorId, AttributeKey, CubeId, DimKey, EntityId, GridId, TypedAttrValue, UserId};
use crate::error::HypergridError;

pub type EdgeId  = Uuid;
pub type NodeRef = (CubeId, EntityId);  // (cube, d1_key)

// ─── EdgeType ─────────────────────────────────────────────────────────────────

/// Typed edge classification for HypergraphEdge.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeType {
    /// Parent-child containment hierarchy. Enables ROLLUP/DRILLDOWN.
    /// No cycle detection enforced (hierarchies can share children in DAGs).
    Hierarchy,

    /// Blocking dependency: 'from' cannot complete before 'to' completes.
    /// DFS cycle detection enforced at edge creation time.
    Dependency,

    /// Soft lateral reference. No blocking semantics. Informational only.
    Association,

    /// Ownership containment (Space/Container → Item).
    Contains,

    /// Derivation/fork: 'from' was derived from 'to'. Used for versioning.
    Derives,

    /// Cross-grid link: connects entities in different Grid deployments.
    /// Consent required from target entity owner. Triggers ShadowCell provisioning.
    CrossGridLink,

    /// System edge: marks a ShadowRow as a reflection of a source entity.
    /// Created automatically when a CrossGridLink is accepted.
    ShadowOf,

    /// Identity is a member of a Space.
    SpaceMembership,

    /// Bidirectional collaboration agreement. Both parties must consent.
    Collaborates,

    /// Economic investment relationship. Target consents.
    InvestedIn,

    /// Employment/contracting edge. Worker consents.
    Employs,

    /// Contracting (project-based work). Worker consents.
    Contracted,

    /// Follow/subscribe relationship.
    Follows,

    /// Endorsement relationship.
    Endorses,

    /// Federation peer link between two Grid deployments.
    FederationPeer,

    /// Namespace path alias (one namespace redirects to another).
    NamespaceAlias,

    /// Provenance: this cell was computed from these source cells.
    ComputedFrom,

    /// Cooperative/org membership.
    OrgMembership,

    /// Custom domain-specific edge type (registered by domain system).
    Custom(String),
}

impl EdgeType {
    /// Does this edge type require explicit consent from the target?
    pub fn requires_consent(&self) -> bool {
        matches!(self,
            Self::CrossGridLink | Self::Collaborates | Self::InvestedIn |
            Self::Employs | Self::Contracted | Self::OrgMembership
        )
    }

    /// Does this edge type trigger ShadowCell provisioning when accepted?
    pub fn creates_shadow(&self) -> bool {
        matches!(self,
            Self::CrossGridLink | Self::Collaborates | Self::InvestedIn |
            Self::Employs | Self::Contracted
        )
    }

    /// Must DFS cycle detection be enforced when adding this edge?
    pub fn enforce_cycle_detection(&self) -> bool {
        matches!(self, Self::Dependency)
    }
}

// ─── EdgeDirection ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeDirection {
    Directed,       // from_node → to_node only
    Bidirectional,  // both directions
}

// ─── ConsentStatus ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsentStatus {
    None,       // consent not required for this edge type
    Pending,    // invitation sent; awaiting target's response
    Accepted,   // target has accepted; ShadowCell may proceed
    Declined,   // target declined; edge remains but no sync
    Revoked,    // previously accepted; now revoked; sync halted
}

// ─── ShadowCellConfig ────────────────────────────────────────────────────────

/// Configuration for ShadowCell provisioned by a CrossGridLink or similar edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowCellConfig {
    /// Which attributes from the source entity are mirrored into the shadow.
    pub mirrored_attrs:   Vec<AttributeKey>,
    /// Which mirrored attributes the host may write back to the source.
    pub writeback_attrs:  Vec<AttributeKey>,
    /// How often to sync: RealTime (Kafka push), Batched (scheduled), Manual.
    pub update_policy:    ShadowUpdatePolicy,
    pub consented_at:     DateTime<Utc>,
    pub consented_by:     ActorId,
    pub expires_at:       Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShadowUpdatePolicy {
    /// Push-based: updated within Kafka event processing latency (<1s).
    RealTime,
    /// Pull-based: updated on a configurable schedule (e.g., every 5 minutes).
    Batched { interval_seconds: u64 },
    /// Manual: only updated when explicitly requested via API.
    Manual,
}

// ─── HypergraphNode ───────────────────────────────────────────────────────────

/// A node in the Hypergraph. Every entity in every Hypercube is simultaneously
/// a node in the Hypergraph. Nodes are created implicitly when HyperRows are created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypergraphNode {
    pub node_id:    EdgeId,           // locally unique node identifier
    pub cube_id:    CubeId,
    pub d1_key:     DimKey,           // the entity's D₁ key in its cube
    pub grid_id:    GridId,
    pub node_type:  NodeType,
    pub namespace_path: Option<String>,
    pub is_shadow:  bool,             // true if this is a ShadowRow node
    pub created_at: DateTime<Utc>,
}

impl HypergraphNode {
    pub fn new(cube_id: CubeId, d1_key: DimKey, grid_id: GridId) -> Self {
        Self {
            node_id:    Uuid::new_v4(),
            cube_id, d1_key, grid_id,
            node_type:  NodeType::Row,
            namespace_path: None,
            is_shadow:  false,
            created_at: Utc::now(),
        }
    }

    pub fn entity_ref(&self) -> (CubeId, DimKey) { (self.cube_id, self.d1_key.clone()) }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    Row,    // a HyperRow entity
    Cube,   // a Hypercube itself (for cube-level graph relationships)
    Grid,   // a federation peer Grid
    Space,  // a Space
    Shadow, // a ShadowRow (reflection of a remote entity)
}

// ─── HypergraphEdge ───────────────────────────────────────────────────────────

/// A typed, directed, weighted, consent-gated edge between two Hypergraph nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypergraphEdge {
    pub edge_id:       EdgeId,
    pub from_node:     EdgeId,         // source HypergraphNode.node_id
    pub to_node:       EdgeId,         // target HypergraphNode.node_id
    pub edge_type:     EdgeType,
    pub direction:     EdgeDirection,
    pub weight:        f64,            // 0.0–1.0; used by path-finding algorithms

    // Edge-level attributes (typed, can carry arbitrary data)
    pub attributes:    HashMap<AttributeKey, TypedAttrValue>,

    // Consent and shadow cell configuration
    pub consent_status:  ConsentStatus,
    pub shadow_config:   Option<ShadowCellConfig>,
    pub shadow_cell_id:  Option<Uuid>,  // populated after shadow provisioned

    // Lifecycle
    pub space_scope:   Option<crate::cell::SpaceId>,  // None = global scope
    pub expires_at:    Option<DateTime<Utc>>,
    pub created_at:    DateTime<Utc>,
    pub created_by:    ActorId,
}

impl HypergraphEdge {
    pub fn new(
        from_node: EdgeId, to_node: EdgeId,
        edge_type: EdgeType, created_by: ActorId,
    ) -> Self {
        let consent = if edge_type.requires_consent() {
            ConsentStatus::Pending
        } else {
            ConsentStatus::None
        };
        Self {
            edge_id:      Uuid::new_v4(),
            from_node, to_node, created_by,
            edge_type,
            direction:    EdgeDirection::Directed,
            weight:       1.0,
            attributes:   HashMap::new(),
            consent_status: consent,
            shadow_config:  None,
            shadow_cell_id: None,
            space_scope:  None,
            expires_at:   None,
            created_at:   Utc::now(),
        }
    }

    pub fn is_active(&self) -> bool {
        if let Some(exp) = self.expires_at {
            if Utc::now() > exp { return false; }
        }
        !matches!(self.consent_status, ConsentStatus::Declined | ConsentStatus::Revoked)
    }

    pub fn accept_consent(mut self, config: ShadowCellConfig) -> Self {
        self.consent_status = ConsentStatus::Accepted;
        self.shadow_config = Some(config);
        self
    }

    pub fn revoke(mut self) -> Self {
        self.consent_status = ConsentStatus::Revoked;
        self
    }
}

// ─── AdjacencyIndex ──────────────────────────────────────────────────────────

/// Fast adjacency lookup: NodeId → (outbound edge IDs, inbound edge IDs).
#[derive(Debug, Default)]
struct AdjacencyIndex {
    outbound: HashMap<EdgeId, Vec<EdgeId>>,  // from_node → [edge_id]
    inbound:  HashMap<EdgeId, Vec<EdgeId>>,  // to_node   → [edge_id]
}

impl AdjacencyIndex {
    fn add_edge(&mut self, edge: &HypergraphEdge) {
        self.outbound.entry(edge.from_node).or_default().push(edge.edge_id);
        self.inbound.entry(edge.to_node).or_default().push(edge.edge_id);
        if edge.direction == EdgeDirection::Bidirectional {
            self.outbound.entry(edge.to_node).or_default().push(edge.edge_id);
            self.inbound.entry(edge.from_node).or_default().push(edge.edge_id);
        }
    }

    fn remove_edge(&mut self, edge: &HypergraphEdge) {
        for list in [
            self.outbound.get_mut(&edge.from_node),
            self.inbound.get_mut(&edge.to_node),
        ].into_iter().flatten() {
            list.retain(|&id| id != edge.edge_id);
        }
    }

    fn outbound(&self, node: EdgeId) -> &[EdgeId] {
        self.outbound.get(&node).map(|v| v.as_slice()).unwrap_or(&[])
    }

    fn inbound(&self, node: EdgeId) -> &[EdgeId] {
        self.inbound.get(&node).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

// ─── TraversalConfig ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TraversalConfig {
    pub max_depth:    u8,
    pub min_depth:    u8,
    pub edge_filter:  Option<EdgeTypeFilter>,
    pub direction:    TraversalDirection,
}

impl Default for TraversalConfig {
    fn default() -> Self {
        Self { max_depth: 3, min_depth: 0, edge_filter: None, direction: TraversalDirection::Outbound }
    }
}

#[derive(Debug, Clone)]
pub enum EdgeTypeFilter {
    Only(Vec<EdgeType>),
    Exclude(Vec<EdgeType>),
}

impl EdgeTypeFilter {
    pub fn matches(&self, et: &EdgeType) -> bool {
        match self {
            Self::Only(types)    => types.contains(et),
            Self::Exclude(types) => !types.contains(et),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraversalDirection { Outbound, Inbound, Both }

// ─── TraversalResult ─────────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct TraversalResult {
    pub nodes:       Vec<(EdgeId, u8)>,   // (node_id, depth)
    pub edges:       Vec<EdgeId>,
    pub node_set:    HashSet<EdgeId>,
}

impl TraversalResult {
    pub fn add_node(&mut self, node_id: EdgeId, depth: u8) {
        if self.node_set.insert(node_id) {
            self.nodes.push((node_id, depth));
        }
    }
    pub fn add_edge(&mut self, edge_id: EdgeId) { self.edges.push(edge_id); }
    pub fn contains_node(&self, node_id: EdgeId) -> bool { self.node_set.contains(&node_id) }
    pub fn total_nodes(&self) -> usize { self.nodes.len() }
}

// ─── Hypergraph ───────────────────────────────────────────────────────────────

/// The Hypergraph: a typed, directed property graph over any entities in any Hypercube.
///
/// Every entity in every Hypercube is automatically registered as a node.
/// Edges are typed, weighted, and consent-gated.
/// Provides BFS/DFS traversal, cycle detection, and path finding.
pub struct Hypergraph {
    pub grid_id: GridId,
    nodes:   RwLock<HashMap<EdgeId, HypergraphNode>>,
    edges:   RwLock<HashMap<EdgeId, HypergraphEdge>>,
    adj:     RwLock<AdjacencyIndex>,
    // Entity → NodeId lookup: (cube_id, d1_key) → node_id
    entity_index: RwLock<HashMap<(CubeId, String), EdgeId>>,
}

impl Hypergraph {
    pub fn new(grid_id: GridId) -> Self {
        Self {
            grid_id,
            nodes:  Default::default(),
            edges:  Default::default(),
            adj:    Default::default(),
            entity_index: Default::default(),
        }
    }

    // ── Node management ───────────────────────────────────────────────────

    pub fn register_entity(&self, cube_id: CubeId, d1_key: DimKey, grid_id: GridId) -> EdgeId {
        let node = HypergraphNode::new(cube_id, d1_key.clone(), grid_id);
        let node_id = node.node_id;
        let entity_key = (cube_id, format!("{:?}", d1_key));
        self.nodes.write().unwrap().insert(node_id, node);
        self.entity_index.write().unwrap().insert(entity_key, node_id);
        node_id
    }

    pub fn get_node(&self, node_id: EdgeId) -> Option<HypergraphNode> {
        self.nodes.read().unwrap().get(&node_id).cloned()
    }

    pub fn node_for_entity(&self, cube_id: CubeId, d1_key: &DimKey) -> Option<EdgeId> {
        let key = (cube_id, format!("{:?}", d1_key));
        self.entity_index.read().unwrap().get(&key).copied()
    }

    // ── Edge management ───────────────────────────────────────────────────

    /// Add an edge. Enforces cycle detection for Dependency edges.
    pub fn add_edge(&self, edge: HypergraphEdge) -> Result<EdgeId, HypergridError> {
        if edge.edge_type.enforce_cycle_detection() {
            if self.would_create_cycle(edge.from_node, edge.to_node, &edge.edge_type)? {
                return Err(HypergridError::CyclicDependency {
                    from: edge.from_node, to: edge.to_node,
                });
            }
        }
        let edge_id = edge.edge_id;
        self.adj.write().unwrap().add_edge(&edge);
        self.edges.write().unwrap().insert(edge_id, edge);
        Ok(edge_id)
    }

    pub fn remove_edge(&self, edge_id: EdgeId) -> Result<(), HypergridError> {
        let edge = self.edges.write().unwrap().remove(&edge_id)
            .ok_or_else(|| HypergridError::NotFound(format!("edge: {edge_id}")))?;
        self.adj.write().unwrap().remove_edge(&edge);
        Ok(())
    }

    pub fn get_edge(&self, edge_id: EdgeId) -> Option<HypergraphEdge> {
        self.edges.read().unwrap().get(&edge_id).cloned()
    }

    pub fn update_consent(&self, edge_id: EdgeId, status: ConsentStatus, config: Option<ShadowCellConfig>) {
        if let Some(edge) = self.edges.write().unwrap().get_mut(&edge_id) {
            edge.consent_status = status;
            if let Some(cfg) = config { edge.shadow_config = Some(cfg); }
        }
    }

    pub fn edges_from(&self, node_id: EdgeId) -> Vec<HypergraphEdge> {
        let adj = self.adj.read().unwrap();
        let edge_ids = adj.outbound(node_id).to_vec();
        let edges = self.edges.read().unwrap();
        edge_ids.iter().filter_map(|id| edges.get(id).cloned()).collect()
    }

    pub fn edges_to(&self, node_id: EdgeId) -> Vec<HypergraphEdge> {
        let adj = self.adj.read().unwrap();
        let edge_ids = adj.inbound(node_id).to_vec();
        let edges = self.edges.read().unwrap();
        edge_ids.iter().filter_map(|id| edges.get(id).cloned()).collect()
    }

    pub fn edge_count(&self) -> usize { self.edges.read().unwrap().len() }
    pub fn node_count(&self) -> usize { self.nodes.read().unwrap().len() }

    // ── BFS Traversal ─────────────────────────────────────────────────────

    /// BFS from a start node up to config.max_depth hops.
    /// Returns all reachable nodes and traversed edges respecting the config.
    pub fn bfs(&self, start: EdgeId, config: &TraversalConfig) -> TraversalResult {
        let mut result = TraversalResult::default();
        let mut queue: VecDeque<(EdgeId, u8)> = VecDeque::new();
        queue.push_back((start, 0));

        let edges_guard = self.edges.read().unwrap();
        let adj_guard = self.adj.read().unwrap();

        while let Some((node_id, depth)) = queue.pop_front() {
            if depth > config.max_depth || result.node_set.contains(&node_id) { continue; }
            if depth >= config.min_depth {
                result.add_node(node_id, depth);
            }

            let neighbor_edges: Vec<EdgeId> = match config.direction {
                TraversalDirection::Outbound => adj_guard.outbound(node_id).to_vec(),
                TraversalDirection::Inbound  => adj_guard.inbound(node_id).to_vec(),
                TraversalDirection::Both => {
                    let mut v = adj_guard.outbound(node_id).to_vec();
                    v.extend_from_slice(adj_guard.inbound(node_id));
                    v
                }
            };

            for eid in neighbor_edges {
                if let Some(edge) = edges_guard.get(&eid) {
                    if !edge.is_active() { continue; }
                    if let Some(ref f) = config.edge_filter {
                        if !f.matches(&edge.edge_type) { continue; }
                    }
                    result.add_edge(eid);
                    let next = if edge.from_node == node_id { edge.to_node } else { edge.from_node };
                    if !result.node_set.contains(&next) {
                        queue.push_back((next, depth + 1));
                    }
                }
            }
        }
        result
    }

    /// Return all node IDs reachable from `start` via any outbound edge (unlimited depth).
    pub fn reachable_from(&self, start: EdgeId) -> HashSet<EdgeId> {
        let result = self.bfs(start, &TraversalConfig { max_depth: 64, ..Default::default() });
        result.node_set
    }

    /// Neighbors of a specific edge type and direction.
    pub fn neighbors_of_type(
        &self, node_id: EdgeId, edge_type: &EdgeType, direction: TraversalDirection,
    ) -> Vec<EdgeId> {
        let config = TraversalConfig {
            max_depth: 1,
            min_depth: 1,
            edge_filter: Some(EdgeTypeFilter::Only(vec![edge_type.clone()])),
            direction,
        };
        self.bfs(node_id, &config).nodes.into_iter().map(|(id, _)| id).collect()
    }

    // ── Cycle Detection ───────────────────────────────────────────────────

    /// Would adding an edge from `from` to `to` (of the given type) create a cycle?
    /// Uses DFS starting from `to` looking for `from`.
    pub fn would_create_cycle(
        &self, from: EdgeId, to: EdgeId, edge_type: &EdgeType,
    ) -> Result<bool, HypergridError> {
        // A cycle exists if we can already reach `from` starting from `to`
        let reachable = self.bfs(to, &TraversalConfig {
            max_depth: 64,
            edge_filter: Some(EdgeTypeFilter::Only(vec![edge_type.clone()])),
            direction: TraversalDirection::Outbound,
            ..Default::default()
        });
        Ok(reachable.contains_node(from))
    }

    // ── Shortest Path (Dijkstra) ──────────────────────────────────────────

    /// Find the shortest weighted path between two nodes.
    pub fn shortest_path(&self, from: EdgeId, to: EdgeId) -> Option<Vec<EdgeId>> {
        use std::collections::BinaryHeap;
        use std::cmp::Reverse;

        let mut dist: HashMap<EdgeId, ordered_float::OrderedFloat<f64>> = HashMap::new();
        let mut prev: HashMap<EdgeId, EdgeId> = HashMap::new();
        let mut heap = BinaryHeap::new();

        dist.insert(from, ordered_float::OrderedFloat(0.0));
        heap.push(Reverse((ordered_float::OrderedFloat(0.0f64), from)));

        let edges_guard = self.edges.read().unwrap();
        let adj_guard   = self.adj.read().unwrap();

        while let Some(Reverse((cost, node))) = heap.pop() {
            if node == to {
                // Reconstruct path
                let mut path = vec![to];
                let mut curr = to;
                while let Some(&p) = prev.get(&curr) {
                    path.push(p);
                    curr = p;
                }
                path.reverse();
                return Some(path);
            }
            if &cost > dist.get(&node).unwrap_or(&ordered_float::OrderedFloat(f64::INFINITY)) {
                continue;
            }
            for &eid in adj_guard.outbound(node) {
                if let Some(edge) = edges_guard.get(&eid) {
                    if !edge.is_active() { continue; }
                    let next = edge.to_node;
                    let new_cost = ordered_float::OrderedFloat(cost.0 + (1.0 - edge.weight));
                    if &new_cost < dist.get(&next).unwrap_or(&ordered_float::OrderedFloat(f64::INFINITY)) {
                        dist.insert(next, new_cost);
                        prev.insert(next, node);
                        heap.push(Reverse((new_cost, next)));
                    }
                }
            }
        }
        None
    }

    // ── Shadow Cell helpers ───────────────────────────────────────────────

    /// Return all active CrossGridLink edges that have been accepted (for sync).
    pub fn active_cross_grid_links(&self) -> Vec<HypergraphEdge> {
        self.edges.read().unwrap().values()
            .filter(|e| {
                matches!(e.edge_type, EdgeType::CrossGridLink) &&
                matches!(e.consent_status, ConsentStatus::Accepted)
            })
            .cloned()
            .collect()
    }

    /// Return all edges connecting to a specific target grid.
    pub fn links_to_grid(&self, target_grid: GridId) -> Vec<HypergraphEdge> {
        let nodes = self.nodes.read().unwrap();
        let edges = self.edges.read().unwrap();
        edges.values()
            .filter(|e| {
                nodes.get(&e.to_node)
                    .map(|n| n.grid_id == target_grid)
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }
}
