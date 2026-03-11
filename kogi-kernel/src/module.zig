//! module.zig — Kernel Module Management System
//!
//! Kernel extension extracted and generalised from kernel.zig.
//!
//! Unifies the two parallel tracking systems in kernel.zig (ModuleRecord +
//! ModuleIsolationContext vs. ComponentIsolationContext) into a single,
//! class-agnostic Module entity that covers every kind of kernel unit —
//! built-in platform components, user modules, plugins, services, engines, etc.
//!
//! Provides:
//!   ModuleClass         — taxonomy of kernel units (replaces ComponentClass)
//!   ModuleStatus        — operational state machine (replaces bool active/disabled)
//!   ResourceLimits      — per-module caps (memory, processes, files, resource units)
//!   ResourceUsage       — live counters mirroring every limit dimension
//!   NetworkAllocation   — per-module network bookkeeping (ingress/egress/endpoint)
//!   ModuleDescriptor    — canonical, unified module record
//!   ModuleRegistry      — authoritative register of all modules
//!   ResourceAllocator   — scoped acquire/release of every resource dimension
//!   LifecycleManager    — state transitions, dependency ordering, event emission
//!   ProvisionPlan       — declarative bootstrap helper for multi-resource setup
//!   ModuleSystem        — top-level coordinator wiring all subsystems together

const std = @import("std");

// ─────────────────────────────────────────────────────────────────────────────
// Shared kernel primitives  (self-contained — no imports from other extensions)
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

pub const ModuleError = error{
    AccessDenied,
    ModuleNotFound,
    ModuleAlreadyRegistered,
    ModuleNotActive,
    ModuleAlreadyStopped,
    DependencyNotMet,
    DependencyCycle,
    ResourceLimitExceeded,
    GlobalMemoryExhausted,
    InvalidStatusTransition,
    ProvisionStepFailed,
    EventQueueFull,
    CapabilityNotGranted,
};

// ─────────────────────────────────────────────────────────────────────────────
// 1.  Module taxonomy
// ─────────────────────────────────────────────────────────────────────────────

/// Replaces the split between ModuleRecord (modules map) and ComponentClass
/// (component_isolation map).  Every unit in the kernel — built-in or user —
/// has exactly one class.
pub const ModuleClass = enum {
    /// The kernel itself.
    kernel,
    /// Host runtime orchestrator.
    host,
    /// Network-facing server process.
    server,
    /// Stream/compute engine.
    engine,
    /// Shared platform services layer.
    services,
    /// General user-installed module.
    module,
    /// Lightweight plugin (sandboxed module with narrower default limits).
    plugin,
    /// Background daemon / system service.
    daemon,

    /// The Role that owns processes spawned on behalf of this class.
    pub fn ownerRole(self: ModuleClass) Role {
        return switch (self) {
            .kernel => .root,
            .host => .host,
            .server, .engine, .services => .server,
            .module, .plugin, .daemon => .module_runtime,
        };
    }

    /// Default resource limits appropriate for this class.
    pub fn defaultLimits(self: ModuleClass) ResourceLimits {
        return switch (self) {
            .kernel => .{ .memory_bytes = 1024 * 1024 * 1024, .max_processes = 512, .max_files = 20_000, .max_resource_units = 40_000, .max_network_connections = 1024 },
            .host => .{ .memory_bytes = 768 * 1024 * 1024, .max_processes = 384, .max_files = 16_000, .max_resource_units = 30_000, .max_network_connections = 512 },
            .server => .{ .memory_bytes = 768 * 1024 * 1024, .max_processes = 256, .max_files = 12_000, .max_resource_units = 24_000, .max_network_connections = 256 },
            .engine => .{ .memory_bytes = 1024 * 1024 * 1024, .max_processes = 256, .max_files = 12_000, .max_resource_units = 32_000, .max_network_connections = 256 },
            .services => .{ .memory_bytes = 768 * 1024 * 1024, .max_processes = 256, .max_files = 12_000, .max_resource_units = 24_000, .max_network_connections = 256 },
            .module => .{ .memory_bytes = 512 * 1024 * 1024, .max_processes = 128, .max_files = 5_000, .max_resource_units = 10_000, .max_network_connections = 64 },
            .plugin => .{ .memory_bytes = 64 * 1024 * 1024, .max_processes = 8, .max_files = 256, .max_resource_units = 500, .max_network_connections = 8 },
            .daemon => .{ .memory_bytes = 128 * 1024 * 1024, .max_processes = 16, .max_files = 1_000, .max_resource_units = 2_000, .max_network_connections = 32 },
        };
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// 2.  Status state machine
// ─────────────────────────────────────────────────────────────────────────────

/// Richer than kernel.zig's two-value ModuleStatus enum — covers the full
/// operational lifecycle of a module from registration to removal.
pub const ModuleStatus = enum {
    /// Registered but not yet started.
    registered,
    /// Provisioning resources (memory, processes, files, network).
    provisioning,
    /// Fully operational.
    active,
    /// Temporarily halted; resources retained.
    suspended,
    /// Graceful shutdown in progress.
    draining,
    /// Stopped; resources released; still registered.
    stopped,
    /// Deregistering — will be removed.
    deregistering,
    /// Encountered a fatal error.
    faulted,

    pub fn canTransitionTo(self: ModuleStatus, next: ModuleStatus) bool {
        return switch (self) {
            .registered => next == .provisioning or next == .stopped,
            .provisioning => next == .active or next == .faulted or next == .stopped,
            .active => next == .suspended or next == .draining or next == .faulted,
            .suspended => next == .active or next == .draining,
            .draining => next == .stopped or next == .faulted,
            .stopped => next == .provisioning or next == .deregistering,
            .deregistering => false,
            .faulted => next == .stopped or next == .deregistering,
        };
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// 3.  Resource limits and usage
// ─────────────────────────────────────────────────────────────────────────────

/// Per-module hard caps on every resource dimension.
pub const ResourceLimits = struct {
    memory_bytes: u64,
    max_processes: u32,
    max_files: u32,
    max_resource_units: u32,
    max_network_connections: u32,
};

/// Live counters for every limit dimension.
pub const ResourceUsage = struct {
    memory_bytes: u64 = 0,
    process_count: u32 = 0,
    file_count: u32 = 0,
    resource_units: u32 = 0,
    network_connections: u32 = 0,
};

/// Network bookkeeping: ingress/egress bytes, connection counts, endpoint.
pub const NetworkAllocation = struct {
    ingress_bytes: u64 = 0,
    egress_bytes: u64 = 0,
    endpoint: []const u8 = "", // owned by ModuleRegistry
    network_manager: []const u8 = "", // owned by ModuleRegistry
};

// ─────────────────────────────────────────────────────────────────────────────
// 4.  Capability set
// ─────────────────────────────────────────────────────────────────────────────

/// Fine-grained capabilities that can be individually granted or revoked per
/// module, independent of the coarser Role-based permission system.
pub const Capability = enum {
    /// May call into kernel-admin APIs.
    kernel_api,
    /// May spawn child processes.
    spawn_processes,
    /// May open network connections.
    network_access,
    /// May read/write the kernel event bus.
    event_bus,
    /// May register its own sub-modules.
    register_submodules,
    /// May request file system access.
    file_access,
    /// May request dynamic memory above initial allocation.
    dynamic_memory,
    /// May schedule recurring jobs.
    scheduling,
};

pub const CapabilitySet = std.EnumSet(Capability);

// ─────────────────────────────────────────────────────────────────────────────
// 5.  Module Descriptor  (the unified record)
// ─────────────────────────────────────────────────────────────────────────────

/// Canonical description of a single module.  Merges ModuleRecord,
/// ModuleIsolationContext, and ComponentIsolationContext from kernel.zig into
/// one coherent struct.
pub const ModuleDescriptor = struct {
    // Identity
    id: []const u8, // owned by ModuleRegistry
    kind: []const u8, // owned by ModuleRegistry; e.g. "office", "exchange"
    class: ModuleClass,
    version: []const u8, // owned by ModuleRegistry; semver string
    description: []const u8, // owned by ModuleRegistry

    // Lifecycle
    status: ModuleStatus,
    registered_ms: i64,
    last_status_ms: i64,
    start_count: u32, // how many times this module has been (re)started
    fault_count: u32,

    // Resources
    limits: ResourceLimits,
    usage: ResourceUsage,
    network: NetworkAllocation,

    // Security
    owner_role: Role,
    capabilities: CapabilitySet,

    // Dependency tracking (slice of module IDs — owned by ModuleRegistry)
    dependencies: [][]const u8,
    dep_count: usize,
};

// ─────────────────────────────────────────────────────────────────────────────
// 6.  Event bus  (in-module pub/sub for lifecycle events)
// ─────────────────────────────────────────────────────────────────────────────

pub const ModuleEvent = struct {
    topic: []const u8, // owned by EventBus
    payload: []const u8, // owned by EventBus
    source_id: []const u8, // borrowed — points into ModuleDescriptor.id
    emitted_ms: i64,
};

/// Bounded append-only event bus shared across all modules.
pub const EventBus = struct {
    allocator: std.mem.Allocator,
    events: std.ArrayList(ModuleEvent),
    capacity: usize,

    pub fn init(allocator: std.mem.Allocator, capacity: usize) EventBus {
        return .{
            .allocator = allocator,
            .events = std.ArrayList(ModuleEvent).init(allocator),
            .capacity = capacity,
        };
    }

    pub fn deinit(self: *EventBus) void {
        for (self.events.items) |e| {
            self.allocator.free(e.topic);
            self.allocator.free(e.payload);
        }
        self.events.deinit();
    }

    pub fn publish(
        self: *EventBus,
        source_id: []const u8,
        topic: []const u8,
        payload: []const u8,
    ) !void {
        if (self.events.items.len >= self.capacity) return ModuleError.EventQueueFull;

        const t = try self.allocator.dupe(u8, topic);
        errdefer self.allocator.free(t);
        const p = try self.allocator.dupe(u8, payload);
        errdefer self.allocator.free(p);

        try self.events.append(.{
            .topic = t,
            .payload = p,
            .source_id = source_id,
            .emitted_ms = std.time.milliTimestamp(),
        });
    }

    /// Collect events whose topic matches `prefix`.
    pub fn collect(
        self: *const EventBus,
        prefix: []const u8,
        out: *std.ArrayList(ModuleEvent),
    ) !void {
        for (self.events.items) |e| {
            if (std.mem.startsWith(u8, e.topic, prefix)) try out.append(e);
        }
    }

    pub fn len(self: *const EventBus) usize {
        return self.events.items.len;
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// 7.  Module Registry
// ─────────────────────────────────────────────────────────────────────────────

/// Internal storage limits for dependency arrays per module.
const MAX_DEPS_PER_MODULE = 32;

/// Owns all ModuleDescriptors and their heap-allocated strings.
pub const ModuleRegistry = struct {
    allocator: std.mem.Allocator,
    modules: std.StringHashMap(ModuleDescriptor),
    /// Global memory pool accounting.
    total_memory: u64,
    used_memory: u64,

    pub fn init(allocator: std.mem.Allocator, total_memory: u64) ModuleRegistry {
        return .{
            .allocator = allocator,
            .modules = std.StringHashMap(ModuleDescriptor).init(allocator),
            .total_memory = total_memory,
            .used_memory = 0,
        };
    }

    pub fn deinit(self: *ModuleRegistry) void {
        var it = self.modules.valueIterator();
        while (it.next()) |m| self.freeDescriptorStrings(m);
        self.modules.deinit();
    }

    fn freeDescriptorStrings(self: *ModuleRegistry, m: *ModuleDescriptor) void {
        self.allocator.free(m.id);
        self.allocator.free(m.kind);
        self.allocator.free(m.version);
        self.allocator.free(m.description);
        if (m.network.endpoint.len > 0) self.allocator.free(m.network.endpoint);
        if (m.network.network_manager.len > 0) self.allocator.free(m.network.network_manager);
        for (m.dependencies[0..m.dep_count]) |dep| self.allocator.free(dep);
        if (m.dep_count > 0) self.allocator.free(m.dependencies);
    }

    // ── Registration ──────────────────────────────────────────────────────

    pub fn register(
        self: *ModuleRegistry,
        actor: Role,
        id: []const u8,
        kind: []const u8,
        class: ModuleClass,
        version: []const u8,
        description: []const u8,
        endpoint: []const u8,
        net_manager: []const u8,
        caps: CapabilitySet,
    ) !void {
        if (!hasPermission(actor, .manage_modules)) return ModuleError.AccessDenied;
        if (self.modules.contains(id)) return ModuleError.ModuleAlreadyRegistered;

        const id_copy = try self.allocator.dupe(u8, id);
        errdefer self.allocator.free(id_copy);
        const kind_copy = try self.allocator.dupe(u8, kind);
        errdefer self.allocator.free(kind_copy);
        const ver_copy = try self.allocator.dupe(u8, version);
        errdefer self.allocator.free(ver_copy);
        const desc_copy = try self.allocator.dupe(u8, description);
        errdefer self.allocator.free(desc_copy);
        const ep_copy = try self.allocator.dupe(u8, endpoint);
        errdefer self.allocator.free(ep_copy);
        const nm_copy = try self.allocator.dupe(u8, net_manager);
        errdefer self.allocator.free(nm_copy);

        const now = std.time.milliTimestamp();

        try self.modules.put(id_copy, .{
            .id = id_copy,
            .kind = kind_copy,
            .class = class,
            .version = ver_copy,
            .description = desc_copy,
            .status = .registered,
            .registered_ms = now,
            .last_status_ms = now,
            .start_count = 0,
            .fault_count = 0,
            .limits = class.defaultLimits(),
            .usage = .{},
            .network = .{ .endpoint = ep_copy, .network_manager = nm_copy },
            .owner_role = class.ownerRole(),
            .capabilities = caps,
            .dependencies = &.{},
            .dep_count = 0,
        });
    }

    /// Convenience wrapper using class default limits and standard capabilities.
    pub fn registerSimple(
        self: *ModuleRegistry,
        actor: Role,
        id: []const u8,
        kind: []const u8,
        class: ModuleClass,
    ) !void {
        const endpoint = try std.fmt.allocPrint(self.allocator, "/modules/{s}", .{id});
        defer self.allocator.free(endpoint);
        return self.register(
            actor,
            id,
            kind,
            class,
            "1.0.0",
            kind,
            endpoint,
            "kernel-native",
            defaultCapsForClass(class),
        );
    }

    pub fn deregister(self: *ModuleRegistry, actor: Role, id: []const u8) !void {
        if (!hasPermission(actor, .manage_modules)) return ModuleError.AccessDenied;
        const m = self.modules.getPtr(id) orelse return ModuleError.ModuleNotFound;
        if (m.status != .stopped and m.status != .faulted and m.status != .registered)
            return ModuleError.InvalidStatusTransition;
        self.freeDescriptorStrings(m);
        _ = self.modules.remove(id);
    }

    // ── Lookup ────────────────────────────────────────────────────────────

    pub fn get(self: *const ModuleRegistry, id: []const u8) ?ModuleDescriptor {
        return self.modules.get(id);
    }

    pub fn getPtr(self: *ModuleRegistry, id: []const u8) ?*ModuleDescriptor {
        return self.modules.getPtr(id);
    }

    pub fn count(self: *const ModuleRegistry) usize {
        return self.modules.count();
    }

    pub fn activeCount(self: *const ModuleRegistry) usize {
        var n: usize = 0;
        var it = self.modules.valueIterator();
        while (it.next()) |m| {
            if (m.status == .active) n += 1;
        }
        return n;
    }

    // ── Limits ────────────────────────────────────────────────────────────

    pub fn setLimits(self: *ModuleRegistry, actor: Role, id: []const u8, limits: ResourceLimits) !void {
        if (!hasPermission(actor, .manage_modules)) return ModuleError.AccessDenied;
        const m = self.modules.getPtr(id) orelse return ModuleError.ModuleNotFound;
        m.limits = limits;
    }

    // ── Capabilities ──────────────────────────────────────────────────────

    pub fn grantCapability(self: *ModuleRegistry, actor: Role, id: []const u8, cap: Capability) !void {
        if (!hasPermission(actor, .kernel_admin)) return ModuleError.AccessDenied;
        const m = self.modules.getPtr(id) orelse return ModuleError.ModuleNotFound;
        m.capabilities.insert(cap);
    }

    pub fn revokeCapability(self: *ModuleRegistry, actor: Role, id: []const u8, cap: Capability) !void {
        if (!hasPermission(actor, .kernel_admin)) return ModuleError.AccessDenied;
        const m = self.modules.getPtr(id) orelse return ModuleError.ModuleNotFound;
        m.capabilities.remove(cap);
    }

    pub fn hasCapability(self: *const ModuleRegistry, id: []const u8, cap: Capability) bool {
        const m = self.modules.get(id) orelse return false;
        return m.capabilities.contains(cap);
    }

    // ── Dependencies ──────────────────────────────────────────────────────

    /// Declare that `id` depends on `dep_id` being active before it starts.
    pub fn addDependency(self: *ModuleRegistry, actor: Role, id: []const u8, dep_id: []const u8) !void {
        if (!hasPermission(actor, .manage_modules)) return ModuleError.AccessDenied;
        const m = self.modules.getPtr(id) orelse return ModuleError.ModuleNotFound;

        // Allocate / grow the dependency array if needed.
        if (m.dep_count == 0) {
            m.dependencies = try self.allocator.alloc([]const u8, MAX_DEPS_PER_MODULE);
        }
        if (m.dep_count >= MAX_DEPS_PER_MODULE) return ModuleError.ResourceLimitExceeded;

        const dep_copy = try self.allocator.dupe(u8, dep_id);
        m.dependencies[m.dep_count] = dep_copy;
        m.dep_count += 1;
    }

    /// Return true iff all declared dependencies of `id` are currently active.
    pub fn dependenciesMet(self: *const ModuleRegistry, id: []const u8) bool {
        const m = self.modules.get(id) orelse return false;
        for (m.dependencies[0..m.dep_count]) |dep_id| {
            const dep = self.modules.get(dep_id) orelse return false;
            if (dep.status != .active) return false;
        }
        return true;
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// 8.  Resource Allocator
// ─────────────────────────────────────────────────────────────────────────────

/// Handles scoped acquire and release of every resource dimension for a module.
/// All mutations go through here so limits are never bypassed.
pub const ResourceAllocator = struct {
    registry: *ModuleRegistry,

    pub fn init(registry: *ModuleRegistry) ResourceAllocator {
        return .{ .registry = registry };
    }

    // ── Memory ────────────────────────────────────────────────────────────

    pub fn acquireMemory(self: *ResourceAllocator, actor: Role, id: []const u8, bytes: u64) !void {
        if (!hasPermission(actor, .manage_memory)) return ModuleError.AccessDenied;
        const m = self.registry.modules.getPtr(id) orelse return ModuleError.ModuleNotFound;
        if (!m.capabilities.contains(.dynamic_memory) and m.usage.memory_bytes > 0)
            return ModuleError.CapabilityNotGranted;
        if (self.registry.used_memory + bytes > self.registry.total_memory)
            return ModuleError.GlobalMemoryExhausted;
        if (m.usage.memory_bytes + bytes > m.limits.memory_bytes)
            return ModuleError.ResourceLimitExceeded;

        m.usage.memory_bytes += bytes;
        self.registry.used_memory += bytes;
    }

    pub fn releaseMemory(self: *ResourceAllocator, actor: Role, id: []const u8, bytes: u64) !void {
        if (!hasPermission(actor, .manage_memory)) return ModuleError.AccessDenied;
        const m = self.registry.modules.getPtr(id) orelse return ModuleError.ModuleNotFound;
        const actual = @min(bytes, m.usage.memory_bytes);
        m.usage.memory_bytes -= actual;
        self.registry.used_memory = self.registry.used_memory -| actual;
    }

    /// Release all memory held by a module (used during shutdown).
    pub fn releaseAllMemory(self: *ResourceAllocator, id: []const u8) void {
        const m = self.registry.modules.getPtr(id) orelse return;
        self.registry.used_memory = self.registry.used_memory -| m.usage.memory_bytes;
        m.usage.memory_bytes = 0;
    }

    // ── Processes ─────────────────────────────────────────────────────────

    pub fn acquireProcess(self: *ResourceAllocator, actor: Role, id: []const u8) !void {
        if (!hasPermission(actor, .manage_processes)) return ModuleError.AccessDenied;
        const m = self.registry.modules.getPtr(id) orelse return ModuleError.ModuleNotFound;
        if (!m.capabilities.contains(.spawn_processes)) return ModuleError.CapabilityNotGranted;
        if (m.usage.process_count >= m.limits.max_processes) return ModuleError.ResourceLimitExceeded;
        m.usage.process_count += 1;
    }

    pub fn releaseProcess(self: *ResourceAllocator, id: []const u8) void {
        const m = self.registry.modules.getPtr(id) orelse return;
        if (m.usage.process_count > 0) m.usage.process_count -= 1;
    }

    pub fn releaseAllProcesses(self: *ResourceAllocator, id: []const u8) void {
        const m = self.registry.modules.getPtr(id) orelse return;
        m.usage.process_count = 0;
    }

    // ── Files ─────────────────────────────────────────────────────────────

    pub fn acquireFile(self: *ResourceAllocator, actor: Role, id: []const u8) !void {
        if (!hasPermission(actor, .manage_files)) return ModuleError.AccessDenied;
        const m = self.registry.modules.getPtr(id) orelse return ModuleError.ModuleNotFound;
        if (!m.capabilities.contains(.file_access)) return ModuleError.CapabilityNotGranted;
        if (m.usage.file_count >= m.limits.max_files) return ModuleError.ResourceLimitExceeded;
        m.usage.file_count += 1;
    }

    pub fn releaseFile(self: *ResourceAllocator, id: []const u8) void {
        const m = self.registry.modules.getPtr(id) orelse return;
        if (m.usage.file_count > 0) m.usage.file_count -= 1;
    }

    pub fn releaseAllFiles(self: *ResourceAllocator, id: []const u8) void {
        const m = self.registry.modules.getPtr(id) orelse return;
        m.usage.file_count = 0;
    }

    // ── Resource units (generic quota) ────────────────────────────────────

    pub fn acquireUnits(self: *ResourceAllocator, actor: Role, id: []const u8, units: u32) !void {
        if (!hasPermission(actor, .manage_modules)) return ModuleError.AccessDenied;
        const m = self.registry.modules.getPtr(id) orelse return ModuleError.ModuleNotFound;
        if (m.usage.resource_units + units > m.limits.max_resource_units)
            return ModuleError.ResourceLimitExceeded;
        m.usage.resource_units += units;
    }

    pub fn releaseUnits(self: *ResourceAllocator, id: []const u8, units: u32) void {
        const m = self.registry.modules.getPtr(id) orelse return;
        m.usage.resource_units = m.usage.resource_units -| units;
    }

    pub fn releaseAllUnits(self: *ResourceAllocator, id: []const u8) void {
        const m = self.registry.modules.getPtr(id) orelse return;
        m.usage.resource_units = 0;
    }

    // ── Network connections ───────────────────────────────────────────────

    pub fn acquireConnection(self: *ResourceAllocator, actor: Role, id: []const u8) !void {
        if (!hasPermission(actor, .manage_modules)) return ModuleError.AccessDenied;
        const m = self.registry.modules.getPtr(id) orelse return ModuleError.ModuleNotFound;
        if (!m.capabilities.contains(.network_access)) return ModuleError.CapabilityNotGranted;
        if (m.usage.network_connections >= m.limits.max_network_connections)
            return ModuleError.ResourceLimitExceeded;
        m.usage.network_connections += 1;
    }

    pub fn releaseConnection(self: *ResourceAllocator, id: []const u8) void {
        const m = self.registry.modules.getPtr(id) orelse return;
        if (m.usage.network_connections > 0) m.usage.network_connections -= 1;
    }

    pub fn releaseAllConnections(self: *ResourceAllocator, id: []const u8) void {
        const m = self.registry.modules.getPtr(id) orelse return;
        m.usage.network_connections = 0;
    }

    // ── Network traffic ───────────────────────────────────────────────────

    pub fn recordIngress(self: *ResourceAllocator, actor: Role, id: []const u8, bytes: u64) !void {
        if (!hasPermission(actor, .manage_modules)) return ModuleError.AccessDenied;
        const m = self.registry.modules.getPtr(id) orelse return ModuleError.ModuleNotFound;
        m.network.ingress_bytes += bytes;
    }

    pub fn recordEgress(self: *ResourceAllocator, actor: Role, id: []const u8, bytes: u64) !void {
        if (!hasPermission(actor, .manage_modules)) return ModuleError.AccessDenied;
        const m = self.registry.modules.getPtr(id) orelse return ModuleError.ModuleNotFound;
        m.network.egress_bytes += bytes;
    }

    // ── Bulk release (full shutdown teardown) ─────────────────────────────

    pub fn releaseAll(self: *ResourceAllocator, id: []const u8) void {
        self.releaseAllMemory(id);
        self.releaseAllProcesses(id);
        self.releaseAllFiles(id);
        self.releaseAllUnits(id);
        self.releaseAllConnections(id);
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// 9.  Lifecycle Manager
// ─────────────────────────────────────────────────────────────────────────────

/// Drives status transitions, dependency checks, fault recording, and event
/// emission.  Works with a shared EventBus so lifecycle events are observable.
pub const LifecycleManager = struct {
    registry: *ModuleRegistry,
    resources: *ResourceAllocator,
    bus: *EventBus,

    pub fn init(
        registry: *ModuleRegistry,
        resources: *ResourceAllocator,
        bus: *EventBus,
    ) LifecycleManager {
        return .{ .registry = registry, .resources = resources, .bus = bus };
    }

    // ── Transition helpers ────────────────────────────────────────────────

    fn transition(self: *LifecycleManager, id: []const u8, next: ModuleStatus) !void {
        const m = self.registry.modules.getPtr(id) orelse return ModuleError.ModuleNotFound;
        if (!m.status.canTransitionTo(next)) return ModuleError.InvalidStatusTransition;
        m.status = next;
        m.last_status_ms = std.time.milliTimestamp();
    }

    // ── Start / provision ─────────────────────────────────────────────────

    /// Move a registered module through provisioning → active.
    /// Checks dependency satisfaction first.
    pub fn start(self: *LifecycleManager, actor: Role, id: []const u8) !void {
        if (!hasPermission(actor, .manage_modules)) return ModuleError.AccessDenied;
        if (!self.registry.dependenciesMet(id)) return ModuleError.DependencyNotMet;

        try self.transition(id, .provisioning);
        try self.transition(id, .active);

        const m = self.registry.modules.getPtr(id).?;
        m.start_count += 1;

        try self.bus.publish(id, "module.started", id);
    }

    // ── Suspend / resume ──────────────────────────────────────────────────

    pub fn suspend_(self: *LifecycleManager, actor: Role, id: []const u8) !void {
        if (!hasPermission(actor, .manage_modules)) return ModuleError.AccessDenied;
        try self.transition(id, .suspended);
        try self.bus.publish(id, "module.suspended", id);
    }

    pub fn resume_(self: *LifecycleManager, actor: Role, id: []const u8) !void {
        if (!hasPermission(actor, .manage_modules)) return ModuleError.AccessDenied;
        try self.transition(id, .active);
        try self.bus.publish(id, "module.resumed", id);
    }

    // ── Drain / stop ──────────────────────────────────────────────────────

    /// Begin graceful shutdown: transitions active → draining.
    pub fn drain(self: *LifecycleManager, actor: Role, id: []const u8) !void {
        if (!hasPermission(actor, .manage_modules)) return ModuleError.AccessDenied;
        try self.transition(id, .draining);
        try self.bus.publish(id, "module.draining", id);
    }

    /// Complete shutdown: releases all resources and transitions to stopped.
    pub fn stop(self: *LifecycleManager, actor: Role, id: []const u8) !void {
        if (!hasPermission(actor, .manage_modules)) return ModuleError.AccessDenied;
        // Allow stopping directly from active (force-stop) or from draining.
        const m = self.registry.modules.getPtr(id) orelse return ModuleError.ModuleNotFound;
        if (m.status == .active) try self.transition(id, .draining);
        try self.transition(id, .stopped);
        self.resources.releaseAll(id);
        try self.bus.publish(id, "module.stopped", id);
    }

    // ── Fault ─────────────────────────────────────────────────────────────

    pub fn fault(self: *LifecycleManager, id: []const u8, reason: []const u8) !void {
        const m = self.registry.modules.getPtr(id) orelse return ModuleError.ModuleNotFound;
        m.fault_count += 1;
        // Best-effort transition; might already be in an invalid state.
        self.transition(id, .faulted) catch {};
        try self.bus.publish(id, "module.faulted", reason);
    }

    // ── Restart ───────────────────────────────────────────────────────────

    /// Fully stop then re-start a module.
    pub fn restart(self: *LifecycleManager, actor: Role, id: []const u8) !void {
        if (!hasPermission(actor, .manage_modules)) return ModuleError.AccessDenied;
        try self.stop(actor, id);
        // stopped → provisioning is a legal transition.
        try self.start(actor, id);
        try self.bus.publish(id, "module.restarted", id);
    }

    // ── Remove ────────────────────────────────────────────────────────────

    pub fn remove(self: *LifecycleManager, actor: Role, id: []const u8) !void {
        if (!hasPermission(actor, .manage_modules)) return ModuleError.AccessDenied;
        const m = self.registry.modules.getPtr(id) orelse return ModuleError.ModuleNotFound;
        if (m.status == .active or m.status == .provisioning)
            try self.stop(actor, id);
        try self.transition(id, .deregistering);
        try self.bus.publish(id, "module.removed", id);
        try self.registry.deregister(actor, id);
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// 10.  Provision Plan  (declarative multi-resource bootstrap)
// ─────────────────────────────────────────────────────────────────────────────

/// A single step in a ProvisionPlan.
pub const ProvisionStep = union(enum) {
    /// Allocate `bytes` of memory.
    memory: u64,
    /// Reserve `count` process slots.
    processes: u32,
    /// Reserve `count` file slots.
    files: u32,
    /// Reserve `units` generic resource units.
    units: u32,
    /// Reserve `count` network connection slots.
    connections: u32,
    /// Grant a capability.
    capability: Capability,
    /// Record initial network traffic counters.
    network: struct { ingress: u64, egress: u64 },
};

/// Declarative set of provisioning steps executed atomically on `start`.
/// Mirrors bootstrapOfficeModule / bootstrapCorePlatformComponents from
/// kernel.zig but in a data-driven, reusable form.
pub const ProvisionPlan = struct {
    module_id: []const u8, // borrowed — must outlive the plan
    steps: []const ProvisionStep,

    /// Execute all steps against `alloc` on behalf of `actor`.
    /// If any step fails, already-applied steps are not rolled back (the caller
    /// should fault the module via LifecycleManager).
    pub fn execute(
        self: *const ProvisionPlan,
        actor: Role,
        alloc: *ResourceAllocator,
    ) !void {
        for (self.steps) |step| {
            switch (step) {
                .memory => |b| try alloc.acquireMemory(actor, self.module_id, b),
                .processes => |n| {
                    var i: u32 = 0;
                    while (i < n) : (i += 1) try alloc.acquireProcess(actor, self.module_id);
                },
                .files => |n| {
                    var i: u32 = 0;
                    while (i < n) : (i += 1) try alloc.acquireFile(actor, self.module_id);
                },
                .units => |u| try alloc.acquireUnits(actor, self.module_id, u),
                .connections => |n| {
                    var i: u32 = 0;
                    while (i < n) : (i += 1) try alloc.acquireConnection(actor, self.module_id);
                },
                .capability => |cap| try alloc.registry.grantCapability(actor, self.module_id, cap),
                .network => |net| {
                    try alloc.recordIngress(actor, self.module_id, net.ingress);
                    try alloc.recordEgress(actor, self.module_id, net.egress);
                },
            }
        }
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// 11.  Module System  (top-level coordinator)
// ─────────────────────────────────────────────────────────────────────────────

pub const ModuleSystem = struct {
    allocator: std.mem.Allocator,
    registry: ModuleRegistry,
    resources: ResourceAllocator,
    lifecycle: LifecycleManager,
    bus: EventBus,

    pub fn init(allocator: std.mem.Allocator, total_memory: u64, event_capacity: usize) ModuleSystem {
        var reg = ModuleRegistry.init(allocator, total_memory);
        var res = ResourceAllocator.init(&reg);
        var bus = EventBus.init(allocator, event_capacity);
        const lc = LifecycleManager.init(&reg, &res, &bus);
        return .{
            .allocator = allocator,
            .registry = reg,
            .resources = res,
            .lifecycle = lc,
            .bus = bus,
        };
    }

    /// Re-point internal cross-references after the struct is moved.
    /// Call this once immediately after init if you store ModuleSystem by value.
    pub fn relink(self: *ModuleSystem) void {
        self.resources = ResourceAllocator.init(&self.registry);
        self.lifecycle = LifecycleManager.init(&self.registry, &self.resources, &self.bus);
    }

    pub fn deinit(self: *ModuleSystem) void {
        self.registry.deinit();
        self.bus.deinit();
    }

    // ── Convenience one-liners ────────────────────────────────────────────

    pub fn registerAndStart(
        self: *ModuleSystem,
        actor: Role,
        id: []const u8,
        kind: []const u8,
        class: ModuleClass,
    ) !void {
        try self.registry.registerSimple(actor, id, kind, class);
        try self.lifecycle.start(actor, id);
    }

    /// Execute a ProvisionPlan then start the module.
    pub fn provision(
        self: *ModuleSystem,
        actor: Role,
        plan: *const ProvisionPlan,
    ) !void {
        const id = plan.module_id;
        const m = self.registry.modules.getPtr(id) orelse return ModuleError.ModuleNotFound;
        if (!self.registry.dependenciesMet(id)) return ModuleError.DependencyNotMet;

        try self.lifecycle.transition(id, .provisioning);
        plan.execute(actor, &self.resources) catch |err| {
            try self.lifecycle.fault(id, "provision-failed");
            return err;
        };
        try self.lifecycle.transition(id, .active);
        m.start_count += 1;
        try self.bus.publish(id, "module.provisioned", id);
    }

    // ── Stats ─────────────────────────────────────────────────────────────

    pub const Stats = struct {
        total_modules: usize,
        active_modules: usize,
        faulted_modules: usize,
        total_memory: u64,
        used_memory: u64,
        event_count: usize,
    };

    pub fn stats(self: *const ModuleSystem) Stats {
        var faulted: usize = 0;
        var it = self.registry.modules.valueIterator();
        while (it.next()) |m| {
            if (m.status == .faulted) faulted += 1;
        }
        return .{
            .total_modules = self.registry.count(),
            .active_modules = self.registry.activeCount(),
            .faulted_modules = faulted,
            .total_memory = self.registry.total_memory,
            .used_memory = self.registry.used_memory,
            .event_count = self.bus.len(),
        };
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Default capability set granted to each module class.
pub fn defaultCapsForClass(class: ModuleClass) CapabilitySet {
    return switch (class) {
        .kernel => CapabilitySet.initFull(),
        .host => CapabilitySet.initMany(&.{
            .kernel_api,          .spawn_processes, .network_access, .event_bus,
            .register_submodules, .file_access,     .dynamic_memory, .scheduling,
        }),
        .server, .engine, .services => CapabilitySet.initMany(&.{
            .spawn_processes, .network_access, .event_bus,
            .file_access,     .dynamic_memory, .scheduling,
        }),
        .module => CapabilitySet.initMany(&.{
            .spawn_processes, .network_access, .event_bus,
            .file_access,     .dynamic_memory, .scheduling,
        }),
        .daemon => CapabilitySet.initMany(&.{
            .spawn_processes, .event_bus, .file_access, .scheduling,
        }),
        .plugin => CapabilitySet.initMany(&.{
            .event_bus, .file_access,
        }),
    };
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

test "module registration and deregistration" {
    var sys = ModuleSystem.init(std.testing.allocator, 4 * 1024 * 1024 * 1024, 256);
    sys.relink();
    defer sys.deinit();

    try sys.registry.registerSimple(.host, "kogi.exchange", "exchange", .module);
    try std.testing.expectEqual(@as(usize, 1), sys.registry.count());
    try std.testing.expectEqual(ModuleStatus.registered, sys.registry.get("kogi.exchange").?.status);

    // Duplicate registration must fail.
    try std.testing.expectError(
        ModuleError.ModuleAlreadyRegistered,
        sys.registry.registerSimple(.host, "kogi.exchange", "exchange", .module),
    );

    // Cannot deregister an active module.
    try sys.lifecycle.start(.host, "kogi.exchange");
    try std.testing.expectError(
        ModuleError.InvalidStatusTransition,
        sys.registry.deregister(.host, "kogi.exchange"),
    );

    try sys.lifecycle.stop(.host, "kogi.exchange");
    try sys.registry.deregister(.host, "kogi.exchange");
    try std.testing.expectEqual(@as(usize, 0), sys.registry.count());
}

test "rbac: unprivileged role denied" {
    var sys = ModuleSystem.init(std.testing.allocator, 1024 * 1024 * 1024, 64);
    sys.relink();
    defer sys.deinit();

    try std.testing.expectError(
        ModuleError.AccessDenied,
        sys.registry.registerSimple(.user, "evil.mod", "evil", .module),
    );
}

test "status state machine: legal and illegal transitions" {
    var sys = ModuleSystem.init(std.testing.allocator, 1024 * 1024 * 1024, 64);
    sys.relink();
    defer sys.deinit();

    try sys.registry.registerSimple(.host, "mod-a", "test", .module);

    // registered → active (via start)
    try sys.lifecycle.start(.host, "mod-a");
    try std.testing.expectEqual(ModuleStatus.active, sys.registry.get("mod-a").?.status);

    // active → suspended
    try sys.lifecycle.suspend_(.host, "mod-a");
    try std.testing.expectEqual(ModuleStatus.suspended, sys.registry.get("mod-a").?.status);

    // suspended → active (resume)
    try sys.lifecycle.resume_(.host, "mod-a");

    // active → draining → stopped
    try sys.lifecycle.drain(.host, "mod-a");
    try sys.lifecycle.stop(.host, "mod-a");
    try std.testing.expectEqual(ModuleStatus.stopped, sys.registry.get("mod-a").?.status);

    // stopped → active (restart path — provisioning → active)
    try sys.lifecycle.start(.host, "mod-a");
    try std.testing.expectEqual(@as(u32, 2), sys.registry.get("mod-a").?.start_count);
}

test "resource limits enforced across all dimensions" {
    var sys = ModuleSystem.init(std.testing.allocator, 1024 * 1024 * 1024, 64);
    sys.relink();
    defer sys.deinit();

    try sys.registry.registerSimple(.host, "limited", "test", .module);
    try sys.registry.setLimits(.host, "limited", .{
        .memory_bytes = 1000,
        .max_processes = 2,
        .max_files = 1,
        .max_resource_units = 10,
        .max_network_connections = 1,
    });
    try sys.lifecycle.start(.host, "limited");

    // Memory
    try sys.resources.acquireMemory(.host, "limited", 800);
    try std.testing.expectError(
        ModuleError.ResourceLimitExceeded,
        sys.resources.acquireMemory(.host, "limited", 300),
    );

    // Processes
    try sys.resources.acquireProcess(.host, "limited");
    try sys.resources.acquireProcess(.host, "limited");
    try std.testing.expectError(
        ModuleError.ResourceLimitExceeded,
        sys.resources.acquireProcess(.host, "limited"),
    );

    // Files
    try sys.resources.acquireFile(.host, "limited");
    try std.testing.expectError(
        ModuleError.ResourceLimitExceeded,
        sys.resources.acquireFile(.host, "limited"),
    );

    // Units
    try sys.resources.acquireUnits(.host, "limited", 8);
    try std.testing.expectError(
        ModuleError.ResourceLimitExceeded,
        sys.resources.acquireUnits(.host, "limited", 5),
    );

    // Network connections
    try sys.resources.acquireConnection(.host, "limited");
    try std.testing.expectError(
        ModuleError.ResourceLimitExceeded,
        sys.resources.acquireConnection(.host, "limited"),
    );
}

test "global memory pool limit respected" {
    // Total pool = 500 bytes.
    var sys = ModuleSystem.init(std.testing.allocator, 500, 32);
    sys.relink();
    defer sys.deinit();

    try sys.registry.registerSimple(.host, "mod-a", "test", .module);
    try sys.registry.setLimits(.host, "mod-a", .{
        .memory_bytes = 1000,
        .max_processes = 4,
        .max_files = 10,
        .max_resource_units = 100,
        .max_network_connections = 4,
    });
    try sys.lifecycle.start(.host, "mod-a");

    try sys.resources.acquireMemory(.host, "mod-a", 400);
    try std.testing.expectError(
        ModuleError.GlobalMemoryExhausted,
        sys.resources.acquireMemory(.host, "mod-a", 200),
    );
}

test "capability gating" {
    var sys = ModuleSystem.init(std.testing.allocator, 1024 * 1024 * 1024, 64);
    sys.relink();
    defer sys.deinit();

    // Plugin has very limited default caps (no spawn_processes, no network_access).
    try sys.registry.registerSimple(.host, "plug-a", "plugin", .plugin);
    try sys.lifecycle.start(.host, "plug-a");

    try std.testing.expectError(
        ModuleError.CapabilityNotGranted,
        sys.resources.acquireProcess(.host, "plug-a"),
    );
    try std.testing.expectError(
        ModuleError.CapabilityNotGranted,
        sys.resources.acquireConnection(.host, "plug-a"),
    );

    // Grant network_access and try again.
    try sys.registry.grantCapability(.root, "plug-a", .network_access);
    try sys.resources.acquireConnection(.host, "plug-a");

    // Revoke and confirm it's blocked again.
    try sys.registry.revokeCapability(.root, "plug-a", .network_access);
    sys.resources.releaseAllConnections("plug-a");
    try std.testing.expectError(
        ModuleError.CapabilityNotGranted,
        sys.resources.acquireConnection(.host, "plug-a"),
    );
}

test "dependency satisfaction blocks start" {
    var sys = ModuleSystem.init(std.testing.allocator, 1024 * 1024 * 1024, 128);
    sys.relink();
    defer sys.deinit();

    try sys.registry.registerSimple(.host, "core", "core", .services);
    try sys.registry.registerSimple(.host, "worker", "worker", .module);
    try sys.registry.addDependency(.host, "worker", "core");

    // core not yet active → worker cannot start.
    try std.testing.expectError(
        ModuleError.DependencyNotMet,
        sys.lifecycle.start(.host, "worker"),
    );

    // Start core first.
    try sys.lifecycle.start(.host, "core");
    // Now worker can start.
    try sys.lifecycle.start(.host, "worker");
    try std.testing.expectEqual(ModuleStatus.active, sys.registry.get("worker").?.status);
}

test "provision plan: declarative multi-resource setup" {
    var sys = ModuleSystem.init(std.testing.allocator, 4 * 1024 * 1024 * 1024, 128);
    sys.relink();
    defer sys.deinit();

    try sys.registry.register(
        .host,
        "kogi.office",
        "office",
        .module,
        "2.0.0",
        "Office module",
        "/services/office",
        "kogi-go-network",
        defaultCapsForClass(.module),
    );
    try sys.registry.setLimits(.host, "kogi.office", .{
        .memory_bytes = 768 * 1024 * 1024,
        .max_processes = 96,
        .max_files = 6_000,
        .max_resource_units = 14_000,
        .max_network_connections = 64,
    });

    const steps = [_]ProvisionStep{
        .{ .memory = 64 * 1024 * 1024 },
        .{ .processes = 2 },
        .{ .files = 2 },
        .{ .units = 320 },
        .{ .network = .{ .ingress = 4096, .egress = 8192 } },
    };
    const plan = ProvisionPlan{ .module_id = "kogi.office", .steps = &steps };

    try sys.provision(.host, &plan);

    const m = sys.registry.get("kogi.office").?;
    try std.testing.expectEqual(ModuleStatus.active, m.status);
    try std.testing.expectEqual(@as(u64, 64 * 1024 * 1024), m.usage.memory_bytes);
    try std.testing.expectEqual(@as(u32, 2), m.usage.process_count);
    try std.testing.expectEqual(@as(u32, 2), m.usage.file_count);
    try std.testing.expectEqual(@as(u32, 320), m.usage.resource_units);
    try std.testing.expectEqual(@as(u64, 4096), m.network.ingress_bytes);
    try std.testing.expectEqual(@as(u64, 8192), m.network.egress_bytes);
}

test "lifecycle events are published to event bus" {
    var sys = ModuleSystem.init(std.testing.allocator, 1024 * 1024 * 1024, 64);
    sys.relink();
    defer sys.deinit();

    try sys.registry.registerSimple(.host, "svc", "svc", .services);
    try sys.lifecycle.start(.host, "svc");
    try sys.lifecycle.suspend_(.host, "svc");
    try sys.lifecycle.resume_(.host, "svc");
    try sys.lifecycle.stop(.host, "svc");

    // 4 events: started, suspended, resumed, stopped.
    try std.testing.expect(sys.bus.len() >= 4);

    var started = std.ArrayList(ModuleEvent).init(std.testing.allocator);
    defer started.deinit();
    try sys.bus.collect("module.started", &started);
    try std.testing.expectEqual(@as(usize, 1), started.items.len);
}

test "fault increments counter and transitions to faulted" {
    var sys = ModuleSystem.init(std.testing.allocator, 1024 * 1024 * 1024, 64);
    sys.relink();
    defer sys.deinit();

    try sys.registry.registerSimple(.host, "flaky", "flaky", .daemon);
    try sys.lifecycle.start(.host, "flaky");
    try sys.lifecycle.fault("flaky", "oom");

    const m = sys.registry.get("flaky").?;
    try std.testing.expectEqual(ModuleStatus.faulted, m.status);
    try std.testing.expectEqual(@as(u32, 1), m.fault_count);
}

test "restart: stop then re-provision increments start_count" {
    var sys = ModuleSystem.init(std.testing.allocator, 1024 * 1024 * 1024, 128);
    sys.relink();
    defer sys.deinit();

    try sys.registry.registerSimple(.host, "restartable", "svc", .server);
    try sys.lifecycle.start(.host, "restartable");
    try sys.lifecycle.restart(.host, "restartable");

    try std.testing.expectEqual(@as(u32, 2), sys.registry.get("restartable").?.start_count);
    try std.testing.expectEqual(ModuleStatus.active, sys.registry.get("restartable").?.status);
}

test "remove: stops and deregisters cleanly" {
    var sys = ModuleSystem.init(std.testing.allocator, 1024 * 1024 * 1024, 128);
    sys.relink();
    defer sys.deinit();

    try sys.registry.registerSimple(.host, "tmp", "tmp", .plugin);
    try sys.lifecycle.start(.host, "tmp");
    try sys.lifecycle.remove(.host, "tmp");

    try std.testing.expectEqual(@as(usize, 0), sys.registry.count());
}

test "full stats coverage" {
    var sys = ModuleSystem.init(std.testing.allocator, 1024 * 1024 * 1024, 128);
    sys.relink();
    defer sys.deinit();

    try sys.registerAndStart(.host, "a", "a", .module);
    try sys.registerAndStart(.host, "b", "b", .daemon);
    try sys.lifecycle.fault("b", "crash");

    const s = sys.stats();
    try std.testing.expectEqual(@as(usize, 2), s.total_modules);
    try std.testing.expectEqual(@as(usize, 1), s.active_modules);
    try std.testing.expectEqual(@as(usize, 1), s.faulted_modules);
}
