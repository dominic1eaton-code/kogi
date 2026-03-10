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

Set `KOGI_ENGINE_CLI` to a compiled `KogiEngineCli` binary/script, or ensure `sbt` is on `PATH` so the engine service can call the Scala CLI directly.
Set `KOGI_ENGINE_ENDPOINT` only if you also expose a Scala HTTP endpoint (legacy path).

Engine service endpoints include:
- `POST /api/v1/engine/control`
- `POST /api/v1/engine/ingest`
- `GET /api/v1/engine/snapshot`
- `GET|POST /api/v1/engine/messages` (gateway relay)

## Libraries
- `lib/eventbus`: in-memory pub/sub bus with history, topic stats, wildcard subscriptions, and source/target metadata.
- `lib/mesh`: component registry + topic routes + component-to-component message routing history.

## Gateway API additions
- `POST /api/v1/gateway/components/register`
- `GET /api/v1/gateway/components`
- `POST /api/v1/gateway/pubsub/publish`
- `GET /api/v1/gateway/pubsub/history`
- `GET /api/v1/gateway/pubsub/topics`
- `POST /api/v1/gateway/network/send`
- `GET /api/v1/gateway/network/history`
- `GET /api/v1/gateway/network/routes`
- `GET /api/v1/gateway/engine/stream`

## Build
```powershell
go build ./kogi-services/go/...
```

## Run examples
```powershell
go run ./kogi-services/go/gateway
go run ./kogi-services/go/services/auth
go run ./kogi-services/go/services/office
go run ./kogi-services/go/services/engine
go run ./kogi-services/go/services/database
```

Module services (bank, marketplace, studio, community, developer, profile, organizations) run the same way.

## Message Gateway
All services can publish and subscribe via the gateway (`http://127.0.0.1:8090`). The server bridges client messages into the gateway and host.

## Database System Integration
The database service expects a `kogi-database-system` binary. Build and point to it if needed:

```powershell
cargo build --manifest-path kogi-modules/database/Cargo.toml
$env:KOGI_DATABASE_SYSTEM_BIN = \"C:\\path\\to\\kogi-database-system.exe\"
```

## Office System Integration
The office service can invoke the Rust `kogi-office-system` binary when available:

```powershell
cargo build --manifest-path kogi-modules/office/Cargo.toml
$env:KOGI_OFFICE_SYSTEM_BIN = \"C:\\path\\to\\kogi-office-system.exe\"
```
