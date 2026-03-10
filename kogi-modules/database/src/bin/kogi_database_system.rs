use kogi_database_module::{DatabaseRequest, DatabaseResponse, DatabaseSystem, to_json};
use serde_json::json;
use std::io::{self, Read};
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let input = read_input();
    let request = match parse_request(&input) {
        Ok(request) => request,
        Err(err) => {
            let response = DatabaseResponse {
                request_id: "req-invalid".to_string(),
                action: "error".to_string(),
                status: "error".to_string(),
                message: err.clone(),
                timestamp_ms: now_ms(),
                data: json!({"error": err}),
                access: None,
                metrics: None,
                warnings: Vec::new(),
            };
            println!("{}", to_json(&response));
            return;
        }
    };

    let system = DatabaseSystem::from_env();
    let response = system.handle_request(request);
    println!("{}", to_json(&response));
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

fn parse_request(raw: &str) -> Result<DatabaseRequest, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(DatabaseRequest {
            action: "status".to_string(),
            ..DatabaseRequest::default()
        });
    }

    serde_json::from_str(trimmed).map_err(|err| format!("invalid request json: {err}"))
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
