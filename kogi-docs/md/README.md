# Kogi Platform — Rust Crates

## `hypergrid` + `kogi-portfolio`

Two Rust crates implementing the complete design from:
- Hypergrid NDSS SDD v1.0
- Kogi Portfolio System Design v2.1

---

## `kogi-marketplace`

- Rust core: `kogi-marketplace` (listings, deals, campaigns, snapshots)
- Go service: `kogi-network/services/marketplace`
- Source of truth: `kogi-portfolio` Master Spreadsheet (`SHT-023 Marketplace`)
- Docs: `marketplace-backend.md`

## Workspace layout

```
Cargo.toml                    ← workspace root
hypergrid/
  Cargo.toml
  src/
    lib.rs                    ← module tree + crate-level re-exports
    error.rs                  ← HypergridError, HypergridResult
    crdt.rs     HG-CRDT       ← VectorClock, CrdtLog, CrdtOperation,
                                 OR-Set, PN-Counter, LWW, LatticeOrder
    cell.rs     HG-CELL       ← HyperCell, DimCoordinate, DimKey,
                                 AttributeMap, AttributeKeyRegistry,
                                 TypedAttrValue, PermissionTier
    dim.rs      HG-DIM        ← DimensionAxis, AxisType, KeyType,
                                 DimSlicePredicate, DimFold, DimExpand
    core.rs     HG-CORE       ← Grid, Hypercube, UniversalCellStore,
                                 EventLog, EventEntry, EventKind
    graph.rs    HG-GRAPH      ← Hypergraph, HypergraphNode,
                                 HypergraphEdge, EdgeType,
                                 ShadowCell, LinkForest
    space.rs    HG-SPACE      ← Space, SpaceType, Workspace,
                                 GovernanceConfig, SpaceMember
    view.rs     HG-VIEW       ← HypercubeView, RenderMode,
                                 ViewSort, ViewGroup, HighlightRule
    query.rs    HG-QL         ← HyperQuery, QueryResult, QueryRow
    plugin.rs   HG-PLUGIN     ← HypercubePlugin trait, ComputeContext
kogi-portfolio/
  Cargo.toml                  ← depends on hypergrid = { path = "../hypergrid" }
  src/
    lib.rs                    ← module tree + re-exports (incl. hypergrid types)
    error.rs                  ← PortfolioError, PortfolioResult
    types.rs                  ← all enums & primitives (ComponentStatus,
                                 ItemCategory, ActionKind, RiskSeverity …)
    metadata.rs               ← ComponentMetadata (uses hypergrid VectorClock)
    crdt.rs                   ← re-exports hypergrid CRDT primitives;
                                 adds PortfolioCrdtLog, CrdtOperation
    component.rs              ← Component (Item | Container), ComponentData,
                                 ComponentUsers, ComponentAnalytics,
                                 all item/container payloads
    graph.rs                  ← PortfolioGraph (wraps hypergrid Hypergraph),
                                 PortfolioEdge, PortfolioEdgeType,
                                 Group/Collection/List/Schedule/Directory
    governance.rs             ← PolicyEngine trait, PolicyDecision,
                                 ApprovalWorkflow, ResourceAllocation,
                                 GovernanceEngine
    models.rs                 ← 11 built-in analytical models:
                                 PortfolioHealth, ProjectMetrics,
                                 ProgramAlignment, SubPortfolioRollup,
                                 ResourceUtilisation, AssetValue,
                                 ArtifactMaturity, BinderCoverage,
                                 BookConsistency, FolderOrganisation,
                                 RecordIntegrity
    events.rs                 ← EventLog (wraps hypergrid EventLog,
                                 dual-write), PortfolioEvent,
                                 PortfolioEventKind, Snapshot
    store.rs                  ← ComponentStore — Component ↔ HyperCell
                                 serialisation bridge (write + read)
    system.rs                 ← PortfolioSystem root orchestrator
    search.rs                 ← SearchQuery, PortfolioQuery (PQL), SearchResult
    plugin.rs                 ← PortfolioPlugin trait + lifecycle hooks
    federation.rs             ← PortfolioFederation, FederationPeer
    itembook.rs               ← ItemBookData, BookPayload, Dashboard,
                                 Charter, Catalogue, MetricEntry
    collaboration.rs          ← ContributionRecord, CrowdresourcingCampaign
    benefits.rs               ← BenefitAccount (portable benefits as items)
```

---

## Architecture: kogi-portfolio on top of hypergrid

```
kogi-portfolio (domain layer)
  PortfolioSystem
    ├── grid: hypergrid::core::Grid          ← all cell storage + CRDT
    │    └── components Hypercube            ← D1=ComponentId, D2=field_name
    │         every Component field is one HyperCell at (id, field_name)
    │         cell.attributes["value"] = typed field value
    ├── graph: PortfolioGraph                ← wraps hypergrid::graph::Hypergraph
    │    portfolio edge types → HG EdgeType
    ├── event_log: EventLog                  ← dual-writes to hypergrid EventLog
    │    every portfolio event → hypergrid::core::EventKind::Custom(tag)
    │    gives unified, time-travelable audit trail (AS_OF via hg EventLog)
    ├── crdt_log: PortfolioCrdtLog           ← wraps hypergrid::crdt::CrdtLog
    │    portfolio CrdtOperation → hg-level SetField/AddEdge ops for federation
    ├── governance: GovernanceEngine         ← pure portfolio-layer logic
    └── plugins: Vec<Arc<dyn PortfolioPlugin>>
```

### Cell layout in the components Hypercube

```
D1 = ComponentId (UUID)    ← entity / row axis
D2 = field name  (String)  ← property / column axis

coord(component_id, "name")         → cell.value = Text("My Portfolio")
coord(component_id, "status")       → cell.value = Json("Active")
coord(component_id, "payload")      → cell.value = Json({...})
coord(component_id, "budget")       → cell.value = Number(50000.0)
coord(component_id, "owners")       → cell.value = Json([uuid1, uuid2])
coord(component_id, "tags")         → cell.value = Json(["saas","b2b"])
coord(component_id, "vector_clock") → cell.value = Json({node-1: 5, node-2: 3})
```

### Per-field CRDT semantics (from the SDD)

| Field group      | CRDT              | Hypergrid type         |
|------------------|-------------------|------------------------|
| name, description, status, state, visibility, version | LWW | `CrdtSemantics::LastWriteWins` |
| category, payload, children, dependencies, risks … | LWW | `CrdtSemantics::LastWriteWins` |
| owners, tags, hashtags, topics, toolbox_ids | OR-Set | `CrdtSemantics::OrSet` |
| budget, resource_units | LWW | `CrdtSemantics::LastWriteWins` |
| budget_spent | PN-Counter | `CrdtSemantics::PnCounter` |

*Note: status should use a custom lattice in production (SDD §17.3); LWW is used as the v1 implementation.*

---

## Building

```bash
cargo build --workspace
cargo test --workspace
```

Requires Rust ≥ 1.75.

---

## Test coverage: 37 passing tests

### hypergrid (13 tests)
| Test | Module |
|------|--------|
| `dim_coordinate_validate` | cell |
| `attribute_registry_builtins` | cell |
| `hypercell_set_and_get` | cell |
| `create_cube_and_write_cell` | core |
| `add_custom_axis` | core |
| `vector_clock_happened_before` | crdt |
| `vector_clock_concurrent` | crdt |
| `or_set_add_and_remove` | crdt |
| `pn_counter_concurrent` | crdt |
| `entity_axis_default` | dim |
| `category_axis_key_validation` | dim |
| `add_nodes_and_edge` | graph |
| `cycle_detection` | graph |

### kogi-portfolio (24 tests)
| Test | Module |
|------|--------|
| `create_project_component` | component |
| `toolbox_attach_detach` | component |
| `dual_write_and_query` | events |
| `ring_buffer_cap` | events |
| `cycle_detection_simple` | graph |
| `list_move` | graph |
| `bump_version_uses_hypergrid_vc` | metadata |
| `portfolio_health_healthy` | models |
| `portfolio_health_distressed` | models |
| `asset_value_depreciation` | models |
| `project_metrics_risk_level` | models |
| `create_and_read_from_hypercube` | **system** — round-trip via HyperCells |
| `hypergrid_event_log_receives_events` | **system** — dual-write verified |
| `update_info_persists_to_hypercube` | **system** — mutation persists |
| `permission_denied_for_stranger` | **system** — RBAC enforced |
| `hierarchy_attach_uses_hypergraph` | **system** — graph integration |
| `dependency_cycle_detected_by_portfolio_graph` | **system** — cycle guard |
| `version_bump_persists_to_hypercube` | **system** — versioning |
| `budget_allocation_reflected_in_hypercube` | **system** — resource mgmt |
| `toolbox_attach_persists_and_lists` | **system** — TMS v2.1 |
| `search_filters_from_hypercube` | **system** — in-process search |
| `stats_expose_hypergrid_metrics` | **system** — hg_total_cells verified |
| `approval_workflow` | **system** — governance |
| `health_score_from_hypercube_data` | **system** — PortfolioHealth model |

---

## Key design decisions

**Why hypergrid as the substrate for kogi-portfolio?**

Every design principle in the Portfolio SDD maps directly onto a Hypergrid primitive:
- *"event-sourced, auditable"* → `hypergrid::core::EventLog` (immutable append-only, AS_OF time-travel)
- *"distributed-first, CRDT"* → `hypergrid::crdt::{VectorClock, CrdtLog, OrSet, PnCounter, LwwRegister}` — zero duplication
- *"everything is a Portfolio Item"* → everything is a HyperRow in the components Hypercube
- *"N-attribute cells"* → each component field is one `HyperCell` with typed `"value"` attribute
- *"graph relationships"* → `PortfolioGraph` wraps `hypergrid::graph::Hypergraph` directly; portfolio edge types (`Hierarchy`, `Dependency`, `Link`) map to HG `EdgeType` variants
- *"federation"* → `PortfolioCrdtLog` wraps `hypergrid::crdt::CrdtLog`; delta sync and `VectorClock` causality come from HG
- *"AI intelligence"* → the Hypercube's `attr_registry` already supports `AttributeType::Ai` and `AttrComputation::AiEngine`; the portfolio AI engine writes scores back as computed attributes — no extra infra needed

**What kogi-portfolio adds on top of hypergrid:**
- Domain-typed `Component` model with 7 Item types and 6 Container types
- `ComponentStore` serialisation bridge (Component ↔ HyperCells)
- 11 pure-compute analytical models (PortfolioHealth, ProjectMetrics, etc.)
- `PolicyEngine` trait + `GovernanceEngine` with `ApprovalWorkflow` and `ResourceAllocation`
- `PortfolioPlugin` lifecycle hooks
- Business-level `PortfolioEventKind` enum (dual-written into the hypergrid EventLog)
- `PortfolioCrdtLog` with portfolio-domain typed operations
- `PortfolioGraph` with domain-aware cycle detection on Dependency edges

**HyperQL integration point:**
`PortfolioSystem::hyperql()` exposes a pass-through to the underlying Grid's HyperQL engine. Advanced consumers (analytics, AI engine, dashboards) can issue full N-dimensional queries against the components Hypercube without going through the portfolio domain layer.
