//! PortfolioPlugin trait and lifecycle hooks.

use crate::component::Component;
use crate::crdt::CrdtOperation;
use crate::events::{PortfolioEvent, Snapshot};
use crate::graph::GraphEdge;

/// Plugins are registered as `Arc<dyn PortfolioPlugin>` and receive lifecycle
/// hooks for every significant system event. All hooks have default no-op
/// implementations, so a plugin only needs to implement what it uses.
pub trait PortfolioPlugin: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;

    /// Fired when `create_item()` or `create_container()` succeeds.
    fn on_component_created(&self, _component: &Component) {}

    /// Fired when `update_info()`, `apply_action()`, or `bump_version()` mutates a component.
    fn on_component_updated(&self, _component: &Component) {}

    /// Fired when `delete()` with `hard_delete=true`.
    fn on_component_removed(&self, _component_id: uuid::Uuid) {}

    /// Fired when `attach_child()`, `add_dependency()`, or `add_link()` adds an edge.
    fn on_edge_added(&self, _edge: &GraphEdge) {}

    /// Fired when `detach_child()` or `remove_dependency()` removes an edge.
    fn on_edge_removed(&self, _edge_id: uuid::Uuid) {}

    /// Fired for every `emit()` call — all PortfolioEventKinds.
    fn on_event(&self, _event: &PortfolioEvent) {}

    /// Fired when `save_snapshot()` or `save_checkpoint()` is called.
    fn on_snapshot_saved(&self, _snapshot: &Snapshot) {}

    /// Fired after `apply_crdt_log()` completes a remote CRDT merge.
    fn on_crdt_merge(&self, _ops: &[CrdtOperation]) {}
}

/// A no-op plugin for testing.
pub struct NoOpPlugin { pub id: String }

impl PortfolioPlugin for NoOpPlugin {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "no-op" }
}

/// A logging plugin that prints events to stdout (useful for dev/debug).
pub struct LoggingPlugin;

impl PortfolioPlugin for LoggingPlugin {
    fn id(&self) -> &str { "logging" }
    fn name(&self) -> &str { "Logging Plugin" }

    fn on_component_created(&self, component: &Component) {
        eprintln!("[PortfolioPlugin] ComponentCreated: {} ({})", component.id(), component.name());
    }

    fn on_event(&self, event: &PortfolioEvent) {
        eprintln!("[PortfolioPlugin] Event: {:?} at {}", event.event_kind, event.timestamp);
    }
}
