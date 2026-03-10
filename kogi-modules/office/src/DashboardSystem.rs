use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DashboardNotification {
    pub id: String,
    pub category: String,
    pub message: String,
    pub acknowledged: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DashboardSnapshot {
    pub view: String,
    pub active_projects: Vec<String>,
    pub active_programs: Vec<String>,
    pub portfolio_attention: Vec<String>,
    pub notifications: Vec<DashboardNotification>,
    pub direct_messages: Vec<String>,
    pub event_feed: Vec<String>,
    pub personas_roles: Vec<String>,
    pub quick_links: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct DashboardSystem {
    notifications: Vec<DashboardNotification>,
    portfolio_attention: Vec<String>,
    event_feed: Vec<String>,
    next_feed_seq: u64,
}

impl DashboardSystem {
    pub fn mvp() -> Self {
        Self {
            notifications: vec![
                DashboardNotification {
                    id: "notif-001".to_string(),
                    category: "task".to_string(),
                    message: "3 blocked stories need triage".to_string(),
                    acknowledged: false,
                },
                DashboardNotification {
                    id: "notif-002".to_string(),
                    category: "integration".to_string(),
                    message: "Jira sync delayed for 2 projects".to_string(),
                    acknowledged: false,
                },
            ],
            portfolio_attention: vec![
                "Kogi Kernel Runtime (project)".to_string(),
                "Platform Compliance Book (document)".to_string(),
            ],
            event_feed: vec![
                "events: Weekly governance sync starts in 2h".to_string(),
                "community: New cooperative member onboarding request".to_string(),
            ],
            next_feed_seq: 3,
        }
    }

    pub fn snapshot(&self) -> DashboardSnapshot {
        DashboardSnapshot {
            view: "dashboard".to_string(),
            active_projects: vec![
                "Kogi MVP Prototype".to_string(),
                "Office Backend Systems".to_string(),
            ],
            active_programs: vec![
                "Platform Launch 2026".to_string(),
                "Cooperative Network Pilot".to_string(),
            ],
            portfolio_attention: self.portfolio_attention.clone(),
            notifications: self.notifications.clone(),
            direct_messages: vec![
                "Ari Program Lead: Roadmap checkpoint".to_string(),
                "Mina Investor: Due diligence docs".to_string(),
            ],
            event_feed: self.event_feed.clone(),
            personas_roles: vec![
                "investor:owner".to_string(),
                "developer:builder".to_string(),
                "donor:sponsor".to_string(),
            ],
            quick_links: vec![
                "/office/portfolio".to_string(),
                "/office/timeline".to_string(),
                "/office/workspace".to_string(),
                "/office/assistant".to_string(),
            ],
        }
    }

    pub fn acknowledge_notification(&mut self, notification_id: &str) -> bool {
        if let Some(notification) = self
            .notifications
            .iter_mut()
            .find(|x| x.id == notification_id)
        {
            notification.acknowledged = true;
            return true;
        }
        false
    }

    pub fn add_attention_from_portfolio(&mut self, entity_type: &str, name: &str) {
        self.portfolio_attention
            .push(format!("{name} ({entity_type})"));
    }

    pub fn push_event(&mut self, source: &str, message: &str) {
        self.event_feed.push(format!("{source}: {message}"));
        self.next_feed_seq += 1;
    }
}
