mod state;
mod runtime;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use kogi_host::{
    NewAffiliate, NewAffiliateLink, NewProvider, NewProviderDataAsset, NewProviderMetadata,
    NewProviderPlatform, NewProviderResource, NewProviderVersion,
};
use serde_json::Value;
use runtime::{env_bool, Runtime};
use state::ServerState;

fn main() {
    let mut debug = env_bool("KOGI_DEBUG");
    let mut silent = env_bool("KOGI_SILENT");
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--debug" => debug = true,
            "--silent" => silent = true,
            _ => {}
        }
    }

    let runtime = Arc::new(Runtime::new("kogi-server", debug, silent));
    runtime.state("init");
    runtime.status("startup", "binding=127.0.0.1:8080");
    if runtime.debug_enabled() {
        runtime.debug("debug enabled");
    }

    let rt_for_signal = runtime.clone();
    if let Err(err) = ctrlc::set_handler(move || {
        rt_for_signal.state("shutting_down");
        rt_for_signal.status("signal", "interrupt");
        std::process::exit(0);
    }) {
        runtime.error(&format!("failed to set signal handler: {err}"));
    }

    let listener = match TcpListener::bind("127.0.0.1:8080") {
        Ok(l) => l,
        Err(err) => {
            runtime.state("shutting_down");
            runtime.error(&format!("failed to bind server: {err}"));
            std::process::exit(1);
        }
    };

    runtime.state("configure");
    let state = match ServerState::bootstrap(runtime.clone()) {
        Ok(state) => Arc::new(Mutex::new(state)),
        Err(err) => {
            runtime.state("shutting_down");
            runtime.error(&format!("failed to bootstrap host app: {err}"));
            std::process::exit(1);
        }
    };
    runtime.state("running");
    runtime.status("ok", "listening=http://127.0.0.1:8080");
    runtime.info("listening on http://127.0.0.1:8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let state = Arc::clone(&state);
                let runtime = runtime.clone();
                handle_connection(stream, state, runtime);
            }
            Err(err) => runtime.error(&format!("connection error: {err}")),
        }
    }
}

fn handle_connection(mut stream: TcpStream, state: Arc<Mutex<ServerState>>, runtime: Arc<Runtime>) {
    let mut buffer = [0_u8; 4096];
    let bytes = match stream.read(&mut buffer) {
        Ok(n) => n,
        Err(_) => return,
    };

    let request = String::from_utf8_lossy(&buffer[..bytes]);
    let first_line = request.lines().next().unwrap_or("GET / HTTP/1.1");
    let mut parts = first_line.split_whitespace();
    let method = parts.next().unwrap_or("-");
    let path = parts.next().unwrap_or("-");
    runtime.debug(&format!(
        "message received kind=http method={method} path={path} bytes={bytes}"
    ));

    let request_body = request.split("\r\n\r\n").nth(1).unwrap_or("");
    let (status, content_type, body) = route(first_line, request_body, &state, &runtime);
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );

    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
    runtime.debug(&format!(
        "message sent kind=http method={method} path={path} status={status} bytes={}",
        body.len()
    ));
}

fn route(
    first_line: &str,
    request_body: &str,
    state: &Arc<Mutex<ServerState>>,
    runtime: &Arc<Runtime>,
) -> (&'static str, &'static str, String) {
    if first_line.starts_with("GET /health") {
        runtime.status("ok", "health_check");
        return (
            "200 OK",
            "application/json",
            "{\"status\":\"ok\",\"service\":\"kogi-server\"}".to_string(),
        );
    }

    if first_line.starts_with("POST /api/v1/messages") {
        let topic = json_string(request_body, "topic")
            .unwrap_or_else(|| "host.message".to_string());
        let payload = json_payload_string(request_body, "payload")
            .unwrap_or_else(|| "{}".to_string());
        let source = json_string(request_body, "source")
            .unwrap_or_else(|| "client".to_string());
        let target = json_string(request_body, "target").unwrap_or_default();
        runtime.message("receive", &topic, &source, &target, &payload);
        let mut s = state.lock().expect("state lock poisoned");
        return (
            "200 OK",
            "application/json",
            s.relay_message_json(&topic, &payload, &source, &target),
        );
    }

    if first_line.starts_with("GET /api/v1/messages") {
        let limit = query_param(first_line, "limit")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(100);
        let topic = query_param(first_line, "topic");
        runtime.debug(&format!("message receive history limit={limit} topic={}", topic.clone().unwrap_or_default()));
        let s = state.lock().expect("state lock poisoned");
        return (
            "200 OK",
            "application/json",
            s.gateway_history_json(limit, topic.as_deref()),
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

    if first_line.starts_with("GET /api/v1/providers/platforms") {
        let s = state.lock().expect("state lock poisoned");
        return ("200 OK", "application/json", s.providers_platforms_json());
    }

    if first_line.starts_with("GET /api/v1/providers/providers") {
        let s = state.lock().expect("state lock poisoned");
        return ("200 OK", "application/json", s.providers_list_json());
    }

    if first_line.starts_with("GET /api/v1/providers/resources") {
        let s = state.lock().expect("state lock poisoned");
        return ("200 OK", "application/json", s.providers_resources_json());
    }

    if first_line.starts_with("GET /api/v1/providers/versions") {
        let s = state.lock().expect("state lock poisoned");
        return ("200 OK", "application/json", s.providers_versions_json());
    }

    if first_line.starts_with("GET /api/v1/providers/metadata") {
        let s = state.lock().expect("state lock poisoned");
        return ("200 OK", "application/json", s.providers_metadata_json());
    }

    if first_line.starts_with("GET /api/v1/providers/data") {
        let s = state.lock().expect("state lock poisoned");
        return ("200 OK", "application/json", s.providers_data_assets_json());
    }

    if first_line.starts_with("GET /api/v1/providers/affiliates") {
        let s = state.lock().expect("state lock poisoned");
        return ("200 OK", "application/json", s.providers_affiliates_json());
    }

    if first_line.starts_with("GET /api/v1/providers/affiliate-links") {
        let s = state.lock().expect("state lock poisoned");
        return ("200 OK", "application/json", s.providers_affiliate_links_json());
    }

    if first_line.starts_with("GET /api/v1/providers") {
        let s = state.lock().expect("state lock poisoned");
        return ("200 OK", "application/json", s.providers_snapshot_json());
    }

    if first_line.starts_with("POST /api/v1/providers/platforms") {
        let req: NewProviderPlatform = match serde_json::from_str(request_body) {
            Ok(value) => value,
            Err(err) => {
                return (
                    "400 Bad Request",
                    "application/json",
                    format!("{{\"error\":\"{}\"}}", escape_json(&err.to_string())),
                )
            }
        };
        let mut s = state.lock().expect("state lock poisoned");
        return (
            "200 OK",
            "application/json",
            s.providers_register_platform_json(req),
        );
    }

    if first_line.starts_with("POST /api/v1/providers/providers") {
        let req: NewProvider = match serde_json::from_str(request_body) {
            Ok(value) => value,
            Err(err) => {
                return (
                    "400 Bad Request",
                    "application/json",
                    format!("{{\"error\":\"{}\"}}", escape_json(&err.to_string())),
                )
            }
        };
        let mut s = state.lock().expect("state lock poisoned");
        return (
            "200 OK",
            "application/json",
            s.providers_register_json(req),
        );
    }

    if first_line.starts_with("POST /api/v1/providers/resources") {
        let req: NewProviderResource = match serde_json::from_str(request_body) {
            Ok(value) => value,
            Err(err) => {
                return (
                    "400 Bad Request",
                    "application/json",
                    format!("{{\"error\":\"{}\"}}", escape_json(&err.to_string())),
                )
            }
        };
        let mut s = state.lock().expect("state lock poisoned");
        return (
            "200 OK",
            "application/json",
            s.providers_add_resource_json(req),
        );
    }

    if first_line.starts_with("POST /api/v1/providers/versions") {
        let req: NewProviderVersion = match serde_json::from_str(request_body) {
            Ok(value) => value,
            Err(err) => {
                return (
                    "400 Bad Request",
                    "application/json",
                    format!("{{\"error\":\"{}\"}}", escape_json(&err.to_string())),
                )
            }
        };
        let mut s = state.lock().expect("state lock poisoned");
        return (
            "200 OK",
            "application/json",
            s.providers_add_version_json(req),
        );
    }

    if first_line.starts_with("POST /api/v1/providers/metadata") {
        let req: NewProviderMetadata = match serde_json::from_str(request_body) {
            Ok(value) => value,
            Err(err) => {
                return (
                    "400 Bad Request",
                    "application/json",
                    format!("{{\"error\":\"{}\"}}", escape_json(&err.to_string())),
                )
            }
        };
        let mut s = state.lock().expect("state lock poisoned");
        return (
            "200 OK",
            "application/json",
            s.providers_set_metadata_json(req),
        );
    }

    if first_line.starts_with("POST /api/v1/providers/data") {
        let req: NewProviderDataAsset = match serde_json::from_str(request_body) {
            Ok(value) => value,
            Err(err) => {
                return (
                    "400 Bad Request",
                    "application/json",
                    format!("{{\"error\":\"{}\"}}", escape_json(&err.to_string())),
                )
            }
        };
        let mut s = state.lock().expect("state lock poisoned");
        return (
            "200 OK",
            "application/json",
            s.providers_add_data_asset_json(req),
        );
    }

    if first_line.starts_with("POST /api/v1/providers/affiliates") {
        let req: NewAffiliate = match serde_json::from_str(request_body) {
            Ok(value) => value,
            Err(err) => {
                return (
                    "400 Bad Request",
                    "application/json",
                    format!("{{\"error\":\"{}\"}}", escape_json(&err.to_string())),
                )
            }
        };
        let mut s = state.lock().expect("state lock poisoned");
        return (
            "200 OK",
            "application/json",
            s.providers_register_affiliate_json(req),
        );
    }

    if first_line.starts_with("POST /api/v1/providers/affiliate-links") {
        let req: NewAffiliateLink = match serde_json::from_str(request_body) {
            Ok(value) => value,
            Err(err) => {
                return (
                    "400 Bad Request",
                    "application/json",
                    format!("{{\"error\":\"{}\"}}", escape_json(&err.to_string())),
                )
            }
        };
        let mut s = state.lock().expect("state lock poisoned");
        return (
            "200 OK",
            "application/json",
            s.providers_add_affiliate_link_json(req),
        );
    }

    if first_line.starts_with("GET /api/v1/host/components") {
        let s = state.lock().expect("state lock poisoned");
        return ("200 OK", "application/json", s.host_components_json());
    }

    if first_line.starts_with("GET /api/v1/host") {
        let s = state.lock().expect("state lock poisoned");
        return ("200 OK", "application/json", s.host_summary_json());
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

    if first_line.starts_with("GET /api/v1/engine/runtime") {
        let s = state.lock().expect("state lock poisoned");
        return match s.engine_service_runtime() {
            Ok(body) => ("200 OK", "application/json", body),
            Err(err) => (
                "502 Bad Gateway",
                "application/json",
                format!("{{\"error\":\"{}\"}}", escape_json(&err.to_string())),
            ),
        };
    }

    if first_line.starts_with("POST /api/v1/engine/control") {
        let action = json_string(request_body, "action").unwrap_or_else(|| "start".to_string());
        runtime.message("receive", "engine.control", "client", "kogi.server", &action);
        let s = state.lock().expect("state lock poisoned");
        return match s.engine_control(&action) {
            Ok(body) => ("200 OK", "application/json", body),
            Err(err) => (
                "502 Bad Gateway",
                "application/json",
                format!("{{\"error\":\"{}\"}}", escape_json(&err.to_string())),
            ),
        };
    }

    if first_line.starts_with("POST /api/v1/engine/ingest") {
        let payload = if request_body.trim().is_empty() {
            "{}".to_string()
        } else {
            request_body.to_string()
        };
        runtime.message("receive", "engine.ingest", "client", "kogi.server", &payload);
        let s = state.lock().expect("state lock poisoned");
        return match s.engine_ingest(&payload) {
            Ok(body) => ("200 OK", "application/json", body),
            Err(err) => (
                "502 Bad Gateway",
                "application/json",
                format!("{{\"error\":\"{}\"}}", escape_json(&err.to_string())),
            ),
        };
    }

    if first_line.starts_with("GET /api/v1/database/runtime") {
        let s = state.lock().expect("state lock poisoned");
        return match s.database_service_runtime() {
            Ok(body) => ("200 OK", "application/json", body),
            Err(err) => (
                "502 Bad Gateway",
                "application/json",
                format!("{{\"error\":\"{}\"}}", escape_json(&err.to_string())),
            ),
        };
    }

    if first_line.starts_with("POST /api/v1/database/query") {
        let sql = json_string(request_body, "sql").unwrap_or_else(|| "select 1".to_string());
        runtime.message("receive", "database.query", "client", "kogi.server", &sql);
        let s = state.lock().expect("state lock poisoned");
        return match s.database_query(&sql) {
            Ok(body) => ("200 OK", "application/json", body),
            Err(err) => (
                "502 Bad Gateway",
                "application/json",
                format!("{{\"error\":\"{}\"}}", escape_json(&err.to_string())),
            ),
        };
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

fn json_payload_string(body: &str, key: &str) -> Option<String> {
    if body.trim().is_empty() {
        return None;
    }
    let parsed: Value = serde_json::from_str(body).ok()?;
    let value = parsed.get(key)?;
    if let Some(str_value) = value.as_str() {
        return Some(str_value.to_string());
    }
    Some(value.to_string())
}

fn query_param(first_line: &str, key: &str) -> Option<String> {
    let path = first_line.split_whitespace().nth(1)?;
    let query = path.split('?').nth(1)?;
    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        let name = parts.next()?.trim();
        let value = parts.next().unwrap_or("").trim();
        if name == key {
            return Some(url_decode(value));
        }
    }
    None
}

fn url_decode(value: &str) -> String {
    let mut decoded = String::new();
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '%' {
            let hi = chars.next();
            let lo = chars.next();
            if let (Some(hi), Some(lo)) = (hi, lo) {
                let hex = format!("{hi}{lo}");
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    decoded.push(byte as char);
                    continue;
                }
            }
            decoded.push('%');
            if let Some(hi) = hi {
                decoded.push(hi);
            }
            if let Some(lo) = lo {
                decoded.push(lo);
            }
        } else if ch == '+' {
            decoded.push(' ');
        } else {
            decoded.push(ch);
        }
    }
    decoded
}

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
