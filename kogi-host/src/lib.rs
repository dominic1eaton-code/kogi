pub mod executive;
pub mod kernel_bridge;
pub mod module_runtime;
pub mod shell;
pub mod runtime;

pub const DEFAULT_MODULE_ROOTS: [&str; 2] = ["kogi-modules", "../kogi-modules"];

pub fn load_modules_from_default_roots<B: kernel_bridge::KernelBridge>(
    host: &mut executive::HostExecutive<B>,
) -> Result<(), executive::HostError> {
    let mut last_error = None;
    for root in DEFAULT_MODULE_ROOTS {
        match host.load_modules(root) {
            Ok(()) => return Ok(()),
            Err(err) => last_error = Some(err),
        }
    }

    Err(last_error.unwrap_or_else(|| {
        executive::HostError::InvalidManifest("no module paths checked".to_string())
    }))
}
