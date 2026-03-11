//! resources.zig — Kernel ResourcesManager
//!
//! Sits one layer above the individual subsystem managers
//! (memory.zig, network.zig, processes.zig, module.zig, services.zig)
//! and below kernel.zig.  Its job is to give the rest of the kernel a
//! single, uniform surface for:
//!
//!   • Provisioning and releasing every resource dimension in one call
//!   • Enforcing cross-subsystem consistency (global memory ceiling,
//!     process + connection counts, per-tenant caps)
//!   • Aggregating live usage and health across all subsystems
//!   • Emitting a unified resource event bus entry for every allocation
//!     or release
//!
//! Subsystem dependency: none — this file imports the five subsystem
//! files but does not import kernel.zig, avoiding a circular dependency.
//!
//! Provides:
//!   ResourceDimension    — enum of every managed resource axis
//!   ResourcePolicy       — per-tenant caps across all dimensions
//!   TenantUsage          — live counters mirroring ResourcePolicy
//!   AllocationRequest    — declarative multi-dimension acquire spec
//!   ReleaseRequest       — declarative multi-dimension release spec
//!   ResourceEvent        — single record emitted to the event log
//!   ResourceEventLog     — bounded append-only event log
//!   TenantRegistry       — owns all per-tenant policy + usage records
//!   GlobalPool           — enforces system-wide ceilings
//!   ResourceEnforcer     — validates every acquire / release
//!   SubsystemBridge      — forwards allocations into actual subsystems
//!   ResourcesManager     — top-level coordinator (public API)

const std = @import("std");

const mem_mod = @import("memory.zig");
const proc_mod = @import("processes.zig");
const net_mod = @import("network.zig");
const mod_mod = @import("module.zig");
const svc_mod = @import("services.zig");

// ─────────────────────────────────────────────────────────────────────────────
// Shared kernel primitives  (canonical — must stay consistent with other files)
// ─────────────────────────────────────────────────────────────────────────────

pub const Role = enum { root, host, server, module_runtime, user };

pub const Permission = enum {
    kernel_admin,
    schedule_tasks,
    manage_modules,
    manage_memory,
    manage_processes,
    manage_files,
    read_audit,
};

pub fn hasPermission(role: Role, permission: Permission) bool {
    return switch (role) {
        .root => true,
        .host => switch (permission) {
            .kernel_admin, .schedule_tasks, .manage_modules, .manage_memory, .manage_processes, .manage_files, .read_audit => true,
        },
        .server => switch (permission) {
            .schedule_tasks, .manage_modules, .manage_processes, .manage_files => true,
            .kernel_admin, .manage_memory, .read_audit => false,
        },
        .module_runtime => switch (permission) {
            .schedule_tasks, .manage_files => true,
            .kernel_admin, .manage_modules, .manage_memory, .manage_processes, .read_audit => false,
        },
        .user => permission == .read_audit,
    };
}

// ─────────────────────────────────────────────────────────────────────────────
// Errors
// ─────────────────────────────────────────────────────────────────────────────

pub const ResourceError = error{
    AccessDenied,
    TenantNotFound,
    TenantAlreadyRegistered,
    GlobalCeilingExceeded,
    TenantLimitExceeded,
    InvalidDimension,
    InvalidRelease,
    EventLogFull,
    SubsystemError,
    BridgeNotReady,
};

// ─────────────────────────────────────────────────────────────────────────────
// §1  Resource dimensions
// ─────────────────────────────────────────────────────────────────────────────

/// Every axis of resource that the kernel tracks.
pub const ResourceDimension = enum {
    /// Heap-allocated bytes (backed by MemoryManager).
    memory_bytes,
    /// Logical process slots (backed by ProcessTable).
    processes,
    /// File-descriptor slots (tracked inline).
    files,
    /// Generic quota units (application-defined cost tokens).
    units,
    /// Network connection slots (backed by ConnectionPool).
    connections,
    /// Network ingress bytes (counter only — no cap by default).
    network_ingress,
    /// Network egress bytes (counter only — no cap by default).
    network_egress,
};

// ─────────────────────────────────────────────────────────────────────────────
// §2  Policy & usage records
// ─────────────────────────────────────────────────────────────────────────────

/// Hard caps for a single tenant across every dimension.
/// A cap of 0 means "unlimited" for counter-only dimensions
/// (network_ingress, network_egress).
pub const ResourcePolicy = struct {
    memory_bytes: u64 = 512 * 1024 * 1024,
    max_processes: u32 = 128,
    max_files: u32 = 5_000,
    max_units: u32 = 10_000,
    max_connections: u32 = 64,
};

/// Live consumption counters — one per ResourceDimension.
pub const TenantUsage = struct {
    memory_bytes: u64 = 0,
    process_count: u32 = 0,
    file_count: u32 = 0,
    unit_count: u32 = 0,
    connection_count: u32 = 0,
    ingress_bytes: u64 = 0,
    egress_bytes: u64 = 0,
};

// ─────────────────────────────────────────────────────────────────────────────
// §3  Declarative acquire / release specs
// ─────────────────────────────────────────────────────────────────────────────

/// A single dimension's acquisition amount.
pub const DimensionAcquire = union(ResourceDimension) {
    memory_bytes: u64,
    processes: u32,
    files: u32,
    units: u32,
    connections: u32,
    network_ingress: u64,
    network_egress: u64,
};

/// Declarative multi-dimension acquire request.
pub const AllocationRequest = struct {
    tenant_id: []const u8, // borrowed — caller keeps alive
    dimensions: []const DimensionAcquire,
};

/// Declarative multi-dimension release request (symmetric to AllocationRequest).
pub const ReleaseRequest = AllocationRequest;

// ─────────────────────────────────────────────────────────────────────────────
// §4  Resource event log
// ─────────────────────────────────────────────────────────────────────────────

pub const ResourceEventKind = enum {
    acquired,
    released,
    tenant_registered,
    tenant_removed,
    limit_exceeded,
    global_ceiling_hit,
};

pub const ResourceEvent = struct {
    id: u64,
    kind: ResourceEventKind,
    tenant_id: []const u8, // owned by ResourceEventLog
    dimension: ?ResourceDimension,
    amount: u64,
    emitted_ms: i64,
};

/// Bounded, append-only log of every resource operation.
pub const ResourceEventLog = struct {
    allocator: std.mem.Allocator,
    events: std.ArrayList(ResourceEvent),
    capacity: usize,
    next_id: u64,

    pub fn init(allocator: std.mem.Allocator, capacity: usize) ResourceEventLog {
        return .{
            .allocator = allocator,
            .events = std.ArrayList(ResourceEvent).init(allocator),
            .capacity = capacity,
            .next_id = 1,
        };
    }

    pub fn deinit(self: *ResourceEventLog) void {
        for (self.events.items) |e| self.allocator.free(e.tenant_id);
        self.events.deinit();
    }

    pub fn record(
        self: *ResourceEventLog,
        kind: ResourceEventKind,
        tenant_id: []const u8,
        dimension: ?ResourceDimension,
        amount: u64,
    ) !void {
        if (self.events.items.len >= self.capacity) return ResourceError.EventLogFull;
        const tid = try self.allocator.dupe(u8, tenant_id);
        errdefer self.allocator.free(tid);
        const id = self.next_id;
        self.next_id += 1;
        try self.events.append(.{
            .id = id,
            .kind = kind,
            .tenant_id = tid,
            .dimension = dimension,
            .amount = amount,
            .emitted_ms = std.time.milliTimestamp(),
        });
    }

    /// Collect all events with matching `kind`.
    pub fn collect(
        self: *const ResourceEventLog,
        kind: ResourceEventKind,
        out: *std.ArrayList(ResourceEvent),
    ) !void {
        for (self.events.items) |e| {
            if (e.kind == kind) try out.append(e);
        }
    }

    pub fn len(self: *const ResourceEventLog) usize {
        return self.events.items.len;
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// §5  Tenant record
// ─────────────────────────────────────────────────────────────────────────────

const Tenant = struct {
    id: []const u8, // owned by TenantRegistry
    policy: ResourcePolicy,
    usage: TenantUsage,
};

// ─────────────────────────────────────────────────────────────────────────────
// §6  Tenant Registry
// ─────────────────────────────────────────────────────────────────────────────

/// Owns all per-tenant policy and live-usage records.
pub const TenantRegistry = struct {
    allocator: std.mem.Allocator,
    tenants: std.StringHashMap(Tenant),

    pub fn init(allocator: std.mem.Allocator) TenantRegistry {
        return .{
            .allocator = allocator,
            .tenants = std.StringHashMap(Tenant).init(allocator),
        };
    }

    pub fn deinit(self: *TenantRegistry) void {
        var it = self.tenants.valueIterator();
        while (it.next()) |t| self.allocator.free(t.id);
        self.tenants.deinit();
    }

    pub fn register(self: *TenantRegistry, id: []const u8, policy: ResourcePolicy) !void {
        if (self.tenants.contains(id)) return ResourceError.TenantAlreadyRegistered;
        const id_copy = try self.allocator.dupe(u8, id);
        errdefer self.allocator.free(id_copy);
        try self.tenants.put(id_copy, .{ .id = id_copy, .policy = policy, .usage = .{} });
    }

    pub fn remove(self: *TenantRegistry, id: []const u8) !void {
        const t = self.tenants.getPtr(id) orelse return ResourceError.TenantNotFound;
        self.allocator.free(t.id);
        _ = self.tenants.remove(id);
    }

    pub fn setPolicy(self: *TenantRegistry, id: []const u8, policy: ResourcePolicy) !void {
        const t = self.tenants.getPtr(id) orelse return ResourceError.TenantNotFound;
        t.policy = policy;
    }

    pub fn getUsage(self: *const TenantRegistry, id: []const u8) ?TenantUsage {
        return if (self.tenants.get(id)) |t| t.usage else null;
    }

    pub fn getPtr(self: *TenantRegistry, id: []const u8) ?*Tenant {
        return self.tenants.getPtr(id);
    }

    pub fn count(self: *const TenantRegistry) usize {
        return self.tenants.count();
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// §7  Global pool  (system-wide ceilings)
// ─────────────────────────────────────────────────────────────────────────────

/// System-wide hard ceilings.  These are checked before per-tenant limits.
pub const GlobalCeilings = struct {
    memory_bytes: u64 = 4 * 1024 * 1024 * 1024,
    max_processes: u32 = 4096,
    max_connections: u32 = 8192,
};

pub const GlobalUsage = struct {
    memory_bytes: u64 = 0,
    process_count: u32 = 0,
    connection_count: u32 = 0,
};

pub const GlobalPool = struct {
    ceilings: GlobalCeilings,
    usage: GlobalUsage,

    pub fn init(ceilings: GlobalCeilings) GlobalPool {
        return .{ .ceilings = ceilings, .usage = .{} };
    }

    pub fn acquireMemory(self: *GlobalPool, bytes: u64) !void {
        if (self.usage.memory_bytes + bytes > self.ceilings.memory_bytes)
            return ResourceError.GlobalCeilingExceeded;
        self.usage.memory_bytes += bytes;
    }

    pub fn releaseMemory(self: *GlobalPool, bytes: u64) void {
        self.usage.memory_bytes = self.usage.memory_bytes -| bytes;
    }

    pub fn acquireProcesses(self: *GlobalPool, n: u32) !void {
        if (self.usage.process_count + n > self.ceilings.max_processes)
            return ResourceError.GlobalCeilingExceeded;
        self.usage.process_count += n;
    }

    pub fn releaseProcesses(self: *GlobalPool, n: u32) void {
        self.usage.process_count = self.usage.process_count -| n;
    }

    pub fn acquireConnections(self: *GlobalPool, n: u32) !void {
        if (self.usage.connection_count + n > self.ceilings.max_connections)
            return ResourceError.GlobalCeilingExceeded;
        self.usage.connection_count += n;
    }

    pub fn releaseConnections(self: *GlobalPool, n: u32) void {
        self.usage.connection_count = self.usage.connection_count -| n;
    }

    pub fn utilisation(self: *const GlobalPool) struct { mem: u8, procs: u8, conns: u8 } {
        const mem_pct: u8 = if (self.ceilings.memory_bytes > 0)
            @intCast(@min(100, self.usage.memory_bytes * 100 / self.ceilings.memory_bytes))
        else
            0;
        const proc_pct: u8 = if (self.ceilings.max_processes > 0)
            @intCast(@min(100, self.usage.process_count * 100 / self.ceilings.max_processes))
        else
            0;
        const conn_pct: u8 = if (self.ceilings.max_connections > 0)
            @intCast(@min(100, self.usage.connection_count * 100 / self.ceilings.max_connections))
        else
            0;
        return .{ .mem = mem_pct, .procs = proc_pct, .conns = conn_pct };
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// §8  Resource Enforcer  (validates acquire / release)
// ─────────────────────────────────────────────────────────────────────────────

/// Validates every acquire/release against both global ceilings and
/// per-tenant policy.  All mutations are applied atomically (or rolled back
/// on the first failure via errdefer).
pub const ResourceEnforcer = struct {
    pool: *GlobalPool,
    registry: *TenantRegistry,
    log: *ResourceEventLog,

    pub fn init(
        pool: *GlobalPool,
        registry: *TenantRegistry,
        log: *ResourceEventLog,
    ) ResourceEnforcer {
        return .{ .pool = pool, .registry = registry, .log = log };
    }

    // ── Memory ────────────────────────────────────────────────────────────

    pub fn acquireMemory(self: *ResourceEnforcer, actor: Role, tenant_id: []const u8, bytes: u64) !void {
        if (!hasPermission(actor, .manage_memory)) return ResourceError.AccessDenied;
        const t = self.registry.getPtr(tenant_id) orelse return ResourceError.TenantNotFound;
        if (t.usage.memory_bytes + bytes > t.policy.memory_bytes)
            return ResourceError.TenantLimitExceeded;
        try self.pool.acquireMemory(bytes);
        errdefer self.pool.releaseMemory(bytes);
        t.usage.memory_bytes += bytes;
        self.log.record(.acquired, tenant_id, .memory_bytes, bytes) catch {};
    }

    pub fn releaseMemory(self: *ResourceEnforcer, actor: Role, tenant_id: []const u8, bytes: u64) !void {
        if (!hasPermission(actor, .manage_memory)) return ResourceError.AccessDenied;
        const t = self.registry.getPtr(tenant_id) orelse return ResourceError.TenantNotFound;
        const actual = @min(bytes, t.usage.memory_bytes);
        t.usage.memory_bytes -= actual;
        self.pool.releaseMemory(actual);
        self.log.record(.released, tenant_id, .memory_bytes, actual) catch {};
    }

    pub fn releaseAllMemory(self: *ResourceEnforcer, tenant_id: []const u8) void {
        const t = self.registry.getPtr(tenant_id) orelse return;
        self.pool.releaseMemory(t.usage.memory_bytes);
        t.usage.memory_bytes = 0;
    }

    // ── Processes ─────────────────────────────────────────────────────────

    pub fn acquireProcesses(self: *ResourceEnforcer, actor: Role, tenant_id: []const u8, n: u32) !void {
        if (!hasPermission(actor, .manage_processes)) return ResourceError.AccessDenied;
        const t = self.registry.getPtr(tenant_id) orelse return ResourceError.TenantNotFound;
        if (t.usage.process_count + n > t.policy.max_processes)
            return ResourceError.TenantLimitExceeded;
        try self.pool.acquireProcesses(n);
        errdefer self.pool.releaseProcesses(n);
        t.usage.process_count += n;
        self.log.record(.acquired, tenant_id, .processes, n) catch {};
    }

    pub fn releaseProcesses(self: *ResourceEnforcer, actor: Role, tenant_id: []const u8, n: u32) !void {
        if (!hasPermission(actor, .manage_processes)) return ResourceError.AccessDenied;
        const t = self.registry.getPtr(tenant_id) orelse return ResourceError.TenantNotFound;
        const actual = @min(n, t.usage.process_count);
        t.usage.process_count -= actual;
        self.pool.releaseProcesses(actual);
        self.log.record(.released, tenant_id, .processes, actual) catch {};
    }

    pub fn releaseAllProcesses(self: *ResourceEnforcer, tenant_id: []const u8) void {
        const t = self.registry.getPtr(tenant_id) orelse return;
        self.pool.releaseProcesses(t.usage.process_count);
        t.usage.process_count = 0;
    }

    // ── Files ─────────────────────────────────────────────────────────────

    pub fn acquireFiles(self: *ResourceEnforcer, actor: Role, tenant_id: []const u8, n: u32) !void {
        if (!hasPermission(actor, .manage_files)) return ResourceError.AccessDenied;
        const t = self.registry.getPtr(tenant_id) orelse return ResourceError.TenantNotFound;
        if (t.usage.file_count + n > t.policy.max_files)
            return ResourceError.TenantLimitExceeded;
        t.usage.file_count += n;
        self.log.record(.acquired, tenant_id, .files, n) catch {};
    }

    pub fn releaseFiles(self: *ResourceEnforcer, actor: Role, tenant_id: []const u8, n: u32) !void {
        if (!hasPermission(actor, .manage_files)) return ResourceError.AccessDenied;
        const t = self.registry.getPtr(tenant_id) orelse return ResourceError.TenantNotFound;
        const actual = @min(n, t.usage.file_count);
        t.usage.file_count -= actual;
        self.log.record(.released, tenant_id, .files, actual) catch {};
    }

    pub fn releaseAllFiles(self: *ResourceEnforcer, tenant_id: []const u8) void {
        const t = self.registry.getPtr(tenant_id) orelse return;
        t.usage.file_count = 0;
    }

    // ── Generic units ─────────────────────────────────────────────────────

    pub fn acquireUnits(self: *ResourceEnforcer, actor: Role, tenant_id: []const u8, n: u32) !void {
        if (!hasPermission(actor, .manage_modules)) return ResourceError.AccessDenied;
        const t = self.registry.getPtr(tenant_id) orelse return ResourceError.TenantNotFound;
        if (t.usage.unit_count + n > t.policy.max_units)
            return ResourceError.TenantLimitExceeded;
        t.usage.unit_count += n;
        self.log.record(.acquired, tenant_id, .units, n) catch {};
    }

    pub fn releaseUnits(self: *ResourceEnforcer, actor: Role, tenant_id: []const u8, n: u32) !void {
        if (!hasPermission(actor, .manage_modules)) return ResourceError.AccessDenied;
        const t = self.registry.getPtr(tenant_id) orelse return ResourceError.TenantNotFound;
        const actual = @min(n, t.usage.unit_count);
        t.usage.unit_count -= actual;
        self.log.record(.released, tenant_id, .units, actual) catch {};
    }

    pub fn releaseAllUnits(self: *ResourceEnforcer, tenant_id: []const u8) void {
        const t = self.registry.getPtr(tenant_id) orelse return;
        t.usage.unit_count = 0;
    }

    // ── Connections ───────────────────────────────────────────────────────

    pub fn acquireConnections(self: *ResourceEnforcer, actor: Role, tenant_id: []const u8, n: u32) !void {
        if (!hasPermission(actor, .manage_modules)) return ResourceError.AccessDenied;
        const t = self.registry.getPtr(tenant_id) orelse return ResourceError.TenantNotFound;
        if (t.usage.connection_count + n > t.policy.max_connections)
            return ResourceError.TenantLimitExceeded;
        try self.pool.acquireConnections(n);
        errdefer self.pool.releaseConnections(n);
        t.usage.connection_count += n;
        self.log.record(.acquired, tenant_id, .connections, n) catch {};
    }

    pub fn releaseConnections(self: *ResourceEnforcer, actor: Role, tenant_id: []const u8, n: u32) !void {
        if (!hasPermission(actor, .manage_modules)) return ResourceError.AccessDenied;
        const t = self.registry.getPtr(tenant_id) orelse return ResourceError.TenantNotFound;
        const actual = @min(n, t.usage.connection_count);
        t.usage.connection_count -= actual;
        self.pool.releaseConnections(actual);
        self.log.record(.released, tenant_id, .connections, actual) catch {};
    }

    pub fn releaseAllConnections(self: *ResourceEnforcer, tenant_id: []const u8) void {
        const t = self.registry.getPtr(tenant_id) orelse return;
        self.pool.releaseConnections(t.usage.connection_count);
        t.usage.connection_count = 0;
    }

    // ── Network traffic counters (no global ceiling) ──────────────────────

    pub fn recordIngress(self: *ResourceEnforcer, actor: Role, tenant_id: []const u8, bytes: u64) !void {
        if (!hasPermission(actor, .manage_modules)) return ResourceError.AccessDenied;
        const t = self.registry.getPtr(tenant_id) orelse return ResourceError.TenantNotFound;
        t.usage.ingress_bytes += bytes;
        self.log.record(.acquired, tenant_id, .network_ingress, bytes) catch {};
    }

    pub fn recordEgress(self: *ResourceEnforcer, actor: Role, tenant_id: []const u8, bytes: u64) !void {
        if (!hasPermission(actor, .manage_modules)) return ResourceError.AccessDenied;
        const t = self.registry.getPtr(tenant_id) orelse return ResourceError.TenantNotFound;
        t.usage.egress_bytes += bytes;
        self.log.record(.acquired, tenant_id, .network_egress, bytes) catch {};
    }

    // ── Bulk teardown ─────────────────────────────────────────────────────

    pub fn releaseAll(self: *ResourceEnforcer, tenant_id: []const u8) void {
        self.releaseAllMemory(tenant_id);
        self.releaseAllProcesses(tenant_id);
        self.releaseAllFiles(tenant_id);
        self.releaseAllUnits(tenant_id);
        self.releaseAllConnections(tenant_id);
        if (self.registry.getPtr(tenant_id)) |t| {
            t.usage.ingress_bytes = 0;
            t.usage.egress_bytes = 0;
        }
        self.log.record(.released, tenant_id, null, 0) catch {};
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// §9  Subsystem Bridge  (optional — forwards into actual subsystem objects)
// ─────────────────────────────────────────────────────────────────────────────

/// Optional pointers to live subsystem objects.  When set, the bridge
/// forwards acquire/release calls into the real subsystem so the ResourcesManager
/// acts as a drop-in replacement for per-subsystem resource calls.
///
/// All pointers are optional; the bridge silently skips missing subsystems.
pub const SubsystemBridge = struct {
    memory_manager: ?*mem_mod.MemoryManager = null,
    process_table: ?*proc_mod.ProcessTable = null,
    network_ledger: ?*net_mod.TrafficLedger = null,
    module_registry: ?*mod_mod.ModuleRegistry = null,

    /// Forward a memory acquire into MemoryManager (tenant-scoped).
    pub fn forwardMemoryAcquire(self: *SubsystemBridge, tenant_id: []const u8, bytes: u64) void {
        if (self.memory_manager) |mgr| {
            mgr.allocateTenant(tenant_id, bytes) catch {};
        }
    }

    /// Forward a memory release into MemoryManager.
    pub fn forwardMemoryRelease(self: *SubsystemBridge, tenant_id: []const u8, bytes: u64) void {
        if (self.memory_manager) |mgr| {
            mgr.freeTenant(tenant_id, bytes) catch {};
        }
    }

    /// Forward traffic recording into the NetworkManager's TrafficLedger.
    pub fn forwardIngress(self: *SubsystemBridge, component_id: []const u8, bytes: u64) void {
        if (self.network_ledger) |ledger| {
            ledger.recordIngress(component_id, bytes) catch {};
        }
    }

    pub fn forwardEgress(self: *SubsystemBridge, component_id: []const u8, bytes: u64) void {
        if (self.network_ledger) |ledger| {
            ledger.recordEgress(component_id, bytes) catch {};
        }
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// §10  ResourcesManager  (top-level public API)
// ─────────────────────────────────────────────────────────────────────────────

/// Top-level coordinator.  All kernel code that needs to acquire or release
/// resources should go through ResourcesManager rather than calling individual
/// subsystem managers directly.
pub const ResourcesManager = struct {
    allocator: std.mem.Allocator,

    pool: GlobalPool,
    registry: TenantRegistry,
    log: ResourceEventLog,
    enforcer: ResourceEnforcer,
    bridge: SubsystemBridge,

    pub fn init(
        allocator: std.mem.Allocator,
        ceilings: GlobalCeilings,
        log_cap: usize,
    ) ResourcesManager {
        var pool = GlobalPool.init(ceilings);
        var reg = TenantRegistry.init(allocator);
        var log = ResourceEventLog.init(allocator, log_cap);
        const enf = ResourceEnforcer.init(&pool, &reg, &log);
        return .{
            .allocator = allocator,
            .pool = pool,
            .registry = reg,
            .log = log,
            .enforcer = enf,
            .bridge = .{},
        };
    }

    /// Re-seat internal cross-references after the struct is moved into
    /// its final storage location.  Call once immediately after init.
    pub fn relink(self: *ResourcesManager) void {
        self.enforcer = ResourceEnforcer.init(&self.pool, &self.registry, &self.log);
    }

    pub fn deinit(self: *ResourcesManager) void {
        self.registry.deinit();
        self.log.deinit();
    }

    // ── Tenant management ─────────────────────────────────────────────────

    /// Register a tenant with a given resource policy.
    pub fn registerTenant(
        self: *ResourcesManager,
        actor: Role,
        id: []const u8,
        policy: ResourcePolicy,
    ) !void {
        if (!hasPermission(actor, .manage_modules)) return ResourceError.AccessDenied;
        try self.registry.register(id, policy);
        try self.log.record(.tenant_registered, id, null, 0);
    }

    pub fn removeTenant(self: *ResourcesManager, actor: Role, id: []const u8) !void {
        if (!hasPermission(actor, .manage_modules)) return ResourceError.AccessDenied;
        self.enforcer.releaseAll(id);
        try self.registry.remove(id);
        try self.log.record(.tenant_removed, id, null, 0);
    }

    pub fn setPolicy(self: *ResourcesManager, actor: Role, id: []const u8, policy: ResourcePolicy) !void {
        if (!hasPermission(actor, .manage_modules)) return ResourceError.AccessDenied;
        try self.registry.setPolicy(id, policy);
    }

    // ── Declarative acquire / release ─────────────────────────────────────

    /// Execute an AllocationRequest.  Processes steps in order; if any step
    /// fails the caller should call releaseAll to undo partial allocations.
    pub fn acquire(self: *ResourcesManager, actor: Role, req: AllocationRequest) !void {
        for (req.dimensions) |dim| {
            switch (dim) {
                .memory_bytes => |b| try self.enforcer.acquireMemory(actor, req.tenant_id, b),
                .processes => |n| try self.enforcer.acquireProcesses(actor, req.tenant_id, n),
                .files => |n| try self.enforcer.acquireFiles(actor, req.tenant_id, n),
                .units => |n| try self.enforcer.acquireUnits(actor, req.tenant_id, n),
                .connections => |n| try self.enforcer.acquireConnections(actor, req.tenant_id, n),
                .network_ingress => |b| try self.enforcer.recordIngress(actor, req.tenant_id, b),
                .network_egress => |b| try self.enforcer.recordEgress(actor, req.tenant_id, b),
            }
        }
    }

    /// Execute a ReleaseRequest.  All releases are best-effort.
    pub fn release(self: *ResourcesManager, actor: Role, req: ReleaseRequest) void {
        for (req.dimensions) |dim| {
            switch (dim) {
                .memory_bytes => |b| self.enforcer.releaseMemory(actor, req.tenant_id, b) catch {},
                .processes => |n| self.enforcer.releaseProcesses(actor, req.tenant_id, n) catch {},
                .files => |n| self.enforcer.releaseFiles(actor, req.tenant_id, n) catch {},
                .units => |n| self.enforcer.releaseUnits(actor, req.tenant_id, n) catch {},
                .connections => |n| self.enforcer.releaseConnections(actor, req.tenant_id, n) catch {},
                .network_ingress => {},
                .network_egress => {},
            }
        }
    }

    /// Release ALL resources held by a tenant.
    pub fn releaseAll(self: *ResourcesManager, tenant_id: []const u8) void {
        self.enforcer.releaseAll(tenant_id);
    }

    // ── Individual acquire helpers (convenience wrappers) ─────────────────

    pub fn acquireMemory(self: *ResourcesManager, actor: Role, id: []const u8, bytes: u64) !void {
        try self.enforcer.acquireMemory(actor, id, bytes);
        self.bridge.forwardMemoryAcquire(id, bytes);
    }

    pub fn releaseMemory(self: *ResourcesManager, actor: Role, id: []const u8, bytes: u64) !void {
        try self.enforcer.releaseMemory(actor, id, bytes);
        self.bridge.forwardMemoryRelease(id, bytes);
    }

    pub fn acquireProcess(self: *ResourcesManager, actor: Role, id: []const u8) !void {
        try self.enforcer.acquireProcesses(actor, id, 1);
    }

    pub fn releaseProcess(self: *ResourcesManager, actor: Role, id: []const u8) !void {
        try self.enforcer.releaseProcesses(actor, id, 1);
    }

    pub fn acquireFile(self: *ResourcesManager, actor: Role, id: []const u8) !void {
        try self.enforcer.acquireFiles(actor, id, 1);
    }

    pub fn releaseFile(self: *ResourcesManager, actor: Role, id: []const u8) !void {
        try self.enforcer.releaseFiles(actor, id, 1);
    }

    pub fn acquireUnits(self: *ResourcesManager, actor: Role, id: []const u8, n: u32) !void {
        try self.enforcer.acquireUnits(actor, id, n);
    }

    pub fn releaseUnits(self: *ResourcesManager, actor: Role, id: []const u8, n: u32) !void {
        try self.enforcer.releaseUnits(actor, id, n);
    }

    pub fn acquireConnection(self: *ResourcesManager, actor: Role, id: []const u8) !void {
        try self.enforcer.acquireConnections(actor, id, 1);
    }

    pub fn releaseConnection(self: *ResourcesManager, actor: Role, id: []const u8) !void {
        try self.enforcer.releaseConnections(actor, id, 1);
    }

    pub fn recordIngress(self: *ResourcesManager, actor: Role, id: []const u8, bytes: u64) !void {
        try self.enforcer.recordIngress(actor, id, bytes);
        self.bridge.forwardIngress(id, bytes);
    }

    pub fn recordEgress(self: *ResourcesManager, actor: Role, id: []const u8, bytes: u64) !void {
        try self.enforcer.recordEgress(actor, id, bytes);
        self.bridge.forwardEgress(id, bytes);
    }

    // ── Subsystem bridge ──────────────────────────────────────────────────

    /// Attach live subsystem objects so the bridge can forward calls.
    pub fn attachMemoryManager(self: *ResourcesManager, mgr: *mem_mod.MemoryManager) void {
        self.bridge.memory_manager = mgr;
    }

    pub fn attachTrafficLedger(self: *ResourcesManager, ledger: *net_mod.TrafficLedger) void {
        self.bridge.network_ledger = ledger;
    }

    pub fn attachProcessTable(self: *ResourcesManager, table: *proc_mod.ProcessTable) void {
        self.bridge.process_table = table;
    }

    pub fn attachModuleRegistry(self: *ResourcesManager, reg: *mod_mod.ModuleRegistry) void {
        self.bridge.module_registry = reg;
    }

    // ── Queries ───────────────────────────────────────────────────────────

    pub fn getUsage(self: *const ResourcesManager, id: []const u8) ?TenantUsage {
        return self.registry.getUsage(id);
    }

    pub fn globalUtilisation(self: *const ResourcesManager) struct { mem: u8, procs: u8, conns: u8 } {
        return self.pool.utilisation();
    }

    // ── Stats ─────────────────────────────────────────────────────────────

    pub const Stats = struct {
        tenant_count: usize,
        global_memory_used: u64,
        global_memory_cap: u64,
        global_process_count: u32,
        global_process_cap: u32,
        global_conn_count: u32,
        global_conn_cap: u32,
        event_count: usize,
    };

    pub fn stats(self: *const ResourcesManager) Stats {
        return .{
            .tenant_count = self.registry.count(),
            .global_memory_used = self.pool.usage.memory_bytes,
            .global_memory_cap = self.pool.ceilings.memory_bytes,
            .global_process_count = self.pool.usage.process_count,
            .global_process_cap = self.pool.ceilings.max_processes,
            .global_conn_count = self.pool.usage.connection_count,
            .global_conn_cap = self.pool.ceilings.max_connections,
            .event_count = self.log.len(),
        };
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

test "tenant registration and deregistration" {
    var mgr = ResourcesManager.init(std.testing.allocator, .{}, 256);
    mgr.relink();
    defer mgr.deinit();

    try mgr.registerTenant(.host, "mod-a", .{});
    try std.testing.expectEqual(@as(usize, 1), mgr.registry.count());

    try std.testing.expectError(
        ResourceError.TenantAlreadyRegistered,
        mgr.registerTenant(.host, "mod-a", .{}),
    );

    try mgr.removeTenant(.host, "mod-a");
    try std.testing.expectEqual(@as(usize, 0), mgr.registry.count());
}

test "RBAC: unprivileged role denied" {
    var mgr = ResourcesManager.init(std.testing.allocator, .{}, 64);
    mgr.relink();
    defer mgr.deinit();

    try std.testing.expectError(
        ResourceError.AccessDenied,
        mgr.registerTenant(.user, "bad-actor", .{}),
    );
}

test "memory: per-tenant limit enforced" {
    var mgr = ResourcesManager.init(std.testing.allocator, .{}, 64);
    mgr.relink();
    defer mgr.deinit();

    try mgr.registerTenant(.host, "mod-a", .{ .memory_bytes = 1000 });
    try mgr.acquireMemory(.host, "mod-a", 800);
    try std.testing.expectError(
        ResourceError.TenantLimitExceeded,
        mgr.acquireMemory(.host, "mod-a", 300),
    );
    try mgr.releaseMemory(.host, "mod-a", 800);
    const u = mgr.getUsage("mod-a").?;
    try std.testing.expectEqual(@as(u64, 0), u.memory_bytes);
}

test "memory: global ceiling enforced" {
    var mgr = ResourcesManager.init(std.testing.allocator, .{ .memory_bytes = 500 }, 64);
    mgr.relink();
    defer mgr.deinit();

    try mgr.registerTenant(.host, "a", .{ .memory_bytes = 1000 });
    try mgr.registerTenant(.host, "b", .{ .memory_bytes = 1000 });

    try mgr.acquireMemory(.host, "a", 400);
    try std.testing.expectError(
        ResourceError.GlobalCeilingExceeded,
        mgr.acquireMemory(.host, "b", 200),
    );
}

test "processes: limit enforced" {
    var mgr = ResourcesManager.init(std.testing.allocator, .{}, 64);
    mgr.relink();
    defer mgr.deinit();

    try mgr.registerTenant(.host, "mod-p", .{ .max_processes = 2 });
    try mgr.acquireProcess(.host, "mod-p");
    try mgr.acquireProcess(.host, "mod-p");
    try std.testing.expectError(
        ResourceError.TenantLimitExceeded,
        mgr.acquireProcess(.host, "mod-p"),
    );
}

test "files: limit enforced" {
    var mgr = ResourcesManager.init(std.testing.allocator, .{}, 64);
    mgr.relink();
    defer mgr.deinit();

    try mgr.registerTenant(.host, "mod-f", .{ .max_files = 1 });
    try mgr.acquireFile(.host, "mod-f");
    try std.testing.expectError(
        ResourceError.TenantLimitExceeded,
        mgr.acquireFile(.host, "mod-f"),
    );
}

test "units: limit enforced" {
    var mgr = ResourcesManager.init(std.testing.allocator, .{}, 64);
    mgr.relink();
    defer mgr.deinit();

    try mgr.registerTenant(.host, "mod-u", .{ .max_units = 10 });
    try mgr.acquireUnits(.host, "mod-u", 8);
    try std.testing.expectError(
        ResourceError.TenantLimitExceeded,
        mgr.acquireUnits(.host, "mod-u", 5),
    );
}

test "connections: limit enforced" {
    var mgr = ResourcesManager.init(std.testing.allocator, .{}, 64);
    mgr.relink();
    defer mgr.deinit();

    try mgr.registerTenant(.host, "mod-c", .{ .max_connections = 1 });
    try mgr.acquireConnection(.host, "mod-c");
    try std.testing.expectError(
        ResourceError.TenantLimitExceeded,
        mgr.acquireConnection(.host, "mod-c"),
    );
}

test "network traffic counters accumulate" {
    var mgr = ResourcesManager.init(std.testing.allocator, .{}, 64);
    mgr.relink();
    defer mgr.deinit();

    try mgr.registerTenant(.host, "svc", .{});
    try mgr.recordIngress(.host, "svc", 1024);
    try mgr.recordEgress(.host, "svc", 2048);
    const u = mgr.getUsage("svc").?;
    try std.testing.expectEqual(@as(u64, 1024), u.ingress_bytes);
    try std.testing.expectEqual(@as(u64, 2048), u.egress_bytes);
}

test "releaseAll zeroes every dimension" {
    var mgr = ResourcesManager.init(std.testing.allocator, .{}, 128);
    mgr.relink();
    defer mgr.deinit();

    try mgr.registerTenant(.host, "big", .{});
    try mgr.acquireMemory(.host, "big", 100);
    try mgr.acquireProcess(.host, "big");
    try mgr.acquireFile(.host, "big");
    try mgr.acquireUnits(.host, "big", 5);
    try mgr.acquireConnection(.host, "big");
    try mgr.recordIngress(.host, "big", 512);
    try mgr.recordEgress(.host, "big", 256);

    mgr.releaseAll("big");

    const u = mgr.getUsage("big").?;
    try std.testing.expectEqual(@as(u64, 0), u.memory_bytes);
    try std.testing.expectEqual(@as(u32, 0), u.process_count);
    try std.testing.expectEqual(@as(u32, 0), u.file_count);
    try std.testing.expectEqual(@as(u32, 0), u.unit_count);
    try std.testing.expectEqual(@as(u32, 0), u.connection_count);
    try std.testing.expectEqual(@as(u64, 0), mgr.pool.usage.memory_bytes);
}

test "declarative AllocationRequest" {
    var mgr = ResourcesManager.init(std.testing.allocator, .{}, 128);
    mgr.relink();
    defer mgr.deinit();

    try mgr.registerTenant(.host, "decl", .{});

    const dims = [_]DimensionAcquire{
        .{ .memory_bytes = 4096 },
        .{ .processes = 2 },
        .{ .files = 3 },
        .{ .units = 10 },
    };
    try mgr.acquire(.host, .{ .tenant_id = "decl", .dimensions = &dims });

    const u = mgr.getUsage("decl").?;
    try std.testing.expectEqual(@as(u64, 4096), u.memory_bytes);
    try std.testing.expectEqual(@as(u32, 2), u.process_count);
    try std.testing.expectEqual(@as(u32, 3), u.file_count);
    try std.testing.expectEqual(@as(u32, 10), u.unit_count);
}

test "event log records tenant lifecycle and resource events" {
    var mgr = ResourcesManager.init(std.testing.allocator, .{}, 256);
    mgr.relink();
    defer mgr.deinit();

    try mgr.registerTenant(.host, "logged", .{});
    try mgr.acquireMemory(.host, "logged", 512);
    try mgr.releaseMemory(.host, "logged", 512);
    try mgr.removeTenant(.host, "logged");

    // At least: tenant_registered, acquired, released, tenant_removed
    try std.testing.expect(mgr.log.len() >= 4);

    var acquires = std.ArrayList(ResourceEvent).init(std.testing.allocator);
    defer acquires.deinit();
    try mgr.log.collect(.acquired, &acquires);
    try std.testing.expect(acquires.items.len >= 1);
}

test "stats reflect live state" {
    var mgr = ResourcesManager.init(std.testing.allocator, .{ .memory_bytes = 1024 }, 64);
    mgr.relink();
    defer mgr.deinit();

    try mgr.registerTenant(.host, "s1", .{ .memory_bytes = 512 });
    try mgr.registerTenant(.host, "s2", .{ .memory_bytes = 512 });
    try mgr.acquireMemory(.host, "s1", 200);
    try mgr.acquireMemory(.host, "s2", 300);

    const s = mgr.stats();
    try std.testing.expectEqual(@as(usize, 2), s.tenant_count);
    try std.testing.expectEqual(@as(u64, 500), s.global_memory_used);
    try std.testing.expectEqual(@as(u64, 1024), s.global_memory_cap);
}
