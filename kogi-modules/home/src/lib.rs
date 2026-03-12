// ============================================================================
//  lib.rs — kogi-home crate root
//  Independent Worker Operating System
//
//  This crate implements the Kogi Home module — the primary entry point for
//  every user session on the Kogi platform.  It owns three sub-systems:
//
//    DashboardSystem  → real-time overview, notifications, alerts, KPIs, AI
//    ProfileSystem    → identity, skills, personas, contact, benefits, actions
//    WorkspaceSystem  → files, notebooks, boards, schedules, contact books
//
//  The `HomeModule` struct (re-exported from model) is the single stateful
//  object the host process instantiates and routes calls through.
// ============================================================================

// ─────────────────────────────────────────────────────────────────────────────
// Sub-modules (source files use PascalCase per Kogi naming convention)
// ─────────────────────────────────────────────────────────────────────────────

#[path = "DashboardSystem.rs"]
pub mod dashboard_system;

#[path = "ProfileSystem.rs"]
pub mod profile_system;

#[path = "WorkspaceSystem.rs"]
pub mod workspace_system;

pub mod model;

// ─────────────────────────────────────────────────────────────────────────────
// Flat public re-exports — callers can `use kogi_home::*`
// ─────────────────────────────────────────────────────────────────────────────

// -- Dashboard ----------------------------------------------------------------
pub use model::{
    ActivityFeedItem,
    AiSuggestion,
    AlertSeverity,
    DashboardAlert,
    DashboardCategory,
    DashboardKpi,
    DashboardMutationResponse,
    DashboardNotification,
    DashboardOverview,
    DashboardOverviewCounts,
    DashboardSnapshot,
    NewActivityFeedItem,
    NewAiSuggestion,
    NewDashboardAlert,
    NewDashboardKpi,
    NewDashboardNotification,
};

// -- Profile ------------------------------------------------------------------
pub use model::{
    ContactEntry,
    NewProfileContact,
    NewProfilePersona,
    NewProfileSkill,
    NewUserAction,
    PortableBenefit,
    ProfileAccount,
    ProfileMutationResponse,
    ProfilePersona,
    ProfileSkill,
    ProfileSnapshot,
    UserActionOption,
    UserActionRecord,
    UserPresence,
};

// -- Workspace ----------------------------------------------------------------
pub use model::{
    BoardCard,
    NewBoardCard,
    NewWorkspaceContact,
    NewWorkspaceContent,
    NewWorkspaceNote,
    NewWorkspaceNotebook,
    NewWorkspaceSchedule,
    WorkspaceBoard,
    WorkspaceMutationResponse,
    WorkspaceContact,
    WorkspaceContactBook,
    WorkspaceContentItem,
    WorkspaceContentSnapshot,
    WorkspaceNote,
    WorkspaceNotebook,
    WorkspaceScheduleItem,
    WorkspaceSnapshot,
};

// -- Module-level -------------------------------------------------------------
pub use model::{
    ActionResult,
    HomeModule,
    HomeOverview,
    HomeViewInfo,
};

// ─────────────────────────────────────────────────────────────────────────────
// Utility helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Serialize any `serde::Serialize` value to a JSON string.
/// Returns a safe error sentinel on serialization failure — never panics.
pub fn to_json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value)
        .unwrap_or_else(|e| format!("{{\"error\":\"serialization_failed\",\"detail\":\"{e}\"}}"))
}

/// Serialize any `serde::Serialize` value to a pretty-printed JSON string.
pub fn to_json_pretty<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value)
        .unwrap_or_else(|e| format!("{{\"error\":\"serialization_failed\",\"detail\":\"{e}\"}}"))
}

/// Attempt to deserialize a JSON string into `T`.
pub fn from_json<T: serde::de::DeserializeOwned>(json: &str) -> Result<T, serde_json::Error> {
    serde_json::from_str(json)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── HomeModule bootstrap ─────────────────────────────────────────────────

    #[test]
    fn test_mvp_initializes() {
        let home = HomeModule::mvp();
        let overview = home.overview();
        assert_eq!(overview.module, "kogi.home");
        assert_eq!(overview.systems.len(), 3);
        assert_eq!(overview.views.len(), 3);
    }

    // ── Dashboard ────────────────────────────────────────────────────────────

    #[test]
    fn test_dashboard_snapshot_has_kpis_and_ai() {
        let home = HomeModule::mvp();
        let snap = home.dashboard_snapshot();
        assert!(!snap.kpis.is_empty(), "KPIs should be non-empty");
        assert!(!snap.ai_suggestions.is_empty(), "AI suggestions should be non-empty");
        assert!(!snap.activity_feed.is_empty(), "Activity feed should be non-empty");
    }

    #[test]
    fn test_push_notification() {
        let mut home = HomeModule::mvp();
        let before = home.dashboard_snapshot().overview.counts.unread_notifications;
        let res = home.push_dashboard_notification(NewDashboardNotification {
            category: "work".to_string(),
            message: "New task assigned".to_string(),
        });
        assert!(res.result.ok);
        assert!(res.result.entity_id.is_some());
        let after = res.dashboard.overview.counts.unread_notifications;
        assert_eq!(after, before + 1);
    }

    #[test]
    fn test_acknowledge_notification() {
        let mut home = HomeModule::mvp();
        let snap = home.dashboard_snapshot();
        let first_id = snap.notifications[0].id.clone();
        let res = home.acknowledge_dashboard_notification(&first_id);
        assert!(res.result.ok);
        let acknowledged = res.dashboard.notifications.iter()
            .find(|n| n.id == first_id)
            .map(|n| n.acknowledged)
            .unwrap_or(false);
        assert!(acknowledged);
    }

    #[test]
    fn test_push_alert() {
        let mut home = HomeModule::mvp();
        let res = home.push_dashboard_alert(NewDashboardAlert {
            severity: "critical".to_string(),
            category: "system".to_string(),
            message: "Database connection pool exhausted".to_string(),
        });
        assert!(res.result.ok);
        let alert = res.dashboard.alerts.iter()
            .find(|a| a.id == res.result.entity_id.clone().unwrap());
        assert!(alert.is_some());
        assert_eq!(alert.unwrap().severity, "critical");
    }

    #[test]
    fn test_kpi_add_and_update() {
        let mut home = HomeModule::mvp();
        let res = home.add_dashboard_kpi(NewDashboardKpi {
            label: "Churn Rate".to_string(),
            value: "2.1%".to_string(),
            delta: Some("-0.3% vs last month".to_string()),
            trend: "positive".to_string(),
            category: "finance".to_string(),
        });
        let kpi_id = res.result.entity_id.unwrap();
        assert!(res.result.ok);

        let update_res = home.update_dashboard_kpi(&kpi_id, "1.8%".to_string(), None, "positive".to_string());
        assert!(update_res.result.ok);
        let updated = update_res.dashboard.kpis.iter().find(|k| k.id == kpi_id).unwrap();
        assert_eq!(updated.value, "1.8%");
    }

    #[test]
    fn test_ai_suggestion_lifecycle() {
        let mut home = HomeModule::mvp();
        let res = home.push_ai_suggestion(NewAiSuggestion {
            kind: "insight".to_string(),
            message: "Your best earning day is Friday.".to_string(),
            action_label: None,
            action_link: None,
        });
        let sid = res.result.entity_id.unwrap();
        let dismiss = home.dismiss_ai_suggestion(&sid);
        assert!(dismiss.result.ok);
        let dismissed = dismiss.dashboard.ai_suggestions.iter()
            .find(|s| s.id == sid)
            .map(|s| s.dismissed)
            .unwrap_or(false);
        assert!(dismissed);
    }

    // ── Profile ──────────────────────────────────────────────────────────────

    #[test]
    fn test_add_skill_deduplication() {
        let mut home = HomeModule::mvp();
        let first = home.add_profile_skill(NewProfileSkill {
            skill: "rust".to_string(),
            level: None,
            category: None,
        });
        assert!(!first.result.ok, "duplicate skill should not be added");

        let second = home.add_profile_skill(NewProfileSkill {
            skill: "typescript".to_string(),
            level: Some("advanced".to_string()),
            category: Some("engineering".to_string()),
        });
        assert!(second.result.ok);
        assert!(second.profile.skills.iter().any(|s| s.name == "typescript"));
    }

    #[test]
    fn test_presence_update() {
        let mut home = HomeModule::mvp();
        let res = home.set_user_presence(
            "busy".to_string(),
            Some("Deep work until 3pm".to_string()),
        );
        assert!(res.result.ok);
        assert_eq!(res.profile.presence.status, "busy");
        assert_eq!(
            res.profile.presence.message.as_deref(),
            Some("Deep work until 3pm")
        );
    }

    #[test]
    fn test_add_portable_benefit() {
        let mut home = HomeModule::mvp();
        let res = home.add_portable_benefit(
            "insurance".to_string(),
            "Next Insurance".to_string(),
            "Professional liability — $1M coverage".to_string(),
        );
        assert!(res.result.ok);
        assert!(res.profile.portable_benefits.iter().any(|b| b.kind == "insurance"));
    }

    #[test]
    fn test_persona_toggle() {
        let mut home = HomeModule::mvp();
        // "advisor" starts as inactive in mvp seed
        let res = home.toggle_profile_persona("advisor");
        assert!(res.result.ok);
        let persona = res.profile.personas.iter().find(|p| p.name == "advisor").unwrap();
        assert!(persona.active);
    }

    // ── Workspace ────────────────────────────────────────────────────────────

    #[test]
    fn test_add_workspace_content() {
        let mut home = HomeModule::mvp();
        let res = home.add_workspace_content(NewWorkspaceContent {
            kind: "file".to_string(),
            name: "budget-2026.xlsx".to_string(),
            location: "/drive/finance".to_string(),
            tags: vec!["finance".to_string()],
        });
        assert!(res.result.ok);
        assert!(res.workspace.content.files.iter().any(|f| f.name == "budget-2026.xlsx"));
    }

    #[test]
    fn test_add_workspace_note() {
        let mut home = HomeModule::mvp();
        // nb-001 exists in mvp seed
        let res = home.add_workspace_note(NewWorkspaceNote {
            notebook_id: "nb-001".to_string(),
            title: "Roadmap Delta".to_string(),
            body: "Shifted wallet module to Phase 2.".to_string(),
            tags: vec!["roadmap".to_string()],
        });
        assert!(res.result.ok);
        let nb = res.workspace.notebooks.iter().find(|n| n.id == "nb-001").unwrap();
        assert!(nb.notes.iter().any(|n| n.title == "Roadmap Delta"));
    }

    #[test]
    fn test_board_card_move() {
        let mut home = HomeModule::mvp();
        // card-001 is in board-001, column "in-progress"
        let res = home.move_board_card("board-001", "card-001", "review".to_string());
        assert!(res.result.ok);
        let board = res.workspace.boards.iter().find(|b| b.id == "board-001").unwrap();
        let card = board.cards.iter().find(|c| c.id == "card-001").unwrap();
        assert_eq!(card.column, "review");
    }

    #[test]
    fn test_serialization_roundtrip() {
        let home = HomeModule::mvp();
        let snap = home.dashboard_snapshot();
        let json = to_json(&snap);
        assert!(!json.contains("error"));
        let parsed: DashboardSnapshot = from_json(&json).expect("should parse");
        assert_eq!(parsed.view, snap.view);
        assert_eq!(parsed.kpis.len(), snap.kpis.len());
    }
}
