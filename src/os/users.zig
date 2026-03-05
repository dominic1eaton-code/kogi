const std = @import("std");
const identity_module = @import("identity.zig");
const registry_module = @import("registry.zig");
const providers_module = @import("providers.zig");
const accounts_module = @import("accounts.zig");
const profile_module = @import("profile.zig");
const contacts_module = @import("contacts.zig");

pub const AccountType = accounts_module.AccountType;
pub const ProfileType = profile_module.ProfileType;
pub const ProfileEntry = profile_module.ProfileProperty;
pub const UserProfile = profile_module.ManagedProfile;
pub const ProfileAccount = accounts_module.ManagedAccount;
pub const UserContact = contacts_module.ManagedContact;
pub const ContactKind = contacts_module.ContactKind;
pub const ContactChannelKind = contacts_module.ContactChannelKind;

pub const UserRole = enum {
    admin,
    worker,
    viewer,
};

pub const CredentialKind = enum {
    password,
    api_token,
    ssh_key,
};

pub const AuthTokenKind = enum {
    session,
    bearer,
    refresh,
};

pub const KeyKind = enum {
    api,
    ssh,
    signing,
    encryption,
};

pub const Permission = enum {
    manage_users,
    manage_profiles,
    manage_accounts,
    manage_contacts,
    manage_providers,
    manage_security,
    read_data,
    write_data,
    delete_data,
};

pub const Privilege = enum {
    superuser,
    support_audit,
    automation,
};

pub const User = struct {
    id: u32,
    username: []const u8,
    email: []const u8,
    role: UserRole,
    linked_identity_id: ?u32,
    active: bool,
    created_at: i64,
    last_login_at: ?i64,
};

pub const Credential = struct {
    id: u32,
    user_id: u32,
    label: []const u8,
    kind: CredentialKind,
    secret_hash: []const u8,
    active: bool,
    created_at: i64,
    last_used_at: ?i64,
};

pub const Persona = struct {
    id: u32,
    user_id: u32,
    name: []const u8,
    description: []const u8,
    focus_area: []const u8,
    tone: []const u8,
    active: bool,
    created_at: i64,
};

pub const AuthToken = struct {
    id: u32,
    user_id: u32,
    label: []const u8,
    kind: AuthTokenKind,
    token_hash: []const u8,
    active: bool,
    created_at: i64,
    expires_at: i64,
    last_used_at: ?i64,
};

pub const IssuedToken = struct {
    id: u32,
    value: []u8,
    expires_at: i64,
};

pub const UserKey = struct {
    id: u32,
    user_id: u32,
    label: []const u8,
    kind: KeyKind,
    key_hash: []const u8,
    public_material: []const u8,
    active: bool,
    created_at: i64,
    last_used_at: ?i64,
};

pub const UserCertificate = struct {
    id: u32,
    user_id: u32,
    label: []const u8,
    subject: []const u8,
    issuer: []const u8,
    serial_number: []const u8,
    pem_data: []const u8,
    revoked: bool,
    valid_from: i64,
    valid_to: i64,
    created_at: i64,
};

pub const UserPermissionGrant = struct {
    user_id: u32,
    permission: Permission,
    granted: bool,
};

pub const UserPrivilegeGrant = struct {
    user_id: u32,
    privilege: Privilege,
    enabled: bool,
};

pub const EncryptedPayload = struct {
    nonce: [16]u8,
    mac: [32]u8,
    ciphertext: []u8,
};

pub const UserError = error{
    UserNotFound,
    UsernameTaken,
    InvalidCredentials,
    UserDisabled,
    AlreadyLoggedIn,
    NotLoggedIn,
    CredentialNotFound,
    ProfileNotFound,
    ProfileOwnershipMismatch,
    ProfileAccountNotFound,
    PersonaNotFound,
    PersonaOwnershipMismatch,
    ContactNotFound,
    ContactOwnershipMismatch,
    AccountOwnershipMismatch,
    TokenNotFound,
    TokenExpired,
    KeyNotFound,
    CertificateNotFound,
    PermissionDenied,
    EncryptionFailed,
    DecryptionFailed,
};

pub const UserManager = struct {
    allocator: std.mem.Allocator,
    users: std.array_list.Managed(User),
    credentials: std.array_list.Managed(Credential),
    personas: std.array_list.Managed(Persona),
    auth_tokens: std.array_list.Managed(AuthToken),
    keys: std.array_list.Managed(UserKey),
    certificates: std.array_list.Managed(UserCertificate),
    permission_overrides: std.array_list.Managed(UserPermissionGrant),
    privilege_overrides: std.array_list.Managed(UserPrivilegeGrant),
    identity_manager: identity_module.IdentityManager,
    profile_management: profile_module.ProfileManagement,
    account_management: accounts_module.AccountManagement,
    contact_management: contacts_module.ContactManagement,
    provider_manager: providers_module.ProviderManager,
    current_user_id: ?u32,
    next_credential_id: u32,
    next_persona_id: u32,
    next_token_id: u32,
    next_key_id: u32,
    next_certificate_id: u32,
    encryption_master_key: [32]u8,

    pub fn init(allocator: std.mem.Allocator) UserManager {
        var master_key: [32]u8 = undefined;
        std.crypto.random.bytes(&master_key);
        return UserManager{
            .allocator = allocator,
            .users = std.array_list.Managed(User).init(allocator),
            .credentials = std.array_list.Managed(Credential).init(allocator),
            .personas = std.array_list.Managed(Persona).init(allocator),
            .auth_tokens = std.array_list.Managed(AuthToken).init(allocator),
            .keys = std.array_list.Managed(UserKey).init(allocator),
            .certificates = std.array_list.Managed(UserCertificate).init(allocator),
            .permission_overrides = std.array_list.Managed(UserPermissionGrant).init(allocator),
            .privilege_overrides = std.array_list.Managed(UserPrivilegeGrant).init(allocator),
            .identity_manager = identity_module.IdentityManager.init(allocator),
            .profile_management = profile_module.ProfileManagement.init(allocator),
            .account_management = accounts_module.AccountManagement.init(allocator),
            .contact_management = contacts_module.ContactManagement.init(allocator),
            .provider_manager = providers_module.ProviderManager.init(allocator),
            .current_user_id = null,
            .next_credential_id = 0,
            .next_persona_id = 0,
            .next_token_id = 0,
            .next_key_id = 0,
            .next_certificate_id = 0,
            .encryption_master_key = master_key,
        };
    }

    pub fn deinit(self: *UserManager) void {
        for (self.users.items) |user| {
            self.allocator.free(user.username);
            self.allocator.free(user.email);
        }
        self.users.deinit();

        for (self.credentials.items) |cred| {
            self.allocator.free(cred.label);
            self.allocator.free(cred.secret_hash);
        }
        self.credentials.deinit();

        for (self.personas.items) |persona| {
            self.allocator.free(persona.name);
            self.allocator.free(persona.description);
            self.allocator.free(persona.focus_area);
            self.allocator.free(persona.tone);
        }
        self.personas.deinit();

        for (self.auth_tokens.items) |token| {
            self.allocator.free(token.label);
            self.allocator.free(token.token_hash);
        }
        self.auth_tokens.deinit();

        for (self.keys.items) |key| {
            self.allocator.free(key.label);
            self.allocator.free(key.key_hash);
            self.allocator.free(key.public_material);
        }
        self.keys.deinit();

        for (self.certificates.items) |cert| {
            self.allocator.free(cert.label);
            self.allocator.free(cert.subject);
            self.allocator.free(cert.issuer);
            self.allocator.free(cert.serial_number);
            self.allocator.free(cert.pem_data);
        }
        self.certificates.deinit();
        self.permission_overrides.deinit();
        self.privilege_overrides.deinit();

        self.identity_manager.deinit();
        self.profile_management.deinit();
        self.account_management.deinit();
        self.contact_management.deinit();
        self.provider_manager.deinit();
    }

    // ========== Cohesive User + Identity Lifecycle ==========

    pub fn createUser(
        self: *UserManager,
        username: []const u8,
        email: []const u8,
        password: []const u8,
        role: UserRole,
    ) !u32 {
        if (self.findUserIndexByUsername(username) != null) return UserError.UsernameTaken;

        const identity_id = try self.identity_manager.createIdentity(
            username,
            email,
            0.0,
            roleToIdentityType(role),
        );

        const user_id = @as(u32, @intCast(self.users.items.len));
        const user = User{
            .id = user_id,
            .username = try self.allocator.dupe(u8, username),
            .email = try self.allocator.dupe(u8, email),
            .role = role,
            .linked_identity_id = identity_id,
            .active = true,
            .created_at = std.time.timestamp(),
            .last_login_at = null,
        };
        try self.users.append(user);

        _ = try self.profile_management.createProfile(user_id, .personal, "default-personal", "");
        try self.setPassword(user_id, password);
        return user_id;
    }

    pub fn disableUser(self: *UserManager, user_id: u32) !void {
        const idx = self.getUserIndexById(user_id) orelse return UserError.UserNotFound;
        self.users.items[idx].active = false;
        if (self.current_user_id != null and self.current_user_id.? == user_id) {
            self.current_user_id = null;
        }

        for (self.credentials.items) |*cred| {
            if (cred.user_id == user_id) cred.active = false;
        }
        for (self.personas.items) |*persona| {
            if (persona.user_id == user_id) persona.active = false;
        }
        for (self.profile_management.profiles.items) |*profile| {
            if (profile.owner_user_id == user_id) profile.active = false;
        }
        for (self.account_management.accounts.items) |*acct| {
            if (acct.owner_user_id == user_id) acct.active = false;
        }
        for (self.contact_management.contacts.items) |*contact| {
            if (contact.owner_user_id == user_id) contact.active = false;
        }
        for (self.auth_tokens.items) |*token| {
            if (token.user_id == user_id) token.active = false;
        }
        for (self.keys.items) |*key| {
            if (key.user_id == user_id) key.active = false;
        }
        for (self.certificates.items) |*cert| {
            if (cert.user_id == user_id) cert.revoked = true;
        }
        if (self.users.items[idx].linked_identity_id) |identity_id| {
            self.identity_manager.deactivateIdentity(identity_id);
        }
    }

    pub fn login(self: *UserManager, username: []const u8, password: []const u8) !u32 {
        if (self.current_user_id != null) return UserError.AlreadyLoggedIn;

        const idx = self.findUserIndexByUsername(username) orelse return UserError.InvalidCredentials;
        const user = &self.users.items[idx];
        if (!user.active) return UserError.UserDisabled;
        if (!(try self.verifyPassword(user.id, password))) return UserError.InvalidCredentials;

        user.last_login_at = std.time.timestamp();
        self.current_user_id = user.id;
        return user.id;
    }

    pub fn logout(self: *UserManager) !void {
        if (self.current_user_id == null) return UserError.NotLoggedIn;
        self.current_user_id = null;
    }

    pub fn getCurrentUser(self: *UserManager) ?User {
        const uid = self.current_user_id orelse return null;
        const idx = self.getUserIndexById(uid) orelse return null;
        return self.users.items[idx];
    }

    pub fn getUsers(self: *UserManager) []User {
        return self.users.items;
    }

    // ========== Credential Management ==========

    pub fn setPassword(self: *UserManager, user_id: u32, password: []const u8) !void {
        _ = self.getUserIndexById(user_id) orelse return UserError.UserNotFound;
        const hash = try hashSecret(self.allocator, password);
        errdefer self.allocator.free(hash);

        if (self.findPrimaryPasswordCredentialIndex(user_id)) |idx| {
            const cred = &self.credentials.items[idx];
            self.allocator.free(cred.secret_hash);
            cred.secret_hash = hash;
            cred.active = true;
            cred.last_used_at = null;
            return;
        }

        const cred = Credential{
            .id = self.next_credential_id,
            .user_id = user_id,
            .label = try self.allocator.dupe(u8, "primary"),
            .kind = .password,
            .secret_hash = hash,
            .active = true,
            .created_at = std.time.timestamp(),
            .last_used_at = null,
        };
        self.next_credential_id += 1;
        try self.credentials.append(cred);
    }

    pub fn verifyPassword(self: *UserManager, user_id: u32, password: []const u8) !bool {
        const idx = self.findPrimaryPasswordCredentialIndex(user_id) orelse return false;
        const cred = &self.credentials.items[idx];
        if (!cred.active) return false;

        const supplied_hash = try hashSecret(self.allocator, password);
        defer self.allocator.free(supplied_hash);

        const ok = std.mem.eql(u8, cred.secret_hash, supplied_hash);
        if (ok) cred.last_used_at = std.time.timestamp();
        return ok;
    }

    pub fn addCredential(
        self: *UserManager,
        user_id: u32,
        label: []const u8,
        kind: CredentialKind,
        secret: []const u8,
    ) !u32 {
        _ = self.getUserIndexById(user_id) orelse return UserError.UserNotFound;
        const secret_hash = try hashSecret(self.allocator, secret);
        errdefer self.allocator.free(secret_hash);

        const cred_id = self.next_credential_id;
        const cred = Credential{
            .id = cred_id,
            .user_id = user_id,
            .label = try self.allocator.dupe(u8, label),
            .kind = kind,
            .secret_hash = secret_hash,
            .active = true,
            .created_at = std.time.timestamp(),
            .last_used_at = null,
        };
        self.next_credential_id += 1;
        try self.credentials.append(cred);
        return cred_id;
    }

    pub fn disableCredential(self: *UserManager, credential_id: u32) !void {
        const idx = self.findCredentialIndexById(credential_id) orelse return UserError.CredentialNotFound;
        self.credentials.items[idx].active = false;
    }

    pub fn getCredentials(self: *UserManager) []Credential {
        return self.credentials.items;
    }

    // ========== Authentication (Tokens) ==========

    pub fn issueAuthToken(
        self: *UserManager,
        user_id: u32,
        label: []const u8,
        kind: AuthTokenKind,
        ttl_seconds: i64,
    ) !IssuedToken {
        _ = self.getUserIndexById(user_id) orelse return UserError.UserNotFound;

        const token_value = try self.generateTokenValue();
        errdefer self.allocator.free(token_value);
        const token_hash = try hashSecret(self.allocator, token_value);
        errdefer self.allocator.free(token_hash);

        const now = std.time.timestamp();
        const expires_at = now + ttl_seconds;
        const token_id = self.next_token_id;
        self.next_token_id += 1;

        const entry = AuthToken{
            .id = token_id,
            .user_id = user_id,
            .label = try self.allocator.dupe(u8, label),
            .kind = kind,
            .token_hash = token_hash,
            .active = true,
            .created_at = now,
            .expires_at = expires_at,
            .last_used_at = null,
        };
        try self.auth_tokens.append(entry);
        return IssuedToken{
            .id = token_id,
            .value = token_value,
            .expires_at = expires_at,
        };
    }

    pub fn freeIssuedToken(self: *UserManager, token: IssuedToken) void {
        self.allocator.free(token.value);
    }

    pub fn authenticateToken(self: *UserManager, token_value: []const u8) !u32 {
        const token_hash = try hashSecret(self.allocator, token_value);
        defer self.allocator.free(token_hash);

        for (self.auth_tokens.items) |*token| {
            if (!token.active) continue;
            if (!std.mem.eql(u8, token.token_hash, token_hash)) continue;
            if (std.time.timestamp() > token.expires_at) return UserError.TokenExpired;
            token.last_used_at = std.time.timestamp();
            return token.user_id;
        }
        return UserError.TokenNotFound;
    }

    pub fn revokeAuthToken(self: *UserManager, token_id: u32) !void {
        const idx = self.findAuthTokenIndexById(token_id) orelse return UserError.TokenNotFound;
        self.auth_tokens.items[idx].active = false;
    }

    pub fn getAuthTokens(self: *UserManager) []AuthToken {
        return self.auth_tokens.items;
    }

    // ========== Key Management ==========

    pub fn addKey(
        self: *UserManager,
        user_id: u32,
        label: []const u8,
        kind: KeyKind,
        key_material: []const u8,
        public_material: []const u8,
    ) !u32 {
        _ = self.getUserIndexById(user_id) orelse return UserError.UserNotFound;
        const key_hash = try hashSecret(self.allocator, key_material);
        errdefer self.allocator.free(key_hash);

        const key_id = self.next_key_id;
        self.next_key_id += 1;

        const entry = UserKey{
            .id = key_id,
            .user_id = user_id,
            .label = try self.allocator.dupe(u8, label),
            .kind = kind,
            .key_hash = key_hash,
            .public_material = try self.allocator.dupe(u8, public_material),
            .active = true,
            .created_at = std.time.timestamp(),
            .last_used_at = null,
        };
        try self.keys.append(entry);
        return key_id;
    }

    pub fn verifyKey(self: *UserManager, user_id: u32, key_material: []const u8) !bool {
        _ = self.getUserIndexById(user_id) orelse return UserError.UserNotFound;
        const key_hash = try hashSecret(self.allocator, key_material);
        defer self.allocator.free(key_hash);

        for (self.keys.items) |*key| {
            if (key.user_id != user_id or !key.active) continue;
            if (std.mem.eql(u8, key.key_hash, key_hash)) {
                key.last_used_at = std.time.timestamp();
                return true;
            }
        }
        return false;
    }

    pub fn disableKey(self: *UserManager, key_id: u32) !void {
        const idx = self.findKeyIndexById(key_id) orelse return UserError.KeyNotFound;
        self.keys.items[idx].active = false;
    }

    pub fn getKeys(self: *UserManager) []UserKey {
        return self.keys.items;
    }

    // ========== Certificate Management ==========

    pub fn addCertificate(
        self: *UserManager,
        user_id: u32,
        label: []const u8,
        subject: []const u8,
        issuer: []const u8,
        serial_number: []const u8,
        pem_data: []const u8,
        valid_from: i64,
        valid_to: i64,
    ) !u32 {
        _ = self.getUserIndexById(user_id) orelse return UserError.UserNotFound;
        const cert_id = self.next_certificate_id;
        self.next_certificate_id += 1;

        const entry = UserCertificate{
            .id = cert_id,
            .user_id = user_id,
            .label = try self.allocator.dupe(u8, label),
            .subject = try self.allocator.dupe(u8, subject),
            .issuer = try self.allocator.dupe(u8, issuer),
            .serial_number = try self.allocator.dupe(u8, serial_number),
            .pem_data = try self.allocator.dupe(u8, pem_data),
            .revoked = false,
            .valid_from = valid_from,
            .valid_to = valid_to,
            .created_at = std.time.timestamp(),
        };
        try self.certificates.append(entry);
        return cert_id;
    }

    pub fn revokeCertificate(self: *UserManager, certificate_id: u32) !void {
        const idx = self.findCertificateIndexById(certificate_id) orelse return UserError.CertificateNotFound;
        self.certificates.items[idx].revoked = true;
    }

    pub fn validateCertificate(self: *UserManager, certificate_id: u32) !bool {
        const idx = self.findCertificateIndexById(certificate_id) orelse return UserError.CertificateNotFound;
        const cert = self.certificates.items[idx];
        if (cert.revoked) return false;
        const now = std.time.timestamp();
        return now >= cert.valid_from and now <= cert.valid_to;
    }

    pub fn getCertificates(self: *UserManager) []UserCertificate {
        return self.certificates.items;
    }

    // ========== Authorization (Permissions & Privileges) ==========

    pub fn grantPermission(self: *UserManager, user_id: u32, permission: Permission) !void {
        _ = self.getUserIndexById(user_id) orelse return UserError.UserNotFound;
        for (self.permission_overrides.items) |*entry| {
            if (entry.user_id == user_id and entry.permission == permission) {
                entry.granted = true;
                return;
            }
        }
        try self.permission_overrides.append(.{
            .user_id = user_id,
            .permission = permission,
            .granted = true,
        });
    }

    pub fn revokePermission(self: *UserManager, user_id: u32, permission: Permission) !void {
        _ = self.getUserIndexById(user_id) orelse return UserError.UserNotFound;
        for (self.permission_overrides.items) |*entry| {
            if (entry.user_id == user_id and entry.permission == permission) {
                entry.granted = false;
                return;
            }
        }
        try self.permission_overrides.append(.{
            .user_id = user_id,
            .permission = permission,
            .granted = false,
        });
    }

    pub fn hasPermission(self: *UserManager, user_id: u32, permission: Permission) bool {
        const idx = self.getUserIndexById(user_id) orelse return false;
        for (self.permission_overrides.items) |entry| {
            if (entry.user_id == user_id and entry.permission == permission) return entry.granted;
        }
        return roleHasPermission(self.users.items[idx].role, permission);
    }

    pub fn grantPrivilege(self: *UserManager, user_id: u32, privilege: Privilege) !void {
        _ = self.getUserIndexById(user_id) orelse return UserError.UserNotFound;
        for (self.privilege_overrides.items) |*entry| {
            if (entry.user_id == user_id and entry.privilege == privilege) {
                entry.enabled = true;
                return;
            }
        }
        try self.privilege_overrides.append(.{
            .user_id = user_id,
            .privilege = privilege,
            .enabled = true,
        });
    }

    pub fn revokePrivilege(self: *UserManager, user_id: u32, privilege: Privilege) !void {
        _ = self.getUserIndexById(user_id) orelse return UserError.UserNotFound;
        for (self.privilege_overrides.items) |*entry| {
            if (entry.user_id == user_id and entry.privilege == privilege) {
                entry.enabled = false;
                return;
            }
        }
        try self.privilege_overrides.append(.{
            .user_id = user_id,
            .privilege = privilege,
            .enabled = false,
        });
    }

    pub fn hasPrivilege(self: *UserManager, user_id: u32, privilege: Privilege) bool {
        const idx = self.getUserIndexById(user_id) orelse return false;
        for (self.privilege_overrides.items) |entry| {
            if (entry.user_id == user_id and entry.privilege == privilege) return entry.enabled;
        }
        return roleHasPrivilege(self.users.items[idx].role, privilege);
    }

    pub fn requirePermission(self: *UserManager, user_id: u32, permission: Permission) !void {
        if (!self.hasPermission(user_id, permission)) return UserError.PermissionDenied;
    }

    // ========== Encryption ==========

    pub fn encryptForUser(self: *UserManager, user_id: u32, plaintext: []const u8) !EncryptedPayload {
        _ = self.getUserIndexById(user_id) orelse return UserError.UserNotFound;
        var payload = EncryptedPayload{
            .nonce = undefined,
            .mac = undefined,
            .ciphertext = try self.allocator.alloc(u8, plaintext.len),
        };
        errdefer self.allocator.free(payload.ciphertext);

        std.crypto.random.bytes(&payload.nonce);
        const enc_key = deriveUserScopedKey(self.encryption_master_key, user_id, payload.nonce, "enc");
        const mac_key = deriveUserScopedKey(self.encryption_master_key, user_id, payload.nonce, "mac");

        xorWithKeystream(payload.ciphertext, plaintext, enc_key, payload.nonce);
        payload.mac = computePayloadMac(mac_key, payload.nonce, payload.ciphertext);
        return payload;
    }

    pub fn decryptForUser(self: *UserManager, user_id: u32, payload: EncryptedPayload) ![]u8 {
        _ = self.getUserIndexById(user_id) orelse return UserError.UserNotFound;
        const enc_key = deriveUserScopedKey(self.encryption_master_key, user_id, payload.nonce, "enc");
        const mac_key = deriveUserScopedKey(self.encryption_master_key, user_id, payload.nonce, "mac");
        const expected_mac = computePayloadMac(mac_key, payload.nonce, payload.ciphertext);
        if (!std.mem.eql(u8, &expected_mac, &payload.mac)) return UserError.DecryptionFailed;

        const plaintext = try self.allocator.alloc(u8, payload.ciphertext.len);
        errdefer self.allocator.free(plaintext);
        xorWithKeystream(plaintext, payload.ciphertext, enc_key, payload.nonce);
        return plaintext;
    }

    pub fn freeEncryptedPayload(self: *UserManager, payload: EncryptedPayload) void {
        self.allocator.free(payload.ciphertext);
    }

    // ========== Provider Management ==========

    pub fn addProvider(
        self: *UserManager,
        name: []const u8,
        slug: []const u8,
        category: providers_module.ProviderCategory,
        homepage_url: []const u8,
        api_base_url: []const u8,
    ) !u32 {
        return try self.provider_manager.addProvider(name, slug, category, homepage_url, api_base_url);
    }

    pub fn disableProvider(self: *UserManager, provider_id: u32) !void {
        try self.provider_manager.disableProvider(provider_id);
    }

    pub fn enableProvider(self: *UserManager, provider_id: u32) !void {
        try self.provider_manager.enableProvider(provider_id);
    }

    pub fn getProviders(self: *UserManager) []providers_module.Provider {
        return self.provider_manager.getProviders();
    }

    pub fn getProvider(self: *UserManager, provider_id: u32) !providers_module.Provider {
        return try self.provider_manager.getProvider(provider_id);
    }

    pub fn findProviderBySlug(self: *UserManager, slug: []const u8) ?providers_module.Provider {
        return self.provider_manager.findBySlug(slug);
    }

    // ========== Profile Management ==========

    pub fn createProfile(
        self: *UserManager,
        user_id: u32,
        profile_type: ProfileType,
        name: []const u8,
        description: []const u8,
    ) !u32 {
        _ = self.getUserIndexById(user_id) orelse return UserError.UserNotFound;
        return try self.profile_management.createProfile(user_id, profile_type, name, description);
    }

    pub fn setActiveProfile(self: *UserManager, user_id: u32, profile_id: u32) !void {
        try self.profile_management.setActiveProfile(user_id, profile_id);
    }

    pub fn updateProfileMetadata(self: *UserManager, profile_id: u32, name: []const u8, description: []const u8) !void {
        try self.profile_management.updateProfileMetadata(profile_id, name, description);
    }

    pub fn setProfileConfiguration(self: *UserManager, profile_id: u32, key: []const u8, value: []const u8) !void {
        try self.profile_management.setConfiguration(profile_id, key, value);
    }

    pub fn setProfilePreference(self: *UserManager, profile_id: u32, key: []const u8, value: []const u8) !void {
        try self.profile_management.setPreference(profile_id, key, value);
    }

    pub fn getProfiles(self: *UserManager) []UserProfile {
        return self.profile_management.getProfiles();
    }

    pub fn getProfileById(self: *UserManager, profile_id: u32) !UserProfile {
        return try self.profile_management.getProfileById(profile_id);
    }

    pub fn getActiveProfile(self: *UserManager, user_id: u32) ?UserProfile {
        return self.profile_management.getActiveProfile(user_id);
    }

    // Backward-compatible adapter for previous user-profile setter.
    pub fn setProfile(
        self: *UserManager,
        user_id: u32,
        display_name: []const u8,
        bio: []const u8,
        timezone: []const u8,
        locale: []const u8,
        avatar_url: []const u8,
    ) !void {
        _ = self.getUserIndexById(user_id) orelse return UserError.UserNotFound;

        const profile_id = blk: {
            if (self.getActiveProfile(user_id)) |p| break :blk p.id;
            for (self.profile_management.profiles.items) |p| {
                if (p.owner_user_id == user_id) break :blk p.id;
            }
            break :blk try self.createProfile(user_id, .personal, display_name, bio);
        };

        try self.updateProfileMetadata(profile_id, display_name, bio);
        try self.setProfilePreference(profile_id, "timezone", timezone);
        try self.setProfilePreference(profile_id, "locale", locale);
        try self.setProfilePreference(profile_id, "avatar_url", avatar_url);
    }

    pub fn getProfile(self: *UserManager, user_id: u32) !UserProfile {
        if (self.getActiveProfile(user_id)) |profile| return profile;
        for (self.profile_management.profiles.items) |profile| {
            if (profile.owner_user_id == user_id) return profile;
        }
        return UserError.ProfileNotFound;
    }

    // ========== Account Management ==========

    pub fn addProfileAccount(
        self: *UserManager,
        profile_id: u32,
        account_type: AccountType,
        username: []const u8,
        provider: []const u8,
        details: []const u8,
    ) !u32 {
        if (!self.provider_manager.isActiveBySlug(provider)) {
            return providers_module.ProviderError.ProviderNotFound;
        }
        const profile = try self.profile_management.getProfileById(profile_id);
        const account_id = try self.account_management.createAccount(
            profile.owner_user_id,
            account_type,
            provider,
            username,
            details,
        );
        try self.account_management.associateToProfile(account_id, profile_id);
        try self.profile_management.linkAccount(profile_id, account_id);
        return account_id;
    }

    pub fn disableProfileAccount(self: *UserManager, account_id: u32) !void {
        try self.account_management.deactivateAccount(account_id);
    }

    pub fn getProfileAccounts(self: *UserManager) []ProfileAccount {
        return self.account_management.getAccounts();
    }

    // ========== Contact Management ==========

    pub fn createContact(
        self: *UserManager,
        user_id: u32,
        profile_id: ?u32,
        kind: ContactKind,
        display_name: []const u8,
        notes: []const u8,
    ) !u32 {
        _ = self.getUserIndexById(user_id) orelse return UserError.UserNotFound;
        if (profile_id) |pid| {
            const profile = try self.profile_management.getProfileById(pid);
            if (profile.owner_user_id != user_id) return UserError.ProfileOwnershipMismatch;
        }
        return try self.contact_management.createContact(user_id, profile_id, kind, display_name, notes);
    }

    pub fn setContactProfile(self: *UserManager, user_id: u32, contact_id: u32, profile_id: ?u32) !void {
        _ = self.getUserIndexById(user_id) orelse return UserError.UserNotFound;
        const contact = try self.contact_management.getContactById(contact_id);
        if (contact.owner_user_id != user_id) return UserError.ContactOwnershipMismatch;
        if (profile_id) |pid| {
            const profile = try self.profile_management.getProfileById(pid);
            if (profile.owner_user_id != user_id) return UserError.ProfileOwnershipMismatch;
        }
        try self.contact_management.setProfile(contact_id, profile_id);
    }

    pub fn updateContact(self: *UserManager, user_id: u32, contact_id: u32, display_name: []const u8, notes: []const u8) !void {
        _ = self.getUserIndexById(user_id) orelse return UserError.UserNotFound;
        const contact = try self.contact_management.getContactById(contact_id);
        if (contact.owner_user_id != user_id) return UserError.ContactOwnershipMismatch;
        try self.contact_management.updateDetails(contact_id, display_name, notes);
    }

    pub fn addContactChannel(
        self: *UserManager,
        user_id: u32,
        contact_id: u32,
        kind: ContactChannelKind,
        label: []const u8,
        value: []const u8,
    ) !void {
        _ = self.getUserIndexById(user_id) orelse return UserError.UserNotFound;
        const contact = try self.contact_management.getContactById(contact_id);
        if (contact.owner_user_id != user_id) return UserError.ContactOwnershipMismatch;
        try self.contact_management.addChannel(contact_id, kind, label, value);
    }

    pub fn verifyContactChannel(self: *UserManager, user_id: u32, contact_id: u32, channel_index: usize) !void {
        _ = self.getUserIndexById(user_id) orelse return UserError.UserNotFound;
        const contact = try self.contact_management.getContactById(contact_id);
        if (contact.owner_user_id != user_id) return UserError.ContactOwnershipMismatch;
        try self.contact_management.verifyChannel(contact_id, channel_index);
    }

    pub fn addContactTag(self: *UserManager, user_id: u32, contact_id: u32, tag: []const u8) !void {
        _ = self.getUserIndexById(user_id) orelse return UserError.UserNotFound;
        const contact = try self.contact_management.getContactById(contact_id);
        if (contact.owner_user_id != user_id) return UserError.ContactOwnershipMismatch;
        try self.contact_management.addTag(contact_id, tag);
    }

    pub fn linkContactToAccount(self: *UserManager, user_id: u32, contact_id: u32, account_id: u32) !void {
        _ = self.getUserIndexById(user_id) orelse return UserError.UserNotFound;
        const contact = try self.contact_management.getContactById(contact_id);
        if (contact.owner_user_id != user_id) return UserError.ContactOwnershipMismatch;
        const acct = self.getManagedAccountById(account_id) orelse return UserError.ProfileAccountNotFound;
        if (acct.owner_user_id != user_id) return UserError.AccountOwnershipMismatch;
        try self.contact_management.linkAccount(contact_id, account_id);
    }

    pub fn unlinkContactFromAccount(self: *UserManager, user_id: u32, contact_id: u32, account_id: u32) !void {
        _ = self.getUserIndexById(user_id) orelse return UserError.UserNotFound;
        const contact = try self.contact_management.getContactById(contact_id);
        if (contact.owner_user_id != user_id) return UserError.ContactOwnershipMismatch;
        try self.contact_management.unlinkAccount(contact_id, account_id);
    }

    pub fn deactivateContact(self: *UserManager, user_id: u32, contact_id: u32) !void {
        _ = self.getUserIndexById(user_id) orelse return UserError.UserNotFound;
        const contact = try self.contact_management.getContactById(contact_id);
        if (contact.owner_user_id != user_id) return UserError.ContactOwnershipMismatch;
        try self.contact_management.deactivateContact(contact_id);
    }

    pub fn getContacts(self: *UserManager) []UserContact {
        return self.contact_management.getContacts();
    }

    pub fn getContactById(self: *UserManager, contact_id: u32) !UserContact {
        return try self.contact_management.getContactById(contact_id);
    }

    // ========== Identity Management ==========

    pub fn createIdentity(
        self: *UserManager,
        name: []const u8,
        email: []const u8,
        hourly_rate: f32,
        itype: identity_module.IdentityType,
    ) !u32 {
        return try self.identity_manager.createIdentity(name, email, hourly_rate, itype);
    }

    pub fn addIdentitySkill(self: *UserManager, identity_id: u32, skill: []const u8) !void {
        try self.identity_manager.addSkill(identity_id, skill);
    }

    pub fn getActiveIdentitiesCount(self: *UserManager) u32 {
        return self.identity_manager.getActiveCount();
    }

    pub fn getIdentities(self: *UserManager) []identity_module.Identity {
        return self.identity_manager.identities.items;
    }

    pub fn addIdentityConnection(
        self: *UserManager,
        identity_id: u32,
        conn_type: registry_module.ConnectionType,
        username: []const u8,
        provider: []const u8,
        details: []const u8,
    ) !u32 {
        if (!self.provider_manager.isActiveBySlug(provider)) {
            return providers_module.ProviderError.ProviderNotFound;
        }
        return try self.identity_manager.addIdentityConnection(identity_id, conn_type, username, provider, details);
    }

    pub fn deactivateIdentityConnection(self: *UserManager, identity_id: u32, conn_id: u32) !void {
        try self.identity_manager.deactivateIdentityConnection(identity_id, conn_id);
    }

    pub fn getIdentityActiveConnectionCount(self: *UserManager, identity_id: u32) u32 {
        return self.identity_manager.getIdentityActiveConnectionCount(identity_id);
    }

    // ========== Persona Management ==========

    pub fn addPersona(
        self: *UserManager,
        user_id: u32,
        name: []const u8,
        description: []const u8,
        focus_area: []const u8,
        tone: []const u8,
    ) !u32 {
        _ = self.getUserIndexById(user_id) orelse return UserError.UserNotFound;
        const persona_id = self.next_persona_id;
        self.next_persona_id += 1;

        const persona = Persona{
            .id = persona_id,
            .user_id = user_id,
            .name = try self.allocator.dupe(u8, name),
            .description = try self.allocator.dupe(u8, description),
            .focus_area = try self.allocator.dupe(u8, focus_area),
            .tone = try self.allocator.dupe(u8, tone),
            .active = false,
            .created_at = std.time.timestamp(),
        };
        try self.personas.append(persona);
        return persona_id;
    }

    pub fn activatePersona(self: *UserManager, user_id: u32, persona_id: u32) !void {
        _ = self.getUserIndexById(user_id) orelse return UserError.UserNotFound;
        const idx = self.findPersonaIndexById(persona_id) orelse return UserError.PersonaNotFound;
        if (self.personas.items[idx].user_id != user_id) return UserError.PersonaOwnershipMismatch;

        for (self.personas.items) |*persona| {
            if (persona.user_id == user_id) persona.active = false;
        }
        self.personas.items[idx].active = true;
    }

    pub fn getActivePersona(self: *UserManager, user_id: u32) ?Persona {
        for (self.personas.items) |persona| {
            if (persona.user_id == user_id and persona.active) return persona;
        }
        return null;
    }

    pub fn getPersonas(self: *UserManager) []Persona {
        return self.personas.items;
    }

    fn findUserIndexByUsername(self: *UserManager, username: []const u8) ?usize {
        for (self.users.items, 0..) |user, idx| {
            if (std.mem.eql(u8, user.username, username)) return idx;
        }
        return null;
    }

    fn getUserIndexById(self: *UserManager, user_id: u32) ?usize {
        if (user_id >= @as(u32, @intCast(self.users.items.len))) return null;
        return @as(usize, @intCast(user_id));
    }

    fn findCredentialIndexById(self: *UserManager, credential_id: u32) ?usize {
        for (self.credentials.items, 0..) |cred, idx| {
            if (cred.id == credential_id) return idx;
        }
        return null;
    }

    fn findPrimaryPasswordCredentialIndex(self: *UserManager, user_id: u32) ?usize {
        for (self.credentials.items, 0..) |cred, idx| {
            if (cred.user_id == user_id and cred.kind == .password and std.mem.eql(u8, cred.label, "primary")) {
                return idx;
            }
        }
        return null;
    }

    fn findPersonaIndexById(self: *UserManager, persona_id: u32) ?usize {
        for (self.personas.items, 0..) |persona, idx| {
            if (persona.id == persona_id) return idx;
        }
        return null;
    }

    fn findAuthTokenIndexById(self: *UserManager, token_id: u32) ?usize {
        for (self.auth_tokens.items, 0..) |token, idx| {
            if (token.id == token_id) return idx;
        }
        return null;
    }

    fn findKeyIndexById(self: *UserManager, key_id: u32) ?usize {
        for (self.keys.items, 0..) |key, idx| {
            if (key.id == key_id) return idx;
        }
        return null;
    }

    fn findCertificateIndexById(self: *UserManager, certificate_id: u32) ?usize {
        for (self.certificates.items, 0..) |cert, idx| {
            if (cert.id == certificate_id) return idx;
        }
        return null;
    }

    fn getManagedAccountById(self: *UserManager, account_id: u32) ?accounts_module.ManagedAccount {
        for (self.account_management.accounts.items) |account| {
            if (account.id == account_id) return account;
        }
        return null;
    }

    fn generateTokenValue(self: *UserManager) ![]u8 {
        var random_bytes: [32]u8 = undefined;
        std.crypto.random.bytes(&random_bytes);
        var out: [64]u8 = undefined;
        _ = try std.fmt.bufPrint(&out, "{}", .{std.fmt.fmtSliceHexLower(&random_bytes)});
        return self.allocator.dupe(u8, &out);
    }
};

fn roleToIdentityType(role: UserRole) identity_module.IdentityType {
    return switch (role) {
        .admin => .consultant,
        .worker => .developer,
        .viewer => .other,
    };
}

pub fn parseRole(name: []const u8) ?UserRole {
    if (std.mem.eql(u8, name, "admin")) return .admin;
    if (std.mem.eql(u8, name, "worker")) return .worker;
    if (std.mem.eql(u8, name, "viewer")) return .viewer;
    return null;
}

pub fn parseProfileType(name: []const u8) ?ProfileType {
    return profile_module.parseProfileType(name);
}

pub fn parseCredentialKind(name: []const u8) ?CredentialKind {
    if (std.mem.eql(u8, name, "password")) return .password;
    if (std.mem.eql(u8, name, "api_token")) return .api_token;
    if (std.mem.eql(u8, name, "ssh_key")) return .ssh_key;
    return null;
}

pub fn parsePermission(name: []const u8) ?Permission {
    if (std.mem.eql(u8, name, "manage_users")) return .manage_users;
    if (std.mem.eql(u8, name, "manage_profiles")) return .manage_profiles;
    if (std.mem.eql(u8, name, "manage_accounts")) return .manage_accounts;
    if (std.mem.eql(u8, name, "manage_contacts")) return .manage_contacts;
    if (std.mem.eql(u8, name, "manage_providers")) return .manage_providers;
    if (std.mem.eql(u8, name, "manage_security")) return .manage_security;
    if (std.mem.eql(u8, name, "read_data")) return .read_data;
    if (std.mem.eql(u8, name, "write_data")) return .write_data;
    if (std.mem.eql(u8, name, "delete_data")) return .delete_data;
    return null;
}

pub fn parsePrivilege(name: []const u8) ?Privilege {
    if (std.mem.eql(u8, name, "superuser")) return .superuser;
    if (std.mem.eql(u8, name, "support_audit")) return .support_audit;
    if (std.mem.eql(u8, name, "automation")) return .automation;
    return null;
}

pub fn parseAuthTokenKind(name: []const u8) ?AuthTokenKind {
    if (std.mem.eql(u8, name, "session")) return .session;
    if (std.mem.eql(u8, name, "bearer")) return .bearer;
    if (std.mem.eql(u8, name, "refresh")) return .refresh;
    return null;
}

pub fn parseKeyKind(name: []const u8) ?KeyKind {
    if (std.mem.eql(u8, name, "api")) return .api;
    if (std.mem.eql(u8, name, "ssh")) return .ssh;
    if (std.mem.eql(u8, name, "signing")) return .signing;
    if (std.mem.eql(u8, name, "encryption")) return .encryption;
    return null;
}

pub fn parseAccountType(name: []const u8) ?AccountType {
    return accounts_module.parseAccountType(name);
}

pub fn parseContactKind(name: []const u8) ?ContactKind {
    return contacts_module.parseContactKind(name);
}

pub fn parseContactChannelKind(name: []const u8) ?ContactChannelKind {
    return contacts_module.parseContactChannelKind(name);
}

fn hashSecret(allocator: std.mem.Allocator, secret: []const u8) ![]u8 {
    var digest: [32]u8 = undefined;
    std.crypto.hash.sha2.Sha256.hash(secret, &digest, .{});

    const hex_buf = std.fmt.bytesToHex(digest, .lower);
    return allocator.dupe(u8, &hex_buf);
}

fn roleHasPermission(role: UserRole, permission: Permission) bool {
    return switch (role) {
        .admin => true,
        .worker => switch (permission) {
            .manage_profiles, .manage_accounts, .manage_contacts, .read_data, .write_data => true,
            else => false,
        },
        .viewer => switch (permission) {
            .read_data => true,
            else => false,
        },
    };
}

fn roleHasPrivilege(role: UserRole, privilege: Privilege) bool {
    return switch (role) {
        .admin => true,
        .worker => switch (privilege) {
            .automation => true,
            else => false,
        },
        .viewer => false,
    };
}

fn deriveUserScopedKey(master_key: [32]u8, user_id: u32, nonce: [16]u8, context: []const u8) [32]u8 {
    var input: [64]u8 = undefined;
    @memcpy(input[0..32], &master_key);
    input[32] = @as(u8, @intCast(user_id & 0xff));
    input[33] = @as(u8, @intCast((user_id >> 8) & 0xff));
    input[34] = @as(u8, @intCast((user_id >> 16) & 0xff));
    input[35] = @as(u8, @intCast((user_id >> 24) & 0xff));
    @memcpy(input[36..52], &nonce);
    for (input[52..64], 0..) |*b, i| {
        b.* = if (i < context.len) context[i] else 0;
    }

    var digest: [32]u8 = undefined;
    std.crypto.hash.sha2.Sha256.hash(&input, &digest, .{});
    return digest;
}

fn xorWithKeystream(dst: []u8, src: []const u8, key: [32]u8, nonce: [16]u8) void {
    var counter: u64 = 0;
    var offset: usize = 0;
    while (offset < src.len) {
        var block_input: [56]u8 = undefined;
        @memcpy(block_input[0..32], &key);
        @memcpy(block_input[32..48], &nonce);
        block_input[48] = @as(u8, @intCast(counter & 0xff));
        block_input[49] = @as(u8, @intCast((counter >> 8) & 0xff));
        block_input[50] = @as(u8, @intCast((counter >> 16) & 0xff));
        block_input[51] = @as(u8, @intCast((counter >> 24) & 0xff));
        block_input[52] = @as(u8, @intCast((counter >> 32) & 0xff));
        block_input[53] = @as(u8, @intCast((counter >> 40) & 0xff));
        block_input[54] = @as(u8, @intCast((counter >> 48) & 0xff));
        block_input[55] = @as(u8, @intCast((counter >> 56) & 0xff));

        var stream: [32]u8 = undefined;
        std.crypto.hash.sha2.Sha256.hash(&block_input, &stream, .{});

        var i: usize = 0;
        while (i < stream.len and offset + i < src.len) : (i += 1) {
            dst[offset + i] = src[offset + i] ^ stream[i];
        }
        offset += i;
        counter += 1;
    }
}

fn computePayloadMac(key: [32]u8, nonce: [16]u8, ciphertext: []const u8) [32]u8 {
    var hasher = std.crypto.hash.sha2.Sha256.init(.{});
    hasher.update(&key);
    hasher.update(&nonce);
    hasher.update(ciphertext);
    var mac: [32]u8 = undefined;
    hasher.final(&mac);
    return mac;
}

test "cohesive user lifecycle with identity/profile/account/provider" {
    const allocator = std.testing.allocator;
    var mgr = UserManager.init(allocator);
    defer mgr.deinit();

    _ = try mgr.addProvider("GitHub", "github", .code_hosting, "https://github.com", "https://api.github.com");
    const uid = try mgr.createUser("alice", "alice@example.com", "secret123", .worker);
    try std.testing.expectEqual(@as(u32, 0), uid);

    const p2 = try mgr.createProfile(uid, .work, "work", "work profile");
    try mgr.setProfileConfiguration(p2, "theme", "dark");
    const acct = try mgr.addProfileAccount(p2, .software, "alice-dev", "github", "primary");
    try std.testing.expectEqual(@as(u32, 0), acct);
    const contact = try mgr.createContact(uid, p2, .person, "Bob Stone", "project lead");
    try mgr.addContactChannel(uid, contact, .email, "work", "bob@example.com");
    try mgr.verifyContactChannel(uid, contact, 0);
    try mgr.addContactTag(uid, contact, "client");
    try mgr.linkContactToAccount(uid, contact, acct);
    try std.testing.expectEqual(@as(usize, 1), (try mgr.getContactById(contact)).linked_account_ids.items.len);

    const login_id = try mgr.login("alice", "secret123");
    try std.testing.expectEqual(uid, login_id);
    try mgr.logout();

    try std.testing.expect(mgr.getActiveIdentitiesCount() >= 1);

    // authn/authz/token/key/certificate/encryption lifecycle
    try std.testing.expect(mgr.hasPermission(uid, .manage_profiles));
    try std.testing.expect(!mgr.hasPermission(uid, .manage_users));
    try mgr.grantPermission(uid, .manage_users);
    try std.testing.expect(mgr.hasPermission(uid, .manage_users));

    const issued = try mgr.issueAuthToken(uid, "cli-session", .session, 3600);
    defer mgr.freeIssuedToken(issued);
    const token_user = try mgr.authenticateToken(issued.value);
    try std.testing.expectEqual(uid, token_user);

    const key_id = try mgr.addKey(uid, "primary-api", .api, "secret-api-key", "pk-live-123");
    try std.testing.expect(try mgr.verifyKey(uid, "secret-api-key"));
    try mgr.disableKey(key_id);
    try std.testing.expect(!(try mgr.verifyKey(uid, "secret-api-key")));

    const now = std.time.timestamp();
    const cert_id = try mgr.addCertificate(
        uid,
        "default-cert",
        "CN=alice",
        "CN=kogi-ca",
        "0001",
        "-----BEGIN CERTIFICATE-----x-----END CERTIFICATE-----",
        now - 60,
        now + 3600,
    );
    try std.testing.expect(try mgr.validateCertificate(cert_id));
    try mgr.revokeCertificate(cert_id);
    try std.testing.expect(!(try mgr.validateCertificate(cert_id)));

    const payload = try mgr.encryptForUser(uid, "hello secure world");
    defer mgr.freeEncryptedPayload(payload);
    const plain = try mgr.decryptForUser(uid, payload);
    defer allocator.free(plain);
    try std.testing.expect(std.mem.eql(u8, plain, "hello secure world"));
}
