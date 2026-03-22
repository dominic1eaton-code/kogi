//! Search & Query — SearchQuery, PortfolioQuery (PQL), SearchResult.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;


use crate::types::{ComponentCategory, ComponentId, ComponentStatus, UserId, Visibility};

/// Full-featured search query over the component store.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: Option<String>,
    pub tags: Vec<String>,
    pub hashtags: Vec<String>,
    pub categories: Vec<ComponentCategory>,
    pub statuses: Vec<ComponentStatus>,
    pub owner: Option<UserId>,
    pub visibility: Option<Visibility>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub limit: usize,
    pub offset: usize,
}

impl SearchQuery {
    pub fn new() -> Self {
        Self { limit: 50, ..Default::default() }
    }

    pub fn text(mut self, q: impl Into<String>) -> Self {
        self.text = Some(q.into()); self
    }

    pub fn status(mut self, s: ComponentStatus) -> Self {
        self.statuses.push(s); self
    }

    pub fn owner(mut self, id: UserId) -> Self {
        self.owner = Some(id); self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit = n; self
    }
}

/// Structured predicate model for the Portfolio Query Language.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PortfolioQuery {
    pub categories: Vec<ComponentCategory>,
    pub statuses: Vec<ComponentStatus>,
    pub tags: Vec<String>,
    pub name_contains: Option<String>,
    pub owner: Option<UserId>,
    pub has_dependencies: Option<bool>,
    pub has_children: Option<bool>,
    pub property_filter: Option<(String, JsonValue)>,
}

/// A single search result entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub component_id: ComponentId,
    pub name: String,
    pub category: ComponentCategory,
    pub status: ComponentStatus,
    /// Relevance float (higher = more relevant).
    pub score: f64,
    /// Which fields matched the query.
    pub matched_fields: Vec<String>,
}
