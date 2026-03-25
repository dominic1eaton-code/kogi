//! Component — the universal node: Item | Container.
//!
//! Every entity in the Kogi ecosystem is a Component. Items are leaf work
//! entities; Containers are organising structures.

use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::metadata::{BumpKind, ComponentMetadata};
use crate::types::*;

// ── Component Analytics ───────────────────────────────────────────────────────

/// In-process engagement counters updated by `apply_action()`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComponentAnalytics {
    pub views: u64,
    pub likes: u64,
    pub comments: u64,
    pub shares: u64,
    pub followers: u64,
    pub saves: u64,
    pub click_through_rate: f64,
    pub engagement_rate: f64,
    pub spread: u64,        // active hashtag / tag count across platform
}

impl ComponentAnalytics {
    pub fn apply_action(&mut self, action: &ActionKind) {
        match action {
            ActionKind::Like          => self.likes += 1,
            ActionKind::Comment       => self.comments += 1,
            ActionKind::Share         => self.shares += 1,
            ActionKind::Follow        => self.followers += 1,
            ActionKind::Unfollow      => self.followers = self.followers.saturating_sub(1),
            ActionKind::Save
            | ActionKind::Bookmark    => self.saves += 1,
            ActionKind::Hashtag
            | ActionKind::Tag
            | ActionKind::Label       => self.spread += 1,
            _                         => {}
        }
        // Recompute derived metrics
        let total_engagements = self.likes + self.comments + self.shares + self.saves;
        if self.views > 0 {
            self.engagement_rate = total_engagements as f64 / self.views as f64;
        }
    }
}

// ── ComponentUsers ────────────────────────────────────────────────────────────

/// Tiered user registry for a Component.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComponentUsers {
    /// Tiered access map: UserId → PermissionTier.
    pub tiers: HashMap<UserId, PermissionTier>,
    /// Watchers (opt-in notification list).
    pub watchers: HashSet<UserId>,
    pub subscribers: HashSet<UserId>,
    pub followers: HashSet<UserId>,
    pub investors: HashSet<UserId>,
    pub donors: HashSet<UserId>,
}

impl ComponentUsers {
    pub fn new() -> Self { Self::default() }

    pub fn add_user(&mut self, user_id: UserId, tier: PermissionTier) {
        self.tiers.insert(user_id, tier);
    }

    pub fn remove_user(&mut self, user_id: &UserId) {
        self.tiers.remove(user_id);
        self.watchers.remove(user_id);
        self.subscribers.remove(user_id);
        self.followers.remove(user_id);
    }

    pub fn tier_of(&self, user_id: &UserId) -> PermissionTier {
        self.tiers.get(user_id).copied().unwrap_or(PermissionTier::Viewer)
    }

    pub fn has_permission(&self, user_id: &UserId, required: PermissionTier) -> bool {
        self.tier_of(user_id) >= required
    }
}

// ── Shared ComponentData ──────────────────────────────────────────────────────

/// The shared mutable body of every Component node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentData {
    pub category: ComponentCategory,
    pub name: String,
    pub description: String,

    pub status: ComponentStatus,
    pub state: ComponentState,
    pub visibility: Visibility,

    /// DAG parent IDs.
    pub parents: Vec<ComponentId>,
    /// DAG child IDs.
    pub children: Vec<ComponentId>,

    pub links: Vec<ComponentId>,
    pub dependents: Vec<ComponentId>,
    pub dependencies: Vec<ComponentId>,

    pub users: ComponentUsers,
    pub analytics: ComponentAnalytics,

    pub risks: Vec<Risk>,
    pub hashtags: HashSet<String>,
    pub topics: HashSet<String>,

    /// TMS ToolBox references (v2.1).
    pub toolbox_ids: Vec<ToolBoxId>,

    /// Per-plugin configuration store.
    pub plugin_configs: HashMap<String, JsonValue>,
}

impl ComponentData {
    pub fn new(category: ComponentCategory, name: impl Into<String>) -> Self {
        Self {
            category,
            name: name.into(),
            description: String::new(),
            status: ComponentStatus::Draft,
            state: ComponentState::Initializing,
            visibility: Visibility::Private,
            parents: vec![],
            children: vec![],
            links: vec![],
            dependents: vec![],
            dependencies: vec![],
            users: ComponentUsers::new(),
            analytics: ComponentAnalytics::default(),
            risks: vec![],
            hashtags: HashSet::new(),
            topics: HashSet::new(),
            toolbox_ids: vec![],
            plugin_configs: HashMap::new(),
        }
    }
}

// ── Item Payloads ─────────────────────────────────────────────────────────────

/// Metric KPI used in Portfolio and Program payloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub target: f64,
    pub actual: f64,
    pub unit: String,
}

/// Sprint within a Project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sprint {
    pub sprint_id: Uuid,
    pub name: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub velocity: f64,
    pub story_points_committed: u32,
    pub story_points_completed: u32,
}

/// A backlog item (user story, task, bug).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacklogItem {
    pub item_id: Uuid,
    pub title: String,
    pub story_points: u32,
    pub status: ComponentStatus,
    pub priority: u8,
    pub assigned_to: Option<UserId>,
    pub sprint_id: Option<Uuid>,
}

/// A file reference within an Artifact or Folder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRef {
    pub file_id: Uuid,
    pub name: String,
    pub url: String,
    pub media_type: String,
    pub size_bytes: u64,
    pub uploaded_at: DateTime<Utc>,
    pub uploaded_by: UserId,
}

/// A vesting schedule for an Asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VestingSchedule {
    pub cliff_months: u32,
    pub total_months: u32,
    pub vested_pct: f64,
}

// ── ItemPayload enum ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioItemPayload {
    pub mission: Option<String>,
    pub focus_areas: Vec<String>,
    pub kpis: Vec<Metric>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramPayload {
    pub objective: String,
    pub projects: Vec<ComponentId>,
    pub budget: f64,
    pub kpis: Vec<Metric>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectPayload {
    pub project_type: ProjectType,
    pub methodology: Methodology,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub sprints: Vec<Sprint>,
    pub backlog: Vec<BacklogItem>,
    pub releases: Vec<String>,
    pub completion_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePayload {
    pub resource_type: ResourceType,
    pub skills: Vec<String>,
    /// Availability 0.0–1.0 (fraction of full-time).
    pub availability: f64,
    pub hourly_rate: Option<f64>,
    pub currency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactPayload {
    pub artifact_type: ArtifactType,
    pub file_refs: Vec<FileRef>,
    pub produced_by: Option<UserId>,
    pub version_tag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPayload {
    pub asset_type: AssetType,
    pub valuation: f64,
    pub currency: String,
    pub acquired_at: DateTime<Utc>,
    pub depreciation_rate_annual: f64,  // 0.0–1.0
    pub cost_basis: f64,
    pub vesting_schedule: Option<VestingSchedule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubPortfolioPayload {
    pub parent_portfolio_id: ComponentId,
    pub scope: String,
    pub strategic_goals: Vec<String>,
}

/// The discriminated payload for any Item component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemPayload {
    Portfolio(PortfolioItemPayload),
    Program(ProgramPayload),
    Project(ProjectPayload),
    Resource(ResourcePayload),
    Artifact(ArtifactPayload),
    Asset(AssetPayload),
    SubPortfolio(SubPortfolioPayload),
    Custom(JsonValue),
}

// ── Container Payloads ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinderPayload {
    pub held_items: Vec<ComponentId>,
    pub dashboards: Vec<String>,    // dashboard widget config JSON keys
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderPayload {
    pub file_refs: Vec<FileRef>,
    pub max_depth: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryPayload {
    pub entries: HashMap<String, JsonValue>,
    pub schema: Option<JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordPayload {
    pub entries: HashMap<String, JsonValue>,
    pub schema: Option<JsonValue>,
    pub linked_component: Option<ComponentId>,
    pub integrity_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivePayload {
    pub archived_components: Vec<ComponentId>,
    pub encrypted: bool,
    pub compression: Option<String>,
    pub retention_policy: Option<String>,
}

/// The discriminated payload for any Container component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContainerPayload {
    Binder(BinderPayload),
    Book(crate::itembook::BookPayload),
    Record(RecordPayload),
    Folder(FolderPayload),
    Registry(RegistryPayload),
    Archive(ArchivePayload),
}

// ── Activity Log ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEntry {
    pub entry_id: Uuid,
    pub action: ActionKind,
    pub actor: UserId,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<JsonValue>,
}

// ── Component (the universal node) ───────────────────────────────────────────

/// The universal node in the Portfolio System.
/// Every entity — leaf items and organising containers — is a Component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub metadata: ComponentMetadata,
    pub data: ComponentData,
    pub payload: ComponentPayload,
    /// Rich dossier (ItemBook), present for Item components that opt in.
    pub item_book: Option<crate::itembook::ItemBookData>,
    /// Activity log for this component.
    pub activity_log: Vec<ActivityEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentPayload {
    Item(ItemPayload),
    Container(ContainerPayload),
}

impl Component {
    // ── Constructors ──────────────────────────────────────────────────────

    pub fn new_item(
        owner: UserId,
        name: impl Into<String>,
        item_category: ItemCategory,
        payload: ItemPayload,
        node_id: &str,
    ) -> Self {
        let id = Uuid::new_v4();
        let mut data = ComponentData::new(ComponentCategory::Item(item_category), name);
        data.users.add_user(owner, PermissionTier::Owner);
        Self {
            metadata: ComponentMetadata::new(id, owner, node_id),
            data,
            payload: ComponentPayload::Item(payload),
            item_book: None,
            activity_log: vec![],
        }
    }

    pub fn new_container(
        owner: UserId,
        name: impl Into<String>,
        container_category: ContainerCategory,
        payload: ContainerPayload,
        node_id: &str,
    ) -> Self {
        let id = Uuid::new_v4();
        let mut data = ComponentData::new(ComponentCategory::Container(container_category), name);
        data.users.add_user(owner, PermissionTier::Owner);
        Self {
            metadata: ComponentMetadata::new(id, owner, node_id),
            data,
            payload: ComponentPayload::Container(payload),
            item_book: None,
            activity_log: vec![],
        }
    }

    // ── Identity ──────────────────────────────────────────────────────────

    pub fn id(&self) -> ComponentId { self.metadata.id }
    pub fn name(&self) -> &str { &self.data.name }
    pub fn status(&self) -> &ComponentStatus { &self.data.status }
    pub fn state(&self) -> &ComponentState { &self.data.state }
    pub fn category(&self) -> &ComponentCategory { &self.data.category }

    pub fn is_item(&self) -> bool { matches!(self.payload, ComponentPayload::Item(_)) }
    pub fn is_container(&self) -> bool { matches!(self.payload, ComponentPayload::Container(_)) }

    // ── Permission Check ──────────────────────────────────────────────────

    pub fn check_permission(&self, user_id: &UserId, required: PermissionTier) -> bool {
        self.data.users.has_permission(user_id, required)
    }

    // ── Mutation ──────────────────────────────────────────────────────────

    pub fn update_info(&mut self, name: Option<String>, description: Option<String>, node_id: &str) {
        if let Some(n) = name { self.data.name = n; }
        if let Some(d) = description { self.data.description = d; }
        self.metadata.touch(node_id);
    }

    pub fn set_status(&mut self, status: ComponentStatus, node_id: &str) {
        // Side effects by status
        match &status {
            ComponentStatus::Archived => { self.data.state = ComponentState::Sealed; }
            ComponentStatus::Active if matches!(self.data.status, ComponentStatus::Archived) => {
                self.data.state = ComponentState::Configured;
            }
            _ => {}
        }
        self.data.status = status;
        self.metadata.touch(node_id);
    }

    pub fn set_state(&mut self, state: ComponentState, node_id: &str) {
        self.data.state = state;
        self.metadata.touch(node_id);
    }

    pub fn set_visibility(&mut self, visibility: Visibility, node_id: &str) {
        self.data.visibility = visibility;
        self.metadata.touch(node_id);
    }

    /// Apply an ActionKind, updating analytics and logging the activity.
    pub fn apply_action(&mut self, action: ActionKind, actor: UserId) {
        self.data.analytics.apply_action(&action);
        self.activity_log.push(ActivityEntry {
            entry_id: Uuid::new_v4(),
            action,
            actor,
            timestamp: Utc::now(),
            metadata: None,
        });
        // views updated separately — this is for engagement actions
        self.data.analytics.views += 1;
    }

    pub fn bump_version(&mut self, kind: BumpKind, node_id: &str, summary: Option<String>) {
        self.metadata.bump_version(kind, node_id, summary);
        if let Some(ref mut book) = self.item_book {
            book.on_version_bump(&self.metadata.version);
        }
    }

    // ── ToolBox integration (TMS v2.1) ────────────────────────────────────

    pub fn attach_toolbox(&mut self, toolbox_id: ToolBoxId) {
        if !self.data.toolbox_ids.contains(&toolbox_id) {
            self.data.toolbox_ids.push(toolbox_id);
        }
    }

    pub fn detach_toolbox(&mut self, toolbox_id: ToolBoxId) {
        self.data.toolbox_ids.retain(|&id| id != toolbox_id);
    }

    pub fn has_toolbox(&self, toolbox_id: ToolBoxId) -> bool {
        self.data.toolbox_ids.contains(&toolbox_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_project_component() {
        let owner = Uuid::new_v4();
        let comp = Component::new_item(
            owner,
            "Website Redesign",
            ItemCategory::Project,
            ItemPayload::Project(ProjectPayload {
                project_type: ProjectType::Creative,
                methodology: Methodology::Agile,
                start_date: None,
                end_date: None,
                sprints: vec![],
                backlog: vec![],
                releases: vec![],
                completion_pct: 0.0,
            }),
            "node-1",
        );
        assert_eq!(comp.name(), "Website Redesign");
        assert!(comp.check_permission(&owner, PermissionTier::Owner));
        assert!(!comp.check_permission(&Uuid::new_v4(), PermissionTier::Editor));
    }

    #[test]
    fn toolbox_attach_detach() {
        let owner = Uuid::new_v4();
        let mut comp = Component::new_item(
            owner, "My Portfolio",
            ItemCategory::Portfolio,
            ItemPayload::Portfolio(PortfolioItemPayload {
                mission: None, focus_areas: vec![], kpis: vec![],
            }),
            "node-1",
        );
        let tb = Uuid::new_v4();
        comp.attach_toolbox(tb);
        assert!(comp.has_toolbox(tb));
        comp.detach_toolbox(tb);
        assert!(!comp.has_toolbox(tb));
    }
}
