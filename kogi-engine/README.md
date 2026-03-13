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
- `AnalyticsEngine`: realtime analytics signals, health scoring, and system snapshots.
- `TelemetryEngine`: platform flow ingest, envelope/ledger tracking, and component stats.
- `RecommendationEngine`: recommendations + discovery + explore synthesis.
- `PersonalizationEngine`: segments, experiments, persona-driven delivery configs.
- `MatchEngine`: multi-dimensional matching across users, components, resources, artifacts.
- `GraphEngine`: dependency graph traversal, impact analysis, cycles, critical path, diff.
- `RiskEngine`: portfolio risk scoring and risk optimization plans.
- `PolicyEngine`: governance policy evaluation + approval workflows.
- `OptimizationEngine`: workload optimization heuristics and plans.
- `SearchEngine`: search, filtering, indexing, personalized re-ranking.
- `QueryEngine`: SQL interpretation + optimization hints for execution.
- `AllocationEngine`: bid scoring + allocation for game listings.
- `IncentiveEngine`: KP ledger, incentive profiles, streak logic.
- `GameEngine`: listings, bids, allocations, incentives orchestration.
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
The engine exposes a lightweight CLI (JSON output) for automation and the Go engine service.

```powershell
cd kogi-engine
sbt "runMain kogi.engine.KogiEngineCli --action status"
sbt "runMain kogi.engine.KogiEngineCli --action control --mode start"
sbt "runMain kogi.engine.KogiEngineCli --action ingest --topic engine.ingest --payload event_type=demo"
sbt "runMain kogi.engine.KogiEngineCli --action snapshot --host-id kogi-host-001 --window-ms 300000"
sbt "runMain kogi.engine.KogiEngineCli --action search --query \"portfolio risk\" --tag risk --meta domain=finance"
sbt "runMain kogi.engine.KogiEngineCli --action graph-impact --node auth-service"
```

Common actions include `control`, `status`, `ingest`, `snapshot`, `query`, `search`, `index`,
`match-profiles-resources`, and `graph-*` (impact/closure/rdeps/neighbors/ancestors/descendants/cycles/topo/critical/diff).
Use `GraphEngineCLI` for graph-only workflows with `--format text|json|dot`.

## gRPC Server
The engine can expose a gRPC endpoint used by the Go engine service. Requests and responses
use `google.protobuf.Struct`, and reflection is enabled.

```powershell
cd kogi-engine
sbt "runMain kogi.engine.EngineGrpcServer"
```

Set `KOGI_ENGINE_GRPC_PORT` to override the default `9100`.
Set `KOGI_ENGINE_GRPC_REFLECTION` to `off`/`false`/`0` to disable reflection (enabled by default).
Set `KOGI_ENGINE_GRPC_PROTO` to the proto file path (used for reflection metadata/logging; default `kogi_engine.proto`).

Run via Bazel:
```powershell
bazel run //:engine_grpc_run
```

### grpcurl
Reflection is enabled by default, so you can use `grpcurl` directly. Full command set
(matches `grpcurl-commands.ps1`):
```bash
grpcurl -plaintext localhost:9100 list
grpcurl -plaintext localhost:9100 describe kogi.engine.v1.EngineService
grpcurl -plaintext -d '{}' localhost:9100 kogi.engine.v1.EngineService/Status
grpcurl -plaintext -d '{"action":"start"}' localhost:9100 kogi.engine.v1.EngineService/Control
grpcurl -plaintext -d '{"topic":"engine.ingest","source":"kogi.network.engine","target":"kogi.engine","payload":{"event_type":"demo"}}' localhost:9100 kogi.engine.v1.EngineService/Ingest
grpcurl -plaintext -d '{"host_id":"kogi-host-001","window_ms":300000}' localhost:9100 kogi.engine.v1.EngineService/Snapshot
grpcurl -plaintext -d '{"action":"pause"}' localhost:9100 kogi.engine.v1.EngineService/Control
grpcurl -plaintext -d '{"action":"stop"}' localhost:9100 kogi.engine.v1.EngineService/Control
```

You can also run `.\grpcurl-commands.ps1` to print and execute the same list.

If reflection is disabled, point grpcurl at the proto file:
```powershell
grpcurl -plaintext -import-path kogi-infra/proto -proto kogi_engine.proto localhost:9100 kogi.engine.v1.EngineService/Status
netstat -ano | findstr 9100
```

## Engine Service (Host Control Plane)
The host calls the engine service for control + ingest. Run it from the Go services workspace:

```powershell
go run ./kogi-network/services/engine
```

Set `KOGI_ENGINE_GRPC_MODE` to `on` or `required` to enable gRPC forwarding (default off unless addr is set).
Set `KOGI_ENGINE_GRPC_ADDR` to override the gRPC target (default `127.0.0.1:9100` when mode is `on`/`required`).
Set `KOGI_ENGINE_GRPC_TIMEOUT` to a duration like `2s`.
Set `KOGI_ENGINE_CLI` to point at a compiled CLI binary/script, or ensure `sbt` is on `PATH` so the Go engine service can call `KogiEngineCli`.
Set `KOGI_ENGINE_ENDPOINT` only if you also expose a Scala HTTP endpoint (legacy path).

## Core APIs
- `KogiEngine.control(action: String)` / `status`: lifecycle control.
- `KogiEngine.ingestEnvelope(envelope: PlatformDataEnvelope)`: platform-wide canonical ingest entrypoint.
- `KogiEngine.ingestGatewayMessage(topic, payload, ...)`: gateway pub/sub payload ingestion.
- `KogiEngine.snapshot(hostId, windowMs)`: combined engine + host + module realtime snapshot.
- `KogiEngine.flowLedger(limit)` / `flowEnvelopes(limit)`: data flow audit trail.
- `KogiEngine.search(query: SearchQuery)` / `index(document: SearchDocument)`: search + indexing.
- `KogiEngine.query(sql: String)`: query analysis + optimization hints.
- `KogiEngine.graphAddEdge(...)`, `graphImpact(...)`, `graphCycles(...)`, `graphCriticalPath(...)`, `graphDiff(...)`.
- `KogiEngine.matchProfilesToResources(...)`: profile/resource matching.
- `KogiEngine.allocate(...)` / `scoreAllocation(...)`: allocation scoring + selection.
- `KogiEngine.incentiveApply(...)`, `incentiveProfile(...)`, `incentiveLedger(...)`: incentives + KP ledger.
- `KogiEngine.gameRegisterParticipant(...)`, `gameUpsertListing(...)`, `gameSubmitBid(...)`, `gameAllocateListing(...)`, `gameSnapshot(...)`.
