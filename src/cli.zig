const std = @import("std");
const system_module = @import("system.zig");
const identity_module = @import("identity.zig");
const security_module = @import("security.zig");

/// Command-line interface for the KOGI Operating System
pub const CLI = struct {
    allocator: std.mem.Allocator,
    system: *system_module.System,

    pub fn init(allocator: std.mem.Allocator, system: *system_module.System) CLI {
        return CLI{
            .allocator = allocator,
            .system = system,
        };
    }

    /// Display system statistics
    fn handleViewStatistics(self: *CLI) void {
        std.debug.print("\n╔════════════════════════════════════════════╗\n", .{});
        std.debug.print("║       KOGI SYSTEM STATISTICS               ║\n", .{});
        std.debug.print("╚════════════════════════════════════════════╝\n\n", .{});
        std.debug.print("Active Identities: {}\n", .{self.system.getActiveIdentitiesCount()});
        std.debug.print("Active Tasks: {}\n", .{self.system.getActiveTasksCount()});
        std.debug.print("Total Engagements: {}\n", .{self.system.engagements.items.len});
        // Collections and items managed through workspace_manager
        const collection_count = if (self.system.workspace_manager.workspace) |p| p.collections.items.len else 0;
        const item_count = if (self.system.workspace_manager.workspace) |p| p.items.items.len else 0;
        std.debug.print("Workspace Collections: {}\n", .{collection_count});
        std.debug.print("Workspace Items: {}\n", .{item_count});
        std.debug.print("Directory Organizations: {}\n", .{self.system.directory.getOrganizationCount()});
        std.debug.print("Vault Total Value: ${:.2}\n", .{self.system.vault.getTotalValue()});
        std.debug.print("Audit Log Entries: {}\n", .{self.system.security_manager.getAuditLog().len});
        std.debug.print("Active Sessions: {}\n", .{self.system.security_manager.sessions.items.len});
    }

    /// List all identities
    fn handleListIdentities(self: *CLI) void {
        std.debug.print("\n╔════════════════════════════════════════════╗\n", .{});
        std.debug.print("║           SYSTEM IDENTITIES                ║\n", .{});
        std.debug.print("╚════════════════════════════════════════════╝\n\n", .{});

        const identities = self.system.getIdentities();
        if (identities.len == 0) {
            std.debug.print("No identities created.\n", .{});
            return;
        }

        for (identities, 0..) |identity, idx| {
            std.debug.print("\n[Identity {}] {s}\n", .{ idx, identity.name });
            std.debug.print("  Email: {s}\n", .{identity.email});
            std.debug.print("  Hourly Rate: ${:.2}\n", .{identity.hourly_rate});
            std.debug.print("  Status: {s}\n", .{if (identity.active) "Active" else "Inactive"});

            if (identity.skills.items.len > 0) {
                std.debug.print("  Skills: ", .{});
                for (identity.skills.items, 0..) |skill, skill_idx| {
                    if (skill_idx > 0) std.debug.print(", ", .{});
                    std.debug.print("{s}", .{skill});
                }
                std.debug.print("\n", .{});
            }
        }
    }

    /// List all tasks
    fn handleListTasks(self: *CLI) void {
        std.debug.print("\n╔════════════════════════════════════════════╗\n", .{});
        std.debug.print("║           ACTIVE TASK POSTINGS             ║\n", .{});
        std.debug.print("╚════════════════════════════════════════════╝\n\n", .{});

        const tasks = self.system.getTasks();
        if (tasks.len == 0) {
            std.debug.print("No tasks posted.\n", .{});
            return;
        }

        for (tasks, 0..) |task, idx| {
            std.debug.print("\n[Task {}] {s}\n", .{ idx, task.title });
            std.debug.print("  Description: {s}\n", .{task.description});
            std.debug.print("  Budget: ${:.2}\n", .{task.budget});
            std.debug.print("  Status: {s}\n", .{if (task.completed) "Completed" else "Open"});

            if (task.assigned_to) |identity_id| {
                std.debug.print("  Assigned to: Identity {}\n", .{identity_id});
            } else {
                std.debug.print("  Assigned to: Unassigned\n", .{});
            }

            if (task.required_skills.items.len > 0) {
                std.debug.print("  Required Skills: ", .{});
                for (task.required_skills.items, 0..) |skill, skill_idx| {
                    if (skill_idx > 0) std.debug.print(", ", .{});
                    std.debug.print("{s}", .{skill});
                }
                std.debug.print("\n", .{});
            }
        }
    }

    /// Run a demo of the KOGI OS with sample data
    pub fn runDemo(self: *CLI) !void {
        std.debug.print("\n╔════════════════════════════════════════════╗\n", .{});
        std.debug.print("║   KOGI - Operating System for Workers      ║\n", .{});
        std.debug.print("║          Running Demo Mode                 ║\n", .{});
        std.debug.print("╚════════════════════════════════════════════╝\n", .{});

        // Create sample identities
        std.debug.print("\n📝 Creating system identities...\n", .{});
        const identity1_id = try self.system.createIdentity("Alice Johnson", "alice@example.com", 75.50, identity_module.IdentityType.developer);
        try self.system.addIdentitySkill(identity1_id, "Zig Programming");
        try self.system.addIdentitySkill(identity1_id, "System Design");

        const identity2_id = try self.system.createIdentity("Bob Smith", "bob@example.com", 65.00, identity_module.IdentityType.developer);
        try self.system.addIdentitySkill(identity2_id, "Zig Programming");
        try self.system.addIdentitySkill(identity2_id, "Testing");

        const identity3_id = try self.system.createIdentity("Carol White", "carol@example.com", 85.75, identity_module.IdentityType.consultant);
        try self.system.addIdentitySkill(identity3_id, "Project Management");
        try self.system.addIdentitySkill(identity3_id, "Documentation");

        std.debug.print("✓ Created 3 identities\n", .{});

        // Create sample tasks
        std.debug.print("\n📝 Posting tasks to system...\n", .{});
        const task1_id = try self.system.postTask(
            "Build REST API Server",
            "Create a high-performance REST API server in Zig",
            5000.00,
            1746000000,
        );
        try self.system.addTaskSkillRequirement(task1_id, "Zig Programming");
        try self.system.addTaskSkillRequirement(task1_id, "System Design");

        const task2_id = try self.system.postTask(
            "Write Unit Tests",
            "Comprehensive test suite for core modules",
            2000.00,
            1745000000,
        );
        try self.system.addTaskSkillRequirement(task2_id, "Zig Programming");
        try self.system.addTaskSkillRequirement(task2_id, "Testing");

        const task3_id = try self.system.postTask(
            "Project Documentation",
            "Create complete API documentation and user guides",
            1500.00,
            1744000000,
        );
        try self.system.addTaskSkillRequirement(task3_id, "Documentation");

        std.debug.print("✓ Posted 3 tasks\n", .{});

        // Create engagements
        std.debug.print("\n📝 Creating engagements...\n", .{});
        const engagement1_id = try self.system.createEngagement(identity1_id, task1_id, 1735000000, 75.50);
        try self.system.logHours(engagement1_id, 32.5);

        const engagement2_id = try self.system.createEngagement(identity2_id, task2_id, 1735000000, 65.00);
        try self.system.logHours(engagement2_id, 24.0);

        const engagement3_id = try self.system.createEngagement(identity3_id, task3_id, 1735000000, 85.75);
        try self.system.logHours(engagement3_id, 18.5);

        std.debug.print("✓ Created 3 engagements\n", .{});

        // Display statistics and listings
        self.handleViewStatistics();
        self.handleListIdentities();
        self.handleListTasks();

        // Display earnings information
        std.debug.print("\n╔════════════════════════════════════════════╗\n", .{});
        std.debug.print("║          IDENTITY EARNINGS REPORT          ║\n", .{});
        std.debug.print("╚════════════════════════════════════════════╝\n\n", .{});

        const identities = self.system.getIdentities();
        for (identities) |identity| {
            const earnings = self.system.calculateIdentityEarnings(identity.id);
            std.debug.print("{s}: ${:.2}\n", .{ identity.name, earnings });
            const conn_count = self.system.getIdentityActiveConnectionCount(identity.id);
            std.debug.print("  Active Connections: {}\n", .{conn_count});
        }

        std.debug.print("\n✓ Demo completed successfully!\n", .{});

        // Demonstrate security features
        std.debug.print("\n╔════════════════════════════════════════════╗\n", .{});
        std.debug.print("║         SECURITY & ACCESS CONTROL          ║\n", .{});
        std.debug.print("╚════════════════════════════════════════════╝\n\n", .{});

        // Initialize default roles
        var default_roles = try security_module.initializeDefaultRoles(self.allocator);
        defer {
            for (default_roles.items) |role| {
                var perms = role.permissions;
                perms.deinit(self.allocator);
            }
            default_roles.deinit(self.allocator);
        }

        // Assign roles to identities
        try self.system.security_manager.assignRole(identity1_id, security_module.Role.worker, 0, null);
        try self.system.security_manager.assignRole(identity2_id, security_module.Role.contractor, 0, null);
        try self.system.security_manager.assignRole(identity3_id, security_module.Role.admin, 0, null);

        std.debug.print("Assigned roles:\n", .{});
        std.debug.print("  Alice Johnson: Worker\n", .{});
        std.debug.print("  Bob Smith: Contractor\n", .{});
        std.debug.print("  Carol White: Admin\n\n", .{});

        // Create sessions
        _ = try self.system.security_manager.createSession(identity1_id, "token_alice_123", "192.168.1.100", "device_fp_1");
        _ = try self.system.security_manager.createSession(identity2_id, "token_bob_456", "192.168.1.101", "device_fp_2");
        std.debug.print("Created 2 sessions\n", .{});

        // Test access control
        std.debug.print("\nAccess Control Examples:\n", .{});
        const can_alice_create_task = self.system.security_manager.hasPermission(identity1_id, security_module.Permission.create_task);
        const can_bob_create_task = self.system.security_manager.hasPermission(identity2_id, security_module.Permission.create_task);
        const can_carol_manage_users = self.system.security_manager.hasPermission(identity3_id, security_module.Permission.manage_users);

        std.debug.print("  Alice (worker) can create tasks: {}\n", .{can_alice_create_task});
        std.debug.print("  Bob (contractor) can create tasks: {}\n", .{can_bob_create_task});
        std.debug.print("  Carol (admin) can manage users: {}\n", .{can_carol_manage_users});

        // Display audit log
        std.debug.print("\n╔════════════════════════════════════════════╗\n", .{});
        std.debug.print("║              AUDIT LOG (Recent)            ║\n", .{});
        std.debug.print("╚════════════════════════════════════════════╝\n\n", .{});
        const audit_log = self.system.security_manager.getAuditLog();
        const start_idx = if (audit_log.len > 5) audit_log.len - 5 else 0;
        for (audit_log[start_idx..]) |entry| {
            std.debug.print("[{s}] Identity {}: {s}\n", .{ @tagName(entry.event_type), entry.identity_id, entry.action });
        }

        std.debug.print("\n✓ Security demonstration completed!\n\n", .{});
    }
};

/// Start the KOGI OS CLI shell
pub fn startCLI(allocator: std.mem.Allocator, system: *system_module.System) !void {
    var cli = CLI.init(allocator, system);
    try cli.runDemo();
}
