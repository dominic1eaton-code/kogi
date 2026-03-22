//! # Hypergrid — N-Dimensional Distributed Spreadsheet System (NDSS)
//!
//! Hypergrid is a platform-agnostic, open-architecture, N-dimensional distributed
//! spreadsheet system. It provides any application platform with a universal,
//! infinitely extensible data substrate that models all entities, relationships,
//! computations, and intelligence as a single, coherent, living grid.
//!
//! ## Core Modules
//!
//! | Module     | Code      | Description                                         |
//! |------------|-----------|-----------------------------------------------------|
//! | [`core`]   | HG-CORE   | Grid, Hypercube, DimensionAxis, HyperCell           |
//! | [`dim`]    | HG-DIM    | Dimension axis registry, types, encoding            |
//! | [`cell`]   | HG-CELL   | N-attribute cell model, attribute key registry      |
//! | [`crdt`]   | HG-CRDT   | N-dim CRDT: LWW, OR-Set, Counter, Lattice           |
//! | [`graph`]  | HG-GRAPH  | Hypergraph: typed edges, shadow cells, link forests |
//! | [`view`]   | HG-VIEW   | HypercubeView: slices, folds, render modes          |
//! | [`space`]  | HG-SPACE  | Spaces, Workspaces, Namespaces                      |
//! | [`query`]  | HG-QL     | HyperQL N-dimensional query engine                  |
//! | [`plugin`] | HG-PLUGIN | Plugin / extension trait                            |
//! | [`error`]  | —         | Unified error type                                  |

pub mod core;
pub mod dim;
pub mod cell;
pub mod crdt;
pub mod graph;
pub mod view;
pub mod space;
pub mod query;
pub mod plugin;
pub mod error;

// Re-export commonly used types at the crate root
pub use crate::core::{Grid, GridId, Hypercube, CubeId};
pub use crate::cell::{HyperCell, DimCoordinate, DimKey, AttributeMap, AttributeKey, TypedAttrValue};
pub use crate::dim::{DimensionAxis, AxisId, AxisType, KeyType, KeyCardinality, AxisOrdering};
pub use crate::crdt::{VectorClock, CrdtLog, CrdtOperation, CrdtSemantics};
pub use crate::graph::{Hypergraph, HypergraphNode, HypergraphEdge, EdgeType, NodeType};
pub use crate::space::{Space, SpaceId, SpaceType, Workspace, WorkspaceId};
pub use crate::error::{HypergridError, HypergridResult};
