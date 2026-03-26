# Kogi Office

Kogi Office is the operational backend for office workspaces, work management, analytics, governance, and studio workflows. The service derives its source-of-truth data from the `kogi-portfolio` master spreadsheet and exposes a Go microservice facade for the UI.

## Contents
- `office-backend.md` - backend architecture, data model, and endpoint map

## Quick Start
1. Build the Rust library (DLL) or the CLI fallback:
   - `cargo build -p kogi-office`
2. Run the Go microservice:
   - `go run .` from `kogi-office/service`

## Environment
- `KOGI_OFFICE_DLL` - path to `kogi_office.dll` (or `.so`/`.dylib`)
- `KOGI_OFFICE_SYSTEM_BIN` - path to `kogi-office-system` CLI fallback
- `KOGI_OFFICE_PORT` - HTTP port for the Go service (default `9015`)

## Data Source
All office work management data is derived from the `kogi-portfolio` master spreadsheet. Use portfolio `item_type`, `container_type`, tags, topics, and metadata fields (for example `story_type`, `description`, and `attachments`) to classify office work items.
