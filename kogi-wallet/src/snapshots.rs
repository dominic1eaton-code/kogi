use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryCard {
    pub label: String,
    pub value: String,
    pub meta: String,
    pub tone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankingConnection {
    pub name: String,
    pub r#type: String,
    pub status: String,
    pub balance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferQueueItem {
    pub name: String,
    pub direction: String,
    pub amount: String,
    pub status: String,
    pub date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashTool {
    pub label: String,
    pub desc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub date: String,
    pub desc: String,
    pub account: String,
    pub amount: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationTask {
    pub label: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowDeal {
    pub title: String,
    pub client: String,
    pub amount: String,
    pub status: String,
    pub milestone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseScheduleItem {
    pub date: String,
    pub event: String,
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceRow {
    pub id: String,
    pub client: String,
    pub due: String,
    pub amount: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceTemplate {
    pub name: String,
    pub cadence: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestmentAllocation {
    pub label: String,
    pub value: String,
    pub tone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestmentHolding {
    pub name: String,
    pub r#type: String,
    pub value: String,
    pub change: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestmentWatch {
    pub name: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingCapRow {
    pub holder: String,
    pub shares: String,
    pub stake: String,
    pub vesting: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingDistribution {
    pub stream: String,
    pub amount: String,
    pub cadence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingLiquidityEvent {
    pub event: String,
    pub status: String,
    pub owner: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingAsset {
    pub name: String,
    pub r#type: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenefitAccountProgress {
    pub label: String,
    pub progress: String,
    pub value: String,
    pub tone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenefitProgram {
    pub name: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantPipelineItem {
    pub name: String,
    pub stage: String,
    pub amount: String,
    pub due: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantReportTask {
    pub task: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupPool {
    pub name: String,
    pub value: String,
    pub policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupPayout {
    pub member: String,
    pub amount: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignItem {
    pub name: String,
    pub r#type: String,
    pub goal: String,
    pub raised: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignInvestor {
    pub name: String,
    pub amount: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtObligation {
    pub name: String,
    pub balance: String,
    pub payment: String,
    pub rate: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtScheduleItem {
    pub date: String,
    pub item: String,
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtStrategy {
    pub label: String,
    pub desc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxCalendarItem {
    pub date: String,
    pub item: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxFiling {
    pub r#type: String,
    pub period: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxInsight {
    pub label: String,
    pub value: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardMetric {
    pub label: String,
    pub value: String,
    pub meta: String,
    pub tone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletCard {
    pub name: String,
    pub balance: String,
    pub detail: String,
    pub tag: String,
    pub tone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSummary {
    pub id: String,
    pub name: String,
    pub wallet_type: String,
    pub balance: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSummary {
    pub id: String,
    pub name: String,
    pub account_type: String,
    pub balance: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBankingSnapshot {
    pub summary_cards: Vec<SummaryCard>,
    pub connections: Vec<BankingConnection>,
    pub transfer_queue: Vec<TransferQueueItem>,
    pub cash_tools: Vec<CashTool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletLedgerSnapshot {
    pub summary_cards: Vec<SummaryCard>,
    pub ledger_entries: Vec<LedgerEntry>,
    pub reconciliation_tasks: Vec<ReconciliationTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletEscrowSnapshot {
    pub summary_cards: Vec<SummaryCard>,
    pub escrow_deals: Vec<EscrowDeal>,
    pub release_schedule: Vec<ReleaseScheduleItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInvoicesSnapshot {
    pub summary_cards: Vec<SummaryCard>,
    pub invoices: Vec<InvoiceRow>,
    pub templates: Vec<InvoiceTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInvestmentsSnapshot {
    pub summary_cards: Vec<SummaryCard>,
    pub allocations: Vec<InvestmentAllocation>,
    pub holdings: Vec<InvestmentHolding>,
    pub watchlist: Vec<InvestmentWatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletFundingSnapshot {
    pub summary_cards: Vec<SummaryCard>,
    pub cap_table: Vec<FundingCapRow>,
    pub distributions: Vec<FundingDistribution>,
    pub liquidity: Vec<FundingLiquidityEvent>,
    pub assets: Vec<FundingAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBenefitsSnapshot {
    pub summary_cards: Vec<SummaryCard>,
    pub benefit_accounts: Vec<BenefitAccountProgress>,
    pub benefit_programs: Vec<BenefitProgram>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletGrantsSnapshot {
    pub summary_cards: Vec<SummaryCard>,
    pub pipeline: Vec<GrantPipelineItem>,
    pub reporting: Vec<GrantReportTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletGroupEconomicsSnapshot {
    pub summary_cards: Vec<SummaryCard>,
    pub pools: Vec<GroupPool>,
    pub payouts: Vec<GroupPayout>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletCampaignsSnapshot {
    pub summary_cards: Vec<SummaryCard>,
    pub campaigns: Vec<CampaignItem>,
    pub investors: Vec<CampaignInvestor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletDebtsSnapshot {
    pub summary_cards: Vec<SummaryCard>,
    pub obligations: Vec<DebtObligation>,
    pub schedule: Vec<DebtScheduleItem>,
    pub strategies: Vec<DebtStrategy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletTaxesSnapshot {
    pub summary_cards: Vec<SummaryCard>,
    pub calendar: Vec<TaxCalendarItem>,
    pub filings: Vec<TaxFiling>,
    pub insights: Vec<TaxInsight>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletDashboardSnapshot {
    pub summary_cards: Vec<DashboardMetric>,
    pub wallet_cards: Vec<WalletCard>,
    pub quick_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletsOverviewSnapshot {
    pub wallets: Vec<WalletSummary>,
    pub accounts: Vec<AccountSummary>,
}

fn summary(label: &str, value: &str, meta: &str, tone: &str) -> SummaryCard {
    SummaryCard {
        label: label.to_string(),
        value: value.to_string(),
        meta: meta.to_string(),
        tone: tone.to_string(),
    }
}

pub fn sample_banking_snapshot() -> WalletBankingSnapshot {
    WalletBankingSnapshot {
        summary_cards: vec![
            summary("Total Balance", "$214,880", "Across 6 accounts", "text-[#10b981]"),
            summary("Liquidity", "$92,400", "30 day runway", "text-[#60a5fa]"),
            summary("Available Credit", "$48,000", "3 active lines", "text-[#f59e0b]"),
            summary("Reserves", "$24,900", "Emergency fund", "text-[#a855f7]"),
        ],
        connections: vec![
            BankingConnection { name: "Kogi Treasury".into(), r#type: "Primary Checking".into(), status: "Connected".into(), balance: "$48,200".into() },
            BankingConnection { name: "Ops Sweep".into(), r#type: "Sweep Account".into(), status: "Auto-sync".into(), balance: "$21,900".into() },
            BankingConnection { name: "Capital Reserve".into(), r#type: "Savings Vault".into(), status: "Connected".into(), balance: "$74,800".into() },
            BankingConnection { name: "Escrow Float".into(), r#type: "Escrow Holdings".into(), status: "Linked".into(), balance: "$18,300".into() },
        ],
        transfer_queue: vec![
            TransferQueueItem { name: "Payroll Run".into(), direction: "Outbound".into(), amount: "$18,200".into(), status: "Scheduled".into(), date: "Mar 22".into() },
            TransferQueueItem { name: "Client Retainer".into(), direction: "Inbound".into(), amount: "$12,500".into(), status: "Pending".into(), date: "Mar 21".into() },
            TransferQueueItem { name: "Investment Sweep".into(), direction: "Outbound".into(), amount: "$9,000".into(), status: "Queued".into(), date: "Mar 20".into() },
            TransferQueueItem { name: "Tax Reserve".into(), direction: "Outbound".into(), amount: "$4,200".into(), status: "Approved".into(), date: "Mar 20".into() },
        ],
        cash_tools: vec![
            CashTool { label: "Auto-Split Rules".into(), desc: "Distribute inflows across ops, tax, and investment accounts.".into() },
            CashTool { label: "Liquidity Lanes".into(), desc: "Define minimum cash levels per portfolio and project.".into() },
            CashTool { label: "Card Controls".into(), desc: "Set spend limits, merchant locks, and approval flows.".into() },
            CashTool { label: "Treasury Forecast".into(), desc: "Plan 90 day inflows, outflows, and reserves.".into() },
        ],
    }
}

pub fn sample_ledger_snapshot() -> WalletLedgerSnapshot {
    WalletLedgerSnapshot {
        summary_cards: vec![
            summary("Credits", "$42,800", "30 days", "text-[#10b981]"),
            summary("Debits", "$11,460", "30 days", "text-[#ef4444]"),
            summary("Net Flow", "$31,340", "Net positive", "text-[#60a5fa]"),
            summary("Reconciled", "92%", "12 items open", "text-[#f59e0b]"),
        ],
        ledger_entries: vec![
            LedgerEntry { date: "Mar 18".into(), desc: "Retainer Invoice #084".into(), account: "Checking".into(), amount: "+$12,000".into(), status: "Cleared".into() },
            LedgerEntry { date: "Mar 17".into(), desc: "Cloud Hosting".into(), account: "Ops".into(), amount: "-$420".into(), status: "Cleared".into() },
            LedgerEntry { date: "Mar 16".into(), desc: "Escrow Release - Deal 44".into(), account: "Escrow".into(), amount: "+$4,200".into(), status: "Pending".into() },
            LedgerEntry { date: "Mar 16".into(), desc: "Payroll Run".into(), account: "Ops".into(), amount: "-$8,900".into(), status: "Scheduled".into() },
            LedgerEntry { date: "Mar 15".into(), desc: "Dividend Deposit".into(), account: "Investment".into(), amount: "+$1,100".into(), status: "Cleared".into() },
        ],
        reconciliation_tasks: vec![
            ReconciliationTask { label: "Match 3 receipts with ops charges".into(), status: "Due Mar 22".into() },
            ReconciliationTask { label: "Confirm escrow release schedule".into(), status: "Due Mar 21".into() },
            ReconciliationTask { label: "Sync bank feed - Capital Reserve".into(), status: "Waiting on bank".into() },
        ],
    }
}

pub fn sample_escrow_snapshot() -> WalletEscrowSnapshot {
    WalletEscrowSnapshot {
        summary_cards: vec![
            summary("Active Escrows", "6", "2 pending release", "text-[#f59e0b]"),
            summary("Locked Funds", "$24,600", "Across 4 deals", "text-[#f97316]"),
            summary("Pending Release", "$6,200", "Awaiting approval", "text-[#60a5fa]"),
            summary("Disputes", "1", "Needs mediation", "text-[#ef4444]"),
        ],
        escrow_deals: vec![
            EscrowDeal { title: "Brand Identity System".into(), client: "Acme Corp".into(), amount: "$4,200".into(), status: "Locked".into(), milestone: "Design review".into() },
            EscrowDeal { title: "Data Pipeline Audit".into(), client: "DataStream Co".into(), amount: "$2,800".into(), status: "Pending".into(), milestone: "Final report".into() },
            EscrowDeal { title: "Mobile App MVP".into(), client: "NovaTech".into(), amount: "$7,500".into(), status: "Release Ready".into(), milestone: "QA signoff".into() },
            EscrowDeal { title: "DAO Tooling Sprint".into(), client: "Collective Studio".into(), amount: "$10,100".into(), status: "Milestone 2".into(), milestone: "Sprint demo".into() },
        ],
        release_schedule: vec![
            ReleaseScheduleItem { date: "Mar 20".into(), event: "Release - Mobile App MVP".into(), amount: "$3,000".into() },
            ReleaseScheduleItem { date: "Mar 22".into(), event: "Release - Brand Identity System".into(), amount: "$1,200".into() },
            ReleaseScheduleItem { date: "Mar 25".into(), event: "Release - DAO Tooling Sprint".into(), amount: "$4,000".into() },
        ],
    }
}

pub fn sample_invoices_snapshot() -> WalletInvoicesSnapshot {
    WalletInvoicesSnapshot {
        summary_cards: vec![
            summary("Receivable", "$28,400", "12 open invoices", "text-[#10b981]"),
            summary("Payable", "$6,900", "5 bills", "text-[#ef4444]"),
            summary("Overdue", "$1,200", "1 invoice", "text-[#f59e0b]"),
            summary("Drafts", "3", "Need approval", "text-[#60a5fa]"),
        ],
        invoices: vec![
            InvoiceRow { id: "#091".into(), client: "Venture Partners".into(), due: "Mar 25".into(), amount: "$12,000".into(), status: "Sent".into() },
            InvoiceRow { id: "#090".into(), client: "Pixel Studio".into(), due: "Mar 22".into(), amount: "$3,400".into(), status: "Viewed".into() },
            InvoiceRow { id: "#089".into(), client: "Cloudflare".into(), due: "Mar 20".into(), amount: "$200".into(), status: "Scheduled".into() },
            InvoiceRow { id: "#088".into(), client: "Collective Studio".into(), due: "Mar 18".into(), amount: "$1,800".into(), status: "Overdue".into() },
        ],
        templates: vec![
            InvoiceTemplate { name: "Retainer Invoice".into(), cadence: "Monthly".into(), status: "Active".into() },
            InvoiceTemplate { name: "Escrow Release".into(), cadence: "Milestone".into(), status: "Active".into() },
            InvoiceTemplate { name: "Grant Disbursement".into(), cadence: "Quarterly".into(), status: "Paused".into() },
        ],
    }
}

pub fn sample_investments_snapshot() -> WalletInvestmentsSnapshot {
    WalletInvestmentsSnapshot {
        summary_cards: vec![
            summary("Portfolio Value", "$184,600", "Long-term holdings", "text-[#10b981]"),
            summary("Gain / Loss", "+$12,400", "Year to date", "text-[#60a5fa]"),
            summary("Allocation", "62% Equity", "Balanced mix", "text-[#a855f7]"),
            summary("Liquidity", "$32,000", "Available to deploy", "text-[#f59e0b]"),
        ],
        allocations: vec![
            InvestmentAllocation { label: "Equities".into(), value: "42%".into(), tone: "bg-[#10b981]".into() },
            InvestmentAllocation { label: "Funds".into(), value: "18%".into(), tone: "bg-[#60a5fa]".into() },
            InvestmentAllocation { label: "Real Estate".into(), value: "16%".into(), tone: "bg-[#f59e0b]".into() },
            InvestmentAllocation { label: "REITs".into(), value: "12%".into(), tone: "bg-[#a855f7]".into() },
            InvestmentAllocation { label: "Crypto".into(), value: "6%".into(), tone: "bg-[#f97316]".into() },
            InvestmentAllocation { label: "Cash".into(), value: "6%".into(), tone: "bg-[#22c55e]".into() },
        ],
        holdings: vec![
            InvestmentHolding { name: "Kogi Equity Fund".into(), r#type: "Fund".into(), value: "$48,000".into(), change: "+4.2%".into() },
            InvestmentHolding { name: "Atlas REIT".into(), r#type: "REIT".into(), value: "$28,600".into(), change: "+2.1%".into() },
            InvestmentHolding { name: "Solar Grid Notes".into(), r#type: "Private".into(), value: "$18,400".into(), change: "+7.5%".into() },
            InvestmentHolding { name: "Blue River Shares".into(), r#type: "Equity".into(), value: "$24,300".into(), change: "-1.2%".into() },
        ],
        watchlist: vec![
            InvestmentWatch { name: "Community Land Trust".into(), status: "Due diligence".into() },
            InvestmentWatch { name: "Green Logistics Bond".into(), status: "Term sheet".into() },
            InvestmentWatch { name: "Impact Crowdfund".into(), status: "Monitoring".into() },
        ],
    }
}

pub fn sample_funding_snapshot() -> WalletFundingSnapshot {
    WalletFundingSnapshot {
        summary_cards: vec![
            summary("Equity Issued", "72%", "Cap table utilization", "text-[#10b981]"),
            summary("Royalty Streams", "$6,400", "Monthly forecast", "text-[#60a5fa]"),
            summary("Dividend Pool", "$18,000", "Next payout", "text-[#f59e0b]"),
            summary("Liquidity Events", "2", "IPO / ICO tracking", "text-[#a855f7]"),
        ],
        cap_table: vec![
            FundingCapRow { holder: "Founder Pool".into(), shares: "420,000".into(), stake: "42%".into(), vesting: "4y cliff 1y".into() },
            FundingCapRow { holder: "Community Equity".into(), shares: "220,000".into(), stake: "22%".into(), vesting: "Mission based".into() },
            FundingCapRow { holder: "Investors Series A".into(), shares: "180,000".into(), stake: "18%".into(), vesting: "Preferred".into() },
            FundingCapRow { holder: "Advisors".into(), shares: "60,000".into(), stake: "6%".into(), vesting: "18 mo".into() },
        ],
        distributions: vec![
            FundingDistribution { stream: "Product Royalties".into(), amount: "$2,400".into(), cadence: "Monthly".into() },
            FundingDistribution { stream: "License Fees".into(), amount: "$1,600".into(), cadence: "Quarterly".into() },
            FundingDistribution { stream: "Dividend Reserve".into(), amount: "$18,000".into(), cadence: "Semiannual".into() },
        ],
        liquidity: vec![
            FundingLiquidityEvent { event: "IPO Prep".into(), status: "Due diligence".into(), owner: "Finance Council".into() },
            FundingLiquidityEvent { event: "Community ICO".into(), status: "Tokenomics review".into(), owner: "Governance".into() },
            FundingLiquidityEvent { event: "Secondary Sale".into(), status: "Draft terms".into(), owner: "Legal".into() },
        ],
        assets: vec![
            FundingAsset { name: "Estate Holding Trust".into(), r#type: "Trust".into(), value: "$140,000".into() },
            FundingAsset { name: "Kogi Studio HQ".into(), r#type: "Real Estate".into(), value: "$320,000".into() },
            FundingAsset { name: "Heritage Reserve".into(), r#type: "Estate".into(), value: "$90,000".into() },
        ],
    }
}

pub fn sample_benefits_snapshot() -> WalletBenefitsSnapshot {
    WalletBenefitsSnapshot {
        summary_cards: vec![
            summary("HSA Balance", "$3,200", "Annual goal $4,150", "text-[#06b6d4]"),
            summary("IRA Balance", "$42,800", "Contribution 74%", "text-[#10b981]"),
            summary("Portable Benefits", "Active", "4 programs", "text-[#a855f7]"),
            summary("Coverage", "98%", "Insurance current", "text-[#f59e0b]"),
        ],
        benefit_accounts: vec![
            BenefitAccountProgress { label: "Solo 401(k)".into(), progress: "79%".into(), value: "$18,200 / $23,000".into(), tone: "bg-[#10b981]".into() },
            BenefitAccountProgress { label: "HSA".into(), progress: "58%".into(), value: "$2,400 / $4,150".into(), tone: "bg-[#06b6d4]".into() },
            BenefitAccountProgress { label: "IRA".into(), progress: "74%".into(), value: "$4,800 / $6,500".into(), tone: "bg-[#a855f7]".into() },
            BenefitAccountProgress { label: "Health Coverage".into(), progress: "100%".into(), value: "Active through Mar 2027".into(), tone: "bg-[#f59e0b]".into() },
        ],
        benefit_programs: vec![
            BenefitProgram { name: "Portable PTO".into(), status: "24 hours accrued".into() },
            BenefitProgram { name: "Education Stipend".into(), status: "$1,200 remaining".into() },
            BenefitProgram { name: "Wellness Fund".into(), status: "Quarterly reload".into() },
        ],
    }
}

pub fn sample_grants_snapshot() -> WalletGrantsSnapshot {
    WalletGrantsSnapshot {
        summary_cards: vec![
            summary("Active Grants", "7", "$82,000 total", "text-[#10b981]"),
            summary("Submitted", "4", "Awaiting review", "text-[#60a5fa]"),
            summary("Awarded", "$28,500", "Last 90 days", "text-[#f59e0b]"),
            summary("Compliance", "93%", "Reports on time", "text-[#a855f7]"),
        ],
        pipeline: vec![
            GrantPipelineItem { name: "Green Futures Fund".into(), stage: "Submitted".into(), amount: "$18,000".into(), due: "Apr 2".into() },
            GrantPipelineItem { name: "Community Build Grant".into(), stage: "Negotiation".into(), amount: "$12,500".into(), due: "Mar 28".into() },
            GrantPipelineItem { name: "Open Source Fellowship".into(), stage: "Awarded".into(), amount: "$6,000".into(), due: "Mar 20".into() },
            GrantPipelineItem { name: "Education Access".into(), stage: "Draft".into(), amount: "$4,500".into(), due: "Apr 8".into() },
        ],
        reporting: vec![
            GrantReportTask { task: "Impact report - Open Source Fellowship".into(), status: "Due Apr 10".into() },
            GrantReportTask { task: "Financial audit - Community Build".into(), status: "Due Apr 15".into() },
            GrantReportTask { task: "Milestone update - Green Futures".into(), status: "Due Apr 20".into() },
        ],
    }
}

pub fn sample_group_economics_snapshot() -> WalletGroupEconomicsSnapshot {
    WalletGroupEconomicsSnapshot {
        summary_cards: vec![
            summary("Group Treasury", "$128,000", "Shared funds", "text-[#10b981]"),
            summary("Distribution Cycle", "Biweekly", "Next run Mar 22", "text-[#60a5fa]"),
            summary("Reserve Pool", "$24,500", "Stability fund", "text-[#f59e0b]"),
            summary("Allocation Rules", "6", "Active policies", "text-[#a855f7]"),
        ],
        pools: vec![
            GroupPool { name: "Operations Pool".into(), value: "$48,200".into(), policy: "40% of inflows".into() },
            GroupPool { name: "Community Dividend".into(), value: "$22,400".into(), policy: "20% of surplus".into() },
            GroupPool { name: "Innovation Fund".into(), value: "$31,100".into(), policy: "25% of revenue".into() },
            GroupPool { name: "Mutual Aid".into(), value: "$9,800".into(), policy: "15% of grants".into() },
        ],
        payouts: vec![
            GroupPayout { member: "Studio Collective".into(), amount: "$3,400".into(), status: "Scheduled".into() },
            GroupPayout { member: "Open Source Guild".into(), amount: "$2,100".into(), status: "Pending vote".into() },
            GroupPayout { member: "Coop Team Alpha".into(), amount: "$1,600".into(), status: "Approved".into() },
        ],
    }
}

pub fn sample_campaigns_snapshot() -> WalletCampaignsSnapshot {
    WalletCampaignsSnapshot {
        summary_cards: vec![
            summary("Active Campaigns", "3", "Crowdfund + equity", "text-[#10b981]"),
            summary("Total Raised", "$92,400", "Across 180 backers", "text-[#60a5fa]"),
            summary("Open Pledges", "$12,600", "Pending close", "text-[#f59e0b]"),
            summary("Grant Match", "$18,000", "Match pipeline", "text-[#a855f7]"),
        ],
        campaigns: vec![
            CampaignItem { name: "Community Studio Build".into(), r#type: "Crowdfund".into(), goal: "$60,000".into(), raised: "$42,000".into(), status: "Live".into() },
            CampaignItem { name: "Equity Round - Seed".into(), r#type: "Equity".into(), goal: "$120,000".into(), raised: "$68,500".into(), status: "Open".into() },
            CampaignItem { name: "Mutual Aid Pool".into(), r#type: "Donation".into(), goal: "$15,000".into(), raised: "$12,900".into(), status: "Final week".into() },
        ],
        investors: vec![
            CampaignInvestor { name: "Lumen Ventures".into(), amount: "$12,000".into(), status: "Committed".into() },
            CampaignInvestor { name: "Northwind Collective".into(), amount: "$6,500".into(), status: "Pending".into() },
            CampaignInvestor { name: "Open Source Guild".into(), amount: "$4,200".into(), status: "Matched".into() },
        ],
    }
}

pub fn sample_debts_snapshot() -> WalletDebtsSnapshot {
    WalletDebtsSnapshot {
        summary_cards: vec![
            summary("Total Debt", "$48,600", "Across 5 obligations", "text-[#ef4444]"),
            summary("Monthly Payments", "$3,420", "Auto-pay enabled", "text-[#f59e0b]"),
            summary("Interest Rate", "6.2%", "Weighted average", "text-[#60a5fa]"),
            summary("Payoff ETA", "18 mo", "Current schedule", "text-[#10b981]"),
        ],
        obligations: vec![
            DebtObligation { name: "Studio Credit Line".into(), balance: "$18,200".into(), payment: "$1,200".into(), rate: "5.4%".into(), status: "Current".into() },
            DebtObligation { name: "Equipment Lease".into(), balance: "$9,600".into(), payment: "$620".into(), rate: "7.1%".into(), status: "Current".into() },
            DebtObligation { name: "Bridge Loan".into(), balance: "$12,800".into(), payment: "$940".into(), rate: "6.8%".into(), status: "Auto-pay".into() },
            DebtObligation { name: "Vendor Payable".into(), balance: "$8,000".into(), payment: "$660".into(), rate: "0%".into(), status: "Negotiating".into() },
        ],
        schedule: vec![
            DebtScheduleItem { date: "Mar 20".into(), item: "Studio Credit Line".into(), amount: "$1,200".into() },
            DebtScheduleItem { date: "Mar 22".into(), item: "Equipment Lease".into(), amount: "$620".into() },
            DebtScheduleItem { date: "Mar 25".into(), item: "Bridge Loan".into(), amount: "$940".into() },
            DebtScheduleItem { date: "Mar 28".into(), item: "Vendor Payable".into(), amount: "$660".into() },
        ],
        strategies: vec![
            DebtStrategy { label: "Snowball Plan".into(), desc: "Prioritize smallest balances to reduce accounts quickly.".into() },
            DebtStrategy { label: "Avalanche Plan".into(), desc: "Target highest interest first to reduce total cost.".into() },
            DebtStrategy { label: "Refinance Review".into(), desc: "Evaluate lower rate offers for the credit line.".into() },
        ],
    }
}

pub fn sample_taxes_snapshot() -> WalletTaxesSnapshot {
    WalletTaxesSnapshot {
        summary_cards: vec![
            summary("Tax Reserve", "$6,900", "24% rate", "text-[#ef4444]"),
            summary("Estimated Due", "$2,100", "Next payment", "text-[#f59e0b]"),
            summary("Withholding", "$1,400", "Auto-sweep", "text-[#60a5fa]"),
            summary("Compliance", "On track", "Q1 filings", "text-[#10b981]"),
        ],
        calendar: vec![
            TaxCalendarItem { date: "Apr 15".into(), item: "Estimated tax payment".into(), status: "Upcoming".into() },
            TaxCalendarItem { date: "May 15".into(), item: "Sales tax remittance".into(), status: "Scheduled".into() },
            TaxCalendarItem { date: "Jun 30".into(), item: "Quarterly filing".into(), status: "Draft".into() },
        ],
        filings: vec![
            TaxFiling { r#type: "Federal".into(), period: "Q1 2026".into(), status: "In prep".into() },
            TaxFiling { r#type: "State".into(), period: "Q1 2026".into(), status: "Not started".into() },
            TaxFiling { r#type: "Local".into(), period: "Monthly".into(), status: "Current".into() },
        ],
        insights: vec![
            TaxInsight { label: "Projected Liability".into(), value: "$8,600".into(), note: "Based on YTD income".into() },
            TaxInsight { label: "Safe Harbor".into(), value: "92%".into(), note: "Target 100%".into() },
            TaxInsight { label: "Expense Coverage".into(), value: "$4,200".into(), note: "Eligible deductions".into() },
        ],
    }
}

pub fn sample_dashboard_snapshot() -> WalletDashboardSnapshot {
    WalletDashboardSnapshot {
        summary_cards: vec![
            DashboardMetric { label: "Net Worth".into(), value: "$142,800".into(), meta: "+4.2% over 30d".into(), tone: "text-[#10b981]".into() },
            DashboardMetric { label: "Liquid Cash".into(), value: "$28,400".into(), meta: "Checking + Ops".into(), tone: "text-[#e6f1f4]".into() },
            DashboardMetric { label: "Invested".into(), value: "$89,200".into(), meta: "+1.8% over 7d".into(), tone: "text-[#06b6d4]".into() },
            DashboardMetric { label: "In Escrow".into(), value: "$18,300".into(), meta: "4 active deals".into(), tone: "text-[#f59e0b]".into() },
            DashboardMetric { label: "Tax Reserve".into(), value: "$6,900".into(), meta: "24% rate Q1".into(), tone: "text-[#ef4444]".into() },
        ],
        wallet_cards: vec![
            WalletCard { name: "Checking".into(), balance: "$14,200".into(), detail: "Personal � KWLT-001".into(), tag: "Primary".into(), tone: "text-[#10b981]".into() },
            WalletCard { name: "Operations".into(), balance: "$14,200".into(), detail: "Business Ops � KWLT-002".into(), tag: "Business".into(), tone: "text-[#f59e0b]".into() },
            WalletCard { name: "Investment".into(), balance: "$89,200".into(), detail: "Long-term � KWLT-003".into(), tag: "Invested".into(), tone: "text-[#a855f7]".into() },
            WalletCard { name: "Tax Reserve".into(), balance: "$6,900".into(), detail: "SE Tax � KWLT-004".into(), tag: "Q1 2026".into(), tone: "text-[#ef4444]".into() },
        ],
        quick_actions: vec![
            "Send".into(),
            "Receive".into(),
            "Transfer".into(),
            "Invoice".into(),
        ],
    }
}
