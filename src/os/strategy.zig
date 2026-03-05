const std = @import("std");

pub const Vision = struct {
    statement: []const u8,
};

pub const Mission = struct {
    statement: []const u8,
};

pub const Objective = struct {
    id: u32,
    description: []const u8,
    success_metric: []const u8,
};

pub const Goal = struct {
    id: u32,
    description: []const u8,
    due_date: i64,
    completed: bool,
};

pub const Outcome = struct {
    id: u32,
    description: []const u8,
    achieved: bool,
};

pub const Tactic = struct {
    id: u32,
    description: []const u8,
    linked_goal_id: ?u32,
};

pub const Milestone = struct {
    id: u32,
    title: []const u8,
    due_date: i64,
    completed: bool,
};

pub const Roadmap = struct {
    milestones: std.array_list.Managed(Milestone),
};

pub const GanttEntry = struct {
    id: u32,
    title: []const u8,
    start_ts: i64,
    end_ts: i64,
};

pub const GanttChart = struct {
    entries: std.array_list.Managed(GanttEntry),
};

pub const TimelineEvent = struct {
    id: u32,
    title: []const u8,
    ts: i64,
    note: []const u8,
};

pub const Timeline = struct {
    events: std.array_list.Managed(TimelineEvent),
};

pub const Charter = struct {
    title: []const u8,
    owner: []const u8,
    summary: []const u8,
};

pub const Operation = struct {
    id: u32,
    name: []const u8,
    running: bool,
};

pub const OperationsManager = struct {
    allocator: std.mem.Allocator,
    operations: std.array_list.Managed(Operation),
    next_id: u32,

    pub fn init(allocator: std.mem.Allocator) OperationsManager {
        return OperationsManager{
            .allocator = allocator,
            .operations = std.array_list.Managed(Operation){},
            .next_id = 1,
        };
    }

    pub fn deinit(self: *OperationsManager) void {
        for (self.operations.items) |op| {
            if (op.name) |n| self.allocator.free(n);
        }
        self.operations.deinit(self.allocator);
    }

    pub fn createOperation(self: *OperationsManager, name: []const u8) !u32 {
        const id = self.next_id;
        self.next_id += 1;
        const len = std.mem.len(name);
        const buf = try self.allocator.alloc(u8, len);
        std.mem.copy(u8, buf, name);
        const op = Operation{ .id = id, .name = buf, .running = false };
        try self.operations.append(self.allocator, op);
        return id;
    }

    pub fn startOperation(self: *OperationsManager, id: u32) bool {
        for (self.operations.items) |*op| {
            if (op.id == id) {
                op.running = true;
                return true;
            }
        }
        return false;
    }
};

pub const StrategicManager = struct {
    allocator: std.mem.Allocator,
    vision: ?Vision,
    mission: ?Mission,
    objectives: std.array_list.Managed(Objective),
    goals: std.array_list.Managed(Goal),
    outcomes: std.array_list.Managed(Outcome),
    tactics: std.array_list.Managed(Tactic),
    roadmaps: std.array_list.Managed(Roadmap),
    gantts: std.array_list.Managed(GanttChart),
    timelines: std.array_list.Managed(Timeline),
    charters: std.array_list.Managed(Charter),
    operations_mgr: OperationsManager,
    next_id: u32,

    pub fn init(allocator: std.mem.Allocator) StrategicManager {
        return StrategicManager{
            .allocator = allocator,
            .vision = null,
            .mission = null,
            .objectives = std.array_list.Managed(Objective){},
            .goals = std.array_list.Managed(Goal){},
            .outcomes = std.array_list.Managed(Outcome){},
            .tactics = std.array_list.Managed(Tactic){},
            .roadmaps = std.array_list.Managed(Roadmap){},
            .gantts = std.array_list.Managed(GanttChart){},
            .timelines = std.array_list.Managed(Timeline){},
            .charters = std.array_list.Managed(Charter){},
            .operations_mgr = OperationsManager.init(allocator),
            .next_id = 1,
        };
    }

    pub fn deinit(self: *StrategicManager) void {
        // free strings inside objectives
        for (self.objectives.items) |o| {
            if (o.description) |d| self.allocator.free(d);
            if (o.success_metric) |m| self.allocator.free(m);
        }
        self.objectives.deinit(self.allocator);

        for (self.goals.items) |g| {
            if (g.description) |d| self.allocator.free(d);
        }
        self.goals.deinit(self.allocator);

        for (self.outcomes.items) |o| {
            if (o.description) |d| self.allocator.free(d);
        }
        self.outcomes.deinit(self.allocator);

        for (self.tactics.items) |t| {
            if (t.description) |d| self.allocator.free(d);
        }
        self.tactics.deinit(self.allocator);

        // Deinit roadmaps, gantts, timelines (shallow)
        for (self.roadmaps.items) |r| r.milestones.deinit(self.allocator);
        self.roadmaps.deinit(self.allocator);

        for (self.gantts.items) |g| g.entries.deinit(self.allocator);
        self.gantts.deinit(self.allocator);

        for (self.timelines.items) |t| t.events.deinit(self.allocator);
        self.timelines.deinit(self.allocator);

        for (self.charters.items) |c| {
            if (c.title) |t| self.allocator.free(t);
            if (c.owner) |o| self.allocator.free(o);
            if (c.summary) |s| self.allocator.free(s);
        }
        self.charters.deinit(self.allocator);

        // Deinit operations manager
        self.operations_mgr.deinit();
    }

    pub fn setVision(self: *StrategicManager, statement: []const u8) !void {
        if (self.vision) |v| {
            if (v.statement) |s| self.allocator.free(s);
        }
        const len = std.mem.len(statement);
        const buf = try self.allocator.alloc(u8, len);
        std.mem.copy(u8, buf, statement);
        self.vision = Vision{ .statement = buf };
    }

    pub fn setMission(self: *StrategicManager, statement: []const u8) !void {
        if (self.mission) |m| {
            if (m.statement) |s| self.allocator.free(s);
        }
        const len = std.mem.len(statement);
        const buf = try self.allocator.alloc(u8, len);
        std.mem.copy(u8, buf, statement);
        self.mission = Mission{ .statement = buf };
    }

    pub fn createObjective(self: *StrategicManager, description: []const u8, metric: []const u8) !u32 {
        const id = self.next_id;
        self.next_id += 1;
        const dlen = std.mem.len(description);
        const mlen = std.mem.len(metric);
        const dcopy = try self.allocator.alloc(u8, dlen);
        const mcopy = try self.allocator.alloc(u8, mlen);
        std.mem.copy(u8, dcopy, description);
        std.mem.copy(u8, mcopy, metric);
        const obj = Objective{ .id = id, .description = dcopy, .success_metric = mcopy };
        try self.objectives.append(self.allocator, obj);
        return id;
    }

    pub fn createGoal(self: *StrategicManager, description: []const u8, due_date: i64) !u32 {
        const id = self.next_id;
        self.next_id += 1;
        const dlen = std.mem.len(description);
        const dcopy = try self.allocator.alloc(u8, dlen);
        std.mem.copy(u8, dcopy, description);
        const g = Goal{ .id = id, .description = dcopy, .due_date = due_date, .completed = false };
        try self.goals.append(self.allocator, g);
        return id;
    }

    pub fn createTactic(self: *StrategicManager, description: []const u8, linked_goal: ?u32) !u32 {
        const id = self.next_id;
        self.next_id += 1;
        const dlen = std.mem.len(description);
        const dcopy = try self.allocator.alloc(u8, dlen);
        std.mem.copy(u8, dcopy, description);
        const t = Tactic{ .id = id, .description = dcopy, .linked_goal_id = linked_goal };
        try self.tactics.append(self.allocator, t);
        return id;
    }

    pub fn createRoadmap(self: *StrategicManager) !u32 {
        const id = self.next_id;
        self.next_id += 1;
        var r = Roadmap{ .milestones = std.array_list.Managed(Milestone){} };
        try r.milestones.append(self.allocator, Milestone{ .id = 0, .title = "", .due_date = 0, .completed = false }) catch {}; // keep list initialized
        // remove placeholder
        r.milestones.pop() catch {};
        try self.roadmaps.append(self.allocator, r);
        return id;
    }

    pub fn addMilestoneToRoadmap(self: *StrategicManager, roadmap_index: usize, title: []const u8, due_date: i64) !u32 {
        if (roadmap_index >= self.roadmaps.items.len) return error.IndexOutOfBounds;
        const id = self.next_id;
        self.next_id += 1;
        const tlen = std.mem.len(title);
        const tcopy = try self.allocator.alloc(u8, tlen);
        std.mem.copy(u8, tcopy, title);
        const m = Milestone{ .id = id, .title = tcopy, .due_date = due_date, .completed = false };
        try self.roadmaps.items[roadmap_index].milestones.append(self.allocator, m);
        return id;
    }

    pub fn createGantt(self: *StrategicManager) !u32 {
        const id = self.next_id;
        self.next_id += 1;
        var g = GanttChart{ .entries = std.array_list.Managed(GanttEntry){} };
        try g.entries.append(self.allocator, GanttEntry{ .id = 0, .title = "", .start_ts = 0, .end_ts = 0 }) catch {};
        g.entries.pop() catch {};
        try self.gantts.append(self.allocator, g);
        return id;
    }

    pub fn addGanttEntry(self: *StrategicManager, gantt_index: usize, title: []const u8, start_ts: i64, end_ts: i64) !u32 {
        if (gantt_index >= self.gantts.items.len) return error.IndexOutOfBounds;
        const id = self.next_id;
        self.next_id += 1;
        const tlen = std.mem.len(title);
        const tcopy = try self.allocator.alloc(u8, tlen);
        std.mem.copy(u8, tcopy, title);
        const e = GanttEntry{ .id = id, .title = tcopy, .start_ts = start_ts, .end_ts = end_ts };
        try self.gantts.items[gantt_index].entries.append(self.allocator, e);
        return id;
    }

    pub fn createTimeline(self: *StrategicManager) !u32 {
        const id = self.next_id;
        self.next_id += 1;
        var t = Timeline{ .events = std.array_list.Managed(TimelineEvent){} };
        try t.events.append(self.allocator, TimelineEvent{ .id = 0, .title = "", .ts = 0, .note = "" }) catch {};
        t.events.pop() catch {};
        try self.timelines.append(self.allocator, t);
        return id;
    }

    pub fn addTimelineEvent(self: *StrategicManager, timeline_index: usize, title: []const u8, ts: i64, note: []const u8) !u32 {
        if (timeline_index >= self.timelines.items.len) return error.IndexOutOfBounds;
        const id = self.next_id;
        self.next_id += 1;
        const tlen = std.mem.len(title);
        const nlen = std.mem.len(note);
        const tcopy = try self.allocator.alloc(u8, tlen);
        const ncopy = try self.allocator.alloc(u8, nlen);
        std.mem.copy(u8, tcopy, title);
        std.mem.copy(u8, ncopy, note);
        const e = TimelineEvent{ .id = id, .title = tcopy, .ts = ts, .note = ncopy };
        try self.timelines.items[timeline_index].events.append(self.allocator, e);
        return id;
    }

    pub fn createCharter(self: *StrategicManager, title: []const u8, owner: []const u8, summary: []const u8) !void {
        const tlen = std.mem.len(title);
        const olen = std.mem.len(owner);
        const slen = std.mem.len(summary);
        const tcopy = try self.allocator.alloc(u8, tlen);
        const ocopy = try self.allocator.alloc(u8, olen);
        const scopy = try self.allocator.alloc(u8, slen);
        std.mem.copy(u8, tcopy, title);
        std.mem.copy(u8, ocopy, owner);
        std.mem.copy(u8, scopy, summary);
        const c = Charter{ .title = tcopy, .owner = ocopy, .summary = scopy };
        try self.charters.append(self.allocator, c);
    }

    pub fn evaluateProgress(self: *StrategicManager) f32 {
        // simple heuristic: completed goals / total goals
        var total: usize = 0;
        var done: usize = 0;
        for (self.goals.items) |g| {
            total += 1;
            if (g.completed) done += 1;
        }
        if (total == 0) return 0.0;
        return @as(f32, @floatFromInt(done)) / @as(f32, @floatFromInt(total));
    }
};
