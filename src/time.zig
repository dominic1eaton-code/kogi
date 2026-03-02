//! KOGI Time & Clock Management System
//! Provides precise timekeeping, time zones, scheduling, and profiling.

const std = @import("std");

/// Represents a point in time with high precision
pub const TimePoint = struct {
    seconds: i64,      // Unix epoch seconds
    nanoseconds: u32,  // Additional nanoseconds [0, 999_999_999]
    
    pub fn now() TimePoint {
        return fromNanoseconds(std.time.nanoTimestamp());
    }
    
    pub fn fromNanoseconds(nanos: i128) TimePoint {
        const secs = nanos / 1_000_000_000;
        const nanos_remainder = @as(u32, @intCast(@mod(nanos, 1_000_000_000)));
        return TimePoint{
            .seconds = @intCast(secs),
            .nanoseconds = nanos_remainder,
        };
    }
    
    pub fn toNanoseconds(self: TimePoint) i128 {
        return @as(i128, self.seconds) * 1_000_000_000 + @as(i128, self.nanoseconds);
    }
    
    pub fn duration(self: TimePoint, other: TimePoint) Duration {
        const diff = self.toNanoseconds() - other.toNanoseconds();
        return Duration.fromNanoseconds(diff);
    }
};

/// Represents a duration between two time points
pub const Duration = struct {
    nanos: i128,  // Nanoseconds
    
    pub fn fromNanoseconds(nanos: i128) Duration {
        return Duration{ .nanos = nanos };
    }
    
    pub fn fromMicroseconds(micros: i64) Duration {
        return Duration{ .nanos = @as(i128, micros) * 1_000 };
    }
    
    pub fn fromMilliseconds(millis: i64) Duration {
        return Duration{ .nanos = @as(i128, millis) * 1_000_000 };
    }
    
    pub fn fromSeconds(secs: i64) Duration {
        return Duration{ .nanos = @as(i128, secs) * 1_000_000_000 };
    }
    
    pub fn fromMinutes(mins: i64) Duration {
        return Duration.fromSeconds(mins * 60);
    }
    
    pub fn fromHours(hours: i64) Duration {
        return Duration.fromMinutes(hours * 60);
    }
    
    pub fn fromDays(days: i64) Duration {
        return Duration.fromHours(days * 24);
    }
    
    pub fn toNanoseconds(self: Duration) i128 {
        return self.nanos;
    }
    
    pub fn toMicroseconds(self: Duration) i64 {
        return @intCast(@divFloor(self.nanos, 1_000));
    }
    
    pub fn toMilliseconds(self: Duration) i64 {
        return @intCast(@divFloor(self.nanos, 1_000_000));
    }
    
    pub fn toSeconds(self: Duration) i64 {
        return @intCast(@divFloor(self.nanos, 1_000_000_000));
    }
};

/// Calendar date and time breakdown
pub const DateTime = struct {
    year: u16,
    month: u4,    // 1-12
    day: u5,      // 1-31
    hour: u5,     // 0-23
    minute: u6,   // 0-59
    second: u6,   // 0-59
    nanosecond: u32, // 0-999_999_999
    
    pub fn now() DateTime {
        return TimePoint.now().toDateTime();
    }
    
    pub fn toTimePoint(self: DateTime) TimePoint {
        // Simplified conversion (assumes UTC)
        // In production, use a proper date/time library
        const epoch_days = daysSinceEpoch(self.year, self.month, self.day);
        const total_secs = epoch_days * 86400 + 
                          self.hour * 3600 + 
                          self.minute * 60 + 
                          self.second;
        
        return TimePoint{
            .seconds = total_secs,
            .nanoseconds = self.nanosecond,
        };
    }
};

pub fn daysSinceEpoch(year: u16, month: u4, day: u5) i64 {
    // Simplified calculation (not leap-year aware for full accuracy)
    var days: i64 = 0;
    var y = year;
    
    // Count days for complete years since 1970
    while (y > 1970) : (y -= 1) {
        days += if (isLeapYear(y)) 366 else 365;
    }
    
    // Add days for months in current year
    const month_days = [12]u5{ 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31 };
    var m: u4 = 1;
    while (m < month) : (m += 1) {
        days += month_days[m - 1];
        if (m == 2 and isLeapYear(year)) {
            days += 1;
        }
    }
    
    days += day - 1;
    return days;
}

fn isLeapYear(year: u16) bool {
    return (year % 4 == 0 and year % 100 != 0) or (year % 400 == 0);
}

pub fn TimePointToDateTime(tp: TimePoint) DateTime {
    // Simplified conversion from TimePoint to DateTime
    // In production, use a proper date/time library
    
    var remaining = tp.seconds;
    var year: u16 = 1970;
    
    // Find year
    while (true) {
        const days_in_year = if (isLeapYear(year)) @as(i64, 366) else 365;
        const secs_in_year = days_in_year * 86400;
        if (remaining < secs_in_year) break;
        remaining -= secs_in_year;
        year += 1;
    }
    
    // Find month and day
    const month_days = [12]u5{ 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31 };
    var month: u4 = 1;
    while (month <= 12) : (month += 1) {
        var days = month_days[month - 1];
        if (month == 2 and isLeapYear(year)) days = 29;
        
        const secs_in_month = @as(i64, days) * 86400;
        if (remaining < secs_in_month) break;
        remaining -= secs_in_month;
    }
    
    const day: u5 = @intCast(remaining / 86400) + 1;
    remaining = remaining % 86400;
    
    const hour: u5 = @intCast(remaining / 3600);
    remaining = remaining % 3600;
    
    const minute: u6 = @intCast(remaining / 60);
    const second: u6 = @intCast(remaining % 60);
    
    return DateTime{
        .year = year,
        .month = month,
        .day = day,
        .hour = hour,
        .minute = minute,
        .second = second,
        .nanosecond = tp.nanoseconds,
    };
}

/// Time zone representation
pub const TimeZone = struct {
    name: []const u8,
    offset_hours: i8,
    offset_minutes: u6,
    is_dst: bool,
    
    pub const UTC = TimeZone{
        .name = "UTC",
        .offset_hours = 0,
        .offset_minutes = 0,
        .is_dst = false,
    };
};

/// Clock managing system time
pub const Clock = struct {
    allocator: std.mem.Allocator,
    base_time: i64,           // Reference time in nanoseconds
    start_time: i64,          // System start time
    timezone: TimeZone,
    paused: bool = false,
    pause_offset: i64 = 0,
    mutex: std.Thread.Mutex = .{},
    
    pub fn init(allocator: std.mem.Allocator) Clock {
        const now = std.time.nanoTimestamp();
        return Clock{
            .allocator = allocator,
            .base_time = now,
            .start_time = now,
            .timezone = TimeZone.UTC,
        };
    }
    
    pub fn now(self: *Clock) TimePoint {
        self.mutex.lock();
        defer self.mutex.unlock();
        
        const current = std.time.nanoTimestamp();
        const elapsed = if (self.paused) self.pause_offset else (current - self.base_time);
        
        return TimePoint.fromNanoseconds(self.start_time + elapsed);
    }
    
    pub fn uptime(self: *Clock) Duration {
        const start_tp = TimePoint.fromNanoseconds(self.start_time);
        const now_tp = self.now();
        return start_tp.duration(now_tp);
    }
    
    pub fn setTimezone(self: *Clock, tz: TimeZone) void {
        self.mutex.lock();
        defer self.mutex.unlock();
        self.timezone = tz;
    }
    
    pub fn getTimezone(self: *Clock) TimeZone {
        self.mutex.lock();
        defer self.mutex.unlock();
        return self.timezone;
    }
    
    pub fn pause(self: *Clock) void {
        self.mutex.lock();
        defer self.mutex.unlock();
        
        if (!self.paused) {
            const current = std.time.nanoTimestamp();
            self.pause_offset = current - self.base_time;
            self.paused = true;
        }
    }
    pub fn unpause(self: *Clock) void {
        self.mutex.lock();
        defer self.mutex.unlock();
        
        if (self.paused) {
            const current = std.time.nanoTimestamp();
            self.base_time = current - self.pause_offset;
            self.paused = false;
        }
    }
};

/// Timer for measuring elapsed time
pub const Timer = struct {
    start: TimePoint,
    label: []const u8,
    allocator: std.mem.Allocator,
    
    pub fn start(allocator: std.mem.Allocator, label: []const u8) !Timer {
        return Timer{
            .start = TimePoint.now(),
            .label = try allocator.dupe(u8, label),
            .allocator = allocator,
        };
    }
    
    pub fn elapsed(self: Timer) Duration {
        const end = TimePoint.now();
        return self.start.duration(end);
    }
    
    pub fn stop(self: Timer) void {
        const dur = self.elapsed();
        std.debug.print("[{s}] Elapsed: {d} ms\n", .{ self.label, dur.toMilliseconds() });
        self.allocator.free(self.label);
    }
};

/// Profiler for performance analysis
pub const Profiler = struct {
    allocator: std.mem.Allocator,
    timers: std.StringHashMap(std.ArrayList(Duration)),
    total_samples: u32 = 0,
    mutex: std.Thread.Mutex = .{},
    
    pub fn init(allocator: std.mem.Allocator) Profiler {
        return Profiler{
            .allocator = allocator,
            .timers = std.StringHashMap(std.ArrayList(Duration)).init(allocator),
        };
    }
    
    pub fn deinit(self: *Profiler) void {
        var iter = self.timers.iterator();
        while (iter.next()) |entry| {
            entry.value_ptr.deinit(self.allocator);
            self.allocator.free(entry.key_ptr.*);
        }
        self.timers.deinit();
    }
    
    pub fn recordDuration(self: *Profiler, name: []const u8, duration: Duration) !void {
        self.mutex.lock();
        defer self.mutex.unlock();
        
        const name_dup = try self.allocator.dupe(u8, name);
        
        if (self.timers.getPtr(name)) |list_ptr| {
            try list_ptr.append(self.allocator, duration);
        } else {
            var list = std.ArrayList(Duration){};
            try list.append(self.allocator, duration);
            try self.timers.put(name_dup, list);
        }
        
        self.total_samples += 1;
    }
    
    pub fn getStats(self: *Profiler, name: []const u8) ?struct { min: i64, max: i64, avg: i64, count: u32 } {
        self.mutex.lock();
        defer self.mutex.unlock();
        
        if (self.timers.get(name)) |list| {
            var min = std.math.maxInt(i64);
            var max = std.math.minInt(i64);
            var sum: i128 = 0;
            
            for (list.items) |dur| {
                const ms = dur.toMilliseconds();
                if (ms < min) min = ms;
                if (ms > max) max = ms;
                sum += ms;
            }
            
            const avg = @divFloor(sum, @as(i128, list.items.len));
            
            return .{
                .min = min,
                .max = max,
                .avg = @intCast(avg),
                .count = @intCast(list.items.len),
            };
        }
        
        return null;
    }
    
    pub fn printStats(self: *Profiler) void {
        std.debug.print("\n=== Profiler Results ===\n", .{});
        std.debug.print("Total samples: {d}\n", .{self.total_samples});
        
        self.mutex.lock();
        defer self.mutex.unlock();
        
        var iter = self.timers.iterator();
        while (iter.next()) |entry| {
            if (self.getStats(entry.key_ptr.*)) |stats| {
                std.debug.print("[{s}] min={d}ms max={d}ms avg={d}ms count={d}\n",
                    .{ entry.key_ptr.*, stats.min, stats.max, stats.avg, stats.count });
            }
        }
        std.debug.print("=======================\n\n", .{});
    }
};

/// Stopwatch for simple time measurement
pub const StopWatch = struct {
    start_time: TimePoint,
    is_running: bool,
    total_time: Duration,
    
    pub fn new() StopWatch {
        return StopWatch{
            .start_time = TimePoint.now(),
            .is_running = true,
            .total_time = Duration.fromNanoseconds(0),
        };
    }
    
    pub fn pause(self: *StopWatch) void {
        if (self.is_running) {
            const now = TimePoint.now();
            const elapsed = self.start_time.duration(now);
            self.total_time.nanos += elapsed.nanos;
            self.is_running = false;
        }
    }
    
    pub fn resume(self: *StopWatch) void {
        if (!self.is_running) {
            self.start_time = TimePoint.now();
            self.is_running = true;
        }
    }
    
    pub fn elapsed(self: StopWatch) Duration {
        var result = self.total_time;
        if (self.is_running) {
            const now = TimePoint.now();
            const current = self.start_time.duration(now);
            result.nanos += current.nanos;
        }
        return result;
    }
    
    pub fn reset(self: *StopWatch) void {
        self.start_time = TimePoint.now();
        self.is_running = true;
        self.total_time = Duration.fromNanoseconds(0);
    }
};

/// Time source interface for testing
pub const TimeSource = struct {
    ptr: *anyopaque,
    vtable: *const VTable,
    
    pub const VTable = struct {
        now: *const fn (*anyopaque) TimePoint,
        sleep: *const fn (*anyopaque, duration: Duration) void,
    };
    
    pub fn now(self: TimeSource) TimePoint {
        return self.vtable.now(self.ptr);
    }
    
    pub fn sleep(self: TimeSource, duration: Duration) void {
        return self.vtable.sleep(self.ptr, duration);
    }
};

/// System time source using real system clock
pub const SystemTimeSource = struct {
    pub fn now() TimePoint {
        return TimePoint.now();
    }
    
    pub fn sleep(duration: Duration) void {
        std.time.sleep(@intCast(duration.toNanoseconds()));
    }
};
