//! Computational Models — 11 built-in analytical models.
//!
//! Each is a stateless compute struct with a `compute()` method,
//! making them composable and testable in isolation.

use serde::{Deserialize, Serialize};



// ── 1. PortfolioHealth ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioHealthInput {
    pub total_components: usize,
    pub active_components: usize,
    pub budget_allocated: f64,
    pub budget_consumed: f64,
    pub avg_risk_severity: f64,   // e.g. average of RiskSeverity scores
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioHealthResult {
    pub health_score: f64,
    pub active_ratio: f64,
    pub budget_health: f64,
    pub risk_penalty: f64,
    pub size_bonus: f64,
}

/// Primary health signal exposed to the AI Engine and dashboard.
///
/// ```text
/// active_ratio    = active_components / total_components
/// budget_health   = clamp(1.0 - max(consumed/allocated - 1.0, 0.0), 0.0, 1.0)
/// risk_penalty    = avg_severity_score × 5.0  (Critical=4, High=3, Medium=2, Low=1)
/// size_bonus      = ln(1 + total_components) × 2.0
/// health_score    = clamp(active_ratio×50 + budget_health×30 - risk_penalty + size_bonus, 0, 100)
/// ```
pub struct PortfolioHealth;

impl PortfolioHealth {
    pub fn compute(input: &PortfolioHealthInput) -> PortfolioHealthResult {
        let active_ratio = if input.total_components == 0 {
            0.0
        } else {
            input.active_components as f64 / input.total_components as f64
        };

        let budget_health = if input.budget_allocated == 0.0 {
            1.0
        } else {
            let overrun = (input.budget_consumed / input.budget_allocated - 1.0).max(0.0);
            (1.0 - overrun).clamp(0.0, 1.0)
        };

        let risk_penalty = input.avg_risk_severity * 5.0;
        let size_bonus = (1.0 + input.total_components as f64).ln() * 2.0;

        let health_score = (active_ratio * 50.0
            + budget_health * 30.0
            - risk_penalty
            + size_bonus)
            .clamp(0.0, 100.0);

        PortfolioHealthResult {
            health_score,
            active_ratio,
            budget_health,
            risk_penalty,
            size_bonus,
        }
    }
}

// ── 2. ProjectMetrics ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetricsInput {
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub overdue_tasks: usize,
    pub backlog_size: usize,
    pub sprint_velocity: f64,       // avg story points per sprint
    pub story_points_completed: u32,
    pub story_points_total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel { Low, Medium, High, Critical }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetricsResult {
    pub completion_pct: f64,
    pub velocity: f64,
    pub risk_level: RiskLevel,
    pub backlog_ratio: f64,
}

pub struct ProjectMetrics;

impl ProjectMetrics {
    pub fn compute(input: &ProjectMetricsInput) -> ProjectMetricsResult {
        let completion_pct = if input.total_tasks == 0 { 0.0 } else {
            input.completed_tasks as f64 / input.total_tasks as f64 * 100.0
        };

        let backlog_ratio = if input.total_tasks == 0 { 0.0 } else {
            input.backlog_size as f64 / input.total_tasks as f64
        };

        let overdue_ratio = if input.total_tasks == 0 { 0.0 } else {
            input.overdue_tasks as f64 / input.total_tasks as f64
        };

        let risk_level = if overdue_ratio > 0.3 || backlog_ratio > 0.6 {
            RiskLevel::Critical
        } else if overdue_ratio > 0.15 || backlog_ratio > 0.4 {
            RiskLevel::High
        } else if overdue_ratio > 0.05 || backlog_ratio > 0.25 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        ProjectMetricsResult {
            completion_pct,
            velocity: input.sprint_velocity,
            risk_level,
            backlog_ratio,
        }
    }
}

// ── 3. ProgramAlignment ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramAlignmentInput {
    pub total_child_projects: usize,
    pub active_child_projects: usize,
    pub completed_child_projects: usize,
    pub budget_allocated: f64,
    pub budget_consumed: f64,
    pub kpi_count: usize,
    pub kpi_achieved: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramAlignmentResult {
    pub alignment_score: f64,
}

pub struct ProgramAlignment;

impl ProgramAlignment {
    pub fn compute(input: &ProgramAlignmentInput) -> ProgramAlignmentResult {
        let project_health = if input.total_child_projects == 0 { 0.0 } else {
            (input.active_child_projects + input.completed_child_projects) as f64
                / input.total_child_projects as f64
        };

        let budget_health = if input.budget_allocated == 0.0 { 1.0 } else {
            (1.0 - (input.budget_consumed / input.budget_allocated - 1.0).max(0.0)).clamp(0.0, 1.0)
        };

        let kpi_rate = if input.kpi_count == 0 { 1.0 } else {
            input.kpi_achieved as f64 / input.kpi_count as f64
        };

        let alignment_score = (project_health * 40.0 + budget_health * 30.0 + kpi_rate * 30.0)
            .clamp(0.0, 100.0);

        ProgramAlignmentResult { alignment_score }
    }
}

// ── 4. SubPortfolioRollup ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubPortfolioRollupInput {
    pub child_health_scores: Vec<f64>,
    pub total_allocated: f64,
    pub total_consumed: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubPortfolioRollupResult {
    pub rollup_score: f64,
    pub avg_child_health: f64,
    pub budget_health: f64,
}

pub struct SubPortfolioRollup;

impl SubPortfolioRollup {
    pub fn compute(input: &SubPortfolioRollupInput) -> SubPortfolioRollupResult {
        let avg_child_health = if input.child_health_scores.is_empty() { 0.0 } else {
            input.child_health_scores.iter().sum::<f64>() / input.child_health_scores.len() as f64
        };

        let budget_health = if input.total_allocated == 0.0 { 1.0 } else {
            (1.0 - (input.total_consumed / input.total_allocated - 1.0).max(0.0)).clamp(0.0, 1.0)
        };

        let rollup_score = (avg_child_health * 0.7 + budget_health * 100.0 * 0.3).clamp(0.0, 100.0);

        SubPortfolioRollupResult { rollup_score, avg_child_health, budget_health }
    }
}

// ── 5. ResourceUtilisation ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilisationInput {
    pub total_capacity: f64,
    pub committed: f64,
    pub consumed: f64,
    pub assigned_items: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilisationResult {
    pub utilisation_pct: f64,
    pub demand_ratio: f64,
    pub over_committed: bool,
}

pub struct ResourceUtilisation;

impl ResourceUtilisation {
    pub fn compute(input: &ResourceUtilisationInput) -> ResourceUtilisationResult {
        let utilisation_pct = if input.total_capacity == 0.0 { 0.0 } else {
            (input.consumed / input.total_capacity * 100.0).min(200.0)
        };

        let demand_ratio = if input.total_capacity == 0.0 { 0.0 } else {
            input.committed / input.total_capacity
        };

        ResourceUtilisationResult {
            utilisation_pct,
            demand_ratio,
            over_committed: input.committed > input.total_capacity,
        }
    }
}

// ── 6. AssetValue ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetValueInput {
    pub initial_value: f64,
    pub cost_basis: f64,
    pub age_years: f64,
    pub annual_depreciation_rate: f64,  // 0.0–1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetValueResult {
    pub current_value: f64,
    pub total_depreciation: f64,
    pub roi_pct: f64,
}

pub struct AssetValue;

impl AssetValue {
    pub fn compute(input: &AssetValueInput) -> AssetValueResult {
        // Straight-line depreciation
        let total_depreciation = (input.initial_value * input.annual_depreciation_rate * input.age_years)
            .min(input.initial_value);
        let current_value = (input.initial_value - total_depreciation).max(0.0);

        let roi_pct = if input.cost_basis == 0.0 { 0.0 } else {
            (current_value - input.cost_basis) / input.cost_basis * 100.0
        };

        AssetValueResult { current_value, total_depreciation, roi_pct }
    }
}

// ── 7. ArtifactMaturity ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMaturityInput {
    pub completeness: f64,      // 0.0–1.0 (how many required fields are filled)
    pub freshness_days: u32,    // days since last update (lower = fresher)
    pub reuse_count: u32,       // how many other items reference this artifact
    pub reviewed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMaturityResult {
    pub maturity_score: f64,
}

pub struct ArtifactMaturity;

impl ArtifactMaturity {
    pub fn compute(input: &ArtifactMaturityInput) -> ArtifactMaturityResult {
        let freshness_score = 1.0 / (1.0 + input.freshness_days as f64 / 30.0); // decay over 30 days
        let reuse_bonus = (input.reuse_count as f64 * 2.0).min(10.0);
        let review_bonus = if input.reviewed { 10.0 } else { 0.0 };

        let maturity_score = (input.completeness * 50.0
            + freshness_score * 30.0
            + reuse_bonus
            + review_bonus)
            .clamp(0.0, 100.0);

        ArtifactMaturityResult { maturity_score }
    }
}

// ── 8. BinderCoverage ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinderCoverageInput {
    pub expected_ids: std::collections::HashSet<uuid::Uuid>,
    pub actual_ids: std::collections::HashSet<uuid::Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinderCoverageResult {
    pub coverage_pct: f64,
    pub missing_count: usize,
    pub extra_count: usize,
}

pub struct BinderCoverage;

impl BinderCoverage {
    pub fn compute(input: &BinderCoverageInput) -> BinderCoverageResult {
        let missing_count = input.expected_ids.difference(&input.actual_ids).count();
        let extra_count = input.actual_ids.difference(&input.expected_ids).count();

        let coverage_pct = if input.expected_ids.is_empty() { 100.0 } else {
            let covered = input.expected_ids.len() - missing_count;
            covered as f64 / input.expected_ids.len() as f64 * 100.0
        };

        BinderCoverageResult { coverage_pct, missing_count, extra_count }
    }
}

// ── 9. BookConsistency ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookConsistencyInput {
    pub section_count: usize,
    pub empty_sections: usize,
    pub schema_compliant_sections: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookConsistencyResult {
    pub consistency_score: f64,
}

pub struct BookConsistency;

impl BookConsistency {
    pub fn compute(input: &BookConsistencyInput) -> BookConsistencyResult {
        let fullness = if input.section_count == 0 { 0.0 } else {
            1.0 - (input.empty_sections as f64 / input.section_count as f64)
        };

        let compliance = if input.section_count == 0 { 1.0 } else {
            input.schema_compliant_sections as f64 / input.section_count as f64
        };

        let consistency_score = (fullness * 50.0 + compliance * 50.0).clamp(0.0, 100.0);

        BookConsistencyResult { consistency_score }
    }
}

// ── 10. FolderOrganisation ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderOrganisationInput {
    pub max_depth: u8,
    pub orphan_count: usize,
    pub duplicate_names: usize,
    pub total_items: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderOrganisationResult {
    pub organisation_score: f64,
}

pub struct FolderOrganisation;

impl FolderOrganisation {
    pub fn compute(input: &FolderOrganisationInput) -> FolderOrganisationResult {
        // Depth penalty: ideal depth ≤ 4; penalise for excessive nesting
        let depth_penalty = ((input.max_depth as f64 - 4.0).max(0.0) * 5.0).min(20.0);
        let orphan_penalty = if input.total_items == 0 { 0.0 } else {
            (input.orphan_count as f64 / input.total_items as f64 * 30.0).min(30.0)
        };
        let dup_penalty = (input.duplicate_names as f64 * 2.0).min(20.0);

        let organisation_score = (100.0 - depth_penalty - orphan_penalty - dup_penalty)
            .clamp(0.0, 100.0);

        FolderOrganisationResult { organisation_score }
    }
}

// ── 11. RecordIntegrity ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordIntegrityInput {
    pub entry_count: usize,
    pub duplicate_count: usize,
    pub has_integrity_hash: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordIntegrityResult {
    pub integrity_score: f64,
}

pub struct RecordIntegrity;

impl RecordIntegrity {
    pub fn compute(input: &RecordIntegrityInput) -> RecordIntegrityResult {
        let dup_penalty = if input.entry_count == 0 { 0.0 } else {
            (input.duplicate_count as f64 / input.entry_count as f64 * 40.0).min(40.0)
        };
        let hash_bonus = if input.has_integrity_hash { 10.0 } else { 0.0 };
        let base = if input.entry_count > 0 { 90.0 } else { 0.0 };

        let integrity_score = (base - dup_penalty + hash_bonus).clamp(0.0, 100.0);

        RecordIntegrityResult { integrity_score }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portfolio_health_healthy() {
        let result = PortfolioHealth::compute(&PortfolioHealthInput {
            total_components: 20,
            active_components: 18,
            budget_allocated: 100_000.0,
            budget_consumed: 60_000.0,
            avg_risk_severity: 1.0,
        });
        assert!(result.health_score > 70.0, "score={}", result.health_score);
    }

    #[test]
    fn portfolio_health_distressed() {
        let result = PortfolioHealth::compute(&PortfolioHealthInput {
            total_components: 10,
            active_components: 2,
            budget_allocated: 50_000.0,
            budget_consumed: 80_000.0,
            avg_risk_severity: 4.0,
        });
        assert!(result.health_score < 40.0, "score={}", result.health_score);
    }

    #[test]
    fn asset_value_depreciation() {
        let result = AssetValue::compute(&AssetValueInput {
            initial_value: 10_000.0,
            cost_basis: 8_000.0,
            age_years: 2.0,
            annual_depreciation_rate: 0.2,
        });
        assert!((result.current_value - 6_000.0).abs() < 1.0);
        assert!((result.roi_pct - (-25.0)).abs() < 0.1);
    }

    #[test]
    fn project_metrics_risk_level() {
        let result = ProjectMetrics::compute(&ProjectMetricsInput {
            total_tasks: 10,
            completed_tasks: 3,
            overdue_tasks: 4,
            backlog_size: 7,
            sprint_velocity: 5.0,
            story_points_completed: 15,
            story_points_total: 40,
        });
        assert!(matches!(result.risk_level, RiskLevel::Critical));
    }
}
