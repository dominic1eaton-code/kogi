//! By convention, root.zig is the root source file when making a library.
//! KOGI is an operating system for independent workers.
const std = @import("std");

// Core OS modules
const system_module = @import("system.zig");
const identity_module = @import("identity.zig");
const users_module = @import("users.zig");
const providers_module = @import("providers.zig");
const account_management_module = @import("accounts.zig");
const profile_management_module = @import("profile.zig");
const contacts_module = @import("contacts.zig");
const content_module = @import("content.zig");
const design_module = @import("design.zig");
const directory_book_module = @import("directory_book.zig");
const database_module = @import("database.zig");
const workspace_module = @import("workspace.zig");
const portfolio_module = @import("portfolio.zig");
const ideas_module = @import("ideas.zig");
const registry_module = @import("registry.zig");
const directory_module = @import("directory.zig");
const vault_module = @import("vault.zig");
const security_module = @import("security.zig");
const logging_module = @import("logging.zig");
const events_module = @import("events.zig");
const state_module = @import("state.zig");
const distributed_module = @import("node.zig");
const networking_module = @import("networking.zig");
const cpu_module = @import("cpu.zig");
const drivers_module = @import("device_drivers.zig");
// portfolio.zig already exists; imported via earlier root exports under workspace
const calendar_module = @import("calendar.zig");
const time_module = @import("time.zig");
const scheduler_module = @import("scheduler.zig");
const observability_module = @import("observability.zig");
const processes_module = @import("processes.zig");
const memory_module = @import("memory.zig");
const trace_module = @import("trace.zig");
const bootloader_module = @import("bootloader.zig");
const cli_module = @import("cli.zig");

const kernel_module = @import("kernel.zig");
// ========== Core KOGI OS Exports ==========

// System (Core OS)
pub const System = system_module.System;
pub const Task = system_module.Task;
pub const Engagement = system_module.Engagement;
pub const EngagementStatus = system_module.EngagementStatus;

pub const Kernel = kernel_module.Kernel;
pub const IdentityType = identity_module.IdentityType;
pub const User = users_module.User;
pub const UserRole = users_module.UserRole;
pub const ProfileType = users_module.ProfileType;
pub const ProfileEntry = users_module.ProfileEntry;
pub const UserCredential = users_module.Credential;
pub const CredentialKind = users_module.CredentialKind;
pub const AuthToken = users_module.AuthToken;
pub const IssuedToken = users_module.IssuedToken;
pub const AuthTokenKind = users_module.AuthTokenKind;
pub const UserKey = users_module.UserKey;
pub const KeyKind = users_module.KeyKind;
pub const UserCertificate = users_module.UserCertificate;
pub const UserPermission = users_module.Permission;
pub const UserPrivilege = users_module.Privilege;
pub const EncryptedPayload = users_module.EncryptedPayload;
pub const ProfileAccount = users_module.ProfileAccount;
pub const UserAccountType = users_module.AccountType;
pub const UserProfile = users_module.UserProfile;
pub const Persona = users_module.Persona;
pub const UserContact = users_module.UserContact;
pub const ContactKind = users_module.ContactKind;
pub const ContactChannelKind = users_module.ContactChannelKind;
pub const UserManager = users_module.UserManager;
pub const UserError = users_module.UserError;
pub const Provider = providers_module.Provider;
pub const ProviderCategory = providers_module.ProviderCategory;
pub const ProviderManager = providers_module.ProviderManager;
pub const ProviderError = providers_module.ProviderError;
pub const ManagedAccount = account_management_module.ManagedAccount;
pub const ManagedAccountType = account_management_module.AccountType;
pub const AccountManagement = account_management_module.AccountManagement;
pub const AccountManagementError = account_management_module.AccountManagementError;
pub const ManagedProfile = profile_management_module.ManagedProfile;
pub const ManagedProfileType = profile_management_module.ProfileType;
pub const ProfileProperty = profile_management_module.ProfileProperty;
pub const ProfileManagement = profile_management_module.ProfileManagement;
pub const ProfileManagementError = profile_management_module.ProfileManagementError;
pub const ManagedContact = contacts_module.ManagedContact;
pub const ManagedContactKind = contacts_module.ContactKind;
pub const ContactChannel = contacts_module.ContactChannel;
pub const ManagedContactChannelKind = contacts_module.ContactChannelKind;
pub const ContactManagement = contacts_module.ContactManagement;
pub const ContactManagementError = contacts_module.ContactManagementError;
pub const ContentKind = content_module.ContentKind;
pub const ContentMetadata = content_module.ContentMetadata;
pub const ContentItem = content_module.ContentItem;
pub const Note = content_module.Note;
pub const NoteImportance = content_module.NoteImportance;
pub const ContentManager = content_module.ContentManager;
pub const ContentError = content_module.ContentError;
pub const DesignKind = design_module.DesignKind;
pub const DesignStatus = design_module.DesignStatus;
pub const DesignMetadata = design_module.DesignMetadata;
pub const DesignItem = design_module.DesignItem;
pub const DesignRevision = design_module.DesignRevision;
pub const DesignManager = design_module.DesignManager;
pub const DesignError = design_module.DesignError;
pub const DirectoryEntityKind = directory_book_module.DirectoryEntityKind;
pub const RelationshipStatus = directory_book_module.RelationshipStatus;
pub const RelationshipStrength = directory_book_module.RelationshipStrength;
pub const RelationshipLink = directory_book_module.RelationshipLink;
pub const DirectoryBookEntry = directory_book_module.DirectoryEntry;
pub const DirectoryBookOrganization = directory_book_module.DirectoryOrganization;
pub const DirectoryBookManager = directory_book_module.DirectoryBookManager;
pub const DirectoryBookError = directory_book_module.DirectoryBookError;
pub const ColumnType = database_module.ColumnType;
pub const TransactionOp = database_module.TransactionOp;
pub const RecordField = database_module.RecordField;
pub const TableColumn = database_module.TableColumn;
pub const Database = database_module.Database;
pub const Table = database_module.Table;
pub const Record = database_module.Record;
pub const Transaction = database_module.Transaction;
pub const DatabaseManager = database_module.DatabaseManager;
pub const DatabaseError = database_module.DatabaseError;

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

// Project Tracking
pub const ProjectTrackingManager = ideas_module.ProjectTrackingManager;
pub const TrackedProject = ideas_module.TrackedProject;
pub const ProjectStatus = ideas_module.ProjectStatus;
pub const ProjectPriority = ideas_module.ProjectPriority;
pub const ProjectTaskStatus = ideas_module.TaskStatus;
pub const ProjectMilestoneStatus = ideas_module.MilestoneStatus;
pub const ProjectIssueSeverity = ideas_module.IssueSeverity;
pub const ProjectProgressUpdateType = ideas_module.ProgressUpdateType;
pub const ProjectTaskItem = ideas_module.ProjectTask;
pub const ProjectMilestoneItem = ideas_module.ProjectMilestone;
pub const ProjectIssueItem = ideas_module.ProjectIssue;
pub const ProjectNoteItem = ideas_module.ProjectNote;
pub const ProjectProgressUpdate = ideas_module.ProjectProgressUpdate;
pub const ProjectFilter = ideas_module.ProjectFilter;
pub const ProjectSummary = ideas_module.ProjectSummary;
pub const StoryType = ideas_module.StoryType;
pub const StoryStatus = ideas_module.StoryStatus;
pub const Story = ideas_module.Story;
pub const StoryFilter = ideas_module.StoryFilter;
pub const StorySummary = ideas_module.StorySummary;
pub const WorkBreakdownStructure = ideas_module.WorkBreakdownStructure;
pub const WorkPackage = ideas_module.WorkPackage;
pub const WbsTheme = ideas_module.WbsTheme;
pub const WbsInitiative = ideas_module.WbsInitiative;
pub const WbsEpic = ideas_module.WbsEpic;
pub const WbsStory = ideas_module.WbsStory;
pub const WbsTask = ideas_module.WbsTask;

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

// CPU Abstraction
pub const ICPU = cpu_module.ICPU;
pub const ICPUStatus = cpu_module.ICPUStatus;
pub const CPUManager = cpu_module.CPUManager;

// Device Drivers
pub const IDD = drivers_module.IDD;
pub const IDDStatus = drivers_module.IDDStatus;
pub const DriverManager = drivers_module.DriverManager;

// Calendar & Scheduling
pub const CalendarEvent = calendar_module.CalendarEvent;
pub const CalendarScheduler = calendar_module.Scheduler;

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

// Scheduling & Dispatch System
pub const Scheduler = scheduler_module.Scheduler;
pub const Dispatcher = scheduler_module.Dispatcher;
pub const TaskQueue = scheduler_module.TaskQueue;
pub const PriorityQueue = scheduler_module.PriorityQueue;
pub const DelayedQueue = scheduler_module.DelayedQueue;
pub const WorkStealingQueue = scheduler_module.WorkStealingQueue;
pub const TaskPriorityLevel = scheduler_module.TaskPriority;
pub const TaskState = scheduler_module.TaskState;

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
pub const SystemKernelCompat = system_module.System;

// Worker compatibility
pub const Worker = identity_module.Identity;
pub const WorkerManager = identity_module.IdentityManager;
pub const WorkerType = identity_module.IdentityType;

// Job & Contract compatibility
pub const Job = system_module.Task;
pub const Contract = system_module.Engagement;

// Portfolio compatibility (now mapped to portfolio_module)
pub const Portfolio = portfolio_module.Portfolio;
pub const SubPortfolio = portfolio_module.SubPortfolio;
pub const Program = portfolio_module.Program;
pub const Project = portfolio_module.Project;
pub const PortfolioManager = portfolio_module.PortfolioManager;
pub const PortfolioItem = portfolio_module.PortfolioItem;
pub const PortfolioCollection = portfolio_module.PortfolioCollection;

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

// Time & Clock Management
pub const TimePoint = time_module.TimePoint;
pub const Duration = time_module.Duration;
pub const DateTime = time_module.DateTime;
pub const TimeZone = time_module.TimeZone;
pub const Clock = time_module.Clock;
pub const Timer = time_module.Timer;
pub const Profiler = time_module.Profiler;
pub const StopWatch = time_module.StopWatch;
pub const TimeSource = time_module.TimeSource;
pub const SystemTimeSource = time_module.SystemTimeSource;

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
