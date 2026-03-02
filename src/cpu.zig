//! KOGI CPU Abstraction - Independent Worker CPUs (ICPU)
//! Each CPU represents an independent worker (contractor, freelancer, consultant, etc.).

const std = @import("std");

/// Status of an ICPU
pub const ICPUStatus = enum {
    offline,
    idle,
    busy,
    maintenance,
    retired,
};

/// Representation of an independent worker CPU
pub const ICPU = struct {
    id: u32,
    name: []const u8,
    email: []const u8,
    skills: std.ArrayList([]const u8),
    status: ICPUStatus,
    hourly_rate: f32,
};

/// CPU manager storing all ICPUs
pub const CPUManager = struct {
    allocator: std.mem.Allocator,
    cpus: std.ArrayList(ICPU),
    next_id: u32,
    mutex: std.Thread.Mutex = .{},

    pub fn init(allocator: std.mem.Allocator) CPUManager {
        return CPUManager{
            .allocator = allocator,
            .cpus = std.ArrayList(ICPU){},
            .next_id = 1,
        };
    }

    pub fn deinit(self: *CPUManager) void {
        for (self.cpus.items) |cpu| {
            self.allocator.free(cpu.name);
            self.allocator.free(cpu.email);
            for (cpu.skills.items) |s| self.allocator.free(s);
            cpu.skills.deinit(self.allocator);
        }
        self.cpus.deinit(self.allocator);
    }

    pub fn createCPU(
        self: *CPUManager,
        name: []const u8,
        email: []const u8,
        hourly_rate: f32,
    ) !u32 {
        self.mutex.lock();
        defer self.mutex.unlock();

        const id = self.next_id;
        self.next_id += 1;
        const cpu = ICPU{
            .id = id,
            .name = try self.allocator.dupe(u8, name),
            .email = try self.allocator.dupe(u8, email),
            .skills = std.ArrayList([]const u8){},
            .status = .offline,
            .hourly_rate = hourly_rate,
        };
        try self.cpus.append(self.allocator, cpu);
        return id;
    }

    pub fn getCPU(self: *CPUManager, id: u32) ?*ICPU {
        for (self.cpus.items) |*cpu| if (cpu.id == id) return cpu;
        return null;
    }

    pub fn listCPUs(self: *CPUManager) []ICPU {
        return self.cpus.items;
    }
};
