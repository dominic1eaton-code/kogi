const std = @import("std");

pub const ProviderCategory = enum {
    social_media,
    email,
    code_hosting,
    productivity,
    communication,
    payments,
    storage,
    other,
};

pub const Provider = struct {
    id: u32,
    name: []const u8,
    slug: []const u8,
    category: ProviderCategory,
    homepage_url: []const u8,
    api_base_url: []const u8,
    active: bool,
    created_at: i64,
    updated_at: i64,
};

pub const ProviderError = error{
    ProviderNotFound,
    ProviderSlugTaken,
};

pub const ProviderManager = struct {
    allocator: std.mem.Allocator,
    providers: std.ArrayList(Provider),
    next_provider_id: u32,

    pub fn init(allocator: std.mem.Allocator) ProviderManager {
        return ProviderManager{
            .allocator = allocator,
            .providers = std.ArrayList(Provider).init(allocator),
            .next_provider_id = 0,
        };
    }

    pub fn deinit(self: *ProviderManager) void {
        for (self.providers.items) |provider| {
            self.allocator.free(provider.name);
            self.allocator.free(provider.slug);
            self.allocator.free(provider.homepage_url);
            self.allocator.free(provider.api_base_url);
        }
        self.providers.deinit();
    }

    pub fn addProvider(
        self: *ProviderManager,
        name: []const u8,
        slug: []const u8,
        category: ProviderCategory,
        homepage_url: []const u8,
        api_base_url: []const u8,
    ) !u32 {
        if (self.findProviderIndexBySlug(slug) != null) {
            return ProviderError.ProviderSlugTaken;
        }

        const id = self.next_provider_id;
        self.next_provider_id += 1;

        const now = std.time.timestamp();
        const provider = Provider{
            .id = id,
            .name = try self.allocator.dupe(u8, name),
            .slug = try self.allocator.dupe(u8, slug),
            .category = category,
            .homepage_url = try self.allocator.dupe(u8, homepage_url),
            .api_base_url = try self.allocator.dupe(u8, api_base_url),
            .active = true,
            .created_at = now,
            .updated_at = now,
        };
        errdefer {
            self.allocator.free(provider.name);
            self.allocator.free(provider.slug);
            self.allocator.free(provider.homepage_url);
            self.allocator.free(provider.api_base_url);
        }

        try self.providers.append(provider);
        return id;
    }

    pub fn disableProvider(self: *ProviderManager, provider_id: u32) !void {
        const idx = self.findProviderIndexById(provider_id) orelse return ProviderError.ProviderNotFound;
        self.providers.items[idx].active = false;
        self.providers.items[idx].updated_at = std.time.timestamp();
    }

    pub fn enableProvider(self: *ProviderManager, provider_id: u32) !void {
        const idx = self.findProviderIndexById(provider_id) orelse return ProviderError.ProviderNotFound;
        self.providers.items[idx].active = true;
        self.providers.items[idx].updated_at = std.time.timestamp();
    }

    pub fn getProviders(self: *ProviderManager) []Provider {
        return self.providers.items;
    }

    pub fn getProvider(self: *ProviderManager, provider_id: u32) !Provider {
        const idx = self.findProviderIndexById(provider_id) orelse return ProviderError.ProviderNotFound;
        return self.providers.items[idx];
    }

    pub fn findBySlug(self: *ProviderManager, slug: []const u8) ?Provider {
        const idx = self.findProviderIndexBySlug(slug) orelse return null;
        return self.providers.items[idx];
    }

    pub fn isActiveBySlug(self: *ProviderManager, slug: []const u8) bool {
        if (self.findBySlug(slug)) |provider| {
            return provider.active;
        }
        return false;
    }

    fn findProviderIndexById(self: *ProviderManager, provider_id: u32) ?usize {
        for (self.providers.items, 0..) |provider, idx| {
            if (provider.id == provider_id) return idx;
        }
        return null;
    }

    fn findProviderIndexBySlug(self: *ProviderManager, slug: []const u8) ?usize {
        for (self.providers.items, 0..) |provider, idx| {
            if (std.mem.eql(u8, provider.slug, slug)) return idx;
        }
        return null;
    }
};

pub fn parseProviderCategory(name: []const u8) ?ProviderCategory {
    if (std.mem.eql(u8, name, "social_media")) return .social_media;
    if (std.mem.eql(u8, name, "email")) return .email;
    if (std.mem.eql(u8, name, "code_hosting")) return .code_hosting;
    if (std.mem.eql(u8, name, "productivity")) return .productivity;
    if (std.mem.eql(u8, name, "communication")) return .communication;
    if (std.mem.eql(u8, name, "payments")) return .payments;
    if (std.mem.eql(u8, name, "storage")) return .storage;
    if (std.mem.eql(u8, name, "other")) return .other;
    return null;
}

test "provider manager add and lookup" {
    const allocator = std.testing.allocator;
    var mgr = ProviderManager.init(allocator);
    defer mgr.deinit();

    const id = try mgr.addProvider(
        "GitHub",
        "github",
        .code_hosting,
        "https://github.com",
        "https://api.github.com",
    );
    try std.testing.expectEqual(@as(u32, 0), id);
    try std.testing.expectEqual(@as(usize, 1), mgr.getProviders().len);

    const found = mgr.findBySlug("github");
    try std.testing.expect(found != null);
    try std.testing.expectEqualStrings("GitHub", found.?.name);
    try std.testing.expect(mgr.isActiveBySlug("github"));
}

test "provider manager disable enable and uniqueness" {
    const allocator = std.testing.allocator;
    var mgr = ProviderManager.init(allocator);
    defer mgr.deinit();

    const id = try mgr.addProvider(
        "LinkedIn",
        "linkedin",
        .social_media,
        "https://linkedin.com",
        "https://api.linkedin.com",
    );
    try std.testing.expectError(
        ProviderError.ProviderSlugTaken,
        mgr.addProvider("LinkedIn Mirror", "linkedin", .social_media, "", ""),
    );

    try mgr.disableProvider(id);
    try std.testing.expect(!mgr.isActiveBySlug("linkedin"));
    try mgr.enableProvider(id);
    try std.testing.expect(mgr.isActiveBySlug("linkedin"));
}
