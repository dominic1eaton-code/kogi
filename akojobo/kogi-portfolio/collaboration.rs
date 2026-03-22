//! Shared Portfolios & Portfolio Collaboration (§18).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::types::{
    AccountId, ComponentId, ContributionType, EntityId, GovernanceStatus,
    LedgerId, VersionId, Visibility,
};

// ── Contribution Attribution ───────────────────────────────────────────────────

/// Valuation method for a contribution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValuationMethod {
    MarketRate, SelfReported, PeerReview, AlgorithmicEstimate, Fixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionValue {
    pub amount: f64,
    pub unit: String,
    pub valuation_method: ValuationMethod,
}

/// A single contribution record appended to a component's EventLog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionRecord {
    pub record_id: Uuid,
    pub contributor_id: EntityId,
    pub contribution_type: ContributionType,
    pub portfolio_component_id: ComponentId,
    pub contribution_value: ContributionValue,
    pub timestamp: DateTime<Utc>,
    pub version_id: VersionId,
    /// Used in distribution calculations.
    pub attribution_weight: f64,
    pub governance_status: GovernanceStatus,
    /// kogi-bank entry for capital contributions.
    pub linked_ledger_entry: Option<LedgerId>,
}

impl ContributionRecord {
    pub fn new(
        contributor_id: EntityId,
        component_id: ComponentId,
        contribution_type: ContributionType,
        value: ContributionValue,
        attribution_weight: f64,
    ) -> Self {
        Self {
            record_id: Uuid::new_v4(),
            contributor_id,
            contribution_type,
            portfolio_component_id: component_id,
            contribution_value: value,
            timestamp: Utc::now(),
            version_id: Uuid::new_v4(),
            attribution_weight,
            governance_status: GovernanceStatus::PendingReview,
            linked_ledger_entry: None,
        }
    }
}

// ── Review Policy ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewPolicy {
    AutoAccept,
    StewardReview,
    GovernanceVote,
}

// ── Campaign Status ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CampaignStatus {
    Draft, Open, Paused, Closed, Completed,
}

// ── Reward Rule ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardRule {
    pub reward_type: String,    // "attribution_weight" | "equity" | "payment"
    pub value: f64,
    pub currency: Option<String>,
}

// ── Crowdresourcing Campaign ──────────────────────────────────────────────────

/// A CrowdresourcingCampaign opens a portfolio component or program to community contributions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrowdresourcingCampaign {
    pub campaign_id: Uuid,
    pub target_component_id: ComponentId,
    pub contribution_types: Vec<ContributionType>,
    pub open_to: Visibility,
    pub contribution_review: ReviewPolicy,
    pub goal: Option<JsonValue>,
    pub deadline: Option<DateTime<Utc>>,
    pub contributor_rewards: Vec<RewardRule>,
    pub linked_bank_account: Option<AccountId>,
    pub status: CampaignStatus,
    pub created_at: DateTime<Utc>,
}

impl CrowdresourcingCampaign {
    pub fn new(target_component_id: ComponentId, open_to: Visibility) -> Self {
        Self {
            campaign_id: Uuid::new_v4(),
            target_component_id,
            contribution_types: vec![],
            open_to,
            contribution_review: ReviewPolicy::StewardReview,
            goal: None,
            deadline: None,
            contributor_rewards: vec![],
            linked_bank_account: None,
            status: CampaignStatus::Draft,
            created_at: Utc::now(),
        }
    }

    pub fn is_open(&self) -> bool {
        self.status == CampaignStatus::Open
            && self.deadline.map(|d| d > Utc::now()).unwrap_or(true)
    }
}
