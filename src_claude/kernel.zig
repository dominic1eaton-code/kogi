// kernel/src/kernel.zig — Kogi Kernel: central orchestrator
const std = @import("std");
const Allocator = std.mem.Allocator;
const EventBus = @import("event_bus.zig").EventBus;

pub const ModuleStatus = enum { registered, running, stopped, error_state };

pub const Module = struct {
    name: []const u8,
    status: ModuleStatus,
    pid: ?u32,

    pub fn init(name: []const u8) Module {
        return .{ .name = name, .status = .registered, .pid = null };
    }
};

pub const KernelConfig = struct {
    max_modules: usize = 64,
    max_users: usize = 1_000_000,
    event_bus_capacity: usize = 4096,
};

pub const Kernel = struct {
    allocator: Allocator,
    config: KernelConfig,
    modules: std.StringHashMap(Module),
    module_count: usize,
    event_bus: EventBus,
    running: bool,

    pub fn init(allocator: Allocator) !Kernel {
        const config = KernelConfig{};
        return Kernel{
            .allocator = allocator,
            .config = config,
            .modules = std.StringHashMap(Module).init(allocator),
            .module_count = 0,
            .event_bus = try EventBus.init(allocator, config.event_bus_capacity),
            .running = false,
        };
    }

    pub fn deinit(self: *Kernel) void {
        self.modules.deinit();
        self.event_bus.deinit();
    }

    pub fn registerModule(self: *Kernel, name: []const u8) !void {
        if (self.module_count >= self.config.max_modules) {
            return error.MaxModulesReached;
        }
        const module = Module.init(name);
        try self.modules.put(name, module);
        self.module_count += 1;
        std.log.debug("Module registered: {s}", .{name});
    }

    pub fn getModule(self: *Kernel, name: []const u8) ?*Module {
        return self.modules.getPtr(name);
    }

    pub fn startModule(self: *Kernel, name: []const u8) !void {
        if (self.modules.getPtr(name)) |mod| {
            mod.status = .running;
            try self.event_bus.publish(.{
                .event_type = "module.started",
                .payload = name,
                .timestamp = std.time.timestamp(),
            });
        } else {
            return error.ModuleNotFound;
        }
    }

    pub fn stopModule(self: *Kernel, name: []const u8) !void {
        if (self.modules.getPtr(name)) |mod| {
            mod.status = .stopped;
        } else {
            return error.ModuleNotFound;
        }
    }
};

// ─── Tests ────────────────────────────────────────────────────────────────────
test "kernel init and module registration" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var k = try Kernel.init(allocator);
    defer k.deinit();

    try k.registerModule("portfolio");
    try k.registerModule("workspace");

    try std.testing.expectEqual(@as(usize, 2), k.module_count);
    try std.testing.expect(k.getModule("portfolio") != null);
    try std.testing.expect(k.getModule("workspace") != null);
    try std.testing.expect(k.getModule("missing") == null);
}
