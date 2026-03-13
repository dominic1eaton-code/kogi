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

Write-Host "# Control (pause)"
Invoke-Grpcurl -Method "Control" -Json '{"action":"pause"}'
Write-Host ""

Write-Host "# Control (stop)"
Invoke-Grpcurl -Method "Control" -Json '{"action":"stop"}'
