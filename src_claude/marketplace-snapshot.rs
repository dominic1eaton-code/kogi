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
            tag(" Behance", "text-[#10b981]"),
            tag("-> Upwork", "text-[#22c55e]"),
            tag("-> Fiverr", "text-[#f59e0b]"),
            tag(" Etsy", "text-[#ec4899]"),
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
            "min_rating": "3",
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
            "title": "Oba matched 8 listings to your Developer  Systems Architect personas.",
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
                "summary": "Design tokens  component library (React  Figma) with coop branding.",
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
                "summary": "Deep-dive architecture review with written report  2hr call.",
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
            metric("My Active Orders", "18", "Seller  buyer", "text-[#22c55e]"),
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
                "price": "Rev Share  Equity",
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
            {"title": "Frontend Engineer - React  TypeScript", "note": "$120-160/hr - 8 proposals"},
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
            {"category": "Labor - Code - Docs", "title": "API Gateway v2 - Community Dev", "reward": "Attribution  revenue share", "contributors": 14, "progress": "72%"},
            {"category": "Design - Assets", "title": "Kogi Design System - Open Contributions", "reward": "Equity allocation", "contributors": 8, "progress": "48%"},
            {"category": "Knowledge - Research", "title": "Tokenomics Research - Community Input", "reward": "Direct payment", "contributors": "Invite only", "progress": "30%"},
        ],
        "portfolio_exchange": [
            {"type": "Template - Fixed", "title": "Client Onboarding Playbook", "price": "$89", "note": "Fork rights - Bank instant"},
            {"type": "IP License", "title": "API Spec v2 License", "price": "$2,400/yr", "note": "Escrow-backed - DD required"},
            {"type": "Equity Stake", "title": "Kogi Platform 2.4%", "price": "$24K", "note": "Gov approval  cap table update"},
            {"type": "Revenue Stream", "title": "Design Retainer Rev-Share", "price": "$18K", "note": "12mo forward - MatchEngine"},
        ]
    })
}

pub fn sample_market_crm_snapshot() -> MarketplaceSnapshot {
    json!({
        "segments": ["Buy - Sell - Barter", "Grants  Microfinancing", "Crowdresourcing", "CRM  Booking", "Logistics"],
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
                    {"name": "Acme - Design  Dev", "value": "$28K", "note": "In progress"}
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
                "title": "Cooperative Design System Kit v3 - Figma  React",
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
        "subtitle": "Production-ready brand identity package with 480 deliverables.",
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
            "Color token system (light  dark)",
            "Typography scale  font files",
            "Component library (160 components)",
            "Icon set (480 icons)",
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
            "last_7d": "$48K",
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
            "Two proposals have been open for 8 days. Drafted follow-ups are ready."
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

pub fn sample_campaigns_collective_snapshot() -> MarketplaceSnapshot {
    json!({
        "hero": {
            "title": "Design System Revenue Share Campaign",
            "org": "Mesh Studio Cooperative",
            "terms": "12% revenue share - 2 years",
            "raised": "$27,000",
            "goal": "$60,000",
            "days_left": "56",
            "backers": "18",
            "avg_invest": "$1,500",
            "velocity": "$482/day",
            "progress": "45%"
        },
        "summary_cards": [
            metric("Funded", "45%", "Progress", "text-[#a855f7]"),
            metric("Backers", "18", "Active", "text-[#22c55e]"),
            metric("Velocity", "$482/d", "Daily", "text-[#10b981]"),
            metric("Days Left", "56", "Countdown", "text-[#f59e0b]"),
            metric("Page Views", "124", "Campaign page", "text-[#60a5fa]"),
            metric("Conversion", "7.2%", "View to back", "text-[#e6f1f4]"),
        ],
        "team": [
            {
                "name": "Jordan Chen",
                "role": "Campaign Lead",
                "tasks": ["Launch campaign page", "Write campaign description", "Send investor outreach x12", "Post update thread"]
            },
            {
                "name": "Lena Maier",
                "role": "Governance",
                "tasks": ["Draft governance proposal", "Collect member votes (5/8)", "Awaiting Tariq  Priya"]
            },
            {
                "name": "Sofia Reyes",
                "role": "Comms",
                "tasks": ["Design campaign banner", "Write pitch deck", "Product video (Loom)"]
            }
        ],
        "activity": [
            "Governance proposal: authorize milestone 2 disbursement - $18,000 to Operating Account.",
            "New backer confirmed - Cooperative Tech Inc committed $5,000.",
            "Outreach update: sent outreach to 8 warm leads.",
            "Content update: product demo video ready for review.",
            "Oba briefing: at current velocity, goal will be met 12 days after deadline."
        ],
        "details": {
            "entity": "Mesh Studio",
            "type": "Cooperative",
            "campaign": "Revenue Share",
            "goal": "$60,000",
            "min_investment": "$500",
            "all_or_nothing": "Yes",
            "governance": "6/8 vote"
        },
        "top_backers": [
            {"name": "Coop Tech Inc", "amount": "$5,000"},
            {"name": "Aisha Patel", "amount": "$4,000"},
            {"name": "Protocol Inc", "amount": "$3,500"},
        ],
        "milestones": [
            {"label": "M1 - Engineering Q2", "status": "Vote pending"},
            {"label": "M2 - Design  Docs", "status": "Locked"},
            {"label": "M3 - Launch  Marketing", "status": "Locked"}
        ],
        "quick_actions": [
            "Propose Disbursement Vote",
            "Post Campaign Update",
            "Invite Member",
            "Export Backer Report"
        ]
    })
}

pub fn sample_campaigns_capital_snapshot() -> MarketplaceSnapshot {
    json!({
        "summary_cards": [
            metric("Capital Raised", "$124,500", "Total to date", "text-[#22c55e]"),
            metric("Active Campaign", "$45,000", "Current round", "text-[#f59e0b]"),
            metric("Investors", "2", "Confirmed", "text-[#10b981]"),
            metric("Valuation Cap", "$14M", "SAFE", "text-[#a855f7]"),
        ],
        "active_campaign": {
            "title": "Alpha Platform - Seed Round Series Pre-A",
            "code": "KCMP-004",
            "raised": "$45,000",
            "goal": "$500,000",
            "days_left": "42",
            "investors": "2",
            "min_ticket": "$5,000",
            "cap": "$14M"
        },
        "milestones": [
            {"label": "M1 - Platform Beta Launch", "status": "Completed Feb 28, 2026", "amount": "$10,000"},
            {"label": "M2 - 500 Active Users", "status": "Target Mar 31, 2026", "amount": "$15,000"},
            {"label": "M3 - First Revenue $50K MRR", "status": "Target Jun 30, 2026", "amount": "$20,000"}
        ],
        "investor_contributions": [
            {"name": "Horizon Ventures LLC", "amount": "$25,000", "status": "Confirmed", "note": "Lead Investor - SAFE - Mar 02, 2026"},
            {"name": "Priya Nair", "amount": "$20,000", "status": "Confirmed", "note": "Angel - SAFE - Mar 08, 2026"},
            {"name": "DataStream Co", "amount": "$50,000", "status": "Pending", "note": "Strategic - Pending term sheet"}
        ],
        "investment_portfolio": [
            {"name": "Community Bond - Vehicle Fund", "note": "Invested $12,000 - Current $13,200"},
            {"name": "Seed Fund - Pamoja Collective", "note": "Invested $5,000 - Current $5,200"},
            {"name": "Equity Stake - NovaTech", "note": "Cost $8,000 - Mark $9,400"}
        ],
        "campaign_account": [
            {"label": "Account Balance", "value": "$35,000"},
            {"label": "Disbursed (M1)", "value": "$10,000"},
            {"label": "Held (M2)", "value": "$25,000"},
            {"label": "Pending (Priya)", "value": "$10,000"},
            {"label": "Platform Fee (1%)", "value": "-$450"}
        ],
        "cap_table": [
            {"holder": "Jordan Chen", "stake": "82%"},
            {"holder": "Horizon Ventures", "stake": "10.5%"},
            {"holder": "Priya Nair", "stake": "7.5%"},
            {"holder": "Option Pool", "stake": "10%"},
            {"holder": "Post-Round Valuation", "stake": "$14,000,000"}
        ]
    })
}

pub fn sample_campaigns_builder_snapshot() -> MarketplaceSnapshot {
    json!({
        "steps": [
            {"step": "Entity Type", "status": "done", "detail": "Individual Worker"},
            {"step": "Campaign Type", "status": "done", "detail": "Equity Crowdfunding"},
            {"step": "Link Portfolio", "status": "done", "detail": "Kogi Engine Project"},
            {"step": "Campaign Details", "status": "active", "detail": "Title, goal, terms, milestones"},
            {"step": "Governance Setup", "status": "pending", "detail": "Approval rules and multi-sig"},
            {"step": "Distribution Channels", "status": "pending", "detail": "Where to publish"},
            {"step": "Review and Launch", "status": "pending", "detail": "Final check before go-live"}
        ],
        "templates": [
            "Seed Round Template",
            "Revenue Share Template",
            "Resource Pool Template",
            "Mutual Aid Template"
        ],
        "campaign_details": {
            "title": "Campaign Details",
            "note": "Equity crowdfunding - individual worker - Step 4"
        },
        "campaign_basics": {
            "title": "Kogi Engine - Series Seed Round - $450,000",
            "pitch": "Raising $450K seed capital to scale the kogi-engine AI analytics and matching platform.",
            "description": "The kogi-engine powers matching, recommendations, risk scoring, and financial intelligence across the platform."
        },
        "funding_terms": {
            "goal": "$450,000",
            "currency": "USD",
            "equity_offered": "10%",
            "valuation": "$4,500,000",
            "min_investment": "$1,000",
            "max_investment": "$50,000",
            "all_or_nothing": "On",
            "governance_rights": "On"
        },
        "milestones": [
            {"label": "Milestone 1 - Engineering onboarding", "amount": "$135,000"},
            {"label": "Milestone 2 - Inference cluster", "amount": "$112,500"},
            {"label": "Milestone 3 - AnalyticsEngine v2", "amount": "$112,500"}
        ],
        "publish_channels": [
            {"channel": "Your Public Profile", "status": "Enabled"},
            {"channel": "Marketplace - Campaigns", "status": "Enabled"},
            {"channel": "Exchange - Financial Market", "status": "Enabled"},
            {"channel": "Builder Network Space", "status": "Enabled"},
            {"channel": "Mesh Studio Cooperative", "status": "Disabled"}
        ],
        "preview_card": {
            "title": "Kogi Engine - Series Seed Round",
            "owner": "Jordan Chen - Individual",
            "progress": "73%",
            "raised": "$328,500",
            "backers": "42",
            "cta": "Invest - Min $1,000"
        },
        "validation_checklist": [
            {"label": "Entity selected", "status": "ok"},
            {"label": "Campaign type set", "status": "ok"},
            {"label": "Portfolio linked", "status": "ok"},
            {"label": "Governance not configured", "status": "warn"}
        ]
    })
}

pub fn sample_campaigns_portfolio_snapshot() -> MarketplaceSnapshot {
    json!({
        "summary_cards": [
            metric("My Campaigns", "3", "Owned", "text-[#10b981]"),
            metric("Raised Total", "$124,500", "All time", "text-[#f59e0b]"),
            metric("Backers", "284", "Network", "text-[#e6f1f4]"),
            metric("Backed Campaigns", "7", "Invested", "text-[#22c55e]"),
            metric("Contributed", "$4,200", "Total", "text-[#a855f7]"),
        ],
        "filters": ["All Campaigns", "My Campaigns", "Backing", "Fundraise", "Equity", "Revenue Share", "Mutual Aid", "Group Buy"],
        "my_campaigns": [
            {
                "type": "Equity Crowdfunding",
                "title": "Kogi Platform Series A Infrastructure Round",
                "goal": "$450,000",
                "raised": "$124,500",
                "backers": "42",
                "avg_invest": "$4,500",
                "deadline": "28d"
            },
            {
                "type": "Revenue Share Pool",
                "title": "crdt-rs Library Sustainability Fund",
                "goal": "$25,000",
                "raised": "$18,400",
                "backers": "28",
                "avg_invest": "$657",
                "deadline": "14d"
            },
            {
                "type": "Fundraise - Donation",
                "title": "Worker OS Open-Source Infrastructure Fund",
                "goal": "$12,000",
                "raised": "$8,240",
                "backers": "214",
                "avg_invest": "$38",
                "deadline": "21d"
            },
            {
                "type": "Mutual Aid",
                "title": "Freelancers Emergency Fund - Round 3",
                "goal": "$30,000",
                "raised": "$24,800",
                "backers": "342",
                "avg_invest": "$72",
                "deadline": "7d"
            }
        ],
        "group_economics": [
            {"org": "Mesh Studio", "balance": "$124,500", "note": "MRR $28,400 - Payroll $18,200 - Reserve $48,000"},
            {"org": "OS Collective", "balance": "$18,400", "note": "Members 420 - Mutual Aid $4,200 - Grants $8,000"},
            {"org": "Builder Network", "balance": "$42,000", "note": "Active campaigns 4 - Backers 1,284"}
        ]
    })
}

pub fn sample_escrow_overview_snapshot() -> MarketplaceSnapshot {
    json!({
        "summary_cards": [
            metric("Total Locked", "$84,000", "Across escrows", "text-[#10b981]"),
            metric("Active Escrows", "5", "Live", "text-[#e6f1f4]"),
            metric("Released YTD", "$28,400", "Settled", "text-[#22c55e]"),
            metric("Pending Release", "$14,000", "Awaiting approval", "text-[#f59e0b]"),
            metric("Disputed", "1", "Needs mediation", "text-[#ef4444]"),
            metric("Avg Release Time", "3 days", "Escrow SLA", "text-[#60a5fa]"),
        ],
        "active_escrows": [
            {"title": "Brand Identity System v2.0", "deal": "DEAL-2026-012", "status": "Disputed", "amount": "$28,000"},
            {"title": "Rust Systems Engineer - 6mo", "deal": "DEAL-2026-018", "status": "Funded", "amount": "$43,200"},
            {"title": "CRDT Library License x2", "deal": "MKT-7721", "status": "Pending", "amount": "$1,680"},
            {"title": "DeFi Architecture Proposal", "deal": "DEAL-2026-021", "status": "Funded", "amount": "$11,250"},
        ],
        "selected_escrow": {
            "title": "Brand Identity System v2.0 - Milestone Escrow",
            "escrow_id": "ESC-2026-008",
            "deal_id": "DEAL-2026-012",
            "parties": "Patel Ventures <-> Jordan Chen",
            "status": "Dispute hold"
        },
        "dispute": {
            "note": "Deliverables did not match the agreed spec. Requesting partial refund or revised delivery.",
            "response": "Scope updated after kickoff. Will provide 2 additional concepts as goodwill."
        },
        "milestones": [
            {"label": "Milestone 1 of 4 - Initial Brand Concepts", "due": "Mar 10", "amount": "$7,000", "status": "Disputed"},
            {"label": "Milestone 2 of 4 - Brand Identity System", "due": "Mar 28", "amount": "$9,000", "status": "Locked"},
            {"label": "Milestone 3 of 4 - Component Library  Figma", "due": "Apr 14", "amount": "$8,000", "status": "Locked"},
            {"label": "Milestone 4 of 4 - Final Delivery", "due": "Apr 28", "amount": "$4,000", "status": "Locked"}
        ],
        "escrow_details": [
            {"label": "Escrow ID", "value": "ESC-2026-008"},
            {"label": "Type", "value": "Milestone - 4-stage"},
            {"label": "Status", "value": "Disputed - Frozen"},
            {"label": "Total Value", "value": "$28,000"},
            {"label": "Funded On", "value": "Mar 10, 2026"},
            {"label": "Auto-release", "value": "7 days after delivery"},
            {"label": "Platform Fee", "value": "2.5% ($700)"}
        ],
        "parties": [
            {"name": "Aisha Patel", "role": "Buyer - Dispute initiator"},
            {"name": "Jordan Chen", "role": "Seller - You"},
            {"name": "Kogi Mediation", "role": "Optional arbiter"}
        ],
        "resolution_actions": [
            "Propose Settlement Terms",
            "Request Kogi Mediation",
            "Submit Evidence",
            "Accept Revised Delivery"
        ],
        "activity_log": [
            "Dispute opened by Aisha Patel - Mar 12 2:41 PM",
            "Milestone 1 delivery submitted - Mar 9 11:00 AM",
            "Escrow funded - $28,000 locked - Mar 3 9:30 AM",
            "Escrow created from deal acceptance - Mar 2 4:15 PM"
        ]
    })
}

pub fn sample_escrow_bank_snapshot() -> MarketplaceSnapshot {
    json!({
        "summary_cards": [
            metric("Total Locked", "$18,300", "Escrow bank", "text-[#f59e0b]"),
            metric("Pending Release", "$13,900", "Awaiting approval", "text-[#22c55e]"),
            metric("Dispute Rate", "0%", "All clear", "text-[#e6f1f4]"),
            metric("Avg Release Time", "3.2d", "Escrow SLA", "text-[#38bdf8]"),
            metric("Released YTD", "$64,200", "Settled", "text-[#a855f7]"),
        ],
        "accounts": [
            {"escrow": "ESC-044", "deal": "Deal #44", "title": "Brand Identity System 2.0", "amount": "$4,200", "status": "Locked - M2"},
            {"escrow": "ESC-045", "deal": "Deal #45", "title": "API Integration Package", "amount": "$12,400", "status": "Pending Deposit"},
            {"escrow": "ESC-038", "deal": "Deal #38", "title": "Mobile App MVP", "amount": "$1,500", "status": "Release Ready"},
            {"escrow": "ESC-046", "deal": "Deal #46", "title": "Data Pipeline Audit", "amount": "$200", "status": "In Progress"}
        ],
        "selected_account": {
            "escrow": "ESC-044",
            "title": "Brand Identity System 2.0",
            "milestones": [
                {"label": "1 - Initial Concept  Research", "amount": "$840", "status": "Released"},
                {"label": "2 - Logo System  Brand Guidelines", "amount": "$1,680", "status": "In Review"},
                {"label": "3 - Digital Asset Package", "amount": "$1,260", "status": "Locked"},
                {"label": "4 - Final Delivery  Handoff", "amount": "$420", "status": "Locked"}
            ],
            "progress": "45% of funds released - $1,890 of $4,200"
        },
        "lifecycle": [
            "Escrow created - Mar 01, 2026 09:14 UTC",
            "Milestone 1 approved - Mar 05, 2026 14:22 UTC",
            "Milestone 2 in review - Mar 10, 2026 11:00 UTC",
            "Final delivery est. Mar 25, 2026"
        ],
        "escrow_details": [
            {"label": "Escrow ID", "value": "ESC-044"},
            {"label": "Total Value", "value": "$4,200.00"},
            {"label": "Released", "value": "$840.00"},
            {"label": "Remaining", "value": "$3,360.00"},
            {"label": "Platform Fee", "value": "2.5% - $105.00"},
            {"label": "Expiry", "value": "Apr 30, 2026"},
            {"label": "State Machine", "value": "KESC - Rust"}
        ]
    })
}

pub fn sample_exchange_overview_snapshot() -> MarketplaceSnapshot {
    json!({
        "summary_cards": [
            metric("Active Markets", "7", "Live markets", "text-[#10b981]"),
            metric("Open Deals", "12", "Negotiations", "text-[#e6f1f4]"),
            metric("Volume (30d)", "$284K", "All markets", "text-[#f59e0b]"),
            metric("Escrow Locked", "$38,400", "In escrow", "text-[#a855f7]"),
            metric("Settled (30d)", "$124,500", "Completed", "text-[#60a5fa]"),
            metric("Match Score", "94%", "AI match", "text-[#22c55e]"),
        ],
        "quick_links": [
            {"label": "Wallet", "value": "$4,820 Available", "note": "Add funds, invoice, or withdraw"},
            {"label": "Labor Market", "value": "284 Active Workers", "note": "AI-matched talent pools"},
            {"label": "Capital", "value": "6 Live Campaigns", "note": "Equity, rev-share, bonds"},
            {"label": "Asset Transfer", "value": "12 Active Transfers", "note": "Escrow-backed handoffs"}
        ],
        "active_markets": [
            {"market": "Labor Market", "listings": "284", "workers": "1,840", "avg_rate": "$94/hr", "heat": "78%"},
            {"market": "Portfolio Commodities", "items": "142", "volume": "$89K", "active": "28", "heat": "52%"},
            {"market": "Financial Instruments", "instruments": "34", "volume": "$124K", "rounds": "6", "heat": "64%"},
            {"market": "Resource Exchange", "resources": "97", "volume": "$41K", "types": "8", "heat": "43%"},
        ],
        "recent_activity": [
            {"event": "Escrow funded - $12,400 locked", "time": "2m ago"},
            {"event": "Proposal sent - Web Platform RFP", "time": "8m ago"},
            {"event": "Deal closed - React Component Kit", "time": "24m ago"},
            {"event": "New bid - Legal Template Bundle", "time": "41m ago"},
            {"event": "Listing active - Data Pipeline Automation", "time": "1h ago"},
        ],
        "open_deals": [
            {"title": "Brand Identity System v2.0", "owner": "Jordan Chen", "status": "Escrow", "amount": "$12,400"},
            {"title": "Web Platform Architecture", "owner": "Priya Singh", "status": "Negotiation", "amount": "$28,000"},
            {"title": "Community Growth Strategy", "owner": "Marcus Brown", "status": "Review", "amount": "$5,500"},
            {"title": "Legal Contract Templates", "owner": "Lena M.", "status": "Complete", "amount": "$2,800"},
        ],
        "order_book": [
            {"side": "BID", "item": "Senior React Developer - 3mo retainer", "price": "$9,200/mo", "status": "Active"},
            {"side": "ASK", "item": "UX/UI Design - Brand refresh - 6wk", "price": "$14,400", "status": "Active"},
            {"side": "BID", "item": "Data Engineer - Kafka pipeline setup", "price": "$7,800", "status": "Pending"},
            {"side": "ASK", "item": "Legal Review - SaaS contract templates", "price": "$3,200", "status": "Active"},
        ],
        "linked_platforms": ["Robinhood", "Stripe", "AngelList", "Coinbase", "Upwork", "Fiverr", "Wefunder", "Ethereum"]
    })
}

pub fn sample_exchange_labor_snapshot() -> MarketplaceSnapshot {
    json!({
        "summary_cards": [
            metric("Active Workers", "284", "Supply", "text-[#10b981]"),
            metric("Open Requirements", "48", "Demand", "text-[#e6f1f4]"),
            metric("Avg Hourly Rate", "$94/hr", "Blended", "text-[#f59e0b]"),
            metric("Active Contracts", "28", "Live", "text-[#60a5fa]"),
            metric("AI Match Score", "91%", "Fit", "text-[#22c55e]"),
        ],
        "filters": {
            "types": ["All", "Gig", "Contract", "Retainer"],
            "skills": ["Design", "Engineering", "Strategy", "Legal"]
        },
        "matched_workers": [
            {"name": "Jordan Chen", "role": "Brand Designer - Visual Identity", "rate": "$124/hr", "match": "96%", "jobs": "48", "rating": "4.9"},
            {"name": "Priya Singh", "role": "Full-Stack Engineer - Platform", "rate": "$145/hr", "match": "91%", "jobs": "62", "rating": "4.8"},
            {"name": "Marcus Brown", "role": "Growth Strategist - Community", "rate": "$95/hr", "match": "88%", "jobs": "34", "rating": "4.7"},
        ],
        "worker_profile": {
            "name": "Jordan Chen",
            "location": "San Francisco, CA",
            "rate": "$124/hr",
            "range": "$740-$4,960/day",
            "skills": ["Brand Identity", "Visual Design", "Figma", "Motion", "Design Systems"],
            "stats": [
                {"label": "Jobs done", "value": "48"},
                {"label": "Rating", "value": "4.9/5"},
                {"label": "On-time", "value": "98%"},
                {"label": "AI rep", "value": "94/100"}
            ]
        },
        "open_requirements": [
            {"title": "Platform Visual Redesign - Web  Mobile", "note": "RFP - 10-12 weeks - $28,000-$40,000"},
            {"title": "Brand Style Guide - SaaS Startup", "note": "Gig - 2-3 weeks - $8,000"},
            {"title": "Design Lead - Community Platform", "note": "Contract - $9,000/mo"}
        ]
    })
}

pub fn sample_exchange_commodities_snapshot() -> MarketplaceSnapshot {
    json!({
        "summary_cards": [
            metric("Listed Items", "142", "Catalog", "text-[#10b981]"),
            metric("Volume (30d)", "$89,400", "Trades", "text-[#f59e0b]"),
            metric("Avg Asset Val.", "$8,200", "Blended", "text-[#e6f1f4]"),
            metric("Active Trades", "21", "Live", "text-[#60a5fa]"),
            metric("DD Reports", "8", "Due diligence", "text-[#10b981]"),
        ],
        "listings": [
            {"title": "Brand Identity System v2.0", "owner": "Jordan Chen", "type": "Asset", "price": "$12,400", "ai": "94"},
            {"title": "Token Distribution Engine - Rust", "owner": "Priya Singh", "type": "Artifact", "price": "$6,500", "ai": "88"},
            {"title": "Investment Playbook - Independent Worker", "owner": "Marcus Brown", "type": "Playbook", "price": "$180", "ai": "92"}
        ],
        "asset_detail": {
            "title": "Brand Identity System v2.0",
            "tags": ["Active", "142 followers", "94 AI"],
            "price": "$12,400",
            "pricing": "Fixed - Full IP license",
            "ai_due_diligence": [
                {"label": "Portfolio Health", "value": "94/100"},
                {"label": "Artifact Maturity", "value": "91/100"},
                {"label": "Risk Score", "value": "Low - 12"},
                {"label": "Asset Value AI", "value": "$13,200"}
            ],
            "governance": [
                {"label": "Created", "value": "Nov 2024"},
                {"label": "Last Updated", "value": "Mar 8, 2026"},
                {"label": "IP Ownership", "value": "Verified"}
            ]
        }
    })
}

pub fn sample_exchange_capital_snapshot() -> MarketplaceSnapshot {
    json!({
        "summary_cards": [
            metric("Total Raised", "$124,500", "All instruments", "text-[#f59e0b]"),
            metric("Active Campaigns", "6", "Live", "text-[#e6f1f4]"),
            metric("My Invested Capital", "$450K", "Deployed", "text-[#10b981]"),
            metric("Portfolio Return", "18.4%", "IRR", "text-[#22c55e]"),
            metric("Funding Deadline", "Apr 1", "Next close", "text-[#f59e0b]"),
            metric("Avg Campaign Fill", "73%", "Blended", "text-[#60a5fa]"),
        ],
        "active_campaigns": [
            {"title": "Alpha Platform - Seed Round", "type": "Equity", "progress": "89%", "note": "$450K of $500K", "days_left": "19"},
            {"title": "Community Growth Fund", "type": "Rev Share", "progress": "62%", "note": "$62K of $100K", "days_left": "41"},
            {"title": "Vehicle Pool - Community Bond", "type": "Bond", "progress": "48%", "note": "$120K of $250K", "days_left": "60"},
        ],
        "watchlist": [
            {"name": "Alpha Platform - Pre-A", "value": "$6.25M"},
            {"name": "Community Growth - Rev Pool", "value": "12% / 36mo"},
            {"name": "Data Infrastructure - Conv Note", "value": "$50K - 18mo"},
            {"name": "Vehicle Pool Bond", "value": "6.5% - 24mo"}
        ],
        "top_backers": [
            {"name": "Marcus & Rina Ventures", "amount": "$125,000"},
            {"name": "Nkrumah Fund", "amount": "$100,000"},
            {"name": "Jordan Park (Angel)", "amount": "$75,000"},
            {"name": "Asha Singh (Angel)", "amount": "$50,000"}
        ],
        "positions": [
            {"name": "Alpha Platform - Seed (2024)", "amount": "$50,000 in", "return": "62%"},
            {"name": "Community Growth Fund (2025)", "amount": "$25,000 in", "return": "16.8%"},
            {"name": "Vehicle Pool Bond Series A", "amount": "$40,000 in", "return": "6.5%"},
            {"name": "Data Infrastructure - Convertible Note", "amount": "$30,000 in", "return": "Pending"}
        ]
    })
}

pub fn sample_exchange_resources_snapshot() -> MarketplaceSnapshot {
    json!({
        "summary_cards": [
            metric("Listed Resources", "142", "Marketplace", "text-[#10b981]"),
            metric("Active Allocations", "8", "In use", "text-[#e6f1f4]"),
            metric("Volume (30d)", "$41,200", "Trades", "text-[#f59e0b]"),
            metric("Avg Utilization", "73%", "Capacity", "text-[#60a5fa]"),
            metric("Barter Matches", "5", "Swaps", "text-[#10b981]"),
        ],
        "categories": ["All", "Compute", "Design Tools", "Data", "Infrastructure", "Legal", "Barter"],
        "resources": [
            {"title": "Cloud GPU Cluster - A100 x8", "type": "Compute", "note": "8x NVIDIA A100 80GB on-demand", "price": "$4.80 / GPU hr"},
            {"title": "Design System Assets  Figma Library", "type": "Design", "note": "Component library, icons, patterns, tokens", "price": "$80 / seat / mo"},
            {"title": "Legal Template Bundle - SaaS  IP", "type": "Legal", "note": "40 templates - US  EU versions", "price": "$240 fixed"},
            {"title": "Kafka  Flink Data Pipeline", "type": "Infra", "note": "10M events/day capacity - 99.9% SLA", "price": "$320 / mo"},
            {"title": "Worker Market Research 2025", "type": "Data", "note": "Survey data  analysis from 4,200 workers", "price": "$180 one-time"},
            {"title": "Barter: Dev Time <-> Design Hours", "type": "Barter", "note": "Offering 10hrs/wk React development", "price": "AI match 88%"}
        ]
    })
}

pub fn sample_exchange_wallet_snapshot() -> MarketplaceSnapshot {
    json!({
        "balance": {
            "available": "$4,820.00",
            "kyc": "KYC Level 2 - Wire transfers enabled",
            "actions": ["Withdraw", "Deposit", "Send", "Invoice"]
        },
        "summary_cards": [
            metric("Revenue (YTD)", "$12,340", "$3,200 vs last year", "text-[#22c55e]"),
            metric("In Escrow", "$2,180", "2 active orders", "text-[#f59e0b]"),
            metric("Available", "$4,820", "Ready to withdraw", "text-[#60a5fa]"),
            metric("Overdue Invoices", "2", "$3,400 outstanding", "text-[#ef4444]"),
        ],
        "transactions": [
            {"title": "Invoice #108 - DesignCo", "note": "Payment received - Mar 5, 2025", "amount": "$2,000", "time": "3 days ago"},
            {"title": "Escrow release - TechCorp", "note": "Order #ORD-2024-089 complete", "amount": "$1,200", "time": "5 days ago"},
            {"title": "Figma subscription", "note": "Monthly recurring - Auto-pay", "amount": "-$45", "time": "Mar 1"},
            {"title": "Invoice #107 - StartupXYZ", "note": "Partial payment received", "amount": "$800", "time": "Feb 28"},
            {"title": "Marketplace order - Alex Kim", "note": "Escrow deposit locked", "amount": "~$980", "time": "Feb 26"}
        ],
        "invoices": [
            {"id": "INV-2024-108", "client": "DesignCo", "status": "14 days overdue", "amount": "$2,000"},
            {"id": "INV-2024-109", "client": "StartupXYZ", "status": "7 days overdue", "amount": "$1,400"},
            {"id": "INV-2024-110", "client": "TechCorp", "status": "Due Mar 20", "amount": "$3,200"},
            {"id": "INV-2024-111", "client": "FintechApp", "status": "Draft", "amount": "$800"}
        ],
        "escrow_orders": [
            {"order": "ORD-2025-001", "client": "Alex Kim", "note": "React dev - In progress", "amount": "$1,200"},
            {"order": "ORD-2025-002", "client": "Marcus Chen", "note": "Design - Review", "amount": "$980"}
        ]
    })
}

pub fn sample_exchange_bids_snapshot() -> MarketplaceSnapshot {
    json!({
        "summary_cards": [
            metric("Active Bids", "12", "Live bids", "text-[#10b981]"),
            metric("Offers Out", "8", "Active offers", "text-[#22c55e]"),
            metric("Closed Deals", "5", "Completed", "text-[#a855f7]"),
            metric("Pipeline Value", "$142K", "Open value", "text-[#f59e0b]"),
        ],
        "bids_offers": [
            {"item": "Design System License", "type": "Offer", "value": "$12K", "status": "Active", "counterparty": "Active"},
            {"item": "API Access Token", "type": "Bid", "value": "$3,500", "status": "Pending", "counterparty": "Pending"},
            {"item": "Brand Strategy Pack", "type": "Offer", "value": "$8K", "status": "Active", "counterparty": "Active"},
            {"item": "UX Audit Service", "type": "Bid", "value": "$5K", "status": "Review", "counterparty": "Review"},
            {"item": "Full-Stack Dev Sprint", "type": "Offer", "value": "$15K", "status": "Expiring", "counterparty": "Expiring"},
        ],
        "deal_pipeline": [
            {"title": "Acme Contract", "value": "$28K", "progress": "55%"},
            {"title": "DevDAO Sprint", "value": "$48K", "progress": "70%"},
            {"title": "Brand Package", "value": "$18K", "progress": "40%"},
            {"title": "Startup Deal", "value": "$12K", "progress": "25%"}
        ],
        "requests": [
            {"title": "Due Diligence Review", "org": "Acme Corp", "status": "Pending"},
            {"title": "Contract Signed", "org": "DevDAO", "status": "Done"},
            {"title": "Proposal Sent", "org": "Invest Club", "status": "Sent"}
        ],
        "linked_platforms": ["AngelList", "Deel", "Upwork", "Toptal"],
        "actions": ["Open Deal Room", "View Escrow"]
    })
}

pub fn sample_exchange_barter_snapshot() -> MarketplaceSnapshot {
    json!({
        "summary_cards": [
            metric("Open Swap Offers", "284", "Marketplace", "text-[#10b981]"),
            metric("My Active Offers", "3", "In flight", "text-[#e6f1f4]"),
            metric("Swaps Completed", "8", "Closed", "text-[#22c55e]"),
            metric("Pending Matches", "2", "Awaiting", "text-[#f59e0b]"),
            metric("Value Swapped", "$48K", "Total", "text-[#60a5fa]"),
        ],
        "active_swap": {
            "offer": "Systems Architecture Review",
            "offer_value": "$1,440",
            "receive": "Cooperative Design System v3",
            "receive_value": "$1,200",
            "counterparty": "Amara Kofi",
            "match_score": "97",
            "status": "Awaiting confirmation"
        },
        "open_offers": [
            {"name": "Sofia Reyes", "offering": "Brand Identity Kit v2", "wants": "Architecture Review", "match": "91%"},
            {"name": "Dev Kim", "offering": "React Dev Time", "wants": "Brand System", "match": "88%"},
        ],
        "my_offers": [
            {"offering": "Systems Architecture Review", "wants": "Design System v3", "status": "Pending"},
            {"offering": "Brand Strategy Pack", "wants": "Legal Templates", "status": "Accepted"},
            {"offering": "Growth Sprint", "wants": "Data Pipeline Audit", "status": "Expired"},
        ]
    })
}

pub fn sample_exchange_deal_room_snapshot() -> MarketplaceSnapshot {
    json!({
        "header": {
            "buyer": "You",
            "seller": "Jordan Chen",
            "listing": "Brand Identity System v2.0",
            "opened": "Mar 3",
            "deadline": "Apr 14",
            "milestones": "3",
            "price": "$12,400"
        },
        "actions": ["DD Report", "Dispute", "Counter", "Fund Escrow", "Approve Delivery", "View Escrow"],
        "chat": [
            {"sender": "System", "message": "Deal Room opened - Mar 3, 2026 - #DR-2281"},
            {"sender": "You", "time": "Mar 3 10:14am", "message": "Looking to refresh the brand for Alpha Platform. Need full identity system over 6 weeks. Budget around $12K."},
            {"sender": "Jordan Chen", "time": "Mar 3 11:32am", "message": "Happy to break into three milestones: Research  Strategy, Core Identity, Full System Delivery. Formal proposal coming."},
            {"sender": "System", "message": "Proposal v2 submitted - $12,400 - Mar 5"},
            {"sender": "System", "message": "You accepted Proposal v2 - Mar 6"},
            {"sender": "System", "message": "Escrow funded - $12,400 locked - Mar 6"},
            {"sender": "Jordan Chen", "time": "Mar 8 2:00pm", "message": "Milestone 1 complete. Research and strategy deck uploaded."}
        ],
        "oba_note": "Milestone 1 is ready for review. Recommend approving to release $3,800 and keep the project moving.",
        "proposal": {
            "scope": "Full Brand Identity",
            "timeline": "Mar 8 -> Apr 14",
            "total_price": "$12,400",
            "pricing_model": "Fixed - Milestone",
            "revisions": "2 per milestone",
            "ip_transfer": "On final payment",
            "milestones": [
                {"label": "Milestone 1 - Research  Strategy", "amount": "$3,800", "status": "Pending"},
                {"label": "Milestone 2 - Core Identity", "amount": "$5,200", "status": "Locked"},
                {"label": "Milestone 3 - Full System  Motion", "amount": "$3,400", "status": "Locked"}
            ]
        },
        "escrow_status": {
            "locked": "$12,400",
            "released": "$0",
            "pending": "$3,800",
            "remaining": "$8,600"
        },
        "activity_log": [
            "Milestone 1 submitted for review - Mar 8",
            "Escrow funded - $12,400 locked - Mar 6",
            "Proposal v2 accepted - Mar 6",
            "Proposal v1 submitted - Mar 4"
        ]
    })
}

pub fn sample_exchange_asset_transfer_snapshot() -> MarketplaceSnapshot {
    json!({
        "summary_cards": [
            metric("Active Transfers", "12", "In motion", "text-[#10b981]"),
            metric("Transfer Volume", "$64,200", "Total", "text-[#f59e0b]"),
            metric("Avg Settlement", "1.4 days", "Speed", "text-[#e6f1f4]"),
            metric("Items Transferred", "48", "Count", "text-[#60a5fa]"),
            metric("Success Rate", "99.2%", "Reliability", "text-[#22c55e]"),
        ],
        "workflow_steps": [
            {"step": "Select Asset", "status": "done"},
            {"step": "Set Recipient", "status": "done"},
            {"step": "Configure Terms", "status": "active"},
            {"step": "Review and Sign", "status": "pending"},
            {"step": "Escrow and Execute", "status": "pending"}
        ],
        "asset": {
            "title": "Brand Identity System v2.0",
            "type": "Asset - v2.4 - AI 94",
            "value": "$12,400",
            "transfer_type": "Full IP Transfer",
            "payment_method": "Escrow - kogi-bank"
        },
        "recipient": {
            "name": "Jordan Park (Angel)",
            "note": "Kogi user - Verified - Alpha Platform backer",
            "ip_assignment": "On final payment",
            "retention": "Seller retains portfolio reference",
            "governance": "Verified",
            "audit_trail": "Immutable EventLog"
        },
        "transfer_log": [
            {"title": "Brand Identity System v2.0", "direction": "Outgoing", "status": "Escrow funded", "amount": "$12,400"},
            {"title": "React Component Library - v4.1", "direction": "Incoming", "status": "Active subscription", "amount": "$280/mo"},
            {"title": "Investment Playbook v3.1", "direction": "Outgoing", "status": "Completed", "amount": "$45/mo"},
        ],
        "transfer_parties": [
            {"name": "Jordan Chen", "role": "Seller", "note": "Transfers 12 - Vol $64K"},
            {"name": "Jordan Park", "role": "Buyer", "note": "Capital $450K"},
            {"name": "kogi-bank", "role": "Escrow service", "note": "$12,400 locked"}
        ]
    })
}

pub fn sample_barter_exchange_snapshot() -> MarketplaceSnapshot {
    json!({
        "summary_cards": [
            metric("Open Swap Offers", "284", "Marketplace", "text-[#10b981]"),
            metric("My Active Offers", "3", "In flight", "text-[#e6f1f4]"),
            metric("Swaps Completed", "8", "Closed", "text-[#22c55e]"),
            metric("Pending Matches", "2", "Awaiting", "text-[#f59e0b]"),
            metric("Total Value Swapped", "$48K", "All time", "text-[#60a5fa]"),
        ],
        "active_swap": {
            "offer": "Systems Architecture Review",
            "offer_type": "Labor - Portfolio Item",
            "offer_value": "$1,440",
            "receive": "Cooperative Design System v3",
            "receive_type": "Asset - Portfolio Item",
            "receive_value": "$1,200",
            "counterparty": "Amara Kofi",
            "match_score": "97",
            "status": "Awaiting confirmation"
        },
        "open_offers": [
            {"name": "Sofia Reyes", "offering": "Brand Identity Kit v2", "wants": "Architecture Review", "match": "91%", "value": "$1,200"},
            {"name": "Lena Maier", "offering": "Governance Framework", "wants": "Infrastructure Audit", "match": "84%", "value": "$800"},
            {"name": "Tariq Nasir", "offering": "500 AWS Credits", "wants": "Rust Code Review", "match": "78%", "value": "$500"},
        ],
        "my_active_offers": [
            {"offer": "Architecture Consulting - 8 hrs", "want": "Design system", "status": "Pending", "proposals": "1"},
            {"offer": "crdt-rs License", "want": "UI component library", "status": "Active", "proposals": "3"},
            {"offer": "Technical Writing - 6 hrs", "want": "Motion edit", "status": "Active", "proposals": "0"},
        ],
        "how_it_works": [
            "Post what you are offering",
            "Describe what you want",
            "MatchEngine pairs offers",
            "Negotiate in Deal Room",
            "Both confirm -> swap closes"
        ],
        "swappable_types": [
            "Labor / Skills",
            "Assets and Artifacts",
            "Resources",
            "Knowledge"
        ]
    })
}

pub fn sample_barter_bids_snapshot() -> MarketplaceSnapshot {
    json!({
        "summary_cards": [
            metric("Open Bids", "27", "Active", "text-[#10b981]"),
            metric("Matched This Month", "9", "Swaps", "text-[#22c55e]"),
            metric("Bid Bond Locked", "$4,800", "Escrow reserve", "text-[#60a5fa]"),
            metric("Win Rate", "68%", "Bid success", "text-[#f59e0b]"),
        ],
        "active_bids": [
            {"bid": "Rust Systems Audit", "market": "KLM", "type": "Bid", "status": "Leading", "amount": "$3,200"},
            {"bid": "Design Token Library", "market": "KPCM", "type": "Limit", "status": "Outbid", "amount": "$980"},
            {"bid": "Compute Capacity - Q2", "market": "KRM", "type": "Auction", "status": "Queued", "amount": "$1,450"},
        ],
        "bid_ladder": {
            "top_bids": [
                {"item": "Security Audit", "amount": "$3,250"},
                {"item": "Tokenized License", "amount": "$1,120"},
                {"item": "Compute Credits", "amount": "$540"},
                {"item": "Governance Advisor", "amount": "$2,100"},
            ],
            "alerts": [
                {"event": "Outbid on Design Tokens", "time": "Just now"},
                {"event": "Bid accepted - KLM", "time": "25m"},
                {"event": "New RFP matched", "time": "2h"},
                {"event": "Bid expiring", "time": "4h"},
            ]
        },
        "bid_builder": {
            "market": "KLM",
            "bid_type": "Limit Bid",
            "price": "$3,200",
            "expiration": "72h"
        },
        "bid_bond": {
            "reserve": "$4,800",
            "note": "Bid bonds release on loss and convert to escrow on acceptance.",
            "account": "KESC"
        },
        "matchengine_insights": "3 new RFPs match your skill profile. Raising max bid by 5% improves win odds by 12%."
    })
}

pub fn sample_barter_offers_snapshot() -> MarketplaceSnapshot {
    json!({
        "summary_cards": [
            metric("Active Offers", "18", "Open asks", "text-[#60a5fa]"),
            metric("Matched This Month", "6", "Swaps", "text-[#22c55e]"),
            metric("Avg Ask Value", "$1,240", "Median", "text-[#a855f7]"),
            metric("Expiring Soon", "3", "Urgent", "text-[#f59e0b]"),
        ],
        "active_offers": [
            {"offer": "Design System License", "market": "KPCM", "type": "Limit", "status": "Active", "ask": "$1,200"},
            {"offer": "Research Sprint - 4 weeks", "market": "KLM", "type": "Proposal", "status": "Countered", "ask": "$8,500"},
            {"offer": "Compute Credits - 500 units", "market": "KRM", "type": "Market", "status": "Matched", "ask": "$540"},
        ],
        "order_book": {
            "top_asks": [
                {"item": "Design Systems Kit", "amount": "$1,050"},
                {"item": "Governance Framework", "amount": "$820"},
                {"item": "Compute Credits", "amount": "$520"},
                {"item": "Data Licensing Pack", "amount": "$2,400"},
            ],
            "activity": [
                {"event": "New ask posted - KPCM", "time": "12m"},
                {"event": "Offer matched - KLM", "time": "1h"},
                {"event": "Ask updated - KRM", "time": "3h"},
                {"event": "OTC offer opened", "time": "6h"},
            ]
        },
        "offer_builder": {
            "market": "KPCM",
            "offer_type": "Limit Ask",
            "price": "$1,250",
            "terms": "Net 7 - Escrow"
        },
        "offer_types": ["Ask", "Limit", "Auction", "Proposal", "Swap", "OTC"],
        "oba_insights": "Your top offer is 12% above market median. Consider dropping by $80 to improve match speed."
    })
}

pub fn sample_barter_deals_snapshot() -> MarketplaceSnapshot {
    json!({
        "deal_header": {
            "title": "Brand Identity System v2.0",
            "buyer": "You",
            "seller": "Jordan Chen",
            "room": "KRMS Deal Room",
            "escrow": "Active",
            "price": "$12,400"
        },
        "deal_summary": [
            {"label": "Opened", "value": "Mar 3"},
            {"label": "Deadline", "value": "Apr 14"},
            {"label": "Milestones", "value": "3"},
            {"label": "Proposals", "value": "v2"}
        ],
        "deal_rooms": [
            {"title": "Brand Identity System v2.0", "status": "Negotiation", "note": "Counter-proposal sent", "amount": "$28K"},
            {"title": "Senior Rust Engineer - 6mo", "status": "Proposal received", "note": "New bid $180/hr", "amount": "$180/hr"},
            {"title": "Mesh Studio Q2 Consulting", "status": "Signed", "note": "Contract active - Monthly", "amount": "$8K/mo"},
            {"title": "Open Source Contributor Grant", "status": "Investment room", "note": "Review committee notified", "amount": "$10K"}
        ],
        "chat": [
            {"sender": "You", "time": "Mar 3 10:14am", "message": "Looking to refresh the brand for Alpha Platform. We need a full identity system with three milestones over six weeks."},
            {"sender": "Jordan Chen", "time": "Mar 3 11:32am", "message": "Happy to break this into three milestones: Research, Core Identity, and Full System Delivery. I will send a proposal shortly."},
            {"sender": "System", "time": "Mar 5 11:22am", "message": "Proposal v2 submitted - $12,400"}
        ],
        "oba_note": "Milestone 1 is ready for review. Recommendation: approve to release $3,800 and keep delivery on track.",
        "proposal": {
            "scope": "Full Brand Identity",
            "timeline": "Mar 8 -> Apr 14",
            "total_price": "$12,400",
            "pricing_model": "Fixed - Milestones",
            "milestones": [
                {"label": "Milestone 1 - Research", "amount": "$3,800", "due": "Mar 15"},
                {"label": "Milestone 2 - Core Identity", "amount": "$5,200", "due": "Apr 2"},
                {"label": "Milestone 3 - Full System", "amount": "$3,400", "due": "Apr 14"}
            ]
        },
        "escrow_status": {
            "released": "$0",
            "pending": "$3,800",
            "locked": "$8,600"
        },
        "activity_log": [
            "Milestone 1 submitted for review - Mar 8",
            "Escrow funded - $12,400 locked - Mar 6",
            "Proposal v2 accepted - Mar 6",
            "Proposal v2 submitted - Mar 5"
        ],
        "due_diligence": [
            {"label": "Portfolio verified", "status": "Verified"},
            {"label": "AI reputation score", "status": "97 / 100"},
            {"label": "IP assignment clause", "status": "Pending"},
            {"label": "Proposal v2", "status": "View"},
            {"label": "Counter #2 - $28K", "status": "Rejected"},
            {"label": "Brand Strategy Deck", "status": "Delivered"}
        ]
    })
}

