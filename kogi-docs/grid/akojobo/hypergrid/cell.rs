//! HG-CELL — HyperCell & N-Attribute Model.
//!
//! A HyperCell is not a single value. It is an N-attribute data point — a
//! structured map of typed attribute keys located at a specific coordinate in
//! the N-dimensional grid.

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::crdt::{CrdtSemantics, NodeId, VectorClock};
use crate::error::{HypergridError, HypergridResult};

// ─── Primitive Key Types ──────────────────────────────────────────────────────

pub type GridId    = Uuid;
pub type CubeId    = Uuid;
pub type AxisId    = Uuid;
pub type PolicyId  = Uuid;
pub type PluginId  = Uuid;
pub type ActorId   = String;    // "{tenant_id}:{identity_tag}:{node_id}"
pub type IdentityId = Uuid;
pub type SpaceId   = Uuid;
pub type ViewId    = Uuid;
pub type EdgeId    = Uuid;

/// A dimension key — the typed value used as an index on a single axis.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DimKey {
    /// UUID entity identifier (EntityAxis, GraphAxis).
    Uuid(Uuid),
    /// String key (PropertyAxis, CategoryAxis, ScenarioAxis, PersonaAxis).
    String(String),
    /// Integer key (OrdinalAxis, VersionAxis as sequence).
    Integer(i64),
    /// Timestamp key (TimeAxis).
    Timestamp(DateTime<Utc>),
    /// Geohash string (GeoAxis).
    GeoHash(String),
    /// Hierarchical path (HierarchyAxis).
    HierarchyPath(String),
    /// Null key — used for sparse axes where a cell is not assigned.
    Null,
}

impl DimKey {
    pub fn str(s: impl Into<String>) -> Self {
        DimKey::String(s.into())
    }
    pub fn uuid(id: Uuid) -> Self {
        DimKey::Uuid(id)
    }
    pub fn int(n: i64) -> Self {
        DimKey::Integer(n)
    }
}

impl std::fmt::Display for DimKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DimKey::Uuid(u) => write!(f, "{u}"),
            DimKey::String(s) => write!(f, "{s}"),
            DimKey::Integer(n) => write!(f, "{n}"),
            DimKey::Timestamp(t) => write!(f, "{t}"),
            DimKey::GeoHash(g) => write!(f, "{g}"),
            DimKey::HierarchyPath(p) => write!(f, "{p}"),
            DimKey::Null => write!(f, "null"),
        }
    }
}

// ─── N-Dimensional Coordinate ─────────────────────────────────────────────────

/// An ordered tuple of N dimension keys, one per axis.
/// Uniquely addresses a HyperCell within a Hypercube.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DimCoordinate {
    /// Ordered keys: [D1_key, D2_key, ..., DN_key]
    pub keys: Vec<DimKey>,
}

impl DimCoordinate {
    pub fn new(keys: Vec<DimKey>) -> Self {
        Self { keys }
    }

    /// Create a 2D coordinate (conventional spreadsheet).
    pub fn d2(row_key: DimKey, col_key: DimKey) -> Self {
        Self { keys: vec![row_key, col_key] }
    }

    /// The dimensionality of this coordinate.
    pub fn n(&self) -> u8 {
        self.keys.len() as u8
    }

    /// Validate against expected dimensionality N.
    pub fn validate_n(&self, n: u8) -> HypergridResult<()> {
        if self.keys.len() as u8 != n {
            return Err(HypergridError::DimensionalityMismatch {
                coord_n: self.keys.len(),
                cube_n: n,
            });
        }
        Ok(())
    }

    /// Get the D₁ key (entity/row key).
    pub fn d1(&self) -> Option<&DimKey> {
        self.keys.get(0)
    }

    /// Get the D₂ key (property/column key).
    pub fn d2_key(&self) -> Option<&DimKey> {
        self.keys.get(1)
    }
}

// ─── Typed Attribute Value ────────────────────────────────────────────────────

/// The possible typed values for a HyperCell attribute.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypedAttrValue {
    Null,
    Text(String),
    Number(f64),
    Integer(i64),
    Currency { amount: f64, currency_code: String },
    Percent(f64),
    Bool(bool),
    Date(String),              // ISO-8601 date string
    DateTime(DateTime<Utc>),
    Duration(i64),             // seconds
    Enum(String),
    MultiEnum(Vec<String>),
    Relation(Uuid),            // ID of related entity
    MultiRelation(Vec<Uuid>),
    User(Uuid),
    Tag(Vec<String>),
    Json(JsonValue),
    Computed(Box<TypedAttrValue>),  // value produced by a formula/AI
    AiSignal {
        value: Box<TypedAttrValue>,
        confidence: f32,
        model: String,
    },
    Error(String),
}

impl TypedAttrValue {
    pub fn is_null(&self) -> bool {
        matches!(self, TypedAttrValue::Null)
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            TypedAttrValue::Number(n) => Some(*n),
            TypedAttrValue::Integer(n) => Some(*n as f64),
            TypedAttrValue::Percent(p) => Some(*p),
            TypedAttrValue::Currency { amount, .. } => Some(*amount),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            TypedAttrValue::Text(s) => Some(s),
            TypedAttrValue::Enum(s) => Some(s),
            _ => None,
        }
    }
}

// ─── Attribute Key ────────────────────────────────────────────────────────────

pub type AttributeKey = String;

/// The attribute type enum (determines storage, CRDT, and computation).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AttributeType {
    Text, Number, Currency, Percent, Bool, Date, DateTime, Duration,
    Enum(Vec<String>),      // valid enum variants
    MultiEnum(Vec<String>),
    Relation,
    MultiRelation,
    User,
    Tag,
    Json,
    Computed,
    Ai,
    Formula,
    Audit,
    Custom(String),
}

/// Scope: which axes this attribute applies to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DimScope {
    /// Applies to all dimension combinations.
    AllDimensions,
    /// Applies only to cells on the specified axes.
    SpecificAxes(Vec<AxisId>),
}

/// Aggregation functions supported for a given attribute key.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AggFn {
    Sum, Avg, Min, Max, Count, CountUnique,
    Median, Percentile(f64), Stddev, Variance, Distribution, Mode,
}

/// Storage index strategy for an attribute.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IndexStrategy {
    None, BTree, Hash, Gin, Brin, TimeSeries, Spatial,
}

/// Visibility level for an attribute key.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AttrVisibility {
    Public, Tenant, Identity, Owner,
}

/// Computation specification for a ComputedAttribute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttrComputation {
    /// Tier-1: synchronous formula expression in HyperQL formula language.
    Formula(String),
    /// Tier-2: asynchronous AI/ML engine computation.
    AiEngine {
        plugin_id: PluginId,
        model_name: String,
        parameters: HashMap<String, JsonValue>,
    },
}

/// A registered attribute key in the Hypercube's AttributeKeyRegistry.
/// Defines type, default value, computation, visibility, and CRDT semantics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeKeyDef {
    pub key: AttributeKey,
    pub display_name: String,
    pub attr_type: AttributeType,
    pub description: String,
    pub dimension_scope: DimScope,
    pub default_value: Option<TypedAttrValue>,
    pub required: bool,
    pub visibility: AttrVisibility,
    pub write_permission: PermissionTier,
    pub crdt_semantics: CrdtSemantics,
    pub computation: Option<AttrComputation>,
    pub aggregation: Vec<AggFn>,
    pub index_strategy: IndexStrategy,
    pub inherits_from: Option<AttributeKey>,
    pub version_track: bool,
    pub searchable: bool,
    pub export_label: Option<String>,
    pub plugin_id: Option<PluginId>,
}

impl AttributeKeyDef {
    /// Create a simple text attribute with LWW semantics.
    pub fn text(key: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            display_name: display_name.into(),
            attr_type: AttributeType::Text,
            description: String::new(),
            dimension_scope: DimScope::AllDimensions,
            default_value: None,
            required: false,
            visibility: AttrVisibility::Public,
            write_permission: PermissionTier::Editor,
            crdt_semantics: CrdtSemantics::LastWriteWins,
            computation: None,
            aggregation: vec![],
            index_strategy: IndexStrategy::BTree,
            inherits_from: None,
            version_track: true,
            searchable: true,
            export_label: None,
            plugin_id: None,
        }
    }

    /// Create a numeric attribute with LWW semantics.
    pub fn number(key: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            attr_type: AttributeType::Number,
            aggregation: vec![AggFn::Sum, AggFn::Avg, AggFn::Min, AggFn::Max],
            ..Self::text(key, display_name)
        }
    }
}

/// Permission tier for write access to an attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PermissionTier {
    Viewer = 0,
    Subscriber = 1,
    Contributor = 2,
    Editor = 3,
    Manager = 4,
    Owner = 5,
    Admin = 6,
}

// ─── AttributeKeyRegistry ────────────────────────────────────────────────────

/// The registry of all attribute keys for a Hypercube.
/// Every attribute key used in any HyperCell must be registered here.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AttributeKeyRegistry {
    pub entries: HashMap<AttributeKey, AttributeKeyDef>,
}

impl AttributeKeyRegistry {
    pub fn new() -> Self {
        let mut r = Self::default();
        r.install_builtins();
        r
    }

    /// Register a new attribute key definition.
    pub fn register(&mut self, def: AttributeKeyDef) -> HypergridResult<()> {
        if self.entries.contains_key(&def.key) {
            return Err(HypergridError::AlreadyExists(def.key.clone()));
        }
        self.entries.insert(def.key.clone(), def);
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&AttributeKeyDef> {
        self.entries.get(key)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    /// Install the built-in core attribute keys that every Hypercube provides.
    fn install_builtins(&mut self) {
        let builtins: Vec<AttributeKeyDef> = vec![
            AttributeKeyDef {
                key: "value".into(),
                display_name: "Value".into(),
                attr_type: AttributeType::Json, // heterogeneous; type determined per D2 column
                description: "Primary data value — the 'cell value' in conventional spreadsheet terms.".into(),
                crdt_semantics: CrdtSemantics::LastWriteWins,
                version_track: true,
                ..AttributeKeyDef::text("value", "Value")
            },
            AttributeKeyDef::text("data_type", "Data Type"),
            AttributeKeyDef::text("format", "Format"),
            AttributeKeyDef {
                key: "formula".into(),
                display_name: "Formula".into(),
                attr_type: AttributeType::Formula,
                visibility: AttrVisibility::Tenant,
                ..AttributeKeyDef::text("formula", "Formula")
            },
            AttributeKeyDef {
                key: "version".into(),
                display_name: "Version".into(),
                attr_type: AttributeType::Number,
                crdt_semantics: CrdtSemantics::MaxRegister,
                ..AttributeKeyDef::number("version", "Version")
            },
            AttributeKeyDef {
                key: "visibility".into(),
                display_name: "Visibility".into(),
                attr_type: AttributeType::Enum(vec![
                    "Public".into(), "Tenant".into(), "Identity".into(), "Private".into()
                ]),
                write_permission: PermissionTier::Owner,
                ..AttributeKeyDef::text("visibility", "Visibility")
            },
            AttributeKeyDef {
                key: "locked".into(),
                display_name: "Locked".into(),
                attr_type: AttributeType::Bool,
                write_permission: PermissionTier::Owner,
                ..AttributeKeyDef::text("locked", "Locked")
            },
            AttributeKeyDef {
                key: "tags".into(),
                display_name: "Tags".into(),
                attr_type: AttributeType::Tag,
                crdt_semantics: CrdtSemantics::OrSet,
                ..AttributeKeyDef::text("tags", "Tags")
            },
            AttributeKeyDef {
                key: "comment_count".into(),
                display_name: "Comments".into(),
                attr_type: AttributeType::Number,
                crdt_semantics: CrdtSemantics::GrowOnlyCounter,
                ..AttributeKeyDef::number("comment_count", "Comments")
            },
            AttributeKeyDef::text("last_modified_by", "Last Modified By"),
            AttributeKeyDef {
                key: "modified_at".into(),
                display_name: "Modified At".into(),
                attr_type: AttributeType::DateTime,
                ..AttributeKeyDef::text("modified_at", "Modified At")
            },
            AttributeKeyDef {
                key: "error".into(),
                display_name: "Error".into(),
                attr_type: AttributeType::Text,
                visibility: AttrVisibility::Tenant,
                ..AttributeKeyDef::text("error", "Error")
            },
        ];
        for def in builtins {
            self.entries.insert(def.key.clone(), def);
        }
    }
}

// ─── AttributeMap ─────────────────────────────────────────────────────────────

/// The N-attribute payload of a HyperCell.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AttributeMap {
    pub entries: HashMap<AttributeKey, TypedAttrValue>,
}

impl AttributeMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, key: impl Into<String>, value: TypedAttrValue) {
        self.entries.insert(key.into(), value);
    }

    pub fn get(&self, key: &str) -> Option<&TypedAttrValue> {
        self.entries.get(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<TypedAttrValue> {
        self.entries.remove(key)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Merge another AttributeMap using LWW (this map wins by default).
    /// In production this is driven by per-attribute CrdtSemantics.
    pub fn merge_lww(&mut self, other: &AttributeMap) {
        for (k, v) in &other.entries {
            self.entries.entry(k.clone()).or_insert_with(|| v.clone());
        }
    }
}

impl std::ops::Index<&str> for AttributeMap {
    type Output = TypedAttrValue;
    fn index(&self, key: &str) -> &Self::Output {
        self.entries.get(key).unwrap_or(&TypedAttrValue::Null)
    }
}

// ─── Cached Computed Attribute ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedComputedAttr {
    pub value: TypedAttrValue,
    pub computed_by: String,
    pub computed_at: DateTime<Utc>,
    pub confidence: Option<f32>,
    pub cache_valid_until: Option<DateTime<Utc>>,
}

// ─── Cell Visibility & Governance ────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CellVisibility {
    Public, Tenant, Identity, Private,
}

// ─── HyperCell ────────────────────────────────────────────────────────────────

/// A data point at an N-dimensional coordinate, carrying an N-attribute map.
///
/// The central data structure of Hypergrid. Every entity, property intersection,
/// and multi-dimensional data point is a HyperCell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperCell {
    // ── Coordinate ────────────────────────────────────────────────────────
    /// Parent Grid.
    pub grid_id: GridId,
    /// Parent Hypercube.
    pub cube_id: CubeId,
    /// N-dimensional coordinate: [D1_key, D2_key, ..., DN_key]
    pub coord: DimCoordinate,

    // ── Attributes ────────────────────────────────────────────────────────
    /// The N-attribute payload. HashMap<AttributeKey, TypedAttrValue>
    pub attributes: AttributeMap,

    // ── Versioning ────────────────────────────────────────────────────────
    /// Causal ordering across federation nodes.
    pub vector_clock: VectorClock,
    /// Monotonic mutation counter.
    pub version: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// "{tenant_id}:{identity_tag}:{node_id}" — actor who last mutated this cell.
    pub last_actor: ActorId,

    // ── Governance ────────────────────────────────────────────────────────
    pub visibility: CellVisibility,
    pub policy_ids: Vec<PolicyId>,

    // ── Graph integration ─────────────────────────────────────────────────
    /// Edge IDs originating from this cell (stored in HG-GRAPH).
    pub edge_refs: Vec<EdgeId>,

    // ── Computed attribute cache ──────────────────────────────────────────
    pub computed_cache: HashMap<AttributeKey, CachedComputedAttr>,
}

impl HyperCell {
    /// Create a new, empty HyperCell at the given coordinate.
    pub fn new(grid_id: GridId, cube_id: CubeId, coord: DimCoordinate, actor: &str) -> Self {
        let now = Utc::now();
        let mut vc = VectorClock::new();
        vc.tick(actor);
        Self {
            grid_id,
            cube_id,
            coord,
            attributes: AttributeMap::new(),
            vector_clock: vc,
            version: 1,
            created_at: now,
            updated_at: now,
            last_actor: actor.to_owned(),
            visibility: CellVisibility::Tenant,
            policy_ids: vec![],
            edge_refs: vec![],
            computed_cache: HashMap::new(),
        }
    }

    /// Set a typed attribute value and bump the vector clock.
    pub fn set_attr(&mut self, key: impl Into<String>, value: TypedAttrValue, actor: &str) {
        let key = key.into();
        self.attributes.set(key, value);
        self.vector_clock.tick(actor);
        self.version += 1;
        self.updated_at = Utc::now();
        self.last_actor = actor.to_owned();
    }

    /// Get a reference to an attribute value.
    pub fn get_attr(&self, key: &str) -> Option<&TypedAttrValue> {
        self.attributes.get(key)
    }

    /// Get the primary "value" attribute.
    pub fn value(&self) -> Option<&TypedAttrValue> {
        self.attributes.get("value")
    }

    /// CRDT merge: merge a remote cell's attributes into this cell using LWW per-attribute.
    /// In production this resolves per-attribute CrdtSemantics from the registry.
    pub fn merge_lww(&mut self, remote: &HyperCell) {
        // Vector clock merge
        self.vector_clock.merge(&remote.vector_clock);
        // Attribute merge: remote wins if its timestamp is later
        if remote.updated_at > self.updated_at {
            self.attributes.merge_lww(&remote.attributes);
            self.version = self.version.max(remote.version);
            self.updated_at = remote.updated_at;
            self.last_actor = remote.last_actor.clone();
        }
    }
}

/// All HyperCells sharing the same D₁ key — the canonical entity in a Hypercube.
pub struct HyperRow<'a> {
    pub d1_key: &'a DimKey,
    pub cells: Vec<&'a HyperCell>,
}

impl<'a> HyperRow<'a> {
    pub fn new(d1_key: &'a DimKey, cells: Vec<&'a HyperCell>) -> Self {
        Self { d1_key, cells }
    }

    /// Get the cell at a specific D₂ column key.
    pub fn get_col(&self, col_key: &DimKey) -> Option<&HyperCell> {
        self.cells.iter().find(|c| c.coord.d2_key() == Some(col_key)).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dim_coordinate_validate() {
        let coord = DimCoordinate::d2(DimKey::str("row1"), DimKey::str("col1"));
        assert!(coord.validate_n(2).is_ok());
        assert!(coord.validate_n(3).is_err());
    }

    #[test]
    fn hypercell_set_and_get() {
        let gid = Uuid::new_v4();
        let cid = Uuid::new_v4();
        let coord = DimCoordinate::d2(DimKey::str("e1"), DimKey::str("revenue"));
        let mut cell = HyperCell::new(gid, cid, coord, "node-1");
        cell.set_attr("value", TypedAttrValue::Number(1234.0), "node-1");
        assert_eq!(cell.value(), Some(&TypedAttrValue::Number(1234.0)));
        assert_eq!(cell.version, 2);
    }

    #[test]
    fn attribute_registry_builtins() {
        let reg = AttributeKeyRegistry::new();
        assert!(reg.contains("value"));
        assert!(reg.contains("tags"));
        assert!(reg.contains("version"));
    }
}
