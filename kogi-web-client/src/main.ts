import { bootstrapApplication } from '@angular/platform-browser';
import { CommonModule } from '@angular/common';
import { Component, computed, inject, signal } from '@angular/core';
import { HttpClientModule } from '@angular/common/http';
import { firstValueFrom } from 'rxjs';
import { ApiService } from './app/core/api.service';

type ViewKind = 'module' | 'workflow';
type AppMode = 'unified' | 'office';
type OfficeViewId = 'dashboard' | 'portfolio' | 'timeline' | 'workspace' | 'assistant';

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

interface OfficeViewSpec {
  id: OfficeViewId;
  title: string;
  subtitle: string;
  sections: string[];
  flows: string[];
  integrations: string[];
  quickLinks: string[];
  endpoint: string;
}

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
    { id: 'dashboard', title: 'Dashboard', tags: ['overview', 'activity', 'ai'], sections: ['Portfolio Health', 'Quick Access Modules', 'Recent Activity'] },
    { id: 'office', title: 'Office', tags: ['projects', 'programs', 'portfolio'], sections: ['Programs and Projects', 'Milestones Due', 'Team Capacity', '3rd Party Integrations'] },
    { id: 'workspace', title: 'Workspace', tags: ['tasks', 'kanban', 'sprints'], sections: ['Kanban Board', 'Calendar', 'Gantt Timeline'] },
    { id: 'timeline', title: 'Timeline', tags: ['calendar', 'roadmap', 'gantt'], sections: ['Master Timeline', 'Scheduled Events', 'Deadlines'] },
    { id: 'portfolio', title: 'Portfolio', tags: ['assets', 'solutions', 'artifacts'], sections: ['Portfolio Grid', 'Linked Platforms'] },
    { id: 'strategy', title: 'Strategy', tags: ['strategy', 'tactics', 'governance'], sections: ['Strategic OKRs', 'Tactical Initiatives', 'Governance'] },
    { id: 'studio', title: 'Studio', tags: ['ideas', 'prototypes', 'tools'], sections: ['Ideas Grid', 'Testbeds', 'Toolsets'] },
    { id: 'community', title: 'Community', tags: ['feeds', 'spaces', 'messages'], sections: ['Feeds and Timelines', 'Spaces and Rooms', 'Direct Messages'] },
    { id: 'developer', title: 'Developer', tags: ['api', 'sdk', 'integrations'], sections: ['API Reference', 'Webhooks', 'Extensions'] },
    { id: 'profile', title: 'Profile', tags: ['personas', 'settings', 'config'], sections: ['Personas and Roles', 'Settings and Config', 'Activity Stats'] },
    { id: 'organizations', title: 'Organizations', tags: ['coops', 'collectives', 'teams'], sections: ['Organizations Grid', 'Governance and Proposals', 'Cap Tables'] },
    { id: 'legal', title: 'Legal', tags: ['ip', 'contracts', 'compliance'], sections: ['IP and Trademarks', 'Contracts', 'Compliance and Audit'] },
    { id: 'marketplace', title: 'Marketplace', tags: ['buy', 'sell', 'barter'], sections: ['Marketplace Grid', 'Barter System', 'My Orders'] },
    { id: 'bank', title: 'Bank', tags: ['wallets', 'finance', 'fundraising'], sections: ['Wallet Types', 'Fundraising and Capital', 'Tax Summary'] },
    { id: 'exchange', title: 'Exchange', tags: ['bids', 'deals', 'due-diligence'], sections: ['Bids and Offers', 'Deal Pipeline', 'Requests'] },
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

function isOfficeViewId(value: string): value is OfficeViewId {
  return officeViewSpecs.some((x) => x.id === value);
}

@Component({
  selector: 'kogi-root',
  standalone: true,
  imports: [CommonModule, HttpClientModule],
  providers: [ApiService],
  template: `
    <main class="app-shell">
      <aside class="sidebar">
        <div class="brand">KOGI</div>

        <label class="field-label">Identity Profile</label>
        <select [value]="activeProfile()" (change)="setProfile(($any($event.target)).value)">
          <option *ngFor="let profile of identityProfiles" [value]="profile.id">{{ profile.name }}</option>
        </select>

        <div class="sidebar-section">
          <button class="seg" [class.active]="appMode() === 'office'" (click)="setAppMode('office')">Office App</button>
          <button class="seg" [class.active]="appMode() === 'unified'" (click)="setAppMode('unified')">Unified Series</button>
        </div>

        <div class="sidebar-section" *ngIf="appMode() === 'unified'">
          <button class="seg" [class.active]="viewKind() === 'module'" (click)="setViewKind('module')">Module Views</button>
          <button class="seg" [class.active]="viewKind() === 'workflow'" (click)="setViewKind('workflow')">Workflow Views</button>
        </div>

        <div class="nav-list">
          <button
            *ngFor="let item of navItems()"
            class="nav-item"
            [class.active]="isActiveNav(item.id)"
            (click)="selectNav(item.id)">
            {{ item.title }}
          </button>
        </div>
      </aside>

      <section class="content">
        <header class="content-header">
          <div>
            <h1>{{ activeTitle() }}</h1>
            <p>{{ activeSubtitle() }}</p>
          </div>
          <button class="refresh" (click)="refreshActive()">Refresh</button>
        </header>

        <section class="meta-row">
          <article class="metric">
            <span>Mode</span>
            <strong>{{ appMode() === 'office' ? 'Office App' : 'Unified' }}</strong>
          </article>
          <article class="metric">
            <span>Series</span>
            <strong>{{ catalog().series }}</strong>
          </article>
          <article class="metric">
            <span>Version</span>
            <strong>{{ catalog().version }}</strong>
          </article>
          <article class="metric">
            <span>Total Views</span>
            <strong>{{ navItems().length }}</strong>
          </article>
        </section>

        <section class="panel-grid">
          <article class="panel">
            <h2>{{ appMode() === 'office' ? 'Office Sections' : 'Sections' }}</h2>
            <div class="chips">
              <span class="chip" *ngFor="let section of displaySections()">{{ section }}</span>
            </div>
          </article>

          <article class="panel">
            <h2>{{ appMode() === 'office' ? 'Core Flows' : 'Workflow Steps' }}</h2>
            <ol>
              <li *ngFor="let step of displayFlows()">{{ step }}</li>
            </ol>
          </article>

          <article class="panel">
            <h2>{{ appMode() === 'office' ? 'Integrations' : 'Tags' }}</h2>
            <div class="chips">
              <span class="chip secondary" *ngFor="let tag of displayTags()">{{ tag }}</span>
            </div>
          </article>

          <article class="panel">
            <h2>{{ appMode() === 'office' ? 'Quick Links' : 'Source Files' }}</h2>
            <ul>
              <li *ngFor="let link of displayLinks()">{{ link }}</li>
            </ul>
          </article>
        </section>

        <section class="data-tools">
          <button (click)="loadSystem()">System</button>
          <button (click)="loadIdentities()">IMS Identities</button>
          <button (click)="loadProfiles()">IMS Profiles</button>
          <button (click)="loadIsolation()">Module Isolation</button>
          <button (click)="loadOfficeOverview()">Office Overview</button>
          <button (click)="refreshActive()">Active View Data</button>
        </section>

        <section class="office-actions" *ngIf="appMode() === 'office'">
          <input
            [value]="officeActionDraft()"
            (input)="setOfficeActionDraft(($any($event.target)).value)"
            placeholder="Action input (id, name, or topic)" />
          <button (click)="ackOfficeNotification()">Ack Notification</button>
          <button (click)="createOfficePortfolioItem()">Add Portfolio Item</button>
          <button (click)="createOfficeTimelineEvent()">Add Timeline Event</button>
          <button (click)="createOfficeWorkspaceStory()">Add Workspace Story</button>
          <button (click)="subscribeOfficeAssistant()">Subscribe Assistant</button>
        </section>

        <pre>{{ payload() }}</pre>
      </section>
    </main>
  `,
  styles: [
    `
    .app-shell { display: grid; grid-template-columns: 260px 1fr; min-height: 100vh; background: #070b1d; color: #dce4ff; }
    .sidebar { border-right: 1px solid #1a2345; padding: 14px; background: #0a0f28; display: flex; flex-direction: column; gap: 12px; }
    .brand { font-size: 1.6rem; font-weight: 800; letter-spacing: .2rem; color: #7fa5ff; }
    .field-label { font-size: .76rem; text-transform: uppercase; opacity: .7; }
    select { background: #101735; color: #dce4ff; border: 1px solid #203061; border-radius: 8px; padding: 8px; }
    .sidebar-section { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
    .seg { background: #121937; color: #89a2ff; border: 1px solid #283a72; border-radius: 8px; padding: 8px; cursor: pointer; }
    .seg.active { background: #1b2550; color: #fff; }
    .nav-list { display: flex; flex-direction: column; gap: 6px; overflow: auto; max-height: calc(100vh - 220px); }
    .nav-item { text-align: left; background: #0f1632; color: #a8bcff; border: 1px solid #233569; border-radius: 8px; padding: 9px; cursor: pointer; }
    .nav-item.active { background: linear-gradient(90deg, #1f2d63, #203a6a); color: #fff; }

    .content { padding: 18px; display: flex; flex-direction: column; gap: 14px; }
    .content-header { display: flex; justify-content: space-between; gap: 12px; align-items: start; }
    h1 { margin: 0; font-size: 2rem; font-family: "Alegreya", Georgia, serif; }
    .content-header p { margin: 4px 0 0; opacity: .8; }
    .refresh { background: #1b8cff; border: 0; color: #fff; border-radius: 8px; padding: 9px 12px; cursor: pointer; }

    .meta-row { display: grid; gap: 10px; grid-template-columns: repeat(auto-fit, minmax(160px, 1fr)); }
    .metric { background: #101937; border: 1px solid #223564; border-radius: 12px; padding: 10px; }
    .metric span { display: block; font-size: .73rem; opacity: .7; }
    .metric strong { font-size: 1.1rem; color: #5de0ff; }

    .panel-grid { display: grid; gap: 12px; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); }
    .panel { background: #0d1430; border: 1px solid #203263; border-radius: 12px; padding: 12px; }
    .panel h2 { margin-top: 0; font-size: 1rem; }
    .chips { display: flex; flex-wrap: wrap; gap: 6px; }
    .chip { background: #173058; color: #8bc6ff; border: 1px solid #2b558f; border-radius: 999px; padding: 4px 10px; font-size: .8rem; }
    .chip.secondary { background: #2c214f; color: #c9a9ff; border-color: #5d44a3; }
    ol, ul { margin: 0; padding-left: 18px; }

    .data-tools { display: flex; flex-wrap: wrap; gap: 8px; }
    .data-tools button { background: #121b3d; border: 1px solid #2a3d72; color: #a8bdff; border-radius: 8px; padding: 7px 11px; cursor: pointer; }
    .office-actions { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; }
    .office-actions input { min-width: 260px; background: #101735; color: #dce4ff; border: 1px solid #2a3d72; border-radius: 8px; padding: 8px; }
    .office-actions button { background: #152149; border: 1px solid #314782; color: #bfd1ff; border-radius: 8px; padding: 8px 11px; cursor: pointer; }
    pre { margin: 0; background: #070e25; border: 1px solid #1d2d59; color: #d4e2ff; border-radius: 10px; padding: 12px; min-height: 120px; white-space: pre-wrap; }

    @media (max-width: 980px) {
      .app-shell { grid-template-columns: 1fr; }
      .sidebar { border-right: 0; border-bottom: 1px solid #1a2345; }
      .nav-list { max-height: 220px; }
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
  readonly activeProfile = signal('work');
  readonly payload = signal('Select a view from the left sidebar.');
  readonly officeActionDraft = signal('');

  readonly activeUnifiedItems = computed(() =>
    this.viewKind() === 'module' ? this.catalog().modules : this.catalog().workflows,
  );

  readonly activeUnifiedScreen = computed(() =>
    this.activeUnifiedItems().find((screen) => screen.id === this.activeUnifiedId()),
  );

  readonly activeOfficeSpec = computed(() =>
    officeViewSpecs.find((x) => x.id === this.activeOfficeId()) ?? officeViewSpecs[0],
  );

  readonly navItems = computed<NavItem[]>(() => {
    if (this.appMode() === 'office') {
      return officeViewSpecs.map((x) => ({ id: x.id, title: x.title }));
    }
    return this.activeUnifiedItems().map((x) => ({ id: x.id, title: x.title }));
  });

  readonly activeTitle = computed(() => {
    if (this.appMode() === 'office') {
      return this.activeOfficeSpec().title;
    }
    return this.activeUnifiedScreen()?.title ?? 'Kogi';
  });

  readonly activeSubtitle = computed(() => {
    if (this.appMode() === 'office') {
      return this.activeOfficeSpec().subtitle;
    }
    return this.viewKind() === 'module'
      ? 'Unified module screen from reconciled v2/v3 docs.'
      : 'Unified workflow screen from reconciled v2/v3 docs.';
  });

  constructor() {
    void this.loadOfficeView();
  }

  setProfile(profileId: string): void {
    this.activeProfile.set(profileId);
  }

  setOfficeActionDraft(value: string): void {
    this.officeActionDraft.set(value);
  }

  setAppMode(mode: AppMode): void {
    this.appMode.set(mode);
    if (mode === 'office') {
      this.activeOfficeId.set('dashboard');
      void this.loadOfficeView();
      return;
    }
    const first = this.activeUnifiedItems()[0];
    this.activeUnifiedId.set(first?.id ?? '');
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
    return id === this.activeUnifiedId();
  }

  selectNav(id: string): void {
    if (this.appMode() === 'office' && isOfficeViewId(id)) {
      this.activeOfficeId.set(id);
      void this.loadOfficeView();
      return;
    }
    this.activeUnifiedId.set(id);
  }

  displaySections(): string[] {
    if (this.appMode() === 'office') {
      return this.activeOfficeSpec().sections;
    }
    return this.activeUnifiedScreen()?.sections ?? [];
  }

  displayFlows(): string[] {
    if (this.appMode() === 'office') {
      return this.activeOfficeSpec().flows;
    }
    return this.activeUnifiedScreen()?.steps ?? [];
  }

  displayTags(): string[] {
    if (this.appMode() === 'office') {
      return this.activeOfficeSpec().integrations;
    }
    return this.activeUnifiedScreen()?.tags ?? [];
  }

  displayLinks(): string[] {
    if (this.appMode() === 'office') {
      return this.activeOfficeSpec().quickLinks;
    }
    return this.catalog().sources;
  }

  async refreshActive(): Promise<void> {
    if (this.appMode() === 'office') {
      await this.loadOfficeView();
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

  async ackOfficeNotification(): Promise<void> {
    const notificationId = this.officeActionDraft().trim() || 'notif-001';
    try {
      const data = await firstValueFrom(this.api.officeAckNotification(notificationId));
      this.payload.set(JSON.stringify(data, null, 2));
      await this.loadOfficeView();
    } catch (err) {
      this.payload.set(`office action failed: ${String(err)}`);
    }
  }

  async createOfficePortfolioItem(): Promise<void> {
    const name = this.officeActionDraft().trim() || 'Office Generated Item';
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
    const title = this.officeActionDraft().trim() || 'Office Timeline Event';
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
    const title = this.officeActionDraft().trim() || 'As a worker, I can execute office flows';
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
    const topic = this.officeActionDraft().trim() || 'office.dashboard.alerts';
    try {
      const data = await firstValueFrom(this.api.officeSubscribeAssistant(topic));
      this.payload.set(JSON.stringify(data, null, 2));
      this.activeOfficeId.set('assistant');
      await this.loadOfficeView();
    } catch (err) {
      this.payload.set(`office action failed: ${String(err)}`);
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
}

bootstrapApplication(AppComponent).catch((err) => console.error(err));
