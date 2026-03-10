use crate::executive::{ComponentRuntime, HostError, HostExecutive, ModuleIsolationSnapshot, ModuleRuntime};
use crate::kernel_bridge::LocalKernelBridge;
use crate::load_modules_from_default_roots;

pub struct HostRuntime {
    executive: HostExecutive<LocalKernelBridge>,
}

impl HostRuntime {
    pub fn new() -> Self {
        Self {
            executive: HostExecutive::new(LocalKernelBridge::new()),
        }
    }

    pub fn bootstrap() -> Result<Self, HostError> {
        let mut runtime = Self::new();
        load_modules_from_default_roots(&mut runtime.executive)?;
        runtime.executive.boot()?;
        runtime.executive.tick();

        if let Err(err) = runtime.executive.engine_control("start") {
            eprintln!("[host-runtime] engine service control failed: {err}");
        }
        if let Err(err) = runtime
            .executive
            .engine_ingest(&runtime.executive.platform_snapshot_payload())
        {
            eprintln!("[host-runtime] engine ingest failed: {err}");
        }

        Ok(runtime)
    }

    pub fn executive(&self) -> &HostExecutive<LocalKernelBridge> {
        &self.executive
    }

    pub fn executive_mut(&mut self) -> &mut HostExecutive<LocalKernelBridge> {
        &mut self.executive
    }

    pub fn booted(&self) -> bool {
        self.executive.is_booted()
    }

    pub fn module_count(&self) -> usize {
        self.executive.module_count()
    }

    pub fn component_count(&self) -> usize {
        self.executive.component_count()
    }

    pub fn summary_line(&self) -> String {
        self.executive.summary_line()
    }

    pub fn tick(&self) {
        self.executive.tick();
    }

    pub fn modules(&self) -> Vec<ModuleRuntime> {
        self.executive.modules()
    }

    pub fn components(&self) -> Vec<ComponentRuntime> {
        self.executive.components()
    }

    pub fn module_isolation_snapshot(&self) -> Vec<ModuleIsolationSnapshot> {
        self.executive.module_isolation_snapshot()
    }

    pub fn fetch_service_runtime(&self, service_id: &str) -> Result<String, HostError> {
        self.executive.fetch_service_runtime(service_id)
    }

    pub fn engine_control(&self, action: &str) -> Result<String, HostError> {
        self.executive.engine_control(action)
    }

    pub fn engine_ingest(&self, payload: &str) -> Result<String, HostError> {
        self.executive.engine_ingest(payload)
    }

    pub fn database_query(&self, sql: &str) -> Result<String, HostError> {
        self.executive.database_query(sql)
    }

    pub fn refresh_component_health(
        &mut self,
        filter: Option<&str>,
    ) -> Result<Vec<String>, HostError> {
        self.executive.refresh_component_health(filter)
    }
}
