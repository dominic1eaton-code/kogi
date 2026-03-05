const std = @import("std");

pub const ProfileType = enum {
    work,
    personal,
    custom,
};

pub const ProfileProperty = struct {
    key: []const u8,
    value: []const u8,
};

pub const ManagedProfile = struct {
    id: u32,
    owner_user_id: u32,
    profile_type: ProfileType,
    name: []const u8,
    description: []const u8,
    configurations: std.array_list.Managed(ProfileProperty),
    preferences: std.array_list.Managed(ProfileProperty),
    account_ids: std.array_list.Managed(u32),
    active: bool,
    created_at: i64,
    updated_at: i64,
};

pub const ProfileManagementError = error{
    ProfileNotFound,
    ProfileOwnershipMismatch,
};

pub const ProfileManagement = struct {
    allocator: std.mem.Allocator,
    profiles: std.array_list.Managed(ManagedProfile),
    next_profile_id: u32,

    pub fn init(allocator: std.mem.Allocator) ProfileManagement {
        return ProfileManagement{
            .allocator = allocator,
            .profiles = std.array_list.Managed(ManagedProfile).init(allocator),
            .next_profile_id = 0,
        };
    }

    pub fn deinit(self: *ProfileManagement) void {
        for (self.profiles.items) |*profile| {
            self.allocator.free(profile.name);
            self.allocator.free(profile.description);
            freeProperties(self.allocator, profile.configurations.items);
            freeProperties(self.allocator, profile.preferences.items);
            profile.configurations.deinit();
            profile.preferences.deinit();
            profile.account_ids.deinit();
        }
        self.profiles.deinit();
    }

    pub fn createProfile(
        self: *ProfileManagement,
        owner_user_id: u32,
        profile_type: ProfileType,
        name: []const u8,
        description: []const u8,
    ) !u32 {
        const profile_id = self.next_profile_id;
        self.next_profile_id += 1;
        var profile = ManagedProfile{
            .id = profile_id,
            .owner_user_id = owner_user_id,
            .profile_type = profile_type,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .configurations = std.array_list.Managed(ProfileProperty).init(self.allocator),
            .preferences = std.array_list.Managed(ProfileProperty).init(self.allocator),
            .account_ids = std.array_list.Managed(u32).init(self.allocator),
            .active = false,
            .created_at = std.time.timestamp(),
            .updated_at = std.time.timestamp(),
        };
        errdefer {
            self.allocator.free(profile.name);
            self.allocator.free(profile.description);
            profile.configurations.deinit();
            profile.preferences.deinit();
            profile.account_ids.deinit();
        }

        if (self.findActiveProfileIndexForUser(owner_user_id) == null) {
            profile.active = true;
        }
        try self.profiles.append(profile);
        return profile_id;
    }

    pub fn setActiveProfile(self: *ProfileManagement, owner_user_id: u32, profile_id: u32) !void {
        const idx = self.findProfileIndexById(profile_id) orelse return ProfileManagementError.ProfileNotFound;
        if (self.profiles.items[idx].owner_user_id != owner_user_id) return ProfileManagementError.ProfileOwnershipMismatch;

        for (self.profiles.items) |*profile| {
            if (profile.owner_user_id == owner_user_id) profile.active = false;
        }
        self.profiles.items[idx].active = true;
        self.profiles.items[idx].updated_at = std.time.timestamp();
    }

    pub fn setConfiguration(self: *ProfileManagement, profile_id: u32, key: []const u8, value: []const u8) !void {
        const idx = self.findProfileIndexById(profile_id) orelse return ProfileManagementError.ProfileNotFound;
        try upsertProperty(self.allocator, &self.profiles.items[idx].configurations, key, value);
        self.profiles.items[idx].updated_at = std.time.timestamp();
    }

    pub fn setPreference(self: *ProfileManagement, profile_id: u32, key: []const u8, value: []const u8) !void {
        const idx = self.findProfileIndexById(profile_id) orelse return ProfileManagementError.ProfileNotFound;
        try upsertProperty(self.allocator, &self.profiles.items[idx].preferences, key, value);
        self.profiles.items[idx].updated_at = std.time.timestamp();
    }

    pub fn updateProfileMetadata(self: *ProfileManagement, profile_id: u32, name: []const u8, description: []const u8) !void {
        const idx = self.findProfileIndexById(profile_id) orelse return ProfileManagementError.ProfileNotFound;
        const profile = &self.profiles.items[idx];
        self.allocator.free(profile.name);
        self.allocator.free(profile.description);
        profile.name = try self.allocator.dupe(u8, name);
        profile.description = try self.allocator.dupe(u8, description);
        profile.updated_at = std.time.timestamp();
    }

    pub fn linkAccount(self: *ProfileManagement, profile_id: u32, account_id: u32) !void {
        const idx = self.findProfileIndexById(profile_id) orelse return ProfileManagementError.ProfileNotFound;
        for (self.profiles.items[idx].account_ids.items) |id| {
            if (id == account_id) return;
        }
        try self.profiles.items[idx].account_ids.append(account_id);
        self.profiles.items[idx].updated_at = std.time.timestamp();
    }

    pub fn unlinkAccount(self: *ProfileManagement, profile_id: u32, account_id: u32) !void {
        const idx = self.findProfileIndexById(profile_id) orelse return ProfileManagementError.ProfileNotFound;
        var i: usize = 0;
        while (i < self.profiles.items[idx].account_ids.items.len) : (i += 1) {
            if (self.profiles.items[idx].account_ids.items[i] == account_id) {
                _ = self.profiles.items[idx].account_ids.orderedRemove(i);
                self.profiles.items[idx].updated_at = std.time.timestamp();
                return;
            }
        }
    }

    pub fn getProfiles(self: *ProfileManagement) []ManagedProfile {
        return self.profiles.items;
    }

    pub fn getProfileById(self: *ProfileManagement, profile_id: u32) !ManagedProfile {
        const idx = self.findProfileIndexById(profile_id) orelse return ProfileManagementError.ProfileNotFound;
        return self.profiles.items[idx];
    }

    pub fn getActiveProfile(self: *ProfileManagement, owner_user_id: u32) ?ManagedProfile {
        for (self.profiles.items) |profile| {
            if (profile.owner_user_id == owner_user_id and profile.active) return profile;
        }
        return null;
    }

    fn findProfileIndexById(self: *ProfileManagement, profile_id: u32) ?usize {
        for (self.profiles.items, 0..) |profile, idx| {
            if (profile.id == profile_id) return idx;
        }
        return null;
    }

    fn findActiveProfileIndexForUser(self: *ProfileManagement, owner_user_id: u32) ?usize {
        for (self.profiles.items, 0..) |profile, idx| {
            if (profile.owner_user_id == owner_user_id and profile.active) return idx;
        }
        return null;
    }
};

fn upsertProperty(
    allocator: std.mem.Allocator,
    list: *std.array_list.Managed(ProfileProperty),
    key: []const u8,
    value: []const u8,
) !void {
    for (list.items) |*entry| {
        if (std.mem.eql(u8, entry.key, key)) {
            allocator.free(entry.value);
            entry.value = try allocator.dupe(u8, value);
            return;
        }
    }
    const entry = ProfileProperty{
        .key = try allocator.dupe(u8, key),
        .value = try allocator.dupe(u8, value),
    };
    try list.append(entry);
}

fn freeProperties(allocator: std.mem.Allocator, props: []ProfileProperty) void {
    for (props) |entry| {
        allocator.free(entry.key);
        allocator.free(entry.value);
    }
}

pub fn parseProfileType(name: []const u8) ?ProfileType {
    if (std.mem.eql(u8, name, "work")) return .work;
    if (std.mem.eql(u8, name, "personal")) return .personal;
    if (std.mem.eql(u8, name, "custom")) return .custom;
    return null;
}

test "profile management lifecycle" {
    const allocator = std.testing.allocator;
    var mgr = ProfileManagement.init(allocator);
    defer mgr.deinit();

    const p1 = try mgr.createProfile(7, .work, "work", "work profile");
    const p2 = try mgr.createProfile(7, .custom, "streamer", "streaming profile");

    try mgr.setConfiguration(p2, "theme", "dark");
    try mgr.setPreference(p2, "notifications", "mentions-only");
    try mgr.linkAccount(p2, 12);
    try mgr.setActiveProfile(7, p2);

    const active = mgr.getActiveProfile(7);
    try std.testing.expect(active != null);
    try std.testing.expectEqual(p2, active.?.id);
    try std.testing.expectEqual(@as(usize, 1), active.?.account_ids.items.len);
    _ = p1;
}
