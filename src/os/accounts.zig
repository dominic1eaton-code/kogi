const std = @import("std");

/// Account categories shared across lightweight and managed account systems.
pub const AccountType = enum {
    social_media,
    work,
    personal,
    email,
    software,
    other,
};

pub const AccountProfile = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    created_at: i64,
    updated_at: i64,
};

pub const AccountSetting = struct {
    key: []const u8,
    value: []const u8,
    updated_at: i64,
};

pub const AccountOption = struct {
    name: []const u8,
    value: []const u8,
    enabled: bool,
    updated_at: i64,
};

pub const AccountParameter = struct {
    key: []const u8,
    value: []const u8,
    updated_at: i64,
};

pub const AccountKey = struct {
    id: u32,
    label: []const u8,
    material: []const u8,
    created_at: i64,
    updated_at: i64,
};

pub const AccountToken = struct {
    id: u32,
    label: []const u8,
    value: []const u8,
    created_at: i64,
    updated_at: i64,
    expires_at: ?i64 = null,
    revoked: bool = false,
};

// ========== Lightweight Account System ==========

pub const Account = struct {
    id: u32,
    account_type: AccountType,
    username: []const u8,
    provider: []const u8,
    details: []const u8,
    active: bool,
};

pub const AccountManager = struct {
    allocator: std.mem.Allocator,
    accounts: std.array_list.Managed(Account),

    pub fn init(allocator: std.mem.Allocator) AccountManager {
        return AccountManager{
            .allocator = allocator,
            .accounts = std.array_list.Managed(Account).init(allocator),
        };
    }

    pub fn deinit(self: *AccountManager) void {
        for (self.accounts.items) |acct| {
            self.allocator.free(acct.username);
            self.allocator.free(acct.provider);
            self.allocator.free(acct.details);
        }
        self.accounts.deinit();
    }

    pub fn addAccount(
        self: *AccountManager,
        acct_type: AccountType,
        username: []const u8,
        provider: []const u8,
        details: []const u8,
    ) !u32 {
        const id = @as(u32, @intCast(self.accounts.items.len));
        const account = Account{
            .id = id,
            .account_type = acct_type,
            .username = try self.allocator.dupe(u8, username),
            .provider = try self.allocator.dupe(u8, provider),
            .details = try self.allocator.dupe(u8, details),
            .active = true,
        };
        try self.accounts.append(account);
        return id;
    }

    pub fn deactivateAccount(self: *AccountManager, id: u32) !void {
        if (id < @as(u32, @intCast(self.accounts.items.len))) {
            self.accounts.items[id].active = false;
        }
    }

    pub fn getActiveCount(self: *AccountManager) u32 {
        var count: u32 = 0;
        for (self.accounts.items) |acct| {
            if (acct.active) count += 1;
        }
        return count;
    }

    pub fn findByProvider(self: *AccountManager, provider: []const u8) ?*Account {
        for (self.accounts.items) |*acct| {
            if (std.mem.eql(u8, acct.provider, provider)) return acct;
        }
        return null;
    }
};

// ========== Managed Account System ==========

pub const ManagedAccount = struct {
    id: u32,
    owner_user_id: u32,
    profile_id: ?u32,
    account_type: AccountType,
    provider_slug: []const u8,
    account_handle: []const u8,
    metadata: []const u8,
    profiles: std.array_list.Managed(AccountProfile),
    settings: std.array_list.Managed(AccountSetting),
    options: std.array_list.Managed(AccountOption),
    parameters: std.array_list.Managed(AccountParameter),
    keys: std.array_list.Managed(AccountKey),
    tokens: std.array_list.Managed(AccountToken),
    linked_account_ids: std.array_list.Managed(u32),
    active: bool,
    created_at: i64,
    updated_at: i64,
};

pub const AccountManagementError = error{
    AccountNotFound,
};

pub const AccountManagement = struct {
    allocator: std.mem.Allocator,
    accounts: std.array_list.Managed(ManagedAccount),
    next_account_id: u32,

    pub fn init(allocator: std.mem.Allocator) AccountManagement {
        return AccountManagement{
            .allocator = allocator,
            .accounts = std.array_list.Managed(ManagedAccount).init(allocator),
            .next_account_id = 0,
        };
    }

    pub fn deinit(self: *AccountManagement) void {
        for (self.accounts.items) |*account| {
            self.deinitManagedAccount(account);
        }
        self.accounts.deinit();
    }

    fn deinitManagedAccount(self: *AccountManagement, account: *ManagedAccount) void {
        self.allocator.free(account.provider_slug);
        self.allocator.free(account.account_handle);
        self.allocator.free(account.metadata);

        for (account.profiles.items) |*profile| {
            self.allocator.free(profile.name);
            self.allocator.free(profile.description);
        }
        account.profiles.deinit();

        for (account.settings.items) |*setting| {
            self.allocator.free(setting.key);
            self.allocator.free(setting.value);
        }
        account.settings.deinit();

        for (account.options.items) |*option| {
            self.allocator.free(option.name);
            self.allocator.free(option.value);
        }
        account.options.deinit();

        for (account.parameters.items) |*parameter| {
            self.allocator.free(parameter.key);
            self.allocator.free(parameter.value);
        }
        account.parameters.deinit();

        for (account.keys.items) |*key| {
            self.allocator.free(key.label);
            self.allocator.free(key.material);
        }
        account.keys.deinit();

        for (account.tokens.items) |*token| {
            self.allocator.free(token.label);
            self.allocator.free(token.value);
        }
        account.tokens.deinit();

        account.linked_account_ids.deinit();
    }

    pub fn createAccount(
        self: *AccountManagement,
        owner_user_id: u32,
        account_type: AccountType,
        provider_slug: []const u8,
        account_handle: []const u8,
        metadata: []const u8,
    ) !u32 {
        const id = self.next_account_id;
        self.next_account_id += 1;
        const now = std.time.timestamp();
        var account = ManagedAccount{
            .id = id,
            .owner_user_id = owner_user_id,
            .profile_id = null,
            .account_type = account_type,
            .provider_slug = try self.allocator.dupe(u8, provider_slug),
            .account_handle = try self.allocator.dupe(u8, account_handle),
            .metadata = try self.allocator.dupe(u8, metadata),
            .profiles = std.array_list.Managed(AccountProfile).init(self.allocator),
            .settings = std.array_list.Managed(AccountSetting).init(self.allocator),
            .options = std.array_list.Managed(AccountOption).init(self.allocator),
            .parameters = std.array_list.Managed(AccountParameter).init(self.allocator),
            .keys = std.array_list.Managed(AccountKey).init(self.allocator),
            .tokens = std.array_list.Managed(AccountToken).init(self.allocator),
            .linked_account_ids = std.array_list.Managed(u32).init(self.allocator),
            .active = true,
            .created_at = now,
            .updated_at = now,
        };
        errdefer self.deinitManagedAccount(&account);
        try self.accounts.append(account);
        return id;
    }

    pub fn associateToProfile(self: *AccountManagement, account_id: u32, profile_id: ?u32) !void {
        const idx = self.findAccountIndexById(account_id) orelse return AccountManagementError.AccountNotFound;
        self.accounts.items[idx].profile_id = profile_id;
        self.accounts.items[idx].updated_at = std.time.timestamp();
    }

    pub fn deactivateAccount(self: *AccountManagement, account_id: u32) !void {
        const idx = self.findAccountIndexById(account_id) orelse return AccountManagementError.AccountNotFound;
        self.accounts.items[idx].active = false;
        self.accounts.items[idx].updated_at = std.time.timestamp();
    }

    pub fn addProfile(self: *AccountManagement, account_id: u32, name: []const u8, description: []const u8) !u32 {
        const idx = self.findAccountIndexById(account_id) orelse return AccountManagementError.AccountNotFound;
        const account = &self.accounts.items[idx];
        const profile_id = @as(u32, @intCast(account.profiles.items.len));
        const now = std.time.timestamp();

        const profile = AccountProfile{
            .id = profile_id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .created_at = now,
            .updated_at = now,
        };
        errdefer {
            self.allocator.free(profile.name);
            self.allocator.free(profile.description);
        }

        try account.profiles.append(profile);
        account.updated_at = now;
        return profile_id;
    }

    pub fn setSetting(self: *AccountManagement, account_id: u32, key: []const u8, value: []const u8) !void {
        const idx = self.findAccountIndexById(account_id) orelse return AccountManagementError.AccountNotFound;
        const account = &self.accounts.items[idx];
        const now = std.time.timestamp();

        for (account.settings.items) |*setting| {
            if (!std.mem.eql(u8, setting.key, key)) continue;
            const new_value = try self.allocator.dupe(u8, value);
            self.allocator.free(setting.value);
            setting.value = new_value;
            setting.updated_at = now;
            account.updated_at = now;
            return;
        }

        const setting = AccountSetting{
            .key = try self.allocator.dupe(u8, key),
            .value = try self.allocator.dupe(u8, value),
            .updated_at = now,
        };
        errdefer {
            self.allocator.free(setting.key);
            self.allocator.free(setting.value);
        }

        try account.settings.append(setting);
        account.updated_at = now;
    }

    pub fn setOption(self: *AccountManagement, account_id: u32, name: []const u8, value: []const u8, enabled: bool) !void {
        const idx = self.findAccountIndexById(account_id) orelse return AccountManagementError.AccountNotFound;
        const account = &self.accounts.items[idx];
        const now = std.time.timestamp();

        for (account.options.items) |*option| {
            if (!std.mem.eql(u8, option.name, name)) continue;
            const new_value = try self.allocator.dupe(u8, value);
            self.allocator.free(option.value);
            option.value = new_value;
            option.enabled = enabled;
            option.updated_at = now;
            account.updated_at = now;
            return;
        }

        const option = AccountOption{
            .name = try self.allocator.dupe(u8, name),
            .value = try self.allocator.dupe(u8, value),
            .enabled = enabled,
            .updated_at = now,
        };
        errdefer {
            self.allocator.free(option.name);
            self.allocator.free(option.value);
        }

        try account.options.append(option);
        account.updated_at = now;
    }

    pub fn setParameter(self: *AccountManagement, account_id: u32, key: []const u8, value: []const u8) !void {
        const idx = self.findAccountIndexById(account_id) orelse return AccountManagementError.AccountNotFound;
        const account = &self.accounts.items[idx];
        const now = std.time.timestamp();

        for (account.parameters.items) |*parameter| {
            if (!std.mem.eql(u8, parameter.key, key)) continue;
            const new_value = try self.allocator.dupe(u8, value);
            self.allocator.free(parameter.value);
            parameter.value = new_value;
            parameter.updated_at = now;
            account.updated_at = now;
            return;
        }

        const parameter = AccountParameter{
            .key = try self.allocator.dupe(u8, key),
            .value = try self.allocator.dupe(u8, value),
            .updated_at = now,
        };
        errdefer {
            self.allocator.free(parameter.key);
            self.allocator.free(parameter.value);
        }

        try account.parameters.append(parameter);
        account.updated_at = now;
    }

    pub fn addKey(self: *AccountManagement, account_id: u32, label: []const u8, material: []const u8) !u32 {
        const idx = self.findAccountIndexById(account_id) orelse return AccountManagementError.AccountNotFound;
        const account = &self.accounts.items[idx];
        const key_id = @as(u32, @intCast(account.keys.items.len));
        const now = std.time.timestamp();

        const key = AccountKey{
            .id = key_id,
            .label = try self.allocator.dupe(u8, label),
            .material = try self.allocator.dupe(u8, material),
            .created_at = now,
            .updated_at = now,
        };
        errdefer {
            self.allocator.free(key.label);
            self.allocator.free(key.material);
        }

        try account.keys.append(key);
        account.updated_at = now;
        return key_id;
    }

    pub fn addToken(self: *AccountManagement, account_id: u32, label: []const u8, value: []const u8, expires_at: ?i64) !u32 {
        const idx = self.findAccountIndexById(account_id) orelse return AccountManagementError.AccountNotFound;
        const account = &self.accounts.items[idx];
        const token_id = @as(u32, @intCast(account.tokens.items.len));
        const now = std.time.timestamp();

        const token = AccountToken{
            .id = token_id,
            .label = try self.allocator.dupe(u8, label),
            .value = try self.allocator.dupe(u8, value),
            .created_at = now,
            .updated_at = now,
            .expires_at = expires_at,
            .revoked = false,
        };
        errdefer {
            self.allocator.free(token.label);
            self.allocator.free(token.value);
        }

        try account.tokens.append(token);
        account.updated_at = now;
        return token_id;
    }

    pub fn linkAccount(self: *AccountManagement, account_id: u32, linked_account_id: u32) !void {
        const idx = self.findAccountIndexById(account_id) orelse return AccountManagementError.AccountNotFound;
        const account = &self.accounts.items[idx];

        for (account.linked_account_ids.items) |existing_id| {
            if (existing_id == linked_account_id) return;
        }

        try account.linked_account_ids.append(linked_account_id);
        account.updated_at = std.time.timestamp();
    }

    pub fn getAccounts(self: *AccountManagement) []ManagedAccount {
        return self.accounts.items;
    }

    fn findAccountIndexById(self: *AccountManagement, account_id: u32) ?usize {
        for (self.accounts.items, 0..) |account, idx| {
            if (account.id == account_id) return idx;
        }
        return null;
    }
};

pub fn parseAccountType(name: []const u8) ?AccountType {
    if (std.mem.eql(u8, name, "social_media")) return .social_media;
    if (std.mem.eql(u8, name, "work")) return .work;
    if (std.mem.eql(u8, name, "personal")) return .personal;
    if (std.mem.eql(u8, name, "email")) return .email;
    if (std.mem.eql(u8, name, "software")) return .software;
    if (std.mem.eql(u8, name, "other")) return .other;
    return null;
}

pub fn accountsDemo() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var mgr = AccountManager.init(allocator);
    defer mgr.deinit();

    _ = try mgr.addAccount(.social_media, "alice123", "twitter", "https://twitter.com/alice");
    _ = try mgr.addAccount(.email, "alice@example.com", "gmail", "primary email");
    std.debug.print("Created {d} accounts\n", .{mgr.accounts.items.len});
}

test "account management lifecycle" {
    const allocator = std.testing.allocator;
    var mgr = AccountManagement.init(allocator);
    defer mgr.deinit();

    const account_id = try mgr.createAccount(7, .software, "github", "alice-dev", "primary");
    try std.testing.expectEqual(@as(u32, 0), account_id);
    try std.testing.expectEqual(@as(usize, 1), mgr.getAccounts().len);

    try mgr.associateToProfile(account_id, 2);
    try std.testing.expect(mgr.getAccounts()[0].profile_id != null);
    try std.testing.expectEqual(@as(u32, 2), mgr.getAccounts()[0].profile_id.?);
}

test "managed account stores profiles settings options parameters keys tokens and links" {
    const allocator = std.testing.allocator;
    var mgr = AccountManagement.init(allocator);
    defer mgr.deinit();

    const primary = try mgr.createAccount(9, .personal, "kogi", "alice", "root");
    const secondary = try mgr.createAccount(9, .software, "github", "alice-dev", "linked");

    _ = try mgr.addProfile(primary, "root", "Root profile");
    try mgr.setSetting(primary, "theme", "light");
    try mgr.setOption(primary, "notifications", "enabled", true);
    try mgr.setParameter(primary, "workspace_id", "0");
    _ = try mgr.addKey(primary, "bootstrap", "material");
    _ = try mgr.addToken(primary, "session", "token-value", null);
    try mgr.linkAccount(primary, secondary);

    const account = mgr.getAccounts()[0];
    try std.testing.expectEqual(@as(usize, 1), account.profiles.items.len);
    try std.testing.expectEqual(@as(usize, 1), account.settings.items.len);
    try std.testing.expectEqual(@as(usize, 1), account.options.items.len);
    try std.testing.expectEqual(@as(usize, 1), account.parameters.items.len);
    try std.testing.expectEqual(@as(usize, 1), account.keys.items.len);
    try std.testing.expectEqual(@as(usize, 1), account.tokens.items.len);
    try std.testing.expectEqual(@as(usize, 1), account.linked_account_ids.items.len);
    try std.testing.expectEqual(secondary, account.linked_account_ids.items[0]);
}
