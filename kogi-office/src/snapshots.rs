use serde_json::{json, Value};

pub type OfficeSnapshot = Value;

fn metric(label: &str, value: &str, tone: &str) -> Value {
    json!({
        "label": label,
        "value": value,
        "tone": tone,
    })
}

fn tag(label: &str, tone: &str) -> Value {
    json!({
        "label": label,
        "tone": tone,
    })
}

fn status_item(name: &str, status: &str, tone: &str) -> Value {
    json!({
        "name": name,
        "status": status,
        "tone": tone,
    })
}

pub fn sample_overview_snapshot() -> OfficeSnapshot {
    json!({
        "summary_cards": [
            metric("Active Projects", "6", "text-[#10b981]"),
            metric("Upcoming Events", "14", "text-[#60a5fa]"),
            metric("Inbox Unread", "9", "text-[#f59e0b]"),
            metric("Studio Ideas", "42", "text-[#a855f7]"),
        ],
        "active_workstreams": [
            {"title": "Platform Release Sprint", "progress": "72%", "note": "3 blocked · 12 due"},
            {"title": "Client Onboarding Pack", "progress": "5 tasks", "note": "Contracts + kickoff"},
            {"title": "Cooperative Governance", "progress": "Vote due Fri", "note": "Agenda + proposals"},
        ],
        "upcoming_schedule": [
            {"title": "Studio Sync", "time": "10:00 AM", "duration": "30m"},
            {"title": "Client Review", "time": "1:00 PM", "duration": "1h"},
            {"title": "Governance Office Hours", "time": "4:30 PM", "duration": "45m"},
        ],
        "inbox_summary": [
            status_item("Direct Messages", "9 unread", "text-[#10b981]"),
            status_item("Message Requests", "3 pending", "text-[#f59e0b]"),
            status_item("Email Threads", "5 flagged", "text-[#8ea6ad]"),
        ],
        "studio_pipeline": [
            {"label": "Ideas & Concepts", "value": "42"},
            {"label": "Mockups in review", "value": "6"},
            {"label": "Testing cycles", "value": "3 active"},
        ],
        "integrations": [
            tag("Jira", "text-[#60a5fa]"),
            tag("Google Calendar", "text-[#10b981]"),
            tag("Figma", "text-[#a855f7]"),
            tag("Notion", "text-[#f59e0b]"),
        ],
        "oba_note": "Office syncs portfolio workstreams, schedule, and studio assets."
    })
}

pub fn sample_inbox_snapshot() -> OfficeSnapshot {
    json!({
        "filters": ["All", "Unread", "Requests"],
        "focus_modes": ["Off", "DND", "Deep Work"],
        "requests_pending": 3,
        "threads": [
            {"name": "Amara Kofi", "preview": "CRDT diff ready", "unread": true, "time": "2m"},
            {"name": "Sofia Reyes", "preview": "Token export ready", "unread": false, "time": "18m"},
            {"name": "Marcus Park", "preview": "Staging up", "unread": false, "time": "1h"},
            {"name": "Oba Assistant", "preview": "3 gig matches", "unread": false, "time": "2h"}
        ],
        "active_thread": {
            "name": "Amara Kofi",
            "status": "Online",
            "messages": [
                {"from": "Amara", "text": "Finished the CRDT merge. Review before PR?", "time": "Today"},
                {"from": "You", "text": "Share the diff + conflict strategy notes.", "time": "Today"},
                {"from": "Amara", "text": "Key change is merge_ops() resolution.", "time": "Today"}
            ]
        },
        "sidebar": {
            "match_score": "97",
            "mutuals": 14,
            "shared_skills": ["Rust", "CRDT", "TypeScript"],
            "shared_files": ["crdt-arch-notes-v2.md", "Q1 deliverables.md"]
        }
    })
}

pub fn sample_schedule_snapshot() -> OfficeSnapshot {
    json!({
        "summary_cards": [
            metric("Active Timeboxes", "12", "text-[#10b981]"),
            metric("Schedules Linked", "4", "text-[#60a5fa]"),
            metric("Conflicts", "3", "text-[#f59e0b]"),
            metric("Capacity", "62%", "text-[#a855f7]")
        ],
        "schedulebooks": [
            {"name": "Master Office Schedule", "status": "Primary"},
            {"name": "Client Delivery Timeline", "status": "Linked"},
            {"name": "Community Governance", "status": "Synced"},
            {"name": "Personal Timeboxes", "status": "Local"}
        ],
        "today_timeline": [
            {"time": "09:00 AM", "title": "Sprint Planning", "note": "Timebox: Delivery M4", "tone": "text-[#10b981]"},
            {"time": "11:30 AM", "title": "Client Review", "note": "Google Meet · demo", "tone": "text-[#60a5fa]"},
            {"time": "02:00 PM", "title": "Deep Work Block", "note": "Prototype iteration", "tone": "text-[#f59e0b]"}
        ],
        "timeboxes": [
            {"title": "Sprint 14", "window": "Mar 10 - Mar 21", "note": "12 tasks"},
            {"title": "Design Testbed", "window": "UX tests", "note": "4 sessions"},
            {"title": "Governance Cycle", "window": "Proposal window", "note": "Voting open"}
        ],
        "reconciliation": [
            {"status": "Overlap Detected", "note": "Client Review · Studio Sync"},
            {"status": "Resource Conflict", "note": "Studio Room reserved by DAO call"},
            {"status": "Resolved", "note": "Timebox moved to 3:00 PM"}
        ],
        "external_calendars": [
            {"name": "Google Calendar", "status": "Synced"},
            {"name": "Meetup Events", "status": "Linked"},
            {"name": "Calendly", "status": "Pending"}
        ]
    })
}

pub fn sample_calendar_snapshot() -> OfficeSnapshot {
    json!({
        "summary_cards": [
            metric("Calendars", "7", "text-[#10b981]"),
            metric("External Links", "3", "text-[#60a5fa]"),
            metric("Events This Week", "18", "text-[#f59e0b]"),
            metric("Conflicts", "2", "text-[#a855f7]")
        ],
        "calendars": [
            {"name": "Master Office Calendar", "status": "Primary"},
            {"name": "Client Delivery", "status": "Team"},
            {"name": "Governance", "status": "Community"},
            {"name": "Personal Focus", "status": "Local"}
        ],
        "external_sources": [
            {"name": "Google Calendar", "status": "Synced"},
            {"name": "Meetup", "status": "Linked"},
            {"name": "Outlook", "status": "Pending"}
        ],
        "today_events": [
            {"title": "Studio Sync", "time": "10:00 AM", "calendar": "Master"},
            {"title": "Client Review", "time": "1:00 PM", "calendar": "Google Meet"}
        ],
        "upcoming": [
            {"title": "Governance Call", "date": "Mar 17", "calendar": "Community"},
            {"title": "Prototype Demo", "date": "Mar 19", "calendar": "Studio"}
        ]
    })
}

pub fn sample_contacts_snapshot() -> OfficeSnapshot {
    json!({
        "summary_cards": [
            metric("Total", "1,240", "text-[#10b981]"),
            metric("Connections", "284", "text-[#60a5fa]"),
            metric("Clients", "48", "text-[#f59e0b]"),
            metric("Collaborators", "34", "text-[#22c55e]"),
            metric("Investors", "12", "text-[#a855f7]"),
            metric("Groups", "7", "text-[#cbd5f5]")
        ],
        "groups": [
            {"name": "All Contacts", "count": 1240},
            {"name": "Investors", "count": 12},
            {"name": "Partners", "count": 18},
            {"name": "Team", "count": 22}
        ],
        "contact_cards": [
            {"name": "Maya Thornton", "role": "Partner · Seed Capital", "org": "Apex Ventures"},
            {"name": "Kwame Asante", "role": "GP · Workforce Fund", "org": "Telos Capital"},
            {"name": "Layla Hassan", "role": "Business Dev", "org": "Stripe"},
            {"name": "Thomas Wei", "role": "Enterprise Sales", "org": "Salesforce"}
        ],
        "suggestion": "Amara shipped 3 commits to cooperative-os this week. Consider a check-in."
    })
}

pub fn sample_studio_overview_snapshot() -> OfficeSnapshot {
    json!({
        "summary_cards": [
            metric("Ideas", "42", "text-[#a855f7]"),
            metric("Concepts", "18", "text-[#60a5fa]"),
            metric("Designs", "11", "text-[#f59e0b]"),
            metric("Prototypes", "5", "text-[#22c55e]"),
        ],
        "focus_tracks": [
            {"title": "Workspace Refresh", "status": "In review"},
            {"title": "Design System v3", "status": "Draft"},
            {"title": "Prototype Lab", "status": "Testing"}
        ],
        "notes": [
            "Review blueprint dependencies for Q2 roadmap.",
            "Align documentation with portfolio artifacts.",
        ]
    })
}

pub fn sample_studio_detail_snapshot(section: &str) -> OfficeSnapshot {
    json!({
        "section": section,
        "items": [
            {"title": format!("{section} backlog A"), "status": "In review"},
            {"title": format!("{section} backlog B"), "status": "Draft"},
            {"title": format!("{section} backlog C"), "status": "Queued"}
        ],
        "cta": "Create new studio item"
    })
}

pub fn sample_work_backlog_snapshot() -> OfficeSnapshot {
    json!({
        "columns": [
            {"title": "Backlog", "count": 18},
            {"title": "In Progress", "count": 6},
            {"title": "Blocked", "count": 3},
            {"title": "Done", "count": 12}
        ],
        "items": [
            {"title": "Finalize WBS mapping", "status": "Backlog"},
            {"title": "Portfolio CRDT sync", "status": "In Progress"},
            {"title": "Board column review", "status": "Blocked"},
            {"title": "Sprint retrospective", "status": "Done"}
        ]
    })
}

pub fn sample_work_boards_snapshot() -> OfficeSnapshot {
    json!({
        "boards": [
            {"name": "Agile Board", "columns": ["Backlog", "In Progress", "Review", "Done"]},
            {"name": "Kanban Board", "columns": ["Intake", "Doing", "Blocked", "Complete"]},
            {"name": "Idea Board", "columns": ["Ideas", "Concepts", "Designs", "Prototypes"]}
        ]
    })
}

pub fn sample_work_timeline_snapshot() -> OfficeSnapshot {
    json!({
        "today": [
            {"time": "09:00 AM", "title": "Sprint Planning", "duration": "90m"},
            {"time": "11:30 AM", "title": "Client Review", "duration": "30m"},
            {"time": "02:00 PM", "title": "Deep Work Block", "duration": "2h"}
        ],
        "timeboxes": [
            {"title": "Sprint 14", "window": "Mar 10 - Mar 21"},
            {"title": "Governance Cycle", "window": "Mar 15 - Apr 1"}
        ],
        "reconciliation": [
            "Overlap Detected: Client Review vs Studio Sync",
            "Resource Conflict: Studio Room reserved"
        ]
    })
}

pub fn sample_work_analytics_snapshot() -> OfficeSnapshot {
    json!({
        "kpis": [
            {"label": "Velocity", "value": "28 pts", "note": "Last sprint"},
            {"label": "Cycle Time", "value": "3.4 days", "note": "Avg"}
        ],
        "okrs": [
            {"label": "Q2 Delivery", "value": "72%", "note": "On track"},
            {"label": "Docs Coverage", "value": "64%", "note": "Needs attention"}
        ],
        "performance": [
            {"label": "Health", "value": "84", "note": "Portfolio average"},
            {"label": "Risk", "value": "31", "note": "Low"}
        ],
        "telemetry": [
            {"label": "Active Users", "value": "18", "note": "Workspaces"},
            {"label": "Updates", "value": "142", "note": "Last 7d"}
        ]
    })
}

pub fn sample_work_resources_snapshot() -> OfficeSnapshot {
    json!({
        "budgeting": [
            {"label": "Delivery Sprint", "amount": "$12,400", "note": "Allocated"},
            {"label": "R&D", "amount": "$4,800", "note": "Remaining"}
        ],
        "allocation": [
            {"label": "Design", "amount": "42%", "note": "Studio"},
            {"label": "Engineering", "amount": "38%", "note": "Platform"}
        ],
        "todos": {
            "do_now": ["Resolve blocked CRDT task", "Confirm client review agenda"],
            "do_later": ["Update roadmap notes", "Archive old contracts"],
            "delegate": ["Follow up with vendors", "Prepare sprint retro"],
            "delete": ["Deprecated backlog items"]
        }
    })
}

pub fn sample_work_content_snapshot() -> OfficeSnapshot {
    json!({
        "files": ["proposal-v2.pdf", "design-tokens.json"],
        "documents": ["Project Charter", "Status Report"],
        "contracts": ["Client Master Services Agreement"],
        "agreements": ["Co-op Member Agreement"],
        "sops": ["Release SOP", "QA SOP"],
        "policies": ["Data Retention", "Security"],
        "procedures": ["Onboarding", "Incident Response"],
        "frameworks": ["Design System", "Delivery Framework"],
        "models": ["Risk Model", "Cost Model"]
    })
}

pub fn sample_work_governance_snapshot() -> OfficeSnapshot {
    json!({
        "proposals": [
            {"title": "Add moderator role", "status": "Voting", "due": "Fri"},
            {"title": "Allocate Q2 budget", "status": "Review", "due": "Mon"}
        ],
        "policies": [
            {"title": "Access Policy Update", "status": "Draft", "due": null}
        ],
        "votes_due": 2
    })
}
