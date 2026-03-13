

**KOGI**

Independent Worker Operating System

**Portfolio System**

Updated: Shared Portfolios · Portfolio Collaboration · CrowdresourcingPortable Benefits as Portfolio Items · CollaborationEngine

v2.2  ·  Kogi Platform  ·  March 2026

# **18 — Shared Portfolios & Portfolio Collaboration**

A Shared Portfolio is a PortfolioSystem Portfolio component whose ComponentMetadata.owners\[\] contains two or more distinct user or organization entity IDs. All existing lifecycle, governance, CRDT, and analytics operations apply with the following multi-party extensions.

## **Shared Portfolio Types**

| Type | Owner Model | Governance |
| :---- | :---- | :---- |
| Team Portfolio | team entity | Admin-controlled; leader override |
| Organization Portfolio | organization entity | Multi-sig \+ governance proposal |
| Collective Portfolio | collective entity | Open contribution; light moderation |
| Cooperative Portfolio | cooperative entity | Full member vote required |
| Federation Portfolio | federation entity | Inter-org governance protocol |
| Crowdresourced Portfolio | open / community contributors | Contribution ledger; steward review \+ merge |

## **Contribution Attribution Model**

All contributions to a shared portfolio component are tracked as ContributionRecord objects appended to the component's EventLog:

ContributionRecord {

    record\_id:           UUID,

    contributor\_id:      EntityId,           // user | org | collective

    contribution\_type:   ContributionType,   // Labor | Capital | Asset |

                                             // Knowledge | Artifact | Code | Design | Data

    portfolio\_component\_id: ComponentId,

    contribution\_value:  ContributionValue { amount: Decimal, unit: String },

    attribution\_weight:  f64,               // used in distribution calculations

    governance\_status:   GovernanceStatus,  // PendingReview | Accepted | Rejected | Merged

    linked\_ledger\_entry: Option\<LedgerId\>,  // kogi-bank entry for capital contributions

}

## **Portfolio Collaboration Workflow States**

| State | Description |
| :---- | :---- |
| OPEN\_CONTRIBUTION | Portfolio owner creates CrowdresourcingCampaign; specifies types, review policy, reward rules |
| SUBMITTED | Contributor submits labor, asset, capital, or knowledge contribution |
| STEWARD\_REVIEW | Configurable gate: AutoAccept | StewardReview | GovernanceVote |
| MERGED / ACCEPTED | ContributionRecord appended to EventLog; attribution weight updated |
| DISTRIBUTION\_EVENT | Periodic or milestone-triggered; kogi-bank executes distribution to member accounts |

## **Resource Sharing — GraphEdge Extension**

Portfolio Resource Sharing is implemented as a ResourceShare edge in the GraphEdge system:

GraphEdge {

    edge\_type: GraphEdgeType::ResourceShare,

    source\_id: ComponentId,     // resource being shared

    target\_id: EntityId,        // space | org | user receiving access

    metadata: {

        share\_policy\_id:      PolicyId,

        access\_level:         Read | Fork | Contribute | Manage,

        expiry:               Option\<DateTime\>,

        attribution\_required: bool,

    }

}

## **Crowdresourcing Campaign Data Model**

CrowdresourcingCampaign {

    campaign\_id:          UUID,

    target\_component\_id:  ComponentId,

    contribution\_types:   Vec\<ContributionType\>,

    open\_to:              Visibility,   // Public | Space | Organization | Invite

    contribution\_review:  ReviewPolicy, // AutoAccept | StewardReview | GovernanceVote

    contributor\_rewards:  Vec\<RewardRule\>, // attribution\_weight | equity | payment

    linked\_bank\_account:  Option\<AccountId\>,

}

## **CollaborationEngine Integration**

* Compute real-time attribution weight updates for all contributors

* Detect merge conflicts between concurrent CRDT operations on shared components

* Score collaboration health: contributor diversity, contribution velocity, governance participation

* Generate Oba recommendations: "3 contributors have pending submissions awaiting steward review"

* Feed CollaborationScore into PortfolioHealth computational model

# **19 — Portable Benefits as Portfolio Items**

Every portable benefit account is a PortfolioItem (kind: BenefitAccount) within the worker's personal portfolio, providing full lifecycle, governance, versioning, and analytics infrastructure.

BenefitAccount (PortfolioItem) {

    benefit\_type:         BenefitType,  // Health | Dental | Vision | HSA |

                                         // Retirement | PTO | Disability |

                                         // ProfessionalDev | EmergencySavings | PortableSavings

    provider\_id:          ProviderId,

    contribution\_sources: Vec\<ContributionSource\>,

    balance:              Decimal,

    vesting\_schedule:     Option\<VestingSchedule\>,

    eligibility\_rules:    PolicySet,

    claim\_workflow\_id:    Option\<WorkflowId\>,

    tax\_treatment:        PreTax | PostTax | TaxFree,

    linked\_bank\_account:  AccountId,   // kogi-bank BenefitAccount

}

| BenefitType | Tax Treatment | Portfolio Analytics |
| :---- | :---- | :---- |
| Health (HSA) | Pre-tax | Coverage gap analysis, premium tracking, claim history |
| Retirement (SEP-IRA / 401k / PEP) | Pre-tax / Tax-deferred | Vesting progress, projected balance, contribution rate optimization |
| Paid Time Off | N/A | Accrual rate, usage, income replacement coverage |
| Disability / Workers' Comp | Post-tax (premiums) | Coverage adequacy vs. income, claim probability score |
| Professional Development | Post-tax | Skill development ROI, certification tracking, fund utilization |
| Portable Savings | Varies | Gig platform contribution tracking, balance projections, employer match equivalent |

