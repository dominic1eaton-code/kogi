//! HG-GRAPH — Hypergraph: typed edges, shadow cells, link forests.
//!
//! Every Hypergrid deployment includes a built-in graph layer that models
//! relationships between any HyperCells, HyperRows, Hypercubes, and Grids as a
//! typed, directed, weighted property graph.

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::cell::{AttributeMap, DimCoordinate, DimKey, GridId, CubeId, IdentityId, SpaceId};
use crate::error::{HypergridError, HypergridResult};

pub type NodeId = Uuid;
pub type EdgeId = Uuid;
pub type GraphId = Uuid;
pub type ShadowCellId = Uuid;

// ─── Node Types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeType {
    Cell,
    Row,
    Cube,
    Space,
    Grid,
    External,
    Custom(String),
}

/// Reference to the entity this graph node represents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityRef {
    Cell { cube_id: CubeId, coord: DimCoordinate },
    Row { cube_id: CubeId, d1_key: DimKey },
    Cube { cube_id: CubeId },
    Space { space_id: SpaceId },
    Grid { grid_id: GridId },
    External { url: String },
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeVisibility {
    Public, Tenant, Private,
}

/// A node in the Hypergraph — represents any Hypergrid entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypergraphNode {
    pub node_id: NodeId,
    pub node_type: NodeType,
    pub entity_ref: EntityRef,
    /// Node-level N-attributes (reuses HG-CELL attribute model).
    pub attributes: AttributeMap,
    pub namespace_path: Option<String>,
    pub visibility: NodeVisibility,
    pub identity_tag: Option<IdentityId>,
    pub created_at: DateTime<Utc>,
}

impl HypergraphNode {
    pub fn new(node_type: NodeType, entity_ref: EntityRef) -> Self {
        Self {
            node_id: Uuid::new_v4(),
            node_type,
            entity_ref,
            attributes: AttributeMap::new(),
            namespace_path: None,
            visibility: NodeVisibility::Public,
            identity_tag: None,
            created_at: Utc::now(),
        }
    }
}

// ─── Edge Types ───────────────────────────────────────────────────────────────

/// The semantic type of a HypergraphEdge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EdgeType {
    /// Parent-child containment. Used for drill-down and rollup.
    Hierarchy,
    /// One entity depends on another (blocking relationship). Cross-grid supported.
    Dependency,
    /// Soft lateral association. Non-blocking reference.
    Association,
    /// Ownership containment: this Space/Cube contains this entity.
    Contains,
    /// One row is derived from or is a version/fork of another.
    Derives,
    /// Inter-grid connection: the core link network edge. Creates ShadowCells.
    CrossGridLink,
    /// Marks a row as a shadow of a row in another grid.
    ShadowOf,
    /// A tenant/identity is a member of a Space.
    SpaceMembership,
    /// Bidirectional collaboration on a shared entity.
    Collaborates,
    /// Citation or reference. Lighter than Dependency.
    References,
    /// Two Grids are federated for CRDT sync.
    FederationPeer,
    /// One namespace path is an alias for another.
    NamespaceAlias,
    /// This cell's attribute value was computed from the target cell(s).
    ComputedFrom,
    /// Identity receives change notifications for this entity.
    SubscribesTo,
    /// Economic investment relationship.
    InvestedIn,
    /// Resource sharing between portfolio components.
    ResourceShare,
    /// Platform-defined edge type.
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EdgeDirection {
    Directed,
    Bidirectional,
}

/// Consent state for cross-identity edges (CrossGridLink, Collaborates).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConsentStatus {
    Pending, Accepted, Declined, Revoked,
}

/// A directed, typed, weighted edge in the Hypergraph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypergraphEdge {
    pub edge_id: EdgeId,
    pub from_node: NodeId,
    pub to_node: NodeId,
    pub edge_type: EdgeType,
    pub direction: EdgeDirection,
    /// Edge weight 0.0–1.0.
    pub weight: f64,
    /// Edge-level N-attributes.
    pub attributes: AttributeMap,
    pub visibility: NodeVisibility,
    pub consent_status: ConsentStatus,
    pub space_scope: Option<SpaceId>,
    /// Shadow cell created by this edge (for CrossGridLink edges).
    pub shadow_cell_id: Option<ShadowCellId>,
    /// Time-limited edges expire after this timestamp.
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl HypergraphEdge {
    pub fn new(from_node: NodeId, to_node: NodeId, edge_type: EdgeType) -> Self {
        Self {
            edge_id: Uuid::new_v4(),
            from_node,
            to_node,
            edge_type,
            direction: EdgeDirection::Directed,
            weight: 1.0,
            attributes: AttributeMap::new(),
            visibility: NodeVisibility::Public,
            consent_status: ConsentStatus::Accepted,
            space_scope: None,
            shadow_cell_id: None,
            expires_at: None,
            created_at: Utc::now(),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|t| t < Utc::now()).unwrap_or(false)
    }
}

// ─── Shadow Cell ──────────────────────────────────────────────────────────────

/// A ShadowCell is a read-only reflection in Grid A of a linked entity from Grid B.
/// Created automatically when a CrossGridLink edge is established.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowCell {
    pub shadow_id: ShadowCellId,
    /// The CrossGridLink edge that created this shadow.
    pub source_edge_id: EdgeId,
    /// Grid and coordinate in the host grid (where the shadow lives).
    pub host_grid_id: GridId,
    pub host_cube_id: CubeId,
    pub host_coord: DimCoordinate,
    /// Grid and coordinate in the source grid (where the original lives).
    pub source_grid_id: GridId,
    pub source_cube_id: CubeId,
    pub source_coord: DimCoordinate,
    /// Which attribute keys are mirrored from the source cell.
    pub mirrored_attrs: Vec<String>,
    /// Which attribute keys the source can write back to.
    pub writeback_attrs: Vec<String>,
    /// Current synced values (read-only snapshot).
    pub synced_values: AttributeMap,
    pub last_synced_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// ─── Link Forest ─────────────────────────────────────────────────────────────

/// A link tree rooted at one entity — all its cross-grid connections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkTree {
    pub root_node_id: NodeId,
    pub identity_id: IdentityId,
    /// Direct outbound CrossGridLink edges from this root.
    pub links: Vec<EdgeId>,
    /// Sub-trees for transitively linked entities.
    pub children: Vec<LinkTree>,
}

/// The complete set of all link trees rooted at a given identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkForest {
    pub identity_id: IdentityId,
    pub trees: Vec<LinkTree>,
    pub computed_at: DateTime<Utc>,
}

// ─── Subgraph ─────────────────────────────────────────────────────────────────

/// A Space-scoped sub-graph within the full Hypergraph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subgraph {
    pub subgraph_id: Uuid,
    pub space_id: SpaceId,
    pub node_ids: Vec<NodeId>,
    pub edge_ids: Vec<EdgeId>,
}

// ─── NodeStore & EdgeStore ────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct NodeStore {
    nodes: HashMap<NodeId, HypergraphNode>,
}

impl NodeStore {
    pub fn insert(&mut self, node: HypergraphNode) {
        self.nodes.insert(node.node_id, node);
    }
    pub fn get(&self, id: NodeId) -> Option<&HypergraphNode> {
        self.nodes.get(&id)
    }
    pub fn remove(&mut self, id: NodeId) -> Option<HypergraphNode> {
        self.nodes.remove(&id)
    }
    pub fn len(&self) -> usize { self.nodes.len() }
}

#[derive(Debug, Clone, Default)]
pub struct EdgeStore {
    edges: HashMap<EdgeId, HypergraphEdge>,
    /// Adjacency index: from_node → [edge_ids]
    from_index: HashMap<NodeId, Vec<EdgeId>>,
    /// Adjacency index: to_node → [edge_ids]
    to_index: HashMap<NodeId, Vec<EdgeId>>,
}

impl EdgeStore {
    pub fn insert(&mut self, edge: HypergraphEdge) {
        let eid = edge.edge_id;
        self.from_index.entry(edge.from_node).or_default().push(eid);
        self.to_index.entry(edge.to_node).or_default().push(eid);
        self.edges.insert(eid, edge);
    }

    pub fn get(&self, id: EdgeId) -> Option<&HypergraphEdge> {
        self.edges.get(&id)
    }

    pub fn remove(&mut self, id: EdgeId) -> Option<HypergraphEdge> {
        if let Some(e) = self.edges.remove(&id) {
            self.from_index.entry(e.from_node).or_default().retain(|&eid| eid != id);
            self.to_index.entry(e.to_node).or_default().retain(|&eid| eid != id);
            Some(e)
        } else {
            None
        }
    }

    pub fn edges_from(&self, node_id: NodeId) -> Vec<&HypergraphEdge> {
        self.from_index.get(&node_id)
            .map(|ids| ids.iter().filter_map(|id| self.edges.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn edges_to(&self, node_id: NodeId) -> Vec<&HypergraphEdge> {
        self.to_index.get(&node_id)
            .map(|ids| ids.iter().filter_map(|id| self.edges.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn edges_of_type(&self, node_id: NodeId, edge_type: &EdgeType) -> Vec<&HypergraphEdge> {
        self.edges_from(node_id)
            .into_iter()
            .filter(|e| &e.edge_type == edge_type)
            .collect()
    }

    pub fn len(&self) -> usize { self.edges.len() }
}

// ─── Hypergraph ───────────────────────────────────────────────────────────────

/// The graph layer of a Grid. Models relationships between any entities as a
/// typed, directed, weighted property graph.
#[derive(Debug, Default)]
pub struct Hypergraph {
    pub graph_id: GraphId,
    pub grid_id: GridId,
    pub nodes: NodeStore,
    pub edges: EdgeStore,
    pub subgraphs: Vec<Subgraph>,
    pub link_forests: HashMap<IdentityId, LinkForest>,
    /// ShadowCells indexed by shadow_id.
    shadow_cells: HashMap<ShadowCellId, ShadowCell>,
}

impl Hypergraph {
    pub fn new(grid_id: GridId) -> Self {
        Self {
            graph_id: Uuid::new_v4(),
            grid_id,
            ..Default::default()
        }
    }

    // ── Node management ───────────────────────────────────────────────────

    pub fn add_node(&mut self, node: HypergraphNode) -> NodeId {
        let id = node.node_id;
        self.nodes.insert(node);
        id
    }

    pub fn get_node(&self, id: NodeId) -> HypergridResult<&HypergraphNode> {
        self.nodes.get(id).ok_or_else(|| HypergridError::GraphError(format!("node {id} not found")))
    }

    // ── Edge management ───────────────────────────────────────────────────

    /// Add a typed directed edge. Returns the new EdgeId.
    pub fn add_edge(&mut self, edge: HypergraphEdge) -> HypergridResult<EdgeId> {
        // Validate nodes exist
        if self.nodes.get(edge.from_node).is_none() {
            return Err(HypergridError::GraphError(format!("from_node {} not found", edge.from_node)));
        }
        if self.nodes.get(edge.to_node).is_none() {
            return Err(HypergridError::GraphError(format!("to_node {} not found", edge.to_node)));
        }
        let id = edge.edge_id;
        self.edges.insert(edge);
        Ok(id)
    }

    pub fn remove_edge(&mut self, edge_id: EdgeId) -> HypergridResult<HypergraphEdge> {
        self.edges.remove(edge_id).ok_or_else(|| HypergridError::GraphError(format!("edge {edge_id} not found")))
    }

    /// Connect two nodes with a Dependency edge. Checks for cycles.
    pub fn add_dependency(&mut self, from: NodeId, to: NodeId) -> HypergridResult<EdgeId> {
        // Shallow cycle check: reject if `to` already depends on `from`
        let to_deps: Vec<NodeId> = self.edges.edges_of_type(to, &EdgeType::Dependency)
            .iter().map(|e| e.to_node).collect();
        if to_deps.contains(&from) {
            return Err(HypergridError::CyclicDependency(from, to));
        }
        let edge = HypergraphEdge::new(from, to, EdgeType::Dependency);
        self.add_edge(edge)
    }

    /// Establish a CrossGridLink, provisioning a ShadowCell.
    pub fn create_cross_grid_link(
        &mut self,
        from: NodeId,
        to: NodeId,
        host_coord: DimCoordinate,
        source_coord: DimCoordinate,
        mirrored_attrs: Vec<String>,
    ) -> HypergridResult<(EdgeId, ShadowCellId)> {
        let mut edge = HypergraphEdge::new(from, to, EdgeType::CrossGridLink);
        edge.consent_status = ConsentStatus::Pending;
        let shadow_id = Uuid::new_v4();
        edge.shadow_cell_id = Some(shadow_id);

        let host_node = self.get_node(from)?;
        let host_grid_id = self.grid_id;
        let (host_cube_id, h_coord) = match &host_node.entity_ref {
            EntityRef::Row { cube_id, .. } => (*cube_id, host_coord),
            EntityRef::Cell { cube_id, coord } => (*cube_id, coord.clone()),
            _ => return Err(HypergridError::GraphError("CrossGridLink requires Row or Cell node".into())),
        };

        let source_node = self.get_node(to)?;
        let (source_grid_id, source_cube_id, s_coord) = match &source_node.entity_ref {
            EntityRef::Row { cube_id, .. } => (self.grid_id, *cube_id, source_coord),
            EntityRef::Cell { cube_id, coord } => (self.grid_id, *cube_id, coord.clone()),
            EntityRef::Grid { grid_id } => (*grid_id, Uuid::nil(), source_coord),
            _ => return Err(HypergridError::GraphError("CrossGridLink target must be Row, Cell, or Grid".into())),
        };

        let shadow = ShadowCell {
            shadow_id,
            source_edge_id: edge.edge_id,
            host_grid_id,
            host_cube_id,
            host_coord: h_coord,
            source_grid_id,
            source_cube_id,
            source_coord: s_coord,
            mirrored_attrs,
            writeback_attrs: vec![],
            synced_values: AttributeMap::new(),
            last_synced_at: Utc::now(),
            created_at: Utc::now(),
        };

        let edge_id = self.add_edge(edge)?;
        self.shadow_cells.insert(shadow_id, shadow);
        Ok((edge_id, shadow_id))
    }

    // ── Traversal ─────────────────────────────────────────────────────────

    /// Return all neighbors of a node via outbound edges.
    pub fn neighbors(&self, node_id: NodeId) -> Vec<NodeId> {
        self.edges.edges_from(node_id).iter().map(|e| e.to_node).collect()
    }

    /// BFS reachability from a source node.
    pub fn reachable_from(&self, source: NodeId) -> Vec<NodeId> {
        let mut visited = vec![source];
        let mut queue = std::collections::VecDeque::from([source]);
        while let Some(current) = queue.pop_front() {
            for neighbor in self.neighbors(current) {
                if !visited.contains(&neighbor) {
                    visited.push(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }
        visited
    }

    /// Depth-first cycle detection (for dependency graphs).
    pub fn detect_cycle(&self) -> Option<Vec<NodeId>> {
        let mut visited = std::collections::HashSet::new();
        let mut path = vec![];

        fn dfs(
            graph: &Hypergraph,
            node: NodeId,
            visited: &mut std::collections::HashSet<NodeId>,
            path: &mut Vec<NodeId>,
        ) -> Option<Vec<NodeId>> {
            if path.contains(&node) {
                let cycle_start = path.iter().position(|&n| n == node).unwrap();
                return Some(path[cycle_start..].to_vec());
            }
            if visited.contains(&node) { return None; }
            path.push(node);
            for neighbor in graph.neighbors(node) {
                if let Some(cycle) = dfs(graph, neighbor, visited, path) {
                    return Some(cycle);
                }
            }
            path.pop();
            visited.insert(node);
            None
        }

        // Collect all node IDs before iterating to avoid borrow issues
        let all_nodes: Vec<NodeId> = self.nodes.nodes.keys().copied().collect();
        for node_id in all_nodes {
            if !visited.contains(&node_id) {
                if let Some(cycle) = dfs(self, node_id, &mut visited, &mut path) {
                    return Some(cycle);
                }
            }
        }
        None
    }

    // ── Shadow Cell Access ────────────────────────────────────────────────

    pub fn get_shadow_cell(&self, id: ShadowCellId) -> Option<&ShadowCell> {
        self.shadow_cells.get(&id)
    }

    pub fn shadow_cells_in_cube(&self, cube_id: crate::cell::CubeId) -> Vec<&ShadowCell> {
        self.shadow_cells.values().filter(|s| s.host_cube_id == cube_id).collect()
    }

    pub fn stats(&self) -> (usize, usize, usize) {
        (self.nodes.len(), self.edges.len(), self.shadow_cells.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_nodes_and_edge() {
        let grid_id = Uuid::new_v4();
        let mut g = Hypergraph::new(grid_id);
        let n1 = g.add_node(HypergraphNode::new(NodeType::Row, EntityRef::Cube { cube_id: Uuid::new_v4() }));
        let n2 = g.add_node(HypergraphNode::new(NodeType::Row, EntityRef::Cube { cube_id: Uuid::new_v4() }));
        let eid = g.add_edge(HypergraphEdge::new(n1, n2, EdgeType::Association)).unwrap();
        assert!(g.edges.get(eid).is_some());
    }

    #[test]
    fn cycle_detection() {
        let grid_id = Uuid::new_v4();
        let mut g = Hypergraph::new(grid_id);
        let n1 = g.add_node(HypergraphNode::new(NodeType::Row, EntityRef::Cube { cube_id: Uuid::new_v4() }));
        let n2 = g.add_node(HypergraphNode::new(NodeType::Row, EntityRef::Cube { cube_id: Uuid::new_v4() }));
        g.add_edge(HypergraphEdge::new(n1, n2, EdgeType::Dependency)).unwrap();
        // Shallow cycle check: adding n2→n1 after n1→n2 should be detected
        let result = g.add_dependency(n2, n1);
        assert!(result.is_err()); // cyclic dependency detected
    }
}
