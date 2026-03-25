use serde::{Deserialize, Serialize};
use uuid::Uuid;

use hypergrid::cell::DimKey;
use hypergrid::domain::{ComponentCategory, PortfolioComponent, ComponentStatus, ComponentState, Visibility, ComponentStore};

use crate::config::KogiPortfolioConfig;
use crate::error::{KogiPortfolioError, KogiPortfolioResult};
use crate::runtime::{KogiPortfolioRuntime, LinkForestView, LinkForestSheet, portfolio_row_from_component};
use crate::spreadsheet::{ComputationEngine, PortfolioRow, ScorePreference};
use crate::ui::{PortfolioItemsView, PortfolioDashboardSnapshot, PortfolioAnalyticsSnapshot, PortfolioRegistrySnapshot};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioSystemState {
    pub root_component_id: Uuid,
    pub workbook_id: Uuid,
    pub score_preference: ScorePreference,
}

/// PortfolioSystem is the master application execution model for the
/// kogi-portfolio service. It owns the runtime, spreadsheet engine, and
/// high-level orchestration helpers used by API layers.
pub struct PortfolioSystem {
    pub config: KogiPortfolioConfig,
    pub runtime: KogiPortfolioRuntime,
    pub engine: ComputationEngine,
}

impl PortfolioSystem {
    pub fn new(config: KogiPortfolioConfig) -> KogiPortfolioResult<Self> {
        let runtime = KogiPortfolioRuntime::new(config.clone())?;
        let engine = ComputationEngine::default_with_preference(config.score_preference);
        Ok(Self { config, runtime, engine })
    }

    pub fn state(&self) -> PortfolioSystemState {
        PortfolioSystemState {
            root_component_id: self.runtime.master.root_component_id,
            workbook_id: self.runtime.master.workbook_id,
            score_preference: self.runtime.master.workbook.score_preference,
        }
    }

    pub fn root_component_id(&self) -> Uuid {
        self.runtime.master.root_component_id
    }

    pub fn items_view(&self) -> PortfolioItemsView {
        self.runtime.items_view()
    }

    pub fn dashboard_snapshot(&self) -> PortfolioDashboardSnapshot {
        self.runtime.dashboard_snapshot()
    }

    pub fn analytics_snapshot(&self) -> PortfolioAnalyticsSnapshot {
        self.runtime.analytics_snapshot()
    }

    pub fn registry_snapshot(&self) -> PortfolioRegistrySnapshot {
        self.runtime.registry_snapshot()
    }

    pub fn link_forest_view(&self, root_component_id: Option<Uuid>) -> LinkForestView {
        let root = root_component_id.unwrap_or(self.runtime.master.root_component_id);
        self.runtime.link_forest_view(root)
    }

    pub fn link_forest_sheet(&self, root_component_id: Option<Uuid>) -> LinkForestSheet {
        let root = root_component_id.unwrap_or(self.runtime.master.root_component_id);
        self.runtime.link_forest_sheet(root)
    }

    pub fn link_forest_rows(&self, root_component_id: Option<Uuid>) -> Vec<PortfolioRow> {
        let root = root_component_id.unwrap_or(self.runtime.master.root_component_id);
        self.runtime.link_forest_rows(root)
    }

    pub fn recompute(&mut self) {
        self.engine.score_preference = self.config.score_preference;
        self.runtime.master.workbook.score_preference = self.config.score_preference;
        self.runtime.master.workbook.write_computed_columns(&self.engine, "kogi-portfolio");
    }

    pub fn set_score_preference(&mut self, preference: ScorePreference) {
        self.config.score_preference = preference;
        self.engine.score_preference = preference;
        self.runtime.master.workbook.score_preference = preference;
    }

    pub fn create_component(
        &mut self,
        name: impl Into<String>,
        category: ComponentCategory,
        payload: Option<serde_json::Value>,
    ) -> KogiPortfolioResult<PortfolioComponent> {
        let mut component = PortfolioComponent::new(self.config.owner_id, name, category);
        component.payload = payload;
        component.status = ComponentStatus::Active;
        component.state = ComponentState::Running;
        component.visibility = Visibility::Protected;

        ComponentStore::write(
            &self.runtime.apapo.grid,
            self.runtime.cubes.components_cube,
            &component,
            &self.config.node_id,
        )?;
        self.runtime.apapo.grid.hypergraph.register_entity(
            self.runtime.cubes.components_cube,
            DimKey::uuid(component.metadata.id),
            self.runtime.apapo.grid.grid_id,
        );

        let row = portfolio_row_from_component(&component);
        self.runtime.master.workbook.upsert_row(row);
        self.recompute();
        Ok(component)
    }

    pub fn upsert_component(&mut self, component: PortfolioComponent) -> KogiPortfolioResult<()> {
        ComponentStore::write(
            &self.runtime.apapo.grid,
            self.runtime.cubes.components_cube,
            &component,
            &self.config.node_id,
        )?;
        let row = portfolio_row_from_component(&component);
        self.runtime.master.workbook.upsert_row(row);
        self.recompute();
        Ok(())
    }

    pub fn get_component_row(&self, component_id: &Uuid) -> Option<PortfolioRow> {
        self.runtime.master.workbook.get_row(component_id).cloned()
    }

    pub fn sync_from_grid(&mut self) -> KogiPortfolioResult<()> {
        self.runtime.master.sync_from_grid(&self.runtime.apapo.grid, self.runtime.cubes.components_cube)?;
        self.recompute();
        Ok(())
    }

    pub fn persist(&self) -> KogiPortfolioResult<()> {
        self.runtime.master.persist_metadata(
            &self.runtime.apapo.grid,
            &self.runtime.cubes.spreadsheet,
            &self.config.node_id,
        )?;
        Ok(())
    }
}

impl From<KogiPortfolioError> for std::io::Error {
    fn from(err: KogiPortfolioError) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, err.to_string())
    }
}
