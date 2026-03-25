use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WalletStatus {
    Active,
    Paused,
    Frozen,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountStatus {
    Provisioned,
    Active,
    Frozen,
    Suspended,
    Closed,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    Draft,
    Pending,
    Scheduled,
    Cleared,
    Failed,
    Reversed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoSplitRule {
    pub name: String,
    pub rule_type: String,
    pub percent: f64,
    pub destination_account_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub id: Uuid,
    pub wallet_type: String,
    pub owner_id: Uuid,
    pub name: String,
    pub linked_accounts: Vec<Uuid>,
    pub primary_account_id: Option<Uuid>,
    pub balance_display: Decimal,
    pub status: WalletStatus,
    pub transaction_rules: serde_json::Value,
    pub auto_split: Vec<AutoSplitRule>,
    pub allowed_transaction_types: Vec<String>,
    pub connected_external: Vec<String>,
    pub activity_feed_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub portfolio_component_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub account_type: String,
    pub owner_id: Uuid,
    pub owner_type: String,
    pub name: String,
    pub currency: String,
    pub denomination: String,
    pub balance: Decimal,
    pub available_balance: Decimal,
    pub pending_balance: Decimal,
    pub status: AccountStatus,
    pub linked_external: Vec<String>,
    pub linked_wallets: Vec<Uuid>,
    pub policy_ids: Vec<Uuid>,
    pub portfolio_component_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletTransaction {
    pub id: Uuid,
    pub tx_type: String,
    pub status: TransactionStatus,
    pub amount: Decimal,
    pub currency: String,
    pub from_wallet_id: Option<Uuid>,
    pub to_wallet_id: Option<Uuid>,
    pub from_account_id: Option<Uuid>,
    pub to_account_id: Option<Uuid>,
    pub reference_type: Option<String>,
    pub reference_id: Option<String>,
    pub approval_status: Option<String>,
    pub fees: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}
