use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use kogi_portfolio::ScorePreference;

use crate::snapshots::*;
use crate::system::{MarketplaceConfig, MarketplaceState, MarketplaceSystem, PublishListingRequest};

static MARKETPLACE_SYSTEM: OnceLock<Mutex<MarketplaceSystem>> = OnceLock::new();

#[derive(Debug, Deserialize)]
struct MarketplaceInitRequest {
    owner_id: Option<String>,
    grid_name: Option<String>,
    node_id: Option<String>,
    environment: Option<String>,
    data_dir: Option<String>,
    default_currency: Option<String>,
    score_preference: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct ListingRequest {
    listing_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OwnerRequest {
    owner_id: Option<String>,
}

fn with_system<F, R>(op: F) -> Result<R, String>
where
    F: FnOnce(&mut MarketplaceSystem) -> Result<R, String>,
{
    let system = MARKETPLACE_SYSTEM.get_or_init(|| {
        let cfg = MarketplaceConfig::default();
        let system = MarketplaceSystem::new(cfg).expect("marketplace system init");
        Mutex::new(system)
    });
    let mut guard = system.lock().map_err(|_| "marketplace lock poisoned".to_string())?;
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

fn build_config(req: Option<MarketplaceInitRequest>) -> MarketplaceConfig {
    let mut cfg = MarketplaceConfig::default();
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
        if let Some(default_currency) = req.default_currency {
            cfg.default_currency = default_currency;
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
pub extern "C" fn kogi_marketplace_init(args_json: *const c_char) -> *mut c_char {
    let request = c_string_to_opt(args_json)
        .and_then(|s| serde_json::from_str::<MarketplaceInitRequest>(&s).ok());
    let cfg = build_config(request);
    let result = with_system(|system| {
        let new_system = MarketplaceSystem::new(cfg).map_err(|e| e.to_string())?;
        *system = new_system;
        Ok(system.state())
    });

    match result {
        Ok(state) => json_to_c_string(serde_json::to_value(state).unwrap_or(json!({"ok": true}))),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_marketplace_health(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(system.state()));
    match result {
        Ok(state) => json_to_c_string(json!({
            "ok": true,
            "service": "kogi-marketplace",
            "version": env!("CARGO_PKG_VERSION"),
            "state": state,
        })),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_marketplace_state(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok::<MarketplaceState, String>(system.state()));
    match result {
        Ok(state) => json_to_c_string(serde_json::to_value(state).unwrap_or(json!({"ok": true}))),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_marketplace_refresh(_: *const c_char) -> *mut c_char {
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
pub extern "C" fn kogi_marketplace_listings(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(system.listings()));
    match result {
        Ok(listings) => json_to_c_string(json!({ "listings": listings })),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_marketplace_listing(args_json: *const c_char) -> *mut c_char {
    let request = c_string_to_opt(args_json)
        .and_then(|s| serde_json::from_str::<ListingRequest>(&s).ok());
    let listing_id = request
        .and_then(|r| r.listing_id)
        .and_then(|s| Uuid::parse_str(&s).ok());
    let Some(listing_id) = listing_id else {
        return json_to_c_string(json!({"ok": false, "error": "listing_id required"}));
    };

    let result = with_system(|system| Ok(system.listing_by_id(listing_id)));
    match result {
        Ok(Some(listing)) => json_to_c_string(serde_json::to_value(listing).unwrap_or(json!({"ok": true}))),
        Ok(None) => json_to_c_string(json!({"ok": false, "error": "not_found"})),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_marketplace_listings_by_owner(args_json: *const c_char) -> *mut c_char {
    let request = c_string_to_opt(args_json)
        .and_then(|s| serde_json::from_str::<OwnerRequest>(&s).ok());
    let owner_id = request
        .and_then(|r| r.owner_id)
        .and_then(|s| Uuid::parse_str(&s).ok());
    let Some(owner_id) = owner_id else {
        return json_to_c_string(json!({"ok": false, "error": "owner_id required"}));
    };

    let result = with_system(|system| Ok(system.listings_for_owner(owner_id)));
    match result {
        Ok(listings) => json_to_c_string(json!({ "listings": listings })),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_marketplace_publish_listing(args_json: *const c_char) -> *mut c_char {
    let request = c_string_to_opt(args_json)
        .and_then(|s| serde_json::from_str::<PublishListingRequest>(&s).ok());
    let Some(request) = request else {
        return json_to_c_string(json!({"ok": false, "error": "invalid payload"}));
    };

    let result = with_system(|system| system.publish_listing(request).map_err(|e| e.to_string()));
    match result {
        Ok(listing) => json_to_c_string(serde_json::to_value(listing).unwrap_or(json!({"ok": true}))),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_marketplace_dashboard_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_dashboard_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_marketplace_market_overview_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_market_overview_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_marketplace_market_browse_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_market_browse_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_marketplace_market_labor_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_market_labor_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_marketplace_market_grants_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_market_grants_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_marketplace_market_crm_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_market_crm_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_marketplace_listings_catalog_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_listings_catalog_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_marketplace_listings_detail_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_listings_detail_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_marketplace_listings_mine_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_listings_mine_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_marketplace_campaigns_overview_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_campaigns_overview_snapshot())
}

#[no_mangle]
pub extern "C" fn kogi_marketplace_campaigns_discover_snapshot(_: *const c_char) -> *mut c_char {
    json_to_c_string(sample_campaigns_discover_snapshot())
}
