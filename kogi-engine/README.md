# kogi-engine

Scala data/data-processing engine for Kogi platform-wide flows and host analytics.

## Role
- Central data engine for all platform components.
- Ingests stream events from kernel, host, server, services, modules, and clients.
- Processes realtime analytics, recommendations, discovery, exploration, telemetry, search, query planning, and optimization.
- Produces host/module/system snapshots used by orchestration and UIs.
- Exposed to the host via the `kogi-network/services/engine` control + ingest facade.

## Engines
- `KogiEngine`: single access point composing all subengines.
- `AnalyticsEngine`: realtime analytics signals, health scoring, and module/host snapshots.
- `TelemetryEngine`: platform flow ingest, envelope/ledger tracking, and component stats.
- `RecommendationEngine`: recommendations + discovery + explore synthesis.
- `OptimizationEngine`: workload optimization heuristics and plans.
- `SearchEngine`: search, filtering, indexing for platform data.
- `QueryEngine`: SQL interpretation + optimization hints for execution.
- `PortfolioHealthPipeline`: weighted health scoring for productivity/cashflow/collaboration/risk signals.
- `DataStreamingEngine`: bounded event stream ingest with subscriber hooks.

## Build
```powershell
cd kogi-engine
sbt compile
```

## Build (Bazel)
```powershell
bazel build //:engine_build
```

## Run
```powershell
cd kogi-engine
sbt "runMain kogi.engine.Main"
```

## Run (Bazel)
```powershell
bazel run //:engine_run
```

## CLI
The engine exposes a lightweight CLI for the Go engine service:

```powershell
cd kogi-engine
sbt "runMain kogi.engine.KogiEngineCli --action snapshot"
sbt "runMain kogi.engine.KogiEngineCli --action control --mode start"
```

## gRPC Server
The engine can expose a gRPC endpoint used by the Go engine service:

```powershell
cd kogi-engine
sbt "runMain kogi.engine.EngineGrpcServer"
```

Set `KOGI_ENGINE_GRPC_PORT` to override the default `9100`.

Run via Bazel:
```powershell
bazel run //:engine_grpc_run
```

### grpcurl
The server enables reflection, so you can use `grpcurl` directly:
```powershell

grpcurl -plaintext localhost:9100 list

grpcurl -plaintext localhost:9100 describe kogi.engine.v1.EngineService

grpcurl -plaintext -d "{}" localhost:9100 kogi.engine.v1.EngineService/Status

grpcurl -plaintext -d "{\"action\":\"start\"}" localhost:9100 kogi.engine.v1.EngineService/Control

grpcurl -plaintext -d "{\"topic\":\"engine.ingest\",\"source\":\"kogi.network.engine\",\"target\":\"kogi.engine\",\"payload\":{\"event_type\":\"demo\"}}" localhost:9100 kogi.engine.v1.EngineService/Ingest

grpcurl -plaintext -import-path kogi-infra/proto -proto kogi_engine.proto localhost:9100 kogi.engine.v1.EngineService/Status

grpcurl -plaintext -d "{\"action\":\"start\"}"  -import-path kogi-infra/proto -proto kogi_engine.proto localhost:9100 kogi.engine.v1.EngineService/Control

grpcurl -plaintext -d "{\"topic\":\"engine.ingest\",\"source\":\"kogi.network.engine\",\"target\":\"kogi.engine\",\"payload\":{\"event_type\":\"demo\"}}" -import-path kogi-infra/proto -proto kogi_engine.proto localhost:9100 kogi.engine.v1.EngineService/Ingest

netstat -ano | findstr 9100

```

## Engine Service (Host Control Plane)
The host calls the engine service for control + ingest. Run it from the Go services workspace:

```powershell
go run ./kogi-network/services/engine
```

Set `KOGI_ENGINE_CLI` to point at a compiled CLI binary/script, or ensure `sbt` is on `PATH` so the Go engine service can call `KogiEngineCli`.
Set `KOGI_ENGINE_ENDPOINT` only if you also expose a Scala HTTP endpoint (legacy path).

## Core APIs
- `KogiEngine.ingestEnvelope(envelope: PlatformDataEnvelope)`: platform-wide canonical ingest entrypoint.
- `KogiEngine.ingestBusEvent(topic, payload, ...)`: gateway pub/sub payload ingestion.
- `KogiEngine.snapshot(hostId)`: combined engine + host + module realtime snapshot.
- `KogiEngine.search(query: SearchQuery)`: search + filter over indexed documents.
- `KogiEngine.query(sql: String)`: query analysis + optimization hints.

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