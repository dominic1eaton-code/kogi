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
const security_module = @import("security.zig");
const logging_module = @import("logging.zig");
const events_module = @import("events.zig");
const state_module = @import("state.zig");
const distributed_module = @import("distributed.zig");
const networking_module = @import("networking.zig");
const observability_module = @import("observability.zig");
const processes_module = @import("processes.zig");
const memory_module = @import("memory.zig");
const trace_module = @import("trace.zig");
const bootloader_module = @import("bootloader.zig");
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

// Security Management
pub const SecurityManager = security_module.SecurityManager;
pub const Credential = security_module.Credential;
pub const Session = security_module.Session;
pub const Role = security_module.Role;
pub const Permission = security_module.Permission;
pub const RoleAssignment = security_module.RoleAssignment;
pub const RolePermissions = security_module.RolePermissions;
pub const AccessLevel = security_module.AccessLevel;
pub const AccessControlEntry = security_module.AccessControlEntry;
pub const AuditLogEntry = security_module.AuditLogEntry;
pub const SecurityEventType = security_module.SecurityEventType;
pub const MFAMethod = security_module.MFAMethod;
pub const MFAConfig = security_module.MFAConfig;
pub const HashAlgorithm = security_module.HashAlgorithm;
pub const initializeDefaultRoles = security_module.initializeDefaultRoles;

// Logging System
pub const Logger = logging_module.Logger;
pub const LogLevel = logging_module.LogLevel;
pub const LogEntry = logging_module.LogEntry;
pub const LogOutput = logging_module.LogOutput;
pub const LoggerConfig = logging_module.LoggerConfig;
pub const initGlobalLogger = logging_module.initGlobalLogger;
pub const getGlobalLogger = logging_module.getGlobalLogger;
pub const deinitGlobalLogger = logging_module.deinitGlobalLogger;

// Event Management System
pub const EventBus = events_module.EventBus;
pub const Event = events_module.Event;
pub const EventType = events_module.EventType;
pub const EventHandler = events_module.EventHandler;

// State Management System
pub const StateManager = state_module.StateManager;
pub const Checkpoint = state_module.Checkpoint;
pub const Backup = state_module.Backup;
pub const RestorePoint = state_module.RestorePoint;
pub const CompressionType = state_module.CompressionType;
pub const StateManagerConfig = state_module.StateManagerConfig;

// Distributed Node System
pub const DistributedCluster = distributed_module.DistributedCluster;
pub const Node = distributed_module.Node;
pub const NodeStatus = distributed_module.NodeStatus;
pub const NodeRole = distributed_module.NodeRole;
pub const HealthCheckResult = distributed_module.HealthCheckResult;
pub const ReplicationState = distributed_module.ReplicationState;
pub const LoadBalancingStrategy = distributed_module.LoadBalancingStrategy;
pub const ClusterConfig = distributed_module.ClusterConfig;

// Networking & Server System
pub const NetworkServer = networking_module.NetworkServer;
pub const NetworkClient = networking_module.NetworkClient;
pub const NetworkRequest = networking_module.NetworkRequest;
pub const NetworkResponse = networking_module.NetworkResponse;
pub const NetworkConnection = networking_module.Connection;
pub const ServerConfig = networking_module.ServerConfig;
pub const Router = networking_module.Router;
pub const Endpoint = networking_module.Endpoint;
pub const Protocol = networking_module.Protocol;
pub const RequestMethod = networking_module.RequestMethod;
pub const ResponseStatus = networking_module.ResponseStatus;

// Observability System
pub const ObservabilityManager = observability_module.ObservabilityManager;
pub const Breakpoint = observability_module.Breakpoint;
pub const BreakpointType = observability_module.BreakpointType;
pub const SystemMetrics = observability_module.SystemMetrics;
pub const PerformanceProfile = observability_module.PerformanceProfile;
pub const OptimizationHint = observability_module.OptimizationHint;

// Process Management System
pub const ProcessManager = processes_module.ProcessManager;
pub const ProcessState = processes_module.ProcessState;
pub const ProcessPriority = processes_module.ProcessPriority;
pub const Process = processes_module.Process;
pub const ProcessResources = processes_module.ProcessResources;
pub const ProcessEvent = processes_module.ProcessEvent;
pub const ProcessEventType = processes_module.ProcessEventType;
pub const SchedulerConfig = processes_module.SchedulerConfig;

// Memory Management System
pub const MemoryAllocator = memory_module.MemoryAllocator;
pub const MemoryPage = memory_module.MemoryPage;
pub const MemoryPermissions = memory_module.MemoryPermissions;
pub const MemoryRegion = memory_module.MemoryRegion;
pub const MemoryRegionType = memory_module.MemoryRegionType;
pub const MemoryStats = memory_module.MemoryStats;

// Trace & Audit Management System
pub const TraceManager = trace_module.TraceManager;
pub const TraceEvent = trace_module.TraceEvent;
pub const TraceEventType = trace_module.TraceEventType;
pub const AuditManager = trace_module.AuditManager;
pub const AuditRecord = trace_module.AuditRecord;
pub const AuditActionType = trace_module.AuditActionType;
pub const AuditResult = trace_module.AuditResult;
pub const AuditSeverity = trace_module.AuditSeverity;
pub const PerformanceMetrics = trace_module.PerformanceMetrics;

// Bootloader & Kernel Management System
pub const BootManager = bootloader_module.BootManager;
pub const BootConfig = bootloader_module.BootConfig;
pub const BootMode = bootloader_module.BootMode;
pub const BootPhase = bootloader_module.BootPhase;
pub const BootDeviceType = bootloader_module.BootDeviceType;
pub const KernelModule = bootloader_module.KernelModule;
pub const BootStats = bootloader_module.BootStats;
pub const BootEvent = bootloader_module.BootEvent;
pub const BootSeverity = bootloader_module.BootSeverity;

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
