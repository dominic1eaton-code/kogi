// Kernel API and syscall helpers are defined after the Kernel struct
//! KOGI Kernel Module - Core Kernel Functionality
//! Handles low-level OS operations, protection, privacy, security, and hardware abstraction.

const std = @import("std");
const processes_module = @import("processes.zig");
const memory_module = @import("memory.zig");
const scheduler_module = @import("scheduler.zig");
const time_module = @import("time.zig");
const security_module = @import("security.zig");
const trace_module = @import("trace.zig");
const bootloader_module = @import("bootloader.zig");
const cpu_module = @import("cpu.zig");
const drivers_module = @import("device_drivers.zig");

pub const KernelMode = enum { kernel, user };

pub const Kernel = struct {
    allocator: std.mem.Allocator,
    process_manager: processes_module.ProcessManager,
    memory_allocator: memory_module.MemoryAllocator,
    scheduler: scheduler_module.Scheduler,
    clock: time_module.Clock,
    security_manager: security_module.SecurityManager,
    trace_manager: trace_module.TraceManager,
    boot_manager: bootloader_module.BootManager,
    cpu_manager: cpu_module.CPUManager,
    driver_manager: drivers_module.DriverManager,
    mode: KernelMode = .kernel,
    os_api: OSApi = default_os_api,
    abi: OSAbi = OSAbi{ .version_major = 1, .version_minor = 0, .compatible = true },

    pub fn init(allocator: std.mem.Allocator) Kernel {
        return Kernel{
            .allocator = allocator,
            .process_manager = processes_module.ProcessManager.init(allocator, processes_module.SchedulerConfig{}),
            .memory_allocator = memory_module.MemoryAllocator.init(allocator, 4096),
            .scheduler = scheduler_module.Scheduler.init(allocator),
            .clock = time_module.Clock.init(allocator),
            .security_manager = security_module.SecurityManager.init(allocator),
            .trace_manager = trace_module.TraceManager.init(allocator),
            .boot_manager = bootloader_module.BootManager.init(allocator, bootloader_module.BootConfig{}),
            .cpu_manager = cpu_module.CPUManager.init(allocator),
            .driver_manager = drivers_module.DriverManager.init(allocator),
            .mode = .kernel,
        };
    }

    pub fn switchToUserMode(self: *Kernel) void {
        self.mode = .user;
        // Additional logic for switching to user mode
    }

    pub fn switchToKernelMode(self: *Kernel) void {
        self.mode = .kernel;
        // Additional logic for switching to kernel mode
    }

    pub fn deinit(self: *Kernel) void {
        self.process_manager.deinit();
        self.memory_allocator.deinit();
        self.scheduler.deinit();
        self.clock.deinit();
        self.security_manager.deinit();
        self.trace_manager.deinit();
        self.boot_manager.deinit();
        self.cpu_manager.deinit();
        self.driver_manager.deinit();
    }
};

pub fn enforceModeBarrier(kernel: *Kernel, system: *anyopaque) bool {
	pub fn enforceModeBarrier(kernel: *Kernel, _system: *anyopaque) bool {
    // Enforce protection/privacy/security between kernel and system
    // Return true if barrier is intact, false if violation detected
    // (Stub: implement real checks)
    return kernel.mode == .kernel;
}

// --- OS API, syscalls, and helpers (placed after Kernel so Kernel type exists) ---
/// OS ABI versioning
pub const OSAbi = struct {
    version_major: u16,
    version_minor: u16,
    compatible: bool,
};

/// OS API function table
pub const OSApi = struct {
    get_time: *const fn () i64,
    create_process: *const fn (name: []const u8) u32,
    kill_process: *const fn (pid: u32) bool,
    read_memory: *const fn (addr: usize, size: usize) []u8,
    write_memory: *const fn (addr: usize, data: []const u8) bool,
    get_system_info: *const fn () []const u8,
    custom: *const fn (code: u32, args: []const u8) usize,
};

fn getTimeImpl() i64 {
    return 0;
}
fn createProcessImpl(name: []const u8) u32 {
    fn createProcessImpl(_name: []const u8) u32 {
    return 0;
}
fn killProcessImpl(pid: u32) bool {
    fn killProcessImpl(_pid: u32) bool {
    return false;
}
fn readMemoryImpl(addr: usize, size: usize) []u8 {
    fn readMemoryImpl(_addr: usize, _size: usize) []u8 {
    return &[_]u8{};
}
fn writeMemoryImpl(addr: usize, data: []const u8) bool {
    fn writeMemoryImpl(_addr: usize, _data: []const u8) bool {
    return false;
}
fn getSystemInfoImpl() []const u8 {
    return "KOGI OS";
}
fn customImpl(code: u32, args: []const u8) usize {
    fn customImpl(_code: u32, _args: []const u8) usize {
    return 0;
}

/// Example OS API table (stub implementations)
pub const default_os_api = OSApi{
    .get_time = &getTimeImpl,
    .create_process = &createProcessImpl,
    .kill_process = &killProcessImpl,
    .read_memory = &readMemoryImpl,
    .write_memory = &writeMemoryImpl,
    .get_system_info = &getSystemInfoImpl,
    .custom = &customImpl,
};

/// Supported syscalls for KOGI OS
pub const Syscall = enum {
    GetTime,
    CreateProcess,
    KillProcess,
    ReadMemory,
    WriteMemory,
    GetSystemInfo,
    Custom,
};

/// Return a static list of implemented syscall names.
pub fn syscallNames() []const []const u8 {
    return &[_][]const u8{ "GetTime", "CreateProcess", "KillProcess", "ReadMemory", "WriteMemory", "GetSystemInfo", "Custom" };
}

/// Syscall dispatcher
pub fn syscall(kernel: *Kernel, call: Syscall, args: anytype) usize {
    switch (call) {
        .GetTime => return @intCast(usize, kernel.clock.now().seconds),
        .CreateProcess => return kernel.process_manager.createProcess(args) catch 0,
        .KillProcess => return kernel.process_manager.killProcess(args) catch 0,
        .ReadMemory => return 0,
        .WriteMemory => return 0,
        .GetSystemInfo => return 0,
        .Custom => return 0,
    }
}
