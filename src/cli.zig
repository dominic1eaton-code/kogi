const std = @import("std");
const kernel_module = @import("kernel.zig");

/// Command-line interface for the kogi portfolio management system
pub const CLI = struct {
    allocator: std.mem.Allocator,
    kernel: *kernel_module.Kernel,

    pub fn init(allocator: std.mem.Allocator, kernel: *kernel_module.Kernel) CLI {
        return CLI{
            .allocator = allocator,
            .kernel = kernel,
        };
    }

    /// Display system statistics
    fn handleViewStatistics(self: *CLI) void {
        std.debug.print("\n╔════════════════════════════════════════════╗\n", .{});
        std.debug.print("║       SYSTEM STATISTICS                    ║\n", .{});
        std.debug.print("╚════════════════════════════════════════════╝\n\n", .{});
        std.debug.print("Active Workers: {}\n", .{self.kernel.getActiveWorkersCount()});
        std.debug.print("Active Jobs: {}\n", .{self.kernel.getActiveJobsCount()});
        std.debug.print("Total Contracts: {}\n", .{self.kernel.contracts.items.len});
        std.debug.print("Total Projects: {}\n", .{self.kernel.project_manager.projects.items.len});
    }

    /// List all workers
    fn handleListWorkers(self: *CLI) void {
        std.debug.print("\n╔════════════════════════════════════════════╗\n", .{});
        std.debug.print("║           REGISTERED WORKERS               ║\n", .{});
        std.debug.print("╚════════════════════════════════════════════╝\n\n", .{});

        if (self.kernel.workers.items.len == 0) {
            std.debug.print("No workers registered.\n", .{});
            return;
        }

        for (self.kernel.workers.items, 0..) |worker, idx| {
            std.debug.print("\n[Worker {}] {s}\n", .{ idx, worker.name });
            std.debug.print("  Email: {s}\n", .{worker.email});
            std.debug.print("  Hourly Rate: ${:.2}\n", .{worker.hourly_rate});
            std.debug.print("  Status: {s}\n", .{if (worker.active) "Active" else "Inactive"});

            if (worker.skills.items.len > 0) {
                std.debug.print("  Skills: ", .{});
                for (worker.skills.items, 0..) |skill, skill_idx| {
                    if (skill_idx > 0) std.debug.print(", ", .{});
                    std.debug.print("{s}", .{skill});
                }
                std.debug.print("\n", .{});
            }
        }
    }

    /// List all jobs
    fn handleListJobs(self: *CLI) void {
        std.debug.print("\n╔════════════════════════════════════════════╗\n", .{});
        std.debug.print("║           ACTIVE JOB POSTINGS              ║\n", .{});
        std.debug.print("╚════════════════════════════════════════════╝\n\n", .{});

        if (self.kernel.jobs.items.len == 0) {
            std.debug.print("No jobs posted.\n", .{});
            return;
        }

        for (self.kernel.jobs.items, 0..) |job, idx| {
            std.debug.print("\n[Job {}] {s}\n", .{ idx, job.title });
            std.debug.print("  Description: {s}\n", .{job.description});
            std.debug.print("  Budget: ${:.2}\n", .{job.budget});
            std.debug.print("  Status: {s}\n", .{if (job.completed) "Completed" else "Open"});

            if (job.assigned_to) |worker_id| {
                std.debug.print("  Assigned to: Worker {}\n", .{worker_id});
            } else {
                std.debug.print("  Assigned to: Unassigned\n", .{});
            }

            if (job.required_skills.items.len > 0) {
                std.debug.print("  Required Skills: ", .{});
                for (job.required_skills.items, 0..) |skill, skill_idx| {
                    if (skill_idx > 0) std.debug.print(", ", .{});
                    std.debug.print("{s}", .{skill});
                }
                std.debug.print("\n", .{});
            }
        }
    }

    /// Run a demo of the CLI with sample data
    pub fn runDemo(self: *CLI) !void {
        std.debug.print("\n╔════════════════════════════════════════════╗\n", .{});
        std.debug.print("║   KOGI - Portfolio Management System       ║\n", .{});
        std.debug.print("║          Running Demo Mode                 ║\n", .{});
        std.debug.print("╚════════════════════════════════════════════╝\n", .{});

        // Create sample workers
        std.debug.print("\n📝 Creating sample workers...\n", .{});
        const worker1_id = try self.kernel.registerWorker("Alice Johnson", "alice@example.com", 75.50);
        try self.kernel.addWorkerSkill(worker1_id, "Zig Programming");
        try self.kernel.addWorkerSkill(worker1_id, "System Design");

        const worker2_id = try self.kernel.registerWorker("Bob Smith", "bob@example.com", 65.00);
        try self.kernel.addWorkerSkill(worker2_id, "Zig Programming");
        try self.kernel.addWorkerSkill(worker2_id, "Testing");

        const worker3_id = try self.kernel.registerWorker("Carol White", "carol@example.com", 85.75);
        try self.kernel.addWorkerSkill(worker3_id, "Project Management");
        try self.kernel.addWorkerSkill(worker3_id, "Documentation");

        std.debug.print("✓ Created 3 workers\n", .{});

        // Create sample jobs
        std.debug.print("\n📝 Creating sample jobs...\n", .{});
        const job1_id = try self.kernel.postJob(
            "Build REST API Server",
            "Create a high-performance REST API server in Zig",
            5000.00,
            1746000000,
        );
        try self.kernel.addJobSkillRequirement(job1_id, "Zig Programming");
        try self.kernel.addJobSkillRequirement(job1_id, "System Design");

        const job2_id = try self.kernel.postJob(
            "Write Unit Tests",
            "Comprehensive test suite for core modules",
            2000.00,
            1745000000,
        );
        try self.kernel.addJobSkillRequirement(job2_id, "Zig Programming");
        try self.kernel.addJobSkillRequirement(job2_id, "Testing");

        const job3_id = try self.kernel.postJob(
            "Project Documentation",
            "Create complete API documentation and user guides",
            1500.00,
            1744000000,
        );
        try self.kernel.addJobSkillRequirement(job3_id, "Documentation");

        std.debug.print("✓ Created 3 jobs\n", .{});

        // Create contracts
        std.debug.print("\n📝 Creating contracts...\n", .{});
        const contract1_id = try self.kernel.createContract(worker1_id, job1_id, 1735000000, 75.50);
        try self.kernel.logHours(contract1_id, 32.5);

        const contract2_id = try self.kernel.createContract(worker2_id, job2_id, 1735000000, 65.00);
        try self.kernel.logHours(contract2_id, 24.0);

        const contract3_id = try self.kernel.createContract(worker3_id, job3_id, 1735000000, 85.75);
        try self.kernel.logHours(contract3_id, 18.5);

        std.debug.print("✓ Created 3 contracts\n", .{});

        // Display statistics and listings
        self.handleViewStatistics();
        self.handleListWorkers();
        self.handleListJobs();

        // Display earnings information
        std.debug.print("\n╔════════════════════════════════════════════╗\n", .{});
        std.debug.print("║          WORKER EARNINGS REPORT            ║\n", .{});
        std.debug.print("╚════════════════════════════════════════════╝\n\n", .{});

        for (self.kernel.workers.items) |worker| {
            const earnings = self.kernel.calculateWorkerEarnings(worker.id);
            std.debug.print("{s}: ${:.2}\n", .{ worker.name, earnings });
        }

        std.debug.print("\n✓ Demo completed successfully!\n\n", .{});
    }
};

/// Start the CLI demo
pub fn startCLI(allocator: std.mem.Allocator, kernel: *kernel_module.Kernel) !void {
    var cli = CLI.init(allocator, kernel);
    try cli.runDemo();
}
