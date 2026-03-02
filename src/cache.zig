//! KOGI Tiered Caching & Memory Management System
//! Provides a tiered allocator with caching, garbage detection, and leak tracking.

const std = @import("std");

/// Memory tiers
pub const MemoryTier = enum {
    L1,
    L2,
    L3,
};

/// Allocation record for leak detection
const AllocRecord = struct {
    ptr: ?*u8,
    size: usize,
    tier: MemoryTier,
    timestamp: i64,
};

/// Tiered allocator implementing std.mem.Allocator interface
pub const TieredAllocator = struct {
    // underlying allocators per tier
    allocators: [3]std.mem.Allocator,
    // tracking for leak detection
    records: std.AutoHashMap(*u8, AllocRecord),
    mutex: std.Thread.Mutex = .{},

    pub fn init(allocator: std.mem.Allocator) !TieredAllocator {
        const ta = TieredAllocator{
            .allocators = .{ allocator, allocator, allocator },
            .records = std.AutoHashMap(*u8, AllocRecord).init(allocator),
        };
        return ta;
    }

    pub fn deinit(self: *TieredAllocator) void {
        // report leaks
        var iter = self.records.iterator();
        while (iter.next()) |entry| {
            const rec = entry.value_ptr.*;
            std.debug.print("Leak detected: ptr={?} size={}\n", .{ rec.ptr, rec.size });
        }
        self.records.deinit();
    }

    pub fn allocate(self: *TieredAllocator, len: usize, alignment: u29) ?*u8 {
        // choose tier based on length
        const tier = if (len < 256) MemoryTier.L1 else if (len < 4096) MemoryTier.L2 else MemoryTier.L3;

        self.mutex.lock();
        defer self.mutex.unlock();

        // convert u29 to Alignment
        const align_enum = std.mem.Alignment.fromByteUnits(alignment);

        // use rawAlloc to call the vtable directly
        const tier_idx = @as(usize, @intFromEnum(tier));
        const alloc_result = self.allocators[tier_idx].rawAlloc(len, align_enum, @returnAddress());

        if (alloc_result) |mem| {
            // mem is [*]u8, need to cast to *u8
            const ptr: *u8 = @ptrCast(mem);
            self.records.put(ptr, AllocRecord{ .ptr = ptr, .size = len, .tier = tier, .timestamp = std.time.timestamp() }) catch {};
            return ptr;
        }
        return null;
    }

    pub fn free(self: *TieredAllocator, ptr: *u8, _: usize, _: u29) void {
        self.mutex.lock();
        defer self.mutex.unlock();
        // remove record
        _ = self.records.remove(ptr);
    }

    /// Returns an std.mem.Allocator wrapper for this tiered allocator
    pub fn asAllocator(self: *TieredAllocator) std.mem.Allocator {
        return std.mem.Allocator{
            .ptr = self,
            .vtable = &allocator_vtable,
        };
    }
};

const allocator_vtable = std.mem.Allocator.VTable{
    .alloc = allocatorVtableFn,
    .resize = std.mem.Allocator.noResize,
    .remap = std.mem.Allocator.noRemap,
    .free = freeVtableFn,
};

fn allocatorVtableFn(ctx: *anyopaque, len: usize, alignment: std.mem.Alignment, ret_addr: usize) ?[*]u8 {
    const self: *TieredAllocator = @ptrCast(@alignCast(ctx));
    _ = ret_addr;
    // convert Alignment enum to u29 value with cast
    const align_val: u29 = @intCast(alignment.toByteUnits());
    const result = self.allocate(len, align_val);
    if (result) |ptr| {
        return @ptrCast(ptr);
    }
    return null;
}

fn freeVtableFn(ctx: *anyopaque, memory: []u8, alignment: std.mem.Alignment, ret_addr: usize) void {
    const self: *TieredAllocator = @ptrCast(@alignCast(ctx));
    _ = ret_addr;
    // convert Alignment enum to u29 value with cast
    const align_val: u29 = @intCast(alignment.toByteUnits());
    if (memory.len > 0) {
        // convert [*]u8 to *u8
        const ptr: *u8 = @ptrCast(memory.ptr);
        self.free(ptr, memory.len, align_val);
    }
}
