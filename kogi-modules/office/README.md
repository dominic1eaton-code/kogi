# Kogi Office

- Module ID: `kogi.office`
- Runtime language: `hybrid-rust-go`
- Host entrypoint: `kogi-network/services/office`
- Network manager: `kogi-go-network`

## Scope
- Dashboard for active projects/programs, attention items, notifications, direct messages, event feeds, personas/roles, and quick links.
- Portfolio for tiled/tree/modular grid inventory and focus view with binder/book/notebook/playbook/folder/file/version/metadata containers.
- Timeline for calendars, schedules, roadmaps, gantts, and personal timelines.
- Workspace for work/operations/tactics/strategy/governance with stories, work packages, CMS collections, and tools/toolchains.
- Assistant for digital AI chat context, discovery, recommendations, subscriptions, explore, and for-you cards.

## Integrations
`jira`, `monday`, `base44`, `claude`, `chatgpt`, `grok`, `openai`, `gitlab`, `github`

## Service Endpoints
- `GET /api/v1/office`
- `GET /api/v1/office/dashboard`
- `GET /api/v1/office/portfolio`
- `GET /api/v1/office/timeline`
- `GET /api/v1/office/workspace`
- `GET /api/v1/office/assistant`

## Rust System Bridge
The Go office service can invoke the Rust office system binary when available:

```powershell
cargo build --manifest-path kogi-modules/office/Cargo.toml
$env:KOGI_OFFICE_SYSTEM_BIN = "C:\path\to\kogi-office-system.exe"
```



## Notes

### GraphEngine

---

# What This Enables in Your Portfolio System

### Dependency Closure

Find **everything a project depends on**.

Example:

```
A -> B -> C
```

```
dependencyClosure(A)
= {B, C}
```

---

### Reverse Dependency

Find **everything that depends on a project**.

```
reverseDependencyClosure(C)
= {A, B}
```

Useful for:

* release planning
* change risk analysis
* cascading failure detection

---

### Impact Analysis

```
impactAnalysis(projectX)
```

Returns:

```
ImpactReport(
  root = projectX
  affected = downstream dependencies
  dependents = upstream dependents
)
```

Example:

```
projectX failure
 ↓
libraries
 ↓
applications
```

Impact analysis shows **blast radius**.

---

### Hierarchical Traversal

Supports **nested portfolio DAG**.

```
Portfolio
  Program
    Project
      Task
```

Queries:

```
descendants(program)
ancestors(project)
```

---

# Example Usage

```scala
val edges = Seq(

  GraphEdge("projectA", "projectB", Dependency),
  GraphEdge("projectB", "projectC", Dependency),

  GraphEdge("portfolio1", "program1", Hierarchy),
  GraphEdge("program1", "projectA", Hierarchy)

)

val graph = new GraphEngine(edges)

val closure = graph.dependencyClosure("projectA")

val impact = graph.impactAnalysis("projectB")

val subtree = graph.descendants("portfolio1")
```

---

# Performance

Traversal complexity:

```
O(V + E)
```

Efficient for large portfolio graphs.

Works well up to:

```
100k+ nodes
millions of edges
```

---

# If you want, I can also extend this into a **full portfolio graph runtime**, adding:

### Advanced Graph Capabilities

* **Topological sorting (project scheduling)**
* **Critical path analysis**
* **Cycle detection**
* **Graph diff between snapshots**
* **Graph simulation engine**
* **Graph persistence layer**
* **GraphQL-style query interface**

Which would turn your system into something closer to a **Portfolio Knowledge Graph Engine / Governance Graph OS**.

---

Now I have a thorough understanding of the codebase. Let me write the complete implementations.Now let me verify both files compile cleanly and check for any type references that need validation:Good — those were placeholder stubs with undefined types. The new implementation replaces all of them with fully typed implementations. Let me present the files:Here's a summary of what was built:

---

**MatchEngine** — replaces all the empty stubs with a fully typed, multi-dimensional matching system:

- **Subject types**: `UserSubject` (with `UserRole`: Owner, Investor, Donor, Subscriber, etc.), `ComponentSubject` (Portfolio/Program/Project/Resource/Artifact/Asset), `ResourceSubject`, `AssetSubject`, `AnalyticsArtifact`
- **Core match pairs**: user↔component, component↔users (filtered by role), asset↔component, resource↔component, user↔user (talent/similarity)
- **`componentBundle`**: single call that returns matched users + resources + assets for any portfolio component
- **`matchExchangeListings`**: pairs marketplace listings with best-fit buyers/investors
- **`matchWorkloadsToPlans`**: delegates to `OptimizationEngine` and wraps results as match candidates
- **`matchArtifactsToUser`**: re-ranks search results, recommendations, and index items by persona + tag alignment
- **`MatchWeights`**: pluggable scoring strategies (`default`, `investment`, `talent`, `analytics`)
- Scoring uses Jaccard similarity for tags, attribute overlap, role→kind affinity, persona boosting, and budget/value signals

---

**PersonalizationEngine** — fully implemented across all requested dimensions:

- **Core entry point**: `personalize(request)` returns a `PersonalizationResult` with recommendations, delivery config, segments, experiment assignments, and persona metadata in one call
- **Preference management**: explicit/inferred/default three-tier fallback chain; `setPreference`, `resolvePreferences`
- **Content delivery adaptation**: `ContentDeliveryConfig` (density, ordering, feed limits, sidebar, filters) derived from persona × context
- **`rankContent` / `adaptContent`**: dynamic reordering of feed items using persona rules (suppress low-value for ValueSeekers, promote new for EarlyAdopters, cap depth for CasualBrowsers, etc.)
- **Segmentation**: `Segment` definitions with rule-based matching, `assignSegments`, `profilesInSegment`
- **A/B testing**: sticky variant assignment via weighted random sampling, `recordExposure`, `experimentStats`
- **Multi-armed bandit (UCB1)**: `registerBanditSlot`, `selectBanditArm`, `recordBanditReward`, `banditStats`
- **Persona lifecycle**: `buildPersona`, `detectPersonaDrift`, `applyPersonaDriftIfNeeded`
- Composes over `RecommendationEngine` — all interaction recording and hybrid recommendation calls pass through
