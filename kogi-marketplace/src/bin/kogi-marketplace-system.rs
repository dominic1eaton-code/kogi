use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{self, Read};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use kogi_marketplace::{MarketplaceConfig, MarketplaceSystem, PublishListingRequest};

#[derive(Deserialize, Default)]
struct MarketplaceRequest {
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
    if let Some(stripped) = action.strip_prefix("kogi_marketplace_") {
        action = stripped.to_string();
    }
    if let Some(stripped) = action.strip_suffix("_snapshot") {
        action = stripped.to_string();
    }

    let mut system = MarketplaceSystem::new(MarketplaceConfig::default())
        .expect("marketplace init");

    let response = match action.as_str() {
        "health" => json!({
            "ok": true,
            "service": "kogi-marketplace",
            "version": env!("CARGO_PKG_VERSION"),
            "state": system.state(),
            "timestamp_ms": now_ms(),
        }),
        "state" => to_json_value(system.state()),
        "refresh" => {
            let _ = system.refresh();
            to_json_value(system.state())
        }
        "listings" => to_json_value(system.listings()),
        "listing" => listing_by_id(&system, request.payload.as_ref()),
        "listings_by_owner" => listings_by_owner(&system, request.payload.as_ref()),
        "publish_listing" => publish_listing(&mut system, request.payload.as_ref()),
        "dashboard" => kogi_marketplace::sample_dashboard_snapshot(),
        "market_overview" => kogi_marketplace::sample_market_overview_snapshot(),
        "market_browse" => kogi_marketplace::sample_market_browse_snapshot(),
        "market_labor" => kogi_marketplace::sample_market_labor_snapshot(),
        "market_grants" => kogi_marketplace::sample_market_grants_snapshot(),
        "market_crm" => kogi_marketplace::sample_market_crm_snapshot(),
        "listings_catalog" => kogi_marketplace::sample_listings_catalog_snapshot(),
        "listings_detail" => kogi_marketplace::sample_listings_detail_snapshot(),
        "listings_mine" => kogi_marketplace::sample_listings_mine_snapshot(),
        "campaigns_overview" => kogi_marketplace::sample_campaigns_overview_snapshot(),
        "campaigns_discover" => kogi_marketplace::sample_campaigns_discover_snapshot(),
        _ => json!({
            "error": "unknown_action",
            "action": action,
            "timestamp_ms": now_ms(),
        }),
    };

    println!("{}", response);
}

fn listing_by_id(system: &MarketplaceSystem, payload: Option<&Value>) -> Value {
    let Some(id) = payload.and_then(|v| v.get("listing_id")).and_then(|v| v.as_str()) else {
        return json!({ "error": "listing_id required" });
    };
    let Ok(uuid) = Uuid::parse_str(id) else {
        return json!({ "error": "invalid listing_id" });
    };
    system
        .listing_by_id(uuid)
        .map(to_json_value)
        .unwrap_or_else(|| json!({ "error": "not_found" }))
}

fn listings_by_owner(system: &MarketplaceSystem, payload: Option<&Value>) -> Value {
    let Some(id) = payload.and_then(|v| v.get("owner_id")).and_then(|v| v.as_str()) else {
        return json!({ "error": "owner_id required" });
    };
    let Ok(uuid) = Uuid::parse_str(id) else {
        return json!({ "error": "invalid owner_id" });
    };
    to_json_value(system.listings_for_owner(uuid))
}

fn publish_listing(system: &mut MarketplaceSystem, payload: Option<&Value>) -> Value {
    let Some(payload) = payload else {
        return json!({ "error": "payload required" });
    };
    let Ok(req) = serde_json::from_value::<PublishListingRequest>(payload.clone()) else {
        return json!({ "error": "invalid payload" });
    };
    match system.publish_listing(req) {
        Ok(listing) => to_json_value(listing),
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

fn parse_request(raw: &str) -> MarketplaceRequest {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return MarketplaceRequest::default();
    }
    serde_json::from_str(trimmed).unwrap_or_default()
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
