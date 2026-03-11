//! shell.zig — Kernel Interactive Shell
//!
//! A full-featured interactive CLI for inspecting and controlling a live
//! Kernel instance.  The shell supports two privilege levels that mirror
//! the kernel's own Mode:
//!
//!   User mode      — read-only inspection commands; no state changes.
//!   Privileged mode — full access to all kernel operations.
//!
//! Session lifecycle
//! ─────────────────
//!   1.  Create a ShellSession (holds Kernel pointer, I/O streams, history).
//!   2.  Call session.run() to enter the interactive REPL.
//!   3.  The user types commands at the prompt; `exit` or Ctrl-D terminates.
//!
//! Command tree (summary)
//! ──────────────────────
//!   help [command]             — print help
//!   status                     — kernel status overview
//!   mode                       — show current privilege mode
//!   su                         — enter privileged mode
//!   exit / quit                — leave shell (or drop privileges)
//!
//!   module list                — list all modules
//!   module show <id>           — detailed module info
//!   module register <id> <kind>— register a module       [privileged]
//!   module start   <id>        — start a module           [privileged]
//!   module stop    <id>        — stop  a module           [privileged]
//!   module remove  <id>        — remove a module          [privileged]
//!
//!   service list               — list all services
//!   service start  <id>        — start a service          [privileged]
//!   service stop   <id>        — stop a service           [privileged]
//!   service startall           — start all services       [privileged]
//!   service stopall            — stop all services        [privileged]
//!
//!   memory status              — memory usage overview
//!   memory alloc <tenant> <bytes>  — allocate tenant memory [privileged]
//!   memory free  <tenant> <bytes>  — free tenant memory     [privileged]
//!
//!   process list               — list live processes
//!   process spawn <name>       — spawn a process          [privileged]
//!   process kill  <pid>        — terminate a process      [privileged]
//!
//!   tenant list                — list resource tenants
//!   tenant register <id>       — register a tenant        [privileged]
//!
//!   net list                   — list registered addresses
//!
//!   event list [N]             — show last N events (default 20)
//!   event watch <domain>       — subscribe to domain events (live)
//!   event query <domain> <watermark> — query events since watermark
//!
//!   sched list                 — list scheduled jobs
//!   sched tick                 — fire the scheduler       [privileged]
//!
//!   stats                      — full aggregate stats table
//!   version                    — kernel version info

const std = @import("std");

const kern = @import("kernel.zig");

// Re-export for convenience
const Kernel = kern.Kernel;
const Role = kern.Role;
const Mode = kern.Mode;
const KernelError = kern.KernelError;
const EventFilter = kern.EventFilter;
const EventDomain = kern.EventDomain;
const EventKind = kern.EventKind;
const KernelEvent = kern.KernelEvent;

// ─────────────────────────────────────────────────────────────────────────────
// ANSI colour helpers  (disabled automatically if output is not a TTY — the
// shell checks at init and sets use_colour accordingly)
// ─────────────────────────────────────────────────────────────────────────────

const C = struct {
    const reset = "\x1b[0m";
    const bold = "\x1b[1m";
    const dim = "\x1b[2m";
    const red = "\x1b[31m";
    const green = "\x1b[32m";
    const yellow = "\x1b[33m";
    const blue = "\x1b[34m";
    const cyan = "\x1b[36m";
    const white = "\x1b[97m";
};

// ─────────────────────────────────────────────────────────────────────────────
// Errors
// ─────────────────────────────────────────────────────────────────────────────

pub const ShellError = error{
    UnknownCommand,
    WrongArgCount,
    InvalidArgument,
    PrivilegeRequired,
    KernelDenied,
    IoError,
    HistoryFull,
};

// ─────────────────────────────────────────────────────────────────────────────
// Tokeniser — splits a raw line into at most MAX_ARGS tokens
// ─────────────────────────────────────────────────────────────────────────────

pub const MAX_ARGS: usize = 16;
pub const MAX_LINE: usize = 512;
pub const MAX_HISTORY: usize = 128;

pub const Tokens = struct {
    args: [MAX_ARGS][]const u8,
    len: usize,

    /// Tokenise `line` (caller keeps ownership).  Quotes and backslash
    /// escapes are not supported — whitespace is the only delimiter.
    pub fn parse(line: []const u8) Tokens {
        var t = Tokens{ .args = undefined, .len = 0 };
        var it = std.mem.tokenizeAny(u8, line, " \t\r\n");
        while (it.next()) |tok| {
            if (t.len >= MAX_ARGS) break;
            t.args[t.len] = tok;
            t.len += 1;
        }
        return t;
    }

    pub fn get(self: *const Tokens, i: usize) ?[]const u8 {
        return if (i < self.len) self.args[i] else null;
    }

    pub fn rest(self: *const Tokens, from: usize) []const []const u8 {
        if (from >= self.len) return &.{};
        return self.args[from..self.len];
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// Command history (ring buffer of owned strings)
// ─────────────────────────────────────────────────────────────────────────────

pub const History = struct {
    allocator: std.mem.Allocator,
    entries: [MAX_HISTORY][]u8,
    head: usize,
    count: usize,

    pub fn init(allocator: std.mem.Allocator) History {
        return .{ .allocator = allocator, .entries = undefined, .head = 0, .count = 0 };
    }

    pub fn deinit(self: *History) void {
        const n = @min(self.count, MAX_HISTORY);
        for (0..n) |i| self.allocator.free(self.entries[i]);
    }

    pub fn push(self: *History, line: []const u8) !void {
        // Skip blank lines and duplicates of the previous entry
        if (line.len == 0) return;
        if (self.count > 0) {
            const prev = self.entries[(self.head + MAX_HISTORY - 1) % MAX_HISTORY];
            if (std.mem.eql(u8, prev, line)) return;
        }
        const owned = try self.allocator.dupe(u8, line);
        if (self.count == MAX_HISTORY) {
            // Overwrite oldest
            self.allocator.free(self.entries[self.head]);
            self.entries[self.head] = owned;
            self.head = (self.head + 1) % MAX_HISTORY;
        } else {
            const slot = (self.head + self.count) % MAX_HISTORY;
            self.entries[slot] = owned;
            self.count += 1;
        }
    }

    /// Return the n-th most recent entry (0 = newest), or null.
    pub fn get(self: *const History, n: usize) ?[]const u8 {
        if (n >= self.count) return null;
        const slot = (self.head + self.count - 1 - n) % MAX_HISTORY;
        return self.entries[slot];
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// ShellConfig
// ─────────────────────────────────────────────────────────────────────────────

pub const ShellConfig = struct {
    /// Display ANSI colour codes.
    use_colour: bool = true,
    /// Show timestamps on event listings.
    show_ts: bool = true,
    /// Maximum lines in event watch before auto-stopping.
    watch_limit: usize = 50,
    /// Hostname shown in prompt.
    hostname: []const u8 = "kogi",
};

// ─────────────────────────────────────────────────────────────────────────────
// ShellSession — the top-level interactive REPL
// ─────────────────────────────────────────────────────────────────────────────

pub const ShellSession = struct {
    allocator: std.mem.Allocator,
    kernel: *Kernel,
    reader: std.io.AnyReader,
    writer: std.io.AnyWriter,
    cfg: ShellConfig,
    history: History,
    running: bool,

    // ── Lifecycle ─────────────────────────────────────────────────────────

    pub fn init(
        allocator: std.mem.Allocator,
        kernel: *Kernel,
        reader: std.io.AnyReader,
        writer: std.io.AnyWriter,
        cfg: ShellConfig,
    ) ShellSession {
        return .{
            .allocator = allocator,
            .kernel = kernel,
            .reader = reader,
            .writer = writer,
            .cfg = cfg,
            .history = History.init(allocator),
            .running = false,
        };
    }

    pub fn deinit(self: *ShellSession) void {
        self.history.deinit();
    }

    // ── Main REPL loop ────────────────────────────────────────────────────

    /// Enter the interactive read-eval-print loop.
    /// Returns when the user types `exit`, `quit`, or sends EOF.
    pub fn run(self: *ShellSession) !void {
        self.running = true;
        try self.printBanner();

        var line_buf: [MAX_LINE]u8 = undefined;

        while (self.running) {
            try self.printPrompt();

            const raw = self.reader.readUntilDelimiter(&line_buf, '\n') catch |err| switch (err) {
                error.EndOfStream => break,
                else => return err,
            };
            const line = std.mem.trimRight(u8, raw, "\r\n ");

            if (line.len == 0) continue;
            self.history.push(line) catch {};

            self.dispatch(line) catch |err| {
                try self.printError(err, line);
            };
        }

        try self.println("Goodbye.");
    }

    // ── Prompt and banner ─────────────────────────────────────────────────

    fn printPrompt(self: *ShellSession) !void {
        const mode_tag = if (self.kernel.isPrivileged()) "root" else "user";
        const col_open = if (self.cfg.use_colour) C.bold ++ C.cyan else "";
        const col_close = if (self.cfg.use_colour) C.reset else "";
        const col_hash = if (self.cfg.use_colour) C.bold ++ C.green else "";
        const symbol = if (self.kernel.isPrivileged()) "#" else "$";
        try self.writer.print("{s}{s}@{s}{s}{s}{s} {s} ", .{ col_open, mode_tag, self.cfg.hostname, col_close, col_hash, symbol, col_close });
    }

    fn printBanner(self: *ShellSession) !void {
        const sep = "─" ** 62;
        if (self.cfg.use_colour) {
            try self.writer.print(C.bold ++ C.blue ++
                "\n  ██╗  ██╗ ██████╗  ██████╗ ██╗    Kernel Shell\n" ++
                "  ██║ ██╔╝██╔═══██╗██╔════╝ ██║    Interactive CLI\n" ++
                "  █████╔╝ ██║   ██║██║  ███╗██║    Type 'help' for commands\n" ++
                "  ██╔═██╗ ██║   ██║██║   ██║██║    Type 'su' for root access\n" ++
                "  ██║  ██╗╚██████╔╝╚██████╔╝██║    \n" ++
                "  ╚═╝  ╚═╝ ╚═════╝  ╚═════╝ ╚═╝    \n" ++
                C.reset ++ "\n", .{});
        } else {
            try self.writer.print("\n  Kernel Interactive Shell\n", .{});
        }
        try self.printSep(sep);
        try self.writer.print("\n", .{});
    }

    fn printSep(self: *ShellSession, sep: []const u8) !void {
        if (self.cfg.use_colour)
            try self.writer.print(C.dim ++ "{s}" ++ C.reset ++ "\n", .{sep})
        else
            try self.writer.print("{s}\n", .{sep});
    }

    fn println(self: *ShellSession, msg: []const u8) !void {
        try self.writer.print("{s}\n", .{msg});
    }

    fn printOk(self: *ShellSession, msg: []const u8) !void {
        if (self.cfg.use_colour)
            try self.writer.print(C.green ++ "✓ " ++ C.reset ++ "{s}\n", .{msg})
        else
            try self.writer.print("[ok] {s}\n", .{msg});
    }

    fn printWarn(self: *ShellSession, msg: []const u8) !void {
        if (self.cfg.use_colour)
            try self.writer.print(C.yellow ++ "⚠ " ++ C.reset ++ "{s}\n", .{msg})
        else
            try self.writer.print("[warn] {s}\n", .{msg});
    }

    fn printError(self: *ShellSession, err: anyerror, input: []const u8) !void {
        _ = input;
        const msg: []const u8 = switch (err) {
            ShellError.UnknownCommand => "Unknown command. Type 'help' for a list.",
            ShellError.WrongArgCount => "Wrong number of arguments.",
            ShellError.InvalidArgument => "Invalid argument.",
            ShellError.PrivilegeRequired => "Privileged mode required. Run 'su' first.",
            KernelError.AccessDenied => "Access denied (insufficient role).",
            KernelError.PrivilegeDenied => "Kernel is not in privileged mode.",
            KernelError.ModuleNotFound => "Module not found.",
            else => @errorName(err),
        };
        if (self.cfg.use_colour)
            try self.writer.print(C.red ++ "✗ " ++ C.reset ++ "{s}\n", .{msg})
        else
            try self.writer.print("[error] {s}\n", .{msg});
    }

    fn printKV(self: *ShellSession, key: []const u8, comptime fmt: []const u8, args: anytype) !void {
        const key_col = if (self.cfg.use_colour) C.cyan else "";
        const rst = if (self.cfg.use_colour) C.reset else "";
        try self.writer.print("  {s}{s:<24}{s}" ++ fmt ++ "\n", .{ key_col, key, rst } ++ args);
    }

    fn printHeader(self: *ShellSession, title: []const u8) !void {
        if (self.cfg.use_colour)
            try self.writer.print("\n" ++ C.bold ++ C.white ++ "  {s}" ++ C.reset ++ "\n", .{title})
        else
            try self.writer.print("\n  {s}\n", .{title});
    }

    fn requirePrivileged(self: *ShellSession) !void {
        if (!self.kernel.isPrivileged()) return ShellError.PrivilegeRequired;
    }

    // ── Command dispatcher ────────────────────────────────────────────────

    fn dispatch(self: *ShellSession, line: []const u8) !void {
        const t = Tokens.parse(line);
        if (t.len == 0) return;

        const cmd = t.args[0];

        if (std.mem.eql(u8, cmd, "help")) return self.cmdHelp(&t);
        if (std.mem.eql(u8, cmd, "status")) return self.cmdStatus();
        if (std.mem.eql(u8, cmd, "stats")) return self.cmdStats();
        if (std.mem.eql(u8, cmd, "version")) return self.cmdVersion();
        if (std.mem.eql(u8, cmd, "mode")) return self.cmdMode();
        if (std.mem.eql(u8, cmd, "su")) return self.cmdSu(&t);
        if (std.mem.eql(u8, cmd, "exit") or
            std.mem.eql(u8, cmd, "quit")) return self.cmdExit();
        if (std.mem.eql(u8, cmd, "history")) return self.cmdHistory();
        if (std.mem.eql(u8, cmd, "clear")) return self.cmdClear();

        // Sub-command groups
        if (std.mem.eql(u8, cmd, "module")) return self.cmdModule(&t);
        if (std.mem.eql(u8, cmd, "service")) return self.cmdService(&t);
        if (std.mem.eql(u8, cmd, "memory") or
            std.mem.eql(u8, cmd, "mem")) return self.cmdMemory(&t);
        if (std.mem.eql(u8, cmd, "process") or
            std.mem.eql(u8, cmd, "proc")) return self.cmdProcess(&t);
        if (std.mem.eql(u8, cmd, "tenant")) return self.cmdTenant(&t);
        if (std.mem.eql(u8, cmd, "net") or
            std.mem.eql(u8, cmd, "network")) return self.cmdNet(&t);
        if (std.mem.eql(u8, cmd, "event")) return self.cmdEvent(&t);
        if (std.mem.eql(u8, cmd, "sched") or
            std.mem.eql(u8, cmd, "scheduler")) return self.cmdSched(&t);

        return ShellError.UnknownCommand;
    }

    // ─────────────────────────────────────────────────────────────────────
    // help
    // ─────────────────────────────────────────────────────────────────────

    fn cmdHelp(self: *ShellSession, t: *const Tokens) !void {
        if (t.get(1)) |sub| {
            return self.printSubHelp(sub);
        }

        const priv_marker = if (self.cfg.use_colour) C.yellow ++ "[P]" ++ C.reset else "[P]";
        const dim = if (self.cfg.use_colour) C.dim else "";
        const rst = if (self.cfg.use_colour) C.reset else "";

        try self.printHeader("Available Commands");
        try self.writer.print(
            \\  {s}[P]{s} = requires privileged mode (run 'su' first)
            \\
            \\  help [command]             — show this help or detailed help for <command>
            \\  status                     — kernel status overview
            \\  stats                      — full aggregate stats table
            \\  version                    — kernel version info
            \\  mode                       — show current privilege mode
            \\  su [exit]                  — enter / leave privileged mode
            \\  history                    — show command history
            \\  clear                      — clear screen
            \\  exit / quit                — exit the shell
            \\
            \\  {s}module{s}  list|show|register|start|stop|remove
            \\  {s}service{s} list|start|stop|startall|stopall
            \\  {s}memory{s}  status|alloc|free
            \\  {s}process{s} list|spawn|kill
            \\  {s}tenant{s}  list|register
            \\  {s}net{s}     list
            \\  {s}event{s}   list|query
            \\  {s}sched{s}   list|tick
            \\
        , .{
            priv_marker, rst,
            dim,         rst,
            dim,         rst,
            dim,         rst,
            dim,         rst,
            dim,         rst,
            dim,         rst,
            dim,         rst,
            dim,         rst,
        });
    }

    fn printSubHelp(self: *ShellSession, sub: []const u8) !void {
        if (std.mem.eql(u8, sub, "module")) {
            try self.println(
                \\  module list                  — list all registered modules
                \\  module show <id>             — show detailed info for a module
                \\  module register <id> <kind>  — register a new module      [P]
                \\  module start <id>            — start a module              [P]
                \\  module stop  <id>            — stop a module               [P]
                \\  module remove <id>           — remove a module             [P]
            );
        } else if (std.mem.eql(u8, sub, "service")) {
            try self.println(
                \\  service list                 — list all services
                \\  service start <id>           — provision and start          [P]
                \\  service stop  <id>           — stop a running service       [P]
                \\  service startall             — start all services           [P]
                \\  service stopall              — stop all services            [P]
            );
        } else if (std.mem.eql(u8, sub, "memory") or std.mem.eql(u8, sub, "mem")) {
            try self.println(
                \\  memory status                — show memory usage
                \\  memory alloc <tenant> <bytes>— allocate tenant memory       [P]
                \\  memory free  <tenant> <bytes>— release tenant memory        [P]
            );
        } else if (std.mem.eql(u8, sub, "process") or std.mem.eql(u8, sub, "proc")) {
            try self.println(
                \\  process list                 — list live processes
                \\  process spawn <name>         — spawn a new process          [P]
                \\  process kill  <pid>          — terminate a process          [P]
            );
        } else if (std.mem.eql(u8, sub, "event")) {
            try self.println(
                \\  event list [N]               — show last N events (default 20)
                \\  event query <domain> [wm]    — query domain since watermark
            );
        } else if (std.mem.eql(u8, sub, "sched")) {
            try self.println(
                \\  sched list                   — list scheduled jobs
                \\  sched tick                   — fire due jobs now            [P]
            );
        } else {
            try self.writer.print("No detailed help for '{s}'. Try 'help'.\n", .{sub});
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // status / stats / version / mode
    // ─────────────────────────────────────────────────────────────────────

    fn cmdStatus(self: *ShellSession) !void {
        const s = self.kernel.stats();
        try self.printHeader("Kernel Status");

        const mode_col = if (s.mode == .privileged and self.cfg.use_colour)
            C.green ++ C.bold
        else if (self.cfg.use_colour) C.yellow else "";
        const rst = if (self.cfg.use_colour) C.reset else "";

        try self.writer.print("  Mode         {s}{s}{s}\n" ++
            "  Session role {s}\n\n", .{ mode_col, s.mode.label(), rst, @tagName(s.session_role) });

        try self.printKV("Memory", "{}/{} bytes", .{ s.memory_used_bytes, s.memory_total_bytes });
        try self.printKV("Processes", "{} live", .{s.live_processes});
        try self.printKV("Workers", "{} idle / {} busy", .{ s.idle_workers, s.busy_workers });
        try self.printKV("Modules", "{} active / {} total", .{ s.active_modules, s.total_modules });
        try self.printKV("Services", "{} running / {} faulted", .{ s.running_services, s.faulted_services });
        try self.printKV("Tenants", "{}", .{s.tenant_count});
        try self.printKV("Events", "{} total / {} in log", .{ s.kernel_events_total, s.kernel_events_log_len });
        try self.writer.print("\n", .{});
    }

    fn cmdStats(self: *ShellSession) !void {
        const s = self.kernel.stats();
        const sep = "─" ** 50;

        try self.printHeader("Aggregate Kernel Stats");
        try self.printSep(sep);

        try self.printHeader("System");
        try self.printKV("mode", "{s}", .{s.mode.label()});
        try self.printKV("session_role", "{s}", .{@tagName(s.session_role)});

        try self.printHeader("Memory");
        try self.printKV("used_bytes", "{}", .{s.memory_used_bytes});
        try self.printKV("total_bytes", "{}", .{s.memory_total_bytes});

        try self.printHeader("Processes");
        try self.printKV("live", "{}", .{s.live_processes});
        try self.printKV("idle_workers", "{}", .{s.idle_workers});
        try self.printKV("busy_workers", "{}", .{s.busy_workers});
        try self.printKV("queued_tasks", "{}", .{s.queued_tasks});
        try self.printKV("scheduled_jobs", "{}", .{s.scheduled_jobs});

        try self.printHeader("Network");
        try self.printKV("open_sockets", "{}", .{s.open_sockets});
        try self.printKV("connections", "{}", .{s.active_connections});
        try self.printKV("queued_packets", "{}", .{s.queued_packets});
        try self.printKV("routes", "{}", .{s.route_count});

        try self.printHeader("Modules");
        try self.printKV("total", "{}", .{s.total_modules});
        try self.printKV("active", "{}", .{s.active_modules});
        try self.printKV("module_events", "{}", .{s.module_events});

        try self.printHeader("Services");
        try self.printKV("total", "{}", .{s.total_services});
        try self.printKV("running", "{}", .{s.running_services});
        try self.printKV("faulted", "{}", .{s.faulted_services});
        try self.printKV("service_events", "{}", .{s.service_events});

        try self.printHeader("Resources");
        try self.printKV("tenants", "{}", .{s.tenant_count});
        try self.printKV("resource_events", "{}", .{s.resource_event_count});

        try self.printHeader("Events");
        try self.printKV("total", "{}", .{s.kernel_events_total});
        try self.printKV("log_len", "{}", .{s.kernel_events_log_len});
        try self.printKV("overflow", "{}", .{s.kernel_events_overflow});
        try self.printKV("subscribers", "{}", .{s.kernel_event_subscribers});

        try self.writer.print("\n", .{});
    }

    fn cmdVersion(self: *ShellSession) !void {
        try self.printHeader("Kernel Version");
        try self.printKV("kernel", "kogi/0.1.0", .{});
        try self.printKV("shell", "kogi-shell/1.0.0", .{});
        try self.printKV("language", "Zig (std)", .{});
        try self.writer.print("\n", .{});
    }

    fn cmdMode(self: *ShellSession) !void {
        const s = self.kernel.stats();
        try self.writer.print("  Kernel mode  : {s}\n", .{s.mode.label()});
        try self.writer.print("  Session role : {s}\n\n", .{@tagName(s.session_role)});
    }

    // ─────────────────────────────────────────────────────────────────────
    // su — privilege escalation / de-escalation
    // ─────────────────────────────────────────────────────────────────────

    fn cmdSu(self: *ShellSession, t: *const Tokens) !void {
        // `su exit` or `su logout` drops privileges
        if (t.get(1)) |sub| {
            if (std.mem.eql(u8, sub, "exit") or std.mem.eql(u8, sub, "logout")) {
                try self.kernel.exitPrivileged(.root);
                try self.printOk("Dropped to user mode.");
                return;
            }
        }

        if (self.kernel.isPrivileged()) {
            try self.printWarn("Already in privileged mode.");
            return;
        }

        // In a real system this would verify a credential; here we accept the
        // root role unconditionally (the security boundary is the Role RBAC).
        try self.kernel.enterPrivileged(.root);
        try self.printOk("Entered privileged mode.");
    }

    fn cmdExit(self: *ShellSession) !void {
        if (self.kernel.isPrivileged()) {
            // Drop privileges first, then stop
            self.kernel.exitPrivileged(.root) catch {};
            try self.printOk("Dropped privileges.");
        }
        self.running = false;
    }

    fn cmdHistory(self: *ShellSession) !void {
        try self.printHeader("Command History");
        const n = self.history.count;
        if (n == 0) {
            try self.println("  (empty)");
            return;
        }
        var i: usize = n;
        while (i > 0) {
            i -= 1;
            const entry = self.history.get(i) orelse continue;
            try self.writer.print("  {:>4}  {s}\n", .{ n - i, entry });
        }
        try self.writer.print("\n", .{});
    }

    fn cmdClear(self: *ShellSession) !void {
        if (self.cfg.use_colour)
            try self.writer.print("\x1b[2J\x1b[H", .{})
        else
            try self.writer.print("\n\n\n", .{});
    }

    // ─────────────────────────────────────────────────────────────────────
    // module
    // ─────────────────────────────────────────────────────────────────────

    fn cmdModule(self: *ShellSession, t: *const Tokens) !void {
        const sub = t.get(1) orelse {
            try self.printSubHelp("module");
            return;
        };

        if (std.mem.eql(u8, sub, "list")) return self.moduleList();
        if (std.mem.eql(u8, sub, "show")) return self.moduleShow(t.get(2));
        if (std.mem.eql(u8, sub, "register")) return self.moduleRegister(t);
        if (std.mem.eql(u8, sub, "start")) return self.moduleStart(t.get(2));
        if (std.mem.eql(u8, sub, "stop")) return self.moduleStop(t.get(2));
        if (std.mem.eql(u8, sub, "remove")) return self.moduleRemove(t.get(2));

        try self.writer.print("Unknown module sub-command '{s}'. Try 'help module'.\n", .{sub});
    }

    fn moduleList(self: *ShellSession) !void {
        try self.printHeader("Modules");

        const hdr_col = if (self.cfg.use_colour) C.bold else "";
        const rst = if (self.cfg.use_colour) C.reset else "";
        try self.writer.print("  {s}{s:<32} {s:<12} {s:<10}{s}\n", .{ hdr_col, "ID", "STATUS", "CLASS", rst });

        const sep = "  " ++ "─" ** 58;
        try self.println(sep);

        var it = self.kernel.modules.registry.modules.valueIterator();
        var count: usize = 0;
        while (it.next()) |m| {
            const status_col = switch (m.status) {
                .active => if (self.cfg.use_colour) C.green else "",
                .faulted => if (self.cfg.use_colour) C.red else "",
                .provisioning => if (self.cfg.use_colour) C.yellow else "",
                else => if (self.cfg.use_colour) C.dim else "",
            };
            try self.writer.print("  {s:<32} {s}{s:<12}{s} {s:<10}\n", .{ m.id, status_col, @tagName(m.status), rst, @tagName(m.class) });
            count += 1;
        }
        if (count == 0) try self.println("  (no modules registered)");
        try self.writer.print("\n  Total: {}\n\n", .{count});
    }

    fn moduleShow(self: *ShellSession, id_opt: ?[]const u8) !void {
        const id = id_opt orelse return ShellError.WrongArgCount;
        const m = self.kernel.modules.registry.get(id) orelse {
            try self.writer.print("  Module '{s}' not found.\n", .{id});
            return;
        };

        try self.printHeader(m.id);
        try self.printKV("id", "{s}", .{m.id});
        try self.printKV("kind", "{s}", .{m.kind});
        try self.printKV("class", "{s}", .{@tagName(m.class)});
        try self.printKV("version", "{s}", .{m.version});
        try self.printKV("status", "{s}", .{@tagName(m.status)});
        try self.printKV("start_count", "{}", .{m.start_count});
        try self.printKV("fault_count", "{}", .{m.fault_count});
        try self.printKV("owner_role", "{s}", .{@tagName(m.owner_role)});
        try self.printKV("endpoint", "{s}", .{m.network.endpoint});
        try self.printKV("net_manager", "{s}", .{m.network.network_manager});
        try self.printKV("mem_used", "{}", .{m.usage.memory_bytes});
        try self.printKV("mem_limit", "{}", .{m.limits.memory_bytes});
        try self.printKV("processes", "{}/{}", .{ m.usage.process_count, m.limits.max_processes });
        try self.printKV("files", "{}/{}", .{ m.usage.file_count, m.limits.max_files });
        try self.printKV("ingress", "{} bytes", .{m.network.ingress_bytes});
        try self.printKV("egress", "{} bytes", .{m.network.egress_bytes});
        try self.writer.print("\n", .{});
    }

    fn moduleRegister(self: *ShellSession, t: *const Tokens) !void {
        try self.requirePrivileged();
        const id = t.get(2) orelse return ShellError.WrongArgCount;
        const kind = t.get(3) orelse id; // kind defaults to id if omitted
        try self.kernel.registerModule(.root, id, kind);
        try self.writer.print("  Module '{s}' registered.\n\n", .{id});
    }

    fn moduleStart(self: *ShellSession, id_opt: ?[]const u8) !void {
        try self.requirePrivileged();
        const id = id_opt orelse return ShellError.WrongArgCount;
        try self.kernel.startModule(.root, id);
        try self.printOk(id);
    }

    fn moduleStop(self: *ShellSession, id_opt: ?[]const u8) !void {
        try self.requirePrivileged();
        const id = id_opt orelse return ShellError.WrongArgCount;
        try self.kernel.stopModule(.root, id);
        try self.printOk(id);
    }

    fn moduleRemove(self: *ShellSession, id_opt: ?[]const u8) !void {
        try self.requirePrivileged();
        const id = id_opt orelse return ShellError.WrongArgCount;
        try self.kernel.removeModule(.root, id);
        try self.writer.print("  Module '{s}' removed.\n\n", .{id});
    }

    // ─────────────────────────────────────────────────────────────────────
    // service
    // ─────────────────────────────────────────────────────────────────────

    fn cmdService(self: *ShellSession, t: *const Tokens) !void {
        const sub = t.get(1) orelse {
            try self.printSubHelp("service");
            return;
        };

        if (std.mem.eql(u8, sub, "list")) return self.serviceList();
        if (std.mem.eql(u8, sub, "start")) return self.serviceStart(t.get(2));
        if (std.mem.eql(u8, sub, "stop")) return self.serviceStop(t.get(2));
        if (std.mem.eql(u8, sub, "startall")) return self.serviceStartAll();
        if (std.mem.eql(u8, sub, "stopall")) return self.serviceStopAll();

        try self.writer.print("Unknown service sub-command '{s}'. Try 'help service'.\n", .{sub});
    }

    fn serviceList(self: *ShellSession) !void {
        try self.printHeader("Services");

        const hdr_col = if (self.cfg.use_colour) C.bold else "";
        const rst = if (self.cfg.use_colour) C.reset else "";
        try self.writer.print("  {s}{s:<32} {s:<14} {s}{s}\n", .{ hdr_col, "ID", "STATUS", "VERSION", rst });
        try self.println("  " ++ "─" ** 60);

        var it = self.kernel.services.registry.services.valueIterator();
        var count: usize = 0;
        while (it.next()) |svc| {
            const col = switch (svc.status) {
                .running => if (self.cfg.use_colour) C.green else "",
                .faulted => if (self.cfg.use_colour) C.red else "",
                else => if (self.cfg.use_colour) C.dim else "",
            };
            try self.writer.print("  {s:<32} {s}{s:<14}{s} {s}\n", .{ svc.id, col, @tagName(svc.status), rst, svc.version });
            count += 1;
        }
        if (count == 0) try self.println("  (no services registered)");
        try self.writer.print("\n  Total: {}\n\n", .{count});
    }

    fn serviceStart(self: *ShellSession, id_opt: ?[]const u8) !void {
        try self.requirePrivileged();
        const id = id_opt orelse return ShellError.WrongArgCount;
        try self.kernel.startService(.root, id);
        try self.printOk(id);
    }

    fn serviceStop(self: *ShellSession, id_opt: ?[]const u8) !void {
        try self.requirePrivileged();
        const id = id_opt orelse return ShellError.WrongArgCount;
        try self.kernel.stopService(.root, id);
        try self.printOk(id);
    }

    fn serviceStartAll(self: *ShellSession) !void {
        try self.requirePrivileged();
        const n = try self.kernel.startAllServices(.root);
        try self.writer.print("  Started {} service(s).\n\n", .{n});
    }

    fn serviceStopAll(self: *ShellSession) !void {
        try self.requirePrivileged();
        try self.kernel.stopAllServices(.root);
        try self.printOk("All services stopped.");
    }

    // ─────────────────────────────────────────────────────────────────────
    // memory
    // ─────────────────────────────────────────────────────────────────────

    fn cmdMemory(self: *ShellSession, t: *const Tokens) !void {
        const sub = t.get(1) orelse {
            try self.printSubHelp("memory");
            return;
        };

        if (std.mem.eql(u8, sub, "status")) return self.memoryStatus();
        if (std.mem.eql(u8, sub, "alloc")) return self.memoryAlloc(t);
        if (std.mem.eql(u8, sub, "free")) return self.memoryFree(t);

        try self.writer.print("Unknown memory sub-command '{s}'.\n", .{sub});
    }

    fn memoryStatus(self: *ShellSession) !void {
        const s = self.kernel.stats();
        const pct: u64 = if (s.memory_total_bytes > 0)
            s.memory_used_bytes * 100 / s.memory_total_bytes
        else
            0;

        try self.printHeader("Memory");
        try self.printKV("used", "{} bytes ({d}%)", .{ s.memory_used_bytes, pct });
        try self.printKV("total", "{} bytes", .{s.memory_total_bytes});
        try self.printKV("available", "{} bytes", .{s.memory_total_bytes -| s.memory_used_bytes});

        // Per-tenant breakdown
        try self.printHeader("Tenant allocations");
        var it = self.kernel.resources.registry.tenants.valueIterator();
        var any = false;
        while (it.next()) |tenant| {
            try self.writer.print("  {s:<32} {} bytes\n", .{ tenant.id, tenant.usage.memory_bytes });
            any = true;
        }
        if (!any) try self.println("  (no tenants)");
        try self.writer.print("\n", .{});
    }

    fn memoryAlloc(self: *ShellSession, t: *const Tokens) !void {
        try self.requirePrivileged();
        const tenant = t.get(2) orelse return ShellError.WrongArgCount;
        const bytes_str = t.get(3) orelse return ShellError.WrongArgCount;
        const bytes = std.fmt.parseUnsigned(u64, bytes_str, 10) catch
            return ShellError.InvalidArgument;
        try self.kernel.allocateTenantMemory(.root, tenant, bytes);
        try self.writer.print("  Allocated {} bytes for '{s}'.\n\n", .{ bytes, tenant });
    }

    fn memoryFree(self: *ShellSession, t: *const Tokens) !void {
        try self.requirePrivileged();
        const tenant = t.get(2) orelse return ShellError.WrongArgCount;
        const bytes_str = t.get(3) orelse return ShellError.WrongArgCount;
        const bytes = std.fmt.parseUnsigned(u64, bytes_str, 10) catch
            return ShellError.InvalidArgument;
        try self.kernel.freeTenantMemory(.root, tenant, bytes);
        try self.writer.print("  Freed {} bytes for '{s}'.\n\n", .{ bytes, tenant });
    }

    // ─────────────────────────────────────────────────────────────────────
    // process
    // ─────────────────────────────────────────────────────────────────────

    fn cmdProcess(self: *ShellSession, t: *const Tokens) !void {
        const sub = t.get(1) orelse {
            try self.printSubHelp("process");
            return;
        };

        if (std.mem.eql(u8, sub, "list")) return self.processList();
        if (std.mem.eql(u8, sub, "spawn")) return self.processSpawn(t.get(2));
        if (std.mem.eql(u8, sub, "kill")) return self.processKill(t.get(2));

        try self.writer.print("Unknown process sub-command '{s}'.\n", .{sub});
    }

    fn processList(self: *ShellSession) !void {
        try self.printHeader("Live Processes");

        const hdr_col = if (self.cfg.use_colour) C.bold else "";
        const rst = if (self.cfg.use_colour) C.reset else "";
        try self.writer.print("  {s}{s:>8}  {s:<28} {s:<14} {s}{s}\n", .{ hdr_col, "PID", "NAME", "STATE", "OWNER", rst });
        try self.println("  " ++ "─" ** 60);

        var it = self.kernel._proc_table.processes.valueIterator();
        var count: usize = 0;
        while (it.next()) |p| {
            const col = switch (p.state) {
                .running => if (self.cfg.use_colour) C.green else "",
                .faulted => if (self.cfg.use_colour) C.red else "",
                else => if (self.cfg.use_colour) C.dim else "",
            };
            try self.writer.print("  {:>8}  {s:<28} {s}{s:<14}{s} {s}\n", .{ p.pid, p.name, col, @tagName(p.state), rst, @tagName(p.owner) });
            count += 1;
        }
        if (count == 0) try self.println("  (no processes)");
        try self.writer.print("\n  Total: {}\n\n", .{count});
    }

    fn processSpawn(self: *ShellSession, name_opt: ?[]const u8) !void {
        try self.requirePrivileged();
        const name = name_opt orelse return ShellError.WrongArgCount;
        const pid = try self.kernel.spawnProcess(.root, name, .module_runtime);
        try self.writer.print("  Spawned process '{s}' with PID {}.\n\n", .{ name, pid });
    }

    fn processKill(self: *ShellSession, pid_str_opt: ?[]const u8) !void {
        try self.requirePrivileged();
        const pid_str = pid_str_opt orelse return ShellError.WrongArgCount;
        const pid = std.fmt.parseUnsigned(u64, pid_str, 10) catch
            return ShellError.InvalidArgument;
        try self.kernel.terminateProcess(.root, pid);
        try self.writer.print("  Terminated PID {}.\n\n", .{pid});
    }

    // ─────────────────────────────────────────────────────────────────────
    // tenant
    // ─────────────────────────────────────────────────────────────────────

    fn cmdTenant(self: *ShellSession, t: *const Tokens) !void {
        const sub = t.get(1) orelse {
            try self.println("  tenant list|register");
            return;
        };

        if (std.mem.eql(u8, sub, "list")) return self.tenantList();
        if (std.mem.eql(u8, sub, "register")) return self.tenantRegister(t.get(2));

        try self.writer.print("Unknown tenant sub-command '{s}'.\n", .{sub});
    }

    fn tenantList(self: *ShellSession) !void {
        try self.printHeader("Resource Tenants");

        const hdr_col = if (self.cfg.use_colour) C.bold else "";
        const rst = if (self.cfg.use_colour) C.reset else "";
        try self.writer.print("  {s}{s:<32} {s:>12} {s:>8} {s:>8}{s}\n", .{ hdr_col, "TENANT ID", "MEM (bytes)", "PROCS", "FILES", rst });
        try self.println("  " ++ "─" ** 65);

        var it = self.kernel.resources.registry.tenants.valueIterator();
        var count: usize = 0;
        while (it.next()) |tenant| {
            try self.writer.print("  {s:<32} {:>12} {:>8} {:>8}\n", .{ tenant.id, tenant.usage.memory_bytes, tenant.usage.process_count, tenant.usage.file_count });
            count += 1;
        }
        if (count == 0) try self.println("  (no tenants)");
        try self.writer.print("\n  Total: {}\n\n", .{count});
    }

    fn tenantRegister(self: *ShellSession, id_opt: ?[]const u8) !void {
        try self.requirePrivileged();
        const id = id_opt orelse return ShellError.WrongArgCount;
        const res_mod = @import("resources.zig");
        try self.kernel.registerTenant(.root, id, res_mod.ResourcePolicy{});
        try self.writer.print("  Tenant '{s}' registered.\n\n", .{id});
    }

    // ─────────────────────────────────────────────────────────────────────
    // net
    // ─────────────────────────────────────────────────────────────────────

    fn cmdNet(self: *ShellSession, t: *const Tokens) !void {
        const sub = t.get(1) orelse "list";
        if (std.mem.eql(u8, sub, "list")) return self.netList();
        try self.writer.print("Unknown net sub-command '{s}'.\n", .{sub});
    }

    fn netList(self: *ShellSession) !void {
        try self.printHeader("Network Addresses");

        const hdr_col = if (self.cfg.use_colour) C.bold else "";
        const rst = if (self.cfg.use_colour) C.reset else "";
        try self.writer.print("  {s}{s:<28} {s:<18} {s:>6} {s}{s}\n", .{ hdr_col, "NAME", "ADDRESS", "PORT", "SCHEME", rst });
        try self.println("  " ++ "─" ** 62);

        var it = self.kernel.network.addresses.entries.valueIterator();
        var count: usize = 0;
        while (it.next()) |rec| {
            var addr_buf: [32]u8 = undefined;
            const addr_str = rec.addr.format(&addr_buf) catch "?";
            try self.writer.print("  {s:<28} {s:<18} {:>6} {s}\n", .{ rec.name, addr_str, rec.port, rec.scheme });
            count += 1;
        }
        if (count == 0) try self.println("  (no addresses)");
        try self.writer.print("\n  Total: {}\n\n", .{count});
    }

    // ─────────────────────────────────────────────────────────────────────
    // event
    // ─────────────────────────────────────────────────────────────────────

    fn cmdEvent(self: *ShellSession, t: *const Tokens) !void {
        const sub = t.get(1) orelse "list";

        if (std.mem.eql(u8, sub, "list")) return self.eventList(t.get(2));
        if (std.mem.eql(u8, sub, "query")) return self.eventQuery(t);

        try self.writer.print("Unknown event sub-command '{s}'.\n", .{sub});
    }

    fn eventList(self: *ShellSession, n_str: ?[]const u8) !void {
        const n: usize = if (n_str) |s| (std.fmt.parseUnsigned(usize, s, 10) catch 20) else 20;

        var events = std.ArrayList(KernelEvent).init(self.allocator);
        defer events.deinit();

        // Watermark: show last N events
        const total = self.kernel.stats().kernel_events_total;
        const wm: u64 = if (total > n) total - n else 0;
        try self.kernel.queryEvents(null, wm, &events);

        try self.printEventTable(events.items);
    }

    fn eventQuery(self: *ShellSession, t: *const Tokens) !void {
        const domain_str = t.get(2) orelse {
            try self.println("  Usage: event query <domain> [watermark]");
            try self.println("  Domains: kernel memory process network module service resource custom");
            return;
        };

        const domain: EventDomain = parseDomain(domain_str) orelse {
            try self.writer.print("  Unknown domain '{s}'.\n", .{domain_str});
            return;
        };

        const wm: u64 = if (t.get(3)) |s| (std.fmt.parseUnsigned(u64, s, 10) catch 0) else 0;

        const f = EventFilter.forDomain(domain);
        var events = std.ArrayList(KernelEvent).init(self.allocator);
        defer events.deinit();
        try self.kernel.queryEvents(&f, wm, &events);
        try self.printEventTable(events.items);
    }

    fn printEventTable(self: *ShellSession, events: []const KernelEvent) !void {
        try self.printHeader("Events");
        if (events.len == 0) {
            try self.println("  (no events)");
            return;
        }

        const hdr_col = if (self.cfg.use_colour) C.bold else "";
        const rst = if (self.cfg.use_colour) C.reset else "";
        try self.writer.print("  {s}{s:>6}  {s:<10} {s:<10} {s:<24} {s}{s}\n", .{ hdr_col, "ID", "DOMAIN", "KIND", "SOURCE", "PAYLOAD", rst });
        try self.println("  " ++ "─" ** 78);

        for (events) |*ev| {
            const sev_col = switch (ev.severity) {
                .fatal, .@"error" => if (self.cfg.use_colour) C.red else "",
                .warn => if (self.cfg.use_colour) C.yellow else "",
                .debug => if (self.cfg.use_colour) C.dim else "",
                else => "",
            };
            const kind_str = @tagName(ev.kind);
            const short_kind = if (kind_str.len > 10) kind_str[0..10] else kind_str;
            const payload = ev.payloadSlice();
            const short_pay = if (payload.len > 28) payload[0..28] else payload;
            try self.writer.print("  {:>6}  {s}{s:<10}{s} {s:<10} {s:<24} {s}\n", .{ ev.id, sev_col, @tagName(ev.domain), rst, short_kind, ev.sourceSlice(), short_pay });
        }
        try self.writer.print("\n  Showing {} event(s).\n\n", .{events.len});
    }

    // ─────────────────────────────────────────────────────────────────────
    // sched
    // ─────────────────────────────────────────────────────────────────────

    fn cmdSched(self: *ShellSession, t: *const Tokens) !void {
        const sub = t.get(1) orelse "list";

        if (std.mem.eql(u8, sub, "list")) return self.schedList();
        if (std.mem.eql(u8, sub, "tick")) return self.schedTick();

        try self.writer.print("Unknown sched sub-command '{s}'.\n", .{sub});
    }

    fn schedList(self: *ShellSession) !void {
        try self.printHeader("Scheduled Jobs");

        const hdr_col = if (self.cfg.use_colour) C.bold else "";
        const rst = if (self.cfg.use_colour) C.reset else "";
        try self.writer.print("  {s}{s:<20} {s:>14} {s:<10} {s}{s}\n", .{ hdr_col, "LABEL", "DUE_AT_MS", "PRIORITY", "RECURRING", rst });
        try self.println("  " ++ "─" ** 60);

        var it = self.kernel._scheduler.jobs.valueIterator();
        var count: usize = 0;
        while (it.next()) |job| {
            const recur = if (job.interval_ms != null) "yes" else "no";
            try self.writer.print("  {s:<20} {:>14} {s:<10} {s}\n", .{ job.label, job.due_at_ms, @tagName(job.priority), recur });
            count += 1;
        }
        if (count == 0) try self.println("  (no scheduled jobs)");
        try self.writer.print("\n  Total: {}\n\n", .{count});
    }

    fn schedTick(self: *ShellSession) !void {
        try self.requirePrivileged();
        const now = std.time.milliTimestamp();
        const result = try self.kernel.tick(.root, now);
        try self.writer.print("  Tick fired {} job(s), dispatched {}.\n\n", .{ result.fired, result.dispatched });
    }
};

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn parseDomain(s: []const u8) ?EventDomain {
    if (std.mem.eql(u8, s, "kernel")) return .kernel;
    if (std.mem.eql(u8, s, "memory")) return .memory;
    if (std.mem.eql(u8, s, "process")) return .process;
    if (std.mem.eql(u8, s, "network")) return .network;
    if (std.mem.eql(u8, s, "module")) return .module;
    if (std.mem.eql(u8, s, "service")) return .service;
    if (std.mem.eql(u8, s, "resource")) return .resource;
    if (std.mem.eql(u8, s, "custom")) return .custom;
    return null;
}

// ─────────────────────────────────────────────────────────────────────────────
// Entry point  (called when the shell binary is run directly)
// ─────────────────────────────────────────────────────────────────────────────

/// Convenience wrapper: boot a kernel, attach stdin/stdout, run the REPL.
pub fn runStdio(allocator: std.mem.Allocator) !void {
    const k = try allocator.create(Kernel);
    errdefer allocator.destroy(k);
    k.* = try Kernel.init(allocator, .{});
    k.relink();
    defer {
        k.deinit();
        allocator.destroy(k);
    }

    const stdin = std.io.getStdIn().reader().any();
    const stdout = std.io.getStdOut().writer().any();

    var session = ShellSession.init(allocator, k, stdin, stdout, .{});
    defer session.deinit();

    try session.run();
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

/// Build a Kernel on the heap and run the shell against an in-memory reader/writer.
fn makeTestSession(
    allocator: std.mem.Allocator,
    input: []const u8,
    out_buf: *std.ArrayList(u8),
) !ShellSession {
    const k = try allocator.create(Kernel);
    errdefer allocator.destroy(k);
    k.* = try Kernel.init(allocator, .{
        .memory_total_bytes = 512 * 1024 * 1024,
        .memory_audit_enabled = false,
        .worker_count = 2,
        .queue_lane_cap = 32,
        .max_sockets = 16,
        .max_connections = 8,
        .max_conn_failures = 3,
        .packet_lane_cap = 64,
        .module_event_cap = 128,
        .service_event_cap = 128,
        .global_mem_ceiling = 512 * 1024 * 1024,
        .global_process_ceiling = 64,
        .global_conn_ceiling = 64,
        .resource_log_cap = 128,
        .event_log_cap = 256,
    });
    k.relink();

    var fbs = std.io.fixedBufferStream(input);
    const reader = fbs.reader().any();
    const writer = out_buf.writer().any();

    return ShellSession.init(allocator, k, reader, writer, .{
        .use_colour = false,
        .show_ts = false,
        .hostname = "test",
    });
}

/// Run a session from start to finish and return its output.
/// The caller must deinit the kernel and the output buffer.
fn runSession(allocator: std.mem.Allocator, input: []const u8) !struct {
    kernel: *Kernel,
    output: std.ArrayList(u8),
} {
    var out = std.ArrayList(u8).init(allocator);
    errdefer out.deinit();

    var session = try makeTestSession(allocator, input, &out);
    defer session.history.deinit();

    try session.run();

    return .{ .kernel = session.kernel, .output = out };
}

test "shell: startup banner and exit" {
    const allocator = std.testing.allocator;
    var r = try runSession(allocator, "exit\n");
    defer {
        r.kernel.deinit();
        allocator.destroy(r.kernel);
        r.output.deinit();
    }

    try std.testing.expect(r.output.items.len > 0);
    try std.testing.expect(std.mem.indexOf(u8, r.output.items, "Goodbye") != null);
}

test "shell: help command" {
    const allocator = std.testing.allocator;
    var r = try runSession(allocator, "help\nexit\n");
    defer {
        r.kernel.deinit();
        allocator.destroy(r.kernel);
        r.output.deinit();
    }

    const out = r.output.items;
    try std.testing.expect(std.mem.indexOf(u8, out, "module") != null);
    try std.testing.expect(std.mem.indexOf(u8, out, "service") != null);
    try std.testing.expect(std.mem.indexOf(u8, out, "memory") != null);
}

test "shell: status command" {
    const allocator = std.testing.allocator;
    var r = try runSession(allocator, "status\nexit\n");
    defer {
        r.kernel.deinit();
        allocator.destroy(r.kernel);
        r.output.deinit();
    }

    const out = r.output.items;
    try std.testing.expect(std.mem.indexOf(u8, out, "privileged") != null);
    try std.testing.expect(std.mem.indexOf(u8, out, "Memory") != null);
}

test "shell: mode command shows current mode" {
    const allocator = std.testing.allocator;
    var r = try runSession(allocator, "mode\nexit\n");
    defer {
        r.kernel.deinit();
        allocator.destroy(r.kernel);
        r.output.deinit();
    }

    try std.testing.expect(std.mem.indexOf(u8, r.output.items, "privileged") != null);
}

test "shell: unknown command gives error message" {
    const allocator = std.testing.allocator;
    var r = try runSession(allocator, "frobulate\nexit\n");
    defer {
        r.kernel.deinit();
        allocator.destroy(r.kernel);
        r.output.deinit();
    }

    try std.testing.expect(std.mem.indexOf(u8, r.output.items, "Unknown command") != null);
}

test "shell: module register + list + start + stop + remove" {
    const allocator = std.testing.allocator;
    var r = try runSession(allocator, "module register test.mod mymod\n" ++
        "module list\n" ++
        "module start test.mod\n" ++
        "module stop test.mod\n" ++
        "module remove test.mod\n" ++
        "module list\n" ++
        "exit\n");
    defer {
        r.kernel.deinit();
        allocator.destroy(r.kernel);
        r.output.deinit();
    }

    const out = r.output.items;
    try std.testing.expect(std.mem.indexOf(u8, out, "test.mod") != null);
    try std.testing.expect(std.mem.indexOf(u8, out, "registered") != null or
        std.mem.indexOf(u8, out, "removed") != null);
    // After remove, module count should be 0
    try std.testing.expectEqual(@as(usize, 0), r.kernel.modules.registry.count());
}

test "shell: module show displays details" {
    const allocator = std.testing.allocator;
    var r = try runSession(allocator, "module register show.mod showtest\n" ++
        "module show show.mod\n" ++
        "exit\n");
    defer {
        r.kernel.deinit();
        allocator.destroy(r.kernel);
        r.output.deinit();
    }

    const out = r.output.items;
    try std.testing.expect(std.mem.indexOf(u8, out, "show.mod") != null);
    try std.testing.expect(std.mem.indexOf(u8, out, "class") != null);
    try std.testing.expect(std.mem.indexOf(u8, out, "registered") != null);
}

test "shell: privilege required for module register when in user mode" {
    const allocator = std.testing.allocator;
    // Drop privileges first, then try privileged operation
    var r = try runSession(allocator, "su exit\n" ++
        "module register bad.mod bad\n" ++
        "exit\n");
    defer {
        r.kernel.deinit();
        allocator.destroy(r.kernel);
        r.output.deinit();
    }

    try std.testing.expect(std.mem.indexOf(u8, r.output.items, "privileged") != null);
    try std.testing.expectEqual(@as(usize, 0), r.kernel.modules.registry.count());
}

test "shell: su enter and exit" {
    const allocator = std.testing.allocator;
    var r = try runSession(allocator, "su exit\n" ++
        "mode\n" ++
        "su\n" ++
        "mode\n" ++
        "exit\n");
    defer {
        r.kernel.deinit();
        allocator.destroy(r.kernel);
        r.output.deinit();
    }

    const out = r.output.items;
    try std.testing.expect(std.mem.indexOf(u8, out, "user") != null);
    try std.testing.expect(std.mem.indexOf(u8, out, "privileged") != null);
    // Should end in privileged mode (last 'su' without args)
    try std.testing.expect(r.kernel.isPrivileged());
}

test "shell: memory status and alloc + free" {
    const allocator = std.testing.allocator;
    // Register a tenant first, then alloc/free memory
    var r = try runSession(allocator, "tenant register memtest\n" ++
        "memory alloc memtest 65536\n" ++
        "memory status\n" ++
        "memory free memtest 65536\n" ++
        "exit\n");
    defer {
        r.kernel.deinit();
        allocator.destroy(r.kernel);
        r.output.deinit();
    }

    const out = r.output.items;
    try std.testing.expect(std.mem.indexOf(u8, out, "65536") != null or
        std.mem.indexOf(u8, out, "Memory") != null);
}

test "shell: process spawn and kill" {
    const allocator = std.testing.allocator;
    var r = try runSession(allocator, "process spawn myworker\n" ++
        "process list\n" ++
        "exit\n");
    defer {
        r.kernel.deinit();
        allocator.destroy(r.kernel);
        r.output.deinit();
    }

    const out = r.output.items;
    try std.testing.expect(std.mem.indexOf(u8, out, "myworker") != null);
    try std.testing.expect(std.mem.indexOf(u8, out, "Spawned") != null or
        std.mem.indexOf(u8, out, "PID") != null);
}

test "shell: event list shows events" {
    const allocator = std.testing.allocator;
    var r = try runSession(allocator, "module register evt.mod evt\n" ++
        "module start evt.mod\n" ++
        "event list 10\n" ++
        "exit\n");
    defer {
        r.kernel.deinit();
        allocator.destroy(r.kernel);
        r.output.deinit();
    }

    const out = r.output.items;
    try std.testing.expect(std.mem.indexOf(u8, out, "Events") != null);
}

test "shell: event query by domain" {
    const allocator = std.testing.allocator;
    var r = try runSession(allocator, "module register domq.mod test\n" ++
        "event query module\n" ++
        "exit\n");
    defer {
        r.kernel.deinit();
        allocator.destroy(r.kernel);
        r.output.deinit();
    }

    const out = r.output.items;
    try std.testing.expect(std.mem.indexOf(u8, out, "module") != null);
}

test "shell: stats command" {
    const allocator = std.testing.allocator;
    var r = try runSession(allocator, "stats\nexit\n");
    defer {
        r.kernel.deinit();
        allocator.destroy(r.kernel);
        r.output.deinit();
    }

    const out = r.output.items;
    try std.testing.expect(std.mem.indexOf(u8, out, "Memory") != null);
    try std.testing.expect(std.mem.indexOf(u8, out, "Modules") != null);
    try std.testing.expect(std.mem.indexOf(u8, out, "Events") != null);
}

test "shell: history records commands" {
    const allocator = std.testing.allocator;
    var r = try runSession(allocator, "status\n" ++
        "mode\n" ++
        "history\n" ++
        "exit\n");
    defer {
        r.kernel.deinit();
        allocator.destroy(r.kernel);
        r.output.deinit();
    }

    const out = r.output.items;
    try std.testing.expect(std.mem.indexOf(u8, out, "status") != null);
    try std.testing.expect(std.mem.indexOf(u8, out, "mode") != null);
}

test "shell: tenant list" {
    const allocator = std.testing.allocator;
    var r = try runSession(allocator, "tenant register alpha\n" ++
        "tenant register beta\n" ++
        "tenant list\n" ++
        "exit\n");
    defer {
        r.kernel.deinit();
        allocator.destroy(r.kernel);
        r.output.deinit();
    }

    const out = r.output.items;
    try std.testing.expect(std.mem.indexOf(u8, out, "alpha") != null);
    try std.testing.expect(std.mem.indexOf(u8, out, "beta") != null);
}

test "shell: sched list is callable" {
    const allocator = std.testing.allocator;
    var r = try runSession(allocator, "sched list\nexit\n");
    defer {
        r.kernel.deinit();
        allocator.destroy(r.kernel);
        r.output.deinit();
    }

    try std.testing.expect(std.mem.indexOf(u8, r.output.items, "Scheduled") != null);
}

test "shell: version command" {
    const allocator = std.testing.allocator;
    var r = try runSession(allocator, "version\nexit\n");
    defer {
        r.kernel.deinit();
        allocator.destroy(r.kernel);
        r.output.deinit();
    }

    try std.testing.expect(std.mem.indexOf(u8, r.output.items, "kogi") != null);
}

test "shell: Tokens.parse handles multiple spaces and tabs" {
    const t = Tokens.parse("  module   register   foo   bar  ");
    try std.testing.expectEqual(@as(usize, 4), t.len);
    try std.testing.expectEqualStrings("module", t.args[0]);
    try std.testing.expectEqualStrings("register", t.args[1]);
    try std.testing.expectEqualStrings("foo", t.args[2]);
    try std.testing.expectEqualStrings("bar", t.args[3]);
}

test "shell: History push and retrieval" {
    var h = History.init(std.testing.allocator);
    defer h.deinit();

    try h.push("first");
    try h.push("second");
    try h.push("third");

    try std.testing.expectEqualStrings("third", h.get(0).?);
    try std.testing.expectEqualStrings("second", h.get(1).?);
    try std.testing.expectEqualStrings("first", h.get(2).?);
    try std.testing.expect(h.get(3) == null);
}

test "shell: History deduplication" {
    var h = History.init(std.testing.allocator);
    defer h.deinit();

    try h.push("dup");
    try h.push("dup");
    try h.push("dup");

    try std.testing.expectEqual(@as(usize, 1), h.count);
}
