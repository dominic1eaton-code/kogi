const std = @import("std");

pub const DirectoryEntityKind = enum {
    independent_worker,
    organization,
    client,
    investor,
    stakeholder,
    connection,
    networking,
    friend,
    contractor,
    employee,
    customer,
    business_relationship,
    custom,
};

pub const RelationshipStatus = enum {
    prospective,
    active,
    inactive,
    blocked,
};

pub const RelationshipStrength = enum {
    low,
    medium,
    high,
    strategic,
};

pub const RelationshipLink = struct {
    target_entry_id: u32,
    relation_kind: DirectoryEntityKind,
    status: RelationshipStatus,
    strength: RelationshipStrength,
    since_at: i64,
};

pub const DirectoryEntry = struct {
    id: u32,
    owner_user_id: u32,
    profile_id: ?u32,
    persona_id: ?u32,
    managed_contact_id: ?u32,
    organization_id: ?u32,
    kind: DirectoryEntityKind,
    status: RelationshipStatus,
    display_name: []const u8,
    headline: []const u8,
    notes: []const u8,
    tags: std.ArrayList([]const u8),
    linked_note_ids: std.ArrayList(u32),
    relationships: std.ArrayList(RelationshipLink),
    active: bool,
    created_at: i64,
    updated_at: i64,
};

pub const DirectoryOrganization = struct {
    id: u32,
    owner_user_id: u32,
    name: []const u8,
    industry: []const u8,
    notes: []const u8,
    entry_ids: std.ArrayList(u32),
    active: bool,
    created_at: i64,
    updated_at: i64,
};

pub const DirectoryBookError = error{
    EntryNotFound,
    OrganizationNotFound,
};

pub const DirectoryBookManager = struct {
    allocator: std.mem.Allocator,
    entries: std.ArrayList(DirectoryEntry),
    organizations: std.ArrayList(DirectoryOrganization),
    next_entry_id: u32,
    next_org_id: u32,

    pub fn init(allocator: std.mem.Allocator) DirectoryBookManager {
        return DirectoryBookManager{
            .allocator = allocator,
            .entries = std.ArrayList(DirectoryEntry).init(allocator),
            .organizations = std.ArrayList(DirectoryOrganization).init(allocator),
            .next_entry_id = 0,
            .next_org_id = 0,
        };
    }

    pub fn deinit(self: *DirectoryBookManager) void {
        for (self.entries.items) |*entry| {
            self.allocator.free(entry.display_name);
            self.allocator.free(entry.headline);
            self.allocator.free(entry.notes);
            for (entry.tags.items) |tag| self.allocator.free(tag);
            entry.tags.deinit();
            entry.linked_note_ids.deinit();
            entry.relationships.deinit();
        }
        self.entries.deinit();

        for (self.organizations.items) |*org| {
            self.allocator.free(org.name);
            self.allocator.free(org.industry);
            self.allocator.free(org.notes);
            org.entry_ids.deinit();
        }
        self.organizations.deinit();
    }

    pub fn createEntry(
        self: *DirectoryBookManager,
        owner_user_id: u32,
        profile_id: ?u32,
        kind: DirectoryEntityKind,
        display_name: []const u8,
        headline: []const u8,
        notes: []const u8,
    ) !u32 {
        const entry_id = self.next_entry_id;
        self.next_entry_id += 1;
        const now = std.time.timestamp();
        const entry = DirectoryEntry{
            .id = entry_id,
            .owner_user_id = owner_user_id,
            .profile_id = profile_id,
            .persona_id = null,
            .managed_contact_id = null,
            .organization_id = null,
            .kind = kind,
            .status = .prospective,
            .display_name = try self.allocator.dupe(u8, display_name),
            .headline = try self.allocator.dupe(u8, headline),
            .notes = try self.allocator.dupe(u8, notes),
            .tags = std.ArrayList([]const u8).init(self.allocator),
            .linked_note_ids = std.ArrayList(u32).init(self.allocator),
            .relationships = std.ArrayList(RelationshipLink).init(self.allocator),
            .active = true,
            .created_at = now,
            .updated_at = now,
        };
        errdefer {
            self.allocator.free(entry.display_name);
            self.allocator.free(entry.headline);
            self.allocator.free(entry.notes);
            var tags = entry.tags;
            var note_ids = entry.linked_note_ids;
            var rels = entry.relationships;
            tags.deinit();
            note_ids.deinit();
            rels.deinit();
        }
        try self.entries.append(entry);
        return entry_id;
    }

    pub fn updateEntry(self: *DirectoryBookManager, entry_id: u32, display_name: []const u8, headline: []const u8, notes: []const u8) !void {
        const idx = self.findEntryIndexById(entry_id) orelse return DirectoryBookError.EntryNotFound;
        const entry = &self.entries.items[idx];
        self.allocator.free(entry.display_name);
        self.allocator.free(entry.headline);
        self.allocator.free(entry.notes);
        entry.display_name = try self.allocator.dupe(u8, display_name);
        entry.headline = try self.allocator.dupe(u8, headline);
        entry.notes = try self.allocator.dupe(u8, notes);
        entry.updated_at = std.time.timestamp();
    }

    pub fn setEntryStatus(self: *DirectoryBookManager, entry_id: u32, status: RelationshipStatus) !void {
        const idx = self.findEntryIndexById(entry_id) orelse return DirectoryBookError.EntryNotFound;
        self.entries.items[idx].status = status;
        self.entries.items[idx].updated_at = std.time.timestamp();
    }

    pub fn associatePersona(self: *DirectoryBookManager, entry_id: u32, persona_id: ?u32) !void {
        const idx = self.findEntryIndexById(entry_id) orelse return DirectoryBookError.EntryNotFound;
        self.entries.items[idx].persona_id = persona_id;
        self.entries.items[idx].updated_at = std.time.timestamp();
    }

    pub fn linkManagedContact(self: *DirectoryBookManager, entry_id: u32, managed_contact_id: ?u32) !void {
        const idx = self.findEntryIndexById(entry_id) orelse return DirectoryBookError.EntryNotFound;
        self.entries.items[idx].managed_contact_id = managed_contact_id;
        self.entries.items[idx].updated_at = std.time.timestamp();
    }

    pub fn linkNote(self: *DirectoryBookManager, entry_id: u32, note_id: u32) !void {
        const idx = self.findEntryIndexById(entry_id) orelse return DirectoryBookError.EntryNotFound;
        for (self.entries.items[idx].linked_note_ids.items) |existing| {
            if (existing == note_id) return;
        }
        try self.entries.items[idx].linked_note_ids.append(note_id);
        self.entries.items[idx].updated_at = std.time.timestamp();
    }

    pub fn unlinkNote(self: *DirectoryBookManager, entry_id: u32, note_id: u32) !void {
        const idx = self.findEntryIndexById(entry_id) orelse return DirectoryBookError.EntryNotFound;
        var i: usize = 0;
        while (i < self.entries.items[idx].linked_note_ids.items.len) : (i += 1) {
            if (self.entries.items[idx].linked_note_ids.items[i] == note_id) {
                _ = self.entries.items[idx].linked_note_ids.orderedRemove(i);
                self.entries.items[idx].updated_at = std.time.timestamp();
                return;
            }
        }
    }

    pub fn addTag(self: *DirectoryBookManager, entry_id: u32, tag: []const u8) !void {
        const idx = self.findEntryIndexById(entry_id) orelse return DirectoryBookError.EntryNotFound;
        for (self.entries.items[idx].tags.items) |existing| {
            if (std.mem.eql(u8, existing, tag)) return;
        }
        try self.entries.items[idx].tags.append(try self.allocator.dupe(u8, tag));
        self.entries.items[idx].updated_at = std.time.timestamp();
    }

    pub fn addRelationship(
        self: *DirectoryBookManager,
        entry_id: u32,
        target_entry_id: u32,
        relation_kind: DirectoryEntityKind,
        status: RelationshipStatus,
        strength: RelationshipStrength,
    ) !void {
        const idx = self.findEntryIndexById(entry_id) orelse return DirectoryBookError.EntryNotFound;
        _ = self.findEntryIndexById(target_entry_id) orelse return DirectoryBookError.EntryNotFound;
        const rel = RelationshipLink{
            .target_entry_id = target_entry_id,
            .relation_kind = relation_kind,
            .status = status,
            .strength = strength,
            .since_at = std.time.timestamp(),
        };
        try self.entries.items[idx].relationships.append(rel);
        self.entries.items[idx].updated_at = std.time.timestamp();
    }

    pub fn createOrganization(self: *DirectoryBookManager, owner_user_id: u32, name: []const u8, industry: []const u8, notes: []const u8) !u32 {
        const org_id = self.next_org_id;
        self.next_org_id += 1;
        const now = std.time.timestamp();
        const org = DirectoryOrganization{
            .id = org_id,
            .owner_user_id = owner_user_id,
            .name = try self.allocator.dupe(u8, name),
            .industry = try self.allocator.dupe(u8, industry),
            .notes = try self.allocator.dupe(u8, notes),
            .entry_ids = std.ArrayList(u32).init(self.allocator),
            .active = true,
            .created_at = now,
            .updated_at = now,
        };
        errdefer {
            self.allocator.free(org.name);
            self.allocator.free(org.industry);
            self.allocator.free(org.notes);
            var entry_ids = org.entry_ids;
            entry_ids.deinit();
        }
        try self.organizations.append(org);
        return org_id;
    }

    pub fn assignEntryToOrganization(self: *DirectoryBookManager, entry_id: u32, organization_id: ?u32) !void {
        const entry_idx = self.findEntryIndexById(entry_id) orelse return DirectoryBookError.EntryNotFound;

        if (self.entries.items[entry_idx].organization_id) |old_org_id| {
            if (self.findOrganizationIndexById(old_org_id)) |old_idx| {
                var i: usize = 0;
                while (i < self.organizations.items[old_idx].entry_ids.items.len) : (i += 1) {
                    if (self.organizations.items[old_idx].entry_ids.items[i] == entry_id) {
                        _ = self.organizations.items[old_idx].entry_ids.orderedRemove(i);
                        break;
                    }
                }
            }
        }

        self.entries.items[entry_idx].organization_id = organization_id;
        self.entries.items[entry_idx].updated_at = std.time.timestamp();

        if (organization_id) |org_id| {
            const org_idx = self.findOrganizationIndexById(org_id) orelse return DirectoryBookError.OrganizationNotFound;
            for (self.organizations.items[org_idx].entry_ids.items) |existing| {
                if (existing == entry_id) return;
            }
            try self.organizations.items[org_idx].entry_ids.append(entry_id);
            self.organizations.items[org_idx].updated_at = std.time.timestamp();
        }
    }

    pub fn deactivateEntry(self: *DirectoryBookManager, entry_id: u32) !void {
        const idx = self.findEntryIndexById(entry_id) orelse return DirectoryBookError.EntryNotFound;
        self.entries.items[idx].active = false;
        self.entries.items[idx].status = .inactive;
        self.entries.items[idx].updated_at = std.time.timestamp();
    }

    pub fn getEntryById(self: *DirectoryBookManager, entry_id: u32) !DirectoryEntry {
        const idx = self.findEntryIndexById(entry_id) orelse return DirectoryBookError.EntryNotFound;
        return self.entries.items[idx];
    }

    pub fn getEntries(self: *DirectoryBookManager) []DirectoryEntry {
        return self.entries.items;
    }

    pub fn getOrganizationById(self: *DirectoryBookManager, organization_id: u32) !DirectoryOrganization {
        const idx = self.findOrganizationIndexById(organization_id) orelse return DirectoryBookError.OrganizationNotFound;
        return self.organizations.items[idx];
    }

    pub fn getOrganizations(self: *DirectoryBookManager) []DirectoryOrganization {
        return self.organizations.items;
    }

    fn findEntryIndexById(self: *DirectoryBookManager, entry_id: u32) ?usize {
        for (self.entries.items, 0..) |entry, idx| {
            if (entry.id == entry_id) return idx;
        }
        return null;
    }

    fn findOrganizationIndexById(self: *DirectoryBookManager, organization_id: u32) ?usize {
        for (self.organizations.items, 0..) |org, idx| {
            if (org.id == organization_id) return idx;
        }
        return null;
    }
};

pub fn parseDirectoryEntityKind(name: []const u8) ?DirectoryEntityKind {
    if (std.mem.eql(u8, name, "independent_worker")) return .independent_worker;
    if (std.mem.eql(u8, name, "organization")) return .organization;
    if (std.mem.eql(u8, name, "client")) return .client;
    if (std.mem.eql(u8, name, "investor")) return .investor;
    if (std.mem.eql(u8, name, "stakeholder")) return .stakeholder;
    if (std.mem.eql(u8, name, "connection")) return .connection;
    if (std.mem.eql(u8, name, "networking")) return .networking;
    if (std.mem.eql(u8, name, "friend")) return .friend;
    if (std.mem.eql(u8, name, "contractor")) return .contractor;
    if (std.mem.eql(u8, name, "employee")) return .employee;
    if (std.mem.eql(u8, name, "customer")) return .customer;
    if (std.mem.eql(u8, name, "business_relationship")) return .business_relationship;
    if (std.mem.eql(u8, name, "custom")) return .custom;
    return null;
}

test "directory book lifecycle with relationships persona and notes" {
    const allocator = std.testing.allocator;
    var mgr = DirectoryBookManager.init(allocator);
    defer mgr.deinit();

    const org = try mgr.createOrganization(1, "Acme Ventures", "Investment", "Primary investor");
    const alice = try mgr.createEntry(1, 2, .independent_worker, "Alice", "Founder", "Core operator");
    const bob = try mgr.createEntry(1, 2, .investor, "Bob VC", "Partner", "Board observer");

    try mgr.assignEntryToOrganization(bob, org);
    try mgr.associatePersona(alice, 10);
    try mgr.linkNote(alice, 99);
    try mgr.addTag(alice, "priority");
    try mgr.addRelationship(alice, bob, .stakeholder, .active, .high);

    const a = try mgr.getEntryById(alice);
    try std.testing.expectEqual(@as(usize, 1), a.relationships.items.len);
    try std.testing.expectEqual(@as(usize, 1), a.linked_note_ids.items.len);
    try std.testing.expectEqual(@as(u32, 10), a.persona_id.?);
    try std.testing.expectEqual(@as(usize, 1), (try mgr.getOrganizationById(org)).entry_ids.items.len);
}
