# kogi-host

Rust host runtime for Kogi.

## Role
- `HostApp` is the primary host application with lifecycle modes (init/configure/run/pause/shutdown).
- `HostModel` is the execution model that processes service messages and connects them to `HostSystem`.
- `HostSystem` is the core computational interface where the kernel bridge, runtime, module runtime, executive, and shell are orchestrated.
- `HostRuntime` provides runtime boot/tick/engine control/ingest and delegates to the executive.
- `HostExecutive` manages kernel coordination, module registry, services, and component health.
- Loads module manifests from `kogi-modules/*/module.yaml`
- Registers platform components (kernel/host/server/engine/engine-service/database-service/services/modules) with the kernel bridge
- Emits host lifecycle events
- Maintains runtime activation state
- Reads per-module isolation limits (memory/process/file/resource)
- Coordinates module network management metadata
- Publishes module capabilities/integrations metadata and office view registration events
- Uses kogi-services (gateway + module services + engine/database services) to execute service functions and engine ingest/control
- Provides an interactive host shell/CLI for checking modules/services/server/engine through host

## Interactive Shell
Run without `--once` to launch the shell after boot.

Commands:
- `help`
- `status`
- `boot`
- `tick`
- `modules`
- `components`
- `check [component_filter]`
- `show <component_id>`
- `publish <topic> <payload_json>`
- `exit`

## Run
```powershell
cargo run --manifest-path kogi-host/Cargo.toml

# one-shot boot + tick (no shell)
cargo run --manifest-path kogi-host/Cargo.toml -- --once
```

## Run (Bazel)
```powershell
bazel run //:host_run
```

Run the gateway and Go services (module services + engine/database services) to enable health checks and engine ingest/control.
