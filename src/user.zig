//!
//!
//!
const std = @import("std");

pub const User = struct {
    id: u32,
    name: []const u8,
    active: bool,
    token: []const u32,
};

pub const Identity = struct {
    id: u32,
    name: []const u8,
    active: bool,
    token: []const u32,
};

pub const Account = struct {
    id: u32,
    name: []const u8,
    active: bool,
    token: []const u32,
};

pub const Profile = struct {
    id: u32,
    name: []const u8,
};

pub const Contact = struct {
    id: u32,
    name: []const u8,
};

pub const UserManager = struct {
    pub fn init(allocator: std.mem.Allocator) UserManager {
        return UserManager{
            .allocator = allocator,
        };
    }

    // pub fn deinit(self: *UserManager) void {
    //     // cleanup resources if needed
    // }

    pub fn createIdentity(id: u32, name: []const u8, active: bool, token: []const u32) Identity {
        return Identity{
            .id = id,
            .name = name,
            .active = active,
            .token = token,
        };
    }

    pub fn createAccount(id: u32, name: []const u8, active: bool, token: []const u32) Account {
        return Account{
            .id = id,
            .name = name,
            .active = active,
            .token = token,
        };
    }

    pub fn createProfile(id: u32, name: []const u8) Profile {
        return Profile{
            .id = id,
            .name = name,
        };
    }

    pub fn createUser(id: u32, name: []const u8, active: bool, token: []const u32) User {
        return User{
            .id = id,
            .name = name,
            .active = active,
            .token = token,
        };
    }

    pub fn createContact(id: u32, name: []const u8) Contact {
        return Contact{
            .id = id,
            .name = name,
        };
    }
};
