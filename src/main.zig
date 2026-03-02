//! Main entry point for the application.
//! @license MIT
//! @copyright 2026-present Wolof.io Software Studios, Inc.
//! @author Wolof.io Software Studios, Inc.
//! @version 1.0.0
const std = @import("std");

pub fn main() !void {
    const stdout = std.io.getStdOut().writer();
    try stdout.print("Hello, World!\n", .{});
}
