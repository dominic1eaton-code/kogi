const std = @import("std");

pub const AssetType = enum {
    equipment,
    software,
    intellectual_property,
    furniture,
    vehicle,
    other,
};

pub const Asset = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    asset_type: AssetType,
    value: f64,
    acquired_date: i64,
};

pub const AssetManager = struct {
    allocator: std.mem.Allocator,
    assets: std.array_list.Managed(Asset),

    pub fn init(allocator: std.mem.Allocator) AssetManager {
        return AssetManager{
            .allocator = allocator,
            .assets = std.array_list.Managed(Asset).init(allocator),
        };
    }

    pub fn deinit(self: *AssetManager) void {
        for (self.assets.items) |*a| {
            self.allocator.free(a.name);
            self.allocator.free(a.description);
        }
        self.assets.deinit();
    }

    pub fn addAsset(
        self: *AssetManager,
        name: []const u8,
        description: []const u8,
        asset_type: AssetType,
        value: f64,
        acquired_date: i64,
    ) !u32 {
        const id = @as(u32, @intCast(self.assets.items.len));
        const asset = Asset{
            .id = id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .asset_type = asset_type,
            .value = value,
            .acquired_date = acquired_date,
        };
        try self.assets.append(asset);
        return id;
    }

    pub fn getTotalValue(self: *AssetManager) f64 {
        var total: f64 = 0.0;
        for (self.assets.items) |asset| {
            total += asset.value;
        }
        return total;
    }
};

pub fn assetsDemo() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var mgr = AssetManager.init(allocator);
    defer mgr.deinit();

    _ = try mgr.addAsset("Laptop", "Development laptop", AssetType.equipment, 1500.0, 1672531200);
    _ = try mgr.addAsset("Domain", "kogi.app", AssetType.intellectual_property, 200.0, 1672617600);
    std.debug.print("Total asset value: ${:.2}\n", .{mgr.getTotalValue()});
}
