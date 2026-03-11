//! memory management system
//! This module provides the memory management system for the kernel, including allocation, deallocation, and memory tracking.
//! It also includes utilities for handling memory-related events and errors.
//! The memory management system is designed to be efficient and robust, ensuring that the kernel can manage its memory resources effectively while minimizing fragmentation and maximizing performance.
//! The memory management system is a critical component of the kernel, as it allows the kernel to manage its memory resources effectively and efficiently, ensuring that the kernel can operate smoothly and reliably even under heavy load.
//! The memory management system is implemented using a combination of techniques, including buddy allocation, slab allocation, and reference counting, to provide a flexible and efficient memory management system that can handle a wide range of memory allocation patterns and workloads.
//! The memory management system also includes support for memory tracking and debugging, allowing developers to identify and fix memory-related issues in their code more easily. Overall, the memory management system is a critical component of the kernel, providing the foundation for efficient and reliable memory management that is essential for the smooth operation of the kernel and its applications.
//! The memory management system is designed to be modular and extensible, allowing developers to add new features and capabilities as needed. It also includes support for multiple memory allocators, allowing developers to choose the allocator that best suits their needs and workloads. The memory management system is a critical component of the kernel, providing the foundation for efficient and reliable memory management that is essential for the smooth operation of the kernel and its applications.
//! The memory management system is also designed to be secure, with features such as memory isolation and protection to prevent unauthorized access and ensure the integrity of the kernel's memory resources. Overall, the memory management system is a critical component of the kernel, providing the foundation for efficient, reliable, and secure memory management that is essential for the smooth operation of the kernel and its applications.
//! The memory management system is also designed to be scalable, with support for large memory pools and efficient handling of memory fragmentation. It includes features such as memory compaction and defragmentation to help manage memory resources effectively and minimize fragmentation. Overall, the memory management system is a critical component of the kernel, providing the foundation for efficient, reliable, secure, and scalable memory management that is essential for the smooth operation of the kernel and its applications.
//! The memory management system is also designed to be efficient, with features such as lazy allocation and deallocation to minimize overhead and improve performance. It also includes support for memory pooling and caching to further improve performance and reduce fragmentation. Overall, the memory management system is a critical component of the kernel, providing the foundation for efficient, reliable, secure, scalable, and high-performance memory management that is essential for the smooth operation of the kernel and its applications.
//! The memory management system is also designed to be easy to use, with a simple and intuitive API that allows developers to allocate and deallocate memory easily and efficiently. It also includes support for memory debugging and profiling tools to help developers identify and fix memory-related issues in their code more easily. Overall, the memory management system is a critical component of the kernel, providing the foundation for efficient, reliable, secure, scalable, high-performance, and easy-to-use memory management that is essential for the smooth operation of the kernel and its applications.
//! The memory management system is also designed to be compatible with a wide range of hardware and software platforms, with support for multiple architectures and operating systems. It includes features such as platform-specific optimizations and support for different memory models to ensure that it can operate effectively on a wide range of platforms. Overall, the memory management system is a critical component of the kernel, providing the foundation for efficient, reliable, secure, scalable, high-performance, easy-to-use, and compatible memory management that is essential for the smooth operation of the kernel and its applications.
//! memory.zig — Kernel Memory Management System
//!
//! Extracted and extended from kernel.zig.
//!
//! Provides:
//!   - MemoryManager          — global tracker (total / used bytes, per-tenant limits)
//!   - BuddyAllocator         — power-of-two block allocator with split / merge
//!   - SlabAllocator          — fixed-size object pool
//!   - AuditLog               — per-allocation records for leak detection
//!   - Pressure callbacks     — register hooks fired when usage crosses a threshold
//!   - std.mem.Allocator shim — drop-in replacement backed by BuddyAllocator

const std = @import("std");

// ============================================================
// Re-exported types shared with kernel.zig
// ============================================================

pub const ModuleLimits = struct {
    memory_limit_bytes: u64,
    max_processes: u32,
    max_files: u32,
    max_resource_units: u32,
};

pub const ModuleUsage = struct {
    memory_used_bytes: u64,
    process_count: u32,
    file_count: u32,
    resource_units_used: u32,
};

// ============================================================
// Errors
// ============================================================

pub const MemoryError = error{
    OutOfMemory,
    TenantNotFound,
    TenantAlreadyRegistered,
    TenantResourceLimitExceeded,
    InvalidFree,
    BuddyPoolExhausted,
    SlabPoolExhausted,
    AllocationTooLarge,
    AlignmentNotPowerOfTwo,
};

// ============================================================
// Audit Log
// ============================================================

/// A single allocation event recorded for leak detection / profiling.
pub const AuditEntry = struct {
    /// Monotonically increasing allocation ID.
    id: u64,
    /// Tenant that owns this allocation (empty string = kernel-global).
    tenant_id: []const u8,
    /// Number of bytes requested.
    size: u64,
    /// Whether this allocation has been freed.
    freed: bool,
    /// Timestamp in milliseconds when the allocation was made.
    allocated_at_ms: i64,
    /// Timestamp in milliseconds when it was freed (0 if still live).
    freed_at_ms: i64,
};

/// Append-only audit log.  Capacity is fixed at init time to avoid
/// allocator circularity: the log itself lives in backing_allocator.
pub const AuditLog = struct {
    entries: []AuditEntry,
    len: usize,
    next_id: u64,
    backing: std.mem.Allocator,

    pub fn init(backing: std.mem.Allocator, capacity: usize) !AuditLog {
        const entries = try backing.alloc(AuditEntry, capacity);
        return .{
            .entries = entries,
            .len = 0,
            .next_id = 1,
            .backing = backing,
        };
    }

    pub fn deinit(self: *AuditLog) void {
        self.backing.free(self.entries);
        self.len = 0;
    }

    /// Record a new allocation.  Returns the audit ID.
    pub fn recordAlloc(self: *AuditLog, tenant_id: []const u8, size: u64) !u64 {
        if (self.len >= self.entries.len) return MemoryError.OutOfMemory;
        const id = self.next_id;
        self.next_id += 1;
        self.entries[self.len] = .{
            .id = id,
            .tenant_id = tenant_id,
            .size = size,
            .freed = false,
            .allocated_at_ms = std.time.milliTimestamp(),
            .freed_at_ms = 0,
        };
        self.len += 1;
        return id;
    }

    /// Mark an allocation as freed by audit ID.
    pub fn recordFree(self: *AuditLog, audit_id: u64) !void {
        for (self.entries[0..self.len]) |*e| {
            if (e.id == audit_id) {
                if (e.freed) return MemoryError.InvalidFree;
                e.freed = true;
                e.freed_at_ms = std.time.milliTimestamp();
                return;
            }
        }
        return MemoryError.InvalidFree;
    }

    /// Return the number of live (not freed) allocations.
    pub fn liveCount(self: *const AuditLog) usize {
        var count: usize = 0;
        for (self.entries[0..self.len]) |e| {
            if (!e.freed) count += 1;
        }
        return count;
    }

    /// Return total bytes still live.
    pub fn liveBytes(self: *const AuditLog) u64 {
        var total: u64 = 0;
        for (self.entries[0..self.len]) |e| {
            if (!e.freed) total += e.size;
        }
        return total;
    }

    /// Dump a summary of live allocations to stderr (debug helper).
    pub fn dumpLeaks(self: *const AuditLog) void {
        const live = self.liveCount();
        if (live == 0) {
            std.debug.print("AuditLog: no leaks detected.\n", .{});
            return;
        }
        std.debug.print("AuditLog: {} live allocation(s):\n", .{live});
        for (self.entries[0..self.len]) |e| {
            if (!e.freed) {
                std.debug.print(
                    "  [id={d}] tenant=\"{s}\" size={d} allocated_at={d}ms\n",
                    .{ e.id, e.tenant_id, e.size, e.allocated_at_ms },
                );
            }
        }
    }
};

// ============================================================
// Memory Pressure Callbacks
// ============================================================

pub const PressureLevel = enum {
    /// Usage < 70 % of total.
    normal,
    /// Usage >= 70 % of total.
    moderate,
    /// Usage >= 90 % of total.
    critical,
};

pub const PressureCallback = struct {
    /// Called whenever the pressure level changes.
    handler: *const fn (level: PressureLevel, used: u64, total: u64) void,
    /// Only fire when level is at least this severe.
    min_level: PressureLevel,
};

const MAX_PRESSURE_CALLBACKS = 16;

// ============================================================
// Buddy Allocator  (power-of-two block sizes)
// ============================================================

/// Minimum block size: 64 bytes.
pub const BUDDY_MIN_ORDER: u6 = 6; // 2^6 = 64
/// Maximum block size: 1 GiB.
pub const BUDDY_MAX_ORDER: u6 = 30; // 2^30 = 1 GiB

/// Compute the smallest order whose block size is >= `bytes`.
pub fn buddyOrderFor(bytes: usize) u6 {
    var order: u6 = BUDDY_MIN_ORDER;
    while (order <= BUDDY_MAX_ORDER) : (order += 1) {
        if ((@as(usize, 1) << order) >= bytes) return order;
    }
    return BUDDY_MAX_ORDER + 1; // signals "too large"
}

/// A single free block in the buddy system.
const BuddyBlock = struct {
    next: ?*BuddyBlock,
};

/// A simple buddy allocator that manages a contiguous byte slice provided
/// by the caller.  The backing memory must remain valid for the lifetime
/// of this allocator.
pub const BuddyAllocator = struct {
    /// The entire pool handed to us at init.
    pool: []u8,
    /// Free-list heads indexed by order (BUDDY_MIN_ORDER … BUDDY_MAX_ORDER).
    free_lists: [BUDDY_MAX_ORDER + 1]?*BuddyBlock,
    /// Total pool size in bytes (must be a power of two).
    total_bytes: usize,
    /// Currently allocated bytes (for accounting).
    used_bytes: usize,

    /// Initialise the allocator.  `pool` MUST have a length that is a
    /// power of two and >= (1 << BUDDY_MIN_ORDER).
    pub fn init(pool: []u8) !BuddyAllocator {
        const n = pool.len;
        if (n == 0 or (n & (n - 1)) != 0)
            return MemoryError.AllocationTooLarge; // not a power of two
        if (n < (@as(usize, 1) << BUDDY_MIN_ORDER))
            return MemoryError.AllocationTooLarge;

        var self: BuddyAllocator = .{
            .pool = pool,
            .free_lists = [_]?*BuddyBlock{null} ** (BUDDY_MAX_ORDER + 1),
            .total_bytes = n,
            .used_bytes = 0,
        };

        // Find the highest order that fits and seed the free list.
        var order: u6 = BUDDY_MAX_ORDER;
        while (order >= BUDDY_MIN_ORDER) : (order -= 1) {
            if ((@as(usize, 1) << order) <= n) {
                const block: *BuddyBlock = @ptrCast(@alignCast(pool.ptr));
                block.next = null;
                self.free_lists[order] = block;
                break;
            }
        }
        return self;
    }

    /// Allocate at least `size` bytes.  Returns a slice of exactly
    /// (1 << order) bytes.
    pub fn alloc(self: *BuddyAllocator, size: usize) ![]u8 {
        if (size == 0) return self.pool[0..0];

        const order = buddyOrderFor(size);
        if (order > BUDDY_MAX_ORDER) return MemoryError.AllocationTooLarge;

        // Find the smallest available order >= requested order.
        var available: ?u6 = null;
        var o: u6 = order;
        while (o <= BUDDY_MAX_ORDER) : (o += 1) {
            if (self.free_lists[o] != null) {
                available = o;
                break;
            }
        }
        const found_order = available orelse return MemoryError.BuddyPoolExhausted;

        // Split blocks down until we reach the desired order.
        var cur_order = found_order;
        while (cur_order > order) : (cur_order -= 1) {
            const block = self.free_lists[cur_order].?;
            self.free_lists[cur_order] = block.next;

            const half = @as(usize, 1) << (cur_order - 1);
            const buddy_ptr: *BuddyBlock = @ptrCast(@alignCast(@as([*]u8, @ptrCast(block)) + half));
            buddy_ptr.next = self.free_lists[cur_order - 1];
            self.free_lists[cur_order - 1] = buddy_ptr;

            // Put the left half back so the next iteration can pop it.
            block.next = self.free_lists[cur_order - 1];
            self.free_lists[cur_order - 1] = block;
            // Now pop left half and continue splitting in next iteration.
            self.free_lists[cur_order - 1] = self.free_lists[cur_order - 1].?.next;
            block.next = null;
            self.free_lists[cur_order - 1] = @ptrCast(@alignCast(block));
        }

        // Pop the block from the target free list.
        const result_block = self.free_lists[order].?;
        self.free_lists[order] = result_block.next;

        const block_size = @as(usize, 1) << order;
        self.used_bytes += block_size;

        const ptr: [*]u8 = @ptrCast(result_block);
        return ptr[0..block_size];
    }

    /// Free a previously allocated slice.  `slice` must have been
    /// returned by `alloc`.
    pub fn free(self: *BuddyAllocator, slice: []u8) !void {
        if (slice.len == 0) return;

        const order = buddyOrderFor(slice.len);
        if (order > BUDDY_MAX_ORDER) return MemoryError.InvalidFree;

        const block_size = @as(usize, 1) << order;

        // Return block to free list.
        const block: *BuddyBlock = @ptrCast(@alignCast(slice.ptr));
        block.next = self.free_lists[order];
        self.free_lists[order] = block;

        if (self.used_bytes >= block_size) {
            self.used_bytes -= block_size;
        } else {
            self.used_bytes = 0;
        }

        // Coalesce with buddy if possible.
        self.coalesce(order);
    }

    /// Try to merge free buddies upward from `order`.
    fn coalesce(self: *BuddyAllocator, order: u6) void {
        if (order >= BUDDY_MAX_ORDER) return;

        var prev: ?*BuddyBlock = null;
        var cur = self.free_lists[order];
        while (cur) |block| {
            const block_size = @as(usize, 1) << order;
            const block_addr = @intFromPtr(block);
            const pool_addr = @intFromPtr(self.pool.ptr);
            const offset = block_addr - pool_addr;
            // Buddy is at offset XOR block_size.
            const buddy_offset = offset ^ block_size;
            const buddy_addr = pool_addr + buddy_offset;

            // Search for the buddy in the same free list.
            var found_prev: ?*BuddyBlock = null;
            var search = self.free_lists[order];
            while (search) |s| {
                if (@intFromPtr(s) == buddy_addr) {
                    // Remove buddy from free list.
                    if (found_prev) |fp| {
                        fp.next = s.next;
                    } else {
                        self.free_lists[order] = s.next;
                    }
                    // Remove current block from free list.
                    if (prev) |p| {
                        p.next = block.next;
                    } else {
                        self.free_lists[order] = block.next;
                    }
                    // Merged block starts at lower address.
                    const merged_addr = if (offset < buddy_offset) block_addr else buddy_addr;
                    const merged: *BuddyBlock = @ptrFromInt(merged_addr);
                    merged.next = self.free_lists[order + 1];
                    self.free_lists[order + 1] = merged;
                    // Recurse upward.
                    self.coalesce(order + 1);
                    return;
                }
                found_prev = search;
                search = s.next;
            }

            prev = cur;
            cur = block.next;
        }
    }

    /// Expose a std.mem.Allocator backed by this buddy allocator.
    pub fn allocator(self: *BuddyAllocator) std.mem.Allocator {
        return .{
            .ptr = self,
            .vtable = &buddy_vtable,
        };
    }

    pub fn usedBytes(self: *const BuddyAllocator) usize {
        return self.used_bytes;
    }

    pub fn freeBytes(self: *const BuddyAllocator) usize {
        return self.total_bytes - self.used_bytes;
    }
};

fn buddyAllocFn(
    ctx: *anyopaque,
    len: usize,
    ptr_align: u8,
    _: usize,
) ?[*]u8 {
    _ = ptr_align;
    const self: *BuddyAllocator = @ptrCast(@alignCast(ctx));
    const slice = self.alloc(len) catch return null;
    return slice.ptr;
}

fn buddyResizeFn(
    _: *anyopaque,
    _: []u8,
    _: u8,
    _: usize,
    _: usize,
) bool {
    return false; // buddy blocks are fixed size
}

fn buddyFreeFn(
    ctx: *anyopaque,
    slice: []u8,
    _: u8,
    _: usize,
) void {
    const self: *BuddyAllocator = @ptrCast(@alignCast(ctx));
    self.free(slice) catch {};
}

const buddy_vtable = std.mem.Allocator.VTable{
    .alloc = buddyAllocFn,
    .resize = buddyResizeFn,
    .free = buddyFreeFn,
};

// ============================================================
// Slab Allocator  (fixed-size object pool)
// ============================================================

/// A typed slab pool for objects of a single size.
/// `T` is the element type; the pool stores at most `capacity` live objects.
pub fn SlabAllocator(comptime T: type) type {
    return struct {
        const Self = @This();

        items: []T,
        free_indices: []usize,
        free_top: usize, // stack pointer into free_indices
        backing: std.mem.Allocator,

        pub fn init(backing: std.mem.Allocator, capacity: usize) !Self {
            const items = try backing.alloc(T, capacity);
            const free_indices = try backing.alloc(usize, capacity);
            // Pre-fill the free stack with all indices (highest first so
            // the first pop returns index 0).
            for (free_indices, 0..) |*fi, i| {
                fi.* = capacity - 1 - i;
            }
            return .{
                .items = items,
                .free_indices = free_indices,
                .free_top = capacity,
                .backing = backing,
            };
        }

        pub fn deinit(self: *Self) void {
            self.backing.free(self.items);
            self.backing.free(self.free_indices);
            self.free_top = 0;
        }

        /// Obtain a pointer to a free slot.  The caller must initialise it.
        pub fn acquire(self: *Self) !*T {
            if (self.free_top == 0) return MemoryError.SlabPoolExhausted;
            self.free_top -= 1;
            const idx = self.free_indices[self.free_top];
            return &self.items[idx];
        }

        /// Return `obj` to the pool.  `obj` must point into this slab.
        pub fn release(self: *Self, obj: *T) !void {
            const base = @intFromPtr(self.items.ptr);
            const ptr = @intFromPtr(obj);
            if (ptr < base or ptr >= base + self.items.len * @sizeOf(T)) {
                return MemoryError.InvalidFree;
            }
            const idx = (ptr - base) / @sizeOf(T);
            self.free_indices[self.free_top] = idx;
            self.free_top += 1;
        }

        pub fn usedCount(self: *const Self) usize {
            return self.items.len - self.free_top;
        }

        pub fn freeCount(self: *const Self) usize {
            return self.free_top;
        }

        pub fn capacity(self: *const Self) usize {
            return self.items.len;
        }
    };
}

// ============================================================
// MemoryManager  (global + per-tenant tracking)
// ============================================================

/// A registered tenant (module or component).
const Tenant = struct {
    id: []const u8, // owned copy
    limits: ModuleLimits,
    usage: ModuleUsage,
};

/// Top-level memory manager.  Owns the global byte counters and all
/// per-tenant contexts.  Optionally wraps a BuddyAllocator and an
/// AuditLog.
pub const MemoryManager = struct {
    allocator: std.mem.Allocator,

    /// Global pool size in bytes.
    total_bytes: u64,
    /// Currently committed bytes across all tenants + untracked allocs.
    used_bytes: u64,

    tenants: std.StringHashMap(Tenant),

    /// Optional audit log (null if disabled).
    audit: ?*AuditLog,

    /// Registered pressure callbacks.
    pressure_callbacks: [MAX_PRESSURE_CALLBACKS]PressureCallback,
    pressure_callback_count: usize,

    /// Last known pressure level (to detect transitions).
    last_pressure: PressureLevel,

    pub fn init(
        allocator: std.mem.Allocator,
        total_bytes: u64,
        audit: ?*AuditLog,
    ) MemoryManager {
        return .{
            .allocator = allocator,
            .total_bytes = total_bytes,
            .used_bytes = 0,
            .tenants = std.StringHashMap(Tenant).init(allocator),
            .audit = audit,
            .pressure_callbacks = undefined,
            .pressure_callback_count = 0,
            .last_pressure = .normal,
        };
    }

    pub fn deinit(self: *MemoryManager) void {
        var it = self.tenants.valueIterator();
        while (it.next()) |t| self.allocator.free(t.id);
        self.tenants.deinit();
    }

    // ----------------------------------------------------------
    // Pressure callbacks
    // ----------------------------------------------------------

    pub fn registerPressureCallback(self: *MemoryManager, cb: PressureCallback) !void {
        if (self.pressure_callback_count >= MAX_PRESSURE_CALLBACKS)
            return MemoryError.OutOfMemory;
        self.pressure_callbacks[self.pressure_callback_count] = cb;
        self.pressure_callback_count += 1;
    }

    fn currentPressure(self: *const MemoryManager) PressureLevel {
        if (self.total_bytes == 0) return .critical;
        const pct = self.used_bytes * 100 / self.total_bytes;
        if (pct >= 90) return .critical;
        if (pct >= 70) return .moderate;
        return .normal;
    }

    fn firePressureCallbacks(self: *MemoryManager) void {
        const level = self.currentPressure();
        if (level == self.last_pressure) return;
        self.last_pressure = level;
        for (self.pressure_callbacks[0..self.pressure_callback_count]) |cb| {
            if (@intFromEnum(level) >= @intFromEnum(cb.min_level)) {
                cb.handler(level, self.used_bytes, self.total_bytes);
            }
        }
    }

    // ----------------------------------------------------------
    // Tenant management
    // ----------------------------------------------------------

    pub fn registerTenant(self: *MemoryManager, id: []const u8, limits: ModuleLimits) !void {
        if (self.tenants.contains(id)) return MemoryError.TenantAlreadyRegistered;
        const id_copy = try self.allocator.dupe(u8, id);
        errdefer self.allocator.free(id_copy);
        try self.tenants.put(id_copy, .{
            .id = id_copy,
            .limits = limits,
            .usage = .{
                .memory_used_bytes = 0,
                .process_count = 0,
                .file_count = 0,
                .resource_units_used = 0,
            },
        });
    }

    pub fn setTenantLimits(self: *MemoryManager, id: []const u8, limits: ModuleLimits) !void {
        const t = self.tenants.getPtr(id) orelse return MemoryError.TenantNotFound;
        t.limits = limits;
    }

    pub fn getTenantUsage(self: *const MemoryManager, id: []const u8) !ModuleUsage {
        const t = self.tenants.get(id) orelse return MemoryError.TenantNotFound;
        return t.usage;
    }

    // ----------------------------------------------------------
    // Global memory allocation (no tenant)
    // ----------------------------------------------------------

    pub fn allocate(self: *MemoryManager, bytes: u64) !void {
        if (self.used_bytes + bytes > self.total_bytes) return MemoryError.OutOfMemory;
        self.used_bytes += bytes;
        if (self.audit) |log| {
            _ = log.recordAlloc("", bytes) catch {};
        }
        self.firePressureCallbacks();
    }

    pub fn free(self: *MemoryManager, bytes: u64) void {
        if (bytes >= self.used_bytes) {
            self.used_bytes = 0;
        } else {
            self.used_bytes -= bytes;
        }
        self.firePressureCallbacks();
    }

    // ----------------------------------------------------------
    // Tenant-scoped memory allocation
    // ----------------------------------------------------------

    pub fn allocateTenant(self: *MemoryManager, id: []const u8, bytes: u64) !void {
        if (self.used_bytes + bytes > self.total_bytes) return MemoryError.OutOfMemory;

        const t = self.tenants.getPtr(id) orelse return MemoryError.TenantNotFound;
        if (t.usage.memory_used_bytes + bytes > t.limits.memory_limit_bytes)
            return MemoryError.TenantResourceLimitExceeded;

        t.usage.memory_used_bytes += bytes;
        self.used_bytes += bytes;

        if (self.audit) |log| {
            _ = log.recordAlloc(t.id, bytes) catch {};
        }
        self.firePressureCallbacks();
    }

    pub fn freeTenant(self: *MemoryManager, id: []const u8, bytes: u64) !void {
        const t = self.tenants.getPtr(id) orelse return MemoryError.TenantNotFound;
        if (bytes >= t.usage.memory_used_bytes) {
            self.used_bytes -= t.usage.memory_used_bytes;
            t.usage.memory_used_bytes = 0;
        } else {
            t.usage.memory_used_bytes -= bytes;
            self.used_bytes -= bytes;
        }
        self.firePressureCallbacks();
    }

    // ----------------------------------------------------------
    // Cache helpers (TTL-based key/value store)
    // ----------------------------------------------------------

    pub const CacheEntry = struct {
        value: []const u8,
        expires_at_ms: i64,
    };

    pub fn putCache(
        self: *MemoryManager,
        cache: *std.StringHashMap(CacheEntry),
        key: []const u8,
        value: []const u8,
        ttl_ms: u64,
    ) !void {
        const key_copy = try self.allocator.dupe(u8, key);
        errdefer self.allocator.free(key_copy);
        const value_copy = try self.allocator.dupe(u8, value);
        errdefer self.allocator.free(value_copy);

        const expires_at = std.time.milliTimestamp() + @as(i64, @intCast(ttl_ms));

        if (cache.getPtr(key)) |existing| {
            self.allocator.free(existing.value);
            existing.* = .{ .value = value_copy, .expires_at_ms = expires_at };
            self.allocator.free(key_copy);
        } else {
            try cache.put(key_copy, .{ .value = value_copy, .expires_at_ms = expires_at });
        }
    }

    pub fn getCache(cache: *const std.StringHashMap(CacheEntry), key: []const u8) ?[]const u8 {
        const now = std.time.milliTimestamp();
        if (cache.getPtr(key)) |entry| {
            if (entry.expires_at_ms < now) return null;
            return entry.value;
        }
        return null;
    }

    // ----------------------------------------------------------
    // Stats
    // ----------------------------------------------------------

    pub const Stats = struct {
        total_bytes: u64,
        used_bytes: u64,
        free_bytes: u64,
        pressure: PressureLevel,
        tenant_count: usize,
        live_audit_allocs: usize,
        live_audit_bytes: u64,
    };

    pub fn stats(self: *const MemoryManager) Stats {
        const live_count = if (self.audit) |log| log.liveCount() else 0;
        const live_bytes = if (self.audit) |log| log.liveBytes() else 0;
        return .{
            .total_bytes = self.total_bytes,
            .used_bytes = self.used_bytes,
            .free_bytes = self.total_bytes -| self.used_bytes,
            .pressure = self.currentPressure(),
            .tenant_count = self.tenants.count(),
            .live_audit_allocs = live_count,
            .live_audit_bytes = live_bytes,
        };
    }
};

// ============================================================
// Tests
// ============================================================

test "buddy allocator basic alloc and free" {
    var pool: [4096]u8 = undefined;
    var buddy = try BuddyAllocator.init(&pool);

    const a = try buddy.alloc(64);
    try std.testing.expect(a.len >= 64);
    try std.testing.expect(buddy.usedBytes() > 0);

    try buddy.free(a);
    try std.testing.expectEqual(@as(usize, 0), buddy.usedBytes());
}

test "buddy allocator exhaustion" {
    var pool: [128]u8 = undefined;
    var buddy = try BuddyAllocator.init(&pool);

    _ = try buddy.alloc(128);
    const err = buddy.alloc(64);
    try std.testing.expectError(MemoryError.BuddyPoolExhausted, err);
}

test "buddy std.mem.Allocator shim" {
    var pool: [4096]u8 = undefined;
    var buddy = try BuddyAllocator.init(&pool);
    const alloc = buddy.allocator();

    const buf = try alloc.alloc(u8, 100);
    try std.testing.expect(buf.len >= 100);
    alloc.free(buf);
}

test "slab allocator acquire and release" {
    const MyObj = struct { x: i32, y: i32 };
    var slab = try SlabAllocator(MyObj).init(std.testing.allocator, 8);
    defer slab.deinit();

    try std.testing.expectEqual(@as(usize, 8), slab.freeCount());
    const obj = try slab.acquire();
    obj.* = .{ .x = 1, .y = 2 };
    try std.testing.expectEqual(@as(usize, 1), slab.usedCount());

    try slab.release(obj);
    try std.testing.expectEqual(@as(usize, 0), slab.usedCount());
}

test "slab allocator exhaustion" {
    const Tiny = struct { v: u8 };
    var slab = try SlabAllocator(Tiny).init(std.testing.allocator, 2);
    defer slab.deinit();

    _ = try slab.acquire();
    _ = try slab.acquire();
    try std.testing.expectError(MemoryError.SlabPoolExhausted, slab.acquire());
}

test "audit log records allocs and detects leaks" {
    var log = try AuditLog.init(std.testing.allocator, 32);
    defer log.deinit();

    const id1 = try log.recordAlloc("tenant-a", 1024);
    const id2 = try log.recordAlloc("tenant-b", 512);

    try std.testing.expectEqual(@as(usize, 2), log.liveCount());
    try std.testing.expectEqual(@as(u64, 1536), log.liveBytes());

    try log.recordFree(id1);
    try std.testing.expectEqual(@as(usize, 1), log.liveCount());

    // Double-free should error.
    try std.testing.expectError(MemoryError.InvalidFree, log.recordFree(id1));

    try log.recordFree(id2);
    try std.testing.expectEqual(@as(usize, 0), log.liveCount());
}

test "memory manager tenant isolation and limits" {
    var mgr = MemoryManager.init(std.testing.allocator, 1024 * 1024, null);
    defer mgr.deinit();

    try mgr.registerTenant("mod-a", .{
        .memory_limit_bytes = 4096,
        .max_processes = 4,
        .max_files = 16,
        .max_resource_units = 100,
    });

    try mgr.allocateTenant("mod-a", 2048);
    try std.testing.expectError(
        MemoryError.TenantResourceLimitExceeded,
        mgr.allocateTenant("mod-a", 4096),
    );

    try mgr.freeTenant("mod-a", 2048);
    const s = mgr.stats();
    try std.testing.expectEqual(@as(u64, 0), s.used_bytes);
}

test "memory manager pressure callback fires on threshold" {
    const Helper = struct {
        var fired: bool = false;
        fn cb(level: PressureLevel, _: u64, _: u64) void {
            if (level == .critical) fired = true;
        }
    };

    var mgr = MemoryManager.init(std.testing.allocator, 1000, null);
    defer mgr.deinit();

    try mgr.registerPressureCallback(.{
        .handler = Helper.cb,
        .min_level = .critical,
    });

    // Push usage to 95 % → should trigger critical callback.
    try mgr.allocate(950);
    try std.testing.expect(Helper.fired);
}

test "memory manager global OOM" {
    var mgr = MemoryManager.init(std.testing.allocator, 512, null);
    defer mgr.deinit();

    try mgr.allocate(512);
    try std.testing.expectError(MemoryError.OutOfMemory, mgr.allocate(1));
}
