use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use kogi_portfolio::{KogiPortfolioConfig, PortfolioRow, PortfolioSystem};

use crate::error::{SpacesError, SpacesResult};
use crate::model::*;

#[derive(Debug, Clone)]
pub struct SpacesConfig {
    pub portfolio_config: KogiPortfolioConfig,
    pub default_visibility: SpaceVisibility,
    pub default_kind: SpaceKind,
    pub spaces_id: Uuid,
}

impl Default for SpacesConfig {
    fn default() -> Self {
        Self {
            portfolio_config: KogiPortfolioConfig::default(),
            default_visibility: SpaceVisibility::Community,
            default_kind: SpaceKind::Community,
            spaces_id: Uuid::new_v4(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacesState {
    pub spaces_id: Uuid,
    pub portfolio_root_component_id: Uuid,
    pub workbook_id: Uuid,
    pub space_count: usize,
    pub room_count: usize,
    pub channel_count: usize,
    pub event_count: usize,
    pub workspace_count: usize,
    pub last_refresh_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishSpaceRequest {
    pub portfolio_component_id: String,
    pub kind: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub visibility: Option<String>,
    pub tags: Option<Vec<String>>,
    pub topics: Option<Vec<String>>,
    pub governance_model: Option<String>,
    pub members: Option<Vec<String>>,
    pub rooms: Option<Vec<String>>,
    pub channels: Option<Vec<String>>,
    pub events: Option<Vec<String>>,
    pub workspaces: Option<Vec<String>>,
    pub metadata: Option<Value>,
}

pub struct SpacesSystem {
    pub config: SpacesConfig,
    pub portfolio: PortfolioSystem,
    manual_spaces: HashMap<Uuid, Space>,
    last_refresh_at: DateTime<Utc>,
}

impl SpacesSystem {
    pub fn new(config: SpacesConfig) -> SpacesResult<Self> {
        let portfolio = PortfolioSystem::new(config.portfolio_config.clone())?;
        Ok(Self {
            config,
            portfolio,
            manual_spaces: HashMap::new(),
            last_refresh_at: Utc::now(),
        })
    }

    pub fn state(&self) -> SpacesState {
        let spaces = self.spaces();
        let rooms = self.rooms();
        let channels = self.channels();
        let events = self.events();
        let workspaces = self.workspaces();
        SpacesState {
            spaces_id: self.config.spaces_id,
            portfolio_root_component_id: self.portfolio.root_component_id(),
            workbook_id: self.portfolio.state().workbook_id,
            space_count: spaces.len(),
            room_count: rooms.len(),
            channel_count: channels.len(),
            event_count: events.len(),
            workspace_count: workspaces.len(),
            last_refresh_at: self.last_refresh_at,
        }
    }

    pub fn refresh(&mut self) -> SpacesResult<()> {
        self.portfolio.sync_from_grid()?;
        self.portfolio.persist()?;
        self.last_refresh_at = Utc::now();
        Ok(())
    }

    pub fn spaces(&self) -> Vec<Space> {
        let rows = self.portfolio.runtime.master.workbook.rows_for_sheet("SHT-024");
        let mut space_map: HashMap<Uuid, Space> = HashMap::new();

        let mut room_map: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        let mut channel_map: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        let mut event_map: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        let mut workspace_map: HashMap<Uuid, Vec<Uuid>> = HashMap::new();

        for row in &rows {
            if let Some(parent) = row.parent_id {
                if is_room_row(row) {
                    room_map.entry(parent).or_default().push(row.component_id);
                }
                if is_channel_row(row) {
                    channel_map.entry(parent).or_default().push(row.component_id);
                }
                if is_event_row(row) {
                    event_map.entry(parent).or_default().push(row.component_id);
                }
                if is_workspace_row(row) {
                    workspace_map.entry(parent).or_default().push(row.component_id);
                }
            }
        }

        for row in rows {
            if !is_space_row(row) {
                continue;
            }
            let space = space_from_row(
                row,
                &self.config,
                room_map.get(&row.component_id),
                channel_map.get(&row.component_id),
                event_map.get(&row.component_id),
                workspace_map.get(&row.component_id),
            );
            space_map.insert(space.id, space);
        }

        for (id, space) in &self.manual_spaces {
            space_map.insert(*id, space.clone());
        }

        let mut spaces: Vec<Space> = space_map.into_values().collect();
        spaces.sort_by_key(|s| s.updated_at);
        spaces
    }

    pub fn space_by_id(&self, space_id: Uuid) -> Option<Space> {
        if let Some(space) = self.manual_spaces.get(&space_id) {
            return Some(space.clone());
        }
        self.portfolio
            .runtime
            .master
            .workbook
            .get_row(&space_id)
            .filter(|row| is_space_row(row))
            .map(|row| space_from_row(row, &self.config, None, None, None, None))
    }

    pub fn spaces_for_owner(&self, owner_id: Uuid) -> Vec<Space> {
        self.spaces()
            .into_iter()
            .filter(|space| space.owner_id == owner_id)
            .collect()
    }

    pub fn members(&self, space_id: Uuid) -> Vec<SpaceMember> {
        self.space_by_id(space_id)
            .map(|space| space.members)
            .unwrap_or_default()
    }

    pub fn rooms(&self) -> Vec<SpaceResource> {
        self.resources_by_predicate(is_room_row, SpaceResourceType::Room)
    }

    pub fn channels(&self) -> Vec<SpaceResource> {
        self.resources_by_predicate(is_channel_row, SpaceResourceType::Channel)
    }

    pub fn events(&self) -> Vec<SpaceResource> {
        self.resources_by_predicate(is_event_row, SpaceResourceType::Event)
    }

    pub fn workspaces(&self) -> Vec<SpaceResource> {
        self.resources_by_predicate(is_workspace_row, SpaceResourceType::Workspace)
    }

    pub fn publish_space(&mut self, req: PublishSpaceRequest) -> SpacesResult<Space> {
        let component_id = parse_uuid(&req.portfolio_component_id)?;
        let row = self
            .portfolio
            .runtime
            .master
            .workbook
            .get_row(&component_id)
            .ok_or_else(|| SpacesError::NotFound("portfolio component not found".to_string()))?;

        let mut space = space_from_row(row, &self.config, None, None, None, None);
        if let Some(kind) = req.kind {
            space.kind = space_kind_from_str(&kind);
        }
        if let Some(name) = req.name {
            space.name = name;
        }
        if let Some(description) = req.description {
            space.description = description;
        }
        if let Some(status) = req.status {
            space.status = space_status_from_str(&status);
        }
        if let Some(visibility) = req.visibility {
            space.visibility = space_visibility_from_str(&visibility);
        }
        if let Some(tags) = req.tags {
            space.tags = tags;
        }
        if let Some(topics) = req.topics {
            space.topics = topics;
        }
        if let Some(governance_model) = req.governance_model {
            space.governance_model = Some(governance_model);
        }
        if let Some(members) = req.members {
            space.members = members
                .iter()
                .filter_map(|id| Uuid::parse_str(id).ok())
                .map(|id| SpaceMember {
                    user_id: id,
                    role: SpaceMemberRole::Member,
                    status: SpaceMemberStatus::Active,
                    joined_at: Utc::now(),
                })
                .collect();
            space.metrics.members = space.members.len() as u32;
        }
        if let Some(rooms) = req.rooms {
            space.rooms = parse_uuid_list(&rooms);
            space.metrics.rooms = space.rooms.len() as u32;
        }
        if let Some(channels) = req.channels {
            space.channels = parse_uuid_list(&channels);
            space.metrics.channels = space.channels.len() as u32;
        }
        if let Some(events) = req.events {
            space.events = parse_uuid_list(&events);
            space.metrics.events = space.events.len() as u32;
        }
        if let Some(workspaces) = req.workspaces {
            space.workspaces = parse_uuid_list(&workspaces);
            space.metrics.workspaces = space.workspaces.len() as u32;
        }
        if let Some(metadata) = req.metadata {
            space.metadata = metadata;
        }

        self.manual_spaces.insert(space.id, space.clone());
        Ok(space)
    }

    fn resources_by_predicate(
        &self,
        predicate: fn(&PortfolioRow) -> bool,
        resource_type: SpaceResourceType,
    ) -> Vec<SpaceResource> {
        let rows = self.portfolio.runtime.master.workbook.rows_for_sheet("SHT-024");
        let mut resources: Vec<SpaceResource> = rows
            .into_iter()
            .filter(|row| predicate(row))
            .map(|row| resource_from_row(row, &resource_type))
            .collect();
        resources.sort_by_key(|r| r.updated_at);
        resources
    }
}

fn parse_uuid(value: &str) -> SpacesResult<Uuid> {
    Uuid::parse_str(value)
        .map_err(|_| SpacesError::Invalid(format!("invalid uuid: {value}")))
}

fn parse_uuid_list(values: &[String]) -> Vec<Uuid> {
    values
        .iter()
        .filter_map(|v| Uuid::parse_str(v).ok())
        .collect()
}

fn is_space_row(row: &PortfolioRow) -> bool {
    let primary = row.primary_type();
    let type_match = primary
        .as_deref()
        .map(matches_space_type)
        .unwrap_or(false);
    let resource_match = primary
        .as_deref()
        .map(matches_resource_type)
        .unwrap_or(false);
    if resource_match {
        return false;
    }
    let has_tag = row.tags.iter().any(is_space_tag) || row.topics.iter().any(is_space_tag);
    let has_meta = row.ext.contains_key("space")
        || row.ext.contains_key("spaces")
        || row.ext.contains_key("kogi_space")
        || row.ext.contains_key("space_profile");
    type_match || has_tag || has_meta
}

fn is_room_row(row: &PortfolioRow) -> bool {
    let type_match = row
        .primary_type()
        .as_deref()
        .map(matches_room_type)
        .unwrap_or(false);
    let has_tag = row.tags.iter().any(is_room_tag) || row.topics.iter().any(is_room_tag);
    type_match || has_tag
}

fn is_channel_row(row: &PortfolioRow) -> bool {
    let type_match = row
        .primary_type()
        .as_deref()
        .map(matches_channel_type)
        .unwrap_or(false);
    let has_tag = row.tags.iter().any(is_channel_tag) || row.topics.iter().any(is_channel_tag);
    type_match || has_tag
}

fn is_event_row(row: &PortfolioRow) -> bool {
    let type_match = row
        .primary_type()
        .as_deref()
        .map(matches_event_type)
        .unwrap_or(false);
    let has_tag = row.tags.iter().any(is_event_tag) || row.topics.iter().any(is_event_tag);
    type_match || has_tag
}

fn is_workspace_row(row: &PortfolioRow) -> bool {
    let type_match = row
        .primary_type()
        .as_deref()
        .map(matches_workspace_type)
        .unwrap_or(false);
    let has_tag = row.tags.iter().any(is_workspace_tag) || row.topics.iter().any(is_workspace_tag);
    type_match || has_tag
}

fn matches_space_type(raw: &str) -> bool {
    matches!(
        raw.to_lowercase().as_str(),
        "space"
            | "community"
            | "collective"
            | "guild"
            | "hub"
            | "network"
            | "organization"
            | "org"
            | "team"
            | "squad"
            | "cooperative"
            | "coop"
    )
}

fn matches_resource_type(raw: &str) -> bool {
    matches_room_type(raw)
        || matches_channel_type(raw)
        || matches_event_type(raw)
        || matches_workspace_type(raw)
}

fn matches_room_type(raw: &str) -> bool {
    matches!(
        raw.to_lowercase().as_str(),
        "room" | "chat" | "meeting" | "voice" | "video" | "lounge" | "stage"
    )
}

fn matches_channel_type(raw: &str) -> bool {
    matches!(
        raw.to_lowercase().as_str(),
        "channel" | "feed" | "timeline" | "broadcast" | "announcements"
    )
}

fn matches_event_type(raw: &str) -> bool {
    matches!(
        raw.to_lowercase().as_str(),
        "event" | "calendar" | "session" | "workshop" | "webinar"
    )
}

fn matches_workspace_type(raw: &str) -> bool {
    matches!(raw.to_lowercase().as_str(), "workspace" | "workspaces")
}

fn is_space_tag(tag: &str) -> bool {
    matches!(
        tag.to_lowercase().as_str(),
        "space" | "spaces" | "community" | "org" | "organization" | "team" | "collective" | "guild"
    )
}

fn is_room_tag(tag: &str) -> bool {
    matches!(
        tag.to_lowercase().as_str(),
        "room" | "rooms" | "chat" | "voice" | "video"
    )
}

fn is_channel_tag(tag: &str) -> bool {
    matches!(
        tag.to_lowercase().as_str(),
        "channel" | "channels" | "feed" | "timeline" | "announcements" | "broadcast"
    )
}

fn is_event_tag(tag: &str) -> bool {
    matches!(
        tag.to_lowercase().as_str(),
        "event" | "events" | "calendar" | "session" | "workshop" | "webinar"
    )
}

fn is_workspace_tag(tag: &str) -> bool {
    matches!(tag.to_lowercase().as_str(), "workspace" | "workspaces")
}

fn space_from_row(
    row: &PortfolioRow,
    config: &SpacesConfig,
    rooms: Option<&Vec<Uuid>>,
    channels: Option<&Vec<Uuid>>,
    events: Option<&Vec<Uuid>>,
    workspaces: Option<&Vec<Uuid>>,
) -> Space {
    let owner_id = row.owners.first().copied().unwrap_or(row.created_by);
    let name = row
        .display_name
        .clone()
        .unwrap_or_else(|| row.name.clone());
    let description = row
        .ext
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let members = members_from_row(row);
    let rooms = rooms.cloned().unwrap_or_default();
    let channels = channels.cloned().unwrap_or_default();
    let events = events.cloned().unwrap_or_default();
    let workspaces = workspaces.cloned().unwrap_or_default();
    let mut space = Space {
        id: row.component_id,
        kind: space_kind_from_row(row, config),
        portfolio_component_id: Some(row.component_id),
        owner_id,
        name,
        description,
        status: space_status_from_str(&row.status),
        visibility: space_visibility_from_str(&row.visibility),
        icon: row.icon.clone(),
        color: row.color.clone(),
        tags: row.tags.clone(),
        topics: row.topics.clone(),
        governance_model: row.governance_model.clone(),
        members: members.clone(),
        rooms: rooms.clone(),
        channels: channels.clone(),
        events: events.clone(),
        workspaces: workspaces.clone(),
        metrics: SpaceMetrics {
            members: members.len() as u32,
            rooms: rooms.len() as u32,
            channels: channels.len() as u32,
            events: events.len() as u32,
            workspaces: workspaces.len() as u32,
            posts: count_from_value(row.ext.get("posts")).unwrap_or_default(),
            followers: row.followers.min(u32::MAX as u64) as u32,
        },
        analytics: analytics_from_row(row),
        metadata: json!({ "portfolio_row": row }),
        created_at: row.created_at,
        updated_at: row.updated_at,
    };

    apply_space_overrides(&mut space, row);
    if matches!(space.visibility, SpaceVisibility::Public | SpaceVisibility::Community | SpaceVisibility::InviteOnly | SpaceVisibility::Private) {
        // keep visibility
    } else {
        space.visibility = config.default_visibility.clone();
    }

    space
}

fn space_kind_from_row(row: &PortfolioRow, config: &SpacesConfig) -> SpaceKind {
    row.primary_type()
        .as_deref()
        .map(space_kind_from_str)
        .unwrap_or_else(|| config.default_kind.clone())
}

fn space_kind_from_str(raw: &str) -> SpaceKind {
    match raw.to_lowercase().as_str() {
        "personal" => SpaceKind::Personal,
        "team" | "squad" => SpaceKind::Team,
        "organization" | "org" | "company" => SpaceKind::Organization,
        "workspace" | "workspaces" => SpaceKind::Workspace,
        "room" | "chat" | "voice" | "video" => SpaceKind::Room,
        "channel" | "feed" | "timeline" => SpaceKind::Channel,
        "event" | "session" | "workshop" | "webinar" => SpaceKind::Event,
        "directory" => SpaceKind::Directory,
        "network" => SpaceKind::Network,
        "linknet" => SpaceKind::LinkNet,
        "linktree" => SpaceKind::LinkTree,
        "linkforest" => SpaceKind::LinkForest,
        "hub" => SpaceKind::Hub,
        "guild" => SpaceKind::Guild,
        "collective" | "cooperative" | "coop" => SpaceKind::Collective,
        "community" | "space" => SpaceKind::Community,
        _ => SpaceKind::Custom,
    }
}

fn space_status_from_str(raw: &str) -> SpaceStatus {
    match raw.to_lowercase().as_str() {
        "active" => SpaceStatus::Active,
        "paused" => SpaceStatus::Paused,
        "archived" => SpaceStatus::Archived,
        _ => SpaceStatus::Draft,
    }
}

fn space_visibility_from_str(raw: &str) -> SpaceVisibility {
    match raw.to_lowercase().as_str() {
        "public" => SpaceVisibility::Public,
        "community" | "protected" => SpaceVisibility::Community,
        "inviteonly" | "invite_only" | "invite-only" | "unlisted" => SpaceVisibility::InviteOnly,
        "private" => SpaceVisibility::Private,
        _ => SpaceVisibility::Community,
    }
}

fn analytics_from_row(row: &PortfolioRow) -> SpaceAnalytics {
    let views = row.views.min(u32::MAX as u64) as u32;
    let clicks = row.clicks.min(u32::MAX as u64) as u32;
    let shares = row.shares.min(u32::MAX as u64) as u32;
    let followers = row.followers.min(u32::MAX as u64) as u32;
    let engagement = if views > 0 {
        (shares + followers) as f64 / views as f64
    } else {
        0.0
    };
    SpaceAnalytics {
        views,
        clicks,
        shares,
        engagement,
        health_score: float_from_value(row.ext.get("health_score")).unwrap_or(0.0),
        risk_score: float_from_value(row.ext.get("risk_score")).unwrap_or(0.0),
    }
}

fn members_from_row(row: &PortfolioRow) -> Vec<SpaceMember> {
    let mut map: HashMap<Uuid, SpaceMember> = HashMap::new();
    let joined_at = row.created_at;
    insert_member(&mut map, &row.owners, SpaceMemberRole::Owner, joined_at);
    insert_member(&mut map, &row.editors, SpaceMemberRole::Admin, joined_at);
    insert_member(&mut map, &row.contributors, SpaceMemberRole::Member, joined_at);
    insert_member(&mut map, &row.viewers, SpaceMemberRole::Viewer, joined_at);
    insert_member(&mut map, &row.watchers, SpaceMemberRole::Guest, joined_at);
    map.into_values().collect()
}

fn insert_member(
    map: &mut HashMap<Uuid, SpaceMember>,
    ids: &[Uuid],
    role: SpaceMemberRole,
    joined_at: DateTime<Utc>,
) {
    for id in ids {
        let entry = map.entry(*id).or_insert(SpaceMember {
            user_id: *id,
            role: role.clone(),
            status: SpaceMemberStatus::Active,
            joined_at,
        });
        if role_rank(&role) < role_rank(&entry.role) {
            entry.role = role.clone();
        }
    }
}

fn role_rank(role: &SpaceMemberRole) -> u8 {
    match role {
        SpaceMemberRole::Owner => 0,
        SpaceMemberRole::Admin => 1,
        SpaceMemberRole::Moderator => 2,
        SpaceMemberRole::Editor => 3,
        SpaceMemberRole::Member => 4,
        SpaceMemberRole::Viewer => 5,
        SpaceMemberRole::Guest => 6,
    }
}

fn resource_from_row(row: &PortfolioRow, resource_type: &SpaceResourceType) -> SpaceResource {
    let space_id = row
        .parent_id
        .or_else(|| uuid_from_value(row.ext.get("space_id")));
    let name = row
        .display_name
        .clone()
        .unwrap_or_else(|| row.name.clone());
    let description = row
        .ext
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    SpaceResource {
        id: row.component_id,
        space_id,
        resource_type: resource_type.clone(),
        name,
        description,
        status: space_status_from_str(&row.status),
        visibility: space_visibility_from_str(&row.visibility),
        tags: row.tags.clone(),
        topics: row.topics.clone(),
        members: unique_ids(&row.owners, &row.editors, &row.contributors, &row.viewers),
        metadata: json!({ "portfolio_row": row }),
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn unique_ids(groups: &[Uuid], a: &[Uuid], b: &[Uuid], c: &[Uuid]) -> Vec<Uuid> {
    let mut set = HashSet::new();
    for list in [groups, a, b, c] {
        for id in list {
            set.insert(*id);
        }
    }
    set.into_iter().collect()
}

fn apply_space_overrides(space: &mut Space, row: &PortfolioRow) {
    let meta = row
        .ext
        .get("space")
        .or_else(|| row.ext.get("spaces"))
        .or_else(|| row.ext.get("kogi_space"));
    let Some(meta) = meta else { return; };
    let Some(obj) = meta.as_object() else { return; };

    if let Some(value) = obj.get("kind").and_then(|v| v.as_str()) {
        space.kind = space_kind_from_str(value);
    }
    if let Some(value) = obj.get("name").and_then(|v| v.as_str()) {
        space.name = value.to_string();
    }
    if let Some(value) = obj.get("description").and_then(|v| v.as_str()) {
        space.description = value.to_string();
    }
    if let Some(value) = obj.get("status").and_then(|v| v.as_str()) {
        space.status = space_status_from_str(value);
    }
    if let Some(value) = obj.get("visibility").and_then(|v| v.as_str()) {
        space.visibility = space_visibility_from_str(value);
    }
    if let Some(value) = obj.get("tags") {
        space.tags = list_from_value(Some(value));
    }
    if let Some(value) = obj.get("topics") {
        space.topics = list_from_value(Some(value));
    }
    if let Some(value) = obj.get("governance_model").and_then(|v| v.as_str()) {
        space.governance_model = Some(value.to_string());
    }
    if let Some(value) = obj.get("rooms") {
        space.rooms = uuid_list_from_value(Some(value));
    }
    if let Some(value) = obj.get("channels") {
        space.channels = uuid_list_from_value(Some(value));
    }
    if let Some(value) = obj.get("events") {
        space.events = uuid_list_from_value(Some(value));
    }
    if let Some(value) = obj.get("workspaces") {
        space.workspaces = uuid_list_from_value(Some(value));
    }
}

fn uuid_from_value(value: Option<&Value>) -> Option<Uuid> {
    let value = value?;
    match value {
        Value::String(s) => Uuid::parse_str(s).ok(),
        _ => None,
    }
}

fn float_from_value(value: Option<&Value>) -> Option<f64> {
    let value = value?;
    match value {
        Value::Number(num) => num.as_f64(),
        Value::String(s) => s.parse::<f64>().ok(),
        _ => None,
    }
}

fn list_from_value(value: Option<&Value>) -> Vec<String> {
    let Some(value) = value else { return vec![]; };
    match value {
        Value::Array(items) => items
            .iter()
            .filter_map(|item| item.as_str().map(|s| s.to_string()))
            .collect(),
        Value::String(s) => s
            .split(',')
            .map(|item| item.trim())
            .filter(|item| !item.is_empty())
            .map(|item| item.to_string())
            .collect(),
        _ => vec![],
    }
}

fn uuid_list_from_value(value: Option<&Value>) -> Vec<Uuid> {
    let Some(value) = value else { return vec![]; };
    match value {
        Value::Array(items) => items
            .iter()
            .filter_map(|item| item.as_str().and_then(|s| Uuid::parse_str(s).ok()))
            .collect(),
        _ => vec![],
    }
}

fn count_from_value(value: Option<&Value>) -> Option<u32> {
    let value = value?;
    match value {
        Value::Number(num) => num.as_u64().map(|v| v.min(u32::MAX as u64) as u32),
        Value::String(s) => s.parse::<u32>().ok(),
        Value::Array(items) => Some(items.len() as u32),
        _ => None,
    }
}
