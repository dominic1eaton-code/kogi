use serde::{Deserialize, Serialize};

pub use crate::dashboard_system::{
    DashboardAlert, DashboardNotification, DashboardOverview, DashboardOverviewCounts,
    DashboardSnapshot, NewDashboardAlert, NewDashboardNotification,
};
use crate::dashboard_system::DashboardSystem;

pub use crate::profile_system::{
    NewProfileContact, NewProfilePersona, NewProfileSkill, NewUserAction, ProfileAccount,
    ProfileSnapshot, UserActionOption, UserActionRecord,
};
use crate::profile_system::ProfileSystem;

pub use crate::workspace_system::{
    NewWorkspaceContent, NewWorkspaceSchedule, WorkspaceContentItem, WorkspaceContentSnapshot,
    WorkspaceScheduleItem, WorkspaceSnapshot,
};
use crate::workspace_system::WorkspaceSystem;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HomeViewInfo {
    pub id: String,
    pub title: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HomeOverview {
    pub module: String,
    pub application: String,
    pub service: String,
    pub systems: Vec<String>,
    pub views: Vec<HomeViewInfo>,
    pub integrations: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionResult {
    pub ok: bool,
    pub system: String,
    pub message: String,
    pub entity_id: Option<String>,
}

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

#[derive(Clone, Debug)]
pub struct HomeModule {
    dashboard: DashboardSystem,
    profile: ProfileSystem,
    workspace: WorkspaceSystem,
}

impl HomeModule {
    pub fn mvp() -> Self {
        Self {
            dashboard: DashboardSystem::mvp(),
            profile: ProfileSystem::mvp(),
            workspace: WorkspaceSystem::mvp(),
        }
    }

    pub fn overview(&self) -> HomeOverview {
        HomeOverview {
            module: "kogi.home".to_string(),
            application: "Kogi Home".to_string(),
            service: "kogi-network/services/home".to_string(),
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
                },
                HomeViewInfo {
                    id: "profile".to_string(),
                    title: "Home Profile".to_string(),
                    status: "active".to_string(),
                },
                HomeViewInfo {
                    id: "workspace".to_string(),
                    title: "Home Workspace".to_string(),
                    status: "active".to_string(),
                },
            ],
            integrations: vec![
                "wallet".to_string(),
                "portfolio".to_string(),
                "calendar".to_string(),
                "notifications".to_string(),
            ],
        }
    }

    pub fn dashboard_snapshot(&self) -> DashboardSnapshot {
        self.dashboard.snapshot()
    }

    pub fn profile_snapshot(&self) -> ProfileSnapshot {
        self.profile.snapshot()
    }

    pub fn workspace_snapshot(&self) -> WorkspaceSnapshot {
        self.workspace.snapshot()
    }

    pub fn push_dashboard_notification(
        &mut self,
        request: NewDashboardNotification,
    ) -> DashboardMutationResponse {
        let notification = self.dashboard.push_notification(request);
        DashboardMutationResponse {
            result: ActionResult {
                ok: true,
                system: "DashboardSystem".to_string(),
                message: "notification created".to_string(),
                entity_id: Some(notification.id.clone()),
            },
            dashboard: self.dashboard_snapshot(),
        }
    }

    pub fn push_dashboard_alert(&mut self, request: NewDashboardAlert) -> DashboardMutationResponse {
        let alert = self.dashboard.push_alert(request);
        DashboardMutationResponse {
            result: ActionResult {
                ok: true,
                system: "DashboardSystem".to_string(),
                message: "alert created".to_string(),
                entity_id: Some(alert.id.clone()),
            },
            dashboard: self.dashboard_snapshot(),
        }
    }

    pub fn record_user_action(&mut self, request: NewUserAction) -> ProfileMutationResponse {
        let record = self.profile.record_action(request);
        ProfileMutationResponse {
            result: ActionResult {
                ok: true,
                system: "ProfileSystem".to_string(),
                message: "user action queued".to_string(),
                entity_id: Some(record.id.clone()),
            },
            profile: self.profile_snapshot(),
        }
    }

    pub fn add_profile_skill(&mut self, request: NewProfileSkill) -> ProfileMutationResponse {
        let added = self.profile.add_skill(request);
        ProfileMutationResponse {
            result: ActionResult {
                ok: added,
                system: "ProfileSystem".to_string(),
                message: if added {
                    "skill added".to_string()
                } else {
                    "skill already present".to_string()
                },
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
                message: if added {
                    "persona added".to_string()
                } else {
                    "persona already present".to_string()
                },
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
                message: if added {
                    "contact added".to_string()
                } else {
                    "contact already present".to_string()
                },
                entity_id: None,
            },
            profile: self.profile_snapshot(),
        }
    }

    pub fn add_workspace_content(
        &mut self,
        request: NewWorkspaceContent,
    ) -> WorkspaceMutationResponse {
        let item = self.workspace.add_content(request);
        WorkspaceMutationResponse {
            result: ActionResult {
                ok: true,
                system: "WorkspaceSystem".to_string(),
                message: "content added".to_string(),
                entity_id: Some(item.id.clone()),
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
            result: ActionResult {
                ok: true,
                system: "WorkspaceSystem".to_string(),
                message: "schedule item added".to_string(),
                entity_id: Some(item.id.clone()),
            },
            workspace: self.workspace_snapshot(),
        }
    }
}
