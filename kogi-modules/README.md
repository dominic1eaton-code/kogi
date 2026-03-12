# kogi-modules

Module manifests and operating scopes consumed by `kogi-host`.

## Modules
- kogi.office
- kogi.bank
- kogi.exchange
- kogi.marketplace
- kogi.studio
- kogi.community
- kogi.developer
- kogi.profile
- kogi.organizations
- kogi.database

Each module defines `module.yaml` metadata for host provisioning and registry enrollment.
Each manifest also declares per-module isolation limits for memory, processes, files, and resource units managed by kernel/host orchestration.
Each module entrypoint maps to a Go service under `kogi-network/services/<module>` (office/exchange/etc.).
`kogi.database` is a hybrid module: the Go database service calls the Rust `DatabaseSystem` module for core operations.
