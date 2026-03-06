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
        _ = allocator;
        // Initialization logic for dashboard
        return DashboardManager{};
    }
    pub fn deinit(self: *DashboardManager) void {
        _ = self;
        // Cleanup logic for dashboard
    }
};

/// UI connector for OS system
pub const UserMode = enum { user, kernel };

pub const UIConnector = struct {
    pub fn init(allocator: std.mem.Allocator) UIConnector {
        _ = allocator;
        // Initialization logic for UI connector
        return UIConnector{};
    }
    pub fn deinit(self: *UIConnector) void {
        _ = self;
        // Cleanup logic for UI connector
    }
};

/// Application management system
pub const ApplicationManager = struct {
    pub fn init(allocator: std.mem.Allocator) ApplicationManager {
        _ = allocator;
        // Initialization logic for application manager
        return ApplicationManager{};
    }
    pub fn deinit(self: *ApplicationManager) void {
        _ = self;
        // Cleanup logic for application manager
    }
};

/// Session and session management system
pub const SessionManager = struct {
    pub fn init(allocator: std.mem.Allocator) SessionManager {
        _ = allocator;
        // Initialization logic for session manager
        return SessionManager{};
    }
    pub fn deinit(self: *SessionManager) void {
        _ = self;
        // Cleanup logic for session manager
    }
};
const identity_module = @import("identity.zig");
const users_module = @import("users.zig");
const providers_module = @import("providers.zig");
const account_management_module = @import("accounts.zig");
const profile_management_module = @import("profile.zig");
const contacts_module = @import("contacts.zig");
const content_module = @import("content.zig");
const design_module = @import("design.zig");
const ideas_module = @import("ideas.zig");
const directory_book_module = @import("directory_book.zig");
const database_module = @import("database.zig");
const workspace_module = @import("workspace.zig");
const registry_module = @import("registry.zig");
const directory_module = @import("directory.zig");
const vault_module = @import("vault.zig");
const security_module = @import("security.zig");
const logging_module = @import("logging.zig");
const events_module = @import("events.zig");
const state_module = @import("state.zig");
const distributed_module = @import("node.zig");
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
    required_skills: std.array_list.Managed([]const u8),
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
    backing_allocator: std.mem.Allocator,
    custom_apps: std.array_list.Managed(CustomApplication),
    tiered_cache: *cache_module.TieredAllocator,
    user_manager: users_module.UserManager,
    workspace_manager: workspace_module.WorkspaceManager,
    project_tracking: ideas_module.ProjectTrackingManager,
    content_manager: content_module.ContentManager,
    design_manager: design_module.DesignManager,
    directory_book_manager: directory_book_module.DirectoryBookManager,
    database_manager: database_module.DatabaseManager,
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
    tasks: std.array_list.Managed(Task),
    engagements: std.array_list.Managed(Engagement),
    dashboard: ?DashboardManager,
    ui_connector: ?UIConnector,
    app_manager: ?ApplicationManager,
    session_manager: ?SessionManager,

    pub fn init(allocator: std.mem.Allocator, kernel: ?*kernel_module.Kernel) System {
        // Initialize logger with default config
        var log_outputs = std.array_list.Managed(logging_module.LogOutput).init(allocator);
        log_outputs.append(.stdout) catch {};
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

        // initialize tiered cache allocator on heap so allocator pointer remains stable
        const tiered_ptr = allocator.create(cache_module.TieredAllocator) catch unreachable;
        tiered_ptr.* = cache_module.TieredAllocator.init(allocator) catch unreachable;
        const cache_alloc = tiered_ptr.asAllocator();

        // Initialize observability manager
        const observability_mgr = observability_module.ObservabilityManager.init(allocator);

        // initialize cpu and driver managers
        const cpu_mgr = cpu_module.CPUManager.init(allocator);
        const driver_mgr = drivers_module.DriverManager.init(allocator);

        // initialize scheduler from scheduling module
        const scheduler = scheduler_module.Scheduler.init(allocator);
        const dispatcher: ?scheduler_module.Dispatcher = null;

        // initialize clock and profiler
        const clock = time_module.Clock.init(allocator);
        const profiler = time_module.Profiler.init(allocator);

        const dashboard = DashboardManager.init(allocator);
        const ui_connector = UIConnector.init(allocator);
        const app_manager = ApplicationManager.init(allocator);
        const session_manager = SessionManager.init(allocator);

        const custom_apps = std.array_list.Managed(CustomApplication).init(cache_alloc);

        return System{
            .allocator = cache_alloc,
            .backing_allocator = allocator,
            .tiered_cache = tiered_ptr,
            .user_manager = users_module.UserManager.init(cache_alloc),
            .workspace_manager = workspace_module.WorkspaceManager.init(cache_alloc),
            .project_tracking = ideas_module.ProjectTrackingManager.init(cache_alloc),
            .content_manager = content_module.ContentManager.init(cache_alloc),
            .design_manager = design_module.DesignManager.init(cache_alloc),
            .directory_book_manager = directory_book_module.DirectoryBookManager.init(cache_alloc),
            .database_manager = database_module.DatabaseManager.init(cache_alloc),
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
            .tasks = std.array_list.Managed(Task).init(cache_alloc),
            .engagements = std.array_list.Managed(Engagement).init(cache_alloc),
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
        // Clean up tasks created through System APIs.
        for (self.tasks.items) |*task| {
            self.allocator.free(task.title);
            self.allocator.free(task.description);
            for (task.required_skills.items) |skill| {
                self.allocator.free(skill);
            }
            task.required_skills.deinit();
        }
        self.tasks.deinit();

        // Clean up engagements.
        for (self.engagements.items) |*engagement| {
            engagement.completion_date = null;
        }
        self.engagements.deinit();

        // Free custom app registry storage.
        self.custom_apps.deinit();

        // Clean up system-owned components (always)
        self.user_manager.deinit();
        self.workspace_manager.deinit();
        self.project_tracking.deinit();
        self.content_manager.deinit();
        self.design_manager.deinit();
        self.directory_book_manager.deinit();
        self.database_manager.deinit();
        self.directory.deinit();
        self.vault.deinit();
        self.security_manager.deinit();
        self.logger.deinit();
        self.event_bus.deinit();
        self.state_manager.deinit();
        self.cluster.deinit();

        // System owns all subsystem instances initialized in `System.init`.
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
        var clock = self.clock;
        clock.deinit();
        var profiler = self.profiler;
        profiler.deinit();

        if (self.dashboard) |*d| d.deinit();
        if (self.ui_connector) |*u| u.deinit();
        if (self.app_manager) |*a| a.deinit();
        if (self.session_manager) |*s| s.deinit();
        self.tiered_cache.deinit();
        self.backing_allocator.destroy(self.tiered_cache);
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

    pub fn issueUserAuthToken(
        self: *System,
        user_id: u32,
        label: []const u8,
        kind: users_module.AuthTokenKind,
        ttl_seconds: i64,
    ) !users_module.IssuedToken {
        return try self.user_manager.issueAuthToken(user_id, label, kind, ttl_seconds);
    }

    pub fn freeUserIssuedToken(self: *System, token: users_module.IssuedToken) void {
        self.user_manager.freeIssuedToken(token);
    }

    pub fn authenticateUserToken(self: *System, token_value: []const u8) !u32 {
        return try self.user_manager.authenticateToken(token_value);
    }

    pub fn revokeUserAuthToken(self: *System, token_id: u32) !void {
        try self.user_manager.revokeAuthToken(token_id);
    }

    pub fn getUserAuthTokens(self: *System) []users_module.AuthToken {
        return self.user_manager.getAuthTokens();
    }

    pub fn addUserKey(
        self: *System,
        user_id: u32,
        label: []const u8,
        kind: users_module.KeyKind,
        key_material: []const u8,
        public_material: []const u8,
    ) !u32 {
        return try self.user_manager.addKey(user_id, label, kind, key_material, public_material);
    }

    pub fn verifyUserKey(self: *System, user_id: u32, key_material: []const u8) !bool {
        return try self.user_manager.verifyKey(user_id, key_material);
    }

    pub fn disableUserKey(self: *System, key_id: u32) !void {
        try self.user_manager.disableKey(key_id);
    }

    pub fn getUserKeys(self: *System) []users_module.UserKey {
        return self.user_manager.getKeys();
    }

    pub fn addUserCertificate(
        self: *System,
        user_id: u32,
        label: []const u8,
        subject: []const u8,
        issuer: []const u8,
        serial_number: []const u8,
        pem_data: []const u8,
        valid_from: i64,
        valid_to: i64,
    ) !u32 {
        return try self.user_manager.addCertificate(
            user_id,
            label,
            subject,
            issuer,
            serial_number,
            pem_data,
            valid_from,
            valid_to,
        );
    }

    pub fn revokeUserCertificate(self: *System, certificate_id: u32) !void {
        try self.user_manager.revokeCertificate(certificate_id);
    }

    pub fn validateUserCertificate(self: *System, certificate_id: u32) !bool {
        return try self.user_manager.validateCertificate(certificate_id);
    }

    pub fn getUserCertificates(self: *System) []users_module.UserCertificate {
        return self.user_manager.getCertificates();
    }

    pub fn grantUserPermission(self: *System, user_id: u32, permission: users_module.Permission) !void {
        try self.user_manager.grantPermission(user_id, permission);
    }

    pub fn revokeUserPermission(self: *System, user_id: u32, permission: users_module.Permission) !void {
        try self.user_manager.revokePermission(user_id, permission);
    }

    pub fn userHasPermission(self: *System, user_id: u32, permission: users_module.Permission) bool {
        return self.user_manager.hasPermission(user_id, permission);
    }

    pub fn grantUserPrivilege(self: *System, user_id: u32, privilege: users_module.Privilege) !void {
        try self.user_manager.grantPrivilege(user_id, privilege);
    }

    pub fn revokeUserPrivilege(self: *System, user_id: u32, privilege: users_module.Privilege) !void {
        try self.user_manager.revokePrivilege(user_id, privilege);
    }

    pub fn userHasPrivilege(self: *System, user_id: u32, privilege: users_module.Privilege) bool {
        return self.user_manager.hasPrivilege(user_id, privilege);
    }

    pub fn encryptForUser(self: *System, user_id: u32, plaintext: []const u8) !users_module.EncryptedPayload {
        return try self.user_manager.encryptForUser(user_id, plaintext);
    }

    pub fn decryptForUser(self: *System, user_id: u32, payload: users_module.EncryptedPayload) ![]u8 {
        return try self.user_manager.decryptForUser(user_id, payload);
    }

    pub fn freeUserEncryptedPayload(self: *System, payload: users_module.EncryptedPayload) void {
        self.user_manager.freeEncryptedPayload(payload);
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
            .required_skills = std.array_list.Managed([]const u8).init(self.allocator),
            .budget = budget,
            .deadline = deadline,
            .assigned_to = null,
            .completed = false,
        };

        try self.tasks.append(task);
        return task_id;
    }

    /// Add a skill requirement to a task
    pub fn addTaskSkillRequirement(self: *System, task_id: u32, skill: []const u8) !void {
        if (task_id < @as(u32, @intCast(self.tasks.items.len))) {
            const skill_copy = try self.allocator.dupe(u8, skill);
            try self.tasks.items[task_id].required_skills.append(skill_copy);
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

        try self.engagements.append(engagement);
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

    // ========== Project Tracking Management ==========

    pub fn createTrackedProject(
        self: *System,
        name: []const u8,
        description: []const u8,
        owner_id: ?u32,
        priority: ideas_module.ProjectPriority,
        start_at: ?i64,
        target_end_at: ?i64,
        created_at: i64,
    ) !u32 {
        return try self.project_tracking.createProject(
            name,
            description,
            owner_id,
            priority,
            start_at,
            target_end_at,
            created_at,
        );
    }

    pub fn updateTrackedProjectStatus(self: *System, project_id: u32, status: ideas_module.ProjectStatus, updated_at: i64) !void {
        try self.project_tracking.updateProjectStatus(project_id, status, updated_at);
    }

    pub fn getTrackedProjects(self: *System) []ideas_module.TrackedProject {
        return self.project_tracking.listProjects();
    }

    pub fn addTrackedProjectMilestone(
        self: *System,
        project_id: u32,
        name: []const u8,
        description: []const u8,
        due_at: ?i64,
        created_at: i64,
    ) !u32 {
        return try self.project_tracking.addMilestone(project_id, name, description, due_at, created_at);
    }

    pub fn addTrackedProjectTask(
        self: *System,
        project_id: u32,
        title: []const u8,
        description: []const u8,
        assignee_id: ?u32,
        due_at: ?i64,
        created_at: i64,
    ) !u32 {
        return try self.project_tracking.addTask(project_id, title, description, assignee_id, due_at, created_at);
    }

    pub fn addTrackedProjectStory(
        self: *System,
        project_id: u32,
        title: []const u8,
        description: []const u8,
        story_type: ideas_module.StoryType,
        priority: ideas_module.ProjectPriority,
        owner_id: ?u32,
        estimate_points: f32,
        created_at: i64,
    ) !u32 {
        return try self.project_tracking.addStory(
            project_id,
            title,
            description,
            story_type,
            priority,
            owner_id,
            estimate_points,
            created_at,
        );
    }

    pub fn updateTrackedProjectStoryStatus(
        self: *System,
        project_id: u32,
        story_id: u32,
        status: ideas_module.StoryStatus,
        updated_at: i64,
    ) !void {
        try self.project_tracking.setStoryStatus(project_id, story_id, status, updated_at);
    }

    pub fn updateTrackedProjectStoryType(
        self: *System,
        project_id: u32,
        story_id: u32,
        story_type: ideas_module.StoryType,
        updated_at: i64,
    ) !void {
        try self.project_tracking.updateStoryType(project_id, story_id, story_type, updated_at);
    }

    pub fn getTrackedProjectStories(self: *System, project_id: u32) ![]ideas_module.Story {
        return try self.project_tracking.getStories(project_id);
    }

    pub fn filterTrackedProjectStories(
        self: *System,
        project_id: u32,
        filter: ideas_module.StoryFilter,
        allocator: std.mem.Allocator,
    ) !std.array_list.Managed(ideas_module.StorySummary) {
        return try self.project_tracking.filterStories(project_id, filter, allocator);
    }

    pub fn deinitTrackedProjectStorySummaries(
        self: *System,
        summaries: *std.array_list.Managed(ideas_module.StorySummary),
        allocator: std.mem.Allocator,
    ) void {
        _ = self;
        ideas_module.ProjectTrackingManager.deinitStorySummaries(summaries, allocator);
    }

    pub fn addTrackedProjectNote(
        self: *System,
        project_id: u32,
        title: []const u8,
        body: []const u8,
        created_at: i64,
    ) !u32 {
        return try self.project_tracking.addNote(project_id, title, body, created_at);
    }

    pub fn getTrackedProjectProgress(self: *System, project_id: u32) !f32 {
        return try self.project_tracking.calculateProgress(project_id);
    }

    // ========== Content Management ==========

    pub fn createContent(
        self: *System,
        owner_user_id: ?u32,
        profile_id: ?u32,
        parent_id: ?u32,
        kind: content_module.ContentKind,
        title: []const u8,
        body: []const u8,
    ) !u32 {
        return try self.content_manager.createContent(owner_user_id, profile_id, parent_id, kind, title, body);
    }

    pub fn createFile(
        self: *System,
        owner_user_id: ?u32,
        profile_id: ?u32,
        parent_id: ?u32,
        title: []const u8,
        body: []const u8,
    ) !u32 {
        return try self.content_manager.createFile(owner_user_id, profile_id, parent_id, title, body);
    }

    pub fn createDirectory(
        self: *System,
        owner_user_id: ?u32,
        profile_id: ?u32,
        parent_id: ?u32,
        title: []const u8,
    ) !u32 {
        return try self.content_manager.createDirectory(owner_user_id, profile_id, parent_id, title);
    }

    pub fn createFolder(
        self: *System,
        owner_user_id: ?u32,
        profile_id: ?u32,
        parent_id: ?u32,
        title: []const u8,
    ) !u32 {
        return try self.content_manager.createFolder(owner_user_id, profile_id, parent_id, title);
    }

    pub fn createData(
        self: *System,
        owner_user_id: ?u32,
        profile_id: ?u32,
        parent_id: ?u32,
        title: []const u8,
        body: []const u8,
    ) !u32 {
        return try self.content_manager.createData(owner_user_id, profile_id, parent_id, title, body);
    }

    pub fn createInfo(
        self: *System,
        owner_user_id: ?u32,
        profile_id: ?u32,
        parent_id: ?u32,
        title: []const u8,
        body: []const u8,
    ) !u32 {
        return try self.content_manager.createInfo(owner_user_id, profile_id, parent_id, title, body);
    }

    pub fn createKnowledge(
        self: *System,
        owner_user_id: ?u32,
        profile_id: ?u32,
        parent_id: ?u32,
        title: []const u8,
        body: []const u8,
    ) !u32 {
        return try self.content_manager.createKnowledge(owner_user_id, profile_id, parent_id, title, body);
    }

    pub fn createWisdom(
        self: *System,
        owner_user_id: ?u32,
        profile_id: ?u32,
        parent_id: ?u32,
        title: []const u8,
        body: []const u8,
    ) !u32 {
        return try self.content_manager.createWisdom(owner_user_id, profile_id, parent_id, title, body);
    }

    pub fn createArtifact(
        self: *System,
        owner_user_id: ?u32,
        profile_id: ?u32,
        parent_id: ?u32,
        title: []const u8,
        body: []const u8,
    ) !u32 {
        return try self.content_manager.createArtifact(owner_user_id, profile_id, parent_id, title, body);
    }

    pub fn createAsset(
        self: *System,
        owner_user_id: ?u32,
        profile_id: ?u32,
        parent_id: ?u32,
        title: []const u8,
        body: []const u8,
    ) !u32 {
        return try self.content_manager.createAsset(owner_user_id, profile_id, parent_id, title, body);
    }

    pub fn updateContentBody(self: *System, content_id: u32, body: []const u8) !void {
        try self.content_manager.updateBody(content_id, body);
    }

    pub fn renameContent(self: *System, content_id: u32, title: []const u8) !void {
        try self.content_manager.rename(content_id, title);
    }

    pub fn moveContent(self: *System, content_id: u32, new_parent_id: ?u32) !void {
        try self.content_manager.moveContent(content_id, new_parent_id);
    }

    pub fn addContentTag(self: *System, content_id: u32, tag: []const u8) !void {
        try self.content_manager.addTag(content_id, tag);
    }

    pub fn setContentMetadata(self: *System, content_id: u32, key: []const u8, value: []const u8) !void {
        try self.content_manager.setMetadata(content_id, key, value);
    }

    pub fn deactivateContent(self: *System, content_id: u32) !void {
        try self.content_manager.deactivateContent(content_id);
    }

    pub fn getContentById(self: *System, content_id: u32) !content_module.ContentItem {
        return try self.content_manager.getContentById(content_id);
    }

    pub fn getContentItems(self: *System) []content_module.ContentItem {
        return self.content_manager.getItems();
    }

    pub fn countContentByKind(self: *System, kind: content_module.ContentKind) u32 {
        return self.content_manager.countByKind(kind);
    }

    pub fn createNote(
        self: *System,
        owner_user_id: ?u32,
        profile_id: ?u32,
        linked_content_id: ?u32,
        title: []const u8,
        body: []const u8,
        importance: content_module.NoteImportance,
    ) !u32 {
        return try self.content_manager.createNote(owner_user_id, profile_id, linked_content_id, title, body, importance);
    }

    pub fn updateNote(
        self: *System,
        note_id: u32,
        title: []const u8,
        body: []const u8,
        importance: content_module.NoteImportance,
    ) !void {
        try self.content_manager.updateNote(note_id, title, body, importance);
    }

    pub fn addNoteTag(self: *System, note_id: u32, tag: []const u8) !void {
        try self.content_manager.addNoteTag(note_id, tag);
    }

    pub fn pinNote(self: *System, note_id: u32) !void {
        try self.content_manager.pinNote(note_id);
    }

    pub fn unpinNote(self: *System, note_id: u32) !void {
        try self.content_manager.unpinNote(note_id);
    }

    pub fn archiveNote(self: *System, note_id: u32) !void {
        try self.content_manager.archiveNote(note_id);
    }

    pub fn unarchiveNote(self: *System, note_id: u32) !void {
        try self.content_manager.unarchiveNote(note_id);
    }

    pub fn setNoteLinkedContent(self: *System, note_id: u32, linked_content_id: ?u32) !void {
        try self.content_manager.setNoteLinkedContent(note_id, linked_content_id);
    }

    pub fn getNoteById(self: *System, note_id: u32) !content_module.Note {
        return try self.content_manager.getNoteById(note_id);
    }

    pub fn getNotes(self: *System) []content_module.Note {
        return self.content_manager.getNotes();
    }

    // ========== Directory Book Management ==========

    pub fn createDirectoryBookEntry(
        self: *System,
        owner_user_id: u32,
        profile_id: ?u32,
        kind: directory_book_module.DirectoryEntityKind,
        display_name: []const u8,
        headline: []const u8,
        notes: []const u8,
    ) !u32 {
        if (owner_user_id >= @as(u32, @intCast(self.user_manager.getUsers().len))) return users_module.UserError.UserNotFound;
        if (profile_id) |pid| {
            const profile = try self.user_manager.getProfileById(pid);
            if (profile.owner_user_id != owner_user_id) return users_module.UserError.ProfileOwnershipMismatch;
        }
        return try self.directory_book_manager.createEntry(owner_user_id, profile_id, kind, display_name, headline, notes);
    }

    pub fn updateDirectoryBookEntry(self: *System, entry_id: u32, display_name: []const u8, headline: []const u8, notes: []const u8) !void {
        try self.directory_book_manager.updateEntry(entry_id, display_name, headline, notes);
    }

    pub fn setDirectoryBookEntryStatus(self: *System, entry_id: u32, status: directory_book_module.RelationshipStatus) !void {
        try self.directory_book_manager.setEntryStatus(entry_id, status);
    }

    pub fn addDirectoryBookTag(self: *System, entry_id: u32, tag: []const u8) !void {
        try self.directory_book_manager.addTag(entry_id, tag);
    }

    pub fn linkDirectoryBookPersona(self: *System, entry_id: u32, owner_user_id: u32, persona_id: ?u32) !void {
        if (persona_id) |pid| {
            var owned = false;
            for (self.user_manager.getPersonas()) |persona| {
                if (persona.id == pid and persona.user_id == owner_user_id) {
                    owned = true;
                    break;
                }
            }
            if (!owned) return users_module.UserError.PersonaOwnershipMismatch;
        }
        try self.directory_book_manager.associatePersona(entry_id, persona_id);
    }

    pub fn linkDirectoryBookManagedContact(self: *System, entry_id: u32, managed_contact_id: ?u32) !void {
        if (managed_contact_id) |cid| {
            _ = try self.user_manager.getContactById(cid);
        }
        try self.directory_book_manager.linkManagedContact(entry_id, managed_contact_id);
    }

    pub fn linkDirectoryBookNote(self: *System, entry_id: u32, note_id: u32) !void {
        _ = try self.content_manager.getNoteById(note_id);
        try self.directory_book_manager.linkNote(entry_id, note_id);
    }

    pub fn unlinkDirectoryBookNote(self: *System, entry_id: u32, note_id: u32) !void {
        try self.directory_book_manager.unlinkNote(entry_id, note_id);
    }

    pub fn addDirectoryBookRelationship(
        self: *System,
        entry_id: u32,
        target_entry_id: u32,
        relation_kind: directory_book_module.DirectoryEntityKind,
        status: directory_book_module.RelationshipStatus,
        strength: directory_book_module.RelationshipStrength,
    ) !void {
        try self.directory_book_manager.addRelationship(entry_id, target_entry_id, relation_kind, status, strength);
    }

    pub fn createDirectoryBookOrganization(
        self: *System,
        owner_user_id: u32,
        name: []const u8,
        industry: []const u8,
        notes: []const u8,
    ) !u32 {
        if (owner_user_id >= @as(u32, @intCast(self.user_manager.getUsers().len))) return users_module.UserError.UserNotFound;
        return try self.directory_book_manager.createOrganization(owner_user_id, name, industry, notes);
    }

    pub fn assignDirectoryBookEntryToOrganization(self: *System, entry_id: u32, organization_id: ?u32) !void {
        try self.directory_book_manager.assignEntryToOrganization(entry_id, organization_id);
    }

    pub fn deactivateDirectoryBookEntry(self: *System, entry_id: u32) !void {
        try self.directory_book_manager.deactivateEntry(entry_id);
    }

    pub fn getDirectoryBookEntryById(self: *System, entry_id: u32) !directory_book_module.DirectoryEntry {
        return try self.directory_book_manager.getEntryById(entry_id);
    }

    pub fn getDirectoryBookEntries(self: *System) []directory_book_module.DirectoryEntry {
        return self.directory_book_manager.getEntries();
    }

    pub fn getDirectoryBookOrganizationById(self: *System, organization_id: u32) !directory_book_module.DirectoryOrganization {
        return try self.directory_book_manager.getOrganizationById(organization_id);
    }

    pub fn getDirectoryBookOrganizations(self: *System) []directory_book_module.DirectoryOrganization {
        return self.directory_book_manager.getOrganizations();
    }

    // ========== Database Management ==========

    pub fn createDatabase(self: *System, name: []const u8, description: []const u8) !u32 {
        return try self.database_manager.createDatabase(name, description);
    }

    pub fn updateDatabase(self: *System, database_id: u32, name: []const u8, description: []const u8) !void {
        try self.database_manager.updateDatabase(database_id, name, description);
    }

    pub fn createTable(self: *System, database_id: u32, name: []const u8, description: []const u8) !u32 {
        return try self.database_manager.createTable(database_id, name, description);
    }

    pub fn addTableColumn(
        self: *System,
        table_id: u32,
        name: []const u8,
        column_type: database_module.ColumnType,
        nullable: bool,
        default_value: []const u8,
    ) !void {
        try self.database_manager.addTableColumn(table_id, name, column_type, nullable, default_value);
    }

    pub fn createRecord(self: *System, database_id: u32, table_id: u32) !u32 {
        return try self.database_manager.createRecord(database_id, table_id);
    }

    pub fn setRecordField(self: *System, record_id: u32, key: []const u8, value: []const u8) !void {
        try self.database_manager.setRecordField(record_id, key, value);
    }

    pub fn deleteRecord(self: *System, record_id: u32) !void {
        try self.database_manager.deleteRecord(record_id);
    }

    pub fn getDatabaseById(self: *System, database_id: u32) !database_module.Database {
        return try self.database_manager.getDatabaseById(database_id);
    }

    pub fn getTableById(self: *System, table_id: u32) !database_module.Table {
        return try self.database_manager.getTableById(table_id);
    }

    pub fn getRecordById(self: *System, record_id: u32) !database_module.Record {
        return try self.database_manager.getRecordById(record_id);
    }

    pub fn getDatabases(self: *System) []database_module.Database {
        return self.database_manager.getDatabases();
    }

    pub fn getTables(self: *System) []database_module.Table {
        return self.database_manager.getTables();
    }

    pub fn getRecords(self: *System) []database_module.Record {
        return self.database_manager.getRecords();
    }

    pub fn queryRecordsByField(
        self: *System,
        table_id: u32,
        key: []const u8,
        value: []const u8,
        allocator: std.mem.Allocator,
    ) !std.array_list.Managed(database_module.Record) {
        return try self.database_manager.queryRecordsByField(table_id, key, value, allocator);
    }

    pub fn getDatabaseTransactions(self: *System) []database_module.Transaction {
        return self.database_manager.getTransactions();
    }

    // ========== Design Management ==========

    pub fn createDesign(
        self: *System,
        owner_user_id: ?u32,
        profile_id: ?u32,
        linked_content_id: ?u32,
        kind: design_module.DesignKind,
        title: []const u8,
        description: []const u8,
        initial_revision_summary: []const u8,
        initial_revision_content: []const u8,
        created_by_user_id: ?u32,
    ) !u32 {
        return try self.design_manager.createDesign(
            owner_user_id,
            profile_id,
            linked_content_id,
            kind,
            title,
            description,
            initial_revision_summary,
            initial_revision_content,
            created_by_user_id,
        );
    }

    pub fn addDesignRevision(
        self: *System,
        design_id: u32,
        summary: []const u8,
        content: []const u8,
        created_by_user_id: ?u32,
    ) !u32 {
        return try self.design_manager.addRevision(design_id, summary, content, created_by_user_id);
    }

    pub fn setDesignStatus(self: *System, design_id: u32, status: design_module.DesignStatus) !void {
        try self.design_manager.setStatus(design_id, status);
    }

    pub fn renameDesign(self: *System, design_id: u32, title: []const u8) !void {
        try self.design_manager.rename(design_id, title);
    }

    pub fn updateDesignDescription(self: *System, design_id: u32, description: []const u8) !void {
        try self.design_manager.updateDescription(design_id, description);
    }

    pub fn setDesignLinkedContent(self: *System, design_id: u32, linked_content_id: ?u32) !void {
        try self.design_manager.setLinkedContent(design_id, linked_content_id);
    }

    pub fn addDesignTag(self: *System, design_id: u32, tag: []const u8) !void {
        try self.design_manager.addTag(design_id, tag);
    }

    pub fn setDesignMetadata(self: *System, design_id: u32, key: []const u8, value: []const u8) !void {
        try self.design_manager.setMetadata(design_id, key, value);
    }

    pub fn deactivateDesign(self: *System, design_id: u32) !void {
        try self.design_manager.deactivateDesign(design_id);
    }

    pub fn getDesignById(self: *System, design_id: u32) !design_module.DesignItem {
        return try self.design_manager.getDesignById(design_id);
    }

    pub fn getDesigns(self: *System) []design_module.DesignItem {
        return self.design_manager.getDesigns();
    }

    pub fn getDesignRevisionById(self: *System, revision_id: u32) !design_module.DesignRevision {
        return try self.design_manager.getRevisionById(revision_id);
    }

    pub fn getDesignLatestRevision(self: *System, design_id: u32) !design_module.DesignRevision {
        return try self.design_manager.getLatestRevision(design_id);
    }

    pub fn countDesignByKind(self: *System, kind: design_module.DesignKind) u32 {
        return self.design_manager.countByKind(kind);
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
