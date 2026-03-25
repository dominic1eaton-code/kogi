//! Shared primitive types, newtypes, and enums for the Portfolio System.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── ID newtypes ───────────────────────────────────────────────────────────────
pub type ComponentId = Uuid;
pub type UserId      = Uuid;
pub type PolicyId    = Uuid;
pub type ToolBoxId   = Uuid;
pub type WorkflowId  = Uuid;
pub type LedgerId    = Uuid;
pub type AccountId   = Uuid;
pub type ProviderId  = Uuid;
pub type VersionId   = Uuid;
pub type EntityId    = Uuid;   // user | org | collective

/// Version string following semver convention (e.g. "1.2.3").
pub type VersionString = String;

// ── ComponentStatus ───────────────────────────────────────────────────────────

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

impl Default for ComponentStatus {
    fn default() -> Self { Self::Draft }
}

impl std::fmt::Display for ComponentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Custom(s) => write!(f, "{s}"),
            other => write!(f, "{other:?}"),
        }
    }
}

// ── ComponentState ────────────────────────────────────────────────────────────

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

impl Default for ComponentState {
    fn default() -> Self { Self::Initializing }
}

// ── Visibility ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

// ── PermissionTier ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PermissionTier {
    Viewer      = 0,
    Subscriber  = 1,
    Contributor = 2,
    Editor      = 3,
    Manager     = 4,
    Owner       = 5,
    Admin       = 6,
}

// ── Item & Container Categories ───────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemCategory {
    Portfolio,
    Program,
    Project,
    Resource,
    Artifact,
    Asset,
    SubPortfolio,
    /// For crowdresourcing campaigns, benefit accounts, etc.
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
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BookKind {
    Notebook,
    ContactBook,
    PlayBook,
    ScheduleBook,
    PlanBook,
    GuideBook,
    ItemBook,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentCategory {
    Item(ItemCategory),
    Container(ContainerCategory),
}

// ── Project Type ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectType {
    Organizational, Creative, Technical, Research, Ai, Software,
    Media, Marketing, Investment, ContentCreator, Diy, Custom(String),
}

// ── Methodology ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Methodology {
    Agile, Scrum, Kanban, Waterfall, Lean, Custom(String),
}

// ── Asset & Resource Types ────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetType {
    IntellectualProperty, Financial, Physical, Digital, Creative, Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    Human, Machine, Service, License, Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactType {
    Document, Code, Design, Dataset, Notebook, Template, Report,
    Playbook, Contract, Custom(String),
}

// ── Risk Severity ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskSeverity {
    Low = 1, Medium = 2, High = 3, Critical = 4,
}

impl RiskSeverity {
    pub fn score(self) -> f64 {
        self as u8 as f64
    }
}

/// A risk record attached to a Component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub risk_id: Uuid,
    pub title: String,
    pub description: String,
    pub severity: RiskSeverity,
    pub probability: f64,       // 0.0–1.0
    pub mitigation: Option<String>,
    pub owner: Option<UserId>,
    pub identified_at: chrono::DateTime<chrono::Utc>,
}

// ── ActionKind ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionKind {
    // Social
    Like, Comment, Share, Follow, Unfollow, Subscribe, Unsubscribe,
    Bookmark, Save, Watch, Invite, Poll,
    // Content
    Post { visibility: Visibility }, Edit, Create, Delete, Tag, Label, Hashtag, Mention,
    // Economic
    Donate { amount: ordered_float::NotNan<f64> },
    Invest { amount: ordered_float::NotNan<f64>, equity: bool },
    // Access
    Join, Leave, Own { tier: PermissionTier },
    // System
    Archive, Restore, Report, Search, Filter, Index, Notify, Alert,
    // TMS
    AttachToolbox(ToolBoxId), DetachToolbox(ToolBoxId),
}

// ── Contribution Type (shared portfolios) ─────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContributionType {
    Labor, Capital, Asset, Knowledge, Artifact, Code, Design, Data,
}

// ── Governance Status ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GovernanceStatus {
    PendingReview, Accepted, Rejected, Merged,
}

// ── Resource Kind ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceKind {
    Budget, PersonHours, StoryPoints, ComputeUnits, StorageGib, Custom(String),
}

// ── Benefit Type ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BenefitType {
    Health, Dental, Vision, Hsa, Retirement, Pto, Disability,
    ProfessionalDev, EmergencySavings, PortableSavings,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaxTreatment {
    PreTax, PostTax, TaxFree,
}

// ── Access Level (resource sharing) ──────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceAccessLevel {
    Read, Fork, Contribute, Manage,
}

// needed for Donate/Invest f64 in ActionKind hashing
mod ordered_float {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
    pub struct NotNan<T>(ordered_float_inner::OrderedFloat<T>);

    impl NotNan<f64> {
        pub fn new(v: f64) -> Self {
            Self(ordered_float_inner::OrderedFloat(v))
        }
        pub fn get(&self) -> f64 {
            self.0.0
        }
    }

    mod ordered_float_inner {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        pub struct OrderedFloat<T>(pub T);
    }
}
