//! KOGI Logging System - Comprehensive logging infrastructure
//! Provides structured logging with multiple levels, outputs, and filtering

const std = @import("std");

/// Log levels in order of severity
pub const LogLevel = enum {
    trace,
    debug,
    info,
    warn,
    err,
    fatal,

    pub fn toString(self: LogLevel) []const u8 {
        return switch (self) {
            .trace => "TRACE",
            .debug => "DEBUG",
            .info => "INFO",
            .warn => "WARN",
            .err => "ERROR",
            .fatal => "FATAL",
        };
    }

    pub fn severity(self: LogLevel) u8 {
        return switch (self) {
            .trace => 0,
            .debug => 1,
            .info => 2,
            .warn => 3,
            .err => 4,
            .fatal => 5,
        };
    }
};

/// Log entry structure
pub const LogEntry = struct {
    timestamp: i64,
    level: LogLevel,
    module: []const u8,
    message: []const u8,
    context: []const u8,
    metadata: std.StringHashMap([]const u8),
};

/// Log output destination
pub const LogOutput = enum {
    stdout,
    stderr,
    file,
    memory,
    network,
};

/// Logger configuration
pub const LoggerConfig = struct {
    min_level: LogLevel = .info,
    outputs: std.ArrayList(LogOutput),
    file_path: ?[]const u8 = null,
    max_entries: usize = 10000,
    include_timestamp: bool = true,
    include_metadata: bool = true,
    buffer_size: usize = 4096,
    // log rotation settings (only applies when file output is used)
    max_file_size: u64 = 0, // bytes, 0 disables rotation
    max_files: u32 = 0, // number of rotated files to keep (0 disables rotation)
};

/// Main Logger implementation
pub const Logger = struct {
    allocator: std.mem.Allocator,
    config: LoggerConfig,
    entries: std.ArrayList(LogEntry),
    file_handle: ?std.fs.File = null,
    buffer: std.ArrayList(u8),
    mutex: std.Thread.Mutex = .{},
    current_file_size: u64 = 0,
    rotation_index: u32 = 0,

    pub fn init(allocator: std.mem.Allocator, config: LoggerConfig) !Logger {
        var logger = Logger{
            .allocator = allocator,
            .config = config,
            .entries = std.ArrayList(LogEntry){},
            .buffer = std.ArrayList(u8){},
        };

        try logger.entries.ensureTotalCapacity(allocator, config.max_entries);
        try logger.buffer.ensureTotalCapacity(allocator, config.buffer_size);

        if (config.file_path) |path| {
            const dir = std.fs.cwd();
            logger.file_handle = try dir.createFile(path, .{ .truncate = false });
            // determine current file size
            if (logger.file_handle) |handle| {
                const existing_size = try handle.getEndPos();
                logger.current_file_size = existing_size;
            }
        }

        return logger;
    }

    pub fn deinit(self: *Logger) void {
        for (self.entries.items) |entry| {
            self.allocator.free(entry.module);
            self.allocator.free(entry.message);
            self.allocator.free(entry.context);
            var metadata = entry.metadata;
            var iter = metadata.iterator();
            while (iter.next()) |kv| {
                self.allocator.free(kv.key_ptr.*);
                self.allocator.free(kv.value_ptr.*);
            }
            metadata.deinit();
        }
        self.entries.deinit(self.allocator);
        self.buffer.deinit(self.allocator);

        if (self.file_handle) |handle| {
            handle.close();
        }
    }

    /// Log a message at the specified level
    pub fn log(
        self: *Logger,
        level: LogLevel,
        module: []const u8,
        message: []const u8,
    ) !void {
        self.mutex.lock();
        defer self.mutex.unlock();

        if (level.severity() < self.config.min_level.severity()) {
            return;
        }

        const entry = LogEntry{
            .timestamp = std.time.timestamp(),
            .level = level,
            .module = try self.allocator.dupe(u8, module),
            .message = try self.allocator.dupe(u8, message),
            .context = try self.allocator.dupe(u8, ""),
            .metadata = std.StringHashMap([]const u8){},
        };

        if (self.entries.items.len >= self.config.max_entries) {
            const old_entry = self.entries.orderedRemove(0);
            self.allocator.free(old_entry.module);
            self.allocator.free(old_entry.message);
            self.allocator.free(old_entry.context);
            var metadata = old_entry.metadata;
            metadata.deinit();
        }

        try self.entries.append(self.allocator, entry);
        try self.writeOutput(entry);
    }

    /// Log with context information
    pub fn logWithContext(
        self: *Logger,
        level: LogLevel,
        module: []const u8,
        message: []const u8,
        context: []const u8,
    ) !void {
        self.mutex.lock();
        defer self.mutex.unlock();

        if (level.severity() < self.config.min_level.severity()) {
            return;
        }

        const entry = LogEntry{
            .timestamp = std.time.timestamp(),
            .level = level,
            .module = try self.allocator.dupe(u8, module),
            .message = try self.allocator.dupe(u8, message),
            .context = try self.allocator.dupe(u8, context),
            .metadata = std.StringHashMap([]const u8){},
        };

        if (self.entries.items.len >= self.config.max_entries) {
            const old_entry = self.entries.orderedRemove(0);
            self.allocator.free(old_entry.module);
            self.allocator.free(old_entry.message);
            self.allocator.free(old_entry.context);
            var metadata = old_entry.metadata;
            metadata.deinit();
        }

        try self.entries.append(self.allocator, entry);
        try self.writeOutput(entry);
    }

    /// Write output to configured destinations
    fn writeOutput(self: *Logger, entry: LogEntry) !void {
        self.buffer.clearRetainingCapacity();

        // Format log message
        if (self.config.include_timestamp) {
            try self.buffer.writer().print("[{}] ", .{entry.timestamp});
        }
        try self.buffer.writer().print("[{s}] [{s}] {s}", .{
            entry.level.toString(),
            entry.module,
            entry.message,
        });

        if (entry.context.len > 0) {
            try self.buffer.writer().print(" | {s}", .{entry.context});
        }
        try self.buffer.writer().print("\n", .{});

        for (self.config.outputs.items) |output| {
            switch (output) {
                .stdout => {
                    try std.io.getStdOut().writeAll(self.buffer.items);
                },
                .stderr => {
                    try std.io.getStdErr().writeAll(self.buffer.items);
                },
                .file => {
                    if (self.file_handle) |handle| {
                        try handle.writeAll(self.buffer.items);
                        self.current_file_size += @as(u64, self.buffer.items.len);
                        // check rotation
                        if (self.config.max_file_size > 0 and self.current_file_size >= self.config.max_file_size) {
                            try self.rotateFile();
                        }
                    }
                },
                .memory => {
                    // Already stored in entries
                },
                .network => {
                    // Would be handled by network subsystem
                },
            }
        }
    }

    /// Rotate the log file when exceeding size limits
    fn rotateFile(self: *Logger) !void {
        if (self.config.file_path and self.config.max_files > 0) |path| {
            // close current handle
            if (self.file_handle) |handle| {
                try handle.close();
            }

            // compute rotated name
            const dir = std.fs.cwd();
            // increment index and wrap if necessary
            self.rotation_index += 1;
            if (self.rotation_index >= self.config.max_files) {
                self.rotation_index = 1;
            }
            const suffix = try std.fmt.allocPrint(self.allocator, ".{d}", .{self.rotation_index});
            const rotated_name = try std.fmt.allocPrint(self.allocator, "{s}.{s}", .{ path, suffix });
            // rename existing file
            _ = dir.renameFile(path, rotated_name) catch {};
            // open new log file
            self.file_handle = try dir.createFile(path, .{ .truncate = true });
            self.current_file_size = 0;
            self.allocator.free(suffix);
            self.allocator.free(rotated_name);
        }
    }

    /// Get log entries filtered by level
    pub fn getEntriesByLevel(self: *Logger, level: LogLevel, allocator: std.mem.Allocator) !std.ArrayList(LogEntry) {
        var results = std.ArrayList(LogEntry){};
        for (self.entries.items) |entry| {
            if (entry.level == level) {
                try results.append(allocator, entry);
            }
        }
        return results;
    }

    /// Get log entries filtered by module
    pub fn getEntriesByModule(self: *Logger, module: []const u8, allocator: std.mem.Allocator) !std.ArrayList(LogEntry) {
        var results = std.ArrayList(LogEntry){};
        for (self.entries.items) |entry| {
            if (std.mem.eql(u8, entry.module, module)) {
                try results.append(allocator, entry);
            }
        }
        return results;
    }

    /// Get all log entries
    pub fn getEntries(self: *Logger) []LogEntry {
        return self.entries.items;
    }

    /// Clear all log entries
    pub fn clear(self: *Logger) void {
        for (self.entries.items) |entry| {
            self.allocator.free(entry.module);
            self.allocator.free(entry.message);
            self.allocator.free(entry.context);
            var metadata = entry.metadata;
            metadata.deinit();
        }
        self.entries.clearRetainingCapacity();
    }

    // Convenience methods
    pub fn trace(self: *Logger, module: []const u8, message: []const u8) !void {
        try self.log(.trace, module, message);
    }

    pub fn debug(self: *Logger, module: []const u8, message: []const u8) !void {
        try self.log(.debug, module, message);
    }

    pub fn info(self: *Logger, module: []const u8, message: []const u8) !void {
        try self.log(.info, module, message);
    }

    pub fn warn(self: *Logger, module: []const u8, message: []const u8) !void {
        try self.log(.warn, module, message);
    }

    pub fn err(self: *Logger, module: []const u8, message: []const u8) !void {
        try self.log(.err, module, message);
    }

    pub fn fatal(self: *Logger, module: []const u8, message: []const u8) !void {
        try self.log(.fatal, module, message);
    }
};

/// Global logger instance
var global_logger: ?Logger = null;
var logger_mutex = std.Thread.Mutex{};

pub fn initGlobalLogger(allocator: std.mem.Allocator, config: LoggerConfig) !void {
    logger_mutex.lock();
    defer logger_mutex.unlock();

    if (global_logger != null) {
        global_logger.?.deinit();
    }
    global_logger = try Logger.init(allocator, config);
}

pub fn getGlobalLogger() ?*Logger {
    logger_mutex.lock();
    defer logger_mutex.unlock();
    if (global_logger) |*logger| {
        return &logger;
    }
    return null;
}

pub fn deinitGlobalLogger() void {
    logger_mutex.lock();
    defer logger_mutex.unlock();
    if (global_logger) |*logger| {
        logger.deinit();
        global_logger = null;
    }
}
