//! KOGI State Management System - Snapshots, checkpoints, backups, and restore
//! Provides persistent state management with checkpoint and recovery capabilities

const std = @import("std");

/// State checkpoint entry
pub const Checkpoint = struct {
    id: u32,
    timestamp: i64,
    state_hash: []const u8,
    checkpoint_data: []const u8,
    size_bytes: u64,
    metadata: std.StringHashMap([]const u8),
};

/// Backup information
pub const Backup = struct {
    id: u32,
    timestamp: i64,
    filename: []const u8,
    size_bytes: u64,
    checksum: []const u8,
    compression_type: CompressionType,
    retention_days: u32,
    created_at: i64,
};

/// Compression types for backups
pub const CompressionType = enum {
    none,
    gzip,
    zstd,
    lz4,
};

/// Restore point information
pub const RestorePoint = struct {
    id: u32,
    backup_id: u32,
    checkpoint_id: u32,
    description: []const u8,
    created_at: i64,
    can_restore: bool,
};

/// State management configuration
pub const StateManagerConfig = struct {
    auto_checkpoint_interval_seconds: u32 = 300, // 5 minutes
    auto_backup_interval_seconds: u32 = 3600, // 1 hour
    max_checkpoints: usize = 100,
    max_backups: usize = 50,
    backup_directory: []const u8 = "./backups",
    compression: CompressionType = .gzip,
};

/// State manager for checkpoints, backups, and restore
pub const StateManager = struct {
    allocator: std.mem.Allocator,
    config: StateManagerConfig,
    checkpoints: std.array_list.Managed(Checkpoint),
    backups: std.array_list.Managed(Backup),
    restore_points: std.array_list.Managed(RestorePoint),
    next_checkpoint_id: u32 = 0,
    next_backup_id: u32 = 0,
    next_restore_point_id: u32 = 0,
    last_checkpoint_time: i64 = 0,
    last_backup_time: i64 = 0,
    mutex: std.Thread.Mutex = .{},

    pub fn init(allocator: std.mem.Allocator, config: StateManagerConfig) !StateManager {
        return StateManager{
            .allocator = allocator,
            .config = config,
            .checkpoints = std.array_list.Managed(Checkpoint).init(allocator),
            .backups = std.array_list.Managed(Backup).init(allocator),
            .restore_points = std.array_list.Managed(RestorePoint).init(allocator),
        };
    }

    pub fn deinit(self: *StateManager) void {
        for (self.checkpoints.items) |cp| {
            self.allocator.free(cp.state_hash);
            self.allocator.free(cp.checkpoint_data);
            var metadata = cp.metadata;
            var iter = metadata.iterator();
            while (iter.next()) |kv| {
                self.allocator.free(kv.key_ptr.*);
                self.allocator.free(kv.value_ptr.*);
            }
            metadata.deinit();
        }
        self.checkpoints.deinit();

        for (self.backups.items) |backup| {
            self.allocator.free(backup.filename);
            self.allocator.free(backup.checksum);
        }
        self.backups.deinit();

        for (self.restore_points.items) |rp| {
            self.allocator.free(rp.description);
        }
        self.restore_points.deinit();
    }

    /// Create a checkpoint of current state
    pub fn createCheckpoint(
        self: *StateManager,
        state_data: []const u8,
        state_hash: []const u8,
    ) !u32 {
        self.mutex.lock();
        defer self.mutex.unlock();

        const checkpoint_id = self.next_checkpoint_id;
        self.next_checkpoint_id += 1;

        const checkpoint = Checkpoint{
            .id = checkpoint_id,
            .timestamp = std.time.timestamp(),
            .state_hash = try self.allocator.dupe(u8, state_hash),
            .checkpoint_data = try self.allocator.dupe(u8, state_data),
            .size_bytes = state_data.len,
            .metadata = std.StringHashMap([]const u8).init(self.allocator),
        };

        if (self.checkpoints.items.len >= self.config.max_checkpoints) {
            const old_cp = self.checkpoints.orderedRemove(0);
            self.allocator.free(old_cp.state_hash);
            self.allocator.free(old_cp.checkpoint_data);
            var metadata = old_cp.metadata;
            metadata.deinit();
        }

        try self.checkpoints.append(checkpoint);
        self.last_checkpoint_time = std.time.timestamp();

        return checkpoint_id;
    }

    /// Get a checkpoint by ID
    pub fn getCheckpoint(self: *StateManager, checkpoint_id: u32) ?Checkpoint {
        for (self.checkpoints.items) |cp| {
            if (cp.id == checkpoint_id) {
                return cp;
            }
        }
        return null;
    }

    /// Create a backup from a checkpoint
    pub fn createBackup(
        self: *StateManager,
        checkpoint_id: u32,
        backup_filename: []const u8,
    ) !u32 {
        self.mutex.lock();
        defer self.mutex.unlock();

        if (self.getCheckpoint(checkpoint_id) == null) {
            return error.CheckpointNotFound;
        }

        const backup_id = self.next_backup_id;
        self.next_backup_id += 1;

        // In a real implementation, this would compress and write to disk
        const backup = Backup{
            .id = backup_id,
            .timestamp = std.time.timestamp(),
            .filename = try self.allocator.dupe(u8, backup_filename),
            .size_bytes = 0, // Would be actual compressed size
            .checksum = try self.allocator.dupe(u8, "placeholder_checksum"),
            .compression_type = self.config.compression,
            .retention_days = 30,
            .created_at = std.time.timestamp(),
        };

        if (self.backups.items.len >= self.config.max_backups) {
            const old_backup = self.backups.orderedRemove(0);
            self.allocator.free(old_backup.filename);
            self.allocator.free(old_backup.checksum);
        }

        try self.backups.append(backup);
        self.last_backup_time = std.time.timestamp();

        return backup_id;
    }

    /// Create a restore point
    pub fn createRestorePoint(
        self: *StateManager,
        backup_id: u32,
        checkpoint_id: u32,
        description: []const u8,
    ) !u32 {
        self.mutex.lock();
        defer self.mutex.unlock();

        const rp_id = self.next_restore_point_id;
        self.next_restore_point_id += 1;

        const restore_point = RestorePoint{
            .id = rp_id,
            .backup_id = backup_id,
            .checkpoint_id = checkpoint_id,
            .description = try self.allocator.dupe(u8, description),
            .created_at = std.time.timestamp(),
            .can_restore = true,
        };

        try self.restore_points.append(restore_point);

        return rp_id;
    }

    /// Get a restore point by ID
    pub fn getRestorePoint(self: *StateManager, restore_point_id: u32) ?RestorePoint {
        for (self.restore_points.items) |rp| {
            if (rp.id == restore_point_id) {
                return rp;
            }
        }
        return null;
    }

    /// Restore from a checkpoint
    pub fn restoreFromCheckpoint(self: *StateManager, checkpoint_id: u32) ![]const u8 {
        self.mutex.lock();
        defer self.mutex.unlock();

        for (self.checkpoints.items) |cp| {
            if (cp.id == checkpoint_id) {
                return try self.allocator.dupe(u8, cp.checkpoint_data);
            }
        }

        return error.CheckpointNotFound;
    }

    /// Restore from a backup
    pub fn restoreFromBackup(self: *StateManager, backup_id: u32) ![]const u8 {
        self.mutex.lock();
        defer self.mutex.unlock();

        for (self.backups.items) |backup| {
            if (backup.id == backup_id) {
                // In real implementation, would decompress from disk
                return try self.allocator.dupe(u8, "restored_state_data");
            }
        }

        return error.BackupNotFound;
    }

    /// List all checkpoints
    pub fn getCheckpoints(self: *StateManager) []Checkpoint {
        return self.checkpoints.items;
    }

    /// List all backups
    pub fn getBackups(self: *StateManager) []Backup {
        return self.backups.items;
    }

    /// List all restore points
    pub fn getRestorePoints(self: *StateManager) []RestorePoint {
        return self.restore_points.items;
    }

    /// Check if auto-checkpoint is needed
    pub fn needsCheckpoint(self: *StateManager) bool {
        const now = std.time.timestamp();
        const elapsed = now - self.last_checkpoint_time;
        return elapsed >= self.config.auto_checkpoint_interval_seconds;
    }

    /// Check if auto-backup is needed
    pub fn needsBackup(self: *StateManager) bool {
        const now = std.time.timestamp();
        const elapsed = now - self.last_backup_time;
        return elapsed >= self.config.auto_backup_interval_seconds;
    }

    /// Clean up old backups based on retention
    pub fn cleanupOldBackups(self: *StateManager) void {
        self.mutex.lock();
        defer self.mutex.unlock();

        const now = std.time.timestamp();
        const cutoff = now - (@as(i64, @intCast(30 * 24 * 3600))); // 30 days

        var idx: usize = 0;
        while (idx < self.backups.items.len) {
            if (self.backups.items[idx].created_at < cutoff) {
                const old_backup = self.backups.orderedRemove(idx);
                self.allocator.free(old_backup.filename);
                self.allocator.free(old_backup.checksum);
            } else {
                idx += 1;
            }
        }
    }
};
