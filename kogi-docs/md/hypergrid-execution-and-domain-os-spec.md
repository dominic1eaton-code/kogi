# Hypergrid & Apapo — Execution and Computational Models

## Domain Operating System Specification & Practical Usage Guide

---

**Document Type:** Execution Model · Computational Architecture · Implementation Specification  
**Version:** 1.0 · March 2026 · Confidential — Internal Use Only  
**Companion to:** Apapo Platform Master Design Document v1.0  
**Audience:** Platform engineers · Domain system builders · SDK authors · Core contributors · Architects

---

## Table of Contents

**Part I — Hypergrid Execution Model**
1. The Grid Runtime
2. The Write Path: From Mutation to Cell
3. The Read Path: From Query to Result
4. The CRDT Merge Engine in Full
5. The VectorClock and Causal Ordering
6. Per-Attribute CRDT Reference — Complete Specification
7. The EventLog Execution Model
8. The Computation Engine: Tier-1 Formulas and Tier-2 AI
9. The HyperQL Execution Pipeline
10. The Federation Sync Protocol
11. The Plugin Execution Model
12. Schema Evolution Execution

**Part II — Apapo Computational Models**
13. The Dual-Write Event Architecture
14. The AI Computation Pipeline: kogi-engine, Ume AI, Qala AI Agent
15. The WritebackService Protocol
16. The CrossGridLink Sync Engine
17. The NL-to-HyperQL Translation Model
18. The Anomaly Detection Computational Model
19. The Lattice CRDT State Machine

**Part III — Using Hypergrid in Practice**
20. Setting Up a Grid
21. Defining Hypercubes and Schemas
22. Writing and Reading Entities
23. Working with CRDT Semantics
24. Querying with HyperQL
25. Building Views and Sheets
26. Working with the Hypergraph
27. Federation and Multi-Node Operation
28. Using the AI Computation Model
29. Building and Installing Plugins
30. Performance Tuning

**Part IV — Building Domain Operating Systems on Hypergrid**
31. What is a Domain Operating System?
32. The Domain OS Architecture Specification
33. Step-by-Step: Building a Domain OS
34. The Root Domain Concept
35. Hypercube Schema Design
36. CRDT Semantics Assignment
37. The DomainStore Codec
38. Graph Topology Design
39. AI Engine Integration
40. Space and Namespace Design
41. Cross-System Integration Points
42. Domain OS Checklist

**Part V — Domain OS Specifications: Kogi, Ume, Qala**
43. Kogi Domain OS Specification
44. Ume Domain OS Specification
45. Qala Domain OS Specification
46. Cross-Domain OS Integration Specification

**Appendices**
A. CrdtOperation Reference  
B. HyperQL Grammar  
C. Domain OS Implementation Checklist  
D. Error Reference  
E. Performance Benchmarks

---

# Part I — Hypergrid Execution Model

---

## 1. The Grid Runtime

The Grid is the root runtime container. When a domain operating system starts, it instantiates exactly one Grid. All subsequent operations — writes, reads, CRDT merges, AI computations, graph traversals, federation syncs — execute within and against that Grid.

### 1.1 Grid Initialization Sequence

```
Grid::new(config: GridConfig) -> Result<Grid, GridError>

Step 1: Config validation
  - Verify required fields: grid_id, node_id, storage_backend config
  - Validate node_id format: "{region}:{instance_id}"
  - Load KMS credentials; verify encryption key accessibility

Step 2: Storage backend initialization
  - Open PostgreSQL connection pool (max: config.pg_max_connections)
  - Connect Redis cluster; test PING on all nodes
  - Verify Kafka producer connectivity; create topics if absent
  - Apply pending database migrations (Flyway/Liquibase)
  - Initialize ClickHouse connection if analytics_backend = ClickHouse

Step 3: Plugin registry initialization
  - Load all statically-registered HypercubePlugin implementations
  - Call plugin.on_load(ctx) for each; log Degraded plugins; panic on missing Required plugins
  - Build plugin capability index: { PluginCapability → Vec<PluginId> }

Step 4: Grid bootstrap
  - Load existing Hypercube registrations from PostgreSQL cube_registry
  - For each cube: rebuild AttributeKeyRegistry, CRDT config, dimension axis list
  - Call DomainStore::bootstrap() for the configured domain system
    - Idempotent: re-registering an existing cube or attr is a no-op
    - New cubes: CREATE TABLE IF NOT EXISTS and INSERT into cube_registry
    - New attrs: INSERT into attr_key_registry; no migration of existing cells

Step 5: EventLog recovery
  - Query CrdtLog: SELECT * FROM crdt_log WHERE applied = false ORDER BY causal_ts ASC
  - Re-apply each op via apply_crdt_op(); advance VectorClock to recovered state
  - Mark all recovered ops applied = true
  - Emit RecoveryCompleted event to EventLog

Step 6: Async initialization (non-blocking)
  - Namespace registry: warm Redis cache for recently-accessed paths
  - Federation handshake: connect to all configured peers; delta sync diverged VectorClocks
  - AI engine warm-up: health-check all AIEnginePlugin impls; pre-load hot model context
  - Search index: verify Meilisearch connectivity; sync index if stale

Step 7: Ready
  - Set Grid.status = Ready
  - api-gateway readiness probe returns HTTP 200
```

### 1.2 Grid Internal State

```rust
pub struct Grid {
    // ── Identity ─────────────────────────────────────────────────
    pub grid_id:          GridId,           // UUID — globally unique and immutable
    pub node_id:          NodeId,           // "{region}:{instance_id}" for VectorClock
    pub name:             String,
    pub domain_system:    DomainSystem,     // Kogi | Ume | Qala | Standalone

    // ── Registered Hypercubes ─────────────────────────────────────
    pub cubes:            HashMap<CubeId, Arc<RwLock<Hypercube>>>,
    pub cube_name_index:  HashMap<String, CubeId>,  // "kogi.portfolio.components" → CubeId
    pub cube_ns_index:    HashMap<NamespacePath, CubeId>,

    // ── Core runtime infrastructure ───────────────────────────────
    pub hypergraph:       Arc<RwLock<Hypergraph>>,
    pub event_log:        Arc<EventLog>,
    pub crdt_log:         Arc<CrdtLog>,
    pub vector_clock:     Arc<Mutex<VectorClock>>,
    pub merge_engine:     Arc<CrdtMergeEngine>,

    // ── Spatial systems ───────────────────────────────────────────
    pub space_registry:   Arc<SpaceRegistry>,
    pub workspace_store:  Arc<WorkspaceStore>,
    pub namespace_reg:    Arc<NamespaceRegistry>,

    // ── Identity and permissions ──────────────────────────────────
    pub identity_store:   Arc<IdentityStore>,
    pub policy_engine:    Arc<PolicyEngine>,

    // ── Federation ────────────────────────────────────────────────
    pub federation:       Arc<FederationManager>,
    pub federation_peers: Vec<FederationPeer>,

    // ── Plugin registry ───────────────────────────────────────────
    pub plugin_registry:  Arc<PluginRegistry>,

    // ── Storage backends (pluggable) ──────────────────────────────
    pub cell_store:       Arc<dyn CellStoreBackend + Send + Sync>,
    pub snapshot_store:   Arc<dyn SnapshotBackend + Send + Sync>,
    pub search_index:     Arc<dyn SearchBackend + Send + Sync>,
    pub graph_store:      Arc<dyn GraphBackend + Send + Sync>,

    // ── Runtime status ────────────────────────────────────────────
    pub status:           GridStatus,  // Initializing | Ready | Degraded | ShuttingDown
    pub started_at:       DateTime<Utc>,
}
```

---

## 2. The Write Path: From Mutation to Cell

Every write to any entity in any domain system flows through this exact sequence. Understanding the write path is essential for building correct domain systems.

```
User/Service calls: grid.write_cell(cube_id, coord, attr_key, value, actor)

Step 1: Permission check (PermissionTier enforcement)
  actor_tier = identity_store.tier_of(actor, cube_id, coord.d1_key())
  required_tier = cube.attr_registry.get(attr_key).write_permission
  if actor_tier < required_tier:
    event_log.append(PermissionDenied { actor, attr_key, coord })
    return Err(HypergridError::PermissionDenied)

Step 2: Policy engine evaluation
  context = PolicyContext { actor, coord, attr_key, value, cube_id }
  decision = policy_engine.evaluate(context)
  match decision:
    PolicyDecision::Allow       → continue
    PolicyDecision::Deny(msg)   → return Err(PolicyDenied(msg))
    PolicyDecision::RequireApproval(reason) → queue ApprovalRequest; return Pending

Step 3: CRDT operation construction
  vc = vector_clock.tick(node_id)  // advance logical clock BEFORE writing
  op = CrdtOperation::SetAttr {
    coord, attr_key, value, timestamp: vc, actor
  }
  // For OR-Set: CrdtOperation::AddToSet { coord, attr_key, element, unique_tag: Uuid::new_v4() }

Step 4: Local merge (synchronous, in-memory)
  result = merge_engine.apply_crdt_op(&op, &mut cell_store, &mut event_log)?
  // Merge engine resolves conflicts per CRDT semantics (see §4)
  // Result: Applied | ConflictRecorded { kept }

Step 5: Persistence (synchronous write to PostgreSQL)
  cell_store.put_cell_batch(vec![updated_cell])?
  // PostgreSQL UPSERT on (grid_id, cube_id, dim1_key, dim2_key, dim3_key, dim4_key)

Step 6: EventLog write (append-only)
  event_log.append(EventEntry {
    event_id:     Uuid::new_v4(),
    cube_id,
    dim_keys:     coord.keys.clone(),
    attr_key,
    before_value: previous_value,
    after_value:  value,
    crdt_op:      "LWW",
    vector_clock: vc,
    actor,
    timestamp:    Utc::now(),
  })?

Step 7: CrdtLog write (for federation delta sync)
  crdt_log.append(CrdtLogEntry { op, applied: true, sync_cursors: {} })?
  // CrdtLog hot path: Redis LPUSH; background flush to PostgreSQL

Step 8: Async fan-out (non-blocking; scheduled to background task queue)
  a. AI invalidation: notify all registered HypercubePlugin impls
     for each plugin that registered coord via on_cell_mutation(cube_id, coord):
       ai_compute_queue.push(AIComputeRequest { plugin_id, attr_key, coord })

  b. Namespace cache invalidation (if namespace_path cell changed):
       namespace_reg.cache.invalidate(coord.d1_namespace_path())

  c. Search index update:
       search_index.update_row(cube_id, coord.d1_key())

  d. Federation broadcast:
       for each peer in federation.active_peers():
         kafka_producer.send(
           topic: "crdt.delta.{grid_id}",
           key:   peer.node_id,
           value: CrdtDelta { ops: [op], from_vc: vc }
         )

  e. WebSocket push (for real-time collaborative editing):
       ws_hub.broadcast_to_watchers(cube_id, coord.d1_key(), CellUpdate { coord, value })

Step 9: Return Ok(MergeResult::Applied) to caller
```

### 2.1 Batch Write Optimization

For domain system bridge layers writing many fields simultaneously (e.g., `ComponentStore::write()` persisting all 38 fields of a PortfolioComponent in one call), use the batch write API:

```rust
// Batch write: all ops in one PostgreSQL transaction; single EventLog batch entry
grid.write_cell_batch(cube_id, vec![
    (coord(id, "name"),        "value", TypedAttrValue::Text(name), actor),
    (coord(id, "status"),      "value", TypedAttrValue::Json(status), actor),
    (coord(id, "budget"),      "value", TypedAttrValue::Number(budget), actor),
    // ... all 38 fields
], actor)?

// Produces:
//   1 PostgreSQL transaction with N UPSERTs (atomic)
//   N EventLog entries in one batch INSERT
//   N CrdtLog entries in one Redis LPUSH
//   1 AI invalidation notification per affected plugin
//   1 WebSocket broadcast (batched cell updates)
```

---

## 3. The Read Path: From Query to Result

### 3.1 Single Cell Read

```
grid.read_cell(cube_id, coord, attr_key) -> Option<TypedAttrValue>

Step 1: Redis L1 cache check
  key = "cell:{grid_id}:{cube_id}:{dim_keys_hash}:{attr_key}"
  if let Some(cached) = redis.get(key):
    return Some(deserialize(cached))

Step 2: Computed attribute check
  if cube.attr_registry.get(attr_key).computation.is_some():
    return computation_engine.evaluate(attr_key, coord, cube)?
    // Tier-1: synchronous formula evaluation
    // Tier-2: return cached AiSignal or None if not yet computed

Step 3: PostgreSQL primary key lookup
  SELECT attr_value FROM hypergrid_cells
  WHERE grid_id=$1 AND cube_id=$2 AND dim1_key=$3 AND dim2_key=$4
        AND dim3_key=$5 AND dim4_key=$6
  // O(1) via primary key; <5ms p50

Step 4: Visibility check
  visibility_mask = identity_store.mask_for(caller, cube_id)
  if not visibility_mask.allows_attr(attr_key, coord):
    return None  // Attribute not visible to this caller

Step 5: Write to Redis cache (TTL: 30s for hot attrs, 300s for cold)
  redis.setex(key, ttl, serialize(value))

Step 6: Return TypedAttrValue
```

### 3.2 HyperRow Read (all fields for one entity)

```
grid.get_row(cube_id, d1_key) -> Vec<HyperCell>

Step 1: Check row cache
  row_key = "row:{grid_id}:{cube_id}:{d1_key}"
  if let Some(cached_row) = redis.hgetall(row_key):
    apply_visibility_mask(cached_row, caller)
    return cached_row

Step 2: PostgreSQL scan with D1 filter
  SELECT * FROM hypergrid_cells
  WHERE grid_id=$1 AND cube_id=$2 AND dim1_key=$3
  ORDER BY dim2_key ASC
  // Batched read of all property columns for this entity
  // Uses idx_cells_d1 (grid_id, cube_id, dim1_key) index: ~10ms p50

Step 3: Resolve computed attributes for this row
  for each attr in cube.attr_registry.computed_attrs():
    if not in result_set:
      computed = computation_engine.evaluate(attr, coord(d1_key, attr), cube)?
      result_set.push(HyperCell { coord, attributes: {attr: computed} })

Step 4: Apply VisibilityMask
  filtered = result_set.filter(|cell| visibility_mask.allows(cell, caller))

Step 5: Cache the filtered row
  redis.hmset(row_key, filtered.to_hash_map(), EX=30)

Step 6: Return filtered HyperCells
```

### 3.3 DimSlice Query

```
grid.scan_slice(cube_id, slice: DimSlice, limit, offset) -> Vec<HyperRow>

DimSlice = {
  d1_filter: Option<DimKeyPredicate>,  // Point | Range | Set | Predicate | Null
  d2_filter: Option<DimKeyPredicate>,
  d3_filter: Option<DimKeyPredicate>,
  d4_filter: Option<DimKeyPredicate>,
  attr_filters: Vec<(AttributeKey, ValuePredicate)>,  // WHERE cell value matches
}

Step 1: Query plan generation (HG-QL planner)
  plan = planner.plan_slice(cube_id, slice)
  // Selects: primary index | JSONB GIN index | TimeAxis BRIN | full scan
  // Cost estimates based on cardinality statistics per axis

Step 2: Execute via CellStoreBackend.scan_slice()
  For N=2 cubes (dense encoding):
    SELECT * FROM {domain_entity_table}  // e.g. components
    WHERE {d1_filter} AND {d2_filter} AND {attr_filters...}
    ORDER BY {order_by} LIMIT $L OFFSET $O

  For N>2 cubes (hybrid encoding):
    SELECT base.*, ext.dim3_key, ext.dim4_key, ext.attr_value
    FROM {base_table} base
    LEFT JOIN {extension_table} ext ON base.dim1_key = ext.dim1_key
    WHERE {all_dim_filters_and_attr_filters}
    ORDER BY {order_by} LIMIT $L OFFSET $O

  For analytics (N>4 or result set >1M rows):
    Route to ClickHouse via analytics_backend.scan_slice()

Step 3: Assemble HyperRow objects
  group cells by d1_key into HyperRow structs
  resolve computed attributes for each row

Step 4: Apply VisibilityMask per caller
  filter rows and columns per caller's mask

Step 5: Return Vec<HyperRow>
```

---

## 4. The CRDT Merge Engine in Full

The CrdtMergeEngine is the heart of the HG-CRDT module. It applies incoming CrdtOperations from any source — local user writes, federation delta sync, AI writeback, schema migrations — in a way that preserves all CRDT invariants regardless of operation arrival order.

The key invariant: **the same set of CrdtOperations applied in any order always produces the same final state.** This is the mathematical definition of a CRDT, and it is what makes federation correct.

### 4.1 Merge Engine Implementation

```rust
impl CrdtMergeEngine {
    pub fn apply_crdt_op(
        &self,
        op: &CrdtOperation,
        store: &mut dyn CellStoreBackend,
        event_log: &mut EventLog,
    ) -> Result<MergeResult, CrdtError> {

        match op {
            // ── LastWriteWins (LWW) ─────────────────────────────────────
            CrdtOperation::SetAttr { coord, attr_key, value, timestamp, actor }
            if self.crdt_for(coord, attr_key)? == CrdtSemantics::LastWriteWins => {
                let current_ts = store.get_vector_clock(coord, attr_key)
                    .unwrap_or_default();
                if timestamp.dominates(&current_ts) {
                    store.put_attr(coord, attr_key, value.clone())?;
                    event_log.append(EventEntry::set_attr(op))?;
                    Ok(MergeResult::Applied)
                } else if timestamp.concurrent_with(&current_ts) {
                    // Concurrent LWW: deterministic tiebreak by ActorId
                    if actor.as_str() > store.get_last_actor(coord, attr_key)? {
                        store.put_attr(coord, attr_key, value.clone())?;
                        event_log.append(EventEntry::set_attr(op))?;
                        Ok(MergeResult::Applied)
                    } else {
                        // Losing side: record for conflict surfacing
                        event_log.append(EventEntry::conflict(
                            op, ConflictKind::LwwLosing { kept_actor: actor.clone() }
                        ))?;
                        Ok(MergeResult::ConflictRecorded { kept: "current".into() })
                    }
                } else {
                    // op is causally older than current state: safe to discard
                    Ok(MergeResult::DiscardedStale)
                }
            }

            // ── OR-Set Add ──────────────────────────────────────────────
            CrdtOperation::AddToSet { coord, attr_key, element, unique_tag, .. } => {
                // OR-Set: ALL concurrent Adds survive — no conflict possible
                // unique_tag makes this add distinguishable from concurrent adds
                // of the same logical value from different nodes
                store.or_set_add(coord, attr_key, element.clone(), *unique_tag)?;
                event_log.append(EventEntry::or_set_add(op))?;
                Ok(MergeResult::Applied)
            }

            // ── OR-Set Remove ───────────────────────────────────────────
            CrdtOperation::RemoveFromSet { coord, attr_key, unique_tag, .. } => {
                // OR-Set: remove ONLY the entry with this exact unique_tag
                // If a concurrent AddToSet with a different unique_tag arrived
                // for the same logical value, THAT entry survives
                store.or_set_remove(coord, attr_key, *unique_tag)?;
                event_log.append(EventEntry::or_set_remove(op))?;
                Ok(MergeResult::Applied)
            }

            // ── GrowOnly / PN-Counter ───────────────────────────────────
            CrdtOperation::IncrCounter { coord, attr_key, delta, node_id } => {
                match self.crdt_for(coord, attr_key)? {
                    CrdtSemantics::GrowOnlyCounter => {
                        if *delta <= 0 {
                            return Err(CrdtError::GrowOnlyViolation {
                                attempted_delta: *delta
                            });
                        }
                        store.counter_incr(coord, attr_key, *delta, *node_id)?;
                    }
                    CrdtSemantics::PNCounter => {
                        // Any sign permitted; tracked per node for merge
                        store.counter_incr(coord, attr_key, *delta, *node_id)?;
                    }
                    _ => return Err(CrdtError::WrongCrdtType),
                }
                event_log.append(EventEntry::counter_incr(op))?;
                Ok(MergeResult::Applied)
            }

            // ── MaxRegister ─────────────────────────────────────────────
            CrdtOperation::SetAttr { coord, attr_key, value, .. }
            if self.crdt_for(coord, attr_key)? == CrdtSemantics::MaxRegister => {
                let current = store.get_attr_number(coord, attr_key).unwrap_or(f64::NEG_INFINITY);
                let incoming = value.as_f64().ok_or(CrdtError::TypeMismatch)?;
                if incoming > current {
                    store.put_attr(coord, attr_key, value.clone())?;
                    event_log.append(EventEntry::set_attr(op))?;
                    Ok(MergeResult::Applied)
                } else {
                    Ok(MergeResult::DiscardedStale)  // Max register: lower value discarded
                }
            }

            // ── Lattice (lifecycle state machine) ───────────────────────
            CrdtOperation::TransitionState { coord, attr_key, from_state, to_state, actor, timestamp } => {
                let lattice = self.lattice_for(coord, attr_key)?;
                // Validate transition in partial order
                if !lattice.is_valid_transition(from_state, to_state) {
                    return Err(CrdtError::InvalidStateTransition {
                        from: from_state.clone(), to: to_state.clone(),
                        valid_successors: lattice.successors(from_state),
                    });
                }
                let current = store.get_attr_string(coord, attr_key).unwrap_or_default();
                let merged = lattice.join(&current, to_state);
                // join() = least upper bound: always advances, never regresses
                if merged != current {
                    store.put_attr(coord, attr_key, TypedAttrValue::Text(merged))?;
                    event_log.append(EventEntry::state_transition(op))?;
                }
                Ok(MergeResult::Applied)
            }

            // ── AppendLog (audit-only append sequence) ──────────────────
            CrdtOperation::AppendLog { coord, attr_key, entry, causal_ts } => {
                // Insert at causally correct position — not necessarily end
                store.append_log_insert(coord, attr_key, entry.clone(), *causal_ts)?;
                event_log.append(EventEntry::append_log(op))?;
                Ok(MergeResult::Applied)
            }

            // ── Deep JSON Merge ─────────────────────────────────────────
            CrdtOperation::DeepMergeJson { coord, attr_key, delta, timestamp, actor } => {
                // Merges only the leaf keys present in delta into the existing JSON
                // LWW per leaf key using the provided timestamp
                store.json_deep_merge(coord, attr_key, delta, timestamp, actor)?;
                event_log.append(EventEntry::deep_merge(op))?;
                Ok(MergeResult::Applied)
            }

            // ── Schema mutations ────────────────────────────────────────
            CrdtOperation::AddDimension { cube_id, axis, schema_ts } => {
                let cube = self.get_cube_mut(*cube_id)?;
                cube.add_axis(axis.clone(), *schema_ts)?;
                event_log.append(EventEntry::schema_add_dim(op))?;
                Ok(MergeResult::Applied)
            }

            CrdtOperation::RegisterAttrKey { cube_id, attr_def, schema_ts } => {
                let cube = self.get_cube_mut(*cube_id)?;
                cube.attr_registry.register(attr_def.clone(), *schema_ts)?;
                event_log.append(EventEntry::schema_add_attr(op))?;
                Ok(MergeResult::Applied)
            }

            // ── Graph mutations ─────────────────────────────────────────
            CrdtOperation::AddEdge { edge } => {
                self.hypergraph.write().unwrap().add_edge(edge.clone())?;
                event_log.append(EventEntry::add_edge(op))?;
                Ok(MergeResult::Applied)
            }

            CrdtOperation::RemoveEdge { edge_id } => {
                self.hypergraph.write().unwrap().remove_edge(*edge_id)?;
                event_log.append(EventEntry::remove_edge(op))?;
                Ok(MergeResult::Applied)
            }

            _ => self.dispatch_specialized(op, store, event_log),
        }
    }
}
```

### 4.2 Counter Physical Storage

PN-Counter and GrowOnlyCounter values are stored per-node in PostgreSQL to ensure commutative merge:

```sql
-- Counter storage: separate positive/negative accumulators per node
CREATE TABLE hypergrid_counters (
    grid_id      UUID NOT NULL,
    cube_id      UUID NOT NULL,
    dim1_key     UUID NOT NULL,
    dim2_key     TEXT NOT NULL,
    attr_key     TEXT NOT NULL,
    node_id      TEXT NOT NULL,   -- which federation node incremented
    positive_sum BIGINT NOT NULL DEFAULT 0,
    negative_sum BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (grid_id, cube_id, dim1_key, dim2_key, attr_key, node_id)
);

-- Computed value: SELECT SUM(positive_sum) - SUM(negative_sum) GROUP BY (all except node_id)
-- GrowOnly: only positive_sum is ever updated; negative_sum stays 0
```

### 4.3 OR-Set Physical Storage

```sql
-- OR-Set storage: one row per (element, unique_tag) pair
CREATE TABLE hypergrid_orsets (
    grid_id      UUID NOT NULL,
    cube_id      UUID NOT NULL,
    dim1_key     UUID NOT NULL,
    dim2_key     TEXT NOT NULL,
    attr_key     TEXT NOT NULL,
    unique_tag   UUID NOT NULL,     -- unique per AddToSet operation
    element      JSONB NOT NULL,    -- the element value
    added_by     TEXT NOT NULL,
    added_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    removed      BOOLEAN NOT NULL DEFAULT false,
    removed_by   TEXT,
    removed_at   TIMESTAMPTZ,
    PRIMARY KEY (grid_id, cube_id, dim1_key, dim2_key, attr_key, unique_tag)
);

-- Current set value: SELECT element FROM hypergrid_orsets WHERE removed = false
-- Remove: UPDATE SET removed=true WHERE unique_tag=$1 (only marks, never deletes)
-- OR-Set invariant: an element removed and concurrently re-added has two rows;
-- the re-added row (removed=false) means the element is in the set
```

---

## 5. The VectorClock and Causal Ordering

```rust
pub struct VectorClock(pub HashMap<NodeId, u64>);

impl VectorClock {
    /// Advance this node's logical clock before any local mutation.
    /// Must be called BEFORE constructing the CrdtOperation.
    pub fn tick(&mut self, node_id: NodeId) -> u64 {
        let counter = self.0.entry(node_id).or_insert(0);
        *counter += 1;
        *counter
    }

    /// Merge two clocks: element-wise maximum.
    /// Call on receiving remote CrdtOperations (federation sync).
    pub fn merge(&mut self, other: &VectorClock) {
        for (node, &ts) in &other.0 {
            self.0.entry(*node)
                .and_modify(|v| *v = (*v).max(ts))
                .or_insert(ts);
        }
    }

    /// Does self causally dominate other? (self happened-after-or-equal)
    pub fn dominates(&self, other: &VectorClock) -> bool {
        other.0.iter().all(|(node, &ts)|
            self.0.get(node).copied().unwrap_or(0) >= ts
        )
    }

    /// Are these clocks concurrent? (neither happened-before the other)
    pub fn concurrent_with(&self, other: &VectorClock) -> bool {
        !self.dominates(other) && !other.dominates(self)
    }

    /// happened_before: self causally precedes other
    pub fn happened_before(&self, other: &VectorClock) -> bool {
        self.0.iter().all(|(n, &ts)|
            other.0.get(n).copied().unwrap_or(0) >= ts
        ) && self.0 != other.0
    }

    /// Delta: which (node, ts) pairs in self are newer than other?
    /// Used to compute what to send in federation delta sync.
    pub fn delta_since(&self, other: &VectorClock) -> HashMap<NodeId, u64> {
        self.0.iter()
            .filter(|(node, &ts)| ts > other.0.get(node).copied().unwrap_or(0))
            .map(|(&node, &ts)| (node, ts))
            .collect()
    }
}

// NodeId format for identity-aware CRDT:
// "{federation_node_id}:{identity_tag}"
// Example: "node-us-east-1:@alice-work"
// This carries identity context alongside causal context, enabling
// per-identity conflict tracking and actor-level audit attribution.
```

### 5.1 VectorClock Lifecycle in a Write

```
1. grid.write_cell() called by actor "@alice-work" on node "node-us-east-1"

2. tick: vector_clock.tick("node-us-east-1:@alice-work")
   Clock before: { "node-us-east-1:@alice-work": 41, "node-eu-west-1:@bob": 17 }
   Clock after:  { "node-us-east-1:@alice-work": 42, "node-eu-west-1:@bob": 17 }

3. CrdtOperation constructed with timestamp = { ...: 42, ...: 17 }

4. Merge engine checks: does 42 dominate current cell's { ...: 38, ...: 17 }?
   Yes (42 > 38 on alice's node; equal on bob's node) → Applied

5. Cell stored with vector_clock = { "node-us-east-1:@alice-work": 42, "node-eu-west-1:@bob": 17 }

6. Bob's node receives delta sync with this op
   Bob's current cell clock: { "node-us-east-1:@alice-work": 41, "node-eu-west-1:@bob": 17 }
   Incoming op clock:         { "node-us-east-1:@alice-work": 42, "node-eu-west-1:@bob": 17 }
   42 > 41 → Bob's node applies Alice's write
   Bob's VectorClock merges: { "node-us-east-1:@alice-work": max(41,42)=42, ... }
```

---

## 6. Per-Attribute CRDT Reference — Complete Specification

| CRDT Type | Declaration | Merge Rule | Conflict Behavior | Physical Storage | Typical Use |
|-----------|-------------|-----------|-------------------|-----------------|-------------|
| **LastWriteWins (LWW)** | `CrdtSemantics::LastWriteWins` | Highest VectorClock timestamp wins. Tiebreak: ActorId lexicographic sort (deterministic). | Losing write stored in ConflictRecord; surfaced to actor via notification channel | Cell JSONB value overwrite | name, description, config, payload, JSON blobs, enum fields, reference IDs |
| **OR-Set** | `CrdtSemantics::OrSet` | All concurrent Adds survive (each with unique_tag). Removes target only the specific unique_tag. | No conflict for concurrent Adds. Remove vs concurrent Add: Add wins (the un-removed entry survives). | hypergrid_orsets table; one row per (element, unique_tag) | tags, owners, members, children, dependencies, toolbox_ids, policy_ids |
| **GrowOnlyCounter** | `CrdtSemantics::GrowOnlyCounter` | Sum all IncrCounter ops across all nodes. Decrement ops rejected at merge time. | Always commutes; no conflict. Decrement = `CrdtError::GrowOnlyViolation`. | hypergrid_counters per-node positive_sum | restart_count, view_count, follower_count, like_count |
| **PNCounter** | `CrdtSemantics::PNCounter` | Sum of (positive_sum − negative_sum) per node across all nodes. Any sign permitted. | Commutes. No conflict. Net value can go negative. | hypergrid_counters per-node positive_sum + negative_sum | budget_spent (tracked credits/debits), hours_logged, allocated_units |
| **MaxRegister** | `CrdtSemantics::MaxRegister` | Highest numeric value always wins, regardless of causal ordering. | Always resolves to maximum. Losing write discarded without record. | Cell JSONB overwrite only if incoming > current | version, sequence_number, schema_version, sprint_number, generation |
| **MinRegister** | `CrdtSemantics::MinRegister` | Lowest numeric value wins. | Always resolves to minimum. | Cell JSONB overwrite only if incoming < current | earliest_deadline, min_threshold, floor_price |
| **Lattice** | `CrdtSemantics::Lattice(LatticeOrder)` | join(a, b) = least upper bound in the defined partial order. Always advances; never regresses without Admin override. | Concurrent transitions to different successor states: resolved by LUB of the partial order. Invalid transition rejected at merge time. | Cell JSONB string (state name) | lifecycle_state, module_state, solution_lifecycle, ccr_status |
| **AppendLog** | `CrdtSemantics::AppendLog` | All entries from all nodes are preserved. Causally ordered by VectorClock on insertion. | No conflict: all appends survive; causal ordering determines sequence. | Separate append_log table; one row per entry with causal_ts | activity_log, comment_thread, decision_log |
| **DeepMergeJson** | `CrdtSemantics::DeepMergeJson` | LWW applied per leaf key in the JSON tree. Only keys present in the delta are updated. | Per-leaf-key LWW conflicts; handled same as LWW. | Cell JSONB deep merge operation | plugin_configs, feature_flags, configuration trees |
| **Custom** | `CrdtSemantics::Custom(plugin_id)` | Merge logic delegated to AttributeTypePlugin::merge() | Plugin-defined | Plugin-defined | Domain-specific merge semantics not covered by built-ins |

---

## 7. The EventLog Execution Model

The EventLog is append-only. No UPDATE or DELETE is ever issued against it. This is enforced at three layers: application code (no delete methods exposed), PostgreSQL RLS (DELETE policy returns false), and periodic Merkle tree verification to detect tampering.

### 7.1 EventLog Write Path

```rust
impl EventLog {
    pub fn append(&mut self, entry: EventEntry) -> Result<EventId, EventLogError> {
        // 1. Assign monotonic sequence number within this grid
        let seq = self.sequence.fetch_add(1, Ordering::SeqCst);
        let entry = EventEntry { seq, ..entry };

        // 2. Write to hot buffer (in-memory VecDeque, capped at HOT_CAP=10_000)
        self.hot_buffer.push_back(entry.clone());
        if self.hot_buffer.len() > HOT_CAP {
            // 3. Flush oldest entries to PostgreSQL event_log table
            self.flush_to_postgres(100).await?;
            // 4. If PostgreSQL event_log partition is full (>500K entries), archive to S3
            if self.pg_row_count() > PARTITION_ARCHIVE_THRESHOLD {
                self.archive_partition_to_s3().await?;
            }
        }

        // 5. Also write to Kafka for downstream consumers (kogi-engine, Ume AI, Qala AI)
        self.kafka_producer.send("events.audit.{grid_id}", &entry).await?;

        Ok(entry.event_id)
    }
}
```

### 7.2 AS_OF Time-Travel Execution

```
grid.as_of(timestamp: DateTime<Utc>, cube_id: CubeId, d1_key: DimKey)
  -> Result<Vec<HyperCell>, TimeTravelError>

Step 1: Determine EventLog range
  hot_events = event_log.hot_buffer.iter()
    .filter(|e| e.cube_id == cube_id AND e.dim_keys[0] == d1_key AND e.timestamp <= timestamp)
    .collect()

  cold_events = (if timestamp < hot_window_start):
    postgres.query("SELECT * FROM event_log WHERE cube_id=$1 AND dim1_key=$2
                    AND timestamp <= $3 ORDER BY seq ASC", ...)
    // May also query S3 archive for very old timestamps

Step 2: Replay events in causal order (seq ASC)
  state: HashMap<(DimKeys, AttributeKey), TypedAttrValue> = {}
  for event in hot_events + cold_events sorted by seq:
    match event.crdt_op:
      "LWW"          → state[(event.dim_keys, event.attr_key)] = event.after_value
      "OR-Set-Add"   → state[(event.dim_keys, event.attr_key)].or_set_add(...)
      "OR-Set-Remove"→ state[(event.dim_keys, event.attr_key)].or_set_remove(...)
      "Counter"      → state[(event.dim_keys, event.attr_key)].counter_incr(...)
      "Lattice"      → state[(event.dim_keys, event.attr_key)] = lattice.join(current, new)
      "Rollback"     → re-apply the pre-rollback values from rollback event payload

Step 3: Assemble HyperCells from replayed state
  return state.into_iter().map(|((coord, attr), value)| HyperCell { coord, attr, value })

// Performance: AS_OF within hot window (10K events): ~100ms p50
// AS_OF from cold PostgreSQL: ~500ms p50
// AS_OF from S3 archive: restore on demand; ~5s p50 (rare)
```

---

## 8. The Computation Engine: Tier-1 Formulas and Tier-2 AI

The HG-COMP module handles two classes of computed attributes, with completely different execution models.

### 8.1 Tier-1: Synchronous Formula Attributes

Tier-1 computations are deterministic, fast, and always fresh. They are evaluated inline during read operations.

```rust
// Tier-1 formula declaration in AttributeKeyDef
AttributeKeyDef {
    key: "budget_remaining".into(),
    computation: Some(AttrComputation::Formula {
        expression: FormulaExpr::Sub(
            Box::new(FormulaExpr::Attr("budget".into())),
            Box::new(FormulaExpr::Attr("budget_spent".into())),
        ),
    }),
    ..Default::default()
}

// Evaluation on read
impl ComputationEngine {
    pub fn evaluate_formula(
        &self,
        expr: &FormulaExpr,
        row: &HyperRow,
    ) -> TypedAttrValue {
        match expr {
            FormulaExpr::Attr(key) => row.get_attr(key).cloned()
                .unwrap_or(TypedAttrValue::Null),
            FormulaExpr::Add(a, b) => {
                let va = self.evaluate_formula(a, row).as_f64().unwrap_or(0.0);
                let vb = self.evaluate_formula(b, row).as_f64().unwrap_or(0.0);
                TypedAttrValue::Number(va + vb)
            }
            FormulaExpr::Sub(a, b) => {
                TypedAttrValue::Number(
                    self.evaluate_formula(a, row).as_f64().unwrap_or(0.0) -
                    self.evaluate_formula(b, row).as_f64().unwrap_or(0.0)
                )
            }
            FormulaExpr::Mul(a, b) => { /* ... */ }
            FormulaExpr::Div(a, b) => { /* ... division by zero → Null */ }
            FormulaExpr::If { condition, then_expr, else_expr } => {
                if self.evaluate_bool(condition, row) {
                    self.evaluate_formula(then_expr, row)
                } else {
                    self.evaluate_formula(else_expr, row)
                }
            }
            FormulaExpr::GraphAggregate { edge_type, direction, attr, agg_fn } => {
                // Sum/Avg/Count over neighbor rows via Hypergraph traversal
                let neighbors = self.hypergraph.neighbors(row.d1_key(), edge_type, direction);
                let values: Vec<f64> = neighbors.iter()
                    .filter_map(|n| self.cell_store.get_attr_number(n.coord(), attr))
                    .collect();
                apply_agg_fn(agg_fn, &values)
            }
            FormulaExpr::DimFold { axis_id, agg_fn, source_attr } => {
                // Fold over all keys in a dimension axis
                self.cell_store.fold(row.cube_id(), row.d1_key(), axis_id, agg_fn, source_attr)
            }
            FormulaExpr::Literal(val) => val.clone(),
            FormulaExpr::Null => TypedAttrValue::Null,
        }
    }
}
```

Built-in formula functions available to all domain systems:

| Function | Signature | Description |
|----------|-----------|-------------|
| `SUM(attr, [filter])` | f64 | Sum of attr across specified slice |
| `AVG(attr, [filter])` | f64 | Average of attr |
| `MIN(attr, [filter])` | TypedAttrValue | Minimum value |
| `MAX(attr, [filter])` | TypedAttrValue | Maximum value |
| `COUNT([filter])` | i64 | Row count matching filter |
| `RATIO(a, b)` | f64 | a / b; returns Null if b == 0 |
| `GRAPH_COUNT(edge_type, direction)` | i64 | Count of connected neighbors |
| `GRAPH_SUM(edge_type, direction, attr)` | f64 | Sum of attr over neighbors |
| `GRAPH_WALK(edge_type, depth, agg)` | f64 | Aggregate over graph subtree |
| `PERIOD_DELTA(attr, d3_key_a, d3_key_b)` | f64 | value[period_b] - value[period_a] |
| `LAST_N_PERIODS(attr, n, agg)` | f64 | Aggregate over last N TimeAxis keys |
| `DATEDIFF(date_a, date_b, unit)` | f64 | Difference between two dates |
| `IF(cond, then, else)` | TypedAttrValue | Conditional expression |
| `COALESCE(a, b, ...)` | TypedAttrValue | First non-null value |
| `CONCAT(s1, s2, ...)` | Text | String concatenation |
| `NOW()` | DateTime | Current server time |
| `LOOKUP(cube, d1_key, d2_key)` | TypedAttrValue | Cross-cube cell lookup |

### 8.2 Tier-2: Asynchronous AI/ML Attribute Computation

Tier-2 computations are non-deterministic, potentially expensive, and eventually consistent. They are computed asynchronously by registered AIEnginePlugin implementations and written back via the WritebackService.

```
Tier-2 execution model:

Trigger (on_cell_mutation):
  1. A HyperCell is written (end of §2 write path, Step 8a)
  2. For each registered HypercubePlugin:
     affected_attrs = plugin.on_cell_mutation(cube_id, coord)
     for attr in affected_attrs:
       ai_compute_queue.push(AIComputeRequest {
         plugin_id, attr_key: attr, coord: row_coord(coord),
         priority: compute_priority(attr),  // Critical|High|Normal|Background
         requested_at: Utc::now(),
       })

Compute worker (pool of AI_COMPUTE_WORKER_POOL_SIZE workers):
  loop:
    req = ai_compute_queue.pop_highest_priority()
    
    // Check if result is still valid (cell may have changed again)
    current_ts = cell_store.get_vector_clock(req.coord, req.attr_key)
    if req.obsolete_for(current_ts): continue  // skip stale request
    
    // Build ComputeContext
    row = grid.get_row(req.coord.cube_id, req.coord.d1_key())
    ctx = ComputeContext { row, cube: &grid.cubes[req.coord.cube_id], grid: &grid }
    
    // Invoke plugin
    plugin = plugin_registry.get(req.plugin_id)
    result: TypedAttrValue = plugin.compute_attribute(&req.attr_key, &ctx)?
    // result is TypedAttrValue::AiSignal { value, confidence, model_id, computed_at }
    
    // Write result back via WritebackService
    writeback_service.write_ai_result(WritebackRequest {
      coord:       req.coord,
      attr_key:    req.attr_key,
      value:       result,
      actor:       format!("system:{}", req.plugin_id),
      permission:  PermissionTier::System,  // bypasses user permission check
    })?

Caching:
  AiSignal results are cached in:
    - HyperCell.computed_cache (in-memory per cell, TTL from attr def)
    - Redis: "ai:{cube_id}:{d1_key}:{attr_key}" TTL from staleness_ttl
  On TTL expiry: ai_compute_queue.push(refresh_request) for proactive refresh
  Stale reads: return cached AiSignal with stale_at set; UI indicates "updating"
```

---

## 9. The HyperQL Execution Pipeline

HyperQL queries flow through a five-stage pipeline: parse → analyze → plan → optimize → execute.

### 9.1 Query Lifecycle

```
Stage 1: Parse
  Input: HyperQL string or HyperQLQuery struct (from SDK or NL translation)
  Output: Abstract Syntax Tree (AST)
  Parser: PEG parser (pest crate); grammar defined in hg_ql.pest
  Error: HyperQLSyntaxError with line/column for user-facing display

Stage 2: Analyze (semantic analysis)
  - Resolve cube names to CubeIds
  - Resolve attribute keys against AttributeKeyRegistry; error on unknown keys
  - Resolve dimension axis names to AxisIds
  - Type-check expressions: ensure DimFold applied to numeric attrs only
  - Validate AS_OF timestamp: must be parseable DateTime<Utc>
  - Validate TRAVERSE GRAPH: check edge type exists in Hypergraph schema
  - Check caller PermissionTier against all accessed attr keys
  - Output: Resolved AST (all names replaced with internal IDs)

Stage 3: Plan
  - Generate one or more physical plans from the resolved AST
  - Physical plans: IndexScan | FullScan | DimFoldScan | GraphTraversal | CrossGridJoin | Union
  - Cost estimation per plan using:
    - Cardinality statistics (hypergrid_stats table: row counts, NDV per dim axis)
    - Index availability (which dim keys have BTree / BRIN / GIN indexes)
    - Computed attribute presence (can we push filter to attr_value JSONB?)
    - AS_OF time-travel: adds EventLog replay cost

Stage 4: Optimize
  - Choose lowest-cost plan
  - Apply: predicate pushdown, projection pruning, limit pushdown, index selection
  - Determine routing: PostgreSQL vs ClickHouse (based on plan.row_estimate > threshold)
  - For CrossGridJoin: plan which grid to query first (smaller result set drives)
  - For GraphTraversal: choose BFS or DFS based on depth and expected fan-out

Stage 5: Execute
  PhysicalPlan::IndexScan:
    → cell_store.scan_slice(cube_id, dim_slice, limit, offset)

  PhysicalPlan::DimFoldScan:
    if row_estimate < CLICKHOUSE_THRESHOLD:
      → PostgreSQL: SELECT agg_fn(attr) FROM hypergrid_cells WHERE ... GROUP BY ...
    else:
      → ClickHouse: same query on analytics replica

  PhysicalPlan::GraphTraversal:
    → graph_store.traverse(start_node, edge_type, direction, max_depth, filter)
    → PostgreSQL: recursive CTE  (for graphs < 1M nodes)
    → Neo4j: MATCH (n)-[:EDGE_TYPE*1..depth]→(m) WHERE ...  (for > 1M nodes)

  PhysicalPlan::CrossGridJoin:
    → Execute primary cube scan locally
    → For each result row: resolve CrossGridLink edges from Hypergraph
    → Fan out to target Grid via FederationManager.remote_query(target_grid, sub_query)
    → Merge results: local join in memory

  PhysicalPlan::AsOf:
    → event_log.replay(cube_id, d1_key, until: timestamp)
    → assemble materialized state at timestamp
    → apply remaining query filters to materialized state

  Result assembly:
    → Apply VisibilityMask filter to result rows and columns
    → Apply projection (SELECT clause)
    → Apply ORDER BY (in memory if small result; PostgreSQL ORDER BY if large)
    → Apply LIMIT / OFFSET
    → Return QueryResult { rows: Vec<QueryRow>, total_count, plan_info }
```

### 9.2 HyperQL Grammar Summary

```
Query ::= SELECT Projections FROM CubeName
          [AS_OF timestamp]
          [WHERE Predicates]
          [FOLD DimName WITH AggFn [AS alias]]
          [EXPAND DimName AS COLUMNS(AggFn)]
          [TRAVERSE GRAPH FROM NodeRef EDGE_TYPE=EdgeType DIRECTION=Dir [MAX_DEPTH=n]]
          [JOIN CubeName ON GRAPH_EDGE(source, target, EdgeType) [AS alias]]
          [SHADOW INCLUDE FROM GridRef]
          [ORDER BY SortKey [ASC|DESC]]
          [LIMIT n] [OFFSET n]

Projections ::= * | ProjectionItem [, ProjectionItem]*
ProjectionItem ::= CellRef [AS alias] | DimRef | EXPAND DimName | FOLD DimName | AggExpr

CellRef ::= cell[D1_ref, D2_ref [, D3_ref [, D4_ref]]].attr_key
           | cell[D1_ref, "field_name"].value
           | D1_ref.entity_id | D2_ref.field_name | D3_ref.period

Predicates ::= Predicate [AND|OR Predicate]*
Predicate ::= CellRef CompOp Value
            | CellRef IN (Values)
            | CellRef CONTAINS Value        -- OR-Set contains element
            | DimRef BETWEEN Value AND Value -- range on TimeAxis, etc.
            | GRAPH_EDGE(a, b, EdgeType)    -- join predicate
            | IS NULL | IS NOT NULL

AggFn ::= AVG | SUM | MIN | MAX | COUNT | LAST | FIRST | MEDIAN | STDDEV
        | PERCENTILE(n) | ARRAY_AGG

EdgeType ::= Hierarchy | Dependency | Association | Contains | CrossGridLink
           | Collaborates | Employs | InvestedIn | FederationPeer | Custom(name)

Direction ::= Inbound | Outbound | Both
```

---

## 10. The Federation Sync Protocol

### 10.1 Delta Sync Algorithm

```
Federation sync runs on a 30-second heartbeat between all active peers.
Each node maintains a per-peer sync cursor: the VectorClock state known
to have been delivered to that peer.

Sender side (this node → peer):
  peer_cursor = federation.get_cursor(peer.node_id)
  delta_ops = crdt_log.query_since(peer_cursor)
  // Query: SELECT * FROM crdt_log
  //        WHERE created_at > peer_cursor.timestamp
  //        OR any(node, ts: clock[node] > peer_cursor[node])
  //        ORDER BY causal_ts ASC

  if delta_ops.is_empty(): send Heartbeat { vc: self.vector_clock }; return

  kafka_producer.send(
    topic: "crdt.delta.{grid_id}",
    key:   peer.node_id,
    value: CrdtDelta {
      from_vc:  peer_cursor.vector_clock,
      to_vc:    self.vector_clock.snapshot(),
      ops:      delta_ops,
      checksum: sha256(ops),
    }
  )

Receiver side (peer → this node):
  delta = kafka_consumer.poll("crdt.delta.{grid_id}", key=self.node_id)

  // Verify integrity
  assert sha256(delta.ops) == delta.checksum

  // Apply in causal order
  ops_sorted = delta.ops.sort_by(|a, b| a.vector_clock.happened_before(&b.vector_clock))
  for op in ops_sorted:
    result = merge_engine.apply_crdt_op(&op, &mut cell_store, &mut event_log)?
    match result:
      MergeResult::Applied          → update sync cursor
      MergeResult::ConflictRecorded → conflict visible in SHT-029 (Event Log sheet)
      MergeResult::DiscardedStale   → already applied (idempotent); update cursor

  // Advance VectorClock to reflect received state
  self.vector_clock.merge(&delta.to_vc)

  // Acknowledge to Kafka (at-least-once delivery)
  kafka_consumer.commit()

  // Send Ack to sender: confirm which ops were applied
  kafka_producer.send("crdt.ack.{grid_id}", key=delta.from_node_id,
    value: CrdtAck { applied_up_to: delta.to_vc })
```

### 10.2 Conflict Resolution Policy

```
When two concurrent writes conflict on an LWW field:
  1. merge_engine detects: timestamp_a.concurrent_with(timestamp_b)
  2. Deterministic tiebreak: lexicographic comparison of ActorId strings
     Winner: max(actor_a, actor_b) lexicographically
  3. Losing write recorded: EventLog.ConflictRecord { winner, loser, attr, timestamp }
  4. ConflictRecord visible to both parties in SHT-029 (Event Log sheet)
  5. Oba (Kogi) / system notification: "Conflict detected on {attr}: {other_actor}'s
     write won. Your version: {loser.value}. Current: {winner.value}. Accept or override?"
  6. Actor may override: submit a new SetAttr with a fresh VectorClock
     The new write will dominate (newer timestamp) and win LWW

For Lattice fields (lifecycle states):
  Concurrent transitions resolved by LUB — no notification needed.
  The result is always a valid state. No conflict record generated.

For OR-Set fields:
  No conflicts possible. All concurrent adds survive.
  Concurrent remove+add: add wins (the un-removed entry survives).
```

---

## 11. The Plugin Execution Model

### 11.1 Plugin Lifecycle

```
Plugin registration (at Grid startup):
  plugin = SomePlugin::new(config)?
  plugin.on_load(&PluginContext {
    grid_id:    self.grid_id,
    cube_store: &self.cube_store,
    config:     &plugin_config,
  })?
  plugin_registry.register(plugin)

  // For each AttributeKeyDef returned by plugin.attribute_key_defs():
  //   cube.attr_registry.register(attr_def)
  // This enables hot-swapping Domain Packs at runtime:
  //   add pack → new attrs available immediately on next read

Plugin health monitoring (30-second check):
  for plugin in plugin_registry.all():
    health = plugin.health_check()
    match health:
      PluginHealth::Healthy → metrics.gauge("plugin.health", 1.0, plugin_id)
      PluginHealth::Degraded { reason } → log WARN; metrics.gauge(..., 0.5)
      PluginHealth::Unhealthy { reason } → log ERROR; metrics.gauge(..., 0.0)
        → if plugin.required(): grid enters Degraded status; alert

Plugin hot-swap (zero-downtime Domain Pack update):
  // Step 1: Load new plugin version
  new_plugin = NewPluginVersion::new(new_config)?
  new_plugin.on_load(&ctx)?

  // Step 2: Register new attrs (additive; CRDT-commutative)
  for new_attr in new_plugin.attribute_key_defs().diff(old_plugin.attribute_key_defs()):
    cube.attr_registry.register(new_attr)

  // Step 3: Drain in-flight compute requests for old plugin
  ai_compute_queue.drain_plugin(old_plugin_id)

  // Step 4: Swap plugin in registry (atomic pointer swap)
  plugin_registry.swap(old_plugin_id, Arc::new(new_plugin))

  // Step 5: Unload old plugin
  old_plugin.on_unload()?

  // No downtime: read path falls back to stale cache during swap;
  // new plugin handles all compute requests after atomic swap
```

### 11.2 ComputedModelPlugin Execution Detail

```rust
pub trait ComputedModelPlugin: HypercubePlugin {
    // Which attrs this plugin computes
    fn output_attrs(&self) -> Vec<AttributeKeyDef>;

    // Which events trigger recomputation
    fn invalidation_triggers(&self) -> Vec<EventKind>;

    // Cache TTL for this model's outputs
    fn cache_ttl(&self) -> Duration;

    // The computation itself
    fn compute(&self, row: &HyperRow, ctx: &ComputeContext) -> ComputeResult;
}

// ComputeContext gives the plugin everything it needs:
pub struct ComputeContext<'a> {
    pub row:          &'a HyperRow,          // all current attrs of the target row
    pub cube:         &'a Hypercube,          // cube schema, attr registry
    pub grid:         &'a Grid,               // access to other cubes, graph, etc.
    pub neighbors:    NeighborCache<'a>,      // lazy-loaded Hypergraph neighbors
    pub time_series:  TimeSeriesCache<'a>,    // lazy-loaded D₃ TimeAxis slices
    pub external:     ExternalDataCache<'a>,  // cached external API responses
}

// Example: KogiHealthScoreEngine
impl ComputedModelPlugin for KogiHealthScoreEngine {
    fn output_attrs(&self) -> Vec<AttributeKeyDef> {
        vec![attr_def("health_score", AttributeType::Ai, CrdtSemantics::LastWriteWins)]
    }

    fn invalidation_triggers(&self) -> Vec<EventKind> {
        vec![
            EventKind::CellMutation { attr_key: "status".into() },
            EventKind::CellMutation { attr_key: "budget_spent".into() },
            EventKind::CellMutation { attr_key: "risks".into() },
            EventKind::GraphEdgeAdded { edge_type: EdgeType::Dependency },
        ]
    }

    fn cache_ttl(&self) -> Duration { Duration::from_secs(3600) }  // 1 hour

    fn compute(&self, row: &HyperRow, ctx: &ComputeContext) -> ComputeResult {
        let status = row.get_str("status").unwrap_or("Draft");
        let budget_util = row.get_f64("budget_spent").unwrap_or(0.0)
            / row.get_f64("budget").unwrap_or(1.0).max(1.0);
        let risk_count = row.get_json_array_len("risks").unwrap_or(0);
        let dep_count = ctx.neighbors.count(EdgeType::Dependency, EdgeDirection::Inbound);

        let health = match status {
            "Active" => 50.0
                + (1.0 - budget_util).max(0.0) * 30.0
                - (risk_count as f64 * 5.0).min(30.0)
                - (dep_count as f64 * 2.0).min(10.0),
            "Completed" => 90.0,
            "Archived"  => 70.0,
            "Paused"    => 40.0,
            _           => 20.0,  // Draft, Blocked
        }.clamp(0.0, 100.0);

        ComputeResult::Ok(TypedAttrValue::AiSignal {
            value:       Box::new(TypedAttrValue::Number(health)),
            model_id:    "kogi.health_score.v2".into(),
            computed_at: Utc::now(),
            confidence:  0.87,
            explanation: Some(format!(
                "status={status}, budget_util={:.0}%, risks={risk_count}, deps={dep_count}",
                budget_util * 100.0
            )),
        })
    }
}
```

---

## 12. Schema Evolution Execution

| Operation | Safety | Protocol | Downtime |
|-----------|--------|----------|----------|
| Add new DimensionAxis (Dₙ₊₁) | Safe — additive | Register axis in cube schema. Existing cells gain new dim with null/default key. | None |
| Add new AttributeKeyDef | Safe — additive | Register new key in AttributeKeyRegistry. Existing cells return default_value for new key. No migration. | None |
| Expand enum variant set | Safe — additive | Add variant to EnumDef. Old values remain valid. | None |
| Rename attribute key | Safe with alias | Add alias old→new. Old key readable; writes go to new key. Remove alias after migration period. | None |
| Change CRDT semantics | Risky | Schema version bump. New semantics apply only to mutations after version change. No retroactive merge. | None |
| Remove attribute key | Destructive | GovernanceProposal vote required. Key tombstoned (not deleted from EventLog). Historical AS_OF queries still work. | None post-approval |
| Remove DimensionAxis | Destructive | GovernanceProposal vote. All cells on removed axis tombstoned. | Brief recommended |
| Change axis KeyType | Breaking | Add new axis; background migration job; dual-write window; FeatureFlagController | Migration window |
# Part II — Apapo Computational Models

## 13. The Dual-Write Event Architecture

Every domain system in Apapo (Kogi, Ume, Qala) uses the same dual-write event architecture. Every mutation produces two event streams simultaneously:

```
Domain Event Log (typed, domain-specific):
  Purpose: efficient domain-level queries ("all events for component X")
  Storage: in-memory VecDeque (capped at DOMAIN_LOG_CAP = 10,000) + PostgreSQL
  Format:  PortfolioEvent { event_kind: PortfolioEventKind, component_id, actor, ... }
           OrgModuleEvent { event_kind: OrgModuleEventKind, module_id, actor, ... }
           SolutionEvent  { event_kind: SolutionEventKind,  solution_id, actor, ... }

Hypergrid EventLog (universal, substrate-level):
  Purpose: immutable audit trail, AS_OF time-travel, AI feature streaming
  Storage: PostgreSQL event_log table (append-only) + S3 archive
  Format:  EventEntry { event_id, cube_id, dim_keys, attr_key,
                        before_value, after_value, crdt_op, vector_clock, actor }
```

The dual-write is atomic — both writes occur in the same execution block before any async fan-out:

```rust
// Executed synchronously in the write path (§2, Steps 6-7):
fn dual_write_event(&mut self, domain_event: DomainEvent, hg_entry: EventEntry)
    -> Result<(), EventError>
{
    // 1. Write to domain log (fast: in-memory push + async PG flush)
    self.domain_event_log.push(domain_event)?;

    // 2. Write to Hypergrid EventLog as Custom(domain_tag)
    //    Tag format: "{domain}:{event_kind}"
    //    Example: "portfolio:ComponentStatusChanged"
    let hg_entry = hg_entry.with_kind(EventKind::Custom(
        format!("{}:{}", self.domain_prefix, domain_event.kind_tag())
    ));
    self.event_log.append(hg_entry)?;

    // Both writes committed before returning to caller.
    // Async fan-out (Kafka, WebSocket, AI queue) happens after return.
    Ok(())
}
```

This means:
- Any `AS_OF` time-travel query on the Hypergrid EventLog shows **all** domain events (portfolio changes, org module updates, solution lifecycle transitions) as `EventKind::Custom` entries with the domain tag
- A compliance auditor can query the single Hypergrid EventLog across all three domain systems for a unified timeline without any data joining
- kogi-engine, Ume AI, and Qala AI Agent all subscribe to different `events.audit.{grid_id}` Kafka partitions and filter by domain tag

---

## 14. The AI Computation Pipeline: kogi-engine, Ume AI, Qala AI Agent

Each domain system has its own AI intelligence engine. All three share the same execution architecture — they are Kafka consumers that subscribe to EventLog streams, compute signals, and write back via WritebackService.

```
┌─────────────────────────────────────────────────────────────────────┐
│  Kafka Topic: events.audit.{grid_id}  (partitioned by component_id) │
└─────┬────────────────────────────────────────────────────────────────┘
      │
      ├─→ kogi-engine  (Scala 3; consumes portfolio.* events)
      │     12 sub-engines in parallel per partition
      │     Output → WritebackService → kogi.portfolio.kpis
      │
      ├─→ ume-engine   (Go; consumes org module events)
      │     8 AI capabilities in goroutine pool
      │     Output → WritebackService → ume.kernel.modules, ume.analytics.*
      │
      └─→ qala-engine  (Rust; consumes solution/sde/ccr events)
            10 AI capabilities via async task pool
            Output → WritebackService → qala.solutions, qala.sdes, qala.ai_recommendations
```

### 14.1 kogi-engine Execution Model (Scala 3)

```scala
// kogi-engine: Akka Streams pipeline
object KogiEngine extends App {
  val system = ActorSystem("kogi-engine")
  
  // Source: Kafka event stream, partitioned by component_id
  val kafkaSource = Consumer.committableSource(
    consumerSettings,
    Subscriptions.topics("events.audit.kogi-production")
  )

  // Router: dispatch to correct sub-engine based on event kind
  val engineRouter: Flow[EventRecord, ComputeRequest, NotUsed] =
    Flow[EventRecord].mapConcat { record =>
      val event = decode[PortfolioEvent](record.value)
      val requests = mutable.ListBuffer[ComputeRequest]()
      
      // KogiHealthScoreEngine: triggered by status, budget, risk changes
      if (event.attr_key.exists(Set("status", "budget_spent", "risks").contains))
        requests += ComputeRequest(KogiHealthScoreEngine, event.component_id)
      
      // KogiRiskEngine: triggered by risk record or deadline changes
      if (event.attr_key.exists(k => k == "risks" || k == "due_date"))
        requests += ComputeRequest(KogiRiskEngine, event.component_id)
      
      // KogiIncomeProjectionEngine: triggered by gig/contract changes
      if (event.event_kind == "GigUpdated" || event.event_kind == "ContractUpdated")
        requests += ComputeRequest(KogiIncomeProjectionEngine, event.component_id)
      
      // ... (all 12 sub-engines)
      requests.toList
    }
    .deduplicate(within = 5.seconds)  // collapse rapid successive mutations

  // Compute: invoke the specific sub-engine for each request
  val computeStage: Flow[ComputeRequest, WritebackRequest, NotUsed] =
    Flow[ComputeRequest]
      .mapAsyncUnordered(parallelism = 32) { req =>
        Future {
          val row = portfolioService.getRow(req.componentId)
          val result = req.engine.compute(row)
          WritebackRequest(
            coord    = Coord(req.componentId, req.engine.outputAttr),
            value    = result.toTypedAttrValue,
            actor    = s"system:${req.engine.engineId}",
            cubeId   = KOGI_PORTFOLIO_KPIS_CUBE_ID,
          )
        }
      }

  // Sink: write results via WritebackService gRPC
  val writebackSink: Sink[WritebackRequest, Future[Done]] =
    Sink.foreachAsync(parallelism = 16) { req =>
      writebackServiceStub.writeAiResult(req).toScala
    }

  // Wire the pipeline
  kafkaSource
    .map(_.record)
    .via(engineRouter)
    .via(computeStage)
    .alsoTo(writebackSink)
    .toMat(Consumer.committerSink(committerSettings))(Keep.right)
    .run()
}
```

### 14.2 WritebackService Protocol

The WritebackService is a gRPC service that is the **only** path through which AI-computed values enter the cell store. It enforces System PermissionTier and full audit trail regardless of which AI engine is calling.

```protobuf
service WritebackService {
  rpc WriteAiResult(WritebackRequest) returns (WritebackResponse);
  rpc WriteBatch(WriteBatchRequest) returns (WriteBatchResponse);
  rpc GetComputeStatus(ComputeStatusRequest) returns (ComputeStatusResponse);
}

message WritebackRequest {
  string  grid_id    = 1;
  string  cube_id    = 2;
  bytes   dim_keys   = 3;    // serialized DimCoordinate
  string  attr_key   = 4;
  bytes   value      = 5;    // serialized TypedAttrValue (AiSignal)
  string  actor      = 6;    // "system:{engine_id}" — must match registered AI engine
  string  engine_id  = 7;    // verified against plugin_registry
  double  confidence = 8;    // 0.0–1.0
  string  model_id   = 9;
  int64   computed_at_unix = 10;
}
```

WritebackService execution on receiving a WritebackRequest:

```
1. Authentication: verify mTLS client cert matches AI engine identity
2. Actor validation: actor MUST match "system:{engine_id}" format
3. Engine authorization: engine_id MUST be in plugin_registry AND be a registered AIEnginePlugin
4. PermissionTier override: write_permission check uses PermissionTier::System (bypasses user check)
   BUT: verifies attr_def.write_permission == System (i.e., the attr IS declared AI-write-only)
   If attr is NOT System-write-only: reject with WritebackError::AttrNotAiWritable
5. CRDT construction: wrap in CrdtOperation::SetAttr with System-level VectorClock
6. Apply via merge_engine (same write path as §2)
7. EventLog entry: marks actor as "system:{engine_id}" for full audit attribution
8. Return WritebackResponse { success, applied_at, conflict_info? }
```

---

## 15. The CrossGridLink Sync Engine

The CrossGridLink Sync Engine runs within the FederationManager service. It manages the lifecycle of all shadow cells and mirror attributes across linked Grids.

```
FederationManager owns:
  active_links: HashMap<CrossGridLinkId, CrossGridLink>
  shadow_cells: HashMap<ShadowCellId, ShadowCell>
  sync_cursors: HashMap<CrossGridLinkId, SyncCursor>  // per-link EventLog cursor
  consent_store: ConsentStore  // pending/accepted/revoked consent records

On CrossGridLink establishment (after consent acceptance):
  1. Create ShadowCell record in host Grid (Grid A):
     shadow_cell = ShadowCell {
       shadow_id:       Uuid::new_v4(),
       source_grid_id:  link.target_grid_id,
       source_d1_key:   link.target_entity_id,
       source_edge_id:  link.edge_id,
       mirrored_attrs:  link.consent.mirrored_attrs.clone(),
       writeback_attrs: link.consent.writeback_attrs.clone(),
       update_policy:   link.consent.update_policy,  // RealTime | Batch
       shadow_status:   ShadowStatus::Active,
     }
     grid_a.cell_store.insert_shadow_cell(shadow_cell)

  2. Initial snapshot sync: read current values of mirrored_attrs from Grid B
     for attr in shadow_cell.mirrored_attrs:
       value = federation_manager.remote_read(target_grid_id, source_d1_key, attr)
       shadow_cell.synced_values.insert(attr, value)
     grid_a.cell_store.update_shadow_values(shadow_cell.shadow_id, shadow_cell.synced_values)

  3. Set sync cursor to current Grid B EventLog position
     sync_cursors[link.id] = SyncCursor { vector_clock: grid_b_current_vc }

Ongoing sync (real-time mode):
  Grid B write path (Step 8d) publishes to "shadow.sync.{grid_b_id}" Kafka topic
  FederationManager (Grid A side) consumes topic, filters for linked entity_ids
  
  For each relevant event:
    if event.attr_key in shadow_cell.mirrored_attrs:
      // Apply to shadow cell in Grid A (LWW — shadow always accepts Grid B's value)
      grid_a.cell_store.update_shadow_attr(
        shadow_cell_id: shadow_cell.shadow_id,
        attr_key:       event.attr_key,
        value:          event.after_value,
        source_ts:      event.vector_clock,
      )
      // Emit ShadowCellUpdated event to Grid A's EventLog
      grid_a.event_log.append(EventEntry::shadow_updated(shadow_cell.shadow_id, event))

Write-back path (Grid A → Grid B):
  User in Grid A writes to shadow_cell.writeback_attr:
    1. WritebackService validates: attr in shadow_cell.writeback_attrs?
    2. If yes: federate the write to Grid B
       federation_manager.remote_write(target_grid_id, source_d1_key, attr, value, actor_a)
    3. Grid B applies the write as normal (from actor_a, with Grid A attribution)
    4. Grid B's EventLog records the write from actor_a
    5. Grid B's sync loop echoes the change back to Grid A's shadow cell (via real-time sync)

Batch mode sync (update_policy = Batch):
  Scheduled job (configurable: every 5min / 1hr / daily)
  Reads Grid B EventLog for all mirrored events since last cursor
  Bulk-applies all changes to shadow cells in Grid A
  Updates sync_cursor to current Grid B VectorClock
```

---

## 16. The NL-to-HyperQL Translation Model

The NL-to-HyperQL translation model enables users to query their domain data in natural language. It uses a three-stage pipeline: intent classification → slot filling → HyperQL generation.

```
Input: "Show me all my active projects where budget utilization is over 80%"
Context: cube=kogi.portfolio.components, caller=@alice, domain=kogi

Stage 1: Intent Classification
  LLM prompt:
    "You are a HyperQL query builder for the Kogi portfolio system.
     The user is querying: {input}
     Available cubes: {cube_summary}
     Identify the query intent:
     - FILTER_AND_SELECT: filter rows by conditions, select attributes
     - AGGREGATE: compute aggregate over rows (COUNT, SUM, AVG, etc.)
     - GRAPH_TRAVERSE: follow relationships between entities
     - TIME_TRAVEL: query state at a past point in time
     - CROSS_SYSTEM: involve multiple grids
     Intent: ..."
  
  Result: FILTER_AND_SELECT

Stage 2: Slot Filling
  LLM prompt:
    "For a FILTER_AND_SELECT query on kogi.portfolio.components:
     User said: 'active projects where budget utilization > 80%'
     
     Extract:
     - Target entity type: Project
     - Status filter: Active
     - Computed condition: budget_spent / budget > 0.80
     - Attributes to select: name, status, budget, budget_spent, health_score
     - Order by: (none specified)
     - Limit: (none specified)
     
     Map to attribute keys (from registry):
     - 'status' → D₂='status', value='"Active"'
     - 'budget utilization > 80%' → Tier-1 formula: budget_spent/budget > 0.80
       → split into: budget_spent attr AND budget attr with ratio condition
     Output slots: ..."
  
  Result: {
    entity_type: "Project",
    status_filter: "Active",
    attr_conditions: [(budget_spent, budget) -> ratio > 0.80],
    select_attrs: ["name", "status", "budget", "budget_spent", "health_score"],
  }

Stage 3: HyperQL Generation
  Template engine maps slots to HyperQL AST:
  
  SELECT cell[D₁, "name"].value AS name,
         cell[D₁, "status"].value AS status,
         cell[D₁, "budget"].value AS budget,
         cell[D₁, "budget_spent"].value AS budget_spent,
         cell[D₁, D₂, "health_score", CURRENT_PERIOD].value AS health_score,
         (cell[D₁, "budget_spent"].value / cell[D₁, "budget"].value) AS budget_util_pct
  FROM kogi.portfolio.components
  WHERE cell[D₁, "category"].value = '"{"Item":"Project"}"'
    AND cell[D₁, "status"].value = '"Active"'
    AND (cell[D₁, "budget_spent"].value / cell[D₁, "budget"].value) > 0.80
  ORDER BY budget_util_pct DESC
  
  // LLM also generates human-readable explanation for display alongside results:
  explanation: "Showing your active projects with budget utilization above 80%,
                ordered by highest utilization first."

Stage 4: Validation (before execution)
  - Validate generated HyperQL AST (parse + analyze stages)
  - If parse error: retry with error feedback in LLM prompt (max 2 retries)
  - If analysis error (unknown attr): use LLM to fix attribute name mapping
  - If valid: execute and return results + explanation
```

---

## 17. The Anomaly Detection Computational Model

The AnomalyEngine plugin monitors numeric attribute cells across all Hypercubes and fires AnomalySignal cells when values deviate significantly from established baselines.

```
AnomalyEngine initialization:
  For each monitored (cube_id, attr_key) pair:
    Subscribe to EventLog stream: filter by cube_id AND attr_key
    Maintain per-entity rolling baseline:
      baseline = ExponentialMovingAverage(window=30_days, alpha=0.1)
      stddev   = ExponentialMovingStdDev(window=30_days, alpha=0.1)

On receiving a new value V for entity E, attribute A:
  z_score = abs(V - baseline[E][A]) / max(stddev[E][A], EPSILON)
  
  if z_score > ANOMALY_THRESHOLD (default: 2.0):
    kind = classify_anomaly(V, baseline[E][A], history[E][A]):
      SuddenChange   → large immediate jump (>3σ, instant)
      PatternBreak   → sustained deviation from rolling average
      OutlierValue   → single extreme value (isolated)
      StateChange    → categorical field changed unexpectedly
    
    anomaly_signal = TypedAttrValue::AnomalySignal {
      kind,
      severity:  z_score / ANOMALY_THRESHOLD,  // 1.0 = threshold, >1 = worse
      baseline:  TypedAttrValue::Number(baseline[E][A]),
      deviation: V - baseline[E][A],
    }
    
    // Write anomaly flag to cell
    writeback_service.write_ai_result(WritebackRequest {
      coord:     coord(E, "anomaly_flag"),
      attr_key:  "anomaly_flag",
      value:     anomaly_signal,
      actor:     "system:anomaly_engine",
      engine_id: ANOMALY_ENGINE_ID,
      confidence: min(z_score / 4.0, 1.0),
    })
    
    // Publish to notification channel
    notification_service.publish(AnomalyAlert {
      entity_id: E, attr: A, z_score, kind, value: V, baseline: baseline[E][A]
    })
  
  // Always update baseline (whether anomaly or not)
  baseline[E][A] = ema_update(baseline[E][A], V)
  stddev[E][A]   = emstddev_update(stddev[E][A], V, baseline[E][A])
```

Anomaly detection subscriptions by domain system:

| Domain System | Monitored Attributes | Anomaly Kind | Action |
|--------------|---------------------|--------------|--------|
| Kogi | `budget_spent` rate | SuddenChange | Alert portfolio owner via Oba |
| Kogi | `health_score` | PatternBreak | Trigger Oba recommendation |
| Kogi | `income_projection_90d` | SuddenChange | Alert worker |
| Ume | `finance.balance` | OutlierValue | Alert Finance module admin |
| Ume | `module.restart_count` | SuddenChange | Trigger supervisor review |
| Ume | `hr.attrition_risk_score` | PatternBreak | Alert HR module |
| Qala | `defect_density` | PatternBreak | Alert release manager |
| Qala | `sde.drift_status` | StateChange | Trigger drift remediation |
| Qala | `solutions.quality_score` | SuddenChange | Alert factory admin |

---

## 18. The Lattice CRDT State Machine

The Lattice CRDT is the most important CRDT for domain systems because it encodes governance invariants directly in the data layer. Every lifecycle state field in every domain system should use Lattice semantics in v2.

### 18.1 Lattice Definition

```rust
pub struct LatticeOrder {
    // Nodes: all valid states
    pub nodes: Vec<LatticeNode>,
    // The partial order is encoded as a DAG:
    // LatticeNode.successors = valid forward transitions from this state
}

pub struct LatticeNode {
    pub state:      String,
    pub successors: Vec<String>,  // which states this can transition TO
    pub predecessors: Vec<String>, // computed: which states can transition TO this
    pub is_terminal: bool,        // no successors — final state
}

impl LatticeOrder {
    /// Join = least upper bound in the partial order.
    /// Returns the "most advanced" state that is reachable from both inputs.
    /// This is what gets called on concurrent transitions.
    pub fn join(&self, a: &str, b: &str) -> String {
        if a == b { return a.to_owned(); }
        // If a is reachable from b: b is more advanced → return b
        if self.can_reach(b, a) { return b.to_owned(); }
        // If b is reachable from a: a is more advanced → return a
        if self.can_reach(a, b) { return a.to_owned(); }
        // Neither dominates: find LUB (lowest common ancestor in reverse direction)
        // This is the state reachable from both that is closest to both
        self.least_upper_bound(a, b)
            .unwrap_or_else(|| a.to_owned())  // fallback: keep current (safer)
    }

    pub fn is_valid_transition(&self, from: &str, to: &str) -> bool {
        self.nodes.iter()
            .find(|n| n.state == from)
            .map(|n| n.successors.contains(&to.to_owned()))
            .unwrap_or(false)
    }

    pub fn successors(&self, state: &str) -> Vec<String> {
        self.nodes.iter()
            .find(|n| n.state == state)
            .map(|n| n.successors.clone())
            .unwrap_or_default()
    }

    fn can_reach(&self, from: &str, to: &str) -> bool {
        // BFS: can we reach `to` by following successors from `from`?
        let mut visited = HashSet::new();
        let mut queue = VecDeque::from([from.to_owned()]);
        while let Some(current) = queue.pop_front() {
            if current == to { return true; }
            if visited.insert(current.clone()) {
                for successor in self.successors(&current) {
                    queue.push_back(successor);
                }
            }
        }
        false
    }
}
```

### 18.2 Lattice Definitions for Each Domain System

**Kogi — ComponentStatus lattice:**
```
Draft ──→ Active ──→ Completed ──→ Archived
  │                      │
  └──→ Paused ────────────┘
         │
         └──→ Archived

join("Draft", "Active")    = "Active"
join("Active", "Paused")   = "Active" (Active is reachable from both, and Paused from Active)
                           Actually: Active and Paused are concurrent → LUB depends on topology
join("Completed", "Draft") = "Completed" (Completed is forward from Draft)
```

**Ume — ModuleLifecycleState lattice:**
```
Registered → Starting → Running ←→ Degraded → Stopped
                           ↑
                        Recovering
```

**Qala — SolutionLifecycleState lattice:**
```
Draft → InReview → Approved → Active → Deprecated → Retired
           ↑___________↓ (can regress InReview→Draft under governance)
```

**Qala — CcrStatus lattice:**
```
Draft → Submitted → UnderReview → Approved → Implemented → Closed
                         │
                         └──→ Rejected → Closed
```

### 18.3 Admin Override: Forcing Regression

In rare cases (error correction, emergency rollback), a system administrator needs to force a state regression that the Lattice CRDT would normally prevent:

```rust
// Admin override: force-set a state regardless of lattice
// Requires: actor.tier == Admin (PermissionTier::Admin)
// AND: explicit override_lattice = true flag in the request
grid.write_cell_with_override(
    cube_id, coord, "lifecycle_state",
    TypedAttrValue::Text("Draft".into()),  // regression
    actor,
    WriteOptions { override_lattice: true, require_admin: true }
)?

// This generates a special EventLog entry:
// EventEntry { crdt_op: "LatticeAdminOverride", before: "Active", after: "Draft" }
// And a GovernanceRecord: { reason: required, approved_by: admin_id }
// The regression is permanent and visible in audit trail
```

---

# Part III — Using Hypergrid in Practice

## 20. Setting Up a Grid

### 20.1 Rust (Core)

```rust
use hypergrid::{Grid, GridConfig, DomainSystem};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = GridConfig {
        grid_id:       Uuid::new_v4(),
        node_id:       "node-us-east-1".into(),
        name:          "my-domain-system".into(),
        domain_system: DomainSystem::Standalone,
        
        storage: StorageConfig {
            primary:   "postgresql://user:pass@localhost/hypergrid".into(),
            redis:     "redis://localhost:6379".into(),
            kafka:     "localhost:9092".into(),
            s3_bucket: Some("hypergrid-events".into()),
        },
        
        ai: AiConfig {
            mode: AiMode::OpenAiCompatible {
                base_url: "https://api.openai.com/v1".into(),
                api_key:  std::env::var("OPENAI_API_KEY")?,
            },
            worker_pool_size: 8,
        },
        
        federation: FederationConfig {
            enabled:            false,  // start without federation
            heartbeat_secs:     30,
            max_peers:          255,
        },
        
        ..GridConfig::default()
    };
    
    let mut grid = Grid::new(config).await?;
    
    // Register your domain system's Hypercubes (idempotent)
    MyDomainStore::bootstrap(&mut grid).await?;
    
    // Grid is now ready to use
    println!("Grid ready: {} cubes registered", grid.cube_count());
    
    Ok(())
}
```

### 20.2 Go (Services layer)

```go
package main

import (
    "github.com/apapo/hypergrid-go/grid"
    "github.com/apapo/hypergrid-go/config"
)

func main() {
    cfg := config.Grid{
        GridID:     uuid.New(),
        NodeID:     "node-us-east-1",
        Name:       "my-service",
        PostgresURL: os.Getenv("POSTGRES_URL"),
        RedisURL:    os.Getenv("REDIS_URL"),
        KafkaBrokers: []string{os.Getenv("KAFKA_BROKERS")},
    }
    
    g, err := grid.New(ctx, cfg)
    if err != nil { log.Fatal(err) }
    defer g.Close()
    
    // Use grid via gRPC to spreadsheet-service
    // Go services typically talk to the Rust core via gRPC
    client := spreadsheetv1.NewSpreadsheetServiceClient(conn)
}
```

---

## 21. Defining Hypercubes and Schemas

```rust
// Step 1: Create the Hypercube
let cube_id = grid.create_cube(CreateCubeRequest {
    name:     "myapp.products",  // {domain}.{entity_type} convention
    dims:     vec![
        DimensionAxis::entity("products"),    // D₁: EntityAxis for product IDs
        DimensionAxis::property("fields"),    // D₂: PropertyAxis for field names
        DimensionAxis::time("history"),       // D₃: TimeAxis for time-series (optional)
    ],
    encoding: StorageEncoding::Hybrid {
        dense_dims: vec![1, 2],  // D₁×D₂ dense; D₃ sparse
    },
    visibility: CubeVisibility::Tenant,
})?;

// Step 2: Register attribute keys (the column schema)
let cube = grid.get_cube_mut(cube_id)?;

cube.attr_registry.register_batch(vec![
    // Text fields (LWW)
    AttributeKeyDef::text("name",        "Product Name",       PermissionTier::Editor),
    AttributeKeyDef::text("description", "Description",        PermissionTier::Editor),
    AttributeKeyDef::text("sku",         "SKU",                PermissionTier::Manager),
    AttributeKeyDef::text("status",      "Status",             PermissionTier::Manager),
    
    // Numeric fields
    AttributeKeyDef::number("price",       "Price",       PermissionTier::Manager),
    AttributeKeyDef::number("stock_count", "Stock Count", PermissionTier::Editor)
        .with_crdt(CrdtSemantics::PNCounter),  // stock is a transactional counter
    
    // OR-Set fields (sets that survive concurrent modification)
    AttributeKeyDef::json("tags",       "Tags",       PermissionTier::Contributor)
        .with_crdt(CrdtSemantics::OrSet),
    AttributeKeyDef::json("category_ids", "Categories", PermissionTier::Editor)
        .with_crdt(CrdtSemantics::OrSet),
    
    // Lifecycle state (Lattice CRDT in v2)
    AttributeKeyDef::json("lifecycle",  "Lifecycle",  PermissionTier::Manager)
        .with_crdt(CrdtSemantics::Lattice(ProductLifecycleLattice::new())),
    
    // AI-computed fields (System write only)
    AttributeKeyDef::ai("demand_forecast",   "Demand Forecast",    FORECAST_ENGINE_ID),
    AttributeKeyDef::ai("reorder_score",     "Reorder Score",      REORDER_ENGINE_ID),
    AttributeKeyDef::ai("anomaly_flag",      "Anomaly Flag",       ANOMALY_ENGINE_ID),
    
    // Tier-1 formula field (synchronous, always fresh)
    AttributeKeyDef::formula("revenue_ytd", "Revenue YTD",
        FormulaExpr::Mul(
            FormulaExpr::attr("price"),
            FormulaExpr::DimFold { axis: D₃, agg: AggFn::Sum, source: "units_sold" }
        )
    ),
])?;

println!("Schema registered: {} attribute keys", cube.attr_registry.len());
```

---

## 22. Writing and Reading Entities

```rust
// Writing a product entity
let product_id = Uuid::new_v4();
let actor = "user:alice@example.com";

// Option 1: Individual field writes (flexible, slower for many fields)
grid.write_cell(cube_id, coord(product_id, "name"),        "value", Text("Widget Pro"), actor)?;
grid.write_cell(cube_id, coord(product_id, "price"),       "value", Number(29.99), actor)?;
grid.write_cell(cube_id, coord(product_id, "status"),      "value", text_json("Active"), actor)?;
grid.write_cell(cube_id, coord(product_id, "tags"),        "value", json_array(["gadget"]), actor)?;

// Option 2: Batch write (preferred for domain entity creation — one transaction)
grid.write_cell_batch(cube_id, vec![
    (coord(product_id, "name"),        "value", Text("Widget Pro"), actor),
    (coord(product_id, "price"),       "value", Number(29.99), actor),
    (coord(product_id, "sku"),         "value", Text("WP-001"), actor),
    (coord(product_id, "status"),      "value", text_json("Draft"), actor),
    (coord(product_id, "tags"),        "value", json_array(["gadget", "new"]), actor),
    (coord(product_id, "category_ids"), "value", json_array([]), actor),
    (coord(product_id, "created_at"),  "value", DateTime(Utc::now()), actor),
], actor)?;

// OR-Set: add a tag (concurrent adds from multiple users/nodes all survive)
grid.apply_crdt_op(CrdtOperation::AddToSet {
    coord:      coord(product_id, "tags"),
    attr_key:   "value".into(),
    element:    TypedAttrValue::Text("featured".into()),
    unique_tag: Uuid::new_v4(),  // unique per add — identifies THIS specific add
    actor:      actor.into(),
})?;

// OR-Set: remove a specific tag (by unique_tag from the AddToSet op)
grid.apply_crdt_op(CrdtOperation::RemoveFromSet {
    coord:      coord(product_id, "tags"),
    attr_key:   "value".into(),
    unique_tag: the_unique_tag_from_the_add,  // must match the AddToSet
    actor:      actor.into(),
})?;

// Reading a single cell
let name: Option<TypedAttrValue> =
    grid.read_cell(cube_id, coord(product_id, "name"), "value")?;

// Reading a full entity (all fields)
let row: Vec<HyperCell> = grid.get_row(cube_id, &DimKey::uuid(product_id))?;

// Build field_map for easy access
let fields: HashMap<&str, &TypedAttrValue> = row.iter()
    .filter_map(|cell| {
        let d2 = cell.coord.d2_key()?;
        let val = cell.attributes.get("value")?;
        Some((d2.as_str(), val))
    })
    .collect();

let product_name = fields["name"].as_str().unwrap_or("");
let product_price = fields["price"].as_f64().unwrap_or(0.0);
let ai_demand = match fields.get("demand_forecast") {
    Some(TypedAttrValue::AiSignal { value, confidence, .. }) =>
        Some((value.as_f64().unwrap(), confidence)),
    _ => None,
};
```

---

## 23. Working with CRDT Semantics

```rust
// ── LWW: normal write — last writer wins ──────────────────────────────
// Two concurrent writes; the one with the higher VectorClock wins
grid.write_cell(cube_id, coord(id, "name"), "value", Text("Version A"), "user-a")?;
grid.write_cell(cube_id, coord(id, "name"), "value", Text("Version B"), "user-b")?;
// Result: whichever write has the higher VectorClock timestamp wins
// (In practice: the write that arrives at the merge engine later with a higher VC)

// ── OR-Set: all concurrent adds survive ───────────────────────────────
let tag_a = Uuid::new_v4();
let tag_b = Uuid::new_v4();
// Node 1 adds "featured":
grid.apply_op(AddToSet { coord, element: Text("featured"), unique_tag: tag_a, actor: "node-1" })?;
// Node 2 simultaneously adds "on-sale":
grid.apply_op(AddToSet { coord, element: Text("on-sale"), unique_tag: tag_b, actor: "node-2" })?;
// After merge: both "featured" AND "on-sale" are in the set

// ── OR-Set: remove only the exact tag ────────────────────────────────
// If "featured" was added with tag_a, remove it:
grid.apply_op(RemoveFromSet { coord, unique_tag: tag_a, actor: "node-1" })?;
// If a concurrent node added "featured" again with tag_c BEFORE the remove:
// tag_c's entry survives; tag_a's entry is removed
// Result: "featured" is STILL in the set (via tag_c) — OR-Set semantics

// ── Counter: transactional accumulation ──────────────────────────────
// Many nodes simultaneously decrement stock:
for sale in sales { // processing concurrent sales from multiple POS terminals
    grid.apply_op(IncrCounter { coord: coord(product_id, "stock_count"),
                                delta: -1, node_id: sale.pos_terminal_id })?;
}
// Final value = sum of all deltas across all nodes, in any order

// ── Lattice: governed lifecycle transitions ───────────────────────────
// Valid transition: Draft → Active
grid.apply_op(TransitionState {
    coord: coord(product_id, "lifecycle"),
    attr_key: "value".into(),
    from_state: "Draft".into(),
    to_state:   "Active".into(),
    actor:      "manager@example.com".into(),
    timestamp:  vc.tick("node-1"),
})?;

// Invalid transition: Draft → Retired (not in lattice)
// Returns Err(CrdtError::InvalidStateTransition { from: "Draft", to: "Retired",
//              valid_successors: ["Active", "Archived"] })

// Concurrent transitions: node A: Active → Deprecated; node B: Active → Discontinued
// Both are valid successors of Active.
// Lattice join: LUB("Deprecated", "Discontinued") depends on lattice topology.
// If neither is reachable from the other: fallback to Admin resolution.
```

---

## 24. Querying with HyperQL

```rust
// Basic entity filter
let results = grid.hyperql(HyperQuery::from("myapp.products")
    .where_cell("status", Eq, text_json("Active"))
    .where_cell("price",  Lt, Number(50.0))
    .select(vec!["name", "price", "tags", "demand_forecast"])
    .order_by("price", Desc)
    .limit(20)
    .offset(0)
).await?;

// Time-series query (N=3 cube with TimeAxis)
let quarterly = grid.hyperql(HyperQuery::from("myapp.sales_metrics")
    .where_dim(3, Between("2025-Q1", "2026-Q4"))
    .where_dim(2, Eq("revenue"))
    .select(vec!["period", "value"])
    .order_by_dim(3, Asc)
).await?;

// DimFold: aggregate over time axis
let annual_revenue = grid.hyperql(HyperQuery::from("myapp.sales_metrics")
    .where_dim(2, Eq("revenue"))
    .where_dim(3, Between("2026-Q1", "2026-Q4"))
    .fold(3, AggFn::Sum, "annual_revenue")  // collapse D₃ into one row per D₁
).await?;

// AS_OF time-travel
let historical_state = grid.hyperql(HyperQuery::from("myapp.products")
    .as_of("2026-01-01T00:00:00Z")
    .where_cell("status", Eq, text_json("Active"))
    .select(vec!["name", "price", "status"])
).await?;

// Graph traversal: find all products in a category and its subcategories
let category_products = grid.hyperql(HyperQuery::from("myapp.categories")
    .traverse_graph(
        start: category_id,
        edge_type: EdgeType::Hierarchy,
        direction: EdgeDirection::Outbound,
        max_depth: 5,
    )
    .join("myapp.products", GraphEdge("Contains"))
    .select_joined(vec!["products.name", "products.price"])
).await?;

// Natural language query (NL-to-HyperQL)
let nl_results = grid.nl_query(
    "Show me my top 10 best-selling products with low stock this month",
    NlQueryContext { caller: "user:alice", cube: "myapp.products", domain: "ecommerce" }
).await?;
println!("Generated HyperQL: {}", nl_results.generated_query);
println!("Explanation: {}", nl_results.explanation);
```

---

## 25. Building Views and Sheets

```rust
// A View is a saved HypercubeView configuration
let active_products_view = HypercubeView {
    view_id:      Uuid::new_v4(),
    cube_id:      cube_id,
    name:         "Active Products".into(),
    
    // DimSlice: filter to active products only
    dim_filters: vec![
        DimFilter::d2_eq("status", text_json("Active")),
    ],
    
    // Visible columns (subset of all attr keys)
    visible_attrs: vec!["name", "sku", "price", "stock_count",
                        "tags", "demand_forecast", "anomaly_flag"],
    
    // Default sort: by demand_forecast descending
    default_sort: vec![SortSpec { attr: "demand_forecast", dir: Desc }],
    
    // Default group: by status (then by price tier within group)
    default_group: vec!["status"],
    
    // Render mode
    render_mode: RenderMode::Grid2D,
    
    // Computed columns (shown only in this view — not stored)
    computed_cols: vec![
        ComputedCol {
            key:     "price_tier",
            formula: FormulaExpr::If {
                condition: FormulaExpr::Gt(FormulaExpr::attr("price"), FormulaExpr::lit(100.0)),
                then_expr: FormulaExpr::lit("Premium"),
                else_expr: FormulaExpr::lit("Standard"),
            },
        }
    ],
    
    // Visibility: who can see this view
    visibility: ViewVisibility::Tenant,
    
    // Filters applied BEFORE visibility mask (server-side)
    pre_filter: Some(DimFilter::d2_not_null("sku")),
};

// Register and save the view
grid.space_service.register_view(space_id, active_products_view)?;

// Execute a view query
let view_data = grid.execute_view(view_id, ExecuteViewOptions {
    caller:       "user:alice",
    limit:        50,
    offset:       0,
    override_filters: None,  // apply view's own filters
    as_of:        None,       // current state
})?;
```

---

## 26. Working with the Hypergraph

```rust
// Adding edges between entities
grid.hypergraph.write().unwrap().add_edge(HypergraphEdge {
    edge_id:        Uuid::new_v4(),
    from_node:      NodeId::row(cube_id, product_id),
    to_node:        NodeId::row(category_cube_id, category_id),
    edge_type:      EdgeType::Contains,
    direction:      EdgeDirection::Directed,
    weight:         1.0,
    attributes:     AttributeMap::default(),
    consent_status: ConsentStatus::None,  // no consent needed for Contains
    created_at:     Utc::now(),
})?;

// Hierarchy edges (with cycle detection)
let result = grid.add_dependency(parent_id, child_id, "user:alice");
match result {
    Ok(_)                              => println!("Dependency added"),
    Err(HypergridError::CyclicDependency) => println!("Cycle detected — rejected"),
}

// BFS traversal: all products reachable from a root category
let reachable = grid.hypergraph.read().unwrap()
    .reachable_from(NodeId::row(category_cube_id, root_category_id));

// Typed neighbor lookup: all products in category (Contains edge, outbound)
let products_in_category = grid.hypergraph.read().unwrap()
    .neighbors_of_type(
        NodeId::row(category_cube_id, category_id),
        EdgeType::Contains,
        EdgeDirection::Outbound,
    );

// CrossGridLink (cross-system connection)
let link = grid.add_cross_grid_link(CrossGridLinkRequest {
    source_entity: NodeId::row(cube_id, product_id),
    target_grid:   "kogi://alice/portfolio/store-revenue/".parse()?,
    target_entity: kogi_component_id,
    consent_config: ConsentConfig {
        requires_consent:  true,
        mirrored_attrs:    vec!["price", "stock_count", "demand_forecast"],
        writeback_attrs:   vec![],  // read-only link
    },
})?;
println!("CrossGridLink created: consent pending — ID {}", link.edge_id);
```

---

## 27. Federation and Multi-Node Operation

```rust
// Configure a federation peer
grid.federation.add_peer(FederationPeer {
    peer_id:       Uuid::new_v4(),
    node_id:       "node-eu-west-1".into(),
    endpoint_url:  "grpcs://eu-west-1.mygrid.com:9001".into(),
    trusted:       true,
    trust_level:   FederationTrustLevel::Grid,
    kafka_topics:  vec!["crdt.delta.my-grid-id"],
    tls_cert:      load_cert("eu-west-1.crt"),
})?;

// Perform delta sync with a peer
let sync_result = grid.federation.sync_with_peer("node-eu-west-1").await?;
println!("Synced: {} ops applied, {} ops sent", 
         sync_result.applied_count, sync_result.sent_count);

// Check federation health
for peer in grid.federation.all_peers() {
    let health = grid.federation.peer_health(&peer.node_id)?;
    println!("Peer {}: lag={}ms, last_sync={}", 
             peer.node_id, health.lag_ms, health.last_sync_at);
}

// Handle offline→online reconnect (automatic in production; manual here)
let reconnect_result = grid.federation.reconnect_and_sync("node-eu-west-1").await?;
// Automatically:
// 1. Fetches all ops from peer's CrdtLog since our last known cursor
// 2. Applies them via merge_engine (CRDT merge)
// 3. Sends all our ops since peer's last known cursor
// 4. Returns count of applied + sent ops
```

---

## 28. Using the AI Computation Model

```rust
// Register an AI engine plugin
struct MyDemandForecastEngine;
impl ComputedModelPlugin for MyDemandForecastEngine {
    fn output_attrs(&self) -> Vec<AttributeKeyDef> {
        vec![AttributeKeyDef::ai("demand_forecast", "30-Day Demand Forecast", self.plugin_id())]
    }
    
    fn invalidation_triggers(&self) -> Vec<EventKind> {
        vec![
            EventKind::CellMutation { attr_key: "price".into() },
            EventKind::CellMutation { attr_key: "stock_count".into() },
        ]
    }
    
    fn cache_ttl(&self) -> Duration { Duration::from_secs(86400) }  // 24 hours
    
    fn compute(&self, row: &HyperRow, ctx: &ComputeContext) -> ComputeResult {
        // Read inputs
        let price = row.get_f64("price").unwrap_or(0.0);
        let stock = row.get_f64("stock_count").unwrap_or(0.0);
        
        // Load historical sales from TimeAxis
        let sales_history = ctx.time_series.load_last_n("revenue", 12)?;
        
        // Simple moving average forecast (replace with ML model in production)
        let avg_sales: f64 = sales_history.iter().sum::<f64>() / sales_history.len() as f64;
        let forecast = avg_sales * (1.0 - (price / 100.0).clamp(0.0, 0.5)); // price elasticity
        
        ComputeResult::Ok(TypedAttrValue::AiSignal {
            value:       Box::new(TypedAttrValue::Number(forecast)),
            model_id:    "demand_forecast.v1".into(),
            computed_at: Utc::now(),
            confidence:  0.72,
            explanation: Some(format!("Based on {}-period average sales of {:.0}", 
                                       sales_history.len(), avg_sales)),
        })
    }
}

// Register at startup
grid.plugin_registry.register(Arc::new(MyDemandForecastEngine))?;
// From now on: any write to "price" or "stock_count" triggers recomputation
// The result appears in cell[(product_id, "demand_forecast")].value as AiSignal

// Query AI results alongside regular data
let products_with_forecast = grid.hyperql(HyperQuery::from("myapp.products")
    .select(vec!["name", "price", "demand_forecast", "anomaly_flag"])
    .where_ai_confidence("demand_forecast", Gt, 0.7)  // only high-confidence predictions
    .order_by("demand_forecast", Desc)
    .limit(20)
).await?;
```

---

## 29. Building and Installing Plugins

```rust
// Full Domain Pack plugin for an e-commerce domain
pub struct EcommerceDomainPack {
    demand_engine: Arc<DemandForecastEngine>,
    fraud_engine:  Arc<FraudDetectionEngine>,
}

impl HypercubePlugin for EcommerceDomainPack {
    fn plugin_id(&self) -> PluginId { "ecommerce.domain_pack.v1".into() }
    fn plugin_name(&self) -> &str { "E-Commerce Domain Pack" }
    fn version(&self) -> semver::Version { "1.0.0".parse().unwrap() }
    fn capabilities(&self) -> Vec<PluginCapability> {
        vec![
            PluginCapability::AttributeType,
            PluginCapability::ComputedModel,
            PluginCapability::GovernanceModel,
        ]
    }

    fn attribute_key_defs(&self) -> Vec<AttributeKeyDef> {
        vec![
            // Domain-specific attributes added to myapp.products
            AttributeKeyDef::text("gtin", "Global Trade Item Number", PermissionTier::Manager),
            AttributeKeyDef::text("brand", "Brand", PermissionTier::Editor),
            AttributeKeyDef::json("certifications", "Certifications", PermissionTier::Manager)
                .with_crdt(CrdtSemantics::OrSet),
            AttributeKeyDef::json("compliance_flags", "Compliance Flags", PermissionTier::Manager),
            // AI-computed
            AttributeKeyDef::ai("demand_forecast", "Demand Forecast", DEMAND_ENGINE_ID),
            AttributeKeyDef::ai("fraud_risk_score", "Fraud Risk Score", FRAUD_ENGINE_ID),
        ]
    }

    fn on_load(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        // Initialize ML models, load feature vectors, warm caches
        self.demand_engine.initialize(ctx.config.get("demand_model_path"))?;
        self.fraud_engine.initialize(ctx.config.get("fraud_model_path"))?;
        Ok(())
    }

    fn on_cell_mutation(&self, cube_id: CubeId, coord: &DimCoordinate) -> Vec<AttributeKey> {
        // Which AI attrs need recomputation when a cell changes?
        let mut recompute = vec![];
        if let Some(field) = coord.d2_str() {
            if ["price", "stock_count", "sales_history"].contains(&field) {
                recompute.push("demand_forecast".into());
            }
            if ["order_amount", "payment_method", "shipping_address"].contains(&field) {
                recompute.push("fraud_risk_score".into());
            }
        }
        recompute
    }

    fn compute_attribute(&self, key: &AttributeKey, ctx: &ComputeContext<'_>)
        -> HypergridResult<TypedAttrValue>
    {
        match key.as_str() {
            "demand_forecast" => self.demand_engine.compute(ctx),
            "fraud_risk_score" => self.fraud_engine.compute(ctx),
            _ => Ok(TypedAttrValue::Null),
        }
    }

    fn health_check(&self) -> PluginHealth {
        if self.demand_engine.is_ready() && self.fraud_engine.is_ready() {
            PluginHealth::Healthy
        } else {
            PluginHealth::Degraded { reason: "ML model not loaded".into() }
        }
    }
}

// Installation: at Grid startup
let pack = EcommerceDomainPack::new(config)?;
grid.plugin_registry.register(Arc::new(pack))?;
```

---

## 30. Performance Tuning

```rust
// 1. Choose the right storage encoding for your access patterns:
//    Dense:  entity×field with no extra dimensions — fastest for N=2 entity stores
//    Sparse: any dimension with many possible keys but sparse population
//    Hybrid: dense D₁×D₂, sparse D₃+ — best for entity stores with time-series extensions
let cube = grid.create_cube(CreateCubeRequest {
    encoding: StorageEncoding::Hybrid { dense_dims: vec![1, 2] },
    ..
})?;

// 2. Use batch writes for entity creation (one PG transaction, one EventLog batch):
grid.write_cell_batch(cube_id, all_fields_at_once, actor)?;
// NOT: individual write_cell() calls in a loop

// 3. Add domain-specific indexes for hot query patterns:
cube.add_index(IndexSpec {
    axis_idx:  2,  // D₂ (property axis)
    attr_key:  Some("status"),
    strategy:  IndexStrategy::BTree,
    condition: Some("WHERE attr_value::text != '\"Archived\"'"),  // partial index
})?;

// 4. Configure ClickHouse routing for analytics:
grid.set_analytics_threshold(500_000);  // route DimFold > 500K rows to ClickHouse

// 5. Tune Redis cache TTLs per attribute hotness:
cube.attr_registry.set_cache_ttl("name",          Duration::from_secs(300));  // cold
cube.attr_registry.set_cache_ttl("status",        Duration::from_secs(30));   // warm
cube.attr_registry.set_cache_ttl("health_score",  Duration::from_secs(60));   // AI attr
cube.attr_registry.set_cache_ttl("stock_count",   Duration::from_secs(5));    // hot counter

// 6. Use HyperQL projection pruning (only select what you need):
// BAD:
let row = grid.get_row(cube_id, &product_id_key)?;  // reads ALL fields
// GOOD:
let result = grid.hyperql(HyperQuery::from("myapp.products")
    .where_d1(Eq(product_id))
    .select(vec!["name", "price", "status"])  // only 3 fields
).await?;

// 7. For high-throughput counters, use Redis as the hot counter store:
//    Configure PN-Counter to flush to PostgreSQL every 30s rather than on every write
cube.attr_registry.get_mut("stock_count")
    .unwrap().counter_flush_interval = Duration::from_secs(30);

// 8. Pre-warm computed attribute caches on startup:
grid.warm_ai_cache(cube_id, vec!["health_score", "demand_forecast"]).await?;
// Triggers background compute for all rows that have stale/missing AI attrs
```
# Part IV — Building Domain Operating Systems on Hypergrid

## 31. What is a Domain Operating System?

A Domain Operating System (Domain OS) is a software system that provides a complete, opinionated operational environment for entities within a specific domain, built on top of Hypergrid as its universal data substrate.

A Domain OS has five defining characteristics:

1. **A single root domain concept** — one entity type that is the center of gravity for everything in the system. Every other entity in the system exists to enrich, relate to, or govern the root domain concept.

2. **A root Hyperspreadsheet** — the primary Hypercube in which the root domain concept entities are stored as HyperRows. Every other Hypercube in the system exists to support or extend this primary cube.

3. **A domain-typed API surface** — the system exposes domain vocabulary, not Hypergrid vocabulary. Users interact with Portfolios, Organizations, and Solutions — not with HyperRows and DimCoordinates.

4. **A domain intelligence layer** — AI engines that compute domain-specific signals (health scores, risk assessments, alignment scores, compliance predictions) as Tier-2 computed attributes on the primary and supporting cubes.

5. **A federated graph** — a domain-typed subset of the Hypergraph that models the relationships between domain entities, including cross-system connections to other Domain OSes via CrossGridLink edges.

The three Apapo domain operating systems (Kogi, Ume, Qala) are the canonical examples. But the Domain OS pattern applies to any domain: healthcare (Patient OS), supply chain (Supply Chain OS), research (Lab OS), legal practice (Legal OS), financial services (Portfolio Management OS), real estate (Property OS), and so on.

---

## 32. The Domain OS Architecture Specification

Every Domain OS built on Hypergrid MUST conform to the following architectural specification. This is not advisory — it is the contract that enables cross-system federation, unified audit, and platform-level governance.

### 32.1 Mandatory Structural Requirements

```
DOMAIN-OS-SPEC-001: Root Domain Concept
  Every Domain OS MUST declare exactly one root domain concept.
  The root concept MUST be a typed entity stored as a HyperRow in the primary Hypercube.
  All other entities in the system MUST have a defined relationship to the root concept,
  expressed as: part-of | belongs-to | governs | supports | derives-from.

DOMAIN-OS-SPEC-002: Primary Hypercube
  Every Domain OS MUST register exactly one primary Hypercube named:
    {domain_prefix}.{root_entity_type_plural}
  Example: "kogi.portfolio.components", "ume.org.modules", "qala.solutions"
  The primary Hypercube MUST use the N=2 Hybrid encoding (D₁=EntityAxis, D₂=PropertyAxis)
  as the base, with additional dimensions as needed.
  All root domain entities MUST be stored as HyperRows in this cube.

DOMAIN-OS-SPEC-003: DomainStore Codec
  Every Domain OS MUST implement the DomainStore<Entity, EntityId> codec:
    - bootstrap(grid): register all Hypercubes and attribute key schemas
    - write(grid, cube_id, entity, actor): entity → HyperCells
    - read(grid, cube_id, id): HyperCells → entity (or None)
    - exists(grid, cube_id, id): bool
  The codec MUST be the ONLY place where domain entity structs are serialized to
  or deserialized from HyperCells. Domain logic layer MUST NOT access cells directly.

DOMAIN-OS-SPEC-004: Domain System Root
  Every Domain OS MUST have a root orchestrator struct (e.g. PortfolioSystem, UmeKernel,
  QalaOS) that:
    - Owns the Grid (or holds a reference to a shared Grid)
    - Exposes all domain operations as typed methods
    - Delegates all storage to the DomainStore codec
    - Emits dual-write events on all mutations (§DOMAIN-OS-SPEC-007)
    - Never exposes HyperCell, DimCoordinate, or DimKey to its callers

DOMAIN-OS-SPEC-005: Attribute Key CRDT Assignment
  Every attribute key registered in every Domain OS Hypercube MUST have a correct
  CrdtSemantics assignment. No attribute key may use the default (LWW) without
  explicit justification documented in AttributeKeyDef.notes.
  Assignment MUST follow the CRDT Decision Tree (Appendix B of the Master Design Doc).

DOMAIN-OS-SPEC-006: Permission Tier Assignment
  Every attribute key MUST have an explicit write_permission PermissionTier.
  AI-computed attributes MUST use PermissionTier::System.
  System metadata (vector_clock, last_actor, created_at) MUST use PermissionTier::System.
  No attribute key may use PermissionTier::Viewer (0) for write access.

DOMAIN-OS-SPEC-007: Dual-Write Event Architecture
  Every mutating operation in the Domain OS MUST emit two events:
    a. A domain-typed event to the domain event log (for domain queries)
    b. A Hypergrid EventEntry to the shared EventLog as EventKind::Custom(domain_tag)
  The dual-write MUST be synchronous (both before async fan-out).
  Domain tag format: "{domain_prefix}:{event_kind}"  e.g. "portfolio:ComponentCreated"

DOMAIN-OS-SPEC-008: NamespacePath Assignment
  Every root domain entity MUST have a NamespacePath assigned at creation.
  Path format: {domain}://{space_slug}/{entity_type_slug}/{entity_slug}/
  The NamespacePath MUST be registered in the Grid's NamespaceRegistry.
  Cross-system links MUST use fully-qualified NamespacePaths.

DOMAIN-OS-SPEC-009: HyperQL Passthrough
  Every Domain OS root orchestrator MUST expose a hyperql() method that passes
  HyperQL queries directly to the underlying Grid without modification.
  This enables AI engines, analytics, and advanced users to bypass the domain API
  and query the Hypercube substrate directly.

DOMAIN-OS-SPEC-010: Stats Exposure
  Every Domain OS root orchestrator MUST expose a stats() method returning both:
    - Domain-level metrics: entity count, edge count, event count
    - Hypergrid substrate metrics: hg_total_cells, hg_event_count, hg_crdt_ops
  This enables platform-level monitoring without coupling to domain internals.
```

### 32.2 Mandatory Attribute Keys

Every primary Hypercube in every Domain OS MUST register the following attribute keys with the specified CRDT semantics and permissions:

| Attribute Key | Type | CRDT | Permission | Purpose |
|--------------|------|------|------------|---------|
| `created_at` | DateTime | LWW | System | Entity creation timestamp |
| `updated_at` | DateTime | LWW | System | Last mutation timestamp |
| `last_actor` | Text | LWW | System | NodeId of last writer (CRDT tiebreak) |
| `vector_clock` | Json | LWW | System | Causal clock snapshot at last write |
| `version` | Text | MaxRegister | Editor | Semantic version string |
| `status` | Json | LWW → Lattice (v2) | Editor | Lifecycle status enum |
| `visibility` | Json | LWW | Owner | Visibility: Private\|Protected\|Public |
| `owners` | Json | OR-Set | Owner | Vec<UserId> — primary ownership |
| `tags` | Json | OR-Set | Contributor | Freeform labels |
| `policy_ids` | Json | LWW | Admin | Governance policy attachments |
| `namespace_path` | Text | LWW | System | NamespacePath URI |

All additional attribute keys are domain-specific and defined per Domain OS.

### 32.3 Mandatory Hypergraph EdgeTypes

Every Domain OS MUST support the following edge types in its PortfolioGraph / OrgGraph / SolutionGraph:

| EdgeType | Required For | Cycle Detection | Consent |
|----------|-------------|-----------------|---------|
| `Hierarchy` | All Domain OSes | Not enforced | No |
| `Dependency` | All Domain OSes | Enforced (DFS) | No |
| `Association` | All Domain OSes | Not enforced | No |
| `CrossGridLink` | All Domain OSes | N/A | Yes |
| Domain-specific | Per Domain OS | Per type | Per type |

The Domain OS graph implementation MUST:
- Enforce DFS cycle detection on all Dependency edge additions
- Require accepted ConsentStatus before any CrossGridLink edge syncs shadow cells
- Expose reachable_from(id) via BFS traversal
- Expose detect_cycle() returning the cycle path if present

---

## 33. Step-by-Step: Building a Domain OS

This is the practical guide to building a new Domain OS on Hypergrid, using the Kogi/Ume/Qala implementations as references.

### Phase 1: Domain Design (pre-code)

```
1.1 Name your root domain concept.
    Ask: "What is the single entity that everything in this system exists to serve?"
    Examples: Patient (Healthcare OS), Shipment (Logistics OS), Case (Legal OS)

1.2 Define all domain entities.
    For each: is it a core entity (rows in the primary cube) or a supporting entity
    (rows in a separate cube)?
    Rule: if it's always subordinate to the root concept, make it a field or
    a separate cube. If it's independently addressable, make it its own cube.

1.3 Define the graph topology.
    For each pair of entities: what's the relationship? Hierarchy? Dependency?
    Association? Does it require consent? Does it create a shadow?

1.4 Identify your CRDT requirements.
    For each field: which CRDT semantics? Use the decision tree.
    Pay special attention to: set-valued fields (OR-Set), lifecycle states (Lattice),
    accumulated counters (PnCounter), and AI-computed fields (System LWW).

1.5 Identify your AI signals.
    What should the system automatically compute from entity data?
    Health scores? Risk assessments? Predictions? Recommendations?
    Each becomes a Tier-2 computed attribute with its own plugin.
```

### Phase 2: Schema Implementation

```rust
// Step 1: Define the domain entity structs (pure Rust, no storage concerns)
pub struct Patient {
    pub metadata: PatientMetadata,
    pub data:     PatientData,
}

pub struct PatientMetadata {
    pub id:           PatientId,       // Uuid
    pub owners:       Vec<UserId>,     // care team
    pub tags:         HashSet<String>,
    pub policy_ids:   Vec<PolicyId>,
    pub created_at:   DateTime<Utc>,
    pub updated_at:   DateTime<Utc>,
    pub last_actor:   String,
    pub vector_clock: VectorClock,
    pub version:      String,
}

pub struct PatientData {
    pub name:           String,
    pub date_of_birth:  NaiveDate,
    pub status:         PatientStatus,  // enum
    pub conditions:     Vec<String>,    // OR-Set
    pub medications:    Vec<String>,    // OR-Set
    pub care_team:      Vec<UserId>,    // OR-Set
    pub visibility:     Visibility,
    pub risk_score:     Option<f64>,    // AI-computed
    pub anomaly_flag:   Option<AnomalySignal>, // AI-computed
}

// Step 2: Define error types
pub type HealthcareResult<T> = Result<T, HealthcareError>;
pub enum HealthcareError {
    NotFound(PatientId),
    PermissionDenied { user_id: String, action: String },
    StorageError(String),
    CyclicDependency(PatientId, PatientId),
    InvalidOperation(String),
}

// Step 3: Implement the DomainStore codec
pub mod fields {
    pub const NAME:           &str = "name";
    pub const DATE_OF_BIRTH:  &str = "date_of_birth";
    pub const STATUS:         &str = "status";
    pub const CONDITIONS:     &str = "conditions";
    pub const MEDICATIONS:    &str = "medications";
    pub const CARE_TEAM:      &str = "care_team";
    pub const RISK_SCORE:     &str = "risk_score";
    pub const ANOMALY_FLAG:   &str = "anomaly_flag";
    // ... mandatory fields
    pub const OWNERS:         &str = "owners";
    pub const TAGS:           &str = "tags";
    pub const POLICY_IDS:     &str = "policy_ids";
    pub const CREATED_AT:     &str = "created_at";
    pub const UPDATED_AT:     &str = "updated_at";
    pub const LAST_ACTOR:     &str = "last_actor";
    pub const VECTOR_CLOCK:   &str = "vector_clock";
    pub const VERSION:        &str = "version";
    pub const NAMESPACE_PATH: &str = "namespace_path";
}

pub const PATIENTS_CUBE: &str = "healthcare.patients";

pub struct PatientStore;
impl PatientStore {
    pub fn bootstrap(grid: &mut Grid) -> HypergridResult<CubeId> {
        let cube_id = grid.create_cube(PATIENTS_CUBE)?;
        let cube = grid.get_cube_mut(cube_id)?;
        
        // Register all attribute keys with correct CRDT semantics
        cube.attr_registry.register_batch(vec![
            // Text fields (LWW)
            lww_text(fields::NAME,           "Patient Name",       PermissionTier::Editor),
            lww_text(fields::LAST_ACTOR,     "Last Actor",         PermissionTier::System),
            lww_text(fields::VERSION,        "Version",            PermissionTier::System)
                .with_crdt(CrdtSemantics::MaxRegister),
            lww_text(fields::NAMESPACE_PATH, "Namespace Path",     PermissionTier::System),

            // JSON/Enum fields (LWW)
            lww_json(fields::STATUS,         "Patient Status",     PermissionTier::Editor),
            lww_json(fields::VISIBILITY,     "Visibility",         PermissionTier::Owner),
            lww_json(fields::POLICY_IDS,     "Policy IDs",         PermissionTier::Admin),
            lww_json(fields::DATE_OF_BIRTH,  "Date of Birth",      PermissionTier::Editor)
                .with_attr_visibility(AttrVisibility::Owner), // private by default

            // OR-Set fields (concurrent adds survive)
            orset_json(fields::OWNERS,      "Care Team Owners",   PermissionTier::Owner),
            orset_json(fields::TAGS,        "Tags",               PermissionTier::Contributor),
            orset_json(fields::CONDITIONS,  "Conditions",         PermissionTier::Editor),
            orset_json(fields::MEDICATIONS, "Medications",        PermissionTier::Editor),
            orset_json(fields::CARE_TEAM,   "Care Team",          PermissionTier::Manager)
                .with_attr_visibility(AttrVisibility::Tenant), // only visible to care team

            // System timestamps (LWW, system-write only)
            lww_json(fields::CREATED_AT,    "Created At",         PermissionTier::System),
            lww_json(fields::UPDATED_AT,    "Updated At",         PermissionTier::System),
            lww_json(fields::VECTOR_CLOCK,  "Vector Clock",       PermissionTier::System),

            // AI-computed (System-write only)
            AttributeKeyDef::ai(fields::RISK_SCORE,    "Risk Score",    RISK_ENGINE_ID)
                .with_attr_visibility(AttrVisibility::Tenant), // care team only
            AttributeKeyDef::ai(fields::ANOMALY_FLAG,  "Anomaly Flag",  ANOMALY_ENGINE_ID),
        ])?;
        Ok(cube_id)
    }

    pub fn write(grid: &mut Grid, cube_id: CubeId, patient: &Patient, actor: &str)
        -> HypergridResult<()>
    {
        let id = patient.metadata.id;
        macro_rules! w {
            ($field:expr, $val:expr) => {
                grid.write_cell(cube_id, coord(id, $field), "value", $val, actor)?;
            };
        }
        w!(fields::NAME,          Text(patient.data.name.clone()));
        w!(fields::DATE_OF_BIRTH, as_json(&patient.data.date_of_birth));
        w!(fields::STATUS,        as_json(&patient.data.status));
        w!(fields::CONDITIONS,    as_json(&patient.data.conditions));
        w!(fields::MEDICATIONS,   as_json(&patient.data.medications));
        w!(fields::CARE_TEAM,     as_json(&patient.data.care_team));
        w!(fields::VISIBILITY,    as_json(&patient.data.visibility));
        w!(fields::OWNERS,        as_json(&patient.metadata.owners));
        w!(fields::TAGS,          as_json(&patient.metadata.tags));
        w!(fields::POLICY_IDS,    as_json(&patient.metadata.policy_ids));
        w!(fields::CREATED_AT,    as_json(&patient.metadata.created_at));
        w!(fields::UPDATED_AT,    as_json(&patient.metadata.updated_at));
        w!(fields::LAST_ACTOR,    Text(patient.metadata.last_actor.clone()));
        w!(fields::VECTOR_CLOCK,  as_json(&patient.metadata.vector_clock));
        w!(fields::VERSION,       Text(patient.metadata.version.clone()));
        Ok(())
    }

    pub fn read(grid: &Grid, cube_id: CubeId, id: PatientId)
        -> HealthcareResult<Option<Patient>>
    {
        let cells = grid.get_row(cube_id, &DimKey::uuid(id));
        if cells.is_empty() { return Ok(None); }
        
        let fields: HashMap<&str, &TypedAttrValue> = cells.iter()
            .filter_map(|c| Some((c.coord.d2_str()?, c.value()?)))
            .collect();
        
        macro_rules! text { ($f:expr) => { fields.get($f).and_then(|v| v.as_str())
            .unwrap_or_default().to_owned() }; }
        macro_rules! json_of { ($f:expr) => {
            fields.get($f).and_then(|v| v.as_json()).cloned().unwrap_or_default() }; }
        macro_rules! from_json { ($f:expr, $T:ty) =>
            { serde_json::from_value::<$T>(json_of!($f)).unwrap_or_default() }; }
        
        let metadata = PatientMetadata {
            id,
            owners:       from_json!(fields::OWNERS,   Vec<UserId>),
            tags:         from_json!(fields::TAGS,     HashSet<String>),
            policy_ids:   from_json!(fields::POLICY_IDS, Vec<PolicyId>),
            created_at:   serde_json::from_value(json_of!(fields::CREATED_AT))?,
            updated_at:   serde_json::from_value(json_of!(fields::UPDATED_AT))?,
            last_actor:   text!(fields::LAST_ACTOR),
            vector_clock: serde_json::from_value(json_of!(fields::VECTOR_CLOCK))
                          .unwrap_or_default(),
            version:      text!(fields::VERSION),
        };
        
        let data = PatientData {
            name:         text!(fields::NAME),
            date_of_birth: from_json!(fields::DATE_OF_BIRTH, NaiveDate),
            status:       from_json!(fields::STATUS, PatientStatus),
            conditions:   from_json!(fields::CONDITIONS, Vec<String>),
            medications:  from_json!(fields::MEDICATIONS, Vec<String>),
            care_team:    from_json!(fields::CARE_TEAM, Vec<UserId>),
            visibility:   from_json!(fields::VISIBILITY, Visibility),
            risk_score:   None,  // read from AI computed cache separately
            anomaly_flag: None,
        };
        
        Ok(Some(Patient { metadata, data }))
    }

    pub fn exists(grid: &Grid, cube_id: CubeId, id: PatientId) -> bool {
        !grid.get_row(cube_id, &DimKey::uuid(id)).is_empty()
    }
}
```

### Phase 3: Root Orchestrator Implementation

```rust
pub struct HealthcareOS {
    pub grid:         Grid,
    pub patients_cube: CubeId,
    pub graph:        PatientGraph,        // wraps Hypergraph
    pub event_log:    HealthcareEventLog,  // wraps Hypergrid EventLog
    pub node_id:      String,
    plugins:          Vec<Arc<dyn HypercubePlugin>>,
}

impl HealthcareOS {
    pub fn new(node_id: impl Into<String>) -> Self {
        let node_id = node_id.into();
        let mut grid = Grid::new("healthcare", node_id.clone());
        let patients_cube = PatientStore::bootstrap(&mut grid)
            .expect("Failed to bootstrap patients Hypercube");
        let grid_id = grid.grid_id;
        Self {
            patients_cube,
            graph: PatientGraph::new(grid_id, patients_cube),
            event_log: HealthcareEventLog::new(grid_id),
            node_id,
            plugins: vec![],
            grid,
        }
    }

    // ── Create ────────────────────────────────────────────────────
    pub fn admit_patient(&mut self, registrar: UserId, name: String) -> HealthcareResult<PatientId> {
        let actor = self.node_id.clone();
        let patient = Patient::new(registrar, name, &actor);
        let id = patient.metadata.id;
        PatientStore::write(&mut self.grid, self.patients_cube, &patient, &actor)?;
        self.graph.register_patient(id);
        self.event_log.emit(HealthcareEvent::new(
            HealthcareEventKind::PatientAdmitted, Some(id), &actor
        ));
        Ok(id)
    }

    // ── Read ──────────────────────────────────────────────────────
    pub fn get_patient(&self, id: PatientId) -> HealthcareResult<Patient> {
        PatientStore::read(&self.grid, self.patients_cube, id)?
            .ok_or(HealthcareError::NotFound(id))
    }

    // ── Mutate ────────────────────────────────────────────────────
    pub fn add_condition(&mut self, actor: UserId, patient_id: PatientId, condition: String)
        -> HealthcareResult<()>
    {
        let actor_node = self.node_id.clone();
        let mut patient = self.get_patient(patient_id)?;
        if !patient.check_permission(&actor, PermissionTier::Editor) {
            return Err(HealthcareError::PermissionDenied {
                user_id: actor.to_string(), action: "add_condition".into()
            });
        }
        // Use OR-Set directly for concurrent-safe set addition
        self.grid.apply_crdt_op(CrdtOperation::AddToSet {
            coord:      coord(patient_id, fields::CONDITIONS),
            attr_key:   "value".into(),
            element:    TypedAttrValue::Text(condition.clone()),
            unique_tag: Uuid::new_v4(),
            actor:      actor_node.clone(),
        })?;
        self.event_log.emit(HealthcareEvent::new(
            HealthcareEventKind::ConditionAdded, Some(patient_id), &actor_node
        ).with_payload(json!({ "condition": condition })));
        Ok(())
    }

    // ── HyperQL passthrough (SPEC-009) ────────────────────────────
    pub fn hyperql(&self, query: HyperQuery) -> Vec<QueryRow> {
        self.grid.hyperql(query)  // pass through directly
    }

    // ── Stats (SPEC-010) ──────────────────────────────────────────
    pub fn stats(&self) -> HealthcareStats {
        let hg = self.grid.stats();
        HealthcareStats {
            patient_count:    self.all_patient_ids().len(),
            edge_count:       self.graph.edge_count(),
            event_count:      self.event_log.len(),
            hg_total_cells:   hg.total_cells,
            hg_event_count:   hg.event_count,
            hg_crdt_ops:      hg.crdt_op_count,
        }
    }
}
```

---

## 34. The Root Domain Concept

Selecting the root domain concept is the most important design decision in building a Domain OS. The root concept must satisfy:

```
ROOT-CONCEPT-RULE-001: Singularity
  There is exactly one root domain concept. Not two, not "it depends."
  If you are tempted to have two root concepts, you are building two Domain OSes.

ROOT-CONCEPT-RULE-002: Universality
  Every other entity in the system either IS the root concept (a different instance)
  or exists in service of the root concept.
  Test: "Does this entity have meaning outside the context of a [root concept]?"
  If yes: it may be a root concept itself (separate Domain OS).
  If no: it belongs to this Domain OS.

ROOT-CONCEPT-RULE-003: The Master Spreadsheet
  The root concept forms the primary Hypercube: D₁ = root concept entity.
  Every row in the master spreadsheet IS a root domain concept entity.
  The master spreadsheet IS the system.

ROOT-CONCEPT-RULE-004: Self-Sufficiency
  The root concept must be meaningful and operationally useful WITHOUT reference
  to any other entity type in the system. It has its own identity, lifecycle,
  properties, and owners independently of all dependent entities.

ROOT-CONCEPT-RULE-005: Composability
  Other Domain OSes' root concepts may relate to your root concept via CrossGridLink.
  The root concept is the integration point: it is what other systems link to.
```

Root concept selection examples:

| Domain | Root Concept | Why Not the Alternative |
|--------|-------------|------------------------|
| Healthcare | Patient | Not "Visit" (Visit needs Patient); not "Hospital" (Hospital is a Space, not a root entity) |
| Legal | Matter (Case) | Not "Client" (Client exists outside this OS); not "Document" (Document serves the Matter) |
| Supply Chain | Shipment | Not "Product" (Product managed by a Product OS); not "Carrier" (Carrier is a Resource) |
| Real Estate | Property | Not "Lease" (Lease serves the Property); not "Tenant" (Tenant may span properties) |
| Research | Experiment | Not "Dataset" (Dataset is output of Experiment); not "Lab" (Lab is a Space) |
| Education | Enrollment | Not "Student" (Student spans institutions); not "Course" (Course is a Container) |
| Agriculture | Field (Plot) | Not "Crop" (Crop is grown on Field); not "Farm" (Farm is a Space/Container) |

---

## 35. Hypercube Schema Design

### 35.1 Primary Hypercube — Always N=2

The primary Hypercube MUST use N=2 with Hybrid encoding as the base. The entity × field layout is the standard for all domain entity stores.

### 35.2 When to Add N=3

Add a third dimension (D₃) when the entity naturally exists across multiple contexts simultaneously and cross-context queries are a core use case:

| D₃ Axis Type | When to Use | Example |
|-------------|-------------|---------|
| TimeAxis | Time-series tracking of entity metrics | `healthcare.vitals` (patient × metric × timestamp) |
| CategoryAxis | Entity has fundamentally different data per category | `healthcare.compliance` (patient × field × jurisdiction) |
| TenantAxis | One cube serves multiple tenants with isolated data | `healthcare.records` (patient × field × hospital_id) |
| ScenarioAxis | Planning/what-if analysis across scenarios | `healthcare.forecasts` (patient × metric × scenario) |

**Do NOT add D₃ for:**
- "Future extensibility" (add when needed, not speculatively)
- Filtering that can be expressed as a D₂ attribute value filter (use `WHERE status = 'Active'` not `D₃ = 'Active'`)
- Relationships between entities (use Hypergraph edges, not an axis)

### 35.3 Supporting Hypercubes

Beyond the primary cube, a Domain OS typically needs 3–8 supporting cubes:

| Supporting Cube Pattern | Purpose | N | Example |
|------------------------|---------|---|---------|
| `{domain}.{entity_type}.metrics` | KPI and metric time-series | 3 | `healthcare.patients.vitals` (N=3 with TimeAxis) |
| `{domain}.{entity_type}.events` | Domain event ledger | 2 | `healthcare.patients.events` |
| `{domain}.{entity_type}.approvals` | Governance workflow records | 2 | `healthcare.patients.approvals` |
| `{domain}.{entity_type}.snapshots` | Point-in-time snapshots for rollback | 2 | `healthcare.patients.snapshots` |
| `{domain}.{entity_type}.links` | Cross-system link records | 2 | `healthcare.patients.links` |
| `{domain}.{sub_entity_type}` | Supporting entity types | 2 | `healthcare.encounters`, `healthcare.prescriptions` |

---

## 36. CRDT Semantics Assignment

The CRDT assignment is the most consequential technical decision in schema design. Incorrect CRDT semantics cause data loss in federated deployments. Use this complete decision tree:

```
For each attribute key, answer these questions in order:

Q1: Is this attribute a set of values that multiple users may add to concurrently
    and where all concurrent adds must survive?
    Examples: tags, owners, members, conditions, medications, dependencies, toolbox_ids
    → OR-Set

Q2: Is this attribute a counter that only ever increases?
    Examples: restart_count, view_count, follower_count, contribution_count
    → GrowOnlyCounter

Q3: Is this attribute a counter that can be both incremented AND decremented
    concurrently by multiple actors/nodes?
    Examples: stock_count, budget_spent, hours_logged, allocated_units
    → PNCounter

Q4: Is this attribute a number where the highest value should always win,
    regardless of when the write occurred?
    Examples: version number, sequence number, schema version, sprint number
    → MaxRegister

Q5: Is this attribute a lifecycle/workflow state that must follow a
    defined set of valid transitions (partial order)?
    Examples: ComponentStatus, PatientStatus, SolutionLifecycle, ModuleState
    → Lattice (with explicitly defined LatticeOrder)
    (v1 interim: LWW; document the planned lattice in AttributeKeyDef.notes)

Q6: Is this attribute written exclusively by an AI engine or system process
    and must never be overwritten by user action?
    Examples: health_score, risk_score, demand_forecast, anomaly_flag
    → LastWriteWins + write_permission: PermissionTier::System
    (The System PermissionTier enforces the "AI-write only" constraint)

Q7: Is this attribute human-authored content or a configuration value where
    the most recent human edit should always win?
    Examples: name, description, config, payload, JSON blob, any enum field
    → LastWriteWins

Q8: Is this attribute a log/ledger that must preserve all entries from all nodes?
    Examples: activity_log, decision_log, comment_thread
    → AppendLog

Q9: Is this a nested JSON object where concurrent editors may update different
    leaf keys and all their changes must survive?
    Examples: plugin_configs, feature_flags, settings_tree
    → DeepMergeJson

Q10: None of the above?
    → Consult the domain architects. Document the choice explicitly.
       Consider whether this is actually two separate concerns (split into two attrs).
```

---

## 37. The DomainStore Codec

The DomainStore codec is the translation layer between domain entity structs and Hypergrid HyperCells. It must be:

- **Complete**: every field of every domain entity is persisted and reconstructed
- **Correct**: the TypedAttrValue variant exactly matches the declared AttributeType
- **Idempotent**: writing the same entity twice produces the same result
- **Invertible**: read(write(entity)) == entity (round-trip correctness)

Helper functions for the codec (implement once, use everywhere):

```rust
// Codec helper functions — implement in your domain's store module
fn coord(entity_id: Uuid, field: &str) -> DimCoordinate {
    DimCoordinate::d2(DimKey::uuid(entity_id), DimKey::str(field))
}

fn as_json<T: serde::Serialize>(val: &T) -> TypedAttrValue {
    TypedAttrValue::Json(
        serde_json::to_value(val).unwrap_or(serde_json::Value::Null)
    )
}

fn lww_text(key: &str, display: &str, perm: PermissionTier) -> AttributeKeyDef {
    AttributeKeyDef {
        key: key.into(), display_name: display.into(),
        attr_type: AttributeType::Text,
        crdt_semantics: CrdtSemantics::LastWriteWins,
        write_permission: perm,
        ..Default::default()
    }
}

fn lww_json(key: &str, display: &str, perm: PermissionTier) -> AttributeKeyDef {
    AttributeKeyDef { attr_type: AttributeType::Json, ..lww_text(key, display, perm) }
}

fn orset_json(key: &str, display: &str, perm: PermissionTier) -> AttributeKeyDef {
    AttributeKeyDef {
        crdt_semantics: CrdtSemantics::OrSet,
        ..lww_json(key, display, perm)
    }
}

fn lww_number(key: &str, display: &str, perm: PermissionTier) -> AttributeKeyDef {
    AttributeKeyDef { attr_type: AttributeType::Number, ..lww_text(key, display, perm) }
}

fn pn_number(key: &str, display: &str, perm: PermissionTier) -> AttributeKeyDef {
    AttributeKeyDef { crdt_semantics: CrdtSemantics::PNCounter, ..lww_number(key, display, perm) }
}

// Read-side helpers
trait HyperCellMapExt {
    fn text(&self, field: &str) -> String;
    fn json_of(&self, field: &str) -> serde_json::Value;
    fn num(&self, field: &str) -> f64;
    fn from_json<T: serde::de::DeserializeOwned + Default>(&self, field: &str) -> T;
}

impl HyperCellMapExt for HashMap<&str, &TypedAttrValue> {
    fn text(&self, field: &str) -> String {
        self.get(field).and_then(|v| v.as_str()).unwrap_or_default().to_owned()
    }
    fn json_of(&self, field: &str) -> serde_json::Value {
        self.get(field)
            .and_then(|v| if let TypedAttrValue::Json(j) = v { Some(j) } else { None })
            .cloned().unwrap_or(serde_json::Value::Null)
    }
    fn num(&self, field: &str) -> f64 {
        self.get(field).and_then(|v| v.as_f64()).unwrap_or(0.0)
    }
    fn from_json<T: serde::de::DeserializeOwned + Default>(&self, field: &str) -> T {
        serde_json::from_value(self.json_of(field)).unwrap_or_default()
    }
}
```

---

# Part V — Domain OS Specifications: Kogi, Ume, Qala

## 43. Kogi Domain OS Specification

### 43.1 Identity Card

```
Domain OS Name:         Kogi
Domain:                 Independent Worker Operating System (IW-OS)
Root Domain Concept:    The Portfolio
Primary Hyperspreadsheet: kogi.portfolio.components
Root Entity Type:       PortfolioComponent (Item | Container)
Domain Prefix:          kogi://
Primary Language:       Rust (core) · Scala 3 (kogi-engine) · TypeScript (UI)
Hypercube Count:        8
Dimension Range:        N=2 to N=3
AI Engine Count:        12 sub-engines (kogi-engine)
Supported CrossGridLinks: Kogi↔Ume · Kogi↔Qala · Kogi↔Kogi (KLNK)
```

### 43.2 Root Domain Concept Specification

```
Root Concept: The Portfolio
Definition: The complete, unified record of an independent worker's professional
            and economic existence. Every entity in the worker's operational life
            is a row in the Portfolio System.

Portfolio is NOT:
  - A container for projects (it IS projects, tasks, gigs, assets, benefits,
    and everything else — simultaneously)
  - A feature of a project management tool
  - A static document or resume

Portfolio IS:
  - A living N-dimensional hyperspreadsheet
  - The single source of truth for all work, income, resources, and relationships
  - A node in the global Kogi economic graph (KLNK)
  - A governed, versioned, AI-augmented operational system

Root Concept Lifecycle States (Lattice v2):
  Draft → Active → Paused → Active (cycles allowed)
                    ↓
                 Completed → Archived
```

### 43.3 Hypercube Specification

```
kogi.portfolio.components (N=2, Hybrid encoding) [PRIMARY]
  D₁: EntityAxis (Uuid) — ComponentId
  D₂: PropertyAxis (String) — field_name
  38 attribute keys (see §22 of Master Design Doc for complete table)
  AI attributes: health_score, risk_score, alignment_score, anomaly_flag,
                 narrative_summary, income_projection_90d, coverage_gap_flags
  Mandatory CRDT: OR-Set on owners, tags, children, dependencies, links, toolbox_ids
  Lifecycle Lattice: ComponentStatus (Draft|Active|Paused|Completed|Archived|Deleted)

kogi.portfolio.kpis (N=3, Sparse extension)
  D₁: EntityAxis (Uuid) — ComponentId
  D₂: PropertyAxis (String) — metric_name
  D₃: TimeAxis (Timestamp) — period (quarter, sprint, month)
  Purpose: time-series tracking of health_score, risk_score, alignment_score, velocity

kogi.portfolio.finances (N=3, Sparse extension)
  D₁: EntityAxis (Uuid) — AccountId
  D₂: PropertyAxis (String) — field_name
  D₃: TimeAxis (Timestamp) — period
  Purpose: income, expense, budget records with period-sliced analysis

kogi.portfolio.benefits (N=2, Dense)
  D₁: EntityAxis (Uuid) — BenefitId
  D₂: PropertyAxis (String) — field_name
  Purpose: portable benefits coverage records

kogi.portfolio.actions (N=2, Dense)
  kogi.portfolio.approvals (N=2, Dense)
  kogi.portfolio.links (N=2, Dense)
  kogi.portfolio.snapshots (N=2, Dense)
  Purpose: supporting records (see §22 of Master Design Doc)
```

### 43.4 Graph Topology Specification

```
Required edge types in Kogi PortfolioGraph:
  Hierarchy    — Portfolio→Program→Project→Task (no cycle detection)
  Dependency   — Any→Any (DFS cycle detection enforced)
  Contains     — Container→Item (Binder containment)
  Association  — Any→Any (soft cross-reference, no constraints)
  CrossGridLink — Any→Any (consent required, ShadowCell created)

KLNK-specific edge types (via CrossGridLink):
  Collaborates, InvestedIn, Employs, Contracted, Follows, Endorses,
  SharedPortfolio, MarketplaceTransaction, OrgMembership, DependsOn,
  ResourceShares, FederationPeer
  (See §24 of Master Design Doc for complete KLNK edge type table)
```

### 43.5 AI Engine Specification

```
kogi-engine (Scala 3, Kafka consumer, gRPC WritebackService client)
  Parallelism: 32 concurrent compute tasks per Kafka partition
  Event source: Kafka topic "events.audit.{kogi_grid_id}"

Sub-engines (12 total):
  KogiHealthScoreEngine   → health_score        in kogi.portfolio.kpis
  KogiRiskEngine          → risk_score          in kogi.portfolio.kpis
  KogiAlignmentEngine     → alignment_score     in kogi.portfolio.kpis
  KogiCollaborationEngine → collaboration_score in kogi.portfolio.kpis
  KogiMaturityEngine      → maturity_score      in kogi.portfolio.components
  KogiAnomalyEngine       → anomaly_flag        in kogi.portfolio.components
  KogiMatchEngine         → match_score         in kogi.portfolio.links
  KogiIncomeProjectionEngine → income_projection_90d in kogi.portfolio.finances
  KogiBudgetEngine        → budget_burn_rate    in kogi.portfolio.finances
  KogiNarrativeEngine     → narrative_summary   in kogi.portfolio.components
  KogiScheduleRiskEngine  → predicted_completion_date in kogi.portfolio.components
  KogiCoverageGapEngine   → coverage_gap_flags  in kogi.portfolio.benefits

Oba AI Assistant:
  Interface: NL-to-HyperQL + HyperQL execution + EventLog annotation
  Modes: Reactive | Proactive | ColumnCompletion | SmartFilter |
         FormulaSuggestion | BatchAction | HealthBriefing | NarrativeGeneration | Autonomous
```

### 43.6 Sheet Specification (35 HypercubeViews)

All 35 Kogi sheets are registered as HypercubeViews on `kogi.portfolio.components`.
See §23 of the Master Design Doc for the complete table.
Each sheet is a saved DimSlice + visible attribute key set + render mode + default sort/group.

### 43.7 Space Taxonomy

```
SpaceType → GovernanceConfig:
  Personal     → owner-only; no vote required; no quorum
  Team         → manager approval for member ops; simple majority for governance
  Community    → member vote; configurable quorum (default: 50%+1); Shapley distribution
  Federation   → representative democracy; federated peer trust
  Project      → steward-controlled with campaign governance
  Organization → admin-controlled; inherits org's governance model
```

---

## 44. Ume Domain OS Specification

### 44.1 Identity Card

```
Domain OS Name:           Ume
Domain:                   Business / Organization Operating System (B-OS)
Root Domain Concept:      The Organization
Primary Hyperspreadsheet: ume.kernel.modules (kernel registry)
                          + 42 module Hypercubes (domain data)
Root Entity Type:         OrgModule (42 domain areas) / any org entity
Domain Prefix:            ume://
Primary Language:         Rust (kernel) · Go (services) · Python (AI)
Hypercube Count:          44 (ume.kernel.modules + 42 module cubes + ume.templates.library)
Dimension Range:          N=2 to N=4
AI Engine Count:          8 core capabilities (ume-engine)
Supported CrossGridLinks: Ume↔Kogi · Ume↔Qala
```

### 44.2 Root Domain Concept Specification

```
Root Concept: The Organization
Definition: The complete, unified operational model of a business entity.
            Every function, every record, every process, every decision
            that constitutes the organization's existence is a row in
            the Organization System.

Organization Hyperspreadsheet Architecture:
  Not a single cube (too complex); instead: one MASTER module registry cube
  + 42 domain-specific sub-cubes, each representing one organizational function.
  The 42 cubes collectively ARE the Organization System.

Module Registry (ume.kernel.modules):
  Every organization function (Finance, HR, Legal, Marketing, etc.) is a
  registered module — a HyperRow in the module registry.
  The module registry IS the organization chart at the function level.

Organization Lifecycle States (Lattice):
  Registered → Configured → Active → Suspended → Archived
```

### 44.3 42 Module Hypercube Specification

See §32 of the Master Design Doc for the complete table of all 42 module Hypercubes, their dimensions, and key attribute fields.

Key architectural notes:
- Modules 13 (Chombo) and 22 (Soko) use N=3 and N=4 respectively (see §33, §34)
- Modules 02, 11, 17, 30 use N=4 (Entity × Property × Time × Category)
- Modules 38, 26, 12 use N=2 only (no temporal or categorical extensions needed)
- Modules 41, 42 are extensible namespace slots for custom org-specific modules

### 44.4 Kernel-to-Hypergrid Mapping

See §31 of the Master Design Doc for the complete mapping of all UmeKernel services to Hypergrid infrastructure.

### 44.5 AI Engine Specification

```
ume-engine (Go + Python, Kafka consumer, gRPC WritebackService client)
  Capabilities:
    UmeModuleHealthEngine     → health_score          in ume.kernel.modules
    UmeChomboAI               → compliance_prediction in ume.chombo.entities
    UmeStrategyAI             → okr_completion_prob   in ume.strategy.okrs
    UmeHRAI                   → attrition_risk_score  in ume.hr.employees
    UmeFinanceAI              → revenue_forecast_q    in ume.sales.opportunities
    UmeRiskAI                 → correlated_risk_ids   in ume.grc.risks
    UmeSokoAI                 → optimization_suggestions in ume.soko.campaigns
    UmeAnomalyEngine          → anomaly_flag          in all module cubes
```

---

## 45. Qala Domain OS Specification

### 45.1 Identity Card

```
Domain OS Name:           Qala
Domain:                   Solution Factory Operating System (SF-OS)
Root Domain Concept:      The Solution (the universal category)
Primary Hyperspreadsheet: qala.solutions
Root Entity Type:         Solution (Application|System|Good|Product|Service|Platform)
Domain Prefix:            qala://
Primary Language:         Rust (core) · Go (services) · Scala (execution plane)
Hypercube Count:          12 (see §41 of Master Design Doc)
Dimension Range:          N=2 to N=4
AI Engine Count:          10 capabilities (qala-engine) + Domain Pack plugins
Supported CrossGridLinks: Qala↔Kogi · Qala↔Ume
Self-governing:           YES — Kogi and Ume are HyperRows in qala.solutions
```

### 45.2 Root Domain Concept Specification

```
Root Concept: The Solution (Universal Category)
Definition: Any purposeful output designed to address a problem, fulfill a goal,
            achieve an objective, or produce an intended outcome.
            The Solution IS the universal category — everything in the system IS
            a Solution or exists to produce Solutions.

Canonical Solution Types:
  Application — software with defined processes and user interactions
  System      — coordinated collection of applications
  Good        — tangible, physical, or digital deliverable
  Product     — commercially packaged, versioned, distributable artifact
  Service     — ongoing capability delivered to consumers
  Platform    — foundation on which other Solutions are built

Special Solution subtypes:
  Solution Factory (is_factory=true) — a Solution that produces other Solutions
  Platform Solution — Kogi, Ume, and Qala themselves are Platform Solutions

Solution Lifecycle States (Lattice CRDT):
  Draft → InReview → Approved → Active → Deprecated → Retired
  CCR gates: InReview→Approved requires all CCR approvers signed
  Concurrent transitions: resolved by lattice join (forward state wins)

Solution Maturity Stages (separate from lifecycle):
  SANDBOX → DEV → NIGHTLY → TEST → CM (Control Managed)
  Promotion gates: test pass rate, build attestation, security scan
```

### 45.3 Hypercube Specification

See §41 of the Master Design Doc for the complete Qala Hypercube inventory.

Key specifications:
- `qala.sdes` (N=3): full AS_OF time-travel; rollback via temporal write
- `qala.solutions.metrics` (N=4): multi-dim quality analysis
- `qala.ccrs`: Lattice CRDT on status; OR-Set on approver_ids
- Domain Pack cubes: installed per factory; Domain Packs add attribute keys

### 45.4 Self-Governance Specification

```
Root Factory:
  factory_id:      ROOT_FACTORY_UUID (well-known, published)
  root_factory_flag: true
  name:            "Qala Root Factory"
  
Platform Solutions in Root Factory (qala.solutions):
  KOGI_PLATFORM_UUID: Kogi IW-OS, Platform type, Software Domain Pack
  UME_PLATFORM_UUID:  Ume B-OS, Platform type, Software Domain Pack
  QALA_PLATFORM_UUID: Qala SF-OS, Platform type, Software Domain Pack (self-referential)
  HYPERGRID_UUID:     Hypergrid NDSS, Platform type
  
Every Kogi and Ume release MUST go through:
  1. CCR submission in Root Factory (new HyperRow in qala.ccrs)
  2. AI Agent risk assessment (AiSignal in CCR's HyperRow)
  3. Approval workflow (GovernanceEngine + Lattice CRDT on ccr.status)
  4. Release record creation (new HyperRow in qala.releases)
  5. Lifecycle state transition in the platform Solution's HyperRow
```

### 45.5 AI Agent Specification

See §46 of the Master Design Doc for all 10 AI Agent capabilities with plugins, output attributes, and triggers.

---

## 46. Cross-Domain OS Integration Specification

### 46.1 Kogi ↔ Ume Integration

```
Integration:  Worker employment
Edge Type:    Employs (Directed: Ume org → Kogi worker)
Consent:      Required from worker
Mirrored:     work_record fields (tasks, gigs, contributions, availability)
Writeback:    None (read-only from org to worker)
Triggers on:  Employs edge acceptance → ShadowCell provisioned in Ume Grid
              Worker updates work record → shadow syncs to Ume Grid

Integration:  Cooperative membership
Edge Type:    OrgMembership (Directed: Ume org → Kogi worker)
Consent:      Required from worker
Mirrored:     public portfolio columns (name, status, tags, health_score)
Writeback:    Org may update member.visibility and member.role in Kogi
```

### 46.2 Kogi ↔ Qala Integration

```
Integration:  Worker contribution to solution
Edge Type:    CrossGridLink (Kogi portfolio item → Qala solution)
Consent:      Required from worker
Mirrored:     solution lifecycle_state, version, release_date
Writeback:    None
Attribution:  kogi.portfolio.links tracks attribution_weight per worker per solution

Integration:  Worker gig as SDE contribution
Edge Type:    Collaborates (Kogi gig entity ↔ Qala SDE team_members)
Consent:      Via contribution attribution workflow
Mirrored:     SDE contribution record, attribution weight
```

### 46.3 Ume ↔ Qala Integration

```
Integration:  Organization's product catalog linked to its Qala solutions
Edge Type:    Produces (Directed: Ume product entity → Qala solution)
Consent:      None — same organization's federated deployment
Mirrored:     lifecycle_state, version, quality_score, defect_density
Writeback:    None

Integration:  Ume OKRs cascading to Qala SDE delivery goals
Edge Type:    Association (Ume OKR → Qala SDE)
Consent:      None — organizational relationship
Mirrored:     OKR progress, key_result targets, quarter
```

---

# Appendices

## Appendix A: CrdtOperation Reference

| Operation | Fields | Effect | Idempotent? |
|-----------|--------|--------|-------------|
| `SetAttr` | coord, attr_key, value, timestamp, actor | LWW write; winner by VectorClock | Yes (same ts) |
| `AddToSet` | coord, attr_key, element, unique_tag, actor | OR-Set add with unique tag | Yes (same tag) |
| `RemoveFromSet` | coord, attr_key, unique_tag, actor | OR-Set remove by exact tag | Yes |
| `IncrCounter` | coord, attr_key, delta, node_id | Per-node counter increment | No (accumulates) |
| `TransitionState` | coord, attr_key, from_state, to_state, actor, ts | Lattice join; validates transition | Yes (same join) |
| `AppendLog` | coord, attr_key, entry, causal_ts | Causally-ordered append | Yes (same causal_ts) |
| `DeepMergeJson` | coord, attr_key, delta, timestamp, actor | LWW per leaf JSON key | Yes (same ts) |
| `AddDimension` | cube_id, axis, schema_ts | Adds D₃+ axis to Hypercube | Yes |
| `RegisterAttrKey` | cube_id, attr_def, schema_ts | Adds attribute key to registry | Yes |
| `TombstoneAttrKey` | cube_id, attr_key, schema_ts | Marks key as removed | Yes |
| `AddEdge` | edge: HypergraphEdge | Adds typed edge to Hypergraph | Yes (same edge_id) |
| `RemoveEdge` | edge_id | Removes edge from Hypergraph | Yes |
| `RegisterNamespace` | path, entity_ref | Registers NamespacePath URI | Yes |
| `AddFederationPeer` | peer: FederationPeer | Adds peer to federation | Yes |

---

## Appendix B: HyperQL Grammar (Extended)

```
Query         ::= SelectClause FromClause [AsOf] [Where] [Fold] [Expand]
                  [Traverse] [Join] [Shadow] [Order] [Limit]

SelectClause  ::= SELECT Projection (COMMA Projection)*
Projection    ::= STAR | CellRef [AS alias] | DimRef [AS alias]
                | EXPAND DimName AS COLUMNS (AggFn)
                | FOLD DimName WITH AggFn [AS alias]
                | FormulaExpr AS alias

FromClause    ::= FROM CubeName [AS alias]
AsOf          ::= AS_OF StringLiteral   -- ISO 8601 datetime
Where         ::= WHERE Predicate (AND|OR Predicate)*
Predicate     ::= CellRef CompOp Value | CellRef IN (Values)
                | CellRef CONTAINS Value | DimRef BETWEEN Value AND Value
                | GRAPH_EDGE (NodeRef, NodeRef, EdgeType)
                | IS NULL | IS NOT NULL | NOT Predicate | (Predicate)
CompOp        ::= EQ | NEQ | GT | GTE | LT | LTE | LIKE | ILIKE

Fold          ::= FOLD DimName WITH AggFn [AS alias] [LAST n PERIODS]
Expand        ::= EXPAND DimName AS COLUMNS (AggFn)

Traverse      ::= TRAVERSE GRAPH FROM NodeRef
                  EDGE_TYPE = EdgeType DIRECTION = Direction
                  [MAX_DEPTH = Integer] [FILTER Where]

Join          ::= JOIN CubeName [AS alias] ON GRAPH_EDGE (source, target, EdgeType)
                | JOIN CubeName [AS alias] ON CellRef = CellRef  -- value join

Shadow        ::= SHADOW INCLUDE FROM GridRef

Order         ::= ORDER BY SortSpec (COMMA SortSpec)*
SortSpec      ::= (CellRef | DimRef | alias) (ASC | DESC)
Limit         ::= LIMIT Integer [OFFSET Integer]

CellRef       ::= cell[D1_ref, D2_ref [, D3_ref [, D4_ref]]].attr_key
D1_ref        ::= entity_uuid | CURRENT | DimRef
D2_ref        ::= StringLiteral | DimRef
D3_ref        ::= StringLiteral | DimRef | CURRENT_PERIOD | LAST_PERIOD
AggFn         ::= AVG | SUM | MIN | MAX | COUNT | LAST | FIRST | STDDEV
                | PERCENTILE(n) | ARRAY_AGG | DISTINCT_COUNT
EdgeType      ::= Hierarchy | Dependency | Association | Contains | CrossGridLink
                | Collaborates | Employs | InvestedIn | FederationPeer | Custom(name)
Direction     ::= Inbound | Outbound | Both
GridRef       ::= StringLiteral  -- e.g. "kogi://alice-coop/" or "qala://my-factory/"
```

---

## Appendix C: Domain OS Implementation Checklist

```
Phase 1 — Domain Design
  [ ] Root domain concept named and justified (ROOT-CONCEPT-RULE-001)
  [ ] All domain entities classified: root concept | supporting | graph-only
  [ ] Graph topology designed: all edge types with consent/cycle requirements
  [ ] CRDT semantics assigned for every field using decision tree
  [ ] AI signals identified: which fields are Tier-2 computed?
  [ ] Space taxonomy defined: Personal|Team|Community|Federation|Project|Org
  [ ] NamespacePath URI scheme defined: {domain}://{space}/{entity_type}/{id}/
  [ ] Cross-system integration points identified: Kogi|Ume|Qala|other

Phase 2 — Schema Implementation
  [ ] Domain entity structs defined (pure Rust, no storage concerns)
  [ ] Error types defined: DomainError, DomainResult<T>
  [ ] All Hypercube names follow {domain}.{entity_type} convention
  [ ] All cubes use Hybrid encoding for entity×field base
  [ ] All attribute keys have explicit CRDT semantics assignment
  [ ] All attribute keys have explicit PermissionTier assignment
  [ ] All AI-computed attrs use PermissionTier::System
  [ ] All mandatory attribute keys present (DOMAIN-OS-SPEC-002)
  [ ] DomainStore codec implements bootstrap/write/read/exists
  [ ] Round-trip test: read(write(entity)) == entity for all entity types

Phase 3 — Root Orchestrator
  [ ] Root orchestrator struct owns Grid (or Grid reference)
  [ ] All domain operations exposed as typed methods (no HyperCell in API)
  [ ] All mutating methods emit dual-write events (DOMAIN-OS-SPEC-007)
  [ ] Dual-write verified: domain event log AND Hypergrid EventLog updated
  [ ] hyperql() passthrough exposed (DOMAIN-OS-SPEC-009)
  [ ] stats() method exposes both domain and Hypergrid substrate metrics (SPEC-010)
  [ ] NamespacePath assigned and registered for all created entities (SPEC-008)
  [ ] Permission check before all mutating operations
  [ ] Cycle detection enforced on all Dependency edge additions

Phase 4 — Graph and Federation
  [ ] Domain graph wrapper implements: add_edge, remove_edge, reachable_from, detect_cycle
  [ ] All required EdgeTypes supported (DOMAIN-OS-SPEC-003)
  [ ] DFS cycle detection tested with 3+ node cycles
  [ ] CrossGridLink protocol tested: consent → ShadowCell → sync → writeback
  [ ] Federation delta sync tested: offline → reconnect → merge

Phase 5 — AI Integration
  [ ] All AI engines registered as HypercubePlugin implementations
  [ ] on_cell_mutation() returns correct attr keys for each triggering field
  [ ] WritebackService gRPC connection configured and tested
  [ ] AI result written as TypedAttrValue::AiSignal with confidence score
  [ ] Cache TTL configured per attr (default: 3600s; tune per model freshness)
  [ ] Stale AI value handling: AiSignal.stale_at set; UI shows "updating" indicator

Phase 6 — Testing
  [ ] CRDT round-trip: concurrent write from 2 nodes; merge; verify correct result
  [ ] OR-Set: concurrent add+add; concurrent add+remove; concurrent remove+add
  [ ] Lattice: valid transition; invalid transition; concurrent transitions; LUB
  [ ] AS_OF: write at T1; write at T2; AS_OF(T1) returns T1 state
  [ ] Federation: 3-node cluster; partition; heal; verify convergence
  [ ] CrossGridLink: full 6-phase protocol test
  [ ] Performance: HyperRow read <40ms p99; DimSlice <200ms p99
  [ ] AI writeback: mutation → queue → compute → WritebackService → cell updated
```

---

## Appendix D: Error Reference

| Error | Code | Description | Retry? |
|-------|------|-------------|--------|
| `HypergridError::PermissionDenied` | HG-401 | caller.tier < attr.write_permission | No — fix permissions |
| `HypergridError::NotFound` | HG-404 | Entity or cube not found | No |
| `HypergridError::AlreadyExists` | HG-409 | Cube or attr key already registered | No — idempotent |
| `HypergridError::CrdtConflict` | HG-409C | LWW losing write (non-fatal, recorded) | No — recorded only |
| `CrdtError::GrowOnlyViolation` | HG-422G | Attempted decrement of GrowOnly counter | No — wrong op type |
| `CrdtError::InvalidStateTransition` | HG-422L | Lattice transition not in partial order | No — invalid request |
| `CrdtError::WrongCrdtType` | HG-422T | CRDT op doesn't match attr semantics | No — code bug |
| `HypergridError::CyclicDependency` | HG-422C | Dependency edge would create cycle | No — restructure graph |
| `HypergridError::StorageError` | HG-503 | PostgreSQL / Redis / Kafka unreachable | Yes — with backoff |
| `HypergridError::FederationError` | HG-503F | Federation peer unreachable | Yes — standalone mode |
| `HypergridError::PluginError` | HG-503P | Plugin computation failed | Partial — degrade |
| `HypergridError::SchemaConflict` | HG-500S | Destructive schema change without gate | No — fix procedure |
| `TimeTravelError::BeyondHotWindow` | TT-416 | AS_OF target older than hot EventLog | Partial — restore from S3 |

---

## Appendix E: Performance Benchmarks

Reference benchmarks on a standard production node (8 vCPU, 32GB RAM, NVMe SSD):

| Operation | p50 | p99 | p999 | Notes |
|-----------|-----|-----|------|-------|
| Single HyperCell write | 3ms | 12ms | 45ms | Includes PG UPSERT + EventLog |
| Single HyperCell read (cache hit) | 0.2ms | 1ms | 3ms | Redis L1 cache |
| Single HyperCell read (cache miss) | 4ms | 18ms | 60ms | PostgreSQL primary key |
| HyperRow read, 30 fields (cache hit) | 0.3ms | 1.5ms | 5ms | Redis HGETALL |
| HyperRow read, 30 fields (cache miss) | 8ms | 35ms | 120ms | PG D₁ scan |
| Batch write, 30 fields | 12ms | 40ms | 150ms | 1 PG transaction |
| DimSlice, N=2, 1K rows | 35ms | 180ms | 500ms | PG GIN/BTree index |
| DimSlice, N=2, 100K rows | 400ms | 1.8s | 5s | PG full scan + visibility |
| DimFold, N=3, 50K cells | 250ms | 900ms | 3s | PG window functions |
| DimFold, N=3, 5M cells | 1.5s | 6s | 20s | ClickHouse |
| Graph BFS, depth=3, fan-out=10 | 45ms | 200ms | 800ms | PG recursive CTE |
| Graph BFS, depth=5, fan-out=20 | 200ms | 800ms | 3s | PG recursive CTE |
| CRDT merge, in-memory | 0.5ms | 4ms | 12ms | No IO |
| AS_OF, hot window (< 10K events) | 80ms | 400ms | 1.2s | EventLog replay |
| AS_OF, cold (> 10K events) | 300ms | 1.5s | 5s | PG EventLog |
| AS_OF, S3 archive | 4s | 15s | 60s | S3 restore + replay |
| CrossGridLink sync (real-time) | 300ms | 1s | 4s | Kafka + ShadowCell write |
| NL-to-HyperQL translation | 200ms | 800ms | 2s | LLM inference |
| AI Tier-2 computation (simple) | 80ms | 350ms | 1.2s | Plugin compute |
| AI Tier-2 computation (LLM) | 800ms | 3s | 10s | LLM inference |
| Federation heartbeat sync | 150ms | 600ms | 2s | Kafka delta |
| Full portfolio load (500 rows, 35 views) | 1.5s | 4.5s | 12s | Parallel reads + filters |

Scaling limits:
- Max entities per Hypercube: ~1 billion (PG partition limit per grid shard)
- Max attribute keys per Hypercube: 1,024 (AttributeKeyRegistry hard limit)
- Max dimensions per Hypercube: 16 (VectorClock-compatible; >6 not recommended for UI)
- Max federation peers per Grid: 255 (VectorClock size limit)
- Max ClickHouse throughput: ~100M cells/second for DimFold
- CRDT VectorClock max nodes: 255 (HashMap<NodeId, u64>)

---

*End of Document*

*Hypergrid & Apapo — Execution and Computational Models · Domain OS Specification · v1.0 · March 2026*

*Confidential — Internal Use Only*
