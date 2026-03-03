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
};

pub const UserManager = struct {
    allocator: std.mem.Allocator,
    users: std.ArrayList(User),
    credentials: std.ArrayList(Credential),
    personas: std.ArrayList(Persona),
    identity_manager: identity_module.IdentityManager,
    profile_management: profile_module.ProfileManagement,
    account_management: accounts_module.AccountManagement,
    contact_management: contacts_module.ContactManagement,
    provider_manager: providers_module.ProviderManager,
    current_user_id: ?u32,
    next_credential_id: u32,
    next_persona_id: u32,

    pub fn init(allocator: std.mem.Allocator) UserManager {
        return UserManager{
            .allocator = allocator,
            .users = std.ArrayList(User).init(allocator),
            .credentials = std.ArrayList(Credential).init(allocator),
            .personas = std.ArrayList(Persona).init(allocator),
            .identity_manager = identity_module.IdentityManager.init(allocator),
            .profile_management = profile_module.ProfileManagement.init(allocator),
            .account_management = accounts_module.AccountManagement.init(allocator),
            .contact_management = contacts_module.ContactManagement.init(allocator),
            .provider_manager = providers_module.ProviderManager.init(allocator),
            .current_user_id = null,
            .next_credential_id = 0,
            .next_persona_id = 0,
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

    fn getManagedAccountById(self: *UserManager, account_id: u32) ?accounts_module.ManagedAccount {
        for (self.account_management.accounts.items) |account| {
            if (account.id == account_id) return account;
        }
        return null;
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

    var hex_buf: [64]u8 = undefined;
    _ = try std.fmt.bufPrint(&hex_buf, "{}", .{std.fmt.fmtSliceHexLower(&digest)});
    return allocator.dupe(u8, &hex_buf);
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
}
