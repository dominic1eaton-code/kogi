//! KOGI Observability System - Debugging, Monitoring, Performance, Optimization
//! Provides runtime debugging hooks, system health monitoring, performance profiling,
//! and optimization guidance.

const std = @import("std");

/// Breakpoint types
pub const BreakpointType = enum {
    line,
    function,
    condition,
};

/// Breakpoint structure
pub const Breakpoint = struct {
    id: u64,
    bp_type: BreakpointType,
    file: []const u8,
    line: u32,
    enabled: bool,
    condition: ?[]const u8,
};

/// System metric snapshot
pub const SystemMetrics = struct {
    timestamp: i64,
    cpu_usage_percent: f32,
    memory_usage_mb: u32,
    thread_count: u32,
    io_operations: u64,
};

/// Performance profile for functions
pub const PerformanceProfile = struct {
    function_name: []const u8,
    call_count: u64,
    total_time_us: u64,
};

/// Optimization hint
pub const OptimizationHint = struct {
    hint_id: u64,
    timestamp: i64,
    message: []const u8,
};

/// Observability manager
pub const ObservabilityManager = struct {
    allocator: std.mem.Allocator,
    breakpoints: std.ArrayList(Breakpoint),
    metrics_history: std.ArrayList(SystemMetrics),
    profiles: std.ArrayList(PerformanceProfile),
    hints: std.ArrayList(OptimizationHint),
    next_bp_id: u64,
    next_hint_id: u64,
    mutex: std.Thread.Mutex = .{},

    pub fn init(allocator: std.mem.Allocator) ObservabilityManager {
        return ObservabilityManager{
            .allocator = allocator,
            .breakpoints = std.ArrayList(Breakpoint){},
            .metrics_history = std.ArrayList(SystemMetrics){},
            .profiles = std.ArrayList(PerformanceProfile){},
            .hints = std.ArrayList(OptimizationHint){},
            .next_bp_id = 1,
            .next_hint_id = 1,
        };
    }

    pub fn deinit(self: *ObservabilityManager) void {
        for (self.breakpoints.items) |bp| {
            self.allocator.free(bp.file);
            if (bp.condition) |cond| self.allocator.free(cond);
        }
        self.breakpoints.deinit(self.allocator);

        // metrics_history elements contain primitive fields only; nothing to free
        for (self.metrics_history.items) |_| {}
        self.metrics_history.deinit(self.allocator);

        for (self.profiles.items) |p| {
            self.allocator.free(p.function_name);
        }
        self.profiles.deinit(self.allocator);

        for (self.hints.items) |h| {
            self.allocator.free(h.message);
        }
        self.hints.deinit(self.allocator);
    }

    /// Add a breakpoint
    pub fn addBreakpoint(
        self: *ObservabilityManager,
        bp_type: BreakpointType,
        file: []const u8,
        line: u32,
        condition: ?[]const u8,
    ) !u64 {
        self.mutex.lock();
        defer self.mutex.unlock();

        const id = self.next_bp_id;
        self.next_bp_id += 1;

        const bp = Breakpoint{
            .id = id,
            .bp_type = bp_type,
            .file = try self.allocator.dupe(u8, file),
            .line = line,
            .enabled = true,
            .condition = if (condition) |c| try self.allocator.dupe(u8, c) else null,
        };
        try self.breakpoints.append(self.allocator, bp);
        return id;
    }

    pub fn removeBreakpoint(self: *ObservabilityManager, id: u64) void {
        self.mutex.lock();
        defer self.mutex.unlock();
        for (self.breakpoints.items, 0..) |bp, idx| {
            if (bp.id == id) {
                const removed = self.breakpoints.orderedRemove(idx);
                self.allocator.free(removed.file);
                if (removed.condition) |c| self.allocator.free(c);
                return;
            }
        }
    }

    pub fn listBreakpoints(self: *ObservabilityManager) []Breakpoint {
        return self.breakpoints.items;
    }

    /// Record a system metric snapshot
    pub fn recordMetrics(
        self: *ObservabilityManager,
        cpu: f32,
        memory_mb: u32,
        thread_count: u32,
        io_ops: u64,
    ) !void {
        self.mutex.lock();
        defer self.mutex.unlock();

        const m = SystemMetrics{
            .timestamp = std.time.timestamp(),
            .cpu_usage_percent = cpu,
            .memory_usage_mb = memory_mb,
            .thread_count = thread_count,
            .io_operations = io_ops,
        };
        try self.metrics_history.append(self.allocator, m);
    }

    pub fn getMetricsHistory(self: *ObservabilityManager) []SystemMetrics {
        return self.metrics_history.items;
    }

    /// Record a function profile entry
    pub fn recordProfile(
        self: *ObservabilityManager,
        function_name: []const u8,
        duration_us: u64,
    ) !void {
        self.mutex.lock();
        defer self.mutex.unlock();

        // find existing profile
        for (self.profiles.items, 0..) |p, idx| {
            if (std.mem.eql(u8, p.function_name, function_name)) {
                self.profiles.items[idx].call_count += 1;
                self.profiles.items[idx].total_time_us += duration_us;
                return;
            }
        }
        const prof = PerformanceProfile{
            .function_name = try self.allocator.dupe(u8, function_name),
            .call_count = 1,
            .total_time_us = duration_us,
        };
        try self.profiles.append(self.allocator, prof);
    }

    pub fn getProfiles(self: *ObservabilityManager) []PerformanceProfile {
        return self.profiles.items;
    }

    /// Add an optimization hint
    pub fn addHint(self: *ObservabilityManager, message: []const u8) !u64 {
        self.mutex.lock();
        defer self.mutex.unlock();

        const id = self.next_hint_id;
        self.next_hint_id += 1;

        const hint = OptimizationHint{
            .hint_id = id,
            .timestamp = std.time.timestamp(),
            .message = try self.allocator.dupe(u8, message),
        };
        try self.hints.append(self.allocator, hint);
        return id;
    }

    pub fn getHints(self: *ObservabilityManager) []OptimizationHint {
        return self.hints.items;
    }
};
