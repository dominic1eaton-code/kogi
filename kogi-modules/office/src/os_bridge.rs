use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PortfolioEntityType {
    Portfolio,
    SubPortfolio,
    Program,
    Project,
    Resource,
    Artifact,
    Asset,
    Capital,
    Land,
    Estate,
    Labor,
    Investment,
    Account,
    Document,
    Custom,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PortfolioItemType {
    Project,
    Program,
    SubPortfolio,
    Resource,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ResourceType {
    Artifact,
    Asset,
    Capital,
    Land,
    Estate,
    Labor,
    Investment,
    Account,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PortfolioItemContainerType {
    Binder,
    Book,
    Notebook,
    Playbook,
    Folder,
    FileSet,
    VersionControl,
    Metadata,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProcessState {
    Draft,
    Running,
    Paused,
    Completed,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessExecutor {
    pub id: String,
    pub name: String,
    pub executor_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessOutcome {
    pub id: String,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProgramProcess {
    pub id: String,
    pub name: String,
    pub state: ProcessState,
    pub executions: Vec<String>,
    pub actions: Vec<String>,
    pub transformations: Vec<String>,
    pub outcomes: Vec<ProcessOutcome>,
    pub executors: Vec<ProcessExecutor>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StrategyFrame {
    pub vision: String,
    pub mission: String,
    pub objectives: Vec<String>,
    pub goals: Vec<String>,
    pub tactics: Vec<String>,
    pub roadmaps: Vec<String>,
    pub gantt_tracks: Vec<String>,
    pub timelines: Vec<String>,
    pub charters: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: String,
    pub title: String,
    pub description: String,
    pub start_time: String,
    pub end_time: String,
    pub all_day: bool,
    pub owner_profile: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WorkspaceToolType {
    AgileBoards,
    Calendars,
    Timelines,
    Communications,
    WorkStrategyOperations,
    PrototypeStudio,
    Custom,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceTool {
    pub id: String,
    pub tool_type: WorkspaceToolType,
    pub name: String,
    pub description: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceView {
    pub id: String,
    pub name: String,
    pub route: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EventType {
    WorkspaceCreated,
    CollectionCreated,
    ItemAdded,
    TaskCreated,
    TaskCompleted,
    ConnectionAdded,
    SystemStarted,
    SystemError,
    OfficeViewOpened,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventRecord {
    pub id: String,
    pub event_type: EventType,
    pub source: String,
    pub message: String,
    pub timestamp: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OsCompatibilitySnapshot {
    pub portfolio_entity_model: Vec<PortfolioEntityType>,
    pub portfolio_container_model: Vec<PortfolioItemContainerType>,
    pub strategy_frame: StrategyFrame,
    pub scheduler_events: Vec<CalendarEvent>,
    pub workspace_tools: Vec<WorkspaceTool>,
    pub workspace_views: Vec<WorkspaceView>,
    pub event_bus_history: Vec<EventRecord>,
}

pub fn default_strategy_frame() -> StrategyFrame {
    StrategyFrame {
        vision: "Enable independent workers to boot and run complete work systems.".to_string(),
        mission: "Deliver office-grade planning, execution, and governance systems.".to_string(),
        objectives: vec![
            "Consolidate project/program operations".to_string(),
            "Reduce signal loss across modules".to_string(),
            "Improve decision cadence and governance".to_string(),
        ],
        goals: vec![
            "Ship office backend systems".to_string(),
            "Unify web/desktop office views".to_string(),
            "Enable actionable assistant recommendations".to_string(),
        ],
        tactics: vec![
            "Adopt portfolio item containers".to_string(),
            "Track timeline critical-path events".to_string(),
            "Bind stories to workspace toolchains".to_string(),
        ],
        roadmaps: vec![
            "Office MVP -> Office Ops -> Office Autonomous".to_string(),
            "Module isolation hardening roadmap".to_string(),
        ],
        gantt_tracks: vec![
            "Backend systems track".to_string(),
            "Client integration track".to_string(),
            "Data/analytics track".to_string(),
        ],
        timelines: vec![
            "Daily execution timeline".to_string(),
            "Weekly governance timeline".to_string(),
            "Quarterly portfolio timeline".to_string(),
        ],
        charters: vec![
            "Office governance charter".to_string(),
            "Data quality charter".to_string(),
        ],
    }
}

pub fn default_scheduler_events() -> Vec<CalendarEvent> {
    vec![
        CalendarEvent {
            id: "cal-evt-001".to_string(),
            title: "Program standup".to_string(),
            description: "Cross-module office status sync".to_string(),
            start_time: "2026-03-10T15:00:00Z".to_string(),
            end_time: "2026-03-10T15:30:00Z".to_string(),
            all_day: false,
            owner_profile: "work".to_string(),
        },
        CalendarEvent {
            id: "cal-evt-002".to_string(),
            title: "Governance review".to_string(),
            description: "Weekly governance and risk review".to_string(),
            start_time: "2026-03-12T18:00:00Z".to_string(),
            end_time: "2026-03-12T19:00:00Z".to_string(),
            all_day: false,
            owner_profile: "business".to_string(),
        },
    ]
}

pub fn default_workspace_tools() -> Vec<WorkspaceTool> {
    vec![
        WorkspaceTool {
            id: "tool-001".to_string(),
            tool_type: WorkspaceToolType::AgileBoards,
            name: "Agile Boards".to_string(),
            description: "Backlog, board, and sprint management.".to_string(),
            enabled: true,
        },
        WorkspaceTool {
            id: "tool-002".to_string(),
            tool_type: WorkspaceToolType::Calendars,
            name: "Calendars".to_string(),
            description: "Calendar and schedule management.".to_string(),
            enabled: true,
        },
        WorkspaceTool {
            id: "tool-003".to_string(),
            tool_type: WorkspaceToolType::WorkStrategyOperations,
            name: "Work Strategy Operations".to_string(),
            description: "Work, strategy, and operations control center.".to_string(),
            enabled: true,
        },
    ]
}

pub fn default_workspace_views() -> Vec<WorkspaceView> {
    vec![
        WorkspaceView {
            id: "view-hub".to_string(),
            name: "Hub".to_string(),
            route: "/office/workspace/hub".to_string(),
        },
        WorkspaceView {
            id: "view-dashboard".to_string(),
            name: "Dashboard".to_string(),
            route: "/office/workspace/dashboard".to_string(),
        },
        WorkspaceView {
            id: "view-tools".to_string(),
            name: "Tools".to_string(),
            route: "/office/workspace/tools".to_string(),
        },
    ]
}

pub fn default_event_history() -> Vec<EventRecord> {
    vec![
        EventRecord {
            id: "evt-001".to_string(),
            event_type: EventType::SystemStarted,
            source: "office.module".to_string(),
            message: "Office systems initialized".to_string(),
            timestamp: "2026-03-09T17:00:00Z".to_string(),
        },
        EventRecord {
            id: "evt-002".to_string(),
            event_type: EventType::WorkspaceCreated,
            source: "workspace.system".to_string(),
            message: "Primary office workspace loaded".to_string(),
            timestamp: "2026-03-09T17:00:05Z".to_string(),
        },
    ]
}

pub fn default_os_compatibility_snapshot() -> OsCompatibilitySnapshot {
    OsCompatibilitySnapshot {
        portfolio_entity_model: vec![
            PortfolioEntityType::Portfolio,
            PortfolioEntityType::SubPortfolio,
            PortfolioEntityType::Program,
            PortfolioEntityType::Project,
            PortfolioEntityType::Resource,
            PortfolioEntityType::Artifact,
            PortfolioEntityType::Asset,
            PortfolioEntityType::Capital,
            PortfolioEntityType::Land,
            PortfolioEntityType::Estate,
            PortfolioEntityType::Labor,
            PortfolioEntityType::Investment,
            PortfolioEntityType::Account,
            PortfolioEntityType::Document,
            PortfolioEntityType::Custom,
        ],
        portfolio_container_model: vec![
            PortfolioItemContainerType::Binder,
            PortfolioItemContainerType::Book,
            PortfolioItemContainerType::Notebook,
            PortfolioItemContainerType::Playbook,
            PortfolioItemContainerType::Folder,
            PortfolioItemContainerType::FileSet,
            PortfolioItemContainerType::VersionControl,
            PortfolioItemContainerType::Metadata,
        ],
        strategy_frame: default_strategy_frame(),
        scheduler_events: default_scheduler_events(),
        workspace_tools: default_workspace_tools(),
        workspace_views: default_workspace_views(),
        event_bus_history: default_event_history(),
    }
}

pub fn parse_portfolio_kind(item_type: &str) -> (PortfolioItemType, Option<ResourceType>) {
    match item_type {
        "project" => (PortfolioItemType::Project, None),
        "program" => (PortfolioItemType::Program, None),
        "portfolio" | "sub_portfolio" => (PortfolioItemType::SubPortfolio, None),
        "artifact" => (PortfolioItemType::Resource, Some(ResourceType::Artifact)),
        "asset" => (PortfolioItemType::Resource, Some(ResourceType::Asset)),
        "capital" => (PortfolioItemType::Resource, Some(ResourceType::Capital)),
        "land" => (PortfolioItemType::Resource, Some(ResourceType::Land)),
        "estate" => (PortfolioItemType::Resource, Some(ResourceType::Estate)),
        "labor" => (PortfolioItemType::Resource, Some(ResourceType::Labor)),
        "investment" => (PortfolioItemType::Resource, Some(ResourceType::Investment)),
        "account" => (PortfolioItemType::Resource, Some(ResourceType::Account)),
        _ => (PortfolioItemType::Resource, None),
    }
}
