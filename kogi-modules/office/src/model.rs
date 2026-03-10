use serde::{Deserialize, Serialize};

pub use crate::assistant_system::{AssistantSnapshot, NewAssistantSubscription};
use crate::assistant_system::AssistantSystem;
pub use crate::dashboard_system::DashboardSnapshot;
use crate::dashboard_system::DashboardSystem;
pub use crate::portfolio_system::{NewPortfolioItem, PortfolioSnapshot};
use crate::portfolio_system::PortfolioSystem;
pub use crate::timeline_system::{NewTimelineEvent, TimelineSnapshot};
use crate::timeline_system::TimelineSystem;
pub use crate::workspace_system::{NewWorkspaceStory, WorkspaceSnapshot};
use crate::workspace_system::WorkspaceSystem;

use crate::os_bridge::{
    default_os_compatibility_snapshot, EventRecord, EventType, OsCompatibilitySnapshot,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OfficeViewInfo {
    pub id: String,
    pub title: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OfficeOverview {
    pub module: String,
    pub application: String,
    pub service: String,
    pub systems: Vec<String>,
    pub views: Vec<OfficeViewInfo>,
    pub integrations: Vec<String>,
    pub os_compatibility: OsCompatibilitySnapshot,
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
pub struct PortfolioMutationResponse {
    pub result: ActionResult,
    pub portfolio: PortfolioSnapshot,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimelineMutationResponse {
    pub result: ActionResult,
    pub timeline: TimelineSnapshot,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceMutationResponse {
    pub result: ActionResult,
    pub workspace: WorkspaceSnapshot,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssistantMutationResponse {
    pub result: ActionResult,
    pub assistant: AssistantSnapshot,
}

#[derive(Clone, Debug)]
pub struct OfficeModule {
    dashboard: DashboardSystem,
    portfolio: PortfolioSystem,
    timeline: TimelineSystem,
    workspace: WorkspaceSystem,
    assistant: AssistantSystem,
    os_compatibility: OsCompatibilitySnapshot,
}

impl OfficeModule {
    pub fn mvp() -> Self {
        Self {
            dashboard: DashboardSystem::mvp(),
            portfolio: PortfolioSystem::mvp(),
            timeline: TimelineSystem::mvp(),
            workspace: WorkspaceSystem::mvp(),
            assistant: AssistantSystem::mvp(),
            os_compatibility: default_os_compatibility_snapshot(),
        }
    }

    pub fn overview(&self) -> OfficeOverview {
        OfficeOverview {
            module: "kogi.office".to_string(),
            application: "Kogi Office".to_string(),
            service: "kogi-services/go/services/office".to_string(),
            systems: vec![
                "DashboardSystem".to_string(),
                "PortfolioSystem".to_string(),
                "TimelineSystem".to_string(),
                "WorkspaceSystem".to_string(),
                "AssistantSystem".to_string(),
            ],
            views: vec![
                OfficeViewInfo {
                    id: "dashboard".to_string(),
                    title: "Office Dashboard".to_string(),
                    status: "active".to_string(),
                },
                OfficeViewInfo {
                    id: "portfolio".to_string(),
                    title: "Office Portfolio".to_string(),
                    status: "active".to_string(),
                },
                OfficeViewInfo {
                    id: "timeline".to_string(),
                    title: "Office Timeline".to_string(),
                    status: "active".to_string(),
                },
                OfficeViewInfo {
                    id: "workspace".to_string(),
                    title: "Office Workspace".to_string(),
                    status: "active".to_string(),
                },
                OfficeViewInfo {
                    id: "assistant".to_string(),
                    title: "Office Assistant".to_string(),
                    status: "active".to_string(),
                },
            ],
            integrations: vec![
                "jira".to_string(),
                "monday".to_string(),
                "openai".to_string(),
                "gitlab".to_string(),
                "github".to_string(),
            ],
            os_compatibility: self.os_compatibility.clone(),
        }
    }

    pub fn dashboard_snapshot(&self) -> DashboardSnapshot {
        self.dashboard.snapshot()
    }

    pub fn portfolio_snapshot(&self) -> PortfolioSnapshot {
        self.portfolio.snapshot()
    }

    pub fn timeline_snapshot(&self) -> TimelineSnapshot {
        self.timeline.snapshot()
    }

    pub fn workspace_snapshot(&self) -> WorkspaceSnapshot {
        self.workspace.snapshot()
    }

    pub fn assistant_snapshot(&self) -> AssistantSnapshot {
        self.assistant.snapshot()
    }

    pub fn ack_dashboard_notification(&mut self, notification_id: &str) -> DashboardMutationResponse {
        let acknowledged = self.dashboard.acknowledge_notification(notification_id);
        if acknowledged {
            self.dashboard
                .push_event("events", &format!("Notification {notification_id} acknowledged"));
            self.record_os_event(
                EventType::TaskCompleted,
                "dashboard.system",
                &format!("notification {notification_id} acknowledged"),
            );
        }
        DashboardMutationResponse {
            result: ActionResult {
                ok: acknowledged,
                system: "DashboardSystem".to_string(),
                message: if acknowledged {
                    "notification acknowledged".to_string()
                } else {
                    "notification not found".to_string()
                },
                entity_id: Some(notification_id.to_string()),
            },
            dashboard: self.dashboard_snapshot(),
        }
    }

    pub fn create_portfolio_item(&mut self, request: NewPortfolioItem) -> PortfolioMutationResponse {
        let item = self.portfolio.add_item(request);
        let entity_label = format!("{:?}", item.entity_type).to_lowercase();
        self.dashboard
            .add_attention_from_portfolio(&entity_label, &item.name);
        self.dashboard.push_event(
            "portfolio",
            &format!("Portfolio item {} created", item.name),
        );
        self.assistant
            .add_recommendation("Review newly added portfolio item");
        self.record_os_event(
            EventType::ItemAdded,
            "portfolio.system",
            &format!("portfolio item {} created", item.id),
        );

        PortfolioMutationResponse {
            result: ActionResult {
                ok: true,
                system: "PortfolioSystem".to_string(),
                message: "portfolio item created".to_string(),
                entity_id: Some(item.id.clone()),
            },
            portfolio: self.portfolio_snapshot(),
        }
    }

    pub fn create_timeline_event(&mut self, request: NewTimelineEvent) -> TimelineMutationResponse {
        let event = self.timeline.add_event(request.clone());
        self.dashboard
            .push_event("timeline", &format!("Timeline event {} scheduled", event.title));
        self.record_os_event(
            EventType::TaskCreated,
            "timeline.system",
            &format!("timeline event {} created", event.id),
        );
        self.os_compatibility.scheduler_events.push(crate::os_bridge::CalendarEvent {
            id: event.id.clone(),
            title: event.title.clone(),
            description: format!("{} event", event.kind),
            start_time: event.scheduled_for.clone(),
            end_time: event.scheduled_for.clone(),
            all_day: false,
            owner_profile: "work".to_string(),
        });

        TimelineMutationResponse {
            result: ActionResult {
                ok: true,
                system: "TimelineSystem".to_string(),
                message: "timeline event created".to_string(),
                entity_id: Some(event.id.clone()),
            },
            timeline: self.timeline_snapshot(),
        }
    }

    pub fn create_workspace_story(
        &mut self,
        request: NewWorkspaceStory,
    ) -> WorkspaceMutationResponse {
        let story = self.workspace.add_story(request);
        self.dashboard
            .push_event("workspace", &format!("Workspace story {} created", story.title));
        self.assistant
            .add_recommendation("Review and prioritize the new workspace story");
        self.record_os_event(
            EventType::TaskCreated,
            "workspace.system",
            &format!("workspace story {} created", story.id),
        );

        WorkspaceMutationResponse {
            result: ActionResult {
                ok: true,
                system: "WorkspaceSystem".to_string(),
                message: "workspace story created".to_string(),
                entity_id: Some(story.id.clone()),
            },
            workspace: self.workspace_snapshot(),
        }
    }

    pub fn create_assistant_subscription(
        &mut self,
        request: NewAssistantSubscription,
    ) -> AssistantMutationResponse {
        let topic = self.assistant.add_subscription(request);
        self.dashboard
            .push_event("assistant", &format!("Assistant subscribed to {topic}"));
        self.record_os_event(
            EventType::ConnectionAdded,
            "assistant.system",
            &format!("assistant subscription {topic} active"),
        );

        AssistantMutationResponse {
            result: ActionResult {
                ok: true,
                system: "AssistantSystem".to_string(),
                message: "assistant subscription active".to_string(),
                entity_id: Some(topic),
            },
            assistant: self.assistant_snapshot(),
        }
    }

    fn record_os_event(&mut self, event_type: EventType, source: &str, message: &str) {
        let next = self.os_compatibility.event_bus_history.len() + 1;
        self.os_compatibility.event_bus_history.push(EventRecord {
            id: format!("evt-{:03}", next),
            event_type,
            source: source.to_string(),
            message: message.to_string(),
            timestamp: "2026-03-10T00:00:00Z".to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn office_module_coordinates_subsystems() {
        let mut module = OfficeModule::mvp();
        assert!(module.ack_dashboard_notification("notif-001").result.ok);
        assert!(module
            .create_portfolio_item(NewPortfolioItem {
                item_type: "project".to_string(),
                name: "OS Adapter Port".to_string(),
                status: "active".to_string(),
            })
            .result
            .ok);
        assert!(module
            .create_timeline_event(NewTimelineEvent {
                calendar_id: "cal-work".to_string(),
                title: "Design review".to_string(),
                kind: "review".to_string(),
                scheduled_for: "2026-03-12T18:00:00Z".to_string(),
            })
            .result
            .ok);
        assert!(module
            .create_workspace_story(NewWorkspaceStory {
                title: "As a worker, I can execute office flows".to_string(),
                points: 5,
            })
            .result
            .ok);
        assert!(module
            .create_assistant_subscription(NewAssistantSubscription {
                topic: "office.dashboard.alerts".to_string(),
            })
            .result
            .ok);
    }
}
