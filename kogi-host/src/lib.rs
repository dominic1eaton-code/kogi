pub mod executive;
pub mod kernel_bridge;
pub mod module_runtime;
pub mod shell;

use executive::{HostError, HostExecutive};
use kernel_bridge::LocalKernelBridge;

pub fn boot_default_host() -> Result<HostExecutive<LocalKernelBridge>, HostError> {
    let bridge = LocalKernelBridge::new();
    let mut host = HostExecutive::new(bridge);

    let module_roots = ["kogi-modules", "../kogi-modules"];
    let mut load_result = Err(String::from("no module paths checked"));
    for root in module_roots {
        match host.load_modules(root) {
            Ok(()) => {
                load_result = Ok(());
                break;
            }
            Err(err) => {
                load_result = Err(format!("{err}"));
            }
        }
    }
    if let Err(err) = load_result {
        return Err(HostError::InvalidManifest(err));
    }

    host.boot()?;
    Ok(host)
}
