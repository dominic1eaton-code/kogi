pub trait KernelBridge {
    fn register_module(&mut self, id: &str, kind: &str) -> Result<(), String>;
    fn publish_event(&mut self, topic: &str, payload: &str) -> Result<(), String>;
    fn set_mode_kernel(&mut self) -> Result<(), String>;
    fn set_mode_user(&mut self) -> Result<(), String>;
}

#[derive(Default)]
pub struct LocalKernelBridge {
    mode: &'static str,
}

impl LocalKernelBridge {
    pub fn new() -> Self {
        Self { mode: "kernel" }
    }
}

impl KernelBridge for LocalKernelBridge {
    fn register_module(&mut self, id: &str, kind: &str) -> Result<(), String> {
        println!("[kernel-bridge] register module id={id} kind={kind}");
        Ok(())
    }

    fn publish_event(&mut self, topic: &str, payload: &str) -> Result<(), String> {
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