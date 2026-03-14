  
**KOGI**

Independent Worker Operating System

**Platform Tools**

System Design Document

v1.0  ·  Kogi Platform  ·  March 2026

# **1\. Overview**

Kogi Platform Tools are purpose-built, AI-assisted utilities that help independent workers, creatives, professionals, and organizations generate, design, manage, and publish the documents, content, and assets they need to operate and grow. Unlike generic third-party tools, Kogi Tools are deeply integrated with the platform's Portfolio System, kogi-bank, Marketplace, Community, and Oba AI layer — meaning outputs are automatically saved as versioned Portfolio Artifacts, linked to relevant work items, and reusable across every module.

This document defines the full tools catalog: starting with the flagship Resume & Work-Portfolio Highlights Builder, followed by essential tools across identity, business development, financial, work planning, community, and AI assistance categories.

| Tool ID | Tool Name | Category |
| :---- | :---- | :---- |
| **T-01** | Resume & Work-Portfolio Highlights Builder | Identity & Presentation |
| **T-02** | Bio & Elevator Pitch Generator | Identity & Presentation |
| **T-03** | Press Kit Builder | Identity & Presentation |
| **T-04** | Brand Kit & Social Media Kit Builder | Identity & Presentation |
| **T-05** | Proposal & Pitch Deck Generator | Business Development |
| **T-06** | Rate & Pricing Calculator | Business Development |
| **T-07** | Scope of Work (SOW) Builder | Business Development |
| **T-08** | Client Onboarding Toolkit | Business Development |
| **T-09** | Contract Template Generator | Financial & Legal |
| **T-10** | Invoice & Payment Builder | Financial & Legal |
| **T-11** | Tax & Income Estimator | Financial & Legal |
| **T-12** | Grant Application Builder | Financial & Legal |
| **T-13** | Project Brief & Charter Generator | Work Planning |
| **T-14** | OKR & Goal-Setting Tool | Work Planning |
| **T-15** | Retrospective & Review Facilitator | Work Planning |
| **T-16** | Cooperative Charter Builder | Community & Orgs |
| **T-17** | Initiative & Campaign Planner | Community & Orgs |
| **T-18** | Equity Crowdfunding Campaign Builder | Community & Orgs |
| **T-19** | Skills Mapper & Gap Analyzer | AI Tools (Oba) |
| **T-20** | Career Path & Rate Benchmarking Advisor | AI Tools (Oba) |
| **T-21** | Content Repurposer & Publishing Kit | AI Tools (Oba) |
| **T-22** | Opportunity Matcher & Pitch Personalizer | AI Tools (Oba) |

## **1.1 Shared Tool Architecture**

All Kogi Tools share a common architecture and output pipeline ensuring that every tool output is a first-class platform object.

| Layer | Description |
| :---- | :---- |
| **Input Layer** | Form-driven or conversational (Oba-mediated) data collection; pre-populated from Portfolio, Profile, and kogi-bank data where available |
| **Generation Engine** | AI generation via Oba (Claude-powered); template-based composition; rule-based data transformation |
| **Output Layer** | Produces: PDF export · DOCX export · Portfolio Artifact · Shareable link · Direct publish to Profile / Marketplace / Community |
| **Storage Layer** | All outputs saved as versioned Portfolio Artifacts; linked to relevant Portfolio Components; full history retained |
| **Analytics** | View counts, share rates, conversion tracking (e.g., proposal-to-close), download events tracked per artifact |
| **Personalization** | PersonalizationEngine tunes tone, format, and content emphasis based on user Persona, work history, and target audience context |

# **2\. Identity & Presentation Tools**

| T-01 | Resume & Work-Portfolio Highlights Builder *Generate tailored, AI-enhanced resumes and portfolio highlight decks from your Kogi work history* |
| :---: | :---- |

### **Purpose**

The Resume & Work-Portfolio Highlights Builder is Kogi's flagship self-presentation tool. It transforms a user's live platform data — their portfolio components, completed projects, gigs, skills, contributions, endorsements, and analytics — into professionally formatted, targeted, and AI-refined resume documents and portfolio highlight packages. Every output is context-aware: the tool generates different versions for different audiences (employers, clients, investors, grant committees, collaborators) from the same underlying work record.

### **Core Capabilities**

| Capability | Description |
| :---- | :---- |
| **Auto-population from Portfolio** | Imports projects, programs, roles, contributions, skills, endorsements, certifications, and timeline data directly from the user's Kogi portfolio — no manual re-entry |
| **Multi-format resume generation** | Produces: Chronological resume · Functional resume · Hybrid resume · Portfolio-first resume · Executive summary · One-pager bio · CV (academic/extended) |
| **Portfolio highlights deck** | Generates a visual project highlights deck: 1–3 pages per featured project with outcome metrics, role summary, tools used, and impact statement — exportable as PDF or slide deck |
| **Audience targeting** | User specifies target (employer, client, investor, grant committee, collaborator); Oba adjusts tone, vocabulary, emphasis, and which projects to feature |
| **AI narrative generation** | Oba drafts achievement statements using the STAR format (Situation, Task, Action, Result); quantifies impact where analytics data is available |
| **Skills & endorsements import** | Pulls skills from profile, endorsements from community, certifications from professional development account, and platform-verified activity badges |
| **Version management** | Unlimited named versions (e.g., 'Tech Lead Resume', 'Creative Freelancer', 'Grant CV'); each version is a separate Portfolio Artifact |
| **ATS optimization** | Analyzes target job description (paste or URL); optimizes keyword density and formatting for Applicant Tracking System compatibility |
| **Real-time analytics integration** | Where portfolio analytics are available, auto-inserts quantified results: 'Led 3 campaigns generating 42% audience growth'; sourced from AnalyticsEngine |
| **Cover letter companion** | Generates a contextually matched cover letter or client outreach message synchronized with the resume version |
| **Export formats** | PDF (print-ready) · DOCX (editable) · Plain text (ATS-safe) · JSON (machine-readable) · Shareable link (live portfolio page) |
| **Template library** | 30+ professionally designed resume and highlights deck templates organized by industry, persona, and purpose; community-contributed templates available |

### **Workflow**

| Step | Action |
| :---- | :---- |
| **1\. Source Selection** | User selects which portfolio items, projects, roles, and time ranges to include; smart suggestions from Oba based on most recent and highest-engagement items |
| **2\. Audience Config** | Set target audience type, role or opportunity title, desired tone (professional, creative, executive, technical), and output format |
| **3\. Content Review** | Oba presents a structured draft with editable sections; user approves, edits, or prompts Oba to revise specific achievement statements or descriptions |
| **4\. ATS / Keyword Pass** | Optional: paste target job description or opportunity brief; Oba runs keyword gap analysis and suggests additions without compromising authenticity |
| **5\. Design Selection** | Choose template; customize accent color, font weight, section order; preview in real time |
| **6\. Generate Outputs** | One-click production of all selected output formats; cover letter generated in parallel |
| **7\. Save & Publish** | Saved as Portfolio Artifact; optionally published to public profile, shared via link, or attached to a Marketplace application or grant proposal |
| **8\. Version Update** | When new portfolio items are added or analytics refresh, Oba notifies: 'Your Tech Lead resume can be updated with 2 new projects' |

### **Data Sources (Auto-Populated Fields)**

| Resume Section | Kogi Data Source | Auto-populated Content |
| :---- | :---- | :---- |
| **Work Experience** | Portfolio programs \+ projects | Role title, org name, dates, scope description, team size, outcomes, links |
| **Skills** | Profile skills \+ contribution types | Skill names, proficiency levels, platform usage frequency, peer endorsements |
| **Achievements & Impact** | Portfolio analytics \+ project outcomes | Quantified metrics: growth %, revenue generated, audience size, completion rates |
| **Certifications** | Professional development account | Certification name, issuer, date, credential ID, expiry |
| **Education** | Profile education fields | Institution, degree/program, dates, relevant coursework, honors |
| **Volunteering / Contrib** | Community contributions \+ initiatives | Initiative name, role, contribution type, org, dates, impact summary |
| **Portfolio Highlights** | Portfolio artifacts \+ project pages | Project name, role, thumbnail, outcome, tools, external link |
| **Endorsements** | Community endorsements \+ reviews | Endorser name, relationship, skill endorsed, quote (with permission) |
| **Publications** | Content artifacts \+ community posts | Title, platform, date, engagement metrics, external link |
| **Awards & Recognition** | Portfolio badges \+ community recognition | Award name, issuer, date, context |

### **AI Generation — Oba Integration**

* Achievement rewriting: converts raw task descriptions into STAR-format impact statements

* Tone calibration: adjusts formality, confidence level, and vocabulary register per audience

* Gap detection: identifies missing resume sections and prompts the user to fill them from portfolio or manually

* Tailoring suggestions: for each target opportunity, Oba recommends which projects to lead with and which to omit

* Freshness alerts: notifies when a saved resume version is stale relative to new portfolio activity

* Duplicate detection: flags repeated phrases, overused words, and inconsistent date formats

* Comparison mode: side-by-side two resume versions with diff highlighting

### **Output Artifacts**

| Output | Format & Destination |
| :---- | :---- |
| **Tailored Resume** | PDF · DOCX · Plain TXT → saved as Portfolio Artifact; shareable link; attaches to Marketplace applications |
| **Portfolio Highlights Deck** | PDF · PPTX (slide deck) → saved as Portfolio Artifact; embeds in public profile page |
| **Cover Letter** | PDF · DOCX · plain email text → saved as Portfolio Artifact; synced to CRM lead record |
| **Skills Summary** | JSON · plain text → API-accessible; used by MatchEngine for gig and job matching |
| **Executive Bio** | Plain text · PDF → reusable across press kit, conference speaker profiles, social bios |
| **ATS-Optimized Plain Text** | TXT → specifically formatted for copy-paste into online application systems |

| T-02 | Bio & Elevator Pitch Generator *Craft compelling professional bios and spoken / written pitch scripts in seconds* |
| :---: | :---- |

### **Purpose**

Generates multiple-length professional bios (one-liner, 50-word, 150-word, full-page) and structured elevator pitch scripts from profile, portfolio, and persona data. Each bio variant is optimized for a specific context: speaking engagements, social media profiles, press releases, platform profiles, grant applications, and podcast introductions.

| Feature | Description |
| :---- | :---- |
| **Multi-length bio suite** | Generates: one-liner · tweet bio (160 chars) · short bio (50w) · medium bio (150w) · long bio (500w) · full speaker bio (1 page) |
| **Context variants** | Produces different versions for: LinkedIn · Kogi profile · speaking bio · grant committee · press release · conference website · email signature |
| **Third-person / first-person** | Toggle between narrative modes; Oba adapts naturally |
| **Elevator pitch script** | Structured 30-second, 60-second, and 2-minute pitch scripts with opening hook, value proposition, proof point, and call-to-action |
| **Persona-aware tone** | Detects active persona (Creative, Professional, Activist, etc.) and adjusts emphasis — e.g., leading with impact for Activists, with craft for Creatives |
| **Audience targeting** | Specify audience: clients · investors · collaborators · grant committees · general public — Oba adjusts vocabulary and emphasis |
| **Update triggers** | Oba alerts when key portfolio achievements should refresh the bio |

| T-03 | Press Kit Builder *Assemble a complete, publication-ready press kit in one workflow* |
| :---: | :---- |

### **Purpose**

Produces a structured, shareable press kit package for individuals, projects, or organizations launching a release, event, campaign, or funding round. Outputs are hosted as a live link and exported as a downloadable ZIP.

| Component | Description |
| :---- | :---- |
| **Press Release** | AI-drafted announcement in AP Style; customizable headline, quote blocks, and boilerplate; PDF \+ plain text outputs |
| **Fact Sheet** | Bullet-format one-pager: key stats, milestones, team, and contact info; auto-populated from portfolio analytics |
| **Media Bio** | Full 1-page bio formatted for press use; third-person; includes headshot placement |
| **Project / Product Overview** | 2–3 page narrative overview of the work being announced; links to portfolio items |
| **Media Assets Package** | Approved hi-res images, logos, brand marks — zipped; hosted media asset gallery with download links |
| **Notable Quotes** | Curated quote bank: from the creator/founder, partners, clients, and community endorsers |
| **Coverage Links** | Running list of press coverage; auto-updated when new links are added |
| **Contact Block** | Booking, press, and partnership contacts; links to Kogi CRM intake form |
| **Live Hosted Kit** | Shareable URL with password-optional access; analytics on who viewed and which assets downloaded |

| T-04 | Brand Kit & Social Media Kit Builder *Define, generate, and distribute a consistent visual and verbal brand identity* |
| :---: | :---- |

### **Purpose**

Helps independent workers and organizations establish and document their brand identity — covering visual elements, voice guidelines, and ready-to-use social media assets. Outputs are stored as a Brand Kit Portfolio Artifact reused across all platform publishing surfaces.

| Component | Description |
| :---- | :---- |
| **Brand Voice Guide** | Tone, vocabulary, and style guidelines; do/don't examples; Oba-drafted from persona and work history signals |
| **Visual Identity Spec** | Color palette (hex/RGB/CMYK), typography pairings, logo usage rules, iconography notes |
| **Profile Asset Set** | Platform-formatted profile images and banners for Kogi, LinkedIn, Instagram, X, YouTube — correct dimensions, auto-exported |
| **Social Media Templates** | Customizable post templates for announcements, project highlights, quotes, milestones — 9:16, 1:1, 16:9 formats |
| **Content Calendar Starter** | AI-generated 30-day content plan aligned to brand voice, portfolio launches, and initiative timelines |
| **Bio Consistency Audit** | Checks all profile bios across connected platforms for tone and information consistency |
| **Hashtag Strategy** | Recommended primary, secondary, and community hashtags based on persona, industry, and platform analytics |
| **Brand Kit Export** | ZIP package: logo files, color swatches, font files, template files, and brand guide PDF |

# **3\. Business Development Tools**

| T-05 | Proposal & Pitch Deck Generator *Create winning client proposals and investor pitch decks from your portfolio* |
| :---: | :---- |

### **Purpose**

Generates structured, professional proposals and pitch decks tailored to a specific opportunity — client project, partnership, investor round, or grant. Pre-populates from the user's portfolio data, past project outcomes, team profiles, and kogi-bank financial data. Outputs integrate with the CRM pipeline and contract workflow.

| Feature | Description |
| :---- | :---- |
| **Proposal types** | Client project proposal · Partnership proposal · Grant proposal · Event booking proposal · Retainer proposal · Joint venture proposal |
| **Pitch deck types** | Investor pitch deck (12-slide) · Community pitch · Cooperative pitch · Product launch deck · Event pitch · Program pitch |
| **Smart section generation** | Oba drafts: executive summary, problem statement, proposed solution, scope, deliverables, timeline, team bios, past work proof, pricing, and next steps |
| **Portfolio proof insertion** | Auto-selects most relevant past projects and outcome metrics as credibility evidence; links live to portfolio artifacts |
| **Pricing integration** | Pulls from Rate Calculator (T-06) and past invoice data (kogi-bank) to propose competitive, well-anchored pricing |
| **Client-specific tailoring** | Input client/investor name and context; Oba personalizes the narrative, adjusts vocabulary, and mirrors the opportunity language |
| **Deck design** | Professionally designed slide templates; brand kit integration for colors and fonts; drag-reorder sections |
| **Send and track** | Share via Kogi link; tracks opens, time-spent-per-slide, and downloads; Oba alerts on engagement signals |
| **CRM sync** | Proposal auto-linked to CRM deal record; status updates on open, viewed, and accepted events |
| **Accept flow** | Proposal acceptance triggers contract generation (T-09) and deposit invoice (T-10) automatically |

| T-06 | Rate & Pricing Calculator *Set, justify, and communicate your rates with confidence and market data* |
| :---: | :---- |

### **Purpose**

A data-driven pricing tool that helps independent workers determine market-appropriate rates, build pricing packages, and communicate pricing to clients. Combines platform income data, market benchmarks, expense analysis, and goal-based income planning.

| Feature | Description |
| :---- | :---- |
| **Hourly / day rate calculator** | Income goal input → expense baseline → target margin → recommended hourly and day rate with utilization assumptions |
| **Project pricing builder** | Time estimate \+ rate \+ expenses \+ risk buffer \+ profit margin → flat project fee with itemized breakdown |
| **Package builder** | Design tiered service packages (Starter / Standard / Premium) with feature lists and price points; embed on profile |
| **Market benchmarking** | MatchEngine pulls market rate ranges from anonymized platform data by skill category, persona, experience level, and geography |
| **Retainer calculator** | Monthly retained hours \+ scope → retainer price with discount modeling; payment schedule options |
| **Rate increase modeler** | Simulates impact of rate increases on bookings, revenue, and client mix; Oba recommends timing and communication approach |
| **Competitive positioning** | Positions user's rates relative to platform benchmarks: budget / mid-market / premium / luxury tiers |
| **Rate card generator** | Produces a formatted, shareable Rate Card PDF for client distribution; versioned |

| T-07 | Scope of Work (SOW) Builder *Define, document, and lock project scope to prevent creep and protect your time* |
| :---: | :---- |

### **Purpose**

Guides the user through a structured scope definition workflow producing a signed SOW document that integrates with contract generation and project backlog creation.

| Feature | Description |
| :---- | :---- |
| **Guided scope interview** | Oba asks structured questions: deliverables, exclusions, assumptions, acceptance criteria, revision rounds, communication expectations |
| **SOW document generation** | Produces: project overview, objectives, deliverables list, out-of-scope items, assumptions, dependencies, timeline, revision policy |
| **Change order workflow** | Built-in change order template; client signs digitally; kogi-bank generates supplemental invoice automatically |
| **Backlog seeding** | On approval, SOW deliverables optionally seed the project backlog in Work Management as epics and stories |
| **Milestone schedule** | Auto-generates milestone payment schedule from SOW timeline; feeds kogi-bank invoice queue |
| **Conflict detection** | Oba flags vague deliverables, missing acceptance criteria, and undefined revision limits before the SOW is sent |

| T-08 | Client Onboarding Toolkit *Deliver a polished, automated onboarding experience for every new client* |
| :---: | :---- |

### **Purpose**

Packages all the materials, communications, and workflow triggers a worker needs to onboard a new client professionally. Connects to CRM, contracts, invoicing, and project setup in a single guided flow.

| Component | Description |
| :---- | :---- |
| **Welcome package** | Customizable welcome email/document: intro, working style, communication norms, tools, and what to expect |
| **Onboarding questionnaire** | Client intake form: goals, constraints, preferences, assets needed, key contacts — responses saved to CRM record |
| **Contract \+ SOW bundle** | Pre-configured contract (T-09) \+ SOW (T-07) sent as a single e-signature package |
| **Deposit invoice** | Automatically triggered deposit invoice (T-10) sent on contract signature |
| **Shared workspace setup** | Creates a client-specific portfolio space with shared documents, project board, and communication room |
| **Kickoff meeting template** | Structured kickoff agenda, pre-read, and meeting notes template; outcome actions auto-create backlog items |
| **Timeline share** | Client-facing project timeline with milestone visibility; updates automatically as work progresses |
| **Automation triggers** | Sequence: contract signed → deposit invoice → welcome email → shared workspace → kickoff booking link |

# **4\. Financial & Legal Tools**

| T-09 | Contract Template Generator *Generate, customize, and send legally structured contracts from your work history context* |
| :---: | :---- |

### **Purpose**

Produces customizable contract documents for every common independent worker engagement type. Contracts integrate with the e-signature workflow, kogi-bank invoice system, and Portfolio EventLog for full audit history.

| Contract Type | Key Clauses Auto-populated |
| :---- | :---- |
| **Freelance Services Agreement** | Scope, deliverables, payment terms, revision rounds, IP ownership, kill fee, confidentiality, governing law |
| **Consulting Agreement** | Retainer terms, statement of work attachment, expense reimbursement, non-compete (optional), IP ownership |
| **Independent Contractor Agreement** | Contractor status, project scope, payment schedule, equipment/tools, termination, indemnification |
| **Creative Services Agreement** | Licensing terms, usage rights, credit requirements, exclusivity window, deliverable specs |
| **Partnership Agreement** | Roles and responsibilities, revenue share formula, decision-making process, IP ownership, exit terms |
| **Revenue Share Agreement** | Revenue definition, calculation method, payment cadence, audit rights, term and termination |
| **NDA / Confidentiality** | Scope of confidential info, obligations, exclusions, term, remedies |
| **Service Level Agreement (SLA)** | Response times, uptime commitments, escalation path, remedies for breach |
| **Booking / Event Contract** | Event details, deliverables, deposit schedule, cancellation policy, force majeure, overtime terms |
| **License Agreement** | Scope of license, exclusivity, territory, royalty terms, sublicensing, IP warranties |

| Feature | Description |
| :---- | :---- |
| **AI clause drafting** | Oba suggests appropriate clauses based on engagement type, platform usage history, and past contract patterns |
| **Custom clauses** | Free-text custom clause editor with Oba review for legal clarity and consistency |
| **Variable fields** | Auto-populated from booking details, client CRM record, SOW, and kogi-bank account data |
| **E-signature workflow** | Send via integrated e-signature (DocuSign / HelloSign); signed status tracked in Portfolio System |
| **Version control** | All contract revisions versioned; full audit trail; counterparty change requests tracked |
| **Template library** | Personal library of saved and modified templates; community-contributed clause library |
| **Jurisdiction guidance** | Oba notes when clauses may vary significantly by jurisdiction; user consults qualified legal counsel for final review |

| T-10 | Invoice & Payment Builder *Create, send, and track professional invoices with automated reminders and payment routing* |
| :---: | :---- |

### **Purpose**

A full-cycle invoice management tool integrated with kogi-bank ledgers, booking and contract data, and client CRM records. Automates the full billing lifecycle from creation to payment confirmation.

| Feature | Description |
| :---- | :---- |
| **Invoice types** | Deposit · Progress · Milestone · Final · Recurring · Retainer · Credit Note · Expense Reimbursement |
| **Auto-population** | Pulls client info from CRM, line items from SOW/contract, payment terms from agreement, tax rate from location data |
| **Line item templates** | Saved line item libraries: service rates, package prices, expense categories — reusable across invoices |
| **Payment schedules** | Multi-payment schedule builder; automatic staged invoice generation on milestone completion |
| **Recurring billing** | Subscription/retainer billing: auto-generates and sends invoices on configured cadence |
| **Payment tracking** | Real-time payment status: sent → viewed → partial → paid → overdue; kogi-bank reconciliation |
| **Automated reminders** | Configurable sequences: 3-day pre-due reminder · due-date reminder · 3-day late · 7-day late · final notice |
| **Multiple payment methods** | kogi-bank transfer · ACH · card · international wire · platform credits; QR code payment option |
| **Tax handling** | GST/VAT · sales tax · self-employment tax estimates; itemized on invoice; links to Tax Estimator (T-11) |
| **Late fees** | Configurable late fee rules auto-applied and noted on overdue invoices per contract terms |
| **Analytics** | Average days-to-pay by client; revenue by month; outstanding AR aging; invoice conversion rate |

| T-11 | Tax & Income Estimator *Stay ahead of tax obligations with real-time income tracking and self-employment tax modeling* |
| :---: | :---- |

### **Purpose**

A tax planning and income modeling tool designed specifically for independent workers, gig workers, and self-employed individuals. Connects to kogi-bank income and expense data to provide running tax liability estimates, quarterly payment reminders, and deductible expense tracking.

| Feature | Description |
| :---- | :---- |
| **Self-employment tax estimate** | Running estimated SE tax liability based on YTD net income from kogi-bank; updated on each payment received |
| **Quarterly payment reminder** | Alerts for IRS/HMRC estimated tax payment due dates with suggested payment amounts based on YTD actuals |
| **Deduction tracker** | Categorizes and tracks deductible business expenses: home office, equipment, software, travel, professional development |
| **Income scenario modeling** | Models annual tax impact of different income levels; useful for setting rates and evaluating gig acceptance |
| **Retirement contribution impact** | Shows how SEP-IRA or Solo 401k contributions reduce taxable income; links to kogi-bank retirement accounts |
| **Health insurance deduction** | Tracks self-employed health insurance premiums as above-the-line deduction; linked to portable benefits accounts |
| **Multi-state / multi-platform** | Aggregates income from multiple gig platforms; estimates state tax obligations for workers in multiple states |
| **Year-end summary** | Generates a summary of income, deductions, and estimated tax for accountant or tax software handoff; exportable |
| **W-9 / 1099 tracker** | Tracks clients and platforms required to issue 1099s; alerts when thresholds are crossed ($600+ US) |

| T-12 | Grant Application Builder *Discover matching grants and build compelling applications backed by portfolio evidence* |
| :---: | :---- |

### **Purpose**

Combines the kogi-bank GrantEngine with an AI-assisted application writing workflow, portfolio evidence assembly, and impact reporting. Designed for individuals, collectives, cooperatives, and community organizations seeking government, foundation, platform, and community grants.

| Feature | Description |
| :---- | :---- |
| **Grant discovery** | GrantEngine surfaces matching grants from registry based on eligibility profile, portfolio content, entity type, and past award history |
| **Eligibility check** | Structured pre-screen: work type, entity, location, revenue, sector — Oba confirms or disqualifies before the user invests time |
| **Application narrative writer** | Oba drafts responses to common grant questions: project description, community impact, organizational capacity, budget justification |
| **Portfolio evidence assembly** | Auto-attaches relevant portfolio artifacts as supporting documentation: project reports, financial statements, community engagement metrics |
| **Budget builder** | Structured grant budget template with expense categories, justifications, and matching funds tracking |
| **Impact statement generator** | Translates portfolio analytics into a compelling impact narrative: reach, outcomes, community served, measurable change |
| **Reviewer persona mode** | Oba role-plays grant reviewer, scores the application draft, and flags weak sections before submission |
| **Deadline management** | Grant deadlines added to Timeline as milestones; task checklist auto-generated for each application |
| **Application tracker** | Lifecycle tracking: Drafting → Submitted → Under Review → Awarded | Rejected → Reporting |
| **Impact report generator** | Post-award: auto-generates progress and final impact reports from portfolio and kogi-bank data on schedule |

# **5\. Work Planning Tools**

| T-13 | Project Brief & Charter Generator *Launch every project with clarity — scope, goals, team, and success criteria defined from day one* |
| :---: | :---- |

### **Purpose**

A guided tool for generating a complete project brief or formal project charter from a set of structured inputs. Output documents are saved as Portfolio Artifacts linked to the project and seed the Work Management System with initial epics, milestones, and a backlog.

| Section Generated | Description |
| :---- | :---- |
| **Executive Summary** | 1-paragraph project overview for stakeholder communication |
| **Problem / Opportunity** | Root cause or opportunity statement; Oba frames using standard problem framing templates |
| **Goals & Success Criteria** | SMART goals with measurable success criteria; linked to OKR tool (T-14) |
| **Scope & Out of Scope** | Defined inclusions and explicit exclusions; prevents scope creep from project start |
| **Stakeholders & Roles** | RACI matrix: Responsible, Accountable, Consulted, Informed — populated from team and CRM records |
| **Timeline & Milestones** | High-level phased timeline; milestones added to Work Management and Gantt automatically |
| **Budget Envelope** | Initial budget allocation; linked to kogi-bank project account |
| **Risks & Mitigation** | Structured risk register: risk, likelihood, impact, owner, mitigation plan |
| **Dependencies & Assumptions** | External dependencies and working assumptions that could invalidate the plan |
| **Communication Plan** | Stakeholder update cadence, channels, and escalation path |
| **Approval & Sign-off** | Digital sign-off from project sponsor or key stakeholders; audit trail |

| T-14 | OKR & Goal-Setting Tool *Set meaningful objectives, define measurable key results, and track progress with Oba coaching* |
| :---: | :---- |

### **Purpose**

A structured OKR (Objectives and Key Results) authoring, alignment, and tracking tool. Works at individual, team, organization, and federation levels. Integrates with Work Management analytics and Portfolio System for auto-computed key result progress.

| Feature | Description |
| :---- | :---- |
| **OKR authoring wizard** | Guided Oba interview: define aspiration (Objective), identify 3–5 measurable outcomes (Key Results), set confidence levels and scoring scales |
| **Quality check** | Oba reviews drafts: flags unmeasurable KRs, too-broad objectives, and low-ambition targets before finalizing |
| **Hierarchy alignment** | Individual OKRs align to team → org → platform initiative OKRs; parent-child visualization |
| **Auto-progress updates** | Key Results linked to Portfolio analytics data auto-update progress: completed stories, revenue generated, community growth |
| **Weekly check-ins** | Structured 5-minute weekly check-in: confidence update, blocker flag, key result progress; generates momentum score |
| **Scoring & grading** | 0.0–1.0 OKR scoring at period end; auto-computed from KR actuals; Oba drafts retrospective commentary |
| **Cascade view** | Full hierarchy visualization: platform initiatives → org OKRs → team OKRs → individual OKRs |
| **OKR history** | All past OKR periods archived; trend analysis; improvement areas surfaced by Oba |

| T-15 | Retrospective & Review Facilitator *Run structured, productive retrospectives and reviews with any team — sync or async* |
| :---: | :---- |

### **Purpose**

A facilitation tool for running structured retrospectives, sprint reviews, project post-mortems, and quarterly reviews. Supports both real-time facilitated sessions and asynchronous collection workflows. Outputs are Portfolio Artifacts with action items auto-seeded into the backlog.

| Feature | Description |
| :---- | :---- |
| **Format library** | Pre-built formats: 4Ls (Liked, Learned, Lacked, Longed For) · Start/Stop/Continue · Mad/Sad/Glad · Starfish · Timeline Retrospective · Post-Mortem |
| **Async collection** | Team members submit responses before the session; Oba clusters and themes responses; facilitator reviews before the meeting |
| **Live facilitation mode** | Real-time shared board; dot-voting; timer-controlled rounds; facilitator controls progression |
| **Oba synthesis** | After collection, Oba produces a draft narrative summary: top themes, key wins, top concerns, and suggested action items |
| **Action items** | Agreed actions assigned, due-dated, and directly seeded into project backlog or personal TODO list |
| **Trend analysis** | Retro history compared across sprints or quarters; recurring themes flagged by Oba |
| **Sprint / PI review** | Structured sprint review agenda: demo items, velocity review, goal attainment, stakeholder feedback, next sprint preview |
| **Report generation** | Formatted retrospective report: Portfolio Artifact saved; shareable with stakeholders |

# **6\. Community & Organization Tools**

| T-16 | Cooperative Charter Builder *Establish your cooperative, collective, or autonomous org with a governing charter and operating agreement* |
| :---: | :---- |

### **Purpose**

A structured document generation tool for founding organizations on the Kogi platform. Produces a governance charter, operating agreement, membership policy, and initial treasury configuration. Designed for cooperatives, collectives, federations, councils, and autonomous organizations.

| Component Generated | Description |
| :---- | :---- |
| **Founding Charter** | Mission, vision, values, organizational type, founding members, purpose statement, and scope of activity |
| **Membership Policy** | Membership criteria, admission process, dues/contributions, rights, responsibilities, and exit procedures |
| **Governance Rules** | Decision-making model: vote thresholds, quorum, proposal lifecycle, tie-breaking, emergency procedures |
| **Revenue Distribution Policy** | Formula: hours contributed, equity share, governance participation, seniority weighting — configurable per org type |
| **Financial Controls** | Multi-sig treasury configuration: number of signers, threshold amounts, spending categories, audit schedule |
| **Operating Agreement** | Formal operating document for entity registration (LLC/Corp/Trust); state-specific guidance notes |
| **Bylaws** | Structured bylaws document for formal organization types; amendment procedures included |
| **Dissolution Protocol** | Asset distribution plan, wind-down procedure, member notification requirements on org dissolution |

| T-17 | Initiative & Campaign Planner *Design and launch a coordinated community initiative with a clear plan, team, and timeline* |
| :---: | :---- |

### **Purpose**

A structured planning tool for launching platform initiatives — portable benefits programs, community lending pools, shared portfolio campaigns, grant initiatives, advocacy campaigns, and crowdresourcing efforts. Outputs a full initiative plan with team assignments, timeline, budget, and governance setup.

| Feature | Description |
| :---- | :---- |
| **Initiative types** | Portable Benefits · Grants · Microfinancing · Group Economics · Equity Crowdfunding · Crowdresourcing · Advocacy · Shared Portfolio |
| **Guided planning interview** | Oba interviews: initiative goal, target community, resources needed, governance model, success criteria, and risk factors |
| **Plan document generation** | Produces: initiative overview, problem statement, proposed impact, team roles, timeline, budget, communication plan, and KPIs |
| **Team assembly** | Invite co-organizers, stewards, and contributors; roles assigned with governance permissions configured |
| **Governance setup** | Configures approval thresholds, steward committee, and voting rules for the initiative space |
| **Milestone timeline** | Auto-generates milestone schedule from planning inputs; syncs to Work Management timelines |
| **Budget template** | Initiative budget: funding sources (grants, member contributions, donations), planned expenses, and contingency |
| **Launch checklist** | Oba generates a tailored pre-launch checklist based on initiative type; items seed project backlog |

| T-18 | Equity Crowdfunding Campaign Builder *Design and launch a Reg CF or revenue-share capital raise from your portfolio and community* |
| :---: | :---- |

### **Purpose**

An end-to-end campaign design and launch tool for equity crowdfunding on the Kogi platform. Covers campaign type selection, offering document drafting, cap table setup, escrow configuration, investor communication, and Reg CF compliance checks.

| Feature | Description |
| :---- | :---- |
| **Campaign type selector** | Equity (Reg CF / Reg A+) · Revenue Share · SAFE Note · Convertible Note · Donation · Reward-based — with plain-language explanations |
| **Offering summary generator** | Oba drafts: business overview, use of funds, risk factors, team bios, financial summary, and investor return model |
| **Valuation input** | Cap table modeling: pre-money valuation, raise target, equity offered, post-money dilution visualization |
| **Campaign page builder** | Visual campaign page: video embed, team profiles, portfolio highlights, financials, FAQ, and investor updates |
| **Investor communication kit** | Templates: campaign announcement, update emails, milestone notifications, close announcement |
| **Reg CF compliance check** | CrowdfundingEngine checks: raise limit, investor accreditation requirements, disclosure completeness — flags before launch |
| **Escrow setup** | Campaign escrow account configured in kogi-bank; auto-refund on failure; milestone-disbursement on success |
| **Cap table integration** | Investor equity positions tracked in kogi-bank Equity Accounts; vesting enforced; dilution computed on new rounds |
| **Post-campaign reporting** | Automated investor update cadence; financial performance reports; distribution computation by Revenue Share Engine |

# **7\. AI Tools (Oba-Powered)**

The following tools are primarily AI-driven, powered by Oba — Kogi's platform AI assistant. They operate on live platform data and are deeply personalized to the user's portfolio, persona, work history, and goals.

| T-19 | Skills Mapper & Gap Analyzer *Visualize your full skill profile, identify gaps, and build a personalized development plan* |
| :---: | :---- |

### **Purpose**

Constructs a comprehensive, evidence-backed skills map from the user's portfolio contributions, gig work history, certifications, endorsements, and self-assessments. Compares the skill profile against target roles, opportunities, and market benchmarks to surface actionable gaps with concrete development paths.

| Feature | Description |
| :---- | :---- |
| **Skill profile assembly** | Aggregates skills from: profile declarations, project contributions, gig types claimed, certifications, endorsements, and MatchEngine usage signals |
| **Proficiency scoring** | Each skill scored across: frequency of use · peer endorsement count · portfolio evidence strength · recency of application |
| **Skill cluster map** | Visual cluster diagram grouping skills into domains (Technical, Creative, Business, Interpersonal, Domain-Specific) |
| **Gap analysis** | User specifies a target role, gig type, or career goal; Oba identifies the skill gaps between current profile and target requirement |
| **Market demand overlay** | MatchEngine surfaces which skills are most in-demand on the Kogi Marketplace based on current gig and job listings |
| **Development plan generator** | For each gap, Oba recommends: platform resources, certifications, courses, community mentors, practice projects, and suggested timeframe |
| **Professional development account link** | Recommended learning paths linked to kogi-bank Professional Development Account for funding |
| **Progress tracking** | Skills gained through completed projects and certifications auto-update the map; progress toward development plan milestones tracked |

| T-20 | Career Path & Rate Benchmarking Advisor *Understand where you are, where you can go, and what your work is worth in the market* |
| :---: | :---- |

### **Purpose**

An AI-powered career intelligence tool that analyzes the user's work history, income trajectory, skill profile, and market signals to provide personalized career path scenarios, income benchmarks, and strategic positioning advice.

| Feature | Description |
| :---- | :---- |
| **Career trajectory analysis** | Oba analyzes portfolio history and identifies: current career stage, progression velocity, specialization depth, and breadth of experience |
| **Path scenario generator** | Produces 3 alternative career path scenarios based on current profile: depth path (specialist), breadth path (generalist), or leadership path |
| **Market rate benchmarking** | Anonymized platform income data by: skill set, persona type, experience level, geography, engagement type (retainer/project/gig) |
| **Income gap analysis** | Compares current income trajectory against market benchmark; Oba identifies what changes in positioning, rates, or services close the gap |
| **Rate increase readiness** | Assesses whether portfolio evidence, endorsements, and skill depth support a rate increase — and by how much |
| **Positioning recommendations** | Oba recommends: how to niche or expand, which new service types to add, which communities or markets to enter |
| **Peer comparison (anonymized)** | How the user's metrics compare to similar workers on the platform — portfolio size, booking rate, income, endorsements |
| **90-day action plan** | Concrete, prioritized 90-day recommendations with tasks that seed directly into work management backlog |

| T-21 | Content Repurposer & Publishing Kit *Transform one piece of work into a full content strategy across every channel and format* |
| :---: | :---- |

### **Purpose**

Takes any portfolio artifact — a completed project, report, case study, talk, video, or document — and generates a full suite of derivative content assets formatted for every relevant distribution channel. Eliminates the gap between doing great work and communicating it effectively.

| Input Type | Derivative Outputs Generated |
| :---- | :---- |
| **Completed project / case study** | LinkedIn article · Twitter/X thread · Instagram carousel captions · Portfolio case study page · Email newsletter feature · Press release · Talk abstract |
| **Project report / research** | Executive summary · Blog post · Data visualization captions · Slide deck summary · Community post · Podcast talking points |
| **Video / talk / presentation** | Transcript clean-up · Chapter summaries · Quote pull cards · Blog post · Short-form clips script · Newsletter recap |
| **Client testimonial / review** | Quote card (social image spec) · Website testimonial copy · Proposal proof block · Portfolio highlight callout |
| **Portfolio analytics summary** | Impact statement (resume use) · Investor update · Community progress post · Grant report input · Annual review narrative |
| **New gig / service listing** | Service page copy · Social announcement · Email pitch · DM outreach script · Rate card line item |

| Feature | Description |
| :---- | :---- |
| **Channel optimization** | Formats output for: Kogi profile · LinkedIn · X · Instagram · Email · YouTube description · Podcast notes · Press release |
| **Tone variants** | For each output, generates 2–3 tone variants: professional, casual/conversational, and high-energy/promotional |
| **Hashtag generation** | Platform and channel-optimized hashtag sets per piece of content |
| **Scheduling suggestions** | Oba recommends optimal posting times per channel based on audience activity data |
| **Content calendar sync** | Published and scheduled items added to Brand Kit content calendar (T-04) |
| **Analytics loop** | Tracks engagement per published piece; Oba identifies top-performing content types and recommends doubling down |

| T-22 | Opportunity Matcher & Pitch Personalizer *Get matched to the right opportunities and pitch them with surgical precision* |
| :---: | :---- |

### **Purpose**

Combines the MatchEngine's opportunity discovery with AI-assisted, opportunity-specific pitch personalization. The tool continuously surfaces relevant gigs, jobs, grants, partnerships, and investment opportunities, then helps the user craft a tailored pitch response for each one in minutes.

| Feature | Description |
| :---- | :---- |
| **Opportunity feed** | MatchEngine delivers a personalized feed of: gigs, consulting opportunities, grants, partnership invitations, collaborative projects, and investment asks — ranked by fit score |
| **Fit score breakdown** | For each opportunity: why it's a match (skills, history, persona, community alignment), what to lead with, and potential concerns |
| **One-click pitch generation** | User selects an opportunity; Oba generates a tailored pitch drawing from portfolio, skills, past outcomes, and the opportunity brief |
| **Pitch variants** | Generates: short (DM/email pitch) · medium (proposal attachment) · full (proposal document via T-05) — user picks context |
| **Personalization signals** | Oba reads the opportunity description for language, tone, priorities, and requirements, then mirrors them in the pitch without losing authenticity |
| **Follow-up templates** | If no response after N days, Oba generates a contextual follow-up message; links to CRM pipeline (T-08) |
| **Rejection debrief** | If an opportunity is lost, Oba offers a brief debrief: what the portfolio evidence suggests, what could strengthen future applications |
| **Pipeline integration** | All pitched opportunities tracked in CRM pipeline; status updates and next actions managed from one view |

# **8\. Shared Tool Infrastructure**

## **8.1 Tool Output Pipeline**

Every tool in the Kogi Platform Tools catalog produces outputs through a unified pipeline ensuring consistent storage, access, and analytics.

| Pipeline Stage | Description |
| :---- | :---- |
| **Generation** | AI-assisted or template-driven content assembly; inputs validated; Oba review pass for quality and completeness |
| **Preview** | WYSIWYG preview with real-time edits; mobile and print preview available; accessibility checks |
| **Artifact Creation** | Output saved as a versioned Portfolio Artifact with resource\_type, content hash, and generation metadata |
| **Linking** | Artifact bi-directionally linked to: source portfolio items, CRM records, project backlogs, kogi-bank accounts, or community initiatives as relevant |
| **Publishing** | Optionally published to: public profile · Marketplace listing · community space · external shareable link · email send |
| **Analytics** | View, download, share, and conversion events tracked per artifact; surfaced on portfolio and tool dashboards |
| **Version History** | All prior versions retained; diff view between versions; restore to any version |
| **Collaboration** | Multi-user editing with contribution attribution; comment and review workflow; e-signature integration |

## **8.2 Engine Integrations**

| Engine | Tools That Use It | Capability Provided |
| :---- | :---- | :---- |
| **AnalyticsEngine** | T-01, T-10, T-11, T-21 | Portfolio analytics for auto-populating achievement metrics and invoice analytics |
| **PersonalizationEngine** | All tools | Persona-aware tone calibration, field pre-population, and template recommendation |
| **MatchEngine** | T-01, T-06, T-20, T-22 | Market rate benchmarking, opportunity matching, and skills-to-demand alignment |
| **RecommendationEngine** | T-19, T-20, T-22 | Opportunity feed personalization, development path recommendations |
| **SearchEngine** | T-01, T-05, T-12 | Skill and keyword indexing; ATS optimization; grant discovery |
| **GrantEngine** | T-12 | Grant registry matching, eligibility scoring, impact tracking |
| **CRMEngine** | T-05, T-08, T-10, T-22 | Pipeline sync, lead scoring, proposal tracking, invoice reconciliation |
| **RiskEngine** | T-05, T-07, T-12, T-16 | Risk register generation, scope risk detection, grant viability scoring |
| **CollaborationEngine** | T-13, T-15, T-16, T-17 | Multi-party document editing, contribution attribution, retro synthesis |
| **OptimizationEngine** | T-06, T-14, T-20 | Rate optimization, OKR ambition calibration, career path scenario modeling |
| **DataStreamingEngine** | T-10, T-11, T-21 | Real-time income tracking, content analytics feeds, invoice event triggers |
| **CrowdfundingEngine** | T-18 | Cap table modeling, Reg CF compliance, investor matching, escrow automation |
| **BenefitsEngine** | T-11 | Benefits contribution tracking, tax-advantaged account optimization |
| **BookingEngine** | T-08, T-09, T-10 | Booking lifecycle integration with contract and invoice automation |

## **8.3 Template & Asset Library**

Every tool draws from and contributes to the Kogi Template Library — a versioned, community-contributed catalog of document templates, clause libraries, design assets, and generation prompts.

| Library Component | Description |
| :---- | :---- |
| **Document Templates** | 30+ resume templates · 20+ proposal layouts · 15+ contract types · 10+ pitch deck formats · 8+ press kit layouts |
| **Clause Library** | 300+ contract clauses by type and jurisdiction; community-tagged; Oba-curated for clarity and completeness |
| **Achievement Phrases** | STAR-format achievement phrase bank organized by skill domain and impact type; used by resume and bio tools |
| **Brand Starters** | 10+ brand voice profiles by persona type; color palette starters; typography pairings |
| **Prompt Templates** | Curated Oba prompt templates for each tool; community-improvable; A/B tested for output quality |
| **Community Contributions** | Members share improved templates back to the library; steward-reviewed; attribution credited to contributor |

# **9\. Tool Catalog Summary**

| ID | Tool | Primary Users | Key Integration |
| :---- | :---- | :---- | :---- |
| **T-01** | Resume & Portfolio Highlights Builder | All workers, creatives, professionals | Portfolio System · AnalyticsEngine · MatchEngine |
| **T-02** | Bio & Elevator Pitch Generator | All users | Profile · PersonalizationEngine |
| **T-03** | Press Kit Builder | Artists, creatives, public figures, orgs | Portfolio Artifacts · CRM |
| **T-04** | Brand Kit & Social Media Kit | Freelancers, entrepreneurs, orgs | Profile · Content Calendar · Publishing |
| **T-05** | Proposal & Pitch Deck Generator | Freelancers, consultants, entrepreneurs | CRM · Portfolio · kogi-bank · Contract (T-09) |
| **T-06** | Rate & Pricing Calculator | All workers | MatchEngine · kogi-bank Income Data |
| **T-07** | Scope of Work Builder | Freelancers, consultants, service providers | Contracts (T-09) · Work Management Backlog |
| **T-08** | Client Onboarding Toolkit | All service workers | CRM · Contracts · Invoicing · Portfolio Space |
| **T-09** | Contract Template Generator | All workers and orgs | kogi-bank · e-Signature · Portfolio EventLog |
| **T-10** | Invoice & Payment Builder | All workers | kogi-bank Ledgers · CRM · Contract (T-09) |
| **T-11** | Tax & Income Estimator | All self-employed workers | kogi-bank · BenefitsEngine · Portable Benefits |
| **T-12** | Grant Application Builder | Workers, collectives, cooperatives, orgs | GrantEngine · Portfolio · kogi-bank Grants |
| **T-13** | Project Brief & Charter Generator | Project managers, teams, orgs | Work Management WBS · Portfolio · RiskEngine |
| **T-14** | OKR & Goal-Setting Tool | Individuals, teams, organizations | Work Management Analytics · Portfolio Goals |
| **T-15** | Retrospective & Review Facilitator | Teams, cooperatives, program managers | Work Management Backlog · Portfolio Artifacts |
| **T-16** | Cooperative Charter Builder | Cooperatives, collectives, federations | Org System · kogi-bank Treasury · Governance |
| **T-17** | Initiative & Campaign Planner | Organizers, activists, community leads | Community Initiatives · Work Management · Bank |
| **T-18** | Equity Crowdfunding Campaign Builder | Entrepreneurs, cooperatives, orgs | CrowdfundingEngine · kogi-bank Escrow · Cap Table |
| **T-19** | Skills Mapper & Gap Analyzer | All workers | MatchEngine · Professional Dev Account · Profile |
| **T-20** | Career Path & Rate Benchmarking Advisor | All workers | MatchEngine · AnalyticsEngine · OptimizationEngine |
| **T-21** | Content Repurposer & Publishing Kit | Creatives, professionals, entrepreneurs | Portfolio Artifacts · AnalyticsEngine · Brand Kit |
| **T-22** | Opportunity Matcher & Pitch Personalizer | All workers and orgs | MatchEngine · CRM · Proposals (T-05) |

KOGI — Independent Worker Operating System  ·  Platform Tools SDD  ·  v1.0  ·  March 2026