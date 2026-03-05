//! KOGI Process Management System - Process lifecycle and scheduling
//! Provides complete process management with scheduling, priorities, and resource allocation

const std = @import("std");

/// Simple CPU context placeholder for context switching
pub const CPUContext = struct {
    registers: [16]usize,
    stack_ptr: ?*u8,
    pc: ?usize,

    pub fn init() CPUContext {
        return CPUContext{ .registers = undefined, .stack_ptr = null, .pc = null };
    }
};

/// Save the current CPU context (stub implementation)
fn saveCurrentContext() CPUContext {
    // In a real OS this would capture registers/pc/sp
    return CPUContext.init();
}

/// Restore a CPU context (stub implementation)
fn restoreContext(ctx: CPUContext) void {
    // In a real OS this would load registers/pc/sp
    _ = ctx;
}

/// Process state
pub const ProcessState = enum {
    created,
    ready,
    running,
    suspended,
    waiting,
    terminated,
};

/// Process priority levels
pub const ProcessPriority = enum {
    idle,
    low,
    normal,
    high,
    critical,

    pub fn value(self: ProcessPriority) u8 {
        return switch (self) {
            .idle => 0,
            .low => 1,
            .normal => 2,
            .high => 3,
            .critical => 4,
        };
    }
};

/// Process resource allocation
pub const ProcessResources = struct {
    max_memory_mb: u32 = 256,
    max_cpu_percent: f32 = 100.0,
    max_threads: u32 = 8,
    max_open_files: u32 = 1024,
    io_priority: u8 = 5,
};

/// Process context
pub const Process = struct {
    id: u32,
    parent_id: ?u32,
    name: []const u8,
    state: ProcessState,
    priority: ProcessPriority,
    created_at: i64,
    started_at: ?i64 = null,
    terminated_at: ?i64 = null,
    exit_code: ?i32 = null,
    resources: ProcessResources,
    current_memory_mb: u32 = 0,
    current_cpu_percent: f32 = 0.0,
    thread_count: u32 = 0,
    open_file_count: u32 = 0,
    metadata: std.StringHashMap([]const u8),
    // CPU context (registers, stack pointer etc) used for context switching
    context: CPUContext,
};

/// Process lifecycle event
pub const ProcessEvent = struct {
    event_id: u32,
    process_id: u32,
    event_type: ProcessEventType,
    timestamp: i64,
    details: []const u8,
};

/// Process event types
pub const ProcessEventType = enum {
    created,
    started,
    suspended,
    resumed,
    blocked,
    unblocked,
    context_switched,
    resource_exceeded,
    terminated,
    @"error",
};

/// Process scheduler configuration
pub const SchedulerConfig = struct {
    time_slice_ms: u32 = 10,
    preemptive: bool = true,
    priority_based: bool = true,
    max_processes: usize = 10000,
};

/// Process manager
pub const ProcessManager = struct {
    allocator: std.mem.Allocator,
    processes: std.array_list.Managed(Process),
    process_events: std.array_list.Managed(ProcessEvent),
    next_process_id: u32 = 1,
    next_event_id: u32 = 0,
    running_process_id: ?u32 = null,
    scheduler_config: SchedulerConfig,
    mutex: std.Thread.Mutex = .{},

    pub fn init(allocator: std.mem.Allocator, config: SchedulerConfig) ProcessManager {
        return ProcessManager{
            .allocator = allocator,
            .processes = std.array_list.Managed(Process).init(allocator),
            .process_events = std.array_list.Managed(ProcessEvent).init(allocator),
            .scheduler_config = config,
        };
    }

    pub fn deinit(self: *ProcessManager) void {
        for (self.processes.items) |proc| {
            self.allocator.free(proc.name);
            var metadata = proc.metadata;
            var iter = metadata.iterator();
            while (iter.next()) |kv| {
                self.allocator.free(kv.key_ptr.*);
                self.allocator.free(kv.value_ptr.*);
            }
            metadata.deinit();
        }
        self.processes.deinit();

        for (self.process_events.items) |event| {
            self.allocator.free(event.details);
        }
        self.process_events.deinit();
    }

    /// Create a new process
    pub fn createProcess(
        self: *ProcessManager,
        name: []const u8,
        priority: ProcessPriority,
        resources: ProcessResources,
    ) !u32 {
        self.mutex.lock();
        defer self.mutex.unlock();

        if (self.processes.items.len >= self.scheduler_config.max_processes) {
            return error.MaxProcessesReached;
        }

        const process_id = self.next_process_id;
        self.next_process_id += 1;

        const process = Process{
            .id = process_id,
            .parent_id = self.running_process_id,
            .name = try self.allocator.dupe(u8, name),
            .state = .created,
            .priority = priority,
            .created_at = std.time.timestamp(),
            .resources = resources,
            .metadata = std.StringHashMap([]const u8).init(self.allocator),
            .context = CPUContext.init(),
        };

        try self.processes.append(process);
        try self.logProcessEvent(process_id, .created, "Process created");

        return process_id;
    }

    /// Start a process
    pub fn startProcess(self: *ProcessManager, process_id: u32) !void {
        self.mutex.lock();
        defer self.mutex.unlock();

        for (self.processes.items) |*proc| {
            if (proc.id == process_id) {
                if (proc.state == .created or proc.state == .suspended) {
                    proc.state = .ready;
                    proc.started_at = std.time.timestamp();
                    try self.logProcessEvent(process_id, .started, "Process started");
                }
                return;
            }
        }
        return error.ProcessNotFound;
    }

    /// Suspend a process
    pub fn suspendProcess(self: *ProcessManager, process_id: u32) !void {
        self.mutex.lock();
        defer self.mutex.unlock();

        for (self.processes.items) |*proc| {
            if (proc.id == process_id) {
                if (proc.state == .running or proc.state == .ready) {
                    proc.state = .suspended;
                    try self.logProcessEvent(process_id, .suspended, "Process suspended");
                }
                return;
            }
        }
        return error.ProcessNotFound;
    }

    /// Resume a process
    pub fn resumeProcess(self: *ProcessManager, process_id: u32) !void {
        self.mutex.lock();
        defer self.mutex.unlock();

        for (self.processes.items) |*proc| {
            if (proc.id == process_id) {
                if (proc.state == .suspended) {
                    proc.state = .ready;
                    try self.logProcessEvent(process_id, .resumed, "Process resumed");
                }
                return;
            }
        }
        return error.ProcessNotFound;
    }

    /// Perform a context switch between two processes
    pub fn contextSwitch(self: *ProcessManager, from_id: u32, to_id: u32) !void {
        self.mutex.lock();
        defer self.mutex.unlock();

        var from_proc: ?*Process = null;
        var to_proc: ?*Process = null;
        for (self.processes.items) |*p| {
            if (p.id == from_id) from_proc = p;
            if (p.id == to_id) to_proc = p;
        }
        if (from_proc == null or to_proc == null) return error.ProcessNotFound;

        // save current process context and set to ready
        from_proc.*.state = .ready;
        from_proc.*.context = saveCurrentContext();

        // switch in new process
        to_proc.*.state = .running;
        try self.logProcessEvent(from_id, .context_switched, "switched out");
        try self.logProcessEvent(to_id, .context_switched, "switched in");
        restoreContext(to_proc.*.context);
        self.running_process_id = to_id;
    }

    /// Terminate a process
    pub fn terminateProcess(self: *ProcessManager, process_id: u32, exit_code: i32) !void {
        self.mutex.lock();
        defer self.mutex.unlock();

        for (self.processes.items) |*proc| {
            if (proc.id == process_id) {
                proc.state = .terminated;
                proc.exit_code = exit_code;
                proc.terminated_at = std.time.timestamp();
                try self.logProcessEvent(process_id, .terminated, "Process terminated");
                return;
            }
        }
        return error.ProcessNotFound;
    }

    /// Get process by ID
    pub fn getProcess(self: *ProcessManager, process_id: u32) ?Process {
        for (self.processes.items) |proc| {
            if (proc.id == process_id) {
                return proc;
            }
        }
        return null;
    }

    /// Get all running processes
    pub fn getRunningProcesses(self: *ProcessManager, allocator: std.mem.Allocator) !std.array_list.Managed(Process) {
        var running = std.array_list.Managed(Process).init(allocator);
        for (self.processes.items) |proc| {
            if (proc.state == .running or proc.state == .ready or proc.state == .waiting) {
                try running.append(proc);
            }
        }
        return running;
    }

    /// Update process resources
    pub fn updateProcessResources(
        self: *ProcessManager,
        process_id: u32,
        memory_mb: u32,
        cpu_percent: f32,
    ) !void {
        self.mutex.lock();
        defer self.mutex.unlock();

        for (self.processes.items) |*proc| {
            if (proc.id == process_id) {
                // Check resource limits
                if (memory_mb > proc.resources.max_memory_mb) {
                    try self.logProcessEvent(process_id, .resource_exceeded, "Memory limit exceeded");
                    return error.MemoryLimitExceeded;
                }
                if (cpu_percent > proc.resources.max_cpu_percent) {
                    try self.logProcessEvent(process_id, .resource_exceeded, "CPU limit exceeded");
                    return error.CPULimitExceeded;
                }

                proc.current_memory_mb = memory_mb;
                proc.current_cpu_percent = cpu_percent;
                return;
            }
        }
        return error.ProcessNotFound;
    }

    /// Schedule next process to run (simple round-robin)
    pub fn scheduleNext(self: *ProcessManager) ?u32 {
        self.mutex.lock();
        defer self.mutex.unlock();

        var best_process_id: ?u32 = null;
        var best_priority = ProcessPriority.idle;

        for (self.processes.items) |proc| {
            if ((proc.state == .ready or proc.state == .waiting) and proc.priority.value() >= best_priority.value()) {
                best_process_id = proc.id;
                best_priority = proc.priority;
            }
        }

        // perform context switch if appropriate
        if (best_process_id) |next_id| {
            if (self.running_process_id) |current_id| {
                if (current_id != next_id) {
                    _ = self.contextSwitch(current_id, next_id);
                }
            } else {
                // no running process, simply start next
                self.running_process_id = next_id;
                for (self.processes.items) |*p| {
                    if (p.id == next_id) {
                        p.state = .running;
                    }
                }
            }
        }
        return best_process_id;
    }

    /// Get process events
    pub fn getProcessEvents(self: *ProcessManager) []ProcessEvent {
        return self.process_events.items;
    }

    /// Get process events for specific process
    pub fn getProcessEventsByID(self: *ProcessManager, process_id: u32, allocator: std.mem.Allocator) !std.array_list.Managed(ProcessEvent) {
        var events = std.array_list.Managed(ProcessEvent).init(allocator);
        for (self.process_events.items) |event| {
            if (event.process_id == process_id) {
                try events.append(event);
            }
        }
        return events;
    }

    /// Log process event
    fn logProcessEvent(
        self: *ProcessManager,
        process_id: u32,
        event_type: ProcessEventType,
        details: []const u8,
    ) !void {
        const event = ProcessEvent{
            .event_id = self.next_event_id,
            .process_id = process_id,
            .event_type = event_type,
            .timestamp = std.time.timestamp(),
            .details = try self.allocator.dupe(u8, details),
        };

        try self.process_events.append(event);
        self.next_event_id += 1;
    }

    /// Get process count by state
    pub fn getProcessCountByState(self: *ProcessManager, state: ProcessState) usize {
        var count: usize = 0;
        for (self.processes.items) |proc| {
            if (proc.state == state) {
                count += 1;
            }
        }
        return count;
    }

    /// Get all processes
    pub fn getProcesses(self: *ProcessManager) []Process {
        return self.processes.items;
    }
};
