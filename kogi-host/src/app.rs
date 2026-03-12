use crate::executive::{ComponentRuntime, HostError, ModuleIsolationSnapshot, ModuleRuntime};
use crate::host::HostSystem;
use crate::model::{HostMessage, HostMessageResult, HostModel};
use crate::provider::{ProviderSnapshot, ProviderSystem};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostMode {
    Init,
    Configure,
    Run,
    Pause,
    Shutdown,
}

impl HostMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Init => "init",
            Self::Configure => "configure",
            Self::Run => "run",
            Self::Pause => "pause",
            Self::Shutdown => "shutdown",
        }
    }
}

pub struct HostApp {
    mode: HostMode,
    model: HostModel,
}

impl HostApp {
    pub fn new() -> Result<Self, HostError> {
        Ok(Self {
            mode: HostMode::Init,
            model: HostModel::new()?,
        })
    }

    pub fn mode(&self) -> HostMode {
        self.mode
    }

    pub fn mode_label(&self) -> &'static str {
        self.mode.as_str()
    }

    pub fn init(&mut self) -> Result<(), HostError> {
        self.mode = HostMode::Init;
        self.model.init()?;
        Ok(())
    }

    pub fn configure(&mut self) -> Result<(), HostError> {
        self.mode = HostMode::Configure;
        self.model.configure()?;
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), HostError> {
        self.mode = HostMode::Run;
        self.model.run()?;
        Ok(())
    }

    pub fn pause(&mut self) {
        self.mode = HostMode::Pause;
    }

    pub fn shutdown(&mut self) -> Result<(), HostError> {
        self.mode = HostMode::Shutdown;
        self.model.shutdown()?;
        Ok(())
    }

    pub fn model(&self) -> &HostModel {
        &self.model
    }

    pub fn model_mut(&mut self) -> &mut HostModel {
        &mut self.model
    }

    pub fn system(&self) -> &HostSystem {
        self.model.system()
    }

    pub fn system_mut(&mut self) -> &mut HostSystem {
        self.model.system_mut()
    }

    pub fn provider_system(&self) -> &ProviderSystem {
        self.model.system().provider_system()
    }

    pub fn provider_system_mut(&mut self) -> &mut ProviderSystem {
        self.model.system_mut().provider_system_mut()
    }

    pub fn provider_snapshot(&self) -> ProviderSnapshot {
        self.model.system().provider_snapshot()
    }

    pub fn handle_message(&mut self, message: HostMessage) -> HostMessageResult {
        self.model.process_message(message)
    }

    pub fn run_shell(&mut self) -> Result<(), HostError> {
        self.model.system_mut().run_shell()
    }

    pub fn booted(&self) -> bool {
        self.model.system().booted()
    }

    pub fn module_count(&self) -> usize {
        self.model.system().module_count()
    }

    pub fn component_count(&self) -> usize {
        self.model.system().component_count()
    }

    pub fn summary_line(&self) -> String {
        self.model.system().summary_line()
    }

    pub fn modules(&self) -> Vec<ModuleRuntime> {
        self.model.system().modules()
    }

    pub fn components(&self) -> Vec<ComponentRuntime> {
        self.model.system().components()
    }

    pub fn module_isolation_snapshot(&self) -> Vec<ModuleIsolationSnapshot> {
        self.model.system().module_isolation_snapshot()
    }

    pub fn fetch_service_runtime(&self, service_id: &str) -> Result<String, HostError> {
        self.model.system().fetch_service_runtime(service_id)
    }

    pub fn engine_control(&self, action: &str) -> Result<String, HostError> {
        self.model.system().engine_control(action)
    }

    pub fn engine_ingest(&self, payload: &str) -> Result<String, HostError> {
        self.model.system().engine_ingest(payload)
    }

    pub fn database_query(&self, sql: &str) -> Result<String, HostError> {
        self.model.system().database_query(sql)
    }

    pub fn refresh_component_health(
        &mut self,
        filter: Option<&str>,
    ) -> Result<Vec<String>, HostError> {
        self.model.system_mut().refresh_component_health(filter)
    }
}
