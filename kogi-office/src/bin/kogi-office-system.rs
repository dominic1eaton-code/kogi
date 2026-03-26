use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{self, Read};
use std::time::{SystemTime, UNIX_EPOCH};

use kogi_office::{OfficeConfig, OfficeSystem};

#[derive(Deserialize, Default)]
struct OfficeRequest {
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
    if let Some(stripped) = action.strip_prefix("kogi_office_") {
        action = stripped.to_string();
    }
    if let Some(stripped) = action.strip_prefix("office_") {
        action = stripped.to_string();
    }

    let mut system = OfficeSystem::new(OfficeConfig::default())
        .expect("office init");

    let response = match action.as_str() {
        "health" => json!({
            "ok": true,
            "service": "kogi-office",
            "version": env!("CARGO_PKG_VERSION"),
            "state": system.state(),
            "timestamp_ms": now_ms(),
        }),
        "state" => to_json_value(system.state()),
        "refresh" => {
            let _ = system.refresh();
            to_json_value(system.state())
        }
        "work_items" => to_json_value(system.work_items()),
        "work_management" => to_json_value(system.work_management_system()),
        "overview_snapshot" | "overview" => kogi_office::sample_overview_snapshot(),
        "inbox_snapshot" | "inbox" => kogi_office::sample_inbox_snapshot(),
        "schedule_snapshot" | "schedule" => kogi_office::sample_schedule_snapshot(),
        "calendar_snapshot" | "calendar" => kogi_office::sample_calendar_snapshot(),
        "contacts_snapshot" | "contacts" => kogi_office::sample_contacts_snapshot(),
        "studio_overview_snapshot" | "studio_overview" => kogi_office::sample_studio_overview_snapshot(),
        "studio_ideas_snapshot" => kogi_office::sample_studio_detail_snapshot("Ideas"),
        "studio_concepts_snapshot" => kogi_office::sample_studio_detail_snapshot("Concepts"),
        "studio_designs_snapshot" => kogi_office::sample_studio_detail_snapshot("Designs"),
        "studio_blueprints_snapshot" => kogi_office::sample_studio_detail_snapshot("Blueprints"),
        "studio_mockups_snapshot" => kogi_office::sample_studio_detail_snapshot("Mockups"),
        "studio_prototypes_snapshot" => kogi_office::sample_studio_detail_snapshot("Prototypes"),
        "studio_testing_snapshot" => kogi_office::sample_studio_detail_snapshot("Testing"),
        "studio_notes_snapshot" => kogi_office::sample_studio_detail_snapshot("Notes"),
        "studio_docs_snapshot" => kogi_office::sample_studio_detail_snapshot("Docs"),
        "studio_content_snapshot" => kogi_office::sample_studio_detail_snapshot("Content"),
        "work_backlog_snapshot" => kogi_office::sample_work_backlog_snapshot(),
        "work_boards_snapshot" => kogi_office::sample_work_boards_snapshot(),
        "work_timeline_snapshot" => kogi_office::sample_work_timeline_snapshot(),
        "work_analytics_snapshot" => kogi_office::sample_work_analytics_snapshot(),
        "work_resources_snapshot" => kogi_office::sample_work_resources_snapshot(),
        "work_content_snapshot" => kogi_office::sample_work_content_snapshot(),
        "work_governance_snapshot" => kogi_office::sample_work_governance_snapshot(),
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

fn parse_request(raw: &str) -> OfficeRequest {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return OfficeRequest::default();
    }
    serde_json::from_str(trimmed).unwrap_or_default()
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
