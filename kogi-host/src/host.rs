use crate::executive::{ComponentRuntime, HostError, ModuleIsolationSnapshot};
use crate::kernel::{FfiKernelStats, KernelBridge};
use crate::runtime::{HostRuntime, ModuleRuntime};
use crate::shell;

pub struct HostSystem {
    runtime: HostRuntime,
}

impl HostSystem {
    /// Create an uninitialised system.  The kernel bridge is initialised but
    /// modules are not yet loaded.  Call [`bootstrap`] for a fully booted system.
    pub fn new() -> Result<Self, HostError> {
        Ok(Self {
            runtime: HostRuntime::new()?,
        })
    }

    /// Load modules from default roots, boot the executive, and start the engine.
    pub fn bootstrap(&mut self) -> Result<(), HostError> {
        self.runtime = HostRuntime::bootstrap()?;
        Ok(())
    }

    // ── Runtime accessors ─────────────────────────────────────────────────────

    pub fn runtime(&self) -> &HostRuntime {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut HostRuntime {
        &mut self.runtime
    }

    // ── Interactive shell ─────────────────────────────────────────────────────

    pub fn run_shell(&mut self) -> Result<(), HostError> {
        shell::run_shell(self.runtime.executive_mut())
    }

    // ── Convenience passthrough ───────────────────────────────────────────────

    pub fn tick(&self) {
        self.runtime.tick();
    }

    pub fn summary_line(&self) -> String {
        self.runtime.summary_line()
    }

    pub fn booted(&self) -> bool {
        self.runtime.booted()
    }

    pub fn module_count(&self) -> usize {
        self.runtime.module_count()
    }

    pub fn component_count(&self) -> usize {
        self.runtime.component_count()
    }

    pub fn modules(&self) -> Vec<ModuleRuntime> {
        self.runtime.modules()
    }

    pub fn components(&self) -> Vec<ComponentRuntime> {
        self.runtime.components()
    }

    pub fn module_isolation_snapshot(&self) -> Vec<ModuleIsolationSnapshot> {
        self.runtime.module_isolation_snapshot()
    }

    /// Return a live kernel stats snapshot from the Zig kernel.
    pub fn kernel_stats(&self) -> Result<FfiKernelStats, HostError> {
        self.runtime.executive().kernel_stats()
    }

    pub fn fetch_service_runtime(&self, service_id: &str) -> Result<String, HostError> {
        self.runtime.fetch_service_runtime(service_id)
    }

    pub fn engine_control(&self, action: &str) -> Result<String, HostError> {
        self.runtime.engine_control(action)
    }

    pub fn engine_ingest(&self, payload: &str) -> Result<String, HostError> {
        self.runtime.engine_ingest(payload)
    }

    pub fn database_query(&self, sql: &str) -> Result<String, HostError> {
        self.runtime.database_query(sql)
    }

    pub fn refresh_component_health(
        &mut self,
        filter: Option<&str>,
    ) -> Result<Vec<String>, HostError> {
        self.runtime.refresh_component_health(filter)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Module-level utilities
// ─────────────────────────────────────────────────────────────────────────────

/// Format a human-readable description of a host's bridge type and loaded
/// module IDs.  Useful for diagnostic logging.
pub fn describe_host_components<B: KernelBridge>(
    _host: &B,
    module_ids: &[String],
) -> String {
    format!(
        "host_components={{kernel_bridge:{}, module_count:{}}}",
        std::any::type_name::<B>(),
        module_ids.len()
    )
}