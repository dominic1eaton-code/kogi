//! events.zig — Kernel Event Management System
//!
//! Sits at the same layer as resources.zig — below kernel.zig and above the
//! individual subsystem files — and provides a single, unified surface for
//! every event that flows through the kernel.
//!
//! Design goals
//! ────────────
//!  • One authoritative event record (`KernelEvent`) that can represent any
//!    occurrence across all six subsystems.
//!  • Structured routing: subscribers register interest in one or more
//!    `EventDomain` values; the dispatcher delivers only matching events.
//!  • Replay / audit: the bounded `EventLog` keeps every event in arrival
//!    order so callers can reconstruct what happened since any watermark.
//!  • Backpressure: the log is capped; when full, the oldest event is
//!    overwritten (ring-buffer semantics) and an overflow counter is
//!    incremented so no event is silently lost without a trace.
//!  • No imports from kernel.zig (avoids circular dependency); only the
//!    shared primitive types (Role / Permission) imported from resources.zig.
//!
//! Provides
//! ────────
//!   EventDomain      — which subsystem a KernelEvent originates from
//!   EventSeverity    — debug / info / warn / error / fatal
//!   EventKind        — fine-grained taxonomy within each domain
//!   KernelEvent      — canonical, domain-agnostic event record
//!   EventFilter      — predicate for matching events (domain + severity + source)
//!   Subscriber       — named handler + filter registered with the dispatcher
//!   EventDispatcher  — fan-out delivery to all matching subscribers
//!   EventLog         — bounded ring-buffer append-only event log
//!   EventManager     — top-level coordinator (public API)

const std = @import("std");

const res_mod = @import("resources.zig");

// ─────────────────────────────────────────────────────────────────────────────
// Re-export shared primitives
// ─────────────────────────────────────────────────────────────────────────────

pub const Role = res_mod.Role;
pub const Permission = res_mod.Permission;
pub const hasPermission = res_mod.hasPermission;

// ─────────────────────────────────────────────────────────────────────────────
// Errors
// ─────────────────────────────────────────────────────────────────────────────

pub const EventError = error{
    AccessDenied,
    SubscriberAlreadyRegistered,
    SubscriberNotFound,
    SubscriberLimitReached,
    InvalidFilter,
    EventLogFull, // returned only in strict mode; default = ring-buffer
};

// ─────────────────────────────────────────────────────────────────────────────
// 1.  Domain — which subsystem the event originates from
// ─────────────────────────────────────────────────────────────────────────────

pub const EventDomain = enum(u8) {
    /// Events emitted by kernel.zig itself (mode changes, bootstrapping).
    kernel,
    /// Events from memory.zig (allocation, pressure, audit).
    memory,
    /// Events from processes.zig (spawn, terminate, scheduler).
    process,
    /// Events from network.zig (address, socket, connection, firewall).
    network,
    /// Events from module.zig (register, start, stop, fault).
    module,
    /// Events from services.zig (provision, health, restart).
    service,
    /// Events from resources.zig (tenant, limit-exceeded, ceiling-hit).
    resource,
    /// User-defined or cross-cutting events not tied to one subsystem.
    custom,
};

pub const EventDomainSet = std.EnumSet(EventDomain);

// ─────────────────────────────────────────────────────────────────────────────
// 2.  Severity
// ─────────────────────────────────────────────────────────────────────────────

pub const EventSeverity = enum(u8) {
    debug = 0,
    info = 1,
    warn = 2,
    @"error" = 3,
    fatal = 4,

    pub fn atLeast(self: EventSeverity, min: EventSeverity) bool {
        return @intFromEnum(self) >= @intFromEnum(min);
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// 3.  EventKind — fine-grained taxonomy
// ─────────────────────────────────────────────────────────────────────────────

pub const EventKind = enum {
    // ── Kernel ──────────────────────────────────────────────────────────────
    kernel_booted,
    kernel_mode_changed,
    kernel_shutdown_initiated,
    kernel_relinked,

    // ── Memory ──────────────────────────────────────────────────────────────
    memory_allocated,
    memory_freed,
    memory_pressure_raised,
    memory_pressure_cleared,
    memory_limit_exceeded,
    memory_global_exhausted,
    memory_tenant_registered,
    memory_tenant_removed,

    // ── Process ─────────────────────────────────────────────────────────────
    process_spawned,
    process_terminated,
    process_faulted,
    process_state_changed,
    scheduler_job_fired,
    scheduler_job_queued,
    worker_dispatched,
    worker_completed,

    // ── Network ─────────────────────────────────────────────────────────────
    network_address_registered,
    network_address_deregistered,
    network_interface_created,
    network_interface_state_changed,
    network_socket_opened,
    network_socket_closed,
    network_connection_established,
    network_connection_closed,
    network_connection_failed,
    network_firewall_blocked,
    network_route_added,
    network_route_removed,
    network_traffic_recorded,

    // ── Module ──────────────────────────────────────────────────────────────
    module_registered,
    module_provisioning,
    module_started,
    module_suspended,
    module_resumed,
    module_draining,
    module_stopped,
    module_faulted,
    module_restarted,
    module_removed,
    module_capability_granted,
    module_capability_revoked,

    // ── Service ─────────────────────────────────────────────────────────────
    service_registered,
    service_provisioned,
    service_started,
    service_suspended,
    service_resumed,
    service_draining,
    service_stopped,
    service_faulted,
    service_restarted,
    service_removed,
    service_health_ok,
    service_health_degraded,
    service_health_failed,

    // ── Resource ────────────────────────────────────────────────────────────
    resource_tenant_registered,
    resource_tenant_removed,
    resource_acquired,
    resource_released,
    resource_limit_exceeded,
    resource_global_ceiling_hit,
    resource_all_released,

    // ── Custom ──────────────────────────────────────────────────────────────
    custom,
};

// ─────────────────────────────────────────────────────────────────────────────
// 4.  KernelEvent — the canonical event record
// ─────────────────────────────────────────────────────────────────────────────

/// Maximum byte length for an inline payload stored directly in KernelEvent.
/// Payloads larger than this are truncated; callers that need to carry
/// structured data beyond this limit should publish a reference ID instead.
pub const MAX_PAYLOAD_BYTES: usize = 256;

/// Maximum byte length for the source_id field.
pub const MAX_SOURCE_ID_BYTES: usize = 64;

/// A single observable occurrence anywhere in the kernel.
pub const KernelEvent = struct {
    /// Monotonically incrementing identifier assigned by EventLog.
    id: u64,
    /// Wall-clock timestamp (milliseconds since epoch).
    emitted_ms: i64,
    /// Which subsystem produced this event.
    domain: EventDomain,
    /// What happened.
    kind: EventKind,
    /// How serious.
    severity: EventSeverity,
    /// Identity of the module / component / tenant that caused the event.
    /// Fixed-size buffer; zero-terminated if shorter than MAX_SOURCE_ID_BYTES.
    source_id: [MAX_SOURCE_ID_BYTES]u8,
    source_len: u8,
    /// Free-form human-readable description or JSON snippet.
    payload: [MAX_PAYLOAD_BYTES]u8,
    payload_len: u16,

    // ── Helpers ──────────────────────────────────────────────────────────

    pub fn sourceSlice(self: *const KernelEvent) []const u8 {
        return self.source_id[0..self.source_len];
    }

    pub fn payloadSlice(self: *const KernelEvent) []const u8 {
        return self.payload[0..self.payload_len];
    }
};

/// Build a KernelEvent without an allocated log (id will be 0).
pub fn buildEvent(
    domain: EventDomain,
    kind: EventKind,
    severity: EventSeverity,
    source_id: []const u8,
    payload: []const u8,
) KernelEvent {
    var ev = KernelEvent{
        .id = 0,
        .emitted_ms = std.time.milliTimestamp(),
        .domain = domain,
        .kind = kind,
        .severity = severity,
        .source_id = [_]u8{0} ** MAX_SOURCE_ID_BYTES,
        .source_len = 0,
        .payload = [_]u8{0} ** MAX_PAYLOAD_BYTES,
        .payload_len = 0,
    };
    const slen = @min(source_id.len, MAX_SOURCE_ID_BYTES);
    @memcpy(ev.source_id[0..slen], source_id[0..slen]);
    ev.source_len = @intCast(slen);

    const plen = @min(payload.len, MAX_PAYLOAD_BYTES);
    @memcpy(ev.payload[0..plen], payload[0..plen]);
    ev.payload_len = @intCast(plen);
    return ev;
}

// ─────────────────────────────────────────────────────────────────────────────
// 5.  EventFilter — predicate that subscribers use to select events
// ─────────────────────────────────────────────────────────────────────────────

/// A filter matches a KernelEvent when ALL specified constraints are satisfied.
/// A zero-value EventFilter (all fields at default) matches every event.
pub const EventFilter = struct {
    /// Restrict to one or more domains (empty set = accept all domains).
    domains: EventDomainSet = EventDomainSet.initEmpty(),
    /// Minimum severity level (inclusive).  Events below this are ignored.
    min_severity: EventSeverity = .debug,
    /// When non-empty, only events whose source_id starts with this prefix
    /// are delivered (useful for filtering by module ID namespace).
    source_prefix: []const u8 = "",

    pub fn matchesAll() EventFilter {
        return .{};
    }

    pub fn forDomain(domain: EventDomain) EventFilter {
        return .{ .domains = EventDomainSet.initOne(domain) };
    }

    pub fn forDomains(domains: EventDomainSet) EventFilter {
        return .{ .domains = domains };
    }

    pub fn atSeverity(min: EventSeverity) EventFilter {
        return .{ .min_severity = min };
    }

    pub fn matches(self: *const EventFilter, ev: *const KernelEvent) bool {
        // Domain check
        if (!self.domains.eql(EventDomainSet.initEmpty()) and
            !self.domains.contains(ev.domain)) return false;

        // Severity check
        if (!ev.severity.atLeast(self.min_severity)) return false;

        // Source prefix check
        if (self.source_prefix.len > 0) {
            const src = ev.sourceSlice();
            if (!std.mem.startsWith(u8, src, self.source_prefix)) return false;
        }

        return true;
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// 6.  Subscriber — a named handler registered with the dispatcher
// ─────────────────────────────────────────────────────────────────────────────

/// Function pointer type for event delivery.
/// `ctx` is the opaque context pointer supplied at registration time.
/// The handler must not block and must not call back into EventManager.
pub const HandlerFn = *const fn (ctx: ?*anyopaque, ev: *const KernelEvent) void;

/// Maximum number of simultaneously registered subscribers.
pub const MAX_SUBSCRIBERS: usize = 64;

/// Maximum byte length for a subscriber name.
pub const MAX_SUBSCRIBER_NAME: usize = 48;

pub const Subscriber = struct {
    name: [MAX_SUBSCRIBER_NAME]u8,
    name_len: u8,
    filter: EventFilter,
    handler: HandlerFn,
    ctx: ?*anyopaque,
    /// Total events delivered to this subscriber since registration.
    delivered: u64,
    /// Total events that matched the filter but were dropped (handler returned
    /// before this subscriber was registered, or dispatch was suspended).
    skipped: u64,
    active: bool,

    pub fn nameSlice(self: *const Subscriber) []const u8 {
        return self.name[0..self.name_len];
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// 7.  EventLog — bounded ring-buffer, append-only within a generation
// ─────────────────────────────────────────────────────────────────────────────

/// A bounded circular log.  When full, the slot at `head` is overwritten and
/// `overflow_count` is incremented so callers know data was lost.
///
/// Events are addressed by their monotonic `id` field; callers can retrieve
/// all events since a watermark with `since(watermark, out)`.
pub const EventLog = struct {
    allocator: std.mem.Allocator,
    entries: []KernelEvent,
    capacity: usize,
    /// Index of the next write slot (wraps).
    head: usize,
    /// Total events ever appended (never wraps — used as watermark).
    total_appended: u64,
    /// Number of times an existing slot was overwritten due to a full buffer.
    overflow_count: u64,
    /// Monotonic counter used to stamp each KernelEvent.id.
    next_id: u64,

    pub fn init(allocator: std.mem.Allocator, capacity: usize) !EventLog {
        const cap = if (capacity == 0) 1 else capacity;
        const entries = try allocator.alloc(KernelEvent, cap);
        return .{
            .allocator = allocator,
            .entries = entries,
            .capacity = cap,
            .head = 0,
            .total_appended = 0,
            .overflow_count = 0,
            .next_id = 1,
        };
    }

    pub fn deinit(self: *EventLog) void {
        self.allocator.free(self.entries);
    }

    /// Append an event.  Overwrites oldest slot when the log is full.
    /// Assigns ev.id and ev.emitted_ms if they are zero.
    pub fn append(self: *EventLog, ev: KernelEvent) void {
        var e = ev;
        e.id = self.next_id;
        if (e.emitted_ms == 0) e.emitted_ms = std.time.milliTimestamp();
        self.next_id += 1;

        if (self.total_appended >= self.capacity) self.overflow_count += 1;
        self.entries[self.head] = e;
        self.head = (self.head + 1) % self.capacity;
        self.total_appended += 1;
    }

    /// Copy all events with id > `watermark` into `out`, in arrival order.
    /// The caller supplies the ArrayList; it is not cleared first.
    pub fn since(
        self: *const EventLog,
        watermark: u64,
        out: *std.ArrayList(KernelEvent),
    ) !void {
        // Walk the ring from oldest to newest.
        const count = @min(self.total_appended, self.capacity);
        if (count == 0) return;

        // The oldest slot is `head` when the ring is full; otherwise slot 0.
        const start: usize = if (self.total_appended >= self.capacity)
            self.head
        else
            0;

        var i: usize = 0;
        while (i < count) : (i += 1) {
            const slot = (start + i) % self.capacity;
            const e = &self.entries[slot];
            if (e.id > watermark) try out.append(e.*);
        }
    }

    /// Return the most recent event, or null if the log is empty.
    pub fn latest(self: *const EventLog) ?KernelEvent {
        if (self.total_appended == 0) return null;
        const slot = (self.head + self.capacity - 1) % self.capacity;
        return self.entries[slot];
    }

    /// Return all events matching `filter` since `watermark`.
    pub fn query(
        self: *const EventLog,
        filter: *const EventFilter,
        watermark: u64,
        out: *std.ArrayList(KernelEvent),
    ) !void {
        const count = @min(self.total_appended, self.capacity);
        if (count == 0) return;
        const start: usize = if (self.total_appended >= self.capacity)
            self.head
        else
            0;

        var i: usize = 0;
        while (i < count) : (i += 1) {
            const slot = (start + i) % self.capacity;
            const e = &self.entries[slot];
            if (e.id > watermark and filter.matches(e)) try out.append(e.*);
        }
    }

    pub fn len(self: *const EventLog) usize {
        return @min(self.total_appended, self.capacity);
    }

    pub fn isEmpty(self: *const EventLog) bool {
        return self.total_appended == 0;
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// 8.  EventDispatcher — synchronous fan-out to registered subscribers
// ─────────────────────────────────────────────────────────────────────────────

/// Maintains a fixed-size array of Subscriber slots.  Dispatch is O(n)
/// where n = MAX_SUBSCRIBERS but n is bounded and small, so this is
/// preferable to a heap-allocated list for a kernel-level component.
pub const EventDispatcher = struct {
    slots: [MAX_SUBSCRIBERS]Subscriber,
    count: usize,
    /// When true, all dispatch calls return immediately without delivery.
    suspended: bool,

    pub fn init() EventDispatcher {
        return .{
            .slots = undefined,
            .count = 0,
            .suspended = false,
        };
    }

    // ── Subscriber management ─────────────────────────────────────────────

    /// Register a subscriber.  Names must be unique within the dispatcher.
    pub fn subscribe(
        self: *EventDispatcher,
        actor: Role,
        name: []const u8,
        filter: EventFilter,
        handler: HandlerFn,
        ctx: ?*anyopaque,
    ) !void {
        if (!hasPermission(actor, .manage_modules)) return EventError.AccessDenied;
        if (self.count >= MAX_SUBSCRIBERS) return EventError.SubscriberLimitReached;

        // Duplicate name check
        for (self.slots[0..self.count]) |*s| {
            if (s.active and std.mem.eql(u8, s.nameSlice(), name))
                return EventError.SubscriberAlreadyRegistered;
        }

        var sub = Subscriber{
            .name = [_]u8{0} ** MAX_SUBSCRIBER_NAME,
            .name_len = 0,
            .filter = filter,
            .handler = handler,
            .ctx = ctx,
            .delivered = 0,
            .skipped = 0,
            .active = true,
        };
        const nlen = @min(name.len, MAX_SUBSCRIBER_NAME);
        @memcpy(sub.name[0..nlen], name[0..nlen]);
        sub.name_len = @intCast(nlen);

        self.slots[self.count] = sub;
        self.count += 1;
    }

    /// Remove a subscriber by name.
    pub fn unsubscribe(self: *EventDispatcher, actor: Role, name: []const u8) !void {
        if (!hasPermission(actor, .manage_modules)) return EventError.AccessDenied;
        for (self.slots[0..self.count]) |*s| {
            if (s.active and std.mem.eql(u8, s.nameSlice(), name)) {
                s.active = false;
                return;
            }
        }
        return EventError.SubscriberNotFound;
    }

    /// Return a pointer to a subscriber slot by name, or null.
    pub fn getSubscriber(self: *EventDispatcher, name: []const u8) ?*Subscriber {
        for (self.slots[0..self.count]) |*s| {
            if (s.active and std.mem.eql(u8, s.nameSlice(), name)) return s;
        }
        return null;
    }

    pub fn activeCount(self: *const EventDispatcher) usize {
        var n: usize = 0;
        for (self.slots[0..self.count]) |*s| {
            if (s.active) n += 1;
        }
        return n;
    }

    // ── Dispatch ─────────────────────────────────────────────────────────

    /// Deliver `ev` to every active subscriber whose filter matches.
    pub fn dispatch(self: *EventDispatcher, ev: *const KernelEvent) void {
        if (self.suspended) return;
        for (self.slots[0..self.count]) |*s| {
            if (!s.active) continue;
            if (s.filter.matches(ev)) {
                s.handler(s.ctx, ev);
                s.delivered += 1;
            } else {
                s.skipped += 1;
            }
        }
    }

    pub fn suspend_(self: *EventDispatcher) void {
        self.suspended = true;
    }

    pub fn resume_(self: *EventDispatcher) void {
        self.suspended = false;
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// 9.  EventStats — snapshot returned by EventManager.stats()
// ─────────────────────────────────────────────────────────────────────────────

pub const EventStats = struct {
    /// Total events appended to the log since boot (monotonic, never wraps).
    total_events: u64,
    /// Events currently retained in the log ring buffer.
    log_len: usize,
    /// Ring-buffer capacity.
    log_capacity: usize,
    /// Number of times the oldest event was overwritten due to a full buffer.
    overflow_count: u64,
    /// Number of active subscribers.
    active_subscribers: usize,
    /// Highest event ID in the log (0 if empty).
    latest_id: u64,
};

// ─────────────────────────────────────────────────────────────────────────────
// 10. EventManager — top-level coordinator (public API)
// ─────────────────────────────────────────────────────────────────────────────

/// The EventManager owns an EventLog and an EventDispatcher.  Every publish
/// call appends to the log and synchronously fans out to registered subscribers.
///
/// Subsystems obtain a pointer to EventManager from the Kernel after init and
/// call `publish` or the domain-specific helpers to emit events.
///
/// Kernel.relink() must be called after init to hand the EventManager pointer
/// to each subsystem bridge.
pub const EventManager = struct {
    allocator: std.mem.Allocator,
    log: EventLog,
    dispatcher: EventDispatcher,

    // ── Lifecycle ─────────────────────────────────────────────────────────

    pub fn init(allocator: std.mem.Allocator, log_capacity: usize) !EventManager {
        return .{
            .allocator = allocator,
            .log = try EventLog.init(allocator, log_capacity),
            .dispatcher = EventDispatcher.init(),
        };
    }

    pub fn deinit(self: *EventManager) void {
        self.log.deinit();
    }

    // ── Core publish ──────────────────────────────────────────────────────

    /// Append `ev` to the log and fan-out to subscribers.
    /// Assigns ev.id; the caller does not need to set it.
    pub fn publish(self: *EventManager, ev: KernelEvent) void {
        self.log.append(ev);
        // log.append stamped .id; retrieve the stamped copy for dispatch.
        const stamped = self.log.latest() orelse return;
        self.dispatcher.dispatch(&stamped);
    }

    /// Build and publish in one call.
    pub fn emit(
        self: *EventManager,
        domain: EventDomain,
        kind: EventKind,
        severity: EventSeverity,
        source_id: []const u8,
        payload: []const u8,
    ) void {
        self.publish(buildEvent(domain, kind, severity, source_id, payload));
    }

    // ── Domain helpers ────────────────────────────────────────────────────
    //
    // One thin wrapper per domain so call-sites stay readable.

    pub fn emitKernel(self: *EventManager, kind: EventKind, severity: EventSeverity, payload: []const u8) void {
        self.emit(.kernel, kind, severity, "kernel", payload);
    }

    pub fn emitMemory(self: *EventManager, kind: EventKind, severity: EventSeverity, source: []const u8, payload: []const u8) void {
        self.emit(.memory, kind, severity, source, payload);
    }

    pub fn emitProcess(self: *EventManager, kind: EventKind, severity: EventSeverity, source: []const u8, payload: []const u8) void {
        self.emit(.process, kind, severity, source, payload);
    }

    pub fn emitNetwork(self: *EventManager, kind: EventKind, severity: EventSeverity, source: []const u8, payload: []const u8) void {
        self.emit(.network, kind, severity, source, payload);
    }

    pub fn emitModule(self: *EventManager, kind: EventKind, severity: EventSeverity, source: []const u8, payload: []const u8) void {
        self.emit(.module, kind, severity, source, payload);
    }

    pub fn emitService(self: *EventManager, kind: EventKind, severity: EventSeverity, source: []const u8, payload: []const u8) void {
        self.emit(.service, kind, severity, source, payload);
    }

    pub fn emitResource(self: *EventManager, kind: EventKind, severity: EventSeverity, source: []const u8, payload: []const u8) void {
        self.emit(.resource, kind, severity, source, payload);
    }

    pub fn emitCustom(self: *EventManager, source: []const u8, payload: []const u8) void {
        self.emit(.custom, .custom, .info, source, payload);
    }

    // ── Subscriber management (delegates to dispatcher) ───────────────────

    pub fn subscribe(
        self: *EventManager,
        actor: Role,
        name: []const u8,
        filter: EventFilter,
        handler: HandlerFn,
        ctx: ?*anyopaque,
    ) !void {
        try self.dispatcher.subscribe(actor, name, filter, handler, ctx);
    }

    pub fn unsubscribe(self: *EventManager, actor: Role, name: []const u8) !void {
        try self.dispatcher.unsubscribe(actor, name);
    }

    pub fn getSubscriber(self: *EventManager, name: []const u8) ?*Subscriber {
        return self.dispatcher.getSubscriber(name);
    }

    pub fn suspendDispatch(self: *EventManager) void {
        self.dispatcher.suspend_();
    }

    pub fn resumeDispatch(self: *EventManager) void {
        self.dispatcher.resume_();
    }

    // ── Log queries ───────────────────────────────────────────────────────

    /// All events with id > watermark, in arrival order.
    pub fn since(
        self: *const EventManager,
        watermark: u64,
        out: *std.ArrayList(KernelEvent),
    ) !void {
        try self.log.since(watermark, out);
    }

    /// Events matching `filter` with id > watermark.
    pub fn query(
        self: *const EventManager,
        filter: *const EventFilter,
        watermark: u64,
        out: *std.ArrayList(KernelEvent),
    ) !void {
        try self.log.query(filter, watermark, out);
    }

    /// Most recent event, or null if no events have been published.
    pub fn latest(self: *const EventManager) ?KernelEvent {
        return self.log.latest();
    }

    // ── Stats ─────────────────────────────────────────────────────────────

    pub fn stats(self: *const EventManager) EventStats {
        const latest_id: u64 = if (self.log.latest()) |e| e.id else 0;
        return .{
            .total_events = self.log.total_appended,
            .log_len = self.log.len(),
            .log_capacity = self.log.capacity,
            .overflow_count = self.log.overflow_count,
            .active_subscribers = self.dispatcher.activeCount(),
            .latest_id = latest_id,
        };
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

test "buildEvent: source and payload are truncated to fixed-size buffers" {
    const ev = buildEvent(.kernel, .kernel_booted, .info, "kogi.kernel", "booted");
    try std.testing.expectEqualStrings("kogi.kernel", ev.sourceSlice());
    try std.testing.expectEqualStrings("booted", ev.payloadSlice());
}

test "buildEvent: oversized source is truncated without panic" {
    const long_src = "a" ** (MAX_SOURCE_ID_BYTES + 10);
    const ev = buildEvent(.custom, .custom, .debug, long_src, "");
    try std.testing.expectEqual(@as(usize, MAX_SOURCE_ID_BYTES), ev.source_len);
}

test "EventFilter.matches: domain filter" {
    const f = EventFilter.forDomain(.memory);
    const mem_ev = buildEvent(.memory, .memory_allocated, .info, "t", "");
    const proc_ev = buildEvent(.process, .process_spawned, .info, "t", "");
    try std.testing.expect(f.matches(&mem_ev));
    try std.testing.expect(!f.matches(&proc_ev));
}

test "EventFilter.matches: severity filter" {
    const f = EventFilter.atSeverity(.warn);
    const warn_ev = buildEvent(.kernel, .kernel_booted, .warn, "k", "");
    const debug_ev = buildEvent(.kernel, .kernel_booted, .debug, "k", "");
    try std.testing.expect(f.matches(&warn_ev));
    try std.testing.expect(!f.matches(&debug_ev));
}

test "EventFilter.matches: source prefix filter" {
    const f = EventFilter{ .source_prefix = "kogi." };
    const match_ev = buildEvent(.module, .module_started, .info, "kogi.office", "");
    const other_ev = buildEvent(.module, .module_started, .info, "acme.mod", "");
    try std.testing.expect(f.matches(&match_ev));
    try std.testing.expect(!f.matches(&other_ev));
}

test "EventLog: append and since" {
    var log = try EventLog.init(std.testing.allocator, 8);
    defer log.deinit();

    log.append(buildEvent(.kernel, .kernel_booted, .info, "k", "a"));
    log.append(buildEvent(.memory, .memory_allocated, .info, "m", "b"));
    log.append(buildEvent(.module, .module_started, .info, "x", "c"));

    try std.testing.expectEqual(@as(usize, 3), log.len());

    var out = std.ArrayList(KernelEvent).init(std.testing.allocator);
    defer out.deinit();

    // since(0) should return all three
    try log.since(0, &out);
    try std.testing.expectEqual(@as(usize, 3), out.items.len);

    // since first event's id returns only the two newer ones
    out.clearRetainingCapacity();
    try log.since(out.items.len, &out); // watermark = 0 cleared; re-use var

    out.clearRetainingCapacity();
    const first_id = log.entries[0].id;
    try log.since(first_id, &out);
    try std.testing.expectEqual(@as(usize, 2), out.items.len);
}

test "EventLog: ring-buffer overflow increments overflow_count" {
    var log = try EventLog.init(std.testing.allocator, 4);
    defer log.deinit();

    // Fill the ring exactly
    var i: usize = 0;
    while (i < 4) : (i += 1)
        log.append(buildEvent(.custom, .custom, .info, "s", "x"));

    try std.testing.expectEqual(@as(u64, 0), log.overflow_count);

    // One more — should overwrite
    log.append(buildEvent(.custom, .custom, .info, "s", "overflow"));
    try std.testing.expectEqual(@as(u64, 1), log.overflow_count);
    try std.testing.expectEqual(@as(usize, 4), log.len()); // still capped
}

test "EventLog: query filters correctly" {
    var log = try EventLog.init(std.testing.allocator, 16);
    defer log.deinit();

    log.append(buildEvent(.memory, .memory_allocated, .info, "m", ""));
    log.append(buildEvent(.process, .process_spawned, .warn, "p", ""));
    log.append(buildEvent(.memory, .memory_freed, .debug, "m2", ""));

    const f = EventFilter.forDomain(.memory);
    var out = std.ArrayList(KernelEvent).init(std.testing.allocator);
    defer out.deinit();

    try log.query(&f, 0, &out);
    try std.testing.expectEqual(@as(usize, 2), out.items.len);
}

test "EventLog: latest returns most recent event" {
    var log = try EventLog.init(std.testing.allocator, 8);
    defer log.deinit();

    try std.testing.expect(log.latest() == null);

    log.append(buildEvent(.kernel, .kernel_booted, .info, "a", "first"));
    log.append(buildEvent(.module, .module_started, .info, "b", "second"));
    log.append(buildEvent(.service, .service_started, .info, "c", "third"));

    const l = log.latest().?;
    try std.testing.expectEqualStrings("third", l.payloadSlice());
}

test "EventDispatcher: subscribe and dispatch" {
    // Use a simple counter via global (tests can't easily close over mut state)
    const S = struct {
        var count: usize = 0;
        fn handler(_: ?*anyopaque, _: *const KernelEvent) void {
            count += 1;
        }
    };
    S.count = 0;

    var d = EventDispatcher.init();
    try d.subscribe(.host, "watcher", EventFilter.matchesAll(), S.handler, null);

    const ev = buildEvent(.kernel, .kernel_booted, .info, "k", "");
    d.dispatch(&ev);
    d.dispatch(&ev);

    try std.testing.expectEqual(@as(usize, 2), S.count);
    try std.testing.expectEqual(@as(u64, 2), d.getSubscriber("watcher").?.delivered);
}

test "EventDispatcher: filter restricts delivery" {
    const S = struct {
        var count: usize = 0;
        fn handler(_: ?*anyopaque, _: *const KernelEvent) void {
            count += 1;
        }
    };
    S.count = 0;

    var d = EventDispatcher.init();
    try d.subscribe(.host, "mem-only", EventFilter.forDomain(.memory), S.handler, null);

    d.dispatch(&buildEvent(.memory, .memory_allocated, .info, "m", ""));
    d.dispatch(&buildEvent(.process, .process_spawned, .info, "p", ""));

    try std.testing.expectEqual(@as(usize, 1), S.count);
}

test "EventDispatcher: unsubscribe stops delivery" {
    const S = struct {
        var count: usize = 0;
        fn handler(_: ?*anyopaque, _: *const KernelEvent) void {
            count += 1;
        }
    };
    S.count = 0;

    var d = EventDispatcher.init();
    try d.subscribe(.host, "sub", EventFilter.matchesAll(), S.handler, null);
    d.dispatch(&buildEvent(.kernel, .kernel_booted, .info, "k", ""));
    try d.unsubscribe(.host, "sub");
    d.dispatch(&buildEvent(.kernel, .kernel_booted, .info, "k", ""));

    try std.testing.expectEqual(@as(usize, 1), S.count);
}

test "EventDispatcher: suspend and resume" {
    const S = struct {
        var count: usize = 0;
        fn handler(_: ?*anyopaque, _: *const KernelEvent) void {
            count += 1;
        }
    };
    S.count = 0;

    var d = EventDispatcher.init();
    try d.subscribe(.host, "s", EventFilter.matchesAll(), S.handler, null);

    d.suspend_();
    d.dispatch(&buildEvent(.kernel, .kernel_booted, .info, "k", ""));
    try std.testing.expectEqual(@as(usize, 0), S.count);

    d.resume_();
    d.dispatch(&buildEvent(.kernel, .kernel_booted, .info, "k", ""));
    try std.testing.expectEqual(@as(usize, 1), S.count);
}

test "EventDispatcher: duplicate subscriber name rejected" {
    const S = struct {
        fn h(_: ?*anyopaque, _: *const KernelEvent) void {}
    };
    var d = EventDispatcher.init();
    try d.subscribe(.host, "dup", EventFilter.matchesAll(), S.h, null);
    try std.testing.expectError(
        EventError.SubscriberAlreadyRegistered,
        d.subscribe(.host, "dup", EventFilter.matchesAll(), S.h, null),
    );
}

test "EventDispatcher: access denied for unprivileged role" {
    const S = struct {
        fn h(_: ?*anyopaque, _: *const KernelEvent) void {}
    };
    var d = EventDispatcher.init();
    try std.testing.expectError(
        EventError.AccessDenied,
        d.subscribe(.user, "evil", EventFilter.matchesAll(), S.h, null),
    );
}

test "EventManager: emit appends to log and dispatches" {
    const S = struct {
        var count: usize = 0;
        fn handler(_: ?*anyopaque, _: *const KernelEvent) void {
            count += 1;
        }
    };
    S.count = 0;

    var mgr = try EventManager.init(std.testing.allocator, 32);
    defer mgr.deinit();

    try mgr.subscribe(.host, "all", EventFilter.matchesAll(), S.handler, null);

    mgr.emitKernel(.kernel_booted, .info, "{}");
    mgr.emitModule(.module_started, .info, "kogi.office", "{}");
    mgr.emitService(.service_started, .warn, "svc-a", "degraded");

    try std.testing.expectEqual(@as(usize, 3), S.count);
    const s = mgr.stats();
    try std.testing.expectEqual(@as(u64, 3), s.total_events);
    try std.testing.expectEqual(@as(usize, 1), s.active_subscribers);
    try std.testing.expectEqual(@as(u64, 3), s.latest_id);
}

test "EventManager: query returns only matching events" {
    var mgr = try EventManager.init(std.testing.allocator, 32);
    defer mgr.deinit();

    mgr.emitKernel(.kernel_booted, .info, "{}");
    mgr.emitMemory(.memory_allocated, .info, "t", "{}");
    mgr.emitMemory(.memory_freed, .debug, "t", "{}");
    mgr.emitModule(.module_started, .info, "x", "{}");

    const f = EventFilter.forDomain(.memory);
    var out = std.ArrayList(KernelEvent).init(std.testing.allocator);
    defer out.deinit();

    try mgr.query(&f, 0, &out);
    try std.testing.expectEqual(@as(usize, 2), out.items.len);
    for (out.items) |e| try std.testing.expectEqual(EventDomain.memory, e.domain);
}

test "EventManager: since(watermark) pages through events" {
    var mgr = try EventManager.init(std.testing.allocator, 32);
    defer mgr.deinit();

    mgr.emitKernel(.kernel_booted, .info, "first");
    mgr.emitKernel(.kernel_booted, .info, "second");
    const mark = mgr.stats().latest_id;
    mgr.emitKernel(.kernel_booted, .info, "third");
    mgr.emitKernel(.kernel_booted, .info, "fourth");

    var out = std.ArrayList(KernelEvent).init(std.testing.allocator);
    defer out.deinit();
    try mgr.since(mark, &out);

    try std.testing.expectEqual(@as(usize, 2), out.items.len);
    try std.testing.expectEqualStrings("third", out.items[0].payloadSlice());
    try std.testing.expectEqualStrings("fourth", out.items[1].payloadSlice());
}

test "EventManager: stats overflow_count tracks ring-buffer wraps" {
    var mgr = try EventManager.init(std.testing.allocator, 4);
    defer mgr.deinit();

    var i: usize = 0;
    while (i < 7) : (i += 1) mgr.emitKernel(.kernel_booted, .debug, "x");

    const s = mgr.stats();
    try std.testing.expectEqual(@as(u64, 7), s.total_events);
    try std.testing.expectEqual(@as(usize, 4), s.log_len); // capped at capacity
    try std.testing.expectEqual(@as(u64, 3), s.overflow_count);
}
