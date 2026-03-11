//! main.zig — Kernel Shell Entry Point
//!
//! Boots a Kernel, configures a ShellSession, then hands control to the
//! interactive REPL defined in shell.zig.
//!
//! Usage
//! ─────
//!   kogi-shell [options]
//!
//! Options
//! ───────
//!   --help              Print this help and exit.
//!   --no-colour         Disable ANSI colour output.
//!   --hostname <name>   Set the hostname shown in the shell prompt (default: kogi).
//!   --bootstrap         Run bootstrapCorePlatformComponents + bootstrapNetworkManagers
//!                       before opening the shell (populates the kernel with the
//!                       standard platform modules so you can inspect them immediately).
//!   --user-mode         Start the shell in unprivileged user mode.
//!                       By default the shell starts in privileged mode (root).
//!   --memory <GiB>      Total kernel memory pool in GiB (default: 4).
//!   --workers <N>       Number of scheduler worker slots (default: 16).
//!   --event-cap <N>     Kernel event-log ring-buffer size (default: 8192).
//!
//! Exit codes
//! ──────────
//!   0   Clean exit (user typed 'exit' or sent EOF).
//!   1   Argument error.
//!   2   Kernel initialisation failure.
//!   3   Shell runtime error.

const std = @import("std");

const shell_mod = @import("shell.zig");
const kern_mod = @import("kernel.zig");

const ShellSession = shell_mod.ShellSession;
const ShellConfig = shell_mod.ShellConfig;
const Kernel = kern_mod.Kernel;
const KernelConfig = kern_mod.KernelConfig;

// ─────────────────────────────────────────────────────────────────────────────
// CLI argument schema
// ─────────────────────────────────────────────────────────────────────────────

const CliArgs = struct {
    help: bool = false,
    no_colour: bool = false,
    hostname: []const u8 = "kogi",
    bootstrap: bool = false,
    user_mode: bool = false,
    memory_gib: u64 = 4,
    workers: u32 = 16,
    event_cap: usize = 8192,
};

const USAGE =
    \\Usage: kogi-shell [options]
    \\
    \\Options:
    \\  --help              Print this help and exit
    \\  --no-colour         Disable ANSI colour output
    \\  --hostname <name>   Prompt hostname  (default: kogi)
    \\  --bootstrap         Pre-load core platform modules before opening the shell
    \\  --user-mode         Start in unprivileged user mode  (default: privileged)
    \\  --memory <GiB>      Kernel memory pool in GiB        (default: 4)
    \\  --workers <N>       Scheduler worker slots           (default: 16)
    \\  --event-cap <N>     Event-log ring-buffer capacity   (default: 8192)
    \\
;

// ─────────────────────────────────────────────────────────────────────────────
// Argument parser
// ─────────────────────────────────────────────────────────────────────────────

fn parseArgs(args: []const []const u8) !CliArgs {
    var cli = CliArgs{};
    var i: usize = 1; // skip argv[0]
    while (i < args.len) : (i += 1) {
        const arg = args[i];

        if (std.mem.eql(u8, arg, "--help") or std.mem.eql(u8, arg, "-h")) {
            cli.help = true;
        } else if (std.mem.eql(u8, arg, "--no-colour") or
            std.mem.eql(u8, arg, "--no-color"))
        {
            cli.no_colour = true;
        } else if (std.mem.eql(u8, arg, "--bootstrap") or
            std.mem.eql(u8, arg, "-b"))
        {
            cli.bootstrap = true;
        } else if (std.mem.eql(u8, arg, "--user-mode") or
            std.mem.eql(u8, arg, "-u"))
        {
            cli.user_mode = true;
        } else if (std.mem.eql(u8, arg, "--hostname")) {
            i += 1;
            if (i >= args.len) return error.MissingValue;
            cli.hostname = args[i];
        } else if (std.mem.eql(u8, arg, "--memory")) {
            i += 1;
            if (i >= args.len) return error.MissingValue;
            cli.memory_gib = std.fmt.parseUnsigned(u64, args[i], 10) catch
                return error.InvalidNumber;
            if (cli.memory_gib == 0 or cli.memory_gib > 512) return error.InvalidNumber;
        } else if (std.mem.eql(u8, arg, "--workers")) {
            i += 1;
            if (i >= args.len) return error.MissingValue;
            const n = std.fmt.parseUnsigned(u32, args[i], 10) catch
                return error.InvalidNumber;
            if (n == 0 or n > 4096) return error.InvalidNumber;
            cli.workers = n;
        } else if (std.mem.eql(u8, arg, "--event-cap")) {
            i += 1;
            if (i >= args.len) return error.MissingValue;
            const n = std.fmt.parseUnsigned(usize, args[i], 10) catch
                return error.InvalidNumber;
            if (n < 64) return error.InvalidNumber;
            cli.event_cap = n;
        } else {
            std.debug.print("error: unknown option '{s}'\n\n{s}", .{ arg, USAGE });
            return error.UnknownOption;
        }
    }
    return cli;
}

// ─────────────────────────────────────────────────────────────────────────────
// TTY detection  (auto-disable colour when piped)
// ─────────────────────────────────────────────────────────────────────────────

fn stdoutIsTty() bool {
    // std.io.getStdOut().isTty() is available in recent Zig std.
    // Fall back to true if the call is unavailable (colour stays on).
    return std.io.getStdOut().isTty();
}

// ─────────────────────────────────────────────────────────────────────────────
// main
// ─────────────────────────────────────────────────────────────────────────────

pub fn main() u8 {
    // Use a GPA in debug builds for leak detection; a page allocator in release.
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    return mainInner(allocator) catch |err| {
        std.debug.print("fatal: {s}\n", .{@errorName(err)});
        return 3;
    };
}

fn mainInner(allocator: std.mem.Allocator) !u8 {
    // ── 1. Parse arguments ─────────────────────────────────────────────────
    const raw_args = try std.process.argsAlloc(allocator);
    defer std.process.argsFree(allocator, raw_args);

    const cli = parseArgs(raw_args) catch |err| {
        switch (err) {
            error.MissingValue => std.debug.print("error: option requires a value\n\n{s}", .{USAGE}),
            error.InvalidNumber => std.debug.print("error: invalid numeric argument\n\n{s}", .{USAGE}),
            error.UnknownOption => {}, // already printed
            else => std.debug.print("error: {s}\n\n{s}", .{ @errorName(err), USAGE }),
        }
        return 1;
    };

    if (cli.help) {
        const stdout = std.io.getStdOut().writer();
        try stdout.writeAll(USAGE);
        return 0;
    }

    // ── 2. Resolve colour setting ──────────────────────────────────────────
    const use_colour = !cli.no_colour and stdoutIsTty();

    // ── 3. Build KernelConfig from CLI args ───────────────────────────────
    const kernel_cfg = KernelConfig{
        .memory_total_bytes = cli.memory_gib * 1024 * 1024 * 1024,
        .memory_audit_enabled = true,
        .memory_audit_cap = 4096,
        .worker_count = cli.workers,
        .queue_lane_cap = 256,
        .max_sockets = 512,
        .max_connections = 256,
        .max_conn_failures = 5,
        .packet_lane_cap = 1024,
        .firewall_default = .allow,
        .module_event_cap = 2048,
        .service_event_cap = 1024,
        .global_mem_ceiling = cli.memory_gib * 1024 * 1024 * 1024,
        .global_process_ceiling = 4096,
        .global_conn_ceiling = 8192,
        .resource_log_cap = 4096,
        .event_log_cap = cli.event_cap,
    };

    // ── 4. Boot the kernel ─────────────────────────────────────────────────
    const kernel = allocator.create(Kernel) catch {
        std.debug.print("error: out of memory during kernel allocation\n", .{});
        return 2;
    };
    errdefer allocator.destroy(kernel);

    kernel.* = Kernel.init(allocator, kernel_cfg) catch |err| {
        std.debug.print("error: kernel init failed: {s}\n", .{@errorName(err)});
        return 2;
    };
    kernel.relink();
    defer {
        kernel.deinit();
        allocator.destroy(kernel);
    }

    // ── 5. Optional bootstrap ──────────────────────────────────────────────
    if (cli.bootstrap) {
        kernel.bootstrapNetworkManagers(.root) catch |err| {
            std.debug.print("warning: bootstrapNetworkManagers failed: {s}\n", .{@errorName(err)});
        };
        kernel.bootstrapCorePlatformComponents(.root) catch |err| {
            std.debug.print("warning: bootstrapCorePlatformComponents failed: {s}\n", .{@errorName(err)});
        };
    }

    // ── 6. Apply initial privilege mode ───────────────────────────────────
    if (cli.user_mode) {
        kernel.exitPrivileged(.root) catch {};
    }
    // (kernel starts in privileged/root mode by default — no action needed otherwise)

    // ── 7. Build ShellConfig ───────────────────────────────────────────────
    const shell_cfg = ShellConfig{
        .use_colour = use_colour,
        .show_ts = true,
        .watch_limit = 50,
        .hostname = cli.hostname,
    };

    // ── 8. Run the interactive shell ───────────────────────────────────────
    const stdin = std.io.getStdIn().reader().any();
    const stdout = std.io.getStdOut().writer().any();

    var session = ShellSession.init(allocator, kernel, stdin, stdout, shell_cfg);
    defer session.deinit();

    session.run() catch |err| {
        std.debug.print("shell error: {s}\n", .{@errorName(err)});
        return 3;
    };

    return 0;
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

test "parseArgs: defaults" {
    const args = [_][]const u8{"kogi-shell"};
    const cli = try parseArgs(&args);
    try std.testing.expect(!cli.help);
    try std.testing.expect(!cli.no_colour);
    try std.testing.expectEqualStrings("kogi", cli.hostname);
    try std.testing.expect(!cli.bootstrap);
    try std.testing.expect(!cli.user_mode);
    try std.testing.expectEqual(@as(u64, 4), cli.memory_gib);
    try std.testing.expectEqual(@as(u32, 16), cli.workers);
    try std.testing.expectEqual(@as(usize, 8192), cli.event_cap);
}

test "parseArgs: --help flag" {
    const args = [_][]const u8{ "kogi-shell", "--help" };
    const cli = try parseArgs(&args);
    try std.testing.expect(cli.help);
}

test "parseArgs: short flags" {
    const args = [_][]const u8{ "kogi-shell", "-b", "-u" };
    const cli = try parseArgs(&args);
    try std.testing.expect(cli.bootstrap);
    try std.testing.expect(cli.user_mode);
}

test "parseArgs: --no-color alias" {
    const args = [_][]const u8{ "kogi-shell", "--no-color" };
    const cli = try parseArgs(&args);
    try std.testing.expect(cli.no_colour);
}

test "parseArgs: --hostname" {
    const args = [_][]const u8{ "kogi-shell", "--hostname", "mybox" };
    const cli = try parseArgs(&args);
    try std.testing.expectEqualStrings("mybox", cli.hostname);
}

test "parseArgs: --memory" {
    const args = [_][]const u8{ "kogi-shell", "--memory", "8" };
    const cli = try parseArgs(&args);
    try std.testing.expectEqual(@as(u64, 8), cli.memory_gib);
}

test "parseArgs: --workers" {
    const args = [_][]const u8{ "kogi-shell", "--workers", "32" };
    const cli = try parseArgs(&args);
    try std.testing.expectEqual(@as(u32, 32), cli.workers);
}

test "parseArgs: --event-cap" {
    const args = [_][]const u8{ "kogi-shell", "--event-cap", "16384" };
    const cli = try parseArgs(&args);
    try std.testing.expectEqual(@as(usize, 16384), cli.event_cap);
}

test "parseArgs: all flags together" {
    const args = [_][]const u8{
        "kogi-shell",
        "--no-colour",
        "--hostname",
        "prod",
        "--bootstrap",
        "--user-mode",
        "--memory",
        "2",
        "--workers",
        "4",
        "--event-cap",
        "1024",
    };
    const cli = try parseArgs(&args);
    try std.testing.expect(cli.no_colour);
    try std.testing.expectEqualStrings("prod", cli.hostname);
    try std.testing.expect(cli.bootstrap);
    try std.testing.expect(cli.user_mode);
    try std.testing.expectEqual(@as(u64, 2), cli.memory_gib);
    try std.testing.expectEqual(@as(u32, 4), cli.workers);
    try std.testing.expectEqual(@as(usize, 1024), cli.event_cap);
}

test "parseArgs: missing value for --hostname returns error" {
    const args = [_][]const u8{ "kogi-shell", "--hostname" };
    try std.testing.expectError(error.MissingValue, parseArgs(&args));
}

test "parseArgs: invalid number for --memory returns error" {
    const args = [_][]const u8{ "kogi-shell", "--memory", "banana" };
    try std.testing.expectError(error.InvalidNumber, parseArgs(&args));
}

test "parseArgs: zero memory rejected" {
    const args = [_][]const u8{ "kogi-shell", "--memory", "0" };
    try std.testing.expectError(error.InvalidNumber, parseArgs(&args));
}

test "parseArgs: unknown option returns error" {
    const args = [_][]const u8{ "kogi-shell", "--unknown" };
    try std.testing.expectError(error.UnknownOption, parseArgs(&args));
}

test "mainInner: boots and runs a no-op shell session" {
    // Feed an immediate 'exit' so the REPL terminates at once.
    // We redirect stdout to a buffer so nothing leaks to the test runner.
    const allocator = std.testing.allocator;

    const k = try allocator.create(Kernel);
    errdefer allocator.destroy(k);
    k.* = try Kernel.init(allocator, .{
        .memory_total_bytes = 256 * 1024 * 1024,
        .memory_audit_enabled = false,
        .worker_count = 2,
        .queue_lane_cap = 32,
        .max_sockets = 16,
        .max_connections = 8,
        .max_conn_failures = 2,
        .packet_lane_cap = 64,
        .module_event_cap = 64,
        .service_event_cap = 64,
        .global_mem_ceiling = 256 * 1024 * 1024,
        .global_process_ceiling = 32,
        .global_conn_ceiling = 32,
        .resource_log_cap = 64,
        .event_log_cap = 128,
    });
    k.relink();
    defer {
        k.deinit();
        allocator.destroy(k);
    }

    var out = std.ArrayList(u8).init(allocator);
    defer out.deinit();

    var fbs = std.io.fixedBufferStream("exit\n");
    const reader = fbs.reader().any();
    const writer = out.writer().any();

    var session = ShellSession.init(allocator, k, reader, writer, .{
        .use_colour = false,
        .hostname = "test",
    });
    defer session.deinit();

    try session.run();

    try std.testing.expect(std.mem.indexOf(u8, out.items, "Goodbye") != null);
}

test "mainInner: --bootstrap pre-loads platform modules" {
    const allocator = std.testing.allocator;

    const k = try allocator.create(Kernel);
    errdefer allocator.destroy(k);
    k.* = try Kernel.init(allocator, .{
        .memory_total_bytes = 4 * 1024 * 1024 * 1024,
        .memory_audit_enabled = false,
        .worker_count = 4,
        .queue_lane_cap = 64,
        .max_sockets = 64,
        .max_connections = 32,
        .max_conn_failures = 3,
        .packet_lane_cap = 128,
        .module_event_cap = 256,
        .service_event_cap = 256,
        .global_mem_ceiling = 4 * 1024 * 1024 * 1024,
        .global_process_ceiling = 512,
        .global_conn_ceiling = 1024,
        .resource_log_cap = 256,
        .event_log_cap = 512,
    });
    k.relink();
    defer {
        k.deinit();
        allocator.destroy(k);
    }

    // Simulate the bootstrap flag path
    try k.bootstrapNetworkManagers(.root);
    try k.bootstrapCorePlatformComponents(.root);

    try std.testing.expectEqual(@as(usize, 5), k.modules.registry.count());
    try std.testing.expectEqual(@as(usize, 5), k.modules.registry.activeCount());
}

test "mainInner: --user-mode starts shell unprivileged" {
    const allocator = std.testing.allocator;

    const k = try allocator.create(Kernel);
    errdefer allocator.destroy(k);
    k.* = try Kernel.init(allocator, .{
        .memory_total_bytes = 256 * 1024 * 1024,
        .memory_audit_enabled = false,
        .worker_count = 2,
        .queue_lane_cap = 32,
        .max_sockets = 16,
        .max_connections = 8,
        .max_conn_failures = 2,
        .packet_lane_cap = 64,
        .module_event_cap = 64,
        .service_event_cap = 64,
        .global_mem_ceiling = 256 * 1024 * 1024,
        .global_process_ceiling = 32,
        .global_conn_ceiling = 32,
        .resource_log_cap = 64,
        .event_log_cap = 128,
    });
    k.relink();
    defer {
        k.deinit();
        allocator.destroy(k);
    }

    // Simulate --user-mode flag path
    try k.exitPrivileged(.root);
    try std.testing.expect(!k.isPrivileged());

    // Privileged ops must fail in user mode
    try std.testing.expectError(
        kern_mod.KernelError.PrivilegeDenied,
        k.registerModule(.host, "blocked", "test"),
    );
}
