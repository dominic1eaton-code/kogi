//! Portfolio graph layer — thin domain wrapper over hypergrid::graph::Hypergraph.
//!
//! Portfolio edge types map onto Hypergrid EdgeType variants. All structural
//! graph operations (add/remove edges, cycle detection, traversal) delegate
//! to the hypergrid Hypergraph. This module provides:
//!   - PortfolioEdge: the portfolio-domain edge type (serialisable, CRDT-friendly)
//!   - PortfolioGraph: owns a hypergrid Hypergraph + a NodeId registry
//!   - Lightweight structural primitives (Group, Collection, List, Schedule, Directory)

use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use hypergrid::graph::{
    EdgeType as HgEdgeType,
    EntityRef, Hypergraph, HypergraphEdge, HypergraphNode, NodeType,
};
use hypergrid::cell::{CubeId, GridId};


use crate::types::{ComponentId, EntityId, PolicyId, ResourceAccessLevel};

// ── Portfolio Edge Type ───────────────────────────────────────────────────────

/// Portfolio-domain edge semantics, mapped to Hypergrid EdgeType.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortfolioEdgeType {
    /// Parent → Child: compositional containment (Portfolio → Program → Project).
    Hierarchy,
    /// A depends on B: causal/blocking; cycle detection enforced.
    Dependency,
    /// A ↔ B: soft informational cross-reference.
    Link,
    /// Container → Item: Container membership (Binder holds Items).
    Contains,
    /// Cross-portfolio federation link.
    Federation,
    /// Resource sharing between a component and another entity.
    ResourceShare,
    /// Platform-defined custom edge type.
    Custom(String),
}

impl From<&PortfolioEdgeType> for HgEdgeType {
    fn from(p: &PortfolioEdgeType) -> Self {
        match p {
            PortfolioEdgeType::Hierarchy    => HgEdgeType::Hierarchy,
            PortfolioEdgeType::Dependency   => HgEdgeType::Dependency,
            PortfolioEdgeType::Link         => HgEdgeType::Association,
            PortfolioEdgeType::Contains     => HgEdgeType::Contains,
            PortfolioEdgeType::Federation   => HgEdgeType::FederationPeer,
            PortfolioEdgeType::ResourceShare => HgEdgeType::ResourceShare,
            PortfolioEdgeType::Custom(s)    => HgEdgeType::Custom(s.clone()),
        }
    }
}

// ── PortfolioEdge (serialisable, used in CRDT ops) ──────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioEdge {
    pub edge_id:    Uuid,
    pub edge_type:  PortfolioEdgeType,
    pub source_id:  ComponentId,
    pub target_id:  ComponentId,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub metadata:   serde_json::Value,
}

impl PortfolioEdge {
    pub fn new(edge_type: PortfolioEdgeType, source_id: ComponentId, target_id: ComponentId, by: &str) -> Self {
        Self {
            edge_id: Uuid::new_v4(),
            edge_type,
            source_id,
            target_id,
            created_at: Utc::now(),
            created_by: by.to_owned(),
            metadata: serde_json::Value::Null,
        }
    }

    pub fn hierarchy(parent: ComponentId, child: ComponentId, by: &str) -> Self {
        Self::new(PortfolioEdgeType::Hierarchy, parent, child, by)
    }

    pub fn dependency(from: ComponentId, to: ComponentId, by: &str) -> Self {
        Self::new(PortfolioEdgeType::Dependency, from, to, by)
    }

    pub fn link(a: ComponentId, b: ComponentId, by: &str) -> Self {
        Self::new(PortfolioEdgeType::Link, a, b, by)
    }

    pub fn resource_share(
        resource_id: ComponentId,
        target_entity: EntityId,
        access_level: ResourceAccessLevel,
        policy_id: Option<PolicyId>,
        expiry: Option<DateTime<Utc>>,
        attribution_required: bool,
        by: &str,
    ) -> Self {
        let mut e = Self::new(PortfolioEdgeType::ResourceShare, resource_id, target_entity, by);
        e.metadata = serde_json::json!({
            "share_policy_id": policy_id,
            "access_level": access_level,
            "expiry": expiry,
            "attribution_required": attribution_required,
        });
        e
    }

    /// Build the corresponding HypergraphEdge for insertion into the Hypergraph.
    pub fn to_hg_edge(&self, from_node: hypergrid::graph::NodeId, to_node: hypergrid::graph::NodeId) -> HypergraphEdge {
        let mut edge = HypergraphEdge::new(from_node, to_node, HgEdgeType::from(&self.edge_type));
        edge.edge_id = self.edge_id;
        edge
    }
}

// ── PortfolioGraph ────────────────────────────────────────────────────────────

/// Wraps hypergrid's Hypergraph with a portfolio-domain node registry.
/// Each Component gets one graph node; edges are portfolio-typed.
pub struct PortfolioGraph {
    /// The underlying hypergrid graph layer.
    pub hg: Hypergraph,
    /// Maps ComponentId → Hypergraph NodeId.
    component_nodes: HashMap<ComponentId, hypergrid::graph::NodeId>,
    /// Flat portfolio edge list (source of truth for CRDT sync + cycle detection).
    edges: Vec<PortfolioEdge>,
    grid_id: GridId,
    cube_id: CubeId,
}

impl PortfolioGraph {
    pub fn new(grid_id: GridId, cube_id: CubeId) -> Self {
        Self {
            hg: Hypergraph::new(grid_id),
            component_nodes: HashMap::new(),
            edges: vec![],
            grid_id,
            cube_id,
        }
    }

    // ── Node management ───────────────────────────────────────────────────

    /// Register a new component as a graph node. Idempotent.
    pub fn register_component(&mut self, component_id: ComponentId) -> hypergrid::graph::NodeId {
        if let Some(&existing) = self.component_nodes.get(&component_id) {
            return existing;
        }
        use hypergrid::cell::DimKey;
        let node = HypergraphNode::new(
            NodeType::Row,
            EntityRef::Row { cube_id: self.cube_id, d1_key: DimKey::uuid(component_id) },
        );
        let node_id = self.hg.add_node(node);
        self.component_nodes.insert(component_id, node_id);
        node_id
    }

    pub fn node_of(&self, component_id: ComponentId) -> Option<hypergrid::graph::NodeId> {
        self.component_nodes.get(&component_id).copied()
    }

    pub fn deregister_component(&mut self, component_id: ComponentId) {
        if let Some(_node_id) = self.component_nodes.remove(&component_id) {
            // Remove all edges incident to this node
            self.edges.retain(|e| e.source_id != component_id && e.target_id != component_id);
        }
    }

    // ── Edge management ───────────────────────────────────────────────────

    /// Add a portfolio edge. Registers nodes if not already present.
    /// Returns `Err` if a cyclic dependency would result.
    pub fn add_edge(&mut self, edge: PortfolioEdge) -> Result<(), String> {
        // Cycle check for Dependency edges
        if edge.edge_type == PortfolioEdgeType::Dependency {
            let potential: Vec<PortfolioEdge> = {
                let mut all = self.edges.clone();
                all.push(edge.clone());
                all
            };
            if let Some(cycle) = detect_cycle(&potential) {
                return Err(format!(
                    "Adding dependency {:?}→{:?} would create a cycle: {:?}",
                    edge.source_id, edge.target_id, cycle
                ));
            }
        }

        // Ensure both nodes exist in the Hypergraph
        let from_node = self.register_component(edge.source_id);
        let to_node   = self.register_component(edge.target_id);

        let hg_edge = edge.to_hg_edge(from_node, to_node);
        let _ = self.hg.add_edge(hg_edge); // Hypergraph validates node existence
        self.edges.push(edge);
        Ok(())
    }

    pub fn remove_edge(&mut self, edge_id: Uuid) {
        self.edges.retain(|e| e.edge_id != edge_id);
        let _ = self.hg.remove_edge(edge_id);
    }

    /// All edges for a given component (inbound + outbound).
    pub fn edges_for(&self, id: ComponentId) -> Vec<&PortfolioEdge> {
        self.edges.iter().filter(|e| e.source_id == id || e.target_id == id).collect()
    }

    /// Outbound edges by type.
    pub fn edges_of_type(&self, id: ComponentId, t: &PortfolioEdgeType) -> Vec<&PortfolioEdge> {
        self.edges.iter().filter(|e| e.source_id == id && &e.edge_type == t).collect()
    }

    /// Direct children (Hierarchy outbound).
    pub fn children_of(&self, id: ComponentId) -> Vec<ComponentId> {
        self.edges_of_type(id, &PortfolioEdgeType::Hierarchy)
            .iter().map(|e| e.target_id).collect()
    }

    /// Direct parents (Hierarchy inbound).
    pub fn parents_of(&self, id: ComponentId) -> Vec<ComponentId> {
        self.edges.iter()
            .filter(|e| e.target_id == id && e.edge_type == PortfolioEdgeType::Hierarchy)
            .map(|e| e.source_id)
            .collect()
    }

    /// Dependencies (Dependency outbound).
    pub fn dependencies_of(&self, id: ComponentId) -> Vec<ComponentId> {
        self.edges_of_type(id, &PortfolioEdgeType::Dependency)
            .iter().map(|e| e.target_id).collect()
    }

    /// Dependents (Dependency inbound).
    pub fn dependents_of(&self, id: ComponentId) -> Vec<ComponentId> {
        self.edges.iter()
            .filter(|e| e.target_id == id && e.edge_type == PortfolioEdgeType::Dependency)
            .map(|e| e.source_id)
            .collect()
    }

    // ── Traversal (delegates to Hypergraph) ───────────────────────────────

    pub fn reachable_from(&self, id: ComponentId) -> Vec<ComponentId> {
        let Some(node_id) = self.node_of(id) else { return vec![]; };
        self.hg.reachable_from(node_id)
            .iter()
            .filter_map(|&nid| {
                self.component_nodes.iter()
                    .find(|(_, &v)| v == nid)
                    .map(|(&cid, _)| cid)
            })
            .filter(|&cid| cid != id)
            .collect()
    }

    pub fn detect_cycle(&self) -> Option<Vec<ComponentId>> {
        detect_cycle(&self.edges)
    }

    pub fn edges_snapshot(&self) -> &[PortfolioEdge] {
        &self.edges
    }

    pub fn edge_count(&self) -> usize { self.edges.len() }
    pub fn node_count(&self) -> usize { self.component_nodes.len() }
}

// ── Standalone cycle detection ────────────────────────────────────────────────

/// DFS cycle detection on the Dependency sub-graph.
/// Returns Some(cycle_path) if a cycle is detected, None otherwise.
pub fn detect_cycle(edges: &[PortfolioEdge]) -> Option<Vec<ComponentId>> {
    let mut adj: HashMap<ComponentId, Vec<ComponentId>> = HashMap::new();
    for e in edges {
        if e.edge_type == PortfolioEdgeType::Dependency {
            adj.entry(e.source_id).or_default().push(e.target_id);
        }
    }

    let mut visited: HashSet<ComponentId> = HashSet::new();
    let mut rec_stack: Vec<ComponentId> = vec![];

    fn dfs(
        node: ComponentId,
        adj: &HashMap<ComponentId, Vec<ComponentId>>,
        visited: &mut HashSet<ComponentId>,
        rec_stack: &mut Vec<ComponentId>,
    ) -> Option<Vec<ComponentId>> {
        if rec_stack.contains(&node) {
            let start = rec_stack.iter().position(|&n| n == node).unwrap();
            return Some(rec_stack[start..].to_vec());
        }
        if visited.contains(&node) { return None; }
        rec_stack.push(node);
        if let Some(neighbors) = adj.get(&node) {
            for &nb in neighbors {
                if let Some(cycle) = dfs(nb, adj, visited, rec_stack) {
                    return Some(cycle);
                }
            }
        }
        rec_stack.pop();
        visited.insert(node);
        None
    }

    let all_nodes: Vec<ComponentId> = adj.keys().copied().collect();
    for node in all_nodes {
        if !visited.contains(&node) {
            if let Some(cycle) = dfs(node, &adj, &mut visited, &mut rec_stack) {
                return Some(cycle);
            }
        }
    }
    None
}

// ── Lightweight structural primitives ────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Group {
    pub group_id: Uuid,
    pub name: String,
    pub members: HashSet<ComponentId>,
}
impl Group {
    pub fn new(name: impl Into<String>) -> Self {
        Self { group_id: Uuid::new_v4(), name: name.into(), members: HashSet::new() }
    }
    pub fn add(&mut self, id: ComponentId) { self.members.insert(id); }
    pub fn remove(&mut self, id: &ComponentId) { self.members.remove(id); }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Collection {
    pub collection_id: Uuid,
    pub name: String,
    pub members: HashSet<ComponentId>,
}
impl Collection {
    pub fn new(name: impl Into<String>) -> Self {
        Self { collection_id: Uuid::new_v4(), name: name.into(), members: HashSet::new() }
    }
    pub fn add(&mut self, id: ComponentId) { self.members.insert(id); }
    pub fn remove(&mut self, id: &ComponentId) { self.members.remove(id); }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct List {
    pub list_id: Uuid,
    pub name: String,
    pub items: Vec<ComponentId>,
}
impl List {
    pub fn new(name: impl Into<String>) -> Self {
        Self { list_id: Uuid::new_v4(), name: name.into(), items: vec![] }
    }
    pub fn append(&mut self, id: ComponentId) { self.items.push(id); }
    pub fn insert(&mut self, index: usize, id: ComponentId) {
        let i = index.min(self.items.len()); self.items.insert(i, id);
    }
    pub fn remove(&mut self, id: &ComponentId) { self.items.retain(|i| i != id); }
    pub fn move_item(&mut self, id: &ComponentId, to: usize) {
        if let Some(pos) = self.items.iter().position(|i| i == id) {
            let item = self.items.remove(pos);
            let to = to.min(self.items.len());
            self.items.insert(to, item);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleEntry {
    pub entry_id: Uuid,
    pub component_id: ComponentId,
    pub event_at: DateTime<Utc>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Schedule {
    pub schedule_id: Uuid,
    pub name: String,
    pub entries: Vec<ScheduleEntry>,
}
impl Schedule {
    pub fn new(name: impl Into<String>) -> Self {
        Self { schedule_id: Uuid::new_v4(), name: name.into(), entries: vec![] }
    }
    pub fn add(&mut self, component_id: ComponentId, event_at: DateTime<Utc>, label: Option<String>) {
        self.entries.push(ScheduleEntry { entry_id: Uuid::new_v4(), component_id, event_at, label });
        self.entries.sort_by_key(|e| e.event_at);
    }
    pub fn upcoming(&self, now: DateTime<Utc>) -> Vec<&ScheduleEntry> {
        self.entries.iter().filter(|e| e.event_at >= now).collect()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Directory {
    pub directory_id: Uuid,
    pub name: String,
    pub paths: HashMap<String, Vec<ComponentId>>,
}
impl Directory {
    pub fn new(name: impl Into<String>) -> Self {
        Self { directory_id: Uuid::new_v4(), name: name.into(), paths: HashMap::new() }
    }
    pub fn insert(&mut self, path: impl Into<String>, id: ComponentId) {
        self.paths.entry(path.into()).or_default().push(id);
    }
    pub fn remove(&mut self, path: &str, id: &ComponentId) {
        if let Some(ids) = self.paths.get_mut(path) { ids.retain(|i| i != id); }
    }
    pub fn list(&self, path: &str) -> Vec<ComponentId> {
        self.paths.get(path).cloned().unwrap_or_default()
    }
    pub fn list_prefix(&self, prefix: &str) -> Vec<(&str, &Vec<ComponentId>)> {
        self.paths.iter().filter(|(p, _)| p.starts_with(prefix)).map(|(p, v)| (p.as_str(), v)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_detection_simple() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let mut edges = vec![PortfolioEdge::dependency(a, b, "n1")];
        assert!(detect_cycle(&edges).is_none());
        edges.push(PortfolioEdge::dependency(b, a, "n1"));
        assert!(detect_cycle(&edges).is_some());
    }

    #[test]
    fn list_move() {
        let ids: Vec<Uuid> = (0..4).map(|_| Uuid::new_v4()).collect();
        let mut list = List::new("t");
        ids.iter().for_each(|&id| list.append(id));
        list.move_item(&ids[0], 3);
        assert_eq!(list.items.last(), Some(&ids[0]));
    }
}
