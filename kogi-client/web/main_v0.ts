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
    <main class="app-root">
      <header class="topbar">
        <div class="brand">KOGI<span>OS</span></div>
        <label class="search">
          <input
            [value]="navQuery()"
            (input)="setNavQuery(($any($event.target)).value)"
            placeholder="Search modules, flows, integrations" />
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
          <button class="rail-btn active">◎</button>
          <button class="rail-btn">▦</button>
          <button class="rail-btn">↺</button>
          <button class="rail-btn">⚙</button>
        </aside>

        <aside class="navigator">
          <div class="mode-row">
            <button class="mode-btn" [class.active]="appMode() === 'office'" (click)="setAppMode('office')">Office</button>
            <button class="mode-btn" [class.active]="appMode() === 'unified'" (click)="setAppMode('unified')">Unified</button>
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
              <span>{{ appMode() === 'office' ? 'Office Runtime' : 'Unified Runtime' }}</span>
            </div>
          </article>

          <section class="meta-row">
            <article class="metric">
              <span>Series</span>
              <strong>{{ catalog().series }}</strong>
            </article>
            <article class="metric">
              <span>Version</span>
              <strong>{{ catalog().version }}</strong>
            </article>
            <article class="metric">
              <span>Visible Views</span>
              <strong>{{ navItems().length }}</strong>
            </article>
            <article class="metric">
              <span>Active Profile</span>
              <strong>{{ activeProfile() }}</strong>
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
              <h2>{{ appMode() === 'office' ? 'Integrations' : 'Tags' }}</h2>
              <div class="chips">
                <span class="chip chip-alt" *ngFor="let tag of displayTags()">{{ tag }}</span>
              </div>
            </article>

            <article class="panel">
              <h2>{{ appMode() === 'office' ? 'Quick Links' : 'Source Files' }}</h2>
              <ul>
                <li *ngFor="let link of displayLinks()">{{ link }}</li>
              </ul>
            </article>
          </section>

          <section class="tool-row">
            <button (click)="loadSystem()">System</button>
            <button (click)="loadIdentities()">IMS Identities</button>
            <button (click)="loadProfiles()">IMS Profiles</button>
            <button (click)="loadIsolation()">Module Isolation</button>
            <button (click)="loadOfficeOverview()">Office Overview</button>
          </section>

          <section class="office-actions" *ngIf="appMode() === 'office'">
            <input
              [value]="officeActionDraft()"
              (input)="setOfficeActionDraft(($any($event.target)).value)"
              placeholder="Action input (id, name, topic)" />
            <button (click)="ackOfficeNotification()">Ack</button>
            <button (click)="createOfficePortfolioItem()">Portfolio+</button>
            <button (click)="createOfficeTimelineEvent()">Timeline+</button>
            <button (click)="createOfficeWorkspaceStory()">Story+</button>
            <button (click)="subscribeOfficeAssistant()">Subscribe+</button>
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
      font-family: "Manrope", "Segoe UI", sans-serif;
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
    .tool-row, .office-actions {
      display: flex;
      gap: 8px;
      flex-wrap: wrap;
    }
    .tool-row button, .office-actions button {
      border: 1px solid rgba(112, 146, 240, .32);
      border-radius: 10px;
      background: rgba(17, 37, 82, .8);
      color: #c3ddff;
      padding: 8px 11px;
      cursor: pointer;
    }
    .office-actions input {
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
  readonly activeProfile = signal('work');
  readonly payload = signal('Select a view from the left sidebar.');
  readonly officeActionDraft = signal('');
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

  readonly navItems = computed<NavItem[]>(() => {
    const raw = this.appMode() === 'office'
      ? officeViewSpecs.map((x) => ({ id: x.id, title: x.title }))
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

  setNavQuery(value: string): void {
    this.navQuery.set(value);
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
