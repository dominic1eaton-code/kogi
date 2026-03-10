use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssistantSnapshot {
    pub view: String,
    pub assistant_id: String,
    pub context_sources: Vec<String>,
    pub recommendations: Vec<String>,
    pub subscriptions: Vec<String>,
    pub explore: Vec<String>,
    pub for_you: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewAssistantSubscription {
    pub topic: String,
}

#[derive(Clone, Debug)]
pub struct AssistantSystem {
    assistant_id: String,
    recommendations: Vec<String>,
    subscriptions: Vec<String>,
}

impl AssistantSystem {
    pub fn mvp() -> Self {
        Self {
            assistant_id: "office-assistant-001".to_string(),
            recommendations: vec![
                "Resolve high-priority attention items".to_string(),
                "Sync roadmap milestones with timeline calendar".to_string(),
            ],
            subscriptions: vec!["exchange.deal_updates".to_string()],
        }
    }

    pub fn snapshot(&self) -> AssistantSnapshot {
        AssistantSnapshot {
            view: "assistant".to_string(),
            assistant_id: self.assistant_id.clone(),
            context_sources: vec![
                "workspace".to_string(),
                "portfolio".to_string(),
                "timeline".to_string(),
                "messages".to_string(),
            ],
            recommendations: self.recommendations.clone(),
            subscriptions: self.subscriptions.clone(),
            explore: vec![
                "strategy templates".to_string(),
                "workspace automation recipes".to_string(),
            ],
            for_you: vec!["Top stories to complete this week".to_string()],
        }
    }

    pub fn add_subscription(&mut self, request: NewAssistantSubscription) -> String {
        if !self.subscriptions.contains(&request.topic) {
            self.subscriptions.push(request.topic.clone());
        }
        request.topic
    }

    pub fn add_recommendation(&mut self, message: &str) {
        self.recommendations.push(message.to_string());
    }
}
