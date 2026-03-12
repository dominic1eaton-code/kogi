// ============================================================================
//  model.rs — Kogi Home · Public Type Re-exports & HomeModule
//  Independent Worker Operating System
//
//  This is the public API surface of the kogi-home crate.  It:
//    1. Re-exports the canonical types from each sub-system
//    2. Defines cross-cutting response wrappers (MutationResponse)
//    3. Implements HomeModule — the single stateful object the host process
//       instantiates; all mutations flow through it
// ============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────────────
// DashboardSystem re-exports
// ─────────────────────────────────────────────────────────────────────────────

pub use crate::dashboard_system::{
    ActivityFeedItem,
    AiSuggestion,
    AlertSeverity,
    DashboardAlert,
    DashboardCategory,
    DashboardKpi,
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
use crate::dashboard_system::DashboardSystem;

// ─────────────────────────────────────────────────────────────────────────────
// ProfileSystem re-exports
// ─────────────────────────────────────────────────────────────────────────────

pub use crate::profile_system::{
    ContactEntry,
    NewProfileContact,
    NewProfilePersona,
    NewProfileSkill,
    NewUserAction,
    PortableBenefit,
    ProfileAccount,
    ProfilePersona,
    ProfileSkill,
    ProfileSnapshot,
    UserActionOption,
    UserActionRecord,
    UserPresence,
};
use crate::profile_system::ProfileSystem;

// ─────────────────────────────────────────────────────────────────────────────
// WorkspaceSystem re-exports
// ─────────────────────────────────────────────────────────────────────────────

pub use crate::workspace_system::{
    BoardCard,
    NewBoardCard,
    NewWorkspaceContact,
    NewWorkspaceContent,
    NewWorkspaceNote,
    NewWorkspaceNotebook,
    NewWorkspaceSchedule,
    WorkspaceBoard,
    WorkspaceContact,
    WorkspaceContactBook,
    WorkspaceContentItem,
    WorkspaceContentSnapshot,
    WorkspaceNote,
    WorkspaceNotebook,
    WorkspaceScheduleItem,
    WorkspaceSnapshot,
};
use crate::workspace_system::WorkspaceSystem;

// ─────────────────────────────────────────────────────────────────────────────
// Module-level types
// ─────────────────────────────────────────────────────────────────────────────

/// Metadata about a single view within the Home module.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HomeViewInfo {
    pub id: String,
    pub title: String,
    pub status: String,
    pub description: String,
}

/// Top-level overview of the Home module — returned on initial load or health
/// check so that clients know what systems and views are available.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HomeOverview {
    pub module: String,
    pub application: String,
    pub service: String,
    pub version: String,
    pub status: String,
    pub systems: Vec<String>,
    pub views: Vec<HomeViewInfo>,
    pub integrations: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Action result wrapper (shared across all systems)
// ─────────────────────────────────────────────────────────────────────────────

/// Standardised envelope that every mutation returns — always paired with the
/// updated snapshot of the affected system so callers never need a second read.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionResult {
    pub ok: bool,
    /// Name of the system that processed the action
    pub system: String,
    pub message: String,
    /// ID of the created or mutated entity, when applicable
    pub entity_id: Option<String>,
}

impl ActionResult {
    pub fn ok(system: &str, message: &str, entity_id: Option<String>) -> Self {
        Self {
            ok: true,
            system: system.to_string(),
            message: message.to_string(),
            entity_id,
        }
    }

    pub fn err(system: &str, message: &str) -> Self {
        Self {
            ok: false,
            system: system.to_string(),
            message: message.to_string(),
            entity_id: None,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Mutation responses — result + updated snapshot
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DashboardMutationResponse {
    pub result: ActionResult,
    pub dashboard: DashboardSnapshot,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProfileMutationResponse {
    pub result: ActionResult,
    pub profile: ProfileSnapshot,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceMutationResponse {
    pub result: ActionResult,
    pub workspace: WorkspaceSnapshot,
}

// ─────────────────────────────────────────────────────────────────────────────
// HomeModule — the stateful root object
// ─────────────────────────────────────────────────────────────────────────────

/// The Home module owns all three sub-systems and is the single entry-point
/// for the host (gRPC handler, HTTP handler, WASM host, etc.) to call into.
///
/// Usage:
/// ```rust
/// let mut home = HomeModule::mvp();
/// let snap = home.dashboard_snapshot();
/// let res  = home.push_dashboard_notification(NewDashboardNotification { ... });
/// ```
#[derive(Clone, Debug)]
pub struct HomeModule {
    dashboard: DashboardSystem,
    profile: ProfileSystem,
    workspace: WorkspaceSystem,
}

impl HomeModule {
    // ── Constructors ──────────────────────────────────────────────────────────

    /// MVP seed — all three systems initialised with realistic demo data.
    pub fn mvp() -> Self {
        Self {
            dashboard: DashboardSystem::mvp(),
            profile: ProfileSystem::mvp(),
            workspace: WorkspaceSystem::mvp(),
        }
    }

    // ── Overview ──────────────────────────────────────────────────────────────

    pub fn overview(&self) -> HomeOverview {
        HomeOverview {
            module: "kogi.home".to_string(),
            application: "Kogi Home".to_string(),
            service: "kogi-network/services/home".to_string(),
            version: "0.1.0".to_string(),
            status: "active".to_string(),
            systems: vec![
                "DashboardSystem".to_string(),
                "ProfileSystem".to_string(),
                "WorkspaceSystem".to_string(),
            ],
            views: vec![
                HomeViewInfo {
                    id: "dashboard".to_string(),
                    title: "Home Dashboard".to_string(),
                    status: "active".to_string(),
                    description: "Real-time operational overview, KPIs, notifications, and AI insights".to_string(),
                },
                HomeViewInfo {
                    id: "profile".to_string(),
                    title: "Home Profile".to_string(),
                    status: "active".to_string(),
                    description: "Identity, skills, personas, contact, account, and portable benefits".to_string(),
                },
                HomeViewInfo {
                    id: "workspace".to_string(),
                    title: "Home Workspace".to_string(),
                    status: "active".to_string(),
                    description: "Files, documents, notebooks, boards, schedules, and contact books".to_string(),
                },
            ],
            integrations: vec![
                "kogi.wallet".to_string(),
                "kogi.portfolio".to_string(),
                "kogi.calendar".to_string(),
                "kogi.notifications".to_string(),
                "kogi.marketplace".to_string(),
                "kogi.community".to_string(),
                "kogi.engine.ai".to_string(),
            ],
            generated_at: Utc::now(),
        }
    }

    // ── Snapshot reads ────────────────────────────────────────────────────────

    pub fn dashboard_snapshot(&self) -> DashboardSnapshot {
        self.dashboard.snapshot()
    }

    pub fn profile_snapshot(&self) -> ProfileSnapshot {
        self.profile.snapshot()
    }

    pub fn workspace_snapshot(&self) -> WorkspaceSnapshot {
        self.workspace.snapshot()
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Dashboard mutations
    // ─────────────────────────────────────────────────────────────────────────

    pub fn push_dashboard_notification(
        &mut self,
        request: NewDashboardNotification,
    ) -> DashboardMutationResponse {
        let notification = self.dashboard.push_notification(request);
        DashboardMutationResponse {
            result: ActionResult::ok(
                "DashboardSystem",
                "notification created",
                Some(notification.id.clone()),
            ),
            dashboard: self.dashboard_snapshot(),
        }
    }

    pub fn push_dashboard_alert(
        &mut self,
        request: NewDashboardAlert,
    ) -> DashboardMutationResponse {
        let alert = self.dashboard.push_alert(request);
        DashboardMutationResponse {
            result: ActionResult::ok(
                "DashboardSystem",
                "alert created",
                Some(alert.id.clone()),
            ),
            dashboard: self.dashboard_snapshot(),
        }
    }

    pub fn acknowledge_dashboard_notification(
        &mut self,
        notification_id: &str,
    ) -> DashboardMutationResponse {
        let ok = self.dashboard.acknowledge_notification(notification_id);
        DashboardMutationResponse {
            result: ActionResult {
                ok,
                system: "DashboardSystem".to_string(),
                message: if ok { "notification acknowledged" } else { "notification not found" }.to_string(),
                entity_id: if ok { Some(notification_id.to_string()) } else { None },
            },
            dashboard: self.dashboard_snapshot(),
        }
    }

    pub fn acknowledge_dashboard_alert(
        &mut self,
        alert_id: &str,
    ) -> DashboardMutationResponse {
        let ok = self.dashboard.acknowledge_alert(alert_id);
        DashboardMutationResponse {
            result: ActionResult {
                ok,
                system: "DashboardSystem".to_string(),
                message: if ok { "alert acknowledged" } else { "alert not found" }.to_string(),
                entity_id: if ok { Some(alert_id.to_string()) } else { None },
            },
            dashboard: self.dashboard_snapshot(),
        }
    }

    pub fn push_dashboard_activity(
        &mut self,
        request: NewActivityFeedItem,
    ) -> DashboardMutationResponse {
        let item = self.dashboard.push_activity(request);
        DashboardMutationResponse {
            result: ActionResult::ok(
                "DashboardSystem",
                "activity item added",
                Some(item.id.clone()),
            ),
            dashboard: self.dashboard_snapshot(),
        }
    }

    pub fn add_dashboard_kpi(
        &mut self,
        request: NewDashboardKpi,
    ) -> DashboardMutationResponse {
        let kpi = self.dashboard.add_kpi(request);
        DashboardMutationResponse {
            result: ActionResult::ok(
                "DashboardSystem",
                "kpi added",
                Some(kpi.id.clone()),
            ),
            dashboard: self.dashboard_snapshot(),
        }
    }

    pub fn update_dashboard_kpi(
        &mut self,
        kpi_id: &str,
        value: String,
        delta: Option<String>,
        trend: String,
    ) -> DashboardMutationResponse {
        let ok = self.dashboard.update_kpi(kpi_id, value, delta, trend);
        DashboardMutationResponse {
            result: ActionResult {
                ok,
                system: "DashboardSystem".to_string(),
                message: if ok { "kpi updated" } else { "kpi not found" }.to_string(),
                entity_id: if ok { Some(kpi_id.to_string()) } else { None },
            },
            dashboard: self.dashboard_snapshot(),
        }
    }

    pub fn push_ai_suggestion(
        &mut self,
        request: NewAiSuggestion,
    ) -> DashboardMutationResponse {
        let s = self.dashboard.push_ai_suggestion(request);
        DashboardMutationResponse {
            result: ActionResult::ok(
                "DashboardSystem",
                "ai suggestion added",
                Some(s.id.clone()),
            ),
            dashboard: self.dashboard_snapshot(),
        }
    }

    pub fn dismiss_ai_suggestion(&mut self, suggestion_id: &str) -> DashboardMutationResponse {
        let ok = self.dashboard.dismiss_ai_suggestion(suggestion_id);
        DashboardMutationResponse {
            result: ActionResult {
                ok,
                system: "DashboardSystem".to_string(),
                message: if ok { "suggestion dismissed" } else { "suggestion not found" }.to_string(),
                entity_id: if ok { Some(suggestion_id.to_string()) } else { None },
            },
            dashboard: self.dashboard_snapshot(),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Profile mutations
    // ─────────────────────────────────────────────────────────────────────────

    pub fn record_user_action(&mut self, request: NewUserAction) -> ProfileMutationResponse {
        let record = self.profile.record_action(request);
        ProfileMutationResponse {
            result: ActionResult::ok(
                "ProfileSystem",
                "user action queued",
                Some(record.id.clone()),
            ),
            profile: self.profile_snapshot(),
        }
    }

    pub fn add_profile_skill(&mut self, request: NewProfileSkill) -> ProfileMutationResponse {
        let added = self.profile.add_skill(request);
        ProfileMutationResponse {
            result: ActionResult {
                ok: added,
                system: "ProfileSystem".to_string(),
                message: if added { "skill added" } else { "skill already present" }.to_string(),
                entity_id: None,
            },
            profile: self.profile_snapshot(),
        }
    }

    pub fn remove_profile_skill(&mut self, skill_name: &str) -> ProfileMutationResponse {
        let removed = self.profile.remove_skill(skill_name);
        ProfileMutationResponse {
            result: ActionResult {
                ok: removed,
                system: "ProfileSystem".to_string(),
                message: if removed { "skill removed" } else { "skill not found" }.to_string(),
                entity_id: None,
            },
            profile: self.profile_snapshot(),
        }
    }

    pub fn add_profile_persona(&mut self, request: NewProfilePersona) -> ProfileMutationResponse {
        let added = self.profile.add_persona(request);
        ProfileMutationResponse {
            result: ActionResult {
                ok: added,
                system: "ProfileSystem".to_string(),
                message: if added { "persona added" } else { "persona already present" }.to_string(),
                entity_id: None,
            },
            profile: self.profile_snapshot(),
        }
    }

    pub fn toggle_profile_persona(&mut self, persona_name: &str) -> ProfileMutationResponse {
        let ok = self.profile.toggle_persona(persona_name);
        ProfileMutationResponse {
            result: ActionResult {
                ok,
                system: "ProfileSystem".to_string(),
                message: if ok { "persona toggled" } else { "persona not found" }.to_string(),
                entity_id: None,
            },
            profile: self.profile_snapshot(),
        }
    }

    pub fn add_profile_contact(&mut self, request: NewProfileContact) -> ProfileMutationResponse {
        let added = self.profile.add_contact(request);
        ProfileMutationResponse {
            result: ActionResult {
                ok: added,
                system: "ProfileSystem".to_string(),
                message: if added { "contact added" } else { "contact already present" }.to_string(),
                entity_id: None,
            },
            profile: self.profile_snapshot(),
        }
    }

    pub fn set_user_presence(
        &mut self,
        status: String,
        message: Option<String>,
    ) -> ProfileMutationResponse {
        self.profile.set_presence(status, message);
        ProfileMutationResponse {
            result: ActionResult::ok("ProfileSystem", "presence updated", None),
            profile: self.profile_snapshot(),
        }
    }

    pub fn add_portable_benefit(
        &mut self,
        kind: String,
        provider: String,
        details: String,
    ) -> ProfileMutationResponse {
        let benefit = self.profile.add_benefit(kind, provider, details);
        ProfileMutationResponse {
            result: ActionResult::ok(
                "ProfileSystem",
                "benefit added",
                Some(benefit.id.clone()),
            ),
            profile: self.profile_snapshot(),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Workspace mutations
    // ─────────────────────────────────────────────────────────────────────────

    pub fn add_workspace_content(
        &mut self,
        request: NewWorkspaceContent,
    ) -> WorkspaceMutationResponse {
        let item = self.workspace.add_content(request);
        WorkspaceMutationResponse {
            result: ActionResult::ok(
                "WorkspaceSystem",
                "content added",
                Some(item.id.clone()),
            ),
            workspace: self.workspace_snapshot(),
        }
    }

    pub fn archive_workspace_content(&mut self, id: &str) -> WorkspaceMutationResponse {
        let ok = self.workspace.archive_content(id);
        WorkspaceMutationResponse {
            result: ActionResult {
                ok,
                system: "WorkspaceSystem".to_string(),
                message: if ok { "content archived" } else { "content not found" }.to_string(),
                entity_id: if ok { Some(id.to_string()) } else { None },
            },
            workspace: self.workspace_snapshot(),
        }
    }

    pub fn add_workspace_schedule(
        &mut self,
        request: NewWorkspaceSchedule,
    ) -> WorkspaceMutationResponse {
        let item = self.workspace.add_schedule(request);
        WorkspaceMutationResponse {
            result: ActionResult::ok(
                "WorkspaceSystem",
                "schedule item added",
                Some(item.id.clone()),
            ),
            workspace: self.workspace_snapshot(),
        }
    }

    pub fn add_workspace_notebook(
        &mut self,
        request: NewWorkspaceNotebook,
    ) -> WorkspaceMutationResponse {
        let nb = self.workspace.add_notebook(request);
        WorkspaceMutationResponse {
            result: ActionResult::ok(
                "WorkspaceSystem",
                "notebook created",
                Some(nb.id.clone()),
            ),
            workspace: self.workspace_snapshot(),
        }
    }

    pub fn add_workspace_note(&mut self, request: NewWorkspaceNote) -> WorkspaceMutationResponse {
        let result = match self.workspace.add_note(request) {
            Some(note) => ActionResult::ok(
                "WorkspaceSystem",
                "note added",
                Some(note.id.clone()),
            ),
            None => ActionResult::err("WorkspaceSystem", "notebook not found"),
        };
        WorkspaceMutationResponse {
            result,
            workspace: self.workspace_snapshot(),
        }
    }

    pub fn add_workspace_contact(
        &mut self,
        request: NewWorkspaceContact,
    ) -> WorkspaceMutationResponse {
        let result = match self.workspace.add_contact(request) {
            Some(contact) => ActionResult::ok(
                "WorkspaceSystem",
                "contact added",
                Some(contact.id.clone()),
            ),
            None => ActionResult::err("WorkspaceSystem", "contact book not found"),
        };
        WorkspaceMutationResponse {
            result,
            workspace: self.workspace_snapshot(),
        }
    }

    pub fn add_board_card(&mut self, request: NewBoardCard) -> WorkspaceMutationResponse {
        let result = match self.workspace.add_board_card(request) {
            Some(card) => ActionResult::ok(
                "WorkspaceSystem",
                "board card added",
                Some(card.id.clone()),
            ),
            None => ActionResult::err("WorkspaceSystem", "board not found"),
        };
        WorkspaceMutationResponse {
            result,
            workspace: self.workspace_snapshot(),
        }
    }

    pub fn move_board_card(
        &mut self,
        board_id: &str,
        card_id: &str,
        column: String,
    ) -> WorkspaceMutationResponse {
        let ok = self.workspace.move_card(board_id, card_id, column);
        WorkspaceMutationResponse {
            result: ActionResult {
                ok,
                system: "WorkspaceSystem".to_string(),
                message: if ok { "card moved" } else { "card or board not found" }.to_string(),
                entity_id: if ok { Some(card_id.to_string()) } else { None },
            },
            workspace: self.workspace_snapshot(),
        }
    }
}
