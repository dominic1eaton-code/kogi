//! KOGI Security Module - Authentication, Authorization, and Access Control
//! Provides comprehensive security infrastructure for the KOGI OS including
//! identity authentication, permission management, and role-based access control.

const std = @import("std");

// ========== Authentication & Credentials ==========

/// Hash algorithm for password storage
pub const HashAlgorithm = enum {
    argon2,
    scrypt,
    pbkdf2,
};

/// Credential information for an identity
pub const Credential = struct {
    identity_id: u32,
    password_hash: []const u8,
    hash_algorithm: HashAlgorithm,
    salt: []const u8,
    created_at: i64,
    last_changed_at: i64,
    failed_login_attempts: u32,
    locked: bool,
};

/// Authentication session
pub const Session = struct {
    id: u32,
    identity_id: u32,
    token: []const u8,
    created_at: i64,
    expires_at: i64,
    last_activity: i64,
    ip_address: []const u8,
    device_fingerprint: ?[]const u8 = null,
    active: bool,
};

// ========== Authorization & Roles ==========

/// System roles with predefined permissions
pub const Role = enum {
    admin, // Full system access
    moderator, // Content and user management
    worker, // Standard user access
    contractor, // Limited to own work
    guest, // Read-only access
    custom, // Custom permission set
};

/// Permissions in the KOGI OS
pub const Permission = enum {
    // Identity management
    create_identity,
    read_identity,
    update_identity,
    delete_identity,
    manage_roles,
    manage_credentials,

    // Workspace management
    create_workspace,
    read_workspace,
    update_workspace,
    delete_workspace,
    share_workspace,

    // Task management
    create_task,
    read_task,
    update_task,
    delete_task,
    complete_task,

    // Engagement management
    create_engagement,
    read_engagement,
    update_engagement,
    delete_engagement,
    complete_engagement,

    // Directory management
    create_organization,
    read_organization,
    update_organization,
    delete_organization,
    manage_contacts,

    // Vault management
    create_vault_item,
    read_vault_item,
    update_vault_item,
    delete_vault_item,

    // Connection registry
    manage_connections,
    view_connections,

    // System administration
    view_audit_log,
    manage_security_settings,
    manage_users,
    system_configuration,

    // Other
    export_data,
    import_data,
};

/// Role-based permission mapping
pub const RolePermissions = struct {
    role: Role,
    permissions: std.ArrayList(Permission),
};

/// Assignment of a role to an identity
pub const RoleAssignment = struct {
    identity_id: u32,
    role: Role,
    assigned_at: i64,
    assigned_by: u32,
    expires_at: ?i64 = null,
    custom_permissions: ?std.ArrayList(Permission) = null,
};

// ========== Access Control ==========

/// Resource access level
pub const AccessLevel = enum {
    private, // Only owner
    protected, // Owner + specific identities
    internal, // All authenticated users
    public, // Anyone
};

/// Access control entry
pub const AccessControlEntry = struct {
    resource_id: u32,
    resource_type: []const u8,
    owner_id: u32,
    access_level: AccessLevel,
    granted_to: std.ArrayList(u32), // Identity IDs with access
    created_at: i64,
    updated_at: i64,
};

// ========== Audit & Logging ==========

/// Security event types
pub const SecurityEventType = enum {
    login_success,
    login_failure,
    logout,
    password_changed,
    password_reset,
    role_assigned,
    role_revoked,
    permission_denied,
    credential_locked,
    suspicious_activity,
    data_accessed,
    data_modified,
    data_deleted,
    session_expired,
    multi_factor_auth_enabled,
    multi_factor_auth_disabled,
    api_key_created,
    api_key_revoked,
    permission_grant,
    permission_revoke,
};

/// Audit log entry
pub const AuditLogEntry = struct {
    id: u32,
    event_type: SecurityEventType,
    identity_id: u32,
    resource_id: ?u32 = null,
    resource_type: ?[]const u8 = null,
    action: []const u8,
    details: []const u8,
    ip_address: []const u8,
    timestamp: i64,
    success: bool,
};

// ========== Multi-Factor Authentication ==========

/// MFA method types
pub const MFAMethod = enum {
    totp, // Time-based one-time password
    email, // Email verification
    sms, // SMS code
    hardware_token,
};

/// MFA configuration for an identity
pub const MFAConfig = struct {
    identity_id: u32,
    method: MFAMethod,
    enabled: bool,
    secret: ?[]const u8 = null, // For TOTP
    backup_codes: std.ArrayList([]const u8),
    created_at: i64,
};

// ========== Security Manager ==========

/// Security manager for the KOGI OS
pub const SecurityManager = struct {
    allocator: std.mem.Allocator,
    credentials: std.ArrayList(Credential),
    sessions: std.ArrayList(Session),
    role_assignments: std.ArrayList(RoleAssignment),
    role_permissions: std.ArrayList(RolePermissions),
    access_controls: std.ArrayList(AccessControlEntry),
    audit_log: std.ArrayList(AuditLogEntry),
    mfa_configs: std.ArrayList(MFAConfig),
    next_session_id: u32 = 0,
    next_audit_id: u32 = 0,
    session_timeout_seconds: i64 = 3600, // 1 hour default

    pub fn init(allocator: std.mem.Allocator) SecurityManager {
        return SecurityManager{
            .allocator = allocator,
            .credentials = std.ArrayList(Credential){},
            .sessions = std.ArrayList(Session){},
            .role_assignments = std.ArrayList(RoleAssignment){},
            .role_permissions = std.ArrayList(RolePermissions){},
            .access_controls = std.ArrayList(AccessControlEntry){},
            .audit_log = std.ArrayList(AuditLogEntry){},
            .mfa_configs = std.ArrayList(MFAConfig){},
        };
    }

    pub fn deinit(self: *SecurityManager) void {
        for (self.credentials.items) |cred| {
            self.allocator.free(cred.password_hash);
            self.allocator.free(cred.salt);
        }
        self.credentials.deinit(self.allocator);

        for (self.sessions.items) |session| {
            self.allocator.free(session.token);
            self.allocator.free(session.ip_address);
            if (session.device_fingerprint) |fp| {
                self.allocator.free(fp);
            }
        }
        self.sessions.deinit(self.allocator);

        for (self.role_assignments.items) |*ra| {
            if (ra.custom_permissions) |*perms| {
                perms.deinit(self.allocator);
            }
        }
        self.role_assignments.deinit(self.allocator);

        for (self.role_permissions.items) |rp| {
            var perms = rp.permissions;
            perms.deinit(self.allocator);
        }
        self.role_permissions.deinit(self.allocator);

        for (self.access_controls.items) |ac| {
            self.allocator.free(ac.resource_type);
            var granted = ac.granted_to;
            granted.deinit(self.allocator);
        }
        self.access_controls.deinit(self.allocator);

        for (self.audit_log.items) |entry| {
            self.allocator.free(entry.action);
            self.allocator.free(entry.details);
            self.allocator.free(entry.ip_address);
            if (entry.resource_type) |rt| {
                self.allocator.free(rt);
            }
        }
        self.audit_log.deinit(self.allocator);

        for (self.mfa_configs.items) |*mfa| {
            if (mfa.secret) |secret| {
                self.allocator.free(secret);
            }
            for (mfa.backup_codes.items) |code| {
                self.allocator.free(code);
            }
            mfa.backup_codes.deinit(self.allocator);
        }
        self.mfa_configs.deinit(self.allocator);
    }

    // ========== Credential Management ==========

    /// Register a new credential for an identity
    pub fn setCredential(
        self: *SecurityManager,
        identity_id: u32,
        password_hash: []const u8,
        salt: []const u8,
        algorithm: HashAlgorithm,
    ) !void {
        const credential = Credential{
            .identity_id = identity_id,
            .password_hash = try self.allocator.dupe(u8, password_hash),
            .hash_algorithm = algorithm,
            .salt = try self.allocator.dupe(u8, salt),
            .created_at = std.time.timestamp(),
            .last_changed_at = std.time.timestamp(),
            .failed_login_attempts = 0,
            .locked = false,
        };
        try self.credentials.append(self.allocator, credential);
    }

    /// Verify a password against stored hash
    pub fn verifyPassword(self: *SecurityManager, identity_id: u32, password_hash: []const u8) !bool {
        for (self.credentials.items) |cred| {
            if (cred.identity_id == identity_id) {
                if (cred.locked) {
                    try self.logAuditEvent(.login_failure, identity_id, null, null, "Account locked", "", false);
                    return false;
                }
                const matches = std.mem.eql(u8, cred.password_hash, password_hash);
                if (matches) {
                    var cred_mut = cred;
                    cred_mut.failed_login_attempts = 0;
                    try self.logAuditEvent(.login_success, identity_id, null, null, "Password verified", "", true);
                } else {
                    var cred_mut = cred;
                    cred_mut.failed_login_attempts += 1;
                    if (cred_mut.failed_login_attempts >= 5) {
                        cred_mut.locked = true;
                        try self.logAuditEvent(.credential_locked, identity_id, null, null, "Account locked after failed attempts", "", false);
                    }
                    try self.logAuditEvent(.login_failure, identity_id, null, null, "Invalid password", "", false);
                }
                return matches;
            }
        }
        return false;
    }

    // ========== Session Management ==========

    /// Create a new authenticated session
    pub fn createSession(
        self: *SecurityManager,
        identity_id: u32,
        token: []const u8,
        ip_address: []const u8,
        device_fingerprint: ?[]const u8,
    ) !u32 {
        const session_id = self.next_session_id;
        self.next_session_id += 1;
        const now = std.time.timestamp();

        const session = Session{
            .id = session_id,
            .identity_id = identity_id,
            .token = try self.allocator.dupe(u8, token),
            .created_at = now,
            .expires_at = now + self.session_timeout_seconds,
            .last_activity = now,
            .ip_address = try self.allocator.dupe(u8, ip_address),
            .device_fingerprint = if (device_fingerprint) |fp| try self.allocator.dupe(u8, fp) else null,
            .active = true,
        };

        try self.sessions.append(self.allocator, session);
        try self.logAuditEvent(.login_success, identity_id, null, null, "Session created", ip_address, true);
        return session_id;
    }

    /// Validate an active session
    pub fn validateSession(self: *SecurityManager, session_id: u32, token: []const u8) bool {
        if (session_id >= @as(u32, @intCast(self.sessions.items.len))) {
            return false;
        }

        const session = self.sessions.items[session_id];
        if (!session.active) return false;
        if (!std.mem.eql(u8, session.token, token)) return false;

        const now = std.time.timestamp();
        if (now > session.expires_at) {
            return false;
        }

        return true;
    }

    /// Invalidate a session (logout)
    pub fn invalidateSession(self: *SecurityManager, session_id: u32) !void {
        if (session_id < @as(u32, @intCast(self.sessions.items.len))) {
            const identity_id = self.sessions.items[session_id].identity_id;
            self.sessions.items[session_id].active = false;
            try self.logAuditEvent(.logout, identity_id, null, null, "Session terminated", "", true);
        }
    }

    // ========== Role Management ==========

    /// Assign a role to an identity
    pub fn assignRole(
        self: *SecurityManager,
        identity_id: u32,
        role: Role,
        assigned_by: u32,
        expires_at: ?i64,
    ) !void {
        const assignment = RoleAssignment{
            .identity_id = identity_id,
            .role = role,
            .assigned_at = std.time.timestamp(),
            .assigned_by = assigned_by,
            .expires_at = expires_at,
        };
        try self.role_assignments.append(self.allocator, assignment);
        try self.logAuditEvent(.role_assigned, assigned_by, identity_id, null, "Role assigned", "", true);
    }

    /// Get primary role for an identity
    pub fn getIdentityRole(self: *SecurityManager, identity_id: u32) ?Role {
        for (self.role_assignments.items) |assignment| {
            if (assignment.identity_id == identity_id and assignment.expires_at == null) {
                const now = std.time.timestamp();
                if (assignment.expires_at != null and now > assignment.expires_at.?) {
                    continue;
                }
                return assignment.role;
            }
        }
        return null;
    }

    /// Get all permissions for an identity
    pub fn getIdentityPermissions(self: *SecurityManager, identity_id: u32, allocator: std.mem.Allocator) !std.ArrayList(Permission) {
        var permissions = std.ArrayList(Permission){};

        // Get role-based permissions
        if (self.getIdentityRole(identity_id)) |role| {
            for (self.role_permissions.items) |rp| {
                if (rp.role == role) {
                    for (rp.permissions.items) |perm| {
                        try permissions.append(allocator, perm);
                    }
                    break;
                }
            }
        }

        // Get custom permissions
        for (self.role_assignments.items) |assignment| {
            if (assignment.identity_id == identity_id and assignment.custom_permissions != null) {
                for (assignment.custom_permissions.?.items) |perm| {
                    try permissions.append(allocator, perm);
                }
            }
        }

        return permissions;
    }

    /// Check if identity has a permission
    pub fn hasPermission(self: *SecurityManager, identity_id: u32, permission: Permission) bool {
        var perms = self.getIdentityPermissions(identity_id, self.allocator) catch {
            return false;
        };
        defer perms.deinit(self.allocator);

        for (perms.items) |perm| {
            if (perm == permission) return true;
        }
        return false;
    }

    // ========== Access Control ==========

    /// Grant access to a resource
    pub fn grantAccess(
        self: *SecurityManager,
        resource_id: u32,
        resource_type: []const u8,
        owner_id: u32,
        access_level: AccessLevel,
        granted_to: []const u32,
    ) !void {
        const ace = AccessControlEntry{
            .resource_id = resource_id,
            .resource_type = try self.allocator.dupe(u8, resource_type),
            .owner_id = owner_id,
            .access_level = access_level,
            .granted_to = std.ArrayList(u32){},
            .created_at = std.time.timestamp(),
            .updated_at = std.time.timestamp(),
        };
        var ace_mut = ace;
        for (granted_to) |identity_id| {
            try ace_mut.granted_to.append(self.allocator, identity_id);
        }
        try self.access_controls.append(self.allocator, ace_mut);
    }

    /// Check if identity has access to resource
    pub fn canAccess(self: *SecurityManager, identity_id: u32, resource_id: u32, resource_type: []const u8) bool {
        for (self.access_controls.items) |ace| {
            if (ace.resource_id == resource_id and std.mem.eql(u8, ace.resource_type, resource_type)) {
                // Owner always has access
                if (ace.owner_id == identity_id) return true;

                // Check access level
                switch (ace.access_level) {
                    .public => return true,
                    .internal => {
                        // Check if identity is authenticated
                        return self.getIdentityRole(identity_id) != null;
                    },
                    .protected => {
                        // Check if identity is in granted list
                        for (ace.granted_to.items) |id| {
                            if (id == identity_id) return true;
                        }
                    },
                    .private => return false,
                }
            }
        }
        return false;
    }

    // ========== Audit Logging ==========

    /// Log a security event
    pub fn logAuditEvent(
        self: *SecurityManager,
        event_type: SecurityEventType,
        identity_id: u32,
        resource_id: ?u32,
        resource_type: ?[]const u8,
        action: []const u8,
        ip_address: []const u8,
        success: bool,
    ) !void {
        const entry = AuditLogEntry{
            .id = self.next_audit_id,
            .event_type = event_type,
            .identity_id = identity_id,
            .resource_id = resource_id,
            .resource_type = if (resource_type) |rt| try self.allocator.dupe(u8, rt) else null,
            .action = try self.allocator.dupe(u8, action),
            .details = try self.allocator.dupe(u8, ""),
            .ip_address = try self.allocator.dupe(u8, ip_address),
            .timestamp = std.time.timestamp(),
            .success = success,
        };
        try self.audit_log.append(self.allocator, entry);
        self.next_audit_id += 1;
    }

    /// Get audit log entries
    pub fn getAuditLog(self: *SecurityManager) []AuditLogEntry {
        return self.audit_log.items;
    }

    /// Get audit log entries for identity
    pub fn getIdentityAuditLog(self: *SecurityManager, identity_id: u32, allocator: std.mem.Allocator) !std.ArrayList(AuditLogEntry) {
        var results = std.ArrayList(AuditLogEntry){};
        for (self.audit_log.items) |entry| {
            if (entry.identity_id == identity_id) {
                try results.append(allocator, entry);
            }
        }
        return results;
    }

    // ========== MFA Management ==========

    /// Enable MFA for an identity
    pub fn enableMFA(self: *SecurityManager, identity_id: u32, method: MFAMethod, secret: ?[]const u8) !void {
        const mfa = MFAConfig{
            .identity_id = identity_id,
            .method = method,
            .enabled = true,
            .secret = if (secret) |s| try self.allocator.dupe(u8, s) else null,
            .backup_codes = std.ArrayList([]const u8){},
            .created_at = std.time.timestamp(),
        };
        try self.mfa_configs.append(self.allocator, mfa);
        try self.logAuditEvent(.multi_factor_auth_enabled, identity_id, null, null, "MFA enabled", "", true);
    }

    /// Disable MFA for an identity
    pub fn disableMFA(self: *SecurityManager, identity_id: u32) !void {
        for (self.mfa_configs.items) |*mfa| {
            if (mfa.identity_id == identity_id) {
                mfa.enabled = false;
                try self.logAuditEvent(.multi_factor_auth_disabled, identity_id, null, null, "MFA disabled", "", true);
                return;
            }
        }
    }

    /// Verify MFA for identity
    pub fn isMFAEnabled(self: *SecurityManager, identity_id: u32) bool {
        for (self.mfa_configs.items) |mfa| {
            if (mfa.identity_id == identity_id and mfa.enabled) {
                return true;
            }
        }
        return false;
    }
};

// ========== Default Role Permissions ==========

pub fn initializeDefaultRoles(allocator: std.mem.Allocator) !std.ArrayList(RolePermissions) {
    var roles = std.ArrayList(RolePermissions){};

    // Admin role - full access
    var admin_perms = std.ArrayList(Permission){};
    try admin_perms.append(allocator, .create_identity);
    try admin_perms.append(allocator, .read_identity);
    try admin_perms.append(allocator, .update_identity);
    try admin_perms.append(allocator, .delete_identity);
    try admin_perms.append(allocator, .manage_roles);
    try admin_perms.append(allocator, .manage_credentials);
    try admin_perms.append(allocator, .create_workspace);
    try admin_perms.append(allocator, .read_workspace);
    try admin_perms.append(allocator, .update_workspace);
    try admin_perms.append(allocator, .delete_workspace);
    try admin_perms.append(allocator, .manage_users);
    try admin_perms.append(allocator, .view_audit_log);
    try admin_perms.append(allocator, .system_configuration);
    try roles.append(allocator, RolePermissions{ .role = .admin, .permissions = admin_perms });

    // Worker role - standard access
    var worker_perms = std.ArrayList(Permission){};
    try worker_perms.append(allocator, .read_identity);
    try worker_perms.append(allocator, .update_identity);
    try worker_perms.append(allocator, .manage_credentials);
    try worker_perms.append(allocator, .create_workspace);
    try worker_perms.append(allocator, .read_workspace);
    try worker_perms.append(allocator, .update_workspace);
    try worker_perms.append(allocator, .create_task);
    try worker_perms.append(allocator, .read_task);
    try worker_perms.append(allocator, .update_task);
    try worker_perms.append(allocator, .create_engagement);
    try worker_perms.append(allocator, .read_engagement);
    try worker_perms.append(allocator, .create_organization);
    try worker_perms.append(allocator, .read_organization);
    try worker_perms.append(allocator, .manage_connections);
    try worker_perms.append(allocator, .export_data);
    try roles.append(allocator, RolePermissions{ .role = .worker, .permissions = worker_perms });

    // Contractor role - limited access
    var contractor_perms = std.ArrayList(Permission){};
    try contractor_perms.append(allocator, .read_identity);
    try contractor_perms.append(allocator, .update_identity);
    try contractor_perms.append(allocator, .read_workspace);
    try contractor_perms.append(allocator, .read_task);
    try contractor_perms.append(allocator, .read_engagement);
    try contractor_perms.append(allocator, .read_organization);
    try contractor_perms.append(allocator, .view_connections);
    try roles.append(allocator, RolePermissions{ .role = .contractor, .permissions = contractor_perms });

    // Guest role - read-only
    var guest_perms = std.ArrayList(Permission){};
    try guest_perms.append(allocator, .read_identity);
    try guest_perms.append(allocator, .read_workspace);
    try guest_perms.append(allocator, .read_task);
    try guest_perms.append(allocator, .read_organization);
    try roles.append(allocator, RolePermissions{ .role = .guest, .permissions = guest_perms });

    return roles;
}
