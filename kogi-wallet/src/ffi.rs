use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::{Mutex, OnceLock};

use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::{WalletConfig, WalletSystem};

static WALLET_SYSTEM: OnceLock<Mutex<WalletSystem>> = OnceLock::new();

#[derive(Debug, Deserialize)]
struct WalletInitRequest {
    owner_id: Option<String>,
}

fn with_system<F, R>(op: F) -> Result<R, String>
where
    F: FnOnce(&mut WalletSystem) -> Result<R, String>,
{
    let system = WALLET_SYSTEM.get_or_init(|| {
        let cfg = WalletConfig::default();
        let system = WalletSystem::new(cfg).expect("wallet system init");
        Mutex::new(system)
    });
    let mut guard = system.lock().map_err(|_| "wallet lock poisoned".to_string())?;
    op(&mut guard)
}

fn c_string_to_opt(input: *const c_char) -> Option<String> {
    if input.is_null() { return None; }
    unsafe { CStr::from_ptr(input).to_str().ok().map(|s| s.to_string()) }
}

fn json_to_c_string(value: serde_json::Value) -> *mut c_char {
    let payload = serde_json::to_string(&value).unwrap_or_else(|_| "{\"error\":\"serialize\"}".to_string());
    CString::new(payload).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn kogi_free_string(ptr: *mut c_char) {
    if ptr.is_null() { return; }
    unsafe { let _ = CString::from_raw(ptr); }
}

#[no_mangle]
pub extern "C" fn kogi_wallet_init(args_json: *const c_char) -> *mut c_char {
    let request = c_string_to_opt(args_json)
        .and_then(|s| serde_json::from_str::<WalletInitRequest>(&s).ok());
    let mut cfg = WalletConfig::default();
    if let Some(req) = request {
        if let Some(owner) = req.owner_id.and_then(|s| Uuid::parse_str(&s).ok()) {
            cfg.owner_id = owner;
            cfg.portfolio_config.owner_id = owner;
        }
    }
    let result = with_system(|system| {
        let new_system = WalletSystem::new(cfg).map_err(|e| e.to_string())?;
        *system = new_system;
        Ok(json!({"ok": true}))
    });
    match result {
        Ok(value) => json_to_c_string(value),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_wallet_health(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(json!({
        "ok": true,
        "service": "kogi-wallet",
        "version": env!("CARGO_PKG_VERSION"),
        "owner_id": system.config.owner_id.to_string(),
    })));
    match result {
        Ok(value) => json_to_c_string(value),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_wallet_dashboard_snapshot(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(serde_json::to_value(system.dashboard_snapshot()).unwrap_or(json!({}))));
    match result {
        Ok(value) => json_to_c_string(value),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_wallet_banking_snapshot(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(serde_json::to_value(system.banking_snapshot()).unwrap_or(json!({}))));
    match result {
        Ok(value) => json_to_c_string(value),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_wallet_ledger_snapshot(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(serde_json::to_value(system.ledger_snapshot()).unwrap_or(json!({}))));
    match result {
        Ok(value) => json_to_c_string(value),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_wallet_escrow_snapshot(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(serde_json::to_value(system.escrow_snapshot()).unwrap_or(json!({}))));
    match result {
        Ok(value) => json_to_c_string(value),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_wallet_invoices_snapshot(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(serde_json::to_value(system.invoices_snapshot()).unwrap_or(json!({}))));
    match result {
        Ok(value) => json_to_c_string(value),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_wallet_investments_snapshot(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(serde_json::to_value(system.investments_snapshot()).unwrap_or(json!({}))));
    match result {
        Ok(value) => json_to_c_string(value),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_wallet_funding_snapshot(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(serde_json::to_value(system.funding_snapshot()).unwrap_or(json!({}))));
    match result {
        Ok(value) => json_to_c_string(value),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_wallet_benefits_snapshot(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(serde_json::to_value(system.benefits_snapshot()).unwrap_or(json!({}))));
    match result {
        Ok(value) => json_to_c_string(value),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_wallet_grants_snapshot(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(serde_json::to_value(system.grants_snapshot()).unwrap_or(json!({}))));
    match result {
        Ok(value) => json_to_c_string(value),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_wallet_group_economics_snapshot(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(serde_json::to_value(system.group_economics_snapshot()).unwrap_or(json!({}))));
    match result {
        Ok(value) => json_to_c_string(value),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_wallet_campaigns_snapshot(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(serde_json::to_value(system.campaigns_snapshot()).unwrap_or(json!({}))));
    match result {
        Ok(value) => json_to_c_string(value),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_wallet_debts_snapshot(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(serde_json::to_value(system.debts_snapshot()).unwrap_or(json!({}))));
    match result {
        Ok(value) => json_to_c_string(value),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_wallet_taxes_snapshot(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(serde_json::to_value(system.taxes_snapshot()).unwrap_or(json!({}))));
    match result {
        Ok(value) => json_to_c_string(value),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_wallet_wallets_overview(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(serde_json::to_value(system.wallets_overview_snapshot()).unwrap_or(json!({}))));
    match result {
        Ok(value) => json_to_c_string(value),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_wallet_wallets(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(serde_json::to_value(system.wallets()).unwrap_or(json!({}))));
    match result {
        Ok(value) => json_to_c_string(value),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}

#[no_mangle]
pub extern "C" fn kogi_wallet_accounts(_: *const c_char) -> *mut c_char {
    let result = with_system(|system| Ok(serde_json::to_value(system.accounts()).unwrap_or(json!({}))));
    match result {
        Ok(value) => json_to_c_string(value),
        Err(err) => json_to_c_string(json!({"ok": false, "error": err})),
    }
}
