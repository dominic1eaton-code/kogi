use kogi_office_module::{
    to_json, NewAssistantSubscription, NewPortfolioItem, NewTimelineEvent, NewWorkspaceStory,
    OfficeModule,
};

#[derive(Clone, Debug)]
pub struct ModuleStatus {
    pub id: &'static str,
    pub state: &'static str,
    pub language: &'static str,
}

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
pub struct ModuleIsolation {
    pub module_id: &'static str,
    pub memory_limit_mb: u64,
    pub memory_used_mb: u64,
    pub process_limit: u32,
    pub process_count: u32,
    pub file_limit: u32,
    pub file_count: u32,
    pub resource_limit: u32,
    pub resource_used: u32,
    pub network_manager: &'static str,
}

#[derive(Clone, Debug)]
pub struct ServerState {
    pub kernel_mode: &'static str,
    pub modules: Vec<ModuleStatus>,
    pub identities: Vec<WorkerIdentity>,
    pub profiles: Vec<IdentityProfile>,
    pub module_isolation: Vec<ModuleIsolation>,
    pub office_module: OfficeModule,
}

impl ServerState {
    pub fn mvp() -> Self {
        Self {
            kernel_mode: "user",
            modules: vec![
                ModuleStatus {
                    id: "kogi.office",
                    state: "active",
                    language: "hybrid-rust-go",
                },
                ModuleStatus {
                    id: "kogi.bank",
                    state: "active",
                    language: "go",
                },
                ModuleStatus {
                    id: "kogi.exchange",
                    state: "active",
                    language: "go",
                },
                ModuleStatus {
                    id: "kogi.community",
                    state: "active",
                    language: "go",
                },
            ],
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
                    projects: &["co-op-launch", "member-onboarding"],
                    programs: &["mutual-aid-network"],
                    settings_json: "{\"visibility\":\"public\",\"moderation\":\"team\"}",
                    options_json: "{\"allow_dm\":true,\"event_reminders\":true}",
                    parameters_json: "{\"weekly_events\":3,\"message_retention_days\":90}",
                },
            ],
            module_isolation: vec![
                ModuleIsolation {
                    module_id: "kogi.office",
                    memory_limit_mb: 768,
                    memory_used_mb: 142,
                    process_limit: 96,
                    process_count: 12,
                    file_limit: 6000,
                    file_count: 382,
                    resource_limit: 14000,
                    resource_used: 1602,
                    network_manager: "kogi-go-network",
                },
                ModuleIsolation {
                    module_id: "kogi.exchange",
                    memory_limit_mb: 768,
                    memory_used_mb: 210,
                    process_limit: 120,
                    process_count: 16,
                    file_limit: 8000,
                    file_count: 524,
                    resource_limit: 18000,
                    resource_used: 2500,
                    network_manager: "kogi-go-network",
                },
                ModuleIsolation {
                    module_id: "kogi.community",
                    memory_limit_mb: 512,
                    memory_used_mb: 98,
                    process_limit: 96,
                    process_count: 10,
                    file_limit: 6000,
                    file_count: 291,
                    resource_limit: 12000,
                    resource_used: 1310,
                    network_manager: "kogi-go-network",
                },
            ],
            office_module: OfficeModule::mvp(),
        }
    }

    pub fn modules_json(&self) -> String {
        let module_json = self
            .modules
            .iter()
            .map(|m| {
                format!(
                    "{{\"id\":\"{}\",\"state\":\"{}\",\"language\":\"{}\"}}",
                    m.id, m.state, m.language
                )
            })
            .collect::<Vec<_>>()
            .join(",");

        format!("{{\"modules\":[{module_json}]}}")
    }

    pub fn summary_json(&self) -> String {
        format!(
            "{{\"kernel_mode\":\"{}\",\"module_count\":{},\"identity_count\":{},\"profile_count\":{},\"office_views\":5,\"office_service\":\"kogi-services/go/services/office\"}}",
            self.kernel_mode,
            self.modules.len(),
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
        let isolation_json = self
            .module_isolation
            .iter()
            .map(|module| {
                format!(
                    "{{\"module_id\":\"{}\",\"memory_limit_mb\":{},\"memory_used_mb\":{},\"process_limit\":{},\"process_count\":{},\"file_limit\":{},\"file_count\":{},\"resource_limit\":{},\"resource_used\":{},\"network_manager\":\"{}\"}}",
                    module.module_id,
                    module.memory_limit_mb,
                    module.memory_used_mb,
                    module.process_limit,
                    module.process_count,
                    module.file_limit,
                    module.file_count,
                    module.resource_limit,
                    module.resource_used,
                    module.network_manager,
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

    pub fn unified_screens_json(&self) -> String {
        r#"{
  "series":"kogi-unified-screen-system",
  "version":"v3-reconciled",
  "sources":[
    "Kogi_Screen_Flows_v2.pdf",
    "Kogi_Screen_Flows (2).pdf",
    "Kogi Platform - Screen Flows v2.pdf",
    "Kogi Platform - Screen Flows v3.pdf"
  ],
  "modules":[
    {"id":"dashboard","title":"Dashboard","sections":["Portfolio Health","Active Projects","Net Revenue","AI Credits","Quick Access Modules","Recent Activity"],"tags":["overview","activity","ai"]},
    {"id":"office","title":"Office","sections":["Programs and Projects","Milestones Due","Team Capacity","Portfolio Assets","3rd Party Integrations"],"tags":["projects","programs","portfolio"]},
    {"id":"workspace","title":"Workspace","sections":["Kanban Board","Open Tasks","Sprints Active","Blocked Work","Calendar","Gantt Timeline"],"tags":["tasks","kanban","sprints"]},
    {"id":"timeline","title":"Timeline","sections":["Master Timeline","Scheduled Events","Milestones","Roadmap Progress","Deadlines"],"tags":["calendar","roadmap","gantt"]},
    {"id":"portfolio","title":"Portfolio","sections":["Portfolio Grid","Projects","Programs","Assets","Solutions","Artifacts","Linked Platforms"],"tags":["assets","solutions","artifacts"]},
    {"id":"strategy","title":"Strategy","sections":["Strategic OKRs","Tactical Initiatives","Operations Processes","Governance","KR Progress"],"tags":["strategy","tactics","governance"]},
    {"id":"studio","title":"Studio","sections":["Ideas and Concepts","Prototypes","Testing and Testbeds","Toolsets and Toolkits","Files and Notes"],"tags":["ideas","prototypes","tools"]},
    {"id":"community","title":"Community","sections":["Feeds and Timelines","Spaces and Rooms","Direct Messages","Linked Platforms"],"tags":["feeds","spaces","messages"]},
    {"id":"developer","title":"Developer","sections":["API Reference","Active Keys","Webhooks","Extensions and Integrations","SDKs"],"tags":["api","sdk","integrations"]},
    {"id":"profile","title":"Profile","sections":["Personas and Roles","Settings and Configuration","Activity Stats","Reputation"],"tags":["personas","settings","config"]},
    {"id":"organizations","title":"Organizations","sections":["Organizations Grid","Governance and Proposals","Roles","Cap Tables"],"tags":["coops","collectives","teams"]},
    {"id":"legal","title":"Legal","sections":["IP and Trademarks","Contracts and Agreements","Compliance and Audit","Upcoming Obligations"],"tags":["ip","contracts","compliance"]},
    {"id":"marketplace","title":"Marketplace","sections":["Marketplace Grid","Buy/Sell/Barter","Orders","Linked Platforms"],"tags":["buy","sell","barter"]},
    {"id":"bank","title":"Bank","sections":["Wallet Types","Finance Overview","Fundraising and Capital","Tax Summary","Transactions"],"tags":["wallets","finance","fundraising"]},
    {"id":"exchange","title":"Exchange","sections":["Bids and Offers","Deal Pipeline","Requests","Linked Platforms"],"tags":["bids","deals","due-diligence"]}
  ],
  "workflows":[
    {"id":"asset-transfer","title":"Asset Transfer","module":"exchange","steps":["Select asset","Create transfer terms","Assign parties","Set escrow controls","Finalize settlement"],"tags":["transfer","escrow"]},
    {"id":"capital-exchange","title":"Capital Exchange","module":"bank","steps":["Open capital request","Match contributors","Apply governance checks","Distribute capital"],"tags":["capital","governance"]},
    {"id":"community-showcase","title":"Community Showcase","module":"community","steps":["Create showcase post","Attach artifacts","Publish to spaces","Track engagement"],"tags":["community","showcase"]},
    {"id":"coop-governance","title":"Cooperative Governance","module":"organizations","steps":["Draft proposal","Open vote","Reach quorum","Record outcome"],"tags":["cooperative","voting"]},
    {"id":"idea-to-outcome","title":"Idea to Outcome","module":"studio","steps":["Capture idea","Prototype","Validate","Promote to project","Track outcome"],"tags":["idea","outcome"]},
    {"id":"idea-tracker","title":"Idea Tracker","module":"studio","steps":["Capture","Score","Prioritize","Assign owner"],"tags":["ideas","tracker"]},
    {"id":"investor-outreach","title":"Investor Outreach","module":"bank","steps":["Build investor list","Create pitch flow","Schedule outreach","Log responses"],"tags":["investor","outreach"]},
    {"id":"labor-market","title":"Labor Market","module":"marketplace","steps":["Publish need","Match workers","Negotiate terms","Create engagement"],"tags":["labor","matching"]},
    {"id":"marketplace-exchange","title":"Marketplace Exchange","module":"marketplace","steps":["Create listing","Receive offers","Open deal","Route to exchange settlement"],"tags":["marketplace","exchange"]},
    {"id":"note-creation","title":"Note Creation","module":"studio","steps":["Create note","Tag context","Link profile/project","Share"],"tags":["notes","knowledge"]},
    {"id":"portfolio-governance","title":"Portfolio Governance","module":"portfolio","steps":["Review portfolio item","Open governance check","Approve/reject","Log decision"],"tags":["portfolio","governance"]},
    {"id":"program-pipeline","title":"Program Pipeline","module":"office","steps":["Define program","Create project lanes","Track progress","Report status"],"tags":["program","pipeline"]},
    {"id":"project-spotlight","title":"Project Spotlight","module":"office","steps":["Select project","Assemble metrics","Publish summary"],"tags":["project","spotlight"]},
    {"id":"project-workflow","title":"Project Workflow","module":"workspace","steps":["Backlog","In Progress","Review","Done"],"tags":["workflow","kanban"]},
    {"id":"prototype-lifecycle","title":"Prototype Lifecycle","module":"studio","steps":["Prototype","Test","Iterate","Release"],"tags":["prototype","lifecycle"]},
    {"id":"resource-exchange","title":"Resource Exchange","module":"exchange","steps":["Offer resource","Request match","Validate terms","Exchange"],"tags":["resource","exchange"]},
    {"id":"resource-finder","title":"Resource Finder","module":"marketplace","steps":["Set criteria","Search","Compare","Select"],"tags":["resource","discovery"]},
    {"id":"strategy-board","title":"Strategy Board","module":"strategy","steps":["Set objectives","Map tactics","Assign owners","Track KRs"],"tags":["strategy","okr"]},
    {"id":"team-coordination","title":"Team Coordination","module":"office","steps":["Create team plan","Assign roles","Sync cadence","Resolve blockers"],"tags":["team","coordination"]},
    {"id":"tool-builder","title":"Tool Builder","module":"developer","steps":["Define tool spec","Build extension","Test integration","Publish"],"tags":["tooling","builder"]},
    {"id":"toolchain","title":"Toolchain","module":"developer","steps":["Select stack","Configure pipeline","Validate workflow"],"tags":["toolchain","pipeline"]},
    {"id":"tool-integration","title":"Tool Integration","module":"developer","steps":["Authorize provider","Map data","Set webhook","Verify sync"],"tags":["integration","api"]}
  ]
}"#
        .to_string()
    }

    pub fn unified_screens_flat(&self) -> String {
        "module|dashboard|Dashboard\nmodule|office|Office\nmodule|workspace|Workspace\nmodule|timeline|Timeline\nmodule|portfolio|Portfolio\nmodule|strategy|Strategy\nmodule|studio|Studio\nmodule|community|Community\nmodule|developer|Developer\nmodule|profile|Profile\nmodule|organizations|Organizations\nmodule|legal|Legal\nmodule|marketplace|Marketplace\nmodule|bank|Bank\nmodule|exchange|Exchange\nworkflow|asset-transfer|Asset Transfer\nworkflow|capital-exchange|Capital Exchange\nworkflow|community-showcase|Community Showcase\nworkflow|coop-governance|Cooperative Governance\nworkflow|idea-to-outcome|Idea to Outcome\nworkflow|idea-tracker|Idea Tracker\nworkflow|investor-outreach|Investor Outreach\nworkflow|labor-market|Labor Market\nworkflow|marketplace-exchange|Marketplace Exchange\nworkflow|note-creation|Note Creation\nworkflow|portfolio-governance|Portfolio Governance\nworkflow|program-pipeline|Program Pipeline\nworkflow|project-spotlight|Project Spotlight\nworkflow|project-workflow|Project Workflow\nworkflow|prototype-lifecycle|Prototype Lifecycle\nworkflow|resource-exchange|Resource Exchange\nworkflow|resource-finder|Resource Finder\nworkflow|strategy-board|Strategy Board\nworkflow|team-coordination|Team Coordination\nworkflow|tool-builder|Tool Builder\nworkflow|toolchain|Toolchain\nworkflow|tool-integration|Tool Integration\n".to_string()
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
