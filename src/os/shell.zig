const std = @import("std");
const terminal_module = @import("terminal.zig");
const kernel_module = @import("kernel.zig");
const system_module = @import("system.zig");
const identity_module = @import("identity.zig");

pub const CommandDispatcher = struct {
    context: *anyopaque,
    dispatch: *const fn (context: *anyopaque, line: []const u8) anyerror!bool,
};

pub const Shell = struct {
    allocator: std.mem.Allocator,
    term: terminal_module.Terminal,
    history: std.array_list.Managed([]const u8),

    pub fn init(allocator: std.mem.Allocator, prompt: []const u8) Shell {
        return .{
            .allocator = allocator,
            .term = terminal_module.Terminal.init(allocator, prompt),
            .history = std.array_list.Managed([]const u8).init(allocator),
        };
    }

    pub fn deinit(self: *Shell) void {
        for (self.history.items) |entry| self.allocator.free(entry);
        self.history.deinit();
    }

    pub fn run(
        self: *Shell,
        system: *system_module.System,
        kernel: ?*kernel_module.Kernel,
        fallback: ?CommandDispatcher,
    ) !void {
        while (true) {
            try self.term.writePrompt();
            const raw_line = self.term.readLine() catch |err| switch (err) {
                error.EndOfStream => break,
                else => return err,
            };
            defer self.allocator.free(raw_line);

            const line = trimWhitespace(raw_line);
            if (line.len == 0) continue;

            const saved = try self.allocator.dupe(u8, line);
            try self.history.append(saved);

            if (try self.handleShellCommand(line, system, kernel)) |keep| {
                if (!keep) break;
                continue;
            }

            if (fallback) |dispatcher| {
                const keep = try dispatcher.dispatch(dispatcher.context, line);
                if (!keep) break;
            } else {
                std.debug.print("Unknown command: {s}\n", .{line});
            }
        }
    }

    fn handleShellCommand(
        self: *Shell,
        line: []const u8,
        system: *system_module.System,
        kernel: ?*kernel_module.Kernel,
    ) !?bool {
        const split = splitCommand(line);
        const cmd = split.cmd;
        const args = split.args;

        if (std.mem.eql(u8, cmd, "exit") or std.mem.eql(u8, cmd, "quit")) {
            std.debug.print("Goodbye.\n", .{});
            return false;
        }
        if (std.mem.eql(u8, cmd, "help")) {
            self.printHelp();
            return true;
        }
        if (std.mem.eql(u8, cmd, "features")) {
            self.printFeatures();
            return true;
        }
        if (std.mem.eql(u8, cmd, "history")) {
            for (self.history.items, 0..) |entry, idx| {
                std.debug.print("{d}: {s}\n", .{ idx, entry });
            }
            return true;
        }
        if (std.mem.eql(u8, cmd, "status")) {
            self.printStatus(system);
            return true;
        }
        if (std.mem.eql(u8, cmd, "syscalls")) {
            if (kernel) |_| {
                const names = kernel_module.syscallNames();
                for (names) |name| std.debug.print("{s}\n", .{name});
            } else {
                std.debug.print("No kernel available.\n", .{});
            }
            return true;
        }
        if (std.mem.eql(u8, cmd, "time")) {
            if (kernel) |k| {
                const t = kernel_module.syscall(k, kernel_module.Syscall.GetTime, null);
                std.debug.print("Kernel time: {}\n", .{t});
            } else {
                std.debug.print("No kernel available.\n", .{});
            }
            return true;
        }
        if (std.mem.eql(u8, cmd, "identity")) {
            try self.handleIdentity(system, args);
            return true;
        }
        if (std.mem.eql(u8, cmd, "task")) {
            try self.handleTask(system, args);
            return true;
        }
        if (std.mem.eql(u8, cmd, "engagement")) {
            try self.handleEngagement(system, args);
            return true;
        }
        if (std.mem.eql(u8, cmd, "workspace") or std.mem.eql(u8, cmd, "portfolio")) {
            try self.handleWorkspace(system, args);
            return true;
        }
        if (std.mem.eql(u8, cmd, "database") or std.mem.eql(u8, cmd, "db")) {
            try self.handleDatabase(system, args);
            return true;
        }
        return null;
    }

    fn printHelp(self: *Shell) void {
        _ = self;
        std.debug.print("KOGI shell commands:\n", .{});
        std.debug.print("  help, features, status, history, exit|quit\n", .{});
        std.debug.print("  identity <add|list> ...\n", .{});
        std.debug.print("  task <add|list> ...\n", .{});
        std.debug.print("  engagement <create|list> ...\n", .{});
        std.debug.print("  workspace|portfolio <init|list> ...\n", .{});
        std.debug.print("  database|db <create|list> ...\n", .{});
        std.debug.print("  user ..., provider ..., demo, stats ...\n", .{});
    }

    fn printFeatures(self: *Shell) void {
        _ = self;
        std.debug.print("KOGI namespaces:\n", .{});
        std.debug.print("  identity user provider task engagement\n", .{});
        std.debug.print("  workspace portfolio content design directory-book\n", .{});
        std.debug.print("  database security logging events state\n", .{});
        std.debug.print("  process memory scheduler networking cluster trace\n", .{});
    }

    fn printStatus(self: *Shell, system: *system_module.System) void {
        _ = self;
        const collections = if (system.workspace_manager.workspace) |w| w.collections.items.len else 0;
        const workspace_items = if (system.workspace_manager.workspace) |w| w.items.items.len else 0;
        std.debug.print("Status:\n", .{});
        std.debug.print("  identities={d} users={d} providers={d}\n", .{
            system.getIdentities().len,
            system.getUsers().len,
            system.getProviders().len,
        });
        std.debug.print("  tasks={d} engagements={d}\n", .{
            system.getTasks().len,
            system.getEngagements().len,
        });
        std.debug.print("  workspace_collections={d} workspace_items={d}\n", .{ collections, workspace_items });
        std.debug.print("  databases={d} tables={d} records={d}\n", .{
            system.getDatabases().len,
            system.getTables().len,
            system.getRecords().len,
        });
        std.debug.print("  content={d} notes={d} designs={d}\n", .{
            system.getContentItems().len,
            system.getNotes().len,
            system.getDesigns().len,
        });
    }

    fn handleIdentity(self: *Shell, system: *system_module.System, args_raw: []const u8) !void {
        _ = self;
        var it = std.mem.tokenizeAny(u8, args_raw, " \t");
        const action = it.next() orelse {
            std.debug.print("Usage: identity <add|list> ...\n", .{});
            return;
        };

        if (std.mem.eql(u8, action, "list")) {
            const identities = system.getIdentities();
            if (identities.len == 0) {
                std.debug.print("No identities.\n", .{});
                return;
            }
            for (identities) |identity| {
                std.debug.print("id={} name={s} type={s} rate={d:.2}\n", .{
                    identity.id,
                    identity.name,
                    @tagName(identity.identity_type),
                    identity.hourly_rate,
                });
            }
            return;
        }

        if (std.mem.eql(u8, action, "add")) {
            const name = it.next() orelse {
                std.debug.print("Usage: identity add <name> <email> <hourly_rate> <type>\n", .{});
                return;
            };
            const email = it.next() orelse {
                std.debug.print("Usage: identity add <name> <email> <hourly_rate> <type>\n", .{});
                return;
            };
            const rate_raw = it.next() orelse {
                std.debug.print("Usage: identity add <name> <email> <hourly_rate> <type>\n", .{});
                return;
            };
            const type_raw = it.next() orelse {
                std.debug.print("Usage: identity add <name> <email> <hourly_rate> <type>\n", .{});
                return;
            };
            const rate = std.fmt.parseFloat(f32, rate_raw) catch {
                std.debug.print("Invalid rate: {s}\n", .{rate_raw});
                return;
            };
            const identity_type = std.meta.stringToEnum(identity_module.IdentityType, type_raw) orelse {
                std.debug.print("Invalid type: {s}\n", .{type_raw});
                return;
            };
            const id = try system.createIdentity(name, email, rate, identity_type);
            std.debug.print("Identity created: id={}\n", .{id});
            return;
        }

        std.debug.print("Usage: identity <add|list> ...\n", .{});
    }

    fn handleTask(self: *Shell, system: *system_module.System, args_raw: []const u8) !void {
        _ = self;
        var it = std.mem.tokenizeAny(u8, args_raw, " \t");
        const action = it.next() orelse {
            std.debug.print("Usage: task <add|list> ...\n", .{});
            return;
        };

        if (std.mem.eql(u8, action, "list")) {
            const tasks = system.getTasks();
            if (tasks.len == 0) {
                std.debug.print("No tasks.\n", .{});
                return;
            }
            for (tasks) |task| {
                std.debug.print("id={} title={s} budget={d:.2} status={s}\n", .{
                    task.id,
                    task.title,
                    task.budget,
                    if (task.completed) "completed" else "open",
                });
            }
            return;
        }

        if (std.mem.eql(u8, action, "add")) {
            const title = it.next() orelse {
                std.debug.print("Usage: task add <title> <description> <budget> <deadline_unix>\n", .{});
                return;
            };
            const description = it.next() orelse {
                std.debug.print("Usage: task add <title> <description> <budget> <deadline_unix>\n", .{});
                return;
            };
            const budget_raw = it.next() orelse {
                std.debug.print("Usage: task add <title> <description> <budget> <deadline_unix>\n", .{});
                return;
            };
            const deadline_raw = it.next() orelse {
                std.debug.print("Usage: task add <title> <description> <budget> <deadline_unix>\n", .{});
                return;
            };
            const budget = std.fmt.parseFloat(f32, budget_raw) catch {
                std.debug.print("Invalid budget: {s}\n", .{budget_raw});
                return;
            };
            const deadline = std.fmt.parseInt(i64, deadline_raw, 10) catch {
                std.debug.print("Invalid deadline: {s}\n", .{deadline_raw});
                return;
            };
            const id = try system.postTask(title, description, budget, deadline);
            std.debug.print("Task created: id={}\n", .{id});
            return;
        }

        std.debug.print("Usage: task <add|list> ...\n", .{});
    }

    fn handleEngagement(self: *Shell, system: *system_module.System, args_raw: []const u8) !void {
        _ = self;
        var it = std.mem.tokenizeAny(u8, args_raw, " \t");
        const action = it.next() orelse {
            std.debug.print("Usage: engagement <create|list> ...\n", .{});
            return;
        };

        if (std.mem.eql(u8, action, "list")) {
            const engagements = system.getEngagements();
            if (engagements.len == 0) {
                std.debug.print("No engagements.\n", .{});
                return;
            }
            for (engagements) |engagement| {
                std.debug.print("id={} identity={} task={} hours={d:.2} status={s}\n", .{
                    engagement.id,
                    engagement.identity_id,
                    engagement.task_id,
                    engagement.hours_worked,
                    @tagName(engagement.status),
                });
            }
            return;
        }

        if (std.mem.eql(u8, action, "create")) {
            const identity_raw = it.next() orelse {
                std.debug.print("Usage: engagement create <identity_id> <task_id> <start_unix> <hourly_rate>\n", .{});
                return;
            };
            const task_raw = it.next() orelse {
                std.debug.print("Usage: engagement create <identity_id> <task_id> <start_unix> <hourly_rate>\n", .{});
                return;
            };
            const start_raw = it.next() orelse {
                std.debug.print("Usage: engagement create <identity_id> <task_id> <start_unix> <hourly_rate>\n", .{});
                return;
            };
            const rate_raw = it.next() orelse {
                std.debug.print("Usage: engagement create <identity_id> <task_id> <start_unix> <hourly_rate>\n", .{});
                return;
            };

            const identity_id = std.fmt.parseInt(u32, identity_raw, 10) catch {
                std.debug.print("Invalid identity id: {s}\n", .{identity_raw});
                return;
            };
            const task_id = std.fmt.parseInt(u32, task_raw, 10) catch {
                std.debug.print("Invalid task id: {s}\n", .{task_raw});
                return;
            };
            const start = std.fmt.parseInt(i64, start_raw, 10) catch {
                std.debug.print("Invalid start date: {s}\n", .{start_raw});
                return;
            };
            const rate = std.fmt.parseFloat(f32, rate_raw) catch {
                std.debug.print("Invalid hourly rate: {s}\n", .{rate_raw});
                return;
            };

            const id = try system.createEngagement(identity_id, task_id, start, rate);
            std.debug.print("Engagement created: id={}\n", .{id});
            return;
        }

        std.debug.print("Usage: engagement <create|list> ...\n", .{});
    }

    fn handleWorkspace(self: *Shell, system: *system_module.System, args_raw: []const u8) !void {
        _ = self;
        var it = std.mem.tokenizeAny(u8, args_raw, " \t");
        const action = it.next() orelse {
            std.debug.print("Usage: workspace <init|list> ...\n", .{});
            return;
        };

        if (std.mem.eql(u8, action, "list")) {
            if (system.workspace_manager.workspace) |workspace| {
                std.debug.print("workspace id={} name={s} collections={d} items={d}\n", .{
                    workspace.id,
                    workspace.name,
                    workspace.collections.items.len,
                    workspace.items.items.len,
                });
            } else {
                std.debug.print("No workspace initialized.\n", .{});
            }
            return;
        }

        if (std.mem.eql(u8, action, "init")) {
            const name = it.next() orelse {
                std.debug.print("Usage: workspace init <name> <description> <category>\n", .{});
                return;
            };
            const description = it.next() orelse {
                std.debug.print("Usage: workspace init <name> <description> <category>\n", .{});
                return;
            };
            const category = it.next() orelse {
                std.debug.print("Usage: workspace init <name> <description> <category>\n", .{});
                return;
            };
            try system.workspace_manager.createWorkspace(name, description, category);
            std.debug.print("Workspace created: {s}\n", .{name});
            return;
        }

        std.debug.print("Usage: workspace <init|list> ...\n", .{});
    }

    fn handleDatabase(self: *Shell, system: *system_module.System, args_raw: []const u8) !void {
        _ = self;
        var it = std.mem.tokenizeAny(u8, args_raw, " \t");
        const action = it.next() orelse {
            std.debug.print("Usage: database <create|list> ...\n", .{});
            return;
        };

        if (std.mem.eql(u8, action, "list")) {
            const dbs = system.getDatabases();
            if (dbs.len == 0) {
                std.debug.print("No databases.\n", .{});
                return;
            }
            for (dbs) |db| {
                std.debug.print("id={} name={s} active={s}\n", .{
                    db.id,
                    db.name,
                    if (db.active) "true" else "false",
                });
            }
            return;
        }

        if (std.mem.eql(u8, action, "create")) {
            const name = it.next() orelse {
                std.debug.print("Usage: database create <name> <description>\n", .{});
                return;
            };
            const description = it.next() orelse {
                std.debug.print("Usage: database create <name> <description>\n", .{});
                return;
            };
            const id = try system.createDatabase(name, description);
            std.debug.print("Database created: id={}\n", .{id});
            return;
        }

        std.debug.print("Usage: database <create|list> ...\n", .{});
    }
};

fn trimWhitespace(input: []const u8) []const u8 {
    var start: usize = 0;
    var end: usize = input.len;
    while (start < end and (input[start] == ' ' or input[start] == '\t')) : (start += 1) {}
    while (end > start and (input[end - 1] == ' ' or input[end - 1] == '\t')) : (end -= 1) {}
    return input[start..end];
}

fn splitCommand(line: []const u8) struct { cmd: []const u8, args: []const u8 } {
    var cmd_end: usize = 0;
    while (cmd_end < line.len and line[cmd_end] != ' ' and line[cmd_end] != '\t') : (cmd_end += 1) {}
    var args_start = cmd_end;
    while (args_start < line.len and (line[args_start] == ' ' or line[args_start] == '\t')) : (args_start += 1) {}
    return .{ .cmd = line[0..cmd_end], .args = line[args_start..] };
}

pub fn runShell(
    allocator: std.mem.Allocator,
    system: *system_module.System,
    kernel: ?*kernel_module.Kernel,
    fallback: ?CommandDispatcher,
) !void {
    var shell = Shell.init(allocator, "kogi> ");
    defer shell.deinit();
    try shell.run(system, kernel, fallback);
}
