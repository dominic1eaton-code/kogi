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

pub const ModuleKind = enum {
    system,
    platform,
    domain,
    integration,
    custom,
};

pub const ModuleStorageType = enum {
    local_fs,
    object_store,
    block_store,
    database,
    cache,
    temp,
};

pub const ModuleServiceStatus = enum {
    planned,
    running,
    stopped,
    degraded,
};

pub const ModuleSecurityMode = enum {
    strict,
    standard,
    permissive,
    custom,
};

pub const ModuleIODirection = enum {
    input,
    output,
    duplex,
};

pub const ModuleIOKind = enum {
    file,
    network,
    device,
    ipc,
    stream,
    custom,
};

pub const ModuleResource = struct {
    id: u32,
    name: []const u8,
    quantity: f64,
    unit: []const u8,
    created_at: i64,
    updated_at: i64,
};

pub const ModuleMemoryReservation = struct {
    id: u32,
    reserved_bytes: u64,
    committed_bytes: u64,
    region_type: memory_module.MemoryRegionType,
    permissions: memory_module.MemoryPermissions,
    created_at: i64,
    updated_at: i64,
};

pub const ModuleProcess = struct {
    id: u32,
    name: []const u8,
    state: processes_module.ProcessState,
    priority: processes_module.ProcessPriority,
    resources: processes_module.ProcessResources,
    created_at: i64,
    updated_at: i64,
};

pub const ModuleStorage = struct {
    id: u32,
    name: []const u8,
    path: []const u8,
    storage_type: ModuleStorageType,
    capacity_bytes: u64,
    created_at: i64,
    updated_at: i64,
};

pub const ModuleCliCommand = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    usage: []const u8,
    enabled: bool,
    created_at: i64,
    updated_at: i64,
};

pub const ModuleKeyValue = struct {
    key: []const u8,
    value: []const u8,
    updated_at: i64,
};

pub const ModuleOption = struct {
    name: []const u8,
    value: []const u8,
    enabled: bool,
    updated_at: i64,
};

pub const ModuleService = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    endpoint: []const u8,
    status: ModuleServiceStatus,
    created_at: i64,
    updated_at: i64,
};

pub const ModuleSecurityPolicy = struct {
    id: u32,
    name: []const u8,
    description: []const u8,
    mode: ModuleSecurityMode,
    created_at: i64,
    updated_at: i64,
};

pub const ModuleIOChannel = struct {
    id: u32,
    name: []const u8,
    direction: ModuleIODirection,
    kind: ModuleIOKind,
    target: []const u8,
    created_at: i64,
    updated_at: i64,
};

pub const ModuleRuntime = struct {
    module_id: u32,
    kind: ModuleKind,
    name: []const u8,
    description: []const u8,
    active: bool,
    resources: std.array_list.Managed(ModuleResource),
    memory: std.array_list.Managed(ModuleMemoryReservation),
    processes: std.array_list.Managed(ModuleProcess),
    storage: std.array_list.Managed(ModuleStorage),
    cli_commands: std.array_list.Managed(ModuleCliCommand),
    configurations: std.array_list.Managed(ModuleKeyValue),
    settings: std.array_list.Managed(ModuleKeyValue),
    parameters: std.array_list.Managed(ModuleKeyValue),
    options: std.array_list.Managed(ModuleOption),
    services: std.array_list.Managed(ModuleService),
    security: std.array_list.Managed(ModuleSecurityPolicy),
    io_channels: std.array_list.Managed(ModuleIOChannel),
    created_at: i64,
    updated_at: i64,
};

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
    module_runtimes: std.array_list.Managed(ModuleRuntime),
    next_module_id: u32 = 0,
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
            .module_runtimes = std.array_list.Managed(ModuleRuntime).init(allocator),
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

    fn findModuleIndexById(self: *Kernel, module_id: u32) ?usize {
        for (self.module_runtimes.items, 0..) |module, idx| {
            if (module.module_id == module_id) return idx;
        }
        return null;
    }

    fn getModuleMutable(self: *Kernel, module_id: u32) ?*ModuleRuntime {
        const idx = self.findModuleIndexById(module_id) orelse return null;
        return &self.module_runtimes.items[idx];
    }

    fn deinitModuleKeyValues(self: *Kernel, values: *std.array_list.Managed(ModuleKeyValue)) void {
        for (values.items) |*entry| {
            self.allocator.free(entry.key);
            self.allocator.free(entry.value);
        }
        values.deinit();
    }

    fn deinitModuleOptions(self: *Kernel, options: *std.array_list.Managed(ModuleOption)) void {
        for (options.items) |*entry| {
            self.allocator.free(entry.name);
            self.allocator.free(entry.value);
        }
        options.deinit();
    }

    fn deinitModuleRuntime(self: *Kernel, module: *ModuleRuntime) void {
        self.allocator.free(module.name);
        self.allocator.free(module.description);

        for (module.resources.items) |*resource| {
            self.allocator.free(resource.name);
            self.allocator.free(resource.unit);
        }
        module.resources.deinit();

        module.memory.deinit();

        for (module.processes.items) |*proc| {
            self.allocator.free(proc.name);
        }
        module.processes.deinit();

        for (module.storage.items) |*storage| {
            self.allocator.free(storage.name);
            self.allocator.free(storage.path);
        }
        module.storage.deinit();

        for (module.cli_commands.items) |*command| {
            self.allocator.free(command.name);
            self.allocator.free(command.description);
            self.allocator.free(command.usage);
        }
        module.cli_commands.deinit();

        self.deinitModuleKeyValues(&module.configurations);
        self.deinitModuleKeyValues(&module.settings);
        self.deinitModuleKeyValues(&module.parameters);
        self.deinitModuleOptions(&module.options);

        for (module.services.items) |*service| {
            self.allocator.free(service.name);
            self.allocator.free(service.description);
            self.allocator.free(service.endpoint);
        }
        module.services.deinit();

        for (module.security.items) |*policy| {
            self.allocator.free(policy.name);
            self.allocator.free(policy.description);
        }
        module.security.deinit();

        for (module.io_channels.items) |*channel| {
            self.allocator.free(channel.name);
            self.allocator.free(channel.target);
        }
        module.io_channels.deinit();
    }

    fn upsertModuleKeyValue(
        self: *Kernel,
        values: *std.array_list.Managed(ModuleKeyValue),
        key: []const u8,
        value: []const u8,
        updated_at: i64,
    ) !void {
        for (values.items) |*entry| {
            if (!std.mem.eql(u8, entry.key, key)) continue;

            const new_value = try self.allocator.dupe(u8, value);
            self.allocator.free(entry.value);
            entry.value = new_value;
            entry.updated_at = updated_at;
            return;
        }

        const entry = ModuleKeyValue{
            .key = try self.allocator.dupe(u8, key),
            .value = try self.allocator.dupe(u8, value),
            .updated_at = updated_at,
        };
        errdefer {
            self.allocator.free(entry.key);
            self.allocator.free(entry.value);
        }
        try values.append(entry);
    }

    pub fn registerModule(self: *Kernel, kind: ModuleKind, name: []const u8, description: []const u8) !u32 {
        const module_id = self.next_module_id;
        self.next_module_id += 1;
        const now = std.time.timestamp();

        var module = ModuleRuntime{
            .module_id = module_id,
            .kind = kind,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .active = false,
            .resources = std.array_list.Managed(ModuleResource).init(self.allocator),
            .memory = std.array_list.Managed(ModuleMemoryReservation).init(self.allocator),
            .processes = std.array_list.Managed(ModuleProcess).init(self.allocator),
            .storage = std.array_list.Managed(ModuleStorage).init(self.allocator),
            .cli_commands = std.array_list.Managed(ModuleCliCommand).init(self.allocator),
            .configurations = std.array_list.Managed(ModuleKeyValue).init(self.allocator),
            .settings = std.array_list.Managed(ModuleKeyValue).init(self.allocator),
            .parameters = std.array_list.Managed(ModuleKeyValue).init(self.allocator),
            .options = std.array_list.Managed(ModuleOption).init(self.allocator),
            .services = std.array_list.Managed(ModuleService).init(self.allocator),
            .security = std.array_list.Managed(ModuleSecurityPolicy).init(self.allocator),
            .io_channels = std.array_list.Managed(ModuleIOChannel).init(self.allocator),
            .created_at = now,
            .updated_at = now,
        };
        errdefer self.deinitModuleRuntime(&module);

        try self.module_runtimes.append(module);
        return module_id;
    }

    pub fn provisionModuleWithDefaults(
        self: *Kernel,
        kind: ModuleKind,
        name: []const u8,
        description: []const u8,
    ) !u32 {
        const module_id = try self.registerModule(kind, name, description);
        try self.provisionModule(module_id);
        return module_id;
    }

    pub fn provisionModule(self: *Kernel, module_id: u32) !void {
        const process_resources = processes_module.ProcessResources{};
        try self.addModuleResource(module_id, "compute_pool", 1.0, "pool");
        try self.addModuleMemoryReservation(
            module_id,
            64 * 1024 * 1024,
            0,
            .heap,
            .{ .read = true, .write = true, .execute = false, .privileged = false },
        );
        try self.addModuleProcess(module_id, "module-supervisor", .ready, .normal, process_resources);
        try self.addModuleStorage(module_id, "module-storage", "/module/storage", .local_fs, 4 * 1024 * 1024 * 1024);
        try self.addModuleCliCommand(module_id, "status", "Inspect module runtime state.", "<module> status");
        try self.setModuleConfiguration(module_id, "scheduler.mode", "preemptive");
        try self.setModuleSetting(module_id, "telemetry.enabled", "true");
        try self.setModuleParameter(module_id, "retry.max_attempts", "3");
        try self.setModuleOption(module_id, "safe_mode", "enabled", true);
        try self.addModuleService(module_id, "module-service", "Default module service.", "local://service", .running);
        try self.addModuleSecurityPolicy(module_id, "default-policy", "Default module security policy.", .standard);
        try self.addModuleIOChannel(module_id, "module-io", .duplex, .stream, "local://io");

        const module = self.getModuleMutable(module_id) orelse return error.ModuleNotFound;
        module.active = true;
        module.updated_at = std.time.timestamp();
    }

    pub fn addModuleResource(self: *Kernel, module_id: u32, name: []const u8, quantity: f64, unit: []const u8) !u32 {
        const module = self.getModuleMutable(module_id) orelse return error.ModuleNotFound;
        const now = std.time.timestamp();
        const resource_id = @as(u32, @intCast(module.resources.items.len));
        const resource = ModuleResource{
            .id = resource_id,
            .name = try self.allocator.dupe(u8, name),
            .quantity = quantity,
            .unit = try self.allocator.dupe(u8, unit),
            .created_at = now,
            .updated_at = now,
        };
        errdefer {
            self.allocator.free(resource.name);
            self.allocator.free(resource.unit);
        }
        try module.resources.append(resource);
        module.updated_at = now;
        return resource_id;
    }

    pub fn addModuleMemoryReservation(
        self: *Kernel,
        module_id: u32,
        reserved_bytes: u64,
        committed_bytes: u64,
        region_type: memory_module.MemoryRegionType,
        permissions: memory_module.MemoryPermissions,
    ) !u32 {
        const module = self.getModuleMutable(module_id) orelse return error.ModuleNotFound;
        const now = std.time.timestamp();
        const reservation_id = @as(u32, @intCast(module.memory.items.len));
        const reservation = ModuleMemoryReservation{
            .id = reservation_id,
            .reserved_bytes = reserved_bytes,
            .committed_bytes = committed_bytes,
            .region_type = region_type,
            .permissions = permissions,
            .created_at = now,
            .updated_at = now,
        };
        try module.memory.append(reservation);
        module.updated_at = now;
        return reservation_id;
    }

    pub fn addModuleProcess(
        self: *Kernel,
        module_id: u32,
        name: []const u8,
        state: processes_module.ProcessState,
        priority: processes_module.ProcessPriority,
        resources: processes_module.ProcessResources,
    ) !u32 {
        const module = self.getModuleMutable(module_id) orelse return error.ModuleNotFound;
        const now = std.time.timestamp();
        const process_id = @as(u32, @intCast(module.processes.items.len));
        const process = ModuleProcess{
            .id = process_id,
            .name = try self.allocator.dupe(u8, name),
            .state = state,
            .priority = priority,
            .resources = resources,
            .created_at = now,
            .updated_at = now,
        };
        errdefer self.allocator.free(process.name);
        try module.processes.append(process);
        module.updated_at = now;
        return process_id;
    }

    pub fn addModuleStorage(
        self: *Kernel,
        module_id: u32,
        name: []const u8,
        path: []const u8,
        storage_type: ModuleStorageType,
        capacity_bytes: u64,
    ) !u32 {
        const module = self.getModuleMutable(module_id) orelse return error.ModuleNotFound;
        const now = std.time.timestamp();
        const storage_id = @as(u32, @intCast(module.storage.items.len));
        const storage = ModuleStorage{
            .id = storage_id,
            .name = try self.allocator.dupe(u8, name),
            .path = try self.allocator.dupe(u8, path),
            .storage_type = storage_type,
            .capacity_bytes = capacity_bytes,
            .created_at = now,
            .updated_at = now,
        };
        errdefer {
            self.allocator.free(storage.name);
            self.allocator.free(storage.path);
        }
        try module.storage.append(storage);
        module.updated_at = now;
        return storage_id;
    }

    pub fn addModuleCliCommand(
        self: *Kernel,
        module_id: u32,
        name: []const u8,
        description: []const u8,
        usage: []const u8,
    ) !u32 {
        const module = self.getModuleMutable(module_id) orelse return error.ModuleNotFound;
        const now = std.time.timestamp();
        const command_id = @as(u32, @intCast(module.cli_commands.items.len));
        const command = ModuleCliCommand{
            .id = command_id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .usage = try self.allocator.dupe(u8, usage),
            .enabled = true,
            .created_at = now,
            .updated_at = now,
        };
        errdefer {
            self.allocator.free(command.name);
            self.allocator.free(command.description);
            self.allocator.free(command.usage);
        }
        try module.cli_commands.append(command);
        module.updated_at = now;
        return command_id;
    }

    pub fn setModuleConfiguration(self: *Kernel, module_id: u32, key: []const u8, value: []const u8) !void {
        const module = self.getModuleMutable(module_id) orelse return error.ModuleNotFound;
        const now = std.time.timestamp();
        try self.upsertModuleKeyValue(&module.configurations, key, value, now);
        module.updated_at = now;
    }

    pub fn setModuleSetting(self: *Kernel, module_id: u32, key: []const u8, value: []const u8) !void {
        const module = self.getModuleMutable(module_id) orelse return error.ModuleNotFound;
        const now = std.time.timestamp();
        try self.upsertModuleKeyValue(&module.settings, key, value, now);
        module.updated_at = now;
    }

    pub fn setModuleParameter(self: *Kernel, module_id: u32, key: []const u8, value: []const u8) !void {
        const module = self.getModuleMutable(module_id) orelse return error.ModuleNotFound;
        const now = std.time.timestamp();
        try self.upsertModuleKeyValue(&module.parameters, key, value, now);
        module.updated_at = now;
    }

    pub fn setModuleOption(
        self: *Kernel,
        module_id: u32,
        name: []const u8,
        value: []const u8,
        enabled: bool,
    ) !void {
        const module = self.getModuleMutable(module_id) orelse return error.ModuleNotFound;
        const now = std.time.timestamp();

        for (module.options.items) |*option| {
            if (!std.mem.eql(u8, option.name, name)) continue;
            const new_value = try self.allocator.dupe(u8, value);
            self.allocator.free(option.value);
            option.value = new_value;
            option.enabled = enabled;
            option.updated_at = now;
            module.updated_at = now;
            return;
        }

        const option = ModuleOption{
            .name = try self.allocator.dupe(u8, name),
            .value = try self.allocator.dupe(u8, value),
            .enabled = enabled,
            .updated_at = now,
        };
        errdefer {
            self.allocator.free(option.name);
            self.allocator.free(option.value);
        }
        try module.options.append(option);
        module.updated_at = now;
    }

    pub fn addModuleService(
        self: *Kernel,
        module_id: u32,
        name: []const u8,
        description: []const u8,
        endpoint: []const u8,
        status: ModuleServiceStatus,
    ) !u32 {
        const module = self.getModuleMutable(module_id) orelse return error.ModuleNotFound;
        const now = std.time.timestamp();
        const service_id = @as(u32, @intCast(module.services.items.len));
        const service = ModuleService{
            .id = service_id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .endpoint = try self.allocator.dupe(u8, endpoint),
            .status = status,
            .created_at = now,
            .updated_at = now,
        };
        errdefer {
            self.allocator.free(service.name);
            self.allocator.free(service.description);
            self.allocator.free(service.endpoint);
        }
        try module.services.append(service);
        module.updated_at = now;
        return service_id;
    }

    pub fn addModuleSecurityPolicy(
        self: *Kernel,
        module_id: u32,
        name: []const u8,
        description: []const u8,
        mode: ModuleSecurityMode,
    ) !u32 {
        const module = self.getModuleMutable(module_id) orelse return error.ModuleNotFound;
        const now = std.time.timestamp();
        const policy_id = @as(u32, @intCast(module.security.items.len));
        const policy = ModuleSecurityPolicy{
            .id = policy_id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .mode = mode,
            .created_at = now,
            .updated_at = now,
        };
        errdefer {
            self.allocator.free(policy.name);
            self.allocator.free(policy.description);
        }
        try module.security.append(policy);
        module.updated_at = now;
        return policy_id;
    }

    pub fn addModuleIOChannel(
        self: *Kernel,
        module_id: u32,
        name: []const u8,
        direction: ModuleIODirection,
        kind: ModuleIOKind,
        target: []const u8,
    ) !u32 {
        const module = self.getModuleMutable(module_id) orelse return error.ModuleNotFound;
        const now = std.time.timestamp();
        const channel_id = @as(u32, @intCast(module.io_channels.items.len));
        const channel = ModuleIOChannel{
            .id = channel_id,
            .name = try self.allocator.dupe(u8, name),
            .direction = direction,
            .kind = kind,
            .target = try self.allocator.dupe(u8, target),
            .created_at = now,
            .updated_at = now,
        };
        errdefer {
            self.allocator.free(channel.name);
            self.allocator.free(channel.target);
        }
        try module.io_channels.append(channel);
        module.updated_at = now;
        return channel_id;
    }

    pub fn getModules(self: *Kernel) []ModuleRuntime {
        return self.module_runtimes.items;
    }

    pub fn getModuleById(self: *Kernel, module_id: u32) ?*ModuleRuntime {
        return self.getModuleMutable(module_id);
    }

    pub fn deinit(self: *Kernel) void {
        for (self.module_runtimes.items) |*module| {
            self.deinitModuleRuntime(module);
        }
        self.module_runtimes.deinit();

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

test "kernel orchestrates and provisions module runtime components" {
    const allocator = std.testing.allocator;
    var kernel = Kernel.init(allocator);
    defer kernel.deinit();

    const module_id = try kernel.provisionModuleWithDefaults(.platform, "workspace", "Workspace module runtime");
    const module = kernel.getModuleById(module_id) orelse return error.ModuleNotFound;

    try std.testing.expect(module.active);
    try std.testing.expectEqual(@as(usize, 1), module.resources.items.len);
    try std.testing.expectEqual(@as(usize, 1), module.memory.items.len);
    try std.testing.expectEqual(@as(usize, 1), module.processes.items.len);
    try std.testing.expectEqual(@as(usize, 1), module.storage.items.len);
    try std.testing.expectEqual(@as(usize, 1), module.cli_commands.items.len);
    try std.testing.expectEqual(@as(usize, 1), module.configurations.items.len);
    try std.testing.expectEqual(@as(usize, 1), module.settings.items.len);
    try std.testing.expectEqual(@as(usize, 1), module.parameters.items.len);
    try std.testing.expectEqual(@as(usize, 1), module.options.items.len);
    try std.testing.expectEqual(@as(usize, 1), module.services.items.len);
    try std.testing.expectEqual(@as(usize, 1), module.security.items.len);
    try std.testing.expectEqual(@as(usize, 1), module.io_channels.items.len);

    try kernel.setModuleConfiguration(module_id, "scheduler.mode", "cooperative");
    try std.testing.expectEqual(@as(usize, 1), module.configurations.items.len);
    try std.testing.expectEqualStrings("cooperative", module.configurations.items[0].value);
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
fn createProcessImpl(_: []const u8) u32 {
    return 0;
}
fn killProcessImpl(_: u32) bool {
    return false;
}
fn readMemoryImpl(_: usize, _: usize) []u8 {
    return &[_]u8{};
}
fn writeMemoryImpl(_: usize, _: []const u8) bool {
    return false;
}
fn getSystemInfoImpl() []const u8 {
    return "KOGI OS";
}
fn customImpl(_: u32, _: []const u8) usize {
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

/// Enforce protection/privacy/security between kernel and system
/// Return true if barrier is intact, false if violation detected
pub fn enforceModeBarrier(kernel: *Kernel, _: *anyopaque) bool {
    // Stub: implement real checks
    return kernel.mode == .kernel;
}

/// Syscall dispatcher
pub fn syscall(kernel: *Kernel, call: Syscall, args: anytype) usize {
    _ = kernel;
    _ = args;
    switch (call) {
        .GetTime => return 0,
        .CreateProcess => return 0,
        .KillProcess => return 0,
        .ReadMemory => return 0,
        .WriteMemory => return 0,
        .GetSystemInfo => return 0,
        .Custom => return 0,
    }
}
