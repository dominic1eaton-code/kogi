use std::collections::HashMap;

use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use serde_json::json;
use uuid::Uuid;

use kogi_portfolio::{ComponentCategory, ItemCategory, KogiPortfolioConfig, PortfolioSystem};

use crate::error::WalletResult;
use crate::model::{
    Account, AccountStatus, AutoSplitRule, Wallet, WalletStatus, WalletTransaction, TransactionStatus,
};
use crate::snapshots::{
    sample_banking_snapshot, sample_benefits_snapshot, sample_campaigns_snapshot,
    sample_dashboard_snapshot, sample_debts_snapshot, sample_escrow_snapshot,
    sample_funding_snapshot, sample_grants_snapshot, sample_group_economics_snapshot,
    sample_invoices_snapshot, sample_investments_snapshot, sample_ledger_snapshot,
    sample_taxes_snapshot, WalletBankingSnapshot, WalletBenefitsSnapshot, WalletCampaignsSnapshot,
    WalletDashboardSnapshot, WalletDebtsSnapshot, WalletEscrowSnapshot, WalletFundingSnapshot,
    WalletGrantsSnapshot, WalletGroupEconomicsSnapshot, WalletInvoicesSnapshot,
    WalletInvestmentsSnapshot, WalletLedgerSnapshot, WalletTaxesSnapshot, WalletsOverviewSnapshot,
    WalletSummary, AccountSummary,
};

#[derive(Debug, Clone)]
pub struct WalletConfig {
    pub owner_id: Uuid,
    pub portfolio_config: KogiPortfolioConfig,
}

impl Default for WalletConfig {
    fn default() -> Self {
        let owner_id = Uuid::new_v4();
        let mut portfolio_config = KogiPortfolioConfig::default();
        portfolio_config.owner_id = owner_id;
        Self { owner_id, portfolio_config }
    }
}

#[derive(Default)]
struct WalletStore {
    wallets: HashMap<Uuid, Wallet>,
    accounts: HashMap<Uuid, Account>,
    transactions: Vec<WalletTransaction>,
}

pub struct WalletSystem {
    pub config: WalletConfig,
    pub portfolio: PortfolioSystem,
    store: WalletStore,
}

impl WalletSystem {
    pub fn new(config: WalletConfig) -> WalletResult<Self> {
        let portfolio = PortfolioSystem::new(config.portfolio_config.clone())?;
        let mut system = Self {
            config,
            portfolio,
            store: WalletStore::default(),
        };
        system.seed_defaults()?;
        Ok(system)
    }

    pub fn dashboard_snapshot(&self) -> WalletDashboardSnapshot { sample_dashboard_snapshot() }
    pub fn banking_snapshot(&self) -> WalletBankingSnapshot { sample_banking_snapshot() }
    pub fn ledger_snapshot(&self) -> WalletLedgerSnapshot { sample_ledger_snapshot() }
    pub fn escrow_snapshot(&self) -> WalletEscrowSnapshot { sample_escrow_snapshot() }
    pub fn invoices_snapshot(&self) -> WalletInvoicesSnapshot { sample_invoices_snapshot() }
    pub fn investments_snapshot(&self) -> WalletInvestmentsSnapshot { sample_investments_snapshot() }
    pub fn funding_snapshot(&self) -> WalletFundingSnapshot { sample_funding_snapshot() }
    pub fn benefits_snapshot(&self) -> WalletBenefitsSnapshot { sample_benefits_snapshot() }
    pub fn grants_snapshot(&self) -> WalletGrantsSnapshot { sample_grants_snapshot() }
    pub fn group_economics_snapshot(&self) -> WalletGroupEconomicsSnapshot { sample_group_economics_snapshot() }
    pub fn campaigns_snapshot(&self) -> WalletCampaignsSnapshot { sample_campaigns_snapshot() }
    pub fn debts_snapshot(&self) -> WalletDebtsSnapshot { sample_debts_snapshot() }
    pub fn taxes_snapshot(&self) -> WalletTaxesSnapshot { sample_taxes_snapshot() }

    pub fn wallets_overview_snapshot(&self) -> WalletsOverviewSnapshot {
        let wallets = self.store.wallets.values().map(|wallet| WalletSummary {
            id: wallet.id.to_string(),
            name: wallet.name.clone(),
            wallet_type: wallet.wallet_type.clone(),
            balance: format!("{}", wallet.balance_display),
            status: format!("{:?}", wallet.status),
        }).collect();
        let accounts = self.store.accounts.values().map(|account| AccountSummary {
            id: account.id.to_string(),
            name: account.name.clone(),
            account_type: account.account_type.clone(),
            balance: format!("{}", account.balance),
            status: format!("{:?}", account.status),
        }).collect();
        WalletsOverviewSnapshot { wallets, accounts }
    }

    pub fn wallets(&self) -> Vec<Wallet> {
        self.store.wallets.values().cloned().collect()
    }

    pub fn accounts(&self) -> Vec<Account> {
        self.store.accounts.values().cloned().collect()
    }

    fn seed_defaults(&mut self) -> WalletResult<()> {
        let owner_id = self.config.owner_id;
        let now = Utc::now();

        let mut checking = Account {
            id: Uuid::new_v4(),
            account_type: "checking".into(),
            owner_id,
            owner_type: "user".into(),
            name: "Checking".into(),
            currency: "USD".into(),
            denomination: "fiat".into(),
            balance: Decimal::from_f64(14_200.0).unwrap_or(Decimal::ZERO),
            available_balance: Decimal::from_f64(13_900.0).unwrap_or(Decimal::ZERO),
            pending_balance: Decimal::from_f64(300.0).unwrap_or(Decimal::ZERO),
            status: AccountStatus::Active,
            linked_external: vec!["plaid:checking".into()],
            linked_wallets: vec![],
            policy_ids: vec![],
            portfolio_component_id: None,
            metadata: json!({"seed": true}),
            created_at: now,
            updated_at: now,
        };
        checking.portfolio_component_id = Some(self.register_component(
            &checking.name,
            ComponentCategory::Item(ItemCategory::Custom("Account".into())),
            serde_json::to_value(&checking).unwrap_or(json!({})),
        )?);

        let mut ops = Account {
            id: Uuid::new_v4(),
            account_type: "operations".into(),
            owner_id,
            owner_type: "user".into(),
            name: "Operations".into(),
            currency: "USD".into(),
            denomination: "fiat".into(),
            balance: Decimal::from_f64(21_900.0).unwrap_or(Decimal::ZERO),
            available_balance: Decimal::from_f64(20_900.0).unwrap_or(Decimal::ZERO),
            pending_balance: Decimal::from_f64(1_000.0).unwrap_or(Decimal::ZERO),
            status: AccountStatus::Active,
            linked_external: vec!["plaid:ops".into()],
            linked_wallets: vec![],
            policy_ids: vec![],
            portfolio_component_id: None,
            metadata: json!({"seed": true}),
            created_at: now,
            updated_at: now,
        };
        ops.portfolio_component_id = Some(self.register_component(
            &ops.name,
            ComponentCategory::Item(ItemCategory::Custom("Account".into())),
            serde_json::to_value(&ops).unwrap_or(json!({})),
        )?);

        let mut reserve = Account {
            id: Uuid::new_v4(),
            account_type: "reserve".into(),
            owner_id,
            owner_type: "user".into(),
            name: "Capital Reserve".into(),
            currency: "USD".into(),
            denomination: "fiat".into(),
            balance: Decimal::from_f64(74_800.0).unwrap_or(Decimal::ZERO),
            available_balance: Decimal::from_f64(74_800.0).unwrap_or(Decimal::ZERO),
            pending_balance: Decimal::ZERO,
            status: AccountStatus::Active,
            linked_external: vec!["plaid:reserve".into()],
            linked_wallets: vec![],
            policy_ids: vec![],
            portfolio_component_id: None,
            metadata: json!({"seed": true}),
            created_at: now,
            updated_at: now,
        };
        reserve.portfolio_component_id = Some(self.register_component(
            &reserve.name,
            ComponentCategory::Item(ItemCategory::Custom("Account".into())),
            serde_json::to_value(&reserve).unwrap_or(json!({})),
        )?);

        let mut escrow = Account {
            id: Uuid::new_v4(),
            account_type: "escrow".into(),
            owner_id,
            owner_type: "user".into(),
            name: "Escrow Float".into(),
            currency: "USD".into(),
            denomination: "fiat".into(),
            balance: Decimal::from_f64(18_300.0).unwrap_or(Decimal::ZERO),
            available_balance: Decimal::from_f64(12_100.0).unwrap_or(Decimal::ZERO),
            pending_balance: Decimal::from_f64(6_200.0).unwrap_or(Decimal::ZERO),
            status: AccountStatus::Active,
            linked_external: vec!["plaid:escrow".into()],
            linked_wallets: vec![],
            policy_ids: vec![],
            portfolio_component_id: None,
            metadata: json!({"seed": true}),
            created_at: now,
            updated_at: now,
        };
        escrow.portfolio_component_id = Some(self.register_component(
            &escrow.name,
            ComponentCategory::Item(ItemCategory::Custom("Account".into())),
            serde_json::to_value(&escrow).unwrap_or(json!({})),
        )?);

        let mut personal_wallet = Wallet {
            id: Uuid::new_v4(),
            wallet_type: "personal".into(),
            owner_id,
            name: "Personal Wallet".into(),
            linked_accounts: vec![checking.id],
            primary_account_id: Some(checking.id),
            balance_display: Decimal::from_f64(14_200.0).unwrap_or(Decimal::ZERO),
            status: WalletStatus::Active,
            transaction_rules: json!({"limit": 5000}),
            auto_split: vec![
                AutoSplitRule {
                    name: "Tax Reserve".into(),
                    rule_type: "percentage".into(),
                    percent: 0.2,
                    destination_account_id: reserve.id,
                },
            ],
            allowed_transaction_types: vec!["transfer".into(), "payment".into()],
            connected_external: vec!["stripe:card".into()],
            activity_feed_id: None,
            metadata: json!({"seed": true}),
            created_at: now,
            updated_at: now,
            portfolio_component_id: None,
        };
        personal_wallet.portfolio_component_id = Some(self.register_component(
            &personal_wallet.name,
            ComponentCategory::Item(ItemCategory::Custom("Wallet".into())),
            serde_json::to_value(&personal_wallet).unwrap_or(json!({})),
        )?);

        let mut ops_wallet = Wallet {
            id: Uuid::new_v4(),
            wallet_type: "operations".into(),
            owner_id,
            name: "Operations Wallet".into(),
            linked_accounts: vec![ops.id],
            primary_account_id: Some(ops.id),
            balance_display: Decimal::from_f64(21_900.0).unwrap_or(Decimal::ZERO),
            status: WalletStatus::Active,
            transaction_rules: json!({"limit": 25000}),
            auto_split: vec![],
            allowed_transaction_types: vec!["transfer".into(), "payment".into()],
            connected_external: vec!["stripe:ops".into()],
            activity_feed_id: None,
            metadata: json!({"seed": true}),
            created_at: now,
            updated_at: now,
            portfolio_component_id: None,
        };
        ops_wallet.portfolio_component_id = Some(self.register_component(
            &ops_wallet.name,
            ComponentCategory::Item(ItemCategory::Custom("Wallet".into())),
            serde_json::to_value(&ops_wallet).unwrap_or(json!({})),
        )?);

        let mut escrow_wallet = Wallet {
            id: Uuid::new_v4(),
            wallet_type: "escrow".into(),
            owner_id,
            name: "Escrow Wallet".into(),
            linked_accounts: vec![escrow.id],
            primary_account_id: Some(escrow.id),
            balance_display: Decimal::from_f64(18_300.0).unwrap_or(Decimal::ZERO),
            status: WalletStatus::Active,
            transaction_rules: json!({"approval_required": true}),
            auto_split: vec![],
            allowed_transaction_types: vec!["escrow_release".into(), "transfer".into()],
            connected_external: vec![],
            activity_feed_id: None,
            metadata: json!({"seed": true}),
            created_at: now,
            updated_at: now,
            portfolio_component_id: None,
        };
        escrow_wallet.portfolio_component_id = Some(self.register_component(
            &escrow_wallet.name,
            ComponentCategory::Item(ItemCategory::Custom("Wallet".into())),
            serde_json::to_value(&escrow_wallet).unwrap_or(json!({})),
        )?);

        checking.linked_wallets.push(personal_wallet.id);
        ops.linked_wallets.push(ops_wallet.id);
        escrow.linked_wallets.push(escrow_wallet.id);

        self.store.accounts.insert(checking.id, checking);
        self.store.accounts.insert(ops.id, ops);
        self.store.accounts.insert(reserve.id, reserve);
        self.store.accounts.insert(escrow.id, escrow);

        self.store.wallets.insert(personal_wallet.id, personal_wallet);
        self.store.wallets.insert(ops_wallet.id, ops_wallet);
        self.store.wallets.insert(escrow_wallet.id, escrow_wallet);

        self.store.transactions.push(WalletTransaction {
            id: Uuid::new_v4(),
            tx_type: "transfer_in".into(),
            status: TransactionStatus::Cleared,
            amount: Decimal::from_f64(12_000.0).unwrap_or(Decimal::ZERO),
            currency: "USD".into(),
            from_wallet_id: None,
            to_wallet_id: Some(self.store.wallets.iter().next().map(|(id, _)| *id).unwrap_or(Uuid::nil())),
            from_account_id: None,
            to_account_id: Some(self.store.accounts.iter().next().map(|(id, _)| *id).unwrap_or(Uuid::nil())),
            reference_type: Some("invoice".into()),
            reference_id: Some("#084".into()),
            approval_status: None,
            fees: json!({}),
            metadata: json!({}),
            created_at: now,
        });

        Ok(())
    }

    fn register_component(
        &mut self,
        name: &str,
        category: ComponentCategory,
        payload: serde_json::Value,
    ) -> WalletResult<Uuid> {
        let component = self.portfolio.create_component(name, category, Some(payload))?;
        Ok(component.metadata.id)
    }
}
