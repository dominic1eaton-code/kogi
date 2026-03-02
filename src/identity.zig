//! KOGI Identity Module - User/Worker Profiles in the KOGI OS
//! An Identity represents a user account in the KOGI operating system.
//! Analogous to a user account in traditional operating systems.

const std = @import("std");
const registry_module = @import("registry.zig");

/// Error types for identity operations
pub const IdentityError = error{
    IdentityNotFound,
};

/// Identity types supported in the KOGI OS
pub const IdentityType = enum {
    contractor,
    consultant,
    freelancer,
    gig,
    project_based,
    artist,
    musician,
    developer,
    gamer,
    other,
};

/// Identity represents a user account in the KOGI operating system
pub const Identity = struct {
    id: u32,
    name: []const u8,
    email: []const u8,
    identity_type: IdentityType,
    skills: std.ArrayList([]const u8),
    hourly_rate: f32,
    active: bool,
    connection_registry: registry_module.ConnectionRegistry,
};

/// IdentityManager manages all identities in the KOGI system
pub const IdentityManager = struct {
    allocator: std.mem.Allocator,
    identities: std.ArrayList(Identity),

    pub fn init(allocator: std.mem.Allocator) IdentityManager {
        return IdentityManager{
            .allocator = allocator,
            .identities = std.ArrayList(Identity){},
        };
    }

    pub fn deinit(self: *IdentityManager) void {
        for (self.identities.items) |*identity| {
            // Free each skill string
            for (identity.skills.items) |skill| {
                self.allocator.free(skill);
            }
            identity.skills.deinit(self.allocator);
            self.allocator.free(identity.name);
            self.allocator.free(identity.email);
            // deinit registry
            identity.connection_registry.deinit();
        }
        self.identities.deinit(self.allocator);
    }

    /// Create a new identity in the system
    pub fn createIdentity(
        self: *IdentityManager,
        name: []const u8,
        email: []const u8,
        hourly_rate: f32,
        itype: IdentityType,
    ) !u32 {
        const identity_id = @as(u32, @intCast(self.identities.items.len));

        const identity = Identity{
            .id = identity_id,
            .name = try self.allocator.dupe(u8, name),
            .email = try self.allocator.dupe(u8, email),
            .identity_type = itype,
            .skills = std.ArrayList([]const u8){},
            .hourly_rate = hourly_rate,
            .active = true,
            .connection_registry = registry_module.ConnectionRegistry.init(self.allocator),
        };

        try self.identities.append(self.allocator, identity);
        return identity_id;
    }

    /// Add a skill to an identity's profile
    pub fn addSkill(self: *IdentityManager, identity_id: u32, skill: []const u8) !void {
        if (identity_id < @as(u32, @intCast(self.identities.items.len))) {
            const skill_copy = try self.allocator.dupe(u8, skill);
            try self.identities.items[identity_id].skills.append(self.allocator, skill_copy);
        }
    }

    /// Connection management: add a connection for an identity
    pub fn addIdentityConnection(
        self: *IdentityManager,
        identity_id: u32,
        conn_type: registry_module.ConnectionType,
        username: []const u8,
        provider: []const u8,
        details: []const u8,
    ) !u32 {
        if (identity_id >= @as(u32, @intCast(self.identities.items.len))) {
            return IdentityError.IdentityNotFound;
        }
        return try self.identities.items[identity_id].connection_registry.addConnection(
            conn_type,
            username,
            provider,
            details,
        );
    }

    pub fn deactivateIdentityConnection(self: *IdentityManager, identity_id: u32, conn_id: u32) !void {
        if (identity_id >= @as(u32, @intCast(self.identities.items.len))) {
            return IdentityError.IdentityNotFound;
        }
        try self.identities.items[identity_id].connection_registry.deactivateConnection(conn_id);
    }

    pub fn getIdentityActiveConnectionCount(self: *IdentityManager, identity_id: u32) u32 {
        if (identity_id < @as(u32, @intCast(self.identities.items.len))) {
            return self.identities.items[identity_id].connection_registry.getActiveCount();
        }
        return 0;
    }

    /// Deactivate an identity
    pub fn deactivateIdentity(self: *IdentityManager, identity_id: u32) void {
        if (identity_id < @as(u32, @intCast(self.identities.items.len))) {
            self.identities.items[identity_id].active = false;
        }
    }

    /// Get count of active identities
    pub fn getActiveCount(self: *IdentityManager) u32 {
        var count: u32 = 0;
        for (self.identities.items) |identity| {
            if (identity.active) {
                count += 1;
            }
        }
        return count;
    }

    /// Get identities matching a given type
    pub fn getIdentitiesByType(self: *IdentityManager, itype: IdentityType, allocator: std.mem.Allocator) !std.ArrayList(Identity) {
        var result = std.ArrayList(Identity){};
        for (self.identities.items) |identity| {
            if (identity.identity_type == itype) {
                try result.append(allocator, identity);
            }
        }
        return result;
    }
};
