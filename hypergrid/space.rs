// =============================================================================
// hypergrid::space — HG-SPACE: Space, Workspace, and Governance System
// hypergrid::ns    — HG-NS:    Namespace Registry
// hypergrid::id    — HG-ID:    Multi-Tenant Identity System
// =============================================================================

use std::collections::HashMap;
use std::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::cell::{ActorId, CubeId, GridId, SpaceId, UserId};
use crate::error::HypergridError;

// =============================================================================
// HG-SPACE
// =============================================================================

// ─── SpaceType ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpaceType {
    Personal,       // One-per-identity root; auto-created on registration
    Team,           // Small collaborative group
    Organization,   // Formal org with full governance, treasury, member management
    Cooperative,    // Member-owned, democratic governance
    Collective,     // Looser collective with reputation-based influence
    Community,      // Open topical group
    Federation,     // Meta-Space linking multiple organizations
    Project,        // Time-bounded project Space (auto-archives on completion)
    Research,       // Academic/scientific research group
    Studio,         // Creative production Space
    Factory,        // Solution factory (Qala)
    Event,          // Time-bounded event Space
    Custom(String),
}

// ─── SpaceStatus ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpaceStatus {
    Draft,
    Active,
    Paused,
    Restricted,
    Dissolving,
    Dissolved,
    Archived,
}

// ─── SpaceVisibility ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpaceVisibility {
    Private,    // Only members can see the Space exists
    Protected,  // Space is discoverable but content is member-only
    Internal,   // Visible within the same organization/tenant
    Public,     // Fully public: content visible to all
    Unlisted,   // Accessible by direct link but not listed in search
}

// ─── SpaceMemberRole ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpaceMemberRole {
    Owner,      // Full control including dissolution
    Governor,   // Can submit and vote on governance proposals
    Steward,    // Can manage members and settings
    Treasurer,  // Can manage treasury and financial operations
    Editor,     // Can create and edit content
    Contributor,// Can contribute content
    Member,     // Basic membership; read access
    Viewer,     // Read-only; no contribution rights
}

// ─── SpaceMember ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceMember {
    pub user_id:              UserId,
    pub identity_tag:         String,
    pub role:                 SpaceMemberRole,
    pub joined_at:            DateTime<Utc>,
    pub contribution_weight:  f64,   // 0.0–1.0; used for governance weighting
    pub stake_pct:            f64,   // ownership stake percentage
    pub active_workspace_ids: Vec<Uuid>,
}

// ─── GovernanceConfig ────────────────────────────────────────────────────────

/// Governance model configuration for a Space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceConfig {
    pub vote_threshold_pct:  f64,    // % of yes votes required to pass (e.g., 51.0)
    pub quorum_pct:          f64,    // % of members that must vote for quorum (e.g., 30.0)
    pub proposal_types:      Vec<String>,  // which proposal types require a vote
    pub dissolution_threshold: f64,  // supermajority required to dissolve (e.g., 75.0)
    pub multisig_required:   bool,   // treasury operations require multi-sig
    pub multisig_m:          u32,    // M-of-N multisig threshold
    pub multisig_n:          u32,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            vote_threshold_pct: 51.0,
            quorum_pct: 30.0,
            proposal_types: vec!["policy".into(), "budget".into(), "member".into()],
            dissolution_threshold: 75.0,
            multisig_required: false,
            multisig_m: 2,
            multisig_n: 3,
        }
    }
}

// ─── Space ────────────────────────────────────────────────────────────────────

/// A named, governed, bounded operational environment in Hypergrid.
/// Spaces own Hypercubes, have member rosters, governance, and treasury.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Space {
    pub space_id:        SpaceId,
    pub grid_id:         GridId,
    pub space_type:      SpaceType,
    pub slug:            String,         // URL-safe; unique within namespace
    pub name:            String,
    pub description:     Option<String>,
    pub namespace_path:  String,         // e.g. "kogi://coop/pamoja-capital/"
    pub visibility:      SpaceVisibility,
    pub status:          SpaceStatus,
    pub members:         Vec<SpaceMember>,
    pub cube_ids:        Vec<CubeId>,    // Hypercubes owned by this Space
    pub governance:      GovernanceConfig,
    pub treasury_ref:    Option<Uuid>,   // external treasury account reference
    pub link_node_id:    Option<crate::graph::EdgeId>,  // node in the Hypergraph
    pub parent_space_id: Option<SpaceId>,
    pub template_id:     Option<Uuid>,
    pub federation_links: Vec<Uuid>,
    pub policy_ids:      Vec<Uuid>,
    pub created_at:      DateTime<Utc>,
    pub updated_at:      DateTime<Utc>,
}

impl Space {
    pub fn new(
        grid_id: GridId, space_type: SpaceType,
        slug: impl Into<String>, name: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            space_id:    Uuid::new_v4(),
            grid_id,
            space_type,
            slug:        slug.into(),
            name:        name.into(),
            description: None,
            namespace_path: String::new(),
            visibility:  SpaceVisibility::Private,
            status:      SpaceStatus::Active,
            members:     vec![],
            cube_ids:    vec![],
            governance:  GovernanceConfig::default(),
            treasury_ref: None,
            link_node_id: None,
            parent_space_id: None,
            template_id: None,
            federation_links: vec![],
            policy_ids:  vec![],
            created_at:  now,
            updated_at:  now,
        }
    }

    pub fn add_member(&mut self, member: SpaceMember) {
        self.members.retain(|m| m.user_id != member.user_id);
        self.members.push(member);
        self.updated_at = Utc::now();
    }

    pub fn member_count(&self) -> usize { self.members.len() }

    pub fn get_member(&self, user_id: UserId) -> Option<&SpaceMember> {
        self.members.iter().find(|m| m.user_id == user_id)
    }
}

// ─── SpaceRegistry ───────────────────────────────────────────────────────────

pub struct SpaceRegistry {
    spaces:  RwLock<HashMap<SpaceId, Space>>,
    by_slug: RwLock<HashMap<String, SpaceId>>,
}

impl SpaceRegistry {
    pub fn new() -> Self {
        Self { spaces: Default::default(), by_slug: Default::default() }
    }

    pub fn register(&self, space: Space) {
        let slug = space.slug.clone();
        let id   = space.space_id;
        self.spaces.write().unwrap().insert(id, space);
        self.by_slug.write().unwrap().insert(slug, id);
    }

    pub fn get(&self, id: SpaceId) -> Option<Space> {
        self.spaces.read().unwrap().get(&id).cloned()
    }

    pub fn get_by_slug(&self, slug: &str) -> Option<Space> {
        let id = self.by_slug.read().unwrap().get(slug).copied()?;
        self.get(id)
    }

    pub fn update(&self, space: Space) {
        self.spaces.write().unwrap().insert(space.space_id, space);
    }

    pub fn all_for_grid(&self, grid_id: GridId) -> Vec<Space> {
        self.spaces.read().unwrap().values()
            .filter(|s| s.grid_id == grid_id)
            .cloned()
            .collect()
    }
}

// ─── Workspace ────────────────────────────────────────────────────────────────

/// An active working context for one or more users within a Space.
/// Holds session state: open cubes, cursor positions, dimension context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub workspace_id:   Uuid,
    pub space_id:       SpaceId,
    pub grid_id:        GridId,
    pub name:           String,
    pub owner_id:       UserId,
    pub workspace_type: WorkspaceType,

    /// Active open cubes and their view configuration.
    pub open_cubes:     Vec<OpenCubeRef>,

    /// Global DimContext: dimension key defaults applied across ALL open cubes.
    /// Example: { time_axis_id → "2026-Q2" } freezes all views to Q2 time slice.
    pub dim_context:    HashMap<crate::cell::AxisId, crate::cell::DimKey>,

    /// Global filters applied across all open cubes in this Workspace.
    pub global_filters: Vec<crate::dim::DimSlicePredicate>,

    pub created_at:     DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
}

impl Workspace {
    pub fn new(space_id: SpaceId, grid_id: GridId, owner_id: UserId, name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            workspace_id:   Uuid::new_v4(),
            space_id, grid_id, owner_id,
            name:           name.into(),
            workspace_type: WorkspaceType::Personal,
            open_cubes:     vec![],
            dim_context:    HashMap::new(),
            global_filters: vec![],
            created_at:     now,
            last_active_at: now,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkspaceType { Personal, Shared, Project, Guest, Template, AI }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCubeRef {
    pub cube_id:   CubeId,
    pub view_id:   Option<Uuid>,
    pub active_filters: Vec<crate::dim::DimSlicePredicate>,
}

pub struct WorkspaceStore {
    workspaces: RwLock<HashMap<Uuid, Workspace>>,
}

impl WorkspaceStore {
    pub fn new() -> Self { Self { workspaces: Default::default() } }

    pub fn insert(&self, ws: Workspace) { self.workspaces.write().unwrap().insert(ws.workspace_id, ws); }
    pub fn get(&self, id: Uuid) -> Option<Workspace> { self.workspaces.read().unwrap().get(&id).cloned() }
    pub fn update(&self, ws: Workspace) { self.workspaces.write().unwrap().insert(ws.workspace_id, ws); }
}

// =============================================================================
// HG-NS — Namespace System
// =============================================================================

/// A hierarchical URI addressing scheme for every entity in Hypergrid.
///
/// Format: {domain}://{space_type}/{space_slug}/{entity_type}/{entity_id_or_slug}/
///
/// Examples:
///   kogi://@alice-creative/programs/q2-client-work/
///   ume://org/pamoja-capital/modules/finance-accounting/
///   qala://factory/pharma-factory/solutions/tablet-formula-v3/
///   hypergrid://kogi-production/system/spaces/
pub type NamespacePath = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceEntry {
    pub namespace_id:  Uuid,
    pub path:          NamespacePath,
    pub entity_ref:    crate::cell::EntityRef,
    pub entity_type:   String,         // "Row", "Cube", "Space", "Grid", etc.
    pub slug:          String,
    pub parent_id:     Option<Uuid>,
    pub visibility:    SpaceVisibility,
    pub aliases:       Vec<NamespacePath>,
    pub canonical_url: Option<String>, // e.g. https://kogi.io/...
    pub federation_origin: Option<GridId>, // non-None if federated from a peer Grid
    pub created_at:    DateTime<Utc>,
    pub last_resolved_at: Option<DateTime<Utc>>,
}

impl NamespaceEntry {
    pub fn new(path: NamespacePath, entity_ref: crate::cell::EntityRef, entity_type: impl Into<String>) -> Self {
        Self {
            namespace_id:  Uuid::new_v4(),
            path:          path.clone(),
            entity_ref,
            entity_type:   entity_type.into(),
            slug:          path.split('/').filter(|s| !s.is_empty()).last().unwrap_or("").to_owned(),
            parent_id:     None,
            visibility:    SpaceVisibility::Public,
            aliases:       vec![],
            canonical_url: None,
            federation_origin: None,
            created_at:    Utc::now(),
            last_resolved_at: None,
        }
    }
}

/// The namespace registry: maps NamespacePaths to entity references.
/// Hot path: Redis cache (300s TTL). Cold path: PostgreSQL.
pub struct NamespaceRegistry {
    entries:   RwLock<HashMap<NamespacePath, NamespaceEntry>>,
    by_id:     RwLock<HashMap<Uuid, NamespacePath>>,
    aliases:   RwLock<HashMap<NamespacePath, NamespacePath>>, // alias → canonical
}

impl NamespaceRegistry {
    pub fn new() -> Self {
        Self {
            entries: Default::default(),
            by_id:   Default::default(),
            aliases: Default::default(),
        }
    }

    pub fn register(&self, entry: NamespaceEntry) -> Result<(), HypergridError> {
        let path = entry.path.clone();
        let id   = entry.namespace_id;
        if self.entries.read().unwrap().contains_key(&path) {
            return Err(HypergridError::AlreadyExists(format!("namespace: {path}")));
        }
        self.entries.write().unwrap().insert(path.clone(), entry);
        self.by_id.write().unwrap().insert(id, path);
        Ok(())
    }

    /// Resolve a path to its entity reference (follows aliases).
    pub fn resolve(&self, path: &str) -> Option<crate::cell::EntityRef> {
        // Check alias first
        let canonical = self.aliases.read().unwrap()
            .get(path).cloned()
            .unwrap_or_else(|| path.to_owned());

        let mut entries = self.entries.write().unwrap();
        let entry = entries.get_mut(&canonical)?;
        entry.last_resolved_at = Some(Utc::now());
        Some(entry.entity_ref.clone())
    }

    pub fn get_entry(&self, path: &str) -> Option<NamespaceEntry> {
        let canonical = self.aliases.read().unwrap()
            .get(path).cloned()
            .unwrap_or_else(|| path.to_owned());
        self.entries.read().unwrap().get(&canonical).cloned()
    }

    pub fn add_alias(&self, alias: NamespacePath, canonical: NamespacePath) {
        self.aliases.write().unwrap().insert(alias, canonical);
    }

    pub fn rename(&self, old_path: &str, new_path: NamespacePath) -> Result<(), HypergridError> {
        let entry = {
            let mut entries = self.entries.write().unwrap();
            let mut entry = entries.remove(old_path)
                .ok_or_else(|| HypergridError::NotFound(format!("namespace: {old_path}")))?;
            entry.path = new_path.clone();
            entry.slug = new_path.split('/').filter(|s| !s.is_empty()).last().unwrap_or("").to_owned();
            entries.insert(new_path.clone(), entry.clone());
            entry
        };
        self.by_id.write().unwrap().insert(entry.namespace_id, new_path.clone());
        // Create alias from old path → new path
        self.add_alias(old_path.to_owned(), new_path);
        Ok(())
    }

    pub fn list_children(&self, parent_path: &str) -> Vec<NamespaceEntry> {
        self.entries.read().unwrap().values()
            .filter(|e| {
                e.path.starts_with(parent_path) &&
                e.path != parent_path &&
                e.path[parent_path.len()..].trim_start_matches('/').chars().all(|c| c != '/')
            })
            .cloned()
            .collect()
    }
}

// =============================================================================
// HG-ID — Multi-Tenant Identity System
// =============================================================================

/// Isolation level for a TenantPartition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IsolationLevel {
    /// Cross-identity reads permitted for authenticated owner.
    None,
    /// Cross-identity reads LOGGED as CrossPartitionAccessEvent.
    SoftIsolated,
    /// Cross-identity reads BLOCKED at the Substrate layer + field-level encryption.
    HardIsolated,
}

/// A named identity handle within a SovereignTenant.
/// Multiple partitions = multiple @handle identities for one real person/org.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantPartition {
    pub partition_id:       Uuid,
    pub identity_handle:    String,         // @handle — unique within SovereignTenant
    pub sovereign_id:       UserId,         // the owner SovereignTenant
    pub display_name:       String,
    pub isolation_level:    IsolationLevel,
    pub visibility_mask:    VisibilityMask,
    pub identity_tag:       String,         // tag applied to rows in this partition
    pub linked_accounts:    Vec<Uuid>,
    pub linked_spaces:      Vec<SpaceId>,
    pub graph_node_id:      Option<crate::graph::EdgeId>,
    pub namespace_path:     NamespacePath,  // e.g. "kogi://@alice-creative/"
    pub split_policy_id:    Option<Uuid>,
    pub public_view_id:     Option<Uuid>,
    pub follower_view_id:   Option<Uuid>,
    pub connection_view_id: Option<Uuid>,
    pub created_at:         DateTime<Utc>,
}

impl TenantPartition {
    pub fn new(sovereign_id: UserId, handle: impl Into<String>) -> Self {
        let handle_str = handle.into();
        Self {
            partition_id:       Uuid::new_v4(),
            identity_handle:    handle_str.clone(),
            sovereign_id,
            display_name:       handle_str.clone(),
            isolation_level:    IsolationLevel::None,
            visibility_mask:    VisibilityMask::default(),
            identity_tag:       format!("@{handle_str}"),
            linked_accounts:    vec![],
            linked_spaces:      vec![],
            graph_node_id:      None,
            namespace_path:     format!("kogi://{handle_str}/"),
            split_policy_id:    None,
            public_view_id:     None,
            follower_view_id:   None,
            connection_view_id: None,
            created_at:         Utc::now(),
        }
    }
}

/// Rules specifying which rows and attributes are visible to which observer type.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VisibilityMask {
    /// Attribute keys visible to unauthenticated callers.
    pub public_attrs:      Vec<crate::cell::AttributeKey>,
    /// Attribute keys visible to followers.
    pub follower_attrs:    Vec<crate::cell::AttributeKey>,
    /// Attribute keys visible to trusted connections.
    pub connection_attrs:  Vec<crate::cell::AttributeKey>,
    /// Attribute keys NEVER visible to non-owners.
    pub owner_only_attrs:  Vec<crate::cell::AttributeKey>,
    /// Context-dependent rules: (condition predicate → visible attr set).
    pub custom_rules:      Vec<VisibilityRule>,
}

impl VisibilityMask {
    pub fn allows_attr(&self, attr_key: &str, caller_type: ObserverType) -> bool {
        if self.owner_only_attrs.iter().any(|k| k == attr_key) {
            return matches!(caller_type, ObserverType::Owner);
        }
        match caller_type {
            ObserverType::Public     => self.public_attrs.iter().any(|k| k == attr_key),
            ObserverType::Follower   => self.follower_attrs.iter().any(|k| k == attr_key),
            ObserverType::Connection => self.connection_attrs.iter().any(|k| k == attr_key),
            ObserverType::Member     => true, // members see connection-level + more
            ObserverType::Owner      => true, // owner sees everything
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibilityRule {
    pub condition: String,                  // DimSlice predicate expression
    pub visible_attrs: Vec<crate::cell::AttributeKey>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObserverType { Public, Follower, Connection, Member, Owner }

/// The identity store for a Grid.
pub struct IdentityStore {
    partitions:  RwLock<HashMap<Uuid, TenantPartition>>,
    by_handle:   RwLock<HashMap<String, Uuid>>,   // @handle → partition_id
    by_sovereign: RwLock<HashMap<UserId, Vec<Uuid>>>, // sovereign_id → [partition_ids]
}

impl IdentityStore {
    pub fn new() -> Self {
        Self {
            partitions:   Default::default(),
            by_handle:    Default::default(),
            by_sovereign: Default::default(),
        }
    }

    pub fn register_partition(&self, partition: TenantPartition) {
        let id       = partition.partition_id;
        let handle   = partition.identity_handle.clone();
        let sovereign = partition.sovereign_id;
        self.partitions.write().unwrap().insert(id, partition);
        self.by_handle.write().unwrap().insert(handle, id);
        self.by_sovereign.write().unwrap().entry(sovereign).or_default().push(id);
    }

    pub fn get_partition(&self, id: Uuid) -> Option<TenantPartition> {
        self.partitions.read().unwrap().get(&id).cloned()
    }

    pub fn resolve_handle(&self, handle: &str) -> Option<TenantPartition> {
        let id = self.by_handle.read().unwrap().get(handle).copied()?;
        self.partitions.read().unwrap().get(&id).cloned()
    }

    pub fn partitions_for(&self, sovereign_id: UserId) -> Vec<TenantPartition> {
        let ids = self.by_sovereign.read().unwrap()
            .get(&sovereign_id).cloned()
            .unwrap_or_default();
        let store = self.partitions.read().unwrap();
        ids.iter().filter_map(|id| store.get(id).cloned()).collect()
    }

    /// Check the PermissionTier of an actor for a given cube and entity.
    /// In production this would query the entity's `users` field.
    pub fn tier_of(
        &self,
        _actor: &str,
        _cube_id: CubeId,
        _d1_key: Option<&crate::cell::DimKey>,
    ) -> crate::cell::PermissionTier {
        // Stub: real implementation reads ComponentUsers from the entity's HyperRow
        crate::cell::PermissionTier::Editor
    }
}

// ─── FederationPeer ──────────────────────────────────────────────────────────

/// A registered federation peer Grid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationPeer {
    pub peer_id:      Uuid,
    pub grid_id:      GridId,
    pub node_id:      String,           // "{region}:{instance_id}" of the peer
    pub endpoint_url: String,           // gRPC or REST endpoint
    pub trusted:      bool,
    pub trust_level:  FederationTrustLevel,
    pub status:       PeerStatus,
    pub last_seen:    Option<DateTime<Utc>>,
    pub vector_clock: crate::crdt::VectorClock, // last known VC of this peer
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FederationTrustLevel {
    Grid,       // Full grid-level trust
    Namespace,  // Trust for specific namespaces only
    Identity,   // Identity-level trust for specific users
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeerStatus {
    Active,
    Degraded,
    Disconnected,
    Handshaking,
}

// ─── FederationManager ───────────────────────────────────────────────────────

pub struct FederationManager {
    peers:   RwLock<HashMap<Uuid, FederationPeer>>,
    grid_id: GridId,
}

impl FederationManager {
    pub fn new(grid_id: GridId) -> Self {
        Self { peers: Default::default(), grid_id }
    }

    pub fn add_peer(&self, peer: FederationPeer) {
        self.peers.write().unwrap().insert(peer.peer_id, peer);
    }

    pub fn get_peer(&self, peer_id: Uuid) -> Option<FederationPeer> {
        self.peers.read().unwrap().get(&peer_id).cloned()
    }

    pub fn active_peers(&self) -> Vec<FederationPeer> {
        self.peers.read().unwrap().values()
            .filter(|p| p.status == PeerStatus::Active)
            .cloned()
            .collect()
    }

    pub fn all_peers(&self) -> Vec<FederationPeer> {
        self.peers.read().unwrap().values().cloned().collect()
    }

    pub fn update_peer_vc(&self, peer_id: Uuid, vc: crate::crdt::VectorClock) {
        if let Some(peer) = self.peers.write().unwrap().get_mut(&peer_id) {
            peer.vector_clock = vc;
            peer.last_seen = Some(Utc::now());
        }
    }

    pub fn mark_peer_status(&self, peer_id: Uuid, status: PeerStatus) {
        if let Some(peer) = self.peers.write().unwrap().get_mut(&peer_id) {
            peer.status = status;
        }
    }

    /// Compute the delta to send to a peer (all ops since peer's last known VC).
    pub fn delta_for_peer(
        &self,
        peer_id: Uuid,
        crdt_log: &crate::crdt::CrdtLog,
    ) -> Vec<crate::crdt::CrdtOperation> {
        let peer_vc = self.peers.read().unwrap()
            .get(&peer_id)
            .map(|p| p.vector_clock.clone())
            .unwrap_or_default();
        crdt_log.delta_since(&peer_vc)
    }
}
