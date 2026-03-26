use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use kogi_portfolio::ScorePreference;

use crate::snapshots::*;
use crate::system::{HubConfig, HubState, HubSystem};

static HUB_SYSTEM: OnceLock<Mutex<HubSystem>> = OnceLock::new();

#[derive(Debug, Deserialize)]
struct HubInitRequest {
    owner_id: Option<String>,
    grid_name: Option<String>,
    node_id: Option<String>,
    environment: Option<String>,
    data_dir: Option<String>,
    score_preference: Option<serde_json::Value>,
}

fn with_system<F, R>(op: F) -> Result<R, String>
where
    F: FnOnce(&mut HubSystem) -> Result<R, String>,
{
    let system = HUB_SYSTEM.get_or_init(|| {
        let cfg = HubConfig::default();
        let system = HubSystem::new(cfg).expect("hub system init");
        Mutex::new(system)
    });
    let mut guard = system.lock().map_err(|_| "hub lock poisoned".to_string())?;
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

fn build_config(req: Option<HubInitRequest>) -> HubConfig {
    let mut cfg = HubConfig::default();
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
pub extern "C" fn kogi_hub_init(args_json: *const c_char) -> *mut c_char {
    let request = c_string_to_opt(args_json)
        .and_then(|s| serde_json::from_str::<HubInitRequest>(&s).ok());
    let cfg = build_config(request);
    let result = with_system(|system| {
        let new_system = HubSystem::new(cfg).map_err(|e| e.to_string())?;
        *system = new_system;
        Ok(system.state())
    });

    match result {
        Ok(state) => json_to_c_string(serde_json::to_value(state).unwrap_or(json!({"ok": true}))),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_hub_health(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(system.state()));
    match result {
        Ok(state) => json_to_c_string(json!({
            "ok": true,
            "service": "kogi-hub",
            "version": env!("CARGO_PKG_VERSION"),
            "state": state,
        })),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_hub_state(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok::<HubState, String>(system.state()));
    match result {
        Ok(state) => json_to_c_string(serde_json::to_value(state).unwrap_or(json!({"ok": true}))),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_hub_refresh(_: *const c_char) -> *mut c_char {
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
pub extern "C" fn kogi_hub_entities(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(system.entities()));
    match result {
        Ok(entities) => json_to_c_string(json!({ "entities": entities })),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_hub_members(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(system.members()));
    match result {
        Ok(members) => json_to_c_string(json!({ "members": members })),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_hub_governance(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(system.governance()));
    match result {
        Ok(governance) => json_to_c_string(serde_json::to_value(governance).unwrap_or(json!({"ok": true}))),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_hub_voting(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(system.voting()));
    match result {
        Ok(voting) => json_to_c_string(serde_json::to_value(voting).unwrap_or(json!({"ok": true}))),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_hub_contracts(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(system.contracts()));
    match result {
        Ok(contracts) => json_to_c_string(json!({ "contracts": contracts })),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_hub_rights(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(system.rights()));
    match result {
        Ok(rights) => json_to_c_string(json!({ "rights": rights })),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_hub_ip_assets(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(system.ip_assets()));
    match result {
        Ok(assets) => json_to_c_string(json!({ "assets": assets })),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_hub_allocations(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(system.allocations()));
    match result {
        Ok(items) => json_to_c_string(json!({ "allocations": items })),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_hub_distributions(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(system.distributions()));
    match result {
        Ok(items) => json_to_c_string(json!({ "distributions": items })),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_hub_restitutions(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(system.restitutions()));
    match result {
        Ok(items) => json_to_c_string(json!({ "restitutions": items })),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_hub_negotiations(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(system.negotiations()));
    match result {
        Ok(items) => json_to_c_string(json!({ "negotiations": items })),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_hub_dashboard_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_hub_dashboard_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_hub_governance_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_hub_governance_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_hub_voting_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_hub_voting_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_hub_allocation_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_hub_allocation_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_hub_distribution_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_hub_distribution_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_hub_collaboration_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_hub_collaboration_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_hub_restitution_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_hub_restitution_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_hub_negotiations_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_hub_negotiations_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_hub_teams_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_hub_teams_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_hub_organizations_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_hub_organizations_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_hub_collectives_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_hub_collectives_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_hub_cooperatives_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_hub_cooperatives_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_hub_federations_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_hub_federations_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_hub_autonomous_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_hub_autonomous_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_hub_open_source_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_hub_open_source_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_hub_group_economics_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_hub_group_economics_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_hub_resource_crowdfund_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_hub_resource_crowdfund_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_hub_community_showcase_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_hub_community_showcase_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_hub_contracts_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_hub_contracts_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_hub_ip_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_hub_ip_snapshot())
}
