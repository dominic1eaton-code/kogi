use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use kogi_portfolio::{KogiPortfolioConfig, PortfolioRow, PortfolioSystem};

use crate::error::HubResult;
use crate::model::*;

#[derive(Debug, Clone)]
pub struct HubConfig {
    pub portfolio_config: KogiPortfolioConfig,
    pub hub_id: Uuid,
}

impl Default for HubConfig {
    fn default() -> Self {
        Self {
            portfolio_config: KogiPortfolioConfig::default(),
            hub_id: Uuid::new_v4(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubState {
    pub hub_id: Uuid,
    pub portfolio_root_component_id: Uuid,
    pub workbook_id: Uuid,
    pub entity_count: usize,
    pub member_count: usize,
    pub policy_count: usize,
    pub proposal_count: usize,
    pub vote_count: usize,
    pub contract_count: usize,
    pub rights_count: usize,
    pub ip_asset_count: usize,
    pub allocation_count: usize,
    pub distribution_count: usize,
    pub restitution_count: usize,
    pub negotiation_count: usize,
    pub last_refresh_at: DateTime<Utc>,
}

pub struct HubSystem {
    pub config: HubConfig,
    pub portfolio: PortfolioSystem,
    last_refresh_at: DateTime<Utc>,
}

impl HubSystem {
    pub fn new(config: HubConfig) -> HubResult<Self> {
        let portfolio = PortfolioSystem::new(config.portfolio_config.clone())?;
        Ok(Self {
            config,
            portfolio,
            last_refresh_at: Utc::now(),
        })
    }

    pub fn state(&self) -> HubState {
        let workbook = &self.portfolio.runtime.master.workbook;
        let mut entity_count = 0;
        let mut member_count = 0;
        let mut policy_count = 0;
        let mut proposal_count = 0;
        let mut vote_count = 0;
        let mut contract_count = 0;
        let mut rights_count = 0;
        let mut ip_asset_count = 0;
        let mut allocation_count = 0;
        let mut distribution_count = 0;
        let mut restitution_count = 0;
        let mut negotiation_count = 0;

        for row in workbook.row_store.iter() {
            if entity_kind_from_row(row).is_some() {
                entity_count += 1;
                member_count += member_ids_from_row(row).len();
            }
            if is_policy_row(row) {
                policy_count += 1;
            }
            if is_proposal_row(row) {
                proposal_count += 1;
            }
            if is_vote_row(row) {
                vote_count += 1;
            }
            if is_contract_row(row) {
                contract_count += 1;
            }
            if is_right_row(row) {
                rights_count += 1;
            }
            if is_ip_row(row) {
                ip_asset_count += 1;
            }
            if is_allocation_row(row) {
                allocation_count += 1;
            }
            if is_distribution_row(row) {
                distribution_count += 1;
            }
            if is_restitution_row(row) {
                restitution_count += 1;
            }
            if is_negotiation_row(row) {
                negotiation_count += 1;
            }
        }

        HubState {
            hub_id: self.config.hub_id,
            portfolio_root_component_id: self.portfolio.root_component_id(),
            workbook_id: self.portfolio.state().workbook_id,
            entity_count,
            member_count,
            policy_count,
            proposal_count,
            vote_count,
            contract_count,
            rights_count,
            ip_asset_count,
            allocation_count,
            distribution_count,
            restitution_count,
            negotiation_count,
            last_refresh_at: self.last_refresh_at,
        }
    }

    pub fn refresh(&mut self) -> HubResult<()> {
        self.portfolio.sync_from_grid()?;
        self.portfolio.persist()?;
        self.last_refresh_at = Utc::now();
        Ok(())
    }

    pub fn entities(&self) -> Vec<HubEntity> {
        let mut entities = Vec::new();
        for row in self.portfolio.runtime.master.workbook.row_store.iter() {
            if let Some(kind) = entity_kind_from_row(row) {
                entities.push(entity_from_row(row, kind));
            }
        }
        entities
    }

    pub fn members(&self) -> Vec<HubMember> {
        let mut members = Vec::new();
        for row in self.portfolio.runtime.master.workbook.row_store.iter() {
            if entity_kind_from_row(row).is_some() {
                members.extend(members_from_row(row));
            }
        }
        members
    }

    pub fn governance(&self) -> HubGovernance {
        let workbook = &self.portfolio.runtime.master.workbook;
        let policies = build_policies(workbook);
        let proposals = build_proposals(workbook);
        let frameworks = build_frameworks(workbook);
        HubGovernance {
            policies,
            proposals,
            frameworks,
        }
    }

    pub fn voting(&self) -> HubVoting {
        let workbook = &self.portfolio.runtime.master.workbook;
        build_voting(workbook)
    }

    pub fn contracts(&self) -> Vec<ContractRecord> {
        let workbook = &self.portfolio.runtime.master.workbook;
        build_contracts(workbook)
    }

    pub fn rights(&self) -> Vec<RightRecord> {
        let workbook = &self.portfolio.runtime.master.workbook;
        build_rights(workbook)
    }

    pub fn ip_assets(&self) -> Vec<IpAsset> {
        let workbook = &self.portfolio.runtime.master.workbook;
        build_ip_assets(workbook)
    }

    pub fn allocations(&self) -> Vec<AllocationRecord> {
        let workbook = &self.portfolio.runtime.master.workbook;
        build_allocations(workbook)
    }

    pub fn distributions(&self) -> Vec<DistributionRecord> {
        let workbook = &self.portfolio.runtime.master.workbook;
        build_distributions(workbook)
    }

    pub fn restitutions(&self) -> Vec<RestitutionCase> {
        let workbook = &self.portfolio.runtime.master.workbook;
        build_restitutions(workbook)
    }

    pub fn negotiations(&self) -> Vec<NegotiationRecord> {
        let workbook = &self.portfolio.runtime.master.workbook;
        build_negotiations(workbook)
    }
}

fn normalize(raw: &str) -> String {
    raw.to_lowercase()
        .replace(['-', ' '], "_")
        .replace("__", "_")
}

fn entity_kind_from_str(raw: &str) -> Option<HubEntityKind> {
    let key = normalize(raw);
    Some(match key.as_str() {
        "team" | "squad" | "pod" => HubEntityKind::Team,
        "organization" | "org" => HubEntityKind::Organization,
        "collective" => HubEntityKind::Collective,
        "cooperative" | "coop" => HubEntityKind::Cooperative,
        "federation" => HubEntityKind::Federation,
        "autonomous" | "autonomous_cell" | "autonomouscell" => HubEntityKind::Autonomous,
        "dao" => HubEntityKind::Dao,
        "cell" => HubEntityKind::Cell,
        "guild" => HubEntityKind::Guild,
        "consortium" => HubEntityKind::Consortium,
        "network" => HubEntityKind::Network,
        "foundation" => HubEntityKind::Foundation,
        "fund" => HubEntityKind::Fund,
        "studio" => HubEntityKind::Studio,
        "committee" => HubEntityKind::Committee,
        "council" => HubEntityKind::Council,
        "circle" => HubEntityKind::Circle,
        "working_group" | "workinggroup" => HubEntityKind::WorkingGroup,
        "community" => HubEntityKind::Community,
        _ => return None,
    })
}

fn entity_kind_from_row(row: &PortfolioRow) -> Option<HubEntityKind> {
    let candidates = [
        row.primary_type(),
        Some(row.component_type.clone()),
    ];
    for candidate in candidates.into_iter().flatten() {
        if let Some(kind) = entity_kind_from_str(&candidate) {
            return Some(kind);
        }
    }
    for tag in row.tags.iter().chain(row.topics.iter()) {
        if let Some(kind) = entity_kind_from_str(tag) {
            return Some(kind);
        }
    }
    None
}

fn entity_status_from_str(raw: &str) -> HubEntityStatus {
    match raw.to_lowercase().as_str() {
        "active" => HubEntityStatus::Active,
        "paused" | "hold" | "on_hold" => HubEntityStatus::Paused,
        "archived" => HubEntityStatus::Archived,
        _ => HubEntityStatus::Draft,
    }
}

fn entity_from_row(row: &PortfolioRow, kind: HubEntityKind) -> HubEntity {
    HubEntity {
        id: row.component_id,
        kind,
        name: row.display_name.clone().unwrap_or_else(|| row.name.clone()),
        description: row
            .ext
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        status: entity_status_from_str(&row.status),
        governance_model: row.governance_model.clone(),
        member_ids: member_ids_from_row(row),
        tags: row.tags.clone(),
        topics: row.topics.clone(),
        metadata: json!({ "portfolio_row": row }),
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn member_ids_from_row(row: &PortfolioRow) -> Vec<Uuid> {
    let mut seen = HashSet::new();
    let mut members = Vec::new();
    for id in row
        .owners
        .iter()
        .chain(row.editors.iter())
        .chain(row.contributors.iter())
        .chain(row.viewers.iter())
        .chain(row.watchers.iter())
    {
        if seen.insert(*id) {
            members.push(*id);
        }
    }
    members
}

fn members_from_row(row: &PortfolioRow) -> Vec<HubMember> {
    let mut members = Vec::new();
    let entity_id = row.component_id;
    let joined_at = row.created_at;
    append_members(&mut members, entity_id, &row.owners, HubMemberRole::Owner, joined_at);
    append_members(&mut members, entity_id, &row.editors, HubMemberRole::Admin, joined_at);
    append_members(&mut members, entity_id, &row.contributors, HubMemberRole::Contributor, joined_at);
    append_members(&mut members, entity_id, &row.viewers, HubMemberRole::Observer, joined_at);
    append_members(&mut members, entity_id, &row.watchers, HubMemberRole::Observer, joined_at);
    members
}

fn append_members(
    members: &mut Vec<HubMember>,
    entity_id: Uuid,
    member_ids: &[Uuid],
    role: HubMemberRole,
    joined_at: DateTime<Utc>,
) {
    for member_id in member_ids {
        members.push(HubMember {
            member_id: *member_id,
            entity_id,
            role: role.clone(),
            member_type: HubMemberType::IndependentWorker,
            status: HubMemberStatus::Active,
            joined_at,
        });
    }
}

fn is_policy_row(row: &PortfolioRow) -> bool {
    matches_tag(row, "policy") || matches_type(row, "policy")
}

fn is_proposal_row(row: &PortfolioRow) -> bool {
    matches_tag(row, "proposal") || matches_type(row, "proposal") || matches_tag(row, "governance")
}

fn is_vote_row(row: &PortfolioRow) -> bool {
    matches_tag(row, "vote") || matches_type(row, "vote") || matches_tag(row, "ballot") || matches_type(row, "ballot")
}

fn is_contract_row(row: &PortfolioRow) -> bool {
    matches_type(row, "contract")
        || matches_type(row, "agreement")
        || matches_type(row, "license")
        || matches_type(row, "licence")
        || matches_tag(row, "contract")
        || matches_tag(row, "agreement")
        || matches_tag(row, "license")
        || matches_tag(row, "licence")
}

fn is_right_row(row: &PortfolioRow) -> bool {
    matches_type(row, "right")
        || matches_type(row, "rights")
        || matches_tag(row, "right")
        || matches_tag(row, "rights")
}

fn is_ip_row(row: &PortfolioRow) -> bool {
    matches_tag(row, "ip")
        || matches_tag(row, "patent")
        || matches_tag(row, "copyright")
        || matches_tag(row, "trademark")
        || matches_tag(row, "watermark")
        || matches_tag(row, "license")
        || matches_type(row, "patent")
        || matches_type(row, "copyright")
        || matches_type(row, "trademark")
        || matches_type(row, "watermark")
        || matches_type(row, "license")
        || matches_type(row, "ip")
}

fn is_allocation_row(row: &PortfolioRow) -> bool {
    row.budget_allocated.is_some() || matches_tag(row, "allocation")
}

fn is_distribution_row(row: &PortfolioRow) -> bool {
    let revenue = row.revenue.to_string();
    let has_revenue = revenue != "0" && revenue != "0.0" && revenue != "0.00";
    has_revenue || matches_tag(row, "distribution")
}

fn is_restitution_row(row: &PortfolioRow) -> bool {
    matches_tag(row, "restitution")
}

fn is_negotiation_row(row: &PortfolioRow) -> bool {
    matches_tag(row, "negotiation") || matches_tag(row, "deal")
}

fn matches_tag(row: &PortfolioRow, tag: &str) -> bool {
    row.tags.iter().any(|t| t.eq_ignore_ascii_case(tag))
        || row.topics.iter().any(|t| t.eq_ignore_ascii_case(tag))
}

fn matches_type(row: &PortfolioRow, value: &str) -> bool {
    row.item_type
        .as_deref()
        .or(row.container_type.as_deref())
        .map(|t| t.eq_ignore_ascii_case(value))
        .unwrap_or(false)
}

fn governance_status_from_str(raw: &str) -> GovernanceStatus {
    match raw.to_lowercase().as_str() {
        "review" => GovernanceStatus::Review,
        "voting" => GovernanceStatus::Voting,
        "approved" | "ratified" => GovernanceStatus::Approved,
        "rejected" => GovernanceStatus::Rejected,
        "archived" => GovernanceStatus::Archived,
        _ => GovernanceStatus::Draft,
    }
}

fn build_policies(workbook: &kogi_portfolio::SpreadsheetWorkbook) -> Vec<GovernancePolicy> {
    let mut policies = Vec::new();
    for row in workbook.row_store.iter() {
        if is_policy_row(row) {
            policies.push(GovernancePolicy {
                id: row.component_id,
                title: row.display_name.clone().unwrap_or_else(|| row.name.clone()),
                status: governance_status_from_str(&row.status),
                owner: row.owners.first().map(|id| id.to_string()),
                effective_date: row.start_date,
                tags: row.tags.clone(),
            });
        }
    }
    policies
}

fn build_proposals(workbook: &kogi_portfolio::SpreadsheetWorkbook) -> Vec<GovernanceProposal> {
    let mut proposals = Vec::new();
    for row in workbook.row_store.iter() {
        if is_proposal_row(row) {
            proposals.push(GovernanceProposal {
                id: row.component_id,
                title: row.display_name.clone().unwrap_or_else(|| row.name.clone()),
                status: governance_status_from_str(&row.status),
                owner: row.owners.first().map(|id| id.to_string()),
                due_date: row.due_date,
                tags: row.tags.clone(),
            });
        }
    }
    proposals
}

fn build_frameworks(workbook: &kogi_portfolio::SpreadsheetWorkbook) -> Vec<GovernanceFramework> {
    let mut frameworks = Vec::new();
    let mut seen = HashSet::new();
    for row in workbook.row_store.iter() {
        if let Some(model) = row.governance_model.clone() {
            let key = model.to_lowercase();
            if seen.insert(key) {
                frameworks.push(GovernanceFramework {
                    name: model,
                    description: "Portfolio-defined governance model".to_string(),
                });
            }
        }
    }
    if frameworks.is_empty() {
        frameworks.extend([
            GovernanceFramework {
                name: "Holonic Governance".to_string(),
                description: "Nested councils with clear mandates".to_string(),
            },
            GovernanceFramework {
                name: "Consensus + Delegate".to_string(),
                description: "Hybrid voting with fallback delegates".to_string(),
            },
            GovernanceFramework {
                name: "Cooperative Charter".to_string(),
                description: "Member rights, obligations, and dividends".to_string(),
            },
        ]);
    }
    frameworks
}

fn vote_status_from_str(raw: &str) -> VoteStatus {
    match raw.to_lowercase().as_str() {
        "open" => VoteStatus::Open,
        "consensus" => VoteStatus::Consensus,
        "closed" | "done" => VoteStatus::Closed,
        "archived" => VoteStatus::Archived,
        _ => VoteStatus::Draft,
    }
}

fn build_voting(workbook: &kogi_portfolio::SpreadsheetWorkbook) -> HubVoting {
    let mut ballots = Vec::new();
    let mut total_participation = 0.0;
    let mut count = 0.0;
    for row in workbook.row_store.iter() {
        if is_vote_row(row) {
            let participation = if row.progress_pct > 0.0 { row.progress_pct } else { 0.0 };
            let quorum = row
                .ext
                .get("quorum_pct")
                .and_then(|v| v.as_f64())
                .unwrap_or(55.0);
            ballots.push(GovernanceVote {
                id: row.component_id,
                title: row.display_name.clone().unwrap_or_else(|| row.name.clone()),
                status: vote_status_from_str(&row.status),
                close_date: row.due_date,
                participation_pct: participation,
                quorum_pct: quorum,
            });
            total_participation += participation;
            count += 1.0;
        }
    }

    let participation_pct = if count > 0.0 {
        total_participation / count
    } else {
        0.0
    };

    HubVoting {
        ballots,
        voter_groups: vec![
            VoterGroup {
                group: "Council Delegates".to_string(),
                weight: "32%".to_string(),
                status: "Aligned".to_string(),
            },
            VoterGroup {
                group: "Member Assembly".to_string(),
                weight: "48%".to_string(),
                status: "Voting".to_string(),
            },
            VoterGroup {
                group: "Advisory Board".to_string(),
                weight: "20%".to_string(),
                status: "Pending".to_string(),
            },
        ],
        participation_pct,
        quorum_target_pct: 55.0,
    }
}

fn contract_type_from_row(row: &PortfolioRow) -> ContractType {
    let raw = row
        .primary_type()
        .or_else(|| row.item_type.clone())
        .unwrap_or_else(|| "contract".to_string());
    match normalize(&raw).as_str() {
        "agreement" => ContractType::Agreement,
        "license" | "licence" => ContractType::License,
        "employment" => ContractType::Employment,
        "service" => ContractType::Service,
        "partnership" => ContractType::Partnership,
        "grant" => ContractType::Grant,
        "nda" => ContractType::NDA,
        "mou" => ContractType::MOU,
        "charter" => ContractType::Charter,
        "policy" => ContractType::Policy,
        "procurement" => ContractType::Procurement,
        "subscription" => ContractType::Subscription,
        "ip" => ContractType::IP,
        _ => ContractType::Custom,
    }
}

fn contract_status_from_str(raw: &str) -> ContractStatus {
    match raw.to_lowercase().as_str() {
        "active" => ContractStatus::Active,
        "negotiation" => ContractStatus::Negotiation,
        "expired" => ContractStatus::Expired,
        "terminated" => ContractStatus::Terminated,
        _ => ContractStatus::Draft,
    }
}

fn build_contracts(workbook: &kogi_portfolio::SpreadsheetWorkbook) -> Vec<ContractRecord> {
    let mut contracts = Vec::new();
    for row in workbook.row_store.iter() {
        if is_contract_row(row) {
            let value = row.budget_allocated.map(|b| format!("${}", b));
            contracts.push(ContractRecord {
                id: row.component_id,
                title: row.display_name.clone().unwrap_or_else(|| row.name.clone()),
                contract_type: contract_type_from_row(row),
                status: contract_status_from_str(&row.status),
                parties: row.owners.clone(),
                counterparty: row.ext.get("counterparty").and_then(|v| v.as_str()).map(|s| s.to_string()),
                effective_date: row.start_date,
                expires_at: row.end_date.or(row.due_date),
                value,
                currency: row.currency.clone(),
                rights: list_from_value(row.ext.get("rights")),
                tags: row.tags.clone(),
                metadata: json!({ "portfolio_row": row }),
                created_at: row.created_at,
                updated_at: row.updated_at,
            });
        }
    }
    contracts
}

fn right_type_from_row(row: &PortfolioRow) -> RightType {
    let raw = row
        .primary_type()
        .or_else(|| row.item_type.clone())
        .unwrap_or_else(|| "right".to_string());
    match normalize(&raw).as_str() {
        "patent" => RightType::Patent,
        "copyright" => RightType::Copyright,
        "trademark" => RightType::Trademark,
        "watermark" => RightType::Watermark,
        "license" | "licence" => RightType::License,
        "branding" => RightType::Branding,
        "logo" => RightType::Logo,
        "mark" => RightType::Mark,
        "contract" => RightType::Contract,
        "agreement" => RightType::Agreement,
        _ => RightType::Custom,
    }
}

fn build_rights(workbook: &kogi_portfolio::SpreadsheetWorkbook) -> Vec<RightRecord> {
    let mut rights = Vec::new();
    for row in workbook.row_store.iter() {
        if is_right_row(row) || is_ip_row(row) {
            rights.push(RightRecord {
                id: row.component_id,
                title: row.display_name.clone().unwrap_or_else(|| row.name.clone()),
                right_type: right_type_from_row(row),
                status: row.status.clone(),
                holder: row.owners.first().map(|id| id.to_string()),
                tags: row.tags.clone(),
                created_at: row.created_at,
                updated_at: row.updated_at,
            });
        }
    }
    rights
}

fn ip_asset_type_from_row(row: &PortfolioRow) -> IpAssetType {
    let raw = row
        .primary_type()
        .or_else(|| row.item_type.clone())
        .unwrap_or_else(|| "ip".to_string());
    match normalize(&raw).as_str() {
        "patent" => IpAssetType::Patent,
        "copyright" => IpAssetType::Copyright,
        "trademark" => IpAssetType::Trademark,
        "watermark" => IpAssetType::Watermark,
        "license" | "licence" => IpAssetType::License,
        "brand" | "branding" => IpAssetType::Brand,
        "logo" => IpAssetType::Logo,
        "mark" => IpAssetType::Mark,
        "trade_secret" | "tradesecret" => IpAssetType::TradeSecret,
        _ => IpAssetType::Custom,
    }
}

fn build_ip_assets(workbook: &kogi_portfolio::SpreadsheetWorkbook) -> Vec<IpAsset> {
    let mut assets = Vec::new();
    for row in workbook.row_store.iter() {
        if is_ip_row(row) {
            assets.push(IpAsset {
                id: row.component_id,
                title: row.display_name.clone().unwrap_or_else(|| row.name.clone()),
                asset_type: ip_asset_type_from_row(row),
                status: row.status.clone(),
                owner_id: row.owners.first().copied(),
                rights: list_from_value(row.ext.get("rights")),
                contract_ids: row.linked_ids.clone(),
                tags: row.tags.clone(),
                metadata: json!({ "portfolio_row": row }),
                created_at: row.created_at,
                updated_at: row.updated_at,
            });
        }
    }
    assets
}

fn build_allocations(workbook: &kogi_portfolio::SpreadsheetWorkbook) -> Vec<AllocationRecord> {
    let mut allocations = Vec::new();
    for row in workbook.row_store.iter() {
        if is_allocation_row(row) {
            let amount = row
                .budget_allocated
                .map(|b| format!("${}", b))
                .unwrap_or_else(|| "N/A".to_string());
            allocations.push(AllocationRecord {
                id: row.component_id,
                program: row.display_name.clone().unwrap_or_else(|| row.name.clone()),
                amount,
                owner: row.owners.first().map(|id| id.to_string()),
                status: row.status.clone(),
            });
        }
    }
    allocations
}

fn build_distributions(workbook: &kogi_portfolio::SpreadsheetWorkbook) -> Vec<DistributionRecord> {
    let mut distributions = Vec::new();
    for row in workbook.row_store.iter() {
        if is_distribution_row(row) {
            let amount = if row.revenue.to_string() != "0" {
                format!("${}", row.revenue)
            } else {
                format!("${}", row.budget_spent)
            };
            distributions.push(DistributionRecord {
                id: row.component_id,
                name: row.display_name.clone().unwrap_or_else(|| row.name.clone()),
                amount,
                status: row.status.clone(),
            });
        }
    }
    distributions
}

fn build_restitutions(workbook: &kogi_portfolio::SpreadsheetWorkbook) -> Vec<RestitutionCase> {
    let mut cases = Vec::new();
    for row in workbook.row_store.iter() {
        if is_restitution_row(row) {
            cases.push(RestitutionCase {
                id: row.component_id,
                title: row.display_name.clone().unwrap_or_else(|| row.name.clone()),
                status: row.status.clone(),
                owner: row.owners.first().map(|id| id.to_string()),
            });
        }
    }
    cases
}

fn build_negotiations(workbook: &kogi_portfolio::SpreadsheetWorkbook) -> Vec<NegotiationRecord> {
    let mut negotiations = Vec::new();
    for row in workbook.row_store.iter() {
        if is_negotiation_row(row) {
            let parties = row
                .ext
                .get("parties")
                .and_then(|v| v.as_str())
                .unwrap_or("Multiple parties")
                .to_string();
            negotiations.push(NegotiationRecord {
                id: row.component_id,
                title: row.display_name.clone().unwrap_or_else(|| row.name.clone()),
                parties,
                status: row.status.clone(),
            });
        }
    }
    negotiations
}

fn list_from_value(value: Option<&Value>) -> Vec<String> {
    let Some(value) = value else { return vec![]; };
    match value {
        Value::Array(items) => items
            .iter()
            .filter_map(|item| item.as_str().map(|s| s.to_string()))
            .collect(),
        Value::String(s) => s
            .split(',')
            .map(|item| item.trim())
            .filter(|item| !item.is_empty())
            .map(|item| item.to_string())
            .collect(),
        _ => vec![],
    }
}
