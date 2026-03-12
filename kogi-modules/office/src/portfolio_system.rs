// =============================================================================
//  portfolio_system.rs — Kogi OS · Portfolio System  (Reconciled v2.0)
//  Independent Worker Operating System
//
//  This file is the single authoritative source for the Kogi Portfolio domain.
//  It reconciles all prior versions (portfolio.rs, PortfolioSystem.rs,
//  PortfolioSystem_test.rs) using portfolio.rs as the primary structural
//  baseline, and layering in every proven abstraction from the other files.
//
//  ─────────────────────────────────────────────────────────────────────────────
//  Structural Hierarchy
//  ─────────────────────────────────────────────────────────────────────────────
//
//    PortfolioSystem
//      ├── Component  (Item | Container)
//      │     ├── ComponentMetadata   id, owners, tags, policy_ids,
//      │     │                       timestamps, vector_clock, properties,
//      │     │                       version, budget, resource_units
//      │     ├── ComponentData       category, name, status, state,
//      │     │                       relations, users, analytics, governance
//      │     ├── component:item      Portfolio | Program | Project |
//      │     │                       Resource | Artifact | Asset | SubPortfolio
//      │     └── component:container Binder | Book(*) | Record | Folder |
//      │                             Registry | Archive
//      ├── GraphEdge  (Hierarchy | Dependency | Link | Contains | Federation)
//      ├── EventLog   (append-only, event-sourced audit trail)
//      ├── CrdtLog    (distributed operations — LWW + OR-Set)
//      ├── VectorClock
//      ├── Structural primitives
//      │     Group · Collection · List · Schedule · Directory
//      ├── Governance
//      │     PolicyEngine trait · ApprovalWorkflow · ResourceAllocation
//      ├── Plugins   (PortfolioPlugin trait)
//      ├── Federation (PortfolioFederation — multi-system sync)
//      └── Computational Models
//            PortfolioHealth · ProjectMetrics · ProgramAlignment ·
//            SubPortfolioRollup · ResourceUtilisation · AssetValue ·
//            ArtifactMaturity · BinderCoverage · BookConsistency ·
//            FolderOrganisation · RecordIntegrity
//
//  @author  Kogi Team
//  @version 2.0.0
//  @license MIT
// =============================================================================

#![allow(dead_code)]

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =============================================================================
// §1 — TYPE ALIASES
// =============================================================================

pub type ComponentId   = Uuid;
pub type UserId        = Uuid;
pub type PolicyId      = Uuid;
pub type TagId         = Uuid;
pub type VersionString = String;
pub type Properties    = HashMap<String, serde_json::Value>;

// =============================================================================
// §2 — UTILITIES
// =============================================================================

/// Unix epoch seconds (used where `chrono` is not yet in scope).
fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Generate a unique tag string for OR-Set entries.
fn unique_tag(actor: &str, ts: u64, seq: u64) -> String {
    format!("{actor}:{ts}:{seq}")
}

// =============================================================================
// §3 — ERROR TYPES
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PortfolioError {
    NotFound(ComponentId),
    PermissionDenied { user_id: UserId, action: String },
    InvalidOperation(String),
    CyclicDependency(ComponentId, ComponentId),
    VersionConflict { local: VersionString, remote: VersionString },
    AlreadyExists(ComponentId),
    InvalidState { current: ComponentState, attempted: String },
    StorageError(String),
    ValidationError(String),
}

impl fmt::Display for PortfolioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(id) =>
                write!(f, "Component not found: {id}"),
            Self::PermissionDenied { user_id, action } =>
                write!(f, "User {user_id} denied action '{action}'"),
            Self::InvalidOperation(msg) =>
                write!(f, "Invalid operation: {msg}"),
            Self::CyclicDependency(a, b) =>
                write!(f, "Cyclic dependency: {a} ↔ {b}"),
            Self::VersionConflict { local, remote } =>
                write!(f, "Version conflict: local={local} remote={remote}"),
            Self::AlreadyExists(id) =>
                write!(f, "Component already exists: {id}"),
            Self::InvalidState { current, attempted } =>
                write!(f, "Cannot '{attempted}' in state {current:?}"),
            Self::StorageError(msg) =>
                write!(f, "Storage error: {msg}"),
            Self::ValidationError(msg) =>
                write!(f, "Validation error: {msg}"),
        }
    }
}

pub type PortfolioResult<T> = Result<T, PortfolioError>;

// =============================================================================
// §4 — VECTOR CLOCK  (distributed concurrency / versioning)
// =============================================================================

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct VectorClock {
    /// node_id / actor_id → logical timestamp
    pub clocks: HashMap<String, u64>,
}

impl VectorClock {
    pub fn new() -> Self { Self::default() }

    /// Increment the clock for a given node and return the new value.
    pub fn tick(&mut self, node_id: &str) -> u64 {
        let counter = self.clocks.entry(node_id.to_string()).or_insert(0);
        *counter += 1;
        *counter
    }

    /// Alias of `tick` for callers that use the old name.
    pub fn increment(&mut self, node_id: &str) { self.tick(node_id); }

    /// Merge with another clock, taking the element-wise max.
    pub fn merge(&mut self, other: &VectorClock) {
        for (node, &ts) in &other.clocks {
            let entry = self.clocks.entry(node.clone()).or_insert(0);
            if ts > *entry { *entry = ts; }
        }
    }

    /// Returns `true` if `self` happened-before `other`.
    pub fn happened_before(&self, other: &VectorClock) -> bool {
        self.clocks.iter().all(|(node, &ts)| {
            other.clocks.get(node).copied().unwrap_or(0) >= ts
        }) && self.clocks != other.clocks
    }

    /// Returns `true` if the clocks are concurrent (neither dominates).
    pub fn concurrent_with(&self, other: &VectorClock) -> bool {
        !self.happened_before(other) && !other.happened_before(self)
    }
}

// =============================================================================
// §5 — LIFECYCLE ENUMS: Status · State · Visibility · PermissionTier
// =============================================================================

/// High-level lifecycle status of any component.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentStatus {
    Draft,
    Active,
    Paused,
    Completed,
    Archived,
    Deleted,
    Deprecated,
    UnderReview,
    Rejected,
    Custom(String),
}

/// Operational state — finer-grained than Status.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentState {
    Initializing,
    Configured,
    Running,
    Idle,
    Blocked,
    Failing,
    Recovering,
    Migrating,
    Locked,
    Sealed,
    Custom(String),
}

/// Visibility / publication state.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Visibility {
    Private,
    Protected,
    Public,
    Unlisted,
    DraftOnly,
}

/// Ownership / permission tier hierarchy.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PermissionTier {
    /// Read-only, anonymous
    Viewer,
    /// Can bookmark, save, follow, subscribe
    Subscriber,
    /// Can comment, react, poll
    Contributor,
    /// Can edit content
    Editor,
    /// Can manage settings and members
    Manager,
    /// Full ownership privileges
    Owner,
    /// Super-admin / platform-level control
    Admin,
}

// =============================================================================
// §6 — COMPONENT CATEGORY TAXONOMY
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentCategory {
    Item(ItemCategory),
    Container(ContainerCategory),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemCategory {
    Portfolio,
    Program,
    Project,
    Resource,
    Artifact,
    Asset,
    SubPortfolio,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContainerCategory {
    Binder,
    Book(BookKind),
    Record,
    Folder,
    Registry,
    Archive,
    Custom(String),
}

// ── Book Kinds ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BookKind {
    Notebook,
    ContactBook,
    PlayBook,
    ScheduleBook,
    PlanBook,
    /// Documentation set book
    GuideBook,
    /// Living portfolio item book with rich sub-structure
    ItemBook,
    Custom(String),
}

// =============================================================================
// §7 — ITEMBOOK DATA  (rich per-item dossier container)
// =============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ItemBookData {
    pub dashboard: Option<Dashboard>,
    pub charter: Option<Charter>,
    pub workspace: Option<Workspace>,
    pub catalogue: Option<Catalogue>,
    pub library: Option<Library>,
    pub templates: Vec<Template>,
    pub logs: Vec<ActivityLog>,
    pub metrics: Vec<Metric>,
    pub version: Option<VersionHistory>,
    pub schedule: Option<ItemSchedule>,
    pub directory: Option<Directory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub id: Uuid,
    pub name: String,
    pub widgets: Vec<DashboardWidget>,
    pub layout: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardWidget {
    pub id: Uuid,
    pub widget_type: String,
    pub title: String,
    pub config: serde_json::Value,
    pub position: (u32, u32),
    pub size: (u32, u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Charter {
    pub id: Uuid,
    pub executive_summary: String,
    pub objectives: Vec<String>,
    pub scope: String,
    pub stakeholders: Vec<UserId>,
    pub success_criteria: Vec<String>,
    pub constraints: Vec<String>,
    pub assumptions: Vec<String>,
    pub risks: Vec<Risk>,
    pub version: VersionString,
    pub approved_by: Vec<UserId>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub id: Uuid,
    pub description: String,
    pub severity: RiskSeverity,
    pub probability: f32,
    pub mitigation: String,
    pub owner: Option<UserId>,
    pub status: RiskStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RiskSeverity { Critical, High, Medium, Low, Negligible }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RiskStatus { Open, Mitigated, Accepted, Closed }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: Uuid,
    pub name: String,
    pub files: Vec<FileRef>,
    pub documents: Vec<DocumentRef>,
    pub content_blocks: Vec<ContentBlock>,
    pub connected_items: Vec<ComponentId>,
    pub plugins: Vec<Uuid>,
    pub rooms: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRef {
    pub id: Uuid,
    pub name: String,
    pub mime_type: String,
    pub path: String,
    pub size_bytes: u64,
    pub checksum: String,
    pub version: VersionString,
    pub tags: Vec<String>,
    pub uploaded_by: UserId,
    pub uploaded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRef {
    pub id: Uuid,
    pub title: String,
    pub doc_type: String,
    pub content_ref: String,
    pub version: VersionString,
    pub author: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlock {
    pub id: Uuid,
    pub block_type: String,
    pub content: serde_json::Value,
    pub order: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalogue {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub entries: Vec<CatalogueEntry>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogueEntry {
    pub id: Uuid,
    pub catalogue_id: Uuid,
    pub item_id: ComponentId,
    pub tags: Vec<String>,
    pub searchable_text: String,
    pub metadata: Properties,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub id: Uuid,
    pub name: String,
    pub assets: Vec<LibraryAsset>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryAsset {
    pub id: Uuid,
    pub library_id: Uuid,
    pub name: String,
    pub asset_type: LibraryAssetType,
    pub content_ref: String,
    pub version: VersionString,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LibraryAssetType {
    Template, Plugin, Workflow, Snippet, Schema, Playbook, File, Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: Uuid,
    pub name: String,
    pub template_type: String,
    pub schema: serde_json::Value,
    pub default_values: serde_json::Value,
    pub version: VersionString,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLog {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub actor_id: UserId,
    pub action: ActionKind,
    pub target_id: Option<ComponentId>,
    pub description: String,
    pub metadata: Properties,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub id: Uuid,
    pub name: String,
    pub metric_type: MetricType,
    pub value: f64,
    pub unit: String,
    pub period_start: Option<DateTime<Utc>>,
    pub period_end: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetricType {
    KPI, Counter, Gauge, Histogram, Rate, Ratio, Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionHistory {
    pub current_version: VersionString,
    pub entries: Vec<VersionEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionEntry {
    pub version: VersionString,
    pub author: UserId,
    pub message: String,
    pub diff_ref: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemSchedule {
    pub id: Uuid,
    pub name: String,
    pub milestones: Vec<Milestone>,
    pub timelines: Vec<Timeline>,
    pub roadmap: Vec<RoadmapItem>,
    pub calendar_events: Vec<ScheduledEvent>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub due_date: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: ComponentStatus,
    pub linked_items: Vec<ComponentId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub id: Uuid,
    pub name: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub phases: Vec<TimelinePhase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelinePhase {
    pub id: Uuid,
    pub name: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub deliverables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapItem {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub period: String,
    pub priority: u8,
    pub status: ComponentStatus,
    pub linked_items: Vec<ComponentId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledEvent {
    pub id: Uuid,
    pub title: String,
    pub event_type: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub recurrence: Option<String>,
    pub participants: Vec<UserId>,
    pub location: Option<String>,
    pub virtual_link: Option<String>,
}

// =============================================================================
// §8 — ACTIONS
// =============================================================================

/// Every interaction a user can perform on a component.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionKind {
    // Social
    Like, Comment, Subscribe, Follow, Unfollow, Watch,
    Bookmark, Save, Share, Report, Invite,

    // Content
    Post { visibility: Option<Visibility> },
    Tag { label: Option<String> },
    Mention { target: Option<UserId> },
    Label { value: Option<String> },
    Hashtag { topic: Option<String> },

    // Interaction
    Poll, Survey, Contribute, Join, Leave, Campaign,
    Donate { amount: Option<u64> },
    Invest { amount: Option<u64> },

    // Discovery
    Search, Filter, Index,

    // Ownership / Access
    Own { tier: Option<PermissionTier> },
    Edit,

    // CRUD
    Create, Read, Update, Delete, Archive, Restore, Duplicate,
    Move { target_container: Option<ComponentId> },
    Link { target: Option<ComponentId> },
    Unlink { target: Option<ComponentId> },

    Custom(String),
}

impl ActionKind {
    /// Returns the minimum permission tier required to perform this action.
    pub fn required_permission(&self) -> PermissionTier {
        match self {
            ActionKind::Read | ActionKind::Search | ActionKind::Filter
                => PermissionTier::Viewer,
            ActionKind::Like | ActionKind::Follow | ActionKind::Unfollow
            | ActionKind::Subscribe | ActionKind::Watch
            | ActionKind::Save | ActionKind::Bookmark
                => PermissionTier::Subscriber,
            ActionKind::Comment | ActionKind::Poll | ActionKind::Survey
            | ActionKind::Contribute | ActionKind::Join | ActionKind::Leave
            | ActionKind::Donate { .. } | ActionKind::Invest { .. }
                => PermissionTier::Contributor,
            ActionKind::Edit | ActionKind::Create | ActionKind::Update
            | ActionKind::Tag { .. } | ActionKind::Label { .. }
            | ActionKind::Hashtag { .. } | ActionKind::Mention { .. }
            | ActionKind::Post { .. } | ActionKind::Index
            | ActionKind::Campaign | ActionKind::Invite | ActionKind::Share
                => PermissionTier::Editor,
            ActionKind::Archive | ActionKind::Restore | ActionKind::Duplicate
            | ActionKind::Move { .. } | ActionKind::Link { .. }
            | ActionKind::Unlink { .. }
                => PermissionTier::Manager,
            ActionKind::Delete | ActionKind::Own { .. } | ActionKind::Report
                => PermissionTier::Owner,
            ActionKind::Custom(_)
                => PermissionTier::Contributor,
        }
    }
}

// =============================================================================
// §9 — ANALYTICS
// =============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComponentAnalytics {
    // Traffic
    pub clicks: u64,
    pub click_through_rate: f64,
    pub view_time_seconds: u64,
    pub impressions: u64,

    // Engagement
    pub likes: u64,
    pub comments: u64,
    pub shares: u64,
    pub saves: u64,
    pub bookmarks: u64,
    pub reactions: HashMap<String, u64>,
    pub engagement_rate: f64,
    pub spread: u64,

    // Growth rates (% change per period)
    pub follower_growth_rate: f64,
    pub subscriber_growth_rate: f64,
    pub watcher_growth_rate: f64,

    // Audience counts
    pub followers: u64,
    pub subscribers: u64,
    pub watchers: u64,

    // Benchmarking
    pub benchmark_score: f64,
    pub peer_percentile: Option<f64>,
    pub comparison_kpis: HashMap<String, f64>,

    // Virality
    pub reposts: u64,
    pub referrals: u64,

    // Period
    pub period_start: Option<DateTime<Utc>>,
    pub period_end: Option<DateTime<Utc>>,
    pub last_updated: Option<DateTime<Utc>>,
}

impl ComponentAnalytics {
    pub fn record_click(&mut self) {
        self.clicks += 1;
        self.recalculate_ctr();
    }

    pub fn record_view(&mut self, duration_seconds: u64) {
        self.view_time_seconds += duration_seconds;
        self.impressions += 1;
    }

    pub fn record_engagement(&mut self, action: &ActionKind) {
        match action {
            ActionKind::Like      => self.likes += 1,
            ActionKind::Comment   => self.comments += 1,
            ActionKind::Share     => { self.shares += 1; self.reposts += 1; }
            ActionKind::Save      => self.saves += 1,
            ActionKind::Bookmark  => self.bookmarks += 1,
            ActionKind::Follow    => { self.followers += 1; }
            ActionKind::Subscribe => { self.subscribers += 1; }
            ActionKind::Watch     => { self.watchers += 1; }
            _ => {}
        }
        self.recalculate_engagement_rate();
        self.last_updated = Some(Utc::now());
    }

    fn recalculate_ctr(&mut self) {
        if self.impressions > 0 {
            self.click_through_rate = self.clicks as f64 / self.impressions as f64;
        }
    }

    fn recalculate_engagement_rate(&mut self) {
        let total = self.likes + self.comments + self.shares + self.saves;
        let reach = self.followers.max(1);
        self.engagement_rate = total as f64 / reach as f64;
    }

    pub fn total_engagements(&self) -> u64 {
        self.likes + self.comments + self.shares + self.saves + self.bookmarks
            + self.reactions.values().sum::<u64>()
    }
}

// =============================================================================
// §10 — USER ROLES ON A COMPONENT
// =============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComponentUsers {
    pub owners: HashSet<UserId>,
    pub editors: HashSet<UserId>,
    pub watchers: HashSet<UserId>,
    pub subscribers: HashSet<UserId>,
    pub followers: HashSet<UserId>,
    pub investors: HashSet<UserId>,
    pub donors: HashSet<UserId>,
    /// user_id → tier (authoritative source)
    pub permission_map: HashMap<UserId, PermissionTier>,
}

impl ComponentUsers {
    pub fn add_user(&mut self, user_id: UserId, tier: PermissionTier) {
        self.permission_map.insert(user_id, tier.clone());
        match tier {
            PermissionTier::Owner | PermissionTier::Admin
                => { self.owners.insert(user_id); }
            PermissionTier::Editor | PermissionTier::Manager
                => { self.editors.insert(user_id); }
            PermissionTier::Subscriber
                => { self.subscribers.insert(user_id); }
            PermissionTier::Contributor
                => { self.followers.insert(user_id); }
            PermissionTier::Viewer => {}
        }
    }

    pub fn remove_user(&mut self, user_id: &UserId) {
        self.owners.remove(user_id);
        self.editors.remove(user_id);
        self.watchers.remove(user_id);
        self.subscribers.remove(user_id);
        self.followers.remove(user_id);
        self.investors.remove(user_id);
        self.donors.remove(user_id);
        self.permission_map.remove(user_id);
    }

    pub fn tier_of(&self, user_id: &UserId) -> Option<&PermissionTier> {
        self.permission_map.get(user_id)
    }

    pub fn can(&self, user_id: &UserId, action: &ActionKind) -> bool {
        match self.tier_of(user_id) {
            None       => *action == ActionKind::Read,
            Some(tier) => tier >= &action.required_permission(),
        }
    }
}

// =============================================================================
// §11 — COMPONENT METADATA
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetadata {
    pub id: ComponentId,
    /// List of owning user IDs (first = primary owner)
    pub owners: Vec<UserId>,
    /// Flat tag strings (e.g. "rust", "ai", "q3-2025")
    pub tags: HashSet<String>,
    /// IDs of governance policies attached to this component
    pub policy_ids: Vec<PolicyId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Distributed version tracking
    pub vector_clock: VectorClock,
    /// Arbitrary key-value extension properties
    pub properties: Properties,
    /// Semantic version string
    pub version: VersionString,

    // ── Resource management (from PortfolioSystem_test.rs) ────────────────
    /// Actor that last wrote this component
    pub last_actor: String,
    /// Allocated budget (arbitrary unit — interpreted at application layer)
    pub budget: Option<f64>,
    /// Consumed budget so far
    pub budget_spent: f64,
    /// Generic resource units (person-hours, story-points, etc.)
    pub resource_units: f64,
}

impl ComponentMetadata {
    pub fn new(owner: UserId) -> Self {
        Self {
            id: Uuid::new_v4(),
            owners: vec![owner],
            tags: HashSet::new(),
            policy_ids: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            vector_clock: VectorClock::new(),
            properties: HashMap::new(),
            version: "0.1.0".to_string(),
            last_actor: owner.to_string(),
            budget: None,
            budget_spent: 0.0,
            resource_units: 0.0,
        }
    }

    pub fn touch(&mut self, node_id: &str) {
        self.updated_at = Utc::now();
        self.last_actor = node_id.to_string();
        self.vector_clock.tick(node_id);
    }
}

// =============================================================================
// §12 — COMPONENT DATA  (shared fields for both Item and Container)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentData {
    pub metadata: ComponentMetadata,
    pub category: ComponentCategory,
    pub name: String,
    pub description: String,
    pub status: ComponentStatus,
    pub state: ComponentState,
    pub visibility: Visibility,

    // ── Relations ─────────────────────────────────────────────────────────
    pub children: Vec<ComponentId>,
    pub parents: Vec<ComponentId>,
    pub links: Vec<ComponentId>,
    pub dependents: Vec<ComponentId>,
    pub dependencies: Vec<ComponentId>,

    // ── People ────────────────────────────────────────────────────────────
    pub users: ComponentUsers,

    // ── Behaviour ─────────────────────────────────────────────────────────
    pub allowed_actions: HashSet<String>,
    pub action_history: Vec<ActivityLog>,

    // ── Analytics ─────────────────────────────────────────────────────────
    pub analytics: ComponentAnalytics,

    // ── Governance ────────────────────────────────────────────────────────
    pub risks: Vec<Risk>,
    pub governance_notes: Vec<String>,
    pub compliance_flags: HashMap<String, bool>,

    // ── Hashtags / Topics ─────────────────────────────────────────────────
    pub hashtags: HashSet<String>,
    pub topics: HashSet<String>,

    // ── Extension ─────────────────────────────────────────────────────────
    pub addons: Vec<Uuid>,
    pub plugin_configs: HashMap<Uuid, serde_json::Value>,
}

impl ComponentData {
    pub fn new(
        owner: UserId,
        name: impl Into<String>,
        category: ComponentCategory,
    ) -> Self {
        let mut users = ComponentUsers::default();
        users.add_user(owner, PermissionTier::Owner);

        let mut meta = ComponentMetadata::new(owner);
        // owners is seeded in ComponentMetadata::new; no double-push needed

        Self {
            metadata: meta,
            category,
            name: name.into(),
            description: String::new(),
            status: ComponentStatus::Draft,
            state: ComponentState::Initializing,
            visibility: Visibility::Private,
            children: Vec::new(),
            parents: Vec::new(),
            links: Vec::new(),
            dependents: Vec::new(),
            dependencies: Vec::new(),
            users,
            allowed_actions: HashSet::new(),
            action_history: Vec::new(),
            analytics: ComponentAnalytics::default(),
            risks: Vec::new(),
            governance_notes: Vec::new(),
            compliance_flags: HashMap::new(),
            hashtags: HashSet::new(),
            topics: HashSet::new(),
            addons: Vec::new(),
            plugin_configs: HashMap::new(),
        }
    }

    pub fn id(&self) -> ComponentId { self.metadata.id }

    /// Apply an action by a user, checking permissions first.
    pub fn apply_action(
        &mut self,
        actor: UserId,
        action: ActionKind,
        node_id: &str,
    ) -> PortfolioResult<()> {
        if !self.users.can(&actor, &action) {
            return Err(PortfolioError::PermissionDenied {
                user_id: actor,
                action: format!("{action:?}"),
            });
        }

        self.analytics.record_engagement(&action);

        self.action_history.push(ActivityLog {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            actor_id: actor,
            action: action.clone(),
            target_id: Some(self.id()),
            description: format!("{actor} performed {action:?}"),
            metadata: HashMap::new(),
        });

        self.metadata.touch(node_id);

        match &action {
            ActionKind::Archive => {
                self.status = ComponentStatus::Archived;
                self.state  = ComponentState::Sealed;
            }
            ActionKind::Restore => {
                self.status = ComponentStatus::Active;
                self.state  = ComponentState::Configured;
            }
            ActionKind::Post { visibility } => {
                if let Some(v) = visibility { self.visibility = v.clone(); }
                self.status = ComponentStatus::Active;
            }
            ActionKind::Unfollow => {
                self.analytics.followers = self.analytics.followers.saturating_sub(1);
            }
            ActionKind::Hashtag { topic: Some(t) } => {
                self.hashtags.insert(t.clone());
                self.analytics.spread += 1;
            }
            ActionKind::Label { value: Some(l) } => { self.hashtags.insert(l.clone()); }
            ActionKind::Tag { label: Some(lbl) } => { self.metadata.tags.insert(lbl.clone()); }
            ActionKind::Donate { .. }  => { self.users.donors.insert(actor); }
            ActionKind::Invest { .. }  => { self.users.investors.insert(actor); }
            ActionKind::Join  => { self.users.add_user(actor, PermissionTier::Contributor); }
            ActionKind::Leave => { self.users.remove_user(&actor); }
            _ => {}
        }

        Ok(())
    }

    pub fn add_child(&mut self, child_id: ComponentId) {
        if !self.children.contains(&child_id) { self.children.push(child_id); }
    }

    pub fn remove_child(&mut self, child_id: &ComponentId) {
        self.children.retain(|id| id != child_id);
    }

    /// Shallow dependency add — cycle detection handled at system level.
    pub fn add_dependency(&mut self, dep_id: ComponentId) -> PortfolioResult<()> {
        if dep_id == self.id() {
            return Err(PortfolioError::CyclicDependency(self.id(), dep_id));
        }
        if self.dependents.contains(&dep_id) {
            return Err(PortfolioError::CyclicDependency(self.id(), dep_id));
        }
        if !self.dependencies.contains(&dep_id) { self.dependencies.push(dep_id); }
        Ok(())
    }
}

// =============================================================================
// §13 — ITEM  (leaf portfolio entities)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub data: ComponentData,
    pub item_type: ItemType,
    pub item_book: Option<ItemBookData>,
}

/// Fine-grained item type carrying domain-specific payloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemType {
    Portfolio(PortfolioItemPayload),
    Program(ProgramPayload),
    Project(ProjectPayload),
    Resource(ResourcePayload),
    Artifact(ArtifactPayload),
    Asset(AssetPayload),
    SubPortfolio(SubPortfolioPayload),
    Custom(String),
}

// ── Item Payloads ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioItemPayload {
    pub mission: String,
    pub focus_areas: Vec<String>,
    pub kpis: Vec<Metric>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramPayload {
    pub objective: String,
    pub projects: Vec<ComponentId>,
    pub budget: Option<f64>,
    pub currency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectPayload {
    pub project_type: ProjectType,
    pub methodology: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub sprints: Vec<Sprint>,
    pub backlog: Vec<BacklogItem>,
    pub releases: Vec<Release>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProjectType {
    Organizational, Creative, Technical, Research, AI,
    Software, Media, Marketing, Investment, ContentCreator, DIY,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sprint {
    pub id: Uuid,
    pub name: String,
    pub goal: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub status: ComponentStatus,
    pub items: Vec<Uuid>,
    pub velocity: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacklogItem {
    pub id: Uuid,
    pub item_type: BacklogItemType,
    pub title: String,
    pub description: String,
    pub priority: u8,
    pub estimate: Option<f64>,
    pub status: ComponentStatus,
    pub assignee: Option<UserId>,
    pub sprint_id: Option<Uuid>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BacklogItemType {
    Feature, Requirement, Risk, UseCase, BusinessCase, Innovation,
    Test, Bug, Defect, Blocker, Enhancement, Task,
    Release, Report, Audit, Operation, Strategy, Plan,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub id: Uuid,
    pub version: VersionString,
    pub description: String,
    pub release_date: Option<DateTime<Utc>>,
    pub status: ComponentStatus,
    pub items: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePayload {
    pub resource_type: ResourceType,
    pub skills: Vec<String>,
    pub availability: f64,
    pub hourly_rate: Option<f64>,
    pub currency: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResourceType { Human, Machine, Service, License, Custom(String) }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactPayload {
    pub artifact_type: String,
    pub file_refs: Vec<FileRef>,
    pub produced_by: Vec<ComponentId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPayload {
    pub asset_type: AssetType,
    pub valuation: Option<f64>,
    pub currency: Option<String>,
    pub acquired_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AssetType {
    IntellectualProperty, Financial, Physical, Digital, Creative, Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubPortfolioPayload {
    pub parent_portfolio_id: Option<ComponentId>,
    pub scope: String,
    pub strategic_goals: Vec<String>,
}

// =============================================================================
// §14 — CONTAINER  (organising / grouping entities)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    pub data: ComponentData,
    pub container_type: ContainerType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContainerType {
    Binder(BinderData),
    Book(Book),
    Record(RecordData),
    Folder(FolderData),
    Registry(RegistryData),
    Archive(ArchiveData),
    Custom(String),
}

// ── Container Payloads ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinderData {
    pub description: String,
    pub logic_tags: Vec<String>,
    pub source_items: Vec<ComponentId>,
    pub dashboards: Vec<Dashboard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub kind: BookKind,
    pub contents: BookContents,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BookContents {
    Notebook(NotebookData),
    ContactBook(ContactBookData),
    PlayBook(PlayBookData),
    ScheduleBook(ScheduleBookData),
    PlanBook(PlanBookData),
    GuideBook(GuideBookData),
    ItemBook(ItemBookData),
    Custom(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookData {
    pub pages: Vec<NotePage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotePage {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactBookData {
    pub contacts: Vec<Contact>,
    pub groups: Vec<ContactGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: Uuid,
    pub user_id: Option<UserId>,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub role: String,
    pub organisation: Option<String>,
    pub tags: Vec<String>,
    pub notes: String,
    pub social_links: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactGroup {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub contact_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayBookData {
    pub description: String,
    pub plays: Vec<Play>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Play {
    pub id: Uuid,
    pub name: String,
    pub trigger: String,
    pub steps: Vec<PlayStep>,
    pub outcome: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayStep {
    pub order: u32,
    pub description: String,
    pub assignee: Option<UserId>,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleBookData {
    pub schedules: Vec<ItemSchedule>,
    pub time_blocks: Vec<TimeBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBlock {
    pub id: Uuid,
    pub title: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub block_type: String,
    pub linked_items: Vec<ComponentId>,
    pub recurrence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanBookData {
    pub goals: Vec<Goal>,
    pub strategies: Vec<Strategy>,
    pub initiatives: Vec<Initiative>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub target_date: Option<DateTime<Utc>>,
    pub success_metrics: Vec<Metric>,
    pub status: ComponentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub pillars: Vec<String>,
    pub linked_goals: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Initiative {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub linked_strategy: Option<Uuid>,
    pub linked_projects: Vec<ComponentId>,
    pub status: ComponentStatus,
    pub owner: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuideBookData {
    pub sections: Vec<GuideSection>,
    pub version: VersionString,
    pub audience: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuideSection {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub sub_sections: Vec<GuideSection>,
    pub order: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordData {
    pub record_type: String,
    pub schema: serde_json::Value,
    pub fields: HashMap<String, serde_json::Value>,
    pub linked_item: Option<ComponentId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderData {
    pub path: String,
    pub files: Vec<FileRef>,
    pub sub_folders: Vec<ComponentId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryData {
    pub registry_type: String,
    pub schema: serde_json::Value,
    pub entries: Vec<RegistryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub id: Uuid,
    pub key: String,
    pub value: serde_json::Value,
    pub tags: Vec<String>,
    pub linked_component: Option<ComponentId>,
    pub registered_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveData {
    pub archived_items: Vec<ArchiveEntry>,
    pub compression: Option<String>,
    pub encryption: bool,
    pub retention_policy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveEntry {
    pub id: Uuid,
    pub original_id: ComponentId,
    pub snapshot: serde_json::Value,
    pub version: VersionString,
    pub archived_by: UserId,
    pub archived_at: DateTime<Utc>,
    pub restore_key: Option<String>,
}

// =============================================================================
// §15 — STRUCTURAL PRIMITIVES: Group · Collection · List · Schedule · Directory
// =============================================================================

/// Linked components (sibling / peer set) — bidirectional links.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub members: Vec<ComponentId>,
    pub group_type: String,
    pub owner: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Group {
    pub fn new(name: impl Into<String>, owner: UserId) -> Self {
        Self {
            id: Uuid::new_v4(), name: name.into(), description: String::new(),
            members: Vec::new(), group_type: "default".into(), owner,
            created_at: Utc::now(), updated_at: Utc::now(),
        }
    }
    pub fn add(&mut self, id: ComponentId) {
        if !self.members.contains(&id) { self.members.push(id); }
        self.updated_at = Utc::now();
    }
    pub fn remove(&mut self, id: &ComponentId) {
        self.members.retain(|m| m != id);
        self.updated_at = Utc::now();
    }
}

/// Unordered set of component references.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub items: HashSet<ComponentId>,
    pub tags: HashSet<String>,
    pub owner: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Collection {
    pub fn new(name: impl Into<String>, owner: UserId) -> Self {
        Self {
            id: Uuid::new_v4(), name: name.into(), description: String::new(),
            items: HashSet::new(), tags: HashSet::new(), owner,
            created_at: Utc::now(), updated_at: Utc::now(),
        }
    }
    pub fn insert(&mut self, id: ComponentId)  { self.items.insert(id); }
    pub fn remove(&mut self, id: &ComponentId) { self.items.remove(id); }
    pub fn contains(&self, id: &ComponentId) -> bool { self.items.contains(id) }
    pub fn len(&self) -> usize { self.items.len() }
    pub fn is_empty(&self) -> bool { self.items.is_empty() }
}

/// Ordered, indexed list of component references.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct List {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub items: Vec<ComponentId>,
    pub owner: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl List {
    pub fn new(name: impl Into<String>, owner: UserId) -> Self {
        Self {
            id: Uuid::new_v4(), name: name.into(), description: String::new(),
            items: Vec::new(), owner, created_at: Utc::now(), updated_at: Utc::now(),
        }
    }
    pub fn push(&mut self, id: ComponentId) { self.items.push(id); }
    pub fn insert_at(&mut self, index: usize, id: ComponentId) {
        let idx = index.min(self.items.len());
        self.items.insert(idx, id);
    }
    pub fn remove_at(&mut self, index: usize) -> Option<ComponentId> {
        if index < self.items.len() { Some(self.items.remove(index)) } else { None }
    }
    pub fn reorder(&mut self, from: usize, to: usize) {
        if from < self.items.len() && to < self.items.len() {
            let item = self.items.remove(from);
            self.items.insert(to, item);
        }
    }
}

/// Causal / time-ordered sequence of scheduled component references.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub entries: Vec<ScheduleEntry>,
    pub owner: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleEntry {
    pub id: Uuid,
    pub component_id: ComponentId,
    pub scheduled_at: DateTime<Utc>,
    pub duration_minutes: Option<u32>,
    pub recurrence: Option<String>,
    pub notes: String,
    pub order: u32,
}

impl Schedule {
    pub fn new(name: impl Into<String>, owner: UserId) -> Self {
        Self {
            id: Uuid::new_v4(), name: name.into(), description: String::new(),
            entries: Vec::new(), owner, created_at: Utc::now(), updated_at: Utc::now(),
        }
    }
    pub fn add_entry(&mut self, component_id: ComponentId, scheduled_at: DateTime<Utc>) {
        let order = self.entries.len() as u32;
        self.entries.push(ScheduleEntry {
            id: Uuid::new_v4(), component_id, scheduled_at,
            duration_minutes: None, recurrence: None, notes: String::new(), order,
        });
        self.entries.sort_by_key(|e| e.scheduled_at);
        self.updated_at = Utc::now();
    }
    pub fn upcoming(&self, from: DateTime<Utc>) -> Vec<&ScheduleEntry> {
        self.entries.iter().filter(|e| e.scheduled_at >= from).collect()
    }
}

/// Spatial / hierarchical collection — like a filesystem directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Directory {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub path: String,
    pub entries: Vec<DirectoryEntry>,
    pub sub_directories: Vec<Directory>,
    pub owner: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryEntry {
    pub id: Uuid,
    pub component_id: ComponentId,
    pub name: String,
    pub entry_type: String,
    pub path: String,
    pub order: u32,
    pub created_at: DateTime<Utc>,
}

impl Directory {
    pub fn new(name: impl Into<String>, path: impl Into<String>, owner: UserId) -> Self {
        Self {
            id: Uuid::new_v4(), name: name.into(), description: String::new(),
            path: path.into(), entries: Vec::new(), sub_directories: Vec::new(),
            owner, created_at: Utc::now(), updated_at: Utc::now(),
        }
    }
    pub fn add_entry(&mut self, component_id: ComponentId, name: impl Into<String>, entry_type: &str) {
        let n = name.into();
        let path = format!("{}/{}", self.path.trim_end_matches('/'), n);
        let order = self.entries.len() as u32;
        self.entries.push(DirectoryEntry {
            id: Uuid::new_v4(), component_id, name: n, entry_type: entry_type.to_string(),
            path, order, created_at: Utc::now(),
        });
        self.updated_at = Utc::now();
    }
    pub fn find(&self, component_id: &ComponentId) -> Option<&DirectoryEntry> {
        self.entries.iter().find(|e| &e.component_id == component_id)
    }
    pub fn add_subdirectory(&mut self, sub: Directory) {
        self.sub_directories.push(sub);
        self.updated_at = Utc::now();
    }
    pub fn resolve_path(&self, path: &str) -> Option<&DirectoryEntry> {
        let parts: Vec<&str> = path.trim_start_matches('/').splitn(2, '/').collect();
        match parts.as_slice() {
            [leaf] => self.entries.iter().find(|e| e.name == *leaf),
            [dir, rest] => self.sub_directories.iter()
                .find(|d| d.name == *dir)
                .and_then(|d| d.resolve_path(rest)),
            _ => None,
        }
    }
}

// =============================================================================
// §16 — UNIFIED COMPONENT ENUM
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Component {
    Item(Item),
    Container(Container),
}

impl Component {
    pub fn data(&self) -> &ComponentData {
        match self { Component::Item(i) => &i.data, Component::Container(c) => &c.data }
    }
    pub fn data_mut(&mut self) -> &mut ComponentData {
        match self { Component::Item(i) => &mut i.data, Component::Container(c) => &mut c.data }
    }
    pub fn id(&self)   -> ComponentId { self.data().id() }
    pub fn name(&self) -> &str        { &self.data().name }
    pub fn is_item(&self)      -> bool { matches!(self, Component::Item(_)) }
    pub fn is_container(&self) -> bool { matches!(self, Component::Container(_)) }
    pub fn as_item(&self)      -> Option<&Item>      { if let Component::Item(i)      = self { Some(i) } else { None } }
    pub fn as_container(&self) -> Option<&Container> { if let Component::Container(c) = self { Some(c) } else { None } }
}

// =============================================================================
// §17 — GRAPH LAYER  (typed DAG over component IDs)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GraphEdgeKind {
    /// Strict parent→child ownership hierarchy
    Hierarchy,
    /// `from` depends on `to` before it can proceed
    Dependency,
    /// Peer association (non-hierarchical, non-dependency)
    Link,
    /// Container membership (`from` contains `to`)
    Contains,
    /// Cross-portfolio federation bridge
    Federation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub from: String,
    pub to: String,
    pub kind: GraphEdgeKind,
    pub label: Option<String>,
    pub created_at: u64,
    pub properties: HashMap<String, String>,
}

impl GraphEdge {
    pub fn new(
        id: impl Into<String>,
        from: impl Into<String>,
        to: impl Into<String>,
        kind: GraphEdgeKind,
    ) -> Self {
        Self {
            id: id.into(), from: from.into(), to: to.into(), kind,
            label: None, created_at: now_epoch(), properties: HashMap::new(),
        }
    }
}

/// Read-only view over a component graph.
pub struct GraphView<'a> {
    components: &'a HashMap<ComponentId, Component>,
    edges: &'a Vec<GraphEdge>,
}

impl<'a> GraphView<'a> {
    pub fn new(components: &'a HashMap<ComponentId, Component>, edges: &'a Vec<GraphEdge>) -> Self {
        Self { components, edges }
    }

    pub fn outgoing(&self, id: &str) -> Vec<&GraphEdge> {
        self.edges.iter().filter(|e| e.from == id).collect()
    }

    pub fn incoming(&self, id: &str) -> Vec<&GraphEdge> {
        self.edges.iter().filter(|e| e.to == id).collect()
    }

    /// Transitive reachability from `start` following edges of `kind`.
    pub fn reachable(&self, start: &str, kind: &GraphEdgeKind) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(start.to_string());
        while let Some(node) = queue.pop_front() {
            if visited.contains(&node) { continue; }
            visited.insert(node.clone());
            for edge in self.edges.iter().filter(|e| e.from == node && &e.kind == kind) {
                queue.push_back(edge.to.clone());
            }
        }
        visited.into_iter().filter(|id| id != start).collect()
    }

    /// Kahn's algorithm topological sort over `kind` edges.
    /// Returns `Err` if a cycle is detected.
    pub fn topological_sort(&self, kind: &GraphEdgeKind) -> Result<Vec<String>, String> {
        let relevant: Vec<&GraphEdge> = self.edges.iter().filter(|e| &e.kind == kind).collect();
        let mut in_degree: HashMap<String, usize> = self.components.keys()
            .map(|k| (k.to_string(), 0))
            .collect();
        for e in &relevant { *in_degree.entry(e.to.clone()).or_insert(0) += 1; }

        let mut queue: VecDeque<String> = in_degree.iter()
            .filter(|(_, &d)| d == 0)
            .map(|(k, _)| k.clone())
            .collect();
        let mut sorted = Vec::new();
        while let Some(node) = queue.pop_front() {
            sorted.push(node.clone());
            for e in relevant.iter().filter(|e| e.from == node) {
                let deg = in_degree.entry(e.to.clone()).or_default();
                *deg = deg.saturating_sub(1);
                if *deg == 0 { queue.push_back(e.to.clone()); }
            }
        }
        if sorted.len() == self.components.len() { Ok(sorted) }
        else { Err("Cycle detected in dependency graph".to_string()) }
    }

    /// Check whether adding `from→to` of `kind` would introduce a cycle.
    pub fn would_cycle(&self, from: &str, to: &str, kind: &GraphEdgeKind) -> bool {
        self.reachable(to, kind).contains(&from.to_string())
    }
}

// =============================================================================
// §18 — EVENT SOURCING
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortfolioEventKind {
    // Components
    ComponentCreated, ComponentUpdated, ComponentRemoved, ComponentStatusChanged,
    // Graph
    EdgeAdded, EdgeRemoved, DependencyAdded, DependencyRemoved,
    HierarchyLinked, HierarchyUnlinked, MemberAdded, MemberRemoved,
    // Snapshots
    SnapshotSaved, CheckpointCreated, StateRestored,
    // Governance
    PolicyAttached, PolicyDetached,
    ApprovalRequested, ApprovalGranted, ApprovalRejected,
    ResourceAllocated, ResourceConsumed,
    /// Backward-compat alias
    BudgetAllocated,
    /// Backward-compat alias
    BudgetSpent,
    // Federation / CRDT
    PeerRegistered, PeerRemoved, CrdtMergeApplied, EventStreamPublished,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioEvent {
    pub id: String,
    pub sequence: u64,
    pub timestamp: u64,
    pub actor_id: String,
    pub kind: PortfolioEventKind,
    pub payload: HashMap<String, String>,
}

/// Append-only ordered event log.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventLog {
    events: Vec<PortfolioEvent>,
    next_sequence: u64,
}

impl EventLog {
    pub fn new() -> Self { Self::default() }

    pub fn append(
        &mut self,
        actor_id: impl Into<String>,
        kind: PortfolioEventKind,
        payload: HashMap<String, String>,
    ) -> &PortfolioEvent {
        let seq = self.next_sequence;
        self.next_sequence += 1;
        self.events.push(PortfolioEvent {
            id: format!("evt-{seq:06}"), sequence: seq,
            timestamp: now_epoch(), actor_id: actor_id.into(), kind, payload,
        });
        self.events.last().unwrap()
    }

    pub fn all(&self) -> &[PortfolioEvent] { &self.events }
    pub fn up_to(&self, until: u64) -> impl Iterator<Item = &PortfolioEvent> {
        self.events.iter().filter(move |e| e.timestamp <= until)
    }
    pub fn len(&self) -> usize { self.events.len() }
    pub fn is_empty(&self) -> bool { self.events.is_empty() }
}

// =============================================================================
// §19 — SNAPSHOT & CHECKPOINT
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioSnapshot {
    pub snapshot_id: String,
    pub label: Option<String>,
    pub created_at: u64,
    pub components: HashMap<ComponentId, Component>,
    pub edges: Vec<GraphEdge>,
    pub active_portfolio_id: Option<ComponentId>,
    pub next_id: u64,
    pub next_edge_id: u64,
    pub component_count: usize,
    pub edge_count: usize,
    pub event_count: usize,
    pub last_checkpoint_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioCheckpoint {
    pub id: String,
    pub label: String,
    pub snapshot_id: String,
    pub created_at: u64,
    pub note: Option<String>,
}

// =============================================================================
// §20 — CRDT: LWW-Field · OR-Set · CrdtOperation · CrdtLog
// =============================================================================

/// Last-Write-Wins scalar field.  Ties broken by lexicographic actor_id.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LwwField {
    pub value: String,
    pub timestamp: u64,
    pub actor_id: String,
}

impl LwwField {
    pub fn new(value: impl Into<String>, actor_id: impl Into<String>) -> Self {
        Self { value: value.into(), timestamp: now_epoch(), actor_id: actor_id.into() }
    }
    pub fn merge(&mut self, incoming: LwwField) {
        if incoming.timestamp > self.timestamp
            || (incoming.timestamp == self.timestamp && incoming.actor_id > self.actor_id)
        {
            *self = incoming;
        }
    }
}

/// Observed-Remove Set.  Tracks (element, unique_tag) pairs; removals tombstone tags.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrSet {
    pub observed: HashMap<String, HashSet<String>>,
    pub removed_tags: HashSet<String>,
}

impl OrSet {
    pub fn add(&mut self, element: impl Into<String>, tag: impl Into<String>) {
        self.observed.entry(element.into()).or_default().insert(tag.into());
    }
    pub fn remove(&mut self, element: &str) {
        if let Some(tags) = self.observed.remove(element) {
            self.removed_tags.extend(tags);
        }
    }
    pub fn contains(&self, element: &str) -> bool {
        self.observed.get(element).map_or(false, |t| !t.is_empty())
    }
    pub fn members(&self) -> Vec<&str> { self.observed.keys().map(String::as_str).collect() }
    pub fn merge(&mut self, remote: &OrSet) {
        for tombstone in &remote.removed_tags {
            for tags in self.observed.values_mut() { tags.remove(tombstone); }
        }
        self.removed_tags.extend(remote.removed_tags.iter().cloned());
        for (elem, tags) in &remote.observed {
            let live: HashSet<String> = tags.iter()
                .filter(|t| !self.removed_tags.contains(*t))
                .cloned()
                .collect();
            if !live.is_empty() { self.observed.entry(elem.clone()).or_default().extend(live); }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrdtOperation {
    SetField { component_id: String, field: String, value: String, timestamp: u64, actor: String },
    AddToSet { component_id: String, set_name: String, element: String, tag: String, timestamp: u64, actor: String },
    RemoveFromSet { component_id: String, set_name: String, element: String, timestamp: u64, actor: String },
    AddEdge    { edge: GraphEdge, timestamp: u64, actor: String },
    RemoveEdge { edge_id: String, timestamp: u64, actor: String },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CrdtLog {
    pub operations: Vec<CrdtOperation>,
    pub clock: VectorClock,
}

impl CrdtLog {
    pub fn new() -> Self { Self::default() }

    pub fn append(&mut self, actor: &str, op: CrdtOperation) {
        self.clock.tick(actor);
        self.operations.push(op);
    }

    pub fn merge(&mut self, remote: &CrdtLog) {
        let existing: HashSet<String> = self.operations.iter()
            .filter_map(|o| serde_json::to_string(o).ok())
            .collect();
        for op in &remote.operations {
            if let Ok(s) = serde_json::to_string(op) {
                if !existing.contains(&s) { self.operations.push(op.clone()); }
            }
        }
        self.clock.merge(&remote.clock);
    }
}

// =============================================================================
// §21 — GOVERNANCE: ResourceKind · ResourceAllocation · Policy · Approval
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceKind {
    Budget, PersonHours, StoryPoints, ComputeUnits, StorageGiB, Custom(String),
}

impl ResourceKind {
    pub fn unit_label(&self) -> &str {
        match self {
            ResourceKind::Budget       => "currency",
            ResourceKind::PersonHours  => "person-hours",
            ResourceKind::StoryPoints  => "story-points",
            ResourceKind::ComputeUnits => "compute-units",
            ResourceKind::StorageGiB   => "GiB",
            ResourceKind::Custom(u)    => u.as_str(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub component_id: String,
    pub kind: ResourceKind,
    pub total: f64,
    pub allocated: f64,
    pub consumed: f64,
    pub denomination: String,
    pub period: Option<String>,
}

impl ResourceAllocation {
    pub fn remaining(&self) -> f64 { self.total - self.consumed }
    pub fn utilisation_pct(&self) -> f64 {
        if self.total == 0.0 { 0.0 } else { (self.consumed / self.total) * 100.0 }
    }
    pub fn is_overrun(&self) -> bool { self.consumed > self.total }
    pub fn headroom(&self) -> f64 { self.allocated - self.consumed }
}

/// Backward-compatibility alias.
pub type BudgetAllocation = ResourceAllocation;

/// Evaluation context passed to a policy engine.
pub struct PolicyContext<'a> {
    pub component: &'a Component,
    pub event_kind: &'a PortfolioEventKind,
    pub actor_id: &'a str,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow,
    Deny(String),
    RequireApproval(String),
}

/// Trait that governance policy engines must implement.
pub trait PolicyEngine: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn evaluate(&self, ctx: &PolicyContext<'_>) -> PolicyDecision;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalStatus { Pending, Approved, Rejected, Expired }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: String,
    pub component_id: ComponentId,
    pub requested_by: String,
    pub reason: String,
    pub status: ApprovalStatus,
    pub created_at: u64,
    pub resolved_at: Option<u64>,
    pub resolver: Option<String>,
    pub notes: Option<String>,
}

// =============================================================================
// §22 — PLUGIN TRAIT
// =============================================================================

pub trait PortfolioPlugin: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;

    fn on_component_created(&self, _component: &Component) {}
    fn on_component_updated(&self, _component: &Component) {}
    fn on_component_removed(&self, _component_id: &ComponentId) {}
    fn on_edge_added(&self, _edge: &GraphEdge) {}
    fn on_edge_removed(&self, _edge_id: &str) {}
    fn on_event(&self, _event: &PortfolioEvent) {}
    fn on_snapshot_saved(&self, _snapshot: &PortfolioSnapshot) {}
    fn on_crdt_merge(&self, _ops: &[CrdtOperation]) {}
}

// =============================================================================
// §23 — SEARCH / FILTER
// =============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: Option<String>,
    pub tags: Vec<String>,
    pub hashtags: Vec<String>,
    pub categories: Vec<ComponentCategory>,
    pub statuses: Vec<ComponentStatus>,
    pub owner: Option<UserId>,
    pub visibility: Option<Visibility>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub component_id: ComponentId,
    pub name: String,
    pub category: ComponentCategory,
    pub status: ComponentStatus,
    pub score: f64,
    pub matched_fields: Vec<String>,
}

// PQL (Portfolio Query Language)
#[derive(Debug, Clone, Default)]
pub struct PortfolioQuery {
    pub categories: Option<Vec<ComponentCategory>>,
    pub statuses: Option<Vec<ComponentStatus>>,
    pub tags: Option<Vec<String>>,
    pub name_contains: Option<String>,
    pub owner: Option<UserId>,
    pub has_dependencies: Option<bool>,
    pub has_children: Option<bool>,
    pub property_filter: Option<(String, String)>,
}

// =============================================================================
// §24 — VERSION HELPERS
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum VersionPart { Major, Minor, Patch }

fn increment_version(version: &str, part: VersionPart) -> String {
    let parts: Vec<u64> = version.split('.').filter_map(|s| s.parse().ok()).collect();
    let (mut major, mut minor, mut patch) = match parts.as_slice() {
        [ma, mi, pa, ..] => (*ma, *mi, *pa),
        [ma, mi]         => (*ma, *mi, 0),
        [ma]             => (*ma, 0, 0),
        []               => (0, 1, 0),
    };
    match part {
        VersionPart::Major => { major += 1; minor = 0; patch = 0; }
        VersionPart::Minor => { minor += 1; patch = 0; }
        VersionPart::Patch => { patch += 1; }
    }
    format!("{major}.{minor}.{patch}")
}

// =============================================================================
// §25 — PORTFOLIO SYSTEM  (central registry and orchestrator)
// =============================================================================

pub struct PortfolioSystem {
    // ── Core state ──────────────────────────────────────────────────────────
    components:  HashMap<ComponentId, Component>,
    edges:       Vec<GraphEdge>,
    active_portfolio_id: Option<ComponentId>,

    // ── Structural primitives ───────────────────────────────────────────────
    groups:      HashMap<Uuid, Group>,
    collections: HashMap<Uuid, Collection>,
    lists:       HashMap<Uuid, List>,
    schedules:   HashMap<Uuid, Schedule>,
    directories: HashMap<Uuid, Directory>,

    // ── Identity / sequencing ───────────────────────────────────────────────
    node_id:       String,
    next_edge_id:  u64,
    next_snap_id:  u64,
    next_ckpt_id:  u64,
    next_req_id:   u64,

    // ── History / time-travel ───────────────────────────────────────────────
    event_log:   EventLog,
    global_log:  Vec<ActivityLog>,
    snapshots:   HashMap<String, PortfolioSnapshot>,
    checkpoints: Vec<PortfolioCheckpoint>,

    // ── Distributed ─────────────────────────────────────────────────────────
    crdt: CrdtLog,

    // ── Governance ──────────────────────────────────────────────────────────
    policy_engines:      Vec<Arc<dyn PolicyEngine>>,
    approval_requests:   Vec<ApprovalRequest>,
    resource_allocations: HashMap<ComponentId, ResourceAllocation>,

    // ── Plugins ─────────────────────────────────────────────────────────────
    plugins: Vec<Arc<dyn PortfolioPlugin>>,
}

impl fmt::Debug for PortfolioSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PortfolioSystem")
            .field("components", &self.components.len())
            .field("edges", &self.edges.len())
            .field("node_id", &self.node_id)
            .field("active_portfolio_id", &self.active_portfolio_id)
            .field("event_log_len", &self.event_log.len())
            .field("snapshots", &self.snapshots.len())
            .finish()
    }
}

// ── Construction ─────────────────────────────────────────────────────────────

impl PortfolioSystem {
    pub fn new(node_id: impl Into<String>) -> Self {
        Self {
            components:  HashMap::new(),
            edges:       Vec::new(),
            active_portfolio_id: None,
            groups:      HashMap::new(),
            collections: HashMap::new(),
            lists:       HashMap::new(),
            schedules:   HashMap::new(),
            directories: HashMap::new(),
            node_id:     node_id.into(),
            next_edge_id: 1,
            next_snap_id: 1,
            next_ckpt_id: 1,
            next_req_id:  1,
            event_log:   EventLog::new(),
            global_log:  Vec::new(),
            snapshots:   HashMap::new(),
            checkpoints: Vec::new(),
            crdt:        CrdtLog::new(),
            policy_engines: Vec::new(),
            approval_requests: Vec::new(),
            resource_allocations: HashMap::new(),
            plugins: Vec::new(),
        }
    }

    fn next_edge_id_str(&mut self) -> String {
        let id = format!("edge-{:06}", self.next_edge_id);
        self.next_edge_id += 1;
        id
    }

    fn emit(
        &mut self,
        kind: PortfolioEventKind,
        payload: impl IntoIterator<Item = (&'static str, String)>,
    ) {
        let map: HashMap<String, String> =
            payload.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
        let actor = self.node_id.clone();
        let evt = self.event_log.append(&actor, kind, map).clone();
        for plugin in &self.plugins { plugin.on_event(&evt); }
    }

    fn evaluate_policies(
        &self,
        component: &Component,
        event_kind: &PortfolioEventKind,
    ) -> PolicyDecision {
        for engine in &self.policy_engines {
            let ctx = PolicyContext {
                component,
                event_kind,
                actor_id: &self.node_id,
                properties: HashMap::new(),
            };
            match engine.evaluate(&ctx) {
                PolicyDecision::Allow => continue,
                decision => return decision,
            }
        }
        PolicyDecision::Allow
    }

    fn log_global(
        &mut self,
        actor: UserId,
        action: ActionKind,
        target: Option<ComponentId>,
        description: &str,
    ) {
        self.global_log.push(ActivityLog {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            actor_id: actor,
            action,
            target_id: target,
            description: description.to_string(),
            metadata: HashMap::new(),
        });
    }

    pub fn global_log(&self) -> &[ActivityLog] { &self.global_log }
}

// ── Component CRUD ────────────────────────────────────────────────────────────

impl PortfolioSystem {
    /// Create and register a new Item component.
    pub fn create_item(
        &mut self,
        owner: UserId,
        name: impl Into<String>,
        item_type: ItemType,
    ) -> PortfolioResult<ComponentId> {
        let category = match &item_type {
            ItemType::Portfolio(_)     => ComponentCategory::Item(ItemCategory::Portfolio),
            ItemType::Program(_)       => ComponentCategory::Item(ItemCategory::Program),
            ItemType::Project(_)       => ComponentCategory::Item(ItemCategory::Project),
            ItemType::Resource(_)      => ComponentCategory::Item(ItemCategory::Resource),
            ItemType::Artifact(_)      => ComponentCategory::Item(ItemCategory::Artifact),
            ItemType::Asset(_)         => ComponentCategory::Item(ItemCategory::Asset),
            ItemType::SubPortfolio(_)  => ComponentCategory::Item(ItemCategory::SubPortfolio),
            ItemType::Custom(s)        => ComponentCategory::Item(ItemCategory::Custom(s.clone())),
        };

        let data = ComponentData::new(owner, name, category);
        let id   = data.id();
        let comp = Component::Item(Item { data, item_type, item_book: None });

        // Policy gate
        match self.evaluate_policies(&comp, &PortfolioEventKind::ComponentCreated) {
            PolicyDecision::Allow => {}
            PolicyDecision::Deny(reason) =>
                return Err(PortfolioError::InvalidOperation(format!("Denied: {reason}"))),
            PolicyDecision::RequireApproval(reason) =>
                return Err(PortfolioError::InvalidOperation(format!("Requires approval: {reason}"))),
        }

        for plugin in &self.plugins { plugin.on_component_created(&comp); }
        self.components.insert(id, comp);
        self.log_global(owner, ActionKind::Create, Some(id), "Item created");
        self.emit(PortfolioEventKind::ComponentCreated,
            [("id", id.to_string()), ("owner", owner.to_string())]);
        Ok(id)
    }

    /// Create and register a new Container component.
    pub fn create_container(
        &mut self,
        owner: UserId,
        name: impl Into<String>,
        container_type: ContainerType,
    ) -> PortfolioResult<ComponentId> {
        let category = match &container_type {
            ContainerType::Binder(_)   => ComponentCategory::Container(ContainerCategory::Binder),
            ContainerType::Book(b)     => ComponentCategory::Container(ContainerCategory::Book(b.kind.clone())),
            ContainerType::Record(_)   => ComponentCategory::Container(ContainerCategory::Record),
            ContainerType::Folder(_)   => ComponentCategory::Container(ContainerCategory::Folder),
            ContainerType::Registry(_) => ComponentCategory::Container(ContainerCategory::Registry),
            ContainerType::Archive(_)  => ComponentCategory::Container(ContainerCategory::Archive),
            ContainerType::Custom(s)   => ComponentCategory::Container(ContainerCategory::Custom(s.clone())),
        };

        let data = ComponentData::new(owner, name, category);
        let id   = data.id();
        let comp = Component::Container(Container { data, container_type });

        match self.evaluate_policies(&comp, &PortfolioEventKind::ComponentCreated) {
            PolicyDecision::Allow => {}
            PolicyDecision::Deny(reason) =>
                return Err(PortfolioError::InvalidOperation(format!("Denied: {reason}"))),
            PolicyDecision::RequireApproval(reason) =>
                return Err(PortfolioError::InvalidOperation(format!("Requires approval: {reason}"))),
        }

        for plugin in &self.plugins { plugin.on_component_created(&comp); }
        self.components.insert(id, comp);
        self.log_global(owner, ActionKind::Create, Some(id), "Container created");
        self.emit(PortfolioEventKind::ComponentCreated,
            [("id", id.to_string()), ("owner", owner.to_string())]);
        Ok(id)
    }

    pub fn read(&self, id: &ComponentId) -> PortfolioResult<&Component> {
        self.components.get(id).ok_or(PortfolioError::NotFound(*id))
    }

    pub fn read_mut(&mut self, id: &ComponentId) -> PortfolioResult<&mut Component> {
        self.components.get_mut(id).ok_or(PortfolioError::NotFound(*id))
    }

    pub fn update_info(
        &mut self,
        actor: UserId,
        id: &ComponentId,
        name: Option<String>,
        description: Option<String>,
    ) -> PortfolioResult<()> {
        let node = self.node_id.clone();
        let c = self.read_mut(id)?;
        let data = c.data_mut();

        if !data.users.can(&actor, &ActionKind::Edit) {
            return Err(PortfolioError::PermissionDenied { user_id: actor, action: "Edit".into() });
        }
        if let Some(n) = name        { data.name        = n; }
        if let Some(d) = description { data.description = d; }
        data.metadata.touch(&node);
        Ok(())
    }

    pub fn delete(
        &mut self,
        actor: UserId,
        id: &ComponentId,
        hard_delete: bool,
    ) -> PortfolioResult<()> {
        let node = self.node_id.clone();
        {
            let c = self.read_mut(id)?;
            let data = c.data_mut();
            if !data.users.can(&actor, &ActionKind::Delete) {
                return Err(PortfolioError::PermissionDenied {
                    user_id: actor, action: "Delete".into(),
                });
            }
            data.status = ComponentStatus::Deleted;
            data.metadata.touch(&node);
        }
        if hard_delete {
            // Remove from all edges
            let id_str = id.to_string();
            self.edges.retain(|e| e.from != id_str && e.to != id_str);
            // Remove from parent/child/link sets
            for comp in self.components.values_mut() {
                comp.data_mut().children.retain(|c| c != id);
                comp.data_mut().parents.retain(|p| p != id);
                comp.data_mut().links.retain(|l| l != id);
                comp.data_mut().dependencies.retain(|d| d != id);
                comp.data_mut().dependents.retain(|d| d != id);
            }
            self.components.remove(id);
        }
        self.log_global(actor, ActionKind::Delete, Some(*id), "Component deleted");
        self.emit(PortfolioEventKind::ComponentRemoved, [("id", id.to_string())]);
        Ok(())
    }

    pub fn set_active_portfolio(&mut self, id: ComponentId) -> PortfolioResult<()> {
        self.read(&id)?;
        self.active_portfolio_id = Some(id);
        Ok(())
    }
}

// ── Actions ───────────────────────────────────────────────────────────────────

impl PortfolioSystem {
    /// Apply any ActionKind to a component.
    pub fn act(
        &mut self,
        actor: UserId,
        id: &ComponentId,
        action: ActionKind,
    ) -> PortfolioResult<()> {
        let node = self.node_id.clone();
        let c = self.components.get_mut(id).ok_or(PortfolioError::NotFound(*id))?;
        c.data_mut().apply_action(actor, action.clone(), &node)?;
        self.log_global(actor, action, Some(*id), "Action applied");
        Ok(())
    }

    pub fn archive(&mut self, actor: UserId, id: &ComponentId) -> PortfolioResult<()> {
        self.act(actor, id, ActionKind::Archive)
    }
    pub fn restore(&mut self, actor: UserId, id: &ComponentId) -> PortfolioResult<()> {
        self.act(actor, id, ActionKind::Restore)
    }
    pub fn publish(
        &mut self,
        actor: UserId,
        id: &ComponentId,
        visibility: Visibility,
    ) -> PortfolioResult<()> {
        self.act(actor, id, ActionKind::Post { visibility: Some(visibility) })
    }
}

// ── Graph / Relations ─────────────────────────────────────────────────────────

impl PortfolioSystem {
    /// Add a parent→child containment relationship.
    pub fn attach_child(
        &mut self,
        parent_id: ComponentId,
        child_id: ComponentId,
    ) -> PortfolioResult<()> {
        if parent_id == child_id {
            return Err(PortfolioError::InvalidOperation(
                "A component cannot be its own child.".into(),
            ));
        }
        let view = GraphView::new(&self.components, &self.edges);
        if view.would_cycle(&parent_id.to_string(), &child_id.to_string(), &GraphEdgeKind::Hierarchy) {
            return Err(PortfolioError::CyclicDependency(parent_id, child_id));
        }
        let node = self.node_id.clone();
        {
            let p = self.read_mut(&parent_id)?;
            p.data_mut().add_child(child_id);
            p.data_mut().metadata.touch(&node);
        }
        {
            let c = self.read_mut(&child_id)?;
            if !c.data().parents.contains(&parent_id) {
                c.data_mut().parents.push(parent_id);
            }
            c.data_mut().metadata.touch(&node);
        }
        let eid = self.next_edge_id_str();
        let edge = GraphEdge::new(&eid, parent_id.to_string(), child_id.to_string(), GraphEdgeKind::Hierarchy);
        self.edges.push(edge.clone());
        for plugin in &self.plugins { plugin.on_edge_added(self.edges.last().unwrap()); }
        self.emit(PortfolioEventKind::HierarchyLinked,
            [("parent", parent_id.to_string()), ("child", child_id.to_string())]);
        Ok(())
    }

    pub fn detach_child(
        &mut self,
        parent_id: ComponentId,
        child_id: ComponentId,
    ) -> PortfolioResult<()> {
        let node = self.node_id.clone();
        { let p = self.read_mut(&parent_id)?; p.data_mut().remove_child(&child_id); p.data_mut().metadata.touch(&node); }
        { let c = self.read_mut(&child_id)?;  c.data_mut().parents.retain(|id| id != &parent_id); c.data_mut().metadata.touch(&node); }
        self.edges.retain(|e| !(e.from == parent_id.to_string() && e.to == child_id.to_string() && e.kind == GraphEdgeKind::Hierarchy));
        self.emit(PortfolioEventKind::HierarchyUnlinked,
            [("parent", parent_id.to_string()), ("child", child_id.to_string())]);
        Ok(())
    }

    /// Link two components as siblings (bidirectional).
    pub fn link(&mut self, a: ComponentId, b: ComponentId) -> PortfolioResult<()> {
        let node = self.node_id.clone();
        {
            let ca = self.read_mut(&a)?;
            if !ca.data().links.contains(&b) { ca.data_mut().links.push(b); }
            ca.data_mut().metadata.touch(&node);
        }
        {
            let cb = self.read_mut(&b)?;
            if !cb.data().links.contains(&a) { cb.data_mut().links.push(a); }
            cb.data_mut().metadata.touch(&node);
        }
        let eid = self.next_edge_id_str();
        let edge = GraphEdge::new(&eid, a.to_string(), b.to_string(), GraphEdgeKind::Link);
        self.edges.push(edge);
        self.emit(PortfolioEventKind::EdgeAdded, [("a", a.to_string()), ("b", b.to_string())]);
        Ok(())
    }

    /// Register a dependency: `from` depends on `to`.
    pub fn add_dependency(
        &mut self,
        from: ComponentId,
        to: ComponentId,
    ) -> PortfolioResult<()> {
        if self.transitive_dependencies(&to)?.contains(&from) {
            return Err(PortfolioError::CyclicDependency(from, to));
        }
        let node = self.node_id.clone();
        {
            let cf = self.read_mut(&from)?;
            cf.data_mut().add_dependency(to)?;
            cf.data_mut().metadata.touch(&node);
        }
        {
            let ct = self.read_mut(&to)?;
            if !ct.data().dependents.contains(&from) { ct.data_mut().dependents.push(from); }
            ct.data_mut().metadata.touch(&node);
        }
        let eid = self.next_edge_id_str();
        let edge = GraphEdge::new(&eid, from.to_string(), to.to_string(), GraphEdgeKind::Dependency);
        self.edges.push(edge);
        self.emit(PortfolioEventKind::DependencyAdded,
            [("from", from.to_string()), ("to", to.to_string())]);
        Ok(())
    }

    pub fn transitive_dependencies(
        &self,
        id: &ComponentId,
    ) -> PortfolioResult<HashSet<ComponentId>> {
        let mut visited = HashSet::new();
        self.dfs_dependencies(id, &mut visited)?;
        visited.remove(id);
        Ok(visited)
    }

    fn dfs_dependencies(
        &self,
        id: &ComponentId,
        visited: &mut HashSet<ComponentId>,
    ) -> PortfolioResult<()> {
        if visited.contains(id) { return Ok(()); }
        visited.insert(*id);
        let c = self.read(id)?;
        for dep in &c.data().dependencies.clone() {
            self.dfs_dependencies(dep, visited)?;
        }
        Ok(())
    }

    pub fn subtree(&self, id: &ComponentId) -> Vec<ComponentId> {
        let view = GraphView::new(&self.components, &self.edges);
        view.reachable(&id.to_string(), &GraphEdgeKind::Hierarchy)
            .iter()
            .filter_map(|s| Uuid::parse_str(s).ok())
            .collect()
    }

    pub fn graph_view(&self) -> GraphView<'_> {
        GraphView::new(&self.components, &self.edges)
    }

    pub fn dependency_order(&self) -> Result<Vec<String>, String> {
        self.graph_view().topological_sort(&GraphEdgeKind::Dependency)
    }
}

// ── User / Permission Management ──────────────────────────────────────────────

impl PortfolioSystem {
    pub fn grant(
        &mut self,
        actor: UserId,
        target_user: UserId,
        component_id: &ComponentId,
        tier: PermissionTier,
    ) -> PortfolioResult<()> {
        let node = self.node_id.clone();
        let c = self.read_mut(component_id)?;
        let data = c.data_mut();
        if !data.users.can(&actor, &ActionKind::Own { tier: None }) {
            return Err(PortfolioError::PermissionDenied {
                user_id: actor, action: "grant permission".into(),
            });
        }
        data.users.add_user(target_user, tier);
        data.metadata.touch(&node);
        Ok(())
    }

    pub fn revoke(
        &mut self,
        actor: UserId,
        target_user: UserId,
        component_id: &ComponentId,
    ) -> PortfolioResult<()> {
        let node = self.node_id.clone();
        let c = self.read_mut(component_id)?;
        let data = c.data_mut();
        if !data.users.can(&actor, &ActionKind::Own { tier: None }) {
            return Err(PortfolioError::PermissionDenied {
                user_id: actor, action: "revoke permission".into(),
            });
        }
        data.users.remove_user(&target_user);
        data.metadata.touch(&node);
        Ok(())
    }
}

// ── Tagging / Hashtags ────────────────────────────────────────────────────────

impl PortfolioSystem {
    pub fn tag(&mut self, actor: UserId, id: &ComponentId, tag: impl Into<String>) -> PortfolioResult<()> {
        let t = tag.into();
        self.act(actor, id, ActionKind::Tag { label: Some(t) })
    }
    pub fn hashtag(&mut self, actor: UserId, id: &ComponentId, topic: impl Into<String>) -> PortfolioResult<()> {
        let t = topic.into();
        self.act(actor, id, ActionKind::Hashtag { topic: Some(t) })
    }
}

// ── Search ────────────────────────────────────────────────────────────────────

impl PortfolioSystem {
    pub fn search(&self, query: &SearchQuery) -> Vec<SearchResult> {
        let mut results: Vec<SearchResult> = self.components.values()
            .filter(|c| {
                let data = c.data();
                if data.status == ComponentStatus::Deleted { return false; }
                if !query.statuses.is_empty() && !query.statuses.contains(&data.status) { return false; }
                if let Some(owner) = &query.owner {
                    if !data.metadata.owners.contains(owner) { return false; }
                }
                if let Some(vis) = &query.visibility {
                    if &data.visibility != vis { return false; }
                }
                if let Some(after) = query.created_after {
                    if data.metadata.created_at < after { return false; }
                }
                if let Some(before) = query.created_before {
                    if data.metadata.created_at > before { return false; }
                }
                if !query.categories.is_empty() && !query.categories.contains(&data.category) { return false; }
                if !query.tags.is_empty() {
                    if !query.tags.iter().all(|t| data.metadata.tags.contains(t)) { return false; }
                }
                if !query.hashtags.is_empty() {
                    if !query.hashtags.iter().all(|h| data.hashtags.contains(h)) { return false; }
                }
                true
            })
            .map(|c| {
                let data = c.data();
                let mut score: f64 = 1.0;
                let mut matched = Vec::new();
                if let Some(text) = &query.text {
                    let tl = text.to_lowercase();
                    if data.name.to_lowercase().contains(&tl)               { score += 3.0; matched.push("name".into()); }
                    if data.description.to_lowercase().contains(&tl)        { score += 1.0; matched.push("description".into()); }
                    if data.metadata.tags.iter().any(|t| t.to_lowercase().contains(&tl)) { score += 1.5; matched.push("tags".into()); }
                    if data.hashtags.iter().any(|h| h.to_lowercase().contains(&tl))      { score += 1.5; matched.push("hashtags".into()); }
                }
                score += data.analytics.engagement_rate * 0.5;
                SearchResult {
                    component_id: data.id(), name: data.name.clone(),
                    category: data.category.clone(), status: data.status.clone(),
                    score, matched_fields: matched,
                }
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.into_iter().skip(query.offset).take(query.limit.unwrap_or(usize::MAX)).collect()
    }

    /// PQL filter: type/status/tag/name/owner predicates.
    pub fn query(&self, q: &PortfolioQuery) -> Vec<&Component> {
        self.components.values().filter(|c| {
            let data = c.data();
            if let Some(ref cats) = q.categories {
                if !cats.contains(&data.category) { return false; }
            }
            if let Some(ref statuses) = q.statuses {
                if !statuses.contains(&data.status) { return false; }
            }
            if let Some(ref tags) = q.tags {
                if !tags.iter().all(|t| data.metadata.tags.contains(t)) { return false; }
            }
            if let Some(ref substr) = q.name_contains {
                if !data.name.to_lowercase().contains(&substr.to_lowercase()) { return false; }
            }
            if let Some(ref owner) = q.owner {
                if !data.metadata.owners.contains(owner) { return false; }
            }
            if let Some(want_deps) = q.has_dependencies {
                if data.dependencies.is_empty() == want_deps { return false; }
            }
            if let Some(want_children) = q.has_children {
                if data.children.is_empty() == want_children { return false; }
            }
            true
        }).collect()
    }
}

// ── Analytics ─────────────────────────────────────────────────────────────────

impl PortfolioSystem {
    pub fn record_view(&mut self, id: &ComponentId, duration_seconds: u64) -> PortfolioResult<()> {
        self.components.get_mut(id).ok_or(PortfolioError::NotFound(*id))?
            .data_mut().analytics.record_view(duration_seconds);
        Ok(())
    }
    pub fn record_click(&mut self, id: &ComponentId) -> PortfolioResult<()> {
        self.components.get_mut(id).ok_or(PortfolioError::NotFound(*id))?
            .data_mut().analytics.record_click();
        Ok(())
    }
    pub fn analytics(&self, id: &ComponentId) -> PortfolioResult<&ComponentAnalytics> {
        Ok(&self.read(id)?.data().analytics)
    }
    pub fn benchmark(&self, ids: &[ComponentId]) -> HashMap<ComponentId, ComponentAnalytics> {
        ids.iter().filter_map(|id| self.read(id).ok().map(|c| (*id, c.data().analytics.clone()))).collect()
    }
    pub fn platform_engagement(&self) -> u64 {
        self.components.values().map(|c| c.data().analytics.total_engagements()).sum()
    }
}

// ── Queries (counts / queries) ────────────────────────────────────────────────

impl PortfolioSystem {
    pub fn total_components(&self) -> usize { self.components.len() }
    pub fn total_items(&self)      -> usize { self.components.values().filter(|c| c.is_item()).count() }
    pub fn total_containers(&self) -> usize { self.components.values().filter(|c| c.is_container()).count() }

    pub fn components_by_owner(&self, owner: &UserId) -> Vec<ComponentId> {
        self.components.values()
            .filter(|c| c.data().metadata.owners.contains(owner))
            .map(|c| c.id()).collect()
    }
    pub fn components_by_tag(&self, tag: &str) -> Vec<ComponentId> {
        self.components.values()
            .filter(|c| c.data().metadata.tags.contains(tag))
            .map(|c| c.id()).collect()
    }
    pub fn components_by_status(&self, status: &ComponentStatus) -> Vec<ComponentId> {
        self.components.values()
            .filter(|c| &c.data().status == status)
            .map(|c| c.id()).collect()
    }
    pub fn children(&self, id: &ComponentId) -> PortfolioResult<Vec<&Component>> {
        let parent = self.read(id)?;
        let mut children: Vec<&Component> = parent.data().children.iter()
            .filter_map(|cid| self.components.get(cid)).collect();
        children.sort_by_key(|c| c.name());
        Ok(children)
    }
    pub fn ancestors(&self, id: &ComponentId) -> PortfolioResult<Vec<ComponentId>> {
        let mut result = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(*id);
        let mut seen = HashSet::new();
        while let Some(current) = queue.pop_front() {
            if !seen.insert(current) { continue; }
            let c = self.read(&current)?;
            for parent_id in &c.data().parents {
                if !seen.contains(parent_id) { result.push(*parent_id); queue.push_back(*parent_id); }
            }
        }
        Ok(result)
    }
    pub fn audit_trail(&self, id: &ComponentId) -> PortfolioResult<Vec<&ActivityLog>> {
        let c = self.read(id)?;
        Ok(self.global_log.iter()
            .chain(c.data().action_history.iter())
            .filter(|l| l.target_id.as_ref() == Some(id))
            .collect())
    }
}

// ── Version Control ───────────────────────────────────────────────────────────

impl PortfolioSystem {
    pub fn bump_version(
        &mut self,
        actor: UserId,
        id: &ComponentId,
        part: VersionPart,
        message: impl Into<String>,
    ) -> PortfolioResult<VersionString> {
        let node = self.node_id.clone();
        let c = self.read_mut(id)?;
        let data = c.data_mut();

        if !data.users.can(&actor, &ActionKind::Edit) {
            return Err(PortfolioError::PermissionDenied {
                user_id: actor, action: "bump_version".into(),
            });
        }
        let new_version = increment_version(&data.metadata.version, part);
        data.metadata.version = new_version.clone();
        data.metadata.touch(&node);

        if let Component::Item(item) = c {
            if let Some(book) = &mut item.item_book {
                let history = book.version.get_or_insert(VersionHistory {
                    current_version: new_version.clone(), entries: Vec::new(),
                });
                history.current_version = new_version.clone();
                history.entries.push(VersionEntry {
                    version: new_version.clone(), author: actor,
                    message: message.into(), diff_ref: None, tags: Vec::new(),
                    created_at: Utc::now(),
                });
            }
        }
        Ok(new_version)
    }
}

// ── Structural Primitive CRUD ─────────────────────────────────────────────────

impl PortfolioSystem {
    pub fn create_group(&mut self, name: impl Into<String>, owner: UserId) -> Uuid {
        let g = Group::new(name, owner); let id = g.id; self.groups.insert(id, g); id
    }
    pub fn group(&self, id: &Uuid) -> Option<&Group> { self.groups.get(id) }
    pub fn group_mut(&mut self, id: &Uuid) -> Option<&mut Group> { self.groups.get_mut(id) }

    pub fn create_collection(&mut self, name: impl Into<String>, owner: UserId) -> Uuid {
        let c = Collection::new(name, owner); let id = c.id; self.collections.insert(id, c); id
    }
    pub fn collection(&self, id: &Uuid) -> Option<&Collection> { self.collections.get(id) }
    pub fn collection_mut(&mut self, id: &Uuid) -> Option<&mut Collection> { self.collections.get_mut(id) }

    pub fn create_list(&mut self, name: impl Into<String>, owner: UserId) -> Uuid {
        let l = List::new(name, owner); let id = l.id; self.lists.insert(id, l); id
    }
    pub fn list(&self, id: &Uuid) -> Option<&List> { self.lists.get(id) }
    pub fn list_mut(&mut self, id: &Uuid) -> Option<&mut List> { self.lists.get_mut(id) }

    pub fn create_schedule(&mut self, name: impl Into<String>, owner: UserId) -> Uuid {
        let s = Schedule::new(name, owner); let id = s.id; self.schedules.insert(id, s); id
    }
    pub fn schedule(&self, id: &Uuid) -> Option<&Schedule> { self.schedules.get(id) }
    pub fn schedule_mut(&mut self, id: &Uuid) -> Option<&mut Schedule> { self.schedules.get_mut(id) }

    pub fn create_directory(
        &mut self, name: impl Into<String>, path: impl Into<String>, owner: UserId,
    ) -> Uuid {
        let d = Directory::new(name, path, owner); let id = d.id; self.directories.insert(id, d); id
    }
    pub fn directory(&self, id: &Uuid) -> Option<&Directory> { self.directories.get(id) }
    pub fn directory_mut(&mut self, id: &Uuid) -> Option<&mut Directory> { self.directories.get_mut(id) }
}

// ── Snapshot, Checkpoint & Time-Travel ───────────────────────────────────────

impl PortfolioSystem {
    fn build_snapshot(&self, label: Option<String>) -> PortfolioSnapshot {
        let snap_id = format!("snap-{:04}", self.next_snap_id);
        PortfolioSnapshot {
            snapshot_id: snap_id,
            label,
            created_at: now_epoch(),
            components: self.components.clone(),
            edges: self.edges.clone(),
            active_portfolio_id: self.active_portfolio_id,
            next_id: 0,
            next_edge_id: self.next_edge_id,
            component_count: self.components.len(),
            edge_count: self.edges.len(),
            event_count: self.event_log.len(),
            last_checkpoint_id: self.checkpoints.last().map(|c| c.id.clone()),
        }
    }

    pub fn snapshot(&self) -> PortfolioSnapshot { self.build_snapshot(None) }

    pub fn save_snapshot(&mut self, label: Option<String>) -> PortfolioSnapshot {
        let snap = self.build_snapshot(label);
        let id = snap.snapshot_id.clone();
        self.snapshots.insert(id.clone(), snap.clone());
        for plugin in &self.plugins { plugin.on_snapshot_saved(&snap); }
        self.emit(PortfolioEventKind::SnapshotSaved, [("snapshot_id", id)]);
        snap
    }

    pub fn save_checkpoint(
        &mut self,
        label: impl Into<String>,
        note: Option<String>,
    ) -> PortfolioCheckpoint {
        let label = label.into();
        let snap = self.save_snapshot(Some(label.clone()));
        let ckpt_id = format!("ckpt-{:04}", self.next_ckpt_id);
        self.next_ckpt_id += 1;
        let ckpt = PortfolioCheckpoint {
            id: ckpt_id, label, snapshot_id: snap.snapshot_id.clone(),
            created_at: now_epoch(), note,
        };
        self.checkpoints.push(ckpt.clone());
        self.emit(PortfolioEventKind::CheckpointCreated,
            [("checkpoint_id", ckpt.id.clone()), ("snapshot_id", snap.snapshot_id)]);
        ckpt
    }

    pub fn restore_snapshot(&mut self, snapshot_id: &str) -> PortfolioResult<()> {
        let snap = self.snapshots.get(snapshot_id)
            .ok_or_else(|| PortfolioError::InvalidOperation(
                format!("Snapshot '{snapshot_id}' not found"),
            ))?.clone();
        self.components          = snap.components;
        self.edges               = snap.edges;
        self.active_portfolio_id = snap.active_portfolio_id;
        self.next_edge_id        = snap.next_edge_id;
        self.emit(PortfolioEventKind::StateRestored, [("snapshot_id", snapshot_id.to_string())]);
        Ok(())
    }
}

// ── Governance ────────────────────────────────────────────────────────────────

impl PortfolioSystem {
    pub fn register_policy_engine(&mut self, engine: Arc<dyn PolicyEngine>) {
        self.policy_engines.push(engine);
    }

    pub fn attach_policy(&mut self, component_id: &ComponentId, policy_id: PolicyId) -> PortfolioResult<()> {
        let c = self.read_mut(component_id)?;
        c.data_mut().metadata.policy_ids.push(policy_id);
        self.emit(PortfolioEventKind::PolicyAttached,
            [("component_id", component_id.to_string()), ("policy_id", policy_id.to_string())]);
        Ok(())
    }

    pub fn detach_policy(&mut self, component_id: &ComponentId, policy_id: &PolicyId) -> PortfolioResult<()> {
        let c = self.read_mut(component_id)?;
        c.data_mut().metadata.policy_ids.retain(|p| p != policy_id);
        self.emit(PortfolioEventKind::PolicyDetached,
            [("component_id", component_id.to_string()), ("policy_id", policy_id.to_string())]);
        Ok(())
    }

    pub fn request_approval(
        &mut self,
        component_id: ComponentId,
        reason: impl Into<String>,
    ) -> ApprovalRequest {
        let id = format!("req-{:06}", self.next_req_id);
        self.next_req_id += 1;
        let req = ApprovalRequest {
            id: id.clone(), component_id, requested_by: self.node_id.clone(),
            reason: reason.into(), status: ApprovalStatus::Pending,
            created_at: now_epoch(), resolved_at: None, resolver: None, notes: None,
        };
        self.approval_requests.push(req.clone());
        self.emit(PortfolioEventKind::ApprovalRequested,
            [("request_id", id), ("component_id", component_id.to_string())]);
        req
    }

    pub fn resolve_approval(
        &mut self,
        request_id: &str,
        approved: bool,
        resolver: impl Into<String>,
        notes: Option<String>,
    ) -> PortfolioResult<ApprovalRequest> {
        let req = self.approval_requests.iter_mut()
            .find(|r| r.id == request_id)
            .ok_or_else(|| PortfolioError::InvalidOperation(
                format!("Approval request '{request_id}' not found"),
            ))?;
        req.status     = if approved { ApprovalStatus::Approved } else { ApprovalStatus::Rejected };
        req.resolved_at = Some(now_epoch());
        req.resolver   = Some(resolver.into());
        req.notes      = notes;
        let result = req.clone();
        self.emit(
            if approved { PortfolioEventKind::ApprovalGranted } else { PortfolioEventKind::ApprovalRejected },
            [("request_id", request_id.to_string())],
        );
        Ok(result)
    }

    pub fn allocate_resource(
        &mut self,
        component_id: &ComponentId,
        kind: ResourceKind,
        total: f64,
        denomination: impl Into<String>,
        period: Option<String>,
    ) -> PortfolioResult<ResourceAllocation> {
        self.read(component_id)?;
        let denomination = denomination.into();
        let alloc = ResourceAllocation {
            component_id: component_id.to_string(), kind, total, allocated: total,
            consumed: 0.0, denomination: denomination.clone(), period,
        };
        self.resource_allocations.insert(*component_id, alloc.clone());
        if let Some(comp) = self.components.get_mut(component_id) {
            comp.data_mut().metadata.budget = Some(total);
        }
        self.emit(PortfolioEventKind::ResourceAllocated,
            [("component_id", component_id.to_string()), ("total", total.to_string()),
             ("denomination", denomination)]);
        Ok(alloc)
    }

    pub fn allocate_budget(
        &mut self,
        component_id: &ComponentId,
        total: f64,
        currency: impl Into<String>,
        period: Option<String>,
    ) -> PortfolioResult<ResourceAllocation> {
        self.allocate_resource(component_id, ResourceKind::Budget, total, currency, period)
    }

    pub fn record_consumption(
        &mut self,
        component_id: &ComponentId,
        amount: f64,
    ) -> PortfolioResult<f64> {
        let alloc = self.resource_allocations.get_mut(component_id)
            .ok_or_else(|| PortfolioError::InvalidOperation(
                format!("No resource allocated for '{component_id}'"),
            ))?;
        alloc.consumed += amount;
        let remaining = alloc.remaining();
        if let Some(comp) = self.components.get_mut(component_id) {
            comp.data_mut().metadata.budget_spent += amount;
        }
        self.emit(PortfolioEventKind::ResourceConsumed,
            [("component_id", component_id.to_string()), ("amount", amount.to_string())]);
        Ok(remaining)
    }

    pub fn record_spend(&mut self, id: &ComponentId, amount: f64) -> PortfolioResult<f64> {
        self.record_consumption(id, amount)
    }

    pub fn get_resource_allocation(&self, id: &ComponentId) -> Option<&ResourceAllocation> {
        self.resource_allocations.get(id)
    }

    pub fn overrun_allocations(&self) -> Vec<&ResourceAllocation> {
        self.resource_allocations.values().filter(|a| a.is_overrun()).collect()
    }
}

// ── Plugin Registration ───────────────────────────────────────────────────────

impl PortfolioSystem {
    pub fn register_plugin(&mut self, plugin: Arc<dyn PortfolioPlugin>) {
        self.plugins.push(plugin);
    }
    pub fn unregister_plugin(&mut self, plugin_id: &str) {
        self.plugins.retain(|p| p.id() != plugin_id);
    }
}

// ── CRDT / Distributed Merge ──────────────────────────────────────────────────

impl PortfolioSystem {
    pub fn apply_crdt_log(&mut self, remote: &CrdtLog) {
        let ops: Vec<CrdtOperation> = remote.operations.clone();
        for op in &ops { self.apply_crdt_op(op); }
        self.crdt.merge(remote);
        for plugin in &self.plugins { plugin.on_crdt_merge(&ops); }
        self.emit(PortfolioEventKind::CrdtMergeApplied,
            [("op_count", ops.len().to_string())]);
    }

    fn apply_crdt_op(&mut self, op: &CrdtOperation) {
        match op {
            CrdtOperation::SetField { component_id, field, value, timestamp, actor } => {
                if let Ok(uid) = Uuid::parse_str(component_id) {
                    if let Some(comp) = self.components.get_mut(&uid) {
                        let data = comp.data_mut();
                        let accept = data.metadata.updated_at < chrono::DateTime::from_timestamp(*timestamp as i64, 0).unwrap_or_default()
                            || data.metadata.last_actor < *actor;
                        if accept {
                            match field.as_str() {
                                "name"   => data.name = value.clone(),
                                "status" => {} // status handled via ComponentStatus enum
                                _        => { data.metadata.properties.insert(field.clone(), serde_json::Value::String(value.clone())); }
                            }
                        }
                    }
                }
            }
            CrdtOperation::AddEdge { edge, .. } => {
                if !self.edges.iter().any(|e| e.id == edge.id) { self.edges.push(edge.clone()); }
            }
            CrdtOperation::RemoveEdge { edge_id, .. } => {
                self.edges.retain(|e| &e.id != edge_id);
            }
            _ => {} // AddToSet / RemoveFromSet handled at application layer
        }
    }

    pub fn crdt_log(&self) -> &CrdtLog { &self.crdt }
}

// =============================================================================
// §26 — FEDERATION
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationPeer {
    pub id: String,
    pub name: String,
    pub endpoint: Option<String>,
    pub last_seen: u64,
    pub vector_clock: VectorClock,
    pub trusted: bool,
}

pub struct PortfolioFederation {
    pub portfolios: HashMap<String, PortfolioSystem>,
    pub peers: HashMap<String, FederationPeer>,
    pub federation_edges: Vec<GraphEdge>,
    next_edge_id: u64,
}

impl PortfolioFederation {
    pub fn new() -> Self {
        Self { portfolios: HashMap::new(), peers: HashMap::new(),
               federation_edges: Vec::new(), next_edge_id: 1 }
    }
    pub fn add_portfolio(&mut self, id: impl Into<String>, system: PortfolioSystem) {
        self.portfolios.insert(id.into(), system);
    }
    pub fn remove_portfolio(&mut self, id: &str) -> Option<PortfolioSystem> {
        self.portfolios.remove(id)
    }
    pub fn get_portfolio(&self, id: &str) -> Option<&PortfolioSystem> { self.portfolios.get(id) }
    pub fn get_portfolio_mut(&mut self, id: &str) -> Option<&mut PortfolioSystem> { self.portfolios.get_mut(id) }

    pub fn register_peer(&mut self, peer: FederationPeer) { self.peers.insert(peer.id.clone(), peer); }
    pub fn remove_peer(&mut self, peer_id: &str) -> Option<FederationPeer> { self.peers.remove(peer_id) }

    /// Add a cross-portfolio Federation edge.
    pub fn federate(
        &mut self,
        from_portfolio: &str, from_component: &str,
        to_portfolio: &str,   to_component: &str,
    ) {
        let id = format!("fed-edge-{:04}", self.next_edge_id);
        self.next_edge_id += 1;
        self.federation_edges.push(GraphEdge {
            id, from: format!("{from_portfolio}/{from_component}"),
            to: format!("{to_portfolio}/{to_component}"),
            kind: GraphEdgeKind::Federation, label: None,
            created_at: now_epoch(), properties: HashMap::new(),
        });
    }

    /// Push CRDT log from `source_id` into `target_id`.
    pub fn sync_crdt(&mut self, source_id: &str, target_id: &str) -> Result<(), String> {
        let source_crdt = self.portfolios.get(source_id)
            .ok_or_else(|| format!("Portfolio '{source_id}' not found"))?
            .crdt.clone();
        let target = self.portfolios.get_mut(target_id)
            .ok_or_else(|| format!("Portfolio '{target_id}' not found"))?;
        target.apply_crdt_log(&source_crdt);
        Ok(())
    }

    /// Broadcast one portfolio's CRDT log to all others.
    pub fn broadcast_crdt(&mut self, source_id: &str) -> Result<usize, String> {
        let target_ids: Vec<String> = self.portfolios.keys()
            .filter(|k| k.as_str() != source_id).cloned().collect();
        let count = target_ids.len();
        for tid in target_ids { self.sync_crdt(source_id, &tid)?; }
        Ok(count)
    }
}

impl Default for PortfolioFederation {
    fn default() -> Self { Self::new() }
}

// =============================================================================
// §27 — BUILDER  (fluent construction helpers)
// =============================================================================

pub struct ItemBuilder {
    owner: UserId,
    name: String,
    description: String,
    item_type: ItemType,
    tags: HashSet<String>,
    visibility: Visibility,
    status: ComponentStatus,
}

impl ItemBuilder {
    pub fn portfolio(owner: UserId, name: impl Into<String>) -> Self {
        Self {
            owner, name: name.into(), description: String::new(),
            item_type: ItemType::Portfolio(PortfolioItemPayload {
                mission: String::new(), focus_areas: Vec::new(), kpis: Vec::new(),
            }),
            tags: HashSet::new(), visibility: Visibility::Private,
            status: ComponentStatus::Draft,
        }
    }

    pub fn project(owner: UserId, name: impl Into<String>, project_type: ProjectType) -> Self {
        Self {
            owner, name: name.into(), description: String::new(),
            item_type: ItemType::Project(ProjectPayload {
                project_type, methodology: None, start_date: None, end_date: None,
                sprints: Vec::new(), backlog: Vec::new(), releases: Vec::new(),
            }),
            tags: HashSet::new(), visibility: Visibility::Private,
            status: ComponentStatus::Draft,
        }
    }

    pub fn description(mut self, d: impl Into<String>) -> Self { self.description = d.into(); self }
    pub fn tag(mut self, t: impl Into<String>) -> Self { self.tags.insert(t.into()); self }
    pub fn visibility(mut self, v: Visibility) -> Self { self.visibility = v; self }
    pub fn status(mut self, s: ComponentStatus) -> Self { self.status = s; self }

    pub fn build(self, system: &mut PortfolioSystem) -> PortfolioResult<ComponentId> {
        let id = system.create_item(self.owner, self.name, self.item_type)?;
        {
            let c = system.read_mut(&id)?;
            let data = c.data_mut();
            data.description = self.description;
            data.metadata.tags = self.tags;
            data.visibility = self.visibility;
            data.status = self.status;
        }
        Ok(id)
    }
}

// =============================================================================
// §28 — COMPUTATIONAL MODELS
// =============================================================================

// ── 28.1  Portfolio Health ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioHealthResult {
    pub health_score: f64,
    pub active_ratio: f64,
    pub resource_utilisation_pct: f64,
    pub total_resource_units: f64,
    pub summary: String,
}

pub struct PortfolioHealthModel;
impl PortfolioHealthModel {
    pub fn compute(
        children: &[&Component],
        allocation: Option<&ResourceAllocation>,
    ) -> PortfolioHealthResult {
        let total  = children.len();
        let active = children.iter().filter(|c| c.data().status == ComponentStatus::Active).count();
        let active_ratio = if total == 0 { 0.0 } else { active as f64 / total as f64 };
        let utilisation_pct = allocation.map(|a| a.utilisation_pct()).unwrap_or(0.0);
        let total_resource_units: f64 = children.iter().map(|c| c.data().metadata.resource_units).sum();
        let activity_pts  = if active_ratio >= 0.8 { 50.0 } else { active_ratio * 62.5 };
        let resource_pts  = if utilisation_pct > 120.0 { 0.0 }
                            else { 30.0 * (1.0 - ((utilisation_pct - 100.0).max(0.0) / 20.0)) };
        let coverage_pts  = if total > 0 { 20.0 } else { 0.0 };
        let health_score  = (activity_pts + resource_pts + coverage_pts).clamp(0.0, 100.0);
        PortfolioHealthResult {
            health_score, active_ratio,
            resource_utilisation_pct: utilisation_pct, total_resource_units,
            summary: format!("Health {health_score:.1}/100 — {active}/{total} active, utilisation {utilisation_pct:.1}%"),
        }
    }
}

// ── 28.2  Project Metrics (Earned Value) ──────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetricsInput {
    pub planned_value: f64,
    pub earned_value: f64,
    pub actual_cost: f64,
    pub budget_at_completion: f64,
    pub schedule_risk_weight: f64,
    pub open_risk_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetricsResult {
    pub cpi: f64,
    pub spi: f64,
    pub cost_variance: f64,
    pub schedule_variance: f64,
    pub estimate_at_completion: f64,
    pub estimate_to_complete: f64,
    pub risk_score: f64,
    pub summary: String,
}

pub struct ProjectMetricsModel;
impl ProjectMetricsModel {
    pub fn compute(input: &ProjectMetricsInput) -> ProjectMetricsResult {
        let cpi = if input.actual_cost  == 0.0 { 1.0 } else { input.earned_value / input.actual_cost };
        let spi = if input.planned_value == 0.0 { 1.0 } else { input.earned_value / input.planned_value };
        let eac = if cpi == 0.0 { f64::INFINITY } else { input.budget_at_completion / cpi };
        let cost_risk   = (1.0 - cpi.min(1.0)) * 40.0;
        let sched_risk  = (1.0 - spi.min(1.0)) * input.schedule_risk_weight * 40.0;
        let open_risk_pts = (input.open_risk_count as f64 * 5.0).min(20.0);
        let risk_score  = (cost_risk + sched_risk + open_risk_pts).clamp(0.0, 100.0);
        ProjectMetricsResult {
            cpi, spi,
            cost_variance: input.earned_value - input.actual_cost,
            schedule_variance: input.earned_value - input.planned_value,
            estimate_at_completion: eac,
            estimate_to_complete: eac - input.actual_cost,
            risk_score,
            summary: format!("CPI={cpi:.2} SPI={spi:.2} EAC={eac:.0} risk={risk_score:.1}/100"),
        }
    }
}

// ── 28.3  Program Alignment ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramAlignmentResult {
    pub benefit_realisation_index: f64,
    pub strategic_coverage: f64,
    pub coherence: f64,
    pub alignment_score: f64,
    pub summary: String,
}

pub struct ProgramAlignmentModel;
impl ProgramAlignmentModel {
    pub fn compute(
        children: &[&Component],
        strategic_goal_ids: &[String],
        covered_goal_ids: &[String],
        planned_benefit: f64,
        realised_benefit: f64,
    ) -> ProgramAlignmentResult {
        let bri = if planned_benefit == 0.0 { 1.0 } else { realised_benefit / planned_benefit };
        let strategic_coverage = if strategic_goal_ids.is_empty() { 1.0 } else {
            let covered = covered_goal_ids.iter().filter(|g| strategic_goal_ids.contains(g)).count();
            covered as f64 / strategic_goal_ids.len() as f64
        };
        let total  = children.len();
        let active = children.iter().filter(|c| c.data().status == ComponentStatus::Active).count();
        let coherence = if total == 0 { 0.0 } else { active as f64 / total as f64 };
        let alignment_score = (bri.min(1.0) * 40.0 + strategic_coverage * 40.0 + coherence * 20.0).clamp(0.0, 100.0);
        ProgramAlignmentResult {
            benefit_realisation_index: bri, strategic_coverage, coherence, alignment_score,
            summary: format!("Alignment {alignment_score:.1}/100 — BRI={bri:.2} coverage={:.0}% coherence={:.0}%",
                strategic_coverage * 100.0, coherence * 100.0),
        }
    }
}

// ── 28.4  SubPortfolio Rollup ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubPortfolioRollupResult {
    pub total_components: usize,
    pub active_components: usize,
    pub total_resource_units: f64,
    pub total_budget_allocated: f64,
    pub total_budget_consumed: f64,
    pub rollup_score: f64,
    pub summary: String,
}

pub struct SubPortfolioRollupModel;
impl SubPortfolioRollupModel {
    pub fn compute(
        children: &[&Component],
        allocations: &HashMap<ComponentId, ResourceAllocation>,
    ) -> SubPortfolioRollupResult {
        let total  = children.len();
        let active = children.iter().filter(|c| c.data().status == ComponentStatus::Active).count();
        let total_resource_units: f64 = children.iter().map(|c| c.data().metadata.resource_units).sum();
        let (total_budget_allocated, total_budget_consumed) = children.iter()
            .fold((0.0_f64, 0.0_f64), |(a, c_acc), comp| {
                if let Some(alloc) = allocations.get(&comp.id()) {
                    (a + alloc.allocated, c_acc + alloc.consumed)
                } else { (a, c_acc) }
            });
        let active_ratio   = if total == 0 { 0.0 } else { active as f64 / total as f64 };
        let budget_health  = if total_budget_allocated == 0.0 { 1.0 } else {
            (1.0 - ((total_budget_consumed / total_budget_allocated) - 1.0).max(0.0)).clamp(0.0, 1.0)
        };
        let rollup_score = (active_ratio * 60.0 + budget_health * 40.0).clamp(0.0, 100.0);
        SubPortfolioRollupResult {
            total_components: total, active_components: active,
            total_resource_units, total_budget_allocated, total_budget_consumed, rollup_score,
            summary: format!("Rollup {rollup_score:.1}/100 — {active}/{total} active, budget {total_budget_consumed:.0}/{total_budget_allocated:.0}"),
        }
    }
}

// ── 28.5  Resource Utilisation ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilisationResult {
    pub utilisation_pct: f64,
    pub over_committed: bool,
    pub demand_ratio: f64,
    pub summary: String,
}

pub struct ResourceUtilisationModel;
impl ResourceUtilisationModel {
    pub fn compute(
        total_capacity: f64,
        committed: f64,
        consumed: f64,
        assigned_items: usize,
    ) -> ResourceUtilisationResult {
        let utilisation_pct = if total_capacity == 0.0 { 0.0 } else { (consumed / total_capacity) * 100.0 };
        let demand_ratio    = if total_capacity == 0.0 { 0.0 } else { committed / total_capacity };
        let over_committed  = committed > total_capacity;
        ResourceUtilisationResult {
            utilisation_pct, over_committed, demand_ratio,
            summary: format!("Utilisation {utilisation_pct:.1}% — {assigned_items} items, demand={demand_ratio:.2}"),
        }
    }
}

// ── 28.6  Asset Value ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetValueResult {
    pub current_value: f64,
    pub depreciation: f64,
    pub roi_pct: f64,
    pub summary: String,
}

pub struct AssetValueModel;
impl AssetValueModel {
    /// `annual_rate` — depreciation fraction per year (e.g. 0.2 = 20 %).
    pub fn compute(
        initial_value: f64,
        annual_rate: f64,
        age_years: f64,
        cost_basis: f64,
    ) -> AssetValueResult {
        let depreciation  = initial_value * annual_rate * age_years;
        let current_value = (initial_value - depreciation).max(0.0);
        let roi_pct       = if cost_basis == 0.0 { 0.0 } else { ((current_value - cost_basis) / cost_basis) * 100.0 };
        AssetValueResult {
            current_value, depreciation, roi_pct,
            summary: format!("Value {current_value:.2}, depreciation {depreciation:.2}, ROI {roi_pct:.1}%"),
        }
    }
}

// ── 28.7  Artifact Maturity ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMaturityResult {
    pub completeness: f64,
    pub freshness: f64,
    pub reuse_score: f64,
    pub maturity_score: f64,
    pub summary: String,
}

pub struct ArtifactMaturityModel;
impl ArtifactMaturityModel {
    pub fn compute(
        required_populated: usize,
        required_total: usize,
        days_since_update: u64,
        max_fresh_days: u64,
        reuse_count: usize,
        is_reviewed: bool,
    ) -> ArtifactMaturityResult {
        let completeness = if required_total == 0 { 1.0 } else { required_populated as f64 / required_total as f64 };
        let freshness    = if max_fresh_days == 0  { 1.0 } else { (1.0 - (days_since_update as f64 / max_fresh_days as f64)).clamp(0.0, 1.0) };
        let reuse_score  = (reuse_count as f64 * 10.0).min(100.0) / 100.0;
        let review_bonus = if is_reviewed { 10.0 } else { 0.0 };
        let maturity_score = (completeness * 40.0 + freshness * 30.0 + reuse_score * 20.0 + review_bonus).clamp(0.0, 100.0);
        ArtifactMaturityResult {
            completeness, freshness, reuse_score, maturity_score,
            summary: format!("Maturity {maturity_score:.1}/100 — completeness={:.0}% freshness={:.0}% reuse={reuse_count}",
                completeness * 100.0, freshness * 100.0),
        }
    }
}

// ── 28.8  Binder Coverage ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinderCoverageResult {
    pub coverage_pct: f64,
    pub missing_count: usize,
    pub extra_count: usize,
    pub summary: String,
}

pub struct BinderCoverageModel;
impl BinderCoverageModel {
    pub fn compute(
        expected_ids: &HashSet<String>,
        actual_ids: &HashSet<String>,
    ) -> BinderCoverageResult {
        let covered      = expected_ids.iter().filter(|id| actual_ids.contains(*id)).count();
        let missing      = expected_ids.len() - covered;
        let extra        = actual_ids.iter().filter(|id| !expected_ids.contains(*id)).count();
        let coverage_pct = if expected_ids.is_empty() { 100.0 } else { (covered as f64 / expected_ids.len() as f64) * 100.0 };
        BinderCoverageResult {
            coverage_pct, missing_count: missing, extra_count: extra,
            summary: format!("Coverage {coverage_pct:.1}% — {covered}/{} present, {missing} missing, {extra} extra",
                expected_ids.len()),
        }
    }
}

// ── 28.9  Record Integrity ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordIntegrityResult {
    pub integrity_score: f64,
    pub duplicate_count: usize,
    pub has_integrity_hash: bool,
    pub summary: String,
}

pub struct RecordIntegrityModel;
impl RecordIntegrityModel {
    pub fn compute(
        entry_count: usize,
        duplicate_count: usize,
        has_integrity_hash: bool,
    ) -> RecordIntegrityResult {
        let dup_penalty  = (duplicate_count as f64 * 10.0).min(50.0);
        let hash_bonus   = if has_integrity_hash { 10.0 } else { 0.0 };
        let integrity_score = (100.0 - dup_penalty + hash_bonus).clamp(0.0, 100.0);
        RecordIntegrityResult {
            integrity_score, duplicate_count, has_integrity_hash,
            summary: format!("Integrity {integrity_score:.1}/100 — {entry_count} entries, {duplicate_count} dups"),
        }
    }
}

// ── 28.10  Folder Organisation ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderOrganisationResult {
    pub organisation_score: f64,
    pub max_depth: usize,
    pub orphan_count: usize,
    pub duplicate_name_count: usize,
    pub summary: String,
}

pub struct FolderOrganisationModel;
impl FolderOrganisationModel {
    pub fn compute(
        max_depth: usize,
        depth_threshold: usize,
        orphan_count: usize,
        duplicate_name_count: usize,
    ) -> FolderOrganisationResult {
        let depth_penalty = if max_depth > depth_threshold { (max_depth - depth_threshold) as f64 * 10.0 } else { 0.0 };
        let orphan_penalty = orphan_count as f64 * 5.0;
        let dup_penalty    = duplicate_name_count as f64 * 8.0;
        let organisation_score = (100.0 - depth_penalty - orphan_penalty - dup_penalty).clamp(0.0, 100.0);
        FolderOrganisationResult {
            organisation_score, max_depth, orphan_count, duplicate_name_count,
            summary: format!("Organisation {organisation_score:.1}/100 — depth={max_depth} orphans={orphan_count} dups={duplicate_name_count}"),
        }
    }
}

// ── System-level compute helpers ─────────────────────────────────────────────

impl PortfolioSystem {
    pub fn compute_portfolio_health(&self, id: &ComponentId) -> PortfolioResult<PortfolioHealthResult> {
        let children = self.children(id)?;
        let alloc    = self.resource_allocations.get(id);
        Ok(PortfolioHealthModel::compute(&children, alloc))
    }

    pub fn compute_subportfolio_rollup(&self, id: &ComponentId) -> PortfolioResult<SubPortfolioRollupResult> {
        let children = self.children(id)?;
        Ok(SubPortfolioRollupModel::compute(&children, &self.resource_allocations))
    }

    pub fn compute_binder_coverage(
        &self,
        binder_id: &ComponentId,
        expected_ids: &HashSet<String>,
    ) -> PortfolioResult<BinderCoverageResult> {
        let binder = self.read(binder_id)?;
        let actual: HashSet<String> = binder.data().children.iter().map(|id| id.to_string()).collect();
        Ok(BinderCoverageModel::compute(expected_ids, &actual))
    }

    pub fn compute_record_integrity(&self, record_id: &ComponentId) -> PortfolioResult<RecordIntegrityResult> {
        let record = self.read(record_id)?;
        let data   = record.data();
        let mut seen: HashMap<ComponentId, usize> = HashMap::new();
        let mut dups = 0;
        for c in &data.children {
            let cnt = seen.entry(*c).or_insert(0);
            *cnt += 1;
            if *cnt == 2 { dups += 1; }
        }
        let has_hash = data.metadata.properties.contains_key("integrity_hash");
        Ok(RecordIntegrityModel::compute(data.children.len(), dups, has_hash))
    }
}

// =============================================================================
// §29 — TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn owner() -> UserId { Uuid::new_v4() }

    #[test]
    fn test_create_item_and_read() {
        let mut sys = PortfolioSystem::new("node-1");
        let user = owner();
        let id = sys.create_item(user, "My Portfolio", ItemType::Portfolio(PortfolioItemPayload {
            mission: "Test mission".into(),
            focus_areas: vec!["AI".into()],
            kpis: vec![],
        })).unwrap();

        let c = sys.read(&id).unwrap();
        assert_eq!(c.name(), "My Portfolio");
        assert!(c.is_item());
    }

    #[test]
    fn test_create_container_binder() {
        let mut sys = PortfolioSystem::new("node-1");
        let user = owner();
        let id = sys.create_container(user, "Design Binder", ContainerType::Binder(BinderData {
            description: "Holds design artefacts".into(),
            logic_tags: vec!["design".into()],
            source_items: vec![],
            dashboards: vec![],
        })).unwrap();

        let c = sys.read(&id).unwrap();
        assert!(c.is_container());
        assert_eq!(c.name(), "Design Binder");
    }

    #[test]
    fn test_action_permission() {
        let mut sys = PortfolioSystem::new("node-1");
        let user  = owner();
        let guest = owner();

        let id = sys.create_item(user, "Secret Item", ItemType::Asset(AssetPayload {
            asset_type: AssetType::Digital, valuation: None, currency: None, acquired_at: None,
        })).unwrap();

        let result = sys.act(guest, &id, ActionKind::Edit);
        assert!(matches!(result, Err(PortfolioError::PermissionDenied { .. })));

        let result = sys.act(user, &id, ActionKind::Edit);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parent_child_relation() {
        let mut sys = PortfolioSystem::new("node-1");
        let user = owner();

        let parent_id = sys.create_container(user, "Program Binder", ContainerType::Binder(BinderData {
            description: String::new(), logic_tags: vec![], source_items: vec![], dashboards: vec![],
        })).unwrap();

        let child_id = sys.create_item(user, "Child Project", ItemType::Project(ProjectPayload {
            project_type: ProjectType::Software, methodology: Some("Scrum".into()),
            start_date: None, end_date: None,
            sprints: vec![], backlog: vec![], releases: vec![],
        })).unwrap();

        sys.attach_child(parent_id, child_id).unwrap();

        let children = sys.children(&parent_id).unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id(), child_id);

        let child = sys.read(&child_id).unwrap();
        assert!(child.data().parents.contains(&parent_id));
    }

    #[test]
    fn test_cyclic_dependency_prevented() {
        let mut sys = PortfolioSystem::new("node-1");
        let user = owner();

        let a = sys.create_item(user, "A", ItemType::Resource(ResourcePayload {
            resource_type: ResourceType::Human, skills: vec![], availability: 1.0,
            hourly_rate: None, currency: None,
        })).unwrap();

        let b = sys.create_item(user, "B", ItemType::Resource(ResourcePayload {
            resource_type: ResourceType::Human, skills: vec![], availability: 1.0,
            hourly_rate: None, currency: None,
        })).unwrap();

        sys.add_dependency(a, b).unwrap();
        let result = sys.add_dependency(b, a);
        assert!(matches!(result, Err(PortfolioError::CyclicDependency(_, _))));
    }

    #[test]
    fn test_vector_clock_ordering() {
        let mut vc1 = VectorClock::new();
        let mut vc2 = VectorClock::new();

        vc1.tick("node-1");
        vc1.tick("node-1");
        vc2.merge(&vc1);
        vc2.tick("node-2");

        assert!(vc1.happened_before(&vc2));
        assert!(!vc2.happened_before(&vc1));
    }

    #[test]
    fn test_search() {
        let mut sys = PortfolioSystem::new("node-1");
        let user = owner();

        let id = sys.create_item(user, "AI Research Portfolio", ItemType::Portfolio(PortfolioItemPayload {
            mission: "Explore AI".into(), focus_areas: vec!["research".into()], kpis: vec![],
        })).unwrap();

        sys.tag(user, &id, "ai").unwrap();
        sys.publish(user, &id, Visibility::Public).unwrap();

        let results = sys.search(&SearchQuery {
            text: Some("AI".into()),
            visibility: Some(Visibility::Public),
            ..Default::default()
        });

        assert!(!results.is_empty());
        assert_eq!(results[0].component_id, id);
    }

    #[test]
    fn test_version_bump() {
        let mut sys = PortfolioSystem::new("node-1");
        let user = owner();
        let id = sys.create_item(user, "Versioned Item", ItemType::Artifact(ArtifactPayload {
            artifact_type: "document".into(), file_refs: vec![], produced_by: vec![],
        })).unwrap();

        let v  = sys.bump_version(user, &id, VersionPart::Minor, "Added section 2").unwrap();
        assert_eq!(v, "0.2.0");
        let v2 = sys.bump_version(user, &id, VersionPart::Patch, "Fixed typo").unwrap();
        assert_eq!(v2, "0.2.1");
        let v3 = sys.bump_version(user, &id, VersionPart::Major, "Breaking redesign").unwrap();
        assert_eq!(v3, "1.0.0");
    }

    #[test]
    fn test_analytics_engagement() {
        let mut sys = PortfolioSystem::new("node-1");
        let owner_id = owner();
        let viewer   = owner();

        let id = sys.create_item(owner_id, "Public Asset", ItemType::Asset(AssetPayload {
            asset_type: AssetType::Digital, valuation: None, currency: None, acquired_at: None,
        })).unwrap();

        sys.grant(owner_id, viewer, &id, PermissionTier::Subscriber).unwrap();
        sys.act(viewer, &id, ActionKind::Like).unwrap();
        sys.act(viewer, &id, ActionKind::Follow).unwrap();
        sys.record_view(&id, 120).unwrap();
        sys.record_click(&id).unwrap();

        let analytics = sys.analytics(&id).unwrap();
        assert_eq!(analytics.likes, 1);
        assert_eq!(analytics.followers, 1);
        assert_eq!(analytics.view_time_seconds, 120);
        assert_eq!(analytics.clicks, 1);
    }

    #[test]
    fn test_group_collection_list_schedule_directory() {
        let mut sys = PortfolioSystem::new("node-1");
        let user = owner();

        let item_id = sys.create_item(user, "Item A", ItemType::Artifact(ArtifactPayload {
            artifact_type: "doc".into(), file_refs: vec![], produced_by: vec![],
        })).unwrap();

        let gid = sys.create_group("Project Cluster", user);
        sys.group_mut(&gid).unwrap().add(item_id);
        assert_eq!(sys.group(&gid).unwrap().members.len(), 1);

        let cid = sys.create_collection("Favourites", user);
        sys.collection_mut(&cid).unwrap().insert(item_id);
        assert!(sys.collection(&cid).unwrap().contains(&item_id));

        let lid = sys.create_list("Sprint Backlog", user);
        sys.list_mut(&lid).unwrap().push(item_id);
        assert_eq!(sys.list(&lid).unwrap().items.len(), 1);

        let sid = sys.create_schedule("Q3 Roadmap", user);
        sys.schedule_mut(&sid).unwrap().add_entry(item_id, Utc::now());
        assert_eq!(sys.schedule(&sid).unwrap().entries.len(), 1);

        let did = sys.create_directory("Root", "/", user);
        sys.directory_mut(&did).unwrap().add_entry(item_id, "item-a", "portfolio_item");
        let entry = sys.directory(&did).unwrap().find(&item_id);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().name, "item-a");
    }

    #[test]
    fn test_builder() {
        let mut sys = PortfolioSystem::new("node-1");
        let user = owner();

        let id = ItemBuilder::project(user, "My SaaS App", ProjectType::Software)
            .description("A cloud-native portfolio tool.")
            .tag("saas")
            .tag("rust")
            .visibility(Visibility::Public)
            .status(ComponentStatus::Active)
            .build(&mut sys)
            .unwrap();

        let c = sys.read(&id).unwrap();
        assert_eq!(c.name(), "My SaaS App");
        assert_eq!(c.data().visibility, Visibility::Public);
        assert!(c.data().metadata.tags.contains("rust"));
    }

    #[test]
    fn test_snapshot_and_restore() {
        let mut sys = PortfolioSystem::new("node-1");
        let user = owner();

        let id = sys.create_item(user, "Versioned Asset", ItemType::Asset(AssetPayload {
            asset_type: AssetType::Digital, valuation: None, currency: None, acquired_at: None,
        })).unwrap();

        let snap = sys.save_snapshot(Some("before rename".into()));
        sys.update_info(user, &id, Some("Renamed Asset".into()), None).unwrap();
        assert_eq!(sys.read(&id).unwrap().name(), "Renamed Asset");

        sys.restore_snapshot(&snap.snapshot_id).unwrap();
        assert_eq!(sys.read(&id).unwrap().name(), "Versioned Asset");
    }

    #[test]
    fn test_resource_allocation() {
        let mut sys = PortfolioSystem::new("node-1");
        let user = owner();

        let id = sys.create_item(user, "Funded Project", ItemType::Project(ProjectPayload {
            project_type: ProjectType::Software, methodology: None,
            start_date: None, end_date: None,
            sprints: vec![], backlog: vec![], releases: vec![],
        })).unwrap();

        sys.allocate_budget(&id, 10_000.0, "USD", Some("Q1-2026".into())).unwrap();
        let remaining = sys.record_spend(&id, 3_500.0).unwrap();
        assert!((remaining - 6_500.0).abs() < 0.01);

        let alloc = sys.get_resource_allocation(&id).unwrap();
        assert!((alloc.utilisation_pct() - 35.0).abs() < 0.1);
        assert!(!alloc.is_overrun());
    }

    #[test]
    fn test_governance_approval() {
        let mut sys = PortfolioSystem::new("node-1");
        let user = owner();

        let id = sys.create_item(user, "Sensitive Item", ItemType::Artifact(ArtifactPayload {
            artifact_type: "contract".into(), file_refs: vec![], produced_by: vec![],
        })).unwrap();

        let req = sys.request_approval(id, "Requires legal sign-off");
        assert_eq!(req.status, ApprovalStatus::Pending);

        let resolved = sys.resolve_approval(&req.id, true, "legal-team", Some("Approved by counsel".into())).unwrap();
        assert_eq!(resolved.status, ApprovalStatus::Approved);
    }

    #[test]
    fn test_portfolio_health_model() {
        let mut sys = PortfolioSystem::new("node-1");
        let user = owner();

        let root = sys.create_item(user, "Root Portfolio", ItemType::Portfolio(PortfolioItemPayload {
            mission: "Root".into(), focus_areas: vec![], kpis: vec![],
        })).unwrap();

        for i in 0..5 {
            let child = sys.create_item(user, format!("Project {i}"), ItemType::Project(ProjectPayload {
                project_type: ProjectType::Software, methodology: None,
                start_date: None, end_date: None,
                sprints: vec![], backlog: vec![], releases: vec![],
            })).unwrap();
            sys.act(user, &child, ActionKind::Post { visibility: Some(Visibility::Public) }).unwrap();
            sys.attach_child(root, child).unwrap();
        }

        sys.allocate_budget(&root, 50_000.0, "USD", None).unwrap();
        sys.record_spend(&root, 20_000.0).unwrap();

        let health = sys.compute_portfolio_health(&root).unwrap();
        assert!(health.health_score > 50.0, "Expected healthy score, got {}", health.health_score);
    }

    #[test]
    fn test_federation_crdt_sync() {
        let mut fed = PortfolioFederation::new();
        fed.add_portfolio("alpha", PortfolioSystem::new("alpha"));
        fed.add_portfolio("beta",  PortfolioSystem::new("beta"));

        let alpha_user = owner();
        {
            let alpha = fed.get_portfolio_mut("alpha").unwrap();
            let id = alpha.create_item(alpha_user, "Shared Item", ItemType::Artifact(ArtifactPayload {
                artifact_type: "doc".into(), file_refs: vec![], produced_by: vec![],
            })).unwrap();
            alpha.crdt.append("alpha", CrdtOperation::SetField {
                component_id: id.to_string(),
                field: "name".into(),
                value: "Shared Item".into(),
                timestamp: now_epoch(),
                actor: "alpha".into(),
            });
        }

        fed.sync_crdt("alpha", "beta").unwrap();

        let beta = fed.get_portfolio("beta").unwrap();
        assert!(!beta.crdt.operations.is_empty());
    }
}
