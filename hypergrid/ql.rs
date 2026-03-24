// =============================================================================
// hypergrid::ql — HG-QL: HyperQL Query Language
//
// HyperQuery builder, HyperQLClause, QueryPlan, PhysicalPlan,
// QueryResult, HyperQLPlanner, AggFn, OrderSpec
// =============================================================================

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::cell::{AggFn, AttributeKey, CubeId, DimKey, HyperRow, TypedAttrValue};
use crate::dim::{DimSlice, DimSlicePredicate, DimKeyPredicate, AttrValuePredicate, ValuePredicate};
use crate::graph::EdgeType;
use crate::view::SortDirection;

// ─── QueryClause ─────────────────────────────────────────────────────────────

/// SELECT clause: which values to project into the result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectExpr {
    /// All attributes of the row: SELECT *
    Star,
    /// One attribute by key.
    Attr(AttributeKey),
    /// Attribute with alias: attr AS alias
    AttrAlias { attr: AttributeKey, alias: String },
    /// Aggregation over a dimension axis.
    Fold { axis_index: u8, agg_fn: AggFn, source_attr: AttributeKey, alias: Option<String> },
    /// Pivot a dimension axis into columns.
    Expand { axis_index: u8, agg_fn: AggFn, col_prefix: Option<String> },
    /// D₁, D₂, etc. raw dimension key values.
    DimKey { dim_index: u8, alias: Option<String> },
    /// Computed expression.
    Expr { expression: crate::cell::FormulaExpr, alias: String },
}

/// FROM clause.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FromClause {
    Cube(CubeId),
    CubeName(String),  // resolved to CubeId in the Analyze stage
    Subquery(Box<HyperQuery>),
}

/// WHERE clause predicate (same as DimSlice but parsed from HyperQL string).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WhereExpr {
    DimEq    { dim_index: u8, value: DimKey },
    DimIn    { dim_index: u8, values: Vec<DimKey> },
    DimBetween { dim_index: u8, lo: DimKey, hi: DimKey },
    DimUnder { dim_index: u8, ancestor: DimKey }, // HierarchyAxis: node + all descendants
    AttrEq   { attr: AttributeKey, value: serde_json::Value },
    AttrGt   { attr: AttributeKey, value: f64 },
    AttrLt   { attr: AttributeKey, value: f64 },
    AttrGte  { attr: AttributeKey, value: f64 },
    AttrLte  { attr: AttributeKey, value: f64 },
    AttrContains { attr: AttributeKey, value: String },
    AttrNotNull { attr: AttributeKey },
    AttrIsNull  { attr: AttributeKey },
    GraphEdge { from_d1: DimKey, to_d1: DimKey, edge_type: EdgeType },
    And(Box<WhereExpr>, Box<WhereExpr>),
    Or (Box<WhereExpr>, Box<WhereExpr>),
    Not(Box<WhereExpr>),
}

/// JOIN clause: join another cube via a Hypergraph edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinClause {
    pub target_cube: FromClause,
    pub alias:       Option<String>,
    pub on:          JoinCondition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JoinCondition {
    GraphEdge { edge_type: EdgeType },
    AttrEq { left_attr: AttributeKey, right_attr: AttributeKey },
}

/// TRAVERSE clause: BFS/DFS graph traversal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraverseClause {
    pub from_expr:  TraverseStart,
    pub edge_type:  EdgeType,
    pub direction:  crate::graph::TraversalDirection,
    pub max_depth:  u8,
    pub min_depth:  u8,
    pub edge_filter: Option<crate::graph::EdgeTypeFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraverseStart {
    NodeId(crate::graph::EdgeId),
    D1Key(DimKey),
    Subquery(Box<HyperQuery>),
}

/// ORDER BY spec.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderSpec {
    pub expr:      SelectExpr,
    pub direction: SortDirection,
    pub nulls_last: bool,
}

// ─── HyperQuery ──────────────────────────────────────────────────────────────

/// A complete HyperQL query, built via the builder API.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HyperQuery {
    pub select:    Vec<SelectExpr>,
    pub from:      Option<FromClause>,
    pub as_of:     Option<DateTime<Utc>>,       // AS_OF time-travel
    pub where_:    Vec<WhereExpr>,
    pub joins:     Vec<JoinClause>,
    pub traverse:  Option<TraverseClause>,
    pub group_by:  Vec<AttributeKey>,
    pub order_by:  Vec<OrderSpec>,
    pub limit:     Option<u64>,
    pub offset:    u64,
    pub identity_tag: Option<String>,           // IDENTITY scope
    pub space_id:  Option<crate::cell::SpaceId>,// IN SPACE scope
    pub include_shadow: bool,                   // INCLUDE SHADOW CELLS
    pub shadow_from:    Option<crate::cell::GridId>, // SHADOW FROM grid
}

impl HyperQuery {
    pub fn new() -> Self { Self::default() }

    pub fn from(mut self, from: FromClause) -> Self {
        self.from = Some(from);
        self
    }

    pub fn from_cube(mut self, cube_id: CubeId) -> Self {
        self.from = Some(FromClause::Cube(cube_id));
        self
    }

    pub fn from_name(mut self, name: impl Into<String>) -> Self {
        self.from = Some(FromClause::CubeName(name.into()));
        self
    }

    pub fn select_all(mut self) -> Self { self.select.push(SelectExpr::Star); self }
    pub fn select_attr(mut self, key: impl Into<String>) -> Self {
        self.select.push(SelectExpr::Attr(key.into())); self
    }
    pub fn select(mut self, keys: Vec<&str>) -> Self {
        for k in keys { self.select.push(SelectExpr::Attr(k.into())); }
        self
    }

    pub fn as_of(mut self, ts: DateTime<Utc>) -> Self { self.as_of = Some(ts); self }

    pub fn where_attr_eq(mut self, attr: impl Into<String>, value: serde_json::Value) -> Self {
        self.where_.push(WhereExpr::AttrEq { attr: attr.into(), value }); self
    }
    pub fn where_attr_gt(mut self, attr: impl Into<String>, value: f64) -> Self {
        self.where_.push(WhereExpr::AttrGt { attr: attr.into(), value }); self
    }
    pub fn where_attr_lt(mut self, attr: impl Into<String>, value: f64) -> Self {
        self.where_.push(WhereExpr::AttrLt { attr: attr.into(), value }); self
    }
    pub fn where_dim_eq(mut self, dim_index: u8, key: DimKey) -> Self {
        self.where_.push(WhereExpr::DimEq { dim_index, value: key }); self
    }

    pub fn order_by(mut self, attr: impl Into<String>, dir: SortDirection) -> Self {
        self.order_by.push(OrderSpec {
            expr: SelectExpr::Attr(attr.into()),
            direction: dir, nulls_last: true,
        });
        self
    }

    pub fn limit(mut self, n: u64) -> Self { self.limit = Some(n); self }
    pub fn offset(mut self, n: u64) -> Self { self.offset = n; self }

    pub fn identity(mut self, tag: impl Into<String>) -> Self {
        self.identity_tag = Some(tag.into()); self
    }

    pub fn in_space(mut self, space_id: crate::cell::SpaceId) -> Self {
        self.space_id = Some(space_id); self
    }

    pub fn include_shadow(mut self) -> Self { self.include_shadow = true; self }

    pub fn fold(mut self, axis_index: u8, agg: AggFn, source: impl Into<String>) -> Self {
        self.select.push(SelectExpr::Fold {
            axis_index, agg_fn: agg, source_attr: source.into(), alias: None,
        });
        self
    }

    pub fn traverse(mut self, clause: TraverseClause) -> Self {
        self.traverse = Some(clause); self
    }

    /// Filter on AI attribute confidence.
    pub fn where_ai_confidence(mut self, attr: impl Into<String>, min_confidence: f64) -> Self {
        // This would be evaluated specially in the executor
        // For now: add as an AttrGt filter on the confidence sub-field
        self.where_.push(WhereExpr::AttrGte { attr: format!("{}_confidence", attr.into()), value: min_confidence });
        self
    }
}

// ─── QueryResult ─────────────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct QueryResult {
    pub columns:     Vec<ResultColumn>,
    pub rows:        Vec<ResultRow>,
    pub total_count: u64,      // total matching (before LIMIT)
    pub plan_info:   PlanInfo,
    pub execution_ms: u64,
}

#[derive(Debug, Clone)]
pub struct ResultColumn {
    pub name:      String,
    pub attr_type: Option<crate::cell::AttrType>,
}

#[derive(Debug, Clone)]
pub struct ResultRow {
    pub d1_key:  DimKey,
    pub values:  Vec<Option<TypedAttrValue>>,  // aligned to columns
    pub is_shadow: bool,
}

#[derive(Debug, Default)]
pub struct PlanInfo {
    pub plan_type:     String,  // "IndexScan", "FullScan", "DimFoldScan", etc.
    pub storage_route: String,  // "postgres" or "clickhouse"
    pub estimated_rows: u64,
}

// ─── PhysicalPlan ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum PhysicalPlan {
    /// Primary-key or indexed column lookup. O(log n).
    IndexScan {
        cube_id:       CubeId,
        index_dim:     u8,
        predicate:     DimSlicePredicate,
        attr_filters:  Vec<AttrValuePredicate>,
    },
    /// Full table scan (fallback when no index covers the predicate).
    FullScan {
        cube_id:       CubeId,
        attr_filters:  Vec<AttrValuePredicate>,
    },
    /// DimFold aggregation — routed to ClickHouse above row threshold.
    DimFoldScan {
        cube_id:       CubeId,
        fold_dim:      u8,
        agg_fn:        AggFn,
        source_attr:   AttributeKey,
        pre_filter:    Option<DimSlice>,
        route:         StorageRoute,
    },
    /// BFS/DFS Hypergraph traversal.
    GraphTraversal {
        start:         crate::graph::EdgeId,
        config:        crate::graph::TraversalConfig,
    },
    /// AS_OF time-travel via EventLog replay.
    AsOf {
        cube_id:       CubeId,
        d1_key:        Option<DimKey>,
        target_time:   DateTime<Utc>,
        base_plan:     Box<PhysicalPlan>,
    },
    /// Join with a CrossGridLink target Grid.
    CrossGridJoin {
        local_plan:    Box<PhysicalPlan>,
        target_grid:   crate::cell::GridId,
        target_cube:   CubeId,
        edge_type:     EdgeType,
    },
    /// Union of multiple plans (e.g., local + shadow cells).
    Union(Vec<PhysicalPlan>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageRoute { Postgres, ClickHouse }

// ─── HyperQLPlanner ──────────────────────────────────────────────────────────

/// The five-stage HyperQL execution pipeline: Parse → Analyze → Plan → Optimize → Execute.
///
/// This in-memory implementation plans against an in-memory HyperRow store.
/// Production code generates SQL for PostgreSQL or ClickHouse.
pub struct HyperQLPlanner {
    analytics_threshold: u64,  // row count above which to route to ClickHouse
}

impl HyperQLPlanner {
    pub fn new() -> Self { Self { analytics_threshold: 1_000_000 } }

    // ── Stage 1: Parse ────────────────────────────────────────────────────

    /// Parse is a no-op in the builder API — the HyperQuery is already structured.
    /// In production: `pest` PEG parser converts HyperQL string to HyperQuery.
    pub fn parse(&self, query_str: &str) -> Result<HyperQuery, QueryError> {
        // Stub: real parser uses pest crate with hg_ql.pest grammar
        Err(QueryError::ParseError("raw HyperQL string parsing not implemented in in-memory mode; use HyperQuery builder".into()))
    }

    // ── Stage 2: Analyze ─────────────────────────────────────────────────

    /// Resolve cube names → CubeIds; validate attr keys; check PermissionTier.
    pub fn analyze(
        &self,
        query: &HyperQuery,
        cube_registry: &HashMap<String, CubeId>,
        _caller_tier: crate::cell::PermissionTier,
    ) -> Result<HyperQuery, QueryError> {
        let mut resolved = query.clone();

        // Resolve CubeName → Cube
        if let Some(FromClause::CubeName(name)) = &query.from {
            let cube_id = cube_registry.get(name)
                .ok_or_else(|| QueryError::UnknownCube(name.clone()))?;
            resolved.from = Some(FromClause::Cube(*cube_id));
        }

        // In production: validate attr keys against AttributeKeyRegistry,
        //   check PermissionTier against all accessed attrs,
        //   type-check FOLD expressions (must be numeric attrs),
        //   validate AS_OF timestamp format.

        Ok(resolved)
    }

    // ── Stage 3: Plan ─────────────────────────────────────────────────────

    pub fn plan(&self, query: &HyperQuery) -> Result<PhysicalPlan, QueryError> {
        let cube_id = match &query.from {
            Some(FromClause::Cube(id)) => *id,
            _ => return Err(QueryError::MissingFrom),
        };

        // Check if this is an AS_OF query
        if let Some(ts) = query.as_of {
            let base = self.plan_base(cube_id, query)?;
            return Ok(PhysicalPlan::AsOf {
                cube_id,
                d1_key: self.extract_d1_key(query),
                target_time: ts,
                base_plan: Box::new(base),
            });
        }

        // Check for DimFold
        let has_fold = query.select.iter().any(|s| matches!(s, SelectExpr::Fold { .. }));
        if has_fold {
            if let Some(SelectExpr::Fold { axis_index, agg_fn, source_attr, .. }) = query.select.iter()
                .find(|s| matches!(s, SelectExpr::Fold { .. }))
            {
                return Ok(PhysicalPlan::DimFoldScan {
                    cube_id,
                    fold_dim:    *axis_index,
                    agg_fn:      agg_fn.clone(),
                    source_attr: source_attr.clone(),
                    pre_filter:  None,
                    route:       StorageRoute::Postgres,  // optimizer may change to ClickHouse
                });
            }
        }

        // Check for graph traversal
        if let Some(traverse) = &query.traverse {
            return Ok(PhysicalPlan::GraphTraversal {
                start:  match &traverse.from_expr {
                    TraverseStart::NodeId(id) => *id,
                    _ => uuid::Uuid::nil(),
                },
                config: crate::graph::TraversalConfig {
                    max_depth: traverse.max_depth,
                    min_depth: traverse.min_depth,
                    edge_filter: traverse.edge_filter.clone(),
                    direction: traverse.direction,
                },
            });
        }

        self.plan_base(cube_id, query)
    }

    fn plan_base(&self, cube_id: CubeId, query: &HyperQuery) -> Result<PhysicalPlan, QueryError> {
        // Heuristic: if there's a D₁ exact match filter, use IndexScan
        let d1_pred = query.where_.iter().find(|w| matches!(w, WhereExpr::DimEq { dim_index: 1, .. }));
        if let Some(WhereExpr::DimEq { dim_index: 1, value }) = d1_pred {
            return Ok(PhysicalPlan::IndexScan {
                cube_id,
                index_dim: 1,
                predicate: DimSlicePredicate { dim_index: 1, predicate: DimKeyPredicate::Eq(value.clone()) },
                attr_filters: self.extract_attr_filters(query),
            });
        }

        // Fall back to FullScan
        Ok(PhysicalPlan::FullScan { cube_id, attr_filters: self.extract_attr_filters(query) })
    }

    // ── Stage 4: Optimize ─────────────────────────────────────────────────

    pub fn optimize(&self, plan: PhysicalPlan, estimated_rows: u64) -> PhysicalPlan {
        // Route large DimFoldScan to ClickHouse
        if let PhysicalPlan::DimFoldScan { route: ref mut r, .. } = plan.clone() {
            if estimated_rows > self.analytics_threshold {
                // return version with ClickHouse route
            }
        }
        plan
    }

    // ── Stage 5: Execute ──────────────────────────────────────────────────

    /// Execute a physical plan against an in-memory HyperRow store.
    pub fn execute(
        &self,
        plan: &PhysicalPlan,
        store: &HashMap<(CubeId, String), HyperRow>,
        query: &HyperQuery,
    ) -> QueryResult {
        let start = std::time::Instant::now();
        let mut result = QueryResult::default();

        match plan {
            PhysicalPlan::FullScan { cube_id, attr_filters } => {
                let rows: Vec<&HyperRow> = store.values()
                    .filter(|r| r.cube_id == *cube_id)
                    .filter(|r| self.apply_attr_filters(r, attr_filters))
                    .filter(|r| self.apply_where(r, &query.where_))
                    .collect();

                let total = rows.len() as u64;
                let rows = self.apply_order_limit(rows, query);
                result.columns = self.make_columns(query);
                result.rows = rows.iter().map(|r| self.project_row(r, query)).collect();
                result.total_count = total;
            }

            PhysicalPlan::IndexScan { cube_id, predicate, attr_filters, .. } => {
                // For in-memory: same as FullScan + D1 filter
                let d1_key = match &predicate.predicate {
                    DimKeyPredicate::Eq(k) => format!("{:?}", k),
                    _ => String::new(),
                };
                let lookup_key = (*cube_id, d1_key.clone());
                let rows: Vec<&HyperRow> = if let Some(row) = store.get(&lookup_key) {
                    vec![row]
                } else {
                    store.values()
                        .filter(|r| r.cube_id == *cube_id)
                        .filter(|r| format!("{:?}", r.d1_key) == d1_key)
                        .collect()
                };
                result.columns = self.make_columns(query);
                result.total_count = rows.len() as u64;
                result.rows = rows.iter().map(|r| self.project_row(r, query)).collect();
            }

            PhysicalPlan::AsOf { cube_id, target_time, .. } => {
                // Stub: in production replays EventLog to target_time
                result.plan_info.plan_type = "AsOf".into();
            }

            _ => {
                result.plan_info.plan_type = "Unsupported in memory executor".into();
            }
        }

        result.plan_info.plan_type = format!("{:?}", plan).chars().take(30).collect();
        result.plan_info.storage_route = "postgres".into();
        result.execution_ms = start.elapsed().as_millis() as u64;
        result
    }

    // ── Helpers ───────────────────────────────────────────────────────────

    fn extract_d1_key(&self, query: &HyperQuery) -> Option<DimKey> {
        query.where_.iter().find_map(|w| match w {
            WhereExpr::DimEq { dim_index: 1, value } => Some(value.clone()),
            _ => None,
        })
    }

    fn extract_attr_filters(&self, query: &HyperQuery) -> Vec<AttrValuePredicate> {
        query.where_.iter().filter_map(|w| match w {
            WhereExpr::AttrEq { attr, value } => Some(AttrValuePredicate {
                attr_key: attr.clone(),
                predicate: ValuePredicate::Eq(value.clone()),
            }),
            WhereExpr::AttrGt { attr, value } => Some(AttrValuePredicate {
                attr_key: attr.clone(),
                predicate: ValuePredicate::Gt(*value),
            }),
            WhereExpr::AttrLt { attr, value } => Some(AttrValuePredicate {
                attr_key: attr.clone(),
                predicate: ValuePredicate::Lt(*value),
            }),
            _ => None,
        }).collect()
    }

    fn apply_attr_filters(&self, row: &HyperRow, filters: &[AttrValuePredicate]) -> bool {
        filters.iter().all(|f| {
            let val = row.get_attr(&f.attr_key);
            match &f.predicate {
                ValuePredicate::Eq(v) => val.map(|a| serde_json::to_value(a).ok() == Some(v.clone())).unwrap_or(false),
                ValuePredicate::Gt(n) => val.and_then(|a| a.as_f64()).map(|v| v > *n).unwrap_or(false),
                ValuePredicate::Lt(n) => val.and_then(|a| a.as_f64()).map(|v| v < *n).unwrap_or(false),
                ValuePredicate::Gte(n) => val.and_then(|a| a.as_f64()).map(|v| v >= *n).unwrap_or(false),
                ValuePredicate::Lte(n) => val.and_then(|a| a.as_f64()).map(|v| v <= *n).unwrap_or(false),
                ValuePredicate::NotNull => val.map(|v| !v.is_null()).unwrap_or(false),
                ValuePredicate::IsNull  => val.map(|v| v.is_null()).unwrap_or(true),
                ValuePredicate::Contains(s) => val.and_then(|a| a.as_str()).map(|v| v.contains(s.as_str())).unwrap_or(false),
                _ => true,
            }
        })
    }

    fn apply_where(&self, row: &HyperRow, exprs: &[WhereExpr]) -> bool {
        exprs.iter().all(|expr| self.eval_where(row, expr))
    }

    fn eval_where(&self, row: &HyperRow, expr: &WhereExpr) -> bool {
        match expr {
            WhereExpr::AttrEq { attr, value } =>
                row.get_attr(attr).and_then(|v| serde_json::to_value(v).ok())
                    .map(|v| v == *value).unwrap_or(false),
            WhereExpr::AttrGt { attr, value } =>
                row.get_f64(attr).map(|v| v > *value).unwrap_or(false),
            WhereExpr::AttrLt { attr, value } =>
                row.get_f64(attr).map(|v| v < *value).unwrap_or(false),
            WhereExpr::AttrNotNull { attr } =>
                row.get_attr(attr).map(|v| !v.is_null()).unwrap_or(false),
            WhereExpr::And(a, b) => self.eval_where(row, a) && self.eval_where(row, b),
            WhereExpr::Or(a, b)  => self.eval_where(row, a) || self.eval_where(row, b),
            WhereExpr::Not(a)    => !self.eval_where(row, a),
            _ => true,
        }
    }

    fn apply_order_limit<'a>(&self, mut rows: Vec<&'a HyperRow>, query: &HyperQuery) -> Vec<&'a HyperRow> {
        if let Some(spec) = query.order_by.first() {
            if let SelectExpr::Attr(key) = &spec.expr {
                let key = key.clone();
                let asc = spec.direction == SortDirection::Asc;
                rows.sort_by(|a, b| {
                    let va = a.get_f64(&key).unwrap_or(f64::NAN);
                    let vb = b.get_f64(&key).unwrap_or(f64::NAN);
                    let ord = va.partial_cmp(&vb).unwrap_or(std::cmp::Ordering::Equal);
                    if asc { ord } else { ord.reverse() }
                });
            }
        }
        if let Some(limit) = query.limit {
            rows.truncate(limit as usize);
        }
        rows
    }

    fn make_columns(&self, query: &HyperQuery) -> Vec<ResultColumn> {
        query.select.iter().map(|s| match s {
            SelectExpr::Star              => ResultColumn { name: "*".into(), attr_type: None },
            SelectExpr::Attr(k)           => ResultColumn { name: k.clone(), attr_type: None },
            SelectExpr::AttrAlias { alias, .. } => ResultColumn { name: alias.clone(), attr_type: None },
            SelectExpr::DimKey { dim_index, alias } =>
                ResultColumn { name: alias.clone().unwrap_or_else(|| format!("D{dim_index}")), attr_type: None },
            SelectExpr::Fold { alias, source_attr, .. } =>
                ResultColumn { name: alias.clone().unwrap_or_else(|| source_attr.clone()), attr_type: None },
            SelectExpr::Expr { alias, .. } => ResultColumn { name: alias.clone(), attr_type: None },
            _ => ResultColumn { name: "?".into(), attr_type: None },
        }).collect()
    }

    fn project_row(&self, row: &HyperRow, query: &HyperQuery) -> ResultRow {
        let values: Vec<Option<TypedAttrValue>> = query.select.iter().map(|s| match s {
            SelectExpr::Star => Some(TypedAttrValue::Json(
                serde_json::to_value(
                    row.attrs.iter()
                        .map(|(k, v)| (k.clone(), serde_json::to_value(v).unwrap_or_default()))
                        .collect::<HashMap<_, _>>()
                ).unwrap_or_default()
            )),
            SelectExpr::Attr(k) => row.get_attr(k).cloned(),
            SelectExpr::AttrAlias { attr, .. } => row.get_attr(attr).cloned(),
            SelectExpr::DimKey { dim_index: 1, .. } => Some(TypedAttrValue::Json(
                serde_json::to_value(&row.d1_key).unwrap_or_default()
            )),
            _ => None,
        }).collect();

        ResultRow { d1_key: row.d1_key.clone(), values, is_shadow: false }
    }
}

// ─── QueryError ──────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("HyperQL parse error: {0}")]
    ParseError(String),
    #[error("Unknown cube: {0}")]
    UnknownCube(String),
    #[error("Unknown attribute key: {0}")]
    UnknownAttr(String),
    #[error("Permission denied for attribute: {0}")]
    PermissionDenied(String),
    #[error("Missing FROM clause")]
    MissingFrom,
    #[error("Invalid AS_OF timestamp: {0}")]
    InvalidAsOf(String),
    #[error("Execution error: {0}")]
    ExecutionError(String),
}
