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
//  portfolio_computation_engine.rs — Kogi OS · KPCM v3.0
//  Portfolio Computation Engine
//
//  Powers all ComputedColumns, AIColumns, and rollup aggregations in the
//  portfolio spreadsheet. Runs on two tiers:
//
//    Tier 1 — Synchronous arithmetic:  simple derived columns evaluated inline
//             at query time (budget_remaining, profit, pto_remaining_days, …)
//
//    Tier 2 — Async engine computation: health scores, risk scores, match
//             scores, projections — computed by kogi-engine and written back to
//             the CellStore via the plugin writeback protocol.
//
//  Analytical models
//  ─────────────────
//    PortfolioHealthModel     — 7-dimension health score (0–100)
//    ProjectMetricsModel      — velocity, throughput, cycle-time, lead-time
//    ProgramAlignmentModel    — child project rollup to program KPIs
//    SubPortfolioRollupModel  — financial and status rollup to portfolio
//    ResourceUtilisationModel — allocation utilisation and conflict detection
//    CollaborationScoreModel  — contributor diversity × velocity × governance
//    BenefitsHealthModel      — benefit coverage and income-replacement score
//    RiskScoreModel           — severity × probability weighted risk aggregate
//    IncomeProjectionModel    — forward-looking income forecast
//    ColumnComputer           — formula expression evaluator (Tier 1)
//
//  @author  Kogi Team
//  @version 3.0.0
//  @license MIT
// =============================================================================

#![allow(dead_code)]

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::portfolio_spreadsheet_substrate::{
    CellStore, CellValue, ColumnId, ComponentId, EntityId, PortfolioRow, RowStore,
};

// =============================================================================
// §1 — SHARED TYPES
// =============================================================================

/// A single named sub-score within a composite model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreDimension {
    pub name:   String,
    pub score:  f64,
    pub weight: f64,
    pub detail: Option<String>,
}

/// An anomaly detected by any model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub id:       Uuid,
    pub kind:     AnomalyKind,
    pub message:  String,
    pub severity: AnomalySeverity,
    pub column:   Option<ColumnId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnomalyKind {
    BudgetOverrun, BudgetApproaching, VelocityDrop, ScheduleSlip, StalledProject,
    CoverageGap, RiskEscalation, GovernanceOverdue, ContributionStall, Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AnomalySeverity { Info, Warning, Error, Critical }

/// Engine writeback record: the computation engine writes computed values back
/// to the CellStore via this typed record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineWriteback {
    pub component_id:  ComponentId,
    pub column_id:     ColumnId,
    pub value:         CellValue,
    pub computed_by:   String,   // e.g. "HealthEngine", "RiskEngine"
    pub computed_at:   DateTime<Utc>,
    pub ttl_secs:      Option<u64>,
}

impl EngineWriteback {
    pub fn apply(&self, cs: &mut CellStore) {
        cs.set(
            self.component_id,
            self.column_id.clone(),
            self.value.clone(),
            self.computed_by.clone(),
            None,
            self.ttl_secs,
        );
    }
}

// =============================================================================
// §2 — PORTFOLIO HEALTH MODEL  (7 dimensions, 0–100 composite score)
// =============================================================================

/// Full PortfolioHealth output for a single portfolio or program row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioHealth {
    pub component_id:  ComponentId,
    pub health_score:  f64,
    pub dimensions:    Vec<ScoreDimension>,
    pub anomalies:     Vec<Anomaly>,
    pub computed_at:   DateTime<Utc>,
}

impl PortfolioHealth {
    /// Produce an EngineWriteback for the health_score column.
    pub fn to_writeback(&self) -> EngineWriteback {
        EngineWriteback {
            component_id: self.component_id,
            column_id:    "health_score".into(),
            value:        CellValue::Float(self.health_score),
            computed_by:  "PortfolioHealthModel".into(),
            computed_at:  self.computed_at,
            ttl_secs:     Some(3600),
        }
    }
}

/// Health dimension weights (must sum to 1.0).
pub struct HealthWeights {
    pub delivery:   f64,  // 0.25
    pub financial:  f64,  // 0.20
    pub risk:       f64,  // 0.20
    pub resource:   f64,  // 0.15
    pub engagement: f64,  // 0.10
    pub governance: f64,  // 0.05
    pub benefits:   f64,  // 0.05
}

impl Default for HealthWeights {
    fn default() -> Self {
        Self { delivery: 0.25, financial: 0.20, risk: 0.20, resource: 0.15, engagement: 0.10, governance: 0.05, benefits: 0.05 }
    }
}

pub struct PortfolioHealthModel;

impl PortfolioHealthModel {
    /// Compute the composite health score for a portfolio or program.
    ///
    /// `row`       — the portfolio/program row itself
    /// `children`  — all direct descendants (projects, tasks, etc.)
    /// `cs`        — CellStore for cached AI signals
    /// `weights`   — optional weight override (defaults to spec-standard weights)
    pub fn compute(
        row:     &PortfolioRow,
        children: &[&PortfolioRow],
        cs:       &CellStore,
        weights:  Option<HealthWeights>,
    ) -> PortfolioHealth {
        let w = weights.unwrap_or_default();
        let mut anomalies = Vec::new();

        // ── Delivery Health (25%) ─────────────────────────────────────────────
        let delivery_h = {
            let avg_progress: f64 = if children.is_empty() {
                row.progress_pct as f64
            } else {
                children.iter().map(|c| c.progress_pct as f64).sum::<f64>() / children.len() as f64
            };
            let schedule_penalty = children.iter()
                .filter_map(|c| c.schedule_variance_days())
                .filter(|&d| d > 0)
                .fold(0.0, |acc, d| acc + (d as f64).min(30.0) / 30.0 * 20.0);
            let score = (avg_progress - schedule_penalty).clamp(0.0, 100.0);
            if avg_progress < 30.0 && !children.is_empty() {
                anomalies.push(Anomaly { id: Uuid::new_v4(), kind: AnomalyKind::StalledProject, message: "Low aggregate progress across children.".into(), severity: AnomalySeverity::Warning, column: Some("progress_pct".into()) });
            }
            score
        };

        // ── Financial Health (20%) ────────────────────────────────────────────
        let financial_h = {
            let util = row.budget_utilization_pct().unwrap_or(0.0);
            let mut score = if util <= 80.0 { 100.0 } else { 100.0 - (util - 80.0) * 5.0 };
            score = score.clamp(0.0, 100.0);
            if util > 100.0 {
                anomalies.push(Anomaly { id: Uuid::new_v4(), kind: AnomalyKind::BudgetOverrun, message: format!("Budget overrun: {util:.1}% utilized."), severity: AnomalySeverity::Error, column: Some("budget_utilization_pct".into()) });
            } else if util > 80.0 {
                anomalies.push(Anomaly { id: Uuid::new_v4(), kind: AnomalyKind::BudgetApproaching, message: format!("Budget approaching limit: {util:.1}%."), severity: AnomalySeverity::Warning, column: Some("budget_utilization_pct".into()) });
            }
            score
        };

        // ── Risk Health (20%) ─────────────────────────────────────────────────
        let risk_h = {
            let cached_risk = cs.get(&row.component_id, "risk_score").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let open_critical = children.iter().filter(|c| c.risk_flags.iter().any(|r| r.severity == "Critical")).count() as f64;
            let score = (100.0 - cached_risk - open_critical * 10.0).clamp(0.0, 100.0);
            if open_critical > 0.0 {
                anomalies.push(Anomaly { id: Uuid::new_v4(), kind: AnomalyKind::RiskEscalation, message: format!("{open_critical} critical risk flags open."), severity: AnomalySeverity::Critical, column: Some("risk_flags".into()) });
            }
            score
        };

        // ── Resource Health (15%) ─────────────────────────────────────────────
        let resource_h = {
            let alloc = row.allocation_pct;
            if alloc > 100.0 { 40.0 }
            else if alloc < 30.0 { 60.0 }
            else if (60.0..=80.0).contains(&alloc) { 100.0 }
            else { 80.0 }
        };

        // ── Engagement Health (10%) ───────────────────────────────────────────
        let engagement_h = {
            let followers = row.followers as f64;
            let shares    = row.shares    as f64;
            (followers * 0.1 + shares * 0.5).min(100.0)
        };

        // ── Governance Health (5%) ────────────────────────────────────────────
        let governance_h = {
            let open_approvals  = row.approval_status.as_deref() == Some("Pending");
            let compliance_flags = row.compliance_flags.len();
            let mut score = 100.0f64;
            if open_approvals    { score -= 20.0; }
            score -= (compliance_flags as f64 * 15.0).min(60.0);
            if row.last_reviewed_at.is_none() { score -= 10.0; }
            score.clamp(0.0, 100.0)
        };

        // ── Benefits Health (5%) ──────────────────────────────────────────────
        let benefits_h: f64 = cs.get(&row.component_id, "benefits_health_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(70.0);

        let health_score =
            delivery_h   * w.delivery   +
            financial_h  * w.financial  +
            risk_h       * w.risk       +
            resource_h   * w.resource   +
            engagement_h * w.engagement +
            governance_h * w.governance +
            benefits_h   * w.benefits;

        let dimensions = vec![
            ScoreDimension { name: "Delivery".into(),   score: delivery_h,   weight: w.delivery,   detail: None },
            ScoreDimension { name: "Financial".into(),  score: financial_h,  weight: w.financial,  detail: None },
            ScoreDimension { name: "Risk".into(),       score: risk_h,       weight: w.risk,       detail: None },
            ScoreDimension { name: "Resource".into(),   score: resource_h,   weight: w.resource,   detail: None },
            ScoreDimension { name: "Engagement".into(), score: engagement_h, weight: w.engagement, detail: None },
            ScoreDimension { name: "Governance".into(), score: governance_h, weight: w.governance, detail: None },
            ScoreDimension { name: "Benefits".into(),   score: benefits_h,   weight: w.benefits,   detail: None },
        ];

        PortfolioHealth { component_id: row.component_id, health_score, dimensions, anomalies, computed_at: Utc::now() }
    }
}

// =============================================================================
// §3 — RISK SCORE MODEL
// =============================================================================

/// Probability-weighted aggregate risk score (0–100).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskScoreResult {
    pub component_id: ComponentId,
    pub risk_score:   f64,
    pub open_count:   u32,
    pub critical_count: u32,
    pub computed_at:  DateTime<Utc>,
}

impl RiskScoreResult {
    pub fn to_writeback(&self) -> EngineWriteback {
        EngineWriteback {
            component_id: self.component_id,
            column_id:    "risk_score".into(),
            value:        CellValue::Float(self.risk_score),
            computed_by:  "RiskEngine".into(),
            computed_at:  self.computed_at,
            ttl_secs:     Some(3600),
        }
    }
}

pub struct RiskScoreModel;

impl RiskScoreModel {
    /// severity weights: Critical=40, High=20, Medium=10, Low=5.
    fn severity_weight(sev: &str) -> f64 {
        match sev { "Critical" => 40.0, "High" => 20.0, "Medium" => 10.0, "Low" => 5.0, _ => 2.0 }
    }

    pub fn compute(row: &PortfolioRow, children: &[&PortfolioRow]) -> RiskScoreResult {
        let all_flags: Vec<_> = std::iter::once(row)
            .chain(children.iter().copied())
            .flat_map(|r| r.risk_flags.iter())
            .collect();

        let open_count    = all_flags.len() as u32;
        let critical_count = all_flags.iter().filter(|f| f.severity == "Critical").count() as u32;
        let raw_score: f64 = all_flags.iter().map(|f| Self::severity_weight(&f.severity)).sum();
        let risk_score = raw_score.min(100.0);

        RiskScoreResult { component_id: row.component_id, risk_score, open_count, critical_count, computed_at: Utc::now() }
    }
}

// =============================================================================
// §4 — PROJECT METRICS MODEL
// =============================================================================

/// Sprint velocity, cycle time, lead time, throughput for a project row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetrics {
    pub component_id:        ComponentId,
    pub velocity:            f64,   // story points per sprint
    pub throughput:          f64,   // tasks completed per week
    pub avg_cycle_time_days: f64,
    pub avg_lead_time_days:  f64,
    pub backlog_size:        u32,
    pub completed_tasks:     u32,
    pub computed_at:         DateTime<Utc>,
}

impl ProjectMetrics {
    pub fn writebacks(&self) -> Vec<EngineWriteback> {
        vec![
            EngineWriteback { component_id: self.component_id, column_id: "velocity".into(),        value: CellValue::Float(self.velocity),            computed_by: "ProjectMetrics".into(), computed_at: self.computed_at, ttl_secs: Some(3600) },
            EngineWriteback { component_id: self.component_id, column_id: "cycle_time_days".into(), value: CellValue::Float(self.avg_cycle_time_days), computed_by: "ProjectMetrics".into(), computed_at: self.computed_at, ttl_secs: Some(3600) },
            EngineWriteback { component_id: self.component_id, column_id: "lead_time_days".into(),  value: CellValue::Float(self.avg_lead_time_days),  computed_by: "ProjectMetrics".into(), computed_at: self.computed_at, ttl_secs: Some(3600) },
            EngineWriteback { component_id: self.component_id, column_id: "throughput".into(),      value: CellValue::Float(self.throughput),           computed_by: "ProjectMetrics".into(), computed_at: self.computed_at, ttl_secs: Some(3600) },
        ]
    }
}

pub struct ProjectMetricsModel;

impl ProjectMetricsModel {
    /// Compute metrics from a project's tasks (children with item_type=Task).
    pub fn compute(project: &PortfolioRow, tasks: &[&PortfolioRow]) -> ProjectMetrics {
        let completed: Vec<&&PortfolioRow> = tasks.iter().filter(|t| t.status == "Completed").collect();
        let completed_tasks = completed.len() as u32;

        // Velocity: story points completed (fallback: count of tasks per sprint).
        let sprint_count = project.sprint_refs.len().max(1);
        let velocity = completed_tasks as f64 / sprint_count as f64;

        // Throughput: tasks/week over the project duration.
        let duration_weeks = project.actual_duration_days().unwrap_or(7) as f64 / 7.0;
        let throughput = if duration_weeks > 0.0 { completed_tasks as f64 / duration_weeks } else { 0.0 };

        // Cycle time: avg days from start_date to end_date for completed tasks.
        let cycle_times: Vec<f64> = completed.iter().filter_map(|t| {
            t.actual_duration_days().map(|d| d as f64)
        }).collect();
        let avg_cycle_time_days = if cycle_times.is_empty() { 0.0 } else {
            cycle_times.iter().sum::<f64>() / cycle_times.len() as f64
        };

        // Lead time: avg days from created_at to end_date.
        let lead_times: Vec<f64> = completed.iter().filter_map(|t| {
            t.end_date.map(|end| {
                (end - t.created_at.date_naive()).num_days() as f64
            })
        }).collect();
        let avg_lead_time_days = if lead_times.is_empty() { 0.0 } else {
            lead_times.iter().sum::<f64>() / lead_times.len() as f64
        };

        let backlog_size = tasks.iter().filter(|t| t.status == "Draft" || t.status == "Active").count() as u32;

        ProjectMetrics { component_id: project.component_id, velocity, throughput, avg_cycle_time_days, avg_lead_time_days, backlog_size, completed_tasks, computed_at: Utc::now() }
    }
}

// =============================================================================
// §5 — SUB-PORTFOLIO ROLLUP MODEL
// =============================================================================

/// Rolled-up financial and status data for a portfolio or sub-portfolio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubPortfolioRollup {
    pub component_id:         ComponentId,
    pub total_budget_allocated: Decimal,
    pub total_budget_spent:     Decimal,
    pub total_revenue:          Decimal,
    pub total_expenses:         Decimal,
    pub avg_health_score:       f64,
    pub min_health_score:       f64,
    pub avg_progress_pct:       f64,
    pub active_count:           u32,
    pub completed_count:        u32,
    pub at_risk_count:          u32,
    pub computed_at:            DateTime<Utc>,
}

pub struct SubPortfolioRollupModel;

impl SubPortfolioRollupModel {
    pub fn compute(portfolio: &PortfolioRow, descendants: &[&PortfolioRow], cs: &CellStore) -> SubPortfolioRollup {
        let total_budget_allocated: Decimal = descendants.iter()
            .filter_map(|r| r.budget_allocated)
            .fold(Decimal::ZERO, |a, b| a + b);
        let total_budget_spent: Decimal = descendants.iter()
            .map(|r| r.budget_spent)
            .fold(Decimal::ZERO, |a, b| a + b);
        let total_revenue: Decimal = descendants.iter()
            .map(|r| r.revenue)
            .fold(Decimal::ZERO, |a, b| a + b);
        let total_expenses: Decimal = descendants.iter()
            .map(|r| r.expenses)
            .fold(Decimal::ZERO, |a, b| a + b);

        let health_scores: Vec<f64> = descendants.iter()
            .filter_map(|r| cs.get(&r.component_id, "health_score").and_then(|v| v.as_f64()))
            .collect();
        let avg_health_score = if health_scores.is_empty() { 0.0 } else {
            health_scores.iter().sum::<f64>() / health_scores.len() as f64
        };
        let min_health_score = health_scores.iter().cloned().fold(f64::INFINITY, f64::min);
        let avg_progress_pct = if descendants.is_empty() { 0.0 } else {
            descendants.iter().map(|r| r.progress_pct as f64).sum::<f64>() / descendants.len() as f64
        };

        let active_count    = descendants.iter().filter(|r| r.status == "Active").count() as u32;
        let completed_count = descendants.iter().filter(|r| r.status == "Completed").count() as u32;
        let at_risk_count   = health_scores.iter().filter(|&&s| s < 60.0).count() as u32;

        SubPortfolioRollup { component_id: portfolio.component_id, total_budget_allocated, total_budget_spent, total_revenue, total_expenses, avg_health_score, min_health_score, avg_progress_pct, active_count, completed_count, at_risk_count, computed_at: Utc::now() }
    }
}

// =============================================================================
// §6 — COLLABORATION SCORE MODEL
// =============================================================================

/// Collaboration health score for shared portfolio components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationScore {
    pub component_id:          ComponentId,
    pub collaboration_score:   f64,
    pub contributor_diversity: f64,
    pub contribution_velocity: f64,
    pub governance_participation: f64,
    pub conflict_penalty:      f64,
    pub computed_at:           DateTime<Utc>,
}

impl CollaborationScore {
    pub fn to_writeback(&self) -> EngineWriteback {
        EngineWriteback {
            component_id: self.component_id,
            column_id:    "collaboration_score".into(),
            value:        CellValue::Float(self.collaboration_score),
            computed_by:  "CollaborationEngine".into(),
            computed_at:  self.computed_at,
            ttl_secs:     Some(1800),
        }
    }
}

pub struct CollaborationScoreModel;

impl CollaborationScoreModel {
    pub fn compute(row: &PortfolioRow) -> CollaborationScore {
        // Diversity: normalized distinct contributor count (cap at 10 = full score).
        let contributor_diversity = (row.contributor_count.unwrap_or(0) as f64 / 10.0).min(1.0) * 100.0;

        // Velocity: 100 if contribution within 7 days, else decays.
        let days_since_contrib = row.last_contribution_date.map(|d| {
            (Utc::now() - d).num_days().abs() as f64
        }).unwrap_or(999.0);
        let contribution_velocity = (100.0 - days_since_contrib.min(100.0)).max(0.0);

        // Governance participation: 100 if no pending reviews, else decays.
        let pending = row.pending_review_count.unwrap_or(0) as f64;
        let governance_participation = (100.0 - pending * 10.0).clamp(0.0, 100.0);

        // Conflict penalty: subtract 5 per open merge conflict.
        let conflict_penalty = (row.merge_conflicts as f64 * 5.0).min(50.0);

        let raw = contributor_diversity * 0.35
                + contribution_velocity  * 0.35
                + governance_participation * 0.30;
        let collaboration_score = (raw - conflict_penalty).clamp(0.0, 100.0);

        CollaborationScore { component_id: row.component_id, collaboration_score, contributor_diversity, contribution_velocity, governance_participation, conflict_penalty, computed_at: Utc::now() }
    }
}

// =============================================================================
// §7 — BENEFITS HEALTH MODEL
// =============================================================================

/// Benefits coverage and income-replacement score for a worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenefitsHealth {
    pub owner_id:           EntityId,
    pub benefits_score:     f64,
    pub coverage_gaps:      Vec<String>,
    pub pto_coverage_days:  f64,
    pub retirement_vesting: f64,
    pub computed_at:        DateTime<Utc>,
}

pub struct BenefitsHealthModel;

impl BenefitsHealthModel {
    pub fn compute(owner_id: EntityId, benefit_rows: &[&PortfolioRow]) -> BenefitsHealth {
        let types_present: Vec<String> = benefit_rows.iter()
            .filter_map(|r| r.benefit_type.clone())
            .collect();

        let required = ["Health", "Retirement", "PTO", "EmergencySavings"];
        let coverage_gaps: Vec<String> = required.iter()
            .filter(|&&t| !types_present.iter().any(|p| p == t))
            .map(String::from)
            .collect();

        let gap_penalty = (coverage_gaps.len() as f64 * 20.0).min(80.0);
        let benefits_score = (100.0 - gap_penalty).clamp(0.0, 100.0);

        let pto_coverage_days = benefit_rows.iter()
            .filter(|r| r.benefit_type.as_deref() == Some("PTO"))
            .filter_map(|r| r.pto_remaining_days())
            .fold(Decimal::ZERO, |a, b| a + b)
            .to_string().parse::<f64>().unwrap_or(0.0);

        let retirement_vesting = benefit_rows.iter()
            .filter(|r| r.benefit_type.as_deref() == Some("Retirement"))
            .filter_map(|r| r.vesting_schedule.as_ref().map(|v| v.percent_vested))
            .fold(0.0_f64, f64::max);

        BenefitsHealth { owner_id, benefits_score, coverage_gaps, pto_coverage_days, retirement_vesting, computed_at: Utc::now() }
    }
}

// =============================================================================
// §8 — INCOME PROJECTION MODEL
// =============================================================================

/// 12-month forward income projection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomeProjection {
    pub owner_id:              EntityId,
    pub projected_annual:      f64,
    pub gig_component:         Option<ComponentId>,
    pub projection_confidence: f64,
    pub computed_at:           DateTime<Utc>,
}
