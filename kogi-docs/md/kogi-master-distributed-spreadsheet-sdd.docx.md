

**KOGI**

Independent Worker Operating System

**Master Distributed Spreadsheet System**

KPMS · KPSS · KPRG · KPVW · KPCM · KPCL · KPEX · KLNK · KPID

*Portfolio Spreadsheet · Link Network · Forest & Tree Architecture*

*Multi-Account · Multi-Identity · Multi-Profile · Cross-Portfolio Connections*

| Attribute | Value |
| :---- | :---- |
| Document Version | v1.0 — March 2026 |
| Module Codes | KPMS · KPSS · KPRG · KPVW · KPCM · KPCL · KPEX · KLNK · KPID |
| Primary Languages | Go (services) · Rust (core) · Scala (kogi-engine) · TypeScript (UI) |
| Status | System Design — Active Development |
| Classification | Confidential — Kogi Platform Internal |
| Companion Documents | Portfolio System SDD v2.1 · kogi-bank SDD · kogi-community SDD v2.0 · kogi-game SDD · kogi-marketplace SDD · kogi-rms SDD · kogi-wms SDD |

# **Executive Summary**

The Kogi Master Distributed Spreadsheet System (KMDSS) is the central, sovereign data substrate of the entire Kogi Independent Worker Operating System. It is the unified, configurable, AI-augmented, distributed spreadsheet that every user, organization, collective, cooperative, and federation on the platform uses to record, manage, analyze, and connect every entity in their working, creative, financial, and collaborative life.

At its core, the KMDSS is architecturally simple: every entity in the Kogi ecosystem — programs, projects, resources, assets, artifacts, gigs, benefits, finances, relationships, governance actions, analytics events — is a row in one universal spreadsheet. Every property, metric, relationship, and computed value on that entity is a column. Every curated, filtered, grouped, or aggregated perspective on that data is a sheet. Every user owns their own root spreadsheet — their Master Portfolio Spreadsheet — and the entire Kogi network is the graph of interconnected spreadsheets, linked across users, identities, organizations, and federation nodes.

The KMDSS extends far beyond a conventional spreadsheet tool. It is:

* A living, event-sourced, CRDT-distributed data structure with append-only audit trails and nanosecond-precision VectorClock versioning

* A universal component registry where every Kogi entity is a typed, governed, policy-enforced row with full lifecycle management

* A multi-dimensional view engine that renders the same underlying data as 30+ specialized sheets, plus Kanban boards, Gantt charts, network graphs, treemaps, and pivot tables

* A computation engine with 20+ built-in analytical models producing health scores, risk scores, income projections, and AI-derived signals inline as computed columns

* A federated link network — the KLNK system — that maps the full graph of inter-user, inter-organization, and inter-portfolio connections across the entire platform, rendered as forests and trees of linked spreadsheet nodes

* A multi-account, multi-identity, multi-profile system allowing users to split their root spreadsheet across identities, merge contributions across profiles, and control exactly what is visible to whom

* A collaborative editing platform with real-time concurrent multi-user editing, CRDT-based conflict resolution, and contribution attribution for shared and crowdresourced portfolios

* An intelligence layer where the full kogi-engine stack — 20+ sub-engines — contributes computed columns, anomaly flags, match scores, risk signals, and Oba AI recommendations directly into every sheet

This document provides the complete, exhaustive system design for the KMDSS: every column, every sheet, the full link network architecture, the multi-identity model, all engine integrations, the API surface, persistence strategy, security model, and all open items and roadmap milestones.

# **Table of Contents**

# **1\.  System Overview & Vision**

## **1.1  The Master Spreadsheet Metaphor**

| *Design Principle: Every entity in the Kogi ecosystem — programs, projects, resources, assets, artifacts, gigs, benefits, finances, relationships, governance actions, analytics events — is a row in one universal, scalable, configurable spreadsheet. The Portfolio Management System is that spreadsheet: the master, living, computed record of an independent worker's entire operational and creative world.* |
| :---- |

Every independent worker, autonomous organization, collective, cooperative, and federation on the Kogi platform owns a Master Portfolio Spreadsheet — a root-level spreadsheet workbook that is the authoritative, living record of everything they own, operate, create, manage, finance, and share. This is not a metaphor or a display abstraction. The spreadsheet IS the data model.

The Master Portfolio Spreadsheet is the application of the spreadsheet paradigm — one of the most universal and powerful information organization systems humans have invented — to the full domain of independent work. Where a conventional spreadsheet is a static grid of user-entered values, the Kogi Master Spreadsheet is:

* Event-sourced: every cell mutation is an immutable EventLog entry, never deleted

* Computed: hundreds of columns are derived, not entered — computed from other columns, child rows, external data, or AI engine signals

* Distributed: every spreadsheet is replicated across nodes via CRDT, enabling offline editing, multi-device access, and federated multi-user collaboration without conflicts

* Connected: spreadsheets are linked to each other via the KLNK Link Network, forming a graph of inter-portfolio connections that models the entire Kogi economy

* Policy-governed: every row, column, and cell respects a PermissionTier hierarchy, row-level security, column-level visibility, and a pluggable PolicyEngine

| Spreadsheet Concept | Portfolio System Mapping |
| :---- | :---- |
| Workbook | PortfolioSystem — the root container holding all sheets for a user, org, or federation |
| Sheet | A named, scoped, filterable materialized view of PortfolioComponents (e.g. Projects Sheet, Finances Sheet, Benefits Sheet) |
| Row | A PortfolioComponent — any item, container, resource, artifact, asset, or derived entity |
| Column | A typed PortfolioColumn — a field, computed metric, relationship reference, or engine-derived signal on a component |
| Cell | The intersection of a PortfolioRow and a PortfolioColumn — a single typed, versioned, audited value |
| Formula | A ComputedColumn — a derived field calculated from other columns via the Computation Engine (e.g. Budget Remaining \= Budget Allocated \- Budget Spent) |
| Filter | A ViewFilter — a boolean predicate applied to rows (e.g. Status \= Active, Type \= Project, Owner \= me) |
| Sort | A ViewSort — ordering applied to rows (e.g. sort by Health Score descending) |
| Group | A ViewGroup — hierarchical aggregation of rows by a shared column value |
| Pivot Table | A PivotView — cross-dimensional aggregation (e.g. Budget Spent by Owner x Program x Quarter) |
| Chart | A ChartView — visual rendering of a column's values (bar, line, gauge, treemap, network graph) |
| Freeze Panes | PinnedColumns — identity and key status columns always visible during horizontal scroll |
| Conditional Formatting | RowHighlightRule — color-coding rows based on column values (e.g. red for overdue, green for on-track) |
| Import / Export | KPEX — CSV, XLSX, JSON-LD export of any sheet; webhook push; kogi-dev API pull |
| Workbook Link | KLNK GraphEdge — a typed, directional link between two PortfolioRow stores across users or nodes |

## **1.2  Module Codes & Services**

| Module | Code | Description | Language |
| :---- | :---- | :---- | :---- |
| Portfolio Management System | KPMS | Master orchestration layer: spreadsheet substrate, row/column model, sheet registry, view engine | Go \+ Rust |
| Portfolio Spreadsheet Substrate | KPSS | Low-level data engine: PortfolioRow, ColumnSchema, SheetDefinition, indexed store | Rust |
| Portfolio Component Registry | KPRG | Canonical registry of all PortfolioComponents system-wide; the master index | Go \+ PostgreSQL |
| View Engine | KPVW | Filter · Sort · Group · Pivot · Slice across any column dimension | Go |
| Computation Engine | KPCM | 20+ analytical models; derived column computation; rollup and aggregation | Rust \+ Scala |
| Collaborative Editing | KPCL | CRDT-backed concurrent multi-user spreadsheet editing | Rust |
| Export & Integration | KPEX | CSV · XLSX · JSON · API · Embed · Webhook export of any sheet or view | Go |
| Link Network | KLNK | Inter-portfolio link graph: forests, trees, cross-user connections, visibility protocol | Go \+ Rust (GraphEngine) |
| Identity & Profile System | KPID | Multi-account, multi-identity, multi-profile root spreadsheet management and splitting | Go |

## **1.3  Design Goals**

| Goal | Description |
| :---- | :---- |
| Universal Abstraction | Every platform entity — from a single task to a federation portfolio — is representable as a row in this system |
| Infinitely Configurable | Columns are addable, reorderable, hideable, pinnable, and computable. Sheets are addable, filterable, and shareable. Views are user-defined, template-driven, and AI-suggested |
| Event-Sourced & Auditable | Every cell mutation is appended to the component's EventLog. No data is ever deleted — only archived or superseded |
| Distributed-First | VectorClock \+ CRDT log (LWW \+ OR-Set) ensures that concurrent edits across nodes, collaborators, and federated portfolios converge correctly |
| AI-Augmented | Computed columns, smart defaults, anomaly flags, health scores, risk scores, match scores, and proactive suggestions generated by kogi-engine and surfaced directly in the spreadsheet |
| Policy-Governed | Every row, column, and cell respects a tiered PermissionTier system. Row-level and column-level access control allows fine-grained sharing |
| Composable & Extensible | The PortfolioPlugin trait and ToolBox integration allow developers and power users to extend the system with custom column types, computed models, and automated row generators |
| Collaborative | Multi-user real-time collaborative editing via CRDT; contribution attribution via ContributionRecord; shared portfolio governance via GovernanceProposal |
| Link-Networked | Every spreadsheet is a node in a global link graph. Users can connect their portfolios to others', view linked rows, subscribe to changes, and participate in federated economies |
| Multi-Identity | A single root spreadsheet can span multiple accounts, profiles, and identities, with configurable visibility, data partitioning, and cross-identity merge capabilities |

# **2\.  System Architecture**

## **2.1  Five-Layer Architecture**

The Master Distributed Spreadsheet System is organized in five distinct layers, from the raw data substrate up to the user-facing view and intelligence layers. Each layer builds cleanly on the layer below, enabling both monolithic single-user deployments and massive federated multi-organization deployments to use the same codebase.

| Layer | Name | Components & Responsibility |
| :---- | :---- | :---- |
| L0 — Substrate | Raw Component Store | PortfolioComponent (Rust core) · ComponentMetadata · ComponentData · EventLog · CrdtLog · VectorClock · GraphEdge · PolicyEngine · ToolBoxId refs |
| L1 — Registry | Component Index | KPRG Master Component Registry · ComponentIndex · OwnerIndex · TagIndex · TypeIndex · StatusIndex · FullTextIndex (SearchEngine) · LinkNetworkIndex |
| L2 — Spreadsheet Engine | KPSS Row/Column/Cell | PortfolioRow · ColumnSchema · SheetDefinition · CellStore · ColumnComputer · AggregationEngine · ViewFilter/Sort/Group |
| L3 — View Engine | KPVW View Layer | ViewFilter · ViewSort · ViewGroup · PivotView · ChartView · RowHighlightRule · PinnedColumns · BoardMode · ViewTemplate · SavedViews |
| L4 — Intelligence Layer | kogi-engine Signals | ComputedColumns · HealthScoreColumn · RiskScoreColumn · MatchScoreColumn · RecommendationColumn · AnomalyFlagColumn · Oba AI overlay |
| L5 — Application Layer | User-facing Apps | kogi-portfolio UI · kogi-office Boards · kogi-bank Finance Views · kogi-marketplace Asset Views · kogi-community Feed Integration · KPEX Export |
| L6 — Link Network | KLNK Graph | InterPortfolioLink · LinkForest · LinkTree · VisibilityProtocol · SubscriptionEdge · FederationEdge · ShadowRow · MirrorColumn |
| L7 — Identity Layer | KPID Multi-Identity | RootSpreadsheet · AccountMapping · ProfilePartition · IdentityAnchor · CrossIdentityMerge · VisibilityMask · SplitPolicy |

## **2.2  Core Structural Primitives**

Before describing the full row and column schemas, the following core structural primitives underpin every concept in this document:

| Primitive | Type | Definition |
| :---- | :---- | :---- |
| PortfolioComponent | Rust struct | The universal node. Every row in any sheet is a PortfolioComponent. It is either an Item (leaf work entity) or a Container (organizing structure). Carries ComponentMetadata \+ ComponentData. |
| ComponentId | UUID | Globally unique, immutable, 128-bit identifier for every PortfolioComponent. Never reused, never modified. |
| PortfolioRow | Rust struct \+ Go view layer | A PortfolioComponent projected as a spreadsheet row: flat key-value map of ColumnId → TypedCellValue, computed columns resolved, relationships joined. |
| PortfolioColumn | Rust enum | A typed field definition. Variants: TextField | NumberField | CurrencyField | DateField | EnumField | RelationField | UserField | TagField | ComputedColumn | AIColumn | FormulaColumn | AuditColumn |
| SheetDefinition | Go struct | A named, scoped, filterable view: base row type filter \+ visible column schema \+ default sort \+ default group \+ computed column set. |
| ViewDefinition | Go struct | A user-saved customization of a SheetDefinition: filters, sorts, groups, column widths, pinned columns, highlight rules, board config. |
| CellValue | Rust enum | The typed value at a row-column intersection. Variants: Text | Number | Currency | Date | Enum | Relation | User | Tag | Bool | Json | Null |
| EventLog | Rust VecDeque \+ PostgreSQL | Append-only, immutable log of every mutation on a PortfolioComponent. Every cell write is an EventLogEntry. |
| CrdtLog | Rust Vec | LWW \+ OR-Set CRDT operation log. Applied on merge to resolve concurrent edits without data loss. |
| VectorClock | Rust HashMap\<NodeId, u64\> | Logical clock per node. Provides causal ordering of all mutations across distributed nodes. |
| GraphEdge | Rust struct | A typed, directional relationship between two PortfolioComponents. Types: Hierarchy | Dependency | Link | Contains | Federation | ResourceShare | Attribution | Investment | Derives | InterPortfolioLink |
| InterPortfolioLink | GraphEdge subtype | A link crossing the boundary between two different users' root spreadsheets. The atomic unit of the KLNK link network. |

## **2.3  Component Taxonomy — Full Row Type Registry**

The following is the complete taxonomy of PortfolioComponents — every possible row type that can appear in the master spreadsheet system. All types share the universal PortfolioRow schema (§4) but activate different column groups and carry type-specific payload structs.

| Category | Row Type | Description | Key Columns Active |
| :---- | :---- | :---- | :---- |
| item:portfolio | Portfolio | A named top-level portfolio — the worker's entire work domain, or a specific practice area | mission, focus\_areas, kpis, health\_score, child\_count, budget\_rollup |
| item:portfolio | SubPortfolio | A scoped sub-portfolio nested within a parent Portfolio | parent\_portfolio\_id, scope, strategic\_goals, program\_rollup |
| item:program | Program | A strategic initiative grouping multiple related Projects toward a shared objective | objective, project\_count, budget, kpi\_achievement, program\_alignment\_score |
| item:project | Project | A bounded, deliverable-producing unit of work with methodology and sprints | project\_type, methodology, sprint\_velocity, backlog\_size, completion\_pct |
| item:resource | Resource | A human, financial, equipment, or service resource allocatable to work | resource\_type, skills, availability, hourly\_rate, utilization\_pct |
| item:artifact | Artifact | A typed output document, file, design, code, or generated asset | artifact\_type, file\_refs, produced\_by, maturity\_score, version\_count |
| item:asset | Asset | A capital or intellectual asset owned or managed | asset\_type, valuation, currency, acquired\_at, current\_value, roi\_pct |
| item:benefit | BenefitAccount | A portable benefit account (HSA, retirement, PTO, professional development) | benefit\_type, provider, balance, ytd\_contributions, coverage\_gap\_flag |
| item:gig | Gig | A single gig engagement generating income | platform, earnings\_gross, tips, portable\_contribution, mileage\_deduction |
| item:contract | Contract | A formal agreement with a client, cooperative, or employer | client\_ref, status, payment\_terms, deliverables, escrow\_status |
| item:job | Job | A longer-term employment or consulting engagement | rate, hours, organization\_ref, benefits\_linkage, contract\_type |
| item:task | Task | A single atomic unit of work within a project backlog | story\_type, priority, story\_points, assigned\_to, sprint\_ref, dependencies |
| item:campaign | Campaign | An equity crowdfunding, grants, or microfinancing campaign | campaign\_type, target\_amount, raised\_amount, investor\_count, close\_date |
| item:grant | Grant | A grant application or awarded grant with milestones and reporting | source, status, awarded\_amount, disbursed\_amount, milestone\_refs |
| item:investment | Investment | An equity stake, revenue-share instrument, or pool investment | instrument\_type, amount, units, vesting\_schedule, return\_to\_date |
| item:profile | Profile | A user or organization profile — surfaced in community and directory sheets | display\_name, skills, reputation\_score, connection\_count, link\_count |
| container:binder | Binder | A logical collection of items organized by shared purpose | item\_count, coverage\_pct, last\_activity |
| container:book | Book | A structured, navigable document-like container with typed subtypes | book\_type, section\_count, consistency\_score, last\_updated |
| container:record | Record | A formal, auditable record container | record\_type, integrity\_score, compliance\_flags |
| container:folder | Folder | A general-purpose hierarchical container | item\_count, depth, orphan\_count, organisation\_score |
| container:registry | Registry | A structured, searchable catalog of items | entry\_count, category, last\_indexed |
| container:archive | Archive | Deep-storage container for completed or historical items | archived\_item\_count, restore\_available, retention\_policy |

# **3\.  Multi-Account, Multi-Identity & Multi-Profile Architecture (KPID)**

## **3.1  Overview**

One of the most powerful and distinctive features of the Kogi Master Distributed Spreadsheet System is that a single worker's root spreadsheet can span multiple accounts, profiles, and identities — while the KLNK Link Network shows how all of these identities interconnect with each other and with the portfolios of other users. This is the KPID (Portfolio Identity) system.

| *Design Principle: Identity on Kogi is not one-to-one. A person may be a freelance developer under one identity, an artist collective member under another, a cooperative officer under a third, and an anonymous public voice under a fourth. The Master Spreadsheet respects this reality: a root spreadsheet is not tied to one account — it is tied to one sovereign entity (a person, org, or collective), which may be expressed through multiple accounts and profiles.* |
| :---- |

## **3.2  Identity Hierarchy**

| Concept | Description | Relationship to Spreadsheet |
| :---- | :---- | :---- |
| Sovereign Entity | The real-world person, organization, collective, or cooperative that owns the root spreadsheet. Has exactly one root PortfolioSystem instance as their master store. | One-to-one: one Sovereign Entity \= one Master Portfolio Spreadsheet root |
| Account | A login credential set (email+password, OAuth, passkey) associated with a Sovereign Entity. A Sovereign Entity may have multiple accounts (e.g. personal email \+ work email \+ social OAuth). | Accounts are authentication surfaces. All authenticated accounts for the same Sovereign Entity share read/write access to the full root spreadsheet, subject to per-account VisibilityMask. |
| Profile | A curated, public-facing identity projection of the Sovereign Entity. Profiles select which portfolio components are visible, which skills are highlighted, and which links are shown. | A Profile is a ViewDefinition applied to the Public sheet. Each profile has its own VisibilityMask defining which rows are public, followers-only, or private. |
| Identity | A named, purpose-specific persona with a separate @handle, display name, avatar, and optional pseudonymity. E.g. @jordan-dev (professional), @jordan-art (creative), @studiocollective (org). | Each Identity maps to a ProfilePartition — a subset of rows tagged to that identity. Rows can belong to multiple identities simultaneously. |
| ProfilePartition | A logical partition of the root spreadsheet's rows, owned by one Identity. Rows in a partition carry an identity\_tag\[\] that maps them to one or more profiles. | ProfilePartition defines which rows are visible on which profile. Cross-partition visibility is governed by VisibilityMask rules. |
| VisibilityMask | A per-account or per-profile filter specifying which rows and columns are visible to which observer. | Applied at query time: every GET request against the spreadsheet is filtered through the caller's VisibilityMask before returning data. |
| SplitPolicy | A governance policy governing how data is partitioned across identities. Enforces that certain rows (e.g. financial data, sensitive contracts) are only visible to specific identity tiers. | SplitPolicy is enforced by the PolicyEngine at the Substrate layer — no query can bypass it. |

## **3.3  Root Spreadsheet Splitting**

When a Sovereign Entity creates multiple identities or operates across multiple organizational roles, their root spreadsheet can be "split" — logically partitioned across identity boundaries — while remaining a single unified data store underneath. Splitting is a view-layer operation: the data is never physically duplicated.

### **3.3.1  Split Operations**

| Operation | Description | Reversible? |
| :---- | :---- | :---- |
| CreateIdentity | Create a new Identity (@handle) under the Sovereign Entity. Provisions a new ProfilePartition. No rows are moved — they are tagged. | Yes — identities can be merged or deleted |
| TagRowToIdentity | Assign one or more identity\_tags to a row, making it part of one or more ProfilePartitions. A row may belong to multiple partitions simultaneously. | Yes — tags can be added/removed |
| SetVisibilityMask | Define which rows/columns are visible to observers of a specific identity profile. Observer types: Public | Followers | Connections | Collaborators | Self | Yes — masks can be updated |
| IdentityIsolation | Mark an identity as Isolated: rows tagged only to this identity are completely hidden from all other identity views, even when authenticated as the same Sovereign Entity. | Yes — isolation can be lifted |
| CrossIdentityMerge | When the same Sovereign Entity has operated multiple identities and decides to unify them, CrossIdentityMerge re-tags all rows from the source identity to the target, preserving full history. | Partial — history preserved; cannot un-merge |
| IdentityHandoff | Transfer one Identity (including all its tagged rows) to a different Sovereign Entity — e.g. when a solo practitioner spins a collective identity into an independent organization. | No — creates a new root spreadsheet for recipient |
| FederatedIdentity | Link an identity to an external platform identity (GitHub, LinkedIn, DID/verifiable credential). Federated identities can receive link connections from external platforms. | Yes — links can be revoked |

### **3.3.2  ProfilePartition Schema**

Each ProfilePartition is a lightweight struct referencing the root spreadsheet. It carries no data of its own — it is a view configuration pointing into the master component registry.

| Field | Type | Description |
| :---- | :---- | :---- |
| partition\_id | UUID | Unique identifier for this partition |
| identity\_handle | String | @handle for this identity (unique within Sovereign Entity) |
| sovereign\_entity\_id | EntityId | Owning Sovereign Entity |
| display\_name | String | Public display name for this identity |
| avatar\_ref | Option\<FileId\> | Profile image for this identity |
| bio | Option\<String\> | Bio text for public profile |
| visibility\_mask | VisibilityMask | Row and column visibility rules for public observers |
| identity\_tag | String | Tag applied to rows belonging to this identity |
| isolation\_level | IsolationLevel | None | SoftIsolated | HardIsolated — controls cross-identity data access |
| linked\_accounts | Vec\<AccountId\> | Accounts that can authenticate as this identity |
| public\_sheet\_view\_id | ViewId | The ViewDefinition rendered when someone visits this identity's public profile |
| follower\_sheet\_view\_id | ViewId | View rendered for followers/subscribers |
| connection\_sheet\_view\_id | ViewId | View rendered for trusted connections |
| link\_network\_node\_id | LinkNodeId | This identity's node in the KLNK link network graph |
| created\_at / updated\_at | DateTime | Timestamps |

## **3.4  Multi-Account Authentication & Spreadsheet Access**

A Sovereign Entity may have any number of accounts. Each account has a configurable access scope defining which partitions it can read and write. This allows fine-grained separation — e.g. a "client-facing" account that can only see the Work & Gigs partition, while the full account has access to finances and benefits.

| Account Type | Default Access Scope | Use Case |
| :---- | :---- | :---- |
| Primary Account | Full root spreadsheet — all partitions, all columns | Personal daily use; full portfolio management |
| Work Account | Work & Gigs \+ Projects \+ Contracts partitions; financial summary only | Share with collaborators or embed in a work context |
| Organization Account | Org-tagged rows \+ shared portfolio components | When acting as officer or member of an org with a separate login |
| Public Account | Only rows where visibility \= Public in all partitions | Published portfolio / showcase presence |
| API Account | Scoped by API key permissions — read-only or specific sheet access | Third-party integrations, BI tools, automated pipelines |
| Delegated Account | Defined by AccountDelegation policy — can act on behalf of Sovereign Entity within policy bounds | Virtual assistants, agents, authorized representatives |

# **4\.  Universal Row Schema (PortfolioRow)**

| *Every row in the master spreadsheet implements the PortfolioRow schema — a superset of ComponentMetadata \+ ComponentData \+ all derived and computed fields. Rows are typed: the active columns for a given row are determined by its ItemType or ContainerType, but the underlying storage schema is universal so that cross-type aggregation, filtering, and sorting works uniformly across all sheets.* |
| :---- |

## **4.1  Column Group Overview**

| Column Group | Column Count | Description |
| :---- | :---- | :---- |
| Identity | 9 | Immutable identifier, slug, type discriminants, display fields, color, icon |
| Taxonomy | 8 | Type, subtype, category, labels, tags, themes, domain, lifecycle\_stage |
| Lifecycle | 8 | Status, state, visibility, lifecycle\_stage, timestamps, soft-delete |
| Ownership & Users | 12 | Owners, editors, contributors, viewers, watchers, subscribers, followers, investors, donors |
| Relationships | 11 | Parent, children, dependencies, dependents, linked\_ids, federation\_edges, link\_network\_edges |
| Identity & Profile | 8 | Identity\_tags, partition\_ids, visibility\_masks, public\_view\_id, account\_visibility |
| Version Control | 5 | Version, version\_history, last\_actor, bump\_type, changelog |
| Timeline & Schedule | 12 | Start, end, due, estimated\_duration, actual\_duration, progress\_pct, schedule\_variance, milestones |
| Financial & Budget | 16 | Budget\_allocated, budget\_spent, budget\_remaining, utilization\_pct, revenue, expenses, profit, roi, rate, rate\_type, tax\_category, invoices, bank\_account\_ref |
| Resource Allocation | 8 | Resource\_type, capacity, allocated, consumed, utilization\_pct, over\_committed, allocation\_refs |
| Work & Delivery | 10 | Story\_type, priority, story\_points, sprint\_ref, velocity, throughput, cycle\_time, lead\_time, acceptance\_criteria, delivery\_evidence |
| Governance & Policy | 9 | Policy\_ids, governance\_model, approval\_status, compliance\_flags, risk\_flags, charter\_ref, regulatory\_tags, audit\_trail |
| Analytics & Metrics | 18 | Health\_score, risk\_score, match\_score, views, clicks, ctr, shares, followers, kpi\_refs, okr\_refs, velocity, cycle\_time, collaboration\_score, sentiment\_score, anomaly\_flags |
| Visibility & Access | 5 | Visibility, access\_list, invite\_ids, share\_tokens, embed\_config |
| Tags & Discovery | 6 | Tags, hashtags, topics, labels, categories, search\_keywords |
| AI & Engine Signals | 10 | Oba\_hint, anomaly\_flags, recommendation\_score, predicted\_completion, budget\_risk\_flag, schedule\_risk\_flag, match\_score, income\_projection, grant\_match\_score, benefits\_gap\_flag |
| Collaboration | 8 | Contributor\_count, contribution\_types\_open, review\_policy, attribution\_weights, merge\_conflicts, governance\_model, distribution\_pending |
| Benefits | 11 | Benefit\_type, provider, balance, ytd\_contributions, contribution\_sources, vesting\_pct, pto\_accrued, coverage\_gap\_flag, tax\_treatment, tax\_savings, claim\_count |
| Gig & Contract | 11 | Platform, engagement\_type, earnings\_gross, tips, portable\_contribution, mileage, mileage\_deduction, payment\_status, contract\_status, deliverables, escrow\_status |
| Link Network | 9 | Link\_count, inbound\_link\_count, outbound\_link\_count, linked\_portfolio\_ids, link\_visibility, shadow\_row\_count, mirror\_column\_count, federation\_node\_ids, link\_depth |

## **4.2  Identity Column Group**

| Column | Type | Description |
| :---- | :---- | :---- |
| component\_id | UUID (immutable) | Globally unique immutable identifier for this row. Never changes after creation. |
| slug | String (unique/owner) | URL-safe short identifier. Unique within the owner's namespace. Used in API paths and embed URLs. |
| component\_type | ComponentType enum | Item | Container — top-level type discriminant controlling which column groups are active |
| item\_type | Option\<ItemType\> | Portfolio | Program | Project | Resource | Artifact | Asset | BenefitAccount | Gig | Contract | Job | Task | Campaign | Grant | Investment | Profile |
| container\_type | Option\<ContainerType\> | Binder | Book | Record | Folder | Registry | Archive |
| name | String | Primary display name of the component. Shown on all cards, search results, and row headers. |
| display\_name | Option\<String\> | Override display name for compact views (e.g. short title for board cards, abbreviated name for columns) |
| icon | Option\<String\> | Icon reference: emoji character, icon set key (e.g. "heroicons/folder"), or custom URL |
| color | Option\<HexColor\> | Color tag for board and calendar views. Used in swimlane backgrounds, card accent bars, and calendar event color. |

## **4.3  Lifecycle Column Group**

| Column | Type | Values / Description |
| :---- | :---- | :---- |
| status | ComponentStatus | Draft | Active | Paused | Completed | Archived | Cancelled | Suspended | UnderReview | Rejected | Custom(String) |
| state | ComponentState | Initializing | Configured | Running | Idle | Blocked | Failing | Recovering | Migrating | Locked | Sealed | Custom(String) |
| visibility | Visibility | Private | Protected | Internal | Public | Unlisted | DraftOnly | Custom — governs link network discovery and public sheet visibility |
| lifecycle\_stage | LifecycleStage | Concept | Planning | Execution | Review | Delivery | Complete | Post-Mortem — semantic stage overlay on top of status |
| created\_at | DateTime\<Utc\> | Row creation timestamp. Immutable. Set by the Substrate layer on create\_item() or create\_container(). |
| updated\_at | DateTime\<Utc\> | Last mutation timestamp. Auto-updated on every cell write. Used as primary sort key for "recently updated" views. |
| archived\_at | Option\<DateTime\<Utc\>\> | Archive timestamp. Set on transition to Archived status. Null for non-archived rows. |
| deleted\_at | Option\<DateTime\<Utc\>\> | Soft-delete timestamp. Set on soft-delete. Rows with deleted\_at are excluded from all views unless explicitly queried via SHT-028 Archive Sheet. |

## **4.4  Financial & Budget Column Group (Extended)**

The financial column group is the most comprehensive of all column groups, reflecting the centrality of financial management to the independent worker experience. These columns are active on all row types that carry financial data.

| Column | Type | Applies To | Computation / Notes |
| :---- | :---- | :---- | :---- |
| budget\_allocated | Option\<Decimal\> | Program, Project, Gig, Contract | Manually set or derived from ResourceAllocation. Null if no budget defined. |
| budget\_spent | Decimal | Program, Project, Gig | Cumulative spend recorded via record\_spend(). Auto-updated on each expense transaction. |
| budget\_remaining | ComputedColumn | Program, Project | \= budget\_allocated \- budget\_spent. Negative if over budget. |
| budget\_utilization\_pct | ComputedColumn | Program, Project | \= (budget\_spent / budget\_allocated) x 100\. Flagged amber at 80%, red at 100%. |
| currency | CurrencyCode | All financial rows | ISO 4217 currency code. Multi-currency support: all values normalized to user's base currency for aggregation. |
| rate | Option\<Decimal\> | Gig, Contract, Job, Resource | Hourly, daily, project, or gig rate. Source for earnings computation. |
| rate\_type | RateType | Gig, Contract, Job | Hourly | Daily | Weekly | Monthly | ProjectFixed | PerUnit | Revenue% | Subscription |
| revenue | Decimal | Gig, Contract, Job, Campaign, Investment | Cumulative revenue generated. Updated by kogi-bank on payment receipt. |
| expenses | Decimal | Project, Program, Asset, Gig | Cumulative expenses. Updated by kogi-bank on expense transaction. |
| profit | ComputedColumn | All financial rows | \= revenue \- expenses. Key P\&L metric. |
| roi | ComputedColumn | Asset, Investment, Campaign | \= (revenue \- budget\_spent) / budget\_spent x 100\. Return on Investment percentage. |
| tax\_category | TaxCategory | All financial rows | Business | SelfEmployment | PassThrough | Exempt | Investment | Benefit — drives tax report generation |
| payment\_terms | Option\<String\> | Contract, Gig, Job | Net-30, Net-60, milestone, upfront, subscription — surfaced in Invoices Sheet |
| invoice\_refs | Vec\<InvoiceId\> | Contract, Gig, Job | References to kogi-bank invoices linked to this component. Count shown as badge. |
| payment\_status | PaymentStatus | Contract, Gig, Job | Unpaid | Partial | Paid | Overdue | Disputed. Synced from kogi-bank invoice status. |
| bank\_account\_ref | Option\<AccountId\> | All financial rows | kogi-bank account receiving or funding this component's transactions. |

## **4.5  AI & Engine Signal Column Group**

These columns are written exclusively by the kogi-engine sub-systems via the plugin writeback protocol. They are read-only from the user's perspective. They are cached in the CellStore with a TTL and invalidated on relevant EventLog events.

| Column | Source Engine | Description |
| :---- | :---- | :---- |
| health\_score | AnalyticsEngine \+ RiskEngine | PortfolioHealth model output (0-100): composite of delivery, financial, risk, resource, engagement, governance, and benefit dimensions. Primary AI column. |
| risk\_score | RiskEngine | Probability-weighted aggregate of risk\_flags (0-100). Lower is better. Drives Red/Amber/Green status indicator. |
| match\_score | MatchEngine | Relevance score for this row in the context of a current search or opportunity query. Context-dependent. |
| recommendation\_score | RecommendationEngine | How highly this row is recommended to the current viewer based on their persona and interaction history. |
| predicted\_completion\_date | OptimizationEngine | Monte Carlo simulation-derived completion date estimate with confidence interval. Project rows only. |
| schedule\_risk\_flag | RiskEngine | Boolean: true if predicted\_completion\_date exceeds due\_date at \>50% probability. |
| budget\_risk\_flag | RiskEngine | Boolean: true if projected budget\_spent at current burn rate will exceed budget\_allocated before project end. |
| income\_projection | IncomeProjection model | Projected income from this row over the next 30/60/90 days. Gig, Contract, Investment rows only. |
| grant\_match\_score | GrantEngine | Eligibility-weighted match score for open grants in the registry. Grant rows only. |
| benefits\_gap\_flag | BenefitsEngine | Coverage gap detected for this benefit type. BenefitAccount rows only. |
| oba\_hint | Oba AI overlay | Latest Oba AI annotation for this row — surfaced as inline tooltip or sidebar card. Plain text; updated on each Oba analysis cycle. |
| anomaly\_flags | TelemetryEngine | Vec\<AnomalyFlag\> — engine-detected anomalies: budget spike, velocity drop, engagement cliff, risk escalation. Each flag carries type, severity, detected\_at. |

## **4.6  Link Network Column Group**

These columns describe a row's position and connectivity in the KLNK Link Network. They are computed from the GraphEdge store and updated on every InterPortfolioLink create/delete event.

| Column | Type | Description |
| :---- | :---- | :---- |
| link\_count | ComputedColumn (u32) | Total outbound \+ inbound inter-portfolio links on this row. High link\_count indicates a highly networked component. |
| inbound\_link\_count | ComputedColumn (u32) | Number of other users' spreadsheets that link to this row. Measure of external interest and reputation. |
| outbound\_link\_count | ComputedColumn (u32) | Number of rows in other users' spreadsheets that this row links to. Measure of connectivity. |
| linked\_portfolio\_ids | Vec\<PortfolioId\> | IDs of root spreadsheets that have an active InterPortfolioLink to/from this row. |
| link\_visibility | LinkVisibility | Who can see this row's links: Public | Followers | Connections | Private. Controls KLNK discovery. |
| shadow\_row\_count | ComputedColumn (u32) | Number of ShadowRows created in other users' spreadsheets that mirror this row. |
| mirror\_column\_count | ComputedColumn (u32) | Number of MirrorColumns from this row visible in linked spreadsheets. |
| federation\_node\_ids | Vec\<NodeId\> | CRDT federation nodes that replicate this row. Empty for non-federated rows. |
| link\_depth | ComputedColumn (u32) | Shortest path distance in the link graph to the farthest connected portfolio. 0 \= isolated, 1 \= direct connection, 2+ \= second-degree. |

# **5\.  The Link Network, Forest & Tree Architecture (KLNK)**

## **5.1  Overview**

The KLNK Link Network is the global inter-portfolio graph of the Kogi platform. It is the distributed data structure that maps every connection between every portfolio, portfolio component, profile, and identity across every user, organization, collective, cooperative, and federation on the platform.

| *Design Principle: If the Master Portfolio Spreadsheet is the node, the KLNK Link Network is the edge set. Every time a user connects their portfolio to another's — by following, subscribing, collaborating, investing, linking a dependency, or joining a federation — they are adding a typed, directional edge to this global graph. The network of all these edges is the Kogi economy graph: the complete, queryable picture of how all independent work on the platform is interconnected.* |
| :---- |

The KLNK system models the link network at four levels of granularity:

* Component-level links: individual PortfolioComponents linked to individual components in other users' spreadsheets

* Portfolio-level links: one user's entire portfolio linked to another's — broad connections establishing general interest or collaboration

* Identity-level links: one identity (@handle) connected to another, establishing the social graph

* Federation-level links: entire PortfolioSystem instances (nodes) federated for CRDT sync and cross-node governance

## **5.2  KLNK Graph Model**

| Concept | Description | Data Model |
| :---- | :---- | :---- |
| LinkNode | Any entity that can be a vertex in the KLNK graph: a PortfolioComponent, Profile, Identity, Organization, Portfolio, or Federation Node. | LinkNode { node\_id: NodeId, node\_type: LinkNodeType, owner\_entity\_id: EntityId, visibility: LinkVisibility, metadata: ComponentMetadata } |
| LinkEdge | A typed, directional edge between two LinkNodes in the KLNK graph. The atomic unit of inter-portfolio connection. | LinkEdge { edge\_id: EdgeId, from\_node: NodeId, to\_node: NodeId, edge\_type: LinkEdgeType, visibility: LinkVisibility, weight: f64, created\_at: DateTime, metadata: Map\<String,Value\> } |
| LinkForest | The complete set of all LinkTrees rooted at a given Sovereign Entity. A user's "link forest" is all the trees that fan out from all their identities and portfolio components. | LinkForest { root\_entity\_id: EntityId, trees: Vec\<LinkTree\>, total\_node\_count: u64, total\_edge\_count: u64, depth\_distribution: Map\<u32, u64\> } |
| LinkTree | A rooted, directed subgraph of the KLNK graph rooted at a single LinkNode. The tree represents the reachable graph from that node following all outbound edges up to a configured depth. | LinkTree { root\_node: NodeId, edges: Vec\<LinkEdge\>, nodes: Vec\<LinkNode\>, max\_depth: u32, edge\_type\_filter: Option\<Vec\<LinkEdgeType\>\> } |
| ShadowRow | When User A links their portfolio component to User B's component, a ShadowRow is created in User B's spreadsheet — a read-only reflection of the linked component from User A's spreadsheet, rendered as a row in the relevant sheet. | ShadowRow { shadow\_id: UUID, source\_component\_id: ComponentId, source\_owner\_entity\_id: EntityId, visible\_columns: Vec\<ColumnId\>, update\_policy: ShadowUpdatePolicy, last\_synced: DateTime } |
| MirrorColumn | A subset of columns from a ShadowRow made visible as a ComputedColumn in the host spreadsheet's view. User B can choose to mirror specific columns from User A's linked component into their own sheet. | MirrorColumn { mirror\_id: UUID, shadow\_row\_id: UUID, source\_column\_id: ColumnId, target\_column\_id: ColumnId, transform: Option\<ColumnTransform\> } |

## **5.3  Link Edge Types**

| Edge Type | Direction | Description | Creates ShadowRow? |
| :---- | :---- | :---- | :---- |
| Follows | A → B | User A follows User B's profile or portfolio component. A sees B's public updates in their feed. B's follower\_count increases. | No — subscription only |
| Subscribes | A → B | A paid or privileged follow. A receives subscriber-tier content from B's portfolio. Governed by B's subscription terms. | Optional — if B grants subscriber ShadowRow access |
| Watches | A → B (component) | A watches a specific portfolio component of B. A receives all state-change notifications for that component. | Optional — watcher may create a ShadowRow |
| Collaborates | A ↔ B | A bidirectional collaboration link. A and B are co-contributors on a shared portfolio component. Both have ContributionRecord entries. | Yes — mutual ShadowRows for shared component |
| InvestedIn | A → B (campaign/asset) | A has committed capital to B's campaign, equity round, or revenue share instrument. A receives investor-tier data access. | Yes — investor ShadowRow with financial columns visible |
| Donated | A → B (campaign) | A made a donation to B's campaign or cause. Non-equity, non-return donation tracked in link graph. | No — ledger entry only |
| Contracted | A → B | A has an active or historical contract engagement with B. Both parties' contract rows are linked bidirectionally. | Yes — both parties see the deal in their Contracts sheet |
| DependsOn | A → B (component) | A's portfolio component has a declared dependency on B's component. Used for cross-portfolio critical path analysis. | Yes — B's component appears as dependency row in A's graph |
| ResourceShares | A → B (resource) | A shares a portfolio resource (template, playbook, tool, data, compute) with B at a specified access level. | Yes — ShadowRow of the shared resource in B's Resources sheet |
| FederationPeer | A ↔ B (system) | Two PortfolioSystem nodes federated for CRDT sync. All federated components replicate across the link. | Yes — all replicated components appear as rows in both systems |
| OrgMembership | User → Org | A user is a member of an organization. The org's shared portfolio components are visible to the member. | Yes — org's shared components appear in member's Shared Portfolios sheet |
| Mentors | A → B | A has a mentorship relationship with B. A's relevant portfolio components are visible to B as learning resources. | Optional — configured by mentor |
| Endorses | A → B (skill/component) | A endorses a specific skill or portfolio component of B. Contributes to B's reputation\_score. | No — endorsement record only |
| References | A → B (component) | A explicitly references B's component in their own portfolio documentation or proposal. | No — lightweight citation link |
| CampusLink | Platform → User | A platform-initiated link connecting a user to a platform resource, template, or grant. Used for onboarding flows. | Yes — platform resource appears as ShadowRow |

## **5.4  Forest & Tree Rendering**

The KLNK system provides multiple rendering modes for the link graph, each optimized for different use cases. All modes query the same underlying LinkEdge store; they differ only in how they traverse, filter, and visualize the graph.

| Rendering Mode | Description | Max Depth | Primary Use Case |
| :---- | :---- | :---- | :---- |
| Ego Network View | A force-directed graph centered on the current user's root identity node, showing all first-degree and second-degree connections as a radial network. | Depth 2 | Social graph overview; discover mutual connections; identify collaboration opportunities |
| Link Forest View | All LinkTrees rooted at the current user's identities and portfolio components, rendered side-by-side as a multi-root forest layout. | Depth 3 | Full connectivity overview; identify isolated components; find bridge nodes |
| Component Tree View | A single LinkTree rooted at a specific portfolio component, showing all inbound and outbound links as a tree with branching paths. | Depth 5 | Understand a single component's full network context; due diligence for deals |
| Dependency Tree View | A filtered LinkTree showing only DependsOn edges. Rendered as a top-down dependency diagram with critical path highlighting. | Unlimited (cycle-safe) | Cross-portfolio critical path analysis; dependency risk assessment |
| Investment Graph | A filtered graph showing InvestedIn and Contracted edges only. Reveals the financial network of deals, investments, and capital flows. | Depth 3 | Investor portfolio visualization; deal network analysis |
| Federation Graph | A system-level graph showing FederationPeer edges between PortfolioSystem nodes. Rendered as a network topology diagram. | Full graph | Platform infrastructure overview; CRDT sync health monitoring |
| Organization Tree | An org chart-style tree showing OrgMembership edges from a root organization down through all member users and sub-organizations. | Depth 4 | Org structure visualization; governance participation overview |
| Collaboration Web | A bipartite graph showing Collaborates edges between users and shared portfolio components. | Depth 3 | Team composition analysis; contribution attribution visualization |
| Resource Sharing Map | A directed graph of ResourceShares edges showing how resources (templates, tools, data) flow across users and organizations. | Depth 3 | Resource utilization; identify shared assets; license compliance |
| Spreadsheet Grid View | The KLNK link graph rendered as a spreadsheet sheet (SHT-031 Link Network Sheet): each row is a LinkEdge, columns are from/to node details, edge type, weight, visibility, and shadow row status. | N/A | Data export; bulk link management; programmatic access |

## **5.5  ShadowRow Protocol**

ShadowRows are the mechanism by which linked portfolio components from other users appear as rows in your own spreadsheet. They are one of the most powerful features of the KLNK system — enabling cross-portfolio visibility, shared project tracking, and investment monitoring without requiring full data replication.

### **5.5.1  ShadowRow Lifecycle**

1. Link Created: When a LinkEdge is created between User A's component and User B's component (e.g. A contracts with B), the system evaluates whether a ShadowRow should be created in B's spreadsheet based on the edge type and B's ShadowRowPolicy.

2. Negotiation: If the edge type requires both parties' consent for ShadowRow creation (e.g. Collaborates), an invitation is sent to B. B configures which columns of their component are visible in A's ShadowRow.

3. Provisioning: ShadowRow is created in the target spreadsheet's registry with source\_component\_id pointing to A's component. The ShadowRow is marked as read-only.

4. Sync: On every update to the source component, a ShadowUpdateEvent is published. The target spreadsheet receives the event and updates the ShadowRow's MirrorColumn values according to the configured update policy (RealTime | Batched | Manual).

5. Visibility: The ShadowRow appears in relevant sheets (e.g. Shared Portfolios, Contacts, Contracts) alongside the host user's own rows, clearly marked with a "linked" indicator and the source identity's @handle.

6. Disconnection: When the LinkEdge is removed, the ShadowRow transitions to status=Disconnected. The host user can choose to Archive it (preserve history) or Delete it (remove from spreadsheet). Historical data is always preserved in the EventLog.

| ShadowRow Property | Description |
| :---- | :---- |
| source\_component\_id | ComponentId of the source component in the source user's spreadsheet |
| source\_owner\_identity | @handle of the source identity |
| visible\_columns | Vec\<ColumnId\> — only these columns are populated in the ShadowRow. Other columns are null. |
| update\_policy | RealTime | Batched(every\_n\_hours) | Manual — how often MirrorColumns are refreshed from source |
| write\_back\_columns | Vec\<ColumnId\> — columns the host user can write to, which are then synced back to the source (e.g. for comment annotations on contracts) |
| shadow\_status | Active | Pending | Disconnected | Archived — lifecycle state |
| last\_synced | DateTime — timestamp of last successful sync from source |
| sync\_error | Option\<String\> — last sync error if status is degraded |

## **5.6  Link Discovery & Graph Traversal API**

The KLNK system exposes a rich query API for traversing, filtering, and analyzing the link graph. All queries are subject to the VisibilityMask of the target nodes — no query can see nodes or edges that the caller is not permitted to access.

| API Endpoint | Method | Description |
| :---- | :---- | :---- |
| /klnk/nodes/{node\_id}/tree | GET | Get LinkTree rooted at node\_id with configurable depth, edge\_type filter, and direction (outbound|inbound|both) |
| /klnk/nodes/{node\_id}/forest | GET | Get LinkForest for all nodes owned by the same entity as node\_id |
| /klnk/nodes/{node\_id}/ego | GET | Get ego network (depth 1+2) centered on node\_id — all direct and second-degree connections |
| /klnk/edges | POST | Create a new LinkEdge (link request). Subject to permission checks and edge-type consent requirements. |
| /klnk/edges/{edge\_id} | DELETE | Remove a LinkEdge. Triggers ShadowRow disconnection protocol. |
| /klnk/shadow-rows | GET | List all ShadowRows in the caller's spreadsheet — linked rows from other users' portfolios |
| /klnk/shadow-rows/{shadow\_id}/sync | POST | Manually trigger a sync of a ShadowRow's mirror columns from source |
| /klnk/discover | GET | Discover potential link targets: users, components, and orgs matching a query, filtered by visibility and ranked by match\_score |
| /klnk/graph/path | GET | Find shortest path between two nodes in the link graph (Dijkstra with edge weight). Used for "degrees of separation" queries. |
| /klnk/graph/clusters | GET | Identify community clusters in the link graph (Louvain community detection). Used for org and ecosystem analysis. |
| /klnk/graph/centrality | GET | Compute betweenness, closeness, and eigenvector centrality for nodes. Used for influence scoring. |
| /klnk/subscriptions | GET/POST/DELETE | Manage link change subscriptions — receive webhooks or Kafka events when linked nodes change |

# **6\.  Sheet System — Derived Sheet Taxonomy**

| *A Sheet is a named, scoped, materialized view of the PortfolioComponent registry. Every sheet is defined by: a SheetDefinition (filter predicates, visible columns, default sort, default grouping), a ColumnSchema (the ordered set of columns visible on this sheet), and an optional ComputedColumnSet (additional engine-derived columns rendered only on this sheet). Sheets are not separate databases — they are views over the single underlying PortfolioRow store.* |
| :---- |

## **6.1  Master Sheet Registry — All 35 Sheets**

| Sheet ID | Sheet Name | Primary Row Types | Default Group By | Key Use Case |
| :---- | :---- | :---- | :---- | :---- |
| SHT-001 | Master Registry | All types | Type then Status | Root view of all components; global search and bulk operations |
| SHT-002 | Portfolio Hierarchy | Portfolio, SubPortfolio, Program, Project | Hierarchy (tree) | Visualize structural tree with rollup metrics |
| SHT-003 | Programs | Program | Status | Strategic initiative tracking and KPI alignment |
| SHT-004 | Projects | Project | Program → Status | Operational project management with methodology and sprint data |
| SHT-005 | Tasks & Backlog | Task, Story, Epic, Feature | Project → Sprint | Work execution: backlog, sprint, kanban |
| SHT-006 | Resources | Resource | Resource Type | Manage human, machine, license, and service resources |
| SHT-007 | Assets | Asset | Asset Type | Capital assets, IP, financial assets, digital assets |
| SHT-008 | Artifacts | Artifact | Project → Artifact Type | Versioned outputs: documents, designs, code, reports |
| SHT-009 | Finances | All financial rows | Quarter → Category | Unified P\&L: income, expenses, budget across all engagements |
| SHT-010 | Budget Tracker | Program, Project, Gig, Contract | Program → Status | Budget allocation, spend, remaining, and utilization |
| SHT-011 | Work & Gigs | Gig, Contract, Job, Task | Platform → Status | All income-generating engagements across all platforms |
| SHT-012 | Deliverables | Project, Artifact, Release | Project → Status | Output tracking: what was produced and when |
| SHT-013 | Timeline | All dated rows | Quarter → Program | Temporal view of all work with milestones |
| SHT-014 | Roadmap | Program, Project, Milestone | Program → Quarter | Strategic roadmap for external sharing |
| SHT-015 | Portable Benefits | BenefitAccount | Benefit Type | Benefits dashboard: balances, contributions, coverage gaps |
| SHT-016 | Grants & Microfinancing | Grant, Campaign (grant type) | Status | Grant pipeline from discovery to disbursement and reporting |
| SHT-017 | Equity & Crowdfunding | Campaign, Investment | Campaign Type | Fundraising, equity rounds, and investment positions |
| SHT-018 | Group Economics | Shared Portfolio, Revenue Pool, Org | Organization | Cooperative and collective financial structures |
| SHT-019 | Organizations | Profile (org type) | Org Type | All organizations, collectives, cooperatives the user is part of |
| SHT-020 | Shared Portfolios | Portfolio (shared) | Owner Org | Co-owned and shared portfolio components |
| SHT-021 | Collaboration | All shared/contributed rows | Contributor | Contribution tracking and attribution weights |
| SHT-022 | Risk Register | All rows with risk\_flags | Risk Severity | Unified risk dashboard across all components |
| SHT-023 | Analytics Dashboard | All rows | Type → Health Score | Health scores, performance metrics, AI signals |
| SHT-024 | Feed & Community | Profile, Post, Artifact (public) | Space | Community-facing content and social activity |
| SHT-025 | Contacts & Network | Profile, Contact | Relationship Type | CRM: client, collaborator, investor, and vendor contacts |
| SHT-026 | Marketplace Listings | Asset, Resource, Gig (public) | Category | Published marketplace items and their performance |
| SHT-027 | Exchange | Investment, Deal, Campaign, Asset | Exchange Type | Exchange market activity: bids, deals, instruments |
| SHT-028 | Archive | All archived rows | Archive Date | Deep storage with full restore capability |
| SHT-029 | Event Log | EventLog entries (read-only) | Date → Actor | Immutable audit trail of all mutations |
| SHT-030 | Snapshots | Snapshot records | Date | Point-in-time portfolio snapshots for restore |
| SHT-031 | Link Network | LinkEdge records | Edge Type → Target | All inter-portfolio link connections; KLNK graph in spreadsheet form |
| SHT-032 | Identity & Profiles | ProfilePartition records | Identity | Multi-identity management: all profiles, visibility masks, account mappings |
| SHT-033 | Portable Benefits Deep Dive | BenefitAccount \+ Gig (contribution) | Benefit Type → Platform | Gig platform contribution tracking and benefit projection |
| SHT-034 | Custom Sheet (User) | User-defined | User-defined | Ad-hoc views for specific workflows or client reports |
| SHT-035 | Template Library | ItemBook:Template rows | Category | Reusable templates available from the KOGI-APPSTORE and user's own library |

## **6.2  SHT-001: Master Registry Sheet**

The Master Registry is the root spreadsheet — the unfiltered, full-column view of every PortfolioComponent the authenticated user can access. It is the entry point for global search, bulk operations, cross-type analytics, and administrative management. Every other sheet is a subset or derived view of the Master Registry.

| Configuration Element | Value |
| :---- | :---- |
| Pinned Columns (always visible) | component\_id · slug · component\_type · item\_type · name · status · visibility · owners |
| Default Visible Columns | \+ created\_at · updated\_at · progress\_pct · health\_score · risk\_score · due\_date · budget\_remaining · tags · link\_count |
| Optionally Visible | All remaining 150+ columns — user-toggleable per session or saved view |
| Default Sort | updated\_at descending |
| Default Group | component\_type |
| Row Actions | Open · Edit · Duplicate · Archive · Share · Export · Add to Sheet · Pin · Set Status · Link to Network |
| Bulk Actions | Set Status · Set Visibility · Assign Tags · Archive · Export Selected · Add to Binder · Link to Org · Tag Identity |
| AI Overlay | Oba surfaces: anomalies, stale items (not updated \>30 days), health warnings, overdue items, missing coverage, and quick-action suggestions inline as row-level annotations |
| Link Network | link\_count column shows inter-portfolio connections. Click to open Component Tree View in KLNK. |

## **6.3  SHT-004: Projects Sheet — Detailed Column Schema**

The Projects Sheet is the primary operational view for project-level work. It exposes the full ProjectPayload alongside financial, governance, AI, and link network columns.

| Column Group | Columns | Notes |
| :---- | :---- | :---- |
| Identity | component\_id · name · slug · color · icon | Standard identity group |
| Classification | project\_type · methodology · domain · tags | ProjectType: Software | Design | Content | Research | Operations | Event | Custom |
| Lifecycle | status · state · lifecycle\_stage · visibility | Full lifecycle state machine with Oba annotations on transitions |
| Ownership | owners · editors · watchers | Multi-user role columns; avatar cluster display |
| Timeline | start\_date · end\_date · due\_date · progress\_pct · schedule\_variance\_days · predicted\_completion\_date | Key scheduling columns \+ AI prediction |
| Sprints | sprint\_count · current\_sprint · sprint\_velocity · backlog\_size · throughput | Derived from ProjectPayload.sprints; auto-updated on sprint completion |
| Releases | release\_count · latest\_release · next\_release\_date | Derived from ProjectPayload.releases |
| Finance | budget\_allocated · budget\_spent · budget\_remaining · budget\_utilization\_pct · revenue · profit · roi | Full financial tracking; kogi-bank synced |
| Governance | governance\_model · approval\_status · risk\_flags · compliance\_flags · policy\_ids | Governance state with pending approval count badge |
| Analytics | health\_score · risk\_score · cycle\_time\_days · lead\_time\_days · velocity · collaboration\_score · anomaly\_flags | AI-computed performance signals; all from kogi-engine |
| Work & Delivery | story\_type · priority · story\_points · open\_task\_count · completed\_task\_count · acceptance\_criteria\_count | Work execution metrics |
| AI Signals | predicted\_completion\_date · schedule\_risk\_flag · budget\_risk\_flag · oba\_hint · match\_score | Engine-generated forward-looking signals |
| Link Network | link\_count · inbound\_link\_count · linked\_portfolio\_ids · shadow\_row\_count | Cross-portfolio connections for this project |
| ToolBoxes | toolbox\_ids · toolbox\_names | Attached TMS ToolBoxes |

## **6.4  SHT-009: Finances Sheet — Extended Column Schema**

The Finances Sheet is a unified financial ledger view spanning all portfolio components that carry financial data. It aggregates income, expenses, budget, and asset values across gigs, contracts, projects, campaigns, benefits, and investments. It is the master financial spreadsheet — the equivalent of a real-time P\&L across the worker's entire portfolio of work.

| Column | Type | Applies To | Notes |
| :---- | :---- | :---- | :---- |
| name | String | All | Component display name |
| component\_type | ComponentType | All | Row type for period grouping |
| quarter | String | All | Fiscal quarter label (e.g. Q1-2026) |
| fiscal\_year | u32 | All | Fiscal year for budget alignment |
| revenue | Decimal | Gig, Contract, Job, Campaign, Investment | Income or revenue generated; kogi-bank synced |
| expenses | Decimal | Project, Program, Asset, Gig | Costs incurred; kogi-bank synced |
| budget\_allocated | Decimal | Program, Project | Planned budget envelope |
| budget\_spent | Decimal | Program, Project | Actual spend to date |
| budget\_remaining | ComputedColumn | Program, Project | \= allocated \- spent |
| profit | ComputedColumn | All | \= revenue \- expenses |
| roi | ComputedColumn | Asset, Investment, Campaign | Return on investment % |
| tax\_category | TaxCategory | All | Business | SelfEmployment | Investment | Benefit — for tax report generation |
| invoice\_refs | Vec\<InvoiceId\> | Contract, Gig, Job | Badge showing count; click to expand invoice list |
| payment\_status | PaymentStatus | Contract, Gig, Job | Unpaid | Partial | Paid | Overdue | Disputed — color-coded badge |
| gig\_earnings | Decimal | Gig rows | Base earnings before tips and deductions |
| tips | Decimal | Gig rows | Tip income; displayed separately for platform contribution calculation |
| portable\_contribution | Decimal | Gig rows | Platform contribution to portable savings (e.g. 4% of pre-tip) |
| mileage | Option\<Decimal\> | Gig rows | Miles driven; used for IRS mileage deduction calculation |
| mileage\_deduction | ComputedColumn | Gig rows | \= mileage x current IRS rate per mile |
| benefit\_contributions | Decimal | BenefitAccount rows | Cumulative contributions to benefit account YTD |
| benefit\_balance | Decimal | BenefitAccount rows | Current account balance from kogi-bank |
| investment\_value | Decimal | Investment, Asset rows | Current estimated valuation |
| distributions\_received | Decimal | Investment rows | Cumulative distributions received |
| grant\_awarded | Decimal | Grant rows | Total grant award amount |
| grant\_disbursed | Decimal | Grant rows | Amount disbursed to date |
| campaign\_target | Decimal | Campaign rows | Fundraising target amount |
| campaign\_raised | Decimal | Campaign rows | Amount raised to date |
| campaign\_pct | ComputedColumn | Campaign rows | \= raised / target x 100 |
| bank\_account\_ref | AccountId | All | Linked kogi-bank account |

## **6.5  SHT-015: Portable Benefits Sheet — Full Column Schema**

Every portable benefit account is a PortfolioItem (kind: BenefitAccount) and surfaces as a row on this sheet. This is the worker's complete, real-time benefits dashboard — balances, contributions, coverage gaps, and tax optimization signals all in one place.

| Column | Type | Benefit Types | Notes |
| :---- | :---- | :---- | :---- |
| name | String | All | Account display name |
| benefit\_type | BenefitType | All | Health | Dental | Vision | HSA | Retirement\_SEP | Retirement\_401k | Retirement\_PEP | PTO | Disability | WorkersComp | ProfessionalDev | EmergencySavings | PortableSavings | IncomeProtection |
| provider | String | All | Benefit provider name (e.g. Stride LLC, Fidelity, Cigna, TIAA) |
| status | ComponentStatus | All | Active | Inactive | Suspended | Archived |
| balance | Decimal | HSA, Retirement, Emergency, Portable, ProfDev | Current account balance from kogi-bank |
| ytd\_contributions | Decimal | HSA, Retirement, Portable | Year-to-date contributions from all sources |
| contribution\_sources | Vec\<ContributionSource\> | Portable | Gig platforms contributing (e.g. DoorDash 4%, Lyft 3%) with amounts |
| platform\_contribution\_ytd | Decimal | Portable | Total gig platform contributions YTD |
| self\_contribution\_ytd | Decimal | All | User-directed contributions YTD |
| vesting\_pct | ComputedColumn | Retirement | Vesting schedule completion percentage; cliff and schedule applied |
| projected\_balance\_1yr | ComputedColumn | Retirement, Portable | Engine-projected balance at 1 year given current contribution rate |
| projected\_balance\_5yr | ComputedColumn | Retirement | Engine-projected balance at 5 years (compounding model) |
| coverage\_gap\_flag | AnomalyFlag | All | BenefitsEngine flag: coverage gap detected — type \+ recommended action |
| pto\_accrued\_days | Decimal | PTO | Current accrued paid time off in days |
| pto\_used\_ytd\_days | Decimal | PTO | PTO used year-to-date |
| pto\_remaining\_days | ComputedColumn | PTO | \= accrued \- used |
| income\_replacement\_days | ComputedColumn | Disability, PTO, IncomeProtection | Coverage days vs. income gap risk — how many days of income are covered |
| tax\_treatment | TaxCategory | All | PreTax | PostTax | TaxFree — affects tax savings computation |
| tax\_savings\_ytd | ComputedColumn | HSA, Retirement | Estimated tax savings from pre-tax contributions YTD |
| irs\_contribution\_limit | ComputedColumn | HSA, Retirement | Current IRS annual contribution limit for this benefit type |
| remaining\_contribution\_room | ComputedColumn | HSA, Retirement | \= irs\_contribution\_limit \- ytd\_contributions |
| claim\_count | u32 | Health, Disability | Number of claims filed this year |
| linked\_bank\_account | AccountId | All | kogi-bank BenefitAccount linked to this portfolio item |

## **6.6  SHT-031: Link Network Sheet**

The Link Network Sheet is the spreadsheet representation of the KLNK graph — every inter-portfolio link the user has, presented as a queryable, filterable, exportable table of rows. Each row is a LinkEdge.

| Column | Type | Description |
| :---- | :---- | :---- |
| edge\_id | EdgeId | Unique identifier for this link edge |
| edge\_type | LinkEdgeType | Follows | Subscribes | Watches | Collaborates | InvestedIn | Donated | Contracted | DependsOn | ResourceShares | FederationPeer | OrgMembership | Mentors | Endorses | References |
| direction | ComputedColumn | Outbound (I linked to them) | Inbound (they linked to me) | Mutual |
| from\_node\_name | String | Display name of the source node (my component or identity) |
| from\_node\_type | LinkNodeType | PortfolioComponent | Profile | Identity | Organization | Portfolio | FederationNode |
| to\_node\_name | String | Display name of the target node (their component or identity) |
| to\_node\_owner\_handle | String | @handle of the user/identity who owns the target node |
| to\_node\_type | LinkNodeType | Type of the target node |
| visibility | LinkVisibility | How visible this link is: Public | Followers | Connections | Private |
| shadow\_row\_id | Option\<UUID\> | If a ShadowRow was created from this link, its ID. Click to navigate to the shadow row. |
| shadow\_status | Option\<ShadowStatus\> | Active | Pending | Disconnected — state of the shadow row if one exists |
| last\_synced | Option\<DateTime\> | Last time the shadow row was synced from source |
| weight | f64 | Edge weight in graph algorithms (0.0-1.0). Higher \= stronger connection. |
| created\_at | DateTime | When this link was established |
| link\_depth | ComputedColumn | Shortest path length to this node from the user's root identity |
| mutual\_connections | ComputedColumn | Count of mutual connections (shared nodes in both users' link graphs) |

# **7\.  View Engine (KPVW)**

## **7.1  ViewDefinition Schema**

The View Engine transforms raw PortfolioRow data into the shaped, filtered, sorted, grouped, and visually enriched spreadsheet that the user interacts with. Views are defined as ViewDefinition objects, serialized to JSON, versioned, and shareable.

| Field | Type | Description |
| :---- | :---- | :---- |
| id | UUID | View identifier |
| name | String | User-defined view name |
| sheet\_id | SheetId | Parent sheet this view belongs to (or "master" for cross-sheet views) |
| base\_row\_filter | Option\<ViewFilter\> | Pre-filter applied before any user interaction (e.g. "only rows owned by me") |
| column\_schema | ColumnSchema | Ordered list of visible columns with width, pin, and sort settings per column |
| filters | Vec\<ViewFilter\> | User-configurable filter predicates. Evaluated as AND within a group, OR across groups. |
| sorts | Vec\<ViewSort\> | Ordered sort predicates. Primary sort first, secondary second, etc. |
| groups | Vec\<ViewGroup\> | Hierarchical grouping definitions. First group is outermost heading row. |
| row\_height | RowHeight | Compact | Normal | Tall | Auto — controls card density in list view |
| highlight\_rules | Vec\<RowHighlightRule\> | Conditional color rules. E.g. health\_score \< 60 → red background |
| pinned\_columns | Vec\<ColumnId\> | Columns always visible regardless of horizontal scroll position |
| frozen\_row\_count | u32 | Number of rows pinned at top (e.g. summary or header rows) |
| pivot\_config | Option\<PivotConfig\> | Pivot table mode configuration: row\_field, col\_field, value\_field, aggregation |
| chart\_config | Option\<ChartConfig\> | Chart overlay: type (bar/line/pie/gauge), x\_column, y\_column, series |
| summary\_row | Option\<SummaryRowConfig\> | Aggregate row at sheet bottom: sum/avg/count/max/min per selected numeric columns |
| board\_config | Option\<BoardConfig\> | Kanban/Gantt/Calendar board mode settings |
| link\_network\_overlay | bool | When true, show link\_count badges on rows and allow click-to-open KLNK tree view |
| identity\_filter | Option\<IdentityTag\> | Filter to only show rows tagged to a specific identity/profile partition |
| visibility | ViewVisibility | Private | Shared | Public | Template — governs who can see and copy this view |
| created\_by / at / updated\_at | UserId / DateTime | Authorship and timestamps |

## **7.2  Filter System**

Filters are composable boolean predicates evaluated against row columns. Complex filter trees can be built by nesting AND and OR groups.

| Filter Type | Syntax | Example |
| :---- | :---- | :---- |
| Equality | column \= value | status \= Active |
| Inequality | column \!= value | visibility \!= Private |
| Numeric Range | column BETWEEN a AND b | health\_score BETWEEN 60 AND 100 |
| Greater/Less | column \> or \< value | budget\_utilization\_pct \> 80 |
| Contains (string) | column CONTAINS substring | name CONTAINS "API" |
| In Set | column IN \[v1, v2, ...\] | item\_type IN \[Project, Program\] |
| Not In Set | column NOT IN \[v1, ...\] | status NOT IN \[Archived, Cancelled\] |
| Date Range | column AFTER / BEFORE / BETWEEN dates | due\_date AFTER 2026-03-01 |
| Is Empty | column IS EMPTY | due\_date IS EMPTY |
| Is Not Empty | column IS NOT EMPTY | owners IS NOT EMPTY |
| Tag Includes | tags INCLUDES tag | tags INCLUDES "q2-initiative" |
| Owner Is Me | owners INCLUDES me | owners INCLUDES me |
| Identity Tagged | identity\_tags INCLUDES identity | identity\_tags INCLUDES "@jordan-dev" |
| Has Shadow Row | shadow\_row\_count \> 0 | Shows only rows visible to linked portfolios |
| Has Link | linked\_portfolio\_ids IS NOT EMPTY | Shows only rows with active inter-portfolio links |
| Engine Score | health\_score \>= threshold | health\_score \>= 70 |
| Has Flag | risk\_flags HAS\_SEVERITY HIGH | risk\_flags HAS\_SEVERITY HIGH |
| AND / OR Groups | (filter1 AND filter2) OR filter3 | (status=Active AND health\_score \< 60\) OR risk\_score \> 80 |

## **7.3  Board Modes**

Every sheet can be rendered in an alternative board mode. Board modes are view-layer transforms — the underlying PortfolioRow data is unchanged; only the rendering changes.

| Board Mode | Configuration | Best For |
| :---- | :---- | :---- |
| Kanban Board | Columns \= status values; cards show name, owner, due\_date, health\_score; WIP limits configurable per column | Task and project status management; visual WIP management |
| Gantt Chart | Rows \= timeline components; horizontal bars \= start\_date to end\_date; dependencies rendered as arrows; critical path highlighted | Project and program timeline planning; dependency management |
| Calendar | Rows with date columns placed on calendar grid by due\_date or start\_date; month/week/day views | Scheduling, deadlines, milestone tracking, booking management |
| Agile Board | Sprint-scoped Kanban with story point totals per column; burndown chart overlay; sprint velocity badge | Sprint planning and execution; Scrum and Kanban workflows |
| Resource Board | Rows \= resources; columns \= time periods; cells \= allocation %; red highlighting for over-allocation | Capacity planning and conflict detection (AllocationEngine) |
| Network Graph | Nodes \= rows; edges \= GraphEdges; force-directed layout; color by type; size by health\_score | Dependency visualization; portfolio relationship map; KLNK exploration |
| Treemap | Hierarchical tiles sized by a numeric column (e.g. budget\_allocated); color by health\_score; drill-down | Portfolio composition and budget distribution at a glance |
| Timeline Board | Swimlane rows \= programs; bars \= projects and milestones; nested structure; quarter markers | Cross-program roadmap view; strategic planning overview |
| Link Forest View | Nodes \= portfolio components and linked external nodes; tree structure by identity; KLNK rendering | Visualize all cross-portfolio connections; discover network structure |
| Identity Grid | Rows grouped by identity\_tag; columns show partition visibility settings; color by isolation\_level | Multi-identity management; profile visibility configuration |

# **8\.  Column System — Full Type Registry**

## **8.1  Column Types**

| Column Type | Description | Storage | Aggregations |
| :---- | :---- | :---- | :---- |
| TextField | Free text string; sortable, searchable, filterable; supports markdown preview in tall row mode | String, FTS-indexed by SearchEngine | COUNT, COUNT\_NON\_EMPTY |
| NumberField | Integer or decimal; supports all numeric aggregations; renders with configurable decimal places and thousands separators | Decimal, range-indexed | SUM, AVG, MIN, MAX, MEDIAN, COUNT |
| CurrencyField | Decimal with ISO 4217 currency code; multi-currency aware; auto-conversion to user base currency for aggregation | Decimal \+ CurrencyCode | SUM, AVG, MIN, MAX (in base currency) |
| PercentField | Decimal 0-100; renders with % suffix; progress bar visualization in tall row mode; sortable and range-filterable | f32 | AVG, MIN, MAX, MEDIAN |
| DateField | Calendar date; timezone-aware; relative display ("in 3 days", "2 weeks ago"); supports date math in formulas | Date, calendar-indexed | MIN (earliest), MAX (latest), RANGE |
| DateTimeField | Full timestamp; nanosecond precision for EventLog entries; timezone-aware; relative or absolute rendering | DateTime\<Utc\> | MIN, MAX, RANGE |
| DurationField | Time span; rendered as "3d 4h"; supports arithmetic; auto-computed from date range columns | Duration (seconds) | SUM, AVG, MIN, MAX |
| EnumField | One-of a defined value set; renders as colored badge; fast equality filter; configurable badge colors per value | String enum, hash-indexed | COUNT\_BY\_VALUE, MODE |
| MultiEnumField | Set of enum values; renders as multiple badge chips; "includes" filter; max configurable | Vec\<String\>, set-indexed | COUNT\_BY\_VALUE, INTERSECTION |
| RelationField | Foreign key reference to another PortfolioRow; renders as linked name chip; click-to-navigate; cross-sheet joins | ComponentId, join-resolved at query time | COUNT, COUNT\_UNIQUE |
| MultiRelationField | Set of foreign key references; renders as linked name chip cluster; max 50 relations by default | Vec\<ComponentId\> | COUNT, COUNT\_UNIQUE |
| UserField | Reference to a user or organization entity; resolves to display name \+ avatar; @mention trigger in cell edit | EntityId | COUNT\_UNIQUE |
| MultiUserField | Set of user/org references; renders as avatar cluster ("+N" for overflow); filterable by specific user | Vec\<EntityId\> | COUNT\_UNIQUE, LIST |
| TagField | Vec\<String\>; renders as tag chips; "includes" filter; tag autocomplete; global tag taxonomy | Vec\<String\>, tag-indexed | UNION, INTERSECTION, COUNT\_BY\_TAG |
| BoolField | Boolean; renders as checkbox; filterable; tri-state in some contexts (True/False/Null) | bool | COUNT\_TRUE, COUNT\_FALSE, PCT\_TRUE |
| RichTextField | Markdown-formatted rich text; not sortable; searchable; preview mode in card views | Markdown String | COUNT (non-empty) |
| FileRefField | Reference to a file or artifact stored in kogi-content; renders with file type icon and preview on hover | FileId, resolves to metadata \+ URL | COUNT |
| URLField | Web URL; renders as clickable link with domain favicon; link preview on hover | String | COUNT |
| ComputedColumn | Derived value computed by Computation Engine from other columns; read-only; recalculated on dependency change | Computed at query time; cached with TTL | All numeric aggregations |
| AIColumn | Engine-generated signal: score, flag, recommendation, prediction; read-only; written by kogi-engine via plugin writeback | Written by kogi-engine; cached; TTL-invalidated on EventLog events | AVG, MIN, MAX, MEDIAN |
| FormulaColumn | User-defined formula over other columns using ColumnComputer expression language; power user feature | Evaluated by ColumnComputer; cached | All numeric aggregations |
| AuditColumn | Derived from EventLog; shows last actor, last action, action count, creation info | Derived from EventLog at query time | COUNT |
| AnalyticsColumn | Aggregated engagement or performance metric (view count, click count, engagement rate); updated by AnalyticsEngine | Updated by AnalyticsEngine; stored in materialized view | SUM, AVG, MAX |
| LinkNetworkColumn | Computed from KLNK GraphEdge store: link\_count, linked\_identities, shadow\_row\_count, link\_depth | Computed from GraphEdge store; cached | SUM, AVG, MAX, COUNT |
| IdentityColumn | Identity-specific metadata: which identity tags apply to this row, which profiles it appears on | Vec\<IdentityTag\> from ProfilePartition store | COUNT, COUNT\_BY\_TAG |

## **8.2  Formula Engine — ColumnComputer**

The ColumnComputer evaluates FormulaColumn definitions at query time. Formulas are defined in a type-safe expression language supporting the following operations:

| Function Category | Functions | Example |
| :---- | :---- | :---- |
| Arithmetic | \+  −  ×  ÷  %  ^  ABS  ROUND  CEIL  FLOOR | profit \= revenue \- expenses |
| Comparison | \=  \!=  \>  \<  \>=  \<= | budget\_utilization\_pct \> 80 |
| Logical | AND  OR  NOT  IF(cond, true\_val, false\_val)  SWITCH  CASE | IF(health\_score \< 60, "At Risk", "On Track") |
| String | CONCAT  UPPER  LOWER  LEN  CONTAINS  STARTSWITH  TRIM  LEFT  RIGHT  MID | CONCAT(name, " \- ", status) |
| Date | TODAY  DAYS\_BETWEEN  DATE\_ADD  QUARTER  YEAR  MONTH  DAY  WEEKNUM | DAYS\_BETWEEN(start\_date, due\_date) |
| Aggregation | SUM  AVG  COUNT  MAX  MIN  MEDIAN (over related children) | SUM(CHILDREN.budget\_spent) |
| Lookup | RELATED(relation\_col, target\_col)  LOOKUP(key, table, col) | RELATED(program\_ref, budget\_allocated) |
| Engine | HEALTH\_SCORE()  RISK\_SCORE()  MATCH\_SCORE()  INCOME\_PROJECTION(days) | HEALTH\_SCORE() \>= 80 |
| Identity | IDENTITY\_TAG()  IS\_IN\_PARTITION(tag)  VISIBLE\_TO(observer\_type) | IS\_IN\_PARTITION("@jordan-dev") |
| Link Network | LINK\_COUNT()  INBOUND\_LINKS()  OUTBOUND\_LINKS()  IS\_LINKED\_TO(portfolio\_id) | LINK\_COUNT() \> 5 |

## **8.3  Custom Columns**

| Feature | Description |
| :---- | :---- |
| Add Column | Any user with Editor permission on the sheet can add a custom column of any supported type |
| Column Scope | Custom columns can be scoped to a single sheet (local) or promoted to all sheets (global) by an owner |
| Default Values | Custom columns support default values, required flags, and validation rules (min/max, regex, enum set, conditional default) |
| Computed Custom Column | A FormulaColumn built from other columns using the formula editor (arithmetic, conditional, lookup, engine functions) |
| Org Templates | Organizations can define standard column sets as ColumnTemplates, applied to all member portfolios via org governance |
| Identity-Scoped Columns | Custom columns can be scoped to a specific identity partition — visible only when viewing that identity's profile |
| API Access | Custom column values accessible via kogi-dev API under component.data.properties\["column\_key"\] |
| Export | Custom columns appear in all export formats (CSV, XLSX, JSON) with user-defined column headers |
| Shadow Column Mirroring | Custom columns on a source component can be mirrored as MirrorColumns in a linked user's ShadowRow, subject to visibility policy |

# **9\.  Computation Engine (KPCM)**

| *The Computation Engine powers all ComputedColumns, AIColumns, and rollup aggregations in the spreadsheet. It runs on two tiers: synchronous in-request computation for simple derived columns (arithmetic, rollup), and asynchronous kogi-engine computation for complex AI-driven signals (health score, risk score, match score, predictions). All computed results are cached in the CellStore with a staleness TTL and invalidated on relevant EventLog events.* |
| :---- |

## **9.1  All Computational Models**

| Model | Inputs | Output | Consumer Sheets |
| :---- | :---- | :---- | :---- |
| PortfolioHealth | child component statuses, budget utilization, risk flags, progress %, active owner count, engagement, collaboration\_score, benefit coverage | health\_score 0-100 \+ dimension breakdown | SHT-001, SHT-002, SHT-023 |
| ProjectMetrics | sprint velocity history, backlog size, completion rate, cycle time, lead time, blocker count | completion\_pct, velocity, risk\_level, predicted\_completion\_date | SHT-004, SHT-023 |
| ProgramAlignment | child project health scores, KPI actuals vs targets, budget rollup, milestone status | alignment\_score 0-100, KPI\_achievement\_rate | SHT-003, SHT-002 |
| SubPortfolioRollup | all descendant component financial and status data via recursive tree traversal | rollup\_score 0-100, aggregated financials | SHT-002, SHT-023 |
| ResourceUtilisation | resource allocation records, capacity units, consumed units, schedule overlap | utilisation\_pct, demand\_ratio, over\_committed bool | SHT-006, SHT-010 |
| AssetValue | asset type, acquisition cost, valuation history, market data (if available), depreciation schedule | current\_value, depreciation, roi\_pct, irr | SHT-007, SHT-009, SHT-027 |
| ArtifactMaturity | artifact type, version count, review status, linked project completion, documentation completeness | maturity\_score 0-100, completeness breakdown | SHT-008, SHT-012 |
| BinderCoverage | expected item IDs (from template), actual item IDs present | coverage\_pct, missing\_count, extra\_count | SHT-001 (Binder rows) |
| BookConsistency | section count, empty sections, schema compliance count per book type | consistency\_score 0-100 | SHT-001 (Book rows) |
| FolderOrganisation | max depth, orphan count, duplicate names, naming convention compliance | organisation\_score 0-100 | SHT-001 (Folder rows) |
| RecordIntegrity | entry count, duplicate count, integrity hash presence, required fields | integrity\_score 0-100 | SHT-001 (Record rows) |
| CollaborationScore | contributor count, contribution velocity, governance participation, merge conflict rate, attribution weight distribution | collaboration\_score 0-100 | SHT-020, SHT-021, SHT-023 |
| BenefitsHealth | benefit account balances, coverage types present vs required, income replacement ratio, vesting progress | benefits\_health\_score 0-100, coverage\_gap list | SHT-015, SHT-023 |
| RiskScore | risk\_flags (severity \+ probability), mitigation status, overdue items, blocker chain depth | risk\_score 0-100, top\_risk\_items | SHT-022, SHT-023, all sheets |
| IncomeProjection | gig earnings history (90d), active contract revenue, investment distributions, benefit contributions, seasonal patterns | projected\_income\_{30,60,90}d with confidence intervals | SHT-009, SHT-023 |
| CampaignHealth | contribution velocity, time-to-deadline, backers-per-day, social graph engagement with campaign | campaign\_health\_score, success\_probability, days\_to\_goal | SHT-017, SHT-016 |
| GrantMatchScore | eligibility criteria match, portfolio strength relative to grant requirements, historical award data for similar portfolios | grant\_match\_score 0-100, eligibility\_flags | SHT-016 |
| LinkNetworkStrength | link\_count, inbound\_link\_count, link\_diversity (edge types), mutual connection count, network centrality | network\_strength\_score 0-100, centrality\_rank | SHT-031, SHT-025 |
| IdentityCoverageScore | completeness of each ProfilePartition: required fields, visibility mask set, public content present | identity\_coverage\_score per partition | SHT-032 |
| MarketabilityScore | public asset listing quality (title, description, media, reviews), pricing competitiveness vs PriceEngine, CTR/conversion | marketability\_score, listing\_optimization\_hints | SHT-026 |

## **9.2  PortfolioHealth Scoring — Full Algorithm**

The PortfolioHealth model is the master analytical output of the Computation Engine for Portfolio and SubPortfolio rows. It computes an overall health\_score (0-100) from a weighted composition of seven sub-dimensions.

| Dimension | Weight | Inputs | Scoring Logic |
| :---- | :---- | :---- | :---- |
| Delivery Health | 25% | progress\_pct, schedule\_variance\_days, milestone completion rate, predicted\_completion vs due\_date | Full score if on-track; decreasing score for schedule variance; zero for abandoned milestones |
| Financial Health | 20% | budget\_utilization\_pct, cash\_flow signal, ROI trend, overrun\_flags, outstanding\_invoices | Full score if utilization \< 80%; decreasing for overruns; penalty for unpaid overdue invoices |
| Risk Health | 20% | risk\_score, open\_risk\_count, critical\_risk\_count, mitigation\_completion\_rate | Inverse of risk\_score; full if no open critical risks; rapid decay with unmitigated high-severity risks |
| Resource Health | 15% | resource utilization\_pct, over\_allocation\_flags, toolbox coverage, skill gap flags | Full score if utilization 60-80%; penalty for over-allocation (\>100%) and under-allocation (\<30%) |
| Engagement Health | 10% | engagement\_rate, follower\_growth, active\_contributor\_count, collaboration\_score, link\_count growth | Measures portfolio visibility, community engagement, and network growth; relevant for public/shared portfolios |
| Governance Health | 5% | open\_approval\_requests, compliance\_flags, overdue\_reviews, charter\_present, policy\_ids\_attached | Full score if all approvals resolved; decreasing for unresolved compliance flags |
| Benefit Coverage Health | 5% | benefits\_health\_score, coverage\_gap\_flags, pto\_remaining\_days, income\_replacement\_ratio | Relevant for worker's personal portfolio; scores adequacy of portable benefits coverage |

## **9.3  Aggregation Rules**

| Column Type | Aggregations | Notes |
| :---- | :---- | :---- |
| NumberField / CurrencyField / PercentField | SUM · AVG · MIN · MAX · MEDIAN · COUNT · COUNT\_NON\_EMPTY | SUM is default for financial columns; AVG for percentage/score columns |
| DateField / DateTimeField | MIN (earliest) · MAX (latest) · RANGE (latest \- earliest) | Useful for "earliest deadline in group" or "project span" |
| EnumField | COUNT\_BY\_VALUE (distribution) · MODE (most common) | Rendered as mini bar chart in group row |
| TagField | UNION (all tags) · INTERSECTION (shared tags) · COUNT\_BY\_TAG | Displayed as tag cloud in group header |
| RelationField / MultiRelationField | COUNT · COUNT\_UNIQUE | Count of related rows |
| UserField / MultiUserField | COUNT\_UNIQUE · LIST | Count of distinct users; list as avatar cluster |
| BoolField | COUNT\_TRUE · COUNT\_FALSE · PCT\_TRUE | E.g. "7 / 12 tasks complete (58%)" |
| ComputedColumn / AIColumn | AVG · MIN · MAX · MEDIAN | Aggregate the computed signal across group members |
| LinkNetworkColumn | SUM · AVG · MAX | Total links, average link depth, most-connected component |
| IdentityColumn | COUNT · COUNT\_BY\_TAG | How many rows per identity partition |

# **10\.  Collaborative Editing (KPCL)**

## **10.1  CRDT Architecture**

The Portfolio Spreadsheet is a live, multi-user, collaboratively editable document. Multiple users — across multiple devices, time zones, and federation nodes — can simultaneously view and edit the same portfolio. Consistency is maintained by the CRDT engine (LWW field writes \+ OR-Set for set-valued columns), and every mutation is appended to the immutable EventLog.

For the multi-identity case, CRDT operations are tagged with both the node\_id (physical source) and the identity\_tag (logical source), enabling conflict resolution to respect identity-partition boundaries. An edit to a row tagged to Identity A by Identity B is treated as a cross-partition edit and may require approval depending on the partition's write policy.

## **10.2  CRDT Operation Types**

| CRDT Operation | Semantics | Identity Awareness |
| :---- | :---- | :---- |
| SetField (LWW) | Last-Write-Wins field update. Write a scalar or enum column value with a VectorClock timestamp. Higher timestamp wins concurrent writes. | actor\_identity\_tag included in operation. Cross-partition writes checked against SplitPolicy. |
| AddToSet (OR-Set) | Add a value to a set-valued column (tags, owners, policy\_ids, toolbox\_ids, identity\_tags) with a unique tag. Concurrent adds both survive. | Set additions for identity\_tags require write permission on the target partition. |
| RemoveFromSet (OR-Set) | Remove a specific tagged entry from a set-valued column. Only the explicitly tagged entry is removed; concurrent adds survive. | Identity tag removals are audit-logged separately as PartitionTagEvent. |
| AppendLog | Append an EventLog entry. Commutative — all concurrent appends survive; ordered by VectorClock causal timestamp. | EventLog entries carry identity\_tag for attribution. Cross-partition entries clearly marked. |
| AttachChild / DetachChild | Add or remove a GraphEdge::Hierarchy edge. Concurrent attaches both survive; concurrent detach resolves to higher-timestamp operation. | Cross-partition hierarchy edges require owner approval if target partition is isolated. |
| CreateShadowRow | Specialized operation for KLNK: create a ShadowRow in target spreadsheet when a LinkEdge is established. | ShadowRows are always associated with a specific identity's partition. Visibility governed by source's VisibilityMask. |
| SyncMirrorColumn | Update a MirrorColumn value in a ShadowRow from the source component. Idempotent; last-sync-wins semantics. | Source identity must still have an active LinkEdge to target. Revoked links block further syncs. |
| StatusTransition | Status column mutations follow the lifecycle state machine. Invalid transitions rejected by PolicyEngine regardless of VectorClock. | Status transitions on shared components may require governance approval depending on GovernanceModel. |

## **10.3  Conflict Resolution**

| Conflict Type | Resolution Strategy |
| :---- | :---- |
| Concurrent scalar field writes (LWW) | Higher VectorClock timestamp wins. Losing write preserved in EventLog as ConflictRecord for human review. Oba notifies affected users. |
| Concurrent set additions | Both values survive (OR-Set). No conflict — both are correct. No notification needed. |
| Concurrent add and remove of same set value | Remove wins if remove's VectorClock dominates; otherwise add wins. ConflictFlag displayed on row. |
| Concurrent status transitions to incompatible states | Both rejected; component status reverts to last confirmed state. MergeConflict event appended; users notified via KNTF alert. |
| Concurrent cross-partition writes | Both writes preserved in EventLog. Write that violates SplitPolicy is marked as pending\_approval. Owner or admin resolves via governance workflow. |
| ShadowRow sync failure | ShadowRow marked as shadow\_status \= Degraded. Last successful sync data preserved. Retry with exponential backoff. User notified after 3 consecutive failures. |
| CRDT sync failure across federated nodes | Operations buffered in CrdtLog until connectivity restored. Sync replays buffered operations in causal order on reconnect. Conflicts surfaced as review items. |
| Concurrent contributions to shared portfolio | Both contributions submitted to steward review queue. Steward merges manually or approves both independently. CollaborationEngine recomputes attribution weights. |

## **10.4  Contribution Attribution in Collaborative Sheets**

For shared portfolio components and crowdresourced sheets, every row mutation generates a ContributionRecord attached to the component's EventLog. The CollaborationEngine continuously recomputes attribution weights based on contribution history.

| Attribution Formula | Description | Default Weight |
| :---- | :---- | :---- |
| Labor-based | weight \= hours\_contributed / total\_hours\_contributed\_by\_all | 60% of composite weight |
| Equity-based | weight \= equity\_share\_pct / 100 | 30% of composite weight |
| Governance Participation | weight \= votes\_cast / total\_votes\_in\_period | 10% of composite weight |
| Hybrid (Cooperative Default) | weight \= 0.60 x labor\_weight \+ 0.30 x equity\_weight \+ 0.10 x governance\_weight | Default for cooperative entities |
| Custom Formula | Organization defines formula via GovernanceProposal; CollaborationEngine evaluates against ContributionRecords | Configured per org via PolicyEngine |

## **10.5  Real-Time Collaboration Protocol**

Real-time collaborative editing is powered by WebSocket connections managed by the KPCL service. The protocol ensures that all users viewing the same sheet see the same state within 150ms of any mutation.

7. User opens a sheet: client connects to KPCL service via WebSocket. Server sends current sheet snapshot \+ VectorClock state.

8. User edits a cell: client generates a CrdtOperation locally and applies it optimistically. Operation sent to server over WebSocket.

9. Server receives operation: validates permissions, applies to CrdtLog, broadcasts to all other connected clients on same sheet.

10. Remote clients receive broadcast: apply the operation to their local state. Cursor indicators show other users' active cells.

11. Conflict detected: if a concurrent operation creates a conflict, server generates a MergeConflict record and broadcasts a conflict notification to affected clients.

12. Conflict resolution: user or Oba resolves the conflict. Resolution is a new CrdtOperation that overwrites both conflicting values.

13. Persistence: CrdtLog is flushed to PostgreSQL EventLog asynchronously. Redis CrdtBuffer holds in-flight operations.

# **11\.  kogi-engine Integrations**

| *The Portfolio Spreadsheet is both a data source and a display target for the full kogi-engine intelligence stack. Every engine contributes computed columns, derived rows, anomaly flags, and Oba recommendations to the spreadsheet. The integration protocol is bidirectional: the spreadsheet emits EventLog events to the engine; the engine writes computed values back via the plugin writeback protocol.* |
| :---- |

## **11.1  Engine Contribution Registry**

| Engine | Columns Contributed | Writeback Protocol | Update Trigger |
| :---- | :---- | :---- | :---- |
| AnalyticsEngine | views, clicks, ctr, engagement\_rate, followers, spread, share\_count, sentiment\_score | Plugin writeback: on\_component\_updated() | On every engagement event; batched for high-frequency events |
| TelemetryEngine | flow\_stats, event\_count, last\_event\_timestamp, mutation\_velocity | Plugin writeback: on\_event() | Real-time streaming; Kafka consumer |
| RecommendationEngine | recommendation\_score, similar\_items, suggested\_actions | Daily batch; on-demand for active sessions | Session start \+ nightly rebuild |
| PersonalizationEngine | personalized\_rank, relevance\_to\_active\_persona, feed\_visibility\_score | Session-scoped; per-user cache invalidated on persona update | Persona change; session start |
| MatchEngine | match\_score (opportunity queries), talent\_match\_score (resource rows), investor\_match\_score (campaign rows) | On-demand via gRPC; cached for 5 min | New opportunity posted; user profile update |
| GraphEngine | dependency\_depth, critical\_path\_flag, network\_centrality, cluster\_membership | Async after edge add/remove | GraphEdge mutation events |
| RiskEngine | risk\_score, risk\_breakdown, mitigation\_gap\_flags, at\_risk\_deadline\_count | Plugin writeback: on\_component\_updated() | EventLog events: status change, budget overrun, due date approach |
| OptimizationEngine | resource\_optimization\_hints, budget\_reallocation\_suggestions, schedule\_optimization\_hints, predicted\_completion\_date | Daily batch; on-demand for active projects | Sprint completion; milestone approach |
| SearchEngine | search\_rank, full\_text\_index\_status, keyword\_suggestions | Plugin hook: on\_component\_created() \+ on\_component\_updated() | Component create/update |
| AllocationEngine | capacity\_conflicts, allocation\_recommendations, utilization\_heatmap | Async after allocation change | ResourceAllocation create/update events |
| IncentiveEngine | kogipoints\_earned, incentive\_streak, contribution\_badges | Event-driven: on game event completion | Transaction completion; badge criteria met |
| BenefitsEngine | benefits\_health\_score, coverage\_gap\_flags, tax\_savings\_estimate, contribution\_forecast | Weekly batch \+ on income event | New gig/contract income; quarterly review |
| GrantEngine | grant\_match\_score, eligibility\_flags, recommended\_grants | Weekly batch | Portfolio composition change; new grants in registry |
| CrowdfundingEngine | campaign\_health\_score, success\_probability, investor\_match\_quality | Real-time for active campaigns; daily for inactive | Campaign backing event; velocity check |
| CollaborationEngine | collaboration\_score, contributor\_diversity\_index, merge\_conflict\_count | After contribution events; daily rollup | Contribution accepted/rejected; merge conflict resolved |
| BookingEngine | availability\_conflicts, booking\_revenue\_forecast, utilization\_calendar | On booking event; daily calendar rebuild | New booking; cancellation; availability change |
| CRMEngine | lead\_score, pipeline\_value, follow\_up\_due\_flags | Daily batch; real-time on inquiry | New inquiry; stage transition; activity logging |
| LogisticsEngine | logistics\_status, itinerary\_health, resource\_routing\_conflicts | On logistics event | Booking confirmed; equipment assigned; route changed |
| LinkNetworkEngine (KLNK) | link\_count, inbound\_link\_count, network\_centrality, link\_depth, linked\_portfolio\_ids | On LinkEdge create/delete | KLNK graph mutation events |
| IncomeProjection model | income\_projection, earnings\_trend, platform\_diversification\_score | Weekly batch | New gig/contract completion; quarterly earnings milestone |

## **11.2  Plugin Writeback Protocol**

The plugin writeback protocol is the mechanism by which kogi-engine sub-systems write computed values back into portfolio rows without bypassing the PermissionTier checks or EventLog requirements. All engine writebacks are treated as first-class EventLog mutations.

| Protocol Step | Description |
| :---- | :---- |
| 1\. Engine computes value | The kogi-engine sub-system computes a new value for one or more AIColumns on one or more PortfolioComponents. This may be the result of a batch computation, a real-time signal, or an on-demand query. |
| 2\. Engine calls WritebackService | The engine calls the WritebackService gRPC endpoint with: component\_id, column\_id, new\_value, confidence (0-1), source\_engine, computation\_timestamp. |
| 3\. Permission check | WritebackService validates that the writing engine has the AIWriter permission for the target column. This permission is granted at platform level and cannot be overridden by users. |
| 4\. PolicyEngine evaluation | WritebackService evaluates the PolicyEngine for any policies that might block or modify the writeback (e.g. a policy that requires human review before risk\_score changes above a threshold). |
| 5\. EventLog append | A PortfolioEventKind::AIColumnUpdated event is appended to the component's EventLog with the full writeback details including source engine and computation\_timestamp. |
| 6\. CellStore update | The new column value is written to the CellStore cache with a TTL determined by the column's update\_frequency setting. |
| 7\. WebSocket broadcast | All clients currently viewing the affected sheet receive a real-time CellUpdateEvent for the changed columns. |
| 8\. KNTF notification | If the writeback changes a column value that crosses an alert threshold (e.g. health\_score drops below 60, risk\_score rises above 80), KNTF dispatches the appropriate notification. |

# **12\.  Persistence & Storage Architecture**

## **12.1  Storage Layer Design**

| Store | Technology | What Is Stored | Access Pattern |
| :---- | :---- | :---- | :---- |
| Primary Component Store | PostgreSQL \+ JSONB | PortfolioRow canonical records — ComponentMetadata \+ ComponentData serialized as JSONB with typed indexed columns for hot query fields (status, owner, type, created\_at, health\_score, visibility) | Read-heavy; write on mutation; partitioned by owner\_entity\_id |
| Graph Store | PostgreSQL (adjacency table) | GraphEdge table: source\_id · target\_id · edge\_type · metadata JSONB · created\_at. Indexed on source, target, and type. Includes InterPortfolioLink edges for KLNK. | Read: O(1) by source or target; Write: on edge create/delete |
| EventLog Store | PostgreSQL (append-only, partitioned by created\_at) | Immutable event records: component\_id · actor\_id · identity\_tag · event\_type · delta JSONB · vector\_clock · timestamp. Partitioned by month. | Append-only writes; range reads for audit; never updated or deleted |
| CRDT Buffer | Redis (hash by component\_id) | CrdtLog operations awaiting apply; vector clock cache; hot-path LWW cell cache for sub-millisecond concurrent writes | High-frequency read/write; TTL-based expiry; flushed to PostgreSQL periodically |
| Cell Cache | Redis (hash by component\_id:column\_id) | Computed column values cached with TTL. Prevents re-computation on every read for expensive AI columns. | Read: O(1) by key; Write: on engine writeback or cell mutation; invalidated by TTL or explicit event |
| Link Network Store | PostgreSQL (LinkEdge table) \+ Redis (hot path) | InterPortfolioLink edges, ShadowRow metadata, MirrorColumn configurations, LinkForest snapshots | Read: graph traversal queries; Write: on link create/delete; forest snapshots materialized hourly |
| Identity & Profile Store | PostgreSQL (ProfilePartition table) | ProfilePartition records, VisibilityMask configurations, account-to-partition mappings, identity anchors | Read on every authenticated request; Write on identity management actions |
| Search Index | Meilisearch / Typesense | Full-text index over component name, tags, bio, description, custom fields, identity handles, linked portfolio names | Read: search queries; Write: on component create/update via plugin hook; rebuilt daily |
| Snapshot Store | S3-compatible object store | Full PortfolioSystem snapshots serialized as compressed JSON. Keyed by snapshot\_id; lifecycle-managed with configurable retention. | Write: scheduled \+ manual; Read: on restore; lifecycle: 90-day default retention |
| Analytics Store | ClickHouse (columnar) | High-throughput append of analytics events: views, clicks, engagements, shares, link traversals. Queried for aggregation by AnalyticsEngine. | Append-only writes; aggregate reads; never updated after write |
| Engine Feature Store | Redis \+ PostgreSQL (materialized views) | Pre-computed engine signals (health\_score, risk\_score, match\_score, link\_strength) stored as materialized views, refreshed on schedule and on trigger | Read: per-component on sheet load; Write: engine writeback via WritebackService |
| Shadow Row Store | PostgreSQL (shadow\_rows table) | ShadowRow records with source\_component\_id, visible\_columns, sync status, last\_synced timestamps. Cross-user; indexed by host entity and source entity. | Read: on sheet load for host user; Write: on link creation \+ sync events |

## **12.2  Indexing Strategy**

| Index | Type | Columns Indexed | Purpose |
| :---- | :---- | :---- | :---- |
| Primary Key Index | B-Tree (PostgreSQL) | component\_id | O(1) component lookup |
| Owner Index | B-Tree | owner\_entity\_id \+ status | Fast "all my components" queries |
| Type Index | B-Tree | item\_type \+ status \+ visibility | Sheet filter queries by type |
| Status Index | B-Tree | status \+ updated\_at | Recent active components; lifecycle queries |
| Visibility Index | B-Tree | visibility \+ item\_type | Public component discovery |
| Financial Indexes | B-Tree | budget\_utilization\_pct \+ status; payment\_status \+ due\_date | Finance sheet queries; overdue invoice detection |
| Score Indexes | B-Tree | health\_score DESC; risk\_score DESC | Analytics sheet queries; health/risk rankings |
| Timestamp Indexes | B-Tree | created\_at \+ updated\_at \+ archived\_at | Timeline queries; recent updates feed |
| Tag Index | GIN (PostgreSQL) | tags, hashtags, labels | Tag-based filtering and discovery |
| Full-Text Index | Meilisearch | name, description, custom fields, identity\_handle | Universal search across all components |
| Hierarchy Edge Index | B-Tree (GraphEdge table) | source\_id \+ edge\_type; target\_id \+ edge\_type | O(1) parent/child lookups; tree traversal |
| KLNK Link Index | B-Tree (GraphEdge table) | edge\_type=InterPortfolioLink; from\_entity \+ to\_entity | Link network traversal; mutual connection lookup |
| Identity Tag Index | GIN | identity\_tags | Partition-scoped queries; profile view generation |
| ShadowRow Index | B-Tree | host\_entity\_id \+ source\_component\_id | Shadow row lookup by host \+ source |
| EventLog Partition Index | B-Tree (partitioned by month) | component\_id \+ created\_at | Audit trail queries; event replay |

## **12.3  Snapshotting & Recovery**

| Operation | Trigger | Storage | Recovery |
| :---- | :---- | :---- | :---- |
| Auto Snapshot | Daily at 02:00 UTC per active portfolio | S3: {entity\_id}/{date}.snap.json.gz | restore\_snapshot() replays all ComponentData; post-restore EventLog preserved |
| Manual Snapshot | User or Oba triggers via UI or API | S3: {entity\_id}/{user\_id}/{timestamp}.snap.json.gz | Same as auto snapshot |
| Checkpoint | Before any bulk operation (bulk status change, import, federation sync, identity merge) | Redis (temporary) \+ S3 (permanent after confirmation) | Checkpoint restore is instant for recent (Redis) or slower for S3 |
| Identity Merge Checkpoint | Before any CrossIdentityMerge operation | S3 with merge\_checkpoint tag | Special restore path that reverses identity tag assignments |
| Link Network Snapshot | Hourly LinkForest snapshot per entity | PostgreSQL materialized view \+ S3 daily backup | Used for link graph analytics and restore of disconnected ShadowRows |
| Federation Sync Checkpoint | Before each CRDT sync with a federation peer | CrdtLog buffer in Redis | Replay buffered operations from last successful sync point on reconnect |

# **13\.  Security, Access Control & Compliance**

## **13.1  Permission Tier Hierarchy**

| Tier | Ordinal | Granted Capabilities | Sheet Access |
| :---- | :---- | :---- | :---- |
| Public (Unauthenticated) | 0 | Read-only access to rows where visibility \= Public. No mutations permitted. | SHT-024 (Feed), SHT-026 (Marketplace) — public rows only |
| Viewer | 1 | Read-only access to all rows the caller is permitted to see. No mutations. | All sheets — permitted rows only; computed columns visible |
| Subscriber | 2 | Read access \+ bookmark, save, follow, subscribe actions. No content mutations. | All permitted sheets \+ SHT-025 (Contacts) subscriber view |
| Contributor | 3 | Comment, react, poll, join, submit contributions. Cannot edit parent resource directly. | All permitted sheets \+ contribute actions on SHT-020/SHT-021 |
| Editor | 4 | Edit content, bump version, create child items, modify columns. Cannot delete or transfer ownership. | All sheets with edit capability; can add custom columns |
| Manager | 5 | Manage settings and members, configure views, govern access, configure board modes. | All sheets \+ sheet management; can publish views as templates |
| Owner | 6 | Full ownership privileges: all actions, all columns, all sheets, delete, transfer, identity configuration. | All sheets including SHT-029 (Event Log), SHT-030 (Snapshots), SHT-032 (Identity) |
| Admin | 7 | Super-admin / platform-level override. Can access any component subject to legal holds. | All sheets; audit access to all EventLogs; compliance reporting |

## **13.2  Row-Level Security (RLS)**

Every query against the PortfolioRow store is automatically scoped by PostgreSQL Row-Level Security policies. The RLS layer evaluates the caller's identity and permission tier against the component's visibility field and ACL before returning any data.

| Visibility Level | Who Can Read | Who Can Write | KLNK Link Visibility |
| :---- | :---- | :---- | :---- |
| Private | Owner \+ explicitly invited users with Viewer+ tier | Owner \+ Editors with explicit grant | Not discoverable via KLNK. Links still possible if directly initiated by owner. |
| Protected | Owner \+ approved followers \+ explicit invitees | Owner \+ Editors | Discoverable to approved followers; links shown to connections |
| Internal | All members of the owning organization | Owner \+ Editors \+ org Managers | Visible to org members in KLNK org tree; not in public graph |
| Public | Anyone including unauthenticated visitors | Owner \+ Editors | Fully discoverable in KLNK; appears in public graph traversals |
| Unlisted | Anyone with the direct link/URL; not indexed | Owner \+ Editors | Not indexed in KLNK; direct links still work |
| DraftOnly | Owner only (even co-editors cannot see during draft) | Owner only | Not discoverable; no KLNK visibility |
| Custom (ACL) | Users matching explicit ACL rules defined by owner | Users with Editor role in ACL | Discoverable to ACL members; custom LinkVisibility per edge type |

## **13.3  Column-Level Visibility**

Individual columns can have their own visibility level, independent of the row's visibility. This is particularly important for sensitive columns (financial details, contact info, health data) that should be visible to the row owner but not to followers or the public.

| Column Sensitivity Tier | Default Columns | Visible To |
| :---- | :---- | :---- |
| Public | name, slug, status, tags, description, views, followers, component\_type, item\_type | Everyone who can see the row (based on row visibility) |
| Follower | progress\_pct, health\_score, created\_at, updated\_at, lifecycle\_stage, skill tags | Followers and above |
| Connection | owners (names), budget\_allocated (if public project), milestone dates | Trusted connections and above |
| Contributor | detailed analytics, team member list, detailed timeline, work items | Contributors and above |
| Editor | budget\_spent, financial details, invoice\_refs, risk\_flags, compliance\_flags | Editors and above |
| Owner-only | bank\_account\_ref, tax\_category, personal financial details, benefit balances, identity partition config, VisibilityMask rules | Owner only (even when row is Public) |
| AI-Internal | raw engine signal inputs, algorithm parameters, model confidence intervals | System only; never exposed to user queries |

## **13.4  Multi-Identity Privacy & Compliance**

| Privacy Feature | Description |
| :---- | :---- |
| Identity Isolation | Rows tagged exclusively to an Isolated identity are never accessible through other identity contexts, even when authenticated as the same Sovereign Entity. Full hardware-boundary equivalent at the data layer. |
| Cross-Identity Audit Trail | Any access to cross-partition data by a secondary identity is logged as a CrossPartitionAccessEvent in the immutable EventLog. This provides a full audit trail for identity separation. |
| PII Classification | Legal name, phone numbers, private emails, financial account details, health data, and government IDs are classified as PII. Stored with field-level encryption (AES-256). Never returned in bulk exports without explicit consent per field. |
| GDPR Right to Erasure | Supports GDPR Article 17: user can request erasure of personal data from all sheets and KLNK link records. EventLog entries are anonymized (actor\_id replaced with anonymous\_actor\_id). Cryptographic audit chain maintained without PII. |
| CCPA Data Portability | Full portfolio data export in JSON-LD format including all sheets, event logs, analytics, contributions, and financial summaries. Delivered as encrypted, signed archive. |
| Identity Pseudonymity | Any identity can be configured as pseudonymous: the @handle is not linked to legal identity in any publicly accessible record. Sovereign Entity mapping stored only in encrypted KPID vault. |
| Data Minimization | Only data required for the user's configured use cases is collected. Unused columns are not instantiated. Engine signals are only computed for columns the user has enabled in their sheet configuration. |
| Consent Management | Granular consent controls per data type, engine integration, and KLNK link type. Users can revoke consent for specific data uses at any time. Revocation propagates to linked engine subscriptions. |

# **14\.  API Surface (KPMS · REST \+ gRPC)**

## **14.1  Portfolio REST API — Core Endpoints**

| Endpoint | Method | Description | Auth Required |
| :---- | :---- | :---- | :---- |
| /portfolio/:id | GET | Read a single PortfolioComponent by ID (visibility-gated by caller's tier) | Viewer+ |
| /portfolio/@:handle/:slug | GET | Read component by identity handle and slug | Viewer+ |
| /portfolio | POST | Create a new component (item or container) | Contributor+ |
| /portfolio/:id | PATCH | Update component fields (owner/editor only) | Editor+ |
| /portfolio/:id | DELETE | Soft-archive component | Owner |
| /portfolio/:id/children | GET | List direct children in hierarchy | Viewer+ |
| /portfolio/:id/descendants | GET | List all transitive descendants with pagination | Viewer+ |
| /portfolio/:id/attach/:child\_id | POST | Attach a child component (creates Hierarchy edge) | Editor+ |
| /portfolio/:id/attach/:child\_id | DELETE | Detach a child component | Editor+ |
| /portfolio/:id/actions | POST | Perform an ActionKind: like, share, follow, invest, donate, post, tag | Contributor+ |
| /portfolio/:id/analytics | GET | Get computed analytics for a component | Viewer+ |
| /portfolio/:id/health | GET | Get PortfolioHealth model output with dimension breakdown | Viewer+ |
| /portfolio/:id/eventlog | GET | Get EventLog for a component (paginated) | Editor+ |
| /portfolio/:id/allocations | GET | Get resource allocations for a component | Editor+ |
| /portfolio/:id/allocate | POST | Create a resource or budget allocation | Manager+ |
| /portfolio/:id/record-spend | POST | Record budget spend against a component | Editor+ |
| /portfolio/search | GET | Full-text \+ faceted search across the registry | Viewer+ |
| /portfolio/query | GET/POST | PQL (Portfolio Query Language) query endpoint | Editor+ |
| /portfolio/:id/contributions | GET | List ContributionRecords for a shared component | Contributor+ |
| /portfolio/:id/contribute | POST | Submit a contribution to a shared component | Contributor+ |
| /portfolio/:id/snapshot | POST | Create a named snapshot | Owner |
| /portfolio/restore/:snapshot\_id | POST | Restore from a snapshot (requires confirmation) | Owner |
| /portfolio/export/:sheet\_id | GET | Export sheet as CSV, XLSX, or JSON (content-type negotiation) | Editor+ |
| /portfolio/:id/crdt/sync | POST | Submit a CrdtLog batch for federation synchronization | System (federation peers only) |

## **14.2  Sheet & View API**

| Endpoint | Method | Description |
| :---- | :---- | :---- |
| /sheets | GET | List all sheets accessible to the authenticated user (with row counts and last\_updated) |
| /sheets/:sheet\_id | GET | Get sheet definition and metadata |
| /sheets/:sheet\_id/rows | GET | Query rows on a sheet with filter, sort, group, pagination (PQL-compatible params) |
| /sheets | POST | Create a custom sheet (SHT-034 type) |
| /sheets/:sheet\_id | PATCH | Update sheet definition (column schema, default sort/group) |
| /views | GET | List saved views for the authenticated user |
| /views | POST | Create a new saved view |
| /views/:view\_id | PATCH | Update a saved view (filters, sorts, groups, highlight rules) |
| /views/:view\_id/rows | GET | Render rows for a specific saved view (applies all view filters/sorts/groups) |
| /views/:view\_id/share | POST | Share a view with specific users or make it public template |
| /columns | GET | List column schema for a sheet (including custom columns) |
| /columns | POST | Add a custom column to a sheet |
| /columns/:col\_id | PATCH | Update a custom column definition |
| /columns/:col\_id | DELETE | Remove a custom column (with data preservation option) |
| /columns/:col\_id/compute | GET | Get computed value for a specific ComputedColumn on a row |

## **14.3  Link Network API**

| Endpoint | Method | Description |
| :---- | :---- | :---- |
| /klnk/nodes/:id/tree | GET | Get LinkTree rooted at node\_id with depth, edge\_type filter, and direction params |
| /klnk/nodes/:id/forest | GET | Get complete LinkForest for the entity owning this node |
| /klnk/nodes/:id/ego | GET | Get ego network (depth 1+2) centered on node\_id |
| /klnk/edges | POST | Create a new LinkEdge (link request). Triggers consent flow for consent-required edge types. |
| /klnk/edges/:edge\_id | DELETE | Remove a LinkEdge. Triggers ShadowRow disconnection protocol. |
| /klnk/shadow-rows | GET | List all ShadowRows in the caller's spreadsheet |
| /klnk/shadow-rows/:id/sync | POST | Manually trigger a MirrorColumn sync from source |
| /klnk/shadow-rows/:id/columns | GET/PATCH | Get or update the visible\_columns and write\_back\_columns configuration for a ShadowRow |
| /klnk/discover | GET | Discover link targets matching a query, ranked by match\_score and link\_type relevance |
| /klnk/graph/path | GET | Find shortest path between two nodes (Dijkstra with edge weight) |
| /klnk/graph/clusters | GET | Identify community clusters (Louvain). Useful for ecosystem analysis. |
| /klnk/graph/centrality | GET | Compute betweenness, closeness, and eigenvector centrality for a node set |
| /klnk/subscriptions | GET/POST/DELETE | Manage link change subscriptions: webhooks and Kafka events for linked node changes |
| /klnk/sheet | GET | Render the Link Network as a spreadsheet sheet (SHT-031 view) |

## **14.4  Identity & Profile API (KPID)**

| Endpoint | Method | Description |
| :---- | :---- | :---- |
| /identity/partitions | GET | List all ProfilePartitions for the authenticated Sovereign Entity |
| /identity/partitions | POST | Create a new ProfilePartition (new identity/profile) |
| /identity/partitions/:id | GET/PATCH | Read or update a ProfilePartition configuration |
| /identity/partitions/:id/visibility-mask | GET/PUT | Get or replace the VisibilityMask for a partition |
| /identity/partitions/:id/rows | GET | List all rows tagged to this partition |
| /identity/rows/:component\_id/tags | GET/POST/DELETE | Get, add, or remove identity\_tags on a specific row |
| /identity/accounts | GET | List all accounts associated with the Sovereign Entity |
| /identity/accounts/:id/scope | GET/PUT | Get or update the access scope for a specific account |
| /identity/merge | POST | Initiate a CrossIdentityMerge operation (requires checkpoint creation \+ confirmation) |
| /identity/handoff | POST | Transfer an identity partition to a different Sovereign Entity |
| /identity/:handle/public-view | GET | Get the public spreadsheet view for a specific @handle (respects VisibilityMask) |
| /identity/split-policy | GET/PUT | Get or update the SplitPolicy governing cross-partition data access |

## **14.5  Portfolio Query Language (PQL)**

PQL is a structured query interface for the portfolio registry — SQL-inspired but typed to the PortfolioRow schema. Exposed via the /portfolio/query endpoint and the QueryService gRPC method. Supports cross-partition queries for authenticated owners and filtered cross-portfolio queries for link network traversals.

| PQL Feature | Description | Example |
| :---- | :---- | :---- |
| SELECT | Choose which columns to return. Use \* for all visible columns. Computed columns resolved at query time. | SELECT id, name, health\_score, budget\_remaining |
| FROM rows | Always queries the PortfolioRow store. No FROM variants — single table model. | FROM rows |
| WHERE | Filter predicates: all ViewFilter types supported. AND/OR nesting. Subquery conditions. | WHERE item\_type \= "Project" AND health\_score \< 70 |
| ORDER BY | Multi-column sort. ASC/DESC. NULLS FIRST/LAST. | ORDER BY due\_date ASC NULLS LAST, health\_score DESC |
| GROUP BY | Aggregate with SUM/AVG/COUNT/MIN/MAX. Supports rollup expressions. | GROUP BY program\_ref.name, ROLLUP(quarter) |
| LIMIT / OFFSET | Pagination. Max LIMIT 10000 per query for non-admin callers. | LIMIT 50 OFFSET 100 |
| JOIN | Cross-relation join. Join linked components via RelationField columns. | JOIN program\_ref ON program\_ref.id \= program\_id |
| IDENTITY SCOPE | Filter to rows tagged to a specific identity partition. | IDENTITY "@jordan-dev" |
| SHADOW INCLUDE | Include ShadowRows from linked portfolios in results. | INCLUDE SHADOW ROWS FROM "@partner-org" |
| AS\_OF | Time-travel query: returns row state at a specific timestamp using EventLog replay. | AS\_OF "2026-01-01T00:00:00Z" |

# **15\.  Export, Integration & Embedding (KPEX)**

## **15.1  Export Formats**

| Format | API Method | Notes | Identity Scope |
| :---- | :---- | :---- | :---- |
| CSV | GET /portfolio/export/:sheet\_id?format=csv | Standard delimiter-separated export of visible columns. Computed columns included as static values at export time. | Applies VisibilityMask of exporting identity |
| XLSX | GET /portfolio/export/:sheet\_id?format=xlsx | Multi-sheet Excel workbook: main sheet \+ summary pivot \+ chart data. Uses openpyxl pipeline. | Same as CSV |
| JSON | GET /portfolio/export/:sheet\_id?format=json | Raw PortfolioRow JSON array — full schema including all fields, relationships, and computed values. | Owner-level: all columns. Others: visible columns only. |
| JSON-LD | GET /portfolio/export/:sheet\_id?format=jsonld | Linked Data export with schema.org and kogi-vocab type annotations — portable, semantic, machine-readable. | Public-facing only; applies Public visibility mask |
| Markdown | GET /portfolio/export/:sheet\_id?format=md | Markdown document: section headings per group, tables per component type. Suitable for portfolio showcases and README generation. | Configurable — applies selected identity's public mask |
| PDF Report | GET /portfolio/export/:sheet\_id?format=pdf | Rendered portfolio report document — styled with kogi brand template, charts, health summaries, key metrics. | Configurable per export |
| vCard Bundle | GET /portfolio/export/contacts?format=vcf | Exports ContactBook rows as a vCard (.vcf) bundle for import into external contact managers. | Owner only |
| GDPR Archive | POST /portfolio/export/gdpr-archive | Complete data export: all sheets, all partitions, full EventLog, analytics, contributions, financial summaries. Encrypted \+ signed. | Owner only; triggers audit log entry |
| Webhook Push | POST /portfolio/webhooks | Real-time push of EventLog events or row change deltas to a registered external URL on every mutation. | Configurable per webhook; partition-scoped |
| Embed Widget | GET /portfolio/embed/:view\_id | Embeddable iframe/script rendering a read-only view of a public sheet or portfolio in an external webpage. | Public rows only; respects public VisibilityMask |
| Developer API | REST \+ gRPC | Full programmatic access via kogi-dev SDK. Supports streaming for large result sets. | Per API key permissions |

## **15.2  Import Sources**

| Import Source | Method | Column Mapping | Identity Assignment |
| :---- | :---- | :---- | :---- |
| CSV / XLSX Upload | File upload → Oba-assisted column mapping wizard → batch create | Intelligent type inference \+ user confirmation | Prompt user to select which identity partition to assign imported rows |
| Google Sheets | OAuth \+ Sheets API → pull → import wizard | Schema mapping with type coercion | Same as CSV |
| Notion | Notion integration (API key) → database pull → property mapping | Notion property types mapped to PortfolioColumn types | User selects partition |
| Airtable | Airtable API → base pull → field mapping | Airtable field types → PortfolioColumn types | User selects partition |
| Jira | Jira API → issue pull → story type mapping | Issue types → StoryType enum; fields → custom columns | Defaults to primary work identity |
| Asana | Asana API → project pull → task mapping | Project → Program/Project rows; tasks → Task rows | User selects partition |
| GitHub / GitLab Issues | Issues API → pull → type classification | Issue labels → story\_type; milestones → milestones column | Defaults to developer identity if present |
| Quickbooks / Wave | Accounting API → transaction pull → financial mapping | Transactions → Finances sheet rows; categories → tax\_category | Defaults to financial identity |
| Gig Platform Earnings | Platform API (DoorDash, Uber, etc.) or CSV upload | Earnings fields → Gig row columns; tips → tips column | Defaults to gig worker identity partition |
| ContactBook Import | vCard, Google Contacts, Outlook CSV | Contact fields → Profile row columns | User selects partition; defaults to primary professional identity |

# **16\.  Portfolio Template System**

## **16.1  Template Types**

| Template | Pre-configured For | Included Sheets & Views | Identity Config |
| :---- | :---- | :---- | :---- |
| Independent Worker Starter | Solo freelancer / gig worker; first portfolio setup | Master Registry, Work & Gigs, Finances, Portable Benefits, Timeline, Contacts, Link Network | Single primary identity; default professional profile |
| Multi-Identity Freelancer | Worker with multiple clients/roles requiring separate public profiles | All starter sheets \+ Identity & Profiles sheet; pre-configured with 2 ProfilePartitions | 2 identities: professional \+ personal/creative |
| Software Developer Portfolio | Developer with projects, open-source contributions, and client contracts | Projects (Agile board), Tasks & Backlog, Assets (repos), Artifacts (releases), Finances, Contacts, Link Network | Dev identity with GitHub federation link |
| Creative Portfolio | Designer, artist, musician, content creator | Projects, Artifacts (creative works), Marketplace Listings, Finances, Community Feed, Contacts, Link Network | Creative identity \+ pseudonymous public identity option |
| Freelance Consultant | Consultant with multiple client engagements | Work & Gigs (contract-focused), CRM view, Finances, Timeline, Contracts & Deliverables, Contacts | Professional identity; client-facing profile partition |
| Cooperative / Collective | Multi-member cooperative or collective | Shared Portfolios, Group Economics, Collaboration, Governance, Finances (cooperative treasury), Organizations, Link Network | Org identity as primary; member identities linked |
| Startup Founder | Early-stage startup with team, product, investors | Projects, Team (shared), Equity & Crowdfunding, Grants, Roadmap, Finances, Contacts, Risk Register, Link Network | Founder identity \+ company org identity |
| Gig Worker Benefits Optimizer | Gig worker focused on portable benefits and income optimization | Work & Gigs, Portable Benefits, Finances, Grants & Microfinancing, Income Projection view | Single gig worker identity; benefits-focused profile |
| Event Producer / Booking | Event or touring professional with bookings, crew, logistics | Work & Gigs (booking), Timeline (tour calendar), Resources (crew, equipment), Finances, Logistics | Performance identity \+ production identity |
| Organization / DAO | Formal organization with governance, treasury, and member management | Organizations, Shared Portfolios, Group Economics, Governance, Finances (treasury), Member Directory, Link Network | Org entity as root; member user identities linked via OrgMembership edges |
| Research & Academic | Researcher, academic, or knowledge worker | Projects (research), Artifacts (papers, datasets), Grants, Timeline, Contacts, Knowledge Registry | Academic identity with institution federation link |
| Multi-Platform Gig Aggregator | Worker on 5+ gig platforms who needs unified income and benefits tracking | Work & Gigs, Portable Benefits (deep dive), Finances, Income Projection, Platform Dashboard view | One identity per platform with unified root spreadsheet; cross-platform earnings aggregation |

# **17\.  Oba AI Integration — Spreadsheet Intelligence**

| *Oba is the AI chief-of-staff that lives inside the portfolio spreadsheet. It reads every row, every column, and every engine signal — and surfaces proactive, context-aware intelligence directly in the spreadsheet UI. Oba does not just answer questions; it notices what the user should notice, flags what they should act on, and prepares actions for their approval.* |
| :---- |

## **17.1  Oba Capabilities in the Spreadsheet**

| Capability | Description | Trigger |
| :---- | :---- | :---- |
| Row Annotations | Oba attaches inline annotations to rows: "This project is 14 days behind schedule — adjust due date or reduce scope?" · "This grant deadline is in 5 days — draft application now?" | Health/risk threshold crossed; deadline approach |
| Column Completion | Oba suggests values for empty required fields: "Due date is missing — suggest April 30 based on similar projects?" · "No risk flags — want me to analyze and create an initial risk assessment?" | Required field empty; first open after row creation |
| Smart Filters | "Show me everything at risk this week" → Oba translates natural language to ViewFilter: { risk\_score \> 70 OR due\_date \< TODAY+7 AND status \= Active } | Natural language query in search bar |
| Formula Suggestions | When a user adds a custom column, Oba suggests formulas: "You're tracking revenue and expenses — want me to add a profit margin % formula column?" | New column creation of NumberField or CurrencyField type |
| Batch Actions | "Archive all completed projects older than 90 days" → Oba prepares a bulk archive action for user confirmation before executing | Natural language instruction |
| Sheet Generation | "Create a sheet showing all my income sources this quarter grouped by platform" → Oba generates a ViewDefinition and creates the custom sheet | Natural language request for new view |
| Daily Health Briefing | Oba portfolio briefing: top health risks, upcoming deadlines, pending approvals, contribution opportunities, suggested grant matches, network growth alerts | Scheduled: daily morning; on-demand via Oba chat |
| Anomaly Alerts | Oba proactively flags: budget overrun approaching (\>80% utilization), velocity drop in sprint, stalled project (no updates 14 days), coverage gap in benefits, unusual link network pattern | AnomalyEngine detection; threshold crossing; inactivity timer |
| Import Mapping | During CSV/XLSX import, Oba suggests column mappings: "This looks like 'Project Name' — map to PortfolioRow.name?" | Import wizard launch |
| Link Network Suggestions | "Your portfolio is highly visible to investors based on link patterns — consider publishing your Equity & Crowdfunding sheet" · "Alex (3 mutual connections) has skills that match your current project gap" | KLNK link analysis; MatchEngine collaborator suggestion |
| Identity Optimization | "Your @jordan-dev identity hasn't been updated in 30 days — want me to update your public showcase with your recent project completions?" | Profile staleness detection; new completed work not reflected in public view |
| Contribution Matching | For crowdresourced components: "Alex has the skills listed in this project's requirements — send collaboration invite?" | MatchEngine talent match for open contribution requirements |
| Narrative Generation | "Write a portfolio highlight for my top 3 projects this quarter" → Oba generates structured portfolio narrative from row data for resumes, proposals, grant applications | User request; quarterly review reminder |
| Grant Application Drafting | "I found 3 grants you may be eligible for — want me to draft the eligibility checklist and start the application for the best match?" | GrantEngine match score crosses threshold; new grant in registry matches portfolio |
| ShadowRow Health | "The ShadowRow for 'Design Collective Project' hasn't synced in 72 hours — the source may have changed. Review or refresh?" | ShadowRow sync failure; stale threshold crossed |
| PQL Query Builder | Oba translates natural language into PQL queries: "Show me all projects over budget sorted by overrun amount" → full PQL statement | User question in Oba chat that implies a data query |

## **17.2  Oba Identity & Network Workflows**

Oba has specific capabilities tailored to the multi-identity and link network features that are unique to the KMDSS:

| Workflow | Description |
| :---- | :---- |
| Identity Audit | Oba reviews all ProfilePartitions and identifies: rows that should be visible on a profile but aren't tagged, rows that are public but contain sensitive data that should be owner-only, profiles that are incomplete or stale. Generates an action list for owner review. |
| Visibility Review | "Before you publish this project as Public, here are 3 columns that contain owner-only data that will become visible to followers: bank\_account\_ref, tax\_category, and expense\_details. Confirm or adjust visibility." |
| Link Network Introduction | "You and @partner-org have 8 mutual connections and both work in fintech. You've referenced their playbook twice. Would you like me to draft a collaboration inquiry?" |
| ShadowRow Curator | Oba monitors all ShadowRows in the user's spreadsheet. Flags rows that are disconnected, stale, or where the source component's visibility changed. Prepares update or archive actions. |
| Cross-Portfolio Due Diligence | When a user is considering a deal with a linked portfolio: "Here's a summary of @contractor-x's linked portfolio: 12 completed projects (avg health score 84), 3 open projects, response time 1.2 days, 47 network connections including 4 people you know." |
| Federation Health Monitor | For federated portfolios: monitors CRDT sync health across all federation nodes. Alerts on divergence, identifies conflict hot spots, suggests conflict resolution strategies. |
| Income Attribution by Identity | "Your @jordan-dev identity generated $34,200 this quarter; @studio-collective generated $18,500. Both are under-reporting portable benefits contributions — want me to create the missing BenefitAccount rows?" |

# **18\.  Platform Integration Map**

| kogi-\* Module | Dependency on KMDSS | Data Flow | Identity Scope |
| :---- | :---- | :---- | :---- |
| kogi-home | Reads Master Registry for dashboard overview: active program count, portfolio health summary, wallet balance, upcoming deadlines | Read: portfolio → home dashboard; respects active identity context | Shows data for currently active identity |
| kogi-office (boards/timelines) | Reads SHT-004 (Projects) \+ SHT-005 (Tasks) for board and gantt rendering; writes task status transitions and sprint completions back | Read \+ Write: portfolio ↔ office; real-time via CRDT | Work identity partition by default; configurable |
| kogi-bank | Reads SHT-009 (Finances), SHT-015 (Benefits), SHT-016 (Grants), SHT-017 (Crowdfunding); writes payment events, benefit balance updates, and ledger transactions back | Read \+ Write: portfolio ↔ bank; financial columns always owner-only visibility | All financial data always scoped to Sovereign Entity; never identity-partitioned below owner level |
| kogi-marketplace | Reads SHT-026 (Marketplace Listings) for published assets; writes investment/donation action events; reads talent from SHT-006 (Resources) | Read \+ Write: portfolio ↔ marketplace; KLNK link created on deal acceptance | Public identity used for marketplace presence; deal rows linked to work identity |
| kogi-community | Reads public rows from Master Registry for community feed; writes engagement events (likes, shares, mentions) to analytics columns | Read \+ Write: portfolio ↔ community; engagement events update AnalyticsColumn values | Community presence uses the active public identity (@handle shown on posts) |
| kogi-exchange | Reads SHT-017 (Exchange), SHT-027 (Deals); writes deal execution events and matched allocation records | Read \+ Write: portfolio ↔ exchange; InvestedIn KLNK edges created on investment | Investment identity or primary financial identity |
| kogi-profile (KPRF) | Profile rows ARE PortfolioComponents in SHT-025; linktree leaves stored as Resource rows; linked accounts sync to asset columns | Read \+ Write: portfolio ↔ profile; profile update regenerates public view | Each ProfilePartition generates one public profile at its @handle URL |
| kogi-engine | Reads all sheets via EventLog subscription for feature extraction; writes computed column values back via WritebackService | Read \+ Write: portfolio ↔ engine; all engine signals are AIColumns | Engine operates on full Sovereign Entity scope; per-column visibility mask applied before any public exposure |
| kogi-studio | Reads and writes Artifact rows (creative outputs); links design assets to parent Project rows via Hierarchy edges | Read \+ Write: portfolio ↔ studio; creative outputs create new Artifact rows | Creative identity partition; artifacts tagged to creative profile |
| kogi-dev (API/SDK) | Full read/write access to all sheets and rows via kogi-dev API; PortfolioPlugin trait for custom computed columns and automation | Full bidirectional: portfolio ↔ developer; OAuth-scoped per API key | API key scoped to specific partitions and sheets at creation |
| KOGI-MANAGER | Reads PolicyEngine attachments; writes RBAC policies and governance configurations; enforces PermissionTier on all mutations | Policy: KOGI-MANAGER → portfolio; policy updates trigger PolicyEngine re-evaluation | Platform-admin scope; all partitions |
| KOGI-APPSTORE | Distributes Portfolio Templates, custom column sets, computed model plugins, view theme packages | Distribution: APPSTORE → portfolio templates; template instantiation creates new sheet views | Template installation applies to selected identity partition |

# **19\.  Core User Flows — End-to-End**

## **19.1  Independent Worker — Setup Master Spreadsheet**

14. Create account → Kogi onboarding wizard launches Portfolio Template Selector.

15. Choose template: "Independent Worker Starter" → root PortfolioSystem provisioned with 8 default sheets.

16. Identity setup: wizard prompts for @handle → primary ProfilePartition created.

17. Profile completion: Oba prompts for skills, availability, rate → Resource row created; BenefitAccount rows provisioned.

18. Platform connections: link gig platforms (DoorDash, Uber, etc.) → provider integrations activated; Gig rows auto-imported.

19. Bank connection: link kogi-bank → financial columns populated; tax settings configured.

20. Master spreadsheet is live: Master Registry shows first set of rows; Finances Sheet shows first income entries; Benefits Sheet shows placeholder BenefitAccounts.

## **19.2  Creating and Linking a Second Identity**

21. Open SHT-032 (Identity & Profiles sheet) → "Add Identity" → enter @handle for new creative identity.

22. New ProfilePartition provisioned. VisibilityMask configured: creative projects Public; financial data Owner-only.

23. Tag existing rows: select Artifact rows for design work → "Tag to Identity" → "@jordan-art" applied.

24. New rows created in creative context: new Projects, Artifacts, Gig rows auto-tagged to active identity via session context.

25. Configure public view: select columns visible on @jordan-art public profile → save as public\_sheet\_view.

26. Link network: @jordan-art becomes a KLNK node. Others can follow, discover, and link to this identity independently from @jordan-dev.

## **19.3  Connecting Two Portfolios (KLNK Link)**

27. User A (@ jordan-dev) finds User B (@studio-collective) on the platform via Discover feed.

28. Jordan clicks "Connect" on a specific project in the studio-collective portfolio.

29. KLNK creates a pending Collaborates LinkEdge. @studio-collective receives a link request notification (KNTF).

30. @studio-collective reviews request, configures which columns of their project will be visible in Jordan's ShadowRow, accepts.

31. LinkEdge confirmed. ShadowRow created in Jordan's spreadsheet. Studio-collective's project appears in Jordan's SHT-020 (Shared Portfolios) sheet as a linked row with the configured columns visible.

32. Jordan can now see updates to studio-collective's project (within visibility config) in real-time via MirrorColumn sync.

33. Both users' link\_count column updates. The edge appears in both users' SHT-031 (Link Network) sheets.

## **19.4  Cooperative Monthly Distribution**

34. Month-end: cooperative treasury has surplus. Treasurer opens SHT-018 (Group Economics sheet).

35. Oba detects month-end trigger and pre-computes Shapley Value distribution based on contribution weights from SHT-021.

36. Oba presents distribution preview: each member row shows proposed distribution amount and attribution weight breakdown.

37. Treasurer initiates GovernanceProposal → members receive KNTF notification → vote via SHT-019 (Organizations sheet).

38. Quorum reached, proposal passes → Oba prepares batch payout transaction set in kogi-bank.

39. Multi-sig approvers confirm → batch executed → each member's Payroll Wallet credited.

40. Payroll Journal entries update cooperative's SHT-009 (Finances sheet). Attribution weights recomputed by CollaborationEngine.

## **19.5  Querying the Link Network Forest**

41. User opens KLNK panel from any row with link\_count \> 0\.

42. LinkTree query: GET /klnk/nodes/{component\_id}/tree?depth=3\&edge\_type=Collaborates,InvestedIn,Contracted

43. KLNK renders tree: root node \= selected component; branches \= linked components in connected portfolios; leaves \= third-degree connections.

44. User can filter by edge type, visibility, mutual connection count, or engine score.

45. Clicking a node navigates to the ShadowRow for that component (if one exists) or shows the public view of the linked component.

46. User selects "Link Forest View" to see all trees rooted at all their identities side-by-side — the full picture of their network.

47. Export: GET /klnk/sheet renders the same data as SHT-031 — all link edges as spreadsheet rows for bulk analysis or export.

# **20\.  Open Items & Development Roadmap**

## **20.1  Current Open Items (v3.0 → v3.5)**

| Item | Priority | Description |
| :---- | :---- | :---- |
| Persistence Layer Pluggability | P0 | PortfolioSystem is currently in-memory with PostgreSQL being integrated. Full PortfolioStore trait abstraction needed for pluggable backends (SQLite for local, PostgreSQL for cloud, CouchDB for P2P). |
| Status CRDT Merge Strategy | P0 | apply\_crdt\_op() currently skips status mutations. A custom CRDT lattice based on ComponentStatus lifecycle ordering must be specified before multi-node deployments. |
| Cross-Partition Write Policy | P0 | SplitPolicy is defined but enforcement at the Substrate layer is not yet implemented. Cross-partition edits must be validated before any multi-identity deployment. |
| EventLog Flush to Data Lake | P1 | 10,000 event cap with FIFO eviction risks historical loss. Plugin hook must flush older events to ClickHouse / S3 before eviction, with full restoration path. |
| KLNK Consent Workflow | P1 | For consent-required edge types (Collaborates, InvestedIn), the consent flow (invitation, negotiation, column visibility configuration) is specified but not yet implemented in the LinkEdge service. |
| ShadowRow Real-Time Sync | P1 | MirrorColumn sync currently batch-based. Real-time sync via Kafka event subscription from source component's EventLog needs to be implemented and load-tested. |
| AI Agent Writeback Protocol | P1 | WritebackService is defined but not yet enforcing the full permission check and audit trail. The typed protocol for engine writeback must be hardened before production. |
| Formula Engine (v1) | P1 | ColumnComputer formula engine is defined but not fully implemented. v3.0 must ship the complete expression language evaluator with all listed functions including RELATED() and engine functions. |
| PQL Full Implementation | P2 | PQL parser and evaluator must be implemented against the PostgreSQL storage layer with full JOIN support for RelationField columns and SHADOW INCLUDE syntax. |
| Real-Time Collaborative UI | P2 | WebSocket-based live cursor indicator and cell update propagation — the full multi-user spreadsheet experience with presence indicators. |
| Link Forest Visualization | P2 | The KLNK Forest and Tree rendering modes (force-directed graph, dependency tree) need a production-quality visualization component. D3.js \+ WebGL for large graphs. |
| Spreadsheet Import Pipeline | P2 | CSV/XLSX import with Oba-assisted column mapping wizard and batch row creation with identity partition assignment and progress tracking. |
| Identity Merge Conflict Resolution | P2 | CrossIdentityMerge operation needs a UI-level conflict resolution wizard for cases where both source and target identities have rows with the same component\_id. |
| KLNK Centrality Computation | P3 | Betweenness and eigenvector centrality for the full KLNK graph is computationally expensive at platform scale. Needs a distributed approximation algorithm (e.g. sampled random walk) for graphs \>1M nodes. |
| Template Marketplace | P3 | KOGI-APPSTORE integration for distributing, rating, and purchasing portfolio templates; template versioning and update notifications. |
| On-Chain Identity Anchoring | P3 | For pseudonymous identities requiring external verifiability: DID (Decentralized Identifier) anchoring for identity handles via W3C DID spec. Enables cross-platform reputation portability. |
| Federated MatchEngine | P3 | MatchEngine operating across ShangoOS federated portfolio nodes — cross-federation talent and resource matching against ShadowRows. |

## **20.2  Version Roadmap**

| Version | Milestone | Key Deliverables |
| :---- | :---- | :---- |
| v2.2 (Current) | Portfolio Spreadsheet Foundation | Master Registry Sheet · Hierarchy Sheet · Projects Sheet · Finances Sheet · Benefits Sheet · Work & Gigs Sheet · PortfolioHealth v1 · Basic CRDT · PostgreSQL persistence · Single-identity mode |
| v2.5 | Intelligence & Collaboration | AI ComputedColumns (health, risk, income projection) · CollaborationEngine · Shared Portfolios · Contribution Attribution · CrowdresourcingCampaign · Grant & Crowdfunding Sheets · WritebackService v1 |
| v3.0 | Full Spreadsheet Platform | PQL · Formula Engine · All 35 Sheets · All Board Modes · Template Marketplace · Real-time Collaboration UI · Full Import/Export Pipeline · Oba Spreadsheet Intelligence · KLNK v1 (basic link creation and ShadowRows) |
| v3.5 | Link Network & Multi-Identity | Full KLNK graph API · LinkForest and LinkTree rendering · Multi-identity KPID system · CrossIdentityMerge · ShadowRow real-time sync · Identity-aware VisibilityMask · KLNK centrality computation · Federation of portfolio nodes |
| v4.0 | Autonomous Intelligence & Federation | Oba autonomous portfolio management (approval-first) · Predictive analytics and scenario simulation · AI-generated portfolio narratives · Cross-platform reputation portability via verifiable credentials · DID anchoring · Federated MatchEngine across ShangoOS |
| v4.5 | Web3 & Decentralization | On-chain identity anchoring · Decentralized portfolio storage option (IPFS \+ CouchDB) · Smart contract-backed cooperative distribution · On-chain equity and cap table management · Cross-federation KLNK traversal across decentralized nodes |

## **20.3  Glossary**

| Term | Definition |
| :---- | :---- |
| KMDSS | Kogi Master Distributed Spreadsheet System — the entire system described in this document |
| KPMS | Portfolio Management System — the master orchestration layer of the KMDSS |
| KPSS | Portfolio Spreadsheet Substrate — the low-level Rust data engine: PortfolioRow, ColumnSchema, CellStore |
| KPRG | Portfolio Component Registry — the canonical master index of all PortfolioComponents system-wide |
| KPVW | View Engine — filter, sort, group, pivot, board mode transforms |
| KPCM | Computation Engine — derived columns, analytical models, rollup aggregations |
| KPCL | Collaborative Editing — CRDT-backed multi-user real-time editing |
| KPEX | Export & Integration — CSV, XLSX, JSON, webhook, embed export |
| KLNK | Link Network — the inter-portfolio graph system: forests, trees, cross-user connections |
| KPID | Portfolio Identity System — multi-account, multi-identity, multi-profile management |
| PortfolioComponent | The universal node. Every row in any sheet is a PortfolioComponent (Item or Container). |
| PortfolioRow | A PortfolioComponent projected as a spreadsheet row: flat key-value map of ColumnId → TypedCellValue |
| SheetDefinition | A named, scoped, filterable view definition: base row type filter \+ visible column schema \+ default sort/group |
| ViewDefinition | A user-saved customization of a SheetDefinition: filters, sorts, groups, column widths, highlight rules, board config |
| CellValue | The typed value at a row-column intersection. Variants: Text | Number | Currency | Date | Enum | Relation | User | Tag | Bool | Json | Null |
| EventLog | Append-only, immutable log of every mutation on a PortfolioComponent. Never deleted. |
| CrdtLog | LWW \+ OR-Set CRDT operation log. Applied on merge to resolve concurrent edits without data loss. |
| VectorClock | Logical clock per federation node. Provides causal ordering of all mutations. |
| InterPortfolioLink | A GraphEdge crossing the boundary between two different users' root spreadsheets. The atomic unit of KLNK. |
| LinkEdge | A typed, directional edge between two LinkNodes in the KLNK graph. |
| LinkForest | The complete set of all LinkTrees rooted at a given Sovereign Entity — the full picture of their network. |
| LinkTree | A rooted, directed subgraph of KLNK from a single root node to a configured depth. |
| ShadowRow | A read-only reflection in one user's spreadsheet of a linked component from another user's spreadsheet. |
| MirrorColumn | A subset of columns from a ShadowRow made visible as a ComputedColumn in the host spreadsheet. |
| Sovereign Entity | The real-world person, organization, collective, or cooperative that owns a root PortfolioSystem instance. |
| ProfilePartition | A logical partition of the root spreadsheet's rows, owned by one Identity. Rows are tagged with identity\_tags. |
| VisibilityMask | A per-account or per-profile filter specifying which rows and columns are visible to which observer. |
| SplitPolicy | A governance policy governing how data is partitioned across identity partitions. |
| CrossIdentityMerge | The operation of unifying two identity partitions under one Sovereign Entity. |
| Oba | The Kogi platform AI assistant — the spreadsheet's AI chief-of-staff |
| kogi-engine | The Scala analytics and intelligence engine; consumes all portfolio events via gRPC/Kafka; writes back computed columns via WritebackService |
| WritebackService | The gRPC service that accepts engine-computed values and writes them to AIColumns, respecting all permission and policy checks |
| CRDT | Conflict-free Replicated Data Type — LWW \+ OR-Set — enables safe concurrent edits across all nodes and identities without conflicts |

