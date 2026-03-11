//! processes.zig — Kernel Process Management System
//!
//! Kernel extension extracted and greatly expanded from kernel.zig.
//!
//! Provides:
//!   ProcessTable      — authoritative registry of all live processes
//!   ThreadPool        — fixed-size reusable worker-thread pool
//!   WorkQueue         — FIFO task queue with priority lanes
//!   Scheduler         — tick-driven dispatcher (one-shot + recurring jobs)
//!   ResourceLedger    — acquire / release of named resource units per tenant
//!   Orchestrator      — lifecycle coordinator: spawn, supervise, terminate groups
//!
//! All wall-clock operations use i64 millisecond timestamps consistent
//! with the rest of the kernel codebase.

const std = @import("std");

// ─────────────────────────────────────────────────────────────────────────────
// Shared types (mirror kernel.zig so this file is self-contained)
// ─────────────────────────────────────────────────────────────────────────────

pub const Role = enum {
    root,
    host,
    server,
    module_runtime,
    user,
};

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
            .kernel_admin,
            .schedule_tasks,
            .manage_modules,
            .manage_memory,
            .manage_processes,
            .manage_files,
            .read_audit,
            => true,
        },
        .server => switch (permission) {
            .schedule_tasks,
            .manage_modules,
            .manage_processes,
            .manage_files,
            => true,
            .kernel_admin,
            .manage_memory,
            .read_audit,
            => false,
        },
        .module_runtime => switch (permission) {
            .schedule_tasks,
            .manage_files,
            => true,
            .kernel_admin,
            .manage_modules,
            .manage_memory,
            .manage_processes,
            .read_audit,
            => false,
        },
        .user => permission == .read_audit,
    };
}

// ─────────────────────────────────────────────────────────────────────────────
// Errors
// ─────────────────────────────────────────────────────────────────────────────

pub const ProcessError = error{
    AccessDenied,
    ProcessNotFound,
    ProcessAlreadyExists,
    InvalidStateTransition,
    ThreadPoolExhausted,
    QueueFull,
    ResourceNotFound,
    ResourceLimitExceeded,
    ResourceNotAcquired,
    GroupNotFound,
    GroupAlreadyExists,
    ScheduleEntryNotFound,
    WorkerNotFound,
};

// ─────────────────────────────────────────────────────────────────────────────
// 1.  Process Table
// ─────────────────────────────────────────────────────────────────────────────

/// Life-cycle states a process may move through.
pub const ProcessState = enum {
    ready,
    running,
    waiting,
    suspended,
    terminated,

    /// Returns true when a transition from `self` → `next` is legal.
    pub fn canTransitionTo(self: ProcessState, next: ProcessState) bool {
        return switch (self) {
            .ready => next == .running or next == .terminated,
            .running => next == .waiting or next == .suspended or next == .terminated,
            .waiting => next == .ready or next == .running or next == .terminated,
            .suspended => next == .ready or next == .terminated,
            .terminated => false,
        };
    }
};

/// Extended process descriptor (superset of kernel.zig ProcessRecord).
pub const ProcessRecord = struct {
    pid: u64,
    owner: Role,
    name: []const u8, // owned by ProcessTable
    state: ProcessState,
    priority: u8, // 0 = lowest … 255 = highest
    group_id: ?[]const u8, // optional supervision group (owned by ProcessTable)
    cpu_time_ms: u64, // accumulated CPU-equivalent time
    created_at_ms: i64,
    last_state_ms: i64, // timestamp of most recent state transition
};

/// Authoritative table of all processes, keyed by PID.
pub const ProcessTable = struct {
    allocator: std.mem.Allocator,
    records: std.AutoHashMap(u64, ProcessRecord),
    next_pid: u64,

    pub fn init(allocator: std.mem.Allocator) ProcessTable {
        return .{
            .allocator = allocator,
            .records = std.AutoHashMap(u64, ProcessRecord).init(allocator),
            .next_pid = 1000,
        };
    }

    pub fn deinit(self: *ProcessTable) void {
        var it = self.records.valueIterator();
        while (it.next()) |r| {
            self.allocator.free(r.name);
            if (r.group_id) |g| self.allocator.free(g);
        }
        self.records.deinit();
    }

    /// Spawn a new process and return its PID.
    pub fn spawn(
        self: *ProcessTable,
        actor: Role,
        name: []const u8,
        owner: Role,
        priority: u8,
        group_id: ?[]const u8,
    ) !u64 {
        if (!hasPermission(actor, .manage_processes)) return ProcessError.AccessDenied;

        const pid = self.next_pid;
        self.next_pid += 1;
        const now = std.time.milliTimestamp();

        const name_copy = try self.allocator.dupe(u8, name);
        errdefer self.allocator.free(name_copy);

        const group_copy: ?[]const u8 = if (group_id) |g|
            try self.allocator.dupe(u8, g)
        else
            null;
        errdefer if (group_copy) |g| self.allocator.free(g);

        try self.records.put(pid, .{
            .pid = pid,
            .owner = owner,
            .name = name_copy,
            .state = .ready,
            .priority = priority,
            .group_id = group_copy,
            .cpu_time_ms = 0,
            .created_at_ms = now,
            .last_state_ms = now,
        });

        return pid;
    }

    /// Transition `pid` to `next_state`, enforcing legal state machine edges.
    pub fn transition(self: *ProcessTable, actor: Role, pid: u64, next_state: ProcessState) !void {
        if (!hasPermission(actor, .manage_processes)) return ProcessError.AccessDenied;

        const r = self.records.getPtr(pid) orelse return ProcessError.ProcessNotFound;
        if (!r.state.canTransitionTo(next_state)) return ProcessError.InvalidStateTransition;

        r.state = next_state;
        r.last_state_ms = std.time.milliTimestamp();
    }

    /// Add `delta_ms` of CPU time to a process record.
    pub fn chargeCpu(self: *ProcessTable, pid: u64, delta_ms: u64) !void {
        const r = self.records.getPtr(pid) orelse return ProcessError.ProcessNotFound;
        r.cpu_time_ms += delta_ms;
    }

    /// Return a copy of the record for `pid`.
    pub fn get(self: *const ProcessTable, pid: u64) ?ProcessRecord {
        return self.records.get(pid);
    }

    /// Collect PIDs of all processes in `group_id`.
    pub fn pidsByGroup(
        self: *const ProcessTable,
        group_id: []const u8,
        out: *std.ArrayList(u64),
    ) !void {
        var it = self.records.valueIterator();
        while (it.next()) |r| {
            if (r.group_id) |g| {
                if (std.mem.eql(u8, g, group_id)) try out.append(r.pid);
            }
        }
    }

    /// Count of live (non-terminated) processes.
    pub fn liveCount(self: *const ProcessTable) usize {
        var count: usize = 0;
        var it = self.records.valueIterator();
        while (it.next()) |r| {
            if (r.state != .terminated) count += 1;
        }
        return count;
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// 2.  Work Queue  (priority FIFO lanes)
// ─────────────────────────────────────────────────────────────────────────────

/// Priority levels for queued tasks — higher value = higher urgency.
pub const TaskPriority = enum(u8) {
    low = 0,
    normal = 1,
    high = 2,
    urgent = 3,
};

/// A unit of work held in the queue.
pub const Task = struct {
    id: u64,
    label: []const u8, // owned by WorkQueue
    priority: TaskPriority,
    payload: []const u8, // owned by WorkQueue; arbitrary byte payload
    enqueued_ms: i64,
    /// Optional PID of the process that enqueued this task.
    owner_pid: ?u64,
};

/// Bounded multi-priority FIFO queue.
/// Each priority lane is an independent ring of at most `lane_cap` tasks.
pub const WorkQueue = struct {
    allocator: std.mem.Allocator,
    lanes: [4]std.ArrayList(Task), // indexed by TaskPriority
    next_id: u64,
    lane_cap: usize,

    pub fn init(allocator: std.mem.Allocator, lane_capacity: usize) WorkQueue {
        return .{
            .allocator = allocator,
            .lanes = .{
                std.ArrayList(Task).init(allocator),
                std.ArrayList(Task).init(allocator),
                std.ArrayList(Task).init(allocator),
                std.ArrayList(Task).init(allocator),
            },
            .next_id = 1,
            .lane_cap = lane_capacity,
        };
    }

    pub fn deinit(self: *WorkQueue) void {
        for (&self.lanes) |*lane| {
            for (lane.items) |t| {
                self.allocator.free(t.label);
                self.allocator.free(t.payload);
            }
            lane.deinit();
        }
    }

    /// Enqueue `task` in the lane matching its priority.  Returns the task ID.
    pub fn enqueue(
        self: *WorkQueue,
        label: []const u8,
        payload: []const u8,
        priority: TaskPriority,
        owner_pid: ?u64,
    ) !u64 {
        const idx = @intFromEnum(priority);
        if (self.lanes[idx].items.len >= self.lane_cap) return ProcessError.QueueFull;

        const id = self.next_id;
        self.next_id += 1;

        const label_copy = try self.allocator.dupe(u8, label);
        errdefer self.allocator.free(label_copy);
        const payload_copy = try self.allocator.dupe(u8, payload);
        errdefer self.allocator.free(payload_copy);

        try self.lanes[idx].append(.{
            .id = id,
            .label = label_copy,
            .priority = priority,
            .payload = payload_copy,
            .enqueued_ms = std.time.milliTimestamp(),
            .owner_pid = owner_pid,
        });
        return id;
    }

    /// Dequeue the highest-priority available task, or null if empty.
    /// Ownership of `label` and `payload` transfers to the caller.
    pub fn dequeue(self: *WorkQueue) ?Task {
        // Scan from urgent → low.
        var idx: usize = 3;
        while (true) {
            if (self.lanes[idx].items.len > 0) {
                return self.lanes[idx].orderedRemove(0);
            }
            if (idx == 0) break;
            idx -= 1;
        }
        return null;
    }

    /// Total tasks across all lanes.
    pub fn totalLen(self: *const WorkQueue) usize {
        var total: usize = 0;
        for (&self.lanes) |*lane| total += lane.items.len;
        return total;
    }

    /// Free a Task whose ownership was transferred by `dequeue`.
    pub fn freeTask(self: *WorkQueue, t: Task) void {
        self.allocator.free(t.label);
        self.allocator.free(t.payload);
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// 3.  Thread Pool
// ─────────────────────────────────────────────────────────────────────────────

pub const WorkerState = enum {
    idle,
    busy,
    draining, // finishing current task then going offline
    offline,
};

/// Represents a single logical worker thread slot.
pub const Worker = struct {
    id: u32,
    state: WorkerState,
    current_pid: ?u64, // PID of the process being executed, if busy
    tasks_done: u64,
    started_at_ms: i64,
    last_active_ms: i64,
};

/// Fixed-size pool of worker slots.  Workers are logical (no OS threads are
/// created) so the pool is safe to use in single-threaded simulated kernels.
pub const ThreadPool = struct {
    allocator: std.mem.Allocator,
    workers: []Worker,
    capacity: u32,

    pub fn init(allocator: std.mem.Allocator, capacity: u32) !ThreadPool {
        const workers = try allocator.alloc(Worker, capacity);
        const now = std.time.milliTimestamp();
        for (workers, 0..) |*w, i| {
            w.* = .{
                .id = @intCast(i),
                .state = .idle,
                .current_pid = null,
                .tasks_done = 0,
                .started_at_ms = now,
                .last_active_ms = now,
            };
        }
        return .{
            .allocator = allocator,
            .workers = workers,
            .capacity = capacity,
        };
    }

    pub fn deinit(self: *ThreadPool) void {
        self.allocator.free(self.workers);
    }

    /// Assign `pid` to the first idle worker.  Returns the worker ID.
    pub fn dispatch(self: *ThreadPool, pid: u64) !u32 {
        for (self.workers) |*w| {
            if (w.state == .idle) {
                w.state = .busy;
                w.current_pid = pid;
                w.last_active_ms = std.time.milliTimestamp();
                return w.id;
            }
        }
        return ProcessError.ThreadPoolExhausted;
    }

    /// Mark worker `id` as having completed its task.
    pub fn complete(self: *ThreadPool, worker_id: u32) !void {
        if (worker_id >= self.capacity) return ProcessError.WorkerNotFound;
        const w = &self.workers[worker_id];
        w.tasks_done += 1;
        w.current_pid = null;
        w.state = if (w.state == .draining) .offline else .idle;
        w.last_active_ms = std.time.milliTimestamp();
    }

    /// Begin graceful drain of worker `id` — it finishes its current task
    /// and then goes offline.
    pub fn drain(self: *ThreadPool, worker_id: u32) !void {
        if (worker_id >= self.capacity) return ProcessError.WorkerNotFound;
        const w = &self.workers[worker_id];
        if (w.state == .busy) {
            w.state = .draining;
        } else {
            w.state = .offline;
        }
    }

    /// Bring an offline worker back online.
    pub fn wakeup(self: *ThreadPool, worker_id: u32) !void {
        if (worker_id >= self.capacity) return ProcessError.WorkerNotFound;
        const w = &self.workers[worker_id];
        if (w.state != .offline) return ProcessError.InvalidStateTransition;
        w.state = .idle;
    }

    pub fn idleCount(self: *const ThreadPool) usize {
        var n: usize = 0;
        for (self.workers) |w| {
            if (w.state == .idle) n += 1;
        }
        return n;
    }

    pub fn busyCount(self: *const ThreadPool) usize {
        var n: usize = 0;
        for (self.workers) |w| {
            if (w.state == .busy or w.state == .draining) n += 1;
        }
        return n;
    }

    /// Pump: dequeue one task from `queue`, dispatch it to an idle worker,
    /// transition `pid` in `table` to running, and return the worker ID.
    /// Returns null when there are no tasks or no idle workers.
    pub fn pump(
        self: *ThreadPool,
        queue: *WorkQueue,
        table: *ProcessTable,
    ) !?u32 {
        if (self.idleCount() == 0) return null;

        const task = queue.dequeue() orelse return null;
        defer queue.freeTask(task);

        const pid = task.owner_pid orelse return null;
        const worker_id = try self.dispatch(pid);

        // Best-effort state transition; process may already be running.
        table.transition(.root, pid, .running) catch {};

        return worker_id;
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// 4.  Scheduler  (tick-driven, one-shot + recurring)
// ─────────────────────────────────────────────────────────────────────────────

/// A scheduled job entry (superset of kernel.zig ScheduleEntry).
pub const ScheduleEntry = struct {
    id: []const u8, // owned by Scheduler
    label: []const u8, // owned by Scheduler
    payload: []const u8, // owned by Scheduler
    due_at_ms: i64,
    interval_ms: ?u64, // null = one-shot
    priority: TaskPriority,
    enabled: bool,
    run_count: u64,
    owner_pid: ?u64,
};

/// Tick-driven scheduler that fires jobs into a WorkQueue.
pub const Scheduler = struct {
    allocator: std.mem.Allocator,
    entries: std.ArrayList(ScheduleEntry),
    next_id: u64,

    pub fn init(allocator: std.mem.Allocator) Scheduler {
        return .{
            .allocator = allocator,
            .entries = std.ArrayList(ScheduleEntry).init(allocator),
            .next_id = 1,
        };
    }

    pub fn deinit(self: *Scheduler) void {
        for (self.entries.items) |e| {
            self.allocator.free(e.id);
            self.allocator.free(e.label);
            self.allocator.free(e.payload);
        }
        self.entries.deinit();
    }

    /// Register a new scheduled job.  Returns the entry ID string.
    pub fn schedule(
        self: *Scheduler,
        actor: Role,
        label: []const u8,
        payload: []const u8,
        due_at_ms: i64,
        interval_ms: ?u64,
        priority: TaskPriority,
        owner_pid: ?u64,
    ) ![]const u8 {
        if (!hasPermission(actor, .schedule_tasks)) return ProcessError.AccessDenied;

        const raw_id = self.next_id;
        self.next_id += 1;

        const id_str = try std.fmt.allocPrint(self.allocator, "sched-{d}", .{raw_id});
        errdefer self.allocator.free(id_str);
        const label_copy = try self.allocator.dupe(u8, label);
        errdefer self.allocator.free(label_copy);
        const payload_copy = try self.allocator.dupe(u8, payload);
        errdefer self.allocator.free(payload_copy);

        try self.entries.append(.{
            .id = id_str,
            .label = label_copy,
            .payload = payload_copy,
            .due_at_ms = due_at_ms,
            .interval_ms = interval_ms,
            .priority = priority,
            .enabled = true,
            .run_count = 0,
            .owner_pid = owner_pid,
        });
        return id_str;
    }

    /// Enable or disable a scheduled entry by ID.
    pub fn setEnabled(self: *Scheduler, actor: Role, id: []const u8, enabled: bool) !void {
        if (!hasPermission(actor, .schedule_tasks)) return ProcessError.AccessDenied;
        for (self.entries.items) |*e| {
            if (std.mem.eql(u8, e.id, id)) {
                e.enabled = enabled;
                return;
            }
        }
        return ProcessError.ScheduleEntryNotFound;
    }

    /// Cancel (remove) an entry by ID.
    pub fn cancel(self: *Scheduler, actor: Role, id: []const u8) !void {
        if (!hasPermission(actor, .schedule_tasks)) return ProcessError.AccessDenied;
        for (self.entries.items, 0..) |e, i| {
            if (std.mem.eql(u8, e.id, id)) {
                self.allocator.free(e.id);
                self.allocator.free(e.label);
                self.allocator.free(e.payload);
                _ = self.entries.swapRemove(i);
                return;
            }
        }
        return ProcessError.ScheduleEntryNotFound;
    }

    /// Advance the clock to `now_ms`.  All due entries fire into `queue`.
    /// Returns the number of tasks enqueued.
    pub fn tick(self: *Scheduler, actor: Role, now_ms: i64, queue: *WorkQueue) !u64 {
        if (!hasPermission(actor, .schedule_tasks)) return ProcessError.AccessDenied;
        var fired: u64 = 0;

        for (self.entries.items) |*e| {
            if (!e.enabled or e.due_at_ms > now_ms) continue;

            _ = queue.enqueue(e.label, e.payload, e.priority, e.owner_pid) catch continue;
            e.run_count += 1;
            fired += 1;

            if (e.interval_ms) |interval| {
                e.due_at_ms = now_ms + @as(i64, @intCast(interval));
            } else {
                e.enabled = false;
            }
        }
        return fired;
    }

    pub fn count(self: *const Scheduler) usize {
        return self.entries.items.len;
    }

    pub fn enabledCount(self: *const Scheduler) usize {
        var n: usize = 0;
        for (self.entries.items) |e| {
            if (e.enabled) n += 1;
        }
        return n;
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// 5.  Resource Ledger
// ─────────────────────────────────────────────────────────────────────────────

/// A named pool of discrete resource units (CPU shares, file handles, sockets…).
pub const ResourcePool = struct {
    name: []const u8, // owned by ResourceLedger
    total: u32,
    available: u32,
};

/// Per-tenant acquisition record.
const Acquisition = struct {
    tenant_id: []const u8, // owned by ResourceLedger
    resource: []const u8, // owned by ResourceLedger
    units: u32,
    acquired_ms: i64,
};

/// Central acquire/release registry for named resource pools.
pub const ResourceLedger = struct {
    allocator: std.mem.Allocator,
    pools: std.StringHashMap(ResourcePool),
    acquisitions: std.ArrayList(Acquisition),

    pub fn init(allocator: std.mem.Allocator) ResourceLedger {
        return .{
            .allocator = allocator,
            .pools = std.StringHashMap(ResourcePool).init(allocator),
            .acquisitions = std.ArrayList(Acquisition).init(allocator),
        };
    }

    pub fn deinit(self: *ResourceLedger) void {
        var pit = self.pools.valueIterator();
        while (pit.next()) |p| self.allocator.free(p.name);
        self.pools.deinit();

        for (self.acquisitions.items) |a| {
            self.allocator.free(a.tenant_id);
            self.allocator.free(a.resource);
        }
        self.acquisitions.deinit();
    }

    /// Register a new resource pool with `total` units.
    pub fn registerPool(self: *ResourceLedger, name: []const u8, total: u32) !void {
        if (self.pools.contains(name)) return;
        const name_copy = try self.allocator.dupe(u8, name);
        errdefer self.allocator.free(name_copy);
        try self.pools.put(name_copy, .{
            .name = name_copy,
            .total = total,
            .available = total,
        });
    }

    /// Acquire `units` of `resource` for `tenant_id`.
    pub fn acquire(
        self: *ResourceLedger,
        actor: Role,
        tenant_id: []const u8,
        resource: []const u8,
        units: u32,
    ) !void {
        if (!hasPermission(actor, .manage_processes)) return ProcessError.AccessDenied;

        const pool = self.pools.getPtr(resource) orelse return ProcessError.ResourceNotFound;
        if (pool.available < units) return ProcessError.ResourceLimitExceeded;

        pool.available -= units;

        const tid_copy = try self.allocator.dupe(u8, tenant_id);
        errdefer self.allocator.free(tid_copy);
        const res_copy = try self.allocator.dupe(u8, resource);
        errdefer self.allocator.free(res_copy);

        try self.acquisitions.append(.{
            .tenant_id = tid_copy,
            .resource = res_copy,
            .units = units,
            .acquired_ms = std.time.milliTimestamp(),
        });
    }

    /// Release `units` of `resource` that were previously acquired by `tenant_id`.
    pub fn release(
        self: *ResourceLedger,
        actor: Role,
        tenant_id: []const u8,
        resource: []const u8,
        units: u32,
    ) !void {
        if (!hasPermission(actor, .manage_processes)) return ProcessError.AccessDenied;

        // Find and consume the oldest matching acquisition.
        for (self.acquisitions.items, 0..) |a, i| {
            if (std.mem.eql(u8, a.tenant_id, tenant_id) and
                std.mem.eql(u8, a.resource, resource) and
                a.units == units)
            {
                self.allocator.free(a.tenant_id);
                self.allocator.free(a.resource);
                _ = self.acquisitions.swapRemove(i);

                const pool = self.pools.getPtr(resource) orelse return ProcessError.ResourceNotFound;
                pool.available = @min(pool.available + units, pool.total);
                return;
            }
        }
        return ProcessError.ResourceNotAcquired;
    }

    /// How many units of `resource` are currently available.
    pub fn available(self: *const ResourceLedger, resource: []const u8) ?u32 {
        if (self.pools.get(resource)) |p| return p.available;
        return null;
    }

    /// Total units held by `tenant_id` across all resources.
    pub fn totalHeld(self: *const ResourceLedger, tenant_id: []const u8) u32 {
        var total: u32 = 0;
        for (self.acquisitions.items) |a| {
            if (std.mem.eql(u8, a.tenant_id, tenant_id)) total += a.units;
        }
        return total;
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// 6.  Orchestrator  (supervision groups + coordinated lifecycle)
// ─────────────────────────────────────────────────────────────────────────────

/// Restart strategy applied when a process in a group terminates unexpectedly.
pub const RestartStrategy = enum {
    /// Never restart automatically.
    none,
    /// Restart only the failed process.
    one_for_one,
    /// Restart all processes in the group when any one fails.
    one_for_all,
};

/// A supervision group — a named set of processes managed together.
pub const ProcessGroup = struct {
    id: []const u8, // owned by Orchestrator
    strategy: RestartStrategy,
    max_restarts: u32,
    restart_count: u32,
    created_at_ms: i64,
};

/// High-level coordinator that ties ProcessTable, ThreadPool, Scheduler,
/// WorkQueue, and ResourceLedger into one cohesive lifecycle surface.
pub const Orchestrator = struct {
    allocator: std.mem.Allocator,
    table: *ProcessTable,
    pool: *ThreadPool,
    scheduler: *Scheduler,
    queue: *WorkQueue,
    ledger: *ResourceLedger,
    groups: std.StringHashMap(ProcessGroup),

    pub fn init(
        allocator: std.mem.Allocator,
        table: *ProcessTable,
        pool: *ThreadPool,
        scheduler: *Scheduler,
        queue: *WorkQueue,
        ledger: *ResourceLedger,
    ) Orchestrator {
        return .{
            .allocator = allocator,
            .table = table,
            .pool = pool,
            .scheduler = scheduler,
            .queue = queue,
            .ledger = ledger,
            .groups = std.StringHashMap(ProcessGroup).init(allocator),
        };
    }

    pub fn deinit(self: *Orchestrator) void {
        var it = self.groups.valueIterator();
        while (it.next()) |g| self.allocator.free(g.id);
        self.groups.deinit();
    }

    // ── Group management ──────────────────────────────────────────────────

    /// Register a new supervision group.
    pub fn createGroup(
        self: *Orchestrator,
        actor: Role,
        id: []const u8,
        strategy: RestartStrategy,
        max_restarts: u32,
    ) !void {
        if (!hasPermission(actor, .manage_processes)) return ProcessError.AccessDenied;
        if (self.groups.contains(id)) return ProcessError.GroupAlreadyExists;

        const id_copy = try self.allocator.dupe(u8, id);
        errdefer self.allocator.free(id_copy);
        try self.groups.put(id_copy, .{
            .id = id_copy,
            .strategy = strategy,
            .max_restarts = max_restarts,
            .restart_count = 0,
            .created_at_ms = std.time.milliTimestamp(),
        });
    }

    /// Spawn a process inside a supervision group.
    pub fn spawnInGroup(
        self: *Orchestrator,
        actor: Role,
        group_id: []const u8,
        name: []const u8,
        owner: Role,
        priority: u8,
    ) !u64 {
        if (!self.groups.contains(group_id)) return ProcessError.GroupNotFound;
        return self.table.spawn(actor, name, owner, priority, group_id);
    }

    /// Terminate all processes in a group and record their final state.
    pub fn terminateGroup(self: *Orchestrator, actor: Role, group_id: []const u8) !void {
        if (!hasPermission(actor, .manage_processes)) return ProcessError.AccessDenied;
        if (!self.groups.contains(group_id)) return ProcessError.GroupNotFound;

        var pids = std.ArrayList(u64).init(self.allocator);
        defer pids.deinit();

        try self.table.pidsByGroup(group_id, &pids);
        for (pids.items) |pid| {
            self.table.transition(actor, pid, .terminated) catch {};
        }
    }

    /// Handle a process reaching .terminated state according to group strategy.
    /// Returns the new PID if a restart occurred, null otherwise.
    pub fn handleTermination(
        self: *Orchestrator,
        actor: Role,
        pid: u64,
    ) !?u64 {
        const rec = self.table.get(pid) orelse return ProcessError.ProcessNotFound;
        const gid = rec.group_id orelse return null;

        const group = self.groups.getPtr(gid) orelse return null;

        if (group.restart_count >= group.max_restarts) return null;

        switch (group.strategy) {
            .none => return null,

            .one_for_one => {
                group.restart_count += 1;
                const new_pid = try self.table.spawn(
                    actor,
                    rec.name,
                    rec.owner,
                    rec.priority,
                    gid,
                );
                return new_pid;
            },

            .one_for_all => {
                group.restart_count += 1;
                // Collect siblings.
                var siblings = std.ArrayList(u64).init(self.allocator);
                defer siblings.deinit();
                try self.table.pidsByGroup(gid, &siblings);

                for (siblings.items) |spid| {
                    if (spid == pid) continue;
                    self.table.transition(actor, spid, .terminated) catch {};
                }
                // Re-spawn only the triggering process for simplicity;
                // callers may re-spawn all siblings from their own manifest.
                const new_pid = try self.table.spawn(
                    actor,
                    rec.name,
                    rec.owner,
                    rec.priority,
                    gid,
                );
                return new_pid;
            },
        }
    }

    // ── Coordinated tick ─────────────────────────────────────────────────

    /// Master tick: advance scheduler, pump ready work into the thread pool.
    /// Returns { tasks_fired, tasks_dispatched }.
    pub fn tick(self: *Orchestrator, actor: Role, now_ms: i64) !struct { fired: u64, dispatched: u64 } {
        if (!hasPermission(actor, .schedule_tasks)) return ProcessError.AccessDenied;

        const fired = try self.scheduler.tick(actor, now_ms, self.queue);

        var dispatched: u64 = 0;
        while (true) {
            const worker = try self.pool.pump(self.queue, self.table);
            if (worker == null) break;
            dispatched += 1;
        }

        return .{ .fired = fired, .dispatched = dispatched };
    }

    // ── Stats ─────────────────────────────────────────────────────────────

    pub const Stats = struct {
        live_processes: usize,
        idle_workers: usize,
        busy_workers: usize,
        queued_tasks: usize,
        scheduled_jobs: usize,
        group_count: usize,
    };

    pub fn stats(self: *const Orchestrator) Stats {
        return .{
            .live_processes = self.table.liveCount(),
            .idle_workers = self.pool.idleCount(),
            .busy_workers = self.pool.busyCount(),
            .queued_tasks = self.queue.totalLen(),
            .scheduled_jobs = self.scheduler.enabledCount(),
            .group_count = self.groups.count(),
        };
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

test "process table: spawn, transition, cpu charge" {
    var table = ProcessTable.init(std.testing.allocator);
    defer table.deinit();

    const pid = try table.spawn(.host, "worker-a", .module_runtime, 128, null);
    try std.testing.expect(pid >= 1000);
    try std.testing.expectEqual(@as(usize, 1), table.liveCount());

    try table.transition(.host, pid, .running);
    try std.testing.expectEqual(ProcessState.running, table.get(pid).?.state);

    try table.chargeCpu(pid, 50);
    try std.testing.expectEqual(@as(u64, 50), table.get(pid).?.cpu_time_ms);

    try table.transition(.host, pid, .terminated);
    try std.testing.expectEqual(@as(usize, 0), table.liveCount());
}

test "process table: illegal state transition rejected" {
    var table = ProcessTable.init(std.testing.allocator);
    defer table.deinit();

    const pid = try table.spawn(.host, "worker-b", .module_runtime, 64, null);
    // ready → waiting is illegal
    try std.testing.expectError(
        ProcessError.InvalidStateTransition,
        table.transition(.host, pid, .waiting),
    );
}

test "process table: group membership" {
    var table = ProcessTable.init(std.testing.allocator);
    defer table.deinit();

    _ = try table.spawn(.host, "alpha", .server, 100, "grp-1");
    _ = try table.spawn(.host, "beta", .server, 100, "grp-1");
    _ = try table.spawn(.host, "gamma", .server, 100, "grp-2");

    var pids = std.ArrayList(u64).init(std.testing.allocator);
    defer pids.deinit();
    try table.pidsByGroup("grp-1", &pids);
    try std.testing.expectEqual(@as(usize, 2), pids.items.len);
}

test "work queue: priority ordering" {
    var q = WorkQueue.init(std.testing.allocator, 32);
    defer q.deinit();

    _ = try q.enqueue("low-task", "l", .low, null);
    _ = try q.enqueue("urgent-task", "u", .urgent, null);
    _ = try q.enqueue("normal-task", "n", .normal, null);

    const first = q.dequeue().?;
    defer q.freeTask(first);
    try std.testing.expectEqual(TaskPriority.urgent, first.priority);

    const second = q.dequeue().?;
    defer q.freeTask(second);
    try std.testing.expectEqual(TaskPriority.normal, second.priority);
}

test "work queue: lane capacity enforced" {
    var q = WorkQueue.init(std.testing.allocator, 2);
    defer q.deinit();

    _ = try q.enqueue("a", "x", .normal, null);
    _ = try q.enqueue("b", "x", .normal, null);
    try std.testing.expectError(
        ProcessError.QueueFull,
        q.enqueue("c", "x", .normal, null),
    );
}

test "thread pool: dispatch and complete" {
    var pool = try ThreadPool.init(std.testing.allocator, 4);
    defer pool.deinit();

    try std.testing.expectEqual(@as(usize, 4), pool.idleCount());
    const wid = try pool.dispatch(1001);
    try std.testing.expectEqual(@as(usize, 1), pool.busyCount());

    try pool.complete(wid);
    try std.testing.expectEqual(@as(usize, 4), pool.idleCount());
    try std.testing.expectEqual(@as(u64, 1), pool.workers[wid].tasks_done);
}

test "thread pool: exhaustion" {
    var pool = try ThreadPool.init(std.testing.allocator, 2);
    defer pool.deinit();

    _ = try pool.dispatch(1001);
    _ = try pool.dispatch(1002);
    try std.testing.expectError(ProcessError.ThreadPoolExhausted, pool.dispatch(1003));
}

test "thread pool: drain and wakeup" {
    var pool = try ThreadPool.init(std.testing.allocator, 2);
    defer pool.deinit();

    const wid = try pool.dispatch(1001);
    try pool.drain(wid);
    try std.testing.expectEqual(WorkerState.draining, pool.workers[wid].state);

    try pool.complete(wid);
    try std.testing.expectEqual(WorkerState.offline, pool.workers[wid].state);

    try pool.wakeup(wid);
    try std.testing.expectEqual(WorkerState.idle, pool.workers[wid].state);
}

test "scheduler: one-shot fires once" {
    var sched = Scheduler.init(std.testing.allocator);
    defer sched.deinit();
    var q = WorkQueue.init(std.testing.allocator, 64);
    defer q.deinit();

    const now: i64 = 1000;
    _ = try sched.schedule(.host, "job-a", "{}", now, null, .normal, null);

    const fired1 = try sched.tick(.host, now, &q);
    try std.testing.expectEqual(@as(u64, 1), fired1);
    try std.testing.expectEqual(@as(usize, 1), q.totalLen());

    // A second tick at the same time should not re-fire a one-shot.
    const fired2 = try sched.tick(.host, now + 1, &q);
    try std.testing.expectEqual(@as(u64, 0), fired2);
}

test "scheduler: recurring fires on each interval" {
    var sched = Scheduler.init(std.testing.allocator);
    defer sched.deinit();
    var q = WorkQueue.init(std.testing.allocator, 64);
    defer q.deinit();

    const start: i64 = 0;
    _ = try sched.schedule(.host, "heartbeat", "{}", start, 500, .low, null);

    _ = try sched.tick(.host, 0, &q);
    _ = try sched.tick(.host, 500, &q);
    _ = try sched.tick(.host, 1000, &q);

    // 3 ticks at t=0, 500, 1000 should have enqueued 3 tasks.
    try std.testing.expectEqual(@as(usize, 3), q.totalLen());
}

test "scheduler: cancel removes entry" {
    var sched = Scheduler.init(std.testing.allocator);
    defer sched.deinit();
    var q = WorkQueue.init(std.testing.allocator, 64);
    defer q.deinit();

    const id = try sched.schedule(.host, "temp", "{}", 0, 1000, .normal, null);
    try std.testing.expectEqual(@as(usize, 1), sched.count());

    try sched.cancel(.host, id);
    try std.testing.expectEqual(@as(usize, 0), sched.count());
}

test "resource ledger: acquire and release" {
    var ledger = ResourceLedger.init(std.testing.allocator);
    defer ledger.deinit();

    try ledger.registerPool("cpu-shares", 100);
    try ledger.acquire(.host, "tenant-a", "cpu-shares", 40);
    try std.testing.expectEqual(@as(u32, 60), ledger.available("cpu-shares").?);
    try std.testing.expectEqual(@as(u32, 40), ledger.totalHeld("tenant-a"));

    try ledger.release(.host, "tenant-a", "cpu-shares", 40);
    try std.testing.expectEqual(@as(u32, 100), ledger.available("cpu-shares").?);
}

test "resource ledger: over-acquire rejected" {
    var ledger = ResourceLedger.init(std.testing.allocator);
    defer ledger.deinit();

    try ledger.registerPool("sockets", 10);
    try ledger.acquire(.host, "mod-x", "sockets", 8);
    try std.testing.expectError(
        ProcessError.ResourceLimitExceeded,
        ledger.acquire(.host, "mod-x", "sockets", 5),
    );
}

test "orchestrator: full lifecycle tick" {
    // Set up subsystems.
    var table = ProcessTable.init(std.testing.allocator);
    defer table.deinit();
    var pool = try ThreadPool.init(std.testing.allocator, 4);
    defer pool.deinit();
    var sched = Scheduler.init(std.testing.allocator);
    defer sched.deinit();
    var q = WorkQueue.init(std.testing.allocator, 64);
    defer q.deinit();
    var ledger = ResourceLedger.init(std.testing.allocator);
    defer ledger.deinit();

    var orch = Orchestrator.init(
        std.testing.allocator,
        &table,
        &pool,
        &sched,
        &q,
        &ledger,
    );
    defer orch.deinit();

    // Create a group and spawn two workers.
    try orch.createGroup(.host, "workers", .one_for_one, 3);
    const pid1 = try orch.spawnInGroup(.host, "workers", "w1", .module_runtime, 128);
    const pid2 = try orch.spawnInGroup(.host, "workers", "w2", .module_runtime, 128);

    // Schedule a job that fires immediately and points at pid1.
    _ = try sched.schedule(.host, "ping", "{}", 0, null, .normal, pid1);
    _ = try sched.schedule(.host, "ping", "{}", 0, null, .normal, pid2);

    // Advance: jobs fire → tasks enter queue → workers pick them up.
    const result = try orch.tick(.host, 0);
    try std.testing.expectEqual(@as(u64, 2), result.fired);
    try std.testing.expect(result.dispatched > 0);

    const s = orch.stats();
    try std.testing.expectEqual(@as(usize, 2), s.live_processes);
    try std.testing.expectEqual(@as(usize, 1), s.group_count);
}

test "orchestrator: one_for_one restart" {
    var table = ProcessTable.init(std.testing.allocator);
    defer table.deinit();
    var pool = try ThreadPool.init(std.testing.allocator, 2);
    defer pool.deinit();
    var sched = Scheduler.init(std.testing.allocator);
    defer sched.deinit();
    var q = WorkQueue.init(std.testing.allocator, 16);
    defer q.deinit();
    var ledger = ResourceLedger.init(std.testing.allocator);
    defer ledger.deinit();

    var orch = Orchestrator.init(
        std.testing.allocator,
        &table,
        &pool,
        &sched,
        &q,
        &ledger,
    );
    defer orch.deinit();

    try orch.createGroup(.host, "svc", .one_for_one, 5);
    const pid = try orch.spawnInGroup(.host, "svc", "service", .server, 200);

    try table.transition(.host, pid, .running);
    try table.transition(.host, pid, .terminated);

    const new_pid = try orch.handleTermination(.host, pid);
    try std.testing.expect(new_pid != null);
    try std.testing.expect(new_pid.? != pid);
    try std.testing.expectEqual(@as(usize, 2), table.liveCount()); // old + restarted
}

test "orchestrator: terminate group kills all members" {
    var table = ProcessTable.init(std.testing.allocator);
    defer table.deinit();
    var pool = try ThreadPool.init(std.testing.allocator, 4);
    defer pool.deinit();
    var sched = Scheduler.init(std.testing.allocator);
    defer sched.deinit();
    var q = WorkQueue.init(std.testing.allocator, 16);
    defer q.deinit();
    var ledger = ResourceLedger.init(std.testing.allocator);
    defer ledger.deinit();

    var orch = Orchestrator.init(
        std.testing.allocator,
        &table,
        &pool,
        &sched,
        &q,
        &ledger,
    );
    defer orch.deinit();

    try orch.createGroup(.host, "batch", .none, 0);
    _ = try orch.spawnInGroup(.host, "batch", "job-1", .server, 100);
    _ = try orch.spawnInGroup(.host, "batch", "job-2", .server, 100);
    _ = try orch.spawnInGroup(.host, "batch", "job-3", .server, 100);

    try std.testing.expectEqual(@as(usize, 3), table.liveCount());
    try orch.terminateGroup(.host, "batch");
    try std.testing.expectEqual(@as(usize, 0), table.liveCount());
}
