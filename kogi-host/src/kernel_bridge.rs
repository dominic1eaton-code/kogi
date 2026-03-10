use crate::module_runtime::ResourceLimits;

pub trait KernelBridge {
    fn register_module(&mut self, id: &str, kind: &str) -> Result<(), String>;
    fn register_component(
        &mut self,
        id: &str,
        group: &str,
        endpoint: &str,
        network_manager: &str,
        limits: &ResourceLimits,
    ) -> Result<(), String>;
    fn publish_event(&mut self, topic: &str, payload: &str) -> Result<(), String>;
    fn set_mode_kernel(&mut self) -> Result<(), String>;
    fn set_mode_user(&mut self) -> Result<(), String>;
}

pub struct LocalKernelBridge {
    mode: &'static str,
    registered_components: usize,
    registered_modules: usize,
    published_events: usize,
}

impl LocalKernelBridge {
    pub fn new() -> Self {
        Self {
            mode: "kernel",
            registered_components: 0,
            registered_modules: 0,
            published_events: 0,
        }
    }
}

impl KernelBridge for LocalKernelBridge {
    fn register_module(&mut self, id: &str, kind: &str) -> Result<(), String> {
        self.registered_modules += 1;
        println!("[kernel-bridge] register module id={id} kind={kind}");
        Ok(())
    }

    fn register_component(
        &mut self,
        id: &str,
        group: &str,
        endpoint: &str,
        network_manager: &str,
        limits: &ResourceLimits,
    ) -> Result<(), String> {
        self.registered_components += 1;
        println!(
            "[kernel-bridge] register component id={id} group={group} endpoint={endpoint} network={network_manager} limits(memory={}MB proc={} files={} res={})",
            limits.memory_limit_mb,
            limits.max_processes,
            limits.max_files,
            limits.max_resources,
        );
        Ok(())
    }

    fn publish_event(&mut self, topic: &str, payload: &str) -> Result<(), String> {
        self.published_events += 1;
        println!("[kernel-bridge] event topic={topic} payload={payload}");
        Ok(())
    }

    fn set_mode_kernel(&mut self) -> Result<(), String> {
        self.mode = "kernel";
        println!("[kernel-bridge] mode=kernel");
        Ok(())
    }

    fn set_mode_user(&mut self) -> Result<(), String> {
        self.mode = "user";
        println!("[kernel-bridge] mode=user");
        Ok(())
    }
}
