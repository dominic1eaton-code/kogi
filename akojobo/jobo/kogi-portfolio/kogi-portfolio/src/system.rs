//! PortfolioSystem — the root orchestrator.
//!
//! ## Architecture
//!
//! ```text
//!  PortfolioSystem
//!   ├── grid: hypergrid::Grid           ← all cell-level storage + CRDT
//!   │    └── components_cube_id         ← D1=ComponentId, D2=field_name
//!   ├── graph: PortfolioGraph           ← wraps hypergrid::Hypergraph
//!   ├── event_log: EventLog             ← wraps hypergrid::core::EventLog
//!   ├── crdt_log: PortfolioCrdtLog      ← wraps hypergrid::crdt::CrdtLog
//!   ├── governance: GovernanceEngine    ← pure portfolio-layer logic
//!   ├── federation: PortfolioFederation ← multi-node CRDT sync
//!   └── plugins: Vec<Arc<dyn PortfolioPlugin>>
//! ```
//!
//! Every Component is stored as a HyperRow in `components_cube`:
//!   - D1 = ComponentId (UUID)
//!   - D2 = field name (String)
//!   - `cell.attributes["value"]` = typed field value
//!
//! All write operations persist to the Hypercube via `ComponentStore::write()`.
//! All read operations reconstruct from HyperCells via `ComponentStore::read()`.
//! The hypergrid EventLog receives every mutation (dual-write with the portfolio
//! domain event log) for full audit continuity and AS_OF time-travel.

use std::collections::{HashSet};
use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;

use hypergrid::cell::{CubeId, DimKey};
use hypergrid::core::Grid;


use crate::component::{
    Component, ComponentPayload, ContainerPayload, ItemPayload,
};
use crate::crdt::{CrdtOperation, NodeId, PortfolioCrdtLog};
use crate::error::{PortfolioError, PortfolioResult};
use crate::events::{EventLog, PortfolioEvent, PortfolioEventKind, Snapshot};
use crate::federation::PortfolioFederation;
use crate::governance::{GovernanceEngine, PolicyContext, PolicyDecision, ResourceAllocation};
use crate::graph::{PortfolioEdge, PortfolioEdgeType, PortfolioGraph};
use crate::metadata::BumpKind;
use crate::models::{PortfolioHealth, PortfolioHealthInput, PortfolioHealthResult,
                    ProjectMetrics, ProjectMetricsInput, ProjectMetricsResult};
use crate::plugin::PortfolioPlugin;
use crate::search::{PortfolioQuery, SearchQuery, SearchResult};
use crate::store::{ComponentStore, COMPONENTS_CUBE_NAME};
use crate::types::*;

// ── PortfolioSystem ───────────────────────────────────────────────────────────

pub struct PortfolioSystem {
    // ── Hypergrid substrate ────────────────────────────────────────────────
    /// The hypergrid Grid — owns the Hypercube(s) and Universal Cell Store.
    pub grid: Grid,
    /// The Hypercube in which all components are stored as HyperRows.
    pub components_cube_id: CubeId,

    // ── Portfolio graph layer ──────────────────────────────────────────────
    /// Wraps hypergrid's Hypergraph with portfolio-domain edge types.
    pub graph: PortfolioGraph,

    // ── Portfolio-layer infrastructure ────────────────────────────────────
    pub event_log:  EventLog,
    pub crdt_log:   PortfolioCrdtLog,
    pub governance: GovernanceEngine,
    pub federation: PortfolioFederation,

    // ── Identity ──────────────────────────────────────────────────────────
    pub node_id: NodeId,

    // ── Plugins ───────────────────────────────────────────────────────────
    plugins: Vec<Arc<dyn PortfolioPlugin>>,
}

impl PortfolioSystem {
    // ── Constructor ────────────────────────────────────────────────────────

    /// Create a new PortfolioSystem. Bootstraps the components Hypercube
    /// in the underlying Hypergrid Grid with the full attribute schema.
    pub fn new(node_id: impl Into<NodeId>) -> Self {
        let node_id: NodeId = node_id.into();
        let mut grid = Grid::new("kogi-portfolio", node_id.clone());

        // Create and bootstrap the components Hypercube (D1×D2, N=2).
        let cube_id = grid.create_cube(COMPONENTS_CUBE_NAME)
            .expect("Failed to create components Hypercube");

        // Register all portfolio component attribute keys.
        {
            let cube = grid.get_cube_mut(cube_id).unwrap();
            ComponentStore::register_schema(cube)
                .expect("Failed to register component schema");
        }

        let grid_id = grid.grid_id;

        Self {
            components_cube_id: cube_id,
            graph: PortfolioGraph::new(grid_id, cube_id),
            event_log: EventLog::new(grid_id),
            crdt_log: PortfolioCrdtLog::new(),
            governance: GovernanceEngine::new(),
            federation: PortfolioFederation::new(),
            node_id,
            plugins: vec![],
            grid,
        }
    }

    // ── Plugin registry ───────────────────────────────────────────────────

    pub fn register_plugin(&mut self, plugin: Arc<dyn PortfolioPlugin>) {
        self.plugins.push(plugin);
    }

    fn emit(&mut self, event: PortfolioEvent) {
        for p in &self.plugins { p.on_event(&event); }
        self.event_log.emit(event);
    }

    // ── Internal helpers ──────────────────────────────────────────────────

    /// Persist a Component to the Hypercube and emit a CRDT SetField op.
    fn persist(&mut self, component: &Component) -> PortfolioResult<()> {
        let actor = self.node_id.clone();
        ComponentStore::write(&mut self.grid, self.components_cube_id, component, &actor)
            .map_err(|e| PortfolioError::StorageError(e.to_string()))?;

        // Append a CRDT op for every persisted component (federation sync).
        self.crdt_log.append(CrdtOperation::SetField {
            component_id: component.id(),
            field: "component".into(),
            value: serde_json::json!(component.id()),
            timestamp: Utc::now(),
            actor: actor.clone(),
        });
        Ok(())
    }

    /// Read one component from the Hypercube.
    pub fn get(&self, id: ComponentId) -> PortfolioResult<Component> {
        ComponentStore::read(&self.grid, self.components_cube_id, id)?
            .ok_or(PortfolioError::NotFound(id))
    }

    /// Check that a component exists without fully deserialising it.
    fn exists(&self, id: ComponentId) -> bool {
        ComponentStore::exists(&self.grid, self.components_cube_id, id)
    }

    /// Read a component, verify a permission tier, then call a mutating closure.
    /// The closure receives the component and the node_id string.
    fn with_component_mut<F, T>(
        &mut self,
        user_id: UserId,
        id: ComponentId,
        required: PermissionTier,
        action_name: &str,
        f: F,
    ) -> PortfolioResult<T>
    where
        F: FnOnce(&mut Component, &str) -> PortfolioResult<T>,
    {
        let mut comp = self.get(id)?;
        if !comp.check_permission(&user_id, required) {
            return Err(PortfolioError::PermissionDenied {
                user_id: user_id.to_string(),
                action: action_name.to_owned(),
            });
        }
        let node = self.node_id.clone();
        let result = f(&mut comp, &node)?;
        self.persist(&comp)?;
        Ok(result)
    }

    // ── Component creation ────────────────────────────────────────────────

    /// Create and persist a new Item component.
    pub fn create_item(
        &mut self,
        owner: UserId,
        name: impl Into<String>,
        item_category: ItemCategory,
        payload: ItemPayload,
    ) -> PortfolioResult<ComponentId> {
        let node = self.node_id.clone();
        let comp = Component::new_item(owner, name, item_category, payload, &node);
        let id = comp.id();

        self.persist(&comp)?;
        self.graph.register_component(id);

        let comp_ref = self.get(id).unwrap(); // just persisted
        for p in &self.plugins { p.on_component_created(&comp_ref); }

        self.emit(PortfolioEvent::new(PortfolioEventKind::ComponentCreated, Some(id), &node));
        Ok(id)
    }

    /// Create and persist a new Container component.
    pub fn create_container(
        &mut self,
        owner: UserId,
        name: impl Into<String>,
        container_category: ContainerCategory,
        payload: ContainerPayload,
    ) -> PortfolioResult<ComponentId> {
        let node = self.node_id.clone();
        let comp = Component::new_container(owner, name, container_category, payload, &node);
        let id = comp.id();

        self.persist(&comp)?;
        self.graph.register_component(id);

        let comp_ref = self.get(id).unwrap();
        for p in &self.plugins { p.on_component_created(&comp_ref); }

        self.emit(PortfolioEvent::new(PortfolioEventKind::ComponentCreated, Some(id), &node));
        Ok(id)
    }

    // ── Component reads ───────────────────────────────────────────────────

    /// Scan all component IDs known to the graph (registered nodes).
    pub fn all_component_ids(&self) -> Vec<ComponentId> {
        // Walk every cell in the cube and collect unique D1 keys.
        self.grid.scan_cube(self.components_cube_id)
            .iter()
            .filter_map(|c| c.coord.d1())
            .filter_map(|k| if let DimKey::Uuid(id) = k { Some(*id) } else { None })
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }

    /// Load all components from the Hypercube.
    pub fn all_components(&self) -> Vec<Component> {
        self.all_component_ids()
            .into_iter()
            .filter_map(|id| self.get(id).ok())
            .collect()
    }

    pub fn component_count(&self) -> usize {
        self.all_component_ids().len()
    }

    // ── Component mutation ────────────────────────────────────────────────

    pub fn update_info(
        &mut self,
        user_id: UserId,
        id: ComponentId,
        name: Option<String>,
        description: Option<String>,
    ) -> PortfolioResult<()> {
        let node = self.node_id.clone();
        self.with_component_mut(user_id, id, PermissionTier::Editor, "update_info",
            |comp, node| {
                comp.update_info(name, description, node);
                Ok(())
            }
        )?;

        let comp = self.get(id)?;
        for p in &self.plugins { p.on_component_updated(&comp); }
        self.emit(PortfolioEvent::new(PortfolioEventKind::ComponentUpdated, Some(id), &node));
        Ok(())
    }

    /// Soft or hard delete a component.
    pub fn delete(
        &mut self,
        user_id: UserId,
        id: ComponentId,
        hard_delete: bool,
    ) -> PortfolioResult<()> {
        let node = self.node_id.clone();
        let mut comp = self.get(id)?;
        if !comp.check_permission(&user_id, PermissionTier::Owner) {
            return Err(PortfolioError::PermissionDenied {
                user_id: user_id.to_string(), action: "delete".into(),
            });
        }

        if hard_delete {
            // Purge all cells from the Hypercube for this component.
            for field in [
                crate::store::fields::NAME, crate::store::fields::DESCRIPTION,
                crate::store::fields::STATUS, crate::store::fields::CATEGORY,
                crate::store::fields::PAYLOAD,
            ] {
                let _ = self.grid.write_cell(
                    self.components_cube_id,
                    hypergrid::cell::DimCoordinate::d2(
                        DimKey::uuid(id), DimKey::str(field)),
                    field,
                    hypergrid::cell::TypedAttrValue::Null,
                    &node,
                );
            }
            self.graph.deregister_component(id);
            for p in &self.plugins { p.on_component_removed(id); }
            self.emit(PortfolioEvent::new(PortfolioEventKind::ComponentRemoved, Some(id), &node));
        } else {
            comp.set_status(ComponentStatus::Deleted, &node);
            self.persist(&comp)?;
            self.emit(PortfolioEvent::new(PortfolioEventKind::ComponentStatusChanged, Some(id), &node));
        }
        Ok(())
    }

    pub fn act(
        &mut self,
        user_id: UserId,
        id: ComponentId,
        action: ActionKind,
    ) -> PortfolioResult<()> {
        let node = self.node_id.clone();
        let required = match &action {
            ActionKind::Edit   => PermissionTier::Editor,
            ActionKind::Delete => PermissionTier::Owner,
            ActionKind::Archive => PermissionTier::Manager,
            ActionKind::Own { .. } => PermissionTier::Owner,
            _ => PermissionTier::Contributor,
        };
        self.with_component_mut(user_id, id, required, "act", |comp, node| {
            comp.apply_action(action, user_id);
            comp.metadata.touch(node);
            Ok(())
        })?;

        let comp = self.get(id)?;
        for p in &self.plugins { p.on_component_updated(&comp); }
        self.emit(PortfolioEvent::new(PortfolioEventKind::ActionPerformed, Some(id), &node));
        Ok(())
    }

    pub fn bump_version(
        &mut self,
        user_id: UserId,
        id: ComponentId,
        kind: BumpKind,
        summary: Option<String>,
    ) -> PortfolioResult<String> {
        let _node = self.node_id.clone();
        self.with_component_mut(user_id, id, PermissionTier::Editor, "bump_version",
            |comp, node| {
                comp.bump_version(kind, node, summary);
                Ok(comp.metadata.version.clone())
            }
        )
    }

    // ── User management ───────────────────────────────────────────────────

    pub fn add_user(
        &mut self,
        actor: UserId,
        id: ComponentId,
        user_id: UserId,
        tier: PermissionTier,
    ) -> PortfolioResult<()> {
        let node = self.node_id.clone();
        self.with_component_mut(actor, id, PermissionTier::Manager, "add_user",
            |comp, node| {
                comp.data.users.add_user(user_id, tier);
                comp.metadata.touch(node);
                Ok(())
            }
        )?;
        self.emit(PortfolioEvent::new(PortfolioEventKind::UserAdded, Some(id), &node));
        Ok(())
    }

    pub fn remove_user(
        &mut self,
        actor: UserId,
        id: ComponentId,
        user_id: UserId,
    ) -> PortfolioResult<()> {
        let node = self.node_id.clone();
        self.with_component_mut(actor, id, PermissionTier::Manager, "remove_user",
            |comp, node| {
                comp.data.users.remove_user(&user_id);
                comp.metadata.touch(node);
                Ok(())
            }
        )?;
        self.emit(PortfolioEvent::new(PortfolioEventKind::UserRemoved, Some(id), &node));
        Ok(())
    }

    // ── Graph (edges — delegate to PortfolioGraph + Hypergraph) ──────────

    pub fn attach_child(
        &mut self,
        user_id: UserId,
        parent_id: ComponentId,
        child_id: ComponentId,
    ) -> PortfolioResult<()> {
        if !self.exists(parent_id) { return Err(PortfolioError::NotFound(parent_id)); }
        if !self.exists(child_id)  { return Err(PortfolioError::NotFound(child_id));  }

        let node = self.node_id.clone();
        let edge = PortfolioEdge::hierarchy(parent_id, child_id, &node);

        // Permission check
        {
            let parent = self.get(parent_id)?;
            if !parent.check_permission(&user_id, PermissionTier::Manager) {
                return Err(PortfolioError::PermissionDenied {
                    user_id: user_id.to_string(), action: "attach_child".into(),
                });
            }
        }

        // Update denormalised children/parents lists in cells
        self.with_component_mut(user_id, parent_id, PermissionTier::Manager, "attach_child",
            |comp, node| { comp.data.children.push(child_id); comp.metadata.touch(node); Ok(()) }
        )?;
        // child's parents list — no permission check needed (system operation)
        let mut child = self.get(child_id)?;
        child.data.parents.push(parent_id);
        child.metadata.touch(&node);
        self.persist(&child)?;

        // Add edge to Hypergraph via PortfolioGraph
        let edge_clone = edge.clone();
        self.graph.add_edge(edge_clone)
            .map_err(|e| PortfolioError::InvalidOperation(e))?;

        for p in &self.plugins { p.on_edge_added(&edge); }
        self.crdt_log.append(CrdtOperation::AddEdge {
            edge: edge.clone(), timestamp: Utc::now(), actor: node.clone(),
        });
        self.emit(PortfolioEvent::new(PortfolioEventKind::EdgeAdded, Some(parent_id), &node)
            .with_payload(serde_json::json!({ "child_id": child_id })));
        Ok(())
    }

    pub fn detach_child(
        &mut self,
        user_id: UserId,
        parent_id: ComponentId,
        child_id: ComponentId,
    ) -> PortfolioResult<()> {
        let node = self.node_id.clone();
        self.with_component_mut(user_id, parent_id, PermissionTier::Manager, "detach_child",
            |comp, node| { comp.data.children.retain(|&c| c != child_id); comp.metadata.touch(node); Ok(()) }
        )?;

        // Remove matching Hierarchy edge from graph
        let edge_id = self.graph.edges_for(parent_id)
            .iter()
            .find(|e| e.target_id == child_id && e.edge_type == PortfolioEdgeType::Hierarchy)
            .map(|e| e.edge_id);
        if let Some(eid) = edge_id {
            self.graph.remove_edge(eid);
            for p in &self.plugins { p.on_edge_removed(eid); }
        }
        self.emit(PortfolioEvent::new(PortfolioEventKind::EdgeRemoved, Some(parent_id), &node));
        Ok(())
    }

    pub fn add_dependency(
        &mut self,
        user_id: UserId,
        from_id: ComponentId,
        to_id: ComponentId,
    ) -> PortfolioResult<()> {
        if !self.exists(from_id) { return Err(PortfolioError::NotFound(from_id)); }
        if !self.exists(to_id)   { return Err(PortfolioError::NotFound(to_id));   }

        let node = self.node_id.clone();
        let edge = PortfolioEdge::dependency(from_id, to_id, &node);

        // Permission check
        {
            let from = self.get(from_id)?;
            if !from.check_permission(&user_id, PermissionTier::Editor) {
                return Err(PortfolioError::PermissionDenied {
                    user_id: user_id.to_string(), action: "add_dependency".into(),
                });
            }
        }

        // Add to graph (cycle detection inside PortfolioGraph::add_edge)
        self.graph.add_edge(edge.clone())
            .map_err(|_msg| PortfolioError::CyclicDependency(from_id, to_id))?;

        // Update denormalised dependency/dependent lists
        let mut from_comp = self.get(from_id)?;
        from_comp.data.dependencies.push(to_id);
        from_comp.metadata.touch(&node);
        self.persist(&from_comp)?;

        let mut to_comp = self.get(to_id)?;
        to_comp.data.dependents.push(from_id);
        to_comp.metadata.touch(&node);
        self.persist(&to_comp)?;

        for p in &self.plugins { p.on_edge_added(&edge); }
        self.crdt_log.append(CrdtOperation::AddEdge {
            edge: edge.clone(), timestamp: Utc::now(), actor: node.clone(),
        });
        self.emit(PortfolioEvent::new(PortfolioEventKind::EdgeAdded, Some(from_id), &node));
        Ok(())
    }

    pub fn remove_dependency(
        &mut self,
        user_id: UserId,
        from_id: ComponentId,
        to_id: ComponentId,
    ) -> PortfolioResult<()> {
        let node = self.node_id.clone();
        self.with_component_mut(user_id, from_id, PermissionTier::Editor, "remove_dependency",
            |comp, node| {
                comp.data.dependencies.retain(|&d| d != to_id);
                comp.metadata.touch(node);
                Ok(())
            }
        )?;
        let edge_id = self.graph.edges_for(from_id)
            .iter()
            .find(|e| e.target_id == to_id && e.edge_type == PortfolioEdgeType::Dependency)
            .map(|e| e.edge_id);
        if let Some(eid) = edge_id { self.graph.remove_edge(eid); }
        self.emit(PortfolioEvent::new(PortfolioEventKind::EdgeRemoved, Some(from_id), &node));
        Ok(())
    }

    pub fn add_link(&mut self, user_id: UserId, a: ComponentId, b: ComponentId) -> PortfolioResult<()> {
        if !self.exists(a) { return Err(PortfolioError::NotFound(a)); }
        if !self.exists(b) { return Err(PortfolioError::NotFound(b)); }
        let node = self.node_id.clone();
        let edge = PortfolioEdge::link(a, b, &node);

        {
            let comp_a = self.get(a)?;
            if !comp_a.check_permission(&user_id, PermissionTier::Editor) {
                return Err(PortfolioError::PermissionDenied {
                    user_id: user_id.to_string(), action: "add_link".into(),
                });
            }
        }
        self.with_component_mut(user_id, a, PermissionTier::Editor, "add_link",
            |comp, node| { comp.data.links.push(b); comp.metadata.touch(node); Ok(()) }
        )?;
        self.graph.add_edge(edge.clone()).map_err(|e| PortfolioError::InvalidOperation(e))?;
        self.emit(PortfolioEvent::new(PortfolioEventKind::EdgeAdded, Some(a), &node));
        Ok(())
    }

    pub fn edges_for(&self, id: ComponentId) -> Vec<PortfolioEdge> {
        self.graph.edges_for(id).into_iter().cloned().collect()
    }

    pub fn children_of(&self, id: ComponentId) -> PortfolioResult<Vec<Component>> {
        if !self.exists(id) { return Err(PortfolioError::NotFound(id)); }
        Ok(self.graph.children_of(id).into_iter()
            .filter_map(|cid| self.get(cid).ok())
            .collect())
    }

    pub fn ancestors_of(&self, id: ComponentId) -> PortfolioResult<Vec<Component>> {
        if !self.exists(id) { return Err(PortfolioError::NotFound(id)); }
        Ok(self.graph.parents_of(id).into_iter()
            .filter_map(|cid| self.get(cid).ok())
            .collect())
    }

    pub fn detect_cycle(&self) -> Option<Vec<ComponentId>> {
        self.graph.detect_cycle()
    }

    // ── Policy & Governance ───────────────────────────────────────────────

    pub fn attach_policy(&mut self, id: ComponentId, policy_id: PolicyId) -> PortfolioResult<()> {
        let node = self.node_id.clone();
        self.with_component_mut(Uuid::nil(), id, PermissionTier::Viewer, "attach_policy",
            |comp, node| {
                if !comp.metadata.has_policy(policy_id) {
                    comp.metadata.policy_ids.push(policy_id);
                }
                comp.metadata.touch(node);
                Ok(())
            }
        )?;
        self.emit(PortfolioEvent::new(PortfolioEventKind::PolicyAttached, Some(id), &node));
        Ok(())
    }

    pub fn detach_policy(&mut self, id: ComponentId, policy_id: PolicyId) -> PortfolioResult<()> {
        let node = self.node_id.clone();
        self.with_component_mut(Uuid::nil(), id, PermissionTier::Viewer, "detach_policy",
            |comp, node| {
                comp.metadata.policy_ids.retain(|&p| p != policy_id);
                comp.metadata.touch(node);
                Ok(())
            }
        )?;
        self.emit(PortfolioEvent::new(PortfolioEventKind::PolicyDetached, Some(id), &node));
        Ok(())
    }

    pub fn evaluate_policy(
        &self,
        id: ComponentId,
        event_kind: &PortfolioEventKind,
        actor: &str,
    ) -> PortfolioResult<PolicyDecision> {
        let comp = self.get(id)?;
        let ctx = PolicyContext::new(&comp, event_kind, actor);
        Ok(self.governance.evaluate(&ctx))
    }

    pub fn request_approval(&mut self, id: ComponentId, reason: &str) -> PortfolioResult<Uuid> {
        if !self.exists(id) { return Err(PortfolioError::NotFound(id)); }
        let node = self.node_id.clone();
        let req_id = self.governance.request_approval(id, reason, &node);
        self.emit(PortfolioEvent::new(PortfolioEventKind::ApprovalRequested, Some(id), &node));
        Ok(req_id)
    }

    pub fn resolve_approval(
        &mut self,
        request_id: Uuid,
        approved: bool,
        resolver: &str,
        notes: Option<String>,
    ) -> PortfolioResult<()> {
        if !self.governance.resolve_approval(request_id, approved, resolver, notes) {
            return Err(PortfolioError::NotFound(request_id));
        }
        let node = self.node_id.clone();
        let kind = if approved { PortfolioEventKind::ApprovalGranted } else { PortfolioEventKind::ApprovalRejected };
        self.emit(PortfolioEvent::new(kind, None, &node));
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
        if !self.exists(id) { return Err(PortfolioError::NotFound(id)); }
        let alloc_id = self.governance.allocate_budget(id, total, currency, period);
        let node = self.node_id.clone();
        // Update budget field in the Hypercube
        self.with_component_mut(Uuid::nil(), id, PermissionTier::Viewer, "allocate_budget",
            |comp, node| { comp.metadata.budget = total; comp.metadata.touch(node); Ok(()) }
        )?;
        self.emit(PortfolioEvent::new(PortfolioEventKind::ResourceAllocated, Some(id), &node));
        Ok(alloc_id)
    }

    pub fn record_consumption(&mut self, id: ComponentId, amount: f64) -> PortfolioResult<f64> {
        if !self.exists(id) { return Err(PortfolioError::NotFound(id)); }
        let remaining = self.governance.record_consumption(id, amount)
            .ok_or_else(|| PortfolioError::InvalidOperation(format!("No allocation for {id}")))?;
        let node = self.node_id.clone();
        self.with_component_mut(Uuid::nil(), id, PermissionTier::Viewer, "record_consumption",
            |comp, node| { comp.metadata.budget_spent += amount; comp.metadata.touch(node); Ok(()) }
        )?;
        self.emit(PortfolioEvent::new(PortfolioEventKind::ResourceConsumed, Some(id), &node));
        Ok(remaining)
    }

    pub fn get_resource_allocation(&self, id: ComponentId) -> PortfolioResult<Option<&ResourceAllocation>> {
        if !self.exists(id) { return Err(PortfolioError::NotFound(id)); }
        Ok(self.governance.get_allocation(id))
    }

    pub fn overrun_allocations(&self) -> Vec<&ResourceAllocation> {
        self.governance.overrun_allocations()
    }

    // ── ToolBox (TMS v2.1) ────────────────────────────────────────────────

    pub fn attach_toolbox(&mut self, user_id: UserId, id: ComponentId, tb: ToolBoxId) -> PortfolioResult<()> {
        let node = self.node_id.clone();
        self.with_component_mut(user_id, id, PermissionTier::Owner, "attach_toolbox",
            |comp, node| { comp.attach_toolbox(tb); comp.metadata.touch(node); Ok(()) }
        )?;
        self.crdt_log.append(CrdtOperation::AddToSet {
            component_id: id, set_name: "toolbox_ids".into(),
            value: serde_json::json!(tb), tag: Uuid::new_v4(),
            timestamp: Utc::now(), actor: node.clone(),
        });
        self.emit(PortfolioEvent::new(PortfolioEventKind::ToolboxAttached, Some(id), &node));
        Ok(())
    }

    pub fn detach_toolbox(&mut self, user_id: UserId, id: ComponentId, tb: ToolBoxId) -> PortfolioResult<()> {
        let node = self.node_id.clone();
        self.with_component_mut(user_id, id, PermissionTier::Owner, "detach_toolbox",
            |comp, node| { comp.detach_toolbox(tb); comp.metadata.touch(node); Ok(()) }
        )?;
        self.emit(PortfolioEvent::new(PortfolioEventKind::ToolboxDetached, Some(id), &node));
        Ok(())
    }

    pub fn has_toolbox(&self, id: ComponentId, tb: ToolBoxId) -> PortfolioResult<bool> {
        Ok(self.get(id)?.has_toolbox(tb))
    }

    pub fn toolbox_ids(&self, id: ComponentId) -> PortfolioResult<Vec<ToolBoxId>> {
        Ok(self.get(id)?.data.toolbox_ids.clone())
    }

    pub fn all_toolbox_ids(&self) -> HashSet<ToolBoxId> {
        self.all_components()
            .iter()
            .flat_map(|c| c.data.toolbox_ids.iter().copied())
            .collect()
    }

    // ── CRDT / Federation ─────────────────────────────────────────────────

    /// Merge a remote PortfolioCrdtLog into this system (federation sync).
    pub fn apply_crdt_log(&mut self, remote: &PortfolioCrdtLog) -> PortfolioResult<usize> {
        let ops = remote.delta_since(&self.node_id).to_vec();
        let n = ops.len();
        for op in &ops {
            self.apply_crdt_op(op.clone())?;
        }
        for p in &self.plugins { p.on_crdt_merge(&ops); }
        let node = self.node_id.clone();
        self.emit(PortfolioEvent::new(PortfolioEventKind::CrdtMergeApplied, None, &node)
            .with_payload(serde_json::json!({ "ops_applied": n })));
        Ok(n)
    }

    fn apply_crdt_op(&mut self, op: CrdtOperation) -> PortfolioResult<()> {
        match op {
            CrdtOperation::SetField { component_id, field, value, timestamp, actor } => {
                if let Ok(mut comp) = self.get(component_id) {
                    let accept = comp.metadata.updated_at < timestamp
                        || (comp.metadata.updated_at == timestamp && actor > comp.metadata.last_actor);
                    if accept {
                        match field.as_str() {
                            "name"        => { if let Some(s) = value.as_str() { comp.data.name = s.to_owned(); } }
                            "description" => { if let Some(s) = value.as_str() { comp.data.description = s.to_owned(); } }
                            "status"      => {} // status lattice: skip LWW per §17.3
                            k => { comp.metadata.properties.insert(k.to_owned(), value); }
                        }
                        comp.metadata.last_actor = actor;
                        comp.metadata.updated_at = timestamp;
                        self.persist(&comp)?;
                    }
                }
            }
            CrdtOperation::AddEdge { edge, .. } => {
                let exists = self.graph.edges_for(edge.source_id)
                    .iter()
                    .any(|e| e.edge_id == edge.edge_id);
                if !exists {
                    self.graph.register_component(edge.source_id);
                    self.graph.register_component(edge.target_id);
                    let _ = self.graph.add_edge(edge);
                }
            }
            CrdtOperation::RemoveEdge { edge_id, .. } => {
                self.graph.remove_edge(edge_id);
            }
            CrdtOperation::AddToSet { component_id, set_name, value, .. } => {
                if let Ok(mut comp) = self.get(component_id) {
                    match set_name.as_str() {
                        "tags"     => { if let Some(s) = value.as_str() { comp.metadata.tags.insert(s.to_owned()); } }
                        "hashtags" => { if let Some(s) = value.as_str() { comp.data.hashtags.insert(s.to_owned()); } }
                        "topics"   => { if let Some(s) = value.as_str() { comp.data.topics.insert(s.to_owned()); } }
                        _          => {}
                    }
                    self.persist(&comp)?;
                }
            }
            CrdtOperation::RemoveFromSet { .. } => {
                // OR-Set remove by tag — production would track tag→value mapping.
            }
        }
        Ok(())
    }

    // ── HyperQL pass-through ──────────────────────────────────────────────

    /// Execute a raw HyperQL query on the underlying Grid. Gives full N-dim
    /// query power to advanced callers (analytics, dashboards, AI engine).
    pub fn hyperql(&self, _query: hypergrid::query::HyperQuery)
        -> Vec<hypergrid::query::QueryRow>
    {
        // In a full implementation this calls the HyperQL evaluation engine.
        // For now it returns an empty result set — integration point for v1.1.
        vec![]
    }

    // ── Snapshot / Time-Travel ─────────────────────────────────────────────

    pub fn snapshot(&self) -> serde_json::Value {
        serde_json::json!({
            "component_count": self.component_count(),
            "edge_count": self.graph.edge_count(),
            "event_count": self.event_log.len(),
            "node_id": self.node_id,
            "snapshot_at": Utc::now(),
            // Hypergrid cell-level stats
            "hg_total_cells": self.grid.stats().total_cells,
            "hg_event_count": self.grid.stats().event_count,
        })
    }

    pub fn save_snapshot(&mut self, label: impl Into<String>) -> Uuid {
        let node = self.node_id.clone();
        let snap = Snapshot::new(label, None, &node, false);
        let id = snap.snapshot_id;
        for p in &self.plugins { p.on_snapshot_saved(&snap); }
        self.event_log.save_snapshot(snap);
        self.emit(PortfolioEvent::new(PortfolioEventKind::SnapshotSaved, None, &node));
        id
    }

    pub fn save_checkpoint(&mut self, label: impl Into<String>, note: Option<String>) -> Uuid {
        let node = self.node_id.clone();
        let snap = Snapshot::new(label, note, &node, true);
        let id = snap.snapshot_id;
        for p in &self.plugins { p.on_snapshot_saved(&snap); }
        self.event_log.save_snapshot(snap);
        self.emit(PortfolioEvent::new(PortfolioEventKind::CheckpointCreated, None, &node));
        id
    }

    // ── Search (delegates to Hypercube scan + in-process filter) ─────────

    pub fn search(&self, query: &SearchQuery) -> Vec<SearchResult> {
        self.all_components()
            .into_iter()
            .filter(|c| {
                if let Some(ref text) = query.text {
                    let q = text.to_lowercase();
                    if !c.data.name.to_lowercase().contains(&q)
                        && !c.data.description.to_lowercase().contains(&q) { return false; }
                }
                if !query.tags.is_empty() {
                    if !query.tags.iter().any(|t| c.metadata.tags.contains(t)) { return false; }
                }
                if !query.statuses.is_empty() && !query.statuses.contains(&c.data.status) { return false; }
                if !query.categories.is_empty() && !query.categories.contains(&c.data.category) { return false; }
                if let Some(owner) = query.owner {
                    if !c.metadata.owners.contains(&owner) { return false; }
                }
                if let Some(ref vis) = query.visibility {
                    if &c.data.visibility != vis { return false; }
                }
                if let Some(after) = query.created_after {
                    if c.metadata.created_at < after { return false; }
                }
                if let Some(before) = query.created_before {
                    if c.metadata.created_at > before { return false; }
                }
                true
            })
            .skip(query.offset)
            .take(query.limit)
            .map(|c| SearchResult {
                component_id: c.id(),
                name: c.data.name.clone(),
                category: c.data.category.clone(),
                status: c.data.status.clone(),
                score: 1.0,
                matched_fields: vec!["name".into()],
            })
            .collect()
    }

    pub fn query(&self, pql: &PortfolioQuery) -> Vec<Component> {
        self.all_components()
            .into_iter()
            .filter(|c| {
                if !pql.categories.is_empty() && !pql.categories.contains(&c.data.category) { return false; }
                if !pql.statuses.is_empty() && !pql.statuses.contains(&c.data.status) { return false; }
                if !pql.tags.is_empty() {
                    if !pql.tags.iter().any(|t| c.metadata.tags.contains(t)) { return false; }
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

    pub fn compute_health(&self) -> PortfolioHealthResult {
        let all = self.all_components();
        let total = all.len();
        let active = all.iter().filter(|c| c.data.status == ComponentStatus::Active).count();
        let (tb, ts) = all.iter().fold((0.0, 0.0), |(b, s), c| (b + c.metadata.budget, s + c.metadata.budget_spent));
        let avg_risk = {
            let scores: Vec<f64> = all.iter()
                .flat_map(|c| c.data.risks.iter().map(|r| r.severity.score()))
                .collect();
            if scores.is_empty() { 1.0 } else { scores.iter().sum::<f64>() / scores.len() as f64 }
        };
        PortfolioHealth::compute(&PortfolioHealthInput {
            total_components: total,
            active_components: active,
            budget_allocated: tb,
            budget_consumed: ts,
            avg_risk_severity: avg_risk,
        })
    }

    pub fn compute_project_metrics(&self, id: ComponentId) -> PortfolioResult<ProjectMetricsResult> {
        let comp = self.get(id)?;
        let payload = match &comp.payload {
            ComponentPayload::Item(ItemPayload::Project(p)) => p,
            _ => return Err(PortfolioError::InvalidOperation(format!("{id} is not a Project"))),
        };
        let total = payload.backlog.len();
        let completed = payload.backlog.iter().filter(|b| b.status == ComponentStatus::Completed).count();
        let sp_done: u32 = payload.backlog.iter().filter(|b| b.status == ComponentStatus::Completed).map(|b| b.story_points).sum();
        let sp_total: u32 = payload.backlog.iter().map(|b| b.story_points).sum();
        let velocity = if payload.sprints.is_empty() { 0.0 } else {
            payload.sprints.iter().map(|s| s.velocity).sum::<f64>() / payload.sprints.len() as f64
        };
        Ok(ProjectMetrics::compute(&ProjectMetricsInput {
            total_tasks: total, completed_tasks: completed, overdue_tasks: 0,
            backlog_size: total.saturating_sub(completed), sprint_velocity: velocity,
            story_points_completed: sp_done, story_points_total: sp_total,
        }))
    }

    // ── Stats ─────────────────────────────────────────────────────────────

    pub fn stats(&self) -> SystemStats {
        let hg_stats = self.grid.stats();
        SystemStats {
            component_count: self.component_count(),
            edge_count: self.graph.edge_count(),
            event_count: self.event_log.len(),
            hg_event_count: hg_stats.event_count,   // hypergrid-level events
            hg_total_cells: hg_stats.total_cells,    // raw HyperCells in Hypercube
            crdt_op_count: self.crdt_log.len(),
            pending_approvals: self.governance.pending_approvals().len(),
            federation_peers: self.federation.peers.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemStats {
    pub component_count: usize,
    pub edge_count:      usize,
    pub event_count:     usize,
    pub hg_event_count:  usize,     // hypergrid cell-level events
    pub hg_total_cells:  usize,     // HyperCells in components Hypercube
    pub crdt_op_count:   usize,
    pub pending_approvals: usize,
    pub federation_peers:  usize,
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::{PortfolioItemPayload, ProjectPayload};

    fn sys() -> PortfolioSystem { PortfolioSystem::new("node-1") }
    fn uid() -> UserId { Uuid::new_v4() }

    fn project_payload() -> ItemPayload {
        ItemPayload::Project(ProjectPayload {
            project_type: ProjectType::Technical,
            methodology: Methodology::Agile,
            start_date: None, end_date: None,
            sprints: vec![], backlog: vec![], releases: vec![],
            completion_pct: 0.0,
        })
    }

    fn portfolio_payload() -> ItemPayload {
        ItemPayload::Portfolio(PortfolioItemPayload {
            mission: Some("Enable independent work".into()),
            focus_areas: vec![], kpis: vec![],
        })
    }

    // ── Round-trip: persist and read back ─────────────────────────────────

    #[test]
    fn create_and_read_from_hypercube() {
        let mut s = sys();
        let o = uid();
        let id = s.create_item(o, "Website Redesign", ItemCategory::Project, project_payload()).unwrap();

        // The component lives in the Hypercube — verify cells were written.
        let stats = s.stats();
        assert!(stats.hg_total_cells > 0, "Hypercube should contain cells");

        // Read back and verify fields.
        let comp = s.get(id).unwrap();
        assert_eq!(comp.name(), "Website Redesign");
        assert_eq!(comp.status(), &ComponentStatus::Draft);
        assert!(comp.check_permission(&o, PermissionTier::Owner));
    }

    #[test]
    fn hypergrid_event_log_receives_events() {
        let mut s = sys();
        let o = uid();
        let id = s.create_item(o, "My Portfolio", ItemCategory::Portfolio, portfolio_payload()).unwrap();
        s.update_info(o, id, Some("Updated Portfolio".into()), None).unwrap();

        // Both the portfolio event log and hypergrid's event log should have entries.
        assert!(s.event_log.len() >= 2);
        assert!(s.event_log.hg_log.len() >= 2, "hypergrid EventLog should receive dual-writes");
    }

    #[test]
    fn update_info_persists_to_hypercube() {
        let mut s = sys();
        let o = uid();
        let id = s.create_item(o, "Alpha", ItemCategory::Project, project_payload()).unwrap();
        s.update_info(o, id, Some("Alpha v2".into()), Some("Updated".into())).unwrap();

        let comp = s.get(id).unwrap();
        assert_eq!(comp.name(), "Alpha v2");
        assert_eq!(comp.data.description, "Updated");
    }

    #[test]
    fn permission_denied_for_stranger() {
        let mut s = sys();
        let o = uid();
        let stranger = uid();
        let id = s.create_item(o, "Private Project", ItemCategory::Project, project_payload()).unwrap();
        let result = s.update_info(stranger, id, Some("Hacked".into()), None);
        assert!(matches!(result, Err(PortfolioError::PermissionDenied { .. })));
    }

    #[test]
    fn hierarchy_attach_uses_hypergraph() {
        let mut s = sys();
        let o = uid();
        let portfolio_id = s.create_item(o, "Portfolio", ItemCategory::Portfolio, portfolio_payload()).unwrap();
        let project_id   = s.create_item(o, "Project A", ItemCategory::Project, project_payload()).unwrap();

        s.attach_child(o, portfolio_id, project_id).unwrap();

        // PortfolioGraph node count should include both components.
        assert_eq!(s.graph.node_count(), 2);
        assert_eq!(s.graph.edge_count(), 1);

        let children = s.children_of(portfolio_id).unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id(), project_id);

        // The HyperRow for portfolio should have updated children field.
        let parent = s.get(portfolio_id).unwrap();
        assert!(parent.data.children.contains(&project_id));
    }

    #[test]
    fn dependency_cycle_detected_by_portfolio_graph() {
        let mut s = sys();
        let o = uid();
        let a = s.create_item(o, "A", ItemCategory::Project, project_payload()).unwrap();
        let b = s.create_item(o, "B", ItemCategory::Project, project_payload()).unwrap();
        s.add_dependency(o, a, b).unwrap();
        let result = s.add_dependency(o, b, a);
        assert!(matches!(result, Err(PortfolioError::CyclicDependency(_, _))));
    }

    #[test]
    fn version_bump_persists_to_hypercube() {
        let mut s = sys();
        let o = uid();
        let id = s.create_item(o, "Asset", ItemCategory::Asset,
            ItemPayload::Asset(crate::component::AssetPayload {
                asset_type: AssetType::Digital, valuation: 5000.0, currency: "USD".into(),
                acquired_at: Utc::now(), depreciation_rate_annual: 0.1,
                cost_basis: 5000.0, vesting_schedule: None,
            })
        ).unwrap();
        let v = s.bump_version(o, id, BumpKind::Minor, Some("branding".into())).unwrap();
        assert_eq!(v, "0.2.0");

        let comp = s.get(id).unwrap();
        assert_eq!(comp.metadata.version, "0.2.0");
        assert_eq!(comp.metadata.version_history.len(), 1);
    }

    #[test]
    fn budget_allocation_reflected_in_hypercube() {
        let mut s = sys();
        let o = uid();
        let id = s.create_item(o, "Project X", ItemCategory::Project, project_payload()).unwrap();
        s.allocate_budget(id, 10_000.0, "USD", None).unwrap();
        let remaining = s.record_consumption(id, 3_000.0).unwrap();
        assert!((remaining - 7_000.0).abs() < 0.01);

        let comp = s.get(id).unwrap();
        assert!((comp.metadata.budget_spent - 3_000.0).abs() < 0.01);
    }

    #[test]
    fn toolbox_attach_persists_and_lists() {
        let mut s = sys();
        let o = uid();
        let id = s.create_item(o, "Portfolio", ItemCategory::Portfolio, portfolio_payload()).unwrap();
        let tb1 = Uuid::new_v4();
        let tb2 = Uuid::new_v4();
        s.attach_toolbox(o, id, tb1).unwrap();
        s.attach_toolbox(o, id, tb2).unwrap();

        let ids = s.toolbox_ids(id).unwrap();
        assert_eq!(ids.len(), 2);

        s.detach_toolbox(o, id, tb1).unwrap();
        assert_eq!(s.toolbox_ids(id).unwrap().len(), 1);
    }

    #[test]
    fn search_filters_from_hypercube() {
        let mut s = sys();
        let o = uid();
        s.create_item(o, "Alpha Project", ItemCategory::Project, project_payload()).unwrap();
        s.create_item(o, "Beta Asset", ItemCategory::Asset,
            ItemPayload::Asset(crate::component::AssetPayload {
                asset_type: AssetType::Digital, valuation: 0.0, currency: "USD".into(),
                acquired_at: Utc::now(), depreciation_rate_annual: 0.0,
                cost_basis: 0.0, vesting_schedule: None,
            })
        ).unwrap();
        let q = SearchQuery::new().text("Alpha");
        let results = s.search(&q);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Alpha Project");
    }

    #[test]
    fn stats_expose_hypergrid_metrics() {
        let mut s = sys();
        let o = uid();
        s.create_item(o, "P1", ItemCategory::Project, project_payload()).unwrap();
        s.create_item(o, "P2", ItemCategory::Project, project_payload()).unwrap();

        let stats = s.stats();
        assert_eq!(stats.component_count, 2);
        // Each component writes ~27 fields as HyperCells
        assert!(stats.hg_total_cells >= 2, "hg_total_cells={}", stats.hg_total_cells);
    }

    #[test]
    fn approval_workflow() {
        let mut s = sys();
        let o = uid();
        let id = s.create_item(o, "Regulated Asset", ItemCategory::Asset,
            ItemPayload::Asset(crate::component::AssetPayload {
                asset_type: AssetType::Financial, valuation: 100_000.0, currency: "USD".into(),
                acquired_at: Utc::now(), depreciation_rate_annual: 0.0,
                cost_basis: 100_000.0, vesting_schedule: None,
            })
        ).unwrap();
        let req_id = s.request_approval(id, "Budget threshold exceeded").unwrap();
        assert_eq!(s.governance.pending_approvals().len(), 1);
        s.resolve_approval(req_id, true, "admin", Some("Approved by CFO".into())).unwrap();
        assert_eq!(s.governance.pending_approvals().len(), 0);
    }

    #[test]
    fn health_score_from_hypercube_data() {
        let mut s = sys();
        let o = uid();
        for i in 0..5 {
            let id = s.create_item(o, format!("P{i}"), ItemCategory::Project, project_payload()).unwrap();
            if i < 4 {
                // Activate via with_component_mut-style — use act()
                // We read back and force status change directly for this test
                let mut c = s.get(id).unwrap();
                c.set_status(ComponentStatus::Active, "node-1");
                s.persist(&c).unwrap();
            }
        }
        let h = s.compute_health();
        assert!(h.health_score > 0.0 && h.health_score <= 100.0);
        assert!(h.active_ratio > 0.5);
    }
}
