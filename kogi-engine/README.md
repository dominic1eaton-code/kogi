# kogi-engine

Scala data/data-processing engine for Kogi platform-wide flows.

## Role
- Central data engine for all platform components.
- Ingests stream events from kernel, host, server, services, modules, and clients.
- Processes realtime analytics, recommendations, discovery, and exploration.
- Produces host/module/system snapshots used by orchestration and UIs.
- Exposed to the host via the `kogi-services/go/services/engine` control + ingest facade.

## Engines
- `PortfolioHealthPipeline`: weighted health scoring for productivity/cashflow/collaboration/risk signals.
- `DataStreamingEngine`: bounded event stream ingest with subscriber hooks.
- `AnalyticsRecommendationDiscoverExploreEngine`: realtime analytics/recommendation core.
- `KogiPlatformDataEngine`: end-to-end platform dataflow processor and flow ledger.

## Build
```powershell
cd kogi-engine
sbt compile
```

## Run
```powershell
cd kogi-engine
sbt "runMain kogi.engine.Main"
```

## Engine Service (Host Control Plane)
The host calls the engine service for control + ingest. Run it from the Go services workspace:

```powershell
go run ./kogi-services/go/services/engine
```

## Core APIs
- `ingest(event: StreamEvent)`: profile-level analytics update.
- `ingestBusEvent(topic, payload, ...)`: gateway pub/sub payload ingestion.
- `ingestEnvelope(envelope: PlatformDataEnvelope)`: platform-wide canonical ingest entrypoint.
- `snapshot(hostId)`: combined engine + host + module realtime snapshot.
