use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{self, Read};
use std::time::{SystemTime, UNIX_EPOCH};

use kogi_hub::{HubConfig, HubSystem};

#[derive(Deserialize, Default)]
struct HubRequest {
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
    if let Some(stripped) = action.strip_prefix("kogi_hub_") {
        action = stripped.to_string();
    }
    if let Some(stripped) = action.strip_prefix("hub_") {
        action = stripped.to_string();
    }

    let mut system = HubSystem::new(HubConfig::default())
        .expect("hub init");

    let response = match action.as_str() {
        "health" => json!({
            "ok": true,
            "service": "kogi-hub",
            "version": env!("CARGO_PKG_VERSION"),
            "state": system.state(),
            "timestamp_ms": now_ms(),
        }),
        "state" => to_json_value(system.state()),
        "refresh" => {
            let _ = system.refresh();
            to_json_value(system.state())
        }
        "entities" => to_json_value(system.entities()),
        "members" => to_json_value(system.members()),
        "governance" => to_json_value(system.governance()),
        "voting" => to_json_value(system.voting()),
        "contracts" => to_json_value(system.contracts()),
        "rights" => to_json_value(system.rights()),
        "ip_assets" | "ip" => to_json_value(system.ip_assets()),
        "allocations" => to_json_value(system.allocations()),
        "distributions" => to_json_value(system.distributions()),
        "restitutions" => to_json_value(system.restitutions()),
        "negotiations" => to_json_value(system.negotiations()),
        "dashboard_snapshot" | "dashboard" => kogi_hub::sample_hub_dashboard_snapshot(),
        "governance_snapshot" => kogi_hub::sample_hub_governance_snapshot(),
        "voting_snapshot" => kogi_hub::sample_hub_voting_snapshot(),
        "allocation_snapshot" => kogi_hub::sample_hub_allocation_snapshot(),
        "distribution_snapshot" => kogi_hub::sample_hub_distribution_snapshot(),
        "collaboration_snapshot" => kogi_hub::sample_hub_collaboration_snapshot(),
        "restitution_snapshot" => kogi_hub::sample_hub_restitution_snapshot(),
        "negotiations_snapshot" => kogi_hub::sample_hub_negotiations_snapshot(),
        "teams_snapshot" => kogi_hub::sample_hub_teams_snapshot(),
        "organizations_snapshot" => kogi_hub::sample_hub_organizations_snapshot(),
        "collectives_snapshot" => kogi_hub::sample_hub_collectives_snapshot(),
        "cooperatives_snapshot" => kogi_hub::sample_hub_cooperatives_snapshot(),
        "federations_snapshot" => kogi_hub::sample_hub_federations_snapshot(),
        "autonomous_snapshot" => kogi_hub::sample_hub_autonomous_snapshot(),
        "open_source_snapshot" => kogi_hub::sample_hub_open_source_snapshot(),
        "group_economics_snapshot" => kogi_hub::sample_hub_group_economics_snapshot(),
        "resource_crowdfund_snapshot" => kogi_hub::sample_hub_resource_crowdfund_snapshot(),
        "community_showcase_snapshot" => kogi_hub::sample_hub_community_showcase_snapshot(),
        "contracts_snapshot" => kogi_hub::sample_hub_contracts_snapshot(),
        "ip_snapshot" => kogi_hub::sample_hub_ip_snapshot(),
        _ => json!({
            "error": "unknown_action",
            "action": action,
            "timestamp_ms": now_ms(),
        }),
    };

    println!("{}", response);
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

fn parse_request(raw: &str) -> HubRequest {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return HubRequest::default();
    }
    serde_json::from_str(trimmed).unwrap_or_default()
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
