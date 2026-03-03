const std = @import("std");

/// Simple terminal helper for interactive CLI input/output
pub const Terminal = struct {
    allocator: std.mem.Allocator,
    prompt: []const u8,

    pub fn init(allocator: std.mem.Allocator, prompt: []const u8) Terminal {
        return Terminal{ .allocator = allocator, .prompt = prompt };
    }

    pub fn writePrompt(self: *Terminal) !void {
        var stdout = std.io.getStdOut().writer();
        try stdout.print("{s}", .{self.prompt});
        try stdout.flush();
    }

    /// Read a line from stdin (allocates with `allocator`). Caller must free.
    pub fn readLine(self: *Terminal) ![]u8 {
        var reader = std.io.getStdIn().reader();
        // read until newline, limit to 8KiB
        const buf = try reader.readUntilDelimiterAlloc(self.allocator, '\n', 8192);
        // Trim trailing CR and LF
        var end = buf.len;
        while (end > 0 and (buf[end - 1] == '\n' or buf[end - 1] == '\r')) : (end -= 1) {}
        return buf[0..end];
    }
};
