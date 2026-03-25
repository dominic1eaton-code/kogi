//! HG-CORE — Core substrate: Grid, Hypercube, Universal Cell Store, EventLog.

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::cell::{
    ActorId, AttributeKey, AttributeKeyRegistry, AxisId, DimCoordinate, DimKey,
    GridId, CubeId, HyperCell, TypedAttrValue,
};
use crate::crdt::{CrdtLog, NodeId, VectorClock};
use crate::dim::{DimensionAxis, MAX_DIMENSIONS};
use crate::error::{HypergridError, HypergridResult};
use crate::plugin::HypercubePlugin;

// ─── EventLog ────────────────────────────────────────────────────────────────

/// Kind of mutation event recorded in the EventLog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventKind {
    CellCreated,
    CellUpdated { attribute: AttributeKey },
    CellDeleted,
    AxisAdded { axis_id: AxisId },
    AttributeKeyRegistered { key: AttributeKey },
    HypercubeCreated,
    HypercubeSchemaChanged,
    ViewCreated { view_id: Uuid },
    SpaceCreated,
    EdgeAdded { edge_id: Uuid },
    EdgeRemoved { edge_id: Uuid },
    Custom(String),
}

/// A single entry in the immutable append-only EventLog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEntry {
    pub event_id: Uuid,
    pub event_kind: EventKind,
    pub grid_id: GridId,
    pub cube_id: Option<CubeId>,
    pub coord: Option<DimCoordinate>,
    pub actor: ActorId,
    pub vector_clock: VectorClock,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, JsonValue>,
}

/// Append-only, immutable log of every mutation.
/// Enables full audit trail and AS_OF time-travel queries.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventLog {
    entries: Vec<EventEntry>,
}

impl EventLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(&mut self, kind: EventKind, grid_id: GridId, actor: &str, vc: VectorClock) -> Uuid {
        let event_id = Uuid::new_v4();
        self.entries.push(EventEntry {
            event_id,
            event_kind: kind,
            grid_id,
            cube_id: None,
            coord: None,
            actor: actor.to_owned(),
            vector_clock: vc,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        });
        event_id
    }

    pub fn append_full(&mut self, entry: EventEntry) {
        self.entries.push(entry);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Return all entries before a given timestamp (for AS_OF time-travel).
    pub fn as_of(&self, timestamp: DateTime<Utc>) -> Vec<&EventEntry> {
        self.entries.iter().filter(|e| e.timestamp <= timestamp).collect()
    }

    pub fn entries(&self) -> &[EventEntry] {
        &self.entries
    }
}

// ─── Universal Cell Store ─────────────────────────────────────────────────────

/// The Universal Cell Store (UCS): maps (cube_id, dim_keys_tuple) → HyperCell.
/// This is the primary in-memory cell storage for a Grid.
#[derive(Debug, Clone, Default)]
pub struct UniversalCellStore {
    cells: HashMap<(CubeId, DimCoordinate), HyperCell>,
}

impl UniversalCellStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, cube_id: CubeId, coord: &DimCoordinate) -> Option<&HyperCell> {
        self.cells.get(&(cube_id, coord.clone()))
    }

    pub fn get_mut(&mut self, cube_id: CubeId, coord: &DimCoordinate) -> Option<&mut HyperCell> {
        self.cells.get_mut(&(cube_id, coord.clone()))
    }

    pub fn insert(&mut self, cell: HyperCell) {
        self.cells.insert((cell.cube_id, cell.coord.clone()), cell);
    }

    pub fn remove(&mut self, cube_id: CubeId, coord: &DimCoordinate) -> Option<HyperCell> {
        self.cells.remove(&(cube_id, coord.clone()))
    }

    /// Return all cells in a given Hypercube.
    pub fn cells_in_cube(&self, cube_id: CubeId) -> Vec<&HyperCell> {
        self.cells
            .iter()
            .filter(|((cid, _), _)| *cid == cube_id)
            .map(|(_, c)| c)
            .collect()
    }

    /// Return all cells in a Hypercube that match a simple D₁ point slice.
    pub fn cells_for_d1(&self, cube_id: CubeId, d1_key: &DimKey) -> Vec<&HyperCell> {
        self.cells_in_cube(cube_id)
            .into_iter()
            .filter(|c| c.coord.d1() == Some(d1_key))
            .collect()
    }

    pub fn total_cells(&self) -> usize {
        self.cells.len()
    }
}

// ─── Hypercube ────────────────────────────────────────────────────────────────

/// An N-dimensional grid H = (D₁, D₂, ..., Dₙ).
/// The generalization of a spreadsheet sheet/tab.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypercube {
    pub cube_id: CubeId,
    pub grid_id: GridId,
    pub name: String,
    pub description: String,

    /// The ordered tuple of dimension axes: [D₁, D₂, ..., Dₙ]
    pub axes: Vec<DimensionAxis>,

    /// The N-attribute key registry for this Hypercube.
    pub attr_registry: AttributeKeyRegistry,

    /// Default visibility for cells in this cube.
    pub default_visibility: crate::cell::CellVisibility,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: ActorId,

    pub metadata: HashMap<String, JsonValue>,
}

impl Hypercube {
    /// Create a new 2D Hypercube (N=2: EntityAxis × PropertyAxis).
    pub fn new_2d(grid_id: GridId, name: impl Into<String>, actor: &str) -> Self {
        let now = Utc::now();
        Self {
            cube_id: Uuid::new_v4(),
            grid_id,
            name: name.into(),
            description: String::new(),
            axes: vec![
                DimensionAxis::entity_axis(),
                DimensionAxis::property_axis(),
            ],
            attr_registry: AttributeKeyRegistry::new(),
            default_visibility: crate::cell::CellVisibility::Tenant,
            created_at: now,
            updated_at: now,
            created_by: actor.to_owned(),
            metadata: HashMap::new(),
        }
    }

    /// The dimensionality N of this Hypercube.
    pub fn n(&self) -> u8 {
        self.axes.len() as u8
    }

    /// Add a new dimension axis (D₃–N custom axis).
    pub fn add_axis(&mut self, axis: DimensionAxis) -> HypergridResult<()> {
        if self.n() >= MAX_DIMENSIONS {
            return Err(HypergridError::MaxDimensionalityExceeded);
        }
        if axis.dim_index != self.n() + 1 {
            return Err(HypergridError::InvalidOperation(format!(
                "Axis dim_index {} does not match next expected index {}",
                axis.dim_index, self.n() + 1
            )));
        }
        self.axes.push(axis);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Get the axis at dimension index i (1-indexed).
    pub fn axis(&self, dim_index: u8) -> Option<&DimensionAxis> {
        self.axes.get((dim_index - 1) as usize)
    }

    /// Validate a coordinate against this Hypercube's dimensionality and key sets.
    pub fn validate_coordinate(&self, coord: &DimCoordinate) -> HypergridResult<()> {
        coord.validate_n(self.n())?;
        for (axis, key) in self.axes.iter().zip(coord.keys.iter()) {
            axis.validate_key(key)?;
        }
        Ok(())
    }
}

// ─── Grid ─────────────────────────────────────────────────────────────────────

/// The root container for all Hypercubes, dimensions, tenants, namespaces,
/// spaces, and graph structures in a Hypergrid deployment.
pub struct Grid {
    pub grid_id: GridId,
    pub name: String,
    pub description: String,

    /// All Hypercubes registered in this Grid.
    cubes: HashMap<CubeId, Hypercube>,

    /// The Universal Cell Store: all cells across all Hypercubes.
    ucs: UniversalCellStore,

    /// Distributed CRDT log for this Grid.
    pub crdt_log: CrdtLog,

    /// Append-only event log.
    pub event_log: EventLog,

    /// Grid-level VectorClock.
    pub vector_clock: VectorClock,

    /// This node's ID in the federation network.
    pub node_id: NodeId,

    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, JsonValue>,

    /// Registered plugins.
    plugins: Vec<Box<dyn HypercubePlugin>>,
}

impl Grid {
    /// Create a new Grid with the given name and node ID.
    pub fn new(name: impl Into<String>, node_id: impl Into<NodeId>) -> Self {
        let now = Utc::now();
        Self {
            grid_id: Uuid::new_v4(),
            name: name.into(),
            description: String::new(),
            cubes: HashMap::new(),
            ucs: UniversalCellStore::new(),
            crdt_log: CrdtLog::new(),
            event_log: EventLog::new(),
            vector_clock: VectorClock::new(),
            node_id: node_id.into(),
            created_at: now,
            metadata: HashMap::new(),
            plugins: vec![],
        }
    }

    // ── Hypercube Management ───────────────────────────────────────────────

    /// Register a new Hypercube in this Grid.
    pub fn register_cube(&mut self, cube: Hypercube) -> HypergridResult<CubeId> {
        let cube_id = cube.cube_id;
        if self.cubes.contains_key(&cube_id) {
            return Err(HypergridError::AlreadyExists(format!("cube:{cube_id}")));
        }
        let vc = self.tick();
        self.event_log.append(EventKind::HypercubeCreated, self.grid_id, &self.node_id.clone(), vc);
        self.cubes.insert(cube_id, cube);
        Ok(cube_id)
    }

    /// Create and register a new 2D Hypercube.
    pub fn create_cube(&mut self, name: impl Into<String>) -> HypergridResult<CubeId> {
        let cube = Hypercube::new_2d(self.grid_id, name, &self.node_id.clone());
        self.register_cube(cube)
    }

    pub fn get_cube(&self, cube_id: CubeId) -> HypergridResult<&Hypercube> {
        self.cubes.get(&cube_id).ok_or_else(|| HypergridError::NotFound(format!("cube:{cube_id}")))
    }

    pub fn get_cube_mut(&mut self, cube_id: CubeId) -> HypergridResult<&mut Hypercube> {
        self.cubes.get_mut(&cube_id).ok_or_else(|| HypergridError::NotFound(format!("cube:{cube_id}")))
    }

    pub fn list_cubes(&self) -> Vec<&Hypercube> {
        self.cubes.values().collect()
    }

    // ── Cell Operations ────────────────────────────────────────────────────

    /// Write a value to a cell attribute, creating the cell if it doesn't exist.
    pub fn write_cell(
        &mut self,
        cube_id: CubeId,
        coord: DimCoordinate,
        attribute: impl Into<AttributeKey>,
        value: TypedAttrValue,
        actor: &str,
    ) -> HypergridResult<()> {
        let cube = self.get_cube(cube_id)?;
        cube.validate_coordinate(&coord)?;
        let attr_key = attribute.into();
        if !cube.attr_registry.contains(&attr_key) {
            return Err(HypergridError::UnregisteredAttributeKey(attr_key));
        }

        let vc = self.tick();
        let cell = self.ucs.get_mut(cube_id, &coord);
        let event_kind;
        match cell {
            Some(c) => {
                c.set_attr(attr_key.clone(), value, actor);
                c.vector_clock.merge(&vc);
                event_kind = EventKind::CellUpdated { attribute: attr_key };
            }
            None => {
                let mut new_cell = HyperCell::new(self.grid_id, cube_id, coord.clone(), actor);
                new_cell.set_attr(attr_key, value, actor);
                new_cell.vector_clock.merge(&vc);
                self.ucs.insert(new_cell);
                event_kind = EventKind::CellCreated;
            }
        }
        let vc2 = self.vector_clock.clone();
        self.event_log.append(event_kind, self.grid_id, actor, vc2);
        Ok(())
    }

    /// Read a cell attribute value.
    pub fn read_cell(
        &self,
        cube_id: CubeId,
        coord: &DimCoordinate,
        attribute: &str,
    ) -> HypergridResult<Option<&TypedAttrValue>> {
        self.get_cube(cube_id)?; // validate cube exists
        Ok(self.ucs.get(cube_id, coord).and_then(|c| c.get_attr(attribute)))
    }

    /// Get the raw HyperCell.
    pub fn get_cell(&self, cube_id: CubeId, coord: &DimCoordinate) -> Option<&HyperCell> {
        self.ucs.get(cube_id, coord)
    }

    /// Return all cells in a cube matching a D₁ key (all columns for one entity).
    pub fn get_row(&self, cube_id: CubeId, d1_key: &DimKey) -> Vec<&HyperCell> {
        self.ucs.cells_for_d1(cube_id, d1_key)
    }

    /// Return all cells in a cube (unfiltered scan).
    pub fn scan_cube(&self, cube_id: CubeId) -> Vec<&HyperCell> {
        self.ucs.cells_in_cube(cube_id)
    }

    // ── Dimension Management ───────────────────────────────────────────────

    /// Add a new custom axis (D₃–N) to an existing Hypercube.
    pub fn add_axis_to_cube(&mut self, cube_id: CubeId, axis: DimensionAxis) -> HypergridResult<AxisId> {
        let axis_id = axis.axis_id;
        let vc = self.tick();
        let cube = self.get_cube_mut(cube_id)?;
        cube.add_axis(axis)?;
        self.event_log.append(
            EventKind::AxisAdded { axis_id },
            self.grid_id,
            &self.node_id.clone(),
            vc,
        );
        Ok(axis_id)
    }

    // ── CRDT & Federation ─────────────────────────────────────────────────

    /// Apply a batch of remote CRDT operations (federation sync).
    pub fn apply_crdt_delta(&mut self, remote_log: &CrdtLog) -> HypergridResult<usize> {
        use crate::crdt::CrdtOperation;
        let ops = remote_log.delta_since(&self.node_id);
        let mut applied = 0;
        for op in ops {
            match op {
                CrdtOperation::SetField { cell_ref, attribute, value, timestamp, vector_clock, actor } => {
                    if let Some(cell) = self.ucs.get_mut(cell_ref.cube_id, &DimCoordinate::new(cell_ref.coord.clone())) {
                        let remote_newer = match cell.get_attr(attribute) {
                            None => true,
                            Some(_) => cell.updated_at < *timestamp,
                        };
                        if remote_newer {
                            cell.set_attr(attribute.clone(), value.clone(), actor);
                            cell.vector_clock.merge(vector_clock);
                        }
                        applied += 1;
                    }
                }
                _ => {} // other op types handled by sub-systems (graph, schema)
            }
        }
        Ok(applied)
    }

    // ── Vector Clock ──────────────────────────────────────────────────────

    fn tick(&mut self) -> VectorClock {
        self.vector_clock.tick(&self.node_id.clone());
        self.vector_clock.clone()
    }

    // ── Plugin System ─────────────────────────────────────────────────────

    pub fn register_plugin(&mut self, plugin: Box<dyn HypercubePlugin>) {
        self.plugins.push(plugin);
    }

    // ── Stats ─────────────────────────────────────────────────────────────

    pub fn stats(&self) -> GridStats {
        GridStats {
            grid_id: self.grid_id,
            cube_count: self.cubes.len(),
            total_cells: self.ucs.total_cells(),
            event_count: self.event_log.len(),
            crdt_op_count: self.crdt_log.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GridStats {
    pub grid_id: GridId,
    pub cube_count: usize,
    pub total_cells: usize,
    pub event_count: usize,
    pub crdt_op_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_grid() -> Grid {
        Grid::new("test-grid", "node-1")
    }

    #[test]
    fn create_cube_and_write_cell() {
        let mut grid = make_grid();
        let cube_id = grid.create_cube("Projects").unwrap();
        let cube = grid.get_cube_mut(cube_id).unwrap();
        // register a "name" attribute
        use crate::cell::AttributeKeyDef;
        cube.attr_registry.register(AttributeKeyDef::text("name", "Name")).unwrap();

        let coord = DimCoordinate::d2(
            DimKey::uuid(Uuid::new_v4()),
            DimKey::str("name"),
        );
        grid.write_cell(cube_id, coord.clone(), "name", TypedAttrValue::Text("Alpha".into()), "node-1").unwrap();

        let val = grid.read_cell(cube_id, &coord, "name").unwrap();
        assert_eq!(val, Some(&TypedAttrValue::Text("Alpha".into())));
    }

    #[test]
    fn add_custom_axis() {
        let mut grid = make_grid();
        let cube_id = grid.create_cube("Budget Cube").unwrap();
        let time_axis = DimensionAxis::time_axis(3);
        let axis_id = grid.add_axis_to_cube(cube_id, time_axis).unwrap();
        let cube = grid.get_cube(cube_id).unwrap();
        assert_eq!(cube.n(), 3);
    }
}
