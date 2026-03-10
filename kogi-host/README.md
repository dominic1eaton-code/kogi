# kogi-host

Rust host runtime for Kogi.

## Role
- Acts as central executive control system
- Loads module manifests from `kogi-modules/*/module.yaml`
- Registers modules with the kernel bridge
- Emits host lifecycle events
- Maintains runtime activation state
- Reads per-module isolation limits (memory/process/file/resource)
- Coordinates module network management metadata
- Publishes module capabilities/integrations metadata and office view registration events

## Run
```powershell
cargo run --manifest-path kogi-host/Cargo.toml
```
