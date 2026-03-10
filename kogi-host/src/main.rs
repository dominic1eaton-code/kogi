use kogi_host::{boot_default_host, shell::run_shell};

fn main() {
    let mut host = match boot_default_host() {
        Ok(host) => host,
        Err(err) => {
            eprintln!("host boot failed: {err}");
            std::process::exit(1);
        }
    };
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let shell_mode = !args.iter().any(|arg| arg == "--once");

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
