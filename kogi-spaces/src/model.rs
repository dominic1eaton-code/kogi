use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpaceStatus {
    Draft,
    Active,
    Paused,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpaceVisibility {
    Public,
    Community,
    InviteOnly,
    Private,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpaceKind {
    Personal,
    Team,
    Community,
    Organization,
    Workspace,
    Room,
    Channel,
    Event,
    Directory,
    Network,
    LinkNet,
    LinkTree,
    LinkForest,
    Hub,
    Guild,
    Collective,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpaceMemberRole {
    Owner,
    Admin,
    Moderator,
    Editor,
    Member,
    Viewer,
    Guest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpaceMemberStatus {
    Active,
    Invited,
    Pending,
    Suspended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceMember {
    pub user_id: Uuid,
    pub role: SpaceMemberRole,
    pub status: SpaceMemberStatus,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceMetrics {
    pub members: u32,
    pub rooms: u32,
    pub channels: u32,
    pub events: u32,
    pub workspaces: u32,
    pub posts: u32,
    pub followers: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceAnalytics {
    pub views: u32,
    pub clicks: u32,
    pub shares: u32,
    pub engagement: f64,
    pub health_score: f64,
    pub risk_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Space {
    pub id: Uuid,
    pub kind: SpaceKind,
    pub portfolio_component_id: Option<Uuid>,
    pub owner_id: Uuid,
    pub name: String,
    pub description: String,
    pub status: SpaceStatus,
    pub visibility: SpaceVisibility,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub tags: Vec<String>,
    pub topics: Vec<String>,
    pub governance_model: Option<String>,
    pub members: Vec<SpaceMember>,
    pub rooms: Vec<Uuid>,
    pub channels: Vec<Uuid>,
    pub events: Vec<Uuid>,
    pub workspaces: Vec<Uuid>,
    pub metrics: SpaceMetrics,
    pub analytics: SpaceAnalytics,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpaceResourceType {
    Room,
    Channel,
    Event,
    Workspace,
    Feed,
    Timeline,
    Directory,
    Network,
    Registry,
    LinkNet,
    LinkTree,
    LinkForest,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceResource {
    pub id: Uuid,
    pub space_id: Option<Uuid>,
    pub resource_type: SpaceResourceType,
    pub name: String,
    pub description: String,
    pub status: SpaceStatus,
    pub visibility: SpaceVisibility,
    pub tags: Vec<String>,
    pub topics: Vec<String>,
    pub members: Vec<Uuid>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
