//! KOGI Distributed Node System - Multi-node redundancy and load balancing
//! Enables KOGI OS to run as distributed cluster with failover and load balancing

const std = @import("std");

/// Node status enum
pub const NodeStatus = enum {
    initializing,
    online,
    degraded,
    offline,
    failing,
    recovering,
};

/// Node role in cluster
pub const NodeRole = enum {
    primary,
    replica,
    backup,
    learner,
};

/// Health check result
pub const HealthCheckResult = struct {
    timestamp: i64,
    status: NodeStatus,
    response_time_ms: u32,
    cpu_usage: f32,
    memory_usage: f32,
    disk_usage: f32,
    error_count: u32,
};

/// Node information
pub const Node = struct {
    id: u32,
    node_name: []const u8,
    ip_address: []const u8,
    port: u16,
    status: NodeStatus,
    role: NodeRole,
    version: []const u8,
    joined_at: i64,
    last_heartbeat: i64,
    reachable: bool,
    health: HealthCheckResult,
    metadata: std.StringHashMap([]const u8),
};

/// Replication state
pub const ReplicationState = struct {
    source_node_id: u32,
    target_node_id: u32,
    sequence_number: u64,
    last_replicated_at: i64,
    pending_changes: u32,
    replication_lag_ms: u32,
};

/// Load balancing strategy
pub const LoadBalancingStrategy = enum {
    round_robin,
    least_connections,
    weighted,
    random,
    ip_hash,
    response_time,
};

/// Distributed cluster configuration
pub const ClusterConfig = struct {
    cluster_name: []const u8,
    node_id: u32,
    bind_address: []const u8 = "0.0.0.0",
    bind_port: u16 = 9000,
    replication_interval_ms: u32 = 100,
    health_check_interval_ms: u32 = 5000,
    heartbeat_timeout_ms: u32 = 30000,
    max_nodes: usize = 100,
    load_balancing_strategy: LoadBalancingStrategy = .round_robin,
};

/// Distributed cluster manager
pub const DistributedCluster = struct {
    allocator: std.mem.Allocator,
    config: ClusterConfig,
    nodes: std.array_list.Managed(Node),
    replications: std.array_list.Managed(ReplicationState),
    local_node_id: u32,
    sequence_number: u64 = 0,
    mutex: std.Thread.Mutex = .{},
    next_node_id: u32 = 0,
    load_balance_index: usize = 0,

    pub fn init(allocator: std.mem.Allocator, config: ClusterConfig) DistributedCluster {
        return DistributedCluster{
            .allocator = allocator,
            .config = config,
            .nodes = std.array_list.Managed(Node).init(allocator),
            .replications = std.array_list.Managed(ReplicationState).init(allocator),
            .local_node_id = config.node_id,
        };
    }

    pub fn deinit(self: *DistributedCluster) void {
        for (self.nodes.items) |node| {
            self.allocator.free(node.node_name);
            self.allocator.free(node.ip_address);
            self.allocator.free(node.version);
            var metadata = node.metadata;
            var iter = metadata.iterator();
            while (iter.next()) |kv| {
                self.allocator.free(kv.key_ptr.*);
                self.allocator.free(kv.value_ptr.*);
            }
            metadata.deinit();
        }
        self.nodes.deinit();
        self.replications.deinit();
    }

    /// Add a node to the cluster
    pub fn addNode(
        self: *DistributedCluster,
        node_name: []const u8,
        ip_address: []const u8,
        port: u16,
        role: NodeRole,
    ) !u32 {
        self.mutex.lock();
        defer self.mutex.unlock();

        if (self.nodes.items.len >= self.config.max_nodes) {
            return error.ClusterFull;
        }

        const node_id = self.next_node_id;
        self.next_node_id += 1;

        const node = Node{
            .id = node_id,
            .node_name = try self.allocator.dupe(u8, node_name),
            .ip_address = try self.allocator.dupe(u8, ip_address),
            .port = port,
            .status = .initializing,
            .role = role,
            .version = try self.allocator.dupe(u8, "1.0.0"),
            .joined_at = std.time.timestamp(),
            .last_heartbeat = std.time.timestamp(),
            .reachable = false,
            .health = HealthCheckResult{
                .timestamp = std.time.timestamp(),
                .status = .initializing,
                .response_time_ms = 0,
                .cpu_usage = 0,
                .memory_usage = 0,
                .disk_usage = 0,
                .error_count = 0,
            },
            .metadata = std.StringHashMap([]const u8).init(self.allocator),
        };

        try self.nodes.append(node);
        return node_id;
    }

    /// Get node by ID
    pub fn getNode(self: *DistributedCluster, node_id: u32) ?*Node {
        for (self.nodes.items) |*node| {
            if (node.id == node_id) {
                return node;
            }
        }
        return null;
    }

    /// Update node heartbeat
    pub fn updateHeartbeat(self: *DistributedCluster, node_id: u32) !void {
        self.mutex.lock();
        defer self.mutex.unlock();

        for (self.nodes.items) |*node| {
            if (node.id == node_id) {
                node.last_heartbeat = std.time.timestamp();
                node.status = .online;
                node.reachable = true;
                return;
            }
        }
        return error.NodeNotFound;
    }

    /// Update node health
    pub fn updateHealth(
        self: *DistributedCluster,
        node_id: u32,
        health: HealthCheckResult,
    ) !void {
        self.mutex.lock();
        defer self.mutex.unlock();

        for (self.nodes.items) |*node| {
            if (node.id == node_id) {
                node.health = health;
                if (health.status == .online) {
                    node.status = .online;
                    node.reachable = true;
                } else if (health.status == .degraded) {
                    node.status = .degraded;
                    node.reachable = true;
                }
                return;
            }
        }
        return error.NodeNotFound;
    }

    /// Get all online nodes
    pub fn getOnlineNodes(self: *DistributedCluster, allocator: std.mem.Allocator) !std.array_list.Managed(*Node) {
        self.mutex.lock();
        defer self.mutex.unlock();

        var online_nodes = std.array_list.Managed(*Node).init(allocator);
        for (self.nodes.items) |*node| {
            if (node.status == .online or node.status == .degraded) {
                try online_nodes.append(node);
            }
        }
        return online_nodes;
    }

    /// Get next node for load balancing
    pub fn getNextNode(self: *DistributedCluster) !*Node {
        self.mutex.lock();
        defer self.mutex.unlock();

        // Get online nodes
        var online_nodes = std.array_list.Managed(usize).init(self.allocator);
        for (self.nodes.items, 0..) |node, idx| {
            if (node.status == .online or node.status == .degraded) {
                try online_nodes.append(idx);
            }
        }
        defer online_nodes.deinit();

        if (online_nodes.items.len == 0) {
            return error.NoOnlineNodes;
        }

        const next_idx = switch (self.config.load_balancing_strategy) {
            .round_robin => {
                const idx = online_nodes.items[self.load_balance_index % online_nodes.items.len];
                self.load_balance_index += 1;
                idx;
            },
            .random => blk: {
                var prng = std.rand.DefaultPrng.init(@intCast(std.time.timestamp()));
                const rand_idx = prng.random().intRangeLessThan(usize, 0, online_nodes.items.len);
                break :blk online_nodes.items[rand_idx];
            },
            else => online_nodes.items[0],
        };

        return &self.nodes.items[next_idx];
    }

    /// Setup replication between nodes
    pub fn setupReplication(
        self: *DistributedCluster,
        source_node_id: u32,
        target_node_id: u32,
    ) !void {
        self.mutex.lock();
        defer self.mutex.unlock();

        const replication = ReplicationState{
            .source_node_id = source_node_id,
            .target_node_id = target_node_id,
            .sequence_number = 0,
            .last_replicated_at = std.time.timestamp(),
            .pending_changes = 0,
            .replication_lag_ms = 0,
        };

        try self.replications.append(replication);
    }

    /// Get replication state between nodes
    pub fn getReplicationState(
        self: *DistributedCluster,
        source_node_id: u32,
        target_node_id: u32,
    ) ?ReplicationState {
        for (self.replications.items) |rep| {
            if (rep.source_node_id == source_node_id and rep.target_node_id == target_node_id) {
                return rep;
            }
        }
        return null;
    }

    /// Mark node as offline
    pub fn markNodeOffline(self: *DistributedCluster, node_id: u32) !void {
        self.mutex.lock();
        defer self.mutex.unlock();

        for (self.nodes.items) |*node| {
            if (node.id == node_id) {
                node.status = .offline;
                node.reachable = false;
                return;
            }
        }
        return error.NodeNotFound;
    }

    /// Mark node as recovering
    pub fn markNodeRecovering(self: *DistributedCluster, node_id: u32) !void {
        self.mutex.lock();
        defer self.mutex.unlock();

        for (self.nodes.items) |*node| {
            if (node.id == node_id) {
                node.status = .recovering;
                return;
            }
        }
        return error.NodeNotFound;
    }

    /// Get cluster statistics
    pub fn getClusterStats(self: *DistributedCluster) struct {
        total_nodes: usize,
        online_nodes: usize,
        offline_nodes: usize,
        degraded_nodes: usize,
        total_memory_gb: f32,
        avg_cpu_usage: f32,
    } {
        var online_count: usize = 0;
        var offline_count: usize = 0;
        var degraded_count: usize = 0;
        var total_memory: f32 = 0;
        var total_cpu: f32 = 0;

        for (self.nodes.items) |node| {
            if (node.status == .online) {
                online_count += 1;
            } else if (node.status == .offline) {
                offline_count += 1;
            } else if (node.status == .degraded) {
                degraded_count += 1;
            }
            total_memory += node.health.memory_usage;
            total_cpu += node.health.cpu_usage;
        }

        const avg_cpu = if (self.nodes.items.len > 0) total_cpu / @as(f32, @floatFromInt(self.nodes.items.len)) else 0;

        return .{
            .total_nodes = self.nodes.items.len,
            .online_nodes = online_count,
            .offline_nodes = offline_count,
            .degraded_nodes = degraded_count,
            .total_memory_gb = total_memory,
            .avg_cpu_usage = avg_cpu,
        };
    }

    /// Get all nodes
    pub fn getNodes(self: *DistributedCluster) []Node {
        return self.nodes.items;
    }
};
