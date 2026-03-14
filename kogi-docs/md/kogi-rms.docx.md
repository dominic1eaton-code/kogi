  
**KOGI**

Independent Worker Operating System

**Resource Management System**

System Design Document

v1.0  ·  Kogi Platform  ·  March 2026

# **1\. Overview**

The Kogi Resource Management System (KOGI-RMS) is the unified layer for discovering, organizing, allocating, governing, and tracking every resource on the Kogi platform. A resource is any entity — structured or unstructured, human or non-human, tangible or intangible — that has value, can be acted upon, and can be associated with a Portfolio component.

KOGI-RMS is not a separate application. It is the resource substrate that underlies every other Kogi module: Portfolio, Work Management, Bank, Marketplace, Exchange, Community, and Studio. Every item described in this document is a first-class Resource object with a common metadata envelope, typed schema, lifecycle state machine, access control, and analytics hooks.

| Domain | Resource Categories Covered |
| :---- | :---- |
| **Portfolio & Work** | Portfolios, programs, projects, assets, artifacts, binders, journals, books, dossiers, folders, documents, files, directories |
| **Planning & Time** | Timelines, schedules, roadmaps, calendars, Gantt charts |
| **Work Execution** | Boards, epics, stories, features, tasks, work packages, WBS items, initiatives, strategies, themes |
| **Commerce & Contracts** | Gigs, bookings, consultations, jobs, contracts, offers, deals, requests, proposals, bids, investments |
| **Capital & Physical** | Capital, financial assets, artifacts, labor, land, estates, real estate |
| **Finance & Instruments** | Equity, liquidity, debt, taxes, cash, credit, debit, donations, grants |
| **Contributions** | Labor, skills, financial, support, advertising, marketing, promotions, endorsements, donations, investments |
| **Users & Roles** | Members, contributors, donors, investors, subscribers, followers, watchers, owners, editors |
| **Personas** | Developers, creatives, artists, professionals, architects, organizers, activists, technologists, and more |
| **Workers** | Contractors, consultants, gig workers, freelancers, entrepreneurs, coaches, employees, officers |
| **Organizations** | Autonomous orgs, open source communities, cooperatives, collectives, federations, councils, assemblies |

# **2\. Core Resource Data Model**

Every resource on the Kogi platform is an instance of the base Resource object. Typed resources extend this base with domain-specific fields. All resources are Portfolio Components and inherit CRDT versioning, EventLog auditing, and PolicySet governance.

## **2.1 Resource Base Schema**

| Field | Type | Description |
| :---- | :---- | :---- |
| resource\_id | UUID | Platform-wide unique identifier; immutable |
| resource\_type | ResourceType | Top-level classification — see Resource Type Taxonomy (§3) |
| resource\_subtype | String | Domain-specific subtype (e.g., Portfolio, Gig, HSA, Land) |
| name | String | Display name |
| slug | String | URL-safe unique handle |
| description | String | Rich-text description; markdown supported |
| status | ResourceStatus | Draft | Active | Paused | Archived | Deleted |
| visibility | Visibility | Public | Private | Protected | Space | Organization | Invite |
| owners | Vec\<EntityId\> | One or more users or orgs with Owner privilege |
| editors | Vec\<EntityId\> | Users or orgs with Edit privilege |
| contributors | Vec\<EntityId\> | Users or orgs with Contribute privilege |
| viewers | Vec\<EntityId\> | Users or orgs with View privilege |
| tags | Vec\<String\> | Hashtag-style discovery tags; full-text indexed |
| labels | Vec\<String\> | Structured classification labels; filterable |
| categories | Vec\<CategoryEnum\> | Domain category set (Product, Finance, Creative, Ops, etc.) |
| policy\_ids | Vec\<PolicyId\> | Governance policies; inherited from parent unless overridden |
| parent\_ids | Vec\<ComponentId\> | Parent resources in hierarchy |
| children\_ids | Vec\<ComponentId\> | Child resources in hierarchy |
| linked\_ids | Vec\<ComponentId\> | Lateral links to related resources |
| attachments | Vec\<ArtifactId\> | Attached files, documents, and CMS artifacts |
| metadata | Map\<String, Value\> | Extensible key-value metadata bag |
| created\_at | DateTime | ISO 8601 creation timestamp; immutable |
| updated\_at | DateTime | Last mutation timestamp; auto-managed |
| vector\_clock | VectorClock | CRDT conflict resolution; auto-managed by platform |
| event\_log | Vec\<EventRecord\> | Append-only audit history of all mutations |
| analytics\_id | AnalyticsRef | Pointer to AnalyticsEngine time-series for this resource |

## **2.2 Resource Actions (Universal)**

All resources support the following action surface regardless of type. Additional type-specific actions are defined in each domain section.

| Action Group | Actions |
| :---- | :---- |
| **Discovery** | search · filter · index · tag · label · mention · hashtag · topic · recommend · discover · explore |
| **Social** | like · comment · share · follow · subscribe · watch · save · bookmark · poll · survey · invite |
| **Content** | view · preview · download · fork · clone · embed · export · print · archive |
| **Governance** | own · edit · CRUD · post · report · flag · approve · reject · propose · vote |
| **Commerce** | donate · invest · fund · bid · offer · deal · trade · allocate · contribute |
| **Lifecycle** | create · draft · publish · pause · unpublish · delete · restore · version · snapshot |
| **Collaboration** | assign · delegate · request · respond · notify · alert · broadcast · message |

# **3\. Resource Type Taxonomy**

Resources are organized into eight top-level domains. Each domain has a primary type enum and a subtype registry. New subtypes can be registered at runtime without schema migration.

## **3.1 Portfolio & Work Resources**

| Domain: PORTFOLIO\_WORK  —  Structured knowledge and work artifacts |
| :---- |

| Resource | Subtype | Description |
| :---- | :---- | :---- |
| Portfolio | Personal | Work | Professional | Collective | Public | Custom | Top-level container organizing all of a user's or org's work, assets, and goals |
| Program | Standard | Agile | Initiative | Cooperative | Federation | Multi-project coordinated effort aligned to a strategic objective |
| Project | Standard | Sprint | Campaign | Research | Event | Time-boxed effort producing a defined set of deliverables |
| Asset | Digital | Physical | Financial | IP | Brand | Data | Anything of value owned or controlled by a user or org |
| Artifact | Document | Design | Code | Media | Report | Export | Template | A discrete, versioned output produced during work |
| Binder | Collection | Curated | Smart | Shared | Logical grouping of related resources; ordered or tagged |
| Journal | Work | Personal | Research | Decision | Project | Chronological record of activities, decisions, or observations |
| Book | Notebook | Playbook | Guidebook | Contactbook | Schedulebook | Planbook | Itembook | Structured document containers — see Book Type Registry |
| Dossier | Person | Organization | Topic | Deal | Case | Research-oriented profile package for a subject |
| Folder | Standard | Smart | Shared | Archive | Hierarchical file system container; maps to filesystem path |
| Document | Brief | Report | Memo | SOP | Policy | Agreement | Contract | Framework | Model | Rich-text structured document; versioned; e-signature capable |
| File | Image | Video | Audio | PDF | Spreadsheet | Archive | Binary | Raw binary or media file; attached to parent portfolio items |
| Directory | Workspace | Project | Space | Organization | Spatial collection of resources; navigable; permissioned |
| Registry | Asset | Skill | Grant | Contact | Vendor | Member | Catalogued, searchable, governed registry of typed resources |
| Archive | Portfolio | Project | Program | Space | Deep storage; full restore; read-only by default |

### **Book Type Registry**

| Book Type | Purpose & Contents |
| :---- | :---- |
| **book:notebook** | Freeform notes, annotations, ideas; linked to any portfolio item |
| **book:playbook** | Repeatable process playbooks; steps, roles, checklists, templates |
| **book:guidebook** | Documentation set; structured sections; publishable as external docs site |
| **book:contactbook** | Structured contact directory; linked CRM profiles and relationship metadata |
| **book:schedulebook** | Scheduling templates, availability rules, booking patterns |
| **book:planbook** | Planning artifact; phase gates, milestones, resource plans, risk registers |
| **book:itembook** | Catalogued collection; charter · catalogue · library · templates · logs |

## **3.2 Planning & Time Resources**

| Domain: PLANNING\_TIME  —  Time-axis visualization and coordination |
| :---- |

| Resource | Subtypes | Description |
| :---- | :---- | :---- |
| Timeline | Project | Program | Portfolio | Personal | Public | Chronological view of milestones, events, and item states across any time range |
| Schedule | Work | Booking | Crew | Equipment | Personal | Recurring | Specific time-slot assignments for workers, resources, or events; conflict-aware |
| Roadmap | Product | Platform | Organization | Community | Release | Strategic milestone map; external-publishable; investor-ready view |
| Calendar | Work | Personal | Team | Organization | Public | Booking | Day/week/month event and deadline calendar; unified across modules |
| Gantt | Project | Program | Portfolio | Campaign | Dependency-aware bar chart; critical path detection; resource allocation overlay |
| Timebox | Sprint | PI | Quarter | Custom | Duration | Bounded planning period committing a set of work items; parent-child hierarchy |
| Roadblock | Dependency | Blocker | Risk | Constraint | Structured impediment record linked to timeline items; escalation workflow |
| Milestone | Release | Phase | Goal | Event | Deliverable | A significant checkpoint or decision point; governance gate optional |

## **3.3 Work Execution Resources**

| Domain: WORK\_EXECUTION  —  Work breakdown and agile delivery items |
| :---- |

| Resource | Level / Scope | Description |
| :---- | :---- | :---- |
| Theme | Portfolio level | Broadest strategic investment area; parent of multiple initiatives |
| Strategy | Portfolio / Org level | Direction-setting resource; defines approach, principles, and trade-offs |
| Initiative | Program level | Coordinated set of epics or projects advancing a theme or strategy |
| Epic | Program / PI level | Large capability; decomposed into stories; fits a PI or quarter |
| Feature | Sprint / Epic level | User-facing capability delivering discrete value; subset of an epic |
| Story | Sprint level | Primary delivery unit; estimable, testable, completable in a sprint |
| Task | Daily level | Atomic execution step within a story; hours or sub-day scope |
| Work Package | WBS node | Any addressable WBS node carrying story.data and story.type |
| Board | Project / Team view | Kanban, agile, resource, program, or custom visual management surface |
| Backlog | Project / Program | Ordered, prioritized list of uncommitted work items |
| Tactic | Operational level | Specific action executing a strategy within a timebox |
| Operation | Recurring / Ops | Ongoing or recurring operational work: support, maintenance, monitoring |
| Goal | OKR / Planning | Desired outcome; parent of key results or epics |
| Objective | OKR level | SAFe / OKR objective: aspirational quarterly or PI-level direction |
| Key Result | OKR level | Measurable outcome proving an Objective is being achieved |
| Risk | Any level | Identified risk with probability, impact, owner, and mitigation plan |
| Dependency | Any level | Explicit dependency link between two work resources; tracks resolution |
| Release | Program / Project | Versioned bundle of features and stories delivered to users |
| Deployment | Ops / Engineering | A specific deployment event; environment, version, rollout strategy |

## **3.4 Commerce & Contract Resources**

| Domain: COMMERCE\_CONTRACTS  —  Work sourcing, deal flow, and agreements |
| :---- |

| Resource | Subtypes | Description |
| :---- | :---- | :---- |
| Gig | Task | Project | Retainer | One-time | Recurring | A discrete, scoped unit of work listed on the Marketplace; claimable by qualified workers |
| Consultation | Single | Package | Retainer | Group | Workshop | Advisory engagement sold by a worker or org; time-bounded; bookable |
| Booking | Service | Event | Studio | Venue | Equipment | Team | Confirmed time reservation linking a client, worker, and resource package |
| Job | Contract | Part-time | Full-time | Project | Internship | Employment or contractor engagement with defined terms and compensation |
| Contract | Service | Employment | NDA | License | Partnership | Revenue-Share | Legally binding agreement; e-signature workflow; version-controlled |
| Agreement | MoU | LOI | SLA | Partnership | Confidentiality | Grant | Formal non-binding or binding understanding; structured template; versioned |
| Offer | Service | Product | Bundle | Subscription | Custom | A published value proposition: price, deliverables, terms, and duration |
| Deal | Exchange | Investment | Acquisition | Revenue-Share | Barter | A negotiated transaction in progress; multi-party; stage-gated |
| Request | Work | Resource | Collaboration | Funding | Information | An inbound ask from a user or org requiring a response or action |
| Proposal | Business | Project | Partnership | Grant | Creative Brief | A structured pitch or response to a request; draft → submitted → decided |
| Bid | Service | Asset | Auction | Reverse-Auction | A competitive price offer in a marketplace or auction context |
| Investment | Equity | Revenue-Share | Convertible | Donation | Grant | A committed financial contribution in exchange for return, equity, or impact |
| Order | Product | Service | Subscription | Custom | A confirmed purchase or engagement; triggers fulfillment workflow |
| Invoice | Service | Deposit | Recurring | Final | Credit | A financial demand document linked to a booking, contract, or order |
| Quote | Itemized | Estimate | Rough Order of Magnitude (ROM) | A non-binding price and scope estimate; precursor to contract or order |

## **3.5 Capital & Physical Resources**

| Domain: CAPITAL\_PHYSICAL  —  Tangible and financial assets under management |
| :---- |

| Resource | Subtypes | Description |
| :---- | :---- | :---- |
| Capital | Working | Fixed | Investment | Reserve | Operating | Financial resources available for deployment; tracked in kogi-bank accounts |
| Asset | Digital | Physical | IP | Brand | Equipment | Vehicle | Inventory | Any item of value owned or leased; depreciation-tracked; insurable |
| Artifact | Design | Code | Media | Data | Document | Model | Report | Produced output; versioned; with provenance and attribution chain |
| Labor | Contracted | Employed | Volunteer | Contributed | Gig | Human work capacity; costed; allocated; tracked against budget and timeboxes |
| Land | Residential | Commercial | Agricultural | Mixed-Use | Real property parcel; title-tracked; linked to legal entity |
| Estate | Residential | Commercial | Portfolio | Trust | A portfolio of real property assets under unified management |
| Real Estate | Property | Development | REIT | Fractional | Real estate holdings including fractional ownership and development projects |
| Equipment | Owned | Leased | Shared | Rented | Tools | Technology | Physical equipment; rental lifecycle; maintenance schedule; cost ledgering |
| Inventory | Goods | Materials | Supplies | Products | Physical stock items tracked by quantity, location, and valuation |
| IP | Patent | Trademark | Copyright | Trade Secret | License | Intellectual property; registered or unregistered; licensed; revenue-generating |
| Data | Dataset | Model | Analytics Output | Research | Intelligence | Proprietary data assets; licensed; access-controlled; monetizable |
| Credit | Platform | Gig Platform | Reward | Training | Compute | Non-monetary credits redeemable within the Kogi ecosystem or partner platforms |

## **3.6 Financial Instruments & Accounts**

| Domain: FINANCIAL\_INSTRUMENTS  —  Equity, debt, cash, and alternative instruments |
| :---- |

| Instrument / Account | Subtypes | Description |
| :---- | :---- | :---- |
| Equity | Common | Preferred | Cooperative | Revenue-Share | SAFEs | Ownership stake in an entity or project; cap table tracked in kogi-bank |
| Liquidity | Cash | Stablecoin | Credits | Redeemable Balance | Immediately deployable value; wallet and account balances |
| Debt | Microloan | Line of Credit | Convertible Note | Invoice Financing | Borrowed capital; repayment schedule; interest; origination via kogi-bank |
| Taxes | Self-Employment | Quarterly | Sales | Withholding | Tax obligations and estimated payments tracked against income and expenses |
| Cash | Operating | Reserve | Escrow | Payroll | Campaign | Physical or digital cash positions; journal and ledger entries |
| Credit | Platform | Bank | Vendor | Community Lending | Available credit lines and facilities; utilization and limit tracking |
| Debit | Wallet | Account | Prepaid | Community | Debit instruments and spend accounts; transaction history |
| Donation | One-time | Recurring | Matching | In-Kind | Anonymous | Non-expectation-of-return transfer; tracked for tax deductibility where applicable |
| Grant | Government | Foundation | Platform | Community Pool | Award-based funding; milestone disbursement; impact reporting required |
| HSA | Health Savings Account | Pre-tax health expense fund; portable; rolls over annually |
| Retirement | SEP-IRA | 401k | PEP | Pooled Employer Plan | Tax-advantaged retirement savings; contribution tracking; vesting schedule |
| Income Protection | Disability | Sick Pay | Occupational Accident | Insurance-backed income replacement for missed work or injury |
| Portable Savings | Gig Contribution | Platform % | Emergency Savings | Worker-portable benefit savings; funded by platform contributions |
| Revenue Share | Cooperative | Investor | Platform | Partner | Formula-based periodic revenue distribution; automated via kogi-bank engine |

## **3.7 Contributions**

| Domain: CONTRIBUTIONS  —  All forms of value contributed to the platform or community |
| :---- |

A Contribution is a tracked, attributed, and potentially rewarded transfer of value from a contributor to a resource, project, community, or organization. All contributions generate a ContributionRecord appended to the target resource's EventLog.

| Contribution Type | Sub-Types | Description & Attribution |
| :---- | :---- | :---- |
| Labor | Hours | Tasks | Deliverables | Expertise | On-Call | Work performed; tracked by hours or story points; attribution weight computed by CollaborationEngine |
| Skills | Technical | Creative | Strategic | Administrative | Domain | Specialized capability contributed; recorded in contributor's skill portfolio |
| Financial | Cash | Crypto | Credit | Platform Currency | Monetary contribution to a project, fund, or organization; ledgered in kogi-bank |
| Support | Customer Support | Community Moderation | Mentorship | Coaching | Non-billable support activities; tracked for community health metrics |
| Advertising | Paid Ad | Sponsored Content | Cross-Promotion | Listing Boost | Paid exposure contributions; campaign-linked; ROI tracked |
| Marketing | Content Creation | SEO | Email | Social | Copywriting | Marketing labor and materials contributed; campaign-attributed |
| Promotions | Discount | Offer | Bundle | Affiliate | Referral | Value-exchange promotional contributions; tracked for conversion analytics |
| Endorsements | Testimonial | Badge | Certification | Review | Recommendation | Trust signal contributions; linked to contributor identity and credential |
| Donation | One-time | Recurring | In-Kind | Matched | Anonymous | No-return-expected transfer; tax receipt available; impact tracked |
| Investment | Equity | Revenue-Share | Convertible | SAFE | Loan | Return-expected capital deployment; cap table and ledger entries created |

### **Contribution Attribution Model**

| Field | Description |
| :---- | :---- |
| **contributor\_id** | User, org, or collective making the contribution |
| **contribution\_type** | Labor | Capital | Asset | Knowledge | Artifact | Marketing | Endorsement |
| **contribution\_value** | { amount, unit, valuation\_method } — monetary or effort-unit value |
| **attribution\_weight** | f64 — relative share used in distribution calculations; computed by CollaborationEngine |
| **governance\_status** | PendingReview | Accepted | Rejected | Merged — configurable review gate |
| **linked\_ledger\_entry** | Option\<LedgerId\> — kogi-bank ledger entry for capital contributions |
| **reward\_rules** | Vec\<RewardRule\> — equity allocation | revenue share | direct payment | badge | credits |
| **timestamp** | ISO 8601 contribution event time; immutable |

# **4\. Users, Roles & Personas**

The Kogi platform models human participants as a layered identity system: a base User entity with one or more Profiles, each Profile carrying typed Roles, class-based Personas, and worker-type classifications.

## **4.1 User Roles**

User roles govern access, action privileges, and notification routing within any resource context. Roles are scoped per resource instance and can be inherited from parent containers.

| Role | Access Level | Capabilities |
| :---- | :---- | :---- |
| Owner | Full control | CRUD, governance, policy management, role assignment, deletion, transfer |
| Editor | Read \+ Write | Create, update, publish, manage content; cannot delete or transfer ownership |
| Contributor | Read \+ Contribute | Submit contributions; create linked sub-items; cannot edit parent resource |
| Viewer | Read only | View and comment only; no modification rights |
| Subscriber | Notification access | Receives updates and alerts on resource changes; no edit rights |
| Follower | Social access | Social follow; activity feed updates; public content access |
| Watcher | Activity monitoring | Receives all state-change notifications; used for oversight and review |
| Investor | Financial \+ View | Access to financial performance data, distributions, and cap table position |
| Donor | Social \+ Receipt | Donation confirmation, impact tracking, tax receipt access |
| Steward | Governance role | Reviews and approves contributions, governance proposals, and compliance items |
| Reviewer | Review gate | Required approval step on configured governance transitions |
| Moderator | Community control | Content moderation, member management within a space or org |

## **4.2 Personas**

Personas are interest and identity classifications that inform personalization, recommendation, matching, and community cohesion. A user may carry multiple personas simultaneously. Persona data shapes the PersonalizationEngine and MatchEngine outputs.

| Persona | Cluster | Primary Resource Affinity | Platform Engagement Pattern |
| :---- | :---- | :---- | :---- |
| Developer | Technical | Code, APIs, toolchains, technical docs | Build integrations; contribute code; use Exchange for technical services |
| Creative | Creative | Portfolio, media, design assets, studios | Publish work; monetize via Marketplace; join creative collectives |
| Artist | Creative | Portfolio, bookings, media files, fan community | Manage touring, merch, licensing; use booking \+ logistics |
| Writer | Creative | Documents, publications, content campaigns | Publish articles; manage clients via CRM; sell content licenses |
| Journalist | Creative | Research dossiers, publications, sources | Track stories as projects; manage sources; publish to community feed |
| Professional | Professional | Credentials, client CRM, proposals, invoices | Manage service business; CRM \+ booking \+ invoicing |
| Enthusiast | Consumer | Community feeds, events, reviews, resources | Discover, follow, donate, and participate in community initiatives |
| Hobbyist | Consumer | Personal portfolio, learning resources, community | Share projects; join interest spaces; access learning resources |
| Service Provider | Commerce | Bookings, gigs, offers, contracts, reviews | List services; manage bookings; earn via Marketplace |
| Visionary | Strategic | Strategy docs, roadmaps, initiatives, proposals | Author strategies; rally contributors; lead initiatives |
| Architect | Technical | System designs, frameworks, models, documentation | Design systems; contribute frameworks; advise programs |
| Designer | Creative | Design files, UI components, brand assets | Create and sell design assets; collaborate on product teams |
| Facilitator | Operations | Agendas, playbooks, meeting records, retrospectives | Run collaborative sessions; document outcomes; manage group workflows |
| Integrator | Technical | APIs, connectors, toolchains, automations | Build integrations; publish tools to Marketplace; consult on automation |
| Organizer | Community | Events, spaces, initiatives, membership | Coordinate community activities; manage organizations and collectives |
| Activist | Community | Initiatives, campaigns, grant pools, petitions | Drive social initiatives; coordinate crowdresourcing; manage advocacy campaigns |
| Manager | Operations | Projects, teams, boards, resource allocations | Oversee delivery; manage backlogs; allocate resources; report on KPIs |
| Director | Strategic | Programs, OKRs, portfolios, financials | Set strategy; govern programs; manage stakeholders and budgets |
| Insider | Networked | CRM contacts, deals, intel, exclusive resources | Deal sourcing; relationship management; access gated community content |
| Hacker | Technical | Code, exploits (ethical), technical research | Bug bounty; security research; technical deep-dives |
| Technician | Technical | Equipment, SOPs, maintenance logs, work orders | Manage physical assets; log maintenance; fulfill service work orders |
| Innovator | Strategic | R\&D projects, prototypes, patents, experiments | Run innovation sprints; document experiments; file IP records |
| Technologist | Technical | Emerging tech, research, tools, frameworks | Research and publish insights; advise on technology adoption |

## **4.3 Worker Types**

Worker type classifications determine platform feature access, contract templates, tax treatment, benefit eligibility, and Marketplace listing capabilities. A user may hold multiple worker type designations simultaneously.

| Worker Type | Platform Capabilities | Benefit & Tax Context |
| :---- | :---- | :---- |
| Contractor | Gig listings, project contracts, invoicing, bookings, multi-client CRM | 1099 / self-employment; HSA eligible; SEP-IRA eligible; portable benefits |
| Consultant | Consultation packages, retainer billing, proposals, thought leadership | 1099 / LLC; full portable benefits suite; income protection eligible |
| Gig Worker | Gig claims, rapid booking, task queues, platform contribution tracking | Portable savings account; gig platform contribution % tracking; income protection |
| Freelancer | Service listings, project portfolios, client management, milestone billing | 1099; HSA; portable savings; professional development account |
| Entrepreneur | Company portfolios, equity crowdfunding, team management, cap table | Self-employed; full equity and tax toolset; PEP eligible |
| Micropreneur | Solo product listings, subscriptions, microsite, community monetization | Simplified tax tracking; portable benefits; emergency savings |
| Coach | Consultation booking, program creation, community spaces, certification | Service business tools; CRM; recurring billing; professional development |
| Partner | Co-ownership of portfolios, revenue sharing, co-branding | Revenue share tracking; cooperative treasury participation |
| Employee | Organization portfolio access, team boards, payroll via kogi-bank | Benefits administered by org; potential PEP participation |
| Officer | Org governance, signatory authority, multi-sig approvals, cap table | Director-level fiduciary controls; org-level financial management |

# **5\. Organizations**

Organizations are first-class resource entities on the Kogi platform. An organization owns a Portfolio, holds financial accounts in kogi-bank, manages member roles, and participates in the Marketplace, Exchange, and Community modules. All organization types share the base Organization data model with type-specific governance extensions.

## **5.1 Organization Types**

| Organization Type | Governance Model | Key Platform Capabilities |
| :---- | :---- | :---- |
| Autonomous / Independent Org | Self-governed; founder-led; configurable bylaws | Full portfolio, bank, marketplace, exchange; define own governance rules; multi-sig treasury |
| Ad-hoc Organization | Lightweight; temporary; no formal charter required | Quick spin-up; shared workspace; disbands cleanly; assets redistributable |
| Open Source Community | Maintainer-led; contributor meritocracy; RFC process | Public repository-linked portfolio; contribution attribution; bounty system; governance proposals |
| Cooperative | One member, one vote; democratic; profit-sharing bylaws | Cooperative treasury; revenue distribution engine; PEP benefits; member equity accounts |
| Collective | Open contribution; light moderation; consensus-preferred | Shared portfolio; crowdresourcing campaigns; mutual aid fund; group economics |
| Federation | Inter-org governance; delegated authority; federated identity | Cross-org portfolio; inter-org governance protocol; federation treasury; joint campaigns |
| Autonomous / Independent Team | Team lead authority; scoped to a project or program | Team boards, backlogs, shared portfolio; booking and logistics for team resources |
| Ad-hoc Group | Informal; no charter; temporary coordination | Shared workspace; basic backlog; group messaging; lightweight resource sharing |
| Council | Representative body; delegated authority; formal agenda | Governance proposals; voting records; policy authoring; inter-org coordination |
| Assembly | All-member participatory governance; quorum-required | Community-wide governance events; proposal lifecycle; ratification records; broadcast communications |

## **5.2 Organization Base Data Model**

| Field | Description |
| :---- | :---- |
| **org\_id** | UUID — immutable platform-wide identifier |
| **org\_type** | Enum: Cooperative | Collective | Federation | OpenSource | Autonomous | AdHoc | Team | Council | Assembly |
| **name** | Legal or display name |
| **slug** | URL-safe unique handle |
| **charter\_id** | ArtifactId of founding charter document; versioned |
| **bylaws\_id** | ArtifactId of bylaws or operating agreement; versioned |
| **members** | Vec\<MemberRecord\> — each with EntityId, role, join date, contribution weight |
| **governance\_config** | GovernancePolicy — vote thresholds, quorum, approval gates, multi-sig settings |
| **treasury\_account\_id** | kogi-bank cooperative treasury account; multi-sig controlled |
| **portfolio\_id** | Root portfolio for the organization |
| **legal\_entity** | Option\<LegalEntity\> — LLC | Corp | Trust | Fund | Informal |
| **tax\_id** | Option\<String\> — EIN or equivalent; used for grant and payment processing |
| **public\_profile\_id** | Community-facing profile; public portfolio view; member directory |
| **initiatives** | Vec\<InitiativeId\> — active platform initiatives this org is enrolled in |
| **linked\_orgs** | Vec\<OrgLink\> — federations, partnerships, or coalition memberships |
| **created\_at / updated\_at** | ISO 8601 timestamps |

## **5.3 Organization Governance Features**

| Feature | Description | Governance Gate |
| :---- | :---- | :---- |
| Membership Management | Invite, onboard, offboard, and role-change members | Admin or Council approval |
| Governance Proposals | Formal proposals for policy changes, budget approvals, initiative launches | Vote: simple majority | supermajority | unanimous (configurable) |
| Multi-Sig Treasury | M-of-N required approvals for disbursements above threshold | Configured at org creation; threshold per transaction size |
| Revenue Distribution | Automated periodic payroll or revenue share computation and payout | Treasurer triggers; governance vote approves; batch multi-sig executes |
| Initiative Enrollment | Join platform-coordinated initiatives (portable benefits, grants, equity crowdfunding) | Admin approval; member vote for large financial commitments |
| Federated Identity | Cross-org identity: members can act on behalf of the org in federated spaces | Delegated authority; role-scoped; revocable |
| Audit & Compliance | Full EventLog of all org actions; exportable governance reports | Always-on; immutable |
| Dissolution Protocol | Formal winding-down: asset distribution, member notification, archive | Supermajority \+ legal entity requirements |

# **6\. Resource Lifecycle**

All Kogi resources move through a defined lifecycle state machine. States are enforced by the ResourceEngine; transitions may require governance approval depending on the resource type and policy configuration.

## **6.1 Universal Lifecycle States**

| State | Description |
| :---- | :---- |
| **Draft** | Resource is being authored; not yet visible outside the creator's workspace |
| **Active** | Resource is live; visible and actionable per its visibility setting |
| **Paused** | Temporarily suspended; retains state; not actionable; restorable |
| **Completed** | Work or transactional resource has reached its terminal success state |
| **Cancelled** | Terminated before completion; reason logged; relevant parties notified |
| **Archived** | Closed out; moved to deep storage; read-only; full restore available |
| **Deleted** | Soft-deleted; removed from all views; hard-delete after retention period |
| **Under Review** | Governance review in progress; mutations blocked pending approval |
| **Disputed** | Active dispute between parties; escrow or access may be locked pending resolution |

## **6.2 Resource-Specific Lifecycle Extensions**

| Resource Domain | Additional States | Trigger Condition |
| :---- | :---- | :---- |
| Commerce / Contracts | Inquiry → Quoted → Negotiating → Signed → In-Delivery → Settled → Closed | Client-provider interaction flow |
| Financial Instruments | Proposed → Subscribed → Funded → Vesting → Distributing → Redeemed | Expired | Capital raise or instrument lifecycle |
| Work Execution | Backlog → Ready → In Progress → In Review → Done → Released | Agile / WBS delivery flow |
| Grants | Discovered → Applied → Under Review → Awarded → Disbursing → Reporting → Closed | Grant management flow |
| Contributions | Submitted → Pending Review → Accepted | Rejected → Merged → Distributed | Contribution review and attribution flow |
| Organizations | Forming → Active → Growing → Restructuring → Dormant → Dissolving → Dissolved | Organizational health lifecycle |

# **7\. Resource Analytics**

Every resource generates analytics data captured by the AnalyticsEngine and TelemetryEngine. Analytics are surfaced on individual resource dashboards, portfolio rollups, organizational reports, and the community feed.

## **7.1 Universal Analytics Metrics**

| Metric Category | Metrics | Engine |
| :---- | :---- | :---- |
| Engagement | Views, unique visitors, time-on-resource, interaction count, save/bookmark rate, share rate | AnalyticsEngine |
| Social | Follows, subscribers, likes/reactions, comments, reposts, mentions, hashtag spread | AnalyticsEngine |
| Contribution | Contribution count, contributor diversity, contribution velocity, attribution weight distribution | CollaborationEngine |
| Commerce | Inquiry-to-close rate, average deal size, booking utilization, repeat client rate, LTV | CRMEngine |
| Financial | Revenue generated, expenses incurred, profit margin, grant received, investment raised | kogi-bank |
| Work Performance | Velocity, throughput, cycle time, lead time, defect rate, sprint completion % | AnalyticsEngine \+ OptimizationEngine |
| Growth | Follower growth rate, member growth, resource creation rate, network expansion index | TelemetryEngine |
| Sentiment | Community sentiment score, review ratings, endorsement velocity, NPS equivalent | RecommendationEngine |
| Risk | Blocker count, dependency chain depth, overdue items, budget variance, compliance gaps | RiskEngine |

## **7.2 Resource-Level Dashboard Widgets**

| Widget | Description |
| :---- | :---- |
| **Resource Health Score** | Composite score (0–100) across activity, governance compliance, contribution health, and financial status |
| **Engagement Trend** | 30/90-day rolling engagement trend chart; breakout by action type |
| **Contributor Map** | Visual network map of active contributors with attribution weights |
| **Financial Summary** | Revenue, expenses, allocations, and outstanding obligations; kogi-bank data |
| **Work Progress** | WBS completion percentage; velocity trend; milestone countdown |
| **Lifecycle Stage** | Current state with time-in-state and projected next transition |
| **Dependency Graph** | Visual graph of upstream and downstream dependencies; blocker highlighting |
| **Governance Activity** | Recent proposals, votes, approvals, and audit entries |
| **Oba Recommendations** | AI-generated action recommendations based on resource state, analytics, and platform signals |

# **8\. Engine Integrations**

KOGI-RMS is computation-agnostic at the data layer. All intelligence is provided by the Kogi Engine layer, accessed via the EngineGrpcServer on port 9100\. The following engines serve KOGI-RMS directly:

| Engine | Role in Resource Management |
| :---- | :---- |
| **AnalyticsEngine** | Computes all engagement, performance, and usage analytics for every resource type |
| **TelemetryEngine** | Real-time event streaming: resource state changes, user activity, system events |
| **RecommendationEngine** | Personalized resource discovery; 'resources you may need'; cross-type recommendation |
| **PersonalizationEngine** | Persona-aware UI surfaces; priority sorting of resources per user profile |
| **MatchEngine** | Worker-to-gig, resource-to-need, investor-to-campaign matching across resource types |
| **GraphEngine** | Dependency graph computation; resource relationship traversal; impact analysis |
| **RiskEngine** | Risk scoring for commitments, dependencies, financial instruments, and org governance |
| **OptimizationEngine** | Resource allocation optimization; capacity planning; portfolio health scoring |
| **SearchEngine** | Full-text and structured search across all resource types, metadata, and content |
| **AllocationEngine** | Human and physical resource allocation; conflict detection; over-allocation warnings |
| **IncentiveEngine** | Contribution attribution; reward computation; cooperative revenue distribution |
| **CollaborationEngine** | Shared portfolio coordination; concurrent edit conflict resolution; attribution weighting |
| **BenefitsEngine** | Benefits eligibility scoring for worker types; contribution tracking; coverage optimization |
| **GrantEngine** | Grant matching by eligibility, portfolio strength, and historical data |
| **CRMEngine** | Lead scoring and pipeline management for commerce resources |
| **BookingEngine** | Scheduling optimization; conflict detection; resource calendar coordination |
| **LogisticsEngine** | Physical resource routing; equipment tracking; crew schedule optimization |
| **DataStreamingEngine** | Real-time data export; BI connector feeds; event bus for third-party integrations |
| **GameEngine** | Gamification: contribution points, badges, streaks, community leaderboards |

# **9\. Cross-System Platform Integrations**

KOGI-RMS does not operate in isolation. Resource data flows bidirectionally across every Kogi subsystem. The following integration matrix defines the primary data exchange surfaces:

| Kogi System | Integration Surface | Resource Flow |
| :---- | :---- | :---- |
| Portfolio System | All KOGI-RMS resources are Portfolio Components; full CRDT, EventLog, governance, and analytics framework inherited | Bidirectional: all resource mutations sync to Portfolio EventLog |
| kogi-bank | Financial resources (accounts, instruments, grants, invoices) originate in kogi-bank; budgets and expenses flow back to resource dashboards | Bidirectional: resource financial actions write to bank ledgers |
| Marketplace | Commerce resources (gigs, offers, deals, proposals, bids) published to Marketplace; applications flow into resource assignment workflows | Outbound: publish; Inbound: inquiries, bids, applications |
| Exchange | Portfolio assets, equity stakes, revenue streams, IP, data — listed and traded on Exchange; settlement back to kogi-bank | Outbound: list; Inbound: bids, offers, settlement events |
| Community & Spaces | Resources shared to community spaces; crowdresourced contributions; initiative enrollment; organization membership and governance | Bidirectional: publish to community; contributions flow back |
| Work Management (WMS) | Work execution resources (boards, stories, tasks, sprints) are dual-registered in RMS and WMS; analytics unified | Bidirectional: WMS and RMS share the same underlying data model |
| Developer / API | All resource types accessible via REST and gRPC; webhook notifications on lifecycle transitions; SDK helpers for typed resource creation | Full CRUD via kogi-api; event subscriptions |
| Profiles & Personas | Persona data shapes resource recommendations, search ranking, and matching; user role assignments persisted per resource | Profile data informs MatchEngine and PersonalizationEngine |

# **10\. Glossary**

| Term | Definition |
| :---- | :---- |
| **Resource** | Any entity — structured or unstructured, human or non-human — that has value, can be acted upon, and can be associated with a Portfolio component |
| **ResourceType** | The top-level classification of a resource: Portfolio\_Work | Planning\_Time | Work\_Execution | Commerce\_Contracts | Capital\_Physical | Financial\_Instruments | Contributions |
| **ContributionRecord** | An append-only attribution record in a resource's EventLog tracking who contributed what, valued how, and weighted how in distribution calculations |
| **Portfolio Component** | The base data model for all items on the Kogi platform; every resource is a Portfolio Component with CRDT versioning and EventLog auditing |
| **Persona** | An identity and interest classification assigned to a user that shapes recommendation, matching, and community surfaces |
| **Worker Type** | A platform classification of a user's working arrangement (contractor, gig worker, freelancer, etc.) that governs benefit eligibility and tax treatment |
| **Organization** | A first-class resource entity representing a collective of users with shared governance, treasury, portfolio, and membership |
| **Cooperative** | A worker-owned organization with democratic governance and revenue-sharing bylaws; managed via cooperative treasury in kogi-bank |
| **Federation** | A cross-organization meta-entity enabling joint governance, shared portfolio, and inter-org resource exchange between aligned organizations |
| **Multi-Sig Treasury** | An organization bank account requiring M-of-N approver signatures for disbursements above a configured threshold |
| **Portable Benefits** | Worker-centered benefits (health, retirement, PTO, insurance, professional development) that travel with the individual, not the employer |
| **Crowdresourcing** | Community-sourced contributions of labor, assets, knowledge, and capital to a shared resource or project |
| **Timebox** | A fixed time period bounding a set of committed work items (sprint, PI, quarter, custom duration) |
| **Lifecycle State** | The current stage in a resource's state machine (Draft, Active, Archived, etc.); transitions may require governance approval |
| **EventLog** | An append-only, tamper-evident audit history attached to every resource recording all mutations with actor, timestamp, and delta |
| **Oba** | The Kogi AI assistant; surfaces resource recommendations, analytics summaries, governance alerts, and optimization suggestions across all resource types |
| **MatchEngine** | The Kogi engine responsible for matching workers to gigs, resources to needs, and investors to campaigns based on multi-dimensional compatibility scoring |
| **CRDT** | Conflict-free Replicated Data Type — the distributed data structure enabling safe concurrent edits across all resource types |

KOGI — Independent Worker Operating System  ·  Resource Management System SDD  ·  v1.0  ·  March 2026