use std::collections::HashMap;

use chrono::Utc;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use serde_json::json;
use uuid::Uuid;
use serde::de::DeserializeOwned;

use apapo::{ApapoRuntime, SovereignKind, CrossGridLink};
use hypergrid::graph::ConsentStatus;
use hypergrid::cell::{DimCoordinate, DimKey, EntityRef, TypedAttrValue, HyperCell};
use hypergrid::domain::{
    ComponentStore, PortfolioComponent, ComponentCategory, ItemCategory, ContainerCategory,
    ComponentState, Visibility,
};
use hypergrid::space::NamespaceEntry;
use hypergrid::{Grid, HypergridError};
use hypergrid::spreadsheet::SpreadsheetCubes;

use crate::config::KogiPortfolioConfig;
use crate::error::KogiPortfolioResult;
use crate::spreadsheet::{
    SpreadsheetWorkbook, PortfolioRow, SheetRegistry, ComputationEngine, ViewDefinition, ViewEngine,
};

#[derive(Debug, Clone)]
pub struct KogiPortfolioCubes {
    pub components_cube: hypergrid::CubeId,
    pub spreadsheet: SpreadsheetCubes,
}

#[derive(Debug)]
pub struct MasterPortfolioSpreadsheet {
    pub workbook_id: Uuid,
    pub root_component_id: Uuid,
    pub workbook: SpreadsheetWorkbook,
}

pub struct KogiPortfolioRuntime {
    pub config: KogiPortfolioConfig,
    pub apapo: ApapoRuntime,
    pub cubes: KogiPortfolioCubes,
    pub master: MasterPortfolioSpreadsheet,
}

#[derive(Debug, Clone)]
pub struct LinkEdgeView {
    pub link_id: Uuid,
    pub phase: apapo::LinkPhase,
    pub consent: ConsentStatus,
    pub source: EntityRef,
    pub target: EntityRef,
    pub source_row: Option<PortfolioRow>,
    pub target_row: Option<PortfolioRow>,
}

#[derive(Debug, Clone)]
pub struct LinkForestView {
    pub root: EntityRef,
    pub links: Vec<LinkEdgeView>,
}

impl KogiPortfolioRuntime {
    pub fn new(config: KogiPortfolioConfig) -> KogiPortfolioResult<Self> {
        let apapo = ApapoRuntime::new(config.apapo_config())?;
        apapo.register_builtin_domains();
        let _status = apapo.bootstrap()?;

        let components_cube = ComponentStore::bootstrap(&apapo.grid)?;
        let spreadsheet = SpreadsheetCubes::bootstrap(&apapo.grid)?;

        let _ = apapo.services.namespace.register_cube_path("kogi://portfolio/workbooks/", spreadsheet.workbook_cube);
        let _ = apapo.services.namespace.register_cube_path("kogi://portfolio/sheets/", spreadsheet.sheet_cube);
        let _ = apapo.services.namespace.register_cube_path("kogi://portfolio/columns/", spreadsheet.column_cube);
        let _ = apapo.services.namespace.register_cube_path("kogi://portfolio/cells/", spreadsheet.cell_cube);
        let _ = apapo.services.namespace.register_cube_path("kogi://portfolio/views/", spreadsheet.view_cube);
        let _ = apapo.services.namespace.register_cube_path("kogi://portfolio/links/", spreadsheet.link_cube);

        let root_component_id = ensure_master_component(
            &apapo.grid,
            components_cube,
            &config.node_id,
            config.owner_id,
        )?;
        let workbook = load_workbook_from_grid(&apapo.grid, &spreadsheet, root_component_id)
            .unwrap_or_else(|| SpreadsheetWorkbook::new(config.owner_id, "Master Portfolio Spreadsheet"));
        let workbook_id = workbook.workbook_id;

        let mut master = MasterPortfolioSpreadsheet {
            workbook_id,
            root_component_id,
            workbook,
        };
        master.sync_from_grid(&apapo.grid, components_cube)?;
        let engine = ComputationEngine::default_columns();
        master.workbook.write_computed_columns(&engine, "kogi-engine");
        let _ = link_workbook_to_root(&apapo.grid, components_cube, root_component_id, workbook_id, &config.node_id);
        master.persist_metadata(&apapo.grid, &spreadsheet, &config.node_id)?;

        Ok(Self {
            config,
            apapo,
            cubes: KogiPortfolioCubes { components_cube, spreadsheet },
            master,
        })
    }

    pub fn ensure_owner_identity(&self, name: &str, handle: &str, kind: SovereignKind) -> KogiPortfolioResult<Uuid> {
        let sovereign_id = self.apapo.services.identity.register_sovereign(name, kind)?;
        let _ = self.apapo.services.identity.register_identity(sovereign_id, handle)?;
        Ok(sovereign_id)
    }

    pub fn link_forest_view(&self, root_component_id: Uuid) -> LinkForestView {
        let root = EntityRef {
            cube_id: self.cubes.components_cube,
            d1_key: root_component_id,
            grid_id: Some(self.apapo.grid.grid_id),
        };
        let forest = self.apapo.services.crossgrid.link_forest(&root);
        let links = forest.links.into_iter().map(|link| link_edge_view(&self.master.workbook, link)).collect();
        LinkForestView { root, links }
    }

    pub fn link_forest_rows(&self, root_component_id: Uuid) -> Vec<PortfolioRow> {
        let view = self.link_forest_view(root_component_id);
        let mut rows = HashMap::new();
        for link in view.links {
            if let Some(row) = link.source_row {
                rows.insert(row.component_id, row);
            }
            if let Some(row) = link.target_row {
                rows.insert(row.component_id, row);
            }
        }
        rows.into_values().collect()
    }

    pub fn link_forest_view_rows(&self, root_component_id: Uuid, view: &ViewDefinition) -> Vec<PortfolioRow> {
        let rows = self.link_forest_rows(root_component_id);
        ViewEngine::apply(&rows, &self.master.workbook.cell_store, view)
    }
}

impl MasterPortfolioSpreadsheet {
    pub fn sync_from_grid(&mut self, grid: &Grid, components_cube: hypergrid::CubeId) -> KogiPortfolioResult<()> {
        let rows = grid.scan_rows(components_cube);
        for row in rows {
            let id = match row.d1_key.as_uuid() {
                Some(id) => id,
                None => continue,
            };
            if let Ok(Some(component)) = ComponentStore::read(grid, components_cube, id) {
                let row = portfolio_row_from_component(&component);
                self.workbook.upsert_row(row);
            }
        }
        Ok(())
    }

    pub fn persist_metadata(&self, grid: &Grid, cubes: &SpreadsheetCubes, actor: &str) -> KogiPortfolioResult<()> {
        persist_workbook(grid, cubes.workbook_cube, self, actor)?;
        persist_sheets(grid, cubes.sheet_cube, self.workbook_id, &self.workbook.sheets, actor)?;
        persist_columns(grid, cubes.column_cube, self.workbook_id, &self.workbook.sheets, actor)?;
        persist_cells(grid, cubes.cell_cube, &self.workbook.cell_store, actor)?;
        Ok(())
    }
}

fn ensure_master_component(
    grid: &Grid,
    components_cube: hypergrid::CubeId,
    node_id: &str,
    owner_id: Uuid,
) -> KogiPortfolioResult<Uuid> {
    let namespace = "kogi://portfolio/master/";
    if let Some(entry) = grid.namespace_reg.get_entry(namespace) {
        return Ok(entry.entity_ref.d1_key);
    }

    let mut component = PortfolioComponent::new(owner_id, "Master Portfolio Spreadsheet", ComponentCategory::Item(ItemCategory::Portfolio));
    component.description = "Root portfolio spreadsheet".to_owned();
    component.payload = Some(json!({"kind": "master_spreadsheet"}));
    component.metadata.namespace_path = Some(namespace.to_owned());

    ComponentStore::write(grid, components_cube, &component, node_id)?;
    grid.hypergraph.register_entity(components_cube, DimKey::uuid(component.metadata.id), grid.grid_id);

    let entity_ref = EntityRef { cube_id: components_cube, d1_key: component.metadata.id, grid_id: Some(grid.grid_id) };
    let entry = NamespaceEntry::new(namespace.to_owned(), entity_ref, "Row");
    let _ = grid.namespace_reg.register(entry);

    Ok(component.metadata.id)
}

fn link_workbook_to_root(
    grid: &Grid,
    components_cube: hypergrid::CubeId,
    root_component_id: Uuid,
    workbook_id: Uuid,
    actor: &str,
) -> Result<(), HypergridError> {
    if let Ok(Some(mut component)) = ComponentStore::read(grid, components_cube, root_component_id) {
        let mut payload = component.payload.unwrap_or_else(|| json!({}));
        if payload.get("workbook_id").is_none() {
            payload["workbook_id"] = json!(workbook_id.to_string());
            component.payload = Some(payload);
            ComponentStore::write(grid, components_cube, &component, actor)?;
        }
    }
    Ok(())
}

fn load_workbook_from_grid(
    grid: &Grid,
    cubes: &SpreadsheetCubes,
    root_component_id: Uuid,
) -> Option<SpreadsheetWorkbook> {
    let workbook_fields = collect_fields(grid.scan_cells(cubes.workbook_cube));
    for (d1, fields) in workbook_fields {
        let root_match = fields.get("root_component_id").and_then(attr_to_uuid);
        if root_match != Some(root_component_id) {
            continue;
        }
        let workbook_id = d1.as_uuid()?;
        let name = fields.get("name").and_then(attr_to_text).unwrap_or_else(|| "Master Portfolio Spreadsheet".to_owned());
        let owner_id = fields.get("owner_id").and_then(attr_to_uuid).unwrap_or(Uuid::nil());
        let sheet_order = fields.get("sheet_order").and_then(attr_to_json::<Vec<String>>).unwrap_or_default();

        let column_defs = load_column_definitions(grid, cubes.column_cube, workbook_id);
        let mut sheets = load_sheet_registry(grid, cubes.sheet_cube, workbook_id, &column_defs);
        if !sheet_order.is_empty() {
            sheets.set_order(sheet_order);
        }

        let mut workbook = SpreadsheetWorkbook::with_id(workbook_id, owner_id, name);
        workbook.sheets = sheets;
        workbook.cell_store = load_cell_store(grid, cubes.cell_cube);
        return Some(workbook);
    }
    None
}

fn portfolio_row_from_component(component: &PortfolioComponent) -> PortfolioRow {
    let owner = component.metadata.owners.first().copied().unwrap_or(Uuid::nil());
    let mut row = PortfolioRow::new(component.metadata.id, component_type_string(&component.category), component.name.clone(), owner);
    row.item_type = item_type_string(&component.category);
    row.container_type = container_type_string(&component.category);
    row.status = component.status.to_string();
    row.state = state_string(&component.state);
    row.visibility = visibility_string(&component.visibility);
    row.created_at = component.metadata.created_at;
    row.updated_at = component.metadata.updated_at;
    row.owners = component.metadata.owners.clone();
    row.tags = component.metadata.tags.iter().cloned().collect();
    row.hashtags = component.hashtags.iter().cloned().collect();
    row.topics = component.topics.iter().cloned().collect();
    row.policy_ids = component.metadata.policy_ids.clone();
    row.budget_allocated = Decimal::from_f64(component.budget);
    row.budget_spent = Decimal::from_f64(component.budget_spent).unwrap_or(Decimal::ZERO);
    row.resource_units_total = Some(component.resource_units);
    row.parent_id = component.parents.first().copied();
    row.child_ids = component.children.clone();
    row.dependency_ids = component.dependencies.clone();
    row.dependent_ids = component.dependents.clone();
    row.linked_ids = component.links.clone();
    row.toolbox_ids = component.toolbox_ids.clone();
    if let Some(ns) = &component.metadata.namespace_path {
        row.ext.insert("namespace_path".to_owned(), json!(ns));
    }
    if let Some(space) = component.metadata.space_id {
        row.ext.insert("space_id".to_owned(), json!(space));
    }
    if !component.metadata.workspace_ids.is_empty() {
        row.ext.insert("workspace_ids".to_owned(), json!(component.metadata.workspace_ids));
    }
    if !component.metadata.identity_tags.is_empty() {
        row.ext.insert("identity_tags".to_owned(), json!(component.metadata.identity_tags));
    }
    if !component.metadata.properties.is_empty() {
        row.ext.insert("properties".to_owned(), json!(component.metadata.properties));
    }
    row
}

fn link_edge_view(workbook: &SpreadsheetWorkbook, link: CrossGridLink) -> LinkEdgeView {
    let source_row = workbook.get_row(&link.source.d1_key).cloned();
    let target_row = workbook.get_row(&link.target.d1_key).cloned();
    LinkEdgeView {
        link_id: link.link_id,
        phase: link.phase,
        consent: link.consent,
        source: link.source,
        target: link.target,
        source_row,
        target_row,
    }
}

fn component_type_string(category: &ComponentCategory) -> String {
    match category {
        ComponentCategory::Item(_) => "item".to_owned(),
        ComponentCategory::Container(_) => "container".to_owned(),
    }
}

fn item_type_string(category: &ComponentCategory) -> Option<String> {
    match category {
        ComponentCategory::Item(item) => Some(item_category_string(item)),
        ComponentCategory::Container(_) => None,
    }
}

fn container_type_string(category: &ComponentCategory) -> Option<String> {
    match category {
        ComponentCategory::Container(container) => Some(container_category_string(container)),
        ComponentCategory::Item(_) => None,
    }
}

fn item_category_string(category: &ItemCategory) -> String {
    match category {
        ItemCategory::Portfolio => "Portfolio".to_owned(),
        ItemCategory::Program => "Program".to_owned(),
        ItemCategory::Project => "Project".to_owned(),
        ItemCategory::Task => "Task".to_owned(),
        ItemCategory::Resource => "Resource".to_owned(),
        ItemCategory::Artifact => "Artifact".to_owned(),
        ItemCategory::Asset => "Asset".to_owned(),
        ItemCategory::SubPortfolio => "SubPortfolio".to_owned(),
        ItemCategory::BenefitAccount => "BenefitAccount".to_owned(),
        ItemCategory::Gig => "Gig".to_owned(),
        ItemCategory::Contract => "Contract".to_owned(),
        ItemCategory::Job => "Job".to_owned(),
        ItemCategory::Campaign => "Campaign".to_owned(),
        ItemCategory::Grant => "Grant".to_owned(),
        ItemCategory::Investment => "Investment".to_owned(),
        ItemCategory::Profile => "Profile".to_owned(),
        ItemCategory::Custom(s) => s.clone(),
    }
}

fn container_category_string(category: &ContainerCategory) -> String {
    match category {
        ContainerCategory::Binder => "Binder".to_owned(),
        ContainerCategory::Book(kind) => format!("Book::{kind:?}"),
        ContainerCategory::Record => "Record".to_owned(),
        ContainerCategory::Folder => "Folder".to_owned(),
        ContainerCategory::Registry => "Registry".to_owned(),
        ContainerCategory::Archive => "Archive".to_owned(),
        ContainerCategory::Workspace => "Workspace".to_owned(),
        ContainerCategory::Namespace => "Namespace".to_owned(),
        ContainerCategory::Toolbox => "Toolbox".to_owned(),
        ContainerCategory::LinkNet => "LinkNet".to_owned(),
        ContainerCategory::LinkTree => "LinkTree".to_owned(),
        ContainerCategory::LinkForest => "LinkForest".to_owned(),
        ContainerCategory::Schedule => "Schedule".to_owned(),
        ContainerCategory::Calendar => "Calendar".to_owned(),
        ContainerCategory::Roadmap => "Roadmap".to_owned(),
        ContainerCategory::Gantt => "Gantt".to_owned(),
        ContainerCategory::Timebox => "Timebox".to_owned(),
        ContainerCategory::Wallet => "Wallet".to_owned(),
        ContainerCategory::Account => "Account".to_owned(),
        ContainerCategory::Room => "Room".to_owned(),
        ContainerCategory::Chat => "Chat".to_owned(),
        ContainerCategory::Directory => "Directory".to_owned(),
        ContainerCategory::Graph => "Graph".to_owned(),
        ContainerCategory::Matrix => "Matrix".to_owned(),
        ContainerCategory::Grid => "Grid".to_owned(),
        ContainerCategory::Form => "Form".to_owned(),
        ContainerCategory::Resume => "Resume".to_owned(),
        ContainerCategory::Backlog => "Backlog".to_owned(),
    }
}

fn state_string(state: &ComponentState) -> String {
    match state {
        ComponentState::Custom(s) => s.clone(),
        other => format!("{other:?}"),
    }
}

fn visibility_string(visibility: &Visibility) -> String {
    match visibility {
        Visibility::Private => "Private".to_owned(),
        Visibility::Protected => "Protected".to_owned(),
        Visibility::Public => "Public".to_owned(),
        Visibility::Unlisted => "Unlisted".to_owned(),
        Visibility::DraftOnly => "DraftOnly".to_owned(),
    }
}

fn persist_workbook(
    grid: &Grid,
    workbook_cube: hypergrid::CubeId,
    master: &MasterPortfolioSpreadsheet,
    actor: &str,
) -> Result<(), HypergridError> {
    let coord = |field: &str| DimCoordinate::n2(grid.grid_id, workbook_cube, DimKey::uuid(master.workbook_id), DimKey::text(field));
    grid.write_cell(workbook_cube, coord("name"), "value", TypedAttrValue::Text(master.workbook.name.clone()), actor)?;
    grid.write_cell(workbook_cube, coord("owner_id"), "value", TypedAttrValue::Text(master.workbook.owner_id.to_string()), actor)?;
    grid.write_cell(workbook_cube, coord("root_component_id"), "value", TypedAttrValue::Text(master.root_component_id.to_string()), actor)?;
    grid.write_cell(workbook_cube, coord("sheet_order"), "value", TypedAttrValue::Json(json!(master.workbook.sheets.order())), actor)?;
    grid.write_cell(workbook_cube, coord("updated_at"), "value", TypedAttrValue::DateTime(Utc::now()), actor)?;
    Ok(())
}

fn persist_sheets(
    grid: &Grid,
    sheet_cube: hypergrid::CubeId,
    workbook_id: Uuid,
    registry: &SheetRegistry,
    actor: &str,
) -> Result<(), HypergridError> {
    for sheet in registry.all() {
        let coord = |field: &str| DimCoordinate::n2(grid.grid_id, sheet_cube, DimKey::text(sheet.id.clone()), DimKey::text(field));
        grid.write_cell(sheet_cube, coord("name"), "value", TypedAttrValue::Text(sheet.name.clone()), actor)?;
        grid.write_cell(sheet_cube, coord("description"), "value", TypedAttrValue::Text(sheet.description.clone()), actor)?;
        grid.write_cell(sheet_cube, coord("row_types"), "value", TypedAttrValue::Json(json!(sheet.row_types)), actor)?;
        grid.write_cell(sheet_cube, coord("columns"), "value", TypedAttrValue::Json(json!(sheet.column_schema.columns)), actor)?;
        grid.write_cell(sheet_cube, coord("workbook_id"), "value", TypedAttrValue::Text(workbook_id.to_string()), actor)?;
        grid.write_cell(sheet_cube, coord("is_builtin"), "value", TypedAttrValue::Bool(sheet.is_builtin), actor)?;
    }
    Ok(())
}

fn persist_columns(
    grid: &Grid,
    column_cube: hypergrid::CubeId,
    workbook_id: Uuid,
    registry: &SheetRegistry,
    actor: &str,
) -> Result<(), HypergridError> {
    let mut seen: HashMap<String, crate::spreadsheet::ColumnDefinition> = HashMap::new();
    for sheet in registry.all() {
        for column in &sheet.column_schema.columns {
            seen.entry(column.id.clone()).or_insert_with(|| column.clone());
        }
    }
    for (column_id, definition) in seen {
        let coord = |field: &str| DimCoordinate::n2(grid.grid_id, column_cube, DimKey::text(column_id.clone()), DimKey::text(field));
        grid.write_cell(column_cube, coord("definition"), "value", TypedAttrValue::Json(json!(definition)), actor)?;
        grid.write_cell(column_cube, coord("workbook_id"), "value", TypedAttrValue::Text(workbook_id.to_string()), actor)?;
    }
    Ok(())
}

fn persist_cells(
    grid: &Grid,
    cell_cube: hypergrid::CubeId,
    cells: &crate::spreadsheet::CellStore,
    actor: &str,
) -> Result<(), HypergridError> {
    for ((row_id, column_id), value) in cells.iter() {
        let coord = DimCoordinate::n2(grid.grid_id, cell_cube, DimKey::uuid(*row_id), DimKey::text(column_id.clone()));
        grid.write_cell(cell_cube, coord, "value", TypedAttrValue::Json(json!(value)), actor)?;
    }
    Ok(())
}

fn collect_fields(cells: Vec<HyperCell>) -> HashMap<DimKey, HashMap<String, TypedAttrValue>> {
    let mut fields: HashMap<DimKey, HashMap<String, TypedAttrValue>> = HashMap::new();
    for cell in cells {
        let d1 = cell.coord.d1().cloned().unwrap_or(DimKey::Null);
        let d2 = cell.coord.d2().and_then(|k| k.as_str()).unwrap_or("").to_owned();
        if d2.is_empty() { continue; }
        fields.entry(d1).or_default().insert(d2, cell.value.clone());
    }
    fields
}

fn load_column_definitions(
    grid: &Grid,
    column_cube: hypergrid::CubeId,
    workbook_id: Uuid,
) -> HashMap<String, crate::spreadsheet::ColumnDefinition> {
    let mut output = HashMap::new();
    let field_map = collect_fields(grid.scan_cells(column_cube));
    for (d1, fields) in field_map {
        let column_id = d1.as_str().unwrap_or("").to_owned();
        if column_id.is_empty() { continue; }
        let column_workbook = fields.get("workbook_id").and_then(attr_to_uuid);
        if column_workbook != Some(workbook_id) { continue; }
        if let Some(def) = fields.get("definition").and_then(attr_to_json::<crate::spreadsheet::ColumnDefinition>) {
            output.insert(column_id, def);
        }
    }
    output
}

fn load_sheet_registry(
    grid: &Grid,
    sheet_cube: hypergrid::CubeId,
    workbook_id: Uuid,
    columns: &HashMap<String, crate::spreadsheet::ColumnDefinition>,
) -> SheetRegistry {
    let mut registry = SheetRegistry::empty();
    let field_map = collect_fields(grid.scan_cells(sheet_cube));
    for (d1, fields) in field_map {
        let sheet_id = d1.as_str().unwrap_or("").to_owned();
        if sheet_id.is_empty() { continue; }
        let sheet_workbook = fields.get("workbook_id").and_then(attr_to_uuid);
        if sheet_workbook != Some(workbook_id) { continue; }

        let name = fields.get("name").and_then(attr_to_text).unwrap_or_else(|| sheet_id.clone());
        let description = fields.get("description").and_then(attr_to_text).unwrap_or_default();
        let row_types = fields.get("row_types").and_then(attr_to_json::<Vec<String>>).unwrap_or_default();
        let is_builtin = fields.get("is_builtin").and_then(attr_to_bool).unwrap_or(false);
        let column_defs = if let Some(cols) = fields.get("columns").and_then(attr_to_json::<Vec<crate::spreadsheet::ColumnDefinition>>) {
            cols
        } else {
            columns.values().cloned().collect()
        };

        let sheet = crate::spreadsheet::SheetDefinition {
            id: sheet_id.clone(),
            name,
            description,
            row_types,
            column_schema: crate::spreadsheet::ColumnSchema::new(column_defs),
            is_builtin,
        };
        registry.register(sheet);
    }
    registry
}

fn load_cell_store(grid: &Grid, cell_cube: hypergrid::CubeId) -> crate::spreadsheet::CellStore {
    let mut store = crate::spreadsheet::CellStore::default();
    let cells = grid.scan_cells(cell_cube);
    for cell in cells {
        if cell.attr_key != "value" { continue; }
        let row_id = cell.coord.d1().and_then(|k| k.as_uuid());
        let column_id = cell.coord.d2().and_then(|k| k.as_str()).map(|s| s.to_owned());
        let value = match &cell.value {
            TypedAttrValue::Json(val) => serde_json::from_value::<crate::spreadsheet::CellValue>(val.clone()).ok(),
            TypedAttrValue::Text(text) => Some(crate::spreadsheet::CellValue::Text(text.clone())),
            _ => None,
        };
        if let (Some(rid), Some(cid), Some(val)) = (row_id, column_id, value) {
            store.set_cell(rid, cid, val, "rehydrate", None);
        }
    }
    store
}

fn attr_to_text(value: &TypedAttrValue) -> Option<String> {
    match value {
        TypedAttrValue::Text(v) => Some(v.clone()),
        TypedAttrValue::Json(j) => j.as_str().map(|s| s.to_owned()),
        _ => None,
    }
}

fn attr_to_bool(value: &TypedAttrValue) -> Option<bool> {
    match value {
        TypedAttrValue::Bool(v) => Some(*v),
        TypedAttrValue::Json(j) => j.as_bool(),
        _ => None,
    }
}

fn attr_to_uuid(value: &TypedAttrValue) -> Option<Uuid> {
    let text = attr_to_text(value)?;
    Uuid::parse_str(&text).ok()
}

fn attr_to_json<T: DeserializeOwned>(value: &TypedAttrValue) -> Option<T> {
    match value {
        TypedAttrValue::Json(j) => serde_json::from_value(j.clone()).ok(),
        _ => None,
    }
}
