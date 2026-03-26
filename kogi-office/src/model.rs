use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkItemStatus {
    Backlog,
    Todo,
    InProgress,
    Blocked,
    Review,
    Done,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkItemType {
    WorkPackage,
    Theme,
    Initiative,
    Epic,
    Story,
    Task,
    Portfolio,
    Program,
    Project,
    Resource,
    Asset,
    Artifact,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StoryType {
    Feature,
    Bug,
    Testing,
    Capability,
    Issue,
    Defect,
    Enhancement,
    Innovation,
    Audit,
    Enabler,
    Blocker,
    UseCase,
    BusinessCase,
    Requirement,
    Documentation,
    Milestone,
    Goal,
    Objective,
    Outcome,
    Mission,
    Vision,
    Risk,
    Analysis,
    Strategy,
    Tactic,
    Operation,
    Plan,
    Report,
    Release,
    Deployment,
    Distribution,
    Template,
    Archive,
    Gig,
    Job,
    Contract,
    Consultation,
    Booking,
    Meeting,
    Appointment,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItem {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub item_type: WorkItemType,
    pub story_type: Option<StoryType>,
    pub status: WorkItemStatus,
    pub owners: Vec<Uuid>,
    pub tags: Vec<String>,
    pub dependencies: Vec<Uuid>,
    pub dependents: Vec<Uuid>,
    pub children: Vec<Uuid>,
    pub parents: Vec<Uuid>,
    pub attachments: Vec<String>,
    pub fields: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub due_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkStream {
    pub title: String,
    pub progress_pct: f64,
    pub status_note: String,
    pub due_this_week: u32,
    pub blocked: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceDashboard {
    pub active_projects: u32,
    pub upcoming_events: u32,
    pub inbox_unread: u32,
    pub studio_ideas: u32,
    pub active_workstreams: Vec<WorkStream>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkWorkspace {
    pub workspace_id: Uuid,
    pub name: String,
    pub dashboard: WorkspaceDashboard,
    pub focus_notes: Vec<String>,
    pub upcoming_schedule: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkBoardColumn {
    pub name: String,
    pub items: Vec<WorkItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkBoardType {
    Agile,
    Kanban,
    Scrum,
    Notes,
    Pipeline,
    Ideas,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkBoard {
    pub board_id: Uuid,
    pub name: String,
    pub board_type: WorkBoardType,
    pub columns: Vec<WorkBoardColumn>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkTimelineEvent {
    pub id: Uuid,
    pub title: String,
    pub schedule: String,
    pub duration: String,
    pub tone: String,
    pub linked_item: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkTimebox {
    pub name: String,
    pub window: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkTimeline {
    pub today: Vec<WorkTimelineEvent>,
    pub timeboxes: Vec<WorkTimebox>,
    pub reconciliation_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub label: String,
    pub value: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkAnalytics {
    pub forecasting: Vec<Metric>,
    pub performance: Vec<Metric>,
    pub kpis: Vec<Metric>,
    pub okrs: Vec<Metric>,
    pub telemetry: Vec<Metric>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkTodoBucket {
    pub label: String,
    pub items: Vec<WorkItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkResourceAllocation {
    pub label: String,
    pub amount: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkResources {
    pub budgeting: Vec<WorkResourceAllocation>,
    pub reporting: Vec<Metric>,
    pub allocation: Vec<WorkResourceAllocation>,
    pub delegation: Vec<Metric>,
    pub todos: Vec<WorkTodoBucket>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentItem {
    pub title: String,
    pub kind: String,
    pub status: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkContentLibrary {
    pub files: Vec<ContentItem>,
    pub documents: Vec<ContentItem>,
    pub contracts: Vec<ContentItem>,
    pub agreements: Vec<ContentItem>,
    pub sops: Vec<ContentItem>,
    pub policies: Vec<ContentItem>,
    pub procedures: Vec<ContentItem>,
    pub frameworks: Vec<ContentItem>,
    pub models: Vec<ContentItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceItem {
    pub title: String,
    pub status: String,
    pub due: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkGovernance {
    pub proposals: Vec<GovernanceItem>,
    pub policies: Vec<GovernanceItem>,
    pub votes_due: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkStudio {
    pub requirements: Vec<ContentItem>,
    pub design_systems: Vec<ContentItem>,
    pub studio_notes: Vec<ContentItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WbsLevel {
    WorkPackage,
    Theme,
    Initiative,
    Epic,
    Story,
    Task,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkBreakdownStructure {
    pub levels: Vec<(WbsLevel, Vec<WorkItem>)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkManagementSystem {
    pub workspace: WorkWorkspace,
    pub backlog: Vec<WorkItem>,
    pub boards: Vec<WorkBoard>,
    pub timelines: WorkTimeline,
    pub analytics: WorkAnalytics,
    pub resources: WorkResources,
    pub content: WorkContentLibrary,
    pub governance: WorkGovernance,
    pub studio: WorkStudio,
    pub wbs: WorkBreakdownStructure,
}
