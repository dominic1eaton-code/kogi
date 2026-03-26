use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use kogi_portfolio::{KogiPortfolioConfig, PortfolioRow, PortfolioSystem};

use crate::error::OfficeResult;
use crate::model::*;

#[derive(Debug, Clone)]
pub struct OfficeConfig {
    pub portfolio_config: KogiPortfolioConfig,
    pub office_id: Uuid,
}

impl Default for OfficeConfig {
    fn default() -> Self {
        Self {
            portfolio_config: KogiPortfolioConfig::default(),
            office_id: Uuid::new_v4(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfficeState {
    pub office_id: Uuid,
    pub portfolio_root_component_id: Uuid,
    pub workbook_id: Uuid,
    pub project_count: usize,
    pub task_count: usize,
    pub resource_count: usize,
    pub asset_count: usize,
    pub artifact_count: usize,
    pub work_item_count: usize,
    pub last_refresh_at: DateTime<Utc>,
}

pub struct OfficeSystem {
    pub config: OfficeConfig,
    pub portfolio: PortfolioSystem,
    last_refresh_at: DateTime<Utc>,
}

impl OfficeSystem {
    pub fn new(config: OfficeConfig) -> OfficeResult<Self> {
        let portfolio = PortfolioSystem::new(config.portfolio_config.clone())?;
        Ok(Self {
            config,
            portfolio,
            last_refresh_at: Utc::now(),
        })
    }

    pub fn state(&self) -> OfficeState {
        let projects = self.portfolio.runtime.master.workbook.rows_for_sheet("SHT-004");
        let tasks = self.portfolio.runtime.master.workbook.rows_for_sheet("SHT-005");
        let resources = self.portfolio.runtime.master.workbook.rows_for_sheet("SHT-006");
        let assets = self.portfolio.runtime.master.workbook.rows_for_sheet("SHT-007");
        let artifacts = self.portfolio.runtime.master.workbook.rows_for_sheet("SHT-008");
        let work_items = self.work_items();
        OfficeState {
            office_id: self.config.office_id,
            portfolio_root_component_id: self.portfolio.root_component_id(),
            workbook_id: self.portfolio.state().workbook_id,
            project_count: projects.len(),
            task_count: tasks.len(),
            resource_count: resources.len(),
            asset_count: assets.len(),
            artifact_count: artifacts.len(),
            work_item_count: work_items.len(),
            last_refresh_at: self.last_refresh_at,
        }
    }

    pub fn refresh(&mut self) -> OfficeResult<()> {
        self.portfolio.sync_from_grid()?;
        self.portfolio.persist()?;
        self.last_refresh_at = Utc::now();
        Ok(())
    }

    pub fn work_items(&self) -> Vec<WorkItem> {
        let mut items: Vec<WorkItem> = Vec::new();
        for row in self.portfolio.runtime.master.workbook.row_store.iter() {
            if is_work_item(row) {
                items.push(work_item_from_row(row));
            }
        }
        items
    }

    pub fn work_management_system(&self) -> WorkManagementSystem {
        let work_items = self.work_items();
        let workspace = build_workspace(&work_items);
        let backlog = work_items.clone();
        let boards = build_boards(&work_items);
        let timelines = build_timelines(&work_items);
        let analytics = build_analytics(&work_items);
        let resources = build_resources(&self.portfolio.runtime.master.workbook);
        let content = build_content(&self.portfolio.runtime.master.workbook);
        let governance = build_governance(&self.portfolio.runtime.master.workbook);
        let studio = build_studio(&self.portfolio.runtime.master.workbook);
        let wbs = build_wbs(&work_items);
        WorkManagementSystem {
            workspace,
            backlog,
            boards,
            timelines,
            analytics,
            resources,
            content,
            governance,
            studio,
            wbs,
        }
    }
}

fn is_work_item(row: &PortfolioRow) -> bool {
    row.item_type
        .as_deref()
        .or(row.container_type.as_deref())
        .map(|t| matches_work_item_type(t))
        .unwrap_or(false)
        || row.tags.iter().any(|t| t.eq_ignore_ascii_case("work"))
        || row.tags.iter().any(|t| t.eq_ignore_ascii_case("task"))
        || row.topics.iter().any(|t| t.eq_ignore_ascii_case("work"))
}

fn matches_work_item_type(raw: &str) -> bool {
    matches!(
        raw.to_lowercase().as_str(),
        "work_package"
            | "workpackage"
            | "theme"
            | "initiative"
            | "epic"
            | "story"
            | "task"
            | "portfolio"
            | "program"
            | "project"
            | "resource"
            | "asset"
            | "artifact"
    )
}

fn work_item_from_row(row: &PortfolioRow) -> WorkItem {
    let item_type = work_item_type_from_row(row);
    let story_type = row
        .ext
        .get("story_type")
        .and_then(|v| v.as_str())
        .and_then(story_type_from_str);
    WorkItem {
        id: row.component_id,
        title: row.display_name.clone().unwrap_or_else(|| row.name.clone()),
        description: row
            .ext
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        item_type,
        story_type,
        status: work_status_from_str(&row.status),
        owners: row.owners.clone(),
        tags: row.tags.clone(),
        dependencies: row.dependency_ids.clone(),
        dependents: row.dependent_ids.clone(),
        children: row.child_ids.clone(),
        parents: row.parent_id.into_iter().collect(),
        attachments: list_from_value(row.ext.get("attachments")),
        fields: json!({ "portfolio_row": row }),
        created_at: row.created_at,
        updated_at: row.updated_at,
        due_date: row.due_date,
    }
}

fn work_item_type_from_row(row: &PortfolioRow) -> WorkItemType {
    let primary = row.primary_type().unwrap_or_else(|| row.component_type.clone());
    match primary.to_lowercase().as_str() {
        "work_package" | "workpackage" => WorkItemType::WorkPackage,
        "theme" => WorkItemType::Theme,
        "initiative" => WorkItemType::Initiative,
        "epic" => WorkItemType::Epic,
        "story" => WorkItemType::Story,
        "task" => WorkItemType::Task,
        "portfolio" => WorkItemType::Portfolio,
        "program" => WorkItemType::Program,
        "project" => WorkItemType::Project,
        "resource" => WorkItemType::Resource,
        "asset" => WorkItemType::Asset,
        "artifact" => WorkItemType::Artifact,
        _ => WorkItemType::Custom,
    }
}

fn story_type_from_str(raw: &str) -> Option<StoryType> {
    let key = raw.to_lowercase();
    Some(match key.as_str() {
        "feature" => StoryType::Feature,
        "bug" => StoryType::Bug,
        "testing" => StoryType::Testing,
        "capability" => StoryType::Capability,
        "issue" => StoryType::Issue,
        "defect" => StoryType::Defect,
        "enhancement" => StoryType::Enhancement,
        "innovation" => StoryType::Innovation,
        "audit" => StoryType::Audit,
        "enabler" => StoryType::Enabler,
        "blocker" => StoryType::Blocker,
        "use_case" | "usecase" => StoryType::UseCase,
        "business_case" | "businesscase" => StoryType::BusinessCase,
        "requirement" => StoryType::Requirement,
        "documentation" => StoryType::Documentation,
        "milestone" => StoryType::Milestone,
        "goal" => StoryType::Goal,
        "objective" => StoryType::Objective,
        "outcome" => StoryType::Outcome,
        "mission" => StoryType::Mission,
        "vision" => StoryType::Vision,
        "risk" => StoryType::Risk,
        "analysis" => StoryType::Analysis,
        "strategy" => StoryType::Strategy,
        "tactic" => StoryType::Tactic,
        "operation" => StoryType::Operation,
        "plan" => StoryType::Plan,
        "report" => StoryType::Report,
        "release" => StoryType::Release,
        "deployment" => StoryType::Deployment,
        "distribution" => StoryType::Distribution,
        "template" => StoryType::Template,
        "archive" => StoryType::Archive,
        "gig" => StoryType::Gig,
        "job" => StoryType::Job,
        "contract" => StoryType::Contract,
        "consultation" => StoryType::Consultation,
        "booking" => StoryType::Booking,
        "meeting" => StoryType::Meeting,
        "appointment" => StoryType::Appointment,
        _ => StoryType::Custom,
    })
}

fn work_status_from_str(raw: &str) -> WorkItemStatus {
    match raw.to_lowercase().as_str() {
        "backlog" => WorkItemStatus::Backlog,
        "todo" | "to_do" | "to-do" => WorkItemStatus::Todo,
        "in_progress" | "in-progress" | "active" => WorkItemStatus::InProgress,
        "blocked" => WorkItemStatus::Blocked,
        "review" => WorkItemStatus::Review,
        "done" | "completed" => WorkItemStatus::Done,
        "archived" => WorkItemStatus::Archived,
        _ => WorkItemStatus::Backlog,
    }
}

fn build_workspace(items: &[WorkItem]) -> WorkWorkspace {
    let active_projects = items.iter().filter(|i| matches!(i.item_type, WorkItemType::Project)).count();
    let backlog_count = items.iter().filter(|i| matches!(i.status, WorkItemStatus::Backlog)).count();
    let upcoming_events = items.iter().filter(|i| matches!(i.story_type, Some(StoryType::Meeting | StoryType::Appointment | StoryType::Booking))).count();
    let workstreams = items.iter().take(3).map(|item| WorkStream {
        title: item.title.clone(),
        progress_pct: 60.0,
        status_note: "Active".to_string(),
        due_this_week: 2,
        blocked: 0,
    }).collect::<Vec<_>>();
    WorkWorkspace {
        workspace_id: Uuid::new_v4(),
        name: "Office Workspace".to_string(),
        dashboard: WorkspaceDashboard {
            active_projects: active_projects as u32,
            upcoming_events: upcoming_events as u32,
            inbox_unread: 9,
            studio_ideas: 42,
            active_workstreams: workstreams,
        },
        focus_notes: vec![
            "Resolve blocked items before Friday.".to_string(),
            "Confirm next sprint timebox.".to_string(),
        ],
        upcoming_schedule: vec![
            format!("Backlog items: {backlog_count}"),
            "Studio sync · 10:00 AM".to_string(),
        ],
    }
}

fn build_boards(items: &[WorkItem]) -> Vec<WorkBoard> {
    let mut columns: HashMap<WorkItemStatus, Vec<WorkItem>> = HashMap::new();
    for item in items {
        columns.entry(item.status.clone()).or_default().push(item.clone());
    }
    let board_columns = vec![
        WorkBoardColumn { name: "Backlog".to_string(), items: columns.remove(&WorkItemStatus::Backlog).unwrap_or_default() },
        WorkBoardColumn { name: "In Progress".to_string(), items: columns.remove(&WorkItemStatus::InProgress).unwrap_or_default() },
        WorkBoardColumn { name: "Review".to_string(), items: columns.remove(&WorkItemStatus::Review).unwrap_or_default() },
        WorkBoardColumn { name: "Done".to_string(), items: columns.remove(&WorkItemStatus::Done).unwrap_or_default() },
    ];
    vec![
        WorkBoard {
            board_id: Uuid::new_v4(),
            name: "Agile Board".to_string(),
            board_type: WorkBoardType::Agile,
            columns: board_columns,
        },
        WorkBoard {
            board_id: Uuid::new_v4(),
            name: "Idea Board".to_string(),
            board_type: WorkBoardType::Ideas,
            columns: vec![
                WorkBoardColumn { name: "Ideas".to_string(), items: items.iter().take(4).cloned().collect() },
                WorkBoardColumn { name: "Concepts".to_string(), items: vec![] },
                WorkBoardColumn { name: "Designs".to_string(), items: vec![] },
                WorkBoardColumn { name: "Prototypes".to_string(), items: vec![] },
            ],
        },
    ]
}

fn build_timelines(items: &[WorkItem]) -> WorkTimeline {
    let today = items
        .iter()
        .take(3)
        .enumerate()
        .map(|(idx, item)| WorkTimelineEvent {
            id: item.id,
            title: item.title.clone(),
            schedule: match idx {
                0 => "09:00 AM".to_string(),
                1 => "11:30 AM".to_string(),
                _ => "02:00 PM".to_string(),
            },
            duration: match idx {
                0 => "90m".to_string(),
                1 => "30m".to_string(),
                _ => "2h".to_string(),
            },
            tone: "text-[#10b981]".to_string(),
            linked_item: Some(item.id),
        })
        .collect::<Vec<_>>();
    WorkTimeline {
        today,
        timeboxes: vec![
            WorkTimebox { name: "Sprint 14".to_string(), window: "Mar 10 - Mar 21".to_string(), note: "12 tasks".to_string() },
            WorkTimebox { name: "Governance Cycle".to_string(), window: "Mar 15 - Apr 1".to_string(), note: "Voting open".to_string() },
        ],
        reconciliation_notes: vec![
            "Overlap Detected: Client Review vs Studio Sync".to_string(),
            "Resource Conflict: Studio Room reserved".to_string(),
        ],
    }
}

fn build_analytics(items: &[WorkItem]) -> WorkAnalytics {
    let completed = items.iter().filter(|i| matches!(i.status, WorkItemStatus::Done)).count();
    let total = items.len().max(1);
    let completion_pct = (completed as f64 / total as f64) * 100.0;
    WorkAnalytics {
        forecasting: vec![
            Metric { label: "Forecast Completion".to_string(), value: format!("{completion_pct:.1}%"), note: "Based on backlog velocity".to_string() }
        ],
        performance: vec![
            Metric { label: "Cycle Time".to_string(), value: "3.4 days".to_string(), note: "Avg".to_string() },
            Metric { label: "Health Score".to_string(), value: "84".to_string(), note: "Portfolio avg".to_string() },
        ],
        kpis: vec![
            Metric { label: "Velocity".to_string(), value: "28 pts".to_string(), note: "Last sprint".to_string() }
        ],
        okrs: vec![
            Metric { label: "Q2 Delivery".to_string(), value: "72%".to_string(), note: "On track".to_string() }
        ],
        telemetry: vec![
            Metric { label: "Active Workspaces".to_string(), value: "4".to_string(), note: "Today".to_string() }
        ],
    }
}

fn build_resources(workbook: &kogi_portfolio::SpreadsheetWorkbook) -> WorkResources {
    let mut budgeting = Vec::new();
    let mut allocation = Vec::new();
    for row in workbook.row_store.iter() {
        if let Some(budget) = row.budget_allocated {
            budgeting.push(WorkResourceAllocation {
                label: row.name.clone(),
                amount: format!("${}", budget),
                note: "Allocated".to_string(),
            });
        }
        if let Some(util) = row.resource_units_total {
            allocation.push(WorkResourceAllocation {
                label: row.name.clone(),
                amount: format!("{:.0} units", util),
                note: "Resource pool".to_string(),
            });
        }
        if budgeting.len() >= 3 && allocation.len() >= 3 {
            break;
        }
    }

    let todos = vec![
        WorkTodoBucket { label: "Do Now".to_string(), items: vec![] },
        WorkTodoBucket { label: "Do Later".to_string(), items: vec![] },
        WorkTodoBucket { label: "Delegate".to_string(), items: vec![] },
        WorkTodoBucket { label: "Marked for Deletion".to_string(), items: vec![] },
    ];
    WorkResources {
        budgeting,
        reporting: vec![
            Metric { label: "Budget Utilization".to_string(), value: "62%".to_string(), note: "Allocated vs spent".to_string() }
        ],
        allocation,
        delegation: vec![
            Metric { label: "Delegated Tasks".to_string(), value: "4".to_string(), note: "Awaiting updates".to_string() }
        ],
        todos,
    }
}

fn build_content(workbook: &kogi_portfolio::SpreadsheetWorkbook) -> WorkContentLibrary {
    let mut items = Vec::new();
    for row in workbook.row_store.iter() {
        if let Some(kind) = row.item_type.as_deref() {
            if kind.eq_ignore_ascii_case("artifact") || kind.eq_ignore_ascii_case("document") {
                items.push(ContentItem {
                    title: row.name.clone(),
                    kind: kind.to_string(),
                    status: row.status.clone(),
                    updated_at: row.updated_at,
                });
            }
        }
    }
    let sample = items.iter().take(2).cloned().collect::<Vec<_>>();
    WorkContentLibrary {
        files: sample.clone(),
        documents: sample.clone(),
        contracts: vec![],
        agreements: vec![],
        sops: vec![],
        policies: vec![],
        procedures: vec![],
        frameworks: vec![],
        models: vec![],
    }
}

fn build_governance(workbook: &kogi_portfolio::SpreadsheetWorkbook) -> WorkGovernance {
    let mut proposals = Vec::new();
    let mut policies = Vec::new();
    for row in workbook.row_store.iter() {
        if row.tags.iter().any(|t| t.eq_ignore_ascii_case("governance")) {
            proposals.push(GovernanceItem {
                title: row.name.clone(),
                status: row.status.clone(),
                due: row.due_date.map(|d| d.to_string()),
            });
        }
        if row.tags.iter().any(|t| t.eq_ignore_ascii_case("policy")) {
            policies.push(GovernanceItem {
                title: row.name.clone(),
                status: row.status.clone(),
                due: row.due_date.map(|d| d.to_string()),
            });
        }
        if proposals.len() >= 2 && policies.len() >= 1 {
            break;
        }
    }
    WorkGovernance {
        proposals,
        policies,
        votes_due: 2,
    }
}

fn build_studio(workbook: &kogi_portfolio::SpreadsheetWorkbook) -> WorkStudio {
    let mut requirements = Vec::new();
    let mut design_systems = Vec::new();
    let mut notes = Vec::new();
    for row in workbook.row_store.iter() {
        if row.tags.iter().any(|t| t.eq_ignore_ascii_case("requirement")) {
            requirements.push(ContentItem {
                title: row.name.clone(),
                kind: "requirement".to_string(),
                status: row.status.clone(),
                updated_at: row.updated_at,
            });
        }
        if row.tags.iter().any(|t| t.eq_ignore_ascii_case("design")) {
            design_systems.push(ContentItem {
                title: row.name.clone(),
                kind: "design".to_string(),
                status: row.status.clone(),
                updated_at: row.updated_at,
            });
        }
        if row.tags.iter().any(|t| t.eq_ignore_ascii_case("notes")) {
            notes.push(ContentItem {
                title: row.name.clone(),
                kind: "note".to_string(),
                status: row.status.clone(),
                updated_at: row.updated_at,
            });
        }
        if requirements.len() >= 2 && design_systems.len() >= 2 && notes.len() >= 2 {
            break;
        }
    }
    WorkStudio {
        requirements,
        design_systems,
        studio_notes: notes,
    }
}

fn build_wbs(items: &[WorkItem]) -> WorkBreakdownStructure {
    let mut buckets: HashMap<WbsLevel, Vec<WorkItem>> = HashMap::new();
    for item in items {
        if let Some(level) = wbs_level_for_item(item) {
            buckets.entry(level).or_default().push(item.clone());
        }
    }
    let mut levels = Vec::new();
    for level in [
        WbsLevel::WorkPackage,
        WbsLevel::Theme,
        WbsLevel::Initiative,
        WbsLevel::Epic,
        WbsLevel::Story,
        WbsLevel::Task,
    ] {
        levels.push((level.clone(), buckets.remove(&level).unwrap_or_default()));
    }
    WorkBreakdownStructure { levels }
}

fn wbs_level_for_item(item: &WorkItem) -> Option<WbsLevel> {
    match item.item_type {
        WorkItemType::WorkPackage => Some(WbsLevel::WorkPackage),
        WorkItemType::Theme => Some(WbsLevel::Theme),
        WorkItemType::Initiative => Some(WbsLevel::Initiative),
        WorkItemType::Epic => Some(WbsLevel::Epic),
        WorkItemType::Story => Some(WbsLevel::Story),
        WorkItemType::Task => Some(WbsLevel::Task),
        _ => None,
    }
}

fn list_from_value(value: Option<&Value>) -> Vec<String> {
    let Some(value) = value else { return vec![]; };
    match value {
        Value::Array(items) => items
            .iter()
            .filter_map(|item| item.as_str().map(|s| s.to_string()))
            .collect(),
        Value::String(s) => s
            .split(',')
            .map(|item| item.trim())
            .filter(|item| !item.is_empty())
            .map(|item| item.to_string())
            .collect(),
        _ => vec![],
    }
}
