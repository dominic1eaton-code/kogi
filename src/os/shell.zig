const std = @import("std");
const terminal_module = @import("terminal.zig");
const kernel_module = @import("kernel.zig");
const system_module = @import("system.zig");

pub const Shell = struct {
    allocator: std.mem.Allocator,
    term: terminal_module.Terminal,
    history: std.ArrayList([]const u8),

    pub fn init(allocator: std.mem.Allocator, prompt: []const u8) Shell {
        return Shell{
            .allocator = allocator,
            .term = terminal_module.Terminal.init(allocator, prompt),
            .history = std.ArrayList([]const u8){},
        };
    }

    pub fn deinit(self: *Shell) void {
        for (self.history.items) |s| self.allocator.free(s);
        self.history.deinit();
    }

    pub fn run(self: *Shell, system: *system_module.System, _kernel: ?*kernel_module.Kernel) !void {
        try self.term.writePrompt();
        while (true) {
            try self.term.writePrompt();
            const line = try self.term.readLine();
            if (line.len == 0) {
                self.allocator.free(line);
                continue;
            }
            // store history (dupe)
            const dup = try self.allocator.dupe(u8, line);
            try self.history.append(self.allocator, dup);

            if (std.mem.eql(u8, line, "exit") or std.mem.eql(u8, line, "quit")) {
                self.allocator.free(line);
                break;
            } else if (std.mem.eql(u8, line, "syscalls")) {
                if (kernel) |_| {
                    const names = kernel_module.syscallNames();
                    for (names) |n| {
                        std.debug.print("{s}\n", .{n});
                    }
                } else {
                    std.debug.print("No kernel available (local mode).\n", .{});
                }
            } else if (std.mem.eql(u8, line, "time")) {
                if (_kernel) |k| {
                    const t = kernel_module.syscall(k, kernel_module.Syscall.GetTime, null);
                    std.debug.print("Kernel time: {}\n", .{t});
                } else {
                    std.debug.print("No kernel available to query time.\n", .{});
                }
            } else if (std.mem.eql(u8, line, "history")) {
                var idx: usize = 0;
                for (self.history.items) |h| {
                    std.debug.print("{d}: {s}\n", .{ idx, h });
                    idx += 1;
                }
            } else {
                std.debug.print("Unknown shell command: {s}\n", .{line});
            }

            self.allocator.free(line);
        }
    }
};

pub fn runShell(allocator: std.mem.Allocator, system: *system_module.System, kernel: ?*kernel_module.Kernel) !void {
    var shell = Shell.init(allocator, "sh> ");
    defer shell.deinit();
    try shell.run(system, kernel);
}
