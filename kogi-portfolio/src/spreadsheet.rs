use std::collections::{HashMap, HashSet};

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type ComponentId = Uuid;
pub type EntityId = Uuid;
pub type PolicyId = Uuid;
pub type AccountId = Uuid;
pub type ToolBoxId = Uuid;
pub type SheetId = String;
pub type ColumnId = String;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CellValue {
    Text(String),
    Number(f64),
    Decimal(Decimal),
    Currency { amount: Decimal, code: String },
    Percent(f64),
    Date(NaiveDate),
    DateTime(DateTime<Utc>),
    Bool(bool),
    Enum(String),
    MultiEnum(Vec<String>),
    Relation(ComponentId),
    MultiRelation(Vec<ComponentId>),
    User(EntityId),
    MultiUser(Vec<EntityId>),
    Tags(Vec<String>),
    Json(serde_json::Value),
    Empty,
}

impl Default for CellValue {
    fn default() -> Self { Self::Empty }
}

impl CellValue {
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            CellValue::Number(v) => Some(*v),
            CellValue::Decimal(v) => v.to_f64(),
            CellValue::Percent(v) => Some(*v),
            CellValue::Currency { amount, .. } => amount.to_f64(),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<String> {
        match self {
            CellValue::Text(v) => Some(v.clone()),
            CellValue::Enum(v) => Some(v.clone()),
            CellValue::Number(v) => Some(v.to_string()),
            CellValue::Decimal(v) => Some(v.to_string()),
            CellValue::Percent(v) => Some(v.to_string()),
            CellValue::Currency { amount, code } => Some(format!("{amount} {code}")),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnType {
    Text,
    Number,
    Decimal,
    Currency,
    Percent,
    Date,
    DateTime,
    Bool,
    Enum,
    MultiEnum,
    Relation,
    MultiRelation,
    User,
    MultiUser,
    Tags,
    Json,
    Computed,
    AiSignal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnGroup {
    Identity,
    Lifecycle,
    Ownership,
    Schedule,
    Finance,
    Resource,
    Governance,
    Analytics,
    Relationships,
    Collaboration,
    Benefits,
    Gig,
    Investment,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDefinition {
    pub id: ColumnId,
    pub name: String,
    pub column_type: ColumnType,
    pub group: ColumnGroup,
    pub pinned: bool,
    pub computed: bool,
    pub ai_signal: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ColumnSchema {
    pub columns: Vec<ColumnDefinition>,
}

impl ColumnSchema {
    pub fn new(columns: Vec<ColumnDefinition>) -> Self {
        Self { columns }
    }

    pub fn get(&self, id: &str) -> Option<&ColumnDefinition> {
        self.columns.iter().find(|c| c.id == id)
    }

    pub fn pinned_columns(&self) -> impl Iterator<Item = &ColumnDefinition> {
        self.columns.iter().filter(|c| c.pinned)
    }

    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.columns.iter().map(|c| c.id.as_str())
    }
}

pub fn default_master_schema() -> ColumnSchema {
    ColumnSchema::new(vec![
        ColumnDefinition {
            id: "name".into(),
            name: "Name".into(),
            column_type: ColumnType::Text,
            group: ColumnGroup::Identity,
            pinned: true,
            computed: false,
            ai_signal: false,
        },
        ColumnDefinition {
            id: "component_type".into(),
            name: "Type".into(),
            column_type: ColumnType::Enum,
            group: ColumnGroup::Identity,
            pinned: true,
            computed: false,
            ai_signal: false,
        },
        ColumnDefinition {
            id: "status".into(),
            name: "Status".into(),
            column_type: ColumnType::Enum,
            group: ColumnGroup::Lifecycle,
            pinned: true,
            computed: false,
            ai_signal: false,
        },
        ColumnDefinition {
            id: "owners".into(),
            name: "Owners".into(),
            column_type: ColumnType::MultiUser,
            group: ColumnGroup::Ownership,
            pinned: true,
            computed: false,
            ai_signal: false,
        },
        ColumnDefinition {
            id: "budget_allocated".into(),
            name: "Budget".into(),
            column_type: ColumnType::Decimal,
            group: ColumnGroup::Finance,
            pinned: false,
            computed: false,
            ai_signal: false,
        },
        ColumnDefinition {
            id: "budget_remaining".into(),
            name: "Budget Remaining".into(),
            column_type: ColumnType::Computed,
            group: ColumnGroup::Finance,
            pinned: false,
            computed: true,
            ai_signal: false,
        },
        ColumnDefinition {
            id: "budget_utilization_pct".into(),
            name: "Budget Utilization %".into(),
            column_type: ColumnType::Computed,
            group: ColumnGroup::Finance,
            pinned: false,
            computed: true,
            ai_signal: false,
        },
        ColumnDefinition {
            id: "earnings_net".into(),
            name: "Earnings Net".into(),
            column_type: ColumnType::Computed,
            group: ColumnGroup::Finance,
            pinned: false,
            computed: true,
            ai_signal: false,
        },
        ColumnDefinition {
            id: "due_date".into(),
            name: "Due".into(),
            column_type: ColumnType::Date,
            group: ColumnGroup::Schedule,
            pinned: false,
            computed: false,
            ai_signal: false,
        },
    ])
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioRow {
    pub component_id: ComponentId,
    pub slug: String,
    pub component_type: String,
    pub item_type: Option<String>,
    pub container_type: Option<String>,
    pub name: String,
    pub display_name: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,

    pub status: String,
    pub state: String,
    pub visibility: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub archived_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,

    pub owners: Vec<EntityId>,
    pub editors: Vec<EntityId>,
    pub viewers: Vec<EntityId>,
    pub contributors: Vec<EntityId>,
    pub watchers: Vec<EntityId>,
    pub created_by: EntityId,
    pub updated_by: Option<EntityId>,

    pub tags: Vec<String>,
    pub hashtags: Vec<String>,
    pub topics: Vec<String>,
    pub search_keywords: Vec<String>,
    pub domain: Option<String>,

    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
    pub progress_pct: f64,

    pub budget_allocated: Option<Decimal>,
    pub budget_spent: Decimal,
    pub currency: Option<String>,
    pub rate: Option<Decimal>,
    pub revenue: Decimal,
    pub expenses: Decimal,

    pub resource_units_total: Option<f64>,
    pub resource_units_allocated: f64,
    pub resource_type: Option<String>,

    pub policy_ids: Vec<PolicyId>,
    pub governance_model: Option<String>,
    pub approval_status: Option<String>,
    pub risk_flags: Vec<String>,

    pub views: u64,
    pub clicks: u64,
    pub shares: u64,
    pub followers: u64,

    pub parent_id: Option<ComponentId>,
    pub child_ids: Vec<ComponentId>,
    pub dependency_ids: Vec<ComponentId>,
    pub dependent_ids: Vec<ComponentId>,
    pub linked_ids: Vec<ComponentId>,

    pub toolbox_ids: Vec<ToolBoxId>,

    pub ext: HashMap<String, serde_json::Value>,
}

impl PortfolioRow {
    pub fn new(component_id: ComponentId, component_type: impl Into<String>, name: impl Into<String>, created_by: EntityId) -> Self {
        let now = Utc::now();
        Self {
            component_id,
            slug: String::new(),
            component_type: component_type.into(),
            item_type: None,
            container_type: None,
            name: name.into(),
            display_name: None,
            icon: None,
            color: None,
            status: "Draft".into(),
            state: "Idle".into(),
            visibility: "Private".into(),
            created_at: now,
            updated_at: now,
            archived_at: None,
            deleted_at: None,
            owners: vec![created_by],
            editors: vec![],
            viewers: vec![],
            contributors: vec![],
            watchers: vec![],
            created_by,
            updated_by: None,
            tags: vec![],
            hashtags: vec![],
            topics: vec![],
            search_keywords: vec![],
            domain: None,
            start_date: None,
            end_date: None,
            due_date: None,
            progress_pct: 0.0,
            budget_allocated: None,
            budget_spent: Decimal::ZERO,
            currency: None,
            rate: None,
            revenue: Decimal::ZERO,
            expenses: Decimal::ZERO,
            resource_units_total: None,
            resource_units_allocated: 0.0,
            resource_type: None,
            policy_ids: vec![],
            governance_model: None,
            approval_status: None,
            risk_flags: vec![],
            views: 0,
            clicks: 0,
            shares: 0,
            followers: 0,
            parent_id: None,
            child_ids: vec![],
            dependency_ids: vec![],
            dependent_ids: vec![],
            linked_ids: vec![],
            toolbox_ids: vec![],
            ext: HashMap::new(),
        }
    }

    pub fn primary_type(&self) -> Option<String> {
        self.item_type.clone().or_else(|| self.container_type.clone())
    }

    pub fn budget_remaining(&self) -> Option<Decimal> {
        let allocated = self.budget_allocated?;
        Some(allocated - self.budget_spent)
    }

    pub fn budget_utilization_pct(&self) -> Option<f64> {
        let allocated = self.budget_allocated?.to_f64()?;
        if allocated == 0.0 { return Some(0.0); }
        Some((self.budget_spent.to_f64()? / allocated) * 100.0)
    }

    pub fn earnings_net(&self) -> Decimal {
        self.revenue - self.expenses
    }

    pub fn value_for(&self, column: &str) -> Option<CellValue> {
        match column {
            "name" => Some(CellValue::Text(self.name.clone())),
            "component_type" => Some(CellValue::Enum(self.component_type.clone())),
            "status" => Some(CellValue::Enum(self.status.clone())),
            "state" => Some(CellValue::Enum(self.state.clone())),
            "visibility" => Some(CellValue::Enum(self.visibility.clone())),
            "owners" => Some(CellValue::MultiUser(self.owners.clone())),
            "tags" => Some(CellValue::Tags(self.tags.clone())),
            "budget_allocated" => self.budget_allocated.map(CellValue::Decimal),
            "budget_spent" => Some(CellValue::Decimal(self.budget_spent)),
            "budget_remaining" => self.budget_remaining().map(CellValue::Decimal),
            "budget_utilization_pct" => self.budget_utilization_pct().map(CellValue::Percent),
            "earnings_net" => Some(CellValue::Decimal(self.earnings_net())),
            "due_date" => self.due_date.map(CellValue::Date),
            "progress_pct" => Some(CellValue::Percent(self.progress_pct)),
            "created_at" => Some(CellValue::DateTime(self.created_at)),
            "updated_at" => Some(CellValue::DateTime(self.updated_at)),
            _ => self.ext.get(column).map(|v| CellValue::Json(v.clone())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetDefinition {
    pub id: SheetId,
    pub name: String,
    pub description: String,
    pub row_types: Vec<String>,
    pub column_schema: ColumnSchema,
    pub is_builtin: bool,
}

impl SheetDefinition {
    pub fn builtin(id: impl Into<SheetId>, name: impl Into<String>, description: impl Into<String>, row_types: Vec<String>, schema: ColumnSchema) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            row_types,
            column_schema: schema,
            is_builtin: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SheetRegistry {
    order: Vec<SheetId>,
    sheets: HashMap<SheetId, SheetDefinition>,
}

impl SheetRegistry {
    pub fn empty() -> Self { Self::default() }

    pub fn new() -> Self {
        let mut registry = Self::default();
        register_builtin_sheets(&mut registry);
        registry
    }

    pub fn register(&mut self, sheet: SheetDefinition) {
        let id = sheet.id.clone();
        if !self.sheets.contains_key(&id) {
            self.order.push(id.clone());
        }
        self.sheets.insert(id, sheet);
    }

    pub fn get(&self, id: &str) -> Option<&SheetDefinition> {
        self.sheets.get(id)
    }

    pub fn all(&self) -> Vec<&SheetDefinition> {
        self.order.iter().filter_map(|id| self.sheets.get(id)).collect()
    }

    pub fn order(&self) -> Vec<SheetId> { self.order.clone() }

    pub fn set_order(&mut self, order: Vec<SheetId>) {
        self.order = order;
    }
}

fn register_builtin_sheets(registry: &mut SheetRegistry) {
    let schema = default_master_schema();
    registry.register(SheetDefinition::builtin("SHT-001", "Master", "All components", vec![], schema.clone()));
    registry.register(SheetDefinition::builtin("SHT-002", "Portfolios", "Portfolio roots", vec!["Portfolio".into(), "SubPortfolio".into()], schema.clone()));
    registry.register(SheetDefinition::builtin("SHT-003", "Programs", "Programs", vec!["Program".into()], schema.clone()));
    registry.register(SheetDefinition::builtin("SHT-004", "Projects", "Projects", vec!["Project".into()], schema.clone()));
    registry.register(SheetDefinition::builtin("SHT-005", "Tasks", "Tasks", vec!["Task".into()], schema.clone()));
    registry.register(SheetDefinition::builtin("SHT-006", "Resources", "Resources", vec!["Resource".into()], schema.clone()));
    registry.register(SheetDefinition::builtin("SHT-007", "Assets", "Assets", vec!["Asset".into()], schema.clone()));
    registry.register(SheetDefinition::builtin("SHT-008", "Artifacts", "Artifacts", vec!["Artifact".into()], schema.clone()));
    registry.register(SheetDefinition::builtin("SHT-009", "Benefits", "Benefit accounts", vec!["BenefitAccount".into()], schema.clone()));
    registry.register(SheetDefinition::builtin("SHT-010", "Gigs", "Gigs", vec!["Gig".into()], schema.clone()));
    registry.register(SheetDefinition::builtin("SHT-011", "Contracts", "Contracts", vec!["Contract".into()], schema.clone()));
    registry.register(SheetDefinition::builtin("SHT-012", "Jobs", "Jobs", vec!["Job".into()], schema.clone()));
    registry.register(SheetDefinition::builtin("SHT-013", "Campaigns", "Campaigns", vec!["Campaign".into()], schema.clone()));
    registry.register(SheetDefinition::builtin("SHT-014", "Grants", "Grants", vec!["Grant".into()], schema.clone()));
    registry.register(SheetDefinition::builtin("SHT-015", "Investments", "Investments", vec!["Investment".into()], schema.clone()));
    registry.register(SheetDefinition::builtin("SHT-016", "Profiles", "Profiles", vec!["Profile".into()], schema.clone()));
    registry.register(SheetDefinition::builtin("SHT-017", "Organizations", "Organizations", vec!["Organization".into()], schema.clone()));
    registry.register(SheetDefinition::builtin("SHT-018", "Workspaces", "Workspaces", vec!["Workspace".into()], schema.clone()));
    registry.register(SheetDefinition::builtin("SHT-019", "Finance", "Financial items", vec!["Asset".into(), "BenefitAccount".into(), "Investment".into()], schema.clone()));
    registry.register(SheetDefinition::builtin("SHT-020", "Governance", "Governance", vec![], schema.clone()));
    registry.register(SheetDefinition::builtin("SHT-021", "Analytics", "Analytics", vec![], schema.clone()));
    registry.register(SheetDefinition::builtin("SHT-022", "Collaboration", "Shared work", vec![], schema.clone()));
    registry.register(SheetDefinition::builtin("SHT-023", "Marketplace", "Marketplace items", vec![], schema.clone()));
    registry.register(SheetDefinition::builtin("SHT-024", "Community", "Community", vec![], schema.clone()));
    registry.register(SheetDefinition::builtin("SHT-025", "Exchange", "Exchange", vec![], schema.clone()));
    registry.register(SheetDefinition::builtin("SHT-026", "Bank", "Banking", vec![], schema.clone()));
    registry.register(SheetDefinition::builtin("SHT-027", "Developer", "Developer", vec![], schema.clone()));
    registry.register(SheetDefinition::builtin("SHT-028", "Configuration", "Configuration", vec![], schema.clone()));
    registry.register(SheetDefinition::builtin("SHT-029", "Link Network", "Link network", vec![], schema.clone()));
    registry.register(SheetDefinition::builtin("SHT-030", "Custom", "Custom sheet", vec![], schema));
}

#[derive(Debug, Clone, Default)]
pub struct RowStore {
    rows: HashMap<ComponentId, PortfolioRow>,
}

impl RowStore {
    pub fn insert(&mut self, row: PortfolioRow) { self.rows.insert(row.component_id, row); }
    pub fn get(&self, id: &ComponentId) -> Option<&PortfolioRow> { self.rows.get(id) }
    pub fn get_mut(&mut self, id: &ComponentId) -> Option<&mut PortfolioRow> { self.rows.get_mut(id) }
    pub fn remove(&mut self, id: &ComponentId) -> Option<PortfolioRow> { self.rows.remove(id) }
    pub fn iter(&self) -> impl Iterator<Item = &PortfolioRow> { self.rows.values() }
}

#[derive(Debug, Clone)]
struct CellMeta {
    updated_by: String,
    updated_at: DateTime<Utc>,
    ttl_seconds: Option<u64>,
}

#[derive(Debug, Clone, Default)]
pub struct CellStore {
    cells: HashMap<(ComponentId, ColumnId), CellValue>,
    meta: HashMap<(ComponentId, ColumnId), CellMeta>,
}

impl CellStore {
    pub fn set_cell(&mut self, row_id: ComponentId, column: impl Into<ColumnId>, value: CellValue, actor: impl Into<String>, ttl_seconds: Option<u64>) {
        let col = column.into();
        let now = Utc::now();
        self.cells.insert((row_id, col.clone()), value);
        self.meta.insert((row_id, col), CellMeta { updated_by: actor.into(), updated_at: now, ttl_seconds });
    }

    pub fn get_cell(&self, row_id: &ComponentId, column: &str) -> Option<&CellValue> {
        self.cells.get(&(*row_id, column.to_string()))
    }

    pub fn invalidate_row(&mut self, row_id: &ComponentId) {
        self.cells.retain(|(rid, _), _| rid != row_id);
        self.meta.retain(|(rid, _), _| rid != row_id);
    }

    pub fn entries_for_row(&self, row_id: &ComponentId) -> Vec<(&ColumnId, &CellValue)> {
        self.cells.iter().filter(|((rid, _), _)| rid == row_id).map(|((_, col), val)| (col, val)).collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&(ComponentId, ColumnId), &CellValue)> {
        self.cells.iter()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub component_id: ComponentId,
    pub component_type: String,
    pub item_type: Option<String>,
    pub container_type: Option<String>,
    pub name: String,
    pub status: String,
    pub owners: Vec<EntityId>,
    pub tags: Vec<String>,
    pub domain: Option<String>,
    pub parent_id: Option<ComponentId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RegistryEntry {
    pub fn from_row(row: &PortfolioRow) -> Self {
        Self {
            component_id: row.component_id,
            component_type: row.component_type.clone(),
            item_type: row.item_type.clone(),
            container_type: row.container_type.clone(),
            name: row.name.clone(),
            status: row.status.clone(),
            owners: row.owners.clone(),
            tags: row.tags.clone(),
            domain: row.domain.clone(),
            parent_id: row.parent_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ComponentRegistry {
    entries: HashMap<ComponentId, RegistryEntry>,
    by_owner: HashMap<EntityId, HashSet<ComponentId>>,
    by_type: HashMap<String, HashSet<ComponentId>>,
    by_status: HashMap<String, HashSet<ComponentId>>,
    by_tag: HashMap<String, HashSet<ComponentId>>,
    by_domain: HashMap<String, HashSet<ComponentId>>,
}

impl ComponentRegistry {
    pub fn register_row(&mut self, row: &PortfolioRow) {
        let entry = RegistryEntry::from_row(row);
        self.entries.insert(entry.component_id, entry.clone());

        for owner in &entry.owners {
            self.by_owner.entry(*owner).or_default().insert(entry.component_id);
        }
        if let Some(item_type) = &entry.item_type {
            self.by_type.entry(item_type.clone()).or_default().insert(entry.component_id);
        }
        if let Some(container_type) = &entry.container_type {
            self.by_type.entry(container_type.clone()).or_default().insert(entry.component_id);
        }
        self.by_status.entry(entry.status.clone()).or_default().insert(entry.component_id);
        for tag in &entry.tags {
            self.by_tag.entry(tag.clone()).or_default().insert(entry.component_id);
        }
        if let Some(domain) = &entry.domain {
            self.by_domain.entry(domain.clone()).or_default().insert(entry.component_id);
        }
    }

    pub fn get(&self, id: &ComponentId) -> Option<&RegistryEntry> { self.entries.get(id) }

    pub fn by_owner(&self, owner: &EntityId) -> Vec<ComponentId> {
        self.by_owner.get(owner).map(|s| s.iter().copied().collect()).unwrap_or_default()
    }

    pub fn by_tag(&self, tag: &str) -> Vec<ComponentId> {
        self.by_tag.get(tag).map(|s| s.iter().copied().collect()).unwrap_or_default()
    }

    pub fn by_status(&self, status: &str) -> Vec<ComponentId> {
        self.by_status.get(status).map(|s| s.iter().copied().collect()).unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Aggregation {
    Count,
    Sum,
    Avg,
    Min,
    Max,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewGroup {
    pub column_id: ColumnId,
    pub aggregations: Vec<(ColumnId, Aggregation)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupBucket {
    pub key: String,
    pub rows: Vec<PortfolioRow>,
    pub aggregates: HashMap<ColumnId, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PivotConfig {
    pub row_column: ColumnId,
    pub column_column: ColumnId,
    pub value_column: ColumnId,
    pub aggregation: Aggregation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PivotTable {
    pub row_keys: Vec<String>,
    pub column_keys: Vec<String>,
    pub values: HashMap<(String, String), f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputedColumnKind {
    BudgetRemaining,
    BudgetUtilizationPct,
    EarningsNet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputedColumnSpec {
    pub column_id: ColumnId,
    pub kind: ComputedColumnKind,
}

#[derive(Debug, Clone, Default)]
pub struct ComputationEngine {
    pub columns: Vec<ComputedColumnSpec>,
}

impl ComputationEngine {
    pub fn default_columns() -> Self {
        Self {
            columns: vec![
                ComputedColumnSpec { column_id: "budget_remaining".into(), kind: ComputedColumnKind::BudgetRemaining },
                ComputedColumnSpec { column_id: "budget_utilization_pct".into(), kind: ComputedColumnKind::BudgetUtilizationPct },
                ComputedColumnSpec { column_id: "earnings_net".into(), kind: ComputedColumnKind::EarningsNet },
            ],
        }
    }

    pub fn compute_for_row(&self, row: &PortfolioRow, cells: &CellStore) -> Vec<(ColumnId, CellValue)> {
        let mut out = Vec::new();
        for spec in &self.columns {
            let value = match spec.kind {
                ComputedColumnKind::BudgetRemaining => row.budget_remaining().map(CellValue::Decimal),
                ComputedColumnKind::BudgetUtilizationPct => row.budget_utilization_pct().map(CellValue::Percent),
                ComputedColumnKind::EarningsNet => Some(CellValue::Decimal(row.earnings_net())),
            };
            if let Some(val) = value {
                out.push((spec.column_id.clone(), val));
            }
        }
        // Allow computed columns to reference existing cell overrides if needed.
        let _ = cells;
        out
    }

    pub fn writeback(&self, workbook: &mut SpreadsheetWorkbook, actor: impl Into<String>) {
        let actor_str = actor.into();
        let row_ids: Vec<ComponentId> = workbook.row_store.iter().map(|row| row.component_id).collect();
        for row_id in row_ids {
            if let Some(row) = workbook.row_store.get(&row_id).cloned() {
                let computed = self.compute_for_row(&row, &workbook.cell_store);
                for (column_id, value) in computed {
                    workbook.cell_store.set_cell(row_id, column_id, value, actor_str.clone(), Some(300));
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterPredicate {
    Equals(String, CellValue),
    Contains(String, String),
    GreaterThan(String, f64),
    LessThan(String, f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewSort {
    pub column_id: ColumnId,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewDefinition {
    pub sheet_id: SheetId,
    pub filters: Vec<FilterPredicate>,
    pub sorts: Vec<ViewSort>,
    pub limit: Option<usize>,
    pub offset: usize,
}

pub struct ViewEngine;

impl ViewEngine {
    pub fn apply(rows: &[PortfolioRow], cells: &CellStore, view: &ViewDefinition) -> Vec<PortfolioRow> {
        let mut filtered: Vec<PortfolioRow> = rows.iter().cloned().filter(|row| {
            view.filters.iter().all(|pred| evaluate_filter(pred, row, cells))
        }).collect();

        for sort in view.sorts.iter().rev() {
            filtered.sort_by(|a, b| compare_by_column(a, b, cells, sort));
        }

        let start = view.offset.min(filtered.len());
        let end = match view.limit {
            Some(limit) => (start + limit).min(filtered.len()),
            None => filtered.len(),
        };
        filtered[start..end].to_vec()
    }

    pub fn group(rows: &[PortfolioRow], cells: &CellStore, group: &ViewGroup) -> Vec<GroupBucket> {
        let mut buckets: HashMap<String, Vec<PortfolioRow>> = HashMap::new();
        for row in rows {
            let key = cells.get_cell(&row.component_id, &group.column_id)
                .cloned()
                .or_else(|| row.value_for(&group.column_id))
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| "null".to_owned());
            buckets.entry(key).or_default().push(row.clone());
        }

        buckets.into_iter().map(|(key, rows)| {
            let aggregates = compute_aggregates(&rows, cells, &group.aggregations);
            GroupBucket { key, rows, aggregates }
        }).collect()
    }

    pub fn pivot(rows: &[PortfolioRow], cells: &CellStore, pivot: &PivotConfig) -> PivotTable {
        let mut row_keys = HashSet::new();
        let mut col_keys = HashSet::new();
        let mut values: HashMap<(String, String), Vec<f64>> = HashMap::new();

        for row in rows {
            let row_key = cells.get_cell(&row.component_id, &pivot.row_column)
                .cloned()
                .or_else(|| row.value_for(&pivot.row_column))
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| "null".to_owned());
            let col_key = cells.get_cell(&row.component_id, &pivot.column_column)
                .cloned()
                .or_else(|| row.value_for(&pivot.column_column))
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| "null".to_owned());
            let value = cells.get_cell(&row.component_id, &pivot.value_column)
                .cloned()
                .or_else(|| row.value_for(&pivot.value_column))
                .and_then(|v| v.as_f64());

            row_keys.insert(row_key.clone());
            col_keys.insert(col_key.clone());
            if let Some(val) = value {
                values.entry((row_key, col_key)).or_default().push(val);
            }
        }

        let mut output = HashMap::new();
        for ((row_key, col_key), series) in values {
            let agg = aggregate_series(&series, &pivot.aggregation);
            output.insert((row_key, col_key), agg);
        }

        let mut row_keys: Vec<String> = row_keys.into_iter().collect();
        row_keys.sort();
        let mut column_keys: Vec<String> = col_keys.into_iter().collect();
        column_keys.sort();

        PivotTable {
            row_keys,
            column_keys,
            values: output,
        }
    }
}

fn evaluate_filter(pred: &FilterPredicate, row: &PortfolioRow, cells: &CellStore) -> bool {
    let value = cells.get_cell(&row.component_id, match pred {
        FilterPredicate::Equals(col, _) => col,
        FilterPredicate::Contains(col, _) => col,
        FilterPredicate::GreaterThan(col, _) => col,
        FilterPredicate::LessThan(col, _) => col,
    }).cloned().or_else(|| row.value_for(match pred {
        FilterPredicate::Equals(col, _) => col,
        FilterPredicate::Contains(col, _) => col,
        FilterPredicate::GreaterThan(col, _) => col,
        FilterPredicate::LessThan(col, _) => col,
    }));

    match pred {
        FilterPredicate::Equals(_, target) => value.as_ref() == Some(target),
        FilterPredicate::Contains(_, needle) => value.and_then(|v| v.as_str()).map(|s| s.contains(needle)).unwrap_or(false),
        FilterPredicate::GreaterThan(_, cmp) => value.and_then(|v| v.as_f64()).map(|v| v > *cmp).unwrap_or(false),
        FilterPredicate::LessThan(_, cmp) => value.and_then(|v| v.as_f64()).map(|v| v < *cmp).unwrap_or(false),
    }
}

fn compare_by_column(a: &PortfolioRow, b: &PortfolioRow, cells: &CellStore, sort: &ViewSort) -> std::cmp::Ordering {
    let av = cells.get_cell(&a.component_id, &sort.column_id).cloned().or_else(|| a.value_for(&sort.column_id));
    let bv = cells.get_cell(&b.component_id, &sort.column_id).cloned().or_else(|| b.value_for(&sort.column_id));

    let ordering = match (av, bv) {
        (Some(va), Some(vb)) => {
            match (va.as_f64(), vb.as_f64()) {
                (Some(a_num), Some(b_num)) => a_num.partial_cmp(&b_num).unwrap_or(std::cmp::Ordering::Equal),
                _ => va.as_str().cmp(&vb.as_str()),
            }
        }
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    };

    match sort.direction {
        SortDirection::Asc => ordering,
        SortDirection::Desc => ordering.reverse(),
    }
}

fn compute_aggregates(rows: &[PortfolioRow], cells: &CellStore, aggregations: &[(ColumnId, Aggregation)]) -> HashMap<ColumnId, f64> {
    let mut output = HashMap::new();
    for (column_id, agg) in aggregations {
        let mut series = Vec::new();
        for row in rows {
            if let Some(val) = cells.get_cell(&row.component_id, column_id)
                .cloned()
                .or_else(|| row.value_for(column_id))
                .and_then(|v| v.as_f64())
            {
                series.push(val);
            }
        }
        let result = aggregate_series(&series, agg);
        output.insert(column_id.clone(), result);
    }
    output
}

fn aggregate_series(series: &[f64], agg: &Aggregation) -> f64 {
    if series.is_empty() {
        return 0.0;
    }
    match agg {
        Aggregation::Count => series.len() as f64,
        Aggregation::Sum => series.iter().sum(),
        Aggregation::Avg => series.iter().sum::<f64>() / series.len() as f64,
        Aggregation::Min => series.iter().cloned().fold(f64::INFINITY, f64::min),
        Aggregation::Max => series.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
    }
}

#[derive(Debug, Clone)]
pub struct SpreadsheetWorkbook {
    pub workbook_id: Uuid,
    pub owner_id: EntityId,
    pub name: String,
    pub row_store: RowStore,
    pub cell_store: CellStore,
    pub sheets: SheetRegistry,
    pub registry: ComponentRegistry,
}

impl SpreadsheetWorkbook {
    pub fn new(owner_id: EntityId, name: impl Into<String>) -> Self {
        Self {
            workbook_id: Uuid::new_v4(),
            owner_id,
            name: name.into(),
            row_store: RowStore::default(),
            cell_store: CellStore::default(),
            sheets: SheetRegistry::new(),
            registry: ComponentRegistry::default(),
        }
    }

    pub fn with_id(workbook_id: Uuid, owner_id: EntityId, name: impl Into<String>) -> Self {
        Self {
            workbook_id,
            owner_id,
            name: name.into(),
            row_store: RowStore::default(),
            cell_store: CellStore::default(),
            sheets: SheetRegistry::empty(),
            registry: ComponentRegistry::default(),
        }
    }

    pub fn upsert_row(&mut self, row: PortfolioRow) {
        self.registry.register_row(&row);
        self.row_store.insert(row);
    }

    pub fn get_row(&self, id: &ComponentId) -> Option<&PortfolioRow> {
        self.row_store.get(id)
    }

    pub fn rows_for_sheet(&self, sheet_id: &str) -> Vec<&PortfolioRow> {
        let sheet = self.sheets.get(sheet_id);
        match sheet {
            Some(def) if !def.row_types.is_empty() => self.row_store.iter().filter(|row| {
                row.primary_type().map(|t| def.row_types.contains(&t)).unwrap_or(false)
            }).collect(),
            _ => self.row_store.iter().collect(),
        }
    }

    pub fn set_cell(&mut self, row_id: ComponentId, column: impl Into<ColumnId>, value: CellValue, actor: impl Into<String>, ttl_seconds: Option<u64>) {
        self.cell_store.set_cell(row_id, column, value, actor, ttl_seconds);
    }

    pub fn get_cell(&self, row_id: &ComponentId, column: &str) -> Option<&CellValue> {
        self.cell_store.get_cell(row_id, column)
    }

    pub fn view(&self, view: &ViewDefinition) -> Vec<PortfolioRow> {
        let rows: Vec<PortfolioRow> = self.rows_for_sheet(&view.sheet_id).into_iter().cloned().collect();
        ViewEngine::apply(&rows, &self.cell_store, view)
    }

    pub fn view_grouped(&self, view: &ViewDefinition, group: &ViewGroup) -> Vec<GroupBucket> {
        let rows: Vec<PortfolioRow> = self.view(view);
        ViewEngine::group(&rows, &self.cell_store, group)
    }

    pub fn view_pivot(&self, view: &ViewDefinition, pivot: &PivotConfig) -> PivotTable {
        let rows: Vec<PortfolioRow> = self.view(view);
        ViewEngine::pivot(&rows, &self.cell_store, pivot)
    }

    pub fn write_computed_columns(&mut self, engine: &ComputationEngine, actor: impl Into<String>) {
        engine.writeback(self, actor);
    }
}
