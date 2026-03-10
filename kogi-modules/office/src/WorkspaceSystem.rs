use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceSnapshot {
    pub view: String,
    pub domains: Vec<String>,
    pub user_stories: Vec<String>,
    pub work_packages: Vec<String>,
    pub toolchains: Vec<String>,
    pub tool_links: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewWorkspaceStory {
    pub title: String,
    pub points: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceStory {
    pub id: String,
    pub title: String,
    pub points: i32,
}

#[derive(Clone, Debug)]
pub struct WorkspaceSystem {
    stories: Vec<WorkspaceStory>,
    next_story_id: u64,
}

impl WorkspaceSystem {
    pub fn mvp() -> Self {
        Self {
            stories: vec![WorkspaceStory {
                id: "story-001".to_string(),
                title: "As a worker, I track project blockers".to_string(),
                points: 5,
            }],
            next_story_id: 2,
        }
    }

    pub fn snapshot(&self) -> WorkspaceSnapshot {
        WorkspaceSnapshot {
            view: "workspace".to_string(),
            domains: vec![
                "personal_work".to_string(),
                "operations".to_string(),
                "tactics".to_string(),
                "strategy".to_string(),
                "governance".to_string(),
            ],
            user_stories: self
                .stories
                .iter()
                .map(|x| format!("{} ({})", x.title, x.points))
                .collect(),
            work_packages: vec![
                "Kernel isolation instrumentation".to_string(),
                "Office dashboard feed aggregation".to_string(),
            ],
            toolchains: vec![
                "build-chain".to_string(),
                "release-chain".to_string(),
                "observability-chain".to_string(),
            ],
            tool_links: vec![
                "https://jira.example.local".to_string(),
                "https://github.example.local".to_string(),
            ],
        }
    }

    pub fn add_story(&mut self, request: NewWorkspaceStory) -> WorkspaceStory {
        let story = WorkspaceStory {
            id: format!("story-{:03}", self.next_story_id),
            title: request.title,
            points: request.points,
        };
        self.next_story_id += 1;
        self.stories.push(story.clone());
        story
    }
}
