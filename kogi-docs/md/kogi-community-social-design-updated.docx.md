

**KOGI**

Independent Worker Operating System

**Community, Spaces & Social System**

Updated: Shared Portfolios · Portfolio Collaboration · Group Economics · InitiativesBooking/CRM/Logistics · Crowdresourcing · Grants & Microfinancing

v2.1  ·  Kogi Platform  ·  March 2026

# **15 — Shared Portfolios & Portfolio Collaboration**

The Kogi Community layer is where shared portfolios and portfolio collaboration come to life. Any space, organization, collective, cooperative, or federation can create and manage a shared portfolio — jointly owned, governed, and attributed to multiple members.

## **Shared Portfolio Types**

| Type | Description | Governance Model |
| :---- | :---- | :---- |
| Team Portfolio | Shared working portfolio for a project team or squad | Admin-controlled; team leader can override |
| Organization Portfolio | Formal portfolio for an org entity (LLC, Corp, Trust, Fund) | Multi-sig \+ governance proposal for major changes |
| Collective Portfolio | Informal shared portfolio for a resource-sharing collective | Open contribution; light moderation |
| Cooperative Portfolio | Worker-owned portfolio with equity distribution | Full member vote required for major changes |
| Federation Portfolio | Cross-org portfolio spanning multiple federated spaces | Inter-org governance protocols; federated identity |
| Crowdresourced Portfolio | Portfolio component open to community contributions | Contribution ledger; steward review and merge |

## **Portfolio Resource Sharing — Resource Categories**

| Category | Examples |
| :---- | :---- |
| Artifacts & Templates | Contract templates, design files, playbooks, guides, documentation sets |
| Tools & Toolchains | Shared tool integrations, toolkits, automated workflows, CI/CD pipelines |
| Capital & Credits | Shared budget allocations, resource credits, compute pools |
| Labor & Skills | Time contributions, skill offers, task assignments within collaborative projects |
| Knowledge & Data | Research, datasets, analysis outputs, strategic intelligence |
| Contacts & Networks | Shared contact books, client databases, lead pools (with consent) |

## **Contribution Attribution Data Model**

All contributions to a shared portfolio item are tracked as ContributionRecord objects appended to the component's EventLog:

ContributionRecord {

    contributor\_id:          EntityId,           // user | org | collective

    contribution\_type:       ContributionType,   // Labor | Capital | Asset | Knowledge | Artifact

    contribution\_value:      { amount, unit, valuation\_method }

    attribution\_weight:      f64,               // used in distribution calculations

    governance\_status:       PendingReview | Accepted | Rejected | Merged

    linked\_ledger\_entry:     Option\<LedgerId\>,  // kogi-bank entry for capital contributions

}

## **Portfolio Collaboration Workflow**

| Stage | Description |
| :---- | :---- |
| Open Contribution | Portfolio owner creates CrowdresourcingCampaign; specifies contribution types, review policy, reward rules |
| Contribution Submitted | Contributor submits labor, asset, capital, or knowledge contribution |
| Steward / Governance Review | Configurable: AutoAccept | StewardReview | GovernanceVote |
| Merge / Accept | ContributionRecord created; appended to EventLog; attribution weight updated |
| Distribution Event | Periodic or milestone-triggered; kogi-bank executes distribution to member accounts |

# **16 — Booking, Scheduling, CRM & Logistics**

## **Booking & Scheduling System (KBKG)**

| Feature | Description |
| :---- | :---- |
| Centralized Calendars | Unified calendar aggregating all bookings, events, project milestones, personal schedules |
| Multi-Worker / Multi-Artist Dashboards | Dashboard showing all member schedules, availability windows, booking conflicts |
| Conflict Detection | Automated detection of scheduling conflicts across team members, resources, venues; Oba alerts |
| Event Booking & Reservation Management | Full booking lifecycle: request → quote → confirm → deposit → delivery → close |
| Availability Management | Worker-defined availability windows with blackout dates, recurring blocks, buffer time |
| Resource Booking | Book equipment, studio space, venues, or shared resources alongside worker time |
| Booking Analytics | Utilization rates, booking-to-revenue conversion, peak demand periods, cancellation rates |

## **CRM & Lead Management System (KCRM)**

| Feature | Description |
| :---- | :---- |
| Lead Capture Forms | Embeddable forms linked to worker/org profile pages; leads flow into CRM pipeline |
| Client Database Management | Searchable client records: contact info, booking history, spend, communication log, tags, notes |
| Pipeline Management | Kanban pipeline: Inquiry → Qualified → Proposal → Negotiation → Booked → Retained |
| Automated Follow-up Emails | Triggered sequences on stage changes, quote expiry, post-event follow-ups via kogi-providers |
| Lead Scoring | CRMEngine scores leads by probability-to-book based on interaction history, budget signals, match quality |
| Client Communication Log | All messages, emails, calls logged against the client's CRM record automatically |
| Retention Analytics | Repeat booking rates, lifetime value, churn signals; Oba recommends re-engagement campaigns |

## **Contracts & Invoicing**

| Feature | Description |
| :---- | :---- |
| Customizable Automated Contracts | Contract templates with variable fields auto-populated from booking details; custom clauses supported |
| E-Signatures | Integrated e-signature workflow (DocuSign, HelloSign via ProviderSystem); status tracked |
| Automated Payment Reminders | Configurable deposit and balance due reminders; sent via kogi-bank invoice system |
| Recurring Billing | Subscription-based or retainer billing with automated recurring invoice generation |
| Contract Version Control | All contract revisions versioned; full audit trail in Portfolio System |

## **Logistics & Communication System (KLGS)**

| Feature | Description |
| :---- | :---- |
| Tour / Project Itinerary Planning | Multi-stop itinerary builder with timeline view; links to bookings, venues, and team assignments |
| Resource Allocation | Assign equipment, vehicles, and crew to specific dates/times/locations; conflict detection |
| Logistics Tracking | Live tracking: equipment rental status, transportation, crew check-ins, venue setup progress |
| Equipment Rental Management | Owned and rented equipment inventory; rental period tracking; return reminders; cost ledgering |
| Transportation Management | Vehicle assignments, routing, mileage tracking, driver schedules |
| Crew Schedule Management | Role-based crew assignments per event/day; call times, breaks, wrap times; automated distribution |
| Finance & Reporting | Real-time expense tracking vs. event budget; P\&L per booking; tax report generation |

# **17 — Initiatives**

Initiatives are platform-coordinated programs for collective action within the Kogi Community — structured frameworks for organizations, collectives, and cooperatives to coordinate around shared goals.

| Initiative Type | Description |
| :---- | :---- |
| Portable Benefits Initiative | Coordinate independent workers to pool buying power for health, dental, vision, or retirement through a PEP or collective insurance arrangement |
| Grants Initiative | Community-organized grant discovery, collective application, and fund distribution; managed by a designated steward committee |
| Microfinancing Initiative | Community lending pool where members contribute capital and access microloans at community-set rates; governed by lending policy proposals |
| Group Economics Initiative | Formal program for collective purchasing, shared revenue, cooperative investment, or group crowdfunding |
| Equity Crowdfunding Initiative | Platform-coordinated campaign series where multiple community members or projects raise capital from the broader community |
| Crowdresourcing Initiative | Open call for community contributions (labor, assets, knowledge) to a shared portfolio program or community infrastructure project |
| Shared Portfolio Initiative | Structured program to create, manage, and grow a federation-level or community-level shared portfolio |

# **Updated Terminology**

| Term | Definition |
| :---- | :---- |
| Shared Portfolio | A portfolio jointly owned, managed, or contributed to by multiple users, teams, or organizations |
| Portfolio Collaboration | A structured multi-contributor workflow on a shared portfolio component with attribution tracking |
| Group Economics | Collective financial structures: cooperative treasury, revenue sharing, group investment, mutual aid |
| Portable Benefits | Worker-centered benefits (health, retirement, PTO, insurance) that travel with the individual, not the employer |
| Crowdresourcing | Community-sourced contributions of labor, assets, and knowledge to shared portfolio items |
| Equity Crowdfunding | Community capital raising via Reg CF equity, revenue share, SAFE, or donation instruments |
| Initiative | A platform-coordinated program for collective action: portable benefits, grants, group economics, crowdresourcing |
| Grant Pool | A collectively managed fund for distributing grants to community members or projects |
| BenefitsEngine | Engine subsystem for benefits eligibility scoring, contribution tracking, and coverage optimization |
| CollaborationEngine | Engine subsystem coordinating shared portfolio workflows, contribution attribution, conflict resolution |
| BookingEngine | Engine subsystem for scheduling optimization, conflict detection, and resource allocation |
| CRMEngine | Engine subsystem for lead scoring, pipeline management, and follow-up automation |
| LogisticsEngine | Engine subsystem for itinerary planning, resource routing, and crew/equipment scheduling |

