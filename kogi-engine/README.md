# kogi-engine

Scala data/data-processing engine for Kogi platform-wide flows and host analytics.

## Role
- Central data engine for all platform components.
- Ingests stream events from kernel, host, server, services, modules, and clients.
- Processes realtime analytics, recommendations, discovery, exploration, telemetry, search, query planning, and optimization.
- Produces host/module/system snapshots used by orchestration and UIs.
- Exposed to the host via the `kogi-services/go/services/engine` control + ingest facade.

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

grpcurl -plaintext -d "{\"topic\":\"engine.ingest\",\"source\":\"kogi.services.engine\",\"target\":\"kogi.engine\",\"payload\":{\"event_type\":\"demo\"}}" localhost:9100 kogi.engine.v1.EngineService/Ingest

grpcurl -plaintext -import-path kogi-contracts/proto -proto kogi_engine.proto localhost:9100 kogi.engine.v1.EngineService/Status

grpcurl -plaintext -d "{\"action\":\"start\"}"  -import-path kogi-contracts/proto -proto kogi_engine.proto localhost:9100 kogi.engine.v1.EngineService/Control

grpcurl -plaintext -d "{\"topic\":\"engine.ingest\",\"source\":\"kogi.services.engine\",\"target\":\"kogi.engine\",\"payload\":{\"event_type\":\"demo\"}}" -import-path kogi-contracts/proto -proto kogi_engine.proto localhost:9100 kogi.engine.v1.EngineService/Ingest

netstat -ano | findstr 9100

```

## Engine Service (Host Control Plane)
The host calls the engine service for control + ingest. Run it from the Go services workspace:

```powershell
go run ./kogi-services/go/services/engine
```

Set `KOGI_ENGINE_CLI` to point at a compiled CLI binary/script, or ensure `sbt` is on `PATH` so the Go engine service can call `KogiEngineCli`.
Set `KOGI_ENGINE_ENDPOINT` only if you also expose a Scala HTTP endpoint (legacy path).

## Core APIs
- `KogiEngine.ingestEnvelope(envelope: PlatformDataEnvelope)`: platform-wide canonical ingest entrypoint.
- `KogiEngine.ingestBusEvent(topic, payload, ...)`: gateway pub/sub payload ingestion.
- `KogiEngine.snapshot(hostId)`: combined engine + host + module realtime snapshot.
- `KogiEngine.search(query: SearchQuery)`: search + filter over indexed documents.
- `KogiEngine.query(sql: String)`: query analysis + optimization hints.
