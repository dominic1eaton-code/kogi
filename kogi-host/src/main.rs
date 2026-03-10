mod executive;
mod kernel_bridge;
mod module_runtime;

use executive::HostExecutive;
use kernel_bridge::LocalKernelBridge;

fn main() {
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
        eprintln!("failed to load module manifests: {err}");
        std::process::exit(1);
    }

    if let Err(err) = host.boot() {
        eprintln!("host boot failed: {err}");
        std::process::exit(1);
    }

    host.tick();
    println!("kogi-host running with {} modules", host.module_count());
}
