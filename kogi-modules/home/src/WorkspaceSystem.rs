// ============================================================================
//  WorkspaceSystem.rs — Kogi Home · Workspace System
//  Independent Worker Operating System
//
//  The Workspace is the personal operational hub attached to the user's Home
//  module.  It bridges all other Kogi platform domains and provides the user
//  with day-to-day tooling:
//
//    UserHub       — feed, priorities, inbox, pinned shortcuts
//    Portfolios    — linked portfolio references
//    ContentSystem — files, documents, folders with lifecycle status
//    Notebooks     — note / memo books (quick capture, guidelines, refs)
//    ContactBooks  — contact lists (personal, professional, organization)
//    Calendars     — multiple named calendar contexts
//    Timelines     — project / initiative timelines
//    Schedules     — time-ordered list of upcoming items
//    Boards        — lightweight kanban / task-list surfaces
// ============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────────────
// Content System
// ─────────────────────────────────────────────────────────────────────────────

/// A single content item — file, document, or folder.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceContentItem {
    pub id: String,
    /// "file" | "document" | "folder"
    pub kind: String,
    pub name: String,
    pub location: String,
    /// "synced" | "draft" | "active" | "archived" | "deleted"
    pub status: String,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Categorized view of workspace content.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceContentSnapshot {
    pub files: Vec<WorkspaceContentItem>,
    pub documents: Vec<WorkspaceContentItem>,
    pub folders: Vec<WorkspaceContentItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewWorkspaceContent {
    pub kind: String,
    pub name: String,
    pub location: String,
    pub tags: Vec<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Schedules
// ─────────────────────────────────────────────────────────────────────────────

/// A single entry on the workspace schedule.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceScheduleItem {
    pub id: String,
    pub title: String,
    /// "calendar" | "deadline" | "milestone" | "reminder" | "meeting"
    pub kind: String,
    pub scheduled_for: String,
    pub description: Option<String>,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewWorkspaceSchedule {
    pub title: String,
    pub kind: String,
    pub scheduled_for: String,
    pub description: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Notebooks
// ─────────────────────────────────────────────────────────────────────────────

/// An individual note within a notebook.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceNote {
    pub id: String,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A named notebook that groups related notes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceNotebook {
    pub id: String,
    pub name: String,
    /// "notebook" | "guidebook" | "planbook" | "playbook"
    pub kind: String,
    pub notes: Vec<WorkspaceNote>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewWorkspaceNote {
    pub notebook_id: String,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewWorkspaceNotebook {
    pub name: String,
    pub kind: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Contact Books
// ─────────────────────────────────────────────────────────────────────────────

/// An individual contact record.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceContact {
    pub id: String,
    pub name: String,
    pub role: Option<String>,
    pub organization: Option<String>,
    pub channels: Vec<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

/// A named contact book grouping related contacts.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceContactBook {
    pub id: String,
    pub name: String,
    /// "personal" | "professional" | "organizational"
    pub kind: String,
    pub contacts: Vec<WorkspaceContact>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewWorkspaceContact {
    pub contact_book_id: String,
    pub name: String,
    pub role: Option<String>,
    pub organization: Option<String>,
    pub channels: Vec<String>,
    pub tags: Vec<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Boards
// ─────────────────────────────────────────────────────────────────────────────

/// A lightweight task card on a board.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoardCard {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub column: String,
    pub priority: String,
    pub tags: Vec<String>,
    pub due_date: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// A named lightweight board (kanban/resource/calendar surface).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceBoard {
    pub id: String,
    pub name: String,
    /// "kanban" | "calendar" | "gantt" | "resource" | "agile"
    pub kind: String,
    pub columns: Vec<String>,
    pub cards: Vec<BoardCard>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewBoardCard {
    pub board_id: String,
    pub title: String,
    pub description: Option<String>,
    pub column: String,
    pub priority: String,
    pub tags: Vec<String>,
    pub due_date: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Workspace Snapshot (read model)
// ─────────────────────────────────────────────────────────────────────────────

/// Complete immutable read-model of the workspace panel.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceSnapshot {
    pub view: String,
    pub user_hub: Vec<String>,
    pub portfolios: Vec<String>,
    pub content: WorkspaceContentSnapshot,
    pub notebooks: Vec<WorkspaceNotebook>,
    pub contact_books: Vec<WorkspaceContactBook>,
    pub calendars: Vec<String>,
    pub timelines: Vec<String>,
    pub schedules: Vec<WorkspaceScheduleItem>,
    pub boards: Vec<WorkspaceBoard>,
    pub generated_at: DateTime<Utc>,
}

// ─────────────────────────────────────────────────────────────────────────────
// WorkspaceSystem — mutable runtime
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct WorkspaceSystem {
    user_hub: Vec<String>,
    portfolios: Vec<String>,
    content_items: Vec<WorkspaceContentItem>,
    notebooks: Vec<WorkspaceNotebook>,
    contact_books: Vec<WorkspaceContactBook>,
    schedules: Vec<WorkspaceScheduleItem>,
    boards: Vec<WorkspaceBoard>,
    next_content_id: u64,
    next_schedule_id: u64,
    next_notebook_id: u64,
    next_note_id: u64,
    next_contact_book_id: u64,
    next_contact_id: u64,
    next_board_id: u64,
    next_card_id: u64,
}

impl WorkspaceSystem {
    // ── Constructors ──────────────────────────────────────────────────────────

    pub fn mvp() -> Self {
        let now = Utc::now();

        Self {
            user_hub: vec![
                "home feed".to_string(),
                "priorities".to_string(),
                "inbox".to_string(),
                "saved items".to_string(),
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
                    tags: vec!["roadmap".to_string(), "q1".to_string()],
                    created_at: now,
                    updated_at: now,
                },
                WorkspaceContentItem {
                    id: "content-002".to_string(),
                    kind: "document".to_string(),
                    name: "Launch Brief".to_string(),
                    location: "/docs/launch".to_string(),
                    status: "draft".to_string(),
                    tags: vec!["launch".to_string(), "strategy".to_string()],
                    created_at: now,
                    updated_at: now,
                },
                WorkspaceContentItem {
                    id: "content-003".to_string(),
                    kind: "folder".to_string(),
                    name: "Contracts".to_string(),
                    location: "/legal/contracts".to_string(),
                    status: "active".to_string(),
                    tags: vec!["legal".to_string()],
                    created_at: now,
                    updated_at: now,
                },
                WorkspaceContentItem {
                    id: "content-004".to_string(),
                    kind: "document".to_string(),
                    name: "SDD — Kogi Platform".to_string(),
                    location: "/docs/architecture".to_string(),
                    status: "active".to_string(),
                    tags: vec!["architecture".to_string(), "design".to_string()],
                    created_at: now,
                    updated_at: now,
                },
            ],
            notebooks: vec![
                WorkspaceNotebook {
                    id: "nb-001".to_string(),
                    name: "Platform Notes".to_string(),
                    kind: "notebook".to_string(),
                    notes: vec![
                        WorkspaceNote {
                            id: "note-001".to_string(),
                            title: "Architecture Principles".to_string(),
                            body: "Unified, modular, AI-first. Every component is a portfolio item.".to_string(),
                            tags: vec!["architecture".to_string()],
                            created_at: now,
                            updated_at: now,
                        },
                    ],
                    created_at: now,
                },
                WorkspaceNotebook {
                    id: "nb-002".to_string(),
                    name: "Launch Playbook".to_string(),
                    kind: "playbook".to_string(),
                    notes: vec![
                        WorkspaceNote {
                            id: "note-002".to_string(),
                            title: "Phase 1 Checklist".to_string(),
                            body: "Auth, Portfolio MVP, Project MVP, Chat, AI recommendations baseline.".to_string(),
                            tags: vec!["launch".to_string(), "phase1".to_string()],
                            created_at: now,
                            updated_at: now,
                        },
                    ],
                    created_at: now,
                },
            ],
            contact_books: vec![
                WorkspaceContactBook {
                    id: "cb-001".to_string(),
                    name: "Professional Contacts".to_string(),
                    kind: "professional".to_string(),
                    contacts: vec![
                        WorkspaceContact {
                            id: "contact-001".to_string(),
                            name: "Alex Rivera".to_string(),
                            role: Some("Product Partner".to_string()),
                            organization: Some("Acme Labs".to_string()),
                            channels: vec!["email: alex@acmelabs.io".to_string()],
                            tags: vec!["partner".to_string()],
                            created_at: now,
                        },
                    ],
                    created_at: now,
                },
            ],
            schedules: vec![
                WorkspaceScheduleItem {
                    id: "schedule-001".to_string(),
                    title: "Weekly Planning".to_string(),
                    kind: "calendar".to_string(),
                    scheduled_for: "2026-03-13T14:00:00Z".to_string(),
                    description: Some("Weekly priorities and OKR review".to_string()),
                    status: "scheduled".to_string(),
                },
                WorkspaceScheduleItem {
                    id: "schedule-002".to_string(),
                    title: "Launch 2026 Milestone Review".to_string(),
                    kind: "milestone".to_string(),
                    scheduled_for: "2026-03-31T09:00:00Z".to_string(),
                    description: Some("Phase 1 delivery gate".to_string()),
                    status: "scheduled".to_string(),
                },
            ],
            boards: vec![
                WorkspaceBoard {
                    id: "board-001".to_string(),
                    name: "Sprint Board".to_string(),
                    kind: "kanban".to_string(),
                    columns: vec![
                        "backlog".to_string(),
                        "in-progress".to_string(),
                        "review".to_string(),
                        "done".to_string(),
                    ],
                    cards: vec![
                        BoardCard {
                            id: "card-001".to_string(),
                            title: "Finalize DashboardSystem API".to_string(),
                            description: Some("Add KPIs and AI suggestions".to_string()),
                            column: "in-progress".to_string(),
                            priority: "high".to_string(),
                            tags: vec!["engineering".to_string()],
                            due_date: Some("2026-03-14".to_string()),
                            created_at: now,
                        },
                        BoardCard {
                            id: "card-002".to_string(),
                            title: "Portfolio system integration tests".to_string(),
                            description: None,
                            column: "backlog".to_string(),
                            priority: "medium".to_string(),
                            tags: vec!["engineering".to_string(), "testing".to_string()],
                            due_date: None,
                            created_at: now,
                        },
                    ],
                    created_at: now,
                },
            ],
            next_content_id: 5,
            next_schedule_id: 3,
            next_notebook_id: 3,
            next_note_id: 3,
            next_contact_book_id: 2,
            next_contact_id: 2,
            next_board_id: 2,
            next_card_id: 3,
        }
    }

    // ── Read ──────────────────────────────────────────────────────────────────

    pub fn snapshot(&self) -> WorkspaceSnapshot {
        WorkspaceSnapshot {
            view: "home.workspace".to_string(),
            user_hub: self.user_hub.clone(),
            portfolios: self.portfolios.clone(),
            content: build_content_snapshot(&self.content_items),
            notebooks: self.notebooks.clone(),
            contact_books: self.contact_books.clone(),
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
            boards: self.boards.clone(),
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
            generated_at: Utc::now(),
        }
    }

    pub fn find_content(&self, id: &str) -> Option<&WorkspaceContentItem> {
        self.content_items.iter().find(|c| c.id == id)
    }

    pub fn find_schedule(&self, id: &str) -> Option<&WorkspaceScheduleItem> {
        self.schedules.iter().find(|s| s.id == id)
    }

    // ── Content ───────────────────────────────────────────────────────────────

    pub fn add_content(&mut self, request: NewWorkspaceContent) -> WorkspaceContentItem {
        let now = Utc::now();
        let item = WorkspaceContentItem {
            id: format!("content-{:03}", self.next_content_id),
            kind: request.kind,
            name: request.name,
            location: request.location,
            status: "active".to_string(),
            tags: request.tags,
            created_at: now,
            updated_at: now,
        };
        self.next_content_id += 1;
        self.content_items.push(item.clone());
        item
    }

    pub fn update_content_status(&mut self, id: &str, status: String) -> bool {
        if let Some(item) = self.content_items.iter_mut().find(|c| c.id == id) {
            item.status = status;
            item.updated_at = Utc::now();
            return true;
        }
        false
    }

    pub fn archive_content(&mut self, id: &str) -> bool {
        self.update_content_status(id, "archived".to_string())
    }

    // ── Schedules ─────────────────────────────────────────────────────────────

    pub fn add_schedule(&mut self, request: NewWorkspaceSchedule) -> WorkspaceScheduleItem {
        let item = WorkspaceScheduleItem {
            id: format!("schedule-{:03}", self.next_schedule_id),
            title: request.title,
            kind: request.kind,
            scheduled_for: request.scheduled_for,
            description: request.description,
            status: "scheduled".to_string(),
        };
        self.next_schedule_id += 1;
        self.schedules.push(item.clone());
        item
    }

    pub fn complete_schedule_item(&mut self, id: &str) -> bool {
        if let Some(s) = self.schedules.iter_mut().find(|s| s.id == id) {
            s.status = "completed".to_string();
            return true;
        }
        false
    }

    // ── Notebooks ─────────────────────────────────────────────────────────────

    pub fn add_notebook(&mut self, request: NewWorkspaceNotebook) -> WorkspaceNotebook {
        let nb = WorkspaceNotebook {
            id: format!("nb-{:03}", self.next_notebook_id),
            name: request.name,
            kind: request.kind,
            notes: vec![],
            created_at: Utc::now(),
        };
        self.next_notebook_id += 1;
        self.notebooks.push(nb.clone());
        nb
    }

    pub fn add_note(&mut self, request: NewWorkspaceNote) -> Option<WorkspaceNote> {
        let now = Utc::now();
        if let Some(nb) = self.notebooks.iter_mut().find(|n| n.id == request.notebook_id) {
            let note = WorkspaceNote {
                id: format!("note-{:03}", self.next_note_id),
                title: request.title,
                body: request.body,
                tags: request.tags,
                created_at: now,
                updated_at: now,
            };
            self.next_note_id += 1;
            nb.notes.push(note.clone());
            return Some(note);
        }
        None
    }

    // ── Contact Books ─────────────────────────────────────────────────────────

    pub fn add_contact_book(&mut self, name: String, kind: String) -> WorkspaceContactBook {
        let cb = WorkspaceContactBook {
            id: format!("cb-{:03}", self.next_contact_book_id),
            name,
            kind,
            contacts: vec![],
            created_at: Utc::now(),
        };
        self.next_contact_book_id += 1;
        self.contact_books.push(cb.clone());
        cb
    }

    pub fn add_contact(&mut self, request: NewWorkspaceContact) -> Option<WorkspaceContact> {
        if let Some(cb) = self.contact_books.iter_mut().find(|b| b.id == request.contact_book_id) {
            let contact = WorkspaceContact {
                id: format!("contact-{:03}", self.next_contact_id),
                name: request.name,
                role: request.role,
                organization: request.organization,
                channels: request.channels,
                tags: request.tags,
                created_at: Utc::now(),
            };
            self.next_contact_id += 1;
            cb.contacts.push(contact.clone());
            return Some(contact);
        }
        None
    }

    // ── Boards ────────────────────────────────────────────────────────────────

    pub fn add_board(&mut self, name: String, kind: String, columns: Vec<String>) -> WorkspaceBoard {
        let board = WorkspaceBoard {
            id: format!("board-{:03}", self.next_board_id),
            name,
            kind,
            columns,
            cards: vec![],
            created_at: Utc::now(),
        };
        self.next_board_id += 1;
        self.boards.push(board.clone());
        board
    }

    pub fn add_board_card(&mut self, request: NewBoardCard) -> Option<BoardCard> {
        if let Some(board) = self.boards.iter_mut().find(|b| b.id == request.board_id) {
            let card = BoardCard {
                id: format!("card-{:03}", self.next_card_id),
                title: request.title,
                description: request.description,
                column: request.column,
                priority: request.priority,
                tags: request.tags,
                due_date: request.due_date,
                created_at: Utc::now(),
            };
            self.next_card_id += 1;
            board.cards.push(card.clone());
            return Some(card);
        }
        None
    }

    pub fn move_card(&mut self, board_id: &str, card_id: &str, column: String) -> bool {
        if let Some(board) = self.boards.iter_mut().find(|b| b.id == board_id) {
            if let Some(card) = board.cards.iter_mut().find(|c| c.id == card_id) {
                card.column = column;
                return true;
            }
        }
        false
    }

    // ── User Hub ──────────────────────────────────────────────────────────────

    pub fn add_portfolio_link(&mut self, portfolio: String) -> bool {
        if self.portfolios.contains(&portfolio) {
            return false;
        }
        self.portfolios.push(portfolio);
        true
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

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

    WorkspaceContentSnapshot { files, documents, folders }
}
