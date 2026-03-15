// =============================================================================
//  portfolio_spreadsheet_substrate.rs — Kogi OS · KPSS v3.0
//  Portfolio Spreadsheet Substrate
//
//  The low-level data engine of the Portfolio Management System.
//  Every entity in the Kogi ecosystem is a PortfolioRow in this universal,
//  scalable, configurable spreadsheet. This module provides:
//
//    · PortfolioRow        — universal typed row; superset of all column groups
//    · CellValue           — sum type for every storable value in a cell
//    · ColumnType          — full column type taxonomy (16 types)
//    · ColumnSchema        — ordered, typed column definitions per sheet
//    · ColumnGroup         — named logical groupings of columns
//    · CellStore           — indexed in-memory cell storage (row×col → CellValue)
//    · SheetDefinition     — static metadata for one of the 30 canonical sheets
//    · SheetRegistry       — the catalog of all 30 sheets (SHT-001 … SHT-030)
//    · PortfolioWorkbook   — the root "workbook" container; owns all sheets
//    · RowStore            — the single master row store that all sheets view
//
//  Structural notes
//  ─────────────────
//  A PortfolioWorkbook is the direct Kogi equivalent of an Excel workbook.
//  A SheetDefinition is a named view over the RowStore; rows are not owned
//  by sheets. The CellStore holds every materialized (column-override) value,
//  augmenting the values already present on the PortfolioRow struct fields.
//  The split design lets derived/computed column values live in the CellStore
//  without touching the canonical PortfolioRow, and allows sheet-local custom
//  columns without polluting the universal schema.
//
//  @author  Kogi Team
//  @version 3.0.0
//  @license MIT
// =============================================================================

#![allow(dead_code)]

use std::collections::HashMap;
use chrono::{Date, DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Re-export or inline lightweight copies of the shared type aliases used here
// (full definitions live in portfolio_system.rs).
pub type ComponentId  = Uuid;
pub type UserId       = Uuid;
pub type EntityId     = Uuid;
pub type PolicyId     = Uuid;
pub type AccountId    = Uuid;
pub type InvoiceId    = Uuid;
pub type SprintId     = Uuid;
pub type ToolBoxId    = Uuid;
pub type CampaignId   = Uuid;
pub type MilestoneId  = Uuid;
pub type CapTableId   = Uuid;
pub type SheetId      = &'static str;
pub type ColumnId     = String;

// =============================================================================
// §1 — CELL VALUE  (sum type for every storable value)
// =============================================================================

/// A single cell value — the atomic unit of the spreadsheet.
/// Every CellValue variant maps 1-to-1 with a ColumnType.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CellValue {
    /// Plain text.
    Text(String),
    /// Arbitrary signed integer.
    Integer(i64),
    /// Arbitrary floating-point.
    Float(f64),
    /// High-precision decimal (currency, percentages).
    Decimal(Decimal),
    /// ISO 4217 currency amount + code.
    Currency { amount: Decimal, code: String },
    /// Percentage 0.0–100.0.
    Percent(f64),
    /// Calendar date (no time component).
    Date(NaiveDate),
    /// Full UTC timestamp.
    DateTime(DateTime<Utc>),
    /// Elapsed time in seconds.
    Duration(u64),
    /// Single-value enum variant.
    Enum(String),
    /// Multi-value enum set.
    MultiEnum(Vec<String>),
    /// Reference to another PortfolioRow.
    Relation(ComponentId),
    /// Set of references to other PortfolioRows.
    MultiRelation(Vec<ComponentId>),
    /// User / entity reference.
    User(EntityId),
    /// Set of user / entity references.
    MultiUser(Vec<EntityId>),
    /// Tag list.
    Tags(Vec<String>),
    /// Boolean (renders as checkbox).
    Bool(bool),
    /// Rich-text Markdown.
    RichText(String),
    /// File reference id.
    FileRef(Uuid),
    /// Web URL.
    Url(String),
    /// Engine-computed value; serialized for caching.
    Computed(serde_json::Value),
    /// AI / engine signal (score, flag, recommendation).
    AiSignal(serde_json::Value),
    /// Null / empty cell.
    Empty,
}

impl Default for CellValue {
    fn default() -> Self { CellValue::Empty }
}

impl CellValue {
    /// Return true if the cell contains no meaningful data.
    pub fn is_empty(&self) -> bool { matches!(self, CellValue::Empty) }

    /// Attempt to coerce to f64 for aggregation.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            CellValue::Integer(n)    => Some(*n as f64),
            CellValue::Float(f)      => Some(*f),
            CellValue::Decimal(d)    => d.to_string().parse().ok(),
            CellValue::Currency { amount, .. } => amount.to_string().parse().ok(),
            CellValue::Percent(p)    => Some(*p),
            CellValue::Bool(b)       => Some(if *b { 1.0 } else { 0.0 }),
            _ => None,
        }
    }
}

// =============================================================================
// §2 — COLUMN TYPES
// =============================================================================

/// The type taxonomy for every column in the universal spreadsheet.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ColumnType {
    /// Free-form text; sortable, searchable, filterable.
    TextField,
    /// Integer; supports aggregation.
    NumberField,
    /// High-precision decimal with currency code.
    CurrencyField,
    /// Decimal 0–100; renders with % suffix.
    PercentField,
    /// Calendar date.
    DateField,
    /// Full timezone-aware timestamp.
    DateTimeField,
    /// Time span (seconds); renders as "3d 4h".
    DurationField,
    /// One-of a defined value set; renders as colored badge.
    EnumField { variants: Vec<String> },
    /// Set of enum values; "includes" filterable.
    MultiEnumField { variants: Vec<String> },
    /// Foreign-key reference to another PortfolioRow.
    RelationField,
    /// Set of foreign-key references.
    MultiRelationField,
    /// User / entity reference.
    UserField,
    /// Set of user / entity references.
    MultiUserField,
    /// Tag list.
    TagField,
    /// Boolean checkbox.
    BoolField,
    /// Markdown rich text.
    RichTextField,
    /// File attachment reference.
    FileRefField,
    /// Clickable URL.
    UrlField,
    /// Derived value; read-only; computed from other columns at query time.
    ComputedColumn { formula: Option<String> },
    /// AI / engine signal; written back via plugin writeback protocol.
    AiColumn { engine: String },
    /// User-defined formula column (power-user feature).
    FormulaColumn { expression: String },
    /// Derived from EventLog; shows audit trail values.
    AuditColumn,
    /// Aggregated engagement or performance metric.
    AnalyticsColumn { metric: String },
}

// =============================================================================
// §3 — COLUMN DEFINITION & SCHEMA
// =============================================================================

/// A single column definition within a sheet's ColumnSchema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDefinition {
    /// Machine identifier (snake_case, globally unique within a schema).
    pub id: ColumnId,
    /// User-facing label.
    pub label: String,
    /// Column type and associated metadata.
    pub col_type: ColumnType,
    /// Logical group this column belongs to.
    pub group: ColumnGroup,
    /// Width hint in pixels (for UI rendering).
    pub width_px: u16,
    /// Whether this column is always visible (freeze pane equivalent).
    pub pinned: bool,
    /// Whether this column is visible by default.
    pub visible_by_default: bool,
    /// Whether this column can be sorted.
    pub sortable: bool,
    /// Whether this column can be used in filters.
    pub filterable: bool,
    /// Whether this column can be used in grouping.
    pub groupable: bool,
    /// Whether values in this column can be aggregated across group rows.
    pub aggregatable: bool,
    /// Whether this column can be edited inline by users with Editor permission.
    pub editable: bool,
    /// Optional description / tooltip.
    pub description: Option<String>,
}

impl ColumnDefinition {
    pub fn new(id: impl Into<String>, label: impl Into<String>, col_type: ColumnType, group: ColumnGroup) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            col_type,
            group,
            width_px: 160,
            pinned: false,
            visible_by_default: true,
            sortable: true,
            filterable: true,
            groupable: false,
            aggregatable: false,
            editable: true,
            description: None,
        }
    }

    pub fn pinned(mut self) -> Self { self.pinned = true; self }
    pub fn hidden(mut self) -> Self { self.visible_by_default = false; self }
    pub fn readonly(mut self) -> Self { self.editable = false; self }
    pub fn aggregatable(mut self) -> Self { self.aggregatable = true; self }
    pub fn groupable(mut self) -> Self { self.groupable = true; self }
    pub fn width(mut self, px: u16) -> Self { self.width_px = px; self }
}

/// Ordered, typed column schema for a sheet.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ColumnSchema {
    pub columns: Vec<ColumnDefinition>,
    /// Index: column_id → position in columns vec.
    pub index: HashMap<ColumnId, usize>,
}

impl ColumnSchema {
    pub fn new(columns: Vec<ColumnDefinition>) -> Self {
        let index = columns.iter().enumerate().map(|(i, c)| (c.id.clone(), i)).collect();
        Self { columns, index }
    }

    pub fn get(&self, id: &str) -> Option<&ColumnDefinition> {
        self.index.get(id).map(|&i| &self.columns[i])
    }

    pub fn visible_columns(&self) -> impl Iterator<Item = &ColumnDefinition> {
        self.columns.iter().filter(|c| c.visible_by_default)
    }

    pub fn pinned_columns(&self) -> impl Iterator<Item = &ColumnDefinition> {
        self.columns.iter().filter(|c| c.pinned)
    }
}

// =============================================================================
// §4 — COLUMN GROUPS (logical groupings per SDD §3.1)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ColumnGroup {
    Identity,
    Taxonomy,
    Lifecycle,
    OwnershipAndUsers,
    Relationships,
    VersionControl,
    TimelineAndSchedule,
    FinancialAndBudget,
    ResourceAllocation,
    WorkAndDelivery,
    GovernanceAndPolicy,
    AnalyticsAndMetrics,
    VisibilityAndAccess,
    TagsAndDiscovery,
    AiAndEngineSignals,
    Collaboration,
    BenefitAccount,
    GigAndContract,
    Custom(String),
}

// =============================================================================
// §5 — UNIVERSAL ROW SCHEMA  (PortfolioRow)
// =============================================================================

/// The universal, typed row that represents *any* PortfolioComponent in the
/// master spreadsheet. Every field corresponds to one or more ColumnDefinitions.
///
/// Rules:
///   • All fields are Option where the column is not universal.
///   • ComputedColumn and AiColumn values are NOT stored here; they live in
///     CellStore so they can be invalidated / refreshed independently.
///   • The `ext` map absorbs custom columns and plugin-added fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioRow {
    // ── Identity (8 columns) ─────────────────────────────────────────────────
    pub component_id:    ComponentId,
    pub slug:            String,
    pub component_type:  String,           // "item" | "container"
    pub item_type:       Option<String>,   // Portfolio | Program | Project | …
    pub container_type:  Option<String>,   // Binder | Book | Record | …
    pub name:            String,
    pub display_name:    Option<String>,
    pub icon:            Option<String>,
    pub color:           Option<String>,   // hex #RRGGBB

    // ── Lifecycle (8 columns) ────────────────────────────────────────────────
    pub status:           String,                    // ComponentStatus
    pub state:            String,                    // ComponentState
    pub visibility:       String,                    // Visibility
    pub lifecycle_stage:  Option<String>,
    pub created_at:       DateTime<Utc>,
    pub updated_at:       DateTime<Utc>,
    pub archived_at:      Option<DateTime<Utc>>,
    pub deleted_at:       Option<DateTime<Utc>>,

    // ── Ownership & Users (12 columns) ───────────────────────────────────────
    pub owners:           Vec<EntityId>,
    pub editors:          Vec<EntityId>,
    pub viewers:          Vec<EntityId>,
    pub contributors:     Vec<EntityId>,
    pub watchers:         Vec<EntityId>,
    pub manager_id:       Option<EntityId>,
    pub created_by:       EntityId,
    pub updated_by:       Option<EntityId>,

    // ── Tags & Discovery ─────────────────────────────────────────────────────
    pub tags:             Vec<String>,
    pub search_keywords:  Vec<String>,
    pub domain:           Option<String>,

    // ── Timeline & Schedule (12 columns) ─────────────────────────────────────
    pub start_date:                  Option<NaiveDate>,
    pub end_date:                    Option<NaiveDate>,
    pub due_date:                    Option<NaiveDate>,
    pub estimated_duration_days:     Option<u32>,
    pub progress_pct:                f32,
    pub milestones:                  Vec<MilestoneRef>,
    pub sprint_refs:                 Vec<SprintId>,
    pub quarter:                     Option<String>,
    pub fiscal_year:                 Option<u32>,

    // ── Financial & Budget (14 columns) ──────────────────────────────────────
    pub budget_allocated:    Option<Decimal>,
    pub budget_spent:        Decimal,
    pub currency:            Option<String>,    // ISO 4217
    pub rate:                Option<Decimal>,
    pub rate_type:           Option<String>,    // RateType
    pub revenue:             Decimal,
    pub expenses:            Decimal,
    pub tax_category:        Option<String>,
    pub payment_terms:       Option<String>,
    pub invoice_refs:        Vec<InvoiceId>,
    pub bank_account_ref:    Option<AccountId>,

    // ── Resource Allocation (8 columns) ──────────────────────────────────────
    pub resource_units_total:     Option<f64>,
    pub resource_units_allocated: f64,
    pub resource_type:            Option<String>,
    pub allocation_pct:           f64,

    // ── Governance & Policy (9 columns) ──────────────────────────────────────
    pub policy_ids:          Vec<PolicyId>,
    pub governance_model:    Option<String>,
    pub approval_status:     Option<String>,
    pub compliance_flags:    Vec<String>,
    pub risk_flags:          Vec<RiskFlagRef>,
    pub last_reviewed_at:    Option<DateTime<Utc>>,
    pub charter_ref:         Option<ComponentId>,
    pub regulatory_tags:     Vec<String>,

    // ── Analytics & Metrics (non-computed; engine writes back via CellStore) ─
    pub views:     u64,
    pub clicks:    u64,
    pub shares:    u64,
    pub followers: u64,
    pub kpi_refs:  Vec<KpiRef>,
    pub okr_refs:  Vec<OkrRef>,

    // ── Toolboxes ─────────────────────────────────────────────────────────────
    pub toolbox_ids: Vec<ToolBoxId>,

    // ── BenefitAccount-specific columns ──────────────────────────────────────
    pub benefit_type:            Option<String>,  // BenefitType
    pub benefit_provider:        Option<String>,
    pub benefit_balance:         Option<Decimal>,
    pub benefit_ytd_contributions: Option<Decimal>,
    pub vesting_schedule:        Option<VestingScheduleRef>,
    pub tax_treatment:           Option<String>,  // PreTax | PostTax | TaxFree
    pub claim_workflow_id:       Option<Uuid>,
    pub pto_accrued_days:        Option<Decimal>,
    pub pto_used_ytd_days:       Option<Decimal>,

    // ── Gig / Contract / Job columns ────────────────────────────────────────
    pub engagement_type:      Option<String>,   // EngagementType
    pub platform:             Option<String>,
    pub client_ref:           Option<EntityId>,
    pub earnings_gross:       Option<Decimal>,
    pub tips:                 Option<Decimal>,
    pub expenses_gig:         Option<Decimal>,
    pub portable_contribution: Option<Decimal>,
    pub portable_contribution_pct: Option<Decimal>,
    pub mileage:              Option<Decimal>,
    pub payment_status:       Option<String>,   // PaymentStatus
    pub hours_worked:         Option<Decimal>,
    pub contract_status:      Option<String>,   // ContractStatus

    // ── Campaign / Grant / Investment columns ────────────────────────────────
    pub campaign_type:       Option<String>,
    pub funding_type:        Option<String>,
    pub target_amount:       Option<Decimal>,
    pub raised_amount:       Option<Decimal>,
    pub awarded_amount:      Option<Decimal>,
    pub disbursed_amount:    Option<Decimal>,
    pub investor_count:      Option<u32>,
    pub equity_offered_pct:  Option<Decimal>,
    pub close_date:          Option<NaiveDate>,
    pub linked_bank_account: Option<AccountId>,

    // ── Relationship / graph data ────────────────────────────────────────────
    pub parent_id:            Option<ComponentId>,
    pub child_ids:            Vec<ComponentId>,
    pub dependency_ids:       Vec<ComponentId>,
    pub dependent_ids:        Vec<ComponentId>,
    pub linked_ids:           Vec<ComponentId>,

    // ── Collaboration (shared portfolios) ────────────────────────────────────
    pub owner_count:               Option<u32>,
    pub contributor_count:         Option<u32>,
    pub contribution_types_open:   Vec<String>,
    pub pending_review_count:      Option<u32>,
    pub crowdresourcing_campaign_ref: Option<CampaignId>,
    pub review_policy:             Option<String>,
    pub attribution_weights:       HashMap<EntityId, f64>,
    pub last_contribution_date:    Option<DateTime<Utc>>,
    pub merge_conflicts:           u32,

    // ── Custom / plugin fields ───────────────────────────────────────────────
    /// Absorbs sheet-local custom columns and plugin-added fields.
    pub ext: HashMap<String, serde_json::Value>,
}

// ── Lightweight reference types used inside PortfolioRow ─────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneRef { pub id: MilestoneId, pub name: String, pub due: NaiveDate }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFlagRef { pub severity: String, pub description: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiRef  { pub name: String, pub target: f64, pub current: f64, pub unit: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OkrRef  { pub objective: String, pub key_result: String, pub progress: f64 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VestingScheduleRef { pub percent_vested: f64, pub next_vest_date: NaiveDate }

impl PortfolioRow {
    /// Construct a minimal valid row from core identity fields.
    pub fn new(
        component_id: ComponentId,
        component_type: impl Into<String>,
        name: impl Into<String>,
        created_by: EntityId,
    ) -> Self {
        let now = Utc::now();
        PortfolioRow {
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
            lifecycle_stage: None,
            created_at: now,
            updated_at: now,
            archived_at: None,
            deleted_at: None,
            owners: vec![created_by],
            editors: vec![],
            viewers: vec![],
            contributors: vec![],
            watchers: vec![],
            manager_id: None,
            created_by,
            updated_by: None,
            tags: vec![],
            search_keywords: vec![],
            domain: None,
            start_date: None,
            end_date: None,
            due_date: None,
            estimated_duration_days: None,
            progress_pct: 0.0,
            milestones: vec![],
            sprint_refs: vec![],
            quarter: None,
            fiscal_year: None,
            budget_allocated: None,
            budget_spent: Decimal::ZERO,
            currency: None,
            rate: None,
            rate_type: None,
            revenue: Decimal::ZERO,
            expenses: Decimal::ZERO,
            tax_category: None,
            payment_terms: None,
            invoice_refs: vec![],
            bank_account_ref: None,
            resource_units_total: None,
            resource_units_allocated: 0.0,
            resource_type: None,
            allocation_pct: 0.0,
            policy_ids: vec![],
            governance_model: None,
            approval_status: None,
            compliance_flags: vec![],
            risk_flags: vec![],
            last_reviewed_at: None,
            charter_ref: None,
            regulatory_tags: vec![],
            views: 0,
            clicks: 0,
            shares: 0,
            followers: 0,
            kpi_refs: vec![],
            okr_refs: vec![],
            toolbox_ids: vec![],
            benefit_type: None,
            benefit_provider: None,
            benefit_balance: None,
            benefit_ytd_contributions: None,
            vesting_schedule: None,
            tax_treatment: None,
            claim_workflow_id: None,
            pto_accrued_days: None,
            pto_used_ytd_days: None,
            engagement_type: None,
            platform: None,
            client_ref: None,
            earnings_gross: None,
            tips: None,
            expenses_gig: None,
            portable_contribution: None,
            portable_contribution_pct: None,
            mileage: None,
            payment_status: None,
            hours_worked: None,
            contract_status: None,
            campaign_type: None,
            funding_type: None,
            target_amount: None,
            raised_amount: None,
            awarded_amount: None,
            disbursed_amount: None,
            investor_count: None,
            equity_offered_pct: None,
            close_date: None,
            linked_bank_account: None,
            parent_id: None,
            child_ids: vec![],
            dependency_ids: vec![],
            dependent_ids: vec![],
            linked_ids: vec![],
            owner_count: None,
            contributor_count: None,
            contribution_types_open: vec![],
            pending_review_count: None,
            crowdresourcing_campaign_ref: None,
            review_policy: None,
            attribution_weights: HashMap::new(),
            last_contribution_date: None,
            merge_conflicts: 0,
            ext: HashMap::new(),
        }
    }

    /// Compute budget_remaining synchronously.
    pub fn budget_remaining(&self) -> Option<Decimal> {
        self.budget_allocated.map(|a| a - self.budget_spent)
    }

    /// Compute budget_utilization_pct synchronously.
    pub fn budget_utilization_pct(&self) -> Option<f64> {
        self.budget_allocated.and_then(|a| {
            if a.is_zero() { return None; }
            let spent: f64 = self.budget_spent.to_string().parse().unwrap_or(0.0);
            let alloc: f64 = a.to_string().parse().unwrap_or(1.0);
            Some(spent / alloc * 100.0)
        })
    }

    /// Compute profit (revenue - expenses) synchronously.
    pub fn profit(&self) -> Decimal { self.revenue - self.expenses }

    /// True if this row is soft-deleted.
    pub fn is_deleted(&self) -> bool { self.deleted_at.is_some() }

    /// True if this row is archived.
    pub fn is_archived(&self) -> bool { self.archived_at.is_some() }

    /// Actual duration in days from start_date to end_date (or today).
    pub fn actual_duration_days(&self) -> Option<i64> {
        let start = self.start_date?;
        let end = self.end_date.unwrap_or_else(|| Utc::now().date_naive());
        Some((end - start).num_days())
    }

    /// Schedule variance = actual_duration_days - estimated_duration_days.
    /// Negative = ahead of schedule.
    pub fn schedule_variance_days(&self) -> Option<i64> {
        let actual = self.actual_duration_days()?;
        let est = self.estimated_duration_days? as i64;
        Some(actual - est)
    }

    /// PTO remaining = accrued - used.
    pub fn pto_remaining_days(&self) -> Option<Decimal> {
        Some(self.pto_accrued_days? - self.pto_used_ytd_days.unwrap_or(Decimal::ZERO))
    }

    /// Earnings net = gross - expenses_gig.
    pub fn earnings_net(&self) -> Option<Decimal> {
        Some(self.earnings_gross? - self.expenses_gig.unwrap_or(Decimal::ZERO))
    }

    /// Campaign funding percentage.
    pub fn campaign_funding_pct(&self) -> Option<f64> {
        let target: f64 = self.target_amount?.to_string().parse().ok()?;
        let raised: f64 = self.raised_amount?.to_string().parse().ok()?;
        if target == 0.0 { return None; }
        Some(raised / target * 100.0)
    }
}

// =============================================================================
// §6 — CELL STORE
// =============================================================================

/// A key into the CellStore: (row_id, column_id).
pub type CellKey = (ComponentId, ColumnId);

/// Metadata stored alongside each cached cell value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellEntry {
    pub value:       CellValue,
    /// Epoch seconds when this entry was written.
    pub written_at:  u64,
    /// Epoch seconds after which the value is considered stale (None = never).
    pub ttl_secs:    Option<u64>,
    /// Source: "user" | "computed" | "engine" | "formula"
    pub source:      String,
    /// Actor who wrote or triggered this value (for audit).
    pub actor_id:    Option<EntityId>,
}

impl CellEntry {
    pub fn is_stale(&self, now_epoch: u64) -> bool {
        self.ttl_secs.map_or(false, |ttl| now_epoch > self.written_at + ttl)
    }
}

/// In-memory indexed cell store: (component_id, column_id) → CellEntry.
///
/// Stores computed, AI-signal, formula, and sheet-local custom-column values
/// that augment (but do not replace) the canonical PortfolioRow struct fields.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CellStore {
    cells: HashMap<ComponentId, HashMap<ColumnId, CellEntry>>,
}

impl CellStore {
    pub fn new() -> Self { Self::default() }

    /// Write a cell value (with TTL in seconds).
    pub fn set(
        &mut self,
        row_id: ComponentId,
        col_id: impl Into<ColumnId>,
        value: CellValue,
        source: impl Into<String>,
        actor_id: Option<EntityId>,
        ttl_secs: Option<u64>,
    ) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.cells.entry(row_id).or_default().insert(
            col_id.into(),
            CellEntry { value, written_at: now, ttl_secs, source: source.into(), actor_id },
        );
    }

    /// Read a cell value. Returns None if missing or stale.
    pub fn get(&self, row_id: &ComponentId, col_id: &str) -> Option<&CellValue> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.cells.get(row_id)?.get(col_id).and_then(|e| {
            if e.is_stale(now) { None } else { Some(&e.value) }
        })
    }

    /// Invalidate all cells for a row (called when the row's EventLog is mutated).
    pub fn invalidate_row(&mut self, row_id: &ComponentId) {
        if let Some(cols) = self.cells.get_mut(row_id) {
            cols.clear();
        }
    }

    /// Invalidate a single cell.
    pub fn invalidate(&mut self, row_id: &ComponentId, col_id: &str) {
        if let Some(cols) = self.cells.get_mut(row_id) {
            cols.remove(col_id);
        }
    }

    /// Count of distinct rows with any cached cells.
    pub fn row_count(&self) -> usize { self.cells.len() }

    /// All column values for a single row (including stale, for inspection).
    pub fn row_cells(&self, row_id: &ComponentId) -> Option<&HashMap<ColumnId, CellEntry>> {
        self.cells.get(row_id)
    }
}

// =============================================================================
// §7 — ROW STORE  (the single master store backing all sheets)
// =============================================================================

/// The canonical master store for all PortfolioRows.
/// All sheets are views over this store — rows are never owned by sheets.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RowStore {
    rows: HashMap<ComponentId, PortfolioRow>,
    /// Ordered insertion list for default pagination.
    insertion_order: Vec<ComponentId>,
}

impl RowStore {
    pub fn new() -> Self { Self::default() }

    /// Insert a new row. Idempotent on duplicate component_id (overwrites).
    pub fn insert(&mut self, row: PortfolioRow) {
        let id = row.component_id;
        if !self.rows.contains_key(&id) {
            self.insertion_order.push(id);
        }
        self.rows.insert(id, row);
    }

    pub fn get(&self, id: &ComponentId) -> Option<&PortfolioRow> { self.rows.get(id) }
    pub fn get_mut(&mut self, id: &ComponentId) -> Option<&mut PortfolioRow> { self.rows.get_mut(id) }
    pub fn contains(&self, id: &ComponentId) -> bool { self.rows.contains_key(id) }

    /// Remove a row by ID (hard delete — prefer soft-delete via deleted_at).
    pub fn remove(&mut self, id: &ComponentId) -> Option<PortfolioRow> {
        if self.rows.remove(id).is_some() {
            self.insertion_order.retain(|x| x != id);
            return self.rows.remove(id); // already removed — returns None
        }
        None
    }

    /// Iterate all rows in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = &PortfolioRow> {
        self.insertion_order.iter().filter_map(|id| self.rows.get(id))
    }

    /// Iterate rows that pass the given predicate.
    pub fn iter_filtered<'a, F>(&'a self, predicate: F) -> impl Iterator<Item = &'a PortfolioRow>
    where F: Fn(&PortfolioRow) -> bool + 'a {
        self.iter().filter(move |r| predicate(r))
    }

    pub fn len(&self) -> usize { self.rows.len() }
    pub fn is_empty(&self) -> bool { self.rows.is_empty() }

    /// All component IDs in insertion order.
    pub fn ids(&self) -> &[ComponentId] { &self.insertion_order }
}

// =============================================================================
// §8 — SHEET DEFINITION & SHEET REGISTRY (30 canonical sheets)
// =============================================================================

/// Default sort direction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortDirection { Ascending, Descending }

/// A predefined sort applied to a sheet by default.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultSort { pub column_id: ColumnId, pub direction: SortDirection }

/// A predefined grouping applied to a sheet by default.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultGroup { pub column_id: ColumnId, pub collapse_default: bool }

/// Which item/container types are shown by default on a sheet (None = all).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowTypeFilter { pub item_types: Option<Vec<String>>, pub container_types: Option<Vec<String>> }

/// Static definition for one of the 30 canonical sheets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub primary_row_types: Vec<String>,
    pub default_group_by: Vec<DefaultGroup>,
    pub default_sort: Vec<DefaultSort>,
    pub pinned_column_ids: Vec<&'static str>,
    pub default_visible_column_ids: Vec<&'static str>,
    /// Whether this sheet is a read-only derived/log sheet.
    pub readonly: bool,
}

/// The canonical registry of all 30 portfolio sheets (SHT-001 … SHT-030).
pub struct SheetRegistry;

impl SheetRegistry {
    pub fn all() -> Vec<SheetDefinition> {
        vec![
            SheetDefinition {
                id: "SHT-001", name: "Master Registry",
                description: "Unfiltered full-column view of every PortfolioComponent the authenticated user can access.",
                primary_row_types: vec!["*".into()],
                default_group_by: vec![DefaultGroup { column_id: "component_type".into(), collapse_default: false }],
                default_sort: vec![DefaultSort { column_id: "updated_at".into(), direction: SortDirection::Descending }],
                pinned_column_ids: vec!["component_id","slug","component_type","item_type","name","status","visibility","owners"],
                default_visible_column_ids: vec!["created_at","updated_at","progress_pct","due_date","budget_remaining","tags"],
                readonly: false,
            },
            SheetDefinition {
                id: "SHT-002", name: "Portfolio Hierarchy",
                description: "Structural tree: Portfolio → SubPortfolio → Program → Project → Task.",
                primary_row_types: vec!["Portfolio".into(),"SubPortfolio".into(),"Program".into(),"Project".into(),"Task".into()],
                default_group_by: vec![],
                default_sort: vec![DefaultSort { column_id: "depth_indicator".into(), direction: SortDirection::Ascending }],
                pinned_column_ids: vec!["name","component_type","status"],
                default_visible_column_ids: vec!["child_count","progress_pct","budget_allocated","health_score","risk_score","owners","due_date"],
                readonly: false,
            },
            SheetDefinition {
                id: "SHT-003", name: "Programs",
                description: "Strategic initiatives grouping related projects.",
                primary_row_types: vec!["Program".into()],
                default_group_by: vec![DefaultGroup { column_id: "status".into(), collapse_default: false }],
                default_sort: vec![DefaultSort { column_id: "health_score".into(), direction: SortDirection::Descending }],
                pinned_column_ids: vec!["name","status"],
                default_visible_column_ids: vec!["start_date","end_date","progress_pct","budget_allocated","budget_spent","health_score","risk_score"],
                readonly: false,
            },
            SheetDefinition {
                id: "SHT-004", name: "Projects",
                description: "Bounded, deliverable-producing units of work.",
                primary_row_types: vec!["Project".into()],
                default_group_by: vec![
                    DefaultGroup { column_id: "parent_id".into(), collapse_default: false },
                    DefaultGroup { column_id: "status".into(),    collapse_default: false },
                ],
                default_sort: vec![DefaultSort { column_id: "due_date".into(), direction: SortDirection::Ascending }],
                pinned_column_ids: vec!["name","status","owners"],
                default_visible_column_ids: vec!["start_date","due_date","progress_pct","budget_remaining","health_score","risk_score","velocity"],
                readonly: false,
            },
            SheetDefinition {
                id: "SHT-005", name: "Tasks & Backlog",
                description: "Atomic work items across all projects — Tasks, Stories, Epics, Features.",
                primary_row_types: vec!["Task".into()],
                default_group_by: vec![
                    DefaultGroup { column_id: "parent_id".into(), collapse_default: false },
                    DefaultGroup { column_id: "sprint_refs".into(), collapse_default: false },
                ],
                default_sort: vec![DefaultSort { column_id: "priority".into(), direction: SortDirection::Descending }],
                pinned_column_ids: vec!["name","status","priority"],
                default_visible_column_ids: vec!["parent_id","due_date","editors","story_points","labels"],
                readonly: false,
            },
            SheetDefinition {
                id: "SHT-006", name: "Resources",
                description: "Human, financial, equipment, and service resources.",
                primary_row_types: vec!["Resource".into()],
                default_group_by: vec![DefaultGroup { column_id: "resource_type".into(), collapse_default: false }],
                default_sort: vec![DefaultSort { column_id: "name".into(), direction: SortDirection::Ascending }],
                pinned_column_ids: vec!["name","resource_type","status"],
                default_visible_column_ids: vec!["allocation_pct","resource_units_total","resource_units_allocated","tags"],
                readonly: false,
            },
            SheetDefinition {
                id: "SHT-007", name: "Assets",
                description: "Capital and intellectual assets — digital, physical, financial, IP.",
                primary_row_types: vec!["Asset".into()],
                default_group_by: vec![DefaultGroup { column_id: "item_type".into(), collapse_default: false }],
                default_sort: vec![DefaultSort { column_id: "updated_at".into(), direction: SortDirection::Descending }],
                pinned_column_ids: vec!["name","status"],
                default_visible_column_ids: vec!["budget_allocated","revenue","tax_category","tags"],
                readonly: false,
            },
            SheetDefinition {
                id: "SHT-008", name: "Artifacts",
                description: "Outputs: documents, files, designs, code, generated assets.",
                primary_row_types: vec!["Artifact".into()],
                default_group_by: vec![
                    DefaultGroup { column_id: "parent_id".into(), collapse_default: false },
                    DefaultGroup { column_id: "item_type".into(), collapse_default: false },
                ],
                default_sort: vec![DefaultSort { column_id: "updated_at".into(), direction: SortDirection::Descending }],
                pinned_column_ids: vec!["name","status"],
                default_visible_column_ids: vec!["domain","tags","created_by","updated_at"],
                readonly: false,
            },
            SheetDefinition {
                id: "SHT-009", name: "Finances",
                description: "Unified financial ledger view across all components — master P&L.",
                primary_row_types: vec!["*".into()],
                default_group_by: vec![
                    DefaultGroup { column_id: "quarter".into(),     collapse_default: false },
                    DefaultGroup { column_id: "tax_category".into(), collapse_default: false },
                ],
                default_sort: vec![DefaultSort { column_id: "revenue".into(), direction: SortDirection::Descending }],
                pinned_column_ids: vec!["name","component_type"],
                default_visible_column_ids: vec!["quarter","fiscal_year","revenue","expenses","budget_allocated","budget_spent","profit","tax_category","payment_status"],
                readonly: false,
            },
            SheetDefinition {
                id: "SHT-010", name: "Budget Tracker",
                description: "Budget vs actual spend across programs, projects, gigs, contracts.",
                primary_row_types: vec!["Program".into(),"Project".into(),"Gig".into(),"Contract".into()],
                default_group_by: vec![
                    DefaultGroup { column_id: "parent_id".into(), collapse_default: false },
                    DefaultGroup { column_id: "status".into(),    collapse_default: false },
                ],
                default_sort: vec![DefaultSort { column_id: "budget_utilization_pct".into(), direction: SortDirection::Descending }],
                pinned_column_ids: vec!["name","status"],
                default_visible_column_ids: vec!["budget_allocated","budget_spent","budget_remaining","budget_utilization_pct","currency"],
                readonly: false,
            },
            SheetDefinition {
                id: "SHT-011", name: "Work & Gigs",
                description: "All income-generating engagements: gigs, contracts, jobs, bookings.",
                primary_row_types: vec!["Gig".into(),"Contract".into(),"Job".into(),"Task".into()],
                default_group_by: vec![
                    DefaultGroup { column_id: "platform".into(),         collapse_default: false },
                    DefaultGroup { column_id: "payment_status".into(),   collapse_default: false },
                ],
                default_sort: vec![DefaultSort { column_id: "start_date".into(), direction: SortDirection::Descending }],
                pinned_column_ids: vec!["name","engagement_type","status"],
                default_visible_column_ids: vec!["platform","client_ref","rate","hours_worked","earnings_gross","tips","earnings_net","payment_status","portable_contribution"],
                readonly: false,
            },
            SheetDefinition {
                id: "SHT-012", name: "Kanban Board",
                description: "Kanban view: columns = status values; cards show name, owner, due_date, health_score.",
                primary_row_types: vec!["Project".into(),"Task".into()],
                default_group_by: vec![DefaultGroup { column_id: "status".into(), collapse_default: false }],
                default_sort: vec![DefaultSort { column_id: "priority".into(), direction: SortDirection::Descending }],
                pinned_column_ids: vec!["name"],
                default_visible_column_ids: vec!["owners","due_date","health_score","tags"],
                readonly: false,
            },
            SheetDefinition {
                id: "SHT-013", name: "Timeline",
                description: "All dated rows on a timeline; organized by quarter and program.",
                primary_row_types: vec!["*".into()],
                default_group_by: vec![
                    DefaultGroup { column_id: "quarter".into(), collapse_default: false },
                    DefaultGroup { column_id: "parent_id".into(), collapse_default: false },
                ],
                default_sort: vec![DefaultSort { column_id: "start_date".into(), direction: SortDirection::Ascending }],
                pinned_column_ids: vec!["name","component_type"],
                default_visible_column_ids: vec!["start_date","end_date","due_date","progress_pct","status"],
                readonly: false,
            },
            SheetDefinition {
                id: "SHT-014", name: "Roadmap",
                description: "Programs, projects, and milestones on a quarter-based roadmap.",
                primary_row_types: vec!["Program".into(),"Project".into()],
                default_group_by: vec![
                    DefaultGroup { column_id: "parent_id".into(), collapse_default: false },
                    DefaultGroup { column_id: "quarter".into(),   collapse_default: false },
                ],
                default_sort: vec![DefaultSort { column_id: "start_date".into(), direction: SortDirection::Ascending }],
                pinned_column_ids: vec!["name"],
                default_visible_column_ids: vec!["quarter","fiscal_year","status","milestone_refs","due_date","health_score"],
                readonly: false,
            },
            SheetDefinition {
                id: "SHT-015", name: "Portable Benefits",
                description: "Worker's complete real-time benefits dashboard — balances, coverage, tax signals.",
                primary_row_types: vec!["BenefitAccount".into()],
                default_group_by: vec![DefaultGroup { column_id: "benefit_type".into(), collapse_default: false }],
                default_sort: vec![DefaultSort { column_id: "benefit_balance".into(), direction: SortDirection::Descending }],
                pinned_column_ids: vec!["name","benefit_type","status"],
                default_visible_column_ids: vec!["benefit_provider","benefit_balance","benefit_ytd_contributions","vesting_pct","tax_treatment","pto_remaining_days","coverage_gap_flag"],
                readonly: false,
            },
            SheetDefinition {
                id: "SHT-016", name: "Grants & Microfinancing",
                description: "Grant applications, microloans, community lending — full funding pipeline.",
                primary_row_types: vec!["Grant".into(),"Campaign".into()],
                default_group_by: vec![DefaultGroup { column_id: "status".into(), collapse_default: false }],
                default_sort: vec![DefaultSort { column_id: "due_date".into(), direction: SortDirection::Ascending }],
                pinned_column_ids: vec!["name","funding_type","status"],
                default_visible_column_ids: vec!["target_amount","awarded_amount","disbursed_amount","due_date","linked_bank_account","portfolio_match_score"],
                readonly: false,
            },
            SheetDefinition {
                id: "SHT-017", name: "Equity & Crowdfunding",
                description: "Equity crowdfunding, revenue-share, SAFE, and investment rows.",
                primary_row_types: vec!["Campaign".into(),"Investment".into()],
                default_group_by: vec![DefaultGroup { column_id: "campaign_type".into(), collapse_default: false }],
                default_sort: vec![DefaultSort { column_id: "raised_amount".into(), direction: SortDirection::Descending }],
                pinned_column_ids: vec!["name","campaign_type","status"],
                default_visible_column_ids: vec!["target_amount","raised_amount","funding_pct","investor_count","close_date","equity_offered_pct","crowdfunding_engine_score"],
                readonly: false,
            },
            SheetDefinition {
                id: "SHT-018", name: "Group Economics",
                description: "Shared portfolios, revenue pools, cooperative economics.",
                primary_row_types: vec!["Portfolio".into(),"Campaign".into()],
                default_group_by: vec![DefaultGroup { column_id: "parent_id".into(), collapse_default: false }],
                default_sort: vec![DefaultSort { column_id: "revenue".into(), direction: SortDirection::Descending }],
                pinned_column_ids: vec!["name","status"],
                default_visible_column_ids: vec!["owner_count","contributor_count","revenue","expenses","attribution_weights","distribution_pending"],
                readonly: false,
            },
            SheetDefinition {
                id: "SHT-019", name: "Organizations",
                description: "Org and collective profiles with governance and member management.",
                primary_row_types: vec!["Profile".into()],
                default_group_by: vec![DefaultGroup { column_id: "item_type".into(), collapse_default: false }],
                default_sort: vec![DefaultSort { column_id: "name".into(), direction: SortDirection::Ascending }],
                pinned_column_ids: vec!["name","status"],
                default_visible_column_ids: vec!["governance_model","owners","contributor_count","tags"],
                readonly: false,
            },
            SheetDefinition {
                id: "SHT-020", name: "Shared Portfolios",
                description: "Co-owned portfolios: team, org, collective, cooperative, federation, crowdresourced.",
                primary_row_types: vec!["Portfolio".into()],
                default_group_by: vec![DefaultGroup { column_id: "portfolio_type".into(), collapse_default: false }],
                default_sort: vec![DefaultSort { column_id: "health_score".into(), direction: SortDirection::Descending }],
                pinned_column_ids: vec!["name","status"],
                default_visible_column_ids: vec!["owner_count","contributor_count","review_policy","collaboration_score","last_contribution_date","merge_conflicts"],
                readonly: false,
            },
            SheetDefinition {
                id: "SHT-021", name: "Collaboration",
                description: "Contribution-centric view: all shared and contributed components.",
                primary_row_types: vec!["*".into()],
                default_group_by: vec![DefaultGroup { column_id: "contributors".into(), collapse_default: false }],
                default_sort: vec![DefaultSort { column_id: "last_contribution_date".into(), direction: SortDirection::Descending }],
                pinned_column_ids: vec!["name","component_type"],
                default_visible_column_ids: vec!["owner_count","pending_review_count","contribution_types_open","attribution_weights","collaboration_score"],
                readonly: false,
            },
            SheetDefinition {
                id: "SHT-022", name: "Risk Register",
                description: "All rows with risk_flags, grouped by severity.",
                primary_row_types: vec!["*".into()],
                default_group_by: vec![DefaultGroup { column_id: "risk_flags".into(), collapse_default: false }],
                default_sort: vec![DefaultSort { column_id: "risk_score".into(), direction: SortDirection::Descending }],
                pinned_column_ids: vec!["name","component_type","status"],
                default_visible_column_ids: vec!["risk_flags","compliance_flags","approval_status","last_reviewed_at","risk_score"],
                readonly: false,
            },
            SheetDefinition {
                id: "SHT-023", name: "Analytics Dashboard",
                description: "All rows sorted by health score; engine signals and KPIs.",
                primary_row_types: vec!["*".into()],
                default_group_by: vec![
                    DefaultGroup { column_id: "component_type".into(), collapse_default: false },
                    DefaultGroup { column_id: "health_score".into(),   collapse_default: false },
                ],
                default_sort: vec![DefaultSort { column_id: "health_score".into(), direction: SortDirection::Descending }],
                pinned_column_ids: vec!["name","component_type"],
                default_visible_column_ids: vec!["health_score","risk_score","views","clicks","followers","kpi_refs","anomaly_flags"],
                readonly: false,
            },
            SheetDefinition {
                id: "SHT-024", name: "Feed & Community",
                description: "Published rows: profiles, posts, public artifacts — community feed.",
                primary_row_types: vec!["Profile".into(),"Artifact".into()],
                default_group_by: vec![DefaultGroup { column_id: "domain".into(), collapse_default: false }],
                default_sort: vec![DefaultSort { column_id: "updated_at".into(), direction: SortDirection::Descending }],
                pinned_column_ids: vec!["name","component_type"],
                default_visible_column_ids: vec!["visibility","views","shares","followers","engagement_rate"],
                readonly: false,
            },
            SheetDefinition {
                id: "SHT-025", name: "Contacts & Network",
                description: "Contact profiles, relationship types, CRM linkage.",
                primary_row_types: vec!["Profile".into()],
                default_group_by: vec![DefaultGroup { column_id: "tags".into(), collapse_default: false }],
                default_sort: vec![DefaultSort { column_id: "name".into(), direction: SortDirection::Ascending }],
                pinned_column_ids: vec!["name","status"],
                default_visible_column_ids: vec!["client_ref","domain","tags","linked_ids","created_at"],
                readonly: false,
            },
            SheetDefinition {
                id: "SHT-026", name: "Marketplace Listings",
                description: "Public assets, resources, and gigs available on the marketplace.",
                primary_row_types: vec!["Asset".into(),"Resource".into(),"Gig".into()],
                default_group_by: vec![DefaultGroup { column_id: "domain".into(), collapse_default: false }],
                default_sort: vec![DefaultSort { column_id: "updated_at".into(), direction: SortDirection::Descending }],
                pinned_column_ids: vec!["name","status"],
                default_visible_column_ids: vec!["visibility","rate","rate_type","tags","views","followers"],
                readonly: false,
            },
            SheetDefinition {
                id: "SHT-027", name: "Exchange",
                description: "Investment, deal, campaign, and asset exchange transactions.",
                primary_row_types: vec!["Investment".into(),"Campaign".into(),"Asset".into()],
                default_group_by: vec![DefaultGroup { column_id: "campaign_type".into(), collapse_default: false }],
                default_sort: vec![DefaultSort { column_id: "updated_at".into(), direction: SortDirection::Descending }],
                pinned_column_ids: vec!["name","status"],
                default_visible_column_ids: vec!["raised_amount","target_amount","investor_count","close_date","linked_bank_account"],
                readonly: false,
            },
            SheetDefinition {
                id: "SHT-028", name: "Archive",
                description: "Deep-storage container for completed, retired, and historical items.",
                primary_row_types: vec!["*".into()],
                default_group_by: vec![DefaultGroup { column_id: "component_type".into(), collapse_default: false }],
                default_sort: vec![DefaultSort { column_id: "archived_at".into(), direction: SortDirection::Descending }],
                pinned_column_ids: vec!["name","component_type"],
                default_visible_column_ids: vec!["archived_at","status","owners","tags"],
                readonly: true,
            },
            SheetDefinition {
                id: "SHT-029", name: "Event Log",
                description: "Read-only view of all EventLog entries, sorted by date.",
                primary_row_types: vec![],
                default_group_by: vec![
                    DefaultGroup { column_id: "created_at".into(), collapse_default: false },
                    DefaultGroup { column_id: "created_by".into(), collapse_default: false },
                ],
                default_sort: vec![DefaultSort { column_id: "created_at".into(), direction: SortDirection::Descending }],
                pinned_column_ids: vec!["name","component_type"],
                default_visible_column_ids: vec!["created_at","created_by","status"],
                readonly: true,
            },
            SheetDefinition {
                id: "SHT-030", name: "Custom Sheet",
                description: "User-defined sheet with custom row filter, column set, and grouping.",
                primary_row_types: vec![],
                default_group_by: vec![],
                default_sort: vec![],
                pinned_column_ids: vec!["name"],
                default_visible_column_ids: vec!["status","updated_at"],
                readonly: false,
            },
        ]
    }

    pub fn get(id: &str) -> Option<SheetDefinition> {
        Self::all().into_iter().find(|s| s.id == id)
    }
}

// =============================================================================
// §9 — PORTFOLIO WORKBOOK  (the root "workbook" container)
// =============================================================================

/// The root workbook — one per worker or organization.
/// Equivalent to an Excel workbook: owns the row store, cell store, and all sheets.
#[derive(Debug, Serialize, Deserialize)]
pub struct PortfolioWorkbook {
    pub id:          Uuid,
    pub owner_id:    EntityId,
    pub name:        String,
    pub description: Option<String>,
    pub created_at:  DateTime<Utc>,
    pub updated_at:  DateTime<Utc>,
    /// The single canonical row store shared by all sheets.
    pub row_store:   RowStore,
    /// Computed + AI + custom column value cache.
    pub cell_store:  CellStore,
    /// Ordered list of sheet IDs this workbook exposes (default: all 30).
    pub sheet_order: Vec<String>,
    /// Per-sheet saved custom view definitions (sheet_id → views).
    pub saved_views: HashMap<String, Vec<SavedView>>,
}

impl PortfolioWorkbook {
    pub fn new(owner_id: EntityId, name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            owner_id,
            name: name.into(),
            description: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            row_store: RowStore::new(),
            cell_store: CellStore::new(),
            sheet_order: SheetRegistry::all().iter().map(|s| s.id.to_string()).collect(),
            saved_views: HashMap::new(),
        }
    }

    /// Insert or replace a row in the master row store.
    pub fn upsert_row(&mut self, row: PortfolioRow) {
        self.cell_store.invalidate_row(&row.component_id);
        self.row_store.insert(row);
        self.updated_at = Utc::now();
    }

    /// Get a row by component ID.
    pub fn get_row(&self, id: &ComponentId) -> Option<&PortfolioRow> {
        self.row_store.get(id)
    }

    /// Get rows that match a given item_type string.
    pub fn rows_by_type<'a>(&'a self, item_type: &'a str) -> impl Iterator<Item = &'a PortfolioRow> {
        self.row_store.iter_filtered(move |r| {
            r.item_type.as_deref() == Some(item_type)
        })
    }

    /// Get rows owned by a specific entity.
    pub fn rows_by_owner<'a>(&'a self, owner: &'a EntityId) -> impl Iterator<Item = &'a PortfolioRow> {
        self.row_store.iter_filtered(move |r| r.owners.contains(owner))
    }

    /// All rows visible on a specific sheet (applies the sheet's base filter).
    pub fn rows_for_sheet<'a>(&'a self, sheet_id: &'a str) -> impl Iterator<Item = &'a PortfolioRow> {
        let sheet = SheetRegistry::get(sheet_id);
        self.row_store.iter_filtered(move |r| {
            // Special case: archived sheet shows only archived rows.
            if sheet_id == "SHT-028" { return r.is_archived(); }
            // Event log sheet: always empty (EventLog rows are synthetic).
            if sheet_id == "SHT-029" { return false; }
            // Master registry: show all non-deleted rows.
            if sheet_id == "SHT-001" { return !r.is_deleted(); }
            // Default: filter by primary_row_types if specified.
            match &sheet {
                Some(def) if !def.primary_row_types.is_empty() => {
                    def.primary_row_types.iter().any(|t| {
                        t == "*"
                            || r.item_type.as_deref() == Some(t.as_str())
                            || r.container_type.as_deref() == Some(t.as_str())
                    }) && !r.is_deleted()
                }
                _ => !r.is_deleted(),
            }
        })
    }

    /// Set a computed or AI cell value in the CellStore.
    pub fn set_cell(
        &mut self,
        row_id: ComponentId,
        col_id: impl Into<ColumnId>,
        value: CellValue,
        source: impl Into<String>,
        ttl_secs: Option<u64>,
    ) {
        self.cell_store.set(row_id, col_id, value, source, None, ttl_secs);
    }

    /// Retrieve a computed cell value from the CellStore.
    pub fn get_cell(&self, row_id: &ComponentId, col_id: &str) -> Option<&CellValue> {
        self.cell_store.get(row_id, col_id)
    }

    pub fn row_count(&self) -> usize { self.row_store.len() }
}

/// A named, persisted view configuration saved by a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedView {
    pub id:          Uuid,
    pub name:        String,
    pub sheet_id:    String,
    pub created_by:  EntityId,
    pub created_at:  DateTime<Utc>,
    pub updated_at:  DateTime<Utc>,
    /// Serialized ViewDefinition (defined in portfolio_view_engine.rs).
    pub definition:  serde_json::Value,
}

// =============================================================================
// §10 — CANONICAL COLUMN SCHEMA BUILDERS (for each major sheet)
// =============================================================================

/// Build the canonical ColumnSchema for the Master Registry sheet (SHT-001).
pub fn master_registry_schema() -> ColumnSchema {
    use ColumnGroup::*;
    use ColumnType::*;
    ColumnSchema::new(vec![
        ColumnDefinition::new("component_id",   "ID",             TextField,   Identity).pinned().readonly(),
        ColumnDefinition::new("slug",            "Slug",           TextField,   Identity).pinned().readonly(),
        ColumnDefinition::new("component_type",  "Type",           EnumField { variants: vec!["item".into(),"container".into()] }, Taxonomy).pinned().groupable(),
        ColumnDefinition::new("item_type",       "Item Type",      EnumField { variants: item_type_variants() }, Taxonomy).pinned().groupable().filterable(),
        ColumnDefinition::new("name",            "Name",           TextField,   Identity).pinned().width(240),
        ColumnDefinition::new("status",          "Status",         EnumField { variants: status_variants() }, Lifecycle).pinned().groupable(),
        ColumnDefinition::new("visibility",      "Visibility",     EnumField { variants: vec!["Private".into(),"Protected".into(),"Public".into(),"Internal".into()] }, VisibilityAndAccess).pinned(),
        ColumnDefinition::new("owners",          "Owners",         MultiUserField, OwnershipAndUsers).pinned(),
        ColumnDefinition::new("created_at",      "Created",        DateTimeField, Lifecycle).readonly(),
        ColumnDefinition::new("updated_at",      "Updated",        DateTimeField, Lifecycle).readonly(),
        ColumnDefinition::new("due_date",        "Due Date",       DateField,   TimelineAndSchedule),
        ColumnDefinition::new("progress_pct",    "Progress",       PercentField, WorkAndDelivery).aggregatable(),
        ColumnDefinition::new("health_score",    "Health",         ComputedColumn { formula: None }, AiAndEngineSignals).readonly().aggregatable(),
        ColumnDefinition::new("risk_score",      "Risk",           ComputedColumn { formula: None }, AiAndEngineSignals).readonly().aggregatable(),
        ColumnDefinition::new("budget_remaining","Budget Rem.",    CurrencyField, FinancialAndBudget).aggregatable(),
        ColumnDefinition::new("tags",            "Tags",           TagField,    TagsAndDiscovery).filterable(),
    ])
}

/// Build the ColumnSchema for the Finances sheet (SHT-009).
pub fn finances_schema() -> ColumnSchema {
    use ColumnGroup::*;
    use ColumnType::*;
    ColumnSchema::new(vec![
        ColumnDefinition::new("name",             "Name",           TextField,   Identity).pinned(),
        ColumnDefinition::new("component_type",   "Type",           EnumField { variants: item_type_variants() }, Taxonomy).pinned().groupable(),
        ColumnDefinition::new("quarter",          "Quarter",        TextField,   TimelineAndSchedule).groupable(),
        ColumnDefinition::new("fiscal_year",      "FY",             NumberField, TimelineAndSchedule).groupable(),
        ColumnDefinition::new("revenue",          "Revenue",        CurrencyField, FinancialAndBudget).aggregatable(),
        ColumnDefinition::new("expenses",         "Expenses",       CurrencyField, FinancialAndBudget).aggregatable(),
        ColumnDefinition::new("budget_allocated", "Budget",         CurrencyField, FinancialAndBudget).aggregatable(),
        ColumnDefinition::new("budget_spent",     "Spent",          CurrencyField, FinancialAndBudget).aggregatable(),
        ColumnDefinition::new("budget_remaining", "Remaining",      ComputedColumn { formula: Some("budget_allocated - budget_spent".into()) }, FinancialAndBudget).readonly().aggregatable(),
        ColumnDefinition::new("profit",           "Profit",         ComputedColumn { formula: Some("revenue - expenses".into()) }, FinancialAndBudget).readonly().aggregatable(),
        ColumnDefinition::new("tax_category",     "Tax Category",   EnumField { variants: tax_category_variants() }, FinancialAndBudget).groupable(),
        ColumnDefinition::new("payment_status",   "Payment",        EnumField { variants: payment_status_variants() }, GigAndContract),
        ColumnDefinition::new("bank_account_ref", "Bank Account",   RelationField, FinancialAndBudget),
    ])
}

/// Build the ColumnSchema for the Portable Benefits sheet (SHT-015).
pub fn benefits_schema() -> ColumnSchema {
    use ColumnGroup::*;
    use ColumnType::*;
    ColumnSchema::new(vec![
        ColumnDefinition::new("name",                  "Account Name",   TextField,   Identity).pinned(),
        ColumnDefinition::new("benefit_type",          "Benefit Type",   EnumField { variants: benefit_type_variants() }, BenefitAccount).pinned().groupable(),
        ColumnDefinition::new("benefit_provider",      "Provider",       TextField,   BenefitAccount),
        ColumnDefinition::new("status",                "Status",         EnumField { variants: status_variants() }, Lifecycle),
        ColumnDefinition::new("benefit_balance",       "Balance",        CurrencyField, BenefitAccount).aggregatable(),
        ColumnDefinition::new("benefit_ytd_contributions", "YTD Contributions", CurrencyField, BenefitAccount).aggregatable(),
        ColumnDefinition::new("vesting_pct",           "Vesting %",      ComputedColumn { formula: None }, BenefitAccount).readonly(),
        ColumnDefinition::new("projected_balance_1yr", "Proj. Balance (1yr)", ComputedColumn { formula: None }, AiAndEngineSignals).readonly(),
        ColumnDefinition::new("coverage_gap_flag",     "Coverage Gap",   AiColumn { engine: "BenefitsEngine".into() }, AiAndEngineSignals).readonly(),
        ColumnDefinition::new("pto_accrued_days",      "PTO Accrued",    NumberField, BenefitAccount).aggregatable(),
        ColumnDefinition::new("pto_used_ytd_days",     "PTO Used YTD",   NumberField, BenefitAccount).aggregatable(),
        ColumnDefinition::new("pto_remaining_days",    "PTO Remaining",  ComputedColumn { formula: Some("pto_accrued_days - pto_used_ytd_days".into()) }, BenefitAccount).readonly(),
        ColumnDefinition::new("tax_treatment",         "Tax Treatment",  EnumField { variants: vec!["PreTax".into(),"PostTax".into(),"TaxFree".into()] }, BenefitAccount),
        ColumnDefinition::new("tax_savings_ytd",       "Tax Savings YTD", ComputedColumn { formula: None }, AiAndEngineSignals).readonly(),
        ColumnDefinition::new("linked_bank_account",   "Bank Account",   RelationField, FinancialAndBudget),
    ])
}

// ── Enum variant helpers ──────────────────────────────────────────────────────

fn item_type_variants() -> Vec<String> {
    ["Portfolio","SubPortfolio","Program","Project","Resource","Artifact","Asset",
     "BenefitAccount","Gig","Contract","Job","Task","Campaign","Grant","Investment","Profile"]
        .iter().map(|s| s.to_string()).collect()
}

fn status_variants() -> Vec<String> {
    ["Draft","Active","Paused","Completed","Archived","Cancelled","Suspended","UnderReview","Rejected"]
        .iter().map(|s| s.to_string()).collect()
}

fn tax_category_variants() -> Vec<String> {
    ["Business","SelfEmployment","PassThrough","Exempt","Investment","Benefit"]
        .iter().map(|s| s.to_string()).collect()
}

fn payment_status_variants() -> Vec<String> {
    ["Unpaid","Partial","Paid","Overdue","Disputed"]
        .iter().map(|s| s.to_string()).collect()
}

fn benefit_type_variants() -> Vec<String> {
    ["Health","Dental","Vision","HSA","Retirement","PTO","Disability",
     "ProfessionalDev","EmergencySavings","PortableSavings"]
        .iter().map(|s| s.to_string()).collect()
}

// =============================================================================
// §11 — TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn owner() -> EntityId { Uuid::new_v4() }

    #[test]
    fn test_workbook_upsert_and_retrieve() {
        let owner = owner();
        let mut wb = PortfolioWorkbook::new(owner, "Test Workbook");
        let id = Uuid::new_v4();
        let mut row = PortfolioRow::new(id, "item", "My Project", owner);
        row.item_type = Some("Project".into());
        row.budget_allocated = Some(dec!(10_000));
        row.budget_spent = dec!(3_500);

        wb.upsert_row(row);
        assert_eq!(wb.row_count(), 1);

        let r = wb.get_row(&id).unwrap();
        assert_eq!(r.name, "My Project");
        assert_eq!(r.budget_remaining(), Some(dec!(6_500)));
        assert!((r.budget_utilization_pct().unwrap() - 35.0).abs() < 0.01);
    }

    #[test]
    fn test_sheet_filter() {
        let owner = owner();
        let mut wb = PortfolioWorkbook::new(owner, "Filter Test");

        let add = |wb: &mut PortfolioWorkbook, t: &str| {
            let id = Uuid::new_v4();
            let mut r = PortfolioRow::new(id, "item", format!("{t} item"), owner);
            r.item_type = Some(t.into());
            wb.upsert_row(r);
        };
        add(&mut wb, "Project");
        add(&mut wb, "Project");
        add(&mut wb, "Grant");

        let projects: Vec<_> = wb.rows_for_sheet("SHT-004").collect();
        assert_eq!(projects.len(), 2);

        let grants: Vec<_> = wb.rows_for_sheet("SHT-016").collect();
        assert_eq!(grants.len(), 1);
    }

    #[test]
    fn test_cell_store_set_and_get() {
        let owner = owner();
        let mut wb = PortfolioWorkbook::new(owner, "CellStore Test");
        let id = Uuid::new_v4();
        wb.upsert_row(PortfolioRow::new(id, "item", "Row A", owner));

        wb.set_cell(id, "health_score", CellValue::Float(82.5), "engine", Some(300));
        let val = wb.get_cell(&id, "health_score");
        assert_eq!(val, Some(&CellValue::Float(82.5)));
    }

    #[test]
    fn test_cell_store_invalidation() {
        let owner = owner();
        let mut wb = PortfolioWorkbook::new(owner, "Invalidation Test");
        let id = Uuid::new_v4();
        wb.upsert_row(PortfolioRow::new(id, "item", "Row B", owner));
        wb.set_cell(id, "risk_score", CellValue::Float(45.0), "engine", None);
        wb.cell_store.invalidate_row(&id);
        assert!(wb.get_cell(&id, "risk_score").is_none());
    }

    #[test]
    fn test_portfolio_row_computed_fields() {
        let owner = owner();
        let mut r = PortfolioRow::new(Uuid::new_v4(), "item", "Benefit Row", owner);
        r.item_type           = Some("BenefitAccount".into());
        r.pto_accrued_days    = Some(dec!(15));
        r.pto_used_ytd_days   = Some(dec!(6));
        r.earnings_gross      = Some(dec!(1200));
        r.expenses_gig        = Some(dec!(200));
        r.target_amount       = Some(dec!(50_000));
        r.raised_amount       = Some(dec!(20_000));

        assert_eq!(r.pto_remaining_days(), Some(dec!(9)));
        assert_eq!(r.earnings_net(),       Some(dec!(1000)));
        assert!((r.campaign_funding_pct().unwrap() - 40.0).abs() < 0.01);
    }

    #[test]
    fn test_sheet_registry_all_30_sheets() {
        let sheets = SheetRegistry::all();
        assert_eq!(sheets.len(), 30);
        assert!(sheets.iter().any(|s| s.id == "SHT-001"));
        assert!(sheets.iter().any(|s| s.id == "SHT-030"));
    }

    #[test]
    fn test_column_schema_index() {
        let schema = master_registry_schema();
        assert!(schema.get("name").is_some());
        assert!(schema.get("health_score").is_some());
        assert!(schema.get("nonexistent").is_none());
        assert!(schema.pinned_columns().count() >= 4);
    }
}

// =============================================================================
//  portfolio_component_registry.rs — Kogi OS · KPRG v3.0
//  Portfolio Component Registry
//
//  The canonical, system-wide master index for every PortfolioComponent.
//  Every row in the PortfolioWorkbook is registered here; every other system
//  (office, bank, marketplace, community) resolves component references through
//  this registry.
//
//  Registry structure
//  ──────────────────
//    ComponentRegistry  ← the root registry
//      ├── ComponentIndex      primary store: ComponentId → RegistryEntry
//      ├── OwnerIndex          EntityId → Vec<ComponentId>
//      ├── TypeIndex           item_type/container_type → Vec<ComponentId>
//      ├── StatusIndex         status string → Vec<ComponentId>
//      ├── TagIndex            tag string → Vec<ComponentId>
//      ├── DomainIndex         domain string → Vec<ComponentId>
//      ├── FullTextIndex       keyword → Vec<ComponentId>  (tokenised search)
//      └── ParentIndex         parent_id → Vec<ComponentId>  (hierarchy tree)
//
//  @author  Kogi Team
//  @version 3.0.0
//  @license MIT
// =============================================================================

#![allow(dead_code)]

use std::collections::{HashMap, HashSet, BTreeMap};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::portfolio_spreadsheet_substrate::{
    CellValue, ComponentId, EntityId, PortfolioRow,
};

// =============================================================================
// §1 — REGISTRY ENTRY
// =============================================================================

/// Lightweight index record stored for each registered component.
/// Contains only the fields needed for routing, filtering, and display — not
/// the full PortfolioRow.  The full row lives in PortfolioWorkbook::row_store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub component_id:    ComponentId,
    pub slug:            String,
    pub component_type:  String,   // "item" | "container"
    pub item_type:       Option<String>,
    pub container_type:  Option<String>,
    pub name:            String,
    pub display_name:    Option<String>,
    pub status:          String,
    pub visibility:      String,
    pub owners:          Vec<EntityId>,
    pub tags:            Vec<String>,
    pub domain:          Option<String>,
    pub parent_id:       Option<ComponentId>,
    pub created_at:      DateTime<Utc>,
    pub updated_at:      DateTime<Utc>,
    pub is_deleted:      bool,
    pub is_archived:     bool,
    /// Cached health_score from CellStore (None if not yet computed).
    pub health_score:    Option<f64>,
    /// Cached risk_score from CellStore (None if not yet computed).
    pub risk_score:      Option<f64>,
}

impl RegistryEntry {
    /// Build a RegistryEntry from a PortfolioRow.
    pub fn from_row(row: &PortfolioRow) -> Self {
        Self {
            component_id:   row.component_id,
            slug:           row.slug.clone(),
            component_type: row.component_type.clone(),
            item_type:      row.item_type.clone(),
            container_type: row.container_type.clone(),
            name:           row.name.clone(),
            display_name:   row.display_name.clone(),
            status:         row.status.clone(),
            visibility:     row.visibility.clone(),
            owners:         row.owners.clone(),
            tags:           row.tags.clone(),
            domain:         row.domain.clone(),
            parent_id:      row.parent_id,
            created_at:     row.created_at,
            updated_at:     row.updated_at,
            is_deleted:     row.is_deleted(),
            is_archived:    row.is_archived(),
            health_score:   None,
            risk_score:     None,
        }
    }

    /// Tokenise name, slug, tags, and domain into searchable keywords.
    pub fn keywords(&self) -> Vec<String> {
        let mut kw = Vec::new();
        for token in self.name.split_whitespace() {
            kw.push(token.to_lowercase());
        }
        kw.push(self.slug.to_lowercase());
        for tag in &self.tags { kw.push(tag.to_lowercase()); }
        if let Some(d) = &self.domain { kw.push(d.to_lowercase()); }
        kw.dedup();
        kw
    }
}

// =============================================================================
// §2 — SECONDARY INDEXES
// =============================================================================

/// EntityId → set of ComponentIds owned by that entity.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct OwnerIndex(HashMap<EntityId, HashSet<ComponentId>>);

impl OwnerIndex {
    pub fn add(&mut self, owner: EntityId, id: ComponentId) {
        self.0.entry(owner).or_default().insert(id);
    }
    pub fn remove(&mut self, owner: &EntityId, id: &ComponentId) {
        if let Some(set) = self.0.get_mut(owner) { set.remove(id); }
    }
    pub fn get(&self, owner: &EntityId) -> Option<&HashSet<ComponentId>> { self.0.get(owner) }
    pub fn remove_all_for_component(&mut self, id: &ComponentId) {
        for set in self.0.values_mut() { set.remove(id); }
    }
}

/// type_key (item_type or container_type) → set of ComponentIds.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TypeIndex(HashMap<String, HashSet<ComponentId>>);

impl TypeIndex {
    pub fn add(&mut self, type_key: &str, id: ComponentId) {
        self.0.entry(type_key.to_string()).or_default().insert(id);
    }
    pub fn remove(&mut self, type_key: &str, id: &ComponentId) {
        if let Some(set) = self.0.get_mut(type_key) { set.remove(id); }
    }
    pub fn get(&self, type_key: &str) -> Option<&HashSet<ComponentId>> { self.0.get(type_key) }
    pub fn all_types(&self) -> Vec<&String> { self.0.keys().collect() }
}

/// status string → set of ComponentIds in that status.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StatusIndex(HashMap<String, HashSet<ComponentId>>);

impl StatusIndex {
    pub fn add(&mut self, status: &str, id: ComponentId) {
        self.0.entry(status.to_string()).or_default().insert(id);
    }
    pub fn remove(&mut self, status: &str, id: &ComponentId) {
        if let Some(set) = self.0.get_mut(status) { set.remove(id); }
    }
    pub fn get(&self, status: &str) -> Option<&HashSet<ComponentId>> { self.0.get(status) }
}

/// tag string → set of ComponentIds tagged with it.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TagIndex(HashMap<String, HashSet<ComponentId>>);

impl TagIndex {
    pub fn add_tags(&mut self, tags: &[String], id: ComponentId) {
        for tag in tags { self.0.entry(tag.clone()).or_default().insert(id); }
    }
    pub fn remove_tags(&mut self, tags: &[String], id: &ComponentId) {
        for tag in tags {
            if let Some(set) = self.0.get_mut(tag) { set.remove(id); }
        }
    }
    pub fn get(&self, tag: &str) -> Option<&HashSet<ComponentId>> { self.0.get(tag) }
    pub fn all_tags(&self) -> Vec<&String> { self.0.keys().collect() }
}

/// domain string → set of ComponentIds in that domain.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DomainIndex(HashMap<String, HashSet<ComponentId>>);

impl DomainIndex {
    pub fn add(&mut self, domain: &str, id: ComponentId) {
        self.0.entry(domain.to_string()).or_default().insert(id);
    }
    pub fn remove(&mut self, domain: &str, id: &ComponentId) {
        if let Some(set) = self.0.get_mut(domain) { set.remove(id); }
    }
    pub fn get(&self, domain: &str) -> Option<&HashSet<ComponentId>> { self.0.get(domain) }
}

/// parent_id → ordered list of child ComponentIds.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ParentIndex(HashMap<ComponentId, Vec<ComponentId>>);

impl ParentIndex {
    pub fn add_child(&mut self, parent: ComponentId, child: ComponentId) {
        let children = self.0.entry(parent).or_default();
        if !children.contains(&child) { children.push(child); }
    }
    pub fn remove_child(&mut self, parent: &ComponentId, child: &ComponentId) {
        if let Some(vec) = self.0.get_mut(parent) { vec.retain(|x| x != child); }
    }
    pub fn children(&self, parent: &ComponentId) -> &[ComponentId] {
        self.0.get(parent).map(Vec::as_slice).unwrap_or(&[])
    }
    pub fn remove_component(&mut self, id: &ComponentId) {
        self.0.remove(id);
        for vec in self.0.values_mut() { vec.retain(|x| x != id); }
    }
}

/// Full-text index: lowercase keyword token → set of ComponentIds.
/// Supports prefix-match queries by scanning token keys.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FullTextIndex {
    /// BTreeMap for O(log n) prefix scans.
    tokens: BTreeMap<String, HashSet<ComponentId>>,
}

impl FullTextIndex {
    pub fn index_entry(&mut self, entry: &RegistryEntry) {
        for kw in entry.keywords() {
            self.tokens.entry(kw).or_default().insert(entry.component_id);
        }
    }

    pub fn remove_entry(&mut self, entry: &RegistryEntry) {
        for kw in entry.keywords() {
            if let Some(set) = self.tokens.get_mut(&kw) {
                set.remove(&entry.component_id);
            }
        }
    }

    /// Exact-match search.
    pub fn search_exact(&self, keyword: &str) -> HashSet<ComponentId> {
        self.tokens.get(&keyword.to_lowercase())
            .cloned()
            .unwrap_or_default()
    }

    /// Prefix-match search: all components whose tokens start with `prefix`.
    pub fn search_prefix(&self, prefix: &str) -> HashSet<ComponentId> {
        let lower = prefix.to_lowercase();
        let mut results = HashSet::new();
        for (key, ids) in self.tokens.range(lower.clone()..) {
            if !key.starts_with(&lower) { break; }
            results.extend(ids);
        }
        results
    }

    /// Multi-term AND search: all terms must match at least one token.
    pub fn search_all_terms(&self, terms: &[&str]) -> HashSet<ComponentId> {
        let mut iter = terms.iter().map(|t| self.search_prefix(t));
        let first = iter.next().unwrap_or_default();
        iter.fold(first, |acc, next| acc.intersection(&next).cloned().collect())
    }
}

// =============================================================================
// §3 — COMPONENT REGISTRY
// =============================================================================

/// Query parameters for registry lookups.
#[derive(Debug, Default, Clone)]
pub struct RegistryQuery {
    pub item_types:      Option<Vec<String>>,
    pub statuses:        Option<Vec<String>>,
    pub owner_id:        Option<EntityId>,
    pub tags:            Option<Vec<String>>,
    pub domain:          Option<String>,
    pub parent_id:       Option<ComponentId>,
    pub search_text:     Option<String>,
    pub exclude_deleted: bool,
    pub exclude_archived: bool,
    pub limit:           Option<usize>,
    pub offset:          usize,
}

/// Sort field for registry queries.
#[derive(Debug, Clone, PartialEq)]
pub enum RegistrySortField { Name, CreatedAt, UpdatedAt, HealthScore, RiskScore }

#[derive(Debug, Clone, PartialEq)]
pub enum SortDir { Asc, Desc }

/// A paginated result from the registry.
#[derive(Debug, Clone)]
pub struct RegistryPage {
    pub entries:    Vec<RegistryEntry>,
    pub total:      usize,
    pub offset:     usize,
    pub limit:      usize,
}

/// The master component registry.
///
/// Invariant: every PortfolioRow inserted into a PortfolioWorkbook is also
/// registered here.  The registry is the authoritative routing table for all
/// cross-system component references in the Kogi platform.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ComponentRegistry {
    /// Primary store: ComponentId → RegistryEntry.
    index:        HashMap<ComponentId, RegistryEntry>,
    /// Insertion-order list for default pagination.
    order:        Vec<ComponentId>,
    // Secondary indexes
    owner_idx:    OwnerIndex,
    type_idx:     TypeIndex,
    status_idx:   StatusIndex,
    tag_idx:      TagIndex,
    domain_idx:   DomainIndex,
    parent_idx:   ParentIndex,
    text_idx:     FullTextIndex,
}

impl ComponentRegistry {
    pub fn new() -> Self { Self::default() }

    // ── Write operations ─────────────────────────────────────────────────────

    /// Register a new component or replace an existing one.
    /// All secondary indexes are updated atomically.
    pub fn register(&mut self, entry: RegistryEntry) {
        let id = entry.component_id;

        // Remove stale index entries if replacing.
        if let Some(old) = self.index.remove(&id) {
            self.remove_from_indexes(&old);
        } else {
            self.order.push(id);
        }

        self.add_to_indexes(&entry);
        self.index.insert(id, entry);
    }

    /// Register from a PortfolioRow directly.
    pub fn register_row(&mut self, row: &PortfolioRow) {
        self.register(RegistryEntry::from_row(row));
    }

    /// Update the health_score cache on an existing entry.
    pub fn update_health_score(&mut self, id: &ComponentId, score: f64) {
        if let Some(e) = self.index.get_mut(id) {
            e.health_score = Some(score);
        }
    }

    /// Update the risk_score cache on an existing entry.
    pub fn update_risk_score(&mut self, id: &ComponentId, score: f64) {
        if let Some(e) = self.index.get_mut(id) {
            e.risk_score = Some(score);
        }
    }

    /// Remove a component from the registry and all indexes.
    pub fn deregister(&mut self, id: &ComponentId) {
        if let Some(entry) = self.index.remove(id) {
            self.remove_from_indexes(&entry);
            self.order.retain(|x| x != id);
        }
    }

    // ── Read operations ───────────────────────────────────────────────────────

    pub fn get(&self, id: &ComponentId) -> Option<&RegistryEntry> { self.index.get(id) }

    pub fn contains(&self, id: &ComponentId) -> bool { self.index.contains_key(id) }

    pub fn total_count(&self) -> usize { self.index.len() }

    /// Resolve a ComponentId to a display name (fast path for relation columns).
    pub fn display_name(&self, id: &ComponentId) -> Option<&str> {
        self.index.get(id).map(|e| {
            e.display_name.as_deref().unwrap_or(e.name.as_str())
        })
    }

    /// Return all direct children of a parent component.
    pub fn children(&self, parent: &ComponentId) -> Vec<&RegistryEntry> {
        self.parent_idx.children(parent)
            .iter()
            .filter_map(|id| self.index.get(id))
            .collect()
    }

    /// Return all transitive descendants of a component (BFS).
    pub fn descendants(&self, root: &ComponentId) -> Vec<ComponentId> {
        let mut result = Vec::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(*root);
        while let Some(cur) = queue.pop_front() {
            for child in self.parent_idx.children(&cur) {
                result.push(*child);
                queue.push_back(*child);
            }
        }
        result
    }

    /// All components owned by a given entity.
    pub fn by_owner(&self, owner: &EntityId) -> Vec<&RegistryEntry> {
        self.owner_idx.get(owner)
            .map(|ids| ids.iter().filter_map(|id| self.index.get(id)).collect())
            .unwrap_or_default()
    }

    /// All components with a given item_type.
    pub fn by_type(&self, type_key: &str) -> Vec<&RegistryEntry> {
        self.type_idx.get(type_key)
            .map(|ids| ids.iter().filter_map(|id| self.index.get(id)).collect())
            .unwrap_or_default()
    }

    /// All components in a given status.
    pub fn by_status(&self, status: &str) -> Vec<&RegistryEntry> {
        self.status_idx.get(status)
            .map(|ids| ids.iter().filter_map(|id| self.index.get(id)).collect())
            .unwrap_or_default()
    }

    /// All components with a given tag.
    pub fn by_tag(&self, tag: &str) -> Vec<&RegistryEntry> {
        self.tag_idx.get(tag)
            .map(|ids| ids.iter().filter_map(|id| self.index.get(id)).collect())
            .unwrap_or_default()
    }

    /// Composite query with filtering, sorting, and pagination.
    pub fn query(
        &self,
        q: &RegistryQuery,
        sort: Option<(RegistrySortField, SortDir)>,
    ) -> RegistryPage {
        // Start from type-filtered or all-entries candidate set.
        let candidates: Vec<&RegistryEntry> = {
            let by_type: Option<HashSet<ComponentId>> = q.item_types.as_ref().map(|types| {
                types.iter().flat_map(|t| {
                    self.type_idx.get(t).cloned().unwrap_or_default()
                }).collect()
            });

            let by_status: Option<HashSet<ComponentId>> = q.statuses.as_ref().map(|statuses| {
                statuses.iter().flat_map(|s| {
                    self.status_idx.get(s).cloned().unwrap_or_default()
                }).collect()
            });

            let by_owner: Option<HashSet<ComponentId>> = q.owner_id.as_ref().map(|oid| {
                self.owner_idx.get(oid).cloned().unwrap_or_default()
            });

            let by_tag: Option<HashSet<ComponentId>> = q.tags.as_ref().map(|tags| {
                tags.iter()
                    .map(|t| self.tag_idx.get(t).cloned().unwrap_or_default())
                    .fold(None::<HashSet<ComponentId>>, |acc, next| {
                        Some(acc.map_or(next.clone(), |a| a.intersection(&next).cloned().collect()))
                    })
                    .unwrap_or_default()
            });

            let by_parent: Option<HashSet<ComponentId>> = q.parent_id.as_ref().map(|pid| {
                self.parent_idx.children(pid).iter().cloned().collect()
            });

            let by_text: Option<HashSet<ComponentId>> = q.search_text.as_ref().map(|text| {
                let terms: Vec<&str> = text.split_whitespace().collect();
                self.text_idx.search_all_terms(&terms)
            });

            // Intersect all Some(sets) and fall back to full registry.
            let mut result_ids: Option<HashSet<ComponentId>> = None;
            for filter in [by_type, by_status, by_owner, by_tag, by_parent, by_text] {
                if let Some(ids) = filter {
                    result_ids = Some(match result_ids {
                        None    => ids,
                        Some(r) => r.intersection(&ids).cloned().collect(),
                    });
                }
            }

            match result_ids {
                Some(ids) => self.order.iter()
                    .filter_map(|id| if ids.contains(id) { self.index.get(id) } else { None })
                    .collect(),
                None => self.order.iter()
                    .filter_map(|id| self.index.get(id))
                    .collect(),
            }
        };

        // Apply post-filter predicates.
        let mut filtered: Vec<&RegistryEntry> = candidates.into_iter().filter(|e| {
            if q.exclude_deleted  && e.is_deleted  { return false; }
            if q.exclude_archived && e.is_archived { return false; }
            if let Some(d) = &q.domain {
                if e.domain.as_deref() != Some(d.as_str()) { return false; }
            }
            true
        }).collect();

        let total = filtered.len();

        // Sort.
        if let Some((field, dir)) = sort {
            filtered.sort_by(|a, b| {
                let cmp = match field {
                    RegistrySortField::Name      => a.name.cmp(&b.name),
                    RegistrySortField::CreatedAt => a.created_at.cmp(&b.created_at),
                    RegistrySortField::UpdatedAt => a.updated_at.cmp(&b.updated_at),
                    RegistrySortField::HealthScore => {
                        a.health_score.partial_cmp(&b.health_score).unwrap_or(std::cmp::Ordering::Equal)
                    },
                    RegistrySortField::RiskScore => {
                        a.risk_score.partial_cmp(&b.risk_score).unwrap_or(std::cmp::Ordering::Equal)
                    },
                };
                if dir == SortDir::Desc { cmp.reverse() } else { cmp }
            });
        }

        // Paginate.
        let limit = q.limit.unwrap_or(100);
        let page: Vec<RegistryEntry> = filtered.into_iter()
            .skip(q.offset)
            .take(limit)
            .cloned()
            .collect();

        RegistryPage { entries: page, total, offset: q.offset, limit }
    }

    // ── Index maintenance helpers ─────────────────────────────────────────────

    fn add_to_indexes(&mut self, e: &RegistryEntry) {
        for owner in &e.owners { self.owner_idx.add(*owner, e.component_id); }
        if let Some(it) = &e.item_type { self.type_idx.add(it, e.component_id); }
        if let Some(ct) = &e.container_type { self.type_idx.add(ct, e.component_id); }
        self.status_idx.add(&e.status, e.component_id);
        self.tag_idx.add_tags(&e.tags, e.component_id);
        if let Some(d) = &e.domain { self.domain_idx.add(d, e.component_id); }
        if let Some(pid) = e.parent_id { self.parent_idx.add_child(pid, e.component_id); }
        self.text_idx.index_entry(e);
    }

    fn remove_from_indexes(&mut self, e: &RegistryEntry) {
        for owner in &e.owners { self.owner_idx.remove(owner, &e.component_id); }
        if let Some(it) = &e.item_type { self.type_idx.remove(it, &e.component_id); }
        if let Some(ct) = &e.container_type { self.type_idx.remove(ct, &e.component_id); }
        self.status_idx.remove(&e.status, &e.component_id);
        self.tag_idx.remove_tags(&e.tags, &e.component_id);
        if let Some(d) = &e.domain { self.domain_idx.remove(d, &e.component_id); }
        if let Some(pid) = e.parent_id { self.parent_idx.remove_child(&pid, &e.component_id); }
        self.text_idx.remove_entry(e);
    }

    // ── Slug management ───────────────────────────────────────────────────────

    /// Check whether a slug is already in use by any entry (for uniqueness enforcement).
    pub fn slug_exists(&self, slug: &str) -> bool {
        self.index.values().any(|e| e.slug == slug)
    }

    /// Generate a unique slug from a proposed name (appends a counter if needed).
    pub fn unique_slug(&self, base: &str) -> String {
        let base_slug = base.to_lowercase()
            .replace(|c: char| !c.is_alphanumeric() && c != '-', "-")
            .trim_matches('-')
            .to_string();
        if !self.slug_exists(&base_slug) { return base_slug; }
        let mut counter = 2u32;
        loop {
            let candidate = format!("{base_slug}-{counter}");
            if !self.slug_exists(&candidate) { return candidate; }
            counter += 1;
        }
    }

    // ── Statistics ────────────────────────────────────────────────────────────

    /// Count components per item_type.
    pub fn type_counts(&self) -> HashMap<String, usize> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for entry in self.index.values() {
            if let Some(t) = &entry.item_type {
                *counts.entry(t.clone()).or_insert(0) += 1;
            }
        }
        counts
    }

    /// Count components per status.
    pub fn status_counts(&self) -> HashMap<String, usize> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for entry in self.index.values() {
            *counts.entry(entry.status.clone()).or_insert(0) += 1;
        }
        counts
    }

    /// Average health score across all entries that have one.
    pub fn avg_health_score(&self) -> Option<f64> {
        let scores: Vec<f64> = self.index.values()
            .filter_map(|e| e.health_score)
            .collect();
        if scores.is_empty() { return None; }
        Some(scores.iter().sum::<f64>() / scores.len() as f64)
    }

    /// Count entries that are currently active and not archived.
    pub fn active_count(&self) -> usize {
        self.index.values()
            .filter(|e| e.status == "Active" && !e.is_archived && !e.is_deleted)
            .count()
    }
}

// =============================================================================
// §4 — TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portfolio_spreadsheet_substrate::PortfolioRow;

    fn make_row(name: &str, item_type: &str, owner: EntityId, tags: Vec<&str>) -> PortfolioRow {
        let mut r = PortfolioRow::new(Uuid::new_v4(), "item", name, owner);
        r.item_type = Some(item_type.into());
        r.tags      = tags.into_iter().map(String::from).collect();
        r.status    = "Active".into();
        r
    }

    #[test]
    fn test_register_and_retrieve() {
        let owner = Uuid::new_v4();
        let mut registry = ComponentRegistry::new();
        let row = make_row("API Gateway", "Project", owner, vec!["api","backend"]);
        let id  = row.component_id;
        registry.register_row(&row);

        let entry = registry.get(&id).unwrap();
        assert_eq!(entry.name, "API Gateway");
        assert_eq!(entry.item_type.as_deref(), Some("Project"));
    }

    #[test]
    fn test_type_index() {
        let owner = Uuid::new_v4();
        let mut registry = ComponentRegistry::new();
        for name in ["Proj A", "Proj B"] {
            registry.register_row(&make_row(name, "Project", owner, vec![]));
        }
        registry.register_row(&make_row("Grant X", "Grant", owner, vec![]));

        assert_eq!(registry.by_type("Project").len(), 2);
        assert_eq!(registry.by_type("Grant").len(),   1);
    }

    #[test]
    fn test_owner_index() {
        let owner_a = Uuid::new_v4();
        let owner_b = Uuid::new_v4();
        let mut registry = ComponentRegistry::new();
        for _ in 0..3 { registry.register_row(&make_row("Row", "Project", owner_a, vec![])); }
        registry.register_row(&make_row("Row", "Asset", owner_b, vec![]));

        assert_eq!(registry.by_owner(&owner_a).len(), 3);
        assert_eq!(registry.by_owner(&owner_b).len(), 1);
    }

    #[test]
    fn test_tag_index() {
        let owner = Uuid::new_v4();
        let mut registry = ComponentRegistry::new();
        registry.register_row(&make_row("A", "Project", owner, vec!["rust","api"]));
        registry.register_row(&make_row("B", "Project", owner, vec!["rust","ui"]));
        registry.register_row(&make_row("C", "Asset",   owner, vec!["ui"]));

        assert_eq!(registry.by_tag("rust").len(), 2);
        assert_eq!(registry.by_tag("ui").len(),   2);
        assert_eq!(registry.by_tag("api").len(),  1);
    }

    #[test]
    fn test_full_text_search() {
        let owner = Uuid::new_v4();
        let mut registry = ComponentRegistry::new();
        let mut r = make_row("API Gateway Refactor", "Project", owner, vec!["backend"]);
        r.slug = "api-gateway-refactor".into();
        registry.register_row(&r);

        let results = registry.text_idx.search_prefix("api");
        assert!(results.contains(&r.component_id));

        let results2 = registry.text_idx.search_prefix("gate");
        assert!(results2.contains(&r.component_id));
    }

    #[test]
    fn test_composite_query() {
        let owner = Uuid::new_v4();
        let mut registry = ComponentRegistry::new();
        for i in 0..5 {
            let mut r = make_row(&format!("Project {i}"), "Project", owner, vec!["q2"]);
            if i % 2 == 0 { r.status = "Completed".into(); }
            registry.register_row(&r);
        }
        registry.register_row(&make_row("Grant X", "Grant", owner, vec!["q2"]));

        let q = RegistryQuery {
            item_types: Some(vec!["Project".into()]),
            statuses:   Some(vec!["Active".into()]),
            ..Default::default()
        };
        let page = registry.query(&q, None);
        assert_eq!(page.total, 2);

        let q2 = RegistryQuery {
            search_text: Some("Project".into()),
            ..Default::default()
        };
        let page2 = registry.query(&q2, None);
        assert_eq!(page2.total, 5);
    }

    #[test]
    fn test_deregister() {
        let owner = Uuid::new_v4();
        let mut registry = ComponentRegistry::new();
        let row = make_row("Temp Row", "Task", owner, vec!["tmp"]);
        let id  = row.component_id;
        registry.register_row(&row);
        assert!(registry.contains(&id));

        registry.deregister(&id);
        assert!(!registry.contains(&id));
        assert!(registry.by_tag("tmp").is_empty());
    }

    #[test]
    fn test_unique_slug() {
        let owner = Uuid::new_v4();
        let mut registry = ComponentRegistry::new();
        let mut r = make_row("My Project", "Project", owner, vec![]);
        r.slug = registry.unique_slug("my-project");
        registry.register_row(&r);

        let slug2 = registry.unique_slug("my-project");
        assert_eq!(slug2, "my-project-2");

        let mut r2 = make_row("My Project", "Project", owner, vec![]);
        r2.slug = slug2;
        registry.register_row(&r2);
        assert_eq!(registry.unique_slug("my-project"), "my-project-3");
    }

    #[test]
    fn test_hierarchy_index() {
        let owner = Uuid::new_v4();
        let mut registry = ComponentRegistry::new();
        let parent_id = Uuid::new_v4();

        let mut parent = make_row("Portfolio A", "Portfolio", owner, vec![]);
        parent.component_id = parent_id;
        registry.register_row(&parent);

        for _ in 0..3 {
            let mut child = make_row("Project", "Project", owner, vec![]);
            child.parent_id = Some(parent_id);
            registry.register_row(&child);
        }

        assert_eq!(registry.children(&parent_id).len(), 3);
        assert_eq!(registry.descendants(&parent_id).len(), 3);
    }

    #[test]
    fn test_type_counts_and_stats() {
        let owner = Uuid::new_v4();
        let mut registry = ComponentRegistry::new();
        for _ in 0..4 { registry.register_row(&make_row("P", "Project",  owner, vec![])); }
        for _ in 0..2 { registry.register_row(&make_row("G", "Grant",    owner, vec![])); }

        let counts = registry.type_counts();
        assert_eq!(*counts.get("Project").unwrap(), 4);
        assert_eq!(*counts.get("Grant").unwrap(),   2);
        assert_eq!(registry.active_count(), 6);
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
//  portfolio_registry.rs — Kogi OS · Portfolio Component Registry (KPRG)
//  Independent Worker Operating System
//
//  KPRG is the canonical master index of all PortfolioComponents system-wide.
//  It provides O(1) look-ups by component ID and O(log n) look-ups by owner,
//  type, status, tag, and full-text match.  Every mutation to a component
//  flows through the registry's index update methods to keep all views
//  consistent.
//
//  SDD §§ implemented here:
//    §2.1 Layer L1 — Registry
//    §3   Universal Row Schema (PortfolioRow index)
//    §13  API surface: SearchComponents, QueryPQL
//    §14  Indexing Strategy (§14.2)
//    §15  Row-Level Access Control
//
//  Indices provided:
//    ComponentIndex    — primary key (ComponentId → metadata snapshot)
//    OwnerIndex        — EntityId → Set<ComponentId>
//    TypeIndex         — type_name (String) → Set<ComponentId>
//    StatusIndex       — status string → Set<ComponentId>
//    TagIndex          — tag string → Set<ComponentId>
//    VisibilityIndex   — visibility string → Set<ComponentId>
//    FullTextIndex     — token → Set<ComponentId>  (simple inverted index)
//    HealthScoreIndex  — sorted by health_score (BTreeMap<OrderedDecimal, Set<ComponentId>>)
//    FinancialIndex    — secondary numeric index for budget_spent, budget_allocated
//    HierarchyIndex    — parent_id → children set; child_id → parent_id
//
//  @version  2.2.0
//  @license  MIT
// =============================================================================

#![allow(dead_code)]

use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Cross-module type aliases ─────────────────────────────────────────────────
pub type ComponentId = Uuid;
pub type EntityId    = Uuid;
pub type PolicyId    = Uuid;
pub type TagId       = Uuid;

// =============================================================================
// §1 — ComponentSummary
//
// A lightweight metadata snapshot of a PortfolioComponent stored in the
// registry.  It contains only the indexed fields — not the full cell store.
// =============================================================================

/// Permission tier (§15.1 of SDD).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PermissionTier {
    Public    = 0,
    Viewer    = 1,
    Contributor = 2,
    Editor    = 3,
    Admin     = 4,
    Owner     = 5,
}

/// Lightweight component snapshot stored in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentSummary {
    pub component_id:   ComponentId,
    pub slug:           Option<String>,
    pub component_type: String,     // "item" | "container"
    pub type_name:      String,     // "Project" | "Asset" | "Binder" | …
    pub name:           String,
    pub status:         String,
    pub visibility:     String,
    pub lifecycle_stage:String,

    pub owners:         Vec<EntityId>,
    pub tags:           Vec<String>,
    pub policy_ids:     Vec<PolicyId>,

    pub parent_id:      Option<ComponentId>,
    pub program_ref:    Option<ComponentId>,

    /// Latest computed health score (0–100); updated by engine writeback.
    pub health_score:   Option<f32>,
    /// Latest computed risk score (0–100).
    pub risk_score:     Option<f32>,

    pub budget_allocated: Option<Decimal>,
    pub budget_spent:     Decimal,

    pub start_date:     Option<chrono::NaiveDate>,
    pub due_date:       Option<chrono::NaiveDate>,

    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
    pub archived_at:    Option<DateTime<Utc>>,
}

impl ComponentSummary {
    pub fn new(
        id: ComponentId,
        component_type: impl Into<String>,
        type_name: impl Into<String>,
        name: impl Into<String>,
        owner: EntityId,
    ) -> Self {
        let now = Utc::now();
        Self {
            component_id: id,
            slug: None,
            component_type: component_type.into(),
            type_name: type_name.into(),
            name: name.into(),
            status: "Draft".into(),
            visibility: "Private".into(),
            lifecycle_stage: "Concept".into(),
            owners: vec![owner],
            tags: vec![],
            policy_ids: vec![],
            parent_id: None,
            program_ref: None,
            health_score: None,
            risk_score: None,
            budget_allocated: None,
            budget_spent: Decimal::ZERO,
            start_date: None,
            due_date: None,
            created_at: now,
            updated_at: now,
            archived_at: None,
        }
    }

    /// Returns `true` if the component is accessible at the given tier.
    pub fn is_accessible_by(&self, caller_tier: PermissionTier) -> bool {
        let required = match self.visibility.as_str() {
            "Private"   => PermissionTier::Owner,
            "Protected" => PermissionTier::Contributor,
            "Internal"  => PermissionTier::Viewer,
            "Public"    => PermissionTier::Public,
            _           => PermissionTier::Viewer,
        };
        caller_tier >= required
    }

    pub fn is_archived(&self)  -> bool { self.status == "Archived" }
    pub fn is_active(&self)    -> bool { self.status == "Active" }
}

// =============================================================================
// §2 — Index structures
// =============================================================================

/// Primary key index: ComponentId → ComponentSummary.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ComponentIndex {
    entries: HashMap<ComponentId, ComponentSummary>,
}

impl ComponentIndex {
    pub fn new() -> Self { Self::default() }
    pub fn insert(&mut self, s: ComponentSummary) { self.entries.insert(s.component_id, s); }
    pub fn get(&self, id: &ComponentId) -> Option<&ComponentSummary> { self.entries.get(id) }
    pub fn get_mut(&mut self, id: &ComponentId) -> Option<&mut ComponentSummary> { self.entries.get_mut(id) }
    pub fn remove(&mut self, id: &ComponentId) -> Option<ComponentSummary> { self.entries.remove(id) }
    pub fn contains(&self, id: &ComponentId) -> bool { self.entries.contains_key(id) }
    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
    pub fn all(&self) -> impl Iterator<Item = &ComponentSummary> { self.entries.values() }
}

/// Inverted index: a string key → set of ComponentIds.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct InvertedIndex {
    map: HashMap<String, HashSet<ComponentId>>,
}

impl InvertedIndex {
    pub fn new() -> Self { Self::default() }

    pub fn insert(&mut self, key: impl Into<String>, id: ComponentId) {
        self.map.entry(key.into()).or_default().insert(id);
    }

    pub fn remove(&mut self, key: &str, id: &ComponentId) {
        if let Some(set) = self.map.get_mut(key) {
            set.remove(id);
            if set.is_empty() { self.map.remove(key); }
        }
    }

    pub fn get(&self, key: &str) -> Option<&HashSet<ComponentId>> {
        self.map.get(key)
    }

    pub fn get_or_empty(&self, key: &str) -> &HashSet<ComponentId> {
        self.map.get(key).unwrap_or(&EMPTY_SET)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> { self.map.keys() }
}

static EMPTY_SET: std::sync::OnceLock<HashSet<ComponentId>> = std::sync::OnceLock::new();
fn empty_set() -> &'static HashSet<ComponentId> {
    EMPTY_SET.get_or_init(HashSet::new)
}

/// Hierarchy index: parent_id → Set<child_id>; child_id → parent_id.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct HierarchyIndex {
    parent_to_children: HashMap<ComponentId, HashSet<ComponentId>>,
    child_to_parent:    HashMap<ComponentId, ComponentId>,
}

impl HierarchyIndex {
    pub fn new() -> Self { Self::default() }

    pub fn attach(&mut self, parent: ComponentId, child: ComponentId) {
        // Remove child from any old parent
        if let Some(old_parent) = self.child_to_parent.remove(&child) {
            if let Some(set) = self.parent_to_children.get_mut(&old_parent) {
                set.remove(&child);
            }
        }
        self.parent_to_children.entry(parent).or_default().insert(child);
        self.child_to_parent.insert(child, parent);
    }

    pub fn detach(&mut self, child: &ComponentId) {
        if let Some(parent) = self.child_to_parent.remove(child) {
            if let Some(set) = self.parent_to_children.get_mut(&parent) {
                set.remove(child);
            }
        }
    }

    pub fn children(&self, parent: &ComponentId) -> &HashSet<ComponentId> {
        self.parent_to_children.get(parent).unwrap_or_else(empty_set)
    }

    pub fn parent(&self, child: &ComponentId) -> Option<&ComponentId> {
        self.child_to_parent.get(child)
    }

    pub fn has_children(&self, id: &ComponentId) -> bool {
        self.parent_to_children.get(id).map(|s| !s.is_empty()).unwrap_or(false)
    }

    /// Collect all transitive descendants of `root` (BFS).
    pub fn descendants(&self, root: &ComponentId) -> Vec<ComponentId> {
        let mut result = Vec::new();
        let mut queue  = std::collections::VecDeque::new();
        queue.push_back(*root);
        while let Some(current) = queue.pop_front() {
            for &child in self.children(&current) {
                result.push(child);
                queue.push_back(child);
            }
        }
        result
    }

    /// Detect a cycle: returns `true` if attaching `child` under `parent`
    /// would create a cycle.
    pub fn would_create_cycle(&self, parent: &ComponentId, child: &ComponentId) -> bool {
        if parent == child { return true; }
        let ancestors = self.ancestors(parent);
        ancestors.contains(child)
    }

    /// Collect all ancestors of a node (root last).
    pub fn ancestors(&self, id: &ComponentId) -> HashSet<ComponentId> {
        let mut visited = HashSet::new();
        let mut current = *id;
        while let Some(&p) = self.child_to_parent.get(&current) {
            if !visited.insert(p) { break; } // cycle guard
            current = p;
        }
        visited
    }
}

/// Numeric sorted index for f32 scores (health, risk, match).
/// Implemented as BTreeMap<OrderedF32, HashSet<ComponentId>>.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ScoreIndex {
    /// Truncated to 2 decimal places for bucketing.
    buckets: BTreeMap<u32, HashSet<ComponentId>>, // key = (score * 100) as u32
}

impl ScoreIndex {
    pub fn new() -> Self { Self::default() }

    fn key(score: f32) -> u32 { (score.clamp(0.0, 100.0) * 100.0) as u32 }

    pub fn insert(&mut self, score: f32, id: ComponentId) {
        self.buckets.entry(Self::key(score)).or_default().insert(id);
    }

    pub fn remove_with_score(&mut self, score: f32, id: &ComponentId) {
        let k = Self::key(score);
        if let Some(set) = self.buckets.get_mut(&k) {
            set.remove(id);
            if set.is_empty() { self.buckets.remove(&k); }
        }
    }

    /// Return IDs with score in [min_score, max_score].
    pub fn range(&self, min_score: f32, max_score: f32) -> Vec<ComponentId> {
        let lo = Self::key(min_score);
        let hi = Self::key(max_score);
        self.buckets.range(lo..=hi)
            .flat_map(|(_, set)| set.iter().copied())
            .collect()
    }

    /// Return top-N component IDs by score (highest first).
    pub fn top_n(&self, n: usize) -> Vec<ComponentId> {
        self.buckets.iter().rev()
            .flat_map(|(_, set)| set.iter().copied())
            .take(n)
            .collect()
    }
}

// =============================================================================
// §3 — Full-Text Index  (simple tokenised inverted index)
// =============================================================================

/// Simple token-based inverted index for name, tags, description.
/// In production this would back Meilisearch/Typesense (§14.1).
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FullTextIndex {
    token_index: HashMap<String, HashSet<ComponentId>>,
}

impl FullTextIndex {
    pub fn new() -> Self { Self::default() }

    fn tokenise(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| t.len() > 1)
            .map(|t| t.to_string())
            .collect()
    }

    pub fn index(&mut self, id: ComponentId, text: &str) {
        for token in Self::tokenise(text) {
            self.token_index.entry(token).or_default().insert(id);
        }
    }

    pub fn deindex(&mut self, id: &ComponentId, text: &str) {
        for token in Self::tokenise(text) {
            if let Some(set) = self.token_index.get_mut(&token) {
                set.remove(id);
                if set.is_empty() { self.token_index.remove(&token); }
            }
        }
    }

    /// Return all component IDs that contain ALL query tokens (AND logic).
    pub fn search(&self, query: &str) -> HashSet<ComponentId> {
        let tokens = Self::tokenise(query);
        if tokens.is_empty() { return HashSet::new(); }

        let mut result: Option<HashSet<ComponentId>> = None;
        for token in &tokens {
            let matches = self.token_index.get(token.as_str())
                .cloned()
                .unwrap_or_default();
            result = Some(match result {
                None    => matches,
                Some(r) => r.intersection(&matches).copied().collect(),
            });
        }
        result.unwrap_or_default()
    }

    /// Return all IDs containing ANY query token (OR logic).
    pub fn search_any(&self, query: &str) -> HashSet<ComponentId> {
        Self::tokenise(query).iter()
            .flat_map(|t| {
                self.token_index.get(t.as_str())
                    .into_iter()
                    .flat_map(|s| s.iter().copied())
            })
            .collect()
    }
}

// =============================================================================
// §4 — SearchQuery and SearchResult
// =============================================================================

/// A structured search request for the registry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RegistrySearchQuery {
    /// Full-text query string (applied to name, tags, description).
    pub q:              Option<String>,

    /// Filter to these type_names (OR logic).
    pub types:          Vec<String>,

    /// Filter to these status values (OR logic).
    pub statuses:       Vec<String>,

    /// Filter to these owners (OR logic).
    pub owners:         Vec<EntityId>,

    /// Filter to components with ALL of these tags.
    pub tags:           Vec<String>,

    /// Filter to components with health_score >= this value.
    pub min_health:     Option<f32>,

    /// Filter to components with risk_score >= this value.
    pub min_risk:       Option<f32>,

    /// Exclude archived components (default: true).
    pub exclude_archived: bool,

    /// Max results to return (default: 50).
    pub limit:          usize,

    /// Offset for pagination.
    pub offset:         usize,

    /// Sort field name ("health_score" | "name" | "created_at" | "budget_spent").
    pub sort_by:        Option<String>,

    /// Sort direction.
    pub sort_desc:      bool,
}

impl RegistrySearchQuery {
    pub fn new() -> Self {
        Self { exclude_archived: true, limit: 50, ..Default::default() }
    }
    pub fn with_type(mut self, t: impl Into<String>) -> Self {
        self.types.push(t.into()); self
    }
    pub fn with_status(mut self, s: impl Into<String>) -> Self {
        self.statuses.push(s.into()); self
    }
    pub fn with_owner(mut self, o: EntityId) -> Self {
        self.owners.push(o); self
    }
    pub fn with_tag(mut self, t: impl Into<String>) -> Self {
        self.tags.push(t.into()); self
    }
    pub fn with_q(mut self, q: impl Into<String>) -> Self {
        self.q = Some(q.into()); self
    }
    pub fn limit(mut self, n: usize) -> Self { self.limit = n; self }
}

/// One row in a search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultEntry {
    pub component_id: ComponentId,
    pub type_name:    String,
    pub name:         String,
    pub status:       String,
    pub health_score: Option<f32>,
    pub score:        f32,          // relevance score
}

/// Paginated search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrySearchResult {
    pub entries:      Vec<SearchResultEntry>,
    pub total_count:  usize,
    pub offset:       usize,
    pub limit:        usize,
}

// =============================================================================
// §5 — MasterRegistry
// =============================================================================

/// The canonical master registry of all PortfolioComponents for one portfolio
/// system.  Holds all indices and exposes search + graph traversal.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MasterRegistry {
    // ── Primary ────────────────────────────────────────────────────────────
    pub components:  ComponentIndex,

    // ── Inverted indices ───────────────────────────────────────────────────
    pub owner_index:      InvertedIndex,  // EntityId (string) → component IDs
    pub type_index:       InvertedIndex,  // type_name → component IDs
    pub status_index:     InvertedIndex,  // status → component IDs
    pub tag_index:        InvertedIndex,  // tag → component IDs
    pub visibility_index: InvertedIndex,  // visibility → component IDs
    pub stage_index:      InvertedIndex,  // lifecycle_stage → component IDs

    // ── Structural ─────────────────────────────────────────────────────────
    pub hierarchy:   HierarchyIndex,

    // ── Score indices ──────────────────────────────────────────────────────
    pub health_index: ScoreIndex,
    pub risk_index:   ScoreIndex,

    // ── Full-text ──────────────────────────────────────────────────────────
    pub fts:         FullTextIndex,

    // ── Slug lookup ────────────────────────────────────────────────────────
    slug_map:        HashMap<String, ComponentId>,

    pub created_at:  DateTime<Utc>,
    pub updated_at:  DateTime<Utc>,
}

impl MasterRegistry {
    pub fn new() -> Self {
        Self { created_at: Utc::now(), updated_at: Utc::now(), ..Default::default() }
    }

    // ── Mutation ────────────────────────────────────────────────────────────

    /// Register a new component in all indices.
    pub fn register(&mut self, summary: ComponentSummary) {
        let id = summary.component_id;

        // FTS
        self.fts.index(id, &summary.name);
        if let Some(ref slug) = summary.slug {
            self.slug_map.insert(slug.clone(), id);
            self.fts.index(id, slug);
        }

        // Inverted indices
        for owner in &summary.owners {
            self.owner_index.insert(owner.to_string(), id);
        }
        self.type_index.insert(&summary.type_name, id);
        self.status_index.insert(&summary.status, id);
        self.visibility_index.insert(&summary.visibility, id);
        self.stage_index.insert(&summary.lifecycle_stage, id);
        for tag in &summary.tags {
            self.tag_index.insert(tag, id);
        }

        // Score indices
        if let Some(hs) = summary.health_score {
            self.health_index.insert(hs, id);
        }
        if let Some(rs) = summary.risk_score {
            self.risk_index.insert(rs, id);
        }

        // Hierarchy
        if let Some(parent) = summary.parent_id {
            self.hierarchy.attach(parent, id);
        }

        self.components.insert(summary);
        self.updated_at = Utc::now();
    }

    /// Update indexed fields of an existing component.
    /// Call after any mutation to keep indices consistent.
    pub fn update(&mut self, updated: ComponentSummary) {
        let id = updated.component_id;
        if let Some(old) = self.components.get(&id).cloned() {
            self.deindex_component(&old);
        }
        self.register(updated);
    }

    /// Remove a component from all indices.
    pub fn remove(&mut self, id: &ComponentId) -> Option<ComponentSummary> {
        let summary = self.components.remove(id)?;
        self.deindex_component(&summary);
        self.hierarchy.detach(id);
        self.updated_at = Utc::now();
        Some(summary)
    }

    /// Update just the health score (called by engine writeback).
    pub fn update_health_score(&mut self, id: &ComponentId, score: f32) {
        if let Some(s) = self.components.get(id) {
            if let Some(old) = s.health_score {
                self.health_index.remove_with_score(old, id);
            }
        }
        if let Some(s) = self.components.get_mut(id) {
            s.health_score = Some(score);
        }
        self.health_index.insert(score, *id);
    }

    /// Update just the risk score (called by engine writeback).
    pub fn update_risk_score(&mut self, id: &ComponentId, score: f32) {
        if let Some(s) = self.components.get(id) {
            if let Some(old) = s.risk_score {
                self.risk_index.remove_with_score(old, id);
            }
        }
        if let Some(s) = self.components.get_mut(id) {
            s.risk_score = Some(score);
        }
        self.risk_index.insert(score, *id);
    }

    fn deindex_component(&mut self, s: &ComponentSummary) {
        let id = s.component_id;
        self.fts.deindex(&id, &s.name);
        if let Some(ref slug) = s.slug {
            self.slug_map.remove(slug);
        }
        for owner in &s.owners {
            self.owner_index.remove(&owner.to_string(), &id);
        }
        self.type_index.remove(&s.type_name, &id);
        self.status_index.remove(&s.status, &id);
        self.visibility_index.remove(&s.visibility, &id);
        self.stage_index.remove(&s.lifecycle_stage, &id);
        for tag in &s.tags {
            self.tag_index.remove(tag, &id);
        }
        if let Some(hs) = s.health_score { self.health_index.remove_with_score(hs, &id); }
        if let Some(rs) = s.risk_score   { self.risk_index.remove_with_score(rs, &id); }
    }

    // ── Lookups ─────────────────────────────────────────────────────────────

    pub fn get(&self, id: &ComponentId) -> Option<&ComponentSummary> {
        self.components.get(id)
    }

    pub fn get_by_slug(&self, slug: &str) -> Option<&ComponentSummary> {
        self.slug_map.get(slug).and_then(|id| self.components.get(id))
    }

    pub fn by_owner(&self, owner: &EntityId) -> Vec<&ComponentSummary> {
        self.owner_index
            .get_or_empty(&owner.to_string())
            .iter()
            .filter_map(|id| self.components.get(id))
            .collect()
    }

    pub fn by_type(&self, type_name: &str) -> Vec<&ComponentSummary> {
        self.type_index
            .get_or_empty(type_name)
            .iter()
            .filter_map(|id| self.components.get(id))
            .collect()
    }

    pub fn by_status(&self, status: &str) -> Vec<&ComponentSummary> {
        self.status_index
            .get_or_empty(status)
            .iter()
            .filter_map(|id| self.components.get(id))
            .collect()
    }

    pub fn by_tag(&self, tag: &str) -> Vec<&ComponentSummary> {
        self.tag_index
            .get_or_empty(tag)
            .iter()
            .filter_map(|id| self.components.get(id))
            .collect()
    }

    pub fn top_by_health(&self, n: usize) -> Vec<&ComponentSummary> {
        self.health_index.top_n(n)
            .into_iter()
            .filter_map(|id| self.components.get(&id))
            .collect()
    }

    pub fn at_risk(&self, min_risk: f32) -> Vec<&ComponentSummary> {
        self.risk_index.range(min_risk, 100.0)
            .into_iter()
            .filter_map(|id| self.components.get(&id))
            .collect()
    }

    pub fn children(&self, parent: &ComponentId) -> Vec<&ComponentSummary> {
        self.hierarchy.children(parent)
            .iter()
            .filter_map(|id| self.components.get(id))
            .collect()
    }

    pub fn descendants(&self, root: &ComponentId) -> Vec<&ComponentSummary> {
        self.hierarchy.descendants(root)
            .iter()
            .filter_map(|id| self.components.get(id))
            .collect()
    }

    // ── Hierarchy mutation ──────────────────────────────────────────────────

    pub fn attach_child(&mut self, parent: ComponentId, child: ComponentId)
        -> Result<(), String>
    {
        if self.hierarchy.would_create_cycle(&parent, &child) {
            return Err(format!("Attaching {child} under {parent} would create a cycle"));
        }
        self.hierarchy.attach(parent, child);
        if let Some(s) = self.components.get_mut(&child) {
            s.parent_id = Some(parent);
        }
        Ok(())
    }

    pub fn detach_child(&mut self, child: &ComponentId) {
        self.hierarchy.detach(child);
        if let Some(s) = self.components.get_mut(child) {
            s.parent_id = None;
        }
    }

    // ── Structured search ───────────────────────────────────────────────────

    pub fn search(&self, query: &RegistrySearchQuery) -> RegistrySearchResult {
        // Start from full-text or universe
        let candidate_ids: HashSet<ComponentId> = if let Some(ref q) = query.q {
            self.fts.search(q)
        } else {
            self.components.all().map(|s| s.component_id).collect()
        };

        // Apply inverted-index filters
        let mut pass: Vec<&ComponentSummary> = candidate_ids.iter()
            .filter_map(|id| self.components.get(id))
            .filter(|s| {
                if query.exclude_archived && s.is_archived() { return false; }

                if !query.types.is_empty()
                    && !query.types.contains(&s.type_name) { return false; }

                if !query.statuses.is_empty()
                    && !query.statuses.contains(&s.status) { return false; }

                if !query.owners.is_empty()
                    && !query.owners.iter().any(|o| s.owners.contains(o)) { return false; }

                if !query.tags.is_empty()
                    && !query.tags.iter().all(|t| s.tags.contains(t)) { return false; }

                if let Some(min_h) = query.min_health {
                    if s.health_score.unwrap_or(0.0) < min_h { return false; }
                }
                if let Some(min_r) = query.min_risk {
                    if s.risk_score.unwrap_or(0.0) < min_r { return false; }
                }
                true
            })
            .collect();

        // Sort
        if let Some(ref sort_field) = query.sort_by {
            pass.sort_by(|a, b| {
                let ord = match sort_field.as_str() {
                    "health_score" => a.health_score.unwrap_or(0.0)
                        .partial_cmp(&b.health_score.unwrap_or(0.0))
                        .unwrap_or(std::cmp::Ordering::Equal),
                    "risk_score"   => a.risk_score.unwrap_or(0.0)
                        .partial_cmp(&b.risk_score.unwrap_or(0.0))
                        .unwrap_or(std::cmp::Ordering::Equal),
                    "budget_spent" => a.budget_spent.cmp(&b.budget_spent),
                    "created_at"   => a.created_at.cmp(&b.created_at),
                    "name" | _     => a.name.cmp(&b.name),
                };
                if query.sort_desc { ord.reverse() } else { ord }
            });
        }

        let total_count = pass.len();
        let entries: Vec<SearchResultEntry> = pass
            .into_iter()
            .skip(query.offset)
            .take(query.limit)
            .map(|s| SearchResultEntry {
                component_id: s.component_id,
                type_name:    s.type_name.clone(),
                name:         s.name.clone(),
                status:       s.status.clone(),
                health_score: s.health_score,
                score:        s.health_score.unwrap_or(50.0),
            })
            .collect();

        RegistrySearchResult {
            entries,
            total_count,
            offset: query.offset,
            limit:  query.limit,
        }
    }

    // ── Stats & diagnostics ─────────────────────────────────────────────────

    pub fn total_count(&self) -> usize { self.components.len() }

    pub fn count_by_type(&self) -> HashMap<String, usize> {
        let mut map = HashMap::new();
        for s in self.components.all() {
            *map.entry(s.type_name.clone()).or_insert(0) += 1;
        }
        map
    }

    pub fn count_by_status(&self) -> HashMap<String, usize> {
        let mut map = HashMap::new();
        for s in self.components.all() {
            *map.entry(s.status.clone()).or_insert(0) += 1;
        }
        map
    }

    /// Average health score across all non-archived components.
    pub fn average_health_score(&self) -> Option<f32> {
        let scores: Vec<f32> = self.components.all()
            .filter(|s| !s.is_archived())
            .filter_map(|s| s.health_score)
            .collect();
        if scores.is_empty() { return None; }
        Some(scores.iter().sum::<f32>() / scores.len() as f32)
    }
}

// =============================================================================
// §6 — Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn uid() -> Uuid { Uuid::new_v4() }

    fn make_summary(type_name: &str, name: &str, owner: EntityId) -> ComponentSummary {
        ComponentSummary::new(uid(), "item", type_name, name, owner)
    }

    #[test]
    fn test_register_and_lookup() {
        let mut reg = MasterRegistry::new();
        let owner = uid();
        let s = make_summary("Project", "API Redesign", owner);
        let id = s.component_id;
        reg.register(s);

        assert!(reg.get(&id).is_some());
        assert_eq!(reg.get(&id).unwrap().name, "API Redesign");
    }

    #[test]
    fn test_inverted_index_by_type() {
        let mut reg = MasterRegistry::new();
        let owner = uid();
        for i in 0..3 {
            let mut s = make_summary("Project", &format!("Project {i}"), owner);
            s.status = "Active".into();
            reg.register(s);
        }
        let mut s2 = make_summary("Program", "Prog 0", owner);
        s2.status = "Active".into();
        reg.register(s2);

        assert_eq!(reg.by_type("Project").len(), 3);
        assert_eq!(reg.by_type("Program").len(), 1);
    }

    #[test]
    fn test_full_text_search() {
        let mut reg = MasterRegistry::new();
        let owner = uid();
        let mut s = make_summary("Project", "API Platform Redesign", owner);
        s.tags = vec!["api".into(), "platform".into()];
        let id = s.component_id;
        reg.register(s);

        let results = reg.fts.search("api platform");
        assert!(results.contains(&id));

        let no_results = reg.fts.search("unrelated");
        assert!(!no_results.contains(&id));
    }

    #[test]
    fn test_health_score_index() {
        let mut reg = MasterRegistry::new();
        let owner = uid();
        let mut s = make_summary("Portfolio", "My Portfolio", owner);
        s.health_score = Some(85.0);
        let id = s.component_id;
        reg.register(s);

        let top = reg.top_by_health(5);
        assert!(top.iter().any(|s| s.component_id == id));

        reg.update_health_score(&id, 45.0);
        assert_eq!(reg.get(&id).unwrap().health_score, Some(45.0));
    }

    #[test]
    fn test_hierarchy_attach_detach() {
        let mut reg = MasterRegistry::new();
        let owner = uid();
        let parent_s = make_summary("Program",  "Q2 Program", owner);
        let child_s  = make_summary("Project",  "API Project", owner);
        let parent   = parent_s.component_id;
        let child    = child_s.component_id;
        reg.register(parent_s);
        reg.register(child_s);

        reg.attach_child(parent, child).unwrap();
        assert_eq!(reg.children(&parent).len(), 1);
        assert_eq!(reg.hierarchy.parent(&child), Some(&parent));

        reg.detach_child(&child);
        assert_eq!(reg.children(&parent).len(), 0);
    }

    #[test]
    fn test_cycle_detection() {
        let mut reg = MasterRegistry::new();
        let owner = uid();
        let a = make_summary("Program", "A", owner);
        let b = make_summary("Project", "B", owner);
        let c = make_summary("Task",    "C", owner);
        let (aid, bid, cid) = (a.component_id, b.component_id, c.component_id);
        reg.register(a); reg.register(b); reg.register(c);

        reg.attach_child(aid, bid).unwrap();
        reg.attach_child(bid, cid).unwrap();

        // C → A would create a cycle
        assert!(reg.attach_child(cid, aid).is_err());
    }

    #[test]
    fn test_structured_search() {
        let mut reg = MasterRegistry::new();
        let owner = uid();
        for i in 0..5 {
            let mut s = make_summary("Project", &format!("Project {i}"), owner);
            s.status = if i < 3 { "Active".into() } else { "Completed".into() };
            s.tags = vec!["q2".into()];
            reg.register(s);
        }

        let query = RegistrySearchQuery::new()
            .with_type("Project")
            .with_status("Active")
            .with_tag("q2");

        let result = reg.search(&query);
        assert_eq!(result.total_count, 3);
    }

    #[test]
    fn test_count_diagnostics() {
        let mut reg = MasterRegistry::new();
        let owner = uid();
        let types = ["Project","Project","Program","Asset"];
        for t in &types {
            reg.register(make_summary(t, "x", owner));
        }
        let counts = reg.count_by_type();
        assert_eq!(counts["Project"], 2);
        assert_eq!(counts["Program"], 1);
    }

    #[test]
    fn test_descendants() {
        let mut reg = MasterRegistry::new();
        let owner = uid();
        let root = uid();
        let c1   = uid();
        let c2   = uid();
        let gc1  = uid();

        for (id, t, n) in [
            (root, "Portfolio", "Root"),
            (c1,   "Program",   "Child1"),
            (c2,   "Program",   "Child2"),
            (gc1,  "Project",   "GrandChild1"),
        ] {
            let mut s = ComponentSummary::new(id, "item", t, n, owner);
            reg.register(s);
        }

        reg.attach_child(root, c1).unwrap();
        reg.attach_child(root, c2).unwrap();
        reg.attach_child(c1,  gc1).unwrap();

        let desc = reg.descendants(&root);
        assert_eq!(desc.len(), 3);
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
