use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimelineSnapshot {
    pub view: String,
    pub calendars: Vec<String>,
    pub schedules: Vec<String>,
    pub roadmaps: Vec<String>,
    pub gantts: Vec<String>,
    pub events: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewTimelineEvent {
    pub calendar_id: String,
    pub title: String,
    pub kind: String,
    pub scheduled_for: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimelineEventRecord {
    pub id: String,
    pub calendar_id: String,
    pub title: String,
    pub kind: String,
    pub scheduled_for: String,
}

#[derive(Clone, Debug)]
pub struct TimelineSystem {
    events: Vec<TimelineEventRecord>,
    next_event_id: u64,
}

impl TimelineSystem {
    pub fn mvp() -> Self {
        Self {
            events: vec![],
            next_event_id: 1,
        }
    }

    pub fn snapshot(&self) -> TimelineSnapshot {
        TimelineSnapshot {
            view: "timeline".to_string(),
            calendars: vec![
                "Personal Calendar".to_string(),
                "Work Calendar".to_string(),
                "Community Calendar".to_string(),
            ],
            schedules: vec![
                "Operations Schedule".to_string(),
                "Delivery Schedule".to_string(),
            ],
            roadmaps: vec![
                "Platform Roadmap".to_string(),
                "Office Module Roadmap".to_string(),
            ],
            gantts: vec![
                "Launch Gantt".to_string(),
                "Integrations Gantt".to_string(),
            ],
            events: self
                .events
                .iter()
                .map(|x| format!("{}:{}:{}:{}", x.calendar_id, x.kind, x.title, x.scheduled_for))
                .collect(),
        }
    }

    pub fn add_event(&mut self, request: NewTimelineEvent) -> TimelineEventRecord {
        let event = TimelineEventRecord {
            id: format!("time-evt-{:03}", self.next_event_id),
            calendar_id: request.calendar_id,
            title: request.title,
            kind: request.kind,
            scheduled_for: request.scheduled_for,
        };
        self.next_event_id += 1;
        self.events.push(event.clone());
        event
    }
}
