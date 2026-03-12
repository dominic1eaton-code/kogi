#[path = "DashboardSystem.rs"]
pub mod dashboard_system;
#[path = "ProfileSystem.rs"]
pub mod profile_system;
#[path = "WorkspaceSystem.rs"]
pub mod workspace_system;

pub mod model;
pub mod domain;

pub use model::{
    ActionResult, DashboardAlert, DashboardMutationResponse, DashboardNotification,
    DashboardOverview, DashboardOverviewCounts, DashboardSnapshot, HomeModule, HomeOverview,
    HomeViewInfo, NewDashboardAlert, NewDashboardNotification, NewProfileContact, NewProfilePersona,
    NewProfileSkill, NewUserAction, NewWorkspaceContent, NewWorkspaceSchedule, ProfileAccount,
    ProfileMutationResponse, ProfileSnapshot, UserActionOption, UserActionRecord,
    WorkspaceContentItem, WorkspaceContentSnapshot, WorkspaceMutationResponse, WorkspaceScheduleItem,
    WorkspaceSnapshot,
};

pub fn to_json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value)
        .unwrap_or_else(|_| "{\"error\":\"serialization_failed\"}".to_string())
}
