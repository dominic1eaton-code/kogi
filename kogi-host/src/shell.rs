use std::io::{self, Write};

use crate::executive::{HostError, HostExecutive};
use crate::kernel_bridge::KernelBridge;

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
            "help" => print_help(),
            "status" => println!("{}", host.summary_line()),
            "boot" => match host.boot() {
                Ok(()) => println!("host boot completed"),
                Err(err) => println!("boot failed: {err}"),
            },
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
            "publish" => {
                let topic = parts.next().unwrap_or_default().trim();
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
    println!("  help                        show command list");
    println!("  status                      show host summary");
    println!("  boot                        boot orchestrator and register components");
    println!("  tick                        print host/module runtime summary");
    println!("  modules                     list module runtime states");
    println!("  components                  list registered platform components");
    println!("  check [component_filter]    check health for all or selected components");
    println!("  show <component_id>         show full details for a component");
    println!("  publish <topic> <payload>   publish an event through kernel bridge");
    println!("  exit | quit                 leave shell");
}
