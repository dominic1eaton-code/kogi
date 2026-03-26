use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HubEntityKind {
    Team,
    Organization,
    Collective,
    Cooperative,
    Federation,
    Autonomous,
    Dao,
    Cell,
    Guild,
    Consortium,
    Network,
    Foundation,
    Fund,
    Studio,
    Committee,
    Council,
    Circle,
    WorkingGroup,
    Community,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HubEntityStatus {
    Draft,
    Active,
    Paused,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HubMemberRole {
    Owner,
    Admin,
    Steward,
    Member,
    Contributor,
    Observer,
    Contractor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HubMemberType {
    IndependentWorker,
    Organization,
    Contractor,
    Advisor,
    Investor,
    Partner,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HubMemberStatus {
    Active,
    Invited,
    Pending,
    Suspended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubMember {
    pub member_id: Uuid,
    pub entity_id: Uuid,
    pub role: HubMemberRole,
    pub member_type: HubMemberType,
    pub status: HubMemberStatus,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubEntity {
    pub id: Uuid,
    pub kind: HubEntityKind,
    pub name: String,
    pub description: String,
    pub status: HubEntityStatus,
    pub governance_model: Option<String>,
    pub member_ids: Vec<Uuid>,
    pub tags: Vec<String>,
    pub topics: Vec<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceStatus {
    Draft,
    Review,
    Voting,
    Approved,
    Rejected,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernancePolicy {
    pub id: Uuid,
    pub title: String,
    pub status: GovernanceStatus,
    pub owner: Option<String>,
    pub effective_date: Option<NaiveDate>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceProposal {
    pub id: Uuid,
    pub title: String,
    pub status: GovernanceStatus,
    pub owner: Option<String>,
    pub due_date: Option<NaiveDate>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceFramework {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoteStatus {
    Draft,
    Open,
    Consensus,
    Closed,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceVote {
    pub id: Uuid,
    pub title: String,
    pub status: VoteStatus,
    pub close_date: Option<NaiveDate>,
    pub participation_pct: f64,
    pub quorum_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoterGroup {
    pub group: String,
    pub weight: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubGovernance {
    pub policies: Vec<GovernancePolicy>,
    pub proposals: Vec<GovernanceProposal>,
    pub frameworks: Vec<GovernanceFramework>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubVoting {
    pub ballots: Vec<GovernanceVote>,
    pub voter_groups: Vec<VoterGroup>,
    pub participation_pct: f64,
    pub quorum_target_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractType {
    Agreement,
    License,
    Employment,
    Service,
    Partnership,
    Grant,
    NDA,
    MOU,
    Charter,
    Policy,
    Procurement,
    Subscription,
    IP,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractStatus {
    Draft,
    Active,
    Negotiation,
    Expired,
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractRecord {
    pub id: Uuid,
    pub title: String,
    pub contract_type: ContractType,
    pub status: ContractStatus,
    pub parties: Vec<Uuid>,
    pub counterparty: Option<String>,
    pub effective_date: Option<NaiveDate>,
    pub expires_at: Option<NaiveDate>,
    pub value: Option<String>,
    pub currency: Option<String>,
    pub rights: Vec<String>,
    pub tags: Vec<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RightType {
    Patent,
    Copyright,
    Trademark,
    Watermark,
    License,
    Branding,
    Logo,
    Mark,
    Contract,
    Agreement,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RightRecord {
    pub id: Uuid,
    pub title: String,
    pub right_type: RightType,
    pub status: String,
    pub holder: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IpAssetType {
    Patent,
    Copyright,
    Trademark,
    Watermark,
    License,
    Brand,
    Logo,
    Mark,
    TradeSecret,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpAsset {
    pub id: Uuid,
    pub title: String,
    pub asset_type: IpAssetType,
    pub status: String,
    pub owner_id: Option<Uuid>,
    pub rights: Vec<String>,
    pub contract_ids: Vec<Uuid>,
    pub tags: Vec<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationRecord {
    pub id: Uuid,
    pub program: String,
    pub amount: String,
    pub owner: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionRecord {
    pub id: Uuid,
    pub name: String,
    pub amount: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestitutionCase {
    pub id: Uuid,
    pub title: String,
    pub status: String,
    pub owner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiationRecord {
    pub id: Uuid,
    pub title: String,
    pub parties: String,
    pub status: String,
}
