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

**Payload Conventions**
- Pass scalar inputs with `--key value` (both `snake_case` and `camelCase` are accepted for most fields).
- Build maps with `--payload key=value` or `--meta key=value` (you can repeat these flags).
- Add tags with `--tag value` or pass a CSV list with `--tags a,b,c`.
- Inline maps use comma or semicolon delimiters: `key=a,priority=high` or `key=a;priority=high`.
- Lists of maps use `|` between entries: `id=a,score=1|id=b,score=2`.
- For nested objects, you can either pass a sub-map key (for gRPC/JSON) or prefix fields in the CLI.

```text
# Map list example (events)
--events "id=e1,profile_id=u1,module=core|id=e2,profile_id=u2,module=search"

# Prefix example (listing fields)
--listing_id L-1 --listing_owner_id user-1 --listing_mechanism assignment
```

**Object Payloads**
- `StreamEvent`: `id`, `profile_id`, `module`, `event_type`, `timestamp_ms`, `productivity_delta`, `cashflow_delta`, `collaboration_delta`, `risk_delta`, `metadata`.
- `ModuleMetricSample`: `id`, `module`, `timestamp_ms`, `latency_ms`, `queue_depth`, `error_count`, `success_count`, `throughput_units`, `cpu_pct`, `memory_mb`, `metadata`.
- `HostMetricSample`: `id`, `host_id`, `timestamp_ms`, `cpu_pct`, `memory_pct`, `disk_pct`, `process_count`, `scheduler_load`, `network_in_kbps`, `network_out_kbps`, `metadata`.
- `ItemProfile`: `id`, `title`, `category`, `tags`, `attributes`, `popularity_score`, `created_ms`.
- `UserProfile`: `id`, `demographics`, `preference_vector`, `persona`, `persona_traits`, `persona_confidence`, `total_interactions`, `interaction_breakdown`, `dominant_categories`, `preferred_devices`, `preferred_time_of_day`, `created_ms`, `last_seen_ms`.
- `InteractionContext`: `device`, `location`, `time_of_day`, `referrer`, `metadata`.
- `UserInteraction`: `id`, `user_id`, `item_id`, `interaction_type`, `value`, `session_id`, `timestamp_ms`, `context`.
- `SearchQuery`: `text` or `query`, `tags`, `metadata`, `limit`.
- `SearchDocument`: `id`, `title`, `body`, `tags`, `metadata`.
- `PersonalizationRequest`: `profile_id`, `context`, `preferences`, `persona`.
- `Segment`: `id`, `name`, `rules` (map of string to list of strings), `description`.
- `Experiment`: `id`, `name`, `variants`, `target_segments`, `active`.
- `ExperimentVariant`: `id`, `name`, `weight`.
- `UserSubject`: `id`, `role`, `persona`, `skills`, `interests`, `budget`, `location`, `tags`, `attributes`.
- `ComponentSubject`: `id`, `kind`, `category`, `status`, `owner_ids`, `value_score`, `risk_score`, `tags`, `attributes`.
- `ResourceSubject`: `id`, `resource_type`, `capacity`, `unit`, `category`, `tags`, `attributes`.
- `AssetSubject`: `id`, `asset_class`, `market_value`, `liquidity`, `category`, `tags`, `attributes`.
- `Profile`: `id`, `tags`, `attributes`.
- `Resource`: `id`, `tags`, `attributes`.
- `AnalyticsArtifact`: `id`, `artifact_type`, `score`, `source_module`, `tags`, `attributes`.
- `MatchWeights`: `tag_overlap`, `attribute_overlap`, `category_match`, `persona_alignment`, `signal_strength`, `context_boost`.
- `OptimizationRequest`: `workload_id`, `cpu_pct`, `memory_pct`, `queue_depth`, `latency_ms`, `objectives`, `persona`.
- `RiskOptimizationRequest`: `profile_id`, `signal`, `context`.
- `PortfolioSignal`: `productivity`, `cash_flow`, `collaboration`, `risk`.
- `QueryProfile`: `user_id`, `persona`, `preferred_limit`, `allow_analytic_hints`.
- `PolicyContext`: `component_id`, `event_kind`, `actor_id`, `component_type`, `properties`, `tags`, `timestamp_ms`.
- `GameListing`: `id`, `owner_id`, `component`, `listing_type`, `mechanism`, `asking_price`, `reserve_price`, `quantity`, `min_quality_score`, `status`, `created_at_ms`.
- `GameBid`: `id`, `listing_id`, `bidder_id`, `amount`, `quality_score`, `bid_type`, `submitted_at_ms`, `status`, `metadata`.
- `GameScoreWeights`: `match_weight`, `price_weight`, `quality_weight`, `reputation_weight`.
- `GameAllocation`: `listing_id`, `winner_id`, `clearing_price`, `mechanism`, `composite_score`, `match_score`, `price_score`, `quality_score`, `reputation_score`, `reasons`, `created_at_ms`.
- `GameIncentive`: `participant_id`, `kp_delta`, `reputation_delta`, `reason`, `timestamp_ms`.
- `GameIncentivePolicy`: `winner_base_kp`, `owner_base_kp`, `winner_reputation_delta`, `owner_reputation_delta`, `quality_kp_scale`, `match_kp_scale`.
- `AllocationParticipant`: `subject`, `reputation`.
- `AllocationRequest`: `listing`, `bids`, `participants`, `score_weights`, `match_weights`, `limit`.
- `IncentiveEvent`: `participant_id`, `event_type`, `kp_delta`, `reputation_delta`, `reference_id`, `reference_type`, `timestamp_ms`.
- `GraphEdge`: `from`, `to`, `edge_type`, `weight`.
- `GraphNode`: `id`, `duration`.

**Enums**
- `role`: `owner`, `investor`, `donor`, `subscriber`, `watcher`, `follower`, `editor`, `contributor`, `any`.
- `kind`: `portfolio`, `program`, `project`, `resource`, `artifact`, `asset`, `any`.
- `persona`: `explorer`, `specialist`, `casual-browser`, `power-user`, `early-adopter`, `value-seeker`, `collaborator`, `newcomer`, `unknown`.
- `interaction_type`: `view`, `click`, `dwell`, `search`, `rate`, `review`, `purchase`, `bookmark`, `share`, `ignore`, `dismiss`.
- `edge_type`: `dependency`, `hierarchy`, `relationship`.
- `listing status`: `open`, `closed`, `filled`, `suspended`.
- `bid status`: `active`, `withdrawn`, `accepted`, `rejected`, `expired`.
- `game mechanism`: `assignment`, `first-price`, `second-price`, `reverse`, `reverse-auction`.
- `policy decision`: `allow`, `deny`, `require_approval`.
- `approval status`: `pending`, `approved`, `rejected`, `expired`.

## Command Set
CLI actions are lowercase with hyphens. gRPC methods are CamelCase versions of the action name.

Examples:
- `rec-upsert-item` -> `RecUpsertItem`
- `graph-rtraverse` -> `GraphRtraverse`

**Core Control + Telemetry**
- `control` -> `Control`: engine lifecycle (`mode`/`action`/`value`: `start|pause|stop`).
- `status` -> `Status`: engine status.
- `ingest` -> `Ingest`: gateway ingest (`topic`, `payload`, `source`, `target`, `flow_id`, `timestamp_ms`).
- `ingest-envelope` -> `IngestEnvelope`: canonical envelope ingest (`id`, `flow_id`, `topic`, `payload`, `source`, `target`, `timestamp_ms`).
- `snapshot` -> `Snapshot`: host snapshot (`host_id`, `window_ms`).
- `flow-ledger` -> `FlowLedger`: flow ledger (`limit`).
- `flow-envelopes` -> `FlowEnvelopes`: envelope list (`limit`).

**Analytics**
- `analytics-module-catalog` -> `AnalyticsModuleCatalog`: list known modules.
- `analytics-ingest-event` -> `AnalyticsIngestEvent`: ingest one `StreamEvent`.
- `analytics-ingest-batch` -> `AnalyticsIngestBatch`: ingest list of `events`.
- `analytics-ingest-module-metric` -> `AnalyticsIngestModuleMetric`: ingest `ModuleMetricSample`.
- `analytics-ingest-host-metric` -> `AnalyticsIngestHostMetric`: ingest `HostMetricSample`.
- `analytics-ingest-realtime` -> `AnalyticsIngestRealtime`: combine `event`, `module_metric`, `host_metric`.
- `analytics-ingest-bus` -> `AnalyticsIngestBus`: bus event (`topic`, `payload`, `profile_id`, `host_id`, `timestamp_ms`, `window_ms`).
- `analytics-snapshot-profile` -> `AnalyticsSnapshotProfile`: profile snapshot (`profile_id`, `window`).
- `analytics-module-activity` -> `AnalyticsModuleActivity`: activity window (`window`).
- `analytics-module-snapshot` -> `AnalyticsModuleSnapshot`: module snapshot (`module`, `window_ms`).
- `analytics-host-snapshot` -> `AnalyticsHostSnapshot`: host snapshot (`host_id`, `window_ms`).
- `analytics-system-snapshot` -> `AnalyticsSystemSnapshot`: system snapshot (`host_id`, `modules`, `window_ms`).

**Stream Engine**
- `stream-ingest` -> `StreamIngest`: ingest `StreamEvent`.
- `stream-ingest-batch` -> `StreamIngestBatch`: ingest list of `events`.
- `stream-recent` -> `StreamRecent`: latest events (`limit`).
- `stream-all` -> `StreamAll`: full stream.
- `stream-by-profile` -> `StreamByProfile`: filter by `profile_id` and `limit`.
- `stream-by-module` -> `StreamByModule`: filter by `module` and `limit`.
- `stream-since` -> `StreamSince`: events since `since_ms`.
- `stream-by-module-since` -> `StreamByModuleSince`: events by module since `since_ms`.
- `stream-size` -> `StreamSize`: stream size.

**Recommendation Engine**
- `rec-upsert-item` -> `RecUpsertItem`: register `ItemProfile`.
- `rec-upsert-items` -> `RecUpsertItems`: batch register `items`.
- `rec-upsert-profile` -> `RecUpsertProfile`: register `UserProfile`.
- `rec-record-interaction` -> `RecRecordInteraction`: record `UserInteraction`.
- `rec-record-interactions` -> `RecRecordInteractions`: batch record `interactions`.
- `rec-record-rating` -> `RecRecordRating`: record rating (`user_id`, `item_id`, `rating`, `context`).
- `rec-record-review` -> `RecRecordReview`: record review (`user_id`, `item_id`, `sentiment`, `context`).
- `rec-record-dwell` -> `RecRecordDwell`: record dwell (`user_id`, `item_id`, `dwell_seconds`, `context`).
- `rec-record-search` -> `RecRecordSearch`: record search (`user_id`, `query`, `context`).
- `rec-build-persona` -> `RecBuildPersona`: build persona (`user_id`).
- `rec-profile` -> `RecProfile`: fetch profile (`user_id`).
- `rec-personas` -> `RecPersonas`: list persona labels.
- `rec-hybrid` -> `RecHybrid`: hybrid recommendations (`user_id`, `context`, `exclude`, `limit`).
- `rec-collab` -> `RecCollab`: collaborative recs (`user_id`, `exclude`, `limit`).
- `rec-content` -> `RecContent`: content-based recs (`user_id`, `exclude`, `limit`).
- `rec-contextual` -> `RecContextual`: contextual recs (`user_id`, `context`, `limit`).
- `rec-cold-start` -> `RecColdStart`: cold-start recs (`user_id`, `context`, `limit`).
- `rec-feedback` -> `RecFeedback`: feedback history (`user_id`, `limit`).
- `rec-profile-summary` -> `RecProfileSummary`: profile summary (`user_id`).
- `rec-personalized-search` -> `RecPersonalizedSearch`: personalized search (`user_id`, `SearchQuery`).
- `rec-stats` -> `RecStats`: catalog stats.

**Personalization Engine**
CLI also accepts `personalization` as an alias for `personalize`.
- `personalize` -> `Personalize`: personalize request (`PersonalizationRequest`).
- `personalization-set-preference` -> `PersonalizationSetPreference`: set preference (`profile_id`, `key`, `value`).
- `personalization-set-preferences` -> `PersonalizationSetPreferences`: set preferences (`profile_id`, `preferences`).
- `personalization-clear-preference` -> `PersonalizationClearPreference`: clear preference (`profile_id`, `key`).
- `personalization-preferences` -> `PersonalizationPreferences`: explicit preferences (`profile_id`).
- `personalization-resolve-preferences` -> `PersonalizationResolvePreferences`: resolve preferences (`profile_id`, `context`, `persona`).
- `personalization-delivery-config` -> `PersonalizationDeliveryConfig`: delivery config (`profile_id`, `context`, `persona`).
- `personalization-rank-content` -> `PersonalizationRankContent`: rank items (`profile_id`, `items`, `context`).
- `personalization-register-segment` -> `PersonalizationRegisterSegment`: register `Segment`.
- `personalization-register-segments` -> `PersonalizationRegisterSegments`: batch register `segments`.
- `personalization-assign-segments` -> `PersonalizationAssignSegments`: assign segments (`profile_id`, `attributes`, `persona`).
- `personalization-segment-assignment` -> `PersonalizationSegmentAssignment`: fetch assignment (`profile_id`).
- `personalization-profiles-in-segment` -> `PersonalizationProfilesInSegment`: list profiles (`segment_id`).
- `personalization-build-persona` -> `PersonalizationBuildPersona`: build persona (`profile_id`).
- `personalization-detect-drift` -> `PersonalizationDetectDrift`: drift detection (`profile_id`).
- `personalization-apply-drift` -> `PersonalizationApplyDrift`: apply drift (`profile_id`).
- `personalization-register-experiment` -> `PersonalizationRegisterExperiment`: register `Experiment`.
- `personalization-assign-experiment` -> `PersonalizationAssignExperiment`: assign experiment (`profile_id`, `Experiment`).
- `personalization-record-exposure` -> `PersonalizationRecordExposure`: record exposure (`experiment_id`, `variant_id`, `profile_id`, `converted`).
- `personalization-experiment-stats` -> `PersonalizationExperimentStats`: stats (`experiment_id`).
- `personalization-experiment-assignments` -> `PersonalizationExperimentAssignments`: assignments (`profile_id`).
- `personalization-register-bandit` -> `PersonalizationRegisterBandit`: register bandit (`slot_id`, `arms`).
- `personalization-select-bandit` -> `PersonalizationSelectBandit`: select arm (`slot_id`).
- `personalization-record-bandit` -> `PersonalizationRecordBandit`: record reward (`slot_id`, `arm_id`, `reward`).
- `personalization-bandit-stats` -> `PersonalizationBanditStats`: stats (`slot_id`, `total_pulls`).
- `personalization-adapt-content` -> `PersonalizationAdaptContent`: adapt content (`profile_id`, `items`, `context`, `item_meta`).
- `personalization-record-interaction` -> `PersonalizationRecordInteraction`: record `UserInteraction`.
- `personalization-record-interactions` -> `PersonalizationRecordInteractions`: batch record `interactions`.
- `personalization-register-item` -> `PersonalizationRegisterItem`: register `ItemProfile`.
- `personalization-recommendations` -> `PersonalizationRecommendations`: recs (`profile_id`, `context`, `limit`).

**Search + Query**
- `search` -> `Search`: search by `SearchQuery`.
- `index` -> `Index`: index `SearchDocument`.
- `search-index-batch` -> `SearchIndexBatch`: batch index `documents`.
- `search-remove` -> `SearchRemove`: remove by `id`.
- `search-filter` -> `SearchFilter`: filter by `SearchQuery`.
- `search-personalized` -> `SearchPersonalized`: personalized search (`user_id`, `SearchQuery`, optional `preferences`).
- `query` -> `Query`: analyze + optimize (`sql`).
- `query-analyze` -> `QueryAnalyze`: analysis only (`sql`).
- `query-optimize` -> `QueryOptimize`: optimize only (`sql`).
- `query-optimize-profile` -> `QueryOptimizeProfile`: optimize with `QueryProfile` (`sql`, `profile`).

**Risk + Optimization**
- `risk-score` -> `RiskScore`: score `PortfolioSignal`.
- `risk-score-persona` -> `RiskScorePersona`: score `PortfolioSignal` with `persona`.
- `risk-optimize` -> `RiskOptimize`: optimize risk (`RiskOptimizationRequest`).
- `risk-manage` -> `RiskManage`: manage risk (`profile_id`, `PortfolioSignal`).
- `optimize` -> `Optimize`: optimize workload (`OptimizationRequest`).

**Policy**
- `policy-register-simple` -> `PolicyRegisterSimple`: register simple rule (`decision`, optional match fields).
- `policy-unregister` -> `PolicyUnregister`: remove rule (`id`).
- `policy-list` -> `PolicyList`: list rules.
- `policy-clear` -> `PolicyClear`: clear rules.
- `policy-evaluate` -> `PolicyEvaluate`: evaluate `PolicyContext`.
- `policy-evaluate-report` -> `PolicyEvaluateReport`: evaluation report.
- `policy-request-approval` -> `PolicyRequestApproval`: request approval (`component_id`, `requested_by`, `reason`, `metadata`).
- `policy-resolve-approval` -> `PolicyResolveApproval`: resolve (`request_id`, `approved`, `resolver`, `notes`).
- `policy-approval` -> `PolicyApproval`: fetch request (`request_id`).
- `policy-approvals` -> `PolicyApprovals`: list requests (`status`).

**Match Engine**
- `match-register-user` -> `MatchRegisterUser`: register `UserSubject`.
- `match-register-users` -> `MatchRegisterUsers`: register list of `users`.
- `match-register-component` -> `MatchRegisterComponent`: register `ComponentSubject`.
- `match-register-components` -> `MatchRegisterComponents`: register list of `components`.
- `match-register-resource` -> `MatchRegisterResource`: register `ResourceSubject`.
- `match-register-resources` -> `MatchRegisterResources`: register list of `resources`.
- `match-register-asset` -> `MatchRegisterAsset`: register `AssetSubject`.
- `match-register-assets` -> `MatchRegisterAssets`: register list of `assets`.
- `match-counts` -> `MatchCounts`: registry counts.
- `match-user-to-components` -> `MatchUserToComponents`: match (`user`, `components`, `weights`, `limit`).
- `match-component-to-users` -> `MatchComponentToUsers`: match (`component`, `role`, `users`, `weights`, `limit`).
- `match-assets-to-component` -> `MatchAssetsToComponent`: match (`component`, `assets`, `weights`, `limit`).
- `match-components-to-asset` -> `MatchComponentsToAsset`: match (`asset`, `components`, `weights`, `limit`).
- `match-resources-to-component` -> `MatchResourcesToComponent`: match (`component`, `resources`, `weights`, `limit`).
- `match-components-to-resource` -> `MatchComponentsToResource`: match (`resource`, `components`, `weights`, `limit`).
- `match-profiles-resources` -> `MatchProfilesResources`: match (`profiles`, `resources`).
- `match-talent` -> `MatchTalent`: talent match (`requester`, `users`, `weights`, `limit`).
- `match-by-persona` -> `MatchByPersona`: persona match (`persona`, `users`, `limit`).
- `match-similar-users` -> `MatchSimilarUsers`: similarity (`user`, `users`, `weights`, `limit`).
- `match-workloads-to-plans` -> `MatchWorkloadsToPlans`: workload plans (`requests`, `limit`).
- `match-artifacts-to-user` -> `MatchArtifactsToUser`: artifacts to user (`user`, `artifacts`, `weights`, `limit`).
- `match-component-bundle` -> `MatchComponentBundle`: bundle (`component`, `role`, `users`, `resources`, `assets`, `limit`).
- `match-exchange-listings` -> `MatchExchangeListings`: listings to buyers (`listings`, `buyers`, `limit`).

**Allocation Engine**
- `allocation-score` -> `AllocationScore`: score `AllocationRequest`.
- `allocation-allocate` -> `AllocationAllocate`: allocate `AllocationRequest`.

**Incentive Engine**
- `incentive-register-participant` -> `IncentiveRegisterParticipant`: register (`participant_id`, `reputation`, `kp`).
- `incentive-profile` -> `IncentiveProfile`: profile (`participant_id`).
- `incentive-balance` -> `IncentiveBalance`: balance (`participant_id`).
- `incentive-apply-event` -> `IncentiveApplyEvent`: apply `IncentiveEvent`.
- `incentive-apply-incentive` -> `IncentiveApplyIncentive`: apply `GameIncentive` (`reference_id`, `reference_type`).
- `incentive-apply-incentives` -> `IncentiveApplyIncentives`: apply list of `incentives`.
- `incentive-earn` -> `IncentiveEarn`: earn KP (`participant_id`, `kp`, `reason`, `reference_id`, `reference_type`).
- `incentive-penalize` -> `IncentivePenalize`: penalize (`participant_id`, `kp`, `reputation`, `reason`).
- `incentive-redeem` -> `IncentiveRedeem`: redeem (`participant_id`, `cost`, `reason`).
- `incentive-incentives-for-allocation` -> `IncentiveIncentivesForAllocation`: incentives for allocation (`listing`, `allocation`, `policy`).
- `incentive-ledger` -> `IncentiveLedger`: ledger entries (`limit`).

**Game Engine**
- `game-register-participant` -> `GameRegisterParticipant`: register `UserSubject` (`reputation`, `kp`).
- `game-register-participants` -> `GameRegisterParticipants`: batch register `users`.
- `game-participant` -> `GameParticipant`: lookup by `id`.
- `game-participants` -> `GameParticipants`: snapshot participants.
- `game-upsert-listing` -> `GameUpsertListing`: upsert `GameListing`.
- `game-listing` -> `GameListing`: fetch by `id`.
- `game-close-listing` -> `GameCloseListing`: close listing (`id`).
- `game-suspend-listing` -> `GameSuspendListing`: suspend listing (`id`).
- `game-listings` -> `GameListings`: list by `status`.
- `game-submit-bid` -> `GameSubmitBid`: submit `GameBid`.
- `game-withdraw-bid` -> `GameWithdrawBid`: withdraw by `id`.
- `game-bids` -> `GameBids`: list bids (`listing_id`, `include_inactive`).
- `game-match-listing` -> `GameMatchListing`: match listing (`listing_id`, `limit`, `match_weights`).
- `game-score-bids` -> `GameScoreBids`: score bids (`listing_id`, `score_weights`, `match_weights`).
- `game-allocate-listing` -> `GameAllocateListing`: allocate listing (`listing_id`, `limit`, `score_weights`, `match_weights`, `policy`, `apply_incentives`).
- `game-award-incentives` -> `GameAwardIncentives`: incentives for allocation (`listing`, `allocation`, `policy`).
- `game-snapshot` -> `GameSnapshot`: snapshot.
- `game-events` -> `GameEvents`: event ledger (`limit`).

**Graph Engine**
- `graph-add-edge` -> `GraphAddEdge`: add edge (`from`, `to`, `edge_type`, `weight`).
- `graph-add-node` -> `GraphAddNode`: add node (`id`, `duration`).
- `graph-remove-edge` -> `GraphRemoveEdge`: remove edge (`from`, `to`).
- `graph-remove-node` -> `GraphRemoveNode`: remove node (`id`).
- `graph-load` -> `GraphLoad`: load `edges` and `nodes`.
- `graph-traverse` -> `GraphTraverse`: traverse (`node`, `edge_type` or `any`).
- `graph-rtraverse` -> `GraphRtraverse`: reverse traverse (`node`, `edge_type` or `any`).
- `graph-impact` -> `GraphImpact`: impact report (`node`).
- `graph-closure` -> `GraphClosure`: dependency closure (`node`).
- `graph-rdeps` -> `GraphRdeps`: reverse closure (`node`).
- `graph-neighbors` -> `GraphNeighbors`: neighbors (`node`).
- `graph-ancestors` -> `GraphAncestors`: ancestors (`node`).
- `graph-descendants` -> `GraphDescendants`: descendants (`node`).
- `graph-cycles` -> `GraphCycles`: cycle report.
- `graph-topo` -> `GraphTopo`: topological sort.
- `graph-critical` -> `GraphCritical`: critical path.
- `graph-diff` -> `GraphDiff`: diff vs `before_edges`.
- `graph-nodes` -> `GraphNodes`: list nodes.
- `graph-edges` -> `GraphEdges`: list edges.

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
Reflection is enabled by default, so you can use `grpcurl` directly. The full command set
matches `grpcurl-commands.ps1` and the list in the Command Set section.

```bash
grpcurl -plaintext localhost:9100 list
grpcurl -plaintext localhost:9100 describe kogi.engine.v1.EngineService
grpcurl -plaintext -d '{}' localhost:9100 kogi.engine.v1.EngineService/Status
grpcurl -plaintext -d '{"action":"start"}' localhost:9100 kogi.engine.v1.EngineService/Control
grpcurl -plaintext -d '{"topic":"engine.ingest","source":"kogi.network.engine","target":"kogi.engine","payload":{"event_type":"demo"}}' localhost:9100 kogi.engine.v1.EngineService/Ingest
grpcurl -plaintext -d '{"host_id":"kogi-host-001","window_ms":300000}' localhost:9100 kogi.engine.v1.EngineService/Snapshot
```

You can also run `.\\grpcurl-commands.ps1` to print and execute the same list.

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

