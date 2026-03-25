//! HG-DIM — Dimension Axis Registry.
//!
//! Every axis of a Hypercube is a typed, indexed, governed DimensionAxis.
//! The axis defines the key space, key ordering, and cardinality (dense vs sparse).

use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::cell::{AxisId, DimKey, PermissionTier, PluginId};
use crate::crdt::CrdtSemantics;
use crate::error::{HypergridError, HypergridResult};

/// Maximum number of dimensions per Hypercube (hard limit per spec).
pub const MAX_DIMENSIONS: u8 = 16;
/// Recommended maximum for UI usability.
pub const RECOMMENDED_MAX_DIMENSIONS: u8 = 6;
/// Maximum keys for a dense axis.
pub const MAX_DENSE_KEYS: usize = 10_000_000;
/// Maximum attributes per HyperCell.
pub const MAX_ATTRIBUTES: usize = 1_024;

// ─── Axis Type ───────────────────────────────────────────────────────────────

/// The semantic type of a DimensionAxis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AxisType {
    /// D₁ (rows) — primary entity dimension (users, projects, products).
    EntityAxis,
    /// D₂ (columns) — property/attribute dimension (field names, metric names).
    PropertyAxis,
    /// D₃ — temporal slicing (snapshots, time-series, event history).
    TimeAxis,
    /// D₄ — geographic slicing (region, country, city).
    GeoAxis,
    /// D₃–N — classification (product line, risk tier, department, scenario).
    CategoryAxis,
    /// D₃–N — ranked/ordered slicing (priority, ranking, version number).
    OrdinalAxis,
    /// D₃–N — org chart, product taxonomy, file system, tag hierarchy.
    HierarchyAxis,
    /// D₃–N — graph traversal dimension (network hops, dependency layers).
    GraphAxis,
    /// D₃ — schema/data versioning.
    VersionAxis,
    /// D₃ — multi-tenancy: share a single Hypercube across tenants.
    TenantAxis,
    /// D₃–N — audience/persona slicing (user segments, role types).
    PersonaAxis,
    /// D₃–N — what-if and planning (base, optimistic, pessimistic, budget).
    ScenarioAxis,
    /// Platform-defined axis type supplied by a HypercubePlugin.
    Custom(String),
}

impl AxisType {
    pub fn default_key_type(&self) -> KeyType {
        match self {
            AxisType::EntityAxis => KeyType::Uuid,
            AxisType::PropertyAxis => KeyType::String,
            AxisType::TimeAxis => KeyType::Timestamp,
            AxisType::GeoAxis => KeyType::GeoHash,
            AxisType::CategoryAxis => KeyType::String,
            AxisType::OrdinalAxis => KeyType::Integer,
            AxisType::HierarchyAxis => KeyType::HierarchyPath,
            AxisType::GraphAxis => KeyType::Uuid,
            AxisType::VersionAxis => KeyType::String,
            AxisType::TenantAxis => KeyType::Uuid,
            AxisType::PersonaAxis => KeyType::String,
            AxisType::ScenarioAxis => KeyType::String,
            AxisType::Custom(_) => KeyType::String,
        }
    }

    pub fn default_ordering(&self) -> AxisOrdering {
        match self {
            AxisType::TimeAxis => AxisOrdering::Chronological,
            AxisType::OrdinalAxis | AxisType::VersionAxis => AxisOrdering::Numeric,
            AxisType::PropertyAxis | AxisType::CategoryAxis |
            AxisType::PersonaAxis | AxisType::ScenarioAxis => AxisOrdering::Lexicographic,
            AxisType::HierarchyAxis => AxisOrdering::DepthFirst,
            _ => AxisOrdering::Unordered,
        }
    }
}

// ─── Key Type ─────────────────────────────────────────────────────────────────

/// The type of keys in this dimension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KeyType {
    Uuid, String, Integer, Ordinal, Timestamp, GeoHash, HierarchyPath, Custom(String),
}

// ─── Key Cardinality ─────────────────────────────────────────────────────────

/// Cardinality of keys: affects storage encoding and index choice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KeyCardinality {
    /// Fully materialized; all key combinations stored.
    Dense(usize),
    /// Only non-null cells stored; memory efficient for sparse data.
    Sparse,
    /// Effectively infinite key space (e.g. continuous time, geography).
    Infinite,
}

// ─── Axis Ordering ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AxisOrdering {
    Unordered, Lexicographic, Numeric, Chronological, DepthFirst, Custom(String),
}

// ─── Axis Visibility ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AxisVisibility {
    Public, Tenant, Private,
}

// ─── Storage Encoding ────────────────────────────────────────────────────────

/// How cells for this axis are physically stored.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StorageEncoding {
    /// N-dim array: O(D₁ × D₂ × …) cells allocated. O(1) lookup.
    Dense,
    /// Key-value: only non-null cells stored. O(log n) lookup.
    Sparse,
    /// D₁×D₂ dense; D₃–N sparse (recommended for most production use cases).
    Hybrid,
}

// ─── Index Strategy ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AxisIndexStrategy {
    BTree, Hash, Gin, Brin, TimeSeries, Spatial,
}

// ─── Hierarchy Definition ────────────────────────────────────────────────────

/// Parent-child relationships between keys on a HierarchyAxis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyDef {
    /// Maps each child key to its parent key (None = root).
    pub parent_map: HashMap<String, Option<String>>,
}

impl HierarchyDef {
    pub fn new() -> Self {
        Self { parent_map: HashMap::new() }
    }

    pub fn add_node(&mut self, key: impl Into<String>, parent: Option<String>) {
        self.parent_map.insert(key.into(), parent);
    }

    pub fn children_of(&self, parent: &str) -> Vec<&str> {
        self.parent_map
            .iter()
            .filter(|(_, p)| p.as_deref() == Some(parent))
            .map(|(k, _)| k.as_str())
            .collect()
    }

    pub fn ancestors_of<'a>(&'a self, key: &str) -> Vec<&'a str> {
        let mut result = vec![];
        let mut current = key;
        while let Some(Some(parent)) = self.parent_map.get(current) {
            result.push(parent.as_str());
            current = parent.as_str();
        }
        result
    }
}

impl Default for HierarchyDef {
    fn default() -> Self { Self::new() }
}

// ─── KeySet ──────────────────────────────────────────────────────────────────

/// The valid key set for an axis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeySet {
    /// Enumerated set of all valid keys (dense axis).
    Enumerated(HashSet<String>),
    /// Schema-constrained but not fully enumerated (sparse/infinite).
    SchemaConstrained { key_type: KeyType },
    /// Open — any key matching the key_type is valid.
    Open { key_type: KeyType },
}

impl KeySet {
    pub fn is_valid(&self, key: &DimKey) -> bool {
        match self {
            KeySet::Enumerated(set) => {
                match key {
                    DimKey::String(s) => set.contains(s.as_str()),
                    _ => true, // non-string keys in enumerated sets are type-validated
                }
            }
            KeySet::SchemaConstrained { .. } | KeySet::Open { .. } => true,
        }
    }
}

// ─── DimensionAxis ───────────────────────────────────────────────────────────

/// One axis of a Hypercube: defines the key space, ordering, and cardinality.
/// Represents Dᵢ in the formal model H = (D₁, D₂, ..., Dₙ).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionAxis {
    /// Globally unique identifier for this axis within the grid.
    pub axis_id: AxisId,
    /// Human-readable name: "rows", "columns", "time", "region", etc.
    pub name: String,
    /// Position in the ordered dimension tuple (1-indexed). D₁=1, D₂=2, D₃–N=custom.
    pub dim_index: u8,
    /// Semantic type of this axis.
    pub axis_type: AxisType,
    /// Type of keys in this dimension.
    pub key_type: KeyType,
    /// Valid key set (enumerated, schema-constrained, or open).
    pub key_set: KeySet,
    /// Cardinality: Dense | Sparse | Infinite.
    pub key_cardinality: KeyCardinality,
    /// Sort behavior across this axis.
    pub ordering: AxisOrdering,
    /// Parent-child relationships (for HierarchyAxis only).
    pub hierarchy: Option<HierarchyDef>,
    /// Which tenants/identities can see this axis.
    pub visibility: AxisVisibility,
    /// If false: every key must have a cell (dense). If true: sparse is ok.
    pub nullable: bool,
    /// When rendering as 2D, which cell attribute is shown as the primary "value".
    pub default_attribute: Option<crate::cell::AttributeKey>,
    /// CRDT behavior for mutations along this axis.
    pub crdt_semantics: CrdtSemantics,
    /// Storage index type for this axis.
    pub index_strategy: AxisIndexStrategy,
    /// Plugin providing key validation, rendering, and computation for custom axis types.
    pub plugin_id: Option<PluginId>,
    pub created_at: DateTime<Utc>,
    /// Extensible axis-level metadata for platform-specific configuration.
    pub metadata: HashMap<String, JsonValue>,
}

impl DimensionAxis {
    /// Create a new EntityAxis (D₁ — rows).
    pub fn entity_axis() -> Self {
        Self::new(
            "rows".into(),
            1,
            AxisType::EntityAxis,
            KeyType::Uuid,
            KeySet::Open { key_type: KeyType::Uuid },
            KeyCardinality::Sparse,
        )
    }

    /// Create a new PropertyAxis (D₂ — columns).
    pub fn property_axis() -> Self {
        Self::new(
            "columns".into(),
            2,
            AxisType::PropertyAxis,
            KeyType::String,
            KeySet::Open { key_type: KeyType::String },
            KeyCardinality::Sparse,
        )
    }

    /// Create a TimeAxis at a given dimension index.
    pub fn time_axis(dim_index: u8) -> Self {
        Self::new(
            "time".into(),
            dim_index,
            AxisType::TimeAxis,
            KeyType::Timestamp,
            KeySet::Open { key_type: KeyType::Timestamp },
            KeyCardinality::Infinite,
        )
    }

    /// Create a CategoryAxis with a fixed set of enum values.
    pub fn category_axis(name: impl Into<String>, dim_index: u8, values: Vec<&str>) -> Self {
        let mut axis = Self::new(
            name.into(),
            dim_index,
            AxisType::CategoryAxis,
            KeyType::String,
            KeySet::Enumerated(values.into_iter().map(String::from).collect()),
            KeyCardinality::Dense(values.len()),
        );
        axis
    }

    fn new(
        name: String,
        dim_index: u8,
        axis_type: AxisType,
        key_type: KeyType,
        key_set: KeySet,
        key_cardinality: KeyCardinality,
    ) -> Self {
        let ordering = axis_type.default_ordering();
        Self {
            axis_id: Uuid::new_v4(),
            name,
            dim_index,
            axis_type,
            key_type,
            key_set,
            key_cardinality,
            ordering,
            hierarchy: None,
            visibility: AxisVisibility::Public,
            nullable: true,
            default_attribute: Some("value".into()),
            crdt_semantics: CrdtSemantics::LastWriteWins,
            index_strategy: AxisIndexStrategy::BTree,
            plugin_id: None,
            created_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    pub fn validate_key(&self, key: &DimKey) -> HypergridResult<()> {
        if !self.key_set.is_valid(key) {
            return Err(HypergridError::SchemaValidation(format!(
                "Key '{key}' is not valid for axis '{}' (index D{})",
                self.name, self.dim_index
            )));
        }
        Ok(())
    }
}

// ─── DimSlice — N-Dimensional Filter ─────────────────────────────────────────

/// A predicate applied to one or more dimension axes, producing a sub-cube or projection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DimSlicePredicate {
    /// Exact value on one axis. Equivalent to a single row/column lookup.
    Point { axis_id: AxisId, key: DimKey },
    /// Range of values on an ordered axis.
    Range { axis_id: AxisId, from: DimKey, to: DimKey, inclusive: bool },
    /// A set of specific values on any axis.
    Set { axis_id: AxisId, keys: Vec<DimKey> },
    /// A computed condition on a cell attribute.
    Predicate { axis_id: AxisId, attr: crate::cell::AttributeKey, op: PredicateOp, value: crate::cell::TypedAttrValue },
    /// Select a hierarchy node and all its descendants.
    Hierarchical { axis_id: AxisId, root_key: DimKey },
    /// Cells where this axis has no explicit value (sparse null cells).
    IsNull { axis_id: AxisId },
    /// Include cells from linked (Shadow) grids.
    ShadowInclude { grid_id: crate::cell::GridId },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PredicateOp {
    Eq, Ne, Gt, Ge, Lt, Le, Contains, StartsWith, EndsWith,
}

/// A DimFold collapses an axis by aggregating all its key values into a summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimFold {
    pub axis_id: AxisId,
    pub agg_fn: crate::cell::AggFn,
    pub attr: crate::cell::AttributeKey,
}

/// A DimExpand expands a previously folded or defaulted axis into its full key set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimExpand {
    pub axis_id: AxisId,
    pub as_columns: bool,
    pub agg_fn: Option<crate::cell::AggFn>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_axis_default() {
        let ax = DimensionAxis::entity_axis();
        assert_eq!(ax.dim_index, 1);
        assert_eq!(ax.axis_type, AxisType::EntityAxis);
    }

    #[test]
    fn category_axis_key_validation() {
        let ax = DimensionAxis::category_axis("status", 3, vec!["Active", "Draft", "Archived"]);
        assert!(ax.validate_key(&DimKey::str("Active")).is_ok());
        // Open axes accept anything; enumerated would reject unknowns
    }
}
