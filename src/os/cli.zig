const std = @import("std");
const system_module = @import("system.zig");
const identity_module = @import("identity.zig");
const users_module = @import("users.zig");
const providers_module = @import("providers.zig");
const security_module = @import("security.zig");
const portfolio_module = @import("portfolio.zig");
const contract_module = @import("contract.zig");
const crm_module = @import("crm.zig");
const assets_module = @import("assets.zig");
const accounts_module = @import("accounts.zig");
const terminal_module = @import("terminal.zig");
const shell_module = @import("shell.zig");
const kernel_module = @import("kernel.zig");

/// Command-line interface for the KOGI Operating System
pub const CLI = struct {
    allocator: std.mem.Allocator,
    system: *system_module.System,

    pub fn init(allocator: std.mem.Allocator, system: *system_module.System) CLI {
        return CLI{
            .allocator = allocator,
            .system = system,
        };
    }

    /// Handle a single command line. Returns `true` to continue, `false` to exit.
    pub fn handleCommand(self: *CLI, input: []const u8) !bool {
        // Trim leading/trailing whitespace
        var start: usize = 0;
        var end: usize = input.len;
        while (start < end and (input[start] == ' ' or input[start] == '\t')) : (start += 1) {}
        while (end > start and (input[end - 1] == ' ' or input[end - 1] == '\t')) : (end -= 1) {}
        if (end <= start) return true; // empty

        const s = input[start..end];

        // Find first whitespace to separate command and args
        var cmd_end: usize = 0;
        while (cmd_end < s.len and s[cmd_end] != ' ' and s[cmd_end] != '\t') : (cmd_end += 1) {}
        const cmd = s[0..cmd_end];

        // Simple command dispatch
        if (std.mem.eql(u8, cmd, "help")) {
            std.debug.print("Available commands:\n", .{});
            std.debug.print("  help             - show this help\n", .{});
            std.debug.print("  stats            - show system statistics\n", .{});
            std.debug.print("  list-identities  - list identities\n", .{});
            std.debug.print("  list-tasks       - list tasks\n", .{});
            std.debug.print("  user ...         - user management\n", .{});
            std.debug.print("  provider ...     - provider management\n", .{});
            std.debug.print("  demo             - run demo mode\n", .{});
            std.debug.print("  exit|quit        - exit the shell\n", .{});
            return true;
        } else if (std.mem.eql(u8, cmd, "stats") or std.mem.eql(u8, cmd, "view-stats")) {
            self.handleViewStatistics();
            return true;
        } else if (std.mem.eql(u8, cmd, "list-identities")) {
            self.handleListIdentities();
            return true;
        } else if (std.mem.eql(u8, cmd, "list-tasks")) {
            self.handleListTasks();
            return true;
        } else if (std.mem.eql(u8, cmd, "demo")) {
            try self.runDemo();
            return true;
        } else if (std.mem.eql(u8, cmd, "syscalls")) {
            if (self.system.kernel) |_| {
                const names = kernel_module.syscallNames();
                for (names) |n| std.debug.print("{s}\n", .{n});
            } else {
                std.debug.print("No kernel linked; syscalls unavailable.\n", .{});
            }
            return true;
        } else if (std.mem.eql(u8, cmd, "shell")) {
            // Start the more featureful shell
            const kptr = self.system.kernel;
            try shell_module.runShell(self.allocator, self.system, kptr);
            return true;
        } else if (std.mem.eql(u8, cmd, "user")) {
            const args = if (cmd_end < s.len) s[cmd_end + 1 ..] else "";
            try self.handleUserCommand(args);
            return true;
        } else if (std.mem.eql(u8, cmd, "provider")) {
            const args = if (cmd_end < s.len) s[cmd_end + 1 ..] else "";
            try self.handleProviderCommand(args);
            return true;
        } else if (std.mem.eql(u8, cmd, "exit") or std.mem.eql(u8, cmd, "quit")) {
            std.debug.print("Goodbye.\n", .{});
            return false;
        } else {
            std.debug.print("Unknown command: {s}\n", .{cmd});
            std.debug.print("Type 'help' for available commands.\n", .{});
            return true;
        }
    }

    fn handleProviderCommand(self: *CLI, args_raw: []const u8) !void {
        var args_it = std.mem.tokenizeAny(u8, args_raw, " \t");
        const action = args_it.next() orelse {
            self.printProviderHelp();
            return;
        };

        if (std.mem.eql(u8, action, "add")) {
            const name = args_it.next() orelse {
                std.debug.print("Usage: provider add <name> <slug> <social_media|email|code_hosting|productivity|communication|payments|storage|other> <homepage_url> <api_base_url>\n", .{});
                return;
            };
            const slug = args_it.next() orelse {
                std.debug.print("Usage: provider add <name> <slug> <social_media|email|code_hosting|productivity|communication|payments|storage|other> <homepage_url> <api_base_url>\n", .{});
                return;
            };
            const category_str = args_it.next() orelse {
                std.debug.print("Usage: provider add <name> <slug> <social_media|email|code_hosting|productivity|communication|payments|storage|other> <homepage_url> <api_base_url>\n", .{});
                return;
            };
            const homepage_url = args_it.next() orelse {
                std.debug.print("Usage: provider add <name> <slug> <social_media|email|code_hosting|productivity|communication|payments|storage|other> <homepage_url> <api_base_url>\n", .{});
                return;
            };
            const api_base_url = args_it.next() orelse {
                std.debug.print("Usage: provider add <name> <slug> <social_media|email|code_hosting|productivity|communication|payments|storage|other> <homepage_url> <api_base_url>\n", .{});
                return;
            };
            const category = providers_module.parseProviderCategory(category_str) orelse {
                std.debug.print("Invalid provider category: {s}\n", .{category_str});
                return;
            };

            const provider_id = self.system.addProvider(name, slug, category, homepage_url, api_base_url) catch |err| {
                self.printProviderError(err);
                return;
            };
            std.debug.print("Provider created: id={} slug={s} category={s}\n", .{ provider_id, slug, @tagName(category) });
            return;
        }

        if (std.mem.eql(u8, action, "list")) {
            const providers = self.system.getProviders();
            if (providers.len == 0) {
                std.debug.print("No providers registered.\n", .{});
                return;
            }
            for (providers) |provider| {
                std.debug.print(
                    "id={} name={s} slug={s} category={s} status={s}\n",
                    .{ provider.id, provider.name, provider.slug, @tagName(provider.category), if (provider.active) "active" else "disabled" },
                );
            }
            return;
        }

        if (std.mem.eql(u8, action, "show")) {
            const id_str = args_it.next() orelse {
                std.debug.print("Usage: provider show <provider_id>\n", .{});
                return;
            };
            const provider_id = std.fmt.parseInt(u32, id_str, 10) catch {
                std.debug.print("Invalid provider id: {s}\n", .{id_str});
                return;
            };
            const provider = self.system.getProvider(provider_id) catch |err| {
                self.printProviderError(err);
                return;
            };
            std.debug.print("id={} name={s} slug={s} category={s}\n", .{ provider.id, provider.name, provider.slug, @tagName(provider.category) });
            std.debug.print("homepage={s}\n", .{provider.homepage_url});
            std.debug.print("api_base={s}\n", .{provider.api_base_url});
            std.debug.print("status={s}\n", .{if (provider.active) "active" else "disabled"});
            return;
        }

        if (std.mem.eql(u8, action, "disable")) {
            const id_str = args_it.next() orelse {
                std.debug.print("Usage: provider disable <provider_id>\n", .{});
                return;
            };
            const provider_id = std.fmt.parseInt(u32, id_str, 10) catch {
                std.debug.print("Invalid provider id: {s}\n", .{id_str});
                return;
            };
            self.system.disableProvider(provider_id) catch |err| {
                self.printProviderError(err);
                return;
            };
            std.debug.print("Provider {} disabled.\n", .{provider_id});
            return;
        }

        if (std.mem.eql(u8, action, "enable")) {
            const id_str = args_it.next() orelse {
                std.debug.print("Usage: provider enable <provider_id>\n", .{});
                return;
            };
            const provider_id = std.fmt.parseInt(u32, id_str, 10) catch {
                std.debug.print("Invalid provider id: {s}\n", .{id_str});
                return;
            };
            self.system.enableProvider(provider_id) catch |err| {
                self.printProviderError(err);
                return;
            };
            std.debug.print("Provider {} enabled.\n", .{provider_id});
            return;
        }

        self.printProviderHelp();
    }

    fn printProviderHelp(self: *CLI) void {
        _ = self;
        std.debug.print("Provider commands:\n", .{});
        std.debug.print("  provider add <name> <slug> <social_media|email|code_hosting|productivity|communication|payments|storage|other> <homepage_url> <api_base_url>\n", .{});
        std.debug.print("  provider list\n", .{});
        std.debug.print("  provider show <provider_id>\n", .{});
        std.debug.print("  provider disable <provider_id>\n", .{});
        std.debug.print("  provider enable <provider_id>\n", .{});
    }

    fn printProviderError(self: *CLI, err: anyerror) void {
        _ = self;
        switch (err) {
            providers_module.ProviderError.ProviderNotFound => std.debug.print("Error: provider not found.\n", .{}),
            providers_module.ProviderError.ProviderSlugTaken => std.debug.print("Error: provider slug already exists.\n", .{}),
            else => std.debug.print("Error: {s}\n", .{@errorName(err)}),
        }
    }

    fn handleUserCommand(self: *CLI, args_raw: []const u8) !void {
        var args_it = std.mem.tokenizeAny(u8, args_raw, " \t");
        const subcmd = args_it.next() orelse {
            self.printUserHelp();
            return;
        };

        if (std.mem.eql(u8, subcmd, "add")) {
            const username = args_it.next() orelse {
                std.debug.print("Usage: user add <username> <email> <password> [admin|worker|viewer]\n", .{});
                return;
            };
            const email = args_it.next() orelse {
                std.debug.print("Usage: user add <username> <email> <password> [admin|worker|viewer]\n", .{});
                return;
            };
            const password = args_it.next() orelse {
                std.debug.print("Usage: user add <username> <email> <password> [admin|worker|viewer]\n", .{});
                return;
            };
            const role = if (args_it.next()) |role_str|
                users_module.parseRole(role_str) orelse .worker
            else
                users_module.UserRole.worker;

            const user_id = self.system.createUser(username, email, password, role) catch |err| {
                self.printUserError(err);
                return;
            };
            std.debug.print("User created: id={} username={s} role={s}\n", .{ user_id, username, @tagName(role) });
            return;
        }

        if (std.mem.eql(u8, subcmd, "list")) {
            const users = self.system.getUsers();
            if (users.len == 0) {
                std.debug.print("No users created.\n", .{});
                return;
            }
            for (users) |user| {
                std.debug.print(
                    "id={} username={s} email={s} role={s} status={s}\n",
                    .{ user.id, user.username, user.email, @tagName(user.role), if (user.active) "active" else "disabled" },
                );
            }
            return;
        }

        if (std.mem.eql(u8, subcmd, "disable")) {
            const id_str = args_it.next() orelse {
                std.debug.print("Usage: user disable <user_id>\n", .{});
                return;
            };
            const user_id = std.fmt.parseInt(u32, id_str, 10) catch {
                std.debug.print("Invalid user id: {s}\n", .{id_str});
                return;
            };
            self.system.disableUser(user_id) catch |err| {
                self.printUserError(err);
                return;
            };
            std.debug.print("User {} disabled.\n", .{user_id});
            return;
        }

        if (std.mem.eql(u8, subcmd, "login")) {
            const username = args_it.next() orelse {
                std.debug.print("Usage: user login <username> <password>\n", .{});
                return;
            };
            const password = args_it.next() orelse {
                std.debug.print("Usage: user login <username> <password>\n", .{});
                return;
            };
            const user_id = self.system.loginUser(username, password) catch |err| {
                self.printUserError(err);
                return;
            };
            std.debug.print("Logged in as {s} (id={}).\n", .{ username, user_id });
            return;
        }

        if (std.mem.eql(u8, subcmd, "logout")) {
            self.system.logoutUser() catch |err| {
                self.printUserError(err);
                return;
            };
            std.debug.print("Logged out.\n", .{});
            return;
        }

        if (std.mem.eql(u8, subcmd, "whoami")) {
            if (self.system.getCurrentUser()) |user| {
                std.debug.print(
                    "Current user: id={} username={s} role={s}\n",
                    .{ user.id, user.username, @tagName(user.role) },
                );
            } else {
                std.debug.print("No active user session.\n", .{});
            }
            return;
        }

        if (std.mem.eql(u8, subcmd, "creds")) {
            const action = args_it.next() orelse {
                std.debug.print("Usage: user creds <add|list|disable|setpass> ...\n", .{});
                return;
            };

            if (std.mem.eql(u8, action, "add")) {
                const user_id = self.parseUserId(args_it.next()) orelse return;
                const label = args_it.next() orelse {
                    std.debug.print("Usage: user creds add <user_id> <label> <password|api_token|ssh_key> <secret>\n", .{});
                    return;
                };
                const kind_str = args_it.next() orelse {
                    std.debug.print("Usage: user creds add <user_id> <label> <password|api_token|ssh_key> <secret>\n", .{});
                    return;
                };
                const secret = args_it.next() orelse {
                    std.debug.print("Usage: user creds add <user_id> <label> <password|api_token|ssh_key> <secret>\n", .{});
                    return;
                };
                const kind = users_module.parseCredentialKind(kind_str) orelse {
                    std.debug.print("Invalid credential kind: {s}\n", .{kind_str});
                    return;
                };
                const cred_id = self.system.addUserCredential(user_id, label, kind, secret) catch |err| {
                    self.printUserError(err);
                    return;
                };
                std.debug.print("Credential added: id={} user={} label={s} kind={s}\n", .{ cred_id, user_id, label, @tagName(kind) });
                return;
            }

            if (std.mem.eql(u8, action, "list")) {
                const user_id = self.parseUserId(args_it.next()) orelse return;
                const creds = self.system.getUserCredentials();
                var found = false;
                for (creds) |cred| {
                    if (cred.user_id == user_id) {
                        found = true;
                        std.debug.print(
                            "id={} user={} label={s} kind={s} status={s}\n",
                            .{ cred.id, cred.user_id, cred.label, @tagName(cred.kind), if (cred.active) "active" else "disabled" },
                        );
                    }
                }
                if (!found) std.debug.print("No credentials found for user {}.\n", .{user_id});
                return;
            }

            if (std.mem.eql(u8, action, "disable")) {
                const cred_id_str = args_it.next() orelse {
                    std.debug.print("Usage: user creds disable <credential_id>\n", .{});
                    return;
                };
                const cred_id = std.fmt.parseInt(u32, cred_id_str, 10) catch {
                    std.debug.print("Invalid credential id: {s}\n", .{cred_id_str});
                    return;
                };
                self.system.disableUserCredential(cred_id) catch |err| {
                    self.printUserError(err);
                    return;
                };
                std.debug.print("Credential {} disabled.\n", .{cred_id});
                return;
            }

            if (std.mem.eql(u8, action, "setpass")) {
                const user_id = self.parseUserId(args_it.next()) orelse return;
                const password = args_it.next() orelse {
                    std.debug.print("Usage: user creds setpass <user_id> <password>\n", .{});
                    return;
                };
                self.system.setUserPassword(user_id, password) catch |err| {
                    self.printUserError(err);
                    return;
                };
                std.debug.print("Password updated for user {}.\n", .{user_id});
                return;
            }

            std.debug.print("Usage: user creds <add|list|disable|setpass> ...\n", .{});
            return;
        }

        if (std.mem.eql(u8, subcmd, "profile")) {
            const action = args_it.next() orelse {
                std.debug.print("Usage: user profile <create|list|activate|show|set|config|pref|account> ...\n", .{});
                return;
            };

            if (std.mem.eql(u8, action, "create")) {
                const user_id = self.parseUserId(args_it.next()) orelse return;
                const type_str = args_it.next() orelse {
                    std.debug.print("Usage: user profile create <user_id> <work|personal|custom> <name> <description>\n", .{});
                    return;
                };
                const name = args_it.next() orelse {
                    std.debug.print("Usage: user profile create <user_id> <work|personal|custom> <name> <description>\n", .{});
                    return;
                };
                const description = args_it.next() orelse {
                    std.debug.print("Usage: user profile create <user_id> <work|personal|custom> <name> <description>\n", .{});
                    return;
                };
                const ptype = users_module.parseProfileType(type_str) orelse {
                    std.debug.print("Invalid profile type: {s}\n", .{type_str});
                    return;
                };
                const profile_id = self.system.createUserProfile(user_id, ptype, name, description) catch |err| {
                    self.printUserError(err);
                    return;
                };
                std.debug.print("Profile created: id={} user={} type={s} name={s}\n", .{ profile_id, user_id, @tagName(ptype), name });
                return;
            }

            if (std.mem.eql(u8, action, "list")) {
                const user_id = self.parseUserId(args_it.next()) orelse return;
                const profiles = self.system.getUserProfiles();
                var found = false;
                for (profiles) |profile| {
                    if (profile.user_id == user_id) {
                        found = true;
                        std.debug.print(
                            "id={} type={s} name={s} status={s} accounts={}\n",
                            .{ profile.id, @tagName(profile.profile_type), profile.name, if (profile.active) "active" else "inactive", profile.account_ids.items.len },
                        );
                    }
                }
                if (!found) std.debug.print("No profiles found for user {}.\n", .{user_id});
                return;
            }

            if (std.mem.eql(u8, action, "activate")) {
                const user_id = self.parseUserId(args_it.next()) orelse return;
                const profile_id_str = args_it.next() orelse {
                    std.debug.print("Usage: user profile activate <user_id> <profile_id>\n", .{});
                    return;
                };
                const profile_id = std.fmt.parseInt(u32, profile_id_str, 10) catch {
                    std.debug.print("Invalid profile id: {s}\n", .{profile_id_str});
                    return;
                };
                self.system.setActiveUserProfile(user_id, profile_id) catch |err| {
                    self.printUserError(err);
                    return;
                };
                std.debug.print("Profile {} activated for user {}.\n", .{ profile_id, user_id });
                return;
            }

            if (std.mem.eql(u8, action, "set")) {
                const user_id = self.parseUserId(args_it.next()) orelse return;
                const display_name = args_it.next() orelse {
                    std.debug.print("Usage: user profile set <user_id> <display_name> <bio> <timezone> <locale> <avatar_url>\n", .{});
                    return;
                };
                const bio = args_it.next() orelse {
                    std.debug.print("Usage: user profile set <user_id> <display_name> <bio> <timezone> <locale> <avatar_url>\n", .{});
                    return;
                };
                const timezone = args_it.next() orelse {
                    std.debug.print("Usage: user profile set <user_id> <display_name> <bio> <timezone> <locale> <avatar_url>\n", .{});
                    return;
                };
                const locale = args_it.next() orelse {
                    std.debug.print("Usage: user profile set <user_id> <display_name> <bio> <timezone> <locale> <avatar_url>\n", .{});
                    return;
                };
                const avatar_url = args_it.next() orelse {
                    std.debug.print("Usage: user profile set <user_id> <display_name> <bio> <timezone> <locale> <avatar_url>\n", .{});
                    return;
                };

                self.system.setUserProfile(user_id, display_name, bio, timezone, locale, avatar_url) catch |err| {
                    self.printUserError(err);
                    return;
                };
                std.debug.print("Profile updated for user {}.\n", .{user_id});
                return;
            }

            if (std.mem.eql(u8, action, "show")) {
                const user_id = self.parseUserId(args_it.next()) orelse return;
                const profile = self.system.getUserProfile(user_id) catch |err| {
                    self.printUserError(err);
                    return;
                };
                std.debug.print("profile={} user={} type={s} name={s}\n", .{ profile.id, profile.user_id, @tagName(profile.profile_type), profile.name });
                std.debug.print("description={s}\n", .{profile.description});
                if (profile.configurations.items.len > 0) {
                    std.debug.print("configurations:\n", .{});
                    for (profile.configurations.items) |entry| {
                        std.debug.print("  {s}={s}\n", .{ entry.key, entry.value });
                    }
                }
                if (profile.preferences.items.len > 0) {
                    std.debug.print("preferences:\n", .{});
                    for (profile.preferences.items) |entry| {
                        std.debug.print("  {s}={s}\n", .{ entry.key, entry.value });
                    }
                }
                std.debug.print("linked_accounts={}\n", .{profile.account_ids.items.len});
                return;
            }

            if (std.mem.eql(u8, action, "config")) {
                const op = args_it.next() orelse {
                    std.debug.print("Usage: user profile config set <profile_id> <key> <value>\n", .{});
                    return;
                };
                if (!std.mem.eql(u8, op, "set")) {
                    std.debug.print("Usage: user profile config set <profile_id> <key> <value>\n", .{});
                    return;
                }
                const profile_id_str = args_it.next() orelse {
                    std.debug.print("Usage: user profile config set <profile_id> <key> <value>\n", .{});
                    return;
                };
                const key = args_it.next() orelse {
                    std.debug.print("Usage: user profile config set <profile_id> <key> <value>\n", .{});
                    return;
                };
                const value = args_it.next() orelse {
                    std.debug.print("Usage: user profile config set <profile_id> <key> <value>\n", .{});
                    return;
                };
                const profile_id = std.fmt.parseInt(u32, profile_id_str, 10) catch {
                    std.debug.print("Invalid profile id: {s}\n", .{profile_id_str});
                    return;
                };
                self.system.setUserProfileConfiguration(profile_id, key, value) catch |err| {
                    self.printUserError(err);
                    return;
                };
                std.debug.print("Profile config set: profile={} {s}={s}\n", .{ profile_id, key, value });
                return;
            }

            if (std.mem.eql(u8, action, "pref")) {
                const op = args_it.next() orelse {
                    std.debug.print("Usage: user profile pref set <profile_id> <key> <value>\n", .{});
                    return;
                };
                if (!std.mem.eql(u8, op, "set")) {
                    std.debug.print("Usage: user profile pref set <profile_id> <key> <value>\n", .{});
                    return;
                }
                const profile_id_str = args_it.next() orelse {
                    std.debug.print("Usage: user profile pref set <profile_id> <key> <value>\n", .{});
                    return;
                };
                const key = args_it.next() orelse {
                    std.debug.print("Usage: user profile pref set <profile_id> <key> <value>\n", .{});
                    return;
                };
                const value = args_it.next() orelse {
                    std.debug.print("Usage: user profile pref set <profile_id> <key> <value>\n", .{});
                    return;
                };
                const profile_id = std.fmt.parseInt(u32, profile_id_str, 10) catch {
                    std.debug.print("Invalid profile id: {s}\n", .{profile_id_str});
                    return;
                };
                self.system.setUserProfilePreference(profile_id, key, value) catch |err| {
                    self.printUserError(err);
                    return;
                };
                std.debug.print("Profile preference set: profile={} {s}={s}\n", .{ profile_id, key, value });
                return;
            }

            if (std.mem.eql(u8, action, "account")) {
                const op = args_it.next() orelse {
                    std.debug.print("Usage: user profile account <add|list|disable> ...\n", .{});
                    return;
                };

                if (std.mem.eql(u8, op, "add")) {
                    const profile_id_str = args_it.next() orelse {
                        std.debug.print("Usage: user profile account add <profile_id> <social_media|work|personal|email|software|other> <username> <provider> <details>\n", .{});
                        return;
                    };
                    const account_type_str = args_it.next() orelse {
                        std.debug.print("Usage: user profile account add <profile_id> <social_media|work|personal|email|software|other> <username> <provider> <details>\n", .{});
                        return;
                    };
                    const username = args_it.next() orelse {
                        std.debug.print("Usage: user profile account add <profile_id> <social_media|work|personal|email|software|other> <username> <provider> <details>\n", .{});
                        return;
                    };
                    const provider = args_it.next() orelse {
                        std.debug.print("Usage: user profile account add <profile_id> <social_media|work|personal|email|software|other> <username> <provider> <details>\n", .{});
                        return;
                    };
                    const details = args_it.next() orelse {
                        std.debug.print("Usage: user profile account add <profile_id> <social_media|work|personal|email|software|other> <username> <provider> <details>\n", .{});
                        return;
                    };
                    const profile_id = std.fmt.parseInt(u32, profile_id_str, 10) catch {
                        std.debug.print("Invalid profile id: {s}\n", .{profile_id_str});
                        return;
                    };
                    const account_type = users_module.parseAccountType(account_type_str) orelse {
                        std.debug.print("Invalid account type: {s}\n", .{account_type_str});
                        return;
                    };
                    const account_id = self.system.addUserProfileAccount(profile_id, account_type, username, provider, details) catch |err| {
                        self.printUserError(err);
                        return;
                    };
                    std.debug.print(
                        "Profile account added: id={} profile={} type={s} provider={s}\n",
                        .{ account_id, profile_id, @tagName(account_type), provider },
                    );
                    return;
                }

                if (std.mem.eql(u8, op, "list")) {
                    const profile_id_str = args_it.next() orelse {
                        std.debug.print("Usage: user profile account list <profile_id>\n", .{});
                        return;
                    };
                    const profile_id = std.fmt.parseInt(u32, profile_id_str, 10) catch {
                        std.debug.print("Invalid profile id: {s}\n", .{profile_id_str});
                        return;
                    };
                    const accounts = self.system.getUserProfileAccounts();
                    var found = false;
                    for (accounts) |acct| {
                        if (acct.profile_id == profile_id) {
                            found = true;
                            std.debug.print(
                                "id={} profile={} type={s} username={s} provider={s} status={s}\n",
                                .{ acct.id, acct.profile_id, @tagName(acct.account_type), acct.username, acct.provider, if (acct.active) "active" else "disabled" },
                            );
                        }
                    }
                    if (!found) std.debug.print("No accounts linked to profile {}.\n", .{profile_id});
                    return;
                }

                if (std.mem.eql(u8, op, "disable")) {
                    const account_id_str = args_it.next() orelse {
                        std.debug.print("Usage: user profile account disable <account_id>\n", .{});
                        return;
                    };
                    const account_id = std.fmt.parseInt(u32, account_id_str, 10) catch {
                        std.debug.print("Invalid account id: {s}\n", .{account_id_str});
                        return;
                    };
                    self.system.disableUserProfileAccount(account_id) catch |err| {
                        self.printUserError(err);
                        return;
                    };
                    std.debug.print("Profile account {} disabled.\n", .{account_id});
                    return;
                }

                std.debug.print("Usage: user profile account <add|list|disable> ...\n", .{});
                return;
            }

            std.debug.print("Usage: user profile <create|list|activate|show|set|config|pref|account> ...\n", .{});
            return;
        }

        if (std.mem.eql(u8, subcmd, "persona")) {
            const action = args_it.next() orelse {
                std.debug.print("Usage: user persona <add|list|activate|active> ...\n", .{});
                return;
            };

            if (std.mem.eql(u8, action, "add")) {
                const user_id = self.parseUserId(args_it.next()) orelse return;
                const name = args_it.next() orelse {
                    std.debug.print("Usage: user persona add <user_id> <name> <focus_area> <tone> <description>\n", .{});
                    return;
                };
                const focus_area = args_it.next() orelse {
                    std.debug.print("Usage: user persona add <user_id> <name> <focus_area> <tone> <description>\n", .{});
                    return;
                };
                const tone = args_it.next() orelse {
                    std.debug.print("Usage: user persona add <user_id> <name> <focus_area> <tone> <description>\n", .{});
                    return;
                };
                const description = args_it.next() orelse {
                    std.debug.print("Usage: user persona add <user_id> <name> <focus_area> <tone> <description>\n", .{});
                    return;
                };
                const persona_id = self.system.addUserPersona(user_id, name, description, focus_area, tone) catch |err| {
                    self.printUserError(err);
                    return;
                };
                std.debug.print("Persona added: id={} user={} name={s}\n", .{ persona_id, user_id, name });
                return;
            }

            if (std.mem.eql(u8, action, "list")) {
                const user_id = self.parseUserId(args_it.next()) orelse return;
                const personas = self.system.getUserPersonas();
                var found = false;
                for (personas) |persona| {
                    if (persona.user_id == user_id) {
                        found = true;
                        std.debug.print(
                            "id={} user={} name={s} focus={s} tone={s} status={s}\n",
                            .{ persona.id, persona.user_id, persona.name, persona.focus_area, persona.tone, if (persona.active) "active" else "inactive" },
                        );
                    }
                }
                if (!found) std.debug.print("No personas found for user {}.\n", .{user_id});
                return;
            }

            if (std.mem.eql(u8, action, "activate")) {
                const user_id = self.parseUserId(args_it.next()) orelse return;
                const persona_id_str = args_it.next() orelse {
                    std.debug.print("Usage: user persona activate <user_id> <persona_id>\n", .{});
                    return;
                };
                const persona_id = std.fmt.parseInt(u32, persona_id_str, 10) catch {
                    std.debug.print("Invalid persona id: {s}\n", .{persona_id_str});
                    return;
                };
                self.system.activateUserPersona(user_id, persona_id) catch |err| {
                    self.printUserError(err);
                    return;
                };
                std.debug.print("Persona {} activated for user {}.\n", .{ persona_id, user_id });
                return;
            }

            if (std.mem.eql(u8, action, "active")) {
                const user_id = self.parseUserId(args_it.next()) orelse return;
                if (self.system.getUserActivePersona(user_id)) |persona| {
                    std.debug.print(
                        "Active persona: id={} name={s} focus={s} tone={s}\n",
                        .{ persona.id, persona.name, persona.focus_area, persona.tone },
                    );
                } else {
                    std.debug.print("No active persona for user {}.\n", .{user_id});
                }
                return;
            }

            std.debug.print("Usage: user persona <add|list|activate|active> ...\n", .{});
            return;
        }

        self.printUserHelp();
    }

    fn printUserHelp(self: *CLI) void {
        _ = self;
        std.debug.print("User commands:\n", .{});
        std.debug.print("  user add <username> <email> <password> [admin|worker|viewer]\n", .{});
        std.debug.print("  user list\n", .{});
        std.debug.print("  user disable <user_id>\n", .{});
        std.debug.print("  user login <username> <password>\n", .{});
        std.debug.print("  user logout\n", .{});
        std.debug.print("  user whoami\n", .{});
        std.debug.print("  user creds add <user_id> <label> <password|api_token|ssh_key> <secret>\n", .{});
        std.debug.print("  user creds list <user_id>\n", .{});
        std.debug.print("  user creds disable <credential_id>\n", .{});
        std.debug.print("  user creds setpass <user_id> <password>\n", .{});
        std.debug.print("  user profile create <user_id> <work|personal|custom> <name> <description>\n", .{});
        std.debug.print("  user profile list <user_id>\n", .{});
        std.debug.print("  user profile activate <user_id> <profile_id>\n", .{});
        std.debug.print("  user profile set <user_id> <display_name> <bio> <timezone> <locale> <avatar_url>\n", .{});
        std.debug.print("  user profile show <user_id>\n", .{});
        std.debug.print("  user profile config set <profile_id> <key> <value>\n", .{});
        std.debug.print("  user profile pref set <profile_id> <key> <value>\n", .{});
        std.debug.print("  user profile account add <profile_id> <social_media|work|personal|email|software|other> <username> <provider> <details>\n", .{});
        std.debug.print("  user profile account list <profile_id>\n", .{});
        std.debug.print("  user profile account disable <account_id>\n", .{});
        std.debug.print("  user persona add <user_id> <name> <focus_area> <tone> <description>\n", .{});
        std.debug.print("  user persona list <user_id>\n", .{});
        std.debug.print("  user persona activate <user_id> <persona_id>\n", .{});
        std.debug.print("  user persona active <user_id>\n", .{});
    }

    fn parseUserId(self: *CLI, id_opt: ?[]const u8) ?u32 {
        _ = self;
        const id_str = id_opt orelse {
            std.debug.print("Missing user id.\n", .{});
            return null;
        };
        return std.fmt.parseInt(u32, id_str, 10) catch {
            std.debug.print("Invalid user id: {s}\n", .{id_str});
            return null;
        };
    }

    fn printUserError(self: *CLI, err: anyerror) void {
        _ = self;
        switch (err) {
            users_module.UserError.UserNotFound => std.debug.print("Error: user not found.\n", .{}),
            users_module.UserError.UsernameTaken => std.debug.print("Error: username already exists.\n", .{}),
            users_module.UserError.InvalidCredentials => std.debug.print("Error: invalid credentials.\n", .{}),
            users_module.UserError.UserDisabled => std.debug.print("Error: user is disabled.\n", .{}),
            users_module.UserError.AlreadyLoggedIn => std.debug.print("Error: another user is already logged in.\n", .{}),
            users_module.UserError.NotLoggedIn => std.debug.print("Error: no active login session.\n", .{}),
            users_module.UserError.CredentialNotFound => std.debug.print("Error: credential not found.\n", .{}),
            users_module.UserError.ProfileNotFound => std.debug.print("Error: profile not found.\n", .{}),
            users_module.UserError.ProfileOwnershipMismatch => std.debug.print("Error: profile does not belong to that user.\n", .{}),
            users_module.UserError.ProfileAccountNotFound => std.debug.print("Error: profile account not found.\n", .{}),
            users_module.UserError.PersonaNotFound => std.debug.print("Error: persona not found.\n", .{}),
            users_module.UserError.PersonaOwnershipMismatch => std.debug.print("Error: persona does not belong to that user.\n", .{}),
            else => std.debug.print("Error: {s}\n", .{@errorName(err)}),
        }
    }

    /// Display system statistics
    fn handleViewStatistics(self: *CLI) void {
        std.debug.print("\n╔════════════════════════════════════════════╗\n", .{});
        std.debug.print("║       KOGI SYSTEM STATISTICS               ║\n", .{});
        std.debug.print("╚════════════════════════════════════════════╝\n\n", .{});
        std.debug.print("Active Identities: {}\n", .{self.system.getActiveIdentitiesCount()});
        std.debug.print("Active Tasks: {}\n", .{self.system.getActiveTasksCount()});
        std.debug.print("Total Engagements: {}\n", .{self.system.engagements.items.len});
        // Collections and items managed through workspace_manager
        const collection_count = if (self.system.workspace_manager.workspace) |p| p.collections.items.len else 0;
        const item_count = if (self.system.workspace_manager.workspace) |p| p.items.items.len else 0;
        std.debug.print("Workspace Collections: {}\n", .{collection_count});
        std.debug.print("Workspace Items: {}\n", .{item_count});
        std.debug.print("Directory Organizations: {}\n", .{self.system.directory.getOrganizationCount()});
        std.debug.print("Vault Total Value: ${:.2}\n", .{self.system.vault.getTotalValue()});
        std.debug.print("Audit Log Entries: {}\n", .{self.system.security_manager.getAuditLog().len});
        std.debug.print("Active Sessions: {}\n", .{self.system.security_manager.sessions.items.len});
    }

    /// List all identities
    fn handleListIdentities(self: *CLI) void {
        std.debug.print("\n╔════════════════════════════════════════════╗\n", .{});
        std.debug.print("║           SYSTEM IDENTITIES                ║\n", .{});
        std.debug.print("╚════════════════════════════════════════════╝\n\n", .{});

        const identities = self.system.getIdentities();
        if (identities.len == 0) {
            std.debug.print("No identities created.\n", .{});
            return;
        }

        for (identities, 0..) |identity, idx| {
            std.debug.print("\n[Identity {}] {s}\n", .{ idx, identity.name });
            std.debug.print("  Email: {s}\n", .{identity.email});
            std.debug.print("  Hourly Rate: ${:.2}\n", .{identity.hourly_rate});
            std.debug.print("  Status: {s}\n", .{if (identity.active) "Active" else "Inactive"});

            if (identity.skills.items.len > 0) {
                std.debug.print("  Skills: ", .{});
                for (identity.skills.items, 0..) |skill, skill_idx| {
                    if (skill_idx > 0) std.debug.print(", ", .{});
                    std.debug.print("{s}", .{skill});
                }
                std.debug.print("\n", .{});
            }
        }
    }

    /// List all tasks
    fn handleListTasks(self: *CLI) void {
        std.debug.print("\n╔════════════════════════════════════════════╗\n", .{});
        std.debug.print("║           ACTIVE TASK POSTINGS             ║\n", .{});
        std.debug.print("╚════════════════════════════════════════════╝\n\n", .{});

        const tasks = self.system.getTasks();
        if (tasks.len == 0) {
            std.debug.print("No tasks posted.\n", .{});
            return;
        }

        for (tasks, 0..) |task, idx| {
            std.debug.print("\n[Task {}] {s}\n", .{ idx, task.title });
            std.debug.print("  Description: {s}\n", .{task.description});
            std.debug.print("  Budget: ${:.2}\n", .{task.budget});
            std.debug.print("  Status: {s}\n", .{if (task.completed) "Completed" else "Open"});

            if (task.assigned_to) |identity_id| {
                std.debug.print("  Assigned to: Identity {}\n", .{identity_id});
            } else {
                std.debug.print("  Assigned to: Unassigned\n", .{});
            }

            if (task.required_skills.items.len > 0) {
                std.debug.print("  Required Skills: ", .{});
                for (task.required_skills.items, 0..) |skill, skill_idx| {
                    if (skill_idx > 0) std.debug.print(", ", .{});
                    std.debug.print("{s}", .{skill});
                }
                std.debug.print("\n", .{});
            }
        }
    }

    /// Run a demo of the KOGI OS with sample data
    pub fn runDemo(self: *CLI) !void {
        std.debug.print("\n╔════════════════════════════════════════════╗\n", .{});
        std.debug.print("║   KOGI - Operating System for Workers      ║\n", .{});
        std.debug.print("║          Running Demo Mode                 ║\n", .{});
        std.debug.print("╚════════════════════════════════════════════╝\n", .{});

        // Create sample identities
        std.debug.print("\n📝 Creating system identities...\n", .{});
        const identity1_id = try self.system.createIdentity("Alice Johnson", "alice@example.com", 75.50, identity_module.IdentityType.developer);
        try self.system.addIdentitySkill(identity1_id, "Zig Programming");
        try self.system.addIdentitySkill(identity1_id, "System Design");

        const identity2_id = try self.system.createIdentity("Bob Smith", "bob@example.com", 65.00, identity_module.IdentityType.developer);
        try self.system.addIdentitySkill(identity2_id, "Zig Programming");
        try self.system.addIdentitySkill(identity2_id, "Testing");

        const identity3_id = try self.system.createIdentity("Carol White", "carol@example.com", 85.75, identity_module.IdentityType.consultant);
        try self.system.addIdentitySkill(identity3_id, "Project Management");
        try self.system.addIdentitySkill(identity3_id, "Documentation");

        std.debug.print("✓ Created 3 identities\n", .{});

        // Create sample tasks
        std.debug.print("\n📝 Posting tasks to system...\n", .{});
        const task1_id = try self.system.postTask(
            "Build REST API Server",
            "Create a high-performance REST API server in Zig",
            5000.00,
            1746000000,
        );
        try self.system.addTaskSkillRequirement(task1_id, "Zig Programming");
        try self.system.addTaskSkillRequirement(task1_id, "System Design");

        const task2_id = try self.system.postTask(
            "Write Unit Tests",
            "Comprehensive test suite for core modules",
            2000.00,
            1745000000,
        );
        try self.system.addTaskSkillRequirement(task2_id, "Zig Programming");
        try self.system.addTaskSkillRequirement(task2_id, "Testing");

        const task3_id = try self.system.postTask(
            "Project Documentation",
            "Create complete API documentation and user guides",
            1500.00,
            1744000000,
        );
        try self.system.addTaskSkillRequirement(task3_id, "Documentation");

        std.debug.print("✓ Posted 3 tasks\n", .{});

        // Create engagements
        std.debug.print("\n📝 Creating engagements...\n", .{});
        const engagement1_id = try self.system.createEngagement(identity1_id, task1_id, 1735000000, 75.50);
        try self.system.logHours(engagement1_id, 32.5);

        const engagement2_id = try self.system.createEngagement(identity2_id, task2_id, 1735000000, 65.00);
        try self.system.logHours(engagement2_id, 24.0);

        const engagement3_id = try self.system.createEngagement(identity3_id, task3_id, 1735000000, 85.75);
        try self.system.logHours(engagement3_id, 18.5);

        std.debug.print("✓ Created 3 engagements\n", .{});

        // Display statistics and listings
        self.handleViewStatistics();
        self.handleListIdentities();
        self.handleListTasks();

        // Display earnings information
        std.debug.print("\n╔════════════════════════════════════════════╗\n", .{});
        std.debug.print("║          IDENTITY EARNINGS REPORT          ║\n", .{});
        std.debug.print("╚════════════════════════════════════════════╝\n\n", .{});

        const identities = self.system.getIdentities();
        for (identities) |identity| {
            const earnings = self.system.calculateIdentityEarnings(identity.id);
            std.debug.print("{s}: ${:.2}\n", .{ identity.name, earnings });
            const conn_count = self.system.getIdentityActiveConnectionCount(identity.id);
            std.debug.print("  Active Connections: {}\n", .{conn_count});
        }

        std.debug.print("\n✓ Demo completed successfully!\n", .{});

        // Demonstrate security features
        std.debug.print("\n╔════════════════════════════════════════════╗\n", .{});
        std.debug.print("║         SECURITY & ACCESS CONTROL          ║\n", .{});
        std.debug.print("╚════════════════════════════════════════════╝\n\n", .{});

        // Initialize default roles
        var default_roles = try security_module.initializeDefaultRoles(self.allocator);
        defer {
            for (default_roles.items) |role| {
                var perms = role.permissions;
                perms.deinit(self.allocator);
            }
            default_roles.deinit(self.allocator);
        }

        // Assign roles to identities
        try self.system.security_manager.assignRole(identity1_id, security_module.Role.worker, 0, null);
        try self.system.security_manager.assignRole(identity2_id, security_module.Role.contractor, 0, null);
        try self.system.security_manager.assignRole(identity3_id, security_module.Role.admin, 0, null);

        std.debug.print("Assigned roles:\n", .{});
        std.debug.print("  Alice Johnson: Worker\n", .{});
        std.debug.print("  Bob Smith: Contractor\n", .{});
        std.debug.print("  Carol White: Admin\n\n", .{});

        // Create sessions
        _ = try self.system.security_manager.createSession(identity1_id, "token_alice_123", "192.168.1.100", "device_fp_1");
        _ = try self.system.security_manager.createSession(identity2_id, "token_bob_456", "192.168.1.101", "device_fp_2");
        std.debug.print("Created 2 sessions\n", .{});

        // Test access control
        std.debug.print("\nAccess Control Examples:\n", .{});
        const can_alice_create_task = self.system.security_manager.hasPermission(identity1_id, security_module.Permission.create_task);
        const can_bob_create_task = self.system.security_manager.hasPermission(identity2_id, security_module.Permission.create_task);
        const can_carol_manage_users = self.system.security_manager.hasPermission(identity3_id, security_module.Permission.manage_users);

        std.debug.print("  Alice (worker) can create tasks: {}\n", .{can_alice_create_task});
        std.debug.print("  Bob (contractor) can create tasks: {}\n", .{can_bob_create_task});
        std.debug.print("  Carol (admin) can manage users: {}\n", .{can_carol_manage_users});

        // Display audit log
        std.debug.print("\n╔════════════════════════════════════════════╗\n", .{});
        std.debug.print("║              AUDIT LOG (Recent)            ║\n", .{});
        std.debug.print("╚════════════════════════════════════════════╝\n\n", .{});
        const audit_log = self.system.security_manager.getAuditLog();
        const start_idx = if (audit_log.len > 5) audit_log.len - 5 else 0;
        for (audit_log[start_idx..]) |entry| {
            std.debug.print("[{s}] Identity {}: {s}\n", .{ @tagName(entry.event_type), entry.identity_id, entry.action });
        }

        std.debug.print("\n✓ Security demonstration completed!\n\n", .{});

        // Also exercise standalone demos and subsystems to ensure
        // the CLI touches as many exported APIs as possible.
        std.debug.print("\n▶ Running additional module demos...\n", .{});
        // Portfolio subsystem demo
        portfolio_module.portfolioDemo();

        // CRM demo
        crm_module.crmDemo();

        // Assets demo
        assets_module.assetsDemo();

        // Accounts demo (some modules have lightweight demos)
        accounts_module.accountsDemo();

        // Contract management (kernel-like) runner
        _ = contract_module.runContractManagementSystem();

        std.debug.print("✓ Additional module demos executed.\n\n", .{});
    }
};

/// Start the KOGI OS CLI shell
pub fn startCLI(allocator: std.mem.Allocator, system: *system_module.System) !void {
    var cli = CLI.init(allocator, system);

    // Interactive shell
    var term = terminal_module.Terminal.init(allocator, "kogi> ");
    while (true) {
        try term.writePrompt();
        const line = try term.readLine();
        // handle command
        const keep = try cli.handleCommand(line);
        // free the allocated buffer returned by readLine
        allocator.free(line);
        if (!keep) break;
    }
}
