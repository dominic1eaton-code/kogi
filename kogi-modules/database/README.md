# Kogi Database Module

- Module ID: `kogi.database`
- Runtime language: `hybrid-rust-go`
- Host entrypoint: `kogi-services/go/services/database`
- Rust system crate: `kogi-modules/database`
- Backing database: `kogi-database` (PostgreSQL network + SQLite local)

## Role
`DatabaseSystem` manages CRUD, data queries, access control, concurrency, snapshots, checkpoints, backups, restores, scaling, optimization, storage, and data management for the platform database.

## Interfaces
- Rust binary: `kogi-database-system` (JSON in/out)
- Go service: `kogi-services/go/services/database` invokes the Rust system for all database operations.

## Environment
- `KOGI_DATABASE_MODE` / `KOGI_DATABASE_TARGET` (`local` or `network`)
- `KOGI_DATABASE_DSN` (or `DATABASE_URL`)
- `KOGI_DATABASE_NAME`
- `KOGI_DATABASE_SCHEMA` (fallback for Postgres)
- `KOGI_DATABASE_POSTGRES_SCHEMA`
- `KOGI_DATABASE_SQLITE_SCHEMA`
- `KOGI_DATABASE_SQLITE_PATH`
- `KOGI_DATABASE_STATE_PATH`
- `KOGI_DATABASE_STATE_PATH_LOCAL`
- `KOGI_DATABASE_STATE_PATH_NETWORK`
- `KOGI_DATABASE_STORAGE_ROOT`
- `KOGI_DATABASE_MAX_CONNECTIONS`
- `KOGI_DATABASE_SNAPSHOT_RETENTION`
- `KOGI_DATABASE_BACKUP_RETENTION`

## Storage Targeting
Per-request overrides are accepted via `options.storage` (or `options.storage_mode`, `options.storage_target`, `options.engine`)
with values `local`/`sqlite` or `network`/`postgres`. If not provided, the system uses `KOGI_DATABASE_MODE`.

## Run (Rust System)
```powershell
cargo run --manifest-path kogi-modules/database/Cargo.toml -- --request "{\"action\":\"status\"}"
```

## Service Endpoints
- `POST /api/v1/database/query`
- `POST /api/v1/database/records/create`
- `POST /api/v1/database/records/read`
- `POST /api/v1/database/records/update`
- `POST /api/v1/database/records/delete`
- `POST /api/v1/database/snapshot`
- `POST /api/v1/database/checkpoint`
- `POST /api/v1/database/backup`
- `POST /api/v1/database/restore`
- `POST /api/v1/database/scale`
- `POST /api/v1/database/optimize`
- `POST /api/v1/database/access`
- `POST /api/v1/database/concurrency`
- `GET /api/v1/database/storage`
- `POST /api/v1/database/operation`
