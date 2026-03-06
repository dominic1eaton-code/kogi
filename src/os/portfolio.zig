const std = @import("std");

/// Expanded entity types to support portfolio entities, resources, containers, and indexing.
pub const EntityType = enum {
    portfolio,
    sub_portfolio,
    program,
    project,
    resource,
    collection,
    directory,
    container,
    artifact,
    asset,
    capital,
    land,
    estate,
    labor,
    investment,
    account,
    task,
    initiative,
    deliverable,
    entity,
    product,
    solution,
    service,
    work,
    music,
    artwork,
    code,
    custom,
};

/// Portfolio item classes requested by domain design.
pub const PortfolioItemType = enum {
    project,
    program,
    sub_portfolio,
    resource,
};

/// Resource sub-types.
pub const ResourceType = enum {
    artifact,
    asset,
    capital,
    land,
    estate,
    labor,
    investment,
    account,
};

/// Container sub-types for portfolio items.
pub const ContainerType = enum {
    binder,
    book,
    notepad,
    folder,
    briefcase,
};

/// Entity classes for business domain classification.
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

/// Tag for flexible categorization.
pub const Tag = struct {
    id: u32,
    name: []const u8,
    category: []const u8,
};

/// Metadata container for rich entity information.
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

/// Index entry for fast lookups.
pub const IndexEntry = struct {
    id: u32,
    name: []const u8,
    entity_type: EntityType,
    parent_id: ?u32 = null,
};

/// Container attached to a portfolio item.
pub const ItemContainer = struct {
    id: u32,
    container_type: ContainerType,
    name: []const u8,
    description: []const u8,
    created_at: i64,
    updated_at: i64,
};

/// Portfolio item with metadata and optional container attachments.
pub const PortfolioItem = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    item_type: PortfolioItemType,
    resource_type: ?ResourceType = null,
    containers: std.array_list.Managed(ItemContainer),
    metadata: Metadata,
};

/// A group of portfolio items.
pub const PortfolioCollection = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    item_type: EntityType,
    items: std.array_list.Managed(PortfolioItem),
    metadata: Metadata,
};

/// A line item in a directory schedule/list.
pub const DirectoryEntry = struct {
    id: u32,
    item_id: u32,
    scheduled_at: ?i64 = null,
    note: []const u8,
    created_at: i64,
    updated_at: i64,
};

/// A list/schedule of portfolio items.
pub const PortfolioDirectory = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    entries: std.array_list.Managed(DirectoryEntry),
    metadata: Metadata,
};

/// Top-level portfolio aggregate.
pub const Portfolio = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    collections: std.array_list.Managed(PortfolioCollection),
    directories: std.array_list.Managed(PortfolioDirectory),
    items: std.array_list.Managed(PortfolioItem),
    metadata: Metadata,
};

/// Legacy task structure kept for compatibility.
pub const Task = struct {
    id: u32,
    title: []const u8,
    description: []const u8,
    completed: bool,
    priority: u8,
    metadata: Metadata,
};

/// Backward compatibility aliases.
pub const Project = PortfolioItem;
pub const Program = PortfolioItem;
pub const SubPortfolio = PortfolioItem;

/// Search filter criteria.
pub const SearchFilter = struct {
    name_pattern: ?[]const u8 = null,
    entity_type: ?EntityType = null,
    entity_class: ?EntityClass = null,
    category: ?[]const u8 = null,
    tag_names: ?std.array_list.Managed([]const u8) = null,
    owner_id: ?u32 = null,
};

/// Portfolio manager for hierarchical portfolio operations.
pub const PortfolioManager = struct {
    allocator: std.mem.Allocator,
    portfolio: ?Portfolio = null,
    index: std.array_list.Managed(IndexEntry),
    next_id: u32 = 0,

    pub fn init(allocator: std.mem.Allocator) PortfolioManager {
        return .{
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

    fn deinitItemContainer(self: *PortfolioManager, container: *ItemContainer) void {
        self.allocator.free(container.name);
        self.allocator.free(container.description);
    }

    fn deinitPortfolioItem(self: *PortfolioManager, item: *PortfolioItem) void {
        self.allocator.free(item.name);
        self.allocator.free(item.description);
        for (item.containers.items) |*container| {
            self.deinitItemContainer(container);
        }
        item.containers.deinit();
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

    fn deinitDirectoryEntry(self: *PortfolioManager, entry: *DirectoryEntry) void {
        self.allocator.free(entry.note);
    }

    fn deinitDirectory(self: *PortfolioManager, directory: *PortfolioDirectory) void {
        for (directory.entries.items) |*entry| {
            self.deinitDirectoryEntry(entry);
        }
        directory.entries.deinit();
        self.allocator.free(directory.name);
        self.allocator.free(directory.description);
        self.deinitMetadata(&directory.metadata);
    }

    fn deinitPortfolio(self: *PortfolioManager, portfolio: *Portfolio) void {
        for (portfolio.collections.items) |*collection| {
            self.deinitCollection(collection);
        }
        portfolio.collections.deinit();

        for (portfolio.directories.items) |*directory| {
            self.deinitDirectory(directory);
        }
        portfolio.directories.deinit();

        for (portfolio.items.items) |*item| {
            self.deinitPortfolioItem(item);
        }
        portfolio.items.deinit();

        self.allocator.free(portfolio.name);
        self.allocator.free(portfolio.description);
        self.deinitMetadata(&portfolio.metadata);
    }

    fn createMetadata(
        self: *PortfolioManager,
        entity_type: EntityType,
        entity_class: EntityClass,
        category: []const u8,
        parent_id: ?u32,
    ) !Metadata {
        return .{
            .entity_type = entity_type,
            .entity_class = entity_class,
            .category = try self.allocator.dupe(u8, category),
            .tags = std.array_list.Managed(Tag).init(self.allocator),
            .custom_fields = std.StringHashMap([]const u8).init(self.allocator),
            .created_at = 0,
            .updated_at = 0,
            .parent_id = parent_id,
        };
    }

    fn resourceTypeToEntityType(resource_type: ResourceType) EntityType {
        return switch (resource_type) {
            .artifact => .artifact,
            .asset => .asset,
            .capital => .capital,
            .land => .land,
            .estate => .estate,
            .labor => .labor,
            .investment => .investment,
            .account => .account,
        };
    }

    fn classifyEntityType(entity_type: EntityType) struct { item_type: PortfolioItemType, resource_type: ?ResourceType } {
        return switch (entity_type) {
            .project => .{ .item_type = .project, .resource_type = null },
            .program => .{ .item_type = .program, .resource_type = null },
            .sub_portfolio, .portfolio => .{ .item_type = .sub_portfolio, .resource_type = null },
            .resource => .{ .item_type = .resource, .resource_type = null },
            .artifact => .{ .item_type = .resource, .resource_type = .artifact },
            .asset => .{ .item_type = .resource, .resource_type = .asset },
            .capital => .{ .item_type = .resource, .resource_type = .capital },
            .land => .{ .item_type = .resource, .resource_type = .land },
            .estate => .{ .item_type = .resource, .resource_type = .estate },
            .labor => .{ .item_type = .resource, .resource_type = .labor },
            .investment => .{ .item_type = .resource, .resource_type = .investment },
            .account => .{ .item_type = .resource, .resource_type = .account },
            else => .{ .item_type = .resource, .resource_type = null },
        };
    }

    fn entityTypeForPortfolioItem(item_type: PortfolioItemType, resource_type: ?ResourceType) EntityType {
        return switch (item_type) {
            .project => .project,
            .program => .program,
            .sub_portfolio => .sub_portfolio,
            .resource => if (resource_type) |rtype| resourceTypeToEntityType(rtype) else .resource,
        };
    }

    fn buildPortfolioItem(
        self: *PortfolioManager,
        id: u32,
        parent_id: ?u32,
        name: []const u8,
        description: []const u8,
        item_type: PortfolioItemType,
        resource_type: ?ResourceType,
        category: []const u8,
    ) !PortfolioItem {
        const normalized_resource_type = if (item_type == .resource) resource_type else null;
        const entity_type = entityTypeForPortfolioItem(item_type, normalized_resource_type);

        var metadata = try self.createMetadata(entity_type, .personal, category, parent_id);
        errdefer self.deinitMetadata(&metadata);

        const item_name = try self.allocator.dupe(u8, name);
        errdefer self.allocator.free(item_name);

        const item_description = try self.allocator.dupe(u8, description);
        errdefer self.allocator.free(item_description);

        return .{
            .id = id,
            .name = item_name,
            .description = item_description,
            .item_type = item_type,
            .resource_type = normalized_resource_type,
            .containers = std.array_list.Managed(ItemContainer).init(self.allocator),
            .metadata = metadata,
        };
    }

    fn getCollectionMutable(self: *PortfolioManager, collection_id: u32) ?*PortfolioCollection {
        if (self.portfolio) |*portfolio| {
            for (portfolio.collections.items) |*collection| {
                if (collection.id == collection_id) return collection;
            }
        }
        return null;
    }

    fn getDirectoryMutable(self: *PortfolioManager, directory_id: u32) ?*PortfolioDirectory {
        if (self.portfolio) |*portfolio| {
            for (portfolio.directories.items) |*directory| {
                if (directory.id == directory_id) return directory;
            }
        }
        return null;
    }

    fn findPortfolioItemMutable(self: *PortfolioManager, item_id: u32) ?*PortfolioItem {
        if (self.portfolio) |*portfolio| {
            for (portfolio.items.items) |*item| {
                if (item.id == item_id) return item;
            }

            for (portfolio.collections.items) |*collection| {
                for (collection.items.items) |*item| {
                    if (item.id == item_id) return item;
                }
            }
        }
        return null;
    }

    fn metadataHasTag(metadata: *const Metadata, tag_name: []const u8) bool {
        for (metadata.tags.items) |tag| {
            if (std.mem.eql(u8, tag.name, tag_name)) return true;
        }
        return false;
    }

    /// Create a new portfolio.
    pub fn createPortfolio(self: *PortfolioManager, name: []const u8, description: []const u8, category: []const u8) !void {
        const id = self.next_id;
        self.next_id += 1;

        var metadata = try self.createMetadata(.portfolio, .strategic, category, null);
        errdefer self.deinitMetadata(&metadata);

        const portfolio_name = try self.allocator.dupe(u8, name);
        errdefer self.allocator.free(portfolio_name);

        const portfolio_description = try self.allocator.dupe(u8, description);
        errdefer self.allocator.free(portfolio_description);

        var portfolio = Portfolio{
            .id = id,
            .name = portfolio_name,
            .description = portfolio_description,
            .collections = std.array_list.Managed(PortfolioCollection).init(self.allocator),
            .directories = std.array_list.Managed(PortfolioDirectory).init(self.allocator),
            .items = std.array_list.Managed(PortfolioItem).init(self.allocator),
            .metadata = metadata,
        };

        errdefer self.deinitPortfolio(&portfolio);

        self.portfolio = portfolio;
        errdefer {
            if (self.portfolio) |*created| {
                self.deinitPortfolio(created);
                self.portfolio = null;
            }
        }

        try self.addToIndex(id, name, .portfolio, null);
    }

    /// Add a collection (group of portfolio items).
    pub fn addCollection(
        self: *PortfolioManager,
        name: []const u8,
        description: []const u8,
        item_type: EntityType,
        category: []const u8,
    ) !u32 {
        const portfolio = self.portfolio orelse return error.PortfolioNotFound;

        const id = self.next_id;
        self.next_id += 1;

        var metadata = try self.createMetadata(.collection, .operational, category, portfolio.id);
        errdefer self.deinitMetadata(&metadata);

        const collection_name = try self.allocator.dupe(u8, name);
        errdefer self.allocator.free(collection_name);

        const collection_description = try self.allocator.dupe(u8, description);
        errdefer self.allocator.free(collection_description);

        var collection = PortfolioCollection{
            .id = id,
            .name = collection_name,
            .description = collection_description,
            .item_type = item_type,
            .items = std.array_list.Managed(PortfolioItem).init(self.allocator),
            .metadata = metadata,
        };
        errdefer self.deinitCollection(&collection);

        try self.portfolio.?.collections.append(collection);
        try self.addToIndex(id, name, .collection, self.portfolio.?.id);
        return id;
    }

    /// Add a top-level typed portfolio item.
    pub fn addPortfolioItem(
        self: *PortfolioManager,
        name: []const u8,
        description: []const u8,
        item_type: PortfolioItemType,
        resource_type: ?ResourceType,
        category: []const u8,
    ) !u32 {
        const portfolio = self.portfolio orelse return error.PortfolioNotFound;

        const id = self.next_id;
        self.next_id += 1;

        var item = try self.buildPortfolioItem(id, portfolio.id, name, description, item_type, resource_type, category);
        errdefer self.deinitPortfolioItem(&item);

        try self.portfolio.?.items.append(item);
        try self.addToIndex(id, name, item.metadata.entity_type, portfolio.id);
        return id;
    }

    /// Add a resource portfolio item.
    pub fn addResource(
        self: *PortfolioManager,
        name: []const u8,
        description: []const u8,
        resource_type: ResourceType,
        category: []const u8,
    ) !u32 {
        return try self.addPortfolioItem(name, description, .resource, resource_type, category);
    }

    /// Legacy add-item API, mapped to typed items.
    pub fn addItem(
        self: *PortfolioManager,
        name: []const u8,
        description: []const u8,
        item_type: EntityType,
        category: []const u8,
    ) !u32 {
        const classified = classifyEntityType(item_type);
        return try self.addPortfolioItem(name, description, classified.item_type, classified.resource_type, category);
    }

    /// Add a typed portfolio item to a collection.
    pub fn addPortfolioItemToCollection(
        self: *PortfolioManager,
        collection_id: u32,
        name: []const u8,
        description: []const u8,
        item_type: PortfolioItemType,
        resource_type: ?ResourceType,
        category: []const u8,
    ) !u32 {
        _ = self.portfolio orelse return error.PortfolioNotFound;
        const collection = self.getCollectionMutable(collection_id) orelse return error.CollectionNotFound;

        const id = self.next_id;
        self.next_id += 1;

        var item = try self.buildPortfolioItem(id, collection_id, name, description, item_type, resource_type, category);
        errdefer self.deinitPortfolioItem(&item);

        try collection.items.append(item);
        try self.addToIndex(id, name, item.metadata.entity_type, collection_id);
        return id;
    }

    /// Legacy collection item API, inferred from collection item type.
    pub fn addItemToCollection(
        self: *PortfolioManager,
        collection_id: u32,
        name: []const u8,
        description: []const u8,
    ) !u32 {
        _ = self.portfolio orelse return error.PortfolioNotFound;
        const collection = self.getCollectionMutable(collection_id) orelse return error.CollectionNotFound;

        const classified = classifyEntityType(collection.item_type);
        return try self.addPortfolioItemToCollection(
            collection_id,
            name,
            description,
            classified.item_type,
            classified.resource_type,
            collection.metadata.category,
        );
    }

    /// Add a list/schedule directory for portfolio items.
    pub fn addDirectory(self: *PortfolioManager, name: []const u8, description: []const u8, category: []const u8) !u32 {
        const portfolio = self.portfolio orelse return error.PortfolioNotFound;

        const id = self.next_id;
        self.next_id += 1;

        var metadata = try self.createMetadata(.directory, .operational, category, portfolio.id);
        errdefer self.deinitMetadata(&metadata);

        const directory_name = try self.allocator.dupe(u8, name);
        errdefer self.allocator.free(directory_name);

        const directory_description = try self.allocator.dupe(u8, description);
        errdefer self.allocator.free(directory_description);

        var directory = PortfolioDirectory{
            .id = id,
            .name = directory_name,
            .description = directory_description,
            .entries = std.array_list.Managed(DirectoryEntry).init(self.allocator),
            .metadata = metadata,
        };
        errdefer self.deinitDirectory(&directory);

        try self.portfolio.?.directories.append(directory);
        try self.addToIndex(id, name, .directory, portfolio.id);
        return id;
    }

    /// Add an item reference to a directory list/schedule.
    pub fn addItemToDirectory(
        self: *PortfolioManager,
        directory_id: u32,
        item_id: u32,
        scheduled_at: ?i64,
        note: []const u8,
        created_at: i64,
    ) !u32 {
        if (self.findPortfolioItemMutable(item_id) == null) return error.ItemNotFound;

        const directory = self.getDirectoryMutable(directory_id) orelse return error.DirectoryNotFound;
        const entry_id = @as(u32, @intCast(directory.entries.items.len));

        const entry = DirectoryEntry{
            .id = entry_id,
            .item_id = item_id,
            .scheduled_at = scheduled_at,
            .note = try self.allocator.dupe(u8, note),
            .created_at = created_at,
            .updated_at = created_at,
        };
        errdefer self.allocator.free(entry.note);

        try directory.entries.append(entry);
        directory.metadata.updated_at = created_at;
        return entry_id;
    }

    /// Attach a container to a portfolio item.
    pub fn addContainerToItem(
        self: *PortfolioManager,
        item_id: u32,
        container_type: ContainerType,
        name: []const u8,
        description: []const u8,
        created_at: i64,
    ) !u32 {
        const item = self.findPortfolioItemMutable(item_id) orelse return error.ItemNotFound;
        const container_id = @as(u32, @intCast(item.containers.items.len));

        const container = ItemContainer{
            .id = container_id,
            .container_type = container_type,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .created_at = created_at,
            .updated_at = created_at,
        };
        errdefer {
            self.allocator.free(container.name);
            self.allocator.free(container.description);
        }

        try item.containers.append(container);
        item.metadata.updated_at = created_at;
        return container_id;
    }

    pub fn getItemContainers(self: *PortfolioManager, item_id: u32) ![]ItemContainer {
        const item = self.findPortfolioItemMutable(item_id) orelse return error.ItemNotFound;
        return item.containers.items;
    }

    pub fn getCollectionCount(self: *PortfolioManager) u32 {
        if (self.portfolio) |portfolio| {
            return @as(u32, @intCast(portfolio.collections.items.len));
        }
        return 0;
    }

    pub fn getDirectoryCount(self: *PortfolioManager) u32 {
        if (self.portfolio) |portfolio| {
            return @as(u32, @intCast(portfolio.directories.items.len));
        }
        return 0;
    }

    pub fn getPortfolioItemCount(self: *PortfolioManager) u32 {
        if (self.portfolio) |portfolio| {
            return @as(u32, @intCast(portfolio.items.items.len));
        }
        return 0;
    }

    pub fn getCollectionById(self: *PortfolioManager, collection_id: u32) ?*PortfolioCollection {
        return self.getCollectionMutable(collection_id);
    }

    pub fn getDirectoryById(self: *PortfolioManager, directory_id: u32) ?*PortfolioDirectory {
        return self.getDirectoryMutable(directory_id);
    }

    pub fn getCollectionsByType(self: *PortfolioManager, item_type: EntityType) !std.array_list.Managed(*PortfolioCollection) {
        var results = std.array_list.Managed(*PortfolioCollection).init(self.allocator);
        if (self.portfolio) |*portfolio| {
            for (portfolio.collections.items) |*collection| {
                if (collection.item_type == item_type) {
                    try results.append(collection);
                }
            }
        }
        return results;
    }

    pub fn addSubPortfolio(self: *PortfolioManager, name: []const u8, description: []const u8, category: []const u8) !void {
        if (self.portfolio == null) return;
        _ = try self.addPortfolioItem(name, description, .sub_portfolio, null, category);
    }

    pub fn addProgram(self: *PortfolioManager, name: []const u8, description: []const u8, category: []const u8) !void {
        if (self.portfolio == null) return;
        _ = try self.addPortfolioItem(name, description, .program, null, category);
    }

    pub fn addProject(self: *PortfolioManager, name: []const u8, description: []const u8, category: []const u8) !void {
        if (self.portfolio == null) return;
        _ = try self.addPortfolioItem(name, description, .project, null, category);
    }

    pub fn addTagToEntity(self: *PortfolioManager, entity_id: u32, tag_name: []const u8, tag_category: []const u8) !void {
        _ = self.portfolio orelse return;
        if (self.findEntityMetadata(entity_id)) |metadata| {
            const tag = Tag{
                .id = @as(u32, @intCast(metadata.tags.items.len)),
                .name = try self.allocator.dupe(u8, tag_name),
                .category = try self.allocator.dupe(u8, tag_category),
            };
            errdefer {
                self.allocator.free(tag.name);
                self.allocator.free(tag.category);
            }

            try metadata.tags.append(tag);
            metadata.updated_at += 1;
        }
    }

    fn findEntityMetadata(self: *PortfolioManager, entity_id: u32) ?*Metadata {
        if (self.portfolio) |*portfolio| {
            if (portfolio.id == entity_id) return &portfolio.metadata;

            for (portfolio.collections.items) |*collection| {
                if (collection.id == entity_id) return &collection.metadata;
                for (collection.items.items) |*item| {
                    if (item.id == entity_id) return &item.metadata;
                }
            }

            for (portfolio.directories.items) |*directory| {
                if (directory.id == entity_id) return &directory.metadata;
            }

            for (portfolio.items.items) |*item| {
                if (item.id == entity_id) return &item.metadata;
            }
        }
        return null;
    }

    fn addToIndex(self: *PortfolioManager, id: u32, name: []const u8, entity_type: EntityType, parent_id: ?u32) !void {
        const entry = IndexEntry{
            .id = id,
            .name = try self.allocator.dupe(u8, name),
            .entity_type = entity_type,
            .parent_id = parent_id,
        };
        try self.index.append(entry);
    }

    pub fn search(self: *PortfolioManager, filter: SearchFilter, allocator: std.mem.Allocator) !std.array_list.Managed(IndexEntry) {
        var results = std.array_list.Managed(IndexEntry).init(allocator);

        for (self.index.items) |entry| {
            var matches = true;

            if (filter.name_pattern) |pattern| {
                if (std.mem.indexOf(u8, entry.name, pattern) == null) matches = false;
            }

            if (matches and filter.entity_type) |etype| {
                if (entry.entity_type != etype) matches = false;
            }

            var metadata: ?*Metadata = null;
            if (matches and (filter.entity_class != null or filter.category != null or filter.owner_id != null or filter.tag_names != null)) {
                metadata = self.findEntityMetadata(entry.id);
                if (metadata == null) matches = false;
            }

            if (matches and filter.entity_class) |entity_class| {
                if (metadata.?.entity_class != entity_class) matches = false;
            }

            if (matches and filter.category) |category| {
                if (!std.mem.eql(u8, metadata.?.category, category)) matches = false;
            }

            if (matches and filter.owner_id) |owner_id| {
                if (metadata.?.owner_id != owner_id) matches = false;
            }

            if (matches and filter.tag_names) |tag_names| {
                for (tag_names.items) |tag_name| {
                    if (!metadataHasTag(metadata.?, tag_name)) {
                        matches = false;
                        break;
                    }
                }
            }

            if (!matches) continue;

            const entry_copy = IndexEntry{
                .id = entry.id,
                .name = try allocator.dupe(u8, entry.name),
                .entity_type = entry.entity_type,
                .parent_id = entry.parent_id,
            };
            errdefer allocator.free(entry_copy.name);
            try results.append(entry_copy);
        }

        return results;
    }

    pub fn getEntityById(self: *PortfolioManager, id: u32) ?IndexEntry {
        for (self.index.items) |entry| {
            if (entry.id == id) return entry;
        }
        return null;
    }

    pub fn getEntitiesByType(self: *PortfolioManager, entity_type: EntityType, allocator: std.mem.Allocator) !std.array_list.Managed(IndexEntry) {
        var results = std.array_list.Managed(IndexEntry).init(allocator);
        for (self.index.items) |entry| {
            if (entry.entity_type != entity_type) continue;

            const entry_copy = IndexEntry{
                .id = entry.id,
                .name = try allocator.dupe(u8, entry.name),
                .entity_type = entry.entity_type,
                .parent_id = entry.parent_id,
            };
            errdefer allocator.free(entry_copy.name);
            try results.append(entry_copy);
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
