mod state;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use serde_json::Value;
use state::ServerState;

fn main() {
    let listener = match TcpListener::bind("127.0.0.1:8080") {
        Ok(l) => l,
        Err(err) => {
            eprintln!("failed to bind server: {err}");
            std::process::exit(1);
        }
    };

    let state = Arc::new(Mutex::new(ServerState::mvp()));
    println!("kogi-server listening on http://127.0.0.1:8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let state = Arc::clone(&state);
                handle_connection(stream, state);
            }
            Err(err) => eprintln!("connection error: {err}"),
        }
    }
}

fn handle_connection(mut stream: TcpStream, state: Arc<Mutex<ServerState>>) {
    let mut buffer = [0_u8; 4096];
    let bytes = match stream.read(&mut buffer) {
        Ok(n) => n,
        Err(_) => return,
    };

    let request = String::from_utf8_lossy(&buffer[..bytes]);
    let first_line = request.lines().next().unwrap_or("GET / HTTP/1.1");

    let request_body = request.split("\r\n\r\n").nth(1).unwrap_or("");
    let (status, content_type, body) = route(first_line, request_body, &state);
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );

    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

fn route(
    first_line: &str,
    request_body: &str,
    state: &Arc<Mutex<ServerState>>,
) -> (&'static str, &'static str, String) {
    if first_line.starts_with("GET /health") {
        return (
            "200 OK",
            "application/json",
            "{\"status\":\"ok\",\"service\":\"kogi-server\"}".to_string(),
        );
    }

    if first_line.starts_with("POST /api/v1/office/dashboard/notifications/ack") {
        let notification_id = json_string(request_body, "notification_id")
            .unwrap_or_else(|| "notif-001".to_string());
        let mut s = state.lock().expect("state lock poisoned");
        return (
            "200 OK",
            "application/json",
            s.office_ack_notification_json(&notification_id),
        );
    }

    if first_line.starts_with("POST /api/v1/office/portfolio/items") {
        let item_type = json_string(request_body, "item_type")
            .unwrap_or_else(|| "project".to_string());
        let name = json_string(request_body, "name")
            .unwrap_or_else(|| "Office Item".to_string());
        let status = json_string(request_body, "status")
            .unwrap_or_else(|| "active".to_string());
        let mut s = state.lock().expect("state lock poisoned");
        return (
            "200 OK",
            "application/json",
            s.office_create_portfolio_item_json(&item_type, &name, &status),
        );
    }

    if first_line.starts_with("POST /api/v1/office/timeline/events") {
        let calendar_id = json_string(request_body, "calendar_id")
            .unwrap_or_else(|| "cal-work".to_string());
        let title = json_string(request_body, "title")
            .unwrap_or_else(|| "Office Event".to_string());
        let kind = json_string(request_body, "kind")
            .unwrap_or_else(|| "milestone".to_string());
        let scheduled_for = json_string(request_body, "scheduled_for")
            .unwrap_or_else(|| "2026-03-12T18:00:00Z".to_string());
        let mut s = state.lock().expect("state lock poisoned");
        return (
            "200 OK",
            "application/json",
            s.office_create_timeline_event_json(&calendar_id, &title, &kind, &scheduled_for),
        );
    }

    if first_line.starts_with("POST /api/v1/office/workspace/stories") {
        let title = json_string(request_body, "title")
            .unwrap_or_else(|| "As a worker, I can run office workflows".to_string());
        let points = json_i32(request_body, "points").unwrap_or(5);
        let mut s = state.lock().expect("state lock poisoned");
        return (
            "200 OK",
            "application/json",
            s.office_create_workspace_story_json(&title, points),
        );
    }

    if first_line.starts_with("POST /api/v1/office/assistant/subscriptions") {
        let topic = json_string(request_body, "topic")
            .unwrap_or_else(|| "office.dashboard.alerts".to_string());
        let mut s = state.lock().expect("state lock poisoned");
        return (
            "200 OK",
            "application/json",
            s.office_create_assistant_subscription_json(&topic),
        );
    }

    if first_line.starts_with("GET /api/v1/modules") {
        let s = state.lock().expect("state lock poisoned");
        return ("200 OK", "application/json", s.modules_json());
    }

    if first_line.starts_with("GET /api/v1/system") {
        let s = state.lock().expect("state lock poisoned");
        return ("200 OK", "application/json", s.summary_json());
    }

    if first_line.starts_with("GET /api/v1/engine/system") {
        let s = state.lock().expect("state lock poisoned");
        return ("200 OK", "application/json", s.engine_overview_json());
    }

    if first_line.starts_with("GET /api/v1/ims/identities") {
        let s = state.lock().expect("state lock poisoned");
        return ("200 OK", "application/json", s.identities_json());
    }

    if first_line.starts_with("GET /api/v1/ims/profiles") {
        let s = state.lock().expect("state lock poisoned");
        return ("200 OK", "application/json", s.profiles_json());
    }

    if first_line.starts_with("GET /api/v1/autonomy/capabilities") {
        let s = state.lock().expect("state lock poisoned");
        return ("200 OK", "application/json", s.autonomy_capabilities_json());
    }

    if first_line.starts_with("GET /api/v1/kernel/modules/isolation") {
        let s = state.lock().expect("state lock poisoned");
        return ("200 OK", "application/json", s.module_isolation_json());
    }

    if first_line.starts_with("GET /api/v1/office/dashboard") {
        let s = state.lock().expect("state lock poisoned");
        return ("200 OK", "application/json", s.office_dashboard_json());
    }

    if first_line.starts_with("GET /api/v1/office/portfolio") {
        let s = state.lock().expect("state lock poisoned");
        return ("200 OK", "application/json", s.office_portfolio_json());
    }

    if first_line.starts_with("GET /api/v1/office/timeline") {
        let s = state.lock().expect("state lock poisoned");
        return ("200 OK", "application/json", s.office_timeline_json());
    }

    if first_line.starts_with("GET /api/v1/office/workspace") {
        let s = state.lock().expect("state lock poisoned");
        return ("200 OK", "application/json", s.office_workspace_json());
    }

    if first_line.starts_with("GET /api/v1/office/assistant") {
        let s = state.lock().expect("state lock poisoned");
        return ("200 OK", "application/json", s.office_assistant_json());
    }

    if first_line.starts_with("GET /api/v1/office") {
        let s = state.lock().expect("state lock poisoned");
        return ("200 OK", "application/json", s.office_overview_json());
    }

    if first_line.starts_with("GET /api/v1/screens/unified/flat") {
        let s = state.lock().expect("state lock poisoned");
        return ("200 OK", "text/plain", s.unified_screens_flat());
    }

    if first_line.starts_with("GET /api/v1/screens/unified") {
        let s = state.lock().expect("state lock poisoned");
        return ("200 OK", "application/json", s.unified_screens_json());
    }

    (
        "404 Not Found",
        "application/json",
        "{\"error\":\"not_found\"}".to_string(),
    )
}

fn json_string(body: &str, key: &str) -> Option<String> {
    if body.trim().is_empty() {
        return None;
    }
    let parsed: Value = serde_json::from_str(body).ok()?;
    parsed.get(key)?.as_str().map(ToString::to_string)
}

fn json_i32(body: &str, key: &str) -> Option<i32> {
    if body.trim().is_empty() {
        return None;
    }
    let parsed: Value = serde_json::from_str(body).ok()?;
    parsed.get(key)?.as_i64().and_then(|x| i32::try_from(x).ok())
}
