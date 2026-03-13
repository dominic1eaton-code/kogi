

**KOGI PLATFORM**

**Community, Spaces & Social Management System**

Design Document — v1.0

March 2026

| *This document defines the architecture, subsystems, data models, interaction primitives, and AI workflows for the Kogi Community, Spaces, and Social Management System — the social layer of the Kogi independent-worker operating platform, comprising Spaces (KSPC), Rooms (KRMS), Feeds, Posts, Organizations, and the social graph that connects all users, portfolios, and communities.* |
| :---- |

| Module Code | KSPC · KRMS · KCOM · KORG |
| :---- | :---- |
| **Module Name** | Spaces · Rooms · Community Feed · Organizations |
| **Primary Language** | Go (services) · Rust (systems) · Scala (data engine) · Angular (web client) |
| **Database** | PostgreSQL (network) · SQLite (local) · Redis (cache/realtime) · Kafka (streams) |
| **Protocol** | REST/GraphQL (external) · gRPC (internal) · WebSocket (realtime) |
| **Status** | Design Phase — MVP Definition |

| 1\. SYSTEM OVERVIEW |
| :---: |

## **1.1 Purpose & Vision**

The Kogi Community, Spaces & Social Management System is the connective social tissue of the Kogi platform. Where other modules — Office, Bank, Exchange, Studio — manage a user's work and assets, the Community layer connects users to one another: through shared spaces and rooms, social feeds and posts, events, organizations, teams, collectives, and cooperatives.

The design philosophy is that the social layer must feel native to portfolio work — not a bolted-on social network, but a purposeful space where independent workers, collectives, cooperatives, and autonomous organizations discover one another, collaborate, exchange, and govern themselves. Every post, space, or room is tied to portfolio components, and every action generates data that the Kogi Engine uses to personalize, match, and recommend.

## **1.2 Position in the Kogi Platform**

The Community system sits between the private work world (Office, Bank, Studio) and the public economic layer (Exchange, Marketplace). It is the platform's social graph and real-time collaboration backbone.

| Upstream (feeds into Community) | kogi-office (portfolios, projects) · kogi-studio (creative assets) · kogi-bank (campaigns) |
| :---- | :---- |
| **Downstream (Community feeds into)** | kogi-exchange (matchmaking) · kogi-marketplace (discovery) · kogi-engine (analytics, recommendations) |
| **Parallel modules** | kogi-home (dashboard aggregator) · kogi-organizations (governance) · kogi-profile (identity) |
| **AI Layer** | Oba assistant · RecommendationEngine · PersonalizationEngine · MatchEngine |

## **1.3 Core Subsystems**

| Code | Subsystem | Primary Responsibility |
| :---- | :---- | :---- |
| KSPC | Spaces | Digital community spaces, organizations, teams, events |
| KRMS | Rooms | Real-time chat, DMs, group rooms, project rooms |
| KFED | Feed Service | Activity feeds, posts, timelines, notifications |
| KORG | Organizations | Autonomous orgs, collectives, cooperatives, federations |
| KSGR | Social Graph | Follow, subscribe, watch, connect relationships |
| KMTC | Match & Discovery | Worker matching, space recommendations, talent discovery |
| KENG·COM | Community Engine | Analytics, personalization, sentiment, engagement scoring |

| 2\. KSPC — SPACES |
| :---: |

## **2.1 Concept**

A Space is a named digital community environment. Spaces are the top-level social containers on the Kogi platform, equivalent to a workspace, server, or hub in other social platforms — but purpose-built for portfolio work, cooperative economics, and independent-worker collaboration.

Spaces can represent teams, organizations, communities of practice, project communities, cooperative guilds, investment clubs, DAOs, or any self-defined group. Each Space has rooms, events, member management, a feed, and optionally an organizational structure.

## **2.2 Space Types**

| Type | Description | Example |
| :---- | :---- | :---- |
| Community | Open or invite-only community around a topic or interest | Real Estate Investors Community |
| Team | Collaborative working group, squads, tribes, guilds, chapters | Design Squad Alpha |
| Organization | Formal legal or operational entity (LLC, Corp, Trust, Fund) | Pamoja Capital LLC |
| Collective | Informal resource-sharing group, skill pool | Freelance Dev Collective |
| Cooperative | Member-owned economic cooperative | Worker-Owned Creative Coop |
| DAO / Autonomous Org | Decentralized autonomous organization with on-chain governance | Protocol Builders DAO |
| Federation | A network of linked Spaces, Collectives, or Orgs | Pamoja Federation |
| Project Space | A Space tied to a specific portfolio project or program | Brand Launch 2026 Space |
| Private Studio | Personal creative or research environment | Jordan's Innovation Lab |

## **2.3 Space Data Model**

Every Space is a PortfolioComponent at its core, inheriting the full component graph and metadata system:

| Field | Description |
| :---- | :---- |
| **space.id (UUID)** | Platform-unique identifier |
| **space.type** | community | team | organization | collective | cooperative | dao | federation | project | studio |
| **space.name** | Display name of the Space |
| **space.handle** | Unique @handle for discovery and mention |
| **space.visibility** | public | private | protected | invite-only |
| **space.status** | active | paused | archived | pending |
| **space.owners\[\]** | UUIDs of owning users; hierarchical privilege tiers |
| **space.members\[\]** | Member roster with roles: owner | admin | mod | member | contributor | viewer | guest |
| **space.rooms\[\]** | Linked Room component IDs |
| **space.events\[\]** | Linked Event component IDs |
| **space.feed** | Space-scoped activity feed ID |
| **space.organization** | Optional linked Organization entity |
| **space.portfolio\_links\[\]** | Linked PortfolioComponent IDs (projects, programs, assets) |
| **space.tags\[\]** | Topic/hashtag labels for discoverability |
| **space.policies\[\]** | Governance and access policy IDs |
| **space.metadata** | id · owners · created\_at · updated\_at · vector\_clock · version · properties |
| **space.analytics** | Engagement metrics, member growth, reach, sentiment scores |

## **2.4 Space Actions & Interactions**

Users and systems can take the following actions on a Space:

| Action | Actor | Notes |
| :---- | :---- | :---- |
| create | User / System | Initialize a new Space with type, name, visibility |
| join | User | Request or instant join depending on visibility policy |
| invite | Member | Send invite to specific user or group |
| follow | User | Subscribe to Space feed without joining |
| subscribe | User | Paid or privileged subscription tier |
| post | Member | Publish content to Space feed |
| event.create | Member (admin+) | Create and schedule an Event within the Space |
| room.create | Member (admin+) | Spin up a new Room inside the Space |
| poll / survey | Member | Create a poll visible to Space members |
| campaign | Member (admin+) | Launch fundraising or resource-gathering campaign |
| governance.vote | Member | Participate in Space governance decisions |
| tag / mention | Member | Tag portfolio components or @mention users |
| report | Member | Report content or member for moderation |
| archive | Owner | Move Space to deep archive state |
| federate | Owner | Link Space into a Federation graph |

## **2.5 Space Discovery & Search**

Spaces are discoverable through the Kogi SearchEngine and RecommendationEngine. Discovery surfaces include:

* Explore feed — globally recommended Spaces based on PersonalizationEngine scoring

* Search — full-text and tag-based search via the QueryEngine and SearchEngine

* Discover — AI-curated topic expansions: 'related to X, you might explore...'

* Match — the MatchEngine links user skill profiles to relevant Spaces and communities

* Trending — TelemetryEngine surfaces trending Spaces by engagement velocity

| 3\. KRMS — ROOMS |
| :---: |

## **3.1 Concept**

Rooms are real-time communication channels. They exist within Spaces or as standalone communication threads attached to portfolio items, projects, deals, or other platform entities. Rooms are the primary synchronous and asynchronous collaboration mechanism on the Kogi platform.

## **3.2 Room Types**

| Room Type | Description | Linked Entity |
| :---- | :---- | :---- |
| Direct (DM) | Private 1:1 messaging between two users | User → User |
| Group | Multi-user private channel (up to configurable limit) | User group |
| Space Room | A channel inside a Space (general, topic, project) | Space |
| Project Room | Collaborative room tied to a specific portfolio project | Project (PortfolioItem) |
| Portfolio Room | Coordination room for a whole portfolio or program | Portfolio / Program |
| Deal Room | Negotiation space for marketplace deals or exchange proposals | Exchange listing |
| Community Room | Public or semi-public community discussion channel | Community / Space |
| Event Room | Temporary room for an event or meeting | Event |
| Governance Room | DAO/org voting and proposal discussion room | Organization |
| AI Room | Chat interface with the Oba AI assistant | System / User |
| Support Room | User support and help desk threads | Platform support |

## **3.3 Room Data Model**

| room.id | Platform-unique UUID |
| :---- | :---- |
| **room.type** | dm | group | space | project | portfolio | deal | community | event | governance | ai | support |
| **room.name** | Display name |
| **room.visibility** | public | private | protected |
| **room.members\[\]** | Member roster with roles: owner | moderator | member | readonly | bot |
| **room.messages\[\]** | Ordered message log (paginated, event-sourced) |
| **room.pinned\_messages\[\]** | Pinned high-priority messages |
| **room.linked\_entity** | ID of linked PortfolioComponent, Event, Exchange listing, etc. |
| **room.settings** | Notification preferences, archival policy, retention rules |
| **room.bots\[\]** | AI agents or automation bots attached to the room |
| **room.status** | active | archived | paused | read-only |
| **room.metadata** | Full component metadata: id, owners, created\_at, vector\_clock, version |

## **3.4 Message Data Model**

| message.id | UUID |
| :---- | :---- |
| **message.room\_id** | Parent room reference |
| **message.author\_id** | Sending user or bot ID |
| **message.type** | text | file | media | link | system | reaction | poll | call | embed |
| **message.content** | Rendered content body (markdown-compatible) |
| **message.attachments\[\]** | Files, images, documents, portfolio component embeds |
| **message.reactions\[\]** | Emoji reactions with user ID lists |
| **message.thread\_id** | Parent message ID for threaded replies |
| **message.mentions\[\]** | Mentioned user or component IDs |
| **message.status** | sent | delivered | read | failed |
| **message.edited\_at** | Timestamp of last edit |
| **message.metadata** | Created\_at, vector\_clock, policy\_ids |

## **3.5 Real-Time Architecture**

Rooms use WebSocket connections for real-time delivery, backed by Redis pub/sub for message fanout and Kafka for durable event streaming and replay:

* Client connects to KRMS service via WebSocket on room join

* Messages published to Redis channel for instant delivery to online subscribers

* All messages also written to Kafka topic for durability, analytics, and replay

* Presence (online/typing indicators) managed via Redis TTL keys

* Offline users receive messages on reconnect via PostgreSQL message log

* AI agent (Oba) subscribes to designated AI Room channels and responds via the Sambara agent system

| 4\. KFED — FEED & SOCIAL LAYER |
| :---: |

## **4.1 Feed Architecture**

The Feed Service (KFED) powers all activity streams across the Kogi platform. Unlike a single monolithic social feed, Kogi has a layered feed architecture where feeds are scoped to the entity level — personal, space, portfolio, event, and global feeds — all unified through the personalization and recommendation engines.

## **4.2 Feed Types**

| Feed Scope | Content Sources | Personalization |
| :---- | :---- | :---- |
| Personal Home Feed | Followed users, spaces, portfolios, tags, subscriptions | Full RecommendationEngine scoring |
| Space Feed | Posts within a specific Space from members | Space-scoped ranking, pinned posts |
| Portfolio Feed | Updates, milestones, changes for a portfolio or item | Chronological \+ relevance |
| Event Feed | Discussions, posts, updates tied to an Event | Chronological |
| Organization Feed | Governance updates, announcements, member activity | Role-filtered visibility |
| Marketplace Feed | New listings, deals, offers, campaigns | Match scored by MatchEngine |
| Explore / Discover Feed | Global trending content, recommended Spaces, creators | Full AI personalization |
| Notification Feed | System alerts, mentions, replies, action required | Priority-ranked by urgency |

## **4.3 Post Data Model**

| post.id | UUID |
| :---- | :---- |
| **post.author\_id** | User or organization ID |
| **post.type** | post | update | milestone | announcement | showcase | article | poll | event | campaign | repost |
| **post.content** | Richtext body (markdown \+ embeds) |
| **post.attachments\[\]** | Media, files, portfolio component cards, links |
| **post.visibility** | public | followers | space | private | custom |
| **post.tags\[\]** | Hashtags and topic labels |
| **post.mentions\[\]** | Mentioned user, space, or component IDs |
| **post.linked\_component** | Optional linked PortfolioComponent (project milestone, asset release, etc.) |
| **post.analytics** | Impressions, reach, engagements, CTR, view time, sentiment score |
| **post.reactions** | likes, applause, support, insightful — with user ID lists |
| **post.comments\[\]** | Nested comment thread (comment has same structure as post, scoped) |
| **post.shares** | Repost and share count |
| **post.saves** | Bookmark count and user ID list |
| **post.status** | draft | scheduled | published | archived | removed |
| **post.metadata** | id · created\_at · updated\_at · vector\_clock · version · policy\_ids |

## **4.4 Social Interaction Primitives**

The following primitives are available on Posts, Spaces, PortfolioComponents, and User Profiles:

| Primitive | Target Entities | Engine Impact |
| :---- | :---- | :---- |
| like / react | Post, Comment, Component | Engagement signal → AnalyticsEngine |
| comment | Post, Component, Event | Depth engagement → RecommendationEngine |
| share / repost | Post, Showcase item | Virality signal → TelemetryEngine |
| save / bookmark | Post, Component, Space | Long-term interest signal → PersonalizationEngine |
| follow | User, Space, Tag, Portfolio | Feed subscription and social graph edge |
| subscribe | Space, Portfolio, Creator | Paid or privileged follow tier |
| watch | Component, Project, Portfolio | Change notification subscription |
| tag / mention | Post, Room message | Graph relationship \+ notification trigger |
| hashtag / topic | Post, Component | Topic graph membership, trending signal |
| poll / survey | Space, Post | Governance and engagement data collection |
| invite | Space, Room, Event | Membership and participation management |
| donate | Component, Campaign, Org | Capital flow → kogi-bank |
| invest | Component, Org, Campaign | Equity/stake flow → kogi-bank / kogi-exchange |
| contribute | Project, Program, Campaign | Resource contribution tracking |
| campaign | Space, Component, Org | Fundraising → kogi-bank |
| report | Post, User, Space | Moderation queue entry |

| 5\. KORG — ORGANIZATIONS, COLLECTIVES & COOPERATIVES |
| :---: |

## **5.1 Concept**

The Organizations system (KORG) is the governance and coordination layer for groups of users who wish to operate collectively. It encompasses the full spectrum from loose informal collectives to formally governed autonomous organizations and cooperative federations.

Organizations are first-class citizens on the Kogi platform — they have their own profiles, portfolios, wallets, spaces, and feeds. An organization's portfolio can contain projects, programs, assets, and financial instruments, all managed with multi-member governance rules.

## **5.2 Organization Types**

| Type | Description | Governance Model |
| :---- | :---- | :---- |
| Collective | Informal resource and skill sharing group; no formal legal entity required | Consensus / flat |
| Cooperative | Member-owned economic entity with shared profits and voting rights | One member, one vote |
| Autonomous Organization | Rule-governed org with codified bylaws and automated governance | Proposal \+ quorum vote |
| DAO | Decentralized autonomous org; optionally on-chain governance | Token-weighted or 1M1V |
| Team | Operational working group within an org or space | Hierarchical / lead-managed |
| Guild | Skill-based professional association | Merit / reputation weighted |
| Federation | Network of linked orgs, collectives, cooperatives | Federal / representative |
| Foundation | Non-profit oriented entity; grants, donations | Board / trustee governed |
| Investment Club | Group investing and portfolio management entity | Proportional stake voting |

## **5.3 Organization Data Model**

| org.id | UUID — platform-unique identifier |
| :---- | :---- |
| **org.type** | collective | cooperative | dao | autonomous | team | guild | federation | foundation | club |
| **org.name** | Display name |
| **org.handle** | @handle for discovery and mention |
| **org.legal\_entity** | Optional linked legal entity (LLC, Corp, Trust, Fund, etc.) |
| **org.members\[\]** | Member roster: role, stake, joined\_at, status, permissions |
| **org.roles\[\]** | Custom roles with privilege tiers: founder | admin | governor | treasurer | member | contributor | observer |
| **org.portfolio\_id** | Linked organizational portfolio |
| **org.wallet\_id** | Linked organizational wallet |
| **org.space\_id** | Primary Space for the organization |
| **org.governance** | Governance model config: voting\_type, quorum\_threshold, proposal\_rules |
| **org.proposals\[\]** | Active and historical governance proposals |
| **org.policies\[\]** | Access, financial, and governance policy IDs |
| **org.metadata** | Full PortfolioComponent metadata: id, owners, created\_at, vector\_clock, version |

## **5.4 Governance System**

The governance system allows organizations to run structured decision-making processes:

| Component | Description |
| :---- | :---- |
| Proposal | A motion to take action; has title, body, linked action, voting window, and required quorum |
| Vote | Member vote on a proposal: approve | reject | abstain; weighted by stake or 1-member-1-vote |
| Quorum | Minimum participation threshold for a vote to be valid; configurable per org type |
| Resolution | Outcome of a passed proposal; may trigger automated actions via Oba agent |
| Veto | Founder or governance-tier override mechanism for emergency governance |
| Amendment | Proposal to modify org bylaws, policies, or governance rules |
| Treasury Proposal | Financial action requiring approval: budget allocation, investment, distribution |
| Membership Proposal | Adding, removing, or changing the role of a member |

## **5.5 AI Governance Assistant**

The Oba AI assistant handles the full governance lifecycle for cooperative and DAO members:

* Drafts proposals from natural-language intent (e.g. 'propose we allocate $5K to marketing')

* Monitors quorum status and sends member notifications at configured intervals

* Summarizes arguments and comments from the governance room for absent members

* Executes passed resolutions automatically where linked actions are configured

* Flags unresolved or expiring proposals in the member dashboard briefing

* Provides governance health analytics: participation rates, vote turnout, resolution velocity

| 6\. SOCIAL GRAPH & RELATIONSHIP MODEL |
| :---: |

## **6.1 Graph Design**

The Kogi social graph is a directed, typed property graph. Nodes represent users, spaces, organizations, portfolio components, events, and posts. Edges represent typed relationships with properties such as weight, timestamp, visibility, and metadata.

The GraphEngine (Scala) provides graph traversal, critical path analysis, dependency closure, and social graph analytics.

## **6.2 Node Types**

| Node Type | Description |
| :---- | :---- |
| User | Individual platform user; has profile, personas, skills, contact |
| Space | Community space or organization environment |
| Organization | Formal or informal collective entity |
| PortfolioComponent | Portfolio, program, project, resource, asset, artifact |
| Post | Content item published to a feed |
| Event | Scheduled gathering, meeting, or conference |
| Tag / Topic | Hashtag or topic label node connecting related content |
| Campaign | Fundraising or resource-gathering initiative |

## **6.3 Edge Types (Relationship Primitives)**

| Edge | From → To | Properties |
| :---- | :---- | :---- |
| follows | User → User | Space | Tag | Portfolio | since, notification\_level |
| subscribes | User → Space | Portfolio | Creator | tier, since, auto\_renew |
| watches | User → PortfolioComponent | since, alert\_level |
| member\_of | User → Space | Organization | role, joined\_at, stake, status |
| owns | User → PortfolioComponent | Space | Org | privilege\_tier, since |
| collaborates\_on | User → PortfolioComponent | role, contribution\_type |
| invested\_in | User | Org → PortfolioComponent | Campaign | amount, instrument, date |
| linked\_to | PortfolioComponent → PortfolioComponent | link\_type, weight |
| posted\_in | Post → Space | Feed | visibility, pinned |
| tagged\_with | Post | Component → Tag | weight, source |
| part\_of | Space → Federation | Org → Federation | federate\_since, role |

| 7\. ANALYTICS & ENGAGEMENT ENGINE |
| :---: |

## **7.1 Community Analytics Overview**

All social interactions on the Kogi platform generate telemetry events that flow through the KogiEngine's AnalyticsEngine, TelemetryEngine, and RecommendationEngine. The Community module contributes the richest behavioral and engagement signal set on the platform.

## **7.2 Engagement Metrics**

| Metric Category | Metrics | Engine |
| :---- | :---- | :---- |
| Engagement | Likes, reactions, comments, shares, saves, poll responses, engagement rate (engagements / reach) | AnalyticsEngine |
| Content Performance | Impressions, reach, CTR, video completion rate, hashtag performance, best post time | AnalyticsEngine |
| Audience | Demographics, interests, follower growth rate, member growth rate, churn rate | PersonalizationEngine |
| Sentiment | Positive / neutral / negative categorization of mentions, posts, and comments | AnalyticsEngine |
| Space Health | Member activity rate, post frequency, room message volume, governance participation | RiskEngine |
| Influence | Share of voice, brand mentions, collaboration network centrality score | GraphEngine |
| Discovery | Search impressions, explore appearances, recommendation clicks, match acceptance rate | RecommendationEngine · MatchEngine |
| Paid / Campaign | Campaign reach, contribution rate, cost per contribution, ROAS for promoted posts | AnalyticsEngine |
| Behavioral | Time spent per space/room, scroll depth, session paths from post to portfolio item | TelemetryEngine |

## **7.3 Personalization & Recommendation**

The PersonalizationEngine builds dynamic user profiles from community behavior to drive feed ranking and Space recommendations:

* Collaborative filtering — surfaces content liked by users with similar engagement patterns

* Content-based filtering — surfaces posts and spaces similar to those a user previously engaged with

* Hybrid scoring — combines both methods with recency, diversity, and cold-start handling

* Persona signals — user personas (professional, investor, creator, etc.) modulate scoring weights

* Contextual adaptation — time of day, device, current module, active project context all adjust feed ranking

* Feedback loop — click-through, dwell time, reaction, and ignore signals continuously retrain the model

## **7.4 Match Engine — Community Application**

The MatchEngine applies to community interactions in the following ways:

* Worker-to-space matching: recommends Spaces based on skill profile, project history, and interest graph

* Collaborator matching: 'users who engaged with similar work to yours, who might want to collaborate'

* Investor-to-campaign matching: connects fundraising campaigns to users with matching investment interests

* Contributor-to-project matching: surfaces open project roles to community members with relevant skills

* Org-to-member matching: helps organizations find qualified members or specialists

| 8\. AI WORKFLOWS — COMMUNITY LAYER |
| :---: |

## **8.1 Oba AI Assistant in Community**

The Oba AI assistant (powered by the Sambara agent system) provides intelligent community management, content creation, and collaboration facilitation across all community surfaces.

## **8.2 Content Amplification Workflow**

| *When a user completes a portfolio milestone, Oba automatically drafts a community showcase post, suggests relevant spaces, identifies collaboration opportunities, and recommends optimal posting time — all awaiting one-click user approval.* |
| :---- |

| Capability | Description |
| :---- | :---- |
| Post drafting | Generates compelling post from portfolio milestone or update with tone, tags, and CTA |
| Showcase optimization | Reviews portfolio item descriptions and tags for discoverability improvements |
| Collaboration matching | Identifies community members whose skills complement the user's active projects |
| Engagement analytics | Reports which content types and times drive highest engagement for this user |
| Space recommendations | Suggests spaces the user should join based on active project types |
| DM triage | Categorizes incoming DMs as collaboration, client inquiry, or general; drafts reply suggestions |
| Hashtag intelligence | Analyzes trending tags and recommends which to include for maximum reach |
| Cross-platform scheduling | Plans post timing and coordinates with linked social media provider integrations |

## **8.3 Governance AI Workflow**

| Capability | Description |
| :---- | :---- |
| Proposal drafting | Converts natural language intent into structured governance proposals with linked actions |
| Quorum monitoring | Tracks vote participation and sends reminders at configurable thresholds |
| Argument summary | Summarizes room discussion and comments for members who missed debate |
| Resolution execution | Auto-executes passed proposals: budget allocations, role changes, policy updates |
| Proposal expiry alerts | Flags expiring or unresolved proposals in morning briefing |
| Governance health report | Weekly participation rates, vote turnout, resolution velocity, trend analysis |

## **8.4 Community Growth Workflow**

| Trigger | AI Action |
| :---- | :---- |
| New portfolio item published | Suggest relevant spaces to share in; draft announcement post |
| Project milestone reached | Generate milestone card; identify collaborators for celebration or expansion |
| Campaign launched | Draft campaign announcement; identify matching investor profiles to notify |
| New space member joins | Welcome message; suggest rooms to join; surface relevant portfolio showcases |
| Engagement spike detected | Alert user; surface collaboration requests; suggest follow-up content |
| Collaboration request received | Triage, summarize, draft suggested response for user approval |
| Governance quorum missed | Re-notify members; extend window; suggest simplified proposal if appropriate |

| 9\. SERVICE ARCHITECTURE |
| :---: |

## **9.1 Backend Services**

| Service | Language | Responsibilities |
| :---- | :---- | :---- |
| community-service (Go) | Go | Space CRUD, member management, governance actions, event management |
| post-service (Go) | Go | Post create/read/update/delete, reaction handling, comment management |
| feed-service (Go) | Go | Feed assembly, fanout-on-write, feed ranking, notification push |
| social-graph-service (Go) | Go | Follow/subscribe/watch relationship management, graph queries |
| chat-service (Go) | Go | Room management, message delivery, presence, room membership |
| message-service (Go) | Go | Message CRUD, threading, attachment management, search |
| event-service (Go) | Go | Event planning, RSVP, event feed, event rooms |
| notification-service (Go) | Go | Alert fanout, notification preferences, DLQ management |
| community-matching-engine (Scala) | Scala | Collaborator matching, space recommendations, talent discovery |
| community-analytics (Scala) | Scala | Engagement analytics, sentiment analysis, community health scoring |

## **9.2 Data Storage**

| Store | Usage |
| :---- | :---- |
| PostgreSQL | Durable storage: spaces, orgs, members, posts, messages, events, social graph edges |
| Redis | Real-time: presence, pub/sub fanout, session cache, feed hot data, room membership |
| Kafka | Event streaming: all social events, feed events, analytics pipeline input |
| SQLite | Local/desktop client cache: offline message queue, local feed snapshot |
| S3 / MinIO | Media storage: post attachments, profile images, event media |

## **9.3 API Endpoints**

All community APIs follow the standard Kogi REST pattern at https://api.kogi.io/v1/{service}/{resource}:

| Endpoint Group | Key Endpoints |
| :---- | :---- |
| Spaces | GET /kspc/spaces · POST /kspc/spaces · GET /kspc/spaces/{id} · POST /kspc/spaces/{id}/join · POST /kspc/spaces/{id}/rooms · GET /kspc/spaces/{id}/feed |
| Rooms | GET /krms/rooms · POST /krms/rooms · GET /krms/rooms/{id}/messages · POST /krms/rooms/{id}/messages · WS /krms/rooms/{id}/stream |
| Feed | GET /kfed/feed · GET /kfed/feed/explore · GET /kfed/feed/notifications · POST /kfed/posts · GET /kfed/posts/{id} |
| Social Graph | POST /ksgr/follow · DELETE /ksgr/follow · POST /ksgr/subscribe · GET /ksgr/followers/{user\_id} · GET /ksgr/following/{user\_id} |
| Organizations | GET /korg/orgs · POST /korg/orgs · GET /korg/orgs/{id} · POST /korg/orgs/{id}/proposals · POST /korg/orgs/{id}/vote |
| Events | GET /kspc/events · POST /kspc/events · GET /kspc/events/{id} · POST /kspc/events/{id}/rsvp |
| Search | GET /ksrc/spaces · GET /ksrc/posts · GET /ksrc/users · GET /ksrc/orgs · GET /ksrc/explore |

| 10\. THIRD-PARTY INTEGRATIONS |
| :---: |

## **10.1 Social Platform Integrations**

The Kogi ProviderSystem manages all third-party platform connections. Users can link external social media accounts and platform credentials to their Kogi profile, enabling cross-platform publishing, reach amplification, and communication bridging.

| Provider | Integration Type | Capabilities |
| :---- | :---- | :---- |
| Facebook / Instagram | OAuth2 \+ API | Cross-post content, import followers, sync events |
| YouTube | OAuth2 \+ API | Embed video, sync channel activity, cross-publish |
| WhatsApp | Business API | Broadcast messages, customer communication bridge |
| Slack | OAuth2 \+ Webhooks | Cross-team notifications, channel mirroring, DM bridge |
| Zoom | OAuth2 \+ API | Event room video calls, meeting scheduling integration |
| Twitter / X | OAuth2 \+ API | Cross-post announcements, track mentions, import audience |
| LinkedIn | OAuth2 \+ API | Professional profile sync, job post distribution, network import |
| Discord | Bot API | Community server bridging, role sync, notification relay |
| OnlyFans / Patreon | OAuth2 \+ Webhooks | Subscription tier sync, subscriber import, payout integration |

## **10.2 Communication & Collaboration Integrations**

| Provider | Integration Type | Capabilities |
| :---- | :---- | :---- |
| Gmail | OAuth2 \+ IMAP | Email-to-room bridge, DM-to-email relay, campaign outreach |
| Google Meet | OAuth2 \+ API | Event rooms linked to Meet calls, calendar integration |
| Microsoft Teams | OAuth2 \+ Webhooks | Team channel bridging, notification relay |
| Twilio | API | SMS notification relay, 2FA, event reminders |

## **10.3 Provider System Architecture**

All integrations are managed through the kogi-providers module:

* Provider Registry — central registry of all available third-party providers

* ProviderSystem — manages credentials, OAuth tokens, webhook endpoints, and API rate limits

* Affiliate System — manages affiliate links, discount codes, commission tracking, and payouts

* Tool Integration — providers can be linked as tools in a user's toolbox for workflow automation

* Version Control — provider API version tracking and graceful degradation on version changes

| 11\. SECURITY, PRIVACY & ACCESS CONTROL |
| :---: |

## **11.1 Access Control Model**

The community layer inherits the Kogi platform RBAC model and extends it with community-specific roles:

| Role | Scope | Privileges |
| :---- | :---- | :---- |
| Platform Admin | Global | Full moderation, content removal, ban, platform config |
| Space Owner | Space | Manage space config, members, rooms, events, policies |
| Space Admin | Space | Manage members and rooms, moderate content |
| Space Moderator | Space | Moderate content, manage reports, mute members |
| Space Member | Space | Post, comment, react, join rooms, create events |
| Space Contributor | Space | Post and comment; limited role in governance |
| Space Viewer | Space | Read-only; no posting rights |
| Org Governor | Organization | Propose and vote on governance; manage treasury |
| Org Treasurer | Organization | Manage organizational wallet and financial proposals |
| Org Member | Organization | Vote, comment, contribute; standard membership rights |

## **11.2 Content Visibility Model**

| Visibility Level | Description | Who Can See |
| :---- | :---- | :---- |
| public | Visible to all users and unauthenticated visitors | Anyone |
| followers | Visible to followers and subscribers | Followers, subscribers |
| protected | Visible to approved followers | Approved followers |
| space | Visible to Space members only | Space members |
| organization | Visible to org members only | Org members with appropriate role |
| private | Visible only to explicitly mentioned parties | Tagged users, owner |
| custom | Rule-based visibility defined by owner | Per-policy access list |

## **11.3 Data Privacy & Compliance**

* GDPR right-to-erasure: complete post, message, and social graph data deletion on request

* Data minimization: only social interaction data required for function is collected

* Consent management: granular consent controls per data type and integration

* Network namespace isolation: community services operate in isolated network segments

* Encryption at rest and in transit: TLS 1.3 for all connections; AES-256 for stored message content

* Audit logging: all moderation, governance, and administrative actions logged to immutable audit trail

| 12\. MVP FEATURE SCOPE |
| :---: |

## **12.1 MVP Inclusion**

| Feature | Module | MVP Priority |
| :---- | :---- | :---- |
| Space creation and management | KSPC | P0 — Core |
| Space member management (join, invite, roles) | KSPC | P0 — Core |
| Direct messaging (1:1 rooms) | KRMS | P0 — Core |
| Group rooms | KRMS | P0 — Core |
| Post creation and feed | KFED | P0 — Core |
| Follow / unfollow users and spaces | KSGR | P0 — Core |
| Like, comment, share on posts | KFED | P0 — Core |
| Notifications (mentions, replies, alerts) | KFED | P0 — Core |
| Basic organization creation (collective, team) | KORG | P1 — High |
| Activity feed per space | KFED | P1 — High |
| Event creation and RSVP | KSPC | P1 — High |
| Portfolio showcase posts | KFED | P1 — High |
| Space discovery / explore | KMTC | P1 — High |
| Governance proposals and voting | KORG | P2 — Medium |
| DAO / cooperative governance model | KORG | P2 — Medium |
| AI post drafting (Oba) | AI Layer | P2 — Medium |
| Collaboration matching | KMTC | P2 — Medium |
| Third-party social platform integration | Provider | P3 — Later |
| Advanced analytics dashboard | KENG | P3 — Later |
| Federation of spaces and orgs | KSPC/KORG | P3 — Later |

| 13\. INTEGRATION WITH KOGI PLATFORM MODULES |
| :---: |

## **13.1 Module Integration Map**

| Module | Integration with Community | Data Flow Direction |
| :---- | :---- | :---- |
| kogi-office | Portfolio items shared as showcase posts; project updates published to space feed; project rooms linked to portfolio items | Office → Community (publish) |
| kogi-bank | Fundraising campaigns appear in community feeds; donation and investment actions from community posts; org treasury linked to org wallet | Bi-directional |
| kogi-exchange | Deal rooms for marketplace negotiations; exchange listings shared in community spaces; matchmaking uses community social graph | Exchange → Community (publish) · Community → Exchange (match) |
| kogi-marketplace | Marketplace listings shared to relevant spaces; talent discovery uses space membership data; reviews and ratings posted to community | Bi-directional |
| kogi-studio | Design assets published to community showcase; creative ideas shared to relevant spaces; collaboration requests routed from studio | Studio → Community (publish) |
| kogi-profile | User profiles source community display info, skill data for matching; personas determine feed personalization and posting context | Profile → Community (read) |
| kogi-engine | All community events streamed to engine; feed ranking, match scoring, recommendation, analytics all powered by engine | Community → Engine (stream) · Engine → Community (score) |
| kogi-home | Dashboard aggregates community notifications, mentions, space activity, DM summaries | Community → Home (push) |

| 14\. EVENTS SYSTEM |
| :---: |

## **14.1 Event Types**

Events are a first-class entity in the Kogi Community system — they can be created within a Space, by an Organization, or as standalone portfolio-linked milestones:

| Event Type | Description |
| :---- | :---- |
| Meeting | Scheduled video or audio call; integrates with Zoom, Google Meet |
| Conference | Multi-session structured gathering; has a schedule, multiple event rooms |
| Workshop | Interactive skill-building session with materials and exercises |
| Mastermind | Peer-to-peer knowledge sharing group session |
| AMA (Ask Me Anything) | Open Q\&A session hosted by a user or organization |
| Launch | Product, project, or portfolio launch event |
| Gathering | Informal community social event |
| Tagup | Quick synchronous check-in; typically short duration |
| Demo Day | Showcase of projects, products, or portfolio items |
| Governance Vote | Formal voting event for an organization; tied to a Proposal |

## **14.2 Event Data Model**

| event.id | UUID |
| :---- | :---- |
| **event.type** | meeting | conference | workshop | mastermind | ama | launch | gathering | tagup | demo | vote |
| **event.host\_id** | User or Organization ID |
| **event.space\_id** | Parent Space (if applicable) |
| **event.title** | Event display name |
| **event.description** | Rich text description |
| **event.schedule** | start\_time, end\_time, timezone, recurrence\_rule |
| **event.capacity** | Max participant count; null \= unlimited |
| **event.participants\[\]** | RSVP roster: user\_id, status (going | maybe | not\_going), role |
| **event.rooms\[\]** | Linked rooms: main, breakout, hallway, governance |
| **event.visibility** | public | space | invite | private |
| **event.portfolio\_link** | Optional linked portfolio component (e.g. project milestone) |
| **event.recording** | Optional media attachment for recorded events |
| **event.metadata** | Full PortfolioComponent metadata |

| APPENDIX — TERMINOLOGY REFERENCE |
| :---: |

| Term | Definition |
| :---- | :---- |
| Space (KSPC) | A named digital community environment; top-level social container |
| Room (KRMS) | A real-time or async communication channel; exists within or independent of a Space |
| Feed (KFED) | An activity stream; scoped to user, space, portfolio, org, or global explore |
| Post | A content item published to a feed; has type, content, attachments, and analytics |
| Social Graph | The directed typed property graph of all relationships between users, spaces, components |
| PortfolioComponent | The base type for all portfolio objects: portfolios, items (projects, programs, assets, artifacts, resources), and containers (binders, books, folders, records) |
| Organization (KORG) | A governed group entity: collective, cooperative, autonomous org, DAO, team, guild, federation |
| Governance Proposal | A formal motion within an organization; goes through draft → active → resolved lifecycle |
| Oba | The Kogi platform AI assistant; runs on the Sambara agent system and ImaniOS |
| MatchEngine | Engine subsystem responsible for user-to-space, worker-to-opportunity, and resource-to-need matching |
| PersonalizationEngine | Engine subsystem that builds user profiles and ranks content for personalized feed delivery |
| Federation | A network of linked Spaces, Organizations, or platform nodes coordinating as a unified system |
| jiwe seed | A single boot configuration artifact that can configure the full ShangoOS platform stack |
| ShangoOS | The combined operating system formed by kogi \+ qala \+ ume \+ oru \+ sambara \+ imewe \+ nandi \+ osyse |

© 2026 Kogi Platform — Confidential Design Document