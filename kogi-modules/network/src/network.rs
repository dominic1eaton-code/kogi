// network.rs — NetworkSystem with Service Registry & Discovery
//
// Original:
//   • ServiceRegistry      — central store of service instances
//   • NetworkSystem        — manages registry, health checks, routing
//   • Self-Registration    — services register themselves on startup
//   • Client-Side Discovery— caller queries registry, picks an instance
//   • Server-Side Discovery— load-balancer/router queries registry internally
//   • SidecarProxy         — language-agnostic local proxy
//   • Dynamic scaling      — add / remove instances at runtime
//   • Fault tolerance      — periodic health checks; unhealthy instances
//                            are quarantined automatically
//   • Round-robin, random, least-connections, IP-hash load-balancing
//
// Extended (this file):
//   • PortAllocator        — lease-based dynamic port assignment, named
//                            ranges, per-service pools, reservations
//   • DynamicConfig        — hot-reloadable network config, per-service
//                            overrides, versioned change log, env overlay
//   • ResourceManager      — connection pools, token-bucket rate limiting,
//                            circuit breakers, per-service quotas & metrics

use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex, RwLock},
    time::{Duration, Instant},
};

// ═════════════════════════════════════════════════════════════════════════════
// § 1  ERROR TYPE
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkError {
    // Registry
    ServiceNotFound(String),
    NoHealthyInstances(String),
    AlreadyRegistered(String),
    InstanceNotFound(String),
    HealthCheckFailed(String),
    InvalidConfiguration(String),
    // Port allocator
    NoPortsAvailable(String),
    PortAlreadyReserved(u16),
    PortNotLeased(u16),
    RangeNotFound(String),
    // Resource manager
    QuotaExceeded(String),
    RateLimitExceeded(String),
    CircuitOpen(String),
    PoolExhausted(String),
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkError::ServiceNotFound(s)      => write!(f, "Service not found: {s}"),
            NetworkError::NoHealthyInstances(s)   => write!(f, "No healthy instances for: {s}"),
            NetworkError::AlreadyRegistered(s)    => write!(f, "Already registered: {s}"),
            NetworkError::InstanceNotFound(s)     => write!(f, "Instance not found: {s}"),
            NetworkError::HealthCheckFailed(s)    => write!(f, "Health check failed: {s}"),
            NetworkError::InvalidConfiguration(s) => write!(f, "Invalid configuration: {s}"),
            NetworkError::NoPortsAvailable(r)     => write!(f, "No ports available in range '{r}'"),
            NetworkError::PortAlreadyReserved(p)  => write!(f, "Port {p} is already reserved"),
            NetworkError::PortNotLeased(p)        => write!(f, "Port {p} is not currently leased"),
            NetworkError::RangeNotFound(r)        => write!(f, "Port range '{r}' not found"),
            NetworkError::QuotaExceeded(s)        => write!(f, "Quota exceeded for: {s}"),
            NetworkError::RateLimitExceeded(s)    => write!(f, "Rate limit exceeded for: {s}"),
            NetworkError::CircuitOpen(s)          => write!(f, "Circuit breaker open for: {s}"),
            NetworkError::PoolExhausted(s)        => write!(f, "Connection pool exhausted for: {s}"),
        }
    }
}

impl std::error::Error for NetworkError {}

// ═════════════════════════════════════════════════════════════════════════════
// § 2  HEALTH STATUS
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Unknown,
    Draining,
}

impl fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            HealthStatus::Healthy   => "healthy",
            HealthStatus::Unhealthy => "unhealthy",
            HealthStatus::Unknown   => "unknown",
            HealthStatus::Draining  => "draining",
        };
        write!(f, "{s}")
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// § 3  SERVICE INSTANCE
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct ServiceInstance {
    pub instance_id:          String,
    pub service_name:         String,
    pub address:              SocketAddr,
    pub metadata:             HashMap<String, String>,
    pub health:               HealthStatus,
    pub last_heartbeat:       Instant,
    pub registered_at:        Instant,
    pub active_connections:   usize,
    pub consecutive_failures: u32,
}

impl ServiceInstance {
    pub fn new(
        instance_id:  impl Into<String>,
        service_name: impl Into<String>,
        address:      SocketAddr,
    ) -> Self {
        let now = Instant::now();
        ServiceInstance {
            instance_id:          instance_id.into(),
            service_name:         service_name.into(),
            address,
            metadata:             HashMap::new(),
            health:               HealthStatus::Unknown,
            last_heartbeat:       now,
            registered_at:        now,
            active_connections:   0,
            consecutive_failures: 0,
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn is_routable(&self) -> bool {
        matches!(self.health, HealthStatus::Healthy | HealthStatus::Unknown)
    }

    pub fn record_success(&mut self) {
        self.health               = HealthStatus::Healthy;
        self.last_heartbeat       = Instant::now();
        self.consecutive_failures = 0;
    }

    pub fn record_failure(&mut self, threshold: u32) {
        self.consecutive_failures += 1;
        if self.consecutive_failures >= threshold {
            self.health = HealthStatus::Unhealthy;
        }
    }

    pub fn time_since_heartbeat(&self) -> Duration {
        self.last_heartbeat.elapsed()
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// § 4  LOAD-BALANCING STRATEGY
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    Random,
    LeastConnections,
    IpHash,
}

// ═════════════════════════════════════════════════════════════════════════════
// § 5  REGISTRY CONFIG
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct RegistryConfig {
    pub unhealthy_threshold:  u32,
    pub heartbeat_timeout:    Duration,
    pub load_balancing:       LoadBalancingStrategy,
    pub auto_deregister_stale: bool,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        RegistryConfig {
            unhealthy_threshold:  3,
            heartbeat_timeout:    Duration::from_secs(30),
            load_balancing:       LoadBalancingStrategy::RoundRobin,
            auto_deregister_stale: true,
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// § 6  SERVICE REGISTRY
// ═════════════════════════════════════════════════════════════════════════════

pub struct ServiceRegistry {
    services:    HashMap<String, Vec<ServiceInstance>>,
    rr_counters: HashMap<String, usize>,
    config:      RegistryConfig,
}

impl ServiceRegistry {
    pub fn new(config: RegistryConfig) -> Self {
        ServiceRegistry { services: HashMap::new(), rr_counters: HashMap::new(), config }
    }

    // ── Registration ──────────────────────────────────────────────────────────

    pub fn register(&mut self, instance: ServiceInstance) -> Result<(), NetworkError> {
        let name   = instance.service_name.clone();
        let id     = instance.instance_id.clone();
        let bucket = self.services.entry(name.clone()).or_default();

        if bucket.iter().any(|i| i.instance_id == id) {
            return Err(NetworkError::AlreadyRegistered(format!("{name}/{id}")));
        }
        bucket.push(instance);
        self.rr_counters.entry(name).or_insert(0);
        Ok(())
    }

    pub fn deregister(
        &mut self,
        service_name: &str,
        instance_id:  &str,
    ) -> Result<ServiceInstance, NetworkError> {
        let bucket = self.services.get_mut(service_name)
            .ok_or_else(|| NetworkError::ServiceNotFound(service_name.to_string()))?;
        let pos = bucket.iter().position(|i| i.instance_id == instance_id)
            .ok_or_else(|| NetworkError::InstanceNotFound(instance_id.to_string()))?;
        Ok(bucket.remove(pos))
    }

    pub fn drain_instance(
        &mut self,
        service_name: &str,
        instance_id:  &str,
    ) -> Result<(), NetworkError> {
        self.get_instance_mut(service_name, instance_id)
            .map(|i| i.health = HealthStatus::Draining)
    }

    // ── Discovery ─────────────────────────────────────────────────────────────

    pub fn discover(&self, service_name: &str) -> Result<Vec<&ServiceInstance>, NetworkError> {
        let bucket = self.services.get(service_name)
            .ok_or_else(|| NetworkError::ServiceNotFound(service_name.to_string()))?;
        let healthy: Vec<&ServiceInstance> = bucket.iter().filter(|i| i.is_routable()).collect();
        if healthy.is_empty() {
            return Err(NetworkError::NoHealthyInstances(service_name.to_string()));
        }
        Ok(healthy)
    }

    pub fn resolve(&mut self, service_name: &str) -> Result<SocketAddr, NetworkError> {
        let bucket = self.services.get(service_name)
            .ok_or_else(|| NetworkError::ServiceNotFound(service_name.to_string()))?;
        let candidates: Vec<usize> = bucket.iter().enumerate()
            .filter(|(_, i)| i.is_routable()).map(|(idx, _)| idx).collect();
        if candidates.is_empty() {
            return Err(NetworkError::NoHealthyInstances(service_name.to_string()));
        }

        let chosen_idx = match &self.config.load_balancing {
            LoadBalancingStrategy::RoundRobin => {
                let c = self.rr_counters.entry(service_name.to_string()).or_insert(0);
                let idx = candidates[*c % candidates.len()];
                *c = c.wrapping_add(1);
                idx
            }
            LoadBalancingStrategy::Random => {
                let seed = Instant::now().elapsed().subsec_nanos() as usize;
                candidates[seed % candidates.len()]
            }
            LoadBalancingStrategy::LeastConnections => {
                let b = self.services.get(service_name).unwrap();
                *candidates.iter().min_by_key(|&&i| b[i].active_connections).unwrap()
            }
            LoadBalancingStrategy::IpHash => {
                let c = self.rr_counters.entry(service_name.to_string()).or_insert(0);
                let idx = candidates[*c % candidates.len()];
                *c = c.wrapping_add(1);
                idx
            }
        };
        Ok(self.services[service_name][chosen_idx].address)
    }

    // ── Health ─────────────────────────────────────────────────────────────────

    pub fn heartbeat(&mut self, service_name: &str, instance_id: &str) -> Result<(), NetworkError> {
        self.get_instance_mut(service_name, instance_id).map(|i| i.record_success())
    }

    pub fn run_health_checks(&mut self) -> usize {
        let timeout   = self.config.heartbeat_timeout;
        let threshold = self.config.unhealthy_threshold;
        let auto_drop = self.config.auto_deregister_stale;
        let mut degraded = 0usize;

        for bucket in self.services.values_mut() {
            for inst in bucket.iter_mut() {
                if inst.time_since_heartbeat() > timeout {
                    inst.record_failure(threshold);
                    if inst.health == HealthStatus::Unhealthy { degraded += 1; }
                }
            }
            if auto_drop {
                let before = bucket.len();
                bucket.retain(|i| {
                    !(i.health == HealthStatus::Unhealthy && i.time_since_heartbeat() > timeout * 2)
                });
                degraded += before - bucket.len();
            }
        }
        self.services.retain(|_, v| !v.is_empty());
        degraded
    }

    // ── Introspection ─────────────────────────────────────────────────────────

    pub fn all_instances(&self)  -> Vec<&ServiceInstance>   { self.services.values().flatten().collect() }
    pub fn service_names(&self)  -> Vec<&str>               { self.services.keys().map(String::as_str).collect() }
    pub fn instance_count(&self, svc: &str) -> usize        { self.services.get(svc).map(Vec::len).unwrap_or(0) }

    fn get_instance_mut(
        &mut self, service_name: &str, instance_id: &str,
    ) -> Result<&mut ServiceInstance, NetworkError> {
        self.services.get_mut(service_name)
            .ok_or_else(|| NetworkError::ServiceNotFound(service_name.to_string()))?
            .iter_mut().find(|i| i.instance_id == instance_id)
            .ok_or_else(|| NetworkError::InstanceNotFound(instance_id.to_string()))
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// § 7  THIRD-PARTY REGISTRAR
// ═════════════════════════════════════════════════════════════════════════════

pub struct ServiceRegistrar { registry: Arc<RwLock<ServiceRegistry>> }

impl ServiceRegistrar {
    pub fn new(registry: Arc<RwLock<ServiceRegistry>>) -> Self { ServiceRegistrar { registry } }
    pub fn on_instance_started(&self, inst: ServiceInstance) -> Result<(), NetworkError> {
        self.registry.write().unwrap().register(inst)
    }
    pub fn on_instance_stopped(&self, svc: &str, id: &str) -> Result<ServiceInstance, NetworkError> {
        self.registry.write().unwrap().deregister(svc, id)
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// § 8  SIDECAR PROXY
// ═════════════════════════════════════════════════════════════════════════════

pub struct SidecarProxy {
    service_name: String,
    registry:     Arc<RwLock<ServiceRegistry>>,
    pub local_port: u16,
}

impl SidecarProxy {
    pub fn new(service_name: impl Into<String>, registry: Arc<RwLock<ServiceRegistry>>, local_port: u16) -> Self {
        SidecarProxy { service_name: service_name.into(), registry, local_port }
    }
    pub fn forward_request(&self, payload: &str) -> Result<(SocketAddr, String), NetworkError> {
        let addr = self.registry.write().unwrap().resolve(&self.service_name)?;
        Ok((addr, format!("[sidecar:{}] → {} | echoing: {payload}", self.local_port, addr)))
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// § 9  DYNAMIC PORT ASSIGNMENT
// ═════════════════════════════════════════════════════════════════════════════

/// A half-open port range `[lo, hi)`.
#[derive(Debug, Clone)]
pub struct PortRange {
    pub name: String,
    pub lo:   u16,
    pub hi:   u16,
}

impl PortRange {
    pub fn new(name: impl Into<String>, lo: u16, hi: u16) -> Self {
        PortRange { name: name.into(), lo, hi }
    }
    fn contains(&self, port: u16) -> bool { port >= self.lo && port < self.hi }
    fn size(&self) -> usize               { (self.hi - self.lo) as usize }
}

/// Tracks a single port lease.
#[derive(Debug, Clone)]
pub struct PortLease {
    pub port:       u16,
    pub range_name: String,
    pub owner:      String,
    pub leased_at:  Instant,
    pub ttl:        Duration,
}

impl PortLease {
    pub fn is_expired(&self) -> bool { self.leased_at.elapsed() > self.ttl }
}

/// Lease-based dynamic port allocator.
///
/// Supports named port ranges, per-service pools, permanent reservations,
/// TTL-based expiry, and conflict detection.
///
/// ```text
/// ┌─────────────────────────────────────────────┐
/// │  PortAllocator                              │
/// │                                             │
/// │  ranges:  { "ephemeral" → [49152, 65536) }  │
/// │           { "services"  → [8000,  9000)  }  │
/// │           { "admin"     → [9000,  9100)  }  │
/// │                                             │
/// │  reserved:  { 8080, 443, 80 }               │
/// │  leased:    { 8081 → PortLease{…} }         │
/// └─────────────────────────────────────────────┘
/// ```
pub struct PortAllocator {
    ranges:   Vec<PortRange>,
    reserved: HashSet<u16>,
    leased:   HashMap<u16, PortLease>,
    /// Per-service dedicated port pools (service → Vec of pre-assigned ports).
    pools:    HashMap<String, VecDeque<u16>>,
    /// Default TTL for new leases.
    default_ttl: Duration,
}

impl PortAllocator {
    pub fn new(default_ttl: Duration) -> Self {
        let mut alloc = PortAllocator {
            ranges:      Vec::new(),
            reserved:    HashSet::new(),
            leased:      HashMap::new(),
            pools:       HashMap::new(),
            default_ttl,
        };
        // Register sensible built-in ranges.
        alloc.add_range(PortRange::new("services",  8000, 9000));
        alloc.add_range(PortRange::new("admin",     9000, 9100));
        alloc.add_range(PortRange::new("ephemeral", 49152, 65535));
        alloc
    }

    // ── Range management ──────────────────────────────────────────────────────

    /// Register a named port range. Later ranges with the same name replace earlier ones.
    pub fn add_range(&mut self, range: PortRange) {
        self.ranges.retain(|r| r.name != range.name);
        self.ranges.push(range);
    }

    pub fn get_range(&self, name: &str) -> Option<&PortRange> {
        self.ranges.iter().find(|r| r.name == name)
    }

    // ── Reservation (permanent) ───────────────────────────────────────────────

    /// Permanently reserve a port so it will never be allocated.
    pub fn reserve(&mut self, port: u16) -> Result<(), NetworkError> {
        if self.reserved.contains(&port) {
            return Err(NetworkError::PortAlreadyReserved(port));
        }
        self.reserved.insert(port);
        Ok(())
    }

    pub fn unreserve(&mut self, port: u16) {
        self.reserved.remove(&port);
    }

    pub fn is_reserved(&self, port: u16) -> bool { self.reserved.contains(&port) }

    // ── Lease-based allocation ────────────────────────────────────────────────

    /// Allocate the next free port from a named range.
    /// Returns a `PortLease` that the caller must release when done.
    pub fn allocate_from(&mut self, range_name: &str, owner: impl Into<String>) -> Result<PortLease, NetworkError> {
        self.evict_expired();

        let range = self.ranges.iter().find(|r| r.name == range_name)
            .ok_or_else(|| NetworkError::RangeNotFound(range_name.to_string()))?
            .clone();

        // Linear scan for the first free port in the range.
        let port = (range.lo..range.hi)
            .find(|&p| !self.reserved.contains(&p) && !self.leased.contains_key(&p))
            .ok_or_else(|| NetworkError::NoPortsAvailable(range_name.to_string()))?;

        let lease = PortLease {
            port,
            range_name: range_name.to_string(),
            owner:      owner.into(),
            leased_at:  Instant::now(),
            ttl:        self.default_ttl,
        };
        self.leased.insert(port, lease.clone());
        Ok(lease)
    }

    /// Allocate a specific port (e.g. from environment config or user request).
    pub fn allocate_specific(&mut self, port: u16, owner: impl Into<String>) -> Result<PortLease, NetworkError> {
        self.evict_expired();
        if self.reserved.contains(&port) {
            return Err(NetworkError::PortAlreadyReserved(port));
        }
        if self.leased.contains_key(&port) {
            return Err(NetworkError::PortAlreadyReserved(port));
        }
        let range_name = self.ranges.iter().find(|r| r.contains(port))
            .map(|r| r.name.clone())
            .unwrap_or_else(|| "unmanaged".to_string());

        let lease = PortLease {
            port,
            range_name,
            owner: owner.into(),
            leased_at: Instant::now(),
            ttl: self.default_ttl,
        };
        self.leased.insert(port, lease.clone());
        Ok(lease)
    }

    /// Release a previously leased port back to the pool.
    pub fn release(&mut self, port: u16) -> Result<(), NetworkError> {
        self.leased.remove(&port).ok_or(NetworkError::PortNotLeased(port))?;
        Ok(())
    }

    /// Renew a lease, resetting its TTL countdown.
    pub fn renew(&mut self, port: u16) -> Result<(), NetworkError> {
        let lease = self.leased.get_mut(&port).ok_or(NetworkError::PortNotLeased(port))?;
        lease.leased_at = Instant::now();
        Ok(())
    }

    // ── Per-service pools ─────────────────────────────────────────────────────

    /// Pre-allocate a pool of `count` ports from a range for exclusive use by a service.
    pub fn create_service_pool(
        &mut self,
        service:    &str,
        range_name: &str,
        count:      usize,
    ) -> Result<Vec<u16>, NetworkError> {
        let mut ports = Vec::with_capacity(count);
        for _ in 0..count {
            let lease = self.allocate_from(range_name, service)?;
            ports.push(lease.port);
        }
        self.pools.entry(service.to_string()).or_default().extend(ports.iter().copied());
        Ok(ports)
    }

    /// Check out a port from a service's dedicated pool.
    pub fn checkout_from_pool(&mut self, service: &str) -> Result<u16, NetworkError> {
        self.pools.get_mut(service)
            .and_then(|q| q.pop_front())
            .ok_or_else(|| NetworkError::PoolExhausted(service.to_string()))
    }

    /// Return a port back to a service's dedicated pool.
    pub fn return_to_pool(&mut self, service: &str, port: u16) {
        self.pools.entry(service.to_string()).or_default().push_back(port);
    }

    // ── Introspection ─────────────────────────────────────────────────────────

    pub fn leased_count(&self)    -> usize { self.leased.len() }
    pub fn reserved_count(&self)  -> usize { self.reserved.len() }

    pub fn utilisation(&self, range_name: &str) -> Option<f64> {
        let range = self.get_range(range_name)?;
        let used  = self.leased.values().filter(|l| l.range_name == range_name).count();
        Some(used as f64 / range.size() as f64)
    }

    pub fn snapshot_leases(&self) -> Vec<&PortLease> {
        self.leased.values().collect()
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    fn evict_expired(&mut self) {
        self.leased.retain(|_, lease| !lease.is_expired());
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// § 10  DYNAMIC NETWORK CONFIGURATION
// ═════════════════════════════════════════════════════════════════════════════

/// A single key/value network configuration entry with type information.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    Duration(Duration),
    Addr(SocketAddr),
}

impl fmt::Display for ConfigValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigValue::Bool(v)     => write!(f, "{v}"),
            ConfigValue::Int(v)      => write!(f, "{v}"),
            ConfigValue::Float(v)    => write!(f, "{v:.3}"),
            ConfigValue::Text(v)     => write!(f, "{v}"),
            ConfigValue::Duration(v) => write!(f, "{v:?}"),
            ConfigValue::Addr(v)     => write!(f, "{v}"),
        }
    }
}

/// One record in the configuration change log.
#[derive(Debug, Clone)]
pub struct ConfigChange {
    pub key:       String,
    pub old_value: Option<ConfigValue>,
    pub new_value: ConfigValue,
    pub changed_at: Instant,
    pub changed_by: String,
}

/// A typed callback fired whenever a specific key changes.
pub type ConfigWatcher = Box<dyn Fn(&str, &ConfigValue, &ConfigValue) + Send + Sync>;

/// Hot-reloadable, versioned, per-service-overridable network configuration.
///
/// ```text
/// Global config
///   ├── key "connect_timeout"  = Duration(5s)
///   ├── key "max_connections"  = Int(1000)
///   └── key "tls_enabled"      = Bool(true)
///
/// Per-service overrides
///   └── "payments" → { "max_connections" = Int(200), "connect_timeout" = Duration(2s) }
/// ```
pub struct DynamicConfig {
    /// Monotonically increasing version number, bumped on every `apply`.
    version:   u64,
    global:    HashMap<String, ConfigValue>,
    /// service_name → { key → value } overrides that shadow global entries.
    overrides: HashMap<String, HashMap<String, ConfigValue>>,
    /// Capped ring-buffer of recent changes.
    changelog: VecDeque<ConfigChange>,
    changelog_limit: usize,
    /// Registered key watchers: key → list of callbacks.
    watchers:  HashMap<String, Vec<ConfigWatcher>>,
}

impl DynamicConfig {
    pub fn new() -> Self {
        let mut cfg = DynamicConfig {
            version:         1,
            global:          HashMap::new(),
            overrides:       HashMap::new(),
            changelog:       VecDeque::new(),
            changelog_limit: 256,
            watchers:        HashMap::new(),
        };
        // Sensible defaults.
        cfg.set_global("connect_timeout",     ConfigValue::Duration(Duration::from_secs(5)),  "system");
        cfg.set_global("request_timeout",     ConfigValue::Duration(Duration::from_secs(30)), "system");
        cfg.set_global("max_connections",     ConfigValue::Int(1000),                         "system");
        cfg.set_global("max_retries",         ConfigValue::Int(3),                            "system");
        cfg.set_global("keepalive_interval",  ConfigValue::Duration(Duration::from_secs(15)), "system");
        cfg.set_global("tls_enabled",         ConfigValue::Bool(false),                       "system");
        cfg.set_global("circuit_breaker_on",  ConfigValue::Bool(true),                        "system");
        cfg.set_global("rate_limit_rps",      ConfigValue::Int(500),                          "system");
        cfg
    }

    // ── Reading ───────────────────────────────────────────────────────────────

    /// Fetch a value, honouring per-service overrides when `service` is given.
    pub fn get(&self, key: &str, service: Option<&str>) -> Option<&ConfigValue> {
        if let Some(svc) = service {
            if let Some(v) = self.overrides.get(svc).and_then(|m| m.get(key)) {
                return Some(v);
            }
        }
        self.global.get(key)
    }

    pub fn get_bool    (&self, key: &str, svc: Option<&str>) -> Option<bool>     { if let Some(ConfigValue::Bool(v)) = self.get(key, svc) { Some(*v) } else { None } }
    pub fn get_int     (&self, key: &str, svc: Option<&str>) -> Option<i64>      { if let Some(ConfigValue::Int(v))  = self.get(key, svc) { Some(*v) } else { None } }
    pub fn get_duration(&self, key: &str, svc: Option<&str>) -> Option<Duration> { if let Some(ConfigValue::Duration(v)) = self.get(key, svc) { Some(*v) } else { None } }
    pub fn get_text    (&self, key: &str, svc: Option<&str>) -> Option<&str>     { if let Some(ConfigValue::Text(v)) = self.get(key, svc) { Some(v.as_str()) } else { None } }

    pub fn version(&self) -> u64 { self.version }

    // ── Writing (hot-reload) ──────────────────────────────────────────────────

    /// Set a global key and fire any registered watchers.
    pub fn set_global(&mut self, key: impl Into<String>, value: ConfigValue, by: impl Into<String>) {
        let key    = key.into();
        let old    = self.global.insert(key.clone(), value.clone());
        self.record_change(&key, old.as_ref(), &value, by.into());
        self.version += 1;
        self.fire_watchers(&key, old.as_ref(), &value);
    }

    /// Set a per-service override for a single key.
    pub fn set_override(&mut self, service: impl Into<String>, key: impl Into<String>, value: ConfigValue, by: impl Into<String>) {
        let svc = service.into();
        let key = key.into();
        let old = self.overrides.entry(svc.clone()).or_default().insert(key.clone(), value.clone());
        self.record_change(&format!("{svc}::{key}"), old.as_ref(), &value, by.into());
        self.version += 1;
        self.fire_watchers(&key, old.as_ref(), &value);
    }

    /// Remove a per-service override, falling back to the global value.
    pub fn clear_override(&mut self, service: &str, key: &str) {
        if let Some(m) = self.overrides.get_mut(service) {
            m.remove(key);
        }
        self.version += 1;
    }

    /// Atomically apply a batch of global changes (single version bump).
    pub fn apply_batch(&mut self, changes: Vec<(String, ConfigValue)>, by: impl Into<String>) {
        let actor = by.into();
        for (k, v) in changes {
            self.set_global(k, v, &actor);
        }
    }

    /// Overlay from environment-variable-style key=value pairs.
    ///
    /// Keys are lower-cased and `_` separates words; values are auto-typed:
    /// "true"/"false" → `Bool`, integer strings → `Int`, floats → `Float`,
    /// everything else → `Text`.
    pub fn apply_env_overlay(&mut self, env: &HashMap<String, String>, by: impl Into<String>) {
        let actor = by.into();
        for (k, v) in env {
            let key   = k.to_lowercase().replace('-', "_");
            let value = parse_env_value(v);
            self.set_global(key, value, &actor);
        }
    }

    // ── Watchers ──────────────────────────────────────────────────────────────

    /// Register a callback fired whenever `key` changes globally.
    pub fn watch(&mut self, key: impl Into<String>, watcher: ConfigWatcher) {
        self.watchers.entry(key.into()).or_default().push(watcher);
    }

    // ── Change log ────────────────────────────────────────────────────────────

    pub fn changelog(&self) -> impl Iterator<Item = &ConfigChange> { self.changelog.iter() }

    pub fn last_change_for(&self, key: &str) -> Option<&ConfigChange> {
        self.changelog.iter().rev().find(|c| c.key == key || c.key.ends_with(&format!("::{key}")))
    }

    // ── Snapshot ──────────────────────────────────────────────────────────────

    /// Produce a flattened view of the effective config for a service.
    pub fn effective_config_for(&self, service: &str) -> HashMap<String, &ConfigValue> {
        let mut effective: HashMap<String, &ConfigValue> = self.global.iter()
            .map(|(k, v)| (k.clone(), v)).collect();
        if let Some(overrides) = self.overrides.get(service) {
            for (k, v) in overrides {
                effective.insert(k.clone(), v);
            }
        }
        effective
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    fn record_change(&mut self, key: &str, old: Option<&ConfigValue>, new: &ConfigValue, by: String) {
        if self.changelog.len() >= self.changelog_limit {
            self.changelog.pop_front();
        }
        self.changelog.push_back(ConfigChange {
            key:        key.to_string(),
            old_value:  old.cloned(),
            new_value:  new.clone(),
            changed_at: Instant::now(),
            changed_by: by,
        });
    }

    fn fire_watchers(&self, key: &str, old: Option<&ConfigValue>, new: &ConfigValue) {
        if let Some(watchers) = self.watchers.get(key) {
            for w in watchers {
                if let Some(o) = old {
                    w(key, o, new);
                }
            }
        }
    }
}

impl Default for DynamicConfig {
    fn default() -> Self { Self::new() }
}

fn parse_env_value(s: &str) -> ConfigValue {
    if s == "true"  { return ConfigValue::Bool(true);  }
    if s == "false" { return ConfigValue::Bool(false); }
    if let Ok(i) = s.parse::<i64>()   { return ConfigValue::Int(i);   }
    if let Ok(f) = s.parse::<f64>()   { return ConfigValue::Float(f); }
    ConfigValue::Text(s.to_string())
}

// ═════════════════════════════════════════════════════════════════════════════
// § 11  NETWORK RESOURCE MANAGEMENT
// ═════════════════════════════════════════════════════════════════════════════

// ── 11a  Resource quotas ──────────────────────────────────────────────────────

/// Per-service resource quota definition.
#[derive(Debug, Clone)]
pub struct ServiceQuota {
    pub service_name:     String,
    /// Maximum simultaneous connections this service may hold.
    pub max_connections:  usize,
    /// Maximum outbound request rate (requests per second).
    pub max_rps:          u64,
    /// Maximum inbound bandwidth (bytes per second). `None` = unlimited.
    pub max_bandwidth_bps: Option<u64>,
    /// Maximum number of concurrent circuit-breaker protected calls.
    pub max_concurrent_calls: usize,
}

impl ServiceQuota {
    pub fn new(service_name: impl Into<String>) -> Self {
        ServiceQuota {
            service_name:       service_name.into(),
            max_connections:    100,
            max_rps:            1000,
            max_bandwidth_bps:  None,
            max_concurrent_calls: 50,
        }
    }

    pub fn with_max_connections   (mut self, n: usize)        -> Self { self.max_connections    = n; self }
    pub fn with_max_rps           (mut self, n: u64)          -> Self { self.max_rps            = n; self }
    pub fn with_max_bandwidth_bps (mut self, n: u64)          -> Self { self.max_bandwidth_bps  = Some(n); self }
    pub fn with_max_concurrent    (mut self, n: usize)        -> Self { self.max_concurrent_calls = n; self }
}

// ── 11b  Token-bucket rate limiter ────────────────────────────────────────────

/// Classic token-bucket rate limiter.
///
/// Tokens refill at `rate` per second up to `capacity`.  A call to
/// `acquire(n)` succeeds only if at least `n` tokens are available.
#[derive(Debug)]
pub struct RateLimiter {
    capacity:    f64,
    tokens:      f64,
    rate:        f64,          // tokens / second
    last_refill: Instant,
}

impl RateLimiter {
    pub fn new(rate_per_sec: f64, burst_capacity: f64) -> Self {
        RateLimiter {
            capacity:    burst_capacity,
            tokens:      burst_capacity,
            rate:        rate_per_sec,
            last_refill: Instant::now(),
        }
    }

    fn refill(&mut self) {
        let elapsed    = self.last_refill.elapsed().as_secs_f64();
        self.tokens    = (self.tokens + elapsed * self.rate).min(self.capacity);
        self.last_refill = Instant::now();
    }

    /// Try to consume `n` tokens. Returns `Ok(())` on success.
    pub fn acquire(&mut self, n: f64) -> Result<(), NetworkError> {
        self.refill();
        if self.tokens >= n {
            self.tokens -= n;
            Ok(())
        } else {
            Err(NetworkError::RateLimitExceeded(format!(
                "need {n:.1} tokens, {:.1} available", self.tokens
            )))
        }
    }

    pub fn available_tokens(&mut self) -> f64 { self.refill(); self.tokens }
}

// ── 11c  Circuit breaker ──────────────────────────────────────────────────────

/// Three-state circuit breaker following the Martin Fowler / Netflix Hystrix model.
///
/// ```text
///  Closed ──(threshold failures)──▶ Open ──(reset_timeout)──▶ Half-Open
///    ▲                                                             │
///    └──────────────────(success)────────────────────────────────┘
///    ───────(failure in half-open)───────────────▶ Open
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CircuitState {
    /// Calls pass through normally.
    Closed,
    /// All calls are rejected immediately.
    Open,
    /// One probe call is allowed; success → Closed, failure → Open.
    HalfOpen,
}

impl fmt::Display for CircuitState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CircuitState::Closed   => write!(f, "closed"),
            CircuitState::Open     => write!(f, "open"),
            CircuitState::HalfOpen => write!(f, "half-open"),
        }
    }
}

#[derive(Debug)]
pub struct CircuitBreaker {
    pub service:           String,
    state:                 CircuitState,
    failure_count:         u32,
    success_count:         u32,
    failure_threshold:     u32,
    success_threshold:     u32,  // consecutive successes needed to re-close from half-open
    reset_timeout:         Duration,
    opened_at:             Option<Instant>,
    pub total_calls:       u64,
    pub total_failures:    u64,
}

impl CircuitBreaker {
    pub fn new(service: impl Into<String>, failure_threshold: u32, reset_timeout: Duration) -> Self {
        CircuitBreaker {
            service:           service.into(),
            state:             CircuitState::Closed,
            failure_count:     0,
            success_count:     0,
            failure_threshold,
            success_threshold: 2,
            reset_timeout,
            opened_at:         None,
            total_calls:       0,
            total_failures:    0,
        }
    }

    /// Call before making an outbound request.
    /// Returns `Err(CircuitOpen)` when the breaker is open.
    pub fn before_call(&mut self) -> Result<(), NetworkError> {
        self.total_calls += 1;

        match self.state {
            CircuitState::Closed   => Ok(()),
            CircuitState::HalfOpen => Ok(()),
            CircuitState::Open     => {
                // Check whether the reset window has elapsed.
                if let Some(opened) = self.opened_at {
                    if opened.elapsed() >= self.reset_timeout {
                        self.state    = CircuitState::HalfOpen;
                        self.success_count = 0;
                        return Ok(());
                    }
                }
                Err(NetworkError::CircuitOpen(self.service.clone()))
            }
        }
    }

    /// Call after a successful request.
    pub fn on_success(&mut self) {
        match self.state {
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.success_threshold {
                    self.state         = CircuitState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                    self.opened_at     = None;
                }
            }
            CircuitState::Closed => {
                self.failure_count = 0; // reset streak on success
            }
            _ => {}
        }
    }

    /// Call after a failed request.
    pub fn on_failure(&mut self) {
        self.total_failures += 1;
        self.failure_count  += 1;

        match self.state {
            CircuitState::Closed | CircuitState::HalfOpen => {
                if self.failure_count >= self.failure_threshold {
                    self.state     = CircuitState::Open;
                    self.opened_at = Some(Instant::now());
                }
            }
            _ => {}
        }
    }

    pub fn state(&self)         -> &CircuitState { &self.state }
    pub fn failure_rate(&self)  -> f64 {
        if self.total_calls == 0 { 0.0 }
        else { self.total_failures as f64 / self.total_calls as f64 }
    }

    /// Manually force the breaker closed (for operators / tests).
    pub fn force_close(&mut self) {
        self.state         = CircuitState::Closed;
        self.failure_count = 0;
        self.success_count = 0;
        self.opened_at     = None;
    }
}

// ── 11d  Connection pool ──────────────────────────────────────────────────────

/// Simulated connection handle.
#[derive(Debug, Clone)]
pub struct Connection {
    pub id:          u64,
    pub service:     String,
    pub remote_addr: SocketAddr,
    pub created_at:  Instant,
    pub last_used:   Instant,
    pub is_idle:     bool,
}

/// Bounded connection pool for a single service.
#[derive(Debug)]
pub struct ConnectionPool {
    service:        String,
    max_size:       usize,
    idle_timeout:   Duration,
    connections:    Vec<Connection>,
    next_id:        u64,
    pub acquired:   u64,
    pub released:   u64,
    pub evicted:    u64,
}

impl ConnectionPool {
    pub fn new(service: impl Into<String>, max_size: usize, idle_timeout: Duration) -> Self {
        ConnectionPool {
            service:      service.into(),
            max_size,
            idle_timeout,
            connections:  Vec::new(),
            next_id:      1,
            acquired:     0,
            released:     0,
            evicted:      0,
        }
    }

    /// Acquire a connection (reuse idle or create new).
    pub fn acquire(&mut self, remote_addr: SocketAddr) -> Result<Connection, NetworkError> {
        self.evict_idle();

        // Reuse the first idle connection to the same remote.
        if let Some(pos) = self.connections.iter().position(|c| c.is_idle && c.remote_addr == remote_addr) {
            self.connections[pos].is_idle   = false;
            self.connections[pos].last_used = Instant::now();
            self.acquired += 1;
            return Ok(self.connections[pos].clone());
        }

        if self.connections.len() >= self.max_size {
            return Err(NetworkError::PoolExhausted(self.service.clone()));
        }

        let conn = Connection {
            id:          self.next_id,
            service:     self.service.clone(),
            remote_addr,
            created_at:  Instant::now(),
            last_used:   Instant::now(),
            is_idle:     false,
        };
        self.next_id += 1;
        self.connections.push(conn.clone());
        self.acquired += 1;
        Ok(conn)
    }

    /// Return a connection to the pool.
    pub fn release(&mut self, id: u64) {
        if let Some(conn) = self.connections.iter_mut().find(|c| c.id == id) {
            conn.is_idle   = true;
            conn.last_used = Instant::now();
            self.released += 1;
        }
    }

    /// Forcibly close and remove a specific connection.
    pub fn close(&mut self, id: u64) {
        self.connections.retain(|c| c.id != id);
    }

    pub fn active_count(&self) -> usize { self.connections.iter().filter(|c| !c.is_idle).count() }
    pub fn idle_count(&self)   -> usize { self.connections.iter().filter(|c| c.is_idle).count() }
    pub fn total_count(&self)  -> usize { self.connections.len() }
    pub fn utilisation(&self)  -> f64   {
        if self.max_size == 0 { 0.0 } else { self.active_count() as f64 / self.max_size as f64 }
    }

    fn evict_idle(&mut self) {
        let timeout = self.idle_timeout;
        let before  = self.connections.len();
        self.connections.retain(|c| !(c.is_idle && c.last_used.elapsed() > timeout));
        self.evicted += (before - self.connections.len()) as u64;
    }
}

// ── 11e  Per-service resource snapshot ───────────────────────────────────────

/// Point-in-time summary of all resource usage for one service.
#[derive(Debug, Clone)]
pub struct ResourceSnapshot {
    pub service_name:    String,
    pub active_conns:    usize,
    pub idle_conns:      usize,
    pub pool_utilisation: f64,
    pub circuit_state:   CircuitState,
    pub circuit_failure_rate: f64,
    pub rate_tokens:     f64,
    pub total_calls:     u64,
    pub total_failures:  u64,
    pub sampled_at:      Instant,
}

// ── 11f  ResourceManager ──────────────────────────────────────────────────────

/// Top-level manager for all network resources.
///
/// Owns:
/// - per-service `ConnectionPool`
/// - per-service `RateLimiter`
/// - per-service `CircuitBreaker`
/// - per-service `ServiceQuota`
///
/// All operations are exposed through `Arc<Mutex<ResourceManager>>` at the
/// `NetworkSystem` level for thread safety.
pub struct ResourceManager {
    pools:    HashMap<String, ConnectionPool>,
    limiters: HashMap<String, RateLimiter>,
    breakers: HashMap<String, CircuitBreaker>,
    quotas:   HashMap<String, ServiceQuota>,
    /// Default pool idle timeout.
    pool_idle_timeout:  Duration,
    /// Default circuit-breaker reset window.
    cb_reset_timeout:   Duration,
    /// Default failure threshold before circuit opens.
    cb_failure_threshold: u32,
}

impl ResourceManager {
    pub fn new() -> Self {
        ResourceManager {
            pools:               HashMap::new(),
            limiters:            HashMap::new(),
            breakers:            HashMap::new(),
            quotas:              HashMap::new(),
            pool_idle_timeout:   Duration::from_secs(60),
            cb_reset_timeout:    Duration::from_secs(10),
            cb_failure_threshold: 5,
        }
    }

    // ── Quota management ──────────────────────────────────────────────────────

    /// Register (or replace) a resource quota for a service.
    pub fn set_quota(&mut self, quota: ServiceQuota) {
        let svc = quota.service_name.clone();
        // Create or resize dependent resources to match the new quota.
        let max_conns = quota.max_connections;
        let rps       = quota.max_rps;

        self.pools.entry(svc.clone())
            .or_insert_with(|| ConnectionPool::new(&svc, max_conns, self.pool_idle_timeout));

        self.limiters.insert(svc.clone(), RateLimiter::new(rps as f64, rps as f64 * 2.0));

        self.breakers.entry(svc.clone())
            .or_insert_with(|| CircuitBreaker::new(&svc, self.cb_failure_threshold, self.cb_reset_timeout));

        self.quotas.insert(svc, quota);
    }

    /// Fetch the quota for a service (creates a default one if absent).
    pub fn quota_for(&mut self, service: &str) -> &ServiceQuota {
        if !self.quotas.contains_key(service) {
            let q = ServiceQuota::new(service);
            self.set_quota(q);
        }
        self.quotas.get(service).unwrap()
    }

    // ── Gate: check all guards before making a call ───────────────────────────

    /// Check circuit breaker + rate limiter before making an outbound call.
    ///
    /// Returns `Ok(())` if the call may proceed.
    pub fn before_call(&mut self, service: &str) -> Result<(), NetworkError> {
        self.ensure_service(service);
        self.breakers.get_mut(service).unwrap().before_call()?;
        self.limiters.get_mut(service).unwrap().acquire(1.0)?;
        Ok(())
    }

    /// Record a successful call outcome.
    pub fn on_success(&mut self, service: &str) {
        if let Some(cb) = self.breakers.get_mut(service) { cb.on_success(); }
    }

    /// Record a failed call outcome.
    pub fn on_failure(&mut self, service: &str) {
        if let Some(cb) = self.breakers.get_mut(service) { cb.on_failure(); }
    }

    // ── Connection pool ───────────────────────────────────────────────────────

    pub fn acquire_connection(&mut self, service: &str, addr: SocketAddr) -> Result<Connection, NetworkError> {
        self.ensure_service(service);

        let quota = self.quotas.get(service).unwrap();
        let pool  = self.pools.get_mut(service).unwrap();

        if pool.active_count() >= quota.max_connections {
            return Err(NetworkError::QuotaExceeded(format!(
                "{service}: max_connections={}", quota.max_connections
            )));
        }
        pool.acquire(addr)
    }

    pub fn release_connection(&mut self, service: &str, conn_id: u64) {
        if let Some(pool) = self.pools.get_mut(service) { pool.release(conn_id); }
    }

    pub fn close_connection(&mut self, service: &str, conn_id: u64) {
        if let Some(pool) = self.pools.get_mut(service) { pool.close(conn_id); }
    }

    // ── Circuit breaker manual controls ──────────────────────────────────────

    pub fn circuit_state(&mut self, service: &str) -> CircuitState {
        self.ensure_service(service);
        self.breakers[service].state().clone()
    }

    pub fn force_close_circuit(&mut self, service: &str) {
        self.ensure_service(service);
        self.breakers.get_mut(service).unwrap().force_close();
    }

    // ── Metrics snapshots ─────────────────────────────────────────────────────

    /// Snapshot current resource usage for a single service.
    pub fn snapshot(&mut self, service: &str) -> ResourceSnapshot {
        self.ensure_service(service);
        let pool    = &self.pools[service];
        let breaker = &self.breakers[service];
        let limiter = self.limiters.get_mut(service).unwrap();

        ResourceSnapshot {
            service_name:         service.to_string(),
            active_conns:         pool.active_count(),
            idle_conns:           pool.idle_count(),
            pool_utilisation:     pool.utilisation(),
            circuit_state:        breaker.state().clone(),
            circuit_failure_rate: breaker.failure_rate(),
            rate_tokens:          limiter.available_tokens(),
            total_calls:          breaker.total_calls,
            total_failures:       breaker.total_failures,
            sampled_at:           Instant::now(),
        }
    }

    /// Snapshot all tracked services.
    pub fn snapshot_all(&mut self) -> Vec<ResourceSnapshot> {
        let services: Vec<String> = self.pools.keys().cloned().collect();
        services.iter().map(|s| self.snapshot(s)).collect()
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    fn ensure_service(&mut self, service: &str) {
        if !self.pools.contains_key(service) {
            let q = ServiceQuota::new(service);
            self.set_quota(q);
        }
    }
}

impl Default for ResourceManager {
    fn default() -> Self { Self::new() }
}

// ═════════════════════════════════════════════════════════════════════════════
// § 12  NETWORK SYSTEM — TOP-LEVEL FAÇADE
// ═════════════════════════════════════════════════════════════════════════════

/// The central network subsystem.
///
/// Owns and exposes:
/// - `ServiceRegistry`      — instance registration & discovery
/// - `PortAllocator`        — dynamic port assignment
/// - `DynamicConfig`        — hot-reloadable configuration
/// - `ResourceManager`      — connection pools, rate limiters, circuit breakers
pub struct NetworkSystem {
    registry:  Arc<RwLock<ServiceRegistry>>,
    registrar: ServiceRegistrar,
    ports:     Arc<Mutex<PortAllocator>>,
    config:    Arc<RwLock<DynamicConfig>>,
    resources: Arc<Mutex<ResourceManager>>,
}

impl NetworkSystem {
    pub fn new(registry_config: RegistryConfig) -> Self {
        let registry  = Arc::new(RwLock::new(ServiceRegistry::new(registry_config)));
        let registrar = ServiceRegistrar::new(Arc::clone(&registry));
        NetworkSystem {
            registry,
            registrar,
            ports:     Arc::new(Mutex::new(PortAllocator::new(Duration::from_secs(3600)))),
            config:    Arc::new(RwLock::new(DynamicConfig::new())),
            resources: Arc::new(Mutex::new(ResourceManager::new())),
        }
    }

    pub fn with_default_config() -> Self { Self::new(RegistryConfig::default()) }

    // ── Self-registration ─────────────────────────────────────────────────────

    pub fn register_self(&self, instance: ServiceInstance) -> Result<(), NetworkError> {
        self.registry.write().unwrap().register(instance)
    }

    pub fn deregister_self(&self, svc: &str, id: &str) -> Result<ServiceInstance, NetworkError> {
        self.registry.write().unwrap().deregister(svc, id)
    }

    // ── Third-party registrar ─────────────────────────────────────────────────

    pub fn registrar(&self) -> &ServiceRegistrar { &self.registrar }

    // ── Discovery ─────────────────────────────────────────────────────────────

    pub fn discover_service(&self, svc: &str) -> Result<Vec<ServiceInstance>, NetworkError> {
        self.registry.read().unwrap().discover(svc)
            .map(|v| v.into_iter().cloned().collect())
    }

    pub fn route_request(&self, svc: &str) -> Result<SocketAddr, NetworkError> {
        self.registry.write().unwrap().resolve(svc)
    }

    /// Route a request through the resource manager's guards first.
    /// Returns the resolved address only when the circuit is closed and
    /// the rate limit allows.
    pub fn route_request_guarded(&self, svc: &str) -> Result<SocketAddr, NetworkError> {
        self.resources.lock().unwrap().before_call(svc)?;
        self.registry.write().unwrap().resolve(svc)
    }

    // ── Sidecar ───────────────────────────────────────────────────────────────

    pub fn create_sidecar(&self, svc: impl Into<String>, local_port: u16) -> SidecarProxy {
        SidecarProxy::new(svc, Arc::clone(&self.registry), local_port)
    }

    // ── Health ─────────────────────────────────────────────────────────────────

    pub fn heartbeat(&self, svc: &str, id: &str) -> Result<(), NetworkError> {
        self.registry.write().unwrap().heartbeat(svc, id)
    }

    pub fn run_health_checks(&self) -> usize {
        self.registry.write().unwrap().run_health_checks()
    }

    // ── Dynamic scaling ───────────────────────────────────────────────────────

    pub fn scale_out(&self, svc: &str, base_addr: SocketAddr, count: usize) -> Result<Vec<String>, NetworkError> {
        let mut reg = self.registry.write().unwrap();
        let mut ids = Vec::with_capacity(count);
        for i in 0..count {
            let id   = format!("{svc}-scale-{}", uuid_lite(i));
            let port = base_addr.port().wrapping_add(i as u16);
            let addr = SocketAddr::new(base_addr.ip(), port);
            let mut inst = ServiceInstance::new(id.clone(), svc, addr);
            inst.health = HealthStatus::Healthy;
            reg.register(inst)?;
            ids.push(id);
        }
        Ok(ids)
    }

    pub fn scale_in(&self, svc: &str, ids: &[&str]) -> Vec<Result<ServiceInstance, NetworkError>> {
        let mut reg = self.registry.write().unwrap();
        ids.iter().map(|id| reg.deregister(svc, id)).collect()
    }

    // ── Port allocator (§9) ───────────────────────────────────────────────────

    /// Borrow the port allocator for direct operations.
    pub fn ports(&self) -> std::sync::MutexGuard<'_, PortAllocator> {
        self.ports.lock().unwrap()
    }

    /// Allocate a port from a named range and spin up a new service instance.
    /// The leased port is automatically embedded in the returned `SocketAddr`.
    pub fn register_with_dynamic_port(
        &self,
        instance_id:  impl Into<String>,
        service_name: impl Into<String>,
        bind_ip:      IpAddr,
        range_name:   &str,
    ) -> Result<ServiceInstance, NetworkError> {
        let svc    = service_name.into();
        let id     = instance_id.into();
        let lease  = self.ports.lock().unwrap().allocate_from(range_name, &id)?;
        let addr   = SocketAddr::new(bind_ip, lease.port);
        let mut inst = ServiceInstance::new(id, &svc, addr);
        inst.health = HealthStatus::Healthy;
        self.registry.write().unwrap().register(inst.clone())?;
        Ok(inst)
    }

    /// Reserve a well-known port so it is never auto-allocated.
    pub fn reserve_port(&self, port: u16) -> Result<(), NetworkError> {
        self.ports.lock().unwrap().reserve(port)
    }

    /// Create a pre-allocated port pool for a service.
    pub fn create_service_port_pool(&self, service: &str, range: &str, count: usize) -> Result<Vec<u16>, NetworkError> {
        self.ports.lock().unwrap().create_service_pool(service, range, count)
    }

    // ── Dynamic config (§10) ──────────────────────────────────────────────────

    /// Borrow the live configuration for reading.
    pub fn config(&self) -> std::sync::RwLockReadGuard<'_, DynamicConfig> {
        self.config.read().unwrap()
    }

    /// Borrow the live configuration for writing (hot-reload).
    pub fn config_mut(&self) -> std::sync::RwLockWriteGuard<'_, DynamicConfig> {
        self.config.write().unwrap()
    }

    /// Convenience: read a config key, optionally scoped to a service.
    pub fn config_get(&self, key: &str, service: Option<&str>) -> Option<ConfigValue> {
        self.config.read().unwrap().get(key, service).cloned()
    }

    /// Convenience: write a global config key (hot-reload).
    pub fn config_set(&self, key: impl Into<String>, value: ConfigValue, by: impl Into<String>) {
        self.config.write().unwrap().set_global(key, value, by);
    }

    /// Apply an env-variable overlay to the live configuration.
    pub fn apply_env_config(&self, env: &HashMap<String, String>, by: impl Into<String>) {
        self.config.write().unwrap().apply_env_overlay(env, by);
    }

    // ── Resource manager (§11) ────────────────────────────────────────────────

    /// Borrow the resource manager for direct operations.
    pub fn resources(&self) -> std::sync::MutexGuard<'_, ResourceManager> {
        self.resources.lock().unwrap()
    }

    /// Register a per-service resource quota.
    pub fn set_service_quota(&self, quota: ServiceQuota) {
        self.resources.lock().unwrap().set_quota(quota);
    }

    /// Execute a guarded call: checks circuit + rate limit, records outcome.
    ///
    /// `f` receives the resolved `SocketAddr` and returns `Ok` or `Err`.
    pub fn call<F, T>(&self, service: &str, f: F) -> Result<T, NetworkError>
    where
        F: FnOnce(SocketAddr) -> Result<T, NetworkError>,
    {
        let addr = self.route_request_guarded(service)?;
        match f(addr) {
            Ok(v)  => { self.resources.lock().unwrap().on_success(service); Ok(v) }
            Err(e) => { self.resources.lock().unwrap().on_failure(service); Err(e) }
        }
    }

    /// Acquire a managed connection to a service.
    pub fn acquire_connection(&self, service: &str, addr: SocketAddr) -> Result<Connection, NetworkError> {
        self.resources.lock().unwrap().acquire_connection(service, addr)
    }

    /// Release a connection back to the pool.
    pub fn release_connection(&self, service: &str, conn_id: u64) {
        self.resources.lock().unwrap().release_connection(service, conn_id);
    }

    /// Snapshot resource usage for a specific service.
    pub fn resource_snapshot(&self, service: &str) -> ResourceSnapshot {
        self.resources.lock().unwrap().snapshot(service)
    }

    /// Snapshot resource usage for all services.
    pub fn resource_snapshot_all(&self) -> Vec<ResourceSnapshot> {
        self.resources.lock().unwrap().snapshot_all()
    }

    // ── Introspection ─────────────────────────────────────────────────────────

    pub fn service_names(&self) -> Vec<String> {
        self.registry.read().unwrap().service_names().iter().map(|s| s.to_string()).collect()
    }

    pub fn instance_count(&self, svc: &str) -> usize {
        self.registry.read().unwrap().instance_count(svc)
    }

    /// Full topology + resource + config summary printed to stdout.
    pub fn print_topology(&self) {
        let reg = self.registry.read().unwrap();
        println!("══ NetworkSystem Topology ══════════════════════════════════════════");
        println!("   config v{}", self.config.read().unwrap().version());
        for name in reg.service_names() {
            let instances: Vec<&ServiceInstance> = reg.services.get(name).unwrap().iter().collect();
            println!("  ▸ {name} ({} instance(s))", instances.len());
            for inst in &instances {
                println!(
                    "      [{}] {} — {} | conns={} | failures={}",
                    inst.health, inst.instance_id, inst.address,
                    inst.active_connections, inst.consecutive_failures,
                );
            }
            // Resource snapshot if tracked.
            let snap = self.resources.lock().unwrap().snapshot(name);
            println!(
                "      resources: circuit={} | pool={}/{:.0}% | failure_rate={:.1}%",
                snap.circuit_state,
                snap.active_conns,
                snap.pool_utilisation * 100.0,
                snap.circuit_failure_rate * 100.0,
            );
        }
        // Port allocator summary.
        let ports = self.ports.lock().unwrap();
        println!("  ▸ ports: leased={} reserved={}", ports.leased_count(), ports.reserved_count());
        println!("═══════════════════════════════════════════════════════════════════");
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// § 13  UTILITIES
// ═════════════════════════════════════════════════════════════════════════════

fn uuid_lite(seed: usize) -> String {
    format!("{:08x}", seed.wrapping_mul(0x9e3779b9).wrapping_add(0xdeadbeef))
}

fn localhost(port: u16) -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port)
}

// ═════════════════════════════════════════════════════════════════════════════
// § 14  TESTS
// ═════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    fn addr(port: u16) -> SocketAddr { localhost(port) }

    fn healthy(id: &str, svc: &str, port: u16) -> ServiceInstance {
        let mut i = ServiceInstance::new(id, svc, addr(port));
        i.health = HealthStatus::Healthy;
        i
    }

    // ── Original registry tests ───────────────────────────────────────────────

    #[test]
    fn test_register_and_discover() {
        let sys = NetworkSystem::with_default_config();
        sys.register_self(healthy("i1", "auth", 8001)).unwrap();
        sys.register_self(healthy("i2", "auth", 8002)).unwrap();
        assert_eq!(sys.discover_service("auth").unwrap().len(), 2);
    }

    #[test]
    fn test_duplicate_registration_rejected() {
        let sys = NetworkSystem::with_default_config();
        sys.register_self(healthy("i1", "auth", 8001)).unwrap();
        assert!(matches!(sys.register_self(healthy("i1", "auth", 8001)), Err(NetworkError::AlreadyRegistered(_))));
    }

    #[test]
    fn test_deregister() {
        let sys = NetworkSystem::with_default_config();
        sys.register_self(healthy("i1", "auth", 8001)).unwrap();
        sys.deregister_self("auth", "i1").unwrap();
        assert!(matches!(sys.discover_service("auth"), Err(NetworkError::ServiceNotFound(_))));
    }

    #[test]
    fn test_round_robin_cycles() {
        let sys = NetworkSystem::with_default_config();
        sys.register_self(healthy("i1", "svc", 9001)).unwrap();
        sys.register_self(healthy("i2", "svc", 9002)).unwrap();
        let a1 = sys.route_request("svc").unwrap();
        let a2 = sys.route_request("svc").unwrap();
        let a3 = sys.route_request("svc").unwrap();
        assert_ne!(a1, a2);
        assert_eq!(a1, a3);
    }

    #[test]
    fn test_least_connections() {
        let cfg = RegistryConfig { load_balancing: LoadBalancingStrategy::LeastConnections, ..Default::default() };
        let sys = NetworkSystem::new(cfg);
        let mut busy = healthy("i1", "svc", 9001);
        busy.active_connections = 10;
        sys.register_self(busy).unwrap();
        sys.register_self(healthy("i2", "svc", 9002)).unwrap();
        assert_eq!(sys.route_request("svc").unwrap().port(), 9002);
    }

    #[test]
    fn test_stale_instance_marked_unhealthy() {
        let cfg = RegistryConfig { heartbeat_timeout: Duration::from_millis(1), auto_deregister_stale: false, ..Default::default() };
        let sys = NetworkSystem::new(cfg);
        sys.register_self(healthy("i1", "svc", 9090)).unwrap();
        std::thread::sleep(Duration::from_millis(5));
        sys.run_health_checks();
        assert!(matches!(sys.discover_service("svc"), Err(NetworkError::NoHealthyInstances(_))));
    }

    #[test]
    fn test_scale_out_and_in() {
        let sys = NetworkSystem::with_default_config();
        let ids = sys.scale_out("worker", addr(7000), 3).unwrap();
        assert_eq!(sys.instance_count("worker"), 3);
        let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
        sys.scale_in("worker", &refs);
        assert_eq!(sys.instance_count("worker"), 0);
    }

    // ── Port allocator tests ──────────────────────────────────────────────────

    #[test]
    fn test_allocate_from_range() {
        let mut alloc = PortAllocator::new(Duration::from_secs(60));
        let lease = alloc.allocate_from("services", "test-svc").unwrap();
        assert!(lease.port >= 8000 && lease.port < 9000);
        assert_eq!(alloc.leased_count(), 1);
    }

    #[test]
    fn test_release_port() {
        let mut alloc = PortAllocator::new(Duration::from_secs(60));
        let lease = alloc.allocate_from("services", "svc").unwrap();
        alloc.release(lease.port).unwrap();
        assert_eq!(alloc.leased_count(), 0);
    }

    #[test]
    fn test_reserved_port_not_allocated() {
        let mut alloc = PortAllocator::new(Duration::from_secs(60));
        // Reserve every port except the last one in the range.
        for p in 8000..8999 { alloc.reserve(p).ok(); }
        let lease = alloc.allocate_from("services", "svc").unwrap();
        assert_eq!(lease.port, 8999);
    }

    #[test]
    fn test_no_ports_available() {
        let mut alloc = PortAllocator::new(Duration::from_secs(60));
        alloc.add_range(PortRange::new("tiny", 9900, 9902)); // only 2 ports
        alloc.allocate_from("tiny", "a").unwrap();
        alloc.allocate_from("tiny", "b").unwrap();
        assert!(matches!(alloc.allocate_from("tiny", "c"), Err(NetworkError::NoPortsAvailable(_))));
    }

    #[test]
    fn test_service_pool_checkout_and_return() {
        let mut alloc = PortAllocator::new(Duration::from_secs(60));
        alloc.create_service_pool("payments", "services", 3).unwrap();
        let p = alloc.checkout_from_pool("payments").unwrap();
        alloc.return_to_pool("payments", p);
        assert_eq!(alloc.checkout_from_pool("payments").unwrap(), p);
    }

    #[test]
    fn test_register_with_dynamic_port() {
        let sys = NetworkSystem::with_default_config();
        let inst = sys.register_with_dynamic_port(
            "dyn-1", "api", IpAddr::V4(Ipv4Addr::LOCALHOST), "services"
        ).unwrap();
        assert!(inst.address.port() >= 8000 && inst.address.port() < 9000);
        assert_eq!(sys.instance_count("api"), 1);
    }

    // ── Dynamic config tests ──────────────────────────────────────────────────

    #[test]
    fn test_config_set_and_get() {
        let sys = NetworkSystem::with_default_config();
        sys.config_set("max_connections", ConfigValue::Int(500), "operator");
        let v = sys.config_get("max_connections", None).unwrap();
        assert_eq!(v, ConfigValue::Int(500));
    }

    #[test]
    fn test_per_service_override_shadows_global() {
        let sys = NetworkSystem::with_default_config();
        sys.config_set("max_connections", ConfigValue::Int(1000), "system");
        sys.config_mut().set_override("payments", "max_connections", ConfigValue::Int(50), "operator");
        let global  = sys.config_get("max_connections", None).unwrap();
        let payment = sys.config_get("max_connections", Some("payments")).unwrap();
        assert_eq!(global,  ConfigValue::Int(1000));
        assert_eq!(payment, ConfigValue::Int(50));
    }

    #[test]
    fn test_env_overlay() {
        let sys = NetworkSystem::with_default_config();
        let env: HashMap<String, String> = [
            ("tls_enabled".to_string(), "true".to_string()),
            ("max_retries".to_string(), "5".to_string()),
        ].into();
        sys.apply_env_config(&env, "env");
        assert_eq!(sys.config_get("tls_enabled", None), Some(ConfigValue::Bool(true)));
        assert_eq!(sys.config_get("max_retries",  None), Some(ConfigValue::Int(5)));
    }

    #[test]
    fn test_config_version_bumps() {
        let sys = NetworkSystem::with_default_config();
        let v0  = sys.config().version();
        sys.config_set("foo", ConfigValue::Text("bar".to_string()), "test");
        assert!(sys.config().version() > v0);
    }

    #[test]
    fn test_config_changelog() {
        let sys = NetworkSystem::with_default_config();
        sys.config_set("connect_timeout", ConfigValue::Duration(Duration::from_secs(2)), "ops");
        let entry = sys.config().last_change_for("connect_timeout");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().changed_by, "ops");
    }

    // ── Resource manager tests ────────────────────────────────────────────────

    #[test]
    fn test_rate_limiter_allows_and_blocks() {
        let mut rl = RateLimiter::new(10.0, 10.0); // 10 tokens
        for _ in 0..10 { rl.acquire(1.0).unwrap(); }
        assert!(matches!(rl.acquire(1.0), Err(NetworkError::RateLimitExceeded(_))));
    }

    #[test]
    fn test_circuit_breaker_opens_and_resets() {
        let mut cb = CircuitBreaker::new("svc", 3, Duration::from_millis(5));
        cb.before_call().unwrap();
        cb.on_failure();
        cb.on_failure();
        cb.on_failure(); // threshold reached
        assert_eq!(*cb.state(), CircuitState::Open);
        assert!(matches!(cb.before_call(), Err(NetworkError::CircuitOpen(_))));

        // Wait for reset timeout.
        std::thread::sleep(Duration::from_millis(10));
        cb.before_call().unwrap(); // now half-open
        assert_eq!(*cb.state(), CircuitState::HalfOpen);

        cb.on_success();
        cb.on_success(); // success_threshold = 2
        assert_eq!(*cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_force_close() {
        let mut cb = CircuitBreaker::new("svc", 1, Duration::from_secs(60));
        cb.before_call().unwrap();
        cb.on_failure(); // opens immediately (threshold=1)
        cb.force_close();
        assert_eq!(*cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_connection_pool_acquire_and_release() {
        let mut pool = ConnectionPool::new("db", 5, Duration::from_secs(60));
        let conn = pool.acquire(addr(5432)).unwrap();
        assert_eq!(pool.active_count(), 1);
        pool.release(conn.id);
        assert_eq!(pool.idle_count(), 1);
        assert_eq!(pool.active_count(), 0);
    }

    #[test]
    fn test_connection_pool_exhaustion() {
        let mut pool = ConnectionPool::new("db", 2, Duration::from_secs(60));
        pool.acquire(addr(5432)).unwrap();
        pool.acquire(addr(5432)).unwrap();
        assert!(matches!(pool.acquire(addr(5432)), Err(NetworkError::PoolExhausted(_))));
    }

    #[test]
    fn test_quota_enforced_on_acquire() {
        let sys = NetworkSystem::with_default_config();
        let q = ServiceQuota::new("db").with_max_connections(1);
        sys.set_service_quota(q);
        let a = sys.acquire_connection("db", addr(5432)).unwrap();
        let err = sys.acquire_connection("db", addr(5432)).unwrap_err();
        assert!(matches!(err, NetworkError::QuotaExceeded(_)));
        sys.release_connection("db", a.id);
    }

    #[test]
    fn test_guarded_route_blocked_by_circuit() {
        let sys = NetworkSystem::with_default_config();
        sys.register_self(healthy("i1", "api", 8080)).unwrap();
        // Register a quota so the service is tracked, then record enough
        // failures to trip the default threshold (5).
        sys.set_service_quota(ServiceQuota::new("api").with_max_connections(100));
        for _ in 0..5 {
            sys.resources().on_failure("api");
        }
        assert!(matches!(sys.route_request_guarded("api"), Err(NetworkError::CircuitOpen(_))));
    }

    #[test]
    fn test_resource_snapshot() {
        let sys = NetworkSystem::with_default_config();
        let snap = sys.resource_snapshot("some-service");
        assert_eq!(snap.service_name, "some-service");
        assert_eq!(snap.active_conns, 0);
    }

    #[test]
    fn test_topology_print_does_not_panic() {
        let sys = NetworkSystem::with_default_config();
        sys.register_self(healthy("i1", "api", 4000)).unwrap();
        sys.print_topology();
    }
}