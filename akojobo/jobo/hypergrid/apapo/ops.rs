// =============================================================================
// hypergrid::ops — HG-OPS: Observability, EventLog, PolicyEngine
//
// EventLog, EventEntry, PolicyEngine, PolicyContext, PolicyDecision,
// GridStatus, GridStats, HG-OPS metrics stubs
// =============================================================================

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::cell::{ActorId, AttributeKey, CubeId, DimKey, GridId, PermissionTier, TypedAttrValue};
use crate::crdt::{CrdtOperation, VectorClock};
use crate::error::HypergridError;

// ─── GridStatus ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GridStatus {
    /// Startup sequence in progress — no requests served.
    Initializing,
    /// All subsystems operational — full request serving.
    Ready,
    /// One or more non-critical subsystems failed.
    /// Core reads/writes served; AI computed attributes and federation disabled.
    Degraded { reason: String },
    /// Graceful shutdown — no new requests; in-flight requests completing.
    ShuttingDown,
}

impl Default for GridStatus {
    fn default() -> Self { Self::Initializing }
}

impl GridStatus {
    pub fn is_ready(&self) -> bool { matches!(self, Self::Ready) }
    pub fn accepts_requests(&self) -> bool {
        matches!(self, Self::Ready | Self::Degraded { .. })
    }
}

// ─── GridStats ───────────────────────────────────────────────────────────────

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GridStats {
    pub grid_id:        GridId,
    pub total_cells:    u64,
    pub total_rows:     u64,
    pub cube_count:     u64,
    pub event_count:    u64,
    pub crdt_op_count:  u64,
    pub edge_count:     u64,
    pub space_count:    u64,
    pub snapshot_at:    Option<DateTime<Utc>>,
}

// =============================================================================
// EventLog
// =============================================================================

/// The crdt_op kind stored in EventLog entries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrdtOpKind {
    LWW,
    OrSetAdd,
    OrSetRemove,
    Counter,
    Lattice,
    LatticeAdminOverride,
    AppendLog,
    DeepMerge,
    SchemaChange,
    GraphEdgeAdded,
    GraphEdgeRemoved,
    NamespaceRegistered,
    ShadowSync,
    FederationSync,
    Rollback,
    Custom(String),
}

/// One immutable entry in the EventLog.
/// NEVER updated or deleted — the audit trail is sacred.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEntry {
    pub event_id:     Uuid,
    pub seq:          u64,              // Grid-wide monotonic sequence number
    pub cube_id:      CubeId,
    pub dim_keys:     Vec<DimKey>,      // coordinate of the affected cell
    pub attr_key:     AttributeKey,
    pub before_value: Option<TypedAttrValue>,
    pub after_value:  TypedAttrValue,
    pub crdt_op:      CrdtOpKind,
    pub vector_clock: VectorClock,
    pub actor:        ActorId,
    pub timestamp:    DateTime<Utc>,
    pub domain_tag:   Option<String>,   // e.g. "portfolio:ComponentStatusChanged"
}

impl EventEntry {
    pub fn set_attr(
        cube_id: CubeId, dim_keys: Vec<DimKey>, attr_key: impl Into<String>,
        before: Option<TypedAttrValue>, after: TypedAttrValue,
        vc: VectorClock, actor: impl Into<String>, seq: u64,
    ) -> Self {
        Self {
            event_id:     Uuid::new_v4(),
            seq,
            cube_id,
            dim_keys,
            attr_key:     attr_key.into(),
            before_value: before,
            after_value:  after,
            crdt_op:      CrdtOpKind::LWW,
            vector_clock: vc,
            actor:        actor.into(),
            timestamp:    Utc::now(),
            domain_tag:   None,
        }
    }

    pub fn with_domain_tag(mut self, tag: impl Into<String>) -> Self {
        self.domain_tag = Some(tag.into());
        self
    }
}

/// The EventLog: append-only, immutable, time-indexed history of every mutation.
///
/// Architecture:
///   Hot buffer: in-memory VecDeque (capped at HOT_CAP=10,000 entries)
///   Cold storage: PostgreSQL event_log table (flushed from hot buffer)
///   Archive: S3 Parquet (monthly partitions when PostgreSQL table is large)
///   Kafka: every entry also published to "events.audit.{grid_id}"
pub struct EventLog {
    pub grid_id: GridId,
    hot:         Mutex<VecDeque<EventEntry>>,
    pub seq:     AtomicU64,
    hot_cap:     usize,
    total:       AtomicU64,
}

const HOT_CAP: usize = 10_000;

impl EventLog {
    pub fn new(grid_id: GridId) -> Self {
        Self {
            grid_id,
            hot:     Mutex::new(VecDeque::new()),
            seq:     AtomicU64::new(0),
            hot_cap: HOT_CAP,
            total:   AtomicU64::new(0),
        }
    }

    /// Append an entry. In production also flushes to PostgreSQL and Kafka.
    pub fn append(&self, mut entry: EventEntry) -> Result<Uuid, HypergridError> {
        let seq = self.seq.fetch_add(1, Ordering::SeqCst);
        entry.seq = seq;
        let id = entry.event_id;

        let mut hot = self.hot.lock().unwrap();
        hot.push_back(entry);
        self.total.fetch_add(1, Ordering::Relaxed);

        // Hot buffer overflow: in production flush oldest 100 to PostgreSQL
        if hot.len() > self.hot_cap {
            hot.pop_front();
        }

        Ok(id)
    }

    /// Read events for one entity from the hot buffer.
    pub fn entries_for_entity(
        &self, cube_id: CubeId, d1_key: &DimKey,
    ) -> Vec<EventEntry> {
        let d1_str = format!("{:?}", d1_key);
        self.hot.lock().unwrap().iter()
            .filter(|e| e.cube_id == cube_id &&
                e.dim_keys.first().map(|k| format!("{:?}", k) == d1_str).unwrap_or(false))
            .cloned()
            .collect()
    }

    /// Read events in a timestamp range (for AS_OF time-travel).
    pub fn entries_in_range(
        &self, cube_id: CubeId, d1_key: &DimKey,
        from: DateTime<Utc>, to: DateTime<Utc>,
    ) -> Vec<EventEntry> {
        self.entries_for_entity(cube_id, d1_key)
            .into_iter()
            .filter(|e| e.timestamp >= from && e.timestamp <= to)
            .collect()
    }

    pub fn len(&self) -> u64 { self.total.load(Ordering::Relaxed) }
    pub fn hot_len(&self) -> usize { self.hot.lock().unwrap().len() }

    /// AS_OF time-travel: replay EventLog to reconstruct entity state at target timestamp.
    pub fn replay_to(
        &self,
        cube_id: CubeId,
        d1_key: &DimKey,
        target: DateTime<Utc>,
    ) -> Vec<(AttributeKey, TypedAttrValue)> {
        let events = self.entries_in_range(
            cube_id, d1_key, DateTime::<Utc>::MIN_UTC, target
        );

        let mut state: std::collections::HashMap<AttributeKey, TypedAttrValue> = HashMap::new();

        for event in events {
            match event.crdt_op {
                CrdtOpKind::LWW | CrdtOpKind::Lattice | CrdtOpKind::DeepMerge => {
                    state.insert(event.attr_key, event.after_value);
                }
                CrdtOpKind::OrSetAdd => {
                    // OR-Set: accumulate into a JSON array
                    let arr = state.entry(event.attr_key)
                        .or_insert_with(|| TypedAttrValue::Json(serde_json::json!([])));
                    if let TypedAttrValue::Json(serde_json::Value::Array(ref mut a)) = arr {
                        if let Ok(v) = serde_json::to_value(&event.after_value) {
                            a.push(v);
                        }
                    }
                }
                CrdtOpKind::Counter => {
                    // PN-Counter: accumulate
                    let current = state.entry(event.attr_key.clone())
                        .or_insert(TypedAttrValue::Number(0.0));
                    let cur_val = current.as_f64().unwrap_or(0.0);
                    let delta = event.after_value.as_f64().unwrap_or(0.0);
                    *current = TypedAttrValue::Number(cur_val + delta);
                }
                CrdtOpKind::Rollback => {
                    // Re-apply pre-rollback value
                    if let Some(before) = event.before_value {
                        state.insert(event.attr_key, before);
                    }
                }
                _ => {}
            }
        }

        state.into_iter().collect()
    }
}

use std::collections::HashMap;

// =============================================================================
// PolicyEngine
// =============================================================================

/// The result of a policy evaluation.
#[derive(Debug, Clone)]
pub enum PolicyDecision {
    Allow,
    Deny  { reason: String },
    RequireApproval { reason: String, approvers: Vec<crate::cell::UserId> },
}

/// Context passed to policy evaluation.
pub struct PolicyContext<'a> {
    pub actor:    &'a str,
    pub cube_id:  CubeId,
    pub coord:    &'a crate::cell::DimCoordinate,
    pub attr_key: &'a str,
    pub value:    &'a TypedAttrValue,
    pub tier:     PermissionTier,
}

/// A registered policy rule.
#[derive(Debug, Clone)]
pub struct PolicyRule {
    pub policy_id:   Uuid,
    pub name:        String,
    pub description: String,
    pub rule_fn:     PolicyRuleFn,
}

/// A policy rule function. Returns Allow / Deny / RequireApproval.
/// In production these would be Rego (OPA) policies or Rust closures.
pub type PolicyRuleFn = Arc<dyn Fn(&PolicyContext) -> PolicyDecision + Send + Sync>;

use std::sync::Arc;

/// Evaluates PolicyRules attached to entities.
/// Every write passes through PolicyEngine after PermissionTier check.
pub struct PolicyEngine {
    rules: Mutex<Vec<PolicyRule>>,
}

impl PolicyEngine {
    pub fn new() -> Self { Self { rules: Mutex::new(Vec::new()) } }

    pub fn register_rule(&self, rule: PolicyRule) {
        self.rules.lock().unwrap().push(rule);
    }

    /// Evaluate all registered rules for a write context.
    /// Returns the most restrictive decision: Deny > RequireApproval > Allow.
    pub fn evaluate(&self, ctx: &PolicyContext) -> PolicyDecision {
        let rules = self.rules.lock().unwrap();
        let mut final_decision = PolicyDecision::Allow;

        for rule in rules.iter() {
            match (rule.rule_fn)(ctx) {
                PolicyDecision::Deny { reason } => {
                    return PolicyDecision::Deny { reason };  // Deny is immediate
                }
                PolicyDecision::RequireApproval { reason, approvers } => {
                    // RequireApproval upgrades Allow but yields to Deny
                    if matches!(final_decision, PolicyDecision::Allow) {
                        final_decision = PolicyDecision::RequireApproval { reason, approvers };
                    }
                }
                PolicyDecision::Allow => {}
            }
        }
        final_decision
    }
}

// ─── Standard policy factories ───────────────────────────────────────────────

impl PolicyEngine {
    /// Block AI-tier writes from non-system actors.
    pub fn register_ai_write_protection(&self) {
        self.register_rule(PolicyRule {
            policy_id: Uuid::new_v4(),
            name: "AI write protection".into(),
            description: "AI-computed attributes may only be written by System-tier actors".into(),
            rule_fn: Arc::new(|ctx| {
                let is_ai_attr = ctx.attr_key.ends_with("_score") ||
                    ctx.attr_key.ends_with("_flag") ||
                    ctx.attr_key == "anomaly_flag" ||
                    ctx.attr_key == "drift_status";
                if is_ai_attr && ctx.tier < PermissionTier::System {
                    PolicyDecision::Deny {
                        reason: format!("Attribute '{}' may only be written by the AI engine (System tier)", ctx.attr_key),
                    }
                } else {
                    PolicyDecision::Allow
                }
            }),
        });
    }

    /// Require Manager+ for budget mutations.
    pub fn register_budget_protection(&self) {
        self.register_rule(PolicyRule {
            policy_id: Uuid::new_v4(),
            name: "Budget write protection".into(),
            description: "Budget fields require Manager+ permission tier".into(),
            rule_fn: Arc::new(|ctx| {
                let is_budget = ctx.attr_key == "budget" || ctx.attr_key == "budget_allocated";
                if is_budget && ctx.tier < PermissionTier::Manager {
                    PolicyDecision::Deny {
                        reason: "Budget modifications require Manager tier or above".into(),
                    }
                } else {
                    PolicyDecision::Allow
                }
            }),
        });
    }
}

// =============================================================================
// HG-OPS: Observability stubs
// =============================================================================

/// OpenTelemetry span stub. In production: otlp exporter to Jaeger / Tempo.
#[derive(Debug)]
pub struct Span {
    pub name:       String,
    pub trace_id:   Uuid,
    pub span_id:    Uuid,
    pub started_at: DateTime<Utc>,
    pub tags:       HashMap<String, String>,
}

impl Span {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            trace_id:   Uuid::new_v4(),
            span_id:    Uuid::new_v4(),
            started_at: Utc::now(),
            tags:       HashMap::new(),
        }
    }
    pub fn tag(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.tags.insert(k.into(), v.into()); self
    }
    pub fn finish_ms(&self) -> u64 {
        (Utc::now() - self.started_at).num_milliseconds() as u64
    }
}

/// Prometheus-style counter stub.
pub struct Counter {
    name:  &'static str,
    value: AtomicU64,
}

impl Counter {
    pub const fn new(name: &'static str) -> Self { Self { name, value: AtomicU64::new(0) } }
    pub fn inc(&self)        { self.value.fetch_add(1, Ordering::Relaxed); }
    pub fn add(&self, n: u64) { self.value.fetch_add(n, Ordering::Relaxed); }
    pub fn get(&self) -> u64  { self.value.load(Ordering::Relaxed) }
    pub fn name(&self) -> &'static str { self.name }
}

/// Grid-level metrics.
pub struct GridMetrics {
    pub cell_writes:          Counter,
    pub cell_reads:           Counter,
    pub crdt_conflicts:       Counter,
    pub event_log_entries:    Counter,
    pub ai_compute_requests:  Counter,
    pub ai_compute_completed: Counter,
    pub federation_ops_sent:  Counter,
    pub federation_ops_recv:  Counter,
    pub hyperql_queries:      Counter,
    pub shadow_syncs:         Counter,
}

impl GridMetrics {
    pub fn new() -> Self {
        Self {
            cell_writes:          Counter::new("hg.cell.writes"),
            cell_reads:           Counter::new("hg.cell.reads"),
            crdt_conflicts:       Counter::new("hg.crdt.conflicts"),
            event_log_entries:    Counter::new("hg.eventlog.entries"),
            ai_compute_requests:  Counter::new("hg.ai.compute.requests"),
            ai_compute_completed: Counter::new("hg.ai.compute.completed"),
            federation_ops_sent:  Counter::new("hg.federation.ops.sent"),
            federation_ops_recv:  Counter::new("hg.federation.ops.recv"),
            hyperql_queries:      Counter::new("hg.hyperql.queries"),
            shadow_syncs:         Counter::new("hg.shadow.syncs"),
        }
    }

    pub fn snapshot(&self) -> HashMap<&'static str, u64> {
        let mut m = HashMap::new();
        for c in [
            &self.cell_writes, &self.cell_reads, &self.crdt_conflicts,
            &self.event_log_entries, &self.ai_compute_requests,
            &self.ai_compute_completed, &self.federation_ops_sent,
            &self.federation_ops_recv, &self.hyperql_queries, &self.shadow_syncs,
        ] {
            m.insert(c.name(), c.get());
        }
        m
    }
}

// ─── Health probe ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub grid_status:     String,
    pub cell_store:      SubsystemHealth,
    pub event_log:       SubsystemHealth,
    pub crdt_log:        SubsystemHealth,
    pub plugin_registry: SubsystemHealth,
    pub federation:      Vec<FederationHealth>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubsystemHealth {
    pub status:     String,  // "healthy" | "degraded" | "unhealthy"
    pub latency_ms: Option<u64>,
    pub details:    Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationHealth {
    pub peer_id:    Uuid,
    pub status:     String,
    pub lag_ms:     Option<u64>,
    pub last_sync:  Option<DateTime<Utc>>,
}
