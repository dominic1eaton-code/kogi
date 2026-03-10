# kogi-analytics-scala

Scala analytics and AI/data pipeline management infrastructure for Kogi MVP.

## Engines
- `PortfolioHealthPipeline`: weighted health scoring for productivity/cashflow/collaboration/risk signals
- `DataStreamingEngine`: in-memory event stream ingest + subscription + bounded retention
- `AnalyticsRecommendationDiscoverExploreEngine`: unified analytics + recommendation + discover + explore engine over stream windows
- Real-time module analytics: throughput, error rate, blocked rate, latency p95, queue depth, cpu/memory utilization
- Real-time host analytics: cpu/memory/disk/process/scheduler/network telemetry with saturation scoring and anomaly detection
- System snapshot synthesis: host + module snapshots with system recommendations/discover/explore cards

## Capabilities
- Streaming ingest of module events (office/bank/exchange/community/studio/etc.)
- Derived profile-level signal calculation
- Health scoring and risk-aware recommendations
- Discover cards for trending modules/events/providers
- Explore cards for next-best actions and expansion opportunities
- Module-by-module realtime analytics for `kogi-kernel`, `kogi-host`, `kogi-server`, gateway and services
- Host-level realtime analytics for central Kogi host orchestration runtime
- Bus-topic ingestion (`ingestBusEvent`) to consume gateway/eventbus messages and map them to module + host analytics

## Build
```powershell
cd kogi-analytics-scala
sbt compile
```

## Run
```powershell
cd kogi-analytics-scala
sbt "runMain kogi.analytics.Main"
```

## Core realtime API
- `ingest(event: StreamEvent)`: profile-level analytics update
- `ingestModuleMetric(sample: ModuleMetricSample)`: module telemetry update
- `ingestHostMetric(sample: HostMetricSample)`: host telemetry update
- `moduleSnapshot(module, windowMs)`: realtime module analytics snapshot
- `hostSnapshot(hostId, windowMs)`: realtime host analytics snapshot
- `systemSnapshot(hostId, modules, windowMs)`: combined host + module system snapshot
- `ingestBusEvent(topic, payload, ...)`: gateway/eventbus payload ingestion path
