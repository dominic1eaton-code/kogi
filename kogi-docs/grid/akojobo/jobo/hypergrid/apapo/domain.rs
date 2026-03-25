// =============================================================================
// hypergrid::domain — Domain-Layer Pattern
//
// DomainStore trait, PortfolioSystem (Kogi), UmeKernel, QalaOS
// All three domain operating systems as concrete implementations.
// =============================================================================

use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::cell::{
    AttributeKeyDef, CubeId, DimCoordinate, DimKey,
    EntityId, HyperRow, PermissionTier, TypedAttrValue, UserId,
};
use crate::core::{DomainSystem, Grid, Hypercube};
use crate::crdt::{CrdtOperation, CrdtSemantics, LatticeOrder, SerializableCoord, VectorClock};
use crate::dim::{DimensionAxis, StorageEncoding};
use crate::error::HypergridError;
use crate::graph::{EdgeType, HypergraphEdge};
use crate::ops::EventEntry;

// ─── DomainStore trait ────────────────────────────────────────────────────────

/// The codec contract for the Domain-Hypergrid Bridge Layer.
///
/// Every domain entity type gets one DomainStore implementation.
/// This is the ONLY place where domain entity structs ↔ HyperCells translation happens.
///
/// Properties:
///   - Complete:   every field persisted and reconstructable
///   - Correct:    TypedAttrValue variant matches declared AttributeType
///   - Idempotent: write(entity); write(entity) == write(entity) once
///   - Invertible: read(write(entity)) == entity (round-trip)
pub trait DomainStore<Entity, EntityId>: Send + Sync {
    /// Register all Hypercubes and attribute schemas on Grid startup.
    /// Called idempotently from DomainSystem::bootstrap(grid).
    fn bootstrap(grid: &Grid) -> Result<CubeId, HypergridError>
    where Self: Sized;

    /// Translate a domain entity → HyperCells and write to the Grid.
    fn write(grid: &Grid, cube_id: CubeId, entity: &Entity, actor: &str)
        -> Result<(), HypergridError>
    where Self: Sized;

    /// Read HyperCells from the Grid and reconstruct the domain entity.
    fn read(grid: &Grid, cube_id: CubeId, id: EntityId)
        -> Result<Option<Entity>, HypergridError>
    where Self: Sized;

    /// Check whether an entity exists in the Hypercube.
    fn exists(grid: &Grid, cube_id: CubeId, id: EntityId) -> bool
    where Self: Sized;
}

// ─── codec helpers ────────────────────────────────────────────────────────────

/// Build an N=2 coordinate: (entity_id, field_name)
pub fn coord2(grid_id: Uuid, cube_id: CubeId, entity_id: Uuid, field: &str) -> DimCoordinate {
    DimCoordinate::n2(grid_id, cube_id, DimKey::uuid(entity_id), DimKey::text(field))
}

/// Serialize any serde-serializable value as TypedAttrValue::Json.
pub fn as_json<T: Serialize>(v: &T) -> TypedAttrValue {
    TypedAttrValue::Json(serde_json::to_value(v).unwrap_or(serde_json::Value::Null))
}

/// Deserialize a TypedAttrValue::Json into T.
pub fn from_json<T: for<'de> Deserialize<'de>>(v: &TypedAttrValue) -> Option<T> {
    if let TypedAttrValue::Json(j) = v {
        serde_json::from_value(j.clone()).ok()
    } else {
        None
    }
}

// =============================================================================
// KOGI DOMAIN OS
// =============================================================================

// ─── Portfolio domain types ──────────────────────────────────────────────────

pub type ComponentId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentStatus { Draft, Active, Paused, Completed, Archived, Deleted }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentCategory {
    // Items
    Project, Task, Milestone, Goal, Gig, Artifact,
    Asset, Resource, Benefit, Income, Expense, Note, Link,
    // Containers
    Portfolio, Program, Folder, Binder, Collection,
    Space, Workspace, Manifest,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComponentMetadata {
    pub id:           ComponentId,
    pub owners:       Vec<UserId>,
    pub tags:         HashSet<String>,
    pub hashtags:     HashSet<String>,
    pub policy_ids:   Vec<Uuid>,
    pub created_at:   DateTime<Utc>,
    pub updated_at:   DateTime<Utc>,
    pub last_actor:   String,
    pub vector_clock: VectorClock,
    pub version:      String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioComponent {
    pub metadata:     ComponentMetadata,
    pub name:         String,
    pub description:  String,
    pub status:       ComponentStatus,
    pub category:     ComponentCategory,
    pub visibility:   String,

    // Financial
    pub budget:        f64,
    pub budget_spent:  f64,

    // Graph
    pub children:      Vec<ComponentId>,
    pub dependencies:  Vec<ComponentId>,
    pub links:         Vec<Uuid>,

    // AI-computed (read-only; written by kogi-engine)
    pub health_score:  Option<f64>,
    pub risk_score:    Option<f64>,

    pub due_date:      Option<DateTime<Utc>>,
    pub payload:       Option<serde_json::Value>,
}

impl PortfolioComponent {
    pub fn new(creator: UserId, name: impl Into<String>, category: ComponentCategory) -> Self {
        let now = Utc::now();
        Self {
            metadata: ComponentMetadata {
                id:          Uuid::new_v4(),
                owners:      vec![creator],
                tags:        HashSet::new(),
                hashtags:    HashSet::new(),
                policy_ids:  vec![],
                created_at:  now,
                updated_at:  now,
                last_actor:  String::new(),
                vector_clock: VectorClock::new(),
                version:     "1.0.0".into(),
            },
            name:         name.into(),
            description:  String::new(),
            status:       ComponentStatus::Draft,
            category,
            visibility:   "Private".into(),
            budget:        0.0,
            budget_spent:  0.0,
            children:      vec![],
            dependencies:  vec![],
            links:         vec![],
            health_score:  None,
            risk_score:    None,
            due_date:      None,
            payload:       None,
        }
    }
}

// ─── Kogi field name constants ────────────────────────────────────────────────

mod kogi_fields {
    pub const NAME:          &str = "name";
    pub const DESCRIPTION:   &str = "description";
    pub const STATUS:        &str = "status";
    pub const CATEGORY:      &str = "category";
    pub const VISIBILITY:    &str = "visibility";
    pub const BUDGET:        &str = "budget";
    pub const BUDGET_SPENT:  &str = "budget_spent";
    pub const OWNERS:        &str = "owners";
    pub const TAGS:          &str = "tags";
    pub const HASHTAGS:      &str = "hashtags";
    pub const POLICY_IDS:    &str = "policy_ids";
    pub const CHILDREN:      &str = "children";
    pub const DEPENDENCIES:  &str = "dependencies";
    pub const LINKS:         &str = "links";
    pub const HEALTH_SCORE:  &str = "health_score";
    pub const RISK_SCORE:    &str = "risk_score";
    pub const DUE_DATE:      &str = "due_date";
    pub const PAYLOAD:       &str = "payload";
    pub const VERSION:       &str = "version";
    pub const CREATED_AT:    &str = "created_at";
    pub const UPDATED_AT:    &str = "updated_at";
    pub const LAST_ACTOR:    &str = "last_actor";
    pub const VECTOR_CLOCK:  &str = "vector_clock";
}

// ─── ComponentStore ──────────────────────────────────────────────────────────

/// The Kogi DomainStore codec for PortfolioComponent ↔ HyperCells.
pub struct ComponentStore;

impl ComponentStore {
    pub const CUBE_NAME: &'static str = "kogi.portfolio.components";
    pub const KPI_CUBE:  &'static str = "kogi.portfolio.kpis";

    /// Register schema for kogi.portfolio.components.
    pub fn bootstrap(grid: &Grid) -> Result<CubeId, HypergridError> {
        use kogi_fields as F;

        let dims = vec![
            DimensionAxis::entity("components"),  // D₁: ComponentId
            DimensionAxis::property("fields"),    // D₂: field name
        ];

        let cube_id = grid.create_cube(Self::CUBE_NAME, dims)?;

        grid.register_attrs(cube_id, vec![
            // ── Mandatory metadata ─────────────────────────────────────
            AttributeKeyDef::lww_text(F::NAME,         "Name",         PermissionTier::Contributor),
            AttributeKeyDef::lww_text(F::DESCRIPTION,  "Description",  PermissionTier::Editor),
            AttributeKeyDef::lww_text(F::LAST_ACTOR,   "Last Actor",   PermissionTier::System),
            AttributeKeyDef::max_register(F::VERSION,  "Version",      PermissionTier::System),
            AttributeKeyDef::lww_json(F::STATUS,       "Status",       PermissionTier::Editor)
                .with_lattice(LatticeOrder::kogi_component_status()),
            AttributeKeyDef::lww_json(F::CATEGORY,     "Category",     PermissionTier::Owner),
            AttributeKeyDef::lww_json(F::VISIBILITY,   "Visibility",   PermissionTier::Owner),
            AttributeKeyDef::lww_json(F::CREATED_AT,   "Created At",   PermissionTier::System),
            AttributeKeyDef::lww_json(F::UPDATED_AT,   "Updated At",   PermissionTier::System),
            AttributeKeyDef::lww_json(F::VECTOR_CLOCK, "Vector Clock", PermissionTier::System),

            // ── OR-Set fields ──────────────────────────────────────────
            AttributeKeyDef::orset(F::OWNERS,       "Owners",       PermissionTier::Manager),
            AttributeKeyDef::orset(F::TAGS,         "Tags",         PermissionTier::Contributor),
            AttributeKeyDef::orset(F::HASHTAGS,     "Hashtags",     PermissionTier::Contributor),
            AttributeKeyDef::orset(F::POLICY_IDS,   "Policy IDs",   PermissionTier::Admin),
            AttributeKeyDef::orset(F::CHILDREN,     "Children",     PermissionTier::Manager),
            AttributeKeyDef::orset(F::DEPENDENCIES, "Dependencies", PermissionTier::Editor),
            AttributeKeyDef::orset(F::LINKS,        "Links",        PermissionTier::Editor),

            // ── Financial ─────────────────────────────────────────────
            AttributeKeyDef::lww_json(F::BUDGET,       "Budget",        PermissionTier::Manager),
            AttributeKeyDef::pn_counter(F::BUDGET_SPENT, "Budget Spent", PermissionTier::Editor),

            // ── Optional ──────────────────────────────────────────────
            AttributeKeyDef::lww_json(F::DUE_DATE, "Due Date",  PermissionTier::Editor),
            AttributeKeyDef::lww_json(F::PAYLOAD,  "Payload",   PermissionTier::Editor),

            // ── AI-computed (System write only) ───────────────────────
            AttributeKeyDef::ai_signal(F::HEALTH_SCORE, "Health Score",  "kogi.health_score_engine.v2"),
            AttributeKeyDef::ai_signal(F::RISK_SCORE,   "Risk Score",    "kogi.risk_engine.v1"),
        ])?;

        // Also register the Tier-1 formula for budget_remaining
        grid.register_attrs(cube_id, vec![
            AttributeKeyDef::tier1_formula(
                "budget_remaining", "Budget Remaining",
                crate::cell::FormulaExpr::Sub(
                    Box::new(crate::cell::FormulaExpr::Attr(F::BUDGET.into())),
                    Box::new(crate::cell::FormulaExpr::Attr(F::BUDGET_SPENT.into())),
                ),
            ),
        ])?;

        Ok(cube_id)
    }

    /// Write a PortfolioComponent to the Grid.
    pub fn write(grid: &Grid, cube_id: CubeId, c: &PortfolioComponent, actor: &str)
        -> Result<(), HypergridError>
    {
        use kogi_fields as F;
        let id = c.metadata.id;
        let g  = grid.grid_id;
        macro_rules! w {
            ($field:expr, $val:expr) => {
                grid.write_cell(cube_id, coord2(g, cube_id, id, $field), $field, $val, actor)?;
            };
        }

        w!(F::NAME,         TypedAttrValue::Text(c.name.clone()));
        w!(F::DESCRIPTION,  TypedAttrValue::Text(c.description.clone()));
        w!(F::STATUS,       TypedAttrValue::Text(format!("{:?}", c.status)));
        w!(F::CATEGORY,     as_json(&c.category));
        w!(F::VISIBILITY,   TypedAttrValue::Text(c.visibility.clone()));
        w!(F::BUDGET,       TypedAttrValue::Number(c.budget));
        w!(F::DUE_DATE,     as_json(&c.due_date));
        w!(F::VERSION,      TypedAttrValue::Text(c.metadata.version.clone()));
        w!(F::CREATED_AT,   as_json(&c.metadata.created_at));
        w!(F::UPDATED_AT,   as_json(&c.metadata.updated_at));
        w!(F::LAST_ACTOR,   TypedAttrValue::Text(actor.to_owned()));
        w!(F::VECTOR_CLOCK, as_json(&c.metadata.vector_clock));
        if let Some(payload) = &c.payload {
            w!(F::PAYLOAD, TypedAttrValue::Json(payload.clone()));
        }

        // OR-Set fields: use AddToSet ops (concurrent-safe)
        for owner in &c.metadata.owners {
            let op = CrdtOperation::AddToSet {
                coord:      SerializableCoord { grid_id: g, cube_id, keys: vec![DimKey::uuid(id), DimKey::text(F::OWNERS)] },
                attr_key:   F::OWNERS.into(),
                element:    TypedAttrValue::Text(owner.to_string()),
                unique_tag: Uuid::new_v5(&Uuid::NAMESPACE_OID, owner.to_string().as_bytes()),
                actor:      actor.to_owned(),
            };
            // In production apply via CrdtMergeEngine; here write as Json
            let owners_json: Vec<String> = c.metadata.owners.iter().map(|u| u.to_string()).collect();
            w!(F::OWNERS, as_json(&owners_json));
            break; // write once as JSON array
        }

        let tags_vec: Vec<&str> = c.metadata.tags.iter().map(|s| s.as_str()).collect();
        w!(F::TAGS, as_json(&tags_vec));

        let deps_vec: Vec<String> = c.dependencies.iter().map(|u| u.to_string()).collect();
        w!(F::DEPENDENCIES, as_json(&deps_vec));

        let children_vec: Vec<String> = c.children.iter().map(|u| u.to_string()).collect();
        w!(F::CHILDREN, as_json(&children_vec));

        Ok(())
    }

    /// Read HyperCells from the Grid and reconstruct a PortfolioComponent.
    pub fn read(grid: &Grid, cube_id: CubeId, id: ComponentId)
        -> Result<Option<PortfolioComponent>, HypergridError>
    {
        use kogi_fields as F;
        let row = grid.get_row(cube_id, &DimKey::uuid(id));
        if row.attrs.is_empty() { return Ok(None); }

        let get_text = |k: &str| row.get_str(k).unwrap_or("").to_owned();
        let get_num  = |k: &str| row.get_f64(k).unwrap_or(0.0);

        let status = match get_text(F::STATUS).as_str() {
            "Active"    => ComponentStatus::Active,
            "Paused"    => ComponentStatus::Paused,
            "Completed" => ComponentStatus::Completed,
            "Archived"  => ComponentStatus::Archived,
            "Deleted"   => ComponentStatus::Deleted,
            _           => ComponentStatus::Draft,
        };

        let category: ComponentCategory =
            row.get_attr(F::CATEGORY)
               .and_then(|v| from_json(v))
               .unwrap_or(ComponentCategory::Project);

        let owners: Vec<UserId> = row.get_attr(F::OWNERS)
            .and_then(|v| from_json::<Vec<String>>(v))
            .unwrap_or_default()
            .iter()
            .filter_map(|s| Uuid::parse_str(s).ok())
            .collect();

        let tags: HashSet<String> = row.get_attr(F::TAGS)
            .and_then(|v| from_json::<Vec<String>>(v))
            .unwrap_or_default()
            .into_iter()
            .collect();

        let dependencies: Vec<ComponentId> = row.get_attr(F::DEPENDENCIES)
            .and_then(|v| from_json::<Vec<String>>(v))
            .unwrap_or_default()
            .iter()
            .filter_map(|s| Uuid::parse_str(s).ok())
            .collect();

        let children: Vec<ComponentId> = row.get_attr(F::CHILDREN)
            .and_then(|v| from_json::<Vec<String>>(v))
            .unwrap_or_default()
            .iter()
            .filter_map(|s| Uuid::parse_str(s).ok())
            .collect();

        let vector_clock: VectorClock = row.get_attr(F::VECTOR_CLOCK)
            .and_then(|v| from_json(v))
            .unwrap_or_default();

        let created_at: DateTime<Utc> = row.get_attr(F::CREATED_AT)
            .and_then(|v| from_json(v))
            .unwrap_or_else(Utc::now);

        let updated_at: DateTime<Utc> = row.get_attr(F::UPDATED_AT)
            .and_then(|v| from_json(v))
            .unwrap_or_else(Utc::now);

        let health_score = row.get_attr(F::HEALTH_SCORE).and_then(|v| match v {
            TypedAttrValue::AiSignal { value, .. } => value.as_f64(),
            other => other.as_f64(),
        });
        let risk_score = row.get_attr(F::RISK_SCORE).and_then(|v| match v {
            TypedAttrValue::AiSignal { value, .. } => value.as_f64(),
            other => other.as_f64(),
        });

        Ok(Some(PortfolioComponent {
            metadata: ComponentMetadata {
                id, owners, tags,
                hashtags:    HashSet::new(),
                policy_ids:  vec![],
                created_at, updated_at,
                last_actor:   get_text(F::LAST_ACTOR),
                vector_clock,
                version:      get_text(F::VERSION),
            },
            name:          get_text(F::NAME),
            description:   get_text(F::DESCRIPTION),
            status,
            category,
            visibility:    get_text(F::VISIBILITY),
            budget:         get_num(F::BUDGET),
            budget_spent:   get_num(F::BUDGET_SPENT),
            children, dependencies,
            links:         vec![],
            health_score, risk_score,
            due_date:      row.get_attr(F::DUE_DATE).and_then(|v| from_json(v)),
            payload:       row.get_attr(F::PAYLOAD).and_then(|v| {
                if let TypedAttrValue::Json(j) = v { Some(j.clone()) } else { None }
            }),
        }))
    }

    pub fn exists(grid: &Grid, cube_id: CubeId, id: ComponentId) -> bool {
        !grid.get_row(cube_id, &DimKey::uuid(id)).attrs.is_empty()
    }
}

// ─── PortfolioSystem (Kogi root orchestrator) ─────────────────────────────────

pub struct PortfolioSystem {
    pub grid:            Grid,
    pub components_cube: CubeId,
    pub kpis_cube:       CubeId,
    pub node_id:         String,
}

impl PortfolioSystem {
    pub fn new(node_id: impl Into<String>) -> Result<Self, HypergridError> {
        let node_id_str = node_id.into();
        let grid = Grid::new("kogi-portfolio", &node_id_str);

        let components_cube = ComponentStore::bootstrap(&grid)?;

        // KPIs cube: N=3 (ComponentId × metric_name × period)
        let kpi_cube_id = grid.create_cube(ComponentStore::KPI_CUBE, vec![
            DimensionAxis::entity("components"),
            DimensionAxis::property("metrics"),
            DimensionAxis::time("period"),
        ])?;

        grid.set_ready();

        Ok(Self {
            components_cube,
            kpis_cube:  kpi_cube_id,
            node_id:    node_id_str,
            grid,
        })
    }

    // ── Create ────────────────────────────────────────────────────────────

    pub fn create_component(
        &self,
        creator: UserId,
        name: impl Into<String>,
        category: ComponentCategory,
    ) -> Result<ComponentId, HypergridError> {
        let component = PortfolioComponent::new(creator, name, category);
        let id = component.metadata.id;
        ComponentStore::write(&self.grid, self.components_cube, &component, &self.node_id)?;

        // Register node in Hypergraph
        self.grid.hypergraph.register_entity(self.components_cube, DimKey::uuid(id), self.grid.grid_id);

        // Namespace registration
        let ns_path = format!("kogi://{}/components/{}/", &self.node_id, id);
        let _ = self.grid.namespace_reg.register(crate::space::NamespaceEntry::new(
            ns_path,
            crate::cell::EntityRef { cube_id: self.components_cube, d1_key: id, grid_id: None },
            "Row",
        ));

        tracing::debug!(component_id = %id, "PortfolioComponent created");
        Ok(id)
    }

    // ── Read ──────────────────────────────────────────────────────────────

    pub fn get_component(&self, id: ComponentId) -> Result<Option<PortfolioComponent>, HypergridError> {
        ComponentStore::read(&self.grid, self.components_cube, id)
    }

    // ── Update ────────────────────────────────────────────────────────────

    pub fn update_status(&self, id: ComponentId, new_status: ComponentStatus, actor: &str)
        -> Result<(), HypergridError>
    {
        let coord = coord2(self.grid.grid_id, self.components_cube, id, kogi_fields::STATUS);
        self.grid.write_cell(
            self.components_cube, coord, kogi_fields::STATUS,
            TypedAttrValue::Text(format!("{:?}", new_status)),
            actor,
        )?;
        Ok(())
    }

    pub fn set_budget(&self, id: ComponentId, budget: f64, actor: &str)
        -> Result<(), HypergridError>
    {
        let coord = coord2(self.grid.grid_id, self.components_cube, id, kogi_fields::BUDGET);
        self.grid.write_cell(
            self.components_cube, coord, kogi_fields::BUDGET,
            TypedAttrValue::Number(budget),
            actor,
        )?;
        Ok(())
    }

    // ── Add Dependency (with cycle detection) ─────────────────────────────

    pub fn add_dependency(&self, from_id: ComponentId, to_id: ComponentId, actor: &str)
        -> Result<(), HypergridError>
    {
        let from_node = self.grid.hypergraph
            .node_for_entity(self.components_cube, &DimKey::uuid(from_id))
            .unwrap_or_else(|| self.grid.hypergraph.register_entity(
                self.components_cube, DimKey::uuid(from_id), self.grid.grid_id,
            ));
        let to_node = self.grid.hypergraph
            .node_for_entity(self.components_cube, &DimKey::uuid(to_id))
            .unwrap_or_else(|| self.grid.hypergraph.register_entity(
                self.components_cube, DimKey::uuid(to_id), self.grid.grid_id,
            ));

        let edge = HypergraphEdge::new(from_node, to_node, EdgeType::Dependency, actor.to_owned());
        self.grid.hypergraph.add_edge(edge)?;  // DFS cycle detection enforced

        // Also persist in dependencies OR-Set
        let coord = coord2(self.grid.grid_id, self.components_cube, from_id, kogi_fields::DEPENDENCIES);
        self.grid.write_cell(
            self.components_cube, coord, kogi_fields::DEPENDENCIES,
            TypedAttrValue::Text(to_id.to_string()),
            actor,
        )?;
        Ok(())
    }

    // ── HyperQL passthrough (SPEC-009) ────────────────────────────────────

    pub fn hyperql(&self, query: crate::ql::HyperQuery) -> crate::ql::QueryResult {
        self.grid.hyperql(query)
    }

    // ── Stats (SPEC-010) ──────────────────────────────────────────────────

    pub fn stats(&self) -> PortfolioStats {
        let hg = self.grid.stats();
        let components = self.grid.scan_rows(self.components_cube);
        PortfolioStats {
            component_count: components.len() as u64,
            edge_count:      self.grid.hypergraph.edge_count() as u64,
            event_count:     self.grid.event_log.len(),
            hg_total_cells:  hg.total_cells,
            hg_event_count:  hg.event_count,
            hg_crdt_ops:     hg.crdt_op_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioStats {
    pub component_count: u64,
    pub edge_count:      u64,
    pub event_count:     u64,
    pub hg_total_cells:  u64,
    pub hg_event_count:  u64,
    pub hg_crdt_ops:     u64,
}

// =============================================================================
// UME DOMAIN OS
// =============================================================================

pub type ModuleId = Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleLifecycleState {
    Registered, Starting, Running, Degraded, Recovering, Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgModule {
    pub module_id:       ModuleId,
    pub name:            String,
    pub domain_area:     String,
    pub lifecycle_state: ModuleLifecycleState,
    pub health_score:    Option<f64>,
    pub version:         String,
    pub executor_id:     String,
    pub dependencies:    Vec<ModuleId>,
    pub restart_count:   u64,
    pub created_at:      DateTime<Utc>,
    pub updated_at:      DateTime<Utc>,
}

impl OrgModule {
    pub fn new(name: impl Into<String>, domain_area: impl Into<String>, executor_id: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            module_id:       Uuid::new_v4(),
            name:            name.into(),
            domain_area:     domain_area.into(),
            lifecycle_state: ModuleLifecycleState::Registered,
            health_score:    None,
            version:         "1.0.0".into(),
            executor_id:     executor_id.into(),
            dependencies:    vec![],
            restart_count:   0,
            created_at:      now,
            updated_at:      now,
        }
    }
}

pub struct ModuleStore;

impl ModuleStore {
    pub const CUBE_NAME: &'static str = "ume.kernel.modules";

    pub fn bootstrap(grid: &Grid) -> Result<CubeId, HypergridError> {
        let cube_id = grid.create_cube(Self::CUBE_NAME, vec![
            DimensionAxis::entity("modules"),
            DimensionAxis::property("fields"),
        ])?;

        grid.register_attrs(cube_id, vec![
            AttributeKeyDef::lww_text("name",           "Module Name",    PermissionTier::System),
            AttributeKeyDef::lww_text("domain_area",    "Domain Area",    PermissionTier::System),
            AttributeKeyDef::lww_text("executor_id",    "Executor ID",    PermissionTier::System),
            AttributeKeyDef::lww_text("version",        "Version",        PermissionTier::System),
            AttributeKeyDef::lww_text("last_actor",     "Last Actor",     PermissionTier::System),
            AttributeKeyDef::lww_json("lifecycle_state","Lifecycle State",PermissionTier::System)
                .with_lattice(LatticeOrder::ume_module_lifecycle()),
            AttributeKeyDef::grow_only("restart_count", "Restart Count",  PermissionTier::System),
            AttributeKeyDef::lww_json("created_at",     "Created At",     PermissionTier::System),
            AttributeKeyDef::lww_json("updated_at",     "Updated At",     PermissionTier::System),
            AttributeKeyDef::orset("dependencies",      "Dependencies",   PermissionTier::System),
            AttributeKeyDef::ai_signal("health_score",  "Health Score",   "ume.module_health_engine"),
        ])?;

        Ok(cube_id)
    }

    pub fn write(grid: &Grid, cube_id: CubeId, m: &OrgModule, actor: &str)
        -> Result<(), HypergridError>
    {
        let id = m.module_id;
        let g  = grid.grid_id;
        macro_rules! w {
            ($field:expr, $val:expr) => {
                grid.write_cell(cube_id, coord2(g, cube_id, id, $field), $field, $val, actor)?;
            };
        }
        w!("name",            TypedAttrValue::Text(m.name.clone()));
        w!("domain_area",     TypedAttrValue::Text(m.domain_area.clone()));
        w!("executor_id",     TypedAttrValue::Text(m.executor_id.clone()));
        w!("version",         TypedAttrValue::Text(m.version.clone()));
        w!("lifecycle_state", TypedAttrValue::Text(format!("{:?}", m.lifecycle_state)));
        w!("created_at",      as_json(&m.created_at));
        w!("updated_at",      as_json(&m.updated_at));
        w!("last_actor",      TypedAttrValue::Text(actor.to_owned()));
        let dep_strs: Vec<String> = m.dependencies.iter().map(|u| u.to_string()).collect();
        w!("dependencies",    as_json(&dep_strs));
        Ok(())
    }

    pub fn read(grid: &Grid, cube_id: CubeId, id: ModuleId) -> Result<Option<OrgModule>, HypergridError> {
        let row = grid.get_row(cube_id, &DimKey::uuid(id));
        if row.attrs.is_empty() { return Ok(None); }

        let get_text = |k: &str| row.get_str(k).unwrap_or("").to_owned();

        let lifecycle_state = match get_text("lifecycle_state").as_str() {
            "Starting"   => ModuleLifecycleState::Starting,
            "Running"    => ModuleLifecycleState::Running,
            "Degraded"   => ModuleLifecycleState::Degraded,
            "Recovering" => ModuleLifecycleState::Recovering,
            "Stopped"    => ModuleLifecycleState::Stopped,
            _            => ModuleLifecycleState::Registered,
        };

        Ok(Some(OrgModule {
            module_id:  id,
            name:       get_text("name"),
            domain_area: get_text("domain_area"),
            lifecycle_state,
            health_score: row.get_f64("health_score"),
            version:     get_text("version"),
            executor_id: get_text("executor_id"),
            dependencies: row.get_attr("dependencies")
                .and_then(|v| from_json::<Vec<String>>(v))
                .unwrap_or_default()
                .iter()
                .filter_map(|s| Uuid::parse_str(s).ok())
                .collect(),
            restart_count: row.get_f64("restart_count").unwrap_or(0.0) as u64,
            created_at:  row.get_attr("created_at").and_then(|v| from_json(v)).unwrap_or_else(Utc::now),
            updated_at:  row.get_attr("updated_at").and_then(|v| from_json(v)).unwrap_or_else(Utc::now),
        }))
    }
}

/// Ume root orchestrator.
pub struct UmeKernel {
    pub grid:        Grid,
    pub modules_cube: CubeId,
    pub node_id:     String,
}

impl UmeKernel {
    pub fn new(node_id: impl Into<String>) -> Result<Self, HypergridError> {
        let node_id_str = node_id.into();
        let grid = Grid::new("ume-kernel", &node_id_str);
        let modules_cube = ModuleStore::bootstrap(&grid)?;
        grid.set_ready();
        Ok(Self { modules_cube, node_id: node_id_str, grid })
    }

    pub fn register_module(&self, module: OrgModule) -> Result<ModuleId, HypergridError> {
        let id = module.module_id;
        ModuleStore::write(&self.grid, self.modules_cube, &module, &self.node_id)?;
        self.grid.hypergraph.register_entity(self.modules_cube, DimKey::uuid(id), self.grid.grid_id);
        Ok(id)
    }

    pub fn get_module(&self, id: ModuleId) -> Result<Option<OrgModule>, HypergridError> {
        ModuleStore::read(&self.grid, self.modules_cube, id)
    }

    pub fn transition_module(
        &self, id: ModuleId, new_state: ModuleLifecycleState, actor: &str,
    ) -> Result<(), HypergridError> {
        let coord = coord2(self.grid.grid_id, self.modules_cube, id, "lifecycle_state");
        self.grid.write_cell(
            self.modules_cube, coord, "lifecycle_state",
            TypedAttrValue::Text(format!("{:?}", new_state)),
            actor,
        )?;
        Ok(())
    }

    pub fn hyperql(&self, query: crate::ql::HyperQuery) -> crate::ql::QueryResult {
        self.grid.hyperql(query)
    }

    pub fn stats(&self) -> UmeStats {
        let hg = self.grid.stats();
        UmeStats {
            module_count: self.grid.scan_rows(self.modules_cube).len() as u64,
            hg_total_cells: hg.total_cells, hg_event_count: hg.event_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UmeStats {
    pub module_count:   u64,
    pub hg_total_cells: u64,
    pub hg_event_count: u64,
}

// =============================================================================
// QALA DOMAIN OS
// =============================================================================

pub type SolutionId = Uuid;
pub type SdeId      = Uuid;
pub type FactoryId  = Uuid;
pub type CcrId      = Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SolutionType { Application, System, Good, Product, Service, Platform, Factory }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Solution {
    pub solution_id:     SolutionId,
    pub name:            String,
    pub solution_type:   SolutionType,
    pub lifecycle_state: String,       // Lattice-governed
    pub version:         String,       // MaxRegister
    pub sde_id:          Option<SdeId>,
    pub factory_id:      FactoryId,
    pub artifact_ids:    Vec<Uuid>,
    pub ccr_ids:         Vec<CcrId>,
    pub risk_score:      Option<f64>,
    pub quality_score:   Option<f64>,
    pub is_factory:      bool,
    pub created_at:      DateTime<Utc>,
    pub updated_at:      DateTime<Utc>,
}

impl Solution {
    pub fn new(name: impl Into<String>, sol_type: SolutionType, factory_id: FactoryId) -> Self {
        let now = Utc::now();
        Self {
            solution_id:   Uuid::new_v4(),
            name:          name.into(),
            solution_type: sol_type,
            lifecycle_state: "Draft".into(),
            version:       "0.1.0".into(),
            sde_id:        None,
            factory_id,
            artifact_ids:  vec![],
            ccr_ids:       vec![],
            risk_score:    None,
            quality_score: None,
            is_factory:    false,
            created_at:    now,
            updated_at:    now,
        }
    }
}

pub struct SolutionStore;

impl SolutionStore {
    pub const CUBE_NAME: &'static str = "qala.solutions";

    pub fn bootstrap(grid: &Grid) -> Result<CubeId, HypergridError> {
        let cube_id = grid.create_cube(Self::CUBE_NAME, vec![
            DimensionAxis::entity("solutions"),
            DimensionAxis::property("fields"),
            DimensionAxis::time("versions"), // D₃: version history
        ])?;

        grid.register_attrs(cube_id, vec![
            AttributeKeyDef::lww_text("name",            "Name",             PermissionTier::Editor),
            AttributeKeyDef::lww_json("solution_type",   "Solution Type",    PermissionTier::Owner),
            AttributeKeyDef::lww_json("lifecycle_state", "Lifecycle State",  PermissionTier::Manager)
                .with_lattice(LatticeOrder::qala_solution_lifecycle()),
            AttributeKeyDef::max_register("version",     "Version",          PermissionTier::Manager),
            AttributeKeyDef::lww_json("sde_id",          "SDE ID",           PermissionTier::Manager),
            AttributeKeyDef::lww_json("factory_id",      "Factory ID",       PermissionTier::System),
            AttributeKeyDef::orset("artifact_ids",       "Artifact IDs",     PermissionTier::Editor),
            AttributeKeyDef::orset("ccr_ids",            "CCR IDs",          PermissionTier::Manager),
            AttributeKeyDef::lww_json("is_factory",      "Is Factory",       PermissionTier::System),
            AttributeKeyDef::lww_json("created_at",      "Created At",       PermissionTier::System),
            AttributeKeyDef::lww_json("updated_at",      "Updated At",       PermissionTier::System),
            AttributeKeyDef::ai_signal("risk_score",     "Risk Score",       "qala.defect_prediction_engine"),
            AttributeKeyDef::ai_signal("quality_score",  "Quality Score",    "qala.quality_engine"),
            AttributeKeyDef::ai_signal("drift_status",   "Drift Status",     "qala.drift_detection_engine"),
        ])?;

        Ok(cube_id)
    }

    pub fn write(grid: &Grid, cube_id: CubeId, s: &Solution, actor: &str)
        -> Result<(), HypergridError>
    {
        let id = s.solution_id;
        let g  = grid.grid_id;
        macro_rules! w {
            ($f:expr, $v:expr) => {
                grid.write_cell(cube_id, coord2(g, cube_id, id, $f), $f, $v, actor)?;
            };
        }
        w!("name",            TypedAttrValue::Text(s.name.clone()));
        w!("solution_type",   as_json(&s.solution_type));
        w!("lifecycle_state", TypedAttrValue::Text(s.lifecycle_state.clone()));
        w!("version",         TypedAttrValue::Text(s.version.clone()));
        w!("factory_id",      as_json(&s.factory_id));
        w!("is_factory",      TypedAttrValue::Bool(s.is_factory));
        w!("created_at",      as_json(&s.created_at));
        w!("updated_at",      as_json(&s.updated_at));
        if let Some(sde_id) = &s.sde_id {
            w!("sde_id", as_json(sde_id));
        }
        Ok(())
    }

    pub fn read(grid: &Grid, cube_id: CubeId, id: SolutionId)
        -> Result<Option<Solution>, HypergridError>
    {
        let row = grid.get_row(cube_id, &DimKey::uuid(id));
        if row.attrs.is_empty() { return Ok(None); }

        let get_text = |k: &str| row.get_str(k).unwrap_or("").to_owned();

        Ok(Some(Solution {
            solution_id:    id,
            name:           get_text("name"),
            solution_type:  row.get_attr("solution_type").and_then(|v| from_json(v))
                .unwrap_or(SolutionType::Application),
            lifecycle_state: get_text("lifecycle_state"),
            version:        get_text("version"),
            sde_id:         row.get_attr("sde_id").and_then(|v| from_json(v)),
            factory_id:     row.get_attr("factory_id").and_then(|v| from_json(v))
                .unwrap_or(Uuid::nil()),
            artifact_ids:   row.get_attr("artifact_ids")
                .and_then(|v| from_json::<Vec<String>>(v))
                .unwrap_or_default()
                .iter()
                .filter_map(|s| Uuid::parse_str(s).ok())
                .collect(),
            ccr_ids:        vec![],
            risk_score:     row.get_f64("risk_score"),
            quality_score:  row.get_f64("quality_score"),
            is_factory:     row.get_attr("is_factory")
                .and_then(|v| if let TypedAttrValue::Bool(b) = v { Some(*b) } else { None })
                .unwrap_or(false),
            created_at:     row.get_attr("created_at").and_then(|v| from_json(v)).unwrap_or_else(Utc::now),
            updated_at:     row.get_attr("updated_at").and_then(|v| from_json(v)).unwrap_or_else(Utc::now),
        }))
    }
}

/// Qala root orchestrator.
pub struct QalaOS {
    pub grid:           Grid,
    pub solutions_cube: CubeId,
    pub node_id:        String,
}

impl QalaOS {
    pub fn new(node_id: impl Into<String>) -> Result<Self, HypergridError> {
        let node_id_str = node_id.into();
        let grid = Grid::new("qala-os", &node_id_str);
        let solutions_cube = SolutionStore::bootstrap(&grid)?;
        grid.set_ready();
        Ok(Self { solutions_cube, node_id: node_id_str, grid })
    }

    pub fn create_solution(
        &self, name: impl Into<String>, sol_type: SolutionType, factory_id: FactoryId, actor: &str,
    ) -> Result<SolutionId, HypergridError> {
        let solution = Solution::new(name, sol_type, factory_id);
        let id = solution.solution_id;
        SolutionStore::write(&self.grid, self.solutions_cube, &solution, actor)?;
        self.grid.hypergraph.register_entity(self.solutions_cube, DimKey::uuid(id), self.grid.grid_id);
        Ok(id)
    }

    pub fn get_solution(&self, id: SolutionId) -> Result<Option<Solution>, HypergridError> {
        SolutionStore::read(&self.grid, self.solutions_cube, id)
    }

    pub fn promote_lifecycle(
        &self, id: SolutionId, new_state: &str, actor: &str,
    ) -> Result<(), HypergridError> {
        let coord = coord2(self.grid.grid_id, self.solutions_cube, id, "lifecycle_state");
        self.grid.write_cell(
            self.solutions_cube, coord, "lifecycle_state",
            TypedAttrValue::Text(new_state.to_owned()),
            actor,
        )?;
        Ok(())
    }

    pub fn hyperql(&self, query: crate::ql::HyperQuery) -> crate::ql::QueryResult {
        self.grid.hyperql(query)
    }

    pub fn stats(&self) -> QalaStats {
        let hg = self.grid.stats();
        QalaStats {
            solution_count: self.grid.scan_rows(self.solutions_cube).len() as u64,
            hg_total_cells: hg.total_cells, hg_event_count: hg.event_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QalaStats {
    pub solution_count: u64,
    pub hg_total_cells: u64,
    pub hg_event_count: u64,
}

// ─── AttributeKeyDef builders (re-exported from cell with lattice support) ───

impl AttributeKeyDef {
    pub fn with_lattice(mut self, order: LatticeOrder) -> Self {
        self.crdt_semantics = CrdtSemantics::Lattice(order);
        self
    }
}
