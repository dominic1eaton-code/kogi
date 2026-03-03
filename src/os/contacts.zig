const std = @import("std");

pub const ContactKind = enum {
    person,
    organization,
    service,
    custom,
};

pub const ContactChannelKind = enum {
    email,
    phone,
    social,
    website,
    address,
    other,
};

pub const ContactChannel = struct {
    kind: ContactChannelKind,
    label: []const u8,
    value: []const u8,
    verified: bool,
};

pub const ManagedContact = struct {
    id: u32,
    owner_user_id: u32,
    profile_id: ?u32,
    kind: ContactKind,
    display_name: []const u8,
    notes: []const u8,
    channels: std.ArrayList(ContactChannel),
    tags: std.ArrayList([]const u8),
    linked_account_ids: std.ArrayList(u32),
    active: bool,
    created_at: i64,
    updated_at: i64,
};

pub const ContactManagementError = error{
    ContactNotFound,
};

pub const ContactManagement = struct {
    allocator: std.mem.Allocator,
    contacts: std.ArrayList(ManagedContact),
    next_contact_id: u32,

    pub fn init(allocator: std.mem.Allocator) ContactManagement {
        return ContactManagement{
            .allocator = allocator,
            .contacts = std.ArrayList(ManagedContact).init(allocator),
            .next_contact_id = 0,
        };
    }

    pub fn deinit(self: *ContactManagement) void {
        for (self.contacts.items) |*contact| {
            self.allocator.free(contact.display_name);
            self.allocator.free(contact.notes);
            for (contact.channels.items) |channel| {
                self.allocator.free(channel.label);
                self.allocator.free(channel.value);
            }
            contact.channels.deinit();
            for (contact.tags.items) |tag| {
                self.allocator.free(tag);
            }
            contact.tags.deinit();
            contact.linked_account_ids.deinit();
        }
        self.contacts.deinit();
    }

    pub fn createContact(
        self: *ContactManagement,
        owner_user_id: u32,
        profile_id: ?u32,
        kind: ContactKind,
        display_name: []const u8,
        notes: []const u8,
    ) !u32 {
        const id = self.next_contact_id;
        self.next_contact_id += 1;
        const now = std.time.timestamp();
        const contact = ManagedContact{
            .id = id,
            .owner_user_id = owner_user_id,
            .profile_id = profile_id,
            .kind = kind,
            .display_name = try self.allocator.dupe(u8, display_name),
            .notes = try self.allocator.dupe(u8, notes),
            .channels = std.ArrayList(ContactChannel).init(self.allocator),
            .tags = std.ArrayList([]const u8).init(self.allocator),
            .linked_account_ids = std.ArrayList(u32).init(self.allocator),
            .active = true,
            .created_at = now,
            .updated_at = now,
        };
        errdefer {
            self.allocator.free(contact.display_name);
            self.allocator.free(contact.notes);
            var channels = contact.channels;
            channels.deinit();
            var tags = contact.tags;
            tags.deinit();
            var linked = contact.linked_account_ids;
            linked.deinit();
        }
        try self.contacts.append(contact);
        return id;
    }

    pub fn setProfile(self: *ContactManagement, contact_id: u32, profile_id: ?u32) !void {
        const idx = self.findContactIndexById(contact_id) orelse return ContactManagementError.ContactNotFound;
        self.contacts.items[idx].profile_id = profile_id;
        self.contacts.items[idx].updated_at = std.time.timestamp();
    }

    pub fn updateDetails(self: *ContactManagement, contact_id: u32, display_name: []const u8, notes: []const u8) !void {
        const idx = self.findContactIndexById(contact_id) orelse return ContactManagementError.ContactNotFound;
        const contact = &self.contacts.items[idx];
        self.allocator.free(contact.display_name);
        self.allocator.free(contact.notes);
        contact.display_name = try self.allocator.dupe(u8, display_name);
        contact.notes = try self.allocator.dupe(u8, notes);
        contact.updated_at = std.time.timestamp();
    }

    pub fn addChannel(
        self: *ContactManagement,
        contact_id: u32,
        kind: ContactChannelKind,
        label: []const u8,
        value: []const u8,
    ) !void {
        const idx = self.findContactIndexById(contact_id) orelse return ContactManagementError.ContactNotFound;
        const channel = ContactChannel{
            .kind = kind,
            .label = try self.allocator.dupe(u8, label),
            .value = try self.allocator.dupe(u8, value),
            .verified = false,
        };
        errdefer {
            self.allocator.free(channel.label);
            self.allocator.free(channel.value);
        }
        try self.contacts.items[idx].channels.append(channel);
        self.contacts.items[idx].updated_at = std.time.timestamp();
    }

    pub fn verifyChannel(self: *ContactManagement, contact_id: u32, channel_index: usize) !void {
        const idx = self.findContactIndexById(contact_id) orelse return ContactManagementError.ContactNotFound;
        if (channel_index >= self.contacts.items[idx].channels.items.len) return;
        self.contacts.items[idx].channels.items[channel_index].verified = true;
        self.contacts.items[idx].updated_at = std.time.timestamp();
    }

    pub fn addTag(self: *ContactManagement, contact_id: u32, tag: []const u8) !void {
        const idx = self.findContactIndexById(contact_id) orelse return ContactManagementError.ContactNotFound;
        for (self.contacts.items[idx].tags.items) |existing| {
            if (std.mem.eql(u8, existing, tag)) return;
        }
        try self.contacts.items[idx].tags.append(try self.allocator.dupe(u8, tag));
        self.contacts.items[idx].updated_at = std.time.timestamp();
    }

    pub fn linkAccount(self: *ContactManagement, contact_id: u32, account_id: u32) !void {
        const idx = self.findContactIndexById(contact_id) orelse return ContactManagementError.ContactNotFound;
        for (self.contacts.items[idx].linked_account_ids.items) |existing| {
            if (existing == account_id) return;
        }
        try self.contacts.items[idx].linked_account_ids.append(account_id);
        self.contacts.items[idx].updated_at = std.time.timestamp();
    }

    pub fn unlinkAccount(self: *ContactManagement, contact_id: u32, account_id: u32) !void {
        const idx = self.findContactIndexById(contact_id) orelse return ContactManagementError.ContactNotFound;
        var i: usize = 0;
        while (i < self.contacts.items[idx].linked_account_ids.items.len) : (i += 1) {
            if (self.contacts.items[idx].linked_account_ids.items[i] == account_id) {
                _ = self.contacts.items[idx].linked_account_ids.orderedRemove(i);
                self.contacts.items[idx].updated_at = std.time.timestamp();
                return;
            }
        }
    }

    pub fn deactivateContact(self: *ContactManagement, contact_id: u32) !void {
        const idx = self.findContactIndexById(contact_id) orelse return ContactManagementError.ContactNotFound;
        self.contacts.items[idx].active = false;
        self.contacts.items[idx].updated_at = std.time.timestamp();
    }

    pub fn getContacts(self: *ContactManagement) []ManagedContact {
        return self.contacts.items;
    }

    pub fn getContactById(self: *ContactManagement, contact_id: u32) !ManagedContact {
        const idx = self.findContactIndexById(contact_id) orelse return ContactManagementError.ContactNotFound;
        return self.contacts.items[idx];
    }

    fn findContactIndexById(self: *ContactManagement, contact_id: u32) ?usize {
        for (self.contacts.items, 0..) |contact, idx| {
            if (contact.id == contact_id) return idx;
        }
        return null;
    }
};

pub fn parseContactKind(name: []const u8) ?ContactKind {
    if (std.mem.eql(u8, name, "person")) return .person;
    if (std.mem.eql(u8, name, "organization")) return .organization;
    if (std.mem.eql(u8, name, "service")) return .service;
    if (std.mem.eql(u8, name, "custom")) return .custom;
    return null;
}

pub fn parseContactChannelKind(name: []const u8) ?ContactChannelKind {
    if (std.mem.eql(u8, name, "email")) return .email;
    if (std.mem.eql(u8, name, "phone")) return .phone;
    if (std.mem.eql(u8, name, "social")) return .social;
    if (std.mem.eql(u8, name, "website")) return .website;
    if (std.mem.eql(u8, name, "address")) return .address;
    if (std.mem.eql(u8, name, "other")) return .other;
    return null;
}

test "contact management lifecycle" {
    const allocator = std.testing.allocator;
    var mgr = ContactManagement.init(allocator);
    defer mgr.deinit();

    const contact_id = try mgr.createContact(1, 2, .person, "Alex Doe", "Design partner");
    try mgr.addChannel(contact_id, .email, "work", "alex@example.com");
    try mgr.verifyChannel(contact_id, 0);
    try mgr.addTag(contact_id, "design");
    try mgr.linkAccount(contact_id, 11);

    const contact = try mgr.getContactById(contact_id);
    try std.testing.expect(contact.active);
    try std.testing.expectEqual(@as(usize, 1), contact.channels.items.len);
    try std.testing.expect(contact.channels.items[0].verified);
    try std.testing.expectEqual(@as(usize, 1), contact.tags.items.len);
    try std.testing.expectEqual(@as(usize, 1), contact.linked_account_ids.items.len);
}
