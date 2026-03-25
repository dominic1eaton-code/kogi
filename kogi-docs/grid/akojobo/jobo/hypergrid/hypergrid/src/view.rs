//! HG-VIEW — HypercubeView engine: N-dim filter, sort, group, render modes.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::cell::{AttributeKey, CubeId, IdentityId};
use crate::dim::{DimFold, DimExpand, DimSlicePredicate};
use crate::space::SpaceId;

pub type ViewId = Uuid;
pub type AxisId = Uuid;

// ─── Render Mode ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RenderMode {
    Grid2D,
    PivotTable,
    HeatMap,
    Matrix,
    Kanban,
    Agile,
    Gantt,
    Calendar,
    Timeline,
    HierarchyTree,
    Treemap,
    NetworkGraph,
    LinkForest,
    NDimExplorer,
    Custom(Uuid),
}

// ─── View Visibility ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ViewVisibility {
    Private, Shared, SpaceShared, Public, Template,
}

// ─── Shadow Cell Policy ──────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ShadowCellPolicy {
    Include, Exclude, ShadowOnly,
}

// ─── View Sort ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewSort {
    pub attribute: AttributeKey,
    pub ascending: bool,
}

// ─── View Group ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewGroup {
    pub axis_id: AxisId,
    pub attribute: Option<AttributeKey>,
}

// ─── AttrColumnDef ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttrColumnDef {
    pub attribute: AttributeKey,
    pub width: u32,
    pub visible: bool,
    pub header_label: Option<String>,
}

// ─── Highlight Rule ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightRule {
    pub attribute: AttributeKey,
    pub condition: String,          // e.g. "> 80", "== 'Active'"
    pub color: String,              // hex color code
    pub label: Option<String>,
}

// ─── Summary Row Config ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryRowConfig {
    pub attribute: AttributeKey,
    pub agg_fn: crate::cell::AggFn,
    pub label: String,
}

// ─── HypercubeView ────────────────────────────────────────────────────────────

/// A saved, shareable configuration of N-dimensional slices, sorts, groups, and render mode.
/// A HypercubeView is the generalization of a spreadsheet "sheet tab."
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypercubeView {
    pub view_id: ViewId,
    pub cube_id: CubeId,
    pub name: String,
    pub namespace_path: String,

    // ── Dimensionality configuration ──────────────────────────────────────
    /// Which axis maps to "rows" in 2D rendering.
    pub primary_axis: AxisId,
    /// Which axis maps to "columns" in 2D rendering.
    pub secondary_axis: AxisId,
    /// Predicates on all other axes (reduces the sub-cube).
    pub dim_slices: Vec<DimSlicePredicate>,
    /// Axes collapsed by aggregation.
    pub dim_folds: Vec<DimFold>,
    /// Axes expanded as pivot columns.
    pub dim_expands: Vec<DimExpand>,

    // ── Column/row configuration ──────────────────────────────────────────
    pub attr_schema: Vec<AttrColumnDef>,
    pub pinned_attrs: Vec<AttributeKey>,
    pub hidden_attrs: Vec<AttributeKey>,
    pub col_widths: HashMap<AttributeKey, u32>,

    // ── Filtering / Sorting / Grouping ────────────────────────────────────
    pub filters: Vec<DimSlicePredicate>,
    pub sorts: Vec<ViewSort>,
    pub groups: Vec<ViewGroup>,

    // ── Rendering ─────────────────────────────────────────────────────────
    pub render_mode: RenderMode,

    // ── Augmentation ──────────────────────────────────────────────────────
    pub highlight_rules: Vec<HighlightRule>,
    pub summary_row: Option<SummaryRowConfig>,
    pub shadow_policy: ShadowCellPolicy,
    pub identity_filter: Option<IdentityId>,
    pub space_filter: Option<SpaceId>,
    pub graph_overlay: bool,
    pub ai_overlay: bool,

    // ── Sharing ───────────────────────────────────────────────────────────
    pub visibility: ViewVisibility,
    pub created_by: IdentityId,
    pub metadata: crate::cell::AttributeMap,
}

impl HypercubeView {
    pub fn new_2d_grid(
        cube_id: CubeId,
        name: impl Into<String>,
        primary_axis: AxisId,
        secondary_axis: AxisId,
        created_by: IdentityId,
    ) -> Self {
        Self {
            view_id: Uuid::new_v4(),
            cube_id,
            name: name.into(),
            namespace_path: String::new(),
            primary_axis,
            secondary_axis,
            dim_slices: vec![],
            dim_folds: vec![],
            dim_expands: vec![],
            attr_schema: vec![],
            pinned_attrs: vec![],
            hidden_attrs: vec![],
            col_widths: HashMap::new(),
            filters: vec![],
            sorts: vec![],
            groups: vec![],
            render_mode: RenderMode::Grid2D,
            highlight_rules: vec![],
            summary_row: None,
            shadow_policy: ShadowCellPolicy::Include,
            identity_filter: None,
            space_filter: None,
            graph_overlay: false,
            ai_overlay: false,
            visibility: ViewVisibility::Private,
            created_by,
            metadata: crate::cell::AttributeMap::new(),
        }
    }

    pub fn with_render_mode(mut self, mode: RenderMode) -> Self {
        self.render_mode = mode;
        self
    }

    pub fn with_filter(mut self, filter: DimSlicePredicate) -> Self {
        self.filters.push(filter);
        self
    }

    pub fn with_sort(mut self, sort: ViewSort) -> Self {
        self.sorts.push(sort);
        self
    }
}
