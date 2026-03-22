//! Governance: PolicyEngine trait, ApprovalWorkflow, ResourceAllocation.

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::component::Component;
use crate::events::PortfolioEventKind;
use crate::types::{ComponentId, PolicyId, ResourceKind, UserId};

// ── PolicyEngine trait ────────────────────────────────────────────────────────

/// Decision returned by a PolicyEngine evaluation.
#[derive(Debug, Clone, PartialEq)]
pub enum PolicyDecision {
    Allow,
    Deny(String),
    RequireApproval(String),
}

/// Context passed to each PolicyEngine for evaluation.
pub struct PolicyContext<'a> {
    pub component: &'a Component,
    pub event_kind: &'a PortfolioEventKind,
    pub actor: &'a str,
    pub properties: HashMap<String, JsonValue>,
}

impl<'a> PolicyContext<'a> {
    pub fn new(component: &'a Component, event_kind: &'a PortfolioEventKind, actor: &'a str) -> Self {
        Self { component, event_kind, actor, properties: HashMap::new() }
    }
}

/// Any governance rule can be injected as an `Arc<dyn PolicyEngine>`.
/// Engines are evaluated in registration order; the first non-Allow result is returned.
pub trait PolicyEngine: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn evaluate(&self, ctx: &PolicyContext<'_>) -> PolicyDecision;
}

// ── Built-in policy engines ────────────────────────────────────────────────────

/// A policy engine that denies mutations when budget would be exceeded.
pub struct BudgetCapPolicy {
    pub policy_id: PolicyId,
}

impl PolicyEngine for BudgetCapPolicy {
    fn id(&self) -> &str { "budget-cap" }
    fn name(&self) -> &str { "Budget Cap Policy" }

    fn evaluate(&self, ctx: &PolicyContext<'_>) -> PolicyDecision {
        let meta = &ctx.component.metadata;
        if meta.budget > 0.0 && meta.budget_spent > meta.budget {
            return PolicyDecision::Deny(format!(
                "Component {} has exceeded its budget (spent {:.2} of {:.2})",
                meta.id, meta.budget_spent, meta.budget
            ));
        }
        PolicyDecision::Allow
    }
}

/// A policy that requires approval for status transitions to Completed or Archived.
pub struct LifecycleApprovalPolicy;

impl PolicyEngine for LifecycleApprovalPolicy {
    fn id(&self) -> &str { "lifecycle-approval" }
    fn name(&self) -> &str { "Lifecycle Approval Policy" }

    fn evaluate(&self, ctx: &PolicyContext<'_>) -> PolicyDecision {
        match ctx.event_kind {
            PortfolioEventKind::ComponentStatusChanged => {
                let new_status = ctx.properties.get("new_status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if matches!(new_status, "Completed" | "Archived") {
                    return PolicyDecision::RequireApproval(format!(
                        "Transition to {new_status} requires approval"
                    ));
                }
            }
            _ => {}
        }
        PolicyDecision::Allow
    }
}

// ── ApprovalWorkflow ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalStatus {
    Pending, Approved, Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub request_id: Uuid,
    pub component_id: ComponentId,
    pub reason: String,
    pub requested_by: String,   // node ID
    pub requested_at: DateTime<Utc>,
    pub status: ApprovalStatus,
    pub resolved_by: Option<String>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolver_notes: Option<String>,
}

impl ApprovalRequest {
    pub fn new(component_id: ComponentId, reason: impl Into<String>, requested_by: &str) -> Self {
        Self {
            request_id: Uuid::new_v4(),
            component_id,
            reason: reason.into(),
            requested_by: requested_by.to_owned(),
            requested_at: Utc::now(),
            status: ApprovalStatus::Pending,
            resolved_by: None,
            resolved_at: None,
            resolver_notes: None,
        }
    }

    pub fn resolve(&mut self, approved: bool, resolver: &str, notes: Option<String>) {
        self.status = if approved { ApprovalStatus::Approved } else { ApprovalStatus::Rejected };
        self.resolved_by = Some(resolver.to_owned());
        self.resolved_at = Some(Utc::now());
        self.resolver_notes = notes;
    }

    pub fn is_pending(&self) -> bool { self.status == ApprovalStatus::Pending }
}

// ── ResourceAllocation ────────────────────────────────────────────────────────

/// Tracks budget / resource consumption per component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub allocation_id: Uuid,
    pub component_id: ComponentId,
    pub resource_kind: ResourceKind,
    pub total: f64,
    pub consumed: f64,
    pub denomination: String,       // e.g. "USD", "hours", "story_points"
    pub period: Option<String>,     // e.g. "2026-Q2", "FY2026"
    pub created_at: DateTime<Utc>,
}

impl ResourceAllocation {
    pub fn new(
        component_id: ComponentId,
        kind: ResourceKind,
        total: f64,
        denomination: impl Into<String>,
        period: Option<String>,
    ) -> Self {
        Self {
            allocation_id: Uuid::new_v4(),
            component_id,
            resource_kind: kind,
            total,
            consumed: 0.0,
            denomination: denomination.into(),
            period,
            created_at: Utc::now(),
        }
    }

    /// Record consumption. Returns remaining amount.
    pub fn consume(&mut self, amount: f64) -> f64 {
        self.consumed += amount;
        self.remaining()
    }

    pub fn remaining(&self) -> f64 {
        (self.total - self.consumed).max(0.0)
    }

    pub fn is_overrun(&self) -> bool {
        self.consumed > self.total
    }

    pub fn utilisation_pct(&self) -> f64 {
        if self.total == 0.0 { 0.0 } else { self.consumed / self.total * 100.0 }
    }
}

// ── Governance Engine (registry) ──────────────────────────────────────────────

/// Registry of all PolicyEngine instances for a PortfolioSystem.
pub struct GovernanceEngine {
    engines: Vec<Box<dyn PolicyEngine>>,
    pub approval_requests: Vec<ApprovalRequest>,
    pub allocations: HashMap<ComponentId, ResourceAllocation>,
}

impl GovernanceEngine {
    pub fn new() -> Self {
        Self {
            engines: vec![],
            approval_requests: vec![],
            allocations: HashMap::new(),
        }
    }

    pub fn register_policy(&mut self, engine: Box<dyn PolicyEngine>) {
        self.engines.push(engine);
    }

    /// Evaluate all registered policies for the given context.
    /// Returns first non-Allow decision, or Allow if all pass.
    pub fn evaluate(&self, ctx: &PolicyContext<'_>) -> PolicyDecision {
        for engine in &self.engines {
            let decision = engine.evaluate(ctx);
            if decision != PolicyDecision::Allow {
                return decision;
            }
        }
        PolicyDecision::Allow
    }

    pub fn request_approval(&mut self, component_id: ComponentId, reason: &str, by: &str) -> Uuid {
        let req = ApprovalRequest::new(component_id, reason, by);
        let id = req.request_id;
        self.approval_requests.push(req);
        id
    }

    pub fn resolve_approval(&mut self, request_id: Uuid, approved: bool, resolver: &str, notes: Option<String>) -> bool {
        if let Some(req) = self.approval_requests.iter_mut().find(|r| r.request_id == request_id) {
            req.resolve(approved, resolver, notes);
            return true;
        }
        false
    }

    pub fn pending_approvals(&self) -> Vec<&ApprovalRequest> {
        self.approval_requests.iter().filter(|r| r.is_pending()).collect()
    }

    pub fn allocate_resource(
        &mut self,
        component_id: ComponentId,
        kind: ResourceKind,
        total: f64,
        denomination: &str,
        period: Option<String>,
    ) -> Uuid {
        let alloc = ResourceAllocation::new(component_id, kind, total, denomination, period);
        let id = alloc.allocation_id;
        self.allocations.insert(component_id, alloc);
        id
    }

    pub fn allocate_budget(
        &mut self,
        component_id: ComponentId,
        total: f64,
        currency: &str,
        period: Option<String>,
    ) -> Uuid {
        self.allocate_resource(component_id, ResourceKind::Budget, total, currency, period)
    }

    pub fn record_consumption(&mut self, component_id: ComponentId, amount: f64) -> Option<f64> {
        self.allocations.get_mut(&component_id).map(|a| a.consume(amount))
    }

    pub fn get_allocation(&self, component_id: ComponentId) -> Option<&ResourceAllocation> {
        self.allocations.get(&component_id)
    }

    pub fn overrun_allocations(&self) -> Vec<&ResourceAllocation> {
        self.allocations.values().filter(|a| a.is_overrun()).collect()
    }
}

impl Default for GovernanceEngine {
    fn default() -> Self { Self::new() }
}
