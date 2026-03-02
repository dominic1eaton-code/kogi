const std = @import("std");

// We don't alias `std.ArrayList` here because the generic type has some
// subtle behaviour and the alias was causing confusing diagnostics.
// Using the fully-qualified name makes intent clear and avoids the
// `array_list.Aligned(...).init` error we saw earlier.

// This module defines a simple project management system with projects and tasks.

pub const Task = struct {
    id: u32,
    title: []const u8,
    description: []const u8,
    completed: bool,
    priority: u8,
};

pub const Project = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    tasks: std.ArrayList(Task),
};

pub const ProjectManager = struct {
    allocator: std.mem.Allocator,
    projects: std.ArrayList(Project),

    pub fn init(allocator: std.mem.Allocator) ProjectManager {
        return ProjectManager{
            .allocator = allocator,
            .projects = std.ArrayList(Project){},
        };
    }

    pub fn deinit(self: *ProjectManager) void {
        for (self.projects.items) |*project| {
            project.tasks.deinit(self.allocator);
            self.allocator.free(project.name);
            self.allocator.free(project.description);
        }
        self.projects.deinit(self.allocator);
    }

    pub fn addProject(self: *ProjectManager, name: []const u8, description: []const u8) !void {
        const project = Project{
            .id = @as(u32, @intCast(self.projects.items.len)),
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .tasks = std.ArrayList(Task){},
        };
        try self.projects.append(self.allocator, project);
    }

    pub fn addTask(self: *ProjectManager, project_id: u32, title: []const u8, description: []const u8, priority: u8) !void {
        if (project_id < @as(u32, @intCast(self.projects.items.len))) {
            const task = Task{
                .id = @as(u32, @intCast(self.projects.items[project_id].tasks.items.len)),
                .title = try self.allocator.dupe(u8, title),
                .description = try self.allocator.dupe(u8, description),
                .completed = false,
                .priority = priority,
            };
            try self.projects.items[project_id].tasks.append(self.allocator, task);
        }
    }
};

pub fn portfolio() void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var manager = ProjectManager.init(allocator);
    defer manager.deinit();

    std.debug.print("Project Manager initialized\n", .{});
}
