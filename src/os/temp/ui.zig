const std = @import("std");

pub const UIError = error{
    InvalidComponent,
};

pub const UIComponent = struct {
    id: u32,
    name: []const u8,
    active: bool,
};

pub const UIWindow = struct {
    id: u32,
    title: []const u8,
    active: bool,
};

pub const UIButton = struct {
    id: u32,
    label: []const u8,
    active: bool,
};

pub const UIInput = struct {
    id: u32,
    placeholder: []const u8,
    active: bool,
};

pub const UI = struct {
    id: u32,
    name: []const u8,
    active: bool,
};

pub const Shell = struct {
    id: u32,
    name: []const u8,
    active: bool,
};

pub const Terminal = struct {
    id: u32,
    name: []const u8,
    active: bool,
};

pub const GUI = struct {
    id: u32,
    name: []const u8,
    active: bool,
};

pub const CLI = struct {
    id: u32,
    name: []const u8,
    active: bool,
};

pub const GUIManager = struct {
    allocator: std.mem.Allocator,
    // Add GUI-related fields here

    pub fn init(allocator: std.mem.Allocator) GUIManager {
        return GUIManager{
            .allocator = allocator,
            // Initialize other fields as needed
        };
    }

    // pub fn deinit(self: *GUIManager) void {
    //     // Clean up resources if needed
    // }

    // pub const createGUI = fn(id: u32, name: []const u8, active: bool) GUI {
    //     return GUI{
    //         .id = id,
    //         .name = name,
    //         .active = active,
    //     };
    // }
};

pub const UIManager = struct {
    allocator: std.mem.Allocator,
    // Add UI-related fields here

    pub fn init(allocator: std.mem.Allocator) UIManager {
        return UIManager{
            .allocator = allocator,
            // Initialize other fields as needed
        };
    }

    // pub fn deinit(self: *UIManager) void {
    //     // Clean up resources if needed
    // }

    pub const createUI = fn(id: u32, name: []const u8, active: bool) UI {
        return UI{
            .id = id,
            .name = name,
            .active = active,
        };
    }

    pub const createShell = fn(id: u32, name: []const u8, active: bool) Shell {
        return Shell{
            .id = id,
            .name = name,
            .active = active,
        };
    }

    pub const createTerminal = fn(id: u32, name: []const u8, active: bool) Terminal {
        return Terminal{
            .id = id,
            .name = name,
            .active = active,
        };
    }

    pub const createCLI = fn(id: u32, name: []const u8, active: bool) CLI {
        return CLI{
            .id = id,
            .name = name,
            .active = active,
        };
    }
};
