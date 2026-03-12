//! kernel.zig — Kernel Coordinator
//!
//! The kernel is the thin top-level orchestration layer that wires together
//! the seven specialised subsystems:
//!
//!   memory.zig    → MemoryManager   (allocation, buddy/slab, pressure)
//!   network.zig   → NetworkManager  (addresses, interfaces, sockets, firewall)
//!   process.zig   → Orchestrator    (process table, scheduler, thread pool)
//!   services.zig  → ServiceManager  (lifecycle, health, dep-graph)
//!   module.zig    → ModuleSystem    (module registry, lifecycle, event bus)
//!   resources.zig → ResourcesManager (unified resource accounting)
//!   events.zig    → EventManager    (kernel-wide event log + subscriber dispatch)
//!
//! The Kernel struct itself contains very little logic.  Every resource
//! acquire/release, every lifecycle transition, and every subsystem call
//! are delegated to the appropriate subsystem.  The kernel only:
//!
//!   • Boots the subsystems in the correct dependency order
//!   • Exposes convenience one-liners used by bootstrap code
//!   • Aggregates KernelStats from all seven subsystems
//!   • Enforces the top-level RBAC rules (role → Permission)
//!   • Emits a KernelEvent for every state-changing operation
//!
//! Subsystem dependency start order:
//!   memory → process → network → module → resources → services → events

const std = @import("std");

const mem_mod  = @import("memory.zig");
const proc_mod = @import("processes.zig");
const net_mod  = @import("network.zig");
const mod_mod  = @import("module.zig");
const svc_mod  = @import("services.zig");
const res_mod  = @import("resources.zig");
const evt_mod  = @import("events.zig");

// ─────────────────────────────────────────────────────────────────────────────
// Re-export shared primitives (single canonical definition across the kernel)
// ─────────────────────────────────────────────────────────────────────────────

pub const Role       = res_mod.Role;
pub const Permission = res_mod.Permission;
pub const hasPermission = res_mod.hasPermission;
pub const EventManager   = evt_mod.EventManager;
pub const EventFilter    = evt_mod.EventFilter;
pub const EventDomain    = evt_mod.EventDomain;
pub const EventDomainSet = evt_mod.EventDomainSet;
pub const EventSeverity  = evt_mod.EventSeverity;
pub const EventKind      = evt_mod.EventKind;
pub const KernelEvent    = evt_mod.KernelEvent;
pub const HandlerFn      = evt_mod.HandlerFn;

// ─────────────────────────────────────────────────────────────────────────────
// Errors
// ─────────────────────────────────────────────────────────────────────────────

pub const KernelError = error{
    AccessDenied,
    PrivilegeDenied,
    NotInitialised,
    AlreadyInitialised,
    SubsystemError,
    ModuleNotFound,
    ComponentNotFound,
};

// ─────────────────────────────────────────────────────────────────────────────
// Kernel Mode
// ─────────────────────────────────────────────────────────────────────────────

/// Kernel privilege mode.
/// - `privileged`  — kernel-level operations are permitted; requires root/host role.
/// - `user`        — unprivileged shell; only informational commands are available.
///
/// `kernel` is kept as a compile-time alias for backward compatibility.
pub const Mode = enum {
    privileged,
    user,

    /// Backward-compatible alias so existing code using `.kernel` still compiles.
    pub const kernel = Mode.privileged;

    pub fn label(self: Mode) []const u8 {
        return switch (self) {
            .privileged => "privileged",
            .user        => "user",
        };
    }

    pub fn isPrivileged(self: Mode) bool {
        return self == .privileged;
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// Bootstrap configuration
// ─────────────────────────────────────────────────────────────────────────────

pub const KernelConfig = struct {
    // ── Memory ──────────────────────────────────────────────────────────────
    /// Total heap pool available to the kernel (default 4 GiB).
    memory_total_bytes:   u64   = 4  * 1024 * 1024 * 1024,
    /// Capacity of the MemoryManager audit log (0 = disabled).
    memory_audit_cap:     usize = 4096,
    /// Enable allocation audit log.
    memory_audit_enabled: bool  = true,

    // ── Processes ────────────────────────────────────────────────────────────
    /// Number of logical worker threads in the thread pool.
    worker_count:     u32   = 16,
    /// Per-priority-lane task queue depth.
    queue_lane_cap:   usize = 256,

    // ── Network ──────────────────────────────────────────────────────────────
    max_sockets:       usize                    = 512,
    max_connections:   usize                    = 256,
    max_conn_failures: u32                      = 5,
    packet_lane_cap:   usize                    = 1024,
    firewall_default:  net_mod.FirewallAction   = .allow,

    // ── Module system ────────────────────────────────────────────────────────
    module_event_cap: usize = 2048,

    // ── Services ─────────────────────────────────────────────────────────────
    service_event_cap:      usize = 1024,
    global_mem_ceiling:     u64   = 32 * 1024 * 1024 * 1024,
    global_process_ceiling: u32   = 4096,
    global_conn_ceiling:    u32   = 8192,

    // ── Resources ────────────────────────────────────────────────────────────
    resource_log_cap: usize = 4096,
    /// Capacity of the kernel-wide EventManager ring buffer.
    event_log_cap: usize = 8192,
};

// ─────────────────────────────────────────────────────────────────────────────
// KernelStats  (aggregate snapshot from all six subsystems)
// ─────────────────────────────────────────────────────────────────────────────

pub const KernelStats = struct {
    // Mode
    mode:         Mode,
    session_role: Role,

    // Memory
    memory_used_bytes:  u64,
    memory_total_bytes: u64,

    // Processes
    live_processes:  usize,
    idle_workers:    usize,
    busy_workers:    usize,
    queued_tasks:    usize,
    scheduled_jobs:  usize,

    // Network
    open_sockets:       usize,
    active_connections: usize,
    queued_packets:     usize,
    route_count:        usize,

    // Modules
    total_modules:  usize,
    active_modules: usize,
    module_events:  usize,

    // Services
    total_services:   usize,
    running_services: usize,
    faulted_services: usize,
    service_events:   usize,

    // Resources
    tenant_count:         usize,
    resource_event_count: usize,
    // Events
    kernel_events_total:       u64,
    kernel_events_log_len:     usize,
    kernel_events_overflow:    u64,
    kernel_event_subscribers:  usize,
};

// ─────────────────────────────────────────────────────────────────────────────
// Kernel
// ─────────────────────────────────────────────────────────────────────────────

pub const Kernel = struct {
    allocator:    std.mem.Allocator,
    mode:         Mode,
    /// Role of the currently active shell session (changes with su/sudo).
    session_role: Role,

    // ── Subsystems ───────────────────────────────────────────────────────────
    memory:    mem_mod.MemoryManager,
    processes: proc_mod.Orchestrator,
    network:   net_mod.NetworkManager,
    modules:   mod_mod.ModuleSystem,
    resources: res_mod.ResourcesManager,
    services:  svc_mod.ServiceManager,
    events:    evt_mod.EventManager,

    // ── Process-subsystem owned objects (Orchestrator borrows pointers) ──────
    _proc_table:  proc_mod.ProcessTable,
    _thread_pool: proc_mod.ThreadPool,
    _work_queue:  proc_mod.WorkQueue,
    _scheduler:   proc_mod.Scheduler,
    _res_ledger:  proc_mod.ResourceLedger,

    // ── Memory audit log (optional) ──────────────────────────────────────────
    _audit_log: ?mem_mod.AuditLog,

    // ─────────────────────────────────────────────────────────────────────────

    pub fn init(allocator: std.mem.Allocator, cfg: KernelConfig) !Kernel {
        // ── 1. Memory ──────────────────────────────────────────────────────
        var audit: ?mem_mod.AuditLog = if (cfg.memory_audit_enabled)
            try mem_mod.AuditLog.init(allocator, cfg.memory_audit_cap)
        else
            null;

        const mem_mgr = mem_mod.MemoryManager.init(
            allocator,
            cfg.memory_total_bytes,
            if (cfg.memory_audit_enabled) &(audit.?) else null,
        );

        // ── 2. Processes ───────────────────────────────────────────────────
        const proc_table  = proc_mod.ProcessTable.init(allocator);
        const thread_pool = try proc_mod.ThreadPool.init(allocator, cfg.worker_count);
        const work_queue  = proc_mod.WorkQueue.init(allocator, cfg.queue_lane_cap);
        const scheduler   = proc_mod.Scheduler.init(allocator);
        const res_ledger  = proc_mod.ResourceLedger.init(allocator);

        // ── 3. Network ─────────────────────────────────────────────────────
        const net_mgr = net_mod.NetworkManager.init(
            allocator,
            cfg.max_sockets,
            cfg.max_connections,
            cfg.max_conn_failures,
            cfg.packet_lane_cap,
            cfg.firewall_default,
        );

        // ── 4. Module system ───────────────────────────────────────────────
        var mod_sys = mod_mod.ModuleSystem.init(
            allocator,
            cfg.memory_total_bytes,
            cfg.module_event_cap,
        );
        // ModuleSystem stores cross-references by value; relink after move.
        mod_sys.relink();

        // ── 5. Resources ───────────────────────────────────────────────────
        var res_mgr = res_mod.ResourcesManager.init(
            allocator,
            .{
                .memory_bytes    = cfg.global_mem_ceiling,
                .max_processes   = cfg.global_process_ceiling,
                .max_connections = cfg.global_conn_ceiling,
            },
            cfg.resource_log_cap,
        );
        res_mgr.relink();

        // ── 6. Services ────────────────────────────────────────────────────
        var svc_mgr = svc_mod.ServiceManager.init(
            allocator,
            .{
                .memory_bytes    = cfg.global_mem_ceiling,
                .max_processes   = cfg.global_process_ceiling,
                .max_connections = cfg.global_conn_ceiling,
            },
            cfg.service_event_cap,
        );
        svc_mgr.relink();
        // ── 7. Events ─────────────────────────────────────────────────────
        const evt_mgr = try evt_mod.EventManager.init(
            allocator,
            cfg.event_log_cap,
        );

        // Build Orchestrator last (it borrows pointers to the owned sub-objects
        // above; those are stored inline in the Kernel struct so the pointers
        // will be stable once the struct is in its final location).
        const orch = proc_mod.Orchestrator.init(
            allocator,
            &proc_table,
            &thread_pool,
            &scheduler,
            &work_queue,
            &res_ledger,
        );

        return .{
            .allocator     = allocator,
            .mode          = .privileged,
            .session_role  = .root,
            .memory        = mem_mgr,
            .processes     = orch,
            .network       = net_mgr,
            .modules       = mod_sys,
            .resources     = res_mgr,
            .services      = svc_mgr,
            ._proc_table   = proc_table,
            ._thread_pool  = thread_pool,
            ._work_queue   = work_queue,
            ._scheduler    = scheduler,
            ._res_ledger   = res_ledger,
            ._audit_log    = audit,
            .events        = evt_mgr,
        };
    }

    /// Re-seat all internal pointer cross-references after the Kernel struct
    /// has been moved into its final storage location (e.g. the heap).
    /// Must be called once immediately after init.
    pub fn relink(self: *Kernel) void {
        // Orchestrator borrows pointers — rebuild with addresses of the
        // inline fields at their final location.
        self.processes = proc_mod.Orchestrator.init(
            self.allocator,
            &self._proc_table,
            &self._thread_pool,
            &self._scheduler,
            &self._work_queue,
            &self._res_ledger,
        );

        // ModuleSystem and ResourcesManager also hold self-referential ptrs.
        self.modules.relink();
        self.resources.relink();
        self.services.relink();

        // Point ResourcesManager's bridge at the live MemoryManager.
        self.resources.attachMemoryManager(&self.memory);
        self.resources.attachTrafficLedger(&self.network.ledger);
        self.resources.attachProcessTable(&self._proc_table);
        self.resources.attachModuleRegistry(&self.modules.registry);
    }

    pub fn deinit(self: *Kernel) void {
        self.events.deinit();
        self.services.deinit();
        self.resources.deinit();
        self.modules.deinit();
        self.network.deinit();
        self.processes.deinit();
        self._proc_table.deinit();
        self._thread_pool.deinit();
        self._work_queue.deinit();
        self._scheduler.deinit();
        self._res_ledger.deinit();
        self.memory.deinit();
        if (self._audit_log) |*a| a.deinit();
    }

    // ─────────────────────────────────────────────────────────────────────
    // RBAC enforcement
    // ─────────────────────────────────────────────────────────────────────

    pub fn enforce(_: *const Kernel, actor: Role, permission: Permission) !void {
        if (!hasPermission(actor, permission)) return KernelError.AccessDenied;
    }

    /// Transition the kernel privilege mode.
    /// Entering `.privileged` requires `.root` or `.host`; anyone may drop to `.user`.
    pub fn setMode(self: *Kernel, mode: Mode, actor: Role) !void {
        if (mode == .privileged and actor != .root and actor != .host)
            return KernelError.PrivilegeDenied;
        if (mode == .user and actor != .root and actor != .host and actor != .server)
            return KernelError.AccessDenied;
        self.mode = mode;
        self.events.emitKernel(.kernel_mode_changed, .info, mode.label());
    }

    /// Elevate the current session to privileged mode.
    /// Returns `PrivilegeDenied` if the actor is not root or host.
    pub fn enterPrivileged(self: *Kernel, actor: Role) !void {
        try self.setMode(.privileged, actor);
        self.session_role = actor;
        self.events.emitKernel(.kernel_mode_changed, .info, "enter-privileged");
    }

    /// Drop the current session to user mode.
    pub fn exitPrivileged(self: *Kernel, actor: Role) !void {
        try self.setMode(.user, actor);
        self.session_role = .user;
        self.events.emitKernel(.kernel_mode_changed, .info, "exit-privileged");
    }

    /// Return true iff the kernel is currently in privileged mode.
    pub fn isPrivileged(self: *const Kernel) bool {
        return self.mode == .privileged;
    }

    /// Enforce both RBAC *and* that the kernel is in privileged mode.
    /// Use this for operations that should never run in user mode.
    pub fn enforcePrivileged(self: *const Kernel, actor: Role, permission: Permission) !void {
        if (!self.mode.isPrivileged()) return KernelError.PrivilegeDenied;
        try self.enforce(actor, permission);
    }

    // ─────────────────────────────────────────────────────────────────────
    // Memory convenience wrappers
    // ─────────────────────────────────────────────────────────────────────

    /// Allocate `bytes` from the global pool (no tenant tracking).
    pub fn allocateMemory(self: *Kernel, actor: Role, bytes: u64) !void {
        try self.enforcePrivileged(actor, .manage_memory);
        try self.memory.allocate(bytes);
    }

    pub fn freeMemory(self: *Kernel, actor: Role, bytes: u64) !void {
        try self.enforcePrivileged(actor, .manage_memory);
        self.memory.free(bytes);
    }

    /// Allocate memory for a specific tenant, respecting per-tenant limits.
    pub fn allocateTenantMemory(self: *Kernel, actor: Role, tenant_id: []const u8, bytes: u64) !void {
        try self.enforcePrivileged(actor, .manage_memory);
        try self.resources.acquireMemory(actor, tenant_id, bytes);
        self.events.emitMemory(.memory_allocated, .info, tenant_id, "tenant-memory-acquired");
    }

    pub fn freeTenantMemory(self: *Kernel, actor: Role, tenant_id: []const u8, bytes: u64) !void {
        try self.enforcePrivileged(actor, .manage_memory);
        try self.resources.releaseMemory(actor, tenant_id, bytes);
        self.events.emitMemory(.memory_freed, .info, tenant_id, "tenant-memory-released");
    }

    // ─────────────────────────────────────────────────────────────────────
    // Process convenience wrappers
    // ─────────────────────────────────────────────────────────────────────

    pub fn spawnProcess(self: *Kernel, actor: Role, name: []const u8, owner: Role) !u64 {
        try self.enforcePrivileged(actor, .manage_processes);
        const pid = try self._proc_table.spawn(actor, name, owner, 128, null);
        self.events.emitProcess(.process_spawned, .info, name, "spawned");
        return pid;
    }

    pub fn terminateProcess(self: *Kernel, actor: Role, pid: u64) !void {
        try self.enforcePrivileged(actor, .manage_processes);
        try self._proc_table.transition(actor, pid, .terminated);
        self.events.emitProcess(.process_terminated, .info, "kernel", "terminated");
    }

    // ─────────────────────────────────────────────────────────────────────
    // Network convenience wrappers
    // ─────────────────────────────────────────────────────────────────────

    pub fn registerNetworkAddress(
        self:         *Kernel,
        actor:        Role,
        name:         []const u8,
        addr:         net_mod.Ipv4,
        port:         u16,
        component_id: []const u8,
        scheme:       []const u8,
    ) !void {
        try self.enforcePrivileged(actor, .manage_modules);
        try self.network.addresses.register(actor, name, addr, port, component_id, scheme);
        self.events.emitNetwork(.network_address_registered, .info, name, component_id);
    }

    pub fn recordNetworkTraffic(
        self:         *Kernel,
        actor:        Role,
        component_id: []const u8,
        ingress:      u64,
        egress:       u64,
    ) !void {
        try self.enforcePrivileged(actor, .manage_modules);
        try self.resources.recordIngress(actor, component_id, ingress);
        try self.resources.recordEgress(actor, component_id, egress);
        self.events.emitNetwork(.network_traffic_recorded, .debug, component_id, "traffic");
    }

    // ─────────────────────────────────────────────────────────────────────
    // Module convenience wrappers
    // ─────────────────────────────────────────────────────────────────────

    pub fn registerModule(
        self:  *Kernel,
        actor: Role,
        id:    []const u8,
        kind:  []const u8,
    ) !void {
        try self.enforcePrivileged(actor, .manage_modules);
        const endpoint = try std.fmt.allocPrint(self.allocator, "/modules/{s}", .{id});
        defer self.allocator.free(endpoint);
        try self.modules.registry.register(
            actor,
            id, kind,
            .module,
            "1.0.0", kind,
            endpoint,
            "kogi-go-network",
            mod_mod.defaultCapsForClass(.module),
        );
        // Automatically register the module as a resource tenant.
        self.resources.registerTenant(actor, id, .{}) catch |err| switch (err) {
            res_mod.ResourceError.TenantAlreadyRegistered => {},
            else => return err,
        };
        self.events.emitModule(.module_registered, .info, id, "registered");
    }

    pub fn startModule(self: *Kernel, actor: Role, id: []const u8) !void {
        try self.enforcePrivileged(actor, .manage_modules);
        try self.modules.lifecycle.start(actor, id);
        self.events.emitModule(.module_started, .info, id, "started");
    }

    pub fn stopModule(self: *Kernel, actor: Role, id: []const u8) !void {
        try self.enforcePrivileged(actor, .manage_modules);
        try self.modules.lifecycle.stop(actor, id);
        self.resources.releaseAll(id);
        self.events.emitModule(.module_stopped, .info, id, "stopped");
    }

    pub fn removeModule(self: *Kernel, actor: Role, id: []const u8) !void {
        try self.enforcePrivileged(actor, .manage_modules);
        try self.modules.lifecycle.remove(actor, id);
        self.resources.releaseAll(id);
        self.resources.removeTenant(actor, id) catch {};
        self.events.emitModule(.module_removed, .info, id, "removed");
    }

    // ─────────────────────────────────────────────────────────────────────
    // Service convenience wrappers
    // ─────────────────────────────────────────────────────────────────────

    pub fn registerService(
        self:         *Kernel,
        actor:        Role,
        id:           []const u8,
        display_name: []const u8,
        version:      []const u8,
        budget:       svc_mod.ResourceBudget,
        restart:      svc_mod.RestartConfig,
        health:       svc_mod.HealthPolicy,
        handle:       svc_mod.ServiceHandle,
    ) !void {
        try self.enforcePrivileged(actor, .manage_modules);
        try self.services.register(actor, id, display_name, version, budget, restart, health, handle);
    }

    pub fn startService(self: *Kernel, actor: Role, id: []const u8) !void {
        try self.enforcePrivileged(actor, .manage_modules);
        try self.services.provisionAndStart(actor, id);
        self.events.emitService(.service_started, .info, id, "started");
    }

    pub fn stopService(self: *Kernel, actor: Role, id: []const u8) !void {
        try self.enforcePrivileged(actor, .manage_modules);
        try self.services.stop(actor, id);
        self.events.emitService(.service_stopped, .info, id, "stopped");
    }

    /// Start all registered services in topological dependency order.
    pub fn startAllServices(self: *Kernel, actor: Role) !usize {
        try self.enforcePrivileged(actor, .manage_modules);
        return self.services.startAll(actor);
    }

    /// Stop all running services in reverse dependency order.
    pub fn stopAllServices(self: *Kernel, actor: Role) !void {
        try self.enforcePrivileged(actor, .manage_modules);
        try self.services.stopAll(actor);
    }

    // ─────────────────────────────────────────────────────────────────────
    // Resource convenience wrappers
    // ─────────────────────────────────────────────────────────────────────

    pub fn registerTenant(self: *Kernel, actor: Role, id: []const u8, policy: res_mod.ResourcePolicy) !void {
        try self.enforcePrivileged(actor, .manage_modules);
        try self.resources.registerTenant(actor, id, policy);
    }

    pub fn acquireResource(self: *Kernel, actor: Role, req: res_mod.AllocationRequest) !void {
        try self.resources.acquire(actor, req);
    }

    pub fn releaseResource(self: *Kernel, actor: Role, req: res_mod.ReleaseRequest) void {
        self.resources.release(actor, req);
    }

    // ─────────────────────────────────────────────────────────────────────
    // Scheduler
    // ─────────────────────────────────────────────────────────────────────

    pub fn schedule(
        self:        *Kernel,
        actor:       Role,
        label:       []const u8,
        payload:     []const u8,
        due_at_ms:   i64,
        interval_ms: ?u64,
        priority:    proc_mod.TaskPriority,
        owner_pid:   ?u64,
    ) ![]const u8 {
        try self.enforce(actor, .schedule_tasks);
        return self._scheduler.schedule(actor, label, payload, due_at_ms, interval_ms, priority, owner_pid);
    }

    pub fn tick(self: *Kernel, actor: Role, now_ms: i64) !struct { fired: u64, dispatched: u64 } {
        try self.enforce(actor, .schedule_tasks);
        return self.processes.tick(actor, now_ms);
    }

    // ─────────────────────────────────────────────────────────────────────
    // Health tick
    // ─────────────────────────────────────────────────────────────────────

    pub fn healthTick(self: *Kernel, now_ms: i64) !usize {
        return self.services.healthTick(now_ms);
    }

    pub fn recoverFaultedServices(self: *Kernel, actor: Role) !usize {
        return self.services.recoverFaulted(actor);
    }

    // ─────────────────────────────────────────────────────────────────────
    // Bootstrap helpers
    // ─────────────────────────────────────────────────────────────────────

    /// Register and fully provision all core platform components in one call.
    /// Mirrors the old bootstrapCorePlatformComponents from kernel.zig.
    pub fn bootstrapCorePlatformComponents(self: *Kernel, actor: Role) !void {
        try self.enforcePrivileged(actor, .manage_modules);

        const components = [_]struct {
            id:       []const u8,
            class:    mod_mod.ModuleClass,
            endpoint: []const u8,
            net_mgr:  []const u8,
        }{
            .{ .id = "kogi.kernel",   .class = .kernel,   .endpoint = "local://kogi-kernel",        .net_mgr = "kernel-native"    },
            .{ .id = "kogi.host",     .class = .host,     .endpoint = "local://kogi-host",          .net_mgr = "kernel-native"    },
            .{ .id = "kogi.server",   .class = .server,   .endpoint = "http://127.0.0.1:8080/health",.net_mgr = "kogi-go-network" },
            .{ .id = "kogi.engine",   .class = .engine,   .endpoint = "local://kogi-engine",        .net_mgr = "kogi-go-network" },
            .{ .id = "kogi.network", .class = .services, .endpoint = "http://127.0.0.1:8090/health",.net_mgr = "kogi-go-network" },
        };

        for (components) |c| {
            try self.modules.registry.register(
                actor, c.id, c.id, c.class,
                "1.0.0", c.id, c.endpoint, c.net_mgr,
                mod_mod.defaultCapsForClass(c.class),
            );
            try self.modules.lifecycle.start(actor, c.id);

            // Register as resource tenant with class-default policy.
            const limits = c.class.defaultLimits();
            self.resources.registerTenant(actor, c.id, .{
                .memory_bytes    = limits.memory_bytes,
                .max_processes   = limits.max_processes,
                .max_files       = limits.max_files,
                .max_units       = limits.max_resource_units,
                .max_connections = limits.max_network_connections,
            }) catch |err| switch (err) {
                res_mod.ResourceError.TenantAlreadyRegistered => {},
                else => return err,
            };
        }

        // Provision initial resources for the standard components.
        try self.resources.acquireMemory(actor, "kogi.host",     32 * 1024 * 1024);
        try self.resources.acquireMemory(actor, "kogi.server",   48 * 1024 * 1024);
        try self.resources.acquireMemory(actor, "kogi.engine",   64 * 1024 * 1024);
        try self.resources.acquireProcess(actor, "kogi.host");
        try self.resources.acquireProcess(actor, "kogi.server");
        try self.resources.acquireProcess(actor, "kogi.engine");
        try self.resources.acquireFile(actor, "kogi.engine");
        try self.resources.acquireUnits(actor, "kogi.network", 256);
        try self.resources.recordIngress(actor, "kogi.network", 4096);
        try self.resources.recordEgress(actor, "kogi.network", 8192);
    }

    /// Register the canonical network managers used by all components.
    pub fn bootstrapNetworkManagers(self: *Kernel, actor: Role) !void {
        try self.enforcePrivileged(actor, .manage_modules);
        try self.network.registerManager(actor, "kernel-native",   "native");
        try self.network.registerManager(actor, "kogi-go-network", "go-rpc");
    }

    /// Provision the kogi.office module with its standard resource budget.
    /// Mirrors the old bootstrapOfficeModule from kernel.zig.
    pub fn bootstrapOfficeModule(self: *Kernel, actor: Role) !void {
        try self.enforcePrivileged(actor, .manage_modules);

        const id = "kogi.office";

        try self.modules.registry.register(
            actor, id, "office", .module,
            "2.0.0", "Office Module",
            "/services/office", "kogi-go-network",
            mod_mod.defaultCapsForClass(.module),
        );

        try self.modules.registry.setLimits(actor, id, .{
            .memory_bytes        = 768 * 1024 * 1024,
            .max_processes       = 96,
            .max_files           = 6_000,
            .max_resource_units  = 14_000,
            .max_network_connections = 64,
        });

        try self.modules.lifecycle.start(actor, id);

        self.resources.registerTenant(actor, id, .{
            .memory_bytes    = 768 * 1024 * 1024,
            .max_processes   = 96,
            .max_files       = 6_000,
            .max_units       = 14_000,
            .max_connections = 64,
        }) catch |err| switch (err) {
            res_mod.ResourceError.TenantAlreadyRegistered => {},
            else => return err,
        };

        // Provision initial resources via ResourcesManager.
        const dims = [_]res_mod.DimensionAcquire{
            .{ .memory_bytes = 64 * 1024 * 1024 },
            .{ .processes    = 2  },
            .{ .files        = 2  },
            .{ .units        = 320 },
            .{ .network_ingress = 4096 },
            .{ .network_egress  = 8192 },
        };
        try self.resources.acquire(actor, .{ .tenant_id = id, .dimensions = &dims });

        try self.modules.bus.publish(id, "office.module.bootstrapped",
            "{\\"module\\":\\"kogi.office\\",\\"views\\":[\\"dashboard\\",\\"portfolio\\",\\"timeline\\",\\"workspace\\",\\"assistant\\"]}");
    }

    // ─────────────────────────────────────────────────────────────────────
    // Event management  (pub API delegating to EventManager)
    // ─────────────────────────────────────────────────────────────────────

    /// Subscribe to kernel events.  The handler is called synchronously on
    /// every matching publish; it must not call back into the Kernel.
    pub fn subscribeEvents(
        self:    *Kernel,
        actor:   Role,
        name:    []const u8,
        filter:  evt_mod.EventFilter,
        handler: evt_mod.HandlerFn,
        ctx:     ?*anyopaque,
    ) !void {
        try self.enforcePrivileged(actor, .manage_modules);
        try self.events.subscribe(actor, name, filter, handler, ctx);
    }

    pub fn unsubscribeEvents(self: *Kernel, actor: Role, name: []const u8) !void {
        try self.enforcePrivileged(actor, .manage_modules);
        try self.events.unsubscribe(actor, name);
    }

    /// Retrieve events from the log with id > `watermark`, optionally filtered.
    /// Pass `null` for filter to get every event.
    pub fn queryEvents(
        self:      *const Kernel,
        filter:    ?*const evt_mod.EventFilter,
        watermark: u64,
        out:       *std.ArrayList(evt_mod.KernelEvent),
    ) !void {
        if (filter) |f| {
            try self.events.query(f, watermark, out);
        } else {
            try self.events.since(watermark, out);
        }
    }

    /// Publish a custom cross-cutting event from outside the kernel.
    pub fn publishEvent(
        self:      *Kernel,
        actor:     Role,
        source_id: []const u8,
        payload:   []const u8,
    ) !void {
        try self.enforcePrivileged(actor, .manage_modules);
        self.events.emitCustom(source_id, payload);
    }

    /// Suspend synchronous event delivery to all subscribers.
    /// Events continue to be appended to the log.
    pub fn suspendEventDispatch(self: *Kernel, actor: Role) !void {
        try self.enforcePrivileged(actor, .kernel_admin);
        self.events.suspendDispatch();
    }

    /// Resume synchronous event delivery.
    pub fn resumeEventDispatch(self: *Kernel, actor: Role) !void {
        try self.enforcePrivileged(actor, .kernel_admin);
        self.events.resumeDispatch();
    }

    /// Return the most recently published KernelEvent, or null.
    pub fn latestEvent(self: *const Kernel) ?evt_mod.KernelEvent {
        return self.events.latest();
    }

    // ─────────────────────────────────────────────────────────────────────
    // Aggregate stats
    // ─────────────────────────────────────────────────────────────────────

    pub fn stats(self: *const Kernel) KernelStats {
        const mem_s  = self.memory.stats();
        const proc_s = self.processes.stats();
        const net_s  = self.network.stats();
        const mod_s  = self.modules.stats();
        const svc_s  = self.services.stats();
        const res_s  = self.resources.stats();
        const evt_s  = self.events.stats();

        return .{
            .mode               = self.mode,
            .session_role       = self.session_role,
            .memory_used_bytes  = mem_s.used_bytes,
            .memory_total_bytes = mem_s.total_bytes,
            .live_processes     = proc_s.live_processes,
            .idle_workers       = proc_s.idle_workers,
            .busy_workers       = proc_s.busy_workers,
            .queued_tasks       = proc_s.queued_tasks,
            .scheduled_jobs     = proc_s.scheduled_jobs,
            .open_sockets       = net_s.open_sockets,
            .active_connections = net_s.active_connections,
            .queued_packets     = net_s.queued_packets,
            .route_count        = net_s.route_count,
            .total_modules      = mod_s.total_modules,
            .active_modules     = mod_s.active_modules,
            .module_events      = mod_s.event_count,
            .total_services     = svc_s.total,
            .running_services   = svc_s.running,
            .faulted_services   = svc_s.faulted,
            .service_events     = svc_s.total_events,
            .tenant_count       = res_s.tenant_count,
            .resource_event_count = res_s.event_count,
            .kernel_events_total      = evt_s.total_events,
            .kernel_events_log_len    = evt_s.log_len,
            .kernel_events_overflow   = evt_s.overflow_count,
            .kernel_event_subscribers = evt_s.active_subscribers,
        };
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

fn makeKernel(allocator: std.mem.Allocator) !*Kernel {
    const k = try allocator.create(Kernel);
    errdefer allocator.destroy(k);
    k.* = try Kernel.init(allocator, .{
        .memory_total_bytes    = 4  * 1024 * 1024 * 1024,
        .memory_audit_enabled  = false,
        .worker_count          = 4,
        .queue_lane_cap        = 64,
        .max_sockets           = 64,
        .max_connections       = 32,
        .max_conn_failures     = 3,
        .packet_lane_cap       = 128,
        .module_event_cap      = 256,
        .service_event_cap     = 256,
        .global_mem_ceiling    = 8 * 1024 * 1024 * 1024,
        .global_process_ceiling = 512,
        .global_conn_ceiling   = 1024,
        .resource_log_cap      = 256,
        .event_log_cap         = 512,
    });
    k.relink();
    return k;
}

test "kernel: init and deinit" {
    const k = try makeKernel(std.testing.allocator);
    defer {
        k.deinit();
        std.testing.allocator.destroy(k);
    }
    try std.testing.expectEqual(Mode.privileged, k.mode);
}

test "kernel: RBAC — user role denied" {
    const k = try makeKernel(std.testing.allocator);
    defer { k.deinit(); std.testing.allocator.destroy(k); }

    try std.testing.expectError(
        KernelError.AccessDenied,
        k.registerModule(.user, "evil", "evil"),
    );
}

test "kernel: registerModule registers in module system and resource registry" {
    const k = try makeKernel(std.testing.allocator);
    defer { k.deinit(); std.testing.allocator.destroy(k); }

    try k.registerModule(.host, "test.mod", "test");
    try std.testing.expectEqual(@as(usize, 1), k.modules.registry.count());
    try std.testing.expect(k.resources.registry.getPtr("test.mod") != null);
}

test "kernel: start and stop module" {
    const k = try makeKernel(std.testing.allocator);
    defer { k.deinit(); std.testing.allocator.destroy(k); }

    try k.registerModule(.host, "a.mod", "a");
    try k.startModule(.host, "a.mod");
    try std.testing.expectEqual(
        mod_mod.ModuleStatus.active,
        k.modules.registry.get("a.mod").?.status,
    );

    try k.stopModule(.host, "a.mod");
    try std.testing.expectEqual(
        mod_mod.ModuleStatus.stopped,
        k.modules.registry.get("a.mod").?.status,
    );
}

test "kernel: bootstrapCorePlatformComponents" {
    const k = try makeKernel(std.testing.allocator);
    defer { k.deinit(); std.testing.allocator.destroy(k); }

    try k.bootstrapCorePlatformComponents(.host);
    try std.testing.expectEqual(@as(usize, 5), k.modules.registry.count());
    try std.testing.expectEqual(@as(usize, 5), k.modules.registry.activeCount());
    try std.testing.expect(k.resources.registry.getPtr("kogi.kernel") != null);
}

test "kernel: bootstrapOfficeModule provisions resources" {
    const k = try makeKernel(std.testing.allocator);
    defer { k.deinit(); std.testing.allocator.destroy(k); }

    try k.bootstrapOfficeModule(.host);

    const usage = k.resources.getUsage("kogi.office").?;
    try std.testing.expectEqual(@as(u64, 64 * 1024 * 1024), usage.memory_bytes);
    try std.testing.expectEqual(@as(u32, 2), usage.process_count);
    try std.testing.expectEqual(@as(u32, 2), usage.file_count);
    try std.testing.expectEqual(@as(u32, 320), usage.unit_count);
    try std.testing.expectEqual(@as(u64, 4096), usage.ingress_bytes);
    try std.testing.expectEqual(@as(u64, 8192), usage.egress_bytes);
}

test "kernel: allocateTenantMemory enforces limits" {
    const k = try makeKernel(std.testing.allocator);
    defer { k.deinit(); std.testing.allocator.destroy(k); }

    try k.registerTenant(.host, "limited", .{ .memory_bytes = 1000 });
    try k.allocateTenantMemory(.host, "limited", 800);
    try std.testing.expectError(
        res_mod.ResourceError.TenantLimitExceeded,
        k.allocateTenantMemory(.host, "limited", 300),
    );
    try k.freeTenantMemory(.host, "limited", 800);
    try std.testing.expectEqual(@as(u64, 0), k.resources.getUsage("limited").?.memory_bytes);
}

test "kernel: spawnProcess and terminateProcess" {
    const k = try makeKernel(std.testing.allocator);
    defer { k.deinit(); std.testing.allocator.destroy(k); }

    const pid = try k.spawnProcess(.host, "worker", .module_runtime);
    try std.testing.expect(pid >= 1000);
    try std.testing.expectEqual(@as(usize, 1), k._proc_table.liveCount());

    try k.terminateProcess(.host, pid);
    try std.testing.expectEqual(@as(usize, 0), k._proc_table.liveCount());
}

test "kernel: schedule and tick" {
    const k = try makeKernel(std.testing.allocator);
    defer { k.deinit(); std.testing.allocator.destroy(k); }

    _ = try k.schedule(.host, "job", "{}", 0, null, .normal, null);
    const result = try k.tick(.host, 0);
    try std.testing.expectEqual(@as(u64, 1), result.fired);
}

test "kernel: aggregate stats" {
    const k = try makeKernel(std.testing.allocator);
    defer { k.deinit(); std.testing.allocator.destroy(k); }

    try k.bootstrapCorePlatformComponents(.host);
    const s = k.stats();
    try std.testing.expectEqual(Mode.privileged, s.mode);
    try std.testing.expectEqual(@as(usize, 5), s.active_modules);
    try std.testing.expectEqual(@as(usize, 5), s.tenant_count);
    try std.testing.expect(s.memory_used_bytes > 0);
}

test "kernel: mode transition" {
    const k = try makeKernel(std.testing.allocator);
    defer { k.deinit(); std.testing.allocator.destroy(k); }

    try k.setMode(.user, .root);
    try std.testing.expectEqual(Mode.user, k.mode);
    try std.testing.expectError(KernelError.PrivilegeDenied, k.setMode(.privileged, .user));
}

test "kernel: networkAddress registration" {
    const k = try makeKernel(std.testing.allocator);
    defer { k.deinit(); std.testing.allocator.destroy(k); }

    const ip = try net_mod.Ipv4.parse("127.0.0.1");
    try k.registerNetworkAddress(.host, "kogi.server", ip, 8080, "kogi.server", "http");

    const rec = k.network.addresses.resolve("kogi.server");
    try std.testing.expect(rec != null);
    try std.testing.expectEqual(@as(u16, 8080), rec.?.port);
}

test "kernel: events emitted for module lifecycle" {
    const k = try makeKernel(std.testing.allocator);
    defer { k.deinit(); std.testing.allocator.destroy(k); }

    try k.registerModule(.host, "evtest.mod", "test");
    try k.startModule(.host, "evtest.mod");
    try k.stopModule(.host, "evtest.mod");

    // At least 3 events: registered, started, stopped
    const s = k.stats();
    try std.testing.expect(s.kernel_events_total >= 3);
}

test "kernel: events emitted for process lifecycle" {
    const k = try makeKernel(std.testing.allocator);
    defer { k.deinit(); std.testing.allocator.destroy(k); }

    const pid = try k.spawnProcess(.host, "worker", .module_runtime);
    try k.terminateProcess(.host, pid);

    const s = k.stats();
    try std.testing.expect(s.kernel_events_total >= 2);
}

test "kernel: events emitted for memory allocation" {
    const k = try makeKernel(std.testing.allocator);
    defer { k.deinit(); std.testing.allocator.destroy(k); }

    try k.registerTenant(.host, "memtest", .{ .memory_bytes = 1024 * 1024 });
    try k.allocateTenantMemory(.host, "memtest", 512 * 1024);
    try k.freeTenantMemory(.host, "memtest", 512 * 1024);

    const f = evt_mod.EventFilter.forDomain(.memory);
    var out = std.ArrayList(evt_mod.KernelEvent).init(std.testing.allocator);
    defer out.deinit();
    try k.queryEvents(&f, 0, &out);
    try std.testing.expect(out.items.len >= 2);
}

test "kernel: subscribeEvents receives filtered events" {
    const S = struct {
        var count: usize = 0;
        fn handler(_: ?*anyopaque, _: *const evt_mod.KernelEvent) void { count += 1; }
    };
    S.count = 0;

    const k = try makeKernel(std.testing.allocator);
    defer { k.deinit(); std.testing.allocator.destroy(k); }

    const f = evt_mod.EventFilter.forDomain(.module);
    try k.subscribeEvents(.host, "mod-watcher", f, S.handler, null);

    try k.registerModule(.host, "sub.mod", "test");
    try k.startModule(.host, "sub.mod");
    try k.stopModule(.host, "sub.mod");

    // registered + started + stopped = 3 module-domain events
    try std.testing.expect(S.count >= 3);
}

test "kernel: suspendEventDispatch halts delivery but not logging" {
    const S = struct {
        var count: usize = 0;
        fn handler(_: ?*anyopaque, _: *const evt_mod.KernelEvent) void { count += 1; }
    };
    S.count = 0;

    const k = try makeKernel(std.testing.allocator);
    defer { k.deinit(); std.testing.allocator.destroy(k); }

    try k.subscribeEvents(.host, "paused-watcher", evt_mod.EventFilter.matchesAll(), S.handler, null);
    try k.suspendEventDispatch(.root);

    try k.registerModule(.host, "bg.mod", "bg");
    try k.startModule(.host, "bg.mod");

    // No delivery while suspended
    try std.testing.expectEqual(@as(usize, 0), S.count);

    // But events were still logged
    try std.testing.expect(k.stats().kernel_events_total >= 2);

    // Resume and confirm future events are delivered
    try k.resumeEventDispatch(.root);
    try k.stopModule(.host, "bg.mod");
    try std.testing.expect(S.count >= 1);
}

test "kernel: queryEvents pages via watermark" {
    const k = try makeKernel(std.testing.allocator);
    defer { k.deinit(); std.testing.allocator.destroy(k); }

    try k.registerModule(.host, "page.a", "a");
    const mark = k.stats().kernel_events_total;
    try k.registerModule(.host, "page.b", "b");
    try k.registerModule(.host, "page.c", "c");

    var out = std.ArrayList(evt_mod.KernelEvent).init(std.testing.allocator);
    defer out.deinit();
    try k.queryEvents(null, mark, &out);

    // Events for page.b and page.c only
    try std.testing.expect(out.items.len >= 2);
}

test "kernel: latestEvent returns most recent published event" {
    const k = try makeKernel(std.testing.allocator);
    defer { k.deinit(); std.testing.allocator.destroy(k); }

    try std.testing.expect(k.latestEvent() == null);

    try k.setMode(.user, .root);
    const ev = k.latestEvent();
    try std.testing.expect(ev != null);
    try std.testing.expectEqual(evt_mod.EventDomain.kernel, ev.?.domain);
    try std.testing.expectEqual(evt_mod.EventKind.kernel_mode_changed, ev.?.kind);
}

test "kernel: publishEvent allows custom cross-cutting events" {
    const k = try makeKernel(std.testing.allocator);
    defer { k.deinit(); std.testing.allocator.destroy(k); }

    try k.publishEvent(.host, "integration-test", "{\\"msg\\":\\"hello\\"}");
    const ev = k.latestEvent().?;
    try std.testing.expectEqual(evt_mod.EventDomain.custom, ev.domain);
    try std.testing.expectEqualStrings("integration-test", ev.sourceSlice());
}

test "kernel: aggregate stats include event counters" {
    const k = try makeKernel(std.testing.allocator);
    defer { k.deinit(); std.testing.allocator.destroy(k); }

    try k.bootstrapCorePlatformComponents(.host);
    const s = k.stats();
    try std.testing.expect(s.kernel_events_total > 0);
    try std.testing.expect(s.kernel_events_log_len > 0);
    try std.testing.expectEqual(@as(u64, 0), s.kernel_events_overflow);
}

test "kernel: enterPrivileged and exitPrivileged" {
    const k = try makeKernel(std.testing.allocator);
    defer { k.deinit(); std.testing.allocator.destroy(k); }

    // Start in privileged mode by default
    try std.testing.expect(k.isPrivileged());
    try std.testing.expectEqual(Role.root, k.session_role);

    // Drop to user mode
    try k.exitPrivileged(.root);
    try std.testing.expect(!k.isPrivileged());
    try std.testing.expectEqual(Role.user, k.session_role);

    // Escalate back
    try k.enterPrivileged(.host);
    try std.testing.expect(k.isPrivileged());
    try std.testing.expectEqual(Role.host, k.session_role);
}

test "kernel: PrivilegeDenied — user role cannot enter privileged mode" {
    const k = try makeKernel(std.testing.allocator);
    defer { k.deinit(); std.testing.allocator.destroy(k); }

    try std.testing.expectError(KernelError.PrivilegeDenied, k.enterPrivileged(.user));
    try std.testing.expectError(KernelError.PrivilegeDenied, k.enterPrivileged(.module_runtime));
}

test "kernel: enforcePrivileged blocks ops when in user mode" {
    const k = try makeKernel(std.testing.allocator);
    defer { k.deinit(); std.testing.allocator.destroy(k); }

    try k.exitPrivileged(.root);

    // While in user mode, privileged ops must return PrivilegeDenied
    try std.testing.expectError(
        KernelError.PrivilegeDenied,
        k.registerModule(.host, "blocked.mod", "test"),
    );
    try std.testing.expectError(
        KernelError.PrivilegeDenied,
        k.allocateTenantMemory(.host, "t", 1024),
    );

    // Re-enter privileged and confirm ops work again
    try k.enterPrivileged(.host);
    try k.registerModule(.host, "ok.mod", "test");
    try std.testing.expectEqual(@as(usize, 1), k.modules.registry.count());
}

test "kernel: user-mode stats and queries still work without privilege" {
    const k = try makeKernel(std.testing.allocator);
    defer { k.deinit(); std.testing.allocator.destroy(k); }

    try k.registerModule(.host, "stat.mod", "test");
    try k.exitPrivileged(.root);

    // stats() is always available
    const s = k.stats();
    try std.testing.expectEqual(Mode.user, s.mode);
    try std.testing.expectEqual(Role.user, s.session_role);
    try std.testing.expectEqual(@as(usize, 1), s.total_modules);

    // latestEvent() is always available
    try std.testing.expect(k.latestEvent() != null);
}
