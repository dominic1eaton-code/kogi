// =============================================================================
// hypergrid::dim — HG-DIM: Dimension Axis System
//
// DimensionAxis, AxisType, KeyType, KeyCardinality, AxisOrdering,
// IndexStrategy, StorageEncoding, HypercubeStats
// =============================================================================

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::cell::{AxisId, PluginId};

// ─── AxisType ─────────────────────────────────────────────────────────────────

/// What kind of data does this dimension axis represent?
/// Each AxisType has a recommended KeyType, IndexStrategy, and ordering.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AxisType {
    /// D₁: the primary entity axis. Keys are entity identifiers (typically UUIDs).
    EntityAxis,

    /// D₂: the property axis. Keys are attribute key strings (field names).
    PropertyAxis,

    /// D₃+: time-series axis. Keys are timestamps or period strings ("2026-Q2").
    /// Uses BRIN index. Supports AS_OF time-travel and FOLD aggregations.
    TimeAxis,

    /// D₃/D₄: categorical dimension. Keys are category strings.
    /// Example: jurisdiction codes ("US-DE", "UK", "EU"), audience segments.
    CategoryAxis,

    /// D₃/D₄: geographic axis. Keys are geohash strings.
    /// Uses PostGIS spatial index. Supports bounding box and radius queries.
    GeoAxis,

    /// D₃/D₄: multi-tenant axis. Keys are tenant IDs.
    /// Row-level security per D₃/D₄ key. One cube, isolated tenant data.
    TenantAxis,

    /// D₃/D₄: scenario/planning axis. Keys are scenario names ("base_case", "optimistic").
    ScenarioAxis,

    /// D₃/D₄: organizational hierarchy axis. Keys are org unit IDs.
    HierarchyAxis,

    /// D₃+: version axis. Keys are semver strings.
    VersionAxis,

    /// Ordinal axis with an integer key.
    OrdinalAxis,

    /// Custom axis type registered by a domain plugin.
    Custom(String),
}

impl AxisType {
    pub fn recommended_index(&self) -> IndexStrategy {
        match self {
            Self::EntityAxis    => IndexStrategy::Hash,
            Self::PropertyAxis  => IndexStrategy::Gin,
            Self::TimeAxis      => IndexStrategy::Brin,
            Self::GeoAxis       => IndexStrategy::Spatial,
            Self::CategoryAxis  => IndexStrategy::BTree,
            Self::TenantAxis    => IndexStrategy::BTree,
            Self::HierarchyAxis => IndexStrategy::BTree,
            Self::VersionAxis   => IndexStrategy::BTree,
            Self::OrdinalAxis   => IndexStrategy::BTree,
            _                   => IndexStrategy::BTree,
        }
    }

    pub fn recommended_ordering(&self) -> AxisOrdering {
        match self {
            Self::EntityAxis   => AxisOrdering::Unordered,
            Self::PropertyAxis => AxisOrdering::Lexicographic,
            Self::TimeAxis     => AxisOrdering::Chronological,
            Self::OrdinalAxis  => AxisOrdering::Numeric,
            Self::VersionAxis  => AxisOrdering::SemVer,
            _                  => AxisOrdering::Lexicographic,
        }
    }
}

// ─── KeyType ─────────────────────────────────────────────────────────────────

/// The Rust / database type of the keys in this dimension axis.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyType {
    Uuid,
    String,
    Integer,
    Timestamp,
    GeoHash,
    SemVer,
    HierarchyPath,
}

// ─── KeyCardinality ──────────────────────────────────────────────────────────

/// Expected cardinality of the key space for this axis.
/// Used for storage encoding selection and index strategy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyCardinality {
    /// Bounded, fully-populated key space (e.g., entity IDs where most exist).
    Dense { max_keys: u64 },
    /// Unbounded or sparsely-populated (e.g., timestamps, geohashes).
    Sparse,
    /// Effectively infinite (e.g., timestamp with nanosecond precision).
    Infinite,
}

// ─── AxisOrdering ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AxisOrdering {
    Unordered,
    Lexicographic,
    Chronological,
    Numeric,
    SemVer,
    Custom(String), // plugin_id provides comparison logic
}

// ─── IndexStrategy ───────────────────────────────────────────────────────────

/// Physical database index type for this dimension axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexStrategy {
    None,
    BTree,       // General-purpose; O(log n) lookup; supports range queries
    Hash,        // O(1) exact-match; no range support
    Gin,         // PostgreSQL GIN; inverted index for JSONB and array queries
    Brin,        // Block Range INdex; efficient for temporally-ordered large tables
    Spatial,     // PostGIS spatial index (for GeoAxis)
    FullText,    // For searchable text fields
}

// ─── StorageEncoding ─────────────────────────────────────────────────────────

/// Physical storage layout for a Hypercube.
/// Chosen at creation time based on expected data density and access patterns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StorageEncoding {
    /// Every D₁×D₂ coordinate is expected to have a value (small, fully-populated cubes).
    Dense,

    /// Most coordinates are empty; only non-null cells are stored (TimeAxis, GeoAxis).
    Sparse,

    /// Dense for specified axes (e.g., D₁×D₂), sparse for the rest.
    /// Best for entity×field cubes with optional time/category extension.
    Hybrid { dense_dims: Vec<u8> },
}

impl Default for StorageEncoding {
    fn default() -> Self { Self::Hybrid { dense_dims: vec![1, 2] } }
}

// ─── DimensionAxis ────────────────────────────────────────────────────────────

/// Complete definition of one dimension axis in a Hypercube.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionAxis {
    pub axis_id:       AxisId,
    pub dim_index:     u8,             // 1 = D₁, 2 = D₂, … 16 = D₁₆
    pub name:          String,         // human-readable ("entity", "property", "time", "region")
    pub axis_type:     AxisType,
    pub key_type:      KeyType,
    pub cardinality:   KeyCardinality,
    pub ordering:      AxisOrdering,
    pub nullable:      bool,           // true if Dₙ key may be DimKey::Null
    pub index_strategy: IndexStrategy,
    pub plugin_id:     Option<PluginId>, // None = built-in axis type
    pub description:   String,
}

impl DimensionAxis {
    /// D₁: entity axis (UUID primary key)
    pub fn entity(name: impl Into<String>) -> Self {
        Self {
            axis_id:    Uuid::new_v4(),
            dim_index:  1,
            name:       name.into(),
            axis_type:  AxisType::EntityAxis,
            key_type:   KeyType::Uuid,
            cardinality: KeyCardinality::Sparse,
            ordering:   AxisOrdering::Unordered,
            nullable:   false,
            index_strategy: IndexStrategy::Hash,
            plugin_id:  None,
            description: String::new(),
        }
    }

    /// D₂: property axis (string field names)
    pub fn property(name: impl Into<String>) -> Self {
        Self {
            axis_id:    Uuid::new_v4(),
            dim_index:  2,
            name:       name.into(),
            axis_type:  AxisType::PropertyAxis,
            key_type:   KeyType::String,
            cardinality: KeyCardinality::Dense { max_keys: 1024 },
            ordering:   AxisOrdering::Lexicographic,
            nullable:   false,
            index_strategy: IndexStrategy::Gin,
            plugin_id:  None,
            description: String::new(),
        }
    }

    /// D₃+: time axis (timestamp keys)
    pub fn time(name: impl Into<String>) -> Self {
        Self {
            axis_id:    Uuid::new_v4(),
            dim_index:  3,
            name:       name.into(),
            axis_type:  AxisType::TimeAxis,
            key_type:   KeyType::Timestamp,
            cardinality: KeyCardinality::Infinite,
            ordering:   AxisOrdering::Chronological,
            nullable:   true,
            index_strategy: IndexStrategy::Brin,
            plugin_id:  None,
            description: String::new(),
        }
    }

    /// D₃/D₄: category axis (string keys)
    pub fn category(name: impl Into<String>, dim_index: u8) -> Self {
        Self {
            axis_id:    Uuid::new_v4(),
            dim_index,
            name:       name.into(),
            axis_type:  AxisType::CategoryAxis,
            key_type:   KeyType::String,
            cardinality: KeyCardinality::Dense { max_keys: 1000 },
            ordering:   AxisOrdering::Lexicographic,
            nullable:   true,
            index_strategy: IndexStrategy::BTree,
            plugin_id:  None,
            description: String::new(),
        }
    }

    /// D₃/D₄: geo axis (geohash keys)
    pub fn geo(name: impl Into<String>, dim_index: u8) -> Self {
        Self {
            axis_id:    Uuid::new_v4(),
            dim_index,
            name:       name.into(),
            axis_type:  AxisType::GeoAxis,
            key_type:   KeyType::GeoHash,
            cardinality: KeyCardinality::Infinite,
            ordering:   AxisOrdering::Lexicographic,
            nullable:   true,
            index_strategy: IndexStrategy::Spatial,
            plugin_id:  None,
            description: String::new(),
        }
    }

    /// D₃/D₄: tenant axis (UUID keys)
    pub fn tenant(name: impl Into<String>, dim_index: u8) -> Self {
        Self {
            axis_id:    Uuid::new_v4(),
            dim_index,
            name:       name.into(),
            axis_type:  AxisType::TenantAxis,
            key_type:   KeyType::Uuid,
            cardinality: KeyCardinality::Sparse,
            ordering:   AxisOrdering::Unordered,
            nullable:   true,
            index_strategy: IndexStrategy::BTree,
            plugin_id:  None,
            description: String::new(),
        }
    }
}

// ─── HypercubeStats ──────────────────────────────────────────────────────────

/// Cardinality statistics for query planning.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct HypercubeStats {
    pub d1_cardinality:  u64,  // distinct D₁ key count (number of entities)
    pub total_cells:     u64,  // total HyperCell count
    pub avg_cells_per_row: f64, // average number of attributes per entity
    pub d3_min:          Option<String>,  // min D₃ key (TimeAxis lower bound)
    pub d3_max:          Option<String>,  // max D₃ key (TimeAxis upper bound)
    pub last_updated:    Option<chrono::DateTime<chrono::Utc>>,
}

// ─── DimSlice / DimSlicePredicate ────────────────────────────────────────────

/// A predicate on one dimension axis for DimSlice queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimSlicePredicate {
    pub dim_index: u8,
    pub predicate: DimKeyPredicate,
}

/// Filter on a dimension key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DimKeyPredicate {
    Eq(crate::cell::DimKey),
    In(Vec<crate::cell::DimKey>),
    Between(crate::cell::DimKey, crate::cell::DimKey),
    Null,
    NotNull,
    Under(crate::cell::DimKey), // HierarchyAxis: this node AND all descendants
}

/// A complete multi-axis filter for a Hypercube scan.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DimSlice {
    pub predicates:    Vec<DimSlicePredicate>,
    pub attr_filters:  Vec<AttrValuePredicate>,
    pub include_null_dims: bool,
}

/// Filter on an attribute value (WHERE cell value matches condition).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttrValuePredicate {
    pub attr_key:  String,
    pub predicate: ValuePredicate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValuePredicate {
    Eq(serde_json::Value),
    Ne(serde_json::Value),
    Gt(f64),
    Lt(f64),
    Gte(f64),
    Lte(f64),
    Contains(String),   // substring / OR-Set membership
    NotNull,
    IsNull,
    In(Vec<serde_json::Value>),
}
