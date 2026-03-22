//! HG-QL — HyperQL N-dimensional query engine.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::cell::{AttributeKey, CubeId, DimCoordinate, GridId, TypedAttrValue};
use crate::dim::{DimFold, DimSlicePredicate};

// ─── HyperQL Statement ────────────────────────────────────────────────────────

/// A HyperQL query statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperQuery {
    /// The Hypercubes to query.
    pub from: Vec<CubeId>,
    /// Axis slices (WHERE predicates on any dimension).
    pub where_slices: Vec<DimSlicePredicate>,
    /// Which attribute keys to return (SELECT).
    pub select_attrs: Vec<AttributeKey>,
    /// Axes to collapse by aggregation (FOLD).
    pub fold: Vec<DimFold>,
    /// Sort order.
    pub order_by: Vec<(AttributeKey, bool)>,
    /// Pagination.
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    /// Time-travel: resolve the cube state at this timestamp.
    pub as_of: Option<chrono::DateTime<chrono::Utc>>,
    /// Include shadow cells from linked grids.
    pub include_shadows: bool,
    /// Cross-grid join specification.
    pub joins: Vec<CrossGridJoin>,
}

impl HyperQuery {
    pub fn from(cube_id: CubeId) -> Self {
        Self {
            from: vec![cube_id],
            where_slices: vec![],
            select_attrs: vec![],
            fold: vec![],
            order_by: vec![],
            limit: None,
            offset: None,
            as_of: None,
            include_shadows: false,
            joins: vec![],
        }
    }

    pub fn select(mut self, attrs: Vec<impl Into<AttributeKey>>) -> Self {
        self.select_attrs = attrs.into_iter().map(Into::into).collect();
        self
    }

    pub fn filter(mut self, slice: DimSlicePredicate) -> Self {
        self.where_slices.push(slice);
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    pub fn as_of(mut self, ts: chrono::DateTime<chrono::Utc>) -> Self {
        self.as_of = Some(ts);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossGridJoin {
    pub target_grid_id: GridId,
    pub target_cube_id: CubeId,
    pub join_key_attr: AttributeKey,
}

// ─── Query Result ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct QueryRow {
    pub coord: DimCoordinate,
    pub attrs: std::collections::HashMap<AttributeKey, TypedAttrValue>,
}

#[derive(Debug, Clone)]
pub struct QueryResult {
    pub rows: Vec<QueryRow>,
    pub total_count: usize,
    pub execution_time_ms: u64,
}
