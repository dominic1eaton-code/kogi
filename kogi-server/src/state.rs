use kogi_host::executive::HostError;
use kogi_host::{
    HostApp, HostMessage, HostMessageResult, NewAffiliate, NewAffiliateLink, NewProvider,
    NewProviderDataAsset, NewProviderMetadata, NewProviderPlatform, NewProviderResource,
    NewProviderVersion,
};
use kogi_office_module::{
    to_json, NewAssistantSubscription, NewPortfolioItem, NewTimelineEvent, NewWorkspaceStory,
    OfficeModule,
};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::runtime::Runtime;

#[derive(Clone, Debug)]
pub struct WorkerIdentity {
    pub id: &'static str,
    pub display_name: &'static str,
    pub personas: &'static [&'static str],
    pub roles: &'static [&'static str],
    pub worker_types: &'static [&'static str],
    pub status: &'static str,
}

#[derive(Clone, Debug)]
pub struct IdentityProfile {
    pub id: &'static str,
    pub identity_id: &'static str,
    pub profile_type: &'static str,
    pub name: &'static str,
    pub accounts: &'static [&'static str],
    pub portfolios: &'static [&'static str],
    pub integrations: &'static [&'static str],
    pub tools: &'static [&'static str],
    pub projects: &'static [&'static str],
    pub programs: &'static [&'static str],
    pub settings_json: &'static str,
    pub options_json: &'static str,
    pub parameters_json: &'static str,
}

#[derive(Clone, Debug)]
pub struct ServerState {
    pub kernel_mode: &'static str,
    pub host: HostApp,
    pub gateway: GatewayClient,
    pub identities: Vec<WorkerIdentity>,
    pub profiles: Vec<IdentityProfile>,
    pub office_module: OfficeModule,
    pub runtime: Arc<Runtime>,
}

impl ServerState {
    pub fn bootstrap(runtime: Arc<Runtime>) -> Result<Self, HostError> {
        let mut host = HostApp::new()?;
        runtime.debug("host init");
        host.init()?;
        runtime.debug("host configure");
        host.configure()?;
        runtime.debug("host run");
        host.run()?;
        runtime.status("host", "booted");
        Ok(Self {
            kernel_mode: "user",
            host,
            gateway: GatewayClient::new("http://127.0.0.1:8090", runtime.clone()),
            identities: vec![WorkerIdentity {
                id: "ident-001",
                display_name: "Dominic Worker",
                personas: &["investor", "developer", "donor", "operator"],
                roles: &["owner", "admin", "contributor"],
                worker_types: &[
                    "freelancer",
                    "contractor",
                    "consultant",
                    "entrepreneur",
                    "full_time_worker",
                ],
                status: "active",
            }],
            profiles: vec![
                IdentityProfile {
                    id: "profile-personal-001",
                    identity_id: "ident-001",
                    profile_type: "personal",
                    name: "Personal Profile",
                    accounts: &["gmail", "wallet-personal"],
                    portfolios: &["portfolio-personal"],
                    integrations: &["instagram", "google-drive"],
                    tools: &["calendar", "notes", "task-list"],
                    projects: &["home-renovation"],
                    programs: &["personal-growth"],
                    settings_json: "{\"theme\":\"light\",\"timezone\":\"America/Chicago\"}",
                    options_json: "{\"notifications\":true,\"auto_sync\":true}",
                    parameters_json: "{\"focus_hours\":2}",
                },
                IdentityProfile {
                    id: "profile-work-001",
                    identity_id: "ident-001",
                    profile_type: "work",
                    name: "Work Profile",
                    accounts: &["upwork", "github", "wallet-operations"],
                    portfolios: &["portfolio-kogi", "portfolio-client-alpha"],
                    integrations: &["jira", "monday", "openai", "gitlab"],
                    tools: &["sprint-board", "dev-console", "api-sdk"],
                    projects: &["kogi-mvp", "integration-layer"],
                    programs: &["consulting-practice"],
                    settings_json: "{\"default_workspace\":\"work\",\"inbox_mode\":\"priority\"}",
                    options_json: "{\"time_tracking\":true,\"story_templates\":true}",
                    parameters_json: "{\"billing_rate\":115,\"sprint_days\":14}",
                },
                IdentityProfile {
                    id: "profile-business-001",
                    identity_id: "ident-001",
                    profile_type: "business",
                    name: "Business Profile",
                    accounts: &["stripe", "chase", "wallet-investment"],
                    portfolios: &["portfolio-fund", "portfolio-growth"],
                    integrations: &["quickbooks", "startupengine", "gofundme"],
                    tools: &["capital-planner", "risk-dashboard"],
                    projects: &["fundraise-seed"],
                    programs: &["revenue-ops"],
                    settings_json: "{\"fiscal_year_start\":\"2026-01-01\"}",
                    options_json: "{\"automated_distributions\":true}",
                    parameters_json: "{\"approval_threshold\":0.66}",
                },
                IdentityProfile {
                    id: "profile-community-001",
                    identity_id: "ident-001",
                    profile_type: "community",
                    name: "Community Profile",
                    accounts: &["discord", "youtube", "wallet-community"],
                    portfolios: &["portfolio-open-source"],
                    integrations: &["slack", "zoom", "youtube"],
                    tools: &["community-room", "polls", "events"],
                    projects: &["co-op-launch"],
                    programs: &["mutual-aid-network"],
                    settings_json: "{\"visibility\":\"public\",\"moderation\":\"team\"}",
                    options_json: "{\"allow_dm\":true,\"event_reminders\":true}",
                    parameters_json: "{\"weekly_events\":3,\"message_retention_days\":90}",
                },
            ],
            office_module: OfficeModule::mvp(),
            runtime,
        })
    }

    pub fn modules_json(&self) -> String {
        let modules = self.host.modules();
        let module_json = modules
            .iter()
            .map(|m| {
                format!(
                    "{{\"id\":\"{}\",\"name\":\"{}\",\"kind\":\"{}\",\"version\":\"{}\",\"state\":\"{}\",\"language\":\"{}\",\"entrypoint\":\"{}\",\"network_manager\":\"{}\",\"capabilities\":{},\"integrations\":{}}}",
                    escape_json(&m.id),
                    escape_json(&m.name),
                    escape_json(&m.kind),
                    escape_json(&m.version),
                    if m.active { "active" } else { "disabled" },
                    escape_json(&m.language),
                    escape_json(&m.entrypoint),
                    escape_json(&m.network_manager),
                    json_str_array_owned(&m.capabilities),
                    json_str_array_owned(&m.integrations),
                )
            })
            .collect::<Vec<_>>()
            .join(",");

        format!("{{\"modules\":[{module_json}]}}")
    }

    pub fn host_summary_json(&self) -> String {
        let provider_totals = self.host.provider_snapshot().totals;
        format!(
            "{{\"host_id\":\"kogi-host-001\",\"booted\":{},\"module_count\":{},\"component_count\":{},\"provider_count\":{},\"platform_count\":{},\"kernel_mode\":\"{}\",\"host_mode\":\"{}\",\"engine_service\":\"kogi-network/services/engine\",\"database_service\":\"kogi-network/services/database\"}}",
            self.host.booted(),
            self.host.module_count(),
            self.host.component_count(),
            provider_totals.providers,
            provider_totals.platforms,
            self.kernel_mode,
            self.host.mode_label()
        )
    }

    pub fn host_components_json(&self) -> String {
        let components = self.host.components();
        let component_json = components
            .iter()
            .map(|c| {
                format!(
                    "{{\"id\":\"{}\",\"group\":\"{}\",\"managed_by_kernel\":{},\"active\":{},\"endpoint\":\"{}\",\"network_manager\":\"{}\",\"limits\":{{\"memory_limit_mb\":{},\"max_processes\":{},\"max_files\":{},\"max_resources\":{}}}}}",
                    escape_json(&c.id),
                    c.group.as_str(),
                    c.managed_by_kernel,
                    c.active,
                    escape_json(&c.endpoint),
                    escape_json(&c.network_manager),
                    c.limits.memory_limit_mb,
                    c.limits.max_processes,
                    c.limits.max_files,
                    c.limits.max_resources,
                )
            })
            .collect::<Vec<_>>()
            .join(",");

        format!("{{\"components\":[{component_json}]}}")
    }

    pub fn summary_json(&self) -> String {
        let provider_totals = self.host.provider_snapshot().totals;
        format!(
            "{{\"kernel_mode\":\"{}\",\"host_booted\":{},\"host_mode\":\"{}\",\"module_count\":{},\"component_count\":{},\"provider_count\":{},\"platform_count\":{},\"identity_count\":{},\"profile_count\":{},\"office_views\":5,\"office_service\":\"kogi-network/services/office\",\"engine_service\":\"kogi-network/services/engine\",\"database_service\":\"kogi-network/services/database\",\"gateway\":\"http://127.0.0.1:8090\",\"data_flow\":\"clients->server->gateway->services/modules + server->host->kernel\"}}",
            self.kernel_mode,
            self.host.booted(),
            self.host.mode_label(),
            self.host.module_count(),
            self.host.component_count(),
            provider_totals.providers,
            provider_totals.platforms,
            self.identities.len(),
            self.profiles.len(),
        )
    }

    pub fn identities_json(&self) -> String {
        let identities_json = self
            .identities
            .iter()
            .map(|identity| {
                format!(
                    "{{\"id\":\"{}\",\"display_name\":\"{}\",\"personas\":{},\"roles\":{},\"worker_types\":{},\"status\":\"{}\"}}",
                    identity.id,
                    identity.display_name,
                    json_str_array(identity.personas),
                    json_str_array(identity.roles),
                    json_str_array(identity.worker_types),
                    identity.status,
                )
            })
            .collect::<Vec<_>>()
            .join(",");

        format!("{{\"identities\":[{identities_json}]}}")
    }

    pub fn profiles_json(&self) -> String {
        let profiles_json = self
            .profiles
            .iter()
            .map(|profile| {
                format!(
                    "{{\"id\":\"{}\",\"identity_id\":\"{}\",\"profile_type\":\"{}\",\"name\":\"{}\",\"accounts\":{},\"portfolios\":{},\"integrations\":{},\"tools\":{},\"projects\":{},\"programs\":{},\"settings\":{},\"options\":{},\"parameters\":{}}}",
                    profile.id,
                    profile.identity_id,
                    profile.profile_type,
                    profile.name,
                    json_str_array(profile.accounts),
                    json_str_array(profile.portfolios),
                    json_str_array(profile.integrations),
                    json_str_array(profile.tools),
                    json_str_array(profile.projects),
                    json_str_array(profile.programs),
                    profile.settings_json,
                    profile.options_json,
                    profile.parameters_json,
                )
            })
            .collect::<Vec<_>>()
            .join(",");

        format!("{{\"profiles\":[{profiles_json}]}}")
    }

    pub fn module_isolation_json(&self) -> String {
        let isolation = self.host.module_isolation_snapshot();
        let isolation_json = isolation
            .iter()
            .map(|module| {
                format!(
                    "{{\"module_id\":\"{}\",\"memory_limit_mb\":{},\"memory_used_mb\":{},\"process_limit\":{},\"process_count\":{},\"file_limit\":{},\"file_count\":{},\"resource_limit\":{},\"resource_used\":{},\"network_manager\":\"{}\"}}",
                    escape_json(&module.module_id),
                    module.memory_limit_mb,
                    module.memory_used_mb,
                    module.process_limit,
                    module.process_count,
                    module.file_limit,
                    module.file_count,
                    module.resource_limit,
                    module.resource_used,
                    escape_json(&module.network_manager),
                )
            })
            .collect::<Vec<_>>()
            .join(",");

        format!("{{\"modules\":[{isolation_json}]}}")
    }

    pub fn autonomy_capabilities_json(&self) -> String {
        "{\"abstractions\":[\"identity_management\",\"workspace_organization\",\"connection_registry\",\"contact_directory\",\"asset_vault\"],\"description\":\"System-level autonomy primitives for independent workers.\"}".to_string()
    }

    pub fn office_overview_json(&self) -> String {
        to_json(&self.office_module.overview())
    }

    pub fn office_dashboard_json(&self) -> String {
        to_json(&self.office_module.dashboard_snapshot())
    }

    pub fn office_portfolio_json(&self) -> String {
        to_json(&self.office_module.portfolio_snapshot())
    }

    pub fn office_timeline_json(&self) -> String {
        to_json(&self.office_module.timeline_snapshot())
    }

    pub fn office_workspace_json(&self) -> String {
        to_json(&self.office_module.workspace_snapshot())
    }

    pub fn office_assistant_json(&self) -> String {
        to_json(&self.office_module.assistant_snapshot())
    }

    pub fn providers_snapshot_json(&self) -> String {
        to_json(&self.host.provider_snapshot())
    }

    pub fn providers_platforms_json(&self) -> String {
        let snapshot = self.host.provider_snapshot();
        to_json(&serde_json::json!({"platforms": snapshot.platforms}))
    }

    pub fn providers_list_json(&self) -> String {
        let snapshot = self.host.provider_snapshot();
        to_json(&serde_json::json!({"providers": snapshot.providers}))
    }

    pub fn providers_resources_json(&self) -> String {
        let snapshot = self.host.provider_snapshot();
        to_json(&serde_json::json!({"resources": snapshot.resources}))
    }

    pub fn providers_versions_json(&self) -> String {
        let snapshot = self.host.provider_snapshot();
        to_json(&serde_json::json!({"versions": snapshot.versions}))
    }

    pub fn providers_metadata_json(&self) -> String {
        let snapshot = self.host.provider_snapshot();
        to_json(&serde_json::json!({"metadata": snapshot.metadata}))
    }

    pub fn providers_data_assets_json(&self) -> String {
        let snapshot = self.host.provider_snapshot();
        to_json(&serde_json::json!({"data_assets": snapshot.data_assets}))
    }

    pub fn providers_affiliates_json(&self) -> String {
        let snapshot = self.host.provider_snapshot();
        to_json(&serde_json::json!({"affiliates": snapshot.affiliates}))
    }

    pub fn providers_affiliate_links_json(&self) -> String {
        let snapshot = self.host.provider_snapshot();
        to_json(&serde_json::json!({"affiliate_links": snapshot.affiliate_links}))
    }

    pub fn providers_register_platform_json(&mut self, request: NewProviderPlatform) -> String {
        let platform = self.host.provider_system_mut().register_platform(request);
        to_json(&serde_json::json!({
            "ok": true,
            "platform": platform,
            "snapshot": self.host.provider_snapshot(),
        }))
    }

    pub fn providers_register_json(&mut self, request: NewProvider) -> String {
        match self.host.provider_system_mut().register_provider(request) {
            Ok(provider) => to_json(&serde_json::json!({
                "ok": true,
                "provider": provider,
                "snapshot": self.host.provider_snapshot(),
            })),
            Err(err) => to_json(&serde_json::json!({"ok": false, "error": err})),
        }
    }

    pub fn providers_add_resource_json(&mut self, request: NewProviderResource) -> String {
        match self.host.provider_system_mut().add_resource(request) {
            Ok(resource) => to_json(&serde_json::json!({
                "ok": true,
                "resource": resource,
                "snapshot": self.host.provider_snapshot(),
            })),
            Err(err) => to_json(&serde_json::json!({"ok": false, "error": err})),
        }
    }

    pub fn providers_add_version_json(&mut self, request: NewProviderVersion) -> String {
        match self.host.provider_system_mut().add_version(request) {
            Ok(version) => to_json(&serde_json::json!({
                "ok": true,
                "version": version,
                "snapshot": self.host.provider_snapshot(),
            })),
            Err(err) => to_json(&serde_json::json!({"ok": false, "error": err})),
        }
    }

    pub fn providers_set_metadata_json(&mut self, request: NewProviderMetadata) -> String {
        match self.host.provider_system_mut().set_metadata(request) {
            Ok(metadata) => to_json(&serde_json::json!({
                "ok": true,
                "metadata": metadata,
                "snapshot": self.host.provider_snapshot(),
            })),
            Err(err) => to_json(&serde_json::json!({"ok": false, "error": err})),
        }
    }

    pub fn providers_add_data_asset_json(&mut self, request: NewProviderDataAsset) -> String {
        match self.host.provider_system_mut().add_data_asset(request) {
            Ok(data_asset) => to_json(&serde_json::json!({
                "ok": true,
                "data_asset": data_asset,
                "snapshot": self.host.provider_snapshot(),
            })),
            Err(err) => to_json(&serde_json::json!({"ok": false, "error": err})),
        }
    }

    pub fn providers_register_affiliate_json(&mut self, request: NewAffiliate) -> String {
        let affiliate = self.host.provider_system_mut().register_affiliate(request);
        to_json(&serde_json::json!({
            "ok": true,
            "affiliate": affiliate,
            "snapshot": self.host.provider_snapshot(),
        }))
    }

    pub fn providers_add_affiliate_link_json(&mut self, request: NewAffiliateLink) -> String {
        match self.host.provider_system_mut().add_affiliate_link(request) {
            Ok(link) => to_json(&serde_json::json!({
                "ok": true,
                "affiliate_link": link,
                "snapshot": self.host.provider_snapshot(),
            })),
            Err(err) => to_json(&serde_json::json!({"ok": false, "error": err})),
        }
    }

    pub fn office_ack_notification_json(&mut self, notification_id: &str) -> String {
        to_json(&self.office_module.ack_dashboard_notification(notification_id))
    }

    pub fn office_create_portfolio_item_json(
        &mut self,
        item_type: &str,
        name: &str,
        status: &str,
    ) -> String {
        let request = NewPortfolioItem {
            item_type: item_type.to_string(),
            name: name.to_string(),
            status: status.to_string(),
        };
        to_json(&self.office_module.create_portfolio_item(request))
    }

    pub fn office_create_timeline_event_json(
        &mut self,
        calendar_id: &str,
        title: &str,
        kind: &str,
        scheduled_for: &str,
    ) -> String {
        let request = NewTimelineEvent {
            calendar_id: calendar_id.to_string(),
            title: title.to_string(),
            kind: kind.to_string(),
            scheduled_for: scheduled_for.to_string(),
        };
        to_json(&self.office_module.create_timeline_event(request))
    }

    pub fn office_create_workspace_story_json(&mut self, title: &str, points: i32) -> String {
        let request = NewWorkspaceStory {
            title: title.to_string(),
            points,
        };
        to_json(&self.office_module.create_workspace_story(request))
    }

    pub fn office_create_assistant_subscription_json(&mut self, topic: &str) -> String {
        let request = NewAssistantSubscription {
            topic: topic.to_string(),
        };
        to_json(&self.office_module.create_assistant_subscription(request))
    }

    pub fn engine_overview_json(&self) -> String {
        let component_ids = self
            .host
            .components()
            .into_iter()
            .map(|component| component.id)
            .collect::<Vec<_>>();
        let engine_service = match self.host.fetch_service_runtime("kogi.network.engine") {
            Ok(payload) => payload,
            Err(err) => format!(
                "{{\"status\":\"unreachable\",\"error\":\"{}\"}}",
                escape_json(&err.to_string())
            ),
        };

        format!(
            "{{\"engine\":\"kogi-engine\",\"status\":\"active\",\"ingest_topic\":\"engine.ingest\",\"flow\":\"clients->server->gateway->services/modules->engine + server->host->kernel\",\"components\":{},\"engine_service\":{engine_service},\"capabilities\":[\"analytics\",\"recommendations\",\"discover\",\"explore\",\"realtime snapshots\"]}}",
            json_str_array_owned(&component_ids),
        )
    }

    pub fn engine_service_runtime(&self) -> Result<String, HostError> {
        self.host.fetch_service_runtime("kogi.network.engine")
    }

    pub fn database_service_runtime(&self) -> Result<String, HostError> {
        self.host.fetch_service_runtime("kogi.network.database")
    }

    pub fn engine_control(&self, action: &str) -> Result<String, HostError> {
        let payload = format!("{{\"action\":\"{}\"}}", escape_json(action));
        self.runtime.message(
            "send",
            "engine.control.requested",
            "kogi.server",
            "kogi.network.engine",
            &payload,
        );
        let _ = self.publish_gateway_event(
            "engine.control.requested",
            &payload,
            "kogi.network.engine",
        );
        self.host.engine_control(action)
    }

    pub fn engine_ingest(&self, payload: &str) -> Result<String, HostError> {
        let body = if payload.trim_start().starts_with('{') {
            payload.to_string()
        } else {
            format!("{{\"payload\":\"{}\"}}", escape_json(payload))
        };
        self.runtime
            .message("send", "engine.ingest", "kogi.server", "kogi.engine", &body);
        let _ = self.publish_gateway_event("engine.ingest", &body, "kogi.engine");
        self.host.engine_ingest(payload)
    }

    pub fn database_query(&self, sql: &str) -> Result<String, HostError> {
        let payload = format!("{{\"sql\":\"{}\"}}", escape_json(sql));
        self.runtime.message(
            "send",
            "database.query.executed",
            "kogi.server",
            "kogi.network.database",
            &payload,
        );
        let _ = self.publish_gateway_event(
            "database.query.executed",
            &payload,
            "kogi.network.database",
        );
        self.host.database_query(sql)
    }

    pub fn relay_message_json(
        &mut self,
        topic: &str,
        payload: &str,
        source: &str,
        target: &str,
    ) -> String {
        self.runtime
            .message("receive", topic, source, target, payload);
        let message = HostMessage {
            topic: topic.to_string(),
            payload: payload.to_string(),
            source: source.to_string(),
            target: target.to_string(),
            received_at_ms: now_ms(),
        };
        let host_result = self.host.handle_message(message);
        self.runtime.debug(&format!(
            "host message status={} handled={} topic={}",
            host_result.status, host_result.handled, host_result.topic
        ));
        let gateway_result = match self.gateway.publish(topic, payload, source, target) {
            Ok(body) => body,
            Err(err) => format!(
                "{{\"status\":\"error\",\"error\":\"{}\"}}",
                escape_json(&err)
            ),
        };

        format!(
            "{{\"host\":{},\"gateway\":{}}}",
            host_message_result_json(&host_result),
            gateway_result
        )
    }

    pub fn gateway_history_json(&self, limit: usize, topic: Option<&str>) -> String {
        self.runtime.debug(&format!(
            "gateway history request limit={} topic={}",
            limit,
            topic.unwrap_or("")
        ));
        match self.gateway.history(limit, topic) {
            Ok(body) => body,
            Err(err) => format!(
                "{{\"status\":\"error\",\"error\":\"{}\"}}",
                escape_json(&err)
            ),
        }
    }

    fn publish_gateway_event(
        &self,
        topic: &str,
        payload: &str,
        target: &str,
    ) -> Result<String, String> {
        self.gateway
            .publish(topic, payload, "kogi.server", target)
    }

    pub fn unified_screens_json(&self) -> String {
        r#"{
  \"series\":\"kogi-unified-screen-system\",
  \"version\":\"v3-reconciled\",
  \"sources\":[
    \"Kogi_Screen_Flows_v2.pdf\",
    \"Kogi_Screen_Flows (2).pdf\",
    \"Kogi Platform - Screen Flows v2.pdf\",
    \"Kogi Platform - Screen Flows v3.pdf\"
  ],
  \"modules\":[
    {\"id\":\"dashboard\",\"title\":\"Dashboard\",\"sections\":[\"Portfolio Health\",\"Active Projects\",\"Net Revenue\",\"AI Credits\",\"Quick Access Modules\",\"Recent Activity\"],\"tags\":[\"overview\",\"activity\",\"ai\"]},
    {\"id\":\"office\",\"title\":\"Office\",\"sections\":[\"Programs and Projects\",\"Milestones Due\",\"Team Capacity\",\"Portfolio Assets\",\"3rd Party Integrations\"],\"tags\":[\"projects\",\"programs\",\"portfolio\"]},
    {\"id\":\"workspace\",\"title\":\"Workspace\",\"sections\":[\"Kanban Board\",\"Open Tasks\",\"Sprints Active\",\"Blocked Work\",\"Calendar\",\"Gantt Timeline\"],\"tags\":[\"tasks\",\"kanban\",\"sprints\"]},
    {\"id\":\"timeline\",\"title\":\"Timeline\",\"sections\":[\"Master Timeline\",\"Scheduled Events\",\"Milestones\",\"Roadmap Progress\",\"Deadlines\"],\"tags\":[\"calendar\",\"roadmap\",\"gantt\"]},
    {\"id\":\"portfolio\",\"title\":\"Portfolio\",\"sections\":[\"Portfolio Grid\",\"Projects\",\"Programs\",\"Assets\",\"Solutions\",\"Artifacts\",\"Linked Platforms\"],\"tags\":[\"assets\",\"solutions\",\"artifacts\"]},
    {\"id\":\"strategy\",\"title\":\"Strategy\",\"sections\":[\"Strategic OKRs\",\"Tactical Initiatives\",\"Operations Processes\",\"Governance\",\"KR Progress\"],\"tags\":[\"strategy\",\"tactics\",\"governance\"]},
    {\"id\":\"studio\",\"title\":\"Studio\",\"sections\":[\"Ideas and Concepts\",\"Prototypes\",\"Testing and Testbeds\",\"Toolsets and Toolkits\",\"Files and Notes\"],\"tags\":[\"ideas\",\"prototypes\",\"tools\"]},
    {\"id\":\"community\",\"title\":\"Community\",\"sections\":[\"Feeds and Timelines\",\"Spaces and Rooms\",\"Direct Messages\",\"Linked Platforms\"],\"tags\":[\"feeds\",\"spaces\",\"messages\"]},
    {\"id\":\"developer\",\"title\":\"Developer\",\"sections\":[\"API Reference\",\"Active Keys\",\"Webhooks\",\"Extensions and Integrations\",\"SDKs\"],\"tags\":[\"api\",\"sdk\",\"integrations\"]},
    {\"id\":\"profile\",\"title\":\"Profile\",\"sections\":[\"Personas and Roles\",\"Settings and Configuration\",\"Activity Stats\",\"Reputation\"],\"tags\":[\"personas\",\"settings\",\"config\"]},
    {\"id\":\"organizations\",\"title\":\"Organizations\",\"sections\":[\"Organizations Grid\",\"Governance and Proposals\",\"Roles\",\"Cap Tables\"],\"tags\":[\"coops\",\"collectives\",\"teams\"]},
    {\"id\":\"legal\",\"title\":\"Legal\",\"sections\":[\"IP and Trademarks\",\"Contracts and Agreements\",\"Compliance and Audit\",\"Upcoming Obligations\"],\"tags\":[\"ip\",\"contracts\",\"compliance\"]},
    {\"id\":\"marketplace\",\"title\":\"Marketplace\",\"sections\":[\"Marketplace Grid\",\"Buy/Sell/Barter\",\"Orders\",\"Linked Platforms\"],\"tags\":[\"buy\",\"sell\",\"barter\"]},
    {\"id\":\"bank\",\"title\":\"Bank\",\"sections\":[\"Wallet Types\",\"Finance Overview\",\"Fundraising and Capital\",\"Tax Summary\",\"Transactions\"],\"tags\":[\"wallets\",\"finance\",\"fundraising\"]},
    {\"id\":\"exchange\",\"title\":\"Exchange\",\"sections\":[\"Bids and Offers\",\"Deal Pipeline\",\"Requests\",\"Linked Platforms\"],\"tags\":[\"bids\",\"deals\",\"due-diligence\"]}
  ],
  \"workflows\":[
    {\"id\":\"asset-transfer\",\"title\":\"Asset Transfer\",\"module\":\"exchange\",\"steps\":[\"Select asset\",\"Create transfer terms\",\"Assign parties\",\"Set escrow controls\",\"Finalize settlement\"],\"tags\":[\"transfer\",\"escrow\"]},
    {\"id\":\"capital-exchange\",\"title\":\"Capital Exchange\",\"module\":\"bank\",\"steps\":[\"Open capital request\",\"Match contributors\",\"Apply governance checks\",\"Distribute capital\"],\"tags\":[\"capital\",\"governance\"]},
    {\"id\":\"community-showcase\",\"title\":\"Community Showcase\",\"module\":\"community\",\"steps\":[\"Create showcase post\",\"Attach artifacts\",\"Publish to spaces\",\"Track engagement\"],\"tags\":[\"community\",\"showcase\"]},
    {\"id\":\"coop-governance\",\"title\":\"Cooperative Governance\",\"module\":\"organizations\",\"steps\":[\"Draft proposal\",\"Open vote\",\"Reach quorum\",\"Record outcome\"],\"tags\":[\"cooperative\",\"voting\"]},
    {\"id\":\"idea-to-outcome\",\"title\":\"Idea to Outcome\",\"module\":\"studio\",\"steps\":[\"Capture idea\",\"Prototype\",\"Validate\",\"Promote to project\",\"Track outcome\"],\"tags\":[\"idea\",\"outcome\"]},
    {\"id\":\"idea-tracker\",\"title\":\"Idea Tracker\",\"module\":\"studio\",\"steps\":[\"Capture\",\"Score\",\"Prioritize\",\"Assign owner\"],\"tags\":[\"ideas\",\"tracker\"]},
    {\"id\":\"investor-outreach\",\"title\":\"Investor Outreach\",\"module\":\"bank\",\"steps\":[\"Build investor list\",\"Create pitch flow\",\"Schedule outreach\",\"Log responses\"],\"tags\":[\"investor\",\"outreach\"]},
    {\"id\":\"labor-market\",\"title\":\"Labor Market\",\"module\":\"marketplace\",\"steps\":[\"Publish need\",\"Match workers\",\"Negotiate terms\",\"Create engagement\"],\"tags\":[\"labor\",\"matching\"]},
    {\"id\":\"marketplace-exchange\",\"title\":\"Marketplace Exchange\",\"module\":\"marketplace\",\"steps\":[\"Create listing\",\"Receive offers\",\"Open deal\",\"Route to exchange settlement\"],\"tags\":[\"marketplace\",\"exchange\"]},
    {\"id\":\"note-creation\",\"title\":\"Note Creation\",\"module\":\"studio\",\"steps\":[\"Create note\",\"Tag context\",\"Link profile/project\",\"Share\"],\"tags\":[\"notes\",\"knowledge\"]},
    {\"id\":\"portfolio-governance\",\"title\":\"Portfolio Governance\",\"module\":\"portfolio\",\"steps\":[\"Review portfolio item\",\"Open governance check\",\"Approve/reject\",\"Log decision\"],\"tags\":[\"portfolio\",\"governance\"]},
    {\"id\":\"program-pipeline\",\"title\":\"Program Pipeline\",\"module\":\"office\",\"steps\":[\"Define program\",\"Create project lanes\",\"Track progress\",\"Report status\"],\"tags\":[\"program\",\"pipeline\"]},
    {\"id\":\"project-spotlight\",\"title\":\"Project Spotlight\",\"module\":\"office\",\"steps\":[\"Select project\",\"Assemble metrics\",\"Publish summary\"],\"tags\":[\"project\",\"spotlight\"]},
    {\"id\":\"project-workflow\",\"title\":\"Project Workflow\",\"module\":\"workspace\",\"steps\":[\"Backlog\",\"In Progress\",\"Review\",\"Done\"],\"tags\":[\"workflow\",\"kanban\"]},
    {\"id\":\"prototype-lifecycle\",\"title\":\"Prototype Lifecycle\",\"module\":\"studio\",\"steps\":[\"Prototype\",\"Test\",\"Iterate\",\"Release\"],\"tags\":[\"prototype\",\"lifecycle\"]},
    {\"id\":\"resource-exchange\",\"title\":\"Resource Exchange\",\"module\":\"exchange\",\"steps\":[\"Offer resource\",\"Request match\",\"Validate terms\",\"Exchange\"],\"tags\":[\"resource\",\"exchange\"]},
    {\"id\":\"resource-finder\",\"title\":\"Resource Finder\",\"module\":\"marketplace\",\"steps\":[\"Set criteria\",\"Search\",\"Compare\",\"Select\"],\"tags\":[\"resource\",\"discovery\"]},
    {\"id\":\"strategy-board\",\"title\":\"Strategy Board\",\"module\":\"strategy\",\"steps\":[\"Set objectives\",\"Map tactics\",\"Assign owners\",\"Track KRs\"],\"tags\":[\"strategy\",\"okr\"]},
    {\"id\":\"team-coordination\",\"title\":\"Team Coordination\",\"module\":\"office\",\"steps\":[\"Create team plan\",\"Assign roles\",\"Sync cadence\",\"Resolve blockers\"],\"tags\":[\"team\",\"coordination\"]},
    {\"id\":\"tool-builder\",\"title\":\"Tool Builder\",\"module\":\"developer\",\"steps\":[\"Define tool spec\",\"Build extension\",\"Test integration\",\"Publish\"],\"tags\":[\"tooling\",\"builder\"]},
    {\"id\":\"toolchain\",\"title\":\"Toolchain\",\"module\":\"developer\",\"steps\":[\"Select stack\",\"Configure pipeline\",\"Validate workflow\"],\"tags\":[\"toolchain\",\"pipeline\"]},
    {\"id\":\"tool-integration\",\"title\":\"Tool Integration\",\"module\":\"developer\",\"steps\":[\"Authorize provider\",\"Map data\",\"Set webhook\",\"Verify sync\"],\"tags\":[\"integration\",\"api\"]}
  ]
}"#
        .to_string()
    }

    pub fn unified_screens_flat(&self) -> String {
        "module|dashboard|Dashboard\nmodule|office|Office\nmodule|workspace|Workspace\nmodule|timeline|Timeline\nmodule|portfolio|Portfolio\nmodule|strategy|Strategy\nmodule|studio|Studio\nmodule|community|Community\nmodule|developer|Developer\nmodule|profile|Profile\nmodule|organizations|Organizations\nmodule|legal|Legal\nmodule|marketplace|Marketplace\nmodule|bank|Bank\nmodule|exchange|Exchange\nworkflow|asset-transfer|Asset Transfer\nworkflow|capital-exchange|Capital Exchange\nworkflow|community-showcase|Community Showcase\nworkflow|coop-governance|Cooperative Governance\nworkflow|idea-to-outcome|Idea to Outcome\nworkflow|idea-tracker|Idea Tracker\nworkflow|investor-outreach|Investor Outreach\nworkflow|labor-market|Labor Market\nworkflow|marketplace-exchange|Marketplace Exchange\nworkflow|note-creation|Note Creation\nworkflow|portfolio-governance|Portfolio Governance\nworkflow|program-pipeline|Program Pipeline\nworkflow|project-spotlight|Project Spotlight\nworkflow|project-workflow|Project Workflow\nworkflow|prototype-lifecycle|Prototype Lifecycle\nworkflow|resource-exchange|Resource Exchange\nworkflow|resource-finder|Resource Finder\nworkflow|strategy-board|Strategy Board\nworkflow|team-coordination|Team Coordination\nworkflow|tool-builder|Tool Builder\nworkflow|toolchain|Toolchain\nworkflow|tool-integration|Tool Integration\n"
            .to_string()
    }
}

fn json_str_array(values: &[&str]) -> String {
    let values = values
        .iter()
        .map(|v| format!("\"{}\"", v))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{}]", values)
}

fn json_str_array_owned(values: &[String]) -> String {
    let values = values
        .iter()
        .map(|v| format!("\"{}\"", escape_json(v)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{}]", values)
}

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_millis(0))
        .as_millis() as u64
}

fn host_message_result_json(result: &HostMessageResult) -> String {
    format!(
        "{{\"topic\":\"{}\",\"status\":\"{}\",\"handled\":{},\"processed_at_ms\":{},\"response\":\"{}\",\"error\":{}}}",
        escape_json(&result.topic),
        escape_json(&result.status),
        result.handled,
        result.processed_at_ms,
        escape_json(&result.response),
        match &result.error {
            Some(err) => format!("\"{}\"", escape_json(err)),
            None => "null".to_string(),
        }
    )
}

#[derive(Clone, Debug)]
pub struct GatewayClient {
    endpoint: String,
    runtime: Arc<Runtime>,
}

impl GatewayClient {
    pub fn new(endpoint: &str, runtime: Arc<Runtime>) -> Self {
        Self {
            endpoint: endpoint.trim_end_matches('/').to_string(),
            runtime,
        }
    }

    pub fn publish(
        &self,
        topic: &str,
        payload: &str,
        source: &str,
        target: &str,
    ) -> Result<String, String> {
        let endpoint = format!("{}/api/v1/gateway/pubsub/publish", self.endpoint);
        self.runtime
            .message("send", topic, source, target, payload);
        let body = format!(
            "{{\"topic\":\"{}\",\"payload\":\"{}\",\"source\":\"{}\",\"target\":\"{}\"}}",
            escape_json(topic),
            escape_json(payload),
            escape_json(source),
            escape_json(target)
        );
        let (status, response) = http_request("POST", &endpoint, Some(&body))?;
        self.runtime
            .debug(&format!("gateway publish status={status} endpoint={endpoint}"));
        if (200..300).contains(&status) {
            Ok(response)
        } else {
            Err(format!(
                "gateway publish failed status={status} endpoint={endpoint}"
            ))
        }
    }

    pub fn history(&self, limit: usize, topic: Option<&str>) -> Result<String, String> {
        let mut endpoint = format!("{}/api/v1/gateway/pubsub/history?limit={}", self.endpoint, limit);
        if let Some(topic) = topic {
            endpoint.push_str("&topic=");
            endpoint.push_str(&percent_encode(topic));
        }
        self.runtime
            .debug(&format!("gateway history endpoint={endpoint}"));
        let (status, response) = http_request("GET", &endpoint, None)?;
        self.runtime
            .debug(&format!("gateway history status={status} endpoint={endpoint}"));
        if (200..300).contains(&status) {
            Ok(response)
        } else {
            Err(format!(
                "gateway history failed status={status} endpoint={endpoint}"
            ))
        }
    }
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => encoded.push_str(&format!("%{:02X}", byte)),
        }
    }
    encoded
}

fn http_request(method: &str, endpoint: &str, body: Option<&str>) -> Result<(u16, String), String> {
    let stripped = endpoint
        .strip_prefix("http://")
        .ok_or_else(|| "only http endpoints are supported".to_string())?;

    let (host_port, path) = if let Some((host_port, path)) = stripped.split_once('/') {
        (host_port, format!("/{}", path))
    } else {
        (stripped, "/".to_string())
    };

    if host_port.is_empty() {
        return Err("invalid endpoint host".to_string());
    }

    let request_body = body.unwrap_or("");
    let mut stream = TcpStream::connect(host_port)
        .map_err(|err| format!("connect failed: {err}"))?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));

    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n{body}",
        method = method,
        path = path,
        host = host_port,
        len = request_body.as_bytes().len(),
        body = request_body
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|err| format!("request write failed: {err}"))?;

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|err| format!("response read failed: {err}"))?;

    let status_line = response.lines().next().unwrap_or("HTTP/1.1 000 UNKNOWN");
    let status = status_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("000")
        .parse::<u16>()
        .unwrap_or(0);
    let body = response
        .split("\r\n\r\n")
        .nth(1)
        .unwrap_or("")
        .to_string();
    Ok((status, body))
}
