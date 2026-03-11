# kogi-services-go

Go networking, pub/sub, message-gateway, and component communication infrastructure for Kogi MVP.
Each service is a network facade that can call its underlying system code (Rust module systems, Rust database system, Scala engine).

## Services
- `gateway` (port 8090): centralized message/event/data gateway management, pub/sub, routing, component registry, and engine ingest mirroring.
- `services/auth` (port 9001)
- `services/portfolio` (port 9002)
- `services/exchange` (port 9004)
- `services/ims` (port 9005)
- `services/office` (port 9006)
- `services/bank` (port 9007)
- `services/marketplace` (port 9008)
- `services/studio` (port 9009)
- `services/community` (port 9010)
- `services/developer` (port 9011)
- `services/profile` (port 9012)
- `services/organizations` (port 9013)
- `services/engine` (port 9014): control + ingest facade for the Scala engine (`KogiEngine` + subengines).
- `services/database` (port 9015): PostgreSQL interaction facade. Calls the Rust `DatabaseSystem` (`kogi-modules/database`) for CRUD/query/snapshot/backup operations.

## Build
```powershell
go build ./kogi-services/go/...
```

## Build (Bazel)
```powershell
bazel build //:services_build
```

## Run (foreground)
```powershell
go run ./kogi-services/go/gateway
go run ./kogi-services/go/services/auth
go run ./kogi-services/go/services/office
go run ./kogi-services/go/services/engine
go run ./kogi-services/go/services/database
```

Module services (bank, marketplace, studio, community, developer, profile, organizations) run the same way.

## Run (Bazel)
```powershell
bazel run //:gateway_run
bazel run //:engine_service_run -- --debug
bazel run //:office_run
```

## Run Modes
All Go services (including gateway) support:
- `--silent` or `KOGI_SILENT=1` to suppress stdout/stderr (background-friendly).
- `--debug` or `KOGI_DEBUG=1` to emit structured state/status/message logs.

Examples:
```powershell
go run ./kogi-services/go/gateway --silent
go run ./kogi-services/go/services/engine --debug
$env:KOGI_SILENT = "1"; go run ./kogi-services/go/services/office
.\tools\build\run_go_service.ps1 -Service gateway -Silent
```

## Background Process Manager
Use the helper to start/stop/cycle services as background processes:
```powershell
# start everything (gateway + services + kogi-server)
.\tools\build\kogi_daemon.ps1 -Action start

# cycle gateway + engine only
.\tools\build\kogi_daemon.ps1 -Action cycle -Target gateway,engine

# stop all Go services only
.\tools\build\kogi_daemon.ps1 -Action stop -Target services

# see status
.\tools\build\kogi_daemon.ps1 -Action status
```

## Logs
Background logs are stored in:
- `.cache\kogi\logs\<service>.out.log`
- `.cache\kogi\logs\<service>.err.log`

## Testing
```powershell
go test ./kogi-services/go/...
```

## Engine Service Commands (HTTP)
- `POST /api/v1/engine/control` `{ "action": "start|pause|stop" }`
- `POST /api/v1/engine/ingest` `{ ... }`
- `GET /api/v1/engine/snapshot`
- `GET|POST /api/v1/engine/messages` (gateway relay)

## Engine Service gRPC
The Go engine service can call the Scala engine over gRPC when enabled.

Environment variables:
- `KOGI_ENGINE_GRPC_ADDR` (default: none; set to enable). Example: `127.0.0.1:9100`
- `KOGI_ENGINE_GRPC_MODE` (`required` | `on` | `off`). `required` fails fast if gRPC is unavailable.
- `KOGI_ENGINE_GRPC_TIMEOUT` (optional, e.g. `2s`)

Run the Scala gRPC server:
```powershell
cd kogi-engine
sbt "runMain kogi.engine.EngineGrpcServer"
```

## Engine CLI Fallback
If gRPC is disabled or unavailable, the engine service falls back to the Scala CLI.
Set `KOGI_ENGINE_CLI` to a compiled `KogiEngineCli` binary/script, or ensure `sbt` is on `PATH` so the engine service can call the Scala CLI directly.
Set `KOGI_ENGINE_ENDPOINT` only if you also expose a Scala HTTP endpoint (legacy path).

## Gateway Commands (HTTP)
- `POST /api/v1/gateway/pubsub/publish`
- `GET /api/v1/gateway/pubsub/history`
- `POST /api/v1/gateway/components/register`
- `GET /api/v1/gateway/components`
- `POST /api/v1/gateway/network/send`
- `GET /api/v1/gateway/network/history`
- `GET /api/v1/gateway/engine/stream`

## Message Gateway
All services can publish and subscribe via the gateway (`http://127.0.0.1:8090`). The server bridges client messages into the gateway and host.

## Database System Integration
The database service expects a `kogi-database-system` binary. Build and point to it if needed:
```powershell
cargo build --manifest-path kogi-modules/database/Cargo.toml
$env:KOGI_DATABASE_SYSTEM_BIN = "C:\path\to\kogi-database-system.exe"
```

## Office System Integration
The office service can invoke the Rust `kogi-office-system` binary when available:
```powershell
cargo build --manifest-path kogi-modules/office/Cargo.toml
$env:KOGI_OFFICE_SYSTEM_BIN = "C:\path\to\kogi-office-system.exe"
```
