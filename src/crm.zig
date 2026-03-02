const std = @import("std");

/// Contact represents an individual person or entity that the worker
/// interacts with (clients, partners, vendors, social contacts, etc.).
pub const Contact = struct {
    id: u32,
    name: []const u8,
    email: ?[]const u8,
    phone: ?[]const u8,
    notes: ?[]const u8,
};

/// Client is an organization or person for whom work is performed.
pub const Client = struct {
    id: u32,
    name: []const u8,
    industry: []const u8,
    contacts: std.ArrayList(Contact),
    notes: ?[]const u8,
};

/// Simple CRM manager to track clients and contacts
pub const CRMManager = struct {
    allocator: std.mem.Allocator,
    clients: std.ArrayList(Client),
    next_id: u32,

    pub fn init(allocator: std.mem.Allocator) CRMManager {
        return CRMManager{
            .allocator = allocator,
            .clients = std.ArrayList(Client){},
            .next_id = 0,
        };
    }

    pub fn deinit(self: *CRMManager) void {
        for (self.clients.items) |*client| {
            self.deinitClient(client);
        }
        self.clients.deinit(self.allocator);
    }

    fn deinitClient(self: *CRMManager, client: *Client) void {
        for (client.contacts.items) |*c| {
            self.allocator.free(c.name);
            if (c.email) |e| self.allocator.free(e);
            if (c.phone) |p| self.allocator.free(p);
            if (c.notes) |n| self.allocator.free(n);
        }
        client.contacts.deinit(self.allocator);
        self.allocator.free(client.name);
        self.allocator.free(client.industry);
        if (client.notes) |n| self.allocator.free(n);
    }

    pub fn addClient(self: *CRMManager, name: []const u8, industry: []const u8, notes: ?[]const u8) !u32 {
        const id = self.next_id;
        self.next_id += 1;
        const client = Client{
            .id = id,
            .name = try self.allocator.dupe(u8, name),
            .industry = try self.allocator.dupe(u8, industry),
            .contacts = std.ArrayList(Contact){},
            .notes = if (notes) |n| try self.allocator.dupe(u8, n) else null,
        };
        try self.clients.append(self.allocator, client);
        return id;
    }

    pub fn addContactToClient(
        self: *CRMManager,
        client_id: u32,
        name: []const u8,
        email: ?[]const u8,
        phone: ?[]const u8,
        notes: ?[]const u8,
    ) !u32 {
        if (client_id >= @as(u32, @intCast(self.clients.items.len))) {
            return error.ClientNotFound;
        }
        const client = &self.clients.items[client_id];
        const id = @as(u32, @intCast(client.contacts.items.len));
        const contact = Contact{
            .id = id,
            .name = try self.allocator.dupe(u8, name),
            .email = if (email) |e| try self.allocator.dupe(u8, e) else null,
            .phone = if (phone) |p| try self.allocator.dupe(u8, p) else null,
            .notes = if (notes) |n| try self.allocator.dupe(u8, n) else null,
        };
        try client.contacts.append(self.allocator, contact);
        return id;
    }

    pub fn getClientCount(self: *CRMManager) u32 {
        return @as(u32, @intCast(self.clients.items.len));
    }

    pub fn getContactCount(self: *CRMManager, client_id: u32) u32 {
        if (client_id >= @as(u32, @intCast(self.clients.items.len))) {
            return 0;
        }
        return @as(u32, @intCast(self.clients.items[client_id].contacts.items.len));
    }
};

pub fn crmDemo() void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var crm = CRMManager.init(allocator);
    defer crm.deinit();

    const cid = try crm.addClient("Acme Corp", "Technology", "Key client");
    _ = try crm.addContactToClient(cid, "John Doe", "john@acme.com", null, "CEO");
    std.debug.print("CRM created {d} clients, {d} contacts for first\n", .{ crm.getClientCount(), crm.getContactCount(cid) });
}
