//! KOGI Workspace Module - Personal Work Environment in the KOGI OS
//! A Workspace is analogous to a user's home directory and file hierarchy.
//! It organizes all work artifacts, projects, collections, and assets.

const std = @import("std");

/// Expanded entity types to support diverse workspace contents
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

/// Entity classifications for workspace domains
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

/// Tag for flexible workspace organization
pub const Tag = struct {
    id: u32,
    name: []const u8,
    category: []const u8,
};

/// Metadata container for rich workspace item information
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

/// Index entry for fast workspace lookups
pub const IndexEntry = struct {
    id: u32,
    name: []const u8,
    entity_type: EntityType,
    parent_id: ?u32 = null,
};

/// Generic item that can represent any type of work/artifact/product in workspace
pub const WorkspaceItem = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    metadata: Metadata,
};

/// A collection container - can hold any items with a specific purpose
pub const Collection = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    item_type: EntityType,
    items: std.ArrayList(WorkspaceItem),
    metadata: Metadata,
};

/// Workspace is a personal work environment container for independent worker workspaces
pub const Workspace = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    collections: std.ArrayList(Collection),
    items: std.ArrayList(WorkspaceItem),
    metadata: Metadata,
    // Add APIs for managing independent worker workspaces here
};

/// Task represents a work item (legacy support)
pub const Task = struct {
    id: u32,
    title: []const u8,
    description: []const u8,
    completed: bool,
    priority: u8,
    metadata: Metadata,
};

// Project, Program, and asset logic is now managed by the PortfolioManagementApp in src/apps/portfolio_app.zig

/// Placeholder Project/Program/SubWorkspace types for compatibility
pub const Project = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    tasks: std.ArrayList(Task),
    metadata: Metadata,
};

pub const Program = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    projects: std.ArrayList(Project),
    metadata: Metadata,
};

pub const SubWorkspace = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    programs: std.ArrayList(Program),
    projects: std.ArrayList(Project),
    metadata: Metadata,
};

/// Search filter criteria for workspace
pub const SearchFilter = struct {
    name_pattern: ?[]const u8 = null,
    entity_type: ?EntityType = null,
    entity_class: ?EntityClass = null,
    category: ?[]const u8 = null,
    tag_names: ?std.ArrayList([]const u8) = null,
    owner_id: ?u32 = null,
};

/// Workspace Manager for personal work environment operations
pub const WorkspaceManager = struct {
    allocator: std.mem.Allocator,
    workspace: ?Workspace = null,
    index: std.ArrayList(IndexEntry),
    next_id: u32 = 0,

    pub fn init(allocator: std.mem.Allocator) WorkspaceManager {
        return WorkspaceManager{
            .allocator = allocator,
            .index = std.ArrayList(IndexEntry){},
        };
    }

    pub fn deinit(self: *WorkspaceManager) void {
        if (self.workspace) |*workspace| {
            self.deinitWorkspace(workspace);
        }
        self.deinitIndex();
    }

    fn deinitIndex(self: *WorkspaceManager) void {
        for (self.index.items) |*entry| {
            self.allocator.free(entry.name);
        }
        self.index.deinit(self.allocator);
    }

    fn deinitMetadata(self: *WorkspaceManager, metadata: *Metadata) void {
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

    fn deinitWorkspaceItem(self: *WorkspaceManager, item: *WorkspaceItem) void {
        self.allocator.free(item.name);
        self.allocator.free(item.description);
        self.deinitMetadata(&item.metadata);
    }

    fn deinitCollection(self: *WorkspaceManager, collection: *Collection) void {
        for (collection.items.items) |*item| {
            self.deinitWorkspaceItem(item);
        }
        collection.items.deinit(self.allocator);
        self.allocator.free(collection.name);
        self.allocator.free(collection.description);
        self.deinitMetadata(&collection.metadata);
    }

    fn deinitTask(self: *WorkspaceManager, task: *Task) void {
        self.allocator.free(task.title);
        self.allocator.free(task.description);
        self.deinitMetadata(&task.metadata);
    }

    fn deinitProject(self: *WorkspaceManager, project: *Project) void {
        for (project.tasks.items) |*task| {
            self.deinitTask(task);
        }
        project.tasks.deinit(self.allocator);
        self.allocator.free(project.name);
        self.allocator.free(project.description);
        self.deinitMetadata(&project.metadata);
    }

    fn deinitProgram(self: *WorkspaceManager, program: *Program) void {
        for (program.projects.items) |*project| {
            self.deinitProject(project);
        }
        program.projects.deinit(self.allocator);
        self.allocator.free(program.name);
        self.allocator.free(program.description);
        self.deinitMetadata(&program.metadata);
    }

    fn deinitSubWorkspace(self: *WorkspaceManager, sub_workspace: *SubWorkspace) void {
        for (sub_workspace.programs.items) |*program| {
            self.deinitProgram(program);
        }
        sub_workspace.programs.deinit(self.allocator);
        for (sub_workspace.projects.items) |*project| {
            self.deinitProject(project);
        }
        sub_workspace.projects.deinit(self.allocator);
        self.allocator.free(sub_workspace.name);
        self.allocator.free(sub_workspace.description);
        self.deinitMetadata(&sub_workspace.metadata);
    }

    fn deinitWorkspace(self: *WorkspaceManager, workspace: *Workspace) void {
        for (workspace.collections.items) |*col| {
            self.deinitCollection(col);
        }
        workspace.collections.deinit(self.allocator);
        for (workspace.items.items) |*item| {
            self.deinitWorkspaceItem(item);
        }
        workspace.items.deinit(self.allocator);
        self.allocator.free(workspace.name);
        self.allocator.free(workspace.description);
        self.deinitMetadata(&workspace.metadata);
    }

    /// Create a new workspace
    pub fn createWorkspace(self: *WorkspaceManager, name: []const u8, description: []const u8, category: []const u8) !void {
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

        const workspace = Workspace{
            .id = id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .collections = std.ArrayList(Collection){},
            .items = std.ArrayList(WorkspaceItem){},
            .metadata = metadata,
        };

        self.workspace = workspace;
        try self.addToIndex(id, name, .portfolio, null);
    }

    /// Add a collection to the workspace
    pub fn addCollection(
        self: *WorkspaceManager,
        name: []const u8,
        description: []const u8,
        item_type: EntityType,
        category: []const u8,
    ) !u32 {
        if (self.workspace == null) return error.WorkspaceNotFound;

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
            .parent_id = self.workspace.?.id,
        };

        const collection = Collection{
            .id = id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .item_type = item_type,
            .items = std.ArrayList(WorkspaceItem){},
            .metadata = metadata,
        };

        try self.workspace.?.collections.append(self.allocator, collection);
        try self.addToIndex(id, name, item_type, self.workspace.?.id);
        return id;
    }

    /// Add an item to a collection
    pub fn addItemToCollection(
        self: *WorkspaceManager,
        collection_id: u32,
        name: []const u8,
        description: []const u8,
    ) !u32 {
        if (self.workspace == null) return error.WorkspaceNotFound;

        for (self.workspace.?.collections.items) |*collection| {
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

                const item = WorkspaceItem{
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

    /// Add a top-level item to the workspace
    pub fn addItem(
        self: *WorkspaceManager,
        name: []const u8,
        description: []const u8,
        item_type: EntityType,
        category: []const u8,
    ) !u32 {
        if (self.workspace == null) return error.WorkspaceNotFound;

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
            .parent_id = self.workspace.?.id,
        };

        const item = WorkspaceItem{
            .id = id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .metadata = metadata,
        };

        try self.workspace.?.items.append(self.allocator, item);
        try self.addToIndex(id, name, item_type, self.workspace.?.id);
        return id;
    }

    pub fn getCollectionCount(self: *WorkspaceManager) u32 {
        if (self.workspace) |w| {
            return @as(u32, @intCast(w.collections.items.len));
        }
        return 0;
    }

    pub fn getWorkspaceItemCount(self: *WorkspaceManager) u32 {
        if (self.workspace) |w| {
            return @as(u32, @intCast(w.items.items.len));
        }
        return 0;
    }

    pub fn getCollectionById(self: *WorkspaceManager, collection_id: u32) ?*Collection {
        if (self.workspace) |*w| {
            for (w.collections.items) |*col| {
                if (col.id == collection_id) return col;
            }
        }
        return null;
    }

    pub fn getCollectionsByType(self: *WorkspaceManager, item_type: EntityType, allocator: std.mem.Allocator) !std.ArrayList(*Collection) {
        var results = std.ArrayList(*Collection){};
        if (self.workspace) |*w| {
            for (w.collections.items) |*col| {
                if (col.item_type == item_type) {
                    try results.append(&allocator, col);
                }
            }
        }
        return results;
    }

    pub fn addSubWorkspace(self: *WorkspaceManager, name: []const u8, description: []const u8, category: []const u8) !void {
        if (self.workspace == null) return;
        _ = try self.addItem(name, description, .portfolio, category);
    }

    pub fn addProgram(self: *WorkspaceManager, name: []const u8, description: []const u8, category: []const u8) !void {
        if (self.workspace == null) return;
        _ = try self.addCollection(name, description, .program, category);
    }

    pub fn addProject(self: *WorkspaceManager, name: []const u8, description: []const u8, category: []const u8) !void {
        if (self.workspace == null) return;
        _ = try self.addCollection(name, description, .project, category);
    }

    pub fn addTagToEntity(self: *WorkspaceManager, entity_id: u32, tag_name: []const u8, tag_category: []const u8) !void {
        if (self.workspace == null) return;
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

    fn findEntityMetadata(self: *WorkspaceManager, entity_id: u32) ?*Metadata {
        if (self.workspace) |*workspace| {
            if (workspace.id == entity_id) return &workspace.metadata;

            for (workspace.collections.items) |*col| {
                if (col.id == entity_id) return &col.metadata;
                for (col.items.items) |*item| {
                    if (item.id == entity_id) return &item.metadata;
                }
            }

            for (workspace.items.items) |*item| {
                if (item.id == entity_id) return &item.metadata;
            }
        }
        return null;
    }

    fn addToIndex(self: *WorkspaceManager, id: u32, name: []const u8, entity_type: EntityType, parent_id: ?u32) !void {
        const entry = IndexEntry{
            .id = id,
            .name = try self.allocator.dupe(u8, name),
            .entity_type = entity_type,
            .parent_id = parent_id,
        };
        try self.index.append(self.allocator, entry);
    }

    pub fn search(self: *WorkspaceManager, filter: SearchFilter, allocator: std.mem.Allocator) !std.ArrayList(IndexEntry) {
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

    pub fn getEntityById(self: *WorkspaceManager, id: u32) ?IndexEntry {
        for (self.index.items) |entry| {
            if (entry.id == id) {
                return entry;
            }
        }
        return null;
    }

    pub fn getEntitiesByType(self: *WorkspaceManager, entity_type: EntityType, allocator: std.mem.Allocator) !std.ArrayList(IndexEntry) {
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

// // Backward compatibility aliases
// pub const Portfolio = Workspace;
// pub const PortfolioManager = WorkspaceManager;
// pub const PortfolioItem = WorkspaceItem;
// pub const PortfolioCollection = Collection;
// pub const SubPortfolio = SubWorkspace;
