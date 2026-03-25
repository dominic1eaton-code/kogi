//! PortfolioPlugin trait and lifecycle hooks.
use crate::component::Component;
use crate::crdt::CrdtOperation;
use crate::events::{PortfolioEvent, Snapshot};
use crate::graph::PortfolioEdge;

pub trait PortfolioPlugin: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn on_component_created(&self, _component: &Component) {}
    fn on_component_updated(&self, _component: &Component) {}
    fn on_component_removed(&self, _component_id: uuid::Uuid) {}
    fn on_edge_added(&self, _edge: &PortfolioEdge) {}
    fn on_edge_removed(&self, _edge_id: uuid::Uuid) {}
    fn on_event(&self, _event: &PortfolioEvent) {}
    fn on_snapshot_saved(&self, _snapshot: &Snapshot) {}
    fn on_crdt_merge(&self, _ops: &[CrdtOperation]) {}
}

pub struct NoOpPlugin { pub id: String }
impl PortfolioPlugin for NoOpPlugin {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "no-op" }
}

pub struct LoggingPlugin;
impl PortfolioPlugin for LoggingPlugin {
    fn id(&self) -> &str { "logging" }
    fn name(&self) -> &str { "Logging Plugin" }
    fn on_component_created(&self, c: &Component) {
        eprintln!("[Portfolio] ComponentCreated: {} ({})", c.id(), c.name());
    }
    fn on_event(&self, e: &PortfolioEvent) {
        eprintln!("[Portfolio] Event: {:?} at {}", e.event_kind, e.timestamp);
    }
}
