# APAPO — Universal Hypergrid Platform

## Kogi · Ume · Qala on the Hypergrid N-Dimensional Distributed Spreadsheet System

**Document Type:** Master Design Document — Platform Architecture Reference  
**Version:** 1.0 · March 2026 · Confidential — Internal Use Only  
**Systems:** Apapo Platform · Hypergrid NDSS · Kogi (IW-OS) · Ume (B-OS) · Qala (SF-OS)  
**Languages:** Rust · Go · Scala 3 · TypeScript · SQL (PostgreSQL)

---

## Table of Contents

**Part I — The Apapo Platform:** What is Apapo · The Three Root Domains · Everything is a Spreadsheet Row · The Self-Governing Platform · Design Philosophy

**Part II — Hypergrid Substrate:** What is Hypergrid · The Formal N-Dimensional Model · Conventional Spreadsheet vs. Hypergrid · The HyperCell · Universal Cell Store · The Hypergraph · EventLog · CRDT · HyperQL · AI Computation · HG-SPACE/NS/ID · Federation · Domain-Layer Pattern

**Part III — Kogi:** Portfolio as Root · KIMDSS Architecture · PortfolioComponent · Hypercube Inventory · 35 Sheets as Views · KLNK Graph · KPID Multi-Identity · kogi-engine and Oba · Spaces · Cross-System Integration

**Part IV — Ume:** Organization as Root · Organization System Architecture · Kernel-to-Hypergrid Mapping · 42 Module Hypercubes · Chombo N=3 · Soko N=4 · RBAC · AI Engine · Cross-System Integration

**Part V — Qala:** Solution as Root and Universal Category · Solution System Architecture · Solutions/Factories/SDEs as HyperRows · Hypercube Inventory · SDE as N=3 Versioned Cube · Solution Lifecycle Lattice · CCR on Hypergrid · Domain Packs as Plugins · AI Agent · Root Factory Principle · Kogi and Ume as Solutions

**Part VI — Apapo Convergent Platform:** Cross-System Data Gravity · Cross-System HyperQL · Federation Protocol · CrossGridLink and ShadowCell · Identity Across Systems · The Economic Graph · Unified Audit Trail · Self-Governing Platform

**Part VII — Technical Reference:** Storage Architecture · Service Architecture · Performance Targets · Security Pipeline · Bootstrap Sequence · Configuration Reference

**Appendices:** A. Complete Hypercube Registry · B. CRDT Decision Tree · C. HyperQL Syntax · D. Domain Pack Interface · E. Design Decisions Log · F. Glossary

---

# Part I — The Apapo Platform

## 1. What is Apapo?

Apapo is the canonical name for the complete integrated platform ecosystem formed by the Hypergrid N-Dimensional Distributed Spreadsheet System (NDSS) substrate and the three domain operating systems that run on top of it: **Kogi** (Independent Worker OS), **Ume** (Business OS), and **Qala** (Solution Factory OS).

Apapo is not a product name layered on top of existing systems. It is the correct name for the convergent whole — the living, federated, AI-augmented platform that unifies how individuals, organizations, and solution factories manage their data, operations, intelligence, and economic relationships. Where Hypergrid is the engine, Apapo is the vehicle.

Every entity in Apapo — from a solo freelancer's portfolio item to a multinational organization's legal entity to a regulated pharmaceutical solution under FDA review — is a **HyperRow** in a domain-specific Hypercube in the same Universal Cell Store, versioned by the same EventLog, synchronized by the same CRDT protocol, and reasoned over by the same AI framework. Domain systems are *lenses* over this shared substrate.

> **The Core Insight:** Every complex software system faces the same underlying challenge — how to store, version, relate, compute over, and distribute structured data about entities that exist in multiple contexts simultaneously, change over time, and are operated on collaboratively by many actors at once. Apapo's answer is a generalized N-dimensional distributed spreadsheet: Hypergrid. Every domain entity in every domain system is a row in a living spreadsheet.

---

## 2. The Three Root Domains

The most important architectural decision in the Apapo platform is that each domain operating system is organized around exactly **one root domain concept** — the entity that is the absolute center of gravity for everything in that system. This root concept is not a feature, not a component, not a module. It *is* the system.

| System | Domain | Root Concept | Root Hyperspreadsheet | Core Claim |
|--------|--------|-------------|----------------------|------------|
| **Kogi** | Independent Worker OS | The Portfolio | Portfolio System — master spreadsheet of the independent worker's entire operational world | Every entity in a worker's life — project, task, asset, gig, benefit, finance, relationship — is a row in one living portfolio spreadsheet |
| **Ume** | Business OS | The Organization | Organization System — master spreadsheet of the organization's entire operational world | Every entity in an organization's world — module, employee, legal entity, campaign, risk, OKR, ledger entry — is a row in one living organization spreadsheet |
| **Qala** | Solution Factory OS | The Solution | Solution System — master spreadsheet of the solution factory's entire production world | Every entity in a factory's world — SDE, artifact, CCR, release, toolchain — is a row in one living solution spreadsheet. The factory itself is a Solution. Kogi is a Solution. Ume is a Solution. |

This root-domain architecture is not incidental — it is the primary design principle of Apapo. The Portfolio is not a feature of Kogi — **it is Kogi**. The Organization is not a feature of Ume — **it is Ume**. The Solution is not a feature of Qala — **it is Qala**.

---

## 3. The Radical Claim: Everything is a Spreadsheet Row

The philosophical foundation of Apapo is a deliberately radical claim: every entity in every domain, at any scale, can be represented as a row in an N-dimensional distributed spreadsheet.

This claim would be trivially dismissed if "spreadsheet" meant a conventional 2D Excel-style grid. But Hypergrid generalizes the spreadsheet to N dimensions, makes it distributed, gives every cell per-attribute CRDT semantics, attaches a graph layer, makes it time-travelable, and wires it to an AI computation engine. The resulting structure is powerful enough to model anything.

A "spreadsheet row" in Hypergrid is: an entity with an identifier (D₁ key), a set of typed properties (D₂ keys with TypedAttrValues), existence in multiple simultaneous contexts (D₃–Dₙ axes), a history of changes (EventLog entries), relationships to other rows (Hypergraph edges), computed properties derived from other rows (ComputedAttributes), and governance rules about who can read and write each property (PermissionTier + AttrVisibility).

With those primitives, you can model:
- A freelancer's project — name, status, budget, timeline, risks, deliverables, collaborators, dependencies, AI health score
- A multinational legal entity — name, jurisdiction-sliced compliance data, filing history, risk flags across time
- A pharmaceutical drug formulation — formula version, ingredient specs, regulatory submissions per jurisdiction, GMP compliance status, AI-computed regulatory risk score
- A Solution Factory — owner org, child factory IDs, domain pack registry, governance policy, SDE roster

All of these are HyperRows. The spreadsheet, generalized, is universal.

---

## 4. The Self-Governing Platform Principle

The most philosophically powerful aspect of the Apapo architecture is Qala's self-referential root-domain principle.

Because Qala's root domain concept is the Solution and the Solution is the universal category, **Qala itself is a Solution**. The Qala platform is a first-class HyperRow in its own `qala.solutions` Hypercube. It is governed by its own Root Factory. Every platform update goes through a Change Control Request. Every platform release is a governed lifecycle event.

But it goes further. Because the Solution is the universal category, **Kogi and Ume are also Solutions in Qala's ontology**:

- **Kogi (Independent Worker OS)** is a Solution of type Platform, produced by the Qala Root Factory, tracked through the standard solution lifecycle, versioned with semantic versioning, subject to the Software Domain Pack's governance rules including SAST, DAST, build attestation, and release sign-off.
- **Ume (Business OS)** is a Solution of type Platform, similarly governed by the Qala Root Factory, versioned and life-cycle-managed.
- **The Qala Root Factory** is itself a Solution of type Platform. It is the factory that produced itself.

Practical consequences:
1. The platform operations team uses the same HyperQL queries and AI Agent dashboards to manage Kogi and Ume as solutions that customers use to manage their own solutions.
2. Every Kogi feature release follows the exact same CCR → review → approval → release pipeline that customer solutions follow.
3. The audit trail for the platform's own evolution lives in the same Hypercubes that store customer solution audit trails.
4. The platform continuously demonstrates its own governance capabilities by exercising them on itself.

---

## 5. Design Philosophy

| Principle | Statement | Implementation |
|-----------|-----------|----------------|
| **Universal Abstraction** | Every platform entity is representable as a HyperRow | Single PortfolioComponent / OrgModule / Solution struct; typed discriminant; universal HyperCell schema |
| **Root-Domain Sovereignty** | Each domain system is organized entirely around its root concept | Portfolio, Organization, Solution are not features — they are the systems |
| **Event-Sourced Truth** | Every mutation is an immutable EventLog entry. Nothing deleted — only archived. | PostgreSQL append-only EventLog; CRDT replay on merge; AS_OF time-travel |
| **Distributed-First** | Designed for multi-node, multi-device, offline-capable operation | VectorClock per entity; LWW + OR-Set + Lattice CRDT; federation via Kafka |
| **Identity Sovereignty** | A Sovereign Entity controls their data, identities, visibility, and connections | KPID multi-identity; VisibilityMask; SplitPolicy; owner-level encryption |
| **Connected Economy** | Every spreadsheet is a node in a global economic graph | KLNK CrossGridLinks; ShadowCell; MirrorAttribute; LinkForest |
| **AI-Augmented Intelligence** | The spreadsheet actively surfaces intelligence, flags risks, suggests actions | WritebackService; AiSignal cells; Oba; 12 Kogi engines; 10 Qala AI capabilities |
| **Open Extensibility** | Extensible at every layer without forking the substrate | HypercubePlugin trait; Domain Pack system; custom axis and attribute types |
| **Self-Governing Platform** | Qala governs Kogi and Ume as Solutions in the Root Factory | Platform releases go through CCR; platform telemetry lives in Qala cubes |

---

# Part II — Hypergrid: The Universal Substrate

## 6. What is Hypergrid?

Hypergrid is a platform-agnostic, open-architecture, N-dimensional distributed spreadsheet system. It provides any application platform with a universal, infinitely extensible data substrate that models all entities, relationships, computations, and intelligence as a single, coherent, living grid.

The Hypergrid module inventory:

| Module | Code | Language | Responsibility |
|--------|------|----------|----------------|
| Core Substrate | HG-CORE | Rust | Grid, Hypercube, DimensionAxis, HyperCell, HyperRow, EventLog, CrdtLog, VectorClock, PolicyEngine |
| Dimension System | HG-DIM | Rust | Declare, type, index, and govern custom dimensions. Sparse/dense/hybrid encoding. |
| Cell Attribute Model | HG-CELL | Rust | N-attribute cell: attribute key registry, TypedAttrValue variants, CRDT, computed attributes |
| View Engine | HG-VIEW | Go | HypercubeView: N-dim filter, sort, group, pivot, DimFold, DimExpand, board modes |
| Computation Engine | HG-COMP | Rust + Scala | Tier-1 formula engine and Tier-2 async AI/ML engine. Rollup aggregation. |
| Distributed Consistency | HG-CRDT | Rust | N-dim CRDT: LWW, OR-Set, MaxRegister, GrowOnlyCounter, Lattice. VectorClock per node. |
| Graph & Network | HG-GRAPH | Rust + Go | Hypergraph: typed edges between any entities. CrossGridLink, ShadowCell, LinkForest. |
| Spaces & Workspaces | HG-SPACE | Go | Bounded operational contexts (Spaces) and active working sessions (Workspaces). |
| Identity & Multi-Tenancy | HG-ID | Go | Multi-tenant, multi-identity. TenantPartition, VisibilityMask, SplitPolicy, CrossTenantMerge. |
| Namespace System | HG-NS | Go | Hierarchical URI addressing. NamespacePath URI scheme. Federation-aware resolution. |
| Export & Integration | HG-EXPORT | Go | CSV, XLSX, JSON, JSON-LD, Parquet, Arrow, GraphQL, webhook, streaming export. |
| AI Intelligence Layer | HG-AI | Rust + Python | Pluggable AI engines: anomaly detection, health scoring, recommendation, NL-to-HyperQL. |
| Query Language | HG-QL | Rust | HyperQL: SELECT, DimSlice WHERE, FOLD, EXPAND, TRAVERSE GRAPH, AS_OF, SHADOW INCLUDE. |
| Plugin System | HG-PLUGIN | Rust | HypercubePlugin trait: custom dim types, attr types, AI engines, render modes, connectors. |
| Telemetry & Ops | HG-OPS | Go + Prometheus | OpenTelemetry tracing, Prometheus metrics, structured audit logs, health probes. |

---

## 7. The Formal N-Dimensional Model

```
H = (D₁, D₂, ..., Dₙ)   where Dᵢ = DimensionAxis(id, name, type, key_set, ordering, cardinality)

A HyperCell C is addressed by: C = Cell(k₁ ∈ D₁.key_set, ..., kₙ ∈ Dₙ.key_set)
C.attributes = { aᵢ: vᵢ }   where aᵢ ∈ AttributeKeyRegistry, vᵢ is TypedAttrValue

Universal Cell Store: UCS = { (grid_id, cube_id, dim_keys_tuple) → HyperCell }

Domain entity convention (all three Apapo systems):
  D₁ = EntityAxis  (key_type: Uuid)        ← entity row (entity ID)
  D₂ = PropertyAxis (key_type: String)     ← property column (field name)
  cell(entity_id, "field_name").attributes["value"] = <TypedAttrValue>
```

Built-in AxisType variants:

| AxisType | Key Type | Ordering | Used By |
|----------|----------|----------|---------|
| EntityAxis | Uuid | Unordered | D₁ — rows in all domain cubes |
| PropertyAxis | String | Lexicographic | D₂ — field names in all domain cubes |
| TimeAxis | Timestamp | Chronological | D₃ — Kogi KPIs, Ume analytics, Qala SDE, CI/CD |
| GeoAxis | GeoHash | Spatial | D₄+ — geographic slicing |
| CategoryAxis | String (enum) | Ordinal | D₃ — Chombo jurisdictions; D₄ — Qala metrics |
| HierarchyAxis | HierarchyPath | Tree | Org charts, product taxonomy |
| TenantAxis | TenantId | Unordered | Multi-tenant shared-cube deployments |
| ScenarioAxis | String | Custom | What-if and planning axes |

---

## 8. Conventional Spreadsheet vs. Hypergrid

| Spreadsheet Concept | Conventional Spreadsheet | Hypergrid Generalization |
|---------------------|--------------------------|--------------------------|
| Workbook | Single file with multiple sheets | Grid — root container for all Hypercubes, tenants, namespaces, spaces, graph |
| Sheet | 2D grid of rows × columns | Hypercube H=(D₁,D₂,...,Dₙ) — projection of the Universal Cell Store |
| Row | Horizontal sequence of cells | HyperRow — all cells sharing the same D₁ key. The canonical entity. |
| Column | Vertical sequence of cells | D₂ Axis — typed, indexed, governed. Per-key CRDT semantics and PermissionTier. |
| Extra dims | (none) | D₃…Dₙ — custom named axes: Time \| Geography \| OrgUnit \| Tenant \| Scenario |
| Cell | Single value at row × column | HyperCell — N-dim coordinate carrying an N-attribute map |
| Formula | Function over cells in same sheet | ComputedAttribute — Tier-1 (sync) or Tier-2 (async AI) over any cell |
| Filter | Criterion applied to rows | DimSlice — predicate on any combination of axes |
| Sheet tab | Named view over 2D grid | HypercubeView — saved projection, filter, sort, render mode over N-dim sub-space |
| Workbook link | External reference | CrossGridLink + ShadowCell — typed edge connecting entities across Grids |
| Version history | (none) | EventLog — every mutation captured; AS_OF time-travel to any state |
| Collaborative editing | Last-save-wins (broken) | VectorClock + CRDT per attribute — deterministic concurrent merge |

---

## 9. The HyperCell: Universal Data Atom

```rust
pub struct HyperCell {
    // Coordinate
    pub grid_id:        GridId,
    pub cube_id:        CubeId,
    pub coord:          DimCoordinate,  // Vec<DimKey> — [D₁_key, D₂_key, ..., Dₙ_key]
    // N=2: [ComponentId_uuid, "health_score"]
    // N=3: [ComponentId_uuid, "health_score", "2026-Q2"]
    // N=4: [CampaignId_uuid, "ctr", "2026-Q2", "enterprise"]

    // Payload
    pub attributes:     AttributeMap,   // HashMap<AttributeKey, TypedAttrValue>
    // Always includes built-ins: "value", "data_type", "format"

    // Versioning & causality
    pub vector_clock:   VectorClock,    // HashMap<NodeId, u64>
    pub version:        u64,            // monotonic per-cell counter
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
    pub last_actor:     ActorId,        // "{tenant_id}:{identity_tag}:{node_id}"

    // Governance
    pub visibility:     CellVisibility, // Public|Tenant|Identity|Private
    pub policy_ids:     Vec<PolicyId>,
    pub audit_trail:    EventLogRef,

    // Graph integration
    pub edge_refs:      Vec<EdgeRef>,

    // Computed attribute cache
    pub computed_cache: HashMap<AttributeKey, CachedComputedAttr>,
}
```

The TypedAttrValue enum carries every possible value — from simple text to AI signals with confidence intervals to anomaly detection results to shadow references:

```rust
pub enum TypedAttrValue {
    Text(String),
    Number(f64),
    Integer(i64),
    Currency { amount: Decimal, currency_code: String },
    Percent(f64),
    Bool(bool),
    Date(NaiveDate),
    DateTime(DateTime<Utc>),
    Duration(Duration),
    Relation(EntityRef),
    MultiRelation(Vec<EntityRef>),
    User(IdentityId),
    TagSet(HashSet<String>),
    Json(serde_json::Value),
    Formula { expression: String, result: Box<TypedAttrValue>, cached_at: DateTime<Utc> },
    AiSignal {
        value:       Box<TypedAttrValue>,
        model_id:    String,
        computed_at: DateTime<Utc>,
        confidence:  f64,           // 0.0–1.0
        explanation: Option<String>,
    },
    AnomalySignal {
        kind:      AnomalyKind,    // SuddenChange|PatternBreak|OutlierValue|StateChange
        severity:  f64,
        baseline:  Box<TypedAttrValue>,
        deviation: f64,
    },
    ShadowRef { source_grid: GridId, source_coord: DimCoordinate },
    RichText { markdown: String, plain: String },
    Custom { type_id: String, payload: serde_json::Value },
    Null,
}
```

---

## 10. The Universal Cell Store

The UCS is a hash-indexed table keyed by `(grid_id, cube_id, dim1_key, dim2_key, [dim3_key...dim16_key])`. The PostgreSQL schema:

```sql
CREATE TABLE hypergrid_cells (
    grid_id         UUID NOT NULL,
    cube_id         UUID NOT NULL,
    dim1_key        UUID NOT NULL,              -- EntityAxis (entity ID)
    dim2_key        TEXT NOT NULL,              -- PropertyAxis (field name)
    dim3_key        TEXT,                       -- TimeAxis / CategoryAxis (if N≥3)
    dim4_key        TEXT,                       -- GeoAxis / TenantAxis (if N≥4)
    attr_value      JSONB NOT NULL DEFAULT '{}', -- TypedAttrValue as JSONB
    attr_version    BIGINT NOT NULL DEFAULT 0,
    attr_crdt_ts    BIGINT NOT NULL,            -- Logical clock timestamp (LWW)
    last_actor      TEXT NOT NULL,
    last_mutated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (grid_id, cube_id, dim1_key, dim2_key, dim3_key, dim4_key)
) PARTITION BY HASH(grid_id);
```

Three storage encodings, chosen at Hypercube creation time:

| Encoding | When To Use | Physical Layout | Lookup |
|----------|-------------|-----------------|--------|
| **Dense** | N=2 with small fully-populated key sets | 2D row-per-entity layout; JSONB column | O(1) primary key |
| **Sparse** | TimeAxis, GeoAxis, >100K keys | Key-value table; only non-null cells stored | O(log n) BTree/BRIN |
| **Hybrid** | D₁×D₂ dense; D₃–Dₙ sparse | Dense 2D base + sparse extension table joined at query | O(1) base; O(log n) extension |

Hybrid is the most common production pattern for all three domain systems.

---

## 11. The Hypergraph: Relations as First-Class Citizens

```
HypergraphEdge {
    edge_id:        EdgeId,
    from_node:      NodeId,          // maps to a HyperRow in some Hypercube
    to_node:        NodeId,          // same or different Hypercube / Grid
    edge_type:      EdgeType,        // Hierarchy|Dependency|Association|Contains|
                                     // CrossGridLink|ShadowOf|SpaceMembership|
                                     // Collaborates|References|FederationPeer|
                                     // ComputedFrom|InvestedIn|Custom
    direction:      EdgeDirection,   // Directed | Bidirectional
    weight:         f64,             // 0.0–1.0
    attributes:     AttributeMap,    // edge-level N-attributes
    consent_status: ConsentStatus,   // Pending|Accepted|Declined|Revoked
    expires_at:     Option<DateTime<Utc>>,
}
```

Graph uses by domain system:
- **Kogi**: Portfolio hierarchy, KLNK worker-to-worker links, dependency tracking, social/economic graph
- **Ume**: 42-module dependency map, org chart, cross-module workflow connections
- **Qala**: Solution hierarchy (Factory→SDE→Solution), artifact provenance, CCR approval chains

---

## 12. The EventLog: Immutable Audit and Time-Travel

```sql
CREATE TABLE hypergrid_events (
    event_id        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    grid_id         UUID NOT NULL,
    cube_id         UUID NOT NULL,
    dim_keys        JSONB NOT NULL,     -- Full N-dim coordinate tuple
    attr_key        TEXT NOT NULL,
    before_value    JSONB,
    after_value     JSONB NOT NULL,
    crdt_op         TEXT NOT NULL,      -- LWW|OR-Set-Add|OR-Set-Remove|Counter|Lattice
    vector_clock    JSONB NOT NULL,     -- {node_id: logical_timestamp}
    actor           TEXT NOT NULL,
    timestamp       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT no_update CHECK (true)  -- Enforced at application layer
) PARTITION BY RANGE(timestamp);       -- Monthly partitions
```

The EventLog serves four purposes simultaneously:
1. **Immutable Audit Trail** — every action by every actor, permanently preserved
2. **Time-Travel (AS_OF)** — replay to any historical timestamp for any entity
3. **CRDT Replay** — partition-recovering nodes replay unapplied operations to catch up
4. **AI Feature Store** — kogi-engine, Ume AI, Qala AI consume EventLog streams via Kafka

Dual-write architecture: every domain system maintains its own domain-typed event log (for efficient typed queries) while simultaneously writing every event to the shared Hypergrid EventLog as `EventKind::Custom(domain_tag)`. Single unified audit trail spanning all domains.

---

## 13. CRDT: Distributed Consistency Without Coordination

| CRDT Type | Where Used | Merge Rule |
|-----------|-----------|------------|
| **LastWriteWins (LWW)** | name, description, config, payload, JSON blobs | Highest VectorClock timestamp wins; lexicographic actor tiebreak |
| **OR-Set** | owners, tags, children, dependencies, members, toolbox_ids, approver_ids | All concurrent Adds survive; Removes tagged — only exact tagged entry removed |
| **GrowOnlyCounter** | restart_count, view_count, follower_count | Sum of all increments; never decrements |
| **PnCounter** | budget_spent, hours_logged, allocation consumption | Sum of (positive_ops − negative_ops) per node |
| **MaxRegister** | version, sequence_number, schema_version | Highest numeric value wins |
| **Lattice** | lifecycle_state, status, module_lifecycle_state | Join per partial order; forward state always wins in concurrent transitions |

The Lattice CRDT is the key to enforcing governance invariants at the substrate level. When the Qala solution lifecycle lattice is defined, a Lattice CRDT on `lifecycle_state` guarantees concurrent transitions always resolve to a valid forward state — making governance a substrate property, not application code.

```rust
// VectorClock — causal ordering across all federation nodes
pub struct VectorClock {
    pub counters: HashMap<NodeId, u64>,
}
// A happened-before B iff A.counters ≤ B.counters on all nodes
// Concurrent writes: neither clock dominates — CRDT semantics resolve
```

---

## 14. HyperQL: The N-Dimensional Query Language

Sample queries across the three systems:

```sql
-- Kogi: Portfolio health dashboard
SELECT D₁.component_id, cell[D₁, "name"].value AS name,
       cell[D₁, D₂, "health_score", "2026-Q2"].value AS health
FROM kogi.portfolio.kpis
WHERE D₂.metric_name = 'health_score' AND cell[D₁, "status"].value = '"Active"'
ORDER BY health DESC;

-- Ume: Cross-jurisdiction compliance aggregate  
SELECT D₃.jurisdiction, AVG(cell[D₁, "compliance_score", D₃].value) AS avg_compliance
FROM ume.chombo.entities
WHERE D₃.jurisdiction IN ("US-DE", "UK", "EU", "ZA")
GROUP BY D₃.jurisdiction;

-- Qala: Multi-dim quality pivot
SELECT D₁.solution_id, EXPAND D₄ AS COLUMNS(AVG(cell.value))
FROM qala.solutions.metrics
WHERE D₂.metric_name = 'quality_score' AND D₃.quarter IN ('2026-Q1', '2026-Q2')
GROUP BY D₁.solution_id;
-- Result: solution_id | security_avg | quality_avg | performance_avg | compliance_avg

-- Cross-system: Kogi workers contributing to Qala solutions
TRAVERSE GRAPH
  FROM (SELECT D₁.entity_id FROM kogi.portfolio.components
        WHERE cell[D₁, "tags"].value CONTAINS '"cooperative:X"')
  EDGE_TYPE = CrossGridLink DIRECTION = Outbound TARGET_GRID = qala://
SELECT target.solution_id, target.name, target.lifecycle_state;

-- AS_OF time-travel across all systems at incident timestamp
SELECT 'kogi' AS system, COUNT(*) AS count
FROM kogi.portfolio.components AS_OF '2026-03-15T14:22:00Z'
WHERE cell[D₁, "status"].value = '"Active"'
UNION ALL
SELECT 'ume' AS system, COUNT(*) FROM ume.kernel.modules AS_OF '2026-03-15T14:22:00Z'
WHERE cell[D₁, "lifecycle_state"].value = '"Running"'
UNION ALL
SELECT 'qala' AS system, COUNT(*) FROM qala.sdes AS_OF '2026-03-15T14:22:00Z'
WHERE cell[D₁, "status"].value = '"Active"';
```

---

## 15. The AI Computation Model

**Tier 1 — Synchronous Formula Attributes:** computed in-process at read time, always fresh:
```
budget_remaining = budget - budget_spent
completion_pct = completed_tasks / total_tasks * 100
sde_utilization = active_builds / total_sde_capacity * 100
```

**Tier 2 — Asynchronous AI/ML Attributes:** computed by registered `HypercubePlugin` implementations, written via WritebackService, cached with confidence score and staleness TTL:

```rust
pub trait HypercubePlugin: Send + Sync {
    fn plugin_id(&self) -> &str;
    fn name(&self) -> &str;
    fn attribute_key_defs(&self) -> Vec<AttributeKeyDef>;

    // Tier-2 AI computation
    fn compute_attribute(&self, key: &AttributeKey, ctx: &ComputeContext<'_>)
        -> HypergridResult<TypedAttrValue>;

    // Push-based invalidation: called when relevant cells are mutated
    fn on_cell_mutation(&self, cube_id: CubeId, coord: &DimCoordinate)
        -> Vec<AttributeKey>;
}
```

All AI-computed attributes use `write_permission: PermissionTier::System` (users cannot overwrite) and are stored as `TypedAttrValue::AiSignal` with confidence scores, model IDs, and computation timestamps.

---

## 16. HG-SPACE, HG-NS, and HG-ID

**HG-SPACE:** Spaces are named, governed, bounded operational contexts. Workspaces are the active, personalized working sessions within a Space. The Workspace persists the user's open Hypercubes/views, pinned rows, active DimSlice context (e.g., "freeze D₃=Q2-2026 across all open cubes"), and AI assistant state.

**HG-NS — Namespace System:**
```
kogi://personal/{worker_slug}/portfolio/{component_slug}/
kogi://cooperative/{coop_slug}/shared/{component_slug}/
ume://{org_id}/module/{module_name}/
ume://{org_id}/hr/employees/{employee_id}/
qala://{factory_slug}/sdes/{sde_id}/
qala://{factory_slug}/solutions/{solution_slug}/
qala://root/platform/{service_name}/    ← Root Factory entities
```

**HG-ID — Multi-Identity System:**

| Observer Type | Visible Rows | Visible Columns |
|--------------|-------------|-----------------|
| Public | visibility=Public rows | AttrVisibility=Public columns |
| Follower | Public + Follower rows | Public + Follower columns |
| Connection | Follower + Connection rows | Follower + Connection columns |
| Member | Connection + Space-scoped rows | Connection + Space columns |
| Editor | Member + non-private rows | Member + Editor columns |
| Admin | All rows in managed Space | All columns including governance |
| Owner | All rows including private | All columns including encrypted, AI-write-only |

---

## 17. Federation and Multi-Node Deployment

The federation sync protocol:

```
Node A (reconnecting after 2hr offline, has 47 local ops):
  1. Compare VectorClock to server Grid's VectorClock
  2. Server sends delta_since(node_A.cursor) via Kafka topic
  3. Node_A applies remote ops via apply_crdt_log():
     - LWW: highest VectorClock timestamp wins
     - OR-Set: all Adds survive; Removes by unique tag
     - Lattice: forward state always wins
  4. Node_A pushes its 47 local ops to server
  5. Both nodes emit CrdtMergeApplied to EventLog
  Result: eventual consistency in one round-trip; zero data loss
```

Federation topology supports up to 255 nodes per Grid (VectorClock size limit). CrdtLog hot in Redis; durable in PostgreSQL. Heartbeat every 30 seconds.

---

## 18. The Domain-Layer Pattern

All three domain systems follow the same four-layer architecture:

```
┌─────────────────────────────────────────────────────────────────┐
│  PRESENTATION LAYER                                               │
│  Domain UI · CLI · REST API · WebSocket · SDK                    │
├─────────────────────────────────────────────────────────────────┤
│  DOMAIN LOGIC LAYER                                               │
│  Domain entities · Business rules · Lifecycle state machines      │
│  PortfolioSystem (Kogi) · UmeKernel (Ume) · QalaOS (Qala)       │
├─────────────────────────────────────────────────────────────────┤
│  DOMAIN-HYPERGRID BRIDGE LAYER  (the DomainStore codec)          │
│  Entity ↔ HyperCell serialization · Schema registration           │
│  Attribute key CRDT assignment · Graph edge type mappings         │
│  ComponentStore (Kogi) · OrgModuleStore (Ume) · SolutionStore    │
├─────────────────────────────────────────────────────────────────┤
│  HYPERGRID SUBSTRATE LAYER                                        │
│  Grid · Hypercube · UCS · Hypergraph · VectorClock · CrdtLog     │
│  EventLog · HyperQL · HG-AI · HG-SPACE · HG-NS · HG-ID          │
└─────────────────────────────────────────────────────────────────┘
```

The bridge layer is the critical design point. Domain entity types (PortfolioComponent, OrgModule, Solution) are pure Rust structs with no storage concerns. The Hypercube stores them as HyperCells. The codec translates between the two — and it is the **only** place where domain vocabulary meets Hypergrid primitives.
# Part III — Kogi: The Portfolio as Root Hyperspreadsheet

## 19. The Portfolio as Root Domain

Kogi's root domain concept is the Portfolio. This is not a metaphor or a UI convention — it is the architectural foundation of the entire system.

> The Portfolio is the operating system of an independent worker's professional life. It is the single source of truth for every project, every gig, every asset, every benefit account, every contract, every grant, every relationship, and every financial outcome that constitutes that worker's economic and professional existence. Every entity in a worker's life is a row in one living portfolio spreadsheet.

The Kogi Platform is organized entirely around making that portfolio spreadsheet as powerful, as intelligent, and as connected as possible. Kogi is not a project management tool that happens to call its containers "portfolios." Kogi is a **portfolio operating system** — the Portfolio System is the root hyperspreadsheet, and every Kogi subsystem exists to enrich, extend, or connect the rows in that spreadsheet.

The **KIMDSS** (Kogi Interconnected Master Distributed Spreadsheet System) is the formal name for the complete Kogi data substrate: the Portfolio System Hypercube, its 35 derived views, the KLNK inter-portfolio graph, the kogi-engine AI, the Oba AI assistant, the KPID multi-identity system, and all federation connections.

---

## 20. The Portfolio System: KIMDSS Architecture

Every component of the Kogi KIMDSS maps directly onto Hypergrid primitives:

| Kogi Component | Hypergrid Primitive |
|----------------|---------------------|
| Portfolio (root entity) | HyperRow in `kogi.portfolio.components` (D₁=PortfolioId) |
| PortfolioComponent | HyperRow in `kogi.portfolio.components` — every item and container |
| 35 Sheets | HypercubeViews on `kogi.portfolio.components` — DimSlice + visible columns + render mode |
| KPI time-series | HyperRows in `kogi.portfolio.kpis` (N=3: Entity × Metric × Period) |
| Financial records | HyperRows in `kogi.portfolio.finances` (N=3: Account × Field × Period) |
| KLNK graph | Hypergraph edges — typed CrossGridLinks + ShadowCells + MirrorAttributes |
| EventLog / Audit | Dual-write: domain EventLog + Hypergrid EventLog |
| CRDT sync | Hypergrid VectorClock + CrdtLog (per-attribute LWW/OR-Set/Lattice) |
| kogi-engine AI | 12 HypercubePlugin sub-engines → WritebackService → AiSignal cells |
| Oba AI | NL-to-HyperQL translation + HyperQL execution + UI annotation |
| KPID identities | HG-ID TenantPartition + VisibilityMask per identity |
| Spaces | HG-SPACE SpaceType taxonomy: Personal/Shared/Cooperative/Federation |

---

## 21. The PortfolioComponent as Universal Node

The PortfolioComponent is the universal node of the entire Kogi system — every row in every sheet, every space, every workspace is a PortfolioComponent. It is either an **Item** (leaf work entity) or a **Container** (organizing structure).

```rust
pub struct Component {
    pub metadata:  ComponentMetadata,
    pub data:      ComponentData,
    pub item_book: Option<ItemBookData>,  // rich per-item dossier
}

pub struct ComponentMetadata {
    pub id:             ComponentId,      // UUID — globally unique, immutable
    pub owners:         Vec<UserId>,
    pub tags:           HashSet<String>,
    pub policy_ids:     Vec<PolicyId>,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
    pub last_actor:     String,           // node_id:identity_tag for CRDT tiebreak
    pub vector_clock:   VectorClock,      // from hypergrid::crdt
    pub version:        VersionString,    // semver
    pub budget:         f64,
    pub budget_spent:   f64,
    pub resource_units: f64,
    pub namespace_path: Option<NamespacePath>,
    pub space_id:       Option<ComponentId>,
    pub identity_tags:  Vec<IdentityTag>,
}

pub struct ComponentData {
    pub category:     ComponentCategory,  // Item | Container
    pub name:         String,
    pub description:  String,
    pub status:       ComponentStatus,
    pub state:        ComponentState,
    pub visibility:   Visibility,
    pub children:     Vec<ComponentId>,
    pub parents:      Vec<ComponentId>,
    pub links:        Vec<ComponentId>,
    pub dependencies: Vec<ComponentId>,
    pub dependents:   Vec<ComponentId>,
    pub users:        ComponentUsers,
    pub analytics:    ComponentAnalytics,
    pub risks:        Vec<Risk>,
    pub hashtags:     HashSet<String>,
    pub topics:       HashSet<String>,
    pub toolbox_ids:  Vec<ToolBoxId>,     // TMS v2.1
    pub plugin_configs: HashMap<String, JsonValue>,
    pub space_context:  Option<SpaceContext>,
    pub link_metadata:  Option<LinkMetadata>,
}
```

The complete ItemType taxonomy:

| ItemType | Description | Typical Parent | Key Fields |
|----------|-------------|----------------|------------|
| Portfolio | The root container — the worker's master portfolio | Personal Space | mission, focus_areas, kpis, visibility |
| Program | Strategic initiative grouping multiple projects | Portfolio | mission, objectives, budget_total, kpi_targets |
| Project | Bounded effort with deliverables and timeline | Program | methodology, sprints, releases, budget, due_date |
| Task / Story / Epic | Atomic or composite unit of work | Project or Sprint | priority, story_points, assignee, acceptance_criteria |
| Artifact | Versioned output: document, design, code, report | Project or Resource | artifact_type, version, format, signed_by, hash |
| Resource | Reusable asset: person, tool, license, service | Program or Portfolio | resource_type, capacity, cost_rate, availability |
| Asset | Capital asset: IP, financial asset, digital asset | Portfolio | asset_type, valuation, acquisition_cost |
| Gig / Contract | Income-generating engagement | Program or Resource | platform, rate, currency, deliverables, client_id |
| Benefit Account | Coverage entitlement: health, insurance, retirement | Personal Space | benefit_type, provider, coverage_start, balance |
| Grant | Funding opportunity or awarded grant | Program | funder, amount, deadline, reporting_requirements |
| Metric | KPI measurement or analytical data point | Any container | metric_type, value, period, target, trend |
| GovernanceProposal | Formal governance action requiring member vote | Organization Space | proposal_type, votes, quorum, decision_timestamp |
| CrowdresourcingCampaign | Community contribution campaign | Community Space | goal, progress, contributors, reward_structure |
| MarketplaceListing | Published listing on Kogi Marketplace | Resource or Artifact | category, price, currency, sales_count, rating |

---

## 22. Kogi Hypercube Inventory

Kogi registers eight Hypercubes in its Grid:

| Cube Name | N | D₁ | D₂ | D₃ | Primary Use |
|-----------|---|----|----|-----|------------|
| `kogi.portfolio.components` | 2 | ComponentId | field_name | — | All PortfolioComponents — the root cube |
| `kogi.portfolio.kpis` | 3 | ComponentId | metric_name | TimeAxis (period) | health/risk/alignment scores over time |
| `kogi.portfolio.finances` | 3 | AccountId | field_name | TimeAxis (period) | Income, expenses, budget allocations |
| `kogi.portfolio.benefits` | 2 | BenefitId | field_name | — | Benefits coverage records |
| `kogi.portfolio.actions` | 2 | ActionId | field_name | — | ActionKind event records |
| `kogi.portfolio.approvals` | 2 | ApprovalId | field_name | — | ApprovalRequest governance records |
| `kogi.portfolio.links` | 2 | LinkEdgeId | field_name | — | KLNK InterPortfolioLink records |
| `kogi.portfolio.snapshots` | 2 | SnapshotId | field_name | — | Point-in-time portfolio snapshot records |

### `kogi.portfolio.components` Attribute Key Registry (complete)

| D₂ Key | TypedAttrValue | CRDT | Write Permission | Notes |
|--------|----------------|------|-----------------|-------|
| `name` | Text | LWW | Editor | Display name |
| `description` | Text | LWW | Editor | Rich text description |
| `status` | Json (Enum) | LWW → Lattice (v2) | Editor | Draft\|Active\|Paused\|Completed\|Archived |
| `state` | Json (Enum) | LWW | Manager | Initializing\|Running\|Blocked\|Failing |
| `visibility` | Json (Enum) | LWW | Owner | Private\|Protected\|Public\|Unlisted |
| `category` | Json | LWW | Owner | ComponentCategory discriminant (Item\|Container) |
| `payload` | Json | LWW | Editor | ItemPayload or ContainerPayload |
| `owners` | Json | OR-Set | Owner | Vec<UserId> — concurrent adds never lost |
| `tags` | Json | OR-Set | Contributor | HashSet<String> |
| `hashtags` | Json | OR-Set | Contributor | Social discovery tags |
| `topics` | Json | OR-Set | Contributor | Topic classification |
| `budget` | Number | LWW | Manager | Allocated budget |
| `budget_spent` | Number | PnCounter | Manager | Consumed budget — concurrent spends commute |
| `resource_units` | Number | LWW | Manager | Generic resource unit counter |
| `version` | Text | MaxRegister | Editor | Semver — always maximum seen |
| `version_history` | Json | LWW | System | Vec<VersionHistoryEntry> |
| `policy_ids` | Json | LWW | Admin | Attached governance policy IDs |
| `children` | Json | OR-Set | Manager | DAG child IDs — concurrent adds survive |
| `parents` | Json | OR-Set | Manager | DAG parent IDs |
| `dependencies` | Json | OR-Set | Editor | Dependency component IDs |
| `dependents` | Json | OR-Set | System | System-maintained inverse of dependencies |
| `links` | Json | OR-Set | Editor | Soft link component IDs |
| `toolbox_ids` | Json | OR-Set | Owner | TMS ToolBox IDs (v2.1) |
| `risks` | Json | LWW | Editor | Vec<Risk> |
| `users` | Json | LWW | Manager | ComponentUsers registry |
| `analytics` | Json | LWW | System | ComponentAnalytics: views, likes, engagement |
| `item_book` | Json | LWW | Editor | Option<ItemBookData> — rich dossier |
| `health_score` | Number | LWW (AI-write) | System | AI-computed; AiSignal confidence: 0.0–1.0 |
| `risk_score` | Number | LWW (AI-write) | System | AI-computed |
| `alignment_score` | Number | LWW (AI-write) | System | AI-computed |
| `anomaly_flag` | Json (AnomalySignal) | LWW (AI-write) | System | Anomaly detection output |
| `narrative_summary` | Text | LWW (AI-write) | System | AI-generated portfolio narrative |
| `income_projection_90d` | Number | LWW (AI-write) | System | AI 90-day income projection |
| `coverage_gap_flags` | Json | LWW (AI-write) | System | Benefits coverage gap detection |
| `created_at` | DateTime | LWW | System | |
| `updated_at` | DateTime | LWW | System | |
| `last_actor` | Text | LWW | System | Node ID of last writer (CRDT tiebreak) |
| `vector_clock` | Json | LWW | System | Causal clock snapshot |
| `namespace_path` | Text | LWW | System | kogi://{space}/{slug}/... |
| `space_id` | Json | LWW | Manager | Parent Space ID |
| `identity_tags` | Json | OR-Set | Owner | KPID partition tags for identity isolation |

---

## 23. The 35 Sheets as HypercubeViews

The 35 Kogi sheets are HypercubeViews — saved projections over `kogi.portfolio.components`. Each is defined by a DimSlice filter, visible D₂ keys, default sort/group, optional ComputedColumnSet, and a RenderMode. Navigating between sheets is instant — the same underlying HyperRow data is simply re-projected.

| Sheet ID | Sheet Name | DimSlice Filter | Render Mode | Key Use Case |
|----------|-----------|----------------|-------------|--------------|
| SHT-001 | Master Registry | All types | Grid2D | Root view; global search and bulk operations |
| SHT-002 | Portfolio Hierarchy | Portfolio, SubPortfolio, Program, Project | HierarchyTree | Structural tree with rollup metrics |
| SHT-003 | Programs | Program | Grid2D | Strategic initiative tracking and KPI alignment |
| SHT-004 | Projects | Project | Kanban / Grid2D | Project management with methodology and sprints |
| SHT-005 | Tasks & Backlog | Task, Story, Epic, Feature | Kanban / Grid2D | Work execution: backlog, sprint, kanban |
| SHT-006 | Resources | Resource | Grid2D | Human, machine, license, and service resources |
| SHT-007 | Assets | Asset | Grid2D | Capital assets, IP, financial assets |
| SHT-008 | Artifacts | Artifact | Grid2D | Versioned outputs: documents, designs, code |
| SHT-009 | Finances | All financial rows | Grid2D | Unified P&L: income, expenses, budget |
| SHT-010 | Budget Tracker | Program, Project, Gig, Contract | Grid2D | Budget allocation, spend, remaining |
| SHT-011 | Work & Gigs | Gig, Contract, Job, Task | Grid2D | All income-generating engagements |
| SHT-012 | Deliverables | Project, Artifact, Release | Grid2D | Output tracking: what was produced and when |
| SHT-013 | Timeline | All dated rows | Timeline | Temporal view with milestones |
| SHT-014 | Roadmap | Program, Project, Milestone | Gantt | Strategic roadmap for external sharing |
| SHT-015 | Portable Benefits | BenefitAccount | Grid2D | Benefits dashboard: balances, coverage gaps |
| SHT-016 | Grants & Microfinancing | Grant, Campaign (grant type) | Grid2D | Grant pipeline from discovery to disbursement |
| SHT-017 | Equity & Crowdfunding | Campaign, Investment | Grid2D | Fundraising, equity rounds, investment positions |
| SHT-018 | Group Economics | Shared Portfolio, Revenue Pool, Org | Grid2D | Cooperative and collective financial structures |
| SHT-019 | Organizations | Profile (org type) | Grid2D | All organizations the user belongs to |
| SHT-020 | Shared Portfolios | Portfolio (shared) | Grid2D | Co-owned and shared portfolio components |
| SHT-021 | Collaboration | All shared/contributed rows | Grid2D | Contribution tracking and attribution weights |
| SHT-022 | Risk Register | All rows with risk_flags | HeatMap | Unified risk dashboard |
| SHT-023 | Analytics Dashboard | All rows | Grid2D | Health scores, performance metrics, AI signals |
| SHT-024 | Feed & Community | Profile, Post, Artifact (public) | Grid2D | Community-facing content and activity |
| SHT-025 | Contacts & Network | Profile, Contact | Grid2D | CRM: clients, collaborators, investors |
| SHT-026 | Marketplace Listings | Asset, Resource, Gig (public) | Grid2D | Published marketplace items |
| SHT-027 | Exchange | Investment, Deal, Campaign, Asset | Grid2D | Exchange market activity |
| SHT-028 | Archive | All archived rows | Grid2D | Deep storage with full restore capability |
| SHT-029 | Event Log | EventLog entries (read-only) | Grid2D | Immutable audit trail |
| SHT-030 | Snapshots | Snapshot records | Grid2D | Point-in-time portfolio snapshots for restore |
| SHT-031 | Link Network | LinkEdge records | NetworkGraph | All inter-portfolio links; KLNK graph view |
| SHT-032 | Identity & Profiles | ProfilePartition records | Grid2D | Multi-identity management |
| SHT-033 | Benefits Deep Dive | BenefitAccount + Gig contribution | Grid2D | Gig platform contribution tracking |
| SHT-034 | Custom Sheet | User-defined | User-defined | Ad-hoc views for specific workflows |
| SHT-035 | Template Library | ItemBook:Template rows | Grid2D | Reusable templates from KOGI-APPSTORE |

---

## 24. The KLNK Inter-Portfolio Graph

KLNK (the Kogi Link Network) is the inter-portfolio graph system. Every KLNK connection is a typed CrossGridLink edge in the Hypergraph with corresponding ShadowCells.

| LinkEdgeType | Direction | Consent | Creates ShadowRow? | Description |
|-------------|-----------|---------|-------------------|-------------|
| Collaborates | Undirected | Yes — mutual | Yes — both parties | Active collaboration on shared component |
| InvestedIn | Directed (A→B) | Yes — from B | Yes — investor sees investee's approved columns | Economic investment or patronage |
| Employs | Directed (org→worker) | Yes — from worker | Yes — org sees worker's work record columns | Organization employs/contracts worker |
| Contracted | Directed (A→B) | Yes — from B | Yes — both see agreed delivery columns | Formal contract between two parties |
| Follows | Directed (A→B) | No | No — read-only via public visibility | Subscribe to public updates |
| Endorses | Directed (A→B) | No | No | Endorse a skill or work product |
| SharedPortfolio | Directed (owner→contributor) | Yes — from contributor | Yes — contributor sees permitted columns | Co-owned portfolio access |
| MarketplaceTransaction | Directed (buyer→) | Via transaction | Yes — both see transaction record | Completed marketplace transaction |
| OrgMembership | Directed (org→member) | Yes — from member | Yes — member sees org's public shared portfolio | Member belongs to organization/coop |
| DependsOn | Directed (A→B) | No | Yes — A sees B's delivery status columns | A's work depends on B's output |
| ResourceShares | Directed (owner→receiver) | Yes — from receiver | Yes — receiver sees resource access columns | Share a resource, tool, or template |
| FederationPeer | Undirected | Grid-level trust | Yes — full CrdtLog sync | Two PortfolioSystems in federation |

The ShadowCell protocol in detail:

```
Phase 1: Link Request
  Worker A creates CrossGridLink to Worker B.
  ConsentRequest sent to Worker B. ConsentStatus: Pending.

Phase 2: Consent Configuration
  Worker B configures:
    mirrored_attrs: Vec<AttributeKey>   -- visible in ShadowCell in A's grid
    writeback_attrs: Vec<AttributeKey>  -- A may write back these attributes

Phase 3: ShadowCell Provisioning
  ShadowCell created in Grid A:
    source_grid_id, source_d1_key, source_edge_id
    synced_values: current snapshot of mirrored_attrs

Phase 4: Real-Time Sync
  When B writes to any mirrored_attr:
    EventLog → Kafka shadow.sync.{grid_id} topic
    Federation Coordinator routes delta to Grid A
    Grid A applies delta to ShadowCell (LWW merge)

Phase 5: Write-Back (if configured)
  A writes to writeback_attr on ShadowCell:
    WritebackService validates PermissionTier
    Write applied to Grid B's actual HyperCell
    Attributed to A's actor ID in Grid B's EventLog
```

KLNK visualization modes for SHT-031:

| View Mode | Description | Key Use Case |
|-----------|-------------|--------------|
| Link Forest View | All LinkTrees rooted at all user identities, multi-root | Full connectivity overview; bridge nodes |
| Component Tree View | Single LinkTree from a specific portfolio component | Component's full network context; due diligence |
| Dependency Tree View | DependsOn edges only; critical path rendered | Cross-portfolio critical path analysis |
| Investment Graph | InvestedIn + Contracted edges — financial network | Investor portfolio visualization |
| Organization Tree | OrgMembership edges from root org down through members | Org structure visualization |
| Collaboration Web | Collaborates edges between users and shared components | Contribution attribution |
| Federation Graph | FederationPeer edges between PortfolioSystem nodes | Platform infrastructure; CRDT sync health |
| Spreadsheet Grid View | KLNK as SHT-031: each row = LinkEdge | Bulk link management; data export |

---

## 25. Kogi Multi-Identity System (KPID)

A Sovereign Entity is the real-world person, organization, or collective that owns a root PortfolioSystem instance. A Sovereign Entity can have multiple Identities (multiple @handles), each with its own TenantPartition of the data and configurable VisibilityMask.

```
SovereignEntity
├── Identity: @alice-professional
│   ├── Profile: CV (public professional portfolio view)
│   ├── Profile: Client-facing (selected work visible)
│   └── Accounts: [platform_creds]
├── Identity: @alice-personal
│   ├── Profile: Personal (private)
│   └── Accounts: [personal_creds]
└── Identity: @alice-coop-member
    ├── Profile: Cooperative (shared portfolio view)
    └── Accounts: [coop_creds]
```

The VisibilityMask is a set of DimSlice predicates applied at query time before any HyperCell is returned:

```sql
-- Public observer VisibilityMask
WHERE cell[D₁, "visibility"].value IN ('"Public"')
  AND attr.visibility = 'Public'

-- Member observer in Space S VisibilityMask
WHERE (cell[D₁, "visibility"].value IN ('"Public"', '"Protected"', '"Member"')
    OR cell[D₁, "space_id"].value = '"{S}"')
  AND (attr.visibility IN ('Public', 'Member')
    OR (attr.visibility = 'Space' AND space_id = '{S}'))
```

The SplitPolicy governs cross-partition data access. Owner-level column encryption ensures the platform operator cannot read private encrypted data — KMS-backed AES-256-GCM on PII fields.

---

## 26. kogi-engine and the Oba AI

**kogi-engine** is the Scala 3 analytics and intelligence engine consuming all portfolio events from the Kogi EventLog via Kafka. It operates 12 sub-engines as HypercubePlugin implementations:

| Sub-Engine | Output Attribute | Target Cube | Input Attributes | Trigger |
|------------|-----------------|-------------|-----------------|---------|
| KogiHealthScoreEngine | `health_score` | `kogi.portfolio.kpis` | progress_pct, budget_utilization, schedule_variance, risk_flags, velocity | Any component mutation |
| KogiRiskEngine | `risk_score` | `kogi.portfolio.kpis` | risk_flags, critical_risk_count, overdue_milestones, budget_burn_rate | Risk record change or deadline approach |
| KogiAlignmentEngine | `alignment_score` | `kogi.portfolio.kpis` | okr_ids, kpi_targets, milestone_completion_rate | Program or strategy mutation |
| KogiCollaborationEngine | `collaboration_score` | `kogi.portfolio.kpis` | contributor_count, contribution_distribution, engagement_rate | Contribution event |
| KogiMaturityEngine | `maturity_score` | `kogi.portfolio.components` | artifact_count, version_history, review_completion_rate | Artifact change |
| KogiAnomalyEngine | `anomaly_flag` | `kogi.portfolio.components` | All numeric metrics (rolling baseline) | Metric deviation >2σ |
| KogiMatchEngine | `match_score` | `kogi.portfolio.links` | skills, requirements, availability, portfolio_quality | Skill or requirement update |
| KogiIncomeProjectionEngine | `income_projection_90d` | `kogi.portfolio.finances` | gig_history, contract_pipeline, market_rates | Gig or contract change |
| KogiBudgetEngine | `budget_burn_rate` | `kogi.portfolio.finances` | budget_spent, budget_allocated, historical_spend_rate | Financial transaction |
| KogiNarrativeEngine | `narrative_summary` | `kogi.portfolio.components` | milestone_completions, artifact_outputs, key_metrics | Significant milestone completion |
| KogiScheduleRiskEngine | `predicted_completion_date` | `kogi.portfolio.components` | velocity, remaining_work, schedule_variance | Sprint or task update |
| KogiCoverageGapEngine | `coverage_gap_flags` | `kogi.portfolio.benefits` | benefit_accounts, income_replacement_ratio, pto_remaining | Benefit or gig change |

**Oba** is the Kogi platform AI assistant — the portfolio's chief of staff. It reads every row, every column, and every engine signal, surfacing proactive context-aware intelligence directly in the spreadsheet UI.

| Oba Mode | Trigger | Example |
|----------|---------|---------|
| **Reactive** | User types natural language query | "Show me everything at risk this week" → HyperQL with risk_score > 60 OR due_date < TODAY+3 |
| **Proactive** | EventLog mutation triggers annotation rule | "This project is 14 days behind — adjust due date or reduce scope?" |
| **Column Completion** | Required field empty on new row | "Due date missing — suggest April 30 based on similar projects?" |
| **Smart Filters** | Question implies a filter | "Show me what needs attention today" → ViewFilter: risk or deadline predicate |
| **Formula Suggestions** | User adds a FormulaColumn | "You're tracking revenue and expenses — add a profit margin % column?" |
| **Batch Actions** | User states a bulk intent | "Archive completed projects older than 90 days" → BatchArchiveAction for confirmation |
| **Health Briefing** | Daily scheduled run | Top health risks, upcoming deadlines, pending approvals, contribution opportunities |
| **Narrative Generation** | User requests portfolio narrative | "Write a highlight for my top 3 projects this quarter" → structured narrative from row data |
| **Autonomous (approval)** | User delegates a task | Oba prepares action plan; executes only after explicit user confirmation |

---

## 27. Kogi Spaces and Workspaces

| Kogi Context | SpaceType | Slug Pattern | Auto-Provisioned Cubes | Governance |
|-------------|-----------|-------------|------------------------|------------|
| Personal Portfolio | Personal | `kogi://personal/{user_slug}/` | components, kpis, benefits | Owner only |
| Shared Portfolio | Team | `kogi://shared/{portfolio_slug}/` | components, contribution_ledger, kpis | Manager + member approval |
| Cooperative | Community | `kogi://cooperative/{coop_slug}/` | components, governance, kpis, finances | Member vote + quorum |
| Federation Portfolio | Federation | `kogi://federation/{fed_slug}/` | components, cross-portfolio, governance | Representative governance |
| Crowdresourcing Campaign | Project | `kogi://campaign/{campaign_slug}/` | components, contribution_ledger | Steward + campaign governance |
| Organization Space | Organization | `kogi://org/{org_slug}/` | components, members, governance | Admin-controlled |

Each Workspace stores the user's active session state: open sheets, pinned rows, active DimSlice context (e.g., "freeze D₃=Q2-2026 across all sheets"), and Oba AI conversation history. Workspace state persists in `system.workspaces` and restores on return.

---

## 28. Kogi Cross-System Integration Points

| Integration | Edge Type | Consent | What is Mirrored |
|-------------|-----------|---------|-----------------|
| Portfolio item → Qala solution | CrossGridLink | Yes — worker grants factory access | solution lifecycle_state, version, release_date |
| Ume employment → Kogi work record | Employs | Yes — worker grants org access | Work record columns: tasks, gigs, contributions |
| Kogi cooperative → Ume organization | OrgMembership | Yes — worker grants org access | Member's public portfolio columns |
| Kogi gig → Qala SDE contribution | Collaborates | Via contribution attribution | SDE contribution record, attribution weight |
| Kogi asset → Qala artifact | Contains | No — same owner | Artifact hash, version, deployment status |
| Cross-worker shared portfolio | SharedPortfolio | Yes — from contributor | Permitted columns of shared portfolio |
# Part IV — Ume: The Organization as Root Hyperspreadsheet

## 29. The Organization as Root Domain

Ume's root domain concept is the Organization.

> The Organization is the operating system of a business's operational world. It is the single source of truth for every module, every employee, every legal entity, every campaign, every financial account, every risk record, every OKR, every workflow, and every governance decision. Every entity in an organization's world is a row in one living organization spreadsheet.

Ume maps all 42 organization modules to domain-specific Hypercubes within a single Organization Grid. The UmeKernel — which supervises all 42 modules, manages the event bus, enforces RBAC, and provides shared services — is itself mapped onto Hypergrid infrastructure. The Organization System (Ume OS) is the root hyperspreadsheet.

---

## 30. The Organization System: Architecture

The UmeKernel is the composition root for all 42 domain subsystems:

```
UmeKernel {
    ModuleRegistry       → ume.kernel.modules Hypercube
    InMemoryEventBus     → Hypergrid EventLog + CrdtLog (Kafka)
    RbacEngine           → Hypergrid PermissionTier + AttrVisibility
    SupervisorEngine     → Hypergrid GovernanceEngine + PolicyEngine
    OrchestratorEngine   → HyperQL + computed attributes
    TemplateEngine       → ume.templates.library Hypercube
    SchemaRegistry       → Hypergrid AttributeKeyRegistry per Hypercube
    MetricsCollector     → ume.analytics.kpis (D₃=TimeAxis)
    LogAuditManager      → Hypergrid EventLog (append-only, AS_OF)
    StorageManager       → Hypergrid UniversalCellStore
    MemoryManager        → hot-path cell cache (Redis)
    SearchService        → HyperQL + Meilisearch
    BackupManager        → Hypergrid Snapshot + EventLog replay
    NetworkManager       → CrdtLog federation over Kafka
}
```

Every kernel service that previously required bespoke in-memory data structures is replaced by Hypergrid infrastructure. The Organization System inherits all data infrastructure from the substrate; Ume engineers focus purely on organizational domain logic.

---

## 31. Kernel-to-Hypergrid Mapping

| Ume Kernel Service | Hypergrid Equivalent | Key Detail |
|-------------------|---------------------|------------|
| `ModuleRegistry` | `ume.kernel.modules` Hypercube | 42 module records as HyperRows; versioned, CRDT-synced |
| `InMemoryEventBus` | Hypergrid EventLog + CrdtLog | Persistent, time-travelable; dual-write preserves domain-typed querying |
| `RbacEngine` | PermissionTier + AttrVisibility | `finance.invoice.approve` → PermissionTier::Manager on invoice approval field |
| `SupervisorEngine` | GovernanceEngine + PolicyEngine | restart_count as GrowOnlyCounter CRDT; restart/backoff as governance policy |
| `OrchestratorEngine` | HyperQL + computed attributes | Workflow execution state in `ume.process.workflows` cube |
| `TemplateEngine` | `ume.templates.library` Hypercube | Versioned template storage with D₃=TimeAxis for template history |
| `SchemaRegistry` | AttributeKeyRegistry | Per-Hypercube schema contracts; type-safe attribute key registration |
| `MetricsCollector` | `ume.analytics.kpis` (D₃=TimeAxis) | Time-series metric storage: every KPI has a temporal dimension |
| `LogAuditManager` | Hypergrid EventLog | Immutable audit trail; AS_OF for compliance |
| `StorageManager` | Hypergrid UniversalCellStore | All entity storage partitioned by module cube |
| `SearchService` | HyperQL + Meilisearch | Cross-cube entity search; NL-to-HyperQL |
| `BackupManager` | Hypergrid Snapshot + EventLog replay | Point-in-time state capture; restore from EventLog |

The Ume `ExecutorPool` — named thread pools per domain — maps to Hypergrid Spaces:

| Ume ExecutorId | Hypergrid Space | Cubes Managed |
|----------------|-----------------|---------------|
| `kernel.critical` | `ume://system/kernel/critical/` | `ume.kernel.modules` |
| `kernel.events` | `ume://system/kernel/events/` | EventLog streams |
| `kernel.audit` | `ume://system/kernel/audit/` | Immutable audit records |
| `org.finance` | `ume://org/finance/` | `ume.finance.*` |
| `org.legal` | `ume://org/legal/` | `ume.chombo.*` |
| `org.marketing` | `ume://org/marketing/` | `ume.soko.*` |
| `org.hr` | `ume://org/hr/` | `ume.hr.*` |
| `org.analytics` | `ume://org/analytics/` | `ume.analytics.*` |
| `org.ops` | `ume://org/ops/` | `ume.ops.*`, `ume.supply_chain.*` |

---

## 32. The 42 Organization Module Hypercubes

### Module Registry: `ume.kernel.modules` (N=2)

| D₂ Key | TypedAttrValue | CRDT | Permission | Notes |
|--------|----------------|------|------------|-------|
| `module_id` | Text | LWW | System | Canonical module identifier |
| `name` | Text | LWW | Admin | Display name |
| `domain_area` | Json | LWW | Admin | DomainArea enum |
| `version` | Text | MaxRegister | System | Always the maximum version seen |
| `lifecycle_state` | Json | Lattice | System | Registered→Starting→Running→Degraded→Stopped |
| `status` | Json | LWW | Admin | Active\|Suspended\|Disabled |
| `dependencies` | Json | OR-Set | Admin | Vec<ModuleId> — concurrent adds survive |
| `executor_id` | Text | LWW | System | Assigned thread pool ID |
| `restart_count` | Number | GrowOnlyCounter | System | Only increases |
| `last_error` | Json | LWW | Admin | Option<KernelError> |
| `health_score` | Number | LWW (AI-write) | System | AI-computed module health 0–100 |
| `evaluation_cache` | Json | LWW | System | Cached evaluation output |
| `config` | Json | LWW | Admin | Module configuration blob |
| `metrics` | Json | LWW | System | MetricPoint observations snapshot |

### The 42 Module Hypercubes (complete)

| # | Module Name | Primary Hypercube | Dimensions | Key D₂ Fields |
|---|-------------|------------------|--------------------|---------------|
| 01 | Organization Administration | `ume.admin.org_units` | D₃=TimeAxis | name, type, parent_unit, head_count, budget |
| 02 | Organization Analytics | `ume.analytics.kpis` | D₃=TimeAxis, D₄=DomainAxis | metric_name, value, target, trend, period |
| 03 | Backup & Recovery | `ume.backup.jobs` | D₃=CategoryAxis | job_id, scope, status, schedule, last_run |
| 04 | Board Management | `ume.board.meetings` | D₃=TimeAxis | title, date, quorum, resolution_count, status |
| 05 | Business Development | `ume.bizdev.opportunities` | D₃=CategoryAxis | title, stage, value, probability, close_date |
| 06 | Enterprise CMS | `ume.cms.content` | D₃=TimeAxis | title, content_type, status, author, tags |
| 07 | Communications | `ume.comms.channels` | D₃=CategoryAxis | channel_name, type, members, status |
| 08 | CRM | `ume.crm.contacts` | D₃=CategoryAxis | name, type, stage, owner, last_contact, value |
| 09 | Design System | `ume.design.tokens` | D₃=TimeAxis | token_name, value, category, version, approved_by |
| 10 | Engineering & Technology | `ume.eng.projects` | D₃=TimeAxis | name, stack, status, team, velocity, sprint_count |
| 11 | ESG / CSR / Sustainability | `ume.esg.metrics` | D₃=TimeAxis, D₄=CategoryAxis | metric_name, value, category, period, verified_by |
| 12 | Enterprise Engineering Admin | `ume.enterprise.systems` | — | system_name, type, status, owner, sla_target |
| 13 | Legal Entity (Chombo) | `ume.chombo.entities` | D₃=CategoryAxis (jurisdiction) | entity_name, jurisdiction, type, status, compliance_score |
| 14 | Finance & Accounting | `ume.finance.accounts` | D₃=TimeAxis | account_name, type, balance, currency, period_close |
| 15 | GRC | `ume.grc.risks` | D₃=TimeAxis | title, category, severity, probability, status |
| 16 | Human Resources | `ume.hr.employees` | D₃=TimeAxis | name, role, department, status, hire_date, salary |
| 17 | Investment Management | `ume.investment.portfolio` | D₃=TimeAxis, D₄=CategoryAxis | asset_name, type, value, allocation_pct, return_pct |
| 18 | IT & Asset Management | `ume.it.assets` | D₃=CategoryAxis | asset_name, type, status, owner, cost, depreciation |
| 19 | Enterprise Knowledge | `ume.knowledge.articles` | D₃=TimeAxis | title, category, status, author, views |
| 20 | Learning & Development | `ume.learning.programs` | D₃=TimeAxis | title, type, status, enrolled, completed |
| 21 | Management & Strategy | `ume.strategy.okrs` | D₃=TimeAxis | objective, key_result, progress, owner, quarter |
| 22 | Marketing (Soko) | `ume.soko.campaigns` | D₃=TimeAxis, D₄=CategoryAxis | name, type, status, budget, ctr, conversion_rate |
| 23 | Master Data Management | `ume.mdm.entities` | D₃=CategoryAxis | entity_name, type, golden_record_id, source, quality_score |
| 24 | Office & Facility | `ume.facilities.sites` | D₃=CategoryAxis | site_name, type, capacity, status, cost_per_month |
| 25 | Operations Management | `ume.ops.processes` | D₃=TimeAxis | name, type, status, sla_target, sla_actual |
| 26 | Portal / Hub / Dashboard | `ume.portal.dashboards` | — | title, owner, widgets, layout, visibility |
| 27 | Portfolio & Program | `ume.portfolio.programs` | D₃=TimeAxis | name, status, budget, kpi_count, health, sponsor |
| 28 | PR & Branding | `ume.branding.assets` | D₃=TimeAxis | name, type, status, campaign_id, version |
| 29 | Process, Orchestration & Workflow | `ume.process.workflows` | D₃=TimeAxis | name, type, status, trigger, step_count, avg_duration |
| 30 | Product, Services & Solutions | `ume.product.catalog` | D₃=TimeAxis, D₄=CategoryAxis | name, type, status, sku, pricing_model, revenue_ytd |
| 31 | Production, Manufacturing | `ume.production.batches` | D₃=TimeAxis | batch_id, product, status, quantity, date, defect_rate |
| 32 | Requirements Management | `ume.requirements.specs` | D₃=CategoryAxis | title, type, priority, status, linked_solution_id |
| 33 | Enterprise Risk Management | `ume.risk.register` | D₃=TimeAxis | title, category, severity, probability, mitigation_status |
| 34 | Sales Management | `ume.sales.opportunities` | D₃=TimeAxis | title, stage, value, owner, close_date, win_probability |
| 35 | Enterprise Schedule | `ume.schedule.events` | D₃=TimeAxis | title, type, start_at, end_at, attendees, location |
| 36 | Security, Privacy & Protection | `ume.security.incidents` | D₃=TimeAxis | title, severity, status, assignee, cve_id |
| 37 | Logistics, Supply Chain & WMS | `ume.supply_chain.orders` | D₃=TimeAxis | order_id, supplier, status, total, eta, tracking_id |
| 38 | Team & Cooperative Management | `ume.teams.groups` | — | name, type, members, owner, mission, governance_model |
| 39 | Organization Templating | `ume.templates.library` | D₃=TimeAxis | name, domain, version, status, uses_count, author |
| 40 | Enterprise Work Management | `ume.work.items` | D₃=TimeAxis | title, type, priority, status, assignee, sprint_id |
| 41 | Custom UME Modules | `ume.custom.{vendor}.*` | Vendor-defined | Platform-extensible module namespace |
| 42 | Custom Org Modules | `ume.org.{slug}.*` | Org-defined | Organization-extensible module namespace |

---

## 33. Chombo: Legal Entity on Hypergrid (N=3)

Chombo uses a 3-dimensional Hypercube so that a single multinational legal entity can have jurisdiction-sliced data in the same cube, and cross-jurisdictional compliance reports are native DimFold operations:

```
ume.chombo.entities (N=3)
D₁ = EntityAxis (Uuid)           ← LegalEntityId
D₂ = PropertyAxis (String)       ← entity field name
D₃ = CategoryAxis (String)       ← jurisdiction ("US-DE", "UK", "EU", "ZA", "NG", ...)

Example cells:
  cell[(entity_id, "compliance_score", "US-DE")].value = Number(94.2)
  cell[(entity_id, "open_findings",    "UK")].value    = Number(3)
  cell[(entity_id, "last_evaluated",   "EU")].value    = DateTime(...)
  cell[(entity_id, "gmp_status",       "EU")].value    = Text("Compliant")
  cell[(entity_id, "filing_deadline",  "NG")].value    = Date("2026-06-30")

Cross-jurisdictional aggregate:
SELECT D₃.jurisdiction,
       AVG(cell[D₁, "compliance_score", D₃].value) AS avg_compliance,
       SUM(cell[D₁, "open_findings", D₃].value) AS total_open_findings
FROM ume.chombo.entities
WHERE D₃.jurisdiction IN ("US-DE", "UK", "EU", "ZA")
GROUP BY D₃.jurisdiction
ORDER BY avg_compliance ASC;
```

Each of the 45 Chombo policy packs is a HyperRow in `ume.chombo.policies` with AI-computed `compliance_prediction` as a Tier-2 AiSignal cell.

---

## 34. Soko: Marketing Intelligence on Hypergrid (N=4)

Soko uses a 4-dimensional Hypercube supporting multi-segment, multi-period analytics natively:

```
ume.soko.campaigns (N=4)
D₁ = EntityAxis (Uuid)           ← CampaignId
D₂ = PropertyAxis (String)       ← metric/field name
D₃ = TimeAxis (Timestamp)        ← time period
D₄ = CategoryAxis (String)       ← audience_segment

Single point lookup — CTR, enterprise segment, Q2 2026:
  cell[(campaign_id, "ctr", "2026-Q2", "enterprise")].value = Number(0.0342)

DimExpand pivot — all segments for a campaign in Q2:
SELECT D₁.campaign_id, EXPAND D₄ AS COLUMNS(AVG(cell.value))
FROM ume.soko.campaigns
WHERE D₂.metric_name = 'ctr' AND D₃.period = '2026-Q2';
-- Result: campaign_id | enterprise_ctr | smb_ctr | consumer_ctr | developer_ctr

Full multi-period multi-segment pivot:
SELECT D₁.campaign_id,
       FOLD D₃ WITH AVG(cell.value) LAST 4 QUARTERS,
       EXPAND D₄ AS COLUMNS(avg_ctr)
FROM ume.soko.campaigns WHERE D₂.metric_name = 'ctr'
GROUP BY D₁.campaign_id, D₄.segment;
```

The 70 Soko strategy packs are HypercubePlugin implementations — each subscribes to `on_cell_mutation` on campaign target/budget fields and schedules recomputation of strategy evaluation scores.

---

## 35. Ume RBAC on Hypergrid

Ume's permission namespace (`{domain}.{resource}.{action}`) maps to per-attribute PermissionTier:

| Ume Permission | Attribute Key | Write Permission |
|---------------|--------------|-----------------|
| `finance.journal.write` | Journal entry fields in `ume.finance.accounts` | PermissionTier::Editor (3) |
| `finance.invoice.approve` | `approval_status` in invoice records | PermissionTier::Manager (4) |
| `hr.employee.write` | Employee fields in `ume.hr.employees` | PermissionTier::Manager (4) |
| `hr.payroll.run` | `payroll_execution_trigger` | PermissionTier::Owner (5) |
| `grc.risk.evaluate` | Risk evaluation fields in `ume.grc.risks` | PermissionTier::Editor (3) |
| `chombo.entity.evaluate` | Compliance score fields | PermissionTier::Editor (3) |
| `board.resolution.write` | Resolution fields in `ume.board.meetings` | PermissionTier::Manager (4) |
| `admin.policy.publish` | Policy records in `ume.admin.*` | PermissionTier::Admin (6) |

AttrVisibility enforces read access: `finance.journal` records use `AttrVisibility::Tenant` (org-members only), salary fields in HR use `AttrVisibility::Owner` (employee + HR admin only).

---

## 36. Ume AI Engine

| AI Capability | Plugin | Output Attribute | Target Cube | Trigger |
|--------------|--------|-----------------|-------------|---------|
| Module Health Scoring | `UmeModuleHealthEngine` | `health_score` | `ume.kernel.modules` | Any module lifecycle event |
| Compliance Prediction | `UmeChomboAI` | `compliance_prediction` | `ume.chombo.entities` | Policy pack update or approaching deadline |
| OKR Progress Prediction | `UmeStrategyAI` | `okr_completion_probability` | `ume.strategy.okrs` | KR progress update |
| Attrition Risk Scoring | `UmeHRAI` | `attrition_risk_score` | `ume.hr.employees` | Performance review or engagement event |
| Revenue Forecast | `UmeFinanceAI` | `revenue_forecast_q` | `ume.sales.opportunities` | Pipeline change |
| Risk Correlation | `UmeRiskAI` | `correlated_risk_ids` | `ume.grc.risks` | Risk record creation |
| Campaign Optimization | `UmeSokoAI` | `optimization_suggestions` | `ume.soko.campaigns` | Campaign metric update |
| Anomaly Detection | `UmeAnomalyEngine` | `anomaly_flag` | All module cubes | >2σ deviation in any numeric metric |

---

## 37. Ume Cross-System Integration Points

| Integration | Edge Type | Mirrored Fields | Consent |
|-------------|-----------|-----------------|---------|
| Ume employee → Kogi worker portfolio | Employs | Work record columns: tasks, gigs, contributions | Yes — from worker |
| Ume org → Kogi cooperative member | OrgMembership | Org public shared portfolio columns | Yes — from member |
| Ume product catalog → Qala solution | Produces | lifecycle_state, version, quality_score | No — same org federation |
| Ume OKRs → Qala SDE goals | Association | OKR progress, key_result targets | No — organizational relationship |
| Ume requirements → Qala solutions | Association | Solution status, linked_solution_id | Both directions |

---

# Part V — Qala: The Solution as Root Hyperspreadsheet

## 38. The Solution as Root Domain and Universal Category

Qala's root domain concept is the Solution — and it is the **universal category** from which all other entities derive their meaning.

> The Solution is the operating system of a solution factory's production world. It is the single source of truth for every SDE, every artifact, every CCR, every release, every toolchain, every AI signal. Everything in a factory's world is a row in one living solution spreadsheet. The factory itself is a Solution. Kogi is a Solution. Ume is a Solution. Qala is a Solution.

The canonical six Solution types cover the entire space of things humans and organizations build:

| Solution Type | Description | Apapo Examples |
|---------------|-------------|----------------|
| **Application** | Software application with defined processes and user interactions | Web apps, mobile apps, APIs, firmware |
| **System** | Coordinated collection of applications serving a unified purpose | Banking platform, ERP system, Kogi KIMDSS, Ume OS |
| **Good** | Tangible, physical, or digital deliverable produced by the factory | Physical product, pharmaceutical formulation, hardware |
| **Product** | Commercially packaged, versioned, distributable artifact | SaaS product, packaged software, drug product |
| **Service** | Ongoing capability delivered to consumers continuously | Managed IT service, consulting service |
| **Platform** | Foundation on which other Solutions are built and operated | Hypergrid NDSS, Apapo, Kogi, Ume, Qala |

---

## 39. The Solution System Architecture

Qala's six architectural planes map to Hypergrid:

| Qala Plane | Language | Hypergrid Mapping | Responsibility |
|------------|----------|-------------------|----------------|
| **Kernel Plane** | Rust | `hypergrid::core::Grid` | Platform integrity; service lifecycle; event aggregation |
| **Control Plane** | Go | Domain-layer systems on Hypergrid | Factory/SDE lifecycle; CCR workflow; release pipeline |
| **Execution Plane** | Go + Scala | HyperQL engine + `qala.cicd.*` cubes | Build/test execution; pipeline orchestration |
| **Intelligence Plane** | Rust | HG-AI + `AttrComputation::AiEngine` | 10 AI Agent capabilities; SEM writes to `qala.security.*` |
| **Platform Operations** | Go + Terraform | HG-OPS: Snapshot, Federation, deployment | Infrastructure; federation health; disaster recovery |
| **Domain Extension Plane** | Go + YAML DSL | HypercubePlugin trait — Domain Packs | Domain Pack ecosystem; regulatory extensions |

---

## 40. Solutions, Factories, SDEs as HyperRows

| Qala Entity | Hypercube | Key HyperRow Fields |
|-------------|-----------|---------------------|
| Solution Factory | `qala.factories` | factory_id, name, domain_packs, governance_config, status, root_factory_flag |
| SDE | `qala.sdes` | sde_id, factory_id, name, status, team_members, toolchain_id, hermetic_manifest, drift_status |
| Solution | `qala.solutions` | solution_id, sde_id, type, version, lifecycle_state, quality_score, is_factory |
| CCR | `qala.ccrs` | ccr_id, solution_id, change_type, risk_level, status, approvers, decision_timestamp |
| Release | `qala.releases` | release_id, solution_id, version, release_type, distribution_channels, signed_by |
| Artifact | `qala.artifacts` | artifact_id, solution_id, artifact_type, format, hash, created_at, signed_by |
| Toolchain | `qala.toolchains` | toolchain_id, name, tools, versions, environment_type, hermetic_digest, validated_by |
| AI Recommendation | `qala.ai_recommendations` | rec_id, solution_id, recommendation_type, confidence, action_proposed, status |
| CI/CD Pipeline | `qala.cicd.pipelines` | pipeline_id, sde_id, trigger, status, duration_ms, test_pass_rate, deploy_target |
| Security Event | `qala.security.events` | event_id, solution_id, threat_type, severity, status, cve_id, remediation_status |
| Governance Record | `qala.governance.records` | record_id, solution_id, pack_id, evaluation_result, open_findings |
| Quality Metric | `qala.solutions.metrics` | (solution_id, metric_name, period, category) → value |

---

## 41. Qala Hypercube Inventory

| Cube Name | N | Dimensions | Notes |
|-----------|---|------------|-------|
| `qala.factories` | 2 | Entity, Property | root_factory_flag identifies the Qala Root Factory |
| `qala.sdes` | 3 | Entity, Property, Time | D₃=TimeAxis enables full AS_OF rollback |
| `qala.solutions` | 2 | Entity, Property | is_factory=true for factories; Kogi/Ume platform records here |
| `qala.solutions.metrics` | 4 | Entity, Metric, Time, Category | D₃=TimeAxis, D₄=CategoryAxis (security/quality/performance/compliance) |
| `qala.ccrs` | 2 | Entity, Property | Lattice CRDT on status |
| `qala.releases` | 2 | Entity, Property | Distribution channels, signing records |
| `qala.artifacts` | 2 | Entity, Property | Hermetic hash, SLSA attestation |
| `qala.toolchains` | 2 | Entity, Property | Pinned versions, hermetic digest |
| `qala.ai_recommendations` | 2 | Entity, Property | 10 AI engine output types |
| `qala.cicd.pipelines` | 3 | Entity, Property, Time | Pipeline run history |
| `qala.security.events` | 3 | Entity, Property, Time | Threat timeline |
| `qala.governance.records` | 2 | Entity, Property | Domain Pack evaluation outputs |

---

## 42. The SDE as a Versioned N=3 Hypercube

The SDE uses N=3 (Entity × Property × Time) to capture the full version history of every SDE configuration, enabling hermetic build integrity — every historical state is addressable by timestamp:

```
qala.sdes (N=3):
  D₁ = SdeId, D₂ = field_name, D₃ = snapshot_timestamp

Current state:
  cell[(sde_id, "toolchain_ids",     NOW)].value = Json([...])
  cell[(sde_id, "hermetic_manifest", NOW)].value = Json({...})
  cell[(sde_id, "drift_status",      NOW)].value = Text("Clean")
  cell[(sde_id, "team_members",      NOW)].value = Json([...])

AS_OF time-travel:
  SELECT * FROM qala.sdes
  AS_OF '2026-01-15T10:00:00Z'
  WHERE D₁.id = '{sde_id}';

Rollback operation:
  1. Read all D₂ keys at D₃=target_timestamp via AS_OF
  2. Write those values atomically to D₃=NOW as a single batch
  3. EventLog records the rollback as RollbackEvent audit entry
  4. DriftDetectionEngine fires: new current state vs pinned manifest
  5. If drift_status = "Clean": rollback successful

Drift Detection (DriftDetectionEngine plugin):
  on_cell_mutation called on: environment_manifest, toolchain_ids, hermetic_manifest
  On mutation: hash(current_sde_state) vs hash(pinned_hermetic_manifest)
  If hashes differ: drift_status := "Drifted"; alert to Operations channel
  Target detection latency: < 15 minutes from drift event
```

The N=3 Hypercube eliminates the need for a separate SDE snapshot system. Every configuration change automatically creates a versioned history cell. AS_OF rollback needs no additional infrastructure.

---

## 43. The Solution Lifecycle as a Hypergrid Lattice

```rust
// Qala solution lifecycle lattice — partial order defining valid forward transitions
let lifecycle_lattice = LatticeOrder::new(vec![
    LatticeNode { state: "Draft",      successors: vec!["InReview"] },
    LatticeNode { state: "InReview",   successors: vec!["Approved", "Draft"] },
    LatticeNode { state: "Approved",   successors: vec!["Active", "InReview"] },
    LatticeNode { state: "Active",     successors: vec!["Deprecated"] },
    LatticeNode { state: "Deprecated", successors: vec!["Retired"] },
    LatticeNode { state: "Retired",    successors: vec![] },  // terminal
]);

// Concurrent transitions: Draft→InReview (node A) and InReview→Draft (node B)
// Lattice merge: join("Draft", "InReview") = "InReview"  ← forward state wins
// Governance invariant: lifecycle_state never regresses without Admin override
```

| State | Description | Entry Gate | Exit Conditions |
|-------|-------------|-----------|-----------------|
| **Draft** | Initial creation; work in progress | None | Developer submits for review |
| **InReview** | Formal review by designated reviewers | At least one CCR created | All approvers sign → Approved; any reject → Draft |
| **Approved** | All approvals obtained; staged for release | All CCR approvers signed | Staging validation passes → Active |
| **Active** | Publicly active; consumers can adopt | Release governance sign-off | Deprecation decision → Deprecated |
| **Deprecated** | End-of-life announced; existing consumers notified | Deprecation notice published | Notice period elapsed → Retired |
| **Retired** | Permanently retired; read-only historical record | Deprecation period complete | Terminal — no further transitions |

---

## 44. Change Control Requests on Hypergrid

```
qala.ccrs (N=2):
  cell[(ccr_id, "title")].value               = Text("Add OAuth2 authentication")
  cell[(ccr_id, "solution_id")].value         = Relation(solution_uuid)
  cell[(ccr_id, "change_type")].value         = Enum("FeatureAddition")
  cell[(ccr_id, "risk_tier")].value           = Enum("Medium")
  cell[(ccr_id, "status")].value              = Enum("UnderReview")  // Lattice CRDT
  cell[(ccr_id, "approver_ids")].value        = MultiRelation([...]) // OR-Set
  cell[(ccr_id, "approvals")].value           = Json({...})          // LWW
  cell[(ccr_id, "ai_risk_assessment")].value  = AiSignal(...)        // AI-write only
  cell[(ccr_id, "resolution_time_hours")].value = Number(47.3)       // LWW
```

The `approver_ids` field uses OR-Set — if two CCB members simultaneously add themselves from different federation nodes, both additions survive. The `status` field uses a Lattice CRDT encoding the workflow state machine.

CCR analytics across the N=4 metrics cube:
```sql
SELECT cell[D₁, "risk_tier"].value AS risk_tier,
       COUNT(D₁.ccr_id) AS ccr_count,
       AVG(cell[D₁, "resolution_time_hours"].value) AS avg_resolution_hours
FROM qala.ccrs
WHERE cell[D₁, "created_at"].value > '2026-04-01'
GROUP BY risk_tier ORDER BY avg_resolution_hours DESC;
```

---

## 45. Domain Packs as HypercubePlugins

| Domain Pack | Industry | Key Capabilities | Regulatory Scope |
|------------|----------|-----------------|-----------------|
| Software (Default) | Software / Technology | Code metrics, SAST/DAST, dependency audit, security scoring | SOC 2, ISO 27001, OWASP Top 10 |
| Pharmaceutical | Pharmaceutical | Batch formula versioning, GMP compliance, ingredient specs, regulatory submissions, AI regulatory risk | FDA 21 CFR 211, EMA, ICH Q10, GxP |
| Financial Instrument | Financial Services | KYC/AML, risk model validation, capital computation, stress test orchestration | SEC, FCA, FINRA, Basel III/IV |
| Medical Device | Medical Device | Design history file, ISO 14971 risk management, V&V tracking, adverse event monitoring | FDA 21 CFR 820, ISO 13485, MDR |
| Research Dataset | Academia / Research | Provenance tracking, IRB approval, data anonymization, reproducibility score | IRB, GDPR, NIH, FAIR |
| Creative Works | Creative / Media | IP ownership, license management, attribution chain, similarity detection | Copyright offices, CMOs |
| Government / Public | Government | FedRAMP controls, FISMA compliance, authority-to-operate workflow | FedRAMP, FISMA, NIST SP 800-53 |
| Aerospace / Defense | Aerospace & Defense | DO-178C certification, ITAR compliance, safety case management | FAA, EASA, DO-178C, MIL-STD |

The plugin interface:
```rust
impl HypercubePlugin for PharmaDomainPack {
    fn plugin_id(&self) -> &str { "qala.domain_pack.pharma.v2" }

    fn attribute_key_defs(&self) -> Vec<AttributeKeyDef> {
        vec![
            lww_text("batch_formula_version", "Batch Formula Version"),
            lww_text("pharmacopoeial_grade",   "Pharmacopoeial Grade"),
            lww_json("ingredient_specs",       "Ingredient Specifications"),
            lww_json("regulatory_submissions", "Regulatory Submissions"),
            // AI-computed (system-write only)
            AttributeKeyDef {
                key: "regulatory_risk_score".into(),
                attr_type: AttributeType::Ai,
                write_permission: PermissionTier::System,
                computation: Some(AttrComputation::AiEngine {
                    engine_id: "qala.pharma.regulatory_risk_engine",
                    model_name: "regulatory_risk_v2".into(),
                    input_attrs: vec!["ingredient_specs", "regulatory_submissions"],
                    staleness_ttl: Duration::from_secs(3600),
                    ..Default::default()
                }),
                crdt_semantics: CrdtSemantics::LastWriteWins,
                ..Default::default()
            },
        ]
    }

    fn on_cell_mutation(&self, cube_id: CubeId, coord: &DimCoordinate) -> Vec<AttributeKey> {
        // Invalidate regulatory_risk_score when inputs change
        if coord.d2_key() == Some(&DimKey::str("ingredient_specs"))
            || coord.d2_key() == Some(&DimKey::str("regulatory_submissions")) {
            return vec!["regulatory_risk_score".into()];
        }
        vec![]
    }

    fn compute_attribute(&self, key: &AttributeKey, ctx: &ComputeContext<'_>)
        -> HypergridResult<TypedAttrValue> {
        if key == "regulatory_risk_score" {
            let score = self.regulatory_risk_engine.compute(
                ctx.current_attrs.get("ingredient_specs"),
                ctx.current_attrs.get("regulatory_submissions"),
            );
            return Ok(TypedAttrValue::AiSignal {
                value: Box::new(TypedAttrValue::Number(score.value)),
                model_id: "regulatory_risk_v2".into(),
                computed_at: Utc::now(),
                confidence: score.confidence,
                explanation: Some(score.explanation),
            });
        }
        Ok(TypedAttrValue::Null)
    }
}
```

---

## 46. The Qala AI Agent — Ten Capabilities

| AI Capability | Plugin | Output Attribute | Target Cube | Update Trigger |
|--------------|--------|-----------------|-------------|----------------|
| SDE Optimization | `SdeOptimizationEngine` | `optimization_recommendations` | `qala.sdes` | On SDE team or toolchain change |
| Pipeline Bottleneck Detection | `PipelineAnalysisEngine` | `bottleneck_report` | `qala.cicd.pipelines` | On pipeline run completion |
| Predictive Defect Detection | `DefectPredictionEngine` | `defect_risk_score` | `qala.solutions` | On code metrics or test coverage change |
| Test Case Generation | `TestGenEngine` | `suggested_test_cases` | `qala.solutions` | On requirement or specification change |
| Anomaly Detection | `AnomalyEngine` | `anomaly_flags` | `qala.solutions.metrics` | On metric deviation >2σ |
| Resource Prediction | `ResourceForecastEngine` | `forecast_compute_units` | `qala.sdes` | On sprint planning or team change |
| Content Suggestions | `ContentSuggestionEngine` | `documentation_suggestions` | `qala.solutions` | On significant code or spec change |
| Security Threat Analysis | `SecurityThreatEngine` | `threat_report` | `qala.security.events` | On CVE feed update or dependency change |
| Backup Schedule Optimization | `BackupOptimizationEngine` | `optimal_backup_schedule` | `qala.sdes` | On SDE data volume or access pattern change |
| Prototype Feedback Analysis | `PrototypeFeedbackEngine` | `feedback_synthesis` | `qala.solutions` | On new user feedback or test results |

---

## 47. The Root Factory Principle

The Root Factory is the Qala factory that governs the entire Apapo platform — a HyperRow in `qala.factories` with `root_factory_flag = true`. The Root Factory's `qala.solutions` Hypercube contains:

```
qala.solutions — Root Factory children:
  KOGI_PLATFORM_UUID:   name="Kogi — Independent Worker OS", type="Platform", version="2.1.0"
  UME_PLATFORM_UUID:    name="Ume — Business OS",            type="Platform", version="2.0.0"
  QALA_PLATFORM_UUID:   name="Qala — Solution Factory OS",   type="Platform", version="1.0.0", is_factory=true
  HYPERGRID_UUID:       name="Hypergrid NDSS",               type="Platform", version="1.0.0"
  APAPO_UUID:           name="Apapo Platform",               type="Platform", version="1.0.0"
  + every Domain Pack (type=Good)
  + every built-in plugin (type=Application)
```

The platform operations team uses identical HyperQL dashboards, CCR workflows, and AI Agent signals to manage platform components as customers use to manage their solutions. No "internal systems" database — the platform manages itself through itself.

```sql
-- Platform ops: active Apapo components with elevated defect risk
SELECT cell[D₁, "name"].value AS solution_name,
       cell[D₁, "version"].value AS version,
       cell[D₁, "defect_risk_score"].value AS defect_risk
FROM qala.solutions
WHERE cell[D₁, "factory_id"].value = '{ROOT_FACTORY_ID}'
  AND cell[D₁, "defect_risk_score"].value > 0.7
  AND cell[D₁, "lifecycle_state"].value IN ('"Active"', '"InReview"')
ORDER BY defect_risk DESC;
```

---

## 48. Kogi and Ume as Solutions in Qala

Both platforms are first-class Solution HyperRows in the Qala Root Factory:

```
Kogi Platform (qala.solutions):
  solution_id:       KOGI_PLATFORM_UUID
  name:              "Kogi — Independent Worker OS"
  solution_type:     "Platform"
  version:           "2.1.0"
  maturity_stage:    "CM"  (Control Managed — production release)
  lifecycle_state:   "Active"
  sde_id:            KOGI_SDE_UUID
  factory_id:        ROOT_FACTORY_UUID
  domain_pack_ids:   ["qala.domain_pack.software.v1"]
  quality_score:     91.4  (AI-computed by DefectPredictionEngine)
  defect_density:    0.08  (AI-computed)
  security_score:    94.1  (AI-computed by SecurityThreatEngine)

Ume Platform (qala.solutions):
  solution_id:       UME_PLATFORM_UUID
  name:              "Ume — Business OS"
  solution_type:     "Platform"
  version:           "2.0.0"
  maturity_stage:    "CM"
  lifecycle_state:   "Active"
  quality_score:     88.7  (AI-computed)
```

When the Kogi team ships v2.2.0, the full workflow flows through Qala:
1. Kogi SDE (HyperRow in `qala.sdes`) is used to build and test the release candidate
2. CCR submitted (new HyperRow in `qala.ccrs`) describing all changes
3. AI Agent evaluates CCR → AiSignal risk assessment cell in the CCR's HyperRow
4. CCR flows through Root Factory's approval workflow (GovernanceEngine + Lattice CRDT)
5. On approval: Release record created (HyperRow in `qala.releases`)
6. `lifecycle_state` transitions `InReview → Approved → Active` via Lattice merge
7. Platform operations team queries full Kogi release history with same HyperQL as any customer solution
# Part VI — Apapo: The Convergent Platform

## 49. Cross-System Data Gravity

Because Kogi, Ume, and Qala all store their entities in the same Universal Cell Store architecture (or in federated UCS instances that sync via CrdtLog), cross-system queries are first-class HyperQL operations rather than engineering challenges requiring ETL pipelines, data warehouses, or synchronization jobs.

**Data gravity** is the effect where data attracts more data. Every new system joining the Hypergrid ecosystem adds its entities and relationships to the same shared infrastructure, increasing the value of cross-system queries superlinearly. The Kogi portfolio graph, the Ume organizational graph, and the Qala solution graph are segments of the same Hypergraph — when connected via CrossGridLink edges, the combined graph has emergent properties that none of the individual graphs has alone.

| Cross-System Integration | HyperQL Pattern | Consent |
|-------------------------|-----------------|---------|
| Kogi portfolio item → Qala solution | JOIN ON GRAPH_EDGE('CrossGridLink') | Yes — worker grants factory access |
| Ume employee → Kogi portfolio | JOIN ON GRAPH_EDGE('Employs') | Yes — worker grants org access |
| Qala solution → Ume product catalog | JOIN ON GRAPH_EDGE('Produces') | No — same org federation |
| Ume OKR → Qala SDE goals | JOIN ON GRAPH_EDGE('Association') | No — organizational |
| Kogi cooperative → Qala contributions | TRAVERSE OrgMembership; JOIN CrossGridLink | Yes — per-worker consent |
| Ume anomaly correlated with Qala drift | JOIN ON Association WHERE drift_status != 'Clean' | No — same federation |

---

## 50. Cross-System HyperQL Queries

**Platform health overview — all three systems:**
```sql
SELECT 'kogi' AS system,
       AVG(cell[D₁, D₂, "health_score", "2026-Q2"].value) AS avg_health,
       COUNT(D₁.entity_id) AS active_entities
FROM kogi.portfolio.kpis
WHERE cell[D₁, "status"].value = '"Active"' AND D₂.metric_name = 'health_score'
UNION ALL
SELECT 'ume' AS system,
       AVG(cell[D₁, "health_score"].value), COUNT(D₁.entity_id)
FROM ume.kernel.modules WHERE cell[D₁, "lifecycle_state"].value = '"Running"'
UNION ALL
SELECT 'qala' AS system,
       AVG(cell[D₁, "quality_score"].value), COUNT(D₁.entity_id)
FROM qala.solutions WHERE cell[D₁, "lifecycle_state"].value = '"Active"';
```

**Full contribution attribution chain:**
```sql
SELECT q.solution_id, cell[q.solution_id, "name"].value AS solution_name,
       k.worker_id,   cell[k.worker_id,   "name"].value AS worker_name,
       cell[k.worker_id, D₂, "health_score", "2026-Q2"].value AS worker_health,
       link.attribution_weight
FROM qala.solutions AS q
JOIN kogi.portfolio.links AS link ON GRAPH_EDGE(q.solution_id, link.component_id, 'CrossGridLink')
JOIN kogi.portfolio.components AS k ON link.source_entity_id = k.entity_id
WHERE cell[q.solution_id, "lifecycle_state"].value = '"Active"'
ORDER BY link.attribution_weight DESC;
```

**Organizational capacity analysis across Ume and Kogi:**
```sql
SELECT ume_unit.org_unit_id,
       cell[ume_unit.org_unit_id, "name"].value AS unit_name,
       COUNT(kogi_worker.worker_id) AS contractor_count,
       SUM(cell[kogi_worker.worker_id, "capacity"].value) AS total_capacity
FROM ume.admin.org_units AS ume_unit
TRAVERSE GRAPH FROM ume_unit EDGE_TYPE = Employs TARGET_GRID = kogi:// AS kogi_worker
WHERE cell[ume_unit.org_unit_id, "type"].value = '"Department"'
GROUP BY ume_unit.org_unit_id ORDER BY total_capacity DESC;
```

**Time-travel incident investigation:**
```sql
SELECT 'kogi.active_portfolios' AS entity_type, COUNT(*) AS count
FROM kogi.portfolio.components AS_OF '2026-03-15T14:22:00Z'
WHERE cell[D₁, "status"].value = '"Active"'
UNION ALL
SELECT 'ume.running_modules', COUNT(*) FROM ume.kernel.modules AS_OF '2026-03-15T14:22:00Z'
WHERE cell[D₁, "lifecycle_state"].value = '"Running"'
UNION ALL
SELECT 'qala.active_sdes', COUNT(*) FROM qala.sdes AS_OF '2026-03-15T14:22:00Z'
WHERE cell[D₁, "status"].value = '"Active"';
```

**Solution quality by CCR risk tier — platform management view:**
```sql
SELECT c.risk_tier,
       COUNT(c.ccr_id) AS ccr_count,
       AVG(c.resolution_time_hours) AS avg_resolution_hours,
       AVG(s.quality_score) AS avg_solution_quality
FROM (SELECT cell[D₁, "risk_tier"].value AS risk_tier,
             cell[D₁, "resolution_time_hours"].value AS resolution_time_hours,
             cell[D₁, "solution_id"].value AS solution_id,
             D₁.entity_id AS ccr_id
      FROM qala.ccrs
      WHERE cell[D₁, "created_at"].value > '2026-04-01') AS c
JOIN (SELECT D₁.entity_id AS sol_id,
             cell[D₁, "quality_score"].value AS quality_score
      FROM qala.solutions) AS s ON c.solution_id = s.sol_id
GROUP BY c.risk_tier ORDER BY avg_resolution_hours DESC;
```

---

## 51. The Federation Protocol

Three federation topologies are in use across the Apapo platform:

**Topology 1 — Single-system multi-node:**
```
[Kogi Grid Node 1 (primary)] ←─ CrdtLog delta over Kafka ─→ [Kogi Grid Node 2]
[Ume Grid Node 1 (primary)]  ←─ CrdtLog delta over Kafka ─→ [Ume Grid Node 2]
[Qala Grid Node 1 (primary)] ←─ CrdtLog delta over Kafka ─→ [Qala Grid Node 2]
Each system runs independently; nodes sync via per-system Kafka topics.
```

**Topology 2 — Cross-system integrated ecosystem:**
```
[Kogi Grid] ←──── CrossGridLink ────→ [Qala Grid]
    │                                       │
    │──────── CrossGridLink ──────→ [Ume Grid] ──────┘

[Federation Coordinator Service]
├── CrossGridLink consent flow management
├── Shadow cell sync routing (Kafka-mediated)
├── Cross-grid HyperQL fan-out query coordination
└── Federation health monitoring (HG-OPS)
```

**Topology 3 — Edge/offline deployment:**
```
[Local Kogi Grid (SQLite, offline)] ──reconnect──→ [Kogi Grid (PostgreSQL)]
[Air-gapped Qala Grid]              ──manual sync──→ [Qala Grid (main)]
Offline nodes accumulate CrdtLog operations locally.
On reconnect: delta sync pushes all local ops to server.
CRDT merge resolves all conflicts deterministically.
```

---

## 52. The CrossGridLink and ShadowCell Protocol

CrossGridLink is the mechanism for all cross-domain connections in Apapo. Full six-phase protocol:

```
Phase 1: Link Request
  Entity A creates CrossGridLink to Entity B.
  If consent required: ConsentRequest sent to B's owner.
  ConsentStatus: Pending.

Phase 2: Consent Configuration
  Entity B configures:
    mirrored_attrs: Vec<AttributeKey>   -- visible in ShadowCell in A's grid
    writeback_attrs: Vec<AttributeKey>  -- A may write back these attributes
    update_policy: RealTime | Batch (default: RealTime)
  ConsentStatus: Accepted.

Phase 3: ShadowCell Provisioning
  ShadowCell created in Grid A:
    shadow_id, source_grid_id, source_d1_key, source_edge_id
    synced_values: current snapshot of mirrored_attrs
    last_synced_at, update_policy

Phase 4: Real-Time Sync
  When B writes to any mirrored_attr:
    EventLog → Kafka topic: shadow.sync.{grid_id}
    Federation Coordinator routes delta to Grid A
    Grid A applies delta to ShadowCell (LWW merge)
    Grid A EventLog receives ShadowCellUpdated entry

Phase 5: Write-Back (if configured)
  A writes to a writeback_attr on the ShadowCell:
    WritebackService validates PermissionTier (must be Editor+)
    Write applied to Grid B's actual HyperCell
    Attributed to A's actor ID in Grid B's EventLog
    ConsentLog updated with write record

Phase 6: Link Lifecycle
  Suspended: sync pauses; ShadowCell shows stale marker
  Renegotiated: mirrored_attrs/writeback_attrs changed with mutual agreement
  Revoked: ShadowCell archived; no further sync
  Expired: time-limited CrossGridLink reaches expiry
```

---

## 53. Identity Across All Three Systems

A human actor in Apapo may simultaneously be a Kogi worker, a Ume employee, and a Qala developer. The HG-ID identity model handles this through CrossTenantMerge:

```
SovereignEntity (root identity — one real-world person)
├── Kogi Identity: @alice-work
│   ├── Kogi Profile: Professional (public portfolio)
│   ├── Kogi Profile: Client-facing (selected work)
│   └── Kogi Accounts: [platform_creds, marketplace_creds]
│
├── Ume Identity: alice@acmecorp.com
│   ├── Ume Employee Record: HyperRow in ume.hr.employees
│   ├── Ume Role: Software Engineer (PermissionTier::Editor in eng module)
│   └── Ume Account: [sso_creds]
│
└── Qala Identity: alice@acmecorp.qala
    ├── Qala Developer: HyperRow in qala.sdes (team_members)
    ├── Qala Role: Developer (PermissionTier::Editor in SDE)
    └── Qala Account: [sde_creds]
```

The `Employs` CrossGridLink from Alice's Ume identity to her Kogi identity is the mechanism enabling her org to see her work records (with her consent) and her to see employment status in her Kogi portfolio. The CrossTenantMerge operation establishes a verified cryptographic link between the identities, scoped to the specific mirrored_attrs both parties agree to.

---

## 54. The Economic Graph: Kogi × Ume × Qala

The full Apapo economic graph emerges from the union of all CrossGridLink edges across all three systems:

| Edge Category | Edge Types | What it Models |
|--------------|-----------|----------------|
| **Labor markets** | Employs, Contracted | Ume organizations hiring Kogi workers |
| **Investment flows** | InvestedIn | Workers/organizations investing in portfolio components |
| **Solution production** | CrossGridLink | Who built which Qala solutions (Kogi → Qala) |
| **Organizational membership** | OrgMembership | Kogi cooperative members and Ume orgs |
| **Resource sharing** | ResourceShares | Tools, templates, knowledge flowing across the graph |
| **Marketplace transactions** | MarketplaceTransaction | Economic value exchange |
| **Contribution attribution** | Collaborates | Contribution records with attribution weights |

The LinkForest — full traversal of this graph from any root identity — reveals the complete economic and collaborative context of any entity in the Apapo ecosystem. This is not an analytics view built on top of data — it is the data structure of the platform itself.

---

## 55. Unified Audit Trail and Compliance

Every mutation to every entity in every domain system dual-writes to the shared Hypergrid EventLog. This creates a single unified, time-indexed, causally ordered audit trail spanning all domains.

| System | Compliance Requirement | Hypergrid Implementation |
|--------|----------------------|--------------------------|
| **Kogi** | GDPR (data sovereignty; right to erasure via archival) | VisibilityMask at query time; archival marks rows inaccessible; AS_OF preserves history |
| **Ume** | SOX (immutable financial audit trail); GDPR; SOC 2 | EventLog partitioned by time; no UPDATE/DELETE; PG RLS enforcement |
| **Qala** | FDA GMP (formulation provenance); SEC; SOC 2; ISO 27001 | Every CCR/lifecycle transition/artifact hash in EventLog; Domain Pack adds regulatory evidence |
| **Apapo** | Platform-wide compliance reporting | AS_OF queries across all cubes; HyperQL reports in <4 hours |

The PostgreSQL RLS enforcement:
```sql
-- Append-only enforcement at the database layer
CREATE POLICY event_log_no_update ON hypergrid_events FOR UPDATE USING (false);
CREATE POLICY event_log_no_delete ON hypergrid_events FOR DELETE USING (false);
-- Only INSERTs permitted; no row can ever be modified or removed
```

---

## 56. The Self-Governing Platform in Detail

The self-governing architecture has five structural properties, each independently valuable:

**1. Single Source of Truth for Platform State.** The Qala Root Factory's `qala.solutions` Hypercube is the authoritative record of every version of every Apapo platform component. There is no separate "internal systems" database.

**2. Platform Telemetry as Solution Metrics.** The AI Agent's quality_score, defect_density, and security_score computations run on Kogi and Ume solutions in `qala.solutions` exactly as they run on customer solutions. Platform engineers see identical health dashboards as customers.

**3. Governance Dogfooding.** Every Kogi or Ume feature release goes through a CCR in the Qala Root Factory. The platform team experiences the same governance overhead and benefits as their customers, creating strong incentives to make governance as smooth as possible.

**4. Cross-Platform Dependencies in Hypergraph.** The Root Factory Hypergraph contains Association edges from Kogi to Ume (Kogi's worker experience depends on Ume integrations) and Ume to Qala (Ume's solution module tracks Qala-governed solutions). These live edges affect AI risk assessments and can trigger governance workflows if a dependency's quality degrades.

**5. Self-Upgrading Schema.** New Hypergrid axis types and attribute types are first exercised in the Root Factory on platform solutions before being promoted to the marketplace. The platform is its own first customer.

---

# Part VII — Technical Reference

## 57. Physical Storage Architecture

| Storage Layer | Backend | Use Case | Characteristics |
|--------------|---------|----------|----------------|
| **Primary cell store** | PostgreSQL 16+ JSONB | OLTP HyperCell reads/writes for N≤4 cubes | RLS at DB layer; BRIN for TimeAxis; GIN for JSONB; HASH-partitioned by grid_id |
| **Analytics backend** | ClickHouse | N>4 cubes; DimFold/DimExpand >1M rows | Receives Kafka delta stream; append-only writes; sub-second aggregate reads |
| **CRDT hot buffer** | Redis Cluster | CrdtLog in-flight ops; computed attr cache; WS session | Sub-ms R/W; 24h TTL; flushed to PG on expiry |
| **EventLog cold archive** | S3 / MinIO | EventLog beyond 10K-event hot cap; Parquet exports | S3 versioning; IA tier after 90d; cross-region replication |
| **Search index** | Meilisearch | Full-text + faceted search across all HyperRows | Sub-10ms query; rebuilt daily from PG |
| **Graph backend (v1)** | PostgreSQL recursive CTE | Hypergraph edges; BFS/DFS for graphs <1M nodes | Recursive CTEs; adequate for most domain deployments |
| **Graph backend (v2.5)** | Neo4j Enterprise | KLNK at platform scale >1M nodes | Native graph queries; Louvain community detection |
| **Snapshot store** | S3 / MinIO | HyperCube point-in-time snapshots; disaster recovery | S3 versioning; lifecycle policies per cube |
| **Feature store** | Redis + PostgreSQL MV | kogi-engine / Ume AI / Qala AI feature vectors | Per-entity feature vectors; rebuilt hourly from EventLog |

The physical cell store schema:
```sql
CREATE TABLE hypergrid_cells (
    grid_id         UUID NOT NULL,
    cube_id         UUID NOT NULL,
    dim1_key        UUID NOT NULL,              -- EntityAxis key (entity ID)
    dim2_key        TEXT NOT NULL,              -- PropertyAxis key (field name)
    dim3_key        TEXT,                       -- Optional: TimeAxis / CategoryAxis
    dim4_key        TEXT,                       -- Optional: GeoAxis / TenantAxis
    attr_value      JSONB NOT NULL DEFAULT '{}', -- TypedAttrValue as JSONB
    attr_version    BIGINT NOT NULL DEFAULT 0,
    attr_crdt_ts    BIGINT NOT NULL,            -- Logical clock timestamp (LWW)
    last_actor      TEXT NOT NULL,              -- node_id:identity_tag
    last_mutated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (grid_id, cube_id, dim1_key, dim2_key, dim3_key, dim4_key)
) PARTITION BY HASH(grid_id);

-- Key indexes
CREATE INDEX idx_cells_d1      ON hypergrid_cells (grid_id, cube_id, dim1_key);
CREATE INDEX idx_cells_d3_brin ON hypergrid_cells USING BRIN (dim3_key) -- TimeAxis
  WHERE dim3_key IS NOT NULL;
CREATE INDEX idx_cells_value   ON hypergrid_cells USING GIN (attr_value);
```

---

## 58. Service Architecture

| Service | Protocol | Language | Responsibility |
|---------|----------|----------|---------------|
| `api-gateway` | HTTP/2 + gRPC-Web + WSS | Go | Auth (JWT/API key), rate limiting, routing, CORS, API versioning |
| `portfolio-service` | gRPC + REST | Go | Kogi KPMS/KPSS: PortfolioComponent CRUD, sheet queries |
| `spreadsheet-service` | gRPC + REST | Go | HyperQL execution, DimSlice/DimFold/DimExpand, HypercubeView management |
| `crdt-service` | gRPC | Rust | CRDT merge engine, VectorClock, CrdtLog management, delta sync |
| `federation-coordinator` | gRPC + Kafka | Go | CrossGridLink consent flows, shadow cell sync routing, cross-grid query fan-out |
| `writeback-service` | gRPC | Rust | AI/formula attribute writeback, PermissionTier enforcement, audit trail |
| `namespace-service` | gRPC + REST | Go | NamespacePath resolution, alias management, cross-grid namespace federation |
| `identity-service` | gRPC + REST | Go | SovereignEntity, Identity, TenantPartition, VisibilityMask, CrossTenantMerge |
| `space-service` | gRPC + REST | Go | Space/Workspace lifecycle, GovernanceConfig, Space link subgraph |
| `graph-service` | gRPC + REST | Go | Hypergraph edge management, BFS/DFS traversal, KLNK |
| `kogi-engine` | Kafka + gRPC | Scala 3 | 12 sub-engines consuming EventLog; AI signals via writeback-service |
| `ume-engine` | Kafka + gRPC | Go / Python | Ume AI capabilities; module health; Chombo/Soko evaluations |
| `qala-engine` | Kafka + gRPC | Rust | 10 Qala AI Agent capabilities; Domain Pack AI; drift detection |
| `search-service` | REST | Go | Full-text + faceted search; NL-to-HyperQL translation |
| `export-service` | REST | Go | CSV, XLSX, JSON-LD, Parquet, Arrow, GraphQL export |
| `ops-service` | REST + Prometheus | Go | Health probes, metrics, tracing, federation health monitoring |

---

## 59. Performance Targets

| Operation | Target p50 | Target p99 | Scale Bound |
|-----------|-----------|-----------|-------------|
| Single HyperCell read | <5ms | <20ms | Primary key lookup; scales linearly with N |
| HyperRow read (all fields) | <10ms | <40ms | O(field_count) batched read |
| DimSlice query (N=2, <1K rows) | <50ms | <200ms | PostgreSQL GIN/BTree index |
| DimFold aggregation (N=3, <100K rows) | <500ms | <2s | PG window functions; ClickHouse for larger |
| Graph traversal (depth=5, fan-out=10) | <200ms | <800ms | PG recursive CTE for <1M nodes |
| AS_OF time-travel (single HyperRow) | <100ms | <500ms | EventLog replay; hot window in PG |
| CRDT merge (single attribute, in-memory) | <1ms | <10ms | In-memory merge; async persistence |
| AI attribute computation (Tier 2) | 100ms–30s | <60s | Async; result written via WritebackService |
| CrossGridLink delta sync | <1s | <5s | Kafka propagation + ShadowCell update |
| EventLog flush to S3 archive | <30s | <2min | Batch flush at 10K-event hot cap |
| KLNK graph traversal (depth=3) | <500ms | <2s | PG CTE; Neo4j at >1M nodes |
| NL-to-HyperQL translation | <300ms | <1s | LLM inference; 4K context window |
| Full portfolio load (35 sheets, 500 rows) | <2s | <5s | Parallel HyperRow reads + view filter |

---

## 60. Security Pipeline

Every mutation flows through a multi-layer security pipeline before persistence:

```
1. API Gateway Authentication
   JWT/API key validation · Rate limiting (per-user, per-grid)
   TLS termination · mTLS for service-to-service calls

2. PermissionTier Enforcement (bridge layer)
   write_permission check: caller.tier >= attr.write_permission
   AttrVisibility check enforced at read path
   System-write-only attributes: AI scores, system metadata rejected if non-system caller

3. VisibilityMask Application (identity-service)
   DimSlice predicates per observer type applied to every read
   TenantPartition isolation: identity_tags filter on all reads
   SplitPolicy enforcement for cross-partition access

4. PolicyEngine Evaluation (governance)
   GovernanceEngine evaluates all registered PolicyEngine plugins
   BudgetCapPolicy: reject writes exceeding allocated budget
   LifecycleApprovalPolicy: require CCR for lifecycle transitions
   Custom domain policies: per-cube governance rules

5. CRDT Validation
   LWW: reject future timestamps; reject clock regression
   Lattice: transition validity checked against partial order
   OR-Set: tag uniqueness validated; remove tags checked against known adds

6. Audit Trail (append-only)
   Every write → EventLog entry: actor, VectorClock, before/after values
   EventLog rows: PostgreSQL-level append-only (no UPDATE/DELETE)
   All entries visible to Owner-tier users via AS_OF queries

7. Field-Level Encryption (KMS-backed)
   PII fields: AES-256-GCM via AWS KMS / HashiCorp Vault
   Owner-level encryption: only owner can decrypt private fields
   No raw key material in application code; all via KMS API
```

---

## 61. Bootstrap Sequence

| Step | Operation | Blocking? | Failure Handling |
|------|-----------|-----------|-----------------|
| 1 | Config load and validation | Yes | Fatal: refuse to start; log all missing config keys |
| 2 | Storage connectivity (PG, Redis, Kafka); apply migrations | Yes | Retry with exponential backoff; fatal after 5min |
| 3 | Plugin registry init: load all HypercubePlugin implementations | Yes | Degraded plugins: skip + warn; missing required: fatal |
| 4 | Grid bootstrap: call DomainStore::bootstrap() for all domain systems | Yes | Idempotent: re-registering existing cube/attr is no-op |
| 5 | EventLog recovery: replay unapplied CrdtLog entries in causal order | Yes | CRDT merge on conflict; log recovery events |
| 6 | Namespace warm-up: pre-populate Redis resolution cache | No (async) | Cache starts cold; PG fallback always available |
| 7 | Federation handshake: sync with all registered peers | No (async) | Standalone mode; retry every 30s |
| 8 | AI engine warm-up: health-check all AIEngineAdapter plugins | No (async) | AI degrades gracefully; cache serves stale values |
| 9 | Ready signal: readiness probe = healthy; api-gateway routes traffic | N/A | Only reached if steps 1–5 all succeeded |

---

## 62. Environment Configuration Reference

```bash
# Apapo Grid deployment configuration

# ── Core Grid identity ──────────────────────────────────────────
GRID_ID=<uuid>
GRID_NAME=kogi-production         # or: ume-production / qala-production
GRID_VERSION=1.0.0
NODE_ID=node-us-east-1

# ── Domain system selector ──────────────────────────────────────
DOMAIN_SYSTEM=kogi                # kogi|ume|qala|hypergrid_standalone
KOGI_ENGINE_GRPC_ADDR=kogi-engine:9100
UME_ENGINE_GRPC_ADDR=ume-engine:9200
QALA_ENGINE_GRPC_ADDR=qala-engine:9300

# ── PostgreSQL (primary cell store) ────────────────────────────
POSTGRES_URL=postgresql://user:pass@host:5432/hypergrid
POSTGRES_MAX_CONNECTIONS=100
POSTGRES_PARTITION_COUNT=32

# ── Redis Cluster (CRDT buffer + cache) ────────────────────────
REDIS_CLUSTER_URLS=redis://host1:6379,redis://host2:6379
REDIS_CRDT_TTL_SECONDS=86400
REDIS_CELL_CACHE_DEFAULT_TTL=1800

# ── ClickHouse (analytics backend) ─────────────────────────────
CLICKHOUSE_URL=http://host:8123/hypergrid_analytics
CLICKHOUSE_ANALYTICS_THRESHOLD=1000000

# ── Apache Kafka (event streaming + CRDT delta sync) ───────────
KAFKA_BOOTSTRAP_SERVERS=broker1:9092,broker2:9092
KAFKA_CRDT_TOPIC_PARTITIONS=32
KAFKA_REPLICATION_FACTOR=3
KAFKA_RETENTION_MS=604800000      # 7 days

# ── S3 / Object Store ──────────────────────────────────────────
S3_BUCKET=hypergrid-event-archive
EVENTLOG_HOT_CAP=10000            # Flush to S3 after 10K events per entity

# ── AI Engine ──────────────────────────────────────────────────
AI_ENGINE_MODE=openai_compatible  # openai_compatible|local_llm|disabled
OPENAI_COMPATIBLE_BASE_URL=https://api.openai.com/v1
LOCAL_LLM_URL=http://ollama:11434
AI_COMPUTE_WORKER_POOL_SIZE=16

# ── Federation ─────────────────────────────────────────────────
FEDERATION_ENABLED=true
FEDERATION_HEARTBEAT_INTERVAL_SECS=30
FEDERATION_MAX_PEERS=255
FEDERATION_TRUST_LEVEL=grid       # grid|namespace|identity|all_three

# ── Encryption (KMS) ───────────────────────────────────────────
KMS_PROVIDER=aws_kms              # aws_kms|hashicorp_vault|local_dev
KMS_KEY_ARN=arn:aws:kms:...
FIELD_ENCRYPTION_ENABLED=true

# ── Observability ──────────────────────────────────────────────
OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:4317
PROMETHEUS_METRICS_PORT=9901
LOG_LEVEL=info
LOG_FORMAT=json
```

---

# Appendices

## Appendix A: Complete Hypercube Registry

**Kogi (8 cubes):**
`kogi.portfolio.components` (N=2) · `kogi.portfolio.kpis` (N=3) · `kogi.portfolio.finances` (N=3) · `kogi.portfolio.benefits` (N=2) · `kogi.portfolio.actions` (N=2) · `kogi.portfolio.approvals` (N=2) · `kogi.portfolio.links` (N=2) · `kogi.portfolio.snapshots` (N=2)

**Ume (44 cubes):**
`ume.kernel.modules` (N=2) · 42 module cubes (see §32) · `ume.templates.library` (N=2)

**Qala (12 cubes):**
`qala.factories` (N=2) · `qala.sdes` (N=3) · `qala.solutions` (N=2) · `qala.solutions.metrics` (N=4) · `qala.ccrs` (N=2) · `qala.releases` (N=2) · `qala.artifacts` (N=2) · `qala.toolchains` (N=2) · `qala.ai_recommendations` (N=2) · `qala.cicd.pipelines` (N=3) · `qala.security.events` (N=3) · `qala.governance.records` (N=2)

---

## Appendix B: CRDT Selection Decision Tree

```
Is the attribute a multi-valued set?
  (tags, owners, children, dependencies, members, toolbox_ids, approver_ids)
  YES → OR-Set

Is the attribute a monotonically increasing counter?
  (restart_count, view_count, follower_count, analytics counters)
  YES → GrowOnlyCounter

Is the attribute a monotonically increasing maximum?
  (version, sequence_number, schema_version)
  YES → MaxRegister

Is the attribute a lifecycle state following a partial order?
  (ComponentStatus, ModuleLifecycleState, SolutionLifecycleState, CcrStatus)
  YES → Lattice  (LWW in v1 as interim; Lattice in v2)

Is the attribute a numerical accumulator via concurrent transactions?
  (budget_spent, hours_logged, consumption_units)
  YES → PnCounter

Is the attribute AI-computed?
  (health_score, risk_score, defect_density, drift_status, compliance_status)
  YES → LastWriteWins with PermissionTier::System write restriction

Is the attribute human-authored or a configuration value?
  (name, description, config, payload, JSON blobs, environment_manifest)
  YES → LastWriteWins

Otherwise → Consult domain architects; document choice in AttributeKeyDef.notes
```

---

## Appendix C: HyperQL Syntax Reference

```sql
-- Basic entity query (N=2)
SELECT cell[D₁.id, "field"].value AS field
FROM {cube_name}
WHERE cell[D₁.id, "status"].value = '"Active"'
LIMIT 50;

-- Time-series slice (N=3 with TimeAxis)
SELECT D₁.entity_id, D₃.period, cell[D₁, D₂, D₃, "metric"].value AS value
FROM {cube_name}
WHERE D₃.period BETWEEN '2025-Q1' AND '2026-Q4'
  AND D₂.metric_name = 'health_score'
ORDER BY D₃.period;

-- DimFold (aggregate over a dimension)
SELECT D₁.entity_id,
       FOLD D₃ WITH AVG(cell.value) AS avg_score
FROM {cube_name}
WHERE D₂.metric_name = 'health_score' AND D₃.period >= '2025-Q1'
GROUP BY D₁.entity_id;

-- DimExpand (pivot a dimension into columns)
SELECT D₁.entity_id, EXPAND D₄ AS COLUMNS(AVG(cell.value))
FROM qala.solutions.metrics WHERE D₂.metric_name = 'quality_score'
GROUP BY D₁.entity_id;
-- Result: entity_id | security_avg | quality_avg | performance_avg | compliance_avg

-- Graph traversal
TRAVERSE GRAPH FROM {start_node_id}
  EDGE_TYPE = Hierarchy DIRECTION = Outbound MAX_DEPTH = 5
SELECT node.id, cell[node.id, "name"].value, cell[node.id, "status"].value;

-- AS_OF time-travel
SELECT D₁.entity_id, cell[D₁, "status"].value, cell[D₁, "name"].value
FROM {cube_name} AS_OF '2026-03-15T14:22:00Z';

-- Cross-grid join via Hypergraph
SELECT k.entity_id, cell[k, "name"].value AS kogi_name,
       q.entity_id, cell[q, "name"].value AS qala_name
FROM kogi.portfolio.components AS k
JOIN qala.solutions AS q ON GRAPH_EDGE(k.entity_id, q.entity_id, 'CrossGridLink')
WHERE cell[k, "status"].value = '"Active"' AND cell[q, "lifecycle_state"].value = '"Active"';

-- SHADOW INCLUDE (include mirrored shadow cells in results)
SELECT D₁.entity_id, cell[D₁, "name"].value,
       SHADOW cell[D₁, "delivery_status"].mirror_value AS partner_delivery
FROM kogi.portfolio.components
WHERE cell[D₁, "visibility"].value = '"Public"'
SHADOW INCLUDE FROM grid://partner-org/;
```

---

## Appendix D: Domain Pack Plugin Full Interface

```rust
pub trait HypercubePlugin: Send + Sync {
    // Identity
    fn plugin_id(&self) -> &str;
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn dependencies(&self) -> Vec<PluginId> { vec![] }

    // Schema extension
    fn attribute_key_defs(&self) -> Vec<AttributeKeyDef> { vec![] }
    fn axis_type_defs(&self) -> Vec<CustomAxisTypeDef> { vec![] }
    fn edge_type_defs(&self) -> Vec<CustomEdgeTypeDef> { vec![] }
    fn render_mode_defs(&self) -> Vec<CustomRenderModeDef> { vec![] }

    // Tier-2 AI computation
    fn compute_attribute(&self, key: &AttributeKey, ctx: &ComputeContext<'_>)
        -> HypergridResult<TypedAttrValue> { Ok(TypedAttrValue::Null) }
    fn on_cell_mutation(&self, cube_id: CubeId, coord: &DimCoordinate)
        -> Vec<AttributeKey> { vec![] }

    // Governance hooks
    fn validate_transition(&self, entity_id: EntityId, from_state: &str, to_state: &str)
        -> ValidationResult { ValidationResult::Allow }
    fn on_ccr_submitted(&self, ccr_id: CcrId, solution_id: SolutionId)
        -> Vec<ComplianceCheckResult> { vec![] }

    // Lifecycle
    fn on_load(&self) -> HypergridResult<()> { Ok(()) }
    fn on_unload(&self) {}
    fn health_check(&self) -> PluginHealthReport;
}
```

---

## Appendix E: Key Design Decisions Log

| Decision | Chosen Approach | Rationale | Version |
|----------|-----------------|-----------|---------|
| Primary entity storage layout | D₁=EntityId, D₂=FieldName, cell["value"] | Maximizes HyperCell reuse; simplest codec; uniform across all systems | v1.0 |
| Portfolio as Kogi root domain | Portfolio IS the system — every entity is a row | Architectural clarity; eliminates cross-module data silos | v1.0 |
| Organization as Ume root domain | Organization IS the system — 42 modules are views | Same clarity; replaces per-module schemas with per-module cubes | v1.0 |
| Solution as Qala root domain | Solution IS the universal category; factories, Kogi, Ume are Solutions | Self-describing, self-governing architecture | v1.0 |
| Status/lifecycle CRDT | LWW in v1; Lattice in v2 | LWW safe and simple; Lattice adds governance enforcement at substrate level | v1.0/v2.0 |
| AI writeback mechanism | HypercubePlugin + System PermissionTier | Consistent with plugin architecture; prevents user overwrite; auditable | v1.0 |
| Cross-system integration | CrossGridLink + ShadowCell + MirrorAttribute | Consent-gated; attribute-level granularity; no full data copy | v1.0 |
| Federation sync protocol | CrdtLog delta sync over Apache Kafka | Reliable delivery; causal ordering; horizontally scalable; 255-node limit | v1.0 |
| Schema evolution policy | Additive changes CRDT-commutative; destructive require governance gate | Zero-downtime schema updates; immutability as governance invariant | v1.0 |
| HyperQL design | Custom N-dim + NL bridge for user-facing queries | AS_OF and TRAVERSE GRAPH not expressible in standard SQL | v1.0 |
| EventLog architecture | Dual-write: domain EventLog + HG EventLog | Unified audit trail + domain-typed querying; single time-travel source | v1.0 |
| Namespace scheme | {domain}://{space_type}/{slug}/{entity_type}/{id}/ | Domain prefixes prevent collision; URI-shaped for HTTP integration | v1.0 |
| Max dimensionality | N=16 hard limit; N=6 recommended | Beyond N=6, UI usability degrades significantly | v1.0 |
| Cell store encoding | Hybrid (dense D₁×D₂; sparse D₃+) | Best performance for common N=2 case with higher-dim support | v1.0 |
| AI computation tiers | Tier-1 (sync formula) + Tier-2 (async AI) | Separates fast deterministic from slow non-deterministic computations | v1.0 |
| Self-governing platform | Kogi + Ume as Solutions in Qala Root Factory | Platform demonstrates its own capabilities continuously | v1.0 |

---

## Appendix F: Glossary

| Term | Definition |
|------|-----------|
| **Apapo** | The complete integrated platform ecosystem: Hypergrid substrate + Kogi + Ume + Qala. The name for the convergent whole. |
| **Hypergrid** | The N-Dimensional Distributed Spreadsheet System — the universal substrate providing all data infrastructure. |
| **Grid** | Root runtime container for one Hypergrid deployment. Owns all Hypercubes, Hypergraph, EventLog, CrdtLog. |
| **Hypercube** | An N-dimensional grid H=(D₁,D₂,…,Dₙ). Generalization of a spreadsheet sheet. Named {domain}.{entity_type}. |
| **HyperRow** | All cells sharing the same D₁ key — the canonical entity. Generalization of a spreadsheet row. |
| **HyperCell** | Data point at an N-dimensional coordinate carrying an N-attribute map. The universal data atom. |
| **Universal Cell Store (UCS)** | Physical storage layer holding all HyperCells across all Hypercubes. Keyed by (grid_id, cube_id, dim_keys_tuple). |
| **DimensionAxis (Dᵢ)** | One axis of a Hypercube: EntityAxis (D₁), PropertyAxis (D₂), or custom type (D₃–D₁₆). |
| **TypedAttrValue** | Typed value in a HyperCell attribute: Text\|Number\|Currency\|Bool\|DateTime\|Json\|AiSignal\|AnomalySignal\|…|
| **DimSlice** | Predicate applied to dimension axes producing a sub-cube or projection. Generalization of a filter. |
| **DimFold** | Collapsing an axis by aggregating all its key values into a summary. Generalization of GROUP BY. |
| **DimExpand** | Expanding a dimension's key set as separate result columns. Generalization of PIVOT. |
| **HypercubeView** | Saved configuration of DimSlices, DimFolds, filters, sorts, and render mode. The 35 Kogi sheets are HypercubeViews. |
| **HyperQL** | The N-dimensional query language: SELECT, DimSlice WHERE, FOLD, EXPAND, TRAVERSE GRAPH, AS_OF. |
| **Hypergraph** | Graph layer: typed directed edges between HyperCells, HyperRows, Cubes, Spaces, and Grids. |
| **CrossGridLink** | A HypergraphEdge crossing Grid boundaries. Requires consent. Creates ShadowCells. |
| **ShadowCell** | Read-only reflection in Grid A of a linked entity from Grid B, created by a CrossGridLink. |
| **MirrorAttribute** | A specific attribute from a ShadowCell synced into the host Grid as read-only computed attribute. |
| **LinkForest** | The complete set of all link trees rooted at a given identity — the full cross-grid network. |
| **Space** | Named, governed, bounded operational context within a Grid. Groups Hypercubes, members, and governance. |
| **Workspace** | Active working session within a Space. Personalized cube/view arrangement + session state. |
| **NamespacePath** | Hierarchical URI addressing any entity: {domain}://{space_type}/{slug}/... |
| **SovereignEntity** | Real-world entity (person, org, agent) that owns a root Grid and controls their data absolutely. |
| **VisibilityMask** | N-dimensional DimSlice predicates defining what is visible to each observer type. Enforced at query time. |
| **VectorClock** | HashMap<NodeId, u64>. Logical clock for causal ordering of mutations across federation nodes. |
| **CRDT** | Conflict-free Replicated Data Type — per-attribute semantics: LWW\|OR-Set\|Counter\|MaxRegister\|Lattice. |
| **CrdtLog** | In-flight CRDT operation buffer. Hot in Redis; durable in PostgreSQL. Federated via Kafka. |
| **EventLog** | Append-only immutable log of every mutation. Single source of truth for time-travel and audit. |
| **AS_OF** | Time-travel query operator: returns the cube state at a historical moment via EventLog replay. |
| **WritebackService** | gRPC service accepting AI-computed values and writing them into HyperCells with full audit trail. |
| **HypercubePlugin** | Extension interface for all Apapo customizations: axis types, attr types, AI engines, render modes. |
| **AIEngineAdapter** | Plugin interface connecting AI/ML services to provide Tier-2 computed attributes. |
| **Domain Pack** | Bundled HypercubePlugin collection providing domain-specific governance, compliance, and AI. |
| **KIMDSS** | Kogi Interconnected Master Distributed Spreadsheet System — the complete Kogi platform. |
| **Portfolio** | Kogi's root domain concept. The master spreadsheet of the independent worker's operational world. |
| **KLNK** | Kogi Link Network — the inter-portfolio graph system. All worker-to-worker and worker-to-org connections. |
| **Oba** | The Kogi platform AI assistant — the portfolio's chief of staff. Reactive, Proactive, and Autonomous modes. |
| **kogi-engine** | Scala 3 intelligence engine consuming portfolio events and writing computed signals via WritebackService. |
| **Organization** | Ume's root domain concept. The master spreadsheet of the organization's operational world. |
| **Chombo** | Ume's legal entity management subsystem. Uses N=3 Hypercube with D₃=JurisdictionAxis. |
| **Soko** | Ume's marketing system. Uses N=4 Hypercube with D₃=TimeAxis + D₄=SegmentAxis. |
| **Solution** | Qala's root domain concept and universal category. Kogi, Ume, and Qala itself are Solutions. |
| **SDE** | Solution Development Environment — Qala's atomic operational unit. Uses N=3 versioned Hypercube. |
| **CCR** | Change Control Request — formal governance record for a proposed solution change. Lattice CRDT on status. |
| **Root Factory** | The Qala factory governing the Apapo platform. Kogi and Ume are Solutions in the Root Factory. |
| **Domain-Layer Pattern** | The four-layer architecture: Presentation → Domain Logic → Domain-Hypergrid Bridge → Hypergrid Substrate. |

---

*End of Document*

*Apapo Platform · Master Design Document · v1.0 · March 2026*

*Hypergrid N-Dimensional Distributed Spreadsheet Substrate*  
*Kogi Independent Worker OS · Ume Business OS · Qala Solution Factory OS*

*Confidential — Internal Use Only*
