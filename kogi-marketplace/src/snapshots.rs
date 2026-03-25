use serde_json::{json, Value};

pub type MarketplaceSnapshot = Value;

fn metric(label: &str, value: &str, meta: &str, tone: &str) -> Value {
    json!({
        "label": label,
        "value": value,
        "meta": meta,
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

fn tag(label: &str, tone: &str) -> Value {
    json!({
        "label": label,
        "tone": tone,
    })
}

fn kv(label: &str, value: &str, tone: &str) -> Value {
    json!({
        "label": label,
        "value": value,
        "tone": tone,
    })
}

pub fn sample_dashboard_snapshot() -> MarketplaceSnapshot {
    json!({
        "summary_cards": [
            metric("Active Listings", "142", "Marketplace Hub", "text-[#60a5fa]"),
            metric("Escrow Volume", "$84K", "Across 12 deals", "text-[#10b981]"),
            metric("Open Bids", "27", "Inbound interest", "text-[#f59e0b]"),
        ],
        "latest_drops": [
            status_item("Brand Identity System v2", "Open", "text-[#10b981]"),
            status_item("DAO Tooling Sprint", "Live", "text-[#60a5fa]"),
            status_item("Community Growth Bundle", "Soon", "text-[#f59e0b]"),
        ],
        "exchange_pulse": [
            kv("Avg Deal Time", "4.2 days", "text-[#10b981]"),
            kv("Active Escrows", "12", "text-[#60a5fa]"),
            kv("Buyer Match Rate", "78%", "text-[#f59e0b]"),
        ],
        "note": "Marketplace hub consolidates listings, deal flow, and exchange activity.",
    })
}

pub fn sample_market_overview_snapshot() -> MarketplaceSnapshot {
    json!({
        "summary_cards": [
            metric("Listed Items", "284", "Across market", "text-[#60a5fa]"),
            metric("Active Orders", "18", "Open deals", "text-[#22c55e]"),
            metric("Barter Offers", "12", "Swap requests", "text-[#f59e0b]"),
            metric("My Sales", "$8,400", "Last 30d", "text-[#10b981]"),
            metric("My Purchases", "$2,100", "Last 30d", "text-[#a855f7]"),
        ],
        "market_grid": [
            {"category": "Labor", "title": "Full-Stack Dev", "price": "$120/hr", "tone": "text-[#10b981]"},
            {"category": "Asset", "title": "Logo Design", "price": "$250", "tone": "text-[#a855f7]"},
            {"category": "Service", "title": "Brand Strategy", "price": "$3,500", "tone": "text-[#22c55e]"},
            {"category": "Artifact", "title": "React Template", "price": "$89", "tone": "text-[#f59e0b]"},
            {"category": "Resource", "title": "API Access", "price": "$29/mo", "tone": "text-[#60a5fa]"},
            {"category": "Barter", "title": "Photography Kit", "price": "Trade", "tone": "text-[#eab308]"},
        ],
        "barter_system": [
            {"offering": "Camera Gear", "want": "Laptop"},
            {"offering": "Adobe License", "want": "Web dev"},
            {"offering": "Studio Time", "want": "Design work"},
        ],
        "my_orders": [
            {"name": "Brand Package", "status": "Active", "value": "$3,500", "tone": "text-[#10b981]"},
            {"name": "Dev Hours", "status": "Delivered", "value": "$1,200", "tone": "text-[#60a5fa]"},
            {"name": "Legal Review", "status": "Pending", "value": "$200", "tone": "text-[#f59e0b]"},
        ],
        "linked_platforms": [
            tag("+ Behance", "text-[#10b981]"),
            tag("-> Upwork", "text-[#22c55e]"),
            tag("-> Fiverr", "text-[#f59e0b]"),
            tag("+ Etsy", "text-[#ec4899]"),
        ],
    })
}

pub fn sample_market_browse_snapshot() -> MarketplaceSnapshot {
    json!({
        "summary_cards": [
            metric("Active Listings", "8,420", "Marketplace total", "text-[#10b981]"),
            metric("New Today", "284", "Fresh drops", "text-[#e6f1f4]"),
            metric("MatchEngine Hits", "142", "AI matched", "text-[#f59e0b]"),
            metric("My Active Listings", "7", "Seller view", "text-[#22c55e]"),
            metric("My Open Orders", "28", "Buyer view", "text-[#60a5fa]"),
        ],
        "filters": {
            "categories": ["All", "Work", "Assets", "Services", "Templates", "Campaigns", "Resources"],
            "pricing_models": ["Any", "Fixed", "Hourly", "Subscription", "Negotiable", "Free", "Barter"],
            "price_range": {"min": "$0", "max": "$300"},
            "min_rating": "3+",
            "skills": ["Rust", "TypeScript", "React", "Design Systems", "DevOps", "GraphQL"],
            "availability": ["Available", "Any"],
            "sort_options": ["Best Match (AI)", "Newest", "Highest Rated", "Price: Low-High", "Price: High-Low", "Most Orders"],
            "verification": ["Verified Only", "Any"],
        },
        "segments": [
            {"label": "All", "count": 284},
            {"label": "Work", "count": 142},
            {"label": "Assets", "count": 48},
            {"label": "Services", "count": 38},
            {"label": "Templates", "count": 24},
            {"label": "Campaigns", "count": 18},
            {"label": "Resources", "count": 12},
        ],
        "ai_match_banner": {
            "title": "Oba matched 8 listings to your Developer + Systems Architect personas.",
            "cta": "View all 8 matches",
            "note": "Results ranked by MatchEngine compatibility."
        },
        "recommended_listings": [
            {
                "listing_type": "GIG - WORK",
                "match": "94%",
                "title": "Senior Rust Engineer - Cooperative Tech Platform",
                "org": "@coop.tech",
                "rating": "4.9",
                "summary": "6-month contract building distributed consensus for cooperative OS. Remote-first.",
                "tags": ["Rust", "CRDT", "Remote"],
                "price": "$180/hr",
                "pricing": "Hourly - 6 months",
                "cta": "Apply"
            },
            {
                "listing_type": "ARTIFACT",
                "match": "88%",
                "title": "Cooperative Design System Kit v3.0",
                "org": "@amara.dev",
                "rating": "5.0",
                "summary": "Design tokens + component library (React + Figma) with coop branding.",
                "tags": ["Figma", "React", "Tokens"],
                "price": "$240",
                "pricing": "Fixed - One-time",
                "cta": "Buy Now"
            },
            {
                "listing_type": "SERVICE",
                "match": "91%",
                "title": "Systems Architecture Review and Consulting",
                "org": "@jordan.dev",
                "rating": "4.8",
                "summary": "Deep-dive architecture review with written report + 2hr call.",
                "tags": ["Systems", "Consulting", "Rust"],
                "price": "$1,200",
                "pricing": "Fixed - Engagement",
                "cta": "Your Listing"
            },
            {
                "listing_type": "TEMPLATE",
                "match": "84%",
                "title": "High-Performance Rust Service Playbook",
                "org": "@marcus.systems",
                "rating": "4.7",
                "summary": "Production-tested patterns for high throughput Rust services.",
                "tags": ["Tokio", "Postgres"],
                "price": "$89",
                "pricing": "Fixed - Download",
                "cta": "Buy Now"
            },
            {
                "listing_type": "CAMPAIGN",
                "match": "78%",
                "title": "crdt-rs v3.0 - Community Funding Round",
                "org": "@jordan.dev",
                "rating": "Open Source",
                "summary": "Raising $24,000 to fund 6 months of full-time development.",
                "tags": ["Crowdfunding", "Open Source"],
                "price": "$24,000",
                "pricing": "Goal - 68% funded",
                "cta": "Back"
            },
            {
                "listing_type": "RESOURCE",
                "match": "71%",
                "title": "GPU Compute Credits - H100 Cluster",
                "org": "@tariq.infra",
                "rating": "4.9",
                "summary": "Selling excess GPU time for ML training and rendering.",
                "tags": ["GPU", "ML", "Compute"],
                "price": "$2.40/hr",
                "pricing": "Hourly - GPU compute",
                "cta": "Reserve"
            }
        ],
        "my_active_listings": [
            {"name": "UI Component Library - React Kit", "note": "Asset - $890 - 284 views"},
            {"name": "Brand Strategy Consulting", "note": "Work - $180/hr - 6 proposals"},
            {"name": "Full Stack Audit Playbook", "note": "Template - $220 - Paused"},
        ],
        "open_deals": [
            {"name": "Sofia R. -> Design Systems Contract", "note": "Milestone 2/4 - $12,000 escrow"},
            {"name": "Kai M. -> Brand Identity System", "note": "Proposal sent - $2,400"},
            {"name": "Theo P. -> React Component Kit", "note": "Completed - $890 settled"},
        ],
    })
}

pub fn sample_market_labor_snapshot() -> MarketplaceSnapshot {
    json!({
        "market_tabs": [
            {"label": "Labor Market - KLM", "active": true},
            {"label": "Commodities - KPCM", "active": false},
            {"label": "Financial Instruments - KFIM", "active": false},
            {"label": "Resource Market - KRM", "active": false},
        ],
        "summary_cards": [
            metric("Open Offers", "2,420", "Selling orders", "text-[#10b981]"),
            metric("Open RFPs", "284", "Buyer requests", "text-[#f59e0b]"),
            metric("Live Bids", "48", "Active bidding", "text-[#a855f7]"),
            metric("Rust Avg Rate", "$180/hr", "Benchmark", "text-[#60a5fa]"),
            metric("My Active Orders", "18", "Seller + buyer", "text-[#22c55e]"),
            metric("Your Match Score", "94%", "AI match", "text-[#e6f1f4]"),
            metric("Volume (30d)", "$2.4M", "Market size", "text-[#8ea6ad]"),
        ],
        "top_matches": [
            {
                "type": "RFP - Contract",
                "match": "94%",
                "title": "Senior Rust / Distributed Systems Engineer - 6-month contract",
                "org": "Cooperative Tech Inc",
                "details": "Remote - 8 proposals",
                "tags": ["Rust", "CRDT", "gRPC"],
                "price": "$180/hr",
                "actions": ["Propose", "Save"]
            },
            {
                "type": "Offer - Gig",
                "match": "88%",
                "title": "DeFi Protocol Architecture Review - Fixed price",
                "org": "Protocol Inc",
                "details": "Remote - 3 proposals",
                "tags": ["Architecture", "DeFi"],
                "price": "$45,000",
                "actions": ["Propose", "Save"]
            },
            {
                "type": "Reverse Auction - Job",
                "match": "82%",
                "title": "Full-Stack TypeScript / Next.js Developer - Equity role",
                "org": "Open Cooperative Studio",
                "details": "12 bids",
                "tags": ["Reverse Auction"],
                "price": "<= $12K/mo",
                "actions": ["Place Bid", "Save"]
            },
            {
                "type": "Collaboration - Rev Share",
                "match": "79%",
                "title": "Co-founder / Technical Lead - AI tooling startup",
                "org": "Nadia Wolff",
                "details": "Remote - 2 applications",
                "tags": ["Rev Share", "Equity"],
                "price": "Rev Share + Equity",
                "actions": ["Express Interest"]
            }
        ],
        "order_book": {
            "asks": [
                {"skill": "Rust - Systems", "workers": "3", "rate": "$220/hr"},
                {"skill": "TypeScript - FE", "workers": "18", "rate": "$140/hr"},
                {"skill": "Rust - CRDT (You)", "workers": "1", "rate": "$180/hr"},
            ],
            "bids": [
                {"skill": "Rust - Distributed", "buyers": "8", "rate": "$180/hr"},
                {"skill": "Systems Design", "buyers": "5", "rate": "$160/hr"},
                {"skill": "Design Systems", "buyers": "9", "rate": "$100/hr"},
            ],
            "position": "At-market - rate matches top bid price."
        },
        "active_positions": {
            "selling": {
                "title": "Systems Architecture Consulting",
                "note": "Retainer - Async-friendly",
                "price": "$3,200/mo"
            },
            "buying": {
                "title": "Video Editing / Motion Designer",
                "note": "Mesh Studio - 6 proposals",
                "price": "Budget $2,400"
            },
            "matched": {
                "title": "Rust Systems - Coop Tech Inc",
                "note": "Month 3 of 6 - $43,200 escrow",
                "actions": ["Deal Room", "Escrow"]
            },
            "new_matches": [
                "94% match - Cooperative Tech Inc - 6 months - $180/hr",
                "88% match - Protocol Inc - Fixed price - $45,000"
            ]
        },
        "worker_stats": [
            metric("Workers Listed", "284", "KLM supply", "text-[#10b981]"),
            metric("Open Roles", "408", "Demand", "text-[#f59e0b]"),
            metric("Avg Rate", "$94/hr", "Blended", "text-[#a855f7]"),
            metric("Active Deals", "28", "In progress", "text-[#22c55e]"),
        ],
        "workers": [
            {
                "name": "Sofia Reyes",
                "role": "Senior Product Designer - Design Systems",
                "skills": ["Figma", "Tokens", "Motion"],
                "rate": "$145/hr",
                "rating": "5.0",
                "availability": "Available Now",
                "ai_match": "97"
            },
            {
                "name": "Kai Thompson",
                "role": "Full-Stack Engineer - Platform Architecture",
                "skills": ["Go", "Rust", "Kafka"],
                "rate": "$170/hr",
                "rating": "5.0",
                "availability": "Available Feb 1",
                "ai_match": "94"
            }
        ],
        "requirements": [
            {"title": "Frontend Engineer - React + TypeScript", "note": "$120-160/hr - 8 proposals"},
            {"title": "Data Pipeline Architect", "note": "$180-220/hr - 3 proposals"}
        ],
        "hot_skills": ["Design Systems", "Go / Rust", "ML / AI", "Figma", "TypeScript", "Legal / IP"],
        "market_rates": [
            {"skill": "ML / AI", "rate": "$194"},
            {"skill": "Architecture", "rate": "$178"},
            {"skill": "Legal / IP", "rate": "$172"}
        ],
        "oba_note": "Processing 284 workers against your requirements. 21 high-confidence matches surfaced."
    })
}

pub fn sample_market_grants_snapshot() -> MarketplaceSnapshot {
    json!({
        "summary_cards": [
            metric("Grant Listings", "248", "Active grants", "text-[#22c55e]"),
            metric("Crowdresourcing", "42", "Open calls", "text-[#a855f7]"),
            metric("Portfolio Exchange", "18", "Resources", "text-[#10b981]"),
            metric("Shared Portfolios", "7", "Co-managed", "text-[#f59e0b]"),
        ],
        "grant_filters": ["Government", "Foundation", "Community", "Platform", "All"],
        "grant_table": [
            {"grant": "SBA SBIR Phase I", "amount": "$150K", "type": "Government", "deadline": "Apr 15", "match": "94%"},
            {"grant": "Knight Foundation", "amount": "$50K", "type": "Foundation", "deadline": "Mar 30", "match": "88%"},
            {"grant": "Dev DAO Builder", "amount": "$5K", "type": "Community", "deadline": "Rolling", "match": "97%"},
            {"grant": "NYSCA Digital Arts", "amount": "$12K", "type": "Government", "deadline": "Apr 1", "match": "76%"},
            {"grant": "Mozilla FOSS Grant", "amount": "$30K", "type": "Foundation", "deadline": "May 1", "match": "71%"},
        ],
        "crowdresourcing": [
            {"category": "Labor - Code - Docs", "title": "API Gateway v2 - Community Dev", "reward": "Attribution + revenue share", "contributors": 14, "progress": "72%"},
            {"category": "Design - Assets", "title": "Kogi Design System - Open Contributions", "reward": "Equity allocation", "contributors": 8, "progress": "48%"},
            {"category": "Knowledge - Research", "title": "Tokenomics Research - Community Input", "reward": "Direct payment", "contributors": "Invite only", "progress": "30%"},
        ],
        "portfolio_exchange": [
            {"type": "Template - Fixed", "title": "Client Onboarding Playbook", "price": "$89", "note": "Fork rights - Bank instant"},
            {"type": "IP License", "title": "API Spec v2 License", "price": "$2,400/yr", "note": "Escrow-backed - DD required"},
            {"type": "Equity Stake", "title": "Kogi Platform 2.4%", "price": "$24K", "note": "Gov approval + cap table update"},
            {"type": "Revenue Stream", "title": "Design Retainer Rev-Share", "price": "$18K", "note": "12mo forward - MatchEngine"},
        ]
    })
}

pub fn sample_market_crm_snapshot() -> MarketplaceSnapshot {
    json!({
        "segments": ["Buy - Sell - Barter", "Grants + Microfinancing", "Crowdresourcing", "CRM + Booking", "Logistics"],
        "summary_cards": [
            metric("Pipeline Value", "$84K", "Active pipeline", "text-[#10b981]"),
            metric("Bookings This Month", "7", "Confirmed", "text-[#22c55e]"),
            metric("Conversion Rate", "42%", "Qualified to booked", "text-[#a855f7]"),
            metric("Client LTV Avg", "$12.4K", "Rolling 12m", "text-[#f59e0b]"),
        ],
        "pipeline": [
            {
                "stage": "Inquiry",
                "count": 5,
                "items": [
                    {"name": "TechCorp Inc - Design system audit", "value": "$8K", "score": "72"},
                    {"name": "StartupX - Full-stack dev sprint", "value": "$15K", "score": "65"}
                ]
            },
            {
                "stage": "Qualified",
                "count": 4,
                "items": [
                    {"name": "DevDAO - API integration sprint", "value": "$28K", "score": "91"},
                    {"name": "Acme Corp - Brand redesign", "value": "$18K", "score": "88"}
                ]
            },
            {
                "stage": "Proposal",
                "count": 3,
                "items": [
                    {"name": "Invest Club - Strategy retainer", "value": "$3.5K/mo", "note": "Sent 2d ago"},
                    {"name": "3rd Party Co - UI library", "value": "$12K", "note": "Expires 3d"}
                ]
            },
            {
                "stage": "Negotiating",
                "count": 2,
                "items": [
                    {"name": "MediaCo - Content strategy", "value": "$6K", "note": "Counter sent"}
                ]
            },
            {
                "stage": "Booked",
                "count": 7,
                "items": [
                    {"name": "DevDAO Sprint - Engineering", "value": "$48K", "note": "Starts Mar 18"},
                    {"name": "Acme - Design + Dev", "value": "$28K", "note": "In progress"}
                ]
            }
        ],
        "booking_calendar": [
            {"title": "DevDAO Sprint Kickoff", "time": "Mar 18 - 10:00 AM - Remote", "status": "Confirmed"},
            {"title": "Acme Brand Review", "time": "Mar 20 - 2:00 PM - NYC Office", "status": "Confirmed"},
            {"title": "Strategy Consult - Invest Club", "time": "Mar 25 - 11:00 AM - Video", "status": "Deposit pending"},
        ],
        "logistics_tracker": {
            "title": "Active Booking - DevDAO Sprint",
            "items": [
                {"label": "Equipment", "status": "Ready"},
                {"label": "Crew assigned", "status": "3 confirmed"},
                {"label": "Contract", "status": "Signed"},
                {"label": "Deposit", "status": "Received $9.6K"}
            ]
        }
    })
}

pub fn sample_listings_catalog_snapshot() -> MarketplaceSnapshot {
    json!({
        "summary_cards": [
            metric("Active Listings", "284", "Marketplace", "text-[#10b981]"),
            metric("My Listings", "18", "Seller view", "text-[#e6f1f4]"),
            metric("Pending Orders", "12", "Needs action", "text-[#f59e0b]"),
            metric("Monthly Volume", "$8,400", "30 days", "text-[#22c55e]"),
            metric("My Avg Rating", "4.8", "Recent reviews", "text-[#60a5fa]"),
            metric("MatchEngine", "94%", "AI fit", "text-[#a855f7]"),
        ],
        "filters": {
            "categories": [
                {"label": "All Categories", "count": 284},
                {"label": "Work / Gigs", "count": 142},
                {"label": "Assets", "count": 68},
                {"label": "Templates", "count": 44},
                {"label": "Resources", "count": 18},
                {"label": "Campaigns", "count": 12},
            ],
            "skills": [
                {"label": "Rust", "count": 28},
                {"label": "TypeScript", "count": 44},
                {"label": "Design Systems", "count": 22},
                {"label": "Distributed", "count": 8},
            ],
            "pricing_models": [
                {"label": "All Models", "count": 284},
                {"label": "Fixed Price", "count": 120},
                {"label": "Hourly", "count": 88},
                {"label": "Subscription", "count": 32},
            ],
            "price_range": {"min": "$50", "max": "$300"},
            "rating": ["5.0 only", "4.0 and up", "3.0 and up"],
        },
        "listing_results": [
            {
                "title": "Senior Rust Systems Engineer - 6-month contract",
                "org": "Cooperative Tech Inc",
                "posted": "Posted 2h ago",
                "summary": "Build CRDT-based sync infrastructure. Tokio async, PostgreSQL, gRPC.",
                "tags": ["Rust", "Tokio", "CRDT"],
                "price": "$180/hr",
                "pricing": "Hourly - 6 months",
                "match": "94%",
                "status": "Accepting proposals"
            },
            {
                "title": "Cooperative Design System Kit v3 - Figma + React",
                "org": "Amara Kofi",
                "posted": "Digital download",
                "summary": "Complete design system for cooperatives and collectives.",
                "tags": ["Figma", "React", "Tokens"],
                "price": "$249",
                "pricing": "Fixed - One-time",
                "status": "Instant delivery"
            },
        ],
        "listing_detail": {
            "title": "Senior Rust Systems Engineer",
            "org": "Cooperative Tech Inc",
            "price": "$180/hr",
            "pricing": "Hourly - 6-month contract",
            "type": "Gig - Contract",
            "duration": "6 months",
            "escrow": "Milestone escrow",
            "proposals": "8 received",
            "match": "94% MatchEngine fit"
        }
    })
}

pub fn sample_listings_detail_snapshot() -> MarketplaceSnapshot {
    json!({
        "breadcrumbs": "Marketplace / Assets / Brand Identity System v2.0",
        "status_tags": [
            tag("Asset - Playbook", "text-[#a855f7]"),
            tag("Active", "text-[#22c55e]"),
            tag("AI Score 94", "text-[#10b981]")
        ],
        "title": "Brand Identity System v2.0 - Complete Toolkit",
        "subtitle": "Production-ready brand identity package with 480+ deliverables.",
        "seller": {
            "name": "Jordan Chen",
            "rating": "4.8",
            "reviews": 142,
            "role": "Designer",
            "badge": "Top Seller",
            "response_time": "Responds < 2 hrs"
        },
        "description": "A production-ready brand identity system built over 18 months. Includes logo system, color tokens, typography scale, component library, and usage documentation.",
        "included": [
            "Logo system (24 variants)",
            "Color token system (light + dark)",
            "Typography scale + font files",
            "Component library (160+ components)",
            "Icon set (480+ icons)",
            "Design token export (JSON/CSS)"
        ],
        "tags": ["Branding", "Figma", "Design Systems", "Tokens", "React"],
        "portfolio_source": {
            "title": "Brand Identity System - Portfolio Asset",
            "version": "v2.0",
            "updated": "Jan 2026",
            "health_score": "96"
        },
        "reviews": [
            {"name": "Theo Park", "rating": "5.0", "note": "Exceptional quality. The token system saved us weeks of work."},
            {"name": "Maya D.", "rating": "5.0", "note": "Used this for our Series A pitch deck and rebuild. Great system."}
        ],
        "pricing": {
            "price": "$2,400",
            "model": "Fixed - One-Time License",
            "license_type": "Single entity",
            "delivery": "Instant download",
            "revisions": "2 included",
            "escrow": "kogi-bank",
            "total": "$2,400"
        },
        "analytics": [
            {"label": "Views", "value": "1,249"},
            {"label": "Saves", "value": "87"},
            {"label": "Proposals", "value": "24"},
            {"label": "Sales", "value": "18"}
        ],
        "ai_analysis": {
            "note": "High-quality listing with strong buyer signals and competitive pricing.",
            "scores": [
                {"label": "Match", "value": "94", "tone": "text-[#10b981]"},
                {"label": "Quality", "value": "97", "tone": "text-[#a855f7]"},
                {"label": "Value", "value": "88", "tone": "text-[#f59e0b]"}
            ]
        },
        "similar_listings": [
            {"title": "UI Component Library - React Pro", "note": "$890 - Asset - AI 89"},
            {"title": "Motion Design System Starter", "note": "$480 - Asset - AI 82"}
        ]
    })
}

pub fn sample_listings_mine_snapshot() -> MarketplaceSnapshot {
    json!({
        "summary_cards": [
            metric("Active Listings", "12", "Seller view", "text-[#f59e0b]"),
            metric("Revenue (30d)", "$8,400", "Recent sales", "text-[#10b981]"),
            metric("Open Proposals", "7", "Needs response", "text-[#a855f7]"),
            metric("Reputation", "94", "AI score", "text-[#22c55e]"),
        ],
        "revenue_series": [30, 45, 40, 60, 55, 72, 88, 100],
        "views_series": [50, 60, 45, 70, 65, 80, 90, 100],
        "listings_table": [
            {
                "listing": "Brand Identity System v2.0",
                "type": "Asset - Playbook",
                "status": "Active",
                "price": "$2,400",
                "views": "1,249",
                "proposals": "24",
                "sales": "18",
                "revenue": "$43,200"
            },
            {
                "listing": "UI Component Library - React Kit",
                "type": "Asset - Library",
                "status": "Active",
                "price": "$890",
                "views": "842",
                "proposals": "18",
                "sales": "34",
                "revenue": "$30,260"
            },
            {
                "listing": "Full Stack Audit Playbook",
                "type": "Template - Playbook",
                "status": "Paused",
                "price": "$220",
                "views": "142",
                "proposals": "3",
                "sales": "7",
                "revenue": "$1,540"
            }
        ],
        "ai_reputation": {
            "score": "94",
            "breakdown": [
                {"label": "Completion", "value": "97"},
                {"label": "Review Score", "value": "96"},
                {"label": "Quality", "value": "94"}
            ]
        },
        "recent_activity": [
            {"event": "New proposal: React Kit", "note": "Remi O. - $890 - 2m ago"},
            {"event": "Sale: Brand Identity v2.0", "note": "Jordan K. - $2,400 - 1h ago"},
            {"event": "New review: 5.0", "note": "Maya D. - 3h ago"}
        ],
        "oba_recommendations": [
            "Re-activate Audit Playbook - 3 pending saves",
            "Raise React Kit price to $1,100 (market comps)",
            "Publish Motion Design draft - high demand",
            "Respond to 2 new proposals today"
        ]
    })
}

pub fn sample_campaigns_overview_snapshot() -> MarketplaceSnapshot {
    json!({
        "campaign_types": ["All", "Marketing", "Funding", "Sales", "Barter", "Advocacy", "Referral"],
        "summary_cards": [
            metric("Active Campaigns", "7", "Across marketplace", "text-[#10b981]"),
            metric("Total Raised", "$328,500", "All campaigns", "text-[#22c55e]"),
            metric("Leads Generated", "4,820", "Inbound", "text-[#60a5fa]"),
            metric("Pipeline Value", "$84,200", "Sales funnel", "text-[#f59e0b]"),
            metric("Avg Conversion", "12.4%", "Rolling 30d", "text-[#e6f1f4]"),
            metric("Backers", "284", "Active backers", "text-[#a855f7]"),
        ],
        "funding_campaign": {
            "title": "Kogi Engine - Series Seed",
            "type": "Equity crowdfunding",
            "goal": "$450K",
            "raised": "$328,500",
            "percent": "73%",
            "days_left": "28",
            "backers": "42",
            "avg_invest": "$7,821",
            "last_7d": "+$48K",
            "traffic_sources": [
                {"source": "Profile / Network", "value": "$192K"},
                {"source": "Social Threads", "value": "$80K"},
                {"source": "Direct DMs", "value": "$56K"}
            ]
        },
        "marketing_campaign": {
            "title": "crdt-rs Launch - Awareness",
            "impressions": "28,400",
            "stars": "1,284",
            "followers": "342",
            "channels": [
                {"channel": "Launch Thread", "value": "14.2K"},
                {"channel": "Show HN", "value": "8.4K"},
                {"channel": "Substack", "value": "4.2K"}
            ]
        },
        "sales_campaign": {
            "title": "Q2 Consulting Client Acquisition",
            "funnel": "84 leads -> 14 qualified -> 4 proposals -> 2 signed",
            "leads": "84",
            "signed": "2/5",
            "mrr_added": "$6.4K"
        },
        "oba_intel": [
            "Seed round is at 73% with 28 days left. At $6,900/day velocity you hit the goal in 17.5 days.",
            "Engagement on the launch thread is dropping. Post a benchmark follow-up today.",
            "Two proposals have been open for 8+ days. Drafted follow-ups are ready."
        ],
        "sales_pipeline": [
            {"contact": "Nadia Wolff", "stage": "Signed", "value": "$3.2K", "win": "100%"},
            {"contact": "Protocol Inc", "stage": "Proposal", "value": "$3.2K", "win": "70%"},
            {"contact": "Pamoja Federation", "stage": "Proposal", "value": "$8.0K", "win": "55%"},
            {"contact": "Solo Dev Collective", "stage": "Lead", "value": "$3.2K", "win": "20%"}
        ],
        "outreach_queue": [
            {"name": "Tariq Nasir", "note": "Viewed 3x - $10K capacity"},
            {"name": "Priya Bose", "note": "Research collaborator - angel profile"},
            {"name": "Aisha Patel", "note": "Existing client - high trust"}
        ],
        "referral_program": {
            "per_referral": "$200",
            "conversions": "12"
        }
    })
}

pub fn sample_campaigns_discover_snapshot() -> MarketplaceSnapshot {
    json!({
        "summary_cards": [
            metric("Active Campaigns", "284", "Marketplace", "text-[#10b981]"),
            metric("Total Raised", "$4.2M", "All campaigns", "text-[#22c55e]"),
            metric("Backers Active", "1,248", "Across network", "text-[#a855f7]"),
            metric("Equity Rounds", "42", "Live", "text-[#f59e0b]"),
            metric("Resource Pools", "88", "Open", "text-[#60a5fa]"),
            metric("Fundraises", "124", "Active", "text-[#e6f1f4]"),
        ],
        "filters": {
            "types": ["All", "Equity", "Revenue", "Fundraise", "Resource", "Mutual Aid"],
            "entities": ["All Entities", "Individual", "Team", "Collective", "Cooperative", "Federation"],
            "sort": ["Sort: AI Match", "Newest", "Trending", "Deadline Soon", "Largest"]
        },
        "recommended": [
            {
                "type": "Community Bond",
                "title": "Cooperative Tech Infrastructure Bond",
                "org": "Pamoja Federation",
                "progress": "62%",
                "raised": "$124K",
                "days_left": "45"
            },
            {
                "type": "Revenue Share",
                "title": "OS Collective Tooling Fund",
                "org": "Open Source Collective",
                "progress": "84%",
                "raised": "$42K",
                "days_left": "18"
            },
            {
                "type": "Resource Pool",
                "title": "Builder Network GPU Pool",
                "org": "Kogi Builder Network",
                "progress": "58%",
                "raised": "$14.5K",
                "days_left": "35"
            }
        ],
        "equity_trending": [
            {"title": "Kogi Engine - Series Seed", "raised": "$328.5K", "progress": "73%", "days_left": "28"},
            {"title": "Mesh Studio Design System", "raised": "$27K", "progress": "45%", "days_left": "56"},
            {"title": "Worker OS Cooperative Platform", "raised": "$280K", "progress": "28%", "days_left": "90"},
        ],
        "mutual_aid": [
            {"title": "Freelancers Emergency Fund", "raised": "$9.5K", "progress": "95%", "days_left": "8"},
            {"title": "Worker Rights Legal Defense", "raised": "$21K", "progress": "42%", "days_left": "60"},
            {"title": "Cooperative OS Bootcamp", "raised": "128 pre-orders", "progress": "64%", "days_left": "45"},
        ]
    })
}
