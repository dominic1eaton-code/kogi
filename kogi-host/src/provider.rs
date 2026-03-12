use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderPlatform {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub category: String,
    pub status: String,
    pub home_url: String,
    pub docs_url: String,
    pub support_contact: String,
    pub tags: Vec<String>,
    pub metadata: BTreeMap<String, String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderVersion {
    pub id: String,
    pub provider_id: String,
    pub version: String,
    pub status: String,
    pub released_at: String,
    pub notes: String,
    pub compatibility: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderResource {
    pub id: String,
    pub provider_id: String,
    pub resource_type: String,
    pub name: String,
    pub status: String,
    pub environment: String,
    pub endpoint: String,
    pub credentials_ref: String,
    pub last_checked: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderMetadataEntry {
    pub id: String,
    pub provider_id: String,
    pub key: String,
    pub value: String,
    pub scope: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderDataAsset {
    pub id: String,
    pub provider_id: String,
    pub dataset: String,
    pub status: String,
    pub record_count: u64,
    pub storage: String,
    pub last_sync: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AffiliateRecord {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub status: String,
    pub website: String,
    pub contact: String,
    pub tags: Vec<String>,
    pub metadata: BTreeMap<String, String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AffiliateLink {
    pub id: String,
    pub provider_id: String,
    pub affiliate_id: String,
    pub status: String,
    pub channel: String,
    pub tracking_url: String,
    pub contract_ref: String,
    pub started_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderRecord {
    pub id: String,
    pub name: String,
    pub platform_id: String,
    pub kind: String,
    pub status: String,
    pub owner: String,
    pub primary_contact: String,
    pub tags: Vec<String>,
    pub current_version: String,
    pub versions: Vec<ProviderVersion>,
    pub resources: Vec<ProviderResource>,
    pub metadata: Vec<ProviderMetadataEntry>,
    pub data_assets: Vec<ProviderDataAsset>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderTotals {
    pub platforms: usize,
    pub providers: usize,
    pub resources: usize,
    pub versions: usize,
    pub metadata_entries: usize,
    pub data_assets: usize,
    pub affiliates: usize,
    pub affiliate_links: usize,
    pub active_providers: usize,
    pub active_platforms: usize,
    pub active_affiliates: usize,
    pub active_affiliate_links: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderSnapshot {
    pub view: String,
    pub totals: ProviderTotals,
    pub platforms: Vec<ProviderPlatform>,
    pub providers: Vec<ProviderRecord>,
    pub resources: Vec<ProviderResource>,
    pub versions: Vec<ProviderVersion>,
    pub metadata: Vec<ProviderMetadataEntry>,
    pub data_assets: Vec<ProviderDataAsset>,
    pub affiliates: Vec<AffiliateRecord>,
    pub affiliate_links: Vec<AffiliateLink>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewProviderPlatform {
    pub name: String,
    pub kind: String,
    pub category: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub home_url: Option<String>,
    #[serde(default)]
    pub docs_url: Option<String>,
    #[serde(default)]
    pub support_contact: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewProvider {
    pub name: String,
    pub platform_id: String,
    pub kind: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub primary_contact: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewProviderResource {
    pub provider_id: String,
    pub resource_type: String,
    pub name: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub environment: Option<String>,
    #[serde(default)]
    pub endpoint: Option<String>,
    #[serde(default)]
    pub credentials_ref: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewProviderVersion {
    pub provider_id: String,
    pub version: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub released_at: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub compatibility: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewProviderMetadata {
    pub provider_id: String,
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub scope: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewProviderDataAsset {
    pub provider_id: String,
    pub dataset: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub record_count: Option<u64>,
    #[serde(default)]
    pub storage: Option<String>,
    #[serde(default)]
    pub last_sync: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewAffiliate {
    pub name: String,
    pub kind: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
    #[serde(default)]
    pub contact: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewAffiliateLink {
    pub provider_id: String,
    pub affiliate_id: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub channel: Option<String>,
    #[serde(default)]
    pub tracking_url: Option<String>,
    #[serde(default)]
    pub contract_ref: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ProviderSystem {
    platforms: Vec<ProviderPlatform>,
    providers: Vec<ProviderRecord>,
    affiliates: Vec<AffiliateRecord>,
    affiliate_links: Vec<AffiliateLink>,
    next_platform_id: u64,
    next_provider_id: u64,
    next_resource_id: u64,
    next_version_id: u64,
    next_metadata_id: u64,
    next_data_id: u64,
    next_affiliate_id: u64,
    next_affiliate_link_id: u64,
}

impl ProviderSystem {
    pub fn mvp() -> Self {
        let mut system = Self {
            platforms: Vec::new(),
            providers: Vec::new(),
            affiliates: Vec::new(),
            affiliate_links: Vec::new(),
            next_platform_id: 1,
            next_provider_id: 1,
            next_resource_id: 1,
            next_version_id: 1,
            next_metadata_id: 1,
            next_data_id: 1,
            next_affiliate_id: 1,
            next_affiliate_link_id: 1,
        };

        let platform = system.register_platform(NewProviderPlatform {
            name: "GitHub".to_string(),
            kind: "platform".to_string(),
            category: "developer".to_string(),
            status: Some("active".to_string()),
            home_url: Some("https://github.com".to_string()),
            docs_url: Some("https://docs.github.com".to_string()),
            support_contact: Some("support@github.com".to_string()),
            tags: vec!["code".to_string(), "repo".to_string(), "api".to_string()],
            metadata: BTreeMap::new(),
        });

        let provider = system
            .register_provider(NewProvider {
                name: "GitHub API".to_string(),
                platform_id: platform.id.clone(),
                kind: "api".to_string(),
                status: Some("active".to_string()),
                owner: Some("platform".to_string()),
                primary_contact: Some("devrel@github.com".to_string()),
                tags: vec!["oauth".to_string(), "webhooks".to_string()],
            })
            .expect("provider mvp");

        let _ = system.add_resource(NewProviderResource {
            provider_id: provider.id.clone(),
            resource_type: "webhook".to_string(),
            name: "repo-events".to_string(),
            status: Some("active".to_string()),
            environment: Some("production".to_string()),
            endpoint: Some("https://api.github.com/hooks".to_string()),
            credentials_ref: Some("vault/github/webhook".to_string()),
        });

        let _ = system.add_version(NewProviderVersion {
            provider_id: provider.id.clone(),
            version: "v3".to_string(),
            status: Some("stable".to_string()),
            released_at: Some(default_timestamp()),
            notes: Some("Primary REST API".to_string()),
            compatibility: vec!["oauth2".to_string(), "webhooks".to_string()],
        });

        let _ = system.set_metadata(NewProviderMetadata {
            provider_id: provider.id.clone(),
            key: "rate_limit".to_string(),
            value: "5000/hr".to_string(),
            scope: Some("api".to_string()),
        });

        let _ = system.add_data_asset(NewProviderDataAsset {
            provider_id: provider.id.clone(),
            dataset: "repo-sync".to_string(),
            status: Some("synced".to_string()),
            record_count: Some(320),
            storage: Some("kogi-data/providers/github".to_string()),
            last_sync: Some(default_timestamp()),
        });

        let affiliate = system.register_affiliate(NewAffiliate {
            name: "Kogi Partner Network".to_string(),
            kind: "partner".to_string(),
            status: Some("active".to_string()),
            website: Some("https://partners.kogi.local".to_string()),
            contact: Some("partners@kogi.local".to_string()),
            tags: vec!["affiliate".to_string(), "partner".to_string()],
            metadata: BTreeMap::new(),
        });

        let _ = system.add_affiliate_link(NewAffiliateLink {
            provider_id: provider.id.clone(),
            affiliate_id: affiliate.id.clone(),
            status: Some("active".to_string()),
            channel: Some("referral".to_string()),
            tracking_url: Some("https://kogi.local/track/github".to_string()),
            contract_ref: Some("contract-aff-001".to_string()),
        });

        system
    }

    pub fn snapshot(&self) -> ProviderSnapshot {
        let mut resources = Vec::new();
        let mut versions = Vec::new();
        let mut metadata = Vec::new();
        let mut data_assets = Vec::new();

        for provider in &self.providers {
            resources.extend(provider.resources.clone());
            versions.extend(provider.versions.clone());
            metadata.extend(provider.metadata.clone());
            data_assets.extend(provider.data_assets.clone());
        }

        ProviderSnapshot {
            view: "host.providers".to_string(),
            totals: ProviderTotals {
                platforms: self.platforms.len(),
                providers: self.providers.len(),
                resources: resources.len(),
                versions: versions.len(),
                metadata_entries: metadata.len(),
                data_assets: data_assets.len(),
                affiliates: self.affiliates.len(),
                affiliate_links: self.affiliate_links.len(),
                active_providers: self.providers.iter().filter(|p| p.status == "active").count(),
                active_platforms: self.platforms.iter().filter(|p| p.status == "active").count(),
                active_affiliates: self
                    .affiliates
                    .iter()
                    .filter(|a| a.status == "active")
                    .count(),
                active_affiliate_links: self
                    .affiliate_links
                    .iter()
                    .filter(|l| l.status == "active")
                    .count(),
            },
            platforms: self.platforms.clone(),
            providers: self.providers.clone(),
            resources,
            versions,
            metadata,
            data_assets,
            affiliates: self.affiliates.clone(),
            affiliate_links: self.affiliate_links.clone(),
        }
    }

    pub fn register_platform(&mut self, request: NewProviderPlatform) -> ProviderPlatform {
        let now = default_timestamp();
        let platform = ProviderPlatform {
            id: format!("platform-{:03}", self.next_platform_id),
            name: request.name,
            kind: request.kind,
            category: request.category,
            status: request.status.unwrap_or_else(|| "active".to_string()),
            home_url: request.home_url.unwrap_or_default(),
            docs_url: request.docs_url.unwrap_or_default(),
            support_contact: request.support_contact.unwrap_or_default(),
            tags: request.tags,
            metadata: request.metadata,
            created_at: now.clone(),
            updated_at: now,
        };
        self.next_platform_id += 1;
        self.platforms.push(platform.clone());
        platform
    }

    pub fn register_provider(&mut self, request: NewProvider) -> Result<ProviderRecord, String> {
        if !request.platform_id.is_empty()
            && !self.platforms.iter().any(|p| p.id == request.platform_id)
        {
            return Err(format!("platform not found: {}", request.platform_id));
        }
        let now = default_timestamp();
        let provider = ProviderRecord {
            id: format!("provider-{:03}", self.next_provider_id),
            name: request.name,
            platform_id: request.platform_id,
            kind: request.kind,
            status: request.status.unwrap_or_else(|| "active".to_string()),
            owner: request.owner.unwrap_or_else(|| "external".to_string()),
            primary_contact: request.primary_contact.unwrap_or_default(),
            tags: request.tags,
            current_version: "unversioned".to_string(),
            versions: Vec::new(),
            resources: Vec::new(),
            metadata: Vec::new(),
            data_assets: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
        };
        self.next_provider_id += 1;
        self.providers.push(provider.clone());
        Ok(provider)
    }

    pub fn add_resource(&mut self, request: NewProviderResource) -> Result<ProviderResource, String> {
        let provider = self
            .providers
            .iter_mut()
            .find(|p| p.id == request.provider_id)
            .ok_or_else(|| format!("provider not found: {}", request.provider_id))?;

        let resource = ProviderResource {
            id: format!("resource-{:03}", self.next_resource_id),
            provider_id: provider.id.clone(),
            resource_type: request.resource_type,
            name: request.name,
            status: request.status.unwrap_or_else(|| "active".to_string()),
            environment: request.environment.unwrap_or_else(|| "production".to_string()),
            endpoint: request.endpoint.unwrap_or_default(),
            credentials_ref: request.credentials_ref.unwrap_or_default(),
            last_checked: default_timestamp(),
        };
        self.next_resource_id += 1;
        provider.resources.push(resource.clone());
        provider.updated_at = default_timestamp();
        Ok(resource)
    }

    pub fn add_version(&mut self, request: NewProviderVersion) -> Result<ProviderVersion, String> {
        let provider = self
            .providers
            .iter_mut()
            .find(|p| p.id == request.provider_id)
            .ok_or_else(|| format!("provider not found: {}", request.provider_id))?;

        let version = ProviderVersion {
            id: format!("version-{:03}", self.next_version_id),
            provider_id: provider.id.clone(),
            version: request.version.clone(),
            status: request.status.unwrap_or_else(|| "stable".to_string()),
            released_at: request.released_at.unwrap_or_else(default_timestamp),
            notes: request.notes.unwrap_or_default(),
            compatibility: request.compatibility,
        };
        self.next_version_id += 1;
        provider.current_version = request.version;
        provider.versions.push(version.clone());
        provider.updated_at = default_timestamp();
        Ok(version)
    }

    pub fn set_metadata(
        &mut self,
        request: NewProviderMetadata,
    ) -> Result<ProviderMetadataEntry, String> {
        let provider = self
            .providers
            .iter_mut()
            .find(|p| p.id == request.provider_id)
            .ok_or_else(|| format!("provider not found: {}", request.provider_id))?;

        let scope = request.scope.unwrap_or_else(|| "general".to_string());
        if let Some(entry) = provider
            .metadata
            .iter_mut()
            .find(|m| m.key == request.key && m.scope == scope)
        {
            entry.value = request.value;
            entry.updated_at = default_timestamp();
            provider.updated_at = entry.updated_at.clone();
            return Ok(entry.clone());
        }

        let entry = ProviderMetadataEntry {
            id: format!("meta-{:03}", self.next_metadata_id),
            provider_id: provider.id.clone(),
            key: request.key,
            value: request.value,
            scope,
            updated_at: default_timestamp(),
        };
        self.next_metadata_id += 1;
        provider.metadata.push(entry.clone());
        provider.updated_at = default_timestamp();
        Ok(entry)
    }

    pub fn add_data_asset(
        &mut self,
        request: NewProviderDataAsset,
    ) -> Result<ProviderDataAsset, String> {
        let provider = self
            .providers
            .iter_mut()
            .find(|p| p.id == request.provider_id)
            .ok_or_else(|| format!("provider not found: {}", request.provider_id))?;

        let asset = ProviderDataAsset {
            id: format!("data-{:03}", self.next_data_id),
            provider_id: provider.id.clone(),
            dataset: request.dataset,
            status: request.status.unwrap_or_else(|| "active".to_string()),
            record_count: request.record_count.unwrap_or(0),
            storage: request.storage.unwrap_or_default(),
            last_sync: request.last_sync.unwrap_or_else(default_timestamp),
        };
        self.next_data_id += 1;
        provider.data_assets.push(asset.clone());
        provider.updated_at = default_timestamp();
        Ok(asset)
    }

    pub fn register_affiliate(&mut self, request: NewAffiliate) -> AffiliateRecord {
        let now = default_timestamp();
        let affiliate = AffiliateRecord {
            id: format!("affiliate-{:03}", self.next_affiliate_id),
            name: request.name,
            kind: request.kind,
            status: request.status.unwrap_or_else(|| "active".to_string()),
            website: request.website.unwrap_or_default(),
            contact: request.contact.unwrap_or_default(),
            tags: request.tags,
            metadata: request.metadata,
            created_at: now.clone(),
            updated_at: now,
        };
        self.next_affiliate_id += 1;
        self.affiliates.push(affiliate.clone());
        affiliate
    }

    pub fn add_affiliate_link(
        &mut self,
        request: NewAffiliateLink,
    ) -> Result<AffiliateLink, String> {
        if !self.providers.iter().any(|p| p.id == request.provider_id) {
            return Err(format!("provider not found: {}", request.provider_id));
        }
        if !self
            .affiliates
            .iter()
            .any(|a| a.id == request.affiliate_id)
        {
            return Err(format!("affiliate not found: {}", request.affiliate_id));
        }
        let now = default_timestamp();
        let link = AffiliateLink {
            id: format!("aff-link-{:03}", self.next_affiliate_link_id),
            provider_id: request.provider_id,
            affiliate_id: request.affiliate_id,
            status: request.status.unwrap_or_else(|| "active".to_string()),
            channel: request.channel.unwrap_or_else(|| "referral".to_string()),
            tracking_url: request.tracking_url.unwrap_or_default(),
            contract_ref: request.contract_ref.unwrap_or_default(),
            started_at: now.clone(),
            updated_at: now,
        };
        self.next_affiliate_link_id += 1;
        self.affiliate_links.push(link.clone());
        Ok(link)
    }

    pub fn update_provider_status(
        &mut self,
        provider_id: &str,
        status: &str,
    ) -> Result<ProviderRecord, String> {
        let provider = self
            .providers
            .iter_mut()
            .find(|p| p.id == provider_id)
            .ok_or_else(|| format!("provider not found: {}", provider_id))?;
        provider.status = status.to_string();
        provider.updated_at = default_timestamp();
        Ok(provider.clone())
    }

    pub fn update_platform_status(
        &mut self,
        platform_id: &str,
        status: &str,
    ) -> Result<ProviderPlatform, String> {
        let platform = self
            .platforms
            .iter_mut()
            .find(|p| p.id == platform_id)
            .ok_or_else(|| format!("platform not found: {}", platform_id))?;
        platform.status = status.to_string();
        platform.updated_at = default_timestamp();
        Ok(platform.clone())
    }
}

pub fn to_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value)
        .unwrap_or_else(|_| "{\"error\":\"serialization_failed\"}".to_string())
}

fn default_timestamp() -> String {
    "2026-03-12T00:00:00Z".to_string()
}
