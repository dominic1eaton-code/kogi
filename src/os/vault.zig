//! KOGI Vault Module - Asset Management in the KOGI OS
//! The Vault manages resources, assets, equipment, and capital.
//! Analogous to filesystem storage and resource management in traditional OS.

const std = @import("std");

/// Asset types managed in the vault
pub const VaultItemType = enum {
    equipment,
    software,
    intellectual_property,
    furniture,
    vehicle,
    other,
};

/// Asset/resource in the vault
pub const VaultItem = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    item_type: VaultItemType,
    value: f32,
    acquired_date: i64,
};

/// Vault manages all assets and resources
pub const Vault = struct {
    allocator: std.mem.Allocator,
    items: std.array_list.Managed(VaultItem),

    pub fn init(allocator: std.mem.Allocator) Vault {
        return Vault{
            .allocator = allocator,
            .items = std.array_list.Managed(VaultItem).init(allocator),
        };
    }

    pub fn deinit(self: *Vault) void {
        for (self.items.items) |item| {
            self.allocator.free(item.name);
            self.allocator.free(item.description);
        }
        self.items.deinit();
    }

    /// Add an asset to the vault
    pub fn addItem(
        self: *Vault,
        name: []const u8,
        description: []const u8,
        item_type: VaultItemType,
        value: f32,
        acquired_date: i64,
    ) !u32 {
        const item_id = @as(u32, @intCast(self.items.items.len));

        const item = VaultItem{
            .id = item_id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .item_type = item_type,
            .value = value,
            .acquired_date = acquired_date,
        };

        try self.items.append(item);
        return item_id;
    }

    /// Get total vault value
    pub fn getTotalValue(self: *Vault) f32 {
        var total: f32 = 0.0;
        for (self.items.items) |item| {
            total += item.value;
        }
        return total;
    }
};

// Backward compatibility aliases
pub const Asset = VaultItem;
pub const AssetType = VaultItemType;
pub const AssetManager = Vault;
