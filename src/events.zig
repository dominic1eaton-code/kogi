//! KOGI Event Management System - Event bus and event handling
//! Provides publish-subscribe pattern for system events

const std = @import("std");

/// Event types in the KOGI OS
pub const EventType = enum {
    // Identity events
    identity_created,
    identity_deleted,
    identity_updated,
    identity_role_changed,

    // Task events
    task_created,
    task_completed,
    task_assigned,
    task_deleted,

    // Engagement events
    engagement_created,
    engagement_completed,
    engagement_terminated,

    // Workspace events
    workspace_created,
    workspace_updated,
    collection_created,
    item_added,

    // Connection events
    connection_added,
    connection_removed,

    // Directory events
    organization_added,
    contact_added,

    // Vault events
    vault_item_added,
    vault_item_removed,

    // Security events
    session_created,
    session_ended,
    role_assigned,

    // State events
    checkpoint_created,
    backup_completed,
    restore_completed,

    // Node events
    node_joined,
    node_left,
    node_failed,

    // System events
    system_started,
    system_shutdown,
    system_error,
};

/// Event payload
pub const Event = struct {
    id: u32,
    event_type: EventType,
    timestamp: i64,
    source_id: u32,
    entity_id: ?u32 = null,
    entity_type: ?[]const u8 = null,
    data: std.StringHashMap([]const u8),
    metadata: std.StringHashMap([]const u8),
};

/// Event handler function type
pub const EventHandler = struct {
    handler_id: u32,
    event_type: EventType,
    callback: *const fn (*Event) anyerror!void,
};

/// Event bus for publishing and subscribing to events
pub const EventBus = struct {
    allocator: std.mem.Allocator,
    events: std.ArrayList(Event),
    handlers: std.ArrayList(EventHandler),
    subscribers: std.StringHashMap(std.ArrayList(EventHandler)),
    next_event_id: u32 = 0,
    next_handler_id: u32 = 0,
    mutex: std.Thread.Mutex = .{},
    max_events: usize = 50000,

    pub fn init(allocator: std.mem.Allocator) EventBus {
        return EventBus{
            .allocator = allocator,
            .events = std.ArrayList(Event){},
            .handlers = std.ArrayList(EventHandler){},
            .subscribers = std.StringHashMap(std.ArrayList(EventHandler)).init(allocator),
        };
    }

    pub fn deinit(self: *EventBus) void {
        for (self.events.items) |event| {
            var data = event.data;
            var iter = data.iterator();
            while (iter.next()) |kv| {
                self.allocator.free(kv.key_ptr.*);
                self.allocator.free(kv.value_ptr.*);
            }
            data.deinit();

            var metadata = event.metadata;
            iter = metadata.iterator();
            while (iter.next()) |kv| {
                self.allocator.free(kv.key_ptr.*);
                self.allocator.free(kv.value_ptr.*);
            }
            metadata.deinit();

            if (event.entity_type) |et| {
                self.allocator.free(et);
            }
        }
        self.events.deinit(self.allocator);
        self.handlers.deinit(self.allocator);

        var iter = self.subscribers.iterator();
        while (iter.next()) |kv| {
            self.allocator.free(kv.key_ptr.*);
            kv.value_ptr.deinit(self.allocator);
        }
        self.subscribers.deinit();
    }

    /// Subscribe to an event type
    pub fn subscribe(
        self: *EventBus,
        event_type: EventType,
        callback: *const fn (*Event) anyerror!void,
    ) !u32 {
        self.mutex.lock();
        defer self.mutex.unlock();

        const handler_id = self.next_handler_id;
        self.next_handler_id += 1;

        const handler = EventHandler{
            .handler_id = handler_id,
            .event_type = event_type,
            .callback = callback,
        };

        try self.handlers.append(self.allocator, handler);
        return handler_id;
    }

    /// Unsubscribe from event type
    pub fn unsubscribe(self: *EventBus, handler_id: u32) void {
        self.mutex.lock();
        defer self.mutex.unlock();

        for (self.handlers.items, 0..) |_, idx| {
            if (self.handlers.items[idx].handler_id == handler_id) {
                _ = self.handlers.orderedRemove(idx);
                return;
            }
        }
    }

    /// Publish an event
    pub fn publish(self: *EventBus, event: Event) !void {
        self.mutex.lock();
        defer self.mutex.unlock();

        var event_mut = event;
        event_mut.id = self.next_event_id;
        self.next_event_id += 1;

        if (self.events.items.len >= self.max_events) {
            const old_event = self.events.orderedRemove(0);
            var data = old_event.data;
            var iter = data.iterator();
            while (iter.next()) |kv| {
                self.allocator.free(kv.key_ptr.*);
                self.allocator.free(kv.value_ptr.*);
            }
            data.deinit();
            var metadata = old_event.metadata;
            metadata.deinit();
            if (old_event.entity_type) |et| {
                self.allocator.free(et);
            }
        }

        try self.events.append(self.allocator, event_mut);

        // Call handlers
        for (self.handlers.items) |handler| {
            if (handler.event_type == event_mut.event_type) {
                var event_copy = event_mut;
                handler.callback(&event_copy) catch |e| {
                    // Log error but continue
                    _ = e;
                };
            }
        }
    }

    /// Get all events
    pub fn getEvents(self: *EventBus) []Event {
        return self.events.items;
    }

    /// Get events by type
    pub fn getEventsByType(self: *EventBus, event_type: EventType, allocator: std.mem.Allocator) !std.ArrayList(Event) {
        var results = std.ArrayList(Event){};
        for (self.events.items) |event| {
            if (event.event_type == event_type) {
                try results.append(allocator, event);
            }
        }
        return results;
    }

    /// Clear old events
    pub fn clearOldEvents(self: *EventBus, timestamp_threshold: i64) void {
        self.mutex.lock();
        defer self.mutex.unlock();

        var idx: usize = 0;
        while (idx < self.events.items.len) {
            if (self.events.items[idx].timestamp < timestamp_threshold) {
                const old_event = self.events.orderedRemove(idx);
                var data = old_event.data;
                var iter = data.iterator();
                while (iter.next()) |kv| {
                    self.allocator.free(kv.key_ptr.*);
                    self.allocator.free(kv.value_ptr.*);
                }
                data.deinit();
                var metadata = old_event.metadata;
                metadata.deinit();
                if (old_event.entity_type) |et| {
                    self.allocator.free(et);
                }
            } else {
                idx += 1;
            }
        }
    }
};
