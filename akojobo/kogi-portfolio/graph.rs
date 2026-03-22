//! GraphEdge and structural primitives (Group, Collection, List, Schedule, Directory).

use std::collections::{HashMap, HashSet, VecDeque};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::types::{ComponentId, EntityId, PolicyId, ResourceAccessLevel};

// ── GraphEdgeType ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphEdgeType {
    /// Parent → Child: compositional containment (Portfolio → Program → Project).
    Hierarchy,
    /// A depends on B: causal/blocking relationship; cycle detection enforced.
    Dependency,
    /// A ↔ B: soft association — informational cross-reference.
    Link,
    /// Container → Item: Container membership (Binder holds Items).
    Contains,
    /// Cross-portfolio link (PortfolioFederation.federate()).
    Federation,
    /// Resource sharing between a component and an entity (Space | org | user).
    ResourceShare,
    /// Platform-defined edge type.
    Custom(String),
}

// ── GraphEdge ─────────────────────────────────────────────────────────────────

/// An edge in the Portfolio component graph.
/// All relationships between components — and between federated portfolio systems
/// — are GraphEdge records.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub edge_id: Uuid,
    pub edge_type: GraphEdgeType,
    pub source_id: ComponentId,
    pub target_id: ComponentId,
    pub created_at: DateTime<Utc>,
    pub created_by: String,     // node ID
    pub metadata: JsonValue,
}

impl GraphEdge {
    pub fn new(edge_type: GraphEdgeType, source_id: ComponentId, target_id: ComponentId, by: &str) -> Self {
        Self {
            edge_id: Uuid::new_v4(),
            edge_type,
            source_id,
            target_id,
            created_at: Utc::now(),
            created_by: by.to_owned(),
            metadata: JsonValue::Null,
        }
    }

    pub fn hierarchy(parent: ComponentId, child: ComponentId, by: &str) -> Self {
        Self::new(GraphEdgeType::Hierarchy, parent, child, by)
    }

    pub fn dependency(from: ComponentId, to: ComponentId, by: &str) -> Self {
        Self::new(GraphEdgeType::Dependency, from, to, by)
    }

    pub fn link(a: ComponentId, b: ComponentId, by: &str) -> Self {
        Self::new(GraphEdgeType::Link, a, b, by)
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
        let mut e = Self::new(GraphEdgeType::ResourceShare, resource_id, target_entity, by);
        e.metadata = serde_json::json!({
            "share_policy_id": policy_id,
            "access_level": access_level,
            "expiry": expiry,
            "attribution_required": attribution_required,
        });
        e
    }
}

// ── Structural Primitives ─────────────────────────────────────────────────────

/// A bidirectional peer set — members cross-reference each other.
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

/// An unordered set of component IDs — arbitrary membership.
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

/// An ordered sequence — supports append, insert, move, remove.
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
        let index = index.min(self.items.len());
        self.items.insert(index, id);
    }

    pub fn remove(&mut self, id: &ComponentId) { self.items.retain(|i| i != id); }

    pub fn move_item(&mut self, id: &ComponentId, to_index: usize) {
        if let Some(pos) = self.items.iter().position(|i| i == id) {
            let item = self.items.remove(pos);
            let to = to_index.min(self.items.len());
            self.items.insert(to, item);
        }
    }
}

/// A scheduled item entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleEntry {
    pub entry_id: Uuid,
    pub component_id: ComponentId,
    pub event_at: DateTime<Utc>,
    pub label: Option<String>,
}

/// Time-ordered list of items with event timestamps.
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
        self.entries.push(ScheduleEntry {
            entry_id: Uuid::new_v4(),
            component_id,
            event_at,
            label,
        });
        // Keep chronologically sorted
        self.entries.sort_by_key(|e| e.event_at);
    }

    pub fn upcoming(&self, now: DateTime<Utc>) -> Vec<&ScheduleEntry> {
        self.entries.iter().filter(|e| e.event_at >= now).collect()
    }
}

/// Path-based spatial map (path string → component IDs).
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

    pub fn insert(&mut self, path: impl Into<String>, component_id: ComponentId) {
        self.paths.entry(path.into()).or_default().push(component_id);
    }

    pub fn remove(&mut self, path: &str, component_id: &ComponentId) {
        if let Some(ids) = self.paths.get_mut(path) {
            ids.retain(|id| id != component_id);
        }
    }

    pub fn list(&self, path: &str) -> Vec<ComponentId> {
        self.paths.get(path).cloned().unwrap_or_default()
    }

    /// List all paths under a prefix.
    pub fn list_prefix(&self, prefix: &str) -> Vec<(&str, &Vec<ComponentId>)> {
        self.paths.iter()
            .filter(|(p, _)| p.starts_with(prefix))
            .map(|(p, ids)| (p.as_str(), ids))
            .collect()
    }
}

// ── Cycle Detection Helper ────────────────────────────────────────────────────

/// Detect cycles in a dependency graph via DFS with a visited set.
/// Returns Some(cycle_path) if a cycle is detected, None otherwise.
pub fn detect_cycle(edges: &[GraphEdge]) -> Option<Vec<ComponentId>> {
    use std::collections::HashSet;

    // Build adjacency for Dependency edges only
    let mut adj: HashMap<ComponentId, Vec<ComponentId>> = HashMap::new();
    for e in edges {
        if e.edge_type == GraphEdgeType::Dependency {
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
            let cycle_start = rec_stack.iter().position(|&n| n == node).unwrap();
            return Some(rec_stack[cycle_start..].to_vec());
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

    for &node in adj.keys() {
        if !visited.contains(&node) {
            if let Some(cycle) = dfs(node, &adj, &mut visited, &mut rec_stack) {
                return Some(cycle);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_move_item() {
        let ids: Vec<Uuid> = (0..4).map(|_| Uuid::new_v4()).collect();
        let mut list = List::new("test");
        ids.iter().for_each(|&id| list.append(id));
        list.move_item(&ids[0], 3);
        assert_eq!(list.items.last(), Some(&ids[0]));
    }

    #[test]
    fn cycle_detection_no_cycle() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let edges = vec![GraphEdge::dependency(a, b, "node-1")];
        assert!(detect_cycle(&edges).is_none());
    }
}
