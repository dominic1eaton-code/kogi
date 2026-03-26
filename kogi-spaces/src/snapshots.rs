use serde_json::{json, Value};

pub type SpacesSnapshot = Value;

fn metric(label: &str, value: &str, meta: &str, tone: &str) -> Value {
    json!({
        "label": label,
        "value": value,
        "meta": meta,
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

pub fn sample_dashboard_snapshot() -> SpacesSnapshot {
    json!({
        "summary_cards": [
            metric("Active Spaces", "24", "Community + Team", "text-[#60a5fa]"),
            metric("Rooms Live", "18", "Voice, video, chat", "text-[#10b981]"),
            metric("Members Online", "142", "Across all spaces", "text-[#f59e0b]"),
            metric("Events This Week", "7", "Meetups + workshops", "text-[#a855f7]")
        ],
        "active_spaces": [
            status_item("Kogi Builders Guild", "Active", "text-[#10b981]"),
            status_item("Design Systems Collective", "Active", "text-[#60a5fa]"),
            status_item("Freelancers Union", "Trending", "text-[#f59e0b]")
        ],
        "space_health": [
            {"name": "Kogi Builders Guild", "score": "94", "note": "High engagement"},
            {"name": "Design Systems Collective", "score": "88", "note": "Stable growth"},
            {"name": "Freelancers Union", "score": "76", "note": "New moderators needed"}
        ],
        "governance_queue": [
            {"proposal": "Add new moderator role", "space": "Builders Guild", "status": "Voting"},
            {"proposal": "Budget allocation for events", "space": "Freelancers Union", "status": "Review"},
            {"proposal": "Workspace template: Hiring Sprint", "space": "Design Collective", "status": "Draft"}
        ],
        "events": [
            {"title": "Open Office Hours", "time": "Thu 2:00 PM", "space": "Builders Guild"},
            {"title": "Design Systems Sync", "time": "Fri 11:00 AM", "space": "Design Collective"},
            {"title": "Community Demo Day", "time": "Sat 4:00 PM", "space": "Freelancers Union"}
        ],
        "oba_note": "Spaces sync portfolio items, rooms, and governance into one operational dashboard."
    })
}

pub fn sample_feed_snapshot() -> SpacesSnapshot {
    json!({
        "filters": ["All Spaces", "Following", "Announcements", "Mentions", "Jobs", "Events"],
        "pinned": [
            {"title": "Welcome to the Builders Guild", "space": "Builders Guild", "tone": "text-[#10b981]"},
            {"title": "Q2 Roadmap Voting Opens", "space": "Freelancers Union", "tone": "text-[#f59e0b]"}
        ],
        "posts": [
            {
                "author": "Ari P.",
                "space": "Design Systems Collective",
                "type": "Announcement",
                "summary": "New workspace template: Client Onboarding",
                "reactions": 24,
                "comments": 8,
                "time": "12m"
            },
            {
                "author": "Maya R.",
                "space": "Builders Guild",
                "type": "Update",
                "summary": "LinkForest audit complete. 3 broken cross-space links fixed.",
                "reactions": 18,
                "comments": 4,
                "time": "42m"
            },
            {
                "author": "Kofi T.",
                "space": "Freelancers Union",
                "type": "Event",
                "summary": "Hosting a tax prep AMA on Friday. RSVP in Events.",
                "reactions": 52,
                "comments": 12,
                "time": "2h"
            }
        ],
        "suggested_spaces": [
            {"name": "Kogi Dev DAO", "reason": "Matches your Rust + Systems skills", "cta": "Join"},
            {"name": "Coop Finance Hub", "reason": "Active grant discussions", "cta": "Preview"},
            {"name": "Design Ops Guild", "reason": "Shared workspace templates", "cta": "Follow"}
        ]
    })
}

pub fn sample_timeline_snapshot() -> SpacesSnapshot {
    json!({
        "timeline": [
            {"time": "09:00 AM", "event": "Workspace sync: Builders Guild", "status": "Completed"},
            {"time": "10:30 AM", "event": "Room live: Design Critique", "status": "In progress"},
            {"time": "12:00 PM", "event": "Governance vote closes", "status": "Due"},
            {"time": "02:00 PM", "event": "Community demo rehearsal", "status": "Upcoming"},
            {"time": "04:30 PM", "event": "Space link audit", "status": "Upcoming"}
        ],
        "milestones": [
            {"title": "Space charter signed", "space": "Builders Guild", "date": "Mar 20"},
            {"title": "Workspace template launched", "space": "Design Collective", "date": "Mar 22"},
            {"title": "1000th member joined", "space": "Freelancers Union", "date": "Mar 24"}
        ],
        "signals": [
            {"label": "Emoji sentiment", "value": "84", "note": "Positive trend"},
            {"label": "Response time", "value": "22m", "note": "Improved"},
            {"label": "New members", "value": "+18", "note": "Last 7d"}
        ]
    })
}

pub fn sample_rooms_snapshot() -> SpacesSnapshot {
    json!({
        "room_categories": ["Voice", "Video", "Async", "Live Events", "Office Hours"],
        "active_rooms": [
            {"name": "Design Critique", "space": "Design Collective", "members": 12, "status": "Live"},
            {"name": "Rust Systems AMA", "space": "Builders Guild", "members": 28, "status": "Live"},
            {"name": "Coop Finance Q&A", "space": "Freelancers Union", "members": 8, "status": "Starting soon"}
        ],
        "room_queue": [
            {"name": "Hiring Roundtable", "space": "Builders Guild", "time": "Today 3:00 PM"},
            {"name": "Mentor Hours", "space": "Design Collective", "time": "Tomorrow 10:00 AM"}
        ],
        "quick_actions": [
            tag("Create Room", "text-[#10b981]"),
            tag("Schedule Event", "text-[#60a5fa]"),
            tag("Launch Broadcast", "text-[#f59e0b]")
        ]
    })
}

pub fn sample_channels_snapshot() -> SpacesSnapshot {
    json!({
        "channels": [
            {"name": "#announcements", "space": "Builders Guild", "type": "Broadcast", "unread": 3},
            {"name": "#design-systems", "space": "Design Collective", "type": "Chat", "unread": 12},
            {"name": "#grant-watch", "space": "Freelancers Union", "type": "Feed", "unread": 6},
            {"name": "#ops", "space": "Builders Guild", "type": "Timeline", "unread": 2}
        ],
        "message_stats": [
            metric("Messages Today", "412", "All channels", "text-[#10b981]"),
            metric("Active Threads", "38", "Ongoing", "text-[#60a5fa]"),
            metric("Mentions", "14", "Needs reply", "text-[#f59e0b]")
        ],
        "integrations": [
            tag("Slack Bridge", "text-[#10b981]"),
            tag("Discord Sync", "text-[#a855f7]"),
            tag("Matrix Relay", "text-[#60a5fa]")
        ]
    })
}

pub fn sample_events_snapshot() -> SpacesSnapshot {
    json!({
        "calendar": [
            {"date": "Mar 26", "count": 2},
            {"date": "Mar 27", "count": 1},
            {"date": "Mar 28", "count": 3},
            {"date": "Mar 29", "count": 1}
        ],
        "upcoming_events": [
            {"title": "Community Demo Day", "space": "Builders Guild", "time": "Mar 27 - 4:00 PM", "rsvp": 42},
            {"title": "Design Systems Sync", "space": "Design Collective", "time": "Mar 28 - 11:00 AM", "rsvp": 18},
            {"title": "Taxes + Benefits Clinic", "space": "Freelancers Union", "time": "Mar 29 - 1:00 PM", "rsvp": 54}
        ],
        "event_brief": {
            "title": "Community Demo Day",
            "format": "Hybrid",
            "agenda": ["Showcase", "Q&A", "Matchmaking"],
            "hosts": ["@kogi.ops", "@builders.guild"]
        }
    })
}

pub fn sample_network_overview_snapshot() -> SpacesSnapshot {
    json!({
        "summary_cards": [
            metric("Linked Spaces", "64", "Cross-space graph", "text-[#10b981]"),
            metric("Shared Workspaces", "18", "Active", "text-[#60a5fa]"),
            metric("External Hubs", "12", "Vendors + partners", "text-[#f59e0b]")
        ],
        "linknet": [
            {"source": "Builders Guild", "target": "Design Collective", "type": "Shared Workspace"},
            {"source": "Freelancers Union", "target": "Coop Finance Hub", "type": "Resource Exchange"},
            {"source": "Design Collective", "target": "Dev DAO", "type": "Mentor Channel"}
        ],
        "recommended_links": [
            {"title": "Connect to Kogi Community Hub", "note": "High overlap in members"},
            {"title": "Link to GrantWatch Space", "note": "Matches funding topics"}
        ],
        "oba_note": "LinkNet + LinkTree + LinkForest unify contacts, resources, and cross-space references."
    })
}

pub fn sample_network_linknet_snapshot() -> SpacesSnapshot {
    json!({
        "nodes": [
            {"id": "space-001", "label": "Builders Guild", "tone": "text-[#10b981]"},
            {"id": "space-002", "label": "Design Collective", "tone": "text-[#60a5fa]"},
            {"id": "space-003", "label": "Freelancers Union", "tone": "text-[#f59e0b]"}
        ],
        "edges": [
            {"from": "space-001", "to": "space-002", "type": "workspace"},
            {"from": "space-001", "to": "space-003", "type": "event"},
            {"from": "space-002", "to": "space-003", "type": "channel"}
        ],
        "segments": ["Workspace Links", "Event Links", "Resource Links", "Member Overlap"],
        "note": "LinkNet is a live, typed graph of space relationships."
    })
}

pub fn sample_network_linktree_snapshot() -> SpacesSnapshot {
    json!({
        "tree": {
            "root": "Kogi Community",
            "branches": [
                {"label": "Builders Guild", "children": ["Rust Circle", "Infra Lab"]},
                {"label": "Design Collective", "children": ["Research Lab", "Motion Studio"]},
                {"label": "Freelancers Union", "children": ["Tax Clinic", "Benefits Hub"]}
            ]
        },
        "links": [
            {"label": "Join Builders Guild", "url": "kogi://spaces/builders"},
            {"label": "Visit Design Collective", "url": "kogi://spaces/design"},
            {"label": "Freelancers Union", "url": "kogi://spaces/freelancers"}
        ],
        "cta": "Publish your LinkTree to the community directory."
    })
}

pub fn sample_network_linkforest_snapshot() -> SpacesSnapshot {
    json!({
        "clusters": [
            {"name": "Engineering", "spaces": 12, "density": "High"},
            {"name": "Design", "spaces": 8, "density": "Medium"},
            {"name": "Operations", "spaces": 6, "density": "Medium"},
            {"name": "Finance", "spaces": 4, "density": "Low"}
        ],
        "forest_map": [
            {"space": "Builders Guild", "links": 14, "shared_members": 84},
            {"space": "Design Collective", "links": 9, "shared_members": 56},
            {"space": "Freelancers Union", "links": 7, "shared_members": 48}
        ],
        "note": "LinkForest materializes cross-space relationships into a navigable map."
    })
}
