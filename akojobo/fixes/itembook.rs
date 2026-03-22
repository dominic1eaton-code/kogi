//! ItemBook — per-item rich dossier (living operational context for an Item).

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::types::{ComponentId, UserId};

// ── Dashboard ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardWidget {
    pub widget_id: Uuid,
    pub widget_type: String,    // e.g. "health_score", "gantt", "kpi_bar"
    pub config: JsonValue,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Dashboard {
    pub widgets: Vec<DashboardWidget>,
    pub layout: HashMap<String, JsonValue>,     // widget_id → layout position
}

// ── Charter ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Charter {
    pub executive_summary: String,
    pub objectives: Vec<String>,
    pub scope: String,
    pub stakeholders: Vec<UserId>,
    pub success_criteria: Vec<String>,
    pub constraints: Vec<String>,
    pub assumptions: Vec<String>,
    pub risk_summary: Option<String>,
    pub approval_trail: Vec<String>,    // actor IDs who approved
}

// ── Workspace (ItemBook sub-component) ────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ItemWorkspace {
    pub active_files: Vec<String>,
    pub content_blocks: Vec<JsonValue>,
    pub connected_items: Vec<ComponentId>,
    pub plugin_refs: Vec<String>,
}

// ── Catalogue ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CatalogueEntry {
    pub entry_id: Uuid,
    pub label: String,
    pub tags: Vec<String>,
    pub component_ref: Option<ComponentId>,
    pub url: Option<String>,
    pub metadata: JsonValue,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Catalogue {
    pub entries: Vec<CatalogueEntry>,
}

impl Catalogue {
    pub fn search(&self, query: &str) -> Vec<&CatalogueEntry> {
        let q = query.to_lowercase();
        self.entries.iter()
            .filter(|e| e.label.to_lowercase().contains(&q) || e.tags.iter().any(|t| t.to_lowercase().contains(&q)))
            .collect()
    }
}

// ── Metrics Store ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricEntry {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub recorded_at: chrono::DateTime<chrono::Utc>,
}

// ── Template Store ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub template_id: Uuid,
    pub name: String,
    pub content: JsonValue,
}

// ── ItemBookData ──────────────────────────────────────────────────────────────

/// The full operational context of an Item — all sub-components optional.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ItemBookData {
    pub dashboard:      Option<Dashboard>,
    pub charter:        Option<Charter>,
    pub workspace:      Option<ItemWorkspace>,
    pub catalogue:      Option<Catalogue>,
    pub metrics:        Vec<MetricEntry>,
    pub templates:      Vec<Template>,
    pub logs:           Vec<JsonValue>,
    pub version_notes:  Vec<String>,
}

impl ItemBookData {
    pub fn new() -> Self { Self::default() }

    /// Called by `Component::bump_version()` to record the version change.
    pub fn on_version_bump(&mut self, new_version: &str) {
        self.version_notes.push(format!("Bumped to {new_version} at {}", chrono::Utc::now()));
    }

    pub fn add_metric(&mut self, name: impl Into<String>, value: f64, unit: impl Into<String>) {
        self.metrics.push(MetricEntry {
            name: name.into(),
            value,
            unit: unit.into(),
            recorded_at: chrono::Utc::now(),
        });
    }
}

// ── Book Payload (for Container::Book) ───────────────────────────────────────

/// Payload for Book containers (Notebook, ContactBook, PlayBook, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BookPayload {
    Notebook { pages: Vec<JsonValue> },
    ContactBook { contacts: Vec<JsonValue>, groups: Vec<String> },
    PlayBook { triggers: Vec<JsonValue>, plays: Vec<JsonValue> },
    ScheduleBook { schedules: Vec<JsonValue>, time_blocks: Vec<JsonValue> },
    PlanBook { goals: Vec<String>, strategies: Vec<String>, initiatives: Vec<JsonValue> },
    GuideBook { sections: Vec<JsonValue>, version: String },
    ItemBook(ItemBookData),
}
