use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::{Mutex, OnceLock};

use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::{KogiPortfolioConfig, PortfolioSystem, PortfolioSystemState, ScorePreference};

static PORTFOLIO_SYSTEM: OnceLock<Mutex<PortfolioSystem>> = OnceLock::new();

#[derive(Debug, Deserialize)]
struct PortfolioInitRequest {
    owner_id: Option<String>,
    grid_name: Option<String>,
    node_id: Option<String>,
    environment: Option<String>,
    data_dir: Option<String>,
    persistence: Option<String>,
    score_preference: Option<serde_json::Value>,
}

fn with_system<F, R>(op: F) -> Result<R, String>
where
    F: FnOnce(&mut PortfolioSystem) -> Result<R, String>,
{
    let system = PORTFOLIO_SYSTEM.get_or_init(|| {
        let cfg = KogiPortfolioConfig::default();
        let system = PortfolioSystem::new(cfg).expect("portfolio system init");
        Mutex::new(system)
    });
    let mut guard = system.lock().map_err(|_| "portfolio lock poisoned".to_string())?;
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

fn build_config(req: Option<PortfolioInitRequest>) -> KogiPortfolioConfig {
    let mut cfg = KogiPortfolioConfig::default();
    if let Some(req) = req {
        if let Some(owner_id) = req.owner_id.and_then(|s| Uuid::parse_str(&s).ok()) {
            cfg.owner_id = owner_id;
        }
        if let Some(grid_name) = req.grid_name { cfg.grid_name = grid_name; }
        if let Some(node_id) = req.node_id { cfg.node_id = node_id; }
        if let Some(environment) = req.environment { cfg.environment = environment; }
        if let Some(data_dir) = req.data_dir { cfg.data_dir = data_dir.into(); }
        if let Some(persistence) = req.persistence {
            cfg.persistence = apapo::PersistenceMode::from_str(&persistence);
        }
        if let Some(score_pref) = req.score_preference.and_then(|v| parse_score_preference(&v)) {
            cfg.score_preference = score_pref;
        }
    }
    cfg
}

fn json_to_c_string(value: serde_json::Value) -> *mut c_char {
    let payload = serde_json::to_string(&value).unwrap_or_else(|_| "{\"error\":\"serialize\"}".to_string());
    CString::new(payload).unwrap().into_raw()
}

fn c_string_to_opt(input: *const c_char) -> Option<String> {
    if input.is_null() { return None; }
    unsafe { CStr::from_ptr(input).to_str().ok().map(|s| s.to_string()) }
}

#[no_mangle]
pub extern "C" fn kogi_free_string(ptr: *mut c_char) {
    if ptr.is_null() { return; }
    unsafe { let _ = CString::from_raw(ptr); }
}

#[no_mangle]
pub extern "C" fn kogi_portfolio_init(args_json: *const c_char) -> *mut c_char {
    let request = c_string_to_opt(args_json)
        .and_then(|s| serde_json::from_str::<PortfolioInitRequest>(&s).ok());
    let cfg = build_config(request);
    let result = with_system(|system| {
        let new_system = PortfolioSystem::new(cfg).map_err(|e| e.to_string())?;
        *system = new_system;
        Ok(system.state())
    });

    match result {
        Ok(state) => json_to_c_string(serde_json::to_value(state).unwrap_or(json!({"ok": true}))),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_portfolio_health(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(system.state()));
    match result {
        Ok(state) => json_to_c_string(json!({
            "ok": true,
            "service": "kogi-portfolio",
            "version": env!("CARGO_PKG_VERSION"),
            "state": state,
        })),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_portfolio_state(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok::<PortfolioSystemState, String>(system.state()));
    match result {
        Ok(state) => json_to_c_string(serde_json::to_value(state).unwrap_or(json!({"ok": true}))),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_portfolio_items_view(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(system.items_view()));
    match result {
        Ok(view) => json_to_c_string(serde_json::to_value(view).unwrap_or(json!({"ok": true}))),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_portfolio_dashboard_snapshot(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(system.dashboard_snapshot()));
    match result {
        Ok(snapshot) => json_to_c_string(serde_json::to_value(snapshot).unwrap_or(json!({"ok": true}))),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_portfolio_analytics_snapshot(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(system.analytics_snapshot()));
    match result {
        Ok(snapshot) => json_to_c_string(serde_json::to_value(snapshot).unwrap_or(json!({"ok": true}))),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_portfolio_registry_snapshot(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(system.registry_snapshot()));
    match result {
        Ok(snapshot) => json_to_c_string(serde_json::to_value(snapshot).unwrap_or(json!({"ok": true}))),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[derive(Debug, Deserialize)]
struct RootRequest {
    root_component_id: Option<String>,
}

fn parse_root(args_json: *const c_char) -> Option<Uuid> {
    let request = c_string_to_opt(args_json)
        .and_then(|s| serde_json::from_str::<RootRequest>(&s).ok());
    request.and_then(|req| req.root_component_id.and_then(|s| Uuid::parse_str(&s).ok()))
}

#[no_mangle]
pub extern "C" fn kogi_portfolio_link_forest(args_json: *const c_char) -> *mut c_char {
    let root = parse_root(args_json);
    let result = with_system(|system| Ok(system.link_forest_view(root)));
    match result {
        Ok(view) => json_to_c_string(serde_json::to_value(view).unwrap_or(json!({"ok": true}))),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_portfolio_link_forest_rows(args_json: *const c_char) -> *mut c_char {
    let root = parse_root(args_json);
    let result = with_system(|system| Ok(system.link_forest_rows(root)));
    match result {
        Ok(rows) => json_to_c_string(serde_json::to_value(rows).unwrap_or(json!({"ok": true}))),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_portfolio_link_forest_sheet(args_json: *const c_char) -> *mut c_char {
    let root = parse_root(args_json);
    let result = with_system(|system| Ok(system.link_forest_sheet(root)));
    match result {
        Ok(sheet) => json_to_c_string(serde_json::to_value(sheet).unwrap_or(json!({"ok": true}))),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}
