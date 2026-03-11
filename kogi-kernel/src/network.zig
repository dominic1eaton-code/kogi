//! network.zig — Kernel Network Management System
//!
//! Kernel extension extracted and greatly expanded from kernel.zig.
//!
//! Provides:
//!   AddressTable        — IP / endpoint address registry and resolution
//!   InterfaceRegistry   — virtual network interface management (up/down/stats)
//!   SocketTable         — kernel-level socket lifecycle (open/bind/connect/close)
//!   ConnectionPool      — reusable connection slots with health-check support
//!   TrafficLedger       — per-component ingress/egress accounting + quota enforcement
//!   PacketQueue         — bounded in-kernel packet buffer with priority lanes
//!   RouteTable          — static routing table (longest-prefix match)
//!   FirewallRuleSet     — ordered allow/deny rules evaluated per packet
//!   NetworkManager      — top-level coordinator wiring all subsystems together
//!
//! All timestamps are i64 milliseconds (consistent with kernel.zig / memory.zig /
//! processes.zig).  No OS sockets are created — this is a purely logical,
//! single-threaded kernel-space model.

const std = @import("std");

// ─────────────────────────────────────────────────────────────────────────────
// Shared kernel types  (mirror kernel.zig so the file is self-contained)
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

pub const NetworkError = error{
    AccessDenied,
    AddressNotFound,
    AddressAlreadyRegistered,
    InterfaceNotFound,
    InterfaceAlreadyExists,
    InterfaceDown,
    SocketNotFound,
    SocketAlreadyBound,
    SocketExhausted,
    ConnectionNotFound,
    ConnectionPoolExhausted,
    QuotaExceeded,
    PacketQueueFull,
    RouteNotFound,
    RouteAlreadyExists,
    FirewallDenied,
    InvalidAddress,
    InvalidPort,
    ManagerNotFound,
};

// ─────────────────────────────────────────────────────────────────────────────
// Primitive types
// ─────────────────────────────────────────────────────────────────────────────

/// A compact IPv4 address stored as four octets.
pub const Ipv4 = struct {
    octets: [4]u8,

    pub fn init(a: u8, b: u8, c: u8, d: u8) Ipv4 {
        return .{ .octets = .{ a, b, c, d } };
    }

    /// Parse "a.b.c.d" notation.
    pub fn parse(s: []const u8) !Ipv4 {
        var it = std.mem.splitScalar(u8, s, '.');
        var octets: [4]u8 = undefined;
        for (&octets) |*o| {
            const part = it.next() orelse return NetworkError.InvalidAddress;
            o.* = std.fmt.parseInt(u8, part, 10) catch return NetworkError.InvalidAddress;
        }
        if (it.next() != null) return NetworkError.InvalidAddress;
        return .{ .octets = octets };
    }

    pub fn eql(self: Ipv4, other: Ipv4) bool {
        return std.mem.eql(u8, &self.octets, &other.octets);
    }

    /// Format as "a.b.c.d" into `buf`.  Returns the written slice.
    pub fn format(self: Ipv4, buf: []u8) ![]u8 {
        return std.fmt.bufPrint(buf, "{d}.{d}.{d}.{d}", .{
            self.octets[0], self.octets[1], self.octets[2], self.octets[3],
        });
    }

    /// Mask `self` with a /`prefix_len` mask.
    pub fn masked(self: Ipv4, prefix_len: u6) Ipv4 {
        if (prefix_len == 0) return Ipv4.init(0, 0, 0, 0);
        const shift: u5 = @intCast(32 - @as(u7, prefix_len));
        const raw: u32 = (@as(u32, self.octets[0]) << 24) |
            (@as(u32, self.octets[1]) << 16) |
            (@as(u32, self.octets[2]) << 8) |
            @as(u32, self.octets[3]);
        const mask: u32 = if (prefix_len == 32) 0xFFFF_FFFF else ~((@as(u32, 1) << shift) - 1);
        const result = raw & mask;
        return Ipv4.init(
            @intCast((result >> 24) & 0xFF),
            @intCast((result >> 16) & 0xFF),
            @intCast((result >> 8) & 0xFF),
            @intCast(result & 0xFF),
        );
    }
};

/// A network endpoint: IP address + port.
pub const Endpoint = struct {
    addr: Ipv4,
    port: u16,

    pub fn init(addr: Ipv4, port: u16) Endpoint {
        return .{ .addr = addr, .port = port };
    }

    pub fn eql(self: Endpoint, other: Endpoint) bool {
        return self.addr.eql(other.addr) and self.port == other.port;
    }
};

/// Transport-layer protocol.
pub const Protocol = enum { tcp, udp, icmp, raw };

// ─────────────────────────────────────────────────────────────────────────────
// 1.  Address Table
// ─────────────────────────────────────────────────────────────────────────────

/// A named address record mapping a logical name to an Ipv4 endpoint.
pub const AddressRecord = struct {
    name: []const u8, // owned by AddressTable
    addr: Ipv4,
    port: u16,
    component_id: []const u8, // owned by AddressTable
    scheme: []const u8, // e.g. "http", "local", "grpc" — owned
    registered_ms: i64,
};

/// Kernel address registry — logical name → endpoint resolution.
pub const AddressTable = struct {
    allocator: std.mem.Allocator,
    records: std.StringHashMap(AddressRecord),

    pub fn init(allocator: std.mem.Allocator) AddressTable {
        return .{ .allocator = allocator, .records = std.StringHashMap(AddressRecord).init(allocator) };
    }

    pub fn deinit(self: *AddressTable) void {
        var it = self.records.valueIterator();
        while (it.next()) |r| {
            self.allocator.free(r.name);
            self.allocator.free(r.component_id);
            self.allocator.free(r.scheme);
        }
        self.records.deinit();
    }

    /// Register a name → endpoint mapping.
    pub fn register(
        self: *AddressTable,
        actor: Role,
        name: []const u8,
        addr: Ipv4,
        port: u16,
        component_id: []const u8,
        scheme: []const u8,
    ) !void {
        if (!hasPermission(actor, .manage_modules)) return NetworkError.AccessDenied;
        if (self.records.contains(name)) return NetworkError.AddressAlreadyRegistered;

        const name_copy = try self.allocator.dupe(u8, name);
        errdefer self.allocator.free(name_copy);
        const comp_copy = try self.allocator.dupe(u8, component_id);
        errdefer self.allocator.free(comp_copy);
        const scheme_copy = try self.allocator.dupe(u8, scheme);
        errdefer self.allocator.free(scheme_copy);

        try self.records.put(name_copy, .{
            .name = name_copy,
            .addr = addr,
            .port = port,
            .component_id = comp_copy,
            .scheme = scheme_copy,
            .registered_ms = std.time.milliTimestamp(),
        });
    }

    /// Resolve a name to its AddressRecord.
    pub fn resolve(self: *const AddressTable, name: []const u8) ?AddressRecord {
        return self.records.get(name);
    }

    /// Deregister a name.
    pub fn deregister(self: *AddressTable, actor: Role, name: []const u8) !void {
        if (!hasPermission(actor, .manage_modules)) return NetworkError.AccessDenied;
        const r = self.records.get(name) orelse return NetworkError.AddressNotFound;
        self.allocator.free(r.name);
        self.allocator.free(r.component_id);
        self.allocator.free(r.scheme);
        _ = self.records.remove(name);
    }

    pub fn count(self: *const AddressTable) usize {
        return self.records.count();
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// 2.  Interface Registry
// ─────────────────────────────────────────────────────────────────────────────

pub const InterfaceState = enum { up, down, degraded };

/// A virtual network interface.
pub const Interface = struct {
    name: []const u8, // owned by InterfaceRegistry
    addr: Ipv4,
    prefix_len: u6, // subnet mask length (0–32)
    mtu: u32,
    state: InterfaceState,
    ingress_bytes: u64,
    egress_bytes: u64,
    ingress_packets: u64,
    egress_packets: u64,
    drop_count: u64,
    created_ms: i64,
    last_active_ms: i64,
};

pub const InterfaceRegistry = struct {
    allocator: std.mem.Allocator,
    interfaces: std.StringHashMap(Interface),

    pub fn init(allocator: std.mem.Allocator) InterfaceRegistry {
        return .{ .allocator = allocator, .interfaces = std.StringHashMap(Interface).init(allocator) };
    }

    pub fn deinit(self: *InterfaceRegistry) void {
        var it = self.interfaces.valueIterator();
        while (it.next()) |iface| self.allocator.free(iface.name);
        self.interfaces.deinit();
    }

    /// Register a new virtual interface.
    pub fn create(
        self: *InterfaceRegistry,
        actor: Role,
        name: []const u8,
        addr: Ipv4,
        prefix_len: u6,
        mtu: u32,
    ) !void {
        if (!hasPermission(actor, .manage_modules)) return NetworkError.AccessDenied;
        if (self.interfaces.contains(name)) return NetworkError.InterfaceAlreadyExists;

        const name_copy = try self.allocator.dupe(u8, name);
        errdefer self.allocator.free(name_copy);
        const now = std.time.milliTimestamp();

        try self.interfaces.put(name_copy, .{
            .name = name_copy,
            .addr = addr,
            .prefix_len = prefix_len,
            .mtu = mtu,
            .state = .down,
            .ingress_bytes = 0,
            .egress_bytes = 0,
            .ingress_packets = 0,
            .egress_packets = 0,
            .drop_count = 0,
            .created_ms = now,
            .last_active_ms = now,
        });
    }

    pub fn setState(self: *InterfaceRegistry, actor: Role, name: []const u8, state: InterfaceState) !void {
        if (!hasPermission(actor, .manage_modules)) return NetworkError.AccessDenied;
        const iface = self.interfaces.getPtr(name) orelse return NetworkError.InterfaceNotFound;
        iface.state = state;
        iface.last_active_ms = std.time.milliTimestamp();
    }

    /// Record traffic through an interface.
    pub fn recordTraffic(
        self: *InterfaceRegistry,
        name: []const u8,
        ingress_bytes: u64,
        egress_bytes: u64,
    ) !void {
        const iface = self.interfaces.getPtr(name) orelse return NetworkError.InterfaceNotFound;
        if (iface.state == .down) return NetworkError.InterfaceDown;

        iface.ingress_bytes += ingress_bytes;
        iface.egress_bytes += egress_bytes;
        iface.ingress_packets += if (ingress_bytes > 0) 1 else 0;
        iface.egress_packets += if (egress_bytes > 0) 1 else 0;
        iface.last_active_ms = std.time.milliTimestamp();
    }

    pub fn recordDrop(self: *InterfaceRegistry, name: []const u8) !void {
        const iface = self.interfaces.getPtr(name) orelse return NetworkError.InterfaceNotFound;
        iface.drop_count += 1;
    }

    pub fn get(self: *const InterfaceRegistry, name: []const u8) ?Interface {
        return self.interfaces.get(name);
    }

    pub fn count(self: *const InterfaceRegistry) usize {
        return self.interfaces.count();
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// 3.  Socket Table
// ─────────────────────────────────────────────────────────────────────────────

pub const SocketState = enum { unbound, bound, listening, connected, closing, closed };

pub const Socket = struct {
    fd: u64, // kernel file-descriptor handle
    protocol: Protocol,
    local: ?Endpoint,
    remote: ?Endpoint,
    state: SocketState,
    owner_pid: ?u64,
    iface_name: ?[]const u8, // owned by SocketTable (nullable)
    bytes_sent: u64,
    bytes_recv: u64,
    created_ms: i64,
    last_event_ms: i64,
};

pub const SocketTable = struct {
    allocator: std.mem.Allocator,
    sockets: std.AutoHashMap(u64, Socket),
    iface_names: std.AutoHashMap(u64, []u8), // fd → owned iface name copy
    next_fd: u64,
    max_sockets: usize,

    pub fn init(allocator: std.mem.Allocator, max_sockets: usize) SocketTable {
        return .{
            .allocator = allocator,
            .sockets = std.AutoHashMap(u64, Socket).init(allocator),
            .iface_names = std.AutoHashMap(u64, []u8).init(allocator),
            .next_fd = 100,
            .max_sockets = max_sockets,
        };
    }

    pub fn deinit(self: *SocketTable) void {
        var it = self.iface_names.valueIterator();
        while (it.next()) |name| self.allocator.free(name.*);
        self.iface_names.deinit();
        self.sockets.deinit();
    }

    /// Open a new socket and return its fd.
    pub fn open(self: *SocketTable, actor: Role, protocol: Protocol, owner_pid: ?u64) !u64 {
        if (!hasPermission(actor, .manage_processes)) return NetworkError.AccessDenied;
        if (self.sockets.count() >= self.max_sockets) return NetworkError.SocketExhausted;

        const fd = self.next_fd;
        self.next_fd += 1;
        const now = std.time.milliTimestamp();

        try self.sockets.put(fd, .{
            .fd = fd,
            .protocol = protocol,
            .local = null,
            .remote = null,
            .state = .unbound,
            .owner_pid = owner_pid,
            .iface_name = null,
            .bytes_sent = 0,
            .bytes_recv = 0,
            .created_ms = now,
            .last_event_ms = now,
        });
        return fd;
    }

    /// Bind socket `fd` to a local endpoint.
    pub fn bind(self: *SocketTable, actor: Role, fd: u64, local: Endpoint, iface_name: ?[]const u8) !void {
        if (!hasPermission(actor, .manage_processes)) return NetworkError.AccessDenied;
        const s = self.sockets.getPtr(fd) orelse return NetworkError.SocketNotFound;
        if (s.state != .unbound) return NetworkError.SocketAlreadyBound;

        s.local = local;
        s.state = .bound;
        s.last_event_ms = std.time.milliTimestamp();

        if (iface_name) |n| {
            const n_copy = try self.allocator.dupe(u8, n);
            try self.iface_names.put(fd, n_copy);
            s.iface_name = n_copy;
        }
    }

    /// Connect socket `fd` to a remote endpoint.
    pub fn connect(self: *SocketTable, actor: Role, fd: u64, remote: Endpoint) !void {
        if (!hasPermission(actor, .manage_processes)) return NetworkError.AccessDenied;
        const s = self.sockets.getPtr(fd) orelse return NetworkError.SocketNotFound;
        s.remote = remote;
        s.state = .connected;
        s.last_event_ms = std.time.milliTimestamp();
    }

    /// Mark socket as listening (TCP servers).
    pub fn listen(self: *SocketTable, actor: Role, fd: u64) !void {
        if (!hasPermission(actor, .manage_processes)) return NetworkError.AccessDenied;
        const s = self.sockets.getPtr(fd) orelse return NetworkError.SocketNotFound;
        if (s.state != .bound) return NetworkError.SocketAlreadyBound;
        s.state = .listening;
        s.last_event_ms = std.time.milliTimestamp();
    }

    /// Record bytes transferred on socket `fd`.
    pub fn recordIO(self: *SocketTable, fd: u64, sent: u64, recv: u64) !void {
        const s = self.sockets.getPtr(fd) orelse return NetworkError.SocketNotFound;
        s.bytes_sent += sent;
        s.bytes_recv += recv;
        s.last_event_ms = std.time.milliTimestamp();
    }

    /// Close socket `fd` and free associated resources.
    pub fn close(self: *SocketTable, actor: Role, fd: u64) !void {
        if (!hasPermission(actor, .manage_processes)) return NetworkError.AccessDenied;
        if (!self.sockets.contains(fd)) return NetworkError.SocketNotFound;

        if (self.iface_names.get(fd)) |name| {
            self.allocator.free(name);
            _ = self.iface_names.remove(fd);
        }
        _ = self.sockets.remove(fd);
    }

    pub fn get(self: *const SocketTable, fd: u64) ?Socket {
        return self.sockets.get(fd);
    }

    pub fn openCount(self: *const SocketTable) usize {
        return self.sockets.count();
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// 4.  Connection Pool
// ─────────────────────────────────────────────────────────────────────────────

pub const ConnectionState = enum { idle, active, unhealthy, draining, closed };

pub const Connection = struct {
    id: u64,
    remote: Endpoint,
    protocol: Protocol,
    state: ConnectionState,
    socket_fd: ?u64,
    component_id: []const u8, // owned by ConnectionPool
    bytes_sent: u64,
    bytes_recv: u64,
    created_ms: i64,
    last_used_ms: i64,
    last_health_ms: i64,
    health_failures: u32,
};

pub const ConnectionPool = struct {
    allocator: std.mem.Allocator,
    connections: std.AutoHashMap(u64, Connection),
    comp_names: std.AutoHashMap(u64, []u8), // id → owned component_id copy
    next_id: u64,
    capacity: usize,
    /// Maximum health failures before a connection is marked unhealthy.
    max_failures: u32,

    pub fn init(allocator: std.mem.Allocator, capacity: usize, max_failures: u32) ConnectionPool {
        return .{
            .allocator = allocator,
            .connections = std.AutoHashMap(u64, Connection).init(allocator),
            .comp_names = std.AutoHashMap(u64, []u8).init(allocator),
            .next_id = 1,
            .capacity = capacity,
            .max_failures = max_failures,
        };
    }

    pub fn deinit(self: *ConnectionPool) void {
        var it = self.comp_names.valueIterator();
        while (it.next()) |name| self.allocator.free(name.*);
        self.comp_names.deinit();
        self.connections.deinit();
    }

    /// Acquire a new connection slot.  Returns connection ID.
    pub fn acquire(
        self: *ConnectionPool,
        actor: Role,
        component_id: []const u8,
        remote: Endpoint,
        protocol: Protocol,
    ) !u64 {
        if (!hasPermission(actor, .manage_modules)) return NetworkError.AccessDenied;
        if (self.connections.count() >= self.capacity) return NetworkError.ConnectionPoolExhausted;

        const id = self.next_id;
        self.next_id += 1;
        const now = std.time.milliTimestamp();

        const comp_copy = try self.allocator.dupe(u8, component_id);
        errdefer self.allocator.free(comp_copy);
        try self.comp_names.put(id, comp_copy);

        try self.connections.put(id, .{
            .id = id,
            .remote = remote,
            .protocol = protocol,
            .state = .idle,
            .socket_fd = null,
            .component_id = comp_copy,
            .bytes_sent = 0,
            .bytes_recv = 0,
            .created_ms = now,
            .last_used_ms = now,
            .last_health_ms = now,
            .health_failures = 0,
        });
        return id;
    }

    /// Activate a connection (begin data transfer).
    pub fn activate(self: *ConnectionPool, id: u64, socket_fd: ?u64) !void {
        const c = self.connections.getPtr(id) orelse return NetworkError.ConnectionNotFound;
        c.state = .active;
        c.socket_fd = socket_fd;
        c.last_used_ms = std.time.milliTimestamp();
    }

    /// Record IO on a connection.
    pub fn recordIO(self: *ConnectionPool, id: u64, sent: u64, recv: u64) !void {
        const c = self.connections.getPtr(id) orelse return NetworkError.ConnectionNotFound;
        c.bytes_sent += sent;
        c.bytes_recv += recv;
        c.last_used_ms = std.time.milliTimestamp();
    }

    /// Report a health-check result.  Auto-marks unhealthy after `max_failures`.
    pub fn healthCheck(self: *ConnectionPool, id: u64, healthy: bool) !void {
        const c = self.connections.getPtr(id) orelse return NetworkError.ConnectionNotFound;
        c.last_health_ms = std.time.milliTimestamp();
        if (healthy) {
            c.health_failures = 0;
            if (c.state == .unhealthy) c.state = .idle;
        } else {
            c.health_failures += 1;
            if (c.health_failures >= self.max_failures) c.state = .unhealthy;
        }
    }

    /// Begin graceful drain — connection finishes in-flight work then closes.
    pub fn drain(self: *ConnectionPool, actor: Role, id: u64) !void {
        if (!hasPermission(actor, .manage_modules)) return NetworkError.AccessDenied;
        const c = self.connections.getPtr(id) orelse return NetworkError.ConnectionNotFound;
        c.state = .draining;
    }

    /// Release a connection back to the pool (or fully close it).
    pub fn release(self: *ConnectionPool, actor: Role, id: u64) !void {
        if (!hasPermission(actor, .manage_modules)) return NetworkError.AccessDenied;
        const c = self.connections.getPtr(id) orelse return NetworkError.ConnectionNotFound;
        if (c.state == .draining or c.state == .unhealthy) {
            if (self.comp_names.get(id)) |name| {
                self.allocator.free(name);
                _ = self.comp_names.remove(id);
            }
            _ = self.connections.remove(id);
        } else {
            c.state = .idle;
            c.last_used_ms = std.time.milliTimestamp();
        }
    }

    pub fn activeCount(self: *const ConnectionPool) usize {
        var n: usize = 0;
        var it = self.connections.valueIterator();
        while (it.next()) |c| {
            if (c.state == .active) n += 1;
        }
        return n;
    }

    pub fn idleCount(self: *const ConnectionPool) usize {
        var n: usize = 0;
        var it = self.connections.valueIterator();
        while (it.next()) |c| {
            if (c.state == .idle) n += 1;
        }
        return n;
    }

    pub fn unhealthyCount(self: *const ConnectionPool) usize {
        var n: usize = 0;
        var it = self.connections.valueIterator();
        while (it.next()) |c| {
            if (c.state == .unhealthy) n += 1;
        }
        return n;
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// 5.  Traffic Ledger  (per-component ingress/egress + quota enforcement)
// ─────────────────────────────────────────────────────────────────────────────

/// Per-component traffic counters and optional byte quotas.
pub const TrafficRecord = struct {
    component_id: []const u8, // owned by TrafficLedger
    ingress_bytes: u64,
    egress_bytes: u64,
    ingress_packets: u64,
    egress_packets: u64,
    dropped_bytes: u64,
    /// Optional hard cap per direction (0 = unlimited).
    ingress_quota_bytes: u64,
    egress_quota_bytes: u64,
    last_reset_ms: i64,
};

pub const TrafficLedger = struct {
    allocator: std.mem.Allocator,
    records: std.StringHashMap(TrafficRecord),

    pub fn init(allocator: std.mem.Allocator) TrafficLedger {
        return .{ .allocator = allocator, .records = std.StringHashMap(TrafficRecord).init(allocator) };
    }

    pub fn deinit(self: *TrafficLedger) void {
        var it = self.records.valueIterator();
        while (it.next()) |r| self.allocator.free(r.component_id);
        self.records.deinit();
    }

    /// Register a component (idempotent — silently skips if already present).
    pub fn register(
        self: *TrafficLedger,
        component_id: []const u8,
        ingress_quota_bytes: u64,
        egress_quota_bytes: u64,
    ) !void {
        if (self.records.contains(component_id)) return;
        const comp_copy = try self.allocator.dupe(u8, component_id);
        errdefer self.allocator.free(comp_copy);
        try self.records.put(comp_copy, .{
            .component_id = comp_copy,
            .ingress_bytes = 0,
            .egress_bytes = 0,
            .ingress_packets = 0,
            .egress_packets = 0,
            .dropped_bytes = 0,
            .ingress_quota_bytes = ingress_quota_bytes,
            .egress_quota_bytes = egress_quota_bytes,
            .last_reset_ms = std.time.milliTimestamp(),
        });
    }

    /// Record ingress traffic.  Returns QuotaExceeded if over limit.
    pub fn recordIngress(self: *TrafficLedger, component_id: []const u8, bytes: u64) !void {
        const r = self.records.getPtr(component_id) orelse return NetworkError.ManagerNotFound;
        if (r.ingress_quota_bytes > 0 and r.ingress_bytes + bytes > r.ingress_quota_bytes) {
            r.dropped_bytes += bytes;
            return NetworkError.QuotaExceeded;
        }
        r.ingress_bytes += bytes;
        r.ingress_packets += 1;
    }

    /// Record egress traffic.  Returns QuotaExceeded if over limit.
    pub fn recordEgress(self: *TrafficLedger, component_id: []const u8, bytes: u64) !void {
        const r = self.records.getPtr(component_id) orelse return NetworkError.ManagerNotFound;
        if (r.egress_quota_bytes > 0 and r.egress_bytes + bytes > r.egress_quota_bytes) {
            r.dropped_bytes += bytes;
            return NetworkError.QuotaExceeded;
        }
        r.egress_bytes += bytes;
        r.egress_packets += 1;
    }

    /// Reset counters for a component (e.g. at the start of a billing window).
    pub fn resetCounters(self: *TrafficLedger, actor: Role, component_id: []const u8) !void {
        if (!hasPermission(actor, .manage_modules)) return NetworkError.AccessDenied;
        const r = self.records.getPtr(component_id) orelse return NetworkError.ManagerNotFound;
        r.ingress_bytes = 0;
        r.egress_bytes = 0;
        r.ingress_packets = 0;
        r.egress_packets = 0;
        r.dropped_bytes = 0;
        r.last_reset_ms = std.time.milliTimestamp();
    }

    pub fn get(self: *const TrafficLedger, component_id: []const u8) ?TrafficRecord {
        return self.records.get(component_id);
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// 6.  Packet Queue  (priority-lane in-kernel packet buffer)
// ─────────────────────────────────────────────────────────────────────────────

pub const PacketPriority = enum(u8) { best_effort = 0, normal = 1, expedited = 2, control = 3 };

pub const Packet = struct {
    id: u64,
    src: Endpoint,
    dst: Endpoint,
    protocol: Protocol,
    priority: PacketPriority,
    payload_len: u32,
    /// Inline payload buffer (capped for kernel use — real kernels use sk_buff).
    payload: [256]u8,
    enqueued_ms: i64,
    iface_name: []const u8, // borrowed — caller must keep alive
};

pub const PacketQueue = struct {
    allocator: std.mem.Allocator,
    lanes: [4]std.ArrayList(Packet),
    next_id: u64,
    lane_cap: usize,

    pub fn init(allocator: std.mem.Allocator, lane_capacity: usize) PacketQueue {
        return .{
            .allocator = allocator,
            .lanes = .{
                std.ArrayList(Packet).init(allocator),
                std.ArrayList(Packet).init(allocator),
                std.ArrayList(Packet).init(allocator),
                std.ArrayList(Packet).init(allocator),
            },
            .next_id = 1,
            .lane_cap = lane_capacity,
        };
    }

    pub fn deinit(self: *PacketQueue) void {
        for (&self.lanes) |*lane| lane.deinit();
    }

    /// Enqueue a packet.  Returns the packet ID.
    pub fn enqueue(
        self: *PacketQueue,
        src: Endpoint,
        dst: Endpoint,
        protocol: Protocol,
        priority: PacketPriority,
        payload: []const u8,
        iface: []const u8,
    ) !u64 {
        const idx = @intFromEnum(priority);
        if (self.lanes[idx].items.len >= self.lane_cap) return NetworkError.PacketQueueFull;

        const id = self.next_id;
        self.next_id += 1;

        var pkt = Packet{
            .id = id,
            .src = src,
            .dst = dst,
            .protocol = protocol,
            .priority = priority,
            .payload_len = @intCast(@min(payload.len, 256)),
            .payload = undefined,
            .enqueued_ms = std.time.milliTimestamp(),
            .iface_name = iface,
        };
        const copy_len = @min(payload.len, 256);
        @memcpy(pkt.payload[0..copy_len], payload[0..copy_len]);

        try self.lanes[idx].append(pkt);
        return id;
    }

    /// Dequeue the highest-priority available packet, or null if empty.
    pub fn dequeue(self: *PacketQueue) ?Packet {
        var idx: usize = 3;
        while (true) {
            if (self.lanes[idx].items.len > 0) return self.lanes[idx].orderedRemove(0);
            if (idx == 0) break;
            idx -= 1;
        }
        return null;
    }

    pub fn totalLen(self: *const PacketQueue) usize {
        var n: usize = 0;
        for (&self.lanes) |*lane| n += lane.items.len;
        return n;
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// 7.  Route Table  (static longest-prefix match)
// ─────────────────────────────────────────────────────────────────────────────

pub const RouteEntry = struct {
    network: Ipv4, // destination network address
    prefix_len: u6, // CIDR prefix length
    gateway: Ipv4, // next-hop
    iface_name: []const u8, // owned by RouteTable
    metric: u32, // lower = preferred
};

pub const RouteTable = struct {
    allocator: std.mem.Allocator,
    routes: std.ArrayList(RouteEntry),

    pub fn init(allocator: std.mem.Allocator) RouteTable {
        return .{ .allocator = allocator, .routes = std.ArrayList(RouteEntry).init(allocator) };
    }

    pub fn deinit(self: *RouteTable) void {
        for (self.routes.items) |r| self.allocator.free(r.iface_name);
        self.routes.deinit();
    }

    /// Add a static route.
    pub fn addRoute(
        self: *RouteTable,
        actor: Role,
        network: Ipv4,
        prefix_len: u6,
        gateway: Ipv4,
        iface_name: []const u8,
        metric: u32,
    ) !void {
        if (!hasPermission(actor, .manage_modules)) return NetworkError.AccessDenied;
        // Check for duplicate.
        for (self.routes.items) |r| {
            if (r.network.eql(network) and r.prefix_len == prefix_len)
                return NetworkError.RouteAlreadyExists;
        }
        const iface_copy = try self.allocator.dupe(u8, iface_name);
        errdefer self.allocator.free(iface_copy);
        try self.routes.append(.{
            .network = network,
            .prefix_len = prefix_len,
            .gateway = gateway,
            .iface_name = iface_copy,
            .metric = metric,
        });
    }

    /// Remove a route.
    pub fn removeRoute(self: *RouteTable, actor: Role, network: Ipv4, prefix_len: u6) !void {
        if (!hasPermission(actor, .manage_modules)) return NetworkError.AccessDenied;
        for (self.routes.items, 0..) |r, i| {
            if (r.network.eql(network) and r.prefix_len == prefix_len) {
                self.allocator.free(r.iface_name);
                _ = self.routes.swapRemove(i);
                return;
            }
        }
        return NetworkError.RouteNotFound;
    }

    /// Longest-prefix match for `dst`.  Returns the best RouteEntry or null.
    pub fn lookup(self: *const RouteTable, dst: Ipv4) ?RouteEntry {
        var best: ?RouteEntry = null;
        for (self.routes.items) |r| {
            if (dst.masked(r.prefix_len).eql(r.network)) {
                if (best == null or r.prefix_len > best.?.prefix_len or
                    (r.prefix_len == best.?.prefix_len and r.metric < best.?.metric))
                {
                    best = r;
                }
            }
        }
        return best;
    }

    pub fn count(self: *const RouteTable) usize {
        return self.routes.items.len;
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// 8.  Firewall Rule Set
// ─────────────────────────────────────────────────────────────────────────────

pub const FirewallAction = enum { allow, deny };
pub const FirewallDirection = enum { ingress, egress, both };

pub const FirewallRule = struct {
    id: u32,
    priority: u32, // lower = evaluated first
    direction: FirewallDirection,
    src_net: ?Ipv4, // null = any
    src_prefix: u6,
    dst_net: ?Ipv4, // null = any
    dst_prefix: u6,
    protocol: ?Protocol, // null = any
    src_port: ?u16, // null = any
    dst_port: ?u16, // null = any
    action: FirewallAction,
    enabled: bool,
    hit_count: u64,
};

pub const FirewallRuleSet = struct {
    allocator: std.mem.Allocator,
    rules: std.ArrayList(FirewallRule),
    next_id: u32,
    /// Default action when no rule matches.
    default: FirewallAction,

    pub fn init(allocator: std.mem.Allocator, default_action: FirewallAction) FirewallRuleSet {
        return .{
            .allocator = allocator,
            .rules = std.ArrayList(FirewallRule).init(allocator),
            .next_id = 1,
            .default = default_action,
        };
    }

    pub fn deinit(self: *FirewallRuleSet) void {
        self.rules.deinit();
    }

    /// Insert a firewall rule.  Returns its assigned ID.
    pub fn addRule(
        self: *FirewallRuleSet,
        actor: Role,
        priority: u32,
        direction: FirewallDirection,
        src_net: ?Ipv4,
        src_prefix: u6,
        dst_net: ?Ipv4,
        dst_prefix: u6,
        protocol: ?Protocol,
        src_port: ?u16,
        dst_port: ?u16,
        action: FirewallAction,
    ) !u32 {
        if (!hasPermission(actor, .kernel_admin)) return NetworkError.AccessDenied;

        const id = self.next_id;
        self.next_id += 1;

        try self.rules.append(.{
            .id = id,
            .priority = priority,
            .direction = direction,
            .src_net = src_net,
            .src_prefix = src_prefix,
            .dst_net = dst_net,
            .dst_prefix = dst_prefix,
            .protocol = protocol,
            .src_port = src_port,
            .dst_port = dst_port,
            .action = action,
            .enabled = true,
            .hit_count = 0,
        });

        // Keep rules sorted by ascending priority.
        std.sort.insertion(FirewallRule, self.rules.items, {}, struct {
            pub fn lessThan(_: void, a: FirewallRule, b: FirewallRule) bool {
                return a.priority < b.priority;
            }
        }.lessThan);

        return id;
    }

    /// Enable or disable a rule by ID.
    pub fn setEnabled(self: *FirewallRuleSet, actor: Role, rule_id: u32, enabled: bool) !void {
        if (!hasPermission(actor, .kernel_admin)) return NetworkError.AccessDenied;
        for (self.rules.items) |*r| {
            if (r.id == rule_id) {
                r.enabled = enabled;
                return;
            }
        }
    }

    /// Evaluate a packet against the rule set.
    /// `direction` is from the packet's perspective (ingress = arriving).
    pub fn evaluate(
        self: *FirewallRuleSet,
        src: Endpoint,
        dst: Endpoint,
        protocol: Protocol,
        direction: FirewallDirection,
    ) FirewallAction {
        for (self.rules.items) |*r| {
            if (!r.enabled) continue;
            if (r.direction != .both and r.direction != direction) continue;
            if (r.protocol) |p| {
                if (p != protocol) continue;
            }
            if (r.src_net) |n| {
                if (!src.addr.masked(r.src_prefix).eql(n)) continue;
            }
            if (r.dst_net) |n| {
                if (!dst.addr.masked(r.dst_prefix).eql(n)) continue;
            }
            if (r.src_port) |p| {
                if (src.port != p) continue;
            }
            if (r.dst_port) |p| {
                if (dst.port != p) continue;
            }

            r.hit_count += 1;
            return r.action;
        }
        return self.default;
    }

    pub fn count(self: *const FirewallRuleSet) usize {
        return self.rules.items.len;
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// 9.  Network Manager  (top-level coordinator)
// ─────────────────────────────────────────────────────────────────────────────

/// Named network manager record (mirrors kernel.zig's network_manager field).
pub const NetworkManagerRecord = struct {
    name: []const u8, // owned by NetworkManager
    kind: []const u8, // "kernel-native" | "kogi-go-network" | custom — owned
    active: bool,
    registered_ms: i64,
};

/// Top-level network subsystem — owns and coordinates all sub-registries.
pub const NetworkManager = struct {
    allocator: std.mem.Allocator,
    addresses: AddressTable,
    interfaces: InterfaceRegistry,
    sockets: SocketTable,
    connections: ConnectionPool,
    ledger: TrafficLedger,
    packets: PacketQueue,
    routes: RouteTable,
    firewall: FirewallRuleSet,
    managers: std.StringHashMap(NetworkManagerRecord),

    pub fn init(
        allocator: std.mem.Allocator,
        max_sockets: usize,
        max_connections: usize,
        max_conn_failures: u32,
        packet_lane_cap: usize,
        firewall_default: FirewallAction,
    ) NetworkManager {
        return .{
            .allocator = allocator,
            .addresses = AddressTable.init(allocator),
            .interfaces = InterfaceRegistry.init(allocator),
            .sockets = SocketTable.init(allocator, max_sockets),
            .connections = ConnectionPool.init(allocator, max_connections, max_conn_failures),
            .ledger = TrafficLedger.init(allocator),
            .packets = PacketQueue.init(allocator, packet_lane_cap),
            .routes = RouteTable.init(allocator),
            .firewall = FirewallRuleSet.init(allocator, firewall_default),
            .managers = std.StringHashMap(NetworkManagerRecord).init(allocator),
        };
    }

    pub fn deinit(self: *NetworkManager) void {
        self.addresses.deinit();
        self.interfaces.deinit();
        self.sockets.deinit();
        self.connections.deinit();
        self.ledger.deinit();
        self.packets.deinit();
        self.routes.deinit();
        self.firewall.deinit();

        var it = self.managers.valueIterator();
        while (it.next()) |m| {
            self.allocator.free(m.name);
            self.allocator.free(m.kind);
        }
        self.managers.deinit();
    }

    // ── Named manager registry ────────────────────────────────────────────

    /// Register a named network manager (e.g. "kernel-native", "kogi-go-network").
    pub fn registerManager(self: *NetworkManager, actor: Role, name: []const u8, kind: []const u8) !void {
        if (!hasPermission(actor, .manage_modules)) return NetworkError.AccessDenied;
        if (self.managers.contains(name)) return;

        const name_copy = try self.allocator.dupe(u8, name);
        errdefer self.allocator.free(name_copy);
        const kind_copy = try self.allocator.dupe(u8, kind);
        errdefer self.allocator.free(kind_copy);

        try self.managers.put(name_copy, .{
            .name = name_copy,
            .kind = kind_copy,
            .active = true,
            .registered_ms = std.time.milliTimestamp(),
        });
    }

    pub fn getManager(self: *const NetworkManager, name: []const u8) ?NetworkManagerRecord {
        return self.managers.get(name);
    }

    // ── High-level send path ──────────────────────────────────────────────

    /// Kernel send path:
    ///   1. Firewall egress check
    ///   2. Route lookup
    ///   3. Interface up-check + traffic recording
    ///   4. Traffic ledger egress accounting
    ///   5. Packet enqueue
    /// Returns the packet ID on success.
    pub fn send(
        self: *NetworkManager,
        actor: Role,
        component_id: []const u8,
        src: Endpoint,
        dst: Endpoint,
        protocol: Protocol,
        priority: PacketPriority,
        payload: []const u8,
    ) !u64 {
        if (!hasPermission(actor, .manage_modules)) return NetworkError.AccessDenied;

        // 1. Firewall
        if (self.firewall.evaluate(src, dst, protocol, .egress) == .deny)
            return NetworkError.FirewallDenied;

        // 2. Route
        const route = self.routes.lookup(dst.addr) orelse return NetworkError.RouteNotFound;

        // 3. Interface
        try self.interfaces.recordTraffic(route.iface_name, 0, payload.len);

        // 4. Ledger
        try self.ledger.recordEgress(component_id, payload.len);

        // 5. Enqueue
        return self.packets.enqueue(src, dst, protocol, priority, payload, route.iface_name);
    }

    /// Kernel receive path:
    ///   1. Firewall ingress check
    ///   2. Interface traffic recording
    ///   3. Traffic ledger ingress accounting
    ///   4. Packet enqueue
    pub fn receive(
        self: *NetworkManager,
        component_id: []const u8,
        iface_name: []const u8,
        src: Endpoint,
        dst: Endpoint,
        protocol: Protocol,
        priority: PacketPriority,
        payload: []const u8,
    ) !u64 {
        // 1. Firewall
        if (self.firewall.evaluate(src, dst, protocol, .ingress) == .deny) {
            try self.interfaces.recordDrop(iface_name);
            return NetworkError.FirewallDenied;
        }

        // 2. Interface
        try self.interfaces.recordTraffic(iface_name, payload.len, 0);

        // 3. Ledger
        try self.ledger.recordIngress(component_id, payload.len);

        // 4. Enqueue
        return self.packets.enqueue(src, dst, protocol, priority, payload, iface_name);
    }

    // ── Stats ─────────────────────────────────────────────────────────────

    pub const Stats = struct {
        address_count: usize,
        interface_count: usize,
        open_sockets: usize,
        active_connections: usize,
        idle_connections: usize,
        unhealthy_conns: usize,
        queued_packets: usize,
        route_count: usize,
        firewall_rules: usize,
        manager_count: usize,
    };

    pub fn stats(self: *const NetworkManager) Stats {
        return .{
            .address_count = self.addresses.count(),
            .interface_count = self.interfaces.count(),
            .open_sockets = self.sockets.openCount(),
            .active_connections = self.connections.activeCount(),
            .idle_connections = self.connections.idleCount(),
            .unhealthy_conns = self.connections.unhealthyCount(),
            .queued_packets = self.packets.totalLen(),
            .route_count = self.routes.count(),
            .firewall_rules = self.firewall.count(),
            .manager_count = self.managers.count(),
        };
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

test "Ipv4: parse and format" {
    const ip = try Ipv4.parse("192.168.1.10");
    try std.testing.expectEqual(@as(u8, 192), ip.octets[0]);
    try std.testing.expectEqual(@as(u8, 10), ip.octets[3]);

    var buf: [16]u8 = undefined;
    const s = try ip.format(&buf);
    try std.testing.expectEqualStrings("192.168.1.10", s);
}

test "Ipv4: masked subnet calculation" {
    const ip = try Ipv4.parse("192.168.1.55");
    const net = ip.masked(24);
    try std.testing.expect(net.eql(try Ipv4.parse("192.168.1.0")));

    const wide = ip.masked(16);
    try std.testing.expect(wide.eql(try Ipv4.parse("192.168.0.0")));
}

test "address table: register and resolve" {
    var table = AddressTable.init(std.testing.allocator);
    defer table.deinit();

    try table.register(
        .host,
        "kogi.server",
        try Ipv4.parse("127.0.0.1"),
        8080,
        "kogi.server",
        "http",
    );

    const rec = table.resolve("kogi.server").?;
    try std.testing.expectEqual(@as(u16, 8080), rec.port);
    try std.testing.expectError(
        NetworkError.AddressAlreadyRegistered,
        table.register(.host, "kogi.server", try Ipv4.parse("127.0.0.1"), 9090, "x", "http"),
    );

    try table.deregister(.host, "kogi.server");
    try std.testing.expectEqual(@as(usize, 0), table.count());
}

test "interface registry: create, state, traffic" {
    var reg = InterfaceRegistry.init(std.testing.allocator);
    defer reg.deinit();

    try reg.create(.host, "eth0", try Ipv4.parse("10.0.0.1"), 24, 1500);
    try std.testing.expectEqual(InterfaceState.down, reg.get("eth0").?.state);

    // Traffic on a down interface should fail.
    try std.testing.expectError(
        NetworkError.InterfaceDown,
        reg.recordTraffic("eth0", 512, 256),
    );

    try reg.setState(.host, "eth0", .up);
    try reg.recordTraffic("eth0", 1024, 512);

    const iface = reg.get("eth0").?;
    try std.testing.expectEqual(@as(u64, 1024), iface.ingress_bytes);
    try std.testing.expectEqual(@as(u64, 512), iface.egress_bytes);
}

test "socket table: open, bind, connect, close" {
    var st = SocketTable.init(std.testing.allocator, 16);
    defer st.deinit();

    const fd = try st.open(.host, .tcp, null);
    try std.testing.expectEqual(SocketState.unbound, st.get(fd).?.state);

    const local = Endpoint.init(try Ipv4.parse("0.0.0.0"), 8080);
    try st.bind(.host, fd, local, "eth0");
    try std.testing.expectEqual(SocketState.bound, st.get(fd).?.state);

    const remote = Endpoint.init(try Ipv4.parse("10.0.0.2"), 443);
    try st.connect(.host, fd, remote);
    try std.testing.expectEqual(SocketState.connected, st.get(fd).?.state);

    try st.recordIO(fd, 200, 400);
    try std.testing.expectEqual(@as(u64, 200), st.get(fd).?.bytes_sent);

    try st.close(.host, fd);
    try std.testing.expectEqual(@as(usize, 0), st.openCount());
}

test "socket table: exhaustion" {
    var st = SocketTable.init(std.testing.allocator, 2);
    defer st.deinit();

    _ = try st.open(.host, .tcp, null);
    _ = try st.open(.host, .udp, null);
    try std.testing.expectError(NetworkError.SocketExhausted, st.open(.host, .tcp, null));
}

test "connection pool: acquire, activate, health, release" {
    var pool = ConnectionPool.init(std.testing.allocator, 8, 3);
    defer pool.deinit();

    const remote = Endpoint.init(try Ipv4.parse("10.0.0.5"), 443);
    const id = try pool.acquire(.host, "kogi.server", remote, .tcp);
    try std.testing.expectEqual(@as(usize, 1), pool.idleCount());

    try pool.activate(id, null);
    try std.testing.expectEqual(@as(usize, 1), pool.activeCount());

    try pool.recordIO(id, 1024, 2048);

    // Three consecutive failures → unhealthy.
    try pool.healthCheck(id, false);
    try pool.healthCheck(id, false);
    try pool.healthCheck(id, false);
    try std.testing.expectEqual(@as(usize, 1), pool.unhealthyCount());

    // Drain + release should remove the connection.
    try pool.drain(.host, id);
    try pool.release(.host, id);
    try std.testing.expectEqual(@as(usize, 0), pool.activeCount() + pool.idleCount());
}

test "traffic ledger: quota enforcement" {
    var ledger = TrafficLedger.init(std.testing.allocator);
    defer ledger.deinit();

    try ledger.register("svc-a", 1000, 500);
    try ledger.recordIngress("svc-a", 800);
    try std.testing.expectError(
        NetworkError.QuotaExceeded,
        ledger.recordIngress("svc-a", 300),
    );

    const rec = ledger.get("svc-a").?;
    try std.testing.expectEqual(@as(u64, 300), rec.dropped_bytes);

    try ledger.resetCounters(.host, "svc-a");
    try std.testing.expectEqual(@as(u64, 0), ledger.get("svc-a").?.ingress_bytes);
}

test "packet queue: priority ordering" {
    var q = PacketQueue.init(std.testing.allocator, 32);
    defer q.deinit();

    const a = Endpoint.init(try Ipv4.parse("10.0.0.1"), 1000);
    const b = Endpoint.init(try Ipv4.parse("10.0.0.2"), 2000);

    _ = try q.enqueue(a, b, .tcp, .best_effort, "hello", "eth0");
    _ = try q.enqueue(a, b, .tcp, .control, "ctrl", "eth0");
    _ = try q.enqueue(a, b, .tcp, .normal, "norm", "eth0");

    const first = q.dequeue().?;
    try std.testing.expectEqual(PacketPriority.control, first.priority);

    const second = q.dequeue().?;
    try std.testing.expectEqual(PacketPriority.normal, second.priority);
}

test "route table: longest prefix match" {
    var rt = RouteTable.init(std.testing.allocator);
    defer rt.deinit();

    try rt.addRoute(.host, try Ipv4.parse("0.0.0.0"), 0, try Ipv4.parse("10.0.0.1"), "eth0", 100);
    try rt.addRoute(.host, try Ipv4.parse("10.0.0.0"), 8, try Ipv4.parse("10.0.0.1"), "eth1", 50);
    try rt.addRoute(.host, try Ipv4.parse("10.0.1.0"), 24, try Ipv4.parse("10.0.0.1"), "eth2", 10);

    // Specific /24 should win over /8.
    const r1 = rt.lookup(try Ipv4.parse("10.0.1.55")).?;
    try std.testing.expectEqual(@as(u6, 24), r1.prefix_len);
    try std.testing.expectEqualStrings("eth2", r1.iface_name);

    // /8 should win over default route.
    const r2 = rt.lookup(try Ipv4.parse("10.0.2.1")).?;
    try std.testing.expectEqual(@as(u6, 8), r2.prefix_len);

    // Unknown subnet falls back to default route.
    const r3 = rt.lookup(try Ipv4.parse("8.8.8.8")).?;
    try std.testing.expectEqual(@as(u6, 0), r3.prefix_len);

    try rt.removeRoute(.host, try Ipv4.parse("10.0.1.0"), 24);
    try std.testing.expectEqual(@as(usize, 2), rt.count());
}

test "firewall: allow and deny rules" {
    var fw = FirewallRuleSet.init(std.testing.allocator, .allow);
    defer fw.deinit();

    // Deny all TCP to port 22 (SSH) from any source.
    _ = try fw.addRule(
        .root,
        10,
        .both,
        null,
        0,
        null,
        0,
        .tcp,
        null,
        22,
        .deny,
    );

    const src = Endpoint.init(try Ipv4.parse("10.0.0.5"), 55000);
    const ssh = Endpoint.init(try Ipv4.parse("10.0.0.1"), 22);
    const web = Endpoint.init(try Ipv4.parse("10.0.0.1"), 443);

    try std.testing.expectEqual(FirewallAction.deny, fw.evaluate(src, ssh, .tcp, .ingress));
    try std.testing.expectEqual(FirewallAction.allow, fw.evaluate(src, web, .tcp, .ingress));
}

test "firewall: default deny with explicit allow" {
    var fw = FirewallRuleSet.init(std.testing.allocator, .deny);
    defer fw.deinit();

    _ = try fw.addRule(
        .root,
        5,
        .ingress,
        null,
        0,
        null,
        0,
        .tcp,
        null,
        443,
        .allow,
    );

    const src = Endpoint.init(try Ipv4.parse("1.2.3.4"), 60000);
    const https = Endpoint.init(try Ipv4.parse("10.0.0.1"), 443);
    const http = Endpoint.init(try Ipv4.parse("10.0.0.1"), 80);

    try std.testing.expectEqual(FirewallAction.allow, fw.evaluate(src, https, .tcp, .ingress));
    try std.testing.expectEqual(FirewallAction.deny, fw.evaluate(src, http, .tcp, .ingress));
}

test "network manager: full send path" {
    var nm = NetworkManager.init(
        std.testing.allocator,
        64, // max sockets
        32, // max connections
        3, // max health failures
        64, // packet lane cap
        .allow, // default firewall action
    );
    defer nm.deinit();

    // Register built-in managers (mirrors kernel.zig bootstrap).
    try nm.registerManager(.host, "kernel-native", "kernel-native");
    try nm.registerManager(.host, "kogi-go-network", "proxy");

    // Provision interface and route.
    try nm.interfaces.create(.host, "eth0", try Ipv4.parse("10.0.0.1"), 24, 1500);
    try nm.interfaces.setState(.host, "eth0", .up);
    try nm.routes.addRoute(.host, try Ipv4.parse("0.0.0.0"), 0, try Ipv4.parse("10.0.0.1"), "eth0", 100);

    // Register component in ledger.
    try nm.ledger.register("kogi.server", 0, 0);

    // Send a packet.
    const src = Endpoint.init(try Ipv4.parse("10.0.0.1"), 9000);
    const dst = Endpoint.init(try Ipv4.parse("8.8.8.8"), 53);
    const pkt_id = try nm.send(.host, "kogi.server", src, dst, .udp, .normal, "hello");
    try std.testing.expect(pkt_id > 0);
    try std.testing.expectEqual(@as(usize, 1), nm.packets.totalLen());

    const s = nm.stats();
    try std.testing.expectEqual(@as(usize, 2), s.manager_count);
    try std.testing.expectEqual(@as(usize, 1), s.route_count);
    try std.testing.expectEqual(@as(usize, 1), s.queued_packets);
}

test "network manager: firewall blocks send" {
    var nm = NetworkManager.init(std.testing.allocator, 16, 8, 3, 32, .allow);
    defer nm.deinit();

    try nm.interfaces.create(.host, "eth0", try Ipv4.parse("10.0.0.1"), 24, 1500);
    try nm.interfaces.setState(.host, "eth0", .up);
    try nm.routes.addRoute(.host, try Ipv4.parse("0.0.0.0"), 0, try Ipv4.parse("10.0.0.1"), "eth0", 100);
    try nm.ledger.register("kogi.server", 0, 0);

    // Block all UDP.
    _ = try nm.firewall.addRule(.root, 1, .egress, null, 0, null, 0, .udp, null, null, .deny);

    const src = Endpoint.init(try Ipv4.parse("10.0.0.1"), 9000);
    const dst = Endpoint.init(try Ipv4.parse("8.8.8.8"), 53);
    try std.testing.expectError(
        NetworkError.FirewallDenied,
        nm.send(.host, "kogi.server", src, dst, .udp, .normal, "query"),
    );
}
