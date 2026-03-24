// =============================================================================
// hypergrid::plugin — HG-PLUGIN: Extension Interface
//
// HypercubePlugin (base), ComputedModelPlugin, AIEnginePlugin,
// AttributeTypePlugin, EdgeTypePlugin, GovernancePlugin,
// PluginRegistry, ComputeContext, AIComputeRequest, AIComputeQueue
// =============================================================================

use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::cell::{
    ActorId, AttributeKey, AttributeKeyDef, ComputePriority, CubeId,
    DimCoordinate, HyperRow, PluginId, TypedAttrValue,
};
use crate::crdt::{CrdtOperation, MergeResult};
use crate::dim::DimSlice;
use crate::error::{HypergridError, PluginError};
use crate::graph::EdgeType;

// ─── PluginCapability ─────────────────────────────────────────────────────────

/// Declares what kind of extension this plugin provides.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginCapability {
    /// Adds new computed/AI attribute keys to Hypercubes.
    ComputedModel,
    /// Adds custom CRDT merge logic for a new attribute type.
    AttributeType,
    /// Adds a new HypergraphEdge type with custom validation.
    EdgeType,
    /// Adds a new RenderMode for HypercubeViews.
    RenderMode,
    /// Adds a new DataConnector for import/export.
    DataConnector,
    /// Adds a new GovernanceModel for Spaces.
    Governance,
    /// Adds a custom DimensionAxis type.
    AxisType,
}

// ─── PluginHealth ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PluginHealth {
    Healthy,
    Degraded { reason: String },
    Unhealthy { reason: String },
}

// ─── PluginContext ─────────────────────────────────────────────────────────────

/// Context provided to a plugin at load time.
pub struct PluginContext {
    pub grid_id:    crate::cell::GridId,
    pub config:     HashMap<String, serde_json::Value>,
}

// ─── ComputeContext ──────────────────────────────────────────────────────────

/// Everything a ComputedModelPlugin needs to compute one attribute.
pub struct ComputeContext<'a> {
    /// The full HyperRow of the entity being computed for.
    pub row:       &'a HyperRow,
    /// The Hypercube schema (attribute registry, dimensions).
    pub cube_id:   CubeId,
    /// Lazy-loaded Hypergraph neighbor values.
    pub neighbors: NeighborCache<'a>,
}

impl<'a> ComputeContext<'a> {
    pub fn new(row: &'a HyperRow, cube_id: CubeId) -> Self {
        Self { row, cube_id, neighbors: NeighborCache::empty() }
    }
}

/// Lazy cache for graph neighbor values. Loaded on first access.
pub struct NeighborCache<'a> {
    _phantom: std::marker::PhantomData<&'a ()>,
    data:     HashMap<(EdgeType, crate::graph::EdgeDirection), Vec<HyperRow>>,
}

impl<'a> NeighborCache<'a> {
    pub fn empty() -> Self {
        Self { _phantom: Default::default(), data: HashMap::new() }
    }

    pub fn count(&self, edge_type: EdgeType, direction: crate::graph::EdgeDirection) -> usize {
        self.data.get(&(edge_type, direction)).map(|v| v.len()).unwrap_or(0)
    }

    pub fn sum(&self, edge_type: EdgeType, direction: crate::graph::EdgeDirection, attr: &str) -> f64 {
        self.data.get(&(edge_type, direction))
            .map(|rows| rows.iter().filter_map(|r| r.get_f64(attr)).sum())
            .unwrap_or(0.0)
    }
}

// ─── ComputeResult ────────────────────────────────────────────────────────────

pub enum ComputeResult {
    Ok(TypedAttrValue),
    Err(PluginError),
    Unavailable,  // AI engine not ready; stale cache should be served
}

// ─── HypercubePlugin (base trait) ────────────────────────────────────────────

/// Base trait for all Hypergrid plugins. Every plugin implements this plus
/// at least one of the sub-traits (ComputedModelPlugin, AttributeTypePlugin, etc.).
pub trait HypercubePlugin: Send + Sync {
    fn plugin_id(&self)    -> &str;
    fn plugin_name(&self)  -> &str;
    fn version(&self)      -> semver::Version;
    fn capabilities(&self) -> Vec<PluginCapability>;

    // ── Lifecycle ──────────────────────────────────────────────────────────

    /// Called once at Grid startup. Initialize state, load models, warm caches.
    fn on_load(&mut self, ctx: &PluginContext) -> Result<(), PluginError>;

    /// Called at Grid shutdown. Release resources.
    fn on_unload(&mut self) -> Result<(), PluginError>;

    /// Called every 30 seconds by the health monitor.
    fn health_check(&self) -> PluginHealth;

    // ── Cell mutation hook ────────────────────────────────────────────────

    /// Called after any cell in this Hypercube is mutated.
    /// Return the set of AI attribute keys that should be recomputed as a result.
    ///
    /// Example: if "budget_spent" is mutated, return ["health_score", "risk_score"].
    fn on_cell_mutation(&self, cube_id: CubeId, coord: &DimCoordinate) -> Vec<AttributeKey>;

    // ── Schema contribution ───────────────────────────────────────────────

    /// Returns the set of AttributeKeyDefs this plugin contributes to any Hypercube it is attached to.
    /// These are registered in the AttributeKeyRegistry at plugin load time.
    fn attribute_key_defs(&self) -> Vec<AttributeKeyDef> { vec![] }
}

// ─── ComputedModelPlugin ─────────────────────────────────────────────────────

/// Sub-trait for plugins that compute Tier-2 AI/ML attribute values.
/// The computation is async, expensive, and written back via WritebackService.
pub trait ComputedModelPlugin: HypercubePlugin {
    /// Which AttributeKeyDefs does this model produce?
    fn output_attrs(&self) -> Vec<AttributeKeyDef>;

    /// Which EventLog event kinds should trigger recomputation?
    fn invalidation_triggers(&self) -> Vec<EventKind>;

    /// Cache TTL for this model's outputs (before the value is considered stale).
    fn cache_ttl(&self) -> Duration;

    /// Compute the attribute value. Called by the AI compute worker pool.
    fn compute(&self, row: &HyperRow, ctx: &ComputeContext) -> ComputeResult;

    /// Detect anomalies in a DimSlice. Optional — not all models support this.
    fn detect_anomalies(&self, _slice: &DimSlice) -> Vec<crate::cell::TypedAttrValue> { vec![] }
}

// ─── EventKind ───────────────────────────────────────────────────────────────

/// Which kind of event should trigger Tier-2 recomputation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventKind {
    CellMutation  { attr_key: AttributeKey },
    GraphEdgeAdded { edge_type: EdgeType },
    GraphEdgeRemoved { edge_type: EdgeType },
    SchemaChange,
    Custom(String),
}

// ─── AttributeTypePlugin ─────────────────────────────────────────────────────

/// Sub-trait for plugins that register custom CRDT merge logic for a new data type.
pub trait AttributeTypePlugin: HypercubePlugin {
    fn attr_type_id(&self) -> &str;

    /// Custom CRDT merge: given two concurrent values, return the merged result.
    fn merge(
        &self,
        a: &TypedAttrValue,
        b: &TypedAttrValue,
        ctx: &AttributeMergeContext,
    ) -> TypedAttrValue;

    /// Validate a value before writing.
    fn validate(&self, value: &TypedAttrValue) -> Result<(), PluginError>;
}

#[derive(Debug)]
pub struct AttributeMergeContext {
    pub attr_key:  AttributeKey,
    pub cube_id:   CubeId,
    pub actor_a:   ActorId,
    pub actor_b:   ActorId,
}

// ─── GovernancePlugin ─────────────────────────────────────────────────────────

/// Sub-trait for plugins that provide governance models for Spaces.
pub trait GovernancePlugin: HypercubePlugin {
    fn governance_model_id(&self) -> &str;

    /// Has a governance proposal reached quorum?
    fn quorum_reached(&self, votes: &[GovernanceVote], total_members: u32) -> bool;

    /// Compute the distribution of rewards/shares among contributors.
    fn compute_distribution(&self, contributors: &[Contributor]) -> Vec<Distribution>;

    /// Evaluate whether a proposal passes given the votes.
    fn evaluate_proposal(&self, votes: &[GovernanceVote]) -> ProposalOutcome;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceVote {
    pub voter_id:  crate::cell::UserId,
    pub vote:      VoteOption,
    pub weight:    f64,
    pub voted_at:  DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VoteOption { Yes, No, Abstain }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contributor {
    pub user_id:   crate::cell::UserId,
    pub contribution_weight: f64,
    pub stake_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Distribution {
    pub user_id:   crate::cell::UserId,
    pub share_pct: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProposalOutcome { Passed, Rejected, NoQuorum }

// ─── Example plugin: KogiHealthScoreEngine ────────────────────────────────────

/// Reference implementation of ComputedModelPlugin for Kogi's health_score attribute.
pub struct KogiHealthScoreEngine;

impl HypercubePlugin for KogiHealthScoreEngine {
    fn plugin_id(&self)    -> &str { "kogi.health_score_engine.v2" }
    fn plugin_name(&self)  -> &str { "Kogi Health Score Engine v2" }
    fn version(&self)      -> semver::Version { "2.0.0".parse().unwrap() }
    fn capabilities(&self) -> Vec<PluginCapability> { vec![PluginCapability::ComputedModel] }

    fn on_load(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> { Ok(()) }
    fn on_unload(&mut self) -> Result<(), PluginError> { Ok(()) }
    fn health_check(&self) -> PluginHealth { PluginHealth::Healthy }

    fn on_cell_mutation(&self, _cube_id: CubeId, coord: &DimCoordinate) -> Vec<AttributeKey> {
        // Recompute health_score when status, budget_spent, or risks change
        if let Some(key) = coord.d1_str() {
            if ["status", "budget_spent", "risks", "due_date"].contains(&key) {
                return vec!["health_score".into(), "risk_score".into()];
            }
        }
        vec![]
    }

    fn attribute_key_defs(&self) -> Vec<AttributeKeyDef> {
        vec![
            AttributeKeyDef::ai_signal("health_score", "Health Score",  self.plugin_id()),
            AttributeKeyDef::ai_signal("risk_score",   "Risk Score",    self.plugin_id()),
            AttributeKeyDef::ai_signal("anomaly_flag", "Anomaly Flag",  self.plugin_id()),
        ]
    }
}

impl ComputedModelPlugin for KogiHealthScoreEngine {
    fn output_attrs(&self) -> Vec<AttributeKeyDef> {
        self.attribute_key_defs()
    }

    fn invalidation_triggers(&self) -> Vec<EventKind> {
        vec![
            EventKind::CellMutation { attr_key: "status".into() },
            EventKind::CellMutation { attr_key: "budget_spent".into() },
            EventKind::CellMutation { attr_key: "risks".into() },
            EventKind::GraphEdgeAdded { edge_type: EdgeType::Dependency },
        ]
    }

    fn cache_ttl(&self) -> Duration { Duration::from_secs(3600) }

    fn compute(&self, row: &HyperRow, ctx: &ComputeContext) -> ComputeResult {
        let status      = row.get_str("status").unwrap_or("Draft");
        let budget      = row.get_f64("budget").unwrap_or(1.0).max(1.0);
        let budget_spent = row.get_f64("budget_spent").unwrap_or(0.0);
        let budget_util = budget_spent / budget;
        let risk_count  = row.get_json_array_len("risks").unwrap_or(0);
        let dep_count   = ctx.neighbors.count(
            EdgeType::Dependency, crate::graph::EdgeDirection::Inbound
        );

        let health = match status {
            "Active" => (50.0
                + (1.0 - budget_util).max(0.0) * 30.0
                - (risk_count as f64 * 5.0).min(30.0)
                - (dep_count  as f64 * 2.0).min(10.0))
                .clamp(0.0, 100.0),
            "Completed" => 90.0,
            "Archived"  => 70.0,
            "Paused"    => 40.0,
            _           => 20.0,  // Draft, Blocked
        };

        ComputeResult::Ok(TypedAttrValue::AiSignal {
            value:       Box::new(TypedAttrValue::Number(health)),
            model_id:    "kogi.health_score.v2".into(),
            computed_at: Utc::now(),
            confidence:  0.87,
            explanation: Some(format!(
                "status={status}, budget_util={:.0}%, risks={risk_count}, deps={dep_count}",
                budget_util * 100.0
            )),
            stale_at:    None,
        })
    }
}

// ─── AIComputeRequest ────────────────────────────────────────────────────────

/// Work item for the AI compute worker pool.
#[derive(Debug, Clone)]
pub struct AIComputeRequest {
    pub request_id:  Uuid,
    pub plugin_id:   PluginId,
    pub attr_key:    AttributeKey,
    pub cube_id:     CubeId,
    pub d1_key:      crate::cell::DimKey,
    pub priority:    ComputePriority,
    pub requested_at: DateTime<Utc>,
    pub observed_vc: crate::crdt::VectorClock,  // used for stale-check
}

impl PartialEq for AIComputeRequest {
    fn eq(&self, other: &Self) -> bool { self.request_id == other.request_id }
}
impl Eq for AIComputeRequest {}
impl PartialOrd for AIComputeRequest {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(other)) }
}
impl Ord for AIComputeRequest {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority → earlier in BinaryHeap (max-heap)
        let priority_ord = |p: &ComputePriority| match p {
            ComputePriority::Critical   => 3u8,
            ComputePriority::High       => 2,
            ComputePriority::Normal     => 1,
            ComputePriority::Background => 0,
        };
        priority_ord(&self.priority).cmp(&priority_ord(&other.priority))
            .then(other.requested_at.cmp(&self.requested_at)) // older first within same priority
    }
}

// ─── AIComputeQueue ──────────────────────────────────────────────────────────

/// Priority queue for Tier-2 AI compute requests.
/// Workers pop the highest-priority pending request.
pub struct AIComputeQueue {
    heap: Mutex<BinaryHeap<AIComputeRequest>>,
}

impl AIComputeQueue {
    pub fn new() -> Self { Self { heap: Mutex::new(BinaryHeap::new()) } }

    pub fn push(&self, req: AIComputeRequest) {
        self.heap.lock().unwrap().push(req);
    }

    pub fn pop(&self) -> Option<AIComputeRequest> {
        self.heap.lock().unwrap().pop()
    }

    pub fn len(&self) -> usize { self.heap.lock().unwrap().len() }
}

// ─── PluginRegistry ──────────────────────────────────────────────────────────

/// Central registry of all HypercubePlugin instances in a Grid.
/// Provides capability lookup, health monitoring, and hot-swap support.
pub struct PluginRegistry {
    plugins:     RwLock<HashMap<PluginId, Arc<dyn HypercubePlugin>>>,
    /// Map: PluginCapability → [PluginId] for fast capability lookup.
    cap_index:   RwLock<HashMap<PluginCapability, Vec<PluginId>>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins:   Default::default(),
            cap_index: Default::default(),
        }
    }

    /// Register a plugin. Calls on_load() and indexes capabilities.
    pub fn register(
        &self,
        mut plugin: Box<dyn HypercubePlugin>,
        ctx: &PluginContext,
    ) -> Result<(), PluginError> {
        plugin.on_load(ctx)?;
        let id = plugin.plugin_id().to_owned();
        let caps = plugin.capabilities();

        let plugin_arc: Arc<dyn HypercubePlugin> = Arc::from(plugin);
        self.plugins.write().unwrap().insert(id.clone(), plugin_arc);

        let mut idx = self.cap_index.write().unwrap();
        for cap in caps {
            idx.entry(cap).or_default().push(id.clone());
        }
        Ok(())
    }

    pub fn get(&self, plugin_id: &str) -> Option<Arc<dyn HypercubePlugin>> {
        self.plugins.read().unwrap().get(plugin_id).cloned()
    }

    pub fn all_ids(&self) -> Vec<PluginId> {
        self.plugins.read().unwrap().keys().cloned().collect()
    }

    pub fn with_capability(&self, cap: &PluginCapability) -> Vec<Arc<dyn HypercubePlugin>> {
        let ids = self.cap_index.read().unwrap()
            .get(cap).cloned().unwrap_or_default();
        let plugins = self.plugins.read().unwrap();
        ids.iter().filter_map(|id| plugins.get(id).cloned()).collect()
    }

    /// Hot-swap a plugin: atomic pointer replacement with no downtime.
    pub fn swap(
        &self,
        old_id: &str,
        mut new_plugin: Box<dyn HypercubePlugin>,
        ctx: &PluginContext,
    ) -> Result<(), PluginError> {
        new_plugin.on_load(ctx)?;
        let new_id = new_plugin.plugin_id().to_owned();
        let arc: Arc<dyn HypercubePlugin> = Arc::from(new_plugin);
        self.plugins.write().unwrap().insert(new_id, arc);
        if old_id != new_id.as_str() {
            self.plugins.write().unwrap().remove(old_id);
        }
        Ok(())
    }

    /// Run health check on all plugins. Returns list of unhealthy plugin IDs.
    pub fn health_check_all(&self) -> Vec<(PluginId, PluginHealth)> {
        self.plugins.read().unwrap()
            .iter()
            .map(|(id, p)| (id.clone(), p.health_check()))
            .filter(|(_, h)| !matches!(h, PluginHealth::Healthy))
            .collect()
    }

    /// Notify all plugins that a cell was mutated.
    /// Returns deduplicated set of AI attr keys to recompute.
    pub fn on_cell_mutation(&self, cube_id: CubeId, coord: &DimCoordinate) -> Vec<AttributeKey> {
        let mut attrs: std::collections::HashSet<AttributeKey> = Default::default();
        for plugin in self.plugins.read().unwrap().values() {
            for key in plugin.on_cell_mutation(cube_id, coord) {
                attrs.insert(key);
            }
        }
        attrs.into_iter().collect()
    }
}
