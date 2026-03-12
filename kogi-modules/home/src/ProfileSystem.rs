// ============================================================================
//  ProfileSystem.rs — Kogi Home · Profile System
//  Independent Worker Operating System
//
//  The Profile System manages the user's full identity surface within the
//  Kogi platform:
//
//    Account         — tier, wallet link, status, plan details
//    Profiles        — named identity contexts (personal, work, community…)
//    Personas        — roles the user projects (builder, operator, advisor…)
//    Skills          — tagged capabilities with proficiency levels
//    Contact         — communication channels (email, signal, social…)
//    Data            — linked data domains (activity, transactions, archive…)
//    Metadata        — locale, timezone, preferences
//    PortableConfig  — portable benefits and preferences that travel with the user
//    UserActions     — available actions + full action history
//    Presence        — online / availability status
// ============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────────────
// Account
// ─────────────────────────────────────────────────────────────────────────────

/// The platform account record — tier, billing, and wallet linkage.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProfileAccount {
    pub account_id: String,
    pub status: String,
    /// "free" | "core" | "pro" | "enterprise"
    pub tier: String,
    pub wallet_id: String,
    /// ISO 8601 timestamp of account creation
    pub created_at: DateTime<Utc>,
    /// Plan features enabled on this tier
    pub plan_features: Vec<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Presence
// ─────────────────────────────────────────────────────────────────────────────

/// User presence / availability state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserPresence {
    /// "online" | "away" | "busy" | "offline" | "custom"
    pub status: String,
    /// Optional custom status message, e.g. "Deep work until 3pm"
    pub message: Option<String>,
    /// Time the status was last set
    pub updated_at: DateTime<Utc>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Skills
// ─────────────────────────────────────────────────────────────────────────────

/// A skill entry with an optional self-assessed proficiency level.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProfileSkill {
    pub name: String,
    /// "beginner" | "intermediate" | "advanced" | "expert"
    pub level: Option<String>,
    /// Optional category / domain, e.g. "engineering", "design"
    pub category: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewProfileSkill {
    pub skill: String,
    pub level: Option<String>,
    pub category: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Personas
// ─────────────────────────────────────────────────────────────────────────────

/// A named role the user projects on the platform or to other users.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProfilePersona {
    pub name: String,
    pub description: Option<String>,
    pub active: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewProfilePersona {
    pub persona: String,
    pub description: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Contact
// ─────────────────────────────────────────────────────────────────────────────

/// A contact channel entry, e.g. "email: user@kogi.local".
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContactEntry {
    /// "email" | "phone" | "signal" | "telegram" | "linkedin" | "twitter" | "custom"
    pub channel: String,
    pub value: String,
    /// "public" | "private" | "protected"
    pub visibility: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewProfileContact {
    pub contact: String,
    pub channel: Option<String>,
    pub visibility: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Portable Benefits
// ─────────────────────────────────────────────────────────────────────────────

/// Represents one portable benefit record (health, retirement, insurance, etc.)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortableBenefit {
    pub id: String,
    /// "health" | "retirement" | "insurance" | "legal" | "other"
    pub kind: String,
    pub provider: String,
    pub status: String,
    pub details: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// User Actions
// ─────────────────────────────────────────────────────────────────────────────

/// Definition of an available action the user can trigger.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserActionOption {
    pub id: String,
    pub label: String,
    pub channel: String,
    pub description: String,
    pub category: String,
}

/// A historical record of an action the user triggered.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserActionRecord {
    pub id: String,
    pub action_id: String,
    pub target: String,
    pub message: String,
    /// "queued" | "sent" | "delivered" | "failed"
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewUserAction {
    pub action_id: String,
    pub target: String,
    pub message: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Profile Snapshot (read model)
// ─────────────────────────────────────────────────────────────────────────────

/// Complete immutable read-model of the user's profile panel.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProfileSnapshot {
    pub view: String,
    pub user_id: String,
    pub display_name: String,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub account: ProfileAccount,
    pub presence: UserPresence,
    /// Named profile contexts
    pub profiles: Vec<String>,
    pub personas: Vec<ProfilePersona>,
    pub skills: Vec<ProfileSkill>,
    pub contact: Vec<ContactEntry>,
    pub portable_benefits: Vec<PortableBenefit>,
    /// Linked data domains
    pub data: Vec<String>,
    /// Locale / timezone / preference metadata
    pub metadata: Vec<String>,
    pub actions: Vec<UserActionOption>,
    pub action_history: Vec<UserActionRecord>,
    pub generated_at: DateTime<Utc>,
}

// ─────────────────────────────────────────────────────────────────────────────
// ProfileSystem — mutable runtime
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct ProfileSystem {
    user_id: String,
    display_name: String,
    bio: Option<String>,
    avatar_url: Option<String>,
    account: ProfileAccount,
    presence: UserPresence,
    profiles: Vec<String>,
    personas: Vec<ProfilePersona>,
    skills: Vec<ProfileSkill>,
    contact: Vec<ContactEntry>,
    portable_benefits: Vec<PortableBenefit>,
    data: Vec<String>,
    metadata: Vec<String>,
    actions: Vec<UserActionOption>,
    action_history: Vec<UserActionRecord>,
    next_action_id: u64,
    next_benefit_id: u64,
}

impl ProfileSystem {
    // ── Constructors ──────────────────────────────────────────────────────────

    pub fn mvp() -> Self {
        let now = Utc::now();

        Self {
            user_id: "user-001".to_string(),
            display_name: "Kogi Operator".to_string(),
            bio: Some("Independent builder, operator, and platform architect.".to_string()),
            avatar_url: None,
            account: ProfileAccount {
                account_id: "acct-001".to_string(),
                status: "active".to_string(),
                tier: "core".to_string(),
                wallet_id: "wallet-001".to_string(),
                created_at: now,
                plan_features: vec![
                    "portfolio".to_string(),
                    "workspace".to_string(),
                    "marketplace".to_string(),
                    "ai_suggestions".to_string(),
                    "analytics_basic".to_string(),
                ],
            },
            presence: UserPresence {
                status: "online".to_string(),
                message: None,
                updated_at: now,
            },
            profiles: vec![
                "personal".to_string(),
                "work".to_string(),
                "community".to_string(),
            ],
            personas: vec![
                ProfilePersona {
                    name: "builder".to_string(),
                    description: Some("Focused on creating systems and products".to_string()),
                    active: true,
                },
                ProfilePersona {
                    name: "operator".to_string(),
                    description: Some("Runs programs and manages operations".to_string()),
                    active: true,
                },
                ProfilePersona {
                    name: "advisor".to_string(),
                    description: Some("Offers expertise and mentorship".to_string()),
                    active: false,
                },
            ],
            skills: vec![
                ProfileSkill {
                    name: "systems design".to_string(),
                    level: Some("expert".to_string()),
                    category: Some("engineering".to_string()),
                },
                ProfileSkill {
                    name: "product strategy".to_string(),
                    level: Some("advanced".to_string()),
                    category: Some("product".to_string()),
                },
                ProfileSkill {
                    name: "rust".to_string(),
                    level: Some("advanced".to_string()),
                    category: Some("engineering".to_string()),
                },
            ],
            contact: vec![
                ContactEntry {
                    channel: "email".to_string(),
                    value: "operator@kogi.local".to_string(),
                    visibility: "protected".to_string(),
                },
                ContactEntry {
                    channel: "signal".to_string(),
                    value: "+1-000-000-0000".to_string(),
                    visibility: "private".to_string(),
                },
            ],
            portable_benefits: vec![
                PortableBenefit {
                    id: "benefit-001".to_string(),
                    kind: "health".to_string(),
                    provider: "Stride Health".to_string(),
                    status: "active".to_string(),
                    details: "ACA Marketplace plan – Silver tier".to_string(),
                },
                PortableBenefit {
                    id: "benefit-002".to_string(),
                    kind: "retirement".to_string(),
                    provider: "Self-directed SEP-IRA".to_string(),
                    status: "active".to_string(),
                    details: "$18,000 contributed this year".to_string(),
                },
            ],
            data: vec![
                "activity_stream".to_string(),
                "transactions".to_string(),
                "content_archive".to_string(),
                "analytics".to_string(),
            ],
            metadata: vec![
                "locale: en-US".to_string(),
                "timezone: America/Chicago".to_string(),
                "currency: USD".to_string(),
                "theme: system".to_string(),
            ],
            actions: default_actions(),
            action_history: vec![UserActionRecord {
                id: "action-001".to_string(),
                action_id: "message.dm".to_string(),
                target: "ops-team".to_string(),
                message: "Daily sync at 09:00".to_string(),
                status: "sent".to_string(),
                created_at: now,
            }],
            next_action_id: 2,
            next_benefit_id: 3,
        }
    }

    // ── Read ──────────────────────────────────────────────────────────────────

    pub fn snapshot(&self) -> ProfileSnapshot {
        ProfileSnapshot {
            view: "home.profile".to_string(),
            user_id: self.user_id.clone(),
            display_name: self.display_name.clone(),
            bio: self.bio.clone(),
            avatar_url: self.avatar_url.clone(),
            account: self.account.clone(),
            presence: self.presence.clone(),
            profiles: self.profiles.clone(),
            personas: self.personas.clone(),
            skills: self.skills.clone(),
            contact: self.contact.clone(),
            portable_benefits: self.portable_benefits.clone(),
            data: self.data.clone(),
            metadata: self.metadata.clone(),
            actions: self.actions.clone(),
            action_history: self.action_history.clone(),
            generated_at: Utc::now(),
        }
    }

    /// Active personas only.
    pub fn active_personas(&self) -> Vec<&ProfilePersona> {
        self.personas.iter().filter(|p| p.active).collect()
    }

    /// Skills filtered by category.
    pub fn skills_by_category(&self, category: &str) -> Vec<&ProfileSkill> {
        self.skills
            .iter()
            .filter(|s| s.category.as_deref() == Some(category))
            .collect()
    }

    // ── Identity Mutations ────────────────────────────────────────────────────

    pub fn update_display_name(&mut self, name: String) {
        self.display_name = name;
    }

    pub fn update_bio(&mut self, bio: Option<String>) {
        self.bio = bio;
    }

    pub fn update_avatar(&mut self, url: Option<String>) {
        self.avatar_url = url;
    }

    // ── Presence ──────────────────────────────────────────────────────────────

    pub fn set_presence(&mut self, status: String, message: Option<String>) {
        self.presence = UserPresence {
            status,
            message,
            updated_at: Utc::now(),
        };
    }

    // ── Skills ────────────────────────────────────────────────────────────────

    /// Returns `true` if the skill was added, `false` if it already existed.
    pub fn add_skill(&mut self, request: NewProfileSkill) -> bool {
        if self.skills.iter().any(|s| s.name == request.skill) {
            return false;
        }
        self.skills.push(ProfileSkill {
            name: request.skill,
            level: request.level,
            category: request.category,
        });
        true
    }

    pub fn remove_skill(&mut self, skill_name: &str) -> bool {
        if let Some(pos) = self.skills.iter().position(|s| s.name == skill_name) {
            self.skills.remove(pos);
            return true;
        }
        false
    }

    // ── Personas ──────────────────────────────────────────────────────────────

    pub fn add_persona(&mut self, request: NewProfilePersona) -> bool {
        if self.personas.iter().any(|p| p.name == request.persona) {
            return false;
        }
        self.personas.push(ProfilePersona {
            name: request.persona,
            description: request.description,
            active: true,
        });
        true
    }

    pub fn toggle_persona(&mut self, persona_name: &str) -> bool {
        if let Some(p) = self.personas.iter_mut().find(|p| p.name == persona_name) {
            p.active = !p.active;
            return true;
        }
        false
    }

    // ── Contact ───────────────────────────────────────────────────────────────

    pub fn add_contact(&mut self, request: NewProfileContact) -> bool {
        if self.contact.iter().any(|c| c.value == request.contact) {
            return false;
        }
        self.contact.push(ContactEntry {
            channel: request.channel.unwrap_or_else(|| "custom".to_string()),
            value: request.contact,
            visibility: request.visibility.unwrap_or_else(|| "private".to_string()),
        });
        true
    }

    pub fn remove_contact(&mut self, value: &str) -> bool {
        if let Some(pos) = self.contact.iter().position(|c| c.value == value) {
            self.contact.remove(pos);
            return true;
        }
        false
    }

    // ── Portable Benefits ─────────────────────────────────────────────────────

    pub fn add_benefit(&mut self, kind: String, provider: String, details: String) -> PortableBenefit {
        let benefit = PortableBenefit {
            id: format!("benefit-{:03}", self.next_benefit_id),
            kind,
            provider,
            status: "active".to_string(),
            details,
        };
        self.next_benefit_id += 1;
        self.portable_benefits.push(benefit.clone());
        benefit
    }

    pub fn deactivate_benefit(&mut self, benefit_id: &str) -> bool {
        if let Some(b) = self.portable_benefits.iter_mut().find(|b| b.id == benefit_id) {
            b.status = "inactive".to_string();
            return true;
        }
        false
    }

    // ── Metadata / Profiles ───────────────────────────────────────────────────

    pub fn add_profile_context(&mut self, context: String) -> bool {
        if self.profiles.contains(&context) {
            return false;
        }
        self.profiles.push(context);
        true
    }

    pub fn update_metadata(&mut self, key: String, value: String) {
        let prefix = format!("{key}: ");
        if let Some(entry) = self.metadata.iter_mut().find(|m| m.starts_with(&prefix)) {
            *entry = format!("{key}: {value}");
        } else {
            self.metadata.push(format!("{key}: {value}"));
        }
    }

    // ── Account ───────────────────────────────────────────────────────────────

    pub fn upgrade_tier(&mut self, tier: String) {
        self.account.tier = tier;
    }

    // ── User Actions ──────────────────────────────────────────────────────────

    pub fn record_action(&mut self, request: NewUserAction) -> UserActionRecord {
        let record = UserActionRecord {
            id: format!("action-{:03}", self.next_action_id),
            action_id: request.action_id,
            target: request.target,
            message: request.message,
            status: "queued".to_string(),
            created_at: Utc::now(),
        };
        self.next_action_id += 1;
        self.action_history.push(record.clone());
        record
    }

    pub fn update_action_status(&mut self, action_id: &str, status: String) -> bool {
        if let Some(r) = self.action_history.iter_mut().find(|r| r.id == action_id) {
            r.status = status;
            return true;
        }
        false
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Default Actions Registry
// ─────────────────────────────────────────────────────────────────────────────

fn default_actions() -> Vec<UserActionOption> {
    vec![
        UserActionOption {
            id: "message.dm".to_string(),
            label: "Direct Message".to_string(),
            channel: "dm-unicast".to_string(),
            description: "Send a direct message to a single recipient".to_string(),
            category: "messaging".to_string(),
        },
        UserActionOption {
            id: "message.broadcast".to_string(),
            label: "Broadcast Message".to_string(),
            channel: "broadcast".to_string(),
            description: "Send a message to all subscribers".to_string(),
            category: "messaging".to_string(),
        },
        UserActionOption {
            id: "message.group".to_string(),
            label: "Group Multicast".to_string(),
            channel: "group-multicast".to_string(),
            description: "Send a message to a curated group".to_string(),
            category: "messaging".to_string(),
        },
        UserActionOption {
            id: "notify".to_string(),
            label: "Notify".to_string(),
            channel: "event-notify".to_string(),
            description: "Emit an event notification to subscribers".to_string(),
            category: "events".to_string(),
        },
        UserActionOption {
            id: "alert".to_string(),
            label: "Alert".to_string(),
            channel: "event-alert".to_string(),
            description: "Emit a high-priority event alert".to_string(),
            category: "events".to_string(),
        },
        UserActionOption {
            id: "recommend".to_string(),
            label: "Recommend".to_string(),
            channel: "personalized".to_string(),
            description: "Search personalized engine recommendations".to_string(),
            category: "discovery".to_string(),
        },
        UserActionOption {
            id: "discover".to_string(),
            label: "Discover".to_string(),
            channel: "global".to_string(),
            description: "Get global engine recommendations to explore".to_string(),
            category: "discovery".to_string(),
        },
        UserActionOption {
            id: "explore".to_string(),
            label: "Explore".to_string(),
            channel: "topic-expansion".to_string(),
            description: "Expand a topic into related threads and subtopics".to_string(),
            category: "discovery".to_string(),
        },
        UserActionOption {
            id: "match".to_string(),
            label: "Match".to_string(),
            channel: "match-engine".to_string(),
            description: "Find matched users, resources, or portfolio components".to_string(),
            category: "discovery".to_string(),
        },
        UserActionOption {
            id: "invite".to_string(),
            label: "Invite".to_string(),
            channel: "invite-link".to_string(),
            description: "Invite someone to a project, space, or program".to_string(),
            category: "collaboration".to_string(),
        },
        UserActionOption {
            id: "share".to_string(),
            label: "Share".to_string(),
            channel: "share-link".to_string(),
            description: "Share a portfolio item, post, or resource".to_string(),
            category: "collaboration".to_string(),
        },
        UserActionOption {
            id: "campaign".to_string(),
            label: "Campaign".to_string(),
            channel: "campaign-engine".to_string(),
            description: "Launch or manage a targeted campaign".to_string(),
            category: "marketing".to_string(),
        },
    ]
}
