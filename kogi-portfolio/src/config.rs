use std::path::PathBuf;
use uuid::Uuid;

use apapo::{ApapoConfig, DomainKind, PersistenceMode};
use hypergrid::DomainSystem;

#[derive(Debug, Clone)]
pub struct KogiPortfolioConfig {
    pub grid_name: String,
    pub node_id: String,
    pub environment: String,
    pub data_dir: PathBuf,
    pub persistence: PersistenceMode,
    pub owner_id: Uuid,
    pub enabled_domains: Vec<DomainKind>,
}

impl Default for KogiPortfolioConfig {
    fn default() -> Self {
        Self {
            grid_name: "kogi-portfolio".to_owned(),
            node_id: "node-local:kogi".to_owned(),
            environment: "dev".to_owned(),
            data_dir: PathBuf::from("./data/kogi-portfolio"),
            persistence: PersistenceMode::Memory,
            owner_id: Uuid::new_v4(),
            enabled_domains: vec![DomainKind::Kogi],
        }
    }
}

impl KogiPortfolioConfig {
    pub fn apapo_config(&self) -> ApapoConfig {
        ApapoConfig {
            grid_name: self.grid_name.clone(),
            node_id: self.node_id.clone(),
            domain_system: DomainSystem::Apapo,
            enabled_domains: self.enabled_domains.clone(),
            environment: self.environment.clone(),
            data_dir: self.data_dir.clone(),
            persistence: self.persistence.clone(),
        }
    }
}
