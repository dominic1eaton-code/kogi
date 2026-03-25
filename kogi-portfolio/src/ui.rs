use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use crate::spreadsheet::{
    SpreadsheetWorkbook, PortfolioRow, ColumnGroup, ColumnType,
    compute_health_score_with_pref, compute_risk_score_with_pref, compute_resource_utilization_pct_with_pref,
    compute_health_rollup_with_pref, compute_resource_utilization_rollup_pct_with_pref,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioCard {
    pub component_id: String,
    pub name: String,
    pub kind: String,
    pub status: String,
    pub summary: String,
    pub metric_label: String,
    pub metric_value: String,
    pub secondary_label: String,
    pub secondary_value: String,
    pub progress: f64,
    pub tone: String,
    pub tags: Vec<String>,
    pub owners: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardItem {
    pub component_id: String,
    pub name: String,
    pub meta: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardColumn {
    pub title: String,
    pub tone: String,
    pub items: Vec<BoardItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeItem {
    pub component_id: String,
    pub name: String,
    pub kind: String,
    pub meta: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeGroup {
    pub title: String,
    pub items: Vec<TreeItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioItemsView {
    pub items: Vec<PortfolioCard>,
    pub containers: Vec<PortfolioCard>,
    pub board_columns: Vec<BoardColumn>,
    pub container_board_columns: Vec<BoardColumn>,
    pub tree_groups: Vec<TreeGroup>,
    pub container_tree_groups: Vec<TreeGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryCard {
    pub label: String,
    pub value: String,
    pub delta: String,
    pub tone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetSummary {
    pub code: String,
    pub title: String,
    pub desc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSignal {
    pub label: String,
    pub value: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioDashboardSnapshot {
    pub summary_cards: Vec<SummaryCard>,
    pub sheet_registry: Vec<SheetSummary>,
    pub health_signals: Vec<HealthSignal>,
    pub saved_views: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelScore {
    pub label: String,
    pub value: String,
    pub trend: String,
    pub tone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiRow {
    pub name: String,
    pub value: String,
    pub target: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioAnalyticsSnapshot {
    pub model_scores: Vec<ModelScore>,
    pub kpi_rows: Vec<KpiRow>,
    pub insight_feed: Vec<String>,
    pub model_catalog: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnGroupSummary {
    pub title: String,
    pub desc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioRegistrySnapshot {
    pub column_groups: Vec<ColumnGroupSummary>,
    pub sheet_registry: Vec<SheetSummary>,
    pub view_engine: Vec<ColumnGroupSummary>,
    pub column_types: Vec<String>,
    pub access_control: Vec<ColumnGroupSummary>,
    pub saved_views: usize,
}

impl SpreadsheetWorkbook {
    pub fn items_view(&self) -> PortfolioItemsView {
        let rows: Vec<&PortfolioRow> = self.row_store.iter().collect();
        let (items, containers): (Vec<&PortfolioRow>, Vec<&PortfolioRow>) = rows
            .into_iter()
            .partition(|row| row.component_type.to_lowercase() != "container");

        let item_cards = items.iter().map(|row| make_card(self, row)).collect::<Vec<_>>();
        let container_cards = containers.iter().map(|row| make_card(self, row)).collect::<Vec<_>>();

        PortfolioItemsView {
            board_columns: make_board_columns(&item_cards),
            container_board_columns: make_board_columns(&container_cards),
            tree_groups: make_tree_groups(&item_cards),
            container_tree_groups: make_tree_groups(&container_cards),
            items: item_cards,
            containers: container_cards,
        }
    }

    pub fn dashboard_snapshot(&self) -> PortfolioDashboardSnapshot {
        let rows: Vec<&PortfolioRow> = self.row_store.iter().collect();
        let total = rows.len();
        let active = rows.iter().filter(|row| row.status.eq_ignore_ascii_case("Active")).count();
        let pending_reviews = rows.iter().filter(|row| {
            row.status.to_lowercase().contains("review")
                || row.approval_status.as_deref().unwrap_or("").to_lowercase().contains("pending")
        }).count();

        let (health_avg, _health_count) = average_score(rows.iter().map(|row| row_health(self, row)));
        let (risk_avg, _risk_count) = average_score(rows.iter().map(|row| row_risk(self, row)));

        let mut budget_alloc = Decimal::ZERO;
        let mut budget_spent = Decimal::ZERO;
        for row in &rows {
            if let Some(alloc) = row.budget_allocated {
                budget_alloc += alloc;
            }
            budget_spent += row.budget_spent;
        }
        let budget_util = pct_from_decimal(budget_spent, budget_alloc);

        let at_risk = rows.iter().filter(|row| row_risk(self, row).unwrap_or(0.0) >= 70.0).count();

        let summary_cards = vec![
            SummaryCard {
                label: "Total Components".to_string(),
                value: total.to_string(),
                delta: format!("{} active", active),
                tone: "text-[#60a5fa]".to_string(),
            },
            SummaryCard {
                label: "Active Items".to_string(),
                value: active.to_string(),
                delta: format!("{:.0}% active ratio", pct(active, total)),
                tone: tone_from_ratio(pct(active, total)),
            },
            SummaryCard {
                label: "Portfolio Health".to_string(),
                value: format!("{:.0}", health_avg.unwrap_or(0.0)),
                delta: "Target > 80".to_string(),
                tone: tone_from_ratio(health_avg.unwrap_or(0.0)),
            },
            SummaryCard {
                label: "Budget Utilized".to_string(),
                value: format!("${}", format_compact_decimal(budget_spent)),
                delta: format!("{:.0}% used", budget_util.unwrap_or(0.0)),
                tone: tone_from_ratio(100.0 - budget_util.unwrap_or(0.0)),
            },
            SummaryCard {
                label: "At Risk".to_string(),
                value: at_risk.to_string(),
                delta: format!("{:.0} avg risk", risk_avg.unwrap_or(0.0)),
                tone: tone_from_ratio(100.0 - risk_avg.unwrap_or(0.0)),
            },
            SummaryCard {
                label: "Pending Reviews".to_string(),
                value: pending_reviews.to_string(),
                delta: "Governance queue".to_string(),
                tone: "text-[#8b5cf6]".to_string(),
            },
        ];

        let sheet_registry = self.sheets.all().into_iter().map(|sheet| SheetSummary {
            code: sheet.id.clone(),
            title: sheet.name.clone(),
            desc: sheet.description.clone(),
        }).collect();

        let lifecycle_coverage = pct(
            rows.iter().filter(|row| !row.status.eq_ignore_ascii_case("Draft")).count(),
            total,
        );

        let governance = pct(
            rows.iter().filter(|row| !row.policy_ids.is_empty()).count(),
            total,
        );

        let utilization = average_score(rows.iter().map(|row| row_utilization(self, row))).0
            .unwrap_or(0.0);

        let program_alignment = average_score(
            rows.iter().filter(|row| row.item_type.as_deref() == Some("Program")).map(|row| Some(row.progress_pct)),
        ).0.unwrap_or(0.0);

        let health_signals = vec![
            HealthSignal {
                label: "Lifecycle Coverage".to_string(),
                value: format!("{:.0}%", lifecycle_coverage),
                detail: "Lifecycle completeness across items.".to_string(),
            },
            HealthSignal {
                label: "Governance Compliance".to_string(),
                value: format!("{:.0}%", governance),
                detail: "Policies attached and approvals tracked.".to_string(),
            },
            HealthSignal {
                label: "Resource Utilisation".to_string(),
                value: format!("{:.0}%", utilization),
                detail: "Capacity usage across resource pools.".to_string(),
            },
            HealthSignal {
                label: "Program Alignment".to_string(),
                value: format!("{:.0}", program_alignment),
                detail: "Alignment score across programs.".to_string(),
            },
        ];

        PortfolioDashboardSnapshot {
            summary_cards,
            sheet_registry,
            health_signals,
            saved_views: self.views.len(),
        }
    }

    pub fn analytics_snapshot(&self) -> PortfolioAnalyticsSnapshot {
        let rows: Vec<&PortfolioRow> = self.row_store.iter().collect();
        let total = rows.len();

        let health_avg = average_score(rows.iter().map(|row| row_health(self, row))).0.unwrap_or(0.0);
        let program_alignment = average_score(
            rows.iter().filter(|row| row.item_type.as_deref() == Some("Program")).map(|row| Some(row.progress_pct)),
        ).0.unwrap_or(0.0);
        let subportfolio_rollup = average_score(
            rows.iter().filter(|row| row.item_type.as_deref() == Some("SubPortfolio")).map(|row| {
                Some(compute_health_rollup_with_pref(
                    row,
                    &self.cell_store,
                    &self.row_store,
                    self.score_preference,
                ))
            }),
        ).0.unwrap_or(0.0);
        let resource_util = average_score(
            rows.iter().filter(|row| row.item_type.as_deref() == Some("Resource")).map(|row| row_utilization(self, row)),
        ).0.unwrap_or(0.0);

        let asset_value: Decimal = rows.iter()
            .filter(|row| row.item_type.as_deref() == Some("Asset"))
            .filter_map(|row| row.budget_allocated)
            .fold(Decimal::ZERO, |acc, v| acc + v);

        let binder_rows: Vec<&PortfolioRow> = rows.iter()
            .filter(|row| row.container_type.as_deref() == Some("Binder"))
            .copied()
            .collect();
        let binder_covered = binder_rows.iter().filter(|row| !row.child_ids.is_empty()).count();
        let binder_coverage = pct(binder_covered, binder_rows.len());

        let model_scores = vec![
            ModelScore {
                label: "Portfolio Health".to_string(),
                value: format!("{:.0}", health_avg),
                trend: "0".to_string(),
                tone: tone_from_ratio(health_avg),
            },
            ModelScore {
                label: "Program Alignment".to_string(),
                value: format!("{:.0}", program_alignment),
                trend: "0".to_string(),
                tone: tone_from_ratio(program_alignment),
            },
            ModelScore {
                label: "Subportfolio Rollup".to_string(),
                value: format!("{:.0}", subportfolio_rollup),
                trend: "0".to_string(),
                tone: tone_from_ratio(subportfolio_rollup),
            },
            ModelScore {
                label: "Resource Utilisation".to_string(),
                value: format!("{:.0}%", resource_util),
                trend: "0".to_string(),
                tone: tone_from_ratio(100.0 - (resource_util - 50.0).abs()),
            },
            ModelScore {
                label: "Asset Value".to_string(),
                value: format!("${}", format_compact_decimal(asset_value)),
                trend: "0".to_string(),
                tone: "text-[#22c55e]".to_string(),
            },
            ModelScore {
                label: "Binder Coverage".to_string(),
                value: format!("{:.0}%", binder_coverage),
                trend: "0".to_string(),
                tone: tone_from_ratio(binder_coverage),
            },
        ];

        let risk_mitigation = pct(
            rows.iter().filter(|row| row_risk(self, row).unwrap_or(0.0) < 70.0).count(),
            total,
        );
        let budget_efficiency = average_score(rows.iter().map(|row| row_budget_efficiency(row))).0.unwrap_or(0.0);

        let kpi_rows = vec![
            KpiRow {
                name: "Active Components".to_string(),
                value: format!("{}", rows.iter().filter(|row| row.status.eq_ignore_ascii_case("Active")).count()),
                target: format!("{}", total.max(1)),
                status: if pct(
                    rows.iter().filter(|row| row.status.eq_ignore_ascii_case("Active")).count(),
                    total,
                ) >= 70.0 { "On track".to_string() } else { "Needs review".to_string() },
            },
            KpiRow {
                name: "Budget Efficiency".to_string(),
                value: format!("{:.0}%", budget_efficiency),
                target: "70%".to_string(),
                status: if budget_efficiency >= 70.0 { "Stable".to_string() } else { "Needs review".to_string() },
            },
            KpiRow {
                name: "Risk Mitigation".to_string(),
                value: format!("{:.0}%", risk_mitigation),
                target: "85%".to_string(),
                status: if risk_mitigation >= 85.0 { "On track".to_string() } else { "Needs review".to_string() },
            },
            KpiRow {
                name: "Utilization Rollup".to_string(),
                value: format!("{:.0}%", average_score(rows.iter().map(|row| row_utilization_rollup(self, row))).0.unwrap_or(0.0)),
                target: "75%".to_string(),
                status: "Stable".to_string(),
            },
        ];

        let mut insight_feed = Vec::new();
        let at_risk = rows.iter().filter(|row| row_risk(self, row).unwrap_or(0.0) >= 70.0).count();
        if at_risk > 0 {
            insight_feed.push(format!("{} components are flagged as at risk.", at_risk));
        }
        let pending_reviews = rows.iter().filter(|row| row.status.to_lowercase().contains("review")).count();
        if pending_reviews > 0 {
            insight_feed.push(format!("{} components pending governance review.", pending_reviews));
        }
        if binder_rows.len() > 0 && binder_coverage < 80.0 {
            insight_feed.push("Binder coverage below 80% across shared binders.".to_string());
        }
        if resource_util > 85.0 {
            insight_feed.push("Resource utilization trending high across pools.".to_string());
        }
        if insight_feed.is_empty() {
            insight_feed.push("Portfolio metrics are stable with no critical alerts.".to_string());
        }

        PortfolioAnalyticsSnapshot {
            model_scores,
            kpi_rows,
            insight_feed,
            model_catalog: vec![
                "PortfolioHealth".to_string(),
                "ProjectMetrics".to_string(),
                "ProgramAlignment".to_string(),
                "SubPortfolioRollup".to_string(),
                "ResourceUtilisation".to_string(),
                "AssetValue".to_string(),
                "ArtifactMaturity".to_string(),
                "BinderCoverage".to_string(),
                "BookConsistency".to_string(),
            ],
        }
    }

    pub fn registry_snapshot(&self) -> PortfolioRegistrySnapshot {
        let sheet_registry = self.sheets.all().into_iter().map(|sheet| SheetSummary {
            code: sheet.id.clone(),
            title: sheet.name.clone(),
            desc: sheet.description.clone(),
        }).collect::<Vec<_>>();

        let mut group_map: Vec<ColumnGroupSummary> = Vec::new();
        if let Some(sheet) = self.sheets.get("SHT-001") {
            let mut seen = std::collections::HashSet::new();
            for column in &sheet.column_schema.columns {
                if seen.insert(format!("{:?}", column.group)) {
                    group_map.push(ColumnGroupSummary {
                        title: format!("{:?}", column.group),
                        desc: column_group_desc(&column.group),
                    });
                }
            }
        }

        let column_types = ColumnType::variants().into_iter().map(|s| s.to_string()).collect();

        PortfolioRegistrySnapshot {
            column_groups: group_map,
            sheet_registry,
            view_engine: vec![
                ColumnGroupSummary { title: "View Definitions".to_string(), desc: "Saved filters, groupings, layout presets.".to_string() },
                ColumnGroupSummary { title: "Filter System".to_string(), desc: "Column filters, smart predicates, PQL.".to_string() },
                ColumnGroupSummary { title: "Sort, Group & Pivot".to_string(), desc: "Multi-axis grouping and rollups.".to_string() },
                ColumnGroupSummary { title: "Board Modes".to_string(), desc: "Kanban, swimlanes, status columns.".to_string() },
            ],
            column_types,
            access_control: vec![
                ColumnGroupSummary { title: "Permission Tiers".to_string(), desc: "Owner - Admin - Editor - Commenter - Viewer".to_string() },
                ColumnGroupSummary { title: "Row-Level Controls".to_string(), desc: "Component-specific access policies.".to_string() },
                ColumnGroupSummary { title: "Column-Level Controls".to_string(), desc: "Sensitive columns gated by policy.".to_string() },
                ColumnGroupSummary { title: "Audit Logs".to_string(), desc: "Event log with CRDT merge history.".to_string() },
            ],
            saved_views: self.views.len(),
        }
    }
}

fn make_card(workbook: &SpreadsheetWorkbook, row: &PortfolioRow) -> PortfolioCard {
    let kind = row_kind(row);
    let (metric_label, metric_value, tone) = row_metric(workbook, row);
    let (secondary_label, secondary_value) = row_secondary(row);

    PortfolioCard {
        component_id: row.component_id.to_string(),
        name: row.name.clone(),
        kind,
        status: row.status.clone(),
        summary: row_summary(row),
        metric_label,
        metric_value,
        secondary_label,
        secondary_value,
        progress: row.progress_pct,
        tone,
        tags: row.tags.clone(),
        owners: row.owners.iter().map(|id| id.to_string()).collect(),
    }
}

fn make_board_columns(cards: &[PortfolioCard]) -> Vec<BoardColumn> {
    let mut columns: std::collections::HashMap<String, Vec<BoardItem>> = std::collections::HashMap::new();
    for card in cards {
        let title = normalize_status(&card.status);
        columns.entry(title).or_default().push(BoardItem {
            component_id: card.component_id.clone(),
            name: card.name.clone(),
            meta: format!("{} - {}", card.kind, card.metric_value),
        });
    }

    let order = ["Active", "Paused", "Draft", "Completed", "Archived", "Other"];
    order.iter().filter_map(|title| {
        columns.remove(*title).map(|items| BoardColumn {
            title: title.to_string(),
            tone: status_tone(title),
            items,
        })
    }).collect()
}

fn make_tree_groups(cards: &[PortfolioCard]) -> Vec<TreeGroup> {
    let mut groups: std::collections::HashMap<String, Vec<TreeItem>> = std::collections::HashMap::new();
    for card in cards {
        groups.entry(card.kind.clone()).or_default().push(TreeItem {
            component_id: card.component_id.clone(),
            name: card.name.clone(),
            kind: card.kind.clone(),
            meta: card.status.clone(),
        });
    }

    let mut entries: Vec<TreeGroup> = groups.into_iter().map(|(title, items)| TreeGroup { title, items }).collect();
    entries.sort_by(|a, b| a.title.cmp(&b.title));
    entries
}

fn row_kind(row: &PortfolioRow) -> String {
    row.item_type.clone()
        .or_else(|| row.container_type.clone())
        .unwrap_or_else(|| row.component_type.clone())
}

fn row_summary(row: &PortfolioRow) -> String {
    row.ext.get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{} component", row_kind(row)))
}

fn row_metric(workbook: &SpreadsheetWorkbook, row: &PortfolioRow) -> (String, String, String) {
    if let Some(health) = row_health(workbook, row) {
        return (
            "Health".to_string(),
            format!("{:.0}", health),
            tone_from_ratio(health),
        );
    }
    if let Some(risk) = row_risk(workbook, row) {
        return (
            "Risk".to_string(),
            format!("{:.0}", risk),
            tone_from_ratio(100.0 - risk),
        );
    }
    (
        "Progress".to_string(),
        format!("{:.0}%", row.progress_pct),
        tone_from_ratio(row.progress_pct),
    )
}

fn row_secondary(row: &PortfolioRow) -> (String, String) {
    if let Some(alloc) = row.budget_allocated {
        return (
            "Budget".to_string(),
            format!("${}", format_compact_decimal(alloc)),
        );
    }
    if !row.child_ids.is_empty() {
        return (
            "Items".to_string(),
            format!("{}", row.child_ids.len()),
        );
    }
    if let Some(total) = row.resource_units_total {
        return (
            "Capacity".to_string(),
            format!("{:.0}", total),
        );
    }
    ("".to_string(), "".to_string())
}

fn normalize_status(status: &str) -> String {
    let st = status.to_lowercase();
    if st.contains("active") { "Active".to_string() }
    else if st.contains("pause") { "Paused".to_string() }
    else if st.contains("complete") { "Completed".to_string() }
    else if st.contains("archive") { "Archived".to_string() }
    else if st.contains("draft") { "Draft".to_string() }
    else { "Other".to_string() }
}

fn status_tone(status: &str) -> String {
    match status {
        "Active" => "text-[#10b981]".to_string(),
        "Paused" => "text-[#f59e0b]".to_string(),
        "Draft" => "text-[#8ea6ad]".to_string(),
        "Completed" => "text-[#3b82f6]".to_string(),
        "Archived" => "text-[#94a3b8]".to_string(),
        _ => "text-[#8ea6ad]".to_string(),
    }
}

fn row_health(workbook: &SpreadsheetWorkbook, row: &PortfolioRow) -> Option<f64> {
    cell_f64(workbook, row, "health_score").or_else(|| Some(
        compute_health_score_with_pref(row, &workbook.cell_store, workbook.score_preference),
    ))
}

fn row_risk(workbook: &SpreadsheetWorkbook, row: &PortfolioRow) -> Option<f64> {
    cell_f64(workbook, row, "risk_score").or_else(|| Some(
        compute_risk_score_with_pref(row, &workbook.cell_store, workbook.score_preference),
    ))
}

fn row_utilization(workbook: &SpreadsheetWorkbook, row: &PortfolioRow) -> Option<f64> {
    cell_f64(workbook, row, "resource_utilization_pct")
        .or_else(|| compute_resource_utilization_pct_with_pref(
            row, &workbook.cell_store, workbook.score_preference,
        ))
}

fn row_utilization_rollup(workbook: &SpreadsheetWorkbook, row: &PortfolioRow) -> Option<f64> {
    cell_f64(workbook, row, "resource_utilization_rollup_pct")
        .or_else(|| compute_resource_utilization_rollup_pct_with_pref(
            row, &workbook.cell_store, &workbook.row_store, workbook.score_preference,
        ))
}

fn row_budget_efficiency(row: &PortfolioRow) -> Option<f64> {
    row.budget_allocated.and_then(|alloc| {
        let spent = row.budget_spent.to_f64()?;
        let alloc = alloc.to_f64()?;
        if alloc == 0.0 { Some(0.0) } else { Some((1.0 - (spent / alloc)).max(0.0) * 100.0) }
    })
}

fn cell_f64(workbook: &SpreadsheetWorkbook, row: &PortfolioRow, column_id: &str) -> Option<f64> {
    workbook.get_cell(&row.component_id, column_id)
        .and_then(|v| v.as_f64())
        .or_else(|| row.value_for(column_id).and_then(|v| v.as_f64()))
}

fn tone_from_ratio(value: f64) -> String {
    if value >= 80.0 { "text-[#10b981]".to_string() }
    else if value >= 60.0 { "text-[#f59e0b]".to_string() }
    else { "text-[#ef4444]".to_string() }
}

fn pct(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 { 0.0 } else { (numerator as f64 / denominator as f64) * 100.0 }
}

fn average_score<I>(scores: I) -> (Option<f64>, usize)
where
    I: Iterator<Item = Option<f64>>,
{
    let mut total = 0.0;
    let mut count = 0;
    for score in scores.flatten() {
        total += score;
        count += 1;
    }
    if count == 0 { (None, 0) } else { (Some(total / count as f64), count) }
}

fn pct_from_decimal(spent: Decimal, allocated: Decimal) -> Option<f64> {
    let spent = spent.to_f64()?;
    let alloc = allocated.to_f64()?;
    if alloc == 0.0 { Some(0.0) } else { Some((spent / alloc) * 100.0) }
}

fn format_compact_decimal(value: Decimal) -> String {
    let value = value.to_f64().unwrap_or(0.0);
    format_compact_number(value)
}

fn format_compact_number(value: f64) -> String {
    if value >= 1_000_000.0 {
        format!("{:.1}M", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("{:.0}k", value / 1_000.0)
    } else {
        format!("{:.0}", value)
    }
}

fn column_group_desc(group: &ColumnGroup) -> String {
    match group {
        ColumnGroup::Identity => "Name, type, owner, tags, IDs, classification.".to_string(),
        ColumnGroup::Lifecycle => "State, stage, approvals, version history.".to_string(),
        ColumnGroup::Ownership => "Owners, editors, watchers, contribution.".to_string(),
        ColumnGroup::Schedule => "Milestones, due dates, cadence.".to_string(),
        ColumnGroup::Finance => "Budget, spend, funding sources, ROI.".to_string(),
        ColumnGroup::Resource => "Capacity, utilization, staffing.".to_string(),
        ColumnGroup::Governance => "Permissions, votes, compliance.".to_string(),
        ColumnGroup::Analytics => "Health, KPIs, rollups, forecasts.".to_string(),
        ColumnGroup::Relationships => "Parent-child, dependencies, links.".to_string(),
        ColumnGroup::Collaboration => "Contributors, shared work.".to_string(),
        ColumnGroup::Benefits => "Portable benefits and coverage.".to_string(),
        ColumnGroup::Gig => "Gig-specific planning and execution.".to_string(),
        ColumnGroup::Investment => "Investments, returns, capital.".to_string(),
        ColumnGroup::Custom(label) => format!("Custom group: {label}"),
    }
}

impl ColumnType {
    fn variants() -> Vec<&'static str> {
        vec![
            "Text",
            "Number",
            "Decimal",
            "Currency",
            "Percent",
            "Date",
            "DateTime",
            "Bool",
            "Enum",
            "MultiEnum",
            "Relation",
            "MultiRelation",
            "User",
            "MultiUser",
            "Tags",
            "Json",
            "Computed",
            "AiSignal",
        ]
    }
}
