const std = @import("std");

pub const Mode = enum {
    kernel,
    user,
};

pub const ModuleStatus = enum {
    active,
    disabled,
};

pub const ModuleRecord = struct {
    id: []const u8,
    kind: []const u8,
    status: ModuleStatus,
    registered_at_ms: i64,
};

pub const EventRecord = struct {
    topic: []const u8,
    payload: []const u8,
    emitted_at_ms: i64,
};

pub const Permission = enum {
    kernel_admin,
    schedule_tasks,
    manage_modules,
    manage_memory,
    manage_processes,
    manage_files,
    read_audit,
};

pub const Role = enum {
    root,
    host,
    server,
    module_runtime,
    user,
};

pub const ProcessState = enum {
    ready,
    running,
    waiting,
    terminated,
};

pub const ProcessRecord = struct {
    pid: u64,
    owner: Role,
    name: []const u8,
    state: ProcessState,
    created_at_ms: i64,
};

pub const FileRecord = struct {
    path: []const u8,
    owner: Role,
    size_bytes: u64,
    created_at_ms: i64,
};

pub const ScheduleEntry = struct {
    id: []const u8,
    topic: []const u8,
    payload: []const u8,
    due_at_ms: i64,
    interval_ms: ?u64,
    enabled: bool,
};

pub const CacheEntry = struct {
    value: []const u8,
    expires_at_ms: i64,
};

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

pub const ModuleIsolationContext = struct {
    module_id: []const u8,
    limits: ModuleLimits,
    usage: ModuleUsage,
    network_manager: []const u8,
    service_endpoint: []const u8,
    status: ModuleStatus,
};

pub const ComponentClass = enum {
    kernel,
    host,
    server,
    engine,
    services,
    module,
};

pub const ComponentIsolationContext = struct {
    component_id: []const u8,
    class: ComponentClass,
    limits: ModuleLimits,
    usage: ModuleUsage,
    network_ingress_bytes: u64,
    network_egress_bytes: u64,
    network_manager: []const u8,
    endpoint: []const u8,
    status: ModuleStatus,
    registered_at_ms: i64,
};

pub const AccessDenied = error{AccessDenied};

pub const Kernel = struct {
    allocator: std.mem.Allocator,
    mode: Mode,

    modules: std.StringHashMap(ModuleRecord),
    module_isolation: std.StringHashMap(ModuleIsolationContext),
    component_isolation: std.StringHashMap(ComponentIsolationContext),
    events: std.ArrayList(EventRecord),
    scheduler: std.ArrayList(ScheduleEntry),
    processes: std.ArrayList(ProcessRecord),
    files: std.ArrayList(FileRecord),
    cache: std.StringHashMap(CacheEntry),

    memory_total_bytes: u64,
    memory_used_bytes: u64,
    next_pid: u64,

    pub fn init(allocator: std.mem.Allocator) Kernel {
        return .{
            .allocator = allocator,
            .mode = .kernel,
            .modules = std.StringHashMap(ModuleRecord).init(allocator),
            .module_isolation = std.StringHashMap(ModuleIsolationContext).init(allocator),
            .component_isolation = std.StringHashMap(ComponentIsolationContext).init(allocator),
            .events = std.ArrayList(EventRecord).init(allocator),
            .scheduler = std.ArrayList(ScheduleEntry).init(allocator),
            .processes = std.ArrayList(ProcessRecord).init(allocator),
            .files = std.ArrayList(FileRecord).init(allocator),
            .cache = std.StringHashMap(CacheEntry).init(allocator),
            .memory_total_bytes = 4 * 1024 * 1024 * 1024,
            .memory_used_bytes = 0,
            .next_pid = 1000,
        };
    }

    pub fn deinit(self: *Kernel) void {
        var module_it = self.modules.valueIterator();
        while (module_it.next()) |module| {
            self.allocator.free(module.id);
            self.allocator.free(module.kind);
        }
        self.modules.deinit();

        var isolation_it = self.module_isolation.valueIterator();
        while (isolation_it.next()) |ctx| {
            self.allocator.free(ctx.module_id);
            self.allocator.free(ctx.network_manager);
            self.allocator.free(ctx.service_endpoint);
        }
        self.module_isolation.deinit();

        var component_it = self.component_isolation.valueIterator();
        while (component_it.next()) |ctx| {
            self.allocator.free(ctx.component_id);
            self.allocator.free(ctx.network_manager);
            self.allocator.free(ctx.endpoint);
        }
        self.component_isolation.deinit();

        for (self.events.items) |event| {
            self.allocator.free(event.topic);
            self.allocator.free(event.payload);
        }
        self.events.deinit();

        for (self.scheduler.items) |entry| {
            self.allocator.free(entry.id);
            self.allocator.free(entry.topic);
            self.allocator.free(entry.payload);
        }
        self.scheduler.deinit();

        for (self.processes.items) |process| {
            self.allocator.free(process.name);
        }
        self.processes.deinit();

        for (self.files.items) |file| {
            self.allocator.free(file.path);
        }
        self.files.deinit();

        var cache_it = self.cache.valueIterator();
        while (cache_it.next()) |entry| {
            self.allocator.free(entry.value);
        }
        self.cache.deinit();
    }

    pub fn setMode(self: *Kernel, mode: Mode, actor: Role) !void {
        if (actor != .root and actor != .host) return AccessDenied.AccessDenied;
        self.mode = mode;
    }

    pub fn enforce(self: *Kernel, actor: Role, permission: Permission) !void {
        _ = self;
        if (!hasPermission(actor, permission)) {
            return AccessDenied.AccessDenied;
        }
    }

    pub fn registerModule(self: *Kernel, actor: Role, id: []const u8, kind: []const u8) !void {
        try self.enforce(actor, .manage_modules);

        const now = std.time.milliTimestamp();
        const id_copy = try self.allocator.dupe(u8, id);
        errdefer self.allocator.free(id_copy);
        const kind_copy = try self.allocator.dupe(u8, kind);
        errdefer self.allocator.free(kind_copy);

        try self.modules.put(id_copy, .{
            .id = id_copy,
            .kind = kind_copy,
            .status = .active,
            .registered_at_ms = now,
        });

        const isolation_key = try self.allocator.dupe(u8, id);
        errdefer self.allocator.free(isolation_key);
        const endpoint = if (std.mem.eql(u8, id, "kogi.office"))
            try self.allocator.dupe(u8, "/services/office")
        else
            try std.fmt.allocPrint(self.allocator, "/modules/{s}", .{id});
        errdefer self.allocator.free(endpoint);
        const network_manager = try self.allocator.dupe(u8, "kogi-go-network");
        errdefer self.allocator.free(network_manager);

        try self.module_isolation.put(isolation_key, .{
            .module_id = isolation_key,
            .limits = defaultModuleLimits(),
            .usage = .{
                .memory_used_bytes = 0,
                .process_count = 0,
                .file_count = 0,
                .resource_units_used = 0,
            },
            .network_manager = network_manager,
            .service_endpoint = endpoint,
            .status = .active,
        });

        try self.registerPlatformComponent(
            actor,
            id,
            .module,
            endpoint,
            network_manager,
            defaultModuleLimits(),
        );

        try self.publishEvent(actor, "kernel.module.registered", "{}", now);
    }

    pub fn bootstrapOfficeModule(self: *Kernel, actor: Role) !void {
        try self.registerModule(actor, "kogi.office", "office");

        try self.setModuleLimits(actor, "kogi.office", .{
            .memory_limit_bytes = 768 * 1024 * 1024,
            .max_processes = 96,
            .max_files = 6000,
            .max_resource_units = 14000,
        });

        try self.allocateModuleMemory(actor, "kogi.office", 64 * 1024 * 1024);
        _ = try self.spawnModuleProcess(actor, "kogi.office", "office-dashboard-worker");
        _ = try self.spawnModuleProcess(actor, "kogi.office", "office-assistant-worker");
        try self.createModuleFile(actor, "kogi.office", "/modules/office/dashboard.cache", 8192);
        try self.createModuleFile(actor, "kogi.office", "/modules/office/workspace.cache", 8192);
        try self.reserveModuleResources(actor, "kogi.office", 320);

        try self.publishEvent(
            actor,
            "office.module.bootstrapped",
            "{\"module\":\"kogi.office\",\"views\":[\"dashboard\",\"portfolio\",\"timeline\",\"workspace\",\"assistant\"]}",
            std.time.milliTimestamp(),
        );
    }

    pub fn setModuleLimits(self: *Kernel, actor: Role, module_id: []const u8, limits: ModuleLimits) !void {
        try self.enforce(actor, .manage_modules);
        var ctx = try self.getIsolationContext(module_id);
        ctx.limits = limits;
    }

    pub fn allocateModuleMemory(self: *Kernel, actor: Role, module_id: []const u8, bytes: u64) !void {
        try self.enforce(actor, .manage_memory);

        if (self.memory_used_bytes + bytes > self.memory_total_bytes) {
            return error.OutOfMemory;
        }

        var ctx = try self.getIsolationContext(module_id);
        if (ctx.usage.memory_used_bytes + bytes > ctx.limits.memory_limit_bytes) {
            return error.ModuleResourceLimitExceeded;
        }

        ctx.usage.memory_used_bytes += bytes;
        self.memory_used_bytes += bytes;
    }

    pub fn freeModuleMemory(self: *Kernel, actor: Role, module_id: []const u8, bytes: u64) !void {
        try self.enforce(actor, .manage_memory);

        var ctx = try self.getIsolationContext(module_id);

        if (bytes >= ctx.usage.memory_used_bytes) {
            self.memory_used_bytes -= ctx.usage.memory_used_bytes;
            ctx.usage.memory_used_bytes = 0;
        } else {
            ctx.usage.memory_used_bytes -= bytes;
            self.memory_used_bytes -= bytes;
        }
    }

    pub fn spawnModuleProcess(self: *Kernel, actor: Role, module_id: []const u8, name: []const u8) !u64 {
        try self.enforce(actor, .manage_processes);

        var ctx = try self.getIsolationContext(module_id);
        if (ctx.usage.process_count >= ctx.limits.max_processes) {
            return error.ModuleResourceLimitExceeded;
        }

        const pid = try self.spawnProcess(actor, name, .module_runtime);
        ctx.usage.process_count += 1;
        return pid;
    }

    pub fn createModuleFile(self: *Kernel, actor: Role, module_id: []const u8, path: []const u8, size_bytes: u64) !void {
        try self.enforce(actor, .manage_files);

        var ctx = try self.getIsolationContext(module_id);
        if (ctx.usage.file_count >= ctx.limits.max_files) {
            return error.ModuleResourceLimitExceeded;
        }

        try self.createFile(actor, path, .module_runtime, size_bytes);
        ctx.usage.file_count += 1;
    }

    pub fn reserveModuleResources(self: *Kernel, actor: Role, module_id: []const u8, units: u32) !void {
        try self.enforce(actor, .manage_modules);

        var ctx = try self.getIsolationContext(module_id);
        if (ctx.usage.resource_units_used + units > ctx.limits.max_resource_units) {
            return error.ModuleResourceLimitExceeded;
        }

        ctx.usage.resource_units_used += units;
    }

    pub fn registerPlatformComponent(
        self: *Kernel,
        actor: Role,
        component_id: []const u8,
        class: ComponentClass,
        endpoint: []const u8,
        network_manager: []const u8,
        limits: ModuleLimits,
    ) !void {
        try self.enforce(actor, .manage_modules);
        if (self.component_isolation.get(component_id) != null) {
            return error.ComponentAlreadyRegistered;
        }

        const id_copy = try self.allocator.dupe(u8, component_id);
        errdefer self.allocator.free(id_copy);
        const endpoint_copy = try self.allocator.dupe(u8, endpoint);
        errdefer self.allocator.free(endpoint_copy);
        const network_copy = try self.allocator.dupe(u8, network_manager);
        errdefer self.allocator.free(network_copy);

        try self.component_isolation.put(id_copy, .{
            .component_id = id_copy,
            .class = class,
            .limits = limits,
            .usage = .{
                .memory_used_bytes = 0,
                .process_count = 0,
                .file_count = 0,
                .resource_units_used = 0,
            },
            .network_ingress_bytes = 0,
            .network_egress_bytes = 0,
            .network_manager = network_copy,
            .endpoint = endpoint_copy,
            .status = .active,
            .registered_at_ms = std.time.milliTimestamp(),
        });

        try self.publishEvent(actor, "kernel.component.registered", "{}", std.time.milliTimestamp());
    }

    pub fn bootstrapCorePlatformComponents(self: *Kernel, actor: Role) !void {
        try self.registerPlatformComponent(
            actor,
            "kogi.kernel",
            .kernel,
            "local://kogi-kernel",
            "kernel-native",
            .{
                .memory_limit_bytes = 1024 * 1024 * 1024,
                .max_processes = 512,
                .max_files = 20000,
                .max_resource_units = 40000,
            },
        );
        try self.registerPlatformComponent(
            actor,
            "kogi.host",
            .host,
            "local://kogi-host",
            "kernel-native",
            .{
                .memory_limit_bytes = 768 * 1024 * 1024,
                .max_processes = 384,
                .max_files = 16000,
                .max_resource_units = 30000,
            },
        );
        try self.registerPlatformComponent(
            actor,
            "kogi.server",
            .server,
            "http://127.0.0.1:8080/health",
            "kogi-go-network",
            .{
                .memory_limit_bytes = 768 * 1024 * 1024,
                .max_processes = 256,
                .max_files = 12000,
                .max_resource_units = 24000,
            },
        );
        try self.registerPlatformComponent(
            actor,
            "kogi.engine",
            .engine,
            "local://kogi-engine",
            "kogi-go-network",
            .{
                .memory_limit_bytes = 1024 * 1024 * 1024,
                .max_processes = 256,
                .max_files = 12000,
                .max_resource_units = 32000,
            },
        );
        try self.registerPlatformComponent(
            actor,
            "kogi.services",
            .services,
            "http://127.0.0.1:8090/health",
            "kogi-go-network",
            .{
                .memory_limit_bytes = 768 * 1024 * 1024,
                .max_processes = 256,
                .max_files = 12000,
                .max_resource_units = 24000,
            },
        );

        try self.allocateComponentMemory(actor, "kogi.host", 32 * 1024 * 1024);
        try self.allocateComponentMemory(actor, "kogi.server", 48 * 1024 * 1024);
        try self.allocateComponentMemory(actor, "kogi.engine", 64 * 1024 * 1024);
        _ = try self.spawnComponentProcess(actor, "kogi.host", "host-orchestrator");
        _ = try self.spawnComponentProcess(actor, "kogi.server", "server-router");
        _ = try self.spawnComponentProcess(actor, "kogi.engine", "engine-stream-processor");
        try self.createComponentFile(actor, "kogi.engine", "/engine/flows.log", 4096);
        try self.reserveComponentResources(actor, "kogi.services", 256);
        try self.recordComponentNetwork(actor, "kogi.services", 4096, 8192);
    }

    pub fn setComponentLimits(self: *Kernel, actor: Role, component_id: []const u8, limits: ModuleLimits) !void {
        try self.enforce(actor, .manage_modules);
        var ctx = try self.getComponentContext(component_id);
        ctx.limits = limits;
    }

    pub fn allocateComponentMemory(self: *Kernel, actor: Role, component_id: []const u8, bytes: u64) !void {
        try self.enforce(actor, .manage_memory);

        if (self.memory_used_bytes + bytes > self.memory_total_bytes) {
            return error.OutOfMemory;
        }

        var ctx = try self.getComponentContext(component_id);
        if (ctx.usage.memory_used_bytes + bytes > ctx.limits.memory_limit_bytes) {
            return error.ComponentResourceLimitExceeded;
        }

        ctx.usage.memory_used_bytes += bytes;
        self.memory_used_bytes += bytes;
    }

    pub fn freeComponentMemory(self: *Kernel, actor: Role, component_id: []const u8, bytes: u64) !void {
        try self.enforce(actor, .manage_memory);

        var ctx = try self.getComponentContext(component_id);
        if (bytes >= ctx.usage.memory_used_bytes) {
            self.memory_used_bytes -= ctx.usage.memory_used_bytes;
            ctx.usage.memory_used_bytes = 0;
        } else {
            ctx.usage.memory_used_bytes -= bytes;
            self.memory_used_bytes -= bytes;
        }
    }

    pub fn spawnComponentProcess(self: *Kernel, actor: Role, component_id: []const u8, name: []const u8) !u64 {
        try self.enforce(actor, .manage_processes);

        var ctx = try self.getComponentContext(component_id);
        if (ctx.usage.process_count >= ctx.limits.max_processes) {
            return error.ComponentResourceLimitExceeded;
        }

        const owner = componentOwnerRole(ctx.class);
        const pid = try self.spawnProcess(actor, name, owner);
        ctx.usage.process_count += 1;
        return pid;
    }

    pub fn createComponentFile(
        self: *Kernel,
        actor: Role,
        component_id: []const u8,
        path: []const u8,
        size_bytes: u64,
    ) !void {
        try self.enforce(actor, .manage_files);

        var ctx = try self.getComponentContext(component_id);
        if (ctx.usage.file_count >= ctx.limits.max_files) {
            return error.ComponentResourceLimitExceeded;
        }

        const owner = componentOwnerRole(ctx.class);
        try self.createFile(actor, path, owner, size_bytes);
        ctx.usage.file_count += 1;
    }

    pub fn reserveComponentResources(
        self: *Kernel,
        actor: Role,
        component_id: []const u8,
        units: u32,
    ) !void {
        try self.enforce(actor, .manage_modules);

        var ctx = try self.getComponentContext(component_id);
        if (ctx.usage.resource_units_used + units > ctx.limits.max_resource_units) {
            return error.ComponentResourceLimitExceeded;
        }

        ctx.usage.resource_units_used += units;
    }

    pub fn recordComponentNetwork(
        self: *Kernel,
        actor: Role,
        component_id: []const u8,
        ingress_bytes: u64,
        egress_bytes: u64,
    ) !void {
        try self.enforce(actor, .manage_modules);

        var ctx = try self.getComponentContext(component_id);
        ctx.network_ingress_bytes += ingress_bytes;
        ctx.network_egress_bytes += egress_bytes;
    }

    pub fn getComponentIsolation(self: *Kernel, component_id: []const u8) ?ComponentIsolationContext {
        if (self.component_isolation.get(component_id)) |ctx| {
            return ctx;
        }
        return null;
    }

    pub fn getModuleIsolation(self: *Kernel, module_id: []const u8) ?ModuleIsolationContext {
        if (self.module_isolation.get(module_id)) |ctx| {
            return ctx;
        }
        return null;
    }

    pub fn publishEvent(self: *Kernel, actor: Role, topic: []const u8, payload: []const u8, at_ms: i64) !void {
        try self.enforce(actor, .manage_modules);

        const topic_copy = try self.allocator.dupe(u8, topic);
        errdefer self.allocator.free(topic_copy);
        const payload_copy = try self.allocator.dupe(u8, payload);
        errdefer self.allocator.free(payload_copy);

        try self.events.append(.{
            .topic = topic_copy,
            .payload = payload_copy,
            .emitted_at_ms = at_ms,
        });
    }

    pub fn schedule(self: *Kernel, actor: Role, id: []const u8, topic: []const u8, payload: []const u8, due_at_ms: i64, interval_ms: ?u64) !void {
        try self.enforce(actor, .schedule_tasks);

        try self.scheduler.append(.{
            .id = try self.allocator.dupe(u8, id),
            .topic = try self.allocator.dupe(u8, topic),
            .payload = try self.allocator.dupe(u8, payload),
            .due_at_ms = due_at_ms,
            .interval_ms = interval_ms,
            .enabled = true,
        });
    }

    pub fn tick(self: *Kernel, actor: Role, now_ms: i64) !u64 {
        try self.enforce(actor, .schedule_tasks);
        var triggered: u64 = 0;

        for (self.scheduler.items) |*entry| {
            if (!entry.enabled) continue;
            if (entry.due_at_ms > now_ms) continue;

            try self.publishEvent(actor, entry.topic, entry.payload, now_ms);
            triggered += 1;

            if (entry.interval_ms) |interval| {
                entry.due_at_ms = now_ms + @as(i64, @intCast(interval));
            } else {
                entry.enabled = false;
            }
        }

        return triggered;
    }

    pub fn allocateMemory(self: *Kernel, actor: Role, bytes: u64) !void {
        try self.enforce(actor, .manage_memory);
        if (self.memory_used_bytes + bytes > self.memory_total_bytes) {
            return error.OutOfMemory;
        }
        self.memory_used_bytes += bytes;
    }

    pub fn freeMemory(self: *Kernel, actor: Role, bytes: u64) !void {
        try self.enforce(actor, .manage_memory);
        if (bytes >= self.memory_used_bytes) {
            self.memory_used_bytes = 0;
            return;
        }
        self.memory_used_bytes -= bytes;
    }

    pub fn putCache(self: *Kernel, actor: Role, key: []const u8, value: []const u8, ttl_ms: u64) !void {
        try self.enforce(actor, .manage_memory);

        const key_copy = try self.allocator.dupe(u8, key);
        errdefer self.allocator.free(key_copy);
        const value_copy = try self.allocator.dupe(u8, value);
        errdefer self.allocator.free(value_copy);

        const expires_at = std.time.milliTimestamp() + @as(i64, @intCast(ttl_ms));

        if (self.cache.getPtr(key_copy)) |existing| {
            self.allocator.free(existing.value);
            existing.* = .{ .value = value_copy, .expires_at_ms = expires_at };
            self.allocator.free(key_copy);
        } else {
            try self.cache.put(key_copy, .{ .value = value_copy, .expires_at_ms = expires_at });
        }
    }

    pub fn getCache(self: *Kernel, key: []const u8) ?[]const u8 {
        const now = std.time.milliTimestamp();
        if (self.cache.getPtr(key)) |entry| {
            if (entry.expires_at_ms < now) return null;
            return entry.value;
        }
        return null;
    }

    pub fn spawnProcess(self: *Kernel, actor: Role, name: []const u8, owner: Role) !u64 {
        try self.enforce(actor, .manage_processes);

        const pid = self.next_pid;
        self.next_pid += 1;

        try self.processes.append(.{
            .pid = pid,
            .owner = owner,
            .name = try self.allocator.dupe(u8, name),
            .state = .ready,
            .created_at_ms = std.time.milliTimestamp(),
        });

        return pid;
    }

    pub fn setProcessState(self: *Kernel, actor: Role, pid: u64, state: ProcessState) !void {
        try self.enforce(actor, .manage_processes);

        for (self.processes.items) |*process| {
            if (process.pid == pid) {
                process.state = state;
                return;
            }
        }
        return error.NotFound;
    }

    pub fn createFile(self: *Kernel, actor: Role, path: []const u8, owner: Role, size_bytes: u64) !void {
        try self.enforce(actor, .manage_files);

        try self.files.append(.{
            .path = try self.allocator.dupe(u8, path),
            .owner = owner,
            .size_bytes = size_bytes,
            .created_at_ms = std.time.milliTimestamp(),
        });
    }

    pub fn stats(self: *Kernel) KernelStats {
        var ingress_total: u64 = 0;
        var egress_total: u64 = 0;
        var component_it = self.component_isolation.valueIterator();
        while (component_it.next()) |component| {
            ingress_total += component.network_ingress_bytes;
            egress_total += component.network_egress_bytes;
        }

        return .{
            .module_count = self.modules.count(),
            .isolated_module_count = self.module_isolation.count(),
            .component_count = self.component_isolation.count(),
            .event_count = self.events.items.len,
            .scheduled_count = self.scheduler.items.len,
            .process_count = self.processes.items.len,
            .file_count = self.files.items.len,
            .memory_used_bytes = self.memory_used_bytes,
            .memory_total_bytes = self.memory_total_bytes,
            .network_ingress_bytes = ingress_total,
            .network_egress_bytes = egress_total,
            .mode = self.mode,
        };
    }

    fn getIsolationContext(self: *Kernel, module_id: []const u8) !*ModuleIsolationContext {
        return self.module_isolation.getPtr(module_id) orelse error.ModuleNotFound;
    }

    fn getComponentContext(self: *Kernel, component_id: []const u8) !*ComponentIsolationContext {
        return self.component_isolation.getPtr(component_id) orelse error.ComponentNotFound;
    }
};

pub const KernelStats = struct {
    module_count: usize,
    isolated_module_count: usize,
    component_count: usize,
    event_count: usize,
    scheduled_count: usize,
    process_count: usize,
    file_count: usize,
    memory_used_bytes: u64,
    memory_total_bytes: u64,
    network_ingress_bytes: u64,
    network_egress_bytes: u64,
    mode: Mode,
};

pub fn defaultModuleLimits() ModuleLimits {
    return .{
        .memory_limit_bytes = 512 * 1024 * 1024,
        .max_processes = 128,
        .max_files = 5000,
        .max_resource_units = 10000,
    };
}

fn componentOwnerRole(class: ComponentClass) Role {
    return switch (class) {
        .kernel => .root,
        .host => .host,
        .server => .server,
        .engine => .server,
        .services => .server,
        .module => .module_runtime,
    };
}

pub fn hasPermission(role: Role, permission: Permission) bool {
    return switch (role) {
        .root => true,
        .host => switch (permission) {
            .kernel_admin,
            .schedule_tasks,
            .manage_modules,
            .manage_memory,
            .manage_processes,
            .manage_files,
            .read_audit,
            => true,
        },
        .server => switch (permission) {
            .schedule_tasks,
            .manage_modules,
            .manage_processes,
            .manage_files,
            => true,
            .kernel_admin,
            .manage_memory,
            .read_audit,
            => false,
        },
        .module_runtime => switch (permission) {
            .schedule_tasks,
            .manage_files,
            => true,
            .kernel_admin,
            .manage_modules,
            .manage_memory,
            .manage_processes,
            .read_audit,
            => false,
        },
        .user => permission == .read_audit,
    };
}

test "kernel registers modules and enforces security" {
    var kernel = Kernel.init(std.testing.allocator);
    defer kernel.deinit();

    try kernel.registerModule(.host, "kogi.office", "office");

    const denied = kernel.registerModule(.user, "x", "y");
    try std.testing.expectError(AccessDenied.AccessDenied, denied);

    const s = kernel.stats();
    try std.testing.expectEqual(@as(usize, 1), s.module_count);
    try std.testing.expectEqual(@as(usize, 1), s.isolated_module_count);
    try std.testing.expectEqual(@as(usize, 1), s.component_count);
}

test "module resource isolation limits are enforced" {
    var kernel = Kernel.init(std.testing.allocator);
    defer kernel.deinit();

    try kernel.registerModule(.host, "kogi.exchange", "exchange");
    try kernel.setModuleLimits(.host, "kogi.exchange", .{
        .memory_limit_bytes = 1024,
        .max_processes = 1,
        .max_files = 1,
        .max_resource_units = 10,
    });

    try kernel.allocateModuleMemory(.host, "kogi.exchange", 900);
    try std.testing.expectError(
        error.ModuleResourceLimitExceeded,
        kernel.allocateModuleMemory(.host, "kogi.exchange", 200),
    );

    _ = try kernel.spawnModuleProcess(.host, "kogi.exchange", "exchange-worker");
    try std.testing.expectError(
        error.ModuleResourceLimitExceeded,
        kernel.spawnModuleProcess(.host, "kogi.exchange", "exchange-worker-2"),
    );

    try kernel.createModuleFile(.host, "kogi.exchange", "/tmp/exchange.log", 128);
    try std.testing.expectError(
        error.ModuleResourceLimitExceeded,
        kernel.createModuleFile(.host, "kogi.exchange", "/tmp/exchange-2.log", 128),
    );
}

test "office bootstrap provisions endpoint and resources" {
    var kernel = Kernel.init(std.testing.allocator);
    defer kernel.deinit();

    try kernel.bootstrapOfficeModule(.host);

    const ctx = kernel.getModuleIsolation("kogi.office") orelse return error.ModuleNotFound;
    try std.testing.expectEqualStrings("kogi.office", ctx.module_id);
    try std.testing.expectEqualStrings("/services/office", ctx.service_endpoint);
    try std.testing.expect(ctx.usage.memory_used_bytes > 0);
    try std.testing.expect(ctx.usage.process_count >= 2);
}

test "core platform components are provisioned and network tracked" {
    var kernel = Kernel.init(std.testing.allocator);
    defer kernel.deinit();

    try kernel.bootstrapCorePlatformComponents(.host);
    const services_before = kernel.getComponentIsolation("kogi.services") orelse return error.ComponentNotFound;
    try kernel.recordComponentNetwork(.host, "kogi.services", 128, 256);

    const ctx = kernel.getComponentIsolation("kogi.engine") orelse return error.ComponentNotFound;
    try std.testing.expectEqual(ComponentClass.engine, ctx.class);
    try std.testing.expect(ctx.usage.memory_used_bytes > 0);

    const services = kernel.getComponentIsolation("kogi.services") orelse return error.ComponentNotFound;
    try std.testing.expectEqual(services_before.network_ingress_bytes + 128, services.network_ingress_bytes);
    try std.testing.expectEqual(services_before.network_egress_bytes + 256, services.network_egress_bytes);
}
