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

pub const Credential = struct {
    id: u32,
    name: []const u8,
    active: bool,
    token: []const u32,
};

pub const UserManager = struct {
    allocator: std.mem.Allocator,
    users: std.ArrayList(User),
    identities: std.ArrayList(Identity),
    accounts: std.ArrayList(Account),
    profiles: std.ArrayList(Profile),
    contacts: std.ArrayList(Contact),
    credentials: std.ArrayList(Credential),

    pub fn init(allocator: std.mem.Allocator) UserManager {
        return UserManager{
            .allocator = allocator,
            .users = std.ArrayList(User).init(allocator),
            .identities = std.ArrayList(Identity).init(allocator),
            .accounts = std.ArrayList(Account).init(allocator),
            .profiles = std.ArrayList(Profile).init(allocator),
            .contacts = std.ArrayList(Contact).init(allocator),
            .credentials = std.ArrayList(Credential).init(allocator),
        };
    }

    pub fn deinit(self: *UserManager) void {
        for (self.users) |user| {
            self.allocator.free(user.name);
        }
    }

    pub fn loginUser(self: *UserManager, name: []const u8, token: []const u32) ?u32 {
        for (self.users) |user| {
            if (std.mem.eql(u8, user.name, name) and std.mem.eql(u32, user.token, token)) {
                return user.id;
            }
        }
        return null;
    }

    pub fn logoutUser(self: *UserManager, id: u32) bool {
        for (self.users) |user| {
            if (user.id == id) {
                user.active = false;
                return true;
            }
        }
        return false;
    }

    pub fn disableUser(self: *UserManager, id: u32) bool {
        for (self.users) |user| {
            if (user.id == id) {
                user.active = false;
                return true;
            }
        }
        return false;
    }

    pub fn enableUser(self: *UserManager, id: u32) bool {
        for (self.users) |user| {
            if (user.id == id) {
                user.active = true;
                return true;
            }
        }
        return false;
    }

    pub fn getCurrentUser(self: *UserManager) ?User {
        for (self.users) |user| {
            if (user.active) {
                return user;
            }
        }
        return null;
    }

    pub fn getUserById(self: *UserManager, id: u32) ?User {
        for (self.users) |user| {
            if (user.id == id) {
                return user;
            }
        }
        return null;
    }

    pub fn createUser(self: *UserManager, id: u32, name: []const u8, active: bool, token: []const u32) !u32 {
        const user = User{
            .id = id,
            .name = name,
            .active = active,
            .token = token,
        };

        try self.users.append(user);
        return user.id;
    }

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

    pub fn createContact(id: u32, name: []const u8) Contact {
        return Contact{
            .id = id,
            .name = name,
        };
    }
};
