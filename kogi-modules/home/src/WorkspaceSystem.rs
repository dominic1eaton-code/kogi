use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceContentItem {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub location: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceContentSnapshot {
    pub files: Vec<WorkspaceContentItem>,
    pub documents: Vec<WorkspaceContentItem>,
    pub folders: Vec<WorkspaceContentItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceScheduleItem {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub scheduled_for: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceSnapshot {
    pub view: String,
    pub user_hub: Vec<String>,
    pub portfolios: Vec<String>,
    pub content: WorkspaceContentSnapshot,
    pub calendars: Vec<String>,
    pub timelines: Vec<String>,
    pub schedules: Vec<WorkspaceScheduleItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewWorkspaceContent {
    pub kind: String,
    pub name: String,
    pub location: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewWorkspaceSchedule {
    pub title: String,
    pub kind: String,
    pub scheduled_for: String,
}

#[derive(Clone, Debug)]
pub struct WorkspaceSystem {
    user_hub: Vec<String>,
    portfolios: Vec<String>,
    content_items: Vec<WorkspaceContentItem>,
    schedules: Vec<WorkspaceScheduleItem>,
    next_content_id: u64,
    next_schedule_id: u64,
}

impl WorkspaceSystem {
    pub fn mvp() -> Self {
        Self {
            user_hub: vec![
                "home feed".to_string(),
                "priorities".to_string(),
                "inbox".to_string(),
            ],
            portfolios: vec![
                "Personal Portfolio".to_string(),
                "Studio Portfolio".to_string(),
            ],
            content_items: vec![
                WorkspaceContentItem {
                    id: "content-001".to_string(),
                    kind: "file".to_string(),
                    name: "Q1-roadmap.csv".to_string(),
                    location: "/drive/roadmaps".to_string(),
                    status: "synced".to_string(),
                },
                WorkspaceContentItem {
                    id: "content-002".to_string(),
                    kind: "document".to_string(),
                    name: "Launch brief".to_string(),
                    location: "/docs/launch".to_string(),
                    status: "draft".to_string(),
                },
                WorkspaceContentItem {
                    id: "content-003".to_string(),
                    kind: "folder".to_string(),
                    name: "Contracts".to_string(),
                    location: "/legal/contracts".to_string(),
                    status: "active".to_string(),
                },
            ],
            schedules: vec![WorkspaceScheduleItem {
                id: "schedule-001".to_string(),
                title: "Weekly planning".to_string(),
                kind: "calendar".to_string(),
                scheduled_for: "2026-03-13T14:00:00Z".to_string(),
            }],
            next_content_id: 4,
            next_schedule_id: 2,
        }
    }

    pub fn snapshot(&self) -> WorkspaceSnapshot {
        WorkspaceSnapshot {
            view: "home.workspace".to_string(),
            user_hub: self.user_hub.clone(),
            portfolios: self.portfolios.clone(),
            content: build_content_snapshot(&self.content_items),
            calendars: vec![
                "Personal Calendar".to_string(),
                "Work Calendar".to_string(),
                "Community Calendar".to_string(),
            ],
            timelines: vec![
                "Launch Timeline".to_string(),
                "Delivery Timeline".to_string(),
            ],
            schedules: self.schedules.clone(),
        }
    }

    pub fn add_content(&mut self, request: NewWorkspaceContent) -> WorkspaceContentItem {
        let item = WorkspaceContentItem {
            id: format!("content-{:03}", self.next_content_id),
            kind: request.kind,
            name: request.name,
            location: request.location,
            status: "active".to_string(),
        };
        self.next_content_id += 1;
        self.content_items.push(item.clone());
        item
    }

    pub fn add_schedule(&mut self, request: NewWorkspaceSchedule) -> WorkspaceScheduleItem {
        let item = WorkspaceScheduleItem {
            id: format!("schedule-{:03}", self.next_schedule_id),
            title: request.title,
            kind: request.kind,
            scheduled_for: request.scheduled_for,
        };
        self.next_schedule_id += 1;
        self.schedules.push(item.clone());
        item
    }
}

fn build_content_snapshot(items: &[WorkspaceContentItem]) -> WorkspaceContentSnapshot {
    let mut files = Vec::new();
    let mut documents = Vec::new();
    let mut folders = Vec::new();

    for item in items {
        match item.kind.to_lowercase().as_str() {
            "file" => files.push(item.clone()),
            "document" | "doc" => documents.push(item.clone()),
            "folder" => folders.push(item.clone()),
            _ => files.push(item.clone()),
        }
    }

    WorkspaceContentSnapshot {
        files,
        documents,
        folders,
    }
}
