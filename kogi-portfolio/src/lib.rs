pub mod error;
pub mod config;
pub mod spreadsheet;
pub mod runtime;
pub mod ui;

pub use config::KogiPortfolioConfig;
pub use error::{KogiPortfolioError, KogiPortfolioResult};
pub use runtime::{KogiPortfolioRuntime, MasterPortfolioSpreadsheet, KogiPortfolioCubes};
pub use runtime::{LinkForestView, LinkEdgeView, LinkForestSheet};

pub use spreadsheet::{
    SpreadsheetWorkbook, PortfolioRow, SheetDefinition, SheetRegistry,
    ColumnSchema, ColumnDefinition, ColumnGroup, ColumnType,
    CellValue, ViewDefinition, ViewEngine, FilterPredicate, ViewSort, SortDirection,
    ComponentRegistry, ViewGroup, GroupBucket, PivotConfig, PivotTable, Aggregation,
    ComputationEngine, ComputedColumnKind, ComputedColumnSpec, SavedView, ViewRegistry,
};

pub use ui::{
    PortfolioItemsView, PortfolioDashboardSnapshot, PortfolioAnalyticsSnapshot,
    PortfolioRegistrySnapshot, PortfolioCard, BoardColumn, TreeGroup, SummaryCard,
    SheetSummary, HealthSignal, ModelScore, KpiRow, ColumnGroupSummary,
};

pub use hypergrid::domain::{
    PortfolioComponent, ComponentCategory, ItemCategory, ContainerCategory,
    ComponentStatus, ComponentState, Visibility, SearchQuery, PortfolioQuery, SearchResult,
};
