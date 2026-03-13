  
**KOGI PLATFORM**

**AI Agent &**  
**Assistant Workflows**

*How intelligent agents help users work, earn, govern, and grow across the Kogi platform*

| 12 AI Workflows | 10 Platform Modules | 60+ Agent Actions | 1 Unified Context |
| :---: | :---: | :---: | :---: |

Version 1.0  ·  Kogi Platform  ·  2025

# **Introduction**

The Kogi platform is built around a single organizing idea: that AI agents and assistants should remove friction from every part of a user's professional life — from managing projects and finances, to hiring talent, closing deals, and governing cooperative organizations.

Unlike conventional productivity tools that require users to manually navigate between modules, Kogi's AI layer actively monitors, connects, and acts across the entire platform on the user's behalf. Agents don't simply answer questions — they proactively surface insights, trigger cross-module workflows, draft documents, and execute tasks the moment they become relevant, requiring user approval only for consequential actions.

This document describes every major AI agent and assistant workflow across Kogi's modules: what triggers each workflow, how it executes, what tools the agent calls, and the concrete value it delivers.

## **Core Design Principles**

| Cross-module awareness | The AI holds a unified context window spanning all modules simultaneously — project status, wallet balance, open invoices, OKR progress, and governance votes are all visible at once. Problems that cross module boundaries are caught before the user notices them. |
| :---- | :---- |
| **Proactive, not reactive** | Agents run on a continuous monitoring loop. They surface time-sensitive issues and present suggested actions the moment something requires attention — not when the user happens to open the relevant screen. |
| **Tool-native execution** | When a user approves an action, the agent executes it natively within Kogi: sending inquiries, creating invoices, drafting governance proposals, updating sprint boards, or triggering escrow releases. No copy-paste, no tab-switching. |
| **Approval-first by default** | Every consequential action — sending money, submitting proposals, contacting external parties — requires explicit user confirmation. Agents suggest and prepare; users decide. |
| **Full audit trail** | All agent actions are logged with timestamps, tool names, inputs, and outcomes. Users can inspect or replay any agent decision from the History panel at any time. |
| **Graceful degradation** | If an agent action fails (e.g., no Marketplace contractors match), it clearly reports the failure and proposes an alternative path, rather than silently stopping. |

| WORKFLOW 1  ·  MORNING PORTFOLIO BRIEFING |
| :---- |

## **Morning Portfolio Briefing**

Every morning when a user opens Kogi, the AI Agent automatically runs a portfolio-wide review and surfaces the most important items requiring attention that day. This replaces the need to manually check every module: the AI acts as a chief of staff who has already read everything and distilled it to a prioritized action list.

This is the core behaviour shown in the AI Agent screen — the chat interface on the left shows the agent's proactive briefing, the Portfolio Context sidebar on the right shows what data it read, and the Suggested Actions panel shows what it recommends doing next.

|  | EXAMPLE SCENARIO *Jordan opens Kogi at 9:02 AM. Before Jordan types anything, the agent has already reviewed 4 active projects, scanned 6 open invoices, cross-referenced OKR targets, found 2 Stripe-certified developers on the Marketplace, and identified a governance vote closing in 18 hours. It presents a five-item briefing with one-click action buttons for each.* |
| :---- | :---- |

### **Step-by-Step Agent Execution**

| \# | PHASE | WHAT THE AGENT DOES | TOOLS INVOKED | OUTCOME |
| :---: | :---- | :---- | :---- | :---- |

| 1 | Portfolio scan | Reads all active projects, sprint statuses, story blockers, and milestone dates across the Work Board module | *read\_portfolio, read\_wbs* | Full project snapshot |
| :---: | :---- | :---- | :---- | :---- |

| 2 | Financial review | Checks wallet balance, scans open invoices for overdue items, reviews escrow holds, and flags pending payouts | *read\_exchange, read\_invoices* | Financial health score |
| :---: | :---- | :---- | :---- | :---- |

| 3 | OKR delta check | Compares current revenue and delivery metrics against quarterly OKR targets, calculates velocity and projected shortfall | *read\_okrs, calculate\_velocity* | Risk flags raised |
| :---: | :---- | :---- | :---- | :---- |

| 4 | Governance scan | Checks for active votes, identifies any closing within 48 h where the user has not yet voted, summarises each proposal | *read\_governance, summarise\_proposals* | Vote alerts queued |
| :---: | :---- | :---- | :---- | :---- |

| 5 | Marketplace match | Searches for contractors or resources that could resolve any blockers detected in step 1 | *search\_marketplace, rank\_candidates* | Candidates found |
| :---: | :---- | :---- | :---- | :---- |

| 6 | Briefing composed | Ranks all findings by urgency, composes a prioritised briefing with 3–5 actionable insights and a suggested action for each | *create\_brief, rank\_actions* | Briefing delivered |
| :---: | :---- | :---- | :---- | :---- |

| 7 | User approves | User reviews suggested actions and approves or modifies. Agent executes approved actions immediately in the background | *execute\_actions, log\_audit* | Actions executed |
| :---: | :---- | :---- | :---- | :---- |

### **What the Agent Surfaces**

* **Blocked stories** — Tasks stuck for more than 24 hours, cross-referenced with available Marketplace talent who could unblock them

* **Overdue invoices** — Any invoice past due date, with a pre-drafted payment reminder email ready to send in one click

* **OKR velocity warnings** — When current pace will not reach quarterly targets, with specific lever suggestions and estimated impact for each

* **Governance deadlines** — Votes closing within 48 hours where the user has not voted, with proposal summary and quick-vote buttons

* **Expiring contracts** — Legal agreements within 30 days of expiry, with renewal draft ready for review

* **Escrow release prompts** — Delivered assets awaiting fund release confirmation, with verification checklist pre-completed

| WORKFLOW 2  ·  SPRINT UNBLOCKING VIA LABOR MARKET & MARKETPLACE |
| :---- |

## **Sprint Unblocking via Labor Market & Marketplace**

When a story on the Work Board is flagged as blocked, the AI agent goes to work: it analyses what skill or resource is missing, searches the Marketplace and Labor Market for matching contractors, prepares personalised outreach, and presents the user with a ready-to-send shortlist. This workflow directly connects the Work Board, Marketplace, and Exchange modules.

|  | EXAMPLE SCENARIO *The 'Set up Stripe payment gateway' story has been blocked for 3 days. The agent reads the story description, determines a Stripe-certified backend developer is needed, finds 2 matching contractors on the Marketplace, and prepares personalised inquiry messages with the project brief pre-attached. Jordan approves with one click and both inquiries are sent.* |
| :---- | :---- |

### **Step-by-Step Agent Execution**

| \# | PHASE | WHAT THE AGENT DOES | TOOLS INVOKED | OUTCOME |
| :---: | :---- | :---- | :---- | :---- |

| 1 | Blocker detected | Monitors Work Board continuously. A story in 'Blocked' status for longer than the configured threshold triggers the workflow | *monitor\_board, read\_blockers* | Alert raised |
| :---: | :---- | :---- | :---- | :---- |

| 2 | Blocker analysis | Reads the story description, sub-tasks, and comments to understand what skill, credential, or resource is missing | *read\_story, parse\_requirements* | Skill profile built |
| :---: | :---- | :---- | :---- | :---- |

| 3 | Candidate search | Searches the Labor Market and Marketplace for contractors matching the required skill, filtered by availability and rating | *search\_marketplace, filter\_candidates* | Shortlist ranked |
| :---: | :---- | :---- | :---- | :---- |

| 4 | Inquiry drafts | Prepares personalised inquiry messages for the top 2–3 candidates, attaching relevant project context and estimated scope | *draft\_inquiry, attach\_brief* | Messages ready |
| :---: | :---- | :---- | :---- | :---- |

| 5 | User sends | User sees candidate cards and pre-drafted messages in the AI Agent panel and approves. Agent sends all inquiries simultaneously | *send\_inquiries, create\_orders* | Orders created |
| :---: | :---- | :---- | :---- | :---- |

| 6 | Escrow prepared | When a candidate responds and the user selects them, the agent prepares the escrow agreement and payment terms for final review | *prepare\_escrow, draft\_contract* | Deal ready to close |
| :---: | :---- | :---- | :---- | :---- |

### **Supported Blocker Types**

* **Missing credentials or API keys** — Agent drafts a targeted client-facing request email with the specific technical details needed

* **Skill gap on the current team** — Searches Labor Market for available contractors matching the exact technology stack required

* **Awaiting client approval** — Drafts a follow-up message and schedules an automatic reminder if no response is received within the configured window

* **Infrastructure or tooling issue** — Searches Community spaces and Dev DAO for members who have solved similar problems and can provide guidance

* **Legal or compliance hold** — Routes to the Legal module, flags the relevant contract or compliance item, and suggests appropriate legal consultants from the Labor Market

| WORKFLOW 3  ·  INVOICE & PAYMENT RECOVERY |
| :---- |

## **Invoice & Payment Recovery**

The AI agent monitors all open invoices in the Exchange module and proactively manages overdue payments — from gentle automated reminders through to formal escalation drafts — without the user having to manually track due dates or chase clients.

|  | EXAMPLE SCENARIO *Invoice INV-2024-108 for $2,000 from DesignCo is 14 days overdue. The agent drafts a professional payment reminder citing the original due date, calculates the cash-flow impact on the current sprint budget, flags the overdue amount against the Q3 revenue OKR, and offers to log a follow-up task on the Work Board — all surfaced to Jordan in one briefing card.* |
| :---- | :---- |

### **Step-by-Step Agent Execution**

| \# | PHASE | WHAT THE AGENT DOES | TOOLS INVOKED | OUTCOME |
| :---: | :---- | :---- | :---- | :---- |

| 1 | Overdue detection | Scans all invoices daily. Any invoice past due date plus the configured grace period is escalated to the briefing queue | *scan\_invoices, check\_due\_dates* | Overdue list built |
| :---: | :---- | :---- | :---- | :---- |

| 2 | Context gathering | Pulls the client's relationship history, previous payment behaviour, and the associated project status to calibrate reminder tone | *read\_client\_history, read\_project* | Tone profile set |
| :---: | :---- | :---- | :---- | :---- |

| 3 | Reminder drafted | Generates a professional reminder email with invoice details, amount outstanding, original due date, and a direct payment link | *draft\_email, attach\_invoice* | Email ready |
| :---: | :---- | :---- | :---- | :---- |

| 4 | Cash-flow impact | Calculates the overdue amount's impact on the current sprint budget and maps it against the active OKR revenue target | *calculate\_cashflow, update\_okr\_projection* | Impact surfaced |
| :---: | :---- | :---- | :---- | :---- |

| 5 | Escalation path | If no payment within 7 days of the first reminder, prepares a more formal second notice and flags the debt for legal review in the Legal module | *schedule\_followup, flag\_legal* | Escalation queued |
| :---: | :---- | :---- | :---- | :---- |

| 6 | Payment confirmed | When payment arrives via Stripe or wire, the agent logs the transaction, updates the Exchange wallet balance, and closes the invoice automatically | *confirm\_payment, update\_balance, close\_invoice* | Invoice closed |
| :---: | :---- | :---- | :---- | :---- |

| WORKFLOW 4  ·  COOPERATIVE GOVERNANCE ASSISTANT |
| :---- |

## **Cooperative Governance Assistant**

For cooperative and DAO members, the AI assistant handles the full governance lifecycle — drafting proposals, monitoring quorum, summarising arguments, sending member notifications, and executing passed proposals. This turns governance from an administrative burden into a near-automatic process.

|  | EXAMPLE SCENARIO *Jordan needs to propose allocating $8,000 from Q4 surplus for developer tooling. The agent reads the current treasury balance, checks charter amendment requirements, drafts a structured proposal with full budget breakdown and rationale, and — once submitted — notifies all 35 voting members with a summary and deadline. It then monitors the vote in real time and triggers execution the moment quorum is reached.* |
| :---- | :---- |

### **Step-by-Step Agent Execution**

| \# | PHASE | WHAT THE AGENT DOES | TOOLS INVOKED | OUTCOME |
| :---: | :---- | :---- | :---- | :---- |

| 1 | Proposal drafting | User describes intent in plain language. Agent structures it into a formal proposal with rationale, budget breakdown, and charter cross-references | *read\_charter, draft\_proposal* | Draft ready |
| :---: | :---- | :---- | :---- | :---- |

| 2 | Treasury check | Verifies available funds, checks reserve requirements, and flags any conflicts with existing capital commitments before submission | *read\_treasury, check\_reserves* | Feasibility confirmed |
| :---: | :---- | :---- | :---- | :---- |

| 3 | Conflict detection | Checks whether the proposal conflicts with existing charter rules, active proposals, or scheduled distributions | *scan\_proposals, read\_charter* | Conflicts cleared |
| :---: | :---- | :---- | :---- | :---- |

| 4 | Member notification | On submission, composes and sends a notification to all voting members with proposal summary, key figures, and the closing deadline | *notify\_members, create\_digest* | Members notified |
| :---: | :---- | :---- | :---- | :---- |

| 5 | Vote monitoring | Tracks votes in real time, sends reminders to members who have not voted within 24 hours of the deadline, reports quorum status in briefing | *monitor\_votes, send\_reminders* | Quorum tracked |
| :---: | :---- | :---- | :---- | :---- |

| 6 | Execution | If the proposal passes, initiates execution: releases treasury funds, updates member equity records, logs all actions to the immutable Audit Trail | *execute\_proposal, update\_records, write\_audit* | Executed & logged |
| :---: | :---- | :---- | :---- | :---- |

### **Additional Governance Capabilities**

* **Distribution calculation** — When a profit-share proposal passes, calculates per-member amounts based on equity stakes and prepares a payment batch for one-click execution

* **Charter amendment tracking** — Monitors charter versions and automatically flags when a proposed action requires a charter amendment before it can proceed

* **Abstention follow-up** — Sends personalised messages to abstaining members offering to answer questions, increasing informed participation in future votes

* **Historical precedent search** — Before drafting a new proposal, the agent searches past governance decisions to surface relevant precedents the drafter should be aware of

| WORKFLOW 5  ·  INVESTOR OUTREACH & CAPITAL RAISE |
| :---- |

## **Investor Outreach & Capital Raise**

The Investor Outreach module's AI assistant manages the full fundraising funnel — from identifying target investors and drafting personalised outreach, to tracking pipeline stages, prompting follow-ups, and preparing materials for due diligence — so founders can focus on relationships, not administration.

|  | EXAMPLE SCENARIO *Sequoia Seed has moved from 'Engaged' to the term sheet stage. The agent auto-drafts a follow-up email referencing their portfolio company Stripe and attaching the latest updated financial model. It schedules a founder call in the calendar, moves the pipeline card, and flags that the Data Room is missing a signed LOI — all surfaced as a single action cluster awaiting Jordan's one-click approval.* |
| :---- | :---- |

### **Key AI Capabilities in Capital Raise**

* **Investor profile matching** — Cross-references the user's stage, sector, and raise size against investor preference data to suggest the highest-fit targets and prioritise outreach

* **Personalised outreach drafts** — Generates individual cold-outreach emails that reference each investor's known portfolio companies and stated thesis — not generic templates

* **Pipeline stage automation** — Moves investor records through pipeline stages (Awareness → Engaged → Diligence → Term Sheet → Closed) based on email replies and calendar events

* **Material readiness checks** — Scans the Outreach Materials checklist and flags incomplete items that would block a due diligence process before it starts

* **Follow-up scheduling** — Sets automatic reminders and drafts contextual follow-up messages if an investor goes silent for more than the configured number of days at any pipeline stage

* **Round progress reporting** — Tracks percentage funded against the round target in real time, calculates days remaining, and forecasts the close date based on current velocity

* **Data room preparation** — Compiles all ready materials into a structured data room outline and flags which documents still need review before sharing with an investor

| WORKFLOW 6  ·  COMMUNITY & SHOWCASE CONTENT AMPLIFICATION |
| :---- |

## **Community & Showcase Content Amplification**

The AI assistant helps users grow their presence on the Kogi Community and Showcase by drafting posts, suggesting the right spaces to share in, analysing engagement performance, and identifying collaboration opportunities — turning project completions into community momentum automatically.

|  | EXAMPLE SCENARIO *Jordan finishes the Brand Identity System project. The AI assistant drafts a Showcase post with project highlights and visual descriptions, suggests three relevant hashtags, identifies four community members who have engaged with similar design work and might want to collaborate, and recommends posting at 10 AM on Tuesday for peak engagement. Jordan approves, and the post and connection requests are sent simultaneously.* |
| :---- | :---- |

### **Key AI Capabilities in Community**

* **Post drafting** — Given a project milestone or update, generates a compelling community post with appropriate tone, relevant tags, and a clear call to action

* **Showcase optimisation** — Reviews portfolio items in Community Showcase and suggests improvements to title, description, and tags to maximise discoverability in search

* **Collaboration matching** — Identifies community members whose skills or interests complement the user's current projects and drafts warm introduction messages

* **Engagement analytics** — Monitors post reactions, comments, and profile views and reports which content types and posting times drive the most engagement for the specific user

* **Space recommendations** — Analyses active project types and suggests which community spaces (Dev DAO, Design Collective, Finance Coops, etc.) the user should join or be more active in

* **DM triage** — Categorises incoming DMs as collaboration requests, client inquiries, or general messages — drafts reply suggestions for high-priority ones and flags time-sensitive ones in the morning briefing

| WORKFLOW 7  ·  IDEA-TO-OUTCOME PIPELINE |
| :---- |

## **Idea-to-Outcome Pipeline**

The Studio's Idea-to-Outcome workflow is the most AI-intensive on the platform. The assistant accompanies an idea through every stage gate — from raw concept to published, monetisable asset — generating specifications, user stories, test cases, and marketplace listings at each transition point.

|  | EXAMPLE SCENARIO *Jordan captures 'AI context window for portfolio analytics' as a raw idea. Over 34 days the agent expands it into a concept spec, writes 6 user stories, creates a 22-case test suite for the MVP, schedules a validation demo, and ultimately publishes it as a Community Showcase asset with a marketplace price benchmarked against 4 comparable listings.* |
| :---- | :---- |

### **AI Actions at Each Stage Gate**

| Idea  →  Concept | Generates 3 alternative framings of the raw idea with target user and problem statement for each; surfaces comparable projects already on the Showcase to avoid duplication |
| :---- | :---- |
| **Concept  →  Prototype** | Produces a full structured spec: problem statement, success metrics, 6 user stories, acceptance criteria, and a preliminary data model |
| **Prototype  →  MVP** | Reviews the prototype description against the spec, flags UX gaps and missing edge-case handling, suggests A/B test scenarios and links relevant design system components |
| **MVP  →  Solution** | Generates a full test case suite covering happy path, edge cases, failure modes, and regression scenarios derived directly from the approved spec |
| **Solution  →  Asset** | Compares solution metrics against comparable published assets, recommends a pricing tier, and flags any IP or licensing issues before publication |
| **Asset  →  Published** | Drafts the Marketplace listing, the Community Showcase post, suggests optimal tags for discoverability, and schedules the announcement for peak Community engagement hours |

| WORKFLOW 8  ·  LEGAL & COMPLIANCE MONITORING |
| :---- |

## **Legal & Compliance Monitoring**

The Legal module's AI assistant tracks contract expiry dates, compliance deadlines, and IP filing statuses — and proactively prepares the necessary documents and actions before issues become problems. It connects directly to the Work Board, Exchange, and Bank modules to flag legal dependencies on active workflows.

|  | EXAMPLE SCENARIO *The Dev Contract with DevDAO expires in 7 days. The AI pre-populates a renewal draft with the existing terms, flags two clauses that should be updated to reflect the project scope change made in month 2, cross-references the renewal cost against the current sprint budget, and schedules an automatic escalation if no action is taken within 48 hours.* |
| :---- | :---- |

### **Key AI Capabilities in Legal**

* **Contract expiry monitoring** — Tracks all active contracts and alerts users at 45-, 14-, and 7-day intervals before expiry — with renewal drafts ready at the 14-day mark

* **NDA generation** — Generates customised NDAs for new contractor or client relationships, calibrated to project type and applicable jurisdiction

* **IP filing assistance** — Tracks the status of patent and trademark applications, drafts cover letters, and prepares responses to office actions based on the filing history

* **Compliance calendar** — Maintains a rolling calendar of compliance deadlines — GDPR reviews, SOC 2 controls, AML/KYC renewals — with auto-drafted action items assigned to the relevant team member

* **Contract comparison** — When renewing, presents a tracked-changes view of the existing versus proposed contract, highlighting every changed clause for focused user review

* **Legal spend tracking** — Aggregates legal fees across all active engagements and flags when total legal spend exceeds the configured monthly or quarterly budget threshold

| WORKFLOW 9  ·  ASSET TRANSFER & ESCROW MANAGEMENT |
| :---- |

## **Asset Transfer & Escrow Management**

When users transfer digital assets — brand systems, code packages, design files, IP licences — the AI assistant manages the full transfer lifecycle: provenance verification, escrow setup, delivery monitoring, and fund release. The user's involvement is reduced to two approval checkpoints.

|  | EXAMPLE SCENARIO *Jordan is selling the Brand Identity System to Studio Co. for $3,200. The AI verifies the asset's blockchain provenance hash, confirms licence terms are attached, locks funds in escrow via Stripe, monitors the delivery confirmation, runs an IP check, and prepares the release trigger — Jordan approves at submission and again at release, touching nothing in between.* |
| :---- | :---- |

### **Key AI Capabilities in Asset Transfer**

* **Provenance verification** — Checks blockchain hash, full ownership chain, and attached licence terms before any transfer begins — flags discrepancies and blocks the transfer pending resolution

* **Escrow setup** — Calculates correct escrow amount including platform fees, prepares the escrow agreement, and coordinates fund lock with Stripe or Coinbase

* **Delivery monitoring** — Tracks each step of the asset delivery process (upload, access confirmation, IP check) and sends real-time status updates to both parties

* **Release readiness check** — Before recommending fund release, verifies all six verification checkpoints are complete: authenticity, ownership chain, licence, IP clearance, delivery proof, and payment confirmation

* **Multi-party batch transfers** — For complex transfers such as token distributions to 35 cooperative members, the agent manages the full batch process, tracks individual acknowledgements, and handles exceptions individually

| WORKFLOW 10  ·  OKR MONITORING & REVENUE INTELLIGENCE |
| :---- |

## **OKR Monitoring & Revenue Intelligence**

The AI assistant continuously monitors OKR progress, calculates delivery velocity, forecasts quarter-end outcomes, and generates specific intervention recommendations when any target is at risk — bridging the gap between strategic goals and day-to-day execution across every module.

|  | EXAMPLE SCENARIO *Revenue is at 62% of target with 28 days remaining. The current run rate projects a $2,600 shortfall. The agent identifies three levers: increase Marketplace listing visibility (estimated \+$1,800), follow up on two warm investor leads (potential \+$5,000 in next round), and accelerate one billable project milestone by 4 days (+$800). All three are presented as clickable actions in the morning briefing.* |
| :---- | :---- |

### **Key AI Capabilities in OKR & Analytics**

* **Velocity calculation** — Computes current monthly revenue run rate and forecasts the end-of-quarter outcome with a confidence range based on historical volatility

* **Shortfall intervention planning** — When a target is at risk, generates 3 prioritised intervention suggestions — each with a specific estimated impact and the module where the action should be taken

* **Cross-module metric linking** — Connects each OKR metric to specific projects, invoices, Marketplace listings, and Exchange deals — so every number has a direct action path, not just a trend line

* **Weekly digest** — Every Monday, delivers a week-in-review comparing actual vs. target across all OKRs, with the top 3 focus areas for the week ahead pre-populated with suggested actions

* **Milestone-to-payment tracking** — Monitors project milestones that trigger payment obligations and prompts the user to invoice within hours of a milestone being marked complete on the Work Board

* **Benchmark comparisons** — Optionally compares the user's KPIs against anonymised platform averages for similar project types and revenue bands, surfacing performance gaps

| WORKFLOW 11  ·  CAPITAL EXCHANGE & DEAL MANAGEMENT |
| :---- |

## **Capital Exchange & Deal Management**

The Capital Exchange module's AI agent manages the full lifecycle of a fundraising round — tracking raise progress, managing investor communications, monitoring cap table implications, and coordinating with the Bank module to execute distributions when a milestone is hit.

|  | EXAMPLE SCENARIO *The Seed Pre-A round is at 90% funded. The agent identifies that the final $14,000 gap could be filled by two Kogi Community Bond subscribers who have expressed interest. It drafts personalised outreach to both, prepares the updated cap table showing post-close equity distribution, and queues the automatic token allocation workflow for the moment the round closes.* |
| :---- | :---- |

### **Key AI Capabilities in Capital Exchange**

* **Round progress tracking** — Monitors percentage funded against target in real time and alerts when the round crosses 50%, 75%, 90%, and 100% thresholds

* **Cap table impact modelling** — Before each new investment is confirmed, models the updated cap table and flags dilution impact to existing members for governance review

* **Investor communication drafts** — Generates investor updates at configured intervals — monthly or on milestone events — with the latest financial and product metrics pre-populated

* **Distribution scheduling** — When a distribution event is triggered (revenue share, dividend, token allocation), prepares the full payment batch and executes it on the Bank module upon approval

* **Closing coordination** — When a raise closes, the agent coordinates across Capital Exchange, Bank, Legal, and Governance to execute the full close checklist: cap table update, fund receipt, legal filing, member notification, and token allocation

| WORKFLOW 12  ·  BANK & TREASURY INTELLIGENCE |
| :---- |

## **Bank & Treasury Intelligence**

The Bank module's AI assistant monitors wallet balances, tracks fundraising progress across multiple instruments simultaneously, maintains tax reserve accuracy, and proactively flags treasury risks — acting as an intelligent CFO function for individuals and small cooperatives that can't afford dedicated finance staff.

|  | EXAMPLE SCENARIO *The Equipment Loan fundraising goal is 5 days from its deadline and 73% funded. The agent calculates that the gap can be closed if 3 existing Community Bond holders increase their contribution, drafts personalised asks for each, and simultaneously flags that the current Operating wallet balance will fall below the configured minimum reserve if the equipment purchase proceeds — recommending a timing adjustment.* |
| :---- | :---- |

### **Key AI Capabilities in Bank & Treasury**

* **Reserve monitoring** — Continuously tracks wallet balances against configured minimum reserves and alerts when any wallet approaches its floor, with specific recommended transfers to restore balance

* **Tax reserve automation** — Calculates real-time tax liability estimates based on revenue events and automatically transfers the configured percentage to the Tax Reserve wallet after each invoice payment

* **Fundraising deadline management** — For each active fundraising instrument, tracks days remaining, gap to target, and velocity — and drafts outreach to likely contributors when deadline pressure is high

* **Cross-wallet reconciliation** — Detects mismatches between expected and actual balances across all wallet types and generates a reconciliation report flagging any unexplained variances

* **Expenditure forecasting** — Based on active contracts, scheduled distributions, and upcoming milestones, forecasts 30- and 60-day cash outflows and flags months where outflows exceed projected inflows

# **Summary: AI Value Across All Kogi Modules**

The table below consolidates all twelve AI workflows, mapping each to its module, primary tools, and the concrete value delivered to users.

| MODULE / SCREEN | PRIMARY AI WORKFLOW | TOOLS INVOKED | USER VALUE |
| :---- | :---- | :---- | :---- |
| **AI Agent** | Morning briefing & cross-module insight | *read\_portfolio, scan\_invoices, search\_marketplace* | One dashboard replaces 10 manual module checks every morning |
| **Work Board** | Sprint unblocking via Marketplace | *monitor\_board, search\_marketplace, draft\_inquiry* | Blocked stories resolved in hours rather than days |
| **Exchange** | Invoice recovery & overdue management | *scan\_invoices, draft\_email, calculate\_cashflow* | Overdue payment recovery is proactive and documented |
| **Governance** | Proposal drafting, vote monitoring & execution | *draft\_proposal, notify\_members, execute\_proposal* | Full governance cycle with audit trail and zero missed votes |
| **Investor Outreach** | Pipeline management & personalised outreach | *draft\_email, update\_pipeline, prepare\_materials* | Fundraising velocity increases; no lead goes cold |
| **Community** | Content amplification & collaboration matching | *draft\_post, analyse\_engagement, match\_collaborators* | Showcase visibility and collaboration opportunities grow continuously |
| **Studio (Idea→Asset)** | Stage-gate AI at every transition | *draft\_spec, write\_tests, publish\_asset* | Ideas reach market faster with systematically higher quality |
| **Legal** | Contract & compliance monitoring | *monitor\_contracts, draft\_renewal, track\_compliance* | Zero missed deadlines; renewals prepared 14 days early |
| **Asset Transfer** | Escrow & provenance management | *verify\_provenance, manage\_escrow, confirm\_delivery* | Secure, verified asset transactions with minimal user steps |
| **OKR & Analytics** | Velocity tracking & intervention planning | *calculate\_velocity, suggest\_interventions, digest* | Strategy remains connected to daily execution at all times |
| **Capital Exchange** | Round tracking, cap table & distributions | *track\_round, model\_captable, execute\_distribution* | Raise coordination and investor communication automated |
| **Bank & Treasury** | Reserve monitoring & tax automation | *monitor\_wallets, calculate\_tax, forecast\_cashflow* | CFO-level treasury intelligence without dedicated finance staff |

## **The Agent as a Unified Layer**

While each workflow above is described per module, the most powerful aspect of Kogi's AI is the way agents operate across module boundaries simultaneously. A single morning briefing can surface a blocked sprint story, an overdue invoice from the same client, a governance vote that would release budget to fix the blocker, and a Marketplace contractor who is available today — connecting four separate modules into one coherent action cluster.

This cross-module coherence is what separates Kogi's AI layer from a collection of disconnected chatbots. The agent holds a unified picture of the user's professional life and acts as the connective tissue between strategy (OKRs), execution (Work Board), finances (Exchange & Bank), people (Marketplace), and governance (Coop Governance) — continuously optimising the path between where the user is and where they want to be.