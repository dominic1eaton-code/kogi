//! ComponentStore — Component ↔ HyperCell serialisation.
//!
//! ## Hypergrid cell layout
//!
//! The components Hypercube is N=2 (EntityAxis × PropertyAxis):
//!
//! ```text
//! D1 = ComponentId (UUID)         ← entity row
//! D2 = field name  (String)       ← property column
//!
//! For every cell: coord = (component_id, field_name)
//!                 cell.attributes["value"] = <typed field value>
//! ```
//!
//! This is the canonical Hypergrid convention:
//!   - D2 key  = the column/property name   (e.g. "name", "status")
//!   - "value" attribute = the cell's data  (e.g. Text("My Portfolio"))
//!
//! The `AttributeKeyRegistry` only needs to contain `"value"` (which is
//! registered as a builtin by every new Hypercube). We do NOT register each
//! field name as an attribute key — they are D2 *axis* keys, not attribute keys.

use serde_json::Value as JsonValue;

use hypergrid::cell::{
    CubeId,
    DimCoordinate, DimKey, TypedAttrValue,
};

use hypergrid::core::{Grid, Hypercube};
use hypergrid::error::HypergridResult;

use crate::component::{Component, ComponentPayload};
use crate::error::{PortfolioError, PortfolioResult};
use crate::types::ComponentId;

// ── Constants ─────────────────────────────────────────────────────────────────

pub const COMPONENTS_CUBE_NAME: &str = "kogi.portfolio.components";

/// D2 axis keys (property column names) used in the components Hypercube.
/// These are NOT attribute keys — they are D2 dimension keys.
pub mod fields {
    pub const NAME:           &str = "name";
    pub const DESCRIPTION:    &str = "description";
    pub const STATUS:         &str = "status";
    pub const STATE:          &str = "state";
    pub const VISIBILITY:     &str = "visibility";
    pub const CATEGORY:       &str = "category";
    pub const PAYLOAD:        &str = "payload";
    pub const OWNERS:         &str = "owners";
    pub const TAGS:           &str = "tags";
    pub const HASHTAGS:       &str = "hashtags";
    pub const TOPICS:         &str = "topics";
    pub const BUDGET:         &str = "budget";
    pub const BUDGET_SPENT:   &str = "budget_spent";
    pub const RESOURCE_UNITS: &str = "resource_units";
    pub const VERSION:        &str = "version";
    pub const POLICY_IDS:     &str = "policy_ids";
    pub const CHILDREN:       &str = "children";
    pub const PARENTS:        &str = "parents";
    pub const DEPENDENCIES:   &str = "dependencies";
    pub const DEPENDENTS:     &str = "dependents";
    pub const LINKS:          &str = "links";
    pub const TOOLBOX_IDS:    &str = "toolbox_ids";
    pub const RISKS:          &str = "risks";
    pub const USERS:          &str = "users";
    pub const ANALYTICS:      &str = "analytics";
    pub const ITEM_BOOK:      &str = "item_book";
    pub const CREATED_AT:     &str = "created_at";
    pub const UPDATED_AT:     &str = "updated_at";
    pub const LAST_ACTOR:     &str = "last_actor";
    pub const VECTOR_CLOCK:   &str = "vector_clock";
    pub const VERSION_HISTORY:&str = "version_history";
}

// ── ComponentStore ─────────────────────────────────────────────────────────────

pub struct ComponentStore;

impl ComponentStore {
    // ── Schema bootstrap ──────────────────────────────────────────────────
    //
    // The "value" attribute key is already registered by every new Hypercube
    // as a builtin. We don't need to register the field names — they are D2
    // axis keys, not attribute keys. This method is a no-op kept for clarity.
    pub fn register_schema(_cube: &mut Hypercube) -> HypergridResult<()> {
        // All writes use the builtin "value" attribute key.
        // No additional registration needed.
        Ok(())
    }

    // ── Write (Component → HyperCells) ────────────────────────────────────
    //
    // Each field is stored as one cell at coord(component_id, field_name),
    // with the field value written into cell.attributes["value"].

    pub fn write(
        grid: &mut Grid,
        cube_id: CubeId,
        component: &Component,
        actor: &str,
    ) -> HypergridResult<()> {
        let id = component.id();

        // Every cell uses "value" as the attribute key (Hypergrid convention).
        macro_rules! w {
            ($field:expr, $val:expr) => {
                grid.write_cell(cube_id, cell_coord(id, $field), "value", $val, actor)?;
            };
        }

        let meta = &component.metadata;
        let data = &component.data;

        // ── Text cells ────────────────────────────────────────────────────
        w!(fields::NAME,         TypedAttrValue::Text(data.name.clone()));
        w!(fields::DESCRIPTION,  TypedAttrValue::Text(data.description.clone()));
        w!(fields::STATUS,       as_json(&data.status));
        w!(fields::STATE,        as_json(&data.state));
        w!(fields::VISIBILITY,   as_json(&data.visibility));
        w!(fields::VERSION,      TypedAttrValue::Text(meta.version.clone()));
        w!(fields::LAST_ACTOR,   TypedAttrValue::Text(meta.last_actor.clone()));

        // ── JSON cells ────────────────────────────────────────────────────
        w!(fields::CATEGORY,         as_json(&data.category));
        w!(fields::PAYLOAD,          as_json(&component.payload));
        w!(fields::OWNERS,           as_json(&meta.owners));
        w!(fields::TAGS,             as_json(&meta.tags));
        w!(fields::HASHTAGS,         as_json(&data.hashtags));
        w!(fields::TOPICS,           as_json(&data.topics));
        w!(fields::POLICY_IDS,       as_json(&meta.policy_ids));
        w!(fields::CHILDREN,         as_json(&data.children));
        w!(fields::PARENTS,          as_json(&data.parents));
        w!(fields::DEPENDENCIES,     as_json(&data.dependencies));
        w!(fields::DEPENDENTS,       as_json(&data.dependents));
        w!(fields::LINKS,            as_json(&data.links));
        w!(fields::TOOLBOX_IDS,      as_json(&data.toolbox_ids));
        w!(fields::RISKS,            as_json(&data.risks));
        w!(fields::USERS,            as_json(&data.users));
        w!(fields::ANALYTICS,        as_json(&data.analytics));
        w!(fields::ITEM_BOOK,        as_json(&component.item_book));
        w!(fields::CREATED_AT,       as_json(&meta.created_at));
        w!(fields::UPDATED_AT,       as_json(&meta.updated_at));
        w!(fields::VECTOR_CLOCK,     as_json(&meta.vector_clock));
        w!(fields::VERSION_HISTORY,  as_json(&meta.version_history));

        // ── Numeric cells ─────────────────────────────────────────────────
        w!(fields::BUDGET,         TypedAttrValue::Number(meta.budget));
        w!(fields::BUDGET_SPENT,   TypedAttrValue::Number(meta.budget_spent));
        w!(fields::RESOURCE_UNITS, TypedAttrValue::Number(meta.resource_units));

        Ok(())
    }

    // ── Read (HyperCells → Component) ─────────────────────────────────────
    //
    // Read all cells for D1=component_id, then reconstruct the Component.
    // Each cell at (component_id, field_name) has its value in attributes["value"].

    pub fn read(
        grid: &Grid,
        cube_id: CubeId,
        component_id: ComponentId,
    ) -> PortfolioResult<Option<Component>> {
        let d1 = DimKey::uuid(component_id);
        let cells = grid.get_row(cube_id, &d1);

        if cells.is_empty() {
            return Ok(None);
        }

        // Build a map from D2 field name → TypedAttrValue for fast lookup.
        let mut field_map: std::collections::HashMap<&str, &TypedAttrValue> =
            std::collections::HashMap::new();
        for cell in &cells {
            if let Some(DimKey::String(field)) = cell.coord.d2_key() {
                if let Some(val) = cell.value() {
                    field_map.insert(field.as_str(), val);
                }
            }
        }

        // Helper macros that look up a field by its D2 key.
        macro_rules! text {
            ($f:expr) => {
                field_map.get($f)
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_owned()
            };
        }
        macro_rules! json_of {
            ($f:expr) => {
                field_map.get($f)
                    .and_then(|v| if let TypedAttrValue::Json(j) = v { Some(j) } else { None })
                    .cloned()
                    .unwrap_or(JsonValue::Null)
            };
        }
        macro_rules! num {
            ($f:expr) => {
                field_map.get($f).and_then(|v| v.as_f64()).unwrap_or(0.0)
            };
        }
        macro_rules! from_json {
            ($f:expr, $T:ty) => {
                serde_json::from_value::<$T>(json_of!($f)).unwrap_or_default()
            };
        }

        use crate::component::{ComponentData};
        use crate::metadata::ComponentMetadata;
        use crate::types::*;
        use hypergrid::crdt::VectorClock;
        use std::collections::HashSet;

        let status: ComponentStatus = serde_json::from_value(json_of!(fields::STATUS))
            .unwrap_or(ComponentStatus::Draft);
        let state: ComponentState = serde_json::from_value(json_of!(fields::STATE))
            .unwrap_or(ComponentState::Initializing);
        let visibility: Visibility = serde_json::from_value(json_of!(fields::VISIBILITY))
            .unwrap_or(Visibility::Private);

        let category: ComponentCategory = serde_json::from_value(json_of!(fields::CATEGORY))
            .map_err(|e| PortfolioError::StorageError(
                format!("category: {e} (raw: {})", json_of!(fields::CATEGORY))
            ))?;

        let payload: ComponentPayload = serde_json::from_value(json_of!(fields::PAYLOAD))
            .map_err(|e| PortfolioError::StorageError(
                format!("payload: {e} (raw: {})", json_of!(fields::PAYLOAD))
            ))?;

        let vector_clock: VectorClock =
            serde_json::from_value(json_of!(fields::VECTOR_CLOCK)).unwrap_or_default();

        let version_history: Vec<crate::metadata::VersionHistoryEntry> =
            serde_json::from_value(json_of!(fields::VERSION_HISTORY)).unwrap_or_default();

        let created_at: chrono::DateTime<chrono::Utc> =
            serde_json::from_value(json_of!(fields::CREATED_AT))
            .unwrap_or_else(|_| chrono::Utc::now());
        let updated_at: chrono::DateTime<chrono::Utc> =
            serde_json::from_value(json_of!(fields::UPDATED_AT))
            .unwrap_or_else(|_| chrono::Utc::now());

        let metadata = ComponentMetadata {
            id: component_id,
            owners:          from_json!(fields::OWNERS,   Vec<UserId>),
            tags:            from_json!(fields::TAGS,     HashSet<String>),
            policy_ids:      from_json!(fields::POLICY_IDS, Vec<PolicyId>),
            created_at,
            updated_at,
            last_actor:      text!(fields::LAST_ACTOR),
            vector_clock,
            properties:      std::collections::HashMap::new(),
            version:         text!(fields::VERSION),
            version_history,
            budget:          num!(fields::BUDGET),
            budget_spent:    num!(fields::BUDGET_SPENT),
            resource_units:  num!(fields::RESOURCE_UNITS),
        };

        let data = ComponentData {
            category,
            name:         text!(fields::NAME),
            description:  text!(fields::DESCRIPTION),
            status,
            state,
            visibility,
            parents:      from_json!(fields::PARENTS,      Vec<ComponentId>),
            children:     from_json!(fields::CHILDREN,     Vec<ComponentId>),
            links:        from_json!(fields::LINKS,        Vec<ComponentId>),
            dependents:   from_json!(fields::DEPENDENTS,   Vec<ComponentId>),
            dependencies: from_json!(fields::DEPENDENCIES, Vec<ComponentId>),
            users:        serde_json::from_value(json_of!(fields::USERS))
                            .unwrap_or_default(),
            analytics:    serde_json::from_value(json_of!(fields::ANALYTICS))
                            .unwrap_or_default(),
            risks:        from_json!(fields::RISKS, Vec<crate::types::Risk>),
            hashtags:     from_json!(fields::HASHTAGS, HashSet<String>),
            topics:       from_json!(fields::TOPICS,   HashSet<String>),
            toolbox_ids:  from_json!(fields::TOOLBOX_IDS, Vec<ToolBoxId>),
            plugin_configs: std::collections::HashMap::new(),
        };

        let item_book = serde_json::from_value(json_of!(fields::ITEM_BOOK)).unwrap_or(None);

        Ok(Some(Component {
            metadata,
            data,
            payload,
            item_book,
            activity_log: vec![],
        }))
    }

    // ── Existence check ───────────────────────────────────────────────────

    pub fn exists(grid: &Grid, cube_id: CubeId, component_id: ComponentId) -> bool {
        !grid.get_row(cube_id, &DimKey::uuid(component_id)).is_empty()
    }
}

// ── Private helpers ────────────────────────────────────────────────────────────

fn cell_coord(component_id: ComponentId, field: &str) -> DimCoordinate {
    DimCoordinate::d2(DimKey::uuid(component_id), DimKey::str(field))
}

fn as_json<T: serde::Serialize>(val: &T) -> TypedAttrValue {
    TypedAttrValue::Json(serde_json::to_value(val).unwrap_or(JsonValue::Null))
}
