import { bootstrapApplication } from '@angular/platform-browser';
import { CommonModule } from '@angular/common';
import { Component, computed, inject, signal } from '@angular/core';
import { HttpClientModule } from '@angular/common/http';
import { firstValueFrom } from 'rxjs';
import { ApiService } from './app/core/api.service';

type ViewKind = 'module' | 'workflow';
type AppMode = 'office' | 'unified' | 'platform' | 'providers';
type OfficeViewId = 'dashboard' | 'portfolio' | 'timeline' | 'workspace' | 'assistant';
type PlatformViewId =
  | 'system'
  | 'host'
  | 'host_components'
  | 'engine'
  | 'engine_runtime'
  | 'database'
  | 'modules'
  | 'autonomy';
type ProviderViewId =
  | 'snapshot'
  | 'platforms'
  | 'providers'
  | 'resources'
  | 'versions'
  | 'metadata'
  | 'data'
  | 'affiliates'
  | 'affiliate_links';

interface UnifiedScreen {
  id: string;
  title: string;
  tags?: string[];
  sections?: string[];
  module?: string;
  steps?: string[];
}

interface UnifiedScreenCatalog {
  series: string;
  version: string;
  sources: string[];
  modules: UnifiedScreen[];
  workflows: UnifiedScreen[];
}

interface NavItem {
  id: string;
  title: string;
}

interface ViewSpec<T extends string> {
  id: T;
  title: string;
  subtitle: string;
  sections: string[];
  flows: string[];
  integrations: string[];
  quickLinks: string[];
  endpoint: string;
}

type OfficeViewSpec = ViewSpec<OfficeViewId>;
type PlatformViewSpec = ViewSpec<PlatformViewId>;
type ProviderViewSpec = ViewSpec<ProviderViewId>;

const fallbackCatalog: UnifiedScreenCatalog = {
  series: 'kogi-unified-screen-system',
  version: 'v3-reconciled',
  sources: [
    'Kogi_Screen_Flows_v2.pdf',
    'Kogi_Screen_Flows (2).pdf',
    'Kogi Platform - Screen Flows v2.pdf',
    'Kogi Platform - Screen Flows v3.pdf',
  ],
  modules: [
    {
      id: 'home',
      title: 'Home',
      tags: ['dashboard', 'profile', 'workspace'],
      sections: ['Overview Cards', 'Quicklinks', 'Alerts', 'Profile Hub', 'Workspace Hub'],
    },
    { id: 'dashboard', title: 'Dashboard', tags: ['overview', 'activity', 'ai'], sections: ['Portfolio Health', 'Quick Access Modules', 'Recent Activity'] },
    { id: 'office', title: 'Office', tags: ['projects', 'programs', 'portfolio'], sections: ['Programs and Projects', 'Milestones Due', 'Team Capacity', '3rd Party Integrations'] },
    { id: 'workspace', title: 'Workspace', tags: ['tasks', 'kanban', 'sprints'], sections: ['Kanban Board', 'Calendar', 'Gantt Timeline'] },
    { id: 'timeline', title: 'Timeline', tags: ['calendar', 'roadmap', 'gantt'], sections: ['Master Timeline', 'Scheduled Events', 'Deadlines'] },
    { id: 'portfolio', title: 'Portfolio', tags: ['assets', 'solutions', 'artifacts'], sections: ['Portfolio Grid', 'Linked Platforms'] },
    { id: 'strategy', title: 'Strategy', tags: ['strategy', 'tactics', 'governance'], sections: ['Strategic OKRs', 'Tactical Initiatives', 'Governance'] },
    { id: 'studio', title: 'Studio', tags: ['ideas', 'prototypes', 'tools'], sections: ['Ideas Grid', 'Testbeds', 'Toolsets'] },
    { id: 'community', title: 'Community', tags: ['feeds', 'spaces', 'messages'], sections: ['Feeds and Timelines', 'Spaces and Rooms', 'Direct Messages'] },
    { id: 'developer', title: 'Developer', tags: ['api', 'sdk', 'integrations'], sections: ['API Reference', 'Webhooks', 'Extensions'] },
    { id: 'profile', title: 'Profiles', tags: ['personas', 'settings', 'skills'], sections: ['Profile Types', 'Personas and Roles', 'Skills and Contact', 'Data and Metadata'] },
    { id: 'configuration', title: 'Configuration', tags: ['settings', 'parameters', 'policies'], sections: ['Settings', 'Parameters', 'Options', 'Policies'] },
    { id: 'providers', title: 'Providers', tags: ['registry', 'platforms', 'affiliates'], sections: ['Registry Overview', 'Platform Catalog', 'Resources and Versions', 'Affiliate Links'] },
    { id: 'organizations', title: 'Center', tags: ['coops', 'collectives', 'teams'], sections: ['Organizations Grid', 'Governance and Proposals', 'Federations'] },
    { id: 'marketplace', title: 'Marketplace', tags: ['buy', 'sell', 'barter'], sections: ['Marketplace Grid', 'Barter System', 'My Orders'] },
    { id: 'bank', title: 'Bank', tags: ['wallets', 'finance', 'fundraising'], sections: ['Wallet Types', 'Fundraising and Capital', 'Tax Summary'] },
    { id: 'exchange', title: 'Exchange', tags: ['bids', 'deals', 'due-diligence'], sections: ['Bids and Offers', 'Deal Pipeline', 'Requests'] },
    { id: 'network', title: 'Network', tags: ['gateway', 'services', 'discovery'], sections: ['Gateway', 'Service Mesh', 'Registry', 'Discovery'] },
    { id: 'engine', title: 'Engine', tags: ['data', 'ai', 'pipelines'], sections: ['Ingest Pipelines', 'Optimization', 'Recommendations', 'Telemetry'] },
    { id: 'host', title: 'Host', tags: ['orchestration', 'runtime', 'kernel'], sections: ['Host Runtime', 'Module Orchestration', 'Kernel Bridge'] },
    { id: 'server', title: 'Server', tags: ['api', 'routing', 'gateway'], sections: ['API Surface', 'Request Routing', 'Security'] },
    { id: 'clients', title: 'Clients', tags: ['web', 'desktop', 'mobile'], sections: ['Web Console', 'Desktop Studio', 'Mobile Control'] },
  ],
  workflows: [
    { id: 'asset-transfer', title: 'Asset Transfer', module: 'exchange', tags: ['transfer', 'escrow'], steps: ['Select asset', 'Create transfer terms', 'Assign parties', 'Set escrow controls', 'Finalize settlement'] },
    { id: 'capital-exchange', title: 'Capital Exchange', module: 'bank', tags: ['capital', 'governance'], steps: ['Open capital request', 'Match contributors', 'Apply governance checks', 'Distribute capital'] },
    { id: 'community-showcase', title: 'Community Showcase', module: 'community', tags: ['community', 'showcase'], steps: ['Create showcase post', 'Attach artifacts', 'Publish to spaces', 'Track engagement'] },
    { id: 'coop-governance', title: 'Cooperative Governance', module: 'organizations', tags: ['cooperative', 'voting'], steps: ['Draft proposal', 'Open vote', 'Reach quorum', 'Record outcome'] },
    { id: 'idea-to-outcome', title: 'Idea to Outcome', module: 'studio', tags: ['idea', 'outcome'], steps: ['Capture idea', 'Prototype', 'Validate', 'Promote to project', 'Track outcome'] },
    { id: 'idea-tracker', title: 'Idea Tracker', module: 'studio', tags: ['ideas', 'tracker'], steps: ['Capture', 'Score', 'Prioritize', 'Assign owner'] },
    { id: 'investor-outreach', title: 'Investor Outreach', module: 'bank', tags: ['investor', 'outreach'], steps: ['Build investor list', 'Create pitch flow', 'Schedule outreach', 'Log responses'] },
    { id: 'labor-market', title: 'Labor Market', module: 'marketplace', tags: ['labor', 'matching'], steps: ['Publish need', 'Match workers', 'Negotiate terms', 'Create engagement'] },
    { id: 'marketplace-exchange', title: 'Marketplace Exchange', module: 'marketplace', tags: ['marketplace', 'exchange'], steps: ['Create listing', 'Receive offers', 'Open deal', 'Route to exchange settlement'] },
    { id: 'note-creation', title: 'Note Creation', module: 'studio', tags: ['notes', 'knowledge'], steps: ['Create note', 'Tag context', 'Link profile/project', 'Share'] },
    { id: 'portfolio-governance', title: 'Portfolio Governance', module: 'portfolio', tags: ['portfolio', 'governance'], steps: ['Review portfolio item', 'Open governance check', 'Approve/reject', 'Log decision'] },
    { id: 'program-pipeline', title: 'Program Pipeline', module: 'office', tags: ['program', 'pipeline'], steps: ['Define program', 'Create project lanes', 'Track progress', 'Report status'] },
    { id: 'project-spotlight', title: 'Project Spotlight', module: 'office', tags: ['project', 'spotlight'], steps: ['Select project', 'Assemble metrics', 'Publish summary'] },
    { id: 'project-workflow', title: 'Project Workflow', module: 'workspace', tags: ['workflow', 'kanban'], steps: ['Backlog', 'In Progress', 'Review', 'Done'] },
    { id: 'prototype-lifecycle', title: 'Prototype Lifecycle', module: 'studio', tags: ['prototype', 'lifecycle'], steps: ['Prototype', 'Test', 'Iterate', 'Release'] },
    { id: 'resource-exchange', title: 'Resource Exchange', module: 'exchange', tags: ['resource', 'exchange'], steps: ['Offer resource', 'Request match', 'Validate terms', 'Exchange'] },
    { id: 'resource-finder', title: 'Resource Finder', module: 'marketplace', tags: ['resource', 'discovery'], steps: ['Set criteria', 'Search', 'Compare', 'Select'] },
    { id: 'strategy-board', title: 'Strategy Board', module: 'strategy', tags: ['strategy', 'okr'], steps: ['Set objectives', 'Map tactics', 'Assign owners', 'Track KRs'] },
    { id: 'team-coordination', title: 'Team Coordination', module: 'office', tags: ['team', 'coordination'], steps: ['Create team plan', 'Assign roles', 'Sync cadence', 'Resolve blockers'] },
    { id: 'tool-builder', title: 'Tool Builder', module: 'developer', tags: ['tooling', 'builder'], steps: ['Define tool spec', 'Build extension', 'Test integration', 'Publish'] },
    { id: 'toolchain', title: 'Toolchain', module: 'developer', tags: ['toolchain', 'pipeline'], steps: ['Select stack', 'Configure pipeline', 'Validate workflow'] },
    { id: 'tool-integration', title: 'Tool Integration', module: 'developer', tags: ['integration', 'api'], steps: ['Authorize provider', 'Map data', 'Set webhook', 'Verify sync'] },
  ],
};

const officeViewSpecs: ReadonlyArray<OfficeViewSpec> = [
  {
    id: 'dashboard',
    title: 'Office Dashboard',
    subtitle: 'Active projects/programs, attention, DMs, event feed, personas, quick links',
    sections: ['Active Projects and Programs', 'Portfolio Attention and Notifications', 'Direct Messages', 'User Event, Community, Marketplace, Exchange Feed', 'Personas and Roles', 'Quick Access Links'],
    flows: ['Triage attention items', 'Respond to direct messages', 'Open quick links to priority tools'],
    integrations: ['jira', 'monday', 'openai', 'gitlab', 'github'],
    quickLinks: ['/office/portfolio', '/office/timeline', '/office/workspace', '/office/assistant'],
    endpoint: '/api/v1/office/dashboard',
  },
  {
    id: 'portfolio',
    title: 'Office Portfolio',
    subtitle: 'Tiled/tree/modular grid + focus view with item containers and metadata',
    sections: ['Portfolio Grid Modes', 'Item Types (project/program/resource/asset/capital/investment/solution/document/misc/custom)', 'Focus View', 'Binder/Book/Notebook/Playbook/Folders/Files/Version/Metadata'],
    flows: ['Switch view mode', 'Select focus item', 'Open container and metadata stack'],
    integrations: ['github', 'gitlab', 'claude', 'chatgpt'],
    quickLinks: ['/office/portfolio?mode=tiled', '/office/portfolio?mode=tree', '/office/portfolio?mode=modular_grid'],
    endpoint: '/api/v1/office/portfolio',
  },
  {
    id: 'timeline',
    title: 'Office Timeline',
    subtitle: 'Calendars, schedules, roadmaps, gantts, and personal timelines',
    sections: ['Calendars', 'Schedules', 'Roadmaps', 'Gantts', 'Personal Timelines'],
    flows: ['Resolve scheduling conflicts', 'Track roadmap milestones', 'Inspect critical path'],
    integrations: ['jira', 'monday', 'google-calendar'],
    quickLinks: ['/office/timeline?view=calendar', '/office/timeline?view=gantt'],
    endpoint: '/api/v1/office/timeline',
  },
  {
    id: 'workspace',
    title: 'Office Workspace',
    subtitle: 'Personal work/operations/tactics/strategy/governance with stories and toolchains',
    sections: ['Personal Work Domains', 'User Stories and Work Packages', 'Content Management System', 'Tools, Toolchains, Toolkits, Toolsets'],
    flows: ['Prioritize stories', 'Move work packages', 'Open toolchain links'],
    integrations: ['jira', 'monday', 'github', 'gitlab'],
    quickLinks: ['/office/workspace?panel=stories', '/office/workspace?panel=tools'],
    endpoint: '/api/v1/office/workspace',
  },
  {
    id: 'assistant',
    title: 'Office Assistant',
    subtitle: 'AI chat/context plus discovery, recommendations, subscriptions, explore, for-you',
    sections: ['Chat Context Window', 'Discover', 'Recommendations', 'Subscriptions', 'Explore', 'For You'],
    flows: ['Ask contextual question', 'Apply recommendation', 'Subscribe to updates'],
    integrations: ['openai', 'chatgpt', 'claude', 'grok'],
    quickLinks: ['/office/assistant?panel=chat', '/office/assistant?panel=recommendations'],
    endpoint: '/api/v1/office/assistant',
  },
];

const platformViewSpecs: ReadonlyArray<PlatformViewSpec> = [
  {
    id: 'system',
    title: 'Platform System',
    subtitle: 'Kernel, host, modules, registry totals, and cross-service flow map',
    sections: ['Kernel + Host Mode', 'Module + Component Counts', 'Provider Registry Totals', 'Data Flow Map'],
    flows: ['Review active services', 'Confirm data flow', 'Audit health status'],
    integrations: ['kogi-server', 'kogi-host', 'gateway'],
    quickLinks: ['/api/v1/system', '/api/v1/host', '/api/v1/modules'],
    endpoint: '/api/v1/system',
  },
  {
    id: 'host',
    title: 'Host Summary',
    subtitle: 'Host runtime, component counts, provider totals, and kernel mode',
    sections: ['Host Boot Status', 'Component and Module Totals', 'Provider Registry Totals', 'Kernel Mode'],
    flows: ['Inspect host runtime', 'Verify module orchestration', 'Review provider inventory'],
    integrations: ['kernel', 'modules', 'services'],
    quickLinks: ['/api/v1/host', '/api/v1/host/components'],
    endpoint: '/api/v1/host',
  },
  {
    id: 'host_components',
    title: 'Host Components',
    subtitle: 'Kernel-managed components, limits, and network manager assignments',
    sections: ['Component Inventory', 'Resource Limits', 'Network Managers'],
    flows: ['Inspect component limits', 'Audit active component set'],
    integrations: ['kernel', 'network'],
    quickLinks: ['/api/v1/host/components'],
    endpoint: '/api/v1/host/components',
  },
  {
    id: 'engine',
    title: 'Engine Overview',
    subtitle: 'Data engine status, ingest topics, and capability map',
    sections: ['Engine Status', 'Capabilities', 'Ingest Topics', 'Flow Map'],
    flows: ['Review ingest readiness', 'Validate analytics capabilities'],
    integrations: ['kogi-engine', 'gateway'],
    quickLinks: ['/api/v1/engine/system', '/api/v1/engine/runtime'],
    endpoint: '/api/v1/engine/system',
  },
  {
    id: 'engine_runtime',
    title: 'Engine Runtime',
    subtitle: 'Service runtime snapshot for the data engine',
    sections: ['Runtime Status', 'Service Health', 'Latency and Throughput'],
    flows: ['Ping engine runtime', 'Verify service health'],
    integrations: ['kogi-engine', 'gateway'],
    quickLinks: ['/api/v1/engine/runtime'],
    endpoint: '/api/v1/engine/runtime',
  },
  {
    id: 'database',
    title: 'Database Runtime',
    subtitle: 'Database service status and query interface',
    sections: ['Runtime Status', 'Query Interface', 'Data Store Health'],
    flows: ['Run validation query', 'Review runtime health'],
    integrations: ['kogi-database', 'gateway'],
    quickLinks: ['/api/v1/database/runtime', '/api/v1/database/query'],
    endpoint: '/api/v1/database/runtime',
  },
  {
    id: 'modules',
    title: 'Module Registry',
    subtitle: 'Live module inventory, capabilities, and integration map',
    sections: ['Module Inventory', 'Capabilities', 'Integrations'],
    flows: ['Review module versions', 'Verify integration coverage'],
    integrations: ['kogi-host', 'module services'],
    quickLinks: ['/api/v1/modules'],
    endpoint: '/api/v1/modules',
  },
  {
    id: 'autonomy',
    title: 'Autonomy Capabilities',
    subtitle: 'System-level autonomy primitives for independent workers',
    sections: ['Identity Management', 'Workspace Organization', 'Connection Registry', 'Asset Vault'],
    flows: ['Review autonomy coverage', 'Audit capability list'],
    integrations: ['kogi-host', 'kogi-engine'],
    quickLinks: ['/api/v1/autonomy/capabilities'],
    endpoint: '/api/v1/autonomy/capabilities',
  },
];

const providerViewSpecs: ReadonlyArray<ProviderViewSpec> = [
  {
    id: 'snapshot',
    title: 'Provider Snapshot',
    subtitle: 'Registry totals, active providers, and affiliate links',
    sections: ['Totals', 'Active Providers', 'Active Platforms', 'Affiliate Links'],
    flows: ['Review registry health', 'Validate active inventory'],
    integrations: ['kogi-host', 'provider registry'],
    quickLinks: ['/api/v1/providers', '/api/v1/providers/platforms'],
    endpoint: '/api/v1/providers',
  },
  {
    id: 'platforms',
    title: 'Platforms',
    subtitle: 'Platform catalog with status, links, and tags',
    sections: ['Platform Catalog', 'Status Overview', 'Support Contacts'],
    flows: ['Audit platform coverage', 'Verify platform status'],
    integrations: ['registry', 'platforms'],
    quickLinks: ['/api/v1/providers/platforms'],
    endpoint: '/api/v1/providers/platforms',
  },
  {
    id: 'providers',
    title: 'Providers',
    subtitle: 'Provider inventory with owners, tags, and current versions',
    sections: ['Provider Inventory', 'Owners and Contacts', 'Version Coverage'],
    flows: ['Review provider status', 'Check version coverage'],
    integrations: ['registry', 'provider services'],
    quickLinks: ['/api/v1/providers/providers'],
    endpoint: '/api/v1/providers/providers',
  },
  {
    id: 'resources',
    title: 'Provider Resources',
    subtitle: 'Registered resources, endpoints, and credential refs',
    sections: ['Resource Inventory', 'Environments', 'Credential References'],
    flows: ['Inspect resource endpoints', 'Validate credentials'],
    integrations: ['provider services'],
    quickLinks: ['/api/v1/providers/resources'],
    endpoint: '/api/v1/providers/resources',
  },
  {
    id: 'versions',
    title: 'Provider Versions',
    subtitle: 'Version control snapshots across providers',
    sections: ['Version Inventory', 'Release Status', 'Compatibility Matrix'],
    flows: ['Validate version coverage', 'Review release notes'],
    integrations: ['provider services'],
    quickLinks: ['/api/v1/providers/versions'],
    endpoint: '/api/v1/providers/versions',
  },
  {
    id: 'metadata',
    title: 'Provider Metadata',
    subtitle: 'Metadata key-value entries and scopes',
    sections: ['Metadata Entries', 'Scopes and Policies', 'Update Times'],
    flows: ['Audit metadata coverage', 'Review scopes'],
    integrations: ['provider services'],
    quickLinks: ['/api/v1/providers/metadata'],
    endpoint: '/api/v1/providers/metadata',
  },
  {
    id: 'data',
    title: 'Provider Data Assets',
    subtitle: 'Provider datasets, sync status, and storage locations',
    sections: ['Datasets', 'Sync Status', 'Storage Locations'],
    flows: ['Check sync status', 'Validate data coverage'],
    integrations: ['provider services', 'storage'],
    quickLinks: ['/api/v1/providers/data'],
    endpoint: '/api/v1/providers/data',
  },
  {
    id: 'affiliates',
    title: 'Affiliates',
    subtitle: 'Affiliate registry entries and partner metadata',
    sections: ['Affiliate Inventory', 'Partner Metadata', 'Contact Points'],
    flows: ['Review affiliate status', 'Validate contacts'],
    integrations: ['provider registry'],
    quickLinks: ['/api/v1/providers/affiliates'],
    endpoint: '/api/v1/providers/affiliates',
  },
  {
    id: 'affiliate_links',
    title: 'Affiliate Links',
    subtitle: 'Provider-affiliate link status and tracking URLs',
    sections: ['Link Inventory', 'Channels', 'Tracking URLs'],
    flows: ['Audit affiliate links', 'Validate tracking configuration'],
    integrations: ['provider registry'],
    quickLinks: ['/api/v1/providers/affiliate-links'],
    endpoint: '/api/v1/providers/affiliate-links',
  },
];

function isOfficeViewId(value: string): value is OfficeViewId {
  return officeViewSpecs.some((x) => x.id === value);
}

function isPlatformViewId(value: string): value is PlatformViewId {
  return platformViewSpecs.some((x) => x.id === value);
}

function isProviderViewId(value: string): value is ProviderViewId {
  return providerViewSpecs.some((x) => x.id === value);
}

@Component({
  selector: 'kogi-root',
  standalone: true,
  imports: [CommonModule, HttpClientModule],
  providers: [ApiService],
  template: `
    <main class="app-root">
      <header class="topbar">
        <div class="brand">KOGI<span>OS</span></div>
        <label class="search">
          <input
            [value]="navQuery()"
            (input)="setNavQuery(($any($event.target)).value)"
            placeholder="Search modules, views, services" />
        </label>
        <div class="top-actions">
          <button class="ghost" (click)="refreshActive()">Refresh</button>
          <select [value]="activeProfile()" (change)="setProfile(($any($event.target)).value)">
            <option *ngFor="let profile of identityProfiles" [value]="profile.id">{{ profile.name }}</option>
          </select>
        </div>
      </header>

      <section class="workspace-shell">
        <aside class="icon-rail">
          <button class="rail-btn active">?</button>
          <button class="rail-btn">?</button>
          <button class="rail-btn">?</button>
          <button class="rail-btn">?</button>
        </aside>

        <aside class="navigator">
          <div class="mode-row">
            <button class="mode-btn" [class.active]="appMode() === 'office'" (click)="setAppMode('office')">Office</button>
            <button class="mode-btn" [class.active]="appMode() === 'unified'" (click)="setAppMode('unified')">Unified</button>
          </div>

          <div class="mode-row">
            <button class="mode-btn" [class.active]="appMode() === 'platform'" (click)="setAppMode('platform')">Platform</button>
            <button class="mode-btn" [class.active]="appMode() === 'providers'" (click)="setAppMode('providers')">Providers</button>
          </div>

          <div class="mode-row" *ngIf="appMode() === 'unified'">
            <button class="mode-btn" [class.active]="viewKind() === 'module'" (click)="setViewKind('module')">Modules</button>
            <button class="mode-btn" [class.active]="viewKind() === 'workflow'" (click)="setViewKind('workflow')">Workflows</button>
          </div>

          <p class="label">Views</p>
          <div class="nav-list">
            <button
              *ngFor="let item of navItems()"
              class="nav-item"
              [class.active]="isActiveNav(item.id)"
              (click)="selectNav(item.id)">
              <span>{{ item.title }}</span>
            </button>
          </div>
        </aside>

        <section class="main-stage">
          <article class="hero">
            <div>
              <h1>{{ activeTitle() }}</h1>
              <p>{{ activeSubtitle() }}</p>
            </div>
            <div class="hero-chip">
              <span>{{ activeModeLabel() }}</span>
            </div>
          </article>

          <section class="meta-row">
            <article class="metric">
              <span>Series</span>
              <strong>{{ activeSeries() }}</strong>
            </article>
            <article class="metric">
              <span>Version</span>
              <strong>{{ activeVersion() }}</strong>
            </article>
            <article class="metric">
              <span>Visible Views</span>
              <strong>{{ navItems().length }}</strong>
            </article>
            <article class="metric">
              <span>Endpoint</span>
              <strong>{{ activeEndpoint() }}</strong>
            </article>
          </section>

          <section class="panel-grid">
            <article class="panel">
              <h2>Sections</h2>
              <div class="chips">
                <span class="chip" *ngFor="let section of displaySections()">{{ section }}</span>
              </div>
            </article>

            <article class="panel">
              <h2>Flows</h2>
              <ol>
                <li *ngFor="let step of displayFlows()">{{ step }}</li>
              </ol>
            </article>

            <article class="panel">
              <h2>{{ appMode() === 'unified' ? 'Tags' : 'Integrations' }}</h2>
              <div class="chips">
                <span class="chip chip-alt" *ngFor="let tag of displayTags()">{{ tag }}</span>
              </div>
            </article>

            <article class="panel">
              <h2>{{ appMode() === 'unified' ? 'Source Files' : 'Quick Links' }}</h2>
              <ul>
                <li *ngFor="let link of displayLinks()">{{ link }}</li>
              </ul>
            </article>
          </section>

          <section class="tool-row">
            <button (click)="loadSystem()">System</button>
            <button (click)="loadHostSummary()">Host</button>
            <button (click)="loadHostComponents()">Host Components</button>
            <button (click)="loadModulesList()">Modules</button>
            <button (click)="loadEngineOverview()">Engine</button>
            <button (click)="loadEngineRuntime()">Engine Runtime</button>
            <button (click)="loadDatabaseRuntime()">Database</button>
            <button (click)="loadProvidersSnapshot()">Providers</button>
            <button (click)="loadProvidersAffiliates()">Affiliates</button>
            <button (click)="loadIdentities()">IMS Identities</button>
            <button (click)="loadProfiles()">IMS Profiles</button>
            <button (click)="loadIsolation()">Module Isolation</button>
            <button (click)="loadOfficeOverview()">Office Overview</button>
            <button (click)="loadUnifiedCatalog()">Unified Screens</button>
          </section>

          <section class="action-row" *ngIf="appMode() === 'office'">
            <input
              [value]="actionDraft()"
              (input)="setActionDraft(($any($event.target)).value)"
              placeholder="Office action input (id, name, topic)" />
            <button (click)="ackOfficeNotification()">Ack</button>
            <button (click)="createOfficePortfolioItem()">Portfolio+</button>
            <button (click)="createOfficeTimelineEvent()">Timeline+</button>
            <button (click)="createOfficeWorkspaceStory()">Story+</button>
            <button (click)="subscribeOfficeAssistant()">Subscribe+</button>
          </section>

          <section class="action-row" *ngIf="appMode() === 'platform'">
            <input
              [value]="actionDraft()"
              (input)="setActionDraft(($any($event.target)).value)"
              placeholder="Platform action input (engine action or SQL)" />
            <button (click)="engineControl()">Engine Control</button>
            <button (click)="engineIngest()">Engine Ingest</button>
            <button (click)="databaseQuery()">DB Query</button>
          </section>

          <section class="action-row" *ngIf="appMode() === 'providers'">
            <input
              [value]="actionDraft()"
              (input)="setActionDraft(($any($event.target)).value)"
              placeholder="Provider action input (name|id|extra)" />
            <button (click)="createProviderPlatform()">Platform+</button>
            <button (click)="createProvider()">Provider+</button>
            <button (click)="addProviderResource()">Resource+</button>
            <button (click)="addProviderVersion()">Version+</button>
            <button (click)="setProviderMetadata()">Metadata+</button>
            <button (click)="addProviderDataAsset()">Data+</button>
            <button (click)="registerAffiliate()">Affiliate+</button>
            <button (click)="addAffiliateLink()">Affiliate Link+</button>
          </section>
        </section>

        <aside class="insights">
          <h3>Realtime Payload</h3>
          <pre>{{ payload() }}</pre>
        </aside>
      </section>
    </main>
  `,
  styles: [
    `
    :host { display: block; min-height: 100vh; }
    * { box-sizing: border-box; }
    .app-root {
      min-height: 100vh;
      color: #e4eeff;
      background:
        radial-gradient(1200px 500px at 10% -10%, rgba(61, 130, 255, .28), transparent 60%),
        radial-gradient(1000px 700px at 95% 10%, rgba(23, 214, 255, .18), transparent 60%),
        #070b17;
      font-family: "Space Grotesk", "Manrope", "Segoe UI", sans-serif;
      padding: 16px;
    }
    .topbar {
      display: grid;
      grid-template-columns: 170px 1fr auto;
      gap: 14px;
      align-items: center;
      background: linear-gradient(90deg, rgba(9, 22, 58, .88), rgba(8, 17, 42, .88));
      border: 1px solid rgba(98, 144, 255, .3);
      border-radius: 16px;
      padding: 12px;
      margin-bottom: 14px;
      backdrop-filter: blur(6px);
    }
    .brand {
      font-size: 1.34rem;
      font-weight: 800;
      letter-spacing: .12rem;
      color: #91c5ff;
    }
    .brand span {
      color: #49e2ff;
      margin-left: 4px;
    }
    .search input {
      width: 100%;
      border: 1px solid rgba(115, 151, 245, .35);
      border-radius: 10px;
      background: rgba(6, 16, 41, .78);
      color: #dbe9ff;
      padding: 11px 12px;
      outline: none;
    }
    .top-actions {
      display: flex;
      gap: 10px;
      align-items: center;
    }
    .ghost {
      border: 1px solid rgba(106, 146, 255, .42);
      color: #b5d8ff;
      background: rgba(14, 30, 73, .6);
      border-radius: 10px;
      padding: 9px 12px;
      cursor: pointer;
    }
    select {
      border: 1px solid rgba(115, 151, 245, .35);
      border-radius: 10px;
      background: rgba(6, 16, 41, .78);
      color: #dbe9ff;
      padding: 9px 10px;
    }
    .workspace-shell {
      display: grid;
      grid-template-columns: 58px 250px 1fr 330px;
      gap: 12px;
      min-height: calc(100vh - 102px);
    }
    .icon-rail, .navigator, .main-stage, .insights {
      border: 1px solid rgba(87, 123, 210, .35);
      background: linear-gradient(160deg, rgba(11, 24, 60, .82), rgba(7, 16, 40, .92));
      border-radius: 14px;
      backdrop-filter: blur(4px);
    }
    .icon-rail {
      padding: 10px 8px;
      display: flex;
      flex-direction: column;
      gap: 8px;
    }
    .rail-btn {
      border: 1px solid rgba(117, 155, 252, .26);
      background: rgba(20, 40, 86, .55);
      color: #91b8ff;
      border-radius: 10px;
      height: 42px;
      cursor: pointer;
      font-size: 1.02rem;
    }
    .rail-btn.active { color: #1f1010; background: linear-gradient(135deg, #5be5ff, #5cb4ff); }

    .navigator {
      padding: 12px;
      display: flex;
      flex-direction: column;
      gap: 10px;
    }
    .mode-row {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 8px;
    }
    .mode-btn {
      border: 1px solid rgba(123, 157, 243, .28);
      background: rgba(13, 27, 68, .7);
      color: #a7c7ff;
      border-radius: 10px;
      padding: 8px;
      cursor: pointer;
    }
    .mode-btn.active {
      color: #041223;
      font-weight: 700;
      background: linear-gradient(135deg, #6deaff, #70bdff);
    }
    .label {
      margin: 2px 0 0;
      text-transform: uppercase;
      letter-spacing: .08rem;
      font-size: .72rem;
      color: #82b3ff;
    }
    .nav-list {
      display: flex;
      flex-direction: column;
      gap: 7px;
      overflow: auto;
      padding-right: 3px;
    }
    .nav-item {
      border: 1px solid rgba(114, 145, 235, .24);
      background: rgba(13, 27, 68, .7);
      color: #c6dcff;
      border-radius: 10px;
      padding: 9px;
      text-align: left;
      cursor: pointer;
      font-size: .9rem;
    }
    .nav-item.active {
      border-color: rgba(82, 229, 255, .6);
      box-shadow: inset 0 0 0 1px rgba(45, 226, 255, .44);
      background: linear-gradient(135deg, rgba(22, 53, 120, .85), rgba(21, 77, 137, .85));
    }

    .main-stage {
      padding: 14px;
      display: flex;
      flex-direction: column;
      gap: 12px;
      overflow: auto;
    }
    .hero {
      display: flex;
      justify-content: space-between;
      gap: 12px;
      align-items: center;
      border: 1px solid rgba(116, 154, 248, .24);
      border-radius: 14px;
      background: linear-gradient(140deg, rgba(12, 32, 82, .86), rgba(10, 50, 96, .86));
      padding: 14px;
    }
    .hero h1 { margin: 0; font-size: 1.9rem; font-weight: 700; letter-spacing: .01rem; }
    .hero p { margin: 5px 0 0; color: #b3cbf7; }
    .hero-chip {
      border: 1px solid rgba(80, 232, 255, .42);
      border-radius: 999px;
      padding: 7px 12px;
      color: #79eaff;
      background: rgba(15, 69, 120, .55);
      font-size: .82rem;
      white-space: nowrap;
    }
    .meta-row {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
      gap: 10px;
    }
    .metric {
      border: 1px solid rgba(112, 145, 235, .22);
      border-radius: 12px;
      background: rgba(11, 24, 58, .8);
      padding: 10px;
    }
    .metric span { display: block; font-size: .75rem; color: #8eb4ee; }
    .metric strong { display: block; margin-top: 4px; color: #8ef0ff; font-size: 1.04rem; font-weight: 600; }
    .panel-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
      gap: 10px;
    }
    .panel {
      border: 1px solid rgba(111, 143, 232, .22);
      border-radius: 12px;
      background: rgba(11, 24, 58, .82);
      padding: 12px;
    }
    .panel h2 {
      margin: 0 0 9px;
      font-size: .95rem;
      letter-spacing: .02rem;
      color: #d5e6ff;
    }
    .chips {
      display: flex;
      gap: 7px;
      flex-wrap: wrap;
    }
    .chip {
      border: 1px solid rgba(105, 148, 243, .28);
      border-radius: 999px;
      padding: 4px 10px;
      font-size: .78rem;
      color: #b7d6ff;
      background: rgba(18, 55, 109, .6);
    }
    .chip-alt {
      color: #85f2ff;
      border-color: rgba(72, 214, 255, .4);
      background: rgba(8, 83, 123, .5);
    }
    ol, ul {
      margin: 0;
      padding-left: 17px;
      color: #c3dafd;
    }
    li + li { margin-top: 3px; }
    .tool-row, .action-row {
      display: flex;
      gap: 8px;
      flex-wrap: wrap;
    }
    .tool-row button, .action-row button {
      border: 1px solid rgba(112, 146, 240, .32);
      border-radius: 10px;
      background: rgba(17, 37, 82, .8);
      color: #c3ddff;
      padding: 8px 11px;
      cursor: pointer;
    }
    .action-row input {
      min-width: 230px;
      border: 1px solid rgba(112, 146, 240, .32);
      border-radius: 10px;
      background: rgba(9, 22, 52, .88);
      color: #dbe9ff;
      padding: 8px 10px;
      outline: none;
    }
    .insights {
      padding: 12px;
      display: flex;
      flex-direction: column;
      gap: 8px;
      overflow: hidden;
    }
    .insights h3 {
      margin: 0;
      color: #b8d4ff;
      font-size: .95rem;
      letter-spacing: .03rem;
    }
    pre {
      margin: 0;
      flex: 1;
      min-height: 180px;
      overflow: auto;
      border-radius: 10px;
      border: 1px solid rgba(113, 147, 237, .3);
      background: rgba(6, 14, 35, .95);
      color: #cde1ff;
      padding: 11px;
      white-space: pre-wrap;
      font-size: .79rem;
      line-height: 1.35;
      font-family: "JetBrains Mono", "Cascadia Code", monospace;
    }

    @media (max-width: 1320px) {
      .workspace-shell { grid-template-columns: 54px 220px 1fr; }
      .insights { grid-column: 1 / -1; min-height: 230px; }
    }
    @media (max-width: 980px) {
      .topbar { grid-template-columns: 1fr; }
      .workspace-shell { grid-template-columns: 1fr; }
      .icon-rail {
        flex-direction: row;
        justify-content: center;
      }
      .navigator { max-height: 300px; }
    }
    `,
  ],
})
export class AppComponent {
  private readonly api = inject(ApiService);

  readonly identityProfiles = [
    { id: 'personal', name: 'Personal Profile' },
    { id: 'work', name: 'Work Profile' },
    { id: 'business', name: 'Business Profile' },
    { id: 'community', name: 'Community Profile' },
  ];

  readonly catalog = signal<UnifiedScreenCatalog>(fallbackCatalog);
  readonly appMode = signal<AppMode>('office');
  readonly viewKind = signal<ViewKind>('module');
  readonly activeUnifiedId = signal('dashboard');
  readonly activeOfficeId = signal<OfficeViewId>('dashboard');
  readonly activePlatformId = signal<PlatformViewId>('system');
  readonly activeProviderId = signal<ProviderViewId>('snapshot');
  readonly activeProfile = signal('work');
  readonly payload = signal('Select a view from the left sidebar.');
  readonly actionDraft = signal('');
  readonly navQuery = signal('');

  readonly activeUnifiedItems = computed(() =>
    this.viewKind() === 'module' ? this.catalog().modules : this.catalog().workflows,
  );

  readonly activeUnifiedScreen = computed(() =>
    this.activeUnifiedItems().find((screen) => screen.id === this.activeUnifiedId()),
  );

  readonly activeOfficeSpec = computed(() =>
    officeViewSpecs.find((x) => x.id === this.activeOfficeId()) ?? officeViewSpecs[0],
  );

  readonly activePlatformSpec = computed(() =>
    platformViewSpecs.find((x) => x.id === this.activePlatformId()) ?? platformViewSpecs[0],
  );

  readonly activeProviderSpec = computed(() =>
    providerViewSpecs.find((x) => x.id === this.activeProviderId()) ?? providerViewSpecs[0],
  );

  readonly navItems = computed<NavItem[]>(() => {
    const raw = this.appMode() === 'office'
      ? officeViewSpecs.map((x) => ({ id: x.id, title: x.title }))
      : this.appMode() === 'platform'
        ? platformViewSpecs.map((x) => ({ id: x.id, title: x.title }))
        : this.appMode() === 'providers'
          ? providerViewSpecs.map((x) => ({ id: x.id, title: x.title }))
          : this.activeUnifiedItems().map((x) => ({ id: x.id, title: x.title }));

    const query = this.navQuery().trim().toLowerCase();
    if (!query) {
      return raw;
    }
    return raw.filter((x) => x.title.toLowerCase().includes(query));
  });

  readonly activeTitle = computed(() => {
    if (this.appMode() === 'office') {
      return this.activeOfficeSpec().title;
    }
    if (this.appMode() === 'platform') {
      return this.activePlatformSpec().title;
    }
    if (this.appMode() === 'providers') {
      return this.activeProviderSpec().title;
    }
    return this.activeUnifiedScreen()?.title ?? 'Kogi';
  });

  readonly activeSubtitle = computed(() => {
    if (this.appMode() === 'office') {
      return this.activeOfficeSpec().subtitle;
    }
    if (this.appMode() === 'platform') {
      return this.activePlatformSpec().subtitle;
    }
    if (this.appMode() === 'providers') {
      return this.activeProviderSpec().subtitle;
    }
    return this.viewKind() === 'module'
      ? 'Unified module screen from reconciled v2/v3 docs.'
      : 'Unified workflow screen from reconciled v2/v3 docs.';
  });

  readonly activeSeries = computed(() => {
    if (this.appMode() === 'office') {
      return 'kogi-office-application';
    }
    if (this.appMode() === 'platform') {
      return 'kogi-platform-architecture';
    }
    if (this.appMode() === 'providers') {
      return 'kogi-provider-registry';
    }
    return this.catalog().series;
  });

  readonly activeVersion = computed(() => {
    if (this.appMode() === 'office') {
      return 'v0.3.0';
    }
    if (this.appMode() === 'platform') {
      return 'v1.0.0';
    }
    if (this.appMode() === 'providers') {
      return 'v0.2.0';
    }
    return this.catalog().version;
  });

  readonly activeEndpoint = computed(() => {
    if (this.appMode() === 'office') {
      return this.activeOfficeSpec().endpoint;
    }
    if (this.appMode() === 'platform') {
      return this.activePlatformSpec().endpoint;
    }
    if (this.appMode() === 'providers') {
      return this.activeProviderSpec().endpoint;
    }
    return '/api/v1/screens/unified';
  });

  constructor() {
    void this.loadOfficeView();
  }

  activeModeLabel(): string {
    switch (this.appMode()) {
      case 'office':
        return 'Office Runtime';
      case 'platform':
        return 'Platform Runtime';
      case 'providers':
        return 'Provider Registry';
      default:
        return 'Unified Runtime';
    }
  }

  setProfile(profileId: string): void {
    this.activeProfile.set(profileId);
  }

  setNavQuery(value: string): void {
    this.navQuery.set(value);
  }

  setActionDraft(value: string): void {
    this.actionDraft.set(value);
  }

  setAppMode(mode: AppMode): void {
    this.appMode.set(mode);
    if (mode === 'office') {
      this.activeOfficeId.set('dashboard');
      void this.loadOfficeView();
      return;
    }
    if (mode === 'platform') {
      this.activePlatformId.set('system');
      void this.loadPlatformView();
      return;
    }
    if (mode === 'providers') {
      this.activeProviderId.set('snapshot');
      void this.loadProviderView();
      return;
    }
    const first = this.activeUnifiedItems()[0];
    this.activeUnifiedId.set(first?.id ?? '');
    void this.loadUnifiedCatalog();
  }

  setViewKind(kind: ViewKind): void {
    this.viewKind.set(kind);
    const first = this.activeUnifiedItems()[0];
    this.activeUnifiedId.set(first?.id ?? '');
  }

  isActiveNav(id: string): boolean {
    if (this.appMode() === 'office') {
      return id === this.activeOfficeId();
    }
    if (this.appMode() === 'platform') {
      return id === this.activePlatformId();
    }
    if (this.appMode() === 'providers') {
      return id === this.activeProviderId();
    }
    return id === this.activeUnifiedId();
  }

  selectNav(id: string): void {
    if (this.appMode() === 'office' && isOfficeViewId(id)) {
      this.activeOfficeId.set(id);
      void this.loadOfficeView();
      return;
    }
    if (this.appMode() === 'platform' && isPlatformViewId(id)) {
      this.activePlatformId.set(id);
      void this.loadPlatformView();
      return;
    }
    if (this.appMode() === 'providers' && isProviderViewId(id)) {
      this.activeProviderId.set(id);
      void this.loadProviderView();
      return;
    }
    this.activeUnifiedId.set(id);
  }

  displaySections(): string[] {
    if (this.appMode() === 'office') {
      return this.activeOfficeSpec().sections;
    }
    if (this.appMode() === 'platform') {
      return this.activePlatformSpec().sections;
    }
    if (this.appMode() === 'providers') {
      return this.activeProviderSpec().sections;
    }
    return this.activeUnifiedScreen()?.sections ?? [];
  }

  displayFlows(): string[] {
    if (this.appMode() === 'office') {
      return this.activeOfficeSpec().flows;
    }
    if (this.appMode() === 'platform') {
      return this.activePlatformSpec().flows;
    }
    if (this.appMode() === 'providers') {
      return this.activeProviderSpec().flows;
    }
    return this.activeUnifiedScreen()?.steps ?? [];
  }

  displayTags(): string[] {
    if (this.appMode() === 'office') {
      return this.activeOfficeSpec().integrations;
    }
    if (this.appMode() === 'platform') {
      return this.activePlatformSpec().integrations;
    }
    if (this.appMode() === 'providers') {
      return this.activeProviderSpec().integrations;
    }
    return this.activeUnifiedScreen()?.tags ?? [];
  }

  displayLinks(): string[] {
    if (this.appMode() === 'office') {
      return this.activeOfficeSpec().quickLinks;
    }
    if (this.appMode() === 'platform') {
      return this.activePlatformSpec().quickLinks;
    }
    if (this.appMode() === 'providers') {
      return this.activeProviderSpec().quickLinks;
    }
    return this.catalog().sources;
  }

  async refreshActive(): Promise<void> {
    if (this.appMode() === 'office') {
      await this.loadOfficeView();
      return;
    }
    if (this.appMode() === 'platform') {
      await this.loadPlatformView();
      return;
    }
    if (this.appMode() === 'providers') {
      await this.loadProviderView();
      return;
    }
    await this.loadUnifiedCatalog();
  }

  async loadUnifiedCatalog(): Promise<void> {
    try {
      const data = await firstValueFrom(this.api.unifiedScreens()) as UnifiedScreenCatalog;
      if (Array.isArray(data.modules) && Array.isArray(data.workflows)) {
        this.catalog.set(data);
        if (!this.activeUnifiedItems().some((x) => x.id === this.activeUnifiedId())) {
          this.activeUnifiedId.set(this.activeUnifiedItems()[0]?.id ?? '');
        }
        this.payload.set('Unified screen catalog synced from server.');
        return;
      }
      this.payload.set('Server catalog invalid format; using fallback catalog.');
    } catch (err) {
      this.payload.set(`catalog sync failed: ${String(err)} (fallback in use)`);
    }
  }

  async loadOfficeOverview(): Promise<void> {
    try {
      const data = await firstValueFrom(this.api.officeOverview());
      this.payload.set(JSON.stringify(data, null, 2));
    } catch (err) {
      this.payload.set(`request failed: ${String(err)}`);
    }
  }

  async loadOfficeView(): Promise<void> {
    try {
      const data = await this.fetchOfficeView(this.activeOfficeId());
      this.payload.set(JSON.stringify(data, null, 2));
    } catch (err) {
      this.payload.set(`office view request failed: ${String(err)}`);
    }
  }

  private async fetchOfficeView(viewId: OfficeViewId): Promise<unknown> {
    switch (viewId) {
      case 'dashboard':
        return firstValueFrom(this.api.officeDashboard());
      case 'portfolio':
        return firstValueFrom(this.api.officePortfolio());
      case 'timeline':
        return firstValueFrom(this.api.officeTimeline());
      case 'workspace':
        return firstValueFrom(this.api.officeWorkspace());
      case 'assistant':
        return firstValueFrom(this.api.officeAssistant());
      default:
        return firstValueFrom(this.api.officeOverview());
    }
  }

  async loadPlatformView(): Promise<void> {
    try {
      const data = await this.fetchPlatformView(this.activePlatformId());
      this.payload.set(JSON.stringify(data, null, 2));
    } catch (err) {
      this.payload.set(`platform view request failed: ${String(err)}`);
    }
  }

  private async fetchPlatformView(viewId: PlatformViewId): Promise<unknown> {
    switch (viewId) {
      case 'system':
        return firstValueFrom(this.api.systemSummary());
      case 'host':
        return firstValueFrom(this.api.hostSummary());
      case 'host_components':
        return firstValueFrom(this.api.hostComponents());
      case 'engine':
        return firstValueFrom(this.api.engineOverview());
      case 'engine_runtime':
        return firstValueFrom(this.api.engineRuntime());
      case 'database':
        return firstValueFrom(this.api.databaseRuntime());
      case 'modules':
        return firstValueFrom(this.api.modulesList());
      case 'autonomy':
        return firstValueFrom(this.api.autonomyCapabilities());
      default:
        return firstValueFrom(this.api.systemSummary());
    }
  }

  async loadProviderView(): Promise<void> {
    try {
      const data = await this.fetchProviderView(this.activeProviderId());
      this.payload.set(JSON.stringify(data, null, 2));
    } catch (err) {
      this.payload.set(`provider view request failed: ${String(err)}`);
    }
  }

  private async fetchProviderView(viewId: ProviderViewId): Promise<unknown> {
    switch (viewId) {
      case 'snapshot':
        return firstValueFrom(this.api.providersSnapshot());
      case 'platforms':
        return firstValueFrom(this.api.providersPlatforms());
      case 'providers':
        return firstValueFrom(this.api.providersList());
      case 'resources':
        return firstValueFrom(this.api.providersResources());
      case 'versions':
        return firstValueFrom(this.api.providersVersions());
      case 'metadata':
        return firstValueFrom(this.api.providersMetadata());
      case 'data':
        return firstValueFrom(this.api.providersDataAssets());
      case 'affiliates':
        return firstValueFrom(this.api.providersAffiliates());
      case 'affiliate_links':
        return firstValueFrom(this.api.providersAffiliateLinks());
      default:
        return firstValueFrom(this.api.providersSnapshot());
    }
  }

  async ackOfficeNotification(): Promise<void> {
    const notificationId = this.actionDraft().trim() || 'notif-001';
    try {
      const data = await firstValueFrom(this.api.officeAckNotification(notificationId));
      this.payload.set(JSON.stringify(data, null, 2));
      await this.loadOfficeView();
    } catch (err) {
      this.payload.set(`office action failed: ${String(err)}`);
    }
  }

  async createOfficePortfolioItem(): Promise<void> {
    const name = this.actionDraft().trim() || 'Office Generated Item';
    try {
      const data = await firstValueFrom(
        this.api.officeCreatePortfolioItem('project', name, 'active'),
      );
      this.payload.set(JSON.stringify(data, null, 2));
      this.activeOfficeId.set('portfolio');
      await this.loadOfficeView();
    } catch (err) {
      this.payload.set(`office action failed: ${String(err)}`);
    }
  }

  async createOfficeTimelineEvent(): Promise<void> {
    const title = this.actionDraft().trim() || 'Office Timeline Event';
    try {
      const data = await firstValueFrom(
        this.api.officeCreateTimelineEvent('cal-work', title, 'milestone', '2026-03-12T18:00:00Z'),
      );
      this.payload.set(JSON.stringify(data, null, 2));
      this.activeOfficeId.set('timeline');
      await this.loadOfficeView();
    } catch (err) {
      this.payload.set(`office action failed: ${String(err)}`);
    }
  }

  async createOfficeWorkspaceStory(): Promise<void> {
    const title = this.actionDraft().trim() || 'As a worker, I can execute office flows';
    try {
      const data = await firstValueFrom(this.api.officeCreateWorkspaceStory(title, 5));
      this.payload.set(JSON.stringify(data, null, 2));
      this.activeOfficeId.set('workspace');
      await this.loadOfficeView();
    } catch (err) {
      this.payload.set(`office action failed: ${String(err)}`);
    }
  }

  async subscribeOfficeAssistant(): Promise<void> {
    const topic = this.actionDraft().trim() || 'office.dashboard.alerts';
    try {
      const data = await firstValueFrom(this.api.officeSubscribeAssistant(topic));
      this.payload.set(JSON.stringify(data, null, 2));
      this.activeOfficeId.set('assistant');
      await this.loadOfficeView();
    } catch (err) {
      this.payload.set(`office action failed: ${String(err)}`);
    }
  }

  async engineControl(): Promise<void> {
    const action = this.actionDraft().trim() || 'start';
    try {
      const data = await firstValueFrom(this.api.engineControl(action));
      this.payload.set(JSON.stringify(data, null, 2));
    } catch (err) {
      this.payload.set(`engine control failed: ${String(err)}`);
    }
  }

  async engineIngest(): Promise<void> {
    const raw = this.actionDraft().trim();
    let payload: unknown = { source: 'web', note: raw || 'manual ingest' };
    if (raw.startsWith('{') || raw.startsWith('[')) {
      try {
        payload = JSON.parse(raw);
      } catch {
        payload = { source: 'web', note: raw };
      }
    }
    try {
      const data = await firstValueFrom(this.api.engineIngest(payload));
      this.payload.set(JSON.stringify(data, null, 2));
    } catch (err) {
      this.payload.set(`engine ingest failed: ${String(err)}`);
    }
  }

  async databaseQuery(): Promise<void> {
    const sql = this.actionDraft().trim() || 'select 1';
    try {
      const data = await firstValueFrom(this.api.databaseQuery(sql));
      this.payload.set(JSON.stringify(data, null, 2));
    } catch (err) {
      this.payload.set(`database query failed: ${String(err)}`);
    }
  }

  async createProviderPlatform(): Promise<void> {
    const parts = this.parseDraft();
    const name = parts[0] ?? 'New Platform';
    const kind = parts[1] ?? 'platform';
    const category = parts[2] ?? 'general';
    try {
      const data = await firstValueFrom(this.api.providersCreatePlatform(name, kind, category, 'active'));
      this.payload.set(JSON.stringify(data, null, 2));
      await this.loadProviderView();
    } catch (err) {
      this.payload.set(`provider platform create failed: ${String(err)}`);
    }
  }

  async createProvider(): Promise<void> {
    const parts = this.parseDraft();
    const name = parts[0] ?? 'New Provider';
    const platformId = parts[1] ?? 'platform-001';
    const kind = parts[2] ?? 'api';
    try {
      const data = await firstValueFrom(
        this.api.providersCreateProvider(name, platformId, kind, 'active'),
      );
      this.payload.set(JSON.stringify(data, null, 2));
      await this.loadProviderView();
    } catch (err) {
      this.payload.set(`provider create failed: ${String(err)}`);
    }
  }

  async addProviderResource(): Promise<void> {
    const parts = this.parseDraft();
    const providerId = parts[0] ?? 'provider-001';
    const resourceType = parts[1] ?? 'api';
    const name = parts[2] ?? 'primary-resource';
    try {
      const data = await firstValueFrom(
        this.api.providersAddResource(providerId, resourceType, name, 'active'),
      );
      this.payload.set(JSON.stringify(data, null, 2));
      await this.loadProviderView();
    } catch (err) {
      this.payload.set(`provider resource failed: ${String(err)}`);
    }
  }

  async addProviderVersion(): Promise<void> {
    const parts = this.parseDraft();
    const providerId = parts[0] ?? 'provider-001';
    const version = parts[1] ?? 'v1';
    const status = parts[2] ?? 'stable';
    try {
      const data = await firstValueFrom(
        this.api.providersAddVersion(providerId, version, status),
      );
      this.payload.set(JSON.stringify(data, null, 2));
      await this.loadProviderView();
    } catch (err) {
      this.payload.set(`provider version failed: ${String(err)}`);
    }
  }

  async setProviderMetadata(): Promise<void> {
    const parts = this.parseDraft();
    const providerId = parts[0] ?? 'provider-001';
    const key = parts[1] ?? 'region';
    const value = parts[2] ?? 'us';
    try {
      const data = await firstValueFrom(
        this.api.providersSetMetadata(providerId, key, value, 'general'),
      );
      this.payload.set(JSON.stringify(data, null, 2));
      await this.loadProviderView();
    } catch (err) {
      this.payload.set(`provider metadata failed: ${String(err)}`);
    }
  }

  async addProviderDataAsset(): Promise<void> {
    const parts = this.parseDraft();
    const providerId = parts[0] ?? 'provider-001';
    const dataset = parts[1] ?? 'dataset';
    const status = parts[2] ?? 'active';
    const recordCount = parts[3] ? Number(parts[3]) : 0;
    try {
      const data = await firstValueFrom(
        this.api.providersAddDataAsset(providerId, dataset, status, Number.isNaN(recordCount) ? 0 : recordCount),
      );
      this.payload.set(JSON.stringify(data, null, 2));
      await this.loadProviderView();
    } catch (err) {
      this.payload.set(`provider data asset failed: ${String(err)}`);
    }
  }

  async registerAffiliate(): Promise<void> {
    const parts = this.parseDraft();
    const name = parts[0] ?? 'New Affiliate';
    const kind = parts[1] ?? 'partner';
    const status = parts[2] ?? 'active';
    try {
      const data = await firstValueFrom(
        this.api.providersRegisterAffiliate(name, kind, status),
      );
      this.payload.set(JSON.stringify(data, null, 2));
      await this.loadProviderView();
    } catch (err) {
      this.payload.set(`affiliate register failed: ${String(err)}`);
    }
  }

  async addAffiliateLink(): Promise<void> {
    const parts = this.parseDraft();
    const providerId = parts[0] ?? 'provider-001';
    const affiliateId = parts[1] ?? 'affiliate-001';
    const status = parts[2] ?? 'active';
    try {
      const data = await firstValueFrom(
        this.api.providersAddAffiliateLink(providerId, affiliateId, status),
      );
      this.payload.set(JSON.stringify(data, null, 2));
      await this.loadProviderView();
    } catch (err) {
      this.payload.set(`affiliate link failed: ${String(err)}`);
    }
  }

  async loadSystem(): Promise<void> {
    try {
      const data = await firstValueFrom(this.api.systemSummary());
      this.payload.set(JSON.stringify(data, null, 2));
    } catch (err) {
      this.payload.set(`request failed: ${String(err)}`);
    }
  }

  async loadHostSummary(): Promise<void> {
    try {
      const data = await firstValueFrom(this.api.hostSummary());
      this.payload.set(JSON.stringify(data, null, 2));
    } catch (err) {
      this.payload.set(`request failed: ${String(err)}`);
    }
  }

  async loadHostComponents(): Promise<void> {
    try {
      const data = await firstValueFrom(this.api.hostComponents());
      this.payload.set(JSON.stringify(data, null, 2));
    } catch (err) {
      this.payload.set(`request failed: ${String(err)}`);
    }
  }

  async loadEngineOverview(): Promise<void> {
    try {
      const data = await firstValueFrom(this.api.engineOverview());
      this.payload.set(JSON.stringify(data, null, 2));
    } catch (err) {
      this.payload.set(`request failed: ${String(err)}`);
    }
  }

  async loadEngineRuntime(): Promise<void> {
    try {
      const data = await firstValueFrom(this.api.engineRuntime());
      this.payload.set(JSON.stringify(data, null, 2));
    } catch (err) {
      this.payload.set(`request failed: ${String(err)}`);
    }
  }

  async loadDatabaseRuntime(): Promise<void> {
    try {
      const data = await firstValueFrom(this.api.databaseRuntime());
      this.payload.set(JSON.stringify(data, null, 2));
    } catch (err) {
      this.payload.set(`request failed: ${String(err)}`);
    }
  }

  async loadModulesList(): Promise<void> {
    try {
      const data = await firstValueFrom(this.api.modulesList());
      this.payload.set(JSON.stringify(data, null, 2));
    } catch (err) {
      this.payload.set(`request failed: ${String(err)}`);
    }
  }

  async loadProvidersSnapshot(): Promise<void> {
    try {
      const data = await firstValueFrom(this.api.providersSnapshot());
      this.payload.set(JSON.stringify(data, null, 2));
    } catch (err) {
      this.payload.set(`request failed: ${String(err)}`);
    }
  }

  async loadProvidersAffiliates(): Promise<void> {
    try {
      const data = await firstValueFrom(this.api.providersAffiliates());
      this.payload.set(JSON.stringify(data, null, 2));
    } catch (err) {
      this.payload.set(`request failed: ${String(err)}`);
    }
  }

  async loadIdentities(): Promise<void> {
    try {
      const data = await firstValueFrom(this.api.identities());
      this.payload.set(JSON.stringify(data, null, 2));
    } catch (err) {
      this.payload.set(`request failed: ${String(err)}`);
    }
  }

  async loadProfiles(): Promise<void> {
    try {
      const data = await firstValueFrom(this.api.profiles());
      this.payload.set(JSON.stringify(data, null, 2));
    } catch (err) {
      this.payload.set(`request failed: ${String(err)}`);
    }
  }

  async loadIsolation(): Promise<void> {
    try {
      const data = await firstValueFrom(this.api.moduleIsolation());
      this.payload.set(JSON.stringify(data, null, 2));
    } catch (err) {
      this.payload.set(`request failed: ${String(err)}`);
    }
  }

  private parseDraft(): string[] {
    return this.actionDraft()
      .split('|')
      .map((part) => part.trim())
      .filter((part) => part.length > 0);
  }
}

bootstrapApplication(AppComponent).catch((err) => console.error(err));
