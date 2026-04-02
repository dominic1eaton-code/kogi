

   
**KOGI**  
Independent Worker Operating System  
*Unified Work Management Platform for the Independent Economy*  
 

**Platform Design Document**  
v3.0  ·  March 2026  ·  Confidential

**Kogi Team**

Dashboard  ·  Portfolio  ·  Wallet  ·  Office  ·  Marketplace  ·  Spaces  ·  Hub  ·  Agent

# **1\. Platform Overview**

## **1.1 Vision & Mission**

Kogi is the Independent Worker Operating System — a unified, configurable, and community-connected work management platform designed for the modern independent economy. Kogi unifies the tools and systems that workers use to execute their processes, generate and consume value, and connect with one another into a single scalable platform.

The platform provides independent contractors, freelancers, gig workers, entrepreneurs, creatives, and cooperative organizations with the infrastructure they have historically had to assemble piecemeal: project management, financial accounts, marketplace access, community, benefits, and AI assistance — all tracked, governed, and maintained through a master hyperspreadsheet.

*Design Principle: Everything in the Kogi ecosystem is a Portfolio Item. The Portfolio System is therefore the universal abstraction layer that every platform application builds upon.*

## **1.2 Core Platform Architecture**

The Kogi Platform has eight core modules that together constitute a complete independent worker operating system:

| Module | Code | Primary Responsibility | Sub-Modules / Key Features |
| :---- | :---- | :---- | :---- |
| Dashboard | KDSH | High-level portfolio OS overview; independent worker desktop screen | Portable Benefits · Initiatives · Morning Briefing · OKR Progress |
| Portfolio | KPRT | Complete work lifecycle management; master hyperspreadsheet | Programs · Projects · Assets · Artifacts · Resources · Tools · Containers |
| Wallet / Bank | KBNK | Financial operating layer; accounts, ledgers, payments, escrow | Accounts · Wallets · Ledger · Escrow · Campaigns · Invoicing · Tax |
| Office | KOFF | Portfolio item execution and management system | WMS · RMS · Booking · CRM · Contracts · Logistics |
| Marketplace | KMKT | Portfolio item markets, exchanges, distribution, supply chain | Labor Market · Asset Store · Exchange · Deal Room · Campaigns |
| Spaces | KSPC | Portfolio space and community management system | Feed · Rooms · Events · Organizations · Social Graph · Notifications |
| Hub | KHUB | Multi-portfolio governance management system | Shared Portfolios · Governance · Voting · Rights Management · Federation |
| Agent | KAGT | Portfolio automations; AI assistant system | Oba AI · Multi-agent orchestration · Workflow automation · Analytics |

## **1.3 Platform Substrate — Hypergrid**

Kogi is built on Hypergrid, an N-dimensional distributed spreadsheet engine. Every entity in the platform is a row in a Hypercube, every field is a cell at a typed coordinate, and the entire system is governed by CRDTs (Conflict-free Replicated Data Types), an append-only EventLog, a Hypergraph relational layer, and an AI writeback pipeline.

The primary Hypercube for Kogi is kogi.portfolio.components, which holds every entity in a worker's professional life — programs, projects, tasks, artifacts, assets, gigs, benefits, and financial records — as HyperRows in a single N=2 cube. The 35-sheet system is 35 saved HypercubeViews over the same underlying data.

| Substrate Component | Description |
| :---- | :---- |
| Universal Cell Store | N-dimensional coordinate space with per-attribute CRDT semantics. D₁ \= entity axis (UUID), D₂ \= property axis (field name), D₃+ add time, segment, or other dimensions. |
| Hypergraph | Relational layer. Every HyperRow is a HypergraphNode. Typed edges (Hierarchy, Dependency, Association, CrossGridLink, Employs, Collaborates) connect entities within and across Grids. |
| EventLog | Append-only, immutable, time-travelable. Every mutation is recorded with actor, VectorClock, and payload. AS\_OF operator replays to any prior timestamp. |
| CRDT Engine | Per-attribute CRDT semantics: LastWriteWins for scalars, OR-Set for set fields, Lattice for lifecycle states, PNCounter for transactional accumulators, MaxRegister for versions. |
| AI Writeback Pipeline | kogi-engine subscribes to EventLog mutations, computes domain signals (health scores, risk scores, anomaly flags), and writes results back as system-only AI computed attributes. |
| HyperQL | SQL-plus query language with N-dimensional cell references, DimSlice, DimFold, ROLLUP, DRILLDOWN, AS\_OF time-travel, TRAVERSE GRAPH, and IDENTITY scoping. |
| KLNK (Kogi Link Network) | The economic graph of interconnected worker portfolios. CrossGridLink edges with ShadowCells model employment, investment, collaboration, and marketplace transactions. |

# **2\. Core Personas**

## **2.1 Persona Overview**

Kogi serves a diverse range of independent worker archetypes. Personas are labels associated with users to provide customized experiences and help other users find and filter participants by type. A single user may carry multiple personas simultaneously.

### **2.1.1 The Freelancer**

Entrepreneurs, micropreneurs, gig workers, freelancers, contract workers, consultants, coaches, service-based workers, developers, and side hustlers. This is the primary platform persona — workers who need a systematic work management system and resources to build and develop their personal work portfolios.

### **2.1.2 The Creative**

Content creators, musicians, artists, writers, journalists, media personalities, podcasters, and influencers. Creators looking for a central system to track, manage, and showcase their work, and to connect with other creatives and with those who may desire their services.

### **2.1.3 The Enthusiast**

Hobbyists, spectators, donors, investors, venture capitalists, and skilled workers looking for a new challenge. People looking to invest in things that interest them — video games, new products, interesting service offerings. Enthusiast portfolios tend to contain items they have invested in or contributed to, rather than projects they are directly working on.

### **2.1.4 The Professional**

Professional services workers in a '9-to-5' who want a work system to manage their jobs — the 'employed entrepreneur' or 'entrepreneurial employee.' Legal services, accountants, professional service consultants, engineers, and medical professionals. Veterans who already have mature bespoke work systems in place; the platform facilitates and organizes their assistants and agents to manage their systems, and allows them to market and promote themselves.

### **2.1.5 The Cooperative**

Organizations, teams, collectives, federations, autonomous organizations, and cells. Multi-participant entities that need shared governance, collective treasury, revenue distribution, and decentralized work portfolio management.

### **2.1.6 The Explorer**

People just checking out the platform — looking for community, resources, time, and money. Cold-start users with minimal interaction history; the platform defaults to trending and popular content for this persona.

## **2.2 Macro / Micro Persona Dimensions**

| Dimension | Variants |
| :---- | :---- |
| Technical sophistication | Technical users · Non-technical users · Power users |
| Engagement depth | Power users (high frequency, advanced features) · Casual browsers (low engagement, trending content) · Specialists (deep domain focus) · Collaborators (high social engagement) |
| Economic role | Sellers of labor/assets · Buyers/hirers · Investors/donors · Multi-role participants |
| Organization type | Solo worker · Team member · Cooperative member · Collective participant · Federation member |

# **3\. Portfolio System**

## **3.1 Overview**

The Portfolio is the root domain concept of Kogi. It is the complete, unified record of an independent worker's professional and economic existence. Every entity in the worker's operational life is a row in the Portfolio System. The Portfolio System manages the complete work lifecycle — from produced idea to delivered consumable value and back again — in a closed-loop producer and consumer independent worker economy.

All portfolio items are contained within and managed by a master hyperspreadsheet. Users simultaneously produce and consume work items to supply and deplete their portfolios of value and work.

*Portfolio Root Concept: The Portfolio IS the system. Every row in the master hyperspreadsheet IS a portfolio component. The spreadsheet IS the operating system.*

## **3.2 Portfolio Structure Hierarchy**

The portfolio structure follows this hierarchy from broadest to most granular:

| Level | Description |
| :---- | :---- |
| Portfolio | Top-level container organizing all of a user's or organization's work, assets, and goals. May contain sub-portfolios, programs, and projects. Functions as a content management system. |
| Program | Multiple projects aligned to achieve common outcomes and objectives; shares milestones across constituent projects. |
| Project | A temporary endeavor of work to achieve a finite, distinct outcome (state change / transform). Produces assets and artifacts. |
| Activity | Work performed within a project — a set of coordinated tasks and events contributing to project progress. |
| Task (WBS) / Event (TBS) | The atomic execution unit. Tasks are work breakdown items; Events are time breakdown items anchored to the calendar. |

## **3.3 Portfolio Item Types**

All portfolio items carry RBAC permission models (Public, Protected, Private, Hidden) and are tradeable, configurable, deployable, distributable, and extensible. Users can perform a rich set of actions on any item: like, comment, share, subscribe, watch, bookmark, trade, book, donate, invest, fund, collaborate, contribute, and configure.

### **3.3.1 Assets**

Capability-enabling items which can be consumed for future work pipelines. Transformative intermediary items between raw inputs and final delivered outputs.

* Skills, knowledge, information

* Software, devices, agents, machines

* Equity, securities, liquidity

* Portable benefits

* Funding, donations, investments, accounts, wallets

* Offers, deals, bids, proposals, requests, bookings, consultations, contracts, gigs, jobs

* Ideas, concepts, prototypes, MVPs, designs, mockups, wireframes, renderings, schematics, models, simulations

* IP, rights, licenses, patents, marks, agreements, contracts, memos

### **3.3.2 Artifacts**

Delivered, tangible outcome items. Produced, refined output items representing completed work products. These are the deliverables a worker produces and can showcase or sell.

### **3.3.3 Solutions**

Final delivered product and artifacts sets, deployed and distributed 'in the wild.' Delivered packages, goods, products, and services that constitute the completed output of a portfolio project.

### **3.3.4 Resources**

Raw, consumable input items. Land, labor, capital, estate, real estate, people, skills, knowledge, information, contacts, agents, liquidity, securities, and produced artifacts/solutions ready for consumption. This supports the platform's closed-loop reusability, renewability, recyclability, and sustainability model.

### **3.3.5 Entities**

Teams, groups, legal entities, organizations (autonomous, decentralized), communities, agents, and executors. Multi-participant organizational structures operating within the portfolio system.

### **3.3.6 Links (LinKtrees)**

| Link Structure | Description |
| :---- | :---- |
| Linktree | A user's 'personal work IP address/router' — the interface for all external communications routing, accounts across external platforms, integration toolchains, and RBAC connections. |
| Link Branches | Professional branches · Developer branches · Public branches · Private branches · Custom branches |
| Link Leaves | Specific integration endpoints — connections to individual external platforms and services. |
| Linkforests | Collections of linktrees across multiple identities or organizational contexts. |
| Linknetworks | Interconnected linkforests forming the economic graph of the KLNK system. |

### **3.3.7 Tools**

Portfolio items used to perform and execute work and tasks. Organized hierarchically:

* Toolbox — the top-level tool container for a user's work environment

* Toolset — a categorized collection of related tools within a toolbox

* Toolkit — a bundled set of tools for a specific use case or workflow

* Toolchain — a sequential series of tools executing in an automated pipeline

* Tool — individual integration or platform capability

### **3.3.8 Studio**

Independent worker preference configuration items. The Studio captures a user's personal work environment settings, aesthetic preferences, creative space configurations, and custom work system parameters.

## **3.4 Portfolio Containers**

Containers organize and structure portfolio items. They provide flexible, typed grouping and storage structures for heterogeneous portfolio elements.

| Container Type | Description |
| :---- | :---- |
| Space | Bounded divisions or separations within a portfolio. Includes workspaces (spaces of execution), namespaces (named spaces), and rooms (spaces within spaces). |
| Binder | Can be filled with heterogeneous portfolio elements such as free-floating portfolios, programs, and projects, as well as other containers. |
| Guidebook | Structured collection of documentation for a portfolio item. |
| Playbook | Collection of developed work processes, programs, and strategies. |
| Notebook | Collection of long-lasting notes and letters. |
| Memobook | Collection of sticky-note ephemeral memos. |
| Resourcebook | Contains resource ledger, resource connections and integrations, vendor and supply chain information, and resource versioning history. |
| Schedulebook | Time and schedule book for a portfolio item. |
| Reportbook | Analytics, metrics, KPIs, and performance indicators. |
| Contactbook | List of contacts and connections including CRM profiles. |
| Dossier | Very detailed collection of item documents, reports, evidence, and facts. |
| Brief | Contains charter and high-level information about an item. |
| Folder | Hierarchical filesystem container; name-ordered. |
| Record | Ordered, sequential list of portfolio items. |
| Ledger | Master list of records and accounts. |
| Schedule | List of items; planning temporal breakdown structure. |
| Registry | Named index; list of associations and ownership. |
| Catalogue | Detailed, categorized list of items; curated, searchable, structured, and filterable. |
| Archive | Long-term stored and consolidated (encoded) portfolio items. |
| Directory | Searchable index of items. |

## **3.5 Task Management Systems**

### **3.5.1 Work Breakdown Structure (WBS)**

The WBS decomposes work from strategic themes down to atomic tasks:

* Work Packages — top-level WBS nodes

* Themes — broadest strategic investment areas; parent of multiple initiatives

* Initiatives — coordinated sets advancing a theme or strategy

* Epics — large capabilities decomposed into stories; fits a PI or quarter

* Stories — primary delivery units; estimable, testable, completable in a sprint

* Tasks — atomic execution steps within a story

### **3.5.2 Temporal Breakdown Structure (TBS)**

The TBS organizes work across time dimensions:

| TBS Component | Description |
| :---- | :---- |
| Calendar | Grid/spatial temporal breakdown for day-level scheduling. |
| Timeline | Historical, provenance view of milestones and state changes. |
| Gantt | Detailed, operational time view with dependency-aware bar charts and critical path detection. |
| Schedules | Itemized time view of work assignments and resource allocations. |
| Roadmaps | High-level strategic view of long-horizon milestones; externally shareable. |
| Timeboxes | Time durations and time packets: Quarters, Program Increments, Sprints. |
| Events | Scheduled occurrences — meetings, bookings, appointments, reviews. |

## **3.6 Permission Model**

Portfolio items support a full RBAC permission model with the following visibility tiers:

| Permission Tier | Ordinal | Visibility | Capabilities |
| :---- | :---- | :---- | :---- |
| Viewer | 0 | Public | Read-only, anonymous access |
| Subscriber | 1 | Public/Protected | Bookmark, save, follow, subscribe |
| Contributor | 2 | Protected | Comment, react, poll, join, submit contributions |
| Editor | 3 | Private | Edit content, bump version, manage content |
| Manager | 4 | Private | Manage settings and members, configure policies |
| Owner | 5 | Any | Full ownership privileges, all actions including transfer |
| Admin | 6 | System | Super-admin and platform-level override capabilities |

## **3.7 CRDT Semantics by Attribute Type**

Every attribute in the Portfolio System carries explicit CRDT semantics ensuring safe concurrent mutation across distributed nodes:

| CRDT Type | Attribute Examples | Merge Behavior |
| :---- | :---- | :---- |
| LastWriteWins (LWW) | name, description, status, config, JSON blobs | Highest VectorClock timestamp wins; actor ID tiebreak |
| OR-Set | owners, tags, children, dependencies, toolbox\_ids | All concurrent Adds survive; Removes target only exact unique\_tag |
| PNCounter | budget\_spent, hours\_logged, allocated\_units | Sum of positive/negative increments across all nodes; any sign permitted |
| GrowOnlyCounter | view\_count, follower\_count, restart\_count | Sum of all increments; decrement operations rejected |
| MaxRegister | version, sequence\_number, schema\_version | Highest numeric value always wins regardless of causal ordering |
| Lattice | lifecycle\_state, module\_state, ccr\_status | join() \= least upper bound in partial order; never regresses without Admin override |
| AppendLog | activity\_log, comment\_thread, decision\_log | All entries preserved; causally ordered by VectorClock |
| DeepMergeJson | plugin\_configs, feature\_flags, settings\_tree | LWW per leaf JSON key; only keys in delta are updated |
| System-LWW | health\_score, risk\_score, anomaly\_flag, narrative\_summary | AI-written; PermissionTier::System enforced; no human writes |

# **4\. Wallet & Bank System (kogi-bank)**

## **4.1 Overview**

kogi-bank is the financial operating layer of the Kogi Platform — the centralized banking, accounting, payment, escrow, investment, and capital management system for independent workers, autonomous organizations, collectives, cooperatives, and independent teams.

Every transaction, every account balance, every escrow lock, and every campaign contribution flows through kogi-bank. Every financial entity is a first-class Portfolio Item in the Portfolio System, giving it the same governance, versioning, and analytics infrastructure as any other platform component.

## **4.2 Currency & Transaction Units**

The Kogi platform introduces its own transaction primitives:

| Currency / Unit | Description |
| :---- | :---- |
| Zawadi | The basic unit of transaction on the Kogi platform. Can translate to and from liquidity (USD, foreign currency), equity, securities, credits, debits, taxes, debts, and digital currency. |
| Sundiata Coin | The principal cryptocurrency for the Kogi platform, operating on top of the Sundiata distributed ledger. |
| Exchangeable Resources | Liquidity · equity · securities · credits · skills · bartered items · labor · contracts · agreements · gigs · consultations · time · bookings · meetings · appointments |

## **4.3 Account Architecture**

Accounts are stores of capital, resources, liquidity, or equity. Wallets are the transaction surfaces — the active interfaces through which financial items move.

### **4.3.1 Personal Accounts**

| Account Type | Description |
| :---- | :---- |
| Personal Checking | Day-to-day spending and income reception; primary transaction account. |
| Personal Savings | Reserved capital for specific goals or emergencies. |
| Operations Account | Dedicated business-expense account for gig and freelance activity. |
| Investment Account | Long-term capital deployment in equity, funds, or portfolio assets. |
| Tax Reserve Account | Automated set-aside for self-employment tax obligations; quarterly payments. |
| Retirement Account | Long-term personal retirement savings — SEP-IRA, 401k portability, PEP. |
| Equity Account | Record of equity positions in external organizations or portfolio projects. |
| HSA Account | Health Savings Account — portable benefit, pre-tax health expense fund. |
| Portable Savings Account | Gig platform contribution tracking account; receives % contributions from platforms (e.g., 4% of pre-tip earnings). |
| Income Protection Fund | Reserves for income replacement during missed work, disability, or occupational accident. |
| Professional Development Account | Portable education and training fund; receives contributions from platforms or self. |
| Emergency Savings Account | Rapid-access emergency buffer, separate from general savings. |
| Crypto / Token Account | Digital asset and token holdings; on-chain transfers via crypto bridge. |

### **4.3.2 Organizational Accounts**

| Account Type | Description |
| :---- | :---- |
| Operating Account | Primary revenue and expense account for the organization; multi-sig controls. |
| Payroll Account | Funds designated for member and worker compensation; automated distribution. |
| Project Account | Per-project budget pool; linked to a specific Portfolio Project. |
| Cooperative Treasury | Multi-sig controlled shared fund; democratic governance for disbursements. |
| Campaign Account | Dedicated account for an active fundraising or resource campaign. |
| Escrow Reserve | Collective escrow for multi-party deals; M-of-N release. |
| Mutual Aid Fund | Peer-support pool for member emergencies; governed by mutual aid policy. |
| Equity Pool | Shared equity register for member equity distribution; vesting schedules. |

## **4.4 Wallet Types**

| Wallet Type | Primary Use |
| :---- | :---- |
| Personal Wallet | Day-to-day financial activity; receive income, pay expenses. |
| Operations Wallet | Business expense management for gig and freelance activity. |
| Marketplace Wallet | Buying and selling in kogi-marketplace. |
| Exchange Wallet | Active trading and deal execution. |
| Investment Wallet | Deploying capital in equity, campaigns, and financial instruments. |
| Payroll Wallet | Receive compensation from contracts and gigs. |
| Tax Wallet | Automated tax savings and quarterly payments. |
| Campaign Wallet | Launch and manage fundraising campaigns. |
| Collective Wallet | Shared wallet for collective and cooperative member activity. |
| Escrow Wallet | Temporary custody of deal funds. |
| Crypto Wallet | Digital asset transactions; on-chain transfers. |

## **4.5 Auto-Split Rules**

A powerful Wallet feature is configurable auto-split: when income arrives in a wallet, it is automatically routed to multiple accounts according to defined percentage rules. Example: Jordan sets up 60% → Personal Checking, 20% → Operations Account, 15% → Tax Reserve, 5% → Investment Account. Every contract payment auto-applies the split atomically.

## **4.6 Double-Entry Ledger**

The kogi-bank ledger implements double-entry bookkeeping. Every financial event creates at least two ledger entries: a debit to one account and a credit to another. The ledger is append-only and immutable — corrections are made by posting reversing entries, not by editing.

| Journal Type | Entry Types |
| :---- | :---- |
| Income Journal | Client payments, marketplace proceeds, gig payouts, subscription revenue |
| Expense Journal | Subscriptions, tools, equipment, platform fees, professional services |
| Asset Journal | Equipment purchases, IP acquisitions, investment deployments |
| Liability Journal | Loans, deferred revenue, outstanding payables |
| Equity Journal | Equity issuances, cap table changes, vesting events |
| Escrow Journal | Escrow locks and releases from marketplace and exchange deals |
| Campaign Journal | Campaign contributions received and disbursed |
| Tax Journal | Tax reserve allocations, estimated tax payments, tax refunds |
| Payroll Journal | Member and worker compensation payments |
| Adjustment Journal | Corrections, reversals, write-offs |

## **4.7 Escrow System**

The kogi-bank Escrow System provides secure, conditional custody of funds for Marketplace and Exchange deals. Every escrow is a first-class financial entity with its own account, ledger trail, governance policies, and milestone schedule.

| Escrow Lifecycle Stage | Ledger Effect |
| :---- | :---- |
| Deal Accepted | No ledger entry yet; escrow record created. |
| Funded | Debit: Buyer Checking → Credit: Escrow Reserve Account. |
| Milestone Approved | Debit: Escrow Reserve → Credit: Seller Payroll / Checking. |
| Dispute Raised | No movement; funds frozen; Dispute entity created. |
| Auto-Release | Fires N days after delivery if buyer is silent; Debit: Escrow → Credit: Seller. |
| Full Release | All milestones approved; Escrow account balance → 0\. |
| Refunded | Deal cancelled; Debit: Escrow Reserve → Credit: Buyer Checking. |

## **4.8 Campaigns & Group Economics**

| Campaign Type | Description |
| :---- | :---- |
| Fundraise | Donation-based funding; non-equity, non-repayable; lump sum on goal met. |
| Equity Crowdfunding | Fractional ownership in exchange for capital; Reg CF / Reg A+ compliance; SAFE notes supported. |
| Revenue Share Pool | Backers share in future revenue from a Portfolio Component. |
| Group Buy | Collective purchase at negotiated bulk rate; shared ownership. |
| Community Bond | Debt issuance from a cooperative or collective; fixed-rate repayment. |
| Mutual Aid Round | Community pool for member emergency support; peer reciprocity. |
| Pre-sale | Early commitment to purchase before launch; escrow held until delivery. |
| Resource Campaign | Collective gathering of non-monetary resources: compute, licenses, materials. |

## **4.9 Portable Benefits**

kogi-bank manages portable benefits for independent workers — financial benefits that travel with the worker regardless of which clients or projects they work on. Every benefit account is a Portfolio Item in the Portfolio System.

| Benefit Category | Benefit Types |
| :---- | :---- |
| Health & Wellness | Health insurance, dental, vision, Health Savings Accounts (HSA) |
| Retirement & Savings | SEP-IRA, 401k portability, Pooled Employer Plans (PEP), emergency savings accounts |
| Paid Time Off & Income Security | Paid sick days, vacation time, income replacement for missed work |
| Insurance & Protection | Occupational accident insurance, disability, workers' compensation |
| Professional Development | Portable education accounts, training fund, certification credits |
| Platform Contributions | Gig platform % contributions (e.g. 4% of pre-tip earnings) to portable savings accounts; managed by benefit providers |

# **5\. Office System (kogi-office)**

## **5.1 Overview**

kogi-office is the portfolio item execution and management system. It provides the strategic management, decision management, task management, project management, and program management systems that independent workers use to execute their work. The Office integrates booking, CRM, contracts, logistics, and communication into a unified professional operations layer.

## **5.2 Work Management System (WMS)**

The Work Management System provides planning, execution, and coordination for all work items. It uses the Work Breakdown Structure model to ensure work is traceable from strategic vision to tactical execution.

| WMS Module | Primary Features |
| :---- | :---- |
| Work Dashboard | Active items summary, priority queue, velocity tracker, blocker alerts, upcoming milestones, team capacity, OKR progress, quick actions. |
| Backlogs | Program, project, sprint, team, and triage backlogs; MoSCoW and WSJF prioritization; dependency mapping; capacity planning. |
| Boards | Kanban, Agile, Resource, Priority, Program, and Custom boards; WIP limits; swimlane configurations; board policies. |
| Timelines | Schedule, Gantt, Calendar, Roadmap, Program Increment, Sprint, and Quarterly views; timebox model. |
| Work Analytics | Velocity forecasting, Monte Carlo simulations, cycle time, throughput, OKR progress, team performance dashboards. |
| Resource Management | Budget envelopes, resource allocation, delegation, capacity planning, TODO management. |
| Work CMS | Files, documents, contracts, agreements, SOPs, policies, procedures, frameworks, models — all versioned Portfolio Artifacts. |

## **5.3 Booking & Scheduling**

| Feature | Description |
| :---- | :---- |
| Centralized Calendars | Unified calendar aggregating bookings, events, and milestones across office and community. |
| Multi-Worker Dashboards | Overview of all member schedules, availability, and booking conflicts. |
| Conflict Detection | Automated scheduling conflict detection; Oba AI alerts before confirmation. |
| Event Booking & Reservations | Full lifecycle: inquiry → quote → confirm → deposit → delivery → close. |
| Resource Booking | Book equipment, studios, and venues alongside worker time. |
| Bookable Profiles | Workers and organizations can set profiles as bookable; client-facing booking widgets. |

## **5.4 CRM & Lead Management**

| Feature | Description |
| :---- | :---- |
| Lead Capture Forms | Embeddable forms on profile and portfolio pages; leads flow into CRM pipeline. |
| Client Database | Contact info, booking history, spend, communication log, tags. |
| Pipeline Management | Inquiry → Qualified → Quoted → Negotiating → Booked → Retained. |
| Automated Follow-ups | Triggered sequences on stage changes, quote expiry, post-event completion. |
| Lead Scoring | CRMEngine scores probability-to-book; Oba recommends outreach timing. |

## **5.5 Contracts & Invoicing**

| Feature | Description |
| :---- | :---- |
| Automated Contracts | Template contracts auto-populated from booking details; custom clauses supported. |
| E-Signatures | Integrated e-signature workflow via ProviderSystem (DocuSign, HelloSign). |
| Payment Reminders | Configurable deposit and balance due reminders via kogi-bank invoice system. |
| Recurring Billing | Subscription and retainer billing with automated recurring invoice generation. |

## **5.6 Logistics & Communication**

| Feature | Description |
| :---- | :---- |
| Tour / Project Itinerary | Multi-stop itinerary builder with timeline view; links to bookings and assignments. |
| Resource Allocation | Assign equipment, vehicles, and crew to dates and times; conflict detection. |
| Logistics Tracking | Equipment rentals, transportation, crew schedules — live status tracking. |
| In-App Communication | Dedicated logistics room per booking; broadcast to all crew; real-time updates. |
| Finance & Reporting | Real-time expense tracking vs. budget; P\&L per booking; tax report generation. |

# **6\. Marketplace & Exchange**

## **6.1 Overview**

The Kogi Marketplace and Exchange constitute the economic layer of the platform — the transactional, liquidity, and matching infrastructure that connects independent workers, collectives, cooperatives, and organizations for the purpose of trading labor, skills, assets, resources, financial instruments, and portfolio components.

*Design Principle: The Marketplace is where you offer what you have. The Exchange is where you trade what it's worth. Both are powered by the same Portfolio abstraction — every listing, bid, offer, and deal is ultimately a Portfolio Item with economic intent.*

## **6.2 Marketplace Categories**

| Category | Description |
| :---- | :---- |
| Work | Labor, skills, and services offered for hire. Gigs, contracts, jobs, tasks, consulting engagements. Linked to Resource (Human) Portfolio Items. |
| Assets & Artifacts | Purchasable or licensable portfolio outputs. Playbooks, templates, guides, tools, designs, software. Linked to Asset and Artifact Portfolio Items. |
| Resources | Allocatable resources offered to others. Compute, licenses, equipment, capital. Linked to Resource Portfolio Items. |
| Campaigns | Fundraising, crowdfunding, and group economics. Equity rounds, resource pools, community funds. Linked to Program and Project Portfolio Items. |

## **6.3 Exchange Markets**

| Market | What Is Traded |
| :---- | :---- |
| Labor Market (KLM) | Skills, gigs, contracts, work capacity, tasks — matching workers to opportunities via Deferred Acceptance algorithm. |
| Portfolio Commodities Market (KPCM) | Portfolio Items: assets, artifacts, projects, programs — listed, offered, dealt, and swapped. |
| Financial Instruments Market (KFIM) | Equity, revenue share notes, convertible notes, funds, tokens — order book, auctions, rounds. |
| Resource Market (KRM) | Compute, licenses, physical assets, capital, IP rights — spot trades, futures, leases. |
| Grants Marketplace | Government, foundation, platform, and community grants; discovery, application, and fund disbursement. |
| Microfinancing Market | Microloan origination, crowdlending, community lending pools, credit scoring via RiskEngine. |

## **6.4 Transaction Types**

| Transaction Entity | Description |
| :---- | :---- |
| Bid | A buyer's competitive price offer; remains open until matched, expired, or cancelled. |
| Offer / Ask | A seller's price or terms publication; posted-price or declining-price mechanisms. |
| Deal | A negotiated transaction in progress; multi-party; stage-gated; ZOPA-engine assisted. |
| Request for Proposal (RFP) | Buyer posts requirements; multiple sellers submit competing proposals. |
| Proposal | A structured pitch or response to a request; draft → submitted → decided. |
| Contract | Legally binding engagement agreement with milestone payment schedule and escrow. |
| Gig | Single fixed deliverable; fast-track matching → accept → escrow funded in one step. |
| Task | Sub-unit of a Contract or Gig; allocated within an existing engagement scope. |
| Swap | Direct peer-to-peer exchange of portfolio items, resources, or labor — no currency required. |
| Retainer | Ongoing access to worker time; monthly recurring invoice and auto-payment. |

## **6.5 Deal Room**

The Deal Room is the negotiation and coordination space that every Marketplace and Exchange transaction moves through. It is equipped with:

* Conversation thread — full message history between buyer and seller

* Proposal panel — active Proposal with version history and side-by-side comparison

* Document vault — attached files, contracts, DD reports, portfolio previews

* Milestone tracker — visual milestone board with status and escrow lock amounts

* Oba AI panel — drafts terms, compares proposals, flags risk patterns, computes ZOPA range

* Escrow status widget — live view of escrow balance and release conditions from kogi-bank

* Activity log — immutable deal event history

## **6.6 Due Diligence System**

For high-value deals in the Financial Instruments Market, kogi-exchange provides structured Due Diligence giving buyers access to governed, auditable portfolio intelligence before committing:

| DD Component | Description |
| :---- | :---- |
| Portfolio Health Report | kogi-engine PortfolioHealthScore: overall score, signal breakdown, risk flags. |
| Financial Summary | Revenue, expenses, budget utilization, cash flow signals from kogi-bank. |
| Audit Trail | Full immutable EventLog from the Portfolio System for the subject component. |
| Risk Assessment | kogi-engine RiskEngine: risk score, risk optimization plan, red flags. |
| Asset Valuation | AI-computed AssetValue model with comparable transactions. |
| Social Proof | Community engagement, follower and subscriber count, review summary, spread analytics. |

# **7\. Spaces & Community System**

## **7.1 Overview**

The Kogi Community, Spaces and Social System is the connective social tissue of the platform. It connects independent workers to one another through shared spaces, purposeful rooms, direct and group messaging, broadcast channels, event management, and a rich notifications infrastructure — all built natively on top of the Portfolio System.

*Design Philosophy: Every message, notification, room, and event is anchored to a PortfolioComponent. Social interactions generate signals that flow to kogi-engine and feed the recommendation, risk, and analytics engines.*

## **7.2 Space Types**

| Space Type | Description |
| :---- | :---- |
| Community | Open or invite-only community around a topic, interest, or discipline. |
| Team | Collaborative working group — squads, tribes, guilds, chapters. |
| Organization | Formal legal or operational entity (LLC, Corp, Trust, Fund). |
| Collective | Informal resource-sharing group; skill pool. |
| Cooperative | Member-owned economic cooperative with shared profits and voting rights. |
| DAO / Autonomous Org | Decentralized autonomous org with on-chain or platform-native governance. |
| Federation | Network of linked Spaces, Collectives, or Organizations. |
| Project Space | Space tied to a specific portfolio project or program. |
| Private Studio | Personal creative or research environment. |

## **7.3 Room Types**

| Room Type | Linked Entity |
| :---- | :---- |
| Direct (DM) | User ↔ User; unicast private messaging with end-to-end encryption option. |
| Group | Multi-user private channel; multicast up to 250 members. |
| Broadcast | One-to-many; only admins publish; subscribers read-only. |
| Space Room | Channel inside a Space: general, topic, project, announcements. |
| Project Room | Collaborative room tied to a specific portfolio project. |
| Deal Room | Negotiation space for marketplace deals and exchange proposals. |
| Event Room | Temporary room for an event or meeting; auto-archivable. |
| Governance Room | DAO and org voting and proposal discussion room. |
| AI Room | Chat interface with the Oba AI assistant. |

## **7.4 Messaging Delivery Model**

| Delivery Mode | Description |
| :---- | :---- |
| Unicast (1:1) | One sender → one recipient. Private, end-to-end directed. DM Rooms. |
| Multicast (1:N) | One sender → defined set of N recipients. Group and Project Rooms. |
| Broadcast (1:ALL) | One authorized sender → all channel subscribers. Subscribers read-only. Announcement channels, alert channels. |

## **7.5 Events Management**

| Event Type | Key Features |
| :---- | :---- |
| Meeting | Scheduled video/audio call; Zoom/Google Meet integration; recording, notes room. |
| Workshop | Interactive skill-building; materials library, exercise tracking, attendance certificate. |
| AMA (Ask Me Anything) | Open Q\&A; question queue, upvote, moderation, recording. |
| Demo Day | Portfolio showcase; presenter slots, viewer Q\&A, voting, portfolio links. |
| Governance Vote | Formal voting for an organization; linked to Proposal, quorum tracking, result execution. |
| Launch | Product/project launch event; portfolio item link, media kit, announcement broadcast. |
| Mastermind | Peer-to-peer knowledge sharing; round-robin structure, topic queue. |
| Conference | Multi-session gathering; multiple sessions, breakout rooms, speaker roles, recording. |

## **7.6 Notifications & Alerts**

| Delivery Channel | Priority Routing |
| :---- | :---- |
| In-app (WebSocket) | All priorities; instant delivery when user is online. |
| Notification Feed (KFED) | Normal and Low priority; persistent paginated inbox. |
| Mobile Push (FCM/APNs) | High and Critical priority; requires device token registration. |
| Email (SMTP) | Critical immediate; Normal as daily digest. |
| SMS (Twilio) | Critical only; opt-in; governance deadlines. |
| Third-party bridge | Slack, Discord, WhatsApp; per user preference via kogi-providers. |

## **7.7 Organization Governance**

| Governance Component | Description |
| :---- | :---- |
| Proposal | Motion to take action; title, body, linked action, voting window, required quorum. |
| Vote | Member vote: approve, reject, or abstain; weighted by stake or 1-member-1-vote. |
| Quorum | Minimum participation threshold; configurable per organization type. |
| Resolution | Outcome of a passed proposal; may trigger automated actions via Oba agent. |
| Treasury Proposal | Financial action requiring approval: budget allocation, investment, distribution. |
| Membership Proposal | Add, remove, or change role of a member. |
| Amendment | Proposal to modify org bylaws, policies, or governance rules. |

# **8\. Hub — Multi-Portfolio Governance**

## **8.1 Overview**

The Hub is the multi-portfolio governance management system — the layer of the platform that enables work portfolios to be shared, collaborated on, governed, and managed across organizational boundaries. It supports decentralized organization, voting, rights management, and federated coordination.

## **8.2 Shared Portfolios**

A Shared Portfolio is a Portfolio whose ComponentMetadata.owners\[\] field contains two or more distinct user or organization entity IDs. Shared portfolios support all existing lifecycle, governance, CRDT, and analytics operations, with extensions for multi-party coordination.

| Shared Portfolio Type | Governance Model |
| :---- | :---- |
| Team Portfolio | Admin-controlled; leader override capability. |
| Organization Portfolio | Multi-sig plus governance proposal required. |
| Collective Portfolio | Open contribution; light moderation. |
| Cooperative Portfolio | Full member vote required for all significant actions. |
| Federation Portfolio | Inter-org governance protocol; delegated authority. |
| Crowdresourced Portfolio | Open or community contributors; contribution ledger; steward review. |

## **8.3 Contribution Attribution Model**

All contributions to a shared portfolio component are tracked as ContributionRecord objects appended to the component's EventLog. Each record captures contributor ID, contribution type (Labor, Capital, Asset, Knowledge, Artifact, Code, Design, Data), contribution value, attribution weight, governance status, and optional linked kogi-bank ledger entry.

The attribution weight drives distribution calculations in the CollaborationEngine, which computes real-time attribution updates, detects merge conflicts, scores collaboration health, and generates Oba recommendations for steward review queues.

## **8.4 Portfolio Resource Sharing**

Portfolio Resource Sharing is implemented as a ResourceShare edge in the Graph system. Resources can be shared with spaces, organizations, and individual users with configurable access levels: Read, Fork, Contribute, or Manage. Shareable resource types include: Artifact, Template, Playbook, ToolBox, Contract, Design, Dataset, Notebook, Guidebook, and PlaybookSets.

## **8.5 Federation**

Federations are cross-organization meta-entities enabling joint governance, shared portfolio management, and inter-org resource exchange between aligned organizations. Federation members can:

* Maintain a Federation Reserve — shared capital pool funded by member contributions

* Execute inter-federate transfers — resource movement between member organizations

* Launch Federation Campaigns — cross-organizational fundraising toward a shared goal

* Access consolidated reporting — federation-level financial overview aggregating all member entities

* Use federated identity — members can act on behalf of the org in federated spaces

## **8.6 CrossGridLink Protocol**

The CrossGridLink protocol enables consent-gated data sharing between portfolios across the KLNK (Kogi Link Network). The six-phase lifecycle ensures full data sovereignty:

| Phase | Description |
| :---- | :---- |
| 1\. Request | Source Grid requests link to Target Grid; specifies edge type and desired mirrored attributes. |
| 2\. Configure | Both parties configure consent terms; worker specifies which attributes to mirror and whether write-back is permitted. |
| 3\. Provision | ShadowCell created in host Grid; initial snapshot sync of mirrored attribute values. |
| 4\. Sync | Ongoing real-time or batch synchronization of mirrored attribute changes. |
| 5\. Writeback | Optional: Grid A can write to Grid B on consented write-back attributes. |
| 6\. Lifecycle | Consent can be revoked at any time; revocation immediately halts shadow sync; cannot be silently re-accepted. |

# **9\. Agent & Automation System**

## **9.1 Overview**

The Agent module provides portfolio automations through the Oba AI assistant and a multi-agent orchestration system. Users can connect AI agents, configure work automations, and build intelligent pipelines that execute across their portfolio.

## **9.2 Oba AI Assistant**

Oba is the primary AI assistant for the Kogi platform, running on the Sambara agent system. Oba operates in multiple modes to provide contextual assistance across the entire platform:

| Oba Mode | Description |
| :---- | :---- |
| Reactive | Responds to direct user queries in natural language; NL-to-HyperQL translation. |
| Proactive | Monitors portfolio signals and surfaces recommendations, alerts, and briefings. |
| ColumnCompletion | Auto-completes spreadsheet-style fields based on context and patterns. |
| SmartFilter | Translates natural language filter requests into HyperQL predicates. |
| FormulaSuggestion | Recommends Tier-1 formula attributes based on user's analytical intent. |
| BatchAction | Prepares batch operations (payroll, escrow releases, bulk status updates) for one-click approval. |
| HealthBriefing | Daily morning briefing: portfolio health, financial summary, opportunities, risks, action items. |
| NarrativeGeneration | Auto-generates narrative summaries for portfolio items; powers grant applications and client-facing summaries. |
| Autonomous | Executes pre-approved rules without per-action confirmation; budget-gated automations. |

## **9.3 Connectable AI Agents**

The Agent module supports connection of multiple AI providers alongside the native Oba assistant:

* Oba (native Kogi AI)

* Claude (Anthropic)

* ChatGPT (OpenAI)

* Grok (xAI)

* DeepSeek

* Gemini (Google)

* Mistral

* Perplexity

* Llama (Meta)

* Qwen (Alibaba)

* Kimi.ai

* OpenRouter

* Custom / Self-hosted

## **9.4 Automation Architecture**

| Automation Level | Description |
| :---- | :---- |
| Orchestration | Multiple workflows coordinated into automated work pipelines; cross-module execution. |
| Workflows | Named sequences of actions triggered by events, schedules, or conditions. |
| Tasks | Individual automation steps within a workflow; typed actions on portfolio items. |
| Processes | Repeatable process templates instantiated from playbooks. |
| Work Pipelines | End-to-end automated pipelines from input signal to output action. |

## **9.5 kogi-engine Sub-Engines**

The intelligence layer powering all platform modules consists of specialized engines coordinated by kogi-engine (Scala 3):

| Engine | Primary Responsibility |
| :---- | :---- |
| AnalyticsEngine | Computes engagement, performance, usage analytics, and all quantitative signals. |
| TelemetryEngine | Real-time event streaming; component flow statistics; PlatformDataEnvelope processing. |
| RecommendationEngine | Collaborative and content-based filtering; personalized discovery; cold-start handling. |
| PersonalizationEngine | Persona lifecycle management; A/B experiments; UCB1 multi-armed bandit; content delivery config. |
| MatchEngine | Multi-dimensional worker-to-opportunity, investor-to-campaign, resource-to-project matching. |
| GraphEngine | Dependency graph, critical path analysis, betweenness centrality, community detection. |
| RiskEngine | PortfolioHealthScore; RiskOptimizationPlan; anomaly detection; counterparty risk scoring. |
| OptimizationEngine | Resource allocation optimization; capacity planning; bottleneck detection; flow efficiency. |
| SearchEngine | Full-text and semantic search across all component types; personalized re-ranking. |
| AllocationEngine | Deferred Acceptance algorithm for labor markets; capital allocation; attention markets. |
| IncentiveEngine | Reputation scoring; KogiPoints accounting; streak tracking; Shapley value distribution. |
| CollaborationEngine | Shared portfolio coordination; contribution attribution; conflict resolution; health scoring. |
| BenefitsEngine | Benefits eligibility scoring; coverage optimization; gap analysis; contribution forecasting. |
| GrantEngine | Grant matching; eligibility scoring; impact tracking; community grant pool management. |
| CRMEngine | Lead scoring; pipeline management; follow-up automation; retention analytics. |
| BookingEngine | Scheduling optimization; conflict detection; resource allocation; utilization analytics. |
| LogisticsEngine | Itinerary planning; resource routing; crew scheduling; expense tracking. |
| DataStreamingEngine | Real-time data export; BI connector feeds; event bus for third-party integrations. |

# **10\. Mechanism Design & Game System (kogi-game)**

## **10.1 Overview**

kogi-game is the mechanism design and game engine of the Kogi Platform — the mathematical and computational substrate that governs how resources, capital, labor, portfolio components, and financial instruments are discovered, matched, priced, allocated, exchanged, and incentivized.

*Design Principle: Mechanism design is the engineering of economic games. kogi-game designs rules that align individual incentives with platform-wide efficiency for every interaction — from a single gig bid to a multi-million dollar equity round.*

## **10.2 Game Types**

| Game Type | Mechanism |
| :---- | :---- |
| Assignment Game | Worker-to-gig matching via Deferred Acceptance (Gale-Shapley) with multi-dimensional scoring. |
| Double Auction | Exchange market: Continuous Double Auction with price-time priority order book. |
| Vickrey Auction (VCG) | Single-item high-value sales: second-price sealed-bid; incentive-compatible. |
| Reverse Auction / RFP | Score-based sealed-bid reverse auction with price+quality scoring function. |
| Negotiated Exchange | Alternating-offer Rubinstein bargaining with AI-assisted ZOPA computation. |
| Cooperative Game | Shapley Value computation for fair contribution-based distribution in collectives. |
| All-or-Nothing Campaign | Assurance contract / threshold public goods game; auto-refund on failure. |
| Reputation Game | Repeated game with Folk Theorem equilibrium; credible threat of reputation penalties. |

## **10.3 Reputation System**

Every user and organizational entity has a multi-dimensional ReputationScore computed as:

R(u) \= 0.30 × Delivery Reliability \+ 0.25 × Work Quality \+ 0.20 × Communication \+ 0.15 × Community Contribution \+ 0.10 × Financial Integrity

| Tier | Score Range | Benefits | Access |
| :---- | :---- | :---- | :---- |
| Elite | 90–100 | Featured placement × 1.5; fees −20%; early access to beta; priority dispute resolution | All marketplace, exchange, and campaign types |
| Trusted | 75–89 | Featured placement × 1.2; fees −10%; verified badge on all listings | All marketplace and exchange types |
| Established | 55–74 | Standard placement and fees; verified identity badge | All standard marketplace and exchange types |
| Developing | 35–54 | Standard placement and fees; no promoted listing access | Standard gigs, proposals, and campaigns only |
| New | 0–34 | Cold-start tier; guided onboarding; limited deal sizes; escrow mandatory | Simple gigs and low-value transactions only |

## **10.4 KogiPoints Incentive System**

KogiPoints (KP) are the platform's internal incentive currency — non-monetary credits earned through value-creating behaviors and redeemable against platform fees, promoted listing boosts, and premium features. KP is non-transferable between users and may not function as currency.

| Earning Event | KP Awarded |
| :---- | :---- |
| Complete a gig / contract (seller) | 50–500 KP (proportional to contract value) |
| Receive 5-star rating | 25 KP |
| Publish a portfolio component to marketplace | 75 KP |
| Successful referral (new user completes first deal) | 250 KP |
| Complete onboarding profile (skills \+ portfolio \+ availability) | 100 KP one-time |
| Participate in cooperative governance vote | 10 KP |
| Mentor a Newcomer (guided interaction) | 50 KP |

# **11\. Resource Management System (KOGI-RMS)**

## **11.1 Overview**

KOGI-RMS is the unified layer for discovering, organizing, allocating, governing, and tracking every resource on the Kogi platform. A resource is any entity — structured or unstructured, human or non-human, tangible or intangible — that has value, can be acted upon, and can be associated with a Portfolio component.

## **11.2 Resource Type Taxonomy**

| Domain | Resource Categories |
| :---- | :---- |
| Portfolio & Work | Portfolios, programs, projects, assets, artifacts, binders, journals, books, dossiers, folders, documents, files, directories, registries, archives |
| Planning & Time | Timelines, schedules, roadmaps, calendars, Gantt charts, timeboxes, milestones, roadblocks |
| Work Execution | Boards, epics, stories, features, tasks, work packages, WBS items, initiatives, strategies, themes, goals, OKRs |
| Commerce & Contracts | Gigs, bookings, consultations, jobs, contracts, offers, deals, requests, proposals, bids, investments, orders, invoices |
| Capital & Physical | Capital, financial assets, labor, land, estates, real estate, equipment, inventory, IP, data, credits |
| Financial Instruments | Equity, liquidity, debt, taxes, HSA, retirement, income protection, portable savings, revenue share |
| Contributions | Labor, skills, financial, support, advertising, marketing, promotions, endorsements, donations, investments |
| Users & Roles | Members, contributors, donors, investors, subscribers, followers, watchers, owners, editors, stewards |

## **11.3 Worker Types**

| Worker Type | Platform Capabilities |
| :---- | :---- |
| Contractor | Gig listings, project contracts, invoicing, bookings, multi-client CRM. 1099/self-employment tax; HSA eligible. |
| Consultant | Consultation packages, retainer billing, proposals, thought leadership. Full portable benefits suite. |
| Gig Worker | Gig claims, rapid booking, task queues, platform contribution tracking. Portable savings account. |
| Freelancer | Service listings, project portfolios, client management, milestone billing. |
| Entrepreneur | Company portfolios, equity crowdfunding, team management, cap table. Full equity and tax toolset. |
| Micropreneur | Solo product listings, subscriptions, microsite, community monetization. Simplified tax tracking. |
| Coach | Consultation booking, program creation, community spaces, certification. CRM \+ recurring billing. |
| Employee | Organization portfolio access, team boards, payroll via kogi-bank. |
| Officer | Org governance, signatory authority, multi-sig approvals, cap table management. |

# **12\. Technical Architecture**

## **12.1 Service Architecture**

| Service | Technology Stack |
| :---- | :---- |
| Portfolio System (core) | Rust — in-memory domain engine; PortfolioSystem struct; CRDT log; EventLog; federation |
| kogi-bank | Go (REST/gRPC API) \+ Rust (escrow state machine); PostgreSQL (SERIALIZABLE for ledger); Kafka |
| kogi-marketplace / kogi-exchange | Go (REST/GraphQL); Rust (core modules); PostgreSQL \+ Redis \+ Kafka |
| kogi-community / kogi-spaces | Go (10 services: community, room, message, notification, event, feed, social graph, post, match, analytics) |
| kogi-office / kogi-wms | Go (API \+ WBS engine); Scala 3 (planning analytics via kogi-engine) |
| kogi-engine (AI intelligence) | Scala 3 / JVM (all sub-engines); gRPC server port 9100; Kafka consumer; Akka Streams |
| kogi-game | Go (primary service) \+ Rust (core mechanism engine via FFI); Scala 3 sub-engines embedded in kogi-engine |
| Hypergrid substrate | Rust (cell store, CRDT merge engine, federation manager); PostgreSQL \+ Redis Cluster \+ Kafka \+ ClickHouse \+ Meilisearch |

## **12.2 Data Storage**

| Store | Usage |
| :---- | :---- |
| PostgreSQL | Primary persistence: all domain entities, ledger (SERIALIZABLE), CRDT log, EventLog, federation peers |
| Redis Cluster | Hot-path cache: cell cache (TTL 30s), order book state, active escrow, session deal context, presence |
| Apache Kafka | Event streaming: all domain events, AI feature streaming, federation delta sync, analytics pipeline |
| ClickHouse | Analytics: DimFold queries \> 500K rows; time-series aggregations; N=3/4 dimensional analysis |
| Meilisearch / Typesense | Full-text search: listing search, portfolio component search, user discovery |
| S3 / MinIO | Object storage: snapshots, archives, media files, event recordings, EventLog S3 archive |
| SQLite (kogi-local) | Local/desktop client cache: offline message queue, local feed snapshot, notification inbox cache |

## **12.3 Communication Protocols**

| Protocol | Usage |
| :---- | :---- |
| REST / GraphQL | External API surface for all consumer clients (web, mobile, third-party integrations) |
| gRPC (internal) | Service-to-service communication; kogi-engine EngineGrpcServer on port 9100 |
| WebSocket | Real-time delivery: Deal Room, room messages, presence indicators, live escrow status |
| Kafka (streaming) | Event bus, AI feature streaming, federation delta sync, analytics pipeline, audit log |

## **12.4 Client Architecture**

| Client | Technology |
| :---- | :---- |
| Web Client | Angular \+ TypeScript; WebSocket for real-time Deal Rooms and collaboration |
| Mobile (Android) | Kotlin; push notifications for transactions and deal events |
| Mobile (iOS) | Swift/iOS; push notifications via APNs |
| Desktop (kogi-local) | Electron or native; SQLite local cache; offline-capable |
| API / CLI | REST \+ gRPC; Bazel monorepo build system |

## **12.5 Security & Compliance**

* PCI DSS compliance — all card data handled by Stripe; kogi-bank stores no raw card numbers

* All transactions require explicit user approval — no AI-initiated fund movements without user confirmation

* Multi-factor authentication required for transactions above configurable thresholds

* Encryption at rest (AES-256) for all financial data; TLS 1.3 in transit

* Audit trail via immutable ledger \+ Portfolio System EventLog — every action logged with actor, timestamp, and result

* KYC / AML screening for accounts above transaction thresholds via third-party provider integration

* GDPR / CCPA right-to-be-forgotten — personal data anonymization on account deletion; ledger entries preserved for regulatory purposes

* Role-based access control — PermissionTier hierarchy governs all account, wallet, and portfolio operations

* Append-only ledger enforced at three layers: application code, PostgreSQL RLS, and periodic Merkle tree verification

# **13\. Platform Integrations**

## **13.1 Third-Party Payment Rails**

| Rail | Use Case |
| :---- | :---- |
| Stripe | Client card payments, marketplace payouts, subscription billing, identity verification |
| ACH (via Plaid) | Bank-to-bank transfers, payroll, external account linking |
| PayPal / Venmo | Small payments, contractor payouts, cross-platform transfers |
| Ethereum / ERC-20 | Token payments, crypto payouts, DeFi activity, on-chain escrow (Phase 4\) |
| Coinbase Commerce | Crypto invoice acceptance |
| SEPA | European bank transfers |
| Wire Transfer | Large international payments |

## **13.2 Work & Marketplace Integrations**

| Category | Platforms |
| :---- | :---- |
| Freelance Markets | Upwork, Fiverr, Toptal, Freelancer — profile import, listing sync, order bridging |
| Portfolio / Creative | Behance, Dribbble, GitHub, GitLab — portfolio import, project showcase sync |
| Job Markets | LinkedIn, Indeed, AngelList — job listing sync, profile export |
| Collaboration | Slack, Notion, Google Workspace, Microsoft Teams — project room linking, file sync |
| Commerce | Gumroad, Shopify, Stripe — asset sales routing, payment bridging |
| Investment / Trading | Robinhood, SoFi, Interactive Brokers — portfolio linking, position sync |
| Equity Platforms | AngelList, Republic, Wefunder — round sync, cap table bridge |
| Crypto / Web3 | Coinbase, Ethereum, Polygon, Solana — token bridge, wallet connect, NFT |

## **13.3 Social Platform Integrations**

| Platform | Capabilities |
| :---- | :---- |
| Facebook / Instagram | Cross-post content, import followers, sync events |
| YouTube | Embed video, sync channel activity, cross-publish; event recording upload |
| LinkedIn | Professional profile sync, job post distribution, network import |
| Twitter / X | Cross-post announcements, track mentions, import audience |
| Slack / Discord | Community server bridging, role sync, notification relay |
| WhatsApp | Broadcast messages, customer communication bridge, KNTF relay |
| Zoom / Google Meet | Event room video calls, meeting scheduling integration |
| Gmail | Email-to-room bridge, DM-to-email relay, notification email delivery |
| Twilio | SMS notification relay, 2FA, event reminders (critical alerts) |

# **14\. Platform Economics & Fee Design**

## **14.1 Fee Architecture**

Platform fees are designed to sustain operations and curate quality while not deterring participation. Fees scale with transaction value (not fixed), are lower for repeat participants (loyalty tiers), and a portion of fee revenue is recycled into liquidity incentives.

| Transaction Type | Fee Structure | Rationale | Loyalty Discount |
| :---- | :---- | :---- | :---- |
| Gig / Contract (seller) | 8% → 4% at Elite tier | Covers platform services \+ escrow; incentivizes quality | −1% per tier (8%/7%/6%/4%) |
| Gig / Contract (buyer) | 2% of contract value | Low buyer fee to maintain demand-side participation | −0.5% at Trusted+; waived at Elite |
| Marketplace listing sale | 6% of sale price | Platform facilitation \+ discovery services | −1.5% at Elite tier |
| Exchange trade | 0.5% of trade value (both sides) | Order book maintenance, settlement, anti-fraud | 50% reduction for KP redemption |
| Campaign (success fee) | 5% of successfully raised funds | Campaign hosting \+ disbursement services | −1% for cooperative entities |
| Campaign (equity) | 3% \+ 0.5% carried interest | Reflects higher-value financial facilitation | Fixed (regulatory reasons) |
| Subscription listing | 1% of recurring revenue | Ongoing platform services for subscription management | Waived at Elite tier |

## **14.2 Fee Redistribution**

| Pool | Allocation |
| :---- | :---- |
| Liquidity Incentive Pool | 15% of fee revenue — KP bonuses to participants in thin markets and new categories. |
| Newcomer Subsidy Pool | 5% of fee revenue — reduced fees and matching bonuses for users completing their first 3 transactions. |
| Community Fund | 5% of fee revenue — administered by platform governance council; grants for open community contributions. |
| Cooperative Bonus | 5% of fee revenue — additional fee discounts for formally incorporated cooperative entities. |

# **15\. Initiatives & Special Programs**

## **15.1 Overview**

Initiatives are coordinated platform-level programs that workers and organizations can enroll in to access collective benefits, capital, and resources. Initiatives integrate across the Dashboard, Bank, Community, and Hub modules.

## **15.2 Initiative Catalog**

| Initiative | Description |
| :---- | :---- |
| Portable Benefits | Coordinate worker groups for pooled benefits purchasing power and Pooled Employer Plan (PEP) enrollment. BenefitsEngine provides eligibility scoring, coverage optimization, and gap analysis. |
| Grants & Microfinancing | Grant discovery, application, community lending pools, microloan origination. GrantEngine scores grant matches by eligibility, portfolio strength, and historical award data. |
| Group Economics | Cooperative treasury, revenue sharing, group investment pools, mutual aid funds. Revenue distribution engine computes Shapley Value-based patronage distributions. |
| Equity Crowdfunding | Reg CF/A+ campaigns, revenue share, SAFE notes, cap table management. CrowdfundingEngine scores campaigns, matches investors, models equity, and checks compliance. |
| Crowdresourcing | Community-sourced labor, capital, asset, and knowledge contributions with full attribution tracking and configurable reward rules. |
| Shared Portfolios | Multi-user, multi-org portfolio management with contribution attribution and governance. CollaborationEngine coordinates concurrent edits and health scoring. |
| Portfolio Resource Sharing | Share artifacts, templates, tools, playbooks, and data within community contexts. ResourceShare graph edges with configurable access levels. |

# **16\. Open Items & Roadmap**

## **16.1 Near-Term Open Items**

| Item | Description |
| :---- | :---- |
| Persistence Layer (Portfolio System) | Portfolio System is currently in-memory. Implement pluggable PortfolioStore backend with PostgreSQL \+ Redis \+ S3 implementations. |
| CRDT Status Field Merge | apply\_crdt\_op() currently skips status mutations. Implement Lattice CRDT for lifecycle status fields before multi-node deployments. |
| Ledger Snapshot Optimization | Balance recomputation from full ledger history will slow for long-lived accounts. Implement periodic balance snapshots with verified snapshot \+ incremental replay. |
| Saga Compensation Protocol | Distributed transactions spanning kogi-bank, kogi-exchange, and kogi-portfolio need fully specified Saga compensation maps for escrow, deal, and campaign settlement. |
| KYC / AML Integration | Select and integrate a third-party KYC/AML provider (Stripe Identity, Persona, Onfido) via kogi-providers for accounts above defined thresholds. |
| AI Finance Agent Writeback Protocol | Define typed writeback protocol for Finance Agent to create Invoice drafts, schedule transactions, and append financial governance notes without bypassing PermissionTier checks. |
| Portable Benefits Provider Integration | Define ProviderSystem integration pattern for benefit account management with third-party retirement, health, and insurance providers. |
| SearchEngine Warm-up | Wire on\_component\_created / on\_component\_updated plugin hooks to populate Meilisearch listing index. |
| Combinatorial Auction Solver | Implement LP relaxation \+ branch-and-bound for small N; greedy approximation for large N; bid complexity limit for tractability. |

## **16.2 Phase 4 Considerations**

* On-chain escrow smart contracts — self-executing escrow for crypto/token deals without custodial counterparty risk

* Decentralized finance (DeFi) integration — yield on idle campaign and reserve accounts via DeFi protocols

* Autonomous financial agent — Oba acts on pre-approved rules: auto-pay recurring bills, auto-invest above balance threshold

* Cross-platform marketplace federation — federated listing discovery across ShangoOS platforms (Kogi, Ume, Qala)

* Real-world asset (RWA) tokenization — platform infrastructure to tokenize and trade physical assets on-chain

* Cross-platform reputation portability — export Kogi reputation signals to external platforms via verifiable credential standard

* Prediction markets — workers and investors bet on project outcomes; outcomes calibrate RiskEngine priors

* AI-autonomous market making — Oba-class agents post algorithmic bids/offers to thin markets

# **Appendix A: Glossary**

| Term | Definition |
| :---- | :---- |
| Portfolio | The root domain concept of Kogi — the complete, unified record of an independent worker's professional and economic existence. The master hyperspreadsheet IS the Portfolio. |
| HyperRow | A row in a Hypercube; the basic entity unit in the Hypergrid substrate. |
| Hypercube | An N-dimensional data store whose primary axes are entity (D₁) and property (D₂), with optional additional axes for time, jurisdiction, or segment. |
| CRDT | Conflict-free Replicated Data Type — distributed data structure enabling safe concurrent edits across portfolio nodes and federated deployments. |
| EventLog | Append-only, immutable audit history attached to every Portfolio Component recording all mutations with actor, timestamp, and delta. |
| VectorClock | Per-component distributed logical clock used for causal ordering of concurrent mutations across federation nodes. |
| KLNK | Kogi Link Network — the economic graph of interconnected worker portfolios via CrossGridLink edges. |
| CrossGridLink | A consent-gated graph edge connecting entities across different Portfolio Systems (Grids); enables attribute mirroring with worker data sovereignty. |
| ShadowCell | A mirrored attribute value in Grid A that reflects a consented attribute from Grid B, synced via the CrossGridLink protocol. |
| Zawadi | The basic unit of transaction on the Kogi platform; translates to/from liquidity, equity, securities, credits, debts, and digital currency. |
| Sundiata Coin | The principal cryptocurrency for the Kogi platform on the Sundiata distributed ledger. |
| Oba | The Kogi platform AI assistant; surfaces recommendations, analytics summaries, governance alerts, and optimization suggestions across all modules. |
| Linktree | A user's 'personal work IP address/router' — the interface for all external communications, accounts, integration toolchains, and RBAC connections. |
| KogiPoints (KP) | Non-monetary platform incentive credits earned through value-creating behaviors; redeemable against platform fees and listing boosts. |
| Portable Benefits | Worker-centered benefits that remain with the individual rather than being tied to a single employer or gig platform. |
| Cooperative Treasury | A multi-sig controlled shared fund for a cooperative's operations, payroll, and investment; governed by democratic member vote. |
| Shapley Value | The unique fair allocation that assigns each cooperative member their marginal contribution averaged over all possible coalition orderings. |
| ZOPA | Zone of Possible Agreement — the range of prices/terms where both parties' reservation values overlap; computed by the Deal Room ZOPA Engine. |
| Timebox | A fixed time period bounding committed work items: sprint, PI, quarter, or custom duration. |
| Persona | An interest and identity classification assigned to a user that shapes recommendation, matching, UI configuration, and community cohesion. |

