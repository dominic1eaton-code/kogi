use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProfileAccount {
    pub account_id: String,
    pub status: String,
    pub tier: String,
    pub wallet_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserActionOption {
    pub id: String,
    pub label: String,
    pub channel: String,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserActionRecord {
    pub id: String,
    pub action_id: String,
    pub target: String,
    pub message: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProfileSnapshot {
    pub view: String,
    pub user_id: String,
    pub display_name: String,
    pub account: ProfileAccount,
    pub profiles: Vec<String>,
    pub personas: Vec<String>,
    pub skills: Vec<String>,
    pub contact: Vec<String>,
    pub data: Vec<String>,
    pub metadata: Vec<String>,
    pub actions: Vec<UserActionOption>,
    pub action_history: Vec<UserActionRecord>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewUserAction {
    pub action_id: String,
    pub target: String,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewProfileSkill {
    pub skill: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewProfilePersona {
    pub persona: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewProfileContact {
    pub contact: String,
}

#[derive(Clone, Debug)]
pub struct ProfileSystem {
    user_id: String,
    display_name: String,
    account: ProfileAccount,
    profiles: Vec<String>,
    personas: Vec<String>,
    skills: Vec<String>,
    contact: Vec<String>,
    data: Vec<String>,
    metadata: Vec<String>,
    actions: Vec<UserActionOption>,
    action_history: Vec<UserActionRecord>,
    next_action_id: u64,
}

impl ProfileSystem {
    pub fn mvp() -> Self {
        Self {
            user_id: "user-001".to_string(),
            display_name: "Kogi Operator".to_string(),
            account: ProfileAccount {
                account_id: "acct-001".to_string(),
                status: "active".to_string(),
                tier: "core".to_string(),
                wallet_id: "wallet-001".to_string(),
            },
            profiles: vec![
                "personal".to_string(),
                "work".to_string(),
                "community".to_string(),
            ],
            personas: vec![
                "builder".to_string(),
                "operator".to_string(),
                "advisor".to_string(),
            ],
            skills: vec![
                "systems design".to_string(),
                "product strategy".to_string(),
                "rust".to_string(),
            ],
            contact: vec![
                "email: operator@kogi.local".to_string(),
                "signal: +1-000-000-0000".to_string(),
            ],
            data: vec![
                "activity_stream".to_string(),
                "transactions".to_string(),
                "content_archive".to_string(),
            ],
            metadata: vec![
                "locale: en-US".to_string(),
                "timezone: America/Chicago".to_string(),
            ],
            actions: default_actions(),
            action_history: vec![UserActionRecord {
                id: "action-001".to_string(),
                action_id: "message.dm".to_string(),
                target: "ops-team".to_string(),
                message: "Daily sync at 09:00".to_string(),
                status: "sent".to_string(),
            }],
            next_action_id: 2,
        }
    }

    pub fn snapshot(&self) -> ProfileSnapshot {
        ProfileSnapshot {
            view: "home.profile".to_string(),
            user_id: self.user_id.clone(),
            display_name: self.display_name.clone(),
            account: self.account.clone(),
            profiles: self.profiles.clone(),
            personas: self.personas.clone(),
            skills: self.skills.clone(),
            contact: self.contact.clone(),
            data: self.data.clone(),
            metadata: self.metadata.clone(),
            actions: self.actions.clone(),
            action_history: self.action_history.clone(),
        }
    }

    pub fn record_action(&mut self, request: NewUserAction) -> UserActionRecord {
        let record = UserActionRecord {
            id: format!("action-{:03}", self.next_action_id),
            action_id: request.action_id,
            target: request.target,
            message: request.message,
            status: "queued".to_string(),
        };
        self.next_action_id += 1;
        self.action_history.push(record.clone());
        record
    }

    pub fn add_skill(&mut self, request: NewProfileSkill) -> bool {
        if self.skills.contains(&request.skill) {
            return false;
        }
        self.skills.push(request.skill);
        true
    }

    pub fn add_persona(&mut self, request: NewProfilePersona) -> bool {
        if self.personas.contains(&request.persona) {
            return false;
        }
        self.personas.push(request.persona);
        true
    }

    pub fn add_contact(&mut self, request: NewProfileContact) -> bool {
        if self.contact.contains(&request.contact) {
            return false;
        }
        self.contact.push(request.contact);
        true
    }
}

fn default_actions() -> Vec<UserActionOption> {
    vec![
        UserActionOption {
            id: "message.dm".to_string(),
            label: "Direct Message".to_string(),
            channel: "dm-unicast".to_string(),
            description: "Send a direct message to a single recipient".to_string(),
        },
        UserActionOption {
            id: "message.broadcast".to_string(),
            label: "Broadcast Message".to_string(),
            channel: "broadcast".to_string(),
            description: "Send a message to all subscribers".to_string(),
        },
        UserActionOption {
            id: "message.group".to_string(),
            label: "Group Multicast".to_string(),
            channel: "group-multicast".to_string(),
            description: "Send a message to a curated group".to_string(),
        },
        UserActionOption {
            id: "notify".to_string(),
            label: "Notify".to_string(),
            channel: "event-notify".to_string(),
            description: "Emit an event notification".to_string(),
        },
        UserActionOption {
            id: "alert".to_string(),
            label: "Alert".to_string(),
            channel: "event-alert".to_string(),
            description: "Emit an event alert".to_string(),
        },
        UserActionOption {
            id: "recommend".to_string(),
            label: "Recommend".to_string(),
            channel: "personalized".to_string(),
            description: "Search personalized engine recommendations".to_string(),
        },
        UserActionOption {
            id: "discover".to_string(),
            label: "Discover".to_string(),
            channel: "global".to_string(),
            description: "Get global engine recommendations to explore".to_string(),
        },
        UserActionOption {
            id: "explore".to_string(),
            label: "Explore".to_string(),
            channel: "topic-expansion".to_string(),
            description: "Expand a topic into related threads".to_string(),
        },
    ]
}
