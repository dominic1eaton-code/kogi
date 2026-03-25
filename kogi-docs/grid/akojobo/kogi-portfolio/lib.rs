//! # Kogi Platform — Portfolio System v2.1
//!
//! The Portfolio System is the central domain engine of the Kogi Platform —
//! the single source of truth for every entity belonging to an independent
//! worker's portfolio.
//!
//! ## Design Principle
//! > "Everything in the Kogi ecosystem is a Portfolio Item. The Portfolio System
//! > is therefore the universal abstraction layer that every platform application
//! > builds upon."
//!
//! ## Module Structure
//!
//! | Module          | Contents                                                  |
//! |-----------------|-----------------------------------------------------------|
//! | [`types`]       | All shared enums and primitive newtypes                   |
//! | [`metadata`]    | ComponentMetadata, VectorClock, versioning                |
//! | [`component`]   | Component (Item | Container), ComponentData, payloads     |
//! | [`graph`]       | GraphEdge, structural primitives (Group/Collection/List)  |
//! | [`crdt`]        | VectorClock, CrdtLog, CrdtOperation, federation           |
//! | [`governance`]  | PolicyEngine trait, ApprovalWorkflow, ResourceAllocation  |
//! | [`models`]      | 11 built-in analytical/computational models               |
//! | [`events`]      | EventLog, PortfolioEvent, snapshot/time-travel            |
//! | [`search`]      | SearchQuery, PortfolioQuery (PQL), SearchResult            |
//! | [`plugin`]      | PortfolioPlugin trait and lifecycle hooks                  |
//! | [`federation`]  | PortfolioFederation, FederationPeer, CRDT sync             |
//! | [`itembook`]    | ItemBook — per-item rich dossier                          |
//! | [`collaboration`]| Shared portfolios, ContributionRecord, crowdresourcing   |
//! | [`benefits`]    | Portable benefits as portfolio items                      |
//! | [`system`]      | PortfolioSystem — the root orchestrator                   |
//! | [`error`]       | PortfolioError, PortfolioResult                           |

pub mod types;
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
pub mod system;
pub mod error;

// ── Crate-level re-exports ────────────────────────────────────────────────────
pub use types::*;
pub use error::{PortfolioError, PortfolioResult};
pub use system::PortfolioSystem;
pub use component::{Component, ComponentData, ComponentMetadata, ItemPayload, ContainerPayload};
pub use graph::{GraphEdge, GraphEdgeType};
pub use crdt::{VectorClock, CrdtLog, CrdtOperation};
pub use governance::{PolicyEngine, PolicyDecision, PolicyContext};
pub use models::{PortfolioHealth, ProjectMetrics};
pub use events::{EventLog, PortfolioEvent, PortfolioEventKind};
pub use plugin::PortfolioPlugin;
