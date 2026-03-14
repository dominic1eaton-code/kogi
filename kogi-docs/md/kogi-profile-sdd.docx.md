  
**KOGI**

Independent Worker Operating System

| kogi-profile  ·  Profile Management & LinkNetwork System |
| :---: |

**System Design Document**

KPRF · KNET · KCON · KPRS · KACC · KVIS · KDIR

v1.0  ·  Kogi Platform  ·  March 2026

| Module | Subsystem |
| :---- | :---- |
| **KPRF** | Profile Management — core profile data model, profile types, lifecycle |
| **KNET** | LinkNetwork — linktrees, linkforests, mycorrhizal network substrate |
| **KCON** | ContactBook & Directory — structured contact management and directories |
| **KPRS** | Persona System — multi-persona profiles, persona switching, contexts |
| **KACC** | Linked Accounts — external platform account linking and management |
| **KVIS** | Visibility & Access Control — tiered permissions, RBAC, policy enforcement |
| **KDIR** | Directory & Discovery — profile search, indexing, ranking, discovery |

# **1\.  System Overview**

| *Design Philosophy: A profile is not a page — it is a sovereign digital identity. Every profile on the Kogi platform is a first-class PortfolioComponent with full metadata, CRDT versioning, access control, analytics, and lifecycle management. The profile is the atom from which all social, economic, and collaborative relationships radiate.* |
| :---- |

The Kogi Profile Management System (kogi-profile) provides every independent worker with a flexible, multi-layered identity infrastructure. Users can maintain multiple named profiles of different types simultaneously — work, professional, personal, business, public, private, and custom — each carrying its own personas, linked accounts, communication channels, permissions, and portfolio associations.

Layered on top of individual profiles, the kogi-net LinkNetwork System forms a mycorrhizal substrate connecting users' linktrees into a searchable, navigable, policy-controlled linkforest across the entire platform — consolidating the fragmented landscape of modern digital identity into a unified, centrallymanaged network.

## **1.1  Scope**

| Subsystem | Scope |
| :---- | :---- |
| KPRF — Profile Management | Profile types, data model, creation, editing, switching, lifecycle, configuration |
| KPRS — Persona System | Multi-persona attachments, context switching, persona-aware feed and recommendations |
| KACC — Linked Accounts | External platform account linking, OAuth flows, account trees, sync management |
| KVIS — Visibility & Access Control | Tiered permissions, role-based access, profile privacy zones, audience segmentation |
| KNET — LinkNetwork | Linktree construction, linkforest formation, mycorrhizal network substrate, kogi-net |
| KCON — ContactBook & Directory | Contact import/export, contact directories, linkforest-derived contact books |
| KDIR — Discovery & Search | Profile indexing, search, ranking, discoverability settings, MatchEngine integration |

## **1.2  Relationship to the Platform**

| Direction | Module | Data Flow |
| :---- | :---- | :---- |
| Profile → Office | kogi-office (portfolios, projects) | Profile links to owned portfolios; work profiles expose project lists |
| Profile → Bank | kogi-bank (accounts, benefits) | Profiles hold portable benefits accounts; payment identity linked to profile |
| Profile → Community | kogi-community (social graph) | Profile is node in KSGR; follow/subscribe/connect acts on profiles |
| Profile → Marketplace | kogi-marketplace (talent, gigs) | Worker profiles indexed for talent discovery and gig matching |
| Profile → Exchange | kogi-exchange (deals, matching) | Profile reputation and skills feed MatchEngine for opportunity scoring |
| Bi-directional | kogi-engine | PersonalizationEngine builds persona models; RecommendationEngine scores opportunities per profile |
| Foundation | PortfolioSystem (Rust) | All profiles are PortfolioComponents; metadata, CRDT, EventLog inherited |

# **2\.  Profile Data Model  (KPRF)**

Every profile is a PortfolioComponent of ItemType::Profile. All profiles inherit the standard component metadata, CRDT vector clock, EventLog, policy system, and lifecycle states of the platform. Profile-specific data extends this base with identity, configuration, persona, account, and channel layers.

## **2.1  Profile Schema**

### **2.1.1  ProfileComponent (top-level)**

| Field | Type / Description |
| :---- | :---- |
| id | UUID — globally unique profile identifier |
| owner\_user\_id | UUID — the user who owns this profile |
| profile\_type | ProfileType enum (see §2.2) |
| name | Display name for this profile |
| slug | URL-safe short identifier (e.g. @alex-dev or @alex.studio) |
| status | ProfileStatus: Active | Inactive | Suspended | Archived |
| visibility | VisibilityLevel: Public | Protected | Private | Custom |
| created\_at | Timestamp |
| updated\_at | Timestamp |
| vector\_clock | VectorClock — CRDT distributed sync |
| version | Semantic version of this profile snapshot |
| policy\_ids | Vec\<UUID\> — governing policies attached to this profile |
| metadata | ProfileMetadata block (see §2.1.2) |
| identity | ProfileIdentity block (see §2.1.3) |
| personas | Vec\<Persona\> — attached personas (see §3) |
| linked\_accounts | Vec\<LinkedAccount\> — external accounts (see §4) |
| channels | CommunicationChannels — contact \+ channel config (see §5) |
| permissions | PermissionConfig — access control rules (see §6) |
| portfolios | Vec\<PortfolioRef\> — associated portfolios and programs |
| organizations | Vec\<OrgRef\> — linked organizations, collectives, cooperatives |
| preferences | ProfilePreferences — UI, notification, feed, privacy prefs |
| linktree | LinkTree — this profile's linktree node in kogi-net (see §7) |
| analytics | ProfileAnalytics — computed engagement and reach signals |
| tags | Vec\<String\> — searchable tags and topics |
| event\_log | EventLog — append-only audit of all mutations |

### **2.1.2  ProfileMetadata**

| Field | Description |
| :---- | :---- |
| avatar\_url | Profile image URL |
| banner\_url | Banner/cover image URL |
| tagline | Short bio (max 160 chars) |
| bio | Long-form description (markdown supported) |
| location | City, region, or custom location text |
| timezone | IANA timezone string |
| language | Primary language code (ISO 639-1) |
| website\_url | Primary website (displayed on profile card) |
| theme | UI theme override for this profile |
| badges | Vec\<Badge\> — platform-awarded and verified badges |
| skills | Vec\<Skill\> — declared skills with proficiency levels |
| certifications | Vec\<Certification\> — verified credentials |
| portfolio\_highlight | Option\<PortfolioRef\> — pinned portfolio shown on profile |
| custom\_fields | Map\<String, Value\> — user-defined key-value metadata |

### **2.1.3  ProfileIdentity**

| Field | Description |
| :---- | :---- |
| legal\_name | Optional — verified legal name (private by default) |
| display\_name | Public-facing name for this profile |
| pronouns | Optional pronouns display string |
| verified | Bool — platform identity verification status |
| verification\_type | None | Email | Phone | ID | Professional | Organization |
| reputation\_score | Float — computed by RiskEngine from activity, reviews, deliveries |
| trust\_level | TrustLevel: New | Established | Trusted | Verified | Elite |
| worker\_type | WorkerType: Freelancer | Contractor | Consultant | Gig | Entrepreneur | ... |
| industries | Vec\<Industry\> — associated industry categories |
| years\_experience | Optional integer |

## **2.2  Profile Types**

A single user account can hold an unlimited number of profiles. Each profile is independently configured, has its own visibility settings, persona set, linked accounts, and portfolio associations. Profiles can be switched contextually — the active profile governs what the user presents to the world at any given moment.

| Profile Type | Description | Typical Use |
| :---- | :---- | :---- |
| Personal | The user's primary personal identity on the platform. One per account by default. | Personal brand, general networking, community participation |
| Work | A context-specific professional identity tied to a job, contract engagement, cooperative membership, or client relationship. | Cooperative work, freelance engagements, employment, project-specific identity |
| Professional | A curated, credential-forward profile emphasizing skills, certifications, and portfolio highlights. Typically public-facing. | LinkedIn-equivalent public professional identity, client acquisition |
| Business | An organizational profile for a solo business, LLC, studio, or brand entity the user operates. | Client-facing business identity, brand portfolio showcase |
| Public | Maximally visible profile optimized for discoverability, showcasing, and broadcasting. Indexed by the SearchEngine. | Creator profiles, public portfolios, community leadership |
| Private | Fully private profile visible only to the owner and explicitly granted viewers. Not indexed. | Personal notes identity, private project tracking, off-record communications |
| Protected | Visible to approved followers and explicit connections only. Discoverable but gated. | Semi-private professional networks, closed community participation |
| Template | A reusable profile scaffolding that can be instantiated into a new profile. Carries default personas, settings, and linked account patterns. | Rapid profile creation from org-defined templates |
| Custom | Fully user-defined profile type with custom label, icon, and behavior configuration. | Non-standard use cases: anonymous contributor, research alias, community role |
| Miscellaneous | Catch-all type for profiles that do not fit other categories. Minimal default configuration. | Experimental, transitional, or testing profiles |

## **2.3  Profile States & Lifecycle**

| State | Description |
| :---- | :---- |
| Draft | Profile created but not yet activated. Invisible to all but owner. |
| Active | Live profile. Visibility rules apply. Indexed if visibility permits. |
| Inactive | Temporarily deactivated. Hidden from discovery. All associations preserved. |
| Suspended | Platform-suspended profile. Restricted actions. Under review. |
| Archived | Soft-deleted. Data preserved for restore. Removed from all indexes. |

## **2.4  Multi-Profile User Architecture**

| *Example: A user operates two Work profiles and one Public profile simultaneously. Work Profile A — linked to a software cooperative: GitLab, Jira, Claude, YouTube, Google accounts; software development \+ podcast production projects; linked to the cooperative's public portfolio. Work Profile B — linked to an early-stage startup: GitHub, ChatGPT, Notion, Facebook, Yahoo accounts; social media app project; linked to an open-source community collective. Public Profile — their creator/thought-leader face: Twitter/X, LinkedIn, Substack, YouTube; public portfolio; speaking engagements; writing portfolio.* |
| :---- |

| Concept | Detail |
| :---- | :---- |
| Active Profile | The currently selected profile context. All actions, posts, and messages originate from the active profile unless overridden. |
| Profile Switcher | Quick-access UI element (persistent in nav bar) showing all active profiles for instant context switching. |
| Default Profile | Designated fallback profile for platform interactions when no specific profile is selected. |
| Profile Isolation | Each profile has an independent feed, notification stack, inbox, and analytics. Cross-profile bleed is governed by user policy. |
| Shared Resources | Bank accounts, portable benefits accounts, and core identity (legal name, verification) exist at the User level and are shared across profiles. |
| Profile Groups | Profiles can be grouped (e.g., 'All Work Profiles', 'Creative Profiles') for bulk management and aggregate analytics. |

# **3\.  Persona System  (KPRS)**

A Persona is a named, configurable identity context attached to a profile. Multiple personas can be active simultaneously on a single profile, allowing the user to present contextually appropriate faces to different audiences and platform modules — without managing entirely separate profiles. Personas influence feed personalization, recommendation scoring, communication tone settings, and the OptimizationEngine's workload-tuning behavior.

## **3.1  Persona Data Model**

| Field | Description |
| :---- | :---- |
| id | UUID |
| profile\_id | Parent profile reference |
| name | Display name of this persona (e.g., 'Investor Lens', 'Developer Mode') |
| type | PersonaType enum (see §3.2) |
| icon | Visual identifier for quick persona recognition in the UI |
| is\_active | Bool — whether this persona is currently active on the profile |
| priority | Integer — ordering weight when multiple personas are active |
| preferences | Persona-specific feed, notification, and display preferences |
| engine\_context | Serialized context blob consumed by PersonalizationEngine and OptimizationEngine |
| skills\_emphasized | Vec\<Skill\> — skills foregrounded by this persona |
| portfolios\_emphasized | Vec\<PortfolioRef\> — portfolio items highlighted by this persona |
| visibility\_overrides | Optional visibility rules that override profile defaults for this persona |
| communication\_tone | CommunicationTone: Professional | Casual | Technical | Creative | Formal |
| created\_at / updated\_at | Timestamps |

## **3.2  Persona Types**

| Persona Type | Description | Engine Behavior |
| :---- | :---- | :---- |
| Developer | Software engineering, open source, technical work | RecommendationEngine weights developer gigs, tech communities, code repositories |
| Creative | Design, art, music, content creation, media | Feed weights creative communities, design assets, studio tools |
| Investor | Capital allocation, equity crowdfunding, group economics | Feed weights investment opportunities, campaigns, portfolio performance signals |
| Entrepreneur | Startup building, product development, growth | Weights startup communities, funding opportunities, growth analytics |
| Consultant | Professional advisory, strategy, domain expertise | Weights professional gigs, consulting contracts, B2B marketplace listings |
| Organizer | Community building, cooperative governance, collective action | Weights community spaces, governance proposals, collective initiatives |
| Researcher | Academic, analytical, data-driven inquiry | Weights knowledge resources, data tools, research communities |
| Educator | Teaching, training, mentorship, curriculum design | Weights professional development resources, mentorship matching, training gigs |
| Activist | Advocacy, social change, mission-driven work | Weights mutual aid funds, grant opportunities, community initiatives |
| Technologist | Applied technology, infrastructure, systems | Weights technical projects, infrastructure gigs, tooling communities |
| Facilitator | Meeting design, process facilitation, coordination | Weights project management gigs, cooperative governance roles |
| Hobbyist | Non-commercial passion projects and personal interests | Weights community spaces, creative projects, resource sharing |
| Custom | User-defined persona with configurable type label and engine hints | Engine context manually specified by user |

## **3.3  Persona Switching & Contexts**

* The active persona set is stored on the profile's session context, updated via the profile switcher or Oba AI command.

* Multiple personas can be simultaneously active — the PersonalizationEngine blends their weights.

* Persona-aware notification routing: an Investor persona routes campaign alerts to high priority; a Developer persona suppresses business development notifications during deep work hours.

* Oba (the AI assistant) adapts its communication style, recommendations, and proactive suggestions based on the active persona set.

* OptimizationEngine adjusts resource allocation suggestions, workload recommendations, and income projections based on persona context.

# **4\.  Linked Accounts System  (KACC)**

The Linked Accounts system connects a user's external digital platform accounts to their Kogi profile, forming a digital account tree rooted at the Kogi identity. Each profile can carry a different subset of linked accounts, allowing context-specific account configurations without exposing all accounts in all contexts.

## **4.1  LinkedAccount Data Model**

| Field | Description |
| :---- | :---- |
| id | UUID |
| profile\_id | Profile this account is linked to |
| platform | Platform identifier (github, gitlab, slack, discord, etc.) |
| platform\_user\_id | Account identifier on the external platform |
| platform\_username | Display handle on the external platform |
| platform\_url | Direct URL to the account (forms part of the linktree leaf) |
| account\_type | AccountType enum (see §4.2) |
| auth\_method | AuthMethod: OAuth2 | APIKey | Manual | Verified |
| oauth\_token\_ref | Encrypted token reference (never stored in plaintext) |
| status | LinkStatus: Active | Disconnected | Expired | Revoked |
| visibility | VisibilityLevel — who can see this linked account on the profile |
| is\_primary | Bool — whether this is the primary account for this platform type |
| sync\_enabled | Bool — whether activity data syncs to kogi-engine |
| sync\_scopes | Vec\<SyncScope\> — what data is permitted to sync (profile, activity, content) |
| last\_synced\_at | Timestamp |
| metadata | Map\<String, Value\> — platform-specific supplementary data |
| created\_at / updated\_at | Timestamps |

## **4.2  Account Type Categories**

| Category | Examples | Notes |
| :---- | :---- | :---- |
| Communication & Messaging | Gmail, Yahoo Mail, Outlook, ProtonMail, WhatsApp, iMessage, Signal, Telegram | Contact channels; govern which platforms can receive inbound messages routed via kogi-net |
| Professional Networks | LinkedIn, AngelList, Wellfound, Contra, Toptal, Upwork, Fiverr | Professional identity; gig platform accounts feed into PortableContribution tracking |
| Gig & Work Platforms | DoorDash, Instacart, Lyft, Uber, TaskRabbit, Handy, Rover, Wolt | Income sources; contributions tracked in kogi-bank PortableContributions |
| Developer Platforms | GitHub, GitLab, Bitbucket, npm, PyPI, Hugging Face, Stack Overflow | Technical identity; repository links form part of Developer persona linktree |
| Creative Platforms | Behance, Dribbble, Figma, SoundCloud, Spotify, Bandcamp, Vimeo, YouTube | Creative portfolio; assets can be imported as Portfolio Artifacts |
| Social Media | Twitter/X, Instagram, TikTok, Facebook, Threads, Mastodon, Bluesky | Public identity and reach; follower counts surface in profile analytics |
| Commerce & Marketplace | Amazon, eBay, Etsy, Shopify, Gumroad, LTK, Depop, Poshmark | Seller identities; product listings can link to Portfolio Items |
| Productivity & Collaboration | Notion, Airtable, Trello, Asana, Jira, Monday, ClickUp, Coda | Work tool integrations; project data can sync to kogi-office |
| AI & Developer Tools | OpenAI/ChatGPT, Anthropic/Claude, Cursor, Replit, Vercel, Railway | Tool usage identities; relevant for Developer and Technologist personas |
| Finance & Payments | PayPal, Venmo, Cash App, Stripe, Wave, QuickBooks, Coinbase | Payment identities; routing instructions for marketplace transactions |
| Education & Credentials | Coursera, LinkedIn Learning, Credly, Acclaim, Udemy, Skillshare | Certification and credential proofs; import to ProfileIdentity.certifications |
| Community & Forums | Discord, Slack, Reddit, Discourse, Circle, Geneva, Mighty Networks | Community memberships; relevant for social graph (KSGR) enrichment |
| Publishing & Content | Substack, Medium, Ghost, WordPress, Hashnode, Dev.to, Patreon | Content creator identity; posts can sync as Portfolio Artifacts |
| Phone Numbers | Personal, Work, School, Public — any number of labeled phone numbers | Contact channels; enabled/disabled per profile |
| Email Addresses | Personal, Work, School, Public — any number of labeled email addresses | Contact channels; governed by profile communication policy |
| Websites | Personal site, work site, school/org site, portfolio site | URL leaves in the linktree; surfaced on profile card |

# **5\.  Communication Channels  (KPRF·CHAN)**

Each profile carries a configurable set of communication channels. Channels determine how other users, automated systems, and external platforms can reach the user through a given profile. Channel availability, routing, and access are independently controlled per profile — a Work Profile might expose Slack and email; a Public Profile might expose only a kogi-platform DM form.

## **5.1  Channel Configuration Model**

| Field | Description |
| :---- | :---- |
| channel\_id | UUID |
| profile\_id | Parent profile |
| channel\_type | ChannelType enum (see §5.2) |
| address | The channel address (email, phone number, platform handle, URL) |
| label | User-defined label (e.g., 'Work Email', 'Client DMs Only') |
| linked\_account\_id | Optional ref to a LinkedAccount (for platform-native channels) |
| enabled | Bool — whether this channel is active and accepts inbound contact |
| visibility | Who can see this channel: Public | Connections | Trusted | Private |
| accept\_from | AccessPolicy — who can initiate contact on this channel |
| routing\_rules | Vec\<RoutingRule\> — conditional routing (e.g., 'after 6pm → quiet mode') |
| priority | Integer — ordering of channels in contact card display |
| auto\_reply | Option\<AutoReply\> — configured auto-response message and schedule |
| requires\_request | Bool — if true, sender must send a contact request before messaging |

## **5.2  Channel Types**

| Channel Type | Description | Direction |
| :---- | :---- | :---- |
| kogi-dm | Native kogi platform direct message (unicast) | Bi-directional |
| kogi-group-msg | Native kogi group message (multicast) | Bi-directional |
| kogi-broadcast | Native kogi broadcast channel (one-to-many) | Outbound |
| kogi-notification | Platform event notifications to this profile | Inbound |
| email | Email channel (personal, work, school, public) | Bi-directional |
| phone-call | Voice call contact | Bi-directional |
| sms | SMS/text message | Bi-directional |
| whatsapp | WhatsApp message routing via linked account | Bi-directional |
| slack-dm | Slack DM via linked Slack account and workspace | Bi-directional |
| discord-dm | Discord DM via linked Discord account | Bi-directional |
| telegram | Telegram message via linked Telegram account | Bi-directional |
| linkedin-message | LinkedIn InMail/message via linked LinkedIn account | Bi-directional |
| twitter-dm | Twitter/X DM via linked Twitter account | Bi-directional |
| instagram-dm | Instagram DM via linked Instagram account | Bi-directional |
| facebook-message | Facebook Messenger via linked Facebook account | Bi-directional |
| calendar-invite | Booking/scheduling invite via kogi-office BookingEngine | Inbound |
| contact-form | Public contact form (generates kogi-platform DM) | Inbound |
| webhook | Developer-configured outbound webhook for event routing | Outbound |

## **5.3  Channel Routing & Privacy Controls**

* Channels support granular accept\_from policies: Any | Verified Users | Connections | Trusted | Whitelist | None (closed).

* Quiet Hours: per-channel or profile-wide quiet windows; inbound messages queued, not notified, during quiet hours.

* Contact Request Gate: if requires\_request is enabled, senders must submit a contact request which the user approves before direct messaging is unlocked.

* Channel Visibility: each channel's address can be revealed progressively — public users see the contact form only; connections see the email; trusted contacts see the phone number.

* Auto-routing via kogi-engine: high-priority senders (existing clients, cooperative members, MatchEngine recommendations) can be configured to bypass contact request gates automatically.

# **6\.  Visibility & Access Control  (KVIS)**

The Kogi profile visibility system provides tiered, granular access control over every layer of a profile — from the profile's existence and discoverability, to individual fields, channels, portfolios, and linked accounts. Access control is enforced by the KOGI-MANAGER RBAC layer and governed by attached policy objects.

## **6.1  Visibility Tiers**

| Tier | Audience | Description |
| :---- | :---- | :---- |
| Public | Anyone (including unauthenticated users) | Fully visible and indexed by SearchEngine. Discoverable via kogi-net linkforest and external search engines if configured. |
| Platform | Any authenticated kogi user | Visible to all platform members. Not indexed for external search. |
| Protected | Approved followers and connections | Discoverable but content gated behind follow/connect approval. Profile card visible; content requires approval. |
| Connections | Accepted connections only | Not discoverable externally. Only reachable via direct link or contact-initiated introduction. |
| Trusted | Explicitly whitelisted users and organizations | Highest-trust audience. User manually grants access. No discovery pathway. |
| Private | Owner only | Invisible to all others. No discovery. Useful for draft profiles and private project tracking. |
| Custom | User-defined audience policy | Arbitrary audience definition using organization membership, role, tag, or explicit user list. |

## **6.2  Field-Level Access Control**

Individual profile fields carry their own visibility configuration, which can be set independently of the profile's top-level visibility. Example configurations:

| Profile Field | Typical Default | Notes |
| :---- | :---- | :---- |
| Display name / slug | Public | Always visible at profile tier or above |
| Tagline / bio | Public or Platform | Often public for professional profiles |
| Location | Platform or Protected | City-level only for Public; full location for Connections |
| Email addresses | Connections or Trusted | Work email may be Protected; personal email Private |
| Phone numbers | Trusted or Private | Rarely exposed beyond trusted contacts |
| Linked accounts | Varies per account | Developer platforms often public; payment accounts private |
| Portfolio associations | Public or Protected | Configurable per portfolio link |
| Skills and certifications | Public or Platform | Generally shared broadly for professional discovery |
| Analytics / metrics | Private or Custom | Owner and authorized partners only |
| Legal name | Private | Only exposed for verified legal/financial contexts |
| Organization memberships | Platform or Protected | User controls cooperative/collective membership visibility |

## **6.3  Permission Roles**

| Role | Scope | Permissions |
| :---- | :---- | :---- |
| Owner | Full profile | All CRUD \+ config \+ policy \+ analytics \+ sharing \+ archive |
| Editor | Profile content only | Update bio, metadata, linked accounts, channels — no permission changes |
| Viewer (Trusted) | All visible fields at Trusted tier | Read-only access to trusted-tier content |
| Viewer (Connection) | Connection-tier fields | Read-only access to connection-tier content |
| Viewer (Public) | Public-tier fields | Read-only access to public fields and contact form |
| Manager (Org-delegated) | Org-linked profile fields | Can edit org-associated profile data on behalf of the org |
| Auditor | EventLog only | Read-only access to profile event log for compliance |

# **7\.  kogi-net — LinkNetwork System  (KNET)**

| *Design Philosophy: The internet has fractured every person's digital identity across dozens — often hundreds — of platforms, accounts, handles, and URLs. kogi-net is the mycorrhizal substrate that connects these fragments into a unified, navigable, searchable digital identity forest. Like the mycelial networks beneath a forest floor, kogi-net forms the invisible connective tissue linking every digital node a user occupies, making the whole discoverable and navigable from a single root.* |
| :---- |

## **7.1  Concepts & Hierarchy**

| Concept | Definition |
| :---- | :---- |
| Digital Account | A single external account, address, or URL a user controls on any digital platform (email, phone, social handle, website, etc.) |
| Leaf | The atomic unit of a linktree — a single DigitalAccount entry with metadata, type, visibility, and status |
| LinkTree | The complete, structured collection of a user's digital accounts and addresses, organized as a tree rooted at their Kogi identity. One linktree per profile. |
| Branch | A logical grouping of leaves within a linktree (e.g., 'Developer Accounts', 'Social Media', 'Contact Channels') |
| LinkForest | The aggregate of all user linktrees on the platform, forming a searchable, navigable forest of digital identities |
| kogi-net | The root network — the mycorrhizal substrate connecting all linkforests across the Kogi platform and its federated network |
| LinkNet | A sub-network within kogi-net — a scoped linkforest for a specific community, organization, or cooperative. Organizations grow their own linknets from the root kogi-net. |
| ContactBook | A curated subset of the linkforest scoped to a user's contacts — a structured directory built from linktrees of connected users |
| Directory | An organization- or community-scoped ContactBook — a structured, searchable listing of member linktrees |

## **7.2  LinkTree Data Model**

| Field | Description |
| :---- | :---- |
| id | UUID — globally unique linktree identifier |
| profile\_id | Parent profile (each profile has exactly one linktree) |
| owner\_user\_id | User owning this linktree |
| slug | Shareable URL slug (e.g., kogi.net/@alex or kogi.net/l/alex-dev) |
| title | Display title for the linktree page |
| description | Short description shown on public linktree page |
| branches | Vec\<Branch\> — ordered logical groupings of leaves |
| leaves | Vec\<Leaf\> — flat list of all leaf accounts (also accessible via branches) |
| theme | Visual theme for the shareable linktree page |
| visibility | VisibilityLevel — who can view this linktree |
| analytics | LinkTreeAnalytics — click counts, view counts per leaf |
| featured\_leaves | Vec\<LeafId\> — pinned/featured accounts shown at the top |
| created\_at / updated\_at | Timestamps |

## **7.3  Leaf Data Model**

| Field | Description |
| :---- | :---- |
| id | UUID |
| linktree\_id | Parent linktree |
| linked\_account\_id | Optional ref to KACC LinkedAccount (if sourced from linked account) |
| platform | Platform identifier string |
| platform\_type | PlatformType enum (social, developer, commerce, communication, etc.) |
| label | User-defined display label for this leaf |
| address | The URL, handle, email, phone, or account ID |
| url | Resolved navigable URL (may be generated from address \+ platform template) |
| icon | Platform icon reference for display |
| visibility | VisibilityLevel — who can see this leaf |
| is\_featured | Bool — whether this leaf appears in the featured row |
| is\_verified | Bool — whether this account has been verified by the platform |
| order | Integer — display order within its branch |
| click\_count | Analytics counter |
| tags | Vec\<String\> — searchable tags on this leaf |
| created\_at | Timestamp |

## **7.4  Branch Structure**

Branches organize leaves into logical groupings within a linktree. Default branch templates are provided but fully customizable:

| Default Branch | Typical Leaves |
| :---- | :---- |
| Contact | Primary email, primary phone, contact form, kogi-dm link |
| Professional | LinkedIn, resume link, portfolio site, professional certifications |
| Developer | GitHub, GitLab, npm profile, portfolio site, Stack Overflow |
| Creative | Behance, Dribbble, SoundCloud, YouTube, Vimeo, Instagram |
| Social | Twitter/X, Instagram, TikTok, Facebook, Threads, Mastodon |
| Commerce | Shopify store, Etsy shop, Amazon store, Gumroad |
| Publishing | Substack, Medium, personal blog, podcast feed |
| Community | Discord server, Slack workspace, Reddit profile, Circle community |
| Work Platforms | Upwork, Contra, Toptal, DoorDash worker profile |
| Finance | Payment handles (PayPal, Venmo) — visibility gated to Trusted tier |

## **7.5  LinkForest & kogi-net Architecture**

The linkforest is the aggregated view of all user linktrees across the platform. kogi-net is the indexed, searchable, policy-controlled graph database sitting atop this forest. It functions as a mycelial network — an invisible connective substrate that makes every node reachable from every other node, given appropriate permissions.

| kogi-net Component | Description |
| :---- | :---- |
| Root Network (kogi-net) | The platform-global root linknet. All user linktrees are nodes. All linknets are subgraphs. |
| LinkNet (Organization) | An organization's scoped subgraph — contains the linktrees of all members, searchable and navigable within the org context |
| LinkNet (Community) | A community or space's scoped subgraph — connects the linktrees of community members |
| LinkNet (Federation) | A cross-organization federation's composite subgraph — the union of multiple org linknets |
| GraphEngine Integration | kogi-engine GraphEngine traverses the linkforest for connection path queries, introduction chains, and network distance computation |
| SearchEngine Integration | kogi-engine SearchEngine indexes all public and platform-visible leaves for full-text and faceted search |
| MatchEngine Integration | kogi-engine MatchEngine uses linkforest topology and leaf metadata to score compatibility between workers, collaborators, and opportunities |

## **7.6  Linktree Sharing & Portability**

* Every linktree generates a public shareable URL: kogi.net/@slug or kogi.net/l/\[slug\]

* Linktree pages are fully themeable and embed-ready (iFrame, widget, QR code).

* Users can export their linktree as a vCard, JSON-LD structured data, or a portable HTML page.

* QR codes are auto-generated and downloadable for physical sharing (business cards, event badges, venue walls).

* Linktrees can embed into third-party platforms via the kogi developer SDK.

* Linktrees support custom domain routing: users can point their own domain to their kogi linktree page.

# **8\.  ContactBook & Directory System  (KCON · KDIR)**

The ContactBook is a personal, curated view of the linkforest scoped to a user's network. A Directory is a structured, organization-scoped ContactBook — an authoritative member directory for cooperatives, collectives, and communities. Both are built from linktrees and updated in real time as connections change.

## **8.1  ContactBook**

| Feature | Description |
| :---- | :---- |
| Contact Sourcing | Contacts imported from: accepted kogi connections, linked account contact lists (Google Contacts, iCloud, Outlook), CSV import, QR code scan, linktree link |
| Contact Record | Each contact is a ContactComponent (PortfolioComponent subtype) containing a reference to the contact's linktree and a local annotation layer (private notes, tags, relationship context) |
| Contact Groups | User-defined groupings (e.g., 'Clients', 'Cooperative Members', 'Collaborators', 'Investors') |
| Contact Search | Full-text search across all contact fields: name, handle, company, tags, skills, platform accounts |
| Communication Actions | One-click action on any contact to initiate DM, email, calendar invite, or group message — routing through the contact's enabled channels |
| Contact Sync | Bidirectional sync with linked account contact lists; delta sync on schedule or on-demand |
| Access Control | ContactBook is private by default. Subsets can be shared with org managers or collaborative teams. |
| Oba Integration | Oba can search the ContactBook, draft introductions, suggest connections, and recommend outreach based on active projects and opportunities |

## **8.2  Directory**

| Feature | Description |
| :---- | :---- |
| Scope | Organization, collective, cooperative, community, federation, or custom group |
| Auto-population | Directory auto-populates from org/space membership roster; member linktrees form the directory entries |
| Fields Shown | Configurable by org admin: display name, role, skills, contact channels, portfolio link, bio — each field respects individual member's visibility settings |
| Search & Filter | Full-text \+ faceted search: by name, role, skill, location, persona type, availability |
| Ranking | MatchEngine-powered ranking: sort members by relevance to a query, project need, or skill requirement |
| Export | Export directory as CSV, vCard bundle, PDF roster, or JSON-LD |
| Embeddable Widget | Directories can be embedded on external org websites via kogi developer SDK |
| Governance Integration | Directory membership changes (join, leave, role change) generate EventLog entries and optionally trigger governance notifications |

## **8.3  Contact Channel Routing via kogi-net**

When a user initiates contact with another user through any kogi interface, the platform resolves the optimal channel via kogi-net:

| Step | Action |
| :---- | :---- |
| 1\. Channel Resolution | kogi-net queries the recipient's enabled channels on the relevant profile (determined by context: marketplace gig → work profile; community DM → public profile) |
| 2\. Access Policy Check | KVIS evaluates whether the sender satisfies the accept\_from policy for each available channel |
| 3\. Channel Ranking | Available channels ranked by: platform context match, channel priority setting, recipient's preferred channel for this sender class |
| 4\. Contact Gate Check | If requires\_request is true on the top-ranked channel, a contact request is generated instead of direct delivery |
| 5\. Routing | Message or contact request routed to the resolved channel; delivery receipt generated; analytics event emitted |
| 6\. Cross-platform Delivery | If the resolved channel is an external platform (Slack, WhatsApp, email), the message is relayed via the linked account's ProviderSystem integration |

# **9\.  Profile Configuration & Preferences**

## **9.1  ProfilePreferences**

| Preference Group | Settings |
| :---- | :---- |
| Display & UI | Theme (light/dark/system/custom), color scheme, font scale, layout density, language, timezone |
| Feed & Content | Content categories to prioritize/suppress, feed algorithm weighting, autoplay settings, content language filters |
| Notifications | Per-event-type on/off; channel routing per notification type; digest frequency; quiet hours schedule |
| Privacy | Default visibility for new content, search indexing opt-in/out, data sharing controls, analytics visibility |
| Communication | Default reply tone, auto-reply templates, contact gate defaults, broadcast vs. DM routing preference |
| AI & Oba | Oba proactivity level, suggestion frequency, automation approval thresholds, AI model context permissions |
| Work & Portfolio | Default portfolio association for new projects, work-hour availability signals, timezone availability calendar |
| Discovery | Profile discoverability in MatchEngine, talent marketplace listing opt-in, skill endorsement visibility |

## **9.2  Configuration Parameters**

| Parameter | Description |
| :---- | :---- |
| profile.max\_linked\_accounts | Maximum linked accounts per profile (platform default: unlimited) |
| profile.max\_personas | Maximum personas per profile (platform default: 20\) |
| profile.contact\_request\_cooldown | Minimum time between contact requests from the same sender |
| profile.quiet\_hours.start / .end | Daily quiet window start and end times (IANA timezone) |
| profile.linktree.max\_leaves | Maximum leaves in a linktree (platform default: unlimited) |
| profile.analytics.retention\_days | How many days of analytics data to retain (default: 365\) |
| profile.sync.interval\_minutes | Linked account sync polling interval |
| profile.search.indexing\_enabled | Whether this profile is indexed by SearchEngine |

# **10\.  Engine Integrations**

| Engine | Integration with Profile System |
| :---- | :---- |
| PersonalizationEngine | Builds per-profile user model from persona set, activity history, content interactions, and linked account signals. Powers feed ranking, recommendation scoring, and content filtering. |
| RecommendationEngine | Scores opportunities (gigs, collaborators, communities, campaigns) against profile's persona set, skills, portfolio, and past preferences. Surfaces via Oba and home dashboard. |
| MatchEngine | Uses profile linktrees, skills, organization memberships, and portfolio signals to match workers to gigs, collaborators to projects, investors to campaigns. Core of talent discovery. |
| SearchEngine | Indexes all Public and Platform-visible profile fields, leaf nodes, skills, and portfolio associations. Supports full-text, faceted, and semantic similarity search across the linkforest. |
| GraphEngine | Models the linkforest as a directed property graph. Traverses connection paths, computes social distance, detects communities, and powers 'mutual connections' and introduction chain features. |
| AnalyticsEngine | Computes profile engagement metrics: profile views, linktree clicks, contact requests, follower growth, portfolio engagement driven from profile. Published to profile analytics dashboard. |
| TelemetryEngine | Tracks real-time profile activity events: switches, persona activations, linked account syncs, channel interactions. Feeds flow ledger and component flow stats. |
| RiskEngine | Computes trust and reputation scores from profile activity, delivery history, reviews, governance participation, and linked credential verifications. |
| OptimizationEngine | Persona-aware workload and resource optimization: recommends which profile/persona to activate for a given opportunity context, and when to switch communication channel strategies. |
| IncentiveEngine | Awards KogiPoints for profile completion milestones, identity verification, linked account additions, linktree sharing events, and contact-network growth. |

# **11\.  API Surface  (KPRF · REST \+ gRPC)**

## **11.1  Profile REST API  (External — kogi-dev)**

| Endpoint | Method | Description |
| :---- | :---- | :---- |
| GET /profiles/:id | GET | Retrieve profile by ID (visibility-filtered by caller's access level) |
| GET /profiles/@:slug | GET | Retrieve profile by slug |
| POST /profiles | POST | Create a new profile for the authenticated user |
| PATCH /profiles/:id | PATCH | Update profile fields (owner or editor only) |
| DELETE /profiles/:id | DELETE | Archive profile (soft delete) |
| GET /profiles/:id/linktree | GET | Retrieve the linktree for a profile |
| PATCH /profiles/:id/linktree | PATCH | Update linktree leaves and branches |
| GET /profiles/:id/personas | GET | List all personas on a profile |
| POST /profiles/:id/personas | POST | Add a persona to a profile |
| GET /profiles/:id/linked-accounts | GET | List linked accounts (visibility-filtered) |
| POST /profiles/:id/linked-accounts | POST | Link a new external account |
| DELETE /profiles/:id/linked-accounts/:acc\_id | DELETE | Unlink an account |
| GET /profiles/:id/channels | GET | List communication channels |
| POST /profiles/:id/channels | POST | Add or configure a channel |
| GET /users/:id/profiles | GET | List all profiles for a user (owner only returns all; others see public only) |
| POST /profiles/switch/:id | POST | Switch active profile context |
| GET /linkforest/search | GET | Search the linkforest (query, filters, pagination) |
| GET /contactbook | GET | Retrieve authenticated user's ContactBook |
| POST /contactbook/import | POST | Import contacts from linked account |
| GET /directory/:org\_id | GET | Retrieve organization directory |

## **11.2  Internal gRPC Services**

| Service | Methods |
| :---- | :---- |
| ProfileService | GetProfile, CreateProfile, UpdateProfile, ArchiveProfile, SwitchProfile, ListUserProfiles |
| PersonaService | GetPersonas, AddPersona, UpdatePersona, RemovePersona, ActivatePersona, DeactivatePersona |
| LinkedAccountService | GetLinkedAccounts, LinkAccount, UnlinkAccount, SyncAccount, GetAccountStatus |
| LinkTreeService | GetLinkTree, UpdateLinkTree, AddLeaf, RemoveLeaf, ReorderLeaves, GetLinkTreeAnalytics |
| ChannelService | GetChannels, ConfigureChannel, EnableChannel, DisableChannel, RouteContact |
| ContactBookService | GetContacts, ImportContacts, ExportContacts, SearchContacts, UpdateContactRecord |
| DirectoryService | GetDirectory, SearchDirectory, ExportDirectory, UpdateMemberEntry |
| KogiNetService | SearchLinkForest, GetConnectionPath, GetNearbyProfiles, GetLinkNetMembers |
| VisibilityService | GetPermissions, UpdatePermissions, EvaluateAccess, GetAudienceForField |

# **12\.  Key Workflows**

## **12.1  New Profile Creation**

| Step | Action |
| :---- | :---- |
| 1 | User selects 'New Profile' from profile switcher → selects profile type (or template) |
| 2 | ProfileService creates ProfileComponent with Draft status and default visibility |
| 3 | User fills core metadata: name, slug, tagline, bio, avatar, timezone |
| 4 | Default linktree created with empty branches; user prompted to add leaves (linked accounts) |
| 5 | User selects or creates personas for this profile |
| 6 | Communication channels configured: enabled/disabled, visibility set per channel |
| 7 | Portfolio associations added: link existing portfolios or create new project under this profile |
| 8 | Visibility and permissions configured; profile graduated from Draft → Active |
| 9 | IncentiveEngine awards KogiPoints for profile completion milestones |
| 10 | If visibility is Public/Platform: SearchEngine indexes the profile; linktree leaf appears in linkforest |

## **12.2  Linking an External Account**

| Step | Action |
| :---- | :---- |
| 1 | User selects platform from account library in KACC settings |
| 2 | OAuth2 flow initiated via kogi-services ProviderSystem; token obtained and encrypted |
| 3 | LinkedAccount record created with AccountType, status=Active, sync\_enabled per user choice |
| 4 | Account added as leaf to profile's linktree under appropriate branch |
| 5 | If sync\_enabled: initial sync fetches profile data, follower counts, and other permitted fields |
| 6 | If platform is a gig platform (DoorDash, Lyft, etc.): BenefitsEngine registers as PortableContribution source |
| 7 | Channel created in KPRF·CHAN for messaging platforms (Slack, Discord, WhatsApp, etc.) |
| 8 | Analytics leaf click counter initialized; IncentiveEngine awards KP for new link |

## **12.3  Contact Routing via kogi-net**

| Step | Action |
| :---- | :---- |
| 1 | Sender initiates contact action on a profile (DM button, contact form, marketplace message) |
| 2 | KogiNetService resolves target profile and retrieves enabled channel list |
| 3 | VisibilityService evaluates sender's access tier against each channel's accept\_from policy |
| 4 | Eligible channels ranked; top channel selected |
| 5 | If requires\_request: ContactRequest generated → delivered to recipient's notification stack → awaits approval |
| 6 | If not gated: message composed → routed to ChannelService → delivered via appropriate provider |
| 7 | For external platforms (Slack, email, WhatsApp): ProviderSystem relays via linked account credentials |
| 8 | Delivery analytics event emitted; channel click\_count incremented on linktree leaf |

## **12.4  Linktree Sharing & QR Flow**

| Step | Action |
| :---- | :---- |
| 1 | User opens linktree editor → configures leaf visibility, order, featured items, theme |
| 2 | Linktree page rendered at kogi.net/@slug (or custom domain if configured) |
| 3 | User selects 'Share' → options: copy URL, download QR code, generate embed widget, share via kogi-dm or channel |
| 4 | Third party visits linktree URL → sees all Public-tier leaves \+ contact form |
| 5 | If third party is authenticated kogi user: their access tier determines additional visible leaves |
| 6 | Leaf clicks tracked → ProfileAnalytics updated → surfaced in profile analytics dashboard |
| 7 | Oba can proactively suggest linktree optimization: 'Your Developer branch has no portfolio link — add your GitHub?' |

# **13\.  Security & Compliance**

| Concern | Implementation |
| :---- | :---- |
| OAuth Token Storage | All OAuth tokens stored encrypted at rest; referenced via opaque token\_ref; never exposed via API |
| PII Protection | Legal name, phone numbers, and private emails classified as PII; stored with field-level encryption; subject to data retention policies |
| EventLog Integrity | All profile mutations appended to tamper-evident EventLog with actor, timestamp, and delta; used for audit and compliance |
| RBAC Enforcement | All profile API endpoints guarded by KOGI-MANAGER RBAC; access evaluated against profile's PermissionConfig on every request |
| Rate Limiting | Contact request initiation, linked account sync, and linktree scraping subject to per-user rate limits |
| Data Portability | Users can export their complete profile data (GDPR/CCPA compliance): all linked accounts, linktrees, contact books, channel configs, and analytics as a JSON or CSV archive |
| Right to Erasure | Profile archive → anonymization pipeline: all PII removed; derived signals in kogi-engine anonymized or deleted per data retention policy |
| Cross-Profile Isolation | Profiles of the same user are isolated: no cross-profile data leakage via API; cross-profile analytics aggregation requires explicit user consent |
| Credential Verification | Platform-level identity verification (ID, email, phone) recorded at User level and surfaced per-profile with user consent |

KOGI Independent Worker Operating System  ·  kogi-profile SDD v1.0  ·  March 2026  ·  Confidential