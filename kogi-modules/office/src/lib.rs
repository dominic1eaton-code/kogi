pub mod os_bridge;

#[path = "DashboardSystem.rs"]
pub mod dashboard_system;
#[path = "PortfolioSystem.rs"]
pub mod portfolio_system;
#[path = "TimelineSystem.rs"]
pub mod timeline_system;
#[path = "WorkspaceSystem.rs"]
pub mod workspace_system;
#[path = "AssistantSystem.rs"]
pub mod assistant_system;

pub mod model;
pub mod domain;

pub use model::{
    ActionResult, AssistantMutationResponse, AssistantSnapshot, DashboardMutationResponse,
    DashboardSnapshot, NewAssistantSubscription, NewPortfolioItem, NewTimelineEvent,
    NewWorkspaceStory, OfficeModule, OfficeOverview, PortfolioMutationResponse,
    PortfolioSnapshot, TimelineMutationResponse, TimelineSnapshot, WorkspaceMutationResponse,
    WorkspaceSnapshot,
};

pub fn to_json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value)
        .unwrap_or_else(|_| "{\"error\":\"serialization_failed\"}".to_string())
}
