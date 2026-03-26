# Kogi Hub

Kogi Hub is the governance and community coordination backend for teams, collectives, cooperatives, federations, autonomous cells, contracts, rights, and IP management. The service derives its source-of-truth data from the `kogi-portfolio` master spreadsheet and exposes a Go microservice facade for the UI.

## Contents
- `hub-backend.md` - backend architecture, data model, and endpoint map

## Quick Start
1. Build the Rust library (DLL) or the CLI fallback:
   - `cargo build -p kogi-hub`
2. Run the Go microservice:
   - `go run .` from `kogi-hub/service`

## Environment
- `KOGI_HUB_DLL` - path to `kogi_hub.dll` (or `.so`/`.dylib`)
- `KOGI_HUB_SYSTEM_BIN` - path to `kogi-hub-system` CLI fallback
- `KOGI_HUB_PORT` - HTTP port for the Go service (default `9016`)

## Data Source
All hub entities, governance records, contracts, and IP assets are derived from the `kogi-portfolio` master spreadsheet. Use portfolio `item_type`, `container_type`, tags, topics, and metadata fields (for example `governance_model`, `rights`, `counterparty`) to drive hub classification.
