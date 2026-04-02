# Hypergrid as a Universal Domain-Specific System Substrate

## Domain-Layer Architecture Pattern for Kogi, Ume, and Qala

---

**Document Type:** Reference Architecture & Design Document  
**Version:** 1.0  
**Date:** March 2026  
**Status:** Authoritative Reference  
**Scope:** Hypergrid N-Dimensional Distributed Spreadsheet System as the universal substrate for domain-specific operating systems
**Systems Covered:** Hypergrid (substrate) · Kogi (Independent Worker OS) · Ume (Business OS) · Qala (Solution Factory OS)

---

## Table of Contents

1. Introduction & Design Philosophy  
2. The Hyperspreadsheet as Universal Abstraction  
3. The Domain-Layer Pattern  
4. Hypergrid Core Architecture Review  
5. Kogi Platform on Hypergrid  
6. Ume Platform on Hypergrid  
7. Qala Platform on Hypergrid  
8. Cross-Platform Shared Infrastructure  
9. The Universal Cell Store: Data Gravity and Cross-System Queries  
10. Federation and Multi-System Synchronization  
11. AI and Intelligence Layer Integration  
12. Security, Governance, and Compliance  
13. Deployment Topology  
14. Implementation Guide for New Domain Systems  
15. Comparative Analysis  
16. Roadmap  
17. Appendices  

---

# Part I — Foundation

---

## 1. Introduction & Design Philosophy

### 1.1 The Central Thesis

Every complex software system, regardless of its domain, faces the same underlying architectural challenge: how to store, version, relate, compute over, and distribute structured data about entities that have properties, exist in multiple contexts simultaneously, change over time, and are operated on collaboratively by many actors at once.

Traditional approaches solve this problem by designing bespoke schemas for each domain. An Independent Worker OS stores portfolio items in a relational schema designed for portfolios. A Business OS stores organizational modules in their own relational schemas designed for organizations. A Solution Factory OS stores SDEs and solution models in schemas designed for solution factories. Each system invents its own change tracking, versioning, CRDT strategy, graph model, permission system, and event log from scratch — paying the full infrastructure cost again and again.

**Hypergrid's thesis is that this is unnecessary.** The N-dimensional spreadsheet is a universal model. Every domain entity can be described as "something with properties (columns), organized into groups (rows), that exists across multiple contexts simultaneously (extra dimensions)." Hypergrid makes that model distributed, collaborative, time-travelable, graph-aware, intelligent, and infinitely extensible — and then serves as the substrate from which any domain-specific system can be built.

| **The Core Insight** | A portfolio item, an organization module, a solution development environment, a marketing campaign, a legal entity, a sprint backlog, a product formulation — all of these are HyperRows in domain-specific Hypercubes sitting in the same Universal Cell Store, governed by the same CRDT logic, versioned by the same EventLog, connected by the same Hypergraph, and reasoned over by the same AI engine. Domain-specific systems are *lenses* over Hypergrid — they project a typed, governed, domain-named view onto what is ultimately a unified N-dimensional data substrate. |
|---|---|

### 1.2 The Three Systems in Context

This document examines three distinct domain operating systems that all adopt Hypergrid as their foundational data layer:

| System | Domain | Core Metaphor | Primary Entity | Scale |
|--------|--------|--------------|----------------|-------|
| **Kogi** | Independent Worker OS | Portfolio as living workspace | Component (Item \| Container) | Individual to cooperative |
| **Ume** | Organization / Business OS | Organization as supervised kernel | OrgModule (42 domain areas) | Team to enterprise |
| **Qala** | Solution Factory OS | Factory producing governed solutions | SDE / Solution | Solo developer to global enterprise |

Despite their wildly different domains, all three systems share identical underlying needs: they need entities with typed properties, lifecycle states, version history, graph relationships, CRDT-safe collaborative editing, immutable audit trails, AI-computed attributes, and time-travel queries. Hypergrid provides all of these at the substrate level so that each domain system can focus entirely on its domain logic rather than its data infrastructure.

### 1.3 How to Read This Document

Sections 1–4 establish the theoretical foundation. Sections 5–7 each take one of the three systems and describe in full detail how its domain model maps onto Hypergrid primitives — the Hypercube schemas, dimension axes, CRDT strategies, graph topology, and AI integration points. Section 8 covers the shared infrastructure that all three systems inherit. Sections 9–12 cover cross-cutting concerns. Section 13 covers deployment. Section 14 is a practical guide for building new domain systems on Hypergrid. Section 15 provides a comparative analysis. Section 16 covers the roadmap.

---

## 2. The Hyperspreadsheet as Universal Abstraction

### 2.1 Why a Spreadsheet?

The spreadsheet is, arguably, the most widely understood data model in the world. Virtually every domain expert — a financial analyst, a project manager, a scientist, a supply chain planner — reaches for a spreadsheet when they need to organize structured data. The reason is that the spreadsheet model captures three things that humans naturally think about data:

1. **Entities have properties** (rows × columns = every entity has the same set of named attributes)
2. **Entities exist in groups** (multiple rows = multiple instances of the same concept)
3. **Entities can be compared** (visual layout = easy scanning and comparison)

The problem with the conventional spreadsheet is that it only models two dimensions — rows and columns — and it is not distributed, versioned, or AI-aware. Hypergrid generalizes the spreadsheet to N dimensions, making it distributed-first, CRDT-native, graph-connected, and AI-augmented. This generalization turns the most familiar data model in the world into a universal substrate for arbitrary domain systems.

### 2.2 The N-Dimensional Model

Let H = (D₁, D₂, ..., Dₙ) be a Hypercube with N dimension axes. Every HyperCell C is addressed by a coordinate tuple (k₁ ∈ D₁.key_set, k₂ ∈ D₂.key_set, ..., kₙ ∈ Dₙ.key_set) and carries an N-attribute map C.attributes = { aᵢ: vᵢ }.

The conventional 2D spreadsheet is the N=2 special case:
- D₁ = RowAxis (entity axis) — the "what" dimension
- D₂ = ColumnAxis (property axis) — the "which property" dimension

Hypergrid extends this to N dimensions where each extra dimension provides a new axis of contextual organization:

| Dimension Index | Conventional Spreadsheet Equivalent | Hypergrid Name | Example Uses |
|-----------------|--------------------------------------|----------------|--------------|
| D₁ | Row | EntityAxis | User ID, Project ID, Employee ID, SDE ID |
| D₂ | Column | PropertyAxis | Field name, attribute key, metric name |
| D₃ | (none — new) | TimeAxis / CategoryAxis / Custom | Time period, scenario, version |
| D₄ | (none — new) | GeoAxis / TenantAxis / Custom | Region, org unit, tenant |
| D₅+ | (none — new) | Any custom axis | Domain-specific slicing axis |

### 2.3 Mapping Domain Concepts to Hypergrid Primitives

Every domain concept from all three systems maps cleanly onto Hypergrid primitives:

| Domain Concept (Any System) | Hypergrid Primitive | Notes |
|-----------------------------|---------------------|-------|
| An entity (portfolio item, org module, SDE) | HyperRow (all cells sharing D₁ key) | The canonical record |
| A property or field of an entity | D₂ key + cell.attributes["value"] | Each field is one column key |
| A typed field value | TypedAttrValue variant | Text, Number, Json, Enum, Bool, DateTime, etc. |
| An entity's history over time | D₃=TimeAxis with DimSlice | AS_OF queries |
| A lifecycle state | D₂="status" cell with LWW CRDT | Enum-typed, policy-governed |
| A tag / label / hashtag | D₂="tags" cell with OR-Set CRDT | Multi-valued, concurrent-safe |
| Version number | D₂="version" cell with MaxRegister CRDT | Always monotonically increasing |
| A relationship between entities | HypergraphEdge (any EdgeType) | Typed, weighted, directed |
| A hierarchy (parent-child) | HypergraphEdge::Hierarchy | Rollup, drill-down via graph traversal |
| A dependency between entities | HypergraphEdge::Dependency | With cycle detection |
| Permission level | PermissionTier enum on each attribute key | Per-key write access control |
| Audit trail | EventLog (append-only, time-indexed) | Every mutation captured |
| Multi-node sync | VectorClock + CrdtLog | Causal ordering guaranteed |
| AI-computed attribute | AttrComputation::AiEngine on any AttributeKeyDef | Score, classification, recommendation |
| Cross-system reference | HypergraphEdge::CrossGridLink + ShadowCell | Read-only reflection from another Grid |
| Namespace / address | NamespacePath URI | hypergrid://{grid}/{space_type}/{slug}/... |
| Time-travel query | AS_OF operator in HyperQL | Replay EventLog to any timestamp |

### 2.4 Why This Works for Domain Systems

The reason the N-dimensional model works as a universal substrate is that every domain system's data model is, at its core, a collection of entities with typed properties that exist in multiple contexts simultaneously and change over time. The domain system's job is to:

1. Define *which* entities exist (register Hypercubes with domain-specific names)
2. Define *what properties* those entities have (register AttributeKeyDefs with correct CRDT semantics)
3. Define *which extra dimensions* matter (add TimeAxis, CategoryAxis, TenantAxis as needed)
4. Define *how entities relate* (register EdgeType patterns in the Hypergraph)
5. Define *which computed properties* are AI-derived (register AiEngine computations)
6. Build domain-specific APIs, workflows, and UIs on top of the Hypercube reads and writes

All of the distributed consistency, versioning, graph traversal, AI integration, and time-travel comes from Hypergrid for free.

---

## 3. The Domain-Layer Pattern

### 3.1 Architecture Overview

Every domain system built on Hypergrid follows the same structural pattern. We call this the **Domain-Layer Pattern**. It consists of four layers:

```
┌─────────────────────────────────────────────────────────────────┐
│  PRESENTATION LAYER                                              │
│  Domain-specific UI · CLI · REST API · WebSocket · SDK          │
├─────────────────────────────────────────────────────────────────┤
│  DOMAIN LOGIC LAYER                                              │
│  Domain entities · Business rules · Lifecycle state machines    │
│  Governance policies · Analytical models · Domain workflows     │
├─────────────────────────────────────────────────────────────────┤
│  DOMAIN-HYPERGRID BRIDGE LAYER  (the "Store" or "Codec")        │
│  Entity ↔ HyperCell serialization · Schema registration         │
│  Attribute key definitions · CRDT semantics config              │
│  Graph edge type mappings · AI engine registration              │
├─────────────────────────────────────────────────────────────────┤
│  HYPERGRID SUBSTRATE LAYER                                       │
│  Grid · Hypercube · UniversalCellStore · Hypergraph             │
│  VectorClock · CrdtLog · EventLog · HyperQL · AI Engine         │
│  NamespaceSystem · SpaceSystem · Federation                     │
└─────────────────────────────────────────────────────────────────┘
```

The **Hypergrid Substrate Layer** is shared across all domain systems and never changes per domain. It is the platform.

The **Domain-Hypergrid Bridge Layer** is the critical integration point. It is responsible for translating domain concepts into Hypergrid primitives and back. For Kogi this is `ComponentStore`. For Ume this would be `OrgModuleStore`. For Qala this would be `SolutionStore`. The bridge registers all Hypercube schemas, all attribute key definitions, all CRDT semantics, all graph edge type mappings, and all AI engine bindings.

The **Domain Logic Layer** knows nothing about cells or coordinates. It operates entirely in domain terms — `Component`, `OrgModule`, `Solution`, `SDE`. It calls the bridge layer to persist and read, and calls the Hypergrid layer only for cross-cutting concerns like graph traversal, HyperQL queries, or AI model results.

The **Presentation Layer** is pure domain vocabulary. It never exposes Hypergrid internals to users.

### 3.2 The Codec Contract

Every domain system must implement one central codec — the Store — that fulfills the following contract:

```rust
pub trait DomainStore<Entity, EntityId>: Send + Sync {
    /// Register all Hypercubes and attribute schemas on startup.
    fn bootstrap(grid: &mut Grid) -> HypergridResult<CubeId>;

    /// Write a domain entity into the Hypercube as a set of HyperCells.
    fn write(grid: &mut Grid, cube_id: CubeId, entity: &Entity, actor: &str)
        -> HypergridResult<()>;

    /// Read all HyperCells for an EntityId and reconstruct the domain entity.
    fn read(grid: &Grid, cube_id: CubeId, id: EntityId)
        -> DomainResult<Option<Entity>>;

    /// Check whether an entity exists in the Hypercube.
    fn exists(grid: &Grid, cube_id: CubeId, id: EntityId) -> bool;
}
```

The codec contract enforces complete separation between domain model and storage substrate. The domain entity type (`Component`, `OrgModule`, `Solution`) is a pure Rust struct with no storage concerns. The Hypercube stores it as a set of HyperCells. The codec translates between the two.

### 3.3 Dimension Convention

All domain systems using Hypergrid adopt the following N=2 baseline convention for their primary entity Hypercubes:

```
D₁ = EntityAxis  (key_type: Uuid)        ← entity row (entity ID)
D₂ = PropertyAxis (key_type: String)     ← property column (field name)

cell at (entity_id, "field_name") → cell.attributes["value"] = <TypedAttrValue>
```

Every field of every domain entity becomes one HyperCell in this layout. The "value" attribute key is the Hypergrid builtin that carries the primary field value. This is the canonical cell layout used by all three domain systems.

Domain systems may extend to N>2 for specific Hypercubes that require multi-dimensional analysis:
- A time-series Hypercube for KPIs and metrics adds D₃=TimeAxis
- A multi-tenant Hypercube adds D₃=TenantAxis  
- A scenario/planning Hypercube adds D₃=ScenarioAxis
- An org-hierarchy Hypercube adds D₃=HierarchyAxis (D₃=org_unit)

### 3.4 Cubes Within a Domain System

Each domain system registers multiple Hypercubes in its Grid — one per major entity type. The naming convention is `{domain_name}.{entity_type}`:

| Domain System | Cube Name | Primary Entities |
|---------------|-----------|-----------------|
| Kogi | `kogi.portfolio.components` | All Component nodes (items, containers) |
| Kogi | `kogi.portfolio.actions` | ActionKind event records |
| Kogi | `kogi.portfolio.approvals` | ApprovalRequest records |
| Kogi | `kogi.portfolio.allocations` | ResourceAllocation records |
| Ume | `ume.kernel.modules` | All 42 OrgModule registrations |
| Ume | `ume.org.employees` | HR employee records |
| Ume | `ume.org.legal_entities` | Chombo legal entity records |
| Ume | `ume.org.campaigns` | Soko marketing campaigns |
| Ume | `ume.org.risks` | GRC risk register |
| Ume | `ume.org.audit_records` | Immutable audit trail |
| Qala | `qala.factories` | Solution Factory records |
| Qala | `qala.sdes` | Solution Development Environment records |
| Qala | `qala.solutions` | Solution records (all types) |
| Qala | `qala.ccrs` | Change Control Request records |
| Qala | `qala.releases` | Release records |
| Qala | `qala.artifacts` | Artifact records |
| Qala | `qala.toolchains` | Toolchain records |
| Qala | `qala.ai_recommendations` | AI Agent recommendation records |

Each Hypercube is a fully independent N-dimensional grid with its own axis configuration, attribute key registry, and CRDT semantics. They all share the same Hypergraph for cross-entity relationships and the same EventLog for unified audit.

### 3.5 CRDT Semantics Assignment

Every attribute key in every domain system's Hypercube must be assigned appropriate CRDT semantics. This is the most important design decision in the bridge layer — it determines how concurrent writes from multiple federation nodes are resolved.

The following table defines the CRDT assignment policy used by all three domain systems:

| Attribute Characteristic | CRDT Semantics | Rationale |
|--------------------------|----------------|-----------|
| Human-readable name, description, title | `LastWriteWins` | Text is authored by one person at a time; latest version wins |
| Lifecycle status (Draft, Active, Archived) | `LastWriteWins` (*) | Business logic governs transitions; raw LWW in v1 |
| Lifecycle state (Running, Blocked, Sealed) | `LastWriteWins` | Operational state; latest node wins |
| Visibility (Public, Private, Tenant) | `LastWriteWins` | Security setting; owner decision is latest |
| Set of owners or members | `OrSet` | Concurrent adds must all survive; removes are tagged |
| Tags, hashtags, labels, topics | `OrSet` | Concurrent adds always survive |
| Set of child IDs, dependency IDs | `OrSet` | DAG edges added concurrently from multiple nodes |
| Set of toolbox IDs | `OrSet` | OR-Set ensures no toolbox is silently lost |
| Version number | `MaxRegister` | Always choose highest version; never regress |
| Budget allocated | `LastWriteWins` | A single authorized actor sets the budget |
| Budget spent / consumed | `PnCounter` | Concurrent consumption from multiple nodes commutes |
| Resource units | `LastWriteWins` | Set by allocation authority |
| Analytics counters (views, likes) | `GrowOnlyCounter` | Impressions only grow; never decrement |
| Numeric metric values | `LastWriteWins` or `MaxRegister` | Domain-specific |
| JSON blobs (payload, config) | `LastWriteWins` | Whole-value replacement; no partial merge |
| Computed/AI attributes | System-write only | Written exclusively by compute engine |

(*) In production, lifecycle status should use a `Lattice` CRDT that encodes the valid state transition graph as a partial order, ensuring only valid transitions are accepted during merge. This is the v2 implementation.

---

## 4. Hypergrid Core Architecture Review

Before mapping each domain system, we briefly review the Hypergrid substrate components that every domain system will use.

### 4.1 The Grid

The `Grid` is the root container. Each domain system deployment creates one `Grid` — this Grid owns all Hypercubes, the Hypergraph, the EventLog, the CrdtLog, and the VectorClock for that deployment node.

```
Grid
├── grid_id: UUID                      ← globally unique
├── cubes: HashMap<CubeId, Hypercube>  ← all registered Hypercubes
├── ucs: UniversalCellStore            ← all HyperCells across all cubes
├── hypergraph: Hypergraph             ← typed edges between any entities
├── event_log: EventLog                ← append-only, immutable
├── crdt_log: CrdtLog                  ← in-flight operation buffer
├── vector_clock: VectorClock          ← federation causal ordering
└── node_id: NodeId                    ← this node's federation identity
```

Multiple Grid instances on different nodes form a federation for the same domain system. The CrdtLog + VectorClock ensure causal consistency across nodes through CRDT delta sync.

### 4.2 The Hypercube

A Hypercube is the analog of a spreadsheet sheet. For domain systems using the N=2 convention, the Hypercube is a conventional 2D grid where rows are entities and columns are fields. The Hypercube owns its AttributeKeyRegistry — all field definitions must be registered before writing.

### 4.3 The Universal Cell Store

The `UniversalCellStore` holds all HyperCells across all Hypercubes in the Grid. It is a `HashMap<(CubeId, DimCoordinate), HyperCell>`. Lookup is O(1) by exact coordinate. Scan operations return all cells for a given D₁ key (all fields of one entity) in O(k) where k is the number of fields.

### 4.4 The Hypergraph

The `Hypergraph` is a typed, directed property graph over any entities in any Hypercube. Edge types include: `Hierarchy`, `Dependency`, `Association`, `Contains`, `CrossGridLink`, `ShadowOf`, `SpaceMembership`, `Collaborates`, `FederationPeer`, and `Custom`. Domain systems add edges to the Hypergraph whenever they create relationships between entities. The Hypergraph provides BFS/DFS traversal, cycle detection, and the `reachable_from()` query.

### 4.5 The EventLog

Every mutation to every HyperCell produces an `EventEntry` in the `EventLog`. The EventLog is append-only and never modified. It provides:
- Complete audit trail of every change to every entity
- AS_OF time-travel: replaying the log to any timestamp
- Event-sourced recovery: rebuild any entity's state from its EventLog slice

Domain systems emit domain-typed events that dual-write to the Hypergrid EventLog as `EventKind::Custom(tag)`. This creates a single unified, time-travelable audit trail for both domain-level and cell-level mutations.

### 4.6 The VectorClock and CrdtLog

Every `ComponentMetadata` (or equivalent) carries a `VectorClock` — a `HashMap<NodeId, u64>` that records the logical time at each federation node. The `CrdtLog` buffers operations for delta sync. Together they enable:
- Causal ordering of all mutations across nodes
- Last-Write-Wins conflict resolution per-attribute
- OR-Set merge for set-valued attributes
- PN-Counter merge for numeric accumulators

### 4.7 HyperQL

HyperQL is the N-dimensional query language. Domain systems can issue full structured queries against any Hypercube:

```sql
-- HyperQL: find all active projects in Q2 2026 in the EMEA region
SELECT D₁.entity_id, D₂."name", D₂."status", D₂."budget"
FROM kogi.portfolio.components
WHERE D₂."status" = 'Active'
  AND D₂."category" = '{"Item":"Project"}'
  AND D₃.quarter = '2026-Q2'       -- if using TimeAxis
  AND D₄.region = 'EMEA'           -- if using GeoAxis
ORDER BY D₂."name"
LIMIT 50;
```

Domain systems expose HyperQL pass-through endpoints for advanced analytics, AI engine queries, and dashboard data.

---

# Part II — Domain Systems

---

## 5. Kogi Platform on Hypergrid

### 5.1 Platform Summary

Kogi is an Independent Worker Operating System — the central domain engine is the **Portfolio System**, which is the universal abstraction layer for every entity belonging to an independent worker's portfolio. Everything in the Kogi ecosystem is a Portfolio Item. The Portfolio System therefore maps cleanly to Hypergrid because it already models data as a grid of components with typed properties.

### 5.2 Kogi's Hypercube Registry

Kogi registers the following Hypercubes in its Grid:

#### 5.2.1 `kogi.portfolio.components` — The Primary Hypercube

This is the N=2 primary Hypercube. Every Component (Item or Container) is one HyperRow. Every field of every Component is one HyperCell.

```
D₁ = EntityAxis (Uuid)       ← ComponentId
D₂ = PropertyAxis (String)   ← field name
```

**Attribute Key Definitions (D₂ column registry):**

| D₂ Key (field name) | TypedAttrValue | CRDT Semantics | PermissionTier to Write | Notes |
|---------------------|----------------|----------------|-------------------------|-------|
| `name` | Text | LWW | Editor | Component display name |
| `description` | Text | LWW | Editor | Free-form description |
| `status` | Json | LWW (*lattice in v2) | Editor | ComponentStatus enum |
| `state` | Json | LWW | Manager | ComponentState enum |
| `visibility` | Json | LWW | Owner | Visibility enum |
| `category` | Json | LWW | Owner | ComponentCategory (Item\|Container) |
| `payload` | Json | LWW | Editor | ItemPayload or ContainerPayload |
| `owners` | Json | OrSet | Owner | Vec<UserId> |
| `tags` | Json | OrSet | Contributor | HashSet<String> |
| `hashtags` | Json | OrSet | Contributor | HashSet<String> |
| `topics` | Json | OrSet | Contributor | HashSet<String> |
| `budget` | Number | LWW | Manager | Allocated budget |
| `budget_spent` | Number | PnCounter | Manager | Consumed budget |
| `resource_units` | Number | LWW | Manager | Generic resource counter |
| `version` | Text | LWW | Editor | Semver string |
| `version_history` | Json | LWW | System | Vec<VersionHistoryEntry> |
| `policy_ids` | Json | LWW | Admin | Vec<PolicyId> |
| `children` | Json | OrSet | Manager | Vec<ComponentId> (DAG) |
| `parents` | Json | OrSet | Manager | Vec<ComponentId> (DAG) |
| `dependencies` | Json | OrSet | Editor | Vec<ComponentId> |
| `dependents` | Json | OrSet | System | Vec<ComponentId> |
| `links` | Json | OrSet | Editor | Vec<ComponentId> |
| `toolbox_ids` | Json | OrSet | Owner | Vec<ToolBoxId> (TMS v2.1) |
| `risks` | Json | LWW | Editor | Vec<Risk> |
| `users` | Json | LWW | Manager | ComponentUsers registry |
| `analytics` | Json | LWW | System | ComponentAnalytics counters |
| `item_book` | Json | LWW | Editor | Option<ItemBookData> |
| `created_at` | Json | LWW | System | DateTime<Utc> |
| `updated_at` | Json | LWW | System | DateTime<Utc> |
| `last_actor` | Text | LWW | System | NodeId |
| `vector_clock` | Json | LWW | System | VectorClock map |

#### 5.2.2 `kogi.portfolio.kpis` — N=3 KPI Time-Series Hypercube

For portfolio KPIs and metrics that need time-series tracking, Kogi uses a 3-dimensional Hypercube:

```
D₁ = EntityAxis (Uuid)           ← ComponentId or PortfolioId
D₂ = PropertyAxis (String)       ← metric name ("health_score", "revenue", "velocity")
D₃ = TimeAxis (Timestamp)        ← time period key
```

This allows queries like "what was the health score of Portfolio X for every quarter of 2025?" using a simple D₃ range slice. The AI engine writes `health_score`, `risk_score`, and `alignment_score` as Tier-2 computed attributes in this cube.

```
cell[(portfolio_id, "health_score", "2026-Q1")].value = Number(87.3)
cell[(portfolio_id, "health_score", "2026-Q2")].value = Number(91.1)
cell[(project_id, "velocity",       "2026-S03")].value = Number(42.0)
cell[(project_id, "burndown_rate",  "2026-S03")].value = Number(0.93)
```

HyperQL query for KPI trend:
```sql
SELECT D₁.entity_id, D₂."metric_name", D₃.quarter, SLICE(value)
FROM kogi.portfolio.kpis
WHERE D₁.entity_id = :portfolio_id
  AND D₃.quarter BETWEEN '2025-Q1' AND '2026-Q4'
ORDER BY D₃.quarter;
```

#### 5.2.3 `kogi.portfolio.benefits` — N=3 Benefits Hypercube

Portable benefits as portfolio items. The extra dimension is `benefit_type`:

```
D₁ = EntityAxis (Uuid)           ← UserId (worker)
D₂ = PropertyAxis (String)       ← benefit field name
D₃ = CategoryAxis (String)       ← BenefitType ("Health", "Retirement", "HSA")
```

This allows a single query to retrieve all benefit accounts for a worker across all benefit types, or to slice by a specific benefit type.

#### 5.2.4 `kogi.portfolio.contribution_ledger` — N=3 Collaboration Ledger

For shared portfolios and crowdresourcing, contribution attribution is modeled as a 3D Hypercube:

```
D₁ = EntityAxis (Uuid)           ← ComponentId (the shared portfolio component)
D₂ = PropertyAxis (String)       ← contribution field ("amount_cents", "weight", "type", "status")
D₃ = EntityAxis (Uuid)           ← ContributorId (user, org, or collective)
```

This models the full contribution matrix: for any given component, and any given contributor, what is their contribution amount, attribution weight, and governance status? All contribution records across all contributors are naturally organized as a 3D slice.

### 5.3 Kogi's Hypergraph Topology

The Kogi domain uses the following edge type configuration in the Hypergraph:

| Kogi Relationship | HypergraphEdge EdgeType | Source | Target | Notes |
|-------------------|------------------------|--------|--------|-------|
| Portfolio → Program | `Hierarchy` | Portfolio Component | Program Component | Compositional containment |
| Program → Project | `Hierarchy` | Program Component | Project Component | Compositional containment |
| Project → Resource | `Hierarchy` | Project Component | Resource Component | Resource assignment |
| Component depends on Component | `Dependency` | Any Component | Any Component | Cycle detection enforced |
| Binder → Item | `Contains` | Binder Container | Any Item | Container membership |
| Component link to Component | `Association` | Any Component | Any Component | Soft cross-reference |
| Worker to Portfolio (ownership) | `SpaceMembership` | IdentityId | Portfolio Component | Worker ↔ portfolio |
| Cross-worker shared portfolio | `Collaborates` | IdentityId | Portfolio Component | Multi-owner portfolios |
| ResourceShare to Organization | `ResourceShare` | Resource Component | EntityId | TMS resource sharing |
| Cross-portfolio federation | `CrossGridLink` | Row/Cube | Row/Cube | Multi-node federation |
| Benefit contribution source | `Association` | BenefitAccount | ContributionSource | Linked funding source |

The `PortfolioGraph` wrapper in `kogi-portfolio` translates all of these onto the Hypergraph, giving Kogi the full traversal capability — BFS reachability, DFS cycle detection, rollup aggregation, and cross-grid link following.

### 5.4 Kogi's EventLog Integration

Every mutation to a Kogi Component dual-writes to both the portfolio domain event log (for domain-typed querying) and the Hypergrid EventLog (for immutable audit and AS_OF time-travel).

Domain event kinds like `ComponentCreated`, `ComponentStatusChanged`, and `EdgeAdded` are emitted as `EventKind::Custom("portfolio:ComponentCreated")` into the Hypergrid EventLog. This means the Hypergrid EventLog is a complete, time-ordered, unified record of every component mutation across the entire Kogi deployment.

For AS_OF time-travel: `event_log.as_of(timestamp)` returns all EventEntries before that timestamp. By replaying all `Custom("portfolio:*")` entries, a Kogi system can reconstruct the exact state of any portfolio component at any historical moment.

### 5.5 Kogi's AI Integration Layer

Kogi's AI engine (KOGI-ENGINE) integrates with Hypergrid's Tier-2 computed attribute system:

| AI-Computed Attribute | Hypercube | CRDT | AI Engine Plugin | Update Frequency |
|-----------------------|-----------|------|-----------------|------------------|
| `health_score` | `kogi.portfolio.kpis` | LWW (system-write) | `KogiHealthScoreEngine` | On component mutation |
| `risk_score` | `kogi.portfolio.kpis` | LWW (system-write) | `KogiRiskEngine` | On risk record change |
| `alignment_score` | `kogi.portfolio.kpis` | LWW (system-write) | `KogiAlignmentEngine` | On program mutation |
| `collaboration_score` | `kogi.portfolio.kpis` | LWW (system-write) | `KogiCollaborationEngine` | On contribution event |
| `maturity_score` | `kogi.portfolio.components` | LWW (system-write) | `KogiMaturityEngine` | On artifact change |
| `anomaly_flag` | `kogi.portfolio.components` | LWW (system-write) | `KogiAnomalyEngine` | On metric deviation |

The AI engines are registered as `HypercubePlugin` implementations with `AttrComputation::AiEngine` on their output attribute keys. The Hypergrid attribution model ensures every AI-written value carries:
- `computed_by`: "kogi.health_score_engine.v2"
- `computed_at`: timestamp
- `confidence`: 0.0–1.0

This allows the Kogi UI and AI agent (Oba) to surface confidence intervals alongside AI scores and to decay or invalidate stale scores after a configurable TTL.

### 5.6 Kogi's Space and Namespace Configuration

Kogi uses the Hypergrid Space system to model different types of worker contexts:

| Kogi Context | HG-SPACE SpaceType | Slug Pattern | Auto-Provisioned Cubes |
|--------------|-------------------|--------------|------------------------|
| Personal Portfolio | `Personal` | `kogi://worker/{user_id}/personal/` | `components`, `kpis`, `benefits` |
| Shared Portfolio | `Team` | `kogi://portfolio/{portfolio_slug}/` | `components`, `contribution_ledger`, `kpis` |
| Cooperative Portfolio | `Community` | `kogi://cooperative/{coop_slug}/` | `components`, `governance`, `kpis` |
| Federation Portfolio | `Federation` | `kogi://federation/{fed_slug}/` | `components`, `cross-portfolio` |
| Crowdresourcing Campaign | `Project` | `kogi://campaign/{campaign_slug}/` | `components`, `contribution_ledger` |

Each Space has its own governance configuration (GovernanceConfig), its own member roster with permission tiers, and its own treasury reference for economic Spaces. The Hypergrid NamespacePath system ensures every entity in every Space has a globally unique, human-readable address.

---

## 6. Ume Platform on Hypergrid

### 6.1 Platform Summary

Ume is a software-defined Organization / Business Operating System built in Rust. It models a complete organization as a kernel-managed collection of 42 domain subsystems. The kernel supervises all modules, manages the event bus, enforces RBAC, and provides shared services.

When Ume adopts Hypergrid as its substrate, the mapping is conceptually powerful: each of the 42 organization modules becomes a domain-specific Hypercube in the Ume Grid. The kernel's module registry becomes a HyperRow in a `ume.kernel.modules` Hypercube. The event bus maps to Hypergrid's EventLog and CrdtLog. RBAC enforcement maps to Hypergrid's `PermissionTier` and `AttrVisibility`. The Supervisor maps to Hypergrid's `GovernanceEngine`.

### 6.2 Ume's Kernel Module Hypercube

The most important Hypercube in Ume is the kernel module registry:

#### `ume.kernel.modules` — N=2 Module Registry

```
D₁ = EntityAxis (Uuid)       ← ModuleId
D₂ = PropertyAxis (String)   ← module field name
```

**Attribute Key Registry:**

| D₂ Key | TypedAttrValue | CRDT | Notes |
|--------|----------------|------|-------|
| `module_id` | Text | LWW | Canonical module identifier |
| `name` | Text | LWW | Display name ("Finance & Accounting") |
| `domain_area` | Json | LWW | DomainArea enum |
| `version` | Text | MaxRegister | Module version string |
| `lifecycle_state` | Json | LWW | LifecycleState: Registered → Starting → Running → Degraded → Stopped |
| `status` | Json | LWW | ModuleStatus: Active \| Suspended \| Disabled |
| `dependencies` | Json | OrSet | Vec<ModuleId> that this module depends on |
| `permissions_required` | Json | LWW | Vec<Permission> declared by the module |
| `executor_id` | Text | LWW | Assigned ExecutorPool thread pool ID |
| `restart_count` | Number | GrowOnlyCounter | Supervisor restart counter |
| `last_error` | Json | LWW | Option<KernelError> from most recent failure |
| `health_score` | Number | LWW (AI-write) | AI-computed module health 0–100 |
| `evaluation_cache` | Json | LWW | Cached evaluation output from last evaluation run |
| `config` | Json | LWW | Module configuration blob |
| `metrics` | Json | LWW | MetricPoint observations snapshot |
| `created_at` | Json | LWW | DateTime<Utc> |
| `updated_at` | Json | LWW | DateTime<Utc> |

This registry gives Ume a fully versioned, CRDT-synchronized module registry that survives federation node failures and network partitions. A module's `lifecycle_state` is an OR-Set-safe field with LWW semantics — if two nodes disagree about whether the Finance module is Running or Degraded, the most recent timestamp wins.

### 6.3 The 42 Organization Module Hypercubes

Each of the 42 Ume organization modules maps to one or more domain-specific Hypercubes in the Ume Grid. The following table gives the complete Hypercube inventory for Ume:

| Module # | Module Name | Primary Hypercube | Key D₂ Fields | Extra Dimensions |
|----------|-------------|-------------------|---------------|------------------|
| 01 | Organization Administration | `ume.admin.org_units` | name, type, parent_unit, head_count, budget | D₃=TimeAxis (historical org charts) |
| 02 | Organization Analytics | `ume.analytics.kpis` | metric_name, value, target, trend | D₃=TimeAxis, D₄=DomainAxis |
| 03 | Backup & Recovery | `ume.backup.jobs` | job_id, scope, status, schedule, last_run | D₃=CategoryAxis (backup_type) |
| 04 | Board Management | `ume.board.meetings` | title, date, quorum, resolution_count, status | D₃=TimeAxis |
| 05 | Business Development | `ume.bizdev.opportunities` | title, stage, value, probability, close_date | D₃=CategoryAxis (opportunity_type) |
| 06 | Enterprise CMS | `ume.cms.content` | title, content_type, status, owner, tags | D₃=TimeAxis (version history) |
| 07 | Communications | `ume.comms.channels` | channel_name, type, members, status | D₃=CategoryAxis (channel_type) |
| 08 | CRM | `ume.crm.contacts` | name, type, stage, owner, last_contact | D₃=CategoryAxis (contact_type) |
| 09 | Design System | `ume.design.tokens` | token_name, value, category, version | D₃=TimeAxis |
| 10 | Engineering & Technology | `ume.eng.projects` | name, stack, status, team, velocity | D₃=TimeAxis (sprint data) |
| 11 | ESG / CSR / Sustainability | `ume.esg.metrics` | metric_name, value, category, period | D₃=TimeAxis, D₄=CategoryAxis |
| 12 | Enterprise Engineering Admin | `ume.enterprise.systems` | system_name, type, status, owner | — |
| 13 | Legal Entity (Chombo) | `ume.chombo.entities` | entity_name, jurisdiction, type, status | D₃=CategoryAxis (jurisdiction) |
| 14 | Finance & Accounting | `ume.finance.accounts` | account_name, type, balance, currency | D₃=TimeAxis (period balances) |
| 15 | GRC | `ume.grc.risks` | title, category, severity, status, owner | D₃=TimeAxis (risk history) |
| 16 | Human Resources | `ume.hr.employees` | name, role, department, status, hire_date | D₃=TimeAxis (headcount history) |
| 17 | Investment Management | `ume.investment.portfolio` | asset_name, type, value, allocation_pct | D₃=TimeAxis, D₄=CategoryAxis |
| 18 | IT & Asset Management | `ume.it.assets` | asset_name, type, status, owner, cost | D₃=CategoryAxis (asset_type) |
| 19 | Enterprise Knowledge | `ume.knowledge.articles` | title, category, status, author, views | D₃=TimeAxis |
| 20 | Learning & Development | `ume.learning.programs` | title, type, status, enrolled, completed | D₃=TimeAxis |
| 21 | Management & Strategy | `ume.strategy.okrs` | objective, key_result, progress, owner | D₃=TimeAxis (OKR periods) |
| 22 | Marketing (Soko) | `ume.soko.campaigns` | name, type, status, budget, target_audience | D₃=TimeAxis, D₄=CategoryAxis |
| 23 | Master Data Management | `ume.mdm.entities` | entity_name, type, golden_record_id, source | D₃=CategoryAxis (entity_type) |
| 24 | Office & Facility | `ume.facilities.sites` | site_name, type, capacity, status, owner | D₃=CategoryAxis (site_type) |
| 25 | Operations Management | `ume.ops.processes` | name, type, status, sla_target, owner | D₃=TimeAxis |
| 26 | Portal / Hub / Dashboard | `ume.portal.dashboards` | title, owner, widgets, layout, visibility | — |
| 27 | Portfolio & Program | `ume.portfolio.programs` | name, status, budget, kpi_count, health | D₃=TimeAxis |
| 28 | PR & Branding | `ume.branding.assets` | name, type, status, campaign_id, version | D₃=TimeAxis |
| 29 | Process, Orchestration & Workflow | `ume.process.workflows` | name, type, status, trigger, step_count | D₃=TimeAxis |
| 30 | Product, Services & Solutions | `ume.product.catalog` | name, type, status, sku, pricing_model | D₃=TimeAxis, D₄=CategoryAxis |
| 31 | Production, Manufacturing | `ume.production.batches` | batch_id, product, status, quantity, date | D₃=TimeAxis |
| 32 | Requirements Management | `ume.requirements.specs` | title, type, priority, status, owner | D₃=CategoryAxis (req_type) |
| 33 | Enterprise Risk Management | `ume.risk.register` | title, category, severity, probability | D₃=TimeAxis |
| 34 | Sales Management | `ume.sales.opportunities` | title, stage, value, owner, close_date | D₃=TimeAxis (pipeline history) |
| 35 | Enterprise Schedule | `ume.schedule.events` | title, type, start_at, end_at, attendees | D₃=TimeAxis |
| 36 | Security, Privacy & Protection | `ume.security.incidents` | title, severity, status, assignee, cve_id | D₃=TimeAxis |
| 37 | Logistics, Supply Chain & WMS | `ume.supply_chain.orders` | order_id, supplier, status, total, eta | D₃=TimeAxis |
| 38 | Team & Cooperative Management | `ume.teams.groups` | name, type, members, owner, mission | — |
| 39 | Organization Templating | `ume.templates.library` | name, domain, version, status, uses_count | D₃=TimeAxis |
| 40 | Enterprise Work Management | `ume.work.items` | title, type, priority, status, assignee | D₃=TimeAxis (sprint tracking) |
| 41 | Custom UME Modules | `ume.custom.{vendor}.*` | Platform-defined | Vendor-defined |
| 42 | Custom Org Modules | `ume.org.{slug}.*` | Organization-defined | Org-defined |

### 6.4 Ume's Cross-Module Graph Topology

The Ume Hypergraph models every inter-module relationship described in the Ume SDD's integration map. This replaces the indirect module-to-module call pattern with direct typed edges in the Hypergraph:

| Ume Relationship | HypergraphEdge EdgeType | Example |
|-----------------|------------------------|---------|
| Module depends on Module | `Dependency` | Finance depends on Legal Entity |
| Strategy OKR cascades to Module | `Hierarchy` | OKR → Finance → Sales |
| Employee belongs to Department | `Hierarchy` | Employee → OrgUnit |
| Risk linked to Control | `Association` | GRC Risk → GRC Control |
| Campaign uses Asset | `Contains` | Marketing Campaign → Design Asset |
| Invoice linked to PO | `Association` | Finance Invoice → Supply Chain PO |
| Employee enrolled in Program | `Association` | HR Employee → L&D Program |
| Workflow triggers Module | `Association` | Process Workflow → Any Module |
| Module feeds Analytics | `Association` | Any Module → Analytics KPI |
| Cross-org federated share | `FederationPeer` | Ume Instance A → Ume Instance B |
| Cross-org data mirror | `CrossGridLink` | Partner ume.crm.contacts → shadow |

The Hypergraph's ability to traverse these relationships with a single BFS query replaces what in Ume's original architecture required kernel facade calls between modules. A query like "what is the full dependency chain of the Finance module?" becomes a `reachable_from(finance_module_id)` call on the Hypergraph.

### 6.5 Ume's Kernel-to-Hypergrid Mapping

Ume's kernel services map directly onto Hypergrid infrastructure:

| Ume Kernel Service | Hypergrid Equivalent | Notes |
|--------------------|---------------------|-------|
| `ModuleRegistry` | `ume.kernel.modules` Hypercube | Module registrations as HyperRows |
| `InMemoryEventBus` | Hypergrid EventLog + CrdtLog | Persistent, time-travelable event stream |
| `RbacEngine` | Hypergrid PermissionTier + AttrVisibility | Per-attribute, per-module RBAC |
| `SupervisorEngine` | Hypergrid GovernanceEngine + PolicyEngine | Module restart/backoff policy |
| `OrchestratorEngine` | Hypergrid HyperQL + computed attributes | Workflow query and execution |
| `TemplateEngine` | Hypergrid CMS Hypercube (`ume.templates.library`) | Versioned template storage |
| `SchemaRegistry` | Hypergrid AttributeKeyRegistry | Per-Hypercube schema contracts |
| `MetricsCollector` | `ume.analytics.kpis` Hypercube (D₃=TimeAxis) | Time-series metric storage |
| `LogAuditManager` | Hypergrid EventLog | Immutable, append-only audit trail |
| `StorageManager` | Hypergrid UniversalCellStore | All entity storage |
| `MemoryManager` | Hypergrid hot-path cell cache | In-process read cache |
| `SearchService` | Hypergrid HyperQL + full-text attribute index | Cross-cube entity search |
| `BackupManager` | Hypergrid Snapshot + EventLog replay | Point-in-time state capture |

### 6.6 Ume's Organization CPU Model on Hypergrid

Ume's `ExecutorPool` concept — named thread pools dedicated to domain execution priorities — maps onto Hypergrid's `Space` system. Each `ExecutorId` corresponds to a Hypergrid `Space` of type `Team`:

| Ume ExecutorId | Hypergrid Space | SpaceType | Cubes Managed |
|----------------|-----------------|-----------|---------------|
| `kernel.critical` | `ume://system/kernel/critical/` | `Tenant` | `ume.kernel.modules` |
| `kernel.events` | `ume://system/kernel/events/` | `Tenant` | EventLog streams |
| `kernel.audit` | `ume://system/kernel/audit/` | `Tenant` | Immutable audit records |
| `org.finance` | `ume://org/finance/` | `Team` | `ume.finance.*` |
| `org.legal` | `ume://org/legal/` | `Team` | `ume.chombo.*` |
| `org.marketing` | `ume://org/marketing/` | `Team` | `ume.soko.*` |
| `org.hr` | `ume://org/hr/` | `Team` | `ume.hr.*` |
| `org.analytics` | `ume://org/analytics/` | `Team` | `ume.analytics.*` |
| `org.ops` | `ume://org/ops/` | `Team` | `ume.ops.*`, `ume.supply_chain.*` |

Each Space has its own `GovernanceConfig`, member roster, and Space-scoped sub-graph in the Hypergraph. Domain module-to-module relationships only cross Space boundaries through the Hypergraph — never through direct in-memory calls.

### 6.7 Chombo (Legal Entity) on Hypergrid

Chombo is Ume's legal entity management subsystem. It generates 45 policy packs covering different legal jurisdictions and compliance frameworks. On Hypergrid, Chombo uses a 3-dimensional Hypercube:

```
ume.chombo.entities (N=3)
D₁ = EntityAxis (Uuid)           ← LegalEntityId
D₂ = PropertyAxis (String)       ← entity field name
D₃ = CategoryAxis (String)       ← jurisdiction ("US-DE", "UK", "EU", "ZA", ...)
```

This means a single multinational legal entity can have D₃-sliced data for each jurisdiction it operates in. A UK tax compliance record and a Delaware incorporation record for the same entity live in the same Hypercube at different D₃ keys. Cross-jurisdictional analysis queries become DimSlice operations with D₃ filtering.

Policy packs are stored as HyperRows in `ume.chombo.policies` with their evaluation results as computed attributes:

```
cell[(policy_pack_id, "compliance_score", "US-DE")].value = Number(94.2)
cell[(policy_pack_id, "open_findings",    "UK")].value = Number(3)
cell[(policy_pack_id, "last_evaluated",   "EU")].value = Json("2026-03-15T14:30:00Z")
```

### 6.8 Soko (Marketing) on Hypergrid

Soko, Ume's marketing system, manages campaigns, signals, and 70 strategy packs. On Hypergrid, Soko's campaign data uses a 4-dimensional Hypercube for multi-segment analytics:

```
ume.soko.campaigns (N=4)
D₁ = EntityAxis (Uuid)           ← CampaignId
D₂ = PropertyAxis (String)       ← metric/field name
D₃ = TimeAxis (Timestamp)        ← time period
D₄ = CategoryAxis (String)       ← audience_segment
```

This allows queries like "what was the CTR for Campaign X in segment 'enterprise' in Q2 2026?" as a single point lookup: `cell[(campaign_id, "ctr", "2026-Q2", "enterprise")]`.

Signal processing and strategy pack evaluations are written as Tier-2 AI computed attributes via `AttrComputation::AiEngine` bindings to the Soko Strategy Evaluation Engine.

### 6.9 Ume's RBAC on Hypergrid

Ume's RBAC permission namespace (`{domain}.{resource}.{action}`) maps directly onto Hypergrid's per-attribute PermissionTier. Every attribute key definition in every Ume Hypercube carries a `write_permission: PermissionTier` that enforces the minimum tier required to write that field.

For example, the Ume permission `finance.journal.write` maps to `write_permission: PermissionTier::Editor` on all D₂ keys in `ume.finance.accounts` that correspond to journal entry fields. The Ume permission `finance.invoice.approve` maps to `write_permission: PermissionTier::Manager` on the invoice approval status field.

Hypergrid's `AttrVisibility` (`Public | Tenant | Identity | Owner`) maps onto Ume's role visibility: Public fields are readable by all, Tenant fields are readable within the same organizational tenant, Identity fields are readable only by the owning identity, Owner fields are readable only by the owning user.

---

## 7. Qala Platform on Hypergrid

### 7.1 Platform Summary

Qala is a Universal Solution Factory Operating System. Its core abstraction is the **Solution Development Environment (SDE)** — the atomic operational unit within which solutions of every type (Application, System, Good, Product, Service, Platform) are created, managed, and governed. Qala is self-describing: the platform governs itself through itself.

On Hypergrid, Qala's entities map naturally to the N-dimensional model because solutions have exactly the right properties: they have identity, purpose, versions, changes, states, artifacts, documentation, and lifecycle — all of which are typed, time-indexed properties on a HyperRow.

### 7.2 Qala's Primary Hypercubes

Qala registers the following primary Hypercubes in its Grid:

#### 7.2.1 `qala.factories` — Solution Factory Registry

```
D₁ = EntityAxis (Uuid)             ← FactoryId
D₂ = PropertyAxis (String)         ← factory field name
```

**Key D₂ attribute keys:**

| D₂ Key | Type | CRDT | Notes |
|--------|------|------|-------|
| `name` | Text | LWW | Factory display name |
| `factory_type` | Json | LWW | Personal \| Team \| Org \| Enterprise \| Platform |
| `tier` | Json | LWW | Tier enum |
| `parent_factory_id` | Json | LWW | Parent in factory hierarchy |
| `child_factory_ids` | Json | OrSet | Child factory IDs |
| `sde_ids` | Json | OrSet | SDEs registered under this factory |
| `owner_id` | Json | LWW | Owner user or org |
| `member_ids` | Json | OrSet | Factory members |
| `template_ids` | Json | OrSet | Factory-level SDE templates |
| `registry` | Json | LWW | Solution registry summary |
| `governance_policy` | Json | LWW | Change control, release, versioning policies |
| `quota_config` | Json | LWW | Resource quotas |
| `status` | Json | LWW | Draft \| Active \| Suspended \| Dissolved |
| `created_at` | Json | LWW | Creation timestamp |
| `domain_packs` | Json | OrSet | Installed domain pack IDs |

#### 7.2.2 `qala.sdes` — Solution Development Environment Registry

The SDE is the most central entity in Qala — the atomic unit of solution development. Its Hypercube:

```
D₁ = EntityAxis (Uuid)             ← SdeId
D₂ = PropertyAxis (String)         ← SDE field name
```

**Key D₂ attribute keys:**

| D₂ Key | Type | CRDT | Notes |
|--------|------|------|-------|
| `name` | Text | LWW | SDE display name |
| `factory_id` | Json | LWW | Parent Solution Factory |
| `owner_id` | Json | LWW | SDE owner |
| `template_id` | Json | LWW | Template used to provision |
| `status` | Json | LWW | Provisioning → Active → Suspended → Decommissioned |
| `deployment_target` | Json | LWW | Platform / Cloud / Machine / Edge / Air-gapped |
| `environment_manifest` | Json | LWW | .qala.yaml — full environment spec |
| `toolchain_ids` | Json | OrSet | Registered toolchain IDs |
| `solution_ids` | Json | OrSet | Solutions in this SDE |
| `language_runtimes` | Json | LWW | Supported language runtime configs |
| `config_files` | Json | LWW | Layered config (Factory > SDE > Solution) |
| `env_variables` | Json | LWW | Environment variable set (encrypted ref) |
| `hermetic_manifest` | Json | LWW | Digest-pinned dependency manifest |
| `snapshot_ids` | Json | OrSet | Historical SDE snapshots |
| `drift_status` | Json | LWW (AI-write) | DriftDetectionResult from continuous monitoring |
| `ai_recommendation_ids` | Json | OrSet | AI Agent recommendation IDs for this SDE |
| `last_build_at` | Json | LWW | Most recent build timestamp |
| `last_snapshot_at` | Json | LWW | Most recent snapshot timestamp |
| `provisioning_time_ms` | Number | LWW | Time from request to Active state |
| `version` | Text | MaxRegister | SDE configuration version |
| `created_at` | Json | LWW | Provisioning timestamp |

This Hypercube makes every SDE fully auditable, time-travelable, and CRDT-safe for federated Qala deployments. A drift detection event updates `drift_status` as an AI-computed attribute, giving Qala its 15-minute drift detection SLA backed by Hypergrid's push-based attribute computation.

#### 7.2.3 `qala.solutions` — Solution Registry

Solutions are instances of Solution Models. They are the primary outputs of SDEs:

```
D₁ = EntityAxis (Uuid)             ← SolutionId
D₂ = PropertyAxis (String)         ← solution field name
D₃ = TimeAxis (Timestamp)          ← version history dimension
```

Adding D₃=TimeAxis gives Qala's solution registry a built-in temporal dimension. Querying all versions of a solution becomes a D₃ range slice. The current version is the D₃=latest slice.

**Key D₂ attribute keys:**

| D₂ Key | Type | CRDT | Notes |
|--------|------|------|-------|
| `name` | Text | LWW | Solution name |
| `solution_type` | Json | LWW | Application \| System \| Good \| Product \| Service \| Platform |
| `sde_id` | Json | LWW | Parent SDE |
| `model_id` | Json | LWW | Solution Model this instantiates |
| `lifecycle_state` | Json | LWW (*lattice v2) | Draft → InReview → Approved → Active → Deprecated → Retired |
| `version` | Text | MaxRegister | Semver solution version |
| `structure` | Json | LWW | Seven-level decomposition (System→App→Process→…→Data) |
| `playbook_id` | Json | LWW | Associated Playbook document |
| `solution_book_id` | Json | LWW | Associated Solution Book |
| `artifact_ids` | Json | OrSet | Build artifacts |
| `ccr_ids` | Json | OrSet | Change Control Request IDs |
| `release_ids` | Json | OrSet | Release record IDs |
| `risk_score` | Number | LWW (AI-write) | AI-computed risk score |
| `quality_score` | Number | LWW (AI-write) | AI-computed quality score |
| `security_score` | Number | LWW (AI-write) | AI-computed security score |
| `compliance_status` | Json | LWW (AI-write) | Domain-pack compliance check result |
| `domain_pack_data` | Json | LWW | Domain Pack extension fields |
| `created_at` | Json | LWW | Solution creation timestamp |
| `owner_id` | Json | LWW | Primary owner |
| `contributing_owners` | Json | OrSet | Contributing owner IDs |

#### 7.2.4 `qala.ccrs` — Change Control Request Registry

CCRs are the governance events of Qala. They gate all changes to all solutions. On Hypergrid:

```
D₁ = EntityAxis (Uuid)             ← CcrId
D₂ = PropertyAxis (String)         ← CCR field name
D₃ = CategoryAxis (String)         ← risk_tier ("Low", "Medium", "High", "Critical")
```

Adding D₃=CategoryAxis (risk tier) gives Qala the ability to slice the CCR registry by risk tier and get aggregated counts or governance metrics per tier without any custom aggregation code — it's a built-in DimFold:

```sql
-- HyperQL: CCR volume by risk tier in Q2 2026
SELECT D₃.risk_tier, COUNT(D₁.ccr_id) AS ccr_count,
       AVG(cell[D₁.ccr_id, "resolution_time_hours"].value) AS avg_resolution_hours
FROM qala.ccrs
WHERE cell[D₁.ccr_id, "created_at"].value BETWEEN '2026-04-01' AND '2026-06-30'
GROUP BY D₃.risk_tier;
```

**Key D₂ attribute keys:**

| D₂ Key | Type | CRDT | Notes |
|--------|------|------|-------|
| `title` | Text | LWW | CCR title |
| `description` | Text | LWW | Change description |
| `solution_id` | Json | LWW | Affected solution |
| `sde_id` | Json | LWW | Originating SDE |
| `requestor_id` | Json | LWW | Requesting actor |
| `risk_tier` | Json | LWW | Low \| Medium \| High \| Critical |
| `status` | Json | LWW (*lattice v2) | Draft → Submitted → UnderReview → Approved → Rejected → Implemented → Closed |
| `approver_ids` | Json | OrSet | Required approver IDs |
| `approvals` | Json | LWW | Approval decision records |
| `impact_assessment` | Json | LWW | Structured impact analysis |
| `compliance_evidence` | Json | LWW | Evidence package for audit |
| `implementation_plan` | Json | LWW | Step-by-step implementation |
| `rollback_plan` | Json | LWW | Rollback procedure |
| `ai_risk_assessment` | Json | LWW (AI-write) | AI Agent risk scoring and recommendations |
| `resolution_time_hours` | Number | LWW | Time from submission to decision |
| `created_at` | Json | LWW | CCR creation timestamp |

#### 7.2.5 `qala.sde_snapshots` — SDE Snapshot and Time-Travel Hypercube

One of Qala's most important capabilities is SDE snapshotting — capturing the complete state of an SDE for rollback, clone, and audit purposes. On Hypergrid, this uses a 3-dimensional Hypercube:

```
D₁ = EntityAxis (Uuid)             ← SdeId
D₂ = PropertyAxis (String)         ← snapshot field name
D₃ = TimeAxis (Timestamp)          ← snapshot timestamp
```

This gives Qala AS_OF queries for free. The query "what was the state of SDE X at 2026-03-01T10:00:00Z?" is a point lookup at D₁=sde_id, D₃=timestamp, across all D₂ field names.

```
cell[(sde_id, "environment_manifest", "2026-03-01T10:00:00Z")].value = Json({...})
cell[(sde_id, "toolchain_ids",        "2026-03-01T10:00:00Z")].value = Json([...])
cell[(sde_id, "hermetic_manifest",    "2026-03-01T10:00:00Z")].value = Json({...})
cell[(sde_id, "version",              "2026-03-01T10:00:00Z")].value = Text("3.7.2")
```

Rollback becomes: read D₃=target_timestamp for all D₂ keys, write those values into the current D₃=now slice. The operation is atomic (single transaction across all D₂ keys), and the EventLog records the rollback as a first-class audit event.

#### 7.2.6 `qala.solutions.metrics` — N=4 Solution Quality Hypercube

Qala's quality management system uses a 4-dimensional Hypercube for comprehensive quality metrics:

```
D₁ = EntityAxis (Uuid)             ← SolutionId
D₂ = PropertyAxis (String)         ← metric name
D₃ = TimeAxis (Timestamp)          ← measurement period
D₄ = CategoryAxis (String)         ← metric_category ("security", "quality", "performance", "compliance")
```

This allows the AI Agent to write multi-dimensional quality scores per solution per period per category:

```
cell[(sol_id, "defect_density", "2026-Q2", "quality")].value       = Number(0.12)
cell[(sol_id, "vulnerability_count", "2026-Q2", "security")].value = Number(3)
cell[(sol_id, "p99_latency_ms", "2026-Q2", "performance")].value   = Number(87.4)
cell[(sol_id, "compliance_pct", "2026-Q2", "compliance")].value    = Number(99.2)
```

HyperQL can then produce any cross-dimensional quality report — for example, all solutions in the "qala.pharmaceuticals.formulations" factory with security score below 90 in the last two quarters:

```sql
SELECT D₁.solution_id, D₃.quarter, cell[D₁, D₂, D₃, "security"].value AS security_score
FROM qala.solutions.metrics
WHERE D₄.category = 'security'
  AND cell[D₁, D₂, D₃, "security"].value < 90
  AND D₃.quarter IN ('2026-Q1', '2026-Q2')
ORDER BY security_score ASC;
```

### 7.3 Qala's Six-Plane Architecture on Hypergrid

Qala describes its architecture as six logical planes. Each plane maps onto Hypergrid infrastructure:

| Qala Plane | Language (SDD) | Hypergrid Layer Mapping |
|------------|---------------|-------------------------|
| Kernel Plane | Rust | `hypergrid::core::Grid` — integrity, service registry, event aggregation, solution graph engine |
| Control Plane | Go | Domain-layer systems built on Hypergrid: SDE, Factory, Identity, CCR, Release services |
| Execution Plane | Go/Scala | HyperQL query engine + computed attribute execution; CI/CD pipeline state in `qala.cicd.*` cubes |
| Intelligence Plane | Rust | Hypergrid AI Engine + `AttrComputation::AiEngine` for all AI agents; SEM writes to `qala.security.*` cubes |
| Platform Operations | Go/Terraform | Hypergrid Snapshot + Federation + deployment topology management |
| Domain Extension Plane | Go + YAML DSL | HypercubePlugin trait — Domain Packs as plugins that register new axis types, attribute keys, and computed models |

The **Domain Extension Plane** is particularly elegant on Hypergrid. A Domain Pack for, say, Pharmaceutical Formulation Management is a `HypercubePlugin` implementation that:

1. Registers new axis types: `BatchAxis` (for batch numbers), `IngredientAxis` (for ingredient IDs), `JurisdictionAxis` (for regulatory regions)
2. Registers new attribute keys: `regulatory_status`, `batch_release_flag`, `ingredient_spec_id`, `pharmacopoeial_grade`
3. Registers AI engine bindings: `FormulationRiskEngine`, `RegulatoryComplianceChecker`
4. Registers new edge types: `Contains` (Formulation → Ingredient), `Derives` (Version A → Version B)

The Domain Pack plugin is installed at the factory level, enriches the `qala.solutions` Hypercube with its additional attribute keys, and is versioned independently of the core platform. This is the Hypergrid plugin system doing exactly what Domain Packs need.

### 7.4 Qala's Solution Lifecycle as a Hypergrid Lattice

Qala defines a universal solution lifecycle: Draft → InReview → Approved → Active → Deprecated → Retired. Each state transition is a governed event that must flow through a CCR.

On Hypergrid, the `lifecycle_state` attribute key uses a `Lattice` CRDT that encodes this state machine as a partial order:

```rust
// Qala solution lifecycle lattice
let lifecycle_lattice = LatticeOrder::new(vec![
    "Draft",
    "InReview",
    "Approved",
    "Active",
    "Deprecated",
    "Retired",
]);
```

The `Lattice` CRDT ensures that concurrent state-transition operations from multiple federation nodes never result in an invalid lifecycle state. If two nodes attempt to concurrently transition a solution — one to `Approved` and one back to `Draft` — the lattice merge always resolves to the higher state (`Approved`), preserving the governance invariant that lifecycle state never regresses without an explicit admin override.

This replaces the ad-hoc status management code that would otherwise need to be written in the Qala domain layer.

### 7.5 Qala's Root Factory Principle on Hypergrid

The most philosophically interesting aspect of Qala is its Root Factory Principle: Qala itself is a Solution Factory. The platform governs itself through itself. Every platform update goes through a CCR. Every release is a governed release event.

On Hypergrid, this becomes: the Qala Root Factory's own Grid contains HyperCells that describe the Qala platform's own solutions, SDEs, CCRs, and releases. The Qala platform is a first-class HyperRow in `qala.factories` with `factory_id = QALA_ROOT_FACTORY_ID`. Its child factory IDs include every customer-provisioned Solution Factory in the system.

This self-describing architecture means that Qala's own platform telemetry, governance records, and AI recommendations are stored in the same Hypercubes that govern all customer solutions — giving the Qala operations team access to the full power of HyperQL and the AI Agent for managing the platform itself.

### 7.6 Qala's AI Agent on Hypergrid

Qala's AI Agent provides ten distinct AI capabilities (from the SDD: SDE Optimization, Pipeline Bottleneck Detection, Predictive Defect Detection, Test Case Generation, Anomaly Detection, Resource Prediction, Content Suggestions, Security Threat Analysis, Backup Schedule Optimization, Prototype Feedback Analysis).

On Hypergrid, all ten are implemented as `HypercubePlugin` instances with `AttrComputation::AiEngine` bindings:

| Qala AI Capability | Hypergrid Plugin | Output Attribute | Target Hypercube |
|--------------------|-----------------|-----------------|-----------------|
| SDE Optimization | `SdeOptimizationEngine` | `optimization_recommendations` | `qala.sdes` |
| Pipeline Bottleneck Detection | `PipelineAnalysisEngine` | `bottleneck_report` | `qala.cicd.pipelines` |
| Predictive Defect Detection | `DefectPredictionEngine` | `defect_risk_score` | `qala.solutions` |
| Test Case Generation | `TestGenEngine` | `suggested_test_cases` | `qala.solutions` |
| Anomaly Detection | `AnomalyEngine` | `anomaly_flags` | `qala.solutions.metrics` |
| Resource Prediction | `ResourceForecastEngine` | `forecast_compute_units` | `qala.sdes` |
| Content Suggestions | `ContentSuggestionEngine` | `documentation_suggestions` | `qala.solutions` |
| Security Threat Analysis | `SecurityThreatEngine` | `threat_report` | `qala.security.events` |
| Backup Schedule Optimization | `BackupOptimizationEngine` | `optimal_backup_schedule` | `qala.sdes` |
| Prototype Feedback Analysis | `PrototypeFeedbackEngine` | `feedback_synthesis` | `qala.solutions` |

Each AI attribute is registered with `write_permission: PermissionTier::Admin` (system-write only) and `AttrVisibility::Tenant` (readable within the factory). The `confidence` attribute on AI cells gives the Qala UI the ability to surface AI confidence scores alongside recommendations.

The AI Agent's `ai_recommendation_ids` field on the SDE uses OR-Set CRDT semantics — recommendations from multiple AI engines can be added concurrently from different federation nodes without any recommendation being silently lost.

### 7.7 Qala's Hermetic Build System and SDE Drift Detection

One of Qala's most distinctive requirements is zero environment-induced CI failures — achieved through hermetic SDEs with digest-pinned dependencies. Drift detection must occur within 15 minutes of any drift event.

On Hypergrid, drift detection is a Tier-2 computed attribute. The `DriftDetectionEngine` is a `HypercubePlugin` that:
1. Subscribes to the `qala.sdes` Hypercube's EventLog for any mutation to `environment_manifest`, `toolchain_ids`, or `hermetic_manifest`
2. On each mutation, computes a hash of the current SDE state against the pinned hermetic manifest
3. Writes the result as `drift_status` in the SDE's HyperRow

The EventLog's dual-write to Hypergrid's own event infrastructure means drift events propagate within the event processing latency (sub-second in normal operation, well within the 15-minute SLA).

---

# Part III — Cross-Platform Infrastructure

---

## 8. Cross-Platform Shared Infrastructure

### 8.1 The Universal Cell Store as Shared Foundation

All three domain systems — Kogi, Ume, and Qala — share the same `UniversalCellStore` architecture. Their Grids are separate deployments (unless explicitly federated), but the codebase and operational model are identical. This means:

1. **Same CRDT engine** — VectorClock, OrSet, PnCounter, LwwRegister from `hypergrid::crdt`
2. **Same EventLog** — append-only, AS_OF time-travel from `hypergrid::core::EventLog`
3. **Same Hypergraph** — typed directed edges, BFS/DFS traversal from `hypergrid::graph`
4. **Same query language** — HyperQL from `hypergrid::query`
5. **Same AI plugin interface** — `HypercubePlugin` + `AttrComputation::AiEngine` from `hypergrid::plugin`
6. **Same namespace system** — NamespacePath URIs from `hypergrid::space`
7. **Same permission model** — PermissionTier + AttrVisibility from `hypergrid::cell`

A developer who understands how to write a domain system on Hypergrid for Kogi can apply exactly the same patterns to extend Ume or Qala. The learning investment in the substrate is made once.

### 8.2 Cross-System Federation

Hypergrid's federation mechanism allows any two domain system deployments to link their Grids via `CrossGridLink` edges in the Hypergraph. This is particularly powerful when Kogi, Ume, and Qala are deployed as different applications within the same ecosystem — for example, if a cooperative of independent workers (Kogi) is also an organization (Ume) that uses Qala to govern its solution development.

The federation link works as follows:

1. **Kogi Grid** ← CrossGridLink → **Ume Grid**: A Kogi Portfolio Component (a resource or artifact) is linked to a Ume OrgModule record. A `ShadowCell` is provisioned in the Kogi Grid reflecting the Ume record's key attributes.

2. **Ume Grid** ← CrossGridLink → **Qala Grid**: A Ume Work Management item is linked to a Qala Solution. The work item's status updates are mirrored into the Qala SDE's timeline.

3. **Qala Grid** ← CrossGridLink → **Kogi Grid**: A Qala Solution's artifact is linked to a Kogi Portfolio Artifact Component. The artifact's release status is mirrored as a shadow attribute in Kogi.

All federation links respect consent (ConsentStatus must be `Accepted` before any mirror attributes sync), attribute visibility (only `mirrored_attrs` specified at link creation time are shared), and write-back governance (only `writeback_attrs` can be written back through the link).

### 8.3 The Shared Namespace Architecture

All three systems use the same Hypergrid NamespacePath URI scheme with domain-specific prefixes:

```
kogi://                         ← Kogi namespace root
  {portfolio_slug}/             ← Space root
    components/{id}/            ← Component namespace
    kpis/{metric}/              ← KPI namespace

ume://                          ← Ume namespace root
  {org_id}/                     ← Organization Space root
    modules/{module_id}/        ← Module namespace
    hr/{employee_id}/           ← HR entity namespace

qala://                         ← Qala namespace root
  {factory_slug}/               ← Solution Factory root
    sdes/{sde_id}/              ← SDE namespace
    solutions/{solution_id}/    ← Solution namespace
    ccrs/{ccr_id}/              ← CCR namespace
```

Cross-system links use the full qualified path:
```
kogi://my-cooperative/components/webapp-v2/
  → qala://my-factory/solutions/webapp/releases/v2.0/
```

This link resolves: the Kogi portfolio item for the webapp is linked to its Qala release record. Any developer on either system can navigate between them via the namespace path.

### 8.4 Shared Identity Model

All three systems use Hypergrid's `HG-ID` identity model at the substrate level:
- `IdentityId` (UUID) identifies an actor across all systems
- `TenantPartition` scopes data within a single identity's namespace
- `VisibilityMask` defines what each observer type can see across dimension combinations
- `SovereignTenant` represents the real-world entity that owns the data

At the domain layer:
- Kogi maps this to worker identity (independent contractor profile)
- Ume maps this to employee identity (org member profile)
- Qala maps this to developer identity (SDE operator profile)

A single person can have all three identity profiles simultaneously — they are a Kogi worker, a member of an Ume-governed organization, and a Qala developer. The `CrossTenantMerge` operation in HG-ID enables federated identity across the three systems while preserving each system's governance boundaries.

---

## 9. The Universal Cell Store: Data Gravity and Cross-System Queries

### 9.1 Data Gravity at the Cell Level

Traditional system integration is done by building API bridges between separate databases. Each system has its own schema, its own change tracking, its own version history. Integration means ETL pipelines, data warehouses, and synchronization jobs that are fragile, laggy, and expensive.

Hypergrid replaces this model with **data gravity at the cell level**. Every entity from every system lives in the same Universal Cell Store (or in federated UCS instances that sync via CrdtLog). Every relationship between entities in different systems is a typed edge in the shared Hypergraph. Every mutation from any system is an entry in the shared EventLog.

This creates a kind of gravitational field around the UCS: as more systems join the Hypergrid ecosystem, the value of cross-system queries increases superlinearly because every new system adds its entities and relationships to the same shared graph.

### 9.2 Cross-System HyperQL Queries

With Kogi, Ume, and Qala all building on Hypergrid, a platform administrator with access to the federation layer can issue HyperQL queries that span all three systems:

**Query 1: Find all Qala solutions produced by workers who are also Kogi portfolio contributors to a specific cooperative:**

```sql
-- Step 1: find all Kogi workers in cooperative X
SELECT D₁.entity_id AS worker_id
FROM kogi.portfolio.components
WHERE D₂."category" = '{"Item":"Portfolio"}'
  AND D₂."tags" CONTAINS 'cooperative:X';

-- Step 2: GRAPH TRAVERSE to their Qala solutions via CrossGridLink
TRAVERSE GRAPH
  FROM (SELECT worker_id FROM step_1)
  EDGE_TYPE = CrossGridLink
  DIRECTION = Outbound
  TARGET_GRID = qala://
  TARGET_CUBE = qala.solutions
SELECT target.solution_id, target.name, target.lifecycle_state;
```

**Query 2: Show all Ume OrgModule health scores correlated with Qala SDE drift events in Q2 2026:**

```sql
SELECT ume_module.name, ume_module.health_score,
       qala_sde.name AS affected_sde, qala_sde.drift_status
FROM ume.kernel.modules AS ume_module
JOIN qala.sdes AS qala_sde ON GRAPH_EDGE(ume_module.id, qala_sde.id, 'Association')
WHERE ume_module.lifecycle_state = '"Running"'
  AND qala_sde.drift_status != '"Clean"'
  AND ume_module.updated_at > '2026-04-01';
```

**Query 3: Contribution attribution for a multi-system project — Kogi portfolio contributions linked to Qala solution releases:**

```sql
SELECT k.contributor_id, k.attribution_weight,
       q.solution_id, q.name, q.version,
       q.lifecycle_state
FROM kogi.portfolio.contribution_ledger AS k
JOIN qala.solutions AS q ON GRAPH_EDGE(k.portfolio_component_id, q.solution_id, 'Contains')
WHERE k.governance_status = '"Accepted"'
  AND q.lifecycle_state = '"Active"'
FOLD D₃ WITH SUM(k.attribution_weight) AS total_contribution
ORDER BY total_contribution DESC;
```

These cross-system queries are only possible because all three systems share the same Hypergrid substrate. They replace what would otherwise require data pipelines, event bridges, and ad-hoc integration code.

### 9.3 Time-Travel Across All Systems

Because all three systems dual-write to the Hypergrid EventLog, the AS_OF time-travel operator works across the entire federation:

```sql
-- What was the state of all active Kogi portfolio components,
-- all running Ume modules, and all active Qala SDEs
-- at the moment of a specific incident (2026-03-15T14:22:00Z)?

SELECT *
FROM kogi.portfolio.components, ume.kernel.modules, qala.sdes
AS_OF '2026-03-15T14:22:00Z'
WHERE kogi.components.status = '"Active"'
  AND ume.modules.lifecycle_state = '"Running"'
  AND qala.sdes.status = '"Active"';
```

This query resolves by replaying each Grid's EventLog to the specified timestamp and materializing the state of each entity at that moment. This is invaluable for incident investigation, compliance audits, and capacity planning retrospectives.

---

## 10. Federation and Multi-System Synchronization

### 10.1 The Federation Model

Hypergrid's federation model supports N nodes up to 255 per Grid deployment. For domain systems, federation serves different purposes:

| Domain System | Primary Federation Use Case | Federation Topology |
|--------------|----------------------------|---------------------|
| **Kogi** | Multi-device worker sync (phone + desktop + tablet) | Star topology: one worker = one Grid, multiple device nodes |
| **Kogi** | Shared portfolio collaboration (multiple workers) | Mesh topology: contributor nodes sync to shared portfolio Grid |
| **Ume** | Multi-office organization (headquarters + branches) | Hub-and-spoke: central Grid + regional Grid replicas |
| **Ume** | M&A scenarios (acquiring org federates with acquired org) | CrossGridLink: two separate Ume Grids linked, not merged |
| **Qala** | Distributed enterprise (on-premises + cloud) | Hybrid topology: on-premises node + cloud node |
| **Qala** | Air-gapped deployments | Isolated island: manual delta sync packages |
| **All** | Cross-system collaboration | CrossGridLink with ShadowCell protocol |

### 10.2 Delta Sync Protocol

When a Kogi mobile app comes back online after being offline for 2 hours, its CrdtLog has accumulated 47 operations (tags added, portfolio items updated, toolboxes attached). The delta sync protocol:

1. App's VectorClock is compared to the server Grid's VectorClock
2. Server sends all operations the app hasn't seen yet (delta since app's cursor)
3. App applies remote operations via `apply_crdt_log()`: LWW wins per-attribute, OR-Set merges for sets, PnCounter sums for counters
4. App sends its 47 local operations to the server Grid
5. Server applies them in causal order (VectorClock ordering)
6. Both sides emit a `CrdtMergeApplied` event to the EventLog

The result: eventual consistency is achieved within one round-trip, with no data loss and no user-visible conflicts for well-designed CRDT-typed fields.

### 10.3 Schema Migration in a Federated Environment

When a domain system adds a new attribute key to a Hypercube (e.g., Kogi adds `toolbox_ids` for TMS integration), the federation must handle nodes running old and new schema versions simultaneously.

Hypergrid's `VersionAxis` and schema event log handle this: additive schema changes (new attribute keys) are commutative — any node that doesn't know about a new key simply ignores those cells. The schema event is replicated to all peers. Peers that haven't applied the schema update yet will receive the new cells when they sync, store them with the `UnregisteredAttributeKey` variant, and register them when they next bootstrap. No migration scripts, no downtime.

Destructive schema changes (removing or renaming attribute keys) require a governance gate — a CCR in Qala terms, an approval workflow in Kogi/Ume terms — because they cannot be automatically merged without potential data loss.

---

## 11. AI and Intelligence Layer Integration

### 11.1 The Two-Tier Computation Model

Hypergrid defines two tiers of computed attributes that all domain systems use:

**Tier 1: Synchronous Formula Attributes** — computed in-process at read time from other cell values in the same Hypercube. These are deterministic, cheap, and always fresh:
- Kogi: `budget_remaining = budget - budget_spent`
- Ume: `headcount_total = SUM(SLICE(D₂="headcount", D₃=DomainArea::*))`
- Qala: `sde_utilization_pct = (active_builds / total_sde_capacity) * 100`

**Tier 2: Asynchronous AI/ML Attributes** — computed by a registered AI engine plugin, written back to the Hypercube via the `WritebackProtocol`, cached with a confidence score and TTL. These are powerful, expensive, and eventually consistent:
- Kogi: `health_score`, `risk_score`, `collaboration_score`
- Ume: `module_health_score`, `compliance_prediction`, `hiring_need_forecast`
- Qala: `defect_risk_score`, `drift_status`, `optimization_recommendations`

### 11.2 AI Engine Architecture

All three systems' AI engines share the same plugin interface:

```rust
pub trait HypercubePlugin: Send + Sync {
    fn plugin_id(&self) -> &str;
    fn name(&self) -> &str;

    // Tier-2 AI attribute computation
    fn compute_attribute(
        &self,
        key: &AttributeKey,
        ctx: &ComputeContext<'_>,
    ) -> HypergridResult<TypedAttrValue>;

    // Called when relevant cells are mutated (push-based recomputation)
    fn on_cell_mutation(&self, cube_id: CubeId, coord: &DimCoordinate) -> Vec<AttributeKey>;
}
```

The `on_cell_mutation` hook enables push-based invalidation: when a Portfolio Component's risk records change (Kogi), the `KogiRiskEngine` plugin is notified and schedules a recomputation of `risk_score`. This eliminates polling and ensures AI scores are fresh within the engine's processing latency.

### 11.3 NL-to-HyperQL: The Natural Language Query Bridge

Hypergrid's `HG-AI` module includes a `NLToHyperQLEngine` plugin that translates natural language questions into HyperQL queries. All three systems can expose this capability to their users:

- **Kogi worker**: "Which of my projects are over budget this quarter?" → HyperQL on `kogi.portfolio.components` with budget filters
- **Ume manager**: "Which departments have the highest attrition risk in the last 6 months?" → HyperQL on `ume.hr.employees` joined to `ume.analytics.kpis`
- **Qala developer**: "Which SDEs have had the most drift events and the highest defect rates in the last 90 days?" → HyperQL on `qala.sdes` joined to `qala.solutions.metrics`

The NL engine is domain-aware: it is initialized with each domain system's Hypercube registry, so it understands domain-specific vocabulary ("project", "department", "SDE") and translates it to the correct cube names and attribute keys.

### 11.4 Anomaly Detection Across All Systems

The `AnomalyEngine` plugin can be registered across all three systems' Hypercubes. It subscribes to the EventLog stream for mutations to numeric attributes and flags cells whose values deviate significantly from historical patterns:

| System | Monitored Attribute | Anomaly Type | Action |
|--------|---------------------|--------------|--------|
| Kogi | `budget_spent` rate | SuddenChange | Alert portfolio owner |
| Kogi | `health_score` | PatternBreak | Trigger Oba AI recommendation |
| Ume | `finance.balance` | OutlierValue | Alert Finance module |
| Ume | `module.restart_count` | SuddenChange | Trigger supervisor review |
| Qala | `defect_density` | PatternBreak | Alert release manager |
| Qala | `sde.drift_status` | StateChange | Trigger drift remediation workflow |

The `anomaly_flag` attribute on any cell uses an AI-write only `AttrVisibility::Tenant` setting, so anomaly flags are visible to the domain system but not to external observers.

---

## 12. Security, Governance, and Compliance

### 12.1 Hypergrid's Permission Model Applied to Domain Systems

Hypergrid's `PermissionTier` (Viewer=0 through Admin=6) maps to each domain system's permission model:

| PermissionTier | Kogi | Ume | Qala |
|----------------|------|-----|------|
| Viewer (0) | Anonymous read | Employee read-only | Public solution browsing |
| Subscriber (1) | Follow/watch portfolio | Subscribe to module alerts | SDE observer |
| Contributor (2) | Comment, react, poll | Comment, submit to module | Junior developer |
| Editor (3) | Edit component content | Edit domain records | Developer with SDE access |
| Manager (4) | Manage settings/members | Department head | Tech Lead / Release Manager |
| Owner (5) | Full portfolio ownership | System Administrator | Factory Admin |
| Admin (6) | Platform Admin | Ume Kernel Admin | Qala Platform Admin |

Each `AttributeKeyDef` in every domain system's Hypercube schema specifies the minimum `write_permission` tier required to write that attribute. This is enforced at the bridge layer (`ComponentStore::write()`, `SolutionStore::write()`, etc.) before the value is written to any HyperCell.

### 12.2 Immutable Audit Trail

Because all three domain systems dual-write every mutation to the Hypergrid EventLog, the audit trail is automatically immutable, time-indexed, and tamper-evident. For Qala's compliance requirement (generating compliance reports in under 4 hours vs. 4–12 weeks industry average), this means:

1. Every CCR approval, every release, every SDE mutation is an EventLog entry
2. Compliance reports are generated by HyperQL queries against the EventLog filtered by time range and event kind
3. The EventLog is append-only — no record can be modified or deleted
4. For regulated industries (pharma, finance, government), the EventLog can be exported to a separate compliance vault

For Ume's `LogAuditManager`, the Hypergrid EventLog replaces the separate audit recording system. Every kernel service event, every RBAC decision, every module state change is an EventLog entry. The `AuditRecord` structure (id, at, actor, action, resource_id, outcome, context) maps directly onto `EventEntry` fields.

### 12.3 Privacy and Data Residency

Hypergrid's `VisibilityMask` and `TenantAxis` support the data residency requirements of all three systems:

- **Kogi**: Worker data is owned by the worker (SovereignTenant model). No data crosses tenant boundaries without consent. CrossGridLink edges require explicit consent before any mirror attribute syncs.
- **Ume**: Organization data is scoped to the org's TenantPartition. Branch offices can have their own regional Grid nodes that sync only approved fields to the central Grid.
- **Qala**: Multi-tenant factory isolation is enforced at the Hypercube level via TenantAxis. Each customer's SDEs and solutions are in a separate TenantAxis slice. Row-level security ensures no cross-tenant data leakage at the query level.

---

## 13. Deployment Topology

### 13.1 Single-Node Deployment (Development)

For development and small deployments, all three domain systems run in a single-node configuration: one `Grid` instance, one `UniversalCellStore` (in-memory or SQLite-backed), no federation. This is the configuration used by the Kogi mobile app, a solo Ume instance, or a Qala personal factory.

```
[Domain System Binary]
  └── Grid (single node)
       ├── Hypercube(s)
       ├── UniversalCellStore (in-memory)
       ├── Hypergraph
       ├── EventLog
       └── CrdtLog (no peers)
```

### 13.2 Federated Production Deployment

For production, each domain system runs multiple Grid nodes in a federation:

```
[Load Balancer]
    ├── [Domain System Node 1]
    │     └── Grid (node-1)
    │          ├── UniversalCellStore (PostgreSQL-backed)
    │          ├── CrdtLog → Kafka (sync topic)
    │          └── EventLog → Kafka (audit topic)
    │
    ├── [Domain System Node 2]
    │     └── Grid (node-2)
    │          └── (same structure)
    │
    └── [Domain System Node 3]
          └── Grid (node-3)
               └── (same structure)

[Kafka]
  ├── crdt.delta.{grid_id}    ← CrdtLog delta sync
  ├── events.audit.{grid_id}  ← EventLog replication
  └── ai.compute.{grid_id}    ← AI computation requests
```

The federation sync protocol uses Kafka for reliable delta delivery. Each node subscribes to all other nodes' `crdt.delta` topics, applies incoming operations via `apply_crdt_log()`, and publishes its own deltas when mutations occur.

### 13.3 Cross-System Deployment (Integrated Ecosystem)

When Kogi, Ume, and Qala are deployed as part of an integrated ecosystem (e.g., a cooperative of independent workers that is also a governed organization using both platforms):

```
[Kogi Grid] ←──── CrossGridLink ────→ [Qala Grid]
     │                                      │
     │──── CrossGridLink ────→ [Ume Grid] ──┘
     
[Federation Coordinator Service]
  Manages CrossGridLink consent flows
  Routes shadow cell sync operations
  Handles cross-grid HyperQL fan-out queries
```

---

## 14. Implementation Guide for New Domain Systems

### 14.1 Step-by-Step: Building a Domain System on Hypergrid

This section is a practical guide for engineers building a new domain-specific system on Hypergrid, following the same pattern used by Kogi, Ume, and Qala.

**Step 1: Define Your Domain Entities**

Identify the primary entities your domain system manages. For each entity, define its Rust struct with all fields typed using Hypergrid-compatible types (or types that serialize to `TypedAttrValue` variants).

```rust
// Example: a hypothetical Healthcare OS patient record entity
pub struct PatientRecord {
    pub patient_id:       Uuid,
    pub name:             String,
    pub date_of_birth:    DateTime<Utc>,
    pub conditions:       Vec<String>,          // will be OrSet
    pub medications:      Vec<MedicationRecord>, // will be OrSet
    pub risk_score:       Option<f64>,           // will be AI-computed
    pub care_team:        Vec<UserId>,            // will be OrSet
    // ... additional fields
}
```

**Step 2: Choose Your Hypercube Schema (N=2 or N>2)**

Most domain entities use N=2. Use N>2 when entities have a natural extra axis that enables valuable cross-sectional queries without post-hoc aggregation:

```
// N=2: patient records (entity × field)
healthcare.patients (D1=PatientId, D2=FieldName)

// N=3: patient vitals over time (entity × metric × time period)
healthcare.vitals (D1=PatientId, D2=MetricName, D3=TimeAxis)

// N=4: treatment outcomes by condition and population (entity × metric × time × category)
healthcare.outcomes (D1=PatientId, D2=MetricName, D3=TimeAxis, D4=ConditionCategory)
```

**Step 3: Assign CRDT Semantics to Every Field**

For every D₂ attribute key, determine the correct CRDT semantics from the policy table in §3.5. This is the most critical design decision — wrong CRDT semantics cause data loss or incorrect merge behavior in federated deployments.

```rust
// Example: registering CRDT semantics for patient record fields
AttributeKeyDef { key: "name",       crdt_semantics: CrdtSemantics::LastWriteWins, .. }
AttributeKeyDef { key: "conditions", crdt_semantics: CrdtSemantics::OrSet, .. }  // OR-Set!
AttributeKeyDef { key: "risk_score", crdt_semantics: CrdtSemantics::LastWriteWins, 
                  write_permission: PermissionTier::Admin,  // system-write only
                  computation: Some(AttrComputation::AiEngine { plugin_id: RISK_ENGINE_ID, .. }), .. }
```

**Step 4: Implement the Domain Store Codec**

Write the `DomainStore` codec that serializes/deserializes your domain entities to/from HyperCells. Follow the pattern established by Kogi's `ComponentStore`:

```rust
impl DomainStore<PatientRecord, Uuid> for PatientStore {
    fn bootstrap(grid: &mut Grid) -> HypergridResult<CubeId> {
        let cube_id = grid.create_cube("healthcare.patients")?;
        let cube = grid.get_cube_mut(cube_id)?;
        Self::register_schema(cube)?;
        Ok(cube_id)
    }

    fn write(grid: &mut Grid, cube_id: CubeId, entity: &PatientRecord, actor: &str)
             -> HypergridResult<()> {
        macro_rules! w { ($field:expr, $val:expr) => {
            grid.write_cell(cube_id, coord(entity.patient_id, $field), "value", $val, actor)?;
        }}
        w!("name",    TypedAttrValue::Text(entity.name.clone()));
        w!("dob",     as_json(&entity.date_of_birth));
        w!("conditions", as_json(&entity.conditions));
        // ... all fields
        Ok(())
    }

    fn read(grid: &Grid, cube_id: CubeId, id: Uuid) -> DomainResult<Option<PatientRecord>> {
        let cells = grid.get_row(cube_id, &DimKey::uuid(id));
        if cells.is_empty() { return Ok(None); }
        // build field_map from cells, deserialize entity
        // ...
    }
}
```

**Step 5: Define Your Hypergraph Edge Types**

Map your domain relationships to Hypergrid EdgeType variants. Add `Custom("your_domain_edge")` for domain-specific relationships that don't map to existing edge types.

**Step 6: Implement the Domain System Root**

Create a root struct (analogous to `PortfolioSystem`) that wraps the Hypergrid Grid and exposes domain-typed API methods. This struct is the public API of your domain system.

**Step 7: Register AI Engine Plugins**

For each AI-computed attribute in your schema, implement a `HypercubePlugin` with the `compute_attribute` method and `on_cell_mutation` hook.

**Step 8: Configure Spaces and Namespaces**

Register your domain's Space types and namespace conventions using Hypergrid's `HG-SPACE` system.

**Step 9: Expose HyperQL Pass-Through**

Add a `hyperql()` method to your domain system root that passes HyperQL queries directly to the underlying Grid. This gives AI engines, analytics, and advanced users the full query power of the substrate.

### 14.2 Domain System Checklist

Before shipping a domain system on Hypergrid, verify:

- [ ] All entity Hypercubes are registered and bootstrapped at startup
- [ ] All D₂ attribute keys have correct CRDT semantics (especially OrSet for any Vec/HashSet field)
- [ ] All AI-computed attributes have `write_permission: Admin` to prevent user overwrite
- [ ] Every mutating operation dual-writes to the domain EventLog
- [ ] The domain EventLog dual-writes to the Hypergrid EventLog
- [ ] The `VectorClock` is ticked on every mutation
- [ ] Cycle detection is enforced on all `Dependency` edge additions
- [ ] Consent is required before any `CrossGridLink` is established
- [ ] All sensitive attribute keys use `AttrVisibility::Tenant` or `AttrVisibility::Owner`
- [ ] The domain system exposes a `hyperql()` pass-through for advanced queries
- [ ] A `stats()` method exposes both domain-level and Hypergrid substrate metrics
- [ ] A `snapshot()` / `save_checkpoint()` API is available for backup integrations

---

## 15. Comparative Analysis

### 15.1 Traditional Approach vs. Hypergrid Substrate Approach

The following table compares building Kogi, Ume, and Qala using a traditional bespoke-schema approach versus the Hypergrid substrate approach:

| Concern | Traditional Approach | Hypergrid Substrate Approach |
|---------|---------------------|------------------------------|
| **Data modeling** | Separate schema per system, per entity type | One Universal Cell Store; domain entities are HyperRow projections |
| **Versioning** | Custom version table per entity type | VectorClock + EventLog built into every HyperCell |
| **Audit trail** | Custom audit log per service | EventLog is universal, immutable, time-indexed |
| **Time-travel** | Custom snapshot system or event sourcing | AS_OF operator in HyperQL — always available |
| **CRDT/collaboration** | Re-implement LWW, OR-Set, counters per system | One `hypergrid::crdt` crate shared by all |
| **Graph relationships** | Either separate graph DB or foreign keys | Hypergraph — typed, weighted, traversable, cross-grid |
| **AI attributes** | Separate AI service writing to separate tables | `AttrComputation::AiEngine` on any attribute key |
| **Cross-system integration** | ETL pipelines, API gateways, data warehouses | CrossGridLink + ShadowCell protocol |
| **Time-series data** | Separate time-series DB (InfluxDB, TimescaleDB) | D₃=TimeAxis Hypercube — standard query support |
| **Multi-dimensional analysis** | Separate OLAP cube or pivot tables | Any N-dim Hypercube with DimFold/DimExpand |
| **Federation** | Custom sync logic per system | VectorClock + CrdtLog — consistent delta sync |
| **Namespace/addressing** | Custom URL schemes per system | NamespacePath URI — universal addressing |
| **Permission model** | Custom RBAC per system | PermissionTier + AttrVisibility per attribute key |
| **Schema evolution** | Migration scripts + downtime | Additive schema changes are CRDT-commutative |
| **Cross-system queries** | Separate BI/analytics platform | HyperQL with federation fan-out |
| **Developer learning curve** | Learn each system's data model separately | Learn Hypergrid once; apply everywhere |

### 15.2 What Each System Gains Specifically

**Kogi gains:**
- CRDT-safe concurrent editing on all portfolio components across multiple worker devices
- Automatic time-travel for "what was my portfolio health last month?" queries
- Cross-cooperative attribution tracking via the 3D contribution ledger cube
- AI health/risk/alignment scores with confidence intervals and TTL-based staleness management
- Federation with Ume (organizational context) and Qala (solution development context)

**Ume gains:**
- A unified storage layer for all 42 organization modules instead of 42 separate schemas
- Cross-module queries in HyperQL instead of kernel facade calls
- AI module health monitoring as computed attributes with push-based invalidation
- Chombo multi-jurisdiction support via D₃=JurisdictionAxis
- Soko multi-segment analytics via D₃=TimeAxis + D₄=SegmentAxis
- The kernel's event bus becomes the EventLog — no separate event infrastructure needed

**Qala gains:**
- SDE snapshots as D₃=TimeAxis cells — no separate snapshot storage needed
- Drift detection as a Tier-2 computed attribute with sub-second EventLog trigger
- CCR lifecycle as a Lattice CRDT — governance invariants enforced at the substrate level
- Solution quality metrics as N=4 cube — multi-dimensional analytics without a separate data warehouse
- Domain Packs as HypercubePlugin — standard extension mechanism, no custom code
- Root Factory Principle is architecturally real: Qala's own record lives in its own cubes

### 15.3 Tradeoffs and Limitations

| Concern | Tradeoff | Mitigation |
|---------|----------|------------|
| **Serialization overhead** | Every entity field write is a separate `HyperCell` write | Batch writes in single transaction; profile hot paths |
| **Read reconstruction cost** | Reading an entity requires scanning all cells for its D₁ key | Cell scan is O(k) where k=field count; typically 20–50 fields; fast in practice |
| **Query complexity** | Complex multi-dim queries require HyperQL expertise | NL-to-HyperQL AI bridge; domain-specific query helpers |
| **Schema coupling** | Attribute key changes require all nodes to update | Additive changes are CRDT-commutative; destructive changes require governance gate |
| **Memory pressure** | In-memory UCS can grow large for high-entity-count deployments | Pluggable storage backend (PostgreSQL, ClickHouse) for large deployments |
| **N>6 dimensionality** | Queries become harder to reason about at N>6 | Hard limit N=16; recommendation: N≤6 for UI usability |
| **Status CRDT (v1)** | LWW on status is safe but not governance-enforced | v2 implements Lattice CRDT for lifecycle state machines |

---

## 16. Roadmap

### 16.1 Substrate Milestones (Hypergrid)

| Version | Milestone | Deliverables |
|---------|-----------|--------------|
| v0.5 | Alpha | HyperCell + N=2 dense encoding + EventLog + PostgreSQL backend + LWW CRDT |
| v1.0 | Beta | N=1–6 + Sparse/Hybrid encoding + HyperQL v1 + Formula attributes + HG-GRAPH + Federation v1 + HG-NS |
| v1.1 | Full N-dim + Graph | All 16 dim types + CrossGridLink + ShadowCell + Link forest + N-dim index optimization |
| v1.5 | Intelligence | HG-AI pluggable engine + NL-to-HyperQL + Anomaly detection + HG-ID multi-tenant |
| v2.0 | Full platform | All render modes + Domain Pack plugin system + Export (Parquet, Arrow, GraphQL) |
| v2.5 | Scale + Federation | ClickHouse for N>4 analytics + ZOrder curve multi-dim index + 255-node federation |
| v3.0 | Autonomous AI | AI agents on cubes with approval-first workflows + Predictive DimFold models |

### 16.2 Kogi Platform Milestones

| Milestone | Description |
|-----------|-------------|
| v2.1 (current) | Portfolio System on Hypergrid; `kogi.portfolio.components` cube; 11 analytical models |
| v2.2 | `kogi.portfolio.kpis` (N=3 time-series cube); AI health/risk/alignment scores |
| v2.3 | `kogi.portfolio.contribution_ledger` (N=3 collaboration cube); crowdresourcing campaigns |
| v2.4 | `kogi.portfolio.benefits` (N=3 benefits cube); portable benefit accounts |
| v2.5 | CrossGridLink to Ume Grid (org context); CrossGridLink to Qala Grid (solution context) |
| v3.0 | Kogi Marketplace on Hypergrid; asset exchange via CrossGridLink + ShadowCell protocol |

### 16.3 Ume Platform Milestones

| Milestone | Description |
|-----------|-------------|
| v2.0 (target) | Migrate all 42 module stores to Hypergrid Hypercubes; replace InMemoryEventBus with HG EventLog |
| v2.1 | N=3 cubes for Chombo (D₃=JurisdictionAxis) and Soko (D₃=TimeAxis + D₄=SegmentAxis) |
| v2.2 | Cross-module HyperQL queries replacing kernel facade calls for analytics |
| v2.3 | AI module health scoring via Tier-2 computed attributes |
| v2.4 | Federation support: multi-office org sync via Hypergrid CrdtLog |
| v3.0 | CrossGridLink to Kogi (worker portfolios) and Qala (solution governance) |

### 16.4 Qala Platform Milestones

| Milestone | Description |
|-----------|-------------|
| v2.0 (target) | Core cubes (factories, SDEs, solutions, CCRs) on Hypergrid; EventLog dual-write for compliance |
| v2.1 | SDE snapshots as D₃=TimeAxis cube; AS_OF rollback queries |
| v2.2 | Solution quality metrics as N=4 cube; multi-dimensional quality dashboards |
| v2.3 | Domain Packs as HypercubePlugin; Domain Extension Plane on Hypergrid |
| v2.4 | Drift detection as Tier-2 computed attribute; 15-minute SLA via EventLog push triggers |
| v2.5 | AI Agent fully on Hypergrid; all 10 AI capabilities as HypercubePlugin implementations |
| v3.0 | Root Factory self-governance; Qala's own platform managed through its own Hypercubes |

---

## 17. Appendices

### Appendix A: Hypergrid NamespacePath URI Scheme

All three domain systems use the following namespace addressing conventions:

```
# Hypergrid canonical form
hypergrid://{grid_name}/{space_type}/{slug}/{entity_type}/{entity_slug}/

# Kogi
kogi://worker/{user_slug}/portfolio/{component_slug}/
kogi://cooperative/{coop_slug}/portfolio/{component_slug}/
kogi://campaign/{campaign_slug}/contributions/{contributor_id}/

# Ume
ume://{org_id}/module/{module_name}/
ume://{org_id}/hr/employees/{employee_id}/
ume://{org_id}/finance/accounts/{account_id}/
ume://{org_id}/soko/campaigns/{campaign_id}/

# Qala
qala://{factory_slug}/sdes/{sde_id}/
qala://{factory_slug}/solutions/{solution_slug}/releases/{version}/
qala://{factory_slug}/ccrs/{ccr_id}/
qala://root/platform/{service_name}/  ← Root Factory self-governance

# Cross-system
kogi://worker/{user}/portfolio/{item}/ → qala://{factory}/solutions/{sol}/releases/{ver}/
ume://{org}/hr/employees/{emp}/ → qala://{factory}/sdes/{sde}/
```

### Appendix B: Complete CRDT Semantics Decision Tree

```
Is the attribute a counter that only ever increases?
  YES → GrowOnlyCounter (view counts, follower counts, analytics)
  NO ↓

Is the attribute a counter that can increase AND decrease?
  YES → PnCounter (budget_spent, allocated_units, resource consumption)
  NO ↓

Is the attribute a set of values where concurrent adds must all survive?
  YES → OrSet (tags, owners, members, children, dependencies, toolbox_ids)
  NO ↓

Is the attribute a number where the highest value should always win?
  YES → MaxRegister (version numbers, sequence numbers)
  NO ↓

Is the attribute a lifecycle state that must follow a partial order?
  YES → Lattice (ComponentStatus, SdeStatus, SolutionLifecycleState)
  NO ↓

Is the attribute human-authored content or a configuration value?
  YES → LastWriteWins (name, description, config, payload, JSON blobs)
  NO → Consult domain architects
```

### Appendix C: HyperQL Syntax Reference for Domain Systems

```sql
-- Basic entity query (N=2)
SELECT cell[D₁.id, "field_name"].value AS field
FROM {cube_name}
WHERE cell[D₁.id, "status"].value = '"Active"'
LIMIT 50 OFFSET 0;

-- Time-series slice (N=3 with TimeAxis)
SELECT D₁.entity_id, D₃.period, cell[D₁, D₂, D₃, "metric"].value AS value
FROM {cube_name}
WHERE D₃.period BETWEEN '2025-Q1' AND '2026-Q4'
  AND D₂.metric_name = 'health_score'
ORDER BY D₃.period;

-- DimFold (aggregate over a dimension)
SELECT D₁.entity_id, FOLD D₃ WITH AVG(cell.value) AS avg_score
FROM {cube_name}
WHERE D₂.metric_name = 'health_score'
  AND D₃.period >= '2025-Q1'
GROUP BY D₁.entity_id;

-- Graph traversal
TRAVERSE GRAPH
  FROM {start_node_id}
  EDGE_TYPE = Hierarchy
  DIRECTION = Outbound
  MAX_DEPTH = 5
SELECT node.id, node.name, node.status;

-- AS_OF time-travel
SELECT *
FROM {cube_name}
AS_OF '2026-03-15T14:22:00Z'
WHERE cell[D₁, "status"].value = '"Active"';

-- Cross-cube join via Hypergraph
SELECT a.name AS component_name, b.name AS linked_solution
FROM kogi.portfolio.components AS a
JOIN qala.solutions AS b ON GRAPH_EDGE(a.id, b.id, 'CrossGridLink')
WHERE a.status = '"Active"' AND b.lifecycle_state = '"Active"';

-- Multi-dim pivot (DimExpand)
SELECT D₁.solution_id,
       EXPAND D₄ AS COLUMNS(AVG(cell.value))
FROM qala.solutions.metrics
WHERE D₂.metric_name = 'defect_density'
  AND D₃.period >= '2026-Q1'
GROUP BY D₁.solution_id;
-- Result: solution_id | security_avg | quality_avg | performance_avg | compliance_avg
```

### Appendix D: Domain Pack Plugin Interface for Qala

```rust
// A Qala Domain Pack is a HypercubePlugin + attribute registrations + edge types
pub struct PharmaDomainPack;

impl HypercubePlugin for PharmaDomainPack {
    fn plugin_id(&self) -> &str { "qala.domain_pack.pharma.v1" }
    fn name(&self) -> &str { "Pharmaceutical Formulation Domain Pack" }

    fn attribute_key_defs(&self) -> Vec<AttributeKeyDef> {
        vec![
            // Fields added to qala.solutions for pharma formulations
            lww_text("batch_formula_version", "Batch Formula Version"),
            lww_text("pharmacopoeial_grade",  "Pharmacopoeial Grade"),
            lww_json("ingredient_specs",      "Ingredient Specifications"),
            lww_json("regulatory_submissions","Regulatory Submissions"),
            lww_text("gmp_compliance_status", "GMP Compliance Status"),
            lww_json("stability_data",        "Stability Study Data"),
            // AI-computed
            AttributeKeyDef {
                key: "regulatory_risk_score".into(),
                attr_type: AttributeType::Ai,
                write_permission: PermissionTier::Admin,
                computation: Some(AttrComputation::AiEngine {
                    plugin_id: PHARMA_REGULATORY_AI_ID,
                    model_name: "regulatory_risk_v2".into(),
                    parameters: HashMap::new(),
                }),
                ..lww_number("regulatory_risk_score", "Regulatory Risk Score")
            },
        ]
    }

    fn compute_attribute(
        &self,
        key: &AttributeKey,
        ctx: &ComputeContext<'_>,
    ) -> HypergridResult<TypedAttrValue> {
        if key == "regulatory_risk_score" {
            // invoke pharma AI model
            let ingredients = ctx.current_attrs.get("ingredient_specs");
            let submissions = ctx.current_attrs.get("regulatory_submissions");
            let score = self.regulatory_risk_engine.compute(ingredients, submissions);
            return Ok(TypedAttrValue::AiSignal {
                value: Box::new(TypedAttrValue::Number(score.value)),
                confidence: score.confidence,
                model: "regulatory_risk_v2".into(),
            });
        }
        Ok(TypedAttrValue::Null)
    }
}
```

### Appendix E: Key Design Decisions Summary

| Decision | Chosen Approach | Rationale |
|----------|-----------------|-----------|
| Primary entity storage layout | D₁=EntityId, D₂=FieldName, cell.attributes["value"] | Maximizes HyperCell reuse of built-in "value" attribute; simplest codec |
| Status/lifecycle CRDT | LWW in v1; Lattice in v2 | LWW is safe and simple; Lattice adds governance enforcement at substrate level |
| AI writeback mechanism | `HypercubePlugin::compute_attribute` + system-write PermissionTier | Consistent with plugin architecture; prevents user overwrite of AI scores |
| Cross-system integration | CrossGridLink + ShadowCell | Consent-gated; attribute-level granularity; no full data copy |
| Federation sync | CrdtLog delta sync over Kafka | Reliable delivery; causal ordering; scales to 255 nodes |
| Schema evolution | Additive changes are CRDT-commutative | Zero-downtime schema updates; destructive changes require governance gate |
| Query language | HyperQL (custom N-dim) + NL bridge | Native N-dim query support; NL bridge for user-facing features |
| Event bus | Dual-write to domain EventLog + HG EventLog | Unified audit trail + domain-typed querying |
| Namespace scheme | `{domain}://{space}/{entity_type}/{id}/` | Domain prefixes prevent collision; URI-shaped for HTTP integration |
| Hypercube naming | `{domain}.{entity_type}` | Clear ownership; prevents naming collision across domain systems |

---

*End of Document*

*Hypergrid as a Universal Domain-Specific System Substrate · v1.0 · March 2026*

*Kogi Platform · Ume Platform · Qala Platform*
