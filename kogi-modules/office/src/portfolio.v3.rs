// =============================================================================
//  portfolio_spreadsheet.rs — Kogi OS · Portfolio Spreadsheet Substrate (KPSS)
//  Independent Worker Operating System
//
//  KPSS is the low-level spreadsheet data engine that underpins the entire
//  Portfolio Management System.  Every PortfolioComponent is a PortfolioRow.
//  Every field is a typed, versioned CellValue stored in a CellStore.
//  Every named "view" is a SheetDefinition over the universal PortfolioRow schema.
//
//  SDD §§ implemented here:
//    §2  — CellValue, ComputedCellValue
//    §3  — ColumnType taxonomy (§7.1 of SDD)
//    §4  — ColumnDef, ColumnSchema
//    §5  — PortfolioRow
//    §6  — CellStore (indexed, versioned, TTL-aware)
//    §7  — SheetDefinition, SheetRegistry
//    §8  — SpreadsheetWorkbook (root container; §1.1 metaphor)
//    §9  — Structural Primitives: Group, Collection, List, Schedule, Directory
//    §10 — Built-in sheet constants (SHT-001 … SHT-030)
//    §11 — Universal column set
//
//  @version  2.2.0
//  @license  MIT
// =============================================================================

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::fmt;

use chrono::{DateTime, Duration, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Cross-module opaque IDs ───────────────────────────────────────────────────
pub type ComponentId  = Uuid;
pub type EntityId     = Uuid;
pub type PolicyId     = Uuid;
pub type AccountId    = Uuid;
pub type InvoiceId    = Uuid;
pub type FileId       = Uuid;
pub type SprintId     = Uuid;
pub type ColumnId     = String;
pub type SheetId      = String;
pub type ViewId       = Uuid;
pub type CurrencyCode = String;
pub type HexColor     = String;
pub type Slug         = String;

// =============================================================================
// §2 — CellValue
// =============================================================================

/// Every cell in the portfolio spreadsheet holds a strongly-typed `CellValue`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CellValue {
    Text(String),
    Number(Decimal),
    Currency { amount: Decimal, currency: CurrencyCode },
    Percent(f32),
    Bool(bool),
    Url(String),
    RichText(String),
    Date(NaiveDate),
    DateTime(DateTime<Utc>),
    DurationSecs(i64),
    Enum(String),
    MultiEnum(Vec<String>),
    Relation(ComponentId),
    MultiRelation(Vec<ComponentId>),
    User(EntityId),
    MultiUser(Vec<EntityId>),
    Tags(Vec<String>),
    FileRef(FileId),
    AccountRef(AccountId),
    Computed(Box<ComputedCellValue>),
    Null,
}

impl CellValue {
    pub fn is_null(&self) -> bool { matches!(self, Self::Null) }

    pub fn as_decimal(&self) -> Option<Decimal> {
        match self {
            Self::Number(d)              => Some(*d),
            Self::Currency { amount, .. } => Some(*amount),
            Self::Percent(p)             => Decimal::try_from(*p as f64).ok(),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Text(s) | Self::Url(s) | Self::RichText(s) | Self::Enum(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

impl fmt::Display for CellValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(s) | Self::Url(s) | Self::RichText(s) | Self::Enum(s) => write!(f, "{s}"),
            Self::Number(d)                  => write!(f, "{d}"),
            Self::Currency { amount, currency } => write!(f, "{amount} {currency}"),
            Self::Percent(p)                 => write!(f, "{p:.1}%"),
            Self::Bool(b)                    => write!(f, "{b}"),
            Self::Date(d)                    => write!(f, "{d}"),
            Self::DateTime(dt)               => write!(f, "{dt}"),
            Self::DurationSecs(s)            => write!(f, "{}h {}m", s / 3600, (s % 3600) / 60),
            Self::Tags(t)                    => write!(f, "{}", t.join(", ")),
            Self::MultiEnum(v)               => write!(f, "{}", v.join(", ")),
            Self::Null                       => write!(f, "—"),
            _                                => write!(f, "<ref>"),
        }
    }
}

/// A derived or AI-generated cell value with provenance metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComputedCellValue {
    pub value:        Box<CellValue>,
    pub formula:      Option<String>,
    pub computed_by:  String,           // "engine:PortfolioHealth" | "formula" | "rollup"
    pub computed_at:  DateTime<Utc>,
    pub ttl_secs:     Option<u32>,
    pub stale:        bool,
}

impl ComputedCellValue {
    pub fn expiry(&self) -> Option<DateTime<Utc>> {
        self.ttl_secs.map(|s| self.computed_at + Duration::seconds(s as i64))
    }

    pub fn is_fresh(&self) -> bool {
        if self.stale { return false; }
        if let Some(exp) = self.expiry() {
            return Utc::now() < exp;
        }
        true
    }
}

// =============================================================================
// §3 — ColumnType  (SDD §7.1)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ColumnType {
    // Primitive
    TextField, NumberField, CurrencyField, PercentField,
    DateField, DateTimeField, DurationField,
    BoolField, UrlField, RichTextField,
    // Categorical
    EnumField, MultiEnumField,
    // Relational
    RelationField, MultiRelationField, UserField, MultiUserField,
    TagField, FileRefField,
    // Derived
    ComputedColumn, AIColumn, FormulaColumn, AuditColumn, AnalyticsColumn,
}

impl ColumnType {
    pub fn is_read_only(&self) -> bool {
        matches!(self, Self::ComputedColumn | Self::AIColumn | Self::AuditColumn | Self::AnalyticsColumn)
    }
    pub fn is_numeric(&self) -> bool {
        matches!(self, Self::NumberField | Self::CurrencyField | Self::PercentField
                     | Self::ComputedColumn | Self::AIColumn | Self::FormulaColumn | Self::AnalyticsColumn)
    }
    pub fn is_sortable(&self) -> bool {
        !matches!(self, Self::RichTextField | Self::MultiRelationField | Self::MultiUserField)
    }
    pub fn is_searchable(&self) -> bool {
        matches!(self, Self::TextField | Self::RichTextField | Self::TagField | Self::UrlField)
    }
}

// =============================================================================
// §4 — ColumnDef and ColumnSchema
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnVisibility { Public, Trusted, Private }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnWidth { Compact, Normal, Wide, ExtraWide, Custom(u32) }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Aggregation { Sum, Average, Count, CountDistinct, Min, Max, First, Last, None }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnScope { Sheet(SheetId), Global }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumOption {
    pub value: String,
    pub label: String,
    pub color: HexColor,
    pub icon:  Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnValidation {
    pub required:  bool,
    pub min:       Option<Decimal>,
    pub max:       Option<Decimal>,
    pub regex:     Option<String>,
    pub enum_set:  Vec<String>,
}

/// Full column definition — the schema entry for one column.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDef {
    pub id:               ColumnId,
    pub display_name:     String,
    pub column_type:      ColumnType,
    pub description:      Option<String>,
    pub width:            ColumnWidth,
    pub pinned:           bool,
    pub hidden:           bool,
    pub visibility:       ColumnVisibility,
    pub aggregation:      Aggregation,
    pub enum_options:     Vec<EnumOption>,
    pub relation_targets: Vec<String>,
    pub formula:          Option<String>,
    pub compute_model:    Option<String>,
    pub is_custom:        bool,
    pub scope:            ColumnScope,
    pub default_value:    Option<CellValue>,
    pub validation:       Option<ColumnValidation>,
    pub created_by:       Option<EntityId>,
    pub created_at:       DateTime<Utc>,
}

impl ColumnDef {
    pub fn built_in(id: impl Into<ColumnId>, name: impl Into<String>, col_type: ColumnType) -> Self {
        Self {
            id: id.into(), display_name: name.into(), column_type: col_type,
            description: None, width: ColumnWidth::Normal, pinned: false, hidden: false,
            visibility: ColumnVisibility::Public, aggregation: Aggregation::None,
            enum_options: vec![], relation_targets: vec![], formula: None,
            compute_model: None, is_custom: false, scope: ColumnScope::Global,
            default_value: None, validation: None, created_by: None, created_at: Utc::now(),
        }
    }

    pub fn computed(id: impl Into<ColumnId>, name: impl Into<String>, model: impl Into<String>) -> Self {
        let mut col = Self::built_in(id, name, ColumnType::ComputedColumn);
        col.compute_model = Some(model.into());
        col
    }

    pub fn formula_col(id: impl Into<ColumnId>, name: impl Into<String>, formula: impl Into<String>) -> Self {
        let mut col = Self::built_in(id, name, ColumnType::FormulaColumn);
        col.formula = Some(formula.into());
        col
    }

    pub fn ai_col(id: impl Into<ColumnId>, name: impl Into<String>, model: impl Into<String>) -> Self {
        let mut col = Self::built_in(id, name, ColumnType::AIColumn);
        col.compute_model = Some(model.into());
        col
    }
}

/// An ordered set of column definitions for one sheet's schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnSchema {
    pub columns: Vec<ColumnDef>,
}

impl ColumnSchema {
    pub fn new(columns: Vec<ColumnDef>) -> Self { Self { columns } }
    pub fn empty() -> Self { Self { columns: vec![] } }

    pub fn get(&self, id: &ColumnId) -> Option<&ColumnDef> {
        self.columns.iter().find(|c| &c.id == id)
    }

    pub fn visible(&self) -> Vec<&ColumnDef> {
        let mut out: Vec<_> = self.columns.iter().filter(|c| c.pinned && !c.hidden).collect();
        out.extend(self.columns.iter().filter(|c| !c.pinned && !c.hidden));
        out
    }

    pub fn add_column(&mut self, col: ColumnDef) -> Result<(), String> {
        if self.columns.iter().any(|c| c.id == col.id) {
            return Err(format!("Column '{}' already exists", col.id));
        }
        self.columns.push(col);
        Ok(())
    }

    pub fn remove_column(&mut self, id: &ColumnId) -> Result<(), String> {
        let pos = self.columns.iter().position(|c| &c.id == id)
            .ok_or_else(|| format!("Column '{id}' not found"))?;
        if !self.columns[pos].is_custom {
            return Err(format!("Column '{id}' is built-in and cannot be removed"));
        }
        self.columns.remove(pos);
        Ok(())
    }

    pub fn reorder(&mut self, id: &ColumnId, new_index: usize) -> Result<(), String> {
        let pos = self.columns.iter().position(|c| &c.id == id)
            .ok_or_else(|| format!("Column '{id}' not found"))?;
        let col = self.columns.remove(pos);
        let target = new_index.min(self.columns.len());
        self.columns.insert(target, col);
        Ok(())
    }

    pub fn pinned_ids(&self) -> Vec<&ColumnId> {
        self.columns.iter().filter(|c| c.pinned).map(|c| &c.id).collect()
    }

    pub fn numeric_columns(&self) -> Vec<&ColumnDef> {
        self.columns.iter().filter(|c| c.column_type.is_numeric()).collect()
    }
}

// =============================================================================
// §5 — PortfolioRow
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RowHeight { Compact, Normal, Tall, Auto }

/// One row in the master spreadsheet — a typed, navigable snapshot of a component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioRow {
    pub row_id:         ComponentId,
    pub slug:           Option<Slug>,
    pub component_type: String,      // "item" | "container"
    pub type_name:      String,      // "Project" | "Asset" | "Binder" | …
    pub name:           String,
    pub cells:          HashMap<ColumnId, CellValue>,
    pub depth:          u32,
    pub parent_id:      Option<ComponentId>,
    pub child_ids:      Vec<ComponentId>,
    pub color:          Option<HexColor>,
    pub icon:           Option<String>,
    pub height:         RowHeight,
    pub annotation:     Option<String>,  // Oba inline annotation
    pub highlight:      Option<HexColor>,// from RowHighlightRule
}

impl PortfolioRow {
    pub fn new(
        row_id: ComponentId,
        component_type: impl Into<String>,
        type_name: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            row_id, slug: None,
            component_type: component_type.into(),
            type_name: type_name.into(),
            name: name.into(),
            cells: HashMap::new(),
            depth: 0, parent_id: None, child_ids: vec![],
            color: None, icon: None, height: RowHeight::Normal,
            annotation: None, highlight: None,
        }
    }

    pub fn get(&self, col: &ColumnId) -> &CellValue {
        self.cells.get(col).unwrap_or(&CellValue::Null)
    }

    pub fn set(&mut self, col: impl Into<ColumnId>, val: CellValue) {
        self.cells.insert(col.into(), val);
    }

    pub fn get_decimal(&self, col: &ColumnId) -> Option<Decimal> {
        self.cells.get(col)?.as_decimal()
    }

    pub fn get_str(&self, col: &ColumnId) -> Option<&str> {
        self.cells.get(col)?.as_str()
    }
}

// =============================================================================
// §6 — CellStore
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellEntry {
    pub row_id:     ComponentId,
    pub column_id:  ColumnId,
    pub value:      CellValue,
    pub version:    u64,
    pub written_by: EntityId,
    pub written_at: DateTime<Utc>,
}

/// Aggregate statistics for a numeric column across a result set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnStats {
    pub column_id:  ColumnId,
    pub sum:        Decimal,
    pub average:    Decimal,
    pub min:        Decimal,
    pub max:        Decimal,
    pub count:      usize,
    pub null_count: usize,
}

/// Central cell storage.  Primary key: (ComponentId, ColumnId).
/// Computed values cached separately to allow TTL management.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CellStore {
    /// User-written values: row → column → entry
    entries: HashMap<ComponentId, HashMap<ColumnId, CellEntry>>,
    /// Computed/cached values: row → column → computed
    cache:   HashMap<ComponentId, HashMap<ColumnId, ComputedCellValue>>,
    /// Monotonic version counter
    version: u64,
}

impl CellStore {
    pub fn new() -> Self { Self::default() }

    pub fn write(
        &mut self,
        row_id: ComponentId,
        column_id: impl Into<ColumnId>,
        value: CellValue,
        writer: EntityId,
    ) -> u64 {
        self.version += 1;
        let v = self.version;
        self.entries.entry(row_id).or_default().insert(
            column_id.into(),
            CellEntry { row_id, column_id: String::new(), value, version: v, written_by: writer, written_at: Utc::now() },
        );
        v
    }

    pub fn write_computed(
        &mut self,
        row_id: ComponentId,
        column_id: impl Into<ColumnId>,
        computed: ComputedCellValue,
    ) {
        self.cache.entry(row_id).or_default().insert(column_id.into(), computed);
    }

    pub fn read(&self, row_id: &ComponentId, column_id: &ColumnId) -> CellValue {
        // 1. Fresh computed value
        if let Some(cv) = self.cache.get(row_id).and_then(|m| m.get(column_id)) {
            if cv.is_fresh() {
                return *cv.value.clone();
            }
        }
        // 2. User-written value
        self.entries
            .get(row_id)
            .and_then(|cols| cols.get(column_id))
            .map(|e| e.value.clone())
            .unwrap_or(CellValue::Null)
    }

    /// Mark all cached computed values for a row as stale.
    pub fn invalidate_cache(&mut self, row_id: &ComponentId) {
        if let Some(m) = self.cache.get_mut(row_id) {
            for cv in m.values_mut() { cv.stale = true; }
        }
    }

    pub fn column_stats(&self, row_ids: &[ComponentId], column_id: &ColumnId) -> ColumnStats {
        let mut sum = Decimal::ZERO;
        let mut min: Option<Decimal> = None;
        let mut max: Option<Decimal> = None;
        let mut count = 0usize;
        let mut null_count = 0usize;

        for rid in row_ids {
            if let Some(d) = self.read(rid, column_id).as_decimal() {
                sum += d;
                count += 1;
                min = Some(min.map_or(d, |m: Decimal| m.min(d)));
                max = Some(max.map_or(d, |m: Decimal| m.max(d)));
            } else {
                null_count += 1;
            }
        }

        ColumnStats {
            column_id: column_id.clone(),
            average: if count > 0 { sum / Decimal::from(count) } else { Decimal::ZERO },
            sum,
            min: min.unwrap_or(Decimal::ZERO),
            max: max.unwrap_or(Decimal::ZERO),
            count,
            null_count,
        }
    }

    pub fn all_row_ids(&self) -> impl Iterator<Item = &ComponentId> {
        self.entries.keys()
    }

    pub fn row_entry_count(&self, row_id: &ComponentId) -> usize {
        self.entries.get(row_id).map(|m| m.len()).unwrap_or(0)
    }
}

// =============================================================================
// §7 — SheetDefinition and SheetRegistry
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetRowTypeFilter {
    pub include_types:    Vec<String>,
    pub exclude_statuses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetDefaultGroup {
    pub column_id: ColumnId,
    pub secondary: Option<ColumnId>,
}

/// Persistent specification for a named sheet (SHT-NNN).
/// A sheet is a view recipe, not a query result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetDefinition {
    pub sheet_id:        SheetId,
    pub name:            String,
    pub description:     String,
    pub row_types:       SheetRowTypeFilter,
    pub schema:          ColumnSchema,
    pub default_group:   Option<SheetDefaultGroup>,
    pub pinned_columns:  Vec<ColumnId>,
    pub is_builtin:      bool,
    pub is_visible:      bool,
    pub computed_models: Vec<String>,
    pub created_by:      Option<EntityId>,
    pub created_at:      DateTime<Utc>,
    pub updated_at:      DateTime<Utc>,
}

impl SheetDefinition {
    pub fn builtin(
        sheet_id: impl Into<SheetId>,
        name: impl Into<String>,
        description: impl Into<String>,
        include_types: Vec<String>,
        schema: ColumnSchema,
    ) -> Self {
        let now = Utc::now();
        Self {
            sheet_id: sheet_id.into(), name: name.into(), description: description.into(),
            row_types: SheetRowTypeFilter {
                include_types,
                exclude_statuses: vec!["Archived".into()],
            },
            schema,
            default_group: None,
            pinned_columns: vec!["component_id".into(), "name".into()],
            is_builtin: true, is_visible: true,
            computed_models: vec![],
            created_by: None, created_at: now, updated_at: now,
        }
    }
}

/// All sheets in a workbook, with preserved navigation order.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SheetRegistry {
    order:  Vec<SheetId>,
    sheets: HashMap<SheetId, SheetDefinition>,
}

impl SheetRegistry {
    pub fn new() -> Self { Self::default() }

    pub fn register(&mut self, sheet: SheetDefinition) {
        let id = sheet.sheet_id.clone();
        if !self.order.contains(&id) { self.order.push(id.clone()); }
        self.sheets.insert(id, sheet);
    }

    pub fn get(&self, id: &SheetId) -> Option<&SheetDefinition> { self.sheets.get(id) }
    pub fn get_mut(&mut self, id: &SheetId) -> Option<&mut SheetDefinition> { self.sheets.get_mut(id) }

    pub fn all(&self) -> Vec<&SheetDefinition> {
        self.order.iter().filter_map(|id| self.sheets.get(id)).collect()
    }
    pub fn visible(&self) -> Vec<&SheetDefinition> {
        self.all().into_iter().filter(|s| s.is_visible).collect()
    }

    pub fn remove(&mut self, id: &SheetId) -> Result<SheetDefinition, String> {
        let sheet = self.sheets.get(id).ok_or_else(|| format!("Sheet '{id}' not found"))?;
        if sheet.is_builtin { return Err(format!("Sheet '{id}' is built-in")); }
        self.order.retain(|s| s != id);
        Ok(self.sheets.remove(id).unwrap())
    }

    pub fn reorder(&mut self, id: &SheetId, new_index: usize) {
        self.order.retain(|s| s != id);
        let target = new_index.min(self.order.len());
        self.order.insert(target, id.clone());
    }

    pub fn len(&self) -> usize { self.sheets.len() }
}

// =============================================================================
// §8 — SpreadsheetWorkbook
// =============================================================================

/// The root workbook — one per worker/org.  The "Excel workbook" of the Kogi OS.
/// Contains all sheets, all cell data, and the row cache.
#[derive(Debug, Serialize, Deserialize)]
pub struct SpreadsheetWorkbook {
    pub owner_id:   EntityId,
    pub cells:      CellStore,
    pub sheets:     SheetRegistry,
    #[serde(skip)]
    row_cache:      HashMap<ComponentId, PortfolioRow>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SpreadsheetWorkbook {
    pub fn new(owner_id: EntityId) -> Self {
        let mut wb = Self {
            owner_id, cells: CellStore::new(), sheets: SheetRegistry::new(),
            row_cache: HashMap::new(), created_at: Utc::now(), updated_at: Utc::now(),
        };
        wb.register_builtin_sheets();
        wb
    }

    fn register_builtin_sheets(&mut self) {
        for (id, name, desc, types) in BUILTIN_SHEETS.iter() {
            let schema = ColumnSchema::new(universal_column_set());
            let sheet  = SheetDefinition::builtin(
                *id, *name, *desc,
                types.iter().map(|s| s.to_string()).collect(),
                schema,
            );
            self.sheets.register(sheet);
        }
    }

    pub fn write_cell(
        &mut self,
        row_id: ComponentId,
        column_id: impl Into<ColumnId>,
        value: CellValue,
        writer: EntityId,
    ) -> u64 {
        let v = self.cells.write(row_id, column_id, value, writer);
        self.row_cache.remove(&row_id);
        self.updated_at = Utc::now();
        v
    }

    pub fn write_computed_cell(
        &mut self,
        row_id: ComponentId,
        column_id: impl Into<ColumnId>,
        computed: ComputedCellValue,
    ) {
        self.cells.write_computed(row_id, column_id, computed);
        self.row_cache.remove(&row_id);
    }

    pub fn read_cell(&self, row_id: &ComponentId, column_id: &ColumnId) -> CellValue {
        self.cells.read(row_id, column_id)
    }

    /// Materialise a PortfolioRow from cell store, using cache when available.
    pub fn materialise_row(&mut self, row_id: ComponentId, schema: &ColumnSchema) -> PortfolioRow {
        if let Some(cached) = self.row_cache.get(&row_id) {
            return cached.clone();
        }
        let name = match self.cells.read(&row_id, &"name".into()) {
            CellValue::Text(s) => s,
            _                  => row_id.to_string(),
        };
        let component_type = match self.cells.read(&row_id, &"component_type".into()) {
            CellValue::Text(s) | CellValue::Enum(s) => s,
            _                                        => "item".into(),
        };
        let type_name = match self.cells.read(&row_id, &"type_name".into()) {
            CellValue::Text(s) | CellValue::Enum(s) => s,
            _                                        => "Unknown".into(),
        };

        let mut row = PortfolioRow::new(row_id, component_type, type_name, name);
        for col in &schema.columns {
            row.cells.insert(col.id.clone(), self.cells.read(&row_id, &col.id));
        }

        self.row_cache.insert(row_id, row.clone());
        row
    }

    pub fn invalidate_row(&mut self, row_id: &ComponentId) {
        self.row_cache.remove(row_id);
        self.cells.invalidate_cache(row_id);
    }

    pub fn column_stats(&self, row_ids: &[ComponentId], column_id: &ColumnId) -> ColumnStats {
        self.cells.column_stats(row_ids, column_id)
    }

    /// Return all row IDs that have cell data.
    pub fn all_row_ids(&self) -> Vec<ComponentId> {
        self.cells.all_row_ids().copied().collect()
    }
}

// =============================================================================
// §9 — Structural Primitives  (SDD §12)
// =============================================================================

/// Dynamic, labeled cluster of rows sharing a column value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub label:     String,
    pub column_id: ColumnId,
    pub value:     CellValue,
    pub row_ids:   Vec<ComponentId>,
    pub summary:   HashMap<ColumnId, CellValue>,
}

/// Static, named, user-defined set of ComponentIds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id:          Uuid,
    pub name:        String,
    pub description: Option<String>,
    pub owner_id:    EntityId,
    pub row_ids:     HashSet<ComponentId>,
    pub visibility:  String,
    pub created_at:  DateTime<Utc>,
}

impl Collection {
    pub fn new(name: impl Into<String>, owner: EntityId) -> Self {
        Self {
            id: Uuid::new_v4(), name: name.into(), description: None, owner_id: owner,
            row_ids: HashSet::new(), visibility: "Private".into(), created_at: Utc::now(),
        }
    }
    pub fn add(&mut self, id: ComponentId)     { self.row_ids.insert(id); }
    pub fn remove(&mut self, id: &ComponentId) { self.row_ids.remove(id); }
    pub fn contains(&self, id: &ComponentId)   -> bool { self.row_ids.contains(id) }
    pub fn len(&self)                           -> usize { self.row_ids.len() }
    pub fn is_empty(&self)                      -> bool  { self.row_ids.is_empty() }
}

/// Ordered, persistent sequence of ComponentIds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct List {
    pub id:         Uuid,
    pub name:       String,
    pub owner_id:   EntityId,
    pub items:      Vec<ComponentId>,
    pub created_at: DateTime<Utc>,
}

impl List {
    pub fn new(name: impl Into<String>, owner: EntityId) -> Self {
        Self { id: Uuid::new_v4(), name: name.into(), owner_id: owner,
               items: vec![], created_at: Utc::now() }
    }
    pub fn push(&mut self, id: ComponentId) { self.items.push(id); }
    pub fn move_to(&mut self, id: ComponentId, idx: usize) {
        self.items.retain(|&x| x != id);
        let target = idx.min(self.items.len());
        self.items.insert(target, id);
    }
    pub fn remove(&mut self, id: &ComponentId) { self.items.retain(|x| x != id); }
    pub fn len(&self)      -> usize { self.items.len() }
    pub fn is_empty(&self) -> bool  { self.items.is_empty() }
}

/// Causal sequence ordered by start_date + dependency graph sort.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub id:    Uuid,
    pub name:  String,
    pub items: Vec<ScheduleItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleItem {
    pub component_id: ComponentId,
    pub start_date:   Option<NaiveDate>,
    pub end_date:     Option<NaiveDate>,
    pub dependencies: Vec<ComponentId>,
}

/// Hierarchical path-addressed directory of components.
/// Items addressed as: /programs/q2/projects/api-redesign
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Directory {
    pub id:    Uuid,
    pub name:  String,
    paths: HashMap<String, ComponentId>,
    index: HashMap<ComponentId, String>,
}

impl Directory {
    pub fn new(name: impl Into<String>) -> Self {
        Self { id: Uuid::new_v4(), name: name.into(), paths: HashMap::new(), index: HashMap::new() }
    }
    pub fn insert(&mut self, path: impl Into<String>, id: ComponentId) {
        let p = path.into();
        self.paths.insert(p.clone(), id);
        self.index.insert(id, p);
    }
    pub fn get(&self, path: &str)           -> Option<&ComponentId> { self.paths.get(path) }
    pub fn path_of(&self, id: &ComponentId) -> Option<&str>         { self.index.get(id).map(|s| s.as_str()) }
    pub fn children(&self, prefix: &str) -> Vec<(&str, &ComponentId)> {
        self.paths.iter()
            .filter(|(p, _)| p.starts_with(prefix) && p.len() > prefix.len())
            .map(|(p, id)| (p.as_str(), id))
            .collect()
    }
    pub fn remove(&mut self, id: &ComponentId) {
        if let Some(p) = self.index.remove(id) { self.paths.remove(&p); }
    }
}

// =============================================================================
// §10 — Built-in Sheet Registry  (SDD §4.1)
// =============================================================================

/// (SheetId, Name, Description, Row Types)
pub const BUILTIN_SHEETS: &[(&str, &str, &str, &[&str])] = &[
    ("SHT-001", "Master Registry",             "All portfolio components — universal view",                      &[]),
    ("SHT-002", "Portfolio Hierarchy",         "Structural tree: Portfolio → Program → Project → Task",         &["Portfolio","SubPortfolio","Program","Project"]),
    ("SHT-003", "Programs",                    "Strategic programs and initiatives",                            &["Program"]),
    ("SHT-004", "Projects",                    "All projects across all programs",                              &["Project"]),
    ("SHT-005", "Tasks & Backlog",             "Tasks, stories, epics, and features",                          &["Task"]),
    ("SHT-006", "Resources",                   "Human, financial, and equipment resources",                     &["Resource"]),
    ("SHT-007", "Assets",                      "Capital and intellectual assets",                               &["Asset"]),
    ("SHT-008", "Artifacts",                   "Documents, files, and generated outputs",                       &["Artifact"]),
    ("SHT-009", "Finances",                    "Unified financial ledger across all portfolio entities",         &[]),
    ("SHT-010", "Budget Tracker",              "Budget allocation and spend tracking by program/project",       &["Program","Project","Gig","Contract"]),
    ("SHT-011", "Work & Gigs",                 "All income-generating engagements",                             &["Gig","Contract","Job"]),
    ("SHT-012", "Deliverables",                "Project outputs, artifact releases, and milestones",            &["Project","Artifact"]),
    ("SHT-013", "Timeline",                    "All dated components on a timeline",                            &[]),
    ("SHT-014", "Roadmap",                     "Programs and projects on a quarter roadmap",                    &["Program","Project"]),
    ("SHT-015", "Portable Benefits",           "All portable benefit accounts (HSA, Retirement, PTO, …)",       &["BenefitAccount"]),
    ("SHT-016", "Grants & Microfinancing",     "Grant applications, microloans, and community lending",         &["Grant"]),
    ("SHT-017", "Equity Crowdfunding",         "Equity crowdfunding campaigns and investment instruments",       &["Campaign","Investment"]),
    ("SHT-018", "Contracts",                   "Formal agreements, NDAs, and SOW contracts",                    &["Contract"]),
    ("SHT-019", "Jobs & Consulting",           "Longer-term employment and consulting engagements",             &["Job"]),
    ("SHT-020", "Shared Portfolios",           "Co-owned, cooperative, and crowdresourced portfolios",          &["Portfolio","Program","Project"]),
    ("SHT-021", "Collaboration",               "Contribution-centric view of all shared components",            &[]),
    ("SHT-022", "Governance",                  "Policies, approval queues, compliance flags, and risk flags",   &[]),
    ("SHT-023", "Event Log",                   "Append-only audit trail of all portfolio mutations",            &[]),
    ("SHT-024", "Snapshots",                   "Point-in-time portfolio snapshots and restore points",          &[]),
    ("SHT-025", "Contacts",                    "People and organization profiles in the worker's network",      &["Profile"]),
    ("SHT-026", "Marketplace Listings",        "Assets and resources published to kogi-marketplace",            &["Asset","Resource"]),
    ("SHT-027", "Deals",                       "Active deal proposals and exchange transactions",               &[]),
    ("SHT-028", "Archive",                     "Deep-storage of completed and retired components",              &[]),
    ("SHT-029", "Analytics",                   "Engagement metrics, performance KPIs, and usage analytics",     &[]),
    ("SHT-030", "Engine Signals",              "AI-derived scores, recommendations, and anomaly flags",         &[]),
];

// =============================================================================
// §11 — Universal column set (the baseline every sheet inherits)
// =============================================================================

pub fn universal_column_set() -> Vec<ColumnDef> {
    vec![
        // ── Identity ──────────────────────────────────────────────────────────
        ColumnDef::built_in("component_id",    "ID",            ColumnType::TextField),
        ColumnDef::built_in("name",            "Name",          ColumnType::TextField),
        ColumnDef::built_in("slug",            "Slug",          ColumnType::TextField),
        ColumnDef::built_in("component_type",  "Type",          ColumnType::EnumField),
        ColumnDef::built_in("type_name",       "Kind",          ColumnType::EnumField),
        ColumnDef::built_in("display_name",    "Display Name",  ColumnType::TextField),
        ColumnDef::built_in("icon",            "Icon",          ColumnType::TextField),
        ColumnDef::built_in("color",           "Color",         ColumnType::TextField),

        // ── Lifecycle ─────────────────────────────────────────────────────────
        ColumnDef::built_in("status",          "Status",        ColumnType::EnumField),
        ColumnDef::built_in("state",           "State",         ColumnType::EnumField),
        ColumnDef::built_in("visibility",      "Visibility",    ColumnType::EnumField),
        ColumnDef::built_in("lifecycle_stage", "Stage",         ColumnType::EnumField),
        ColumnDef::built_in("created_at",      "Created",       ColumnType::DateTimeField),
        ColumnDef::built_in("updated_at",      "Updated",       ColumnType::DateTimeField),
        ColumnDef::built_in("archived_at",     "Archived",      ColumnType::DateTimeField),
        ColumnDef::built_in("deleted_at",      "Deleted",       ColumnType::DateTimeField),

        // ── Ownership & Users ─────────────────────────────────────────────────
        ColumnDef::built_in("owners",          "Owners",        ColumnType::MultiUserField),
        ColumnDef::built_in("tags",            "Tags",          ColumnType::TagField),
        ColumnDef::built_in("policy_ids",      "Policies",      ColumnType::MultiRelationField),
        ColumnDef::built_in("toolbox_ids",     "Toolboxes",     ColumnType::MultiRelationField),

        // ── Version Control ───────────────────────────────────────────────────
        ColumnDef::built_in("version",         "Version",       ColumnType::TextField),
        ColumnDef::built_in("vector_clock",    "Vector Clock",  ColumnType::TextField),

        // ── Timeline & Schedule ───────────────────────────────────────────────
        ColumnDef::built_in("start_date",      "Start",         ColumnType::DateField),
        ColumnDef::built_in("end_date",        "End",           ColumnType::DateField),
        ColumnDef::built_in("due_date",        "Due",           ColumnType::DateField),
        ColumnDef::built_in("estimated_duration_days", "Est. Days", ColumnType::NumberField),
        ColumnDef::computed("actual_duration_days",    "Actual Days","DurationModel"),
        ColumnDef::built_in("progress_pct",    "Progress",      ColumnType::PercentField),
        ColumnDef::computed("schedule_variance_days",  "Schedule Variance","ScheduleVarianceModel"),
        ColumnDef::built_in("quarter",         "Quarter",       ColumnType::TextField),
        ColumnDef::built_in("fiscal_year",     "Fiscal Year",   ColumnType::NumberField),

        // ── Financial & Budget ────────────────────────────────────────────────
        ColumnDef::built_in("budget_allocated","Budget",        ColumnType::CurrencyField),
        ColumnDef::built_in("budget_spent",    "Spent",         ColumnType::CurrencyField),
        ColumnDef::formula_col("budget_remaining",   "Remaining",    "budget_allocated - budget_spent"),
        ColumnDef::formula_col("budget_utilization_pct","Utilization %","(budget_spent / budget_allocated) * 100"),
        ColumnDef::built_in("currency",        "Currency",      ColumnType::EnumField),
        ColumnDef::built_in("rate",            "Rate",          ColumnType::CurrencyField),
        ColumnDef::built_in("rate_type",       "Rate Type",     ColumnType::EnumField),
        ColumnDef::built_in("revenue",         "Revenue",       ColumnType::CurrencyField),
        ColumnDef::built_in("expenses",        "Expenses",      ColumnType::CurrencyField),
        ColumnDef::formula_col("profit",       "Profit",        "revenue - expenses"),
        ColumnDef::formula_col("roi",          "ROI %",         "((revenue - budget_spent) / budget_spent) * 100"),
        ColumnDef::built_in("tax_category",    "Tax Category",  ColumnType::EnumField),
        ColumnDef::built_in("invoice_refs",    "Invoices",      ColumnType::MultiRelationField),
        ColumnDef::built_in("bank_account_ref","Bank Account",  ColumnType::RelationField),

        // ── Resource Allocation ───────────────────────────────────────────────
        ColumnDef::built_in("resource_units",  "Resource Units",ColumnType::NumberField),
        ColumnDef::computed("resource_utilization_pct","Resource Util %","AllocationEngine"),

        // ── Governance & Policy ───────────────────────────────────────────────
        ColumnDef::built_in("governance_model","Governance",    ColumnType::EnumField),
        ColumnDef::built_in("approval_status", "Approval",      ColumnType::EnumField),
        ColumnDef::built_in("compliance_flags","Compliance",    ColumnType::MultiEnumField),
        ColumnDef::built_in("risk_flags",      "Risk Flags",    ColumnType::MultiEnumField),
        ColumnDef::built_in("last_reviewed_at","Last Review",   ColumnType::DateTimeField),
        ColumnDef::built_in("charter_ref",     "Charter",       ColumnType::RelationField),
        ColumnDef::built_in("regulatory_tags", "Regulatory",    ColumnType::TagField),

        // ── Analytics & Metrics ───────────────────────────────────────────────
        ColumnDef::computed("health_score",    "Health",        "PortfolioHealth"),
        ColumnDef::computed("risk_score",      "Risk",          "RiskEngine"),
        ColumnDef::computed("match_score",     "Match",         "MatchEngine"),
        ColumnDef::computed("engagement_rate", "Engagement %",  "AnalyticsEngine"),
        ColumnDef::built_in("views",           "Views",         ColumnType::AnalyticsColumn),
        ColumnDef::built_in("clicks",          "Clicks",        ColumnType::AnalyticsColumn),
        ColumnDef::formula_col("ctr",          "CTR %",         "(clicks / views) * 100"),
        ColumnDef::built_in("shares",          "Shares",        ColumnType::AnalyticsColumn),
        ColumnDef::built_in("followers",       "Followers",     ColumnType::AnalyticsColumn),
        ColumnDef::computed("velocity",        "Velocity",      "ProjectMetrics"),
        ColumnDef::computed("collaboration_score","Collab Score","CollaborationEngine"),
        ColumnDef::built_in("anomaly_flags",   "Anomalies",     ColumnType::MultiEnumField),

        // ── Relationships ─────────────────────────────────────────────────────
        ColumnDef::built_in("parent_id",       "Parent",        ColumnType::RelationField),
        ColumnDef::built_in("program_ref",     "Program",       ColumnType::RelationField),
        ColumnDef::built_in("project_ref",     "Project",       ColumnType::RelationField),

        // ── Visibility & Access ───────────────────────────────────────────────
        ColumnDef::built_in("description",     "Description",   ColumnType::RichTextField),
    ]
}

// =============================================================================
// §12 — Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn uid() -> Uuid { Uuid::new_v4() }

    #[test]
    fn test_cell_value_decimal() {
        let cv = CellValue::Currency { amount: Decimal::from(5000), currency: "USD".into() };
        assert_eq!(cv.as_decimal(), Some(Decimal::from(5000)));
        assert!(CellValue::Null.as_decimal().is_none());
    }

    #[test]
    fn test_cell_store_read_write() {
        let mut store = CellStore::new();
        let row = uid();
        let writer = uid();
        store.write(row, "name", CellValue::Text("Alpha".into()), writer);
        assert_eq!(store.read(&row, &"name".into()), CellValue::Text("Alpha".into()));
        assert_eq!(store.read(&row, &"missing".into()), CellValue::Null);
    }

    #[test]
    fn test_computed_cell_freshness() {
        let mut store = CellStore::new();
        let row = uid();
        let cv = ComputedCellValue {
            value: Box::new(CellValue::Percent(90.0)),
            formula: None,
            computed_by: "PortfolioHealth".into(),
            computed_at: Utc::now(),
            ttl_secs: Some(3600),
            stale: false,
        };
        store.write_computed(row, "health_score", cv);
        match store.read(&row, &"health_score".into()) {
            CellValue::Percent(p) => assert!((p - 90.0).abs() < 0.01),
            other                 => panic!("unexpected: {other:?}"),
        }
        store.invalidate_cache(&row);
        assert_eq!(store.read(&row, &"health_score".into()), CellValue::Null);
    }

    #[test]
    fn test_column_stats() {
        let mut store = CellStore::new();
        let w = uid();
        let rows: Vec<Uuid> = (0..5).map(|_| uid()).collect();
        for (i, &r) in rows.iter().enumerate() {
            store.write(r, "budget_spent",
                CellValue::Currency { amount: Decimal::from((i + 1) * 1000), currency: "USD".into() },
                w,
            );
        }
        let stats = store.column_stats(&rows, &"budget_spent".into());
        assert_eq!(stats.count, 5);
        assert_eq!(stats.sum, Decimal::from(15_000));
        assert_eq!(stats.min, Decimal::from(1_000));
        assert_eq!(stats.max, Decimal::from(5_000));
    }

    #[test]
    fn test_workbook_builtin_sheets() {
        let wb = SpreadsheetWorkbook::new(uid());
        assert!(wb.sheets.len() >= 30);
        assert!(wb.sheets.get(&"SHT-001".into()).is_some());
        assert!(wb.sheets.get(&"SHT-015".into()).is_some()); // Portable Benefits
    }

    #[test]
    fn test_sheet_registry_cant_remove_builtin() {
        let mut reg = SheetRegistry::new();
        reg.register(SheetDefinition::builtin("SHT-001","Master","All",vec![],ColumnSchema::empty()));
        assert!(reg.remove(&"SHT-001".into()).is_err());
    }

    #[test]
    fn test_custom_column_lifecycle() {
        let mut schema = ColumnSchema::new(universal_column_set());
        let n = schema.columns.len();
        let custom = ColumnDef { id: "custom_rate".into(), is_custom: true,
            scope: ColumnScope::Sheet("SHT-011".into()),
            ..ColumnDef::built_in("custom_rate","Custom Rate",ColumnType::CurrencyField) };
        schema.add_column(custom).unwrap();
        assert_eq!(schema.columns.len(), n + 1);
        // Duplicate
        let dup = ColumnDef::built_in("custom_rate","Dup",ColumnType::CurrencyField);
        assert!(schema.add_column(dup).is_err());
        // Remove
        schema.remove_column(&"custom_rate".into()).unwrap();
        assert_eq!(schema.columns.len(), n);
        // Can't remove built-in
        assert!(schema.remove_column(&"name".into()).is_err());
    }

    #[test]
    fn test_collection_and_list() {
        let owner = uid();
        let mut coll = Collection::new("Q2 Initiatives", owner);
        let ids: Vec<_> = (0..3).map(|_| uid()).collect();
        for &id in &ids { coll.add(id); }
        assert_eq!(coll.len(), 3);
        coll.remove(&ids[0]);
        assert!(!coll.contains(&ids[0]));

        let mut list = List::new("Backlog", owner);
        for &id in &ids { list.push(id); }
        list.move_to(ids[2], 0);
        assert_eq!(list.items[0], ids[2]);
    }

    #[test]
    fn test_directory_operations() {
        let mut dir = Directory::new("Portfolio Tree");
        let p = uid();
        dir.insert("/programs/q2/api", p);
        assert_eq!(dir.get("/programs/q2/api"), Some(&p));
        assert_eq!(dir.path_of(&p), Some("/programs/q2/api"));
        let children = dir.children("/programs/q2");
        assert_eq!(children.len(), 1);
        dir.remove(&p);
        assert!(dir.get("/programs/q2/api").is_none());
    }

    #[test]
    fn test_materialise_row() {
        let owner = uid();
        let mut wb = SpreadsheetWorkbook::new(owner);
        let row_id = uid();

        wb.write_cell(row_id, "name",           CellValue::Text("API Platform".into()), owner);
        wb.write_cell(row_id, "component_type", CellValue::Enum("item".into()),         owner);
        wb.write_cell(row_id, "type_name",      CellValue::Enum("Project".into()),      owner);
        wb.write_cell(row_id, "status",         CellValue::Enum("Active".into()),       owner);

        let schema = ColumnSchema::new(universal_column_set());
        let row = wb.materialise_row(row_id, &schema);
        assert_eq!(row.name, "API Platform");
        assert_eq!(row.type_name, "Project");
        assert_eq!(row.get(&"status".into()), &CellValue::Enum("Active".into()));
    }
}

// =============================================================================
//  portfolio_spreadsheet.rs — Kogi OS · Portfolio Spreadsheet Substrate (KPSS)
//  Independent Worker Operating System
//
//  KPSS is the low-level spreadsheet data engine that underpins the entire
//  Portfolio Management System.  Every PortfolioComponent is a PortfolioRow.
//  Every field is a typed, versioned CellValue stored in a CellStore.
//  Every named "view" is a SheetDefinition over the universal PortfolioRow schema.
//
//  SDD §§ implemented here:
//    §2  — CellValue, ComputedCellValue
//    §3  — ColumnType taxonomy (§7.1 of SDD)
//    §4  — ColumnDef, ColumnSchema
//    §5  — PortfolioRow
//    §6  — CellStore (indexed, versioned, TTL-aware)
//    §7  — SheetDefinition, SheetRegistry
//    §8  — SpreadsheetWorkbook (root container; §1.1 metaphor)
//    §9  — Structural Primitives: Group, Collection, List, Schedule, Directory
//    §10 — Built-in sheet constants (SHT-001 … SHT-030)
//    §11 — Universal column set
//
//  @version  2.2.0
//  @license  MIT
// =============================================================================

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::fmt;

use chrono::{DateTime, Duration, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Cross-module opaque IDs ───────────────────────────────────────────────────
pub type ComponentId  = Uuid;
pub type EntityId     = Uuid;
pub type PolicyId     = Uuid;
pub type AccountId    = Uuid;
pub type InvoiceId    = Uuid;
pub type FileId       = Uuid;
pub type SprintId     = Uuid;
pub type ColumnId     = String;
pub type SheetId      = String;
pub type ViewId       = Uuid;
pub type CurrencyCode = String;
pub type HexColor     = String;
pub type Slug         = String;

// =============================================================================
// §2 — CellValue
// =============================================================================

/// Every cell in the portfolio spreadsheet holds a strongly-typed `CellValue`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CellValue {
    Text(String),
    Number(Decimal),
    Currency { amount: Decimal, currency: CurrencyCode },
    Percent(f32),
    Bool(bool),
    Url(String),
    RichText(String),
    Date(NaiveDate),
    DateTime(DateTime<Utc>),
    DurationSecs(i64),
    Enum(String),
    MultiEnum(Vec<String>),
    Relation(ComponentId),
    MultiRelation(Vec<ComponentId>),
    User(EntityId),
    MultiUser(Vec<EntityId>),
    Tags(Vec<String>),
    FileRef(FileId),
    AccountRef(AccountId),
    Computed(Box<ComputedCellValue>),
    Null,
}

impl CellValue {
    pub fn is_null(&self) -> bool { matches!(self, Self::Null) }

    pub fn as_decimal(&self) -> Option<Decimal> {
        match self {
            Self::Number(d)              => Some(*d),
            Self::Currency { amount, .. } => Some(*amount),
            Self::Percent(p)             => Decimal::try_from(*p as f64).ok(),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Text(s) | Self::Url(s) | Self::RichText(s) | Self::Enum(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

impl fmt::Display for CellValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(s) | Self::Url(s) | Self::RichText(s) | Self::Enum(s) => write!(f, "{s}"),
            Self::Number(d)                  => write!(f, "{d}"),
            Self::Currency { amount, currency } => write!(f, "{amount} {currency}"),
            Self::Percent(p)                 => write!(f, "{p:.1}%"),
            Self::Bool(b)                    => write!(f, "{b}"),
            Self::Date(d)                    => write!(f, "{d}"),
            Self::DateTime(dt)               => write!(f, "{dt}"),
            Self::DurationSecs(s)            => write!(f, "{}h {}m", s / 3600, (s % 3600) / 60),
            Self::Tags(t)                    => write!(f, "{}", t.join(", ")),
            Self::MultiEnum(v)               => write!(f, "{}", v.join(", ")),
            Self::Null                       => write!(f, "—"),
            _                                => write!(f, "<ref>"),
        }
    }
}

/// A derived or AI-generated cell value with provenance metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComputedCellValue {
    pub value:        Box<CellValue>,
    pub formula:      Option<String>,
    pub computed_by:  String,           // "engine:PortfolioHealth" | "formula" | "rollup"
    pub computed_at:  DateTime<Utc>,
    pub ttl_secs:     Option<u32>,
    pub stale:        bool,
}

impl ComputedCellValue {
    pub fn expiry(&self) -> Option<DateTime<Utc>> {
        self.ttl_secs.map(|s| self.computed_at + Duration::seconds(s as i64))
    }

    pub fn is_fresh(&self) -> bool {
        if self.stale { return false; }
        if let Some(exp) = self.expiry() {
            return Utc::now() < exp;
        }
        true
    }
}

// =============================================================================
// §3 — ColumnType  (SDD §7.1)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ColumnType {
    // Primitive
    TextField, NumberField, CurrencyField, PercentField,
    DateField, DateTimeField, DurationField,
    BoolField, UrlField, RichTextField,
    // Categorical
    EnumField, MultiEnumField,
    // Relational
    RelationField, MultiRelationField, UserField, MultiUserField,
    TagField, FileRefField,
    // Derived
    ComputedColumn, AIColumn, FormulaColumn, AuditColumn, AnalyticsColumn,
}

impl ColumnType {
    pub fn is_read_only(&self) -> bool {
        matches!(self, Self::ComputedColumn | Self::AIColumn | Self::AuditColumn | Self::AnalyticsColumn)
    }
    pub fn is_numeric(&self) -> bool {
        matches!(self, Self::NumberField | Self::CurrencyField | Self::PercentField
                     | Self::ComputedColumn | Self::AIColumn | Self::FormulaColumn | Self::AnalyticsColumn)
    }
    pub fn is_sortable(&self) -> bool {
        !matches!(self, Self::RichTextField | Self::MultiRelationField | Self::MultiUserField)
    }
    pub fn is_searchable(&self) -> bool {
        matches!(self, Self::TextField | Self::RichTextField | Self::TagField | Self::UrlField)
    }
}

// =============================================================================
// §4 — ColumnDef and ColumnSchema
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnVisibility { Public, Trusted, Private }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnWidth { Compact, Normal, Wide, ExtraWide, Custom(u32) }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Aggregation { Sum, Average, Count, CountDistinct, Min, Max, First, Last, None }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnScope { Sheet(SheetId), Global }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumOption {
    pub value: String,
    pub label: String,
    pub color: HexColor,
    pub icon:  Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnValidation {
    pub required:  bool,
    pub min:       Option<Decimal>,
    pub max:       Option<Decimal>,
    pub regex:     Option<String>,
    pub enum_set:  Vec<String>,
}

/// Full column definition — the schema entry for one column.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDef {
    pub id:               ColumnId,
    pub display_name:     String,
    pub column_type:      ColumnType,
    pub description:      Option<String>,
    pub width:            ColumnWidth,
    pub pinned:           bool,
    pub hidden:           bool,
    pub visibility:       ColumnVisibility,
    pub aggregation:      Aggregation,
    pub enum_options:     Vec<EnumOption>,
    pub relation_targets: Vec<String>,
    pub formula:          Option<String>,
    pub compute_model:    Option<String>,
    pub is_custom:        bool,
    pub scope:            ColumnScope,
    pub default_value:    Option<CellValue>,
    pub validation:       Option<ColumnValidation>,
    pub created_by:       Option<EntityId>,
    pub created_at:       DateTime<Utc>,
}

impl ColumnDef {
    pub fn built_in(id: impl Into<ColumnId>, name: impl Into<String>, col_type: ColumnType) -> Self {
        Self {
            id: id.into(), display_name: name.into(), column_type: col_type,
            description: None, width: ColumnWidth::Normal, pinned: false, hidden: false,
            visibility: ColumnVisibility::Public, aggregation: Aggregation::None,
            enum_options: vec![], relation_targets: vec![], formula: None,
            compute_model: None, is_custom: false, scope: ColumnScope::Global,
            default_value: None, validation: None, created_by: None, created_at: Utc::now(),
        }
    }

    pub fn computed(id: impl Into<ColumnId>, name: impl Into<String>, model: impl Into<String>) -> Self {
        let mut col = Self::built_in(id, name, ColumnType::ComputedColumn);
        col.compute_model = Some(model.into());
        col
    }

    pub fn formula_col(id: impl Into<ColumnId>, name: impl Into<String>, formula: impl Into<String>) -> Self {
        let mut col = Self::built_in(id, name, ColumnType::FormulaColumn);
        col.formula = Some(formula.into());
        col
    }

    pub fn ai_col(id: impl Into<ColumnId>, name: impl Into<String>, model: impl Into<String>) -> Self {
        let mut col = Self::built_in(id, name, ColumnType::AIColumn);
        col.compute_model = Some(model.into());
        col
    }
}

/// An ordered set of column definitions for one sheet's schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnSchema {
    pub columns: Vec<ColumnDef>,
}

impl ColumnSchema {
    pub fn new(columns: Vec<ColumnDef>) -> Self { Self { columns } }
    pub fn empty() -> Self { Self { columns: vec![] } }

    pub fn get(&self, id: &ColumnId) -> Option<&ColumnDef> {
        self.columns.iter().find(|c| &c.id == id)
    }

    pub fn visible(&self) -> Vec<&ColumnDef> {
        let mut out: Vec<_> = self.columns.iter().filter(|c| c.pinned && !c.hidden).collect();
        out.extend(self.columns.iter().filter(|c| !c.pinned && !c.hidden));
        out
    }

    pub fn add_column(&mut self, col: ColumnDef) -> Result<(), String> {
        if self.columns.iter().any(|c| c.id == col.id) {
            return Err(format!("Column '{}' already exists", col.id));
        }
        self.columns.push(col);
        Ok(())
    }

    pub fn remove_column(&mut self, id: &ColumnId) -> Result<(), String> {
        let pos = self.columns.iter().position(|c| &c.id == id)
            .ok_or_else(|| format!("Column '{id}' not found"))?;
        if !self.columns[pos].is_custom {
            return Err(format!("Column '{id}' is built-in and cannot be removed"));
        }
        self.columns.remove(pos);
        Ok(())
    }

    pub fn reorder(&mut self, id: &ColumnId, new_index: usize) -> Result<(), String> {
        let pos = self.columns.iter().position(|c| &c.id == id)
            .ok_or_else(|| format!("Column '{id}' not found"))?;
        let col = self.columns.remove(pos);
        let target = new_index.min(self.columns.len());
        self.columns.insert(target, col);
        Ok(())
    }

    pub fn pinned_ids(&self) -> Vec<&ColumnId> {
        self.columns.iter().filter(|c| c.pinned).map(|c| &c.id).collect()
    }

    pub fn numeric_columns(&self) -> Vec<&ColumnDef> {
        self.columns.iter().filter(|c| c.column_type.is_numeric()).collect()
    }
}

// =============================================================================
// §5 — PortfolioRow
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RowHeight { Compact, Normal, Tall, Auto }

/// One row in the master spreadsheet — a typed, navigable snapshot of a component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioRow {
    pub row_id:         ComponentId,
    pub slug:           Option<Slug>,
    pub component_type: String,      // "item" | "container"
    pub type_name:      String,      // "Project" | "Asset" | "Binder" | …
    pub name:           String,
    pub cells:          HashMap<ColumnId, CellValue>,
    pub depth:          u32,
    pub parent_id:      Option<ComponentId>,
    pub child_ids:      Vec<ComponentId>,
    pub color:          Option<HexColor>,
    pub icon:           Option<String>,
    pub height:         RowHeight,
    pub annotation:     Option<String>,  // Oba inline annotation
    pub highlight:      Option<HexColor>,// from RowHighlightRule
}

impl PortfolioRow {
    pub fn new(
        row_id: ComponentId,
        component_type: impl Into<String>,
        type_name: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            row_id, slug: None,
            component_type: component_type.into(),
            type_name: type_name.into(),
            name: name.into(),
            cells: HashMap::new(),
            depth: 0, parent_id: None, child_ids: vec![],
            color: None, icon: None, height: RowHeight::Normal,
            annotation: None, highlight: None,
        }
    }

    pub fn get(&self, col: &ColumnId) -> &CellValue {
        self.cells.get(col).unwrap_or(&CellValue::Null)
    }

    pub fn set(&mut self, col: impl Into<ColumnId>, val: CellValue) {
        self.cells.insert(col.into(), val);
    }

    pub fn get_decimal(&self, col: &ColumnId) -> Option<Decimal> {
        self.cells.get(col)?.as_decimal()
    }

    pub fn get_str(&self, col: &ColumnId) -> Option<&str> {
        self.cells.get(col)?.as_str()
    }
}

// =============================================================================
// §6 — CellStore
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellEntry {
    pub row_id:     ComponentId,
    pub column_id:  ColumnId,
    pub value:      CellValue,
    pub version:    u64,
    pub written_by: EntityId,
    pub written_at: DateTime<Utc>,
}

/// Aggregate statistics for a numeric column across a result set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnStats {
    pub column_id:  ColumnId,
    pub sum:        Decimal,
    pub average:    Decimal,
    pub min:        Decimal,
    pub max:        Decimal,
    pub count:      usize,
    pub null_count: usize,
}

/// Central cell storage.  Primary key: (ComponentId, ColumnId).
/// Computed values cached separately to allow TTL management.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CellStore {
    /// User-written values: row → column → entry
    entries: HashMap<ComponentId, HashMap<ColumnId, CellEntry>>,
    /// Computed/cached values: row → column → computed
    cache:   HashMap<ComponentId, HashMap<ColumnId, ComputedCellValue>>,
    /// Monotonic version counter
    version: u64,
}

impl CellStore {
    pub fn new() -> Self { Self::default() }

    pub fn write(
        &mut self,
        row_id: ComponentId,
        column_id: impl Into<ColumnId>,
        value: CellValue,
        writer: EntityId,
    ) -> u64 {
        self.version += 1;
        let v = self.version;
        self.entries.entry(row_id).or_default().insert(
            column_id.into(),
            CellEntry { row_id, column_id: String::new(), value, version: v, written_by: writer, written_at: Utc::now() },
        );
        v
    }

    pub fn write_computed(
        &mut self,
        row_id: ComponentId,
        column_id: impl Into<ColumnId>,
        computed: ComputedCellValue,
    ) {
        self.cache.entry(row_id).or_default().insert(column_id.into(), computed);
    }

    pub fn read(&self, row_id: &ComponentId, column_id: &ColumnId) -> CellValue {
        // 1. Fresh computed value
        if let Some(cv) = self.cache.get(row_id).and_then(|m| m.get(column_id)) {
            if cv.is_fresh() {
                return *cv.value.clone();
            }
        }
        // 2. User-written value
        self.entries
            .get(row_id)
            .and_then(|cols| cols.get(column_id))
            .map(|e| e.value.clone())
            .unwrap_or(CellValue::Null)
    }

    /// Mark all cached computed values for a row as stale.
    pub fn invalidate_cache(&mut self, row_id: &ComponentId) {
        if let Some(m) = self.cache.get_mut(row_id) {
            for cv in m.values_mut() { cv.stale = true; }
        }
    }

    pub fn column_stats(&self, row_ids: &[ComponentId], column_id: &ColumnId) -> ColumnStats {
        let mut sum = Decimal::ZERO;
        let mut min: Option<Decimal> = None;
        let mut max: Option<Decimal> = None;
        let mut count = 0usize;
        let mut null_count = 0usize;

        for rid in row_ids {
            if let Some(d) = self.read(rid, column_id).as_decimal() {
                sum += d;
                count += 1;
                min = Some(min.map_or(d, |m: Decimal| m.min(d)));
                max = Some(max.map_or(d, |m: Decimal| m.max(d)));
            } else {
                null_count += 1;
            }
        }

        ColumnStats {
            column_id: column_id.clone(),
            average: if count > 0 { sum / Decimal::from(count) } else { Decimal::ZERO },
            sum,
            min: min.unwrap_or(Decimal::ZERO),
            max: max.unwrap_or(Decimal::ZERO),
            count,
            null_count,
        }
    }

    pub fn all_row_ids(&self) -> impl Iterator<Item = &ComponentId> {
        self.entries.keys()
    }

    pub fn row_entry_count(&self, row_id: &ComponentId) -> usize {
        self.entries.get(row_id).map(|m| m.len()).unwrap_or(0)
    }
}

// =============================================================================
// §7 — SheetDefinition and SheetRegistry
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetRowTypeFilter {
    pub include_types:    Vec<String>,
    pub exclude_statuses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetDefaultGroup {
    pub column_id: ColumnId,
    pub secondary: Option<ColumnId>,
}

/// Persistent specification for a named sheet (SHT-NNN).
/// A sheet is a view recipe, not a query result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetDefinition {
    pub sheet_id:        SheetId,
    pub name:            String,
    pub description:     String,
    pub row_types:       SheetRowTypeFilter,
    pub schema:          ColumnSchema,
    pub default_group:   Option<SheetDefaultGroup>,
    pub pinned_columns:  Vec<ColumnId>,
    pub is_builtin:      bool,
    pub is_visible:      bool,
    pub computed_models: Vec<String>,
    pub created_by:      Option<EntityId>,
    pub created_at:      DateTime<Utc>,
    pub updated_at:      DateTime<Utc>,
}

impl SheetDefinition {
    pub fn builtin(
        sheet_id: impl Into<SheetId>,
        name: impl Into<String>,
        description: impl Into<String>,
        include_types: Vec<String>,
        schema: ColumnSchema,
    ) -> Self {
        let now = Utc::now();
        Self {
            sheet_id: sheet_id.into(), name: name.into(), description: description.into(),
            row_types: SheetRowTypeFilter {
                include_types,
                exclude_statuses: vec!["Archived".into()],
            },
            schema,
            default_group: None,
            pinned_columns: vec!["component_id".into(), "name".into()],
            is_builtin: true, is_visible: true,
            computed_models: vec![],
            created_by: None, created_at: now, updated_at: now,
        }
    }
}

/// All sheets in a workbook, with preserved navigation order.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SheetRegistry {
    order:  Vec<SheetId>,
    sheets: HashMap<SheetId, SheetDefinition>,
}

impl SheetRegistry {
    pub fn new() -> Self { Self::default() }

    pub fn register(&mut self, sheet: SheetDefinition) {
        let id = sheet.sheet_id.clone();
        if !self.order.contains(&id) { self.order.push(id.clone()); }
        self.sheets.insert(id, sheet);
    }

    pub fn get(&self, id: &SheetId) -> Option<&SheetDefinition> { self.sheets.get(id) }
    pub fn get_mut(&mut self, id: &SheetId) -> Option<&mut SheetDefinition> { self.sheets.get_mut(id) }

    pub fn all(&self) -> Vec<&SheetDefinition> {
        self.order.iter().filter_map(|id| self.sheets.get(id)).collect()
    }
    pub fn visible(&self) -> Vec<&SheetDefinition> {
        self.all().into_iter().filter(|s| s.is_visible).collect()
    }

    pub fn remove(&mut self, id: &SheetId) -> Result<SheetDefinition, String> {
        let sheet = self.sheets.get(id).ok_or_else(|| format!("Sheet '{id}' not found"))?;
        if sheet.is_builtin { return Err(format!("Sheet '{id}' is built-in")); }
        self.order.retain(|s| s != id);
        Ok(self.sheets.remove(id).unwrap())
    }

    pub fn reorder(&mut self, id: &SheetId, new_index: usize) {
        self.order.retain(|s| s != id);
        let target = new_index.min(self.order.len());
        self.order.insert(target, id.clone());
    }

    pub fn len(&self) -> usize { self.sheets.len() }
}

// =============================================================================
// §8 — SpreadsheetWorkbook
// =============================================================================

/// The root workbook — one per worker/org.  The "Excel workbook" of the Kogi OS.
/// Contains all sheets, all cell data, and the row cache.
#[derive(Debug, Serialize, Deserialize)]
pub struct SpreadsheetWorkbook {
    pub owner_id:   EntityId,
    pub cells:      CellStore,
    pub sheets:     SheetRegistry,
    #[serde(skip)]
    row_cache:      HashMap<ComponentId, PortfolioRow>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SpreadsheetWorkbook {
    pub fn new(owner_id: EntityId) -> Self {
        let mut wb = Self {
            owner_id, cells: CellStore::new(), sheets: SheetRegistry::new(),
            row_cache: HashMap::new(), created_at: Utc::now(), updated_at: Utc::now(),
        };
        wb.register_builtin_sheets();
        wb
    }

    fn register_builtin_sheets(&mut self) {
        for (id, name, desc, types) in BUILTIN_SHEETS.iter() {
            let schema = ColumnSchema::new(universal_column_set());
            let sheet  = SheetDefinition::builtin(
                *id, *name, *desc,
                types.iter().map(|s| s.to_string()).collect(),
                schema,
            );
            self.sheets.register(sheet);
        }
    }

    pub fn write_cell(
        &mut self,
        row_id: ComponentId,
        column_id: impl Into<ColumnId>,
        value: CellValue,
        writer: EntityId,
    ) -> u64 {
        let v = self.cells.write(row_id, column_id, value, writer);
        self.row_cache.remove(&row_id);
        self.updated_at = Utc::now();
        v
    }

    pub fn write_computed_cell(
        &mut self,
        row_id: ComponentId,
        column_id: impl Into<ColumnId>,
        computed: ComputedCellValue,
    ) {
        self.cells.write_computed(row_id, column_id, computed);
        self.row_cache.remove(&row_id);
    }

    pub fn read_cell(&self, row_id: &ComponentId, column_id: &ColumnId) -> CellValue {
        self.cells.read(row_id, column_id)
    }

    /// Materialise a PortfolioRow from cell store, using cache when available.
    pub fn materialise_row(&mut self, row_id: ComponentId, schema: &ColumnSchema) -> PortfolioRow {
        if let Some(cached) = self.row_cache.get(&row_id) {
            return cached.clone();
        }
        let name = match self.cells.read(&row_id, &"name".into()) {
            CellValue::Text(s) => s,
            _                  => row_id.to_string(),
        };
        let component_type = match self.cells.read(&row_id, &"component_type".into()) {
            CellValue::Text(s) | CellValue::Enum(s) => s,
            _                                        => "item".into(),
        };
        let type_name = match self.cells.read(&row_id, &"type_name".into()) {
            CellValue::Text(s) | CellValue::Enum(s) => s,
            _                                        => "Unknown".into(),
        };

        let mut row = PortfolioRow::new(row_id, component_type, type_name, name);
        for col in &schema.columns {
            row.cells.insert(col.id.clone(), self.cells.read(&row_id, &col.id));
        }

        self.row_cache.insert(row_id, row.clone());
        row
    }

    pub fn invalidate_row(&mut self, row_id: &ComponentId) {
        self.row_cache.remove(row_id);
        self.cells.invalidate_cache(row_id);
    }

    pub fn column_stats(&self, row_ids: &[ComponentId], column_id: &ColumnId) -> ColumnStats {
        self.cells.column_stats(row_ids, column_id)
    }

    /// Return all row IDs that have cell data.
    pub fn all_row_ids(&self) -> Vec<ComponentId> {
        self.cells.all_row_ids().copied().collect()
    }
}

// =============================================================================
// §9 — Structural Primitives  (SDD §12)
// =============================================================================

/// Dynamic, labeled cluster of rows sharing a column value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub label:     String,
    pub column_id: ColumnId,
    pub value:     CellValue,
    pub row_ids:   Vec<ComponentId>,
    pub summary:   HashMap<ColumnId, CellValue>,
}

/// Static, named, user-defined set of ComponentIds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id:          Uuid,
    pub name:        String,
    pub description: Option<String>,
    pub owner_id:    EntityId,
    pub row_ids:     HashSet<ComponentId>,
    pub visibility:  String,
    pub created_at:  DateTime<Utc>,
}

impl Collection {
    pub fn new(name: impl Into<String>, owner: EntityId) -> Self {
        Self {
            id: Uuid::new_v4(), name: name.into(), description: None, owner_id: owner,
            row_ids: HashSet::new(), visibility: "Private".into(), created_at: Utc::now(),
        }
    }
    pub fn add(&mut self, id: ComponentId)     { self.row_ids.insert(id); }
    pub fn remove(&mut self, id: &ComponentId) { self.row_ids.remove(id); }
    pub fn contains(&self, id: &ComponentId)   -> bool { self.row_ids.contains(id) }
    pub fn len(&self)                           -> usize { self.row_ids.len() }
    pub fn is_empty(&self)                      -> bool  { self.row_ids.is_empty() }
}

/// Ordered, persistent sequence of ComponentIds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct List {
    pub id:         Uuid,
    pub name:       String,
    pub owner_id:   EntityId,
    pub items:      Vec<ComponentId>,
    pub created_at: DateTime<Utc>,
}

impl List {
    pub fn new(name: impl Into<String>, owner: EntityId) -> Self {
        Self { id: Uuid::new_v4(), name: name.into(), owner_id: owner,
               items: vec![], created_at: Utc::now() }
    }
    pub fn push(&mut self, id: ComponentId) { self.items.push(id); }
    pub fn move_to(&mut self, id: ComponentId, idx: usize) {
        self.items.retain(|&x| x != id);
        let target = idx.min(self.items.len());
        self.items.insert(target, id);
    }
    pub fn remove(&mut self, id: &ComponentId) { self.items.retain(|x| x != id); }
    pub fn len(&self)      -> usize { self.items.len() }
    pub fn is_empty(&self) -> bool  { self.items.is_empty() }
}

/// Causal sequence ordered by start_date + dependency graph sort.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub id:    Uuid,
    pub name:  String,
    pub items: Vec<ScheduleItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleItem {
    pub component_id: ComponentId,
    pub start_date:   Option<NaiveDate>,
    pub end_date:     Option<NaiveDate>,
    pub dependencies: Vec<ComponentId>,
}

/// Hierarchical path-addressed directory of components.
/// Items addressed as: /programs/q2/projects/api-redesign
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Directory {
    pub id:    Uuid,
    pub name:  String,
    paths: HashMap<String, ComponentId>,
    index: HashMap<ComponentId, String>,
}

impl Directory {
    pub fn new(name: impl Into<String>) -> Self {
        Self { id: Uuid::new_v4(), name: name.into(), paths: HashMap::new(), index: HashMap::new() }
    }
    pub fn insert(&mut self, path: impl Into<String>, id: ComponentId) {
        let p = path.into();
        self.paths.insert(p.clone(), id);
        self.index.insert(id, p);
    }
    pub fn get(&self, path: &str)           -> Option<&ComponentId> { self.paths.get(path) }
    pub fn path_of(&self, id: &ComponentId) -> Option<&str>         { self.index.get(id).map(|s| s.as_str()) }
    pub fn children(&self, prefix: &str) -> Vec<(&str, &ComponentId)> {
        self.paths.iter()
            .filter(|(p, _)| p.starts_with(prefix) && p.len() > prefix.len())
            .map(|(p, id)| (p.as_str(), id))
            .collect()
    }
    pub fn remove(&mut self, id: &ComponentId) {
        if let Some(p) = self.index.remove(id) { self.paths.remove(&p); }
    }
}

// =============================================================================
// §10 — Built-in Sheet Registry  (SDD §4.1)
// =============================================================================

/// (SheetId, Name, Description, Row Types)
pub const BUILTIN_SHEETS: &[(&str, &str, &str, &[&str])] = &[
    ("SHT-001", "Master Registry",             "All portfolio components — universal view",                      &[]),
    ("SHT-002", "Portfolio Hierarchy",         "Structural tree: Portfolio → Program → Project → Task",         &["Portfolio","SubPortfolio","Program","Project"]),
    ("SHT-003", "Programs",                    "Strategic programs and initiatives",                            &["Program"]),
    ("SHT-004", "Projects",                    "All projects across all programs",                              &["Project"]),
    ("SHT-005", "Tasks & Backlog",             "Tasks, stories, epics, and features",                          &["Task"]),
    ("SHT-006", "Resources",                   "Human, financial, and equipment resources",                     &["Resource"]),
    ("SHT-007", "Assets",                      "Capital and intellectual assets",                               &["Asset"]),
    ("SHT-008", "Artifacts",                   "Documents, files, and generated outputs",                       &["Artifact"]),
    ("SHT-009", "Finances",                    "Unified financial ledger across all portfolio entities",         &[]),
    ("SHT-010", "Budget Tracker",              "Budget allocation and spend tracking by program/project",       &["Program","Project","Gig","Contract"]),
    ("SHT-011", "Work & Gigs",                 "All income-generating engagements",                             &["Gig","Contract","Job"]),
    ("SHT-012", "Deliverables",                "Project outputs, artifact releases, and milestones",            &["Project","Artifact"]),
    ("SHT-013", "Timeline",                    "All dated components on a timeline",                            &[]),
    ("SHT-014", "Roadmap",                     "Programs and projects on a quarter roadmap",                    &["Program","Project"]),
    ("SHT-015", "Portable Benefits",           "All portable benefit accounts (HSA, Retirement, PTO, …)",       &["BenefitAccount"]),
    ("SHT-016", "Grants & Microfinancing",     "Grant applications, microloans, and community lending",         &["Grant"]),
    ("SHT-017", "Equity Crowdfunding",         "Equity crowdfunding campaigns and investment instruments",       &["Campaign","Investment"]),
    ("SHT-018", "Contracts",                   "Formal agreements, NDAs, and SOW contracts",                    &["Contract"]),
    ("SHT-019", "Jobs & Consulting",           "Longer-term employment and consulting engagements",             &["Job"]),
    ("SHT-020", "Shared Portfolios",           "Co-owned, cooperative, and crowdresourced portfolios",          &["Portfolio","Program","Project"]),
    ("SHT-021", "Collaboration",               "Contribution-centric view of all shared components",            &[]),
    ("SHT-022", "Governance",                  "Policies, approval queues, compliance flags, and risk flags",   &[]),
    ("SHT-023", "Event Log",                   "Append-only audit trail of all portfolio mutations",            &[]),
    ("SHT-024", "Snapshots",                   "Point-in-time portfolio snapshots and restore points",          &[]),
    ("SHT-025", "Contacts",                    "People and organization profiles in the worker's network",      &["Profile"]),
    ("SHT-026", "Marketplace Listings",        "Assets and resources published to kogi-marketplace",            &["Asset","Resource"]),
    ("SHT-027", "Deals",                       "Active deal proposals and exchange transactions",               &[]),
    ("SHT-028", "Archive",                     "Deep-storage of completed and retired components",              &[]),
    ("SHT-029", "Analytics",                   "Engagement metrics, performance KPIs, and usage analytics",     &[]),
    ("SHT-030", "Engine Signals",              "AI-derived scores, recommendations, and anomaly flags",         &[]),
];

// =============================================================================
// §11 — Universal column set (the baseline every sheet inherits)
// =============================================================================

pub fn universal_column_set() -> Vec<ColumnDef> {
    vec![
        // ── Identity ──────────────────────────────────────────────────────────
        ColumnDef::built_in("component_id",    "ID",            ColumnType::TextField),
        ColumnDef::built_in("name",            "Name",          ColumnType::TextField),
        ColumnDef::built_in("slug",            "Slug",          ColumnType::TextField),
        ColumnDef::built_in("component_type",  "Type",          ColumnType::EnumField),
        ColumnDef::built_in("type_name",       "Kind",          ColumnType::EnumField),
        ColumnDef::built_in("display_name",    "Display Name",  ColumnType::TextField),
        ColumnDef::built_in("icon",            "Icon",          ColumnType::TextField),
        ColumnDef::built_in("color",           "Color",         ColumnType::TextField),

        // ── Lifecycle ─────────────────────────────────────────────────────────
        ColumnDef::built_in("status",          "Status",        ColumnType::EnumField),
        ColumnDef::built_in("state",           "State",         ColumnType::EnumField),
        ColumnDef::built_in("visibility",      "Visibility",    ColumnType::EnumField),
        ColumnDef::built_in("lifecycle_stage", "Stage",         ColumnType::EnumField),
        ColumnDef::built_in("created_at",      "Created",       ColumnType::DateTimeField),
        ColumnDef::built_in("updated_at",      "Updated",       ColumnType::DateTimeField),
        ColumnDef::built_in("archived_at",     "Archived",      ColumnType::DateTimeField),
        ColumnDef::built_in("deleted_at",      "Deleted",       ColumnType::DateTimeField),

        // ── Ownership & Users ─────────────────────────────────────────────────
        ColumnDef::built_in("owners",          "Owners",        ColumnType::MultiUserField),
        ColumnDef::built_in("tags",            "Tags",          ColumnType::TagField),
        ColumnDef::built_in("policy_ids",      "Policies",      ColumnType::MultiRelationField),
        ColumnDef::built_in("toolbox_ids",     "Toolboxes",     ColumnType::MultiRelationField),

        // ── Version Control ───────────────────────────────────────────────────
        ColumnDef::built_in("version",         "Version",       ColumnType::TextField),
        ColumnDef::built_in("vector_clock",    "Vector Clock",  ColumnType::TextField),

        // ── Timeline & Schedule ───────────────────────────────────────────────
        ColumnDef::built_in("start_date",      "Start",         ColumnType::DateField),
        ColumnDef::built_in("end_date",        "End",           ColumnType::DateField),
        ColumnDef::built_in("due_date",        "Due",           ColumnType::DateField),
        ColumnDef::built_in("estimated_duration_days", "Est. Days", ColumnType::NumberField),
        ColumnDef::computed("actual_duration_days",    "Actual Days","DurationModel"),
        ColumnDef::built_in("progress_pct",    "Progress",      ColumnType::PercentField),
        ColumnDef::computed("schedule_variance_days",  "Schedule Variance","ScheduleVarianceModel"),
        ColumnDef::built_in("quarter",         "Quarter",       ColumnType::TextField),
        ColumnDef::built_in("fiscal_year",     "Fiscal Year",   ColumnType::NumberField),

        // ── Financial & Budget ────────────────────────────────────────────────
        ColumnDef::built_in("budget_allocated","Budget",        ColumnType::CurrencyField),
        ColumnDef::built_in("budget_spent",    "Spent",         ColumnType::CurrencyField),
        ColumnDef::formula_col("budget_remaining",   "Remaining",    "budget_allocated - budget_spent"),
        ColumnDef::formula_col("budget_utilization_pct","Utilization %","(budget_spent / budget_allocated) * 100"),
        ColumnDef::built_in("currency",        "Currency",      ColumnType::EnumField),
        ColumnDef::built_in("rate",            "Rate",          ColumnType::CurrencyField),
        ColumnDef::built_in("rate_type",       "Rate Type",     ColumnType::EnumField),
        ColumnDef::built_in("revenue",         "Revenue",       ColumnType::CurrencyField),
        ColumnDef::built_in("expenses",        "Expenses",      ColumnType::CurrencyField),
        ColumnDef::formula_col("profit",       "Profit",        "revenue - expenses"),
        ColumnDef::formula_col("roi",          "ROI %",         "((revenue - budget_spent) / budget_spent) * 100"),
        ColumnDef::built_in("tax_category",    "Tax Category",  ColumnType::EnumField),
        ColumnDef::built_in("invoice_refs",    "Invoices",      ColumnType::MultiRelationField),
        ColumnDef::built_in("bank_account_ref","Bank Account",  ColumnType::RelationField),

        // ── Resource Allocation ───────────────────────────────────────────────
        ColumnDef::built_in("resource_units",  "Resource Units",ColumnType::NumberField),
        ColumnDef::computed("resource_utilization_pct","Resource Util %","AllocationEngine"),

        // ── Governance & Policy ───────────────────────────────────────────────
        ColumnDef::built_in("governance_model","Governance",    ColumnType::EnumField),
        ColumnDef::built_in("approval_status", "Approval",      ColumnType::EnumField),
        ColumnDef::built_in("compliance_flags","Compliance",    ColumnType::MultiEnumField),
        ColumnDef::built_in("risk_flags",      "Risk Flags",    ColumnType::MultiEnumField),
        ColumnDef::built_in("last_reviewed_at","Last Review",   ColumnType::DateTimeField),
        ColumnDef::built_in("charter_ref",     "Charter",       ColumnType::RelationField),
        ColumnDef::built_in("regulatory_tags", "Regulatory",    ColumnType::TagField),

        // ── Analytics & Metrics ───────────────────────────────────────────────
        ColumnDef::computed("health_score",    "Health",        "PortfolioHealth"),
        ColumnDef::computed("risk_score",      "Risk",          "RiskEngine"),
        ColumnDef::computed("match_score",     "Match",         "MatchEngine"),
        ColumnDef::computed("engagement_rate", "Engagement %",  "AnalyticsEngine"),
        ColumnDef::built_in("views",           "Views",         ColumnType::AnalyticsColumn),
        ColumnDef::built_in("clicks",          "Clicks",        ColumnType::AnalyticsColumn),
        ColumnDef::formula_col("ctr",          "CTR %",         "(clicks / views) * 100"),
        ColumnDef::built_in("shares",          "Shares",        ColumnType::AnalyticsColumn),
        ColumnDef::built_in("followers",       "Followers",     ColumnType::AnalyticsColumn),
        ColumnDef::computed("velocity",        "Velocity",      "ProjectMetrics"),
        ColumnDef::computed("collaboration_score","Collab Score","CollaborationEngine"),
        ColumnDef::built_in("anomaly_flags",   "Anomalies",     ColumnType::MultiEnumField),

        // ── Relationships ─────────────────────────────────────────────────────
        ColumnDef::built_in("parent_id",       "Parent",        ColumnType::RelationField),
        ColumnDef::built_in("program_ref",     "Program",       ColumnType::RelationField),
        ColumnDef::built_in("project_ref",     "Project",       ColumnType::RelationField),

        // ── Visibility & Access ───────────────────────────────────────────────
        ColumnDef::built_in("description",     "Description",   ColumnType::RichTextField),
    ]
}

// =============================================================================
// §12 — Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn uid() -> Uuid { Uuid::new_v4() }

    #[test]
    fn test_cell_value_decimal() {
        let cv = CellValue::Currency { amount: Decimal::from(5000), currency: "USD".into() };
        assert_eq!(cv.as_decimal(), Some(Decimal::from(5000)));
        assert!(CellValue::Null.as_decimal().is_none());
    }

    #[test]
    fn test_cell_store_read_write() {
        let mut store = CellStore::new();
        let row = uid();
        let writer = uid();
        store.write(row, "name", CellValue::Text("Alpha".into()), writer);
        assert_eq!(store.read(&row, &"name".into()), CellValue::Text("Alpha".into()));
        assert_eq!(store.read(&row, &"missing".into()), CellValue::Null);
    }

    #[test]
    fn test_computed_cell_freshness() {
        let mut store = CellStore::new();
        let row = uid();
        let cv = ComputedCellValue {
            value: Box::new(CellValue::Percent(90.0)),
            formula: None,
            computed_by: "PortfolioHealth".into(),
            computed_at: Utc::now(),
            ttl_secs: Some(3600),
            stale: false,
        };
        store.write_computed(row, "health_score", cv);
        match store.read(&row, &"health_score".into()) {
            CellValue::Percent(p) => assert!((p - 90.0).abs() < 0.01),
            other                 => panic!("unexpected: {other:?}"),
        }
        store.invalidate_cache(&row);
        assert_eq!(store.read(&row, &"health_score".into()), CellValue::Null);
    }

    #[test]
    fn test_column_stats() {
        let mut store = CellStore::new();
        let w = uid();
        let rows: Vec<Uuid> = (0..5).map(|_| uid()).collect();
        for (i, &r) in rows.iter().enumerate() {
            store.write(r, "budget_spent",
                CellValue::Currency { amount: Decimal::from((i + 1) * 1000), currency: "USD".into() },
                w,
            );
        }
        let stats = store.column_stats(&rows, &"budget_spent".into());
        assert_eq!(stats.count, 5);
        assert_eq!(stats.sum, Decimal::from(15_000));
        assert_eq!(stats.min, Decimal::from(1_000));
        assert_eq!(stats.max, Decimal::from(5_000));
    }

    #[test]
    fn test_workbook_builtin_sheets() {
        let wb = SpreadsheetWorkbook::new(uid());
        assert!(wb.sheets.len() >= 30);
        assert!(wb.sheets.get(&"SHT-001".into()).is_some());
        assert!(wb.sheets.get(&"SHT-015".into()).is_some()); // Portable Benefits
    }

    #[test]
    fn test_sheet_registry_cant_remove_builtin() {
        let mut reg = SheetRegistry::new();
        reg.register(SheetDefinition::builtin("SHT-001","Master","All",vec![],ColumnSchema::empty()));
        assert!(reg.remove(&"SHT-001".into()).is_err());
    }

    #[test]
    fn test_custom_column_lifecycle() {
        let mut schema = ColumnSchema::new(universal_column_set());
        let n = schema.columns.len();
        let custom = ColumnDef { id: "custom_rate".into(), is_custom: true,
            scope: ColumnScope::Sheet("SHT-011".into()),
            ..ColumnDef::built_in("custom_rate","Custom Rate",ColumnType::CurrencyField) };
        schema.add_column(custom).unwrap();
        assert_eq!(schema.columns.len(), n + 1);
        // Duplicate
        let dup = ColumnDef::built_in("custom_rate","Dup",ColumnType::CurrencyField);
        assert!(schema.add_column(dup).is_err());
        // Remove
        schema.remove_column(&"custom_rate".into()).unwrap();
        assert_eq!(schema.columns.len(), n);
        // Can't remove built-in
        assert!(schema.remove_column(&"name".into()).is_err());
    }

    #[test]
    fn test_collection_and_list() {
        let owner = uid();
        let mut coll = Collection::new("Q2 Initiatives", owner);
        let ids: Vec<_> = (0..3).map(|_| uid()).collect();
        for &id in &ids { coll.add(id); }
        assert_eq!(coll.len(), 3);
        coll.remove(&ids[0]);
        assert!(!coll.contains(&ids[0]));

        let mut list = List::new("Backlog", owner);
        for &id in &ids { list.push(id); }
        list.move_to(ids[2], 0);
        assert_eq!(list.items[0], ids[2]);
    }

    #[test]
    fn test_directory_operations() {
        let mut dir = Directory::new("Portfolio Tree");
        let p = uid();
        dir.insert("/programs/q2/api", p);
        assert_eq!(dir.get("/programs/q2/api"), Some(&p));
        assert_eq!(dir.path_of(&p), Some("/programs/q2/api"));
        let children = dir.children("/programs/q2");
        assert_eq!(children.len(), 1);
        dir.remove(&p);
        assert!(dir.get("/programs/q2/api").is_none());
    }

    #[test]
    fn test_materialise_row() {
        let owner = uid();
        let mut wb = SpreadsheetWorkbook::new(owner);
        let row_id = uid();

        wb.write_cell(row_id, "name",           CellValue::Text("API Platform".into()), owner);
        wb.write_cell(row_id, "component_type", CellValue::Enum("item".into()),         owner);
        wb.write_cell(row_id, "type_name",      CellValue::Enum("Project".into()),      owner);
        wb.write_cell(row_id, "status",         CellValue::Enum("Active".into()),       owner);

        let schema = ColumnSchema::new(universal_column_set());
        let row = wb.materialise_row(row_id, &schema);
        assert_eq!(row.name, "API Platform");
        assert_eq!(row.type_name, "Project");
        assert_eq!(row.get(&"status".into()), &CellValue::Enum("Active".into()));
    }
}

// =============================================================================
//  portfolio_view_engine.rs — Kogi OS · KPVW v3.0
//  Portfolio View Engine
//
//  Transforms raw PortfolioRow data into shaped, filtered, sorted, grouped,
//  and visually enriched spreadsheet views. Views are defined as
//  ViewDefinition objects, serialized to JSON, versioned, and shareable.
//
//  Key types
//  ─────────
//    ViewDefinition     — complete description of a user-facing view
//    ViewFilter         — composable boolean predicate (AND/OR tree)
//    FilterPredicate    — a single filter clause (operator + value)
//    ViewSort           — ordered sort predicate
//    ViewGroup          — hierarchical group definition
//    PivotConfig        — cross-dimensional aggregation
//    ChartConfig        — chart overlay configuration
//    BoardConfig        — Kanban / Gantt / Calendar board mode settings
//    RowHighlightRule   — conditional row coloring
//    SummaryRowConfig   — aggregate row at sheet bottom
//    ViewEngine         — applies transforms to a PortfolioRow slice
//
//  @author  Kogi Team
//  @version 3.0.0
//  @license MIT
// =============================================================================

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::portfolio_spreadsheet_substrate::{
    CellStore, CellValue, ColumnId, ComponentId, EntityId, PortfolioRow, RowStore,
};

// =============================================================================
// §1 — FILTER SYSTEM
// =============================================================================

/// A single atomic filter clause.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FilterPredicate {
    Equals          { column: ColumnId, value: serde_json::Value },
    NotEquals       { column: ColumnId, value: serde_json::Value },
    GreaterThan     { column: ColumnId, value: f64 },
    LessThan        { column: ColumnId, value: f64 },
    GreaterOrEqual  { column: ColumnId, value: f64 },
    LessOrEqual     { column: ColumnId, value: f64 },
    Between         { column: ColumnId, min: f64, max: f64 },
    Contains        { column: ColumnId, substring: String },
    StartsWith      { column: ColumnId, prefix: String },
    EndsWith        { column: ColumnId, suffix: String },
    DateAfter       { column: ColumnId, date: NaiveDate },
    DateBefore      { column: ColumnId, date: NaiveDate },
    DateBetween     { column: ColumnId, start: NaiveDate, end: NaiveDate },
    InSet           { column: ColumnId, values: Vec<serde_json::Value> },
    NotInSet        { column: ColumnId, values: Vec<serde_json::Value> },
    TagIncludes     { column: ColumnId, tag: String },
    HasRiskSeverity { severity: String },
    HasComplianceFlag { flag: String },
    IsEmpty         { column: ColumnId },
    IsNotEmpty      { column: ColumnId },
    IsTrue          { column: ColumnId },
    IsFalse         { column: ColumnId },
    OwnedByMe,
    HealthScoreAbove { threshold: f64 },
    HealthScoreBelow { threshold: f64 },
    RiskScoreAbove   { threshold: f64 },
}

/// A composable boolean filter node (AND/OR tree).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ViewFilter {
    Predicate(FilterPredicate),
    And(Vec<ViewFilter>),
    Or(Vec<ViewFilter>),
    Not(Box<ViewFilter>),
}

/// Actor context supplied to the view engine for ownership filters.
#[derive(Debug, Clone)]
pub struct ActorContext {
    pub user_id: EntityId,
    pub org_ids: Vec<EntityId>,
}

// =============================================================================
// §2 — SORT SYSTEM
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortDirection { Ascending, Descending }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NullPlacement { First, Last }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewSort {
    pub column_id:      ColumnId,
    pub direction:      SortDirection,
    pub null_placement: NullPlacement,
}

impl ViewSort {
    pub fn asc(col: impl Into<ColumnId>) -> Self {
        Self { column_id: col.into(), direction: SortDirection::Ascending, null_placement: NullPlacement::Last }
    }
    pub fn desc(col: impl Into<ColumnId>) -> Self {
        Self { column_id: col.into(), direction: SortDirection::Descending, null_placement: NullPlacement::Last }
    }
}

// =============================================================================
// §3 — GROUP & ROLLUP SYSTEM
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewGroup {
    pub column_id:        ColumnId,
    pub collapse_default: bool,
    pub label_template:   Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Aggregation {
    Sum, Avg, Min, Max, Median, Count, CountNonEmpty, CountTrue, PctTrue, Mode, Union,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryRowConfig {
    pub columns: Vec<(ColumnId, Aggregation)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollupConfig {
    pub hierarchy_column: ColumnId,
    pub rollup_columns:   Vec<(ColumnId, Aggregation)>,
}

// =============================================================================
// §4 — PIVOT TABLE
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PivotConfig {
    pub row_field:   ColumnId,
    pub col_field:   ColumnId,
    pub value_field: ColumnId,
    pub aggregation: Aggregation,
    pub show_totals: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PivotTable {
    pub row_headers: Vec<String>,
    pub col_headers: Vec<String>,
    pub cells:       Vec<Vec<Option<f64>>>,
    pub row_totals:  Vec<Option<f64>>,
    pub col_totals:  Vec<Option<f64>>,
    pub grand_total: Option<f64>,
}

// =============================================================================
// §5 — CHART CONFIGURATION
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChartType { Bar, StackedBar, Line, Area, Scatter, Pie, Gauge, Treemap, NetworkGraph, Timeline }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartConfig {
    pub chart_type:   ChartType,
    pub x_column:     ColumnId,
    pub y_columns:    Vec<ColumnId>,
    pub group_by:     Option<ColumnId>,
    pub title:        Option<String>,
    pub color_scheme: Option<String>,
    pub show_legend:  bool,
}

// =============================================================================
// §6 — BOARD MODES
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoardMode { Kanban, Gantt, Calendar, AgileBoard, ResourceBoard, NetworkGraph, Treemap, TimelineBoard }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanbanLane { pub status: String, pub color: Option<String>, pub wip_limit: Option<u32> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GanttConfig {
    pub start_column:       ColumnId,
    pub end_column:         ColumnId,
    pub bar_color_column:   Option<ColumnId>,
    pub show_dependencies:  bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardConfig {
    pub mode:                    BoardMode,
    pub kanban_lanes:            Option<Vec<KanbanLane>>,
    pub gantt:                   Option<GanttConfig>,
    pub calendar_date_column:    Option<ColumnId>,
    pub treemap_size_column:     Option<ColumnId>,
    pub card_columns:            Vec<ColumnId>,
}

// =============================================================================
// §7 — ROW HEIGHT & CONDITIONAL FORMATTING
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RowHeight { Compact, Normal, Tall, Auto }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowHighlightRule {
    pub id:       Uuid,
    pub label:    String,
    pub filter:   ViewFilter,
    pub bg_color: String,
    pub fg_color: Option<String>,
    pub priority: u8,
}

// =============================================================================
// §8 — VIEW DEFINITION
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewVisibility { Private, Shared, Public, Template }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibleColumn {
    pub column_id: ColumnId,
    pub width_px:  Option<u16>,
    pub pinned:    bool,
    pub hidden:    bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewDefinition {
    pub id:                Uuid,
    pub name:              String,
    pub sheet_id:          String,
    pub base_row_filter:   Option<ViewFilter>,
    pub visible_columns:   Vec<VisibleColumn>,
    pub filters:           Vec<ViewFilter>,
    pub sorts:             Vec<ViewSort>,
    pub groups:            Vec<ViewGroup>,
    pub row_height:        RowHeight,
    pub highlight_rules:   Vec<RowHighlightRule>,
    pub pinned_column_ids: Vec<ColumnId>,
    pub frozen_row_count:  u32,
    pub pivot_config:      Option<PivotConfig>,
    pub chart_config:      Option<ChartConfig>,
    pub summary_row:       Option<SummaryRowConfig>,
    pub board_config:      Option<BoardConfig>,
    pub rollup_config:     Option<RollupConfig>,
    pub visibility:        ViewVisibility,
    pub created_by:        EntityId,
    pub created_at:        DateTime<Utc>,
    pub updated_at:        DateTime<Utc>,
}

impl ViewDefinition {
    pub fn new(name: impl Into<String>, sheet_id: impl Into<String>, created_by: EntityId) -> Self {
        let now = Utc::now();
        Self {
            id:                Uuid::new_v4(),
            name:              name.into(),
            sheet_id:          sheet_id.into(),
            base_row_filter:   None,
            visible_columns:   vec![],
            filters:           vec![],
            sorts:             vec![],
            groups:            vec![],
            row_height:        RowHeight::Normal,
            highlight_rules:   vec![],
            pinned_column_ids: vec![],
            frozen_row_count:  0,
            pivot_config:      None,
            chart_config:      None,
            summary_row:       None,
            board_config:      None,
            rollup_config:     None,
            visibility:        ViewVisibility::Private,
            created_by,
            created_at:        now,
            updated_at:        now,
        }
    }
}

// =============================================================================
// §9 — GROUPED VIEW OUTPUT
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupHeader {
    pub group_key:    String,
    pub group_value:  String,
    pub row_count:    usize,
    pub is_collapsed: bool,
    pub aggregates:   HashMap<ColumnId, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViewRow {
    Data(ComponentId),
    GroupHeader(GroupHeader),
    SummaryRow(HashMap<ColumnId, f64>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterializedView {
    pub view_id:          Uuid,
    pub sheet_id:         String,
    pub rows:             Vec<ViewRow>,
    pub total_data_rows:  usize,
    pub generated_at:     DateTime<Utc>,
}

// =============================================================================
// §10 — VIEW ENGINE
// =============================================================================

pub struct ViewEngine;

impl ViewEngine {
    pub fn apply(
        view:          &ViewDefinition,
        row_store:     &RowStore,
        cell_store:    &CellStore,
        actor:         &ActorContext,
        sheet_row_ids: &[ComponentId],
    ) -> MaterializedView {
        let mut rows: Vec<&PortfolioRow> = sheet_row_ids
            .iter()
            .filter_map(|id| row_store.get(id))
            .collect();

        if let Some(bf) = &view.base_row_filter {
            rows.retain(|r| Self::evaluate_filter(bf, r, cell_store, actor));
        }
        for f in &view.filters {
            rows.retain(|r| Self::evaluate_filter(f, r, cell_store, actor));
        }

        let total_data_rows = rows.len();

        if !view.sorts.is_empty() {
            rows.sort_by(|a, b| {
                for sort in &view.sorts {
                    let ord = Self::compare_by_col(a, b, &sort.column_id, cell_store, &sort.null_placement);
                    if ord != std::cmp::Ordering::Equal {
                        return if sort.direction == SortDirection::Descending { ord.reverse() } else { ord };
                    }
                }
                std::cmp::Ordering::Equal
            });
        }

        let view_rows = if view.groups.is_empty() {
            let mut out: Vec<ViewRow> = rows.iter().map(|r| ViewRow::Data(r.component_id)).collect();
            if let Some(sr) = &view.summary_row {
                out.push(ViewRow::SummaryRow(Self::summary(&rows, &sr.columns, cell_store)));
            }
            out
        } else {
            Self::group(&rows, &view.groups, &view.summary_row, cell_store)
        };

        MaterializedView {
            view_id:        view.id,
            sheet_id:       view.sheet_id.clone(),
            rows:           view_rows,
            total_data_rows,
            generated_at:   Utc::now(),
        }
    }

    // ── Filter ────────────────────────────────────────────────────────────────

    pub fn evaluate_filter(f: &ViewFilter, row: &PortfolioRow, cs: &CellStore, actor: &ActorContext) -> bool {
        match f {
            ViewFilter::Predicate(p) => Self::eval_pred(p, row, cs, actor),
            ViewFilter::And(fs)      => fs.iter().all(|f| Self::evaluate_filter(f, row, cs, actor)),
            ViewFilter::Or(fs)       => fs.iter().any(|f| Self::evaluate_filter(f, row, cs, actor)),
            ViewFilter::Not(inner)   => !Self::evaluate_filter(inner, row, cs, actor),
        }
    }

    fn eval_pred(p: &FilterPredicate, row: &PortfolioRow, cs: &CellStore, actor: &ActorContext) -> bool {
        match p {
            FilterPredicate::Equals { column, value } =>
                Self::as_json(row, column, cs) == Some(value.clone()),
            FilterPredicate::NotEquals { column, value } =>
                Self::as_json(row, column, cs).as_ref() != Some(value),
            FilterPredicate::GreaterThan    { column, value } => Self::as_f64(row, column, cs).map_or(false, |v| v > *value),
            FilterPredicate::LessThan       { column, value } => Self::as_f64(row, column, cs).map_or(false, |v| v < *value),
            FilterPredicate::GreaterOrEqual { column, value } => Self::as_f64(row, column, cs).map_or(false, |v| v >= *value),
            FilterPredicate::LessOrEqual    { column, value } => Self::as_f64(row, column, cs).map_or(false, |v| v <= *value),
            FilterPredicate::Between { column, min, max } =>
                Self::as_f64(row, column, cs).map_or(false, |v| v >= *min && v <= *max),
            FilterPredicate::Contains { column, substring } =>
                Self::as_str(row, column, cs).map_or(false, |s| s.to_lowercase().contains(&substring.to_lowercase())),
            FilterPredicate::StartsWith { column, prefix } =>
                Self::as_str(row, column, cs).map_or(false, |s| s.to_lowercase().starts_with(&prefix.to_lowercase())),
            FilterPredicate::EndsWith { column, suffix } =>
                Self::as_str(row, column, cs).map_or(false, |s| s.to_lowercase().ends_with(&suffix.to_lowercase())),
            FilterPredicate::DateAfter  { column, date }        => Self::as_date(row, column, cs).map_or(false, |d| d > *date),
            FilterPredicate::DateBefore { column, date }        => Self::as_date(row, column, cs).map_or(false, |d| d < *date),
            FilterPredicate::DateBetween { column, start, end } => Self::as_date(row, column, cs).map_or(false, |d| d >= *start && d <= *end),
            FilterPredicate::InSet    { column, values } => Self::as_json(row, column, cs).map_or(false, |v| values.contains(&v)),
            FilterPredicate::NotInSet { column, values } => Self::as_json(row, column, cs).map_or(true, |v| !values.contains(&v)),
            FilterPredicate::TagIncludes  { tag, .. }   => row.tags.contains(tag),
            FilterPredicate::HasRiskSeverity { severity } => row.risk_flags.iter().any(|r| &r.severity == severity),
            FilterPredicate::HasComplianceFlag { flag }   => row.compliance_flags.contains(flag),
            FilterPredicate::IsEmpty    { column } => Self::as_json(row, column, cs).is_none(),
            FilterPredicate::IsNotEmpty { column } => Self::as_json(row, column, cs).is_some(),
            FilterPredicate::IsTrue     { column } => Self::as_f64(row, column, cs).map_or(false, |v| v != 0.0),
            FilterPredicate::IsFalse    { column } => Self::as_f64(row, column, cs).map_or(false, |v| v == 0.0),
            FilterPredicate::OwnedByMe =>
                row.owners.contains(&actor.user_id)
                || actor.org_ids.iter().any(|oid| row.owners.contains(oid)),
            FilterPredicate::HealthScoreAbove { threshold } =>
                cs.get(&row.component_id, "health_score").and_then(|v| v.as_f64()).map_or(false, |s| s > *threshold),
            FilterPredicate::HealthScoreBelow { threshold } =>
                cs.get(&row.component_id, "health_score").and_then(|v| v.as_f64()).map_or(false, |s| s < *threshold),
            FilterPredicate::RiskScoreAbove { threshold } =>
                cs.get(&row.component_id, "risk_score").and_then(|v| v.as_f64()).map_or(false, |s| s > *threshold),
        }
    }

    // ── Column accessor helpers ───────────────────────────────────────────────

    pub fn as_f64(row: &PortfolioRow, col: &str, cs: &CellStore) -> Option<f64> {
        if let Some(v) = cs.get(&row.component_id, col) { return v.as_f64(); }
        match col {
            "progress_pct"             => Some(row.progress_pct as f64),
            "budget_spent"             => row.budget_spent.to_string().parse().ok(),
            "budget_allocated"         => row.budget_allocated?.to_string().parse().ok(),
            "revenue"                  => row.revenue.to_string().parse().ok(),
            "expenses"                 => row.expenses.to_string().parse().ok(),
            "views"                    => Some(row.views as f64),
            "clicks"                   => Some(row.clicks as f64),
            "shares"                   => Some(row.shares as f64),
            "followers"                => Some(row.followers as f64),
            "resource_units_allocated" => Some(row.resource_units_allocated),
            "allocation_pct"           => Some(row.allocation_pct),
            "merge_conflicts"          => Some(row.merge_conflicts as f64),
            _                          => None,
        }
    }

    pub fn as_str(row: &PortfolioRow, col: &str, cs: &CellStore) -> Option<String> {
        if let Some(CellValue::Text(s)) = cs.get(&row.component_id, col) { return Some(s.clone()); }
        match col {
            "name"             => Some(row.name.clone()),
            "slug"             => Some(row.slug.clone()),
            "status"           => Some(row.status.clone()),
            "state"            => Some(row.state.clone()),
            "visibility"       => Some(row.visibility.clone()),
            "component_type"   => Some(row.component_type.clone()),
            "item_type"        => row.item_type.clone(),
            "container_type"   => row.container_type.clone(),
            "domain"           => row.domain.clone(),
            "display_name"     => row.display_name.clone().or_else(|| Some(row.name.clone())),
            "governance_model" => row.governance_model.clone(),
            "approval_status"  => row.approval_status.clone(),
            "tax_category"     => row.tax_category.clone(),
            "payment_status"   => row.payment_status.clone(),
            "benefit_type"     => row.benefit_type.clone(),
            "benefit_provider" => row.benefit_provider.clone(),
            "engagement_type"  => row.engagement_type.clone(),
            "platform"         => row.platform.clone(),
            "campaign_type"    => row.campaign_type.clone(),
            "funding_type"     => row.funding_type.clone(),
            "quarter"          => row.quarter.clone(),
            _                  => None,
        }
    }

    pub fn as_date(row: &PortfolioRow, col: &str, cs: &CellStore) -> Option<NaiveDate> {
        if let Some(CellValue::Date(d)) = cs.get(&row.component_id, col) { return Some(*d); }
        match col {
            "start_date" => row.start_date,
            "end_date"   => row.end_date,
            "due_date"   => row.due_date,
            "close_date" => row.close_date,
            _            => None,
        }
    }

    pub fn as_json(row: &PortfolioRow, col: &str, cs: &CellStore) -> Option<serde_json::Value> {
        if let Some(cv) = cs.get(&row.component_id, col) {
            return serde_json::to_value(cv).ok();
        }
        match col {
            "status"         => Some(serde_json::json!(row.status)),
            "visibility"     => Some(serde_json::json!(row.visibility)),
            "item_type"      => row.item_type.as_ref().map(|t| serde_json::json!(t)),
            "component_type" => Some(serde_json::json!(row.component_type)),
            "name"           => Some(serde_json::json!(row.name)),
            "platform"       => row.platform.as_ref().map(|p| serde_json::json!(p)),
            "benefit_type"   => row.benefit_type.as_ref().map(|b| serde_json::json!(b)),
            "quarter"        => row.quarter.as_ref().map(|q| serde_json::json!(q)),
            _                => None,
        }
    }

    // ── Grouping ──────────────────────────────────────────────────────────────

    fn group(
        rows:     &[&PortfolioRow],
        groups:   &[ViewGroup],
        summary:  &Option<SummaryRowConfig>,
        cs:       &CellStore,
    ) -> Vec<ViewRow> {
        if groups.is_empty() {
            return rows.iter().map(|r| ViewRow::Data(r.component_id)).collect();
        }
        let g = &groups[0];
        let mut order: Vec<String> = Vec::new();
        let mut buckets: HashMap<String, Vec<&PortfolioRow>> = HashMap::new();
        for row in rows {
            let key = Self::as_str(row, &g.column_id, cs).unwrap_or("(empty)".into());
            if !buckets.contains_key(&key) { order.push(key.clone()); }
            buckets.entry(key).or_default().push(row);
        }
        let mut out = Vec::new();
        for key in &order {
            let bucket = &buckets[key];
            out.push(ViewRow::GroupHeader(GroupHeader {
                group_key:    g.column_id.clone(),
                group_value:  key.clone(),
                row_count:    bucket.len(),
                is_collapsed: g.collapse_default,
                aggregates:   HashMap::new(),
            }));
            if !g.collapse_default {
                out.extend(Self::group(bucket, &groups[1..], &None, cs));
            }
        }
        if let Some(sr) = summary {
            out.push(ViewRow::SummaryRow(Self::summary(rows, &sr.columns, cs)));
        }
        out
    }

    // ── Aggregation ───────────────────────────────────────────────────────────

    fn summary(rows: &[&PortfolioRow], cols: &[(ColumnId, Aggregation)], cs: &CellStore) -> HashMap<ColumnId, f64> {
        cols.iter().filter_map(|(col, agg)| {
            let vals: Vec<f64> = rows.iter().filter_map(|r| Self::as_f64(r, col, cs)).collect();
            Self::aggregate(&vals, agg).map(|v| (col.clone(), v))
        }).collect()
    }

    pub fn aggregate(values: &[f64], agg: &Aggregation) -> Option<f64> {
        if values.is_empty() { return None; }
        match agg {
            Aggregation::Sum          => Some(values.iter().sum()),
            Aggregation::Avg          => Some(values.iter().sum::<f64>() / values.len() as f64),
            Aggregation::Min          => values.iter().cloned().reduce(f64::min),
            Aggregation::Max          => values.iter().cloned().reduce(f64::max),
            Aggregation::Count        => Some(values.len() as f64),
            Aggregation::CountNonEmpty=> Some(values.len() as f64),
            Aggregation::CountTrue    => Some(values.iter().filter(|&&v| v != 0.0).count() as f64),
            Aggregation::PctTrue      => {
                let t = values.iter().filter(|&&v| v != 0.0).count() as f64;
                Some(t / values.len() as f64 * 100.0)
            },
            Aggregation::Median => {
                let mut s = values.to_vec();
                s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let m = s.len() / 2;
                if s.len() % 2 == 0 { Some((s[m-1]+s[m])/2.0) } else { Some(s[m]) }
            },
            _ => None,
        }
    }

    fn compare_by_col(a: &PortfolioRow, b: &PortfolioRow, col: &str, cs: &CellStore, nulls: &NullPlacement) -> std::cmp::Ordering {
        match (Self::as_f64(a, col, cs), Self::as_f64(b, col, cs)) {
            (None, None)     => std::cmp::Ordering::Equal,
            (None, Some(_))  => if *nulls == NullPlacement::First { std::cmp::Ordering::Less  } else { std::cmp::Ordering::Greater },
            (Some(_), None)  => if *nulls == NullPlacement::First { std::cmp::Ordering::Greater } else { std::cmp::Ordering::Less },
            (Some(va), Some(vb)) => va.partial_cmp(&vb).unwrap_or(std::cmp::Ordering::Equal),
        }
    }

    // ── Pivot ─────────────────────────────────────────────────────────────────

    pub fn compute_pivot(cfg: &PivotConfig, rows: &[&PortfolioRow], cs: &CellStore) -> PivotTable {
        let mut row_vals: Vec<String> = Vec::new();
        let mut col_vals: Vec<String> = Vec::new();
        let mut rv_seen  = HashSet::new();
        let mut cv_seen  = HashSet::new();
        for row in rows {
            let rv = Self::as_str(row, &cfg.row_field, cs).unwrap_or("(empty)".into());
            let cv = Self::as_str(row, &cfg.col_field, cs).unwrap_or("(empty)".into());
            if rv_seen.insert(rv.clone()) { row_vals.push(rv); }
            if cv_seen.insert(cv.clone()) { col_vals.push(cv); }
        }
        row_vals.sort(); col_vals.sort();

        let mut buckets: HashMap<(String, String), Vec<f64>> = HashMap::new();
        for row in rows {
            let rv = Self::as_str(row, &cfg.row_field, cs).unwrap_or("(empty)".into());
            let cv = Self::as_str(row, &cfg.col_field, cs).unwrap_or("(empty)".into());
            if let Some(v) = Self::as_f64(row, &cfg.value_field, cs) {
                buckets.entry((rv, cv)).or_default().push(v);
            }
        }

        let cells: Vec<Vec<Option<f64>>> = row_vals.iter().map(|rv| {
            col_vals.iter().map(|cv| {
                buckets.get(&(rv.clone(), cv.clone()))
                    .and_then(|vals| Self::aggregate(vals, &cfg.aggregation))
            }).collect()
        }).collect();

        let row_totals: Vec<Option<f64>> = cells.iter().map(|row| {
            let v: Vec<f64> = row.iter().filter_map(|v| *v).collect();
            Self::aggregate(&v, &cfg.aggregation)
        }).collect();

        let col_totals: Vec<Option<f64>> = (0..col_vals.len()).map(|ci| {
            let v: Vec<f64> = cells.iter().filter_map(|row| row[ci]).collect();
            Self::aggregate(&v, &cfg.aggregation)
        }).collect();

        let all: Vec<f64> = cells.iter().flatten().filter_map(|v| *v).collect();
        PivotTable { row_headers: row_vals, col_headers: col_vals, cells, row_totals, col_totals, grand_total: Self::aggregate(&all, &cfg.aggregation) }
    }
}

// =============================================================================
// §11 — BUILT-IN VIEW TEMPLATES
// =============================================================================

pub struct ViewTemplates;

impl ViewTemplates {
    pub fn at_risk_this_week(created_by: EntityId) -> ViewDefinition {
        let mut vd = ViewDefinition::new("At Risk This Week", "SHT-001", created_by);
        let today     = chrono::Utc::now().date_naive();
        let next_week = today + chrono::Duration::days(7);
        vd.filters = vec![ViewFilter::Or(vec![
            ViewFilter::Predicate(FilterPredicate::HealthScoreBelow { threshold: 60.0 }),
            ViewFilter::And(vec![
                ViewFilter::Predicate(FilterPredicate::DateBefore { column: "due_date".into(), date: next_week }),
                ViewFilter::Predicate(FilterPredicate::DateAfter  { column: "due_date".into(), date: today }),
                ViewFilter::Predicate(FilterPredicate::Equals { column: "status".into(), value: serde_json::json!("Active") }),
            ]),
        ])];
        vd.sorts = vec![ViewSort::asc("due_date")];
        vd
    }

    pub fn my_active_projects(created_by: EntityId) -> ViewDefinition {
        let mut vd = ViewDefinition::new("My Active Projects", "SHT-004", created_by);
        vd.base_row_filter = Some(ViewFilter::And(vec![
            ViewFilter::Predicate(FilterPredicate::OwnedByMe),
            ViewFilter::Predicate(FilterPredicate::Equals { column: "status".into(), value: serde_json::json!("Active") }),
        ]));
        vd.sorts = vec![ViewSort::asc("due_date")];
        vd
    }

    pub fn kanban_board(created_by: EntityId) -> ViewDefinition {
        let mut vd = ViewDefinition::new("Kanban Board", "SHT-005", created_by);
        vd.board_config = Some(BoardConfig {
            mode: BoardMode::Kanban,
            kanban_lanes: Some(vec![
                KanbanLane { status: "Draft".into(),     color: Some("#94a3b8".into()), wip_limit: None },
                KanbanLane { status: "Active".into(),    color: Some("#3b82f6".into()), wip_limit: Some(5) },
                KanbanLane { status: "Blocked".into(),   color: Some("#ef4444".into()), wip_limit: None },
                KanbanLane { status: "Completed".into(), color: Some("#22c55e".into()), wip_limit: None },
            ]),
            gantt: None, calendar_date_column: None, treemap_size_column: None,
            card_columns: vec!["owners".into(), "due_date".into(), "health_score".into()],
        });
        vd.sorts = vec![ViewSort::desc("progress_pct")];
        vd
    }
}

// =============================================================================
// §12 — TESTS
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::portfolio_spreadsheet_substrate::{CellStore, PortfolioRow, RowStore};

    fn owner() -> EntityId { Uuid::new_v4() }
    fn actor(id: EntityId) -> ActorContext { ActorContext { user_id: id, org_ids: vec![] } }

    fn row(name: &str, status: &str, itype: &str, owner: EntityId, progress: f32) -> PortfolioRow {
        let mut r = PortfolioRow::new(Uuid::new_v4(), "item", name, owner);
        r.status = status.into(); r.item_type = Some(itype.into()); r.progress_pct = progress;
        r
    }

    #[test] fn test_equals_filter() {
        let o = owner(); let cs = CellStore::new(); let a = actor(o);
        let r = row("P", "Active", "Project", o, 50.0);
        assert!( ViewEngine::evaluate_filter(&ViewFilter::Predicate(FilterPredicate::Equals { column: "status".into(), value: serde_json::json!("Active") }), &r, &cs, &a));
        assert!(!ViewEngine::evaluate_filter(&ViewFilter::Predicate(FilterPredicate::Equals { column: "status".into(), value: serde_json::json!("Draft")  }), &r, &cs, &a));
    }

    #[test] fn test_gt_filter() {
        let o = owner(); let cs = CellStore::new(); let a = actor(o);
        let r = row("P", "Active", "Project", o, 75.0);
        assert!( ViewEngine::evaluate_filter(&ViewFilter::Predicate(FilterPredicate::GreaterThan { column: "progress_pct".into(), value: 50.0 }), &r, &cs, &a));
        assert!(!ViewEngine::evaluate_filter(&ViewFilter::Predicate(FilterPredicate::GreaterThan { column: "progress_pct".into(), value: 90.0 }), &r, &cs, &a));
    }

    #[test] fn test_and_filter() {
        let o = owner(); let cs = CellStore::new(); let a = actor(o);
        let r = row("P", "Active", "Project", o, 80.0);
        let f = ViewFilter::And(vec![
            ViewFilter::Predicate(FilterPredicate::Equals { column: "status".into(), value: serde_json::json!("Active") }),
            ViewFilter::Predicate(FilterPredicate::GreaterThan { column: "progress_pct".into(), value: 70.0 }),
        ]);
        assert!(ViewEngine::evaluate_filter(&f, &r, &cs, &a));
    }

    #[test] fn test_owned_by_me() {
        let me = owner(); let other = Uuid::new_v4();
        let cs = CellStore::new();
        let r = row("P", "Active", "Project", me, 50.0);
        assert!( ViewEngine::evaluate_filter(&ViewFilter::Predicate(FilterPredicate::OwnedByMe), &r, &cs, &actor(me)));
        assert!(!ViewEngine::evaluate_filter(&ViewFilter::Predicate(FilterPredicate::OwnedByMe), &r, &cs, &actor(other)));
    }

    #[test] fn test_sort() {
        let o = owner(); let a = actor(o);
        let mut rs = RowStore::new(); let cs = CellStore::new();
        let ids: Vec<ComponentId> = vec![30.0f32, 10.0, 20.0].into_iter().map(|p| {
            let r = row(&p.to_string(), "Active", "Project", o, p);
            let id = r.component_id; rs.insert(r); id
        }).collect();
        let mut vd = ViewDefinition::new("T", "SHT-004", o);
        vd.sorts = vec![ViewSort::desc("progress_pct")];
        let mv = ViewEngine::apply(&vd, &rs, &cs, &a, &ids);
        let names: Vec<_> = mv.rows.iter().filter_map(|r| match r {
            ViewRow::Data(id) => rs.get(id).map(|r| r.name.clone()),
            _ => None,
        }).collect();
        assert_eq!(names, vec!["30", "20", "10"]);
    }

    #[test] fn test_aggregation_values() {
        let v = vec![10.0, 20.0, 30.0, 40.0];
        assert_eq!(ViewEngine::aggregate(&v, &Aggregation::Sum),    Some(100.0));
        assert_eq!(ViewEngine::aggregate(&v, &Aggregation::Avg),    Some(25.0));
        assert_eq!(ViewEngine::aggregate(&v, &Aggregation::Min),    Some(10.0));
        assert_eq!(ViewEngine::aggregate(&v, &Aggregation::Max),    Some(40.0));
        assert_eq!(ViewEngine::aggregate(&v, &Aggregation::Median), Some(25.0));
    }

    #[test] fn test_pivot() {
        let o = owner(); let cs = CellStore::new();
        let rows: Vec<PortfolioRow> = [("Uber", 100.0), ("DoorDash", 200.0), ("Uber", 150.0)].iter().map(|(p, amt)| {
            let mut r = PortfolioRow::new(Uuid::new_v4(), "item", "Gig", o);
            r.platform = Some(p.to_string());
            r.quarter  = Some("Q1".into());
            r.revenue  = rust_decimal::Decimal::from(*amt as i64);
            r
        }).collect();
        let refs: Vec<&PortfolioRow> = rows.iter().collect();
        let cfg = PivotConfig { row_field: "platform".into(), col_field: "quarter".into(), value_field: "revenue".into(), aggregation: Aggregation::Sum, show_totals: true };
        let pt = ViewEngine::compute_pivot(&cfg, &refs, &cs);
        assert_eq!(pt.grand_total.unwrap(), 450.0);
    }
}

// =============================================================================
//  portfolio_computation.rs — Kogi OS · Portfolio Computation Engine (KPCM)
//  Independent Worker Operating System
//
//  KPCM powers all ComputedColumns, AIColumns, and rollup aggregations in the
//  spreadsheet.  Two tiers:
//    · Synchronous — simple arithmetic / rollup formulae evaluated in-request
//    · Asynchronous — complex AI-driven signals computed by kogi-engine and
//                     written back via the plugin writeback protocol
//
//  Computational models (SDD §8.1 + §10):
//    1.  PortfolioHealth     — 7-dimension health score (SDD §10)
//    2.  ProjectMetrics      — sprint velocity, cycle time, lead time
//    3.  ProgramAlignment    — child alignment to program KPIs
//    4.  SubPortfolioRollup  — recursive aggregation up the hierarchy
//    5.  ResourceUtilisation — allocation utilisation %
//    6.  AssetValue          — asset valuation tracking
//    7.  ArtifactMaturity    — artifact version maturity scoring
//    8.  BinderCoverage      — binder completeness
//    9.  BookConsistency     — book structural consistency
//   10.  FolderOrganisation  — folder depth and organisation score
//   11.  RecordIntegrity     — formal record completeness
//   12.  CollaborationScore  — contributor diversity × velocity × governance
//   13.  IncomeProjection    — gig income projection model
//   14.  RiskModel           — risk score aggregation
//   15.  BenefitsCoverage    — benefit account coverage analysis
//
//  Formula Language (SDD §13.2 PQL, §7.1 FormulaColumn):
//    Basic arithmetic: +, -, *, /
//    Functions: SUM(), AVG(), MIN(), MAX(), COUNT(), IF(), COALESCE()
//    Column references: bare snake_case identifiers
//
//  @version  2.2.0
//  @license  MIT
// =============================================================================

#![allow(dead_code)]

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Cross-module type aliases ─────────────────────────────────────────────────
pub type ComponentId = Uuid;
pub type EntityId    = Uuid;

// =============================================================================
// §1 — Shared input types
// =============================================================================

/// A lightweight view of a child component, used as computation input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildSnapshot {
    pub component_id:          ComponentId,
    pub type_name:             String,
    pub status:                String,
    pub progress_pct:          f32,
    pub budget_allocated:      Option<Decimal>,
    pub budget_spent:          Decimal,
    pub risk_flags:            Vec<RiskFlag>,
    pub compliance_flags:      Vec<String>,
    pub start_date:            Option<chrono::NaiveDate>,
    pub due_date:              Option<chrono::NaiveDate>,
    pub milestone_count:       u32,
    pub milestone_completed:   u32,
    pub sprint_velocity:       Option<f32>,
    pub resource_utilization_pct: Option<f32>,
    pub contribution_count:    u32,
}

/// A risk flag on a component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFlag {
    pub severity:    RiskSeverity,
    pub probability: f32,           // 0.0 – 1.0
    pub description: String,
    pub mitigated:   bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskSeverity { Low, Medium, High, Critical }

impl RiskFlag {
    /// Risk contribution = severity_weight × probability × (1 if unmitigated, 0.2 if mitigated).
    pub fn weighted_score(&self) -> f32 {
        let sev_weight = match self.severity {
            RiskSeverity::Low      => 0.25,
            RiskSeverity::Medium   => 0.50,
            RiskSeverity::High     => 0.75,
            RiskSeverity::Critical => 1.00,
        };
        let mit_factor = if self.mitigated { 0.2 } else { 1.0 };
        sev_weight * self.probability * mit_factor * 100.0
    }
}

// =============================================================================
// §2 — PortfolioHealth Model  (SDD §10)
// =============================================================================

/// Full decomposed PortfolioHealth output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioHealthOutput {
    /// Composite score (0–100).
    pub health_score:        f32,
    pub delivery_health:     f32,
    pub financial_health:    f32,
    pub risk_health:         f32,
    pub resource_health:     f32,
    pub engagement_health:   f32,
    pub governance_health:   f32,
    pub benefit_coverage:    f32,
    pub anomalies:           Vec<AnomalyFlag>,
    pub computed_at:         DateTime<Utc>,
}

/// An anomaly detected by the health model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyFlag {
    pub dimension:   String,
    pub severity:    RiskSeverity,
    pub description: String,
    pub column_id:   String,
}

/// Inputs for the PortfolioHealth model.
#[derive(Debug, Clone)]
pub struct PortfolioHealthInput {
    pub children:                  Vec<ChildSnapshot>,
    pub budget_allocated:          Option<Decimal>,
    pub budget_spent:              Decimal,
    pub risk_flags:                Vec<RiskFlag>,
    pub compliance_flags:          Vec<String>,
    pub open_approval_count:       u32,
    pub followers:                 u64,
    pub engagement_rate:           f32,
    pub active_contributor_count:  u32,
    pub benefit_coverage_score:    Option<f32>,
    pub pto_remaining_days:        Option<f32>,
}

pub struct PortfolioHealthModel;

impl PortfolioHealthModel {
    /// Compute the full 7-dimension health score (SDD §10.1 / §10.2).
    pub fn compute(input: &PortfolioHealthInput) -> PortfolioHealthOutput {
        let delivery  = Self::score_delivery(input);
        let financial = Self::score_financial(input);
        let risk      = Self::score_risk(input);
        let resource  = Self::score_resources(input);
        let engagement = Self::score_engagement(input);
        let governance = Self::score_governance(input);
        let benefits  = input.benefit_coverage_score.unwrap_or(75.0);

        // Weighted composite (SDD §10.2)
        let score = delivery   * 0.25
                  + financial  * 0.20
                  + risk       * 0.20
                  + resource   * 0.15
                  + engagement * 0.10
                  + governance * 0.05
                  + benefits   * 0.05;

        let anomalies = Self::detect_anomalies(input, delivery, financial, risk, resource);

        PortfolioHealthOutput {
            health_score:     score.clamp(0.0, 100.0),
            delivery_health:  delivery,
            financial_health: financial,
            risk_health:      risk,
            resource_health:  resource,
            engagement_health: engagement,
            governance_health: governance,
            benefit_coverage: benefits,
            anomalies,
            computed_at: Utc::now(),
        }
    }

    fn score_delivery(input: &PortfolioHealthInput) -> f32 {
        if input.children.is_empty() { return 75.0; }
        let on_track: f32 = input.children.iter()
            .filter(|c| matches!(c.status.as_str(), "Active" | "InProgress" | "Completed"))
            .count() as f32 / input.children.len() as f32;

        let avg_progress: f32 = input.children.iter()
            .map(|c| c.progress_pct)
            .sum::<f32>() / input.children.len() as f32;

        let milestone_rate: f32 = {
            let total: u32   = input.children.iter().map(|c| c.milestone_count).sum();
            let done:  u32   = input.children.iter().map(|c| c.milestone_completed).sum();
            if total == 0 { 1.0 } else { done as f32 / total as f32 }
        };

        ((on_track * 40.0) + (avg_progress * 0.4) + (milestone_rate * 20.0)).clamp(0.0, 100.0)
    }

    fn score_financial(input: &PortfolioHealthInput) -> f32 {
        let Some(allocated) = input.budget_allocated else { return 80.0 };
        if allocated.is_zero() { return 80.0; }
        let utilization = (input.budget_spent / allocated)
            .to_f64().unwrap_or(0.0) as f32;

        let overrun_penalty = if utilization > 1.0 { (utilization - 1.0) * 100.0 } else { 0.0 };

        // Full score at ≤80% utilization; linear decay above
        let base = if utilization <= 0.80 {
            100.0
        } else if utilization <= 1.00 {
            100.0 - (utilization - 0.80) * 250.0
        } else {
            0.0
        };

        (base - overrun_penalty).clamp(0.0, 100.0)
    }

    fn score_risk(input: &PortfolioHealthInput) -> f32 {
        let risk_score = Self::compute_risk_score(&input.risk_flags);
        // Risk health is inverse of risk score
        (100.0 - risk_score).clamp(0.0, 100.0)
    }

    pub fn compute_risk_score(flags: &[RiskFlag]) -> f32 {
        if flags.is_empty() { return 0.0; }
        let raw: f32 = flags.iter().map(|f| f.weighted_score()).sum();
        (raw / flags.len() as f32).clamp(0.0, 100.0)
    }

    fn score_resources(input: &PortfolioHealthInput) -> f32 {
        let util_values: Vec<f32> = input.children.iter()
            .filter_map(|c| c.resource_utilization_pct)
            .collect();
        if util_values.is_empty() { return 75.0; }

        let avg_util = util_values.iter().sum::<f32>() / util_values.len() as f32;
        // Ideal utilisation: 60–80%.  Penalty for over (>100%) or under (<30%) allocation.
        let score = if avg_util >= 60.0 && avg_util <= 80.0 {
            100.0
        } else if avg_util > 80.0 && avg_util <= 100.0 {
            100.0 - (avg_util - 80.0) * 2.5
        } else if avg_util > 100.0 {
            50.0 - (avg_util - 100.0) * 2.0
        } else {
            60.0 - (30.0 - avg_util.min(30.0)) * 2.0
        };
        score.clamp(0.0, 100.0)
    }

    fn score_engagement(input: &PortfolioHealthInput) -> f32 {
        let collab_bonus = (input.active_contributor_count as f32 * 2.0).min(20.0);
        let eng_score    = (input.engagement_rate * 100.0).min(60.0);
        let followers_score = ((input.followers as f32).log2().max(0.0) * 5.0).min(20.0);
        (eng_score + collab_bonus + followers_score).clamp(0.0, 100.0)
    }

    fn score_governance(input: &PortfolioHealthInput) -> f32 {
        let mut score = 100.0f32;
        score -= input.open_approval_count as f32 * 5.0;
        score -= input.compliance_flags.len() as f32 * 15.0;
        score.clamp(0.0, 100.0)
    }

    fn detect_anomalies(
        input:     &PortfolioHealthInput,
        delivery:  f32,
        financial: f32,
        risk:      f32,
        resource:  f32,
    ) -> Vec<AnomalyFlag> {
        let mut anomalies = vec![];

        if delivery < 50.0 {
            anomalies.push(AnomalyFlag {
                dimension: "delivery".into(),
                severity: RiskSeverity::High,
                description: "Delivery health critically low — schedule variance detected".into(),
                column_id: "progress_pct".into(),
            });
        }
        if let Some(alloc) = input.budget_allocated {
            if !alloc.is_zero() {
                let util = (input.budget_spent / alloc).to_f64().unwrap_or(0.0) as f32;
                if util > 0.80 {
                    anomalies.push(AnomalyFlag {
                        dimension: "financial".into(),
                        severity: if util > 0.95 { RiskSeverity::Critical } else { RiskSeverity::High },
                        description: format!("Budget utilisation at {:.0}%", util * 100.0),
                        column_id: "budget_utilization_pct".into(),
                    });
                }
            }
        }
        if risk < 40.0 {
            anomalies.push(AnomalyFlag {
                dimension: "risk".into(),
                severity: RiskSeverity::Critical,
                description: "High unmitigated risk concentration".into(),
                column_id: "risk_flags".into(),
            });
        }
        if resource < 40.0 {
            anomalies.push(AnomalyFlag {
                dimension: "resource".into(),
                severity: RiskSeverity::Medium,
                description: "Resource allocation outside optimal range".into(),
                column_id: "resource_utilization_pct".into(),
            });
        }
        anomalies
    }
}

// =============================================================================
// §3 — ProjectMetrics Model  (SDD §8.1)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetricsOutput {
    pub velocity:          f32,   // story points per sprint
    pub throughput:        f32,   // items completed per period
    pub cycle_time_days:   f32,
    pub lead_time_days:    f32,
    pub burndown_slope:    f32,   // points remaining per day (negative = healthy)
    pub completion_rate:   f32,   // % of committed items completed per sprint
    pub computed_at:       DateTime<Utc>,
}

pub struct ProjectMetricsModel;

impl ProjectMetricsModel {
    pub fn compute(
        sprint_velocities:       &[f32],
        cycle_times_days:        &[f32],
        lead_times_days:         &[f32],
        completed_per_period:    &[u32],
        committed_per_sprint:    u32,
        completed_in_sprint:     u32,
    ) -> ProjectMetricsOutput {
        let velocity = if sprint_velocities.is_empty() { 0.0 }
            else { sprint_velocities.iter().sum::<f32>() / sprint_velocities.len() as f32 };

        let cycle_time_days = if cycle_times_days.is_empty() { 0.0 }
            else { cycle_times_days.iter().sum::<f32>() / cycle_times_days.len() as f32 };

        let lead_time_days = if lead_times_days.is_empty() { 0.0 }
            else { lead_times_days.iter().sum::<f32>() / lead_times_days.len() as f32 };

        let throughput = if completed_per_period.is_empty() { 0.0 }
            else { completed_per_period.iter().sum::<u32>() as f32 / completed_per_period.len() as f32 };

        let completion_rate = if committed_per_sprint == 0 { 0.0 }
            else { completed_in_sprint as f32 / committed_per_sprint as f32 * 100.0 };

        // Linear regression slope for remaining work (simplified)
        let burndown_slope = if sprint_velocities.len() >= 2 {
            let last_two: Vec<_> = sprint_velocities.iter().rev().take(2).collect();
            -(last_two[0] - last_two[1])  // negative = burning down correctly
        } else {
            0.0
        };

        ProjectMetricsOutput {
            velocity, throughput, cycle_time_days, lead_time_days,
            burndown_slope, completion_rate, computed_at: Utc::now(),
        }
    }
}

// =============================================================================
// §4 — ProgramAlignment Model
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KPIAlignment {
    pub kpi_name:   String,
    pub target:     f32,
    pub current:    f32,
    pub alignment:  f32,  // 0–100: how close current is to target
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramAlignmentOutput {
    pub alignment_score:  f32,
    pub kpi_alignments:   Vec<KPIAlignment>,
    pub child_count:      u32,
    pub aligned_children: u32,
    pub computed_at:      DateTime<Utc>,
}

pub struct ProgramAlignmentModel;

impl ProgramAlignmentModel {
    pub fn compute(
        kpi_alignments: Vec<KPIAlignment>,
        child_count:    u32,
        aligned_children: u32,
    ) -> ProgramAlignmentOutput {
        let kpi_score = if kpi_alignments.is_empty() { 75.0 }
            else {
                kpi_alignments.iter().map(|k| k.alignment).sum::<f32>()
                    / kpi_alignments.len() as f32
            };

        let child_score = if child_count == 0 { 75.0 }
            else { aligned_children as f32 / child_count as f32 * 100.0 };

        let alignment_score = (kpi_score * 0.6 + child_score * 0.4).clamp(0.0, 100.0);

        ProgramAlignmentOutput {
            alignment_score,
            kpi_alignments,
            child_count,
            aligned_children,
            computed_at: Utc::now(),
        }
    }
}

// =============================================================================
// §5 — SubPortfolioRollup Model
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubPortfolioRollupOutput {
    pub total_budget_allocated:    Decimal,
    pub total_budget_spent:        Decimal,
    pub budget_utilization_pct:    f32,
    pub total_revenue:             Decimal,
    pub total_expenses:            Decimal,
    pub total_profit:              Decimal,
    pub avg_health_score:          f32,
    pub active_count:              u32,
    pub completed_count:           u32,
    pub at_risk_count:             u32,
    pub component_count:           u32,
    pub computed_at:               DateTime<Utc>,
}

pub struct SubPortfolioRollupModel;

impl SubPortfolioRollupModel {
    pub fn compute(children: &[ChildSnapshot], child_health_scores: &[f32]) -> SubPortfolioRollupOutput {
        let total_allocated: Decimal = children.iter()
            .filter_map(|c| c.budget_allocated)
            .sum();
        let total_spent: Decimal = children.iter().map(|c| c.budget_spent).sum();

        let util_pct = if total_allocated.is_zero() { 0.0 }
            else {
                (total_spent / total_allocated).to_f64().unwrap_or(0.0) as f32 * 100.0
            };

        let avg_health = if child_health_scores.is_empty() { 0.0 }
            else { child_health_scores.iter().sum::<f32>() / child_health_scores.len() as f32 };

        let active_count    = children.iter().filter(|c| c.status == "Active").count() as u32;
        let completed_count = children.iter().filter(|c| c.status == "Completed").count() as u32;
        let at_risk_count   = children.iter()
            .filter(|c| !c.risk_flags.is_empty() && c.risk_flags.iter().any(|r| !r.mitigated))
            .count() as u32;

        SubPortfolioRollupOutput {
            total_budget_allocated: total_allocated,
            total_budget_spent:     total_spent,
            budget_utilization_pct: util_pct,
            total_revenue:          Decimal::ZERO,
            total_expenses:         total_spent,
            total_profit:           Decimal::ZERO,
            avg_health_score:       avg_health,
            active_count,
            completed_count,
            at_risk_count,
            component_count: children.len() as u32,
            computed_at: Utc::now(),
        }
    }
}

// =============================================================================
// §6 — ResourceUtilisation Model
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocationRecord {
    pub resource_id:    ComponentId,
    pub allocated_units: f32,
    pub available_units: f32,
    pub period:          String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilisationOutput {
    pub utilisation_pct:    f32,
    pub is_overrun:         bool,
    pub is_underutilised:   bool,
    pub over_allocated:     bool,
    pub computed_at:        DateTime<Utc>,
}

impl ResourceAllocationRecord {
    pub fn utilisation_pct(&self) -> f32 {
        if self.available_units == 0.0 { return 0.0; }
        (self.allocated_units / self.available_units * 100.0).clamp(0.0, 999.0)
    }
    pub fn is_overrun(&self)       -> bool { self.utilisation_pct() > 100.0 }
    pub fn is_underutilised(&self) -> bool { self.utilisation_pct() < 30.0 }
}

pub struct ResourceUtilisationModel;

impl ResourceUtilisationModel {
    pub fn compute(records: &[ResourceAllocationRecord]) -> ResourceUtilisationOutput {
        if records.is_empty() {
            return ResourceUtilisationOutput {
                utilisation_pct: 0.0, is_overrun: false,
                is_underutilised: true, over_allocated: false,
                computed_at: Utc::now(),
            };
        }
        let avg = records.iter().map(|r| r.utilisation_pct()).sum::<f32>() / records.len() as f32;
        ResourceUtilisationOutput {
            utilisation_pct:  avg,
            is_overrun:       avg > 100.0,
            is_underutilised: avg < 30.0,
            over_allocated:   records.iter().any(|r| r.is_overrun()),
            computed_at:        Utc::now(),
        }
    }
}
