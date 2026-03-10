const std = @import("std");
const kernel_mod = @import("kernel.zig");

var kernel_instance: ?kernel_mod.Kernel = null;

fn cStringToSlice(ptr: [*:0]const u8) []const u8 {
    return std.mem.span(ptr);
}

pub export fn kogi_kernel_init() bool {
    if (kernel_instance != null) return true;

    var kernel = kernel_mod.Kernel.init(std.heap.page_allocator);
    const now = std.time.milliTimestamp();
    kernel.publishEvent(.host, "kernel.boot", "{}", now) catch {
        kernel.deinit();
        return false;
    };
    kernel_instance = kernel;
    return true;
}

pub export fn kogi_kernel_deinit() void {
    if (kernel_instance) |*kernel| {
        kernel.deinit();
    }
    kernel_instance = null;
}

pub export fn kogi_kernel_register_module(module_id: [*:0]const u8, module_kind: [*:0]const u8) bool {
    if (kernel_instance == null) return false;

    const id = cStringToSlice(module_id);
    const kind = cStringToSlice(module_kind);

    if (kernel_instance) |*kernel| {
        kernel.registerModule(.host, id, kind) catch return false;
        return true;
    }
    return false;
}

pub export fn kogi_kernel_bootstrap_office() bool {
    if (kernel_instance == null) return false;

    if (kernel_instance) |*kernel| {
        kernel.bootstrapOfficeModule(.host) catch return false;
        return true;
    }
    return false;
}

pub export fn kogi_kernel_bootstrap_core_components() bool {
    if (kernel_instance == null) return false;

    if (kernel_instance) |*kernel| {
        kernel.bootstrapCorePlatformComponents(.host) catch return false;
        return true;
    }
    return false;
}

pub export fn kogi_kernel_register_component(
    component_id: [*:0]const u8,
    class_tag: i32,
    endpoint: [*:0]const u8,
    network_manager: [*:0]const u8,
) bool {
    if (kernel_instance == null) return false;

    const id = cStringToSlice(component_id);
    const endpoint_slice = cStringToSlice(endpoint);
    const network_slice = cStringToSlice(network_manager);

    const class: kernel_mod.ComponentClass = switch (class_tag) {
        0 => .kernel,
        1 => .host,
        2 => .server,
        3 => .engine,
        4 => .services,
        else => .module,
    };

    if (kernel_instance) |*kernel| {
        kernel.registerPlatformComponent(
            .host,
            id,
            class,
            endpoint_slice,
            network_slice,
            kernel_mod.defaultModuleLimits(),
        ) catch return false;
        return true;
    }
    return false;
}

pub export fn kogi_kernel_publish_event(topic: [*:0]const u8, payload: [*:0]const u8) bool {
    if (kernel_instance == null) return false;

    const event_topic = cStringToSlice(topic);
    const event_payload = cStringToSlice(payload);

    if (kernel_instance) |*kernel| {
        kernel.publishEvent(.server, event_topic, event_payload, std.time.milliTimestamp()) catch return false;
        return true;
    }
    return false;
}

pub export fn kogi_kernel_set_mode(mode: i32) bool {
    if (kernel_instance == null) return false;

    if (kernel_instance) |*kernel| {
        const next: kernel_mod.Mode = if (mode == 0) .kernel else .user;
        kernel.setMode(next, .host) catch return false;
        return true;
    }
    return false;
}

pub export fn kogi_kernel_event_count() u64 {
    if (kernel_instance) |*kernel| {
        return @intCast(kernel.events.items.len);
    }
    return 0;
}

pub export fn kogi_kernel_component_count() u64 {
    if (kernel_instance) |*kernel| {
        return @intCast(kernel.component_isolation.count());
    }
    return 0;
}
