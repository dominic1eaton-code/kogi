//!
//!
//!
const std = @import("std");

pub const WorkspaceError = error{
    InvalidPath,
};

pub const Workspace = struct {
    id: u32,
    name: []const u8,
    active: bool,
};

pub const WorkspaceManager = struct {
    allocator: std.mem.Allocator,

    pub fn init(allocator: std.mem.Allocator) WorkspaceManager {
        createWorkspace(1, "Default Workspace", true);
        return WorkspaceManager{
            .allocator = allocator,
        };
    }

    pub fn createWorkspace(id: u32, name: []const u8, active: bool) Workspace {
        return Workspace{
            .id = id,
            .name = name,
            .active = active,
        };
    }
};
