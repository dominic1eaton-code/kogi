use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{self, Read};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use kogi_spaces::{PublishSpaceRequest, SpacesConfig, SpacesSystem};

#[derive(Deserialize, Default)]
struct SpacesRequest {
    action: Option<String>,
    payload: Option<Value>,
}

fn main() {
    let input = read_input();
    let request = parse_request(&input);
    let mut action = request
        .action
        .unwrap_or_else(|| "state".to_string())
        .to_lowercase();
    if let Some(stripped) = action.strip_prefix("kogi_spaces_") {
        action = stripped.to_string();
    }
    if let Some(stripped) = action.strip_prefix("spaces_") {
        action = stripped.to_string();
    }

    let mut system = SpacesSystem::new(SpacesConfig::default())
        .expect("spaces init");

    let response = match action.as_str() {
        "health" => json!({
            "ok": true,
            "service": "kogi-spaces",
            "version": env!("CARGO_PKG_VERSION"),
            "state": system.state(),
            "timestamp_ms": now_ms(),
        }),
        "state" => to_json_value(system.state()),
        "refresh" => {
            let _ = system.refresh();
            to_json_value(system.state())
        }
        "spaces" => to_json_value(system.spaces()),
        "space" => space_by_id(&system, request.payload.as_ref()),
        "spaces_by_owner" => spaces_by_owner(&system, request.payload.as_ref()),
        "members" => members_by_space(&system, request.payload.as_ref()),
        "rooms" => to_json_value(system.rooms()),
        "channels" => to_json_value(system.channels()),
        "events" => to_json_value(system.events()),
        "workspaces" => to_json_value(system.workspaces()),
        "publish_space" => publish_space(&mut system, request.payload.as_ref()),
        "dashboard_snapshot" | "dashboard" => kogi_spaces::sample_dashboard_snapshot(),
        "feed_snapshot" | "feed" => kogi_spaces::sample_feed_snapshot(),
        "timeline_snapshot" | "timeline" => kogi_spaces::sample_timeline_snapshot(),
        "rooms_snapshot" => kogi_spaces::sample_rooms_snapshot(),
        "channels_snapshot" => kogi_spaces::sample_channels_snapshot(),
        "events_snapshot" => kogi_spaces::sample_events_snapshot(),
        "network_overview_snapshot" | "network_overview" => kogi_spaces::sample_network_overview_snapshot(),
        "network_linknet_snapshot" | "network_linknet" => kogi_spaces::sample_network_linknet_snapshot(),
        "network_linktree_snapshot" | "network_linktree" => kogi_spaces::sample_network_linktree_snapshot(),
        "network_linkforest_snapshot" | "network_linkforest" => kogi_spaces::sample_network_linkforest_snapshot(),
        _ => json!({
            "error": "unknown_action",
            "action": action,
            "timestamp_ms": now_ms(),
        }),
    };

    println!("{}", response);
}

fn space_by_id(system: &SpacesSystem, payload: Option<&Value>) -> Value {
    let Some(id) = payload.and_then(|v| v.get("space_id")).and_then(|v| v.as_str()) else {
        return json!({ "error": "space_id required" });
    };
    let Ok(uuid) = Uuid::parse_str(id) else {
        return json!({ "error": "invalid space_id" });
    };
    system
        .space_by_id(uuid)
        .map(to_json_value)
        .unwrap_or_else(|| json!({ "error": "not_found" }))
}

fn spaces_by_owner(system: &SpacesSystem, payload: Option<&Value>) -> Value {
    let Some(id) = payload.and_then(|v| v.get("owner_id")).and_then(|v| v.as_str()) else {
        return json!({ "error": "owner_id required" });
    };
    let Ok(uuid) = Uuid::parse_str(id) else {
        return json!({ "error": "invalid owner_id" });
    };
    to_json_value(system.spaces_for_owner(uuid))
}

fn members_by_space(system: &SpacesSystem, payload: Option<&Value>) -> Value {
    let Some(id) = payload.and_then(|v| v.get("space_id")).and_then(|v| v.as_str()) else {
        return json!({ "error": "space_id required" });
    };
    let Ok(uuid) = Uuid::parse_str(id) else {
        return json!({ "error": "invalid space_id" });
    };
    to_json_value(system.members(uuid))
}

fn publish_space(system: &mut SpacesSystem, payload: Option<&Value>) -> Value {
    let Some(payload) = payload else {
        return json!({ "error": "payload required" });
    };
    let Ok(req) = serde_json::from_value::<PublishSpaceRequest>(payload.clone()) else {
        return json!({ "error": "invalid payload" });
    };
    match system.publish_space(req) {
        Ok(space) => to_json_value(space),
        Err(err) => json!({ "error": err.to_string() }),
    }
}

fn to_json_value<T: serde::Serialize>(value: T) -> Value {
    serde_json::to_value(value).unwrap_or_else(|_| json!({ "error": "serialize" }))
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

fn parse_request(raw: &str) -> SpacesRequest {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return SpacesRequest::default();
    }
    serde_json::from_str(trimmed).unwrap_or_default()
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
