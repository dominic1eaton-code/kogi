//! KOGI System Module - Core Operating System
//! The System is the kernel/core of the KOGI OS.
//! It manages identities, workspaces, tasks, engagements, and coordinates all subsystems.

const std = @import("std");
const identity_module = @import("identity.zig");
const workspace_module = @import("workspace.zig");
const registry_module = @import("registry.zig");
const directory_module = @import("directory.zig");
const vault_module = @import("vault.zig");
const security_module = @import("security.zig");
const logging_module = @import("logging.zig");
const events_module = @import("events.zig");
const state_module = @import("state.zig");
const distributed_module = @import("distributed.zig");
const networking_module = @import("networking.zig");
const observability_module = @import("observability.zig");
const processes_module = @import("processes.zig");
const memory_module = @import("memory.zig");
const trace_module = @import("trace.zig");
const bootloader_module = @import("bootloader.zig");

/// Task represents a unit of work that can be assigned to an identity
pub const Task = struct {
    id: u32,
    title: []const u8,
    description: []const u8,
    required_skills: std.ArrayList([]const u8),
    budget: f32,
    deadline: i64,
    assigned_to: ?u32,
    completed: bool,
};

/// Engagement status
pub const EngagementStatus = enum { active, completed, terminated };

/// Engagement links an identity to a task for a specific time period
pub const Engagement = struct {
    id: u32,
    identity_id: u32,
    task_id: u32,
    start_date: i64,
    end_date: ?i64,
    hourly_rate: f32,
    hours_worked: f32,
    status: EngagementStatus,
    completion_date: ?i64 = null,
};

/// System is the core operating system engine managing all subsystems
pub const System = struct {
    allocator: std.mem.Allocator,
    identity_manager: identity_module.IdentityManager,
    workspace_manager: workspace_module.WorkspaceManager,
    directory: directory_module.Directory,
    vault: vault_module.Vault,
    security_manager: security_module.SecurityManager,
    logger: logging_module.Logger,
    event_bus: events_module.EventBus,
    state_manager: state_module.StateManager,
    cluster: distributed_module.DistributedCluster,
    network_server: networking_module.NetworkServer,
    process_manager: processes_module.ProcessManager,
    memory_allocator: memory_module.MemoryAllocator,
    trace_manager: trace_module.TraceManager,
    audit_manager: trace_module.AuditManager,
    boot_manager: bootloader_module.BootManager,
    observability: observability_module.ObservabilityManager,
    tasks: std.ArrayList(Task),
    engagements: std.ArrayList(Engagement),

    pub fn init(allocator: std.mem.Allocator) System {
        // Initialize logger with default config
        var log_outputs = std.ArrayList(logging_module.LogOutput){};
        log_outputs.append(allocator, .stdout) catch {};
        const logger_config = logging_module.LoggerConfig{
            .min_level = .info,
            .outputs = log_outputs,
        };

        // Initialize state manager with default config
        const state_config = state_module.StateManagerConfig{};

        // Initialize cluster with default config
        const cluster_config = distributed_module.ClusterConfig{
            .cluster_name = "kogi-cluster",
            .node_id = 0,
        };

        // Initialize network server with default config
        const server_config = networking_module.ServerConfig{};

        // Initialize process manager with default config
        const scheduler_config = processes_module.SchedulerConfig{};

        // Initialize memory allocator with 4GB total memory
        const memory_allocator_config = 4096; // 4GB

        // Initialize trace manager
        const trace_manager = trace_module.TraceManager.init(allocator);

        // Initialize audit manager
        const audit_manager = trace_module.AuditManager.init(allocator);

        // Initialize boot manager with default config
        const boot_config = bootloader_module.BootConfig{};

        // Initialize observability manager
        const observability_mgr = observability_module.ObservabilityManager.init(allocator);

        return System{
            .allocator = allocator,
            .identity_manager = identity_module.IdentityManager.init(allocator),
            .workspace_manager = workspace_module.WorkspaceManager.init(allocator),
            .directory = directory_module.Directory.init(allocator),
            .vault = vault_module.Vault.init(allocator),
            .security_manager = security_module.SecurityManager.init(allocator),
            .logger = logging_module.Logger.init(allocator, logger_config) catch unreachable,
            .event_bus = events_module.EventBus.init(allocator),
            .state_manager = state_module.StateManager.init(allocator, state_config) catch unreachable,
            .cluster = distributed_module.DistributedCluster.init(allocator, cluster_config),
            .network_server = networking_module.NetworkServer.init(allocator, server_config),
            .process_manager = processes_module.ProcessManager.init(allocator, scheduler_config),
            .memory_allocator = memory_module.MemoryAllocator.init(allocator, memory_allocator_config),
            .trace_manager = trace_manager,
            .audit_manager = audit_manager,
            .boot_manager = bootloader_module.BootManager.init(allocator, boot_config),
            .observability = observability_mgr,
            .tasks = std.ArrayList(Task){},
            .engagements = std.ArrayList(Engagement){},
        };
    }

    pub fn deinit(self: *System) void {
        // Clean up identities
        self.identity_manager.deinit();
        // Clean up workspace
        self.workspace_manager.deinit();
        // Clean up directory
        self.directory.deinit();
        // Clean up vault
        self.vault.deinit();
        // Clean up security
        self.security_manager.deinit();
        // Clean up logging
        self.logger.deinit();
        // Clean up events
        self.event_bus.deinit();
        // Clean up state
        self.state_manager.deinit();
        // Clean up cluster
        self.cluster.deinit();
        // Clean up network server
        self.network_server.deinit();
        // Clean up process manager
        var process_manager = self.process_manager;
        process_manager.deinit();
        // Clean up memory allocator
        var memory_allocator = self.memory_allocator;
        memory_allocator.deinit();
        // Clean up trace manager
        var trace_manager = self.trace_manager;
        trace_manager.deinit();
        // Clean up audit manager
        var audit_manager = self.audit_manager;
        audit_manager.deinit();
        // Clean up boot manager
        var boot_manager = self.boot_manager;
        boot_manager.deinit();
        // Clean up observability manager
        var observability_mgr = self.observability;
        observability_mgr.deinit();
        // Clean up tasks
        for (self.tasks.items) |*task| {
            self.allocator.free(task.title);
            self.allocator.free(task.description);
            for (task.required_skills.items) |skill| {
                self.allocator.free(skill);
            }
            task.required_skills.deinit(self.allocator);
        }
        self.tasks.deinit(self.allocator);
        // Clean up engagements
        for (self.engagements.items) |*engagement| {
            engagement.completion_date = null;
        }
        self.engagements.deinit(self.allocator);
    }

    // ========== Identity Management Delegation ==========

    /// Create a new identity in the system
    pub fn createIdentity(
        self: *System,
        name: []const u8,
        email: []const u8,
        hourly_rate: f32,
        itype: identity_module.IdentityType,
    ) !u32 {
        return try self.identity_manager.createIdentity(name, email, hourly_rate, itype);
    }

    /// Add a skill to an identity
    pub fn addIdentitySkill(self: *System, identity_id: u32, skill: []const u8) !void {
        try self.identity_manager.addSkill(identity_id, skill);
    }

    /// Get count of active identities
    pub fn getActiveIdentitiesCount(self: *System) u32 {
        return self.identity_manager.getActiveCount();
    }

    pub fn getIdentities(self: *System) []identity_module.Identity {
        return self.identity_manager.identities.items;
    }

    /// Connection management delegation: add a connection for an identity
    pub fn addIdentityConnection(
        self: *System,
        identity_id: u32,
        conn_type: registry_module.ConnectionType,
        username: []const u8,
        provider: []const u8,
        details: []const u8,
    ) !u32 {
        return try self.identity_manager.addIdentityConnection(identity_id, conn_type, username, provider, details);
    }

    pub fn deactivateIdentityConnection(self: *System, identity_id: u32, conn_id: u32) !void {
        try self.identity_manager.deactivateIdentityConnection(identity_id, conn_id);
    }

    pub fn getIdentityActiveConnectionCount(self: *System, identity_id: u32) u32 {
        return self.identity_manager.getIdentityActiveConnectionCount(identity_id);
    }

    // ========== Task Management ==========

    /// Post a new task to the system
    pub fn postTask(self: *System, title: []const u8, description: []const u8, budget: f32, deadline: i64) !u32 {
        const task_id = @as(u32, @intCast(self.tasks.items.len));

        const task = Task{
            .id = task_id,
            .title = try self.allocator.dupe(u8, title),
            .description = try self.allocator.dupe(u8, description),
            .required_skills = std.ArrayList([]const u8){},
            .budget = budget,
            .deadline = deadline,
            .assigned_to = null,
            .completed = false,
        };

        try self.tasks.append(self.allocator, task);
        return task_id;
    }

    /// Add a skill requirement to a task
    pub fn addTaskSkillRequirement(self: *System, task_id: u32, skill: []const u8) !void {
        if (task_id < @as(u32, @intCast(self.tasks.items.len))) {
            const skill_copy = try self.allocator.dupe(u8, skill);
            try self.tasks.items[task_id].required_skills.append(self.allocator, skill_copy);
        }
    }

    /// Get count of active tasks
    pub fn getActiveTasksCount(self: *System) u32 {
        var count: u32 = 0;
        for (self.tasks.items) |task| {
            if (!task.completed) {
                count += 1;
            }
        }
        return count;
    }

    pub fn getTasks(self: *System) []Task {
        return self.tasks.items;
    }

    // ========== Engagement Management ==========

    /// Create an engagement between an identity and a task
    pub fn createEngagement(
        self: *System,
        identity_id: u32,
        task_id: u32,
        start_date: i64,
        hourly_rate: f32,
    ) !u32 {
        const engagement_id = @as(u32, @intCast(self.engagements.items.len));

        const engagement = Engagement{
            .id = engagement_id,
            .identity_id = identity_id,
            .task_id = task_id,
            .start_date = start_date,
            .end_date = null,
            .hourly_rate = hourly_rate,
            .hours_worked = 0.0,
            .status = .active,
        };

        try self.engagements.append(self.allocator, engagement);
        if (task_id < @as(u32, @intCast(self.tasks.items.len))) {
            self.tasks.items[task_id].assigned_to = identity_id;
        }
        return engagement_id;
    }

    /// Log hours to an engagement
    pub fn logHours(self: *System, engagement_id: u32, hours: f32) !void {
        if (engagement_id < @as(u32, @intCast(self.engagements.items.len))) {
            self.engagements.items[engagement_id].hours_worked += hours;
        }
    }

    /// Complete an engagement
    pub fn completeEngagement(self: *System, engagement_id: u32, completion_date: i64) !void {
        if (engagement_id < @as(u32, @intCast(self.engagements.items.len))) {
            self.engagements.items[engagement_id].status = .completed;
            self.engagements.items[engagement_id].completion_date = completion_date;
            self.tasks.items[self.engagements.items[engagement_id].task_id].completed = true;
        }
    }

    /// Calculate earnings for an identity
    pub fn calculateIdentityEarnings(self: *System, identity_id: u32) f32 {
        var earnings: f32 = 0.0;
        for (self.engagements.items) |engagement| {
            if (engagement.identity_id == identity_id and engagement.status == .completed) {
                earnings += engagement.hours_worked * engagement.hourly_rate;
            }
        }
        return earnings;
    }

    pub fn getEngagements(self: *System) []Engagement {
        return self.engagements.items;
    }
};

// Backward compatibility alias
pub const Kernel = System;

pub fn runSystem() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var system = System.init(allocator);
    defer system.deinit();

    // Demo
    _ = try system.createIdentity("Alice", "alice@example.com", 50.0, .developer);
    try system.addIdentitySkill(0, "Zig");

    _ = try system.postTask("Build Parser", "Create a parser", 1000.0, 0);
    try system.addTaskSkillRequirement(0, "Zig");

    _ = try system.createEngagement(0, 0, 0, 50.0);
    try system.logHours(0, 40.0);
    try system.completeEngagement(0, 0);

    std.debug.print("Identity earnings: {d}\n", .{system.calculateIdentityEarnings(0)});
}
