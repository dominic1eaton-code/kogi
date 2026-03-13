// ============================================================================
//  portfolio.rs — Kogi OS · Portfolio System
//  Independent Worker Operating System
//
//  Full implementation of the Kogi Portfolio domain as described in the
//  Kogi OS System Design Document.
//
//  Hierarchy:
//    PortfolioSystem
//      └── Component (Item | Container)
//            ├── ComponentMetadata   (id, owners, tags, policy_ids,
//            │                        timestamps, vector_clock, properties,
//            │                        version)
//            ├── ComponentData       (type, category, name, status, state,
//            │                        relations, users, actions, analytics)
//            ├── component:item      Portfolio | Program | Project |
//            │                        Resource | Artifact | Asset
//            └── component:container Binder | Book(*) | Record | Folder |
//                                     Registry | Archive
//
//  Structural primitives:
//    Group      — linked components (sibling set)
//    Collection — unordered set of components
//    List       — ordered set of components
//    Schedule   — causal/time-ordered list of items
//    Directory  — spatial / hierarchical collection of items
// ============================================================================

use std::collections::{HashMap, HashSet};
use std::fmt;

use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────────────
// Re-export convenience
// ─────────────────────────────────────────────────────────────────────────────

pub type ComponentId   = Uuid;
pub type UserId        = Uuid;
pub type PolicyId      = Uuid;
pub type TagId         = Uuid;
pub type VersionString = String;
pub type Properties    = HashMap<String, serde_json::Value>;

// ─────────────────────────────────────────────────────────────────────────────
// Error
// ─────────────────────────────────────────────────────────────────────────────

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
            Self::NotFound(id)                  => write!(f, "Component not found: {id}"),
            Self::PermissionDenied { user_id, action } =>
                write!(f, "User {user_id} denied action '{action}'"),
            Self::InvalidOperation(msg)         => write!(f, "Invalid operation: {msg}"),
            Self::CyclicDependency(a, b)        => write!(f, "Cyclic dependency: {a} ↔ {b}"),
            Self::VersionConflict { local, remote } =>
                write!(f, "Version conflict: local={local} remote={remote}"),
            Self::AlreadyExists(id)             => write!(f, "Component already exists: {id}"),
            Self::InvalidState { current, attempted } =>
                write!(f, "Cannot '{attempted}' in state {current:?}"),
            Self::StorageError(msg)             => write!(f, "Storage error: {msg}"),
            Self::ValidationError(msg)          => write!(f, "Validation error: {msg}"),
        }
    }
}

pub type PortfolioResult<T> = Result<T, PortfolioError>;

// ─────────────────────────────────────────────────────────────────────────────
// Vector Clock  (distributed concurrency / versioning)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct VectorClock {
    /// node_id → logical timestamp
    pub clocks: HashMap<String, u64>,
}

impl VectorClock {
    pub fn new() -> Self { Self::default() }

    /// Increment the clock for a given node.
    pub fn increment(&mut self, node_id: &str) {
        let counter = self.clocks.entry(node_id.to_string()).or_insert(0);
        *counter += 1;
    }

    /// Merge with another vector clock, taking element-wise max.
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

// ─────────────────────────────────────────────────────────────────────────────
// Lifecycle Enums: Status · State · Visibility · Permission Tier
// ─────────────────────────────────────────────────────────────────────────────

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

/// Visibility / publication state (mirrors the `post` action).
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

// ─────────────────────────────────────────────────────────────────────────────
// Component Category
// ─────────────────────────────────────────────────────────────────────────────

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

// ─────────────────────────────────────────────────────────────────────────────
// Book Kinds
// ─────────────────────────────────────────────────────────────────────────────

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

// ─────────────────────────────────────────────────────────────────────────────
// ItemBook Data  (special container within an ItemBook)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ItemBookData {
    /// KPI / metrics dashboard
    pub dashboard: Option<Dashboard>,
    /// Strategic mandate / project charter
    pub charter: Option<Charter>,
    /// Active workspace (files, documents, content)
    pub workspace: Option<Workspace>,
    /// Searchable resource catalogue
    pub catalogue: Option<Catalogue>,
    /// Reusable templates, assets, plugins, workflows
    pub library: Option<Library>,
    /// Templates specific to this item
    pub templates: Vec<Template>,
    /// Audit and activity logs
    pub logs: Vec<ActivityLog>,
    /// KPI metrics and analytics
    pub metrics: Vec<Metric>,
    /// Version history
    pub version: Option<VersionHistory>,
    /// Calendars, milestones, timelines, roadmaps
    pub schedule: Option<ItemSchedule>,
    /// Hierarchical directory of sub-items
    pub directory: Option<Directory>,
}

// ─── Sub-structures of ItemBookData ─────────────────────────────────────────

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
pub enum RiskSeverity   { Critical, High, Medium, Low, Negligible }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RiskStatus     { Open, Mitigated, Accepted, Closed }

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
    Template,
    Plugin,
    Workflow,
    Snippet,
    Schema,
    Playbook,
    File,
    Custom(String),
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
    KPI,
    Counter,
    Gauge,
    Histogram,
    Rate,
    Ratio,
    Custom(String),
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

// ─────────────────────────────────────────────────────────────────────────────
// Actions
// ─────────────────────────────────────────────────────────────────────────────

/// Every interaction a user can perform on a component.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionKind {
    // Social
    Like,
    Comment,
    Subscribe,
    Follow,
    Unfollow,
    Watch,
    Bookmark,
    Save,
    Share,
    Report,
    Invite,

    // Content
    Post { visibility: Option<Visibility> },
    Tag { label: Option<String> },
    Mention { target: Option<UserId> },
    Label { value: Option<String> },
    Hashtag { topic: Option<String> },

    // Interaction
    Poll,
    Survey,
    Contribute,
    Join,
    Leave,
    Campaign,
    Donate { amount: Option<u64> },
    Invest { amount: Option<u64> },

    // Discovery
    Search,
    Filter,
    Index,

    // Ownership / Access
    Own { tier: Option<PermissionTier> },
    Edit,

    // CRUD
    Create,
    Read,
    Update,
    Delete,
    Archive,
    Restore,
    Duplicate,
    Move { target_container: Option<ComponentId> },
    Link { target: Option<ComponentId> },
    Unlink { target: Option<ComponentId> },

    Custom(String),
}

impl ActionKind {
    /// Returns the minimum permission tier required to perform this action.
    pub fn required_permission(&self) -> PermissionTier {
        match self {
            ActionKind::Read | ActionKind::Search | ActionKind::Filter => PermissionTier::Viewer,
            ActionKind::Like | ActionKind::Follow | ActionKind::Unfollow
            | ActionKind::Subscribe | ActionKind::Watch | ActionKind::Save
            | ActionKind::Bookmark => PermissionTier::Subscriber,
            ActionKind::Comment | ActionKind::Poll | ActionKind::Survey
            | ActionKind::Contribute | ActionKind::Join | ActionKind::Leave
            | ActionKind::Donate { .. } | ActionKind::Invest { .. } => PermissionTier::Contributor,
            ActionKind::Edit | ActionKind::Create | ActionKind::Update
            | ActionKind::Tag { .. } | ActionKind::Label { .. } | ActionKind::Hashtag { .. }
            | ActionKind::Mention { .. } | ActionKind::Post { .. } | ActionKind::Index
            | ActionKind::Campaign | ActionKind::Invite | ActionKind::Share => PermissionTier::Editor,
            ActionKind::Archive | ActionKind::Restore | ActionKind::Duplicate
            | ActionKind::Move { .. } | ActionKind::Link { .. } | ActionKind::Unlink { .. } => PermissionTier::Manager,
            ActionKind::Delete | ActionKind::Own { .. } | ActionKind::Report => PermissionTier::Owner,
            ActionKind::Custom(_) => PermissionTier::Contributor,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Analytics
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComponentAnalytics {
    // Traffic
    pub clicks: u64,
    pub click_through_rate: f64,
    pub view_time_seconds: u64,
    pub impressions: u64,

    // Engagement counts
    pub likes: u64,
    pub comments: u64,
    pub shares: u64,
    pub saves: u64,
    pub bookmarks: u64,
    pub reactions: HashMap<String, u64>,

    /// Total engagements / total followers or reach
    pub engagement_rate: f64,

    /// Number of active hashtags, tags, mentions, etc. across the platform
    pub spread: u64,

    // Growth rates (values represent % change per period)
    pub follower_growth_rate: f64,
    pub subscriber_growth_rate: f64,
    pub watcher_growth_rate: f64,

    // Audience counts
    pub followers: u64,
    pub subscribers: u64,
    pub watchers: u64,

    // Comparison / benchmarking
    /// Scores relative to other portfolios / components (0.0 – 1.0)
    pub benchmark_score: f64,
    /// Percentile rank among peers
    pub peer_percentile: Option<f64>,
    /// Key comparative KPIs
    pub comparison_kpis: HashMap<String, f64>,

    // Virality
    /// Reposts / forwards
    pub reposts: u64,
    /// External referrals
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
            ActionKind::Like     => self.likes += 1,
            ActionKind::Comment  => self.comments += 1,
            ActionKind::Share    => { self.shares += 1; self.reposts += 1; }
            ActionKind::Save     => self.saves += 1,
            ActionKind::Bookmark => self.bookmarks += 1,
            ActionKind::Follow   => { self.followers += 1; }
            ActionKind::Subscribe => { self.subscribers += 1; }
            ActionKind::Watch    => { self.watchers += 1; }
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
        let total_engagements = self.likes + self.comments + self.shares + self.saves;
        let reach = self.followers.max(1);
        self.engagement_rate = total_engagements as f64 / reach as f64;
    }

    pub fn total_engagements(&self) -> u64 {
        self.likes + self.comments + self.shares + self.saves + self.bookmarks
            + self.reactions.values().sum::<u64>()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// User Roles on a Component
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComponentUsers {
    /// Full ownership (highest trust tier)
    pub owners: HashSet<UserId>,
    /// Can modify content
    pub editors: HashSet<UserId>,
    /// Monitoring without editing rights
    pub watchers: HashSet<UserId>,
    /// Opted-in to updates
    pub subscribers: HashSet<UserId>,
    /// Social following
    pub followers: HashSet<UserId>,
    /// Financial investors
    pub investors: HashSet<UserId>,
    /// Donors / gift contributors
    pub donors: HashSet<UserId>,
    /// Extended permissions mapping: user_id → tier
    pub permission_map: HashMap<UserId, PermissionTier>,
}

impl ComponentUsers {
    pub fn add_user(&mut self, user_id: UserId, tier: PermissionTier) {
        self.permission_map.insert(user_id, tier.clone());
        match tier {
            PermissionTier::Owner | PermissionTier::Admin => { self.owners.insert(user_id); }
            PermissionTier::Editor | PermissionTier::Manager => { self.editors.insert(user_id); }
            PermissionTier::Viewer                          => {}
            PermissionTier::Subscriber                     => { self.subscribers.insert(user_id); }
            PermissionTier::Contributor                    => { self.followers.insert(user_id); }
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

// ─────────────────────────────────────────────────────────────────────────────
// Component Metadata
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetadata {
    pub id: ComponentId,
    /// List of owning user IDs
    pub owners: Vec<UserId>,
    /// Flat tag strings (e.g. "rust", "ai", "q3-2025")
    pub tags: HashSet<String>,
    /// IDs of governance / compliance policies attached to this component
    pub policy_ids: Vec<PolicyId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Distributed version tracking
    pub vector_clock: VectorClock,
    /// Arbitrary key-value extension properties
    pub properties: Properties,
    /// Semantic version string
    pub version: VersionString,
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
        }
    }

    pub fn touch(&mut self, node_id: &str) {
        self.updated_at = Utc::now();
        self.vector_clock.increment(node_id);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Component Data
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentData {
    pub metadata: ComponentMetadata,
    /// Category discriminates Item vs Container and its sub-type
    pub category: ComponentCategory,
    pub name: String,
    pub description: String,
    pub status: ComponentStatus,
    pub state: ComponentState,
    pub visibility: Visibility,

    // ── Relations ────────────────────────────────────────────────────────────
    /// Child components (contained-by relationship)
    pub children: Vec<ComponentId>,
    /// Parent components (contained-in relationship)
    pub parents: Vec<ComponentId>,
    /// Sibling / linked components (peer group)
    pub links: Vec<ComponentId>,
    /// Components that depend on this one
    pub dependents: Vec<ComponentId>,
    /// Components this one depends on
    pub dependencies: Vec<ComponentId>,

    // ── People ───────────────────────────────────────────────────────────────
    pub users: ComponentUsers,

    // ── Behaviour ────────────────────────────────────────────────────────────
    /// Allowed actions on this component (empty = all allowed per tier)
    pub allowed_actions: HashSet<String>,
    /// Action event history
    pub action_history: Vec<ActivityLog>,

    // ── Analytics ────────────────────────────────────────────────────────────
    pub analytics: ComponentAnalytics,

    // ── Governance ───────────────────────────────────────────────────────────
    pub risks: Vec<Risk>,
    pub governance_notes: Vec<String>,
    pub compliance_flags: HashMap<String, bool>,

    // ── Hashtags / Topics ────────────────────────────────────────────────────
    pub hashtags: HashSet<String>,
    pub topics: HashSet<String>,

    // ── Extension ────────────────────────────────────────────────────────────
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
        meta.owners.push(owner);

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

        // Record analytics
        self.analytics.record_engagement(&action);

        // Record in action history
        self.action_history.push(ActivityLog {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            actor_id: actor,
            action: action.clone(),
            target_id: Some(self.id()),
            description: format!("{actor} performed {action:?}"),
            metadata: HashMap::new(),
        });

        // Touch metadata
        self.metadata.touch(node_id);

        // Handle specific action side-effects
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
                if let Some(v) = visibility {
                    self.visibility = v.clone();
                }
                self.status = ComponentStatus::Active;
            }
            ActionKind::Unfollow => {
                self.analytics.followers = self.analytics.followers.saturating_sub(1);
            }
            ActionKind::Hashtag { topic: Some(t) } => { self.hashtags.insert(t.clone()); self.analytics.spread += 1; }
            ActionKind::Label { value: Some(l) }   => { self.hashtags.insert(l.clone()); }
            ActionKind::Tag { label: Some(lbl) }   => { self.metadata.tags.insert(lbl.clone()); }
            ActionKind::Donate { .. } => {
                self.users.donors.insert(actor);
            }
            ActionKind::Invest { .. } => {
                self.users.investors.insert(actor);
            }
            ActionKind::Join => {
                self.users.add_user(actor, PermissionTier::Contributor);
            }
            ActionKind::Leave => {
                self.users.remove_user(&actor);
            }
            _ => {}
        }

        Ok(())
    }

    /// Add a child component ID.
    pub fn add_child(&mut self, child_id: ComponentId) {
        if !self.children.contains(&child_id) {
            self.children.push(child_id);
        }
    }

    /// Remove a child component ID.
    pub fn remove_child(&mut self, child_id: &ComponentId) {
        self.children.retain(|id| id != child_id);
    }

    /// Add a dependency, checking for potential cycles (shallow check).
    pub fn add_dependency(
        &mut self,
        dep_id: ComponentId,
    ) -> PortfolioResult<()> {
        if dep_id == self.id() {
            return Err(PortfolioError::CyclicDependency(self.id(), dep_id));
        }
        if self.dependents.contains(&dep_id) {
            return Err(PortfolioError::CyclicDependency(self.id(), dep_id));
        }
        if !self.dependencies.contains(&dep_id) {
            self.dependencies.push(dep_id);
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Item — leaf portfolio entities
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub data: ComponentData,
    pub item_type: ItemType,
    pub item_book: Option<ItemBookData>,
}

/// Fine-grained item type carrying domain-specific payloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemType {
    /// The portfolio itself (recursive: a portfolio is a portfolio item)
    Portfolio(PortfolioItemPayload),
    /// A multi-project umbrella
    Program(ProgramPayload),
    /// An individual project (agile, creative, technical, etc.)
    Project(ProjectPayload),
    /// A human or machine resource
    Resource(ResourcePayload),
    /// A produced deliverable / output
    Artifact(ArtifactPayload),
    /// A valuable item (IP, financial instrument, physical asset)
    Asset(AssetPayload),
    Custom(String),
}

// ─── Item Payloads ───────────────────────────────────────────────────────────

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
    Organizational,
    Creative,
    Technical,
    Research,
    AI,
    Software,
    Media,
    Marketing,
    Investment,
    ContentCreator,
    DIY,
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
    IntellectualProperty,
    Financial,
    Physical,
    Digital,
    Creative,
    Custom(String),
}

// ─────────────────────────────────────────────────────────────────────────────
// Container — organising / grouping entities
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    pub data: ComponentData,
    pub container_type: ContainerType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContainerType {
    /// Collections of items organised by logic / theme
    Binder(BinderData),
    /// Typed book with specialised sub-structures
    Book(Book),
    /// A structured record (log entry, form, artefact record)
    Record(RecordData),
    /// Filesystem-style folder
    Folder(FolderData),
    /// Named registry / index
    Registry(RegistryData),
    /// Deep storage, full restore support
    Archive(ArchiveData),
    Custom(String),
}

// ─── Container Payloads ──────────────────────────────────────────────────────

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

// ─────────────────────────────────────────────────────────────────────────────
// Structural Primitives: Group · Collection · List · Schedule · Directory
// ─────────────────────────────────────────────────────────────────────────────

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
            id: Uuid::new_v4(),
            name: name.into(),
            description: String::new(),
            members: Vec::new(),
            group_type: "default".into(),
            owner,
            created_at: Utc::now(),
            updated_at: Utc::now(),
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
            id: Uuid::new_v4(),
            name: name.into(),
            description: String::new(),
            items: HashSet::new(),
            tags: HashSet::new(),
            owner,
            created_at: Utc::now(),
            updated_at: Utc::now(),
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
            id: Uuid::new_v4(),
            name: name.into(),
            description: String::new(),
            items: Vec::new(),
            owner,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    pub fn push(&mut self, id: ComponentId)    { self.items.push(id); }
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
            id: Uuid::new_v4(),
            name: name.into(),
            description: String::new(),
            entries: Vec::new(),
            owner,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    pub fn add_entry(&mut self, component_id: ComponentId, scheduled_at: DateTime<Utc>) {
        let order = self.entries.len() as u32;
        self.entries.push(ScheduleEntry {
            id: Uuid::new_v4(),
            component_id,
            scheduled_at,
            duration_minutes: None,
            recurrence: None,
            notes: String::new(),
            order,
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
            id: Uuid::new_v4(),
            name: name.into(),
            description: String::new(),
            path: path.into(),
            entries: Vec::new(),
            sub_directories: Vec::new(),
            owner,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn add_entry(&mut self, component_id: ComponentId, name: impl Into<String>, entry_type: &str) {
        let n = name.into();
        let path = format!("{}/{}", self.path.trim_end_matches('/'), n);
        let order = self.entries.len() as u32;
        self.entries.push(DirectoryEntry {
            id: Uuid::new_v4(),
            component_id,
            name: n,
            entry_type: entry_type.to_string(),
            path,
            order,
            created_at: Utc::now(),
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

    /// Recursively find a path.
    pub fn resolve_path(&self, path: &str) -> Option<&DirectoryEntry> {
        let parts: Vec<&str> = path.trim_start_matches('/').splitn(2, '/').collect();
        match parts.as_slice() {
            [leaf] => self.entries.iter().find(|e| e.name == *leaf),
            [dir, rest] => self
                .sub_directories.iter()
                .find(|d| d.name == *dir)
                .and_then(|d| d.resolve_path(rest)),
            _ => None,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Unified Component Enum
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Component {
    Item(Item),
    Container(Container),
}

impl Component {
    pub fn data(&self) -> &ComponentData {
        match self {
            Component::Item(i)      => &i.data,
            Component::Container(c) => &c.data,
        }
    }

    pub fn data_mut(&mut self) -> &mut ComponentData {
        match self {
            Component::Item(i)      => &mut i.data,
            Component::Container(c) => &mut c.data,
        }
    }

    pub fn id(&self)   -> ComponentId  { self.data().id() }
    pub fn name(&self) -> &str         { &self.data().name }

    pub fn is_item(&self)      -> bool { matches!(self, Component::Item(_)) }
    pub fn is_container(&self) -> bool { matches!(self, Component::Container(_)) }

    pub fn as_item(&self)      -> Option<&Item>      { if let Component::Item(i) = self { Some(i) } else { None } }
    pub fn as_container(&self) -> Option<&Container> { if let Component::Container(c) = self { Some(c) } else { None } }
}

// ─────────────────────────────────────────────────────────────────────────────
// Search / Filter
// ─────────────────────────────────────────────────────────────────────────────

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

// ─────────────────────────────────────────────────────────────────────────────
// PortfolioSystem — central registry and orchestrator
// ─────────────────────────────────────────────────────────────────────────────

/// The root system that owns and manages all portfolio components,
/// structural primitives, and cross-cutting concerns.
pub struct PortfolioSystem {
    /// Component registry: id → component
    components: HashMap<ComponentId, Component>,

    /// Structural primitives
    groups:      HashMap<Uuid, Group>,
    collections: HashMap<Uuid, Collection>,
    lists:       HashMap<Uuid, List>,
    schedules:   HashMap<Uuid, Schedule>,
    directories: HashMap<Uuid, Directory>,

    /// Node identity for vector clock increments
    node_id: String,

    /// Global activity log (cross-component events)
    global_log: Vec<ActivityLog>,
}

impl PortfolioSystem {
    // ── Construction ─────────────────────────────────────────────────────────

    pub fn new(node_id: impl Into<String>) -> Self {
        Self {
            components:  HashMap::new(),
            groups:      HashMap::new(),
            collections: HashMap::new(),
            lists:       HashMap::new(),
            schedules:   HashMap::new(),
            directories: HashMap::new(),
            node_id:     node_id.into(),
            global_log:  Vec::new(),
        }
    }

    // ── Component CRUD ───────────────────────────────────────────────────────

    /// Create and register a new Item component.
    pub fn create_item(
        &mut self,
        owner: UserId,
        name: impl Into<String>,
        item_type: ItemType,
    ) -> PortfolioResult<ComponentId> {
        let category = match &item_type {
            ItemType::Portfolio(_) => ComponentCategory::Item(ItemCategory::Portfolio),
            ItemType::Program(_)   => ComponentCategory::Item(ItemCategory::Program),
            ItemType::Project(_)   => ComponentCategory::Item(ItemCategory::Project),
            ItemType::Resource(_)  => ComponentCategory::Item(ItemCategory::Resource),
            ItemType::Artifact(_)  => ComponentCategory::Item(ItemCategory::Artifact),
            ItemType::Asset(_)     => ComponentCategory::Item(ItemCategory::Asset),
            ItemType::Custom(s)    => ComponentCategory::Item(ItemCategory::Custom(s.clone())),
        };

        let data = ComponentData::new(owner, name, category);
        let id   = data.id();

        self.components.insert(id, Component::Item(Item {
            data,
            item_type,
            item_book: None,
        }));

        self.log_global(owner, ActionKind::Create, Some(id), "Item created");
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
            ContainerType::Binder(_)  => ComponentCategory::Container(ContainerCategory::Binder),
            ContainerType::Book(b)    => ComponentCategory::Container(ContainerCategory::Book(b.kind.clone())),
            ContainerType::Record(_)  => ComponentCategory::Container(ContainerCategory::Record),
            ContainerType::Folder(_)  => ComponentCategory::Container(ContainerCategory::Folder),
            ContainerType::Registry(_)=> ComponentCategory::Container(ContainerCategory::Registry),
            ContainerType::Archive(_) => ComponentCategory::Container(ContainerCategory::Archive),
            ContainerType::Custom(s)  => ComponentCategory::Container(ContainerCategory::Custom(s.clone())),
        };

        let data = ComponentData::new(owner, name, category);
        let id   = data.id();

        self.components.insert(id, Component::Container(Container { data, container_type }));
        self.log_global(owner, ActionKind::Create, Some(id), "Container created");
        Ok(id)
    }

    /// Read (immutable reference to) a component by ID.
    pub fn read(&self, id: &ComponentId) -> PortfolioResult<&Component> {
        self.components.get(id).ok_or(PortfolioError::NotFound(*id))
    }

    /// Read (mutable reference to) a component by ID.
    pub fn read_mut(&mut self, id: &ComponentId) -> PortfolioResult<&mut Component> {
        self.components.get_mut(id).ok_or(PortfolioError::NotFound(*id))
    }

    /// Update a component's name and description.
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
            return Err(PortfolioError::PermissionDenied {
                user_id: actor,
                action: "Edit".into(),
            });
        }
        if let Some(n) = name        { data.name = n; }
        if let Some(d) = description { data.description = d; }
        data.metadata.touch(&node);
        Ok(())
    }

    /// Delete a component (marks deleted; does not physically remove by default).
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
                    user_id: actor,
                    action: "Delete".into(),
                });
            }
            data.status = ComponentStatus::Deleted;
            data.metadata.touch(&node);
        }
        if hard_delete {
            self.components.remove(id);
        }
        self.log_global(actor, ActionKind::Delete, Some(*id), "Component deleted");
        Ok(())
    }

    // ── Actions ──────────────────────────────────────────────────────────────

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

    // ── Lifecycle ────────────────────────────────────────────────────────────

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

    // ── Relations ────────────────────────────────────────────────────────────

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
        // Guard: child should not already be an ancestor
        {
            let parent = self.read(&parent_id)?;
            if parent.data().parents.contains(&child_id) {
                return Err(PortfolioError::CyclicDependency(parent_id, child_id));
            }
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
        Ok(())
    }

    /// Remove a parent→child relationship.
    pub fn detach_child(
        &mut self,
        parent_id: ComponentId,
        child_id: ComponentId,
    ) -> PortfolioResult<()> {
        let node = self.node_id.clone();
        {
            let p = self.read_mut(&parent_id)?;
            p.data_mut().remove_child(&child_id);
            p.data_mut().metadata.touch(&node);
        }
        {
            let c = self.read_mut(&child_id)?;
            c.data_mut().parents.retain(|id| id != &parent_id);
            c.data_mut().metadata.touch(&node);
        }
        Ok(())
    }

    /// Link two components as siblings (bidirectional).
    pub fn link(
        &mut self,
        a: ComponentId,
        b: ComponentId,
    ) -> PortfolioResult<()> {
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
        Ok(())
    }

    /// Register a dependency: `from` depends on `to`.
    pub fn add_dependency(
        &mut self,
        from: ComponentId,
        to: ComponentId,
    ) -> PortfolioResult<()> {
        let node = self.node_id.clone();
        // Check that `from` is not already in `to`'s dependency chain
        if self.transitive_dependencies(&to)?.contains(&from) {
            return Err(PortfolioError::CyclicDependency(from, to));
        }
        {
            let cf = self.read_mut(&from)?;
            cf.data_mut().add_dependency(to)?;
            cf.data_mut().metadata.touch(&node);
        }
        {
            let ct = self.read_mut(&to)?;
            if !ct.data().dependents.contains(&from) {
                ct.data_mut().dependents.push(from);
            }
            ct.data_mut().metadata.touch(&node);
        }
        Ok(())
    }

    /// Collect all transitive dependency IDs (depth-first).
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

    // ── User / Permission Management ─────────────────────────────────────────

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
                user_id: actor,
                action: "grant permission".into(),
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
                user_id: actor,
                action: "revoke permission".into(),
            });
        }
        data.users.remove_user(&target_user);
        data.metadata.touch(&node);
        Ok(())
    }

    // ── Tagging / Hashtags ────────────────────────────────────────────────────

    pub fn tag(
        &mut self,
        actor: UserId,
        id: &ComponentId,
        tag: impl Into<String>,
    ) -> PortfolioResult<()> {
        let t = tag.into();
        self.act(actor, id, ActionKind::Tag { label: Some(t) })
    }

    pub fn hashtag(
        &mut self,
        actor: UserId,
        id: &ComponentId,
        topic: impl Into<String>,
    ) -> PortfolioResult<()> {
        let t = topic.into();
        self.act(actor, id, ActionKind::Hashtag { topic: Some(t) })
    }

    // ── Search & Filter ──────────────────────────────────────────────────────

    pub fn search(&self, query: &SearchQuery) -> Vec<SearchResult> {
        let mut results: Vec<SearchResult> = self
            .components
            .values()
            .filter(|c| {
                let data = c.data();
                // Visibility filter (only show non-deleted to anonymous)
                if data.status == ComponentStatus::Deleted { return false; }
                // Status filter
                if !query.statuses.is_empty() && !query.statuses.contains(&data.status) {
                    return false;
                }
                // Owner filter
                if let Some(owner) = &query.owner {
                    if !data.metadata.owners.contains(owner) { return false; }
                }
                // Visibility filter
                if let Some(vis) = &query.visibility {
                    if &data.visibility != vis { return false; }
                }
                // Date filters
                if let Some(after) = query.created_after {
                    if data.metadata.created_at < after { return false; }
                }
                if let Some(before) = query.created_before {
                    if data.metadata.created_at > before { return false; }
                }
                // Category filter
                if !query.categories.is_empty() && !query.categories.contains(&data.category) {
                    return false;
                }
                // Tag filter
                if !query.tags.is_empty() {
                    let has_all = query.tags.iter().all(|t| data.metadata.tags.contains(t));
                    if !has_all { return false; }
                }
                // Hashtag filter
                if !query.hashtags.is_empty() {
                    let has_all = query.hashtags.iter().all(|h| data.hashtags.contains(h));
                    if !has_all { return false; }
                }
                true
            })
            .map(|c| {
                let data = c.data();
                let mut score: f64 = 1.0;
                let mut matched = Vec::new();

                if let Some(text) = &query.text {
                    let tl = text.to_lowercase();
                    if data.name.to_lowercase().contains(&tl) {
                        score += 3.0;
                        matched.push("name".to_string());
                    }
                    if data.description.to_lowercase().contains(&tl) {
                        score += 1.0;
                        matched.push("description".to_string());
                    }
                    let tag_hit = data.metadata.tags.iter().any(|t| t.to_lowercase().contains(&tl));
                    if tag_hit { score += 1.5; matched.push("tags".to_string()); }
                    let hash_hit = data.hashtags.iter().any(|h| h.to_lowercase().contains(&tl));
                    if hash_hit { score += 1.5; matched.push("hashtags".to_string()); }
                }

                // Boost by engagement
                score += data.analytics.engagement_rate * 0.5;

                SearchResult {
                    component_id: data.id(),
                    name: data.name.clone(),
                    category: data.category.clone(),
                    status: data.status.clone(),
                    score,
                    matched_fields: matched,
                }
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        let offset = query.offset;
        let limited: Vec<SearchResult> = results
            .into_iter()
            .skip(offset)
            .take(query.limit.unwrap_or(usize::MAX))
            .collect();

        limited
    }

    // ── Analytics ────────────────────────────────────────────────────────────

    pub fn record_view(
        &mut self,
        id: &ComponentId,
        duration_seconds: u64,
    ) -> PortfolioResult<()> {
        let c = self.components.get_mut(id).ok_or(PortfolioError::NotFound(*id))?;
        c.data_mut().analytics.record_view(duration_seconds);
        Ok(())
    }

    pub fn record_click(&mut self, id: &ComponentId) -> PortfolioResult<()> {
        let c = self.components.get_mut(id).ok_or(PortfolioError::NotFound(*id))?;
        c.data_mut().analytics.record_click();
        Ok(())
    }

    /// Return a snapshot of analytics for a component.
    pub fn analytics(&self, id: &ComponentId) -> PortfolioResult<&ComponentAnalytics> {
        Ok(&self.read(id)?.data().analytics)
    }

    /// Compare analytics across a set of components (benchmarking).
    pub fn benchmark(
        &self,
        ids: &[ComponentId],
    ) -> HashMap<ComponentId, ComponentAnalytics> {
        ids.iter()
            .filter_map(|id| self.read(id).ok().map(|c| (*id, c.data().analytics.clone())))
            .collect()
    }

    // ── Structural Primitive CRUD ────────────────────────────────────────────

    // Groups
    pub fn create_group(&mut self, name: impl Into<String>, owner: UserId) -> Uuid {
        let g = Group::new(name, owner);
        let id = g.id;
        self.groups.insert(id, g);
        id
    }
    pub fn group(&self, id: &Uuid) -> Option<&Group> { self.groups.get(id) }
    pub fn group_mut(&mut self, id: &Uuid) -> Option<&mut Group> { self.groups.get_mut(id) }

    // Collections
    pub fn create_collection(&mut self, name: impl Into<String>, owner: UserId) -> Uuid {
        let c = Collection::new(name, owner);
        let id = c.id;
        self.collections.insert(id, c);
        id
    }
    pub fn collection(&self, id: &Uuid) -> Option<&Collection> { self.collections.get(id) }
    pub fn collection_mut(&mut self, id: &Uuid) -> Option<&mut Collection> { self.collections.get_mut(id) }

    // Lists
    pub fn create_list(&mut self, name: impl Into<String>, owner: UserId) -> Uuid {
        let l = List::new(name, owner);
        let id = l.id;
        self.lists.insert(id, l);
        id
    }
    pub fn list(&self, id: &Uuid) -> Option<&List> { self.lists.get(id) }
    pub fn list_mut(&mut self, id: &Uuid) -> Option<&mut List> { self.lists.get_mut(id) }

    // Schedules
    pub fn create_schedule(&mut self, name: impl Into<String>, owner: UserId) -> Uuid {
        let s = Schedule::new(name, owner);
        let id = s.id;
        self.schedules.insert(id, s);
        id
    }
    pub fn schedule(&self, id: &Uuid) -> Option<&Schedule> { self.schedules.get(id) }
    pub fn schedule_mut(&mut self, id: &Uuid) -> Option<&mut Schedule> { self.schedules.get_mut(id) }

    // Directories
    pub fn create_directory(
        &mut self,
        name: impl Into<String>,
        path: impl Into<String>,
        owner: UserId,
    ) -> Uuid {
        let d = Directory::new(name, path, owner);
        let id = d.id;
        self.directories.insert(id, d);
        id
    }
    pub fn directory(&self, id: &Uuid) -> Option<&Directory> { self.directories.get(id) }
    pub fn directory_mut(&mut self, id: &Uuid) -> Option<&mut Directory> { self.directories.get_mut(id) }

    // ── Queries ──────────────────────────────────────────────────────────────

    /// List all component IDs owned by a user.
    pub fn components_by_owner(&self, owner: &UserId) -> Vec<ComponentId> {
        self.components
            .values()
            .filter(|c| c.data().metadata.owners.contains(owner))
            .map(|c| c.id())
            .collect()
    }

    /// List all components with a given tag.
    pub fn components_by_tag(&self, tag: &str) -> Vec<ComponentId> {
        self.components
            .values()
            .filter(|c| c.data().metadata.tags.contains(tag))
            .map(|c| c.id())
            .collect()
    }

    /// List all components by status.
    pub fn components_by_status(&self, status: &ComponentStatus) -> Vec<ComponentId> {
        self.components
            .values()
            .filter(|c| &c.data().status == status)
            .map(|c| c.id())
            .collect()
    }

    /// Return children of a component, sorted by name.
    pub fn children(&self, id: &ComponentId) -> PortfolioResult<Vec<&Component>> {
        let parent = self.read(id)?;
        let mut children: Vec<&Component> = parent
            .data()
            .children
            .iter()
            .filter_map(|cid| self.components.get(cid))
            .collect();
        children.sort_by_key(|c| c.name());
        Ok(children)
    }

    /// Return the full ancestor chain of a component (breadth-first).
    pub fn ancestors(&self, id: &ComponentId) -> PortfolioResult<Vec<ComponentId>> {
        let mut result = Vec::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(*id);
        let mut seen = HashSet::new();

        while let Some(current) = queue.pop_front() {
            if !seen.insert(current) { continue; }
            let c = self.read(&current)?;
            for parent_id in &c.data().parents {
                if !seen.contains(parent_id) {
                    result.push(*parent_id);
                    queue.push_back(*parent_id);
                }
            }
        }
        Ok(result)
    }

    // ── Version Control ───────────────────────────────────────────────────────

    /// Increment the semantic version of a component.
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
                user_id: actor,
                action: "bump_version".into(),
            });
        }

        let new_version = increment_version(&data.metadata.version, part);
        data.metadata.version = new_version.clone();
        data.metadata.touch(&node);

        // Append to ItemBook version history if available
        if let Component::Item(item) = c {
            if let Some(book) = &mut item.item_book {
                let history = book.version.get_or_insert(VersionHistory {
                    current_version: new_version.clone(),
                    entries: Vec::new(),
                });
                history.current_version = new_version.clone();
                history.entries.push(VersionEntry {
                    version: new_version.clone(),
                    author: actor,
                    message: message.into(),
                    diff_ref: None,
                    tags: Vec::new(),
                    created_at: Utc::now(),
                });
            }
        }

        Ok(new_version)
    }

    // ── Global Log ───────────────────────────────────────────────────────────

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

    pub fn audit_trail(&self, id: &ComponentId) -> PortfolioResult<Vec<&ActivityLog>> {
        let c = self.read(id)?;
        let logs: Vec<&ActivityLog> = self
            .global_log
            .iter()
            .chain(c.data().action_history.iter())
            .filter(|l| l.target_id.as_ref() == Some(id))
            .collect();
        Ok(logs)
    }

    // ── Metrics ───────────────────────────────────────────────────────────────

    /// Count of all components in the system.
    pub fn total_components(&self) -> usize { self.components.len() }

    /// Count of items only.
    pub fn total_items(&self) -> usize {
        self.components.values().filter(|c| c.is_item()).count()
    }

    /// Count of containers only.
    pub fn total_containers(&self) -> usize {
        self.components.values().filter(|c| c.is_container()).count()
    }

    /// Aggregate engagement across all components.
    pub fn platform_engagement(&self) -> u64 {
        self.components
            .values()
            .map(|c| c.data().analytics.total_engagements())
            .sum()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Version helpers
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum VersionPart { Major, Minor, Patch }

fn increment_version(version: &str, part: VersionPart) -> String {
    let parts: Vec<u64> = version
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

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

// ─────────────────────────────────────────────────────────────────────────────
// Builder — fluent construction helpers for common patterns
// ─────────────────────────────────────────────────────────────────────────────

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
            owner,
            name: name.into(),
            description: String::new(),
            item_type: ItemType::Portfolio(PortfolioItemPayload {
                mission: String::new(),
                focus_areas: Vec::new(),
                kpis: Vec::new(),
            }),
            tags: HashSet::new(),
            visibility: Visibility::Private,
            status: ComponentStatus::Draft,
        }
    }

    pub fn project(owner: UserId, name: impl Into<String>, project_type: ProjectType) -> Self {
        Self {
            owner,
            name: name.into(),
            description: String::new(),
            item_type: ItemType::Project(ProjectPayload {
                project_type,
                methodology: None,
                start_date: None,
                end_date: None,
                sprints: Vec::new(),
                backlog: Vec::new(),
                releases: Vec::new(),
            }),
            tags: HashSet::new(),
            visibility: Visibility::Private,
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

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn owner() -> UserId { Uuid::new_v4() }

    #[test]
    fn test_create_item_and_read() {
        let mut sys = PortfolioSystem::new("node-1");
        let user = owner();
        let id = sys
            .create_item(user, "My Portfolio", ItemType::Portfolio(PortfolioItemPayload {
                mission: "Test mission".into(),
                focus_areas: vec!["AI".into()],
                kpis: vec![],
            }))
            .unwrap();

        let c = sys.read(&id).unwrap();
        assert_eq!(c.name(), "My Portfolio");
        assert!(c.is_item());
    }

    #[test]
    fn test_create_container_binder() {
        let mut sys = PortfolioSystem::new("node-1");
        let user = owner();
        let id = sys
            .create_container(user, "Design Binder", ContainerType::Binder(BinderData {
                description: "Holds design artefacts".into(),
                logic_tags: vec!["design".into()],
                source_items: vec![],
                dashboards: vec![],
            }))
            .unwrap();

        let c = sys.read(&id).unwrap();
        assert!(c.is_container());
        assert_eq!(c.name(), "Design Binder");
    }

    #[test]
    fn test_action_permission() {
        let mut sys = PortfolioSystem::new("node-1");
        let user  = owner();
        let guest = owner();

        let id = sys
            .create_item(user, "Secret Item", ItemType::Asset(AssetPayload {
                asset_type: AssetType::Digital,
                valuation: None,
                currency: None,
                acquired_at: None,
            }))
            .unwrap();

        // Guest (no permission) cannot edit
        let result = sys.act(guest, &id, ActionKind::Edit);
        assert!(matches!(result, Err(PortfolioError::PermissionDenied { .. })));

        // Owner can edit
        let result = sys.act(user, &id, ActionKind::Edit);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parent_child_relation() {
        let mut sys = PortfolioSystem::new("node-1");
        let user = owner();

        let parent_id = sys
            .create_container(user, "Program Binder", ContainerType::Binder(BinderData {
                description: String::new(),
                logic_tags: vec![],
                source_items: vec![],
                dashboards: vec![],
            }))
            .unwrap();

        let child_id = sys
            .create_item(user, "Child Project", ItemType::Project(ProjectPayload {
                project_type: ProjectType::Software,
                methodology: Some("Scrum".into()),
                start_date: None,
                end_date: None,
                sprints: vec![],
                backlog: vec![],
                releases: vec![],
            }))
            .unwrap();

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

        sys.add_dependency(a, b).unwrap();           // A depends on B  ✓
        let result = sys.add_dependency(b, a);       // B depends on A  ✗ (cycle)
        assert!(matches!(result, Err(PortfolioError::CyclicDependency(_, _))));
    }

    #[test]
    fn test_vector_clock_ordering() {
        let mut vc1 = VectorClock::new();
        let mut vc2 = VectorClock::new();

        vc1.increment("node-1");
        vc1.increment("node-1");
        vc2.merge(&vc1);
        vc2.increment("node-2");

        assert!(vc1.happened_before(&vc2));
        assert!(!vc2.happened_before(&vc1));
    }

    #[test]
    fn test_search() {
        let mut sys = PortfolioSystem::new("node-1");
        let user = owner();

        let id = sys
            .create_item(user, "AI Research Portfolio", ItemType::Portfolio(PortfolioItemPayload {
                mission: "Explore AI".into(),
                focus_areas: vec!["research".into()],
                kpis: vec![],
            }))
            .unwrap();

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
        let id = sys
            .create_item(user, "Versioned Item", ItemType::Artifact(ArtifactPayload {
                artifact_type: "document".into(),
                file_refs: vec![],
                produced_by: vec![],
            }))
            .unwrap();

        let v = sys.bump_version(user, &id, VersionPart::Minor, "Added section 2").unwrap();
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

        let id = sys
            .create_item(owner_id, "Public Asset", ItemType::Asset(AssetPayload {
                asset_type: AssetType::Digital,
                valuation: None,
                currency: None,
                acquired_at: None,
            }))
            .unwrap();

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

        // Group
        let gid = sys.create_group("Project Cluster", user);
        let item_id = sys.create_item(user, "Item A", ItemType::Artifact(ArtifactPayload {
            artifact_type: "doc".into(), file_refs: vec![], produced_by: vec![],
        })).unwrap();
        sys.group_mut(&gid).unwrap().add(item_id);
        assert_eq!(sys.group(&gid).unwrap().members.len(), 1);

        // Collection
        let cid = sys.create_collection("Favourites", user);
        sys.collection_mut(&cid).unwrap().insert(item_id);
        assert!(sys.collection(&cid).unwrap().contains(&item_id));

        // List
        let lid = sys.create_list("Sprint Backlog", user);
        sys.list_mut(&lid).unwrap().push(item_id);
        assert_eq!(sys.list(&lid).unwrap().items.len(), 1);

        // Schedule
        let sid = sys.create_schedule("Q3 Roadmap", user);
        sys.schedule_mut(&sid).unwrap().add_entry(item_id, Utc::now());
        assert_eq!(sys.schedule(&sid).unwrap().entries.len(), 1);

        // Directory
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
}