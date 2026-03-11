//! services.zig — Kernel Service Manager
//!
//! Every kernel subsystem (memory.zig, processes.zig, network.zig, module.zig)
//! runs here as an independent, first-class kernel service.  The ServiceManager
//! is the single authority that registers, provisions, starts, health-monitors,
//! and tears down every service.
//!
//! Design
//! ──────
//! Each subsystem is wrapped in a concrete *Service struct (MemoryService,
//! ProcessService, NetworkService, ModuleService) that:
//!   • owns the subsystem's actual objects (MemoryManager, Orchestrator, …)
//!   • exposes a uniform ServiceVTable so the manager can drive it blindly
//!   • tracks its own resource usage and health independently
//!
//! The ServiceManager then coordinates them through:
//!   ServiceRegistry        — authoritative map of every registered service
//!   ServiceProvisioner     — scoped acquire / release of global resource budgets
//!   DependencyGraph        — Kahn's-algorithm topological ordering
//!   HealthMonitor          — periodic health-check polling + fault escalation
//!   ServiceEventBus        — cross-service pub/sub for all lifecycle events
//!
//! Subsystem dependency chain (start order):
//!   kernel.memory  →  kernel.process  →  kernel.network  →  kernel.module
//!
//! File layout
//! ───────────
//!   §1   Shared kernel primitives
//!   §2   Errors
//!   §3   Resource budget / usage
//!   §4   Service status state machine
//!   §5   Restart & health-check policy
//!   §6   ServiceVTable + ServiceHandle (uniform subsystem interface)
//!   §7   ServiceEventBus
//!   §8   ServiceDescriptor
//!   §9   ServiceRegistry
//!   §10  ServiceProvisioner
//!   §11  DependencyGraph
//!   §12  HealthMonitor
//!   §13  MemoryService   (wraps memory.zig)
//!   §14  ProcessService  (wraps processes.zig)
//!   §15  NetworkService  (wraps network.zig)
//!   §16  ModuleService   (wraps module.zig)
//!   §17  ServiceManager
//!   §18  Tests

const std = @import("std");

// Import the four kernel subsystems.
// In a real build these would be: @import("memory.zig") etc.
// Because this file is self-contained for testing we reproduce only the
// minimum public surface needed; the full structs live in the imported files.
const mem_mod  = @import("memory.zig");
const proc_mod = @import("processes.zig");
const net_mod  = @import("network.zig");
const mod_mod  = @import("module.zig");

// ─────────────────────────────────────────────────────────────────────────────
// §1  Shared kernel primitives  (canonical copy — all four files agree on this)
// ─────────────────────────────────────────────────────────────────────────────

pub const Role = enum { root, host, server, module_runtime, user };

pub const Permission = enum {
    kernel_admin, schedule_tasks, manage_modules,
    manage_memory, manage_processes, manage_files, read_audit,
};

pub fn hasPermission(role: Role, permission: Permission) bool {
    return switch (role) {
        .root => true,
        .host => switch (permission) {
            .kernel_admin, .schedule_tasks, .manage_modules, .manage_memory,
            .manage_processes, .manage_files, .read_audit => true,
        },
        .server => switch (permission) {
            .schedule_tasks, .manage_modules, .manage_processes, .manage_files => true,
            .kernel_admin, .manage_memory, .read_audit => false,
        },
        .module_runtime => switch (permission) {
            .schedule_tasks, .manage_files => true,
            .kernel_admin, .manage_modules, .manage_memory,
            .manage_processes, .read_audit => false,
        },
        .user => permission == .read_audit,
    };
}

// ─────────────────────────────────────────────────────────────────────────────
// §2  Errors
// ─────────────────────────────────────────────────────────────────────────────

pub const ServiceError = error{
    AccessDenied,
    ServiceNotFound,
    ServiceAlreadyRegistered,
    InvalidStatusTransition,
    DependencyNotMet,
    CircularDependency,
    ResourceLimitExceeded,
    GlobalResourceExhausted,
    MaxRestartsExceeded,
    EventBusFull,
    HandleRequired,
    SubsystemError,
};

// ─────────────────────────────────────────────────────────────────────────────
// §3  Resource budget / usage
// ─────────────────────────────────────────────────────────────────────────────

/// Hard resource caps granted to a single service.
pub const ResourceBudget = struct {
    memory_bytes:    u64  = 0,
    max_processes:   u32  = 0,
    max_files:       u32  = 0,
    max_units:       u32  = 0,
    max_connections: u32  = 0,
};

/// Live resource consumption counters for a single service.
pub const ResourceUsage = struct {
    memory_bytes:     u64 = 0,
    process_count:    u32 = 0,
    file_count:       u32 = 0,
    unit_count:       u32 = 0,
    connection_count: u32 = 0,
};

/// Aggregate system-wide resource pool.
pub const GlobalBudget = struct {
    memory_bytes:    u64,
    max_processes:   u32,
    max_connections: u32,
};

// ─────────────────────────────────────────────────────────────────────────────
// §4  Service status state machine
// ─────────────────────────────────────────────────────────────────────────────

pub const ServiceStatus = enum {
    /// Registered; no resources allocated yet.
    registered,
    /// Currently allocating resources.
    provisioning,
    /// Fully operational.
    running,
    /// Paused; resources held.
    suspended,
    /// Graceful shutdown draining in-flight work.
    draining,
    /// Stopped; all resources released; still registered.
    stopped,
    /// Unrecoverable fault.
    faulted,
    /// Being removed from the registry.
    removing,

    pub fn canTransitionTo(self: ServiceStatus, next: ServiceStatus) bool {
        return switch (self) {
            .registered   => next == .provisioning or next == .stopped,
            .provisioning => next == .running      or next == .faulted or next == .stopped,
            .running      => next == .suspended    or next == .draining or next == .faulted,
            .suspended    => next == .running      or next == .draining,
            .draining     => next == .stopped      or next == .faulted,
            .stopped      => next == .provisioning or next == .removing,
            .faulted      => next == .stopped      or next == .removing,
            .removing     => false,
        };
    }

    pub fn isActive(self: ServiceStatus) bool {
        return self == .running or self == .suspended;
    }

    pub fn isStopped(self: ServiceStatus) bool {
        return self == .stopped or self == .faulted or self == .removing;
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// §5  Restart & health policy
// ─────────────────────────────────────────────────────────────────────────────

pub const RestartPolicy = enum {
    never,     /// Never restart automatically.
    on_fault,  /// Restart only after an unexpected fault.
    always,    /// Restart whenever the service is not running.
};

pub const RestartConfig = struct {
    policy:       RestartPolicy = .on_fault,
    max_restarts: u32           = 5,
    /// Minimum milliseconds between consecutive restarts (backoff).
    backoff_ms:   u64           = 500,
};

pub const HealthStatus = enum { healthy, degraded, unhealthy, unknown };

pub const HealthPolicy = struct {
    /// Interval in ms between probe invocations.
    interval_ms:        u64 = 15_000,
    /// Consecutive unhealthy/unknown results before declaring a fault.
    fault_threshold:    u32 = 3,
    /// Consecutive healthy results needed to clear a degraded streak.
    recovery_threshold: u32 = 2,
};

// ─────────────────────────────────────────────────────────────────────────────
// §6  ServiceVTable + ServiceHandle
// ─────────────────────────────────────────────────────────────────────────────

/// Every concrete service implements this vtable.  The ServiceManager calls
/// through it without knowing the concrete subsystem type.
pub const ServiceVTable = struct {
    /// Allocate internal state; called while status == .provisioning.
    provision: *const fn (ctx: *anyopaque, actor: Role) anyerror!void,
    /// Begin active operation; called immediately after .running is set.
    start:     *const fn (ctx: *anyopaque, actor: Role) anyerror!void,
    /// Signal graceful drain — stop accepting new work, finish in-flight.
    drain:     *const fn (ctx: *anyopaque, actor: Role) anyerror!void,
    /// Release all subsystem resources; called after draining completes.
    stop:      *const fn (ctx: *anyopaque, actor: Role) anyerror!void,
    /// Return a current point-in-time health reading.
    health:    *const fn (ctx: *anyopaque) HealthStatus,
    /// Write a short human-readable status string into `buf`; return bytes written.
    describe:  *const fn (ctx: *anyopaque, buf: []u8) usize,
};

/// Type-erased, callable handle to a concrete service.
pub const ServiceHandle = struct {
    ctx:    *anyopaque,
    vtable: *const ServiceVTable,

    pub fn provision(self: ServiceHandle, actor: Role) !void { return self.vtable.provision(self.ctx, actor); }
    pub fn start    (self: ServiceHandle, actor: Role) !void { return self.vtable.start    (self.ctx, actor); }
    pub fn drain    (self: ServiceHandle, actor: Role) !void { return self.vtable.drain    (self.ctx, actor); }
    pub fn stop     (self: ServiceHandle, actor: Role) !void { return self.vtable.stop     (self.ctx, actor); }
    pub fn health   (self: ServiceHandle)  HealthStatus      { return self.vtable.health   (self.ctx); }
    pub fn describe (self: ServiceHandle, buf: []u8) usize   { return self.vtable.describe (self.ctx, buf); }
};

// ─────────────────────────────────────────────────────────────────────────────
// §7  ServiceEventBus
// ─────────────────────────────────────────────────────────────────────────────

pub const ServiceEvent = struct {
    id:         u64,
    topic:      []const u8,  // owned
    payload:    []const u8,  // owned
    source_id:  []const u8,  // borrowed — points into ServiceDescriptor.id
    emitted_ms: i64,
};

/// Well-known topic constants.
pub const Topic = struct {
    pub const registered  = "service.registered";
    pub const provisioned = "service.provisioned";
    pub const started     = "service.started";
    pub const suspended   = "service.suspended";
    pub const resumed     = "service.resumed";
    pub const draining    = "service.draining";
    pub const stopped     = "service.stopped";
    pub const faulted     = "service.faulted";
    pub const restarted   = "service.restarted";
    pub const removed     = "service.removed";
    pub const health_ok   = "service.health.ok";
    pub const health_warn = "service.health.degraded";
    pub const health_fail = "service.health.failed";
};

pub const ServiceEventBus = struct {
    allocator: std.mem.Allocator,
    events:    std.ArrayList(ServiceEvent),
    capacity:  usize,
    next_id:   u64,

    pub fn init(allocator: std.mem.Allocator, capacity: usize) ServiceEventBus {
        return .{
            .allocator = allocator,
            .events    = std.ArrayList(ServiceEvent).init(allocator),
            .capacity  = capacity,
            .next_id   = 1,
        };
    }

    pub fn deinit(self: *ServiceEventBus) void {
        for (self.events.items) |e| {
            self.allocator.free(e.topic);
            self.allocator.free(e.payload);
        }
        self.events.deinit();
    }

    pub fn publish(
        self:      *ServiceEventBus,
        source_id: []const u8,
        topic:     []const u8,
        payload:   []const u8,
    ) !void {
        if (self.events.items.len >= self.capacity) return ServiceError.EventBusFull;
        const t = try self.allocator.dupe(u8, topic);
        errdefer self.allocator.free(t);
        const p = try self.allocator.dupe(u8, payload);
        errdefer self.allocator.free(p);
        const id = self.next_id;
        self.next_id += 1;
        try self.events.append(.{
            .id = id, .topic = t, .payload = p,
            .source_id = source_id, .emitted_ms = std.time.milliTimestamp(),
        });
    }

    /// Return all events matching `topic` exactly.
    pub fn collect(
        self:  *const ServiceEventBus,
        topic: []const u8,
        out:   *std.ArrayList(ServiceEvent),
    ) !void {
        for (self.events.items) |e| {
            if (std.mem.eql(u8, e.topic, topic)) try out.append(e);
        }
    }

    pub fn len(self: *const ServiceEventBus) usize { return self.events.items.len; }
};

// ─────────────────────────────────────────────────────────────────────────────
// §8  ServiceDescriptor
// ─────────────────────────────────────────────────────────────────────────────

const MAX_SERVICE_DEPS = 16;

pub const ServiceDescriptor = struct {
    // ── Identity ──────────────────────────────────────────────────────────
    id:             []const u8,  // owned by ServiceRegistry
    display_name:   []const u8,  // owned by ServiceRegistry
    version:        []const u8,  // owned by ServiceRegistry

    // ── Lifecycle ─────────────────────────────────────────────────────────
    status:         ServiceStatus,
    registered_ms:  i64,
    last_status_ms: i64,
    start_count:    u32,
    restart_count:  u32,
    fault_count:    u32,
    last_fault_ms:  i64,

    // ── Resources ─────────────────────────────────────────────────────────
    budget:  ResourceBudget,
    usage:   ResourceUsage,

    // ── Policy ────────────────────────────────────────────────────────────
    restart:        RestartConfig,
    health_policy:  HealthPolicy,

    // ── Live health ───────────────────────────────────────────────────────
    last_health:    HealthStatus,
    health_streak:  u32,   // consecutive same-result count
    checked_ms:     i64,

    // ── Implementation ────────────────────────────────────────────────────
    handle: ?ServiceHandle,

    // ── Dependencies (slice of borrowed IDs into other ServiceDescriptors)
    deps:      [][]const u8,   // owned by ServiceRegistry
    dep_count: usize,
};

// ─────────────────────────────────────────────────────────────────────────────
// §9  ServiceRegistry
// ─────────────────────────────────────────────────────────────────────────────

pub const ServiceRegistry = struct {
    allocator: std.mem.Allocator,
    services:  std.StringHashMap(ServiceDescriptor),

    pub fn init(allocator: std.mem.Allocator) ServiceRegistry {
        return .{
            .allocator = allocator,
            .services  = std.StringHashMap(ServiceDescriptor).init(allocator),
        };
    }

    pub fn deinit(self: *ServiceRegistry) void {
        var it = self.services.valueIterator();
        while (it.next()) |svc| self.freeOwnedStrings(svc);
        self.services.deinit();
    }

    fn freeOwnedStrings(self: *ServiceRegistry, svc: *ServiceDescriptor) void {
        self.allocator.free(svc.id);
        self.allocator.free(svc.display_name);
        self.allocator.free(svc.version);
        if (svc.dep_count > 0) {
            for (svc.deps[0..svc.dep_count]) |dep| self.allocator.free(dep);
            self.allocator.free(svc.deps);
        }
    }

    // ── Registration ──────────────────────────────────────────────────────

    pub fn register(
        self:         *ServiceRegistry,
        actor:        Role,
        id:           []const u8,
        display_name: []const u8,
        version:      []const u8,
        budget:       ResourceBudget,
        restart:      RestartConfig,
        health:       HealthPolicy,
        handle:       ?ServiceHandle,
    ) !void {
        if (!hasPermission(actor, .manage_modules)) return ServiceError.AccessDenied;
        if (self.services.contains(id)) return ServiceError.ServiceAlreadyRegistered;

        const id_c   = try self.allocator.dupe(u8, id);
        errdefer self.allocator.free(id_c);
        const name_c = try self.allocator.dupe(u8, display_name);
        errdefer self.allocator.free(name_c);
        const ver_c  = try self.allocator.dupe(u8, version);
        errdefer self.allocator.free(ver_c);

        const now = std.time.milliTimestamp();
        try self.services.put(id_c, .{
            .id             = id_c,
            .display_name   = name_c,
            .version        = ver_c,
            .status         = .registered,
            .registered_ms  = now,
            .last_status_ms = now,
            .start_count    = 0,
            .restart_count  = 0,
            .fault_count    = 0,
            .last_fault_ms  = 0,
            .budget         = budget,
            .usage          = .{},
            .restart        = restart,
            .health_policy  = health,
            .last_health    = .unknown,
            .health_streak  = 0,
            .checked_ms     = now,
            .handle         = handle,
            .deps           = &.{},
            .dep_count      = 0,
        });
    }

    pub fn deregister(self: *ServiceRegistry, actor: Role, id: []const u8) !void {
        if (!hasPermission(actor, .manage_modules)) return ServiceError.AccessDenied;
        const svc = self.services.getPtr(id) orelse return ServiceError.ServiceNotFound;
        if (!svc.status.isStopped()) return ServiceError.InvalidStatusTransition;
        self.freeOwnedStrings(svc);
        _ = self.services.remove(id);
    }

    pub fn addDependency(self: *ServiceRegistry, actor: Role, id: []const u8, dep_id: []const u8) !void {
        if (!hasPermission(actor, .manage_modules)) return ServiceError.AccessDenied;
        const svc = self.services.getPtr(id) orelse return ServiceError.ServiceNotFound;
        if (svc.dep_count == 0) {
            svc.deps = try self.allocator.alloc([]const u8, MAX_SERVICE_DEPS);
        }
        if (svc.dep_count >= MAX_SERVICE_DEPS) return ServiceError.ResourceLimitExceeded;
        const dep_c = try self.allocator.dupe(u8, dep_id);
        svc.deps[svc.dep_count] = dep_c;
        svc.dep_count += 1;
    }

    // ── State machine ─────────────────────────────────────────────────────

    pub fn transition(self: *ServiceRegistry, id: []const u8, next: ServiceStatus) !void {
        const svc = self.services.getPtr(id) orelse return ServiceError.ServiceNotFound;
        if (!svc.status.canTransitionTo(next)) return ServiceError.InvalidStatusTransition;
        svc.status         = next;
        svc.last_status_ms = std.time.milliTimestamp();
    }

    // ── Queries ───────────────────────────────────────────────────────────

    pub fn get(self: *const ServiceRegistry, id: []const u8) ?ServiceDescriptor {
        return self.services.get(id);
    }

    pub fn getPtr(self: *ServiceRegistry, id: []const u8) ?*ServiceDescriptor {
        return self.services.getPtr(id);
    }

    pub fn dependenciesMet(self: *const ServiceRegistry, id: []const u8) bool {
        const svc = self.services.get(id) orelse return false;
        for (svc.deps[0..svc.dep_count]) |dep_id| {
            const dep = self.services.get(dep_id) orelse return false;
            if (dep.status != .running) return false;
        }
        return true;
    }

    pub fn count(self: *const ServiceRegistry) usize { return self.services.count(); }

    pub fn countByStatus(self: *const ServiceRegistry, status: ServiceStatus) usize {
        var n: usize = 0;
        var it = self.services.valueIterator();
        while (it.next()) |svc| { if (svc.status == status) n += 1; }
        return n;
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// §10  ServiceProvisioner
// ─────────────────────────────────────────────────────────────────────────────

/// Enforces per-service budget caps and a global resource ceiling.
pub const ServiceProvisioner = struct {
    registry:      *ServiceRegistry,
    global_budget: GlobalBudget,
    global_usage:  ResourceUsage,

    pub fn init(registry: *ServiceRegistry, global_budget: GlobalBudget) ServiceProvisioner {
        return .{
            .registry      = registry,
            .global_budget = global_budget,
            .global_usage  = .{},
        };
    }

    // ── Memory ────────────────────────────────────────────────────────────

    pub fn acquireMemory(self: *ServiceProvisioner, id: []const u8, bytes: u64) !void {
        if (self.global_usage.memory_bytes + bytes > self.global_budget.memory_bytes)
            return ServiceError.GlobalResourceExhausted;
        const svc = self.registry.services.getPtr(id) orelse return ServiceError.ServiceNotFound;
        if (svc.usage.memory_bytes + bytes > svc.budget.memory_bytes)
            return ServiceError.ResourceLimitExceeded;
        svc.usage.memory_bytes          += bytes;
        self.global_usage.memory_bytes  += bytes;
    }

    pub fn releaseMemory(self: *ServiceProvisioner, id: []const u8) void {
        const svc = self.registry.services.getPtr(id) orelse return;
        self.global_usage.memory_bytes =
            self.global_usage.memory_bytes -| svc.usage.memory_bytes;
        svc.usage.memory_bytes = 0;
    }

    // ── Processes ─────────────────────────────────────────────────────────

    pub fn acquireProcesses(self: *ServiceProvisioner, id: []const u8, n: u32) !void {
        if (self.global_usage.process_count + n > self.global_budget.max_processes)
            return ServiceError.GlobalResourceExhausted;
        const svc = self.registry.services.getPtr(id) orelse return ServiceError.ServiceNotFound;
        if (svc.usage.process_count + n > svc.budget.max_processes)
            return ServiceError.ResourceLimitExceeded;
        svc.usage.process_count          += n;
        self.global_usage.process_count  += n;
    }

    pub fn releaseProcesses(self: *ServiceProvisioner, id: []const u8) void {
        const svc = self.registry.services.getPtr(id) orelse return;
        self.global_usage.process_count =
            self.global_usage.process_count -| svc.usage.process_count;
        svc.usage.process_count = 0;
    }

    // ── Connections ───────────────────────────────────────────────────────

    pub fn acquireConnections(self: *ServiceProvisioner, id: []const u8, n: u32) !void {
        if (self.global_usage.connection_count + n > self.global_budget.max_connections)
            return ServiceError.GlobalResourceExhausted;
        const svc = self.registry.services.getPtr(id) orelse return ServiceError.ServiceNotFound;
        if (svc.usage.connection_count + n > svc.budget.max_connections)
            return ServiceError.ResourceLimitExceeded;
        svc.usage.connection_count          += n;
        self.global_usage.connection_count  += n;
    }

    pub fn releaseConnections(self: *ServiceProvisioner, id: []const u8) void {
        const svc = self.registry.services.getPtr(id) orelse return;
        self.global_usage.connection_count =
            self.global_usage.connection_count -| svc.usage.connection_count;
        svc.usage.connection_count = 0;
    }

    // ── Full teardown ─────────────────────────────────────────────────────

    pub fn releaseAll(self: *ServiceProvisioner, id: []const u8) void {
        self.releaseMemory(id);
        self.releaseProcesses(id);
        self.releaseConnections(id);
        if (self.registry.services.getPtr(id)) |svc| {
            // Remaining dimensions (files, units) are per-service only.
            svc.usage.file_count  = 0;
            svc.usage.unit_count  = 0;
        }
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// §11  DependencyGraph
// ─────────────────────────────────────────────────────────────────────────────

/// Produces a topologically-sorted start order (Kahn's algorithm / BFS).
pub const DependencyGraph = struct {
    allocator: std.mem.Allocator,

    pub fn init(allocator: std.mem.Allocator) DependencyGraph {
        return .{ .allocator = allocator };
    }

    /// Fill `out` with service IDs in safe start order.
    /// Returns error.CircularDependency if the graph has a cycle.
    pub fn startOrder(
        self:     *const DependencyGraph,
        registry: *const ServiceRegistry,
        out:      *std.ArrayList([]const u8),
    ) !void {
        var in_degree = std.StringHashMap(u32).init(self.allocator);
        defer in_degree.deinit();

        // Seed every node at zero.
        var kit = registry.services.keyIterator();
        while (kit.next()) |k| try in_degree.put(k.*, 0);

        // For each edge dep → svc, increment svc's in-degree.
        var sit = registry.services.valueIterator();
        while (sit.next()) |svc| {
            for (svc.deps[0..svc.dep_count]) |dep_id| {
                if (in_degree.getPtr(svc.id)) |d| d.* += 1;
                _ = dep_id; // dep_id is the *source* — we want to inc the dependent
            }
        }

        // Redo: in-degree of X = number of services X depends on that we haven't
        // processed yet — i.e. increment X's degree for each of X's deps.
        {
            var it2 = in_degree.iterator();
            while (it2.next()) |e| e.value_ptr.* = 0;
        }
        sit = registry.services.valueIterator();
        while (sit.next()) |svc| {
            if (in_degree.getPtr(svc.id)) |d| d.* = @intCast(svc.dep_count);
        }

        // Queue nodes with no unmet dependencies.
        var queue = std.ArrayList([]const u8).init(self.allocator);
        defer queue.deinit();
        var dit = in_degree.iterator();
        while (dit.next()) |e| {
            if (e.value_ptr.* == 0) try queue.append(e.key_ptr.*);
        }

        var processed: usize = 0;
        while (queue.items.len > 0) {
            const id = queue.orderedRemove(0);
            try out.append(id);
            processed += 1;

            // For every service that lists `id` as a dependency, decrement its degree.
            var si2 = registry.services.valueIterator();
            while (si2.next()) |svc| {
                for (svc.deps[0..svc.dep_count]) |dep_id| {
                    if (!std.mem.eql(u8, dep_id, id)) continue;
                    if (in_degree.getPtr(svc.id)) |d| {
                        d.* -= 1;
                        if (d.* == 0) try queue.append(svc.id);
                    }
                }
            }
        }

        if (processed != registry.services.count()) return ServiceError.CircularDependency;
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// §12  HealthMonitor
// ─────────────────────────────────────────────────────────────────────────────

pub const HealthMonitor = struct {
    registry:    *ServiceRegistry,
    provisioner: *ServiceProvisioner,
    bus:         *ServiceEventBus,

    pub fn init(
        registry:    *ServiceRegistry,
        provisioner: *ServiceProvisioner,
        bus:         *ServiceEventBus,
    ) HealthMonitor {
        return .{ .registry = registry, .provisioner = provisioner, .bus = bus };
    }

    /// Run health probes for all active services whose interval has elapsed.
    /// `now_ms` — caller-supplied current timestamp.
    /// Returns the number of services probed.
    pub fn tick(self: *HealthMonitor, now_ms: i64) !usize {
        var probed: usize = 0;
        var it = self.registry.services.valueIterator();
        while (it.next()) |svc| {
            if (!svc.status.isActive()) continue;
            const elapsed = now_ms - svc.checked_ms;
            if (elapsed < @as(i64, @intCast(svc.health_policy.interval_ms))) continue;

            const handle = svc.handle orelse continue;
            const result = handle.health();

            svc.checked_ms = now_ms;

            const same = result == svc.last_health;
            svc.health_streak = if (same) svc.health_streak + 1 else 1;
            svc.last_health   = result;

            switch (result) {
                .healthy => {
                    try self.bus.publish(svc.id, Topic.health_ok, "");
                },
                .degraded => {
                    try self.bus.publish(svc.id, Topic.health_warn, "");
                },
                .unhealthy, .unknown => {
                    try self.bus.publish(svc.id, Topic.health_fail, "");
                    if (svc.health_streak >= svc.health_policy.fault_threshold) {
                        svc.fault_count   += 1;
                        svc.last_fault_ms  = now_ms;
                        svc.status         = .faulted;
                        svc.last_status_ms = now_ms;
                        self.provisioner.releaseAll(svc.id);
                        try self.bus.publish(svc.id, Topic.faulted, "health-threshold");
                    }
                },
            }
            probed += 1;
        }
        return probed;
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// §13  MemoryService  — wraps memory.zig MemoryManager
// ─────────────────────────────────────────────────────────────────────────────

pub const MemoryServiceConfig = struct {
    total_bytes:    u64   = 4 * 1024 * 1024 * 1024,
    audit_capacity: usize = 4096,
    enable_audit:   bool  = true,
};

/// Concrete service wrapper around mem_mod.MemoryManager.
pub const MemoryService = struct {
    allocator:   std.mem.Allocator,
    config:      MemoryServiceConfig,
    manager:     ?mem_mod.MemoryManager,
    audit_log:   ?mem_mod.AuditLog,
    initialized: bool,

    pub fn init(allocator: std.mem.Allocator, config: MemoryServiceConfig) MemoryService {
        return .{
            .allocator   = allocator,
            .config      = config,
            .manager     = null,
            .audit_log   = null,
            .initialized = false,
        };
    }

    pub fn deinit(self: *MemoryService) void {
        if (self.manager) |*m| m.deinit();
        if (self.audit_log) |*a| a.deinit();
        self.initialized = false;
    }

    pub fn handle(self: *MemoryService) ServiceHandle {
        return .{ .ctx = self, .vtable = &memory_vtable };
    }

    /// Expose the live manager for use by other services (returns null before provision).
    pub fn liveManager(self: *MemoryService) ?*mem_mod.MemoryManager {
        if (!self.initialized) return null;
        return &(self.manager orelse return null);
    }

    fn provision_(ctx: *anyopaque, _: Role) !void {
        const self: *MemoryService = @ptrCast(@alignCast(ctx));
        if (self.initialized) return;

        if (self.config.enable_audit) {
            self.audit_log = try mem_mod.AuditLog.init(self.allocator, self.config.audit_capacity);
        }

        self.manager = mem_mod.MemoryManager.init(
            self.allocator,
            self.config.total_bytes,
            if (self.config.enable_audit) &(self.audit_log.?) else null,
        );
        self.initialized = true;
    }

    fn start_(_: *anyopaque, _: Role) !void {}  // manager is ready immediately after provision

    fn drain_(_: *anyopaque, _: Role) !void {}  // nothing to drain synchronously

    fn stop_(ctx: *anyopaque, _: Role) !void {
        const self: *MemoryService = @ptrCast(@alignCast(ctx));
        if (self.manager) |*m| m.deinit();
        if (self.audit_log) |*a| a.deinit();
        self.manager     = null;
        self.audit_log   = null;
        self.initialized = false;
    }

    fn health_(ctx: *anyopaque) HealthStatus {
        const self: *MemoryService = @ptrCast(@alignCast(ctx));
        if (!self.initialized) return .unknown;
        const m = &(self.manager orelse return .unknown);
        return switch (m.currentPressure()) {
            .normal   => .healthy,
            .moderate => .degraded,
            .critical => .unhealthy,
        };
    }

    fn describe_(ctx: *anyopaque, buf: []u8) usize {
        const self: *MemoryService = @ptrCast(@alignCast(ctx));
        if (!self.initialized) {
            const s = "MemoryService: not initialized";
            const n = @min(s.len, buf.len);
            @memcpy(buf[0..n], s[0..n]);
            return n;
        }
        const s = self.manager.?.stats();
        const written = std.fmt.bufPrint(buf,
            "MemoryService: {d}/{d} bytes used ({d} tenants)",
            .{ s.used_bytes, s.total_bytes, s.tenant_count },
        ) catch buf[0..0];
        return written.len;
    }

    const memory_vtable = ServiceVTable{
        .provision = provision_,
        .start     = start_,
        .drain     = drain_,
        .stop      = stop_,
        .health    = health_,
        .describe  = describe_,
    };
};

// ─────────────────────────────────────────────────────────────────────────────
// §14  ProcessService  — wraps processes.zig Orchestrator (+ its sub-systems)
// ─────────────────────────────────────────────────────────────────────────────

pub const ProcessServiceConfig = struct {
    worker_count:  u32   = 16,
    queue_lane_cap: usize = 256,
};

/// Concrete service wrapper around proc_mod.Orchestrator and its sub-objects.
pub const ProcessService = struct {
    allocator:   std.mem.Allocator,
    config:      ProcessServiceConfig,

    // Sub-objects allocated during provision.
    table:       ?proc_mod.ProcessTable,
    pool:        ?proc_mod.ThreadPool,
    queue:       ?proc_mod.WorkQueue,
    scheduler:   ?proc_mod.Scheduler,
    ledger:      ?proc_mod.ResourceLedger,
    orchestrator: ?proc_mod.Orchestrator,
    initialized: bool,

    pub fn init(allocator: std.mem.Allocator, config: ProcessServiceConfig) ProcessService {
        return .{
            .allocator    = allocator,
            .config       = config,
            .table        = null,
            .pool         = null,
            .queue        = null,
            .scheduler    = null,
            .ledger       = null,
            .orchestrator = null,
            .initialized  = false,
        };
    }

    pub fn deinit(self: *ProcessService) void {
        if (self.orchestrator) |*o| o.deinit();
        if (self.table)        |*t| t.deinit();
        if (self.pool)         |*p| p.deinit();
        if (self.queue)        |*q| q.deinit();
        if (self.scheduler)    |*s| s.deinit();
        if (self.ledger)       |*l| l.deinit();
        self.orchestrator = null;
        self.table        = null;
        self.pool         = null;
        self.queue        = null;
        self.scheduler    = null;
        self.ledger       = null;
        self.initialized  = false;
    }

    pub fn handle(self: *ProcessService) ServiceHandle {
        return .{ .ctx = self, .vtable = &process_vtable };
    }

    pub fn liveOrchestrator(self: *ProcessService) ?*proc_mod.Orchestrator {
        if (!self.initialized) return null;
        return &(self.orchestrator orelse return null);
    }

    fn provision_(ctx: *anyopaque, _: Role) !void {
        const self: *ProcessService = @ptrCast(@alignCast(ctx));
        if (self.initialized) return;

        self.table     = proc_mod.ProcessTable.init(self.allocator);
        self.pool      = try proc_mod.ThreadPool.init(self.allocator, self.config.worker_count);
        self.queue     = proc_mod.WorkQueue.init(self.allocator, self.config.queue_lane_cap);
        self.scheduler = proc_mod.Scheduler.init(self.allocator);
        self.ledger    = proc_mod.ResourceLedger.init(self.allocator);

        self.orchestrator = proc_mod.Orchestrator.init(
            self.allocator,
            &(self.table.?),
            &(self.pool.?),
            &(self.scheduler.?),
            &(self.queue.?),
            &(self.ledger.?),
        );
        self.initialized = true;
    }

    fn start_(_: *anyopaque, _: Role) !void {}

    fn drain_(ctx: *anyopaque, actor: Role) !void {
        // Drain all worker slots gracefully.
        const self: *ProcessService = @ptrCast(@alignCast(ctx));
        if (self.pool) |*p| {
            for (p.workers) |w| {
                p.drain(w.id) catch {};
            }
        }
        _ = actor;
    }

    fn stop_(ctx: *anyopaque, _: Role) !void {
        const self: *ProcessService = @ptrCast(@alignCast(ctx));
        self.deinit();
    }

    fn health_(ctx: *anyopaque) HealthStatus {
        const self: *ProcessService = @ptrCast(@alignCast(ctx));
        if (!self.initialized) return .unknown;
        const o = self.orchestrator orelse return .unknown;
        const s = o.stats();
        // Degraded if the queue is backed up with no idle workers.
        if (s.idle_workers == 0 and s.queued_tasks > 32) return .degraded;
        return .healthy;
    }

    fn describe_(ctx: *anyopaque, buf: []u8) usize {
        const self: *ProcessService = @ptrCast(@alignCast(ctx));
        if (!self.initialized) {
            const s = "ProcessService: not initialized";
            const n = @min(s.len, buf.len);
            @memcpy(buf[0..n], s[0..n]);
            return n;
        }
        const s = self.orchestrator.?.stats();
        const written = std.fmt.bufPrint(buf,
            "ProcessService: {d} procs, {d}/{d} workers busy, {d} queued",
            .{ s.live_processes, s.busy_workers,
               s.busy_workers + s.idle_workers, s.queued_tasks },
        ) catch buf[0..0];
        return written.len;
    }

    const process_vtable = ServiceVTable{
        .provision = provision_,
        .start     = start_,
        .drain     = drain_,
        .stop      = stop_,
        .health    = health_,
        .describe  = describe_,
    };
};

// ─────────────────────────────────────────────────────────────────────────────
// §15  NetworkService  — wraps network.zig NetworkManager
// ─────────────────────────────────────────────────────────────────────────────

pub const NetworkServiceConfig = struct {
    max_sockets:       usize                = 256,
    max_connections:   usize                = 128,
    max_conn_failures: u32                  = 5,
    packet_lane_cap:   usize                = 512,
    firewall_default:  net_mod.FirewallAction = .allow,
};

/// Concrete service wrapper around net_mod.NetworkManager.
pub const NetworkService = struct {
    allocator:   std.mem.Allocator,
    config:      NetworkServiceConfig,
    manager:     ?net_mod.NetworkManager,
    initialized: bool,

    pub fn init(allocator: std.mem.Allocator, config: NetworkServiceConfig) NetworkService {
        return .{
            .allocator   = allocator,
            .config      = config,
            .manager     = null,
            .initialized = false,
        };
    }

    pub fn deinit(self: *NetworkService) void {
        if (self.manager) |*m| m.deinit();
        self.manager     = null;
        self.initialized = false;
    }

    pub fn handle(self: *NetworkService) ServiceHandle {
        return .{ .ctx = self, .vtable = &network_vtable };
    }

    pub fn liveManager(self: *NetworkService) ?*net_mod.NetworkManager {
        if (!self.initialized) return null;
        return &(self.manager orelse return null);
    }

    fn provision_(ctx: *anyopaque, actor: Role) !void {
        const self: *NetworkService = @ptrCast(@alignCast(ctx));
        if (self.initialized) return;
        const c = self.config;
        self.manager = net_mod.NetworkManager.init(
            self.allocator,
            c.max_sockets,
            c.max_connections,
            c.max_conn_failures,
            c.packet_lane_cap,
            c.firewall_default,
        );
        // Register the two canonical network managers from kernel.zig.
        try self.manager.?.registerManager(actor, "kernel-native",    "native");
        try self.manager.?.registerManager(actor, "kogi-go-network",  "go-rpc");
        self.initialized = true;
    }

    fn start_(_: *anyopaque, _: Role) !void {}

    fn drain_(ctx: *anyopaque, _: Role) !void {
        // Close all open sockets.
        const self: *NetworkService = @ptrCast(@alignCast(ctx));
        if (self.manager) |*m| {
            // Iterate through all sockets and close open ones.
            for (m.sockets.sockets.items) |*sock| {
                if (sock.state != net_mod.SocketState.closed) {
                    m.sockets.close(.host, sock.fd) catch {};
                }
            }
        }
    }

    fn stop_(ctx: *anyopaque, _: Role) !void {
        const self: *NetworkService = @ptrCast(@alignCast(ctx));
        self.deinit();
    }

    fn health_(ctx: *anyopaque) HealthStatus {
        const self: *NetworkService = @ptrCast(@alignCast(ctx));
        if (!self.initialized) return .unknown;
        const s = self.manager.?.stats();
        if (s.unhealthy_conns > 0) return .degraded;
        return .healthy;
    }

    fn describe_(ctx: *anyopaque, buf: []u8) usize {
        const self: *NetworkService = @ptrCast(@alignCast(ctx));
        if (!self.initialized) {
            const s = "NetworkService: not initialized";
            const n = @min(s.len, buf.len);
            @memcpy(buf[0..n], s[0..n]);
            return n;
        }
        const s = self.manager.?.stats();
        const written = std.fmt.bufPrint(buf,
            "NetworkService: {d} sockets, {d} active/{d} idle conns, {d} pkts queued",
            .{ s.open_sockets, s.active_connections, s.idle_connections, s.queued_packets },
        ) catch buf[0..0];
        return written.len;
    }

    const network_vtable = ServiceVTable{
        .provision = provision_,
        .start     = start_,
        .drain     = drain_,
        .stop      = stop_,
        .health    = health_,
        .describe  = describe_,
    };
};

// ─────────────────────────────────────────────────────────────────────────────
// §16  ModuleService  — wraps module.zig ModuleSystem
// ─────────────────────────────────────────────────────────────────────────────

pub const ModuleServiceConfig = struct {
    total_memory:   u64   = 8 * 1024 * 1024 * 1024,
    event_capacity: usize = 2048,
};

/// Concrete service wrapper around mod_mod.ModuleSystem.
pub const ModuleService = struct {
    allocator:   std.mem.Allocator,
    config:      ModuleServiceConfig,
    system:      ?mod_mod.ModuleSystem,
    initialized: bool,

    pub fn init(allocator: std.mem.Allocator, config: ModuleServiceConfig) ModuleService {
        return .{
            .allocator   = allocator,
            .config      = config,
            .system      = null,
            .initialized = false,
        };
    }

    pub fn deinit(self: *ModuleService) void {
        if (self.system) |*s| s.deinit();
        self.system      = null;
        self.initialized = false;
    }

    pub fn handle(self: *ModuleService) ServiceHandle {
        return .{ .ctx = self, .vtable = &module_vtable };
    }

    pub fn liveSystem(self: *ModuleService) ?*mod_mod.ModuleSystem {
        if (!self.initialized) return null;
        return &(self.system orelse return null);
    }

    fn provision_(ctx: *anyopaque, _: Role) !void {
        const self: *ModuleService = @ptrCast(@alignCast(ctx));
        if (self.initialized) return;
        self.system = mod_mod.ModuleSystem.init(
            self.allocator,
            self.config.total_memory,
            self.config.event_capacity,
        );
        self.system.?.relink();
        self.initialized = true;
    }

    fn start_(_: *anyopaque, _: Role) !void {}

    fn drain_(ctx: *anyopaque, actor: Role) !void {
        // Stop all active modules gracefully.
        const self: *ModuleService = @ptrCast(@alignCast(ctx));
        if (self.system) |*sys| {
            var ids = std.ArrayList([]const u8).init(self.allocator);
            defer ids.deinit();
            var it = sys.registry.modules.keyIterator();
            while (it.next()) |k| ids.append(k.*) catch {};
            for (ids.items) |id| {
                const m = sys.registry.get(id) orelse continue;
                if (m.status == .active) {
                    sys.lifecycle.stop(@enumFromInt(@intFromEnum(actor)), id) catch {};
                }
            }
        }
    }

    fn stop_(ctx: *anyopaque, _: Role) !void {
        const self: *ModuleService = @ptrCast(@alignCast(ctx));
        self.deinit();
    }

    fn health_(ctx: *anyopaque) HealthStatus {
        const self: *ModuleService = @ptrCast(@alignCast(ctx));
        if (!self.initialized) return .unknown;
        const s = self.system.?.stats();
        if (s.faulted_modules > 0) return .degraded;
        return .healthy;
    }

    fn describe_(ctx: *anyopaque, buf: []u8) usize {
        const self: *ModuleService = @ptrCast(@alignCast(ctx));
        if (!self.initialized) {
            const s = "ModuleService: not initialized";
            const n = @min(s.len, buf.len);
            @memcpy(buf[0..n], s[0..n]);
            return n;
        }
        const s = self.system.?.stats();
        const written = std.fmt.bufPrint(buf,
            "ModuleService: {d} modules ({d} active, {d} faulted)",
            .{ s.total_modules, s.active_modules, s.faulted_modules },
        ) catch buf[0..0];
        return written.len;
    }

    const module_vtable = ServiceVTable{
        .provision = provision_,
        .start     = start_,
        .drain     = drain_,
        .stop      = stop_,
        .health    = health_,
        .describe  = describe_,
    };
};

// ─────────────────────────────────────────────────────────────────────────────
// §17  ServiceManager
// ─────────────────────────────────────────────────────────────────────────────

/// Top-level coordinator.  Owns the registry, provisioner, health monitor,
/// dependency graph, and event bus.  All public callers go through here.
pub const ServiceManager = struct {
    allocator:   std.mem.Allocator,
    registry:    ServiceRegistry,
    provisioner: ServiceProvisioner,
    monitor:     HealthMonitor,
    graph:       DependencyGraph,
    bus:         ServiceEventBus,

    pub fn init(
        allocator:     std.mem.Allocator,
        global_budget: GlobalBudget,
        event_cap:     usize,
    ) ServiceManager {
        var reg  = ServiceRegistry.init(allocator);
        var prov = ServiceProvisioner.init(&reg, global_budget);
        var bus  = ServiceEventBus.init(allocator, event_cap);
        const mon   = HealthMonitor.init(&reg, &prov, &bus);
        const graph = DependencyGraph.init(allocator);
        return .{
            .allocator   = allocator,
            .registry    = reg,
            .provisioner = prov,
            .monitor     = mon,
            .graph       = graph,
            .bus         = bus,
        };
    }

    /// Re-seat internal cross-references after the struct has been moved into
    /// its final storage location.  Must be called once immediately after init.
    pub fn relink(self: *ServiceManager) void {
        self.provisioner = ServiceProvisioner.init(&self.registry, self.provisioner.global_budget);
        self.monitor     = HealthMonitor.init(&self.registry, &self.provisioner, &self.bus);
        self.graph       = DependencyGraph.init(self.allocator);
    }

    pub fn deinit(self: *ServiceManager) void {
        self.registry.deinit();
        self.bus.deinit();
    }

    // ── Registration ──────────────────────────────────────────────────────

    /// Register a service with the manager.  The handle must remain valid for
    /// the lifetime of the registration.
    pub fn register(
        self:         *ServiceManager,
        actor:        Role,
        id:           []const u8,
        display_name: []const u8,
        version:      []const u8,
        budget:       ResourceBudget,
        restart:      RestartConfig,
        health:       HealthPolicy,
        handle:       ServiceHandle,
    ) !void {
        try self.registry.register(actor, id, display_name, version,
            budget, restart, health, handle);
        try self.bus.publish(id, Topic.registered, id);
    }

    pub fn addDependency(self: *ServiceManager, actor: Role, id: []const u8, dep_id: []const u8) !void {
        try self.registry.addDependency(actor, id, dep_id);
    }

    // ── Provision ─────────────────────────────────────────────────────────

    /// Allocate service resources then call the subsystem's provision hook.
    pub fn provision(self: *ServiceManager, actor: Role, id: []const u8) !void {
        if (!hasPermission(actor, .manage_modules)) return ServiceError.AccessDenied;
        if (!self.registry.dependenciesMet(id)) return ServiceError.DependencyNotMet;

        const svc = self.registry.getPtr(id) orelse return ServiceError.ServiceNotFound;
        const handle = svc.handle orelse return ServiceError.HandleRequired;

        try self.registry.transition(id, .provisioning);

        // Acquire the full memory budget for this service up front.
        self.provisioner.acquireMemory(id, svc.budget.memory_bytes) catch |err| {
            self.registry.transition(id, .faulted) catch {};
            try self.bus.publish(id, Topic.faulted, "provision:memory");
            return err;
        };

        // Acquire process and connection slots.
        self.provisioner.acquireProcesses(id, svc.budget.max_processes) catch |err| {
            self.provisioner.releaseAll(id);
            self.registry.transition(id, .faulted) catch {};
            try self.bus.publish(id, Topic.faulted, "provision:processes");
            return err;
        };

        self.provisioner.acquireConnections(id, svc.budget.max_connections) catch |err| {
            self.provisioner.releaseAll(id);
            self.registry.transition(id, .faulted) catch {};
            try self.bus.publish(id, Topic.faulted, "provision:connections");
            return err;
        };

        // Call the subsystem's own provision hook.
        handle.provision(actor) catch |err| {
            self.provisioner.releaseAll(id);
            self.registry.transition(id, .faulted) catch {};
            try self.bus.publish(id, Topic.faulted, "provision:hook");
            return err;
        };

        try self.registry.transition(id, .running);
        self.registry.getPtr(id).?.start_count += 1;
        try self.bus.publish(id, Topic.provisioned, id);
    }

    // ── Start (call subsystem start hook while already running) ───────────

    pub fn start(self: *ServiceManager, actor: Role, id: []const u8) !void {
        if (!hasPermission(actor, .manage_modules)) return ServiceError.AccessDenied;
        const svc = self.registry.getPtr(id) orelse return ServiceError.ServiceNotFound;
        if (svc.status != .running) return ServiceError.InvalidStatusTransition;
        const handle = svc.handle orelse return ServiceError.HandleRequired;
        handle.start(actor) catch |err| {
            try self.faultService(id, "start:hook");
            return err;
        };
        try self.bus.publish(id, Topic.started, id);
    }

    /// Convenience: provision then immediately call start.
    pub fn provisionAndStart(self: *ServiceManager, actor: Role, id: []const u8) !void {
        try self.provision(actor, id);
        try self.start(actor, id);
    }

    // ── Suspend / resume ──────────────────────────────────────────────────

    pub fn suspend_(self: *ServiceManager, actor: Role, id: []const u8) !void {
        if (!hasPermission(actor, .manage_modules)) return ServiceError.AccessDenied;
        try self.registry.transition(id, .suspended);
        try self.bus.publish(id, Topic.suspended, id);
    }

    pub fn resume_(self: *ServiceManager, actor: Role, id: []const u8) !void {
        if (!hasPermission(actor, .manage_modules)) return ServiceError.AccessDenied;
        try self.registry.transition(id, .running);
        try self.bus.publish(id, Topic.resumed, id);
    }

    // ── Drain / stop ──────────────────────────────────────────────────────

    pub fn drain(self: *ServiceManager, actor: Role, id: []const u8) !void {
        if (!hasPermission(actor, .manage_modules)) return ServiceError.AccessDenied;
        const svc = self.registry.getPtr(id) orelse return ServiceError.ServiceNotFound;
        if (svc.status == .suspended) try self.registry.transition(id, .running);
        try self.registry.transition(id, .draining);
        if (svc.handle) |h| h.drain(actor) catch {};
        try self.bus.publish(id, Topic.draining, id);
    }

    pub fn stop(self: *ServiceManager, actor: Role, id: []const u8) !void {
        if (!hasPermission(actor, .manage_modules)) return ServiceError.AccessDenied;
        const svc = self.registry.getPtr(id) orelse return ServiceError.ServiceNotFound;

        if (svc.status.isActive()) try self.drain(actor, id);

        if (svc.handle) |h| h.stop(actor) catch {};
        try self.registry.transition(id, .stopped);
        self.provisioner.releaseAll(id);
        try self.bus.publish(id, Topic.stopped, id);
    }

    // ── Fault ─────────────────────────────────────────────────────────────

    fn faultService(self: *ServiceManager, id: []const u8, reason: []const u8) !void {
        const svc = self.registry.getPtr(id) orelse return;
        svc.fault_count   += 1;
        svc.last_fault_ms  = std.time.milliTimestamp();
        self.registry.transition(id, .faulted) catch {};
        self.provisioner.releaseAll(id);
        try self.bus.publish(id, Topic.faulted, reason);
    }

    // ── Restart ───────────────────────────────────────────────────────────

    pub fn restart(self: *ServiceManager, actor: Role, id: []const u8) !void {
        if (!hasPermission(actor, .manage_modules)) return ServiceError.AccessDenied;
        const svc = self.registry.getPtr(id) orelse return ServiceError.ServiceNotFound;
        if (svc.restart_count >= svc.restart.max_restarts)
            return ServiceError.MaxRestartsExceeded;

        try self.stop(actor, id);
        svc.restart_count += 1;
        try self.provision(actor, id);
        try self.start(actor, id);
        try self.bus.publish(id, Topic.restarted, id);
    }

    // ── Remove ────────────────────────────────────────────────────────────

    pub fn remove(self: *ServiceManager, actor: Role, id: []const u8) !void {
        if (!hasPermission(actor, .manage_modules)) return ServiceError.AccessDenied;
        const svc = self.registry.getPtr(id) orelse return ServiceError.ServiceNotFound;
        if (!svc.status.isStopped()) try self.stop(actor, id);
        try self.registry.transition(id, .removing);
        try self.bus.publish(id, Topic.removed, id);
        try self.registry.deregister(actor, id);
    }

    // ── Ordered startup / shutdown ────────────────────────────────────────

    /// Start all registered services in topological dependency order.
    /// Already-running services are skipped.  Returns the number started.
    pub fn startAll(self: *ServiceManager, actor: Role) !usize {
        if (!hasPermission(actor, .manage_modules)) return ServiceError.AccessDenied;

        var order = std.ArrayList([]const u8).init(self.allocator);
        defer order.deinit();
        try self.graph.startOrder(&self.registry, &order);

        var n: usize = 0;
        for (order.items) |id| {
            const svc = self.registry.get(id) orelse continue;
            if (svc.status == .running) continue;
            if (svc.status != .registered and svc.status != .stopped) continue;
            self.provisionAndStart(actor, id) catch |err| {
                std.debug.print("[ServiceManager] startAll: '{s}' failed: {}\n", .{ id, err });
                continue;
            };
            n += 1;
        }
        return n;
    }

    /// Stop all running/suspended services in reverse dependency order.
    pub fn stopAll(self: *ServiceManager, actor: Role) !void {
        if (!hasPermission(actor, .manage_modules)) return ServiceError.AccessDenied;

        var order = std.ArrayList([]const u8).init(self.allocator);
        defer order.deinit();
        try self.graph.startOrder(&self.registry, &order);

        var i = order.items.len;
        while (i > 0) {
            i -= 1;
            const id  = order.items[i];
            const svc = self.registry.get(id) orelse continue;
            if (!svc.status.isActive()) continue;
            self.stop(actor, id) catch {};
        }
    }

    // ── Health / recovery tick ────────────────────────────────────────────

    /// Run health probes.  Returns number of services probed.
    pub fn healthTick(self: *ServiceManager, now_ms: i64) !usize {
        return self.monitor.tick(now_ms);
    }

    /// Auto-restart faulted services whose policy permits it.
    /// Returns the number of services successfully restarted.
    pub fn recoverFaulted(self: *ServiceManager, actor: Role) !usize {
        var candidates = std.ArrayList([]const u8).init(self.allocator);
        defer candidates.deinit();

        var it = self.registry.services.valueIterator();
        while (it.next()) |svc| {
            if (svc.status != .faulted) continue;
            if (svc.restart.policy == .never) continue;
            if (svc.restart_count >= svc.restart.max_restarts) continue;
            try candidates.append(svc.id);
        }

        var n: usize = 0;
        for (candidates.items) |id| {
            self.restart(actor, id) catch continue;
            n += 1;
        }
        return n;
    }

    // ── Stats ─────────────────────────────────────────────────────────────

    pub const Stats = struct {
        total:            usize,
        running:          usize,
        provisioning:     usize,
        suspended:        usize,
        faulted:          usize,
        stopped:          usize,
        total_events:     usize,
        global_mem_used:  u64,
        global_mem_cap:   u64,
        global_processes: u32,
        global_conns:     u32,
    };

    pub fn stats(self: *const ServiceManager) Stats {
        return .{
            .total            = self.registry.count(),
            .running          = self.registry.countByStatus(.running),
            .provisioning     = self.registry.countByStatus(.provisioning),
            .suspended        = self.registry.countByStatus(.suspended),
            .faulted          = self.registry.countByStatus(.faulted),
            .stopped          = self.registry.countByStatus(.stopped),
            .total_events     = self.bus.len(),
            .global_mem_used  = self.provisioner.global_usage.memory_bytes,
            .global_mem_cap   = self.provisioner.global_budget.memory_bytes,
            .global_processes = self.provisioner.global_usage.process_count,
            .global_conns     = self.provisioner.global_usage.connection_count,
        };
    }

    // ── Describe ──────────────────────────────────────────────────────────

    /// Write a human-readable snapshot of every service into `buf`.
    /// Returns bytes written.
    pub fn describeAll(self: *ServiceManager, buf: []u8) usize {
        var pos: usize = 0;
        var it = self.registry.services.valueIterator();
        while (it.next()) |svc| {
            if (pos >= buf.len) break;
            const hdr = std.fmt.bufPrint(buf[pos..],
                "[{s}] {s} v{s} — {s}\n",
                .{ svc.id, svc.display_name, svc.version, @tagName(svc.status) },
            ) catch break;
            pos += hdr.len;
            if (svc.handle) |h| {
                const sub = h.describe(buf[pos..]);
                pos += sub;
                if (pos < buf.len) { buf[pos] = '\n'; pos += 1; }
            }
        }
        return pos;
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// §18  Tests
// ─────────────────────────────────────────────────────────────────────────────

fn testGlobalBudget() GlobalBudget {
    return .{
        .memory_bytes    = 32 * 1024 * 1024 * 1024, // 32 GiB
        .max_processes   = 4096,
        .max_connections = 8192,
    };
}

fn testServiceBudget() ResourceBudget {
    return .{
        .memory_bytes    = 256 * 1024 * 1024,
        .max_processes   = 32,
        .max_files       = 512,
        .max_units       = 1000,
        .max_connections = 64,
    };
}

// ── MemoryService ─────────────────────────────────────────────────────────────

test "MemoryService: provision initialises MemoryManager" {
    var svc = MemoryService.init(std.testing.allocator, .{
        .total_bytes = 64 * 1024 * 1024, .audit_capacity = 64, .enable_audit = true,
    });
    defer svc.deinit();

    try svc.handle().provision(.host);
    try std.testing.expect(svc.initialized);
    try std.testing.expect(svc.liveManager() != null);

    const s = svc.liveManager().?.stats();
    try std.testing.expectEqual(@as(u64, 64 * 1024 * 1024), s.total_bytes);
    try std.testing.expectEqual(@as(u64, 0), s.used_bytes);
}

test "MemoryService: health reflects pressure level" {
    var svc = MemoryService.init(std.testing.allocator, .{
        .total_bytes = 100, .audit_capacity = 8, .enable_audit = false,
    });
    defer svc.deinit();
    try svc.handle().provision(.host);

    // Use 75 of 100 bytes → moderate pressure → degraded health.
    try svc.liveManager().?.allocate(75);
    try std.testing.expectEqual(HealthStatus.degraded, svc.handle().health());

    // Use 95 bytes → critical → unhealthy.
    try svc.liveManager().?.allocate(15);
    try std.testing.expectEqual(HealthStatus.unhealthy, svc.handle().health());
}

test "MemoryService: stop tears down manager" {
    var svc = MemoryService.init(std.testing.allocator, .{
        .total_bytes = 1024, .audit_capacity = 16, .enable_audit = false,
    });
    try svc.handle().provision(.host);
    try svc.handle().stop(.host);
    try std.testing.expect(!svc.initialized);
    try std.testing.expect(svc.liveManager() == null);
}

// ── ProcessService ────────────────────────────────────────────────────────────

test "ProcessService: provision creates Orchestrator" {
    var svc = ProcessService.init(std.testing.allocator, .{ .worker_count = 4, .queue_lane_cap = 16 });
    defer svc.deinit();

    try svc.handle().provision(.host);
    try std.testing.expect(svc.initialized);

    const orch = svc.liveOrchestrator().?;
    const s = orch.stats();
    try std.testing.expectEqual(@as(usize, 4), s.idle_workers);
    try std.testing.expectEqual(@as(usize, 0), s.live_processes);
}

test "ProcessService: can spawn processes through live orchestrator" {
    var svc = ProcessService.init(std.testing.allocator, .{ .worker_count = 2, .queue_lane_cap = 8 });
    defer svc.deinit();
    try svc.handle().provision(.host);

    const orch = svc.liveOrchestrator().?;
    try orch.createGroup(.host, "test-group", .one_for_one, 3);
    const pid = try orch.spawnInGroup(.host, "test-group", "worker-a", .server, 100);
    try std.testing.expect(pid >= 1000);
    try std.testing.expectEqual(@as(usize, 1), orch.stats().live_processes);
}

test "ProcessService: health is healthy under normal load" {
    var svc = ProcessService.init(std.testing.allocator, .{ .worker_count = 8, .queue_lane_cap = 64 });
    defer svc.deinit();
    try svc.handle().provision(.host);
    try std.testing.expectEqual(HealthStatus.healthy, svc.handle().health());
}

// ── NetworkService ────────────────────────────────────────────────────────────

test "NetworkService: provision registers canonical network managers" {
    var svc = NetworkService.init(std.testing.allocator, .{});
    defer svc.deinit();

    try svc.handle().provision(.host);
    try std.testing.expect(svc.initialized);

    const nm = svc.liveManager().?;
    try std.testing.expect(nm.getManager("kernel-native")   != null);
    try std.testing.expect(nm.getManager("kogi-go-network") != null);
}

test "NetworkService: health is healthy with no unhealthy connections" {
    var svc = NetworkService.init(std.testing.allocator, .{});
    defer svc.deinit();
    try svc.handle().provision(.host);
    try std.testing.expectEqual(HealthStatus.healthy, svc.handle().health());
}

test "NetworkService: stop cleans up manager" {
    var svc = NetworkService.init(std.testing.allocator, .{});
    try svc.handle().provision(.host);
    try svc.handle().stop(.host);
    try std.testing.expect(!svc.initialized);
}

// ── ModuleService ─────────────────────────────────────────────────────────────

test "ModuleService: provision creates ModuleSystem" {
    var svc = ModuleService.init(std.testing.allocator, .{
        .total_memory = 512 * 1024 * 1024, .event_capacity = 64,
    });
    defer svc.deinit();

    try svc.handle().provision(.host);
    try std.testing.expect(svc.initialized);
    try std.testing.expect(svc.liveSystem() != null);
}

test "ModuleService: health degrades when modules fault" {
    var svc = ModuleService.init(std.testing.allocator, .{
        .total_memory = 512 * 1024 * 1024, .event_capacity = 64,
    });
    defer svc.deinit();
    try svc.handle().provision(.host);

    const sys = svc.liveSystem().?;
    try sys.registerAndStart(.host, "test.mod", "test", .module);
    try std.testing.expectEqual(HealthStatus.healthy, svc.handle().health());

    try sys.lifecycle.fault("test.mod", "crash");
    try std.testing.expectEqual(HealthStatus.degraded, svc.handle().health());
}

// ── ServiceManager: full integration ─────────────────────────────────────────

test "ServiceManager: register all four kernel services" {
    var mgr = ServiceManager.init(std.testing.allocator, testGlobalBudget(), 1024);
    mgr.relink();
    defer mgr.deinit();

    var mem_svc  = MemoryService.init(std.testing.allocator,  .{ .total_bytes = 64 * 1024 * 1024, .audit_capacity = 32, .enable_audit = false });
    var proc_svc = ProcessService.init(std.testing.allocator, .{ .worker_count = 4, .queue_lane_cap = 16 });
    var net_svc  = NetworkService.init(std.testing.allocator, .{});
    var mod_svc  = ModuleService.init(std.testing.allocator,  .{ .total_memory = 256 * 1024 * 1024, .event_capacity = 64 });
    defer mem_svc.deinit();
    defer proc_svc.deinit();
    defer net_svc.deinit();
    defer mod_svc.deinit();

    try mgr.register(.host, "kernel.memory",  "Memory Service",  "1.0.0", testServiceBudget(), .{}, .{}, mem_svc.handle());
    try mgr.register(.host, "kernel.process", "Process Service", "1.0.0", testServiceBudget(), .{}, .{}, proc_svc.handle());
    try mgr.register(.host, "kernel.network", "Network Service", "1.0.0", testServiceBudget(), .{}, .{}, net_svc.handle());
    try mgr.register(.host, "kernel.module",  "Module Service",  "1.0.0", testServiceBudget(), .{}, .{}, mod_svc.handle());

    try std.testing.expectEqual(@as(usize, 4), mgr.registry.count());
    try std.testing.expectEqual(@as(usize, 4), mgr.registry.countByStatus(.registered));
}

test "ServiceManager: provisionAndStart all four services" {
    var mgr = ServiceManager.init(std.testing.allocator, testGlobalBudget(), 1024);
    mgr.relink();
    defer mgr.deinit();

    var mem_svc  = MemoryService.init(std.testing.allocator,  .{ .total_bytes = 64 * 1024 * 1024, .audit_capacity = 32, .enable_audit = false });
    var proc_svc = ProcessService.init(std.testing.allocator, .{ .worker_count = 4, .queue_lane_cap = 16 });
    var net_svc  = NetworkService.init(std.testing.allocator, .{});
    var mod_svc  = ModuleService.init(std.testing.allocator,  .{ .total_memory = 256 * 1024 * 1024, .event_capacity = 64 });
    defer mem_svc.deinit();
    defer proc_svc.deinit();
    defer net_svc.deinit();
    defer mod_svc.deinit();

    try mgr.register(.host, "kernel.memory",  "Memory Service",  "1.0.0", testServiceBudget(), .{}, .{}, mem_svc.handle());
    try mgr.register(.host, "kernel.process", "Process Service", "1.0.0", testServiceBudget(), .{}, .{}, proc_svc.handle());
    try mgr.register(.host, "kernel.network", "Network Service", "1.0.0", testServiceBudget(), .{}, .{}, net_svc.handle());
    try mgr.register(.host, "kernel.module",  "Module Service",  "1.0.0", testServiceBudget(), .{}, .{}, mod_svc.handle());

    try mgr.provisionAndStart(.host, "kernel.memory");
    try mgr.provisionAndStart(.host, "kernel.process");
    try mgr.provisionAndStart(.host, "kernel.network");
    try mgr.provisionAndStart(.host, "kernel.module");

    try std.testing.expectEqual(@as(usize, 4), mgr.registry.countByStatus(.running));
    try std.testing.expect(mem_svc.initialized);
    try std.testing.expect(proc_svc.initialized);
    try std.testing.expect(net_svc.initialized);
    try std.testing.expect(mod_svc.initialized);
}

test "ServiceManager: dependency chain blocks provision until deps are running" {
    var mgr = ServiceManager.init(std.testing.allocator, testGlobalBudget(), 512);
    mgr.relink();
    defer mgr.deinit();

    var mem_svc  = MemoryService.init(std.testing.allocator,  .{ .total_bytes = 64 * 1024 * 1024, .audit_capacity = 32, .enable_audit = false });
    var proc_svc = ProcessService.init(std.testing.allocator, .{ .worker_count = 2, .queue_lane_cap = 8 });
    defer mem_svc.deinit();
    defer proc_svc.deinit();

    try mgr.register(.host, "kernel.memory",  "Memory Service",  "1.0.0", testServiceBudget(), .{}, .{}, mem_svc.handle());
    try mgr.register(.host, "kernel.process", "Process Service", "1.0.0", testServiceBudget(), .{}, .{}, proc_svc.handle());
    try mgr.addDependency(.host, "kernel.process", "kernel.memory");

    // process must not start while memory is still registered-only.
    try std.testing.expectError(
        ServiceError.DependencyNotMet,
        mgr.provision(.host, "kernel.process"),
    );

    try mgr.provisionAndStart(.host, "kernel.memory");
    // Now memory is running — process can proceed.
    try mgr.provision(.host, "kernel.process");
    try std.testing.expectEqual(ServiceStatus.running,
        mgr.registry.get("kernel.process").?.status);
}

test "ServiceManager: startAll respects topological order" {
    var mgr = ServiceManager.init(std.testing.allocator, testGlobalBudget(), 1024);
    mgr.relink();
    defer mgr.deinit();

    var mem_svc  = MemoryService.init(std.testing.allocator,  .{ .total_bytes = 64 * 1024 * 1024, .audit_capacity = 32, .enable_audit = false });
    var proc_svc = ProcessService.init(std.testing.allocator, .{ .worker_count = 4, .queue_lane_cap = 16 });
    var net_svc  = NetworkService.init(std.testing.allocator, .{});
    var mod_svc  = ModuleService.init(std.testing.allocator,  .{ .total_memory = 256 * 1024 * 1024, .event_capacity = 64 });
    defer mem_svc.deinit();
    defer proc_svc.deinit();
    defer net_svc.deinit();
    defer mod_svc.deinit();

    try mgr.register(.host, "kernel.memory",  "Memory Service",  "1.0.0", testServiceBudget(), .{}, .{}, mem_svc.handle());
    try mgr.register(.host, "kernel.process", "Process Service", "1.0.0", testServiceBudget(), .{}, .{}, proc_svc.handle());
    try mgr.register(.host, "kernel.network", "Network Service", "1.0.0", testServiceBudget(), .{}, .{}, net_svc.handle());
    try mgr.register(.host, "kernel.module",  "Module Service",  "1.0.0", testServiceBudget(), .{}, .{}, mod_svc.handle());

    // Declare the subsystem dependency chain.
    try mgr.addDependency(.host, "kernel.process", "kernel.memory");
    try mgr.addDependency(.host, "kernel.network", "kernel.memory");
    try mgr.addDependency(.host, "kernel.module",  "kernel.process");
    try mgr.addDependency(.host, "kernel.module",  "kernel.network");

    const started = try mgr.startAll(.host);
    try std.testing.expectEqual(@as(usize, 4), started);
    try std.testing.expectEqual(@as(usize, 4), mgr.registry.countByStatus(.running));
}

test "ServiceManager: stopAll tears down in reverse order" {
    var mgr = ServiceManager.init(std.testing.allocator, testGlobalBudget(), 1024);
    mgr.relink();
    defer mgr.deinit();

    var mem_svc  = MemoryService.init(std.testing.allocator,  .{ .total_bytes = 64 * 1024 * 1024, .audit_capacity = 32, .enable_audit = false });
    var proc_svc = ProcessService.init(std.testing.allocator, .{ .worker_count = 2, .queue_lane_cap = 8 });
    defer mem_svc.deinit();
    defer proc_svc.deinit();

    try mgr.register(.host, "kernel.memory",  "Memory Service",  "1.0.0", testServiceBudget(), .{}, .{}, mem_svc.handle());
    try mgr.register(.host, "kernel.process", "Process Service", "1.0.0", testServiceBudget(), .{}, .{}, proc_svc.handle());
    try mgr.addDependency(.host, "kernel.process", "kernel.memory");

    _ = try mgr.startAll(.host);
    try mgr.stopAll(.host);

    try std.testing.expectEqual(@as(usize, 0), mgr.registry.countByStatus(.running));
    try std.testing.expect(!mem_svc.initialized);
    try std.testing.expect(!proc_svc.initialized);
}

test "ServiceManager: suspend and resume a service" {
    var mgr = ServiceManager.init(std.testing.allocator, testGlobalBudget(), 256);
    mgr.relink();
    defer mgr.deinit();

    var net_svc = NetworkService.init(std.testing.allocator, .{});
    defer net_svc.deinit();

    try mgr.register(.host, "kernel.network", "Network Service", "1.0.0", testServiceBudget(), .{}, .{}, net_svc.handle());
    try mgr.provisionAndStart(.host, "kernel.network");

    try mgr.suspend_(.host, "kernel.network");
    try std.testing.expectEqual(ServiceStatus.suspended,
        mgr.registry.get("kernel.network").?.status);

    try mgr.resume_(.host, "kernel.network");
    try std.testing.expectEqual(ServiceStatus.running,
        mgr.registry.get("kernel.network").?.status);
}

test "ServiceManager: fault and restart" {
    var mgr = ServiceManager.init(std.testing.allocator, testGlobalBudget(), 512);
    mgr.relink();
    defer mgr.deinit();

    var mem_svc = MemoryService.init(std.testing.allocator, .{ .total_bytes = 64 * 1024 * 1024, .audit_capacity = 32, .enable_audit = false });
    defer mem_svc.deinit();

    try mgr.register(.host, "kernel.memory", "Memory Service", "1.0.0",
        testServiceBudget(), .{ .policy = .on_fault, .max_restarts = 3 }, .{}, mem_svc.handle());
    try mgr.provisionAndStart(.host, "kernel.memory");

    try mgr.faultService("kernel.memory", "test-fault");
    try std.testing.expectEqual(ServiceStatus.faulted,
        mgr.registry.get("kernel.memory").?.status);
    try std.testing.expectEqual(@as(u32, 1),
        mgr.registry.get("kernel.memory").?.fault_count);

    try mgr.restart(.host, "kernel.memory");
    try std.testing.expectEqual(ServiceStatus.running,
        mgr.registry.get("kernel.memory").?.status);
    try std.testing.expectEqual(@as(u32, 1),
        mgr.registry.get("kernel.memory").?.restart_count);
    try std.testing.expect(mem_svc.initialized);
}

test "ServiceManager: max restarts enforced" {
    var mgr = ServiceManager.init(std.testing.allocator, testGlobalBudget(), 512);
    mgr.relink();
    defer mgr.deinit();

    var mem_svc = MemoryService.init(std.testing.allocator, .{ .total_bytes = 64 * 1024 * 1024, .audit_capacity = 32, .enable_audit = false });
    defer mem_svc.deinit();

    try mgr.register(.host, "kernel.memory", "Memory Service", "1.0.0",
        testServiceBudget(), .{ .policy = .on_fault, .max_restarts = 2 }, .{}, mem_svc.handle());
    try mgr.provisionAndStart(.host, "kernel.memory");

    try mgr.faultService("kernel.memory", "err"); try mgr.restart(.host, "kernel.memory");
    try mgr.faultService("kernel.memory", "err"); try mgr.restart(.host, "kernel.memory");
    try mgr.faultService("kernel.memory", "err");

    try std.testing.expectError(
        ServiceError.MaxRestartsExceeded,
        mgr.restart(.host, "kernel.memory"),
    );
}

test "ServiceManager: recoverFaulted auto-restarts eligible services" {
    var mgr = ServiceManager.init(std.testing.allocator, testGlobalBudget(), 512);
    mgr.relink();
    defer mgr.deinit();

    var net_svc = NetworkService.init(std.testing.allocator, .{});
    defer net_svc.deinit();

    try mgr.register(.host, "kernel.network", "Network Service", "1.0.0",
        testServiceBudget(), .{ .policy = .on_fault, .max_restarts = 5 }, .{}, net_svc.handle());
    try mgr.provisionAndStart(.host, "kernel.network");
    try mgr.faultService("kernel.network", "timeout");

    const recovered = try mgr.recoverFaulted(.host);
    try std.testing.expectEqual(@as(usize, 1), recovered);
    try std.testing.expectEqual(ServiceStatus.running,
        mgr.registry.get("kernel.network").?.status);
}

test "ServiceManager: health monitor faults service after threshold" {
    var mgr = ServiceManager.init(std.testing.allocator, testGlobalBudget(), 512);
    mgr.relink();
    defer mgr.deinit();

    // A MemoryService with a tiny pool so we can push it to critical pressure.
    var mem_svc = MemoryService.init(std.testing.allocator, .{
        .total_bytes = 100, .audit_capacity = 8, .enable_audit = false,
    });
    defer mem_svc.deinit();

    try mgr.register(.host, "kernel.memory", "Memory Service", "1.0.0",
        testServiceBudget(),
        .{},
        // interval_ms = 0 so every tick probes; fault after 3 consecutive hits.
        .{ .interval_ms = 0, .fault_threshold = 3, .recovery_threshold = 2 },
        mem_svc.handle(),
    );
    try mgr.provisionAndStart(.host, "kernel.memory");

    // Drive memory to critical pressure so health() returns .unhealthy.
    try mem_svc.liveManager().?.allocate(95);

    _ = try mgr.healthTick(0);
    _ = try mgr.healthTick(1);
    _ = try mgr.healthTick(2);

    try std.testing.expectEqual(ServiceStatus.faulted,
        mgr.registry.get("kernel.memory").?.status);
}

test "ServiceManager: remove deregisters service cleanly" {
    var mgr = ServiceManager.init(std.testing.allocator, testGlobalBudget(), 256);
    mgr.relink();
    defer mgr.deinit();

    var mod_svc = ModuleService.init(std.testing.allocator, .{ .total_memory = 64 * 1024 * 1024, .event_capacity = 16 });
    defer mod_svc.deinit();

    try mgr.register(.host, "kernel.module", "Module Service", "1.0.0", testServiceBudget(), .{}, .{}, mod_svc.handle());
    try mgr.provisionAndStart(.host, "kernel.module");
    try mgr.remove(.host, "kernel.module");

    try std.testing.expectEqual(@as(usize, 0), mgr.registry.count());
}

test "ServiceManager: RBAC — user role cannot provision" {
    var mgr = ServiceManager.init(std.testing.allocator, testGlobalBudget(), 256);
    mgr.relink();
    defer mgr.deinit();

    var mem_svc = MemoryService.init(std.testing.allocator, .{ .total_bytes = 1024, .audit_capacity = 8, .enable_audit = false });
    defer mem_svc.deinit();

    // Register is also blocked for .user.
    try std.testing.expectError(
        ServiceError.AccessDenied,
        mgr.register(.user, "kernel.memory", "Memory Service", "1.0.0",
            testServiceBudget(), .{}, .{}, mem_svc.handle()),
    );
}

test "ServiceManager: global resource pool exhaustion prevents over-commit" {
    // Tiny global pool: only 300 MiB total memory.
    const tight = GlobalBudget{
        .memory_bytes    = 300 * 1024 * 1024,
        .max_processes   = 64,
        .max_connections = 128,
    };
    var mgr = ServiceManager.init(std.testing.allocator, tight, 256);
    mgr.relink();
    defer mgr.deinit();

    const big_budget = ResourceBudget{
        .memory_bytes    = 200 * 1024 * 1024,  // 200 MiB each — two won't fit
        .max_processes   = 8,
        .max_files       = 64,
        .max_units       = 100,
        .max_connections = 16,
    };

    var mem_svc  = MemoryService.init(std.testing.allocator, .{ .total_bytes = 64 * 1024 * 1024, .audit_capacity = 8, .enable_audit = false });
    var proc_svc = ProcessService.init(std.testing.allocator, .{ .worker_count = 2, .queue_lane_cap = 4 });
    defer mem_svc.deinit();
    defer proc_svc.deinit();

    try mgr.register(.host, "kernel.memory",  "Memory Service",  "1.0.0", big_budget, .{}, .{}, mem_svc.handle());
    try mgr.register(.host, "kernel.process", "Process Service", "1.0.0", big_budget, .{}, .{}, proc_svc.handle());

    try mgr.provision(.host, "kernel.memory");  // 200 MiB used

    // 200 + 200 = 400 > 300 — must fail.
    try std.testing.expectError(
        ServiceError.GlobalResourceExhausted,
        mgr.provision(.host, "kernel.process"),
    );
}

test "ServiceManager: event bus records all lifecycle events" {
    var mgr = ServiceManager.init(std.testing.allocator, testGlobalBudget(), 1024);
    mgr.relink();
    defer mgr.deinit();

    var net_svc = NetworkService.init(std.testing.allocator, .{});
    defer net_svc.deinit();

    try mgr.register(.host, "kernel.network", "Network Service", "1.0.0", testServiceBudget(), .{}, .{}, net_svc.handle());
    try mgr.provisionAndStart(.host, "kernel.network");
    try mgr.suspend_(.host, "kernel.network");
    try mgr.resume_(.host, "kernel.network");
    try mgr.stop(.host, "kernel.network");

    // registered, provisioned, started, suspended, resumed, draining, stopped ≥ 7
    try std.testing.expect(mgr.bus.len() >= 7);

    var provisioned = std.ArrayList(ServiceEvent).init(std.testing.allocator);
    defer provisioned.deinit();
    try mgr.bus.collect(Topic.provisioned, &provisioned);
    try std.testing.expectEqual(@as(usize, 1), provisioned.items.len);
}

test "ServiceManager: describe writes human-readable snapshot" {
    var mgr = ServiceManager.init(std.testing.allocator, testGlobalBudget(), 256);
    mgr.relink();
    defer mgr.deinit();

    var mem_svc = MemoryService.init(std.testing.allocator, .{ .total_bytes = 64 * 1024 * 1024, .audit_capacity = 8, .enable_audit = false });
    defer mem_svc.deinit();

    try mgr.register(.host, "kernel.memory", "Memory Service", "1.0.0", testServiceBudget(), .{}, .{}, mem_svc.handle());
    try mgr.provisionAndStart(.host, "kernel.memory");

    var buf: [1024]u8 = undefined;
    const n = mgr.describeAll(&buf);
    try std.testing.expect(n > 0);
    try std.testing.expect(std.mem.indexOf(u8, buf[0..n], "kernel.memory") != null);
}