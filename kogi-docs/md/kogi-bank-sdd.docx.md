

**KOGI PLATFORM**

**kogi-bank**

**Banking, Finance & Capital System**

*System Design Document*

v1.0  ·  Kogi Platform  ·  March 2026

| KWLT  ·  kogi-bank  ·  KFIN  ·  KESC  ·  KLGR *Banking for independent workers, autonomous organizations, collectives,* *cooperatives, and independent teams — the financial operating layer of the Kogi Platform.* |
| ----- |

Kogi Team  ·  Independent Worker Operating System  ·  Confidential

# **Table of Contents**

| 1\. OVERVIEW |
| :---- |

## **1.1 Purpose & Vision**

kogi-bank is the financial operating layer of the Kogi Platform — the centralized banking, accounting, payment, escrow, investment, and capital management system for independent workers, autonomous organizations, collectives, cooperatives, and independent teams. It is the platform module referenced as KWLT (Kogi Wallet) in earlier design documents, now formalized as kogi-bank to reflect its full scope: it is not merely a wallet but a complete financial operating infrastructure.

Where kogi-office organizes work, kogi-community connects people, and kogi-marketplace enables trade, kogi-bank ensures that all financial flows — from a single invoice to a multi-party equity round — are governed, auditable, and intelligently managed. Every transaction, every account balance, every escrow lock, and every campaign contribution flows through kogi-bank.

| *Design Principle: Accounts are stores of capital, resources, liquidity, and equity. Wallets are transaction points — the active interfaces through which financial entities move. Every financial event in the platform is a first-class Portfolio Item in the Portfolio System, giving every financial entity the same governance, versioning, and analytics infrastructure as any other platform component.* |
| :---- |

## **1.2 Core Design Distinctions: Accounts vs. Wallets**

A foundational architectural distinction governs the entire kogi-bank system:

| Concept | Description |
| :---- | :---- |
| **Account** | A store of capital, resources, liquidity, or equity. Accounts hold balances — they are the authoritative record of what is owned or owed. Think: checking account, investment account, escrow reserve, equity stake register. |
| **Wallet** | A transaction point — the active interface through which financial items move. Wallets are operational surfaces for sending, receiving, and routing. A wallet links to one or more accounts and executes transactions on their behalf. Think: spending wallet, payout wallet, campaign wallet. |
| **Ledger** | The immutable, append-only record of every financial event. The ledger is the source of truth for all account balances — balances are always computed from the ledger, never stored independently. |
| **Journal** | A classified grouping of ledger entries by type — income, expense, asset, liability, equity. Journals are the accounting layer above raw ledger entries. |

## **1.3 Position in the Kogi Platform**

| Layer | System | Relationship to kogi-bank |
| :---- | :---- | :---- |
| Upstream | kogi-marketplace / kogi-exchange | All deal completions, escrow operations, and campaign contributions route through kogi-bank |
| Upstream | kogi-gig | Worker scheduling and contract completions trigger payroll, invoicing, and payment events |
| Upstream | kogi-portfolio (Portfolio System) | Budget allocations and resource spend recorded via ResourceAllocation (Budget kind) |
| Downstream | kogi-engine (RiskEngine) | Financial signals (cashFlow, portfolio health) consumed for risk scoring |
| Downstream | kogi-engine (AnalyticsEngine) | Financial StreamEvents feed the analytics and recommendation pipelines |
| Peer | kogi-community / kogi-organizations | Cooperative and collective banking, shared funds, campaign disbursements |
| Peer | kogi-marketplace / kogi-exchange | Escrow lock/release, deal settlement, investment disbursement |
| External | Stripe, Plaid, PayPal, ACH, Crypto bridges | Payment rails, bank account linking, on-chain settlement |

## **1.4 Module Codes & Services**

| Module | Code | Service | Language |
| :---- | :---- | :---- | :---- |
| Bank Core | KBNK | kogi-bank | Go (service) · Rust (BankSystem core) |
| Wallet | KWLT | kogi-wallet | Go (service) |
| Ledger | KLGR | kogi-ledger | Go (service) · PostgreSQL (append-only) |
| Escrow | KESC | kogi-escrow | Go (service) · Rust |
| Accounts | KACC | kogi-accounts | Go (service) |
| Campaigns | KCMP | kogi-campaigns | Go (service) |
| Invoicing | KINV | kogi-invoices | Go (service) |
| Tax & Accounting | KTAX | kogi-tax | Go (service) · Scala (pipelines) |

| 2\. ACCOUNT SYSTEM |
| :---- |

## **2.1 Concept**

Accounts are the foundational stores of financial value in kogi-bank. An account represents a defined pool of capital, resources, liquidity, or equity belonging to a specific entity — an individual worker, an organization, a collective, a cooperative, or an independent team. Every account has an owner, a currency or unit denomination, a type classification, and a balance computed from its ledger entries.

Accounts are Portfolio Items at the system level — they carry the full ComponentMetadata structure (id, owners, tags, policy\_ids, vector\_clock, version) and are governed by the Portfolio System's PermissionTier hierarchy and PolicyEngine. This means an account's full audit trail, governance policies, and access controls are native to the same infrastructure as all other platform entities.

## **2.2 Account Types**

### **2.2.1 Personal Accounts (Individual Worker)**

| Account Type | Description | Currency/Unit | Primary Use |
| :---- | :---- | :---- | :---- |
| Personal Checking | Day-to-day spending and income reception | Fiat (USD, EUR, etc.) | Receive client payments, pay expenses |
| Personal Savings | Reserved capital for specific goals or emergencies | Fiat | Emergency fund, goal-based saving |
| Operations Account | Dedicated business-expense account for gig/freelance activity | Fiat | Software subscriptions, equipment, tools |
| Investment Account | Long-term capital deployment in equity, funds, or portfolio assets | Fiat \+ equity units | Equity stakes, platform investments |
| Tax Reserve Account | Automated set-aside for self-employment tax obligations | Fiat | Quarterly tax payments, year-end filing |
| Retirement Account | Long-term personal retirement savings (portable benefit) | Fiat \+ fund units | IRA-equivalent, SEP, solo 401(k) analogues |
| Equity Account | Record of equity positions in external organizations or portfolio projects | Equity units / shares | Cap table holdings, vesting schedules |
| Resource Account | Non-monetary resource pools (compute credits, license units, hours) | Custom units | Platform compute, license pools, time credits |
| Trading Account | Active trading and exchange activity | Fiat \+ tokens | kogi-exchange activity, market orders |
| Crypto/Token Account | Digital asset and token holdings | Tokens / coins | Web3 assets, platform tokens, NFTs |

### **2.2.2 Organizational Accounts (Autonomous Org, Collective, Cooperative, Team)**

When an organization, collective, cooperative, or team is created in kogi-organizations, kogi-bank provisions a corresponding set of organizational accounts. These accounts are owned by the organization entity and governed by its member permissions and governance policies.

| Account Type | Description | Entity Types | Notes |
| :---- | :---- | :---- | :---- |
| Operating Account | Primary revenue and expense account for the organization | Org, Collective, Cooperative, Team | Multi-signatory controls for large withdrawals |
| Payroll Account | Funds designated for member/worker compensation | Org, Cooperative, Team | Automated distribution per payroll schedule |
| Project Account | Per-project budget pool; linked to a specific Portfolio Project | All | Budget ring-fenced per project; spend tracked in Portfolio System |
| Reserve Account | Organizational reserves and contingency capital | Org, Cooperative | Governed by reserve policy; minimum balance enforced |
| Investment Account | Collective investment capital deployed in portfolio assets or external opportunities | All | Multi-party approval required for deployment |
| Equity Pool | Shared equity register for member equity distribution | Cooperative, DAO | Records vesting, cliff dates, distribution events |
| Campaign Account | Dedicated account for an active fundraising or resource campaign | All | Funds held until campaign resolution; auto-disbursed on success |
| Escrow Reserve | Collective escrow for multi-party deals | All | Funds released per approval workflow |
| Tax Account | Shared tax reserve for organizational obligations | Org, Cooperative | Automated allocation from operating revenue |
| Mutual Aid Fund | Peer-support pool for member emergencies or collective needs | Collective, Cooperative | Governed by mutual aid policy |

## **2.3 Account Data Model**

| Field | Description |
| :---- | :---- |
| **account.id** | Platform-unique UUID |
| **account.type** | checking | savings | operations | investment | tax\_reserve | retirement | equity | resource | trading | crypto | payroll | project | reserve | equity\_pool | campaign | escrow\_reserve | tax | mutual\_aid |
| **account.owner\_id** | Owner entity UUID — user, organization, collective, cooperative, or team |
| **account.owner\_type** | user | organization | collective | cooperative | team |
| **account.name** | Display name of the account |
| **account.currency** | ISO 4217 currency code (USD, EUR, GBP) or custom unit code |
| **account.denomination** | fiat | equity\_units | tokens | resource\_units | custom |
| **account.balance** | Computed: sum of all ledger credits minus debits for this account |
| **account.available\_balance** | Balance minus any pending holds, escrow locks, or reserved amounts |
| **account.pending\_balance** | Amount currently in unconfirmed or in-flight transactions |
| **account.status** | active | frozen | suspended | closed | archived |
| **account.linked\_external** | External account link IDs (via Plaid, Stripe, etc.) if connected |
| **account.linked\_wallets\[\]** | Wallet IDs that can transact against this account |
| **account.policy\_ids\[\]** | Governance policy IDs: withdrawal limits, multi-sig rules, reserve minimums |
| **account.portfolio\_component\_id** | Linked Portfolio Component ID — account is itself a Portfolio Item |
| **account.metadata** | Full ComponentMetadata: id, owners, tags, vector\_clock, version, created\_at, updated\_at |

## **2.4 Account Lifecycle**

| State | Description | Transitions |
| :---- | :---- | :---- |
| Provisioned | Account created but not yet activated. Zero balance. | → Active (deposit or verification) |
| Active | Fully operational. Transactions permitted. | → Frozen, → Suspended, → Closed |
| Frozen | Temporarily locked. No outbound transactions. Deposits allowed. | → Active (unfreeze), → Suspended |
| Suspended | Compliance or policy hold. No transactions. | → Active (resolution), → Closed |
| Closed | Account permanently closed. Balance must be zero or transferred. | → Archived |
| Archived | Historical record only. No transactions. | (Terminal) |

## **2.5 Multi-Signatory & Approval Controls**

Organizational and high-value accounts support configurable multi-signatory controls, enforced through the Portfolio System's ApprovalWorkflow and PolicyEngine:

* Single-sig accounts — standard individual accounts; one authorized user can transact

* 2-of-N multi-sig — two approvers required from a defined set of N authorized signatories

* M-of-N multi-sig — M approvers from N required; configurable per account and per transaction threshold

* Time-locked approvals — transactions above a threshold require a time delay before execution (cooling-off window)

* DAO governance approval — organizational accounts linked to on-chain or platform-native voting for large disbursements

* Delegated authority — specific signatories can be granted time-limited authority to approve up to a specified amount

| 3\. WALLET SYSTEM |
| :---- |

## **3.1 Concept**

Wallets are the active transaction surfaces of kogi-bank. While accounts store value, wallets move it. A wallet is a named, purpose-defined transaction interface that routes financial operations to and from one or more underlying accounts. Users interact with wallets daily; accounts are the deeper financial record layer.

Every user and organization has a primary wallet, but can configure multiple purpose-specific wallets — a personal spending wallet, a gig income wallet, a marketplace wallet for buying and selling, and a campaign wallet for fundraising activity. Each wallet has a defined flow direction, linked accounts, transaction rules, and an activity feed.

## **3.2 Wallet Types**

| Wallet Type | Description | Typical Linked Accounts | Primary Use |
| :---- | :---- | :---- | :---- |
| Personal Wallet | Day-to-day financial activity | Personal Checking, Savings | Receive income, pay expenses, transfer funds |
| Operations Wallet | Business expense management for gig activity | Operations Account | Subscriptions, tools, equipment, platform fees |
| Marketplace Wallet | Buying and selling in kogi-marketplace | Checking, Investment | Purchase assets, receive sales proceeds |
| Exchange Wallet | Active trading and deal execution | Trading Account, Crypto | Exchange orders, deal escrow, bid funding |
| Investment Wallet | Deploying capital in equity, campaigns, and instruments | Investment Account, Equity Account | Back campaigns, invest in rounds, buy equity |
| Payroll Wallet | Receive compensation from contracts and gigs | Personal Checking | Gig payouts, contract milestone receipts |
| Tax Wallet | Automated tax savings and payments | Tax Reserve Account | Estimated quarterly taxes, annual filing |
| Campaign Wallet | Manage and disburse campaign funds | Campaign Account | Launch campaigns, track contributions, disburse |
| Org Wallet | Organizational operational wallet | Operating Account, Payroll Account | Org-wide spend, payroll, vendor payments |
| Collective Wallet | Shared wallet for collective/cooperative member activity | Mutual Aid Fund, Reserve Account | Collective purchases, shared resources, mutual aid |
| Escrow Wallet | Temporary custody of deal funds | Escrow Reserve Account | Deal-in-progress fund custody |
| Crypto Wallet | Digital asset transactions | Crypto/Token Account | On-chain transfers, token purchases, DeFi activity |

## **3.3 Wallet Data Model**

| Field | Description |
| :---- | :---- |
| **wallet.id** | Platform-unique UUID |
| **wallet.type** | personal | operations | marketplace | exchange | investment | payroll | tax | campaign | org | collective | escrow | crypto |
| **wallet.owner\_id** | Owner entity UUID |
| **wallet.name** | Display name |
| **wallet.linked\_accounts\[\]** | Account IDs this wallet can transact against |
| **wallet.primary\_account\_id** | Default account for this wallet's transactions |
| **wallet.balance\_display** | Computed aggregate of linked account balances for display |
| **wallet.status** | active | paused | frozen | closed |
| **wallet.transaction\_rules** | JSONB: spending limits, auto-save rules, auto-tax-split rules |
| **wallet.auto\_split\[\]** | Automatic percentage routing to linked accounts on incoming funds |
| **wallet.allowed\_transaction\_types\[\]** | Subset of transaction types permitted for this wallet |
| **wallet.connected\_external\[\]** | External payment method IDs (Stripe cards, Plaid accounts, crypto addresses) |
| **wallet.activity\_feed\_id** | Linked activity feed for real-time transaction notifications |
| **wallet.metadata** | Full ComponentMetadata |

## **3.4 Auto-Split Rules**

A powerful Wallet feature is configurable auto-split: when income arrives in a wallet, it is automatically routed to multiple accounts according to defined percentage rules. This enables independent workers to automate their financial discipline without manual allocation:

| *Example: Jordan sets up the following auto-split on their Payroll Wallet: 60% → Personal Checking, 20% → Operations Account, 15% → Tax Reserve, 5% → Investment Account. Every time a contract payment or gig payout lands, kogi-bank automatically applies the split, and all four ledgers are updated atomically.* |
| :---- |

| Auto-Split Rule Type | Description | Use Case |
| :---- | :---- | :---- |
| Percentage Split | Fixed % to each destination account | Tax reserve automation, savings allocation |
| Fixed Amount First | A fixed dollar amount to one account, remainder to another | Bill reserve before discretionary |
| Threshold-Triggered | Split activates only when deposit exceeds a threshold | Overflow to investment once bills covered |
| Category-Based | Route based on payer/source category | Client payments → Operations, marketplace → Marketplace Wallet |
| Goal-Linked | Route to account until a balance goal is reached, then switch | Fill tax reserve to $X, then route to savings |

## **3.5 External Payment Method Linking**

Wallets can be linked to external financial instruments via the kogi-providers ProviderSystem:

* Bank accounts (ACH/wire) — linked via Plaid; balance visible, transfers initiated from kogi-bank

* Debit/credit cards — linked via Stripe; card-on-file for platform purchases

* PayPal / Venmo / Cash App — linked via OAuth; cross-platform transfers

* Crypto wallets — linked via WalletConnect; ETH, BTC, SOL, and ERC-20 tokens

* Robinhood / SoFi / brokerage accounts — read-only balance visibility; manual transfer

* Stripe Connect — seller payout routing for marketplace proceeds

| 4\. LEDGER, JOURNALS & ACCOUNTING |
| :---- |

## **4.1 Double-Entry Ledger**

The kogi-bank ledger implements double-entry bookkeeping — the accounting standard used by every professional financial institution. Every financial event creates at least two ledger entries: a debit to one account and a credit to another. This ensures the ledger is always in balance, and that every transaction has a clear origin and destination.

The ledger is append-only and immutable. No entry is ever modified or deleted; corrections are made by posting a reversing entry. This design mirrors the Portfolio System's EventLog (also append-only) and provides a complete, tamper-evident audit trail for every financial event in the platform.

| Accounting Principle | Debits increase asset/expense accounts; decrease liability/equity/revenue accounts. Credits do the inverse. The fundamental equation Assets \= Liabilities \+ Equity holds at all times. |
| :---- | :---- |

## **4.2 Ledger Entry Data Model**

| Field | Description |
| :---- | :---- |
| **entry.id** | Platform-unique UUID |
| **entry.ledger\_id** | Parent ledger reference (one ledger per account owner) |
| **entry.journal\_id** | Journal classification (income, expense, asset, liability, equity, escrow, campaign) |
| **entry.transaction\_id** | Originating transaction UUID (groups the debit/credit pair) |
| **entry.account\_id** | Account this entry affects |
| **entry.type** | debit | credit |
| **entry.amount** | Decimal amount with currency code |
| **entry.balance\_after** | Running balance of the account after this entry (computed, cached for performance) |
| **entry.entry\_type** | income | expense | transfer | escrow\_lock | escrow\_release | investment | dividend | fee | tax | refund | donation | grant | payroll | adjustment | reversal |
| **entry.description** | Human-readable description |
| **entry.reference** | External reference (invoice ID, deal ID, order ID, campaign ID, etc.) |
| **entry.counterparty\_id** | Entity on the other side of the transaction |
| **entry.category** | User-assigned or AI-auto-categorized expense/income category |
| **entry.reconciled** | Boolean: matched against external bank statement |
| **entry.timestamp** | Nanosecond-precision event timestamp |
| **entry.metadata** | Vector clock, policy\_ids, tags, audit trail fields |

## **4.3 Journal Classification**

Every ledger entry belongs to a journal. Journals group entries by financial nature and drive the accounting reports, tax calculations, and P\&L generation:

| Journal | Entry Types | Drives |
| :---- | :---- | :---- |
| Income Journal | Client payments, marketplace proceeds, gig payouts, subscription revenue | Revenue recognition, P\&L income line |
| Expense Journal | Subscriptions, tools, equipment, platform fees, professional services | P\&L expense line, deductible tracking |
| Asset Journal | Equipment purchases, IP acquisitions, investment deployments | Balance sheet assets |
| Liability Journal | Loans, deferred revenue, outstanding payables | Balance sheet liabilities |
| Equity Journal | Equity issuances, cap table changes, vesting events | Balance sheet equity, cap table |
| Escrow Journal | Escrow locks and releases from marketplace/exchange deals | Deal accounting, held funds |
| Campaign Journal | Campaign contributions received and disbursed | Campaign accounting, donor records |
| Tax Journal | Tax reserve allocations, estimated tax payments, tax refunds | Tax liability tracking |
| Payroll Journal | Member/worker compensation payments | Payroll accounting, labor costs |
| Adjustment Journal | Corrections, reversals, write-offs | Error correction, reconciliation |

## **4.4 Accounting Reports**

kogi-bank generates the following standard accounting reports for individual workers and organizational entities, computed from the ledger in real-time:

| Report | Description | Audience |
| :---- | :---- | :---- |
| Profit & Loss (P\&L) | Revenue minus expenses over a defined period | Worker, Org, Cooperative |
| Balance Sheet | Assets, liabilities, and equity at a point in time | Org, Cooperative, Investor due diligence |
| Cash Flow Statement | Operating, investing, and financing cash flows | Worker, Org |
| Income Statement | Detailed income breakdown by client, project, and category | Worker, Org |
| Expense Report | Categorized expense breakdown; deductible flagging | Worker, Org |
| Invoicing Summary | Outstanding, paid, and overdue invoice status | Worker, Org |
| Tax Estimate | Self-employment tax liability estimate with quarterly breakdown | Worker |
| Payroll Summary | Compensation distributed to members/workers per period | Cooperative, Org |
| Campaign Report | Contributions, disbursements, and campaign outcome financials | Org, Campaign owner |
| Equity Register | Cap table, vesting schedules, ownership percentages | Org, Cooperative |
| Portfolio Financial KPIs | Revenue, burn rate, budget utilization, ROI per portfolio project | Worker, Org |

## **4.5 Tax Management**

kogi-bank provides autonomous tax management for independent workers and organizational entities:

* Self-employment tax estimation — automated quarterly SE tax calculation based on net income

* Deduction tracking — AI-auto-categorized expenses flagged as potentially deductible (home office, equipment, professional services)

* Tax reserve automation — configurable % auto-split from every income deposit into Tax Reserve Account

* Quarterly payment reminders — Oba agent notifies when estimated tax payments are due with amount and payment options

* 1099/W-9 management — request, collect, and store contractor tax documents for platform payouts above thresholds

* Multi-entity tax accounts — separate tax reserves for each organizational entity; consolidated reporting for affiliated entities

* International tax awareness — multi-currency income flagging; withholding tax alerts for cross-border transactions

| 5\. TRANSACTION SYSTEM |
| :---- |

## **5.1 Transaction Data Model**

A Transaction is the atomic financial event that drives one or more ledger entries. Every movement of value between accounts — payment, transfer, escrow lock, payout, donation, investment — is represented as a Transaction.

| Field | Description |
| :---- | :---- |
| **tx.id** | Platform-unique UUID |
| **tx.type** | payment | transfer | escrow\_lock | escrow\_release | invoice\_payment | payout | donation | investment | refund | fee | tax\_payment | payroll | campaign\_contribution | campaign\_disbursement | exchange\_settlement | swap | adjustment |
| **tx.status** | initiated | pending | processing | completed | failed | reversed | disputed |
| **tx.amount** | { value, currency, fx\_rate\_at\_execution (for cross-currency) } |
| **tx.from\_wallet\_id** | Source wallet |
| **tx.from\_account\_id** | Source account (resolved from wallet) |
| **tx.to\_wallet\_id** | Destination wallet |
| **tx.to\_account\_id** | Destination account |
| **tx.reference\_type** | invoice | deal | order | campaign | escrow | payroll | gig | contract | manual |
| **tx.reference\_id** | ID of the linked reference entity |
| **tx.description** | Transaction description |
| **tx.initiated\_by** | User or system agent that triggered the transaction |
| **tx.approval\_status** | not\_required | pending\_approval | approved | rejected |
| **tx.approval\_workflow\_id** | Linked ApprovalWorkflow ID (for multi-sig or policy-gated transactions) |
| **tx.ledger\_entries\[\]** | Created debit/credit pair entry IDs |
| **tx.fees\[\]** | Platform or network fees applied to this transaction |
| **tx.external\_id** | External payment processor reference (Stripe charge ID, ACH trace number, etc.) |
| **tx.metadata** | Full ComponentMetadata with audit trail |

## **5.2 Transaction Types**

### **5.2.1 Inbound Transactions**

| Type | Description | Trigger |
| :---- | :---- | :---- |
| Client Payment | Direct payment from a client for services rendered | Invoice marked paid; Stripe webhook |
| Marketplace Payout | Proceeds from a completed marketplace listing sale | Deal completion → escrow release |
| Gig Payout | Payment received for completed gig or contract milestone | Milestone approved in kogi-exchange |
| Campaign Contribution | Backer contributions to an active campaign | Campaign backing action |
| Investment Inflow | Capital received for equity or revenue share offering | Investment round closing |
| Donation Received | Non-equity charitable contribution | ActionKind::Donate in Portfolio System |
| Refund Received | Return of previously paid funds | Refund initiated by counterparty |
| Transfer In | Internal platform transfer from another account/wallet | Manual or auto-split transfer |

### **5.2.2 Outbound Transactions**

| Type | Description | Trigger |
| :---- | :---- | :---- |
| Expense Payment | Payment for tools, subscriptions, services | Manual or auto-pay rule |
| Payroll Distribution | Compensation to team members or cooperative members | Payroll schedule trigger |
| Marketplace Purchase | Payment for a listing, asset, or work engagement | Deal accepted; escrow funded |
| Investment Deployment | Capital deployed into an equity round or campaign | Investment action approved |
| Donation Given | Charitable or mutual-aid contribution | ActionKind::Donate |
| Tax Payment | Estimated tax payment to tax authority | Manual or Oba-assisted |
| Platform Fee | kogi-bank / platform service charges | Transaction-triggered, automated |
| Escrow Funding | Locking funds into an escrow for an active deal | Deal accepted in marketplace |
| Transfer Out | Internal transfer to another account/wallet | Manual or auto-split |

## **5.3 Payment Rails**

kogi-bank supports multiple payment rails for inbound and outbound transactions, managed through the kogi-providers ProviderSystem:

| Rail | Direction | Typical Latency | Use Case |
| :---- | :---- | :---- | :---- |
| Stripe | In \+ Out | Instant (card) / 2 days (payout) | Client card payments, marketplace payouts |
| ACH (via Plaid) | In \+ Out | 1–3 business days | Bank-to-bank transfers, payroll |
| Wire Transfer | In \+ Out | Same / next day | Large international payments |
| PayPal | In \+ Out | Instant (platform) / 1-3 days (bank) | Small payments, contractor payouts |
| Ethereum / ERC-20 | In \+ Out | Minutes (on-chain) | Token payments, crypto payouts, DeFi |
| Coinbase Commerce | In \+ Out | Minutes | Crypto invoice acceptance |
| Internal (kogi-bank) | In \+ Out | Instant | Platform-to-platform transfers |
| SEPA | In \+ Out | 1 business day | European bank transfers |

| 6\. ESCROW SYSTEM |
| :---- |

## **6.1 Concept**

The kogi-bank Escrow System provides secure, conditional custody of funds for Marketplace and Exchange deals. Escrow ensures that buyers' funds are protected until agreed deliverables are confirmed, while sellers are guaranteed payment upon successful delivery — eliminating counterparty risk from platform transactions.

Every escrow in kogi-bank is a first-class financial entity with its own account (Escrow Reserve Account), ledger trail, governance policies, and milestone schedule. Escrows can be simple (single-payment on delivery) or structured (multi-milestone with partial releases).

## **6.2 Escrow Data Model**

| Field | Description |
| :---- | :---- |
| **escrow.id** | Platform-unique UUID |
| **escrow.deal\_id** | Linked Deal ID from kogi-marketplace or kogi-exchange |
| **escrow.type** | simple | milestone | time\_locked | multi\_party | conditional |
| **escrow.status** | unfunded | funded | partial\_release | fully\_released | disputed | refunded | expired |
| **escrow.buyer\_id** | Funding party UUID |
| **escrow.seller\_id** | Receiving party UUID |
| **escrow.total\_amount** | Total escrow value with currency |
| **escrow.funded\_amount** | Amount currently held in escrow |
| **escrow.released\_amount** | Cumulative amount released to seller |
| **escrow.reserve\_account\_id** | Escrow Reserve Account holding the funds |
| **escrow.milestones\[\]** | Array of EscrowMilestone entities |
| **escrow.release\_conditions** | JSONB: conditions that trigger partial or full release |
| **escrow.auto\_release\_after\_days** | If \> 0: auto-releases N days after delivery confirmation if no dispute |
| **escrow.dispute\_window\_days** | Window after delivery in which buyer can raise a dispute |
| **escrow.dispute\_id** | Linked Dispute ID if a dispute is in progress |
| **escrow.metadata** | Full ComponentMetadata with audit trail |

## **6.3 Escrow Milestone Model**

| Field | Description |
| :---- | :---- |
| **milestone.id** | UUID |
| **milestone.escrow\_id** | Parent escrow reference |
| **milestone.name** | Milestone name (e.g. 'Design Phase', 'Alpha Delivery') |
| **milestone.amount** | Amount to release upon this milestone's approval |
| **milestone.due\_date** | Expected delivery date |
| **milestone.status** | pending | delivered | approved | rejected | disputed |
| **milestone.deliverable\_description** | What must be delivered to trigger this milestone |
| **milestone.delivery\_evidence\[\]** | File or link attachments submitted as delivery proof |
| **milestone.approved\_by** | User ID of approver |
| **milestone.approved\_at** | Approval timestamp |
| **milestone.release\_tx\_id** | Transaction ID of the corresponding escrow release payment |

## **6.4 Escrow Lifecycle**

| Stage | Description | Ledger Effect |
| :---- | :---- | :---- |
| Deal Accepted | Buyer and seller agree on terms. Escrow record created. | No ledger entry yet |
| Funded | Buyer transfers funds to Escrow Reserve Account. | Debit: Buyer Checking. Credit: Escrow Reserve. |
| In Progress | Work underway. Funds locked. | No movement. Balance visible to both parties. |
| Milestone Delivered | Seller submits deliverable. Milestone status → 'delivered'. | No movement. Dispute window opens. |
| Milestone Approved | Buyer approves. Partial release triggered. | Debit: Escrow Reserve. Credit: Seller Payroll/Checking. |
| Dispute Raised | Buyer raises dispute within dispute window. Funds frozen. | No movement. Dispute entity created. |
| Dispute Resolved | Resolution reached via platform mediation or arbitration. | Release or refund per resolution. |
| Auto-Release | Buyer silent past dispute window. Auto-release fires. | Debit: Escrow Reserve. Credit: Seller. |
| Full Release | All milestones approved. Escrow closed. | Final debit/credit. Escrow account balance → 0\. |
| Refunded | Deal cancelled before delivery. Buyer refunded. | Debit: Escrow Reserve. Credit: Buyer Checking. |

## **6.5 Dispute Resolution**

When a buyer raises a dispute, kogi-bank freezes the affected escrow funds and creates a Dispute entity. The resolution process:

1. Dispute Raised — buyer submits dispute with reason, evidence, and requested remedy

2. Seller Notified — seller receives dispute notification and has a response window

3. Evidence Exchange — both parties submit evidence via Deal Room

4. Oba AI Summary — AI agent summarizes both positions and evidence for review

5. Mediation — platform mediation team reviews and attempts settlement (automated for small amounts)

6. Arbitration — for unresolved disputes above a threshold; third-party arbitration panel

7. Resolution — funds split, fully released, or fully refunded per resolution decision

8. Appeal — either party can appeal within N days; final decision is binding

| 7\. INVESTMENT & FINANCIAL INSTRUMENTS |
| :---- |

## **7.1 Concept**

kogi-bank manages the full lifecycle of investment activity on the Kogi platform — from individual workers making equity investments in community campaigns, to cooperative members managing shared funds, to organizations issuing equity rounds. All investment activity is recorded as Portfolio Items, giving investors full portfolio-level visibility into their financial positions alongside their work and asset portfolios.

## **7.2 Investment Account & Position Model**

| Field | Description |
| :---- | :---- |
| **investment.id** | UUID |
| **investment.investor\_id** | Investing entity UUID (user, org, collective) |
| **investment.subject\_id** | The Portfolio Component, Campaign, or Financial Instrument being invested in |
| **investment.subject\_type** | campaign | equity\_round | revenue\_share | portfolio\_asset | external\_instrument |
| **investment.instrument\_type** | equity | convertible\_note | revenue\_share | community\_bond | donation | grant | token | portfolio\_index |
| **investment.amount** | Capital committed: { value, currency } |
| **investment.units** | Equity units, tokens, or shares received |
| **investment.unit\_price** | Price per unit at investment time |
| **investment.status** | committed | funded | active | vesting | matured | exited | written\_off |
| **investment.terms** | JSONB: vesting schedule, cliff, liquidation preference, pro-rata rights, interest rate, maturity date |
| **investment.portfolio\_component\_id** | The investment is itself a Portfolio Item (type=Asset) |
| **investment.return\_to\_date** | Current computed return on investment |
| **investment.distributions\[\]** | Dividend or revenue share payments received |
| **investment.metadata** | Full ComponentMetadata |

## **7.3 Investment Types**

| Instrument | Description | Settlement | Portfolio Integration |
| :---- | :---- | :---- | :---- |
| Equity Stake | Fractional ownership in a project, program, or organization | Cap table update in kogi-organizations; equity units in Equity Account | Investment item linked to Organization Portfolio Component |
| Revenue Share Note | Right to a % of future revenue from a portfolio component | Automated periodic transfer from Income Journal when revenue recognized | Revenue trigger linked to Asset Portfolio Component income |
| Convertible Note | Debt convertible to equity upon trigger event (e.g., next funding round) | Conversion event updates Equity Account; debt entry reversed | Liability Journal until conversion; Equity Journal after |
| Community Bond | Fixed-term debt issued by a cooperative or collective | Periodic interest payments \+ principal return | Liability on issuer's Balance Sheet; Asset on holder's |
| Donation / Grant | Non-repayable capital contribution | Single inbound transaction; no obligation created | Income Journal for recipient; Expense Journal for donor |
| Token / Digital Asset | Platform-native or external blockchain token holding | On-chain transfers via crypto bridge | Crypto Account balance; market value tracked via price feed |
| Portfolio Index Unit | Fractional exposure to a curated group of portfolio assets | Composite settlement across constituent assets | Index position as Asset in Investment Account |
| Pre-sale / Pre-purchase | Early commitment to purchase an asset before launch | Escrow until delivery; converts to purchase on delivery | Escrow Journal until resolved |

## **7.4 Cap Table Management**

For organizational entities (organizations, cooperatives, DAOs), kogi-bank maintains an integrated cap table within the organization's Equity Account and equity-linked Portfolio Components:

* Shareholder register — each equity holder's name, units held, % ownership, and class

* Vesting schedule tracking — cliff dates, vesting milestones, forfeiture events

* Dilution modeling — preview impact of new issuances on existing holders' %

* Liquidation waterfall — priority ordering for distributions and exit proceeds

* Transfer restrictions — governance-enforced restrictions on equity transfers

* Distribution history — dividend, revenue share, and buyback events per shareholder

## **7.5 Portable Benefits**

kogi-bank manages portable benefits for independent workers — financial benefits that travel with the worker regardless of which clients or projects they work on:

| Benefit | Account Type | Management |
| :---- | :---- | :---- |
| Health Insurance Reserve | Savings Account (tagged: health) | Monthly auto-allocation from income; provider payment automation |
| Retirement Savings | Retirement Account | Configurable contribution rate; linked to external IRA/401k providers |
| Emergency Fund | Savings Account (tagged: emergency) | Goal-based auto-save until target balance reached |
| PTO Reserve | Resource Account (time units) | Hours credited from completed contracts; withdrawal \= paid leave period |
| Equipment / Tools Fund | Operations Account (tagged: equipment) | Depreciation tracking; replacement fund auto-save |
| Professional Development | Operations Account (tagged: education) | Budget for courses, conferences, certifications |
| Equity Holdings | Equity Account | Cap table positions from work contracts, cooperative membership, investments |

| 8\. INVOICING, BILLING & ORDERS |
| :---- |

## **8.1 Invoice Data Model**

| Field | Description |
| :---- | :---- |
| **invoice.id** | UUID |
| **invoice.type** | client\_invoice | platform\_bill | subscription | milestone\_invoice | retainer | tax\_invoice | collective\_invoice |
| **invoice.issuer\_id** | Entity issuing the invoice (user, org) |
| **invoice.recipient\_id** | Entity receiving the invoice |
| **invoice.status** | draft | sent | viewed | overdue | paid | partially\_paid | disputed | cancelled | void |
| **invoice.line\_items\[\]** | Array of { description, quantity, unit\_price, subtotal, tax\_rate, tax\_amount } |
| **invoice.subtotal** | Pre-tax total |
| **invoice.tax\_amount** | Computed tax (if applicable) |
| **invoice.total** | Final total due |
| **invoice.currency** | ISO 4217 currency code |
| **invoice.due\_date** | Payment due date |
| **invoice.payment\_terms** | net\_30 | net\_15 | due\_on\_receipt | custom |
| **invoice.payment\_link** | Stripe-hosted or platform-native payment URL |
| **invoice.portfolio\_component\_id** | Linked project, gig, or deal generating this invoice |
| **invoice.contract\_id** | Linked contract reference if applicable |
| **invoice.reminders\[\]** | Scheduled reminder events and their sent timestamps |
| **invoice.payments\[\]** | Partial or full payment transaction IDs |
| **invoice.metadata** | Full ComponentMetadata |

## **8.2 Invoice Lifecycle**

| State | Description | Trigger |
| :---- | :---- | :---- |
| Draft | Private; not yet sent | Manual creation or AI-drafted on deal completion |
| Sent | Delivered to recipient; payment link active | Send action or auto-send on milestone approval |
| Viewed | Recipient opened the invoice | Email/link open tracking |
| Overdue | Past due date with unpaid balance | Automated status transition at due date \+ grace period |
| Partially Paid | Partial payment received | Inbound payment \< total |
| Paid | Full payment received; ledger updated | Final inbound payment transaction |
| Disputed | Recipient raised a payment dispute | Dispute action by recipient |
| Cancelled / Void | Invoice nullified before payment | Manual action; reversal entry posted |

## **8.3 Subscription & Recurring Billing**

For ongoing service relationships, kogi-bank supports automated recurring billing:

* Subscription plans — fixed recurring intervals (weekly, monthly, quarterly, annual)

* Retainer billing — regular invoices against an ongoing service retainer agreement

* Usage-based billing — invoices computed from tracked usage metrics (hours, units, compute)

* Auto-invoicing — invoice generated and sent automatically on schedule without manual action

* Failed payment retry — configurable retry logic for failed subscription payments (3/7/14 day retry schedule)

* Proration — mid-cycle plan changes automatically prorated on next invoice

## **8.4 Funding, Bids, Offers, Deals, Requests, Proposals & Contracts**

kogi-bank is the financial settlement layer for all transactional entities that originate in kogi-marketplace and kogi-exchange. Each of these entity types has a corresponding financial lifecycle:

| Entity | Financial Lifecycle | kogi-bank Role | Settlement |
| :---- | :---- | :---- | :---- |
| Bid | Buyer places bid; funds may be reserved as a bid bond | Optional bid bond hold in Escrow Reserve | Release on win; refund on loss |
| Offer | Seller quotes price; no immediate financial commitment | Price commitment recorded in Invoice Draft | Invoice generated on acceptance |
| Deal | Bid/offer matched or proposal accepted | Escrow funded by buyer; released to seller on completion | Multi-milestone escrow lifecycle |
| Request for Proposal | Buyer posts budget ceiling; no immediate commitment | Budget ceiling recorded; escrow provisioned on acceptance | Per accepted proposal terms |
| Proposal | Seller submits scope \+ price; no commitment until accepted | Proposal price drives Invoice Draft and Escrow plan | Invoice \+ Escrow on acceptance |
| Contract | Legally-binding engagement agreement | Milestone payment schedule drives automated invoicing | Per contract milestone schedule |
| Gig | Single deliverable work unit | Fixed-price escrow | Release on delivery approval |
| Task | Sub-unit of a contract or project | Tracked against parent contract budget | No separate escrow; covered by contract |
| Retainer | Ongoing access to worker time | Monthly recurring invoice | Auto-invoice \+ auto-payment from buyer wallet |

| 9\. CAMPAIGNS & GROUP ECONOMICS |
| :---- |

## **9.1 Concept**

Campaigns are the group economic engine of kogi-bank — enabling independent workers, organizations, collectives, and cooperatives to collectively raise, pool, and deploy capital and resources toward shared goals. A Campaign is a time-bounded fundraising or resource-gathering effort with a defined goal, a set of backers/contributors, and a structured disbursement mechanism.

Every Campaign has a dedicated Campaign Account in kogi-bank that holds contributed funds during the campaign period. Upon success, funds are disbursed per the campaign's terms. Upon failure, funds are automatically returned to contributors — atomically, without manual intervention.

## **9.2 Campaign Types**

| Type | Description | Financial Instrument | Disbursement |
| :---- | :---- | :---- | :---- |
| Fundraise | Donation-based funding for a project, mission, or cause | Donation (non-equity, non-repayable) | Lump sum on goal met or deadline reached |
| Equity Crowdfunding | Fractional ownership stake in exchange for capital | Equity units issued to backers | Equity registered in Equity Account; capital to Operating Account |
| Revenue Share Pool | Backers pool capital; share in future revenue from a Portfolio Component | Revenue Share Note | Periodic distribution from Income Journal when revenue recognized |
| Resource Campaign | Collective gathering of non-monetary resources (compute, licenses, materials) | Resource units committed | Resource Account credited per contribution |
| Group Buy | Collective purchase of an asset or license at a negotiated bulk rate | Shared ownership or individual licenses | Payment released to vendor on threshold reached |
| Community Bond | Debt issuance from a cooperative or collective to fund operations | Fixed-rate debt instrument | Principal \+ interest repaid per bond schedule |
| Pre-sale | Early commitment to purchase an asset before launch | Pre-purchase obligation | Escrow released to seller on delivery; refunded if not delivered |
| Mutual Aid Round | Community pool for member emergency support | Donation with soft reciprocity | Disbursed to requesting member per mutual aid policy |

## **9.3 Campaign Data Model**

| Field | Description |
| :---- | :---- |
| **campaign.id** | UUID |
| **campaign.type** | fundraise | equity | revenue\_share | resource | group\_buy | bond | presale | mutual\_aid |
| **campaign.owner\_id** | Campaign creator UUID (user, org, collective) |
| **campaign.portfolio\_component\_id** | Linked Portfolio Program or Project being funded |
| **campaign.campaign\_account\_id** | Dedicated Campaign Account holding contributed funds |
| **campaign.goal** | { amount, currency } or { resource\_units, unit\_type } |
| **campaign.raised** | Current contributed total |
| **campaign.backer\_count** | Number of unique contributors |
| **campaign.backers\[\]** | Array of { backer\_id, amount, committed\_at, status } |
| **campaign.equity\_terms** | JSONB: % equity offered, pre-money valuation, class, rights, vesting (equity campaigns) |
| **campaign.revenue\_share\_terms** | JSONB: % of revenue, duration, payment frequency, portfolio component (revenue share campaigns) |
| **campaign.bond\_terms** | JSONB: interest rate, maturity date, payment schedule (bond campaigns) |
| **campaign.deadline** | Campaign end date/time |
| **campaign.all\_or\_nothing** | Boolean: if true, contributions refunded if goal not met by deadline |
| **campaign.status** | draft | active | paused | funded | failed | distributing | closed |
| **campaign.milestones\[\]** | Use-of-funds milestones with release conditions |
| **campaign.disbursement\_tx\_ids\[\]** | Transaction IDs of disbursements to campaign beneficiaries |
| **campaign.refund\_tx\_ids\[\]** | Transaction IDs of contributor refunds (if campaign fails) |
| **campaign.metadata** | Full ComponentMetadata |

## **9.4 Campaign Financial Lifecycle**

| Stage | Ledger Effect | System Action |
| :---- | :---- | :---- |
| Campaign Created | No ledger entry | Campaign Account provisioned; Portfolio Component linked |
| Contribution Received | Debit: Backer Wallet → Credit: Campaign Account | Backer record added; progress % updated; feed event published |
| Goal Reached | No movement yet | Success event published; milestone schedule activated |
| Milestone Released | Debit: Campaign Account → Credit: Owner Operating Account | Disbursement per milestone; backers notified |
| Equity Issued (Equity type) | Equity Account entry for backer; Cap table updated | Equity units minted and assigned to backer accounts |
| Campaign Failed / Deadline Missed | No movement yet | Failure event triggered; refund workflow initiated |
| Contributor Refunded | Debit: Campaign Account → Credit: Backer Wallet (×N) | Atomic multi-payout; all backers refunded simultaneously |
| Campaign Closed | Campaign Account balance → 0 | Account archived; final report generated |

| 10\. COOPERATIVE, COLLECTIVE & ORGANIZATIONAL BANKING |
| :---- |

## **10.1 Concept**

kogi-bank provides full banking infrastructure for the non-individual entities of the Kogi platform: autonomous organizations (LLCs, Corps, Trusts, Funds), collectives (informal resource-sharing groups), cooperatives (member-owned economic entities), independent teams (squads, guilds, tribes), and federations (networks of linked entities).

Organizational banking in kogi-bank extends all the same account, wallet, ledger, and transaction primitives available to individual workers — but adds the organizational governance layer: multi-signatory accounts, member-split payroll, shared equity management, collective fund governance, and federation-level financial coordination.

## **10.2 Organizational Banking Setup**

When an organizational entity is created in kogi-organizations, kogi-bank automatically provisions a standard set of accounts and wallets:

| Entity Type | Auto-Provisioned Accounts | Auto-Provisioned Wallets | Governance Model |
| :---- | :---- | :---- | :---- |
| Autonomous Org (LLC, Corp, Trust) | Operating, Payroll, Reserve, Tax, Investment | Org Wallet, Payroll Wallet, Campaign Wallet | Single or multi-sig per bylaws; board approval for large transactions |
| Collective | Operating, Mutual Aid Fund, Resource Account | Collective Wallet, Campaign Wallet | Member vote for withdrawals above threshold; flat governance |
| Cooperative | Operating, Payroll (member distribution), Equity Pool, Reserve, Tax | Org Wallet, Payroll Wallet, Campaign Wallet | Democratic 1-member-1-vote governance; patronage distribution schedule |
| Independent Team | Project Account, Operating | Org Wallet | Team lead approval; optional multi-sig |
| Federation | Federation Reserve, Campaign Account | Collective Wallet | Inter-org approval workflow; federate-level governance |

## **10.3 Cooperative & Collective Economics**

### **10.3.1 Patronage Distribution**

Cooperatives operate on a patronage (proportional to contribution) distribution model. kogi-bank computes and distributes patronage dividends from the cooperative's operating surplus:

9. Revenue recognized in cooperative's Income Journal

10. Expenses deducted in Expense Journal; surplus computed

11. Governance vote approves distribution amount and timing

12. Patronage % computed per member based on their contribution (hours, revenue share, capital contribution)

13. Payroll Journal entries created for each member's distribution

14. Transactions executed from Operating Account to each member's Payroll Wallet

### **10.3.2 Resource Sharing Accounts**

Collectives can operate Resource Accounts denominated in non-monetary units — compute hours, license seats, equipment time, or any custom resource unit. Members can contribute to and draw from these pools:

* Resource units contributed are credited to the Resource Account

* Resource units consumed are debited — tracked in Ledger with the same rigor as monetary transactions

* Resource pool governance policies (minimum balance, max single withdrawal, contribution schedules)

* Integration with kogi-exchange Resource Market for external resource acquisition and offering

### **10.3.3 Federation Financial Coordination**

Federations — networks of linked organizations and collectives — can operate a Federation Reserve and federated campaign infrastructure:

* Federation Reserve — shared capital pool funded by member contributions; governed by federation policy

* Inter-federate transfers — member organizations can transfer resources between one another within the federation

* Federation Campaign — cross-organizational fundraising campaign where multiple entities contribute to a shared goal

* Consolidated reporting — federation-level financial overview aggregating all member entities' KPIs

| 11\. AI INTEGRATION |
| :---- |

## **11.1 Finance Agent (kogi-engine)**

The Finance Agent is the AI sub-system dedicated to financial intelligence within kogi-bank. It runs on a continuous monitoring loop, consuming ledger events and portfolio financial signals to provide proactive insights, forecasts, and automated actions:

| Capability | Description | Engine | Output |
| :---- | :---- | :---- | :---- |
| Income Forecasting | Predicts next 30/60/90 day income based on active contracts, seasonal patterns, and historical data | AnalyticsEngine \+ RecommendationEngine | Revenue projection with confidence intervals |
| Expense Analysis & Categorization | Auto-categorizes all transactions using semantic classification; identifies recurring and anomalous expenses | AnalyticsEngine | Categorized expense breakdown; anomaly alerts |
| Tax Estimation | Computes self-employment tax liability in real-time from YTD income and expense data | AnalyticsEngine | Quarterly tax estimate with breakdown |
| Cash Flow Forecasting | Projects account balances forward based on known upcoming transactions and forecasted income | OptimizationEngine | 30-day cash flow projection; overdraft risk warning |
| Budget Utilization | Compares actual spend against budget allocations in Portfolio System; flags overruns | AnalyticsEngine \+ RiskEngine | Budget health score; overrun alerts |
| Savings Goal Tracking | Monitors progress toward defined financial goals; suggests auto-save adjustments | OptimizationEngine | Goal completion ETA; savings rate recommendation |
| Anomaly Detection | Flags unusual transactions — unexpected large charges, duplicate payments, unauthorized activity | RiskEngine | Fraud alert; anomaly report |
| Investment Performance | Computes ROI, IRR, and portfolio-level return on all investment positions | AnalyticsEngine | Investment performance dashboard |
| Campaign Velocity Monitoring | Tracks campaign contribution rate vs. goal; predicts success probability and time-to-goal | AnalyticsEngine | Campaign health score; outreach suggestions |
| Payment Risk Scoring | Scores counterparties in marketplace deals by payment history and risk signals | RiskEngine | Counterparty risk score; escrow recommendation |

## **11.2 Oba AI Agent Workflows for kogi-bank**

### **11.2.1 Invoice Generation & Recovery**

When a deal or gig milestone is completed in kogi-marketplace, Oba automatically drafts the corresponding invoice, attaches project context, and prompts the user for one-click send. For overdue invoices, Oba drafts escalating follow-up messages calibrated to the client relationship history and flags cash flow impact.

### **11.2.2 Morning Financial Briefing**

Each morning, the Oba portfolio briefing includes a financial section: account balances at a glance, invoices due today, upcoming scheduled payments, current cash flow trajectory vs. quarterly OKR, tax reserve status, and any anomalies detected overnight — all as an AI-composed prioritized action list.

### **11.2.3 Tax Reserve Automation**

After each significant income deposit, Oba notifies the user with a tax estimate for the deposit and confirms the auto-split to the Tax Reserve Account. At each quarter-end, Oba drafts the estimated tax payment, shows the exact amount, and offers one-click initiation via the user's linked bank or Stripe.

### **11.2.4 Payroll Distribution Assistance**

For organizational and cooperative entities, Oba computes the payroll distribution schedule based on member contributions, patronage rules, and available operating surplus. It drafts the full payout transaction set for owner or governance review, and executes the batch payout atomically upon approval.

### **11.2.5 Budget Overrun Intervention**

When a portfolio project's actual spend approaches or exceeds its budgeted ResourceAllocation (Budget kind), Oba surfaces the overrun alert with specific line items, computes the projected overrun at current burn rate, and suggests specific interventions: scope reduction, timeline extension, or supplemental budget request.

### **11.2.6 Campaign Intelligence & Outreach**

For active fundraising campaigns, Oba monitors contribution velocity and predicts the probability of reaching the goal by the deadline. When velocity drops below the needed rate, Oba identifies high-probability backers from the social graph and community data, drafts personalized outreach messages, and prepares them for owner approval and dispatch.

| 12\. DATA ARCHITECTURE |
| :---- |

## **12.1 PostgreSQL Schema**

### **12.1.1 Core Tables**

| Table | Description | Key Columns |
| :---- | :---- | :---- |
| accounts | All account records for users and organizations | id, owner\_id, owner\_type, type, denomination, currency, status, policy\_ids JSONB, metadata JSONB |
| wallets | Wallet records linking owners to accounts | id, owner\_id, type, linked\_accounts JSONB, transaction\_rules JSONB, auto\_split JSONB, status |
| ledger\_entries | Immutable double-entry ledger — append only | id, ledger\_id, journal\_id, transaction\_id, account\_id, type (debit/credit), amount, currency, balance\_after, entry\_type, reference\_id, timestamp |
| transactions | Transaction records grouping ledger entry pairs | id, type, status, amount, currency, from\_wallet\_id, to\_wallet\_id, from\_account\_id, to\_account\_id, reference\_type, reference\_id, approval\_status, fees JSONB, metadata JSONB |
| invoices | Invoice records | id, type, issuer\_id, recipient\_id, status, line\_items JSONB, total, currency, due\_date, payment\_terms, contract\_id, metadata JSONB |
| escrow\_accounts | Escrow record per deal | id, deal\_id, type, status, buyer\_id, seller\_id, total\_amount, funded\_amount, released\_amount, milestones JSONB, release\_conditions JSONB, dispute\_id |
| campaigns | Campaign records | id, type, owner\_id, portfolio\_component\_id, campaign\_account\_id, goal JSONB, raised, deadline, status, backers JSONB, equity\_terms JSONB, milestones JSONB |
| investments | Investment position records | id, investor\_id, subject\_id, instrument\_type, amount, units, status, terms JSONB, distributions JSONB, metadata JSONB |
| disputes | Dispute records for escrow and payment disputes | id, type, initiator\_id, respondent\_id, escrow\_id, status, evidence JSONB, resolution JSONB, timeline JSONB |
| tax\_estimates | Computed tax estimate records | id, owner\_id, period, gross\_income, deductions, net\_income, estimated\_tax, breakdown JSONB, computed\_at |
| external\_payment\_methods | Linked external accounts and cards | id, owner\_id, provider, method\_type, external\_id, metadata JSONB, status |

## **12.2 Kafka Event Topics**

| Topic | Events | Consumers | Purpose |
| :---- | :---- | :---- | :---- |
| bank.transactions | transaction.initiated, transaction.completed, transaction.failed, transaction.reversed | AnalyticsEngine, kogi-portfolio, kogi-exchange, Feed Service | Real-time transaction stream for analytics and cross-module updates |
| bank.ledger | entry.posted | AnalyticsEngine (cashFlow signal), Tax pipeline | Append-only ledger stream for downstream analytics |
| bank.invoices | invoice.created, invoice.sent, invoice.paid, invoice.overdue, invoice.disputed | AnalyticsEngine, Oba Agent, Feed Service | Invoice lifecycle events; trigger reminders and briefing cards |
| bank.escrow | escrow.funded, escrow.milestone\_released, escrow.disputed, escrow.refunded, escrow.closed | kogi-marketplace, kogi-exchange, AnalyticsEngine | Escrow lifecycle; trigger deal status updates |
| bank.campaigns | campaign.created, campaign.backed, campaign.funded, campaign.failed, campaign.distributed | kogi-marketplace, kogi-organizations, AnalyticsEngine | Campaign lifecycle; trigger equity issuance, refunds, notifications |
| bank.investments | investment.committed, investment.funded, investment.distribution\_paid, investment.exited | kogi-portfolio, kogi-organizations, AnalyticsEngine | Investment lifecycle; cap table and portfolio updates |

## **12.3 Redis Caching**

* Account balances — hot-path cached per account ID; TTL 60s; invalidated on every new ledger entry

* Wallet aggregated balance display — computed aggregate for UI; TTL 30s

* Active escrow state — deal-in-progress escrow status for Deal Room widget; TTL 10s

* Invoice summary cache — outstanding invoices per user; TTL 120s; invalidated on payment

* Campaign progress — contribution totals and backer counts; TTL 30s

* Tax estimate cache — current-period estimate; TTL 1 hour; invalidated on new income entry

| 13\. UX ARCHITECTURE & USER FLOWS |
| :---- |

## **13.1 kogi-bank UI Structure**

| View | Description |
| :---- | :---- |
| **Bank Home / Dashboard** | Account balances at a glance, recent transactions, upcoming invoices, AI financial summary, quick actions |
| **Accounts** | All accounts with type labels, balances, status; create account, link external, view ledger |
| **Wallets** | Wallet grid with balance display, linked accounts, auto-split rules; add wallet, configure rules |
| **Ledger** | Paginated, filterable ledger view with journal classification; search by date, type, reference, amount |
| **Transactions** | Transaction history with status, type, counterparty; filter by wallet, account, date, type |
| **Invoices** | Invoice management: create, send, track, collect; overdue alerts; AI draft button |
| **Escrow** | Active escrows with milestone tracker, release controls, dispute button; deal context panel |
| **Investments** | Investment portfolio: positions, returns, distributions; campaign tracking, equity register |
| **Campaigns** | Campaign management: create, track, disburse; backer list; contribution analytics |
| **Reports** | P\&L, Balance Sheet, Cash Flow, Tax Estimate, Expense Report — downloadable and shareable |
| **Tax Center** | YTD income, estimated tax liability, deduction tracker, quarterly reminders, 1099 management |
| **Org Banking** | Multi-entity banking: org account overview, payroll, equity register, governance approvals |
| **Settings** | Payment methods, auto-split rules, notification preferences, compliance documents |

## **13.2 Core User Flows**

### **13.2.1 Independent Worker — Setup Financial Infrastructure**

15. Open kogi-bank → Bank Home → Account Setup Wizard

16. Create Personal Checking, Operations, and Tax Reserve accounts

17. Configure auto-split: 60% Checking / 20% Operations / 20% Tax Reserve

18. Link external bank via Plaid and credit card via Stripe

19. Connect Payroll Wallet (receives client payments) to Personal Checking as primary

20. Set quarterly tax reminder — Oba schedules alerts

21. Financial infrastructure is now operational — every income deposit auto-routed

### **13.2.2 Create and Collect an Invoice**

22. Complete work milestone → kogi-gig or kogi-marketplace triggers invoice draft in Oba

23. Oba pre-populates: client name, line items from project scope, amount, due date

24. User reviews draft → adjusts if needed → one-click Send

25. Client receives invoice with payment link (Stripe-hosted or platform payment)

26. On payment: transaction created → ledger entries posted → auto-split fires → notification sent

27. Invoice status → Paid; project financial KPIs updated in Portfolio System

### **13.2.3 Escrow — Hire a Worker**

28. Accept Proposal in kogi-marketplace → Escrow created automatically

29. Fund Escrow: select Marketplace Wallet → Escrow Wallet → Confirm

30. Funds locked: Debit Marketplace Wallet account, Credit Escrow Reserve Account

31. Track milestones in Deal Room → escrow status widget shows locked amount

32. Seller delivers milestone → approve in Deal Room → milestone payment released

33. Final milestone approved → full escrow released → review workflow initiated

### **13.2.4 Cooperative — Monthly Payroll Distribution**

34. Month-end: Oba triggers payroll computation for Cooperative entity

35. Oba computes member distributions based on hours contributed and patronage formula

36. Payroll summary presented to cooperative governance for approval vote

37. Vote passes → Oba prepares batch payout transaction set

38. Owner or multi-sig approvers confirm → batch executed atomically

39. All member Payroll Wallets credited simultaneously → Payroll Journal updated

### **13.2.5 Launch an Equity Campaign**

40. Open kogi-bank → Campaigns → New Campaign → Type: Equity

41. Link to Portfolio Project/Program → set goal, % equity offered, valuation, deadline

42. Oba drafts campaign description from portfolio context → publish to kogi-marketplace

43. Backers contribute → funds held in Campaign Account

44. Goal reached → governance approval → equity units issued to backers' Equity Accounts

45. Capital disbursed from Campaign Account to org Operating Account per milestone

46. Cap table updated in kogi-organizations → investment positions recorded in kogi-bank

| 14\. TECHNICAL ARCHITECTURE |
| :---- |

## **14.1 Service Architecture**

| Service | Description | Technology |
| :---- | :---- | :---- |
| kogi-bank (Go) | Primary REST/gRPC API — account, wallet, transaction, and campaign management | Go, PostgreSQL, Kafka producer |
| kogi-ledger (Go) | Append-only ledger service — all write operations to ledger\_entries table; balance computation | Go, PostgreSQL (SERIALIZABLE isolation for ledger writes) |
| kogi-escrow (Go/Rust) | Escrow lifecycle management; integrates with kogi-marketplace deal events | Go service \+ Rust escrow state machine |
| kogi-invoices (Go) | Invoice CRUD, send, payment tracking, recurring billing | Go, PostgreSQL, Stripe webhooks |
| kogi-campaigns (Go) | Campaign lifecycle, contribution tracking, disbursement, refund orchestration | Go, PostgreSQL, Kafka |
| kogi-tax (Go \+ Scala) | Tax estimation, deduction tracking, 1099 management | Go (API), Scala (data pipeline for YTD computation) |
| kogi-engine (Scala) | Finance Agent analytics, forecasting, anomaly detection, cashFlow signals | Scala 3, gRPC server |
| kogi-payments-bridge (Go) | External payment rail integration: Stripe, Plaid, ACH, crypto bridges | Go, Stripe SDK, Plaid SDK, Web3 providers |

## **14.2 Critical Design: Ledger Consistency**

The ledger is the most critical data store in kogi-bank. Its consistency guarantees are the foundation of all financial trust in the platform. The following design decisions enforce correctness:

* Append-only writes — no UPDATE or DELETE on ledger\_entries; corrections via reversal entries only

* Serializable isolation — all ledger writes execute at SERIALIZABLE isolation level in PostgreSQL; no phantom reads

* Atomic debit/credit pairs — every transaction creates both entries in a single database transaction; partial pairs are impossible

* Idempotency keys — all transaction requests carry an idempotency key; duplicate submissions are safely detected and rejected

* Event sourcing — account balances are always recomputed from ledger history; never stored as a mutable balance field

* Saga pattern for distributed transactions — multi-service transactions (e.g., escrow release spanning kogi-bank \+ kogi-exchange \+ kogi-portfolio) use the Saga pattern with compensating transactions for rollback

* CRDT reconciliation — for multi-device and offline scenarios, CRDT-safe merge strategies align with the Portfolio System's existing CrdtLog

## **14.3 Technology Stack**

| Layer | Technology | Component | Notes |
| :---- | :---- | :---- | :---- |
| Core Services | Go | kogi-bank, kogi-ledger, kogi-escrow, kogi-invoices | REST \+ gRPC; Kafka producers |
| State Machine | Rust | Escrow state machine, BankSystem core module | Embedded in kogi-modules bank module |
| Intelligence | Scala 3 / JVM | kogi-engine Finance Agent | gRPC server; cashFlow signal feed |
| Database | PostgreSQL | Ledger, accounts, transactions, escrow, campaigns | SERIALIZABLE for ledger; standard for others |
| Cache | Redis | Balance cache, active escrow state, invoice summary | TTL-based invalidation on writes |
| Streaming | Apache Kafka | bank.transactions, bank.ledger, bank.escrow, bank.campaigns | All financial events streamed for analytics |
| Payment Rails | Stripe, Plaid, Coinbase | External payment bridge service | Managed by kogi-payments-bridge |
| Crypto | Ethereum / EVM, Solana | On-chain escrow (future), token bridge | Planned Phase 4 feature |
| Web Client | Angular \+ TypeScript | kogi-web-client bank module | WebSocket for real-time balance and escrow updates |
| Mobile | Kotlin / iOS | kogi-mobile-client bank module | Push notifications for transactions and escrow events |

## **14.4 Security & Compliance**

* PCI DSS compliance — all card data handled by Stripe; kogi-bank stores no raw card numbers

* All transactions require explicit user approval — no AI-initiated fund movements without user confirmation

* Multi-factor authentication required for transactions above configurable thresholds

* Encryption at rest (AES-256) for all financial data; TLS 1.3 in transit

* Audit trail via immutable ledger \+ Portfolio System EventLog — every action logged with actor, timestamp, and result

* Regulatory compliance hooks — KYC/AML screening for accounts above transaction thresholds (via third-party provider integration)

* GDPR / CCPA right-to-be-forgotten — personal financial data anonymization on account deletion (ledger entries preserved for regulatory purposes)

* Role-based access control — PermissionTier hierarchy governs all account and wallet operations

| 15\. OPEN ITEMS & FUTURE CONSIDERATIONS |
| :---- |

## **15.1 Near-Term Open Items**

| Item | Description |
| :---- | :---- |
| **Ledger Persistence Optimization** | Balance recomputation from full ledger history will become slow for long-lived accounts. Implement periodic balance snapshots with a verified snapshot \+ incremental replay approach for performance. |
| **Saga Compensation Protocol** | Distributed transactions spanning kogi-bank, kogi-exchange, and kogi-portfolio need a fully-specified Saga compensation map. Define compensating transactions for each step in the escrow, deal, and campaign settlement flows. |
| **Crypto Bridge Specification** | Token-based escrow and investment settlement requires a defined bridge protocol between kogi-bank and EVM / Solana chains. Specify the bridge adapter interface, key management, and on-chain escrow contract architecture. |
| **KYC/AML Integration** | Accounts above defined thresholds need Know Your Customer / Anti-Money Laundering screening. Select and integrate a third-party KYC/AML provider (e.g., Stripe Identity, Persona, Onfido) via kogi-providers. |
| **Tax Engine Jurisdiction Handling** | Tax estimation currently assumes US self-employment tax. Multi-jurisdiction tax logic (VAT, GST, withholding) needs a pluggable jurisdiction rules engine. |
| **Cooperative Governance Voting** | Payroll distribution and large withdrawal approval for cooperatives needs a lightweight on-platform voting mechanism. Define integration between kogi-organizations governance and kogi-bank multi-sig approval. |
| **AI Finance Agent Writeback Protocol** | Finance Agent produces forecasts and recommendations but needs a defined writeback protocol to create Invoice drafts, schedule transactions, and append financial governance notes — without bypassing PermissionTier checks. |
| **Portable Benefits Provider Integration** | Retirement, health, and insurance benefit accounts need integration with third-party benefit providers. Define the ProviderSystem integration pattern for benefit account management. |

## **15.2 Phase 4 Considerations**

* On-chain escrow smart contracts — for crypto/token deals; self-executing escrow without custodial counterparty risk

* Decentralized finance (DeFi) integration — yield on idle campaign and reserve accounts via DeFi protocols

* Autonomous financial agent — Oba acts on pre-approved rules autonomously: auto-pay recurring bills, auto-invest above a balance threshold, auto-rebalance tax reserve

* Federation-level financial clearing — cross-federation settlement and clearing for multi-platform (ShangoOS) economic activity

* Real-world asset (RWA) tokenization — platform infrastructure to tokenize and trade physical assets held by cooperative or org entities on-chain

* Automated payroll compliance — automated payroll tax filing, W-2/1099 generation, and employer of record functions for cooperative and org entities

* AI-driven investment recommendations — personalized investment opportunity matching based on portfolio risk profile, liquidity needs, and financial goals

© 2026 Kogi Platform — kogi-bank System Design Document — Confidential