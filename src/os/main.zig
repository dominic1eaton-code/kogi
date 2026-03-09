const std = @import("std");
const kogi = @import("kogi");

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var kernel = kogi.Kernel.init(allocator);
    defer kernel.deinit();

    var system = kogi.System.init(allocator, &kernel);
    defer system.deinit();

    try kogi.startCLI(allocator, &system);
}
