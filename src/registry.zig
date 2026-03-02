//! KOGI Connection Registry - Account/Connection Management in the KOGI OS
//! The registry manages all external connections/accounts for an identity.
//! Analogous to the filesystem mount table or device registry in traditional OS.

const std = @import("std");

/// Connection types supported in the KOGI OS
pub const ConnectionType = enum {
    social_media,
    work,
    personal,
    email,
    software,
    other,
};

/// Connection represents an external account/connection for an identity
pub const Connection = struct {
    id: u32,
    connection_type: ConnectionType,
    username: []const u8,
    provider: []const u8,
    details: []const u8,
    active: bool,
};

/// ConnectionRegistry manages all external connections for an identity
pub const ConnectionRegistry = struct {
    allocator: std.mem.Allocator,
    connections: std.ArrayList(Connection),

    pub fn init(allocator: std.mem.Allocator) ConnectionRegistry {
        return ConnectionRegistry{
            .allocator = allocator,
            .connections = std.ArrayList(Connection){},
        };
    }

    pub fn deinit(self: *ConnectionRegistry) void {
        for (self.connections.items) |connection| {
            self.allocator.free(connection.username);
            self.allocator.free(connection.provider);
            self.allocator.free(connection.details);
        }
        self.connections.deinit(self.allocator);
    }

    /// Add a new connection to the registry
    pub fn addConnection(
        self: *ConnectionRegistry,
        conn_type: ConnectionType,
        username: []const u8,
        provider: []const u8,
        details: []const u8,
    ) !u32 {
        const connection_id = @as(u32, @intCast(self.connections.items.len));

        const connection = Connection{
            .id = connection_id,
            .connection_type = conn_type,
            .username = try self.allocator.dupe(u8, username),
            .provider = try self.allocator.dupe(u8, provider),
            .details = try self.allocator.dupe(u8, details),
            .active = true,
        };

        try self.connections.append(self.allocator, connection);
        return connection_id;
    }

    /// Deactivate a connection
    pub fn deactivateConnection(self: *ConnectionRegistry, connection_id: u32) !void {
        if (connection_id < @as(u32, @intCast(self.connections.items.len))) {
            self.connections.items[connection_id].active = false;
        }
    }

    /// Get count of active connections
    pub fn getActiveCount(self: *ConnectionRegistry) u32 {
        var count: u32 = 0;
        for (self.connections.items) |connection| {
            if (connection.active) {
                count += 1;
            }
        }
        return count;
    }

    /// Find connection by provider
    pub fn findByProvider(self: *ConnectionRegistry, provider: []const u8) ?Connection {
        for (self.connections.items) |connection| {
            if (std.mem.eql(u8, connection.provider, provider) and connection.active) {
                return connection;
            }
        }
        return null;
    }
};

// Backward compatibility alias
pub const Account = Connection;
pub const AccountType = ConnectionType;
pub const AccountManager = ConnectionRegistry;
