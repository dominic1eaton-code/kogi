use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DashboardOverviewCounts {
    pub active_programs: u32,
    pub active_projects: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DashboardOverview {
    pub counts: DashboardOverviewCounts,
    pub portfolio: Vec<String>,
    pub wallet: Vec<String>,
    pub work: Vec<String>,
    pub commerce: Vec<String>,
    pub campaigns: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DashboardNotification {
    pub id: String,
    pub category: String,
    pub message: String,
    pub acknowledged: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DashboardAlert {
    pub id: String,
    pub severity: String,
    pub category: String,
    pub message: String,
    pub acknowledged: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DashboardSnapshot {
    pub view: String,
    pub overview: DashboardOverview,
    pub quick_links: Vec<String>,
    pub notifications: Vec<DashboardNotification>,
    pub alerts: Vec<DashboardAlert>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewDashboardNotification {
    pub category: String,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewDashboardAlert {
    pub severity: String,
    pub category: String,
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct DashboardSystem {
    overview: DashboardOverview,
    quick_links: Vec<String>,
    notifications: Vec<DashboardNotification>,
    alerts: Vec<DashboardAlert>,
    next_notification_id: u64,
    next_alert_id: u64,
}

impl DashboardSystem {
    pub fn mvp() -> Self {
        Self {
            overview: DashboardOverview {
                counts: DashboardOverviewCounts {
                    active_programs: 4,
                    active_projects: 12,
                },
                portfolio: vec![
                    "Core Portfolio: 9 holdings".to_string(),
                    "Growth Portfolio: 4 initiatives".to_string(),
                    "Risk Watchlist: 2 items".to_string(),
                ],
                wallet: vec![
                    "Wallet balance: $32,450".to_string(),
                    "Cashflow runway: 9 months".to_string(),
                    "Pending payouts: $4,800".to_string(),
                ],
                work: vec![
                    "Tasks: 18 open".to_string(),
                    "Gigs: 2 active".to_string(),
                    "Contracts: 3 active".to_string(),
                ],
                commerce: vec![
                    "Orders: 4 pending".to_string(),
                    "Bids: 2 submitted".to_string(),
                    "Deals: 1 negotiation".to_string(),
                    "Requests: 6 open".to_string(),
                    "Proposals: 3 drafted".to_string(),
                ],
                campaigns: vec![
                    "Launch 2026: 42% complete".to_string(),
                    "Community Growth: 18% complete".to_string(),
                ],
            },
            quick_links: vec![
                "/home/workspace".to_string(),
                "/home/profile".to_string(),
                "/home/portfolio".to_string(),
                "/home/notifications".to_string(),
                "/home/calendar".to_string(),
            ],
            notifications: vec![
                DashboardNotification {
                    id: "notif-001".to_string(),
                    category: "work".to_string(),
                    message: "5 tasks are waiting on review".to_string(),
                    acknowledged: false,
                },
                DashboardNotification {
                    id: "notif-002".to_string(),
                    category: "finance".to_string(),
                    message: "Invoice #1024 clears tomorrow".to_string(),
                    acknowledged: false,
                },
            ],
            alerts: vec![
                DashboardAlert {
                    id: "alert-001".to_string(),
                    severity: "high".to_string(),
                    category: "wallet".to_string(),
                    message: "Payout delay detected in funding queue".to_string(),
                    acknowledged: false,
                },
                DashboardAlert {
                    id: "alert-002".to_string(),
                    severity: "medium".to_string(),
                    category: "campaign".to_string(),
                    message: "Campaign CTR dropped below baseline".to_string(),
                    acknowledged: false,
                },
            ],
            next_notification_id: 3,
            next_alert_id: 3,
        }
    }

    pub fn snapshot(&self) -> DashboardSnapshot {
        DashboardSnapshot {
            view: "home.dashboard".to_string(),
            overview: self.overview.clone(),
            quick_links: self.quick_links.clone(),
            notifications: self.notifications.clone(),
            alerts: self.alerts.clone(),
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

    pub fn acknowledge_alert(&mut self, alert_id: &str) -> bool {
        if let Some(alert) = self.alerts.iter_mut().find(|x| x.id == alert_id) {
            alert.acknowledged = true;
            return true;
        }
        false
    }

    pub fn push_notification(&mut self, request: NewDashboardNotification) -> DashboardNotification {
        let notification = DashboardNotification {
            id: format!("notif-{:03}", self.next_notification_id),
            category: request.category,
            message: request.message,
            acknowledged: false,
        };
        self.next_notification_id += 1;
        self.notifications.push(notification.clone());
        notification
    }

    pub fn push_alert(&mut self, request: NewDashboardAlert) -> DashboardAlert {
        let alert = DashboardAlert {
            id: format!("alert-{:03}", self.next_alert_id),
            severity: request.severity,
            category: request.category,
            message: request.message,
            acknowledged: false,
        };
        self.next_alert_id += 1;
        self.alerts.push(alert.clone());
        alert
    }

    pub fn update_counts(&mut self, active_programs: u32, active_projects: u32) {
        self.overview.counts.active_programs = active_programs;
        self.overview.counts.active_projects = active_projects;
    }
}
