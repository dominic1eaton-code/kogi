// ============================================================================
//  DashboardSystem.rs — Kogi Home · Dashboard System
//  Independent Worker Operating System
//
//  The Dashboard is the primary entry point of the Home module — a real-time
//  command surface giving the user a holistic, at-a-glance view of their
//  entire Kogi workspace:
//
//    Overview          — aggregated counts + section summaries across all
//                        major platform domains (portfolio, wallet, work,
//                        commerce, campaigns, community)
//    QuickLinks        — curated navigation shortcuts
//    Notifications     — informational signals (unacknowledged / acknowledged)
//    Alerts            — severity-tiered operational alerts
//    ActivityFeed      — ordered stream of recent platform events
//    KPIs              — key performance indicators surfaced from the engine
//    AISuggestions     — AI-generated nudges, recommendations, and insights
// ============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────────────
// Severity / Category Enums
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl AlertSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DashboardCategory {
    Work,
    Finance,
    Wallet,
    Portfolio,
    Campaign,
    Community,
    Marketplace,
    System,
    Custom(String),
}

impl DashboardCategory {
    pub fn as_str(&self) -> String {
        match self {
            Self::Work => "work".to_string(),
            Self::Finance => "finance".to_string(),
            Self::Wallet => "wallet".to_string(),
            Self::Portfolio => "portfolio".to_string(),
            Self::Campaign => "campaign".to_string(),
            Self::Community => "community".to_string(),
            Self::Marketplace => "marketplace".to_string(),
            Self::System => "system".to_string(),
            Self::Custom(s) => s.clone(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Overview
// ─────────────────────────────────────────────────────────────────────────────

/// Aggregated summary counts shown at the top of the dashboard.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DashboardOverviewCounts {
    pub active_programs: u32,
    pub active_projects: u32,
    pub open_tasks: u32,
    pub active_gigs: u32,
    pub active_contracts: u32,
    pub pending_orders: u32,
    pub unread_notifications: u32,
    pub unacknowledged_alerts: u32,
}

/// Rich section summaries — each `Vec<String>` holds human-readable bullet
/// items that can be rendered as a mini-panel on the dashboard UI.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DashboardOverview {
    pub counts: DashboardOverviewCounts,
    /// Portfolio section summary lines
    pub portfolio: Vec<String>,
    /// Wallet / financial section summary lines
    pub wallet: Vec<String>,
    /// Work (tasks, gigs, contracts) summary lines
    pub work: Vec<String>,
    /// Commerce (orders, bids, deals, requests, proposals) summary lines
    pub commerce: Vec<String>,
    /// Campaign summary lines
    pub campaigns: Vec<String>,
    /// Community activity summary lines
    pub community: Vec<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Notifications
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DashboardNotification {
    pub id: String,
    pub category: String,
    pub message: String,
    pub acknowledged: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewDashboardNotification {
    pub category: String,
    pub message: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Alerts
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DashboardAlert {
    pub id: String,
    pub severity: String,
    pub category: String,
    pub message: String,
    pub acknowledged: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewDashboardAlert {
    pub severity: String,
    pub category: String,
    pub message: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Activity Feed
// ─────────────────────────────────────────────────────────────────────────────

/// A single event in the platform-wide activity stream surfaced on the dashboard.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActivityFeedItem {
    pub id: String,
    /// Domain that produced the event: "portfolio", "work", "finance", etc.
    pub source: String,
    /// Short human-readable description of what happened.
    pub event: String,
    /// Optional deep-link to the relevant resource.
    pub link: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewActivityFeedItem {
    pub source: String,
    pub event: String,
    pub link: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// KPIs
// ─────────────────────────────────────────────────────────────────────────────

/// A single key-performance-indicator tile displayed on the dashboard.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DashboardKpi {
    pub id: String,
    pub label: String,
    pub value: String,
    /// Human-readable delta vs the previous period, e.g. "+12%", "-$200"
    pub delta: Option<String>,
    /// "positive" | "negative" | "neutral" — drives color coding in the UI
    pub trend: String,
    pub category: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewDashboardKpi {
    pub label: String,
    pub value: String,
    pub delta: Option<String>,
    pub trend: String,
    pub category: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// AI Suggestions
// ─────────────────────────────────────────────────────────────────────────────

/// An AI-generated nudge, recommendation, or insight surfaced on the dashboard.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiSuggestion {
    pub id: String,
    /// "recommendation" | "insight" | "warning" | "opportunity"
    pub kind: String,
    pub message: String,
    /// Optional action the user can take in response.
    pub action_label: Option<String>,
    pub action_link: Option<String>,
    pub dismissed: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewAiSuggestion {
    pub kind: String,
    pub message: String,
    pub action_label: Option<String>,
    pub action_link: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Dashboard Snapshot (read model)
// ─────────────────────────────────────────────────────────────────────────────

/// Complete immutable read-model of the dashboard at a point in time.
/// This is what the UI renders; it carries no mutable handles.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DashboardSnapshot {
    pub view: String,
    pub overview: DashboardOverview,
    pub quick_links: Vec<String>,
    pub notifications: Vec<DashboardNotification>,
    pub alerts: Vec<DashboardAlert>,
    pub activity_feed: Vec<ActivityFeedItem>,
    pub kpis: Vec<DashboardKpi>,
    pub ai_suggestions: Vec<AiSuggestion>,
    pub generated_at: DateTime<Utc>,
}

// ─────────────────────────────────────────────────────────────────────────────
// DashboardSystem — mutable runtime
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct DashboardSystem {
    overview: DashboardOverview,
    quick_links: Vec<String>,
    notifications: Vec<DashboardNotification>,
    alerts: Vec<DashboardAlert>,
    activity_feed: Vec<ActivityFeedItem>,
    kpis: Vec<DashboardKpi>,
    ai_suggestions: Vec<AiSuggestion>,
    next_notification_id: u64,
    next_alert_id: u64,
    next_activity_id: u64,
    next_kpi_id: u64,
    next_suggestion_id: u64,
}

impl DashboardSystem {
    // ── Constructors ──────────────────────────────────────────────────────────

    /// Minimal viable product seed state — rich enough to demonstrate all
    /// dashboard panels with realistic data.
    pub fn mvp() -> Self {
        let now = Utc::now();

        Self {
            overview: DashboardOverview {
                counts: DashboardOverviewCounts {
                    active_programs: 4,
                    active_projects: 12,
                    open_tasks: 18,
                    active_gigs: 2,
                    active_contracts: 3,
                    pending_orders: 4,
                    unread_notifications: 2,
                    unacknowledged_alerts: 2,
                },
                portfolio: vec![
                    "Core Portfolio: 9 holdings".to_string(),
                    "Growth Portfolio: 4 initiatives".to_string(),
                    "Risk Watchlist: 2 items".to_string(),
                ],
                wallet: vec![
                    "Balance: $32,450".to_string(),
                    "Cashflow runway: 9 months".to_string(),
                    "Pending payouts: $4,800".to_string(),
                    "MTD income: $8,200".to_string(),
                    "MTD expenses: $3,100".to_string(),
                ],
                work: vec![
                    "Tasks: 18 open".to_string(),
                    "Gigs: 2 active".to_string(),
                    "Contracts: 3 active".to_string(),
                    "Hours logged this week: 24".to_string(),
                ],
                commerce: vec![
                    "Orders: 4 pending".to_string(),
                    "Bids: 2 submitted".to_string(),
                    "Deals: 1 in negotiation".to_string(),
                    "Requests: 6 open".to_string(),
                    "Proposals: 3 drafted".to_string(),
                ],
                campaigns: vec![
                    "Launch 2026: 42% complete".to_string(),
                    "Community Growth: 18% complete".to_string(),
                ],
                community: vec![
                    "Followers: 340".to_string(),
                    "New connections this week: 12".to_string(),
                    "Unread messages: 5".to_string(),
                ],
            },
            quick_links: vec![
                "/home/workspace".to_string(),
                "/home/profile".to_string(),
                "/home/portfolio".to_string(),
                "/home/notifications".to_string(),
                "/home/calendar".to_string(),
                "/office/boards".to_string(),
                "/marketplace".to_string(),
                "/bank/wallet".to_string(),
            ],
            notifications: vec![
                DashboardNotification {
                    id: "notif-001".to_string(),
                    category: DashboardCategory::Work.as_str(),
                    message: "5 tasks are waiting on review".to_string(),
                    acknowledged: false,
                    created_at: now,
                },
                DashboardNotification {
                    id: "notif-002".to_string(),
                    category: DashboardCategory::Finance.as_str(),
                    message: "Invoice #1024 clears tomorrow".to_string(),
                    acknowledged: false,
                    created_at: now,
                },
            ],
            alerts: vec![
                DashboardAlert {
                    id: "alert-001".to_string(),
                    severity: AlertSeverity::High.to_string(),
                    category: DashboardCategory::Wallet.as_str(),
                    message: "Payout delay detected in funding queue".to_string(),
                    acknowledged: false,
                    created_at: now,
                },
                DashboardAlert {
                    id: "alert-002".to_string(),
                    severity: AlertSeverity::Medium.to_string(),
                    category: DashboardCategory::Campaign.as_str(),
                    message: "Campaign CTR dropped below baseline".to_string(),
                    acknowledged: false,
                    created_at: now,
                },
            ],
            activity_feed: vec![
                ActivityFeedItem {
                    id: "act-001".to_string(),
                    source: "portfolio".to_string(),
                    event: "Project 'API Gateway v2' moved to Active".to_string(),
                    link: Some("/portfolio/projects/api-gateway-v2".to_string()),
                    created_at: now,
                },
                ActivityFeedItem {
                    id: "act-002".to_string(),
                    source: "finance".to_string(),
                    event: "Payment received: $1,200 from client Stark Industries".to_string(),
                    link: Some("/bank/wallet/transactions".to_string()),
                    created_at: now,
                },
                ActivityFeedItem {
                    id: "act-003".to_string(),
                    source: "community".to_string(),
                    event: "New community post received 14 reactions".to_string(),
                    link: Some("/community/feed".to_string()),
                    created_at: now,
                },
                ActivityFeedItem {
                    id: "act-004".to_string(),
                    source: "marketplace".to_string(),
                    event: "Proposal for 'Brand Redesign' accepted".to_string(),
                    link: Some("/marketplace/deals".to_string()),
                    created_at: now,
                },
            ],
            kpis: vec![
                DashboardKpi {
                    id: "kpi-001".to_string(),
                    label: "MTD Revenue".to_string(),
                    value: "$8,200".to_string(),
                    delta: Some("+18% vs last month".to_string()),
                    trend: "positive".to_string(),
                    category: "finance".to_string(),
                },
                DashboardKpi {
                    id: "kpi-002".to_string(),
                    label: "Task Completion Rate".to_string(),
                    value: "74%".to_string(),
                    delta: Some("+5% vs last week".to_string()),
                    trend: "positive".to_string(),
                    category: "work".to_string(),
                },
                DashboardKpi {
                    id: "kpi-003".to_string(),
                    label: "Active Portfolio Value".to_string(),
                    value: "$142,000".to_string(),
                    delta: Some("+$4,200 this quarter".to_string()),
                    trend: "positive".to_string(),
                    category: "portfolio".to_string(),
                },
                DashboardKpi {
                    id: "kpi-004".to_string(),
                    label: "Cashflow Runway".to_string(),
                    value: "9 months".to_string(),
                    delta: Some("-1 month vs last quarter".to_string()),
                    trend: "negative".to_string(),
                    category: "finance".to_string(),
                },
                DashboardKpi {
                    id: "kpi-005".to_string(),
                    label: "Community Engagement Rate".to_string(),
                    value: "6.2%".to_string(),
                    delta: Some("+0.8% vs last week".to_string()),
                    trend: "positive".to_string(),
                    category: "community".to_string(),
                },
            ],
            ai_suggestions: vec![
                AiSuggestion {
                    id: "ai-001".to_string(),
                    kind: "insight".to_string(),
                    message: "You're doing great. 3 tasks are ready to ship — consider closing them to improve your completion rate.".to_string(),
                    action_label: Some("Review tasks".to_string()),
                    action_link: Some("/office/boards/tasks".to_string()),
                    dismissed: false,
                    created_at: now,
                },
                AiSuggestion {
                    id: "ai-002".to_string(),
                    kind: "recommendation".to_string(),
                    message: "Your campaign CTR has dipped. Here's a clearer financial picture for this month — consider adjusting your ad spend.".to_string(),
                    action_label: Some("View campaign".to_string()),
                    action_link: Some("/campaigns/launch-2026".to_string()),
                    dismissed: false,
                    created_at: now,
                },
                AiSuggestion {
                    id: "ai-003".to_string(),
                    kind: "opportunity".to_string(),
                    message: "There are 3 open marketplace requests that match your top skills. Would you like to explore them?".to_string(),
                    action_label: Some("Explore matches".to_string()),
                    action_link: Some("/marketplace/requests".to_string()),
                    dismissed: false,
                    created_at: now,
                },
            ],
            next_notification_id: 3,
            next_alert_id: 3,
            next_activity_id: 5,
            next_kpi_id: 6,
            next_suggestion_id: 4,
        }
    }

    // ── Read ──────────────────────────────────────────────────────────────────

    /// Return a complete immutable snapshot of the dashboard.
    pub fn snapshot(&self) -> DashboardSnapshot {
        DashboardSnapshot {
            view: "home.dashboard".to_string(),
            overview: self.overview.clone(),
            quick_links: self.quick_links.clone(),
            notifications: self.notifications.clone(),
            alerts: self.alerts.clone(),
            activity_feed: self.activity_feed.clone(),
            kpis: self.kpis.clone(),
            ai_suggestions: self.ai_suggestions.clone(),
            generated_at: Utc::now(),
        }
    }

    /// Unacknowledged notifications only.
    pub fn unread_notifications(&self) -> Vec<&DashboardNotification> {
        self.notifications.iter().filter(|n| !n.acknowledged).collect()
    }

    /// Unacknowledged alerts, sorted by severity (critical first).
    pub fn active_alerts(&self) -> Vec<&DashboardAlert> {
        let mut alerts: Vec<&DashboardAlert> =
            self.alerts.iter().filter(|a| !a.acknowledged).collect();
        alerts.sort_by_key(|a| match a.severity.as_str() {
            "critical" => 0,
            "high" => 1,
            "medium" => 2,
            _ => 3,
        });
        alerts
    }

    /// Recent activity feed — most recent first, capped at `limit`.
    pub fn recent_activity(&self, limit: usize) -> Vec<&ActivityFeedItem> {
        self.activity_feed.iter().rev().take(limit).collect()
    }

    // ── Notifications ─────────────────────────────────────────────────────────

    pub fn push_notification(
        &mut self,
        request: NewDashboardNotification,
    ) -> DashboardNotification {
        let notification = DashboardNotification {
            id: format!("notif-{:03}", self.next_notification_id),
            category: request.category,
            message: request.message,
            acknowledged: false,
            created_at: Utc::now(),
        };
        self.next_notification_id += 1;
        self.overview.counts.unread_notifications += 1;
        self.notifications.push(notification.clone());
        notification
    }

    /// Returns `true` if the notification was found and acknowledged.
    pub fn acknowledge_notification(&mut self, notification_id: &str) -> bool {
        if let Some(n) = self.notifications.iter_mut().find(|x| x.id == notification_id) {
            if !n.acknowledged {
                n.acknowledged = true;
                self.overview.counts.unread_notifications =
                    self.overview.counts.unread_notifications.saturating_sub(1);
            }
            return true;
        }
        false
    }

    /// Acknowledge all pending notifications. Returns count acknowledged.
    pub fn acknowledge_all_notifications(&mut self) -> u32 {
        let mut count = 0u32;
        for n in self.notifications.iter_mut().filter(|x| !x.acknowledged) {
            n.acknowledged = true;
            count += 1;
        }
        self.overview.counts.unread_notifications = 0;
        count
    }

    // ── Alerts ────────────────────────────────────────────────────────────────

    pub fn push_alert(&mut self, request: NewDashboardAlert) -> DashboardAlert {
        let alert = DashboardAlert {
            id: format!("alert-{:03}", self.next_alert_id),
            severity: request.severity,
            category: request.category,
            message: request.message,
            acknowledged: false,
            created_at: Utc::now(),
        };
        self.next_alert_id += 1;
        self.overview.counts.unacknowledged_alerts += 1;
        self.alerts.push(alert.clone());
        alert
    }

    /// Returns `true` if the alert was found and acknowledged.
    pub fn acknowledge_alert(&mut self, alert_id: &str) -> bool {
        if let Some(a) = self.alerts.iter_mut().find(|x| x.id == alert_id) {
            if !a.acknowledged {
                a.acknowledged = true;
                self.overview.counts.unacknowledged_alerts =
                    self.overview.counts.unacknowledged_alerts.saturating_sub(1);
            }
            return true;
        }
        false
    }

    // ── Activity Feed ─────────────────────────────────────────────────────────

    pub fn push_activity(&mut self, request: NewActivityFeedItem) -> ActivityFeedItem {
        let item = ActivityFeedItem {
            id: format!("act-{:03}", self.next_activity_id),
            source: request.source,
            event: request.event,
            link: request.link,
            created_at: Utc::now(),
        };
        self.next_activity_id += 1;
        self.activity_feed.push(item.clone());
        item
    }

    // ── KPIs ──────────────────────────────────────────────────────────────────

    pub fn add_kpi(&mut self, request: NewDashboardKpi) -> DashboardKpi {
        let kpi = DashboardKpi {
            id: format!("kpi-{:03}", self.next_kpi_id),
            label: request.label,
            value: request.value,
            delta: request.delta,
            trend: request.trend,
            category: request.category,
        };
        self.next_kpi_id += 1;
        self.kpis.push(kpi.clone());
        kpi
    }

    pub fn update_kpi(&mut self, kpi_id: &str, value: String, delta: Option<String>, trend: String) -> bool {
        if let Some(kpi) = self.kpis.iter_mut().find(|k| k.id == kpi_id) {
            kpi.value = value;
            kpi.delta = delta;
            kpi.trend = trend;
            return true;
        }
        false
    }

    // ── AI Suggestions ────────────────────────────────────────────────────────

    pub fn push_ai_suggestion(&mut self, request: NewAiSuggestion) -> AiSuggestion {
        let suggestion = AiSuggestion {
            id: format!("ai-{:03}", self.next_suggestion_id),
            kind: request.kind,
            message: request.message,
            action_label: request.action_label,
            action_link: request.action_link,
            dismissed: false,
            created_at: Utc::now(),
        };
        self.next_suggestion_id += 1;
        self.ai_suggestions.push(suggestion.clone());
        suggestion
    }

    pub fn dismiss_ai_suggestion(&mut self, suggestion_id: &str) -> bool {
        if let Some(s) = self.ai_suggestions.iter_mut().find(|x| x.id == suggestion_id) {
            s.dismissed = true;
            return true;
        }
        false
    }

    // ── Overview Mutations ────────────────────────────────────────────────────

    pub fn update_counts(&mut self, active_programs: u32, active_projects: u32) {
        self.overview.counts.active_programs = active_programs;
        self.overview.counts.active_projects = active_projects;
    }

    pub fn update_portfolio_summary(&mut self, lines: Vec<String>) {
        self.overview.portfolio = lines;
    }

    pub fn update_wallet_summary(&mut self, lines: Vec<String>) {
        self.overview.wallet = lines;
    }

    pub fn update_work_summary(&mut self, lines: Vec<String>) {
        self.overview.work = lines;
    }

    pub fn update_commerce_summary(&mut self, lines: Vec<String>) {
        self.overview.commerce = lines;
    }

    pub fn update_campaigns_summary(&mut self, lines: Vec<String>) {
        self.overview.campaigns = lines;
    }

    pub fn update_community_summary(&mut self, lines: Vec<String>) {
        self.overview.community = lines;
    }

    pub fn add_quick_link(&mut self, link: String) -> bool {
        if self.quick_links.contains(&link) {
            return false;
        }
        self.quick_links.push(link);
        true
    }

    pub fn remove_quick_link(&mut self, link: &str) -> bool {
        if let Some(pos) = self.quick_links.iter().position(|l| l == link) {
            self.quick_links.remove(pos);
            return true;
        }
        false
    }
}
