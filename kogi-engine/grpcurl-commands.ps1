<#
.SYNOPSIS
  grpcurl helper commands for kogi-engine EngineGrpcServer.

.DESCRIPTION
  Uses reflection by default. If reflection is disabled, pass -UseProto
  or set KOGI_ENGINE_GRPC_PROTO to a local proto path.

.EXAMPLE
  .\grpcurl-commands.ps1 -Addr 127.0.0.1:9100 -UseProto

.EXAMPLE
  $env:KOGI_ENGINE_GRPC_PROTO = "kogi-infra/proto/kogi_engine.proto"
  .\grpcurl-commands.ps1
#>

param(
  [string]$Addr = $env:KOGI_ENGINE_GRPC_ADDR,
  [string]$Host = "localhost",
  [int]$Port = 9100,
  [switch]$UseProto,
  [string]$ProtoPath = $env:KOGI_ENGINE_GRPC_PROTO
)

if ([string]::IsNullOrWhiteSpace($Addr)) {
  $Addr = "${Host}:${Port}"
}

if ([string]::IsNullOrWhiteSpace($ProtoPath)) {
  $ProtoPath = "kogi-infra/proto/kogi_engine.proto"
}

$Service = "kogi.engine.v1.EngineService"

$Methods = @(
  "Control",
  "Status",
  "Ingest",
  "IngestEnvelope",
  "Snapshot",
  "FlowLedger",
  "FlowEnvelopes",
  "AnalyticsModuleCatalog",
  "AnalyticsIngestEvent",
  "AnalyticsIngestBatch",
  "AnalyticsIngestModuleMetric",
  "AnalyticsIngestHostMetric",
  "AnalyticsIngestRealtime",
  "AnalyticsIngestBus",
  "AnalyticsSnapshotProfile",
  "AnalyticsModuleActivity",
  "AnalyticsModuleSnapshot",
  "AnalyticsHostSnapshot",
  "AnalyticsSystemSnapshot",
  "StreamIngest",
  "StreamIngestBatch",
  "StreamRecent",
  "StreamAll",
  "StreamByProfile",
  "StreamByModule",
  "StreamSince",
  "StreamByModuleSince",
  "StreamSize",
  "RecUpsertItem",
  "RecUpsertItems",
  "RecUpsertProfile",
  "RecRecordInteraction",
  "RecRecordInteractions",
  "RecRecordRating",
  "RecRecordReview",
  "RecRecordDwell",
  "RecRecordSearch",
  "RecBuildPersona",
  "RecProfile",
  "RecPersonas",
  "RecHybrid",
  "RecCollab",
  "RecContent",
  "RecContextual",
  "RecColdStart",
  "RecFeedback",
  "RecProfileSummary",
  "RecPersonalizedSearch",
  "RecStats",
  "Personalize",
  "PersonalizationSetPreference",
  "PersonalizationSetPreferences",
  "PersonalizationClearPreference",
  "PersonalizationPreferences",
  "PersonalizationResolvePreferences",
  "PersonalizationDeliveryConfig",
  "PersonalizationRankContent",
  "PersonalizationRegisterSegment",
  "PersonalizationRegisterSegments",
  "PersonalizationAssignSegments",
  "PersonalizationSegmentAssignment",
  "PersonalizationProfilesInSegment",
  "PersonalizationBuildPersona",
  "PersonalizationDetectDrift",
  "PersonalizationApplyDrift",
  "PersonalizationRegisterExperiment",
  "PersonalizationAssignExperiment",
  "PersonalizationRecordExposure",
  "PersonalizationExperimentStats",
  "PersonalizationExperimentAssignments",
  "PersonalizationRegisterBandit",
  "PersonalizationSelectBandit",
  "PersonalizationRecordBandit",
  "PersonalizationBanditStats",
  "PersonalizationAdaptContent",
  "PersonalizationRecordInteraction",
  "PersonalizationRecordInteractions",
  "PersonalizationRegisterItem",
  "PersonalizationRecommendations",
  "Search",
  "Index",
  "SearchIndexBatch",
  "SearchRemove",
  "SearchFilter",
  "SearchPersonalized",
  "Query",
  "QueryAnalyze",
  "QueryOptimize",
  "QueryOptimizeProfile",
  "RiskScore",
  "RiskScorePersona",
  "RiskOptimize",
  "RiskManage",
  "Optimize",
  "PolicyRegisterSimple",
  "PolicyUnregister",
  "PolicyList",
  "PolicyClear",
  "PolicyEvaluate",
  "PolicyEvaluateReport",
  "PolicyRequestApproval",
  "PolicyResolveApproval",
  "PolicyApproval",
  "PolicyApprovals",
  "MatchRegisterUser",
  "MatchRegisterUsers",
  "MatchRegisterComponent",
  "MatchRegisterComponents",
  "MatchRegisterResource",
  "MatchRegisterResources",
  "MatchRegisterAsset",
  "MatchRegisterAssets",
  "MatchCounts",
  "MatchUserToComponents",
  "MatchComponentToUsers",
  "MatchAssetsToComponent",
  "MatchComponentsToAsset",
  "MatchResourcesToComponent",
  "MatchComponentsToResource",
  "MatchProfilesResources",
  "MatchTalent",
  "MatchByPersona",
  "MatchSimilarUsers",
  "MatchWorkloadsToPlans",
  "MatchArtifactsToUser",
  "MatchComponentBundle",
  "MatchExchangeListings",
  "AllocationScore",
  "AllocationAllocate",
  "IncentiveRegisterParticipant",
  "IncentiveProfile",
  "IncentiveBalance",
  "IncentiveApplyEvent",
  "IncentiveApplyIncentive",
  "IncentiveApplyIncentives",
  "IncentiveEarn",
  "IncentivePenalize",
  "IncentiveRedeem",
  "IncentiveIncentivesForAllocation",
  "IncentiveLedger",
  "GameRegisterParticipant",
  "GameRegisterParticipants",
  "GameParticipant",
  "GameParticipants",
  "GameUpsertListing",
  "GameListing",
  "GameCloseListing",
  "GameSuspendListing",
  "GameListings",
  "GameSubmitBid",
  "GameWithdrawBid",
  "GameBids",
  "GameMatchListing",
  "GameScoreBids",
  "GameAllocateListing",
  "GameAwardIncentives",
  "GameSnapshot",
  "GameEvents",
  "GraphAddEdge",
  "GraphAddNode",
  "GraphRemoveEdge",
  "GraphRemoveNode",
  "GraphLoad",
  "GraphTraverse",
  "GraphRtraverse",
  "GraphImpact",
  "GraphClosure",
  "GraphRdeps",
  "GraphNeighbors",
  "GraphAncestors",
  "GraphDescendants",
  "GraphCycles",
  "GraphTopo",
  "GraphCritical",
  "GraphDiff",
  "GraphNodes",
  "GraphEdges"
)

function Invoke-Grpcurl {
  param(
    [string]$Method,
    [string]$Json = "{}"
  )

  if ($UseProto) {
    grpcurl -plaintext -import-path (Split-Path $ProtoPath) -proto (Split-Path $ProtoPath -Leaf) `
      -d $Json $Addr "$Service/$Method"
  } else {
    grpcurl -plaintext -d $Json $Addr "$Service/$Method"
  }
}

Write-Host "Target: $Addr"
Write-Host "Reflection: $([bool](-not $UseProto))"
Write-Host "Proto: $ProtoPath"
Write-Host ""

Write-Host "# List services"
if ($UseProto) {
  grpcurl -plaintext -import-path (Split-Path $ProtoPath) -proto (Split-Path $ProtoPath -Leaf) $Addr list
} else {
  grpcurl -plaintext $Addr list
}
Write-Host ""

Write-Host "# Describe service"
if ($UseProto) {
  grpcurl -plaintext -import-path (Split-Path $ProtoPath) -proto (Split-Path $ProtoPath -Leaf) $Addr describe $Service
} else {
  grpcurl -plaintext $Addr describe $Service
}
Write-Host ""

Write-Host "# Methods"
$Methods | ForEach-Object { Write-Host " - $_" }
Write-Host ""

Write-Host "# Status"
Invoke-Grpcurl -Method "Status" -Json "{}"
Write-Host ""

Write-Host "# Control (start)"
Invoke-Grpcurl -Method "Control" -Json '{"action":"start"}'
Write-Host ""

Write-Host "# Ingest"
Invoke-Grpcurl -Method "Ingest" -Json '{"topic":"engine.ingest","source":"kogi.network.engine","target":"kogi.engine","payload":{"event_type":"demo"}}'
Write-Host ""

Write-Host "# Snapshot"
Invoke-Grpcurl -Method "Snapshot" -Json '{"host_id":"kogi-host-001","window_ms":300000}'
Write-Host ""

Write-Host "# Analytics module catalog"
Invoke-Grpcurl -Method "AnalyticsModuleCatalog" -Json "{}"
Write-Host ""

Write-Host "# Stream size"
Invoke-Grpcurl -Method "StreamSize" -Json "{}"
Write-Host ""

Write-Host "# Rec stats"
Invoke-Grpcurl -Method "RecStats" -Json "{}"
Write-Host ""

Write-Host "# Graph nodes"
Invoke-Grpcurl -Method "GraphNodes" -Json "{}"
Write-Host ""

Write-Host "# Control (pause)"
Invoke-Grpcurl -Method "Control" -Json '{"action":"pause"}'
Write-Host ""

Write-Host "# Control (stop)"
Invoke-Grpcurl -Method "Control" -Json '{"action":"stop"}'
