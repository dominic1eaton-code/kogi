

**HYPERGRID**

**N-Dimensional Distributed Spreadsheet System**

**Platform-Agnostic Software Design Document**

Version 1.0  ·  March 2026

*Rows · Columns · N Custom Dimensions · N-Attribute Cells · Graphs & Networks*

*Spaces · Workspaces · Namespaces · Multi-Identity · CRDT · Link Forest · AI Intelligence*

| Attribute | Value |
| :---- | :---- |
| System Name | Hypergrid — N-Dimensional Distributed Spreadsheet System |
| Acronym | NDSS / Hypergrid |
| Version | 1.0 |
| Status | Active Design — Reference Architecture |
| Audience | Platform architects, system engineers, product teams adopting Hypergrid |
| Primary Languages | Rust (core substrate) · Go (services) · TypeScript (SDK/UI) · SQL (PostgreSQL) |
| Core Modules | HG-CORE · HG-DIM · HG-CELL · HG-VIEW · HG-COMP · HG-CRDT · HG-GRAPH · HG-SPACE · HG-ID · HG-NS · HG-EXPORT · HG-AI |
| License | Apache 2.0 (reference implementation) · Commercial license available |

# **Table of Contents**

# **1\.  Overview & Design Philosophy**

## **1.1  What is Hypergrid?**

Hypergrid is a platform-agnostic, open-architecture, N-dimensional distributed spreadsheet system. It provides any application platform — productivity tools, project management systems, business intelligence platforms, operating systems for organizations, marketplaces, social networks, or domain-specific SaaS products — with a universal, infinitely extensible data substrate that models all entities, relationships, computations, and intelligence as a single, coherent, living grid.

A conventional spreadsheet has two dimensions: rows and columns. Hypergrid generalizes this to N dimensions: rows are Dimension 1, columns are Dimension 2, and any number of additional custom dimensions — time slices, geographic regions, organizational units, product versions, personas, risk categories, or any domain-specific axis — can be declared as Dimension 3 through N. Every cell in this N-dimensional grid is itself an N-attribute data point, carrying not just a single value but a structured map of typed attribute keys.

| *The central insight of Hypergrid: a spreadsheet is a universal model. Anything that can be described as "an entity with properties (columns), organized in groups (rows), across contexts (dimensions)" is a natural fit for a spreadsheet substrate. Hypergrid makes that substrate distributed, collaborative, intelligent, connected, and infinitely extensible — for any platform that needs it.* |
| :---- |

## **1.2  Core Abstractions**

| Abstraction | Traditional Spreadsheet | Hypergrid Generalization |
| :---- | :---- | :---- |
| Workbook | A single file containing multiple sheets | A Grid — the root container for all sheets/cubes within a tenant or namespace |
| Sheet | A 2D grid of rows × columns | A Hypercube — an N-dimensional grid: Dim1 × Dim2 × … × DimN. Every Hypercube is a view/projection of the Universal Cell Store. |
| Row | A horizontal sequence of cells, first dimension | Dimension-1 Axis — a typed, indexed, governed slice of the Hypercube along the primary entity axis |
| Column | A vertical sequence of cells, second dimension | Dimension-2 Axis — a typed, indexed, governed slice of the Hypercube along the property/attribute axis |
| Extra dims | (none) | Dimension-3…N — custom, named, typed, indexed axes: Time | Geography | OrgUnit | Version | Persona | RiskTier | Custom |
| Cell | A single value at the intersection of one row and one column | A HyperCell — a data point at the intersection of N dimension keys, carrying N typed attribute keys (value, type, formula, version, computed\_by, visibility, tags, and any domain-defined key) |
| Formula | A function over other cells in the same sheet | A ComputedAttribute — a function over any cell attributes across any dimension combination, including AI-derived signals and cross-grid joins |
| Filter | A criterion applied to rows | A DimSlice — a predicate applied to any combination of dimension axes, producing a sub-cube or flattened projection |
| Sheet tab | A named view over the same 2D grid | A HypercubeView — a saved projection, filter, sort, group, and rendering configuration over any N-dimensional sub-space |
| Workbook link | An external reference to another workbook | A GraphEdge — a typed, directed edge in the Hypergraph connecting cells, rows, or full grids across tenants, namespaces, or platforms |

## **1.3  Module Inventory**

| Module | Code | Description |
| :---- | :---- | :---- |
| Core Substrate | HG-CORE | Universal data model: Grid, Hypercube, DimensionAxis, HyperCell, HyperRow, EventLog, CrdtLog, VectorClock, PolicyEngine |
| Dimension System | HG-DIM | Declare, type, index, and govern custom dimensions. DimensionAxis registry. Sparse/dense encoding. Cross-dim join planning. |
| Cell Attribute Model | HG-CELL | N-attribute cell: attribute key registry, typed attribute values, attribute-level CRDT, computed attributes, inheritance. |
| View Engine | HG-VIEW | HypercubeView: N-dim filter (DimSlice), sort, group, pivot, projection, DimFold, board modes, rendering adapters. |
| Computation Engine | HG-COMP | ComputedAttribute evaluation: synchronous arithmetic tier and asynchronous AI/ML tier. Formula language. Rollup engine. |
| Distributed Consistency | HG-CRDT | N-dim CRDT: LWW per attribute, OR-Set for set-valued attributes, custom lattice for status axes, VectorClock per federation node. |
| Graph & Network | HG-GRAPH | Hypergraph: typed edges between HyperCells/HyperRows/Grids. Link forest and tree traversal. Shadow cell protocol. Mirror attribute sync. |
| Spaces & Workspaces | HG-SPACE | Bounded operational contexts (Spaces) and active working sessions (Workspaces). Governance. Treasury. Space-link sub-graph. |
| Identity & Multi-Tenancy | HG-ID | Multi-tenant, multi-identity, multi-profile model. TenantPartition, VisibilityMask, SplitPolicy, CrossTenantMerge. |
| Namespace System | HG-NS | Hierarchical addressing for all entities. NamespacePath URI scheme. Resolution protocol. Federation sync. |
| Export & Integration | HG-EXPORT | CSV, XLSX, JSON, JSON-LD, Parquet, Arrow, GraphQL, webhook, embed, streaming export of any N-dim view. |
| AI Intelligence Layer | HG-AI | Pluggable intelligence engines: anomaly detection, health scoring, recommendation, prediction, NL-to-query translation. |
| Query Language | HG-QL | HyperQL — N-dimensional query language: SELECT over any dim combination, DimSlice predicates, AS\_OF time travel, JOIN, SHADOW INCLUDE. |
| Plugin System | HG-PLUGIN | HypercubePlugin trait for custom dimension types, attribute types, computed models, rendering adapters, and data connectors. |
| Telemetry & Ops | HG-OPS | Distributed tracing, metrics, structured audit logs, health probes, federation sync health. |

# **2\.  N-Dimensional Data Model**

| *The N-dimensional model is the theoretical foundation of Hypergrid. Every concept — rows, columns, cells, sheets, formulas, filters, groups — is a special case of the general N-dimensional framework. Conventional 2D spreadsheet operations remain available as the most common projection of this framework.* |
| :---- |

## **2.1  Formal Definition**

Let a Hypercube H be defined by an ordered tuple of N dimension axes:

| H \= (D₁, D₂, ..., Dₙ) |
| :---- |
|  |
| where: |
|   Dᵢ \= DimensionAxis(id, name, type, key\_set, ordering, cardinality) |
|   N  \= the dimensionality of the Hypercube (N ≥ 2\) |
|  |
| A HyperCell C is a data point addressed by a coordinate tuple: |
|   C \= Cell(k₁ ∈ D₁.key\_set, k₂ ∈ D₂.key\_set, ..., kₙ ∈ Dₙ.key\_set) |
|  |
| Each HyperCell carries an attribute map: |
|   C.attributes \= { a₁: v₁, a₂: v₂, ..., aₘ: vₘ } |
|   where aᵢ ∈ AttributeKeyRegistry and vᵢ is a typed value |
|  |
| The Universal Cell Store UCS holds all HyperCells across all Hypercubes: |
|   UCS \= { (grid\_id, dim\_keys\_tuple) → HyperCell } |
|  |
| A conventional 2D spreadsheet is the N=2 special case: |
|   D₁ \= RowAxis    (entity dimension) |
|   D₂ \= ColumnAxis (attribute/property dimension) |
|   C.attributes \= { "value": V, "type": T, "format": F }  \-- conventional cell |

## **2.2  Dimension Axis Model**

Every axis of a Hypercube is a typed, indexed, governed DimensionAxis. The axis defines the key space (what values can appear in that dimension), the ordering of keys, and the cardinality (dense vs. sparse).

| Field | Type | Description |
| :---- | :---- | :---- |
| axis\_id | UUID | Globally unique identifier for this axis within the grid |
| name | String | Human-readable name: "rows", "columns", "time", "region", "org\_unit", etc. |
| dim\_index | u8 (1–255) | Position in the ordered dimension tuple. D₁=1 (rows), D₂=2 (columns), D₃–Dₙ=custom. |
| axis\_type | AxisType enum | EntityAxis | PropertyAxis | TimeAxis | GeoAxis | CategoryAxis | OrdinalAxis | HierarchyAxis | GraphAxis | Custom(String) |
| key\_type | KeyType enum | Uuid | String | Integer | Ordinal | Timestamp | GeoHash | HierarchyPath | Custom — the type of keys in this dimension |
| key\_set | KeySet | The set of valid keys for this axis. Enumerated (dense) or schema-constrained (sparse/infinite). |
| key\_cardinality | KeyCardinality | Dense(max\_keys) | Sparse | Infinite — affects storage encoding and index choice |
| ordering | AxisOrdering | Unordered | Lexicographic | Numeric | Chronological | Custom(CompareFn) — determines sort behavior across this axis |
| hierarchy | Option\<HierarchyDef\> | If AxisType=HierarchyAxis: parent-child relationships between keys. Enables group-by and drill-down across this axis. |
| visibility | AxisVisibility | Public | Tenant | Private — governs which tenants/identities can see this axis in shared grids |
| nullable | bool | If true: HyperCells need not exist for every combination including this axis (sparse). If false: every key must have a cell (dense). |
| default\_attribute | Option\<AttributeKey\> | When this axis is rendered as a 2D spreadsheet, which cell attribute is shown as the primary "value" |
| crdt\_semantics | CrdtSemantics | LWW | OR-Set | Counter | Lattice — CRDT behavior for mutations along this axis |
| index\_strategy | IndexStrategy | BTree | Hash | GIN | BRIN | TimeSeries | Spatial — storage index type for this axis |
| plugin\_id | Option\<PluginId\> | If this is a custom axis type: the HypercubePlugin that provides key validation, rendering, and computation |
| created\_at | DateTime\<Utc\> | Axis registration timestamp |
| metadata | Map\<String,Value\> | Extensible axis-level metadata for platform-specific configuration |

## **2.3  Built-in Axis Types & Their Properties**

| AxisType | Key Type | Ordering | Typical Use | Special Behaviors |
| :---- | :---- | :---- | :---- | :---- |
| EntityAxis | Uuid / String | Unordered / Custom | D₁ (rows) — primary entity dimension: users, projects, products, events, transactions | Full PortfolioRow/record model. GraphEdge::Hierarchy supported between keys. CRDT: LWW per attribute. |
| PropertyAxis | String | Lexicographic/Custom | D₂ (columns) — property/attribute dimension: field names, metric names, column labels | Column type registry (see §4). ComputedAttribute support. VisibilityMask per column key. |
| TimeAxis | Timestamp | Chronological | D₃ — temporal slicing: snapshots, time-series, event history, fiscal periods | AS\_OF time-travel queries. Automatic snapshot generation. BRIN index. Rollup by hour/day/week/month/year. |
| GeoAxis | GeoHash / Coord | Spatial | D₄ — geographic slicing: region, country, city, custom polygon, lat/lng bucket | Spatial index (PostGIS/GeoHash). Drill-down: continent → country → region → city. Spatial join support. |
| CategoryAxis | String (enum) | Ordinal / Custom | D₃–N — classification: product line, risk tier, department, scenario, cohort | Enum key validation. Hierarchy support. Cross-category pivot. |
| OrdinalAxis | Integer | Numeric | D₃–N — ranked/ordered slicing: priority, ranking, version number, sequence | Dense numeric key space. Efficient range queries. Used for versioned cubes. |
| HierarchyAxis | HierarchyPath | Tree (depth-first) | D₃–N — org chart, product taxonomy, file system, tag hierarchy, geographic admin levels | Parent-child key relationships. Rollup propagation. Drill-down/drill-up operations. |
| GraphAxis | NodeId | Graph-order / Custom | D₃–N — graph traversal dimension: network hops, dependency layers, influence tiers | KLNK/Hypergraph integration. Keys are graph node IDs. Enables adjacency matrix representations. |
| VersionAxis | SemVer / Integer | Numeric | D₃ — schema/data versioning: parallel versions of the same cube across schema migrations | Fork-and-merge version operations. Diff between version keys. Schema migration tracking. |
| TenantAxis | TenantId | Unordered | D₃ — multi-tenancy: share a single Hypercube across tenants while enforcing RLS per key | Row-level security: each key visible only to matching tenant. Used for SaaS shared-cube deployments. |
| PersonaAxis | String | Custom | D₃–N — audience/persona slicing: user segments, role types, maturity levels, view contexts | A/B testing support. Personalized ComputedAttributes. Feed personalization. |
| ScenarioAxis | String | Custom | D₃–N — what-if and planning: base, optimistic, pessimistic, budget, forecast, actuals | Scenario comparison views. Variance computed across scenario keys. Plan-vs-actual analysis. |
| Custom | Defined by plugin | Plugin-defined | Any domain-specific axis not covered above: genome position, musical note, physics parameter | HypercubePlugin provides key\_type, validation, index, rendering, and computation handlers |

## **2.4  Dimensionality Constraints**

| Constraint | Value | Rationale |
| :---- | :---- | :---- |
| Minimum dimensions (N) | 2 (D₁=EntityAxis, D₂=PropertyAxis) | 2D is the degenerate case: a conventional spreadsheet. All Hypergrid operations are valid at N=2. |
| Maximum dimensions (N) | 16 (hard limit) | Beyond 16 dims, query planning complexity exceeds practical utility. Recommended max: 6 for most use cases. |
| Max keys per dense axis | 10,000,000 (10M) | Dense axes are fully materialized. Beyond 10M keys, use SparseAxis with on-demand materialization. |
| Max keys per sparse axis | Unlimited (constrained by storage) | Sparse axes store only cells with non-null attributes. Storage: O(non-null cells). |
| Max attributes per HyperCell | 1,024 | Attribute key registry limit per grid. In practice: 20–150 attributes per cell are typical. |
| Max concurrent editors per cube | 1,000 | CRDT buffer capacity. Beyond 1K concurrent writers: federation node sharding recommended. |
| Max federation peers per grid | 255 | VectorClock size. Each peer is one slot in the VectorClock map. |
| Max custom dimensions added at runtime | Unlimited (up to N=16 total) | Custom axes can be added to any Hypercube post-creation. Existing cells gain the new axis with null/default attribute values. |

## **2.5  Dimensional Sparsity & Encoding**

Hypergrid supports two storage encodings for the cell store, chosen at axis declaration time based on the axis's expected density:

| Encoding | When To Use | Storage Schema | Query Behavior |
| :---- | :---- | :---- | :---- |
| Dense Encoding | Axis is small and fully populated. Every (D₁×D₂) row-column pair is expected to have a value. N=2 conventional spreadsheets always use dense encoding for D₁×D₂. | N-dim array storage: PostgreSQL ARRAY or columnar layout. O(D₁.card × D₂.card × …) cells allocated. | O(1) cell lookup by coordinate. Range scans are contiguous. Aggregations are SIMD-friendly. |
| Sparse Encoding | Axis is large or partially populated. Most coordinate combinations have no meaningful value (null). Any axis with cardinality \> 100,000 keys or TimeAxis/GeoAxis/GraphAxis. | Key-value storage: {(k₁, k₂, …, kₙ): attribute\_map} in a hash-indexed PostgreSQL table. Only non-null cells stored. | O(log n) cell lookup by coordinate. Range scans require index traversal. Best for time-series and geo. |
| Hybrid Encoding | D₁×D₂ are dense; D₃–Dₙ are sparse. The most common production pattern: rows and columns are dense, custom time/geo/category axes are sparse. | Dense 2D layout for D₁×D₂ slice; sparse key-value index for D₃–Dₙ extension cells. | Fast conventional 2D access \+ efficient N-dim extension queries. Recommended for most use cases. |

# **3\.  HyperCell & N-Attribute Model (HG-CELL)**

| *A HyperCell is not a single value. It is an N-attribute data point — a structured map of typed attribute keys located at a specific coordinate in the N-dimensional grid. Conventional spreadsheet cells have 3 implicit attributes: value, type, and format. HyperCells generalize this to N attributes, each with its own type, visibility, computation, versioning, and CRDT semantics.* |
| :---- |

## **3.1  HyperCell Data Model**

| pub struct HyperCell { |
| :---- |
|     // Coordinate — uniquely identifies this cell in the N-dim grid |
|     pub grid\_id:       GridId,          // parent Grid |
|     pub cube\_id:       CubeId,          // parent Hypercube |
|     pub coord:         DimCoordinate,   // Vec\<DimKey\> — one key per dimension |
|     // coord example for N=4: (entity\_id, column\_name, "2026-Q1", "EMEA") |
|  |
|     // Attributes — the N-attribute data payload |
|     pub attributes:    AttributeMap,    // HashMap\<AttributeKey, TypedAttrValue\> |
|  |
|     // Versioning |
|     pub vector\_clock:  VectorClock,     // causal ordering across federation nodes |
|     pub version:       u64,             // monotonic mutation counter |
|     pub created\_at:    DateTime\<Utc\>, |
|     pub updated\_at:    DateTime\<Utc\>, |
|     pub last\_actor:    ActorId,         // "{tenant\_id}:{identity\_tag}:{node\_id}" |
|  |
|     // Governance |
|     pub visibility:    CellVisibility,  // Public|Tenant|Identity|Private |
|     pub policy\_ids:    Vec\<PolicyId\>, |
|     pub audit\_trail:   EventLogRef,     // pointer to EventLog entry sequence |
|  |
|     // Graph integration |
|     pub edge\_refs:     Vec\<EdgeRef\>,    // edges originating from this cell |
|  |
|     // Computed attribute cache |
|     pub computed\_cache: HashMap\<AttributeKey, CachedComputedAttr\>, |
| } |
|  |
| pub struct DimCoordinate { |
|     pub keys: Vec\<DimKey\>,  // ordered: \[D1\_key, D2\_key, ..., DN\_key\] |
|     pub n:    u8,           // dimensionality — must match parent Hypercube.N |
| } |
|  |
| pub struct AttributeMap { |
|     pub entries: HashMap\<AttributeKey, TypedAttrValue\>, |
|     // AttributeKey: String — registered in the cube's AttributeKeyRegistry |
|     // TypedAttrValue: Text|Number|Currency|Bool|Date|Enum|Relation|Tag|Json|Null |
| } |

## **3.2  Attribute Key Registry**

Every attribute key used in any HyperCell must be registered in the Hypercube's AttributeKeyRegistry. The registry defines the type, default value, computation, visibility, and CRDT semantics for each attribute. This is the equivalent of a "column schema" generalized to apply across all dimensions.

| Field | Type | Description |
| :---- | :---- | :---- |
| key | String | The attribute key name. Unique within the Hypercube. E.g. "value", "health\_score", "owner", "tags", "budget", "status" |
| display\_name | String | Human-readable label shown in UI headers and export column names |
| attr\_type | AttributeType enum | Text | Number | Currency | Percent | Bool | Date | DateTime | Duration | Enum | MultiEnum | Relation | MultiRelation | User | Tag | Json | Computed | AI | Formula | Audit | Custom |
| description | String | Documentation string for this attribute key |
| dimension\_scope | DimScope | AllDimensions | SpecificAxes(Vec\<AxisId\>) — which dimension axes this attribute applies to. Allows axis-specific attributes. |
| default\_value | Option\<TypedAttrValue\> | Default value when this attribute is not explicitly set on a HyperCell |
| required | bool | If true: cells in the in-scope dimension must have this attribute set |
| visibility | AttrVisibility | Public | Tenant | Identity | Owner — which observers can read this attribute |
| write\_permission | PermissionTier | Minimum permission tier required to write this attribute. "Computed" and "AI" attrs are system-write-only. |
| crdt\_semantics | CrdtSemantics | LWW | OR-Set | Counter | Max | Min | Lattice(order\_def) — conflict resolution strategy |
| computation | Option\<AttrComputation\> | If attr\_type=Computed: synchronous formula expression. If attr\_type=AI: engine plugin reference \+ model name. |
| aggregation | Option\<Vec\<AggFn\>\> | Supported aggregation functions: SUM | AVG | MIN | MAX | COUNT | MEDIAN | STDDEV | MODE | DISTRIBUTION |
| index\_strategy | IndexStrategy | None | BTree | Hash | GIN — storage index for this attribute key |
| inherits\_from | Option\<AttributeKey\> | Attribute inheritance: if not set on a HyperCell, inherit from parent in hierarchy axis (HierarchyAxis types only) |
| version\_track | bool | If true: maintain full history of this attribute's values in EventLog. If false: only current value stored. |
| searchable | bool | If true: indexed by the full-text search engine |
| export\_label | Option\<String\> | Override label used in CSV/XLSX/JSON export headers for this attribute |
| plugin\_id | Option\<PluginId\> | For Custom attribute types: the plugin providing validation, rendering, and computation |

## **3.3  Core Attribute Key Set (Built-in)**

Every Hypercube includes the following built-in attribute keys on every HyperCell unless explicitly disabled:

| Attribute Key | Type | Description | CRDT | Visibility |
| :---- | :---- | :---- | :---- | :---- |
| value | Any (configured per D₂ column key) | The primary data value — the "cell value" in conventional spreadsheet terms. Type and computation determined by D₂ column key registration. | LWW | Inherits cell visibility |
| data\_type | Enum (AttributeType) | The runtime type of value. Allows heterogeneous cells in sparse grids. | LWW | Public |
| format | String (format pattern) | Display format string: "0.00%", "$\#,\#\#\#.00", "YYYY-MM-DD", "\#,\#\#\#", etc. | LWW | Public |
| formula | Option\<String\> | Formula expression if this cell is formula-driven (FormulaAttribute). Null for data cells. | LWW | Tenant |
| computed\_by | Option\<String\> | If type=AI|Computed: the plugin/model that computed this value. | System | Tenant |
| computed\_at | Option\<DateTime\> | Timestamp of last computation. | System | Tenant |
| confidence | Option\<f32\> | AI-computed confidence score (0.0–1.0) for computed attributes. | System | Tenant |
| version | u64 | Monotonic mutation counter for this specific attribute on this cell. | Counter (max) | Public |
| visibility | CellVisibility | Per-cell visibility override. Default: inherits from row then cube default. | LWW | Owner |
| locked | bool | If true: no mutations permitted by non-owner actors. Used for audit-locked records. | LWW (Owner only) | Public |
| tags | Vec\<String\> | User-defined discovery tags on this specific cell (rare; more common on row-level cells). | OR-Set | Inherits |
| comment\_count | u32 | Count of active comments on this cell (annotation system integration). | Counter | Tenant |
| change\_highlight | Option\<ChangeType\> | UI signal: Added|Modified|Deleted|Imported — shown as color indicator in current session. | Session-local | Session |
| error | Option\<CellError\> | Formula or computation error state: DivisionByZero|CircularRef|RefError|ComputeError|etc. | LWW | Tenant |
| anomaly\_flag | Option\<AnomalySignal\> | AI-detected anomaly on this cell: OutlierValue|SuddenChange|PatternBreak|etc. | AI-write | Tenant |
| last\_modified\_by | ActorId | Actor who last modified this cell's value attribute. | LWW | Tenant |
| modified\_at | DateTime | Timestamp of last value modification. | LWW | Public |
| source | Option\<String\> | Data provenance: "manual", "import:csv", "api:service-name", "computed:model-name", "sync:peer-node" | LWW | Tenant |
| dim\_context | Map\<AxisId, ContextMeta\> | Per-dimension context metadata for this cell. Allows cells to carry axis-specific annotations beyond the coordinate. | LWW per axis | Tenant |

## **3.4  Domain Attribute Extensions**

Platforms adopting Hypergrid define domain-specific attribute key extensions for their specific use cases. These are registered in the AttributeKeyRegistry and follow all the same CRDT, visibility, and computation rules as built-in attributes. Example domain extension sets:

| Domain | Extension Set Name | Example Attribute Keys Added |
| :---- | :---- | :---- |
| Project Management | PM Extension | status, priority, assignee, due\_date, story\_points, sprint\_ref, acceptance\_criteria, cycle\_time, lead\_time, blocked\_by |
| Financial / ERP | Finance Extension | budget\_allocated, budget\_spent, currency, cost\_center, gl\_account, tax\_category, approval\_status, payment\_terms, invoice\_ref |
| CRM | CRM Extension | lead\_score, pipeline\_stage, close\_probability, deal\_value, company\_ref, contact\_ref, last\_contacted\_at, next\_action\_due |
| HR / People Ops | People Extension | employment\_type, department, start\_date, end\_date, compensation, performance\_rating, skills, reporting\_to, benefit\_tier |
| Product / Engineering | Eng Extension | ticket\_type, component, severity, environment, repro\_steps, fix\_version, test\_coverage, code\_owner, sla\_tier |
| Scientific Research | Research Extension | experiment\_id, hypothesis, measurement\_unit, uncertainty, confidence\_interval, replications, p\_value, citation\_refs |
| Healthcare | Clinical Extension | patient\_id, diagnosis\_code, procedure\_code, provider\_id, encounter\_date, dosage, clinical\_status, privacy\_tier |
| Supply Chain | SCM Extension | sku, supplier\_id, lead\_time\_days, safety\_stock, reorder\_point, lot\_number, expiry\_date, warehouse\_location |
| Marketing | Marketing Extension | campaign\_id, channel, impressions, clicks, conversions, cpa, roas, audience\_segment, creative\_ref, a\_b\_variant |
| Compliance / Legal | Legal Extension | regulation\_ref, obligation\_type, control\_owner, last\_audit\_date, risk\_rating, remediation\_due, evidence\_refs |

# **4\.  Grid & Hypercube Architecture (HG-CORE)**

## **4.1  Grid — The Root Container**

A Grid is the top-level container for all Hypercubes, dimensions, attribute registries, tenants, namespaces, spaces, workspaces, and graph structures within a logical deployment unit. It is equivalent to a database in the RDBMS analogy, a workbook in the 2D spreadsheet analogy, or a portfolio system in a specific platform implementation.

| pub struct Grid { |
| :---- |
|     pub grid\_id:            GridId,              // UUID — globally unique |
|     pub name:               String, |
|     pub description:        Option\<String\>, |
|     pub tenant\_id:          TenantId,            // top-level tenancy boundary |
|     pub namespace\_root:     NamespacePath,       // root of this grid's NS tree |
|     pub cubes:              Vec\<CubeId\>,         // registered Hypercubes |
|     pub dim\_registry:       DimensionRegistry,   // all axes defined for this grid |
|     pub attr\_registry:      AttributeKeyRegistry,// all attribute keys for this grid |
|     pub graph:              HypergraphRef,        // this grid's node/edge store |
|     pub spaces:             Vec\<SpaceId\>,        // Spaces within this grid |
|     pub federation\_peers:   Vec\<FederationPeer\>, // CRDT peers for this grid |
|     pub policy\_engine:      PolicyEngineRef, |
|     pub ai\_engine:          Option\<AIEngineRef\>, // pluggable AI/ML engine |
|     pub storage\_config:     StorageConfig, |
|     pub created\_at:         DateTime\<Utc\>, |
|     pub metadata:           Map\<String, Value\>,  // platform-defined extensions |
| } |

## **4.2  Hypercube — The N-Dimensional Sheet**

| pub struct Hypercube { |
| :---- |
|     pub cube\_id:           CubeId, |
|     pub grid\_id:           GridId, |
|     pub name:              String,               // display name (e.g. "Projects") |
|     pub namespace\_path:    NamespacePath,        // NS address: grid://tenant/cube/name/ |
|     pub dimensions:        Vec\<DimensionAxis\>,   // ordered D₁…Dₙ |
|     pub n:                 u8,                   // dimensionality (2–16) |
|     pub schema\_version:    u32,                  // incremented on dim/attr add |
|     pub encoding:          CubeEncoding,         // Dense | Sparse | Hybrid |
|     pub cell\_count:        u64,                  // non-null cells |
|     pub default\_view\_id:   Option\<ViewId\>, |
|     pub views:             Vec\<ViewId\>, |
|     pub attr\_overrides:    AttrKeyOverrides,     // cube-level attribute config overrides |
|     pub governance:        CubeGovernance,       // access, approval, audit settings |
|     pub space\_id:          Option\<SpaceId\>,      // parent Space (if Space-scoped) |
|     pub status:            CubeStatus,           // Draft|Active|Archived|Migrating |
|     pub created\_at:        DateTime\<Utc\>, |
|     pub updated\_at:        DateTime\<Utc\>, |
|     pub metadata:          Map\<String, Value\>, |
| } |

## **4.3  Dimension Addition & Schema Migration**

One of Hypergrid's most powerful features is the ability to add new dimensions to an existing Hypercube at runtime without downtime or data migration. When a new axis Dₙ₊₁ is added to a Hypercube with existing cells:

1. Schema Migration: A SchemaVersion bump is recorded. A SchemaEvent is appended to the EventLog.

2. Default Key Assignment: All existing HyperCells receive a default key for the new axis. The default is configured in the DimensionAxis declaration (e.g. "unclassified" for a CategoryAxis, "2026-Q1" for a TimeAxis).

3. Sparse Handling: If the new axis uses Sparse encoding, no cells are physically created. Existing cells simply have no explicit entry for the new axis — queries that include the new axis treat these cells as having the default key.

4. Index Provisioning: The appropriate index is provisioned asynchronously (background job). The cube remains fully queryable during provisioning via a pre-existing full-scan fallback.

5. View Updates: All existing ViewDefinitions are updated to include the new axis as a collapsed (default-value) DimSlice. Existing views appear unchanged to users until they explicitly expand the new axis.

6. CRDT Broadcast: A DimensionAdded CrdtOperation is broadcast to all federation peers. Peers apply the same migration to their replicas.

| Migration Type | Description | Downtime? | Reversible? |
| :---- | :---- | :---- | :---- |
| Add Dimension | Add a new axis Dₙ₊₁ to a Hypercube. All existing cells get default key. | None | Yes (remove axis — only if all cells still have default key) |
| Remove Dimension | Remove axis Dₖ. Collapses that dimension: all cells at different keys for Dₖ are merged (conflict resolution required). | \<1 min | No — destructive. Snapshot recommended before removal. |
| Reorder Dimensions | Change the index order of dimensions. Primarily affects storage layout and query planning. | None | Yes |
| Rename Axis | Change the name or display\_name of an axis. Does not affect data. | None | Yes |
| Change Key Type | Change the key\_type of an axis (e.g. String → Uuid). Requires key migration for all existing cells. | \<5 min | Yes (if key format compatible) |
| Add Attribute Key | Register a new attribute key in the AttributeKeyRegistry. Existing cells gain attribute with default value. | None | Yes (if no required=true) |
| Remove Attribute Key | Unregister an attribute key. Values for this key are stripped from all cells. | \<1 min | No — destructive. EventLog preserves history. |
| Change Encoding | Migrate from Dense to Sparse or vice versa for a specific axis. | \<15 min | Yes |

## **4.4  HyperRow — The Canonical Row Entity**

While any D₁ key in an EntityAxis is technically a "row", Hypergrid provides the HyperRow abstraction as the canonical first-class entity in a Hypercube. A HyperRow represents the full slice of the Hypercube along the D₁ axis for a given D₁ key — all its D₂ column values and all its D₃–Dₙ extension cells.

| pub struct HyperRow { |
| :---- |
|     pub row\_id:         DimKey,          // D₁ key for this row |
|     pub cube\_id:        CubeId, |
|     pub grid\_id:        GridId, |
|  |
|     // D₂ cells — conventional "columns" for this row |
|     pub cells:          HashMap\<D2Key, HyperCell\>, |
|  |
|     // D₃–Dₙ extension cells — additional dimensions |
|     pub ext\_cells:      HashMap\<ExtCoord, HyperCell\>, |
|     // ExtCoord \= (D3\_key, ..., DN\_key) — the non-D1/D2 coordinate portion |
|  |
|     // Row-level metadata (shared across all cells in this row) |
|     pub row\_attrs:      AttributeMap,    // row-level attributes |
|     pub vector\_clock:   VectorClock, |
|     pub version:        u64, |
|     pub visibility:     RowVisibility,   // Public|Tenant|Identity|Space|Private |
|     pub space\_context:  Option\<SpaceContext\>, |
|     pub graph\_node\_id:  Option\<NodeId\>,  // this row's node in HG-GRAPH |
|     pub namespace\_path: Option\<NamespacePath\>, |
|     pub identity\_tags:  Vec\<IdentityTag\>, |
|     pub event\_log\_ref:  EventLogRef, |
| } |
|  |
| // Row-level attribute keys are registered separately from cell-level keys: |
| // Row-level attrs describe the entity itself (id, name, status, owner, tags, ...) |
| // Cell-level attrs describe specific data points (value, format, formula, ...) |

# **5\.  N-Dimensional Operations**

## **5.1  Cell Addressing — Coordinate System**

Every HyperCell is addressed by a DimCoordinate — an ordered tuple of N dimension keys, one per registered axis. Operations that address cells must provide a coordinate with exactly N keys, one per axis, in dimension order (D₁ first, Dₙ last).

| // N=2 (conventional spreadsheet) |
| :---- |
| cell\[(project\_uuid, "budget\_allocated")\] |
|  |
| // N=3 (+ time dimension) |
| cell\[(project\_uuid, "budget\_spent", "2026-Q2")\] |
|  |
| // N=4 (+ geo dimension) |
| cell\[(product\_sku, "revenue", "2026-Q2", "EMEA")\] |
|  |
| // N=5 (+ scenario dimension) |
| cell\[(product\_sku, "revenue", "2026-Q2", "EMEA", "forecast")\] |
|  |
| // N=6 (+ org unit dimension) |
| cell\[(employee\_id, "performance\_rating", "2026-H1", "US-WEST", "base\_case", "Engineering")\] |
|  |
| // Wildcard / slice — all values on one axis |
| cell\[(project\_uuid, "budget\_spent", \*)\]    \-- all time periods for this project+attr |
| cell\[(\*, "revenue", "2026-Q2", "EMEA")\]    \-- all products in EMEA in Q2 2026 |

## **5.2  DimSlice — N-Dimensional Filtering**

A DimSlice is the generalization of a row filter to N dimensions. It defines predicates on any subset of axes, producing a sub-cube (if all non-predicate axes are retained) or a projection (if some axes are collapsed/aggregated).

| DimSlice Type | Description | Example | Result |
| :---- | :---- | :---- | :---- |
| Point Slice | Exact value on one axis. Equivalent to a single row lookup (D₁) or column lookup (D₂). | D₁ \= project\_id\_42 | All cells for that single project across all other axes |
| Range Slice | Range of values on an ordered axis. Efficient on BTree or TimeSeries indexes. | D₃ (Time) BETWEEN 2026-01-01 AND 2026-06-30 | All cells in H1 2026 across all rows and columns |
| Set Slice | A set of specific values on any axis. | D₄ (Region) IN \["EMEA", "APAC"\] | All cells in EMEA and APAC only |
| Predicate Slice | A computed condition on any attribute of cells at a given axis coordinate. | D₂ \= "health\_score" AND value \> 80 | All rows whose health\_score \> 80 |
| Hierarchical Slice | Drill-down into a HierarchyAxis: select a node and all its descendants. | D₅ (OrgUnit) UNDER "Engineering" | All cells in Engineering \+ all sub-units |
| Cross-Dim Slice | Predicates on multiple axes simultaneously. Produces the intersection sub-cube. | D₁ \= active\_projects AND D₃ \= "2026-Q2" | Active projects in Q2 2026 only |
| DimFold | Collapse an axis by aggregating all values along that axis into a single summary. | FOLD D₃ (Time) WITH SUM(value) | Flattened cube without time axis; values summed across all time periods |
| DimExpand | Expand a previously folded or defaulted axis into its full key set. | EXPAND D₃ showing quarterly breakdown | Cube split into separate slices per quarter |
| NullSlice | Filter to cells where a specific axis has no explicit value (sparse cells with default key). | D₄ IS NULL | Cells not yet assigned a geo region |
| ShadowSlice | Include cells from linked (Shadow) grids in results alongside owned cells. | INCLUDE SHADOW CELLS FROM grid://partner/ | Own cells \+ shadow-linked cells from specified grid |

## **5.3  DimFold & DimExpand — Aggregation Across Axes**

DimFold and DimExpand are the N-dimensional generalizations of GROUP BY and PIVOT. They allow any axis to be collapsed into a summary (DimFold) or a previously collapsed axis to be re-expanded into its constituent keys (DimExpand).

| \-- DimFold: collapse D₃ (Time) by summing revenue, for a revenue cube |
| :---- |
| SELECT D₁.entity\_name, D₂."revenue" |
| FROM revenue\_cube |
| WHERE D₄.region \= "EMEA" |
| FOLD D₃ WITH SUM(value)    \-- time axis collapsed, revenue summed across all periods |
| ORDER BY D₁.entity\_name; |
|  |
| \-- DimExpand: pivot D₄ (Region) into separate attribute columns per region |
| SELECT D₁.entity\_name, D₃.quarter, |
|        EXPAND D₄ AS COLUMNS(value)   \-- one column per region key |
| FROM revenue\_cube |
| WHERE D₃.quarter IN ("2026-Q1", "2026-Q2"); |
| \-- Result: entity\_name | quarter | EMEA.revenue | APAC.revenue | NA.revenue | ... |
|  |
| \-- DimFold \+ DimExpand (full N-dim pivot) |
| SELECT D₁.entity\_name, |
|        EXPAND D₄ AS COLUMNS(SUM(value)) |
| FROM revenue\_cube |
| FOLD D₃ WITH SUM(value) |
| GROUP BY D₁.entity\_name, D₄.region; |
| \-- Result: entity\_name | EMEA\_total | APAC\_total | NA\_total | ... |

## **5.4  Computed Attributes Across Dimensions**

ComputedAttributes (formula and AI attributes) in Hypergrid can reference cells across any dimension combination, not just within the same row. This enables cross-dimensional computations that would be impossible in a conventional spreadsheet:

| Computation Pattern | Description | Formula Example | Use Case |
| :---- | :---- | :---- | :---- |
| Intra-cell | Computed from other attributes of the same cell | profit \= revenue \- expenses | Standard derived field |
| Intra-row, same dim | Computed from other D₂ column attributes in the same D₁ row | budget\_remaining \= budget\_allocated \- budget\_spent | Row-level derived metric |
| Cross-D₁ (aggregate) | Aggregate across D₁ rows for the same D₂ column key | avg\_health \= AVG(SLICE(D₂="health\_score")) | Column summary |
| Cross-D₃ (time) | Compute a time-series aggregation across the D₃ time axis | ytd\_revenue \= SUM(SLICE(D₂="revenue", D₃ BETWEEN ytd\_start AND today)) | YTD/rolling window KPIs |
| Cross-D₄ (geo) | Aggregate across D₄ geo axis for a specific row+column | global\_sales \= SUM(SLICE(D₂="sales", D₄=\*)) | Geographic aggregation |
| Cross-dim pivot | Compute a ratio between values at different dimension key combinations | emea\_pct \= CELL(D₁,D₂,"revenue","EMEA") / global\_sales | Market share / contribution |
| DimFold formula | Apply a formula that itself performs a DimFold as part of its computation | rolling\_avg\_4q \= AVG(FOLD D₃ LAST 4 QUARTERS) | Rolling window analytics |
| Graph-traversal | Aggregate values across graph-connected cells in HG-GRAPH | total\_dependencies \= SUM(GRAPH\_NEIGHBORS(D₁=row\_id, edge\_type="DependsOn", attr="budget")) | Network-aware aggregation |
| Cross-grid join | Compute from attribute values in a different linked Grid | ext\_data \= JOIN(GRID("partner://grid"), D₁=row\_id, attr="market\_rate") | Cross-grid reference |
| Shadow cell ref | Reference a value from a shadow cell (linked from another tenant's grid) | partner\_health \= SHADOW\_CELL(grid\_id, coord, "health\_score") | Cross-tenant data reference |

## **5.5  N-Dimensional Aggregation Engine**

The aggregation engine must be capable of evaluating aggregation functions across any combination of axes. The following aggregation operations are supported in any dimension combination:

| Operation | Axis Types | Description | HyperQL Syntax |
| :---- | :---- | :---- | :---- |
| SUM | All numeric | Sum of value attribute across matching cells | SUM(SLICE(D₂="revenue", D₃="2026")) |
| AVG | All numeric | Mean average across matching cells | AVG(SLICE(D₂="health\_score")) |
| COUNT | All | Count of non-null cells matching the slice | COUNT(SLICE(D₄="EMEA")) |
| COUNT\_UNIQUE | All | Count of distinct values for a given attribute across a slice | COUNT\_UNIQUE(SLICE(D₂="owner")) |
| MIN / MAX | Numeric \+ Date | Minimum or maximum value across a slice | MIN(SLICE(D₂="due\_date", D₁=active\_rows)) |
| MEDIAN | Numeric | Median value across a slice | MEDIAN(SLICE(D₂="cycle\_time")) |
| PERCENTILE | Numeric | Arbitrary percentile: PERCENTILE(attr, p) | PERCENTILE(SLICE(D₂="salary"), 0.90) |
| STDDEV / VAR | Numeric | Standard deviation / variance across a slice | STDDEV(SLICE(D₂="lead\_time")) |
| DISTRIBUTION | Enum / Category | Frequency distribution of enum values across a slice | DISTRIBUTION(SLICE(D₂="status")) |
| FOLD\_SUM | N-dim (any axis) | DimFold \+ SUM: collapse an axis by summing the value attribute | FOLD D₃ WITH SUM(value) |
| FOLD\_AVG | N-dim (any axis) | DimFold \+ AVG: collapse an axis by averaging the value attribute | FOLD D₃ WITH AVG(value) |
| ROLLUP | HierarchyAxis only | Bottom-up hierarchical aggregation: leaf nodes sum to parent nodes | ROLLUP(D₅, attr="budget\_allocated") |
| DRILLDOWN | HierarchyAxis only | Top-down hierarchical decomposition: show contribution of each child | DRILLDOWN(D₅ UNDER "Engineering", attr="headcount") |
| MOVING\_AVG | TimeAxis only | Moving window average: AVG of last N time periods for each D₁ entity | MOVING\_AVG(D₃, window=4, attr="revenue") |
| CUMULATIVE | TimeAxis only | Cumulative sum up to each time period: prefix sum along time axis | CUMULATIVE(D₃ FROM ytd\_start, attr="expenses") |
| GRAPH\_SUM | GraphAxis \+ any | Sum across graph-connected nodes: aggregate neighbors in HG-GRAPH | GRAPH\_SUM(D₁=row\_id, edge\_type="DependsOn", attr="budget") |

# **6\.  Spaces, Workspaces & Namespaces (HG-SPACE · HG-NS)**

| *Spaces, Workspaces, and Namespaces are the three spatial organization layers of Hypergrid. They are platform-agnostic — any platform using Hypergrid inherits these primitives and can extend them for their specific domain (projects, organizations, products, research labs, events, etc.).* |
| :---- |

## **6.1  Space — Governed Bounded Context**

A Space is a named, governed, bounded operational context within a Grid. It groups related Hypercubes, member identities, Workspaces, communication channels, and governance structures into a cohesive unit. A Space is itself a HyperRow in the system's reserved Spaces Hypercube (cube\_id: "system.spaces"), giving it all the versioning, CRDT, graph, and AI capabilities of any other entity.

| Space Property | Type | Description |
| :---- | :---- | :---- |
| space\_id | UUID | Globally unique. Also the D₁ key for this Space's row in the Spaces Hypercube. |
| space\_type | SpaceType | Platform-defined enum. Examples: Personal | Team | Org | Cooperative | Community | Federation | Project | Research | Event | Product | Studio | Tenant | Custom(String) |
| slug | String | URL-safe unique handle within parent namespace. Used in NamespacePath. |
| namespace\_path | NamespacePath | Full hierarchical address: hypergrid://{tenant}/{type}/{slug}/ |
| grid\_id | GridId | Parent Grid this Space belongs to. |
| member\_roster | Vec\<SpaceMember\> | Members with roles. Role \= PermissionTier \+ Space-specific role label. |
| cubes | Vec\<CubeId\> | Hypercubes scoped to this Space (Space-owned data). |
| shared\_cubes | Vec\<CubeId\> | Hypercubes shared with but not owned by this Space (from other Spaces via ShadowRow links). |
| workspaces | Vec\<WorkspaceId\> | Active Workspaces within this Space. |
| governance\_config | GovernanceConfig | Vote thresholds, quorum rules, proposal types, treasury policy, multi-sig config. |
| treasury\_ref | Option\<TreasuryRef\> | Treasury account for economic Spaces. Platform connects to its financial layer. |
| link\_node\_id | NodeId | This Space's node in HG-GRAPH. All member identities link here via SpaceMembership edges. |
| visibility | Visibility | Private | Tenant | Internal | Public | Unlisted |
| status | SpaceStatus | Draft | Active | Paused | Restricted | Dissolving | Dissolved | Archived |
| federation\_links | Vec\<FedLinkId\> | Cross-Space and cross-Grid federation links. |
| plugin\_config | Map\<String,Value\> | Platform-specific Space configuration registered via HG-PLUGIN. |
| metadata | AttributeMap | Extensible platform-defined attributes for this Space. |

## **6.2  Space Type Taxonomy (Platform-Configurable)**

The SpaceType enum is configurable per Grid deployment. The following types are provided as reference defaults — platforms register their own types via HG-PLUGIN:

| Default Space Type | Description | Auto-provisioned Cubes | Governance Default |
| :---- | :---- | :---- | :---- |
| Personal | One-per-identity. Private by default. All identity's private grids, workspaces, and data. | SystemCubes \+ all cubes owned by this identity | Owner only |
| Team | Collaborative working unit. Members share cubes and workspaces. | Shared work cube \+ team kanban \+ member directory | Team lead \+ majority |
| Organization | Formal legal entity. Full governance, treasury, and member management. | Org portfolio \+ governance \+ members \+ finances | Org governance model (varies) |
| Community | Open or semi-open topical group. Content-sharing, discussion, collaboration. | Community content cube \+ member directory | Admin \+ community vote |
| Federation | Meta-Space linking multiple Organizations/Spaces for joint governance. | Federation portfolio \+ cross-space governance cube | Representative/federal |
| Project | Time-bounded project context. Lifecycle tied to project start/end dates. | Project tasks \+ timeline \+ resources \+ budget | Project owner |
| Research | Academic/scientific/investigative collaboration with structured outputs. | Research data cube \+ publications \+ experiments | Principal investigator |
| Event | Time-bounded event context. Auto-archives after event lifecycle ends. | Event schedule \+ attendees \+ session recordings | Event host |
| Tenant | Top-level tenancy boundary for SaaS multi-tenancy scenarios. | All tenant-owned cubes | Tenant admin |
| Custom | Platform-defined type registered via HG-PLUGIN with custom governance, cube provisioning, and lifecycle logic. | Plugin-defined | Plugin-defined |

## **6.3  Workspace — Active Working Session**

A Workspace is the active, personalized working context within a Space. It records the user's current arrangement of open Hypercubes and views, pinned rows, active session state, tool configuration, and AI assistant context. Workspaces are ephemeral in feel but durable in storage — all state is persisted in the system Workspaces Hypercube (cube\_id: "system.workspaces").

| pub struct Workspace { |
| :---- |
|     pub workspace\_id:      UUID, |
|     pub workspace\_type:    WorkspaceType,  // Personal|Shared|Project|Guest|Template|AI|Archive |
|     pub owner\_id:          IdentityId, |
|     pub space\_id:          SpaceId, |
|     pub grid\_id:           GridId, |
|     pub namespace\_path:    NamespacePath, |
|  |
|     // Session state |
|     pub open\_cubes:        Vec\<OpenCubeRef\>, |
|     // OpenCubeRef { cube\_id, view\_id, active\_dim\_slices, scroll\_state, active\_cell\_coord } |
|     pub active\_cube\_id:    Option\<CubeId\>, |
|     pub pinned\_rows:       Vec\<(CubeId, D1Key)\>, |
|     pub pinned\_columns:    Vec\<(CubeId, D2Key)\>, |
|     pub recent\_cells:      Vec\<(CubeId, DimCoordinate, DateTime)\>, |
|  |
|     // Active dimension context (workspace-level DimSlice defaults) |
|     pub dim\_context:       HashMap\<AxisId, DimKey\>, |
|     // e.g. D₃ (Time) \= "2026-Q2" \-- all cubes default to Q2 slice in this workspace |
|  |
|     // Workspace-level filters (applied across all open cubes) |
|     pub global\_filters:    Vec\<DimSlicePredicate\>, |
|  |
|     // UI state |
|     pub layout\_config:     WorkspaceLayout, |
|     pub active\_board\_mode: Option\<BoardMode\>, |
|     pub graph\_panel\_open:  bool, |
|  |
|     // AI context |
|     pub ai\_assistant\_state: AIAssistantState, |
|  |
|     // Session metadata |
|     pub session:           Option\<ActiveSession\>, |
|     pub metadata:          AttributeMap, |
| } |
|  |
| // Workspace-level DimContext: when a user sets D₃ (Time) \= "2026-Q2" |
| // in their workspace, ALL open cubes automatically show the Q2 slice. |
| // This is the N-dimensional generalization of "freeze panes" — it freezes |
| // a dimension at a specific key across all views simultaneously. |

## **6.4  Namespace (HG-NS) — Hierarchical Addressing**

The Namespace system provides globally unique, hierarchical, human-readable addresses for every entity in the Hypergrid ecosystem. The address scheme uses a URI format inspired by DNS and filesystem paths.

| NamespacePath URI scheme: |
| :---- |
|   hypergrid://{grid\_name}/{space\_type}/{slug}/{entity\_type}/{entity\_slug}/... |
|  |
| Examples: |
|   hypergrid://acme/org/engineering/              \-- Engineering Space root |
|   hypergrid://acme/org/engineering/cube/projects/ \-- Projects Hypercube |
|   hypergrid://acme/org/engineering/workspace/sprint-board/ \-- Workspace |
|   hypergrid://acme/personal/alice/               \-- Alice's Personal Space |
|   hypergrid://acme/team/platform/cube/roadmap/row/feature-42/ \-- a specific HyperRow |
|   hypergrid://global/community/data-science/     \-- Community Space |
|   hypergrid://fed/partner-network/               \-- Federation Space |
|  |
| System-reserved addresses: |
|   hypergrid://{grid}/system/spaces/              \-- Spaces Hypercube |
|   hypergrid://{grid}/system/workspaces/          \-- Workspaces Hypercube |
|   hypergrid://{grid}/system/identities/          \-- Identity registry |
|   hypergrid://{grid}/system/namespaces/          \-- Namespace registry |
|   hypergrid://{grid}/system/graph/               \-- Hypergraph store |
|   hypergrid://{grid}/system/eventlog/            \-- Global EventLog |

| Namespace Operation | Description | Permission | Effect |
| :---- | :---- | :---- | :---- |
| resolve(path) | Resolve a NamespacePath to a Grid entity (GridId, CubeId, RowId, etc.) | Viewer | Cache populated; entity returned |
| register(path, id) | Register a new path → entity mapping | Space owner | Entry added to NamespaceRegistry cube |
| rename(old, new) | Rename a path segment. Old path aliased to new. | Entity owner | Alias created; downstream refs async-updated |
| alias(path, target) | Create an additional path that resolves to the same entity | Entity owner | Alias entry added |
| federate(path, peer) | Replicate a namespace entry to a federation peer grid | Federation admin | Entry pushed to peer's NamespaceRegistry |
| transfer(path, new\_owner) | Transfer ownership of a namespace entry to another identity | Current owner | Owner updated; history preserved |
| list(prefix) | List all namespace entries under a given path prefix | Viewer | Paginated list of children |

# **7\.  Hypergraph — Graph & Network Layer (HG-GRAPH)**

| *Every Hypergrid deployment includes a built-in graph layer (the Hypergraph) that models relationships between any HyperCells, HyperRows, Hypercubes, and Grids as a typed, directed, weighted property graph. The Hypergraph enables cross-grid connections, dependency trees, link forests, shadow cell propagation, and network-aware computations — all from within the spreadsheet.* |
| :---- |

## **7.1  Hypergraph Data Model**

| pub struct Hypergraph { |
| :---- |
|     pub graph\_id:      GraphId, |
|     pub grid\_id:       GridId, |
|     pub nodes:         NodeStore,      // NodeId → HypergraphNode |
|     pub edges:         EdgeStore,      // EdgeId → HypergraphEdge |
|     pub subgraphs:     Vec\<Subgraph\>,  // Space-scoped sub-graphs |
|     pub link\_forests:  HashMap\<IdentityId, LinkForest\>, |
| } |
|  |
| pub struct HypergraphNode { |
|     pub node\_id:        NodeId,        // UUID |
|     pub node\_type:      NodeType,      // Cell|Row|Cube|Space|Grid|External|Custom |
|     pub entity\_ref:     EntityRef,     // reference to the entity this node represents |
|     // EntityRef \= CellRef|RowRef|CubeRef|SpaceRef|GridRef|ExternalRef |
|     pub attributes:     AttributeMap,  // node-level N-attributes (reuses HG-CELL model) |
|     pub namespace\_path: Option\<NamespacePath\>, |
|     pub visibility:     NodeVisibility, |
|     pub identity\_tag:   Option\<IdentityTag\>, |
| } |
|  |
| pub struct HypergraphEdge { |
|     pub edge\_id:        EdgeId, |
|     pub from\_node:      NodeId, |
|     pub to\_node:        NodeId, |
|     pub edge\_type:      EdgeType,      // see §7.2 |
|     pub direction:      EdgeDirection, // Directed|Bidirectional |
|     pub weight:         f64,           // 0.0–1.0 |
|     pub attributes:     AttributeMap,  // edge-level N-attributes |
|     pub visibility:     EdgeVisibility, |
|     pub consent\_status: ConsentStatus, // Pending|Accepted|Declined|Revoked |
|     pub space\_scope:    Option\<SpaceId\>, |
|     pub shadow\_cell\_id: Option\<ShadowCellId\>,  // shadow cell created by this edge |
|     pub expires\_at:     Option\<DateTime\<Utc\>\>, // for time-limited edges |
|     pub created\_at:     DateTime\<Utc\>, |
| } |

## **7.2  Edge Type Registry**

| Edge Type | Source Node | Target Node | Description | Shadow Cell? |
| :---- | :---- | :---- | :---- | :---- |
| Hierarchy | Any | Any | Parent-child containment. Used for drill-down and rollup along any axis. | No |
| Dependency | Row/Cell | Row/Cell | One entity depends on another (blocking relationship). Cross-grid supported. | Optional |
| Association | Any | Any | Soft lateral association. Non-blocking reference between entities. | No |
| Contains | Space/Cube | Cube/Row | Ownership containment: this Space/Cube contains this entity. | No |
| Derives | Row | Row | One row is derived from or is a version/fork of another. | Optional |
| CrossGridLink | Row/Cell | Row/Cell | Inter-grid connection: the core link network edge. Enables link forests. | Yes |
| ShadowOf | ShadowRow | Source Row | The inverse of CrossGridLink: marks a row as a shadow of a row in another grid. | N/A (IS shadow) |
| SpaceMembership | Identity | Space | A tenant/identity is a member of a Space. | No |
| Collaborates | Identity | Row/Cube | Bidirectional collaboration on a shared entity. | Yes |
| References | Cell/Row | Cell/Row | Citation or reference. Lighter than Dependency. | No |
| FederationPeer | Grid | Grid | Two Grids are federated for CRDT sync. All replicated entities share edges here. | N/A (system) |
| NamespaceAlias | Namespace | Namespace | One namespace path is an alias for another. | No |
| ComputedFrom | Cell | Cell(s) | This cell's attribute value was computed from the target cell(s). | No |
| SubscribesTo | Identity | Row/Cube/Space | Identity receives change notifications for this entity. | Optional |
| InvestedIn | Identity | Row/Space | Economic investment relationship. Platform-specific semantics. | Yes |
| Custom(String) | Any | Any | Platform-defined edge type registered via HG-PLUGIN. | Plugin-defined |

## **7.3  Shadow Cell Protocol**

The Shadow Cell protocol is the generalization of Hypergrid's ShadowRow pattern to the N-dimensional model. When a CrossGridLink edge is established between a cell/row in Grid A and a cell/row in Grid B, a ShadowCell (or ShadowRow) is provisioned in Grid B — a read-only reflection of the linked entity from Grid A.

| Phase | Description | N-Dim Extension |
| :---- | :---- | :---- |
| Link Request | Entity A creates a CrossGridLink to Entity B. If consent required, invitation sent to Entity B's owner. | The link can target a specific DimCoordinate, a full HyperRow, an entire Hypercube, or a DimSlice. |
| Visibility Config | Entity B configures which attribute keys are visible in the ShadowCell, and optionally which attributes Entity A can write back. | In N-dim: can also configure which dimension keys are shadowed (e.g. only share time periods D₃ \>= "2026"). |
| Shadow Provisioning | ShadowCell (or ShadowRow) created in Grid A's grid, registered in HG-GRAPH with ShadowOf edge. | ShadowCell carries a dim\_scope: which dimensional slices of the source are reflected. |
| Attribute Sync | When source entity's visible attributes change, ShadowCell receives delta via Kafka. Mirror attributes updated in Cell Cache. | Only attribute keys in visible\_attrs and dimension keys in dim\_scope are synced. |
| Write-back | If write\_back\_attrs configured, Grid A's edits to those attributes are propagated back to the source as cross-grid CrdtOps. | Write-back can be scoped to specific dimension keys (e.g. only write-back comments, not financial data). |
| Disconnection | When CrossGridLink is deleted, ShadowCell marked Disconnected. Last known values preserved in EventLog. | All N-dim slices of the ShadowCell disconnected simultaneously. |

## **7.4  Link Forest & Tree Traversal**

| Traversal Mode | Algorithm | Depth | N-Dim Extension |
| :---- | :---- | :---- | :---- |
| Ego Graph | BFS from root node; depth ≤ 2; all cross-grid edges | 2 | Filter by edge type and by dimension scope (e.g. only show links where D₃ \= current quarter) |
| Link Forest | BFS from all nodes owned by an identity; union of trees | 3 | Forest scoped to a DimSlice: shows only links relevant to current cube+dim context |
| Dependency Tree | Topological sort on Dependency edges; cycle-safe | unlimited | Full N-dim coordinates shown per node; critical path highlights cross-dim bottlenecks |
| Component Tree | DFS from a single row/cell node; all in/out edges | 5 | N-dim attribute panel shown for each node; filter by attribute key |
| Space Subgraph | All edges between members of a Space | 4 | Slice to show only edges involving a specific dimension (e.g. geo=EMEA connections) |
| Shortest Path | Dijkstra on weighted Hypergraph; edge.weight as cost | computed | Edge weights can incorporate N-dim attribute values (e.g. D₃-weighted time proximity) |
| Cluster Detection | Louvain on undirected projection of full link graph | full | Cluster assignments exposed as an additional computed attribute on HyperRow nodes |
| Centrality | PageRank / betweenness (sampled approximation for scale) | full | Centrality stored as AI-computed attribute on each node row |

# **8\.  Multi-Tenancy & Identity (HG-ID)**

| *HG-ID provides a platform-agnostic multi-tenancy and multi-identity model. "Tenant" is the general term for the Sovereign Entity of a Kogi-style system — it can be a user, organization, research group, or machine agent. Every tenant may have multiple identities, each with its own namespace, visibility configuration, and partition of the shared grid.* |
| :---- |

## **8.1  Tenant & Identity Hierarchy**

| Level | Concept | Definition | Grid Mapping |
| :---- | :---- | :---- | :---- |
| L0 | SovereignTenant | The real-world entity (person, org, agent, service) that owns a root set of Hypercubes and Spaces in a Grid. | One SovereignTenant \= one root namespace root \+ one Personal Space |
| L1 | Account | A credential set (API key, OAuth, passkey) granting authenticated access to a SovereignTenant's grid. | Multiple accounts may share one SovereignTenant. Each has an access scope (cube \+ attribute \+ dim restrictions). |
| L2 | Identity | A named, purpose-specific @handle with its own display name, bio, and public profile. | Each Identity is a DimensionKey in the system.identities cube \+ a HypergraphNode. |
| L3 | TenantPartition | A logical partition of a SovereignTenant's grid, scoped to one Identity. Rows tagged to a partition via identity\_tags. | NOT a physical copy. OR-Set of identity\_tag attribute keys on HyperRows determines partition membership. |
| L4 | VisibilityMask | Rules specifying which rows, cells, and attribute keys are visible to which observer type. | Applied at query time: DimSlice predicate injected based on caller's identity \+ relationship to data owner. |
| L5 | SplitPolicy | Governance rules on cross-partition data access. Enforced by PolicyEngine on every CrdtOperation crossing partition boundaries. | PolicyEngine.evaluate() called on every cross-partition write. HardIsolated partitions have field-level encryption. |

## **8.2  TenantPartition Data Model**

| pub struct TenantPartition { |
| :---- |
|     pub partition\_id:       UUID, |
|     pub identity\_handle:    String,          // @handle — unique within SovereignTenant |
|     pub sovereign\_id:       SovereignTenantId, |
|     pub display\_name:       String, |
|     pub isolation\_level:    IsolationLevel,  // None | Soft | Hard |
|     pub visibility\_mask:    VisibilityMask, |
|     pub identity\_tag:       IdentityTag,     // tag applied to rows in this partition |
|     pub linked\_accounts:    Vec\<AccountId\>, |
|     pub linked\_spaces:      Vec\<SpaceId\>, |
|     pub graph\_node\_id:      NodeId,          // this identity's node in HG-GRAPH |
|     pub namespace\_path:     NamespacePath,   // hypergrid://{grid}/identity/{handle}/ |
|     pub split\_policy\_id:    Option\<PolicyId\>, |
|     pub public\_cube\_view\_id:  ViewId,        // public profile view |
|     pub follower\_view\_id:     ViewId, |
|     pub connection\_view\_id:   ViewId, |
| } |
|  |
| pub struct VisibilityMask { |
|     pub public\_row\_filter:   DimSlicePredicate,  // which rows are public |
|     pub follower\_row\_filter: DimSlicePredicate, |
|     pub connection\_filter:   DimSlicePredicate, |
|     pub public\_attrs:        Vec\<AttributeKey\>,  // which attrs visible to public |
|     pub follower\_attrs:      Vec\<AttributeKey\>, |
|     pub connection\_attrs:    Vec\<AttributeKey\>, |
|     pub owner\_only\_attrs:    Vec\<AttributeKey\>,  // never visible to non-owner |
|     pub custom\_rules:        Vec\<VisibilityRule\>, |
|     // VisibilityRule: (DimSlicePredicate, AttrSet) — context-dependent visibility |
| } |

## **8.3  N-Dimensional Visibility — DimSlice-Based Visibility Masks**

In Hypergrid's N-dimensional model, visibility masks are not just row-level filters — they are full N-dimensional DimSlice predicates. This enables sophisticated visibility rules that vary by any combination of dimension axes:

| Visibility Pattern | DimSlice Predicate | Description |
| :---- | :---- | :---- |
| Row-level (conventional) | D₁ \= {owned\_by\_me} OR D₁.visibility \= Public | Standard row-level visibility. Works identically to 2D spreadsheet access control. |
| Column-level | D₂ IN {public\_attrs} (for non-owner callers) | Certain column keys hidden from specific observer types. |
| Time-scoped | D₃ \>= {public\_from\_date} (TimeAxis) | Historical data visible to public only from a specific date onward. Older data owner-only. |
| Geo-scoped | D₄ IN {permitted\_regions} (GeoAxis) | Geographic data access: a researcher can only see data for their approved countries. |
| Scenario-scoped | D₅ IN {"base\_case"} (non-owner excludes forecast/pessimistic) | Scenario axes: public sees only base case; internal users see all scenarios. |
| Org-unit-scoped | D₆ \= {caller's\_org\_unit} OR D₆ \= "cross-unit-approved" | Users see only their own org unit's data plus explicitly shared cross-unit rows. |
| N-dim combined | D₁=active AND D₃\>=2026-Q1 AND D₄ IN {allowed\_regions} | Full N-dim visibility intersection: multiple axes restrict simultaneously. |
| Attribute-conditional | WHEN D₂="salary" THEN visibility=Owner; WHEN D₂="name" THEN visibility=Public | Attribute-conditional: different attrs of the same row/cell have different visibility rules. |
| Graph-traversal-based | D₁ IN (GRAPH\_NEIGHBORS(my\_node, edge="Collaborates")) | Social/network visibility: can see rows of direct collaborators but not the wider network. |

# **9\.  View Engine (HG-VIEW)**

| *The View Engine transforms the raw N-dimensional cell store into shaped, filtered, sorted, grouped, and visually rendered presentations. A HypercubeView is the primary user-facing artifact: a fully described, shareable, versionable configuration that projects a sub-space of the Hypercube onto a specific rendering surface.* |
| :---- |

## **9.1  HypercubeView Data Model**

| pub struct HypercubeView { |
| :---- |
|     pub view\_id:           ViewId, |
|     pub cube\_id:           CubeId, |
|     pub name:              String, |
|     pub namespace\_path:    NamespacePath, |
|  |
|     // Dimensionality configuration |
|     pub primary\_axis:      AxisId,        // which axis maps to "rows" in 2D rendering |
|     pub secondary\_axis:    AxisId,        // which axis maps to "columns" in 2D rendering |
|     pub dim\_slices:        Vec\<DimSlice\>, // predicates on all other axes |
|     pub dim\_folds:         Vec\<DimFold\>,  // axes collapsed by aggregation |
|     pub dim\_expands:       Vec\<DimExpand\>,// axes expanded as pivot columns |
|  |
|     // Column/row configuration |
|     pub attr\_schema:       Vec\<AttrColumnDef\>,  // which attribute keys shown \+ order |
|     pub pinned\_attrs:      Vec\<AttributeKey\>,   // always-visible attributes (left freeze) |
|     pub hidden\_attrs:      Vec\<AttributeKey\>,   // hidden from this view |
|     pub col\_widths:        HashMap\<AttributeKey, u32\>, |
|  |
|     // Filtering / Sorting / Grouping |
|     pub filters:           Vec\<DimSlicePredicate\>, |
|     pub sorts:             Vec\<ViewSort\>, |
|     pub groups:            Vec\<ViewGroup\>, |
|  |
|     // Rendering mode |
|     pub render\_mode:       RenderMode, |
|     // Grid2D | Kanban | Gantt | Calendar | NetworkGraph | Treemap | |
|     // Timeline | HierarchyTree | PivotTable | HeatMap | Matrix | |
|     // LinkForest | N-DimExplorer | Custom(PluginId) |
|  |
|     // Augmentation |
|     pub highlight\_rules:   Vec\<HighlightRule\>, |
|     pub summary\_row:       Option\<SummaryRowConfig\>, |
|     pub shadow\_policy:     ShadowCellPolicy,  // Include|Exclude|ShadowOnly |
|     pub identity\_filter:   Option\<IdentityTag\>, |
|     pub space\_filter:      Option\<SpaceId\>, |
|     pub graph\_overlay:     bool, |
|     pub ai\_overlay:        bool, |
|  |
|     // Sharing |
|     pub visibility:        ViewVisibility,  // Private|Shared|SpaceShared|Public|Template |
|     pub created\_by:        IdentityId, |
|     pub metadata:          AttributeMap, |
| } |

## **9.2  Axis Remapping — Any Dimension Can Be "Rows" or "Columns"**

One of the most powerful features of the HypercubeView is axis remapping: the user can choose which dimension maps to the row axis and which maps to the column axis for a given view, without changing the underlying data. This enables radically different perspectives on the same cube:

| View Name | primary\_axis (rows) | secondary\_axis (cols) | Description |
| :---- | :---- | :---- | :---- |
| Standard | D₁ (Entity) | D₂ (Property) | Conventional spreadsheet: entities as rows, properties as columns |
| Transposed | D₂ (Property) | D₁ (Entity) | Properties as rows, entities as columns — useful for attribute-comparison views |
| Time Series | D₃ (Time) | D₂ (Property) | Time periods as rows, properties as columns — timeline/history view |
| Geographic | D₄ (Region) | D₂ (Property) | Regions as rows, properties as columns — geo breakdown view |
| Scenario Compare | D₅ (Scenario) | D₁ (Entity) | Scenarios as rows, entities as columns — plan-vs-actual comparison |
| Cross-dim Matrix | D₃ (Time) | D₄ (Region) | Time × Geography matrix with D₁-aggregated values — heatmap ready |
| Entity × Time | D₁ (Entity) | D₃ (Time) | Entities as rows, time periods as columns — standard time-series pivot |
| Org × Scenario | D₆ (OrgUnit) | D₅ (Scenario) | Org units as rows, scenarios as columns — planning comparison matrix |

## **9.3  Render Modes**

| Render Mode | Description | Primary/Secondary Axes | Best For |
| :---- | :---- | :---- | :---- |
| Grid2D | Standard 2D spreadsheet grid. Rows × Columns with cell values. | Any two axes | General data viewing, editing, and bulk operations |
| PivotTable | Cross-tabulation of two axes with aggregate values. DimFold \+ DimExpand applied. | Any two \+ fold/expand | Multi-dimensional summaries; cross-dim analysis |
| HeatMap | Color-coded matrix: rows × columns with value-to-color mapping. | Any two numeric axes | Pattern detection; anomaly identification; risk matrices |
| Matrix | Square matrix for relationship visualization (dependency matrix, correlation matrix). | Any two (same type) | Dependencies; correlations; cross-entity relationships |
| Kanban | Swimlane columns \= enum axis values; cards \= row entities with key attributes. | D₁ (entity), D₂/D₃ (status) | Workflow management; status-driven work |
| Agile | Sprint-scoped Kanban with story points, burndown overlay, velocity tracking. | D₁ (tasks), sprint dim | Software development; agile project management |
| Gantt | Horizontal bars from start to end date; dependencies as arrows; critical path. | D₁ (entity), D₃ (time) | Project planning; timeline visualization |
| Calendar | Day/week/month grid rendering by date attribute. | D₃ (time), D₁ (entity) | Scheduling; event planning; deadline management |
| Timeline | Swimlane rows; bars per entity across time axis. | D₁ (entity), D₃ (time) | Roadmaps; strategic planning; phased rollouts |
| HierarchyTree | Collapsible tree rendered along a HierarchyAxis. | D₁ with hierarchy | Org charts; file systems; product taxonomies |
| Treemap | Hierarchical tiles sized by numeric attribute; color by score attribute. | D₁ hierarchy, D₂ (size) | Budget composition; portfolio breakdown; category distribution |
| NetworkGraph | Force-directed graph: nodes \= row entities; edges \= HG-GRAPH edges. Color by attribute. | D₁ (entities) \+ edges | Dependency visualization; link network exploration |
| LinkForest | KLNK-style tree rendering of cross-grid links. Identity-colored branches. | D₁ \+ CrossGridLink edges | Inter-grid connection visualization; partner network |
| N-DimExplorer | Interactive N-dimensional browser: choose primary/secondary axes, sliders for all other axes. Pan/zoom. | All N axes user-selectable | Deep exploratory N-dim analysis; data science |
| Custom | Platform-defined render mode registered via HG-PLUGIN. | Plugin-defined | Domain-specific visualizations (maps, genomic plots, circuit diagrams) |

# **10\.  Distributed Consistency (HG-CRDT)**

| *Hypergrid is distributed-first: every Grid is designed for multi-node, multi-device, offline-capable concurrent editing. Consistency is maintained by a generalized N-dimensional CRDT that applies appropriate merge semantics per attribute type, per axis type, and per governance policy.* |
| :---- |

## **10.1  N-Dimensional CRDT Model**

Hypergrid's CRDT operates at three granularity levels:

| CRDT Level | Applies To | Semantics | Merge Strategy |
| :---- | :---- | :---- | :---- |
| Grid-level | Full Grid CRDT sync across federation peers | Gossip protocol: each peer maintains a vector clock per grid; delta sync on heartbeat | VectorClock merge: element-wise max; delta replay in causal order |
| Hypercube-level | Schema: dimension additions, attribute registrations | SchemaEvent log: ordered, idempotent. Schema merges are non-conflicting if additive. | All additive schema changes commute. Destructive changes require governance gate. |
| Cell-attribute-level | Individual cell attribute values | Per-attribute CRDT semantics (see §10.2). Each attribute key in the AttributeKeyRegistry has independently configured CRDT semantics. | Per-attribute: LWW | OR-Set | Counter | Lattice |

## **10.2  Per-Attribute CRDT Semantics**

| CRDT Type | Attribute Types | Merge Rule | Conflict Behavior |
| :---- | :---- | :---- | :---- |
| LWW (Last-Write-Wins) | Text, Number, Enum, Date, Bool, Relation | Highest VectorClock timestamp wins. Tiebreak: ActorId lexicographic sort. | Losing write stored in ConflictRecord; surfaced to users for review. Oba notifies affected actors. |
| OR-Set | MultiEnum, Tag, MultiRelation, MultiUser | All concurrent Adds survive. Removes tagged: only the exact tagged entry removed. | No conflict for concurrent Adds. Remove vs concurrent Add: both survive (Add wins per OR-Set semantics). |
| GrowOnly Counter | view\_count, follower\_count, analytics counters | Sum all increment operations. Never decrements. | Always commutes. No conflict possible. |
| PN-Counter | budget\_spent, allocated\_units | Sum of (positive\_ops \- negative\_ops) per node. | Commutes across all concurrent increments/decrements. |
| Max Register | version, sequence\_number, generation | Highest numeric value wins. | Always resolves to maximum; no human review needed. |
| Min Register | earliest\_deadline, creation\_timestamp | Lowest value wins. | Always resolves to minimum; no human review needed. |
| Status Lattice | status, state, lifecycle\_stage | Merge to least upper bound in the configured partial order. No raw LWW for status. | Governance gate: lattice transitions that require approval are flagged for GovernanceWorkflow before applying. |
| Append-only Log | EventLog, comment\_thread, change\_history | All appends commute. Causally ordered by VectorClock. | No conflict: all appends survive. Causal order guaranteed. |
| JSON CRDT | Json, plugin-defined complex attrs | Deep-merge of JSON objects: recursive LWW per leaf key. | Key-level conflicts use LWW; structural conflicts (key type change) surfaced as ConflictRecord. |
| Custom (Plugin) | Custom attribute types | Plugin provides merge(a: AttrValue, b: AttrValue, ctx: MergeContext) → AttrValue. | Plugin-defined conflict resolution. Plugin may surface ConflictRecord for human review. |

## **10.3  N-Dimensional Conflict Scenarios**

| Scenario | Description | Resolution |
| :---- | :---- | :---- |
| Concurrent cell value writes (same coord) | Two nodes write different values to the same (D₁k, D₂k, ..., DNk) coordinate's "value" attribute simultaneously. | LWW: higher VectorClock timestamp wins. Losing value in ConflictRecord. |
| Concurrent dimension key add \+ remove | Node A adds D₃ key "2026-Q3"; Node B removes "2026-Q3" concurrently. | OR-Set semantics: Add wins over concurrent Remove. Key retained. |
| Concurrent schema changes | Two nodes add different custom axes D₃ simultaneously. | Additive schema changes commute. Both axes added. dim\_index assigned by timestamp order. |
| Concurrent status transitions (lattice) | Node A: status Active → Paused. Node B: status Active → Completed. Both concurrent. | Lattice join: Completed dominates Paused (further in lifecycle). Status \= Completed. ConflictRecord for visibility. |
| Cross-partition write (identity boundary) | Actor from Identity A writes a cell attribute on a HyperRow tagged to Identity B (HardIsolated). | PolicyEngine: DENY. Operation rejected at gateway. CrossPartitionViolationEvent appended to audit log. |
| DimSlice predicate conflict (view filter) | Concurrent view edits change incompatible DimSlice predicates for the same view. | LWW per ViewDefinition attribute. Latest full ViewDefinition wins. Previous view preserved in ViewHistory. |
| Shadow cell sync vs local edit | Grid A's ShadowCell has a value; Grid B (source) updates it; simultaneously Grid A's user edits the shadow cell locally. | Shadow cells are read-only for non-write-back attributes. Local edit rejected. User shown "shadow cell — edit at source" message. |
| Concurrent axis rename | Two federation peers rename the same axis with different names simultaneously. | LWW: higher VectorClock timestamp wins for axis.name. Alias created for the losing name. |
| Concurrent federation peer add | Two Grid admins add different federation peers simultaneously. | OR-Set: both federation peers added. VectorClock extended for both. |

## **10.4  Federation Protocol**

| // Federation sync lifecycle (same for Grid-to-Grid) |
| :---- |
|  |
| 1\. DISCOVERY: Register peer. POST /federation/peers { peer\_url, trust\_level } |
|    → Mutual challenge-response auth. Grid IDs exchanged. Peer record stored. |
|  |
| 2\. HANDSHAKE: Exchange current VectorClock state \+ schema hashes. |
|    → Identify: (a) schema delta (dim/attr additions), (b) event delta (cell mutations) |
|  |
| 3\. SCHEMA SYNC: Replay SchemaEvents missing from peer (additive only). |
|    → Peer applies dimension additions, attribute registrations idempotently. |
|  |
| 4\. CELL DELTA SYNC: Compute set of CrdtOperations since peer's last VectorClock. |
|    → Batch into chunks of max 10,000 ops. Compress (Zstd). Send to peer. |
|    → Peer applies CrdtOperation stream via apply\_crdt\_op() in causal order. |
|  |
| 5\. CONFLICT SURFACE: Conflicts detected during apply are sent back to origin. |
|    → Origin surfaces ConflictRecord to relevant actors via HG-AI anomaly \+ notification. |
|  |
| 6\. NAMESPACE SYNC: Propagate NamespaceRegistry entries for federated entities. |
|  |
| 7\. HEARTBEAT: Every 30s. Exchange VectorClock only. |
|    → If divergence detected: trigger full delta sync. |
|  |
| 8\. SHADOW SYNC: Ongoing. Kafka subscription per shadow-linked entity. |
|    → Cell attribute delta pushed to subscribing grids via shadow\_sync topic. |

# **11\.  Computation Engine (HG-COMP)**

## **11.1  Two-Tier Architecture**

| Tier | Description | Latency | Technology |
| :---- | :---- | :---- | :---- |
| Tier 1 — Synchronous | Arithmetic and structural ComputedAttributes: derived from other cell attributes in the same HyperRow, simple aggregations over a DimSlice, formula expressions without AI/ML. | \<1ms per cell | Rust ComputedAttrEvaluator, embedded in cell query pipeline |
| Tier 2 — Asynchronous | AI/ML-driven signals: anomaly detection, health scoring, classification, prediction, recommendation, entity resolution. | 50ms–30s (model-dependent) | Pluggable AIEngineAdapter \+ CellCache (Redis) \+ WritebackProtocol |

## **11.2  Formula Language — HyperQL Expressions**

| Category | Functions | N-Dim Extensions |
| :---- | :---- | :---- |
| Arithmetic | \+  −  ×  ÷  %  ^  ABS ROUND CEIL FLOOR MIN MAX | Operands can be any DimCoordinate reference: CELL(D₁=id, D₂="revenue", D₃="2026-Q2") |
| Comparison | \=  \!=  \>  \<  \>=  \<= | Compare across dim keys: CELL(D₃="2026-Q2", D₂=attr) \> CELL(D₃="2026-Q1", D₂=attr) |
| Logical | AND OR NOT IF(cond,t,f) SWITCH COALESCE |  |
| String | CONCAT UPPER LOWER LEN TRIM LEFT RIGHT SUBSTR CONTAINS STARTS |  |
| Date | TODAY NOW YEAR MONTH QUARTER DAYS\_BETWEEN DATE\_ADD DATE\_FORMAT | Works naturally on TimeAxis keys |
| Aggregation | SUM AVG COUNT MIN MAX MEDIAN PERCENTILE STDDEV DISTRIBUTION | Over any DimSlice: SUM(SLICE(D₃="2026", D₄="EMEA", D₂="revenue")) |
| DimFold | FOLD(axis\_id, agg\_fn, attr) | Collapse any axis: FOLD(time\_axis, SUM, "revenue") |
| DimSlice | SLICE(dim\_predicates...) → set of cells | Any combination of axis predicates: SLICE(D₁=active, D₃ \>= ytd\_start) |
| Graph | GRAPH\_NEIGHBORS GRAPH\_PATH GRAPH\_DIST GRAPH\_CENTRALITY | Traverses HG-GRAPH: GRAPH\_NEIGHBORS(D₁=row\_id, "DependsOn", attr="budget\_allocated") |
| Shadow | SHADOW\_CELL(grid\_id, coord, attr) | Reference a mirror-attribute from a shadow cell in a linked grid |
| AI Engine | AI\_SCORE(model\_id, attrs...) AI\_CLASSIFY AI\_PREDICT AI\_ANOMALY | Inject Tier-2 AI signals as formula operands. Cached per model TTL. |
| Namespace | NS\_RESOLVE(path) NS\_PATH(entity\_ref) NS\_SCOPE(entity\_ref) | Navigate by namespace address |
| Space | SPACE\_NAME(space\_id) SPACE\_MEMBER\_COUNT(space\_id) IN\_SPACE(space\_id) |  |
| Lookup | RELATED(rel\_attr, target\_attr) LOOKUP(key, cube\_id, key\_attr, value\_attr) | Cross-cube join: RELATED(customer\_id\_attr, "customer\_name") from CRM cube |
| Identity | IDENTITY\_TAG() IS\_PARTITION(tag) VISIBLE\_TO(obs\_type) | VisibilityMask-aware formula expressions |
| Time | AS\_OF(timestamp, formula) | Time-travel: evaluate formula against historical cell state |

## **11.3  Pluggable AI Engine (HG-AI)**

The AI Engine in Hypergrid is fully pluggable — any platform can connect their own ML/AI service to provide Tier-2 computed attributes. The HG-AI module defines the AIEngineAdapter interface:

| pub trait AIEngineAdapter { |
| :---- |
|     // Register a computed attribute model |
|     fn register\_model(\&self, model: AIModel) \-\> Result\<ModelId, AIError\>; |
|  |
|     // Compute a batch of cell attribute values |
|     fn compute\_batch( |
|         \&self, |
|         requests: Vec\<ComputeRequest\>, |
|         context: \&ComputeContext, |
|     ) \-\> Result\<Vec\<ComputeResult\>, AIError\>; |
|  |
|     // Detect anomalies across a DimSlice |
|     fn detect\_anomalies( |
|         \&self, |
|         slice: \&DimSlice, |
|         attr: \&AttributeKey, |
|         sensitivity: f32, |
|     ) \-\> Result\<Vec\<AnomalySignal\>, AIError\>; |
|  |
|     // Translate natural language to HyperQL |
|     fn nl\_to\_query( |
|         \&self, |
|         natural\_language: \&str, |
|         cube\_context: \&CubeContext, |
|     ) \-\> Result\<HyperQLQuery, AIError\>; |
|  |
|     // Writeback: accept AI-computed values and write to cells |
|     fn writeback\_protocol(\&self) \-\> WritebackProtocolRef; |
| } |
|  |
| // Built-in reference implementation: OpenAI-compatible adapter |
| // Platform can replace with local LLM, custom ML service, or any REST/gRPC AI API |

# **12\.  HyperQL — N-Dimensional Query Language (HG-QL)**

| *HyperQL is the N-dimensional query language of Hypergrid. It is a SQL-inspired language typed to the N-dimensional cell model. All DimSlice, DimFold, DimExpand, time-travel, shadow-cell inclusion, cross-grid join, and graph-traversal operations are expressible as first-class HyperQL constructs.* |
| :---- |

## **12.1  Full Grammar (EBNF)**

| query       ::= SELECT sel FROM source where? fold? expand? group? order? limit? options? |
| :---- |
|  |
| sel         ::= "\*" | sel\_item ("," sel\_item)\* |
| sel\_item    ::= attr\_key | agg\_expr | formula\_expr | dim\_key\_ref | "EXPAND" axis\_id |
|  |
| source      ::= cube\_ref | grid\_ref | "SHADOW CUBES" | "(" query ")" |
| cube\_ref    ::= cube\_id | namespace\_path |
| grid\_ref    ::= "GRID(" string\_lit ")" |
|  |
| where       ::= "WHERE" condition |
| condition   ::= predicate |
|               | "(" condition "AND" condition ")" |
|               | "(" condition "OR"  condition ")" |
|               | "NOT" condition |
|  |
| predicate   ::= dim\_pred | attr\_pred | graph\_pred | shadow\_pred |
| dim\_pred    ::= "D" uint "=" dim\_key |
|               | "D" uint "IN" "\[" dim\_key ("," dim\_key)\* "\]" |
|               | "D" uint "BETWEEN" dim\_key "AND" dim\_key |
|               | "D" uint "UNDER" dim\_key          \-- HierarchyAxis drill-down |
|               | "D" uint "IS NULL" |
|               | "D" uint "IS NOT NULL" |
| attr\_pred   ::= attr\_key op val |
|               | attr\_key "IS EMPTY" | "IS NOT EMPTY" |
|               | attr\_key "INCLUDES" val |
|               | attr\_key "HAS\_SEVERITY" severity |
| graph\_pred  ::= "CONNECTED\_TO(" node\_id "," edge\_type ")" |
|               | "GRAPH\_DISTANCE(" node\_id ")" op uint |
| shadow\_pred ::= "IS SHADOW" | "IS NOT SHADOW" |
|  |
| fold        ::= "FOLD" axis\_id "WITH" agg\_fn "(" attr\_key ")" |
| expand      ::= "EXPAND" axis\_id "AS COLUMNS(" sel\_item ")" |
| group       ::= "GROUP BY" groupby\_item ("," groupby\_item)\* rollup? |
| rollup      ::= ", ROLLUP(" axis\_id ")" |
| order       ::= "ORDER BY" order\_item ("," order\_item)\* |
| order\_item  ::= attr\_key | dim\_key\_ref | agg\_expr |
|               | ("ASC" | "DESC") ("NULLS FIRST" | "NULLS LAST")? |
| limit       ::= "LIMIT" uint ("OFFSET" uint)? |
|  |
| options     ::= option+ |
| option      ::= identity\_scope | space\_scope | shadow\_opt | as\_of\_opt | ns\_opt | graph\_opt |
| identity\_scope ::= "IDENTITY" string\_lit |
| space\_scope    ::= "IN SPACE" string\_lit |
| shadow\_opt     ::= "INCLUDE SHADOW CELLS" |
|                  | "SHADOW FROM" string\_lit    \-- specific grid/tenant |
|                  | "SHADOW ONLY" |
| as\_of\_opt      ::= "AS\_OF" timestamp |
| ns\_opt         ::= "UNDER" namespace\_path      \-- namespace subtree filter |
| graph\_opt      ::= "TRAVERSE GRAPH" edge\_type\_list depth\_spec |

## **12.2  HyperQL Examples**

| \-- 1\. Basic 2D query (conventional spreadsheet equivalent) |
| :---- |
| SELECT name, status, budget\_allocated, budget\_spent, health\_score |
| FROM projects\_cube |
| WHERE D₂.status \= "Active" |
| ORDER BY health\_score DESC |
| LIMIT 50; |
|  |
| \-- 2\. Time-sliced query (N=3) |
| SELECT D₁.entity\_name, D₂.revenue, D₂.expenses, D₂.profit |
| FROM financials\_cube |
| WHERE D₃ BETWEEN "2026-01-01" AND "2026-06-30" |
|   AND D₁.status \= "Active" |
| ORDER BY D₂.revenue DESC; |
|  |
| \-- 3\. Geographic \+ time DimFold |
| SELECT D₁.product\_name, |
|        SUM(D₂.revenue) AS total\_revenue\_emea |
| FROM sales\_cube |
| WHERE D₄ \= "EMEA" |
| FOLD D₃ WITH SUM(revenue)    \-- collapse time: sum all periods |
| GROUP BY D₁.product\_name |
| ORDER BY total\_revenue\_emea DESC; |
|  |
| \-- 4\. Full N-dim pivot (time × region) |
| SELECT D₁.product\_name, |
|        EXPAND D₄ AS COLUMNS(SUM(revenue)) |
| FROM sales\_cube |
| FOLD D₃ WITH SUM(revenue) |
| GROUP BY D₁.product\_name, D₄.region; |
| \-- Result: product\_name | EMEA\_revenue | APAC\_revenue | NA\_revenue | LATAM\_revenue |
|  |
| \-- 5\. Hierarchy drilldown (HierarchyAxis) |
| SELECT D₆.org\_unit, SUM(budget\_allocated), AVG(health\_score) |
| FROM projects\_cube |
| WHERE D₆ UNDER "Engineering"    \-- Engineering \+ all sub-units |
| GROUP BY D₆.org\_unit, ROLLUP(D₆); |
|  |
| \-- 6\. Graph-traversal query |
| SELECT D₁.project\_name, D₂.budget\_allocated, |
|        GRAPH\_NEIGHBORS(D₁, "DependsOn", attr="project\_name") AS dependencies |
| FROM projects\_cube |
| WHERE D₂.status \= "Active" |
| TRAVERSE GRAPH "DependsOn" DEPTH 3; |
|  |
| \-- 7\. Shadow cell inclusion |
| SELECT D₁.entity\_name, D₂.status, D₂.health\_score, |
|        shadow\_source\_grid AS source\_grid |
| FROM projects\_cube |
| WHERE D₂.status \= "Active" |
| INCLUDE SHADOW CELLS |
| ORDER BY D₂.health\_score DESC; |
|  |
| \-- 8\. Time-travel query |
| SELECT D₁.entity\_name, D₂.status, D₂.budget\_remaining |
| FROM projects\_cube |
| AS\_OF "2026-01-01T00:00:00Z";     \-- state as of Jan 1 2026 |
|  |
| \-- 9\. Cross-grid join |
| SELECT p.project\_name, p.budget\_allocated, |
|        c.customer\_name, c.mrr |
| FROM projects\_cube AS p |
| JOIN GRID("crm://acme/cube/customers") AS c |
|   ON p.customer\_ref \= c.D₁; |
|  |
| \-- 10\. Namespace-scoped query |
| SELECT D₁.entity\_name, D₂.status |
| FROM \* |
| UNDER "hypergrid://acme/org/engineering/" |
| WHERE D₂.status \= "Active"; |
|  |
| \-- 11\. Identity-partitioned query |
| SELECT D₁.entity\_name, D₂.income, D₂.benefit\_balance |
| FROM personal\_finance\_cube |
| IDENTITY "@alice-freelance"      \-- only rows in this identity partition |
| ORDER BY D₂.income DESC; |
|  |
| \-- 12\. Multi-dim scenario comparison |
| SELECT D₁.product\_name, |
|        CELL(D₁, D₂="revenue", D₃="2026-Q2", D₅="base\_case") AS base\_revenue, |
|        CELL(D₁, D₂="revenue", D₃="2026-Q2", D₅="optimistic") AS opt\_revenue, |
|        CELL(D₁, D₂="revenue", D₃="2026-Q2", D₅="pessimistic") AS pess\_revenue |
| FROM planning\_cube |
| ORDER BY base\_revenue DESC; |

# **13\.  Persistence & Storage Architecture**

## **13.1  Storage Layer Design**

| Store | Technology | Responsibility | Encoding | Access Pattern |
| :---- | :---- | :---- | :---- | :---- |
| Primary Cell Store | PostgreSQL 16 \+ JSONB | HyperCell data for Dense/Hybrid cubes. Schema: (grid\_id, cube\_id, d1\_key, d2\_key, ..., dn\_key, attributes JSONB). | HASH partitioned by (grid\_id, cube\_id) | Read-heavy; JSONB-indexed; N-dim composite index |
| Sparse Cell Store | PostgreSQL 16 | HyperCell data for Sparse cubes. Only non-null cells stored. Full N-dim coordinate as composite PK. | Composite PK \+ GIN on attributes | O(log n) lookup; range scan via N-dim index |
| Schema Registry | PostgreSQL 16 | DimensionAxis definitions, AttributeKeyRegistry, CubeDefinitions, GridDefinitions. | Normalized relational schema | Read on every query; cached in memory |
| Event Log | PostgreSQL 16 (append-only, monthly partitioned) | Immutable log: every cell mutation, schema change, governance action, graph event. | Partitioned by month; append-only | Append on every mutation; range reads for audit |
| CRDT Buffer | Redis Cluster | In-flight CrdtLog operations; VectorClock cache; hot-path LWW cell cache. | Hash by (grid\_id, cube\_id, coord\_hash) | Sub-ms read/write; TTL expiry; async PG flush |
| Cell Attribute Cache | Redis Cluster | Computed and AI attribute values. TTL per model. Invalidated on cell mutation or schema event. | Key: cell\_cache:{grid}:{cube}:{coord\_hash}:{attr\_key} | O(1) read/write; TTL invalidation |
| Graph Store | PostgreSQL 16 (+ optional Neo4j for scale) | HypergraphNode and HypergraphEdge tables. Cross-grid link edges. Shadow cell mapping. | B-tree \+ GIN on edge\_type, node attributes | Graph traversal queries; topology updates |
| Space/Workspace Store | PostgreSQL 16 | Space, Workspace, SpaceMember, GovernanceProposal, WorkspaceSession records. | Normalized relational | Read on auth; write on events |
| Namespace Registry | PostgreSQL 16 \+ Redis | NamespacePath → entity mapping. Resolution cache in Redis (300s TTL). | BTree index on path; Redis cache | Every request (fast path: Redis hit) |
| Shadow Cell Store | PostgreSQL 16 | ShadowCell records: source grid, coord, visible\_attrs, sync state, last\_synced. | Indexed by (host\_grid, source\_grid) | Read on cube query; write on shadow sync |
| Full-Text Search | Meilisearch / OpenSearch | Full-text index over row names, descriptions, tags, attribute text values, namespace paths. | Incremental index update on mutation | Search queries; faceted search |
| Time-Series Analytics | ClickHouse | High-throughput append of cell events, engagement metrics, mutation velocity. TimeAxis native support. | Column-oriented; partitioned by date | Append-only; aggregate reads; never updated |
| Snapshot Store | S3-compatible | Full Grid snapshots \+ per-cube checkpoints. Compressed JSON-LD. Signed for integrity. | Object store; versioned; lifecycle policy | Write: scheduled \+ manual; Read: on restore |
| AI Feature Store | Redis \+ PG materialized views | Pre-computed AI model inputs and outputs. Refreshed on schedule and on trigger. | Materialized views rebuilt per model TTL | Read: on AIAttr eval; Write: AI engine push |

## **13.2  N-Dimensional Index Strategy**

| Index Type | Axes Covered | Query Types Accelerated | Storage Overhead |
| :---- | :---- | :---- | :---- |
| Composite B-Tree (D₁×D₂) | Dense cubes: D₁ \+ D₂ keys | Row lookup, column scan, 2D range scan | Low — standard PG index |
| N-dim Composite (D₁…Dₙ) | All N axes | Full N-dim point lookup, partial prefix scan | O(N × row\_count) — use only for N ≤ 5 |
| JSONB GIN (attributes) | Cell attribute map | Attribute key existence, attribute value containment | Moderate — GIN index on JSONB |
| TimeSeries (BRIN on D₃) | TimeAxis keys (D₃) | Time range scans; chronological aggregation | Very low — BRIN is compact |
| Spatial (GeoHash on D₄) | GeoAxis keys (D₄) | Geographic bounding box, radius, nearest-neighbor | Low — PostGIS/GeoHash index |
| GIN on Tag/MultiEnum attrs | Tag, MultiEnum attribute keys | Attribute set membership, tag filtering | Moderate — GIN array index |
| Full-text (Meilisearch) | Text attribute keys (name, description) | Free-text search, phrase matching, faceted filters | Separate service; moderate storage |
| Graph adjacency (B-Tree) | HypergraphEdge from\_node \+ to\_node | O(1) neighbor lookup, O(log n) edge traversal | Low — standard FK index |
| IdentityTag GIN | identity\_tags array attribute | Partition-scoped queries, multi-identity filtering | Low-moderate |
| Namespace B-Tree | NamespacePath string column | Path prefix lookup, hierarchical namespace queries | Low — string prefix index |
| ZOrder curve (multi-dim) | D₃×D₄ (Time+Geo composite) | Range scans across two sparse axes simultaneously | Moderate — space-filling curve index |

# **14\.  Security, Access Control & Compliance**

## **14.1  N-Dimensional Security Model**

Security in Hypergrid is enforced at five nested levels, from the Grid down to individual cell attributes:

| Level | Enforcement Point | Mechanism | Bypass Protection |
| :---- | :---- | :---- | :---- |
| Grid | Authentication layer | JWT/API key validates tenant membership. Grid-level permissions granted. | No grid access without valid credential. |
| Space | SpaceMember permission tier | PermissionTier evaluated against SpaceMember.role on every request involving Space-scoped entities. | Role evaluated at API gateway. Cannot be elevated by application code. |
| Hypercube | CubeGovernance policy | CubeGovernance.access\_policy evaluated before any cube-level operation (query, mutate, schema change). | PolicyEngine.evaluate() called before query planning. |
| HyperRow | Row-level security (PostgreSQL RLS \+ application) | VisibilityMask DimSlice predicate injected into all queries. HardIsolated partition rows have field-level encryption. | RLS enforced at PostgreSQL layer — no application-layer bypass. |
| Cell Attribute | Attribute-level visibility (application layer) | owner\_only\_attrs stripped from results for non-owner callers. Per-attribute visibility checked per-result-row. | Applied after RLS, before serialization. Cannot be bypassed by client. |

## **14.2  Permission Tier Model**

| Tier | Ordinal | Capabilities |
| :---- | :---- | :---- |
| Public (unauthenticated) | 0 | Read cells where visibility=Public. No mutations. No graph traversal beyond public nodes. |
| Viewer | 1 | Read all permitted cells. No mutations. Can follow/subscribe (creates graph edge). |
| Subscriber | 2 | Read \+ bookmark \+ subscribe actions. No content mutations. |
| Contributor | 3 | Submit contributions to shared cubes. Comment. Vote. Cannot directly edit cells. |
| Editor | 4 | Edit cell values, add rows, add custom attribute keys to own cubes. Create DimSlice views. |
| Manager | 5 | Manage Space settings and members. Configure cube governance. Add new dimensions (requires governance gate for shared cubes). |
| Owner | 6 | Full ownership: all operations including dimension removal, cube deletion, identity configuration, federation setup. |
| Admin | 7 | Platform super-admin override. Legal hold access. Audit report generation. |
| System | 8 | Internal service-to-service. AI engine writeback. CRDT federation peer sync. |

## **14.3  Data Governance for N-Dimensional Data**

| Governance Feature | Description | Enforcement Layer |
| :---- | :---- | :---- |
| Dimension-level Access Control | Individual axes can have axis\_visibility settings. A Dim can be tenant-only (hidden from public queries) or identity-specific. | Query planner: invisible axes excluded from query scope before planning. |
| Attribute-level Encryption | owner\_only\_attrs are field-level AES-256 encrypted. Key derived from tenant identity. Never decrypted for non-owner queries. | PostgreSQL pgcrypto / application-level encryption before storage. |
| Time-axis Data Retention | TimeAxis cubes support configurable retention policies: delete cells older than N days/months. | Background job: purge cells outside retention window. EventLog preserves mutation history. |
| Schema Governance | Adding a new dimension or removing an attribute key requires a GovernanceProposal vote in multi-owner Spaces. | CubeGovernance.require\_vote\_for: \[add\_dimension, remove\_attr, change\_visibility\] |
| Audit Trail | Every cell mutation, schema change, governance action, and identity operation appended to the immutable EventLog. | PostgreSQL append-only table. No UPDATE/DELETE permitted. RLS prevents tampering. |
| Right to Erasure (GDPR/CCPA) | Tenant can request erasure of personal data. Cell values anonymized; EventLog entries have actor\_id replaced with anonymous\_actor. | Erasure job: null personal attribute values \+ anonymize actor\_id in EventLog. Cryptographic audit chain maintained. |
| Data Portability | Full Grid export in JSON-LD with schema definitions. Machine-readable, standards-aligned, signed for integrity. | HG-EXPORT: /export/grid/{grid\_id}?format=jsonld — includes all cubes, axes, attrs, cells, graph, namespaces. |
| Cross-jurisdiction Data Residency | Per-tenant storage location configuration. Grid data confined to configured regions (AWS/GCP/Azure region selection). | Grid-level storage\_config.region enforcement. Federation restricted to same-region peers by default. |

# **15\.  API Surface (HG-API)**

## **15.1  REST API — Core Endpoints**

| Endpoint | Method | Description | Min Auth |
| :---- | :---- | :---- | :---- |
| /grids | GET | List all Grids accessible to the caller | Viewer |
| /grids | POST | Create a new Grid | Admin |
| /grids/{id}/cubes | GET | List all Hypercubes in a Grid | Viewer |
| /cubes | POST | Create a new Hypercube | Editor |
| /cubes/{id} | GET | Get Hypercube schema and metadata | Viewer |
| /cubes/{id}/dims | GET | List all DimensionAxes for a Hypercube | Viewer |
| /cubes/{id}/dims | POST | Add a new DimensionAxis (N+1) | Manager |
| /cubes/{id}/attrs | GET | List AttributeKeyRegistry for a Hypercube | Viewer |
| /cubes/{id}/attrs | POST | Register a new attribute key | Editor |
| /cubes/{id}/rows | GET | Query HyperRows (DimSlice params \+ pagination) | Viewer |
| /cubes/{id}/rows | POST | Create a new HyperRow (set D₁ key \+ initial attrs) | Editor |
| /cubes/{id}/rows/{d1\_key} | GET | Get all cells for one HyperRow | Viewer |
| /cubes/{id}/rows/{d1\_key} | PATCH | Update cell attribute values for a HyperRow | Editor |
| /cubes/{id}/rows/{d1\_key} | DELETE | Soft-archive a HyperRow | Owner |
| /cubes/{id}/cells/{coord} | GET | Get a single HyperCell by N-dim coordinate | Viewer |
| /cubes/{id}/cells/{coord} | PUT | Set a HyperCell attribute map at a coordinate | Editor |
| /cubes/{id}/cells/{coord}/{attr\_key} | PATCH | Update a single attribute of a HyperCell | Editor |
| /cubes/{id}/query | POST | Execute a HyperQL query | Viewer |
| /cubes/{id}/views | GET | List HypercubeViews | Viewer |
| /cubes/{id}/views | POST | Create a HypercubeView | Editor |
| /cubes/{id}/views/{view\_id}/rows | GET | Render rows for a specific view (with all view config) | Viewer |
| /cubes/{id}/fold | POST | Execute a DimFold aggregation | Viewer |
| /cubes/{id}/expand | POST | Execute a DimExpand (pivot) | Viewer |
| /cubes/{id}/export | GET | Export cube (CSV/XLSX/JSON/Parquet/Arrow/JSON-LD) | Editor |
| /cubes/{id}/snapshot | POST | Create a cube snapshot | Owner |
| /cubes/{id}/restore/{snapshot\_id} | POST | Restore cube from snapshot | Owner |
| /cubes/{id}/eventlog | GET | Get EventLog entries for this cube (paginated) | Editor |
| /cubes/{id}/crdt/sync | POST | Submit CrdtLog batch (federation peers only) | System |

## **15.2  Graph, Space & Namespace API**

| Endpoint | Method | Description |
| :---- | :---- | :---- |
| /graph/nodes/{id}/tree | GET | Get link tree from node (depth, edge\_type, direction params) |
| /graph/nodes/{id}/forest | GET | Get full link forest for all nodes owned by this identity |
| /graph/edges | POST | Create HypergraphEdge (triggers consent flow if required) |
| /graph/edges/{id} | DELETE | Delete edge (triggers shadow cell disconnection) |
| /graph/shadow-cells | GET | List all shadow cells in this grid from linked external grids |
| /graph/shadow-cells/{id}/sync | POST | Manually trigger shadow cell attribute sync from source |
| /graph/path | GET | Shortest path between two nodes (Dijkstra) |
| /graph/clusters | GET | Community cluster detection (Louvain) |
| /spaces | GET | List Spaces in this Grid |
| /spaces | POST | Create a Space |
| /spaces/{id}/members | GET | List Space members \+ roles |
| /spaces/{id}/governance/propose | POST | Submit governance proposal |
| /spaces/{id}/governance/vote | POST | Cast vote on proposal |
| /workspaces | POST | Create a Workspace |
| /workspaces/{id}/session | POST | Open Workspace session (returns WebSocket URL) |
| /workspaces/{id}/dim-context | PUT | Set workspace-level DimKey defaults (dim context) |
| /ns/resolve | GET | Resolve a NamespacePath to an entity ID |
| /ns | POST | Register a namespace entry |
| /ns/{id}/alias | POST | Add a namespace alias |
| /ns/{id}/federate | POST | Federate namespace entry to peer grid |
| /federation/peers | GET | List federation peers for this Grid |
| /federation/peers | POST | Register a new federation peer |
| /federation/sync | POST | Trigger manual federation sync with all peers |
| /identity/partitions | GET | List all TenantPartitions for authenticated identity |
| /identity/partitions | POST | Create a new TenantPartition |
| /identity/merge | POST | Initiate CrossTenantMerge (requires checkpoint \+ confirmation) |
| /ai/compute | POST | Request AI-computed attribute values for a batch of cells |
| /ai/nl-query | POST | Translate natural language to HyperQL |
| /ai/anomalies | GET | List AI-detected anomalies in a cube or DimSlice |

## **15.3  WebSocket & Streaming**

| Channel | Description | Events |
| :---- | :---- | :---- |
| wss://.../cubes/{id}/stream | Real-time cell mutation stream for a Hypercube. Clients subscribe and receive all cell changes. | CellUpdated · RowCreated · RowArchived · DimAdded · AttrRegistered · AIWriteback · ConflictDetected |
| wss://.../workspaces/{id}/session | Workspace collaboration session. Cursor presence, cell lock indicators, co-editing synchronization. | CursorMoved · CellLocked · CellUnlocked · PresenceJoined · PresenceLeft · DimContextChanged |
| wss://.../spaces/{id}/activity | Space activity stream: member events, governance actions, graph edge changes. | MemberJoined · ProposalCreated · VoteSubmitted · EdgeCreated · ShadowCellUpdated |
| wss://.../graph/shadow-sync | Shadow cell synchronization stream. Receives mirror attribute deltas from linked external grids. | ShadowAttrUpdated · ShadowDisconnected · ShadowReconnected |
| wss://.../crdt/federation/{peer\_id} | CRDT federation sync stream. Gossip CrdtOperations between peers. | CrdtOpBatch · VectorClockExchange · ConflictSurfaced · SchemaDelta |

# **16\.  Plugin & Extension System (HG-PLUGIN)**

| *HG-PLUGIN is the formal extension interface for Hypergrid. All platform-specific customizations — domain attribute sets, custom axis types, AI engine adapters, render mode adapters, data connectors, governance modules, and export adapters — are registered as HypercubePlugin implementations. Plugins are loaded at Grid initialization and can be updated without downtime.* |
| :---- |

## **16.1  Plugin Interface**

| // The HypercubePlugin trait — all Hypergrid extensions implement this |
| :---- |
| pub trait HypercubePlugin: Send \+ Sync { |
|     fn plugin\_id(\&self)   \-\> PluginId; |
|     fn plugin\_name(\&self) \-\> \&str; |
|     fn version(\&self)     \-\> semver::Version; |
|     fn capabilities(\&self)-\> Vec\<PluginCapability\>; |
|  |
|     // Lifecycle |
|     fn on\_load(\&mut self, ctx: \&PluginContext) \-\> Result\<(), PluginError\>; |
|     fn on\_unload(\&mut self)                    \-\> Result\<(), PluginError\>; |
|     fn health\_check(\&self)                     \-\> PluginHealth; |
| } |
|  |
| // Plugin capability mix-ins — implement the ones your plugin provides |
| pub trait AxisTypePlugin: HypercubePlugin { |
|     fn axis\_type\_id(\&self) \-\> AxisType; |
|     fn validate\_key(\&self, key: \&DimKey) \-\> Result\<(), ValidationError\>; |
|     fn compare\_keys(\&self, a: \&DimKey, b: \&DimKey) \-\> Ordering; |
|     fn key\_to\_display(\&self, key: \&DimKey) \-\> String; |
|     fn index\_strategy(\&self) \-\> IndexStrategy; |
| } |
|  |
| pub trait AttributeTypePlugin: HypercubePlugin { |
|     fn attr\_type\_id(\&self) \-\> AttributeType; |
|     fn validate\_value(\&self, v: \&TypedAttrValue) \-\> Result\<(), ValidationError\>; |
|     fn merge(\&self, a: \&TypedAttrValue, b: \&TypedAttrValue, |
|              ctx: \&MergeContext) \-\> TypedAttrValue; |
|     fn render\_cell(\&self, v: \&TypedAttrValue) \-\> RenderCell; |
| } |
|  |
| pub trait ComputedModelPlugin: HypercubePlugin { |
|     fn model\_id(\&self) \-\> ModelId; |
|     fn output\_attrs(\&self) \-\> Vec\<AttributeKeyDef\>; |
|     fn compute(\&self, row: \&HyperRow, ctx: \&ComputeContext) \-\> ComputeResult; |
|     fn invalidation\_triggers(\&self) \-\> Vec\<EventKind\>; |
|     fn cache\_ttl(\&self) \-\> Duration; |
| } |
|  |
| pub trait RenderModePlugin: HypercubePlugin { |
|     fn render\_mode\_id(\&self) \-\> RenderMode; |
|     fn render(\&self, view: \&HypercubeView, data: \&ProjectedCube) \-\> RenderOutput; |
|     fn supported\_axes(\&self) \-\> Vec\<AxisType\>; |
| } |
|  |
| pub trait DataConnectorPlugin: HypercubePlugin { |
|     fn connector\_type(\&self) \-\> ConnectorType; |
|     fn import(\&self, source: ConnectorSource) \-\> Result\<ImportPlan, ConnectorError\>; |
|     fn export(\&self, cube: \&HypercubeSlice, target: ConnectorTarget) \-\> Result\<(), ConnectorError\>; |
|     fn sync\_stream(\&self) \-\> Option\<Box\<dyn SyncStream\>\>; |
| } |
|  |
| pub trait AIEnginePlugin: HypercubePlugin { |
|     fn ai\_engine\_id(\&self) \-\> AIEngineId; |
|     fn register\_model(\&self, model: AIModelDef) \-\> ModelId; |
|     fn compute\_batch(\&self, reqs: Vec\<ComputeRequest\>) \-\> Vec\<ComputeResult\>; |
|     fn detect\_anomalies(\&self, slice: \&DimSlice, attr: \&AttributeKey) \-\> Vec\<AnomalySignal\>; |
|     fn nl\_to\_query(\&self, nl: \&str, ctx: \&CubeContext) \-\> HyperQLQuery; |
| } |
|  |
| pub trait GovernancePlugin: HypercubePlugin { |
|     fn governance\_model\_id(\&self) \-\> GovernanceModelId; |
|     fn evaluate\_proposal(\&self, proposal: \&Proposal) \-\> GovernanceDecision; |
|     fn quorum\_reached(\&self, votes: &\[Vote\], total\_members: u32) \-\> bool; |
|     fn compute\_distribution(\&self, contributors: &\[Contributor\]) \-\> Vec\<Distribution\>; |
| } |

## **16.2  Reference Plugin Catalog**

| Plugin Name | Type | Description |
| :---- | :---- | :---- |
| time-axis | AxisType | Built-in TimeAxis plugin: Timestamp keys, BRIN index, chronological ordering, rollup by period, AS\_OF time-travel support |
| geo-axis | AxisType | Built-in GeoAxis plugin: GeoHash keys, PostGIS index, bounding box and radius queries, NUTS/ISO admin level hierarchies |
| hierarchy-axis | AxisType | Built-in HierarchyAxis plugin: HierarchyPath keys, parent-child validation, ROLLUP and DRILLDOWN computation, closure table index |
| version-axis | AxisType | Built-in VersionAxis plugin: SemVer keys, fork/merge operations, diff computation between version keys |
| openai-ai-engine | AIEngine | Reference AI engine adapter: OpenAI-compatible API. NL-to-HyperQL, anomaly detection, health scoring, attribute completion |
| pm-domain | ComputedModel | Project management attribute pack: health\_score, schedule\_risk, budget\_risk, predicted\_completion (ComputedModelPlugin) |
| finance-domain | ComputedModel | Financial domain attribute pack: profit, roi, ytd\_actuals, budget\_variance, cash\_flow\_forecast |
| crm-domain | ComputedModel | CRM attribute pack: lead\_score, pipeline\_value, churn\_probability, engagement\_health |
| kanban-renderer | RenderMode | Kanban board renderer: swimlane columns \= enum axis values; cards \= row entities; WIP limits; drag-and-drop events |
| gantt-renderer | RenderMode | Gantt chart renderer: horizontal bars on TimeAxis; Dependency edges as arrows; critical path; milestones |
| ndim-explorer | RenderMode | N-Dim interactive explorer: axis selection dropdowns; DimSlice sliders; real-time DimFold/DimExpand |
| network-graph-renderer | RenderMode | Force-directed graph renderer: HG-GRAPH edges as visual links; D3.js \+ WebGL for large graphs |
| csv-connector | DataConnector | CSV import/export: column-to-attr mapping wizard, type inference, batch row creation, streaming export |
| excel-connector | DataConnector | XLSX import/export: multi-sheet mapping, formula preservation, chart data export |
| parquet-connector | DataConnector | Apache Parquet export: columnar format, Snappy compression, schema embedding — for data lake pipelines |
| graphql-connector | DataConnector | GraphQL API adapter: expose any Hypercube as a GraphQL schema; mutation subscriptions |
| cooperative-governance | Governance | Cooperative democracy governance: 1-member-1-vote, Shapley value distribution, multi-sig treasury |
| democracy-governance | Governance | Democratic governance: configurable quorum, supermajority options, ranked-choice voting |
| multisig-treasury | Governance | Multi-signature treasury: M-of-N approval for financial operations; escrow support |

# **17\.  Export & Integration (HG-EXPORT)**

## **17.1  Export Formats**

| Format | Content | N-Dim Handling | API Endpoint |
| :---- | :---- | :---- | :---- |
| CSV | Flat 2D view of any HypercubeView (DimFold applied to reduce to 2D). One row per D₁ entity, one column per D₂ attr (+ folded dim values as additional columns). | DimFolds reduce N→2 before export. DimExpands create extra columns. | /cubes/{id}/export?format=csv\&view\_id=... |
| XLSX | Multi-sheet Excel workbook: one sheet per unique combination of D₃–Dₙ keys (if N≤4); or one sheet per DimFold configuration. | Multiple sheets for different dim slices. Pivot table per DimExpand. | /cubes/{id}/export?format=xlsx |
| JSON | Full N-dim JSON: cube schema \+ array of HyperCell objects with full DimCoordinate and AttributeMap. | Full fidelity: all N dimension keys and all attributes per cell. | /cubes/{id}/export?format=json |
| JSON-LD | Linked Data export with schema.org / custom vocab type annotations. Machine-readable and semantically rich. | DimAxis types mapped to schema.org dimensions. Full coordinates. | /cubes/{id}/export?format=jsonld |
| Parquet | Apache Parquet columnar format: each attribute key is a column; each dimension key is a partition column. | Natural fit: Parquet supports partition columns for each dim axis. | /cubes/{id}/export?format=parquet |
| Arrow | Apache Arrow in-memory columnar format for high-performance data pipeline integration. | Dimension keys as partition columns; attribute values as record columns. | /cubes/{id}/export?format=arrow |
| GraphQL | GraphQL schema \+ resolvers auto-generated from cube schema. Mutations and subscriptions included. | DimSlice args on every GraphQL query. N-dim pagination built-in. | /cubes/{id}/graphql — auto-generated schema |
| Webhook | Real-time push of cell mutation events to a registered external URL on every EventLog append. | Events include full DimCoordinate and changed AttributeMap. | /cubes/{id}/webhooks — manage subscriptions |
| Embed | Embeddable iframe/script: read-only or edit-enabled view of any HypercubeView in an external webpage. | Respects all DimSlice \+ VisibilityMask settings of the view. | /cubes/{id}/views/{view\_id}/embed |
| GDPR Archive | Complete Grid export: all cubes, all axes, all cells, EventLog, graph, namespaces. Encrypted, signed. | Full N-dim fidelity. Compressed. Verifiable chain of custody. | /grids/{id}/export/gdpr-archive |

## **17.2  Import Sources**

| Source | Method | N-Dim Mapping | Dim Assignment |
| :---- | :---- | :---- | :---- |
| CSV / XLSX | File upload → AI-assisted attribute mapping wizard → batch HyperRow creation | Single dim by default (D₁×D₂). Multi-sheet XLSX: each sheet \= one D₃ slice. | Wizard prompts for D₃–Dₙ key assignment (default or auto-detect) |
| Relational DB | JDBC/ODBC connector → table-to-cube mapping → schema inference | Table cols → D₂ attrs. FK relationships → HG-GRAPH edges. Views → DimSlice configs. | D₃–Dₙ from table partitions or timestamp columns |
| REST / GraphQL API | API connector → response mapping → incremental sync (webhook or polling) | JSON response fields → D₂ attrs. Nested objects → custom D₃ expansion. | API response pagination key → D₃ if time or ordinal |
| Apache Kafka | Kafka consumer → streaming cell mutations. Real-time event sourcing into cube. | Event fields → attrs. Kafka topic partition → custom axis key. | Kafka offset → D₃ (sequence axis) or timestamp → D₃ (time axis) |
| Parquet / Arrow | Object store pull → columnar load → high-throughput batch insert | Parquet partition columns → dim axis keys. Record columns → attrs. | Auto-inferred from Parquet schema |
| HyperQL Feed | Another Hypergrid instance pushes via CrossGridLink \+ federation sync protocol. | Full N-dim fidelity: DimCoordinates preserved. Attrs synchronized. | Source dim keys preserved; local axis aliases configurable |
| Git Repository | Git connector: commits → rows, changed files → cells, branches → D₃ (version axis). | D₁=commit, D₂=file metric, D₃=branch/tag | Branch as version axis; commit timestamp as time sub-axis |

# **18\.  Performance, Scalability & Operations**

## **18.1  Performance Targets**

| Operation | N | Target p50 | Target p99 | Key Optimization |
| :---- | :---- | :---- | :---- | :---- |
| 2D cell query (1K rows) | 2 | \<30ms | \<150ms | Dense encoding; hot CellCache; result compression |
| N-dim query with 2 DimSlices | 4 | \<80ms | \<400ms | Composite N-dim index; DimSlice predicate pushdown to PG |
| DimFold \+ DimExpand (pivot) | 4 | \<200ms | \<1s | Materialized DimFold views for common pivot configurations |
| HyperQL complex join | 3 | \<300ms | \<2s | Query planner cost estimation; index-only scans; join reordering |
| CRDT merge (1K ops) | \- | \<80ms | \<400ms | Rust native CRDT; batched merge for federation peers |
| Shadow cell sync (single attr) | \- | \<300ms | \<2s | Kafka priority queue; WebSocket push for active sessions |
| Namespace resolution | \- | \<5ms | \<20ms | Redis cache (300s TTL); pre-warm on Space load |
| Graph tree traversal (depth 3\) | \- | \<150ms | \<800ms | Closure table for HierarchyAxis; adjacency list for others |
| AI attribute compute (batch 100\) | \- | \<500ms | \<5s | AI engine batching; CellCache pre-populate; circuit breaker |
| Full cube export (100K rows, N=3) | 3 | \<10s | \<60s | Streaming export; server-side DimFold before serialization |

## **18.2  Horizontal Scalability Architecture**

| Component | Scale Strategy | Limits | Scale Trigger |
| :---- | :---- | :---- | :---- |
| Cell Store (PostgreSQL) | HASH partitioning by (grid\_id, cube\_id). Read replicas for queries. PgBouncer connection pool. | 10B cells per deployment (estimated) | CPU \> 70% on primary; replica lag \> 1s |
| CRDT Service | Horizontal scale: stateless (VectorClock in Redis). Kafka for async persistence. Multiple instances. | 500K CRDT ops/sec per cluster | Queue depth \> 10K; latency p99 \> 200ms |
| Graph Store | PostgreSQL for hot graphs (\< 10M edges). Neo4j/DGraph for full global graph at scale. | 1B edges (Neo4j); 100M edges (PostgreSQL) | Graph traversal p99 \> 1s; node count \> 10M |
| AI Compute Tier | AI engine pods auto-scale on compute queue depth. Circuit breaker prevents cascade on model unavailability. | Model-dependent (API rate limits apply) | Compute queue \> 1K requests; p99 \> 2s |
| WebSocket (Workspace) | Sticky sessions on workspace\_id. Redis pub/sub for fanout. Horizontal scale with consistent hashing. | 100K concurrent sessions per cluster | \>80% WS connection capacity |
| Namespace Cache (Redis) | Redis Cluster: 6 shards. LRU eviction. Pre-warm on Space load. CDN-cached for public paths. | 1B cached paths (estimated) | Cache miss rate \> 20%; eviction rate spike |
| Kafka (Event streaming) | Topic partitioned by (grid\_id % num\_partitions). Consumer group per service. Dead letter queue per group. | 10M events/sec per cluster | Consumer lag \> 5K; topic partition imbalance |
| Federation Sync | Per-peer sync worker. Isolated queue per federation peer. Back-pressure on slow peers. | 255 federation peers per Grid | Peer delta \> 500K ops; sync lag \> 5 min |

## **18.3  Observability**

| Signal | Tool | Key Metrics | Alerts |
| :---- | :---- | :---- | :---- |
| Metrics | Prometheus \+ Grafana | Query latency by (cube\_id, N, operation\_type); CRDT merge rate; cell cache hit/miss; AI compute queue depth; graph traversal latency; shadow sync lag | p99 \> threshold; cache miss \> 20%; queue depth spike |
| Traces | OpenTelemetry \+ Jaeger | Per-request distributed trace: grid\_id, cube\_id, N, dim\_slices, identity\_tag, space\_id, AI model calls | Trace \> 2s; AI model call fails |
| Logs | Structured JSON → Loki | All HyperQL executions, cell mutations, CRDT ops, governance actions, identity events, graph edge changes | Error rate spike; auth failure burst |
| Audit | Immutable EventLog | Every cell mutation with (actor, coord, attr, old\_value, new\_value, vector\_clock, identity\_tag) | Cross-partition access; owner permission change; dimension removal |
| Synthetic | k6 / playwright | E2E cube query test (every 5 min); CRDT sync test; shadow cell sync test; namespace resolution test | Any synthetic \> SLA; federation sync failure |

# **19\.  Platform Integration Guide**

| *Hypergrid is designed to be embedded in any platform as the universal data substrate. This section provides a practical guide for platform teams adopting Hypergrid, covering initialization, schema design, the N-dimensional modeling methodology, and common integration patterns.* |
| :---- |

## **19.1  Adoption Checklist**

| Step | Description | Responsible | Outcome |
| :---- | :---- | :---- | :---- |
| 1\. Grid Provisioning | Create a Grid for your platform. Configure: grid\_name, default tenant model, storage region, AI engine adapter, plugin set. | Platform architect | Grid running; system cubes provisioned |
| 2\. Domain Modeling | Identify your domain entities (→ D₁ EntityAxis), properties (→ D₂ PropertyAxis), and additional contexts (→ D₃–Dₙ custom axes). | Domain architect | N-dim schema design document |
| 3\. Attribute Pack Design | Define your domain AttributeKeyRegistry. Specify types, CRDT semantics, visibility, aggregation, and computation for each key. | Domain architect | AttributeKeyRegistry registered |
| 4\. Plugin Registration | Register domain plugins: computed model plugins, custom axis type plugins (if any), AI model definitions. | Platform engineer | All plugins loaded and healthy |
| 5\. Space Types | Configure your Space type taxonomy. Define default cube provisioning per Space type. Set governance defaults. | Platform architect | SpaceType registry configured |
| 6\. Template Cubes | Define template Hypercubes (pre-configured axis \+ attribute schema) for common use cases. Publish to template library. | Platform team | Template library available to tenants |
| 7\. Permission Model | Map your platform's permission model to Hypergrid PermissionTiers. Configure VisibilityMask defaults per Space type. | Security architect | Permission model documented \+ tested |
| 8\. SDK Integration | Integrate the HG-SDK into your platform's backend and frontend. Configure API gateway routing to Hypergrid services. | Platform engineer | SDK integrated; API routes configured |
| 9\. Migration | Import existing data: use HG-IMPORT connectors or HyperQL INSERT for initial data load. Set up incremental sync if needed. | Data engineer | Historical data imported \+ validated |
| 10\. Federation Setup | If multi-tenant or multi-region: configure federation peers. Set CRDT sync intervals. Test delta sync with test data. | Platform engineer | Federation sync healthy; conflict rate \< 0.1% |

## **19.2  N-Dimensional Modeling Methodology**

Follow this decision framework when designing the axis structure for a new Hypercube:

| Question | Answer → Design Decision |
| :---- | :---- |
| What is the primary entity being tracked? | → D₁ EntityAxis. Each row \= one entity. Choose key type (UUID for new entities; existing ID for migration). |
| What properties does each entity have? | → D₂ PropertyAxis. Each column \= one property. Register as attribute keys in AttributeKeyRegistry. |
| Do I need to track how properties change over time? | → Yes: add D₃ TimeAxis. Each cell now holds a (entity, property, time\_period) data point. |
| Do I need a geographic or regional breakdown? | → Yes: add D₄ GeoAxis. Use GeoHash or NUTS/ISO admin level keys. |
| Do I need to compare scenarios, plans, or versions? | → Yes: add D₅ ScenarioAxis or VersionAxis. Keys: "base\_case", "optimistic", "forecast", "actuals", etc. |
| Do I need org/department breakdowns? | → Yes: add D₆ HierarchyAxis (OrgUnit). Parent-child relationships defined in hierarchy. ROLLUP enabled. |
| Am I adding a dimension just to filter (not pivot/aggregate)? | → No: use a D₂ attribute key (filter column) instead of a new axis. Keep N small. |
| Will most (D₁, D₂, D₃, …) combinations have a value? | → Yes: use Dense encoding. No (many null combinations): use Sparse encoding. |
| Is N \> 6 in my design? | → Warning: reconsider. Flatten some axes into attributes. Complex N-dim cubes are hard to query and visualize. |
| Do I need cells to be addressable by namespace? | → Yes: register HyperRows in HG-NS. NamespacePath: hypergrid://{grid}/{cube}/row/{d1\_key}/ |

## **19.3  Common Integration Patterns**

| Pattern | N | Description | Example Domain |
| :---- | :---- | :---- | :---- |
| Entity ledger | 2 | D₁=entity, D₂=property. Simple key-value record store with governance and CRDT. | CRM contacts, Product catalog, User profiles |
| Timeseries cube | 3 | D₁=entity, D₂=metric, D₃=timestamp. Track how entity metrics change over time. | Financial time-series, IoT sensor readings, Analytics metrics |
| Geo cube | 3 | D₁=entity, D₂=metric, D₃/D₄=geo. Regional performance breakdown. | Sales by region, Public health by geography, Weather data |
| Planning matrix | 4 | D₁=entity, D₂=metric, D₃=time, D₄=scenario. Track actuals vs plan vs forecast. | FP\&A planning, Budget vs actuals, Capacity planning |
| Org hierarchy cube | 4 | D₁=employee, D₂=attribute, D₃=time, D₄=org\_unit (hierarchy). Roll up people metrics across org structure. | HR analytics, Compensation planning, Performance management |
| Dependency graph cube | 3 | D₁=component, D₂=attribute, D₃=version. Plus HG-GRAPH Dependency edges for critical path. | Software dependency management, Supply chain, Project dependencies |
| Research data cube | 5 | D₁=sample, D₂=measurement, D₃=time, D₄=condition, D₅=researcher. Multi-condition experimental data with full provenance. | Clinical trials, Lab experiments, Market research |
| Marketplace exchange | 4 | D₁=listing, D₂=attribute, D₃=time, D₄=market\_segment. Track listing performance across time and market segment. | E-commerce, Financial exchange, Labor marketplace |
| Federated workspace | 2 | N=2 cube but federated across multiple Grid deployments via CrossGridLink \+ ShadowCell protocol. | Inter-company collaboration, Open-source project tracking, Partner data sharing |
| Multi-tenant SaaS | 3 | D₁=entity, D₂=attribute, D₃=tenant\_id (TenantAxis). Single physical cube serving multiple tenants with RLS per D₃ key. | SaaS platform shared infrastructure, White-label products, Multi-org portals |

# **20\.  Open Items, Decisions & Roadmap**

## **20.1  Open Technical Decisions**

| Decision | Options | Recommendation | Decision By |
| :---- | :---- | :---- | :---- |
| N-dim index at scale (N≥4) | PostgreSQL composite index | Z-order curve | Specialized OLAP DB (DuckDB, ClickHouse) | Hybrid | Hybrid: PG for N≤4; ClickHouse for N\>4 analytics cubes | v1.1 milestone |
| CRDT conflict surfacing UX | Silent (last-write-wins always) | Notify (user review on conflict) | Block (require resolution) | Notify for semantic conflicts (status lattice); Silent for counters/sets | v1.0 milestone |
| Shadow cell write-back governance | Open (any editor) | Governed (proposal required) | Opt-in per attribute | Opt-in per attribute: write\_back\_attrs configurable per CrossGridLink | v1.0 milestone |
| Max N (dimensionality limit) | 8 | 12 | 16 | Unlimited | Hard limit 16; recommended max 6 for UI usability | v1.0 (finalize) |
| Formula engine sandboxing | WASM sandbox | Process isolation | Interpreted AST | JVM sandbox | Interpreted AST with resource limits (no arbitrary code) | v1.0 milestone |
| Federation identity model | Grid-level trust | Namespace-level trust | Identity-level trust | All three tiers | All three: Grid(full) \> Namespace(partial) \> Identity(minimal) | v1.5 milestone |
| AI engine default | OpenAI-compatible API | Local LLM (Ollama) | No default (platform-provided only) | Multiple | Multiple: OpenAI-compatible \+ local LLM option \+ no-AI mode | v1.0 milestone |
| Time-travel granularity | Day | Hour | Minute | Second | Millisecond | EventLog entry (highest fidelity) | EventLog entry: AS\_OF any timestamp resolves to last EventLog entry before that time | v1.0 milestone |

## **20.2  Version Roadmap**

| Version | Milestone | Key Deliverables |
| :---- | :---- | :---- |
| v0.5 (Alpha) | Core substrate | HyperCell model · N=2 dense encoding · AttributeKeyRegistry · EventLog · PostgreSQL persistence · Basic REST API · LWW CRDT |
| v1.0 (Beta) | 2D+ platform | N=1–6 dimensions · Sparse \+ Hybrid encoding · HyperQL v1 · Formula language · View Engine (10 render modes) · Federation v1 · Spaces v1 · HG-NS v1 · Go SDK \+ TypeScript SDK |
| v1.1 | Full N-dim \+ Graph | All 16 dimension types · HG-GRAPH full implementation · CrossGridLink \+ Shadow Cell protocol · Link forest traversal · N-dim index optimization · ClickHouse analytics integration |
| v1.5 | Intelligence \+ Identity | HG-AI pluggable engine · OpenAI-compatible adapter · NL-to-HyperQL · Anomaly detection · HG-ID multi-tenant identity · TenantPartition · VisibilityMask · CrossTenantMerge |
| v2.0 | Full platform | All render modes including N-DimExplorer · Governance plugins (cooperative, democracy, multisig) · All export formats (Parquet, Arrow, GraphQL) · Plugin marketplace · Web-based N-dim exploration UI |
| v2.5 | Scale \+ Federation | Neo4j graph backend for scale · Cross-Grid AI (federated ML models) · ZOrder curve multi-dim index · 255-node federation · GDPR/CCPA automated compliance workflows |
| v3.0 | Autonomous Intelligence | Autonomous AI agents operating on cubes with approval-first workflows · Predictive DimFold models · Cross-Grid MatchEngine · Semantic layer (AI-generated cube descriptions \+ ontology mapping) · On-chain identity anchoring (DID) |

## **20.3  Glossary**

| Term | Definition |
| :---- | :---- |
| Hypergrid | N-Dimensional Distributed Spreadsheet System — the complete platform described in this document |
| Grid | Root container for all Hypercubes, dimensions, tenants, namespaces, spaces, and graph structures |
| Hypercube | An N-dimensional grid H=(D₁,D₂,…,Dₙ). The generalization of a spreadsheet sheet. |
| DimensionAxis (Dᵢ) | One axis of a Hypercube: EntityAxis (D₁), PropertyAxis (D₂), or any custom type (D₃–Dₙ) |
| DimCoordinate | An ordered tuple of N dimension keys, one per axis, uniquely addressing a HyperCell |
| HyperCell | A data point at an N-dimensional coordinate, carrying an N-attribute map |
| HyperRow | All cells sharing the same D₁ key — the canonical entity in a Hypercube |
| AttributeMap | A HashMap\<AttributeKey, TypedAttrValue\> — the N-attribute payload of a HyperCell |
| AttributeKey | A registered string key in the AttributeKeyRegistry. The "column name" generalized to apply across all dimensions. |
| DimSlice | A predicate applied to one or more dimension axes, producing a sub-cube or projection |
| DimFold | Collapsing an axis by aggregating all its key values into a summary. Generalization of GROUP BY. |
| DimExpand | Expanding a dimension's key set as separate columns. Generalization of PIVOT. |
| HypercubeView | A saved, shareable configuration of DimSlices, DimFolds, axis remappings, filters, sorts, and render mode |
| Hypergraph | The graph layer: typed, directed edges between HyperCells, HyperRows, Cubes, Spaces, and Grids |
| CrossGridLink | A HypergraphEdge crossing Grid boundaries. The inter-grid link network atom. |
| ShadowCell | A read-only reflection in Grid A of a linked entity from Grid B, created by a CrossGridLink |
| MirrorAttribute | A specific attribute from a ShadowCell synced into the host Grid as a read-only computed attribute |
| LinkForest | The complete set of all link trees rooted at a given identity — their full cross-grid network |
| Space | A named, governed, bounded operational context within a Grid. Groups cubes, members, and governance. |
| Workspace | Active working session within a Space. Personalized cube/view arrangement \+ session state. |
| NamespacePath | Hierarchical URI addressing any entity: hypergrid://{grid}/{space\_type}/{slug}/... |
| SovereignTenant | The real-world entity (person, org, or agent) that owns a root set of Hypercubes and Spaces |
| TenantPartition | A logical partition of a SovereignTenant's data, scoped to one Identity. Row-level via identity\_tags. |
| VisibilityMask | N-dimensional DimSlice predicates defining what is visible to each observer type |
| VectorClock | HashMap\<NodeId, u64\>. Logical clock for causal ordering of all mutations across federation nodes |
| CRDT | Conflict-free Replicated Data Type — per-attribute semantics: LWW | OR-Set | Counter | Lattice |
| HyperQL | The N-dimensional query language of Hypergrid: SELECT, DimSlice WHERE, FOLD, EXPAND, TRAVERSE GRAPH, AS\_OF |
| HypercubePlugin | The extension interface for all Hypergrid customizations: axis types, attr types, AI engines, render modes, connectors |
| AIEngineAdapter | Plugin interface for connecting any AI/ML service to provide Tier-2 computed attributes |
| ComputedAttribute | A derived attribute value: Tier-1 (synchronous formula) or Tier-2 (asynchronous AI/ML signal) |
| WritebackProtocol | The secure channel by which AI engines write computed attribute values into HyperCells, respecting all permission and audit requirements |
| AS\_OF | Time-travel query operator: AS\_OF timestamp returns the state of the cube at that historical moment by replaying the EventLog |
| EventLog | Append-only, immutable log of every mutation. Every cell attribute change is an EventLog entry. |
| CrdtLog | In-flight CRDT operation buffer: operations pending application and federation peer sync |
| Federation | CRDT synchronization between two or more Grid deployments. Delta sync via Kafka; VectorClock-based causal ordering |
| N-DimExplorer | Interactive browser for N-dimensional data: axis selection, DimSlice sliders, live DimFold/DimExpand, graph overlay |

