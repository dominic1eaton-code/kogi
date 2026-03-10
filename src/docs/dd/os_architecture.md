# Kogi OS Architecture (Refactor Snapshot)

## Runtime Layers

- Kernel layer (`kogi-kernel`, Zig): orchestrator, scheduler, registry, RBAC, memory/process/file management.
- Host layer (`kogi-host`, Rust): executive control and module runtime supervision.
- Service layer (`kogi-server` + `kogi-services/go`): backend APIs, routing, module services.
- Data/AI layer (`kogi-database` + `kogi-analytics-scala`): PostgreSQL storage and analytics pipelines.
- Client layer (`kogi-desktop-client`, `kogi-web-client`, `kogi-mobile`): desktop/web/mobile entrypoints.

## Security Boundary

- Kernel mode and user mode are explicitly represented in the Zig kernel runtime.
- Access enforcement is role-based (`root`, `host`, `server`, `module_runtime`, `user`) with permission checks per privileged operation.

## Integration Model

- Module registration through host->kernel bridge.
- Event-driven integration topics defined in `kogi-contracts/events/topics.md`.
- API contracts defined in `kogi-contracts/openapi/kogi-server.yaml`.
- Financial correctness constraint: immutable ledger entries in PostgreSQL via DB triggers.
- IMS API surfaces support multi-profile identities and autonomy abstractions.
- Module resource isolation limits are defined per service and managed by host/kernel.
