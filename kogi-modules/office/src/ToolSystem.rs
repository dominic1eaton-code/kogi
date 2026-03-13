// =============================================================================
//  tool_management_system.rs — Kogi OS · Tool Management System  v1.0
//  Independent Worker Operating System
//
//  This file is the single authoritative source for the Kogi Tool domain.
//  It provides the full layered architecture for tools, toolkits, toolchains,
//  toolsets, toolboxes, integrations, and the automation/orchestration engine.
//
//  ─────────────────────────────────────────────────────────────────────────────
//  Structural Hierarchy
//  ─────────────────────────────────────────────────────────────────────────────
//
//    ToolManagementSystem
//      ├── Tool
//      │     ├── ToolAssembly   (blueprint)
//      │     │     ├── ToolData      (low-level: config, params, schema)
//      │     │     ├── ToolInfo      (high-level: name, description, docs)
//      │     │     └── ToolMetadata  (id, tags, labels, tokens, provider ref,
//      │     │                        version control, categories, types, class)
//      │     ├── ToolProvider   (root | 3rd-party, links to provider.rs)
//      │     └── ToolIntegration (tool + provider bound to kogi platform target)
//      │
//      ├── ToolKit    (unordered collection/grouping of tools)
//      ├── ToolChain  (ordered pipeline/sequence of connected tools)
//      ├── ToolSet    (template: toolkits + toolchains + ungrouped tools)
//      ├── ToolBox    (container for toolsets; attached to portfolio elements)
//      │
//      └── ToolAutomationSystem
//            ├── ToolOrchestration  (top-level automation: N workflows)
//            ├── ToolWorkflow       (series of executable tasks)
//            └── ToolTask           (atomic executable unit)
//
//  ─────────────────────────────────────────────────────────────────────────────
//  Example
//  ─────────────────────────────────────────────────────────────────────────────
//
//    A CMT (code management tool) with a GitHub provider becomes a
//    ToolIntegration. It lives in a "devops tools" ToolSet inside a
//    "software development" ToolBox. Within the ToolSet it belongs to a
//    "code management" ToolKit and is a step in the "software build+delivery"
//    ToolChain.  A "CICD manager" ToolOrchestration drives "software X|Y|Z
//    builds" ToolWorkflows that execute ToolTasks through that ToolChain.
//
//  @author  Kogi Team
//  @version 1.0.0
//  @license MIT
// =============================================================================

#![allow(dead_code)]

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =============================================================================
// §1 — TYPE ALIASES
// =============================================================================

pub type ToolId             = Uuid;
pub type ToolKitId          = Uuid;
pub type ToolChainId        = Uuid;
pub type ToolSetId          = Uuid;
pub type ToolBoxId          = Uuid;
pub type ToolProviderId     = Uuid;
pub type ToolIntegrationId  = Uuid;
pub type ToolAssemblyId     = Uuid;
pub type ToolOrchestrationId = Uuid;
pub type ToolWorkflowId     = Uuid;
pub type ToolTaskId         = Uuid;
pub type ToolVersionId      = Uuid;

/// Reference to a kogi platform element that a ToolBox or Integration attaches to.
pub type PlatformElementId  = Uuid;
/// Provider record ID string — mirrors provider.rs ProviderRecord.id
pub type ProviderRecordId   = String;

pub type ToolProperties     = HashMap<String, serde_json::Value>;
pub type ToolTags           = HashSet<String>;
pub type ToolLabels         = HashMap<String, String>;

// =============================================================================
// §2 — ERROR TYPES
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ToolError {
    NotFound(ToolId),
    ToolKitNotFound(ToolKitId),
    ToolChainNotFound(ToolChainId),
    ToolSetNotFound(ToolSetId),
    ToolBoxNotFound(ToolBoxId),
    IntegrationNotFound(ToolIntegrationId),
    OrchestrationNotFound(ToolOrchestrationId),
    WorkflowNotFound(ToolWorkflowId),
    TaskNotFound(ToolTaskId),
    ProviderNotFound(ProviderRecordId),
    PermissionDenied { user: String, action: String },
    InvalidOperation(String),
    AlreadyExists(String),
    CyclicDependency(ToolId, ToolId),
    VersionConflict { local: String, remote: String },
    InvalidState { current: String, attempted: String },
    ValidationError(String),
    IntegrationError(String),
    ExecutionError { task_id: ToolTaskId, message: String },
    WorkflowError { workflow_id: ToolWorkflowId, message: String },
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(id)                    => write!(f, "Tool not found: {id}"),
            Self::ToolKitNotFound(id)             => write!(f, "ToolKit not found: {id}"),
            Self::ToolChainNotFound(id)           => write!(f, "ToolChain not found: {id}"),
            Self::ToolSetNotFound(id)             => write!(f, "ToolSet not found: {id}"),
            Self::ToolBoxNotFound(id)             => write!(f, "ToolBox not found: {id}"),
            Self::IntegrationNotFound(id)         => write!(f, "Integration not found: {id}"),
            Self::OrchestrationNotFound(id)       => write!(f, "Orchestration not found: {id}"),
            Self::WorkflowNotFound(id)            => write!(f, "Workflow not found: {id}"),
            Self::TaskNotFound(id)                => write!(f, "Task not found: {id}"),
            Self::ProviderNotFound(id)            => write!(f, "Provider not found: {id}"),
            Self::PermissionDenied { user, action }
                => write!(f, "User '{user}' denied action '{action}'"),
            Self::InvalidOperation(msg)           => write!(f, "Invalid operation: {msg}"),
            Self::AlreadyExists(name)             => write!(f, "Already exists: {name}"),
            Self::CyclicDependency(a, b)          => write!(f, "Cyclic tool dependency: {a} ↔ {b}"),
            Self::VersionConflict { local, remote }
                => write!(f, "Version conflict: local={local} remote={remote}"),
            Self::InvalidState { current, attempted }
                => write!(f, "Cannot '{attempted}' from state '{current}'"),
            Self::ValidationError(msg)            => write!(f, "Validation error: {msg}"),
            Self::IntegrationError(msg)           => write!(f, "Integration error: {msg}"),
            Self::ExecutionError { task_id, message }
                => write!(f, "Execution error on task {task_id}: {message}"),
            Self::WorkflowError { workflow_id, message }
                => write!(f, "Workflow error on {workflow_id}: {message}"),
        }
    }
}

pub type ToolResult<T> = Result<T, ToolError>;

// =============================================================================
// §3 — ENUMS: CATEGORIES · TYPES · CLASSES · STATUSES
// =============================================================================

// ── Tool Categories ──────────────────────────────────────────────────────────

/// Broad domain classification of a tool.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolCategory {
    Development,
    DevOps,
    Design,
    Analytics,
    Communication,
    ProjectManagement,
    DataManagement,
    Security,
    Finance,
    Marketing,
    AIAssistant,
    Automation,
    Infrastructure,
    Monitoring,
    Documentation,
    Testing,
    Deployment,
    Collaboration,
    Storage,
    Integration,
    Custom(String),
}

/// Fine-grained operational type of a tool.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolType {
    /// Source control, code review
    CodeManagement,
    /// CI/CD runners, build servers
    ContinuousIntegration,
    /// Container orchestration, deployment
    ContinuousDeployment,
    /// Issue tracking, sprint boards
    ProjectTracker,
    /// Wireframing, UI design
    DesignCanvas,
    /// Metrics, dashboards
    Observability,
    /// Webhook/event automation
    EventTrigger,
    /// AI / LLM inference
    AIModel,
    /// Data pipeline / ETL
    DataPipeline,
    /// API gateway / connector
    ApiGateway,
    /// Notification / messaging
    Messenger,
    /// Auth / IAM
    AccessControl,
    /// File / asset management
    AssetManager,
    /// Scheduling / cron
    Scheduler,
    /// Testing / QA runner
    TestRunner,
    /// Documentation generator
    DocGenerator,
    /// Billing / payments
    PaymentProcessor,
    /// CRM / customer data
    CustomerRelations,
    /// Workflow/process builder
    WorkflowBuilder,
    Custom(String),
}

/// Structural class that determines how a tool participates in pipelines.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolClass {
    /// Source: only produces output, no pipeline input required
    Source,
    /// Sink: only consumes input, produces no pipeline output
    Sink,
    /// Transformer: transforms input into output
    Transformer,
    /// Connector: bridges two other tools
    Connector,
    /// Trigger: conditionally starts a workflow
    Trigger,
    /// Observer: monitors without transforming
    Observer,
    /// Controller: governs other tools in the chain
    Controller,
    Custom(String),
}

// ── Lifecycle statuses ───────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolStatus {
    Draft,
    Active,
    Inactive,
    Deprecated,
    Archived,
    UnderReview,
    Beta,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolState {
    Uninitialized,
    Configured,
    Connected,
    Running,
    Idle,
    Error,
    Disconnected,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolProviderKind {
    /// Tool is native to the Kogi platform
    Root,
    /// Tool comes from a third-party provider
    ThirdParty,
    /// Tool is provided by an affiliate
    Affiliate,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolIntegrationStatus {
    Pending,
    Active,
    Suspended,
    Revoked,
    Error,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolTaskStatus {
    Pending,
    Queued,
    Running,
    Succeeded,
    Failed,
    Skipped,
    Cancelled,
    TimedOut,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolWorkflowStatus {
    Draft,
    Active,
    Running,
    Paused,
    Completed,
    Failed,
    Archived,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolOrchestrationStatus {
    Draft,
    Active,
    Running,
    Paused,
    Completed,
    Archived,
    Custom(String),
}

/// Defines when / how a workflow or task is triggered.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ToolTriggerKind {
    Manual,
    Scheduled { cron: String },
    EventDriven { event_type: String, source_tool_id: Option<ToolId> },
    WebHook { url: String },
    ChainCompletion { chain_id: ToolChainId },
    WorkflowCompletion { workflow_id: ToolWorkflowId },
    Custom(String),
}

// =============================================================================
// §4 — VERSION CONTROL FOR TOOLS
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolVersionEntry {
    pub id: ToolVersionId,
    pub semver: String,
    pub author: String,
    pub message: String,
    pub breaking_change: bool,
    pub compatibility_notes: Vec<String>,
    pub published_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

impl ToolVersionEntry {
    pub fn new(semver: impl Into<String>, author: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            semver: semver.into(),
            author: author.into(),
            message: message.into(),
            breaking_change: false,
            compatibility_notes: Vec::new(),
            published_at: Utc::now(),
            tags: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolVersionHistory {
    pub current_version: String,
    pub entries: Vec<ToolVersionEntry>,
}

impl Default for ToolVersionHistory {
    fn default() -> Self {
        Self {
            current_version: "0.1.0".to_string(),
            entries: Vec::new(),
        }
    }
}

impl ToolVersionHistory {
    pub fn bump_patch(&mut self, author: &str, message: &str) -> String {
        let (major, minor, patch) = parse_semver(&self.current_version);
        let new_ver = format!("{major}.{minor}.{}", patch + 1);
        self.push_version(new_ver.clone(), author, message, false);
        new_ver
    }

    pub fn bump_minor(&mut self, author: &str, message: &str) -> String {
        let (major, minor, _) = parse_semver(&self.current_version);
        let new_ver = format!("{major}.{}.0", minor + 1);
        self.push_version(new_ver.clone(), author, message, false);
        new_ver
    }

    pub fn bump_major(&mut self, author: &str, message: &str) -> String {
        let (major, _, _) = parse_semver(&self.current_version);
        let new_ver = format!("{}.0.0", major + 1);
        self.push_version(new_ver.clone(), author, message, true);
        new_ver
    }

    fn push_version(&mut self, ver: String, author: &str, message: &str, breaking: bool) {
        self.current_version = ver.clone();
        self.entries.push(ToolVersionEntry {
            id: Uuid::new_v4(),
            semver: ver,
            author: author.to_string(),
            message: message.to_string(),
            breaking_change: breaking,
            compatibility_notes: Vec::new(),
            published_at: Utc::now(),
            tags: Vec::new(),
        });
    }
}

fn parse_semver(v: &str) -> (u64, u64, u64) {
    let parts: Vec<u64> = v.split('.').map(|s| s.parse().unwrap_or(0)).collect();
    let major = parts.first().copied().unwrap_or(0);
    let minor = parts.get(1).copied().unwrap_or(0);
    let patch = parts.get(2).copied().unwrap_or(0);
    (major, minor, patch)
}

// =============================================================================
// §5 — TOOL DATA · INFO · METADATA
// =============================================================================

/// Low-level technical data: configuration, parameters, I/O schema.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolData {
    /// Static configuration key-value pairs
    pub config: HashMap<String, serde_json::Value>,
    /// Typed parameter definitions (name → schema)
    pub parameters: HashMap<String, serde_json::Value>,
    /// JSON schema for tool input
    pub input_schema: Option<serde_json::Value>,
    /// JSON schema for tool output
    pub output_schema: Option<serde_json::Value>,
    /// Environment variables required
    pub env_vars: Vec<String>,
    /// Endpoint or connection string template
    pub endpoint_template: Option<String>,
    /// Auth method (oauth2, api_key, basic, none)
    pub auth_method: Option<String>,
    /// Rate limits (calls per minute, etc.)
    pub rate_limits: HashMap<String, u64>,
    /// Arbitrary binary or text payloads by key
    pub payloads: HashMap<String, String>,
    /// Internal feature flags
    pub feature_flags: HashMap<String, bool>,
}

/// High-level human-readable information about a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub display_name: String,
    pub short_description: String,
    pub long_description: String,
    pub documentation_url: Option<String>,
    pub homepage_url: Option<String>,
    pub icon_url: Option<String>,
    pub screenshots: Vec<String>,
    pub use_cases: Vec<String>,
    pub capabilities: Vec<String>,
    pub limitations: Vec<String>,
    pub tutorials: Vec<String>,
    pub changelog_url: Option<String>,
    pub support_url: Option<String>,
    pub license: Option<String>,
}

impl Default for ToolInfo {
    fn default() -> Self {
        Self {
            display_name: String::new(),
            short_description: String::new(),
            long_description: String::new(),
            documentation_url: None,
            homepage_url: None,
            icon_url: None,
            screenshots: Vec::new(),
            use_cases: Vec::new(),
            capabilities: Vec::new(),
            limitations: Vec::new(),
            tutorials: Vec::new(),
            changelog_url: None,
            support_url: None,
            license: None,
        }
    }
}

/// Machine/meta information: unique identity, taxonomy, version, provider ref.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    pub id: ToolId,
    /// Human-readable slug / token (e.g. "github-cmt", "figma-design")
    pub token: String,
    /// Canonical system name (snake_case)
    pub system_name: String,
    /// Alternative display names / aliases
    pub aliases: Vec<String>,
    pub category: ToolCategory,
    pub tool_type: ToolType,
    pub tool_class: ToolClass,
    /// User-defined tags for search and filtering
    pub tags: ToolTags,
    /// Key-value labels (e.g. "env: production", "team: devops")
    pub labels: ToolLabels,
    /// Reference to provider.rs ProviderRecord.id; None for root tools
    pub provider_record_id: Option<ProviderRecordId>,
    pub provider_kind: ToolProviderKind,
    pub version_history: ToolVersionHistory,
    /// Indicates whether this tool is a prepackaged template
    pub is_template: bool,
    /// Indicates whether this tool is available on the Kogi marketplace
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
}

impl ToolMetadata {
    pub fn new(
        system_name: impl Into<String>,
        token: impl Into<String>,
        category: ToolCategory,
        tool_type: ToolType,
        tool_class: ToolClass,
        created_by: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            token: token.into(),
            system_name: system_name.into(),
            aliases: Vec::new(),
            category,
            tool_type,
            tool_class,
            tags: HashSet::new(),
            labels: HashMap::new(),
            provider_record_id: None,
            provider_kind: ToolProviderKind::Root,
            version_history: ToolVersionHistory::default(),
            is_template: false,
            is_public: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: created_by.into(),
        }
    }

    pub fn current_version(&self) -> &str {
        &self.version_history.current_version
    }

    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

// =============================================================================
// §6 — TOOL ASSEMBLY  (blueprint: brief + description + parts list + design)
// =============================================================================

/// Complete blueprint/design of a tool — describes how a tool is built/assembled.
/// Contains all three layers: ToolData (low-level), ToolInfo (high-level),
/// ToolMetadata (meta), plus a brief and a structural parts/component list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolAssembly {
    pub id: ToolAssemblyId,
    /// One-line summary of what this tool does
    pub brief: String,
    /// Detailed prose description of the tool's design and purpose
    pub description: String,
    /// Structural components / dependency list (e.g. libraries, APIs used)
    pub components: Vec<AssemblyComponent>,
    /// The low-level data layer
    pub data: ToolData,
    /// The high-level info layer
    pub info: ToolInfo,
    /// The metadata / identity layer
    pub metadata: ToolMetadata,
    pub assembled_at: DateTime<Utc>,
    pub assembled_by: String,
}

/// An individual piece/part that makes up a tool's assembly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssemblyComponent {
    pub id: Uuid,
    pub name: String,
    pub component_type: AssemblyComponentType,
    pub version: Option<String>,
    pub description: String,
    pub required: bool,
    pub config: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssemblyComponentType {
    /// An external API or SDK
    Api,
    /// A library or package dependency
    Library,
    /// Another Kogi tool (sub-tool)
    SubTool,
    /// A webhook endpoint
    WebHook,
    /// A data schema
    Schema,
    /// A configuration file
    ConfigFile,
    /// An authentication provider
    AuthProvider,
    Custom(String),
}

impl ToolAssembly {
    pub fn new(
        brief: impl Into<String>,
        description: impl Into<String>,
        metadata: ToolMetadata,
        assembled_by: impl Into<String>,
    ) -> Self {
        let info = ToolInfo {
            display_name: metadata.system_name.clone(),
            short_description: brief.to_string(),
            ..Default::default()
        };
        Self {
            id: Uuid::new_v4(),
            brief: brief.into(),
            description: description.into(),
            components: Vec::new(),
            data: ToolData::default(),
            info,
            metadata,
            assembled_at: Utc::now(),
            assembled_by: assembled_by.into(),
        }
    }

    pub fn tool_id(&self) -> ToolId { self.metadata.id }

    pub fn add_component(&mut self, component: AssemblyComponent) {
        self.components.push(component);
        self.metadata.touch();
    }
}

// =============================================================================
// §7 — TOOL PROVIDER
// =============================================================================

/// Information about who/what provides a tool — either native (Root) or 3rd-party.
/// Links back to provider.rs ProviderRecord via `provider_record_id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolProvider {
    pub id: ToolProviderId,
    pub name: String,
    pub kind: ToolProviderKind,
    /// Foreign key to provider.rs ProviderRecord.id (None = root/kogi-native)
    pub provider_record_id: Option<ProviderRecordId>,
    pub homepage_url: Option<String>,
    pub docs_url: Option<String>,
    pub api_base_url: Option<String>,
    pub supported_auth_methods: Vec<String>,
    pub capabilities: Vec<String>,
    pub status: ToolStatus,
    pub tags: ToolTags,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ToolProvider {
    /// Create the root Kogi platform provider.
    pub fn root() -> Self {
        Self {
            id: Uuid::nil(),
            name: "Kogi Platform".to_string(),
            kind: ToolProviderKind::Root,
            provider_record_id: None,
            homepage_url: Some("https://kogi.io".to_string()),
            docs_url: Some("https://docs.kogi.io".to_string()),
            api_base_url: Some("https://api.kogi.io".to_string()),
            supported_auth_methods: vec!["oauth2".to_string(), "api_key".to_string()],
            capabilities: vec!["native".to_string()],
            status: ToolStatus::Active,
            tags: HashSet::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn new(
        name: impl Into<String>,
        kind: ToolProviderKind,
        provider_record_id: Option<ProviderRecordId>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            kind,
            provider_record_id,
            homepage_url: None,
            docs_url: None,
            api_base_url: None,
            supported_auth_methods: Vec::new(),
            capabilities: Vec::new(),
            status: ToolStatus::Active,
            tags: HashSet::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

// =============================================================================
// §8 — TOOL  (core entity)
// =============================================================================

/// A Tool is the core entity — a utility used to help a user accomplish
/// an outcome/objective/goal/desire. Every tool has a full assembly (blueprint),
/// a provider, lifecycle status/state, and optional platform connections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// The complete blueprint/assembly of this tool
    pub assembly: ToolAssembly,
    /// Provider that supplies / hosts this tool
    pub provider_id: ToolProviderId,
    pub status: ToolStatus,
    pub state: ToolState,
    /// IDs of integrations this tool participates in
    pub integration_ids: Vec<ToolIntegrationId>,
    /// IDs of toolkits this tool belongs to
    pub toolkit_memberships: Vec<ToolKitId>,
    /// IDs of toolchains this tool is a step in
    pub toolchain_memberships: Vec<ToolChainId>,
    pub notes: String,
}

impl Tool {
    pub fn new(assembly: ToolAssembly, provider_id: ToolProviderId) -> Self {
        Self {
            assembly,
            provider_id,
            status: ToolStatus::Draft,
            state: ToolState::Uninitialized,
            integration_ids: Vec::new(),
            toolkit_memberships: Vec::new(),
            toolchain_memberships: Vec::new(),
            notes: String::new(),
        }
    }

    pub fn id(&self) -> ToolId { self.assembly.tool_id() }
    pub fn name(&self) -> &str { &self.assembly.metadata.system_name }
    pub fn display_name(&self) -> &str { &self.assembly.info.display_name }
    pub fn version(&self) -> &str { self.assembly.metadata.current_version() }
    pub fn token(&self) -> &str { &self.assembly.metadata.token }
    pub fn category(&self) -> &ToolCategory { &self.assembly.metadata.category }
    pub fn tool_type(&self) -> &ToolType { &self.assembly.metadata.tool_type }
    pub fn tool_class(&self) -> &ToolClass { &self.assembly.metadata.tool_class }

    pub fn activate(&mut self) -> ToolResult<()> {
        if self.status == ToolStatus::Archived {
            return Err(ToolError::InvalidState {
                current: "Archived".to_string(),
                attempted: "activate".to_string(),
            });
        }
        self.status = ToolStatus::Active;
        self.state  = ToolState::Configured;
        self.assembly.metadata.touch();
        Ok(())
    }

    pub fn connect(&mut self) -> ToolResult<()> {
        if self.status != ToolStatus::Active {
            return Err(ToolError::InvalidState {
                current: format!("{:?}", self.status),
                attempted: "connect".to_string(),
            });
        }
        self.state = ToolState::Connected;
        Ok(())
    }

    pub fn deprecate(&mut self) {
        self.status = ToolStatus::Deprecated;
        self.state  = ToolState::Disconnected;
        self.assembly.metadata.touch();
    }

    pub fn archive(&mut self) {
        self.status = ToolStatus::Archived;
        self.state  = ToolState::Disconnected;
        self.assembly.metadata.touch();
    }
}

// =============================================================================
// §9 — TOOL INTEGRATION
// =============================================================================

/// Defines which kogi platform elements a tool+provider is integrated into.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntegrationTarget {
    Portfolio(PlatformElementId),
    PortfolioComponent(PlatformElementId),
    PortfolioItem(PlatformElementId),
    PortfolioContainer(PlatformElementId),
    Timeline(PlatformElementId),
    Schedule(PlatformElementId),
    AIAssistant(PlatformElementId),
    ToolBox(ToolBoxId),
    Global,
    Custom { target_type: String, target_id: PlatformElementId },
}

/// A ToolIntegration binds a Tool + ToolProvider to the Kogi platform,
/// making the tool available within specific platform elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolIntegration {
    pub id: ToolIntegrationId,
    pub tool_id: ToolId,
    pub provider_id: ToolProviderId,
    pub name: String,
    pub description: String,
    pub status: ToolIntegrationStatus,
    pub targets: Vec<IntegrationTarget>,
    /// OAuth / API key / credential references
    pub credential_refs: Vec<String>,
    /// Runtime configuration overrides for this integration
    pub config_overrides: HashMap<String, serde_json::Value>,
    /// Scopes / permissions granted
    pub scopes: Vec<String>,
    pub webhook_url: Option<String>,
    pub callback_url: Option<String>,
    pub tags: ToolTags,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
    pub last_synced_at: Option<DateTime<Utc>>,
}

impl ToolIntegration {
    pub fn new(
        tool_id: ToolId,
        provider_id: ToolProviderId,
        name: impl Into<String>,
        created_by: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            tool_id,
            provider_id,
            name: name.into(),
            description: String::new(),
            status: ToolIntegrationStatus::Pending,
            targets: Vec::new(),
            credential_refs: Vec::new(),
            config_overrides: HashMap::new(),
            scopes: Vec::new(),
            webhook_url: None,
            callback_url: None,
            tags: HashSet::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: created_by.into(),
            last_synced_at: None,
        }
    }

    pub fn add_target(&mut self, target: IntegrationTarget) {
        if !self.targets.contains(&target) {
            self.targets.push(target);
            self.updated_at = Utc::now();
        }
    }

    pub fn activate(&mut self) {
        self.status = ToolIntegrationStatus::Active;
        self.updated_at = Utc::now();
    }

    pub fn suspend(&mut self) {
        self.status = ToolIntegrationStatus::Suspended;
        self.updated_at = Utc::now();
    }

    pub fn revoke(&mut self) {
        self.status = ToolIntegrationStatus::Revoked;
        self.credential_refs.clear();
        self.updated_at = Utc::now();
    }
}

// =============================================================================
// §10 — TOOLKIT  (unordered collection / grouping of tools)
// =============================================================================

/// A ToolKit is an unordered collection/grouping of tools assembled to
/// accomplish some task or reach some outcome. Can be a prepackaged template
/// or a custom user-defined assembly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolKit {
    pub id: ToolKitId,
    pub name: String,
    pub description: String,
    pub purpose: String,
    pub tool_ids: HashSet<ToolId>,
    pub tags: ToolTags,
    pub labels: ToolLabels,
    pub is_template: bool,
    pub toolset_id: Option<ToolSetId>,
    pub status: ToolStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
    pub properties: ToolProperties,
}

impl ToolKit {
    pub fn new(name: impl Into<String>, purpose: impl Into<String>, created_by: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: String::new(),
            purpose: purpose.into(),
            tool_ids: HashSet::new(),
            tags: HashSet::new(),
            labels: HashMap::new(),
            is_template: false,
            toolset_id: None,
            status: ToolStatus::Draft,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: created_by.into(),
            properties: HashMap::new(),
        }
    }

    pub fn add_tool(&mut self, tool_id: ToolId) {
        self.tool_ids.insert(tool_id);
        self.updated_at = Utc::now();
    }

    pub fn remove_tool(&mut self, tool_id: &ToolId) {
        self.tool_ids.remove(tool_id);
        self.updated_at = Utc::now();
    }

    pub fn contains(&self, tool_id: &ToolId) -> bool {
        self.tool_ids.contains(tool_id)
    }

    pub fn len(&self) -> usize { self.tool_ids.len() }
    pub fn is_empty(&self) -> bool { self.tool_ids.is_empty() }

    pub fn activate(&mut self) {
        self.status = ToolStatus::Active;
        self.updated_at = Utc::now();
    }
}

// =============================================================================
// §11 — TOOLCHAIN  (ordered pipeline / sequence of connected tools)
// =============================================================================

/// A single step in a ToolChain pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChainStep {
    pub id: Uuid,
    pub step_index: u32,
    pub name: String,
    pub description: String,
    pub tool_id: ToolId,
    /// IDs of steps that must complete before this one runs
    pub depends_on: Vec<Uuid>,
    /// How output from the previous step maps to this step's input
    pub input_mapping: HashMap<String, String>,
    /// How this step's output maps to the next step's input
    pub output_mapping: HashMap<String, String>,
    /// Condition expression for conditional execution
    pub condition: Option<String>,
    /// Timeout for this step in seconds
    pub timeout_seconds: Option<u64>,
    /// Whether to continue chain on this step's failure
    pub continue_on_failure: bool,
    pub config_overrides: HashMap<String, serde_json::Value>,
}

impl ToolChainStep {
    pub fn new(step_index: u32, name: impl Into<String>, tool_id: ToolId) -> Self {
        Self {
            id: Uuid::new_v4(),
            step_index,
            name: name.into(),
            description: String::new(),
            tool_id,
            depends_on: Vec::new(),
            input_mapping: HashMap::new(),
            output_mapping: HashMap::new(),
            condition: None,
            timeout_seconds: None,
            continue_on_failure: false,
            config_overrides: HashMap::new(),
        }
    }
}

/// A ToolChain is an ordered, pipeline sequence/set of connected tools
/// used to accomplish some task and reach some outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChain {
    pub id: ToolChainId,
    pub name: String,
    pub description: String,
    pub purpose: String,
    /// Ordered steps — index 0 is the first step
    pub steps: Vec<ToolChainStep>,
    pub tags: ToolTags,
    pub labels: ToolLabels,
    pub is_template: bool,
    pub toolset_id: Option<ToolSetId>,
    pub status: ToolStatus,
    /// Global timeout for the entire chain in seconds
    pub timeout_seconds: Option<u64>,
    /// Whether steps can run in parallel where there are no dependencies
    pub allow_parallel: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
    pub properties: ToolProperties,
}

impl ToolChain {
    pub fn new(name: impl Into<String>, purpose: impl Into<String>, created_by: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: String::new(),
            purpose: purpose.into(),
            steps: Vec::new(),
            tags: HashSet::new(),
            labels: HashMap::new(),
            is_template: false,
            toolset_id: None,
            status: ToolStatus::Draft,
            timeout_seconds: None,
            allow_parallel: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: created_by.into(),
            properties: HashMap::new(),
        }
    }

    /// Append a step; step_index is auto-assigned to current length.
    pub fn add_step(&mut self, mut step: ToolChainStep) -> ToolResult<()> {
        step.step_index = self.steps.len() as u32;
        self.steps.push(step);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Insert a step at a specific index, re-numbering subsequent steps.
    pub fn insert_step(&mut self, index: usize, mut step: ToolChainStep) -> ToolResult<()> {
        if index > self.steps.len() {
            return Err(ToolError::InvalidOperation(
                format!("Insert index {index} out of bounds (chain has {} steps)", self.steps.len())
            ));
        }
        step.step_index = index as u32;
        self.steps.insert(index, step);
        for (i, s) in self.steps.iter_mut().enumerate() {
            s.step_index = i as u32;
        }
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn remove_step(&mut self, step_id: &Uuid) -> ToolResult<()> {
        let before = self.steps.len();
        self.steps.retain(|s| &s.id != step_id);
        if self.steps.len() == before {
            return Err(ToolError::InvalidOperation(format!("Step {step_id} not found in chain")));
        }
        for (i, s) in self.steps.iter_mut().enumerate() {
            s.step_index = i as u32;
        }
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn tool_ids(&self) -> Vec<ToolId> {
        self.steps.iter().map(|s| s.tool_id).collect()
    }

    pub fn activate(&mut self) {
        self.status = ToolStatus::Active;
        self.updated_at = Utc::now();
    }

    pub fn len(&self) -> usize { self.steps.len() }
    pub fn is_empty(&self) -> bool { self.steps.is_empty() }
}

// =============================================================================
// §12 — TOOLSET  (general, complete template set of toolkits + toolchains + tools)
// =============================================================================

/// A ToolSet is a general, complete template set/group of associated ToolKits,
/// ToolChains, and ungrouped tools that are all related/connected with one
/// another. ToolSets live in a ToolBox and represent templated tool
/// orchestrations/workflows/task automations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSet {
    pub id: ToolSetId,
    pub name: String,
    pub description: String,
    /// Broad purpose / domain of this toolset (e.g. "DevOps", "Marketing Automation")
    pub domain: String,
    /// IDs of ToolKits belonging to this ToolSet
    pub toolkit_ids: Vec<ToolKitId>,
    /// IDs of ToolChains belonging to this ToolSet
    pub toolchain_ids: Vec<ToolChainId>,
    /// Ungrouped tool IDs directly in this ToolSet (not in any Kit or Chain)
    pub ungrouped_tool_ids: HashSet<ToolId>,
    pub tags: ToolTags,
    pub labels: ToolLabels,
    pub is_template: bool,
    pub toolbox_id: Option<ToolBoxId>,
    pub status: ToolStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
    pub properties: ToolProperties,
}

impl ToolSet {
    pub fn new(name: impl Into<String>, domain: impl Into<String>, created_by: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: String::new(),
            domain: domain.into(),
            toolkit_ids: Vec::new(),
            toolchain_ids: Vec::new(),
            ungrouped_tool_ids: HashSet::new(),
            tags: HashSet::new(),
            labels: HashMap::new(),
            is_template: false,
            toolbox_id: None,
            status: ToolStatus::Draft,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: created_by.into(),
            properties: HashMap::new(),
        }
    }

    pub fn add_toolkit(&mut self, kit_id: ToolKitId) {
        if !self.toolkit_ids.contains(&kit_id) {
            self.toolkit_ids.push(kit_id);
            self.updated_at = Utc::now();
        }
    }

    pub fn add_toolchain(&mut self, chain_id: ToolChainId) {
        if !self.toolchain_ids.contains(&chain_id) {
            self.toolchain_ids.push(chain_id);
            self.updated_at = Utc::now();
        }
    }

    pub fn add_ungrouped_tool(&mut self, tool_id: ToolId) {
        self.ungrouped_tool_ids.insert(tool_id);
        self.updated_at = Utc::now();
    }

    pub fn remove_toolkit(&mut self, kit_id: &ToolKitId) {
        self.toolkit_ids.retain(|id| id != kit_id);
        self.updated_at = Utc::now();
    }

    pub fn remove_toolchain(&mut self, chain_id: &ToolChainId) {
        self.toolchain_ids.retain(|id| id != chain_id);
        self.updated_at = Utc::now();
    }

    pub fn activate(&mut self) {
        self.status = ToolStatus::Active;
        self.updated_at = Utc::now();
    }
}

// =============================================================================
// §13 — TOOLBOX  (core container where tools exist/live)
// =============================================================================

/// A ToolBox is the core space where tools exist and live.
/// It contains prepackaged or custom ToolSets, which are composed of
/// ToolKits, ToolChains, and ungrouped tools.
/// ToolBoxes can be attached to portfolio elements (portfolios, items,
/// containers, components).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolBox {
    pub id: ToolBoxId,
    pub name: String,
    pub description: String,
    /// IDs of ToolSets inside this ToolBox
    pub toolset_ids: Vec<ToolSetId>,
    /// Loose tool IDs not organized into any ToolSet
    pub loose_tool_ids: HashSet<ToolId>,
    /// Integration IDs available within this ToolBox
    pub integration_ids: Vec<ToolIntegrationId>,
    /// Platform elements this ToolBox is attached to
    pub attached_to: Vec<ToolBoxAttachment>,
    pub tags: ToolTags,
    pub labels: ToolLabels,
    pub status: ToolStatus,
    pub is_template: bool,
    pub owner: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub properties: ToolProperties,
}

/// Describes a platform element that a ToolBox is attached to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolBoxAttachment {
    pub element_type: ToolBoxElementType,
    pub element_id: PlatformElementId,
    pub attached_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolBoxElementType {
    Portfolio,
    PortfolioComponent,
    PortfolioItem,
    PortfolioContainer,
    Custom(String),
}

impl ToolBox {
    pub fn new(name: impl Into<String>, owner: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: String::new(),
            toolset_ids: Vec::new(),
            loose_tool_ids: HashSet::new(),
            integration_ids: Vec::new(),
            attached_to: Vec::new(),
            tags: HashSet::new(),
            labels: HashMap::new(),
            status: ToolStatus::Active,
            is_template: false,
            owner: owner.into(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            properties: HashMap::new(),
        }
    }

    pub fn add_toolset(&mut self, set_id: ToolSetId) {
        if !self.toolset_ids.contains(&set_id) {
            self.toolset_ids.push(set_id);
            self.updated_at = Utc::now();
        }
    }

    pub fn remove_toolset(&mut self, set_id: &ToolSetId) {
        self.toolset_ids.retain(|id| id != set_id);
        self.updated_at = Utc::now();
    }

    pub fn add_loose_tool(&mut self, tool_id: ToolId) {
        self.loose_tool_ids.insert(tool_id);
        self.updated_at = Utc::now();
    }

    pub fn add_integration(&mut self, integration_id: ToolIntegrationId) {
        if !self.integration_ids.contains(&integration_id) {
            self.integration_ids.push(integration_id);
            self.updated_at = Utc::now();
        }
    }

    pub fn attach_to_element(&mut self, element_type: ToolBoxElementType, element_id: PlatformElementId) {
        let attachment = ToolBoxAttachment {
            element_type,
            element_id,
            attached_at: Utc::now(),
        };
        if !self.attached_to.iter().any(|a| a.element_id == element_id) {
            self.attached_to.push(attachment);
            self.updated_at = Utc::now();
        }
    }

    pub fn detach_from_element(&mut self, element_id: &PlatformElementId) {
        self.attached_to.retain(|a| &a.element_id != element_id);
        self.updated_at = Utc::now();
    }
}

// =============================================================================
// §14 — TOOL TASK  (atomic executable unit in a workflow)
// =============================================================================

/// A ToolTask is an atomic, executable unit within a ToolWorkflow.
/// It references a specific ToolChain step or a direct tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolTask {
    pub id: ToolTaskId,
    pub name: String,
    pub description: String,
    pub workflow_id: ToolWorkflowId,
    /// The tool this task uses directly
    pub tool_id: ToolId,
    /// Optional toolchain this task is part of
    pub toolchain_id: Option<ToolChainId>,
    /// Optional specific step within the toolchain
    pub toolchain_step_id: Option<Uuid>,
    pub task_index: u32,
    pub status: ToolTaskStatus,
    pub trigger: Option<ToolTriggerKind>,
    pub input: HashMap<String, serde_json::Value>,
    pub output: HashMap<String, serde_json::Value>,
    pub depends_on: Vec<ToolTaskId>,
    pub timeout_seconds: Option<u64>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub continue_on_failure: bool,
    pub error_message: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl ToolTask {
    pub fn new(
        name: impl Into<String>,
        workflow_id: ToolWorkflowId,
        tool_id: ToolId,
        task_index: u32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: String::new(),
            workflow_id,
            tool_id,
            toolchain_id: None,
            toolchain_step_id: None,
            task_index,
            status: ToolTaskStatus::Pending,
            trigger: None,
            input: HashMap::new(),
            output: HashMap::new(),
            depends_on: Vec::new(),
            timeout_seconds: None,
            retry_count: 0,
            max_retries: 3,
            continue_on_failure: false,
            error_message: None,
            started_at: None,
            completed_at: None,
            created_at: Utc::now(),
        }
    }

    pub fn start(&mut self) {
        self.status = ToolTaskStatus::Running;
        self.started_at = Some(Utc::now());
    }

    pub fn succeed(&mut self, output: HashMap<String, serde_json::Value>) {
        self.status = ToolTaskStatus::Succeeded;
        self.output = output;
        self.completed_at = Some(Utc::now());
    }

    pub fn fail(&mut self, message: impl Into<String>) {
        if self.retry_count < self.max_retries {
            self.retry_count += 1;
            self.status = ToolTaskStatus::Pending;
        } else {
            self.status = ToolTaskStatus::Failed;
            self.error_message = Some(message.into());
            self.completed_at = Some(Utc::now());
        }
    }

    pub fn skip(&mut self) {
        self.status = ToolTaskStatus::Skipped;
        self.completed_at = Some(Utc::now());
    }

    pub fn cancel(&mut self) {
        self.status = ToolTaskStatus::Cancelled;
        self.completed_at = Some(Utc::now());
    }

    pub fn duration_ms(&self) -> Option<i64> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some((end - start).num_milliseconds()),
            _ => None,
        }
    }
}

// =============================================================================
// §15 — TOOL WORKFLOW  (series of automated, executable, connected tasks)
// =============================================================================

/// A ToolWorkflow contains a series of automated executable connected/linked/
/// sequenced ToolTasks.  Workflows can use ToolChains, ToolKits, or reference
/// tools directly to accomplish a defined goal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolWorkflow {
    pub id: ToolWorkflowId,
    pub name: String,
    pub description: String,
    pub goal: String,
    pub orchestration_id: ToolOrchestrationId,
    pub tasks: Vec<ToolTask>,
    /// ToolChains used in this workflow
    pub toolchain_ids: Vec<ToolChainId>,
    /// ToolKits used in this workflow
    pub toolkit_ids: Vec<ToolKitId>,
    pub status: ToolWorkflowStatus,
    pub trigger: ToolTriggerKind,
    pub tags: ToolTags,
    pub context: HashMap<String, serde_json::Value>,
    pub allow_parallel_tasks: bool,
    pub timeout_seconds: Option<u64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_run_at: Option<DateTime<Utc>>,
    pub run_count: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub created_by: String,
}

impl ToolWorkflow {
    pub fn new(
        name: impl Into<String>,
        goal: impl Into<String>,
        orchestration_id: ToolOrchestrationId,
        created_by: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: String::new(),
            goal: goal.into(),
            orchestration_id,
            tasks: Vec::new(),
            toolchain_ids: Vec::new(),
            toolkit_ids: Vec::new(),
            status: ToolWorkflowStatus::Draft,
            trigger: ToolTriggerKind::Manual,
            tags: HashSet::new(),
            context: HashMap::new(),
            allow_parallel_tasks: false,
            timeout_seconds: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_run_at: None,
            run_count: 0,
            success_count: 0,
            failure_count: 0,
            created_by: created_by.into(),
        }
    }

    pub fn add_task(&mut self, mut task: ToolTask) -> ToolTaskId {
        task.task_index = self.tasks.len() as u32;
        let id = task.id;
        self.tasks.push(task);
        self.updated_at = Utc::now();
        id
    }

    pub fn use_toolchain(&mut self, chain_id: ToolChainId) {
        if !self.toolchain_ids.contains(&chain_id) {
            self.toolchain_ids.push(chain_id);
            self.updated_at = Utc::now();
        }
    }

    pub fn use_toolkit(&mut self, kit_id: ToolKitId) {
        if !self.toolkit_ids.contains(&kit_id) {
            self.toolkit_ids.push(kit_id);
            self.updated_at = Utc::now();
        }
    }

    pub fn activate(&mut self) {
        self.status = ToolWorkflowStatus::Active;
        self.updated_at = Utc::now();
    }

    pub fn record_run(&mut self, succeeded: bool) {
        self.run_count += 1;
        self.last_run_at = Some(Utc::now());
        if succeeded { self.success_count += 1; } else { self.failure_count += 1; }
    }

    pub fn success_rate(&self) -> f64 {
        if self.run_count == 0 { return 0.0; }
        self.success_count as f64 / self.run_count as f64 * 100.0
    }

    pub fn pending_tasks(&self) -> Vec<&ToolTask> {
        self.tasks.iter().filter(|t| t.status == ToolTaskStatus::Pending).collect()
    }

    pub fn all_tasks_complete(&self) -> bool {
        self.tasks.iter().all(|t| matches!(
            t.status,
            ToolTaskStatus::Succeeded | ToolTaskStatus::Skipped | ToolTaskStatus::Cancelled
        ))
    }
}

// =============================================================================
// §16 — TOOL ORCHESTRATION  (collection of tool workflows)
// =============================================================================

/// A ToolOrchestration is a high-level automation composed of ToolWorkflows.
/// It represents a full automation plan: when and how tools interact/connect
/// with one another to accomplish a complex goal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOrchestration {
    pub id: ToolOrchestrationId,
    pub name: String,
    pub description: String,
    pub goal: String,
    pub workflow_ids: Vec<ToolWorkflowId>,
    pub status: ToolOrchestrationStatus,
    pub trigger: ToolTriggerKind,
    /// ToolSets providing context for this orchestration
    pub toolset_ids: Vec<ToolSetId>,
    /// ToolBoxes this orchestration operates within
    pub toolbox_ids: Vec<ToolBoxId>,
    pub tags: ToolTags,
    pub labels: ToolLabels,
    pub schedule: Option<OrchestrationSchedule>,
    pub run_history: Vec<OrchestrationRun>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
    pub properties: ToolProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationSchedule {
    pub cron_expression: String,
    pub timezone: String,
    pub enabled: bool,
    pub next_run_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationRun {
    pub id: Uuid,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: ToolTaskStatus,
    pub workflows_run: Vec<ToolWorkflowId>,
    pub error: Option<String>,
}

impl ToolOrchestration {
    pub fn new(
        name: impl Into<String>,
        goal: impl Into<String>,
        created_by: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: String::new(),
            goal: goal.into(),
            workflow_ids: Vec::new(),
            status: ToolOrchestrationStatus::Draft,
            trigger: ToolTriggerKind::Manual,
            toolset_ids: Vec::new(),
            toolbox_ids: Vec::new(),
            tags: HashSet::new(),
            labels: HashMap::new(),
            schedule: None,
            run_history: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: created_by.into(),
            properties: HashMap::new(),
        }
    }

    pub fn add_workflow(&mut self, workflow_id: ToolWorkflowId) {
        if !self.workflow_ids.contains(&workflow_id) {
            self.workflow_ids.push(workflow_id);
            self.updated_at = Utc::now();
        }
    }

    pub fn activate(&mut self) {
        self.status = ToolOrchestrationStatus::Active;
        self.updated_at = Utc::now();
    }

    pub fn record_run(&mut self, run: OrchestrationRun) {
        self.run_history.push(run);
        self.updated_at = Utc::now();
    }
}

// =============================================================================
// §17 — TOOL AUTOMATION SYSTEM  (orchestrations + workflow engine)
// =============================================================================

/// The ToolAutomationSystem manages all orchestrations and drives
/// workflow/task execution. It is the runtime engine of the TMS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolAutomationSystem {
    pub orchestrations: HashMap<ToolOrchestrationId, ToolOrchestration>,
    pub workflows: HashMap<ToolWorkflowId, ToolWorkflow>,
    /// Queue of workflow IDs pending execution
    pub execution_queue: VecDeque<ToolWorkflowId>,
    pub created_at: DateTime<Utc>,
}

impl Default for ToolAutomationSystem {
    fn default() -> Self { Self::new() }
}

impl ToolAutomationSystem {
    pub fn new() -> Self {
        Self {
            orchestrations: HashMap::new(),
            workflows: HashMap::new(),
            execution_queue: VecDeque::new(),
            created_at: Utc::now(),
        }
    }

    // ── Orchestrations ───────────────────────────────────────────────────────

    pub fn create_orchestration(&mut self, orch: ToolOrchestration) -> ToolOrchestrationId {
        let id = orch.id;
        self.orchestrations.insert(id, orch);
        id
    }

    pub fn get_orchestration(&self, id: &ToolOrchestrationId) -> ToolResult<&ToolOrchestration> {
        self.orchestrations.get(id).ok_or(ToolError::OrchestrationNotFound(*id))
    }

    pub fn get_orchestration_mut(&mut self, id: &ToolOrchestrationId) -> ToolResult<&mut ToolOrchestration> {
        self.orchestrations.get_mut(id).ok_or(ToolError::OrchestrationNotFound(*id))
    }

    // ── Workflows ────────────────────────────────────────────────────────────

    pub fn create_workflow(&mut self, wf: ToolWorkflow) -> ToolWorkflowId {
        let id = wf.id;
        self.workflows.insert(id, wf);
        id
    }

    pub fn get_workflow(&self, id: &ToolWorkflowId) -> ToolResult<&ToolWorkflow> {
        self.workflows.get(id).ok_or(ToolError::WorkflowNotFound(*id))
    }

    pub fn get_workflow_mut(&mut self, id: &ToolWorkflowId) -> ToolResult<&mut ToolWorkflow> {
        self.workflows.get_mut(id).ok_or(ToolError::WorkflowNotFound(*id))
    }

    /// Attach a workflow to an orchestration and register it.
    pub fn attach_workflow_to_orchestration(
        &mut self,
        orch_id: &ToolOrchestrationId,
        workflow: ToolWorkflow,
    ) -> ToolResult<ToolWorkflowId> {
        let wf_id = workflow.id;
        {
            let orch = self.get_orchestration_mut(orch_id)?;
            orch.add_workflow(wf_id);
        }
        self.workflows.insert(wf_id, workflow);
        Ok(wf_id)
    }

    /// Enqueue a workflow for execution.
    pub fn enqueue(&mut self, workflow_id: ToolWorkflowId) -> ToolResult<()> {
        self.get_workflow(&workflow_id)?;
        self.execution_queue.push_back(workflow_id);
        Ok(())
    }

    /// Simulate execution of the next workflow in the queue.
    /// In production, this would dispatch to an actual executor.
    pub fn execute_next(&mut self) -> ToolResult<Option<ToolWorkflowId>> {
        let Some(wf_id) = self.execution_queue.pop_front() else {
            return Ok(None);
        };
        let wf = self.get_workflow_mut(&wf_id)?;
        wf.status = ToolWorkflowStatus::Running;
        wf.last_run_at = Some(Utc::now());
        // Simulate: mark all pending tasks as succeeded
        for task in wf.tasks.iter_mut() {
            if task.status == ToolTaskStatus::Pending {
                task.start();
                task.succeed(HashMap::new());
            }
        }
        let all_ok = wf.all_tasks_complete();
        wf.record_run(all_ok);
        wf.status = if all_ok {
            ToolWorkflowStatus::Completed
        } else {
            ToolWorkflowStatus::Failed
        };
        Ok(Some(wf_id))
    }

    /// Workflows in active orchestrations with pending tasks.
    pub fn runnable_workflows(&self) -> Vec<&ToolWorkflow> {
        self.workflows
            .values()
            .filter(|wf| wf.status == ToolWorkflowStatus::Active && !wf.pending_tasks().is_empty())
            .collect()
    }
}

// =============================================================================
// §18 — TOOL MANAGEMENT SYSTEM  (top-level orchestrator)
// =============================================================================

/// ToolManagementSystem is the root system that manages all tool-related
/// entities: tools, providers, integrations, toolkits, toolchains, toolsets,
/// toolboxes, and the automation engine.
///
/// This is the single entry point for all tool operations on the Kogi platform.
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolManagementSystem {
    // ── Core registry ────────────────────────────────────────────────────────
    pub tools:        HashMap<ToolId, Tool>,
    pub providers:    HashMap<ToolProviderId, ToolProvider>,
    pub integrations: HashMap<ToolIntegrationId, ToolIntegration>,

    // ── Collections ──────────────────────────────────────────────────────────
    pub toolkits:   HashMap<ToolKitId, ToolKit>,
    pub toolchains: HashMap<ToolChainId, ToolChain>,
    pub toolsets:   HashMap<ToolSetId, ToolSet>,
    pub toolboxes:  HashMap<ToolBoxId, ToolBox>,

    // ── Automation engine ────────────────────────────────────────────────────
    pub automation: ToolAutomationSystem,

    // ── System metadata ──────────────────────────────────────────────────────
    pub created_at: DateTime<Utc>,
}

impl Default for ToolManagementSystem {
    fn default() -> Self { Self::new() }
}

impl ToolManagementSystem {
    pub fn new() -> Self {
        let mut system = Self {
            tools:        HashMap::new(),
            providers:    HashMap::new(),
            integrations: HashMap::new(),
            toolkits:     HashMap::new(),
            toolchains:   HashMap::new(),
            toolsets:     HashMap::new(),
            toolboxes:    HashMap::new(),
            automation:   ToolAutomationSystem::new(),
            created_at:   Utc::now(),
        };
        // Register the root Kogi provider
        let root = ToolProvider::root();
        system.providers.insert(root.id, root);
        system
    }

    // =========================================================================
    // Tools
    // =========================================================================

    /// Register a new tool.  Returns its ToolId.
    pub fn register_tool(&mut self, tool: Tool) -> ToolId {
        let id = tool.id();
        self.tools.insert(id, tool);
        id
    }

    pub fn get_tool(&self, id: &ToolId) -> ToolResult<&Tool> {
        self.tools.get(id).ok_or(ToolError::NotFound(*id))
    }

    pub fn get_tool_mut(&mut self, id: &ToolId) -> ToolResult<&mut Tool> {
        self.tools.get_mut(id).ok_or(ToolError::NotFound(*id))
    }

    pub fn activate_tool(&mut self, id: &ToolId) -> ToolResult<()> {
        self.get_tool_mut(id)?.activate()
    }

    pub fn deprecate_tool(&mut self, id: &ToolId) -> ToolResult<()> {
        self.get_tool_mut(id)?.deprecate();
        Ok(())
    }

    pub fn archive_tool(&mut self, id: &ToolId) -> ToolResult<()> {
        self.get_tool_mut(id)?.archive();
        Ok(())
    }

    /// Bump the patch version of a tool's assembly.
    pub fn bump_tool_version(
        &mut self,
        id: &ToolId,
        part: VersionPart,
        author: &str,
        message: &str,
    ) -> ToolResult<String> {
        let tool = self.get_tool_mut(id)?;
        let new_ver = match part {
            VersionPart::Patch => tool.assembly.metadata.version_history.bump_patch(author, message),
            VersionPart::Minor => tool.assembly.metadata.version_history.bump_minor(author, message),
            VersionPart::Major => tool.assembly.metadata.version_history.bump_major(author, message),
        };
        tool.assembly.metadata.touch();
        Ok(new_ver)
    }

    /// Query tools by category, type, status, or tag.
    pub fn query_tools(&self, filter: ToolFilter) -> Vec<&Tool> {
        self.tools.values().filter(|t| filter.matches(t)).collect()
    }

    // =========================================================================
    // Providers
    // =========================================================================

    pub fn register_provider(&mut self, provider: ToolProvider) -> ToolProviderId {
        let id = provider.id;
        self.providers.insert(id, provider);
        id
    }

    pub fn get_provider(&self, id: &ToolProviderId) -> ToolResult<&ToolProvider> {
        self.providers.get(id).ok_or_else(|| ToolError::ProviderNotFound(id.to_string()))
    }

    pub fn root_provider_id(&self) -> ToolProviderId { Uuid::nil() }

    /// Look up a ToolProvider that wraps a specific provider.rs ProviderRecord.
    pub fn find_provider_by_record_id(&self, record_id: &str) -> Option<&ToolProvider> {
        self.providers.values().find(|p| {
            p.provider_record_id.as_deref() == Some(record_id)
        })
    }

    // =========================================================================
    // Integrations
    // =========================================================================

    pub fn create_integration(&mut self, integration: ToolIntegration) -> ToolIntegrationId {
        let id = integration.id;
        // Record on the tool
        if let Ok(tool) = self.get_tool_mut(&integration.tool_id) {
            tool.integration_ids.push(id);
        }
        self.integrations.insert(id, integration);
        id
    }

    pub fn get_integration(&self, id: &ToolIntegrationId) -> ToolResult<&ToolIntegration> {
        self.integrations.get(id).ok_or(ToolError::IntegrationNotFound(*id))
    }

    pub fn get_integration_mut(&mut self, id: &ToolIntegrationId) -> ToolResult<&mut ToolIntegration> {
        self.integrations.get_mut(id).ok_or(ToolError::IntegrationNotFound(*id))
    }

    pub fn activate_integration(&mut self, id: &ToolIntegrationId) -> ToolResult<()> {
        self.get_integration_mut(id)?.activate();
        Ok(())
    }

    pub fn integrations_for_tool(&self, tool_id: &ToolId) -> Vec<&ToolIntegration> {
        self.integrations.values().filter(|i| &i.tool_id == tool_id).collect()
    }

    // =========================================================================
    // ToolKits
    // =========================================================================

    pub fn create_toolkit(&mut self, kit: ToolKit) -> ToolKitId {
        let id = kit.id;
        self.toolkits.insert(id, kit);
        id
    }

    pub fn get_toolkit(&self, id: &ToolKitId) -> ToolResult<&ToolKit> {
        self.toolkits.get(id).ok_or(ToolError::ToolKitNotFound(*id))
    }

    pub fn get_toolkit_mut(&mut self, id: &ToolKitId) -> ToolResult<&mut ToolKit> {
        self.toolkits.get_mut(id).ok_or(ToolError::ToolKitNotFound(*id))
    }

    /// Add a tool to a toolkit.  Updates membership records on both sides.
    pub fn add_tool_to_kit(&mut self, kit_id: &ToolKitId, tool_id: ToolId) -> ToolResult<()> {
        self.get_tool(&tool_id)?; // validate tool exists
        let kit = self.get_toolkit_mut(kit_id)?;
        kit.add_tool(tool_id);
        if let Ok(tool) = self.tools.get_mut(&tool_id) {
            tool.toolkit_memberships.push(*kit_id);
        }
        Ok(())
    }

    // =========================================================================
    // ToolChains
    // =========================================================================

    pub fn create_toolchain(&mut self, chain: ToolChain) -> ToolChainId {
        let id = chain.id;
        self.toolchains.insert(id, chain);
        id
    }

    pub fn get_toolchain(&self, id: &ToolChainId) -> ToolResult<&ToolChain> {
        self.toolchains.get(id).ok_or(ToolError::ToolChainNotFound(*id))
    }

    pub fn get_toolchain_mut(&mut self, id: &ToolChainId) -> ToolResult<&mut ToolChain> {
        self.toolchains.get_mut(id).ok_or(ToolError::ToolChainNotFound(*id))
    }

    /// Add a step to a toolchain, validating that the tool exists.
    pub fn add_step_to_chain(&mut self, chain_id: &ToolChainId, step: ToolChainStep) -> ToolResult<()> {
        self.get_tool(&step.tool_id)?; // validate
        let chain = self.get_toolchain_mut(chain_id)?;
        chain.add_step(step)?;
        Ok(())
    }

    // =========================================================================
    // ToolSets
    // =========================================================================

    pub fn create_toolset(&mut self, set: ToolSet) -> ToolSetId {
        let id = set.id;
        self.toolsets.insert(id, set);
        id
    }

    pub fn get_toolset(&self, id: &ToolSetId) -> ToolResult<&ToolSet> {
        self.toolsets.get(id).ok_or(ToolError::ToolSetNotFound(*id))
    }

    pub fn get_toolset_mut(&mut self, id: &ToolSetId) -> ToolResult<&mut ToolSet> {
        self.toolsets.get_mut(id).ok_or(ToolError::ToolSetNotFound(*id))
    }

    /// Attach a toolkit to a toolset.
    pub fn attach_kit_to_set(&mut self, set_id: &ToolSetId, kit_id: ToolKitId) -> ToolResult<()> {
        self.get_toolkit(&kit_id)?; // validate
        let set = self.get_toolset_mut(set_id)?;
        set.add_toolkit(kit_id);
        if let Ok(kit) = self.toolkits.get_mut(&kit_id) {
            kit.toolset_id = Some(*set_id);
        }
        Ok(())
    }

    /// Attach a toolchain to a toolset.
    pub fn attach_chain_to_set(&mut self, set_id: &ToolSetId, chain_id: ToolChainId) -> ToolResult<()> {
        self.get_toolchain(&chain_id)?; // validate
        let set = self.get_toolset_mut(set_id)?;
        set.add_toolchain(chain_id);
        if let Ok(chain) = self.toolchains.get_mut(&chain_id) {
            chain.toolset_id = Some(*set_id);
        }
        Ok(())
    }

    // =========================================================================
    // ToolBoxes
    // =========================================================================

    pub fn create_toolbox(&mut self, toolbox: ToolBox) -> ToolBoxId {
        let id = toolbox.id;
        self.toolboxes.insert(id, toolbox);
        id
    }

    pub fn get_toolbox(&self, id: &ToolBoxId) -> ToolResult<&ToolBox> {
        self.toolboxes.get(id).ok_or(ToolError::ToolBoxNotFound(*id))
    }

    pub fn get_toolbox_mut(&mut self, id: &ToolBoxId) -> ToolResult<&mut ToolBox> {
        self.toolboxes.get_mut(id).ok_or(ToolError::ToolBoxNotFound(*id))
    }

    /// Add a ToolSet to a ToolBox.
    pub fn add_set_to_box(&mut self, box_id: &ToolBoxId, set_id: ToolSetId) -> ToolResult<()> {
        self.get_toolset(&set_id)?;
        let tb = self.get_toolbox_mut(box_id)?;
        tb.add_toolset(set_id);
        if let Ok(set) = self.toolsets.get_mut(&set_id) {
            set.toolbox_id = Some(*box_id);
        }
        Ok(())
    }

    /// Attach a toolbox to a kogi platform element.
    pub fn attach_toolbox(
        &mut self,
        box_id: &ToolBoxId,
        element_type: ToolBoxElementType,
        element_id: PlatformElementId,
    ) -> ToolResult<()> {
        let tb = self.get_toolbox_mut(box_id)?;
        tb.attach_to_element(element_type, element_id);
        Ok(())
    }

    // =========================================================================
    // Automation
    // =========================================================================

    pub fn create_orchestration(&mut self, orch: ToolOrchestration) -> ToolOrchestrationId {
        self.automation.create_orchestration(orch)
    }

    pub fn get_orchestration(&self, id: &ToolOrchestrationId) -> ToolResult<&ToolOrchestration> {
        self.automation.get_orchestration(id)
    }

    pub fn create_workflow(&mut self, wf: ToolWorkflow) -> ToolWorkflowId {
        self.automation.create_workflow(wf)
    }

    pub fn get_workflow(&self, id: &ToolWorkflowId) -> ToolResult<&ToolWorkflow> {
        self.automation.get_workflow(id)
    }

    pub fn attach_workflow_to_orchestration(
        &mut self,
        orch_id: &ToolOrchestrationId,
        workflow: ToolWorkflow,
    ) -> ToolResult<ToolWorkflowId> {
        self.automation.attach_workflow_to_orchestration(orch_id, workflow)
    }

    pub fn enqueue_workflow(&mut self, workflow_id: ToolWorkflowId) -> ToolResult<()> {
        self.automation.enqueue(workflow_id)
    }

    pub fn execute_next_workflow(&mut self) -> ToolResult<Option<ToolWorkflowId>> {
        self.automation.execute_next()
    }

    // =========================================================================
    // Cross-cutting helpers
    // =========================================================================

    /// Full snapshot of tool counts.
    pub fn snapshot(&self) -> ToolSystemSnapshot {
        ToolSystemSnapshot {
            total_tools:        self.tools.len(),
            active_tools:       self.tools.values().filter(|t| t.status == ToolStatus::Active).count(),
            total_providers:    self.providers.len(),
            total_integrations: self.integrations.len(),
            active_integrations: self.integrations.values()
                .filter(|i| i.status == ToolIntegrationStatus::Active).count(),
            total_toolkits:     self.toolkits.len(),
            total_toolchains:   self.toolchains.len(),
            total_toolsets:     self.toolsets.len(),
            total_toolboxes:    self.toolboxes.len(),
            total_orchestrations: self.automation.orchestrations.len(),
            total_workflows:    self.automation.workflows.len(),
            queued_workflows:   self.automation.execution_queue.len(),
        }
    }

    /// All tools belonging to a specific ToolBox (via all its ToolSets / Kits / Chains).
    pub fn tools_in_box(&self, box_id: &ToolBoxId) -> ToolResult<Vec<&Tool>> {
        let tb = self.get_toolbox(box_id)?;
        let mut ids: HashSet<ToolId> = tb.loose_tool_ids.clone();

        for set_id in &tb.toolset_ids {
            if let Ok(set) = self.get_toolset(set_id) {
                ids.extend(set.ungrouped_tool_ids.iter().copied());
                for kit_id in &set.toolkit_ids {
                    if let Ok(kit) = self.get_toolkit(kit_id) {
                        ids.extend(kit.tool_ids.iter().copied());
                    }
                }
                for chain_id in &set.toolchain_ids {
                    if let Ok(chain) = self.get_toolchain(chain_id) {
                        ids.extend(chain.tool_ids());
                    }
                }
            }
        }

        Ok(ids.iter().filter_map(|id| self.tools.get(id)).collect())
    }
}

// =============================================================================
// §19 — SUPPORTING TYPES
// =============================================================================

/// Semver bump level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VersionPart { Major, Minor, Patch }

/// Filter criteria for tool queries.
#[derive(Debug, Clone, Default)]
pub struct ToolFilter {
    pub category:   Option<ToolCategory>,
    pub tool_type:  Option<ToolType>,
    pub tool_class: Option<ToolClass>,
    pub status:     Option<ToolStatus>,
    pub tag:        Option<String>,
    pub provider_id: Option<ToolProviderId>,
    pub is_template: Option<bool>,
}

impl ToolFilter {
    fn matches(&self, tool: &Tool) -> bool {
        if let Some(ref cat) = self.category {
            if tool.category() != cat { return false; }
        }
        if let Some(ref tt) = self.tool_type {
            if tool.tool_type() != tt { return false; }
        }
        if let Some(ref tc) = self.tool_class {
            if tool.tool_class() != tc { return false; }
        }
        if let Some(ref st) = self.status {
            if &tool.status != st { return false; }
        }
        if let Some(ref tag) = self.tag {
            if !tool.assembly.metadata.tags.contains(tag) { return false; }
        }
        if let Some(pid) = self.provider_id {
            if tool.provider_id != pid { return false; }
        }
        if let Some(is_tmpl) = self.is_template {
            if tool.assembly.metadata.is_template != is_tmpl { return false; }
        }
        true
    }
}

/// High-level count snapshot of the TMS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSystemSnapshot {
    pub total_tools: usize,
    pub active_tools: usize,
    pub total_providers: usize,
    pub total_integrations: usize,
    pub active_integrations: usize,
    pub total_toolkits: usize,
    pub total_toolchains: usize,
    pub total_toolsets: usize,
    pub total_toolboxes: usize,
    pub total_orchestrations: usize,
    pub total_workflows: usize,
    pub queued_workflows: usize,
}

// =============================================================================
// §20 — BUILDER API
// =============================================================================

/// Fluent builder for constructing a complete Tool with its full assembly.
pub struct ToolBuilder {
    system_name: String,
    token: String,
    brief: String,
    description: String,
    category: ToolCategory,
    tool_type: ToolType,
    tool_class: ToolClass,
    provider_id: ToolProviderId,
    provider_record_id: Option<ProviderRecordId>,
    provider_kind: ToolProviderKind,
    created_by: String,
    tags: ToolTags,
    labels: ToolLabels,
    capabilities: Vec<String>,
    use_cases: Vec<String>,
    config: HashMap<String, serde_json::Value>,
    is_template: bool,
    is_public: bool,
    components: Vec<AssemblyComponent>,
}

impl ToolBuilder {
    pub fn new(
        system_name: impl Into<String>,
        token: impl Into<String>,
        brief: impl Into<String>,
        category: ToolCategory,
        tool_type: ToolType,
        tool_class: ToolClass,
        provider_id: ToolProviderId,
        created_by: impl Into<String>,
    ) -> Self {
        Self {
            system_name: system_name.into(),
            token: token.into(),
            brief: brief.into(),
            description: String::new(),
            category,
            tool_type,
            tool_class,
            provider_id,
            provider_record_id: None,
            provider_kind: ToolProviderKind::Root,
            created_by: created_by.into(),
            tags: HashSet::new(),
            labels: HashMap::new(),
            capabilities: Vec::new(),
            use_cases: Vec::new(),
            config: HashMap::new(),
            is_template: false,
            is_public: false,
            components: Vec::new(),
        }
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into(); self
    }
    pub fn provider_record(mut self, id: impl Into<String>, kind: ToolProviderKind) -> Self {
        self.provider_record_id = Some(id.into());
        self.provider_kind = kind;
        self
    }
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.insert(tag.into()); self
    }
    pub fn label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into()); self
    }
    pub fn capability(mut self, cap: impl Into<String>) -> Self {
        self.capabilities.push(cap.into()); self
    }
    pub fn use_case(mut self, uc: impl Into<String>) -> Self {
        self.use_cases.push(uc.into()); self
    }
    pub fn config(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.config.insert(key.into(), value); self
    }
    pub fn template(mut self) -> Self { self.is_template = true; self }
    pub fn public(mut self) -> Self { self.is_public = true; self }
    pub fn component(mut self, c: AssemblyComponent) -> Self {
        self.components.push(c); self
    }

    pub fn build(self) -> Tool {
        let mut meta = ToolMetadata::new(
            self.system_name.clone(),
            self.token,
            self.category,
            self.tool_type,
            self.tool_class,
            self.created_by.clone(),
        );
        meta.tags = self.tags;
        meta.labels = self.labels;
        meta.provider_record_id = self.provider_record_id;
        meta.provider_kind = self.provider_kind;
        meta.is_template = self.is_template;
        meta.is_public = self.is_public;

        let mut assembly = ToolAssembly::new(
            self.brief.clone(),
            self.description.clone(),
            meta,
            self.created_by,
        );
        assembly.info.capabilities = self.capabilities;
        assembly.info.use_cases = self.use_cases;
        assembly.info.short_description = self.brief;
        assembly.info.long_description = self.description;
        assembly.data.config = self.config;
        assembly.components = self.components;

        Tool::new(assembly, self.provider_id)
    }
}

// =============================================================================
// §21 — TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn build_cmt_tool(tms: &mut ToolManagementSystem) -> ToolId {
        // Simulate GitHub as a third-party provider linked to provider.rs record
        let github_provider = ToolProvider::new(
            "GitHub",
            ToolProviderKind::ThirdParty,
            Some("provider-001".to_string()),
        );
        let github_pid = tms.register_provider(github_provider);

        let tool = ToolBuilder::new(
            "code_management_tool",
            "cmt",
            "Manages source code via remote VCS providers",
            ToolCategory::Development,
            ToolType::CodeManagement,
            ToolClass::Connector,
            github_pid,
            "user-alice",
        )
        .description("A code management tool with CI/CD capabilities backed by GitHub.")
        .tag("vcs")
        .tag("cicd")
        .label("env", "production")
        .capability("push")
        .capability("pull_request")
        .capability("webhook_trigger")
        .use_case("Remote build step in a CICD pipeline")
        .provider_record("provider-001", ToolProviderKind::ThirdParty)
        .build();

        tms.register_tool(tool)
    }

    #[test]
    fn test_register_and_activate_tool() {
        let mut tms = ToolManagementSystem::new();
        let cmt_id = build_cmt_tool(&mut tms);

        assert!(tms.get_tool(&cmt_id).is_ok());
        assert_eq!(tms.get_tool(&cmt_id).unwrap().status, ToolStatus::Draft);

        tms.activate_tool(&cmt_id).unwrap();
        assert_eq!(tms.get_tool(&cmt_id).unwrap().status, ToolStatus::Active);
    }

    #[test]
    fn test_version_bump() {
        let mut tms = ToolManagementSystem::new();
        let id = build_cmt_tool(&mut tms);
        tms.activate_tool(&id).unwrap();

        let v1 = tms.bump_tool_version(&id, VersionPart::Minor, "alice", "Added webhook support").unwrap();
        assert_eq!(v1, "0.2.0");

        let v2 = tms.bump_tool_version(&id, VersionPart::Patch, "alice", "Fixed auth bug").unwrap();
        assert_eq!(v2, "0.2.1");

        let v3 = tms.bump_tool_version(&id, VersionPart::Major, "alice", "Breaking API overhaul").unwrap();
        assert_eq!(v3, "1.0.0");
    }

    #[test]
    fn test_integration_lifecycle() {
        let mut tms = ToolManagementSystem::new();
        let cmt_id = build_cmt_tool(&mut tms);
        tms.activate_tool(&cmt_id).unwrap();

        let provider_id = tms.get_tool(&cmt_id).unwrap().provider_id;
        let mut integration = ToolIntegration::new(
            cmt_id,
            provider_id,
            "GitHub CMT Integration",
            "user-alice",
        );
        integration.add_target(IntegrationTarget::Global);
        integration.scopes.push("repo".to_string());

        let int_id = tms.create_integration(integration);
        assert_eq!(tms.get_integration(&int_id).unwrap().status, ToolIntegrationStatus::Pending);

        tms.activate_integration(&int_id).unwrap();
        assert_eq!(tms.get_integration(&int_id).unwrap().status, ToolIntegrationStatus::Active);

        let tool_integrations = tms.integrations_for_tool(&cmt_id);
        assert_eq!(tool_integrations.len(), 1);
    }

    #[test]
    fn test_toolkit_management() {
        let mut tms = ToolManagementSystem::new();
        let cmt_id = build_cmt_tool(&mut tms);
        tms.activate_tool(&cmt_id).unwrap();

        let mut kit = ToolKit::new("Code Management Kit", "Manage source code workflows", "alice");
        kit.activate();
        let kit_id = tms.create_toolkit(kit);

        tms.add_tool_to_kit(&kit_id, cmt_id).unwrap();
        assert!(tms.get_toolkit(&kit_id).unwrap().contains(&cmt_id));
        assert_eq!(tms.get_toolkit(&kit_id).unwrap().len(), 1);
        assert!(tms.get_tool(&cmt_id).unwrap().toolkit_memberships.contains(&kit_id));
    }

    #[test]
    fn test_toolchain_pipeline() {
        let mut tms = ToolManagementSystem::new();
        let cmt_id = build_cmt_tool(&mut tms);
        tms.activate_tool(&cmt_id).unwrap();

        let mut chain = ToolChain::new(
            "Software Build & Delivery",
            "Automate build, test, and deploy pipeline",
            "alice",
        );
        chain.activate();
        let chain_id = tms.create_toolchain(chain);

        let step = ToolChainStep::new(0, "CICD Remote Build Step", cmt_id);
        tms.add_step_to_chain(&chain_id, step).unwrap();

        let chain = tms.get_toolchain(&chain_id).unwrap();
        assert_eq!(chain.len(), 1);
        assert_eq!(chain.steps[0].tool_id, cmt_id);
        assert!(chain.tool_ids().contains(&cmt_id));
    }

    #[test]
    fn test_toolset_composition() {
        let mut tms = ToolManagementSystem::new();
        let cmt_id = build_cmt_tool(&mut tms);
        tms.activate_tool(&cmt_id).unwrap();

        // Build a kit
        let mut kit = ToolKit::new("Code Mgmt Kit", "code", "alice");
        kit.activate();
        let kit_id = tms.create_toolkit(kit);
        tms.add_tool_to_kit(&kit_id, cmt_id).unwrap();

        // Build a chain
        let mut chain = ToolChain::new("Build Delivery", "CICD pipeline", "alice");
        chain.activate();
        let chain_id = tms.create_toolchain(chain);
        let step = ToolChainStep::new(0, "CMT build step", cmt_id);
        tms.add_step_to_chain(&chain_id, step).unwrap();

        // Compose into a toolset
        let set = ToolSet::new("DevOps Tools", "DevOps", "alice");
        let set_id = tms.create_toolset(set);
        tms.attach_kit_to_set(&set_id, kit_id).unwrap();
        tms.attach_chain_to_set(&set_id, chain_id).unwrap();

        let set = tms.get_toolset(&set_id).unwrap();
        assert!(set.toolkit_ids.contains(&kit_id));
        assert!(set.toolchain_ids.contains(&chain_id));
    }

    #[test]
    fn test_toolbox_full_stack() {
        let mut tms = ToolManagementSystem::new();
        let cmt_id = build_cmt_tool(&mut tms);
        tms.activate_tool(&cmt_id).unwrap();

        // Kit → Set → Box
        let kit_id = {
            let mut kit = ToolKit::new("Code Mgmt Kit", "code", "alice");
            kit.activate();
            let id = tms.create_toolkit(kit);
            tms.add_tool_to_kit(&id, cmt_id).unwrap();
            id
        };
        let set_id = {
            let set = ToolSet::new("DevOps Tools", "DevOps", "alice");
            let id = tms.create_toolset(set);
            tms.attach_kit_to_set(&id, kit_id).unwrap();
            id
        };
        let box_id = {
            let tb = ToolBox::new("Software Development Toolbox", "alice");
            let id = tms.create_toolbox(tb);
            tms.add_set_to_box(&id, set_id).unwrap();
            id
        };

        // Attach to a fake portfolio element
        let portfolio_id = Uuid::new_v4();
        tms.attach_toolbox(&box_id, ToolBoxElementType::Portfolio, portfolio_id).unwrap();

        // Enumerate all tools in the box
        let tools = tms.tools_in_box(&box_id).unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].id(), cmt_id);

        // Check attachment
        let tb = tms.get_toolbox(&box_id).unwrap();
        assert_eq!(tb.attached_to.len(), 1);
        assert_eq!(tb.attached_to[0].element_id, portfolio_id);
    }

    #[test]
    fn test_orchestration_workflow_execution() {
        let mut tms = ToolManagementSystem::new();
        let cmt_id = build_cmt_tool(&mut tms);
        tms.activate_tool(&cmt_id).unwrap();

        // Orchestration
        let mut orch = ToolOrchestration::new(
            "CICD Manager",
            "Automate software build and delivery across all projects",
            "alice",
        );
        orch.activate();
        let orch_id = tms.create_orchestration(orch);

        // Workflow
        let mut wf = ToolWorkflow::new(
            "Software Alpha Build",
            "Build and deploy the Alpha release",
            orch_id,
            "alice",
        );
        wf.activate();
        let task = ToolTask::new("CMT CICD Step", wf.id, cmt_id, 0);
        wf.add_task(task);

        let wf_id = tms.attach_workflow_to_orchestration(&orch_id, wf).unwrap();
        tms.enqueue_workflow(wf_id).unwrap();

        let executed = tms.execute_next_workflow().unwrap();
        assert_eq!(executed, Some(wf_id));

        let wf = tms.get_workflow(&wf_id).unwrap();
        assert_eq!(wf.status, ToolWorkflowStatus::Completed);
        assert_eq!(wf.run_count, 1);
        assert_eq!(wf.success_count, 1);
        assert!((wf.success_rate() - 100.0).abs() < 0.001);
        assert!(wf.all_tasks_complete());
    }

    #[test]
    fn test_tool_query_filter() {
        let mut tms = ToolManagementSystem::new();
        let cmt_id = build_cmt_tool(&mut tms);
        tms.activate_tool(&cmt_id).unwrap();

        let results = tms.query_tools(ToolFilter {
            category: Some(ToolCategory::Development),
            status: Some(ToolStatus::Active),
            ..Default::default()
        });
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id(), cmt_id);

        let empty = tms.query_tools(ToolFilter {
            category: Some(ToolCategory::Finance),
            ..Default::default()
        });
        assert!(empty.is_empty());
    }

    #[test]
    fn test_snapshot() {
        let mut tms = ToolManagementSystem::new();
        let cmt_id = build_cmt_tool(&mut tms);
        tms.activate_tool(&cmt_id).unwrap();

        let snap = tms.snapshot();
        assert_eq!(snap.total_tools, 1);
        assert_eq!(snap.active_tools, 1);
        assert!(snap.total_providers >= 2); // root + github
    }

    #[test]
    fn test_provider_lookup_by_record_id() {
        let mut tms = ToolManagementSystem::new();
        build_cmt_tool(&mut tms);

        let found = tms.find_provider_by_record_id("provider-001");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "GitHub");
    }

    #[test]
    fn test_toolchain_step_reordering() {
        let mut tms = ToolManagementSystem::new();
        let cmt_id = build_cmt_tool(&mut tms);
        tms.activate_tool(&cmt_id).unwrap();

        let chain = ToolChain::new("Test Chain", "test", "alice");
        let chain_id = tms.create_toolchain(chain);

        tms.add_step_to_chain(&chain_id, ToolChainStep::new(0, "Step A", cmt_id)).unwrap();
        tms.add_step_to_chain(&chain_id, ToolChainStep::new(1, "Step B", cmt_id)).unwrap();
        tms.add_step_to_chain(&chain_id, ToolChainStep::new(2, "Step C", cmt_id)).unwrap();

        let chain = tms.get_toolchain(&chain_id).unwrap();
        assert_eq!(chain.steps[0].name, "Step A");
        assert_eq!(chain.steps[1].name, "Step B");
        assert_eq!(chain.steps[2].name, "Step C");
        assert_eq!(chain.steps[2].step_index, 2);
    }

    #[test]
    fn test_workflow_task_failure_and_retry() {
        let mut tms = ToolManagementSystem::new();
        let cmt_id = build_cmt_tool(&mut tms);
        tms.activate_tool(&cmt_id).unwrap();

        let orch_id = tms.create_orchestration(ToolOrchestration::new("O", "goal", "alice"));
        let wf = ToolWorkflow::new("WF", "goal", orch_id, "alice");
        let wf_id = tms.create_workflow(wf);

        let wf = tms.automation.get_workflow_mut(&wf_id).unwrap();
        let mut task = ToolTask::new("Task 1", wf_id, cmt_id, 0);
        task.max_retries = 2;
        task.start();
        task.fail("network error");
        assert_eq!(task.status, ToolTaskStatus::Pending); // retrying
        assert_eq!(task.retry_count, 1);
        task.fail("network error");
        assert_eq!(task.status, ToolTaskStatus::Pending); // still retrying
        assert_eq!(task.retry_count, 2);
        task.fail("network error");
        assert_eq!(task.status, ToolTaskStatus::Failed); // exhausted
    }
}
