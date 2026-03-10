# kogi-host

Rust host runtime for Kogi.

## Role
- Acts as central coordinator/orchestrator and executive control system
- Loads module manifests from `kogi-modules/*/module.yaml`
- Registers platform components (kernel/host/server/engine/services/modules) with the kernel bridge
- Emits host lifecycle events
- Maintains runtime activation state
- Reads per-module isolation limits (memory/process/file/resource)
- Coordinates module network management metadata
- Publishes module capabilities/integrations metadata and office view registration events
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
