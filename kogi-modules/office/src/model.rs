use serde::{Deserialize, Serialize};

pub use crate::assistant_system::{AssistantSnapshot, NewAssistantSubscription};
use crate::assistant_system::AssistantSystem;
pub use crate::dashboard_system::DashboardSnapshot;
use crate::dashboard_system::DashboardSystem;
pub use crate::portfolio_system::{
    NewPortfolioItem, PortfolioSnapshot,
    // Component CRUD
    PortfolioComponentType, BookType, PortfolioComponent,
    // Graph
    GraphEdge,
    // Snapshots / checkpoints
    PortfolioCheckpoint,
    // Governance
    ResourceKind, ResourceAllocation, ApprovalRequest,
    // Computational models
    PortfolioHealthResult, ProjectMetricsResult, ProgramAlignmentResult,
    SubPortfolioRollupResult, ResourceUtilisationResult, AssetValueResult,
    ArtifactMaturityResult, BinderCoverageResult, BookConsistencyResult,
    FolderOrganisationResult, RecordIntegrityResult,
    ProjectMetricsInput, AssetValueInput, BookConsistencyInput,
    ArtifactMaturityInput, BinderCoverageInput, FolderOrganisationInput,
};
use crate::portfolio_system::PortfolioSystem;
pub use crate::timeline_system::{NewTimelineEvent, TimelineSnapshot};
use crate::timeline_system::TimelineSystem;
pub use crate::workspace_system::{NewWorkspaceStory, WorkspaceSnapshot};
use crate::workspace_system::WorkspaceSystem;

use crate::os_bridge::{
    default_os_compatibility_snapshot, EventRecord, EventType, OsCompatibilitySnapshot,
};

use std::collections::{HashMap, HashSet};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// =============================================================================
// Shared types
// =============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OfficeViewInfo {
    pub id: String,
    pub title: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OfficeOverview {
    pub module: String,
    pub application: String,
    pub service: String,
    pub systems: Vec<String>,
    pub views: Vec<OfficeViewInfo>,
    pub integrations: Vec<String>,
    pub os_compatibility: OsCompatibilitySnapshot,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionResult {
    pub ok: bool,
    pub system: String,
    pub message: String,
    pub entity_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DashboardMutationResponse {
    pub result: ActionResult,
    pub dashboard: DashboardSnapshot,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortfolioMutationResponse {
    pub result: ActionResult,
    pub portfolio: PortfolioSnapshot,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimelineMutationResponse {
    pub result: ActionResult,
    pub timeline: TimelineSnapshot,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceMutationResponse {
    pub result: ActionResult,
    pub workspace: WorkspaceSnapshot,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssistantMutationResponse {
    pub result: ActionResult,
    pub assistant: AssistantSnapshot,
}

// =============================================================================
// Portfolio-specific FFI request / response wrappers
// =============================================================================

/// Unified FFI response: JSON-encoded result or an error string.
#[derive(Serialize, Deserialize)]
struct FfiResponse {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl FfiResponse {
    fn ok(data: serde_json::Value) -> Self {
        Self { ok: true, data: Some(data), error: None }
    }
    fn err(msg: impl Into<String>) -> Self {
        Self { ok: false, data: None, error: Some(msg.into()) }
    }
    fn into_json(self) -> *mut c_char {
        let s = serde_json::to_string(&self).unwrap_or_else(|_| r#"{"ok":false,"error":"serialize"}"#.into());
        CString::new(s).unwrap_or_default().into_raw()
    }
}

// Deserialize a raw C string pointer into T, returning an Err FfiResponse on failure.
fn parse_arg<T: for<'de> Deserialize<'de>>(raw: *const c_char) -> Result<T, *mut c_char> {
    if raw.is_null() {
        return Err(FfiResponse::err("null argument").into_json());
    }
    let cstr = unsafe { CStr::from_ptr(raw) };
    let s = cstr.to_str().map_err(|e| FfiResponse::err(e.to_string()).into_json())?;
    serde_json::from_str::<T>(s).map_err(|e| FfiResponse::err(format!("parse: {e}")).into_json())
}

// =============================================================================
// OfficeModule (existing Rust runtime)
// =============================================================================

#[derive(Clone, Debug)]
pub struct OfficeModule {
    dashboard: DashboardSystem,
    portfolio: PortfolioSystem,
    timeline: TimelineSystem,
    workspace: WorkspaceSystem,
    assistant: AssistantSystem,
    os_compatibility: OsCompatibilitySnapshot,
}

impl OfficeModule {
    pub fn mvp() -> Self {
        Self {
            dashboard: DashboardSystem::mvp(),
            portfolio: PortfolioSystem::mvp(),
            timeline: TimelineSystem::mvp(),
            workspace: WorkspaceSystem::mvp(),
            assistant: AssistantSystem::mvp(),
            os_compatibility: default_os_compatibility_snapshot(),
        }
    }

    pub fn overview(&self) -> OfficeOverview {
        OfficeOverview {
            module: "kogi.office".to_string(),
            application: "Kogi Office".to_string(),
            service: "kogi-services/go/services/office".to_string(),
            systems: vec![
                "DashboardSystem".to_string(),
                "PortfolioSystem".to_string(),
                "TimelineSystem".to_string(),
                "WorkspaceSystem".to_string(),
                "AssistantSystem".to_string(),
            ],
            views: vec![
                OfficeViewInfo { id: "dashboard".to_string(), title: "Office Dashboard".to_string(), status: "active".to_string() },
                OfficeViewInfo { id: "portfolio".to_string(), title: "Office Portfolio".to_string(), status: "active".to_string() },
                OfficeViewInfo { id: "timeline".to_string(), title: "Office Timeline".to_string(), status: "active".to_string() },
                OfficeViewInfo { id: "workspace".to_string(), title: "Office Workspace".to_string(), status: "active".to_string() },
                OfficeViewInfo { id: "assistant".to_string(), title: "Office Assistant".to_string(), status: "active".to_string() },
            ],
            integrations: vec![
                "jira".to_string(), "monday".to_string(), "openai".to_string(),
                "gitlab".to_string(), "github".to_string(),
            ],
            os_compatibility: self.os_compatibility.clone(),
        }
    }

    // ── Snapshot accessors ────────────────────────────────────────────────

    pub fn dashboard_snapshot(&self) -> DashboardSnapshot   { self.dashboard.snapshot() }
    pub fn portfolio_snapshot(&self) -> PortfolioSnapshot   { self.portfolio.snapshot() }
    pub fn timeline_snapshot(&self) -> TimelineSnapshot     { self.timeline.snapshot() }
    pub fn workspace_snapshot(&self) -> WorkspaceSnapshot   { self.workspace.snapshot() }
    pub fn assistant_snapshot(&self) -> AssistantSnapshot   { self.assistant.snapshot() }

    // ── Dashboard ─────────────────────────────────────────────────────────

    pub fn ack_dashboard_notification(&mut self, notification_id: &str) -> DashboardMutationResponse {
        let acknowledged = self.dashboard.acknowledge_notification(notification_id);
        if acknowledged {
            self.dashboard.push_event("events", &format!("Notification {notification_id} acknowledged"));
            self.record_os_event(EventType::TaskCompleted, "dashboard.system",
                &format!("notification {notification_id} acknowledged"));
        }
        DashboardMutationResponse {
            result: ActionResult {
                ok: acknowledged,
                system: "DashboardSystem".to_string(),
                message: if acknowledged { "notification acknowledged".to_string() } else { "notification not found".to_string() },
                entity_id: Some(notification_id.to_string()),
            },
            dashboard: self.dashboard_snapshot(),
        }
    }

    // ── Portfolio (legacy add_item path) ──────────────────────────────────

    pub fn create_portfolio_item(&mut self, request: NewPortfolioItem) -> PortfolioMutationResponse {
        let item = self.portfolio.add_item(request).expect("add_item failed");
        let entity_label = format!("{:?}", item.component_type).to_lowercase();
        self.dashboard.add_attention_from_portfolio(&entity_label, &item.name);
        self.dashboard.push_event("portfolio", &format!("Portfolio item {} created", item.name));
        self.assistant.add_recommendation("Review newly added portfolio item");
        self.record_os_event(EventType::ItemAdded, "portfolio.system",
            &format!("portfolio item {} created", item.metadata.id));
        PortfolioMutationResponse {
            result: ActionResult {
                ok: true,
                system: "PortfolioSystem".to_string(),
                message: "portfolio item created".to_string(),
                entity_id: Some(item.metadata.id.clone()),
            },
            portfolio: self.portfolio_snapshot(),
        }
    }

    // ── Timeline ──────────────────────────────────────────────────────────

    pub fn create_timeline_event(&mut self, request: NewTimelineEvent) -> TimelineMutationResponse {
        let event = self.timeline.add_event(request.clone());
        self.dashboard.push_event("timeline", &format!("Timeline event {} scheduled", event.title));
        self.record_os_event(EventType::TaskCreated, "timeline.system",
            &format!("timeline event {} created", event.id));
        self.os_compatibility.scheduler_events.push(crate::os_bridge::CalendarEvent {
            id: event.id.clone(),
            title: event.title.clone(),
            description: format!("{} event", event.kind),
            start_time: event.scheduled_for.clone(),
            end_time: event.scheduled_for.clone(),
            all_day: false,
            owner_profile: "work".to_string(),
        });
        TimelineMutationResponse {
            result: ActionResult {
                ok: true,
                system: "TimelineSystem".to_string(),
                message: "timeline event created".to_string(),
                entity_id: Some(event.id.clone()),
            },
            timeline: self.timeline_snapshot(),
        }
    }

    // ── Workspace ─────────────────────────────────────────────────────────

    pub fn create_workspace_story(&mut self, request: NewWorkspaceStory) -> WorkspaceMutationResponse {
        let story = self.workspace.add_story(request);
        self.dashboard.push_event("workspace", &format!("Workspace story {} created", story.title));
        self.assistant.add_recommendation("Review and prioritize the new workspace story");
        self.record_os_event(EventType::TaskCreated, "workspace.system",
            &format!("workspace story {} created", story.id));
        WorkspaceMutationResponse {
            result: ActionResult {
                ok: true,
                system: "WorkspaceSystem".to_string(),
                message: "workspace story created".to_string(),
                entity_id: Some(story.id.clone()),
            },
            workspace: self.workspace_snapshot(),
        }
    }

    // ── Assistant ─────────────────────────────────────────────────────────

    pub fn create_assistant_subscription(&mut self, request: NewAssistantSubscription) -> AssistantMutationResponse {
        let topic = self.assistant.add_subscription(request);
        self.dashboard.push_event("assistant", &format!("Assistant subscribed to {topic}"));
        self.record_os_event(EventType::ConnectionAdded, "assistant.system",
            &format!("assistant subscription {topic} active"));
        AssistantMutationResponse {
            result: ActionResult {
                ok: true,
                system: "AssistantSystem".to_string(),
                message: "assistant subscription active".to_string(),
                entity_id: Some(topic),
            },
            assistant: self.assistant_snapshot(),
        }
    }

    // ── Internal ──────────────────────────────────────────────────────────

    fn record_os_event(&mut self, event_type: EventType, source: &str, message: &str) {
        let next = self.os_compatibility.event_bus_history.len() + 1;
        self.os_compatibility.event_bus_history.push(EventRecord {
            id: format!("evt-{:03}", next),
            event_type,
            source: source.to_string(),
            message: message.to_string(),
            timestamp: "2026-03-10T00:00:00Z".to_string(),
        });
    }

    // ── Direct PortfolioSystem accessors (used by FFI layer below) ────────

    pub fn portfolio_mut(&mut self) -> &mut PortfolioSystem {
        &mut self.portfolio
    }

    pub fn portfolio_ref(&self) -> &PortfolioSystem {
        &self.portfolio
    }
}

// =============================================================================
// FFI — extern "C" entry points exported from kogi_office.dll
//
// Calling convention
// ------------------
//   * Every function accepts JSON-encoded arguments as null-terminated C strings
//     and returns a heap-allocated null-terminated C string.
//   * The caller MUST free the returned pointer with `kogi_free_string`.
//   * All returned strings are UTF-8 encoded JSON conforming to FfiResponse.
//   * The DLL maintains a single process-wide OfficeModule instance protected
//     by a Mutex.  This is intentional: the Go service is single-process and
//     serialises all mutations through the service layer.
// =============================================================================

use std::sync::Mutex;

// Process-wide singleton.
static OFFICE: Mutex<Option<OfficeModule>> = Mutex::new(None);

fn with_office<F, R>(f: F) -> R
where
    F: FnOnce(&mut OfficeModule) -> R,
{
    let mut guard = OFFICE.lock().expect("office mutex poisoned");
    if guard.is_none() {
        *guard = Some(OfficeModule::mvp());
    }
    f(guard.as_mut().unwrap())
}

// ── Memory management ─────────────────────────────────────────────────────────

/// Free a string returned by any `kogi_*` function.
///
/// # Safety
/// `ptr` must be a pointer previously returned by a `kogi_*` function and must
/// not have been freed already.
#[no_mangle]
pub unsafe extern "C" fn kogi_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}

// ── Office overview / system snapshots ───────────────────────────────────────

/// Returns the full OfficeOverview as JSON.
#[no_mangle]
pub extern "C" fn kogi_office_overview() -> *mut c_char {
    with_office(|o| {
        match serde_json::to_value(o.overview()) {
            Ok(v)  => FfiResponse::ok(v).into_json(),
            Err(e) => FfiResponse::err(e.to_string()).into_json(),
        }
    })
}

/// Returns the DashboardSnapshot as JSON.
#[no_mangle]
pub extern "C" fn kogi_office_dashboard() -> *mut c_char {
    with_office(|o| match serde_json::to_value(o.dashboard_snapshot()) {
        Ok(v)  => FfiResponse::ok(v).into_json(),
        Err(e) => FfiResponse::err(e.to_string()).into_json(),
    })
}

/// Returns the PortfolioSnapshot as JSON.
#[no_mangle]
pub extern "C" fn kogi_office_portfolio_snapshot() -> *mut c_char {
    with_office(|o| match serde_json::to_value(o.portfolio_snapshot()) {
        Ok(v)  => FfiResponse::ok(v).into_json(),
        Err(e) => FfiResponse::err(e.to_string()).into_json(),
    })
}

/// Returns the TimelineSnapshot as JSON.
#[no_mangle]
pub extern "C" fn kogi_office_timeline() -> *mut c_char {
    with_office(|o| match serde_json::to_value(o.timeline_snapshot()) {
        Ok(v)  => FfiResponse::ok(v).into_json(),
        Err(e) => FfiResponse::err(e.to_string()).into_json(),
    })
}

/// Returns the WorkspaceSnapshot as JSON.
#[no_mangle]
pub extern "C" fn kogi_office_workspace() -> *mut c_char {
    with_office(|o| match serde_json::to_value(o.workspace_snapshot()) {
        Ok(v)  => FfiResponse::ok(v).into_json(),
        Err(e) => FfiResponse::err(e.to_string()).into_json(),
    })
}

/// Returns the AssistantSnapshot as JSON.
#[no_mangle]
pub extern "C" fn kogi_office_assistant() -> *mut c_char {
    with_office(|o| match serde_json::to_value(o.assistant_snapshot()) {
        Ok(v)  => FfiResponse::ok(v).into_json(),
        Err(e) => FfiResponse::err(e.to_string()).into_json(),
    })
}

// ── Dashboard mutations ───────────────────────────────────────────────────────

/// `args_json`: `{"notification_id": "notif-001"}`
#[no_mangle]
pub extern "C" fn kogi_ack_dashboard_notification(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { notification_id: String }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| {
        let r = o.ack_dashboard_notification(&args.notification_id);
        match serde_json::to_value(r) {
            Ok(v)  => FfiResponse::ok(v).into_json(),
            Err(e) => FfiResponse::err(e.to_string()).into_json(),
        }
    })
}

// ── Portfolio: component CRUD ─────────────────────────────────────────────────

/// `args_json`: `{"component_type": "Project", "name": "My Project"}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_create_component(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { component_type: String, name: String }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| {
        let ct = parse_component_type_str(&args.component_type);
        match o.portfolio_mut().create_component(ct, args.name) {
            Ok(comp) => match serde_json::to_value(comp) {
                Ok(v)  => FfiResponse::ok(v).into_json(),
                Err(e) => FfiResponse::err(e.to_string()).into_json(),
            },
            Err(e) => FfiResponse::err(e).into_json(),
        }
    })
}

/// `args_json`: `{"book_type": "Notebook", "name": "Sprint Notes"}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_create_book(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { book_type: String, name: String }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| {
        let bt = parse_book_type_str(&args.book_type);
        match o.portfolio_mut().create_book(bt, args.name) {
            Ok(comp) => match serde_json::to_value(comp) {
                Ok(v)  => FfiResponse::ok(v).into_json(),
                Err(e) => FfiResponse::err(e.to_string()).into_json(),
            },
            Err(e) => FfiResponse::err(e).into_json(),
        }
    })
}

/// `args_json`: `{"id": "comp-000001"}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_get_component(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { id: String }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| {
        match o.portfolio_ref().get_component(&args.id) {
            Some(comp) => match serde_json::to_value(comp) {
                Ok(v)  => FfiResponse::ok(v).into_json(),
                Err(e) => FfiResponse::err(e.to_string()).into_json(),
            },
            None => FfiResponse::err(format!("component '{}' not found", args.id)).into_json(),
        }
    })
}

/// Returns all components as a JSON array.
#[no_mangle]
pub extern "C" fn kogi_portfolio_all_components() -> *mut c_char {
    with_office(|o| {
        let comps: Vec<_> = o.portfolio_ref().all_components();
        match serde_json::to_value(comps) {
            Ok(v)  => FfiResponse::ok(v).into_json(),
            Err(e) => FfiResponse::err(e.to_string()).into_json(),
        }
    })
}

/// `args_json`: `{"component_type": "Project"}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_components_by_type(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { component_type: String }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| {
        let ct = parse_component_type_str(&args.component_type);
        let comps = o.portfolio_ref().components_by_type(&ct);
        match serde_json::to_value(comps) {
            Ok(v)  => FfiResponse::ok(v).into_json(),
            Err(e) => FfiResponse::err(e.to_string()).into_json(),
        }
    })
}

/// `args_json`: EditComponentReq — all fields except `id` are optional.
/// `{"id":"comp-000001","name":"New Name","status":"inactive","tags":["a"],"properties":{},"owner":"alice"}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_edit_component(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)]
    struct Args {
        id: String,
        name: Option<String>,
        status: Option<String>,
        tags: Option<Vec<String>>,
        properties: Option<HashMap<String, String>>,
        owner: Option<String>,
    }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| {
        match o.portfolio_mut().edit_component(&args.id, args.name, args.status, args.tags, args.properties, args.owner) {
            Ok(comp) => match serde_json::to_value(comp) {
                Ok(v)  => FfiResponse::ok(v).into_json(),
                Err(e) => FfiResponse::err(e.to_string()).into_json(),
            },
            Err(e) => FfiResponse::err(e).into_json(),
        }
    })
}

/// `args_json`: `{"id": "comp-000001"}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_remove_component(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { id: String }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| {
        match o.portfolio_mut().remove_component(&args.id) {
            Ok(comp) => match serde_json::to_value(comp) {
                Ok(v)  => FfiResponse::ok(v).into_json(),
                Err(e) => FfiResponse::err(e.to_string()).into_json(),
            },
            Err(e) => FfiResponse::err(e).into_json(),
        }
    })
}

/// `args_json`: `{"id": "comp-000001"}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_set_active(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { id: String }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| {
        match o.portfolio_mut().set_active_portfolio(&args.id) {
            Ok(()) => FfiResponse::ok(serde_json::json!({"active_portfolio_id": args.id})).into_json(),
            Err(e) => FfiResponse::err(e).into_json(),
        }
    })
}

// ── Portfolio: graph ──────────────────────────────────────────────────────────

/// `args_json`: `{"parent_id": "comp-000001", "child_id": "comp-000002"}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_add_hierarchy(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { parent_id: String, child_id: String }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| match o.portfolio_mut().add_hierarchy(&args.parent_id, &args.child_id) {
        Ok(edge_id) => FfiResponse::ok(serde_json::json!({"edge_id": edge_id})).into_json(),
        Err(e)      => FfiResponse::err(e).into_json(),
    })
}

/// `args_json`: `{"parent_id": "...", "child_id": "..."}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_remove_hierarchy(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { parent_id: String, child_id: String }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| match o.portfolio_mut().remove_hierarchy(&args.parent_id, &args.child_id) {
        Ok(())  => FfiResponse::ok(serde_json::json!({"ok": true})).into_json(),
        Err(e)  => FfiResponse::err(e).into_json(),
    })
}

/// `args_json`: `{"from_id": "...", "to_id": "..."}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_add_dependency(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { from_id: String, to_id: String }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| match o.portfolio_mut().add_dependency(&args.from_id, &args.to_id) {
        Ok(edge_id) => FfiResponse::ok(serde_json::json!({"edge_id": edge_id})).into_json(),
        Err(e)      => FfiResponse::err(e).into_json(),
    })
}

/// `args_json`: `{"from_id": "...", "to_id": "..."}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_remove_dependency(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { from_id: String, to_id: String }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| match o.portfolio_mut().remove_dependency(&args.from_id, &args.to_id) {
        Ok(())  => FfiResponse::ok(serde_json::json!({"ok": true})).into_json(),
        Err(e)  => FfiResponse::err(e).into_json(),
    })
}

/// `args_json`: `{"a_id": "...", "b_id": "..."}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_add_link(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { a_id: String, b_id: String }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| match o.portfolio_mut().add_link(&args.a_id, &args.b_id) {
        Ok(edge_id) => FfiResponse::ok(serde_json::json!({"edge_id": edge_id})).into_json(),
        Err(e)      => FfiResponse::err(e).into_json(),
    })
}

/// `args_json`: `{"a_id": "...", "b_id": "..."}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_remove_link(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { a_id: String, b_id: String }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| match o.portfolio_mut().remove_link(&args.a_id, &args.b_id) {
        Ok(())  => FfiResponse::ok(serde_json::json!({"ok": true})).into_json(),
        Err(e)  => FfiResponse::err(e).into_json(),
    })
}

/// `args_json`: `{"container_id": "...", "item_id": "..."}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_add_member(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { container_id: String, item_id: String }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| match o.portfolio_mut().add_member(&args.container_id, &args.item_id) {
        Ok(())  => FfiResponse::ok(serde_json::json!({"ok": true})).into_json(),
        Err(e)  => FfiResponse::err(e).into_json(),
    })
}

/// `args_json`: `{"container_id": "...", "item_id": "..."}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_remove_member(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { container_id: String, item_id: String }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| match o.portfolio_mut().remove_member(&args.container_id, &args.item_id) {
        Ok(())  => FfiResponse::ok(serde_json::json!({"ok": true})).into_json(),
        Err(e)  => FfiResponse::err(e).into_json(),
    })
}

/// `args_json`: `{"id": "comp-000001"}` — returns JSON array of reachable child IDs.
#[no_mangle]
pub extern "C" fn kogi_portfolio_subtree(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { id: String }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| {
        let ids = o.portfolio_ref().subtree(&args.id);
        FfiResponse::ok(serde_json::json!({"root": args.id, "subtree": ids})).into_json()
    })
}

/// `args_json`: `{"id": "comp-000001"}` — returns transitive dependency IDs.
#[no_mangle]
pub extern "C" fn kogi_portfolio_transitive_dependencies(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { id: String }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| {
        let ids = o.portfolio_ref().transitive_dependencies(&args.id);
        FfiResponse::ok(serde_json::json!({"root": args.id, "dependencies": ids})).into_json()
    })
}

/// Returns topological ordering of the dependency graph.
#[no_mangle]
pub extern "C" fn kogi_portfolio_dependency_order() -> *mut c_char {
    with_office(|o| match o.portfolio_ref().dependency_order() {
        Ok(order) => FfiResponse::ok(serde_json::json!({"order": order})).into_json(),
        Err(e)    => FfiResponse::err(e).into_json(),
    })
}

// ── Portfolio: snapshots & checkpoints ────────────────────────────────────────

/// Returns the current in-memory snapshot without storing it.
#[no_mangle]
pub extern "C" fn kogi_portfolio_snapshot() -> *mut c_char {
    with_office(|o| match serde_json::to_value(o.portfolio_ref().snapshot()) {
        Ok(v)  => FfiResponse::ok(v).into_json(),
        Err(e) => FfiResponse::err(e.to_string()).into_json(),
    })
}

/// `args_json`: `{"label": "before-release"}` (label is optional)
#[no_mangle]
pub extern "C" fn kogi_portfolio_save_snapshot(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { label: Option<String> }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| match serde_json::to_value(o.portfolio_mut().save_snapshot(args.label)) {
        Ok(v)  => FfiResponse::ok(v).into_json(),
        Err(e) => FfiResponse::err(e.to_string()).into_json(),
    })
}

/// `args_json`: `{"snapshot_id": "snap-000001"}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_restore_snapshot(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { snapshot_id: String }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| match o.portfolio_mut().restore_snapshot(&args.snapshot_id) {
        Ok(())  => FfiResponse::ok(serde_json::json!({"ok": true, "snapshot_id": args.snapshot_id})).into_json(),
        Err(e)  => FfiResponse::err(e).into_json(),
    })
}

/// Returns all stored snapshots.
#[no_mangle]
pub extern "C" fn kogi_portfolio_list_snapshots() -> *mut c_char {
    with_office(|o| {
        let snaps: Vec<_> = o.portfolio_ref().snapshots.values().collect();
        match serde_json::to_value(snaps) {
            Ok(v)  => FfiResponse::ok(v).into_json(),
            Err(e) => FfiResponse::err(e.to_string()).into_json(),
        }
    })
}

/// `args_json`: `{"label": "v1.0", "note": "optional note"}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_save_checkpoint(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { label: String, note: Option<String> }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| match serde_json::to_value(o.portfolio_mut().save_checkpoint(args.label, args.note)) {
        Ok(v)  => FfiResponse::ok(v).into_json(),
        Err(e) => FfiResponse::err(e.to_string()).into_json(),
    })
}

/// `args_json`: `{"checkpoint_id": "ckpt-0001"}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_restore_checkpoint(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { checkpoint_id: String }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| match o.portfolio_mut().restore_checkpoint(&args.checkpoint_id) {
        Ok(())  => FfiResponse::ok(serde_json::json!({"ok": true, "checkpoint_id": args.checkpoint_id})).into_json(),
        Err(e)  => FfiResponse::err(e).into_json(),
    })
}

/// Returns all stored checkpoints.
#[no_mangle]
pub extern "C" fn kogi_portfolio_list_checkpoints() -> *mut c_char {
    with_office(|o| match serde_json::to_value(&o.portfolio_ref().checkpoints) {
        Ok(v)  => FfiResponse::ok(v).into_json(),
        Err(e) => FfiResponse::err(e.to_string()).into_json(),
    })
}

// ── Portfolio: query ──────────────────────────────────────────────────────────

/// `args_json`: `{"pql": "type=Project status=active"}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_query_pql(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { pql: String }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| {
        let results = o.portfolio_ref().query_pql(&args.pql);
        match serde_json::to_value(results) {
            Ok(v)  => FfiResponse::ok(serde_json::json!({"query": args.pql, "results": v})).into_json(),
            Err(e) => FfiResponse::err(e.to_string()).into_json(),
        }
    })
}

/// Returns the portfolio metadata summary map.
#[no_mangle]
pub extern "C" fn kogi_portfolio_metadata() -> *mut c_char {
    with_office(|o| match serde_json::to_value(o.portfolio_ref().portfolio_metadata()) {
        Ok(v)  => FfiResponse::ok(v).into_json(),
        Err(e) => FfiResponse::err(e.to_string()).into_json(),
    })
}

/// Returns the full event log.
#[no_mangle]
pub extern "C" fn kogi_portfolio_event_log() -> *mut c_char {
    with_office(|o| match serde_json::to_value(o.portfolio_ref().event_log.all()) {
        Ok(v)  => FfiResponse::ok(serde_json::json!({"events": v})).into_json(),
        Err(e) => FfiResponse::err(e.to_string()).into_json(),
    })
}

// ── Portfolio: governance ─────────────────────────────────────────────────────

/// `args_json`: `{"component_id": "...", "policy_id": "pol-001"}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_attach_policy(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { component_id: String, policy_id: String }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| match o.portfolio_mut().attach_policy(&args.component_id, &args.policy_id) {
        Ok(())  => FfiResponse::ok(serde_json::json!({"ok": true})).into_json(),
        Err(e)  => FfiResponse::err(e).into_json(),
    })
}

/// `args_json`: `{"component_id": "...", "policy_id": "pol-001"}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_detach_policy(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { component_id: String, policy_id: String }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| match o.portfolio_mut().detach_policy(&args.component_id, &args.policy_id) {
        Ok(())  => FfiResponse::ok(serde_json::json!({"ok": true})).into_json(),
        Err(e)  => FfiResponse::err(e).into_json(),
    })
}

/// `args_json`: `{"component_id": "...", "reason": "Budget threshold exceeded"}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_request_approval(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { component_id: String, reason: String }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| {
        let req = o.portfolio_mut().request_approval(&args.component_id, args.reason);
        match serde_json::to_value(req) {
            Ok(v)  => FfiResponse::ok(v).into_json(),
            Err(e) => FfiResponse::err(e.to_string()).into_json(),
        }
    })
}

/// `args_json`: `{"request_id":"req-000001","approved":true,"resolver":"alice","notes":null}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_resolve_approval(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)]
    struct Args {
        request_id: String,
        approved: bool,
        resolver: String,
        notes: Option<String>,
    }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| match o.portfolio_mut().resolve_approval(&args.request_id, args.approved, args.resolver, args.notes) {
        Ok(req) => match serde_json::to_value(req) {
            Ok(v)  => FfiResponse::ok(v).into_json(),
            Err(e) => FfiResponse::err(e.to_string()).into_json(),
        },
        Err(e) => FfiResponse::err(e).into_json(),
    })
}

/// `args_json`: `{"component_id":"...","kind":"Budget","total":100000.0,"denomination":"USD","period":"2026-Q1"}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_allocate_resource(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)]
    struct Args {
        component_id: String,
        kind: String,
        total: f64,
        denomination: String,
        period: Option<String>,
    }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    let kind = parse_resource_kind(&args.kind);
    with_office(|o| match o.portfolio_mut().allocate_resource(&args.component_id, kind, args.total, args.denomination, args.period) {
        Ok(alloc) => match serde_json::to_value(alloc) {
            Ok(v)  => FfiResponse::ok(v).into_json(),
            Err(e) => FfiResponse::err(e.to_string()).into_json(),
        },
        Err(e) => FfiResponse::err(e).into_json(),
    })
}

/// `args_json`: `{"component_id": "...", "amount": 5000.0}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_record_consumption(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { component_id: String, amount: f64 }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| match o.portfolio_mut().record_consumption(&args.component_id, args.amount) {
        Ok(remaining) => FfiResponse::ok(serde_json::json!({"remaining": remaining})).into_json(),
        Err(e)        => FfiResponse::err(e).into_json(),
    })
}

/// Returns all over-budget / over-capacity allocations.
#[no_mangle]
pub extern "C" fn kogi_portfolio_overrun_allocations() -> *mut c_char {
    with_office(|o| match serde_json::to_value(o.portfolio_ref().overrun_allocations()) {
        Ok(v)  => FfiResponse::ok(serde_json::json!({"overruns": v})).into_json(),
        Err(e) => FfiResponse::err(e.to_string()).into_json(),
    })
}

/// `args_json`: `{"component_id": "..."}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_get_resource_allocation(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { component_id: String }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| match o.portfolio_ref().get_resource_allocation(&args.component_id) {
        Some(alloc) => match serde_json::to_value(alloc) {
            Ok(v)  => FfiResponse::ok(v).into_json(),
            Err(e) => FfiResponse::err(e.to_string()).into_json(),
        },
        None => FfiResponse::err(format!("no allocation for '{}'", args.component_id)).into_json(),
    })
}

// ── Portfolio: computational models ──────────────────────────────────────────

/// `args_json`: `{"portfolio_id": "comp-000001"}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_compute_health(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { portfolio_id: String }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| match o.portfolio_ref().compute_portfolio_health(&args.portfolio_id) {
        Some(r) => match serde_json::to_value(r) {
            Ok(v)  => FfiResponse::ok(v).into_json(),
            Err(e) => FfiResponse::err(e.to_string()).into_json(),
        },
        None => FfiResponse::err(format!("portfolio '{}' not found", args.portfolio_id)).into_json(),
    })
}

/// `args_json`: full ProjectMetricsInput JSON.
#[no_mangle]
pub extern "C" fn kogi_portfolio_compute_project_metrics(args_json: *const c_char) -> *mut c_char {
    let input: ProjectMetricsInput = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    use crate::portfolio_system::ProjectMetricsModel;
    match serde_json::to_value(ProjectMetricsModel::compute(&input)) {
        Ok(v)  => FfiResponse::ok(v).into_json(),
        Err(e) => FfiResponse::err(e.to_string()).into_json(),
    }
}

/// `args_json`: `{"subportfolio_id": "comp-000002"}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_compute_subportfolio_rollup(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { subportfolio_id: String }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| match o.portfolio_ref().compute_subportfolio_rollup(&args.subportfolio_id) {
        Some(r) => match serde_json::to_value(r) {
            Ok(v)  => FfiResponse::ok(v).into_json(),
            Err(e) => FfiResponse::err(e.to_string()).into_json(),
        },
        None => FfiResponse::err(format!("subportfolio '{}' not found", args.subportfolio_id)).into_json(),
    })
}

/// `args_json`: `{"resource_id": "comp-000003"}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_compute_resource_utilisation(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { resource_id: String }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| match o.portfolio_ref().compute_resource_utilisation(&args.resource_id) {
        Some(r) => match serde_json::to_value(r) {
            Ok(v)  => FfiResponse::ok(v).into_json(),
            Err(e) => FfiResponse::err(e.to_string()).into_json(),
        },
        None => FfiResponse::err(format!("resource '{}' not found or wrong type", args.resource_id)).into_json(),
    })
}

/// `args_json`: full AssetValueInput JSON.
#[no_mangle]
pub extern "C" fn kogi_portfolio_compute_asset_value(args_json: *const c_char) -> *mut c_char {
    let input: AssetValueInput = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    use crate::portfolio_system::AssetValueModel;
    match serde_json::to_value(AssetValueModel::compute(&input)) {
        Ok(v)  => FfiResponse::ok(v).into_json(),
        Err(e) => FfiResponse::err(e.to_string()).into_json(),
    }
}

/// `args_json`: `{"artifact_id":"...","required_fields":["description"],"max_fresh_days":30}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_compute_artifact_maturity(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)]
    struct Args { artifact_id: String, required_fields: Vec<String>, max_fresh_days: u64 }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    let fields: Vec<&str> = args.required_fields.iter().map(String::as_str).collect();
    with_office(|o| match o.portfolio_ref().compute_artifact_maturity(&args.artifact_id, &fields, args.max_fresh_days) {
        Some(r) => match serde_json::to_value(r) {
            Ok(v)  => FfiResponse::ok(v).into_json(),
            Err(e) => FfiResponse::err(e.to_string()).into_json(),
        },
        None => FfiResponse::err(format!("artifact '{}' not found or wrong type", args.artifact_id)).into_json(),
    })
}

/// `args_json`: `{"binder_id": "...", "expected_ids": ["comp-1", "comp-2"]}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_compute_binder_coverage(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)]
    struct Args { binder_id: String, expected_ids: Vec<String> }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    let expected: HashSet<String> = args.expected_ids.into_iter().collect();
    with_office(|o| match o.portfolio_ref().compute_binder_coverage(&args.binder_id, expected) {
        Some(r) => match serde_json::to_value(r) {
            Ok(v)  => FfiResponse::ok(v).into_json(),
            Err(e) => FfiResponse::err(e.to_string()).into_json(),
        },
        None => FfiResponse::err(format!("binder '{}' not found or wrong type", args.binder_id)).into_json(),
    })
}

/// `args_json`: full BookConsistencyInput JSON.
#[no_mangle]
pub extern "C" fn kogi_portfolio_compute_book_consistency(args_json: *const c_char) -> *mut c_char {
    let input: BookConsistencyInput = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    use crate::portfolio_system::BookConsistencyModel;
    match serde_json::to_value(BookConsistencyModel::compute(&input)) {
        Ok(v)  => FfiResponse::ok(v).into_json(),
        Err(e) => FfiResponse::err(e.to_string()).into_json(),
    }
}

/// `args_json`: `{"folder_id": "...", "depth_threshold": 4}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_compute_folder_organisation(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { folder_id: String, depth_threshold: usize }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| match o.portfolio_ref().compute_folder_organisation(&args.folder_id, args.depth_threshold) {
        Some(r) => match serde_json::to_value(r) {
            Ok(v)  => FfiResponse::ok(v).into_json(),
            Err(e) => FfiResponse::err(e.to_string()).into_json(),
        },
        None => FfiResponse::err(format!("folder '{}' not found or wrong type", args.folder_id)).into_json(),
    })
}

/// `args_json`: `{"record_id": "comp-000004"}`
#[no_mangle]
pub extern "C" fn kogi_portfolio_compute_record_integrity(args_json: *const c_char) -> *mut c_char {
    #[derive(Deserialize)] struct Args { record_id: String }
    let args: Args = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| match o.portfolio_ref().compute_record_integrity(&args.record_id) {
        Some(r) => match serde_json::to_value(r) {
            Ok(v)  => FfiResponse::ok(v).into_json(),
            Err(e) => FfiResponse::err(e.to_string()).into_json(),
        },
        None => FfiResponse::err(format!("record '{}' not found or wrong type", args.record_id)).into_json(),
    })
}

// ── Office: timeline / workspace / assistant mutations ────────────────────────

/// `args_json`: NewTimelineEvent JSON.
#[no_mangle]
pub extern "C" fn kogi_office_create_timeline_event(args_json: *const c_char) -> *mut c_char {
    let req: NewTimelineEvent = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| match serde_json::to_value(o.create_timeline_event(req)) {
        Ok(v)  => FfiResponse::ok(v).into_json(),
        Err(e) => FfiResponse::err(e.to_string()).into_json(),
    })
}

/// `args_json`: NewWorkspaceStory JSON.
#[no_mangle]
pub extern "C" fn kogi_office_create_workspace_story(args_json: *const c_char) -> *mut c_char {
    let req: NewWorkspaceStory = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| match serde_json::to_value(o.create_workspace_story(req)) {
        Ok(v)  => FfiResponse::ok(v).into_json(),
        Err(e) => FfiResponse::err(e.to_string()).into_json(),
    })
}

/// `args_json`: NewAssistantSubscription JSON.
#[no_mangle]
pub extern "C" fn kogi_office_create_assistant_subscription(args_json: *const c_char) -> *mut c_char {
    let req: NewAssistantSubscription = match parse_arg(args_json) { Ok(a) => a, Err(e) => return e };
    with_office(|o| match serde_json::to_value(o.create_assistant_subscription(req)) {
        Ok(v)  => FfiResponse::ok(v).into_json(),
        Err(e) => FfiResponse::err(e.to_string()).into_json(),
    })
}

// ── Parsing helpers ───────────────────────────────────────────────────────────

fn parse_component_type_str(s: &str) -> PortfolioComponentType {
    match s.to_lowercase().as_str() {
        "portfolio"                    => PortfolioComponentType::Portfolio,
        "project"                      => PortfolioComponentType::Project,
        "program"                      => PortfolioComponentType::Program,
        "resource"                     => PortfolioComponentType::Resource,
        "asset"                        => PortfolioComponentType::Asset,
        "artifact"                     => PortfolioComponentType::Artifact,
        "subportfolio" | "sub_portfolio" => PortfolioComponentType::SubPortfolio,
        "binder"                       => PortfolioComponentType::Binder,
        "book"                         => PortfolioComponentType::Book,
        "folder"                       => PortfolioComponentType::Folder,
        "record"                       => PortfolioComponentType::Record,
        _                              => PortfolioComponentType::Project,
    }
}

fn parse_book_type_str(s: &str) -> BookType {
    match s.to_lowercase().as_str() {
        "playbook"     => BookType::Playbook,
        "contactbook"  => BookType::Contactbook,
        "schedulebook" => BookType::Schedulebook,
        "itembook"     => BookType::Itembook,
        _              => BookType::Notebook,
    }
}

fn parse_resource_kind(s: &str) -> ResourceKind {
    match s.to_lowercase().as_str() {
        "budget"       => ResourceKind::Budget,
        "personhours"  => ResourceKind::PersonHours,
        "storypoints"  => ResourceKind::StoryPoints,
        "computeunits" => ResourceKind::ComputeUnits,
        "storagegib"   => ResourceKind::StorageGiB,
        other          => ResourceKind::Custom(other.to_string()),
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn office_module_coordinates_subsystems() {
        let mut module = OfficeModule::mvp();
        assert!(module.ack_dashboard_notification("notif-001").result.ok);
        assert!(module.create_portfolio_item(NewPortfolioItem {
            item_type: "project".to_string(),
            name: "OS Adapter Port".to_string(),
            status: "active".to_string(),
        }).result.ok);
        assert!(module.create_timeline_event(NewTimelineEvent {
            calendar_id: "cal-work".to_string(),
            title: "Design review".to_string(),
            kind: "review".to_string(),
            scheduled_for: "2026-03-12T18:00:00Z".to_string(),
        }).result.ok);
        assert!(module.create_workspace_story(NewWorkspaceStory {
            title: "As a worker, I can execute office flows".to_string(),
            points: 5,
        }).result.ok);
        assert!(module.create_assistant_subscription(NewAssistantSubscription {
            topic: "office.dashboard.alerts".to_string(),
        }).result.ok);
    }

    #[test]
    fn ffi_create_and_get_component() {
        let args = r#"{"component_type":"Project","name":"Test FFI Project"}"#;
        let cargs = std::ffi::CString::new(args).unwrap();
        let raw = kogi_portfolio_create_component(cargs.as_ptr());
        assert!(!raw.is_null());
        let resp: serde_json::Value = serde_json::from_str(
            unsafe { std::ffi::CStr::from_ptr(raw) }.to_str().unwrap()
        ).unwrap();
        assert_eq!(resp["ok"], true);
        unsafe { kogi_free_string(raw) };
    }
}
