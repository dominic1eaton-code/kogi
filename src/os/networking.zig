//! KOGI Networking & Server System - Network communication and API server
//! Provides networking infrastructure for remote access and inter-node communication

const std = @import("std");

/// Connection protocol types
pub const Protocol = enum {
    http,
    https,
    grpc,
    websocket,
    tcp,
    udp,
};

/// Request method
pub const RequestMethod = enum {
    get,
    post,
    put,
    delete,
    patch,
    head,
    options,

    pub fn toString(self: RequestMethod) []const u8 {
        return switch (self) {
            .get => "GET",
            .post => "POST",
            .put => "PUT",
            .delete => "DELETE",
            .patch => "PATCH",
            .head => "HEAD",
            .options => "OPTIONS",
        };
    }
};

/// HTTP response status
pub const ResponseStatus = enum(u16) {
    ok = 200,
    created = 201,
    accepted = 202,
    no_content = 204,
    bad_request = 400,
    unauthorized = 401,
    forbidden = 403,
    not_found = 404,
    conflict = 409,
    internal_error = 500,
    service_unavailable = 503,
};

/// Network request
pub const NetworkRequest = struct {
    request_id: u32,
    method: RequestMethod,
    path: []const u8,
    headers: std.StringHashMap([]const u8),
    body: ?[]const u8 = null,
    timestamp: i64,
    client_ip: []const u8,
    protocol: Protocol,
};

/// Network response
pub const NetworkResponse = struct {
    request_id: u32,
    status: ResponseStatus,
    headers: std.StringHashMap([]const u8),
    body: []const u8,
    timestamp: i64,
    response_time_ms: u32,
};

/// Connection information
pub const Connection = struct {
    id: u32,
    remote_addr: []const u8,
    remote_port: u16,
    local_addr: []const u8,
    local_port: u16,
    protocol: Protocol,
    connected_at: i64,
    last_activity: i64,
    bytes_sent: u64,
    bytes_received: u64,
    authenticated: bool,
    user_id: ?u32 = null,
};

/// Server configuration
pub const ServerConfig = struct {
    port: u16 = 8080,
    bind_address: []const u8 = "0.0.0.0",
    protocol: Protocol = .http,
    max_connections: u32 = 1000,
    read_timeout_ms: u32 = 30000,
    write_timeout_ms: u32 = 30000,
    max_request_size: u64 = 1024 * 1024,
    enable_compression: bool = true,
    enable_keep_alive: bool = true,
    thread_pool_size: u32 = 8,
};

/// API endpoint
pub const Endpoint = struct {
    path: []const u8,
    method: RequestMethod,
    description: []const u8,
    require_auth: bool = true,
    handler: *const fn (*NetworkRequest, *NetworkResponse) anyerror!void,
};

/// API route matcher
pub const Router = struct {
    allocator: std.mem.Allocator,
    endpoints: std.array_list.Managed(Endpoint),

    pub fn init(allocator: std.mem.Allocator) Router {
        return Router{
            .allocator = allocator,
            .endpoints = std.array_list.Managed(Endpoint).init(allocator),
        };
    }

    pub fn deinit(self: *Router) void {
        for (self.endpoints.items) |endpoint| {
            self.allocator.free(endpoint.path);
            self.allocator.free(endpoint.description);
        }
        self.endpoints.deinit();
    }

    /// Register an endpoint
    pub fn register(
        self: *Router,
        path: []const u8,
        method: RequestMethod,
        description: []const u8,
        require_auth: bool,
        handler: *const fn (*NetworkRequest, *NetworkResponse) anyerror!void,
    ) !void {
        const endpoint = Endpoint{
            .path = try self.allocator.dupe(u8, path),
            .method = method,
            .description = try self.allocator.dupe(u8, description),
            .require_auth = require_auth,
            .handler = handler,
        };
        try self.endpoints.append(endpoint);
    }

    /// Find matching endpoint
    pub fn findEndpoint(self: *Router, path: []const u8, method: RequestMethod) ?Endpoint {
        for (self.endpoints.items) |endpoint| {
            if (std.mem.eql(u8, endpoint.path, path) and endpoint.method == method) {
                return endpoint;
            }
        }
        return null;
    }
};

/// Network server
pub const NetworkServer = struct {
    allocator: std.mem.Allocator,
    config: ServerConfig,
    router: Router,
    connections: std.array_list.Managed(Connection),
    requests: std.array_list.Managed(NetworkRequest),
    responses: std.array_list.Managed(NetworkResponse),
    running: bool = false,
    next_conn_id: u32 = 0,
    next_request_id: u32 = 0,
    mutex: std.Thread.Mutex = .{},

    pub fn init(allocator: std.mem.Allocator, config: ServerConfig) NetworkServer {
        return NetworkServer{
            .allocator = allocator,
            .config = config,
            .router = Router.init(allocator),
            .connections = std.array_list.Managed(Connection).init(allocator),
            .requests = std.array_list.Managed(NetworkRequest).init(allocator),
            .responses = std.array_list.Managed(NetworkResponse).init(allocator),
        };
    }

    pub fn deinit(self: *NetworkServer) void {
        for (self.connections.items) |conn| {
            self.allocator.free(conn.remote_addr);
            self.allocator.free(conn.local_addr);
        }
        self.connections.deinit();

        for (self.requests.items) |req| {
            var headers = req.headers;
            var iter = headers.iterator();
            while (iter.next()) |kv| {
                self.allocator.free(kv.key_ptr.*);
                self.allocator.free(kv.value_ptr.*);
            }
            headers.deinit();
            self.allocator.free(req.path);
            self.allocator.free(req.client_ip);
            if (req.body) |body| {
                self.allocator.free(body);
            }
        }
        self.requests.deinit();

        for (self.responses.items) |resp| {
            var headers = resp.headers;
            var iter = headers.iterator();
            while (iter.next()) |kv| {
                self.allocator.free(kv.key_ptr.*);
                self.allocator.free(kv.value_ptr.*);
            }
            headers.deinit();
            self.allocator.free(resp.body);
        }
        self.responses.deinit();

        self.router.deinit();
    }

    /// Start the server
    pub fn start(self: *NetworkServer) !void {
        self.mutex.lock();
        self.running = true;
        self.mutex.unlock();
    }

    /// Stop the server
    pub fn stop(self: *NetworkServer) void {
        self.mutex.lock();
        self.running = false;
        self.mutex.unlock();
    }

    /// Register a new connection
    pub fn registerConnection(
        self: *NetworkServer,
        remote_addr: []const u8,
        remote_port: u16,
    ) !u32 {
        self.mutex.lock();
        defer self.mutex.unlock();

        if (self.connections.items.len >= self.config.max_connections) {
            return error.MaxConnectionsReached;
        }

        const conn_id = self.next_conn_id;
        self.next_conn_id += 1;

        const connection = Connection{
            .id = conn_id,
            .remote_addr = try self.allocator.dupe(u8, remote_addr),
            .remote_port = remote_port,
            .local_addr = try self.allocator.dupe(u8, self.config.bind_address),
            .local_port = self.config.port,
            .protocol = self.config.protocol,
            .connected_at = std.time.timestamp(),
            .last_activity = std.time.timestamp(),
            .bytes_sent = 0,
            .bytes_received = 0,
            .authenticated = false,
        };

        try self.connections.append(connection);
        return conn_id;
    }

    /// Handle incoming request
    pub fn handleRequest(
        self: *NetworkServer,
        req: NetworkRequest,
    ) !NetworkResponse {
        self.mutex.lock();
        defer self.mutex.unlock();

        const request_id = self.next_request_id;
        self.next_request_id += 1;

        var req_mut = req;
        req_mut.request_id = request_id;

        try self.requests.append(req_mut);

        // Find matching endpoint
        if (self.router.findEndpoint(req.path, req.method)) |endpoint| {
            var response = NetworkResponse{
                .request_id = request_id,
                .status = .ok,
                .headers = std.StringHashMap([]const u8).init(self.allocator),
                .body = try self.allocator.dupe(u8, ""),
                .timestamp = std.time.timestamp(),
                .response_time_ms = 0,
            };

            // Call handler
            var req_mutable = req_mut;
            endpoint.handler(&req_mutable, &response) catch |e| {
                response.status = .internal_error;
                _ = e;
            };

            try self.responses.append(response);
            return response;
        }

        // Not found
        const response = NetworkResponse{
            .request_id = request_id,
            .status = .not_found,
            .headers = std.StringHashMap([]const u8).init(self.allocator),
            .body = try self.allocator.dupe(u8, "Not Found"),
            .timestamp = std.time.timestamp(),
            .response_time_ms = 0,
        };

        try self.responses.append(response);
        return response;
    }

    /// Close a connection
    pub fn closeConnection(self: *NetworkServer, conn_id: u32) !void {
        self.mutex.lock();
        defer self.mutex.unlock();

        for (self.connections.items, 0..) |_, idx| {
            if (self.connections.items[idx].id == conn_id) {
                const conn = self.connections.orderedRemove(idx);
                self.allocator.free(conn.remote_addr);
                self.allocator.free(conn.local_addr);
                return;
            }
        }
        return error.ConnectionNotFound;
    }

    /// Get active connection count
    pub fn getConnectionCount(self: *NetworkServer) usize {
        return self.connections.items.len;
    }

    /// Get server status
    pub fn isRunning(self: *NetworkServer) bool {
        return self.running;
    }

    /// Get all connections
    pub fn getConnections(self: *NetworkServer) []Connection {
        return self.connections.items;
    }

    /// Get request history
    pub fn getRequests(self: *NetworkServer) []NetworkRequest {
        return self.requests.items;
    }

    /// Get response history
    pub fn getResponses(self: *NetworkServer) []NetworkResponse {
        return self.responses.items;
    }
};

/// Network client for inter-node communication
pub const NetworkClient = struct {
    allocator: std.mem.Allocator,
    server_addr: []const u8,
    server_port: u16,
    connected: bool = false,
    requests_sent: u64 = 0,
    responses_received: u64 = 0,

    pub fn init(allocator: std.mem.Allocator, server_addr: []const u8, server_port: u16) NetworkClient {
        return NetworkClient{
            .allocator = allocator,
            .server_addr = server_addr,
            .server_port = server_port,
        };
    }

    pub fn deinit(self: *NetworkClient) void {
        _ = self; // Cleanup if needed
    }

    /// Connect to remote server
    pub fn connect(self: *NetworkClient) !void {
        self.connected = true;
    }

    /// Send a request
    pub fn sendRequest(self: *NetworkClient, request: NetworkRequest) !NetworkResponse {
        self.requests_sent += 1;
        self.responses_received += 1;

        const response = NetworkResponse{
            .request_id = request.request_id,
            .status = .ok,
            .headers = std.StringHashMap([]const u8).init(self.allocator),
            .body = try self.allocator.dupe(u8, ""),
            .timestamp = std.time.timestamp(),
            .response_time_ms = 0,
        };

        return response;
    }

    /// Disconnect from server
    pub fn disconnect(self: *NetworkClient) void {
        self.connected = false;
    }
};
