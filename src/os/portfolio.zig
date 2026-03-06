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
        self.deinitIdeas();
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
