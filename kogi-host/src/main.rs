mod executive;
mod kernel_bridge;
mod module_runtime;
mod shell;

use executive::HostExecutive;
use kernel_bridge::LocalKernelBridge;
use shell::run_shell;

fn main() {
    let bridge = LocalKernelBridge::new();
    let mut host = HostExecutive::new(bridge);
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let shell_mode = !args.iter().any(|arg| arg == "--once");

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
    println!(
        "kogi-host orchestrator online: modules={} components={}",
        host.module_count(),
        host.component_count()
    );

    if shell_mode {
        if let Err(err) = run_shell(&mut host) {
            eprintln!("shell terminated with error: {err}");
            std::process::exit(1);
        }
    }
}
