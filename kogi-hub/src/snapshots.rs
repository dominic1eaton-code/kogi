use serde_json::{json, Value};

pub type HubSnapshot = Value;

fn card(label: &str, value: &str, meta: &str, tone: &str) -> Value {
    json!({
        "label": label,
        "value": value,
        "meta": meta,
        "tone": tone,
    })
}

pub fn sample_hub_dashboard_snapshot() -> HubSnapshot {
    json!({
        "summary_cards": [
            card("Active Orgs", "18", "Federated teams", "text-[#60a5fa]"),
            card("Open Votes", "6", "Governance ballots", "text-[#10b981]"),
            card("Allocations", "$42,000", "Current cycle", "text-[#f59e0b]"),
            card("Negotiations", "3", "In progress", "text-[#a855f7]"),
        ],
        "agenda": [
            {"title": "Quarterly Budget Ratification", "date": "Mar 22", "status": "Voting live"},
            {"title": "Resource Allocation Review", "date": "Mar 24", "status": "Ready"},
            {"title": "Restitution Proposal 12", "date": "Mar 26", "status": "Draft"}
        ],
        "spotlight": [
            {"name": "Open Source Guild", "focus": "Governance update", "status": "Submitted"},
            {"name": "Coop Design Collective", "focus": "New member intake", "status": "Active"},
            {"name": "Autonomous Cell Sigma", "focus": "Resource request", "status": "Review"}
        ]
    })
}

pub fn sample_hub_governance_snapshot() -> HubSnapshot {
    json!({
        "summary_cards": [
            card("Policies", "24", "Active policies", "text-[#60a5fa]"),
            card("Proposals", "9", "In review", "text-[#10b981]"),
            card("Ratified", "17", "Last 90 days", "text-[#f59e0b]"),
            card("Compliance", "96%", "Audit ready", "text-[#a855f7]"),
        ],
        "proposals": [
            {"title": "Budget Allocation Q2", "status": "Voting", "owner": "Finance Council"},
            {"title": "Community Charter Update", "status": "Review", "owner": "Steward Circle"},
            {"title": "Resource Access Policy", "status": "Draft", "owner": "Operations"},
            {"title": "Restitution Protocol", "status": "Consensus", "owner": "Justice Council"}
        ],
        "frameworks": [
            {"name": "Holonic Governance", "desc": "Nested councils with clear mandates."},
            {"name": "Consensus + Delegate", "desc": "Hybrid voting with fallback delegates."},
            {"name": "Cooperative Charter", "desc": "Member rights, obligations, and dividends."}
        ]
    })
}

pub fn sample_hub_voting_snapshot() -> HubSnapshot {
    json!({
        "summary_cards": [
            card("Active Votes", "6", "3 closing soon", "text-[#10b981]"),
            card("Participation", "78%", "7d rolling", "text-[#60a5fa]"),
            card("Quorum", "62%", "Target 55%", "text-[#f59e0b]"),
            card("Delegations", "24", "Trusted delegates", "text-[#a855f7]"),
        ],
        "ballots": [
            {"title": "Budget Allocation Q2", "status": "Open", "close": "Mar 22"},
            {"title": "Resource Access Policy", "status": "Open", "close": "Mar 24"},
            {"title": "Restitution Proposal 12", "status": "Consensus", "close": "Mar 25"},
            {"title": "Community Charter Update", "status": "Draft", "close": "Mar 28"}
        ],
        "voters": [
            {"group": "Council Delegates", "weight": "32%", "status": "Aligned"},
            {"group": "Member Assembly", "weight": "48%", "status": "Voting"},
            {"group": "Advisory Board", "weight": "20%", "status": "Pending"}
        ]
    })
}

pub fn sample_hub_allocation_snapshot() -> HubSnapshot {
    json!({
        "summary_cards": [
            card("Allocation Pool", "$64,000", "Quarterly budget", "text-[#10b981]"),
            card("Programs Funded", "12", "Across 5 orgs", "text-[#60a5fa]"),
            card("Requests", "8", "Awaiting vote", "text-[#f59e0b]"),
            card("Utilization", "71%", "Budget used", "text-[#a855f7]"),
        ],
        "allocations": [
            {"program": "Community Infrastructure", "amount": "$18,000", "owner": "Operations", "status": "Approved"},
            {"program": "Open Source Fund", "amount": "$12,000", "owner": "Guild Council", "status": "Voting"},
            {"program": "Education Access", "amount": "$9,500", "owner": "Collective Care", "status": "Draft"},
            {"program": "Mutual Aid", "amount": "$6,800", "owner": "Treasury", "status": "Approved"}
        ],
        "forecasts": [
            {"label": "Reserve Target", "value": "$22,000", "status": "On track"},
            {"label": "Next Allocation Cycle", "value": "Apr 10", "status": "Scheduled"},
            {"label": "Unused Budget", "value": "$18,400", "status": "Available"}
        ]
    })
}

pub fn sample_hub_distribution_snapshot() -> HubSnapshot {
    json!({
        "summary_cards": [
            card("Total Distribution", "$36,200", "Current cycle", "text-[#10b981]"),
            card("Recipients", "28", "Teams and members", "text-[#60a5fa]"),
            card("Pending", "$6,800", "Awaiting approvals", "text-[#f59e0b]"),
            card("Next Run", "Mar 24", "Biweekly payout", "text-[#a855f7]"),
        ],
        "payouts": [
            {"name": "Open Source Guild", "amount": "$8,400", "status": "Approved"},
            {"name": "Coop Team Alpha", "amount": "$5,200", "status": "Scheduled"},
            {"name": "Mutual Aid Pool", "amount": "$3,600", "status": "Pending vote"},
            {"name": "Research Cell", "amount": "$2,400", "status": "Approved"}
        ],
        "rules": [
            {"rule": "40% to Operations", "detail": "Covers core infrastructure"},
            {"rule": "25% to Community Dividends", "detail": "Member payouts"},
            {"rule": "20% to Innovation", "detail": "R and D funds"},
            {"rule": "15% to Mutual Aid", "detail": "Care and emergency"}
        ]
    })
}

pub fn sample_hub_collaboration_snapshot() -> HubSnapshot {
    json!({
        "summary_cards": [
            card("Working Groups", "14", "Cross-org teams", "text-[#10b981]"),
            card("Shared Projects", "9", "Active collaborations", "text-[#60a5fa]"),
            card("Open Requests", "5", "Needs staffing", "text-[#f59e0b]"),
            card("Partners", "22", "Verified orgs", "text-[#a855f7]"),
        ],
        "groups": [
            {"name": "Open Source Council", "focus": "Release planning", "status": "Active"},
            {"name": "Community Care", "focus": "Mutual aid protocols", "status": "Active"},
            {"name": "Infrastructure Guild", "focus": "Shared tooling", "status": "Recruiting"}
        ],
        "requests": [
            {"title": "Design System Sprint", "need": "2 designers", "status": "Open"},
            {"title": "Governance Docs Audit", "need": "1 researcher", "status": "Open"},
            {"title": "Data Migration", "need": "1 engineer", "status": "Pending review"}
        ]
    })
}

pub fn sample_hub_restitution_snapshot() -> HubSnapshot {
    json!({
        "summary_cards": [
            card("Open Cases", "4", "Active mediation", "text-[#f59e0b]"),
            card("Resolved", "12", "Last 12 months", "text-[#10b981]"),
            card("Restitution Fund", "$14,600", "Available", "text-[#60a5fa]"),
            card("Policies", "3", "Justice protocols", "text-[#a855f7]"),
        ],
        "cases": [
            {"title": "Resource Access Dispute", "status": "Mediation", "owner": "Justice Council"},
            {"title": "Equity Allocation Appeal", "status": "Review", "owner": "Governance"},
            {"title": "Grant Compliance Issue", "status": "Resolution", "owner": "Treasury"}
        ],
        "actions": [
            {"step": "Collect statements and evidence", "status": "In progress"},
            {"step": "Draft restitution plan", "status": "Pending review"},
            {"step": "Schedule community circle", "status": "Mar 23"}
        ]
    })
}

pub fn sample_hub_negotiations_snapshot() -> HubSnapshot {
    json!({
        "summary_cards": [
            card("Active Talks", "5", "Cross-org deals", "text-[#10b981]"),
            card("Term Sheets", "3", "Drafting", "text-[#60a5fa]"),
            card("Mediation", "1", "Needs review", "text-[#f59e0b]"),
            card("Closings", "2", "This month", "text-[#a855f7]"),
        ],
        "negotiations": [
            {"title": "Resource Sharing Agreement", "parties": "Collective + Coop Alpha", "status": "Drafting"},
            {"title": "IP Licensing Deal", "parties": "Open Source Guild + Studio", "status": "Review"},
            {"title": "Service Exchange", "parties": "Federation + Cell Sigma", "status": "Negotiation"}
        ],
        "checklist": [
            {"step": "Confirm allocation terms", "status": "In progress"},
            {"step": "Finalize governance clauses", "status": "Pending"},
            {"step": "Approve signature authority", "status": "Mar 24"}
        ]
    })
}

pub fn sample_hub_teams_snapshot() -> HubSnapshot {
    json!({
        "summary_cards": [
            card("Teams", "26", "Across orgs", "text-[#10b981]"),
            card("Active Members", "140", "Verified profiles", "text-[#60a5fa]"),
            card("Open Roles", "18", "Seeking talent", "text-[#f59e0b]"),
            card("Pods", "9", "Autonomous cells", "text-[#a855f7]"),
        ],
        "teams": [
            {"name": "Design Collective", "focus": "Brand systems", "status": "Active"},
            {"name": "Operations Pod", "focus": "Finance + compliance", "status": "Active"},
            {"name": "Community Care", "focus": "Support + onboarding", "status": "Recruiting"}
        ],
        "roles": [
            {"role": "Governance Facilitator", "team": "Steward Circle", "status": "Open"},
            {"role": "Data Steward", "team": "Infrastructure Guild", "status": "Review"},
            {"role": "Product Lead", "team": "Open Source Guild", "status": "Open"}
        ]
    })
}

pub fn sample_hub_organizations_snapshot() -> HubSnapshot {
    json!({
        "summary_cards": [
            card("Organizations", "32", "Verified entities", "text-[#10b981]"),
            card("Federations", "4", "Shared governance", "text-[#60a5fa]"),
            card("Charters", "28", "Active charters", "text-[#f59e0b]"),
            card("Compliance", "94%", "On track", "text-[#a855f7]"),
        ],
        "organizations": [
            {"name": "Kogi Studios", "type": "Studio", "status": "Active"},
            {"name": "Open Source Guild", "type": "Guild", "status": "Active"},
            {"name": "Community Trust", "type": "Trust", "status": "Review"},
            {"name": "Coop Alpha", "type": "Cooperative", "status": "Active"}
        ],
        "compliance": [
            {"item": "Charter renewal - Community Trust", "status": "Due Apr 4"},
            {"item": "Policy audit - Kogi Studios", "status": "In progress"},
            {"item": "Federation review - North Cluster", "status": "Scheduled"}
        ]
    })
}

pub fn sample_hub_collectives_snapshot() -> HubSnapshot {
    json!({
        "summary_cards": [
            card("Collectives", "11", "Active groups", "text-[#10b981]"),
            card("Members", "84", "Cross-org", "text-[#60a5fa]"),
            card("Shared Funds", "$22,600", "Treasury pools", "text-[#f59e0b]"),
            card("Projects", "19", "Open initiatives", "text-[#a855f7]"),
        ],
        "collectives": [
            {"name": "Design Commons", "focus": "Shared design systems", "status": "Active"},
            {"name": "Research Collective", "focus": "Policy and strategy", "status": "Active"},
            {"name": "Community Care", "focus": "Mutual aid", "status": "Recruiting"}
        ],
        "charters": [
            {"name": "Design Commons Charter", "status": "Signed"},
            {"name": "Research Collective Charter", "status": "Draft"},
            {"name": "Care Circle Charter", "status": "Review"}
        ]
    })
}

pub fn sample_hub_cooperatives_snapshot() -> HubSnapshot {
    json!({
        "summary_cards": [
            card("Cooperatives", "6", "Member owned", "text-[#10b981]"),
            card("Member Owners", "72", "Active members", "text-[#60a5fa]"),
            card("Dividends", "$12,400", "Next distribution", "text-[#f59e0b]"),
            card("Governance", "100%", "Charter compliant", "text-[#a855f7]"),
        ],
        "coops": [
            {"name": "Coop Alpha", "focus": "Production", "status": "Active"},
            {"name": "Coop Beta", "focus": "Research", "status": "Active"},
            {"name": "Coop Gamma", "focus": "Community services", "status": "Onboarding"}
        ],
        "dividends": [
            {"cycle": "Q1 2026", "amount": "$6,800", "status": "Scheduled"},
            {"cycle": "Q4 2025", "amount": "$5,600", "status": "Paid"}
        ]
    })
}

pub fn sample_hub_federations_snapshot() -> HubSnapshot {
    json!({
        "summary_cards": [
            card("Federations", "4", "Multi-org networks", "text-[#10b981]"),
            card("Member Orgs", "21", "Active nodes", "text-[#60a5fa]"),
            card("Shared Assets", "$92,000", "Federated pools", "text-[#f59e0b]"),
            card("Agreements", "9", "Active MOUs", "text-[#a855f7]"),
        ],
        "federations": [
            {"name": "North Cluster", "focus": "Infrastructure sharing", "status": "Active"},
            {"name": "Open Knowledge Network", "focus": "Research exchange", "status": "Active"},
            {"name": "Care Alliance", "focus": "Mutual aid", "status": "Review"}
        ],
        "agreements": [
            {"title": "Shared Services MOU", "status": "Signed"},
            {"title": "Resource Exchange Protocol", "status": "Draft"},
            {"title": "Data Governance Policy", "status": "Review"}
        ]
    })
}

pub fn sample_hub_autonomous_snapshot() -> HubSnapshot {
    json!({
        "summary_cards": [
            card("Autonomous Cells", "9", "Independent teams", "text-[#10b981]"),
            card("Active Missions", "14", "Self-managed", "text-[#60a5fa]"),
            card("Cell Budgets", "$48,500", "Allocated funds", "text-[#f59e0b]"),
            card("Compliance", "90%", "Charter alignment", "text-[#a855f7]"),
        ],
        "cells": [
            {"name": "Cell Sigma", "focus": "Infrastructure tooling", "status": "Active"},
            {"name": "Cell Aurora", "focus": "Community onboarding", "status": "Active"},
            {"name": "Cell Delta", "focus": "Research and policy", "status": "Review"}
        ],
        "charters": [
            {"name": "Sigma Charter", "status": "Signed"},
            {"name": "Aurora Charter", "status": "Signed"},
            {"name": "Delta Charter", "status": "Draft"}
        ]
    })
}

pub fn sample_hub_open_source_snapshot() -> HubSnapshot {
    json!({
        "summary_cards": [
            card("Projects", "22", "Open source repos", "text-[#10b981]"),
            card("Contributors", "148", "Active this quarter", "text-[#60a5fa]"),
            card("Releases", "7", "This month", "text-[#f59e0b]"),
            card("Funding", "$18,200", "Sponsor pool", "text-[#a855f7]"),
        ],
        "projects": [
            {"name": "Kogi UI System", "status": "Maintained", "owner": "Design Guild"},
            {"name": "Governance Toolkit", "status": "Release candidate", "owner": "Ops Guild"},
            {"name": "Community API", "status": "Active", "owner": "Engineering"}
        ],
        "sponsors": [
            {"name": "Open Collective", "amount": "$6,400", "status": "Active"},
            {"name": "Local Funders", "amount": "$4,800", "status": "Pending"},
            {"name": "Community Grants", "amount": "$2,200", "status": "Approved"}
        ]
    })
}

pub fn sample_hub_group_economics_snapshot() -> HubSnapshot {
    json!({
        "summary_cards": [
            card("Group Treasury", "$148,200", "Federated funds", "text-[#10b981]"),
            card("Economic Policies", "8", "Active rulesets", "text-[#60a5fa]"),
            card("Distribution Cycle", "Monthly", "Next run Apr 1", "text-[#f59e0b]"),
            card("Mutual Aid", "$18,400", "Reserve", "text-[#a855f7]"),
        ],
        "economics": [
            {"item": "Operations Allocation", "amount": "$48,000", "status": "Active"},
            {"item": "Community Dividend", "amount": "$26,000", "status": "Active"},
            {"item": "Innovation Pool", "amount": "$18,500", "status": "Review"}
        ],
        "policies": [
            {"name": "Solidarity Distribution", "status": "Approved"},
            {"name": "Mutual Aid Escrow", "status": "Active"},
            {"name": "Shared Asset Policy", "status": "Draft"}
        ]
    })
}

pub fn sample_hub_resource_crowdfund_snapshot() -> HubSnapshot {
    json!({
        "summary_cards": [
            card("Campaigns", "5", "Resource drives", "text-[#10b981]"),
            card("Raised", "$42,600", "This quarter", "text-[#60a5fa]"),
            card("Backers", "240", "Community support", "text-[#f59e0b]"),
            card("Match Pool", "$8,000", "Available match", "text-[#a855f7]"),
        ],
        "campaigns": [
            {"name": "Community Equipment Fund", "goal": "$20,000", "raised": "$14,200", "status": "Live"},
            {"name": "Open Source Maintenance", "goal": "$12,000", "raised": "$9,600", "status": "Live"},
            {"name": "Care Relief Sprint", "goal": "$6,500", "raised": "$5,300", "status": "Closing"}
        ],
        "needs": [
            {"item": "Audio gear for media lab", "status": "Requested"},
            {"item": "Accessibility upgrades", "status": "Reviewed"},
            {"item": "Community travel fund", "status": "New"}
        ]
    })
}

pub fn sample_hub_community_showcase_snapshot() -> HubSnapshot {
    json!({
        "summary_cards": [
            card("Showcases", "12", "Featured stories", "text-[#10b981]"),
            card("Featured Orgs", "8", "Rotating highlights", "text-[#60a5fa]"),
            card("Events", "5", "Upcoming", "text-[#f59e0b]"),
            card("Contributions", "260", "Community wins", "text-[#a855f7]"),
        ],
        "highlights": [
            {"title": "Community Media Lab", "focus": "Open access studio", "status": "Featured"},
            {"title": "Cooperative Launch Week", "focus": "Member onboarding", "status": "Live"},
            {"title": "Open Source Sprint", "focus": "Tooling release", "status": "Showcase"}
        ],
        "events": [
            {"name": "Governance Town Hall", "date": "Mar 23", "status": "Scheduled"},
            {"name": "Community Demo Day", "date": "Mar 27", "status": "Open"},
            {"name": "Funding Showcase", "date": "Apr 2", "status": "Planned"}
        ]
    })
}

pub fn sample_hub_contracts_snapshot() -> HubSnapshot {
    json!({
        "summary_cards": [
            card("Active Contracts", "14", "Across entities", "text-[#10b981]"),
            card("Agreements", "9", "Live terms", "text-[#60a5fa]"),
            card("Licenses", "6", "IP coverage", "text-[#f59e0b]"),
            card("Negotiations", "3", "In progress", "text-[#a855f7]"),
        ],
        "contracts": [
            {"title": "Studio Collaboration Agreement", "type": "Agreement", "status": "Active"},
            {"title": "Open Source Licensing", "type": "License", "status": "Review"},
            {"title": "Federation Services MOU", "type": "MOU", "status": "Draft"}
        ],
        "rights": [
            {"title": "Brand Usage Rights", "status": "Active"},
            {"title": "Distribution Rights", "status": "Negotiation"},
            {"title": "Trademark Protection", "status": "Reviewed"}
        ]
    })
}

pub fn sample_hub_ip_snapshot() -> HubSnapshot {
    json!({
        "summary_cards": [
            card("IP Assets", "18", "Registered", "text-[#10b981]"),
            card("Patents", "4", "Filed", "text-[#60a5fa]"),
            card("Trademarks", "6", "Active", "text-[#f59e0b]"),
            card("Licenses", "9", "In force", "text-[#a855f7]"),
        ],
        "assets": [
            {"name": "Kogi Identity System", "type": "Patent", "status": "Filed"},
            {"name": "Community Brand Kit", "type": "Trademark", "status": "Active"},
            {"name": "Media Lab Toolkit", "type": "Copyright", "status": "Active"}
        ],
        "registrations": [
            {"title": "Logo Mark", "status": "Approved"},
            {"title": "Coop Alpha Name", "status": "Review"},
            {"title": "Studio IP Bundle", "status": "Draft"}
        ]
    })
}
