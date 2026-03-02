//! KOGI Memory Management System - Memory allocation, tracking, and protection
//! Provides virtual memory management, page tables, and memory protection

const std = @import("std");

/// Memory page
pub const MemoryPage = struct {
    id: u32,
    base_address: u64,
    size_bytes: u64,
    owner_process_id: ?u32 = null,
    permissions: MemoryPermissions,
    resident: bool = true,
    dirty: bool = false,
    accessed: bool = false,
    created_at: i64,
};

/// Memory permissions
pub const MemoryPermissions = packed struct {
    read: bool = true,
    write: bool = false,
    execute: bool = false,
    privileged: bool = false,
};

/// Memory region
pub const MemoryRegion = struct {
    region_id: u32,
    name: []const u8,
    start_address: u64,
    end_address: u64,
    size_bytes: u64,
    region_type: MemoryRegionType,
    permissions: MemoryPermissions,
    process_id: ?u32 = null,
};

/// Memory region types
pub const MemoryRegionType = enum {
    kernel,
    heap,
    stack,
    code,
    data,
    shared,
    io,
};

/// Memory statistics
pub const MemoryStats = struct {
    total_memory_bytes: u64,
    used_memory_bytes: u64,
    free_memory_bytes: u64,
    page_count: u32,
    resident_pages: u32,
    swapped_pages: u32,
    page_faults: u64,
    page_faults_major: u64,
    page_faults_minor: u64,
};

/// Memory allocator with tracking
pub const MemoryAllocator = struct {
    allocator: std.mem.Allocator,
    pages: std.ArrayList(MemoryPage),
    regions: std.ArrayList(MemoryRegion),
    allocations: std.ArrayList(Allocation),
    next_page_id: u32 = 0,
    next_region_id: u32 = 0,
    next_address: u64 = 0x1000000,
    total_allocated: u64 = 0,
    total_freed: u64 = 0,
    page_fault_count: u64 = 0,
    major_fault_count: u64 = 0,
    minor_fault_count: u64 = 0,
    mutex: std.Thread.Mutex = .{},

    /// Allocation tracking
    pub const Allocation = struct {
        id: u32,
        address: u64,
        size_bytes: u64,
        owner_process_id: ?u32 = null,
        allocated_at: i64,
        freed_at: ?i64 = null,
        freed: bool = false,
    };

    pub fn init(allocator: std.mem.Allocator, total_memory_mb: u64) MemoryAllocator {
        return MemoryAllocator{
            .allocator = allocator,
            .pages = std.ArrayList(MemoryPage){},
            .regions = std.ArrayList(MemoryRegion){},
            .allocations = std.ArrayList(Allocation){},
            .total_allocated = total_memory_mb * 1024 * 1024,
        };
    }

    pub fn deinit(self: *MemoryAllocator) void {
        for (self.regions.items) |region| {
            self.allocator.free(region.name);
        }
        self.regions.deinit(self.allocator);
        self.pages.deinit(self.allocator);
        self.allocations.deinit(self.allocator);
    }

    /// Allocate memory for a process
    pub fn allocate(
        self: *MemoryAllocator,
        process_id: u32,
        size_bytes: u64,
        region_type: MemoryRegionType,
        permissions: MemoryPermissions,
    ) !u64 {
        _ = region_type;
        self.mutex.lock();
        defer self.mutex.unlock();

        const address = self.next_address;
        self.next_address += std.mem.alignForward(u64, size_bytes, 4096);

        const allocation = Allocation{
            .id = @intCast(self.allocations.items.len),
            .address = address,
            .size_bytes = size_bytes,
            .owner_process_id = process_id,
            .allocated_at = std.time.timestamp(),
        };

        try self.allocations.append(self.allocator, allocation);

        // Create pages for allocation
        const num_pages = (size_bytes + 4095) / 4096;
        for (0..num_pages) |i| {
            const page = MemoryPage{
                .id = self.next_page_id,
                .base_address = address + (i * 4096),
                .size_bytes = 4096,
                .owner_process_id = process_id,
                .permissions = permissions,
                .created_at = std.time.timestamp(),
            };
            try self.pages.append(self.allocator, page);
            self.next_page_id += 1;
        }

        return address;
    }

    /// Free memory
    pub fn free(self: *MemoryAllocator, address: u64) !void {
        self.mutex.lock();
        defer self.mutex.unlock();

        for (self.allocations.items) |*alloc| {
            if (alloc.address == address and !alloc.freed) {
                alloc.freed = true;
                alloc.freed_at = std.time.timestamp();

                // Mark pages as free
                for (self.pages.items) |*page| {
                    if (page.base_address >= address and page.base_address < address + alloc.size_bytes) {
                        page.owner_process_id = null;
                    }
                }

                return;
            }
        }
        return error.AllocationNotFound;
    }

    /// Create memory region
    pub fn createRegion(
        self: *MemoryAllocator,
        name: []const u8,
        size_bytes: u64,
        region_type: MemoryRegionType,
        permissions: MemoryPermissions,
    ) !u32 {
        self.mutex.lock();
        defer self.mutex.unlock();

        const region_id = self.next_region_id;
        self.next_region_id += 1;

        const start_address = self.next_address;
        const end_address = start_address + size_bytes;
        self.next_address = end_address + 4096;

        const region = MemoryRegion{
            .region_id = region_id,
            .name = try self.allocator.dupe(u8, name),
            .start_address = start_address,
            .end_address = end_address,
            .size_bytes = size_bytes,
            .region_type = region_type,
            .permissions = permissions,
        };

        try self.regions.append(self.allocator, region);
        return region_id;
    }

    /// Get memory statistics
    pub fn getStats(self: *MemoryAllocator) MemoryStats {
        var used: u64 = 0;
        var resident_count: u32 = 0;
        var swapped_count: u32 = 0;

        for (self.allocations.items) |alloc| {
            if (!alloc.freed) {
                used += alloc.size_bytes;
            }
        }

        for (self.pages.items) |page| {
            if (page.owner_process_id != null) {
                if (page.resident) {
                    resident_count += 1;
                } else {
                    swapped_count += 1;
                }
            }
        }

        return MemoryStats{
            .total_memory_bytes = self.total_allocated,
            .used_memory_bytes = used,
            .free_memory_bytes = self.total_allocated - used,
            .page_count = @intCast(self.pages.items.len),
            .resident_pages = resident_count,
            .swapped_pages = swapped_count,
            .page_faults = self.page_fault_count,
            .page_faults_major = self.major_fault_count,
            .page_faults_minor = self.minor_fault_count,
        };
    }

    /// Record page fault
    pub fn recordPageFault(self: *MemoryAllocator, is_major: bool) void {
        self.mutex.lock();
        defer self.mutex.unlock();

        self.page_fault_count += 1;
        if (is_major) {
            self.major_fault_count += 1;
        } else {
            self.minor_fault_count += 1;
        }
    }

    /// Get regions
    pub fn getRegions(self: *MemoryAllocator) []MemoryRegion {
        return self.regions.items;
    }

    /// Get pages
    pub fn getPages(self: *MemoryAllocator) []MemoryPage {
        return self.pages.items;
    }

    /// Get allocations
    pub fn getAllocations(self: *MemoryAllocator) []Allocation {
        return self.allocations.items;
    }
};
