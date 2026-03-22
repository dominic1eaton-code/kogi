//! PortfolioSystem — the root orchestrator.
//!
//! The central in-memory data structure for the Kogi Portfolio System.
//! All public API methods are on this struct.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use chrono::Utc;
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::component::{
    Component, ComponentData, ComponentPayload, ContainerPayload, ItemPayload,
};
use crate::crdt::{CrdtLog, CrdtOperation, NodeId, VectorClock};
use crate::error::{PortfolioError, PortfolioResult};
use crate::events::{EventLog, PortfolioEvent, PortfolioEventKind, Snapshot};
use crate::federation::PortfolioFederation;
use crate::governance::{
    ApprovalRequest, GovernanceEngine, PolicyContext, PolicyDecision,
    ResourceAllocation, ResourceKind,
};
use crate::graph::{detect_cycle, GraphEdge, GraphEdgeType};
use crate::itembook::ItemBookData;
use crate::metadata::BumpKind;
use crate::models::{
    ArtifactMaturity, ArtifactMaturityInput,
    AssetValue, AssetValueInput,
    BinderCoverage, BinderCoverageInput,
    BookConsistency, BookConsistencyInput,
    FolderOrganisation, FolderOrganisationInput,
    PortfolioHealth, PortfolioHealthInput, PortfolioHealthResult,
    ProgramAlignment, ProgramAlignmentInput,
    ProjectMetrics, ProjectMetricsInput, ProjectMetricsResult,
    RecordIntegrity, RecordIntegrityInput,
    ResourceUtilisation, ResourceUtilisationInput,
    SubPortfolioRollup, SubPortfolioRollupInput,
};
use crate::plugin::PortfolioPlugin;
use crate::search::{PortfolioQuery, SearchQuery, SearchResult};
use crate::types::*;

// ── PortfolioSystem ───────────────────────────────────────────────────────────

/// The root data structure and API surface for the Kogi Portfolio System.
///
/// All components, edges, CRDT ops, governance, events, and plugins are
/// managed through a single `PortfolioSystem` instance per node.
pub struct PortfolioSystem {
    // ── State ──────────────────────────────────────────────────────────────
    /// The component registry: ComponentId → Component.
    components: HashMap<ComponentId, Component>,

    /// All graph edges (flat, CRDT-synced).
    edges: Vec<GraphEdge>,

    // ── Infrastructure ────────────────────────────────────────────────────
    pub event_log: EventLog,
    pub crdt_log: CrdtLog,
    pub governance: GovernanceEngine,
    pub federation: PortfolioFederation,

    // ── Identity ──────────────────────────────────────────────────────────
    /// This node's ID in the federation (used for VectorClock ticks).
    pub node_id: NodeId,

    // ── Plugins ───────────────────────────────────────────────────────────
    plugins: Vec<Arc<dyn PortfolioPlugin>>,
}

impl PortfolioSystem {
    // ── Constructor ────────────────────────────────────────────────────────

    pub fn new(node_id: impl Into<NodeId>) -> Self {
        Self {
            components: HashMap::new(),
            edges: vec![],
            event_log: EventLog::new(),
            crdt_log: CrdtLog::new(),
            governance: GovernanceEngine::new(),
            federation: PortfolioFederation::new(),
            node_id: node_id.into(),
            plugins: vec![],
        }
    }

    // ── Plugin registration ───────────────────────────────────────────────

    pub fn register_plugin(&mut self, plugin: Arc<dyn PortfolioPlugin>) {
        self.plugins.push(plugin);
    }

    fn fire_event(&mut self, event: PortfolioEvent) {
        for p in &self.plugins { p.on_event(&event); }
        self.event_log.emit(event);
    }

    // ── Component creation ────────────────────────────────────────────────

    /// Create and register a new Item component.
    pub fn create_item(
        &mut self,
        owner: UserId,
        name: impl Into<String>,
        item_category: ItemCategory,
        payload: ItemPayload,
    ) -> PortfolioResult<ComponentId> {
        let comp = Component::new_item(owner, name, item_category, payload, &self.node_id.clone());
        let id = comp.id();
        self.components.insert(id, comp);

        let event = PortfolioEvent::new(PortfolioEventKind::ComponentCreated, Some(id), &self.node_id.clone());
        let comp_ref = self.components.get(&id).unwrap();
        for p in &self.plugins { p.on_component_created(comp_ref); }
        self.event_log.emit(event);

        self.crdt_log.append(CrdtOperation::SetField {
            component_id: id,
            field: "status".into(),
            value: serde_json::json!("Draft"),
            timestamp: Utc::now(),
            actor: self.node_id.clone(),
        });
        Ok(id)
    }

    /// Create and register a new Container component.
    pub fn create_container(
        &mut self,
        owner: UserId,
        name: impl Into<String>,
        container_category: ContainerCategory,
        payload: ContainerPayload,
    ) -> PortfolioResult<ComponentId> {
        let comp = Component::new_container(owner, name, container_category, payload, &self.node_id.clone());
        let id = comp.id();
        self.components.insert(id, comp);

        let event = PortfolioEvent::new(PortfolioEventKind::ComponentCreated, Some(id), &self.node_id.clone());
        let comp_ref = self.components.get(&id).unwrap();
        for p in &self.plugins { p.on_component_created(comp_ref); }
        self.event_log.emit(event);
        Ok(id)
    }

    // ── Component read access ─────────────────────────────────────────────

    pub fn get(&self, id: ComponentId) -> PortfolioResult<&Component> {
        self.components.get(&id).ok_or(PortfolioError::NotFound(id))
    }

    pub fn get_mut(&mut self, id: ComponentId) -> PortfolioResult<&mut Component> {
        self.components.get_mut(&id).ok_or(PortfolioError::NotFound(id))
    }

    pub fn all_components(&self) -> Vec<&Component> {
        self.components.values().collect()
    }

    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    // ── Component mutation ────────────────────────────────────────────────

    /// Update name and/or description of a component.
    pub fn update_info(
        &mut self,
        user_id: UserId,
        id: ComponentId,
        name: Option<String>,
        description: Option<String>,
    ) -> PortfolioResult<()> {
        let comp = self.get_mut(id)?;
        if !comp.check_permission(&user_id, PermissionTier::Editor) {
            return Err(PortfolioError::PermissionDenied {
                user_id: user_id.to_string(),
                action: "update_info".into(),
            });
        }
        comp.update_info(name, description, &self.node_id.clone());
        let event = PortfolioEvent::new(PortfolioEventKind::ComponentUpdated, Some(id), &self.node_id.clone());
        for p in &self.plugins { p.on_component_updated(self.components.get(&id).unwrap()); }
        self.event_log.emit(event);
        Ok(())
    }

    /// Soft or hard delete a component.
    pub fn delete(
        &mut self,
        user_id: UserId,
        id: ComponentId,
        hard_delete: bool,
    ) -> PortfolioResult<()> {
        let comp = self.get_mut(id)?;
        if !comp.check_permission(&user_id, PermissionTier::Owner) {
            return Err(PortfolioError::PermissionDenied {
                user_id: user_id.to_string(),
                action: "delete".into(),
            });
        }
        if hard_delete {
            if let Some(removed) = self.components.remove(&id) {
                // Purge edges
                self.edges.retain(|e| e.source_id != id && e.target_id != id);
                for p in &self.plugins { p.on_component_removed(id); }
                let event = PortfolioEvent::new(PortfolioEventKind::ComponentRemoved, Some(id), &self.node_id.clone());
                self.event_log.emit(event);
            }
        } else {
            // Soft delete: retain edges, mark status
            let node = &self.node_id.clone();
            let comp = self.get_mut(id)?;
            comp.set_status(ComponentStatus::Deleted, node);
            let event = PortfolioEvent::new(PortfolioEventKind::ComponentStatusChanged, Some(id), &self.node_id.clone());
            self.event_log.emit(event);
        }
        Ok(())
    }

    /// Apply an action (social, content, economic, access, system).
    pub fn act(
        &mut self,
        user_id: UserId,
        id: ComponentId,
        action: ActionKind,
    ) -> PortfolioResult<()> {
        let comp = self.get_mut(id)?;
        let required = match &action {
            ActionKind::Edit => PermissionTier::Editor,
            ActionKind::Delete => PermissionTier::Owner,
            ActionKind::Archive => PermissionTier::Manager,
            ActionKind::Own { tier } => PermissionTier::Owner,
            _ => PermissionTier::Contributor,
        };
        if !comp.check_permission(&user_id, required) {
            return Err(PortfolioError::PermissionDenied {
                user_id: user_id.to_string(),
                action: format!("{action:?}"),
            });
        }
        let node = self.node_id.clone();
        let comp = self.get_mut(id)?;
        comp.apply_action(action, user_id);
        comp.metadata.touch(&node);

        let event = PortfolioEvent::new(PortfolioEventKind::ActionPerformed, Some(id), &node);
        for p in &self.plugins { p.on_component_updated(self.components.get(&id).unwrap()); }
        self.event_log.emit(event);
        Ok(())
    }

    /// Bump the semantic version of a component.
    pub fn bump_version(
        &mut self,
        user_id: UserId,
        id: ComponentId,
        kind: BumpKind,
        summary: Option<String>,
    ) -> PortfolioResult<String> {
        let comp = self.get_mut(id)?;
        if !comp.check_permission(&user_id, PermissionTier::Editor) {
            return Err(PortfolioError::PermissionDenied {
                user_id: user_id.to_string(),
                action: "bump_version".into(),
            });
        }
        let node = self.node_id.clone();
        comp.bump_version(kind, &node, summary);
        let version = comp.metadata.version.clone();
        let event = PortfolioEvent::new(PortfolioEventKind::ComponentUpdated, Some(id), &node);
        self.event_log.emit(event);
        Ok(version)
    }

    // ── User management ───────────────────────────────────────────────────

    pub fn add_user(
        &mut self,
        actor: UserId,
        id: ComponentId,
        user_id: UserId,
        tier: PermissionTier,
    ) -> PortfolioResult<()> {
        let comp = self.get_mut(id)?;
        if !comp.check_permission(&actor, PermissionTier::Manager) {
            return Err(PortfolioError::PermissionDenied {
                user_id: actor.to_string(),
                action: "add_user".into(),
            });
        }
        comp.data.users.add_user(user_id, tier);
        let node = self.node_id.clone();
        comp.metadata.touch(&node);
        let event = PortfolioEvent::new(PortfolioEventKind::UserAdded, Some(id), &node);
        self.event_log.emit(event);
        Ok(())
    }

    pub fn remove_user(
        &mut self,
        actor: UserId,
        id: ComponentId,
        user_id: &UserId,
    ) -> PortfolioResult<()> {
        let comp = self.get_mut(id)?;
        if !comp.check_permission(&actor, PermissionTier::Manager) {
            return Err(PortfolioError::PermissionDenied {
                user_id: actor.to_string(),
                action: "remove_user".into(),
            });
        }
        comp.data.users.remove_user(user_id);
        let node = self.node_id.clone();
        comp.metadata.touch(&node);
        let event = PortfolioEvent::new(PortfolioEventKind::UserRemoved, Some(id), &node);
        self.event_log.emit(event);
        Ok(())
    }

    // ── Graph (edges) ─────────────────────────────────────────────────────

    /// Attach a child component (Hierarchy edge).
    pub fn attach_child(
        &mut self,
        user_id: UserId,
        parent_id: ComponentId,
        child_id: ComponentId,
    ) -> PortfolioResult<()> {
        self.get(parent_id)?;
        self.get(child_id)?;
        let parent = self.get_mut(parent_id)?;
        if !parent.check_permission(&user_id, PermissionTier::Manager) {
            return Err(PortfolioError::PermissionDenied {
                user_id: user_id.to_string(),
                action: "attach_child".into(),
            });
        }
        parent.data.children.push(child_id);
        let node = self.node_id.clone();
        parent.metadata.touch(&node);

        let child = self.get_mut(child_id)?;
        child.data.parents.push(parent_id);

        let edge = GraphEdge::hierarchy(parent_id, child_id, &node);
        let eid = edge.edge_id;
        self.edges.push(edge.clone());

        for p in &self.plugins { p.on_edge_added(&self.edges.last().unwrap()); }
        self.crdt_log.append(CrdtOperation::AddEdge {
            edge: edge,
            timestamp: Utc::now(),
            actor: node.clone(),
        });
        let event = PortfolioEvent::new(PortfolioEventKind::EdgeAdded, Some(parent_id), &node)
            .with_payload(serde_json::json!({ "edge_id": eid, "child_id": child_id }));
        self.event_log.emit(event);
        Ok(())
    }

    /// Add a dependency edge from `from_id` to `to_id`.
    /// Performs shallow cycle detection.
    pub fn add_dependency(
        &mut self,
        user_id: UserId,
        from_id: ComponentId,
        to_id: ComponentId,
    ) -> PortfolioResult<()> {
        self.get(from_id)?;
        self.get(to_id)?;

        // Shallow cycle check: reject if `to` already depends on `from`
        let to_comp = self.get(to_id)?;
        if to_comp.data.dependencies.contains(&from_id) {
            return Err(PortfolioError::CyclicDependency(from_id, to_id));
        }
        // Deep cycle check via DFS
        let potential_edges: Vec<GraphEdge> = {
            let mut all = self.edges.clone();
            all.push(GraphEdge::dependency(from_id, to_id, &self.node_id));
            all
        };
        if detect_cycle(&potential_edges).is_some() {
            return Err(PortfolioError::CyclicDependency(from_id, to_id));
        }

        let node = self.node_id.clone();
        let from_comp = self.get_mut(from_id)?;
        if !from_comp.check_permission(&user_id, PermissionTier::Editor) {
            return Err(PortfolioError::PermissionDenied {
                user_id: user_id.to_string(),
                action: "add_dependency".into(),
            });
        }
        from_comp.data.dependencies.push(to_id);

        let to_comp = self.get_mut(to_id)?;
        to_comp.data.dependents.push(from_id);

        let edge = GraphEdge::dependency(from_id, to_id, &node);
        self.edges.push(edge.clone());
        let event = PortfolioEvent::new(PortfolioEventKind::EdgeAdded, Some(from_id), &node);
        self.event_log.emit(event);
        Ok(())
    }

    /// Add a soft link between two components.
    pub fn add_link(
        &mut self,
        user_id: UserId,
        a: ComponentId,
        b: ComponentId,
    ) -> PortfolioResult<()> {
        self.get(a)?;
        self.get(b)?;
        let node = self.node_id.clone();
        let comp_a = self.get_mut(a)?;
        if !comp_a.check_permission(&user_id, PermissionTier::Editor) {
            return Err(PortfolioError::PermissionDenied {
                user_id: user_id.to_string(),
                action: "add_link".into(),
            });
        }
        comp_a.data.links.push(b);
        let edge = GraphEdge::link(a, b, &node);
        self.edges.push(edge);
        let event = PortfolioEvent::new(PortfolioEventKind::EdgeAdded, Some(a), &node);
        self.event_log.emit(event);
        Ok(())
    }

    pub fn detach_child(&mut self, user_id: UserId, parent_id: ComponentId, child_id: ComponentId) -> PortfolioResult<()> {
        let comp = self.get_mut(parent_id)?;
        if !comp.check_permission(&user_id, PermissionTier::Manager) {
            return Err(PortfolioError::PermissionDenied { user_id: user_id.to_string(), action: "detach_child".into() });
        }
        comp.data.children.retain(|&c| c != child_id);
        self.edges.retain(|e| !(e.source_id == parent_id && e.target_id == child_id && e.edge_type == GraphEdgeType::Hierarchy));
        for p in &self.plugins {
            // notify edge removed (simplified — we don't have edge_id handy after retain)
        }
        let node = self.node_id.clone();
        let event = PortfolioEvent::new(PortfolioEventKind::EdgeRemoved, Some(parent_id), &node);
        self.event_log.emit(event);
        Ok(())
    }

    pub fn remove_dependency(&mut self, user_id: UserId, from_id: ComponentId, to_id: ComponentId) -> PortfolioResult<()> {
        let comp = self.get_mut(from_id)?;
        if !comp.check_permission(&user_id, PermissionTier::Editor) {
            return Err(PortfolioError::PermissionDenied { user_id: user_id.to_string(), action: "remove_dependency".into() });
        }
        comp.data.dependencies.retain(|&d| d != to_id);
        self.edges.retain(|e| !(e.source_id == from_id && e.target_id == to_id && e.edge_type == GraphEdgeType::Dependency));
        let node = self.node_id.clone();
        let event = PortfolioEvent::new(PortfolioEventKind::EdgeRemoved, Some(from_id), &node);
        self.event_log.emit(event);
        Ok(())
    }

    pub fn edges_for(&self, id: ComponentId) -> Vec<&GraphEdge> {
        self.edges.iter().filter(|e| e.source_id == id || e.target_id == id).collect()
    }

    // ── Policy & Governance ───────────────────────────────────────────────

    pub fn attach_policy(&mut self, id: ComponentId, policy_id: PolicyId) -> PortfolioResult<()> {
        let comp = self.get_mut(id)?;
        if !comp.metadata.has_policy(policy_id) {
            comp.metadata.policy_ids.push(policy_id);
        }
        let node = self.node_id.clone();
        comp.metadata.touch(&node);
        let event = PortfolioEvent::new(PortfolioEventKind::PolicyAttached, Some(id), &node);
        self.event_log.emit(event);
        Ok(())
    }

    pub fn detach_policy(&mut self, id: ComponentId, policy_id: PolicyId) -> PortfolioResult<()> {
        let comp = self.get_mut(id)?;
        comp.metadata.policy_ids.retain(|&p| p != policy_id);
        let node = self.node_id.clone();
        comp.metadata.touch(&node);
        let event = PortfolioEvent::new(PortfolioEventKind::PolicyDetached, Some(id), &node);
        self.event_log.emit(event);
        Ok(())
    }

    /// Evaluate all registered policies for the given component + event.
    pub fn evaluate_policy(&self, id: ComponentId, event_kind: &PortfolioEventKind, actor: &str) -> PortfolioResult<PolicyDecision> {
        let comp = self.get(id)?;
        let ctx = PolicyContext::new(comp, event_kind, actor);
        Ok(self.governance.evaluate(&ctx))
    }

    pub fn request_approval(&mut self, id: ComponentId, reason: &str) -> PortfolioResult<Uuid> {
        self.get(id)?;
        let node = self.node_id.clone();
        let req_id = self.governance.request_approval(id, reason, &node);
        let event = PortfolioEvent::new(PortfolioEventKind::ApprovalRequested, Some(id), &node);
        self.event_log.emit(event);
        Ok(req_id)
    }

    pub fn resolve_approval(&mut self, request_id: Uuid, approved: bool, resolver: &str, notes: Option<String>) -> PortfolioResult<()> {
        if !self.governance.resolve_approval(request_id, approved, resolver, notes) {
            return Err(PortfolioError::NotFound(request_id));
        }
        let event_kind = if approved { PortfolioEventKind::ApprovalGranted } else { PortfolioEventKind::ApprovalRejected };
        let node = self.node_id.clone();
        let event = PortfolioEvent::new(event_kind, None, &node);
        self.event_log.emit(event);
        Ok(())
    }

    // ── Resource Allocation ───────────────────────────────────────────────

    pub fn allocate_budget(
        &mut self,
        id: ComponentId,
        total: f64,
        currency: &str,
        period: Option<String>,
    ) -> PortfolioResult<Uuid> {
        self.get(id)?;
        let alloc_id = self.governance.allocate_budget(id, total, currency, period);
        let comp = self.get_mut(id).unwrap();
        comp.metadata.budget = total;
        let node = self.node_id.clone();
        comp.metadata.touch(&node);
        let event = PortfolioEvent::new(PortfolioEventKind::ResourceAllocated, Some(id), &node);
        self.event_log.emit(event);
        Ok(alloc_id)
    }

    pub fn record_consumption(&mut self, id: ComponentId, amount: f64) -> PortfolioResult<f64> {
        self.get(id)?;
        let remaining = self.governance.record_consumption(id, amount)
            .ok_or(PortfolioError::InvalidOperation(format!("No allocation for {id}")))?;
        let comp = self.get_mut(id).unwrap();
        comp.metadata.budget_spent += amount;
        let node = self.node_id.clone();
        comp.metadata.touch(&node);
        let event = PortfolioEvent::new(PortfolioEventKind::ResourceConsumed, Some(id), &node);
        self.event_log.emit(event);
        Ok(remaining)
    }

    pub fn get_resource_allocation(&self, id: ComponentId) -> PortfolioResult<Option<&ResourceAllocation>> {
        self.get(id)?;
        Ok(self.governance.get_allocation(id))
    }

    pub fn overrun_allocations(&self) -> Vec<&ResourceAllocation> {
        self.governance.overrun_allocations()
    }

    // ── ToolBox (TMS v2.1) ────────────────────────────────────────────────

    pub fn attach_toolbox(&mut self, user_id: UserId, id: ComponentId, tb: ToolBoxId) -> PortfolioResult<()> {
        let comp = self.get_mut(id)?;
        if !comp.check_permission(&user_id, PermissionTier::Owner) {
            return Err(PortfolioError::PermissionDenied { user_id: user_id.to_string(), action: "attach_toolbox".into() });
        }
        comp.attach_toolbox(tb);
        let node = self.node_id.clone();
        comp.metadata.touch(&node);
        let event = PortfolioEvent::new(PortfolioEventKind::ToolboxAttached, Some(id), &node);
        self.event_log.emit(event);
        Ok(())
    }

    pub fn detach_toolbox(&mut self, user_id: UserId, id: ComponentId, tb: ToolBoxId) -> PortfolioResult<()> {
        let comp = self.get_mut(id)?;
        if !comp.check_permission(&user_id, PermissionTier::Owner) {
            return Err(PortfolioError::PermissionDenied { user_id: user_id.to_string(), action: "detach_toolbox".into() });
        }
        comp.detach_toolbox(tb);
        let node = self.node_id.clone();
        comp.metadata.touch(&node);
        let event = PortfolioEvent::new(PortfolioEventKind::ToolboxDetached, Some(id), &node);
        self.event_log.emit(event);
        Ok(())
    }

    pub fn has_toolbox(&self, id: ComponentId, tb: ToolBoxId) -> PortfolioResult<bool> {
        Ok(self.get(id)?.has_toolbox(tb))
    }

    pub fn toolbox_ids(&self, id: ComponentId) -> PortfolioResult<Vec<ToolBoxId>> {
        Ok(self.get(id)?.data.toolbox_ids.clone())
    }

    pub fn all_toolbox_ids(&self) -> HashSet<ToolBoxId> {
        self.components.values().flat_map(|c| c.data.toolbox_ids.iter().copied()).collect()
    }

    // ── CRDT / Federation ─────────────────────────────────────────────────

    /// Merge a remote CRDT log delta into this system.
    pub fn apply_crdt_log(&mut self, remote: &CrdtLog) -> PortfolioResult<usize> {
        let ops = remote.delta_since(&self.node_id).to_vec();
        let n = ops.len();
        for op in &ops {
            self.apply_crdt_op(op.clone())?;
        }
        self.crdt_log.merge_from(remote);
        for p in &self.plugins { p.on_crdt_merge(&ops); }
        let node = self.node_id.clone();
        let event = PortfolioEvent::new(PortfolioEventKind::CrdtMergeApplied, None, &node)
            .with_payload(serde_json::json!({ "ops_applied": n }));
        self.event_log.emit(event);
        Ok(n)
    }

    fn apply_crdt_op(&mut self, op: CrdtOperation) -> PortfolioResult<()> {
        match op {
            CrdtOperation::SetField { component_id, field, value, timestamp, actor } => {
                if let Some(comp) = self.components.get_mut(&component_id) {
                    // LWW: accept if remote is newer or actor sorts higher
                    let accept = comp.metadata.updated_at < timestamp
                        || (comp.metadata.updated_at == timestamp && actor > comp.metadata.last_actor);
                    if accept {
                        match field.as_str() {
                            "name" => {
                                if let Some(s) = value.as_str() {
                                    comp.data.name = s.to_owned();
                                }
                            }
                            "description" => {
                                if let Some(s) = value.as_str() {
                                    comp.data.description = s.to_owned();
                                }
                            }
                            // status mutations are skipped — require lattice merge (§17.3)
                            "status" => {}
                            key => {
                                comp.metadata.properties.insert(key.to_owned(), value);
                            }
                        }
                        comp.metadata.last_actor = actor;
                        comp.metadata.updated_at = timestamp;
                    }
                }
            }
            CrdtOperation::AddEdge { edge, .. } => {
                if !self.edges.iter().any(|e| e.edge_id == edge.edge_id) {
                    self.edges.push(edge);
                }
            }
            CrdtOperation::RemoveEdge { edge_id, .. } => {
                self.edges.retain(|e| e.edge_id != edge_id);
            }
            CrdtOperation::AddToSet { component_id, set_name, value, tag, .. } => {
                if let Some(comp) = self.components.get_mut(&component_id) {
                    match set_name.as_str() {
                        "tags" => { if let Some(s) = value.as_str() { comp.metadata.tags.insert(s.to_owned()); } }
                        "hashtags" => { if let Some(s) = value.as_str() { comp.data.hashtags.insert(s.to_owned()); } }
                        "topics" => { if let Some(s) = value.as_str() { comp.data.topics.insert(s.to_owned()); } }
                        _ => {}
                    }
                }
            }
            CrdtOperation::RemoveFromSet { component_id, set_name, tag, .. } => {
                // OR-Set: remove by tag. Simplified: skip (production would track tags).
            }
        }
        Ok(())
    }

    // ── Snapshot / Time-Travel ─────────────────────────────────────────────

    /// Return a point-in-time snapshot struct (not persisted).
    pub fn snapshot(&self) -> serde_json::Value {
        serde_json::json!({
            "component_count": self.components.len(),
            "edge_count": self.edges.len(),
            "event_count": self.event_log.len(),
            "node_id": self.node_id,
            "snapshot_at": Utc::now(),
        })
    }

    /// Save a named, persisted snapshot.
    pub fn save_snapshot(&mut self, label: impl Into<String>) -> Uuid {
        let node = self.node_id.clone();
        let snap = Snapshot::new(label, None, &node, false);
        let id = snap.snapshot_id;
        for p in &self.plugins { p.on_snapshot_saved(&snap); }
        let event = PortfolioEvent::new(PortfolioEventKind::SnapshotSaved, None, &node);
        self.event_log.emit(event);
        self.event_log.save_snapshot(snap);
        id
    }

    /// Save a named checkpoint (immutable reference point).
    pub fn save_checkpoint(&mut self, label: impl Into<String>, note: Option<String>) -> Uuid {
        let node = self.node_id.clone();
        let snap = Snapshot::new(label, note, &node, true);
        let id = snap.snapshot_id;
        for p in &self.plugins { p.on_snapshot_saved(&snap); }
        let event = PortfolioEvent::new(PortfolioEventKind::CheckpointCreated, None, &node);
        self.event_log.emit(event);
        self.event_log.save_snapshot(snap);
        id
    }

    // ── Search ────────────────────────────────────────────────────────────

    /// Basic in-process search over the component store.
    pub fn search(&self, query: &SearchQuery) -> Vec<SearchResult> {
        let mut results: Vec<SearchResult> = self.components.values()
            .filter(|c| {
                // text filter
                if let Some(ref text) = query.text {
                    let q = text.to_lowercase();
                    if !c.data.name.to_lowercase().contains(&q) && !c.data.description.to_lowercase().contains(&q) {
                        return false;
                    }
                }
                // tag filter
                if !query.tags.is_empty() {
                    let has_tag = query.tags.iter().any(|t| c.metadata.tags.contains(t));
                    if !has_tag { return false; }
                }
                // status filter
                if !query.statuses.is_empty() && !query.statuses.contains(&c.data.status) {
                    return false;
                }
                // category filter
                if !query.categories.is_empty() && !query.categories.contains(&c.data.category) {
                    return false;
                }
                // owner filter
                if let Some(owner) = query.owner {
                    if !c.metadata.owners.contains(&owner) { return false; }
                }
                // visibility filter
                if let Some(ref vis) = query.visibility {
                    if &c.data.visibility != vis { return false; }
                }
                // date range
                if let Some(after) = query.created_after {
                    if c.metadata.created_at < after { return false; }
                }
                if let Some(before) = query.created_before {
                    if c.metadata.created_at > before { return false; }
                }
                true
            })
            .map(|c| SearchResult {
                component_id: c.id(),
                name: c.data.name.clone(),
                category: c.data.category.clone(),
                status: c.data.status.clone(),
                score: 1.0,
                matched_fields: vec!["name".into()],
            })
            .skip(query.offset)
            .take(query.limit)
            .collect();

        results
    }

    /// Apply a structured PortfolioQuery (PQL) predicate.
    pub fn query(&self, pql: &PortfolioQuery) -> Vec<&Component> {
        self.components.values()
            .filter(|c| {
                if !pql.categories.is_empty() && !pql.categories.contains(&c.data.category) { return false; }
                if !pql.statuses.is_empty() && !pql.statuses.contains(&c.data.status) { return false; }
                if !pql.tags.is_empty() {
                    let has_tag = pql.tags.iter().any(|t| c.metadata.tags.contains(t));
                    if !has_tag { return false; }
                }
                if let Some(ref nc) = pql.name_contains {
                    if !c.data.name.to_lowercase().contains(&nc.to_lowercase()) { return false; }
                }
                if let Some(owner) = pql.owner {
                    if !c.metadata.owners.contains(&owner) { return false; }
                }
                if let Some(hd) = pql.has_dependencies {
                    if c.data.dependencies.is_empty() == hd { return false; }
                }
                if let Some(hc) = pql.has_children {
                    if c.data.children.is_empty() == hc { return false; }
                }
                if let Some((ref key, ref val)) = pql.property_filter {
                    if c.metadata.properties.get(key) != Some(val) { return false; }
                }
                true
            })
            .collect()
    }

    // ── Computational Models ──────────────────────────────────────────────

    /// Compute the PortfolioHealth score for all components.
    pub fn compute_health(&self) -> PortfolioHealthResult {
        let total = self.components.len();
        let active = self.components.values().filter(|c| c.data.status == ComponentStatus::Active).count();
        let (total_budget, total_spent) = self.components.values().fold((0.0, 0.0), |(b, s), c| {
            (b + c.metadata.budget, s + c.metadata.budget_spent)
        });
        let avg_risk = {
            let all_risks: Vec<f64> = self.components.values()
                .flat_map(|c| c.data.risks.iter().map(|r| r.severity.score()))
                .collect();
            if all_risks.is_empty() { 1.0 } else { all_risks.iter().sum::<f64>() / all_risks.len() as f64 }
        };
        PortfolioHealth::compute(&PortfolioHealthInput {
            total_components: total,
            active_components: active,
            budget_allocated: total_budget,
            budget_consumed: total_spent,
            avg_risk_severity: avg_risk,
        })
    }

    /// Compute ProjectMetrics for a specific project component.
    pub fn compute_project_metrics(&self, id: ComponentId) -> PortfolioResult<ProjectMetricsResult> {
        let comp = self.get(id)?;
        let payload = match &comp.payload {
            ComponentPayload::Item(ItemPayload::Project(p)) => p,
            _ => return Err(PortfolioError::InvalidOperation(format!("{id} is not a Project"))),
        };
        let total_tasks = payload.backlog.len();
        let completed = payload.backlog.iter().filter(|b| b.status == ComponentStatus::Completed).count();
        let overdue = 0usize; // would check due_date vs now in production
        let sp_done: u32 = payload.backlog.iter().filter(|b| b.status == ComponentStatus::Completed).map(|b| b.story_points).sum();
        let sp_total: u32 = payload.backlog.iter().map(|b| b.story_points).sum();
        let velocity = if payload.sprints.is_empty() { 0.0 } else {
            payload.sprints.iter().map(|s| s.velocity).sum::<f64>() / payload.sprints.len() as f64
        };
        Ok(ProjectMetrics::compute(&ProjectMetricsInput {
            total_tasks,
            completed_tasks: completed,
            overdue_tasks: overdue,
            backlog_size: total_tasks.saturating_sub(completed),
            sprint_velocity: velocity,
            story_points_completed: sp_done,
            story_points_total: sp_total,
        }))
    }

    // ── Misc utilities ────────────────────────────────────────────────────

    pub fn children_of(&self, id: ComponentId) -> PortfolioResult<Vec<&Component>> {
        let comp = self.get(id)?;
        Ok(comp.data.children.iter().filter_map(|c| self.components.get(c)).collect())
    }

    pub fn ancestors_of(&self, id: ComponentId) -> PortfolioResult<Vec<&Component>> {
        let comp = self.get(id)?;
        Ok(comp.data.parents.iter().filter_map(|p| self.components.get(p)).collect())
    }

    /// Deep cycle detection across all Dependency edges.
    pub fn detect_cycle(&self) -> Option<Vec<ComponentId>> {
        detect_cycle(&self.edges)
    }

    pub fn stats(&self) -> SystemStats {
        SystemStats {
            component_count: self.components.len(),
            edge_count: self.edges.len(),
            event_count: self.event_log.len(),
            crdt_op_count: self.crdt_log.len(),
            pending_approvals: self.governance.pending_approvals().len(),
            federation_peers: self.federation.peers.len(),
        }
    }
}

// ── SystemStats ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SystemStats {
    pub component_count: usize,
    pub edge_count: usize,
    pub event_count: usize,
    pub crdt_op_count: usize,
    pub pending_approvals: usize,
    pub federation_peers: usize,
}

// ─────────────────────────────────────────────────────────────────────────────
//  Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::{PortfolioItemPayload, ProjectPayload};

    fn make_system() -> PortfolioSystem {
        PortfolioSystem::new("node-1")
    }

    fn owner() -> UserId { Uuid::new_v4() }

    #[test]
    fn create_item_and_get() {
        let mut sys = make_system();
        let o = owner();
        let id = sys.create_item(
            o,
            "My Portfolio",
            ItemCategory::Portfolio,
            ItemPayload::Portfolio(PortfolioItemPayload {
                mission: Some("Enable independent work".into()),
                focus_areas: vec!["tech".into()],
                kpis: vec![],
            }),
        ).unwrap();
        let comp = sys.get(id).unwrap();
        assert_eq!(comp.name(), "My Portfolio");
        assert_eq!(comp.status(), &ComponentStatus::Draft);
    }

    #[test]
    fn permission_denied_for_stranger() {
        let mut sys = make_system();
        let o = owner();
        let stranger = owner();
        let id = sys.create_item(
            o,
            "Secret Project",
            ItemCategory::Project,
            ItemPayload::Project(ProjectPayload {
                project_type: ProjectType::Technical,
                methodology: Methodology::Agile,
                start_date: None,
                end_date: None,
                sprints: vec![],
                backlog: vec![],
                releases: vec![],
                completion_pct: 0.0,
            }),
        ).unwrap();
        let result = sys.update_info(stranger, id, Some("Hacked".into()), None);
        assert!(matches!(result, Err(PortfolioError::PermissionDenied { .. })));
    }

    #[test]
    fn dependency_cycle_detection() {
        let mut sys = make_system();
        let o = owner();
        let a = sys.create_item(o, "A", ItemCategory::Project, ItemPayload::Project(ProjectPayload {
            project_type: ProjectType::Technical, methodology: Methodology::Agile,
            start_date: None, end_date: None, sprints: vec![], backlog: vec![], releases: vec![], completion_pct: 0.0,
        })).unwrap();
        let b = sys.create_item(o, "B", ItemCategory::Project, ItemPayload::Project(ProjectPayload {
            project_type: ProjectType::Technical, methodology: Methodology::Agile,
            start_date: None, end_date: None, sprints: vec![], backlog: vec![], releases: vec![], completion_pct: 0.0,
        })).unwrap();
        sys.add_dependency(o, a, b).unwrap();
        let result = sys.add_dependency(o, b, a);
        assert!(matches!(result, Err(PortfolioError::CyclicDependency(_, _))));
    }

    #[test]
    fn budget_allocation_and_consumption() {
        let mut sys = make_system();
        let o = owner();
        let id = sys.create_item(o, "Project X", ItemCategory::Project, ItemPayload::Project(ProjectPayload {
            project_type: ProjectType::Technical, methodology: Methodology::Agile,
            start_date: None, end_date: None, sprints: vec![], backlog: vec![], releases: vec![], completion_pct: 0.0,
        })).unwrap();
        sys.allocate_budget(id, 10_000.0, "USD", None).unwrap();
        let remaining = sys.record_consumption(id, 3_000.0).unwrap();
        assert!((remaining - 7_000.0).abs() < 0.01);
        let comp = sys.get(id).unwrap();
        assert!((comp.metadata.budget_spent - 3_000.0).abs() < 0.01);
    }

    #[test]
    fn version_bump() {
        let mut sys = make_system();
        let o = owner();
        let id = sys.create_item(o, "Asset", ItemCategory::Asset, ItemPayload::Asset(
            crate::component::AssetPayload {
                asset_type: AssetType::Digital,
                valuation: 5000.0,
                currency: "USD".into(),
                acquired_at: Utc::now(),
                depreciation_rate_annual: 0.1,
                cost_basis: 5000.0,
                vesting_schedule: None,
            }
        )).unwrap();
        let v = sys.bump_version(o, id, BumpKind::Minor, Some("Added branding".into())).unwrap();
        assert_eq!(v, "0.2.0");
    }

    #[test]
    fn toolbox_attach_and_list() {
        let mut sys = make_system();
        let o = owner();
        let id = sys.create_item(o, "Biz Portfolio", ItemCategory::Portfolio, ItemPayload::Portfolio(
            PortfolioItemPayload { mission: None, focus_areas: vec![], kpis: vec![] }
        )).unwrap();
        let tb1 = Uuid::new_v4();
        let tb2 = Uuid::new_v4();
        sys.attach_toolbox(o, id, tb1).unwrap();
        sys.attach_toolbox(o, id, tb2).unwrap();
        let ids = sys.toolbox_ids(id).unwrap();
        assert_eq!(ids.len(), 2);
        sys.detach_toolbox(o, id, tb1).unwrap();
        let ids = sys.toolbox_ids(id).unwrap();
        assert_eq!(ids.len(), 1);
    }

    #[test]
    fn search_by_text() {
        let mut sys = make_system();
        let o = owner();
        sys.create_item(o, "Alpha Project", ItemCategory::Project, ItemPayload::Project(ProjectPayload {
            project_type: ProjectType::Technical, methodology: Methodology::Agile,
            start_date: None, end_date: None, sprints: vec![], backlog: vec![], releases: vec![], completion_pct: 0.0,
        })).unwrap();
        sys.create_item(o, "Beta Asset", ItemCategory::Asset, ItemPayload::Asset(
            crate::component::AssetPayload {
                asset_type: AssetType::Digital, valuation: 0.0, currency: "USD".into(),
                acquired_at: Utc::now(), depreciation_rate_annual: 0.0, cost_basis: 0.0, vesting_schedule: None,
            }
        )).unwrap();
        let q = SearchQuery::new().text("Alpha");
        let results = sys.search(&q);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Alpha Project");
    }

    #[test]
    fn portfolio_health_smoke() {
        let mut sys = make_system();
        let o = owner();
        for i in 0..5 {
            let id = sys.create_item(o, format!("Project {i}"), ItemCategory::Project, ItemPayload::Project(ProjectPayload {
                project_type: ProjectType::Technical, methodology: Methodology::Agile,
                start_date: None, end_date: None, sprints: vec![], backlog: vec![], releases: vec![], completion_pct: 0.0,
            })).unwrap();
            // activate 4 of them
            if i < 4 {
                sys.get_mut(id).unwrap().set_status(ComponentStatus::Active, "node-1");
            }
        }
        let health = sys.compute_health();
        assert!(health.health_score > 0.0);
        assert!(health.health_score <= 100.0);
    }

    #[test]
    fn approval_workflow() {
        let mut sys = make_system();
        let o = owner();
        let id = sys.create_item(o, "Regulated Asset", ItemCategory::Asset, ItemPayload::Asset(
            crate::component::AssetPayload {
                asset_type: AssetType::Financial, valuation: 100_000.0, currency: "USD".into(),
                acquired_at: Utc::now(), depreciation_rate_annual: 0.0, cost_basis: 100_000.0, vesting_schedule: None,
            }
        )).unwrap();
        let req_id = sys.request_approval(id, "Budget threshold exceeded").unwrap();
        assert_eq!(sys.governance.pending_approvals().len(), 1);
        sys.resolve_approval(req_id, true, "admin-node", Some("Approved by CFO".into())).unwrap();
        assert_eq!(sys.governance.pending_approvals().len(), 0);
    }

    #[test]
    fn hierarchy_attach_child() {
        let mut sys = make_system();
        let o = owner();
        let portfolio_id = sys.create_item(o, "Main Portfolio", ItemCategory::Portfolio, ItemPayload::Portfolio(
            PortfolioItemPayload { mission: None, focus_areas: vec![], kpis: vec![] }
        )).unwrap();
        let project_id = sys.create_item(o, "Child Project", ItemCategory::Project, ItemPayload::Project(ProjectPayload {
            project_type: ProjectType::Technical, methodology: Methodology::Agile,
            start_date: None, end_date: None, sprints: vec![], backlog: vec![], releases: vec![], completion_pct: 0.0,
        })).unwrap();
        sys.attach_child(o, portfolio_id, project_id).unwrap();
        let children = sys.children_of(portfolio_id).unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id(), project_id);
    }
}
