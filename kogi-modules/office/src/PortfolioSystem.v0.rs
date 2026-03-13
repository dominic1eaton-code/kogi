// =============================================================================
//  PortfolioSystem.rs  —  Kogi OS · Portfolio System  (unified module)
//
//  Merges:
//    • PortfolioSystem.rs  — runtime infrastructure (event sourcing, CRDT,
//      snapshots, governance, federation, analytics models, PQL, plugin API)
//    • portfolio.rs        — rich domain types (users, permissions, analytics,
//      item/container payloads, social actions, structural primitives)
//
//  Sections:
//    §1   Imports & utilities
//    §2   Error types
//    §3   Primitive enums  (Status · State · Visibility · PermissionTier)
//    §4   Component taxonomy  (PortfolioComponentType · BookType · Category)
//    §5   ComponentMetadata  (id · owners · tags · policy_ids · vector_clock …)
//    §6   Social actions  (ActionKind · PermissionTier hierarchy)
//    §7   Component users  (owners · editors · watchers · subscribers …)
//    §8   Component analytics  (CTR · engagement · spread · benchmarking)
//    §9   Rich domain payloads  (Item* · Container* sub-structures)
//    §10  ItemBookData  (Dashboard · Charter · Workspace · Catalogue …)
//    §11  PortfolioComponent  (unified base)
//    §12  Structural primitives  (Group · Collection · List · Schedule · Directory)
//    §13  Graph layer  (GraphEdgeKind · GraphEdge · GraphView)
//    §14  Event sourcing  (PortfolioEventKind · PortfolioEvent · EventLog)
//    §15  Snapshot & checkpoint
//    §16  CRDT  (VectorClock · LwwField · OrSet · CrdtOperation · CrdtLog)
//    §17  Governance  (ResourceKind · PolicyEngine · ApprovalWorkflow · ResourceAllocation)
//    §18  Plugin traits
//    §19  Federation  (FederationPeer · PortfolioFederation)
//    §20  Query language  (PortfolioQuery / PQL · SearchQuery)
//    §21  PortfolioSystem  (main runtime — CRUD · graph · snapshots · time-travel …)
//    §22  OS-bridge adapter  (NewPortfolioItem — backward-compat shim, inlined)
//    §23  Computational models  (per-component-type analytics & scoring)
//    §24  Builder helpers  (ItemBuilder)
//    §25  Tests
// =============================================================================

#![allow(dead_code)]

// =============================================================================
// §1 — IMPORTS & UTILITIES
// =============================================================================

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

// ── ID aliases ────────────────────────────────────────────────────────────────

/// String-keyed component identity (e.g. `"comp-000042"`).
pub type ComponentId   = String;
/// String-keyed user / actor identity.
pub type ActorId       = String;
/// Arbitrary key→value property bag.
pub type Properties    = HashMap<String, String>;
/// SemVer-style version string.
pub type VersionString = String;

// ── Time helper ───────────────────────────────────────────────────────────────

/// Unix epoch seconds.
#[inline]
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Generate a unique tag for OR-Set entries.
fn unique_tag(actor: &str, ts: u64, seq: u64) -> String {
    format!("{actor}:{ts}:{seq}")
}

// =============================================================================
// §2 — ERROR TYPES
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PortfolioError {
    NotFound(ComponentId),
    PermissionDenied { actor: ActorId, action: String },
    InvalidOperation(String),
    CyclicRelation(ComponentId, ComponentId),
    VersionConflict { local: VersionString, remote: VersionString },
    AlreadyExists(ComponentId),
    InvalidState { current: String, attempted: String },
    PolicyDenied(String),
    StorageError(String),
    ValidationError(String),
}

impl fmt::Display for PortfolioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(id)                 => write!(f, "Component not found: {id}"),
            Self::PermissionDenied { actor, action } =>
                write!(f, "Actor '{actor}' denied action '{action}'"),
            Self::InvalidOperation(m)          => write!(f, "Invalid operation: {m}"),
            Self::CyclicRelation(a, b)         => write!(f, "Cyclic relation: {a} ↔ {b}"),
            Self::VersionConflict { local, remote } =>
                write!(f, "Version conflict: local={local} remote={remote}"),
            Self::AlreadyExists(id)            => write!(f, "Already exists: {id}"),
            Self::InvalidState { current, attempted } =>
                write!(f, "Cannot '{attempted}' in state '{current}'"),
            Self::PolicyDenied(r)              => write!(f, "Policy denied: {r}"),
            Self::StorageError(m)              => write!(f, "Storage error: {m}"),
            Self::ValidationError(m)           => write!(f, "Validation error: {m}"),
        }
    }
}

pub type PortfolioResult<T> = Result<T, PortfolioError>;

// =============================================================================
// §3 — PRIMITIVE ENUMS
// =============================================================================

/// High-level lifecycle status of any component.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

impl Default for ComponentStatus {
    fn default() -> Self { Self::Draft }
}

impl fmt::Display for ComponentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Custom(s) => write!(f, "{s}"),
            other           => write!(f, "{other:?}"),
        }
    }
}

/// Operational / runtime state — finer-grained than `ComponentStatus`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

impl Default for ComponentState {
    fn default() -> Self { Self::Initializing }
}

/// Publication / visibility state.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Visibility {
    Private,
    Protected,
    Public,
    Unlisted,
    DraftOnly,
}

impl Default for Visibility {
    fn default() -> Self { Self::Private }
}

/// Permission tier hierarchy.  Higher tier ⊇ all lower-tier privileges.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PermissionTier {
    Viewer,
    Subscriber,
    Contributor,
    Editor,
    Manager,
    Owner,
    Admin,
}

// =============================================================================
// §4 — COMPONENT TAXONOMY
// =============================================================================

/// Every first-class object type in the portfolio system.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortfolioComponentType {
    // ── Items ─────────────────────────────────────────────────────────────
    Portfolio,
    Project,
    Program,
    Resource,
    Asset,
    Artifact,
    SubPortfolio,
    // ── Containers ────────────────────────────────────────────────────────
    /// Unordered, logic-tagged collection of heterogeneous components.
    Binder,
    /// Typed document container; sub-type in `book_type`.
    Book,
    /// Hierarchical filesystem-style container (name-ordered).
    Folder,
    /// Ordered, sequential list of components.
    Record,
    /// Named index / catalogue.
    Registry,
    /// Deep storage with full-restore support.
    Archive,
}

impl PortfolioComponentType {
    pub fn is_container(&self) -> bool {
        matches!(
            self,
            Self::Binder | Self::Book | Self::Folder
            | Self::Record | Self::Registry | Self::Archive
        )
    }
    pub fn is_item(&self) -> bool { !self.is_container() }
}

/// Sub-types for `PortfolioComponentType::Book`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BookType {
    /// General notes / wiki pages.
    Notebook,
    /// Runbooks, procedures, tactical playbooks.
    Playbook,
    /// People, stakeholders, organisations.
    Contactbook,
    /// Timelines, milestones, calendar events.
    Schedulebook,
    /// Goals, strategies, initiatives.
    Planbook,
    /// Documentation sets (guides, references).
    Guidebook,
    /// Living dossier for one portfolio item (charter, workspace, metrics …).
    Itembook,
    Custom(String),
}

/// Project methodology / style.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProjectType {
    Organisational,
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

/// Resource granularity type.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceSubtype {
    Human,
    Machine,
    Service,
    License,
    Custom(String),
}

/// Asset class.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetSubtype {
    IntellectualProperty,
    Financial,
    Physical,
    Digital,
    Creative,
    Custom(String),
}

// =============================================================================
// §5 — COMPONENT METADATA
// =============================================================================

/// Canonical metadata carried by every `PortfolioComponent`.
///
/// Covers: version control, unique identity, resource tracking, governance,
/// searchability, and distributed causality (vector clock).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComponentMetadata {
    /// Globally unique component identifier (e.g. `"comp-000042"`).
    pub id: ComponentId,
    /// Monotonic integer version; incremented on every mutation.
    pub version: u64,
    /// Epoch-seconds of first creation.
    pub created_at: u64,
    /// Epoch-seconds of last mutation.
    pub updated_at: u64,
    /// Actor (user / peer) that last wrote this component.
    pub actor_id: ActorId,
    /// All owning actors (first is creator/primary owner).
    pub owners: Vec<ActorId>,
    /// Causal vector clock: actor → logical timestamp.
    pub vector_clock: HashMap<ActorId, u64>,
    /// Flat string tags for search and cross-referencing.
    pub tags: Vec<String>,
    /// Open-ended key→value property bag.
    pub properties: Properties,
    // ── Governance ────────────────────────────────────────────────────────
    /// IDs of governance/compliance policies attached to this component.
    pub policy_ids: Vec<String>,
    // ── Resource management ───────────────────────────────────────────────
    /// Allocated budget (arbitrary unit — interpret at application layer).
    pub budget: Option<f64>,
    /// Consumed budget so far.
    pub budget_spent: f64,
    /// Generic resource units (person-hours, story-points, etc.).
    pub resource_units: f64,
    /// Semantic version string.
    pub semver: VersionString,
}

impl ComponentMetadata {
    pub fn new(id: impl Into<String>, actor_id: impl Into<String>) -> Self {
        let ts = now();
        let actor: ActorId = actor_id.into();
        Self {
            id: id.into(),
            version: 1,
            created_at: ts,
            updated_at: ts,
            actor_id: actor.clone(),
            owners: vec![actor],
            vector_clock: HashMap::new(),
            tags: Vec::new(),
            properties: HashMap::new(),
            policy_ids: Vec::new(),
            budget: None,
            budget_spent: 0.0,
            resource_units: 0.0,
            semver: "0.1.0".to_string(),
        }
    }

    /// Advance the clock for `actor_id` and update timestamps.
    pub fn touch(&mut self, actor_id: &str) {
        self.version += 1;
        self.updated_at = now();
        self.actor_id = actor_id.to_string();
        let t = self.vector_clock.entry(actor_id.to_string()).or_insert(0);
        *t += 1;
    }

    /// Merge a remote metadata update using last-write-wins semantics.
    pub fn merge_clock(&mut self, remote: &HashMap<ActorId, u64>) {
        for (actor, &ts) in remote {
            let e = self.vector_clock.entry(actor.clone()).or_insert(0);
            if ts > *e { *e = ts; }
        }
    }
}

// =============================================================================
// §6 — SOCIAL ACTIONS
// =============================================================================

/// Every interaction a user / actor can perform on a component.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionKind {
    // Social ──────────────────────────────────────────────────────────────
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
    // Content ─────────────────────────────────────────────────────────────
    Post,
    Tag,
    Mention,
    Label,
    Hashtag,
    // Interaction ─────────────────────────────────────────────────────────
    Poll,
    Survey,
    Contribute,
    Join,
    Leave,
    Campaign,
    Donate,
    Invest,
    // Discovery ───────────────────────────────────────────────────────────
    Search,
    Filter,
    Index,
    // Ownership / access ──────────────────────────────────────────────────
    Own,
    Edit,
    // CRUD ────────────────────────────────────────────────────────────────
    Create,
    Read,
    Update,
    Delete,
    Archive,
    Restore,
    Duplicate,
    Move,
    Link,
    Unlink,
    Custom(String),
}

impl ActionKind {
    /// Minimum `PermissionTier` required to perform this action.
    pub fn required_tier(&self) -> PermissionTier {
        match self {
            Self::Read | Self::Search | Self::Filter => PermissionTier::Viewer,
            Self::Like | Self::Follow | Self::Unfollow | Self::Subscribe
            | Self::Watch | Self::Save | Self::Bookmark => PermissionTier::Subscriber,
            Self::Comment | Self::Poll | Self::Survey | Self::Contribute
            | Self::Join | Self::Leave | Self::Donate | Self::Invest => PermissionTier::Contributor,
            Self::Edit | Self::Create | Self::Update | Self::Tag | Self::Label
            | Self::Hashtag | Self::Mention | Self::Post | Self::Index
            | Self::Campaign | Self::Invite | Self::Share => PermissionTier::Editor,
            Self::Archive | Self::Restore | Self::Duplicate | Self::Move
            | Self::Link | Self::Unlink => PermissionTier::Manager,
            Self::Delete | Self::Own | Self::Report => PermissionTier::Owner,
            Self::Custom(_) => PermissionTier::Contributor,
        }
    }
}

/// A recorded action event attached to a component.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionEvent {
    pub id: String,
    pub timestamp: u64,
    pub actor_id: ActorId,
    pub action: ActionKind,
    pub target_id: Option<ComponentId>,
    pub description: String,
    pub metadata: Properties,
}

impl ActionEvent {
    pub fn new(actor_id: impl Into<String>, action: ActionKind, target_id: Option<ComponentId>, desc: &str) -> Self {
        Self {
            id: format!("act-{}", now()),
            timestamp: now(),
            actor_id: actor_id.into(),
            action,
            target_id,
            description: desc.to_string(),
            metadata: HashMap::new(),
        }
    }
}

// =============================================================================
// §7 — COMPONENT USERS (role roster)
// =============================================================================

/// Per-component user roster with tiered permissions.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ComponentUsers {
    pub owners:      HashSet<ActorId>,
    pub editors:     HashSet<ActorId>,
    pub watchers:    HashSet<ActorId>,
    pub subscribers: HashSet<ActorId>,
    pub followers:   HashSet<ActorId>,
    pub investors:   HashSet<ActorId>,
    pub donors:      HashSet<ActorId>,
    /// Extended map: actor_id → permission tier.
    pub permission_map: HashMap<ActorId, PermissionTier>,
}

impl ComponentUsers {
    pub fn new(owner: impl Into<ActorId>) -> Self {
        let mut u = Self::default();
        u.add(owner.into(), PermissionTier::Owner);
        u
    }

    pub fn add(&mut self, actor_id: ActorId, tier: PermissionTier) {
        self.permission_map.insert(actor_id.clone(), tier.clone());
        match tier {
            PermissionTier::Admin | PermissionTier::Owner => { self.owners.insert(actor_id); }
            PermissionTier::Manager | PermissionTier::Editor => { self.editors.insert(actor_id); }
            PermissionTier::Subscriber => { self.subscribers.insert(actor_id); }
            PermissionTier::Contributor => { self.followers.insert(actor_id); }
            PermissionTier::Viewer => {}
        }
    }

    pub fn remove(&mut self, actor_id: &str) {
        self.owners.remove(actor_id);
        self.editors.remove(actor_id);
        self.watchers.remove(actor_id);
        self.subscribers.remove(actor_id);
        self.followers.remove(actor_id);
        self.investors.remove(actor_id);
        self.donors.remove(actor_id);
        self.permission_map.remove(actor_id);
    }

    pub fn tier(&self, actor_id: &str) -> Option<&PermissionTier> {
        self.permission_map.get(actor_id)
    }

    pub fn can(&self, actor_id: &str, action: &ActionKind) -> bool {
        match self.tier(actor_id) {
            None        => *action == ActionKind::Read,
            Some(tier)  => tier >= &action.required_tier(),
        }
    }

    pub fn is_owner(&self, actor_id: &str) -> bool { self.owners.contains(actor_id) }
}

// =============================================================================
// §8 — COMPONENT ANALYTICS
// =============================================================================

/// Platform analytics attached to every component.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ComponentAnalytics {
    // ── Traffic ───────────────────────────────────────────────────────────
    pub clicks:               u64,
    pub click_through_rate:   f64,
    pub view_time_seconds:    u64,
    pub impressions:          u64,
    // ── Engagement counts ────────────────────────────────────────────────
    pub likes:       u64,
    pub comments:    u64,
    pub shares:      u64,
    pub saves:       u64,
    pub bookmarks:   u64,
    pub reposts:     u64,
    pub reactions:   HashMap<String, u64>,
    /// Total engagements / total followers (or reach).
    pub engagement_rate: f64,
    /// Number of active hashtags, tags, mentions across the platform.
    pub spread: u64,
    // ── Audience ─────────────────────────────────────────────────────────
    pub followers:   u64,
    pub subscribers: u64,
    pub watchers:    u64,
    // ── Growth rates (% change per period) ───────────────────────────────
    pub follower_growth_rate:    f64,
    pub subscriber_growth_rate:  f64,
    pub watcher_growth_rate:     f64,
    // ── Benchmarking ─────────────────────────────────────────────────────
    /// Percentile rank among peers (0.0–100.0).
    pub peer_percentile:  Option<f64>,
    /// Key KPIs used for user/portfolio comparison.
    pub comparison_kpis: HashMap<String, f64>,
    // ── Period ────────────────────────────────────────────────────────────
    pub period_start: Option<u64>,
    pub period_end:   Option<u64>,
    pub last_updated: Option<u64>,
}

impl ComponentAnalytics {
    pub fn record_click(&mut self) {
        self.clicks += 1;
        self.impressions += 1;
        if self.impressions > 0 {
            self.click_through_rate = self.clicks as f64 / self.impressions as f64;
        }
    }

    pub fn record_view(&mut self, duration_seconds: u64) {
        self.view_time_seconds += duration_seconds;
        self.impressions += 1;
    }

    pub fn record_action(&mut self, action: &ActionKind) {
        match action {
            ActionKind::Like       => self.likes += 1,
            ActionKind::Comment    => self.comments += 1,
            ActionKind::Share      => { self.shares += 1; self.reposts += 1; }
            ActionKind::Save       => self.saves += 1,
            ActionKind::Bookmark   => self.bookmarks += 1,
            ActionKind::Follow     => self.followers += 1,
            ActionKind::Unfollow   => self.followers = self.followers.saturating_sub(1),
            ActionKind::Subscribe  => self.subscribers += 1,
            ActionKind::Watch      => self.watchers += 1,
            ActionKind::Hashtag | ActionKind::Tag | ActionKind::Mention => self.spread += 1,
            _ => {}
        }
        let total = self.likes + self.comments + self.shares + self.saves + self.bookmarks;
        let reach = self.followers.max(1);
        self.engagement_rate = total as f64 / reach as f64;
        self.last_updated = Some(now());
    }

    pub fn total_engagements(&self) -> u64 {
        self.likes + self.comments + self.shares + self.saves + self.bookmarks
            + self.reactions.values().sum::<u64>()
    }
}

// =============================================================================
// §9 — RICH DOMAIN PAYLOADS
// =============================================================================

// ── Shared sub-structures ─────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileRef {
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub path: String,
    pub size_bytes: u64,
    pub checksum: String,
    pub version: VersionString,
    pub tags: Vec<String>,
    pub uploaded_by: ActorId,
    pub uploaded_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Risk {
    pub id: String,
    pub description: String,
    pub severity: RiskSeverity,
    pub probability: f32,
    pub mitigation: String,
    pub owner: Option<ActorId>,
    pub status: RiskStatus,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RiskSeverity { Critical, High, Medium, Low, Negligible }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RiskStatus { Open, Mitigated, Accepted, Closed }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KpiMetric {
    pub id: String,
    pub name: String,
    pub metric_type: KpiMetricType,
    pub value: f64,
    pub unit: String,
    pub period_start: Option<u64>,
    pub period_end: Option<u64>,
    pub tags: Vec<String>,
    pub recorded_at: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum KpiMetricType { KPI, Counter, Gauge, Rate, Ratio, Histogram, Custom(String) }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionEntry {
    pub version: VersionString,
    pub author: ActorId,
    pub message: String,
    pub diff_ref: Option<String>,
    pub tags: Vec<String>,
    pub created_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionHistory {
    pub current_version: VersionString,
    pub entries: Vec<VersionEntry>,
}

// ── Item-level payloads ───────────────────────────────────────────────────────

/// Extended payload for `PortfolioComponentType::Portfolio`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PortfolioPayload {
    pub mission: String,
    pub focus_areas: Vec<String>,
    pub kpis: Vec<KpiMetric>,
}

/// Extended payload for `PortfolioComponentType::Program`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProgramPayload {
    pub objective: String,
    pub linked_project_ids: Vec<ComponentId>,
    pub budget: Option<f64>,
    pub currency: Option<String>,
}

/// Extended payload for `PortfolioComponentType::Project`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectPayload {
    pub project_type: Option<ProjectType>,
    pub methodology: Option<String>,
    pub start_epoch: Option<u64>,
    pub end_epoch: Option<u64>,
    pub sprints: Vec<Sprint>,
    pub backlog: Vec<BacklogItem>,
    pub releases: Vec<Release>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sprint {
    pub id: String,
    pub name: String,
    pub goal: String,
    pub start_epoch: u64,
    pub end_epoch: u64,
    pub status: ComponentStatus,
    pub item_ids: Vec<String>,
    pub velocity: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BacklogItem {
    pub id: String,
    pub item_type: BacklogItemType,
    pub title: String,
    pub description: String,
    pub priority: u8,
    pub estimate: Option<f64>,
    pub status: ComponentStatus,
    pub assignee: Option<ActorId>,
    pub sprint_id: Option<String>,
    pub tags: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BacklogItemType {
    Feature, Requirement, Risk, UseCase, BusinessCase, Innovation,
    Test, Bug, Defect, Blocker, Enhancement, Task,
    Release, Report, Audit, Operation, Strategy, Plan,
    Custom(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Release {
    pub id: String,
    pub version: VersionString,
    pub description: String,
    pub release_epoch: Option<u64>,
    pub status: ComponentStatus,
    pub item_ids: Vec<String>,
}

/// Extended payload for `PortfolioComponentType::Resource`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ResourcePayload {
    pub subtype: Option<ResourceSubtype>,
    pub skills: Vec<String>,
    pub availability: f64,
    pub hourly_rate: Option<f64>,
    pub currency: Option<String>,
}

/// Extended payload for `PortfolioComponentType::Artifact`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ArtifactPayload {
    pub artifact_kind: String,
    pub file_refs: Vec<FileRef>,
    pub produced_by: Vec<ComponentId>,
}

/// Extended payload for `PortfolioComponentType::Asset`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AssetPayload {
    pub subtype: Option<AssetSubtype>,
    pub valuation: Option<f64>,
    pub currency: Option<String>,
    pub acquired_epoch: Option<u64>,
}

// ── Container-level payloads ──────────────────────────────────────────────────

/// Extended payload for `PortfolioComponentType::Binder`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BinderPayload {
    pub description: String,
    pub logic_tags: Vec<String>,
}

/// Extended payload for `PortfolioComponentType::Registry`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RegistryPayload {
    pub registry_type: String,
    pub schema: Option<serde_json::Value>,
    pub entries: Vec<RegistryEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub id: String,
    pub key: String,
    pub value: serde_json::Value,
    pub tags: Vec<String>,
    pub linked_component: Option<ComponentId>,
    pub registered_at: u64,
    pub updated_at: u64,
}

/// Extended payload for `PortfolioComponentType::Archive`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ArchivePayload {
    pub entries: Vec<ArchiveEntry>,
    pub compression: Option<String>,
    pub encrypted: bool,
    pub retention_policy: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchiveEntry {
    pub id: String,
    pub original_id: ComponentId,
    pub snapshot_json: serde_json::Value,
    pub version: VersionString,
    pub archived_by: ActorId,
    pub archived_at: u64,
    pub restore_key: Option<String>,
}

/// Unified optional extended payload.  Only the relevant variant is populated.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ComponentPayload {
    Portfolio(PortfolioPayload),
    Program(ProgramPayload),
    Project(ProjectPayload),
    Resource(ResourcePayload),
    Artifact(ArtifactPayload),
    Asset(AssetPayload),
    Binder(BinderPayload),
    Registry(RegistryPayload),
    Archive(ArchivePayload),
}

// =============================================================================
// §10 — ITEM BOOK DATA
// =============================================================================

/// Full dossier attached to any `Itembook`-typed `Book` component.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ItemBookData {
    pub dashboard: Option<BookDashboard>,
    pub charter: Option<BookCharter>,
    pub workspace: Option<BookWorkspace>,
    pub catalogue: Option<BookCatalogue>,
    pub library: Option<BookLibrary>,
    pub templates: Vec<BookTemplate>,
    pub logs: Vec<ActionEvent>,
    pub metrics: Vec<KpiMetric>,
    pub version: Option<VersionHistory>,
    pub schedule: Option<BookSchedule>,
    pub directory: Option<BookDirectory>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BookDashboard {
    pub id: String,
    pub name: String,
    pub widgets: Vec<DashboardWidget>,
    pub layout: serde_json::Value,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DashboardWidget {
    pub id: String,
    pub widget_type: String,
    pub title: String,
    pub config: serde_json::Value,
    pub position: (u32, u32),
    pub size: (u32, u32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BookCharter {
    pub id: String,
    pub executive_summary: String,
    pub objectives: Vec<String>,
    pub scope: String,
    pub stakeholders: Vec<ActorId>,
    pub success_criteria: Vec<String>,
    pub constraints: Vec<String>,
    pub assumptions: Vec<String>,
    pub risks: Vec<Risk>,
    pub version: VersionString,
    pub approved_by: Vec<ActorId>,
    pub approved_at: Option<u64>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BookWorkspace {
    pub id: String,
    pub name: String,
    pub files: Vec<FileRef>,
    pub content_blocks: Vec<ContentBlock>,
    pub connected_item_ids: Vec<ComponentId>,
    pub plugin_ids: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContentBlock {
    pub id: String,
    pub block_type: String,
    pub content: serde_json::Value,
    pub order: u32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BookCatalogue {
    pub id: String,
    pub name: String,
    pub description: String,
    pub entries: Vec<CatalogueEntry>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CatalogueEntry {
    pub id: String,
    pub item_id: ComponentId,
    pub tags: Vec<String>,
    pub searchable_text: String,
    pub metadata: Properties,
    pub added_at: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BookLibrary {
    pub id: String,
    pub name: String,
    pub assets: Vec<LibraryAsset>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LibraryAsset {
    pub id: String,
    pub name: String,
    pub asset_type: LibraryAssetType,
    pub content_ref: String,
    pub version: VersionString,
    pub tags: Vec<String>,
    pub created_at: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LibraryAssetType {
    Template, Plugin, Workflow, Snippet, Schema, Playbook, File, Custom(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BookTemplate {
    pub id: String,
    pub name: String,
    pub template_type: String,
    pub schema: serde_json::Value,
    pub default_values: serde_json::Value,
    pub version: VersionString,
    pub created_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BookSchedule {
    pub id: String,
    pub name: String,
    pub milestones: Vec<Milestone>,
    pub calendar_events: Vec<CalendarEvent>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Milestone {
    pub id: String,
    pub name: String,
    pub description: String,
    pub due_epoch: u64,
    pub completed_epoch: Option<u64>,
    pub status: ComponentStatus,
    pub linked_ids: Vec<ComponentId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: String,
    pub title: String,
    pub event_type: String,
    pub start_epoch: u64,
    pub end_epoch: u64,
    pub recurrence: Option<String>,
    pub participants: Vec<ActorId>,
    pub location: Option<String>,
    pub virtual_link: Option<String>,
}

/// Hierarchical spatial directory (path-addressable).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BookDirectory {
    pub id: String,
    pub name: String,
    pub path: String,
    pub entries: Vec<DirectoryEntry>,
    pub sub_directories: Vec<BookDirectory>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DirectoryEntry {
    pub id: String,
    pub component_id: ComponentId,
    pub name: String,
    pub entry_type: String,
    pub path: String,
    pub order: u32,
    pub created_at: u64,
}

impl BookDirectory {
    pub fn add_entry(&mut self, component_id: ComponentId, name: impl Into<String>, entry_type: &str) {
        let n = name.into();
        let path = format!("{}/{}", self.path.trim_end_matches('/'), n);
        let order = self.entries.len() as u32;
        self.entries.push(DirectoryEntry {
            id: format!("dent-{}", now()),
            component_id,
            name: n,
            entry_type: entry_type.to_string(),
            path,
            order,
            created_at: now(),
        });
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
// §11 — PORTFOLIO COMPONENT  (unified base)
// =============================================================================

/// The universal base type for every portfolio object.
///
/// # Container semantics
/// | Type     | Ordering | Deduplication |
/// |----------|----------|---------------|
/// | Binder   | none     | yes (HashSet) |
/// | Record   | strict   | no (Vec)      |
/// | Folder   | by-name  | yes (HashSet) |
/// | Book     | loose    | yes (HashSet) |
/// | Registry | none     | yes (HashSet) |
/// | Archive  | strict   | no (Vec)      |
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortfolioComponent {
    pub metadata: ComponentMetadata,
    pub component_type: PortfolioComponentType,
    /// Optional sub-type for `Book` components.
    pub book_type: Option<BookType>,
    pub name: String,
    pub description: String,

    // ── Status / state ────────────────────────────────────────────────────
    pub status: ComponentStatus,
    pub state: ComponentState,
    pub visibility: Visibility,

    // ── Graph relationships ───────────────────────────────────────────────
    /// Directed hierarchy: IDs of direct children.
    pub children: HashSet<ComponentId>,
    /// Directed hierarchy: IDs of direct parents.
    pub parents: HashSet<ComponentId>,
    /// Bidirectional peer links (non-hierarchical).
    pub links: HashSet<ComponentId>,
    /// This component depends on these IDs.
    pub dependencies: HashSet<ComponentId>,
    /// These IDs depend on this component.
    pub dependents: HashSet<ComponentId>,

    // ── Container membership ─────────────────────────────────────────────
    /// Unordered member IDs (Binder, Folder, Book, Registry).
    pub unordered_members: HashSet<ComponentId>,
    /// Strictly-ordered member IDs (Record, Archive).
    pub ordered_members: Vec<ComponentId>,

    // ── People / permissions ──────────────────────────────────────────────
    pub users: ComponentUsers,

    // ── Analytics / engagement ────────────────────────────────────────────
    pub analytics: ComponentAnalytics,

    // ── Social / discovery ────────────────────────────────────────────────
    pub hashtags: HashSet<String>,
    pub topics: HashSet<String>,
    pub action_history: Vec<ActionEvent>,

    // ── Governance ────────────────────────────────────────────────────────
    pub risks: Vec<Risk>,
    pub compliance_flags: HashMap<String, bool>,

    // ── Extended domain payload ───────────────────────────────────────────
    pub payload: Option<ComponentPayload>,

    // ── ItemBook (rich dossier — populated when book_type == Itembook) ────
    pub item_book: Option<ItemBookData>,

    // ── Legacy container refs (os_bridge backward compat) ─────────────────
    pub container_refs: Vec<LegacyContainerRef>,
}

/// Backward-compatible legacy container ref (replaces the old `PortfolioItemContainerType` vec).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LegacyContainerRef(pub String);

impl PortfolioComponent {
    // ── Constructors ──────────────────────────────────────────────────────

    pub fn new(
        id: impl Into<String>,
        actor_id: impl Into<String>,
        component_type: PortfolioComponentType,
        name: impl Into<String>,
    ) -> Self {
        let id: String = id.into();
        let actor: ActorId = actor_id.into();
        Self {
            metadata: ComponentMetadata::new(&id, &actor),
            component_type,
            book_type: None,
            name: name.into(),
            description: String::new(),
            status: ComponentStatus::Active,
            state: ComponentState::Configured,
            visibility: Visibility::Private,
            children: HashSet::new(),
            parents: HashSet::new(),
            links: HashSet::new(),
            dependencies: HashSet::new(),
            dependents: HashSet::new(),
            unordered_members: HashSet::new(),
            ordered_members: Vec::new(),
            users: ComponentUsers::new(actor),
            analytics: ComponentAnalytics::default(),
            hashtags: HashSet::new(),
            topics: HashSet::new(),
            action_history: Vec::new(),
            risks: Vec::new(),
            compliance_flags: HashMap::new(),
            payload: None,
            item_book: None,
            container_refs: Vec::new(),
        }
    }

    // ── Type predicates ───────────────────────────────────────────────────

    pub fn is_container(&self) -> bool { self.component_type.is_container() }
    pub fn is_item(&self) -> bool { self.component_type.is_item() }

    pub fn id(&self) -> &str { &self.metadata.id }

    // ── Member management ────────────────────────────────────────────────

    /// Add a member; dispatches to ordered or unordered based on type.
    pub fn add_member(&mut self, id: impl Into<ComponentId>) {
        let id = id.into();
        if matches!(
            self.component_type,
            PortfolioComponentType::Record | PortfolioComponentType::Archive
        ) {
            self.ordered_members.push(id);
        } else {
            self.unordered_members.insert(id);
        }
    }

    /// Remove a member from either list; returns `true` if found.
    pub fn remove_member(&mut self, id: &str) -> bool {
        let a = self.unordered_members.remove(id);
        let before = self.ordered_members.len();
        self.ordered_members.retain(|m| m != id);
        a || self.ordered_members.len() < before
    }

    /// Iterator over all member IDs regardless of ordering semantics.
    pub fn all_members(&self) -> impl Iterator<Item = &str> {
        self.unordered_members
            .iter()
            .map(String::as_str)
            .chain(self.ordered_members.iter().map(String::as_str))
    }

    // ── Action dispatch ───────────────────────────────────────────────────

    /// Apply a social/operational action, checking permissions first.
    pub fn apply_action(&mut self, actor_id: &str, action: ActionKind) -> PortfolioResult<()> {
        if !self.users.can(actor_id, &action) {
            return Err(PortfolioError::PermissionDenied {
                actor: actor_id.to_string(),
                action: format!("{action:?}"),
            });
        }
        let evt = ActionEvent::new(actor_id, action.clone(), Some(self.id().to_string()), "applied");
        self.analytics.record_action(&action);
        self.action_history.push(evt);
        self.metadata.touch(actor_id);

        // Side-effects
        match &action {
            ActionKind::Archive => {
                self.status = ComponentStatus::Archived;
                self.state  = ComponentState::Sealed;
            }
            ActionKind::Restore => {
                self.status = ComponentStatus::Active;
                self.state  = ComponentState::Configured;
            }
            ActionKind::Post => { self.visibility = Visibility::Public; }
            ActionKind::Hashtag => { self.analytics.spread += 1; }
            ActionKind::Donate  => { self.users.donors.insert(actor_id.to_string()); }
            ActionKind::Invest  => { self.users.investors.insert(actor_id.to_string()); }
            ActionKind::Join    => { self.users.add(actor_id.to_string(), PermissionTier::Contributor); }
            ActionKind::Leave   => { self.users.remove(actor_id); }
            _ => {}
        }
        Ok(())
    }

    // ── Version helpers ───────────────────────────────────────────────────

    pub fn bump_version(&mut self, actor_id: &str, part: VersionPart, message: &str) {
        let new = bump_semver(&self.metadata.semver, part);
        self.metadata.semver = new.clone();
        self.metadata.touch(actor_id);

        if let Some(book) = &mut self.item_book {
            let hist = book.version.get_or_insert(VersionHistory {
                current_version: new.clone(),
                entries: Vec::new(),
            });
            hist.current_version = new.clone();
            hist.entries.push(VersionEntry {
                version: new,
                author: actor_id.to_string(),
                message: message.to_string(),
                diff_ref: None,
                tags: Vec::new(),
                created_at: now(),
            });
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VersionPart { Major, Minor, Patch }

fn bump_semver(version: &str, part: VersionPart) -> String {
    let parts: Vec<u64> = version.split('.').filter_map(|s| s.parse().ok()).collect();
    let (mut ma, mut mi, mut pa) = match parts.as_slice() {
        [a, b, c, ..] => (*a, *b, *c),
        [a, b]        => (*a, *b, 0),
        [a]           => (*a, 0, 0),
        []            => (0, 1, 0),
    };
    match part {
        VersionPart::Major => { ma += 1; mi = 0; pa = 0; }
        VersionPart::Minor => { mi += 1; pa = 0; }
        VersionPart::Patch => { pa += 1; }
    }
    format!("{ma}.{mi}.{pa}")
}

// =============================================================================
// §12 — STRUCTURAL PRIMITIVES
// =============================================================================

/// Linked / peer group — bidirectional sibling set.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub description: String,
    pub group_type: String,
    pub members: Vec<ComponentId>,
    pub owner: ActorId,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Group {
    pub fn new(name: impl Into<String>, owner: impl Into<ActorId>) -> Self {
        Self {
            id: format!("grp-{}", now()),
            name: name.into(),
            description: String::new(),
            group_type: "default".to_string(),
            members: Vec::new(),
            owner: owner.into(),
            created_at: now(),
            updated_at: now(),
        }
    }
    pub fn add(&mut self, id: ComponentId) {
        if !self.members.contains(&id) { self.members.push(id); }
        self.updated_at = now();
    }
    pub fn remove(&mut self, id: &str) {
        self.members.retain(|m| m != id);
        self.updated_at = now();
    }
}

/// Unordered set of component references.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub description: String,
    pub items: HashSet<ComponentId>,
    pub tags: HashSet<String>,
    pub owner: ActorId,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Collection {
    pub fn new(name: impl Into<String>, owner: impl Into<ActorId>) -> Self {
        Self {
            id: format!("col-{}", now()),
            name: name.into(),
            description: String::new(),
            items: HashSet::new(),
            tags: HashSet::new(),
            owner: owner.into(),
            created_at: now(),
            updated_at: now(),
        }
    }
    pub fn insert(&mut self, id: ComponentId)  { self.items.insert(id); }
    pub fn remove(&mut self, id: &str) -> bool { self.items.remove(id) }
    pub fn contains(&self, id: &str) -> bool   { self.items.contains(id) }
    pub fn len(&self) -> usize { self.items.len() }
    pub fn is_empty(&self) -> bool { self.items.is_empty() }
}

/// Ordered, indexed list.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct List {
    pub id: String,
    pub name: String,
    pub description: String,
    pub items: Vec<ComponentId>,
    pub owner: ActorId,
    pub created_at: u64,
    pub updated_at: u64,
}

impl List {
    pub fn new(name: impl Into<String>, owner: impl Into<ActorId>) -> Self {
        Self {
            id: format!("lst-{}", now()),
            name: name.into(),
            description: String::new(),
            items: Vec::new(),
            owner: owner.into(),
            created_at: now(),
            updated_at: now(),
        }
    }
    pub fn push(&mut self, id: ComponentId) { self.items.push(id); }
    pub fn insert_at(&mut self, index: usize, id: ComponentId) {
        let i = index.min(self.items.len());
        self.items.insert(i, id);
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

/// Causal / time-ordered sequence.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Schedule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub entries: Vec<ScheduleEntry>,
    pub owner: ActorId,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduleEntry {
    pub id: String,
    pub component_id: ComponentId,
    pub scheduled_epoch: u64,
    pub duration_minutes: Option<u32>,
    pub recurrence: Option<String>,
    pub notes: String,
    pub order: u32,
}

impl Schedule {
    pub fn new(name: impl Into<String>, owner: impl Into<ActorId>) -> Self {
        Self {
            id: format!("sch-{}", now()),
            name: name.into(),
            description: String::new(),
            entries: Vec::new(),
            owner: owner.into(),
            created_at: now(),
            updated_at: now(),
        }
    }
    pub fn add_entry(&mut self, component_id: ComponentId, scheduled_epoch: u64) {
        let order = self.entries.len() as u32;
        self.entries.push(ScheduleEntry {
            id: format!("se-{}", now()),
            component_id,
            scheduled_epoch,
            duration_minutes: None,
            recurrence: None,
            notes: String::new(),
            order,
        });
        self.entries.sort_by_key(|e| e.scheduled_epoch);
        self.updated_at = now();
    }
    pub fn upcoming(&self, from_epoch: u64) -> Vec<&ScheduleEntry> {
        self.entries.iter().filter(|e| e.scheduled_epoch >= from_epoch).collect()
    }
}

/// Spatial / hierarchical directory (path-addressable).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Directory {
    pub id: String,
    pub name: String,
    pub description: String,
    pub path: String,
    pub entries: Vec<DirectoryEntry>,
    pub sub_directories: Vec<Directory>,
    pub owner: ActorId,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Directory {
    pub fn new(name: impl Into<String>, path: impl Into<String>, owner: impl Into<ActorId>) -> Self {
        Self {
            id: format!("dir-{}", now()),
            name: name.into(),
            description: String::new(),
            path: path.into(),
            entries: Vec::new(),
            sub_directories: Vec::new(),
            owner: owner.into(),
            created_at: now(),
            updated_at: now(),
        }
    }
    pub fn add_entry(&mut self, component_id: ComponentId, name: impl Into<String>, entry_type: &str) {
        let n = name.into();
        let path = format!("{}/{}", self.path.trim_end_matches('/'), n);
        let order = self.entries.len() as u32;
        self.entries.push(DirectoryEntry {
            id: format!("de-{}", now()),
            component_id,
            name: n,
            entry_type: entry_type.to_string(),
            path,
            order,
            created_at: now(),
        });
        self.updated_at = now();
    }
    pub fn find(&self, component_id: &str) -> Option<&DirectoryEntry> {
        self.entries.iter().find(|e| e.component_id == component_id)
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
    pub fn add_subdirectory(&mut self, sub: Directory) { self.sub_directories.push(sub); }
}

// =============================================================================
// §13 — GRAPH LAYER
// =============================================================================

/// Typed relationship between two components (directed edge).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GraphEdgeKind {
    /// Strict parent→child ownership hierarchy.
    Hierarchy,
    /// `from` depends on `to` before it can proceed.
    Dependency,
    /// Peer association (non-hierarchical, non-dependency).
    Link,
    /// Container membership (`from` contains `to`).
    Contains,
    /// Cross-portfolio federation bridge.
    Federation,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub from: ComponentId,
    pub to: ComponentId,
    pub kind: GraphEdgeKind,
    pub label: Option<String>,
    pub created_at: u64,
    pub properties: Properties,
}

impl GraphEdge {
    pub fn new(
        id: impl Into<String>,
        from: impl Into<String>,
        to: impl Into<String>,
        kind: GraphEdgeKind,
    ) -> Self {
        Self {
            id: id.into(),
            from: from.into(),
            to: to.into(),
            kind,
            label: None,
            created_at: now(),
            properties: HashMap::new(),
        }
    }
}

/// Immutable read-only view over the component graph.
pub struct GraphView<'a> {
    components: &'a HashMap<ComponentId, PortfolioComponent>,
    edges: &'a Vec<GraphEdge>,
}

impl<'a> GraphView<'a> {
    pub fn new(
        components: &'a HashMap<ComponentId, PortfolioComponent>,
        edges: &'a Vec<GraphEdge>,
    ) -> Self {
        Self { components, edges }
    }

    pub fn outgoing(&self, id: &str) -> Vec<&GraphEdge> {
        self.edges.iter().filter(|e| e.from == id).collect()
    }

    pub fn incoming(&self, id: &str) -> Vec<&GraphEdge> {
        self.edges.iter().filter(|e| e.to == id).collect()
    }

    /// All nodes reachable from `start` by following `kind` edges (BFS).
    pub fn reachable(&self, start: &str, kind: &GraphEdgeKind) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut queue   = VecDeque::new();
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

    /// Kahn's topological sort over `kind` edges.  Returns `Err` on cycle.
    pub fn topological_sort(&self, kind: &GraphEdgeKind) -> Result<Vec<String>, String> {
        let relevant: Vec<&GraphEdge> = self.edges.iter().filter(|e| &e.kind == kind).collect();
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        for c in self.components.keys() {
            in_degree.entry(c.as_str()).or_insert(0);
        }
        for e in &relevant { *in_degree.entry(e.to.as_str()).or_insert(0) += 1; }

        let mut queue: VecDeque<&str> = in_degree.iter()
            .filter(|(_, &d)| d == 0).map(|(&k, _)| k).collect();

        let mut sorted = Vec::new();
        while let Some(node) = queue.pop_front() {
            sorted.push(node.to_string());
            for e in relevant.iter().filter(|e| e.from == node) {
                let deg = in_degree.entry(e.to.as_str()).or_default();
                *deg = deg.saturating_sub(1);
                if *deg == 0 { queue.push_back(e.to.as_str()); }
            }
        }
        if sorted.len() == self.components.len() {
            Ok(sorted)
        } else {
            Err("Cycle detected in dependency graph".to_string())
        }
    }

    /// Returns `true` if adding `from→to` of `kind` would introduce a cycle.
    pub fn would_cycle(&self, from: &str, to: &str, kind: &GraphEdgeKind) -> bool {
        self.reachable(to, kind).contains(&from.to_string())
    }
}

// =============================================================================
// §14 — EVENT SOURCING
// =============================================================================

/// Every mutation the system can undergo.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PortfolioEventKind {
    // Components
    ComponentCreated, ComponentUpdated, ComponentRemoved, ComponentStatusChanged,
    // Graph
    EdgeAdded, EdgeRemoved,
    DependencyAdded, DependencyRemoved,
    HierarchyLinked, HierarchyUnlinked,
    MemberAdded, MemberRemoved,
    // Social / engagement
    ActionApplied,
    // Snapshots
    SnapshotSaved, CheckpointCreated, StateRestored,
    // Governance
    PolicyAttached, PolicyDetached,
    ApprovalRequested, ApprovalGranted, ApprovalRejected,
    ResourceAllocated, ResourceConsumed,
    /// Backward-compat alias for ResourceAllocated.
    BudgetAllocated,
    /// Backward-compat alias for ResourceConsumed.
    BudgetSpent,
    // Federation / CRDT
    PeerRegistered, PeerRemoved, CrdtMergeApplied, EventStreamPublished,
}

/// An immutable record of one state transition.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortfolioEvent {
    pub id: String,
    pub sequence: u64,
    pub timestamp: u64,
    pub actor_id: ActorId,
    pub kind: PortfolioEventKind,
    pub payload: HashMap<String, String>,
}

/// Append-only ordered event log.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
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
            id: format!("evt-{seq:06}"),
            sequence: seq,
            timestamp: now(),
            actor_id: actor_id.into(),
            kind,
            payload,
        });
        self.events.last().unwrap()
    }

    pub fn all(&self) -> &[PortfolioEvent]      { &self.events }
    pub fn len(&self) -> usize                   { self.events.len() }
    pub fn is_empty(&self) -> bool               { self.events.is_empty() }

    pub fn up_to(&self, until: u64) -> impl Iterator<Item = &PortfolioEvent> {
        self.events.iter().filter(move |e| e.timestamp <= until)
    }

    pub fn by_kind<'a>(&'a self, kind: &'a PortfolioEventKind) -> impl Iterator<Item = &'a PortfolioEvent> {
        self.events.iter().filter(move |e| {
            std::mem::discriminant(&e.kind) == std::mem::discriminant(kind)
        })
    }
}

// =============================================================================
// §15 — SNAPSHOT & CHECKPOINT
// =============================================================================

/// Complete, serialisable picture of the system at a point in time.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortfolioSnapshot {
    pub snapshot_id: String,
    pub label: Option<String>,
    pub created_at: u64,
    pub components: HashMap<ComponentId, PortfolioComponent>,
    pub edges: Vec<GraphEdge>,
    pub active_portfolio_id: Option<ComponentId>,
    pub next_id: u64,
    pub next_edge_id: u64,
    pub component_count: usize,
    pub edge_count: usize,
    pub event_count: usize,
    pub last_checkpoint_id: Option<String>,
}

/// Named, human-memorable pointer to a stored snapshot.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortfolioCheckpoint {
    pub id: String,
    pub label: String,
    pub snapshot_id: String,
    pub created_at: u64,
    pub note: Option<String>,
}

// =============================================================================
// §16 — CRDT
// =============================================================================

/// Lamport-style vector clock: actor → logical timestamp.
///
/// Provides `tick`/`merge` (runtime use) and `happened_before`/`concurrent_with`
/// (causality analysis).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct VectorClock(pub HashMap<ActorId, u64>);

impl VectorClock {
    pub fn new() -> Self { Self::default() }

    /// Increment clock for `actor`; returns the new value.
    pub fn tick(&mut self, actor: &str) -> u64 {
        let t = self.0.entry(actor.to_string()).or_insert(0);
        *t += 1;
        *t
    }

    /// Alias for `tick` (consistent with earlier portfolio.rs API).
    pub fn increment(&mut self, actor: &str) { self.tick(actor); }

    /// Merge two clocks, taking element-wise maximum.
    pub fn merge(&mut self, other: &VectorClock) {
        for (actor, &ts) in &other.0 {
            let e = self.0.entry(actor.clone()).or_insert(0);
            if ts > *e { *e = ts; }
        }
    }

    /// Returns `true` if `self` happened-before `other`.
    pub fn happened_before(&self, other: &VectorClock) -> bool {
        self.0.iter().all(|(a, &ts)| other.0.get(a).copied().unwrap_or(0) >= ts)
            && self.0 != other.0
    }

    /// Returns `true` if the two clocks are concurrent (neither dominates).
    pub fn concurrent_with(&self, other: &VectorClock) -> bool {
        !self.happened_before(other) && !other.happened_before(self)
    }
}

/// Last-Write-Wins scalar field.  Ties broken by lexicographic actor ID.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LwwField {
    pub value: String,
    pub timestamp: u64,
    pub actor_id: ActorId,
}

impl LwwField {
    pub fn new(value: impl Into<String>, actor_id: impl Into<ActorId>) -> Self {
        Self { value: value.into(), timestamp: now(), actor_id: actor_id.into() }
    }

    pub fn merge(&mut self, incoming: LwwField) {
        if incoming.timestamp > self.timestamp
            || (incoming.timestamp == self.timestamp && incoming.actor_id > self.actor_id)
        {
            *self = incoming;
        }
    }
}

/// Observed-Remove Set (OR-Set).
/// Tracks `(element, unique_tag)` pairs; removals tombstone specific tags.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
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

    pub fn members(&self) -> Vec<&str> {
        self.observed.keys().map(String::as_str).collect()
    }

    pub fn merge(&mut self, remote: &OrSet) {
        for tombstone in &remote.removed_tags {
            for tags in self.observed.values_mut() { tags.remove(tombstone); }
        }
        self.removed_tags.extend(remote.removed_tags.iter().cloned());
        for (elem, tags) in &remote.observed {
            let live: HashSet<String> = tags.iter()
                .filter(|t| !self.removed_tags.contains(*t))
                .cloned().collect();
            if !live.is_empty() {
                self.observed.entry(elem.clone()).or_default().extend(live);
            }
        }
    }
}

/// A single portable CRDT mutation (for peer exchange).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CrdtOperation {
    SetField {
        component_id: ComponentId,
        field: String,
        value: String,
        timestamp: u64,
        actor: ActorId,
    },
    AddToSet {
        component_id: ComponentId,
        set_name: String,
        element: String,
        tag: String,
        timestamp: u64,
        actor: ActorId,
    },
    RemoveFromSet {
        component_id: ComponentId,
        set_name: String,
        element: String,
        timestamp: u64,
        actor: ActorId,
    },
    AddEdge    { edge: GraphEdge, timestamp: u64, actor: ActorId },
    RemoveEdge { edge_id: String,  timestamp: u64, actor: ActorId },
}

/// Append-only log of CRDT operations for distributed merge.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
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
            .filter_map(|o| serde_json::to_string(o).ok()).collect();
        for op in &remote.operations {
            if let Ok(s) = serde_json::to_string(op) {
                if !existing.contains(&s) { self.operations.push(op.clone()); }
            }
        }
        self.clock.merge(&remote.clock);
    }
}

// =============================================================================
// §17 — GOVERNANCE
// =============================================================================

/// Resource kind discriminant.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceKind {
    Budget,
    PersonHours,
    StoryPoints,
    ComputeUnits,
    StorageGiB,
    Custom(String),
}

impl ResourceKind {
    pub fn unit_label(&self) -> &str {
        match self {
            Self::Budget       => "currency",
            Self::PersonHours  => "person-hours",
            Self::StoryPoints  => "story-points",
            Self::ComputeUnits => "compute-units",
            Self::StorageGiB   => "GiB",
            Self::Custom(u)    => u.as_str(),
        }
    }
}

/// Context passed to a `PolicyEngine` for evaluation.
#[derive(Clone, Debug)]
pub struct PolicyContext<'a> {
    pub component: &'a PortfolioComponent,
    pub event_kind: &'a PortfolioEventKind,
    pub actor_id: &'a str,
    pub properties: Properties,
}

/// Decision returned by policy evaluation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow,
    Deny(String),
    RequireApproval(String),
}

/// Governance policy engine trait.
pub trait PolicyEngine: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn evaluate(&self, ctx: &PolicyContext<'_>) -> PolicyDecision;
}

/// Approval request lifecycle stages.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalStatus { Pending, Approved, Rejected, Expired }

/// A single approval request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: String,
    pub component_id: ComponentId,
    pub requested_by: ActorId,
    pub reason: String,
    pub status: ApprovalStatus,
    pub created_at: u64,
    pub resolved_at: Option<u64>,
    pub resolver: Option<ActorId>,
    pub notes: Option<String>,
}

/// Generalised resource allocation and spend tracking per component.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub component_id: ComponentId,
    pub kind: ResourceKind,
    pub total: f64,
    pub allocated: f64,
    pub consumed: f64,
    pub denomination: String,
    pub period: Option<String>,
}

impl ResourceAllocation {
    pub fn remaining(&self)       -> f64 { self.total - self.consumed }
    pub fn utilisation_pct(&self) -> f64 {
        if self.total == 0.0 { 0.0 } else { self.consumed / self.total * 100.0 }
    }
    pub fn is_overrun(&self)  -> bool { self.consumed > self.total }
    pub fn headroom(&self)    -> f64  { self.allocated - self.consumed }
}

/// Backward-compat alias.
pub type BudgetAllocation = ResourceAllocation;

// =============================================================================
// §18 — PLUGIN TRAITS
// =============================================================================

/// Lifecycle hooks for extending `PortfolioSystem` behaviour.
pub trait PortfolioPlugin: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn on_component_created(&self, _c: &PortfolioComponent) {}
    fn on_component_updated(&self, _c: &PortfolioComponent) {}
    fn on_component_removed(&self, _id: &str) {}
    fn on_edge_added(&self, _e: &GraphEdge) {}
    fn on_edge_removed(&self, _id: &str) {}
    fn on_event(&self, _e: &PortfolioEvent) {}
    fn on_snapshot_saved(&self, _s: &PortfolioSnapshot) {}
    fn on_crdt_merge(&self, _ops: &[CrdtOperation]) {}
}

// =============================================================================
// §19 — FEDERATION
// =============================================================================

/// A registered remote peer in the portfolio federation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FederationPeer {
    pub id: String,
    pub name: String,
    pub endpoint: Option<String>,
    pub last_seen: u64,
    pub vector_clock: VectorClock,
    pub trusted: bool,
}

/// Multi-portfolio federation: named `PortfolioSystem` instances that can
/// exchange CRDT operations and share cross-portfolio edges.
pub struct PortfolioFederation {
    pub portfolios: HashMap<String, PortfolioSystem>,
    pub peers: HashMap<String, FederationPeer>,
    pub federation_edges: Vec<GraphEdge>,
    next_edge_id: u64,
}

impl PortfolioFederation {
    pub fn new() -> Self {
        Self {
            portfolios: HashMap::new(),
            peers: HashMap::new(),
            federation_edges: Vec::new(),
            next_edge_id: 1,
        }
    }

    pub fn add_portfolio(&mut self, id: impl Into<String>, sys: PortfolioSystem) {
        self.portfolios.insert(id.into(), sys);
    }
    pub fn remove_portfolio(&mut self, id: &str) -> Option<PortfolioSystem> {
        self.portfolios.remove(id)
    }
    pub fn get_portfolio(&self, id: &str) -> Option<&PortfolioSystem> {
        self.portfolios.get(id)
    }
    pub fn get_portfolio_mut(&mut self, id: &str) -> Option<&mut PortfolioSystem> {
        self.portfolios.get_mut(id)
    }

    pub fn register_peer(&mut self, peer: FederationPeer) {
        self.peers.insert(peer.id.clone(), peer);
    }
    pub fn remove_peer(&mut self, id: &str) -> Option<FederationPeer> {
        self.peers.remove(id)
    }

    /// Add a cross-portfolio `Federation` edge.
    pub fn federate(
        &mut self,
        from_portfolio: &str, from_component: &str,
        to_portfolio: &str,   to_component: &str,
    ) {
        let id = format!("fed-edge-{:04}", self.next_edge_id);
        self.next_edge_id += 1;
        self.federation_edges.push(GraphEdge {
            id,
            from: format!("{from_portfolio}/{from_component}"),
            to:   format!("{to_portfolio}/{to_component}"),
            kind: GraphEdgeKind::Federation,
            label: None,
            created_at: now(),
            properties: HashMap::new(),
        });
    }

    /// Push CRDT log from `source_id` into `target_id`.
    pub fn sync_crdt(&mut self, source_id: &str, target_id: &str) -> Result<(), String> {
        let source_crdt = self.portfolios.get(source_id)
            .ok_or_else(|| format!("Portfolio '{source_id}' not found"))?
            .crdt.clone();

        let target = self.portfolios.get_mut(target_id)
            .ok_or_else(|| format!("Portfolio '{target_id}' not found"))?;

        target.crdt.merge(&source_crdt);
        target.event_log.append(
            "federation",
            PortfolioEventKind::CrdtMergeApplied,
            HashMap::from([
                ("source".to_string(), source_id.to_string()),
                ("ops".to_string(), source_crdt.operations.len().to_string()),
            ]),
        );
        Ok(())
    }

    /// Broadcast one portfolio's CRDT log to all other portfolios.
    pub fn broadcast_crdt(&mut self, source_id: &str) -> Result<usize, String> {
        let targets: Vec<String> = self.portfolios.keys()
            .filter(|k| k.as_str() != source_id).cloned().collect();
        let n = targets.len();
        for t in targets { self.sync_crdt(source_id, &t)?; }
        Ok(n)
    }
}

impl Default for PortfolioFederation {
    fn default() -> Self { Self::new() }
}

// =============================================================================
// §20 — QUERY LANGUAGE  (PQL + SearchQuery)
// =============================================================================

/// Structured predicate filter — used by `PortfolioSystem::query`.
#[derive(Clone, Debug, Default)]
pub struct PortfolioQuery {
    pub component_types: Option<Vec<PortfolioComponentType>>,
    pub statuses: Option<Vec<ComponentStatus>>,
    pub tags: Option<Vec<String>>,
    pub name_contains: Option<String>,
    pub owner: Option<ActorId>,
    pub has_dependencies: Option<bool>,
    pub has_children: Option<bool>,
    pub property_filter: Option<(String, String)>,
    pub visibility: Option<Visibility>,
}

impl PortfolioQuery {
    /// Parse a minimal PQL (Portfolio Query Language) string.
    ///
    /// Tokens are space-separated, evaluated as AND:
    ///   `type=<Type>`   `status=<value>`   `tag=<value>`
    ///   `name=<substr>` `owner=<id>`
    pub fn parse(pql: &str) -> Self {
        let mut q = Self::default();
        for token in pql.split_whitespace() {
            if let Some(rest) = token.strip_prefix("type=") {
                q.component_types.get_or_insert_with(Vec::new)
                    .push(parse_component_type(rest));
            } else if let Some(rest) = token.strip_prefix("status=") {
                q.statuses.get_or_insert_with(Vec::new)
                    .push(parse_status(rest));
            } else if let Some(rest) = token.strip_prefix("tag=") {
                q.tags.get_or_insert_with(Vec::new).push(rest.to_string());
            } else if let Some(rest) = token.strip_prefix("name=") {
                q.name_contains = Some(rest.to_string());
            } else if let Some(rest) = token.strip_prefix("owner=") {
                q.owner = Some(rest.to_string());
            }
        }
        q
    }
}

fn parse_component_type(s: &str) -> PortfolioComponentType {
    match s.to_lowercase().as_str() {
        "portfolio"               => PortfolioComponentType::Portfolio,
        "project"                 => PortfolioComponentType::Project,
        "program"                 => PortfolioComponentType::Program,
        "resource"                => PortfolioComponentType::Resource,
        "asset"                   => PortfolioComponentType::Asset,
        "artifact"                => PortfolioComponentType::Artifact,
        "subportfolio"|"sub_portfolio" => PortfolioComponentType::SubPortfolio,
        "binder"                  => PortfolioComponentType::Binder,
        "book"                    => PortfolioComponentType::Book,
        "folder"                  => PortfolioComponentType::Folder,
        "record"                  => PortfolioComponentType::Record,
        "registry"                => PortfolioComponentType::Registry,
        "archive"                 => PortfolioComponentType::Archive,
        _                         => PortfolioComponentType::Project,
    }
}

fn parse_status(s: &str) -> ComponentStatus {
    match s.to_lowercase().as_str() {
        "draft"       => ComponentStatus::Draft,
        "active"      => ComponentStatus::Active,
        "paused"      => ComponentStatus::Paused,
        "completed"   => ComponentStatus::Completed,
        "archived"    => ComponentStatus::Archived,
        "deleted"     => ComponentStatus::Deleted,
        "deprecated"  => ComponentStatus::Deprecated,
        "underreview" => ComponentStatus::UnderReview,
        "rejected"    => ComponentStatus::Rejected,
        other         => ComponentStatus::Custom(other.to_string()),
    }
}

/// Rich search query (includes scoring, pagination).
#[derive(Clone, Debug, Default)]
pub struct SearchQuery {
    pub text: Option<String>,
    pub tags: Vec<String>,
    pub hashtags: Vec<String>,
    pub component_types: Vec<PortfolioComponentType>,
    pub statuses: Vec<ComponentStatus>,
    pub owner: Option<ActorId>,
    pub visibility: Option<Visibility>,
    pub created_after: Option<u64>,
    pub created_before: Option<u64>,
    pub limit: Option<usize>,
    pub offset: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub component_id: ComponentId,
    pub name: String,
    pub component_type: PortfolioComponentType,
    pub status: ComponentStatus,
    pub score: f64,
    pub matched_fields: Vec<String>,
}

// =============================================================================
// §21 — PORTFOLIO SYSTEM  (main runtime)
// =============================================================================

/// The central runtime.
///
/// All state is stored in `components` (flat `HashMap<id, PortfolioComponent>`)
/// plus `edges` (the typed relationship graph).  Snapshots, checkpoints,
/// events, CRDT operations, governance data, plugins, and structural primitives
/// are maintained alongside.
pub struct PortfolioSystem {
    // ── Core state ────────────────────────────────────────────────────────
    pub components: HashMap<ComponentId, PortfolioComponent>,
    pub edges: Vec<GraphEdge>,
    pub active_portfolio_id: Option<ComponentId>,

    // ── Identity / sequencing ─────────────────────────────────────────────
    pub actor_id: ActorId,
    next_id: u64,
    next_edge_id: u64,
    next_snap_id: u64,
    next_ckpt_id: u64,
    next_req_id: u64,

    // ── Structural primitives ─────────────────────────────────────────────
    pub groups:      HashMap<String, Group>,
    pub collections: HashMap<String, Collection>,
    pub lists:       HashMap<String, List>,
    pub schedules:   HashMap<String, Schedule>,
    pub directories: HashMap<String, Directory>,

    // ── History / time-travel ─────────────────────────────────────────────
    pub event_log:   EventLog,
    pub snapshots:   HashMap<String, PortfolioSnapshot>,
    pub checkpoints: Vec<PortfolioCheckpoint>,

    // ── Distributed ───────────────────────────────────────────────────────
    pub crdt: CrdtLog,

    // ── Governance ────────────────────────────────────────────────────────
    pub policy_engines: Vec<Arc<dyn PolicyEngine>>,
    pub approval_requests: Vec<ApprovalRequest>,
    pub resource_allocations: HashMap<ComponentId, ResourceAllocation>,

    // ── Plugins ───────────────────────────────────────────────────────────
    pub plugins: Vec<Arc<dyn PortfolioPlugin>>,
}

impl std::fmt::Debug for PortfolioSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PortfolioSystem")
            .field("components", &self.components.len())
            .field("edges", &self.edges.len())
            .field("active_portfolio_id", &self.active_portfolio_id)
            .field("actor_id", &self.actor_id)
            .field("snapshots", &self.snapshots.len())
            .field("checkpoints", &self.checkpoints.len())
            .field("policy_engines", &self.policy_engines.len())
            .field("approval_requests", &self.approval_requests.len())
            .field("resource_allocations", &self.resource_allocations.len())
            .field("plugins", &self.plugins.len())
            .finish()
    }
}

// ── Constructors ──────────────────────────────────────────────────────────────

impl PortfolioSystem {
    /// Create an empty system bound to `actor_id`.
    pub fn new(actor_id: impl Into<ActorId>) -> Self {
        Self {
            components: HashMap::new(),
            edges: Vec::new(),
            active_portfolio_id: None,
            actor_id: actor_id.into(),
            next_id: 1,
            next_edge_id: 1,
            next_snap_id: 1,
            next_ckpt_id: 1,
            next_req_id: 1,
            groups: HashMap::new(),
            collections: HashMap::new(),
            lists: HashMap::new(),
            schedules: HashMap::new(),
            directories: HashMap::new(),
            event_log: EventLog::new(),
            snapshots: HashMap::new(),
            checkpoints: Vec::new(),
            crdt: CrdtLog::new(),
            policy_engines: Vec::new(),
            approval_requests: Vec::new(),
            resource_allocations: HashMap::new(),
            plugins: Vec::new(),
        }
    }

    /// Backward-compatible MVP seed (mirrors the original hardcoded state).
    pub fn mvp() -> Self {
        let mut sys = Self::new("system");
        let comp = sys.create_component_raw(
            PortfolioComponentType::Project,
            "Kogi Kernel Runtime",
            vec![
                "Binder", "Book", "Notebook", "Playbook",
                "Folder", "FileSet", "VersionControl", "Metadata",
            ],
        );
        sys.active_portfolio_id = Some(comp.metadata.id.clone());
        sys
    }

    // ── Private helpers ───────────────────────────────────────────────────

    fn next_comp_id(&mut self) -> ComponentId {
        let id = format!("comp-{:06}", self.next_id);
        self.next_id += 1;
        id
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
        let map: HashMap<String, String> = payload.into_iter()
            .map(|(k, v)| (k.to_string(), v)).collect();
        let actor = self.actor_id.clone();
        let evt = self.event_log.append(&actor, kind, map).clone();
        for plugin in &self.plugins { plugin.on_event(&evt); }
    }

    fn evaluate_policies(
        &self,
        component: &PortfolioComponent,
        event_kind: &PortfolioEventKind,
    ) -> PolicyDecision {
        for engine in &self.policy_engines {
            let ctx = PolicyContext {
                component,
                event_kind,
                actor_id: &self.actor_id,
                properties: HashMap::new(),
            };
            match engine.evaluate(&ctx) {
                PolicyDecision::Allow => continue,
                d => return d,
            }
        }
        PolicyDecision::Allow
    }

    fn create_component_raw(
        &mut self,
        component_type: PortfolioComponentType,
        name: &str,
        container_refs: Vec<&str>,
    ) -> PortfolioComponent {
        let id    = self.next_comp_id();
        let actor = self.actor_id.clone();
        let mut comp = PortfolioComponent::new(&id, &actor, component_type, name);
        comp.container_refs = container_refs.into_iter().map(|s| LegacyContainerRef(s.to_string())).collect();
        self.emit(
            PortfolioEventKind::ComponentCreated,
            [("id", id.clone()), ("name", name.to_string())],
        );
        for plugin in &self.plugins { plugin.on_component_created(&comp); }
        self.components.insert(id, comp.clone());
        comp
    }
}

// ── Component CRUD ────────────────────────────────────────────────────────────

impl PortfolioSystem {
    /// Create and register a new component.
    /// Returns `Err` if a governance policy denies the operation.
    pub fn create_component(
        &mut self,
        component_type: PortfolioComponentType,
        name: impl Into<String>,
    ) -> Result<PortfolioComponent, String> {
        let name = name.into();
        let id   = self.next_comp_id();
        let actor = self.actor_id.clone();
        let comp  = PortfolioComponent::new(&id, &actor, component_type.clone(), &name);

        match self.evaluate_policies(&comp, &PortfolioEventKind::ComponentCreated) {
            PolicyDecision::Allow => {}
            PolicyDecision::Deny(r)            => return Err(format!("Denied: {r}")),
            PolicyDecision::RequireApproval(r) => return Err(format!("Requires approval: {r}")),
        }

        self.emit(
            PortfolioEventKind::ComponentCreated,
            [
                ("id",   id.clone()),
                ("name", name.clone()),
                ("type", format!("{component_type:?}")),
            ],
        );
        self.crdt.append(&actor, CrdtOperation::SetField {
            component_id: id.clone(),
            field: "name".into(),
            value: name,
            timestamp: now(),
            actor: actor.clone(),
        });
        for plugin in &self.plugins { plugin.on_component_created(&comp); }
        self.components.insert(id.clone(), comp.clone());
        Ok(comp)
    }

    /// Create a `Book` component with a specific `BookType`.
    pub fn create_book(
        &mut self,
        book_type: BookType,
        name: impl Into<String>,
    ) -> Result<PortfolioComponent, String> {
        let mut comp = self.create_component(PortfolioComponentType::Book, name)?;
        comp.book_type = Some(book_type.clone());
        if book_type == BookType::Itembook {
            if let Some(pid) = self.active_portfolio_id.clone() {
                self.add_member(&pid.clone(), &comp.metadata.id.clone()).ok();
            }
        }
        self.components.entry(comp.metadata.id.clone())
            .and_modify(|c| c.book_type = Some(book_type));
        Ok(comp)
    }

    pub fn get(&self, id: &str) -> Option<&PortfolioComponent> {
        self.components.get(id)
    }
    pub fn get_mut(&mut self, id: &str) -> Option<&mut PortfolioComponent> {
        self.components.get_mut(id)
    }
    /// Alias for `get`.
    pub fn get_component(&self, id: &str) -> Option<&PortfolioComponent> { self.get(id) }
    /// Alias for `get_mut`.
    pub fn get_component_mut(&mut self, id: &str) -> Option<&mut PortfolioComponent> { self.get_mut(id) }

    pub fn all_components(&self) -> Vec<&PortfolioComponent> {
        self.components.values().collect()
    }

    pub fn components_by_type(&self, ct: &PortfolioComponentType) -> Vec<&PortfolioComponent> {
        self.components.values().filter(|c| &c.component_type == ct).collect()
    }

    pub fn components_subset<'a>(&'a self, ids: &[ComponentId]) -> Vec<&'a PortfolioComponent> {
        ids.iter().filter_map(|id| self.components.get(id)).collect()
    }

    /// Update name / status / description / tags / properties / owner.
    /// Only `Some(…)` fields are applied.
    pub fn edit_component(
        &mut self,
        id: &str,
        name: Option<String>,
        status: Option<ComponentStatus>,
        description: Option<String>,
        tags: Option<Vec<String>>,
        properties: Option<Properties>,
        owner: Option<ActorId>,
    ) -> Result<PortfolioComponent, String> {
        let actor = self.actor_id.clone();
        let comp  = self.components.get_mut(id)
            .ok_or_else(|| format!("Component '{id}' not found"))?;

        if let Some(n) = name {
            self.crdt.append(&actor, CrdtOperation::SetField {
                component_id: id.to_string(),
                field: "name".into(),
                value: n.clone(),
                timestamp: now(),
                actor: actor.clone(),
            });
            comp.name = n;
        }
        if let Some(s) = status      { comp.status = s; }
        if let Some(d) = description { comp.description = d; }
        if let Some(t) = tags        { comp.metadata.tags = t; }
        if let Some(p) = properties  { comp.metadata.properties.extend(p); }
        if let Some(o) = owner       {
            comp.users.add(o.clone(), PermissionTier::Owner);
            if !comp.metadata.owners.contains(&o) { comp.metadata.owners.push(o); }
        }
        comp.metadata.touch(&actor);
        let result = comp.clone();
        self.emit(PortfolioEventKind::ComponentUpdated, [("id", id.to_string())]);
        for plugin in &self.plugins { plugin.on_component_updated(&result); }
        Ok(result)
    }

    /// Remove a component and all its incident edges.
    pub fn remove_component(&mut self, id: &str) -> Result<PortfolioComponent, String> {
        let removed = self.components.remove(id)
            .ok_or_else(|| format!("Component '{id}' not found"))?;

        let edge_ids: Vec<String> = self.edges.iter()
            .filter(|e| e.from == id || e.to == id)
            .map(|e| e.id.clone()).collect();
        for eid in &edge_ids { self.remove_edge(eid).ok(); }

        for comp in self.components.values_mut() {
            comp.children.remove(id);
            comp.parents.remove(id);
            comp.links.remove(id);
            comp.dependencies.remove(id);
            comp.dependents.remove(id);
            comp.unordered_members.remove(id);
            comp.ordered_members.retain(|m| m != id);
        }
        self.emit(PortfolioEventKind::ComponentRemoved, [("id", id.to_string())]);
        for plugin in &self.plugins { plugin.on_component_removed(id); }
        Ok(removed)
    }

    pub fn set_active_portfolio(&mut self, id: &str) -> Result<(), String> {
        if self.components.contains_key(id) {
            self.active_portfolio_id = Some(id.to_string());
            Ok(())
        } else {
            Err(format!("Component '{id}' not found"))
        }
    }
}

// ── Graph operations ──────────────────────────────────────────────────────────

impl PortfolioSystem {
    /// Add a directed `Hierarchy` (parent→child) edge.  Returns `Err` on cycle.
    pub fn add_hierarchy(&mut self, parent_id: &str, child_id: &str) -> Result<String, String> {
        if GraphView::new(&self.components, &self.edges)
            .would_cycle(parent_id, child_id, &GraphEdgeKind::Hierarchy)
        {
            return Err(format!("Hierarchy {parent_id}→{child_id} would create a cycle"));
        }
        let edge_id = self.next_edge_id_str();
        let edge = GraphEdge::new(&edge_id, parent_id, child_id, GraphEdgeKind::Hierarchy);
        self.edges.push(edge.clone());
        if let Some(p) = self.components.get_mut(parent_id) { p.children.insert(child_id.to_string()); }
        if let Some(c) = self.components.get_mut(child_id)  { c.parents.insert(parent_id.to_string()); }
        self.emit(PortfolioEventKind::HierarchyLinked, [
            ("parent", parent_id.to_string()),
            ("child", child_id.to_string()),
            ("edge_id", edge_id.clone()),
        ]);
        self.crdt.append(&self.actor_id.clone(), CrdtOperation::AddEdge { edge, timestamp: now(), actor: self.actor_id.clone() });
        for plugin in &self.plugins { plugin.on_edge_added(self.edges.last().unwrap()); }
        Ok(edge_id)
    }

    /// Remove a `Hierarchy` edge by component IDs.
    pub fn remove_hierarchy(&mut self, parent_id: &str, child_id: &str) -> Result<(), String> {
        let eid = self.edges.iter()
            .find(|e| e.from == parent_id && e.to == child_id && e.kind == GraphEdgeKind::Hierarchy)
            .map(|e| e.id.clone())
            .ok_or_else(|| format!("Hierarchy {parent_id}→{child_id} not found"))?;
        self.remove_edge(&eid)?;
        if let Some(p) = self.components.get_mut(parent_id) { p.children.remove(child_id); }
        if let Some(c) = self.components.get_mut(child_id)  { c.parents.remove(parent_id); }
        self.emit(PortfolioEventKind::HierarchyUnlinked, [
            ("parent", parent_id.to_string()),
            ("child", child_id.to_string()),
        ]);
        Ok(())
    }

    /// Add a directed `Dependency` (from→to) edge.  Returns `Err` on cycle.
    pub fn add_dependency(&mut self, from: &str, to: &str) -> Result<String, String> {
        if GraphView::new(&self.components, &self.edges)
            .would_cycle(from, to, &GraphEdgeKind::Dependency)
        {
            return Err(format!("Dependency {from}→{to} would create a cycle"));
        }
        let edge_id = self.next_edge_id_str();
        let edge = GraphEdge::new(&edge_id, from, to, GraphEdgeKind::Dependency);
        self.edges.push(edge.clone());
        if let Some(f) = self.components.get_mut(from) { f.dependencies.insert(to.to_string()); }
        if let Some(t) = self.components.get_mut(to)   { t.dependents.insert(from.to_string()); }
        self.emit(PortfolioEventKind::DependencyAdded, [
            ("from", from.to_string()), ("to", to.to_string()), ("edge_id", edge_id.clone()),
        ]);
        self.crdt.append(&self.actor_id.clone(), CrdtOperation::AddEdge { edge, timestamp: now(), actor: self.actor_id.clone() });
        Ok(edge_id)
    }

    /// Remove a `Dependency` edge by component IDs.
    pub fn remove_dependency(&mut self, from: &str, to: &str) -> Result<(), String> {
        let eid = self.edges.iter()
            .find(|e| e.from == from && e.to == to && e.kind == GraphEdgeKind::Dependency)
            .map(|e| e.id.clone())
            .ok_or_else(|| format!("Dependency {from}→{to} not found"))?;
        self.remove_edge(&eid)?;
        if let Some(f) = self.components.get_mut(from) { f.dependencies.remove(to); }
        if let Some(t) = self.components.get_mut(to)   { t.dependents.remove(from); }
        self.emit(PortfolioEventKind::DependencyRemoved, [("from", from.to_string()), ("to", to.to_string())]);
        Ok(())
    }

    /// Add a bidirectional `Link` edge.
    pub fn add_link(&mut self, a: &str, b: &str) -> Result<String, String> {
        let edge_id = self.next_edge_id_str();
        let edge = GraphEdge::new(&edge_id, a, b, GraphEdgeKind::Link);
        self.edges.push(edge.clone());
        if let Some(ca) = self.components.get_mut(a) { ca.links.insert(b.to_string()); }
        if let Some(cb) = self.components.get_mut(b) { cb.links.insert(a.to_string()); }
        self.emit(PortfolioEventKind::EdgeAdded, [("a", a.to_string()), ("b", b.to_string()), ("edge_id", edge_id.clone())]);
        self.crdt.append(&self.actor_id.clone(), CrdtOperation::AddEdge { edge, timestamp: now(), actor: self.actor_id.clone() });
        for plugin in &self.plugins { plugin.on_edge_added(self.edges.last().unwrap()); }
        Ok(edge_id)
    }

    /// Remove a `Link` edge by component IDs.
    pub fn remove_link(&mut self, a: &str, b: &str) -> Result<(), String> {
        let eid = self.edges.iter()
            .find(|e| e.kind == GraphEdgeKind::Link && ((e.from == a && e.to == b) || (e.from == b && e.to == a)))
            .map(|e| e.id.clone())
            .ok_or_else(|| format!("Link {a}↔{b} not found"))?;
        self.remove_edge(&eid)?;
        if let Some(ca) = self.components.get_mut(a) { ca.links.remove(b); }
        if let Some(cb) = self.components.get_mut(b) { cb.links.remove(a); }
        Ok(())
    }

    /// Remove any edge by its ID.
    pub fn remove_edge(&mut self, edge_id: &str) -> Result<GraphEdge, String> {
        let pos = self.edges.iter().position(|e| e.id == edge_id)
            .ok_or_else(|| format!("Edge '{edge_id}' not found"))?;
        let removed = self.edges.remove(pos);
        self.emit(PortfolioEventKind::EdgeRemoved, [("edge_id", edge_id.to_string())]);
        self.crdt.append(&self.actor_id.clone(), CrdtOperation::RemoveEdge {
            edge_id: edge_id.to_string(), timestamp: now(), actor: self.actor_id.clone(),
        });
        for plugin in &self.plugins { plugin.on_edge_removed(edge_id); }
        Ok(removed)
    }

    /// Add `item_id` as a member of `container_id`.
    pub fn add_member(&mut self, container_id: &str, item_id: &str) -> Result<(), String> {
        let comp = self.components.get_mut(container_id)
            .ok_or_else(|| format!("Container '{container_id}' not found"))?;
        comp.add_member(item_id);
        comp.metadata.touch(&self.actor_id.clone());
        let edge_id = self.next_edge_id_str();
        let edge = GraphEdge::new(&edge_id, container_id, item_id, GraphEdgeKind::Contains);
        self.edges.push(edge);
        self.emit(PortfolioEventKind::MemberAdded, [
            ("container", container_id.to_string()), ("item", item_id.to_string()),
        ]);
        Ok(())
    }

    /// Remove `item_id` from the membership of `container_id`.
    pub fn remove_member(&mut self, container_id: &str, item_id: &str) -> Result<(), String> {
        let comp = self.components.get_mut(container_id)
            .ok_or_else(|| format!("Container '{container_id}' not found"))?;
        if !comp.remove_member(item_id) {
            return Err(format!("Item '{item_id}' not a member of '{container_id}'"));
        }
        comp.metadata.touch(&self.actor_id.clone());
        self.edges.retain(|e| !(e.from == container_id && e.to == item_id && e.kind == GraphEdgeKind::Contains));
        self.emit(PortfolioEventKind::MemberRemoved, [
            ("container", container_id.to_string()), ("item", item_id.to_string()),
        ]);
        Ok(())
    }

    /// Read-only graph view over the current state.
    pub fn graph_view(&self) -> GraphView<'_> {
        GraphView::new(&self.components, &self.edges)
    }

    pub fn dependency_order(&self) -> Result<Vec<String>, String> {
        self.graph_view().topological_sort(&GraphEdgeKind::Dependency)
    }

    pub fn transitive_dependencies(&self, id: &str) -> Vec<String> {
        self.graph_view().reachable(id, &GraphEdgeKind::Dependency)
    }

    pub fn subtree(&self, id: &str) -> Vec<String> {
        self.graph_view().reachable(id, &GraphEdgeKind::Hierarchy)
    }

    pub fn ancestors(&self, id: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut queue  = VecDeque::new();
        let mut seen   = HashSet::new();
        queue.push_back(id.to_string());
        while let Some(current) = queue.pop_front() {
            if !seen.insert(current.clone()) { continue; }
            if let Some(comp) = self.components.get(&current) {
                for parent in &comp.parents {
                    if !seen.contains(parent) {
                        result.push(parent.clone());
                        queue.push_back(parent.clone());
                    }
                }
            }
        }
        result
    }
}

// ── Social actions ────────────────────────────────────────────────────────────

impl PortfolioSystem {
    /// Apply a social/operational `ActionKind` to a component.
    pub fn act(&mut self, actor_id: &str, component_id: &str, action: ActionKind) -> PortfolioResult<()> {
        let comp = self.components.get_mut(component_id)
            .ok_or_else(|| PortfolioError::NotFound(component_id.to_string()))?;
        comp.apply_action(actor_id, action.clone())?;
        self.emit(PortfolioEventKind::ActionApplied, [
            ("actor", actor_id.to_string()),
            ("component", component_id.to_string()),
            ("action", format!("{action:?}")),
        ]);
        Ok(())
    }

    pub fn publish(&mut self, actor_id: &str, component_id: &str) -> PortfolioResult<()> {
        let comp = self.components.get_mut(component_id)
            .ok_or_else(|| PortfolioError::NotFound(component_id.to_string()))?;
        comp.apply_action(actor_id, ActionKind::Post)?;
        Ok(())
    }

    pub fn tag_component(&mut self, actor_id: &str, component_id: &str, tag: &str) -> PortfolioResult<()> {
        let comp = self.components.get_mut(component_id)
            .ok_or_else(|| PortfolioError::NotFound(component_id.to_string()))?;
        if !comp.metadata.tags.contains(&tag.to_string()) {
            comp.metadata.tags.push(tag.to_string());
        }
        comp.apply_action(actor_id, ActionKind::Tag)?;
        Ok(())
    }

    pub fn hashtag_component(&mut self, actor_id: &str, component_id: &str, topic: &str) -> PortfolioResult<()> {
        let comp = self.components.get_mut(component_id)
            .ok_or_else(|| PortfolioError::NotFound(component_id.to_string()))?;
        comp.hashtags.insert(topic.to_string());
        comp.apply_action(actor_id, ActionKind::Hashtag)?;
        Ok(())
    }
}

// ── Permissions ───────────────────────────────────────────────────────────────

impl PortfolioSystem {
    pub fn grant(&mut self, actor_id: &str, target: &str, component_id: &str, tier: PermissionTier) -> PortfolioResult<()> {
        let comp = self.components.get_mut(component_id)
            .ok_or_else(|| PortfolioError::NotFound(component_id.to_string()))?;
        if !comp.users.is_owner(actor_id) {
            return Err(PortfolioError::PermissionDenied {
                actor: actor_id.to_string(),
                action: "grant".into(),
            });
        }
        comp.users.add(target.to_string(), tier);
        comp.metadata.touch(actor_id);
        Ok(())
    }

    pub fn revoke(&mut self, actor_id: &str, target: &str, component_id: &str) -> PortfolioResult<()> {
        let comp = self.components.get_mut(component_id)
            .ok_or_else(|| PortfolioError::NotFound(component_id.to_string()))?;
        if !comp.users.is_owner(actor_id) {
            return Err(PortfolioError::PermissionDenied {
                actor: actor_id.to_string(),
                action: "revoke".into(),
            });
        }
        comp.users.remove(target);
        comp.metadata.touch(actor_id);
        Ok(())
    }
}

// ── Snapshot / checkpoint / time-travel ──────────────────────────────────────

impl PortfolioSystem {
    pub fn snapshot(&self) -> PortfolioSnapshot { self.build_snapshot(None) }

    pub fn save_snapshot(&mut self, label: Option<String>) -> PortfolioSnapshot {
        let snap = self.build_snapshot(label);
        let id   = snap.snapshot_id.clone();
        self.snapshots.insert(id.clone(), snap.clone());
        self.emit(PortfolioEventKind::SnapshotSaved, [("snapshot_id", id)]);
        for plugin in &self.plugins { plugin.on_snapshot_saved(&snap); }
        snap
    }

    pub fn save_checkpoint(&mut self, label: impl Into<String>, note: Option<String>) -> PortfolioCheckpoint {
        let label = label.into();
        let snap  = self.save_snapshot(Some(label.clone()));
        let id    = format!("ckpt-{:04}", self.next_ckpt_id);
        self.next_ckpt_id += 1;
        let ckpt  = PortfolioCheckpoint {
            id: id.clone(), label, snapshot_id: snap.snapshot_id.clone(), created_at: now(), note,
        };
        self.checkpoints.push(ckpt.clone());
        self.emit(PortfolioEventKind::CheckpointCreated, [
            ("checkpoint_id", id), ("snapshot_id", snap.snapshot_id),
        ]);
        ckpt
    }

    pub fn restore_snapshot(&mut self, snapshot_id: &str) -> Result<(), String> {
        let snap = self.snapshots.get(snapshot_id)
            .ok_or_else(|| format!("Snapshot '{snapshot_id}' not found"))?.clone();
        self.apply_snapshot(snap);
        Ok(())
    }

    pub fn restore_checkpoint(&mut self, checkpoint_id: &str) -> Result<(), String> {
        let snap_id = self.checkpoints.iter()
            .find(|c| c.id == checkpoint_id || c.label == checkpoint_id)
            .ok_or_else(|| format!("Checkpoint '{checkpoint_id}' not found"))?
            .snapshot_id.clone();
        self.restore_snapshot(&snap_id)
    }

    pub fn restore_from(&mut self, snapshot: PortfolioSnapshot) { self.apply_snapshot(snapshot); }

    /// Reconstruct a read-only state image by replaying events up to `timestamp`.
    pub fn reconstruct_at(&self, timestamp: u64) -> PortfolioSystem {
        let base_snap = self.snapshots.values()
            .filter(|s| s.created_at <= timestamp)
            .max_by_key(|s| s.created_at);

        let mut rebuilt = PortfolioSystem::new(&self.actor_id);
        if let Some(snap) = base_snap { rebuilt.apply_snapshot(snap.clone()); }

        let snap_ts = base_snap.map_or(0, |s| s.created_at);
        for evt in self.event_log.all().iter()
            .filter(|e| e.timestamp > snap_ts && e.timestamp <= timestamp)
        {
            rebuilt.apply_event_replay(evt);
        }
        rebuilt
    }

    fn build_snapshot(&self, label: Option<String>) -> PortfolioSnapshot {
        let id = format!("snap-{:06}", self.next_snap_id);
        PortfolioSnapshot {
            snapshot_id: id,
            label,
            created_at: now(),
            components: self.components.clone(),
            edges: self.edges.clone(),
            active_portfolio_id: self.active_portfolio_id.clone(),
            next_id: self.next_id,
            next_edge_id: self.next_edge_id,
            component_count: self.components.len(),
            edge_count: self.edges.len(),
            event_count: self.event_log.len(),
            last_checkpoint_id: self.checkpoints.last().map(|c| c.id.clone()),
        }
    }

    fn apply_snapshot(&mut self, snap: PortfolioSnapshot) {
        self.components = snap.components;
        self.edges = snap.edges;
        self.active_portfolio_id = snap.active_portfolio_id;
        self.next_id = snap.next_id;
        self.next_edge_id = snap.next_edge_id;
        self.emit(PortfolioEventKind::StateRestored, [("snapshot_id", snap.snapshot_id)]);
    }

    fn apply_event_replay(&mut self, event: &PortfolioEvent) {
        match &event.kind {
            PortfolioEventKind::ComponentCreated => {
                if let (Some(id), Some(name)) = (event.payload.get("id"), event.payload.get("name")) {
                    let ct = event.payload.get("type")
                        .map(|t| parse_component_type(t))
                        .unwrap_or(PortfolioComponentType::Project);
                    let comp = PortfolioComponent::new(id, &event.actor_id, ct, name);
                    self.components.insert(id.clone(), comp);
                }
            }
            PortfolioEventKind::ComponentRemoved => {
                if let Some(id) = event.payload.get("id") { self.components.remove(id); }
            }
            PortfolioEventKind::EdgeRemoved => {
                if let Some(eid) = event.payload.get("edge_id") { self.edges.retain(|e| &e.id != eid); }
            }
            _ => {}
        }
    }
}

// ── Queries & search ──────────────────────────────────────────────────────────

impl PortfolioSystem {
    /// Filter components by a structured `PortfolioQuery`.
    pub fn query(&self, q: &PortfolioQuery) -> Vec<&PortfolioComponent> {
        self.components.values().filter(|c| {
            if let Some(ref types) = q.component_types {
                if !types.contains(&c.component_type) { return false; }
            }
            if let Some(ref statuses) = q.statuses {
                if !statuses.contains(&c.status) { return false; }
            }
            if let Some(ref tags) = q.tags {
                if !tags.iter().all(|t| c.metadata.tags.contains(t)) { return false; }
            }
            if let Some(ref substr) = q.name_contains {
                if !c.name.to_lowercase().contains(&substr.to_lowercase()) { return false; }
            }
            if let Some(ref owner) = q.owner {
                if !c.metadata.owners.contains(owner) { return false; }
            }
            if let Some(want_deps) = q.has_dependencies {
                if c.dependencies.is_empty() == want_deps { return false; }
            }
            if let Some(want_children) = q.has_children {
                if c.children.is_empty() == want_children { return false; }
            }
            if let Some((ref key, ref val)) = q.property_filter {
                if c.metadata.properties.get(key).map(String::as_str) != Some(val) { return false; }
            }
            if let Some(ref vis) = q.visibility {
                if &c.visibility != vis { return false; }
            }
            true
        }).collect()
    }

    pub fn query_pql(&self, pql: &str) -> Vec<&PortfolioComponent> {
        self.query(&PortfolioQuery::parse(pql))
    }

    /// Scored full-text + facet search.
    pub fn search(&self, q: &SearchQuery) -> Vec<SearchResult> {
        let mut results: Vec<SearchResult> = self.components.values()
            .filter(|c| {
                if c.status == ComponentStatus::Deleted { return false; }
                if !q.statuses.is_empty() && !q.statuses.contains(&c.status) { return false; }
                if !q.component_types.is_empty() && !q.component_types.contains(&c.component_type) { return false; }
                if let Some(ref owner) = q.owner {
                    if !c.metadata.owners.contains(owner) { return false; }
                }
                if let Some(ref vis) = q.visibility {
                    if &c.visibility != vis { return false; }
                }
                if let Some(after) = q.created_after {
                    if c.metadata.created_at < after { return false; }
                }
                if let Some(before) = q.created_before {
                    if c.metadata.created_at > before { return false; }
                }
                if !q.tags.is_empty() {
                    let ok = q.tags.iter().all(|t| c.metadata.tags.contains(t));
                    if !ok { return false; }
                }
                if !q.hashtags.is_empty() {
                    let ok = q.hashtags.iter().all(|h| c.hashtags.contains(h));
                    if !ok { return false; }
                }
                true
            })
            .map(|c| {
                let mut score = 1.0f64;
                let mut matched = Vec::new();
                if let Some(ref text) = q.text {
                    let tl = text.to_lowercase();
                    if c.name.to_lowercase().contains(&tl) { score += 3.0; matched.push("name".to_string()); }
                    if c.description.to_lowercase().contains(&tl) { score += 1.0; matched.push("description".to_string()); }
                    if c.metadata.tags.iter().any(|t| t.to_lowercase().contains(&tl)) { score += 1.5; matched.push("tags".to_string()); }
                    if c.hashtags.iter().any(|h| h.to_lowercase().contains(&tl)) { score += 1.5; matched.push("hashtags".to_string()); }
                }
                score += c.analytics.engagement_rate * 0.5;
                SearchResult {
                    component_id: c.id().to_string(),
                    name: c.name.clone(),
                    component_type: c.component_type.clone(),
                    status: c.status.clone(),
                    score,
                    matched_fields: matched,
                }
            }).collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        let offset = q.offset;
        results.into_iter().skip(offset).take(q.limit.unwrap_or(usize::MAX)).collect()
    }

    /// Summary metadata (counts, active portfolio, etc.).
    pub fn portfolio_metadata(&self) -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("component_count".into(), self.components.len().to_string());
        m.insert("edge_count".into(), self.edges.len().to_string());
        m.insert("event_count".into(), self.event_log.len().to_string());
        m.insert("snapshot_count".into(), self.snapshots.len().to_string());
        m.insert("checkpoint_count".into(), self.checkpoints.len().to_string());
        m.insert("active_portfolio_id".into(),
            self.active_portfolio_id.clone().unwrap_or_else(|| "none".into()));
        for (ct, count) in self.type_counts() {
            m.insert(format!("count_{ct:?}"), count.to_string());
        }
        m
    }

    fn type_counts(&self) -> HashMap<PortfolioComponentType, usize> {
        let mut map = HashMap::new();
        for c in self.components.values() {
            *map.entry(c.component_type.clone()).or_insert(0) += 1;
        }
        map
    }

    pub fn components_by_owner(&self, owner: &str) -> Vec<&PortfolioComponent> {
        self.components.values().filter(|c| c.metadata.owners.iter().any(|o| o == owner)).collect()
    }

    pub fn components_by_tag(&self, tag: &str) -> Vec<&PortfolioComponent> {
        self.components.values().filter(|c| c.metadata.tags.contains(&tag.to_string())).collect()
    }

    pub fn components_by_status(&self, status: &ComponentStatus) -> Vec<&PortfolioComponent> {
        self.components.values().filter(|c| &c.status == status).collect()
    }
}

// ── Analytics ─────────────────────────────────────────────────────────────────

impl PortfolioSystem {
    pub fn record_view(&mut self, id: &str, duration_seconds: u64) -> Result<(), String> {
        let comp = self.components.get_mut(id).ok_or_else(|| format!("'{id}' not found"))?;
        comp.analytics.record_view(duration_seconds);
        Ok(())
    }

    pub fn record_click(&mut self, id: &str) -> Result<(), String> {
        let comp = self.components.get_mut(id).ok_or_else(|| format!("'{id}' not found"))?;
        comp.analytics.record_click();
        Ok(())
    }

    pub fn analytics(&self, id: &str) -> Option<&ComponentAnalytics> {
        self.components.get(id).map(|c| &c.analytics)
    }

    pub fn benchmark(&self, ids: &[&str]) -> HashMap<ComponentId, ComponentAnalytics> {
        ids.iter().filter_map(|id| {
            self.components.get(*id).map(|c| ((*id).to_string(), c.analytics.clone()))
        }).collect()
    }

    pub fn platform_total_engagements(&self) -> u64 {
        self.components.values().map(|c| c.analytics.total_engagements()).sum()
    }
}

// ── Governance ────────────────────────────────────────────────────────────────

impl PortfolioSystem {
    pub fn register_policy_engine(&mut self, engine: Arc<dyn PolicyEngine>) {
        self.policy_engines.push(engine);
    }

    pub fn attach_policy(&mut self, component_id: &str, policy_id: &str) -> Result<(), String> {
        let comp = self.components.get_mut(component_id)
            .ok_or_else(|| format!("Component '{component_id}' not found"))?;
        comp.metadata.policy_ids.push(policy_id.to_string());
        self.emit(PortfolioEventKind::PolicyAttached, [
            ("component_id", component_id.to_string()),
            ("policy_id", policy_id.to_string()),
        ]);
        Ok(())
    }

    pub fn detach_policy(&mut self, component_id: &str, policy_id: &str) -> Result<(), String> {
        let comp = self.components.get_mut(component_id)
            .ok_or_else(|| format!("Component '{component_id}' not found"))?;
        comp.metadata.policy_ids.retain(|p| p != policy_id);
        self.emit(PortfolioEventKind::PolicyDetached, [
            ("component_id", component_id.to_string()),
            ("policy_id", policy_id.to_string()),
        ]);
        Ok(())
    }

    pub fn request_approval(&mut self, component_id: &str, reason: impl Into<String>) -> ApprovalRequest {
        let id  = format!("req-{:06}", self.next_req_id);
        self.next_req_id += 1;
        let req = ApprovalRequest {
            id: id.clone(),
            component_id: component_id.to_string(),
            requested_by: self.actor_id.clone(),
            reason: reason.into(),
            status: ApprovalStatus::Pending,
            created_at: now(),
            resolved_at: None,
            resolver: None,
            notes: None,
        };
        self.approval_requests.push(req.clone());
        self.emit(PortfolioEventKind::ApprovalRequested, [
            ("request_id", id), ("component_id", component_id.to_string()),
        ]);
        req
    }

    pub fn resolve_approval(
        &mut self,
        request_id: &str,
        approved: bool,
        resolver: impl Into<ActorId>,
        notes: Option<String>,
    ) -> Result<ApprovalRequest, String> {
        let req = self.approval_requests.iter_mut()
            .find(|r| r.id == request_id)
            .ok_or_else(|| format!("Approval request '{request_id}' not found"))?;
        req.status       = if approved { ApprovalStatus::Approved } else { ApprovalStatus::Rejected };
        req.resolved_at  = Some(now());
        req.resolver     = Some(resolver.into());
        req.notes        = notes;
        let result = req.clone();
        self.emit(
            if approved { PortfolioEventKind::ApprovalGranted } else { PortfolioEventKind::ApprovalRejected },
            [("request_id", request_id.to_string())],
        );
        Ok(result)
    }

    pub fn allocate_resource(
        &mut self,
        component_id: &str,
        kind: ResourceKind,
        total: f64,
        denomination: impl Into<String>,
        period: Option<String>,
    ) -> Result<ResourceAllocation, String> {
        if !self.components.contains_key(component_id) {
            return Err(format!("Component '{component_id}' not found"));
        }
        let denom = denomination.into();
        let alloc = ResourceAllocation {
            component_id: component_id.to_string(),
            kind,
            total,
            allocated: total,
            consumed: 0.0,
            denomination: denom.clone(),
            period,
        };
        self.resource_allocations.insert(component_id.to_string(), alloc.clone());
        if let Some(comp) = self.components.get_mut(component_id) { comp.metadata.budget = Some(total); }
        self.emit(PortfolioEventKind::ResourceAllocated, [
            ("component_id", component_id.to_string()),
            ("total", total.to_string()),
            ("denomination", denom),
        ]);
        Ok(alloc)
    }

    pub fn allocate_budget(
        &mut self,
        component_id: &str,
        total: f64,
        currency: impl Into<String>,
        period: Option<String>,
    ) -> Result<ResourceAllocation, String> {
        self.allocate_resource(component_id, ResourceKind::Budget, total, currency, period)
    }

    pub fn record_consumption(&mut self, component_id: &str, amount: f64) -> Result<f64, String> {
        let alloc = self.resource_allocations.get_mut(component_id)
            .ok_or_else(|| format!("No resource allocated for '{component_id}'"))?;
        alloc.consumed += amount;
        let remaining = alloc.remaining();
        if let Some(comp) = self.components.get_mut(component_id) { comp.metadata.budget_spent += amount; }
        self.emit(PortfolioEventKind::ResourceConsumed, [
            ("component_id", component_id.to_string()),
            ("amount", amount.to_string()),
            ("remaining", remaining.to_string()),
        ]);
        Ok(remaining)
    }

    /// Backward-compat alias for `record_consumption`.
    pub fn record_spend(&mut self, component_id: &str, amount: f64) -> Result<f64, String> {
        self.record_consumption(component_id, amount)
    }

    pub fn get_resource_allocation(&self, component_id: &str) -> Option<&ResourceAllocation> {
        self.resource_allocations.get(component_id)
    }

    pub fn overrun_allocations(&self) -> Vec<&ResourceAllocation> {
        self.resource_allocations.values().filter(|a| a.is_overrun()).collect()
    }

    pub fn total_consumption_by_kind(&self, kind: &ResourceKind) -> f64 {
        self.resource_allocations.values()
            .filter(|a| &a.kind == kind).map(|a| a.consumed).sum()
    }
}

// ── Version control ───────────────────────────────────────────────────────────

impl PortfolioSystem {
    pub fn bump_version(&mut self, actor_id: &str, component_id: &str, part: VersionPart, message: &str) -> Result<VersionString, String> {
        let comp = self.components.get_mut(component_id)
            .ok_or_else(|| format!("Component '{component_id}' not found"))?;
        if !comp.users.can(actor_id, &ActionKind::Edit) {
            return Err(format!("Actor '{actor_id}' cannot edit component '{component_id}'"));
        }
        comp.bump_version(actor_id, part, message);
        let v = comp.metadata.semver.clone();
        Ok(v)
    }
}

// ── Plugin registration ───────────────────────────────────────────────────────

impl PortfolioSystem {
    pub fn register_plugin(&mut self, plugin: Arc<dyn PortfolioPlugin>) {
        self.plugins.push(plugin);
    }
    pub fn unregister_plugin(&mut self, plugin_id: &str) {
        self.plugins.retain(|p| p.id() != plugin_id);
    }
}

// ── CRDT / distributed merge ──────────────────────────────────────────────────

impl PortfolioSystem {
    pub fn apply_crdt_log(&mut self, remote: &CrdtLog) {
        let ops: Vec<CrdtOperation> = remote.operations.clone();
        for op in &ops { self.apply_crdt_op(op); }
        self.crdt.merge(remote);
        self.emit(PortfolioEventKind::CrdtMergeApplied, [("op_count", ops.len().to_string())]);
        for plugin in &self.plugins { plugin.on_crdt_merge(&ops); }
    }

    fn apply_crdt_op(&mut self, op: &CrdtOperation) {
        match op {
            CrdtOperation::SetField { component_id, field, value, timestamp, actor } => {
                if let Some(comp) = self.components.get_mut(component_id) {
                    let accept = comp.metadata.updated_at < *timestamp
                        || (comp.metadata.updated_at == *timestamp && comp.metadata.actor_id < *actor);
                    if accept {
                        match field.as_str() {
                            "name"   => comp.name = value.clone(),
                            "status" => comp.status = parse_status(value),
                            _        => { comp.metadata.properties.insert(field.clone(), value.clone()); }
                        }
                        comp.metadata.updated_at = *timestamp;
                        comp.metadata.actor_id = actor.clone();
                    }
                }
            }
            CrdtOperation::AddToSet { component_id, set_name, element, .. } => {
                if let Some(comp) = self.components.get_mut(component_id) {
                    match set_name.as_str() {
                        "tags"         => { if !comp.metadata.tags.contains(element) { comp.metadata.tags.push(element.clone()); } }
                        "dependencies" => { comp.dependencies.insert(element.clone()); }
                        "links"        => { comp.links.insert(element.clone()); }
                        "members"      => { comp.unordered_members.insert(element.clone()); }
                        "hashtags"     => { comp.hashtags.insert(element.clone()); }
                        _              => {}
                    }
                }
            }
            CrdtOperation::RemoveFromSet { component_id, set_name, element, .. } => {
                if let Some(comp) = self.components.get_mut(component_id) {
                    match set_name.as_str() {
                        "tags"         => comp.metadata.tags.retain(|t| t != element),
                        "dependencies" => { comp.dependencies.remove(element); }
                        "links"        => { comp.links.remove(element); }
                        "members"      => { comp.unordered_members.remove(element); }
                        "hashtags"     => { comp.hashtags.remove(element); }
                        _              => {}
                    }
                }
            }
            CrdtOperation::AddEdge { edge, .. } => {
                if !self.edges.iter().any(|e| e.id == edge.id) { self.edges.push(edge.clone()); }
            }
            CrdtOperation::RemoveEdge { edge_id, .. } => {
                self.edges.retain(|e| &e.id != edge_id);
            }
        }
    }
}

// ── Structural primitive management ──────────────────────────────────────────

impl PortfolioSystem {
    // Groups
    pub fn create_group(&mut self, name: impl Into<String>, owner: impl Into<ActorId>) -> String {
        let g = Group::new(name, owner); let id = g.id.clone(); self.groups.insert(id.clone(), g); id
    }
    pub fn group(&self, id: &str) -> Option<&Group>         { self.groups.get(id) }
    pub fn group_mut(&mut self, id: &str) -> Option<&mut Group> { self.groups.get_mut(id) }

    // Collections
    pub fn create_collection(&mut self, name: impl Into<String>, owner: impl Into<ActorId>) -> String {
        let c = Collection::new(name, owner); let id = c.id.clone(); self.collections.insert(id.clone(), c); id
    }
    pub fn collection(&self, id: &str) -> Option<&Collection>         { self.collections.get(id) }
    pub fn collection_mut(&mut self, id: &str) -> Option<&mut Collection> { self.collections.get_mut(id) }

    // Lists
    pub fn create_list(&mut self, name: impl Into<String>, owner: impl Into<ActorId>) -> String {
        let l = List::new(name, owner); let id = l.id.clone(); self.lists.insert(id.clone(), l); id
    }
    pub fn list(&self, id: &str) -> Option<&List>         { self.lists.get(id) }
    pub fn list_mut(&mut self, id: &str) -> Option<&mut List> { self.lists.get_mut(id) }

    // Schedules
    pub fn create_schedule(&mut self, name: impl Into<String>, owner: impl Into<ActorId>) -> String {
        let s = Schedule::new(name, owner); let id = s.id.clone(); self.schedules.insert(id.clone(), s); id
    }
    pub fn schedule(&self, id: &str) -> Option<&Schedule>         { self.schedules.get(id) }
    pub fn schedule_mut(&mut self, id: &str) -> Option<&mut Schedule> { self.schedules.get_mut(id) }

    // Directories
    pub fn create_directory(&mut self, name: impl Into<String>, path: impl Into<String>, owner: impl Into<ActorId>) -> String {
        let d = Directory::new(name, path, owner); let id = d.id.clone(); self.directories.insert(id.clone(), d); id
    }
    pub fn directory(&self, id: &str) -> Option<&Directory>         { self.directories.get(id) }
    pub fn directory_mut(&mut self, id: &str) -> Option<&mut Directory> { self.directories.get_mut(id) }
}

// =============================================================================
// §22 — OS-BRIDGE ADAPTER  (backward-compat shim — inlined, no external deps)
// =============================================================================

/// Backward-compat request shape retained for callers using `NewPortfolioItem`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewPortfolioItem {
    pub item_type: String,
    pub name: String,
    pub status: String,
}

/// Legacy entity discriminant (maps to `PortfolioComponentType` via bridge).
#[derive(Clone, Debug, PartialEq)]
pub enum PortfolioEntityType {
    Portfolio, Project, Program, SubPortfolio,
    Resource, Capital, Investment, Account,
    Land, Estate, Labor,
    Artifact, Asset, Document, Custom,
}

/// Legacy item type (top-level).
#[derive(Clone, Debug, PartialEq)]
pub enum PortfolioItemType {
    Project, Program, SubPortfolio, Resource,
}

/// Legacy resource sub-type.
#[derive(Clone, Debug, PartialEq)]
pub enum LegacyResourceType {
    Artifact, Asset, Capital, Investment, Account, Land, Estate, Labor,
}

/// Parse a legacy `item_type` string into `(PortfolioItemType, Option<LegacyResourceType>)`.
pub fn parse_portfolio_kind(s: &str) -> (PortfolioItemType, Option<LegacyResourceType>) {
    match s.to_lowercase().as_str() {
        "project"      => (PortfolioItemType::Project,      None),
        "program"      => (PortfolioItemType::Program,      None),
        "subportfolio" => (PortfolioItemType::SubPortfolio, None),
        "artifact"     => (PortfolioItemType::Resource,     Some(LegacyResourceType::Artifact)),
        "asset"        => (PortfolioItemType::Resource,     Some(LegacyResourceType::Asset)),
        "capital"      => (PortfolioItemType::Resource,     Some(LegacyResourceType::Capital)),
        "investment"   => (PortfolioItemType::Resource,     Some(LegacyResourceType::Investment)),
        "account"      => (PortfolioItemType::Resource,     Some(LegacyResourceType::Account)),
        "land"         => (PortfolioItemType::Resource,     Some(LegacyResourceType::Land)),
        "estate"       => (PortfolioItemType::Resource,     Some(LegacyResourceType::Estate)),
        "labor"        => (PortfolioItemType::Resource,     Some(LegacyResourceType::Labor)),
        _              => (PortfolioItemType::Resource,     None),
    }
}

fn entity_type_to_component_type(et: &PortfolioEntityType) -> PortfolioComponentType {
    match et {
        PortfolioEntityType::Portfolio    => PortfolioComponentType::Portfolio,
        PortfolioEntityType::Project      => PortfolioComponentType::Project,
        PortfolioEntityType::Program      => PortfolioComponentType::Program,
        PortfolioEntityType::SubPortfolio => PortfolioComponentType::SubPortfolio,
        PortfolioEntityType::Artifact     => PortfolioComponentType::Artifact,
        PortfolioEntityType::Asset        => PortfolioComponentType::Asset,
        PortfolioEntityType::Document     => PortfolioComponentType::Artifact,
        PortfolioEntityType::Custom       => PortfolioComponentType::Resource,
        _                                 => PortfolioComponentType::Resource,
    }
}

fn parse_entity_type(item_type: &str) -> PortfolioEntityType {
    let (kind, resource_type) = parse_portfolio_kind(item_type);
    match (kind, resource_type) {
        (PortfolioItemType::Project,      _)                              => PortfolioEntityType::Project,
        (PortfolioItemType::Program,      _)                              => PortfolioEntityType::Program,
        (PortfolioItemType::SubPortfolio, _)                              => PortfolioEntityType::SubPortfolio,
        (PortfolioItemType::Resource,     Some(LegacyResourceType::Artifact))   => PortfolioEntityType::Artifact,
        (PortfolioItemType::Resource,     Some(LegacyResourceType::Asset))      => PortfolioEntityType::Asset,
        (PortfolioItemType::Resource,     Some(LegacyResourceType::Capital))    => PortfolioEntityType::Capital,
        (PortfolioItemType::Resource,     Some(LegacyResourceType::Investment)) => PortfolioEntityType::Investment,
        (PortfolioItemType::Resource,     Some(LegacyResourceType::Account))    => PortfolioEntityType::Account,
        (PortfolioItemType::Resource,     Some(LegacyResourceType::Land))       => PortfolioEntityType::Land,
        (PortfolioItemType::Resource,     Some(LegacyResourceType::Estate))     => PortfolioEntityType::Estate,
        (PortfolioItemType::Resource,     Some(LegacyResourceType::Labor))      => PortfolioEntityType::Labor,
        (PortfolioItemType::Resource,     None)                           => PortfolioEntityType::Resource,
    }
}

impl PortfolioSystem {
    /// Backward-compatible entry-point: create a component from the legacy
    /// `NewPortfolioItem` shape and attach it to the active portfolio.
    pub fn add_item(&mut self, req: NewPortfolioItem) -> Result<PortfolioComponent, String> {
        let ct    = entity_type_to_component_type(&parse_entity_type(&req.item_type));
        let mut comp = self.create_component(ct, req.name)?;
        comp.status = parse_status(&req.status);
        if let Some(c) = self.components.get_mut(&comp.metadata.id) {
            c.status = comp.status.clone();
        }
        if let Some(pid) = self.active_portfolio_id.clone() {
            self.add_member(&pid, &comp.metadata.id).ok();
        }
        Ok(comp)
    }
}

// =============================================================================
// §23 — COMPUTATIONAL MODELS
// =============================================================================
//
// Each component type has a dedicated analytics / scoring model.
// Models are pure-function structs; they borrow data from PortfolioSystem (or
// PortfolioComponent directly) and return typed result objects.
//
// Component type → Model:
//   Portfolio    → PortfolioHealthModel    (health score, resource roll-up)
//   Project      → ProjectMetricsModel     (schedule, CPI/SPI, risk)
//   Program      → ProgramAlignmentModel   (benefit realisation, coherence)
//   SubPortfolio → SubPortfolioRollupModel (aggregated child metrics)
//   Resource     → ResourceUtilisationModel (capacity vs demand)
//   Asset        → AssetValueModel         (depreciation, ROI)
//   Artifact     → ArtifactMaturityModel   (completeness, currency, reuse)
//   Binder       → BinderCoverageModel     (membership completeness)
//   Book         → BookConsistencyModel    (page completeness by BookType)
//   Folder       → FolderOrganisationModel (depth, orphan detection)
//   Record       → RecordIntegrityModel    (sequence gaps, duplicate entries)
// =============================================================================

// ── 23.1  Portfolio Health ────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct PortfolioHealthInput<'a> {
    pub portfolio: &'a PortfolioComponent,
    pub children: Vec<&'a PortfolioComponent>,
    pub allocation: Option<&'a ResourceAllocation>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortfolioHealthResult {
    pub health_score: f64,
    pub active_ratio: f64,
    pub resource_utilisation_pct: f64,
    pub total_resource_units: f64,
    pub summary: String,
}

pub struct PortfolioHealthModel;
impl PortfolioHealthModel {
    pub fn compute(input: &PortfolioHealthInput<'_>) -> PortfolioHealthResult {
        let total  = input.children.len();
        let active = input.children.iter().filter(|c| c.status == ComponentStatus::Active).count();
        let active_ratio = if total == 0 { 0.0 } else { active as f64 / total as f64 };
        let utilisation_pct = input.allocation.map(|a| a.utilisation_pct()).unwrap_or(0.0);
        let total_resource_units: f64 = input.children.iter().map(|c| c.metadata.resource_units).sum();
        let activity_pts = if active_ratio >= 0.8 { 50.0 } else { active_ratio * 62.5 };
        let resource_pts = if utilisation_pct > 120.0 { 0.0 }
            else { 30.0 * (1.0 - ((utilisation_pct - 100.0).max(0.0) / 20.0)) };
        let coverage_pts = if total > 0 { 20.0 } else { 0.0 };
        let health_score = (activity_pts + resource_pts + coverage_pts).clamp(0.0, 100.0);
        PortfolioHealthResult {
            health_score, active_ratio, resource_utilisation_pct: utilisation_pct, total_resource_units,
            summary: format!("Health {health_score:.1}/100 — {active}/{total} active, util {utilisation_pct:.1}%"),
        }
    }
}

// ── 23.2  Project Metrics (Earned Value) ──────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectMetricsInput {
    pub planned_value: f64,
    pub earned_value: f64,
    pub actual_cost: f64,
    pub budget_at_completion: f64,
    pub schedule_risk_weight: f64,
    pub open_risk_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectMetricsResult {
    pub cpi: f64, pub spi: f64,
    pub cost_variance: f64, pub schedule_variance: f64,
    pub estimate_at_completion: f64, pub estimate_to_complete: f64,
    pub risk_score: f64, pub summary: String,
}

pub struct ProjectMetricsModel;
impl ProjectMetricsModel {
    pub fn compute(i: &ProjectMetricsInput) -> ProjectMetricsResult {
        let cpi = if i.actual_cost == 0.0 { 1.0 } else { i.earned_value / i.actual_cost };
        let spi = if i.planned_value == 0.0 { 1.0 } else { i.earned_value / i.planned_value };
        let eac = if cpi == 0.0 { f64::INFINITY } else { i.budget_at_completion / cpi };
        let etc = eac - i.actual_cost;
        let risk_score = ((1.0 - cpi.min(1.0)) * 40.0
            + (1.0 - spi.min(1.0)) * i.schedule_risk_weight * 40.0
            + (i.open_risk_count as f64 * 5.0).min(20.0)).clamp(0.0, 100.0);
        ProjectMetricsResult {
            cpi, spi, cost_variance: i.earned_value - i.actual_cost,
            schedule_variance: i.earned_value - i.planned_value,
            estimate_at_completion: eac, estimate_to_complete: etc, risk_score,
            summary: format!("CPI={cpi:.2} SPI={spi:.2} EAC={eac:.0} risk={risk_score:.1}/100"),
        }
    }
}

// ── 23.3  Program Alignment ───────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct ProgramAlignmentInput<'a> {
    pub program: &'a PortfolioComponent,
    pub children: Vec<&'a PortfolioComponent>,
    pub strategic_goal_ids: Vec<String>,
    pub covered_goal_ids: Vec<String>,
    pub planned_benefit: f64,
    pub realised_benefit: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProgramAlignmentResult {
    pub benefit_realisation_index: f64,
    pub strategic_coverage: f64,
    pub coherence: f64,
    pub alignment_score: f64,
    pub summary: String,
}

pub struct ProgramAlignmentModel;
impl ProgramAlignmentModel {
    pub fn compute(i: &ProgramAlignmentInput<'_>) -> ProgramAlignmentResult {
        let bri = if i.planned_benefit == 0.0 { 1.0 } else { i.realised_benefit / i.planned_benefit };
        let strategic_coverage = if i.strategic_goal_ids.is_empty() { 1.0 } else {
            i.covered_goal_ids.iter().filter(|g| i.strategic_goal_ids.contains(g)).count() as f64
                / i.strategic_goal_ids.len() as f64
        };
        let total  = i.children.len();
        let active = i.children.iter().filter(|c| c.status == ComponentStatus::Active).count();
        let coherence = if total == 0 { 0.0 } else { active as f64 / total as f64 };
        let alignment_score = (bri.min(1.0) * 40.0 + strategic_coverage * 40.0 + coherence * 20.0).clamp(0.0, 100.0);
        ProgramAlignmentResult {
            benefit_realisation_index: bri, strategic_coverage, coherence, alignment_score,
            summary: format!("Alignment {alignment_score:.1}/100 — BRI={bri:.2} coverage={:.0}% coherence={:.0}%",
                strategic_coverage * 100.0, coherence * 100.0),
        }
    }
}

// ── 23.4  Sub-Portfolio Rollup ────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubPortfolioRollupResult {
    pub total_components: usize, pub active_components: usize,
    pub total_resource_units: f64,
    pub total_budget_allocated: f64, pub total_budget_consumed: f64,
    pub avg_child_health: f64, pub rollup_score: f64, pub summary: String,
}

pub struct SubPortfolioRollupModel;
impl SubPortfolioRollupModel {
    pub fn compute(children: &[&PortfolioComponent], allocs: &HashMap<ComponentId, ResourceAllocation>) -> SubPortfolioRollupResult {
        let total  = children.len();
        let active = children.iter().filter(|c| c.status == ComponentStatus::Active).count();
        let total_resource_units: f64 = children.iter().map(|c| c.metadata.resource_units).sum();
        let (alloc_sum, cons_sum) = children.iter().fold((0.0_f64, 0.0_f64), |(a, b), c| {
            if let Some(al) = allocs.get(c.id()) { (a + al.allocated, b + al.consumed) }
            else { (a, b) }
        });
        let active_ratio = if total == 0 { 0.0 } else { active as f64 / total as f64 };
        let budget_health = if alloc_sum == 0.0 { 1.0 }
            else { (1.0 - ((cons_sum / alloc_sum) - 1.0).max(0.0)).clamp(0.0, 1.0) };
        let rollup_score = (active_ratio * 60.0 + budget_health * 40.0).clamp(0.0, 100.0);
        SubPortfolioRollupResult {
            total_components: total, active_components: active, total_resource_units,
            total_budget_allocated: alloc_sum, total_budget_consumed: cons_sum,
            avg_child_health: rollup_score, rollup_score,
            summary: format!("Rollup {rollup_score:.1}/100 — {active}/{total} active, budget {cons_sum:.0}/{alloc_sum:.0}"),
        }
    }
}

// ── 23.5  Resource Utilisation ────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct ResourceUtilisationInput {
    pub total_capacity: f64,
    pub committed: f64,
    pub consumed: f64,
    pub assigned_items: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceUtilisationResult {
    pub commitment_ratio: f64, pub consumption_ratio: f64,
    pub slack: f64, pub efficiency: f64, pub utilisation_score: f64, pub summary: String,
}

pub struct ResourceUtilisationModel;
impl ResourceUtilisationModel {
    pub fn compute(i: &ResourceUtilisationInput) -> ResourceUtilisationResult {
        let cr = if i.total_capacity == 0.0 { 0.0 } else { i.committed / i.total_capacity };
        let xr = if i.total_capacity == 0.0 { 0.0 } else { i.consumed  / i.total_capacity };
        let eff = if i.committed == 0.0 { 1.0 } else { (i.consumed / i.committed).min(2.0) };
        let over_commit = ((cr - 1.0).max(0.0) * 100.0).min(60.0);
        let idle = if cr < 0.4 { (0.4 - cr) * 50.0 } else { 0.0 };
        let score = (100.0 - over_commit - idle).clamp(0.0, 100.0);
        ResourceUtilisationResult {
            commitment_ratio: cr, consumption_ratio: xr, slack: i.total_capacity - i.committed,
            efficiency: eff, utilisation_score: score,
            summary: format!("Util {score:.1}/100 — committed={:.0}% consumed={:.0}% slack={:.1}",
                cr * 100.0, xr * 100.0, i.total_capacity - i.committed),
        }
    }
}

// ── 23.6  Asset Value ─────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetValueInput {
    pub acquisition_cost: f64, pub current_book_value: f64, pub market_value: f64,
    pub total_return: f64, pub age_periods: f64, pub useful_life_periods: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetValueResult {
    pub roi: f64, pub depreciation_rate: f64, pub life_consumed_ratio: f64,
    pub market_to_book_ratio: f64, pub asset_score: f64, pub summary: String,
}

pub struct AssetValueModel;
impl AssetValueModel {
    pub fn compute(i: &AssetValueInput) -> AssetValueResult {
        let roi  = if i.acquisition_cost == 0.0 { 0.0 } else { i.total_return / i.acquisition_cost };
        let dr   = if i.useful_life_periods == 0.0 { 0.0 } else { 1.0 / i.useful_life_periods };
        let lcr  = if i.useful_life_periods == 0.0 { 0.0 } else { i.age_periods / i.useful_life_periods };
        let mtb  = if i.current_book_value == 0.0 { 1.0 } else { i.market_value / i.current_book_value };
        let score = ((roi * 40.0).clamp(0.0, 40.0)
            + ((mtb - 1.0).clamp(-1.0, 1.0) * 30.0 + 30.0).clamp(0.0, 30.0)
            + ((1.0 - lcr) * 30.0).clamp(0.0, 30.0)).clamp(0.0, 100.0);
        AssetValueResult {
            roi, depreciation_rate: dr, life_consumed_ratio: lcr, market_to_book_ratio: mtb, asset_score: score,
            summary: format!("Asset {score:.1}/100 — ROI={:.1}% MTB={mtb:.2} life={:.0}%", roi * 100.0, lcr * 100.0),
        }
    }
}

// ── 23.7  Artifact Maturity ───────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct ArtifactMaturityInput {
    pub required_fields_populated: usize,
    pub required_fields_total: usize,
    pub days_since_update: u64,
    pub max_fresh_days: u64,
    pub reuse_count: usize,
    pub is_reviewed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtifactMaturityResult {
    pub completeness: f64, pub currency: f64,
    pub reuse_score_component: f64, pub maturity_score: f64, pub summary: String,
}

pub struct ArtifactMaturityModel;
impl ArtifactMaturityModel {
    pub fn compute(i: &ArtifactMaturityInput) -> ArtifactMaturityResult {
        let completeness = if i.required_fields_total == 0 { 1.0 }
            else { i.required_fields_populated as f64 / i.required_fields_total as f64 };
        let currency = if i.days_since_update <= i.max_fresh_days { 1.0 }
            else { (1.0 - (i.days_since_update - i.max_fresh_days) as f64 / i.max_fresh_days as f64).clamp(0.0, 1.0) };
        let reuse = ((i.reuse_count as f64 + 1.0).ln() / 11_f64.ln()).min(1.0);
        let score = (completeness * 40.0 + currency * 30.0 + reuse * 20.0
            + if i.is_reviewed { 10.0 } else { 0.0 }).clamp(0.0, 100.0);
        ArtifactMaturityResult {
            completeness, currency, reuse_score_component: reuse, maturity_score: score,
            summary: format!("Maturity {score:.1}/100 — complete={:.0}% currency={:.0}% reuse={}",
                completeness * 100.0, currency * 100.0, i.reuse_count),
        }
    }
}

// ── 23.8  Binder Coverage ────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct BinderCoverageInput {
    pub expected_ids: HashSet<ComponentId>,
    pub actual_ids: HashSet<ComponentId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BinderCoverageResult {
    pub coverage_ratio: f64, pub missing: Vec<String>, pub extra: Vec<String>,
    pub coverage_score: f64, pub summary: String,
}

pub struct BinderCoverageModel;
impl BinderCoverageModel {
    pub fn compute(i: &BinderCoverageInput) -> BinderCoverageResult {
        let mut missing: Vec<String> = i.expected_ids.difference(&i.actual_ids).cloned().collect(); missing.sort();
        let mut extra:   Vec<String> = i.actual_ids.difference(&i.expected_ids).cloned().collect(); extra.sort();
        let cr = if i.expected_ids.is_empty() { 1.0 }
            else { i.expected_ids.intersection(&i.actual_ids).count() as f64 / i.expected_ids.len() as f64 };
        let extra_penalty = ((extra.len() as f64 / (i.actual_ids.len() as f64 + 1.0)) * 10.0).min(10.0);
        let score = (cr * 100.0 - extra_penalty).clamp(0.0, 100.0);
        BinderCoverageResult {
            coverage_ratio: cr, missing: missing.clone(), extra: extra.clone(), coverage_score: score,
            summary: format!("Coverage {score:.1}/100 — {:.0}% present, {} missing, {} extra",
                cr * 100.0, missing.len(), extra.len()),
        }
    }
}

// ── 23.9  Book Consistency ───────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BookConsistencyInput {
    pub book_type: BookType,
    pub page_count: usize,
    pub min_expected_pages: usize,
    pub reviewed_pages: usize,
    pub broken_refs: usize,
    pub has_index: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BookConsistencyResult {
    pub page_completeness: f64, pub review_coverage: f64,
    pub reference_integrity: f64, pub consistency_score: f64, pub summary: String,
}

pub struct BookConsistencyModel;
impl BookConsistencyModel {
    pub fn compute(i: &BookConsistencyInput) -> BookConsistencyResult {
        let pc = if i.min_expected_pages == 0 { 1.0 }
            else { (i.page_count as f64 / i.min_expected_pages as f64).min(1.0) };
        let rc = if i.page_count == 0 { 1.0 } else { i.reviewed_pages as f64 / i.page_count as f64 };
        let ri = if i.page_count == 0 { 1.0 }
            else { (1.0 - i.broken_refs as f64 / i.page_count as f64).clamp(0.0, 1.0) };
        let score = (pc * 40.0 + rc * 30.0 + ri * 20.0 + if i.has_index { 10.0 } else { 0.0 }).clamp(0.0, 100.0);
        BookConsistencyResult {
            page_completeness: pc, review_coverage: rc, reference_integrity: ri, consistency_score: score,
            summary: format!("[{:?}] Consistency {score:.1}/100 — complete={:.0}% reviewed={:.0}% integrity={:.0}%",
                i.book_type, pc * 100.0, rc * 100.0, ri * 100.0),
        }
    }
}

// ── 23.10  Folder Organisation ────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct FolderOrganisationInput {
    pub max_depth: usize, pub depth_threshold: usize,
    pub direct_item_count: usize, pub orphan_count: usize, pub duplicate_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FolderOrganisationResult {
    pub depth_penalty: f64, pub organisation_score: f64, pub summary: String,
}

pub struct FolderOrganisationModel;
impl FolderOrganisationModel {
    pub fn compute(i: &FolderOrganisationInput) -> FolderOrganisationResult {
        let dp = if i.max_depth <= i.depth_threshold { 0.0 }
            else { ((i.max_depth - i.depth_threshold) as f64 / i.depth_threshold as f64).clamp(0.0, 1.0) };
        let score = (100.0 - dp * 30.0 - (i.orphan_count as f64 * 5.0).min(30.0)
            - (i.duplicate_count as f64 * 5.0).min(20.0)).clamp(0.0, 100.0);
        FolderOrganisationResult {
            depth_penalty: dp, organisation_score: score,
            summary: format!("Organisation {score:.1}/100 — depth={} orphans={} dupes={}",
                i.max_depth, i.orphan_count, i.duplicate_count),
        }
    }
}

// ── 23.11  Record Integrity ───────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct RecordIntegrityInput {
    pub entry_count: usize, pub out_of_order_count: usize,
    pub duplicate_count: usize, pub incomplete_entries: usize,
    pub has_integrity_hash: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecordIntegrityResult {
    pub sequence_quality: f64, pub uniqueness_ratio: f64,
    pub completeness_ratio: f64, pub integrity_score: f64, pub summary: String,
}

pub struct RecordIntegrityModel;
impl RecordIntegrityModel {
    pub fn compute(i: &RecordIntegrityInput) -> RecordIntegrityResult {
        let base = i.entry_count.max(1) as f64;
        let sq = (1.0 - i.out_of_order_count as f64 / base).clamp(0.0, 1.0);
        let ur = (1.0 - i.duplicate_count   as f64 / base).clamp(0.0, 1.0);
        let cr = (1.0 - i.incomplete_entries as f64 / base).clamp(0.0, 1.0);
        let score = (sq * 35.0 + ur * 30.0 + cr * 25.0 + if i.has_integrity_hash { 10.0 } else { 0.0 }).clamp(0.0, 100.0);
        RecordIntegrityResult {
            sequence_quality: sq, uniqueness_ratio: ur, completeness_ratio: cr, integrity_score: score,
            summary: format!("Integrity {score:.1}/100 — seq={:.0}% unique={:.0}% complete={:.0}%",
                sq * 100.0, ur * 100.0, cr * 100.0),
        }
    }
}

// ── 23.12  Computational model registry (PortfolioSystem integration) ─────────

impl PortfolioSystem {
    pub fn compute_portfolio_health(&self, portfolio_id: &str) -> Option<PortfolioHealthResult> {
        let p = self.components.get(portfolio_id)?;
        let children: Vec<&PortfolioComponent> = p.children.iter()
            .filter_map(|id| self.components.get(id)).collect();
        Some(PortfolioHealthModel::compute(&PortfolioHealthInput {
            portfolio: p, children, allocation: self.resource_allocations.get(portfolio_id),
        }))
    }

    pub fn compute_subportfolio_rollup(&self, id: &str) -> Option<SubPortfolioRollupResult> {
        let sp = self.components.get(id)?;
        let children: Vec<&PortfolioComponent> = sp.children.iter()
            .filter_map(|id| self.components.get(id)).collect();
        Some(SubPortfolioRollupModel::compute(&children, &self.resource_allocations))
    }

    pub fn compute_resource_utilisation(&self, resource_id: &str) -> Option<ResourceUtilisationResult> {
        let comp = self.components.get(resource_id)?;
        if comp.component_type != PortfolioComponentType::Resource { return None; }
        let alloc = self.resource_allocations.get(resource_id);
        let total    = alloc.map(|a| a.total).unwrap_or(comp.metadata.resource_units);
        let committed = alloc.map(|a| a.allocated).unwrap_or(0.0);
        let consumed  = alloc.map(|a| a.consumed).unwrap_or(comp.metadata.budget_spent);
        Some(ResourceUtilisationModel::compute(&ResourceUtilisationInput {
            total_capacity: total, committed, consumed,
            assigned_items: comp.unordered_members.len() + comp.ordered_members.len(),
        }))
    }

    pub fn compute_artifact_maturity(
        &self, artifact_id: &str, required_fields: &[&str], max_fresh_days: u64,
    ) -> Option<ArtifactMaturityResult> {
        let comp = self.components.get(artifact_id)?;
        if comp.component_type != PortfolioComponentType::Artifact { return None; }
        let days = (now() - comp.metadata.updated_at) / 86_400;
        let populated = required_fields.iter().filter(|k| comp.metadata.properties.contains_key(**k)).count();
        let is_reviewed = comp.metadata.properties.get("reviewed").map(|v| v == "true").unwrap_or(false);
        Some(ArtifactMaturityModel::compute(&ArtifactMaturityInput {
            required_fields_populated: populated,
            required_fields_total: required_fields.len(),
            days_since_update: days, max_fresh_days,
            reuse_count: comp.dependents.len(), is_reviewed,
        }))
    }

    pub fn compute_binder_coverage(&self, binder_id: &str, expected: HashSet<ComponentId>) -> Option<BinderCoverageResult> {
        let b = self.components.get(binder_id)?;
        if b.component_type != PortfolioComponentType::Binder { return None; }
        Some(BinderCoverageModel::compute(&BinderCoverageInput {
            expected_ids: expected, actual_ids: b.unordered_members.clone(),
        }))
    }

    pub fn compute_record_integrity(&self, record_id: &str) -> Option<RecordIntegrityResult> {
        let r = self.components.get(record_id)?;
        if r.component_type != PortfolioComponentType::Record { return None; }
        let mut seen: HashMap<&String, usize> = HashMap::new();
        let mut dupes = 0;
        for m in &r.ordered_members {
            let cnt = seen.entry(m).or_insert(0);
            *cnt += 1;
            if *cnt == 2 { dupes += 1; }
        }
        Some(RecordIntegrityModel::compute(&RecordIntegrityInput {
            entry_count: r.ordered_members.len(),
            out_of_order_count: 0,
            duplicate_count: dupes,
            incomplete_entries: 0,
            has_integrity_hash: r.metadata.properties.contains_key("integrity_hash"),
        }))
    }

    pub fn compute_folder_organisation(&self, folder_id: &str, depth_threshold: usize) -> Option<FolderOrganisationResult> {
        let f = self.components.get(folder_id)?;
        if f.component_type != PortfolioComponentType::Folder { return None; }
        let max_depth = self.folder_max_depth(folder_id, 0);
        let direct_items = f.unordered_members.iter()
            .filter(|id| self.components.get(id.as_str()).map(|c| !c.is_container()).unwrap_or(false)).count();
        let orphans = f.unordered_members.iter()
            .filter(|id| self.components.get(id.as_str()).map(|c| !c.parents.contains(folder_id)).unwrap_or(true)).count();
        let mut freq: HashMap<&str, usize> = HashMap::new();
        for id in &f.unordered_members {
            if let Some(c) = self.components.get(id) { *freq.entry(c.name.as_str()).or_insert(0) += 1; }
        }
        let dupes = freq.values().filter(|&&n| n > 1).count();
        Some(FolderOrganisationModel::compute(&FolderOrganisationInput {
            max_depth, depth_threshold, direct_item_count: direct_items,
            orphan_count: orphans, duplicate_count: dupes,
        }))
    }

    fn folder_max_depth(&self, folder_id: &str, current: usize) -> usize {
        let folder = match self.components.get(folder_id) { Some(f) => f, None => return current };
        let subs: Vec<&str> = folder.unordered_members.iter()
            .filter(|id| self.components.get(id.as_str())
                .map(|c| c.component_type == PortfolioComponentType::Folder).unwrap_or(false))
            .map(String::as_str).collect();
        if subs.is_empty() { current }
        else { subs.iter().map(|id| self.folder_max_depth(id, current + 1)).max().unwrap_or(current) }
    }
}

// =============================================================================
// §24 — BUILDER HELPERS
// =============================================================================

/// Fluent builder for quickly constructing common component configurations.
pub struct ComponentBuilder {
    actor_id: ActorId,
    name: String,
    description: String,
    component_type: PortfolioComponentType,
    book_type: Option<BookType>,
    status: ComponentStatus,
    visibility: Visibility,
    tags: Vec<String>,
    hashtags: Vec<String>,
    payload: Option<ComponentPayload>,
    properties: Properties,
}

impl ComponentBuilder {
    pub fn portfolio(actor_id: impl Into<ActorId>, name: impl Into<String>) -> Self {
        Self::new(actor_id, name, PortfolioComponentType::Portfolio)
    }
    pub fn project(actor_id: impl Into<ActorId>, name: impl Into<String>) -> Self {
        Self::new(actor_id, name, PortfolioComponentType::Project)
    }
    pub fn program(actor_id: impl Into<ActorId>, name: impl Into<String>) -> Self {
        Self::new(actor_id, name, PortfolioComponentType::Program)
    }
    pub fn resource(actor_id: impl Into<ActorId>, name: impl Into<String>) -> Self {
        Self::new(actor_id, name, PortfolioComponentType::Resource)
    }
    pub fn artifact(actor_id: impl Into<ActorId>, name: impl Into<String>) -> Self {
        Self::new(actor_id, name, PortfolioComponentType::Artifact)
    }
    pub fn asset(actor_id: impl Into<ActorId>, name: impl Into<String>) -> Self {
        Self::new(actor_id, name, PortfolioComponentType::Asset)
    }
    pub fn binder(actor_id: impl Into<ActorId>, name: impl Into<String>) -> Self {
        Self::new(actor_id, name, PortfolioComponentType::Binder)
    }
    pub fn book(actor_id: impl Into<ActorId>, name: impl Into<String>, book_type: BookType) -> Self {
        let mut b = Self::new(actor_id, name, PortfolioComponentType::Book);
        b.book_type = Some(book_type);
        b
    }
    pub fn folder(actor_id: impl Into<ActorId>, name: impl Into<String>) -> Self {
        Self::new(actor_id, name, PortfolioComponentType::Folder)
    }
    pub fn registry(actor_id: impl Into<ActorId>, name: impl Into<String>) -> Self {
        Self::new(actor_id, name, PortfolioComponentType::Registry)
    }
    pub fn archive(actor_id: impl Into<ActorId>, name: impl Into<String>) -> Self {
        Self::new(actor_id, name, PortfolioComponentType::Archive)
    }

    fn new(actor_id: impl Into<ActorId>, name: impl Into<String>, ct: PortfolioComponentType) -> Self {
        Self {
            actor_id: actor_id.into(), name: name.into(), description: String::new(),
            component_type: ct, book_type: None,
            status: ComponentStatus::Draft, visibility: Visibility::Private,
            tags: Vec::new(), hashtags: Vec::new(), payload: None, properties: HashMap::new(),
        }
    }

    pub fn description(mut self, d: impl Into<String>)  -> Self { self.description = d.into(); self }
    pub fn status(mut self, s: ComponentStatus)          -> Self { self.status = s; self }
    pub fn visibility(mut self, v: Visibility)           -> Self { self.visibility = v; self }
    pub fn tag(mut self, t: impl Into<String>)           -> Self { self.tags.push(t.into()); self }
    pub fn hashtag(mut self, h: impl Into<String>)       -> Self { self.hashtags.push(h.into()); self }
    pub fn payload(mut self, p: ComponentPayload)        -> Self { self.payload = Some(p); self }
    pub fn property(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.properties.insert(k.into(), v.into()); self
    }

    /// Build and register the component in `system`.
    pub fn build(self, system: &mut PortfolioSystem) -> Result<String, String> {
        let mut comp = system.create_component(self.component_type, self.name)?;
        let id = comp.metadata.id.clone();
        if let Some(c) = system.components.get_mut(&id) {
            c.description = self.description;
            c.status      = self.status;
            c.visibility  = self.visibility;
            c.metadata.tags = self.tags;
            c.hashtags = self.hashtags.into_iter().collect();
            c.metadata.properties.extend(self.properties);
            c.payload     = self.payload;
            if let Some(bt) = self.book_type { c.book_type = Some(bt); }
        }
        Ok(id)
    }
}

// =============================================================================
// §25 — TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sys() -> PortfolioSystem { PortfolioSystem::new("actor-1") }

    // ── Core CRUD ─────────────────────────────────────────────────────────────

    #[test]
    fn create_and_get_component() {
        let mut s = sys();
        let c = s.create_component(PortfolioComponentType::Portfolio, "My Portfolio").unwrap();
        assert_eq!(s.get(&c.metadata.id).unwrap().name, "My Portfolio");
    }

    #[test]
    fn remove_component_cleans_edges() {
        let mut s = sys();
        let a = s.create_component(PortfolioComponentType::Project, "A").unwrap().metadata.id;
        let b = s.create_component(PortfolioComponentType::Project, "B").unwrap().metadata.id;
        s.add_dependency(&a, &b).unwrap();
        assert_eq!(s.edges.len(), 1);
        s.remove_component(&b).unwrap();
        assert_eq!(s.edges.len(), 0);
        assert!(!s.components.get(&a).unwrap().dependencies.contains(&b));
    }

    // ── Graph ─────────────────────────────────────────────────────────────────

    #[test]
    fn hierarchy_cycle_prevented() {
        let mut s = sys();
        let a = s.create_component(PortfolioComponentType::Program, "A").unwrap().metadata.id;
        let b = s.create_component(PortfolioComponentType::Project, "B").unwrap().metadata.id;
        s.add_hierarchy(&a, &b).unwrap();
        assert!(s.add_hierarchy(&b, &a).is_err());
    }

    #[test]
    fn dependency_cycle_prevented() {
        let mut s = sys();
        let a = s.create_component(PortfolioComponentType::Project, "A").unwrap().metadata.id;
        let b = s.create_component(PortfolioComponentType::Project, "B").unwrap().metadata.id;
        s.add_dependency(&a, &b).unwrap();
        assert!(s.add_dependency(&b, &a).is_err());
    }

    #[test]
    fn transitive_dependencies() {
        let mut s = sys();
        let a = s.create_component(PortfolioComponentType::Project, "A").unwrap().metadata.id;
        let b = s.create_component(PortfolioComponentType::Project, "B").unwrap().metadata.id;
        let c = s.create_component(PortfolioComponentType::Project, "C").unwrap().metadata.id;
        s.add_dependency(&a, &b).unwrap();
        s.add_dependency(&b, &c).unwrap();
        let deps = s.transitive_dependencies(&a);
        assert!(deps.contains(&b));
        assert!(deps.contains(&c));
    }

    // ── Container membership ──────────────────────────────────────────────────

    #[test]
    fn binder_membership() {
        let mut s = sys();
        let binder = s.create_component(PortfolioComponentType::Binder, "Cluster").unwrap().metadata.id;
        let item   = s.create_component(PortfolioComponentType::Artifact, "Doc").unwrap().metadata.id;
        s.add_member(&binder, &item).unwrap();
        let b = s.get(&binder).unwrap();
        assert!(b.unordered_members.contains(&item));
        s.remove_member(&binder, &item).unwrap();
        assert!(!s.get(&binder).unwrap().unordered_members.contains(&item));
    }

    // ── Social actions & permissions ──────────────────────────────────────────

    #[test]
    fn action_permission_enforcement() {
        let mut s = sys();
        let comp_id = s.create_component(PortfolioComponentType::Asset, "Secret").unwrap().metadata.id;
        // actor-1 (owner) can edit
        assert!(s.act("actor-1", &comp_id, ActionKind::Edit).is_ok());
        // actor-2 (no permission) cannot edit
        assert!(s.act("actor-2", &comp_id, ActionKind::Edit).is_err());
        // actor-2 can read (Viewer-level)
        assert!(s.act("actor-2", &comp_id, ActionKind::Read).is_ok());
    }

    #[test]
    fn grant_and_revoke_permission() {
        let mut s = sys();
        let comp_id = s.create_component(PortfolioComponentType::Project, "P").unwrap().metadata.id;
        s.grant("actor-1", "actor-2", &comp_id, PermissionTier::Editor).unwrap();
        assert!(s.act("actor-2", &comp_id, ActionKind::Edit).is_ok());
        s.revoke("actor-1", "actor-2", &comp_id).unwrap();
        assert!(s.act("actor-2", &comp_id, ActionKind::Edit).is_err());
    }

    // ── Analytics ────────────────────────────────────────────────────────────

    #[test]
    fn analytics_engagement_tracking() {
        let mut s = sys();
        let id = s.create_component(PortfolioComponentType::Portfolio, "P").unwrap().metadata.id;
        s.grant("actor-1", "actor-2", &id, PermissionTier::Subscriber).unwrap();
        s.act("actor-2", &id, ActionKind::Like).unwrap();
        s.act("actor-2", &id, ActionKind::Follow).unwrap();
        s.record_view(&id, 90).unwrap();
        s.record_click(&id).unwrap();
        let a = s.analytics(&id).unwrap();
        assert_eq!(a.likes, 1);
        assert_eq!(a.followers, 1);
        assert_eq!(a.view_time_seconds, 90);
        assert_eq!(a.clicks, 1);
    }

    // ── Snapshot & time-travel ────────────────────────────────────────────────

    #[test]
    fn snapshot_and_restore() {
        let mut s = sys();
        let id = s.create_component(PortfolioComponentType::Project, "Original").unwrap().metadata.id;
        let snap = s.save_snapshot(Some("v1".into()));

        s.edit_component(&id, Some("Modified".into()), None, None, None, None, None).unwrap();
        assert_eq!(s.get(&id).unwrap().name, "Modified");

        s.restore_snapshot(&snap.snapshot_id).unwrap();
        assert_eq!(s.get(&id).unwrap().name, "Original");
    }

    #[test]
    fn checkpoint_round_trip() {
        let mut s = sys();
        s.create_component(PortfolioComponentType::Program, "V1 Program").unwrap();
        let ckpt = s.save_checkpoint("release-1", Some("First stable release".into()));
        s.create_component(PortfolioComponentType::Program, "V2 Program").unwrap();
        assert_eq!(s.components.len(), 2);
        s.restore_checkpoint(&ckpt.label).unwrap();
        assert_eq!(s.components.len(), 1);
    }

    // ── Governance & resources ────────────────────────────────────────────────

    #[test]
    fn resource_allocation_and_overrun() {
        let mut s = sys();
        let id = s.create_component(PortfolioComponentType::Project, "Project X").unwrap().metadata.id;
        s.allocate_budget(&id, 10_000.0, "USD", None).unwrap();
        s.record_spend(&id, 8_000.0).unwrap();
        let a = s.get_resource_allocation(&id).unwrap();
        assert!(!a.is_overrun());
        s.record_spend(&id, 3_000.0).unwrap();
        assert!(s.get_resource_allocation(&id).unwrap().is_overrun());
        assert_eq!(s.overrun_allocations().len(), 1);
    }

    // ── CRDT ─────────────────────────────────────────────────────────────────

    #[test]
    fn vector_clock_causality() {
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
    fn crdt_merge_propagates_name_change() {
        let mut s1 = PortfolioSystem::new("s1");
        let mut s2 = PortfolioSystem::new("s2");
        let id = s1.create_component(PortfolioComponentType::Project, "Alpha").unwrap().metadata.id;
        let remote_log = s1.crdt.clone();
        s2.apply_crdt_log(&remote_log);
        // s2 doesn't have the component itself (no full state sync in this test),
        // but the CRDT log has been merged successfully.
        assert_eq!(s2.crdt.operations.len(), s1.crdt.operations.len());
        let _ = id; // suppress unused warning
    }

    // ── PQL Query ────────────────────────────────────────────────────────────

    #[test]
    fn pql_query() {
        let mut s = sys();
        let _a = s.create_component(PortfolioComponentType::Project, "Alpha Rust").unwrap();
        s.create_component(PortfolioComponentType::Program, "Beta Program").unwrap();

        let results = s.query_pql("type=Project name=alpha");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Alpha Rust");
    }

    // ── Scored search ─────────────────────────────────────────────────────────

    #[test]
    fn search_with_text_and_visibility() {
        let mut s = sys();
        let id = s.create_component(PortfolioComponentType::Portfolio, "AI Research Hub").unwrap().metadata.id;
        s.tag_component("actor-1", &id, "ai").unwrap();
        s.publish("actor-1", &id).unwrap();

        let results = s.search(&SearchQuery {
            text: Some("AI".into()),
            visibility: Some(Visibility::Public),
            ..Default::default()
        });
        assert!(!results.is_empty());
        assert_eq!(results[0].component_id, id);
    }

    // ── Structural primitives ─────────────────────────────────────────────────

    #[test]
    fn structural_primitives() {
        let mut s = sys();
        let item = s.create_component(PortfolioComponentType::Artifact, "Doc").unwrap().metadata.id;

        // Group
        let gid = s.create_group("Cluster", "actor-1");
        s.group_mut(&gid).unwrap().add(item.clone());
        assert_eq!(s.group(&gid).unwrap().members.len(), 1);

        // Collection
        let cid = s.create_collection("Favourites", "actor-1");
        s.collection_mut(&cid).unwrap().insert(item.clone());
        assert!(s.collection(&cid).unwrap().contains(&item));

        // List
        let lid = s.create_list("Backlog", "actor-1");
        s.list_mut(&lid).unwrap().push(item.clone());
        assert_eq!(s.list(&lid).unwrap().items.len(), 1);

        // Schedule
        let sid = s.create_schedule("Q3 Roadmap", "actor-1");
        s.schedule_mut(&sid).unwrap().add_entry(item.clone(), now() + 1000);
        assert_eq!(s.schedule(&sid).unwrap().entries.len(), 1);

        // Directory
        let did = s.create_directory("Root", "/", "actor-1");
        s.directory_mut(&did).unwrap().add_entry(item.clone(), "doc", "artifact");
        assert!(s.directory(&did).unwrap().find(&item).is_some());
    }

    // ── Builder ───────────────────────────────────────────────────────────────

    #[test]
    fn component_builder() {
        let mut s = sys();
        let id = ComponentBuilder::project("actor-1", "My SaaS")
            .description("Cloud-native portfolio tool.")
            .tag("saas")
            .tag("rust")
            .visibility(Visibility::Public)
            .status(ComponentStatus::Active)
            .property("team", "engineering")
            .build(&mut s)
            .unwrap();

        let c = s.get(&id).unwrap();
        assert_eq!(c.name, "My SaaS");
        assert_eq!(c.visibility, Visibility::Public);
        assert!(c.metadata.tags.contains(&"rust".to_string()));
        assert_eq!(c.metadata.properties.get("team").unwrap(), "engineering");
    }

    // ── Computational models ──────────────────────────────────────────────────

    #[test]
    fn portfolio_health_model() {
        let mut s = sys();
        let pid = s.create_component(PortfolioComponentType::Portfolio, "P").unwrap().metadata.id;
        let c1  = s.create_component(PortfolioComponentType::Project, "C1").unwrap().metadata.id;
        let c2  = s.create_component(PortfolioComponentType::Project, "C2").unwrap().metadata.id;
        s.add_hierarchy(&pid, &c1).unwrap();
        s.add_hierarchy(&pid, &c2).unwrap();
        s.allocate_budget(&pid, 50_000.0, "USD", None).unwrap();
        let result = s.compute_portfolio_health(&pid).unwrap();
        assert!(result.health_score > 0.0);
        assert_eq!(result.active_ratio, 1.0);
    }

    #[test]
    fn project_metrics_model() {
        let result = ProjectMetricsModel::compute(&ProjectMetricsInput {
            planned_value: 100.0, earned_value: 80.0, actual_cost: 90.0,
            budget_at_completion: 150.0, schedule_risk_weight: 0.5, open_risk_count: 3,
        });
        assert!(result.cpi < 1.0); // over budget
        assert!(result.spi < 1.0); // behind schedule
        assert!(result.risk_score > 0.0);
    }

    #[test]
    fn os_bridge_add_item() {
        let mut s = PortfolioSystem::mvp();
        let c = s.add_item(NewPortfolioItem {
            item_type: "project".into(),
            name: "Legacy Project".into(),
            status: "active".into(),
        }).unwrap();
        assert_eq!(c.component_type, PortfolioComponentType::Project);
        assert_eq!(c.status, ComponentStatus::Active);
    }

    #[test]
    fn version_bump() {
        let mut s = sys();
        let id = s.create_component(PortfolioComponentType::Artifact, "Doc").unwrap().metadata.id;
        let v1 = s.bump_version("actor-1", &id, VersionPart::Minor, "Add section").unwrap();
        assert_eq!(v1, "0.2.0");
        let v2 = s.bump_version("actor-1", &id, VersionPart::Patch, "Fix typo").unwrap();
        assert_eq!(v2, "0.2.1");
        let v3 = s.bump_version("actor-1", &id, VersionPart::Major, "Breaking change").unwrap();
        assert_eq!(v3, "1.0.0");
    }
}
