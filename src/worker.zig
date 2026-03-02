const std = @import("std");
const accounts_module = @import("accounts.zig");

/// Common errors for the worker module
pub const WorkerError = error{
    WorkerNotFound,
};

/// Broad classification of worker roles supported by the platform.
pub const WorkerType = enum {
    contractor,
    consultant,
    freelancer,
    gig,
    project_based,
    artist,
    musician,
    developer,
    gamer,
    other,
};

/// Worker represents an independent worker/freelancer in the kogi system
pub const Worker = struct {
    id: u32,
    name: []const u8,
    email: []const u8,
    worker_type: WorkerType,
    skills: std.ArrayList([]const u8),
    hourly_rate: f32,
    active: bool,
    account_manager: accounts_module.AccountManager,
};

/// WorkerManager handles all worker-related operations
pub const WorkerManager = struct {
    allocator: std.mem.Allocator,
    workers: std.ArrayList(Worker),

    pub fn init(allocator: std.mem.Allocator) WorkerManager {
        return WorkerManager{
            .allocator = allocator,
            .workers = std.ArrayList(Worker){},
        };
    }

    pub fn deinit(self: *WorkerManager) void {
        for (self.workers.items) |*worker| {
            // Free each skill string
            for (worker.skills.items) |skill| {
                self.allocator.free(skill);
            }
            worker.skills.deinit(self.allocator);
            self.allocator.free(worker.name);
            self.allocator.free(worker.email);
            // deinit accounts
            worker.account_manager.deinit();
        }
        self.workers.deinit(self.allocator);
    }

    /// Register a new worker in the system
    pub fn registerWorker(
        self: *WorkerManager,
        name: []const u8,
        email: []const u8,
        hourly_rate: f32,
        wtype: WorkerType,
    ) !u32 {
        const worker_id = @as(u32, @intCast(self.workers.items.len));

        const worker = Worker{
            .id = worker_id,
            .name = try self.allocator.dupe(u8, name),
            .email = try self.allocator.dupe(u8, email),
            .worker_type = wtype,
            .skills = std.ArrayList([]const u8){},
            .hourly_rate = hourly_rate,
            .active = true,
            .account_manager = accounts_module.AccountManager.init(self.allocator),
        };

        try self.workers.append(self.allocator, worker);
        return worker_id;
    }

    /// Add a skill to a worker's profile
    pub fn addSkill(self: *WorkerManager, worker_id: u32, skill: []const u8) !void {
        if (worker_id < @as(u32, @intCast(self.workers.items.len))) {
            const skill_copy = try self.allocator.dupe(u8, skill);
            try self.workers.items[worker_id].skills.append(self.allocator, skill_copy);
        }
    }

    /// Account management helpers: add an account for a worker
    pub fn addWorkerAccount(
        self: *WorkerManager,
        worker_id: u32,
        acct_type: accounts_module.AccountType,
        username: []const u8,
        provider: []const u8,
        details: []const u8,
    ) !u32 {
        if (worker_id >= @as(u32, @intCast(self.workers.items.len))) {
            return WorkerError.WorkerNotFound;
        }
        return try self.workers.items[worker_id].account_manager.addAccount(
            acct_type,
            username,
            provider,
            details,
        );
    }

    pub fn deactivateWorkerAccount(self: *WorkerManager, worker_id: u32, acct_id: u32) !void {
        if (worker_id >= @as(u32, @intCast(self.workers.items.len))) {
            return WorkerError.WorkerNotFound;
        }
        try self.workers.items[worker_id].account_manager.deactivateAccount(acct_id);
    }

    pub fn getWorkerActiveAccountCount(self: *WorkerManager, worker_id: u32) u32 {
        if (worker_id < @as(u32, @intCast(self.workers.items.len))) {
            return self.workers.items[worker_id].account_manager.getActiveCount();
        }
        return 0;
    }

    /// Deactivate a worker
    pub fn deactivateWorker(self: *WorkerManager, worker_id: u32) void {
        if (worker_id < @as(u32, @intCast(self.workers.items.len))) {
            self.workers.items[worker_id].active = false;
        }
    }

    /// Get count of active workers
    pub fn getActiveCount(self: *WorkerManager) u32 {
        var count: u32 = 0;
        for (self.workers.items) |worker| {
            if (worker.active) {
                count += 1;
            }
        }
        return count;
    }

    /// Return a list of workers matching a given type.
    pub fn getWorkersByType(self: *WorkerManager, wtype: WorkerType, allocator: std.mem.Allocator) !std.ArrayList(Worker) {
        var result = std.ArrayList(Worker){};
        for (self.workers.items) |worker| {
            if (worker.worker_type == wtype) {
                try result.append(allocator, worker);
            }
        }
        return result;
    }
};
