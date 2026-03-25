//! ComponentMetadata — universal identity and provenance block.
//! Uses hypergrid::crdt::VectorClock for causal ordering.

use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;


// Re-use hypergrid's VectorClock directly — no duplication.
use hypergrid::crdt::VectorClock;

use crate::types::{ComponentId, PolicyId, UserId, VersionString};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BumpKind { Major, Minor, Patch }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionHistoryEntry {
    pub version:         VersionString,
    pub bumped_at:       DateTime<Utc>,
    pub bumped_by:       String,
    pub change_summary:  Option<String>,
}

/// The universal identity and provenance block attached to every Component.
/// Stored as a set of HyperCells in the components Hypercube — one per field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetadata {
    pub id:              ComponentId,
    pub owners:          Vec<UserId>,
    pub tags:            HashSet<String>,
    pub policy_ids:      Vec<PolicyId>,
    pub created_at:      DateTime<Utc>,
    pub updated_at:      DateTime<Utc>,
    /// Node ID of last writer — CRDT tiebreak.
    pub last_actor:      String,
    /// Logical clock from hypergrid::crdt.
    pub vector_clock:    VectorClock,
    pub properties:      HashMap<String, JsonValue>,
    pub version:         VersionString,
    pub version_history: Vec<VersionHistoryEntry>,
    pub budget:          f64,
    pub budget_spent:    f64,
    pub resource_units:  f64,
}

impl ComponentMetadata {
    pub fn new(id: ComponentId, owner: UserId, node_id: &str) -> Self {
        let mut vc = VectorClock::new();
        vc.tick(node_id);
        let now = Utc::now();
        Self {
            id,
            owners: vec![owner],
            tags: HashSet::new(),
            policy_ids: vec![],
            created_at: now,
            updated_at: now,
            last_actor: node_id.to_owned(),
            vector_clock: vc,
            properties: HashMap::new(),
            version: "0.1.0".to_owned(),
            version_history: vec![],
            budget: 0.0,
            budget_spent: 0.0,
            resource_units: 0.0,
        }
    }

    pub fn bump_version(&mut self, kind: BumpKind, node_id: &str, summary: Option<String>) {
        let parts: Vec<u64> = self.version.splitn(3, '.').map(|p| p.parse().unwrap_or(0)).collect();
        let (major, minor, patch) = (
            parts.get(0).copied().unwrap_or(0),
            parts.get(1).copied().unwrap_or(0),
            parts.get(2).copied().unwrap_or(0),
        );
        let new_version = match kind {
            BumpKind::Major => format!("{}.0.0", major + 1),
            BumpKind::Minor => format!("{major}.{}.0", minor + 1),
            BumpKind::Patch => format!("{major}.{minor}.{}", patch + 1),
        };
        self.version_history.push(VersionHistoryEntry {
            version: self.version.clone(),
            bumped_at: Utc::now(),
            bumped_by: node_id.to_owned(),
            change_summary: summary,
        });
        self.version = new_version;
        self.touch(node_id);
    }

    pub fn touch(&mut self, node_id: &str) {
        self.vector_clock.tick(node_id);
        self.updated_at = Utc::now();
        self.last_actor = node_id.to_owned();
    }

    pub fn add_tag(&mut self, tag: impl Into<String>) { self.tags.insert(tag.into()); }
    pub fn remove_tag(&mut self, tag: &str) { self.tags.remove(tag); }
    pub fn has_policy(&self, policy_id: PolicyId) -> bool { self.policy_ids.contains(&policy_id) }
    pub fn budget_remaining(&self) -> f64 { (self.budget - self.budget_spent).max(0.0) }
    pub fn budget_utilisation(&self) -> f64 {
        if self.budget == 0.0 { 0.0 } else { (self.budget_spent / self.budget).min(1.0) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    #[test]
    fn bump_version_uses_hypergrid_vc() {
        let id = Uuid::new_v4();
        let owner = Uuid::new_v4();
        let mut meta = ComponentMetadata::new(id, owner, "node-1");
        meta.bump_version(BumpKind::Minor, "node-1", None);
        assert_eq!(meta.version, "0.2.0");
        // VectorClock came from hypergrid — should have been ticked
        assert!(meta.vector_clock.get("node-1") >= 2);
    }
}
