

**KOGI**

Independent Worker Operating System

**Marketplace & Exchange**

Updated: Grants · Microfinancing · Crowdresourcing · Shared Portfolio ExchangeBooking & Scheduling · CRM & Lead Management · Logistics Tracking

v1.1  ·  Kogi Platform  ·  March 2026

# **11 — Grants, Microfinancing & Crowdresourcing**

## **11.1 Grants Marketplace**

The Kogi Marketplace extends into a grants discovery and application layer, enabling independent workers, collectives, and cooperatives to find, apply for, and manage grants without leaving the platform.

| Feature | Description |
| :---- | :---- |
| Grant Registry | Searchable catalog of government, foundation, platform, and community grants; filtered by eligibility, category, amount, deadline, entity type |
| Grant Listings | Grant providers list grants as Marketplace items with eligibility criteria, required documents, and review timeline |
| Grant Application Flow | Grant discovery → eligibility check → application → document submission → review → award; all linked to Portfolio Program/Project |
| Grant Fund Disbursement | Awarded funds routed through kogi-bank Campaign Account; milestone-based disbursement |
| Community Grant Pools | Organizations and collectives create shared grant pools; governed by community governance proposals |
| Grant Impact Reporting | Auto-generated impact reports from Portfolio System analytics; submitted on schedule to providers |
| GrantEngine Integration | kogi-engine GrantEngine scores grant matches by eligibility, portfolio strength, and historical award data |

## **11.2 Microfinancing**

| Feature | Description |
| :---- | :---- |
| Microloan Marketplace | Workers and organizations post loan requests; community lenders browse and fund fractional amounts (crowdlending) |
| Credit Scoring | kogi-engine RiskEngine computes platform credit score from portfolio activity, income, and repayment history |
| Loan Terms Negotiation | Lender proposes terms; borrower accepts or counter-proposes via Exchange Deal mechanics |
| Repayment Tracking | Automated repayment schedule; kogi-bank processes payments and distributes pro-rated returns to lenders |
| Community Lending Pools | Collectives establish shared lending pools; members contribute capital and access loans at preferential rates |
| Collateral Support | Portfolio assets (IP, equity stakes, revenue streams) pledged as collateral; locked in kogi-bank escrow |

## **11.3 Crowdresourcing**

| Feature | Description |
| :---- | :---- |
| Crowdresourcing Listings | Portfolio owners open items for community contribution; listed with contribution type, goal, and reward rules |
| Contribution Types | Labor (hours, tasks), Capital (monetary), Assets (designs, code, data), Knowledge (research, strategy, documentation) |
| Contribution Ledger | All contributions recorded with attribution in Portfolio EventLog; weight computed by CollaborationEngine |
| Reward Rules | Configurable: equity allocation, revenue share, direct payment, recognition badge, platform credits |
| Steward Review | Configurable gate: AutoAccept | StewardReview | GovernanceVote |
| Analytics | Contribution velocity, contributor diversity, goal progress, open gaps — surfaced in Portfolio Dashboard |

# **12 — Shared Portfolios & Portfolio Resource Exchange**

## **12.1 Shared Portfolio Listings**

| Listing Type | Description |
| :---- | :---- |
| Shared Portfolio Showcase | A group portfolio published for community visibility; followers and investors can track progress |
| Collaborative Project Listing | A portfolio project open to new contributors; lists skills needed, contribution types, and reward model |
| Portfolio Resource Listing | Specific resources (templates, playbooks, tools, datasets) shared with or sold to the community |
| Federation Portfolio | Cross-organization portfolio open to aligned orgs for joint investment or resource contribution |
| Crowdfunded Portfolio Program | A portfolio program raising capital via equity, revenue share, or donation instruments |

## **12.2 Portfolio Resource Exchange — Tradeable Resource Types**

| Resource Type | Exchange Mechanism | Settlement |
| :---- | :---- | :---- |
| Portfolio Templates & Playbooks | Fixed-price listing or auction; buyer gains fork rights | kogi-bank instant settlement |
| Intellectual Property / Licenses | License agreement deal flow; DD report for IP assets | Escrow-backed settlement |
| Portfolio Equity Stakes | Secondary market offer/bid for equity in projects or orgs | Governance approval \+ cap table update |
| Revenue Streams | Forward-sale of future revenue share; exchange pricing via MatchEngine | kogi-bank revenue routing |
| Tool Integrations | Toolbox and toolchain configurations listed and traded | Fork \+ provider credential transfer |
| Data & Datasets | Research, analytics datasets, model outputs with licensing terms | Access-key settlement via kogi-bank |

# **13 — Booking, CRM & Logistics**

## **13.1 Booking & Scheduling**

| Feature | Description |
| :---- | :---- |
| Bookable Profiles | Worker and org profiles set as "bookable"; clients browse and initiate booking requests |
| Availability Calendar | Public calendar; client-facing booking widget for self-service scheduling |
| Booking Request Flow | Client inquiry → worker quotes → client accepts → deposit collected via kogi-bank → confirmed |
| Multi-Worker Booking | Book a team for multi-role events; conflict detection across all assigned workers |
| Resource Booking | Bundle equipment, venue, or other resources into a booking package |
| Booking Analytics | Conversion rate, utilization, average deal size, peak demand periods |

## **13.2 CRM & Lead Management**

| Feature | Description |
| :---- | :---- |
| Lead Capture | Marketplace profile inquiry forms feed directly into worker's CRM pipeline |
| Client Database | Full record: contact, booking history, spend, communication log, tags |
| Pipeline Stages | Inquiry → Qualified → Quoted → Negotiating → Booked → Retained → Lapsed |
| Automated Follow-ups | Configurable email/message sequences on stage transitions, quote expiry, post-booking |
| CRMEngine Integration | Scores lead probability-to-book; Oba recommends outreach timing and messaging |
| Retention Analytics | Repeat booking rate, lifetime value, churn risk; feeds Marketplace profile ranking |

## **13.3 Contracts & Invoicing**

| Feature | Description |
| :---- | :---- |
| Auto-Generated Contracts | Booking confirmation auto-generates contract from template; pre-populated with booking details |
| E-Signature Workflow | Contract sent for e-signature via ProviderSystem; signed status tracked in Portfolio Item |
| Invoice Automation | Deposit and balance invoices auto-generated from booking; payment reminders via kogi-bank |
| Contract Versioning | All revisions versioned and stored in Portfolio System; audit-ready |

## **13.4 Logistics Tracking**

| Feature | Description |
| :---- | :---- |
| Tour / Event Itinerary | Multi-stop itinerary linked to Marketplace bookings; shareable with clients and crew |
| Equipment Tracking | Rental and owned equipment status per booking; return deadlines with automated reminders |
| Transportation Management | Vehicle assignments, routing, and mileage for multi-location bookings |
| Crew Schedules | Role-based crew assignments per booking; call times and wrap times distributed via logistics room |
| Live Status Updates | In-app logistics room per booking for real-time status broadcast to all stakeholders |
| Expense Tracking | Per-booking expense log reconciled against booking budget in kogi-bank |

