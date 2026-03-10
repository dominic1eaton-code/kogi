# kogi-engine

Scala data/data-processing engine for Kogi platform-wide flows.

## Role
- Central data engine for all platform components.
- Ingests stream events from kernel, host, server, services, modules, and clients.
- Processes realtime analytics, recommendations, discovery, and exploration.
- Produces host/module/system snapshots used by orchestration and UIs.

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

## Core APIs
- `ingest(event: StreamEvent)`: profile-level analytics update.
- `ingestBusEvent(topic, payload, ...)`: gateway pub/sub payload ingestion.
- `ingestEnvelope(envelope: PlatformDataEnvelope)`: platform-wide canonical ingest entrypoint.
- `snapshot(hostId)`: combined engine + host + module realtime snapshot.
