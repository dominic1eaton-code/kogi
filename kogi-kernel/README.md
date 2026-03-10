# kogi-kernel

Zig kernel for the Kogi independent worker operating system MVP.

## Responsibilities
- Module registry and orchestration
- Event bus publication (kernel-scoped)
- Scheduler and dispatch tick
- Memory and cache coordination
- Process and file metadata management
- Kernel/user mode barrier and RBAC checks
- C ABI surface for host/server/gateway integration
- Office module bootstrap helper with isolated endpoint provisioning (`/services/office`)

## Files
- `src/kernel.zig` core kernel runtime
- `src/ffi.zig` C ABI entrypoints
- `src/main.zig` standalone demo
