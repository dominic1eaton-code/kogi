# kogi-kernel

Zig kernel for the Kogi independent worker operating system MVP.

## Responsibilities
- Module registry and orchestration
- Platform component registry/orchestration for `kernel`, `host`, `server`, `engine`, `services`, and `modules`
- Event bus publication (kernel-scoped)
- Scheduler and dispatch tick
- Memory and cache coordination
- Process and file metadata management
- Kernel/user mode barrier and RBAC checks
- C ABI surface for host/server/gateway integration
- Office module bootstrap helper with isolated endpoint provisioning (`/services/office`)
- Core platform bootstrap helper with component resource provisioning + network usage tracking
- Bootstrapped by `kogi-host` (and by `kogi-server` when running the host)

## Files
- `src/kernel.zig` core kernel runtime
- `src/ffi.zig` C ABI entrypoints
- `src/main.zig` standalone demo
