const std = @import("std");

// Custom application interface
pub const CustomApplication = struct {
    name: []const u8,
    start: *const fn (system: *anyopaque) void,
    stop: *const fn (system: *anyopaque) void,
    status: *const fn (system: *anyopaque) []const u8,
};
/// Dashboard manager for OS system dashboard
pub const DashboardManager = struct {
    pub fn init(allocator: std.mem.Allocator) DashboardManager {
        // Initialization logic for dashboard
        return DashboardManager{};
    }
    pub fn deinit(self: *DashboardManager) void {
        // Cleanup logic for dashboard
    }
};

/// UI connector for OS system
pub const UserMode = enum { user, kernel };

pub const UIConnector = struct {
    pub fn init(allocator: std.mem.Allocator) UIConnector {
        // Initialization logic for UI connector
        return UIConnector{};
    }
    pub fn deinit(self: *UIConnector) void {
        // Cleanup logic for UI connector
    }
};

/// Application management system
pub const ApplicationManager = struct {
    pub fn init(allocator: std.mem.Allocator) ApplicationManager {
        // Initialization logic for application manager
        return ApplicationManager{};
    }
    pub fn deinit(self: *ApplicationManager) void {
        // Cleanup logic for application manager
    }
};

/// Session and session management system
pub const SessionManager = struct {
    pub fn init(allocator: std.mem.Allocator) SessionManager {
        // Initialization logic for session manager
        return SessionManager{};
    }
    pub fn deinit(self: *SessionManager) void {
        // Cleanup logic for session manager
    }
};
const identity_module = @import("identity.zig");
const users_module = @import("users.zig");
const providers_module = @import("providers.zig");
const account_management_module = @import("accounts.zig");
const profile_management_module = @import("profile.zig");
const contacts_module = @import("contacts.zig");
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
const cpu_module = @import("cpu.zig");
const drivers_module = @import("device_drivers.zig");
const calendar_module = @import("calendar.zig");
const time_module = @import("time.zig");
const scheduler_module = @import("scheduler.zig");
const strategy_module = @import("strategy.zig");
const cache_module = @import("cache.zig");
const kernel_module = @import("kernel.zig");
const processes_module = @import("processes.zig");
const memory_module = @import("memory.zig");
const trace_module = @import("trace.zig");
const bootloader_module = @import("bootloader.zig");

/// Task represents a unit of work that can be assigned to an identity
pub const Task = struct {
    id: u32,
    title: []const u8,
    description: []const u8,
    required_skills: std.ArrayList([]const u8),
    budget: f32,
    deadline: i64,
    assigned_to: ?u32,
    completed: bool,
};

/// Engagement status
pub const EngagementStatus = enum { active, completed, terminated };

/// Engagement links an identity to a task for a specific time period
pub const Engagement = struct {
    id: u32,
    identity_id: u32,
    task_id: u32,
    start_date: i64,
    end_date: ?i64,
    hourly_rate: f32,
    hours_worked: f32,
    status: EngagementStatus,
    completion_date: ?i64 = null,
};

/// System is the core operating system engine managing all subsystems
pub const System = struct {
    allocator: std.mem.Allocator,
    custom_apps: std.ArrayList(CustomApplication),
    tiered_cache: cache_module.TieredAllocator,
    user_manager: users_module.UserManager,
    workspace_manager: workspace_module.WorkspaceManager,
    directory: directory_module.Directory,
    vault: vault_module.Vault,
    security_manager: security_module.SecurityManager,
    logger: logging_module.Logger,
    event_bus: events_module.EventBus,
    state_manager: state_module.StateManager,
    cluster: distributed_module.DistributedCluster,
    network_server: networking_module.NetworkServer,
    process_manager: processes_module.ProcessManager,
    memory_allocator: memory_module.MemoryAllocator,
    strategy_manager: ?strategy_module.StrategicManager,
    trace_manager: trace_module.TraceManager,
    audit_manager: trace_module.AuditManager,
    boot_manager: bootloader_module.BootManager,
    observability: observability_module.ObservabilityManager,
    cpu_manager: cpu_module.CPUManager,
    driver_manager: drivers_module.DriverManager,
    scheduler: scheduler_module.Scheduler,
    dispatcher: ?scheduler_module.Dispatcher,
    clock: time_module.Clock,
    profiler: time_module.Profiler,
    mode: UserMode,
    kernel: ?*kernel_module.Kernel,
    tasks: std.ArrayList(Task),
    engagements: std.ArrayList(Engagement),
    dashboard: ?DashboardManager,
    ui_connector: ?UIConnector,
    app_manager: ?ApplicationManager,
    session_manager: ?SessionManager,

    pub fn init(allocator: std.mem.Allocator, kernel: ?*kernel_module.Kernel) System {
        // Initialize logger with default config
        var log_outputs = std.ArrayList(logging_module.LogOutput){};
        log_outputs.append(allocator, .stdout) catch {};
        const logger_config = logging_module.LoggerConfig{
            .min_level = .info,
            .outputs = log_outputs,
        };

        // Initialize state manager with default config
        const state_config = state_module.StateManagerConfig{};

        // Initialize cluster with default config
        const cluster_config = distributed_module.ClusterConfig{
            .cluster_name = "kogi-cluster",
            .node_id = 0,
        };

        // Initialize network server with default config
        const server_config = networking_module.ServerConfig{};

        // Initialize process manager with default config
        const scheduler_config = processes_module.SchedulerConfig{};

        // Initialize memory allocator with 4GB total memory
        const memory_allocator_config = 4096; // 4GB

        // Initialize trace manager
        const trace_manager = trace_module.TraceManager.init(allocator);

        // Initialize audit manager
        const audit_manager = trace_module.AuditManager.init(allocator);

        // Initialize boot manager with default config
        const boot_config = bootloader_module.BootConfig{};

        // initialize tiered cache allocator
        var tiered = cache_module.TieredAllocator.init(allocator) catch unreachable;
        const cache_alloc = tiered.asAllocator();

        // Initialize observability manager
        const observability_mgr = observability_module.ObservabilityManager.init(allocator);

        // initialize cpu and driver managers
        const cpu_mgr = cpu_module.CPUManager.init(allocator);
        const driver_mgr = drivers_module.DriverManager.init(allocator);

        // initialize scheduler from scheduling module
        const scheduler = scheduler_module.Scheduler.init(allocator);
        const dispatcher = scheduler_module.Dispatcher.init(allocator, @constCast(&scheduler), 4);

        // initialize clock and profiler
        const clock = time_module.Clock.init(allocator);
        const profiler = time_module.Profiler.init(allocator);

        const dashboard = DashboardManager.init(allocator);
        const ui_connector = UIConnector.init(allocator);
        const app_manager = ApplicationManager.init(allocator);
        const session_manager = SessionManager.init(allocator);

        var custom_apps = std.ArrayList(CustomApplication){};

        return System{
            .allocator = cache_alloc,
            .tiered_cache = tiered,
            .user_manager = users_module.UserManager.init(cache_alloc),
            .workspace_manager = workspace_module.WorkspaceManager.init(cache_alloc),
            .directory = directory_module.Directory.init(cache_alloc),
            .vault = vault_module.Vault.init(cache_alloc),
            .security_manager = security_module.SecurityManager.init(cache_alloc),
            .logger = logging_module.Logger.init(cache_alloc, logger_config) catch unreachable,
            .event_bus = events_module.EventBus.init(cache_alloc),
            .state_manager = state_module.StateManager.init(cache_alloc, state_config) catch unreachable,
            .cluster = distributed_module.DistributedCluster.init(cache_alloc, cluster_config),
            .network_server = networking_module.NetworkServer.init(cache_alloc, server_config),
            .process_manager = processes_module.ProcessManager.init(cache_alloc, scheduler_config),
            .memory_allocator = memory_module.MemoryAllocator.init(cache_alloc, memory_allocator_config),
            .strategy_manager = null,
            .trace_manager = trace_manager,
            .audit_manager = audit_manager,
            .boot_manager = bootloader_module.BootManager.init(cache_alloc, boot_config),
            .observability = observability_mgr,
            .cpu_manager = cpu_mgr,
            .driver_manager = driver_mgr,
            .scheduler = scheduler,
            .dispatcher = dispatcher,
            .clock = clock,
            .profiler = profiler,
            .tasks = std.ArrayList(Task){},
            .engagements = std.ArrayList(Engagement){},
            .dashboard = dashboard,
            .ui_connector = ui_connector,
            .app_manager = app_manager,
            .session_manager = session_manager,
            .custom_apps = custom_apps,
            .mode = .user,
            .kernel = kernel,
        };
    }

    pub fn deinit(self: *System) void {
        // Clean up system-owned components (always)
        self.user_manager.deinit();
        self.workspace_manager.deinit();
        self.directory.deinit();
        self.vault.deinit();
        self.security_manager.deinit();
        self.logger.deinit();
        self.event_bus.deinit();
        self.state_manager.deinit();
        self.cluster.deinit();

        // If System was created with an external kernel, the kernel owns
        // memory/process/CPU/driver/time/scheduler related deinitialization.
        if (self.kernel) |*k| {
            // System-specific cleanup only
            // Clean up tiered cache and system-managed lists
            self.tiered_cache.deinit();

            for (self.tasks.items) |*task| {
                self.allocator.free(task.title);
                self.allocator.free(task.description);
                for (task.required_skills.items) |skill| {
                    self.allocator.free(skill);
                }
                task.required_skills.deinit(self.allocator);
            }
            self.tasks.deinit(self.allocator);

            for (self.engagements.items) |*engagement| {
                engagement.completion_date = null;
            }
            self.engagements.deinit(self.allocator);

            if (self.strategy_manager) |*s| s.deinit();
            if (self.dashboard) |*d| d.deinit();
            if (self.ui_connector) |*u| u.deinit();
            if (self.app_manager) |*a| a.deinit();
            if (self.session_manager) |*s| s.deinit();
            return;
        }

        // Otherwise, System owns and should deinit all subsystems it created
        self.network_server.deinit();
        var process_manager = self.process_manager;
        process_manager.deinit();
        if (self.dispatcher) |*d| {
            d.stop();
            d.deinit();
        }
        var memory_allocator = self.memory_allocator;
        memory_allocator.deinit();
        var trace_manager = self.trace_manager;
        trace_manager.deinit();
        var audit_manager = self.audit_manager;
        audit_manager.deinit();
        var boot_manager = self.boot_manager;
        boot_manager.deinit();
        var observability_mgr = self.observability;
        observability_mgr.deinit();
        var cpu_mgr = self.cpu_manager;
        cpu_mgr.deinit();
        var driver_mgr = self.driver_manager;
        driver_mgr.deinit();
        var scheduler = self.scheduler;
        scheduler.deinit();
        var profiler = self.profiler;
        profiler.deinit();
        self.tiered_cache.deinit();

        // Clean up tasks
        for (self.tasks.items) |*task| {
            self.allocator.free(task.title);
            self.allocator.free(task.description);
            for (task.required_skills.items) |skill| {
                self.allocator.free(skill);
            }
            task.required_skills.deinit(self.allocator);
        }
        self.tasks.deinit(self.allocator);

        // Clean up engagements
        for (self.engagements.items) |*engagement| {
            engagement.completion_date = null;
        }
        self.engagements.deinit(self.allocator);

        if (self.strategy_manager) |*s| s.deinit();
        if (self.dashboard) |*d| d.deinit();
        if (self.ui_connector) |*u| u.deinit();
        if (self.app_manager) |*a| a.deinit();
        if (self.session_manager) |*s| s.deinit();
    }

    // ========== Identity Management Delegation ==========

    /// Create a new identity in the system
    pub fn createIdentity(
        self: *System,
        name: []const u8,
        email: []const u8,
        hourly_rate: f32,
        itype: identity_module.IdentityType,
    ) !u32 {
        return try self.user_manager.createIdentity(name, email, hourly_rate, itype);
    }

    /// Add a skill to an identity
    pub fn addIdentitySkill(self: *System, identity_id: u32, skill: []const u8) !void {
        try self.user_manager.addIdentitySkill(identity_id, skill);
    }

    /// Get count of active identities
    pub fn getActiveIdentitiesCount(self: *System) u32 {
        return self.user_manager.getActiveIdentitiesCount();
    }

    pub fn getIdentities(self: *System) []identity_module.Identity {
        return self.user_manager.getIdentities();
    }

    // ========== User Management ==========

    pub fn createUser(
        self: *System,
        username: []const u8,
        email: []const u8,
        password: []const u8,
        role: users_module.UserRole,
    ) !u32 {
        return try self.user_manager.createUser(username, email, password, role);
    }

    pub fn getUsers(self: *System) []users_module.User {
        return self.user_manager.getUsers();
    }

    pub fn disableUser(self: *System, user_id: u32) !void {
        try self.user_manager.disableUser(user_id);
    }

    pub fn loginUser(self: *System, username: []const u8, password: []const u8) !u32 {
        return try self.user_manager.login(username, password);
    }

    pub fn logoutUser(self: *System) !void {
        try self.user_manager.logout();
    }

    pub fn getCurrentUser(self: *System) ?users_module.User {
        return self.user_manager.getCurrentUser();
    }

    pub fn addUserCredential(
        self: *System,
        user_id: u32,
        label: []const u8,
        kind: users_module.CredentialKind,
        secret: []const u8,
    ) !u32 {
        return try self.user_manager.addCredential(user_id, label, kind, secret);
    }

    pub fn disableUserCredential(self: *System, credential_id: u32) !void {
        try self.user_manager.disableCredential(credential_id);
    }

    pub fn getUserCredentials(self: *System) []users_module.Credential {
        return self.user_manager.getCredentials();
    }

    pub fn setUserPassword(self: *System, user_id: u32, password: []const u8) !void {
        try self.user_manager.setPassword(user_id, password);
    }

    pub fn setUserProfile(
        self: *System,
        user_id: u32,
        display_name: []const u8,
        bio: []const u8,
        timezone: []const u8,
        locale: []const u8,
        avatar_url: []const u8,
    ) !void {
        try self.user_manager.setProfile(user_id, display_name, bio, timezone, locale, avatar_url);
    }

    pub fn getUserProfile(self: *System, user_id: u32) !users_module.UserProfile {
        return try self.user_manager.getProfile(user_id);
    }

    pub fn getUserProfiles(self: *System) []users_module.UserProfile {
        return self.user_manager.getProfiles();
    }

    pub fn createUserProfile(
        self: *System,
        user_id: u32,
        profile_type: users_module.ProfileType,
        name: []const u8,
        description: []const u8,
    ) !u32 {
        return try self.user_manager.createProfile(user_id, profile_type, name, description);
    }

    pub fn setActiveUserProfile(self: *System, user_id: u32, profile_id: u32) !void {
        try self.user_manager.setActiveProfile(user_id, profile_id);
    }

    pub fn updateUserProfileMetadata(self: *System, profile_id: u32, name: []const u8, description: []const u8) !void {
        try self.user_manager.updateProfileMetadata(profile_id, name, description);
    }

    pub fn setUserProfileConfiguration(self: *System, profile_id: u32, key: []const u8, value: []const u8) !void {
        try self.user_manager.setProfileConfiguration(profile_id, key, value);
    }

    pub fn setUserProfilePreference(self: *System, profile_id: u32, key: []const u8, value: []const u8) !void {
        try self.user_manager.setProfilePreference(profile_id, key, value);
    }

    pub fn getUserProfileById(self: *System, profile_id: u32) !users_module.UserProfile {
        return try self.user_manager.getProfileById(profile_id);
    }

    pub fn getUserActiveProfile(self: *System, user_id: u32) ?users_module.UserProfile {
        return self.user_manager.getActiveProfile(user_id);
    }

    pub fn addUserProfileAccount(
        self: *System,
        profile_id: u32,
        account_type: users_module.AccountType,
        username: []const u8,
        provider: []const u8,
        details: []const u8,
    ) !u32 {
        return try self.user_manager.addProfileAccount(profile_id, account_type, username, provider, details);
    }

    pub fn disableUserProfileAccount(self: *System, account_id: u32) !void {
        try self.user_manager.disableProfileAccount(account_id);
    }

    pub fn getUserProfileAccounts(self: *System) []users_module.ProfileAccount {
        return self.user_manager.getProfileAccounts();
    }

    pub fn createUserContact(
        self: *System,
        user_id: u32,
        profile_id: ?u32,
        kind: users_module.ContactKind,
        display_name: []const u8,
        notes: []const u8,
    ) !u32 {
        return try self.user_manager.createContact(user_id, profile_id, kind, display_name, notes);
    }

    pub fn setUserContactProfile(self: *System, user_id: u32, contact_id: u32, profile_id: ?u32) !void {
        try self.user_manager.setContactProfile(user_id, contact_id, profile_id);
    }

    pub fn updateUserContact(self: *System, user_id: u32, contact_id: u32, display_name: []const u8, notes: []const u8) !void {
        try self.user_manager.updateContact(user_id, contact_id, display_name, notes);
    }

    pub fn addUserContactChannel(
        self: *System,
        user_id: u32,
        contact_id: u32,
        kind: users_module.ContactChannelKind,
        label: []const u8,
        value: []const u8,
    ) !void {
        try self.user_manager.addContactChannel(user_id, contact_id, kind, label, value);
    }

    pub fn verifyUserContactChannel(self: *System, user_id: u32, contact_id: u32, channel_index: usize) !void {
        try self.user_manager.verifyContactChannel(user_id, contact_id, channel_index);
    }

    pub fn addUserContactTag(self: *System, user_id: u32, contact_id: u32, tag: []const u8) !void {
        try self.user_manager.addContactTag(user_id, contact_id, tag);
    }

    pub fn linkUserContactToAccount(self: *System, user_id: u32, contact_id: u32, account_id: u32) !void {
        try self.user_manager.linkContactToAccount(user_id, contact_id, account_id);
    }

    pub fn unlinkUserContactFromAccount(self: *System, user_id: u32, contact_id: u32, account_id: u32) !void {
        try self.user_manager.unlinkContactFromAccount(user_id, contact_id, account_id);
    }

    pub fn deactivateUserContact(self: *System, user_id: u32, contact_id: u32) !void {
        try self.user_manager.deactivateContact(user_id, contact_id);
    }

    pub fn getUserContacts(self: *System) []users_module.UserContact {
        return self.user_manager.getContacts();
    }

    pub fn getUserContactById(self: *System, contact_id: u32) !users_module.UserContact {
        return try self.user_manager.getContactById(contact_id);
    }

    pub fn addUserPersona(
        self: *System,
        user_id: u32,
        name: []const u8,
        description: []const u8,
        focus_area: []const u8,
        tone: []const u8,
    ) !u32 {
        return try self.user_manager.addPersona(user_id, name, description, focus_area, tone);
    }

    pub fn activateUserPersona(self: *System, user_id: u32, persona_id: u32) !void {
        try self.user_manager.activatePersona(user_id, persona_id);
    }

    pub fn getUserActivePersona(self: *System, user_id: u32) ?users_module.Persona {
        return self.user_manager.getActivePersona(user_id);
    }

    pub fn getUserPersonas(self: *System) []users_module.Persona {
        return self.user_manager.getPersonas();
    }

    // ========== Provider Management ==========

    pub fn addProvider(
        self: *System,
        name: []const u8,
        slug: []const u8,
        category: providers_module.ProviderCategory,
        homepage_url: []const u8,
        api_base_url: []const u8,
    ) !u32 {
        return try self.user_manager.addProvider(name, slug, category, homepage_url, api_base_url);
    }

    pub fn disableProvider(self: *System, provider_id: u32) !void {
        try self.user_manager.disableProvider(provider_id);
    }

    pub fn enableProvider(self: *System, provider_id: u32) !void {
        try self.user_manager.enableProvider(provider_id);
    }

    pub fn getProviders(self: *System) []providers_module.Provider {
        return self.user_manager.getProviders();
    }

    pub fn getProvider(self: *System, provider_id: u32) !providers_module.Provider {
        return try self.user_manager.getProvider(provider_id);
    }

    pub fn findProviderBySlug(self: *System, slug: []const u8) ?providers_module.Provider {
        return self.user_manager.findProviderBySlug(slug);
    }

    // ========== Account Management System ==========

    pub fn createManagedAccount(
        self: *System,
        owner_user_id: u32,
        account_type: account_management_module.AccountType,
        provider_slug: []const u8,
        account_handle: []const u8,
        metadata: []const u8,
    ) !u32 {
        if (!self.user_manager.provider_manager.isActiveBySlug(provider_slug)) {
            return providers_module.ProviderError.ProviderNotFound;
        }
        return try self.user_manager.account_management.createAccount(
            owner_user_id,
            account_type,
            provider_slug,
            account_handle,
            metadata,
        );
    }

    pub fn associateManagedAccountToProfile(self: *System, account_id: u32, profile_id: ?u32) !void {
        try self.user_manager.account_management.associateToProfile(account_id, profile_id);
        if (profile_id) |pid| {
            try self.user_manager.profile_management.linkAccount(pid, account_id);
        }
    }

    pub fn deactivateManagedAccount(self: *System, account_id: u32) !void {
        try self.user_manager.account_management.deactivateAccount(account_id);
    }

    pub fn getManagedAccounts(self: *System) []account_management_module.ManagedAccount {
        return self.user_manager.account_management.getAccounts();
    }

    // ========== Profile Management System ==========

    pub fn createManagedProfile(
        self: *System,
        owner_user_id: u32,
        profile_type: profile_management_module.ProfileType,
        name: []const u8,
        description: []const u8,
    ) !u32 {
        return try self.user_manager.profile_management.createProfile(owner_user_id, profile_type, name, description);
    }

    pub fn setActiveManagedProfile(self: *System, owner_user_id: u32, profile_id: u32) !void {
        try self.user_manager.profile_management.setActiveProfile(owner_user_id, profile_id);
    }

    pub fn setManagedProfileConfiguration(self: *System, profile_id: u32, key: []const u8, value: []const u8) !void {
        try self.user_manager.profile_management.setConfiguration(profile_id, key, value);
    }

    pub fn setManagedProfilePreference(self: *System, profile_id: u32, key: []const u8, value: []const u8) !void {
        try self.user_manager.profile_management.setPreference(profile_id, key, value);
    }

    pub fn linkManagedAccountToProfile(self: *System, profile_id: u32, account_id: u32) !void {
        try self.user_manager.profile_management.linkAccount(profile_id, account_id);
        try self.user_manager.account_management.associateToProfile(account_id, profile_id);
    }

    pub fn unlinkManagedAccountFromProfile(self: *System, profile_id: u32, account_id: u32) !void {
        try self.user_manager.profile_management.unlinkAccount(profile_id, account_id);
        try self.user_manager.account_management.associateToProfile(account_id, null);
    }

    pub fn getManagedProfiles(self: *System) []profile_management_module.ManagedProfile {
        return self.user_manager.profile_management.getProfiles();
    }

    // ========== Contact Management System ==========

    pub fn createManagedContact(
        self: *System,
        owner_user_id: u32,
        profile_id: ?u32,
        kind: contacts_module.ContactKind,
        display_name: []const u8,
        notes: []const u8,
    ) !u32 {
        return try self.user_manager.contact_management.createContact(owner_user_id, profile_id, kind, display_name, notes);
    }

    pub fn setManagedContactProfile(self: *System, contact_id: u32, profile_id: ?u32) !void {
        try self.user_manager.contact_management.setProfile(contact_id, profile_id);
    }

    pub fn updateManagedContact(self: *System, contact_id: u32, display_name: []const u8, notes: []const u8) !void {
        try self.user_manager.contact_management.updateDetails(contact_id, display_name, notes);
    }

    pub fn addManagedContactChannel(
        self: *System,
        contact_id: u32,
        kind: contacts_module.ContactChannelKind,
        label: []const u8,
        value: []const u8,
    ) !void {
        try self.user_manager.contact_management.addChannel(contact_id, kind, label, value);
    }

    pub fn addManagedContactTag(self: *System, contact_id: u32, tag: []const u8) !void {
        try self.user_manager.contact_management.addTag(contact_id, tag);
    }

    pub fn linkManagedContactToAccount(self: *System, contact_id: u32, account_id: u32) !void {
        try self.user_manager.contact_management.linkAccount(contact_id, account_id);
    }

    pub fn unlinkManagedContactFromAccount(self: *System, contact_id: u32, account_id: u32) !void {
        try self.user_manager.contact_management.unlinkAccount(contact_id, account_id);
    }

    pub fn deactivateManagedContact(self: *System, contact_id: u32) !void {
        try self.user_manager.contact_management.deactivateContact(contact_id);
    }

    pub fn getManagedContacts(self: *System) []contacts_module.ManagedContact {
        return self.user_manager.contact_management.getContacts();
    }

    /// Connection management delegation: add a connection for an identity
    pub fn addIdentityConnection(
        self: *System,
        identity_id: u32,
        conn_type: registry_module.ConnectionType,
        username: []const u8,
        provider: []const u8,
        details: []const u8,
    ) !u32 {
        return try self.user_manager.addIdentityConnection(identity_id, conn_type, username, provider, details);
    }

    pub fn deactivateIdentityConnection(self: *System, identity_id: u32, conn_id: u32) !void {
        try self.user_manager.deactivateIdentityConnection(identity_id, conn_id);
    }

    pub fn getIdentityActiveConnectionCount(self: *System, identity_id: u32) u32 {
        return self.user_manager.getIdentityActiveConnectionCount(identity_id);
    }

    // ========== Task Management ==========

    /// Post a new task to the system
    pub fn postTask(self: *System, title: []const u8, description: []const u8, budget: f32, deadline: i64) !u32 {
        const task_id = @as(u32, @intCast(self.tasks.items.len));

        const task = Task{
            .id = task_id,
            .title = try self.allocator.dupe(u8, title),
            .description = try self.allocator.dupe(u8, description),
            .required_skills = std.ArrayList([]const u8){},
            .budget = budget,
            .deadline = deadline,
            .assigned_to = null,
            .completed = false,
        };

        try self.tasks.append(self.allocator, task);
        return task_id;
    }

    /// Add a skill requirement to a task
    pub fn addTaskSkillRequirement(self: *System, task_id: u32, skill: []const u8) !void {
        if (task_id < @as(u32, @intCast(self.tasks.items.len))) {
            const skill_copy = try self.allocator.dupe(u8, skill);
            try self.tasks.items[task_id].required_skills.append(self.allocator, skill_copy);
        }
    }

    /// Get count of active tasks
    pub fn getActiveTasksCount(self: *System) u32 {
        var count: u32 = 0;
        for (self.tasks.items) |task| {
            if (!task.completed) {
                count += 1;
            }
        }
        return count;
    }

    pub fn getTasks(self: *System) []Task {
        return self.tasks.items;
    }

    // ========== Engagement Management ==========

    /// Create an engagement between an identity and a task
    pub fn createEngagement(
        self: *System,
        identity_id: u32,
        task_id: u32,
        start_date: i64,
        hourly_rate: f32,
    ) !u32 {
        const engagement_id = @as(u32, @intCast(self.engagements.items.len));

        const engagement = Engagement{
            .id = engagement_id,
            .identity_id = identity_id,
            .task_id = task_id,
            .start_date = start_date,
            .end_date = null,
            .hourly_rate = hourly_rate,
            .hours_worked = 0.0,
            .status = .active,
        };

        try self.engagements.append(self.allocator, engagement);
        if (task_id < @as(u32, @intCast(self.tasks.items.len))) {
            self.tasks.items[task_id].assigned_to = identity_id;
        }
        return engagement_id;
    }

    /// Log hours to an engagement
    pub fn logHours(self: *System, engagement_id: u32, hours: f32) !void {
        if (engagement_id < @as(u32, @intCast(self.engagements.items.len))) {
            self.engagements.items[engagement_id].hours_worked += hours;
        }
    }

    /// Complete an engagement
    pub fn completeEngagement(self: *System, engagement_id: u32, completion_date: i64) !void {
        if (engagement_id < @as(u32, @intCast(self.engagements.items.len))) {
            self.engagements.items[engagement_id].status = .completed;
            self.engagements.items[engagement_id].completion_date = completion_date;
            self.tasks.items[self.engagements.items[engagement_id].task_id].completed = true;
        }
    }

    /// Calculate earnings for an identity
    pub fn calculateIdentityEarnings(self: *System, identity_id: u32) f32 {
        var earnings: f32 = 0.0;
        for (self.engagements.items) |engagement| {
            if (engagement.identity_id == identity_id and engagement.status == .completed) {
                earnings += engagement.hours_worked * engagement.hourly_rate;
            }
        }
        return earnings;
    }

    pub fn getEngagements(self: *System) []Engagement {
        return self.engagements.items;
    }
};

// Backward compatibility alias
pub const Kernel = System;

pub fn runSystem() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var system = System.init(allocator);
    defer system.deinit();

    // Demo
    _ = try system.createIdentity("Alice", "alice@example.com", 50.0, .developer);
    try system.addIdentitySkill(0, "Zig");

    _ = try system.postTask("Build Parser", "Create a parser", 1000.0, 0);
    try system.addTaskSkillRequirement(0, "Zig");

    _ = try system.createEngagement(0, 0, 0, 50.0);
    try system.logHours(0, 40.0);
    try system.completeEngagement(0, 0);

    std.debug.print("Identity earnings: {d}\n", .{system.calculateIdentityEarnings(0)});
}
