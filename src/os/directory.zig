//! KOGI Directory Module - Contact Management in the KOGI OS
//! The Directory manages contacts and relationships for identities.
//! Analogous to system users database and contacts registry.

const std = @import("std");

/// Directory contact entry
pub const Contact = struct {
    id: u32,
    name: []const u8,
    email: []const u8,
    phone: []const u8,
    notes: []const u8,
};

/// Organization/Client in the directory
pub const Organization = struct {
    id: u32,
    name: []const u8,
    industry: []const u8,
    contacts: std.array_list.Managed(Contact),
    notes: []const u8,
};

/// Directory manages all contacts and organizations
pub const Directory = struct {
    allocator: std.mem.Allocator,
    organizations: std.array_list.Managed(Organization),

    pub fn init(allocator: std.mem.Allocator) Directory {
        return Directory{
            .allocator = allocator,
            .organizations = std.array_list.Managed(Organization).init(allocator),
        };
    }

    pub fn deinit(self: *Directory) void {
        for (self.organizations.items) |*org| {
            self.allocator.free(org.name);
            self.allocator.free(org.industry);
            self.allocator.free(org.notes);
            for (org.contacts.items) |contact| {
                self.allocator.free(contact.name);
                self.allocator.free(contact.email);
                self.allocator.free(contact.phone);
                self.allocator.free(contact.notes);
            }
            org.contacts.deinit();
        }
        self.organizations.deinit();
    }

    /// Add an organization to the directory
    pub fn addOrganization(
        self: *Directory,
        name: []const u8,
        industry: []const u8,
        notes: []const u8,
    ) !u32 {
        const org_id = @as(u32, @intCast(self.organizations.items.len));

        const organization = Organization{
            .id = org_id,
            .name = try self.allocator.dupe(u8, name),
            .industry = try self.allocator.dupe(u8, industry),
            .contacts = std.array_list.Managed(Contact).init(self.allocator),
            .notes = try self.allocator.dupe(u8, notes),
        };

        try self.organizations.append(organization);
        return org_id;
    }

    /// Add a contact to an organization
    pub fn addContactToOrganization(
        self: *Directory,
        org_id: u32,
        name: []const u8,
        email: []const u8,
        phone: []const u8,
        notes: []const u8,
    ) !u32 {
        if (org_id >= @as(u32, @intCast(self.organizations.items.len))) {
            return error.OrganizationNotFound;
        }

        const contact_id = @as(u32, @intCast(self.organizations.items[org_id].contacts.items.len));

        const contact = Contact{
            .id = contact_id,
            .name = try self.allocator.dupe(u8, name),
            .email = try self.allocator.dupe(u8, email),
            .phone = try self.allocator.dupe(u8, phone),
            .notes = try self.allocator.dupe(u8, notes),
        };

        try self.organizations.items[org_id].contacts.append(contact);
        return contact_id;
    }

    /// Get organization count
    pub fn getOrganizationCount(self: *Directory) u32 {
        return @as(u32, @intCast(self.organizations.items.len));
    }

    /// Get total contact count
    pub fn getContactCount(self: *Directory) u32 {
        var count: u32 = 0;
        for (self.organizations.items) |org| {
            count += @as(u32, @intCast(org.contacts.items.len));
        }
        return count;
    }
};

// Backward compatibility aliases
pub const Client = Organization;
pub const CRMManager = Directory;
