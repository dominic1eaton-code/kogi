const std = @import("std");
const kernel_mod = @import("kernel.zig");

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();

    var kernel = kernel_mod.Kernel.init(gpa.allocator());
    defer kernel.deinit();

    try kernel.bootstrapCorePlatformComponents(.host);
    try kernel.registerModule(.host, "kogi.host", "host");
    try kernel.registerModule(.host, "kogi.server", "backend");
    try kernel.bootstrapOfficeModule(.host);

    try kernel.allocateMemory(.host, 64 * 1024 * 1024);
    _ = try kernel.spawnProcess(.host, "module-supervisor", .host);

    const now = std.time.milliTimestamp();
    try kernel.schedule(.host, "heartbeat", "kernel.heartbeat", "{}", now, 1000);
    _ = try kernel.tick(.host, now);

    const s = kernel.stats();
    std.debug.print(
        "kogi-kernel mvp: modules={d} isolated={d} components={d} events={d} mem={d}/{d} net(in={d},out={d}) mode={s}\n",
        .{
            s.module_count,
            s.isolated_module_count,
            s.component_count,
            s.event_count,
            s.memory_used_bytes,
            s.memory_total_bytes,
            s.network_ingress_bytes,
            s.network_egress_bytes,
            @tagName(s.mode),
        },
    );
}
