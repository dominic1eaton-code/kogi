# kogi-database

PostgreSQL and SQLite schemas for Kogi MVP. Desktop clients use the SQLite database for local storage.

## Apply schema (PostgreSQL)
```sql
\i kogi-database/postgres/schema.sql
\i kogi-database/postgres/seed.sql
```

## Apply schema (SQLite - local/desktop)
```powershell
sqlite3 kogi-database/sqlite/kogi.db < kogi-database/sqlite/schema.sql
sqlite3 kogi-database/sqlite/kogi.db < kogi-database/sqlite/seed.sql
```

## Database Service
The host reaches PostgreSQL via the database service facade:

```powershell
go run ./kogi-network/services/database
```

The database service delegates CRUD/query/snapshot/backup logic to the Rust `DatabaseSystem` module in `kogi-modules/database`.
`DatabaseSystem` routes local requests to SQLite and network requests to PostgreSQL.
Build it and set the binary path if needed:

```powershell
cargo build --manifest-path kogi-modules/database/Cargo.toml
$env:KOGI_DATABASE_SYSTEM_BIN = \"C:\\path\\to\\kogi-database-system.exe\"
```

## Included domains
- auth and sessions
- IMS identities, personas, roles, worker types, and multi-profile configuration
- profile-scoped connection registry, contact directory, and asset vault
- kernel registry/events/scheduler
- portfolio and resources
- WBS/stories/sprints
- market orders and reviews
- exchange wallet, immutable ledger, escrow
- community spaces/messages
- work automation and OKRs
- crowdfunding/capital pools
- autonomous organizations and governance log
- AI agent session and action audit
