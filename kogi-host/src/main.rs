use kogi_host::executive::HostExecutive;
use kogi_host::kernel_bridge::LocalKernelBridge;
use kogi_host::load_modules_from_default_roots;
use kogi_host::shell::run_shell;

fn main() {
    let bridge = LocalKernelBridge::new();
    let mut host = HostExecutive::new(bridge);
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let shell_mode = !args.iter().any(|arg| arg == "--once");

    if let Err(err) = load_modules_from_default_roots(&mut host) {
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

    if let Err(err) = host.engine_control("start") {
        eprintln!("engine service control failed: {err}");
    }
    if let Err(err) = host.engine_ingest(&host.platform_snapshot_payload()) {
        eprintln!("engine service ingest failed: {err}");
    }

    if shell_mode {
        if let Err(err) = run_shell(&mut host) {
            eprintln!("shell terminated with error: {err}");
            std::process::exit(1);
        }
    }
}
