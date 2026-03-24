// =============================================================================
// hypergrid::comp — HG-COMP: Computation Engine
//
// ComputationEngine (Tier-1 formula evaluator), Tier2AIPipeline,
// WritebackService, AnomalyEngine, AnomalyState
// =============================================================================

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::cell::{
    AggFn, AttributeKey, CubeId, DimKey, FormulaExpr, HyperRow,
    PermissionTier, TypedAttrValue,
};
use crate::crdt::{CrdtMergeEngine, CrdtOperation, SerializableCoord, VectorClock};
use crate::error::{HypergridError, PluginError};
use crate::graph::{EdgeDirection, EdgeType};
use crate::plugin::{
    AIComputeQueue, AIComputeRequest, ComputeContext, ComputePriority,
    ComputeResult, EventKind, HypercubePlugin, NeighborCache, PluginRegistry,
};

// =============================================================================
// Tier-1: Synchronous Formula Evaluator
// =============================================================================

/// Evaluates Tier-1 FormulaExpr trees synchronously on every read.
/// Always returns a fresh value — no caching; extremely fast (<1ms).
pub struct ComputationEngine;

impl ComputationEngine {
    /// Evaluate a FormulaExpr against one HyperRow.
    pub fn evaluate_formula(
        &self,
        expr: &FormulaExpr,
        row: &HyperRow,
    ) -> TypedAttrValue {
        match expr {
            // ── Leaf nodes ───────────────────────────────────────────────
            FormulaExpr::Attr(key) =>
                row.get_attr(key).cloned().unwrap_or(TypedAttrValue::Null),

            FormulaExpr::Literal(val) => val.clone(),

            FormulaExpr::Null => TypedAttrValue::Null,

            // ── Arithmetic ───────────────────────────────────────────────
            FormulaExpr::Add(a, b) => {
                let va = self.evaluate_formula(a, row).as_f64().unwrap_or(0.0);
                let vb = self.evaluate_formula(b, row).as_f64().unwrap_or(0.0);
                TypedAttrValue::Number(va + vb)
            }
            FormulaExpr::Sub(a, b) => {
                let va = self.evaluate_formula(a, row).as_f64().unwrap_or(0.0);
                let vb = self.evaluate_formula(b, row).as_f64().unwrap_or(0.0);
                TypedAttrValue::Number(va - vb)
            }
            FormulaExpr::Mul(a, b) => {
                let va = self.evaluate_formula(a, row).as_f64().unwrap_or(0.0);
                let vb = self.evaluate_formula(b, row).as_f64().unwrap_or(0.0);
                TypedAttrValue::Number(va * vb)
            }
            FormulaExpr::Div(a, b) => {
                let va = self.evaluate_formula(a, row).as_f64().unwrap_or(0.0);
                let vb = self.evaluate_formula(b, row).as_f64().unwrap_or(0.0);
                if vb == 0.0 { TypedAttrValue::Null }
                else          { TypedAttrValue::Number(va / vb) }
            }

            // ── Comparison ───────────────────────────────────────────────
            FormulaExpr::Eq(a, b) => {
                let va = self.evaluate_formula(a, row);
                let vb = self.evaluate_formula(b, row);
                TypedAttrValue::Bool(va == vb)
            }
            FormulaExpr::Lt(a, b) => {
                let va = self.evaluate_formula(a, row).as_f64().unwrap_or(0.0);
                let vb = self.evaluate_formula(b, row).as_f64().unwrap_or(0.0);
                TypedAttrValue::Bool(va < vb)
            }
            FormulaExpr::Gt(a, b) => {
                let va = self.evaluate_formula(a, row).as_f64().unwrap_or(0.0);
                let vb = self.evaluate_formula(b, row).as_f64().unwrap_or(0.0);
                TypedAttrValue::Bool(va > vb)
            }

            // ── Logic ────────────────────────────────────────────────────
            FormulaExpr::And(a, b) => {
                let va = self.eval_bool(a, row);
                let vb = self.eval_bool(b, row);
                TypedAttrValue::Bool(va && vb)
            }
            FormulaExpr::Or(a, b) => {
                let va = self.eval_bool(a, row);
                let vb = self.eval_bool(b, row);
                TypedAttrValue::Bool(va || vb)
            }
            FormulaExpr::Not(a) => {
                TypedAttrValue::Bool(!self.eval_bool(a, row))
            }
            FormulaExpr::If { condition, then_expr, else_expr } => {
                if self.eval_bool(condition, row) {
                    self.evaluate_formula(then_expr, row)
                } else {
                    self.evaluate_formula(else_expr, row)
                }
            }

            // ── Built-in functions ───────────────────────────────────────
            FormulaExpr::Now => TypedAttrValue::DateTime(Utc::now()),

            FormulaExpr::Coalesce(exprs) => {
                exprs.iter()
                    .map(|e| self.evaluate_formula(e, row))
                    .find(|v| !v.is_null())
                    .unwrap_or(TypedAttrValue::Null)
            }

            FormulaExpr::Concat(exprs) => {
                let s: String = exprs.iter()
                    .filter_map(|e| {
                        let v = self.evaluate_formula(e, row);
                        v.as_str().map(|s| s.to_owned())
                    })
                    .collect();
                TypedAttrValue::Text(s)
            }

            FormulaExpr::Ratio { numerator, denominator } => {
                let n = self.evaluate_formula(numerator, row).as_f64().unwrap_or(0.0);
                let d = self.evaluate_formula(denominator, row).as_f64().unwrap_or(0.0);
                if d == 0.0 { TypedAttrValue::Null } else { TypedAttrValue::Number(n / d) }
            }

            FormulaExpr::DateDiff { a, b, unit } => {
                // Stub: full implementation computes DateTime difference in requested unit
                TypedAttrValue::Number(0.0)
            }

            // ── Graph aggregation ────────────────────────────────────────
            // NOTE: In-memory implementation; production uses Hypergraph traversal
            FormulaExpr::GraphAggregate { attr, agg_fn, .. } => {
                // Stub: in production traverses Hypergraph, collects attr values, applies agg
                TypedAttrValue::Number(0.0)
            }

            // ── Dimension fold ───────────────────────────────────────────
            FormulaExpr::DimFold { source_attr, agg_fn, .. } => {
                // Stub: in production folds over D₃+ axis keys for this D₁ entity
                TypedAttrValue::Number(0.0)
            }

            // ── Cross-cube lookup ────────────────────────────────────────
            FormulaExpr::Lookup { cube_name, d1_key_expr, d2_key } => {
                // Stub: in production: resolve cube, look up cell, return value
                TypedAttrValue::Null
            }
        }
    }

    /// Evaluate a formula as a boolean (for IF conditions).
    pub fn eval_bool(&self, expr: &FormulaExpr, row: &HyperRow) -> bool {
        match self.evaluate_formula(expr, row) {
            TypedAttrValue::Bool(b)   => b,
            TypedAttrValue::Number(n) => n != 0.0,
            TypedAttrValue::Null      => false,
            _                         => true,
        }
    }

    /// Aggregate a set of numeric values using an AggFn.
    pub fn aggregate(values: &[f64], agg: &AggFn) -> TypedAttrValue {
        if values.is_empty() {
            return TypedAttrValue::Null;
        }
        let result = match agg {
            AggFn::Sum   => values.iter().sum(),
            AggFn::Avg   => values.iter().sum::<f64>() / values.len() as f64,
            AggFn::Min   => values.iter().cloned().fold(f64::INFINITY, f64::min),
            AggFn::Max   => values.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            AggFn::Count => values.len() as f64,
            AggFn::First => values[0],
            AggFn::Last  => *values.last().unwrap(),
            AggFn::Median => {
                let mut sorted = values.to_vec();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let mid = sorted.len() / 2;
                if sorted.len() % 2 == 0 { (sorted[mid-1] + sorted[mid]) / 2.0 } else { sorted[mid] }
            }
            AggFn::StdDev => {
                let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;
                let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
                variance.sqrt()
            }
            AggFn::Percentile(p) => {
                let mut sorted = values.to_vec();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let idx = (*p as f64 / 100.0 * (sorted.len() - 1) as f64).round() as usize;
                sorted.get(idx).copied().unwrap_or(0.0)
            }
            _ => values.iter().sum(), // fallback
        };
        TypedAttrValue::Number(result)
    }
}

// =============================================================================
// Tier-2: Asynchronous AI Compute Pipeline
// =============================================================================

/// Manages the lifecycle of Tier-2 AI compute requests.
/// Triggers → Priority Queue → Worker Pool → WritebackService → Cache update.
pub struct Tier2AIPipeline {
    pub queue:           Arc<AIComputeQueue>,
    pub plugin_registry: Arc<PluginRegistry>,
    pub writeback:       Arc<WritebackService>,
}

impl Tier2AIPipeline {
    pub fn new(
        plugin_registry: Arc<PluginRegistry>,
        writeback: Arc<WritebackService>,
    ) -> Self {
        Self {
            queue: Arc::new(AIComputeQueue::new()),
            plugin_registry,
            writeback,
        }
    }

    /// Called after a cell mutation (Write Path Step 8a).
    /// Dispatches compute requests for all affected AI attributes.
    pub fn on_cell_mutation(
        &self,
        cube_id: CubeId,
        coord: &crate::cell::DimCoordinate,
        current_vc: VectorClock,
    ) {
        let affected = self.plugin_registry.on_cell_mutation(cube_id, coord);
        for attr_key in affected {
            self.queue.push(AIComputeRequest {
                request_id:  Uuid::new_v4(),
                plugin_id:   self.find_plugin_for_attr(cube_id, &attr_key),
                attr_key,
                cube_id,
                d1_key:      coord.d1().cloned().unwrap_or(DimKey::Null),
                priority:    ComputePriority::Normal,
                requested_at: Utc::now(),
                observed_vc: current_vc.clone(),
            });
        }
    }

    fn find_plugin_for_attr(&self, _cube_id: CubeId, _attr_key: &str) -> String {
        // In production: look up AttrComputation::AiEngine.plugin_id in AttributeKeyRegistry
        String::new()
    }

    /// Process one request from the compute queue. Called by worker pool threads.
    pub fn process_one(
        &self,
        row_store: &HashMap<(CubeId, String), HyperRow>,
        node_id: &str,
    ) -> Option<WritebackRequest> {
        let req = self.queue.pop()?;

        // Staleness check: if the request's observed VC is older than the current cell VC,
        // discard it — a newer request is already in the queue.
        // (In production: compare against cell_store.get_vector_clock(req.coord, req.attr_key))

        // Build ComputeContext
        let row_key = (req.cube_id, format!("{:?}", req.d1_key));
        let row = row_store.get(&row_key)?;
        let ctx = ComputeContext::new(row, req.cube_id);

        // Invoke plugin
        let plugin = self.plugin_registry.get(&req.plugin_id)?;
        // Cast to ComputedModelPlugin — in production use a typed registry
        let result = TypedAttrValue::AiSignal {
            value:       Box::new(TypedAttrValue::Number(0.0)), // placeholder
            model_id:    req.plugin_id.clone(),
            computed_at: Utc::now(),
            confidence:  0.0,
            explanation: None,
            stale_at:    None,
        };

        let coord = SerializableCoord {
            grid_id: req.d1_key.as_uuid().map(|_| Uuid::nil()).unwrap_or(Uuid::nil()),
            cube_id: req.cube_id,
            keys:    vec![req.d1_key.clone(), DimKey::text(&req.attr_key)],
        };

        Some(WritebackRequest {
            coord,
            attr_key:   req.attr_key,
            value:      result,
            actor:      format!("system:{}", req.plugin_id),
            permission: PermissionTier::System,
            ttl_secs:   3600,
        })
    }

    pub fn queue_depth(&self) -> usize { self.queue.len() }
}

// ─── WritebackRequest ────────────────────────────────────────────────────────

/// A request to write an AI-computed value back into the HyperCell store.
/// All Tier-2 AI writes go through this path (bypasses user permission check).
#[derive(Debug, Clone)]
pub struct WritebackRequest {
    pub coord:      SerializableCoord,
    pub attr_key:   AttributeKey,
    pub value:      TypedAttrValue,
    pub actor:      String,            // "system:{plugin_id}"
    pub permission: PermissionTier,    // always System for AI writes
    pub ttl_secs:   u64,               // Redis cache TTL
}

// ─── WritebackService ────────────────────────────────────────────────────────

/// The sole channel through which AI engines write computed values into HyperCells.
/// Validates engine identity, enforces System permission, appends to EventLog.
///
/// In production this is a gRPC service at port 9009.
pub struct WritebackService {
    /// In-memory writeback log (production: PostgreSQL via write path).
    pub log: Mutex<Vec<WritebackRecord>>,
}

#[derive(Debug, Clone)]
pub struct WritebackRecord {
    pub coord:       SerializableCoord,
    pub attr_key:    AttributeKey,
    pub value:       TypedAttrValue,
    pub engine_id:   String,
    pub written_at:  DateTime<Utc>,
    pub confidence:  f64,
}

impl WritebackService {
    pub fn new() -> Self { Self { log: Mutex::new(Vec::new()) } }

    /// Write an AI-computed value.
    /// In production: validate engine identity, call write path Steps 4–8.
    pub fn write_ai_result(&self, req: WritebackRequest) -> Result<(), HypergridError> {
        let confidence = match &req.value {
            TypedAttrValue::AiSignal { confidence, .. } => *confidence,
            _ => 1.0,
        };

        self.log.lock().unwrap().push(WritebackRecord {
            coord:      req.coord,
            attr_key:   req.attr_key,
            value:      req.value,
            engine_id:  req.actor.strip_prefix("system:").unwrap_or("").to_owned(),
            written_at: Utc::now(),
            confidence,
        });
        Ok(())
    }

    pub fn recent_writes(&self) -> Vec<WritebackRecord> {
        self.log.lock().unwrap().clone()
    }
}

// =============================================================================
// Anomaly Detection Engine
// =============================================================================

/// Monitors numeric attribute cells and fires AnomalySignals when values
/// deviate significantly from established baselines.
///
/// Uses Exponential Moving Average (EMA) and Exponential Moving StdDev (EMSTD).
pub struct AnomalyEngine {
    pub threshold:  f64,     // z-score threshold for anomaly detection (default: 2.0)
    pub alpha:      f64,     // EMA smoothing factor (default: 0.1)
    baselines: Mutex<HashMap<AnomalyKey, AnomalyState>>,
    pub subscriptions: Vec<AnomalySubscription>,
}

/// (cube_id, D₁ key string, attr_key) → baseline state.
type AnomalyKey = (CubeId, String, AttributeKey);

#[derive(Debug, Clone)]
pub struct AnomalyState {
    pub baseline: f64,        // EMA of recent values
    pub stddev:   f64,        // EMSTD of recent values
    pub n:        u64,        // number of observations
    pub history:  Vec<f64>,   // last N values (for pattern detection)
}

impl AnomalyState {
    pub fn new(first_value: f64) -> Self {
        Self { baseline: first_value, stddev: 0.0, n: 1, history: vec![first_value] }
    }

    pub fn z_score(&self, value: f64) -> f64 {
        if self.stddev < f64::EPSILON { return 0.0; }
        (value - self.baseline).abs() / self.stddev
    }
}

/// Configuration for which attributes to monitor.
#[derive(Debug, Clone)]
pub struct AnomalySubscription {
    pub cube_id:    CubeId,
    pub attr_key:   AttributeKey,
    pub kind:       AnomalyKindFilter,
}

#[derive(Debug, Clone)]
pub enum AnomalyKindFilter { All, SuddenChange, PatternBreak, OutlierValue, StateChange }

impl AnomalyEngine {
    pub fn new() -> Self {
        Self {
            threshold:     2.0,
            alpha:         0.1,
            baselines:     Mutex::new(HashMap::new()),
            subscriptions: vec![],
        }
    }

    pub fn subscribe(&mut self, cube_id: CubeId, attr_key: impl Into<String>) {
        self.subscriptions.push(AnomalySubscription {
            cube_id,
            attr_key: attr_key.into(),
            kind: AnomalyKindFilter::All,
        });
    }

    /// Process a new observed value. Returns an AnomalySignal if anomalous.
    pub fn observe(
        &self,
        cube_id: CubeId,
        d1_key_str: &str,
        attr_key: &str,
        value: f64,
    ) -> Option<TypedAttrValue> {
        let key = (cube_id, d1_key_str.to_owned(), attr_key.to_owned());
        let mut baselines = self.baselines.lock().unwrap();

        let state = baselines.entry(key).or_insert_with(|| AnomalyState::new(value));

        // Update history (keep last 30 observations for pattern detection)
        state.history.push(value);
        if state.history.len() > 30 { state.history.remove(0); }

        let z = state.z_score(value);
        let anomaly = if z > self.threshold {
            let kind = self.classify_anomaly(value, state);
            let severity = z / self.threshold;  // 1.0 = threshold, >1 = worse

            Some(TypedAttrValue::AnomalySignal {
                kind,
                severity,
                baseline: Box::new(TypedAttrValue::Number(state.baseline)),
                deviation: value - state.baseline,
            })
        } else {
            None
        };

        // Always update baseline (whether anomaly or not)
        state.n += 1;
        let old_baseline = state.baseline;
        state.baseline = self.alpha * value + (1.0 - self.alpha) * state.baseline;
        state.stddev = (self.alpha * (value - old_baseline).powi(2)
            + (1.0 - self.alpha) * state.stddev.powi(2)).sqrt();

        anomaly
    }

    fn classify_anomaly(&self, value: f64, state: &AnomalyState) -> crate::cell::AnomalyKind {
        use crate::cell::AnomalyKind;

        if state.history.len() < 3 {
            return AnomalyKind::OutlierValue;
        }

        let recent = &state.history[state.history.len().saturating_sub(3)..];
        let sustained = recent.iter().all(|&v| (v - state.baseline).abs() > state.stddev * self.threshold);

        if sustained {
            AnomalyKind::PatternBreak
        } else if (value - state.baseline).abs() > state.stddev * 3.0 {
            AnomalyKind::SuddenChange
        } else {
            AnomalyKind::OutlierValue
        }
    }

    /// Detect anomalies for all subscribed attributes in a batch of rows.
    pub fn scan_batch(
        &self,
        cube_id: CubeId,
        rows: &[HyperRow],
        attr_key: &str,
    ) -> Vec<(DimKey, TypedAttrValue)> {
        rows.iter()
            .filter_map(|row| {
                let value = row.get_f64(attr_key)?;
                let d1_str = format!("{:?}", row.d1_key);
                let signal = self.observe(cube_id, &d1_str, attr_key, value)?;
                Some((row.d1_key.clone(), signal))
            })
            .collect()
    }
}

// ─── Predefined Anomaly Subscriptions ────────────────────────────────────────

/// Returns the standard anomaly subscriptions for a Kogi Grid.
pub fn kogi_anomaly_subscriptions(components_cube: CubeId) -> Vec<AnomalySubscription> {
    vec![
        AnomalySubscription { cube_id: components_cube, attr_key: "budget_spent".into(),          kind: AnomalyKindFilter::SuddenChange },
        AnomalySubscription { cube_id: components_cube, attr_key: "health_score".into(),           kind: AnomalyKindFilter::PatternBreak },
        AnomalySubscription { cube_id: components_cube, attr_key: "income_projection_90d".into(),  kind: AnomalyKindFilter::SuddenChange },
    ]
}

/// Returns the standard anomaly subscriptions for a Qala Grid.
pub fn qala_anomaly_subscriptions(solutions_cube: CubeId, sdes_cube: CubeId) -> Vec<AnomalySubscription> {
    vec![
        AnomalySubscription { cube_id: solutions_cube, attr_key: "defect_density".into(),   kind: AnomalyKindFilter::PatternBreak },
        AnomalySubscription { cube_id: sdes_cube,      attr_key: "drift_status".into(),      kind: AnomalyKindFilter::StateChange },
        AnomalySubscription { cube_id: solutions_cube, attr_key: "quality_score".into(),     kind: AnomalyKindFilter::SuddenChange },
    ]
}
