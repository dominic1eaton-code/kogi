

**KOGI PLATFORM**

**kogi-game**

**Game System & Mechanism Engine**

*System Design Document*

v1.0  ·  Kogi Platform  ·  March 2026

| KGAM  ·  kogi-game  ·  kogi-engine  ·  MatchEngine  ·  AllocEngine  ·  IncentiveEngine *Resource · Capital · Labor · Exchange · Marketplace · Portfolio · Assets* *Bids · Offers · Deals · Requests · Proposals · Contracts · Gigs · Tasks* *Matching · Allocation · Incentive Mechanisms · Recommendations · Personalization* |
| ----- |

Kogi Team  ·  Independent Worker Operating System  ·  Confidential

# **Table of Contents**

| 1\. OVERVIEW & PHILOSOPHY |
| :---- |

## **1.1 What is kogi-game?**

kogi-game is the mechanism design and game engine of the Kogi Platform — the mathematical and computational substrate that governs how resources, capital, labor, portfolio components, and financial instruments are discovered, matched, priced, allocated, exchanged, and incentivized across the platform.

Every transaction on the Kogi Platform is, at its core, an economic game: a buyer and seller seeking mutually beneficial terms, a worker and opportunity seeking optimal fit, an investor and portfolio seeking the best risk-adjusted return, a collective seeking a fair distribution rule. kogi-game formalizes these games — specifying the rules, the scoring functions, the information revelation protocols, and the equilibrium-seeking mechanisms — so that outcomes are efficient, fair, and incentive-compatible for all participants.

| *Design Principle: Mechanism design is the engineering of economic games. Where economics asks 'what outcomes do given rules produce?', mechanism design asks 'what rules produce the outcomes we want?' kogi-game answers that question for every interaction on the platform — from a single gig bid to a multi-million dollar equity round — designing rules that align individual incentives with platform-wide efficiency.* |
| :---- |

## **1.2 The Four-Layer Architecture**

kogi-game operates as four interdependent layers, each building on the layer below:

| Layer | Name | Responsibility | Implementation |
| :---- | :---- | :---- | :---- |
| L4 (Top) | Incentive Layer | Reputation, rewards, stakes, and long-run behavior shaping | IncentiveEngine — Go service \+ kogi-engine signals |
| L3 | Allocation Layer | Deciding who gets what, when, at what price, in what order | AllocEngine — Rust core \+ Go service |
| L2 | Matching Layer | Finding optimal pairings across all entity types | MatchEngine — Scala (kogi-engine) \+ Go service |
| L1 (Base) | Signal Layer | Collecting, normalizing, and distributing the data that all higher layers depend on | TelemetryEngine \+ AnalyticsEngine (kogi-engine) |

## **1.3 Entities in the Game**

Every entity in the Kogi Platform is a player in one or more games. The game system categorizes all participants and resources:

| Entity Class | Platform Types | Game Role |
| :---- | :---- | :---- |
| Labor | Independent workers, contractors, freelancers, gig workers, team members | Sellers of time, skills, and deliverables; bidders in RFP auctions |
| Capital | Investors, donors, lenders, campaign backers, grant providers | Allocators of financial resources; participants in investment matching |
| Resources | Compute, licenses, materials, equipment, tools, data, templates | Supply-side of resource allocation games; inputs to production |
| Portfolio Components | Programs, Projects, Assets, Artifacts, Resources (Portfolio System kinds) | Objects of investment, trade, and collaboration; generate signals |
| Financial Instruments | Equity, revenue-share notes, bonds, tokens, derivatives | Traded in kogi-exchange financial instruments market |
| Demand-side entities | Buyers, hirers, organizations, cooperatives, teams | Create demand signals; fund escrow; initiate matching |
| Transactional entities | Bids, Offers, Deals, Requests, Proposals, Contracts, Gigs, Tasks | Game moves — the atomic actions through which players interact |
| Campaigns | Fundraise, equity, group-buy, resource, bond, pre-sale | Coordination games — multiple buyers coordinating on a shared purchase or investment |
| Collectives/Cooperatives | Worker collectives, DAOs, federated entities | Cooperative games — designing fair distribution rules within groups |

## **1.4 Service Architecture**

| Service | Code | Technology |
| :---- | :---- | :---- |
| kogi-game | KGAM | Go (primary service) \+ Rust (core mechanisms) |
| kogi-engine MatchEngine | KMTCH | Scala 3, gRPC port 9100 |
| kogi-engine AllocEngine | KALLOC | Scala 3, embedded in kogi-engine |
| kogi-engine IncentiveEngine | KINC | Scala 3, embedded in kogi-engine |
| kogi-engine RecommendationEngine | KREC | Scala 3, embedded in kogi-engine |
| kogi-engine PersonalizationEngine | KPERS | Scala 3, embedded in kogi-engine |
| kogi-engine AnalyticsEngine | KANL | Scala 3, embedded in kogi-engine |
| kogi-engine RiskEngine | KRSK | Scala 3, embedded in kogi-engine |
| kogi-engine GraphEngine | KGRPH | Scala 3, embedded in kogi-engine |

| 2\. MECHANISM DESIGN FOUNDATIONS |
| :---- |

## **2.1 Core Principles**

kogi-game's mechanism design is grounded in the following economic design principles, applied pragmatically to the independent worker economy context:

| Principle | Application in kogi-game |
| :---- | :---- |
| **Incentive Compatibility** | Mechanisms are designed so that the dominant strategy for every participant is to reveal their true preferences — honest bidding, accurate skill declaration, genuine availability — because truthful behavior maximizes their payoff. |
| **Individual Rationality** | No participant is made worse off by participating. Every mechanism guarantees that the outside option (not participating) is always at least as good as the worst outcome from participating. |
| **Pareto Efficiency** | Allocations are designed to exhaust all mutually beneficial trades before closing a market. No resource sits idle when a willing buyer and seller could be matched. |
| **Budget Balance** | Platform fees and mechanism payouts are designed so the platform's net revenue from running a mechanism is non-negative (weak budget balance) without extracting rents that would deter participation. |
| **Fairness & Non-Discrimination** | Matching algorithms do not discriminate on protected characteristics. Scoring functions are transparent, auditable, and explainable to all participants. |
| **Liquidity Design** | Mechanisms specifically account for thin markets (few participants) by using reservation prices, automated market makers, and fallback mechanisms to prevent market failure. |
| **Dynamic Consistency** | Mechanisms produce consistent outcomes over time — a rule that produces a good outcome today should not produce a bad outcome tomorrow due to strategic adaptation by players. |
| **Transparency** | Mechanism rules, scoring weights, and outcome explanations are available to participants. Opaque 'black box' allocations that participants cannot audit are avoided. |

## **2.2 Game Types by Platform Context**

Different platform interactions are formalized as distinct game types, each with appropriate mechanism design solutions:

| Game Type | Platform Context | Mechanism |
| :---- | :---- | :---- |
| Assignment Game | Worker-to-gig matching, resource-to-project matching | Deferred Acceptance (Gale-Shapley) with multi-dimensional scoring |
| Double Auction | Exchange market — simultaneous buyers and sellers posting prices | Continuous Double Auction (CDA) with price-time priority order book |
| Vickrey Auction (VCG) | Single-item high-value asset sales, exclusive contracts | Second-price sealed-bid; incentive-compatible, truthful revelation |
| Combinatorial Auction | Bundled resource or asset packages, multi-resource project staffing | Iterative Combinatorial Auction (ICA) with proxy bidding |
| Reverse Auction / RFP | Buyer posts requirements; sellers compete for the contract | Score-based sealed-bid reverse auction with price+quality scoring |
| Negotiated Exchange | Bilateral deal negotiation (Deal Room) | Alternating-offer Rubinstein bargaining model with AI-assisted ZOPA computation |
| Cooperative Game | Payoff distribution in collectives and cooperatives | Shapley Value computation for fair contribution-based distribution |
| All-or-Nothing Campaign | Crowdfunding with a minimum threshold | Assurance contract / threshold public goods game |
| Dynamic Pricing | Platform fee setting, listing boost pricing | Myerson-optimal mechanism with demand curve estimation |
| Reputation Game | Long-run behavior shaping via reputation scores | Repeated game with discount factor; Folk Theorem equilibrium maintenance |

## **2.3 Zone of Possible Agreement (ZOPA) Engine**

For bilateral deal negotiations in the Deal Room, kogi-game computes the Zone of Possible Agreement — the range of prices/terms where both parties' reservation values overlap — using signals from both sides' platform history and market data:

| ZOPA Condition | ZOPA ≠ ∅  iff  seller\_reservation\_price ≤ buyer\_reservation\_price |
| :---- | :---- |

| ZOPA Range | ZOPA \= \[seller\_reservation\_price,  buyer\_reservation\_price\] |
| :---- | :---- |

The ZOPA Engine collects: seller's historical deal prices for similar work, buyer's historical spending for similar engagements, current market rate from MatchEngine comparables, and time-pressure signals (deadline proximity, competing offers). Oba presents the ZOPA as a price range guidance panel in the Deal Room, enabling faster convergence to agreement without revealing private reservation prices.

## **2.4 Shapley Value for Cooperative Distribution**

For collectives, cooperatives, and multi-contributor campaigns, kogi-game computes the Shapley Value — the unique fair allocation that assigns each participant their marginal contribution averaged over all possible orderings of the coalition:

| Shapley Value | φᵢ(v) \= Σ\_{S⊆N\\{i}} \[|S|\!(|N|-|S|-1)\! / |N|\!\] × \[v(S∪{i}) \- v(S)\] |
| :---- | :---- |

In practice, exact Shapley computation is NP-hard for large coalitions, so kogi-game uses stratified Monte Carlo sampling with O(n log n) convergence for cooperative entities up to 50 members, and approximation algorithms (Group Shapley) for larger federations. The result drives patronage distribution in cooperatives and contribution-weighted payouts in campaigns.

| 3\. ALLOCATION ENGINE (AllocEngine) |
| :---- |

## **3.1 Purpose**

The AllocEngine is the computational core that takes the output of the MatchEngine (ranked candidate pairs) and the rules specified by a given mechanism, and produces a binding allocation: who gets what resource/opportunity, at what price, under what terms, and in what order. It is the 'solver' that turns preferences and bids into outcomes.

## **3.2 Resource Allocation**

### **3.2.1 Resource Types**

| Resource Category | Platform Instances | Allocation Mechanism |
| :---- | :---- | :---- |
| Human Labor | Worker time, contractor availability, gig capacity | Assignment market — Deferred Acceptance with skill \+ reputation scoring |
| Financial Capital | Funds from investors, donors, lenders, campaigners | Budget-constrained combinatorial allocation with portfolio fit scoring |
| Compute Resources | Platform compute credits, GPU time, API quotas | First-price auction \+ priority queue for real-time allocation |
| Intellectual Assets | Templates, playbooks, designs, code modules, datasets | Posted-price marketplace; recommendation-driven discovery |
| Physical Resources | Equipment, materials, real estate (future) | Lease/rental auction with availability scheduling |
| Platform Attention | Feed placement, recommendation slots, promoted listings | Ad-auction style with quality-adjusted bid (QScore × CPM) |
| Cooperative Commons | Shared tools, pooled licenses, mutual aid funds | Equal-access commons with governance-controlled rationing |

### **3.2.2 Labor Allocation — Deferred Acceptance Protocol**

The primary mechanism for worker-to-opportunity allocation is a Deferred Acceptance (DA) algorithm — a stable matching mechanism proven to produce stable outcomes where no worker-opportunity pair would both prefer to be matched to each other over their current assignment:

1. Opportunities (gigs, contracts, RFPs) post their requirements and scoring weights

2. Workers apply to opportunities, optionally in rank-ordered preference lists

3. AllocEngine runs DA: opportunities provisionally accept their highest-scored applicants, reject others

4. Rejected workers apply to their next-preferred opportunity

5. Process repeats until no rejected applicant remains — stable matching reached

6. Final assignments reported to kogi-marketplace; escrow instructions generated for kogi-bank

### **3.2.3 Capital Allocation — Portfolio Budget Engine**

When an investor, donor, or funder commits capital, the AllocEngine runs a portfolio budget allocation to determine optimal deployment across competing opportunities:

| Budget Allocation | max Σᵢ (expected\_return\_i × allocation\_i)  s.t.  Σᵢ allocation\_i ≤ budget,  allocation\_i ≥ min\_ticket\_i |
| :---- | :---- |

Inputs: investor preference vector (sector, stage, instrument type, risk tolerance), portfolio fit scores from MatchEngine, expected return signals from AnalyticsEngine (cashFlow, productivity, portfolio health), and diversification constraints from the investor's risk policy. Output: ranked investment allocation plan presented to the investor as an Oba-generated recommendation card with one-click execution.

## **3.3 Mechanism-Specific Allocation Rules**

### **3.3.1 Bid Allocation**

| Bid Type | Allocation Rule |
| :---- | :---- |
| **Open Ascending Bid (English Auction)** | Highest bidder at close wins; price \= their bid. Used for asset sales and exclusive contract awards. |
| **Sealed First-Price Bid** | Highest bidder wins; pays their bid. Used in sealed RFP competitions and blind auctions. |
| **Sealed Second-Price (Vickrey)** | Highest bidder wins; pays second-highest bid. Incentive-compatible; dominant strategy is truthful. Used for high-value, single-item allocations. |
| **Combinatorial Bid** | Bidder bids on bundles of items. Winner determined by revenue-maximizing allocation over all bids. Used for multi-resource project staffing packages. |
| **Reverse Bid (Buyer Auction)** | Lowest qualifying bid wins the buyer's contract. Price \= winning bid. Qualification assessed by quality score threshold. |
| **Proxy Bid** | Bidder sets max price; system bids incrementally on their behalf up to max. Maintains bidder privacy while pursuing optimal outcome. |

### **3.3.2 Offer Allocation**

| Offer Type | Allocation Rule |
| :---- | :---- |
| **Posted Price (Take-or-Leave)** | Single fixed price. First buyer to accept gets the item. No negotiation. Used for standard marketplace listings. |
| **Declining Price (Dutch Auction)** | Price starts high and decreases over time. First buyer to accept at current price wins. Creates urgency for buyers. |
| **Bundle Offer** | Multiple items offered as a package at a discount. All-or-nothing acceptance. Used for asset portfolios, template bundles. |
| **Subscription Offer** | Ongoing access to a portfolio component or service stream at a recurring price. First-come first-served up to capacity limit. |

### **3.3.3 Deal Allocation**

| Deal Type | Allocation Rule |
| :---- | :---- |
| **Bilateral Negotiated Deal** | ZOPA Engine guides price discovery; offer/counter-offer protocol. Deal closes when both parties accept simultaneously or one party accepts the last standing offer. |
| **Tripartite Deal** | Three-party deal (e.g., worker \+ client \+ platform). Each pair has bilateral preferences; AllocEngine finds a jointly Pareto-improving allocation. |
| **Multi-Party Consortium Deal** | N-party collective purchase or investment. Each party has a budget share; AllocEngine maximizes collective utility subject to each party's share constraint. |
| **Conditional Deal** | Deal terms contingent on a future event (milestone, funding close, regulatory approval). AllocEngine tracks the condition and executes allocation on trigger. |

### **3.3.4 Request / RFP Allocation**

| Request Type | Allocation Rule |
| :---- | :---- |
| **Open RFP** | Request broadcast to all qualified workers/vendors. AllocEngine scores all proposals against buyer's scoring rubric. Buyer selects from ranked shortlist. |
| **Restricted RFP** | Request sent only to a pre-qualified vendor pool (curated by MatchEngine or buyer). Same scoring; restricted supply side. |
| **Budget-Capped RFP** | All proposals at or below budget ceiling scored; cheapest acceptable proposal wins (reverse auction). Scope quality must meet minimum threshold. |
| **Blind RFP** | Buyer identities and budgets are anonymized to prevent strategic pricing. Only skills, timeline, and scope are revealed. Promotes honest proposal pricing. |

### **3.3.5 Contract, Gig & Task Allocation**

| Entity | Allocation Rule |
| :---- | :---- |
| **Contract** | Bilateral assignment after RFP or direct hire. AllocEngine generates milestone payment schedule, assigns escrow plan, and links to PortfolioSystem project as a ResourceAllocation. |
| **Gig** | Direct fixed-price assignment, typically from a Proposal or Offer. Fast-track: MatchEngine proposes → buyer accepts → escrow funded in one step. |
| **Task** | Sub-unit of a Contract or Gig. Allocated to a worker within an already-agreed engagement scope. No separate bidding; assignment governed by contract terms. |
| **Bounty Task** | Open task with a fixed reward posted publicly. First worker to meet acceptance criteria wins the bounty. AllocEngine validates delivery before releasing funds. |

## **3.4 Platform Attention Allocation**

Feed placement, recommendation slot prominence, and marketplace discovery ranking are themselves allocation problems. kogi-game governs these 'attention markets' using quality-adjusted score auctions:

| Quality Score (QScore) | QScore \= RelevanceScore × 0.40 \+ ReputationScore × 0.30 \+ EngagementScore × 0.20 \+ FreshnessScore × 0.10 |
| :---- | :---- |

| Rank Score (for promoted listings) | RankScore \= QScore × bid\_per\_impression |
| :---- | :---- |

Organic rankings use QScore alone. Promoted slots use RankScore, ensuring that poorly-quality listings cannot simply outbid high-quality ones to dominate placement. This protects ecosystem health by requiring minimum quality thresholds for promotion participation.

| 4\. MATCHING ENGINE |
| :---- |

## **4.1 Overview**

The MatchEngine (kogi-engine sub-system 8\) performs multi-dimensional scoring to match any MatchSubject — a worker, portfolio component, resource, investor, or analytics artifact — against any other MatchSubject or pool of subjects. kogi-game extends the base MatchEngine with game-theoretic scoring layers, market-aware boosts, and mechanism-specific match operations for each transaction type in the platform.

## **4.2 Match Subject Taxonomy**

| MatchSubject Type | Sub-types | Key Discriminants |
| :---- | :---- | :---- |
| UserSubject | Worker, Investor, Donor, Contributor, Collaborator, Buyer, Hirer | role, persona, skills\[\], interests\[\], budget, location, reputation\_score, availability |
| ComponentSubject | Portfolio, Program, Project, Resource, Asset, Artifact | kind, status, requiredSkills\[\], budget, tags\[\], health\_score, visibility |
| ResourceSubject | Compute, License, Equipment, Template, Dataset | resourceType, capacity, availability, cost, location, tags\[\] |
| TransactionSubject | Bid, Offer, Deal, Request, Proposal, Contract, Gig, Task | transaction\_type, price, scope, deadline, required\_skills\[\], budget, status |
| FinancialSubject | Equity round, Campaign, Bond, Revenue-share note | instrument\_type, raise\_amount, stage, sector, expected\_return, risk\_level |
| AnalyticsArtifact | Recommendation, Search result, Index entry, Filter result | artifactType, affinity\_tags\[\], score, freshness |

## **4.3 Core Scoring Dimensions & Weights**

| Dimension | Default Weight | Computation | Overridable? |
| :---- | :---- | :---- | :---- |
| Tag Similarity | 40% | Jaccard similarity of subject tag sets | Yes — per mechanism type |
| Attribute Affinity | 30% | Weighted overlap of shared attribute key-value pairs | Yes — per mechanism type |
| Persona Alignment | 20% | Persona-to-component-type affinity matrix lookup | Partial — base matrix fixed |
| Signal Strength | 10% | Analytics signal from interaction history and engagement events | No — engine-determined |

## **4.4 Mechanism-Specific Match Operations**

### **4.4.1 Worker-to-Opportunity Matching**

Triggered by: new gig/contract posting, RFP broadcast, or worker's Oba morning briefing.

| Scoring Dimension | Weight | Data Source |
| :---- | :---- | :---- |
| Skills Coverage | 35% | Worker skill tags vs. opportunity requiredSkills — exact \+ fuzzy semantic match |
| Reputation Score | 25% | Worker's historical ratings, completion rate, dispute rate |
| Budget Compatibility | 20% | Worker's rate range vs. opportunity budget — no match if rate \> 2× budget ceiling |
| Availability Alignment | 10% | Worker's calendar availability vs. opportunity start/end dates |
| Location / Timezone Compatibility | 5% | Timezone overlap for synchronous collaboration requirements |
| Past Collaboration Boost | 5% | Bonus if worker has worked with this buyer before and both rated positively |

### **4.4.2 Investor-to-Campaign / Round Matching**

Triggered by: new campaign/equity round posted, or investor's campaign browsing session.

| Scoring Dimension | Weight | Data Source |
| :---- | :---- | :---- |
| Portfolio Fit (Sector / Stage) | 35% | Investor's portfolio preferences vs. campaign's sector and stage tags |
| Financial Instrument Preference | 25% | Investor's instrument history (equity vs. revenue-share vs. bond) vs. campaign type |
| Risk Profile Alignment | 20% | Investor's risk tolerance vs. campaign's PortfolioHealthScore and RiskEngine output |
| Return Signal | 10% | Expected return estimate from AnalyticsEngine (cashFlow delta, comparable outcomes) |
| Social Proof | 10% | Number of existing investors the target investor follows who have already backed the campaign |

### **4.4.3 Resource-to-Project Matching**

Triggered by: project's resource gap detection by RiskEngine (OptimizationPlan), or manual resource search.

| Scoring Dimension | Weight | Data Source |
| :---- | :---- | :---- |
| Functional Fit | 40% | Resource capability tags vs. project requiredSkills / requiredResources |
| Cost Within Budget | 30% | Resource cost vs. project ResourceAllocation budget remaining |
| Availability | 20% | Resource availability window vs. project timeline (GraphEngine CPM) |
| Quality Signal | 10% | Resource's historical usage ratings and delivery success rate |

### **4.4.4 Collaborator Discovery Matching**

Triggered by: user searching for co-founders, contributors, co-creators, or community collaborators.

| Scoring Dimension | Weight | Data Source |
| :---- | :---- | :---- |
| Complementary Skills | 30% | Jaccard distance of skill sets — seeks differentiation, not duplication |
| Shared Interests / Goals | 30% | Interest tag overlap weighted by interaction depth |
| Community Graph Proximity | 25% | Second-degree social graph distance via GraphEngine relationship edges |
| Portfolio Compatibility | 15% | Component kind and status alignment — both in compatible lifecycle stages |

### **4.4.5 Portfolio Asset / Artifact Trading Matching**

Triggered by: user browsing marketplace, or asset listed for sale.

| Scoring Dimension | Weight | Data Source |
| :---- | :---- | :---- |
| Category / Tag Relevance | 40% | User's dominant categories and interest tags vs. asset tags |
| Price Affinity | 30% | Buyer's typical spend range vs. asset price — strong signal from purchase history |
| Quality Score | 20% | Asset's PortfolioHealthScore, ratings, download/usage count |
| Social Proof | 10% | Number of people in buyer's network who own or rated this asset |

## **4.5 Persona-Aware Match Boosting**

| Persona | Boost Applied | Rationale |
| :---- | :---- | :---- |
| PowerUser | Asset and Program component matches × 1.2 | Power users extract more value from complex components |
| Investor (role) | Equity and revenue-share campaign matches × 1.3 | Primary motivation is financial return |
| Explorer | Diversity bonus — surfaces less-obvious but high-potential matches | Explorers seek discovery, not confirmation |
| Specialist | Domain-exact tag matches × 1.25 | Specialists want depth, not breadth |
| Collaborator | Network-adjacent matches × 1.2 — people known by people they know | Collaborators build through trust networks |
| Newcomer | Trending and well-rated items prioritized — cold-start protection | Insufficient history to personalize; use global signals |
| ValueSeeker | Price-efficiency score boosted × 1.2 | Maximizing return per dollar is primary driver |

## **4.6 Match Confidence & Explainability**

Every match produced by the MatchEngine carries a confidence score (0.0–1.0) computed from the completeness of input signals and the margin between the top match and the second-best match. Matches with confidence \< 0.4 are flagged for human review or Oba clarification. Every match result also carries an explanation vector — the weighted contribution of each scoring dimension — enabling the UI to display 'Why we matched you: Skills 62%, Budget 22%, Availability 16%' to both parties.

| 5\. RECOMMENDATION ENGINE |
| :---- |

## **5.1 Architecture Overview**

The RecommendationEngine is the platform's content and opportunity discovery system. Where the MatchEngine finds the best fit for a specific defined need, the RecommendationEngine proactively surfaces things users did not know to search for — portfolio components to follow, marketplace listings to buy, collaborators to connect with, investments to consider, campaigns to back — based on behavioral signals, interaction history, and social graph proximity.

## **5.2 User Profile & Behavioral Signals**

| Profile Field | Description |
| :---- | :---- |
| **interaction\_history** | Chronological log of all platform interactions: views, clicks, purchases, ratings, bookmarks, dismisses |
| **preference\_vector** | Normalized weighted sum over all interaction categories — continuously updated after each interaction |
| **dominant\_categories** | Top-3 categories by cumulative interaction weight — drives content ordering |
| **session\_context** | Current device, location, time-of-day, referrer — contextual adaptation |
| **persona** | Derived behavioral archetype: PowerUser, CasualBrowser, Explorer, EarlyAdopter, ValueSeeker, Specialist, Collaborator, Newcomer |
| **explicit\_preferences** | User-declared interests, preferred content types, blocked topics, notification preferences |
| **social\_graph** | Follow, subscribe, collaborate, and team relationships — second-degree social signals |
| **financial\_preferences** | Investment preferences, budget ranges, risk tolerance — for financial instrument recommendations |
| **portfolio\_context** | Active portfolio components and their tags — context for 'things relevant to my current work' |

## **5.3 Recommendation Methods**

### **5.3.1 Collaborative Filtering**

Finds users with similar interaction histories using cosine similarity on preference vectors. Items interacted with by similar users that the target user has not yet seen are surfaced, weighted by the collaborating user's similarity score and interaction weight. Best for: discovering popular items in the user's interest space that they have not found independently.

| Collaborative Score | cf\_score(u,i) \= Σ\_{v∈neighbors(u)} sim(u,v) × interaction\_weight(v,i) / |neighbors(u)| |
| :---- | :---- |

### **5.3.2 Content-Based Filtering**

Compares the user's preference vector against item feature vectors derived from tags, category, and attributes. Items with highest cosine similarity to the user's accumulated profile are returned with a recency boost for recently created items. Best for: recommending items highly similar to what the user already engages with.

| Content Score | cb\_score(u,i) \= cosine(preference\_vector(u), feature\_vector(i)) × recency\_boost(i) |
| :---- | :---- |

### **5.3.3 Hybrid Scoring (Default)**

Blends collaborative (40%), content-based (40%), and global popularity (20%) scores. Contextual boosting applied on top based on current session context (device, time-of-day, location). Cold-start fallback to trending+popular when interaction history is insufficient.

| Hybrid Score | hybrid(u,i) \= 0.40 × cf\_score \+ 0.40 × cb\_score \+ 0.20 × popularity\_score \+ context\_boost |
| :---- | :---- |

## **5.4 Recommendation Surfaces**

| Surface | Recommendation Type | Trigger |
| :---- | :---- | :---- |
| Home Dashboard Feed | Mixed: portfolio updates, new opportunities, community content, trending assets | Page load; real-time push on new matching events |
| Marketplace Discovery | Listings, workers, assets, and campaigns matching buyer's profile | Browse session; after purchase event (complementary goods) |
| Exchange Discovery | Financial instruments, equity rounds, and trading opportunities | Exchange session; after investment event |
| Collaborator Suggestions | Users with complementary skills or shared interests | Project creation; sprint blocker detected by RiskEngine |
| Resource Suggestions | Tools, templates, datasets, and services for active projects | Project creation; resource gap detected in OptimizationPlan |
| Morning Briefing Cards | Personalized action items: opportunities, follow-ups, risk alerts | Oba morning briefing generation; daily digest |
| Invest/Back Suggestions | Campaigns and rounds matching investor's portfolio preferences | Campaign creation; investor browsing session |
| Portfolio Expansion | New portfolio component types and templates to start | After project completion; portfolio health review |
| Community Space Suggestions | Spaces and communities matching user's tags and persona | Community browse; new member onboarding |
| Explore Feed | Directional expansion: 'based on X, explore Y and related areas' | Explicitly triggered by user; auto-surfaced for Explorer persona |

## **5.5 Feedback Loop & Continuous Improvement**

The RecommendationEngine maintains a continuous feedback loop. Every user interaction with a recommendation — click, view, dwell, purchase, bookmark, dismiss — is recorded as a UserInteraction and fed back into the profile, adjusting the preference vector and updating collaborative similarity scores. The feedback loop has the following latency profile:

| Feedback Event | Update Latency | Effect |
| :---- | :---- | :---- |
| Purchase / Rate | \< 2s | Immediate: preference\_vector \+= 5.0 × item feature vector; nearest-neighbor graph updated |
| Bookmark / Share | \< 2s | Immediate: preference\_vector \+= 3.0 × item feature vector |
| Click / View | \< 10s (batched) | Near-real-time: preference\_vector \+= 1.5 × item feature vector |
| Dismiss | \< 10s (batched) | Near-real-time: preference\_vector \-= 2.0 × item feature vector; item suppressed for 30 days |
| Dwell (\>30s) | \< 30s | Background: preference\_vector \+= 1.0 × item feature vector |
| Session-level re-ranking | Every 5 minutes | Background: re-score all cached recommendation lists with updated profile |
| Full profile rebuild | Daily | Batch: recompute all similarity scores, dominant categories, persona re-inference |

| 6\. PERSONALIZATION ENGINE |
| :---- |

## **6.1 Purpose**

The PersonalizationEngine extends the RecommendationEngine with platform-level personalization: how content is displayed, how users are segmented into cohorts, how A/B experiments are assigned and tracked, how the multi-armed bandit selects between content variants, and how the persona lifecycle evolves over time. Where the RecommendationEngine answers 'what should this user see?', the PersonalizationEngine answers 'how should they see it?'

## **6.2 Persona System**

| Persona | Behavioral Profile | UI Config | Score Weights |
| :---- | :---- | :---- | :---- |
| PowerUser | High frequency, advanced features, broad exploration | Compact density, RecencyFirst, 50 feed items, analytics sidebar, advanced filters | Risk × 1.10 (tighter guardrails) |
| CasualBrowser | Low engagement, short sessions, trending content | Spacious, TrendingFirst, 20 feed items, auto-expand items, suppress low-engagement | Risk × 0.85, Productivity × 0.90 (reduced noise) |
| Explorer | Diverse topics, high discover/explore engagement | Standard, RelevanceFirst, 40 feed items, diversity bonus, highlight new content | Collaboration × 1.15 (compensates lower natural score) |
| EarlyAdopter | Rapid new item adoption, beta features | Compact, RecencyFirst, highlight new, novelty score boost | No persona-specific weight adjustment |
| ValueSeeker | Price/ROI sensitive, deal-oriented | Standard, ValueFirst, price-efficiency score boosted | CashFlow × 0.90 (amplifies cashflow sensitivity) |
| Specialist | Deep domain focus, low breadth / high depth | Standard, RelevanceFirst, domain-specific recs, 10K row query cap, analytic hints | No persona-specific weight adjustment |
| Collaborator | High social engagement, groups, sharing | Standard, PersonalFirst, collaboration signal boost | No persona-specific weight adjustment |
| Newcomer | Cold-start, low history | Spacious, TrendingFirst, 20 feed items, cold-start recs | All signals × 0.80 (low confidence, prefer global signals) |

## **6.3 User Preferences System**

Users can explicitly override any persona-derived default through the Preferences system — a structured set of user-declared configuration options that always take precedence over inferred defaults:

| Preference Category | Options | Effect on Engine |
| :---- | :---- | :---- |
| Content Ordering | RecencyFirst, RelevanceFirst, TrendingFirst, ValueFirst, PersonalFirst | Overrides persona's default ordering in all feed surfaces |
| Feed Density | Compact, Standard, Spacious | Overrides persona's layout; affects all card and list views |
| Interest Topics | Explicit include and exclude topic tags | Boosts included tags × 1.5; filters out excluded tags entirely |
| Notification Preferences | Which event types trigger push/email; frequency (real-time, daily, weekly digest) | Governs Oba briefing card categories and alert thresholds |
| Financial Preferences | Preferred instrument types, min/max deal sizes, sector interests, risk appetite (1–5) | Weights FinancialSubject matches and IncentiveEngine financial nudges |
| Collaboration Preferences | Open to collaborator suggestions (Y/N), preferred team size, remote/local preference | Governs Collaborator Discovery surface and community space suggestions |
| Privacy Preferences | Profile visibility, portfolio visibility, anonymization on analytics exports | Governs what signals are observable by the RecommendationEngine and shared with third parties |

## **6.4 A/B Experiment Framework**

The PersonalizationEngine manages platform-wide A/B experiments for mechanism and UI variants:

* Experiment definition — id, name, variants (each with traffic allocation weight), status (draft / active / paused / completed), hypothesis, success metric

* Deterministic assignment — hash-based user-to-variant assignment ensures every user always gets the same variant for a given experiment; prevents experience fragmentation

* Exposure tracking — records which users were exposed to which variants, with timestamp and session context

* Conversion recording — records when an exposed user completes the experiment's target action (click, purchase, deal close, etc.)

* Statistical analysis — Bayesian A/B test with credible intervals; lift calculation per variant; minimum detectable effect sizing

* Guard rails — automatic experiment pause if any variant produces anomalous negative signals (error rate, churn rate, dispute rate above threshold)

## **6.5 Multi-Armed Bandit (UCB1)**

For content slot optimization and mechanism variant selection — where the best option is unknown and must be learned through exploration — the PersonalizationEngine implements the UCB1 (Upper Confidence Bound) algorithm:

| UCB1 Score | UCB1(arm) \= avg\_reward(arm) \+ √(2 × ln(total\_pulls) / arm\_pulls) |
| :---- | :---- |

Arms with zero pulls are always selected first (pure exploration priority). After sufficient pulls, exploitation naturally dominates. This balances exploration of untested variants against exploitation of known-good variants, converging to the optimal variant without requiring pre-specified exploration schedules.

## **6.6 Persona Lifecycle Management**

Personas evolve dynamically as user behavior changes — they are not static labels:

7. Build — initial persona inference from interaction history (minimum 20 interactions) and available profile data

8. Confirm — persona stabilizes after N consecutive sessions without drift crossing the re-assignment threshold

9. Drift Detection — current interaction pattern compared against persona archetype thresholds; drift score computed

10. Update — new persona label assigned if drift score exceeds 0.35 threshold; full history log maintained

11. Hysteresis — persona downgrade requires stronger evidence than upgrade to prevent oscillation

12. Manual Override — user can pin a persona label in preferences, preventing automatic re-assignment

| 7\. INCENTIVE ENGINE (IncentiveEngine) |
| :---- |

## **7.1 Purpose & Design**

The IncentiveEngine governs the long-run behavior of all platform participants by designing, tracking, and administering a multi-dimensional incentive system that rewards value-creating behaviors, penalizes value-destroying behaviors, and maintains equilibrium conditions necessary for a healthy multi-sided market. It is the platform's mechanism for shaping participant behavior across repeated interactions over time.

The IncentiveEngine draws on Repeated Game Theory — specifically the Folk Theorem, which establishes that in infinitely repeated games with patient players, cooperative equilibria (where all participants behave well) can be sustained through credible threats of punishment for defection. The platform acts as the long-run game coordinator, maintaining the threat of reputation penalties and access restrictions that make good behavior the dominant strategy even for self-interested actors.

## **7.2 Reputation System**

### **7.2.1 Reputation Score Architecture**

Every user and organizational entity has a multi-dimensional ReputationScore — not a single number, but a vector of scores across the interaction domains relevant to them:

| Dimension | Measures | Decay / Update Rule |
| :---- | :---- | :---- |
| Delivery Reliability | Contract / gig completion rate; on-time delivery rate; cancellation rate | Rolling 12-month window; recent events weighted 2× older events |
| Communication Quality | Response time to messages; clarity of scope documentation; dispute rate | Per-interaction rating; Bayesian update with prior \= platform average |
| Work Quality | Client ratings 1–5 across Quality, Timeliness, Communication, Value | Weighted average; minimum 5 ratings for public display |
| Financial Integrity | Invoice accuracy; payment timeliness (for buyers); dispute resolution outcome | Binary flags for fraud, non-payment; decay after 24 clean months |
| Community Contribution | Post quality ratings; knowledge sharing; mentorship; collaborative deliveries | Accumulated score; decays slowly if inactive (×0.95 per 90 days) |
| Investment Track Record | Campaign success rate backed; portfolio health of invested components | Rolling score; only applies to investor role |
| Governance Participation | Vote participation rate in collectives; proposal quality ratings | Cooperative/collective entities only |

### **7.2.2 Composite Reputation Score**

| Composite Score | R(u) \= 0.30×Delivery \+ 0.25×Quality \+ 0.20×Communication \+ 0.15×Community \+ 0.10×Financial |
| :---- | :---- |

The composite score produces a single 0–100 number displayed on user and organization profiles. Score bands translate to platform status tiers with associated incentive benefits:

| Tier | Score Range | Benefits | Access |
| :---- | :---- | :---- | :---- |
| Elite | 90–100 | Featured placement boost × 1.5; reduced platform fees (–20%); early access to beta features; priority dispute resolution | All marketplace, exchange, and campaign types |
| Trusted | 75–89 | Featured placement boost × 1.2; reduced platform fees (–10%); verified badge on all listings | All marketplace and exchange types |
| Established | 55–74 | Standard placement; standard fees; verified identity badge | All standard marketplace and exchange types |
| Developing | 35–54 | Standard placement; standard fees; no promoted listing access until 55+ | Standard gigs, proposals, and campaigns only |
| New | 0–34 | Cold-start tier; guided onboarding; limited deal sizes; escrow mandatory | Simple gigs and low-value transactions only |

## **7.3 Reward Mechanisms**

### **7.3.1 Platform Credit System (KogiPoints)**

KogiPoints (KP) are the platform's internal incentive currency — non-monetary credits earned through value-creating behaviors and redeemable against platform fees, promoted listing boosts, and premium features:

| Earning Event | KP Awarded | Rationale |
| :---- | :---- | :---- |
| Complete a gig / contract (seller) | 50–500 KP (proportional to contract value) | Core labor market activity |
| Complete a deal as buyer (first hire of a worker) | 100 KP | Buyer activation incentive |
| Receive 5-star rating | 25 KP | Quality signaling reward |
| Publish a portfolio component to marketplace | 75 KP | Supply-side contribution |
| Back a campaign (first backing) | 50 KP | Campaign liquidity incentive |
| Successful referral (new user completes first deal) | 250 KP | Network growth reward |
| Complete onboarding profile (skills \+ portfolio \+ availability) | 100 KP one-time | Information quality incentive |
| Write a detailed review (\>100 words) | 15 KP | Review quality incentive |
| Participate in cooperative governance vote | 10 KP | Governance participation |
| Mentor a Newcomer (guided interaction) | 50 KP | Community contribution |
| Resolve a dispute without escalation | 30 KP | Platform health reward |

| Redemption | KP Cost | Effect |
| :---- | :---- | :---- |
| Platform fee discount (10%) | 200 KP per invoice | Direct financial benefit |
| Promoted listing boost (7 days) | 500 KP | Increased discovery placement |
| Priority matching queue (30 days) | 1000 KP | Top-of-list for incoming RFP matches |
| Campaign boost (7 days) | 750 KP | Campaign featured in discovery feed |
| Exchange order fee waiver (1 trade) | 150 KP | Per-transaction fee reduction |

### **7.3.2 Streak & Consistency Incentives**

Streaks reward sustained behavioral consistency — a form of dynamic incentive that increases marginal reward for continued participation:

* Activity streak — earn bonus KP multiplier for each consecutive week with at least one completed transaction (×1.05 per week, cap ×2.0)

* Response rate streak — maintain \>90% response rate for 30+ consecutive days: promoted in worker discovery ranking

* Review streak — write a quality review after every engagement for 5+ consecutive engagements: earn Reviewer badge

* Campaign streak — successfully complete 3+ sequential campaigns as creator: earn Campaign Creator badge with fee discount

* Governance streak — participate in every cooperative vote for 6+ consecutive months: earn Governance Champion badge

### **7.3.3 Subscription & Loyalty Incentives**

Portfolio component creators can offer subscriber-specific incentives, governed by the IncentiveEngine:

* Subscriber discounts — creators set percentage discounts for subscribers on new paid components (the user 1 / user 2 real-estate playbook model)

* Early access tiers — subscribers get 24-hour early access before public listing

* Loyalty pricing — subscribers who have maintained subscription for 12+ months receive a locked-in price regardless of future price increases

* Affiliate commissions — creators can define affiliate link programs; referred buyers tracked by ProviderSystem; commissions automatically paid from kogi-bank

## **7.4 Penalty Mechanisms**

The IncentiveEngine enforces penalties for value-destroying behaviors to maintain the repeated-game equilibrium:

| Behavior | Penalty | Duration |
| :---- | :---- | :---- |
| Contract cancellation (seller-initiated without cause) | –25 Delivery Reliability score; –50 KP; listing visibility reduced for 30 days | 30-day visibility penalty; score impact decays over 12 months |
| Non-payment (buyer-initiated without dispute) | –25 Financial Integrity score; account flagged; escrow mandatory for 6 months | Account flag for 6 months; cleared on 3 clean payments |
| Fraudulent listing (counterfeit asset, misleading scope) | Listing removed; –50 overall reputation score; 30-day suspension | Permanent listing removal; appeal process available |
| Dispute filed and lost (either party) | –10 overall reputation score | Score impact decays over 12 months |
| Repeated late delivery (3+ in 6 months) | –15 Delivery Reliability; Developed tier downgrade | Tier downgrade for 90 days; restored on improved delivery |
| Platform abuse (spam, fake reviews, bot activity) | Account suspension; KP forfeiture; IP-level block after 3 offenses | 1st: warning; 2nd: 30-day suspension; 3rd: permanent |

## **7.5 Market Liquidity Incentives**

Thin markets — categories with few buyers and sellers — can cause market failure. The IncentiveEngine detects thin market conditions (\< 10 active listings in a category, \< 5 active buyers in a period) and activates liquidity incentives:

* Supply-side bonus — new listings in thin categories earn 2× KP for the first 30 days

* First-mover bonus — the first worker to complete a gig in a new category earns a Pioneer badge and 500 KP

* Buyer activation — platform temporarily reduces or waives listing fees in thin categories to increase supply

* Cross-category recommendations — MatchEngine boosts adjacent category workers into thin category searches with a skill-transfer score

* Campaign seeding — platform can seed small initial contributions to new campaigns in thin funding categories to trigger social proof effects

| 8\. ANALYTICS & INTELLIGENCE LAYER |
| :---- |

## **8.1 PortfolioSignal — The Universal Health Metric**

The PortfolioSignal is the four-dimensional score that every platform entity — user, portfolio component, organization, marketplace listing — continuously emits. It is the primary input to the RiskEngine, AnalyticsEngine, MatchEngine, and IncentiveEngine:

| Dimension | Range | Sources | Game Role |
| :---- | :---- | :---- | :---- |
| Productivity | 0–100 | Task completion velocity, throughput, sprint burn rate, gig completion rate | Supply quality signal; drives MatchEngine labor scoring |
| CashFlow | 0–100 | Invoice payment rate, incoming transaction velocity, budget utilization, escrow release rate | Financial health signal; drives IncentiveEngine financial nudges |
| Collaboration | 0–100 | Response rate, co-contributor density, community engagement, dispute rate | Network quality signal; drives Collaborator matching boost |
| Risk | 0–100 (lower is better) | Error rates, overdue items, blocker count, anomaly flags | Risk signal; drives RiskEngine status (green/amber/red) |

## **8.2 PortfolioHealthScore**

| Health Score | H \= 0.30×Productivity \+ 0.30×CashFlow \+ 0.20×Collaboration \+ 0.20×(100–Risk) |
| :---- | :---- |

| Health Status | Score Range | Engine Actions |
| :---- | :---- | :---- |
| Green | ≥ 80 | Standard recommendations; no intervention required |
| Amber | 60–79 | Oba generates OptimizationPlan; specific improvement suggestions surfaced in morning briefing |
| Red | \< 60 | High-priority alert; Oba triggers Stabilization Playbook; opportunity matching temporarily narrowed to lower-risk engagements |

## **8.3 RiskEngine Outputs for Game Mechanics**

The RiskEngine produces RiskOptimizationPlans that directly feed into allocation and matching decisions:

| Risk Condition | Threshold | Allocation / Matching Response |
| :---- | :---- | :---- |
| High risk (risk ≥ 70\) | risk signal | MatchEngine applies risk discount on new high-value matches; IncentiveEngine reduces promoted listing eligibility |
| Low cashflow (cashFlow \< 50\) | cashFlow signal | Oba prioritizes invoice recovery and deal completion in briefing; RFP match score temporarily downweighted (buyer may not pay) |
| Low productivity (productivity \< 50\) | productivity signal | New gig/contract matching paused until sprint workload rebalanced; Oba flags WIP overload |
| Low collaboration (collaboration \< 50\) | collaboration signal | Collaborator suggestions increased in feed; community space recommendations amplified |
| Health status Red | composite | New campaign backing temporarily restricted; escrow terms tightened; reputation tier review triggered |

## **8.4 GraphEngine Integration**

The GraphEngine powers the dependency and impact analysis that makes kogi-game's allocation decisions portfolio-aware — not just point-in-time transactions, but decisions informed by the full graph of what depends on what:

* Critical Path Analysis (CPM) — identifies the longest delivery path through a project's dependency graph; AllocEngine uses critical path nodes to prioritize resource allocation to unblock the schedule

* Impact Report — before approving a new contract or resource allocation, GraphEngine computes the downstream impact on all dependent portfolio components

* Cycle Detection — prevents circular dependencies in multi-party deals and sub-contractor chains that could create allocation deadlocks

* Topological Scheduling — AllocEngine uses topological sort to sequence milestone payments in multi-milestone contracts so that payments respect delivery dependencies

* Change Propagation — when a portfolio component's status changes (delayed, cancelled, completed), GraphEngine propagates the impact to all dependent components and triggers re-matching for affected resource gaps

## **8.5 Streaming Analytics Pipeline**

All game events are streamed through the kogi-engine analytics pipeline in real-time:

| Kafka Topic | Events | Consumer / Purpose |
| :---- | :---- | :---- |
| game.matches | match.computed, match.accepted, match.rejected, match.expired | AnalyticsEngine: match quality tracking; RecommendationEngine: feedback loop update |
| game.allocations | alloc.proposed, alloc.accepted, alloc.failed, alloc.cancelled | AllocEngine: market clearing analysis; IncentiveEngine: completion/cancellation scoring |
| game.bids | bid.placed, bid.accepted, bid.rejected, bid.expired, bid.withdrawn | AllocEngine: auction clearing; PriceEngine: market rate estimation |
| game.deals | deal.initiated, deal.agreed, deal.milestone\_completed, deal.disputed, deal.closed | IncentiveEngine: reputation updates; kogi-bank: escrow instructions |
| game.reputation | score.updated, tier.changed, penalty.applied, reward.credited | UI feed: reputation notifications; MatchEngine: score cache invalidation |
| game.campaigns | campaign.backed, campaign.funded, campaign.failed | IncentiveEngine: backer KP; AllocEngine: campaign allocation execution |
| game.incentives | kp.earned, kp.redeemed, streak.updated, badge.awarded | UI feed: achievement notifications; Oba: briefing cards |

| 9\. TRANSACTION-SPECIFIC GAME DESIGNS |
| :---- |

## **9.1 Bid / Offer Game**

Bids and Offers are the atomic price-discovery instruments of the Kogi Exchange. kogi-game governs the bid/offer lifecycle with mechanism rules that ensure price efficiency and prevent gaming:

| Mechanism | Applies To | Rules |
| :---- | :---- | :---- |
| Open Ascending (English) | Public asset auctions, featured contract awards | Minimum bid increment: 2% of current price or $1 (whichever greater). Auction extension: if bid placed in last 5 minutes, extend by 5 minutes (anti-sniping). Reserve price: seller may set a hidden reserve. |
| Second-Price Sealed (Vickrey) | High-value exclusive contracts, single premium asset sales | All bids submitted simultaneously. Winner \= highest bidder; price \= second-highest bid. Dominant strategy: bid truthfully. |
| Continuous Double Auction | Exchange market — labor, commodities, financial instruments | Order book matching: price-time priority. Market order: execute at best available price. Limit order: execute only at specified price or better. |
| Reverse Auction | Buyer-initiated RFP competition | Seller bids are scope+price packages. Buyer's scoring rubric: price weight \+ quality weight (configurable). Minimum quality threshold must be met; lowest weighted score wins. |
| Dutch Auction | Time-sensitive asset offers, expiring resource listings | Starting price set by seller. Price decrements every N hours by a defined step. First buyer to accept wins. |

## **9.2 Deal Room Game**

The Deal Room is the platform's bilateral negotiation environment. kogi-game provides the Deal Room with a structured game protocol that guides both parties toward agreement without requiring external arbitration:

13. Game opens: Seller posts Proposal (scope \+ price \+ timeline). Buyer opens Deal Room.

14. ZOPA Engine computes price range guidance from historical market data (displayed privately to each party as 'market range').

15. Alternating-offer protocol: Buyer can Accept, Counter, or Request Modification. Each counter-offer resets a response timer.

16. Oba facilitates: drafts structured counter-offers for both parties, highlights ZOPA proximity, and suggests standard deal terms from platform templates.

17. Commitment devices: either party can issue a 'final offer' flag — accepted or negotiation restarts. Prevents endless iteration.

18. Escrow provision: once terms agreed, AllocEngine generates the escrow plan; kogi-bank provisions the escrow account.

19. Game closes: deal signed, escrow funded, project kicks off in kogi-portfolio.

## **9.3 Proposal Scoring**

Proposals submitted to an RFP or open marketplace are scored by the AllocEngine using a configurable scoring rubric. The default rubric weights:

| Scoring Dimension | Default Weight | Buyer Can Adjust? | Signal Source |
| :---- | :---- | :---- | :---- |
| Scope Coverage | 25% | Yes (±10%) | AI-assessed: proposal text coverage of requirement tags |
| Price Competitiveness | 30% | Yes (±15%) | Ratio of bid price to median comparable market rate |
| Reputation Score | 25% | Yes (±10%) | MatchEngine: worker's composite ReputationScore |
| Timeline Commitment | 10% | Yes (±5%) | Milestone schedule vs. buyer's deadline constraints |
| Portfolio Evidence | 10% | Yes (±5%) | Relevance of worker's portfolio components to scope |

Buyers who consistently apply extreme weightings (e.g., 90% price / 10% quality) are flagged by the IncentiveEngine — statistically, extreme price-weighting correlates with higher dispute rates. Oba suggests rebalancing the rubric toward better outcomes.

## **9.4 Campaign Game (Threshold Public Goods)**

Campaigns are assurance contract games — a form of threshold public goods provision where individual contributions are only realized if the collective threshold is met. kogi-game implements this as follows:

20. Campaign creator sets goal G, deadline D, and type (all-or-nothing or keep-what-you-raise)

21. Backers commit contributions c\_i, held in escrow in the Campaign Account in kogi-bank

22. If Σc\_i ≥ G before deadline D: campaign succeeds; funds disbursed per milestone schedule

23. If Σc\_i \< G at deadline D (for all-or-nothing): all c\_i refunded atomically — no backer is worse off

24. Dominant strategy for backers: contribute your true value v\_i if v\_i \> 0 and you believe the campaign will succeed (coordination game — social proof and velocity signals reduce coordination failure)

25. IncentiveEngine activates velocity warnings when campaign pace falls behind the required rate: Oba generates targeted outreach to likely contributors

## **9.5 Cooperative Game (Shapley Distribution)**

When a cooperative or collective completes a joint project or closes a financial period, the AllocEngine distributes the surplus using a Shapley Value approximation:

26. Collect contribution\_vector for each member: hours contributed, revenue generated, capital contributed, skills deployed

27. Normalize contribution dimensions to a common scale (percentile rank within the cooperative)

28. Compute weighted contribution score per member: w\_1 × hours \+ w\_2 × revenue \+ w\_3 × capital \+ w\_4 × skills

29. Run Monte Carlo Shapley approximation (1,000 samples for ≤ 20 members; Group Shapley for larger groups)

30. Present distribution plan to governance for approval vote

31. On vote passage, AllocEngine generates payroll batch in kogi-bank; executed atomically

## **9.6 Resource Sharing Game (Commons Management)**

Collectives operating shared resource pools (compute credits, licenses, mutual aid funds) face a commons management problem — how to allocate finite shared resources fairly without the Tragedy of the Commons. kogi-game applies a governance-aware commons mechanism:

* Usage quota — each member has a per-period quota determined by their contribution history (contributive justice) and need signals

* Excess transfer — unused quota can be transferred to other members within a period; non-transferable across periods (prevents hoarding)

* Contribution multiplier — members who contribute to the commons pool above their quota earn a multiplier on future quota allocations

* Emergency override — members can request emergency access above quota with a governance fast-track approval (simple majority, 24-hour window)

* Replenishment signal — IncentiveEngine alerts when pool is below 20% capacity; auto-sends replenishment campaign to member group

| 10\. PLATFORM ECONOMICS & FEE DESIGN |
| :---- |

## **10.1 Fee Architecture**

Platform fees are themselves a mechanism design problem: fees must be high enough to sustain platform operations and curate quality, but low enough to not deter participation or create incentives to route transactions off-platform. kogi-game designs fees with the following principles: fees scale with transaction value (not fixed), fees are lower for repeat participants (loyalty), and fee revenue is partially recycled into liquidity incentives.

| Transaction Type | Fee Structure | Rationale | Loyalty Discount |
| :---- | :---- | :---- | :---- |
| Gig / Contract (seller fee) | 8% of contract value for new sellers; declining to 4% at Elite tier | Covers platform services \+ escrow; incentivizes quality progression | –1% per tier level (Developing=8%, Established=7%, Trusted=6%, Elite=4%) |
| Gig / Contract (buyer fee) | 2% of contract value (covers payment processing and matching services) | Low buyer fee to maintain demand-side participation | –0.5% at Trusted+; waived at Elite |
| Marketplace listing sale | 6% of sale price | Platform facilitation \+ discovery services | –1.5% at Elite tier |
| Exchange trade | 0.5% of trade value (both sides) | Order book maintenance, settlement, anti-fraud | Flat rate; reduced by 50% for KP redemption |
| Campaign (success fee) | 5% of successfully raised funds | Campaign hosting \+ disbursement services | –1% for cooperative / collective entities |
| Campaign (equity) | 3% \+ 0.5% carried interest on exit | Reflects higher-value financial facilitation | Fixed; non-discountable (regulatory reasons) |
| Promoted listing | KP-based or $5–$50/week based on category competition | Attention allocation market | KP redemption available |
| Subscription listing fee | 1% of recurring subscription revenue | Ongoing platform services for subscription management | Waived for Elite tier |

## **10.2 Market Rate Engine (PriceEngine)**

The PriceEngine maintains a continuously-updated model of market rates for every skill category, resource type, and asset class on the platform. It is used by the ZOPA Engine, MatchEngine, AllocEngine, and Oba to provide pricing guidance to all participants:

| Rate Category | Sources | Output |
| :---- | :---- | :---- |
| Skill / Labor Rates | Completed contract prices by skill tag and complexity; worker rate cards; external benchmarks (Upwork, Fiverr integration data) | Median, P25, P75 rates per skill × complexity × region |
| Asset Prices | Completed marketplace sales by asset type and category; listing price history | Price distribution per asset category; comparable recent sales |
| Resource Rates | Platform resource pricing; third-party provider pricing (compute, licenses) | Unit cost by resource type |
| Financial Returns | Completed campaign outcomes, equity exit returns, revenue-share performance | Expected return distribution by instrument type and sector |

| Market Rate (skill s, complexity c) | rate(s,c) \= median(completed\_contracts where skill=s and complexity\_band=c, last\_90\_days) |
| :---- | :---- |

## **10.3 Fee Redistribution**

A portion of platform fee revenue is recycled into the incentive economy to sustain liquidity and grow participation:

* Liquidity incentive pool (15% of fee revenue) — allocated as KP bonuses to participants in thin markets and new categories

* Newcomer subsidy pool (5% of fee revenue) — reduced fees and matching bonuses for users completing their first 3 transactions

* Community fund (5% of fee revenue) — administered by a platform governance council; distributed as grants for open community contributions (templates, playbooks, public datasets)

* Cooperative bonus (5% of fee revenue) — additional fee discounts for formally incorporated cooperative entities, recognizing their positive externality contribution to the platform ecosystem

| 11\. OBA AI GAME AGENT |
| :---- |

## **11.1 Oba as Market Advisor**

Oba, the Kogi Platform's AI assistant, is the primary interface through which kogi-game's mechanism outputs are delivered to users. Oba does not make allocation decisions — those are made by the AllocEngine and governed by the mechanism rules specified in this document — but it explains, facilitates, and implements those decisions in user-facing natural language. Oba is the game's 'referee announcer': it tells each player what the game state is, what their options are, and what the likely consequences of each option are.

## **11.2 Game Intelligence Workflows**

### **11.2.1 Match & Opportunity Briefing**

Each morning, Oba composes a personalized match briefing for each user, driven by overnight MatchEngine results:

* Top-3 new opportunity matches (gigs, contracts, RFPs) ranked by composite match score — with explanation ('Skills match 85%, Budget compatible, New client with 4.8 reputation')

* Top-3 new asset/resource matches — things to buy, license, or follow based on current project context

* Investor suggestion cards for users with investment preferences and available capital

* Collaborator suggestions for users with active projects in Amber or Red health status

### **11.2.2 ZOPA Facilitation in Deal Room**

When a user enters a Deal Room, Oba activates as negotiation facilitator:

* Computes and presents the ZOPA range: 'Based on comparable deals, market rate for this scope is $X–$Y per milestone'

* Drafts structured counter-offers in the user's voice when they specify intent in plain language

* Tracks offer history and highlights trajectory: 'You are $200 apart — typically deals at this stage close within 2 rounds'

* Suggests standard deal terms (payment schedule, milestone definitions, revision limits) from platform templates

* Flags deal terms that statistically correlate with disputes: 'Scope as written has led to disputes in 23% of similar deals — suggest adding deliverable acceptance criteria'

### **11.2.3 Proposal Drafting & RFP Response**

For sellers responding to RFPs, Oba auto-drafts competitive proposals calibrated to the buyer's scoring rubric:

* Reads the RFP requirements and buyer's scoring weights

* Pulls relevant portfolio components, past work evidence, and skill evidence to populate the proposal

* Computes a Price Competitiveness score: 'Your draft price is at the 45th percentile — competitive but room to price higher given your reputation tier'

* Suggests scope clarifications where the RFP is ambiguous — preventing future disputes

### **11.2.4 Campaign Intelligence & Velocity Coaching**

For campaign creators, Oba runs a continuous velocity monitoring loop:

* Tracks contribution rate vs. required pace to meet goal by deadline

* Predicts success probability: 'At current pace, 67% chance of reaching goal — need to accelerate by $X/day'

* Identifies high-probability backers from social graph and platform data — drafts personalized outreach

* Suggests campaign boosts (KP redemption for featured placement) when velocity drops below critical threshold

### **11.2.5 Incentive Status & KP Management**

Oba tracks and surfaces each user's incentive status proactively:

* Morning briefing: KP balance, active streaks, next milestone to next reputation tier, pending badge progress

* Post-transaction: 'You earned 125 KP for completing this contract — your Delivery Reliability score increased to 94'

* Tier upgrade notification: 'You have reached Trusted tier — your platform fee has reduced from 7% to 6% effective immediately'

* Streak at-risk warning: 'Your activity streak is at 6 weeks — complete one transaction this week to keep your ×1.30 KP multiplier'

### **11.2.6 Risk-Adjusted Opportunity Guidance**

Oba integrates RiskEngine output into opportunity guidance — steering users away from commitments that would overload already-stressed portfolios:

* If portfolio health is Amber: 'Your current workload is near capacity — this new gig would push your sprint utilization to 130%. Recommend declining or deferring to next sprint.'

* If cashFlow signal is low: 'Two invoices are overdue totaling $3,400. Recommend recovering these before taking on new unpaid work.'

* If a counterparty has low reputation: 'This buyer's Financial Integrity score is 35 — historically 40% of contracts with this score tier result in late payment. Escrow strongly recommended.'

| 12\. DATA ARCHITECTURE |
| :---- |

## **12.1 Core Tables**

| Table | Description | Key Columns |
| :---- | :---- | :---- |
| reputation\_scores | Per-user/org multi-dimensional reputation records | id, entity\_id, entity\_type, delivery, quality, communication, community, financial, composite, tier, updated\_at |
| kogi\_points | KP ledger — earn/redeem events | id, entity\_id, event\_type, amount, balance\_after, reference\_id, reference\_type, timestamp |
| match\_results | MatchEngine output records | id, query\_id, subject\_a\_id, subject\_b\_id, score, confidence, explanation JSONB, mechanism\_type, accepted, timestamp |
| allocations | AllocEngine decisions | id, mechanism\_type, auction\_id, winner\_id, item\_id, clearing\_price, scoring\_breakdown JSONB, status, executed\_at |
| auction\_bids | All bids submitted across all auction types | id, auction\_id, bidder\_id, amount, bid\_type, proxy\_max JSONB, status, submitted\_at, withdrawn\_at |
| auction\_offers | All seller offers | id, listing\_id, seller\_id, price, offer\_type, bundle\_items JSONB, auction\_config JSONB, status |
| deal\_negotiations | Deal Room negotiation state | id, deal\_id, round\_number, party\_id, action\_type (offer/counter/accept/withdraw), terms JSONB, zopa\_range JSONB, timestamp |
| incentive\_events | All incentive events (earn, penalty, streak, badge) | id, entity\_id, event\_type, kp\_delta, reputation\_delta, badge\_id, streak\_id, reference\_id, timestamp |
| streaks | Active streak records per entity | id, entity\_id, streak\_type, current\_count, last\_activity\_at, multiplier, expires\_at, status |
| badges | Badge award records | id, entity\_id, badge\_type, awarded\_at, evidence JSONB |
| market\_rates | PriceEngine rate snapshots | id, category, skill\_tag, complexity\_band, median\_rate, p25\_rate, p75\_rate, sample\_size, computed\_at |
| personalization\_profiles | PersonalizationEngine user state | id, user\_id, persona, preference\_vector JSONB, dominant\_categories JSONB, content\_config JSONB, segments JSONB, updated\_at |

## **12.2 Redis Hot-Path Caches**

| Cache Key Pattern | TTL | Content |
| :---- | :---- | :---- |
| match:user:{id}:opportunities | 5 min | Top-20 current opportunity matches for user — invalidated on new listing or profile update |
| match:listing:{id}:candidates | 2 min | Top-50 worker candidates for a listing — invalidated on new worker availability signal |
| reputation:{entity\_id} | 10 min | Composite ReputationScore and tier — invalidated on any reputation event |
| kp:balance:{entity\_id} | 1 min | Current KP balance — invalidated on any KP earn/redeem event |
| market\_rate:{skill}:{complexity} | 1 hour | Median market rate — refreshed hourly by PriceEngine batch |
| alloc:auction:{id}:book | 30s | Live auction state — current bids, clearing price — invalidated on every bid |
| persona:{user\_id} | 30 min | Current persona and ContentDeliveryConfig — invalidated on profile update |
| zopa:{deal\_id} | deal lifetime | Computed ZOPA range for active Deal Room — invalidated on each negotiation round |

## **12.3 Technology Stack**

| Component | Technology | Role | Notes |
| :---- | :---- | :---- | :---- |
| kogi-game service | Go | Primary REST/gRPC API; orchestrates all mechanism calls | Event producer to game.\* Kafka topics |
| AllocEngine core | Rust (kogi-modules) | High-performance auction clearing, DA algorithm, Shapley approximation | Embedded in kogi-game via FFI |
| MatchEngine | Scala 3 (kogi-engine) | Multi-dimensional scoring, similarity computation | gRPC port 9100; stateful user profiles |
| RecommendationEngine | Scala 3 (kogi-engine) | Collaborative \+ content-based filtering, hybrid scoring | In-memory profile store; Kafka for feedback |
| PersonalizationEngine | Scala 3 (kogi-engine) | A/B experiments, UCB1 bandit, persona lifecycle | Deterministic assignment via hash |
| IncentiveEngine | Scala 3 (kogi-engine) | KP accounting, reputation scoring, streak tracking | Writes to game.reputation Kafka topic |
| PriceEngine | Go service | Market rate estimation; ZOPA computation | Reads from game.bids and game.allocations |
| PostgreSQL | Primary DB | Persistent state for all game entities | Partitioned by entity\_id for large tables |
| Redis | Cache | Hot-path balance, match, and auction state | LRU eviction; TTL-based invalidation |
| Kafka | Streaming | All game events streamed for analytics and cross-service coordination | game.\* topics; consumed by kogi-engine |

| 13\. UX ARCHITECTURE & KEY GAME FLOWS |
| :---- |

## **13.1 Core UI Surfaces for kogi-game**

| Surface | Game Features Exposed |
| :---- | :---- |
| **Home Dashboard** | Personalized opportunity feed; reputation tier badge; KP balance; streak status; Oba briefing cards with match and incentive updates |
| **Marketplace Browse** | QScore-ranked listings; match-score badges on recommended items; promoted placements (KP/paid); ZOPA hints on listing prices |
| **Exchange Order Book** | Real-time bid/offer ladder; auction countdown; proxy bid controls; clearing price trajectory; liquidity depth chart |
| **Deal Room** | ZOPA panel (market rate range); offer/counter-offer history; Oba negotiation assistant; milestone scheduler; escrow status |
| **RFP / Proposal Center** | Proposal scoring rubric (buyer); proposal competitiveness score (seller); ranked proposal shortlist; one-click accept |
| **Campaign Dashboard** | Velocity tracker; success probability gauge; backer list; Oba outreach center; milestone disbursement controls |
| **Reputation Center** | Dimension-by-dimension score breakdown; tier progress bar; badge showcase; KP ledger; streak log |
| **Incentive Rewards** | KP balance and earn history; available redemptions; streak status; badge library; affiliate commission tracker |
| **Analytics / OKR** | PortfolioSignal gauges; health trend charts; risk recommendations; market rate benchmarks; allocation efficiency report |
| **Oba Chat Interface** | Natural language access to all mechanism outputs: 'Find me 3 workers for my React project under $80/hr', 'How competitive is my proposal?' |

## **13.2 End-to-End Game Flows**

### **13.2.1 Worker Wins a Contract via Reverse Auction RFP**

32. Buyer posts RFP in kogi-marketplace with scope, budget, and scoring rubric (30% price / 40% skills / 20% reputation / 10% timeline)

33. MatchEngine broadcasts to top-50 worker matches; Oba notifies each in next morning briefing

34. Worker reviews RFP in their Proposal Center; Oba drafts a competitive proposal with market-rate pricing guidance

35. Worker submits proposal; AllocEngine scores it against the buyer's rubric and ranks it against all proposals

36. Buyer reviews ranked shortlist; Oba summarizes top-3 proposals with key trade-offs

37. Buyer accepts top-ranked proposal; Deal Room opens; milestone schedule negotiated

38. Deal agreed; kogi-bank provisions escrow; project appears in worker's kogi-portfolio

39. On deal close: worker earns KP \+ Delivery Reliability update; buyer earns KP \+ Financial Integrity confirmation

### **13.2.2 Asset Creator Builds a Subscription Ecosystem**

40. User 1 creates Real Estate Investment Playbook as a Portfolio Asset in kogi-portfolio

41. Publishes to kogi-marketplace: subscription tier ($19/mo) \+ one-time purchase ($99)

42. RecommendationEngine surfaces the playbook to User 2 via content-based filtering (shared real estate interest tags)

43. User 2 subscribes; playbook added to their portfolio; User 1 earns KP \+ subscription revenue in kogi-bank

44. User 1 publishes an updated playbook edition; IncentiveEngine applies subscriber discount (20% for 12+ month subscribers)

45. Oba notifies User 2 of the new edition with the personalized discount applied

46. User 2 purchases; referral-chain tracked if User 2 referred a User 3 with an affiliate link

### **13.2.3 Cooperative Closes a Sprint and Distributes Patronage**

47. Cooperative's sprint closes; all task completions recorded in kogi-portfolio

48. AnalyticsEngine computes per-member contribution signals (hours, quality scores, deliverables)

49. AllocEngine runs Monte Carlo Shapley approximation on contribution vectors

50. Oba presents distribution plan to cooperative governance: 'Proposed patronage distribution for Sprint 12: Jordan 34%, Alex 28%, Sam 22%, Taylor 16%'

51. Governance vote initiated; members vote via kogi-organizations; quorum reached

52. Vote passes; kogi-bank executes batch payroll to all member Payroll Wallets atomically

53. All members earn Governance Participation KP; distribution logged in cooperative's ledger

| 14\. OPEN ITEMS & FUTURE CONSIDERATIONS |
| :---- |

## **14.1 Near-Term Open Items**

| Item | Description |
| :---- | :---- |
| **Combinatorial Auction Solver** | Combinatorial bids (bundles of resources) require a winner determination algorithm that is NP-hard in general. Implement an approximate solver (LP relaxation \+ branch-and-bound for small N; greedy approximation for large N) with a bid complexity limit to keep computation tractable. |
| **Shapley Approximation Calibration** | Validate Monte Carlo Shapley convergence against exact computation for small cooperatives (≤ 10 members). Establish confidence intervals for approximation accuracy. Define policy for edge cases where contribution vectors are near-uniform. |
| **Multi-Jurisdiction Compliance for Auctions** | Real-money auctions with equity instruments may require broker-dealer licensing in certain jurisdictions. Mechanism design must comply with FINRA and SEC Regulation CF for equity crowdfunding above threshold amounts. Legal review required before production deployment of equity campaigns. |
| **KogiPoints Valuation & Anti-Gaming** | KP must not become a de facto currency that triggers money transmission licensing. Design redemption mechanics to keep KP as non-transferable, platform-internal credits only. Anti-gaming controls needed for fake review KP farming. |
| **MatchEngine Cold-Start for New Listings** | New listings have no interaction history. Current cold-start relies on global popularity signals. Implement category-median profile seeding and seller's historical listing performance as cold-start priors. |
| **Real-Time ZOPA Update** | ZOPA computation currently uses a static market rate snapshot. High-volume exchange categories need continuous ZOPA updates as new deals close. Streaming ZOPA update from game.allocations Kafka topic needed. |
| **Federated Game Mechanics** | Cross-platform (ShangoOS) game mechanics — how kogi-game interacts with ume's organizational games and qala's solution games — require a defined Federated Mechanism Protocol. Specify the cross-platform incentive alignment rules for jiwe seed loaded environments. |
| **Oba Writeback Protocol for Mechanisms** | Oba generates allocation recommendations and draft proposals but requires a defined writeback protocol to submit bids, accept proposals, and execute game actions on behalf of users with pre-approved automation rules — without bypassing PermissionTier checks. |

## **14.2 Phase 4+ Game Mechanics**

* Prediction markets — workers and investors bet on project outcomes (will this campaign fund? will this gig deliver on time?); outcomes calibrate RiskEngine priors

* Mechanism tournaments — quarterly platform-wide competitions where participants compete on defined efficiency metrics (fastest gig delivery, highest campaign funding velocity) for prize pool rewards

* On-chain mechanism execution — migrate high-value auction clearing and cooperative distribution to smart contracts for trustless, auditable execution without platform intermediation

* Dynamic fee optimization — replace fixed fee tiers with Myerson-optimal dynamic fees computed per-transaction based on elasticity estimates from PriceEngine

* Cross-platform reputation portability — export Kogi reputation signals to external platforms (Upwork, Fiverr, GitHub) via a verifiable credential standard

* Federated matching — MatchEngine operates across ShangoOS federates: a gig posted in ume is matched by kogi's MatchEngine and fulfilled by a qala solution component

* Simulation environment (oru integration) — run counterfactual simulations of allocation and incentive rule changes in oru before deploying to production, using historical kogi-game data as the simulation substrate

* AI-autonomous market making — Oba-class agents post algorithmic bids and offers to thin markets to maintain price discovery without human participation, acting as algorithmic market makers in underserved categories

© 2026 Kogi Platform  —  kogi-game System Design Document  —  Confidential