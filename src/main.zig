const std = @import("std");
const kogi = @import("kogi");

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    // Initialize KOGI System (core OS)
    var system = kogi.System.init(allocator);
    defer system.deinit();

    // Start interactive CLI shell
    try kogi.startCLI(allocator, &system);
}
