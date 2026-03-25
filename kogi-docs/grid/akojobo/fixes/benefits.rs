//! Portable Benefits as Portfolio Items (§19).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{AccountId, BenefitType, PolicyId, ProviderId, TaxTreatment, WorkflowId};

// ── Contribution Source ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionSource {
    pub source_id: Uuid,
    pub source_name: String,        // employer name, client name, self, etc.
    pub amount_per_period: f64,
    pub currency: String,
    pub period: String,             // "monthly" | "quarterly" | "per-engagement"
}

// ── VestingSchedule ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VestingSchedule {
    pub cliff_months: u32,
    pub total_months: u32,
    pub vested_pct: f64,
}

// ── Policy Set ────────────────────────────────────────────────────────────────

pub type PolicySet = Vec<PolicyId>;

// ── BenefitAccount ────────────────────────────────────────────────────────────

/// Every portable benefit account is a PortfolioItem (kind: BenefitAccount)
/// within the worker's personal portfolio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenefitAccount {
    pub benefit_type: BenefitType,
    pub provider_id: ProviderId,
    pub contribution_sources: Vec<ContributionSource>,
    pub balance: f64,
    pub currency: String,
    pub coverage_amount: Option<f64>,
    pub vesting_schedule: Option<VestingSchedule>,
    pub eligibility_rules: PolicySet,
    pub claim_workflow_id: Option<WorkflowId>,
    pub tax_treatment: TaxTreatment,
    /// kogi-bank BenefitAccount linked for fund flows.
    pub linked_bank_account: AccountId,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

impl BenefitAccount {
    pub fn new(
        benefit_type: BenefitType,
        provider_id: ProviderId,
        tax_treatment: TaxTreatment,
        linked_bank_account: AccountId,
    ) -> Self {
        let now = Utc::now();
        Self {
            benefit_type,
            provider_id,
            contribution_sources: vec![],
            balance: 0.0,
            currency: "USD".into(),
            coverage_amount: None,
            vesting_schedule: None,
            eligibility_rules: vec![],
            claim_workflow_id: None,
            tax_treatment,
            linked_bank_account,
            created_at: now,
            last_updated: now,
        }
    }

    pub fn add_contribution_source(&mut self, source: ContributionSource) {
        self.contribution_sources.push(source);
        self.last_updated = Utc::now();
    }

    pub fn credit(&mut self, amount: f64) {
        self.balance += amount;
        self.last_updated = Utc::now();
    }

    pub fn debit(&mut self, amount: f64) -> Result<(), &'static str> {
        if amount > self.balance {
            return Err("Insufficient balance");
        }
        self.balance -= amount;
        self.last_updated = Utc::now();
        Ok(())
    }

    /// Total monthly contribution from all sources (annualised / 12).
    pub fn monthly_contribution_estimate(&self) -> f64 {
        self.contribution_sources.iter().map(|s| {
            match s.period.as_str() {
                "monthly" => s.amount_per_period,
                "quarterly" => s.amount_per_period / 3.0,
                "annually" => s.amount_per_period / 12.0,
                _ => 0.0,
            }
        }).sum()
    }
}
