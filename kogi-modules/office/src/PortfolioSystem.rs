use serde::{Deserialize, Serialize};

use crate::os_bridge::{
    parse_portfolio_kind, PortfolioEntityType, PortfolioItemContainerType, PortfolioItemType,
    ResourceType,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortfolioItem {
    pub id: String,
    pub entity_type: PortfolioEntityType,
    pub name: String,
    pub status: String,
    pub containers: Vec<PortfolioItemContainerType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortfolioSnapshot {
    pub view: String,
    pub items: Vec<PortfolioItem>,
    pub focus_item_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewPortfolioItem {
    pub item_type: String,
    pub name: String,
    pub status: String,
}

#[derive(Clone, Debug)]
pub struct PortfolioSystem {
    items: Vec<PortfolioItem>,
    focus_item_id: Option<String>,
    next_portfolio_id: u64,
}

impl PortfolioSystem {
    pub fn mvp() -> Self {
        Self {
            items: vec![PortfolioItem {
                id: "port-proj-001".to_string(),
                entity_type: PortfolioEntityType::Project,
                name: "Kogi Kernel Runtime".to_string(),
                status: "active".to_string(),
                containers: vec![
                    PortfolioItemContainerType::Binder,
                    PortfolioItemContainerType::Book,
                    PortfolioItemContainerType::Notebook,
                    PortfolioItemContainerType::Playbook,
                    PortfolioItemContainerType::Folder,
                    PortfolioItemContainerType::FileSet,
                    PortfolioItemContainerType::VersionControl,
                    PortfolioItemContainerType::Metadata,
                ],
            }],
            focus_item_id: Some("port-proj-001".to_string()),
            next_portfolio_id: 2,
        }
    }

    pub fn snapshot(&self) -> PortfolioSnapshot {
        PortfolioSnapshot {
            view: "portfolio".to_string(),
            items: self.items.clone(),
            focus_item_id: self.focus_item_id.clone(),
        }
    }

    pub fn add_item(&mut self, request: NewPortfolioItem) -> PortfolioItem {
        let id = format!("port-item-{:03}", self.next_portfolio_id);
        self.next_portfolio_id += 1;

        let item = PortfolioItem {
            id: id.clone(),
            entity_type: parse_entity_type(&request.item_type),
            name: request.name,
            status: request.status,
            containers: vec![
                PortfolioItemContainerType::Binder,
                PortfolioItemContainerType::Metadata,
            ],
        };

        self.focus_item_id = Some(id);
        self.items.push(item.clone());
        item
    }
}

fn parse_entity_type(item_type: &str) -> PortfolioEntityType {
    let (kind, resource_type) = parse_portfolio_kind(item_type);
    match (kind, resource_type) {
        (PortfolioItemType::Project, _) => PortfolioEntityType::Project,
        (PortfolioItemType::Program, _) => PortfolioEntityType::Program,
        (PortfolioItemType::SubPortfolio, _) => PortfolioEntityType::SubPortfolio,
        (PortfolioItemType::Resource, Some(ResourceType::Artifact)) => {
            PortfolioEntityType::Artifact
        }
        (PortfolioItemType::Resource, Some(ResourceType::Asset)) => PortfolioEntityType::Asset,
        (PortfolioItemType::Resource, Some(ResourceType::Capital)) => {
            PortfolioEntityType::Capital
        }
        (PortfolioItemType::Resource, Some(ResourceType::Investment)) => {
            PortfolioEntityType::Investment
        }
        (PortfolioItemType::Resource, Some(ResourceType::Account)) => {
            PortfolioEntityType::Account
        }
        (PortfolioItemType::Resource, Some(ResourceType::Land)) => PortfolioEntityType::Land,
        (PortfolioItemType::Resource, Some(ResourceType::Estate)) => {
            PortfolioEntityType::Estate
        }
        (PortfolioItemType::Resource, Some(ResourceType::Labor)) => PortfolioEntityType::Labor,
        (PortfolioItemType::Resource, None) => PortfolioEntityType::Resource,
    }
}
