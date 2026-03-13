

**KOGI PLATFORM**

**Marketplace & Exchange**

*System Design Document*

v1.0  ·  Kogi Platform  ·  March 2026

| kogi-marketplace  ·  kogi-exchange *The economic layer of the Kogi Independent Worker Operating System —* *where workers trade skills, assets, labor, and financial instruments.* |
| ----- |

Kogi Team  ·  Independent Worker Operating System  ·  Confidential

# **Table of Contents**

| 1\. OVERVIEW |
| :---- |

## **1.1 Purpose & Vision**

The Kogi Marketplace and Exchange constitute the economic layer of the Kogi Independent Worker Operating System. Together they form the transactional, liquidity, and matching infrastructure that connects independent workers, collectives, cooperatives, and organizations to one another for the purpose of trading labor, skills, assets, resources, financial instruments, and portfolio components.

Where kogi-office organizes the worker's private portfolio, and kogi-community connects workers socially, the Marketplace and Exchange are the surfaces through which portfolio value is realized: skills are hired, assets are sold, deals are struck, capital is raised, and portfolio components are exchanged.

| *Design Principle: The Marketplace is where you offer what you have. The Exchange is where you trade what it's worth. Both are powered by the same Portfolio abstraction — every listing, bid, offer, and deal is ultimately a Portfolio Item with economic intent.* |
| :---- |

## **1.2 Position in the Kogi Platform**

The Marketplace and Exchange sit at the boundary between a worker's private operating world and the broader Kogi network economy. Community feeds discovery into both; the Portfolio System is the source of all tradable entities; the Bank provides payment infrastructure; and the Engine powers matching, pricing, and risk intelligence.

| Layer | System | Relationship |
| :---- | :---- | :---- |
| Upstream | kogi-office / Portfolio System | Source of all listings — Assets, Resources, Artifacts with visibility=Public |
| Upstream | kogi-community | Discovery engine — posts, spaces, and social graph feed deal flow |
| Downstream | kogi-bank | Payment, escrow, invoicing, wallet, and settlement layer |
| Downstream | kogi-engine | Matching, recommendation, risk scoring, and price intelligence |
| Peer | kogi-gig | Work contracts and scheduled engagements originate from Exchange deals |
| Peer | kogi-organizations | Cooperatives, collectives, and DAOs operate campaigns and group buys |
| External | 3rd Party Integrations | Upwork, Fiverr, Behance, Robinhood, Coinbase, SoFi, and others |

## **1.3 Module Codes**

| Module | Code | Service | Language |
| :---- | :---- | :---- | :---- |
| Marketplace | KMKT | kogi-marketplace | Go (service) · Rust (core) |
| Exchange | KEXC | kogi-exchange | Go (service) · Rust (core) |
| Deal Engine | KDEAL | kogi-deal | Go (service) |
| Escrow / Settlement | KESC | kogi-escrow (via kogi-bank) | Go (service) |

| 2\. kogi-marketplace |
| :---- |

## **2.1 Concept**

kogi-marketplace is the platform's primary buying-and-selling surface. It enables independent workers and organizations to list, discover, acquire, and sell skills, labor, assets, resources, artifacts, templates, playbooks, and any other portfolio-derived value. It is simultaneously a labor market, an asset store, a service marketplace, and a crowdfunding platform — unified under one coherent UI and data model.

Every item listed on the Marketplace is directly connected to a Portfolio Component. This means every listing carries the full context of its creator's portfolio: version history, analytics, governance metadata, and AI-computed health and maturity scores — giving buyers unprecedented transparency and sellers differentiated, trust-backed offerings.

## **2.2 Marketplace Categories**

The Marketplace is organized across four primary categories:

| Category | Description | Examples | Portfolio Link |
| :---- | :---- | :---- | :---- |
| Work | Labor, skills, and services offered for hire | Gigs, contracts, jobs, tasks, consulting | Resource (Human) Items |
| Assets & Artifacts | Purchasable or licensable portfolio outputs | Playbooks, templates, guides, tools, designs, software | Asset / Artifact Items |
| Resources | Allocatable resources offered to others | Compute, licenses, equipment, capital | Resource Items |
| Campaigns | Fundraising, crowdfunding, and group economics | Equity rounds, resource pools, community funds | Program / Project Items |

## **2.3 Listings**

### **2.3.1 Listing Data Model**

Every Marketplace listing is a Portfolio Item of type Asset, Resource, or Artifact with visibility=Public, extended with the following marketplace-specific fields:

| Field | Description |
| :---- | :---- |
| **listing.id** | Platform-unique UUID |
| **listing.type** | work | asset | artifact | resource | campaign | template | service | bundle |
| **listing.portfolio\_component\_id** | The source Portfolio Component ID (Asset / Resource / Artifact) |
| **listing.owner\_id** | Listing creator's user ID |
| **listing.title** | Display title of the listing |
| **listing.description** | Rich-text description (markdown-compatible) |
| **listing.price** | Price struct: { amount, currency, pricing\_model } |
| **listing.pricing\_model** | fixed | hourly | daily | subscription | negotiable | free | auction | revenue\_share |
| **listing.status** | draft | active | paused | sold | expired | archived |
| **listing.visibility** | public | community | invite\_only |
| **listing.tags\[\]** | Topic and skill tags for discovery |
| **listing.skills\[\]** | Required or offered skills (for work listings) |
| **listing.media\[\]** | Images, videos, portfolio previews |
| **listing.reviews\[\]** | Linked Review entity IDs |
| **listing.rating** | Aggregate rating (0–5, computed) |
| **listing.orders\[\]** | Linked Order entity IDs |
| **listing.proposals\[\]** | Linked Proposal entity IDs |
| **listing.analytics** | Impressions, CTR, conversion rate, views, engagement |
| **listing.ai\_score** | AI-computed quality/match score from kogi-engine |
| **listing.metadata** | Full ComponentMetadata: id, owners, tags, policy\_ids, timestamps, version |

### **2.3.2 Listing Lifecycle**

Listings follow a governed lifecycle managed by the Portfolio System's ApprovalWorkflow and status machine:

| State | Description | Transitions |
| :---- | :---- | :---- |
| Draft | Private, under construction. Not visible to others. | → Active (publish), → Archived |
| Active | Publicly discoverable and available for purchase/hire. | → Paused, → Sold, → Expired, → Archived |
| Paused | Temporarily hidden by owner. | → Active, → Archived |
| Sold | Work or asset fully contracted/purchased. Fulfilled. | → Active (re-list), → Archived |
| Expired | Past end date or automated expiry. | → Active (renew), → Archived |
| Archived | Deep storage. Recoverable. | → Active (restore) |

## **2.4 Work Marketplace (Labor Market)**

The Work sub-market is kogi-marketplace's labor exchange — connecting workers who offer skills, services, and time with those who need them. It covers the full spectrum from short-term gigs to long-term contracts, one-off tasks to ongoing employment relationships.

### **2.4.1 Work Listing Types**

| Type | Description | Duration | Engagement Model |
| :---- | :---- | :---- | :---- |
| Gig | Single, defined deliverable task | Hours – Days | Fixed price or hourly |
| Task | Discrete sub-work unit within a larger project | Hours | Fixed price |
| Contract | Ongoing service engagement under agreed terms | Weeks – Months | Retainer / milestone |
| Job | Longer-term role or embedded position | Months+ | Salary / daily rate |
| Consulting | Advice, strategy, or expert review | Variable | Hourly / project |
| Collaboration | Joint creative or technical project partnership | Variable | Revenue share / co-ownership |

### **2.4.2 Worker Profile & Skill Graph**

Each user's work offering is backed by their Portfolio Resource Item (type=Human), which contains:

* skills\[\] — tagged skill taxonomy (design, engineering, writing, legal, finance, etc.)

* availability — schedule windows pulled from kogi-gig ScheduleBook

* rate — hourly/daily/project rates

* portfolio\_links\[\] — linked public Portfolio Components as proof of work

* reviews\[\] — aggregated reviews from completed engagements

* certifications\[\] — credential badges and verified qualifications

* ai\_reputation\_score — kogi-engine computed trust score based on completion rate, review sentiment, and portfolio health

## **2.5 Asset & Artifact Marketplace**

Workers can publish and sell the outputs of their portfolio — documents, playbooks, templates, designs, software, datasets, research, and any other artifact or asset — as purchasable or licensable Marketplace listings. This is how knowledge, creative work, and intellectual products become economic assets on the platform.

### **2.5.1 Asset/Artifact Listing Types**

| Type | Examples | Pricing Models |
| :---- | :---- | :---- |
| Playbook | Investment playbooks, process guides, strategy docs | Fixed, Subscription, Free |
| Template | Project templates, proposal templates, contract starters | Fixed, Free |
| Design / Blueprint | UI kits, architecture diagrams, brand kits | Fixed, License |
| Software / Tool | Scripts, plugins, automation workflows, SDK extensions | Fixed, Subscription, License |
| Dataset / Research | Market research, curated data sets, analytics reports | Fixed, Subscription |
| Course / Guide | Instructional content, how-to guides, tutorials | Fixed, Subscription |
| Contract Template | Legal templates, NDAs, service agreements | Fixed, Free |
| Bundle | Curated collection of multiple assets | Fixed, Subscription |

| *Example: A user creates a Real Estate Investment Playbook as a Book item in their Portfolio. They publish it to the Marketplace at a fixed price. The kogi-engine RecommendationEngine surfaces it to users who have created Real Estate project types. Subscribers receive automatic updates when a new version is published. This is the end-to-end flow from portfolio to market to subscription.* |
| :---- |

## **2.6 Proposals & Deals**

### **2.6.1 The Proposal Flow**

The primary transactional mechanism in the Marketplace is the Proposal → Deal flow. Rather than simple one-click purchases, most Marketplace engagements involve a structured negotiation that preserves context, terms, and documentation through the full lifecycle.

| Stage | Description |
| :---- | :---- |
| **Inquiry** | Buyer sends initial message or inquiry to a listing. Creates a Deal Room. |
| **Proposal** | Seller or buyer submits a formal Proposal: scope, deliverables, timeline, price, terms. |
| **Negotiation** | Counter-proposals exchanged in the Deal Room. AI assistant can draft and compare terms. |
| **Accepted** | Both parties approve. Proposal transitions to a Deal. |
| **Escrow Funded** | Buyer funds the escrow via kogi-bank. Work begins. |
| **Milestone Check** | Milestone deliverables reviewed. Partial escrow releases triggered. |
| **Completion** | Final deliverable accepted. Full escrow released. |
| **Review** | Both parties submit ratings and reviews. Portfolio analytics updated. |

### **2.6.2 Proposal Data Model**

| Field | Description |
| :---- | :---- |
| **proposal.id** | UUID |
| **proposal.listing\_id** | Source listing reference |
| **proposal.buyer\_id / seller\_id** | Party references |
| **proposal.status** | draft | sent | countered | accepted | rejected | expired | cancelled |
| **proposal.scope** | Rich text: deliverables, exclusions, assumptions |
| **proposal.timeline** | Start date, end date, milestone schedule |
| **proposal.price** | Proposed price with pricing\_model |
| **proposal.milestones\[\]** | Milestone items: name, due\_date, amount, deliverable\_description |
| **proposal.terms** | Payment terms, IP assignment, revision policy, cancellation policy |
| **proposal.attachments\[\]** | Supporting files, portfolio component links |
| **proposal.deal\_room\_id** | Linked Deal Room ID |
| **proposal.metadata** | Full ComponentMetadata |

## **2.7 Campaigns, Crowdfunding & Group Economics**

kogi-marketplace supports collective economic activity through Campaigns — a mechanism for resource gathering, crowdfunding, cooperative purchasing, and equity participation. Campaigns are closely integrated with kogi-organizations and kogi-bank.

### **2.7.1 Campaign Types**

| Campaign Type | Description | Primary Actor |
| :---- | :---- | :---- |
| Resource Campaign | Collective gathering of tools, compute, or materials | Collective / Individual |
| Fundraise | Donation-based funding for a project or mission | Individual / Collective |
| Equity Crowdfunding | Fractional ownership stake in a portfolio project or org | Organization / Cooperative |
| Group Buy | Collective purchasing of an asset or license at bulk rate | Community / Space |
| Revenue Share Pool | Pool of backers sharing in a project's future revenue | Organization |
| Community Fund | Ongoing mutual-aid or community support pool | Cooperative / Federation |
| Pre-sale | Early access or pre-purchase of an asset before launch | Individual / Org |

### **2.7.2 Campaign Data Model**

| Field | Description |
| :---- | :---- |
| **campaign.id** | UUID |
| **campaign.type** | resource | fundraise | equity | group\_buy | revenue\_share | community\_fund | presale |
| **campaign.portfolio\_component\_id** | Linked Portfolio Program or Project being funded |
| **campaign.goal** | Funding goal: { amount, currency, resource\_units } |
| **campaign.raised** | Current amount raised |
| **campaign.backers\[\]** | User IDs and contribution amounts |
| **campaign.equity\_terms** | For equity campaigns: % offered, valuation, rights, vesting schedule |
| **campaign.deadline** | Campaign end date |
| **campaign.status** | draft | active | funded | failed | distributing | closed |
| **campaign.milestones\[\]** | Use-of-funds milestones with release conditions |
| **campaign.org\_id** | Optional linked Organization entity |

## **2.8 Actions & Interactions**

The full set of user-facing actions available on Marketplace entities:

| Action | Actor | Effect | Portfolio Update |
| :---- | :---- | :---- | :---- |
| List | Seller | Creates a Listing from a Portfolio Component | Sets visibility=Public |
| Browse / Search | Buyer | Discovery via SearchEngine \+ RecommendationEngine | No change |
| Save / Bookmark | Buyer | Saves listing to collection | Bookmark count \+1 |
| Inquire | Buyer | Opens a Deal Room and sends initial inquiry | Creates Deal Room |
| Propose | Buyer / Seller | Submits formal Proposal | Creates Proposal entity |
| Accept | Counterparty | Approves Proposal → Deal | Status → Active Deal |
| Purchase | Buyer | Direct buy of fixed-price asset | Order created |
| Fund Escrow | Buyer | Deposits payment into escrow via kogi-bank | Escrow funded |
| Deliver | Seller | Marks deliverable as submitted | Milestone update |
| Approve Delivery | Buyer | Accepts deliverable, triggers escrow release | Escrow release |
| Rate & Review | Both | Submits rating and review after completion | Review analytics updated |
| Invest | Investor | Commits to equity campaign contribution | ActionKind::Invest |
| Donate | Donor | Contributes to a fundraise campaign | ActionKind::Donate |
| Subscribe | User | Subscribes to a seller's ongoing content or updates | Subscriber count \+1 |
| Report | User | Flags listing for review | Moderation queue |
| Share | User | Shares listing to Community feed or externally | Share count \+1 |

## **2.9 Third-Party Integrations**

Marketplace tiles can link to and import from external platforms, managed by the kogi-providers ProviderSystem:

| Category | Platforms | Integration Type |
| :---- | :---- | :---- |
| Freelance Markets | Upwork, Fiverr, Toptal, Freelancer | Profile import, listing sync, order bridging |
| Portfolio / Creative | Behance, Dribbble, GitHub, GitLab | Portfolio import, project showcase sync |
| Job Markets | LinkedIn, Indeed, AngelList | Job listing sync, profile export |
| Collaboration | Slack, Notion, Google Workspace, Microsoft Teams | Project room linking, file sync |
| Commerce | Gumroad, Shopify, Stripe | Asset sales routing, payment bridging |

| 3\. kogi-exchange |
| :---- |

## **3.1 Concept**

kogi-exchange is the platform's structured trading and value-exchange layer. Where the Marketplace facilitates buying and selling, the Exchange facilitates bidding, offering, trading, and structured deal-making across a broader set of value types: financial instruments, portfolio commodities, equity positions, labor contracts, and resource allocations.

The Exchange supports order-book style mechanics (bids and offers), negotiated deals, and peer-to-peer swaps — all within a governed, auditable framework connected to the Portfolio System. It is designed to support both the day-to-day operational needs of independent workers (e.g., trading a contract slot) and more sophisticated financial activity (e.g., equity crowdfunding rounds, portfolio asset sales, resource swaps).

| *Design Principle: The Exchange is not just a financial market — it is a multi-dimensional portfolio economy. Labor, knowledge, capital, resources, and ownership stakes are all first-class tradable entities in the Kogi Exchange.* |
| :---- |

## **3.2 Exchange Markets**

kogi-exchange is organized across four distinct but integrated markets:

| Market | Code | What is Traded | Mechanics |
| :---- | :---- | :---- | :---- |
| Labor Market | KLM | Skills, gigs, contracts, work capacity, tasks | Bids & offers, proposals, matching |
| Portfolio Commodities Market | KPCM | Portfolio Items: assets, artifacts, projects, programs | Listings, offers, deals, swaps |
| Financial Instruments Market | KFIM | Equity, revenue share, convertible notes, funds, tokens | Order book, auctions, rounds |
| Resource Market | KRM | Compute, licenses, physical assets, capital, IP rights | Spot trades, futures, leases |

## **3.3 Order Book & Trading Mechanics**

### **3.3.1 Bids, Offers & Orders**

The Exchange implements a structured order system supporting multiple execution models:

| Order Type | Description |
| :---- | :---- |
| **Bid** | A buyer's offer to purchase at a specified price or terms. Remains open until matched, expired, or cancelled. |
| **Ask / Offer** | A seller's offer to sell at a specified price or terms. |
| **Market Order** | Immediate execution at the best available price. |
| **Limit Order** | Execute only at a specific price or better. |
| **Proposal** | Structured negotiated offer — full terms, scope, timeline, price. |
| **RFP (Request for Proposal)** | Buyer posts requirements; sellers submit competing proposals. |
| **Auction** | Time-limited competitive bidding; highest bid wins. |
| **Reverse Auction** | Buyer posts a budget ceiling; sellers compete on price and quality. |
| **Swap** | Direct peer-to-peer exchange of portfolio items, resources, or labor — no currency required. |
| **OTC Deal** | Over-the-counter bilateral negotiated deal, off order book. |

### **3.3.2 Order Data Model**

| Field | Description |
| :---- | :---- |
| **order.id** | UUID |
| **order.market** | labor | commodities | financial | resource |
| **order.type** | bid | ask | market | limit | proposal | rfp | auction | swap | otc |
| **order.subject** | The Portfolio Component, financial instrument, or resource being traded |
| **order.price** | { amount, currency, pricing\_model, min\_price (for auctions) } |
| **order.quantity** | Units, hours, shares, licenses, etc. |
| **order.terms** | Delivery terms, conditions, expiry |
| **order.status** | open | matched | pending\_escrow | in\_progress | completed | cancelled | expired |
| **order.counterparty\_id** | Matched counterparty once paired |
| **order.deal\_id** | Linked Deal entity upon match |
| **order.deal\_room\_id** | Linked Deal Room for negotiation |
| **order.metadata** | Full ComponentMetadata with audit trail |

## **3.4 Labor Market**

The Labor Market is the Exchange's primary operational market for independent workers. It supports the full lifecycle of finding, hiring, contracting, executing, and settling work engagements.

### **3.4.1 Labor Market Flows**

| Flow | Initiator | Description |
| :---- | :---- | :---- |
| Post & Apply | Buyer posts requirement; Workers apply | Standard job/gig posting with application flow |
| Search & Propose | Worker searches listings; Proposes terms | Outbound worker-driven deal initiation |
| AI Match & Introduce | kogi-engine initiates | MatchEngine surfaces worker-buyer pairs proactively |
| RFP | Buyer posts structured requirements | Multiple workers submit competing proposals |
| Direct Hire | Buyer invites specific worker | Targeted engagement without open listing |
| Collective Staffing | Organization posts team requirement | Fills multiple roles via a single coordinated campaign |

### **3.4.2 Labor Contract Types**

| Contract Type | Description | Payment Model | Portfolio Integration |
| :---- | :---- | :---- | :---- |
| Gig Contract | Single fixed deliverable | Fixed \+ Escrow | Creates Project item |
| Retainer | Ongoing access to worker capacity | Recurring billing via kogi-bank | Creates Resource allocation |
| Milestone Contract | Multi-phase project with staged payments | Escrow per milestone | Links to sprint milestones |
| Revenue Share Contract | Worker paid as % of project revenue | Ongoing via kogi-bank trigger | Links to Asset revenue tracking |
| Equity Contract | Worker compensated with ownership stake | Token / share issuance | Links to Organization cap table |
| Barter / Swap | Skill or service traded for another skill or asset | No currency — swap settlement | Dual Portfolio updates |

## **3.5 Portfolio Commodities Market**

The Portfolio Commodities Market enables direct trading of Portfolio Items — projects, programs, assets, artifacts, and other components. This is where portfolio value becomes directly tradable: a completed project can be sold, a program portfolio can be acquired, an artifact collection can be licensed.

### **3.5.1 Tradable Portfolio Item Types**

| Item Type | Trade Mechanism | Valuation Model |
| :---- | :---- | :---- |
| Asset | Direct sale, license, auction | AssetValue computational model from kogi-engine |
| Artifact | License, sale, subscription | ArtifactMaturity score \+ market comps |
| Project (Completed) | Acquisition, IP transfer | ProjectMetrics model \+ negotiated valuation |
| Program | Partial stake, full acquisition | ProgramAlignment \+ SubPortfolioRollup models |
| Resource (Machine / License) | Rental, lease, sale | ResourceUtilisation model |
| Template / Playbook | Fixed sale, subscription | Engagement analytics \+ follower count |
| Tool / Integration | License, subscription, sale | Usage analytics \+ TMS toolbox data |

## **3.6 Financial Instruments Market**

The Financial Instruments Market is kogi-exchange's most sophisticated sub-market. It supports structured financial transactions including equity sales, convertible instruments, revenue participation, and token-based arrangements — all anchored to real portfolio entities.

### **3.6.1 Supported Instruments**

| Instrument | Description | Settlement |
| :---- | :---- | :---- |
| Equity Stake | Fractional ownership in a project, program, or organization | Cap table update via kogi-organizations |
| Revenue Share Note | Right to a % of future revenue from a portfolio component | Automated trigger via kogi-bank revenue tracking |
| Convertible Note | Debt that converts to equity upon a trigger event | Conversion event tracked in PortfolioSystem |
| Community Bond | Debt instrument issued by a cooperative or collective | Fixed-term repayment via kogi-bank |
| Donation / Grant | Non-repayable capital contribution to a mission | Donor records via ActionKind::Donate |
| Token / Digital Asset | Platform-native or external blockchain asset | Bridge to external chains (Ethereum, Coinbase) |
| Portfolio Index Unit | Fractional exposure to a curated group of portfolio assets | Composite settlement |

## **3.7 Due Diligence System**

For high-value deals — particularly in the Financial Instruments Market and large Portfolio Commodity trades — kogi-exchange provides a structured Due Diligence (DD) framework that gives buyers access to governed, auditable portfolio intelligence before committing.

| DD Component | Description |
| :---- | :---- |
| **Portfolio Health Report** | kogi-engine PortfolioHealthScore: overall score, signal breakdown, risk flags |
| **Project Metrics** | Velocity, completion rates, sprint data, milestone history |
| **Financial Summary** | Revenue, expenses, budget utilization, cash flow signals from kogi-bank |
| **Audit Trail** | Full immutable EventLog from the Portfolio System for the subject component |
| **Risk Assessment** | kogi-engine RiskEngine: risk score, risk optimization plan, red flags |
| **Asset Valuation** | AI-computed AssetValue model with comparable transactions |
| **Artifact Maturity** | ArtifactMaturity score: completeness, test coverage, documentation, update frequency |
| **Governance Snapshot** | Policy compliance, approval workflows, charter commitments |
| **Social Proof** | Community engagement, follower/subscriber count, review summary, spread analytics |

| *DD reports are generated on-demand by the kogi-engine and presented in the Deal Room. They are version-snapshotted at the time of deal acceptance to create an immutable record of the basis on which the deal was agreed.* |
| :---- |

## **3.8 Third-Party Integrations**

| Category | Platforms | Integration Type |
| :---- | :---- | :---- |
| Investment / Trading | Robinhood, SoFi, Interactive Brokers | Portfolio linking, position sync |
| Crypto / Web3 | Coinbase, Ethereum, Polygon, Solana | Token bridge, wallet connect, NFT integration |
| Banking | Stripe, Plaid, Chase, Wells Fargo | Account linking, payment rails via kogi-bank |
| Equity Platforms | AngelList, Republic, Wefunder | Round sync, cap table bridge |
| Commodity Markets | CME, commodity APIs | Real-world asset price feeds |

| 4\. SHARED INFRASTRUCTURE |
| :---- |

## **4.1 Deal Room**

The Deal Room (KRMS type=deal) is the negotiation and coordination space that every Marketplace and Exchange transaction moves through. It is a purpose-built Room from the kogi-community Rooms system, pre-loaded with deal context and equipped with AI assistance.

### **4.1.1 Deal Room Components**

* Conversation thread — full message history between buyer and seller

* Proposal panel — active Proposal with version history and side-by-side comparison

* Document vault — attached files, contracts, DD reports, portfolio previews

* Milestone tracker — visual milestone board with status and escrow lock amounts

* Oba AI panel — AI assistant for drafting terms, comparing proposals, flagging risks

* Activity log — immutable deal event history (viewed, proposed, accepted, paid, delivered)

* Escrow status widget — live view of escrow balance and release conditions from kogi-bank

* Action bar — Propose, Accept, Counter, Fund Escrow, Deliver, Approve, Close

## **4.2 Escrow & Payment**

Payment for Marketplace and Exchange transactions flows through kogi-bank's escrow subsystem. This ensures that funds are protected, conditions are enforced, and settlement is automated where possible.

### **4.2.1 Escrow Lifecycle**

| Stage | Description |
| :---- | :---- |
| **Funded** | Buyer deposits funds into escrow when deal is accepted. Funds locked in kogi-bank. |
| **Milestone Release** | Seller delivers milestone. Buyer approves. Partial escrow released to seller. |
| **Full Release** | Final deliverable approved. Remaining escrow balance released. |
| **Dispute Hold** | Either party raises a dispute. Funds frozen pending resolution. |
| **Refund** | Deal cancelled before delivery. Funds returned to buyer minus any completed milestones. |
| **Auto-Release** | Configurable: if buyer doesn't dispute within N days of delivery, funds auto-release. |

## **4.3 Ratings & Reviews**

Every completed deal generates a mutual review. Reviews are tied to both the listing and the worker's Portfolio Resource item, feeding the platform-wide reputation graph.

| Field | Description |
| :---- | :---- |
| **review.rating** | 1–5 star score |
| **review.dimensions** | Quality, Communication, Timeliness, Value (each 1–5) |
| **review.text** | Free-text review body |
| **review.author\_id** | Reviewing party |
| **review.subject\_id** | Reviewed party |
| **review.deal\_id** | Linked deal reference |
| **review.verified** | Boolean: deal-linked reviews are auto-verified |
| **review.ai\_sentiment** | kogi-engine sentiment analysis: positive | neutral | negative \+ score |
| **review.response** | Reviewed party's optional public response |

## **4.4 Search & Discovery**

Marketplace and Exchange discovery is powered by a multi-layer search and recommendation stack, drawing on the kogi-engine sub-engines:

| Capability | Engine | Description | Trigger |
| :---- | :---- | :---- | :---- |
| Full-text Search | SearchEngine | Indexed listing titles, descriptions, tags, skills | User search query |
| Semantic Search | SearchEngine \+ QueryEngine | Intent-aware search via AI embedding similarity | User search query |
| Personalized Recommendations | RecommendationEngine | Collaborative \+ content-based filtering based on user profile | Feed refresh |
| Discover | AnalyticsEngine | AI-curated topic expansions beyond immediate interests | User action |
| Explore | AnalyticsEngine \+ GraphEngine | Deep expansion into related topics and entities | User action |
| AI Match | MatchEngine | Multi-dimensional worker-project-resource matching | Portfolio scan / blocker |
| Trending | TelemetryEngine | Listings with highest engagement velocity | Real-time feed |
| Similar Listings | RecommendationEngine | Content-based similarity on viewed listing | Listing view |

## **4.5 Notifications & Alerts**

The Feed and Notification system surfaces Marketplace and Exchange events in real time:

* New proposal received on active listing

* Counterproposal submitted by buyer

* Deal accepted — escrow funded

* Milestone due or overdue

* Escrow released — payment received

* New review posted on your listing

* AI match found for an open requirement

* Campaign goal reached

* Bid matched on Exchange order

* Contract expiry approaching

| 5\. AI INTEGRATION |
| :---- |

## **5.1 kogi-engine Connections**

The Marketplace and Exchange are among the most AI-intensive modules in the Kogi platform. Every sub-engine in kogi-engine has a meaningful role in powering the economic layer:

| Sub-engine | Role in Marketplace / Exchange | Key Output | Trigger |
| :---- | :---- | :---- | :---- |
| MatchEngine | Multi-dimensional worker-to-project matching | Ranked match list with compatibility scores | Blocker detected / RFP posted |
| RecommendationEngine | Personalized listing discovery | Curated listing feed per user persona | Feed refresh |
| SearchEngine | Full-text \+ semantic listing search | Ranked search results with highlights | User search |
| RiskEngine | Deal and counterparty risk scoring | PortfolioHealthScore \+ risk flags | Deal initiation / DD request |
| AnalyticsEngine | Portfolio signal computation for pricing | AssetValue \+ ArtifactMaturity models | Listing creation / DD |
| PersonalizationEngine | Feed density, ordering, segment targeting | ContentDeliveryConfig | Session start |
| TelemetryEngine | Trending listings, market activity velocity | EngineFlowSnapshot with market signals | Continuous |
| QueryEngine | SQL optimization for deal \+ order queries | QueryPlan with optimized execution | Complex data queries |
| GraphEngine | Dependency impact analysis on deal subjects | ImpactReport for deal scope analysis | Deal due diligence |
| OptimizationEngine | Pricing and resource allocation recommendations | OptimizationPlan with lever suggestions | Campaign / pricing review |

## **5.2 Oba AI Agent Workflows**

The Oba AI assistant runs several proactive workflows specifically for Marketplace and Exchange activity:

### **5.2.1 Sprint Unblocking via Labor Market**

When a project story is blocked, Oba automatically: (1) analyzes the blocker to build a skill profile, (2) searches the Labor Market for matching contractors via MatchEngine, (3) prepares personalized inquiry messages with project context attached, and (4) presents a ready-to-send shortlist to the user for one-click approval.

### **5.2.2 Listing Quality Optimization**

When a listing is underperforming (low CTR, low conversion), Oba analyzes the listing against high-performing comparables in the same category, surfaces specific improvement suggestions (title, description, pricing, tags), and offers to rewrite the listing body with one click.

### **5.2.3 Proposal Drafting & Review**

When a buyer submits an inquiry, Oba drafts a tailored Proposal response using the seller's portfolio context, current availability from kogi-gig, and pricing intelligence from the Exchange. It flags unusual terms in received proposals and surfaces risk factors identified by the RiskEngine.

### **5.2.4 Deal Monitoring & Escrow Management**

Oba monitors all active deals for milestone due dates, delivery confirmation delays, and communication gaps. It proactively drafts follow-up messages, flags overdue deliverables, and prepares escrow release or dispute documentation when needed.

### **5.2.5 Campaign Intelligence**

For active fundraising or equity campaigns, Oba tracks contribution velocity against the goal deadline, identifies likely-to-convert prospects from the platform's social graph, and drafts targeted outreach messages for the campaign owner's approval.

### **5.2.6 Morning Portfolio Briefing — Market Section**

The daily AI briefing includes a Marketplace & Exchange summary: active deals requiring attention, open proposals pending response, approaching contract expiries, escrow releases awaiting confirmation, and AI-matched opportunities that became available since the last session.

| 6\. PORTFOLIO SYSTEM INTEGRATION |
| :---- |

## **6.1 The Portfolio as the Source of Truth**

Every entity traded on the Marketplace or Exchange is ultimately a Portfolio Component. The Portfolio System is the source of record for all listings, deals, orders, and their associated analytics. This means the full power of the Portfolio System — its audit trail, governance model, CRDT distribution, computational analytics, and permission hierarchy — is available to every economic transaction.

## **6.2 Integration Points**

| Integration Point | Direction | Description |
| :---- | :---- | :---- |
| Listing Creation | Portfolio → Market | Asset/Artifact/Resource with visibility=Public is surfaced to the Marketplace listing index |
| ActionKind::Invest | Market → Portfolio | Investor action recorded in Portfolio Component users.investors set |
| ActionKind::Donate | Market → Portfolio | Donor action recorded in Portfolio Component users.donors set |
| ActionKind::Subscribe | Market → Portfolio | Subscription recorded in analytics.subscribers; seller notified on updates |
| Deal Completion | Market → Portfolio | Completed deal creates a new linked Asset or Resource item in buyer's portfolio |
| Campaign Funding | Market → Portfolio | Campaign funding recorded via ResourceAllocation (Budget kind) in Portfolio System |
| Due Diligence | Portfolio → Market | PortfolioHealthScore, ArtifactMaturity, and EventLog exported to Deal Room DD panel |
| Milestone Tracking | Portfolio ↔ Market | Project sprint milestones linked to Deal milestone schedule |
| Asset Valuation | Portfolio → Market | AssetValue model consumed for listing price intelligence and DD reports |
| Version Notifications | Portfolio → Market | Portfolio Component version bump triggers subscriber notifications |

## **6.3 Visibility & Access Control**

The Portfolio System's permission and visibility model governs what can be listed and who can see it:

| Visibility Level | Marketplace / Exchange Behavior |
| :---- | :---- |
| **Private** | Not surfaced to any marketplace or exchange. Owner only. |
| **Protected** | Visible only to explicitly invited users or community members. Invite-only listings. |
| **Public** | Fully discoverable on Marketplace and Exchange. Indexed by SearchEngine. |
| **Community** | Visible only to members of linked Community Space(s). Semi-public listings. |

| 7\. DATA ARCHITECTURE |
| :---- |

## **7.1 Database Schema Design**

### **7.1.1 Primary Storage — PostgreSQL**

Marketplace and Exchange data is persisted in PostgreSQL with JSONB for flexible payload fields. Core tables:

| Table | Description |
| :---- | :---- |
| **listings** | Primary listing records: id, type, owner\_id, portfolio\_component\_id, price, status, metadata JSONB |
| **orders** | Exchange orders: id, market, type, subject JSONB, price JSONB, status, counterparty\_id |
| **proposals** | Proposal records: id, listing\_id, buyer\_id, seller\_id, scope, terms JSONB, milestones JSONB, status |
| **deals** | Active and completed deals: id, proposal\_id, order\_id, escrow\_id, status, timeline JSONB |
| **campaigns** | Campaign records: id, type, goal JSONB, raised, backers JSONB, equity\_terms JSONB, status |
| **reviews** | Review records: id, deal\_id, author\_id, subject\_id, rating, dimensions JSONB, ai\_sentiment |
| **deal\_rooms** | Deal Room references: id, deal\_id, room\_id (FK to KRMS rooms) |
| **escrow\_locks** | Escrow lock records (FK to kogi-bank): id, deal\_id, amount, status, release\_conditions JSONB |
| **listing\_analytics** | Time-series analytics: listing\_id, timestamp, impressions, clicks, conversions |

### **7.1.2 Cache Layer — Redis**

* Active listing index — hot-path read cache for search results and recommendation feeds

* Order book state — live bid/ask queues for Exchange markets

* Session deal context — current Deal Room state per active negotiation

* User match cache — MatchEngine result cache keyed by user profile hash

### **7.1.3 Event Streaming — Kafka**

All Marketplace and Exchange events are published to Kafka for downstream consumption by kogi-engine and the Data Lake:

| Topic | Events | Consumers |
| :---- | :---- | :---- |
| marketplace.listings | listing.created, listing.updated, listing.published, listing.sold | SearchEngine, AnalyticsEngine, Feed Service |
| marketplace.deals | proposal.created, proposal.accepted, deal.started, deal.completed, deal.disputed | kogi-bank, AnalyticsEngine, AI Agent |
| exchange.orders | order.placed, order.matched, order.cancelled, order.expired | MatchEngine, AnalyticsEngine, Feed Service |
| marketplace.campaigns | campaign.created, campaign.backed, campaign.funded, campaign.failed | kogi-bank, AnalyticsEngine, kogi-organizations |
| marketplace.reviews | review.created, review.response\_added | RecommendationEngine, SearchEngine |

## **7.2 Analytics**

The Marketplace and Exchange analytics system tracks performance across five dimensions, all powered by kogi-engine's AnalyticsEngine:

### **7.2.1 Listing Performance Metrics**

* Impressions — total times listing was surfaced in search or feed

* Reach — unique users who viewed the listing

* Click-through Rate (CTR) — listing views / impressions

* Conversion Rate — proposals or orders / listing views

* Revenue — total value of completed deals originating from this listing

* Average Deal Value — mean deal size across all completed engagements

### **7.2.2 Worker / Seller Performance Metrics**

* Completion Rate — completed deals / accepted deals

* On-Time Delivery Rate — delivered by milestone date / total milestones

* Average Rating — aggregate review score across all completed deals

* Response Time — median time to respond to new inquiries

* Repeat Buyer Rate — buyers who initiated more than one deal

* Revenue Growth Rate — month-over-month change in earned revenue

### **7.2.3 Market Health Metrics**

* Liquidity — ratio of active orders to total registered users

* Match Rate — % of posted requirements that receive a qualified proposal

* Time to Match — median time from requirement posting to first qualified proposal

* Deal Close Rate — % of proposals that convert to funded deals

* Market GMV — Gross Merchandise Value of completed transactions per period

| 8\. UX ARCHITECTURE & USER FLOWS |
| :---- |

## **8.1 Marketplace UI Structure**

The kogi-marketplace UI is organized around a primary discovery surface and a deal management layer:

| View | Description |
| :---- | :---- |
| **Marketplace Home** | Personalized feed: AI-curated recommendations, trending listings, recent activity from followed workers |
| **Browse / Search** | Category-filtered search with facets: type, skills, price range, rating, availability, tags |
| **Listing Detail** | Full listing page: description, media, reviews, seller portfolio preview, Propose / Buy / Save actions |
| **My Listings** | Seller dashboard: all listings with performance analytics, edit, pause, promote controls |
| **Proposals & Deals** | Buyer/Seller inbox: active proposals, deal rooms, pending actions |
| **Deal Room** | Negotiation space: message thread, proposal panel, milestone tracker, escrow widget, Oba AI panel |
| **Campaigns** | Campaign browser: category filter, progress bars, backer counts, invest / donate actions |
| **Saved / Bookmarks** | User's saved listings collection |
| **Analytics Dashboard** | Seller performance: revenue, CTR, conversion, review trends, AI optimization suggestions |

## **8.2 Exchange UI Structure**

| View | Description |
| :---- | :---- |
| **Exchange Home** | Market overview: active orders, trending trades, AI match alerts, portfolio signals |
| **Labor Market** | Worker discovery: skill search, availability filter, AI match recommendations, post a requirement |
| **Commodities Market** | Portfolio item browser: trade type filter, asset category, valuation range |
| **Financial Market** | Investment opportunities: equity rounds, revenue share offers, campaign listings, due diligence access |
| **Resource Market** | Available resources: compute, licenses, equipment, capital offerings |
| **Order Book** | Live bid/ask order book for active markets |
| **My Orders** | All open and completed orders with status tracking |
| **Deal Room** | Shared with Marketplace — same negotiation infrastructure |
| **Due Diligence Portal** | DD report viewer: portfolio health, risk score, audit trail, asset valuation |

## **8.3 Core User Flows**

### **8.3.1 Buyer — Find & Hire a Worker**

1. Open Marketplace → Work category → search skills \+ availability

2. Browse AI-matched candidates or search results

3. View worker listing: portfolio preview, reviews, rate, AI reputation score

4. Open Deal Room via 'Propose' → write inquiry or let Oba draft it

5. Seller submits Proposal with scope, timeline, milestones, price

6. Review and negotiate terms in Deal Room

7. Accept Proposal → Fund Escrow via kogi-bank

8. Monitor work progress via milestone tracker

9. Approve deliverables → Escrow releases to worker

10. Submit mutual review → Analytics updated

### **8.3.2 Seller — List and Sell an Asset**

11. Open kogi-portfolio → select Asset or Artifact → Set visibility=Public

12. Marketplace listing wizard auto-populated from Portfolio Component data

13. Set price, pricing model, tags, and media → Publish

14. Receive notifications as buyers save, view, and propose

15. Respond to proposals via Deal Room (Oba can draft responses)

16. Accept → Escrow funded → Deliver asset or access

17. Completion confirmed → Payment received → Review collected

### **8.3.3 Exchange — Post a Requirement (RFP)**

18. Open Exchange → Labor Market → Post Requirement

19. Describe scope, required skills, timeline, budget ceiling

20. AI MatchEngine immediately surfaces pre-qualified candidates

21. Sellers submit competing proposals

22. Compare proposals (Oba can summarize and compare)

23. Select and accept best proposal → Deal Room activated

24. Fund escrow → Work begins

### **8.3.4 Campaign — Launch Equity Round**

25. Open Marketplace → Campaigns → Create Campaign

26. Link to Portfolio Program or Project

27. Set campaign type (Equity), funding goal, equity terms, deadline

28. AI generates campaign description draft from portfolio context

29. Publish → Share to Community via Spaces and Feed

30. Track contributions in real-time; Oba monitors velocity

31. Goal reached → Funds distributed via kogi-bank → Cap table updated in kogi-organizations

| 9\. TECHNICAL ARCHITECTURE |
| :---- |

## **9.1 Service Architecture**

| Service | Description |
| :---- | :---- |
| **kogi-marketplace (Go)** | Primary REST/GraphQL API service for listings, proposals, and campaigns. Publishes events to Kafka. |
| **kogi-exchange (Go)** | Order book management, trading mechanics, and market data API. Publishes events to Kafka. |
| **kogi-deal (Go)** | Deal lifecycle orchestration: proposal → deal → escrow → delivery → settlement → review. |
| **kogi-match (Go bridge)** | Thin bridge service wrapping MatchEngine gRPC calls from kogi-engine Scala service. |
| **kogi-engine (Scala)** | Intelligence substrate: MatchEngine, RecommendationEngine, RiskEngine, SearchEngine, AnalyticsEngine. |
| **kogi-bank (Go/Rust)** | Escrow, payment, wallet, settlement, invoice services consumed by both Market and Exchange. |
| **kogi-network (Go)** | Gateway management, publish/subscribe, service-to-service communication. |

## **9.2 Data Flow**

| *User Action → kogi-network (Gateway) → kogi-marketplace / kogi-exchange API → Portfolio System (Rust) → Kafka Event Bus → kogi-engine (Scala) → Recommendations / Risk / Match → Feed Service → Client Notification* |
| :---- |

## **9.3 Technology Stack**

| Layer | Technology | Component | Notes |
| :---- | :---- | :---- | :---- |
| Core Services | Go | kogi-marketplace, kogi-exchange, kogi-deal | REST \+ gRPC; Kafka producers |
| Systems Layer | Rust | PortfolioSystem, core modules | In-memory \+ PostgreSQL backend planned |
| Intelligence | Scala 3 / JVM | kogi-engine (all sub-engines) | gRPC server on port 9100 |
| Database | PostgreSQL | Primary persistence | JSONB for flexible payloads |
| Cache | Redis | Hot-path reads, order book, session state | Pub/sub for real-time fanout |
| Streaming | Apache Kafka | Event bus, analytics pipeline, audit log | Topics per module |
| Web Client | Angular \+ TypeScript | kogi-web-client | WebSocket for real-time Deal Rooms |
| Mobile | Kotlin / iOS | kogi-mobile-client | Push notifications for deal events |
| Search Index | Meilisearch / Typesense | Listing search backend | Fed from Portfolio plugin hooks |
| Build | Bazel | Monorepo build system | Cross-language build consistency |

## **9.4 Security & Governance**

* All marketplace and exchange actions validated against PortfolioSystem PermissionTier hierarchy

* Deal escrow operations require explicit user approval — no AI-initiated fund movements

* Full audit trail via Portfolio EventLog for every listing, deal, order, and payment action

* PolicyEngine hooks enforced on all listing publications and deal completions

* Federated identity via OIDC/SAML for cross-platform authentication

* Encrypted Deal Room messages; at-rest encryption for DD report snapshots

* Rate limiting and anomaly detection on order book to prevent market manipulation

| 10\. OPEN ITEMS & FUTURE CONSIDERATIONS |
| :---- |

## **10.1 Near-Term Open Items**

| Item | Description |
| :---- | :---- |
| **Order Book Persistence** | Exchange order book currently in-memory. Needs Redis-backed persistent queue with Kafka replay for crash recovery. |
| **Escrow Smart Contracts** | For crypto/token deals, escrow should be backed by on-chain smart contracts. Define bridge protocol between kogi-bank and supported chains (Ethereum, Polygon). |
| **AI Writeback Protocol** | Define typed protocol for Oba agent to create Proposal drafts, update deal metadata, and append governance notes without bypassing PermissionTier checks. |
| **Match Algorithm Tuning** | MatchEngine currently uses linear multi-dimensional scoring. Define weighting schema per market type (Labor vs. Commodities vs. Financial) with A/B experiment infrastructure. |
| **CRDT for Deal State** | Deal state (proposal counters, milestone status) needs CRDT-safe merge strategy for multi-device and offline scenarios. |
| **SearchEngine Warm-up** | SearchEngine listing index must be populated via Portfolio System plugin hooks on component creation/update. Wire on\_component\_created → listing index upsert. |
| **Provider Integration Depth** | Initial provider integrations (Upwork, Fiverr, etc.) are tile-level deep links. Full API integrations for profile import and order sync require provider-specific OAuth flows. |

## **10.2 Phase 4 Considerations**

* Global marketplace liquidity pools — pooled order routing across federated Kogi nodes

* Autonomous agent-driven deal negotiation — Oba negotiates within user-defined constraints without per-action approval

* On-chain settlement for financial instruments — token issuance, distribution, and cap table management on-chain

* Portfolio Index Products — curated composite indices of portfolio assets tradable as a single instrument

* Cross-platform marketplace federation — federated listing discovery across ShangoOS platforms (Kogi, Ume, Qala)

* Algorithmic pricing suggestions — ML-driven price optimization based on market comps, demand signals, and portfolio health

* DAO governance integration — on-chain voting for community fund distributions and cooperative economic decisions

---

| 11\. GRANTS, MICROFINANCING & CROWDRESOURCING |
| :---- |

## **11.1 Grants Marketplace**

The Kogi Marketplace extends into a grants discovery and application layer, enabling independent workers, collectives, and cooperatives to find, apply for, and manage grants without leaving the platform.

| Feature | Description |
| :---- | :---- |
| **Grant Registry** | Searchable catalog of government, foundation, platform, and community grants; filtered by eligibility, category, amount, deadline, and entity type |
| **Grant Listings** | Grant providers (foundations, DAOs, government bodies) list grants as Marketplace items with eligibility criteria, required documents, and review timeline |
| **Grant Application Flow** | Structured application workflow: grant discovery → eligibility check → application form → document submission → review → award; all linked to Portfolio Program/Project |
| **Grant Fund Disbursement** | Awarded funds routed through kogi-bank Campaign Account; milestone-based disbursement against grant agreement |
| **Community Grant Pools** | Organizations and collectives create shared grant pools funded by member contributions; governed by community governance proposals |
| **Grant Impact Reporting** | Auto-generated impact reports from Portfolio System analytics; submitted to grant providers on schedule |
| **GrantEngine Integration** | kogi-engine GrantEngine scores grant matches by eligibility, portfolio strength, and historical award data; Oba proactively surfaces relevant grants |

## **11.2 Microfinancing**

| Feature | Description |
| :---- | :---- |
| **Microloan Marketplace** | Workers and organizations can post loan requests; community lenders browse and fund fractional amounts (crowdlending) |
| **Loan Application** | Borrower submits application with portfolio context, income history, use-of-funds description, and requested terms |
| **Credit Scoring** | kogi-engine RiskEngine computes platform credit score from portfolio activity, income, repayment history, and community standing |
| **Loan Terms Negotiation** | Lender proposes terms; borrower accepts or counter-proposes via Exchange Deal mechanics |
| **Repayment Tracking** | Automated repayment schedule; kogi-bank processes payments and distributes pro-rated returns to lenders |
| **Community Lending Pools** | Collectives establish shared lending pools; members contribute capital and access loans at preferential rates |
| **Collateral Support** | Portfolio assets (IP, equity stakes, revenue streams) can be pledged as collateral; locked in kogi-bank escrow |

## **11.3 Crowdresourcing**

Crowdresourcing enables portfolio items and programs to receive community contributions of labor, capital, assets, and knowledge — tracked with full attribution and governed by the Portfolio System.

| Feature | Description |
| :---- | :---- |
| **Crowdresourcing Listings** | Portfolio owners open specific items for community contribution; listed on the Marketplace with contribution type, goal, and reward rules |
| **Contribution Types** | Labor (hours, tasks), Capital (monetary), Assets (designs, code, data), Knowledge (research, strategy, documentation), Artifacts (templates, playbooks) |
| **Contribution Ledger** | All contributions recorded with full attribution in Portfolio EventLog; contribution weight computed by CollaborationEngine |
| **Reward Rules** | Configurable rewards: equity allocation, revenue share, direct payment, recognition badge, platform credits |
| **Steward Review** | Contributions go through configurable review: AutoAccept | StewardReview | GovernanceVote |
| **Crowdresourcing Analytics** | Contribution velocity, contributor diversity, goal progress, open contribution gaps — surfaced in Portfolio Dashboard |

---

| 12\. SHARED PORTFOLIOS & PORTFOLIO RESOURCE SHARING |
| :---- |

## **12.1 Shared Portfolio Listings**

Shared portfolios — owned by teams, organizations, collectives, cooperatives, or federations — are first-class listable entities on the Marketplace and Exchange:

| Listing Type | Description |
| :---- | :---- |
| **Shared Portfolio Showcase** | A group portfolio published for community visibility; followers and investors can track progress |
| **Collaborative Project Listing** | A portfolio project open to new contributors; lists skills needed, contribution types, and reward model |
| **Portfolio Resource Listing** | Specific portfolio resources (templates, playbooks, tools, datasets) shared with or sold to the community |
| **Federation Portfolio** | Cross-organization portfolio open to aligned orgs for joint investment or resource contribution |
| **Crowdfunded Portfolio Program** | A portfolio program raising capital from community investors via equity, revenue share, or donation |

## **12.2 Portfolio Resource Exchange**

The Exchange subsystem handles structured trading of portfolio-derived resources:

| Resource Type | Exchange Mechanism | Settlement |
| :---- | :---- | :---- |
| **Portfolio Templates & Playbooks** | Fixed-price listing or auction; buyer gains fork rights | kogi-bank instant settlement |
| **Intellectual Property / Licenses** | License agreement deal flow; DD report for IP assets | Escrow-backed settlement |
| **Portfolio Equity Stakes** | Secondary market offer/bid for equity positions in projects or orgs | Governance approval + cap table update |
| **Revenue Streams** | Forward-sale of future revenue share; exchange pricing via MatchEngine | kogi-bank revenue routing |
| **Tool Integrations** | Toolbox and toolchain configurations listed and traded | Fork + provider credential transfer |
| **Data & Datasets** | Research, analytics datasets, or model outputs listed with licensing terms | Access-key settlement via kogi-bank |

---

| 13\. BOOKING, CRM & LOGISTICS |
| :---- |

## **13.1 Booking & Scheduling**

The Marketplace surfaces booking capabilities for independent workers and organizations that offer time-based services (creative professionals, consultants, performers, educators, multi-worker agencies).

| Feature | Description |
| :---- | :---- |
| **Bookable Profiles** | Worker and organization profiles can be set as "bookable"; potential clients browse and initiate booking requests |
| **Availability Calendar** | Public availability calendar on bookable profiles; client-facing booking widget for self-service scheduling |
| **Booking Request Flow** | Client submits inquiry → worker reviews and quotes → client accepts → deposit collected via kogi-bank → booking confirmed |
| **Multi-Worker Booking** | Book a team or collective for multi-role events; availability conflict detection across all assigned workers |
| **Resource Booking** | Bundle equipment, venue, or other resources into a booking package |
| **Booking Analytics** | Conversion rate, utilization, average deal size, peak demand periods — surfaced in worker Marketplace dashboard |

## **13.2 CRM & Lead Management**

| Feature | Description |
| :---- | :---- |
| **Lead Capture** | Marketplace profile inquiry forms feed directly into worker's CRM pipeline |
| **Client Database** | Full client record: contact, booking history, spend, communication log, tags |
| **Pipeline Stages** | Inquiry → Qualified → Quoted → Negotiating → Booked → Retained → Lapsed |
| **Automated Follow-ups** | Configurable email/message sequences on stage transitions, quote expiry, post-booking |
| **CRMEngine Integration** | kogi-engine CRMEngine scores lead probability-to-book; Oba recommends outreach timing and messaging |
| **Retention Analytics** | Repeat booking rate, lifetime value, churn risk; feed into Marketplace profile ranking |

## **13.3 Contracts & Invoicing (Marketplace Context)**

| Feature | Description |
| :---- | :---- |
| **Auto-Generated Contracts** | Booking confirmation auto-generates contract from template; pre-populated with booking details |
| **E-Signature Workflow** | Contract sent for e-signature via ProviderSystem (DocuSign, HelloSign); signed status tracked |
| **Invoice Automation** | Deposit invoice and balance invoice auto-generated from booking; payment reminders via kogi-bank |
| **Contract Versioning** | All contract revisions versioned and stored in Portfolio System; audit-ready |

## **13.4 Logistics Tracking**

| Feature | Description |
| :---- | :---- |
| **Tour / Event Itinerary** | Multi-stop itinerary linked to Marketplace bookings; shareable with clients and crew |
| **Equipment Tracking** | Rental and owned equipment status per booking; return deadlines with automated reminders |
| **Transportation Management** | Vehicle assignments, routing, and mileage for multi-location bookings |
| **Crew Schedules** | Role-based crew assignments per booking; call times and wrap times distributed via kogi-community logistics room |
| **Live Status Updates** | In-app logistics room per booking for real-time status broadcast to all stakeholders |
| **Expense Tracking** | Per-booking expense log; reconciled against booking budget in kogi-bank |

© 2026 Kogi Platform — Marketplace & Exchange System Design Document — Confidential