const std = @import("std");
const worker_module = @import("worker.zig");
const accounts_module = @import("accounts.zig");
const portfolio_module = @import("portfolio.zig");

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

/// Kernel manages jobs and contracts for the portfolio system
pub const Kernel = struct {
    allocator: std.mem.Allocator,
    worker_manager: worker_module.WorkerManager,
    portfolio_manager: portfolio_module.PortfolioManager,
    jobs: std.ArrayList(Job),
    contracts: std.ArrayList(Contract),

    pub fn init(allocator: std.mem.Allocator) Kernel {
        return Kernel{
            .allocator = allocator,
            .worker_manager = worker_module.WorkerManager.init(allocator),
            .portfolio_manager = portfolio_module.PortfolioManager.init(allocator),
            .jobs = std.ArrayList(Job){},
            .contracts = std.ArrayList(Contract){},
        };
    }

    pub fn deinit(self: *Kernel) void {
        // Clean up worker manager
        self.worker_manager.deinit();

        // Clean up portfolio manager
        self.portfolio_manager.deinit();

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
    }

    /// ============ Worker Management (delegates to WorkerManager) ============
    pub fn registerWorker(self: *Kernel, name: []const u8, email: []const u8, hourly_rate: f32, wtype: worker_module.WorkerType) !u32 {
        return try self.worker_manager.registerWorker(name, email, hourly_rate, wtype);
    }

    pub fn addWorkerSkill(self: *Kernel, worker_id: u32, skill: []const u8) !void {
        return try self.worker_manager.addSkill(worker_id, skill);
    }

    pub fn getActiveWorkersCount(self: *Kernel) u32 {
        return self.worker_manager.getActiveCount();
    }

    /// ============ Account Delegation ============
    pub fn addWorkerAccount(
        self: *Kernel,
        worker_id: u32,
        acct_type: accounts_module.AccountType,
        username: []const u8,
        provider: []const u8,
        details: []const u8,
    ) !u32 {
        return try self.worker_manager.addWorkerAccount(worker_id, acct_type, username, provider, details);
    }

    pub fn deactivateWorkerAccount(self: *Kernel, worker_id: u32, acct_id: u32) !void {
        return try self.worker_manager.deactivateWorkerAccount(worker_id, acct_id);
    }

    pub fn getWorkerActiveAccountCount(self: *Kernel, worker_id: u32) u32 {
        return self.worker_manager.getWorkerActiveAccountCount(worker_id);
    }

    pub fn getWorkers(self: *Kernel) []worker_module.Worker {
        return self.worker_manager.workers.items;
    }

    /// ============ Job Management ============
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

    pub fn addJobSkillRequirement(self: *Kernel, job_id: u32, skill: []const u8) !void {
        if (job_id < @as(u32, @intCast(self.jobs.items.len))) {
            const skill_copy = try self.allocator.dupe(u8, skill);
            try self.jobs.items[job_id].required_skills.append(self.allocator, skill_copy);
        }
    }

    pub fn getActiveJobsCount(self: *Kernel) u32 {
        var count: u32 = 0;
        for (self.jobs.items) |job| {
            if (!job.completed) {
                count += 1;
            }
        }
        return count;
    }

    /// ============ Contract Management ============
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

    pub fn logHours(self: *Kernel, contract_id: u32, hours: f32) !void {
        if (contract_id < @as(u32, @intCast(self.contracts.items.len))) {
            self.contracts.items[contract_id].hours_worked += hours;
        }
    }

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

    /// ============ Reporting and Analytics ============
    pub fn calculateWorkerEarnings(self: *Kernel, worker_id: u32) f32 {
        var total_earnings: f32 = 0.0;

        for (self.contracts.items) |contract| {
            if (contract.worker_id == worker_id) {
                total_earnings += contract.hours_worked * contract.hourly_rate;
            }
        }

        return total_earnings;
    }
};

/// Initialize and run the kernel
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
