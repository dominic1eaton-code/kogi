use std::io::{self, Write};

use crate::executive::{HostError, HostExecutive};
use crate::kernel::KernelBridge;

pub fn run_shell<B: KernelBridge>(host: &mut HostExecutive<B>) -> Result<(), HostError> {
    println!("kogi-host interactive shell");
    println!("type `help` for commands");

    loop {
        print!("kogi-host> ");
        io::stdout().flush().map_err(HostError::Io)?;

        let mut line = String::new();
        let bytes = io::stdin().read_line(&mut line).map_err(HostError::Io)?;
        if bytes == 0 {
            break;
        }

        let input = line.trim();
        if input.is_empty() {
            continue;
        }

        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
            println!("exiting shell");
            break;
        }

        let mut parts = input.splitn(3, ' ');
        let command = parts.next().unwrap_or_default();

        match command {
            // ── Informational ─────────────────────────────────────────────────
            "help" => print_help(),

            "status" => println!("{}", host.summary_line()),

            "tick" => host.tick(),

            "modules" => {
                for line in host.module_snapshot_lines() {
                    println!("{line}");
                }
            }

            "components" => {
                for line in host.component_snapshot_lines() {
                    println!("{line}");
                }
            }

            "show" => {
                let id = parts.next().unwrap_or_default().trim();
                if id.is_empty() {
                    println!("usage: show <component_id>");
                    continue;
                }
                match host.component_detail_lines(id) {
                    Some(lines) => {
                        for line in lines {
                            println!("{line}");
                        }
                    }
                    None => println!("component not found: {id}"),
                }
            }

            // ── Kernel stats ──────────────────────────────────────────────────
            "stats" => match host.kernel_stats() {
                Ok(s) => {
                    let mode_str = if s.mode == 0 { "privileged" } else { "user" };
                    println!("kernel stats:");
                    println!("  mode             : {mode_str}");
                    println!("  memory           : {} / {} MiB", s.memory_used_mb, s.memory_total_mb);
                    println!("  processes (live) : {}", s.live_processes);
                    println!("  workers (idle/busy/queued): {} / {} / {}", s.idle_workers, s.busy_workers, s.queued_tasks);
                    println!("  modules (total/active)    : {} / {}", s.total_modules, s.active_modules);
                    println!("  services (total/run/fault): {} / {} / {}", s.total_services, s.running_services, s.faulted_services);
                    println!("  sockets / connections     : {} / {}", s.open_sockets, s.active_connections);
                    println!("  events (total/log/overflow): {} / {} / {}", s.kernel_events_total, s.kernel_events_log_len, s.kernel_event_overflow);
                }
                Err(err) => println!("stats failed: {err}"),
            },

            // ── Boot / bootstrap ──────────────────────────────────────────────
            "boot" => match host.boot() {
                Ok(()) => println!("host boot completed"),
                Err(err) => println!("boot failed: {err}"),
            },

            "bootstrap" => {
                let target = parts.next().unwrap_or("core").trim();
                match target {
                    "core" => match host.bootstrap_core() {
                        Ok(()) => println!("bootstrap core completed"),
                        Err(err) => println!("bootstrap core failed: {err}"),
                    },
                    "office" => match host.bootstrap_office() {
                        Ok(()) => println!("bootstrap office completed"),
                        Err(err) => println!("bootstrap office failed: {err}"),
                    },
                    _ => println!("usage: bootstrap [core|office]"),
                }
            }

            // ── Module lifecycle ──────────────────────────────────────────────
            "start" => {
                let id = parts.next().unwrap_or_default().trim();
                if id.is_empty() {
                    println!("usage: start <module_id>");
                    continue;
                }
                match host.start_module(id) {
                    Ok(()) => println!("started {id}"),
                    Err(err) => println!("start failed: {err}"),
                }
            }

            "stop" => {
                let id = parts.next().unwrap_or_default().trim();
                if id.is_empty() {
                    println!("usage: stop <module_id>");
                    continue;
                }
                match host.stop_module(id) {
                    Ok(()) => println!("stopped {id}"),
                    Err(err) => println!("stop failed: {err}"),
                }
            }

            "remove" => {
                let id = parts.next().unwrap_or_default().trim();
                if id.is_empty() {
                    println!("usage: remove <module_id>");
                    continue;
                }
                match host.remove_module(id) {
                    Ok(()) => println!("removed {id}"),
                    Err(err) => println!("remove failed: {err}"),
                }
            }

            // ── Privilege management ──────────────────────────────────────────
            "privilege" => match host.enter_privileged() {
                Ok(()) => println!("kernel mode: privileged"),
                Err(err) => println!("privilege escalation failed: {err}"),
            },

            "unprivilege" => match host.exit_privileged() {
                Ok(()) => println!("kernel mode: user"),
                Err(err) => println!("privilege drop failed: {err}"),
            },

            // ── Health check ──────────────────────────────────────────────────
            "check" => {
                let filter = parts.next().map(str::trim).filter(|x| !x.is_empty());
                let results = host.refresh_component_health(filter)?;
                if results.is_empty() {
                    println!("no components matched check filter");
                } else {
                    for line in results {
                        println!("{line}");
                    }
                }
            }

            // ── Events ────────────────────────────────────────────────────────
            "publish" => {
                let topic   = parts.next().unwrap_or_default().trim();
                let payload = parts.next().unwrap_or("{}").trim();
                if topic.is_empty() {
                    println!("usage: publish <topic> <payload_json>");
                    continue;
                }
                match host.publish_event(topic, payload) {
                    Ok(()) => println!("published topic={topic}"),
                    Err(err) => println!("publish failed: {err}"),
                }
            }

            _ => {
                println!("unknown command: {command}");
                print_help();
            }
        }
    }

    Ok(())
}

fn print_help() {
    println!("commands:");
    println!("  help                          show command list");
    println!("  status                        show host summary line");
    println!("  boot                          boot orchestrator and register all components");
    println!("  bootstrap [core|office]       bootstrap kernel platform components");
    println!("  tick                          print host/module runtime summary");
    println!("  modules                       list module runtime states");
    println!("  components                    list registered platform components");
    println!("  show <component_id>           show full details for a component");
    println!("  stats                         show live kernel stats snapshot");
    println!("  start <module_id>             start a registered module via kernel");
    println!("  stop  <module_id>             stop a running module via kernel");
    println!("  remove <module_id>            deregister a module and release its resources");
    println!("  privilege                     elevate kernel session to privileged mode");
    println!("  unprivilege                   drop kernel session to user mode");
    println!("  check [component_filter]      probe health for all or selected components");
    println!("  publish <topic> <payload>     publish a custom event through kernel bridge");
    println!("  exit | quit                   leave shell");
}