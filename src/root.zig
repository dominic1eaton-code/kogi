//! By convention, root.zig is the root source file when making a library.
//! KOGI is an operating system for independent workers.
const std = @import("std");

// Core OS modules
const system_module = @import("system.zig");
const identity_module = @import("identity.zig");
const workspace_module = @import("workspace.zig");
const registry_module = @import("registry.zig");
const directory_module = @import("directory.zig");
const vault_module = @import("vault.zig");
const cli_module = @import("cli.zig");

// ========== Core KOGI OS Exports ==========

// System (Core OS)
pub const System = system_module.System;
pub const Task = system_module.Task;
pub const Engagement = system_module.Engagement;
pub const EngagementStatus = system_module.EngagementStatus;

// Identity Management
pub const Identity = identity_module.Identity;
pub const IdentityManager = identity_module.IdentityManager;
pub const IdentityType = identity_module.IdentityType;

// Workspace (Personal Work Environment)
pub const Workspace = workspace_module.Workspace;
pub const WorkspaceManager = workspace_module.WorkspaceManager;
pub const WorkspaceItem = workspace_module.WorkspaceItem;
pub const Collection = workspace_module.Collection;

// Workspace metadata and search
pub const EntityType = workspace_module.EntityType;
pub const EntityClass = workspace_module.EntityClass;
pub const Metadata = workspace_module.Metadata;
pub const Tag = workspace_module.Tag;
pub const SearchFilter = workspace_module.SearchFilter;
pub const IndexEntry = workspace_module.IndexEntry;

// Connection Registry
pub const Connection = registry_module.Connection;
pub const ConnectionRegistry = registry_module.ConnectionRegistry;
pub const ConnectionType = registry_module.ConnectionType;

// Directory (Contact Management)
pub const Contact = directory_module.Contact;
pub const Organization = directory_module.Organization;
pub const Directory = directory_module.Directory;

// Vault (Asset Management)
pub const VaultItem = vault_module.VaultItem;
pub const Vault = vault_module.Vault;
pub const VaultItemType = vault_module.VaultItemType;

// ========== Backward Compatibility Aliases ==========

// System compatibility
pub const Kernel = system_module.System;

// Worker compatibility
pub const Worker = identity_module.Identity;
pub const WorkerManager = identity_module.IdentityManager;
pub const WorkerType = identity_module.IdentityType;

// Job & Contract compatibility
pub const Job = system_module.Task;
pub const Contract = system_module.Engagement;

// Portfolio compatibility
pub const Portfolio = workspace_module.Workspace;
pub const SubPortfolio = workspace_module.SubWorkspace;
pub const Program = workspace_module.Program;
pub const Project = workspace_module.Project;
pub const PortfolioManager = workspace_module.WorkspaceManager;
pub const PortfolioItem = workspace_module.WorkspaceItem;
pub const PortfolioCollection = workspace_module.Collection;

// Account compatibility
pub const Account = registry_module.Connection;
pub const AccountManager = registry_module.ConnectionRegistry;
pub const AccountType = registry_module.ConnectionType;

// CRM compatibility
pub const Client = directory_module.Organization;
pub const CRMManager = directory_module.Directory;

// Assets compatibility
pub const Asset = vault_module.VaultItem;
pub const AssetManager = vault_module.Vault;
pub const AssetType = vault_module.VaultItemType;

pub fn bufferedPrint() !void {
    // Stdout is for the actual output of your application, for example if you
    // are implementing gzip, then only the compressed bytes should be sent to
    // stdout, not any debugging messages.
    var stdout_buffer: [1024]u8 = undefined;
    var stdout_writer = std.fs.File.stdout().writer(&stdout_buffer);
    const stdout = &stdout_writer.interface;

    try stdout.print("Run `zig build test` to run the tests.\n", .{});

    try stdout.flush(); // Don't forget to flush!
}

pub fn startCLI(allocator: std.mem.Allocator, system: *system_module.System) !void {
    try cli_module.startCLI(allocator, system);
}
