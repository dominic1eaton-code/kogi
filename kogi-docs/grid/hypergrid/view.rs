// =============================================================================
// hypergrid::view — HG-VIEW: View Engine
//
// HypercubeView, RenderMode, DimFold, DimExpand, ViewSort, ViewGroup,
// HighlightRule, ShadowCellPolicy, ViewRegistry
// =============================================================================

use std::collections::HashMap;
use std::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::cell::{AggFn, AttributeKey, AxisId, CubeId, DimKey, PluginId, SpaceId, UserId};
use crate::dim::{DimSlicePredicate, DimSlice};

// ─── RenderMode ──────────────────────────────────────────────────────────────

/// How a HypercubeView should be rendered in the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderMode {
    /// Standard 2D spreadsheet grid.
    Grid2D,
    /// Kanban board: swimlane columns = enum axis values; cards = row entities.
    Kanban { status_attr: AttributeKey, wip_limit: Option<u32> },
    /// Agile sprint board: columns = sprint status; burndown overlay.
    Agile  { sprint_attr: AttributeKey },
    /// Gantt chart: entities as rows; time as horizontal bars.
    Gantt  { start_attr: AttributeKey, end_attr: AttributeKey },
    /// Calendar: entities displayed on a calendar by date attribute.
    Calendar { date_attr: AttributeKey, granularity: CalendarGranularity },
    /// Timeline / roadmap view.
    Timeline { start_attr: AttributeKey, end_attr: AttributeKey },
    /// Hierarchical tree view (requires HierarchyAxis).
    HierarchyTree,
    /// Treemap: tiles sized by numeric attr; color by score attr.
    Treemap { size_attr: AttributeKey, color_attr: Option<AttributeKey> },
    /// Force-directed network graph.
    NetworkGraph { edge_types: Vec<crate::graph::EdgeType> },
    /// Link forest: KLNK-style inter-portfolio connection tree.
    LinkForest,
    /// Pivot table: DimFold + DimExpand in a combined view.
    PivotTable,
    /// Color-coded matrix for pattern detection.
    HeatMap { value_attr: AttributeKey },
    /// N-dimensional explorer: user can select any two axes as rows/columns.
    NDimExplorer,
    /// Custom render mode implemented by a HypercubePlugin.
    Custom(PluginId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalendarGranularity { Day, Week, Month, Quarter, Year }

// ─── DimFold ─────────────────────────────────────────────────────────────────

/// Collapse a dimension axis by aggregating all its keys into a scalar summary.
/// Example: FOLD D₃ (TimeAxis) WITH SUM(revenue) collapses all time periods.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimFold {
    pub axis_id:     AxisId,
    pub agg_fn:      AggFn,
    pub source_attr: AttributeKey,
    pub alias:       Option<String>,     // output column name
    pub last_n:      Option<u32>,        // only fold the last N periods (TimeAxis)
}

// ─── DimExpand ───────────────────────────────────────────────────────────────

/// Expand a dimension axis's key set into separate output columns (PIVOT).
/// Example: EXPAND D₄ (CategoryAxis) → columns: "category_a", "category_b", …
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimExpand {
    pub axis_id:    AxisId,
    pub agg_fn:     AggFn,
    pub col_prefix: Option<String>,
}

// ─── ViewSort ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewSort {
    pub attr_key:     AttributeKey,
    pub direction:    SortDirection,
    pub nulls_last:   bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortDirection { Asc, Desc }

// ─── ViewGroup ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewGroup {
    pub attr_key:     AttributeKey,
    pub aggregations: Vec<GroupAgg>,
    pub collapsed:    bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupAgg { pub attr_key: AttributeKey, pub agg_fn: AggFn }

// ─── HighlightRule ────────────────────────────────────────────────────────────

/// Conditional row/cell highlighting for visual anomaly detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightRule {
    pub condition:  String,      // formula expression, e.g. "health_score < 50"
    pub color_hex:  String,      // e.g. "#FEE2E2"
    pub label:      Option<String>, // e.g. "At Risk"
}

// ─── ShadowCellPolicy ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShadowCellPolicy {
    Include,     // include local rows AND shadow rows from CrossGridLinks
    Exclude,     // exclude all shadow rows (show only local data)
    ShadowOnly,  // show ONLY shadow rows (cross-grid data lens)
}

// ─── AttrColumnDef ───────────────────────────────────────────────────────────

/// Display configuration for one attribute key in a HypercubeView.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttrColumnDef {
    pub attr_key:     AttributeKey,
    pub display_name: Option<String>,  // override from AttributeKeyDef.display_name
    pub width:        u32,             // DXA (20 DXA = 1 point)
    pub pinned:       bool,            // always visible (left freeze pane)
    pub hidden:       bool,
    pub format:       Option<String>,  // display format hint (e.g. "0.0%", "currency:USD")
}

// ─── ViewVisibility ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewVisibility {
    Private,      // Only the creator can see this view
    Shared,       // Shared with specific users (via sharing settings)
    SpaceShared,  // All Space members can access
    Public,       // Anyone with the link can access
    Template,     // Available as a template in the Space's view library
}

// ─── SummaryRowConfig ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryRowConfig {
    pub position:      SummaryPosition,
    pub aggregations:  Vec<GroupAgg>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SummaryPosition { Top, Bottom }

// ─── HypercubeView ───────────────────────────────────────────────────────────

/// A complete, shareable, versionable view configuration over a Hypercube.
/// The primary user-facing artifact in Hypergrid — what the user opens and works in.
///
/// HypercubeViews:
///   - Select which axes map to "rows" and "columns" (axis remapping)
///   - Apply DimSlice filters to focus on a subset of the data
///   - Define DimFolds and DimExpands for analytical projections
///   - Configure column order, widths, sorts, groups, and highlights
///   - Select the render mode (Grid2D, Kanban, Gantt, HeatMap, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypercubeView {
    pub view_id:       Uuid,
    pub cube_id:       CubeId,
    pub name:          String,
    pub namespace_path: Option<String>,

    // ── Axis remapping ─────────────────────────────────────────────────────
    /// Which axis maps to "rows" in 2D rendering. Default: D₁ (EntityAxis).
    pub primary_axis:   AxisId,
    /// Which axis maps to "columns" in 2D rendering. Default: D₂ (PropertyAxis).
    pub secondary_axis: AxisId,

    // ── Dimensionality configuration ───────────────────────────────────────
    /// DimSlice predicates applied to non-primary/secondary axes.
    pub dim_slices:    Vec<DimSlicePredicate>,
    /// Axes to collapse by aggregation (FOLD).
    pub dim_folds:     Vec<DimFold>,
    /// Axes to expand as pivot columns (EXPAND).
    pub dim_expands:   Vec<DimExpand>,

    // ── Column schema ──────────────────────────────────────────────────────
    pub attr_schema:   Vec<AttrColumnDef>,
    pub pinned_attrs:  Vec<AttributeKey>,   // always-visible (left freeze)
    pub hidden_attrs:  Vec<AttributeKey>,

    // ── Filtering / sorting / grouping ─────────────────────────────────────
    pub filters:       Vec<DimSlicePredicate>,
    pub sorts:         Vec<ViewSort>,
    pub groups:        Vec<ViewGroup>,

    // ── Rendering ──────────────────────────────────────────────────────────
    pub render_mode:   RenderMode,
    pub highlight_rules: Vec<HighlightRule>,
    pub summary_row:   Option<SummaryRowConfig>,

    // ── Augmentation ───────────────────────────────────────────────────────
    pub shadow_policy: ShadowCellPolicy,
    pub identity_tag:  Option<String>,
    pub space_filter:  Option<SpaceId>,
    pub graph_overlay: bool,    // show Hypergraph edges as arrows in Grid2D
    pub ai_overlay:    bool,    // show AiSignal values inline with confidence badges

    // ── Sharing ────────────────────────────────────────────────────────────
    pub visibility:    ViewVisibility,
    pub created_by:    UserId,
    pub created_at:    chrono::DateTime<chrono::Utc>,
    pub updated_at:    chrono::DateTime<chrono::Utc>,
}

impl HypercubeView {
    pub fn new(
        cube_id: CubeId, name: impl Into<String>,
        primary_axis: AxisId, secondary_axis: AxisId,
        created_by: UserId,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            view_id:       Uuid::new_v4(),
            cube_id,
            name:          name.into(),
            namespace_path: None,
            primary_axis,
            secondary_axis,
            dim_slices:    vec![],
            dim_folds:     vec![],
            dim_expands:   vec![],
            attr_schema:   vec![],
            pinned_attrs:  vec![],
            hidden_attrs:  vec![],
            filters:       vec![],
            sorts:         vec![],
            groups:        vec![],
            render_mode:   RenderMode::Grid2D,
            highlight_rules: vec![],
            summary_row:   None,
            shadow_policy: ShadowCellPolicy::Exclude,
            identity_tag:  None,
            space_filter:  None,
            graph_overlay: false,
            ai_overlay:    true,
            visibility:    ViewVisibility::Private,
            created_by,
            created_at:    now,
            updated_at:    now,
        }
    }

    /// Grid2D view with default settings.
    pub fn grid2d(cube_id: CubeId, name: impl Into<String>, created_by: UserId) -> Self {
        let placeholder_axis = Uuid::nil();
        Self::new(cube_id, name, placeholder_axis, placeholder_axis, created_by)
    }

    pub fn add_filter(mut self, filter: DimSlicePredicate) -> Self {
        self.filters.push(filter);
        self
    }

    pub fn add_sort(mut self, attr_key: impl Into<String>, dir: SortDirection) -> Self {
        self.sorts.push(ViewSort { attr_key: attr_key.into(), direction: dir, nulls_last: true });
        self
    }

    pub fn add_highlight(mut self, condition: impl Into<String>, color: impl Into<String>, label: Option<&str>) -> Self {
        self.highlight_rules.push(HighlightRule {
            condition: condition.into(),
            color_hex: color.into(),
            label: label.map(|s| s.to_owned()),
        });
        self
    }

    pub fn with_render_mode(mut self, mode: RenderMode) -> Self {
        self.render_mode = mode;
        self
    }

    pub fn with_ai_overlay(mut self) -> Self {
        self.ai_overlay = true;
        self
    }

    pub fn with_shadow_cells(mut self) -> Self {
        self.shadow_policy = ShadowCellPolicy::Include;
        self
    }

    pub fn for_space(mut self, space_id: SpaceId) -> Self {
        self.space_filter = Some(space_id);
        self
    }
}

// ─── ViewRegistry ─────────────────────────────────────────────────────────────

pub struct ViewRegistry {
    views:    RwLock<HashMap<Uuid, HypercubeView>>,
    by_cube:  RwLock<HashMap<CubeId, Vec<Uuid>>>,
    by_space: RwLock<HashMap<SpaceId, Vec<Uuid>>>,
}

impl ViewRegistry {
    pub fn new() -> Self {
        Self { views: Default::default(), by_cube: Default::default(), by_space: Default::default() }
    }

    pub fn register(&self, view: HypercubeView) {
        let view_id  = view.view_id;
        let cube_id  = view.cube_id;
        let space_id = view.space_filter;
        self.views.write().unwrap().insert(view_id, view);
        self.by_cube.write().unwrap().entry(cube_id).or_default().push(view_id);
        if let Some(sid) = space_id {
            self.by_space.write().unwrap().entry(sid).or_default().push(view_id);
        }
    }

    pub fn get(&self, view_id: Uuid) -> Option<HypercubeView> {
        self.views.read().unwrap().get(&view_id).cloned()
    }

    pub fn update(&self, view: HypercubeView) {
        self.views.write().unwrap().insert(view.view_id, view);
    }

    pub fn delete(&self, view_id: Uuid) -> Option<HypercubeView> {
        self.views.write().unwrap().remove(&view_id)
    }

    pub fn for_cube(&self, cube_id: CubeId) -> Vec<HypercubeView> {
        let ids = self.by_cube.read().unwrap().get(&cube_id).cloned().unwrap_or_default();
        let store = self.views.read().unwrap();
        ids.iter().filter_map(|id| store.get(id).cloned()).collect()
    }

    pub fn for_space(&self, space_id: SpaceId) -> Vec<HypercubeView> {
        let ids = self.by_space.read().unwrap().get(&space_id).cloned().unwrap_or_default();
        let store = self.views.read().unwrap();
        ids.iter().filter_map(|id| store.get(id).cloned()).collect()
    }
}

// ─── Predefined views for Kogi (sample) ────────────────────────────────────

/// Create the standard Kogi "Active Projects" view.
pub fn kogi_active_projects_view(cube_id: CubeId, created_by: UserId) -> HypercubeView {
    HypercubeView::grid2d(cube_id, "Active Projects", created_by)
        .add_filter(DimSlicePredicate {
            dim_index: 2,
            predicate: crate::dim::DimKeyPredicate::Eq(crate::cell::DimKey::text("status")),
        })
        .add_sort("health_score", SortDirection::Asc)
        .add_highlight("health_score < 50", "#FEE2E2", Some("At Risk"))
        .add_highlight("health_score >= 80", "#D1FAE5", Some("Healthy"))
        .with_ai_overlay()
}

/// Create the standard Kogi "Finances" view.
pub fn kogi_finances_view(cube_id: CubeId, created_by: UserId) -> HypercubeView {
    HypercubeView::grid2d(cube_id, "Finances", created_by)
        .add_sort("budget_spent", SortDirection::Desc)
        .add_highlight("budget_remaining < 0", "#FEE2E2", Some("Over Budget"))
        .with_ai_overlay()
}

/// Create the standard Kogi "Risk Dashboard" Kanban view.
pub fn kogi_risk_kanban_view(cube_id: CubeId, created_by: UserId) -> HypercubeView {
    let mut view = HypercubeView::grid2d(cube_id, "Risk Dashboard", created_by);
    view.render_mode = RenderMode::Kanban {
        status_attr: "status".into(),
        wip_limit: None,
    };
    view.add_sort("risk_score", SortDirection::Desc)
}
