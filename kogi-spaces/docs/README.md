# Kogi Spaces

Kogi Spaces is the community and organizational backend for spaces, rooms, channels, events, feeds, and network linking. The service derives its source-of-truth data from the `kogi-portfolio` master spreadsheet and exposes a Go microservice facade for the UI.

## Contents
- `spaces-backend.md` - backend architecture, data model, and endpoint map

## Quick Start
1. Build the Rust library (DLL) or the CLI fallback:
   - `cargo build -p kogi-spaces`
2. Run the Go microservice:
   - `go run .` from `kogi-spaces/service`

## Environment
- `KOGI_SPACES_DLL` - path to `kogi_spaces.dll` (or `.so`/`.dylib`)
- `KOGI_SPACES_SYSTEM_BIN` - path to `kogi-spaces-system` CLI fallback
- `KOGI_SPACES_PORT` - HTTP port for the Go service (default `9014`)

## Data Source
All spaces, rooms, channels, events, and workspaces are derived from the `kogi-portfolio` master spreadsheet. Use portfolio item `item_type`, `container_type`, tags, or metadata fields (`space`, `spaces`, `kogi_space`) to drive space classification.
