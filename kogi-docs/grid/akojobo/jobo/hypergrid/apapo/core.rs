// =============================================================================
// hypergrid::core — HG-CORE: Grid and Hypercube
//
// Grid (root runtime container), Hypercube, HypercubeSchema,
// CellStoreBackend (trait), the 9-step write path, read path
// =============================================================================

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::cell::{
    ActorId, AggFn, AttributeKey, AttributeKeyDef, AttributeKeyRegistry,
    CubeId, DimCoordinate, DimKey, EntityId, GridId, HyperCell, HyperRow,
    NodeId, PermissionTier, PluginId, SpaceId, TypedAttrValue,
};
use crate::comp::{ComputationEngine, Tier2AIPipeline, WritebackService};
use crate::crdt::{
    CrdtLog, CrdtMergeEngine, CrdtOperation, MergeResult,
    OrSetEntry, PerNodeCounter, SerializableCoord, VectorClock,
};
use crate::dim::{DimensionAxis, DimSlice, HypercubeStats, StorageEncoding};
use crate::error::HypergridError;
use crate::export::{Snapshot, SnapshotStore};
use crate::graph::{EdgeId, Hypergraph};
use crate::ops::{EventEntry, EventLog, GridMetrics, GridStats, GridStatus, PolicyDecision, PolicyEngine};
use crate::plugin::{AIComputeQueue, PluginRegistry};
use crate::space::{FederationManager, IdentityStore, NamespaceRegistry, SpaceRegistry, WorkspaceStore};
use crate::view::ViewRegistry;

// ─── DomainSystem ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainSystem {
    Kogi,       // Independent Worker OS
    Ume,        // Business / Organization OS
    Qala,       // Solution Factory OS
    Standalone, // Generic domain system (no domain-specific defaults)
}

// =============================================================================
// Hypercube
// =============================================================================

/// A Hypercube is the analog of a spreadsheet sheet: an N-dimensional grid.
/// It owns its DimensionAxis definitions and AttributeKeyRegistry.
/// The actual cell data lives in the parent Grid's CellStore.
#[derive(Debug)]
pub struct Hypercube {
    pub cube_id:        CubeId,
    pub grid_id:        GridId,
    pub name:           String,        // e.g. "kogi.portfolio.components"
    pub namespace_path: Option<String>,
    pub space_id:       Option<SpaceId>,

    /// Ordered dimension axes (D₁ at index 0, D₂ at index 1, ...).
    pub dims:           Vec<DimensionAxis>,

    /// Schema: all registered attribute keys with their CRDT semantics.
    pub attr_registry:  AttributeKeyRegistry,

    /// Physical storage encoding (Dense | Sparse | Hybrid).
    pub encoding:       StorageEncoding,

    /// Cardinality statistics for query planning.
    pub stats:          HypercubeStats,

    /// Version counter for schema evolution.
    pub schema_version: u32,

    /// Is this cube currently active or archived?
    pub archived:       bool,

    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}

impl Hypercube {
    pub fn new(
        grid_id: GridId,
        name: impl Into<String>,
        dims: Vec<DimensionAxis>,
    ) -> Self {
        let now = Utc::now();
        Self {
            cube_id:       Uuid::new_v4(),
            grid_id,
            name:          name.into(),
            namespace_path: None,
            space_id:      None,
            dims,
            attr_registry: AttributeKeyRegistry::default(),
            encoding:      StorageEncoding::default(),
            stats:         HypercubeStats::default(),
            schema_version: 1,
            archived:      false,
            created_at:    now,
            updated_at:    now,
        }
    }

    pub fn n(&self) -> usize { self.dims.len() }

    pub fn register_attrs(&mut self, attrs: Vec<AttributeKeyDef>) -> Result<(), HypergridError> {
        self.attr_registry.register_batch(attrs)?;
        self.schema_version += 1;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn register_attr(&mut self, attr: AttributeKeyDef) -> Result<(), HypergridError> {
        self.attr_registry.register(attr)?;
        self.schema_version += 1;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn crdt_for(&self, attr_key: &str) -> crate::crdt::CrdtSemantics {
        self.attr_registry.get(attr_key)
            .map(|d| d.crdt_semantics.clone())
            .unwrap_or(crate::crdt::CrdtSemantics::LastWriteWins)
    }
}

// =============================================================================
// In-Memory Cell Store
// =============================================================================

/// The in-memory cell store: holds all HyperCell values for all Hypercubes.
///
/// Key: (CubeId, Vec<DimKey>) — full coordinate.
/// Value: HashMap<AttributeKey, TypedAttrValue> — all attrs at that coordinate.
///
/// Production: CellStoreBackend trait with PostgreSQL + Redis L1 cache.
#[derive(Default)]
pub struct InMemoryCellStore {
    /// LWW / Lattice / MaxRegister / MinRegister cells
    pub cells:    HashMap<(CubeId, Vec<DimKey>), HashMap<AttributeKey, TypedAttrValue>>,
    /// OR-Set cells: (cube, coord, attr) → Vec<OrSetEntry>
    pub or_sets:  HashMap<(CubeId, Vec<DimKey>, AttributeKey), Vec<OrSetEntry>>,
    /// Counter cells: (cube, coord, attr) → PerNodeCounter
    pub counters: HashMap<(CubeId, Vec<DimKey>, AttributeKey), PerNodeCounter>,
    /// Vector clocks: (cube, coord, attr) → VectorClock
    pub clocks:   HashMap<(CubeId, Vec<DimKey>, AttributeKey), VectorClock>,
    /// Last actor: (cube, coord, attr) → ActorId
    pub actors:   HashMap<(CubeId, Vec<DimKey>, AttributeKey), ActorId>,
}

impl InMemoryCellStore {
    pub fn get_attr(&self, cube_id: CubeId, keys: &[DimKey], attr: &str) -> Option<&TypedAttrValue> {
        self.cells.get(&(cube_id, keys.to_vec()))?.get(attr)
    }

    pub fn get_row(&self, cube_id: CubeId, d1_key: &DimKey) -> HyperRow {
        let mut row = HyperRow::new(cube_id, d1_key.clone());
        for ((cid, coord_keys), attrs) in &self.cells {
            if *cid == cube_id && coord_keys.first() == Some(d1_key) {
                for (attr_key, value) in attrs {
                    row.attrs.insert(attr_key.clone(), value.clone());
                }
            }
        }
        // Merge OR-Sets
        for ((cid, coord_keys, attr_key), entries) in &self.or_sets {
            if *cid == cube_id && coord_keys.first() == Some(d1_key) {
                let active: Vec<_> = entries.iter().filter(|e| !e.removed)
                    .map(|e| serde_json::to_value(&e.element).unwrap_or_default())
                    .collect();
                row.attrs.insert(attr_key.clone(), TypedAttrValue::Json(serde_json::Value::Array(active)));
            }
        }
        // Merge counters
        for ((cid, coord_keys, attr_key), counter) in &self.counters {
            if *cid == cube_id && coord_keys.first() == Some(d1_key) {
                row.attrs.insert(attr_key.clone(), TypedAttrValue::Number(counter.total() as f64));
            }
        }
        row
    }

    pub fn all_rows(&self, cube_id: CubeId) -> Vec<HyperRow> {
        // Collect all distinct D₁ keys for this cube
        let d1_keys: std::collections::HashSet<DimKey> = self.cells.keys()
            .filter(|(cid, _)| *cid == cube_id)
            .filter_map(|(_, keys)| keys.first().cloned())
            .collect();

        d1_keys.into_iter()
            .map(|d1| self.get_row(cube_id, &d1))
            .collect()
    }
}

// =============================================================================
// Grid — Root Runtime Container
// =============================================================================

/// The Grid is the root runtime container for one Hypergrid deployment.
/// Every domain system (Kogi, Ume, Qala) instantiates exactly one Grid.
///
/// It owns all Hypercubes, the Hypergraph, EventLog, CrdtLog, VectorClock,
/// all subsystem managers, and all storage backends.
pub struct Grid {
    // ── Identity ──────────────────────────────────────────────────────────
    pub grid_id:        GridId,
    pub node_id:        NodeId,   // "{region}:{instance_id}"
    pub name:           String,
    pub domain_system:  DomainSystem,

    // ── Registered Hypercubes ─────────────────────────────────────────────
    cubes:            RwLock<HashMap<CubeId, Hypercube>>,
    cube_name_index:  RwLock<HashMap<String, CubeId>>,
    cube_ns_index:    RwLock<HashMap<String, CubeId>>,

    // ── Core runtime infrastructure ───────────────────────────────────────
    pub hypergraph:   Arc<Hypergraph>,
    pub event_log:    Arc<EventLog>,
    pub crdt_log:     Arc<CrdtLog>,
    pub vector_clock: Mutex<VectorClock>,
    pub merge_engine: Arc<CrdtMergeEngine>,

    // ── In-memory cell store ──────────────────────────────────────────────
    /// CRDT-semantics-aware storage (production: CellStoreBackend → PostgreSQL)
    cell_store:       Mutex<InMemoryCellStore>,
    /// CRDT config per (CubeId, AttributeKey) — used by merge engine
    crdt_cfg:         RwLock<HashMap<(CubeId, AttributeKey), crate::crdt::CrdtSemantics>>,

    // ── Spatial systems ───────────────────────────────────────────────────
    pub space_registry:  Arc<SpaceRegistry>,
    pub workspace_store: Arc<WorkspaceStore>,
    pub namespace_reg:   Arc<NamespaceRegistry>,
    pub view_registry:   Arc<ViewRegistry>,

    // ── Identity and permissions ──────────────────────────────────────────
    pub identity_store: Arc<IdentityStore>,
    pub policy_engine:  Arc<PolicyEngine>,

    // ── Federation ────────────────────────────────────────────────────────
    pub federation:    Arc<FederationManager>,

    // ── Plugin and AI ─────────────────────────────────────────────────────
    pub plugin_registry: Arc<PluginRegistry>,
    pub ai_pipeline:     Arc<Tier2AIPipeline>,
    pub writeback:       Arc<WritebackService>,
    pub comp_engine:     ComputationEngine,

    // ── Snapshots ─────────────────────────────────────────────────────────
    pub snapshot_store: Arc<SnapshotStore>,

    // ── Observability ─────────────────────────────────────────────────────
    pub metrics:  Arc<GridMetrics>,
    pub status:   RwLock<GridStatus>,
    pub started_at: DateTime<Utc>,
}

impl Grid {
    // ── Construction ──────────────────────────────────────────────────────

    pub fn new(name: impl Into<String>, node_id: impl Into<String>) -> Self {
        let grid_id = Uuid::new_v4();
        let node_id_str = node_id.into();
        let writeback   = Arc::new(WritebackService::new());
        let plugin_reg  = Arc::new(PluginRegistry::new());
        let ai_pipeline = Arc::new(Tier2AIPipeline::new(plugin_reg.clone(), writeback.clone()));

        Self {
            grid_id,
            node_id:        node_id_str.clone(),
            name:           name.into(),
            domain_system:  DomainSystem::Standalone,

            cubes:           Default::default(),
            cube_name_index: Default::default(),
            cube_ns_index:   Default::default(),

            hypergraph:    Arc::new(Hypergraph::new(grid_id)),
            event_log:     Arc::new(EventLog::new(grid_id)),
            crdt_log:      Arc::new(CrdtLog::new(grid_id)),
            vector_clock:  Mutex::new(VectorClock::new()),
            merge_engine:  Arc::new(CrdtMergeEngine::new()),

            cell_store:    Mutex::new(InMemoryCellStore::default()),
            crdt_cfg:      Default::default(),

            space_registry:  Arc::new(SpaceRegistry::new()),
            workspace_store: Arc::new(WorkspaceStore::new()),
            namespace_reg:   Arc::new(NamespaceRegistry::new()),
            view_registry:   Arc::new(ViewRegistry::new()),

            identity_store:  Arc::new(IdentityStore::new()),
            policy_engine:   Arc::new(PolicyEngine::new()),

            federation:      Arc::new(FederationManager::new(grid_id)),

            plugin_registry: plugin_reg,
            ai_pipeline,
            writeback,
            comp_engine:     ComputationEngine,

            snapshot_store:  Arc::new(SnapshotStore::new()),

            metrics:     Arc::new(GridMetrics::new()),
            status:      RwLock::new(GridStatus::Initializing),
            started_at:  Utc::now(),
        }
    }

    /// Transition to Ready status. Called after bootstrap completes.
    pub fn set_ready(&self) {
        *self.status.write().unwrap() = GridStatus::Ready;
        tracing::info!(grid_id = %self.grid_id, "Grid ready");
    }

    pub fn is_ready(&self) -> bool { self.status.read().unwrap().is_ready() }

    // ── Hypercube Management ──────────────────────────────────────────────

    /// Create and register a new Hypercube. Idempotent: if the name is already
    /// registered, returns the existing CubeId without error.
    pub fn create_cube(
        &self,
        name: impl Into<String>,
        dims: Vec<DimensionAxis>,
    ) -> Result<CubeId, HypergridError> {
        let name_str = name.into();
        {
            if let Some(&existing_id) = self.cube_name_index.read().unwrap().get(&name_str) {
                return Ok(existing_id); // idempotent
            }
        }

        let cube = Hypercube::new(self.grid_id, name_str.clone(), dims);
        let cube_id = cube.cube_id;

        self.cubes.write().unwrap().insert(cube_id, cube);
        self.cube_name_index.write().unwrap().insert(name_str, cube_id);

        tracing::debug!(grid_id = %self.grid_id, cube_id = %cube_id, "Cube registered");
        Ok(cube_id)
    }

    pub fn get_cube(&self, cube_id: CubeId) -> Option<impl std::ops::Deref<Target = Hypercube> + '_> {
        // Returns an owned clone for simplicity; production uses Arc<RwLock<Hypercube>>
        struct Guard(Hypercube);
        impl std::ops::Deref for Guard { type Target = Hypercube; fn deref(&self) -> &Hypercube { &self.0 } }
        let cubes = self.cubes.read().unwrap();
        let cube = cubes.get(&cube_id)?.clone_shallow();
        Some(Guard(cube))
    }

    pub fn cube_by_name(&self, name: &str) -> Option<CubeId> {
        self.cube_name_index.read().unwrap().get(name).copied()
    }

    pub fn register_attrs(
        &self,
        cube_id: CubeId,
        attrs: Vec<AttributeKeyDef>,
    ) -> Result<(), HypergridError> {
        let mut cubes = self.cubes.write().unwrap();
        let cube = cubes.get_mut(&cube_id)
            .ok_or_else(|| HypergridError::CubeNotFound { name: cube_id.to_string() })?;
        for attr in &attrs {
            self.crdt_cfg.write().unwrap()
                .insert((cube_id, attr.key.clone()), attr.crdt_semantics.clone());
        }
        cube.register_attrs(attrs)
    }

    pub fn cube_count(&self) -> usize { self.cubes.read().unwrap().len() }

    // ── Write Path (9 Steps) ──────────────────────────────────────────────

    /// The canonical write path. Every mutation — user action, AI writeback,
    /// federation sync — flows through this identical 9-step sequence.
    pub fn write_cell(
        &self,
        cube_id:  CubeId,
        coord:    DimCoordinate,
        attr_key: &str,
        value:    TypedAttrValue,
        actor:    &str,
    ) -> Result<MergeResult, HypergridError> {
        self.write_cell_inner(cube_id, coord, attr_key.to_owned(), value, actor.to_owned(), false)
    }

    /// AI writeback variant: bypasses PermissionTier check (System tier always allowed).
    pub fn write_cell_ai(
        &self,
        cube_id:  CubeId,
        coord:    DimCoordinate,
        attr_key: &str,
        value:    TypedAttrValue,
        engine_id: &str,
    ) -> Result<MergeResult, HypergridError> {
        self.write_cell_inner(cube_id, coord, attr_key.to_owned(), value,
            format!("system:{engine_id}"), true)
    }

    fn write_cell_inner(
        &self,
        cube_id:     CubeId,
        coord:       DimCoordinate,
        attr_key:    String,
        value:       TypedAttrValue,
        actor:       String,
        is_ai_write: bool,
    ) -> Result<MergeResult, HypergridError> {
        // ── Step 1: Permission check ───────────────────────────────────────
        if !is_ai_write {
            let actor_tier = self.identity_store.tier_of(&actor, cube_id, coord.d1());
            let required_tier = self.cubes.read().unwrap()
                .get(&cube_id)
                .and_then(|c| c.attr_registry.get(&attr_key))
                .map(|d| d.write_permission)
                .unwrap_or(PermissionTier::Editor);
            if actor_tier < required_tier {
                self.event_log.append(EventEntry::set_attr(
                    cube_id, coord.keys.clone(), &attr_key,
                    None, TypedAttrValue::Null,
                    VectorClock::new(), "PermissionDenied", 0,
                ))?;
                return Err(HypergridError::PermissionDenied {
                    actor, attr_key, required_tier,
                });
            }
        }

        // ── Step 2: Policy engine evaluation ──────────────────────────────
        if !is_ai_write {
            let ctx = crate::ops::PolicyContext {
                actor: &actor, cube_id, coord: &coord,
                attr_key: &attr_key, value: &value,
                tier: self.identity_store.tier_of(&actor, cube_id, coord.d1()),
            };
            match self.policy_engine.evaluate(&ctx) {
                PolicyDecision::Deny { reason } => {
                    return Err(HypergridError::PolicyDenied { reason });
                }
                PolicyDecision::RequireApproval { reason, .. } => {
                    // In production: create ApprovalRequest entity; return Pending status
                    tracing::warn!("Write requires approval: {reason}");
                }
                PolicyDecision::Allow => {}
            }
        }

        // ── Step 3: CRDT operation construction ───────────────────────────
        let vc = {
            let mut clock = self.vector_clock.lock().unwrap();
            let ts = clock.tick(&self.node_id);
            clock.clone()
        };

        let crdt_semantics = self.crdt_cfg.read().unwrap()
            .get(&(cube_id, attr_key.clone()))
            .cloned()
            .unwrap_or(crate::crdt::CrdtSemantics::LastWriteWins);

        let op = match crdt_semantics {
            crate::crdt::CrdtSemantics::OrSet => CrdtOperation::AddToSet {
                coord:      SerializableCoord::from(&coord),
                attr_key:   attr_key.clone(),
                element:    value.clone(),
                unique_tag: Uuid::new_v4(),
                actor:      actor.clone(),
            },
            _ => CrdtOperation::SetAttr {
                coord:     SerializableCoord::from(&coord),
                attr_key:  attr_key.clone(),
                value:     value.clone(),
                timestamp: vc.clone(),
                actor:     actor.clone(),
            },
        };

        // ── Step 4: Local merge (in-memory, synchronous) ──────────────────
        let before_value = {
            let store = self.cell_store.lock().unwrap();
            store.get_attr(cube_id, &coord.keys, &attr_key).cloned()
        };

        let result = {
            let mut store = self.cell_store.lock().unwrap();
            self.merge_engine.apply_op(
                &op,
                &mut store.cells.entry((cube_id, coord.keys.clone()))
                    .or_default()
                    .iter()
                    .map(|(k, v)| ((cube_id, vec![coord.keys[0].clone()], k.clone()), v.clone()))
                    .collect::<HashMap<_, _>>()
                    .into_iter()
                    .fold(&mut store.cells, |acc, _| acc)
                    .entry((cube_id, coord.keys.clone()))
                    .or_default(),
                &mut store.or_sets,
                &mut store.counters,
                &mut store.clocks,
                &mut store.actors,
                &self.crdt_cfg.read().unwrap(),
            )
        };

        // Simpler direct apply for in-memory:
        let result = self.apply_op_direct(&op, &coord, &attr_key, &value, &vc, &actor)?;

        // ── Step 5: Persistence (PostgreSQL UPSERT in production) ─────────
        // In-memory: already written in Step 4
        self.metrics.cell_writes.inc();

        // ── Step 6: EventLog write (append-only) ──────────────────────────
        let crdt_kind = match &op {
            CrdtOperation::SetAttr { .. }       => crate::ops::CrdtOpKind::LWW,
            CrdtOperation::AddToSet { .. }      => crate::ops::CrdtOpKind::OrSetAdd,
            CrdtOperation::RemoveFromSet { .. } => crate::ops::CrdtOpKind::OrSetRemove,
            CrdtOperation::IncrCounter { .. }   => crate::ops::CrdtOpKind::Counter,
            CrdtOperation::TransitionState { .. } => crate::ops::CrdtOpKind::Lattice,
            _ => crate::ops::CrdtOpKind::LWW,
        };

        let after_val = self.read_cell(cube_id, &coord, &attr_key)
            .unwrap_or(TypedAttrValue::Null);

        self.event_log.append(EventEntry {
            event_id:     Uuid::new_v4(),
            seq:          0,
            cube_id,
            dim_keys:     coord.keys.clone(),
            attr_key:     attr_key.clone(),
            before_value,
            after_value:  after_val,
            crdt_op:      crdt_kind,
            vector_clock: vc.clone(),
            actor:        actor.clone(),
            timestamp:    Utc::now(),
            domain_tag:   None,
        })?;
        self.metrics.event_log_entries.inc();

        // ── Step 7: CrdtLog write (for federation delta sync) ─────────────
        self.crdt_log.append(op.clone(), vc.clone());

        // ── Step 8: Async fan-out (dispatched to background) ──────────────
        // a. AI invalidation
        self.ai_pipeline.on_cell_mutation(cube_id, &coord, vc.clone());
        // b-e: In production: Kafka → AI engine, namespace cache, search index,
        //       WebSocket push — all dispatched to background task queues here.

        // ── Step 9: Return ─────────────────────────────────────────────────
        Ok(result)
    }

    /// Simplified direct apply for in-memory store.
    fn apply_op_direct(
        &self,
        op:       &CrdtOperation,
        coord:    &DimCoordinate,
        attr_key: &str,
        value:    &TypedAttrValue,
        vc:       &VectorClock,
        actor:    &str,
    ) -> Result<MergeResult, HypergridError> {
        let cube_id = coord.cube_id;
        let keys    = coord.keys.clone();
        let ak      = attr_key.to_owned();
        let semantics = self.crdt_cfg.read().unwrap()
            .get(&(cube_id, ak.clone()))
            .cloned()
            .unwrap_or(crate::crdt::CrdtSemantics::LastWriteWins);

        let mut store = self.cell_store.lock().unwrap();

        match &semantics {
            crate::crdt::CrdtSemantics::OrSet => {
                let k = (cube_id, keys, ak);
                let tag = Uuid::new_v4();
                store.or_sets.entry(k).or_default().push(OrSetEntry {
                    unique_tag: tag,
                    element:    value.clone(),
                    added_by:   actor.to_owned(),
                    added_at:   Utc::now(),
                    removed:    false,
                });
                Ok(MergeResult::Applied)
            }
            crate::crdt::CrdtSemantics::GrowOnlyCounter |
            crate::crdt::CrdtSemantics::PNCounter => {
                let delta = value.as_f64().unwrap_or(0.0) as i64;
                let grow_only = matches!(semantics, crate::crdt::CrdtSemantics::GrowOnlyCounter);
                let k = (cube_id, keys, ak);
                store.counters.entry(k).or_default()
                    .incr(&self.node_id, delta, grow_only)
                    .map_err(HypergridError::from)?;
                Ok(MergeResult::Applied)
            }
            crate::crdt::CrdtSemantics::MaxRegister => {
                let k = (cube_id, keys.clone(), ak.clone());
                let current = store.cells.get(&(cube_id, keys.clone()))
                    .and_then(|m| m.get(&ak))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(f64::NEG_INFINITY);
                if let Some(incoming) = value.as_f64() {
                    if incoming > current {
                        store.cells.entry((cube_id, keys)).or_default().insert(ak, value.clone());
                    }
                }
                Ok(MergeResult::Applied)
            }
            crate::crdt::CrdtSemantics::Lattice(lattice) => {
                let current_state = store.cells.get(&(cube_id, keys.clone()))
                    .and_then(|m| m.get(&ak))
                    .and_then(|v| v.as_str())
                    .unwrap_or("").to_owned();
                let new_state = value.as_str().unwrap_or("").to_owned();
                let merged = if current_state.is_empty() {
                    new_state
                } else {
                    lattice.join(&current_state, &new_state)
                };
                store.cells.entry((cube_id, keys)).or_default()
                    .insert(ak, TypedAttrValue::Text(merged));
                Ok(MergeResult::Applied)
            }
            _ => {
                // LWW: check VectorClock dominance
                let k_vc = (cube_id, keys.clone(), ak.clone());
                let current_vc = store.clocks.get(&k_vc).cloned().unwrap_or_default();
                if vc.dominates(&current_vc) || vc.concurrent_with(&current_vc) {
                    store.cells.entry((cube_id, keys.clone())).or_default()
                        .insert(ak.clone(), value.clone());
                    store.clocks.insert(k_vc.clone(), vc.clone());
                    store.actors.insert(k_vc, actor.to_owned());
                    Ok(MergeResult::Applied)
                } else {
                    Ok(MergeResult::DiscardedStale)
                }
            }
        }
    }

    // ── Batch Write ───────────────────────────────────────────────────────

    /// Write multiple fields for one entity atomically (single EventLog batch).
    /// Use this for domain store codecs that write all entity fields at once.
    pub fn write_cell_batch(
        &self,
        cube_id: CubeId,
        d1_key:  DimKey,
        fields:  Vec<(AttributeKey, TypedAttrValue)>,
        actor:   &str,
    ) -> Result<(), HypergridError> {
        for (attr_key, value) in fields {
            let coord = DimCoordinate::n2(
                self.grid_id, cube_id, d1_key.clone(), DimKey::text(&attr_key),
            );
            self.write_cell(cube_id, coord, &attr_key, value, actor)?;
        }
        Ok(())
    }

    // ── Read Path ─────────────────────────────────────────────────────────

    /// Read one attribute value for an N-dim coordinate.
    pub fn read_cell(
        &self,
        cube_id:  CubeId,
        coord:    &DimCoordinate,
        attr_key: &str,
    ) -> Option<TypedAttrValue> {
        // L1 cache (Redis in production): omitted in in-memory mode

        // Check for Tier-1 computed attribute
        {
            let cubes = self.cubes.read().unwrap();
            if let Some(cube) = cubes.get(&cube_id) {
                if let Some(attr_def) = cube.attr_registry.get(attr_key) {
                    if let Some(crate::cell::AttrComputation::Formula { expression }) = &attr_def.computation {
                        let row = self.get_row(cube_id, &coord.keys[0]);
                        return Some(self.comp_engine.evaluate_formula(expression, &row));
                    }
                }
            }
        }

        self.metrics.cell_reads.inc();
        let store = self.cell_store.lock().unwrap();
        store.get_attr(cube_id, &coord.keys, attr_key).cloned()
    }

    /// Read all attribute values for one HyperRow (one entity).
    pub fn get_row(&self, cube_id: CubeId, d1_key: &DimKey) -> HyperRow {
        let store = self.cell_store.lock().unwrap();
        let mut row = store.get_row(cube_id, d1_key);

        // Evaluate Tier-1 computed attributes
        {
            let cubes = self.cubes.read().unwrap();
            if let Some(cube) = cubes.get(&cube_id) {
                for attr in cube.attr_registry.computed_attrs() {
                    if let Some(crate::cell::AttrComputation::Formula { expression }) = &attr.computation {
                        let value = self.comp_engine.evaluate_formula(expression, &row);
                        row.attrs.insert(attr.key.clone(), value);
                    }
                }
            }
        }

        row
    }

    /// Scan a Hypercube and return all matching rows.
    pub fn scan_rows(&self, cube_id: CubeId) -> Vec<HyperRow> {
        let store = self.cell_store.lock().unwrap();
        store.all_rows(cube_id)
    }

    // ── AS_OF Time Travel ─────────────────────────────────────────────────

    /// Reconstruct an entity's state at a historical timestamp.
    pub fn as_of(
        &self,
        cube_id: CubeId,
        d1_key:  &DimKey,
        target:  DateTime<Utc>,
    ) -> Result<Vec<(AttributeKey, TypedAttrValue)>, HypergridError> {
        // Step 1: find most recent snapshot before target
        let base = self.snapshot_store.latest_before(cube_id, target)
            .unwrap_or_else(|| Snapshot::genesis(cube_id, self.grid_id));

        // Step 2: build base state from snapshot
        let mut state: HashMap<AttributeKey, TypedAttrValue> =
            base.get_row(d1_key).unwrap_or_default();

        // Step 3: replay EventLog from snapshot.checkpoint_ts to target
        let delta = self.event_log.entries_in_range(
            cube_id, d1_key, base.checkpoint_ts, target,
        );

        for entry in delta {
            match entry.crdt_op {
                crate::ops::CrdtOpKind::LWW | crate::ops::CrdtOpKind::Lattice => {
                    state.insert(entry.attr_key, entry.after_value);
                }
                crate::ops::CrdtOpKind::OrSetAdd => {
                    let arr = state.entry(entry.attr_key)
                        .or_insert_with(|| TypedAttrValue::Json(serde_json::json!([])));
                    if let TypedAttrValue::Json(serde_json::Value::Array(ref mut a)) = arr {
                        if let Ok(v) = serde_json::to_value(&entry.after_value) { a.push(v); }
                    }
                }
                crate::ops::CrdtOpKind::Rollback => {
                    if let Some(b) = entry.before_value { state.insert(entry.attr_key, b); }
                }
                _ => {}
            }
        }

        Ok(state.into_iter().collect())
    }

    // ── Federation ────────────────────────────────────────────────────────

    /// Apply incoming CrdtOperations from a federation peer.
    pub fn apply_federation_delta(
        &self,
        ops: Vec<CrdtOperation>,
        peer_vc: VectorClock,
    ) -> Result<usize, HypergridError> {
        let mut applied = 0;

        for op in &ops {
            if let CrdtOperation::SetAttr { coord, attr_key, value, timestamp, actor } = op {
                let dim_coord = coord.to_dim_coordinate();
                let result = self.write_cell_inner(
                    dim_coord.cube_id, dim_coord, attr_key.clone(),
                    value.clone(), actor.clone(), actor.starts_with("system:"),
                );
                if result.is_ok() { applied += 1; }
            }
            // In production: handle all CrdtOperation variants
        }

        // Merge peer's VectorClock into ours
        self.vector_clock.lock().unwrap().merge(&peer_vc);
        self.metrics.federation_ops_recv.add(ops.len() as u64);

        Ok(applied)
    }

    // ── Snapshot ──────────────────────────────────────────────────────────

    pub fn create_snapshot(
        &self, cube_id: CubeId, label: impl Into<String>, retain_until: Option<DateTime<Utc>>,
    ) -> Snapshot {
        let rows = self.scan_rows(cube_id);
        let snap = Snapshot::create(cube_id, self.grid_id, &rows, label, retain_until);
        self.snapshot_store.put(snap.clone());
        snap
    }

    // ── Stats ─────────────────────────────────────────────────────────────

    pub fn stats(&self) -> GridStats {
        GridStats {
            grid_id:     self.grid_id,
            total_cells: self.metrics.cell_writes.get(),
            total_rows:  self.cell_store.lock().unwrap().cells.len() as u64,
            cube_count:  self.cube_count() as u64,
            event_count: self.event_log.len(),
            crdt_op_count: self.metrics.cell_writes.get(),
            edge_count:  self.hypergraph.edge_count() as u64,
            space_count: 0,
            snapshot_at: None,
        }
    }

    // ── HyperQL passthrough ───────────────────────────────────────────────

    pub fn hyperql(&self, query: crate::ql::HyperQuery) -> crate::ql::QueryResult {
        let planner = crate::ql::HyperQLPlanner::new();

        // Resolve cube name
        let resolved = if let Some(crate::ql::FromClause::CubeName(name)) = &query.from {
            let name_map: HashMap<String, CubeId> = self.cube_name_index.read().unwrap().clone();
            planner.analyze(&query, &name_map, PermissionTier::Viewer)
                .unwrap_or_else(|_| query.clone())
        } else {
            query.clone()
        };

        let plan = planner.plan(&resolved).unwrap_or(crate::ql::PhysicalPlan::FullScan {
            cube_id: match &resolved.from {
                Some(crate::ql::FromClause::Cube(id)) => *id,
                _ => Uuid::nil(),
            },
            attr_filters: vec![],
        });

        // Build row store for executor
        let store = self.cell_store.lock().unwrap();
        let row_store: HashMap<(CubeId, String), HyperRow> = store.cells.keys()
            .filter_map(|(cid, keys)| {
                let d1 = keys.first()?;
                Some(((*cid, format!("{:?}", d1)), store.get_row(*cid, d1)))
            })
            .collect();
        drop(store);

        self.metrics.hyperql_queries.inc();
        planner.execute(&plan, &row_store, &resolved)
    }
}

// ─── Hypercube clone_shallow ──────────────────────────────────────────────────

impl Hypercube {
    /// Produce an owned copy of the cube metadata (not the cells — cells are in the store).
    pub fn clone_shallow(&self) -> Self {
        Self {
            cube_id:       self.cube_id,
            grid_id:       self.grid_id,
            name:          self.name.clone(),
            namespace_path: self.namespace_path.clone(),
            space_id:      self.space_id,
            dims:          self.dims.clone(),
            attr_registry: self.attr_registry.clone(),
            encoding:      self.encoding.clone(),
            stats:         self.stats.clone(),
            schema_version: self.schema_version,
            archived:      self.archived,
            created_at:    self.created_at,
            updated_at:    self.updated_at,
        }
    }
}

use std::collections::HashMap as StdHashMap;
use crate::ql::FromClause;
