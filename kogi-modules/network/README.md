# `network.rs` — NetworkSystem & Service Registry

A self-contained Rust module implementing a full service-discovery stack: a thread-safe service registry, multiple discovery patterns, pluggable load balancing, heartbeat-based health monitoring, dynamic scaling, and a sidecar proxy abstraction.

---

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Core Types](#core-types)
- [Service Registration](#service-registration)
- [Service Discovery](#service-discovery)
- [Load Balancing](#load-balancing)
- [Health Management](#health-management)
- [Dynamic Scaling](#dynamic-scaling)
- [Sidecar Proxy](#sidecar-proxy)
- [Configuration](#configuration)
- [Error Handling](#error-handling)
- [Usage Examples](#usage-examples)
- [Testing](#testing)

---

## Overview

Modern distributed systems need a way to locate service instances without hardcoded addresses. `network.rs` solves this by providing:

- A **`ServiceRegistry`** — the single source of truth for all running instances
- A **`NetworkSystem`** facade — the entry point for all registration, discovery, and routing operations
- Both **client-side** and **server-side** discovery patterns
- A **`ServiceRegistrar`** for third-party / orchestrator-driven registration (Kubernetes-style)
- A **`SidecarProxy`** for language-agnostic, transparent service communication
- Automatic **health checking** with configurable failure thresholds
- **Dynamic scaling** helpers to add or remove instances at runtime

---

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                   NetworkSystem                     │
│                                                     │
│  ┌─────────────┐   ┌──────────────────────────┐    │
│  │ Registrar   │   │      ServiceRegistry      │    │
│  │ (3rd party) │──▶│  service_name → [inst…]  │    │
│  └─────────────┘   └──────────┬───────────────┘    │
│                               │                     │
│         ┌─────────────────────┼──────────────────┐  │
│         ▼                     ▼                  ▼  │
│   Client-Side            Server-Side          Sidecar│
│   Discovery              Discovery            Proxy  │
│   (caller queries)       (LB resolves)    (local     │
│                                            intercept)│
└─────────────────────────────────────────────────────┘
```

The registry is wrapped in `Arc<RwLock<_>>` so it is safe to share across threads. The `NetworkSystem` holds the canonical reference; the `ServiceRegistrar` and `SidecarProxy` each hold a clone of the same `Arc`.

---

## Core Types

### `ServiceInstance`

Represents a single running copy of a named service.

| Field | Type | Description |
|---|---|---|
| `instance_id` | `String` | Globally unique ID (e.g. UUID) |
| `service_name` | `String` | Logical service name (e.g. `"auth-service"`) |
| `address` | `SocketAddr` | Reachable network address |
| `metadata` | `HashMap<String, String>` | Arbitrary key/value tags (version, region…) |
| `health` | `HealthStatus` | Current health state |
| `last_heartbeat` | `Instant` | Timestamp of the last successful check |
| `active_connections` | `usize` | Open connections (used by least-connections LB) |
| `consecutive_failures` | `u32` | Failed health-check streak |

```rust
let instance = ServiceInstance::new("api-1", "api-gateway", "127.0.0.1:8080".parse().unwrap())
    .with_metadata("version", "2.1.0")
    .with_metadata("region", "us-east-1");
```

### `HealthStatus`

| Variant | Meaning |
|---|---|
| `Healthy` | Fully operational; receives traffic |
| `Unhealthy` | Failing checks; excluded from routing |
| `Unknown` | Newly registered; not yet verified |
| `Draining` | Graceful shutdown; finishes active work, no new traffic |

---

## Service Registration

### Self-Registration (Client-Side)

The service itself calls the registry on startup — it owns its own lifecycle.

```rust
let sys = NetworkSystem::with_default_config();

let mut instance = ServiceInstance::new("payment-1", "payment-service", addr);
instance.health = HealthStatus::Healthy;

sys.register_self(instance)?;
```

On graceful shutdown:

```rust
sys.deregister_self("payment-service", "payment-1")?;
```

### Third-Party Registration (Server-Side / Kubernetes-Style)

An external orchestrator registers and deregisters on behalf of services. This is the pattern used in container platforms where services may not control their own lifecycle.

```rust
let registrar = sys.registrar();

// Called when a new pod starts
registrar.on_instance_started(instance)?;

// Called when a pod terminates
registrar.on_instance_stopped("payment-service", "payment-1")?;
```

---

## Service Discovery

### Client-Side Discovery

The caller queries the registry directly and receives all routable instances, then chooses one itself.

```rust
let instances = sys.discover_service("payment-service")?;

// Caller picks an instance using its own logic
let target = instances.first().unwrap();
println!("Connecting to {}", target.address);
```

Returns `Err(NoHealthyInstances)` if every instance is unhealthy or draining.

### Server-Side Discovery (Router / Load Balancer)

The caller asks for a single resolved address. The registry applies the configured load-balancing strategy internally — the caller never sees the instance list.

```rust
let addr = sys.route_request("payment-service")?;
// addr is a SocketAddr, ready to connect to
```

This mirrors what HAProxy, Nginx, or an ingress controller does when fronting a service mesh.

---

## Load Balancing

Configured once in `RegistryConfig` and applied globally to all `resolve()` / `route_request()` calls.

| Strategy | Behaviour |
|---|---|
| `RoundRobin` *(default)* | Distributes requests evenly in rotation |
| `LeastConnections` | Routes to the instance with the lowest `active_connections` |
| `Random` | Uniform random selection |
| `IpHash` | Consistent routing by caller IP (falls back to round-robin until a real IP is threaded through) |

```rust
let config = RegistryConfig {
    load_balancing: LoadBalancingStrategy::LeastConnections,
    ..Default::default()
};
let sys = NetworkSystem::new(config);
```

---

## Health Management

### Heartbeats

Services (or their sidecars) send periodic heartbeats to stay alive in the registry.

```rust
// Called by the service or a health-check loop
sys.heartbeat("payment-service", "payment-1")?;
```

A successful heartbeat resets `consecutive_failures` and marks the instance `Healthy`.

### Health Check Sweeps

Call `run_health_checks()` on a timer (e.g. every 10 seconds). It checks every instance for heartbeat staleness and applies the failure threshold.

```rust
let degraded = sys.run_health_checks();
println!("{degraded} instance(s) quarantined or removed");
```

**What happens during a sweep:**

1. Instances whose last heartbeat is older than `heartbeat_timeout` receive a failure tick.
2. Once `consecutive_failures >= unhealthy_threshold`, the instance becomes `Unhealthy`.
3. If `auto_deregister_stale` is enabled, instances that have been unhealthy for more than `2 × heartbeat_timeout` are removed entirely.

### Graceful Drain

Before shutting down, signal an instance to stop receiving traffic while it finishes in-flight requests:

```rust
// registry-level drain (write lock required)
registry.drain_instance("payment-service", "payment-1")?;
```

---

## Dynamic Scaling

### Scale Out

Register `n` new instances under a service name, assigning sequential ports from a base address.

```rust
let new_ids = sys.scale_out("worker", "10.0.0.1:7000".parse().unwrap(), 5)?;
// Registers worker on ports 7000, 7001, 7002, 7003, 7004
```

### Scale In

Deregister a specific set of instances by ID.

```rust
let ids: Vec<&str> = new_ids.iter().map(String::as_str).collect();
let results = sys.scale_in("worker", &ids);
```

Returns one `Result` per ID so partial failures are visible.

---

## Sidecar Proxy

Creates a per-service local proxy that intercepts outbound calls, resolves the real upstream from the registry, and handles forwarding — keeping the service itself entirely discovery-agnostic.

```rust
let sidecar = sys.create_sidecar("inventory-service", 15001);

// The service just sends to localhost:15001
let (resolved_addr, response) = sidecar.forward_request("GET /stock/42")?;
```

The sidecar pattern allows services written in any language to benefit from registry-based discovery through a shared local proxy (Envoy / Linkerd style), rather than embedding discovery logic in every service.

---

## Configuration

`RegistryConfig` is passed once at construction time.

| Field | Type | Default | Description |
|---|---|---|---|
| `unhealthy_threshold` | `u32` | `3` | Consecutive failures before an instance is quarantined |
| `heartbeat_timeout` | `Duration` | `30s` | Silence duration before a failure is recorded |
| `load_balancing` | `LoadBalancingStrategy` | `RoundRobin` | Strategy used by `resolve()` |
| `auto_deregister_stale` | `bool` | `true` | Remove long-dead instances automatically |

```rust
let config = RegistryConfig {
    unhealthy_threshold: 5,
    heartbeat_timeout: Duration::from_secs(15),
    load_balancing: LoadBalancingStrategy::LeastConnections,
    auto_deregister_stale: true,
};
```

---

## Error Handling

All fallible operations return `Result<T, NetworkError>`.

| Variant | Trigger |
|---|---|
| `ServiceNotFound(name)` | No instances registered under `name` |
| `NoHealthyInstances(name)` | All instances are unhealthy/draining |
| `AlreadyRegistered(key)` | `instance_id` already exists under the service |
| `InstanceNotFound(id)` | `instance_id` not found for the given service |
| `HealthCheckFailed(msg)` | Health check returned a non-OK result |
| `InvalidConfiguration(msg)` | Bad config value supplied |

`NetworkError` implements `std::error::Error` and `Display`.

---

## Usage Examples

### Minimal Setup

```rust
use network::{NetworkSystem, ServiceInstance, HealthStatus};

let sys = NetworkSystem::with_default_config();

let mut inst = ServiceInstance::new("auth-1", "auth", "127.0.0.1:9000".parse().unwrap());
inst.health = HealthStatus::Healthy;

sys.register_self(inst).unwrap();

// Route a request
let addr = sys.route_request("auth").unwrap();
println!("→ {addr}");
```

### Multi-Instance with Health Loop

```rust
// Register several instances
for i in 0..3 {
    let mut inst = ServiceInstance::new(
        format!("svc-{i}"),
        "api",
        format!("10.0.0.{}:8080", i + 1).parse().unwrap(),
    );
    inst.health = HealthStatus::Healthy;
    sys.register_self(inst).unwrap();
}

// Simulate a heartbeat loop (run in a background task)
loop {
    sys.heartbeat("api", "svc-0").ok();
    sys.heartbeat("api", "svc-1").ok();
    sys.heartbeat("api", "svc-2").ok();
    sys.run_health_checks();
    std::thread::sleep(Duration::from_secs(10));
}
```

### Topology Snapshot

```rust
sys.print_topology();
// ══ NetworkSystem Topology ══════════════════════════════
//   ▸ api (3 instance(s))
//       [healthy] svc-0 10.0.0.1:8080 | conns=0 | failures=0
//       [healthy] svc-1 10.0.0.2:8080 | conns=2 | failures=0
//       [healthy] svc-2 10.0.0.3:8080 | conns=0 | failures=0
// ════════════════════════════════════════════════════════
```

---

## Testing

The module ships with 13 unit tests covering every major code path. Run them with:

```bash
cargo test
```

| Test | What it verifies |
|---|---|
| `test_register_and_discover` | Basic registration + discovery |
| `test_duplicate_registration_rejected` | Idempotency guard |
| `test_deregister` | Instance removal and empty-registry cleanup |
| `test_unknown_service_returns_error` | `ServiceNotFound` path |
| `test_no_healthy_instances_error` | `NoHealthyInstances` path |
| `test_round_robin_cycles` | RR cursor wraps correctly |
| `test_least_connections_strategy` | LB picks lowest-connection instance |
| `test_heartbeat_keeps_instance_healthy` | Heartbeat resets failures |
| `test_stale_instance_marked_unhealthy` | Timeout → `Unhealthy` |
| `test_scale_out_and_in` | Dynamic add/remove instances |
| `test_sidecar_forwards_to_resolved_address` | Sidecar resolves and echoes |
| `test_third_party_registrar` | Orchestrator-driven lifecycle |
| `test_print_topology_does_not_panic` | Smoke test for introspection |