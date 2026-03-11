pub mod app;
pub mod executive;
pub mod host;
pub mod kernel;
pub mod model;
pub mod runtime;
pub mod shell;

pub use app::{HostApp, HostMode};
pub use host::HostSystem;
pub use model::{HostMessage, HostMessageResult, HostModel};

// Default search paths for the module manifest root directory.
// The first path that exists and contains at least one valid module is used.
pub const DEFAULT_MODULE_ROOTS: [&str; 2] = ["kogi-modules", "../kogi-modules"];

/// Walk `DEFAULT_MODULE_ROOTS` in order and call `host.load_modules(root)` on
/// the first root that succeeds.  Returns the last error if none succeed.
pub fn load_modules_from_default_roots<B: kernel::KernelBridge>(
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