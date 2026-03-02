const std = @import("std");

/// A general-purpose account belonging to a worker.
/// Different categories might include social_media, work, personal, email, software, etc.
pub const AccountType = enum {
    social_media,
    work,
    personal,
    email,
    software,
    other,
};

pub const Account = struct {
    id: u32,
    account_type: AccountType,
    username: []const u8,
    provider: []const u8,
    details: []const u8, // optional free-form details or URL
    active: bool,
};

/// Manages a collection of accounts for a single worker.
pub const AccountManager = struct {
    allocator: std.mem.Allocator,
    accounts: std.ArrayList(Account),

    pub fn init(allocator: std.mem.Allocator) AccountManager {
        return AccountManager{
            .allocator = allocator,
            .accounts = std.ArrayList(Account){},
        };
    }

    pub fn deinit(self: *AccountManager) void {
        for (self.accounts.items) |*acct| {
            self.allocator.free(acct.username);
            self.allocator.free(acct.provider);
            self.allocator.free(acct.details);
        }
        self.accounts.deinit(self.allocator);
    }

    /// Add a new account
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
        try self.accounts.append(self.allocator, account);
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
            if (std.mem.eql(u8, acct.provider, provider)) {
                return acct;
            }
        }
        return null;
    }
};

// simple demo
pub fn accountsDemo() void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var mgr = AccountManager.init(allocator);
    defer mgr.deinit();

    _ = try mgr.addAccount(AccountType.social_media, "alice123", "twitter", "https://twitter.com/alice");
    _ = try mgr.addAccount(AccountType.email, "alice@example.com", "gmail", "primary email");
    std.debug.print("Created {d} accounts\n", .{mgr.accounts.items.len});
}
