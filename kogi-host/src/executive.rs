use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::Path;

use crate::kernel_bridge::KernelBridge;
use crate::module_runtime::ModuleRuntime;

#[derive(Debug)]
pub enum HostError {
    Io(std::io::Error),
    InvalidManifest(String),
    Kernel(String),
}

impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::InvalidManifest(msg) => write!(f, "invalid manifest: {msg}"),
            Self::Kernel(msg) => write!(f, "kernel bridge error: {msg}"),
        }
    }
}

impl From<std::io::Error> for HostError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

pub struct HostExecutive<B: KernelBridge> {
    bridge: B,
    runtimes: BTreeMap<String, ModuleRuntime>,
}

impl<B: KernelBridge> HostExecutive<B> {
    pub fn new(bridge: B) -> Self {
        Self {
            bridge,
            runtimes: BTreeMap::new(),
        }
    }

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
            let runtime = ModuleRuntime::from_manifest(&content)
                .map_err(HostError::InvalidManifest)?;

            self.runtimes.insert(runtime.id.clone(), runtime);
        }

        Ok(())
    }

    pub fn boot(&mut self) -> Result<(), HostError> {
        self.bridge
            .set_mode_kernel()
            .map_err(HostError::Kernel)?;

        for runtime in self.runtimes.values_mut() {
            self.bridge
                .register_module(&runtime.id, &runtime.kind)
                .map_err(HostError::Kernel)?;
            runtime.active = true;

            let payload = format!(
                "{{\"module\":\"{}\",\"kind\":\"{}\",\"language\":\"{}\",\"network_manager\":\"{}\",\"capabilities\":{},\"integrations\":{},\"limits\":{{\"memory_mb\":{},\"processes\":{},\"files\":{},\"resources\":{}}}}}",
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
                        "{\"module\":\"kogi.office\",\"views\":[\"dashboard\",\"portfolio\",\"timeline\",\"workspace\",\"assistant\"]}",
                    )
                    .map_err(HostError::Kernel)?;
            }
        }

        self.bridge
            .set_mode_user()
            .map_err(HostError::Kernel)?;

        Ok(())
    }

    pub fn tick(&self) {
        for runtime in self.runtimes.values() {
            if runtime.active {
                println!(
                    "[host] module={} name={} version={} kind={} lang={} endpoint={} capabilities={} integrations={} limits(memory={}MB,proc={},files={},res={})",
                    runtime.id,
                    runtime.name,
                    runtime.version,
                    runtime.kind,
                    runtime.language,
                    runtime.entrypoint,
                    runtime.capabilities.len(),
                    runtime.integrations.len(),
                    runtime.limits.memory_limit_mb,
                    runtime.limits.max_processes,
                    runtime.limits.max_files,
                    runtime.limits.max_resources,
                );
            }
        }
    }

    pub fn module_count(&self) -> usize {
        self.runtimes.len()
    }
}

fn json_str_array(values: &[String]) -> String {
    let values = values
        .iter()
        .map(|v| format!("\"{v}\""))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{values}]")
}
