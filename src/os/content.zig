const std = @import("std");

pub const ContentKind = enum {
    file,
    directory,
    folder,
    data,
    info,
    knowledge,
    wisdom,
    artifact,
    asset,
    custom,
};

pub const ContentMetadata = struct {
    key: []const u8,
    value: []const u8,
};

pub const NoteImportance = enum {
    low,
    normal,
    high,
    critical,
};

pub const Note = struct {
    id: u32,
    owner_user_id: ?u32,
    profile_id: ?u32,
    linked_content_id: ?u32,
    title: []const u8,
    body: []const u8,
    importance: NoteImportance,
    tags: std.ArrayList([]const u8),
    pinned: bool,
    archived: bool,
    created_at: i64,
    updated_at: i64,
};

pub const ContentItem = struct {
    id: u32,
    owner_user_id: ?u32,
    profile_id: ?u32,
    parent_id: ?u32,
    kind: ContentKind,
    title: []const u8,
    body: []const u8,
    tags: std.ArrayList([]const u8),
    metadata: std.ArrayList(ContentMetadata),
    active: bool,
    created_at: i64,
    updated_at: i64,
};

pub const ContentError = error{
    ContentNotFound,
    NoteNotFound,
    ParentNotFound,
    InvalidParent,
};

pub const ContentManager = struct {
    allocator: std.mem.Allocator,
    items: std.ArrayList(ContentItem),
    notes: std.ArrayList(Note),
    next_id: u32,
    next_note_id: u32,

    pub fn init(allocator: std.mem.Allocator) ContentManager {
        return ContentManager{
            .allocator = allocator,
            .items = std.ArrayList(ContentItem).init(allocator),
            .notes = std.ArrayList(Note).init(allocator),
            .next_id = 0,
            .next_note_id = 0,
        };
    }

    pub fn deinit(self: *ContentManager) void {
        for (self.items.items) |*item| {
            self.allocator.free(item.title);
            self.allocator.free(item.body);
            for (item.tags.items) |tag| self.allocator.free(tag);
            for (item.metadata.items) |entry| {
                self.allocator.free(entry.key);
                self.allocator.free(entry.value);
            }
            item.tags.deinit();
            item.metadata.deinit();
        }
        self.items.deinit();

        for (self.notes.items) |*note| {
            self.allocator.free(note.title);
            self.allocator.free(note.body);
            for (note.tags.items) |tag| self.allocator.free(tag);
            note.tags.deinit();
        }
        self.notes.deinit();
    }

    pub fn createContent(
        self: *ContentManager,
        owner_user_id: ?u32,
        profile_id: ?u32,
        parent_id: ?u32,
        kind: ContentKind,
        title: []const u8,
        body: []const u8,
    ) !u32 {
        if (parent_id) |pid| {
            const parent = try self.getContentById(pid);
            if (!isContainerKind(parent.kind)) return ContentError.InvalidParent;
        }

        const id = self.next_id;
        self.next_id += 1;
        const now = std.time.timestamp();
        const item = ContentItem{
            .id = id,
            .owner_user_id = owner_user_id,
            .profile_id = profile_id,
            .parent_id = parent_id,
            .kind = kind,
            .title = try self.allocator.dupe(u8, title),
            .body = try self.allocator.dupe(u8, body),
            .tags = std.ArrayList([]const u8).init(self.allocator),
            .metadata = std.ArrayList(ContentMetadata).init(self.allocator),
            .active = true,
            .created_at = now,
            .updated_at = now,
        };
        errdefer {
            self.allocator.free(item.title);
            self.allocator.free(item.body);
            var tags = item.tags;
            var metadata = item.metadata;
            tags.deinit();
            metadata.deinit();
        }
        try self.items.append(item);
        return id;
    }

    pub fn createFile(
        self: *ContentManager,
        owner_user_id: ?u32,
        profile_id: ?u32,
        parent_id: ?u32,
        title: []const u8,
        body: []const u8,
    ) !u32 {
        return try self.createContent(owner_user_id, profile_id, parent_id, .file, title, body);
    }

    pub fn createDirectory(
        self: *ContentManager,
        owner_user_id: ?u32,
        profile_id: ?u32,
        parent_id: ?u32,
        title: []const u8,
    ) !u32 {
        return try self.createContent(owner_user_id, profile_id, parent_id, .directory, title, "");
    }

    pub fn createFolder(
        self: *ContentManager,
        owner_user_id: ?u32,
        profile_id: ?u32,
        parent_id: ?u32,
        title: []const u8,
    ) !u32 {
        return try self.createContent(owner_user_id, profile_id, parent_id, .folder, title, "");
    }

    pub fn createData(
        self: *ContentManager,
        owner_user_id: ?u32,
        profile_id: ?u32,
        parent_id: ?u32,
        title: []const u8,
        body: []const u8,
    ) !u32 {
        return try self.createContent(owner_user_id, profile_id, parent_id, .data, title, body);
    }

    pub fn createInfo(
        self: *ContentManager,
        owner_user_id: ?u32,
        profile_id: ?u32,
        parent_id: ?u32,
        title: []const u8,
        body: []const u8,
    ) !u32 {
        return try self.createContent(owner_user_id, profile_id, parent_id, .info, title, body);
    }

    pub fn createKnowledge(
        self: *ContentManager,
        owner_user_id: ?u32,
        profile_id: ?u32,
        parent_id: ?u32,
        title: []const u8,
        body: []const u8,
    ) !u32 {
        return try self.createContent(owner_user_id, profile_id, parent_id, .knowledge, title, body);
    }

    pub fn createWisdom(
        self: *ContentManager,
        owner_user_id: ?u32,
        profile_id: ?u32,
        parent_id: ?u32,
        title: []const u8,
        body: []const u8,
    ) !u32 {
        return try self.createContent(owner_user_id, profile_id, parent_id, .wisdom, title, body);
    }

    pub fn createArtifact(
        self: *ContentManager,
        owner_user_id: ?u32,
        profile_id: ?u32,
        parent_id: ?u32,
        title: []const u8,
        body: []const u8,
    ) !u32 {
        return try self.createContent(owner_user_id, profile_id, parent_id, .artifact, title, body);
    }

    pub fn createAsset(
        self: *ContentManager,
        owner_user_id: ?u32,
        profile_id: ?u32,
        parent_id: ?u32,
        title: []const u8,
        body: []const u8,
    ) !u32 {
        return try self.createContent(owner_user_id, profile_id, parent_id, .asset, title, body);
    }

    // ========== Note Management ==========

    pub fn createNote(
        self: *ContentManager,
        owner_user_id: ?u32,
        profile_id: ?u32,
        linked_content_id: ?u32,
        title: []const u8,
        body: []const u8,
        importance: NoteImportance,
    ) !u32 {
        if (linked_content_id) |cid| {
            _ = try self.getContentById(cid);
        }
        const note_id = self.next_note_id;
        self.next_note_id += 1;
        const now = std.time.timestamp();
        const note = Note{
            .id = note_id,
            .owner_user_id = owner_user_id,
            .profile_id = profile_id,
            .linked_content_id = linked_content_id,
            .title = try self.allocator.dupe(u8, title),
            .body = try self.allocator.dupe(u8, body),
            .importance = importance,
            .tags = std.ArrayList([]const u8).init(self.allocator),
            .pinned = false,
            .archived = false,
            .created_at = now,
            .updated_at = now,
        };
        errdefer {
            self.allocator.free(note.title);
            self.allocator.free(note.body);
            var tags = note.tags;
            tags.deinit();
        }
        try self.notes.append(note);
        return note_id;
    }

    pub fn updateNote(self: *ContentManager, note_id: u32, title: []const u8, body: []const u8, importance: NoteImportance) !void {
        const idx = self.findNoteIndexById(note_id) orelse return ContentError.NoteNotFound;
        const note = &self.notes.items[idx];
        self.allocator.free(note.title);
        self.allocator.free(note.body);
        note.title = try self.allocator.dupe(u8, title);
        note.body = try self.allocator.dupe(u8, body);
        note.importance = importance;
        note.updated_at = std.time.timestamp();
    }

    pub fn addNoteTag(self: *ContentManager, note_id: u32, tag: []const u8) !void {
        const idx = self.findNoteIndexById(note_id) orelse return ContentError.NoteNotFound;
        for (self.notes.items[idx].tags.items) |existing| {
            if (std.mem.eql(u8, existing, tag)) return;
        }
        try self.notes.items[idx].tags.append(try self.allocator.dupe(u8, tag));
        self.notes.items[idx].updated_at = std.time.timestamp();
    }

    pub fn pinNote(self: *ContentManager, note_id: u32) !void {
        const idx = self.findNoteIndexById(note_id) orelse return ContentError.NoteNotFound;
        self.notes.items[idx].pinned = true;
        self.notes.items[idx].updated_at = std.time.timestamp();
    }

    pub fn unpinNote(self: *ContentManager, note_id: u32) !void {
        const idx = self.findNoteIndexById(note_id) orelse return ContentError.NoteNotFound;
        self.notes.items[idx].pinned = false;
        self.notes.items[idx].updated_at = std.time.timestamp();
    }

    pub fn archiveNote(self: *ContentManager, note_id: u32) !void {
        const idx = self.findNoteIndexById(note_id) orelse return ContentError.NoteNotFound;
        self.notes.items[idx].archived = true;
        self.notes.items[idx].updated_at = std.time.timestamp();
    }

    pub fn unarchiveNote(self: *ContentManager, note_id: u32) !void {
        const idx = self.findNoteIndexById(note_id) orelse return ContentError.NoteNotFound;
        self.notes.items[idx].archived = false;
        self.notes.items[idx].updated_at = std.time.timestamp();
    }

    pub fn setNoteLinkedContent(self: *ContentManager, note_id: u32, linked_content_id: ?u32) !void {
        const idx = self.findNoteIndexById(note_id) orelse return ContentError.NoteNotFound;
        if (linked_content_id) |cid| {
            _ = try self.getContentById(cid);
        }
        self.notes.items[idx].linked_content_id = linked_content_id;
        self.notes.items[idx].updated_at = std.time.timestamp();
    }

    pub fn getNoteById(self: *ContentManager, note_id: u32) !Note {
        const idx = self.findNoteIndexById(note_id) orelse return ContentError.NoteNotFound;
        return self.notes.items[idx];
    }

    pub fn getNotes(self: *ContentManager) []Note {
        return self.notes.items;
    }

    pub fn getNotesForContent(self: *ContentManager, content_id: u32, allocator: std.mem.Allocator) !std.ArrayList(Note) {
        _ = try self.getContentById(content_id);
        var results = std.ArrayList(Note).init(allocator);
        for (self.notes.items) |note| {
            if (note.linked_content_id != null and note.linked_content_id.? == content_id and !note.archived) {
                try results.append(note);
            }
        }
        return results;
    }

    pub fn updateBody(self: *ContentManager, content_id: u32, body: []const u8) !void {
        const idx = self.findIndexById(content_id) orelse return ContentError.ContentNotFound;
        self.allocator.free(self.items.items[idx].body);
        self.items.items[idx].body = try self.allocator.dupe(u8, body);
        self.items.items[idx].updated_at = std.time.timestamp();
    }

    pub fn rename(self: *ContentManager, content_id: u32, title: []const u8) !void {
        const idx = self.findIndexById(content_id) orelse return ContentError.ContentNotFound;
        self.allocator.free(self.items.items[idx].title);
        self.items.items[idx].title = try self.allocator.dupe(u8, title);
        self.items.items[idx].updated_at = std.time.timestamp();
    }

    pub fn moveContent(self: *ContentManager, content_id: u32, new_parent_id: ?u32) !void {
        const idx = self.findIndexById(content_id) orelse return ContentError.ContentNotFound;
        if (new_parent_id) |pid| {
            const parent = try self.getContentById(pid);
            if (!isContainerKind(parent.kind)) return ContentError.InvalidParent;
            if (pid == content_id) return ContentError.InvalidParent;
        }
        self.items.items[idx].parent_id = new_parent_id;
        self.items.items[idx].updated_at = std.time.timestamp();
    }

    pub fn addTag(self: *ContentManager, content_id: u32, tag: []const u8) !void {
        const idx = self.findIndexById(content_id) orelse return ContentError.ContentNotFound;
        for (self.items.items[idx].tags.items) |existing| {
            if (std.mem.eql(u8, existing, tag)) return;
        }
        try self.items.items[idx].tags.append(try self.allocator.dupe(u8, tag));
        self.items.items[idx].updated_at = std.time.timestamp();
    }

    pub fn setMetadata(self: *ContentManager, content_id: u32, key: []const u8, value: []const u8) !void {
        const idx = self.findIndexById(content_id) orelse return ContentError.ContentNotFound;
        for (self.items.items[idx].metadata.items) |*entry| {
            if (std.mem.eql(u8, entry.key, key)) {
                self.allocator.free(entry.value);
                entry.value = try self.allocator.dupe(u8, value);
                self.items.items[idx].updated_at = std.time.timestamp();
                return;
            }
        }
        try self.items.items[idx].metadata.append(.{
            .key = try self.allocator.dupe(u8, key),
            .value = try self.allocator.dupe(u8, value),
        });
        self.items.items[idx].updated_at = std.time.timestamp();
    }

    pub fn deactivateContent(self: *ContentManager, content_id: u32) !void {
        const idx = self.findIndexById(content_id) orelse return ContentError.ContentNotFound;
        self.items.items[idx].active = false;
        self.items.items[idx].updated_at = std.time.timestamp();
    }

    pub fn getContentById(self: *ContentManager, content_id: u32) !ContentItem {
        const idx = self.findIndexById(content_id) orelse return ContentError.ContentNotFound;
        return self.items.items[idx];
    }

    pub fn getItems(self: *ContentManager) []ContentItem {
        return self.items.items;
    }

    pub fn getChildren(self: *ContentManager, parent_id: u32, allocator: std.mem.Allocator) !std.ArrayList(ContentItem) {
        var results = std.ArrayList(ContentItem).init(allocator);
        for (self.items.items) |item| {
            if (item.parent_id != null and item.parent_id.? == parent_id) {
                try results.append(item);
            }
        }
        return results;
    }

    pub fn countByKind(self: *ContentManager, kind: ContentKind) u32 {
        var count: u32 = 0;
        for (self.items.items) |item| {
            if (item.kind == kind and item.active) count += 1;
        }
        return count;
    }

    fn findIndexById(self: *ContentManager, content_id: u32) ?usize {
        for (self.items.items, 0..) |item, idx| {
            if (item.id == content_id) return idx;
        }
        return null;
    }

    fn findNoteIndexById(self: *ContentManager, note_id: u32) ?usize {
        for (self.notes.items, 0..) |note, idx| {
            if (note.id == note_id) return idx;
        }
        return null;
    }
};

fn isContainerKind(kind: ContentKind) bool {
    return switch (kind) {
        .directory, .folder => true,
        else => false,
    };
}

pub fn parseContentKind(name: []const u8) ?ContentKind {
    if (std.mem.eql(u8, name, "file")) return .file;
    if (std.mem.eql(u8, name, "directory")) return .directory;
    if (std.mem.eql(u8, name, "folder")) return .folder;
    if (std.mem.eql(u8, name, "data")) return .data;
    if (std.mem.eql(u8, name, "info")) return .info;
    if (std.mem.eql(u8, name, "knowledge")) return .knowledge;
    if (std.mem.eql(u8, name, "wisdom")) return .wisdom;
    if (std.mem.eql(u8, name, "artifact")) return .artifact;
    if (std.mem.eql(u8, name, "asset")) return .asset;
    if (std.mem.eql(u8, name, "custom")) return .custom;
    return null;
}

pub fn parseNoteImportance(name: []const u8) ?NoteImportance {
    if (std.mem.eql(u8, name, "low")) return .low;
    if (std.mem.eql(u8, name, "normal")) return .normal;
    if (std.mem.eql(u8, name, "high")) return .high;
    if (std.mem.eql(u8, name, "critical")) return .critical;
    return null;
}

test "content management lifecycle" {
    const allocator = std.testing.allocator;
    var mgr = ContentManager.init(allocator);
    defer mgr.deinit();

    const root_dir = try mgr.createDirectory(1, 2, null, "root");
    const docs = try mgr.createFolder(1, 2, root_dir, "docs");
    const file_id = try mgr.createFile(1, 2, docs, "readme.md", "hello");
    _ = try mgr.createData(1, 2, docs, "dataset", "{\"a\":1}");
    _ = try mgr.createInfo(1, 2, docs, "brief", "facts");
    _ = try mgr.createKnowledge(1, 2, docs, "playbook", "how-to");
    _ = try mgr.createWisdom(1, 2, docs, "lessons", "why");
    _ = try mgr.createArtifact(1, 2, docs, "design-v1", "artifact body");
    _ = try mgr.createAsset(1, 2, docs, "logo", "svg bytes");

    try mgr.addTag(file_id, "docs");
    try mgr.setMetadata(file_id, "mime", "text/markdown");
    try mgr.updateBody(file_id, "hello world");
    try mgr.rename(file_id, "README.md");

    const file = try mgr.getContentById(file_id);
    try std.testing.expectEqual(@as(usize, 1), file.tags.items.len);
    try std.testing.expectEqual(@as(usize, 1), file.metadata.items.len);
    try std.testing.expect(mgr.countByKind(.file) >= 1);

    const note_id = try mgr.createNote(1, 2, file_id, "todo", "expand intro", .high);
    try mgr.addNoteTag(note_id, "writing");
    try mgr.pinNote(note_id);
    try mgr.updateNote(note_id, "todo updated", "expand intro and examples", .critical);
    const note = try mgr.getNoteById(note_id);
    try std.testing.expect(note.pinned);
    try std.testing.expectEqual(NoteImportance.critical, note.importance);
    try std.testing.expectEqual(@as(usize, 1), note.tags.items.len);

    {
        var linked_notes = try mgr.getNotesForContent(file_id, allocator);
        defer linked_notes.deinit();
        try std.testing.expectEqual(@as(usize, 1), linked_notes.items.len);
    }

    try mgr.archiveNote(note_id);
    {
        var linked_notes = try mgr.getNotesForContent(file_id, allocator);
        defer linked_notes.deinit();
        try std.testing.expectEqual(@as(usize, 0), linked_notes.items.len);
    }
}
