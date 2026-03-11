# kogi-server

Rust backend platform server that boots `kogi-host` via `HostApp` and exposes the host system to clients.
The server bridges client requests into the host and publishes/subscribes messages through the gateway.

## Endpoints
- `GET /health`
- `GET /api/v1/system`
- `GET /api/v1/host`
- `GET /api/v1/host/components`
- `GET /api/v1/engine/system`
- `GET /api/v1/engine/runtime`
- `POST /api/v1/engine/control`
- `POST /api/v1/engine/ingest`
- `GET /api/v1/modules`
- `POST /api/v1/messages`
- `GET /api/v1/messages`
- `GET /api/v1/ims/identities`
- `GET /api/v1/ims/profiles`
- `GET /api/v1/autonomy/capabilities`
- `GET /api/v1/kernel/modules/isolation`
- `GET /api/v1/database/runtime`
- `POST /api/v1/database/query`
- `GET /api/v1/office`
- `GET /api/v1/office/dashboard`
- `GET /api/v1/office/portfolio`
- `GET /api/v1/office/timeline`
- `GET /api/v1/office/workspace`
- `GET /api/v1/office/assistant`
- `GET /api/v1/screens/unified`
- `GET /api/v1/screens/unified/flat`

## Run
```powershell
cargo run --manifest-path kogi-server/Cargo.toml
```

## Run (Bazel)
```powershell
bazel run //:server_run
```

## Run Modes
The server supports:
- `--silent` or `KOGI_SILENT=1` to suppress stdout/stderr (background-friendly).
- `--debug` or `KOGI_DEBUG=1` to emit structured state/status/message logs.

Examples:
```powershell
cargo run --manifest-path kogi-server/Cargo.toml -- --silent
cargo run --manifest-path kogi-server/Cargo.toml -- --debug
$env:KOGI_DEBUG = "1"; cargo run --manifest-path kogi-server/Cargo.toml
```

## Background Manager
Use the repo helper to run server + services in the background:
```powershell
.\tools\build\kogi_daemon.ps1 -Action start -Target server
.\tools\build\kogi_daemon.ps1 -Action status -Target server
.\tools\build\kogi_daemon.ps1 -Action stop -Target server
```
