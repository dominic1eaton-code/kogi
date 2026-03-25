use std::collections::HashMap;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use kogi_portfolio::{KogiPortfolioConfig, PortfolioRow, PortfolioSystem};

use crate::error::{MarketplaceError, MarketplaceResult};
use crate::model::*;

#[derive(Debug, Clone)]
pub struct MarketplaceConfig {
    pub portfolio_config: KogiPortfolioConfig,
    pub default_currency: String,
    pub default_pricing_model: PricingModel,
    pub default_visibility: ListingVisibility,
    pub marketplace_id: Uuid,
}

impl Default for MarketplaceConfig {
    fn default() -> Self {
        Self {
            portfolio_config: KogiPortfolioConfig::default(),
            default_currency: "USD".to_owned(),
            default_pricing_model: PricingModel::Negotiable,
            default_visibility: ListingVisibility::Public,
            marketplace_id: Uuid::new_v4(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceState {
    pub marketplace_id: Uuid,
    pub portfolio_root_component_id: Uuid,
    pub workbook_id: Uuid,
    pub listing_count: usize,
    pub active_listings: usize,
    pub last_refresh_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceInput {
    pub amount: Decimal,
    pub currency: Option<String>,
    pub pricing_model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishListingRequest {
    pub portfolio_component_id: String,
    pub listing_type: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub price: Option<PriceInput>,
    pub status: Option<String>,
    pub visibility: Option<String>,
    pub tags: Option<Vec<String>>,
    pub skills: Option<Vec<String>>,
    pub media: Option<Vec<String>>,
    pub metadata: Option<Value>,
}

pub struct MarketplaceSystem {
    pub config: MarketplaceConfig,
    pub portfolio: PortfolioSystem,
    manual_listings: HashMap<Uuid, Listing>,
    last_refresh_at: DateTime<Utc>,
}

impl MarketplaceSystem {
    pub fn new(config: MarketplaceConfig) -> MarketplaceResult<Self> {
        let portfolio = PortfolioSystem::new(config.portfolio_config.clone())?;
        Ok(Self {
            config,
            portfolio,
            manual_listings: HashMap::new(),
            last_refresh_at: Utc::now(),
        })
    }

    pub fn state(&self) -> MarketplaceState {
        let listings = self.listings();
        let active = listings.iter().filter(|l| matches!(l.status, ListingStatus::Active)).count();
        MarketplaceState {
            marketplace_id: self.config.marketplace_id,
            portfolio_root_component_id: self.portfolio.root_component_id(),
            workbook_id: self.portfolio.state().workbook_id,
            listing_count: listings.len(),
            active_listings: active,
            last_refresh_at: self.last_refresh_at,
        }
    }

    pub fn refresh(&mut self) -> MarketplaceResult<()> {
        self.portfolio.sync_from_grid()?;
        self.portfolio.persist()?;
        self.last_refresh_at = Utc::now();
        Ok(())
    }

    pub fn listings(&self) -> Vec<Listing> {
        let rows = self.portfolio.runtime.master.workbook.rows_for_sheet("SHT-023");
        let mut listing_map: HashMap<Uuid, Listing> = HashMap::new();

        for row in rows {
            if !is_marketplace_row(row) {
                continue;
            }
            let listing = listing_from_row(row, &self.config);
            listing_map.insert(listing.id, listing);
        }

        for (id, listing) in &self.manual_listings {
            listing_map.insert(*id, listing.clone());
        }

        let mut listings: Vec<Listing> = listing_map.into_values().collect();
        listings.sort_by_key(|l| l.updated_at);
        listings
    }

    pub fn listing_by_id(&self, listing_id: Uuid) -> Option<Listing> {
        if let Some(listing) = self.manual_listings.get(&listing_id) {
            return Some(listing.clone());
        }
        self.portfolio
            .runtime
            .master
            .workbook
            .get_row(&listing_id)
            .filter(|row| is_marketplace_row(row))
            .map(|row| listing_from_row(row, &self.config))
    }

    pub fn listings_for_owner(&self, owner_id: Uuid) -> Vec<Listing> {
        self.listings()
            .into_iter()
            .filter(|listing| listing.owner_id == owner_id)
            .collect()
    }

    pub fn publish_listing(&mut self, req: PublishListingRequest) -> MarketplaceResult<Listing> {
        let component_id = parse_uuid(&req.portfolio_component_id)?;
        let row = self
            .portfolio
            .runtime
            .master
            .workbook
            .get_row(&component_id)
            .ok_or_else(|| MarketplaceError::NotFound("portfolio component not found".to_string()))?;

        let mut listing = listing_from_row(row, &self.config);
        if let Some(listing_type) = req.listing_type {
            listing.listing_type = listing_type_from_str(&listing_type);
        }
        if let Some(title) = req.title {
            listing.title = title;
        }
        if let Some(description) = req.description {
            listing.description = description;
        }
        if let Some(price) = req.price {
            listing.price.amount = price.amount;
            if let Some(currency) = price.currency {
                listing.price.currency = currency;
            }
            if let Some(model) = price.pricing_model {
                listing.price.pricing_model = pricing_model_from_str(&model);
            }
        }
        if let Some(status) = req.status {
            listing.status = listing_status_from_str(&status);
        }
        if let Some(visibility) = req.visibility {
            listing.visibility = listing_visibility_from_str(&visibility);
        }
        if let Some(tags) = req.tags {
            listing.tags = tags;
        }
        if let Some(skills) = req.skills {
            listing.skills = skills;
        }
        if let Some(media) = req.media {
            listing.media = media;
        }
        if let Some(metadata) = req.metadata {
            listing.metadata = metadata;
        }

        self.manual_listings.insert(listing.id, listing.clone());
        Ok(listing)
    }
}

fn parse_uuid(value: &str) -> MarketplaceResult<Uuid> {
    Uuid::parse_str(value)
        .map_err(|_| MarketplaceError::Invalid(format!("invalid uuid: {value}")))
}

fn is_marketplace_row(row: &PortfolioRow) -> bool {
    let type_match = row
        .item_type
        .as_deref()
        .map(|t| matches_marketplace_type(t))
        .unwrap_or(false);
    let has_tag = row
        .tags
        .iter()
        .any(|t| t.eq_ignore_ascii_case("marketplace"))
        || row.topics.iter().any(|t| t.eq_ignore_ascii_case("marketplace"));
    let has_meta = row.ext.contains_key("marketplace") || row.ext.contains_key("marketplace_listing");
    type_match || has_tag || has_meta
}

fn matches_marketplace_type(raw: &str) -> bool {
    matches!(
        raw.to_lowercase().as_str(),
        "gig"
            | "contract"
            | "job"
            | "task"
            | "asset"
            | "artifact"
            | "resource"
            | "campaign"
            | "grant"
            | "investment"
            | "template"
            | "service"
            | "bundle"
    )
}

fn listing_from_row(row: &PortfolioRow, config: &MarketplaceConfig) -> Listing {
    let owner_id = row.owners.first().copied().unwrap_or(row.created_by);
    let title = row
        .display_name
        .clone()
        .unwrap_or_else(|| row.name.clone());
    let description = row
        .ext
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let mut listing = Listing {
        id: row.component_id,
        listing_type: listing_type_from_row(row),
        portfolio_component_id: Some(row.component_id),
        owner_id,
        title,
        description,
        price: price_from_row(row, config),
        status: listing_status_from_str(&row.status),
        visibility: listing_visibility_from_str(&row.visibility),
        tags: row.tags.clone(),
        skills: row.topics.clone(),
        media: list_from_value(row.ext.get("media")),
        review_ids: uuid_list_from_value(row.ext.get("review_ids")),
        rating: float_from_value(row.ext.get("rating")).unwrap_or(0.0),
        order_ids: uuid_list_from_value(row.ext.get("order_ids")),
        proposal_ids: uuid_list_from_value(row.ext.get("proposal_ids")),
        ai_score: float_from_value(row.ext.get("health_score")).unwrap_or(0.0),
        analytics: analytics_from_row(row),
        metadata: json!({ "portfolio_row": row }),
        created_at: row.created_at,
        updated_at: row.updated_at,
    };

    apply_marketplace_overrides(&mut listing, row);
    if matches!(listing.visibility, ListingVisibility::Public | ListingVisibility::Community | ListingVisibility::InviteOnly) {
        // keep listing visibility from row
    } else {
        listing.visibility = config.default_visibility.clone();
    }

    listing
}

fn listing_type_from_row(row: &PortfolioRow) -> ListingType {
    row.item_type
        .as_deref()
        .map(listing_type_from_str)
        .unwrap_or(ListingType::Custom)
}

fn listing_type_from_str(raw: &str) -> ListingType {
    match raw.to_lowercase().as_str() {
        "gig" | "contract" | "task" | "work" | "service" => ListingType::Work,
        "job" => ListingType::Job,
        "asset" => ListingType::Asset,
        "artifact" => ListingType::Artifact,
        "resource" => ListingType::Resource,
        "campaign" | "grant" | "investment" => ListingType::Campaign,
        "template" => ListingType::Template,
        "bundle" => ListingType::Bundle,
        _ => ListingType::Custom,
    }
}

fn listing_status_from_str(raw: &str) -> ListingStatus {
    match raw.to_lowercase().as_str() {
        "active" => ListingStatus::Active,
        "paused" => ListingStatus::Paused,
        "sold" => ListingStatus::Sold,
        "expired" => ListingStatus::Expired,
        "archived" => ListingStatus::Archived,
        _ => ListingStatus::Draft,
    }
}

fn listing_visibility_from_str(raw: &str) -> ListingVisibility {
    match raw.to_lowercase().as_str() {
        "public" => ListingVisibility::Public,
        "protected" | "community" => ListingVisibility::Community,
        "inviteonly" | "invite_only" | "invite-only" | "unlisted" => ListingVisibility::InviteOnly,
        _ => ListingVisibility::Public,
    }
}

fn pricing_model_from_str(raw: &str) -> PricingModel {
    match raw.to_lowercase().as_str() {
        "fixed" => PricingModel::Fixed,
        "hourly" => PricingModel::Hourly,
        "daily" => PricingModel::Daily,
        "subscription" => PricingModel::Subscription,
        "free" => PricingModel::Free,
        "auction" => PricingModel::Auction,
        "revshare" | "rev_share" | "revenue_share" | "revenue-share" => PricingModel::RevenueShare,
        _ => PricingModel::Negotiable,
    }
}

fn price_from_row(row: &PortfolioRow, config: &MarketplaceConfig) -> Price {
    let amount = row
        .rate
        .or(row.budget_allocated)
        .unwrap_or(Decimal::ZERO);
    let currency = row
        .currency
        .clone()
        .unwrap_or_else(|| config.default_currency.clone());
    let mut pricing_model = config.default_pricing_model.clone();
    if row.rate.is_some() {
        pricing_model = PricingModel::Hourly;
    }
    if let Some(model) = row.ext.get("pricing_model").and_then(|v| v.as_str()) {
        pricing_model = pricing_model_from_str(model);
    }
    Price { amount, currency, pricing_model }
}

fn analytics_from_row(row: &PortfolioRow) -> ListingAnalytics {
    let impressions = row.views.min(u32::MAX as u64) as u32;
    let clicks = row.clicks.min(u32::MAX as u64) as u32;
    let ctr = if impressions > 0 {
        clicks as f64 / impressions as f64
    } else {
        0.0
    };
    let engagement = if impressions > 0 {
        (row.shares + row.followers) as f64 / impressions as f64
    } else {
        0.0
    };
    ListingAnalytics {
        impressions,
        views: impressions,
        ctr,
        conversion_rate: 0.0,
        engagement,
        orders: 0,
        proposals: 0,
    }
}

fn apply_marketplace_overrides(listing: &mut Listing, row: &PortfolioRow) {
    let meta = row
        .ext
        .get("marketplace")
        .or_else(|| row.ext.get("marketplace_listing"));
    let Some(meta) = meta else { return; };
    let Some(obj) = meta.as_object() else { return; };

    if let Some(value) = obj.get("listing_type").and_then(|v| v.as_str()) {
        listing.listing_type = listing_type_from_str(value);
    }
    if let Some(value) = obj.get("title").and_then(|v| v.as_str()) {
        listing.title = value.to_string();
    }
    if let Some(value) = obj.get("description").and_then(|v| v.as_str()) {
        listing.description = value.to_string();
    }
    if let Some(value) = obj.get("status").and_then(|v| v.as_str()) {
        listing.status = listing_status_from_str(value);
    }
    if let Some(value) = obj.get("visibility").and_then(|v| v.as_str()) {
        listing.visibility = listing_visibility_from_str(value);
    }
    if let Some(value) = obj.get("tags") {
        listing.tags = list_from_value(Some(value));
    }
    if let Some(value) = obj.get("skills") {
        listing.skills = list_from_value(Some(value));
    }
    if let Some(value) = obj.get("media") {
        listing.media = list_from_value(Some(value));
    }
    if let Some(value) = obj.get("rating") {
        if let Some(rating) = float_from_value(Some(value)) {
            listing.rating = rating;
        }
    }
    if let Some(value) = obj.get("ai_score") {
        if let Some(score) = float_from_value(Some(value)) {
            listing.ai_score = score;
        }
    }
    if let Some(value) = obj.get("reviews") {
        listing.review_ids = uuid_list_from_value(Some(value));
    }
    if let Some(value) = obj.get("orders") {
        listing.order_ids = uuid_list_from_value(Some(value));
    }
    if let Some(value) = obj.get("proposals") {
        listing.proposal_ids = uuid_list_from_value(Some(value));
    }

    if let Some(price_val) = obj.get("price") {
        apply_price_override(listing, price_val);
    } else {
        if let Some(amount) = obj.get("amount").and_then(|v| decimal_from_value(v)) {
            listing.price.amount = amount;
        }
        if let Some(currency) = obj.get("currency").and_then(|v| v.as_str()) {
            listing.price.currency = currency.to_string();
        }
        if let Some(model) = obj.get("pricing_model").and_then(|v| v.as_str()) {
            listing.price.pricing_model = pricing_model_from_str(model);
        }
    }
}

fn apply_price_override(listing: &mut Listing, value: &Value) {
    if let Some(obj) = value.as_object() {
        if let Some(amount) = obj.get("amount").and_then(decimal_from_value) {
            listing.price.amount = amount;
        }
        if let Some(currency) = obj.get("currency").and_then(|v| v.as_str()) {
            listing.price.currency = currency.to_string();
        }
        if let Some(model) = obj
            .get("pricing_model")
            .or_else(|| obj.get("model"))
            .and_then(|v| v.as_str())
        {
            listing.price.pricing_model = pricing_model_from_str(model);
        }
        return;
    }
    if let Some(amount) = decimal_from_value(value) {
        listing.price.amount = amount;
    }
}

fn decimal_from_value(value: &Value) -> Option<Decimal> {
    match value {
        Value::Number(num) => num.as_f64().and_then(Decimal::from_f64),
        Value::String(s) => Decimal::from_str(s).ok(),
        _ => None,
    }
}

fn float_from_value(value: Option<&Value>) -> Option<f64> {
    let value = value?;
    match value {
        Value::Number(num) => num.as_f64(),
        Value::String(s) => s.parse::<f64>().ok(),
        _ => None,
    }
}

fn list_from_value(value: Option<&Value>) -> Vec<String> {
    let Some(value) = value else { return vec![]; };
    match value {
        Value::Array(items) => items
            .iter()
            .filter_map(|item| item.as_str().map(|s| s.to_string()))
            .collect(),
        Value::String(s) => s
            .split(',')
            .map(|item| item.trim())
            .filter(|item| !item.is_empty())
            .map(|item| item.to_string())
            .collect(),
        _ => vec![],
    }
}

fn uuid_list_from_value(value: Option<&Value>) -> Vec<Uuid> {
    let Some(value) = value else { return vec![]; };
    match value {
        Value::Array(items) => items
            .iter()
            .filter_map(|item| item.as_str().and_then(|s| Uuid::parse_str(s).ok()))
            .collect(),
        _ => vec![],
    }
}
