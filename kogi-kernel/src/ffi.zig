//! ffi.zig — Rust ↔ Zig Kernel Syscall Interface
//!
//! This file is the **only** boundary between the Rust host and the Zig kernel.
//! Every exported symbol follows the `kogi_kernel_*` naming convention so the
//! Rust side can declare a matching `extern "C"` block without a linker script.
//!
//! Syscall surface (in call order):
//!
//!   Lifecycle
//!   ─────────
//!   kogi_kernel_init()                          → bool
//!   kogi_kernel_deinit()                        → void
//!
//!   Privilege / mode
//!   ────────────────
//!   kogi_kernel_set_mode(mode: i32)             → bool   (0=privileged, 1=user)
//!   kogi_kernel_enter_privileged()              → bool
//!   kogi_kernel_exit_privileged()               → bool
//!
//!   Module management
//!   ─────────────────
//!   kogi_kernel_register_module(id, kind)       → bool
//!   kogi_kernel_module_start(id)                → bool
//!   kogi_kernel_module_stop(id)                 → bool
//!   kogi_kernel_module_remove(id)               → bool
//!
//!   Component registration (full descriptor)
//!   ─────────────────────────────────────────
//!   kogi_kernel_register_component(id, class_tag, endpoint, net_mgr, limits*) → bool
//!
//!   Bootstrap helpers
//!   ─────────────────
//!   kogi_kernel_bootstrap_core()               → bool
//!   kogi_kernel_bootstrap_office()             → bool
//!
//!   Events
//!   ──────
//!   kogi_kernel_publish_event(topic, payload)  → bool
//!   kogi_kernel_event_count()                  → u64
//!
//!   Stats
//!   ─────
//!   kogi_kernel_module_count()                 → u64
//!   kogi_kernel_get_stats(out: *FfiKernelStats) → bool
//!
//! ComponentClass tag mapping (i32 → ModuleClass):
//!   0 = kernel   1 = host   2 = server   3 = engine   4 = services   5+ = module

const std = @import("std");
const kernel_mod = @import("kernel.zig");
const mod_mod = @import("module.zig");

// ─────────────────────────────────────────────────────────────────────────────
// C-compatible structs shared with the Rust host
// ─────────────────────────────────────────────────────────────────────────────

/// Resource limit descriptor passed from Rust when registering a component.
/// Memory is expressed in MiB so the struct fits in a single cache line.
/// Must stay ABI-compatible with `FfiResourceLimits` in kernel.rs.
pub const FfiResourceLimits = extern struct {
    memory_limit_mb: u64 = 64,
    max_processes: u32 = 16,
    max_files: u32 = 1024,
    max_resource_units: u32 = 512,
    max_connections: u32 = 32,
};

/// Flat snapshot of kernel stats returned to the Rust host.
/// All `usize` kernel fields are widened to `u64` for cross-platform safety.
/// Must stay ABI-compatible with `FfiKernelStats` in kernel.rs.
pub const FfiKernelStats = extern struct {
    // Privilege
    mode: i32, // 0 = privileged, 1 = user

    // Memory (converted from bytes → MiB)
    memory_used_mb: u64,
    memory_total_mb: u64,

    // Processes
    live_processes: u64,
    idle_workers: u64,
    busy_workers: u64,
    queued_tasks: u64,

    // Modules
    total_modules: u64,
    active_modules: u64,

    // Services
    total_services: u64,
    running_services: u64,
    faulted_services: u64,

    // Network
    open_sockets: u64,
    active_connections: u64,

    // Events
    kernel_events_total: u64,
    kernel_events_log_len: u64,
    kernel_event_overflow: u64,
};

// ─────────────────────────────────────────────────────────────────────────────
// Singleton kernel instance
// ─────────────────────────────────────────────────────────────────────────────

var kernel_instance: ?kernel_mod.Kernel = null;

// ─────────────────────────────────────────────────────────────────────────────
// Internal helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Convert a null-terminated C string pointer to a Zig slice.
inline fn cStr(ptr: [*:0]const u8) []const u8 {
    return std.mem.span(ptr);
}

/// Map a Rust-side i32 class tag to the kernel's ModuleClass enum.
inline fn classFromTag(tag: i32) mod_mod.ModuleClass {
    return switch (tag) {
        0 => .kernel,
        1 => .host,
        2 => .server,
        3 => .engine,
        4 => .services,
        else => .module,
    };
}

// ─────────────────────────────────────────────────────────────────────────────
// Syscall: lifecycle
// ─────────────────────────────────────────────────────────────────────────────

/// Initialise the kernel singleton with production-default config.
/// Safe to call multiple times — subsequent calls are no-ops and return true.
pub export fn kogi_kernel_init() bool {
    if (kernel_instance != null) return true;

    var k = kernel_mod.Kernel.init(std.heap.page_allocator, .{}) catch return false;
    k.relink();

    // Emit the boot event now that the event subsystem is live.
    k.publishEvent(.host, "kernel.boot", "{\"event\":\"boot\"}") catch {
        k.deinit();
        return false;
    };

    kernel_instance = k;
    return true;
}

/// Tear down the kernel singleton and release all resources.
pub export fn kogi_kernel_deinit() void {
    if (kernel_instance) |*k| {
        k.deinit();
    }
    kernel_instance = null;
}

// ─────────────────────────────────────────────────────────────────────────────
// Syscall: privilege / mode
// ─────────────────────────────────────────────────────────────────────────────

/// Switch privilege mode.  `mode` values: 0 = privileged, 1 = user.
/// The actor is always `.host`; the Rust side must acquire privilege first.
pub export fn kogi_kernel_set_mode(mode: i32) bool {
    var k = &(kernel_instance orelse return false);
    const next: kernel_mod.Mode = if (mode == 0) .privileged else .user;
    k.setMode(next, .host) catch return false;
    return true;
}

/// Elevate the host session to privileged (kernel) mode.
pub export fn kogi_kernel_enter_privileged() bool {
    var k = &(kernel_instance orelse return false);
    k.enterPrivileged(.host) catch return false;
    return true;
}

/// Drop the host session back to user (unprivileged) mode.
pub export fn kogi_kernel_exit_privileged() bool {
    var k = &(kernel_instance orelse return false);
    k.exitPrivileged(.host) catch return false;
    return true;
}

// ─────────────────────────────────────────────────────────────────────────────
// Syscall: module management
// ─────────────────────────────────────────────────────────────────────────────

/// Register a new module by id and kind string.
/// The kernel auto-assigns a default endpoint and resource limits.
pub export fn kogi_kernel_register_module(
    module_id: [*:0]const u8,
    module_kind: [*:0]const u8,
) bool {
    var k = &(kernel_instance orelse return false);
    k.registerModule(.host, cStr(module_id), cStr(module_kind)) catch return false;
    return true;
}

/// Start a previously registered module.
pub export fn kogi_kernel_module_start(module_id: [*:0]const u8) bool {
    var k = &(kernel_instance orelse return false);
    k.startModule(.host, cStr(module_id)) catch return false;
    return true;
}

/// Stop a running module (resources remain allocated).
pub export fn kogi_kernel_module_stop(module_id: [*:0]const u8) bool {
    var k = &(kernel_instance orelse return false);
    k.stopModule(.host, cStr(module_id)) catch return false;
    return true;
}

/// Stop and fully deregister a module, releasing all its resources.
pub export fn kogi_kernel_module_remove(module_id: [*:0]const u8) bool {
    var k = &(kernel_instance orelse return false);
    k.removeModule(.host, cStr(module_id)) catch return false;
    return true;
}

// ─────────────────────────────────────────────────────────────────────────────
// Syscall: component registration (full descriptor)
// ─────────────────────────────────────────────────────────────────────────────

/// Register a platform component with explicit class, network descriptor, and
/// resource limits.  This is the low-level syscall behind the Rust
/// `KernelBridge::register_component` call.
///
/// `class_tag` — see ComponentClass mapping at the top of this file.
/// `limits`    — pointer to an `FfiResourceLimits` struct; must not be null.
pub export fn kogi_kernel_register_component(
    component_id: [*:0]const u8,
    class_tag: i32,
    endpoint: [*:0]const u8,
    network_manager: [*:0]const u8,
    limits: *const FfiResourceLimits,
) bool {
    var k = &(kernel_instance orelse return false);

    const id = cStr(component_id);
    const ep = cStr(endpoint);
    const net_mgr = cStr(network_manager);
    const class = classFromTag(class_tag);

    // 1. Register in the module registry with class-default capability caps.
    k.modules.registry.register(
        .host,
        id,
        id,
        class,
        "1.0.0",
        id,
        ep,
        net_mgr,
        mod_mod.defaultCapsForClass(class),
    ) catch return false;

    // 2. Override limits with the caller-supplied values.
    k.modules.registry.setLimits(.host, id, .{
        .memory_bytes = limits.memory_limit_mb * 1024 * 1024,
        .max_processes = limits.max_processes,
        .max_files = limits.max_files,
        .max_resource_units = limits.max_resource_units,
        .max_network_connections = limits.max_connections,
    }) catch return false;

    // 3. Start the component so it is immediately addressable.
    k.modules.lifecycle.start(.host, id) catch return false;

    // 4. Register as a resource tenant with matching policy.
    k.resources.registerTenant(.host, id, .{
        .memory_bytes = limits.memory_limit_mb * 1024 * 1024,
        .max_processes = limits.max_processes,
        .max_files = limits.max_files,
        .max_units = limits.max_resource_units,
        .max_connections = limits.max_connections,
    }) catch |err| switch (err) {
        error.TenantAlreadyRegistered => {},
        else => return false,
    };

    // 5. Emit a registration event visible to all subscribers.
    k.events.emitModule(.module_registered, .info, id, "component-registered");

    return true;
}

// ─────────────────────────────────────────────────────────────────────────────
// Syscall: bootstrap helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Bootstrap all five core platform components (kernel, host, server, engine,
/// services) in a single call.  Idempotent — already-registered components are
/// silently skipped.
pub export fn kogi_kernel_bootstrap_core() bool {
    var k = &(kernel_instance orelse return false);
    k.bootstrapCorePlatformComponents(.host) catch return false;
    return true;
}

/// Provision the kogi.office module with its canonical resource budget.
pub export fn kogi_kernel_bootstrap_office() bool {
    var k = &(kernel_instance orelse return false);
    k.bootstrapOfficeModule(.host) catch return false;
    return true;
}

// ─────────────────────────────────────────────────────────────────────────────
// Syscall: events
// ─────────────────────────────────────────────────────────────────────────────

/// Publish a custom cross-cutting event.  Both `topic` and `payload` are
/// copied into the kernel's event log before this call returns.
pub export fn kogi_kernel_publish_event(
    topic: [*:0]const u8,
    payload: [*:0]const u8,
) bool {
    var k = &(kernel_instance orelse return false);
    k.publishEvent(.host, cStr(topic), cStr(payload)) catch return false;
    return true;
}

/// Return the total number of events recorded since kernel init.
pub export fn kogi_kernel_event_count() u64 {
    const k = &(kernel_instance orelse return 0);
    return k.events.stats().total_events;
}

// ─────────────────────────────────────────────────────────────────────────────
// Syscall: stats / introspection
// ─────────────────────────────────────────────────────────────────────────────

/// Return the number of registered modules (active + inactive).
pub export fn kogi_kernel_module_count() u64 {
    const k = &(kernel_instance orelse return 0);
    return @intCast(k.stats().total_modules);
}

/// Fill `out` with an atomic snapshot of kernel stats and return true.
/// Returns false if the kernel has not been initialised.
pub export fn kogi_kernel_get_stats(out: *FfiKernelStats) bool {
    const k = &(kernel_instance orelse return false);
    const s = k.stats();

    out.* = .{
        .mode = if (s.mode == .privileged) @as(i32, 0) else @as(i32, 1),
        .memory_used_mb = s.memory_used_bytes / (1024 * 1024),
        .memory_total_mb = s.memory_total_bytes / (1024 * 1024),
        .live_processes = @intCast(s.live_processes),
        .idle_workers = @intCast(s.idle_workers),
        .busy_workers = @intCast(s.busy_workers),
        .queued_tasks = @intCast(s.queued_tasks),
        .total_modules = @intCast(s.total_modules),
        .active_modules = @intCast(s.active_modules),
        .total_services = @intCast(s.total_services),
        .running_services = @intCast(s.running_services),
        .faulted_services = @intCast(s.faulted_services),
        .open_sockets = @intCast(s.open_sockets),
        .active_connections = @intCast(s.active_connections),
        .kernel_events_total = s.kernel_events_total,
        .kernel_events_log_len = @intCast(s.kernel_events_log_len),
        .kernel_event_overflow = s.kernel_events_overflow,
    };
    return true;
}
