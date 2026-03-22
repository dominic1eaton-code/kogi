//! HG-SPACE — Spaces, Workspaces, and Namespace (HG-NS).

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::cell::{AttributeMap, DimKey, AxisId, GridId, CubeId, IdentityId, PermissionTier};
use crate::dim::DimSlicePredicate;

pub type SpaceId   = Uuid;
pub type WorkspaceId = Uuid;
pub type FedLinkId = Uuid;
pub type NodeId    = Uuid;   // graph node for this space

// ─── Space Type ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SpaceType {
    Personal,
    Team,
    Organization,
    Community,
    Federation,
    Project,
    Research,
    Event,
    Product,
    Studio,
    Tenant,
    Custom(String),
}

// ─── Visibility ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Visibility {
    Private, Tenant, Internal, Public, Unlisted,
}

// ─── Space Status ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SpaceStatus {
    Draft, Active, Paused, Restricted, Dissolving, Dissolved, Archived,
}

// ─── Governance ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceConfig {
    pub vote_threshold: f64,          // 0.0–1.0 (e.g. 0.51 = simple majority)
    pub quorum: f64,                  // minimum participation fraction
    pub proposal_types: Vec<String>,
    pub multisig_required: bool,
    pub multisig_signers: Vec<IdentityId>,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            vote_threshold: 0.51,
            quorum: 0.25,
            proposal_types: vec![],
            multisig_required: false,
            multisig_signers: vec![],
        }
    }
}

// ─── Space Member ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceMember {
    pub identity_id: IdentityId,
    pub role_label: String,
    pub permission_tier: PermissionTier,
    pub joined_at: DateTime<Utc>,
}

// ─── Space ────────────────────────────────────────────────────────────────────

/// A named, governed, bounded operational context within a Grid.
/// Groups related Hypercubes, member identities, Workspaces, and governance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Space {
    pub space_id: SpaceId,
    pub space_type: SpaceType,
    pub slug: String,
    pub namespace_path: String,     // hypergrid://{grid}/{type}/{slug}/
    pub grid_id: GridId,
    pub name: String,
    pub description: String,

    pub member_roster: Vec<SpaceMember>,
    pub cubes: Vec<CubeId>,
    pub shared_cubes: Vec<CubeId>,
    pub workspaces: Vec<WorkspaceId>,

    pub governance_config: GovernanceConfig,
    pub treasury_ref: Option<Uuid>,

    pub link_node_id: NodeId,

    pub visibility: Visibility,
    pub status: SpaceStatus,
    pub federation_links: Vec<FedLinkId>,
    pub plugin_config: HashMap<String, JsonValue>,
    pub metadata: AttributeMap,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Space {
    pub fn new(grid_id: GridId, space_type: SpaceType, slug: impl Into<String>, name: impl Into<String>) -> Self {
        let slug_str: String = slug.into();
        let ns_type = match &space_type {
            SpaceType::Team => "team",
            SpaceType::Organization => "org",
            SpaceType::Community => "community",
            SpaceType::Personal => "personal",
            _ => "space",
        };
        let namespace_path = format!("hypergrid://{grid_id}/{ns_type}/{slug_str}/");
        let now = Utc::now();
        Self {
            space_id: Uuid::new_v4(),
            space_type,
            slug: slug_str,
            namespace_path,
            grid_id,
            name: name.into(),
            description: String::new(),
            member_roster: vec![],
            cubes: vec![],
            shared_cubes: vec![],
            workspaces: vec![],
            governance_config: GovernanceConfig::default(),
            treasury_ref: None,
            link_node_id: Uuid::new_v4(),
            visibility: Visibility::Private,
            status: SpaceStatus::Active,
            federation_links: vec![],
            plugin_config: HashMap::new(),
            metadata: AttributeMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_member(&mut self, identity_id: IdentityId, role: impl Into<String>, tier: PermissionTier) {
        self.member_roster.push(SpaceMember {
            identity_id,
            role_label: role.into(),
            permission_tier: tier,
            joined_at: Utc::now(),
        });
    }

    pub fn member_tier(&self, identity_id: IdentityId) -> Option<PermissionTier> {
        self.member_roster.iter().find(|m| m.identity_id == identity_id).map(|m| m.permission_tier)
    }

    pub fn attach_cube(&mut self, cube_id: CubeId) {
        if !self.cubes.contains(&cube_id) {
            self.cubes.push(cube_id);
        }
    }
}

// ─── Open Cube Reference ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCubeRef {
    pub cube_id: CubeId,
    pub view_id: Option<Uuid>,
    pub active_dim_slices: Vec<DimSlicePredicate>,
    pub scroll_state: Option<(u64, u64)>,
    pub active_cell_coord: Option<crate::cell::DimCoordinate>,
}

// ─── Workspace ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkspaceType {
    Personal, Shared, Project, Guest, Template, Ai, Archive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceLayout {
    pub panels: Vec<String>,
    pub split: String,
}

impl Default for WorkspaceLayout {
    fn default() -> Self {
        Self { panels: vec!["main".into()], split: "none".into() }
    }
}

/// Active, personalized working context within a Space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub workspace_id: WorkspaceId,
    pub workspace_type: WorkspaceType,
    pub owner_id: IdentityId,
    pub space_id: SpaceId,
    pub grid_id: GridId,
    pub namespace_path: String,

    // Session state
    pub open_cubes: Vec<OpenCubeRef>,
    pub active_cube_id: Option<CubeId>,
    pub pinned_rows: Vec<(CubeId, DimKey)>,
    pub pinned_columns: Vec<(CubeId, DimKey)>,
    pub recent_cells: Vec<(CubeId, crate::cell::DimCoordinate, DateTime<Utc>)>,

    /// Workspace-level dim context: e.g. D₃ (Time) = "2026-Q2" applies to all open cubes.
    pub dim_context: HashMap<AxisId, DimKey>,

    /// Global filters applied across all open cubes.
    pub global_filters: Vec<DimSlicePredicate>,

    // UI state
    pub layout_config: WorkspaceLayout,
    pub graph_panel_open: bool,

    pub metadata: AttributeMap,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Workspace {
    pub fn new(
        owner_id: IdentityId,
        space_id: SpaceId,
        grid_id: GridId,
        workspace_type: WorkspaceType,
    ) -> Self {
        let now = Utc::now();
        Self {
            workspace_id: Uuid::new_v4(),
            workspace_type,
            owner_id,
            space_id,
            grid_id,
            namespace_path: format!("hypergrid://{grid_id}/workspace/{}/", Uuid::new_v4()),
            open_cubes: vec![],
            active_cube_id: None,
            pinned_rows: vec![],
            pinned_columns: vec![],
            recent_cells: vec![],
            dim_context: HashMap::new(),
            global_filters: vec![],
            layout_config: WorkspaceLayout::default(),
            graph_panel_open: false,
            metadata: AttributeMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Set the workspace-level dimension context (e.g. freeze D₃ = "2026-Q2").
    pub fn set_dim_context(&mut self, axis_id: AxisId, key: DimKey) {
        self.dim_context.insert(axis_id, key);
        self.updated_at = Utc::now();
    }

    pub fn open_cube(&mut self, cube_ref: OpenCubeRef) {
        if !self.open_cubes.iter().any(|c| c.cube_id == cube_ref.cube_id) {
            self.active_cube_id = Some(cube_ref.cube_id);
            self.open_cubes.push(cube_ref);
        }
    }
}
