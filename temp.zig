
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
    pub fn addTagToEntity(self: *PortfolioManager, entity_id: u32, tag_name: []const u8, tag_category: []const u8) !void {
        if (self.portfolio == null) return;
        _ = try self.addTag(entity_id, tag_name, tag_category);
    }

    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    /// Legacy project/program/sub-portfolio cleanup removed in favor
    /// of the generic collection/item model. This section can be removed once the new model is fully implemented and tested.
  
  
  const std = @import("std");

/// Expanded entity types to support diverse portfolio contents
pub const EntityType = enum {
    portfolio,
    program,
    project,
    task,
    initiative,
    deliverable,
    investment,
    entity,
    artifact,
    product,
    solution,
    service,
    work,
    music,
    artwork,
    code,
    asset,
    custom,
};

/// Entity classes for business domain classification
pub const EntityClass = enum {
    strategic,
    operational,
    tactical,
    support,
    research,
    development,
    creative,
    financial,
    technical,
    artistic,
    commercial,
    personal,
};

/// Tag for flexible categorization
pub const Tag = struct {
    id: u32,
    name: []const u8,
    category: []const u8,
};

/// Metadata container for rich entity information
pub const Metadata = struct {
    entity_type: EntityType,
    entity_class: EntityClass,
    category: []const u8,
    tags: std.array_list.Managed(Tag),
    custom_fields: std.StringHashMap([]const u8),
    created_at: i64,
    updated_at: i64,
    owner_id: ?u32 = null,
    parent_id: ?u32 = null,
};

/// Index entry for fast lookups
pub const IndexEntry = struct {
    id: u32,
    name: []const u8,
    entity_type: EntityType,
    parent_id: ?u32 = null,
};

/// Generic item that can represent any type of work/artifact/product
pub const PortfolioItem = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    metadata: Metadata,
};

/// A generic collection container - can hold any items with a specific purpose
pub const PortfolioCollection = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    item_type: EntityType,
    items: std.array_list.Managed(PortfolioItem),
    metadata: Metadata,
};

/// Portfolio is now a generic top-level container that can hold
/// any mix of collections and items
pub const Portfolio = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    collections: std.array_list.Managed(PortfolioCollection),
    items: std.array_list.Managed(PortfolioItem),
    metadata: Metadata,
};

/// Task represents work items (legacy support)
pub const Task = struct {
    id: u32,
    title: []const u8,
    description: []const u8,
    completed: bool,
    priority: u8,
    metadata: Metadata,
};

// Project, Program and SubPortfolio legacy types are intentionally omitted
// in favor of the generic collection/item model defined above.

/// Search filter criteria
pub const SearchFilter = struct {
    name_pattern: ?[]const u8 = null,
    entity_type: ?EntityType = null,
    entity_class: ?EntityClass = null,
    category: ?[]const u8 = null,
    tag_names: ?std.array_list.Managed([]const u8) = null,
    owner_id: ?u32 = null,
};

/// Portfolio Manager for hierarchical portfolio operations
pub const PortfolioManager = struct {
    allocator: std.mem.Allocator,
    portfolio: ?Portfolio = null,
    index: std.array_list.Managed(IndexEntry),
    next_id: u32 = 0,

    pub fn init(allocator: std.mem.Allocator) PortfolioManager {
        return PortfolioManager{
            .allocator = allocator,
            .index = std.array_list.Managed(IndexEntry).init(allocator),
        };
    }

    pub fn deinit(self: *PortfolioManager) void {
        if (self.portfolio) |*portfolio| {
            self.deinitPortfolio(portfolio);
        }
        self.deinitIndex();
    }

    fn deinitIndex(self: *PortfolioManager) void {
        for (self.index.items) |*entry| {
            self.allocator.free(entry.name);
        }
        self.index.deinit();
    }

    fn deinitMetadata(self: *PortfolioManager, metadata: *Metadata) void {
        self.allocator.free(metadata.category);
        for (metadata.tags.items) |*tag| {
            self.allocator.free(tag.name);
            self.allocator.free(tag.category);
        }
        metadata.tags.deinit();
        var iter = metadata.custom_fields.iterator();
        while (iter.next()) |entry| {
            self.allocator.free(entry.key_ptr.*);
            self.allocator.free(entry.value_ptr.*);
        }
        metadata.custom_fields.deinit();
    }

    fn deinitPortfolioItem(self: *PortfolioManager, item: *PortfolioItem) void {
        self.allocator.free(item.name);
        self.allocator.free(item.description);
        self.deinitMetadata(&item.metadata);
    }

    fn deinitCollection(self: *PortfolioManager, collection: *PortfolioCollection) void {
        for (collection.items.items) |*item| {
            self.deinitPortfolioItem(item);
        }
        collection.items.deinit();
        self.allocator.free(collection.name);
        self.allocator.free(collection.description);
        self.deinitMetadata(&collection.metadata);
    }

    fn deinitTask(self: *PortfolioManager, task: *Task) void {
        self.allocator.free(task.title);
        self.allocator.free(task.description);
        self.deinitMetadata(&task.metadata);
    }

    // Legacy project/program/sub-portfolio cleanup removed in favor
    // of the generic collection/item model.

    fn deinitPortfolio(self: *PortfolioManager, portfolio: *Portfolio) void {
        for (portfolio.collections.items) |*col| {
            self.deinitCollection(col);
        }
        portfolio.collections.deinit();
        for (portfolio.items.items) |*item| {
            self.deinitPortfolioItem(item);
        }
        portfolio.items.deinit();
        self.allocator.free(portfolio.name);
        self.allocator.free(portfolio.description);
        self.deinitMetadata(&portfolio.metadata);
    }

    /// Create a new portfolio
    pub fn createPortfolio(self: *PortfolioManager, name: []const u8, description: []const u8, category: []const u8) !void {
        const id = self.next_id;
        self.next_id += 1;

        const metadata = Metadata{
            .entity_type = .portfolio,
            .entity_class = .strategic,
            .category = try self.allocator.dupe(u8, category),
            .tags = std.array_list.Managed(Tag).init(self.allocator),
            .custom_fields = std.StringHashMap([]const u8).init(self.allocator),
            .created_at = 0,
            .updated_at = 0,
        };

        const portfolio = Portfolio{
            .id = id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .collections = std.array_list.Managed(PortfolioCollection).init(self.allocator),
            .items = std.array_list.Managed(PortfolioItem).init(self.allocator),
            .metadata = metadata,
        };

        self.portfolio = portfolio;
        try self.addToIndex(id, name, .portfolio, null);
    }

    /// Add a generic collection to the portfolio (e.g., music collection, artwork, products, etc)
    pub fn addCollection(
        self: *PortfolioManager,
        name: []const u8,
        description: []const u8,
        item_type: EntityType,
        category: []const u8,
    ) !u32 {
        if (self.portfolio == null) return error.PortfolioNotFound;

        const id = self.next_id;
        self.next_id += 1;

        const metadata = Metadata{
            .entity_type = item_type,
            .entity_class = .personal,
            .category = try self.allocator.dupe(u8, category),
            .tags = std.array_list.Managed(Tag).init(self.allocator),
            .custom_fields = std.StringHashMap([]const u8).init(self.allocator),
            .created_at = 0,
            .updated_at = 0,
            .parent_id = self.portfolio.?.id,
        };

        const collection = PortfolioCollection{
            .id = id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .item_type = item_type,
            .items = std.array_list.Managed(PortfolioItem).init(self.allocator),
            .metadata = metadata,
        };

        try self.portfolio.?.collections.append(collection);
        try self.addToIndex(id, name, item_type, self.portfolio.?.id);
        return id;
    }

    /// Add an item to a collection
    pub fn addItemToCollection(
        self: *PortfolioManager,
        collection_id: u32,
        name: []const u8,
        description: []const u8,
    ) !u32 {
        if (self.portfolio == null) return error.PortfolioNotFound;

        for (self.portfolio.?.collections.items) |*collection| {
            if (collection.id == collection_id) {
                const id = self.next_id;
                self.next_id += 1;

                const metadata = Metadata{
                    .entity_type = collection.item_type,
                    .entity_class = .personal,
                    .category = try self.allocator.dupe(u8, collection.metadata.category),
                    .tags = std.array_list.Managed(Tag).init(self.allocator),
                    .custom_fields = std.StringHashMap([]const u8).init(self.allocator),
                    .created_at = 0,
                    .updated_at = 0,
                    .parent_id = collection_id,
                };

                const item = PortfolioItem{
                    .id = id,
                    .name = try self.allocator.dupe(u8, name),
                    .description = try self.allocator.dupe(u8, description),
                    .metadata = metadata,
                };

                try collection.items.append(item);
                try self.addToIndex(id, name, collection.item_type, collection_id);
                return id;
            }
        }
        return error.CollectionNotFound;
    }

    /// Add a top-level item to the portfolio
    pub fn addItem(
        self: *PortfolioManager,
        name: []const u8,
        description: []const u8,
        item_type: EntityType,
        category: []const u8,
    ) !u32 {
        if (self.portfolio == null) return error.PortfolioNotFound;

        const id = self.next_id;
        self.next_id += 1;

        const metadata = Metadata{
            .entity_type = item_type,
            .entity_class = .personal,
            .category = try self.allocator.dupe(u8, category),
            .tags = std.array_list.Managed(Tag).init(self.allocator),
            .custom_fields = std.StringHashMap([]const u8).init(self.allocator),
            .created_at = 0,
            .updated_at = 0,
            .parent_id = self.portfolio.?.id,
        };

        const item = PortfolioItem{
            .id = id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .metadata = metadata,
        };

        try self.portfolio.?.items.append(item);
        try self.addToIndex(id, name, item_type, self.portfolio.?.id);
        return id;
    }

    /// Generic getCollectionCount
    pub fn getCollectionCount(self: *PortfolioManager) u32 {
        if (self.portfolio) |p| {
            return @as(u32, @intCast(p.collections.items.len));
        }
        return 0;
    }

    /// Generic getItemCount for portfolio or collection
    pub fn getPortfolioItemCount(self: *PortfolioManager) u32 {
        if (self.portfolio) |p| {
            return @as(u32, @intCast(p.items.items.len));
        }
        return 0;
    }

    /// Get collection by ID
    pub fn getCollectionById(self: *PortfolioManager, collection_id: u32) ?*PortfolioCollection {
        if (self.portfolio) |*p| {
            for (p.collections.items) |*col| {
                if (col.id == collection_id) return col;
            }
        }
        return null;
    }

    /// Get collections by item type
    pub fn getCollectionsByType(self: *PortfolioManager, item_type: EntityType) !std.array_list.Managed(*PortfolioCollection) {
        var results = std.array_list.Managed(*PortfolioCollection).init(self.allocator);
        if (self.portfolio) |*p| {
            for (p.collections.items) |*col| {
                if (col.item_type == item_type) {
                    try results.append(col);
                }
            }
        }
        return results;
    }

    /// Add sub-portfolio to main portfolio (legacy support - uses generic addItem)
    pub fn addSubPortfolio(self: *PortfolioManager, name: []const u8, description: []const u8, category: []const u8) !void {
        if (self.portfolio == null) return;

        // Store as a collection for now via generic addItem
        _ = try self.addItem(name, description, .portfolio, category);
    }

    /// Add program to portfolio
    pub fn addProgram(self: *PortfolioManager, name: []const u8, description: []const u8, category: []const u8) !void {
        if (self.portfolio == null) return;
        _ = try self.addCollection(name, description, .program, category);
    }

    /// Add project to portfolio
    pub fn addProject(self: *PortfolioManager, name: []const u8, description: []const u8, category: []const u8) !void {
        if (self.portfolio == null) return;
        _ = try self.addCollection(name, description, .project, category);
    }

    /// Add tag to entity
    pub fn addTagToEntity(self: *PortfolioManager, entity_id: u32, tag_name: []const u8, tag_category: []const u8) !void {
        if (self.portfolio == null) return;

        // Search through all entities
        if (self.findEntityMetadata(entity_id)) |metadata| {
            const tag_id = @as(u32, @intCast(metadata.tags.items.len));
            const tag = Tag{
                .id = tag_id,
                .name = try self.allocator.dupe(u8, tag_name),
                .category = try self.allocator.dupe(u8, tag_category),
            };
            try metadata.tags.append(tag);
        }
    }

    /// Find metadata for entity by ID
    fn findEntityMetadata(self: *PortfolioManager, entity_id: u32) ?*Metadata {
        if (self.portfolio) |*portfolio| {
            if (portfolio.id == entity_id) return &portfolio.metadata;

            // Search collections and their items
            for (portfolio.collections.items) |*collection| {
                if (collection.id == entity_id) return &collection.metadata;
                for (collection.items.items) |*item| {
                    if (item.id == entity_id) return &item.metadata;
                }
            }

            // Search top-level items
            for (portfolio.items.items) |*item| {
                if (item.id == entity_id) return &item.metadata;
            }
        }

        return null;
    }

    /// Add index entry
    fn addToIndex(self: *PortfolioManager, id: u32, name: []const u8, entity_type: EntityType, parent_id: ?u32) !void {
        const entry = IndexEntry{
            .id = id,
            .name = try self.allocator.dupe(u8, name),
            .entity_type = entity_type,
            .parent_id = parent_id,
        };
        try self.index.append(entry);
    }

    /// Search entities by filter
    pub fn search(self: *PortfolioManager, filter: SearchFilter, allocator: std.mem.Allocator) !std.array_list.Managed(IndexEntry) {
        var results = std.array_list.Managed(IndexEntry).init(allocator);

        for (self.index.items) |entry| {
            var matches = true;

            if (filter.name_pattern) |pattern| {
                if (std.mem.indexOf(u8, entry.name, pattern) == null) {
                    matches = false;
                }
            }

            if (filter.entity_type) |etype| {
                if (entry.entity_type != etype) {
                    matches = false;
                }
            }

            if (matches) {
                const entry_copy = IndexEntry{
                    .id = entry.id,
                    .name = try allocator.dupe(u8, entry.name),
                    .entity_type = entry.entity_type,
                    .parent_id = entry.parent_id,
                };
                try results.append(entry_copy);
            }
        }

        return results;
    }

    /// Get entity by ID from index
    pub fn getEntityById(self: *PortfolioManager, id: u32) ?IndexEntry {
        for (self.index.items) |entry| {
            if (entry.id == id) {
                return entry;
            }
        }
        return null;
    }

    /// Get all entities of a specific type
    pub fn getEntitiesByType(self: *PortfolioManager, entity_type: EntityType, allocator: std.mem.Allocator) !std.array_list.Managed(IndexEntry) {
        var results = std.array_list.Managed(IndexEntry).init(allocator);
        for (self.index.items) |entry| {
            if (entry.entity_type == entity_type) {
                const entry_copy = IndexEntry{
                    .id = entry.id,
                    .name = try allocator.dupe(u8, entry.name),
                    .entity_type = entry.entity_type,
                    .parent_id = entry.parent_id,
                };
                try results.append(entry_copy);
            }
        }
        return results;
    }
};

pub fn portfolioDemo() void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var manager = PortfolioManager.init(allocator);
    defer manager.deinit();

    std.debug.print("Portfolio Manager initialized\n", .{});
}




    const std = @import("std");

pub const ProjectStatus = enum {
    proposed,
    planned,
    active,
    blocked,
    completed,
    canceled,
    archived,
};

pub const ProjectPriority = enum {
    low,
    medium,
    high,
    critical,
};

pub const MilestoneStatus = enum {
    pending,
    in_progress,
    completed,
    delayed,
    canceled,
};

pub const TaskStatus = enum {
    todo,
    in_progress,
    blocked,
    done,
    canceled,
};

pub const IssueSeverity = enum {
    low,
    medium,
    high,
    critical,
};

pub const ProgressUpdateType = enum {
    general,
    scope,
    timeline,
    budget,
    risk,
    decision,
};

pub const StoryType = enum {
    feature,
    bug,
    capability,
    enabler,
    blocker,
    @"test",
    requirement,
    defect,
    issue,
    review,
    audit,
    report,
    performance,
    strategy,
    tactic,
    operation,
    business_case,
    reqruiement,
    research,
    prototype,
    spike,
    idea,
    concept,
    design,
    documentation,
    milestone,
    objective,
    outcome,
    risk,
    mission,
    vision,
    goal,
};

pub const StoryStatus = enum {
    backlog,
    ready,
    in_progress,
    blocked,
    in_review,
    done,
    canceled,
};

pub const ProjectMilestone = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    due_at: ?i64,
    status: MilestoneStatus,
    created_at: i64,
    updated_at: i64,
};

pub const ProjectTask = struct {
    id: u32,
    title: []const u8,
    description: []const u8,
    assignee_id: ?u32,
    due_at: ?i64,
    status: TaskStatus,
    created_at: i64,
    updated_at: i64,
};

pub const ProjectIssue = struct {
    id: u32,
    title: []const u8,
    description: []const u8,
    severity: IssueSeverity,
    is_resolved: bool,
    created_at: i64,
    resolved_at: ?i64 = null,
};

pub const ProjectNote = struct {
    id: u32,
    title: []const u8,
    body: []const u8,
    created_at: i64,
    updated_at: i64,
};

pub const ProjectProgressUpdate = struct {
    id: u32,
    update_type: ProgressUpdateType,
    message: []const u8,
    created_at: i64,
    author_id: ?u32 = null,
};

pub const Story = struct {
    id: u32,
    title: []const u8,
    description: []const u8,
    story_type: StoryType,
    status: StoryStatus,
    priority: ProjectPriority,
    owner_id: ?u32,
    estimate_points: f32,
    created_at: i64,
    updated_at: i64,
};

pub const WbsTask = struct {
    id: u32,
    title: []const u8,
    description: []const u8,
    assignee_id: ?u32,
    due_at: ?i64,
    status: TaskStatus,
    created_at: i64,
    updated_at: i64,
};

pub const WbsStory = struct {
    id: u32,
    title: []const u8,
    description: []const u8,
    story_type: StoryType,
    status: StoryStatus,
    priority: ProjectPriority,
    owner_id: ?u32,
    estimate_points: f32,
    created_at: i64,
    updated_at: i64,
    tasks: std.array_list.Managed(WbsTask),
};

pub const WbsEpic = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    status: ProjectStatus,
    priority: ProjectPriority,
    owner_id: ?u32,
    created_at: i64,
    updated_at: i64,
    stories: std.array_list.Managed(WbsStory),
};

pub const WbsInitiative = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    status: ProjectStatus,
    priority: ProjectPriority,
    owner_id: ?u32,
    created_at: i64,
    updated_at: i64,
    epics: std.array_list.Managed(WbsEpic),
};

pub const WbsTheme = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    status: ProjectStatus,
    created_at: i64,
    updated_at: i64,
    initiatives: std.array_list.Managed(WbsInitiative),
};

pub const WorkPackage = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    status: ProjectStatus,
    created_at: i64,
    updated_at: i64,
    themes: std.array_list.Managed(WbsTheme),
};

pub const WorkBreakdownStructure = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    status: ProjectStatus,
    created_at: i64,
    updated_at: i64,
    work_packages: std.array_list.Managed(WorkPackage),
};

pub const TrackedProject = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    owner_id: ?u32,
    status: ProjectStatus,
    priority: ProjectPriority,
    created_at: i64,
    updated_at: i64,
    start_at: ?i64,
    target_end_at: ?i64,
    actual_end_at: ?i64 = null,
    tags: std.array_list.Managed([]const u8),
    milestones: std.array_list.Managed(ProjectMilestone),
    tasks: std.array_list.Managed(ProjectTask),
    issues: std.array_list.Managed(ProjectIssue),
    stories: std.array_list.Managed(Story),
    wbs_structures: std.array_list.Managed(WorkBreakdownStructure),
    notes: std.array_list.Managed(ProjectNote),
    updates: std.array_list.Managed(ProjectProgressUpdate),
};

pub const ProjectFilter = struct {
    status: ?ProjectStatus = null,
    priority: ?ProjectPriority = null,
    owner_id: ?u32 = null,
    tag_name: ?[]const u8 = null,
    name_pattern: ?[]const u8 = null,
    active_on: ?i64 = null,
    include_archived: bool = false,
};

pub const ProjectSummary = struct {
    id: u32,
    name: []const u8,
    status: ProjectStatus,
    priority: ProjectPriority,
    owner_id: ?u32,
    progress_percent: f32,
    open_issues: u32,
    target_end_at: ?i64,
    updated_at: i64,
};

pub const StoryFilter = struct {
    story_type: ?StoryType = null,
    status: ?StoryStatus = null,
    priority: ?ProjectPriority = null,
    owner_id: ?u32 = null,
    query: ?[]const u8 = null,
    include_done: bool = true,
};

pub const StorySummary = struct {
    id: u32,
    title: []const u8,
    story_type: StoryType,
    status: StoryStatus,
    priority: ProjectPriority,
    owner_id: ?u32,
    updated_at: i64,
};

pub const ProjectTrackingManager = struct {
    allocator: std.mem.Allocator,
    projects: std.array_list.Managed(TrackedProject),
    next_project_id: u32 = 1,

    pub fn init(allocator: std.mem.Allocator) ProjectTrackingManager {
        return .{
            .allocator = allocator,
            .projects = std.array_list.Managed(TrackedProject).init(allocator),
        };
    }

    pub fn deinit(self: *ProjectTrackingManager) void {
        for (self.projects.items) |*project| {
            self.deinitProject(project);
        }
        self.projects.deinit();
    }

    fn deinitWbsTask(self: *ProjectTrackingManager, task: *WbsTask) void {
        self.allocator.free(task.title);
        self.allocator.free(task.description);
    }

    fn deinitWbsStory(self: *ProjectTrackingManager, story: *WbsStory) void {
        self.allocator.free(story.title);
        self.allocator.free(story.description);

        for (story.tasks.items) |*task| {
            self.deinitWbsTask(task);
        }
        story.tasks.deinit();
    }

    fn deinitWbsEpic(self: *ProjectTrackingManager, epic: *WbsEpic) void {
        self.allocator.free(epic.name);
        self.allocator.free(epic.description);

        for (epic.stories.items) |*story| {
            self.deinitWbsStory(story);
        }
        epic.stories.deinit();
    }

    fn deinitWbsInitiative(self: *ProjectTrackingManager, initiative: *WbsInitiative) void {
        self.allocator.free(initiative.name);
        self.allocator.free(initiative.description);

        for (initiative.epics.items) |*epic| {
            self.deinitWbsEpic(epic);
        }
        initiative.epics.deinit();
    }

    fn deinitWbsTheme(self: *ProjectTrackingManager, theme: *WbsTheme) void {
        self.allocator.free(theme.name);
        self.allocator.free(theme.description);

        for (theme.initiatives.items) |*initiative| {
            self.deinitWbsInitiative(initiative);
        }
        theme.initiatives.deinit();
    }

    fn deinitWorkPackage(self: *ProjectTrackingManager, work_package: *WorkPackage) void {
        self.allocator.free(work_package.name);
        self.allocator.free(work_package.description);

        for (work_package.themes.items) |*theme| {
            self.deinitWbsTheme(theme);
        }
        work_package.themes.deinit();
    }

    fn deinitWorkBreakdownStructure(self: *ProjectTrackingManager, wbs: *WorkBreakdownStructure) void {
        self.allocator.free(wbs.name);
        self.allocator.free(wbs.description);

        for (wbs.work_packages.items) |*work_package| {
            self.deinitWorkPackage(work_package);
        }
        wbs.work_packages.deinit();
    }

    fn deinitProject(self: *ProjectTrackingManager, project: *TrackedProject) void {
        self.allocator.free(project.name);
        self.allocator.free(project.description);

        for (project.tags.items) |tag| {
            self.allocator.free(tag);
        }
        project.tags.deinit();

        for (project.milestones.items) |*milestone| {
            self.allocator.free(milestone.name);
            self.allocator.free(milestone.description);
        }
        project.milestones.deinit();

        for (project.tasks.items) |*task| {
            self.allocator.free(task.title);
            self.allocator.free(task.description);
        }
        project.tasks.deinit();

        for (project.issues.items) |*issue| {
            self.allocator.free(issue.title);
            self.allocator.free(issue.description);
        }
        project.issues.deinit();

        for (project.stories.items) |*story| {
            self.allocator.free(story.title);
            self.allocator.free(story.description);
        }
        project.stories.deinit();

        for (project.wbs_structures.items) |*wbs| {
            self.deinitWorkBreakdownStructure(wbs);
        }
        project.wbs_structures.deinit();

        for (project.notes.items) |*note| {
            self.allocator.free(note.title);
            self.allocator.free(note.body);
        }
        project.notes.deinit();

        for (project.updates.items) |*update| {
            self.allocator.free(update.message);
        }
        project.updates.deinit();
    }

    fn getProjectMutable(self: *ProjectTrackingManager, project_id: u32) ?*TrackedProject {
        for (self.projects.items) |*project| {
            if (project.id == project_id) return project;
        }
        return null;
    }

    fn getWbsMutable(project: *TrackedProject, wbs_id: u32) ?*WorkBreakdownStructure {
        if (wbs_id >= @as(u32, @intCast(project.wbs_structures.items.len))) return null;
        return &project.wbs_structures.items[wbs_id];
    }

    fn getWorkPackageMutable(wbs: *WorkBreakdownStructure, work_package_id: u32) ?*WorkPackage {
        if (work_package_id >= @as(u32, @intCast(wbs.work_packages.items.len))) return null;
        return &wbs.work_packages.items[work_package_id];
    }

    fn getThemeMutable(work_package: *WorkPackage, theme_id: u32) ?*WbsTheme {
        if (theme_id >= @as(u32, @intCast(work_package.themes.items.len))) return null;
        return &work_package.themes.items[theme_id];
    }

    fn getInitiativeMutable(theme: *WbsTheme, initiative_id: u32) ?*WbsInitiative {
        if (initiative_id >= @as(u32, @intCast(theme.initiatives.items.len))) return null;
        return &theme.initiatives.items[initiative_id];
    }

    fn getEpicMutable(initiative: *WbsInitiative, epic_id: u32) ?*WbsEpic {
        if (epic_id >= @as(u32, @intCast(initiative.epics.items.len))) return null;
        return &initiative.epics.items[epic_id];
    }

    fn getWbsStoryMutable(epic: *WbsEpic, story_id: u32) ?*WbsStory {
        if (story_id >= @as(u32, @intCast(epic.stories.items.len))) return null;
        return &epic.stories.items[story_id];
    }

    fn projectHasTag(project: *const TrackedProject, tag_name: []const u8) bool {
        for (project.tags.items) |tag| {
            if (std.mem.eql(u8, tag, tag_name)) return true;
        }
        return false;
    }

    fn openIssueCount(project: *const TrackedProject) u32 {
        var count: u32 = 0;
        for (project.issues.items) |issue| {
            if (!issue.is_resolved) count += 1;
        }
        return count;
    }

    fn matchesFilter(project: *const TrackedProject, filter: ProjectFilter) bool {
        if (!filter.include_archived and project.status == .archived) return false;

        if (filter.status) |status| {
            if (project.status != status) return false;
        }
        if (filter.priority) |priority| {
            if (project.priority != priority) return false;
        }
        if (filter.owner_id) |owner_id| {
            if (project.owner_id != owner_id) return false;
        }
        if (filter.tag_name) |tag_name| {
            if (!projectHasTag(project, tag_name)) return false;
        }
        if (filter.name_pattern) |name_pattern| {
            if (std.mem.indexOf(u8, project.name, name_pattern) == null) return false;
        }
        if (filter.active_on) |ts| {
            if (project.start_at) |start| {
                if (ts < start) return false;
            }
            const end_bound = project.actual_end_at orelse project.target_end_at;
            if (end_bound) |end| {
                if (ts > end) return false;
            }
        }
        return true;
    }

    fn matchesStoryFilter(story: *const Story, filter: StoryFilter) bool {
        if (!filter.include_done and story.status == .done) return false;

        if (filter.story_type) |story_type| {
            if (story.story_type != story_type) return false;
        }
        if (filter.status) |status| {
            if (story.status != status) return false;
        }
        if (filter.priority) |priority| {
            if (story.priority != priority) return false;
        }
        if (filter.owner_id) |owner_id| {
            if (story.owner_id != owner_id) return false;
        }
        if (filter.query) |query| {
            const in_title = std.mem.indexOf(u8, story.title, query) != null;
            const in_description = std.mem.indexOf(u8, story.description, query) != null;
            if (!in_title and !in_description) return false;
        }

        return true;
    }

    pub fn createProject(
        self: *ProjectTrackingManager,
        name: []const u8,
        description: []const u8,
        owner_id: ?u32,
        priority: ProjectPriority,
        start_at: ?i64,
        target_end_at: ?i64,
        created_at: i64,
    ) !u32 {
        const id = self.next_project_id;
        self.next_project_id += 1;

        var project = TrackedProject{
            .id = id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .owner_id = owner_id,
            .status = .proposed,
            .priority = priority,
            .created_at = created_at,
            .updated_at = created_at,
            .start_at = start_at,
            .target_end_at = target_end_at,
            .tags = std.array_list.Managed([]const u8).init(self.allocator),
            .milestones = std.array_list.Managed(ProjectMilestone).init(self.allocator),
            .tasks = std.array_list.Managed(ProjectTask).init(self.allocator),
            .issues = std.array_list.Managed(ProjectIssue).init(self.allocator),
            .stories = std.array_list.Managed(Story).init(self.allocator),
            .wbs_structures = std.array_list.Managed(WorkBreakdownStructure).init(self.allocator),
            .notes = std.array_list.Managed(ProjectNote).init(self.allocator),
            .updates = std.array_list.Managed(ProjectProgressUpdate).init(self.allocator),
        };

        errdefer self.deinitProject(&project);
        try self.projects.append(project);
        return id;
    }

    pub fn updateProjectStatus(self: *ProjectTrackingManager, project_id: u32, status: ProjectStatus, updated_at: i64) !void {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        project.status = status;
        project.updated_at = updated_at;
        if (status == .completed or status == .canceled or status == .archived) {
            project.actual_end_at = updated_at;
        }
    }

    pub fn setProjectSchedule(self: *ProjectTrackingManager, project_id: u32, start_at: ?i64, target_end_at: ?i64, updated_at: i64) !void {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        project.start_at = start_at;
        project.target_end_at = target_end_at;
        project.updated_at = updated_at;
    }

    pub fn addProjectTag(self: *ProjectTrackingManager, project_id: u32, tag_name: []const u8, updated_at: i64) !void {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        if (projectHasTag(project, tag_name)) return;

        try project.tags.append(try self.allocator.dupe(u8, tag_name));
        project.updated_at = updated_at;
    }

    pub fn addWorkBreakdownStructure(
        self: *ProjectTrackingManager,
        project_id: u32,
        name: []const u8,
        description: []const u8,
        created_at: i64,
    ) !u32 {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        const wbs_id = @as(u32, @intCast(project.wbs_structures.items.len));

        const wbs = WorkBreakdownStructure{
            .id = wbs_id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .status = .planned,
            .created_at = created_at,
            .updated_at = created_at,
            .work_packages = std.array_list.Managed(WorkPackage).init(self.allocator),
        };
        errdefer {
            self.allocator.free(wbs.name);
            self.allocator.free(wbs.description);
        }

        try project.wbs_structures.append(wbs);
        project.updated_at = created_at;
        return wbs_id;
    }

    pub fn addWorkPackage(
        self: *ProjectTrackingManager,
        project_id: u32,
        wbs_id: u32,
        name: []const u8,
        description: []const u8,
        created_at: i64,
    ) !u32 {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        const wbs = getWbsMutable(project, wbs_id) orelse return error.WbsNotFound;
        const work_package_id = @as(u32, @intCast(wbs.work_packages.items.len));

        const work_package = WorkPackage{
            .id = work_package_id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .status = .planned,
            .created_at = created_at,
            .updated_at = created_at,
            .themes = std.array_list.Managed(WbsTheme).init(self.allocator),
        };
        errdefer {
            self.allocator.free(work_package.name);
            self.allocator.free(work_package.description);
        }

        try wbs.work_packages.append(work_package);
        wbs.updated_at = created_at;
        project.updated_at = created_at;
        return work_package_id;
    }

    pub fn addTheme(
        self: *ProjectTrackingManager,
        project_id: u32,
        wbs_id: u32,
        work_package_id: u32,
        name: []const u8,
        description: []const u8,
        created_at: i64,
    ) !u32 {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        const wbs = getWbsMutable(project, wbs_id) orelse return error.WbsNotFound;
        const work_package = getWorkPackageMutable(wbs, work_package_id) orelse return error.WorkPackageNotFound;
        const theme_id = @as(u32, @intCast(work_package.themes.items.len));

        const theme = WbsTheme{
            .id = theme_id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .status = .planned,
            .created_at = created_at,
            .updated_at = created_at,
            .initiatives = std.array_list.Managed(WbsInitiative).init(self.allocator),
        };
        errdefer {
            self.allocator.free(theme.name);
            self.allocator.free(theme.description);
        }

        try work_package.themes.append(theme);
        work_package.updated_at = created_at;
        wbs.updated_at = created_at;
        project.updated_at = created_at;
        return theme_id;
    }

    pub fn addInitiative(
        self: *ProjectTrackingManager,
        project_id: u32,
        wbs_id: u32,
        work_package_id: u32,
        theme_id: u32,
        name: []const u8,
        description: []const u8,
        priority: ProjectPriority,
        owner_id: ?u32,
        created_at: i64,
    ) !u32 {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        const wbs = getWbsMutable(project, wbs_id) orelse return error.WbsNotFound;
        const work_package = getWorkPackageMutable(wbs, work_package_id) orelse return error.WorkPackageNotFound;
        const theme = getThemeMutable(work_package, theme_id) orelse return error.ThemeNotFound;
        const initiative_id = @as(u32, @intCast(theme.initiatives.items.len));

        const initiative = WbsInitiative{
            .id = initiative_id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .status = .planned,
            .priority = priority,
            .owner_id = owner_id,
            .created_at = created_at,
            .updated_at = created_at,
            .epics = std.array_list.Managed(WbsEpic).init(self.allocator),
        };
        errdefer {
            self.allocator.free(initiative.name);
            self.allocator.free(initiative.description);
        }

        try theme.initiatives.append(initiative);
        theme.updated_at = created_at;
        work_package.updated_at = created_at;
        wbs.updated_at = created_at;
        project.updated_at = created_at;
        return initiative_id;
    }

    pub fn addEpic(
        self: *ProjectTrackingManager,
        project_id: u32,
        wbs_id: u32,
        work_package_id: u32,
        theme_id: u32,
        initiative_id: u32,
        name: []const u8,
        description: []const u8,
        priority: ProjectPriority,
        owner_id: ?u32,
        created_at: i64,
    ) !u32 {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        const wbs = getWbsMutable(project, wbs_id) orelse return error.WbsNotFound;
        const work_package = getWorkPackageMutable(wbs, work_package_id) orelse return error.WorkPackageNotFound;
        const theme = getThemeMutable(work_package, theme_id) orelse return error.ThemeNotFound;
        const initiative = getInitiativeMutable(theme, initiative_id) orelse return error.InitiativeNotFound;
        const epic_id = @as(u32, @intCast(initiative.epics.items.len));

        const epic = WbsEpic{
            .id = epic_id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .status = .planned,
            .priority = priority,
            .owner_id = owner_id,
            .created_at = created_at,
            .updated_at = created_at,
            .stories = std.array_list.Managed(WbsStory).init(self.allocator),
        };
        errdefer {
            self.allocator.free(epic.name);
            self.allocator.free(epic.description);
        }

        try initiative.epics.append(epic);
        initiative.updated_at = created_at;
        theme.updated_at = created_at;
        work_package.updated_at = created_at;
        wbs.updated_at = created_at;
        project.updated_at = created_at;
        return epic_id;
    }

    pub fn addWbsStory(
        self: *ProjectTrackingManager,
        project_id: u32,
        wbs_id: u32,
        work_package_id: u32,
        theme_id: u32,
        initiative_id: u32,
        epic_id: u32,
        title: []const u8,
        description: []const u8,
        story_type: StoryType,
        priority: ProjectPriority,
        owner_id: ?u32,
        estimate_points: f32,
        created_at: i64,
    ) !u32 {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        const wbs = getWbsMutable(project, wbs_id) orelse return error.WbsNotFound;
        const work_package = getWorkPackageMutable(wbs, work_package_id) orelse return error.WorkPackageNotFound;
        const theme = getThemeMutable(work_package, theme_id) orelse return error.ThemeNotFound;
        const initiative = getInitiativeMutable(theme, initiative_id) orelse return error.InitiativeNotFound;
        const epic = getEpicMutable(initiative, epic_id) orelse return error.EpicNotFound;
        const story_id = @as(u32, @intCast(epic.stories.items.len));

        const story = WbsStory{
            .id = story_id,
            .title = try self.allocator.dupe(u8, title),
            .description = try self.allocator.dupe(u8, description),
            .story_type = story_type,
            .status = .backlog,
            .priority = priority,
            .owner_id = owner_id,
            .estimate_points = estimate_points,
            .created_at = created_at,
            .updated_at = created_at,
            .tasks = std.array_list.Managed(WbsTask).init(self.allocator),
        };
        errdefer {
            self.allocator.free(story.title);
            self.allocator.free(story.description);
        }

        try epic.stories.append(story);
        epic.updated_at = created_at;
        initiative.updated_at = created_at;
        theme.updated_at = created_at;
        work_package.updated_at = created_at;
        wbs.updated_at = created_at;
        project.updated_at = created_at;
        return story_id;
    }

    pub fn addWbsTask(
        self: *ProjectTrackingManager,
        project_id: u32,
        wbs_id: u32,
        work_package_id: u32,
        theme_id: u32,
        initiative_id: u32,
        epic_id: u32,
        story_id: u32,
        title: []const u8,
        description: []const u8,
        assignee_id: ?u32,
        due_at: ?i64,
        created_at: i64,
    ) !u32 {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        const wbs = getWbsMutable(project, wbs_id) orelse return error.WbsNotFound;
        const work_package = getWorkPackageMutable(wbs, work_package_id) orelse return error.WorkPackageNotFound;
        const theme = getThemeMutable(work_package, theme_id) orelse return error.ThemeNotFound;
        const initiative = getInitiativeMutable(theme, initiative_id) orelse return error.InitiativeNotFound;
        const epic = getEpicMutable(initiative, epic_id) orelse return error.EpicNotFound;
        const story = getWbsStoryMutable(epic, story_id) orelse return error.WbsStoryNotFound;
        const task_id = @as(u32, @intCast(story.tasks.items.len));

        const task = WbsTask{
            .id = task_id,
            .title = try self.allocator.dupe(u8, title),
            .description = try self.allocator.dupe(u8, description),
            .assignee_id = assignee_id,
            .due_at = due_at,
            .status = .todo,
            .created_at = created_at,
            .updated_at = created_at,
        };
        errdefer {
            self.allocator.free(task.title);
            self.allocator.free(task.description);
        }

        try story.tasks.append(task);
        story.updated_at = created_at;
        epic.updated_at = created_at;
        initiative.updated_at = created_at;
        theme.updated_at = created_at;
        work_package.updated_at = created_at;
        wbs.updated_at = created_at;
        project.updated_at = created_at;
        return task_id;
    }

    pub fn setWbsStoryStatus(
        self: *ProjectTrackingManager,
        project_id: u32,
        wbs_id: u32,
        work_package_id: u32,
        theme_id: u32,
        initiative_id: u32,
        epic_id: u32,
        story_id: u32,
        status: StoryStatus,
        updated_at: i64,
    ) !void {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        const wbs = getWbsMutable(project, wbs_id) orelse return error.WbsNotFound;
        const work_package = getWorkPackageMutable(wbs, work_package_id) orelse return error.WorkPackageNotFound;
        const theme = getThemeMutable(work_package, theme_id) orelse return error.ThemeNotFound;
        const initiative = getInitiativeMutable(theme, initiative_id) orelse return error.InitiativeNotFound;
        const epic = getEpicMutable(initiative, epic_id) orelse return error.EpicNotFound;
        const story = getWbsStoryMutable(epic, story_id) orelse return error.WbsStoryNotFound;

        story.status = status;
        story.updated_at = updated_at;
        epic.updated_at = updated_at;
        initiative.updated_at = updated_at;
        theme.updated_at = updated_at;
        work_package.updated_at = updated_at;
        wbs.updated_at = updated_at;
        project.updated_at = updated_at;
    }

    pub fn setWbsTaskStatus(
        self: *ProjectTrackingManager,
        project_id: u32,
        wbs_id: u32,
        work_package_id: u32,
        theme_id: u32,
        initiative_id: u32,
        epic_id: u32,
        story_id: u32,
        task_id: u32,
        status: TaskStatus,
        updated_at: i64,
    ) !void {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        const wbs = getWbsMutable(project, wbs_id) orelse return error.WbsNotFound;
        const work_package = getWorkPackageMutable(wbs, work_package_id) orelse return error.WorkPackageNotFound;
        const theme = getThemeMutable(work_package, theme_id) orelse return error.ThemeNotFound;
        const initiative = getInitiativeMutable(theme, initiative_id) orelse return error.InitiativeNotFound;
        const epic = getEpicMutable(initiative, epic_id) orelse return error.EpicNotFound;
        const story = getWbsStoryMutable(epic, story_id) orelse return error.WbsStoryNotFound;
        if (task_id >= @as(u32, @intCast(story.tasks.items.len))) return error.WbsTaskNotFound;
        const task = &story.tasks.items[task_id];

        task.status = status;
        task.updated_at = updated_at;
        story.updated_at = updated_at;
        epic.updated_at = updated_at;
        initiative.updated_at = updated_at;
        theme.updated_at = updated_at;
        work_package.updated_at = updated_at;
        wbs.updated_at = updated_at;
        project.updated_at = updated_at;
    }

    pub fn getWorkBreakdownStructures(self: *ProjectTrackingManager, project_id: u32) ![]WorkBreakdownStructure {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        return project.wbs_structures.items;
    }

    pub fn addMilestone(
        self: *ProjectTrackingManager,
        project_id: u32,
        name: []const u8,
        description: []const u8,
        due_at: ?i64,
        created_at: i64,
    ) !u32 {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        const milestone_id = @as(u32, @intCast(project.milestones.items.len));

        const milestone = ProjectMilestone{
            .id = milestone_id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .due_at = due_at,
            .status = .pending,
            .created_at = created_at,
            .updated_at = created_at,
        };
        errdefer {
            self.allocator.free(milestone.name);
            self.allocator.free(milestone.description);
        }

        try project.milestones.append(milestone);
        project.updated_at = created_at;
        return milestone_id;
    }

    pub fn setMilestoneStatus(self: *ProjectTrackingManager, project_id: u32, milestone_id: u32, status: MilestoneStatus, updated_at: i64) !void {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        if (milestone_id >= @as(u32, @intCast(project.milestones.items.len))) return error.MilestoneNotFound;

        const milestone = &project.milestones.items[milestone_id];
        milestone.status = status;
        milestone.updated_at = updated_at;
        project.updated_at = updated_at;
    }

    pub fn addTask(
        self: *ProjectTrackingManager,
        project_id: u32,
        title: []const u8,
        description: []const u8,
        assignee_id: ?u32,
        due_at: ?i64,
        created_at: i64,
    ) !u32 {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        const task_id = @as(u32, @intCast(project.tasks.items.len));

        const task = ProjectTask{
            .id = task_id,
            .title = try self.allocator.dupe(u8, title),
            .description = try self.allocator.dupe(u8, description),
            .assignee_id = assignee_id,
            .due_at = due_at,
            .status = .todo,
            .created_at = created_at,
            .updated_at = created_at,
        };
        errdefer {
            self.allocator.free(task.title);
            self.allocator.free(task.description);
        }

        try project.tasks.append(task);
        project.updated_at = created_at;
        return task_id;
    }

    pub fn setTaskStatus(self: *ProjectTrackingManager, project_id: u32, task_id: u32, status: TaskStatus, updated_at: i64) !void {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        if (task_id >= @as(u32, @intCast(project.tasks.items.len))) return error.TaskNotFound;

        const task = &project.tasks.items[task_id];
        task.status = status;
        task.updated_at = updated_at;
        project.updated_at = updated_at;
    }

    pub fn addStory(
        self: *ProjectTrackingManager,
        project_id: u32,
        title: []const u8,
        description: []const u8,
        story_type: StoryType,
        priority: ProjectPriority,
        owner_id: ?u32,
        estimate_points: f32,
        created_at: i64,
    ) !u32 {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        const story_id = @as(u32, @intCast(project.stories.items.len));

        const story = Story{
            .id = story_id,
            .title = try self.allocator.dupe(u8, title),
            .description = try self.allocator.dupe(u8, description),
            .story_type = story_type,
            .status = .backlog,
            .priority = priority,
            .owner_id = owner_id,
            .estimate_points = estimate_points,
            .created_at = created_at,
            .updated_at = created_at,
        };
        errdefer {
            self.allocator.free(story.title);
            self.allocator.free(story.description);
        }

        try project.stories.append(story);
        project.updated_at = created_at;
        return story_id;
    }

    pub fn setStoryStatus(self: *ProjectTrackingManager, project_id: u32, story_id: u32, status: StoryStatus, updated_at: i64) !void {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        if (story_id >= @as(u32, @intCast(project.stories.items.len))) return error.StoryNotFound;

        const story = &project.stories.items[story_id];
        story.status = status;
        story.updated_at = updated_at;
        project.updated_at = updated_at;
    }

    pub fn updateStoryType(self: *ProjectTrackingManager, project_id: u32, story_id: u32, story_type: StoryType, updated_at: i64) !void {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        if (story_id >= @as(u32, @intCast(project.stories.items.len))) return error.StoryNotFound;

        const story = &project.stories.items[story_id];
        story.story_type = story_type;
        story.updated_at = updated_at;
        project.updated_at = updated_at;
    }

    pub fn getStories(self: *ProjectTrackingManager, project_id: u32) ![]Story {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        return project.stories.items;
    }

    pub fn filterStories(
        self: *ProjectTrackingManager,
        project_id: u32,
        filter: StoryFilter,
        allocator: std.mem.Allocator,
    ) !std.array_list.Managed(StorySummary) {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        var results = std.array_list.Managed(StorySummary).init(allocator);

        for (project.stories.items) |*story| {
            if (!matchesStoryFilter(story, filter)) continue;

            const summary = StorySummary{
                .id = story.id,
                .title = try allocator.dupe(u8, story.title),
                .story_type = story.story_type,
                .status = story.status,
                .priority = story.priority,
                .owner_id = story.owner_id,
                .updated_at = story.updated_at,
            };
            errdefer allocator.free(summary.title);
            try results.append(summary);
        }

        return results;
    }

    pub fn deinitStorySummaries(results: *std.array_list.Managed(StorySummary), allocator: std.mem.Allocator) void {
        for (results.items) |summary| {
            allocator.free(summary.title);
        }
        results.deinit();
    }

    pub fn addIssue(
        self: *ProjectTrackingManager,
        project_id: u32,
        title: []const u8,
        description: []const u8,
        severity: IssueSeverity,
        created_at: i64,
    ) !u32 {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        const issue_id = @as(u32, @intCast(project.issues.items.len));

        const issue = ProjectIssue{
            .id = issue_id,
            .title = try self.allocator.dupe(u8, title),
            .description = try self.allocator.dupe(u8, description),
            .severity = severity,
            .is_resolved = false,
            .created_at = created_at,
        };
        errdefer {
            self.allocator.free(issue.title);
            self.allocator.free(issue.description);
        }

        try project.issues.append(issue);
        project.updated_at = created_at;
        return issue_id;
    }

    pub fn setIssueResolved(self: *ProjectTrackingManager, project_id: u32, issue_id: u32, resolved: bool, updated_at: i64) !void {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        if (issue_id >= @as(u32, @intCast(project.issues.items.len))) return error.IssueNotFound;

        const issue = &project.issues.items[issue_id];
        issue.is_resolved = resolved;
        issue.resolved_at = if (resolved) updated_at else null;
        project.updated_at = updated_at;
    }

    pub fn addNote(self: *ProjectTrackingManager, project_id: u32, title: []const u8, body: []const u8, created_at: i64) !u32 {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        const note_id = @as(u32, @intCast(project.notes.items.len));

        const note = ProjectNote{
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

        try project.notes.append(note);
        project.updated_at = created_at;
        return note_id;
    }

    pub fn updateNote(self: *ProjectTrackingManager, project_id: u32, note_id: u32, title: []const u8, body: []const u8, updated_at: i64) !void {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        if (note_id >= @as(u32, @intCast(project.notes.items.len))) return error.NoteNotFound;

        const note = &project.notes.items[note_id];
        const new_title = try self.allocator.dupe(u8, title);
        errdefer self.allocator.free(new_title);
        const new_body = try self.allocator.dupe(u8, body);
        errdefer self.allocator.free(new_body);

        self.allocator.free(note.title);
        self.allocator.free(note.body);
        note.title = new_title;
        note.body = new_body;
        note.updated_at = updated_at;
        project.updated_at = updated_at;
    }

    pub fn addProgressUpdate(
        self: *ProjectTrackingManager,
        project_id: u32,
        update_type: ProgressUpdateType,
        message: []const u8,
        created_at: i64,
        author_id: ?u32,
    ) !u32 {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;
        const update_id = @as(u32, @intCast(project.updates.items.len));

        const update = ProjectProgressUpdate{
            .id = update_id,
            .update_type = update_type,
            .message = try self.allocator.dupe(u8, message),
            .created_at = created_at,
            .author_id = author_id,
        };
        errdefer self.allocator.free(update.message);

        try project.updates.append(update);
        project.updated_at = created_at;
        return update_id;
    }

    pub fn calculateProgress(self: *ProjectTrackingManager, project_id: u32) !f32 {
        const project = self.getProjectMutable(project_id) orelse return error.ProjectNotFound;

        const milestone_count = project.milestones.items.len;
        var completed_milestones: usize = 0;
        for (project.milestones.items) |milestone| {
            if (milestone.status == .completed) completed_milestones += 1;
        }

        const task_count = project.tasks.items.len;
        var completed_tasks: usize = 0;
        for (project.tasks.items) |task| {
            if (task.status == .done) completed_tasks += 1;
        }

        const story_count = project.stories.items.len;
        var completed_stories: usize = 0;
        for (project.stories.items) |story| {
            if (story.status == .done) completed_stories += 1;
        }

        if (milestone_count == 0 and task_count == 0 and story_count == 0) return 0.0;

        const milestone_progress: f32 = if (milestone_count == 0)
            0.0
        else
            @as(f32, @floatFromInt(completed_milestones)) / @as(f32, @floatFromInt(milestone_count));

        const task_progress: f32 = if (task_count == 0)
            0.0
        else
            @as(f32, @floatFromInt(completed_tasks)) / @as(f32, @floatFromInt(task_count));

        const story_progress: f32 = if (story_count == 0)
            0.0
        else
            @as(f32, @floatFromInt(completed_stories)) / @as(f32, @floatFromInt(story_count));

        if (milestone_count == 0 and task_count == 0) return story_progress * 100.0;
        if (milestone_count == 0 and story_count == 0) return task_progress * 100.0;
        if (task_count == 0 and story_count == 0) return milestone_progress * 100.0;

        return ((milestone_progress * 0.4) + (task_progress * 0.3) + (story_progress * 0.3)) * 100.0;
    }

    pub fn getProjectById(self: *ProjectTrackingManager, project_id: u32) ?*TrackedProject {
        return self.getProjectMutable(project_id);
    }

    pub fn listProjects(self: *ProjectTrackingManager) []TrackedProject {
        return self.projects.items;
    }

    pub fn filterProjects(self: *ProjectTrackingManager, filter: ProjectFilter, allocator: std.mem.Allocator) !std.array_list.Managed(ProjectSummary) {
        var results = std.array_list.Managed(ProjectSummary).init(allocator);

        for (self.projects.items) |*project| {
            if (!matchesFilter(project, filter)) continue;

            const summary = ProjectSummary{
                .id = project.id,
                .name = try allocator.dupe(u8, project.name),
                .status = project.status,
                .priority = project.priority,
                .owner_id = project.owner_id,
                .progress_percent = try self.calculateProgress(project.id),
                .open_issues = openIssueCount(project),
                .target_end_at = project.target_end_at,
                .updated_at = project.updated_at,
            };
            errdefer allocator.free(summary.name);

            try results.append(summary);
        }

        return results;
    }

    pub fn deinitProjectSummaries(results: *std.array_list.Managed(ProjectSummary), allocator: std.mem.Allocator) void {
        for (results.items) |summary| {
            allocator.free(summary.name);
        }
        results.deinit();
    }
};

test "work breakdown hierarchy supports full chain" {
    const allocator = std.testing.allocator;
    var manager = ProjectTrackingManager.init(allocator);
    defer manager.deinit();

    const created_at: i64 = 1_700_000_000;
    const project_id = try manager.createProject(
        "Kogi Platform",
        "Tracking delivery",
        null,
        .high,
        null,
        null,
        created_at,
    );

    const wbs_id = try manager.addWorkBreakdownStructure(project_id, "Release 1", "Primary release WBS", created_at);
    const work_package_id = try manager.addWorkPackage(project_id, wbs_id, "Platform Foundation", "Core platform work", created_at);
    const theme_id = try manager.addTheme(project_id, wbs_id, work_package_id, "Runtime", "Runtime capabilities", created_at);
    const initiative_id = try manager.addInitiative(
        project_id,
        wbs_id,
        work_package_id,
        theme_id,
        "Reliability",
        "Platform reliability improvements",
        .high,
        null,
        created_at,
    );
    const epic_id = try manager.addEpic(
        project_id,
        wbs_id,
        work_package_id,
        theme_id,
        initiative_id,
        "Observability Epic",
        "Improve visibility and diagnostics",
        .high,
        null,
        created_at,
    );
    const story_id = try manager.addWbsStory(
        project_id,
        wbs_id,
        work_package_id,
        theme_id,
        initiative_id,
        epic_id,
        "Instrument scheduler",
        "Add scheduling telemetry",
        .feature,
        .high,
        null,
        5.0,
        created_at,
    );
    const task_id = try manager.addWbsTask(
        project_id,
        wbs_id,
        work_package_id,
        theme_id,
        initiative_id,
        epic_id,
        story_id,
        "Emit queue metrics",
        "Capture queue depth and latency",
        null,
        null,
        created_at,
    );

    try std.testing.expectEqual(@as(u32, 0), wbs_id);
    try std.testing.expectEqual(@as(u32, 0), work_package_id);
    try std.testing.expectEqual(@as(u32, 0), theme_id);
    try std.testing.expectEqual(@as(u32, 0), initiative_id);
    try std.testing.expectEqual(@as(u32, 0), epic_id);
    try std.testing.expectEqual(@as(u32, 0), story_id);
    try std.testing.expectEqual(@as(u32, 0), task_id);

    const structures = try manager.getWorkBreakdownStructures(project_id);
    try std.testing.expectEqual(@as(usize, 1), structures.len);
    try std.testing.expectEqual(@as(usize, 1), structures[0].work_packages.items.len);
    try std.testing.expectEqual(@as(usize, 1), structures[0].work_packages.items[0].themes.items.len);
    try std.testing.expectEqual(@as(usize, 1), structures[0].work_packages.items[0].themes.items[0].initiatives.items.len);
    try std.testing.expectEqual(@as(usize, 1), structures[0].work_packages.items[0].themes.items[0].initiatives.items[0].epics.items.len);
    try std.testing.expectEqual(@as(usize, 1), structures[0].work_packages.items[0].themes.items[0].initiatives.items[0].epics.items[0].stories.items.len);
    try std.testing.expectEqual(@as(usize, 1), structures[0].work_packages.items[0].themes.items[0].initiatives.items[0].epics.items[0].stories.items[0].tasks.items.len);

    try manager.setWbsStoryStatus(
        project_id,
        wbs_id,
        work_package_id,
        theme_id,
        initiative_id,
        epic_id,
        story_id,
        .done,
        created_at + 1,
    );
    try manager.setWbsTaskStatus(
        project_id,
        wbs_id,
        work_package_id,
        theme_id,
        initiative_id,
        epic_id,
        story_id,
        task_id,
        .done,
        created_at + 1,
    );

    const updated = try manager.getWorkBreakdownStructures(project_id);
    const updated_story = updated[0].work_packages.items[0].themes.items[0].initiatives.items[0].epics.items[0].stories.items[0];
    const updated_task = updated_story.tasks.items[0];
    try std.testing.expectEqual(StoryStatus.done, updated_story.status);
    try std.testing.expectEqual(TaskStatus.done, updated_task.status);
}

pub fn projectTrackingDemo() void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var manager = ProjectTrackingManager.init(allocator);
    defer manager.deinit();

    std.debug.print("Project tracking manager initialized\n", .{});
}



//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/// 
/// 

const std = @import("std");

/// Expanded entity types to support diverse portfolio contents
pub const EntityType = enum {
    portfolio,
    program,
    project,
    task,
    initiative,
    deliverable,
    investment,
    entity,
    artifact,
    product,
    solution,
    service,
    work,
    music,
    artwork,
    code,
    asset,
    custom,
};

/// Entity classes for business domain classification
pub const EntityClass = enum {
    strategic,
    operational,
    tactical,
    support,
    research,
    development,
    creative,
    financial,
    technical,
    artistic,
    commercial,
    personal,
};

/// Tag for flexible categorization
pub const Tag = struct {
    id: u32,
    name: []const u8,
    category: []const u8,
};

/// Metadata container for rich entity information
pub const Metadata = struct {
    entity_type: EntityType,
    entity_class: EntityClass,
    category: []const u8,
    tags: std.array_list.Managed(Tag),
    custom_fields: std.StringHashMap([]const u8),
    created_at: i64,
    updated_at: i64,
    owner_id: ?u32 = null,
    parent_id: ?u32 = null,
};

/// Index entry for fast lookups
pub const IndexEntry = struct {
    id: u32,
    name: []const u8,
    entity_type: EntityType,
    parent_id: ?u32 = null,
};

/// Generic item that can represent any type of work/artifact/product
pub const PortfolioItem = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    metadata: Metadata,
};

/// A generic collection container - can hold any items with a specific purpose
pub const PortfolioCollection = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    item_type: EntityType,
    items: std.array_list.Managed(PortfolioItem),
    metadata: Metadata,
};

/// Portfolio is now a generic top-level container that can hold
/// any mix of collections and items
pub const Portfolio = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    collections: std.array_list.Managed(PortfolioCollection),
    items: std.array_list.Managed(PortfolioItem),
    metadata: Metadata,
};

/// Task represents work items (legacy support)
pub const Task = struct {
    id: u32,
    title: []const u8,
    description: []const u8,
    completed: bool,
    priority: u8,
    metadata: Metadata,
};

// Project, Program and SubPortfolio legacy types are intentionally omitted
// in favor of the generic collection/item model defined above.

/// Search filter criteria
pub const SearchFilter = struct {
    name_pattern: ?[]const u8 = null,
    entity_type: ?EntityType = null,
    entity_class: ?EntityClass = null,
    category: ?[]const u8 = null,
    tag_names: ?std.array_list.Managed([]const u8) = null,
    owner_id: ?u32 = null,
};

/// Portfolio Manager for hierarchical portfolio operations
pub const PortfolioManager = struct {
    allocator: std.mem.Allocator,
    portfolio: ?Portfolio = null,
    index: std.array_list.Managed(IndexEntry),
    next_id: u32 = 0,

    pub fn init(allocator: std.mem.Allocator) PortfolioManager {
        return PortfolioManager{
            .allocator = allocator,
            .index = std.array_list.Managed(IndexEntry).init(allocator),
        };
    }

    pub fn deinit(self: *PortfolioManager) void {
        if (self.portfolio) |*portfolio| {
            self.deinitPortfolio(portfolio);
        }
        self.deinitIndex();
    }

    fn deinitIndex(self: *PortfolioManager) void {
        for (self.index.items) |*entry| {
            self.allocator.free(entry.name);
        }
        self.index.deinit();
    }

    fn deinitMetadata(self: *PortfolioManager, metadata: *Metadata) void {
        self.allocator.free(metadata.category);
        for (metadata.tags.items) |*tag| {
            self.allocator.free(tag.name);
            self.allocator.free(tag.category);
        }
        metadata.tags.deinit();
        var iter = metadata.custom_fields.iterator();
        while (iter.next()) |entry| {
            self.allocator.free(entry.key_ptr.*);
            self.allocator.free(entry.value_ptr.*);
        }
        metadata.custom_fields.deinit();
    }

    fn deinitPortfolioItem(self: *PortfolioManager, item: *PortfolioItem) void {
        self.allocator.free(item.name);
        self.allocator.free(item.description);
        self.deinitMetadata(&item.metadata);
    }

    fn deinitCollection(self: *PortfolioManager, collection: *PortfolioCollection) void {
        for (collection.items.items) |*item| {
            self.deinitPortfolioItem(item);
        }
        collection.items.deinit();
        self.allocator.free(collection.name);
        self.allocator.free(collection.description);
        self.deinitMetadata(&collection.metadata);
    }

    fn deinitTask(self: *PortfolioManager, task: *Task) void {
        self.allocator.free(task.title);
        self.allocator.free(task.description);
        self.deinitMetadata(&task.metadata);
    }

    // Legacy project/program/sub-portfolio cleanup removed in favor
    // of the generic collection/item model.

    fn deinitPortfolio(self: *PortfolioManager, portfolio: *Portfolio) void {
        for (portfolio.collections.items) |*col| {
            self.deinitCollection(col);
        }
        portfolio.collections.deinit();
        for (portfolio.items.items) |*item| {
            self.deinitPortfolioItem(item);
        }
        portfolio.items.deinit();
        self.allocator.free(portfolio.name);
        self.allocator.free(portfolio.description);
        self.deinitMetadata(&portfolio.metadata);
    }

    /// Create a new portfolio
    pub fn createPortfolio(self: *PortfolioManager, name: []const u8, description: []const u8, category: []const u8) !void {
        const id = self.next_id;
        self.next_id += 1;

        const metadata = Metadata{
            .entity_type = .portfolio,
            .entity_class = .strategic,
            .category = try self.allocator.dupe(u8, category),
            .tags = std.array_list.Managed(Tag).init(self.allocator),
            .custom_fields = std.StringHashMap([]const u8).init(self.allocator),
            .created_at = 0,
            .updated_at = 0,
        };

        const portfolio = Portfolio{
            .id = id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .collections = std.array_list.Managed(PortfolioCollection).init(self.allocator),
            .items = std.array_list.Managed(PortfolioItem).init(self.allocator),
            .metadata = metadata,
        };

        self.portfolio = portfolio;
        try self.addToIndex(id, name, .portfolio, null);
    }

    /// Add a generic collection to the portfolio (e.g., music collection, artwork, products, etc)
    pub fn addCollection(
        self: *PortfolioManager,
        name: []const u8,
        description: []const u8,
        item_type: EntityType,
        category: []const u8,
    ) !u32 {
        if (self.portfolio == null) return error.PortfolioNotFound;

        const id = self.next_id;
        self.next_id += 1;

        const metadata = Metadata{
            .entity_type = item_type,
            .entity_class = .personal,
            .category = try self.allocator.dupe(u8, category),
            .tags = std.array_list.Managed(Tag).init(self.allocator),
            .custom_fields = std.StringHashMap([]const u8).init(self.allocator),
            .created_at = 0,
            .updated_at = 0,
            .parent_id = self.portfolio.?.id,
        };

        const collection = PortfolioCollection{
            .id = id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .item_type = item_type,
            .items = std.array_list.Managed(PortfolioItem).init(self.allocator),
            .metadata = metadata,
        };

        try self.portfolio.?.collections.append(collection);
        try self.addToIndex(id, name, item_type, self.portfolio.?.id);
        return id;
    }

    /// Add an item to a collection
    pub fn addItemToCollection(
        self: *PortfolioManager,
        collection_id: u32,
        name: []const u8,
        description: []const u8,
    ) !u32 {
        if (self.portfolio == null) return error.PortfolioNotFound;

        for (self.portfolio.?.collections.items) |*collection| {
            if (collection.id == collection_id) {
                const id = self.next_id;
                self.next_id += 1;

                const metadata = Metadata{
                    .entity_type = collection.item_type,
                    .entity_class = .personal,
                    .category = try self.allocator.dupe(u8, collection.metadata.category),
                    .tags = std.array_list.Managed(Tag).init(self.allocator),
                    .custom_fields = std.StringHashMap([]const u8).init(self.allocator),
                    .created_at = 0,
                    .updated_at = 0,
                    .parent_id = collection_id,
                };

                const item = PortfolioItem{
                    .id = id,
                    .name = try self.allocator.dupe(u8, name),
                    .description = try self.allocator.dupe(u8, description),
                    .metadata = metadata,
                };

                try collection.items.append(item);
                try self.addToIndex(id, name, collection.item_type, collection_id);
                return id;
            }
        }
        return error.CollectionNotFound;
    }

    /// Add a top-level item to the portfolio
    pub fn addItem(
        self: *PortfolioManager,
        name: []const u8,
        description: []const u8,
        item_type: EntityType,
        category: []const u8,
    ) !u32 {
        if (self.portfolio == null) return error.PortfolioNotFound;
    
        const id = self.next_id;
        self.next_id += 1;

        const metadata = Metadata{
            .entity_type = item_type,
            .entity_class = .personal,
            .category = try self.allocator.dupe(u8, category),
            .tags = std.array_list.Managed(Tag).init(self.allocator),
            .custom_fields = std.StringHashMap([]const u8).init(self.allocator),
            .created_at = 0,
            .updated_at = 0,
            .parent_id = self.portfolio.?.id,
        };

        const item = PortfolioItem{
            .id = id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .metadata = metadata,
        };

        try self.portfolio.?.items.append(item);
        try self.addToIndex(id, name, item_type, self.portfolio.?.id);
        return id;
    }

    /// Generic getCollectionCount
    pub fn getCollectionCount(self: *PortfolioManager) u32 {
        if (self.portfolio) |p| {
            return @as(u32, @intCast(p.collections.items.len));
        }
        return 0;
    }

    /// Generic getItemCount for portfolio or collection
    pub fn getPortfolioItemCount(self: *PortfolioManager) u32 {
        if (self.portfolio) |p| {
            return @as(u32, @intCast(p.items.items.len));
        }
        return 0;
    }

    /// Get collection by ID
    pub fn getCollectionById(self: *PortfolioManager, collection_id: u32) ?*PortfolioCollection {
        if (self.portfolio) |*p| {
            for (p.collections.items) |*col| {
                if (col.id == collection_id) return col;
            }
        }
        return null;
    }

    /// Get collections by item type
    pub fn getCollectionsByType(self: *PortfolioManager, item_type: EntityType) !std.array_list.Managed(*PortfolioCollection) {
        var results = std.array_list.Managed(*PortfolioCollection).init(self.allocator);
        if (self.portfolio) |*p| {
            for (p.collections.items) |*col| {
                if (col.item_type == item_type) {
                    try results.append(col);
                }
            }
        }
        return results;
    }

    /// Add sub-portfolio to main portfolio (legacy support - uses generic addItem)
    pub fn addSubPortfolio(self: *PortfolioManager, name: []const u8, description: []const u8, category: []const u8) !void {
        if (self.portfolio == null) return;

        // Store as a collection for now via generic addItem
        _ = try self.addItem(name, description, .portfolio, category);
    }

    /// Add program to portfolio
    pub fn addProgram(self: *PortfolioManager, name: []const u8, description: []const u8, category: []const u8) !void {
        if (self.portfolio == null) return;
        _ = try self.addCollection(name, description, .program, category);
    }

    /// Add project to portfolio
    pub fn addProject(self: *PortfolioManager, name: []const u8, description: []const u8, category: []const u8) !void {
        if (self.portfolio == null) return;
        _ = try self.addCollection(name, description, .project, category);
    }

    /// Add tag to entity
    pub fn addTagToEntity(self: *PortfolioManager, entity_id: u32, tag_name: []const u8, tag_category: []const u8) !void {
        if (self.portfolio == null) return;

        // Search through all entities
        if (self.findEntityMetadata(entity_id)) |metadata| {
            const tag_id = @as(u32, @intCast(metadata.tags.items.len));
            const tag = Tag{
                .id = tag_id,
                .name = try self.allocator.dupe(u8, tag_name),
                .category = try self.allocator.dupe(u8, tag_category),
            };
            try metadata.tags.append(tag);
        }
    }

    /// Find metadata for entity by ID
    fn findEntityMetadata(self: *PortfolioManager, entity_id: u32) ?*Metadata {
        if (self.portfolio) |*portfolio| {
            if (portfolio.id == entity_id) return &portfolio.metadata;

            // Search collections and their items
            for (portfolio.collections.items) |*collection| {
                if (collection.id == entity_id) return &collection.metadata;
                for (collection.items.items) |*item| {
                    if (item.id == entity_id) return &item.metadata;
                }
            }

            // Search top-level items
            for (portfolio.items.items) |*item| {
                if (item.id == entity_id) return &item.metadata;
            }
        }

        return null;
    }

    /// Add index entry
    fn addToIndex(self: *PortfolioManager, id: u32, name: []const u8, entity_type: EntityType, parent_id: ?u32) !void {
        const entry = IndexEntry{
            .id = id,
            .name = try self.allocator.dupe(u8, name),
            .entity_type = entity_type,
            .parent_id = parent_id,
        };
        try self.index.append(entry);
    }

    /// Search entities by filter
    pub fn search(self: *PortfolioManager, filter: SearchFilter, allocator: std.mem.Allocator) !std.array_list.Managed(IndexEntry) {
        var results = std.array_list.Managed(IndexEntry).init(allocator);

        for (self.index.items) |entry| {
            var matches = true;

            if (filter.name_pattern) |pattern| {
                if (std.mem.indexOf(u8, entry.name, pattern) == null) {
                    matches = false;
                }
            }

            if (filter.entity_type) |etype| {
                if (entry.entity_type != etype) {
                    matches = false;
                }
            }

            if (matches) {
                const entry_copy = IndexEntry{
                    .id = entry.id,
                    .name = try allocator.dupe(u8, entry.name),
                    .entity_type = entry.entity_type,
                    .parent_id = entry.parent_id,
                };
                try results.append(entry_copy);
            }
        }

        return results;
    }

    /// Get entity by ID from index
    pub fn getEntityById(self: *PortfolioManager, id: u32) ?IndexEntry {
        for (self.index.items) |entry| {
            if (entry.id == id) {
                return entry;
            }
        }
        return null;
    }

    /// Get all entities of a specific type
    pub fn getEntitiesByType(self: *PortfolioManager, entity_type: EntityType, allocator: std.mem.Allocator) !std.array_list.Managed(IndexEntry) {
        var results = std.array_list.Managed(IndexEntry).init(allocator);
        for (self.index.items) |entry| {
            if (entry.entity_type == entity_type) {
                const entry_copy = IndexEntry{
                    .id = entry.id,
                    .name = try allocator.dupe(u8, entry.name),
                    .entity_type = entry.entity_type,
                    .parent_id = entry.parent_id,
                };
                try results.append(entry_copy);
            }
        }
        return results;
    }
};

pub fn portfolioDemo() void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var manager = PortfolioManager.init(allocator);
    defer manager.deinit();

    std.debug.print("Portfolio Manager initialized\n", .{});
}

const std = @import("std");

/// Expanded entity types to support diverse portfolio contents
pub const EntityType = enum {
    portfolio,
    program,
    project,
    task,
    initiative,
    deliverable,
    investment,
    entity,
    artifact,
    product,
    solution,
    service,
    work,
    music,
    artwork,
    code,
    asset,
    custom,
};

/// Entity classes for business domain classification
pub const EntityClass = enum {
    strategic,
    operational,
    tactical,
    support,
    research,
    development,
    creative,
    financial,
    technical,
    artistic,
    commercial,
    personal,
};

/// Tag for flexible categorization
pub const Tag = struct {
    id: u32,
    name: []const u8,
    category: []const u8,
};

/// Metadata container for rich entity information
pub const Metadata = struct {
    entity_type: EntityType,
    entity_class: EntityClass,
    category: []const u8,
    tags: std.array_list.Managed(Tag),
    custom_fields: std.StringHashMap([]const u8),
    created_at: i64,
    updated_at: i64,
    owner_id: ?u32 = null,
    parent_id: ?u32 = null,
};

/// Index entry for fast lookups
pub const IndexEntry = struct {
    id: u32,
    name: []const u8,
    entity_type: EntityType,
    parent_id: ?u32 = null,
};

/// Generic item that can represent any type of work/artifact/product
pub const PortfolioItem = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    metadata: Metadata,
};

/// A generic collection container - can hold any items with a specific purpose
pub const PortfolioCollection = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    item_type: EntityType,
    items: std.array_list.Managed(PortfolioItem),
    metadata: Metadata,
};

/// Portfolio is now a generic top-level container that can hold
/// any mix of collections and items
pub const Portfolio = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    collections: std.array_list.Managed(PortfolioCollection),
    items: std.array_list.Managed(PortfolioItem),
    metadata: Metadata,
};

/// Task represents work items (legacy support)
pub const Task = struct {
    id: u32,
    title: []const u8,
    description: []const u8,
    completed: bool,
    priority: u8,
    metadata: Metadata,
};

// Project, Program and SubPortfolio legacy types are intentionally omitted
// in favor of the generic collection/item model defined above.

/// Search filter criteria
pub const SearchFilter = struct {
    name_pattern: ?[]const u8 = null,
    entity_type: ?EntityType = null,
    entity_class: ?EntityClass = null,
    category: ?[]const u8 = null,
    tag_names: ?std.array_list.Managed([]const u8) = null,
    owner_id: ?u32 = null,
};

/// Portfolio Manager for hierarchical portfolio operations
pub const PortfolioManager = struct {
    allocator: std.mem.Allocator,
    portfolio: ?Portfolio = null,
    index: std.array_list.Managed(IndexEntry),
    next_id: u32 = 0,

    pub fn init(allocator: std.mem.Allocator) PortfolioManager {
        return PortfolioManager{
            .allocator = allocator,
            .index = std.array_list.Managed(IndexEntry).init(allocator),
        };
    }

    pub fn deinit(self: *PortfolioManager) void {
        if (self.portfolio) |*portfolio| {
            self.deinitPortfolio(portfolio);
        }
        self.deinitIndex();
    }

    fn deinitIndex(self: *PortfolioManager) void {
        for (self.index.items) |*entry| {
            self.allocator.free(entry.name);
        }
        self.index.deinit();
    }

    fn deinitMetadata(self: *PortfolioManager, metadata: *Metadata) void {
        self.allocator.free(metadata.category);
        for (metadata.tags.items) |*tag| {
            self.allocator.free(tag.name);
            self.allocator.free(tag.category);
        }
        metadata.tags.deinit();
        var iter = metadata.custom_fields.iterator();
        while (iter.next()) |entry| {
            self.allocator.free(entry.key_ptr.*);
            self.allocator.free(entry.value_ptr.*);
        }
        metadata.custom_fields.deinit();
    }

    fn deinitPortfolioItem(self: *PortfolioManager, item: *PortfolioItem) void {
        self.allocator.free(item.name);
        self.allocator.free(item.description);
        self.deinitMetadata(&item.metadata);
    }

    fn deinitCollection(self: *PortfolioManager, collection: *PortfolioCollection) void {
        for (collection.items.items) |*item| {
            self.deinitPortfolioItem(item);
        }
        collection.items.deinit();
        self.allocator.free(collection.name);
        self.allocator.free(collection.description);
        self.deinitMetadata(&collection.metadata);
    }

    fn deinitTask(self: *PortfolioManager, task: *Task) void {
        self.allocator.free(task.title);
        self.allocator.free(task.description);
        self.deinitMetadata(&task.metadata);
    }

    // Legacy project/program/sub-portfolio cleanup removed in favor
    // of the generic collection/item model.

    fn deinitPortfolio(self: *PortfolioManager, portfolio: *Portfolio) void {
        for (portfolio.collections.items) |*col| {
            self.deinitCollection(col);
        }
        portfolio.collections.deinit();
        for (portfolio.items.items) |*item| {
            self.deinitPortfolioItem(item);
        }
        portfolio.items.deinit();
        self.allocator.free(portfolio.name);
        self.allocator.free(portfolio.description);
        self.deinitMetadata(&portfolio.metadata);
    }

    /// Create a new portfolio
    pub fn createPortfolio(self: *PortfolioManager, name: []const u8, description: []const u8, category: []const u8) !void {
        const id = self.next_id;
        self.next_id += 1;

        const metadata = Metadata{
            .entity_type = .portfolio,
            .entity_class = .strategic,
            .category = try self.allocator.dupe(u8, category),
            .tags = std.array_list.Managed(Tag).init(self.allocator),
            .custom_fields = std.StringHashMap([]const u8).init(self.allocator),
            .created_at = 0,
            .updated_at = 0,
        };

        const portfolio = Portfolio{
            .id = id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .collections = std.array_list.Managed(PortfolioCollection).init(self.allocator),
            .items = std.array_list.Managed(PortfolioItem).init(self.allocator),
            .metadata = metadata,
        };

        self.portfolio = portfolio;
        try self.addToIndex(id, name, .portfolio, null);
    }

    /// Add a generic collection to the portfolio (e.g., music collection, artwork, products, etc)
    pub fn addCollection(
        self: *PortfolioManager,
        name: []const u8,
        description: []const u8,
        item_type: EntityType,
        category: []const u8,
    ) !u32 {
        if (self.portfolio == null) return error.PortfolioNotFound;

        const id = self.next_id;
        self.next_id += 1;

        const metadata = Metadata{
            .entity_type = item_type,
            .entity_class = .personal,
            .category = try self.allocator.dupe(u8, category),
            .tags = std.array_list.Managed(Tag).init(self.allocator),
            .custom_fields = std.StringHashMap([]const u8).init(self.allocator),
            .created_at = 0,
            .updated_at = 0,
            .parent_id = self.portfolio.?.id,
        };

        const collection = PortfolioCollection{
            .id = id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .item_type = item_type,
            .items = std.array_list.Managed(PortfolioItem).init(self.allocator),
            .metadata = metadata,
        };

        try self.portfolio.?.collections.append(collection);
        try self.addToIndex(id, name, item_type, self.portfolio.?.id);
        return id;
    }

    /// Add an item to a collection
    pub fn addItemToCollection(
        self: *PortfolioManager,
        collection_id: u32,
        name: []const u8,
        description: []const u8,
    ) !u32 {
        if (self.portfolio == null) return error.PortfolioNotFound;

        for (self.portfolio.?.collections.items) |*collection| {
            if (collection.id == collection_id) {
                const id = self.next_id;
                self.next_id += 1;

                const metadata = Metadata{
                    .entity_type = collection.item_type,
                    .entity_class = .personal,
                    .category = try self.allocator.dupe(u8, collection.metadata.category),
                    .tags = std.array_list.Managed(Tag).init(self.allocator),
                    .custom_fields = std.StringHashMap([]const u8).init(self.allocator),
                    .created_at = 0,
                    .updated_at = 0,
                    .parent_id = collection_id,
                };

                const item = PortfolioItem{
                    .id = id,
                    .name = try self.allocator.dupe(u8, name),
                    .description = try self.allocator.dupe(u8, description),
                    .metadata = metadata,
                };

                try collection.items.append(item);
                try self.addToIndex(id, name, collection.item_type, collection_id);
                return id;
            }
        }
        return error.CollectionNotFound;
    }

    /// Add a top-level item to the portfolio
    pub fn addItem(
        self: *PortfolioManager,
        name: []const u8,
        description: []const u8,
        item_type: EntityType,
        category: []const u8,
    ) !u32 {
        if (self.portfolio == null) return error.PortfolioNotFound;
    
        const id = self.next_id;
        self.next_id += 1;

        const metadata = Metadata{
            .entity_type = item_type,
            .entity_class = .personal,
            .category = try self.allocator.dupe(u8, category),
            .tags = std.array_list.Managed(Tag).init(self.allocator),
            .custom_fields = std.StringHashMap([]const u8).init(self.allocator),
            .created_at = 0,
            .updated_at = 0,
            .parent_id = self.portfolio.?.id,
        };

        const item = PortfolioItem{
            .id = id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .metadata = metadata,
        };

        try self.portfolio.?.items.append(item);
        try self.addToIndex(id, name, item_type, self.portfolio.?.id);
        return id;
    }

    /// Generic getCollectionCount
    pub fn getCollectionCount(self: *PortfolioManager) u32 {
        if (self.portfolio) |p| {
            return @as(u32, @intCast(p.collections.items.len));
        }
        return 0;
    }

    /// Generic getItemCount for portfolio or collection
    pub fn getPortfolioItemCount(self: *PortfolioManager) u32 {
        if (self.portfolio) |p| {
            return @as(u32, @intCast(p.items.items.len));
        }
        return 0;
    }

    /// Get collection by ID
    pub fn getCollectionById(self: *PortfolioManager, collection_id: u32) ?*PortfolioCollection {
        if (self.portfolio) |*p| {
            for (p.collections.items) |*col| {
                if (col.id == collection_id) return col;
            }
        }
        return null;
    }

    /// Get collections by item type
    pub fn getCollectionsByType(self: *PortfolioManager, item_type: EntityType) !std.array_list.Managed(*PortfolioCollection) {
        var results = std.array_list.Managed(*PortfolioCollection).init(self.allocator);
        if (self.portfolio) |*p| {
            for (p.collections.items) |*col| {
                if (col.item_type == item_type) {
                    try results.append(col);
                }
            }
        }
        return results;
    }

    /// Add sub-portfolio to main portfolio (legacy support - uses generic addItem)
    pub fn addSubPortfolio(self: *PortfolioManager, name: []const u8, description: []const u8, category: []const u8) !void {
        if (self.portfolio == null) return;

        // Store as a collection for now via generic addItem
        _ = try self.addItem(name, description, .portfolio, category);
    }

    /// Add program to portfolio
    pub fn addProgram(self: *PortfolioManager, name: []const u8, description: []const u8, category: []const u8) !void {
        if (self.portfolio == null) return;
        _ = try self.addCollection(name, description, .program, category);
    }

    /// Add project to portfolio
    pub fn addProject(self: *PortfolioManager, name: []const u8, description: []const u8, category: []const u8) !void {
        if (self.portfolio == null) return;
        _ = try self.addCollection(name, description, .project, category);
    }

    /// Add tag to entity
    pub fn addTagToEntity(self: *PortfolioManager, entity_id: u32, tag_name: []const u8, tag_category: []const u8) !void {
        if (self.portfolio == null) return;

        // Search through all entities
        if (self.findEntityMetadata(entity_id)) |metadata| {
            const tag_id = @as(u32, @intCast(metadata.tags.items.len));
            const tag = Tag{
                .id = tag_id,
                .name = try self.allocator.dupe(u8, tag_name),
                .category = try self.allocator.dupe(u8, tag_category),
            };
            try metadata.tags.append(tag);
        }
    }

    /// Find metadata for entity by ID
    fn findEntityMetadata(self: *PortfolioManager, entity_id: u32) ?*Metadata {
        if (self.portfolio) |*portfolio| {
            if (portfolio.id == entity_id) return &portfolio.metadata;

            // Search collections and their items
            for (portfolio.collections.items) |*collection| {
                if (collection.id == entity_id) return &collection.metadata;
                for (collection.items.items) |*item| {
                    if (item.id == entity_id) return &item.metadata;
                }
            }

            // Search top-level items
            for (portfolio.items.items) |*item| {
                if (item.id == entity_id) return &item.metadata;
            }
        }

        return null;
    }

    /// Add index entry
    fn addToIndex(self: *PortfolioManager, id: u32, name: []const u8, entity_type: EntityType, parent_id: ?u32) !void {
        const entry = IndexEntry{
            .id = id,
            .name = try self.allocator.dupe(u8, name),
            .entity_type = entity_type,
            .parent_id = parent_id,
        };
        try self.index.append(entry);
    }

    /// Search entities by filter
    pub fn search(self: *PortfolioManager, filter: SearchFilter, allocator: std.mem.Allocator) !std.array_list.Managed(IndexEntry) {
        var results = std.array_list.Managed(IndexEntry).init(allocator);

        for (self.index.items) |entry| {
            var matches = true;

            if (filter.name_pattern) |pattern| {
                if (std.mem.indexOf(u8, entry.name, pattern) == null) {
                    matches = false;
                }
            }

            if (filter.entity_type) |etype| {
                if (entry.entity_type != etype) {
                    matches = false;
                }
            }

            if (matches) {
                const entry_copy = IndexEntry{
                    .id = entry.id,
                    .name = try allocator.dupe(u8, entry.name),
                    .entity_type = entry.entity_type,
                    .parent_id = entry.parent_id,
                };
                try results.append(entry_copy);
            }
        }

        return results;
    }

    /// Get entity by ID from index
    pub fn getEntityById(self: *PortfolioManager, id: u32) ?IndexEntry {
        for (self.index.items) |entry| {
            if (entry.id == id) {
                return entry;
            }
        }
        return null;
    }

    /// Get all entities of a specific type
    pub fn getEntitiesByType(self: *PortfolioManager, entity_type: EntityType, allocator: std.mem.Allocator) !std.array_list.Managed(IndexEntry) {
        var results = std.array_list.Managed(IndexEntry).init(allocator);
        for (self.index.items) |entry| {
            if (entry.entity_type == entity_type) {
                const entry_copy = IndexEntry{
                    .id = entry.id,
                    .name = try allocator.dupe(u8, entry.name),
                    .entity_type = entry.entity_type,
                    .parent_id = entry.parent_id,
                };
                try results.append(entry_copy);
            }
        }
        return results;
    }
};

pub fn portfolioDemo() void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var manager = PortfolioManager.init(allocator);
    defer manager.deinit();

    std.debug.print("Portfolio Manager initialized\n", .{});
}
