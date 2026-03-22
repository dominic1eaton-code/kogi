//! # Kogi Platform — Portfolio System v2.1
//!
//! Built on top of **Hypergrid** (N-Dimensional Distributed Spreadsheet System).
//!
//! ## Architecture
//!
//! ```text
//!  kogi-portfolio (domain layer)
//!    ├── PortfolioSystem         ← root orchestrator
//!    │    ├── hypergrid::Grid    ← cell storage + CRDT (components Hypercube)
//!    │    ├── PortfolioGraph     ← wraps hypergrid::Hypergraph
//!    │    ├── EventLog           ← wraps hypergrid::core::EventLog
//!    │    └── PortfolioCrdtLog   ← wraps hypergrid::crdt::CrdtLog
//!    ├── ComponentStore          ← Component ↔ HyperCell serialisation
//!    ├── GovernanceEngine        ← PolicyEngine, ApprovalWorkflow, ResourceAllocation
//!    ├── PortfolioPlugin trait   ← lifecycle hooks
//!    └── 11 Analytical Models    ← pure compute, no storage
//! ```
//!
//! Every Component is stored as a HyperRow in the `kogi.portfolio.components`
//! Hypercube (D1=ComponentId, D2=field_name). Reads reconstruct the Component
//! from its HyperCells; writes decompose it back. All mutations flow through
//! the hypergrid EventLog for a single, unified, time-travelable audit trail.

pub mod types;
pub mod error;
pub mod metadata;
pub mod component;
pub mod graph;
pub mod crdt;
pub mod governance;
pub mod models;
pub mod events;
pub mod search;
pub mod plugin;
pub mod federation;
pub mod itembook;
pub mod collaboration;
pub mod benefits;
pub mod store;
pub mod system;

// ── Crate-level re-exports ────────────────────────────────────────────────────
pub use types::*;
pub use error::{PortfolioError, PortfolioResult};
pub use system::{PortfolioSystem, SystemStats};
pub use component::{Component, ComponentData, ComponentPayload, ItemPayload, ContainerPayload};
pub use metadata::{BumpKind, ComponentMetadata};
pub use graph::{PortfolioEdge, PortfolioEdgeType, PortfolioGraph};
pub use crdt::{PortfolioCrdtLog, CrdtOperation, VectorClock};
pub use governance::{PolicyEngine, PolicyDecision, PolicyContext};
pub use models::{PortfolioHealth, PortfolioHealthResult, ProjectMetrics, ProjectMetricsResult};
pub use events::{EventLog, PortfolioEvent, PortfolioEventKind};
pub use plugin::PortfolioPlugin;
pub use store::{ComponentStore, COMPONENTS_CUBE_NAME};

// Re-export core hypergrid types that portfolio consumers frequently need.
pub use hypergrid::cell::{DimCoordinate, DimKey, TypedAttrValue, AttributeMap};
pub use hypergrid::core::Grid;
pub use hypergrid::crdt::CrdtLog;
