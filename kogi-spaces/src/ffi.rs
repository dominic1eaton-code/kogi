use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use kogi_portfolio::ScorePreference;

use crate::snapshots::*;
use crate::system::{PublishSpaceRequest, SpacesConfig, SpacesState, SpacesSystem};
use crate::model::{SpaceKind, SpaceVisibility};

static SPACES_SYSTEM: OnceLock<Mutex<SpacesSystem>> = OnceLock::new();

#[derive(Debug, Deserialize)]
struct SpacesInitRequest {
    owner_id: Option<String>,
    grid_name: Option<String>,
    node_id: Option<String>,
    environment: Option<String>,
    data_dir: Option<String>,
    default_visibility: Option<String>,
    default_kind: Option<String>,
    score_preference: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct SpaceRequest {
    space_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OwnerRequest {
    owner_id: Option<String>,
}

fn with_system<F, R>(op: F) -> Result<R, String>
where
    F: FnOnce(&mut SpacesSystem) -> Result<R, String>,
{
    let system = SPACES_SYSTEM.get_or_init(|| {
        let cfg = SpacesConfig::default();
        let system = SpacesSystem::new(cfg).expect("spaces system init");
        Mutex::new(system)
    });
    let mut guard = system.lock().map_err(|_| "spaces lock poisoned".to_string())?;
    op(&mut guard)
}

fn parse_score_preference(value: &serde_json::Value) -> Option<ScorePreference> {
    match value {
        serde_json::Value::String(s) => match s.to_lowercase().as_str() {
            "policy" | "policy_first" | "policy-first" => Some(ScorePreference::PolicyFirst),
            "ai" | "ai_first" | "ai-first" => Some(ScorePreference::AiFirst),
            "blend" => Some(ScorePreference::Blend { ai_weight: 0.5 }),
            _ => None,
        },
        serde_json::Value::Object(map) => {
            let mode = map.get("mode").and_then(|v| v.as_str()).unwrap_or("policy");
            let weight = map.get("ai_weight").and_then(|v| v.as_f64()).unwrap_or(0.5);
            match mode.to_lowercase().as_str() {
                "ai" | "ai_first" | "ai-first" => Some(ScorePreference::AiFirst),
                "blend" => Some(ScorePreference::Blend { ai_weight: weight }),
                _ => Some(ScorePreference::PolicyFirst),
            }
        }
        _ => None,
    }
}

fn parse_visibility(value: &str) -> Option<SpaceVisibility> {
    match value.to_lowercase().as_str() {
        "public" => Some(SpaceVisibility::Public),
        "community" | "protected" => Some(SpaceVisibility::Community),
        "inviteonly" | "invite_only" | "invite-only" | "unlisted" => Some(SpaceVisibility::InviteOnly),
        "private" => Some(SpaceVisibility::Private),
        _ => None,
    }
}

fn parse_kind(value: &str) -> Option<SpaceKind> {
    match value.to_lowercase().as_str() {
        "personal" => Some(SpaceKind::Personal),
        "team" | "squad" => Some(SpaceKind::Team),
        "community" | "space" => Some(SpaceKind::Community),
        "organization" | "org" => Some(SpaceKind::Organization),
        "workspace" | "workspaces" => Some(SpaceKind::Workspace),
        "room" => Some(SpaceKind::Room),
        "channel" => Some(SpaceKind::Channel),
        "event" => Some(SpaceKind::Event),
        "directory" => Some(SpaceKind::Directory),
        "network" => Some(SpaceKind::Network),
        "linknet" => Some(SpaceKind::LinkNet),
        "linktree" => Some(SpaceKind::LinkTree),
        "linkforest" => Some(SpaceKind::LinkForest),
        "hub" => Some(SpaceKind::Hub),
        "guild" => Some(SpaceKind::Guild),
        "collective" | "cooperative" | "coop" => Some(SpaceKind::Collective),
        _ => None,
    }
}

fn build_config(req: Option<SpacesInitRequest>) -> SpacesConfig {
    let mut cfg = SpacesConfig::default();
    if let Some(req) = req {
        if let Some(owner_id) = req.owner_id.and_then(|s| Uuid::parse_str(&s).ok()) {
            cfg.portfolio_config.owner_id = owner_id;
        }
        if let Some(grid_name) = req.grid_name {
            cfg.portfolio_config.grid_name = grid_name;
        }
        if let Some(node_id) = req.node_id {
            cfg.portfolio_config.node_id = node_id;
        }
        if let Some(environment) = req.environment {
            cfg.portfolio_config.environment = environment;
        }
        if let Some(data_dir) = req.data_dir {
            cfg.portfolio_config.data_dir = PathBuf::from(data_dir);
        }
        if let Some(default_visibility) = req.default_visibility.and_then(|s| parse_visibility(&s)) {
            cfg.default_visibility = default_visibility;
        }
        if let Some(default_kind) = req.default_kind.and_then(|s| parse_kind(&s)) {
            cfg.default_kind = default_kind;
        }
        if let Some(score_pref) = req.score_preference.and_then(|v| parse_score_preference(&v)) {
            cfg.portfolio_config.score_preference = score_pref;
        }
    }
    cfg
}

fn json_to_c_string(value: serde_json::Value) -> *mut c_char {
    let payload = serde_json::to_string(&value).unwrap_or_else(|_| "{\"error\":\"serialize\"}".to_string());
    CString::new(payload).unwrap().into_raw()
}

fn c_string_to_opt(input: *const c_char) -> Option<String> {
    if input.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(input).to_str().ok().map(|s| s.to_string()) }
}

#[no_mangle]
pub extern "C" fn kogi_free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(ptr);
    }
}

#[no_mangle]
pub extern "C" fn kogi_spaces_init(args_json: *const c_char) -> *mut c_char {
    let request = c_string_to_opt(args_json)
        .and_then(|s| serde_json::from_str::<SpacesInitRequest>(&s).ok());
    let cfg = build_config(request);
    let result = with_system(|system| {
        let new_system = SpacesSystem::new(cfg).map_err(|e| e.to_string())?;
        *system = new_system;
        Ok(system.state())
    });

    match result {
        Ok(state) => json_to_c_string(serde_json::to_value(state).unwrap_or(json!({"ok": true}))),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_spaces_health(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(system.state()));
    match result {
        Ok(state) => json_to_c_string(json!({
            "ok": true,
            "service": "kogi-spaces",
            "version": env!("CARGO_PKG_VERSION"),
            "state": state,
        })),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_spaces_state(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok::<SpacesState, String>(system.state()));
    match result {
        Ok(state) => json_to_c_string(serde_json::to_value(state).unwrap_or(json!({"ok": true}))),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_spaces_refresh(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| {
        system.refresh().map_err(|e| e.to_string())?;
        Ok(system.state())
    });
    match result {
        Ok(state) => json_to_c_string(serde_json::to_value(state).unwrap_or(json!({"ok": true}))),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_spaces_spaces(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(system.spaces()));
    match result {
        Ok(spaces) => json_to_c_string(json!({ "spaces": spaces })),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_spaces_space(args_json: *const c_char) -> *mut c_char {
    let request = c_string_to_opt(args_json)
        .and_then(|s| serde_json::from_str::<SpaceRequest>(&s).ok());
    let space_id = request
        .and_then(|r| r.space_id)
        .and_then(|s| Uuid::parse_str(&s).ok());
    let Some(space_id) = space_id else {
        return json_to_c_string(json!({"ok": false, "error": "space_id required"}));
    };

    let result = with_system(|system| Ok(system.space_by_id(space_id)));
    match result {
        Ok(Some(space)) => json_to_c_string(serde_json::to_value(space).unwrap_or(json!({"ok": true}))),
        Ok(None) => json_to_c_string(json!({"ok": false, "error": "not_found"})),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_spaces_spaces_by_owner(args_json: *const c_char) -> *mut c_char {
    let request = c_string_to_opt(args_json)
        .and_then(|s| serde_json::from_str::<OwnerRequest>(&s).ok());
    let owner_id = request
        .and_then(|r| r.owner_id)
        .and_then(|s| Uuid::parse_str(&s).ok());
    let Some(owner_id) = owner_id else {
        return json_to_c_string(json!({"ok": false, "error": "owner_id required"}));
    };

    let result = with_system(|system| Ok(system.spaces_for_owner(owner_id)));
    match result {
        Ok(spaces) => json_to_c_string(json!({ "spaces": spaces })),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_spaces_members(args_json: *const c_char) -> *mut c_char {
    let request = c_string_to_opt(args_json)
        .and_then(|s| serde_json::from_str::<SpaceRequest>(&s).ok());
    let space_id = request
        .and_then(|r| r.space_id)
        .and_then(|s| Uuid::parse_str(&s).ok());
    let Some(space_id) = space_id else {
        return json_to_c_string(json!({"ok": false, "error": "space_id required"}));
    };

    let result = with_system(|system| Ok(system.members(space_id)));
    match result {
        Ok(members) => json_to_c_string(json!({ "members": members })),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_spaces_rooms(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(system.rooms()));
    match result {
        Ok(rooms) => json_to_c_string(json!({ "rooms": rooms })),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_spaces_channels(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(system.channels()));
    match result {
        Ok(channels) => json_to_c_string(json!({ "channels": channels })),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_spaces_events(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(system.events()));
    match result {
        Ok(events) => json_to_c_string(json!({ "events": events })),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_spaces_workspaces(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(system.workspaces()));
    match result {
        Ok(workspaces) => json_to_c_string(json!({ "workspaces": workspaces })),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_spaces_publish_space(args_json: *const c_char) -> *mut c_char {
    let request = c_string_to_opt(args_json)
        .and_then(|s| serde_json::from_str::<PublishSpaceRequest>(&s).ok());
    let Some(request) = request else {
        return json_to_c_string(json!({"ok": false, "error": "invalid payload"}));
    };

    let result = with_system(|system| system.publish_space(request).map_err(|e| e.to_string()));
    match result {
        Ok(space) => json_to_c_string(serde_json::to_value(space).unwrap_or(json!({"ok": true}))),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_spaces_dashboard_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_dashboard_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_spaces_feed_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_feed_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_spaces_timeline_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_timeline_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_spaces_rooms_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_rooms_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_spaces_channels_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_channels_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_spaces_events_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_events_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_spaces_network_overview_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_network_overview_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_spaces_network_linknet_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_network_linknet_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_spaces_network_linktree_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_network_linktree_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_spaces_network_linkforest_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_network_linkforest_snapshot())
}
