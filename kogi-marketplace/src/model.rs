use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ListingStatus {
    Draft,
    Active,
    Paused,
    Sold,
    Expired,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ListingType {
    Work,
    Asset,
    Artifact,
    Resource,
    Campaign,
    Template,
    Service,
    Bundle,
    Job,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ListingVisibility {
    Public,
    Community,
    InviteOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PricingModel {
    Fixed,
    Hourly,
    Daily,
    Subscription,
    Negotiable,
    Free,
    Auction,
    RevenueShare,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Price {
    pub amount: Decimal,
    pub currency: String,
    pub pricing_model: PricingModel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListingAnalytics {
    pub impressions: u32,
    pub views: u32,
    pub ctr: f64,
    pub conversion_rate: f64,
    pub engagement: f64,
    pub orders: u32,
    pub proposals: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Listing {
    pub id: Uuid,
    pub listing_type: ListingType,
    pub portfolio_component_id: Option<Uuid>,
    pub owner_id: Uuid,
    pub title: String,
    pub description: String,
    pub price: Price,
    pub status: ListingStatus,
    pub visibility: ListingVisibility,
    pub tags: Vec<String>,
    pub skills: Vec<String>,
    pub media: Vec<String>,
    pub review_ids: Vec<Uuid>,
    pub rating: f64,
    pub order_ids: Vec<Uuid>,
    pub proposal_ids: Vec<Uuid>,
    pub ai_score: f64,
    pub analytics: ListingAnalytics,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BidStatus {
    Open,
    Accepted,
    Declined,
    Withdrawn,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bid {
    pub id: Uuid,
    pub listing_id: Uuid,
    pub bidder_id: Uuid,
    pub amount: Decimal,
    pub currency: String,
    pub status: BidStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    Draft,
    Sent,
    Countered,
    Accepted,
    Rejected,
    Expired,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalMilestone {
    pub name: String,
    pub due_date: Option<DateTime<Utc>>,
    pub amount: Decimal,
    pub deliverable: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: Uuid,
    pub listing_id: Uuid,
    pub buyer_id: Uuid,
    pub seller_id: Uuid,
    pub status: ProposalStatus,
    pub scope: String,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub price: Price,
    pub milestones: Vec<ProposalMilestone>,
    pub terms: String,
    pub attachments: Vec<String>,
    pub deal_room_id: Option<Uuid>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DealStatus {
    Proposed,
    Active,
    InEscrow,
    Delivered,
    Completed,
    Disputed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealMilestone {
    pub label: String,
    pub amount: Decimal,
    pub status: String,
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deal {
    pub id: Uuid,
    pub listing_id: Uuid,
    pub buyer_id: Uuid,
    pub seller_id: Uuid,
    pub status: DealStatus,
    pub total: Decimal,
    pub currency: String,
    pub escrow_id: Option<Uuid>,
    pub milestones: Vec<DealMilestone>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscrowStatus {
    Pending,
    Funded,
    PartialReleased,
    Released,
    Disputed,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowAccount {
    pub id: Uuid,
    pub deal_id: Uuid,
    pub status: EscrowStatus,
    pub total_value: Decimal,
    pub released_value: Decimal,
    pub fee_percent: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CampaignType {
    Resource,
    Fundraise,
    Equity,
    GroupBuy,
    RevenueShare,
    CommunityFund,
    Presale,
    GrantMatch,
    Donation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CampaignStatus {
    Draft,
    Active,
    Funded,
    Failed,
    Distributing,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignBacker {
    pub backer_id: Uuid,
    pub amount: Decimal,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityTerms {
    pub percent_offered: f64,
    pub valuation: Decimal,
    pub rights: String,
    pub vesting: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Campaign {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub campaign_type: CampaignType,
    pub status: CampaignStatus,
    pub portfolio_component_id: Option<Uuid>,
    pub goal: Decimal,
    pub raised: Decimal,
    pub currency: String,
    pub backers: Vec<CampaignBacker>,
    pub equity_terms: Option<EquityTerms>,
    pub deadline: Option<DateTime<Utc>>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderMarket {
    Labor,
    Commodities,
    Financial,
    Resource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    Bid,
    Ask,
    Market,
    Limit,
    Proposal,
    Rfp,
    Auction,
    ReverseAuction,
    Swap,
    Otc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Open,
    Matched,
    Filled,
    Cancelled,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeOrder {
    pub id: Uuid,
    pub market: OrderMarket,
    pub order_type: OrderType,
    pub subject: String,
    pub price: Price,
    pub quantity: Decimal,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarterOffer {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub offering: String,
    pub requesting: String,
    pub estimated_value: Decimal,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarterBid {
    pub id: Uuid,
    pub offer_id: Uuid,
    pub bidder_id: Uuid,
    pub message: String,
    pub estimated_value: Decimal,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSignal {
    pub label: String,
    pub value: String,
    pub tone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeQuote {
    pub label: String,
    pub price: String,
    pub delta: String,
}
