pub mod domain;
pub mod os_bridge;

pub use domain::{
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
