package com.kogi.desktop;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;

public final class UnifiedScreenCatalog {
    private final String series;
    private final String version;
    private final List<String> sources;
    private final List<UnifiedScreenView> modules;
    private final List<UnifiedScreenView> workflows;

    public UnifiedScreenCatalog(
        String series,
        String version,
        List<String> sources,
        List<UnifiedScreenView> modules,
        List<UnifiedScreenView> workflows
    ) {
        this.series = series;
        this.version = version;
        this.sources = List.copyOf(sources);
        this.modules = List.copyOf(modules);
        this.workflows = List.copyOf(workflows);
    }

    public static UnifiedScreenCatalog fallback() {
        return new UnifiedScreenCatalog(
            "kogi-unified-screen-system",
            "v3-reconciled",
            List.of(
                "Kogi_Screen_Flows_v2.pdf",
                "Kogi_Screen_Flows (2).pdf",
                "Kogi Platform - Screen Flows v2.pdf",
                "Kogi Platform - Screen Flows v3.pdf"
            ),
            List.of(
                UnifiedScreenView.module(
                    "home",
                    "Home",
                    List.of("dashboard", "profile", "workspace"),
                    List.of("Overview Cards", "Quicklinks", "Alerts", "Profile Hub", "Workspace Hub")
                ),
                UnifiedScreenView.module(
                    "dashboard",
                    "Dashboard",
                    List.of("overview", "activity", "ai"),
                    List.of("Portfolio Health", "Active Projects", "Net Revenue", "Recent Activity")
                ),
                UnifiedScreenView.module(
                    "office",
                    "Office",
                    List.of("projects", "programs", "portfolio"),
                    List.of("Programs and Projects", "Milestones Due", "Team Capacity", "Integrations")
                ),
                UnifiedScreenView.module(
                    "workspace",
                    "Workspace",
                    List.of("tasks", "kanban", "sprints"),
                    List.of("Kanban Board", "Open Tasks", "Sprints", "Gantt Timeline")
                ),
                UnifiedScreenView.module(
                    "timeline",
                    "Timeline",
                    List.of("calendar", "roadmap", "gantt"),
                    List.of("Master Timeline", "Scheduled Events", "Milestones", "Deadlines")
                ),
                UnifiedScreenView.module(
                    "portfolio",
                    "Portfolio",
                    List.of("assets", "solutions", "artifacts"),
                    List.of("Projects", "Programs", "Assets", "Artifacts")
                ),
                UnifiedScreenView.module(
                    "strategy",
                    "Strategy",
                    List.of("strategy", "tactics", "governance"),
                    List.of("Strategic OKRs", "Tactical Initiatives", "Operations", "Governance")
                ),
                UnifiedScreenView.module(
                    "studio",
                    "Studio",
                    List.of("ideas", "prototypes", "tools"),
                    List.of("Ideas and Concepts", "Prototypes", "Testbeds", "Tools")
                ),
                UnifiedScreenView.module(
                    "community",
                    "Community",
                    List.of("feeds", "spaces", "messages"),
                    List.of("Feeds", "Spaces", "Messages", "Linked Platforms")
                ),
                UnifiedScreenView.module(
                    "developer",
                    "Developer",
                    List.of("api", "sdk", "integrations"),
                    List.of("API Reference", "Keys", "Webhooks", "Extensions")
                ),
                UnifiedScreenView.module(
                    "profile",
                    "Profiles",
                    List.of("personas", "settings", "skills"),
                    List.of("Profile Types", "Personas and Roles", "Skills and Contact", "Data and Metadata")
                ),
                UnifiedScreenView.module(
                    "configuration",
                    "Configuration",
                    List.of("settings", "parameters", "policies"),
                    List.of("Settings", "Parameters", "Options", "Policies")
                ),
                UnifiedScreenView.module(
                    "providers",
                    "Providers",
                    List.of("registry", "platforms", "affiliates"),
                    List.of("Registry Overview", "Platform Catalog", "Resources and Versions", "Affiliate Links")
                ),
                UnifiedScreenView.module(
                    "organizations",
                    "Organizations",
                    List.of("coops", "collectives", "teams"),
                    List.of("Organizations Grid", "Proposals", "Roles")
                ),
                UnifiedScreenView.module(
                    "legal",
                    "Legal",
                    List.of("ip", "contracts", "compliance"),
                    List.of("IP", "Contracts", "Compliance", "Audit")
                ),
                UnifiedScreenView.module(
                    "marketplace",
                    "Marketplace",
                    List.of("buy", "sell", "barter"),
                    List.of("Listings", "Buy/Sell/Barter", "Orders")
                ),
                UnifiedScreenView.module(
                    "bank",
                    "Bank",
                    List.of("wallets", "finance", "fundraising"),
                    List.of("Wallet Types", "Capital and Fundraising", "Tax Summary")
                ),
                UnifiedScreenView.module(
                    "exchange",
                    "Exchange",
                    List.of("bids", "deals", "due-diligence"),
                    List.of("Bids and Offers", "Deal Pipeline", "Requests")
                ),
                UnifiedScreenView.module(
                    "network",
                    "Network",
                    List.of("gateway", "services", "discovery"),
                    List.of("Gateway", "Service Mesh", "Registry", "Discovery")
                ),
                UnifiedScreenView.module(
                    "engine",
                    "Engine",
                    List.of("data", "ai", "pipelines"),
                    List.of("Ingest Pipelines", "Optimization", "Recommendations", "Telemetry")
                ),
                UnifiedScreenView.module(
                    "host",
                    "Host",
                    List.of("orchestration", "runtime", "kernel"),
                    List.of("Host Runtime", "Module Orchestration", "Kernel Bridge")
                ),
                UnifiedScreenView.module(
                    "server",
                    "Server",
                    List.of("api", "routing", "gateway"),
                    List.of("API Surface", "Request Routing", "Security")
                ),
                UnifiedScreenView.module(
                    "clients",
                    "Clients",
                    List.of("web", "desktop", "mobile"),
                    List.of("Web Console", "Desktop Studio", "Mobile Control")
                )
            ),
            List.of(
                UnifiedScreenView.workflow(
                    "asset-transfer",
                    "Asset Transfer",
                    "exchange",
                    List.of("transfer", "escrow"),
                    List.of("Select asset", "Set terms", "Assign parties", "Settle")
                ),
                UnifiedScreenView.workflow(
                    "capital-exchange",
                    "Capital Exchange",
                    "bank",
                    List.of("capital", "fundraising"),
                    List.of("Open request", "Match contributors", "Distribute capital")
                ),
                UnifiedScreenView.workflow(
                    "community-showcase",
                    "Community Showcase",
                    "community",
                    List.of("community", "showcase"),
                    List.of("Publish showcase", "Attach artifacts", "Track engagement")
                ),
                UnifiedScreenView.workflow(
                    "coop-governance",
                    "Cooperative Governance",
                    "organizations",
                    List.of("governance", "voting"),
                    List.of("Draft proposal", "Open vote", "Record outcome")
                ),
                UnifiedScreenView.workflow(
                    "idea-to-outcome",
                    "Idea to Outcome",
                    "studio",
                    List.of("idea", "prototype"),
                    List.of("Capture idea", "Prototype", "Validate", "Ship")
                ),
                UnifiedScreenView.workflow(
                    "idea-tracker",
                    "Idea Tracker",
                    "studio",
                    List.of("ideas", "tracker"),
                    List.of("Capture", "Score", "Prioritize", "Assign owner")
                ),
                UnifiedScreenView.workflow(
                    "investor-outreach",
                    "Investor Outreach",
                    "bank",
                    List.of("investor", "outreach"),
                    List.of("Build list", "Create pitch flow", "Schedule outreach", "Log responses")
                ),
                UnifiedScreenView.workflow(
                    "labor-market",
                    "Labor Market",
                    "marketplace",
                    List.of("labor", "matching"),
                    List.of("Publish need", "Match workers", "Negotiate", "Create engagement")
                ),
                UnifiedScreenView.workflow(
                    "marketplace-exchange",
                    "Marketplace Exchange",
                    "marketplace",
                    List.of("marketplace", "exchange"),
                    List.of("Create listing", "Receive offers", "Open deal", "Settle")
                ),
                UnifiedScreenView.workflow(
                    "note-creation",
                    "Note Creation",
                    "studio",
                    List.of("notes", "knowledge"),
                    List.of("Create note", "Tag context", "Link project", "Share")
                ),
                UnifiedScreenView.workflow(
                    "portfolio-governance",
                    "Portfolio Governance",
                    "portfolio",
                    List.of("portfolio", "governance"),
                    List.of("Review item", "Open governance check", "Log decision")
                ),
                UnifiedScreenView.workflow(
                    "program-pipeline",
                    "Program Pipeline",
                    "office",
                    List.of("program", "pipeline"),
                    List.of("Define program", "Create project lanes", "Track progress")
                ),
                UnifiedScreenView.workflow(
                    "project-spotlight",
                    "Project Spotlight",
                    "office",
                    List.of("project", "spotlight"),
                    List.of("Select project", "Assemble metrics", "Publish summary")
                ),
                UnifiedScreenView.workflow(
                    "project-workflow",
                    "Project Workflow",
                    "workspace",
                    List.of("workflow", "kanban"),
                    List.of("Backlog", "In Progress", "Review", "Done")
                ),
                UnifiedScreenView.workflow(
                    "prototype-lifecycle",
                    "Prototype Lifecycle",
                    "studio",
                    List.of("prototype", "lifecycle"),
                    List.of("Prototype", "Test", "Iterate", "Release")
                ),
                UnifiedScreenView.workflow(
                    "resource-exchange",
                    "Resource Exchange",
                    "exchange",
                    List.of("resource", "exchange"),
                    List.of("Offer resource", "Request match", "Validate terms", "Exchange")
                ),
                UnifiedScreenView.workflow(
                    "resource-finder",
                    "Resource Finder",
                    "marketplace",
                    List.of("resource", "discovery"),
                    List.of("Set criteria", "Search", "Compare", "Select")
                ),
                UnifiedScreenView.workflow(
                    "strategy-board",
                    "Strategy Board",
                    "strategy",
                    List.of("strategy", "okr"),
                    List.of("Set objectives", "Map tactics", "Assign owners", "Track KRs")
                ),
                UnifiedScreenView.workflow(
                    "team-coordination",
                    "Team Coordination",
                    "office",
                    List.of("team", "coordination"),
                    List.of("Create plan", "Assign roles", "Sync cadence", "Resolve blockers")
                ),
                UnifiedScreenView.workflow(
                    "tool-builder",
                    "Tool Builder",
                    "developer",
                    List.of("tooling", "builder"),
                    List.of("Define tool spec", "Build extension", "Test", "Publish")
                ),
                UnifiedScreenView.workflow(
                    "toolchain",
                    "Toolchain",
                    "developer",
                    List.of("toolchain", "pipeline"),
                    List.of("Select stack", "Configure pipeline", "Validate workflow")
                ),
                UnifiedScreenView.workflow(
                    "tool-integration",
                    "Tool Integration",
                    "developer",
                    List.of("integration", "api"),
                    List.of("Authorize provider", "Map data", "Set webhook", "Verify sync")
                )
            )
        );
    }

    public String series() {
        return series;
    }

    public String version() {
        return version;
    }

    public List<String> sources() {
        return sources;
    }

    public List<UnifiedScreenView> modules() {
        return modules;
    }

    public List<UnifiedScreenView> workflows() {
        return workflows;
    }

    public List<UnifiedScreenView> screensForKind(String kind) {
        if ("workflow".equals(kind)) {
            return workflows;
        }
        return modules;
    }

    public UnifiedScreenCatalog mergeFlat(String flatCatalog) {
        if (flatCatalog == null || flatCatalog.isBlank()) {
            return this;
        }

        Map<String, UnifiedScreenView> moduleIndex = indexById(modules);
        Map<String, UnifiedScreenView> workflowIndex = indexById(workflows);
        List<UnifiedScreenView> mergedModules = new ArrayList<>();
        List<UnifiedScreenView> mergedWorkflows = new ArrayList<>();

        for (String line : flatCatalog.split("\\R")) {
            String trimmed = line.trim();
            if (trimmed.isEmpty()) {
                continue;
            }
            String[] parts = trimmed.split("\\|", 3);
            if (parts.length != 3) {
                continue;
            }

            String kind = parts[0].trim().toLowerCase(Locale.ROOT);
            String id = parts[1].trim();
            String title = parts[2].trim();

            if ("module".equals(kind)) {
                UnifiedScreenView base = moduleIndex.get(id);
                mergedModules.add(base == null ? UnifiedScreenView.module(id, title, List.of(), List.of()) : base.withTitle(title));
            } else if ("workflow".equals(kind)) {
                UnifiedScreenView base = workflowIndex.get(id);
                mergedWorkflows.add(base == null ? UnifiedScreenView.workflow(id, title, "", List.of(), List.of()) : base.withTitle(title));
            }
        }

        if (mergedModules.isEmpty()) {
            mergedModules = new ArrayList<>(modules);
        }
        if (mergedWorkflows.isEmpty()) {
            mergedWorkflows = new ArrayList<>(workflows);
        }

        return new UnifiedScreenCatalog(series, version, sources, mergedModules, mergedWorkflows);
    }

    private static Map<String, UnifiedScreenView> indexById(List<UnifiedScreenView> screens) {
        Map<String, UnifiedScreenView> index = new LinkedHashMap<>();
        for (UnifiedScreenView screen : screens) {
            index.put(screen.id(), screen);
        }
        return index;
    }
}
