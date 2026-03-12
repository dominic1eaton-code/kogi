# kogi-infra

Shared infrastructure contracts and topology references for the Kogi platform.

## Contents
- `openapi/kogi-server.yaml`: HTTP API contract for `kogi-server`.
- `proto/kogi_engine.proto`: gRPC contract for `kogi-engine` (used by the Go engine service).
- `events/topics.md`: Gateway pub/sub topic inventory.

## Topology (Defaults)
- `kogi-server` HTTP API: `http://127.0.0.1:8080`
- `kogi-network` gateway: `http://127.0.0.1:8090`
- `kogi-engine` gRPC: `0.0.0.0:9100`
- `kogi-host` + `kogi-kernel`: internal runtime booted by `kogi-server` or `kogi-host` CLI

## Data Flow
- Clients → `kogi-server` → gateway → Go services → Rust modules
- `kogi-server` → `kogi-host` → `kogi-kernel`
- Go engine service ↔ `kogi-engine` gRPC ↔ gateway pub/sub

## Notes
- The OpenAPI spec tracks current server endpoints, including office CRUD helpers.
- gRPC uses JSON-like `google.protobuf.Struct` payloads for flexible ingestion/control.
- Event topics are the shared language across the gateway and services.
