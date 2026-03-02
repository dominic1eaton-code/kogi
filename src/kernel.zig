const std = @import("std");
const portfolio = @import("portfolio.zig");

/// Worker represents an independent worker/freelancer in the kogi system
pub const Worker = struct {
    id: u32,
    name: []const u8,
    email: []const u8,
    skills: std.ArrayList([]const u8),
    hourly_rate: f32,
    active: bool,
};

/// Job represents a job posting or work assignment
pub const Job = struct {
    id: u32,
    title: []const u8,
    description: []const u8,
    required_skills: std.ArrayList([]const u8),
    budget: f32,
    deadline: i64,
    assigned_to: ?u32, // worker_id or null if unassigned
    completed: bool,
};

/// Contract represents a work agreement between a worker and a job
pub const Contract = struct {
    id: u32,
    worker_id: u32,
    job_id: u32,
    start_date: i64,
    end_date: ?i64,
    hourly_rate: f32,
    hours_worked: f32,
    status: enum { active, completed, terminated },
};

/// Core kernel managing workers, jobs, and contracts
pub const Kernel = struct {
    allocator: std.mem.Allocator,
    workers: std.ArrayList(Worker),
    jobs: std.ArrayList(Job),
    contracts: std.ArrayList(Contract),
    project_manager: portfolio.ProjectManager,

    pub fn init(allocator: std.mem.Allocator) Kernel {
        return Kernel{
            .allocator = allocator,
            .workers = std.ArrayList(Worker){},
            .jobs = std.ArrayList(Job){},
            .contracts = std.ArrayList(Contract){},
            .project_manager = portfolio.ProjectManager.init(allocator),
        };
    }

    pub fn deinit(self: *Kernel) void {
        // Clean up workers
        for (self.workers.items) |*worker| {
            // Free each skill string
            for (worker.skills.items) |skill| {
                self.allocator.free(skill);
            }
            worker.skills.deinit(self.allocator);
            self.allocator.free(worker.name);
            self.allocator.free(worker.email);
        }
        self.workers.deinit(self.allocator);

        // Clean up jobs
        for (self.jobs.items) |*job| {
            // Free each required skill string
            for (job.required_skills.items) |skill| {
                self.allocator.free(skill);
            }
            job.required_skills.deinit(self.allocator);
            self.allocator.free(job.title);
            self.allocator.free(job.description);
        }
        self.jobs.deinit(self.allocator);

        // Clean up contracts
        self.contracts.deinit(self.allocator);

        // Clean up project manager
        self.project_manager.deinit();
    }

    /// Register a new worker in the system
    pub fn registerWorker(self: *Kernel, name: []const u8, email: []const u8, hourly_rate: f32) !u32 {
        const worker_id = @as(u32, @intCast(self.workers.items.len));

        const worker = Worker{
            .id = worker_id,
            .name = try self.allocator.dupe(u8, name),
            .email = try self.allocator.dupe(u8, email),
            .skills = std.ArrayList([]const u8){},
            .hourly_rate = hourly_rate,
            .active = true,
        };

        try self.workers.append(self.allocator, worker);
        return worker_id;
    }

    /// Add a skill to a worker's profile
    pub fn addWorkerSkill(self: *Kernel, worker_id: u32, skill: []const u8) !void {
        if (worker_id < @as(u32, @intCast(self.workers.items.len))) {
            const skill_copy = try self.allocator.dupe(u8, skill);
            try self.workers.items[worker_id].skills.append(self.allocator, skill_copy);
        }
    }

    /// Post a new job to the system
    pub fn postJob(self: *Kernel, title: []const u8, description: []const u8, budget: f32, deadline: i64) !u32 {
        const job_id = @as(u32, @intCast(self.jobs.items.len));

        const job = Job{
            .id = job_id,
            .title = try self.allocator.dupe(u8, title),
            .description = try self.allocator.dupe(u8, description),
            .required_skills = std.ArrayList([]const u8){},
            .budget = budget,
            .deadline = deadline,
            .assigned_to = null,
            .completed = false,
        };

        try self.jobs.append(self.allocator, job);
        return job_id;
    }

    /// Add a required skill for a job
    pub fn addJobSkillRequirement(self: *Kernel, job_id: u32, skill: []const u8) !void {
        if (job_id < @as(u32, @intCast(self.jobs.items.len))) {
            const skill_copy = try self.allocator.dupe(u8, skill);
            try self.jobs.items[job_id].required_skills.append(self.allocator, skill_copy);
        }
    }

    /// Create a contract between a worker and a job
    pub fn createContract(self: *Kernel, worker_id: u32, job_id: u32, start_date: i64, hourly_rate: f32) !u32 {
        const contract_id = @as(u32, @intCast(self.contracts.items.len));

        const contract = Contract{
            .id = contract_id,
            .worker_id = worker_id,
            .job_id = job_id,
            .start_date = start_date,
            .end_date = null,
            .hourly_rate = hourly_rate,
            .hours_worked = 0.0,
            .status = .active,
        };

        try self.contracts.append(self.allocator, contract);

        // Mark job as assigned
        if (job_id < @as(u32, @intCast(self.jobs.items.len))) {
            self.jobs.items[job_id].assigned_to = worker_id;
        }

        return contract_id;
    }

    /// Log hours worked on a contract
    pub fn logHours(self: *Kernel, contract_id: u32, hours: f32) !void {
        if (contract_id < @as(u32, @intCast(self.contracts.items.len))) {
            self.contracts.items[contract_id].hours_worked += hours;
        }
    }

    /// Complete a contract
    pub fn completeContract(self: *Kernel, contract_id: u32, end_date: i64) !void {
        if (contract_id < @as(u32, @intCast(self.contracts.items.len))) {
            self.contracts.items[contract_id].status = .completed;
            self.contracts.items[contract_id].end_date = end_date;

            // Mark associated job as completed
            const job_id = self.contracts.items[contract_id].job_id;
            if (job_id < @as(u32, @intCast(self.jobs.items.len))) {
                self.jobs.items[job_id].completed = true;
            }
        }
    }

    /// Calculate total earnings for a worker
    pub fn calculateWorkerEarnings(self: *Kernel, worker_id: u32) f32 {
        var total_earnings: f32 = 0.0;

        for (self.contracts.items) |contract| {
            if (contract.worker_id == worker_id) {
                total_earnings += contract.hours_worked * contract.hourly_rate;
            }
        }

        return total_earnings;
    }

    /// Get active jobs count
    pub fn getActiveJobsCount(self: *Kernel) u32 {
        var count: u32 = 0;
        for (self.jobs.items) |job| {
            if (!job.completed) {
                count += 1;
            }
        }
        return count;
    }

    /// Get active workers count
    pub fn getActiveWorkersCount(self: *Kernel) u32 {
        var count: u32 = 0;
        for (self.workers.items) |worker| {
            if (worker.active) {
                count += 1;
            }
        }
        return count;
    }
};

/// Initialize and run the kogi kernel
pub fn runKernel() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var kernel = Kernel.init(allocator);
    defer kernel.deinit();

    std.debug.print("Kogi Kernel initialized successfully\n", .{});
    std.debug.print("Active workers: {}\n", .{kernel.getActiveWorkersCount()});
    std.debug.print("Active jobs: {}\n", .{kernel.getActiveJobsCount()});
}
