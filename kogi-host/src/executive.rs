use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::kernel::{ComponentClass, FfiKernelStats, FfiResourceLimits, KernelBridge};
use crate::runtime::{ModuleRuntime, ResourceLimits};

// ─────────────────────────────────────────────────────────────────────────────
// HostError
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum HostError {
    Io(std::io::Error),
    InvalidManifest(String),
    Kernel(String),
    Service(String),
}

impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err)             => write!(f, "io error: {err}"),
            Self::InvalidManifest(msg) => write!(f, "invalid manifest: {msg}"),
            Self::Kernel(msg)          => write!(f, "kernel bridge error: {msg}"),
            Self::Service(msg)         => write!(f, "service error: {msg}"),
        }
    }
}

impl From<std::io::Error> for HostError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ComponentGroup
//
// This is the Rust-side classification used in the host's component registry.
// It maps to `ComponentClass` (the FFI enum) when crossing the kernel boundary.
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ComponentGroup {
    Kernel,
    Host,
    Server,
    Engine,
    Gateway,
    Service,
    Module,
}

impl ComponentGroup {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Kernel  => "kernel",
            Self::Host    => "host",
            Self::Server  => "server",
            Self::Engine  => "engine",
            Self::Gateway => "gateway",
            Self::Service => "service",
            Self::Module  => "module",
        }
    }

    /// Map to the FFI `ComponentClass` tag used by `kogi_kernel_register_component`.
    ///
    /// Gateway has no direct kernel class; it is registered as a `Module` so the
    /// kernel's resource accounting still tracks it without a dedicated class.
    pub fn to_kernel_class(&self) -> ComponentClass {
        match self {
            Self::Kernel  => ComponentClass::Kernel,
            Self::Host    => ComponentClass::Host,
            Self::Server  => ComponentClass::Server,
            Self::Engine  => ComponentClass::Engine,
            Self::Gateway => ComponentClass::Module,
            Self::Service => ComponentClass::Services,
            Self::Module  => ComponentClass::Module,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ResourceLimits → FfiResourceLimits conversion
// ─────────────────────────────────────────────────────────────────────────────

impl ResourceLimits {
    /// Convert to the `#[repr(C)]` struct required by the kernel FFI boundary.
    /// `max_resources` (internal) maps to `max_resource_units` (kernel ABI).
    pub fn to_ffi(&self) -> FfiResourceLimits {
        FfiResourceLimits {
            memory_limit_mb:    self.memory_limit_mb,
            max_processes:      self.max_processes,
            max_files:          self.max_files,
            max_resource_units: self.max_resources,
            max_connections:    32, // host-side default; kernel may override
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ComponentHealth
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct ComponentHealth {
    pub healthy:       bool,
    pub checked_at_ms: i64,
    pub details:       String,
}

// ─────────────────────────────────────────────────────────────────────────────
// ComponentRuntime
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct ComponentRuntime {
    pub id:                String,
    pub group:             ComponentGroup,
    pub endpoint:          String,
    pub network_manager:   String,
    pub limits:            ResourceLimits,
    pub managed_by_kernel: bool,
    pub active:            bool,
    pub metadata:          BTreeMap<String, String>,
    pub last_health:       Option<ComponentHealth>,
}

impl ComponentRuntime {
    fn new(
        id:              impl Into<String>,
        group:           ComponentGroup,
        endpoint:        impl Into<String>,
        network_manager: impl Into<String>,
        limits:          ResourceLimits,
    ) -> Self {
        Self {
            id:                id.into(),
            group,
            endpoint:          endpoint.into(),
            network_manager:   network_manager.into(),
            limits,
            managed_by_kernel: true,
            active:            true,
            metadata:          BTreeMap::new(),
            last_health:       None,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ModuleIsolationSnapshot
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct ModuleIsolationSnapshot {
    pub module_id:       String,
    pub memory_limit_mb: u64,
    pub memory_used_mb:  u64,
    pub process_limit:   u32,
    pub process_count:   u32,
    pub file_limit:      u32,
    pub file_count:      u32,
    pub resource_limit:  u32,
    pub resource_used:   u32,
    pub network_manager: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// HostExecutive
// ─────────────────────────────────────────────────────────────────────────────

pub struct HostExecutive<B: KernelBridge> {
    bridge:     B,
    runtimes:   BTreeMap<String, ModuleRuntime>,
    components: BTreeMap<String, ComponentRuntime>,
    booted:     bool,
}

impl<B: KernelBridge> HostExecutive<B> {
    pub fn new(bridge: B) -> Self {
        let mut executive = Self {
            bridge,
            runtimes:   BTreeMap::new(),
            components: BTreeMap::new(),
            booted:     false,
        };
        executive.register_core_components();
        executive
    }

    // ── Module loading ────────────────────────────────────────────────────────

    pub fn load_modules<P: AsRef<Path>>(&mut self, modules_root: P) -> Result<(), HostError> {
        let root = modules_root.as_ref();
        if !root.exists() {
            return Err(HostError::InvalidManifest(format!(
                "module root does not exist: {}",
                root.display()
            )));
        }

        for entry in fs::read_dir(root)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }

            let manifest_path = entry.path().join("module.yaml");
            if !manifest_path.exists() {
                continue;
            }

            let content = fs::read_to_string(&manifest_path)?;
            let runtime =
                ModuleRuntime::from_manifest(&content).map_err(HostError::InvalidManifest)?;

            let module_id  = runtime.id.clone();
            let module_kind = runtime.kind.clone();
            let endpoint   = resolve_service_endpoint(&runtime.entrypoint, &module_kind);

            let mut component = ComponentRuntime::new(
                module_id.clone(),
                ComponentGroup::Module,
                endpoint,
                runtime.network_manager.clone(),
                runtime.limits.clone(),
            );
            component.active = runtime.status != "disabled";
            component.metadata.insert("entrypoint".to_string(),   runtime.entrypoint.clone());
            component.metadata.insert("module_kind".to_string(),  module_kind.clone());
            component.metadata.insert("language".to_string(),     runtime.language.clone());
            component.metadata.insert("status".to_string(),       runtime.status.clone());

            self.upsert_component(component);
            self.runtimes.insert(module_id, runtime);
        }

        Ok(())
    }

    // ── Boot sequence ─────────────────────────────────────────────────────────

    pub fn boot(&mut self) -> Result<(), HostError> {
        self.register_core_components();

        // Elevate to privileged mode so all registration syscalls are permitted.
        self.bridge.enter_privileged().map_err(HostError::Kernel)?;

        // Register every known component with the kernel.
        // Collect IDs first to avoid borrowing `self.components` and `self.bridge`
        // at the same time.
        let component_ids: Vec<String> = self.components.keys().cloned().collect();
        for id in &component_ids {
            let component = &self.components[id];
            let class  = component.group.to_kernel_class();
            let limits = component.limits.to_ffi();
            self.bridge
                .register_component(
                    &component.id,
                    class,
                    &component.endpoint,
                    &component.network_manager,
                    &limits,
                )
                .map_err(HostError::Kernel)?;
        }

        // Register all manifest-loaded modules and publish activation events.
        let runtime_ids: Vec<String> = self.runtimes.keys().cloned().collect();
        for id in &runtime_ids {
            let runtime = self.runtimes.get_mut(id).unwrap();

            self.bridge
                .register_module(&runtime.id, &runtime.kind)
                .map_err(HostError::Kernel)?;

            runtime.active = runtime.status != "disabled";

            let payload = format!(
                "{{\"module\":\"{}\",\"kind\":\"{}\",\"language\":\"{}\",\"network_manager\":\"{}\",\
                 \"capabilities\":{},\"integrations\":{},\
                 \"limits\":{{\"memory_mb\":{},\"processes\":{},\"files\":{},\"resources\":{}}}}}",
                runtime.id,
                runtime.kind,
                runtime.language,
                runtime.network_manager,
                json_str_array(&runtime.capabilities),
                json_str_array(&runtime.integrations),
                runtime.limits.memory_limit_mb,
                runtime.limits.max_processes,
                runtime.limits.max_files,
                runtime.limits.max_resources,
            );
            self.bridge
                .publish_event("host.module.activated", &payload)
                .map_err(HostError::Kernel)?;

            if runtime.kind == "office" {
                self.bridge
                    .publish_event(
                        "office.views.registered",
                        "{\"module\":\"kogi.office\",\"views\":[\"dashboard\",\"portfolio\",\
                         \"timeline\",\"workspace\",\"assistant\"]}",
                    )
                    .map_err(HostError::Kernel)?;
            }
        }

        self.bridge
            .publish_event(
                "host.orchestrator.booted",
                &format!(
                    "{{\"module_count\":{},\"component_count\":{},\
                     \"engine\":\"kogi-engine\",\
                     \"engine_service\":\"kogi-services/go/services/engine\",\
                     \"database_service\":\"kogi-services/go/services/database\",\
                     \"server\":\"kogi-server\",\"gateway\":\"kogi-go-gateway\"}}",
                    self.runtimes.len(),
                    self.components.len()
                ),
            )
            .map_err(HostError::Kernel)?;

        // Drop back to user mode now that boot registration is complete.
        self.bridge.exit_privileged().map_err(HostError::Kernel)?;
        self.booted = true;
        Ok(())
    }

    // ── Runtime tick ──────────────────────────────────────────────────────────

    pub fn tick(&self) {
        println!("[host] {}", self.summary_line());
        for line in self.module_snapshot_lines() {
            println!("{line}");
        }
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    pub fn summary_line(&self) -> String {
        format!(
            "booted={} modules={} components={} kernel_orchestration=enabled",
            self.booted,
            self.runtimes.len(),
            self.components.len()
        )
    }

    pub fn platform_snapshot_payload(&self) -> String {
        format!(
            "{{\"host_id\":\"kogi-host-001\",\"module_count\":{},\"component_count\":{},\"timestamp_ms\":{}}}",
            self.runtimes.len(),
            self.components.len(),
            now_ms()
        )
    }

    pub fn is_booted(&self) -> bool {
        self.booted
    }

    pub fn module_count(&self) -> usize {
        self.runtimes.len()
    }

    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    // ── Kernel bridge passthrough ─────────────────────────────────────────────

    pub fn publish_event(&mut self, topic: &str, payload: &str) -> Result<(), HostError> {
        self.bridge
            .publish_event(topic, payload)
            .map_err(HostError::Kernel)
    }

    /// Return a live kernel stats snapshot from the Zig kernel.
    pub fn kernel_stats(&self) -> Result<FfiKernelStats, HostError> {
        self.bridge.get_stats().map_err(HostError::Kernel)
    }

    /// Elevate the kernel session to privileged mode.
    pub fn enter_privileged(&mut self) -> Result<(), HostError> {
        self.bridge.enter_privileged().map_err(HostError::Kernel)
    }

    /// Drop the kernel session to user mode.
    pub fn exit_privileged(&mut self) -> Result<(), HostError> {
        self.bridge.exit_privileged().map_err(HostError::Kernel)
    }

    /// Bootstrap all five core platform components via the kernel.
    pub fn bootstrap_core(&mut self) -> Result<(), HostError> {
        self.bridge.bootstrap_core().map_err(HostError::Kernel)
    }

    /// Provision the kogi.office module via the kernel.
    pub fn bootstrap_office(&mut self) -> Result<(), HostError> {
        self.bridge.bootstrap_office().map_err(HostError::Kernel)
    }

    /// Start a registered module via the kernel.
    pub fn start_module(&mut self, id: &str) -> Result<(), HostError> {
        self.bridge.start_module(id).map_err(HostError::Kernel)
    }

    /// Stop a running module via the kernel.
    pub fn stop_module(&mut self, id: &str) -> Result<(), HostError> {
        self.bridge.stop_module(id).map_err(HostError::Kernel)
    }

    /// Stop and deregister a module, releasing all its kernel resources.
    pub fn remove_module(&mut self, id: &str) -> Result<(), HostError> {
        self.bridge.remove_module(id).map_err(HostError::Kernel)?;
        self.runtimes.remove(id);
        self.components.remove(id);
        Ok(())
    }

    // ── Snapshot helpers ──────────────────────────────────────────────────────

    pub fn modules(&self) -> Vec<ModuleRuntime> {
        self.runtimes.values().cloned().collect()
    }

    pub fn components(&self) -> Vec<ComponentRuntime> {
        self.components.values().cloned().collect()
    }

    pub fn module_isolation_snapshot(&self) -> Vec<ModuleIsolationSnapshot> {
        self.runtimes
            .values()
            .map(|runtime| {
                let memory_used   = estimate_u64(runtime.limits.memory_limit_mb, 5);
                let process_count = estimate_u32(runtime.limits.max_processes, 8);
                let file_count    = estimate_u32(runtime.limits.max_files, 10);
                let resource_used = estimate_u32(runtime.limits.max_resources, 9);
                ModuleIsolationSnapshot {
                    module_id:       runtime.id.clone(),
                    memory_limit_mb: runtime.limits.memory_limit_mb,
                    memory_used_mb:  memory_used,
                    process_limit:   runtime.limits.max_processes,
                    process_count,
                    file_limit:      runtime.limits.max_files,
                    file_count,
                    resource_limit:  runtime.limits.max_resources,
                    resource_used,
                    network_manager: runtime.network_manager.clone(),
                }
            })
            .collect()
    }

    pub fn module_snapshot_lines(&self) -> Vec<String> {
        self.runtimes
            .values()
            .map(|runtime| {
                format!(
                    "[module] id={} name={} kind={} version={} state={} endpoint={} lang={} \
                     caps={} integrations={} limits(memory={}MB,proc={},files={},res={})",
                    runtime.id,
                    runtime.name,
                    runtime.kind,
                    runtime.version,
                    if runtime.active { "active" } else { "disabled" },
                    runtime.entrypoint,
                    runtime.language,
                    runtime.capabilities.len(),
                    runtime.integrations.len(),
                    runtime.limits.memory_limit_mb,
                    runtime.limits.max_processes,
                    runtime.limits.max_files,
                    runtime.limits.max_resources,
                )
            })
            .collect()
    }

    pub fn component_snapshot_lines(&self) -> Vec<String> {
        self.components
            .values()
            .map(|component| {
                let (health, checked_at, details) = if let Some(last) = &component.last_health {
                    (
                        if last.healthy { "healthy" } else { "down" },
                        last.checked_at_ms.to_string(),
                        last.details.clone(),
                    )
                } else {
                    ("unknown", "never".to_string(), "not checked".to_string())
                };

                format!(
                    "[component] id={} group={} managed_by_kernel={} active={} \
                     endpoint={} network={} limits(memory={}MB,proc={},files={},res={}) \
                     health={} checked_at={} details={}",
                    component.id,
                    component.group.as_str(),
                    component.managed_by_kernel,
                    component.active,
                    component.endpoint,
                    component.network_manager,
                    component.limits.memory_limit_mb,
                    component.limits.max_processes,
                    component.limits.max_files,
                    component.limits.max_resources,
                    health,
                    checked_at,
                    details,
                )
            })
            .collect()
    }

    pub fn component_detail_lines(&self, id: &str) -> Option<Vec<String>> {
        let component = self.components.get(id)?;
        let mut lines = vec![
            format!("id={}", component.id),
            format!("group={}", component.group.as_str()),
            format!("managed_by_kernel={}", component.managed_by_kernel),
            format!("active={}", component.active),
            format!("endpoint={}", component.endpoint),
            format!("network_manager={}", component.network_manager),
            format!(
                "limits=memory:{}MB processes:{} files:{} resources:{}",
                component.limits.memory_limit_mb,
                component.limits.max_processes,
                component.limits.max_files,
                component.limits.max_resources
            ),
        ];

        if let Some(health) = &component.last_health {
            lines.push(format!(
                "health={} checked_at={} details={}",
                if health.healthy { "healthy" } else { "down" },
                health.checked_at_ms,
                health.details
            ));
        }

        for (key, value) in &component.metadata {
            lines.push(format!("meta.{key}={value}"));
        }
        Some(lines)
    }

    pub fn refresh_component_health(
        &mut self,
        filter: Option<&str>,
    ) -> Result<Vec<String>, HostError> {
        let mut output = Vec::new();
        let now = now_ms();

        for component in self.components.values_mut() {
            if let Some(filter) = filter {
                if component.id != filter && !component.id.contains(filter) {
                    continue;
                }
            }

            let probe = probe_component_health(&component.endpoint, now);
            component.last_health = Some(probe.clone());
            output.push(format!(
                "component={} health={} details={}",
                component.id,
                if probe.healthy { "healthy" } else { "down" },
                probe.details
            ));
        }

        self.bridge
            .publish_event(
                "host.orchestrator.health.checked",
                &format!(
                    "{{\"checked\":{},\"filter\":\"{}\"}}",
                    output.len(),
                    filter.unwrap_or("*")
                ),
            )
            .map_err(HostError::Kernel)?;

        Ok(output)
    }

    // ── Service / engine / database HTTP passthrough ──────────────────────────

    pub fn fetch_service_runtime(&self, service_id: &str) -> Result<String, HostError> {
        let endpoint = resolve_service_runtime_endpoint(service_id)
            .ok_or_else(|| HostError::Service(format!("unknown service id: {service_id}")))?;
        let (status, body) =
            http_request("GET", &endpoint, None).map_err(HostError::Service)?;
        if (200..300).contains(&status) {
            Ok(body)
        } else {
            Err(HostError::Service(format!(
                "runtime request failed status={status} endpoint={endpoint}"
            )))
        }
    }

    pub fn engine_control(&self, action: &str) -> Result<String, HostError> {
        let endpoint = resolve_engine_control_endpoint();
        let payload = format!("{{\"action\":\"{}\"}}", escape_json(action));
        let (status, body) =
            http_request("POST", &endpoint, Some(&payload)).map_err(HostError::Service)?;
        if (200..300).contains(&status) {
            Ok(body)
        } else {
            Err(HostError::Service(format!(
                "engine control failed status={status} endpoint={endpoint}"
            )))
        }
    }

    pub fn engine_ingest(&self, payload: &str) -> Result<String, HostError> {
        let endpoint = resolve_engine_ingest_endpoint();
        let body_payload = if payload.trim_start().starts_with('{') {
            payload.to_string()
        } else {
            format!("{{\"payload\":\"{}\"}}", escape_json(payload))
        };
        let (status, body) =
            http_request("POST", &endpoint, Some(&body_payload)).map_err(HostError::Service)?;
        if (200..300).contains(&status) {
            Ok(body)
        } else {
            Err(HostError::Service(format!(
                "engine ingest failed status={status} endpoint={endpoint}"
            )))
        }
    }

    pub fn database_query(&self, sql: &str) -> Result<String, HostError> {
        let endpoint = resolve_database_query_endpoint();
        let payload = format!("{{\"sql\":\"{}\"}}", escape_json(sql));
        let (status, body) =
            http_request("POST", &endpoint, Some(&payload)).map_err(HostError::Service)?;
        if (200..300).contains(&status) {
            Ok(body)
        } else {
            Err(HostError::Service(format!(
                "database query failed status={status} endpoint={endpoint}"
            )))
        }
    }

    // ── Core component registration ───────────────────────────────────────────

    fn register_core_components(&mut self) {
        let cores: &[(&str, ComponentGroup, &str, &str)] = &[
            ("kogi.kernel",             ComponentGroup::Kernel,  "local://kogi-kernel",         "kernel-native"),
            ("kogi.host",               ComponentGroup::Host,    "local://kogi-host",            "kernel-native"),
            ("kogi.server",             ComponentGroup::Server,  "http://127.0.0.1:8080/health", "kogi-go-network"),
            ("kogi.engine",             ComponentGroup::Engine,  "local://kogi-engine",          "kogi-go-network"),
            ("kogi.services.gateway",   ComponentGroup::Gateway, "http://127.0.0.1:8090/health", "kogi-go-network"),
            ("kogi.services.auth",      ComponentGroup::Service, "http://127.0.0.1:9001/health", "kogi-go-network"),
            ("kogi.services.portfolio", ComponentGroup::Service, "http://127.0.0.1:9002/health", "kogi-go-network"),
            ("kogi.services.exchange",  ComponentGroup::Service, "http://127.0.0.1:9004/health", "kogi-go-network"),
            ("kogi.services.ims",       ComponentGroup::Service, "http://127.0.0.1:9005/health", "kogi-go-network"),
            ("kogi.services.office",    ComponentGroup::Service, "http://127.0.0.1:9006/health", "kogi-go-network"),
            ("kogi.services.bank",      ComponentGroup::Service, "http://127.0.0.1:9007/health", "kogi-go-network"),
            ("kogi.services.marketplace", ComponentGroup::Service, "http://127.0.0.1:9008/health", "kogi-go-network"),
            ("kogi.services.studio",    ComponentGroup::Service, "http://127.0.0.1:9009/health", "kogi-go-network"),
            ("kogi.services.community", ComponentGroup::Service, "http://127.0.0.1:9010/health", "kogi-go-network"),
            ("kogi.services.developer", ComponentGroup::Service, "http://127.0.0.1:9011/health", "kogi-go-network"),
            ("kogi.services.profile",   ComponentGroup::Service, "http://127.0.0.1:9012/health", "kogi-go-network"),
            ("kogi.services.organizations", ComponentGroup::Service, "http://127.0.0.1:9013/health", "kogi-go-network"),
            ("kogi.services.engine",    ComponentGroup::Service, "http://127.0.0.1:9014/health", "kogi-go-network"),
            ("kogi.services.database",  ComponentGroup::Service, "http://127.0.0.1:9015/health", "kogi-go-network"),
        ];

        for (id, group, endpoint, net_mgr) in cores {
            let limits = default_limits_for_group(group);
            self.upsert_component(ComponentRuntime::new(
                *id,
                group.clone(),
                *endpoint,
                *net_mgr,
                limits,
            ));
        }
    }

    fn upsert_component(&mut self, component: ComponentRuntime) {
        self.components.insert(component.id.clone(), component);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Default resource limits per component group
// ─────────────────────────────────────────────────────────────────────────────

fn default_limits_for_group(group: &ComponentGroup) -> ResourceLimits {
    match group {
        ComponentGroup::Kernel => ResourceLimits {
            memory_limit_mb: 1024,
            max_processes:   512,
            max_files:       20000,
            max_resources:   40000,
        },
        ComponentGroup::Host => ResourceLimits {
            memory_limit_mb: 768,
            max_processes:   384,
            max_files:       16000,
            max_resources:   30000,
        },
        ComponentGroup::Server => ResourceLimits {
            memory_limit_mb: 768,
            max_processes:   256,
            max_files:       12000,
            max_resources:   24000,
        },
        ComponentGroup::Engine => ResourceLimits {
            memory_limit_mb: 1024,
            max_processes:   256,
            max_files:       12000,
            max_resources:   32000,
        },
        ComponentGroup::Gateway => ResourceLimits {
            memory_limit_mb: 512,
            max_processes:   192,
            max_files:       10000,
            max_resources:   20000,
        },
        ComponentGroup::Service => ResourceLimits {
            memory_limit_mb: 512,
            max_processes:   160,
            max_files:       8000,
            max_resources:   16000,
        },
        ComponentGroup::Module => ResourceLimits {
            memory_limit_mb: 512,
            max_processes:   128,
            max_files:       6000,
            max_resources:   12000,
        },
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Endpoint resolution
// ─────────────────────────────────────────────────────────────────────────────

fn resolve_service_endpoint(entrypoint: &str, kind: &str) -> String {
    let lower = entrypoint.to_lowercase();
    let port_map: &[(&str, &str)] = &[
        ("services/auth",          "http://127.0.0.1:9001/health"),
        ("services/portfolio",     "http://127.0.0.1:9002/health"),
        ("services/exchange",      "http://127.0.0.1:9004/health"),
        ("services/ims",           "http://127.0.0.1:9005/health"),
        ("services/office",        "http://127.0.0.1:9006/health"),
        ("services/bank",          "http://127.0.0.1:9007/health"),
        ("services/marketplace",   "http://127.0.0.1:9008/health"),
        ("services/studio",        "http://127.0.0.1:9009/health"),
        ("services/community",     "http://127.0.0.1:9010/health"),
        ("services/developer",     "http://127.0.0.1:9011/health"),
        ("services/profile",       "http://127.0.0.1:9012/health"),
        ("services/organizations", "http://127.0.0.1:9013/health"),
        ("services/engine",        "http://127.0.0.1:9014/health"),
        ("services/database",      "http://127.0.0.1:9015/health"),
    ];
    for (pattern, endpoint) in port_map {
        if lower.contains(pattern) {
            return endpoint.to_string();
        }
    }
    if lower.contains("/gateway") || lower.ends_with("gateway") {
        return "http://127.0.0.1:8090/health".to_string();
    }
    if lower.contains("kogi-server") {
        return "http://127.0.0.1:8080/health".to_string();
    }
    if entrypoint.starts_with("http://") {
        return entrypoint.to_string();
    }
    format!("module://{}::{}", kind, entrypoint)
}

fn resolve_service_runtime_endpoint(service_id: &str) -> Option<String> {
    match service_id {
        "kogi.services.auth"          => Some("http://127.0.0.1:9001/api/v1/auth/runtime".to_string()),
        "kogi.services.portfolio"     => Some("http://127.0.0.1:9002/api/v1/portfolio/runtime".to_string()),
        "kogi.services.exchange"      => Some("http://127.0.0.1:9004/api/v1/exchange/runtime".to_string()),
        "kogi.services.ims"           => Some("http://127.0.0.1:9005/api/v1/ims/runtime".to_string()),
        "kogi.services.office"        => Some("http://127.0.0.1:9006/api/v1/office/runtime".to_string()),
        "kogi.services.bank"          => Some("http://127.0.0.1:9007/api/v1/bank/runtime".to_string()),
        "kogi.services.marketplace"   => Some("http://127.0.0.1:9008/api/v1/marketplace/runtime".to_string()),
        "kogi.services.studio"        => Some("http://127.0.0.1:9009/api/v1/studio/runtime".to_string()),
        "kogi.services.community"     => Some("http://127.0.0.1:9010/api/v1/community/runtime".to_string()),
        "kogi.services.developer"     => Some("http://127.0.0.1:9011/api/v1/developer/runtime".to_string()),
        "kogi.services.profile"       => Some("http://127.0.0.1:9012/api/v1/profile/runtime".to_string()),
        "kogi.services.organizations" => Some("http://127.0.0.1:9013/api/v1/organizations/runtime".to_string()),
        "kogi.services.engine"        => Some("http://127.0.0.1:9014/api/v1/engine/runtime".to_string()),
        "kogi.services.database"      => Some("http://127.0.0.1:9015/api/v1/database/runtime".to_string()),
        _ => None,
    }
}

fn resolve_engine_control_endpoint()  -> String { "http://127.0.0.1:9014/api/v1/engine/control".to_string() }
fn resolve_engine_ingest_endpoint()   -> String { "http://127.0.0.1:9014/api/v1/engine/ingest".to_string()  }
fn resolve_database_query_endpoint()  -> String { "http://127.0.0.1:9015/api/v1/database/query".to_string() }

// ─────────────────────────────────────────────────────────────────────────────
// Health probing
// ─────────────────────────────────────────────────────────────────────────────

fn probe_component_health(endpoint: &str, checked_at_ms: i64) -> ComponentHealth {
    if endpoint.starts_with("local://") {
        return ComponentHealth {
            healthy: true,
            checked_at_ms,
            details: "local runtime managed by host".to_string(),
        };
    }
    if endpoint.starts_with("module://") {
        return ComponentHealth {
            healthy: true,
            checked_at_ms,
            details: "module endpoint registered".to_string(),
        };
    }
    if endpoint.starts_with("http://") {
        return match probe_http_endpoint(endpoint) {
            Ok((healthy, details)) => ComponentHealth { healthy, checked_at_ms, details },
            Err(err) => ComponentHealth { healthy: false, checked_at_ms, details: err },
        };
    }
    ComponentHealth {
        healthy: false,
        checked_at_ms,
        details: "unsupported endpoint scheme".to_string(),
    }
}

fn probe_http_endpoint(endpoint: &str) -> Result<(bool, String), String> {
    let (status, _) = http_request("GET", endpoint, None)?;
    Ok(((200..300).contains(&status), format!("HTTP {status}")))
}

// ─────────────────────────────────────────────────────────────────────────────
// HTTP utility
// ─────────────────────────────────────────────────────────────────────────────

fn http_request(method: &str, endpoint: &str, body: Option<&str>) -> Result<(u16, String), String> {
    let stripped = endpoint
        .strip_prefix("http://")
        .ok_or_else(|| "only http endpoints are supported".to_string())?;

    let (host_port, path) = if let Some((hp, p)) = stripped.split_once('/') {
        (hp, format!("/{p}"))
    } else {
        (stripped, "/".to_string())
    };

    if host_port.is_empty() {
        return Err("invalid endpoint host".to_string());
    }

    let request_body = body.unwrap_or("");
    let mut stream = TcpStream::connect(host_port)
        .map_err(|err| format!("connect failed: {err}"))?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));

    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\n\
         Content-Length: {len}\r\nConnection: close\r\n\r\n{body}",
        method = method,
        path   = path,
        host   = host_port,
        len    = request_body.as_bytes().len(),
        body   = request_body
    );
    stream.write_all(request.as_bytes())
        .map_err(|err| format!("request write failed: {err}"))?;

    let mut response = String::new();
    stream.read_to_string(&mut response)
        .map_err(|err| format!("response read failed: {err}"))?;

    let status_line = response.lines().next().unwrap_or("HTTP/1.1 000 UNKNOWN");
    let status = status_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("000")
        .parse::<u16>()
        .unwrap_or(0);
    let body = response.split("\r\n\r\n").nth(1).unwrap_or("").to_string();
    Ok((status, body))
}

// ─────────────────────────────────────────────────────────────────────────────
// Misc utilities
// ─────────────────────────────────────────────────────────────────────────────

fn json_str_array(values: &[String]) -> String {
    let inner = values.iter().map(|v| format!("\"{v}\"")).collect::<Vec<_>>().join(",");
    format!("[{inner}]")
}

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"',  "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_millis(0))
        .as_millis() as i64
}

fn estimate_u64(limit: u64, divisor: u64) -> u64 {
    if limit == 0 { return 0; }
    let v = limit / divisor;
    if v == 0 { 1 } else { v }
}

fn estimate_u32(limit: u32, divisor: u32) -> u32 {
    if limit == 0 { return 0; }
    let v = limit / divisor;
    if v == 0 { 1 } else { v }
}