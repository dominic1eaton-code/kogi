use kogi_home_module::{to_json, HomeModule};
use serde::Deserialize;
use serde_json::json;
use std::io::{self, Read};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize, Default)]
struct HomeRequest {
    action: Option<String>,
}

fn main() {
    let input = read_input();
    let request = parse_request(&input);
    let action = request
        .action
        .unwrap_or_else(|| "overview".to_string())
        .to_lowercase();

    let module = HomeModule::mvp();
    let response = match action.as_str() {
        "overview" => to_json(&module.overview()),
        "dashboard" => to_json(&module.dashboard_snapshot()),
        "profile" => to_json(&module.profile_snapshot()),
        "workspace" => to_json(&module.workspace_snapshot()),
        _ => to_json(&json!({
            "error": "unknown_action",
            "action": action,
            "timestamp_ms": now_ms(),
        })),
    };

    println!("{}", response);
}

fn read_input() -> String {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--request" {
            if let Some(payload) = args.next() {
                return payload;
            }
        }
    }

    let mut input = String::new();
    let _ = io::stdin().read_to_string(&mut input);
    input
}

fn parse_request(raw: &str) -> HomeRequest {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return HomeRequest { action: None };
    }

    serde_json::from_str(trimmed).unwrap_or_default()
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
