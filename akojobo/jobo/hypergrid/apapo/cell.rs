// =============================================================================
// hypergrid::cell — HG-CELL: Core Data Types
//
// TypedAttrValue, DimKey, DimCoordinate, HyperCell, HyperRow,
// AttributeKeyDef, AttributeKeyRegistry, PermissionTier, AttrVisibility
// =============================================================================

use std::collections::{HashMap, HashSet};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::crdt::CrdtSemantics;
use crate::error::HypergridError;

// ─── Type Aliases ─────────────────────────────────────────────────────────────

pub type GridId     = Uuid;
pub type CubeId     = Uuid;
pub type AxisId     = Uuid;
pub type NodeId     = String;   // "{region}:{instance_id}" e.g. "node-us-east-1:@alice"
pub type ActorId    = String;   // "{user_uuid}:{identity_tag}:{node_id}"
pub type AttributeKey = String;
pub type EntityId   = Uuid;
pub type PluginId   = String;
pub type PolicyId   = Uuid;
pub type SpaceId    = Uuid;
pub type UserId     = Uuid;

// ─── PermissionTier ──────────────────────────────────────────────────────────

/// Minimum tier required to write a given attribute key, or to perform an operation.
/// Ordered numerically: higher values require greater privilege.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum PermissionTier {
    Public      = 0,  // Unauthenticated read (no write allowed at this tier)
    Viewer      = 1,  // Authenticated read-only
    Subscriber  = 2,  // Follow/subscribe; no write
    Contributor = 3,  // Write content fields on assigned entities
    Editor      = 4,  // Write all content + config fields
    Manager     = 5,  // Write governance, member roster, budgets
    Owner       = 6,  // Full ownership: archive, delete, transfer
    Admin       = 7,  // Platform admin: override lattice, force transitions
    System      = 8,  // Internal system / AI engine writes only
}

impl Default for PermissionTier {
    fn default() -> Self { Self::Editor }
}

// ─── AttrVisibility ──────────────────────────────────────────────────────────

/// Who can READ a given attribute value. Write permission is controlled by PermissionTier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttrVisibility {
    Public,   // Readable by any unauthenticated caller
    Tenant,   // Readable within the same organizational tenant
    Identity, // Readable only by the owning identity
    Owner,    // Readable only by the entity owner(s)
}

impl Default for AttrVisibility { fn default() -> Self { Self::Tenant } }

// ─── AnomalyKind ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnomalyKind {
    SuddenChange,   // z > 3σ in a single observation
    PatternBreak,   // sustained deviation over 3+ consecutive observations
    OutlierValue,   // single extreme point surrounded by normal values
    StateChange,    // unexpected categorical transition
}

// ─── TypedAttrValue ──────────────────────────────────────────────────────────

/// The complete union of all value types a HyperCell attribute may carry.
/// Every field of every domain entity is stored as one of these variants.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum TypedAttrValue {
    // ── Primitive scalars ───────────────────────────────────────────────────
    Text(String),
    Number(f64),
    Integer(i64),
    Bool(bool),
    Date(NaiveDate),
    DateTime(DateTime<Utc>),
    Duration { seconds: i64 },

    // ── Financial ──────────────────────────────────────────────────────────
    Currency { amount: String, currency_code: String }, // amount as Decimal string

    // ── Enumerated ─────────────────────────────────────────────────────────
    Enum { variant: String, enum_type: String },
    MultiEnum(Vec<String>),

    // ── Relations ──────────────────────────────────────────────────────────
    Relation(EntityRef),
    MultiRelation(Vec<EntityRef>),
    User(UserId),
    MultiUser(Vec<UserId>),

    // ── Tags / Labels ─────────────────────────────────────────────────────
    Tag(String),
    TagSet(HashSet<String>),

    // ── Structured data ────────────────────────────────────────────────────
    Json(serde_json::Value),

    // ── Computed: Tier-1 formula ───────────────────────────────────────────
    Formula {
        expression: String,        // human-readable formula expression
        result:     Box<TypedAttrValue>,
        cached_at:  DateTime<Utc>,
    },

    // ── Computed: Tier-2 AI signal ─────────────────────────────────────────
    AiSignal {
        value:       Box<TypedAttrValue>,
        model_id:    String,       // e.g. "kogi.health_score.v2"
        computed_at: DateTime<Utc>,
        confidence:  f64,          // 0.0 – 1.0
        explanation: Option<String>,
        stale_at:    Option<DateTime<Utc>>, // set when TTL has expired but value not yet refreshed
    },

    // ── Anomaly signal (from AnomalyEngine) ───────────────────────────────
    AnomalySignal {
        kind:      AnomalyKind,
        severity:  f64,           // z-score / threshold; 1.0 = threshold, >1 = worse
        baseline:  Box<TypedAttrValue>,
        deviation: f64,
    },

    // ── Cross-grid reference ───────────────────────────────────────────────
    ShadowRef {
        source_grid:  GridId,
        source_coord: SerializableDimCoordinate,
    },

    // ── Append-only log entry ─────────────────────────────────────────────
    LogEntry {
        seq:       u64,
        entry:     Box<TypedAttrValue>,
        actor:     ActorId,
        timestamp: DateTime<Utc>,
    },

    // ── Rich text ─────────────────────────────────────────────────────────
    RichText { markdown: String, plain: String },

    // ── Attachment reference ──────────────────────────────────────────────
    Attachment { url: String, mime_type: String, size_bytes: u64 },

    // ── Null / empty ──────────────────────────────────────────────────────
    Null,
}

impl TypedAttrValue {
    /// Return as f64 if this value is numeric (Number, Integer, Currency amount).
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Number(n)    => Some(*n),
            Self::Integer(i)   => Some(*i as f64),
            Self::Currency { amount, .. } => amount.parse().ok(),
            Self::AiSignal { value, .. } => value.as_f64(),
            Self::Formula { result, .. }  => result.as_f64(),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Text(s)  => Some(s.as_str()),
            Self::Enum { variant, .. } => Some(variant.as_str()),
            _ => None,
        }
    }

    pub fn as_json(&self) -> Option<&serde_json::Value> {
        match self {
            Self::Json(v) => Some(v),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool { matches!(self, Self::Null) }

    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Text(_)         => "Text",
            Self::Number(_)       => "Number",
            Self::Integer(_)      => "Integer",
            Self::Bool(_)         => "Bool",
            Self::Date(_)         => "Date",
            Self::DateTime(_)     => "DateTime",
            Self::Duration { .. } => "Duration",
            Self::Currency { .. } => "Currency",
            Self::Enum { .. }     => "Enum",
            Self::MultiEnum(_)    => "MultiEnum",
            Self::Relation(_)     => "Relation",
            Self::MultiRelation(_)=> "MultiRelation",
            Self::User(_)         => "User",
            Self::MultiUser(_)    => "MultiUser",
            Self::Tag(_)          => "Tag",
            Self::TagSet(_)       => "TagSet",
            Self::Json(_)         => "Json",
            Self::Formula { .. }  => "Formula",
            Self::AiSignal { .. } => "AiSignal",
            Self::AnomalySignal {..} => "AnomalySignal",
            Self::ShadowRef { .. }=> "ShadowRef",
            Self::LogEntry { .. } => "LogEntry",
            Self::RichText { .. } => "RichText",
            Self::Attachment {..} => "Attachment",
            Self::Null            => "Null",
        }
    }
}

// ─── EntityRef ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityRef {
    pub cube_id:  CubeId,
    pub d1_key:   EntityId,
    pub grid_id:  Option<GridId>, // None = same grid
}

// ─── DimKey ──────────────────────────────────────────────────────────────────

/// A typed key value for one dimension axis of a DimCoordinate.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "v")]
pub enum DimKey {
    Uuid(Uuid),
    Text(String),
    Integer(i64),
    Timestamp(String),  // ISO 8601 (DateTime serialized as string for hash stability)
    GeoHash(String),    // h3 geohash or similar
    HierarchyPath(Vec<String>),
    Null,               // missing / sparse dimension key
}

impl DimKey {
    pub fn uuid(id: Uuid) -> Self { Self::Uuid(id) }
    pub fn text(s: impl Into<String>) -> Self { Self::Text(s.into()) }
    pub fn integer(i: i64) -> Self { Self::Integer(i) }
    pub fn timestamp(dt: &DateTime<Utc>) -> Self { Self::Timestamp(dt.to_rfc3339()) }

    pub fn as_uuid(&self) -> Option<Uuid> {
        match self { Self::Uuid(u) => Some(*u), _ => None }
    }
    pub fn as_str(&self) -> Option<&str> {
        match self { Self::Text(s) | Self::Timestamp(s) | Self::GeoHash(s) => Some(s), _ => None }
    }
}

/// Serializable DimCoordinate (no reference to Hypercube at this level).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SerializableDimCoordinate {
    pub cube_id: CubeId,
    pub keys:    Vec<DimKey>,
}

// ─── DimCoordinate ────────────────────────────────────────────────────────────

/// Full N-dimensional coordinate for one HyperCell.
/// keys[0] = D₁, keys[1] = D₂, … keys[n-1] = Dₙ
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DimCoordinate {
    pub grid_id: GridId,
    pub cube_id: CubeId,
    pub keys:    Vec<DimKey>,   // length == cube.n
}

impl DimCoordinate {
    pub fn new(grid_id: GridId, cube_id: CubeId, keys: Vec<DimKey>) -> Self {
        Self { grid_id, cube_id, keys }
    }

    /// Convenience: N=2 coordinate (entity × property).
    pub fn n2(grid_id: GridId, cube_id: CubeId, d1: DimKey, d2: DimKey) -> Self {
        Self::new(grid_id, cube_id, vec![d1, d2])
    }

    pub fn d1(&self) -> Option<&DimKey> { self.keys.first() }
    pub fn d2(&self) -> Option<&DimKey> { self.keys.get(1) }
    pub fn d3(&self) -> Option<&DimKey> { self.keys.get(2) }
    pub fn d4(&self) -> Option<&DimKey> { self.keys.get(3) }

    pub fn d1_str(&self) -> Option<&str> {
        self.d2().and_then(|k| k.as_str())
    }

    pub fn n(&self) -> usize { self.keys.len() }

    pub fn to_serializable(&self) -> SerializableDimCoordinate {
        SerializableDimCoordinate { cube_id: self.cube_id, keys: self.keys.clone() }
    }
}

// ─── HyperCell ────────────────────────────────────────────────────────────────

/// The atomic data unit of Hypergrid — one attribute at one N-dim coordinate.
#[derive(Debug, Clone)]
pub struct HyperCell {
    pub coord:       DimCoordinate,
    pub attr_key:    AttributeKey,
    pub value:       TypedAttrValue,
    pub vector_clock: crate::crdt::VectorClock,
    pub last_actor:  ActorId,
    pub version:     u64,
    pub visibility:  AttrVisibility,
    pub created_at:  DateTime<Utc>,
    pub updated_at:  DateTime<Utc>,
}

impl HyperCell {
    pub fn new(coord: DimCoordinate, attr_key: impl Into<String>, value: TypedAttrValue, actor: ActorId) -> Self {
        let now = Utc::now();
        Self {
            coord,
            attr_key: attr_key.into(),
            value,
            vector_clock: crate::crdt::VectorClock::default(),
            last_actor: actor,
            version: 1,
            visibility: AttrVisibility::Tenant,
            created_at: now,
            updated_at: now,
        }
    }
}

// ─── HyperRow ─────────────────────────────────────────────────────────────────

/// All HyperCells sharing the same D₁ key — the canonical entity record.
/// Analogous to one row in a conventional spreadsheet.
#[derive(Debug, Clone, Default)]
pub struct HyperRow {
    pub cube_id:  CubeId,
    pub d1_key:   DimKey,
    pub attrs:    HashMap<AttributeKey, TypedAttrValue>,
    pub vector_clock: crate::crdt::VectorClock,
    pub version:  u64,
}

impl HyperRow {
    pub fn new(cube_id: CubeId, d1_key: DimKey) -> Self {
        Self { cube_id, d1_key, attrs: HashMap::new(), vector_clock: Default::default(), version: 0 }
    }

    pub fn get_attr(&self, key: &str) -> Option<&TypedAttrValue> { self.attrs.get(key) }

    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.attrs.get(key)?.as_str()
    }

    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.attrs.get(key)?.as_f64()
    }

    pub fn get_json(&self, key: &str) -> Option<&serde_json::Value> {
        self.attrs.get(key)?.as_json()
    }

    pub fn get_json_array_len(&self, key: &str) -> Option<usize> {
        self.get_json(key)?.as_array().map(|a| a.len())
    }

    pub fn set_attr(&mut self, key: impl Into<String>, value: TypedAttrValue) {
        self.attrs.insert(key.into(), value);
        self.version += 1;
    }
}

// ─── AttrComputation ─────────────────────────────────────────────────────────

/// Describes how a computed attribute is evaluated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttrComputation {
    /// Tier-1: synchronous formula evaluated inline on read.
    Formula { expression: FormulaExpr },

    /// Tier-2: async AI/ML engine writes the value via WritebackService.
    AiEngine {
        plugin_id:     PluginId,
        model_id:      String,
        staleness_ttl: u64,      // seconds before the cached value is considered stale
        priority:      ComputePriority,
    },

    /// Tier-1: roll up a DimFold aggregate across a dimension axis.
    DimFold {
        axis_index: u8,
        agg_fn:     AggFn,
        source_attr: AttributeKey,
    },

    /// Tier-1: aggregate over Hypergraph neighbors.
    GraphAggregate {
        edge_type:  crate::graph::EdgeType,
        direction:  crate::graph::EdgeDirection,
        agg_fn:     AggFn,
        source_attr: AttributeKey,
    },
}

/// Priority level for Tier-2 AI compute requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComputePriority { Critical, High, Normal, Background }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AggFn {
    Sum, Avg, Min, Max, Count, First, Last,
    Median, StdDev, Percentile(u8),
    ArrayAgg, DistinctCount,
}

// ─── FormulaExpr ─────────────────────────────────────────────────────────────

/// Tier-1 formula expression tree — evaluated synchronously at read time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormulaExpr {
    Attr(AttributeKey),              // read another attr from the same HyperRow
    Literal(TypedAttrValue),         // constant value
    Null,

    // Arithmetic
    Add(Box<FormulaExpr>, Box<FormulaExpr>),
    Sub(Box<FormulaExpr>, Box<FormulaExpr>),
    Mul(Box<FormulaExpr>, Box<FormulaExpr>),
    Div(Box<FormulaExpr>, Box<FormulaExpr>),  // division by zero → Null

    // Logic
    If { condition: Box<FormulaExpr>, then_expr: Box<FormulaExpr>, else_expr: Box<FormulaExpr> },
    And(Box<FormulaExpr>, Box<FormulaExpr>),
    Or(Box<FormulaExpr>,  Box<FormulaExpr>),
    Not(Box<FormulaExpr>),

    // Comparison
    Eq(Box<FormulaExpr>,  Box<FormulaExpr>),
    Lt(Box<FormulaExpr>,  Box<FormulaExpr>),
    Gt(Box<FormulaExpr>,  Box<FormulaExpr>),

    // Cross-cube lookup
    Lookup {
        cube_name: String,
        d1_key_expr: Box<FormulaExpr>,
        d2_key:    AttributeKey,
    },

    // Graph aggregation
    GraphAggregate {
        edge_type:  crate::graph::EdgeType,
        direction:  crate::graph::EdgeDirection,
        attr:       AttributeKey,
        agg_fn:     AggFn,
    },

    // Dimension fold
    DimFold {
        axis_index:  u8,
        agg_fn:      AggFn,
        source_attr: AttributeKey,
    },

    // Built-in functions
    Now,
    DateDiff { a: Box<FormulaExpr>, b: Box<FormulaExpr>, unit: DateUnit },
    Coalesce(Vec<FormulaExpr>),
    Concat(Vec<FormulaExpr>),
    Ratio { numerator: Box<FormulaExpr>, denominator: Box<FormulaExpr> },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DateUnit { Days, Hours, Months, Years }

// ─── AttributeKeyDef ─────────────────────────────────────────────────────────

/// Complete schema definition for one attribute key (D₂ column) in a Hypercube.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeKeyDef {
    pub key:              AttributeKey,
    pub display_name:     String,
    pub description:      String,
    pub attr_type:        AttrType,
    pub crdt_semantics:   CrdtSemantics,
    pub write_permission: PermissionTier,
    pub visibility:       AttrVisibility,
    pub computation:      Option<AttrComputation>,
    pub default_value:    Option<TypedAttrValue>,
    pub required:         bool,
    pub searchable:       bool,
    pub version_track:    bool,  // if true, full history kept in EventLog
    pub index_strategy:   IndexStrategy,
    pub aggregations:     Vec<AggFn>,
    pub alias_of:         Option<AttributeKey>, // for rename migrations
    pub tombstoned:       bool,                 // soft-deleted key
    pub plugin_id:        Option<PluginId>,
    pub notes:            String,
}

impl AttributeKeyDef {
    pub fn new(key: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            display_name: display_name.into(),
            description: String::new(),
            attr_type: AttrType::Json,
            crdt_semantics: CrdtSemantics::LastWriteWins,
            write_permission: PermissionTier::Editor,
            visibility: AttrVisibility::Tenant,
            computation: None,
            default_value: None,
            required: false,
            searchable: false,
            version_track: false,
            index_strategy: IndexStrategy::None,
            aggregations: vec![],
            alias_of: None,
            tombstoned: false,
            plugin_id: None,
            notes: String::new(),
        }
    }

    // ── Builder helpers ────────────────────────────────────────────────────

    pub fn lww_text(key: impl Into<String>, name: impl Into<String>, tier: PermissionTier) -> Self {
        Self {
            key: key.into(), display_name: name.into(),
            attr_type: AttrType::Text, crdt_semantics: CrdtSemantics::LastWriteWins,
            write_permission: tier, ..Self::default_key()
        }
    }

    pub fn lww_json(key: impl Into<String>, name: impl Into<String>, tier: PermissionTier) -> Self {
        Self {
            key: key.into(), display_name: name.into(),
            attr_type: AttrType::Json, crdt_semantics: CrdtSemantics::LastWriteWins,
            write_permission: tier, ..Self::default_key()
        }
    }

    pub fn orset(key: impl Into<String>, name: impl Into<String>, tier: PermissionTier) -> Self {
        Self {
            key: key.into(), display_name: name.into(),
            attr_type: AttrType::Json, crdt_semantics: CrdtSemantics::OrSet,
            write_permission: tier, ..Self::default_key()
        }
    }

    pub fn max_register(key: impl Into<String>, name: impl Into<String>, tier: PermissionTier) -> Self {
        Self {
            key: key.into(), display_name: name.into(),
            attr_type: AttrType::Text, crdt_semantics: CrdtSemantics::MaxRegister,
            write_permission: tier, ..Self::default_key()
        }
    }

    pub fn pn_counter(key: impl Into<String>, name: impl Into<String>, tier: PermissionTier) -> Self {
        Self {
            key: key.into(), display_name: name.into(),
            attr_type: AttrType::Number, crdt_semantics: CrdtSemantics::PNCounter,
            write_permission: tier, ..Self::default_key()
        }
    }

    pub fn grow_only(key: impl Into<String>, name: impl Into<String>, tier: PermissionTier) -> Self {
        Self {
            key: key.into(), display_name: name.into(),
            attr_type: AttrType::Number, crdt_semantics: CrdtSemantics::GrowOnlyCounter,
            write_permission: tier, ..Self::default_key()
        }
    }

    pub fn ai_signal(key: impl Into<String>, name: impl Into<String>, plugin_id: impl Into<String>) -> Self {
        let pid = plugin_id.into();
        Self {
            key: key.into(), display_name: name.into(),
            attr_type: AttrType::AiSignal,
            crdt_semantics: CrdtSemantics::LastWriteWins,
            write_permission: PermissionTier::System,
            visibility: AttrVisibility::Tenant,
            computation: Some(AttrComputation::AiEngine {
                plugin_id: pid,
                model_id: String::new(),
                staleness_ttl: 3600,
                priority: ComputePriority::Normal,
            }),
            ..Self::default_key()
        }
    }

    pub fn tier1_formula(key: impl Into<String>, name: impl Into<String>, expr: FormulaExpr) -> Self {
        Self {
            key: key.into(), display_name: name.into(),
            attr_type: AttrType::Computed,
            crdt_semantics: CrdtSemantics::LastWriteWins,
            write_permission: PermissionTier::System,
            computation: Some(AttrComputation::Formula { expression: expr }),
            ..Self::default_key()
        }
    }

    pub fn with_lattice(mut self, order: crate::crdt::LatticeOrder) -> Self {
        self.crdt_semantics = CrdtSemantics::Lattice(order);
        self
    }

    fn default_key() -> Self {
        Self {
            key: String::new(), display_name: String::new(),
            description: String::new(), attr_type: AttrType::Json,
            crdt_semantics: CrdtSemantics::LastWriteWins,
            write_permission: PermissionTier::Editor,
            visibility: AttrVisibility::Tenant,
            computation: None, default_value: None, required: false,
            searchable: false, version_track: false,
            index_strategy: IndexStrategy::None, aggregations: vec![],
            alias_of: None, tombstoned: false, plugin_id: None,
            notes: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttrType {
    Text, Number, Integer, Bool, Date, DateTime, Duration, Currency,
    Enum, MultiEnum, Relation, MultiRelation, User, Tag, TagSet, Json,
    Computed, AiSignal, AnomalySignal, RichText, Attachment, Custom(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexStrategy {
    None, BTree, Hash, Gin, Brin, FullText,
}

// ─── AttributeKeyRegistry ─────────────────────────────────────────────────────

/// Schema registry for all attribute keys in one Hypercube.
/// Ordered by insertion for consistent column ordering in views.
#[derive(Debug, Default, Clone)]
pub struct AttributeKeyRegistry {
    entries:  indexmap::IndexMap<AttributeKey, AttributeKeyDef>,
    aliases:  HashMap<AttributeKey, AttributeKey>, // old_key → canonical_key
}

impl AttributeKeyRegistry {
    pub fn register(&mut self, def: AttributeKeyDef) -> Result<(), HypergridError> {
        if let Some(existing) = self.entries.get(&def.key) {
            if !existing.tombstoned {
                // Idempotent: re-registering same key is a no-op
                return Ok(());
            }
        }
        self.entries.insert(def.key.clone(), def);
        Ok(())
    }

    pub fn register_batch(&mut self, defs: Vec<AttributeKeyDef>) -> Result<(), HypergridError> {
        for def in defs { self.register(def)?; }
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&AttributeKeyDef> {
        // Resolve alias if needed
        let resolved = self.aliases.get(key).map(|s| s.as_str()).unwrap_or(key);
        let def = self.entries.get(resolved)?;
        if def.tombstoned { None } else { Some(def) }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut AttributeKeyDef> {
        self.entries.get_mut(key)
    }

    pub fn tombstone(&mut self, key: &str) -> Result<(), HypergridError> {
        self.entries.get_mut(key)
            .ok_or_else(|| HypergridError::NotFound(format!("attr key: {key}")))?
            .tombstoned = true;
        Ok(())
    }

    pub fn add_alias(&mut self, old_key: impl Into<String>, new_key: impl Into<String>) {
        self.aliases.insert(old_key.into(), new_key.into());
    }

    pub fn computed_attrs(&self) -> Vec<&AttributeKeyDef> {
        self.entries.values()
            .filter(|d| !d.tombstoned && d.computation.is_some())
            .collect()
    }

    pub fn ai_attrs(&self) -> Vec<&AttributeKeyDef> {
        self.entries.values()
            .filter(|d| !d.tombstoned && matches!(d.computation, Some(AttrComputation::AiEngine { .. })))
            .collect()
    }

    pub fn all(&self) -> impl Iterator<Item = &AttributeKeyDef> {
        self.entries.values().filter(|d| !d.tombstoned)
    }

    pub fn len(&self) -> usize {
        self.entries.values().filter(|d| !d.tombstoned).count()
    }
}
