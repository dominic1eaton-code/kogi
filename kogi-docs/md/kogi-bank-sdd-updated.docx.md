

**KOGI**

Independent Worker Operating System

**kogi-bank — Banking, Finance & Capital System**

Updated: Portable Benefits · Grants & Microfinancing · Group Economics · Equity Crowdfunding

v1.1  ·  Kogi Platform  ·  March 2026

# **New Account Types — Personal**

The following account types have been added to the personal account suite to support portable benefits, group economics, and platform contribution tracking:

| Account Type | Description | Primary Use |
| :---- | :---- | :---- |
| HSA Account | Health Savings Account — portable benefit, pre-tax health expense fund; rolls over annually | Medical, dental, vision expenses |
| Portable Savings Account | Gig platform contribution tracking account; receives % contributions (e.g. 4% of pre-tip earnings) from gig platforms | Managed alongside benefit providers (e.g. Stride LLC) |
| Income Protection Fund | Reserves for income replacement during missed work, disability, or occupational accident | Short-term disability, sick pay, income gap coverage |
| Professional Development Account | Portable education and training fund; receives contributions from platforms or self | Certifications, courses, training programs |
| Emergency Savings Account | Rapid-access emergency buffer, separate from general savings | Sudden expenses, income interruptions |

# **13.3 — Portable Benefits System**

The kogi-bank Portable Benefits System (KBNF-BEN) manages worker-centered benefits that remain with the individual rather than being tied to a single employer or gig platform. Every benefit account is a Portfolio Item carrying full metadata, governance, lifecycle management, and audit trails.

## **Benefit Categories**

| Category | Benefit Types | Account Type |
| :---- | :---- | :---- |
| Health & Wellness | Health insurance, dental, vision, HSA | HSA Account |
| Retirement & Savings | SEP-IRA, 401k portability, Pooled Employer Plans (PEP), emergency savings | Retirement Account, Emergency Savings Account |
| Paid Time Off & Income Security | Paid sick days, vacation time, income replacement | Income Protection Fund |
| Insurance & Protection | Occupational accident, disability, workers' compensation | Income Protection Fund |
| Professional Development | Education accounts, training fund, certification credits | Professional Development Account |
| Platform Contributions | Gig platform % contributions (e.g. 4% of pre-tip earnings); custodied by benefit manager (e.g. Stride LLC) | Portable Savings Account |

## **Platform Contribution Tracking Flow**

| Step | Action |
| :---- | :---- |
| 1 | Platform posts earnings event → kogi-bank PortableContributionEvent |
| 2 | BenefitsEngine computes contribution amount (e.g., 4% of pre-tip earnings) |
| 3 | Ledger entry created: debit Platform Contribution Receivable → credit Portable Savings Account |
| 4 | Contribution posted to benefit provider (e.g., Stride LLC) via ProviderSystem API |
| 5 | Worker notified via Oba: "DoorDash contributed $12.40 to your portable savings this week." |

## **BenefitsEngine Capabilities**

* Eligibility scoring — assesses which benefit types a worker qualifies for based on income, activity, and gig platform participation

* Coverage optimization — recommends benefit product combinations and contribution rates for maximum coverage at minimum cost

* Gap analysis — identifies coverage gaps (e.g. no disability coverage) and recommends remediation

* Contribution forecasting — projects benefit account balances given current income trajectory

* Tax optimization — identifies pre-tax contribution opportunities (HSA, retirement) to reduce self-employment tax burden

# **13.4 — Grants & Microfinancing**

kogi-bank provides a complete grants and microfinancing subsystem for independent workers, collectives, cooperatives, and autonomous organizations.

## **Grant System**

| Component | Description |
| :---- | :---- |
| Grant Registry | Catalog of available grants from government, foundation, platform, and community sources; filtered by eligibility, category, amount, and deadline |
| Grant Application | Structured workflow linked to a Portfolio Program or Project; supporting documents attached as Portfolio Artifacts |
| Grant Fund Disbursement | Awarded funds deposited to a dedicated Grant Account; disbursed against milestones per grant agreement |
| Grant Reporting | Impact and milestone reports auto-generated from Portfolio System data; submitted to grant providers on schedule |
| Community Grant Pools | Organizations and collectives create shared grant pools funded by member contributions; governed by governance proposals |
| GrantEngine Integration | kogi-engine GrantEngine scores grant matches by eligibility, portfolio strength, and historical award data |

## **Microfinancing System**

| Component | Description |
| :---- | :---- |
| Microloan Marketplace | Workers and orgs post loan requests; community lenders fund fractional amounts (crowdlending) |
| Credit Scoring | kogi-engine RiskEngine computes platform credit score from portfolio activity, income, repayment history |
| Loan Terms Negotiation | Lender proposes terms; borrower accepts or counter-proposes via Exchange Deal mechanics |
| Repayment Tracking | Automated repayment schedule; kogi-bank processes payments and distributes pro-rated returns to lenders |
| Community Lending Pools | Collectives establish shared pools; members contribute capital and access loans at preferential rates |
| Collateral Support | Portfolio assets (IP, equity stakes, revenue streams) can be pledged as collateral; locked in kogi-bank escrow |

# **13.5 — Group Economics**

Group Economics encompasses all collective financial structures and instruments on the kogi platform — from cooperative treasury management and shared revenue distribution to group investment pools and collective resource acquisition.

## **Group Economics Instruments**

| Instrument | Description | Governance |
| :---- | :---- | :---- |
| Cooperative Treasury | Multi-sig controlled shared fund for operations, payroll, and investment | Multi-sig \+ governance vote for large disbursements |
| Revenue Sharing Pool | Collective revenue aggregated and distributed by formula (hours, equity, contribution) | Automated computation \+ manual approval |
| Group Investment Pool | Members collectively deploy capital into portfolio assets or equity campaigns | Governance proposal required |
| Mutual Aid Fund | Peer-support pool for member emergencies; funded by contributions | Claims reviewed by stewards |
| Resource Pooling Account | Shared account for collectively purchasing and managing shared resources | Majority approval above threshold |
| Crowdresourcing Pool | Community-sourced contributions of labor, assets, knowledge; attribution tracked | Contribution ledger with provenance |
| Collective Escrow | Multi-party escrow for group deals, shared purchases, or collaborative project funding | M-of-N multi-sig release |

## **Revenue Distribution Engine — Cooperative Payroll Flow**

| Step | Action |
| :---- | :---- |
| 1 | Treasurer or Oba triggers distribution computation at period end |
| 2 | BenefitsEngine computes each member's share: hours contributed (60%) \+ equity share (30%) \+ governance bonus (10%) |
| 3 | Distribution summary posted as Governance Proposal for member approval vote |
| 4 | Approved → batch payout transaction set prepared |
| 5 | Multi-sig approvers confirm → batch executed atomically |
| 6 | All member Payroll Wallets credited simultaneously → Payroll Journal updated |
| 7 | Oba notifies each member: "Q1 distribution of $X has been deposited." |

## **Equity Crowdfunding Infrastructure**

| Component | Description |
| :---- | :---- |
| Campaign Types | Equity (Reg CF, Reg A+), Revenue Share, SAFE Note, Convertible Note, Donation, Reward-based |
| Cap Table Management | Investor equity positions tracked in kogi-bank Equity Accounts; vesting enforced; dilution computed on new rounds |
| Escrow Management | Campaign funds held in Campaign Account/Escrow Reserve; auto-disbursed on success; auto-refunded on failure |
| Revenue Share Engine | For revenue-share instruments: automated periodic computation and distribution of investor returns |
| Regulatory Compliance | Reg CF limits enforced; accredited investor verification hooks via ProviderSystem (Parallel Markets, Carta) |
| Investor Portal | Investor view of equity positions, expected returns, distribution history, portfolio performance |

