//! KOGI Device Drivers - Independent Worker Device Drivers (IDD)
//! Abstracts devices/resources that ICPUs can use.

const std = @import("std");

/// Status of a device driver
pub const IDDStatus = enum {
    inactive,
    active,
    @"error",
};

/// Independent-worker Device Driver
pub const IDD = struct {
    id: u32,
    name: []const u8,
    driver_type: []const u8,
    status: IDDStatus,
};

/// Manager for IDDs
pub const DriverManager = struct {
    allocator: std.mem.Allocator,
    drivers: std.ArrayList(IDD),
    next_id: u32,
    mutex: std.Thread.Mutex = .{},

    pub fn init(allocator: std.mem.Allocator) DriverManager {
        return DriverManager{
            .allocator = allocator,
            .drivers = std.ArrayList(IDD){},
            .next_id = 1,
        };
    }

    pub fn deinit(self: *DriverManager) void {
        for (self.drivers.items) |drv| {
            self.allocator.free(drv.name);
            self.allocator.free(drv.driver_type);
        }
        self.drivers.deinit(self.allocator);
    }

    pub fn registerDriver(
        self: *DriverManager,
        name: []const u8,
        driver_type: []const u8,
    ) !u32 {
        self.mutex.lock();
        defer self.mutex.unlock();
        const id = self.next_id;
        self.next_id += 1;
        const drv = IDD{
            .id = id,
            .name = try self.allocator.dupe(u8, name),
            .driver_type = try self.allocator.dupe(u8, driver_type),
            .status = .inactive,
        };
        try self.drivers.append(self.allocator, drv);
        return id;
    }

    pub fn listDrivers(self: *DriverManager) []IDD {
        return self.drivers.items;
    }
};
