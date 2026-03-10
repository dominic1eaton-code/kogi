use kogi_host::app::HostApp;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let shell_mode = !args.iter().any(|arg| arg == "--once");

    let mut app = match HostApp::new() {
        Ok(app) => app,
        Err(err) => {
            eprintln!("failed to initialize host app: {err}");
            std::process::exit(1);
        }
    };

    if let Err(err) = app.init() {
        eprintln!("host init failed: {err}");
        std::process::exit(1);
    }

    if let Err(err) = app.configure() {
        eprintln!("host configure failed: {err}");
        std::process::exit(1);
    }

    if let Err(err) = app.run() {
        eprintln!("host run failed: {err}");
        std::process::exit(1);
    }

    println!(
        "kogi-host app online: mode={} modules={} components={}",
        app.mode_label(),
        app.module_count(),
        app.component_count()
    );

    if shell_mode {
        if let Err(err) = app.run_shell() {
            eprintln!("shell terminated with error: {err}");
            std::process::exit(1);
        }
    }

    if let Err(err) = app.shutdown() {
        eprintln!("host shutdown failed: {err}");
        std::process::exit(1);
    }
}
