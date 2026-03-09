//!
const std = @import("std");
const system = @import("system.zig");

pub fn run_system() !void {
    try system.System.run();
}
