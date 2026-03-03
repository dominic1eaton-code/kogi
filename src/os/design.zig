const std = @import("std");

pub const DesignKind = enum {
    idea,
    concept,
    prototype,
    @"test",
    testbed,
    design,
    design_artifact,
    blueprint,
    mockup,
    mindmap,
    diagram,
    chart,
    notes,
    design_document,
    design_content,
    custom,
};

pub const DesignStatus = enum {
    draft,
    in_review,
    approved,
    rejected,
    archived,
};

pub const DesignMetadata = struct {
    key: []const u8,
    value: []const u8,
};

pub const DesignRevision = struct {
    id: u32,
    design_id: u32,
    version: u32,
    summary: []const u8,
    content: []const u8,
    created_by_user_id: ?u32,
    created_at: i64,
};

pub const DesignItem = struct {
    id: u32,
    owner_user_id: ?u32,
    profile_id: ?u32,
    linked_content_id: ?u32,
    kind: DesignKind,
    status: DesignStatus,
    title: []const u8,
    description: []const u8,
    tags: std.ArrayList([]const u8),
    metadata: std.ArrayList(DesignMetadata),
    current_revision_id: ?u32,
    active: bool,
    created_at: i64,
    updated_at: i64,
};

pub const DesignError = error{
    DesignNotFound,
    RevisionNotFound,
};

pub const DesignManager = struct {
    allocator: std.mem.Allocator,
    designs: std.ArrayList(DesignItem),
    revisions: std.ArrayList(DesignRevision),
    next_design_id: u32,
    next_revision_id: u32,

    pub fn init(allocator: std.mem.Allocator) DesignManager {
        return DesignManager{
            .allocator = allocator,
            .designs = std.ArrayList(DesignItem).init(allocator),
            .revisions = std.ArrayList(DesignRevision).init(allocator),
            .next_design_id = 0,
            .next_revision_id = 0,
        };
    }

    pub fn deinit(self: *DesignManager) void {
        for (self.designs.items) |*design| {
            self.allocator.free(design.title);
            self.allocator.free(design.description);
            for (design.tags.items) |tag| self.allocator.free(tag);
            for (design.metadata.items) |entry| {
                self.allocator.free(entry.key);
                self.allocator.free(entry.value);
            }
            design.tags.deinit();
            design.metadata.deinit();
        }
        self.designs.deinit();

        for (self.revisions.items) |rev| {
            self.allocator.free(rev.summary);
            self.allocator.free(rev.content);
        }
        self.revisions.deinit();
    }

    pub fn createDesign(
        self: *DesignManager,
        owner_user_id: ?u32,
        profile_id: ?u32,
        linked_content_id: ?u32,
        kind: DesignKind,
        title: []const u8,
        description: []const u8,
        initial_revision_summary: []const u8,
        initial_revision_content: []const u8,
        created_by_user_id: ?u32,
    ) !u32 {
        const design_id = self.next_design_id;
        self.next_design_id += 1;

        const now = std.time.timestamp();
        const design = DesignItem{
            .id = design_id,
            .owner_user_id = owner_user_id,
            .profile_id = profile_id,
            .linked_content_id = linked_content_id,
            .kind = kind,
            .status = .draft,
            .title = try self.allocator.dupe(u8, title),
            .description = try self.allocator.dupe(u8, description),
            .tags = std.ArrayList([]const u8).init(self.allocator),
            .metadata = std.ArrayList(DesignMetadata).init(self.allocator),
            .current_revision_id = null,
            .active = true,
            .created_at = now,
            .updated_at = now,
        };
        errdefer {
            self.allocator.free(design.title);
            self.allocator.free(design.description);
            var tags = design.tags;
            var metadata = design.metadata;
            tags.deinit();
            metadata.deinit();
        }
        try self.designs.append(design);

        const rev_id = try self.addRevision(
            design_id,
            initial_revision_summary,
            initial_revision_content,
            created_by_user_id,
        );
        const idx = self.findDesignIndexById(design_id).?;
        self.designs.items[idx].current_revision_id = rev_id;
        return design_id;
    }

    pub fn addRevision(
        self: *DesignManager,
        design_id: u32,
        summary: []const u8,
        content: []const u8,
        created_by_user_id: ?u32,
    ) !u32 {
        const design_idx = self.findDesignIndexById(design_id) orelse return DesignError.DesignNotFound;
        const revision_id = self.next_revision_id;
        self.next_revision_id += 1;

        const revision = DesignRevision{
            .id = revision_id,
            .design_id = design_id,
            .version = self.nextVersionForDesign(design_id),
            .summary = try self.allocator.dupe(u8, summary),
            .content = try self.allocator.dupe(u8, content),
            .created_by_user_id = created_by_user_id,
            .created_at = std.time.timestamp(),
        };
        errdefer {
            self.allocator.free(revision.summary);
            self.allocator.free(revision.content);
        }
        try self.revisions.append(revision);
        self.designs.items[design_idx].current_revision_id = revision_id;
        self.designs.items[design_idx].updated_at = std.time.timestamp();
        return revision_id;
    }

    pub fn setStatus(self: *DesignManager, design_id: u32, status: DesignStatus) !void {
        const idx = self.findDesignIndexById(design_id) orelse return DesignError.DesignNotFound;
        self.designs.items[idx].status = status;
        self.designs.items[idx].updated_at = std.time.timestamp();
    }

    pub fn rename(self: *DesignManager, design_id: u32, title: []const u8) !void {
        const idx = self.findDesignIndexById(design_id) orelse return DesignError.DesignNotFound;
        self.allocator.free(self.designs.items[idx].title);
        self.designs.items[idx].title = try self.allocator.dupe(u8, title);
        self.designs.items[idx].updated_at = std.time.timestamp();
    }

    pub fn updateDescription(self: *DesignManager, design_id: u32, description: []const u8) !void {
        const idx = self.findDesignIndexById(design_id) orelse return DesignError.DesignNotFound;
        self.allocator.free(self.designs.items[idx].description);
        self.designs.items[idx].description = try self.allocator.dupe(u8, description);
        self.designs.items[idx].updated_at = std.time.timestamp();
    }

    pub fn setLinkedContent(self: *DesignManager, design_id: u32, linked_content_id: ?u32) !void {
        const idx = self.findDesignIndexById(design_id) orelse return DesignError.DesignNotFound;
        self.designs.items[idx].linked_content_id = linked_content_id;
        self.designs.items[idx].updated_at = std.time.timestamp();
    }

    pub fn addTag(self: *DesignManager, design_id: u32, tag: []const u8) !void {
        const idx = self.findDesignIndexById(design_id) orelse return DesignError.DesignNotFound;
        for (self.designs.items[idx].tags.items) |existing| {
            if (std.mem.eql(u8, existing, tag)) return;
        }
        try self.designs.items[idx].tags.append(try self.allocator.dupe(u8, tag));
        self.designs.items[idx].updated_at = std.time.timestamp();
    }

    pub fn setMetadata(self: *DesignManager, design_id: u32, key: []const u8, value: []const u8) !void {
        const idx = self.findDesignIndexById(design_id) orelse return DesignError.DesignNotFound;
        for (self.designs.items[idx].metadata.items) |*entry| {
            if (std.mem.eql(u8, entry.key, key)) {
                self.allocator.free(entry.value);
                entry.value = try self.allocator.dupe(u8, value);
                self.designs.items[idx].updated_at = std.time.timestamp();
                return;
            }
        }
        try self.designs.items[idx].metadata.append(.{
            .key = try self.allocator.dupe(u8, key),
            .value = try self.allocator.dupe(u8, value),
        });
        self.designs.items[idx].updated_at = std.time.timestamp();
    }

    pub fn deactivateDesign(self: *DesignManager, design_id: u32) !void {
        const idx = self.findDesignIndexById(design_id) orelse return DesignError.DesignNotFound;
        self.designs.items[idx].active = false;
        self.designs.items[idx].updated_at = std.time.timestamp();
    }

    pub fn getDesignById(self: *DesignManager, design_id: u32) !DesignItem {
        const idx = self.findDesignIndexById(design_id) orelse return DesignError.DesignNotFound;
        return self.designs.items[idx];
    }

    pub fn getDesigns(self: *DesignManager) []DesignItem {
        return self.designs.items;
    }

    pub fn getRevisionById(self: *DesignManager, revision_id: u32) !DesignRevision {
        const idx = self.findRevisionIndexById(revision_id) orelse return DesignError.RevisionNotFound;
        return self.revisions.items[idx];
    }

    pub fn getLatestRevision(self: *DesignManager, design_id: u32) !DesignRevision {
        _ = self.findDesignIndexById(design_id) orelse return DesignError.DesignNotFound;
        var latest: ?DesignRevision = null;
        for (self.revisions.items) |rev| {
            if (rev.design_id != design_id) continue;
            if (latest == null or rev.version > latest.?.version) latest = rev;
        }
        return latest orelse DesignError.RevisionNotFound;
    }

    pub fn getRevisionsForDesign(self: *DesignManager, design_id: u32, allocator: std.mem.Allocator) !std.ArrayList(DesignRevision) {
        _ = self.findDesignIndexById(design_id) orelse return DesignError.DesignNotFound;
        var results = std.ArrayList(DesignRevision).init(allocator);
        for (self.revisions.items) |rev| {
            if (rev.design_id == design_id) {
                try results.append(rev);
            }
        }
        return results;
    }

    pub fn countByKind(self: *DesignManager, kind: DesignKind) u32 {
        var count: u32 = 0;
        for (self.designs.items) |design| {
            if (design.kind == kind and design.active) count += 1;
        }
        return count;
    }

    fn nextVersionForDesign(self: *DesignManager, design_id: u32) u32 {
        var max_version: u32 = 0;
        for (self.revisions.items) |rev| {
            if (rev.design_id == design_id and rev.version > max_version) max_version = rev.version;
        }
        return max_version + 1;
    }

    fn findDesignIndexById(self: *DesignManager, design_id: u32) ?usize {
        for (self.designs.items, 0..) |design, idx| {
            if (design.id == design_id) return idx;
        }
        return null;
    }

    fn findRevisionIndexById(self: *DesignManager, revision_id: u32) ?usize {
        for (self.revisions.items, 0..) |revision, idx| {
            if (revision.id == revision_id) return idx;
        }
        return null;
    }
};

pub fn parseDesignKind(name: []const u8) ?DesignKind {
    if (std.mem.eql(u8, name, "idea")) return .idea;
    if (std.mem.eql(u8, name, "concept")) return .concept;
    if (std.mem.eql(u8, name, "prototype")) return .prototype;
    if (std.mem.eql(u8, name, "test")) return .@"test";
    if (std.mem.eql(u8, name, "testbed")) return .testbed;
    if (std.mem.eql(u8, name, "design")) return .design;
    if (std.mem.eql(u8, name, "design_artifact")) return .design_artifact;
    if (std.mem.eql(u8, name, "blueprint")) return .blueprint;
    if (std.mem.eql(u8, name, "mockup")) return .mockup;
    if (std.mem.eql(u8, name, "mindmap")) return .mindmap;
    if (std.mem.eql(u8, name, "diagram")) return .diagram;
    if (std.mem.eql(u8, name, "chart")) return .chart;
    if (std.mem.eql(u8, name, "design_document")) return .design_document;
    if (std.mem.eql(u8, name, "design_content")) return .design_content;
    if (std.mem.eql(u8, name, "custom")) return .custom;
    return null;
}

pub fn parseDesignStatus(name: []const u8) ?DesignStatus {
    if (std.mem.eql(u8, name, "draft")) return .draft;
    if (std.mem.eql(u8, name, "in_review")) return .in_review;
    if (std.mem.eql(u8, name, "approved")) return .approved;
    if (std.mem.eql(u8, name, "rejected")) return .rejected;
    if (std.mem.eql(u8, name, "archived")) return .archived;
    return null;
}

test "design management lifecycle and versioning" {
    const allocator = std.testing.allocator;
    var mgr = DesignManager.init(allocator);
    defer mgr.deinit();

    const design_id = try mgr.createDesign(
        1,
        2,
        9,
        .prototype,
        "Search UX Revamp",
        "Main design initiative",
        "initial draft",
        "v1 content",
        1,
    );
    try mgr.addTag(design_id, "ux");
    try mgr.setMetadata(design_id, "team", "product-design");
    const rev2 = try mgr.addRevision(design_id, "feedback applied", "v2 content", 1);
    try mgr.setStatus(design_id, .in_review);

    const design = try mgr.getDesignById(design_id);
    try std.testing.expectEqual(@as(usize, 1), design.tags.items.len);
    try std.testing.expectEqual(DesignStatus.in_review, design.status);
    try std.testing.expectEqual(@as(u32, 1), mgr.countByKind(.prototype));

    const latest = try mgr.getLatestRevision(design_id);
    try std.testing.expectEqual(rev2, latest.id);
    try std.testing.expectEqual(@as(u32, 2), latest.version);

    var revs = try mgr.getRevisionsForDesign(design_id, allocator);
    defer revs.deinit();
    try std.testing.expectEqual(@as(usize, 2), revs.items.len);
}
