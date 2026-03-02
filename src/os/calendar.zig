//! KOGI Calendar & Scheduling System - Time and Date management for ICPUs

const std = @import("std");

/// A calendar event
pub const CalendarEvent = struct {
    id: u32,
    title: []const u8,
    description: []const u8,
    start_time: i64,
    end_time: i64,
    all_day: bool,
    owner_id: u32, // ICPU id
};

/// Recurrence rule type
pub const Recurrence = enum {
    none,
    daily,
    weekly,
    monthly,
    yearly,
};

/// Scheduler managing events and clock
pub const Scheduler = struct {
    allocator: std.mem.Allocator,
    events: std.ArrayList(CalendarEvent),
    next_id: u32,
    mutex: std.Thread.Mutex = .{},

    pub fn init(allocator: std.mem.Allocator) Scheduler {
        return Scheduler{
            .allocator = allocator,
            .events = std.ArrayList(CalendarEvent){},
            .next_id = 1,
        };
    }

    pub fn deinit(self: *Scheduler) void {
        for (self.events.items) |evt| {
            self.allocator.free(evt.title);
            self.allocator.free(evt.description);
        }
        self.events.deinit(self.allocator);
    }

    pub fn addEvent(
        self: *Scheduler,
        title: []const u8,
        description: []const u8,
        start_time: i64,
        end_time: i64,
        all_day: bool,
        owner_id: u32,
    ) !u32 {
        self.mutex.lock();
        defer self.mutex.unlock();
        const id = self.next_id;
        self.next_id += 1;
        const evt = CalendarEvent{
            .id = id,
            .title = try self.allocator.dupe(u8, title),
            .description = try self.allocator.dupe(u8, description),
            .start_time = start_time,
            .end_time = end_time,
            .all_day = all_day,
            .owner_id = owner_id,
        };
        try self.events.append(self.allocator, evt);
        return id;
    }

    pub fn listEvents(self: *Scheduler) []CalendarEvent {
        return self.events.items;
    }

    pub fn getEventsByOwner(self: *Scheduler, owner_id: u32, allocator: std.mem.Allocator) !std.ArrayList(CalendarEvent) {
        var res = std.ArrayList(CalendarEvent){};
        for (self.events.items) |evt| {
            if (evt.owner_id == owner_id) {
                try res.append(allocator, evt);
            }
        }
        return res;
    }
};
