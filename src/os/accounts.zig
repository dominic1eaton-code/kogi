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
        for (self.accounts.items) |account| {
            self.allocator.free(account.provider_slug);
            self.allocator.free(account.account_handle);
            self.allocator.free(account.metadata);
        }
        self.accounts.deinit();
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
        const account = ManagedAccount{
            .id = id,
            .owner_user_id = owner_user_id,
            .profile_id = null,
            .account_type = account_type,
            .provider_slug = try self.allocator.dupe(u8, provider_slug),
            .account_handle = try self.allocator.dupe(u8, account_handle),
            .metadata = try self.allocator.dupe(u8, metadata),
            .active = true,
            .created_at = now,
            .updated_at = now,
        };
        errdefer {
            self.allocator.free(account.provider_slug);
            self.allocator.free(account.account_handle);
            self.allocator.free(account.metadata);
        }
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
