// This module defines a worker management system with workers, assignments, and payments.
// @copyright 2026 Kogi Inc. All rights reserved.
// @license MIT
// @author Kogi Team <
const std = @import("std");
const ArrayList = std.ArrayList;
const StringHashMap = std.StringHashMap;

pub const WorkerManagementSystem = struct {
    workers: StringHashMap(Worker),
    assignments: StringHashMap(Assignment),
    payments: StringHashMap(Payment),
    allocator: std.mem.Allocator,

    pub const Worker = struct {
        name: []const u8,
        skills: []const u8,
        hourly_rate: f64,
        status: []const u8,
    };

    pub const Assignment = struct {
        worker_id: []const u8,
        task: []const u8,
        deadline: []const u8,
        budget: f64,
        status: []const u8,
    };

    pub const Payment = struct {
        worker_id: []const u8,
        amount: f64,
        status: []const u8,
    };

    pub fn init(allocator: std.mem.Allocator) WorkerManagementSystem {
        return .{
            .workers = StringHashMap(Worker).init(allocator),
            .assignments = StringHashMap(Assignment).init(allocator),
            .payments = StringHashMap(Payment).init(allocator),
            .allocator = allocator,
        };
    }

    pub fn registerWorker(self: *WorkerManagementSystem, worker_id: []const u8, name: []const u8, skills: []const u8, rate: f64) !void {
        try self.workers.put(worker_id, .{ .name = name, .skills = skills, .hourly_rate = rate, .status = "active" });
    }

    pub fn createAssignment(self: *WorkerManagementSystem, assignment_id: []const u8, worker_id: []const u8, task: []const u8, deadline: []const u8, budget: f64) !void {
        if (!self.workers.contains(worker_id)) return error.WorkerNotFound;
        try self.assignments.put(assignment_id, .{ .worker_id = worker_id, .task = task, .deadline = deadline, .budget = budget, .status = "pending" });
    }

    pub fn updateAssignmentStatus(self: *WorkerManagementSystem, assignment_id: []const u8, status: []const u8) void {
        if (self.assignments.getPtr(assignment_id)) |assignment| {
            assignment.status = status;
        }
    }

    pub fn processPayment(self: *WorkerManagementSystem, assignment_id: []const u8, amount: f64) !void {
        if (self.assignments.get(assignment_id)) |assignment| {
            try self.payments.put(assignment_id, .{ .worker_id = assignment.worker_id, .amount = amount, .status = "completed" });
        } else return error.AssignmentNotFound;
    }

    pub fn deinit(self: *WorkerManagementSystem) void {
        self.workers.deinit();
        self.assignments.deinit();
        self.payments.deinit();
    }
};

pub fn worker() void {
    const allocator = std.heap.page_allocator;
    var system = WorkerManagementSystem.init(allocator);
    defer system.deinit();

    // Example usage
    _ = system.registerWorker("worker1", "Alice", "Programming, Design", 50.0);
    _ = system.createAssignment("assignment1", "worker1", "Build a website", "2024-12-31", 5000.0);
    system.updateAssignmentStatus("assignment1", "in progress");
    _ = system.processPayment("assignment1", 2500.0);
}

pub fn executor() void {
    worker();
}
