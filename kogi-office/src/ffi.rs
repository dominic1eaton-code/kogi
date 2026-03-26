use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use kogi_portfolio::ScorePreference;

use crate::snapshots::*;
use crate::system::{OfficeConfig, OfficeState, OfficeSystem};

static OFFICE_SYSTEM: OnceLock<Mutex<OfficeSystem>> = OnceLock::new();

#[derive(Debug, Deserialize)]
struct OfficeInitRequest {
    owner_id: Option<String>,
    grid_name: Option<String>,
    node_id: Option<String>,
    environment: Option<String>,
    data_dir: Option<String>,
    score_preference: Option<serde_json::Value>,
}

fn with_system<F, R>(op: F) -> Result<R, String>
where
    F: FnOnce(&mut OfficeSystem) -> Result<R, String>,
{
    let system = OFFICE_SYSTEM.get_or_init(|| {
        let cfg = OfficeConfig::default();
        let system = OfficeSystem::new(cfg).expect("office system init");
        Mutex::new(system)
    });
    let mut guard = system.lock().map_err(|_| "office lock poisoned".to_string())?;
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

fn build_config(req: Option<OfficeInitRequest>) -> OfficeConfig {
    let mut cfg = OfficeConfig::default();
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
pub extern "C" fn kogi_office_init(args_json: *const c_char) -> *mut c_char {
    let request = c_string_to_opt(args_json)
        .and_then(|s| serde_json::from_str::<OfficeInitRequest>(&s).ok());
    let cfg = build_config(request);
    let result = with_system(|system| {
        let new_system = OfficeSystem::new(cfg).map_err(|e| e.to_string())?;
        *system = new_system;
        Ok(system.state())
    });

    match result {
        Ok(state) => json_to_c_string(serde_json::to_value(state).unwrap_or(json!({"ok": true}))),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_office_health(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(system.state()));
    match result {
        Ok(state) => json_to_c_string(json!({
            "ok": true,
            "service": "kogi-office",
            "version": env!("CARGO_PKG_VERSION"),
            "state": state,
        })),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_office_state(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok::<OfficeState, String>(system.state()));
    match result {
        Ok(state) => json_to_c_string(serde_json::to_value(state).unwrap_or(json!({"ok": true}))),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_office_refresh(_: *const c_char) -> *mut c_char {
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
pub extern "C" fn kogi_office_work_items(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(system.work_items()));
    match result {
        Ok(items) => json_to_c_string(json!({ "items": items })),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_office_work_management(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(system.work_management_system()));
    match result {
        Ok(wms) => json_to_c_string(serde_json::to_value(wms).unwrap_or(json!({"ok": true}))),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_office_overview_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_overview_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_office_inbox_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_inbox_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_office_schedule_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_schedule_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_office_calendar_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_calendar_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_office_contacts_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_contacts_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_office_studio_overview_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_studio_overview_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_office_studio_ideas_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_studio_detail_snapshot("Ideas"))
}

#[no_mangle]
pub extern "C" fn kogi_office_studio_concepts_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_studio_detail_snapshot("Concepts"))
}

#[no_mangle]
pub extern "C" fn kogi_office_studio_designs_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_studio_detail_snapshot("Designs"))
}

#[no_mangle]
pub extern "C" fn kogi_office_studio_blueprints_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_studio_detail_snapshot("Blueprints"))
}

#[no_mangle]
pub extern "C" fn kogi_office_studio_mockups_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_studio_detail_snapshot("Mockups"))
}

#[no_mangle]
pub extern "C" fn kogi_office_studio_prototypes_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_studio_detail_snapshot("Prototypes"))
}

#[no_mangle]
pub extern "C" fn kogi_office_studio_testing_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_studio_detail_snapshot("Testing"))
}

#[no_mangle]
pub extern "C" fn kogi_office_studio_notes_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_studio_detail_snapshot("Notes"))
}

#[no_mangle]
pub extern "C" fn kogi_office_studio_docs_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_studio_detail_snapshot("Docs"))
}

#[no_mangle]
pub extern "C" fn kogi_office_studio_content_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_studio_detail_snapshot("Content"))
}

#[no_mangle]
pub extern "C" fn kogi_office_work_backlog_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_work_backlog_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_office_work_boards_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_work_boards_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_office_work_timeline_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_work_timeline_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_office_work_analytics_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_work_analytics_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_office_work_resources_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_work_resources_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_office_work_content_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_work_content_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_office_work_governance_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_work_governance_snapshot())
}
