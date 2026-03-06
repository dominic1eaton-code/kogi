
/// Search filter criteria
pub const SearchFilter = struct {
    name_pattern: ?[]const u8 = null,
    entity_type: ?EntityType = null,
    entity_class: ?EntityClass = null,
    category: ?[]const u8 = null,
    tag_names: ?std.array_list.Managed([]const u8) = null,
    owner_id: ?u32 = null,
};
pub const SearchFilter = struct {
    name_pattern: ?[]const u8 = null,
    entity_type: ?EntityType = null,
    entity_class: ?EntityClass = null,
    category: ?[]const u8 = null,
    tag_names: ?std.array_list.Managed([]const u8) = null,
    owner_id: ?u32 = null,
};

/// Idea lifecycle states from capture to retirement.
pub const IdeaLifecycle = enum {
    captured,
    discovered,
    validated,
    planned,
    in_design,
    in_execution,
    launched,
    archived,
    rejected,
};

/// Timeline state for idea milestones and deliverables.
pub const IdeaTimelineStatus = enum {
    planned,
    in_progress,
    completed,
    blocked,
    canceled,
};

/// Version snapshot for an idea.
pub const IdeaVersion = struct {
    id: u32,
    version_number: u32,
    title: []const u8,
    summary: []const u8,
    details: []const u8,
    created_at: i64,
};

/// Free-form note attached to an idea.
pub const IdeaNote = struct {
    id: u32,
    title: []const u8,
    body: []const u8,
    created_at: i64,
    updated_at: i64,
};

/// Design artifact attached to an idea.
pub const IdeaDesign = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    artifact_ref: ?[]const u8 = null,
    created_at: i64,
    updated_at: i64,
};

/// Timeline event for planning and execution visibility.
pub const IdeaTimelineEvent = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    start_at: i64,
    end_at: ?i64 = null,
    status: IdeaTimelineStatus,
    created_at: i64,
    updated_at: i64,
};

/// Tracking fields for operational idea management.
pub const IdeaTracking = struct {
    priority: u8 = 0,
    impact_score: f32 = 0.0,
    effort_score: f32 = 0.0,
    confidence_score: f32 = 0.0,
    total_versions: u32 = 0,
    total_notes: u32 = 0,
    total_designs: u32 = 0,
    total_timeline_events: u32 = 0,
    last_activity_at: i64 = 0,
    next_review_at: ?i64 = null,
};

/// Core idea aggregate.
pub const Idea = struct {
    id: u32,
    title: []const u8,
    summary: []const u8,
    organization: []const u8,
    category: []const u8,
    lifecycle: IdeaLifecycle,
    owner_id: ?u32 = null,
    created_at: i64,
    updated_at: i64,
    tags: std.array_list.Managed([]const u8),
    versions: std.array_list.Managed(IdeaVersion),
    notes: std.array_list.Managed(IdeaNote),
    designs: std.array_list.Managed(IdeaDesign),
    timeline: std.array_list.Managed(IdeaTimelineEvent),
    tracking: IdeaTracking,
    timebox_start: ?i64 = null,
    timebox_end: ?i64 = null,
};

/// Filtering options for idea search and organization.
pub const IdeaFilter = struct {
    lifecycle: ?IdeaLifecycle = null,
    tag_name: ?[]const u8 = null,
    organization: ?[]const u8 = null,
    category: ?[]const u8 = null,
    owner_id: ?u32 = null,
    query: ?[]const u8 = null,
    active_at: ?i64 = null,
    updated_after: ?i64 = null,
    updated_before: ?i64 = null,
};

/// Lightweight view for idea search results.
pub const IdeaSummary = struct {
    id: u32,
    title: []const u8,
    lifecycle: IdeaLifecycle,
    organization: []const u8,
    category: []const u8,
    owner_id: ?u32,
    updated_at: i64,
    tag_count: u32,
    next_review_at: ?i64,
};

/// Portfolio Manager for hierarchical portfolio operations
pub const PortfolioManager = struct {
    allocator: std.mem.Allocator,
    portfolio: ?Portfolio = null,
    index: std.array_list.Managed(IndexEntry),
    next_id: u32 = 0,

pub const PortfolioManager = struct {
    allocator: std.mem.Allocator,
    portfolio: ?Portfolio = null,
    ideas: std.array_list.Managed(Idea),
    index: std.array_list.Managed(IndexEntry),
    next_id: u32 = 0,

    pub fn init(allocator: std.mem.Allocator) PortfolioManager {
        return PortfolioManager{
            .allocator = allocator,
            .ideas = std.array_list.Managed(Idea).init(allocator),
            .index = std.array_list.Managed(IndexEntry).init(allocator),
        };
    }

    pub fn deinit(self: *PortfolioManager) void {
        if (self.portfolio) |*portfolio| {
            self.deinitPortfolio(portfolio);
        }
        self.deinitIndex();
    }

    pub fn deinit(self: *PortfolioManager) void {
        if (self.portfolio) |*portfolio| {
            self.deinitPortfolio(portfolio);
        }
        self.deinitIdeas();
        self.deinitIndex();
    }

    fn deinitIndex(self: *PortfolioManager) void {
        for (self.index.items) |*entry| {
        self.deinitMetadata(&collection.metadata);
    }

    fn deinitTask(self: *PortfolioManager, task: *Task) void {
        self.allocator.free(task.title);
        self.allocator.free(task.description);
        self.deinitMetadata(&task.metadata);
    }
    fn deinitTask(self: *PortfolioManager, task: *Task) void {
        self.allocator.free(task.title);
        self.allocator.free(task.description);
        self.deinitMetadata(&task.metadata);
    }

    fn deinitIdeaVersion(self: *PortfolioManager, version: *IdeaVersion) void {
        self.allocator.free(version.title);
        self.allocator.free(version.summary);
        self.allocator.free(version.details);
    }

    fn deinitIdeaNote(self: *PortfolioManager, note: *IdeaNote) void {
        self.allocator.free(note.title);
        self.allocator.free(note.body);
    }

    fn deinitIdeaDesign(self: *PortfolioManager, design: *IdeaDesign) void {
        self.allocator.free(design.name);
        self.allocator.free(design.description);
        if (design.artifact_ref) |artifact| self.allocator.free(artifact);
    }

    fn deinitIdeaTimelineEvent(self: *PortfolioManager, event: *IdeaTimelineEvent) void {
        self.allocator.free(event.name);
        self.allocator.free(event.description);
    }

    fn deinitIdea(self: *PortfolioManager, idea: *Idea) void {
        self.allocator.free(idea.title);
        self.allocator.free(idea.summary);
        self.allocator.free(idea.organization);
        self.allocator.free(idea.category);

        for (idea.tags.items) |tag_name| {
            self.allocator.free(tag_name);
        }
        idea.tags.deinit();

        for (idea.versions.items) |*version| {
            self.deinitIdeaVersion(version);
        }
        idea.versions.deinit();

        for (idea.notes.items) |*note| {
            self.deinitIdeaNote(note);
        }
        idea.notes.deinit();

        for (idea.designs.items) |*design| {
            self.deinitIdeaDesign(design);
        }
        idea.designs.deinit();

        for (idea.timeline.items) |*event| {
            self.deinitIdeaTimelineEvent(event);
        }
        idea.timeline.deinit();
    }

    fn deinitIdeas(self: *PortfolioManager) void {
        for (self.ideas.items) |*idea| {
            self.deinitIdea(idea);
        }
        self.ideas.deinit();
    }

    // Legacy project/program/sub-portfolio cleanup removed in favor
    // of the generic collection/item model.
    }

    /// Add project to portfolio
    pub fn addProject(self: *PortfolioManager, name: []const u8, description: []const u8, category: []const u8) !void {
        if (self.portfolio == null) return;
        _ = try self.addCollection(name, description, .project, category);
    }
    pub fn addProject(self: *PortfolioManager, name: []const u8, description: []const u8, category: []const u8) !void {
        if (self.portfolio == null) return;
        _ = try self.addCollection(name, description, .project, category);
    }

    fn findIdeaMutable(self: *PortfolioManager, idea_id: u32) ?*Idea {
        for (self.ideas.items) |*idea| {
            if (idea.id == idea_id) return idea;
        }
        return null;
    }

    fn ideaHasTag(idea: *const Idea, tag_name: []const u8) bool {
        for (idea.tags.items) |tag| {
            if (std.mem.eql(u8, tag, tag_name)) return true;
        }
        return false;
    }

    fn isLifecycleTransitionAllowed(current: IdeaLifecycle, next: IdeaLifecycle) bool {
        if (current == next) return true;
        return switch (current) {
            .captured => next == .discovered or next == .rejected or next == .archived,
            .discovered => next == .validated or next == .rejected or next == .archived,
            .validated => next == .planned or next == .rejected or next == .archived,
            .planned => next == .in_design or next == .rejected or next == .archived,
            .in_design => next == .in_execution or next == .planned or next == .archived,
            .in_execution => next == .launched or next == .planned or next == .archived,
            .launched => next == .archived,
            .archived => false,
            .rejected => false,
        };
    }

    fn matchesIdeaFilter(idea: *const Idea, filter: IdeaFilter) bool {
        if (filter.lifecycle) |lifecycle| {
            if (idea.lifecycle != lifecycle) return false;
        }
        if (filter.owner_id) |owner_id| {
            if (idea.owner_id != owner_id) return false;
        }
        if (filter.organization) |organization| {
            if (!std.mem.eql(u8, idea.organization, organization)) return false;
        }
        if (filter.category) |category| {
            if (!std.mem.eql(u8, idea.category, category)) return false;
        }
        if (filter.tag_name) |tag_name| {
            if (!ideaHasTag(idea, tag_name)) return false;
        }
        if (filter.query) |query| {
            const in_title = std.mem.indexOf(u8, idea.title, query) != null;
            const in_summary = std.mem.indexOf(u8, idea.summary, query) != null;
            if (!in_title and !in_summary) return false;
        }
        if (filter.active_at) |ts| {
            if (idea.timebox_start) |start| {
                if (ts < start) return false;
            }
            if (idea.timebox_end) |end| {
                if (ts > end) return false;
            }
        }
        if (filter.updated_after) |updated_after| {
            if (idea.updated_at < updated_after) return false;
        }
        if (filter.updated_before) |updated_before| {
            if (idea.updated_at > updated_before) return false;
        }
        return true;
    }

    /// Create a lifecycle-managed idea inside the portfolio.
    pub fn createIdea(
        self: *PortfolioManager,
        title: []const u8,
        summary: []const u8,
        organization: []const u8,
        category: []const u8,
        owner_id: ?u32,
        created_at: i64,
    ) !u32 {
        if (self.portfolio == null) return error.PortfolioNotFound;

        const id = self.next_id;
        self.next_id += 1;

        var idea = Idea{
            .id = id,
            .title = try self.allocator.dupe(u8, title),
            .summary = try self.allocator.dupe(u8, summary),
            .organization = try self.allocator.dupe(u8, organization),
            .category = try self.allocator.dupe(u8, category),
            .lifecycle = .captured,
            .owner_id = owner_id,
            .created_at = created_at,
            .updated_at = created_at,
            .tags = std.array_list.Managed([]const u8).init(self.allocator),
            .versions = std.array_list.Managed(IdeaVersion).init(self.allocator),
            .notes = std.array_list.Managed(IdeaNote).init(self.allocator),
            .designs = std.array_list.Managed(IdeaDesign).init(self.allocator),
            .timeline = std.array_list.Managed(IdeaTimelineEvent).init(self.allocator),
            .tracking = .{ .last_activity_at = created_at },
        };
        {
            errdefer self.deinitIdea(&idea);

            const initial_version = IdeaVersion{
                .id = 0,
                .version_number = 1,
                .title = try self.allocator.dupe(u8, title),
                .summary = try self.allocator.dupe(u8, summary),
                .details = try self.allocator.dupe(u8, summary),
                .created_at = created_at,
            };
            errdefer {
                self.allocator.free(initial_version.title);
                self.allocator.free(initial_version.summary);
                self.allocator.free(initial_version.details);
            }
            try idea.versions.append(initial_version);
            idea.tracking.total_versions = 1;

            try self.ideas.append(idea);
        }

        errdefer {
            var removed = self.ideas.pop();
            self.deinitIdea(&removed);
        }
        try self.addToIndex(id, title, .initiative, self.portfolio.?.id);
        return id;
    }

    /// Transition idea through controlled lifecycle states.
    pub fn transitionIdeaLifecycle(self: *PortfolioManager, idea_id: u32, next: IdeaLifecycle, changed_at: i64) !void {
        const idea = self.findIdeaMutable(idea_id) orelse return error.IdeaNotFound;
        if (!isLifecycleTransitionAllowed(idea.lifecycle, next)) {
            return error.InvalidLifecycleTransition;
        }
        idea.lifecycle = next;
        idea.updated_at = changed_at;
        idea.tracking.last_activity_at = changed_at;
    }

    /// Add a new version snapshot for an idea.
    pub fn addIdeaVersion(
        self: *PortfolioManager,
        idea_id: u32,
        title: []const u8,
        summary: []const u8,
        details: []const u8,
        created_at: i64,
    ) !u32 {
        const idea = self.findIdeaMutable(idea_id) orelse return error.IdeaNotFound;
        const version_id = @as(u32, @intCast(idea.versions.items.len));

        const version = IdeaVersion{
            .id = version_id,
            .version_number = version_id + 1,
            .title = try self.allocator.dupe(u8, title),
            .summary = try self.allocator.dupe(u8, summary),
            .details = try self.allocator.dupe(u8, details),
            .created_at = created_at,
        };
        errdefer {
            self.allocator.free(version.title);
            self.allocator.free(version.summary);
            self.allocator.free(version.details);
        }

        try idea.versions.append(version);
        idea.updated_at = created_at;
        idea.tracking.total_versions = @as(u32, @intCast(idea.versions.items.len));
        idea.tracking.last_activity_at = created_at;
        return version_id;
    }

    pub fn getIdeaVersions(self: *PortfolioManager, idea_id: u32) ![]IdeaVersion {
        const idea = self.findIdeaMutable(idea_id) orelse return error.IdeaNotFound;
        return idea.versions.items;
    }

    /// Add an idea note for capture/save/access workflows.
    pub fn addIdeaNote(self: *PortfolioManager, idea_id: u32, title: []const u8, body: []const u8, created_at: i64) !u32 {
        const idea = self.findIdeaMutable(idea_id) orelse return error.IdeaNotFound;
        const note_id = @as(u32, @intCast(idea.notes.items.len));
        const note = IdeaNote{
            .id = note_id,
            .title = try self.allocator.dupe(u8, title),
            .body = try self.allocator.dupe(u8, body),
            .created_at = created_at,
            .updated_at = created_at,
        };
        errdefer {
            self.allocator.free(note.title);
            self.allocator.free(note.body);
        }

        try idea.notes.append(note);
        idea.updated_at = created_at;
        idea.tracking.total_notes = @as(u32, @intCast(idea.notes.items.len));
        idea.tracking.last_activity_at = created_at;
        return note_id;
    }

    pub fn updateIdeaNote(self: *PortfolioManager, idea_id: u32, note_id: u32, title: []const u8, body: []const u8, updated_at: i64) !void {
        const idea = self.findIdeaMutable(idea_id) orelse return error.IdeaNotFound;
        if (note_id >= @as(u32, @intCast(idea.notes.items.len))) return error.NoteNotFound;

        const note = &idea.notes.items[note_id];
        const new_title = try self.allocator.dupe(u8, title);
        errdefer self.allocator.free(new_title);
        const new_body = try self.allocator.dupe(u8, body);
        errdefer self.allocator.free(new_body);

        self.allocator.free(note.title);
        self.allocator.free(note.body);
        note.title = new_title;
        note.body = new_body;
        note.updated_at = updated_at;
        idea.updated_at = updated_at;
        idea.tracking.last_activity_at = updated_at;
    }

    pub fn getIdeaNotes(self: *PortfolioManager, idea_id: u32) ![]IdeaNote {
        const idea = self.findIdeaMutable(idea_id) orelse return error.IdeaNotFound;
        return idea.notes.items;
    }

    /// Attach design artifacts to idea execution.
    pub fn addIdeaDesign(
        self: *PortfolioManager,
        idea_id: u32,
        name: []const u8,
        description: []const u8,
        artifact_ref: ?[]const u8,
        created_at: i64,
    ) !u32 {
        const idea = self.findIdeaMutable(idea_id) orelse return error.IdeaNotFound;
        const design_id = @as(u32, @intCast(idea.designs.items.len));
        const design = IdeaDesign{
            .id = design_id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .artifact_ref = if (artifact_ref) |artifact| try self.allocator.dupe(u8, artifact) else null,
            .created_at = created_at,
            .updated_at = created_at,
        };
        errdefer {
            self.allocator.free(design.name);
            self.allocator.free(design.description);
            if (design.artifact_ref) |artifact| self.allocator.free(artifact);
        }

        try idea.designs.append(design);
        idea.updated_at = created_at;
        idea.tracking.total_designs = @as(u32, @intCast(idea.designs.items.len));
        idea.tracking.last_activity_at = created_at;
        return design_id;
    }

    pub fn getIdeaDesigns(self: *PortfolioManager, idea_id: u32) ![]IdeaDesign {
        const idea = self.findIdeaMutable(idea_id) orelse return error.IdeaNotFound;
        return idea.designs.items;
    }

    /// Create and manage timeline entries for idea delivery.
    pub fn addIdeaTimelineEvent(
        self: *PortfolioManager,
        idea_id: u32,
        name: []const u8,
        description: []const u8,
        start_at: i64,
        end_at: ?i64,
        status: IdeaTimelineStatus,
        created_at: i64,
    ) !u32 {
        const idea = self.findIdeaMutable(idea_id) orelse return error.IdeaNotFound;
        const event_id = @as(u32, @intCast(idea.timeline.items.len));
        const event = IdeaTimelineEvent{
            .id = event_id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .start_at = start_at,
            .end_at = end_at,
            .status = status,
            .created_at = created_at,
            .updated_at = created_at,
        };
        errdefer {
            self.allocator.free(event.name);
            self.allocator.free(event.description);
        }

        try idea.timeline.append(event);
        idea.updated_at = created_at;
        idea.tracking.total_timeline_events = @as(u32, @intCast(idea.timeline.items.len));
        idea.tracking.last_activity_at = created_at;
        return event_id;
    }

    pub fn setIdeaTimelineStatus(self: *PortfolioManager, idea_id: u32, event_id: u32, status: IdeaTimelineStatus, updated_at: i64) !void {
        const idea = self.findIdeaMutable(idea_id) orelse return error.IdeaNotFound;
        if (event_id >= @as(u32, @intCast(idea.timeline.items.len))) return error.TimelineEventNotFound;

        const event = &idea.timeline.items[event_id];
        event.status = status;
        event.updated_at = updated_at;
        idea.updated_at = updated_at;
        idea.tracking.last_activity_at = updated_at;
    }

    pub fn setIdeaTimebox(self: *PortfolioManager, idea_id: u32, start_at: ?i64, end_at: ?i64, next_review_at: ?i64, updated_at: i64) !void {
        const idea = self.findIdeaMutable(idea_id) orelse return error.IdeaNotFound;
        idea.timebox_start = start_at;
        idea.timebox_end = end_at;
        idea.tracking.next_review_at = next_review_at;
        idea.updated_at = updated_at;
        idea.tracking.last_activity_at = updated_at;
    }

    /// Organize idea ownership by organization/category.
    pub fn organizeIdea(self: *PortfolioManager, idea_id: u32, organization: []const u8, category: []const u8, updated_at: i64) !void {
        const idea = self.findIdeaMutable(idea_id) orelse return error.IdeaNotFound;

        const new_organization = try self.allocator.dupe(u8, organization);
        errdefer self.allocator.free(new_organization);
        const new_category = try self.allocator.dupe(u8, category);
        errdefer self.allocator.free(new_category);

        self.allocator.free(idea.organization);
        self.allocator.free(idea.category);
        idea.organization = new_organization;
        idea.category = new_category;
        idea.updated_at = updated_at;
        idea.tracking.last_activity_at = updated_at;
    }

    /// Tag idea for taxonomy/filter flows.
    pub fn addIdeaTag(self: *PortfolioManager, idea_id: u32, tag_name: []const u8, updated_at: i64) !void {
        const idea = self.findIdeaMutable(idea_id) orelse return error.IdeaNotFound;
        if (ideaHasTag(idea, tag_name)) return;
        try idea.tags.append(try self.allocator.dupe(u8, tag_name));
        idea.updated_at = updated_at;
        idea.tracking.last_activity_at = updated_at;
    }

    pub fn getIdeaById(self: *PortfolioManager, idea_id: u32) ?*Idea {
        return self.findIdeaMutable(idea_id);
    }

    pub fn listIdeas(self: *PortfolioManager) []Idea {
        return self.ideas.items;
    }

    /// Filter ideas by lifecycle, tags, ownership, text, organization, and time windows.
    pub fn filterIdeas(self: *PortfolioManager, filter: IdeaFilter, allocator: std.mem.Allocator) !std.array_list.Managed(IdeaSummary) {
        var results = std.array_list.Managed(IdeaSummary).init(allocator);

        for (self.ideas.items) |*idea| {
            if (!matchesIdeaFilter(idea, filter)) continue;

            const summary = IdeaSummary{
                .id = idea.id,
                .title = try allocator.dupe(u8, idea.title),
                .lifecycle = idea.lifecycle,
                .organization = try allocator.dupe(u8, idea.organization),
                .category = try allocator.dupe(u8, idea.category),
                .owner_id = idea.owner_id,
                .updated_at = idea.updated_at,
                .tag_count = @as(u32, @intCast(idea.tags.items.len)),
                .next_review_at = idea.tracking.next_review_at,
            };
            errdefer {
                allocator.free(summary.title);
                allocator.free(summary.organization);
                allocator.free(summary.category);
            }
            try results.append(summary);
        }

        return results;
    }

    pub fn deinitIdeaSummaries(results: *std.array_list.Managed(IdeaSummary), allocator: std.mem.Allocator) void {
        for (results.items) |summary| {
            allocator.free(summary.title);
            allocator.free(summary.organization);
            allocator.free(summary.category);
        }
        results.deinit();
    }


    
    /// Add tag to entity
    pub fn addTagToEntity(self: *PortfolioManager, entity_id: u32, tag_name: []const u8, tag_category: []const u8) !void {temp.