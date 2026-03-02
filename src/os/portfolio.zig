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
    tags: std.ArrayList(Tag),
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
    items: std.ArrayList(PortfolioItem),
    metadata: Metadata,
};

/// Portfolio is now a generic top-level container that can hold
/// any mix of collections and items
pub const Portfolio = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    collections: std.ArrayList(PortfolioCollection),
    items: std.ArrayList(PortfolioItem),
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
    tag_names: ?std.ArrayList([]const u8) = null,
    owner_id: ?u32 = null,
};

/// Portfolio Manager for hierarchical portfolio operations
pub const PortfolioManager = struct {
    allocator: std.mem.Allocator,
    portfolio: ?Portfolio = null,
    index: std.ArrayList(IndexEntry),
    next_id: u32 = 0,

    pub fn init(allocator: std.mem.Allocator) PortfolioManager {
        return PortfolioManager{
            .allocator = allocator,
            .index = std.ArrayList(IndexEntry){},
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
        self.index.deinit(self.allocator);
    }

    fn deinitMetadata(self: *PortfolioManager, metadata: *Metadata) void {
        self.allocator.free(metadata.category);
        for (metadata.tags.items) |*tag| {
            self.allocator.free(tag.name);
            self.allocator.free(tag.category);
        }
        metadata.tags.deinit(self.allocator);
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
        collection.items.deinit(self.allocator);
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
        portfolio.collections.deinit(self.allocator);
        for (portfolio.items.items) |*item| {
            self.deinitPortfolioItem(item);
        }
        portfolio.items.deinit(self.allocator);
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
            .tags = std.ArrayList(Tag){},
            .custom_fields = std.StringHashMap([]const u8).init(self.allocator),
            .created_at = 0,
            .updated_at = 0,
        };

        const portfolio = Portfolio{
            .id = id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .collections = std.ArrayList(PortfolioCollection){},
            .items = std.ArrayList(PortfolioItem){},
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
            .tags = std.ArrayList(Tag){},
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
            .items = std.ArrayList(PortfolioItem){},
            .metadata = metadata,
        };

        try self.portfolio.?.collections.append(self.allocator, collection);
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
                    .tags = std.ArrayList(Tag){},
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

                try collection.items.append(self.allocator, item);
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
            .tags = std.ArrayList(Tag){},
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

        try self.portfolio.?.items.append(self.allocator, item);
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
    pub fn getCollectionsByType(self: *PortfolioManager, item_type: EntityType) !std.ArrayList(*PortfolioCollection) {
        var results = std.ArrayList(*PortfolioCollection){};
        if (self.portfolio) |*p| {
            for (p.collections.items) |*col| {
                if (col.item_type == item_type) {
                    try results.append(&self.allocator, col);
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
            try metadata.tags.append(self.allocator, tag);
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
        try self.index.append(self.allocator, entry);
    }

    /// Search entities by filter
    pub fn search(self: *PortfolioManager, filter: SearchFilter, allocator: std.mem.Allocator) !std.ArrayList(IndexEntry) {
        var results = std.ArrayList(IndexEntry){};

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
                try results.append(allocator, entry_copy);
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
    pub fn getEntitiesByType(self: *PortfolioManager, entity_type: EntityType, allocator: std.mem.Allocator) !std.ArrayList(IndexEntry) {
        var results = std.ArrayList(IndexEntry){};
        for (self.index.items) |entry| {
            if (entry.entity_type == entity_type) {
                const entry_copy = IndexEntry{
                    .id = entry.id,
                    .name = try allocator.dupe(u8, entry.name),
                    .entity_type = entry.entity_type,
                    .parent_id = entry.parent_id,
                };
                try results.append(allocator, entry_copy);
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
