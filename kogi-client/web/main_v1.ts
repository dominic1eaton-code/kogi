import { bootstrapApplication } from '@angular/platform-browser';
import { CommonModule } from '@angular/common';
import { Component, computed, signal } from '@angular/core';

type ScreenId =
  | 'dashboard'
  | 'office'
  | 'portfolio'
  | 'workspace'
  | 'timeline'
  | 'strategy'
  | 'bank'
  | 'exchange'
  | 'marketplace'
  | 'studio'
  | 'community'
  | 'developer'
  | 'profile'
  | 'organizations'
  | 'legal';

type Trend = 'up' | 'down' | 'flat';

type Tone =
  | 'tone-blue'
  | 'tone-indigo'
  | 'tone-teal'
  | 'tone-emerald'
  | 'tone-amber'
  | 'tone-rose'
  | 'tone-purple'
  | 'tone-cyan'
  | 'tone-slate';

interface NavItem {
  id: ScreenId;
  label: string;
}

interface ScreenMeta {
  title: string;
  subtitle: string;
  tabs: string[];
}

interface Metric {
  label: string;
  value: string;
  change?: string;
  trend?: Trend;
  caption?: string;
  progress?: number;
  tone: Tone;
}

interface ActivityItem {
  event: string;
  module: string;
  time: string;
  status: string;
  tone: Tone;
}

interface CardItem {
  title: string;
  subtitle?: string;
  meta?: string;
  status?: string;
  tone: Tone;
  progress?: number;
  value?: string;
  members?: string[];
}

interface TableRow {
  cells: string[];
  tone?: Tone;
  badge?: string;
  badgeTone?: Tone;
}

interface ChipItem {
  label: string;
  tone: Tone;
}

interface FeedItem {
  author: string;
  message: string;
  time: string;
  stats: string;
}

interface KanbanColumn {
  title: string;
  tone: Tone;
  items: string[];
}

interface TimelineBar {
  label: string;
  tone: Tone;
  start: number;
  width: number;
}

interface GanttRow {
  label: string;
  bars: TimelineBar[];
}

interface ApiRow {
  method: string;
  path: string;
  description: string;
  tone: Tone;
}

const NAV_ITEMS: NavItem[] = [
  { id: 'dashboard', label: 'Dashboard' },
  { id: 'office', label: 'Office' },
  { id: 'portfolio', label: 'Portfolio' },
  { id: 'workspace', label: 'Workspace' },
  { id: 'timeline', label: 'Timeline' },
  { id: 'strategy', label: 'Strategy' },
  { id: 'bank', label: 'Bank' },
  { id: 'exchange', label: 'Exchange' },
  { id: 'marketplace', label: 'Marketplace' },
  { id: 'studio', label: 'Studio' },
  { id: 'community', label: 'Community' },
  { id: 'developer', label: 'Developer' },
  { id: 'profile', label: 'Profile' },
  { id: 'organizations', label: 'Organizations' },
  { id: 'legal', label: 'Legal' },
];

const SCREEN_META: Record<ScreenId, ScreenMeta> = {
  dashboard: {
    title: 'Dashboard',
    subtitle: 'Overview',
    tabs: ['Overview', 'Activity', 'AI'],
  },
  office: {
    title: 'Office',
    subtitle: 'Projects - Programs - Portfolio',
    tabs: ['Projects', 'Programs', 'Portfolio'],
  },
  portfolio: {
    title: 'Portfolio',
    subtitle: 'Assets - Solutions - Artifacts',
    tabs: ['Assets', 'Solutions', 'Artifacts'],
  },
  workspace: {
    title: 'Workspace',
    subtitle: 'Tasks - Kanban - Sprints',
    tabs: ['Tasks', 'Kanban', 'Sprints'],
  },
  timeline: {
    title: 'Timeline',
    subtitle: 'Calendar - Roadmap - Gantt',
    tabs: ['Calendar', 'Roadmap', 'Gantt'],
  },
  strategy: {
    title: 'Strategy',
    subtitle: 'Strategy - Tactics - Governance',
    tabs: ['Strategy', 'Tactics', 'Governance'],
  },
  bank: {
    title: 'Bank',
    subtitle: 'Wallets - Finance - Fundraising',
    tabs: ['Wallets', 'Finance', 'Fundraising'],
  },
  exchange: {
    title: 'Exchange',
    subtitle: 'Bids - Deals - Due Diligence',
    tabs: ['Bids', 'Deals', 'Due Diligence'],
  },
  marketplace: {
    title: 'Marketplace',
    subtitle: 'Buy - Sell - Barter',
    tabs: ['Buy', 'Sell', 'Barter'],
  },
  studio: {
    title: 'Studio',
    subtitle: 'Ideas - Prototypes - Tools',
    tabs: ['Ideas', 'Prototypes', 'Tools'],
  },
  community: {
    title: 'Community',
    subtitle: 'Feeds - Spaces - Messages',
    tabs: ['Feeds', 'Spaces', 'Messages'],
  },
  developer: {
    title: 'Developer',
    subtitle: 'API - SDK - Integrations',
    tabs: ['API', 'SDK', 'Integrations'],
  },
  profile: {
    title: 'Profile',
    subtitle: 'Personas - Settings - Config',
    tabs: ['Personas', 'Settings', 'Config'],
  },
  organizations: {
    title: 'Organizations',
    subtitle: 'Coops - Collectives - Teams',
    tabs: ['Coops', 'Collectives', 'Teams'],
  },
  legal: {
    title: 'Legal',
    subtitle: 'IP - Contracts - Compliance',
    tabs: ['IP', 'Contracts', 'Compliance'],
  },
};

@Component({
  selector: 'kogi-root',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="app-shell">
      <aside class="sidebar">
        <div class="brand">KOGI</div>
        <nav class="nav">
          <button
            *ngFor="let item of navItems"
            type="button"
            class="nav-item"
            [class.active]="activeScreen() === item.id"
            (click)="setScreen(item.id)"
          >
            <span class="nav-indicator"></span>
            <span>{{ item.label }}</span>
          </button>
        </nav>
        <div class="sidebar-footer">
          <div class="card sidebar-card">
            <div class="card-title">System Status</div>
            <div class="sidebar-metric">All modules healthy</div>
            <div class="chip-row">
              <span class="chip tone-emerald">Online</span>
              <span class="chip tone-blue">Sync OK</span>
            </div>
          </div>
        </div>
      </aside>

      <main class="main">
        <header class="topbar">
          <div class="topbar-title">
            <div class="eyebrow">{{ meta().subtitle }}</div>
            <h1>{{ meta().title }}</h1>
            <div class="tabs">
              <button
                *ngFor="let tab of meta().tabs; let i = index"
                class="tab"
                [class.active]="i === 0"
                type="button"
              >
                {{ tab }}
              </button>
            </div>
          </div>
          <div class="topbar-actions">
            <div class="search">
              <input type="text" placeholder="Search..." />
            </div>
            <button class="pill" type="button">PRO</button>
            <button class="avatar" type="button">J</button>
          </div>
        </header>

        <section class="content">
          <ng-container [ngSwitch]="activeScreen()">
            <section *ngSwitchCase="'dashboard'" class="screen">
              <div class="stats-row">
                <div
                  class="card metric-card"
                  *ngFor="let stat of dashboardMetrics"
                  [ngClass]="stat.tone"
                >
                  <div class="metric-label">{{ stat.label }}</div>
                  <div class="metric-value">{{ stat.value }}</div>
                  <div class="metric-meta">
                    <span
                      *ngIf="stat.change"
                      class="trend"
                      [class.up]="stat.trend === 'up'"
                      [class.down]="stat.trend === 'down'"
                      [class.flat]="stat.trend === 'flat'"
                    >
                      {{ stat.change }}
                    </span>
                    <span class="metric-caption">{{ stat.caption }}</span>
                  </div>
                  <div class="progress" *ngIf="stat.progress !== undefined">
                    <span [style.width.%]="stat.progress"></span>
                  </div>
                </div>
              </div>

              <div class="grid-2">
                <div class="card">
                  <div class="card-title">Quick Access - Modules</div>
                  <div class="quick-grid">
                    <button
                      *ngFor="let item of dashboardQuickLinks"
                      class="quick-card"
                      type="button"
                      [ngClass]="item.tone"
                    >
                      <span class="quick-label">{{ item.label }}</span>
                    </button>
                  </div>
                </div>

                <div class="card">
                  <div class="card-title">Recent Activity</div>
                  <div class="table-row table-header" style="--cols: 2.2fr 1fr 1fr 1fr;">
                    <span>Event</span>
                    <span>Module</span>
                    <span>Time</span>
                    <span>Status</span>
                  </div>
                  <div
                    class="table-row"
                    *ngFor="let item of dashboardActivity"
                    style="--cols: 2.2fr 1fr 1fr 1fr;"
                  >
                    <span>{{ item.event }}</span>
                    <span class="muted">{{ item.module }}</span>
                    <span class="muted">{{ item.time }}</span>
                    <span class="status" [ngClass]="item.tone">{{ item.status }}</span>
                  </div>
                </div>
              </div>
            </section>

            <section *ngSwitchCase="'office'" class="screen">
              <div class="stats-row">
                <div class="card metric-card" *ngFor="let stat of officeMetrics" [ngClass]="stat.tone">
                  <div class="metric-label">{{ stat.label }}</div>
                  <div class="metric-value">{{ stat.value }}</div>
                  <div class="metric-meta">
                    <span class="metric-caption">{{ stat.caption }}</span>
                  </div>
                  <div class="progress" *ngIf="stat.progress !== undefined">
                    <span [style.width.%]="stat.progress"></span>
                  </div>
                </div>
              </div>

              <div class="layout-2">
                <div class="stack">
                  <div class="card">
                    <div class="card-title">Programs & Projects</div>
                    <div class="program-grid">
                      <div class="card sub-card" *ngFor="let item of officePrograms" [ngClass]="item.tone">
                        <div class="card-title-sm">{{ item.title }}</div>
                        <div class="card-subtitle">{{ item.subtitle }}</div>
                        <div class="progress">
                          <span [style.width.%]="item.progress"></span>
                        </div>
                        <div class="avatar-row">
                          <span class="avatar-chip" *ngFor="let member of item.members">{{ member }}</span>
                        </div>
                      </div>
                    </div>
                  </div>

                  <div class="card">
                    <div class="card-title">Portfolio Assets</div>
                    <div class="asset-grid">
                      <button
                        *ngFor="let item of officeAssets"
                        type="button"
                        class="asset-card"
                        [ngClass]="item.tone"
                      >
                        <span>{{ item.title }}</span>
                      </button>
                    </div>
                  </div>
                </div>

                <div class="stack">
                  <div class="card">
                    <div class="card-title">3rd Party</div>
                    <div class="chip-row">
                      <span class="chip" *ngFor="let item of officeThirdParty" [ngClass]="item.tone">
                        {{ item.label }}
                      </span>
                    </div>
                  </div>

                  <div class="card">
                    <div class="card-title">Team</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let item of officeTeam">
                        <div class="avatar-chip">{{ item.title }}</div>
                        <div class="list-body">
                          <div class="list-title">{{ item.subtitle }}</div>
                          <div class="list-meta">{{ item.meta }}</div>
                        </div>
                        <span class="status" [ngClass]="item.tone">Active</span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </section>

            <section *ngSwitchCase="'portfolio'" class="screen">
              <div class="stats-row">
                <div class="card metric-card" *ngFor="let stat of portfolioMetrics" [ngClass]="stat.tone">
                  <div class="metric-label">{{ stat.label }}</div>
                  <div class="metric-value">{{ stat.value }}</div>
                  <div class="metric-meta">
                    <span class="metric-caption">{{ stat.caption }}</span>
                  </div>
                </div>
              </div>

              <div class="card">
                <div class="card-title">Portfolio Grid - Modular Tile View</div>
                <div class="portfolio-grid">
                  <div class="card sub-card" *ngFor="let item of portfolioItems" [ngClass]="item.tone">
                    <div class="card-title-sm">{{ item.title }}</div>
                    <div class="card-subtitle">{{ item.subtitle }}</div>
                    <span class="status" [ngClass]="item.tone">{{ item.status }}</span>
                  </div>
                </div>
              </div>

              <div class="card">
                <div class="card-title">Linked Platforms</div>
                <div class="chip-row">
                  <span class="chip" *ngFor="let item of portfolioPlatforms" [ngClass]="item.tone">
                    {{ item.label }}
                  </span>
                </div>
              </div>
            </section>

            <section *ngSwitchCase="'workspace'" class="screen">
              <div class="stats-row">
                <div class="card metric-card" *ngFor="let stat of workspaceMetrics" [ngClass]="stat.tone">
                  <div class="metric-label">{{ stat.label }}</div>
                  <div class="metric-value">{{ stat.value }}</div>
                  <div class="metric-meta">
                    <span class="metric-caption">{{ stat.caption }}</span>
                  </div>
                  <div class="progress" *ngIf="stat.progress !== undefined">
                    <span [style.width.%]="stat.progress"></span>
                  </div>
                </div>
              </div>

              <div class="layout-2">
                <div class="stack">
                  <div class="card">
                    <div class="card-title">Kanban Board</div>
                    <div class="kanban">
                      <div class="kanban-col" *ngFor="let col of workspaceKanban" [ngClass]="col.tone">
                        <div class="kanban-title">{{ col.title }}</div>
                        <div class="kanban-item" *ngFor="let task of col.items">
                          {{ task }}
                        </div>
                      </div>
                    </div>
                  </div>

                  <div class="card">
                    <div class="card-title">Gantt Timeline</div>
                    <div class="gantt">
                      <div class="gantt-row" *ngFor="let row of workspaceGantt">
                        <div class="gantt-label">{{ row.label }}</div>
                        <div class="gantt-track">
                          <span
                            *ngFor="let bar of row.bars"
                            class="gantt-bar"
                            [ngClass]="bar.tone"
                            [style.left.%]="bar.start"
                            [style.width.%]="bar.width"
                          ></span>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>

                <div class="stack">
                  <div class="card">
                    <div class="card-title">Calendar</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let item of workspaceCalendar">
                        <div class="list-body">
                          <div class="list-title">{{ item.title }}</div>
                          <div class="list-meta">{{ item.subtitle }}</div>
                        </div>
                        <span class="status" [ngClass]="item.tone">{{ item.meta }}</span>
                      </div>
                    </div>
                  </div>

                  <div class="card">
                    <div class="card-title">Roadmaps</div>
                    <div class="progress-list">
                      <div class="progress-item" *ngFor="let item of workspaceRoadmaps">
                        <div class="progress-meta">
                          <span>{{ item.title }}</span>
                          <span class="muted">{{ item.meta }}</span>
                        </div>
                        <div class="progress" [ngClass]="item.tone">
                          <span [style.width.%]="item.progress"></span>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </section>

            <section *ngSwitchCase="'timeline'" class="screen">
              <div class="stats-row">
                <div class="card metric-card" *ngFor="let stat of timelineMetrics" [ngClass]="stat.tone">
                  <div class="metric-label">{{ stat.label }}</div>
                  <div class="metric-value">{{ stat.value }}</div>
                  <div class="metric-meta">
                    <span class="metric-caption">{{ stat.caption }}</span>
                  </div>
                  <div class="progress" *ngIf="stat.progress !== undefined">
                    <span [style.width.%]="stat.progress"></span>
                  </div>
                </div>
              </div>

              <div class="card">
                <div class="card-title">Master Timeline - Q1 2026</div>
                <div class="gantt">
                  <div class="gantt-row" *ngFor="let row of timelineMaster">
                    <div class="gantt-label">{{ row.label }}</div>
                    <div class="gantt-track">
                      <span
                        *ngFor="let bar of row.bars"
                        class="gantt-bar"
                        [ngClass]="bar.tone"
                        [style.left.%]="bar.start"
                        [style.width.%]="bar.width"
                      ></span>
                    </div>
                  </div>
                </div>
              </div>

              <div class="layout-2">
                <div class="stack">
                  <div class="card">
                    <div class="card-title">Scheduled Events</div>
                    <div class="event-grid">
                      <div class="card sub-card" *ngFor="let item of timelineEvents" [ngClass]="item.tone">
                        <div class="card-title-sm">{{ item.title }}</div>
                        <div class="card-subtitle">{{ item.subtitle }}</div>
                      </div>
                    </div>
                  </div>
                </div>

                <div class="stack">
                  <div class="card">
                    <div class="card-title">Roadmap Progress</div>
                    <div class="progress-list">
                      <div class="progress-item" *ngFor="let item of timelineRoadmap">
                        <div class="progress-meta">
                          <span>{{ item.title }}</span>
                          <span class="muted">{{ item.meta }}</span>
                        </div>
                        <div class="progress" [ngClass]="item.tone">
                          <span [style.width.%]="item.progress"></span>
                        </div>
                      </div>
                    </div>
                  </div>

                  <div class="card">
                    <div class="card-title">Next Deadlines</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let item of timelineDeadlines">
                        <div class="list-body">
                          <div class="list-title">{{ item.title }}</div>
                          <div class="list-meta">{{ item.subtitle }}</div>
                        </div>
                        <span class="status" [ngClass]="item.tone">{{ item.meta }}</span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </section>

            <section *ngSwitchCase="'strategy'" class="screen">
              <div class="stats-row">
                <div class="card metric-card" *ngFor="let stat of strategyMetrics" [ngClass]="stat.tone">
                  <div class="metric-label">{{ stat.label }}</div>
                  <div class="metric-value">{{ stat.value }}</div>
                  <div class="metric-meta">
                    <span class="metric-caption">{{ stat.caption }}</span>
                  </div>
                  <div class="progress" *ngIf="stat.progress !== undefined">
                    <span [style.width.%]="stat.progress"></span>
                  </div>
                </div>
              </div>

              <div class="card">
                <div class="card-title">Strategy Tree</div>
                <div class="strategy-grid">
                  <div class="card sub-card" *ngFor="let item of strategyTree" [ngClass]="item.tone">
                    <div class="card-title-sm">{{ item.title }}</div>
                    <div class="card-subtitle">{{ item.subtitle }}</div>
                    <div class="card-meta">{{ item.meta }}</div>
                  </div>
                </div>
              </div>

              <div class="layout-2">
                <div class="card">
                  <div class="card-title">Tactics & Operations</div>
                  <div class="tactic-grid">
                    <div class="card sub-card" *ngFor="let item of strategyTactics" [ngClass]="item.tone">
                      <div class="card-title-sm">{{ item.title }}</div>
                      <div class="card-subtitle">{{ item.subtitle }}</div>
                    </div>
                  </div>
                </div>

                <div class="stack">
                  <div class="card">
                    <div class="card-title">Governance</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let item of strategyGovernance">
                        <div class="list-body">
                          <div class="list-title">{{ item.title }}</div>
                          <div class="list-meta">{{ item.subtitle }}</div>
                        </div>
                        <span class="status" [ngClass]="item.tone">{{ item.meta }}</span>
                      </div>
                    </div>
                  </div>

                  <div class="card">
                    <div class="card-title">OKR Progress</div>
                    <div class="progress-list">
                      <div class="progress-item" *ngFor="let item of strategyOkrs">
                        <div class="progress-meta">
                          <span>{{ item.title }}</span>
                          <span class="muted">{{ item.meta }}</span>
                        </div>
                        <div class="progress" [ngClass]="item.tone">
                          <span [style.width.%]="item.progress"></span>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </section>

            <section *ngSwitchCase="'bank'" class="screen">
              <div class="stats-row">
                <div class="card metric-card" *ngFor="let stat of bankMetrics" [ngClass]="stat.tone">
                  <div class="metric-label">{{ stat.label }}</div>
                  <div class="metric-value">{{ stat.value }}</div>
                  <div class="metric-meta">
                    <span class="metric-caption">{{ stat.caption }}</span>
                  </div>
                  <div class="progress" *ngIf="stat.progress !== undefined">
                    <span [style.width.%]="stat.progress"></span>
                  </div>
                </div>
              </div>

              <div class="layout-2">
                <div class="stack">
                  <div class="card">
                    <div class="card-title">Wallet Types</div>
                    <div class="wallet-grid">
                      <div class="card sub-card" *ngFor="let item of bankWallets" [ngClass]="item.tone">
                        <div class="card-title-sm">{{ item.title }}</div>
                        <div class="card-subtitle">{{ item.subtitle }}</div>
                        <div class="card-meta">{{ item.value }}</div>
                      </div>
                    </div>
                  </div>

                  <div class="card">
                    <div class="card-title">Fundraising & Capital</div>
                    <div class="fund-grid">
                      <div class="card sub-card" *ngFor="let item of bankFundraising" [ngClass]="item.tone">
                        <div class="card-title-sm">{{ item.title }}</div>
                        <div class="card-subtitle">{{ item.value }}</div>
                        <div class="progress">
                          <span [style.width.%]="item.progress"></span>
                        </div>
                        <div class="card-meta">{{ item.meta }}</div>
                      </div>
                    </div>
                  </div>
                </div>

                <div class="stack">
                  <div class="card">
                    <div class="card-title">Linked Platforms</div>
                    <div class="chip-row">
                      <span class="chip" *ngFor="let item of bankPlatforms" [ngClass]="item.tone">
                        {{ item.label }}
                      </span>
                    </div>
                  </div>

                  <div class="card">
                    <div class="card-title">Transactions</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let item of bankTransactions">
                        <div class="list-body">
                          <div class="list-title">{{ item.title }}</div>
                          <div class="list-meta">{{ item.subtitle }}</div>
                        </div>
                        <span class="status" [ngClass]="item.tone">{{ item.meta }}</span>
                      </div>
                    </div>
                  </div>

                  <div class="card">
                    <div class="card-title">Tax Summary</div>
                    <div class="tax-grid">
                      <div class="card sub-card" *ngFor="let item of bankTaxes" [ngClass]="item.tone">
                        <div class="card-title-sm">{{ item.title }}</div>
                        <div class="card-meta">{{ item.value }}</div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </section>

            <section *ngSwitchCase="'exchange'" class="screen">
              <div class="stats-row">
                <div class="card metric-card" *ngFor="let stat of exchangeMetrics" [ngClass]="stat.tone">
                  <div class="metric-label">{{ stat.label }}</div>
                  <div class="metric-value">{{ stat.value }}</div>
                  <div class="metric-meta">
                    <span class="metric-caption">{{ stat.caption }}</span>
                  </div>
                </div>
              </div>

              <div class="layout-2">
                <div class="stack">
                  <div class="card">
                    <div class="card-title">Bids & Offers</div>
                    <div class="table-row table-header" style="--cols: 2fr 1fr 1fr 1fr 1fr;">
                      <span>Item</span>
                      <span>Type</span>
                      <span>Value</span>
                      <span>Status</span>
                      <span>Counterparty</span>
                    </div>
                    <div
                      class="table-row"
                      *ngFor="let row of exchangeBids"
                      style="--cols: 2fr 1fr 1fr 1fr 1fr;"
                    >
                      <span>{{ row.cells[0] }}</span>
                      <span class="muted">{{ row.cells[1] }}</span>
                      <span class="muted">{{ row.cells[2] }}</span>
                      <span class="status" [ngClass]="row.badgeTone">{{ row.badge }}</span>
                      <span class="muted">{{ row.cells[4] }}</span>
                    </div>
                  </div>

                  <div class="card">
                    <div class="card-title">Deal Pipeline</div>
                    <div class="deal-grid">
                      <div class="card sub-card" *ngFor="let item of exchangePipeline" [ngClass]="item.tone">
                        <div class="card-title-sm">{{ item.title }}</div>
                        <div class="card-subtitle">{{ item.subtitle }}</div>
                        <div class="card-meta">{{ item.value }}</div>
                        <div class="progress">
                          <span [style.width.%]="item.progress"></span>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>

                <div class="stack">
                  <div class="card">
                    <div class="card-title">Requests</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let item of exchangeRequests">
                        <div class="list-body">
                          <div class="list-title">{{ item.title }}</div>
                          <div class="list-meta">{{ item.subtitle }}</div>
                        </div>
                        <span class="status" [ngClass]="item.tone">{{ item.meta }}</span>
                      </div>
                    </div>
                  </div>

                  <div class="card">
                    <div class="card-title">Linked Platforms</div>
                    <div class="chip-row">
                      <span class="chip" *ngFor="let item of exchangePlatforms" [ngClass]="item.tone">
                        {{ item.label }}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </section>

            <section *ngSwitchCase="'marketplace'" class="screen">
              <div class="stats-row">
                <div class="card metric-card" *ngFor="let stat of marketplaceMetrics" [ngClass]="stat.tone">
                  <div class="metric-label">{{ stat.label }}</div>
                  <div class="metric-value">{{ stat.value }}</div>
                  <div class="metric-meta">
                    <span class="metric-caption">{{ stat.caption }}</span>
                  </div>
                </div>
              </div>

              <div class="layout-2">
                <div class="card">
                  <div class="card-title">Marketplace - Modular Grid</div>
                  <div class="market-grid">
                    <div class="card sub-card" *ngFor="let item of marketplaceItems" [ngClass]="item.tone">
                      <div class="card-title-sm">{{ item.title }}</div>
                      <div class="card-subtitle">{{ item.subtitle }}</div>
                      <div class="card-meta">{{ item.value }}</div>
                    </div>
                  </div>
                </div>

                <div class="stack">
                  <div class="card">
                    <div class="card-title">Barter System</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let item of marketplaceBarter">
                        <div class="list-body">
                          <div class="list-title">{{ item.title }}</div>
                          <div class="list-meta">{{ item.subtitle }}</div>
                        </div>
                        <span class="status" [ngClass]="item.tone">{{ item.meta }}</span>
                      </div>
                    </div>
                  </div>

                  <div class="card">
                    <div class="card-title">My Orders</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let item of marketplaceOrders">
                        <div class="list-body">
                          <div class="list-title">{{ item.title }}</div>
                          <div class="list-meta">{{ item.subtitle }}</div>
                        </div>
                        <span class="status" [ngClass]="item.tone">{{ item.meta }}</span>
                      </div>
                    </div>
                  </div>

                  <div class="card">
                    <div class="card-title">Linked Platforms</div>
                    <div class="chip-row">
                      <span class="chip" *ngFor="let item of marketplacePlatforms" [ngClass]="item.tone">
                        {{ item.label }}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </section>

            <section *ngSwitchCase="'studio'" class="screen">
              <div class="stats-row">
                <div class="card metric-card" *ngFor="let stat of studioMetrics" [ngClass]="stat.tone">
                  <div class="metric-label">{{ stat.label }}</div>
                  <div class="metric-value">{{ stat.value }}</div>
                  <div class="metric-meta">
                    <span class="metric-caption">{{ stat.caption }}</span>
                  </div>
                </div>
              </div>

              <div class="layout-2">
                <div class="stack">
                  <div class="card">
                    <div class="card-title">Ideas & Concepts</div>
                    <div class="studio-grid">
                      <div class="card sub-card" *ngFor="let item of studioIdeas" [ngClass]="item.tone">
                        <div class="card-title-sm">{{ item.title }}</div>
                        <div class="card-subtitle">{{ item.subtitle }}</div>
                        <span class="status" [ngClass]="item.tone">{{ item.status }}</span>
                      </div>
                    </div>
                  </div>

                  <div class="card">
                    <div class="card-title">Testing & Testbeds</div>
                    <div class="studio-grid">
                      <div class="card sub-card" *ngFor="let item of studioTestbeds" [ngClass]="item.tone">
                        <div class="card-title-sm">{{ item.title }}</div>
                        <div class="card-subtitle">{{ item.subtitle }}</div>
                      </div>
                    </div>
                  </div>
                </div>

                <div class="stack">
                  <div class="card">
                    <div class="card-title">Toolsets & Toolkits</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let item of studioToolsets">
                        <div class="list-body">
                          <div class="list-title">{{ item.title }}</div>
                          <div class="list-meta">{{ item.subtitle }}</div>
                        </div>
                        <span class="status" [ngClass]="item.tone">{{ item.meta }}</span>
                      </div>
                    </div>
                  </div>

                  <div class="card">
                    <div class="card-title">Files & Notes</div>
                    <div class="asset-grid">
                      <button class="asset-card" *ngFor="let item of studioFiles" [ngClass]="item.tone" type="button">
                        {{ item.title }}
                      </button>
                    </div>
                  </div>

                  <div class="card">
                    <div class="card-title">Linked Platforms</div>
                    <div class="chip-row">
                      <span class="chip" *ngFor="let item of studioPlatforms" [ngClass]="item.tone">
                        {{ item.label }}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </section>

            <section *ngSwitchCase="'community'" class="screen">
              <div class="stats-row">
                <div class="card metric-card" *ngFor="let stat of communityMetrics" [ngClass]="stat.tone">
                  <div class="metric-label">{{ stat.label }}</div>
                  <div class="metric-value">{{ stat.value }}</div>
                  <div class="metric-meta">
                    <span class="metric-caption">{{ stat.caption }}</span>
                  </div>
                </div>
              </div>

              <div class="layout-2">
                <div class="card">
                  <div class="card-title">Feeds & Timelines</div>
                  <div class="feed-list">
                    <div class="feed-item" *ngFor="let item of communityFeeds">
                      <div class="avatar-chip">{{ item.author.charAt(0) }}</div>
                      <div class="feed-body">
                        <div class="feed-author">{{ item.author }}</div>
                        <div class="feed-message">{{ item.message }}</div>
                        <div class="feed-meta">{{ item.stats }}</div>
                      </div>
                      <div class="feed-time">{{ item.time }}</div>
                    </div>
                  </div>
                </div>

                <div class="stack">
                  <div class="card">
                    <div class="card-title">Spaces & Rooms</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let item of communitySpaces">
                        <div class="list-body">
                          <div class="list-title">{{ item.title }}</div>
                          <div class="list-meta">{{ item.subtitle }}</div>
                        </div>
                        <span class="status" [ngClass]="item.tone">{{ item.meta }}</span>
                      </div>
                    </div>
                  </div>

                  <div class="card">
                    <div class="card-title">DMs</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let item of communityDms">
                        <div class="avatar-chip">{{ item.title.charAt(0) }}</div>
                        <div class="list-body">
                          <div class="list-title">{{ item.title }}</div>
                          <div class="list-meta">{{ item.subtitle }}</div>
                        </div>
                      </div>
                    </div>
                  </div>

                  <div class="card">
                    <div class="card-title">Linked Platforms</div>
                    <div class="chip-row">
                      <span class="chip" *ngFor="let item of communityPlatforms" [ngClass]="item.tone">
                        {{ item.label }}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </section>

            <section *ngSwitchCase="'developer'" class="screen">
              <div class="stats-row">
                <div class="card metric-card" *ngFor="let stat of developerMetrics" [ngClass]="stat.tone">
                  <div class="metric-label">{{ stat.label }}</div>
                  <div class="metric-value">{{ stat.value }}</div>
                  <div class="metric-meta">
                    <span class="metric-caption">{{ stat.caption }}</span>
                  </div>
                  <div class="progress" *ngIf="stat.progress !== undefined">
                    <span [style.width.%]="stat.progress"></span>
                  </div>
                </div>
              </div>

              <div class="layout-2">
                <div class="stack">
                  <div class="card">
                    <div class="card-title">API Reference</div>
                    <div class="api-list">
                      <div class="api-row" *ngFor="let row of developerApi">
                        <span class="api-method" [ngClass]="row.tone">{{ row.method }}</span>
                        <span class="api-path">{{ row.path }}</span>
                        <span class="api-desc">{{ row.description }}</span>
                      </div>
                    </div>
                  </div>

                  <div class="card">
                    <div class="card-title">Extensions & Integrations</div>
                    <div class="extension-grid">
                      <div class="card sub-card" *ngFor="let item of developerExtensions" [ngClass]="item.tone">
                        <div class="card-title-sm">{{ item.title }}</div>
                        <div class="card-subtitle">{{ item.subtitle }}</div>
                      </div>
                    </div>
                  </div>
                </div>

                <div class="stack">
                  <div class="card">
                    <div class="card-title">API Keys</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let item of developerKeys">
                        <div class="list-body">
                          <div class="list-title">{{ item.title }}</div>
                          <div class="list-meta">{{ item.subtitle }}</div>
                        </div>
                        <span class="status" [ngClass]="item.tone">{{ item.meta }}</span>
                      </div>
                    </div>
                  </div>

                  <div class="card">
                    <div class="card-title">SDKs</div>
                    <div class="chip-row">
                      <span class="chip" *ngFor="let item of developerSdks" [ngClass]="item.tone">
                        {{ item.label }}
                      </span>
                    </div>
                  </div>

                  <div class="card">
                    <div class="card-title">Webhooks</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let item of developerWebhooks">
                        <div class="list-body">
                          <div class="list-title">{{ item.title }}</div>
                          <div class="list-meta">{{ item.subtitle }}</div>
                        </div>
                        <span class="status" [ngClass]="item.tone">{{ item.meta }}</span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </section>

            <section *ngSwitchCase="'profile'" class="screen">
              <div class="profile-header">
                <div class="card profile-card tone-amber">
                  <div class="avatar-lg">J</div>
                  <div class="profile-info">
                    <div class="profile-name">Jordan Chen</div>
                    <div class="profile-title">Independent Technology Consultant</div>
                    <div class="chip-row">
                      <span class="chip tone-amber">Pro</span>
                      <span class="chip tone-emerald">Verified</span>
                      <span class="chip tone-rose">Builder</span>
                    </div>
                  </div>
                </div>

                <div class="stats-row profile-stats">
                  <div class="card metric-card" *ngFor="let stat of profileMetrics" [ngClass]="stat.tone">
                    <div class="metric-label">{{ stat.label }}</div>
                    <div class="metric-value">{{ stat.value }}</div>
                    <div class="metric-meta">
                      <span class="metric-caption">{{ stat.caption }}</span>
                    </div>
                  </div>
                </div>
              </div>

              <div class="card">
                <div class="card-title">Personas & Roles</div>
                <div class="persona-grid">
                  <div class="card sub-card" *ngFor="let item of profilePersonas" [ngClass]="item.tone">
                    <div class="card-title-sm">{{ item.title }}</div>
                    <div class="card-subtitle">{{ item.subtitle }}</div>
                  </div>
                </div>
              </div>

              <div class="card">
                <div class="card-title">Settings & Configuration</div>
                <div class="settings-grid">
                  <button class="asset-card" *ngFor="let item of profileSettings" [ngClass]="item.tone" type="button">
                    {{ item.title }}
                  </button>
                </div>
              </div>

              <div class="stats-row">
                <div class="card metric-card" *ngFor="let stat of profileStats" [ngClass]="stat.tone">
                  <div class="metric-label">{{ stat.label }}</div>
                  <div class="metric-value">{{ stat.value }}</div>
                </div>
              </div>
            </section>

            <section *ngSwitchCase="'organizations'" class="screen">
              <div class="stats-row">
                <div class="card metric-card" *ngFor="let stat of orgMetrics" [ngClass]="stat.tone">
                  <div class="metric-label">{{ stat.label }}</div>
                  <div class="metric-value">{{ stat.value }}</div>
                  <div class="metric-meta">
                    <span class="metric-caption">{{ stat.caption }}</span>
                  </div>
                </div>
              </div>

              <div class="layout-2">
                <div class="stack">
                  <div class="card">
                    <div class="card-title">Organizations - Modular Grid</div>
                    <div class="org-grid">
                      <div class="card sub-card" *ngFor="let item of orgCards" [ngClass]="item.tone">
                        <div class="card-title-sm">{{ item.title }}</div>
                        <div class="card-subtitle">{{ item.subtitle }}</div>
                        <div class="card-meta">{{ item.meta }}</div>
                        <span class="status" [ngClass]="item.tone">{{ item.status }}</span>
                      </div>
                    </div>
                  </div>

                  <div class="card">
                    <div class="card-title">Governance & Proposals</div>
                    <div class="table-row table-header" style="--cols: 2fr 1fr 1fr 1fr;">
                      <span>Proposal</span>
                      <span>Org</span>
                      <span>Votes</span>
                      <span>Status</span>
                    </div>
                    <div
                      class="table-row"
                      *ngFor="let row of orgProposals"
                      style="--cols: 2fr 1fr 1fr 1fr;"
                    >
                      <span>{{ row.cells[0] }}</span>
                      <span class="muted">{{ row.cells[1] }}</span>
                      <span class="muted">{{ row.cells[2] }}</span>
                      <span class="status" [ngClass]="row.badgeTone">{{ row.badge }}</span>
                    </div>
                  </div>
                </div>

                <div class="stack">
                  <div class="card">
                    <div class="card-title">My Roles</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let item of orgRoles">
                        <div class="list-body">
                          <div class="list-title">{{ item.title }}</div>
                          <div class="list-meta">{{ item.subtitle }}</div>
                        </div>
                        <span class="status" [ngClass]="item.tone">{{ item.meta }}</span>
                      </div>
                    </div>
                  </div>

                  <div class="card">
                    <div class="card-title">Cap Tables</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let item of orgCapTables">
                        <div class="list-body">
                          <div class="list-title">{{ item.title }}</div>
                          <div class="list-meta">{{ item.subtitle }}</div>
                        </div>
                        <span class="status" [ngClass]="item.tone">{{ item.meta }}</span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </section>

            <section *ngSwitchCase="'legal'" class="screen">
              <div class="stats-row">
                <div class="card metric-card" *ngFor="let stat of legalMetrics" [ngClass]="stat.tone">
                  <div class="metric-label">{{ stat.label }}</div>
                  <div class="metric-value">{{ stat.value }}</div>
                  <div class="metric-meta">
                    <span class="metric-caption">{{ stat.caption }}</span>
                  </div>
                </div>
              </div>

              <div class="layout-2">
                <div class="stack">
                  <div class="card">
                    <div class="card-title">IP & Trademarks</div>
                    <div class="legal-grid">
                      <div class="card sub-card" *ngFor="let item of legalIp" [ngClass]="item.tone">
                        <div class="card-title-sm">{{ item.title }}</div>
                        <div class="card-subtitle">{{ item.subtitle }}</div>
                        <span class="status" [ngClass]="item.tone">{{ item.status }}</span>
                      </div>
                    </div>
                  </div>

                  <div class="card">
                    <div class="card-title">Contracts & Agreements</div>
                    <div class="table-row table-header" style="--cols: 2fr 1fr 1fr 1fr 1fr;">
                      <span>Contract</span>
                      <span>Party</span>
                      <span>Value</span>
                      <span>Expires</span>
                      <span>Status</span>
                    </div>
                    <div
                      class="table-row"
                      *ngFor="let row of legalContracts"
                      style="--cols: 2fr 1fr 1fr 1fr 1fr;"
                    >
                      <span>{{ row.cells[0] }}</span>
                      <span class="muted">{{ row.cells[1] }}</span>
                      <span class="muted">{{ row.cells[2] }}</span>
                      <span class="muted">{{ row.cells[3] }}</span>
                      <span class="status" [ngClass]="row.badgeTone">{{ row.badge }}</span>
                    </div>
                  </div>

                  <div class="card">
                    <div class="card-title">Compliance & Audit</div>
                    <div class="legal-grid">
                      <div class="card sub-card" *ngFor="let item of legalCompliance" [ngClass]="item.tone">
                        <div class="card-title-sm">{{ item.title }}</div>
                        <div class="card-subtitle">{{ item.subtitle }}</div>
                        <span class="status" [ngClass]="item.tone">{{ item.status }}</span>
                      </div>
                    </div>
                  </div>
                </div>

                <div class="stack">
                  <div class="card">
                    <div class="card-title">Upcoming</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let item of legalUpcoming">
                        <div class="list-body">
                          <div class="list-title">{{ item.title }}</div>
                          <div class="list-meta">{{ item.subtitle }}</div>
                        </div>
                        <span class="status" [ngClass]="item.tone">{{ item.meta }}</span>
                      </div>
                    </div>
                  </div>

                  <div class="card">
                    <div class="card-title">Quick Actions</div>
                    <div class="action-list">
                      <button
                        *ngFor="let item of legalActions"
                        type="button"
                        class="action-button"
                        [ngClass]="item.tone"
                      >
                        {{ item.title }}
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </section>
          </ng-container>
        </section>
      </main>
    </div>
  `,
})
class AppComponent {
  readonly navItems = NAV_ITEMS;
  readonly activeScreen = signal<ScreenId>('dashboard');
  readonly meta = computed(() => SCREEN_META[this.activeScreen()]);

  readonly dashboardMetrics: Metric[] = [
    {
      label: 'Portfolio Health',
      value: '87%',
      change: '4%',
      trend: 'up',
      caption: '3 at risk',
      progress: 87,
      tone: 'tone-blue',
    },
    {
      label: 'Active Projects',
      value: '12',
      change: '2%',
      trend: 'up',
      caption: 'Concurrent',
      tone: 'tone-teal',
    },
    {
      label: 'Net Revenue',
      value: '$24.8K',
      change: '11%',
      trend: 'up',
      caption: 'This month',
      tone: 'tone-emerald',
    },
    {
      label: 'AI Credits',
      value: '8,420',
      change: '2%',
      trend: 'down',
      caption: 'Remaining',
      progress: 66,
      tone: 'tone-purple',
    },
  ];

  readonly dashboardQuickLinks: ChipItem[] = [
    { label: 'Office', tone: 'tone-blue' },
    { label: 'Portfolio', tone: 'tone-purple' },
    { label: 'Bank', tone: 'tone-emerald' },
    { label: 'Exchange', tone: 'tone-amber' },
    { label: 'Marketplace', tone: 'tone-teal' },
    { label: 'Studio', tone: 'tone-indigo' },
    { label: 'Community', tone: 'tone-rose' },
    { label: 'Developer', tone: 'tone-cyan' },
  ];

  readonly dashboardActivity: ActivityItem[] = [
    {
      event: 'Payment received - $3,200',
      module: 'Exchange',
      time: '2 min ago',
      status: 'Active',
      tone: 'tone-emerald',
    },
    {
      event: 'New proposal: Brand Redesign',
      module: 'Studio',
      time: '14 min ago',
      status: 'Active',
      tone: 'tone-blue',
    },
    {
      event: 'Project milestone hit',
      module: 'Office',
      time: '1 hr ago',
      status: 'Active',
      tone: 'tone-purple',
    },
    {
      event: 'Community post: 24 reactions',
      module: 'Community',
      time: '2 hr ago',
      status: 'Active',
      tone: 'tone-rose',
    },
    {
      event: 'Token distribution complete',
      module: 'Bank',
      time: '5 hr ago',
      status: 'Active',
      tone: 'tone-teal',
    },
  ];

  readonly officeMetrics: Metric[] = [
    { label: 'Programs', value: '4', caption: '2 delayed', tone: 'tone-blue' },
    { label: 'Projects', value: '18', caption: '12 active', tone: 'tone-teal', progress: 70 },
    { label: 'Milestones Due', value: '6', caption: 'Next 7 days', tone: 'tone-amber' },
    { label: 'Team Capacity', value: '73%', caption: '-5%', tone: 'tone-emerald', progress: 73 },
  ];

  readonly officePrograms: CardItem[] = [
    {
      title: 'Alpha Platform',
      subtitle: 'In Progress - 8/12',
      tone: 'tone-blue',
      progress: 70,
      members: ['A', 'A', 'A'],
    },
    {
      title: 'Beta Campaign',
      subtitle: 'Planning - 2/6',
      tone: 'tone-teal',
      progress: 32,
      members: ['A', 'A', 'A'],
    },
    {
      title: 'Gamma Research',
      subtitle: 'On Hold - 5/5',
      tone: 'tone-emerald',
      progress: 100,
      members: ['A', 'A', 'A'],
    },
    {
      title: 'Delta Ops',
      subtitle: 'In Progress - 11/15',
      tone: 'tone-amber',
      progress: 60,
      members: ['A', 'A', 'A'],
    },
    {
      title: 'Epsilon Design',
      subtitle: 'Active - 3/8',
      tone: 'tone-purple',
      progress: 38,
      members: ['A', 'A', 'A'],
    },
    {
      title: 'Zeta Legal',
      subtitle: 'Review - 7/9',
      tone: 'tone-rose',
      progress: 78,
      members: ['A', 'A', 'A'],
    },
  ];

  readonly officeAssets: CardItem[] = [
    { title: 'Projects', tone: 'tone-blue' },
    { title: 'Programs', tone: 'tone-teal' },
    { title: 'Assets', tone: 'tone-purple' },
    { title: 'Solutions', tone: 'tone-emerald' },
    { title: 'Artifacts', tone: 'tone-indigo' },
    { title: 'Resources', tone: 'tone-rose' },
    { title: 'Blueprints', tone: 'tone-amber' },
    { title: 'APIs', tone: 'tone-cyan' },
    { title: 'Components', tone: 'tone-blue' },
    { title: 'Datasets', tone: 'tone-teal' },
  ];

  readonly officeThirdParty: ChipItem[] = [
    { label: 'Jira', tone: 'tone-blue' },
    { label: 'Monday', tone: 'tone-amber' },
    { label: 'GitHub', tone: 'tone-indigo' },
    { label: 'GitLab', tone: 'tone-rose' },
    { label: 'Notion', tone: 'tone-teal' },
  ];

  readonly officeTeam: CardItem[] = [
    { title: 'J', subtitle: 'Jordan C.', meta: '2 active projects', tone: 'tone-blue' },
    { title: 'M', subtitle: 'Maria S.', meta: '2 active projects', tone: 'tone-emerald' },
    { title: 'D', subtitle: 'Devon P.', meta: '2 active projects', tone: 'tone-amber' },
    { title: 'P', subtitle: 'Priya N.', meta: '3 active projects', tone: 'tone-rose' },
  ];

  readonly portfolioMetrics: Metric[] = [
    { label: 'Projects', value: '12', caption: 'Active', tone: 'tone-blue' },
    { label: 'Programs', value: '4', caption: 'Operating', tone: 'tone-teal' },
    { label: 'Assets', value: '28', caption: 'Managed', tone: 'tone-purple' },
    { label: 'Solutions', value: '7', caption: 'Live', tone: 'tone-emerald' },
    { label: 'Artifacts', value: '15', caption: 'Ready', tone: 'tone-indigo' },
  ];

  readonly portfolioItems: CardItem[] = [
    { title: 'Brand Identity System', subtitle: 'Design', status: 'Active', tone: 'tone-purple' },
    { title: 'UI Component Lib', subtitle: 'Asset', status: 'Active', tone: 'tone-rose' },
    { title: 'Product Roadmap', subtitle: 'Strategy', status: 'Active', tone: 'tone-blue' },
    { title: 'API Gateway v2', subtitle: 'Dev', status: 'Released', tone: 'tone-teal' },
    { title: 'Tokenomics Model', subtitle: 'Finance', status: 'Review', tone: 'tone-cyan' },
    { title: 'Legal Templates', subtitle: 'Legal', status: 'Active', tone: 'tone-rose' },
    { title: 'Market Research', subtitle: 'Research', status: 'Draft', tone: 'tone-amber' },
    { title: 'CRM Integration', subtitle: 'Solution', status: 'Active', tone: 'tone-emerald' },
    { title: 'Analytics Dashboard', subtitle: 'Dev', status: 'Active', tone: 'tone-indigo' },
    { title: 'Content Strategy', subtitle: 'Marketing', status: 'Draft', tone: 'tone-amber' },
    { title: 'Mobile App MVP', subtitle: 'Dev', status: 'Building', tone: 'tone-purple' },
    { title: 'Partnership Deck', subtitle: 'Sales', status: 'Active', tone: 'tone-amber' },
  ];

  readonly portfolioPlatforms: ChipItem[] = [
    { label: 'Behance', tone: 'tone-blue' },
    { label: 'GitHub', tone: 'tone-indigo' },
    { label: 'Dribbble', tone: 'tone-rose' },
    { label: 'Figma', tone: 'tone-purple' },
    { label: 'Notion', tone: 'tone-teal' },
    { label: 'Google Drive', tone: 'tone-cyan' },
  ];

  readonly workspaceMetrics: Metric[] = [
    { label: 'Open Tasks', value: '34', caption: '8 overdue', tone: 'tone-blue' },
    { label: 'Sprints Active', value: '3', caption: '2 on track', tone: 'tone-teal', progress: 60 },
    { label: 'Blocked', value: '5', caption: 'Needs action', tone: 'tone-amber' },
    { label: 'Done This Week', value: '21', caption: '15%', tone: 'tone-emerald' },
  ];

  readonly workspaceKanban: KanbanColumn[] = [
    {
      title: 'Backlog',
      tone: 'tone-blue',
      items: ['Auth redesign', 'Payment flow', 'API docs', 'Onboard UX'],
    },
    {
      title: 'In Progress',
      tone: 'tone-amber',
      items: ['Mobile nav', 'Token calc', 'Legal review', 'Test suite'],
    },
    {
      title: 'Review',
      tone: 'tone-purple',
      items: ['Brand deck', 'Cop model', 'RFC-009', 'Data model'],
    },
    {
      title: 'Done',
      tone: 'tone-emerald',
      items: ['Login fix', 'CSV export', 'Error states', 'Docs v2'],
    },
  ];

  readonly workspaceCalendar: CardItem[] = [
    { title: 'Client sync 10am', subtitle: 'Mon Mar 10', meta: 'Today', tone: 'tone-blue' },
    { title: 'Sprint review 3pm', subtitle: 'Mon Mar 10', meta: 'Today', tone: 'tone-teal' },
    { title: 'Payment due', subtitle: 'Tue Mar 11', meta: 'Tomorrow', tone: 'tone-amber' },
    { title: 'Board meeting', subtitle: 'Wed Mar 12', meta: 'Upcoming', tone: 'tone-rose' },
    { title: 'Milestone delivery', subtitle: 'Thu Mar 13', meta: 'Upcoming', tone: 'tone-purple' },
  ];

  readonly workspaceRoadmaps: Array<CardItem & { progress: number }> = [
    { title: 'Q1 2026 - Foundation', meta: '65%', progress: 65, tone: 'tone-blue' },
    { title: 'Q2 2026 - Growth', meta: '33%', progress: 33, tone: 'tone-teal' },
    { title: 'Q3 2026 - Scale', meta: '12%', progress: 12, tone: 'tone-rose' },
  ];

  readonly workspaceGantt: GanttRow[] = [
    {
      label: 'Alpha Platform',
      bars: [
        { label: 'Alpha', tone: 'tone-blue', start: 10, width: 55 },
        { label: 'Alpha', tone: 'tone-teal', start: 70, width: 15 },
      ],
    },
    {
      label: 'Beta Campaign',
      bars: [{ label: 'Beta', tone: 'tone-amber', start: 25, width: 40 }],
    },
    {
      label: 'Gamma Research',
      bars: [{ label: 'Gamma', tone: 'tone-emerald', start: 5, width: 70 }],
    },
    {
      label: 'Delta Ops',
      bars: [{ label: 'Delta', tone: 'tone-purple', start: 40, width: 35 }],
    },
    {
      label: 'Epsilon Design',
      bars: [{ label: 'Epsilon', tone: 'tone-rose', start: 55, width: 30 }],
    },
  ];

  readonly timelineMetrics: Metric[] = [
    { label: 'Milestones', value: '24', caption: 'Q1 2026', tone: 'tone-blue' },
    { label: 'Scheduled Events', value: '18', caption: 'Next 30 days', tone: 'tone-teal' },
    { label: 'Overdue Items', value: '3', caption: 'Needs attention', tone: 'tone-rose' },
    { label: 'Completion Rate', value: '78%', caption: 'This quarter', tone: 'tone-emerald', progress: 78 },
  ];

  readonly timelineMaster: GanttRow[] = [
    {
      label: 'Platform MVP',
      bars: [
        { label: 'MVP', tone: 'tone-blue', start: 5, width: 55 },
        { label: 'MVP', tone: 'tone-purple', start: 62, width: 25 },
      ],
    },
    {
      label: 'Design System',
      bars: [{ label: 'Design', tone: 'tone-purple', start: 15, width: 70 }],
    },
    {
      label: 'API v1 Launch',
      bars: [{ label: 'API', tone: 'tone-emerald', start: 30, width: 40 }],
    },
    {
      label: 'Beta Onboarding',
      bars: [{ label: 'Beta', tone: 'tone-amber', start: 45, width: 35 }],
    },
    {
      label: 'Legal Review',
      bars: [{ label: 'Legal', tone: 'tone-rose', start: 20, width: 25 }],
    },
    {
      label: 'Exchange Module',
      bars: [{ label: 'Exchange', tone: 'tone-cyan', start: 60, width: 30 }],
    },
    {
      label: 'Community Beta',
      bars: [{ label: 'Community', tone: 'tone-rose', start: 68, width: 26 }],
    },
    {
      label: 'AI Agent v1',
      bars: [{ label: 'AI', tone: 'tone-indigo', start: 52, width: 30 }],
    },
  ];

  readonly timelineEvents: CardItem[] = [
    { title: 'Client sync', subtitle: 'Mon Mar 10', tone: 'tone-blue' },
    { title: 'Sprint review', subtitle: 'Mon Mar 10', tone: 'tone-purple' },
    { title: 'Board meeting', subtitle: 'Wed Mar 12', tone: 'tone-amber' },
    { title: 'Milestone gate', subtitle: 'Fri Mar 14', tone: 'tone-rose' },
    { title: 'Tax filing due', subtitle: 'Mar 22', tone: 'tone-emerald' },
    { title: 'Q1 close', subtitle: 'Mar 31', tone: 'tone-teal' },
  ];

  readonly timelineRoadmap: Array<CardItem & { progress: number }> = [
    { title: 'Q1 Foundation', meta: '65%', progress: 65, tone: 'tone-blue' },
    { title: 'Q2 Growth', meta: '33%', progress: 33, tone: 'tone-teal' },
    { title: 'Q3 Scale', meta: '12%', progress: 12, tone: 'tone-rose' },
    { title: 'Q4 Expand', meta: '4%', progress: 4, tone: 'tone-amber' },
  ];

  readonly timelineDeadlines: CardItem[] = [
    { title: 'Legal Review', subtitle: 'In 7 days', meta: 'Due', tone: 'tone-rose' },
    { title: 'API Launch', subtitle: 'In 12 days', meta: 'Due', tone: 'tone-teal' },
    { title: 'Board Deck', subtitle: 'In 18 days', meta: 'Due', tone: 'tone-amber' },
    { title: 'Q1 Close', subtitle: 'In 22 days', meta: 'Due', tone: 'tone-blue' },
  ];

  readonly strategyMetrics: Metric[] = [
    { label: 'Strategic OKRs', value: '5', caption: '3 on track', tone: 'tone-purple', progress: 60 },
    { label: 'Tactical Initiatives', value: '18', caption: '11 active', tone: 'tone-blue' },
    { label: 'Ops Processes', value: '24', caption: 'Documented', tone: 'tone-emerald' },
    { label: 'Governance Items', value: '7', caption: '2 pending vote', tone: 'tone-amber' },
  ];

  readonly strategyTree: CardItem[] = [
    {
      title: 'Vision',
      subtitle: 'Be the OS for independent work',
      meta: 'Strategic',
      tone: 'tone-purple',
    },
    {
      title: 'Mission',
      subtitle: 'Unify tools - empower workers',
      meta: 'Strategic',
      tone: 'tone-blue',
    },
    {
      title: 'OBJ 1',
      subtitle: 'Reach 50K users by Q4',
      meta: 'Objective',
      tone: 'tone-teal',
    },
    {
      title: 'OBJ 2',
      subtitle: '$1M ARR by Q3',
      meta: 'Objective',
      tone: 'tone-emerald',
    },
    {
      title: 'OBJ 3',
      subtitle: 'Community 10K members',
      meta: 'Objective',
      tone: 'tone-rose',
    },
    {
      title: 'KR 1',
      subtitle: 'NPS > 60',
      meta: 'Key Result',
      tone: 'tone-amber',
    },
  ];

  readonly strategyTactics: CardItem[] = [
    { title: 'Pricing Experiments', subtitle: 'Tactical - Active', tone: 'tone-blue' },
    { title: 'Onboarding Funnel', subtitle: 'Tactical - Active', tone: 'tone-teal' },
    { title: 'Partnership Program', subtitle: 'Tactical - Active', tone: 'tone-emerald' },
    { title: 'Content Marketing', subtitle: 'Tactical - Active', tone: 'tone-amber' },
    { title: 'API Integrations', subtitle: 'Tactical - Active', tone: 'tone-purple' },
    { title: 'Support Playbook', subtitle: 'Tactical - Active', tone: 'tone-rose' },
    { title: 'Legal Frameworks', subtitle: 'Tactical - Active', tone: 'tone-rose' },
    { title: 'Data Infrastructure', subtitle: 'Tactical - Active', tone: 'tone-amber' },
  ];

  readonly strategyGovernance: CardItem[] = [
    { title: 'RFC-009', subtitle: 'Under Review', meta: 'Pending', tone: 'tone-rose' },
    { title: 'Equity Policy', subtitle: 'Approved', meta: 'Approved', tone: 'tone-emerald' },
    { title: 'Data Retention', subtitle: 'Pending Vote', meta: 'Pending', tone: 'tone-amber' },
    { title: 'Member Charter', subtitle: 'Draft', meta: 'Draft', tone: 'tone-purple' },
  ];

  readonly strategyOkrs: Array<CardItem & { progress: number }> = [
    { title: 'User Growth', meta: '78%', progress: 78, tone: 'tone-blue' },
    { title: 'Revenue', meta: '45%', progress: 45, tone: 'tone-emerald' },
    { title: 'Community', meta: '62%', progress: 62, tone: 'tone-rose' },
  ];

  readonly bankMetrics: Metric[] = [
    { label: 'Total Balance', value: '$142,800', caption: '8%', tone: 'tone-emerald' },
    { label: 'Operations', value: '$28,400', caption: 'Ops wallet', tone: 'tone-teal', progress: 55 },
    { label: 'Investments', value: '$89,200', caption: 'Portfolio', tone: 'tone-blue' },
    { label: 'Trading', value: '$18,300', caption: 'Exchange', tone: 'tone-amber' },
    { label: 'Personal', value: '$6,900', caption: 'Spending', tone: 'tone-cyan', progress: 40 },
  ];

  readonly bankWallets: CardItem[] = [
    { title: 'Personal Spending', subtitle: 'Daily - bills', value: '$6,900', tone: 'tone-cyan' },
    { title: 'Operations', subtitle: 'Payroll - tools', value: '$28,400', tone: 'tone-emerald' },
    { title: 'Investment', subtitle: 'Stocks - bonds', value: '$89,200', tone: 'tone-blue' },
    { title: 'Trading', subtitle: 'Crypto - tokens', value: '$18,300', tone: 'tone-amber' },
    { title: 'Marketplace', subtitle: 'Purchases', value: '$4,200', tone: 'tone-teal' },
    { title: 'Coop Pool', subtitle: 'Collective capital', value: '$124,500', tone: 'tone-purple' },
  ];

  readonly bankFundraising: CardItem[] = [
    { title: 'Seed Round', subtitle: '$450K raised', meta: '72% of goal', progress: 72, tone: 'tone-blue' },
    { title: 'Community Bond', subtitle: '$120K raised', meta: '48% of goal', progress: 48, tone: 'tone-emerald' },
    { title: 'Equipment Lease', subtitle: '$28K raised', meta: '96% of goal', progress: 96, tone: 'tone-amber' },
  ];

  readonly bankPlatforms: ChipItem[] = [
    { label: 'Stripe', tone: 'tone-blue' },
    { label: 'Wells Fargo', tone: 'tone-rose' },
    { label: 'Chase', tone: 'tone-teal' },
    { label: 'GoFundMe', tone: 'tone-emerald' },
    { label: 'Patreon', tone: 'tone-amber' },
  ];

  readonly bankTransactions: CardItem[] = [
    { title: 'Stripe payout', subtitle: '$3,200', meta: 'Cleared', tone: 'tone-emerald' },
    { title: 'Tool subscription', subtitle: '-$49', meta: 'Posted', tone: 'tone-rose' },
    { title: 'Coop distribution', subtitle: '$840', meta: 'Posted', tone: 'tone-teal' },
    { title: 'Tax payment', subtitle: '-$2,100', meta: 'Pending', tone: 'tone-amber' },
    { title: 'Invoice paid', subtitle: '$8,400', meta: 'Cleared', tone: 'tone-blue' },
  ];

  readonly bankTaxes: CardItem[] = [
    { title: 'Income Tax', value: '$18,400', tone: 'tone-rose' },
    { title: 'Self-Employ', value: '$4,200', tone: 'tone-amber' },
    { title: 'Deductions', value: '-$6,800', tone: 'tone-emerald' },
    { title: 'Estimated', value: '$15,800', tone: 'tone-amber' },
  ];

  readonly exchangeMetrics: Metric[] = [
    { label: 'Open Bids', value: '7', caption: '$42K value', tone: 'tone-amber' },
    { label: 'Active Deals', value: '3', caption: '$18K escrow', tone: 'tone-emerald' },
    { label: 'Proposals Sent', value: '12', caption: '4 responded', tone: 'tone-blue' },
    { label: 'Trading Volume', value: '$284K', caption: 'Last 30 days', tone: 'tone-amber' },
  ];

  readonly exchangeBids: TableRow[] = [
    { cells: ['Brand Identity', 'Offer', '$8,400', 'Active', 'Studio Co'], badge: 'Active', badgeTone: 'tone-emerald' },
    { cells: ['API Integration', 'Bid', '$12,000', 'Pending', 'TechCorp'], badge: 'Pending', badgeTone: 'tone-amber' },
    { cells: ['UX Audit', 'Proposal', '$3,200', 'Countered', 'StartupX'], badge: 'Countered', badgeTone: 'tone-rose' },
    { cells: ['Data Pipeline', 'Offer', '$22,000', 'Due Diligence', 'Enterprise Y'], badge: 'Due Diligence', badgeTone: 'tone-blue' },
    { cells: ['Community Module', 'Bid', '$5,800', 'Active', 'DevDAO'], badge: 'Active', badgeTone: 'tone-emerald' },
  ];

  readonly exchangePipeline: CardItem[] = [
    { title: 'Enterprise Y Deal', subtitle: 'Due Diligence', value: '$22,000', progress: 70, tone: 'tone-amber' },
    { title: 'API Integration', subtitle: 'Escrow Active', value: '$12,000', progress: 55, tone: 'tone-emerald' },
    { title: 'DevDAO Proposal', subtitle: 'Negotiation', value: '$5,800', progress: 35, tone: 'tone-blue' },
  ];

  readonly exchangeRequests: CardItem[] = [
    { title: 'UX Design RFP', subtitle: 'Open - Bids: 3', meta: 'Open', tone: 'tone-amber' },
    { title: 'Backend Dev RFQ', subtitle: 'Open - Bids: 3', meta: 'Open', tone: 'tone-amber' },
    { title: 'Legal Review RFP', subtitle: 'Open - Bids: 3', meta: 'Open', tone: 'tone-amber' },
    { title: 'Data Science RFQ', subtitle: 'Open - Bids: 3', meta: 'Open', tone: 'tone-amber' },
  ];

  readonly exchangePlatforms: ChipItem[] = [
    { label: 'Robinhood', tone: 'tone-emerald' },
    { label: 'SoFi', tone: 'tone-purple' },
    { label: 'Coinbase', tone: 'tone-blue' },
    { label: 'Ethereum', tone: 'tone-indigo' },
  ];

  readonly marketplaceMetrics: Metric[] = [
    { label: 'Listed Items', value: '284', caption: 'Skills - Assets', tone: 'tone-teal' },
    { label: 'Active Orders', value: '18', caption: '$24K value', tone: 'tone-emerald' },
    { label: 'Barter Offers', value: '12', caption: 'Active trades', tone: 'tone-amber' },
    { label: 'My Sales', value: '$8,400', caption: '18%', tone: 'tone-emerald' },
    { label: 'My Purchases', value: '$2,100', caption: 'This month', tone: 'tone-cyan' },
  ];

  readonly marketplaceItems: CardItem[] = [
    { title: 'Full-Stack Dev', subtitle: 'Labor', value: '$120/hr', tone: 'tone-blue' },
    { title: 'Logo Design', subtitle: 'Asset', value: '$250', tone: 'tone-purple' },
    { title: 'Brand Strategy', subtitle: 'Service', value: '$3,500', tone: 'tone-rose' },
    { title: 'React Template', subtitle: 'Artifact', value: '$89', tone: 'tone-cyan' },
    { title: 'Data Analysis', subtitle: 'Labor', value: '$85/hr', tone: 'tone-emerald' },
    { title: 'Legal Review', subtitle: 'Service', value: '$200/hr', tone: 'tone-rose' },
    { title: 'Office Chair', subtitle: 'Barter', value: 'Trade', tone: 'tone-amber' },
    { title: '3D Models Pack', subtitle: 'Asset', value: '$149', tone: 'tone-indigo' },
    { title: 'Content Writing', subtitle: 'Labor', value: '$0.12/wd', tone: 'tone-emerald' },
    { title: 'API Access', subtitle: 'Resource', value: '$29/mo', tone: 'tone-blue' },
    { title: 'Photography Kit', subtitle: 'Barter', value: 'Trade', tone: 'tone-amber' },
    { title: 'Copywriting', subtitle: 'Service', value: '$1,200', tone: 'tone-teal' },
  ];

  readonly marketplaceBarter: CardItem[] = [
    { title: 'Camera gear', subtitle: 'Want: Laptop', meta: 'Open', tone: 'tone-amber' },
    { title: 'Adobe License', subtitle: 'Want: Web dev', meta: 'Open', tone: 'tone-amber' },
    { title: 'Studio Time', subtitle: 'Want: Design work', meta: 'Open', tone: 'tone-amber' },
  ];

  readonly marketplaceOrders: CardItem[] = [
    { title: 'Brand Package', subtitle: 'Active', meta: '$3,500', tone: 'tone-emerald' },
    { title: 'Dev Hours', subtitle: 'Delivered', meta: '$1,200', tone: 'tone-teal' },
    { title: 'Legal Review', subtitle: 'Pending', meta: '$200', tone: 'tone-amber' },
  ];

  readonly marketplacePlatforms: ChipItem[] = [
    { label: 'Behance', tone: 'tone-blue' },
    { label: 'Upwork', tone: 'tone-emerald' },
    { label: 'Fiverr', tone: 'tone-teal' },
    { label: 'Etsy', tone: 'tone-amber' },
  ];

  readonly studioMetrics: Metric[] = [
    { label: 'Ideas', value: '42', caption: 'In development', tone: 'tone-purple' },
    { label: 'Prototypes', value: '8', caption: '3 testing', tone: 'tone-indigo', progress: 36 },
    { label: 'Published Assets', value: '24', caption: 'Available', tone: 'tone-teal' },
    { label: 'Notes / Binders', value: '138', caption: 'Organized', tone: 'tone-cyan' },
  ];

  readonly studioIdeas: CardItem[] = [
    { title: 'Mobile App Concept', subtitle: 'Prototype', status: 'Concept', tone: 'tone-purple' },
    { title: 'Brand System', subtitle: 'Asset', status: 'Ready', tone: 'tone-emerald' },
    { title: 'Tokenomics v3', subtitle: 'Blueprint', status: 'Draft', tone: 'tone-blue' },
    { title: 'Landing Page', subtitle: 'Mockup', status: 'Draft', tone: 'tone-rose' },
    { title: 'AI Agent UX', subtitle: 'Design', status: 'Active', tone: 'tone-amber' },
    { title: 'Community RFC', subtitle: 'Draft', status: 'Draft', tone: 'tone-amber' },
    { title: 'Data Schema', subtitle: 'Blueprint', status: 'Active', tone: 'tone-indigo' },
    { title: 'API Spec v2', subtitle: 'Document', status: 'Active', tone: 'tone-cyan' },
  ];

  readonly studioTestbeds: CardItem[] = [
    { title: 'A/B Test: Onboarding', subtitle: 'Testbed', tone: 'tone-purple' },
    { title: 'Perf Benchmark', subtitle: 'Testbed', tone: 'tone-indigo' },
    { title: 'API Load Test', subtitle: 'Testbed', tone: 'tone-cyan' },
    { title: 'UX Usability Study', subtitle: 'Testbed', tone: 'tone-purple' },
    { title: 'Payment Flow Test', subtitle: 'Testbed', tone: 'tone-emerald' },
    { title: 'AI Prompt Eval', subtitle: 'Testbed', tone: 'tone-rose' },
  ];

  readonly studioToolsets: CardItem[] = [
    { title: 'Design Tools', subtitle: 'Figma - Framer', meta: 'Active', tone: 'tone-purple' },
    { title: 'Dev Stack', subtitle: 'Node - React - PG', meta: 'Active', tone: 'tone-teal' },
    { title: 'AI Toolkit', subtitle: 'Claude - GPT - Grok', meta: 'Active', tone: 'tone-amber' },
    { title: 'Analytics', subtitle: 'Posthog - Mixpanel', meta: 'Active', tone: 'tone-cyan' },
  ];

  readonly studioFiles: CardItem[] = [
    { title: 'Binders', tone: 'tone-blue' },
    { title: 'Books', tone: 'tone-purple' },
    { title: 'Content', tone: 'tone-emerald' },
    { title: 'Files', tone: 'tone-cyan' },
  ];

  readonly studioPlatforms: ChipItem[] = [
    { label: 'Google Drive', tone: 'tone-blue' },
    { label: 'Figma', tone: 'tone-purple' },
    { label: 'Notion', tone: 'tone-teal' },
    { label: 'MS Teams', tone: 'tone-cyan' },
  ];

  readonly communityMetrics: Metric[] = [
    { label: 'Members', value: '8,420', caption: '12%', tone: 'tone-rose' },
    { label: 'Active Spaces', value: '34', caption: '12 rooms open', tone: 'tone-teal' },
    { label: 'Posts Today', value: '284', caption: '18% vs yesterday', tone: 'tone-purple' },
    { label: 'DMs Unread', value: '7', caption: 'Priority', tone: 'tone-amber' },
  ];

  readonly communityFeeds: FeedItem[] = [
    {
      author: 'Jordan C.',
      message: 'New project launched! Brand redesign for a VC-backed startup.',
      time: '2 min ago',
      stats: '12 replies • 48 reactions',
    },
    {
      author: 'Maria S.',
      message: 'Coop milestone: $1M ARR achieved. Proud of our 35-member team.',
      time: '8 min ago',
      stats: '48 replies • 96 reactions',
    },
    {
      author: 'Devon P.',
      message: 'Side project update: photography clients x3 this week.',
      time: '22 min ago',
      stats: '23 replies • 41 reactions',
    },
    {
      author: 'Priya N.',
      message: 'Investment club vote: 87% approve new property acquisition.',
      time: '1 hr ago',
      stats: '31 replies • 75 reactions',
    },
  ];

  readonly communitySpaces: CardItem[] = [
    { title: '#Office', subtitle: '21 online', meta: 'Live', tone: 'tone-blue' },
    { title: '#Finance', subtitle: '18 online', meta: 'Live', tone: 'tone-emerald' },
    { title: '#Studio', subtitle: '31 online', meta: 'Live', tone: 'tone-purple' },
    { title: '#General', subtitle: '12 online', meta: 'Live', tone: 'tone-cyan' },
    { title: '#Coops', subtitle: '9 online', meta: 'Live', tone: 'tone-rose' },
  ];

  readonly communityDms: CardItem[] = [
    { title: 'Jordan C.', subtitle: 'Hey, quick question...', tone: 'tone-blue' },
    { title: 'Maria S.', subtitle: 'Hey, quick question...', tone: 'tone-emerald' },
    { title: 'Devon P.', subtitle: 'Hey, quick question...', tone: 'tone-amber' },
  ];

  readonly communityPlatforms: ChipItem[] = [
    { label: 'Slack', tone: 'tone-purple' },
    { label: 'Zoom', tone: 'tone-emerald' },
    { label: 'Discord', tone: 'tone-blue' },
    { label: 'WhatsApp', tone: 'tone-amber' },
  ];

  readonly developerMetrics: Metric[] = [
    { label: 'API Calls (30d)', value: '284K', caption: '34%', tone: 'tone-purple' },
    { label: 'Active Keys', value: '5', caption: '2 production', tone: 'tone-blue' },
    { label: 'Webhooks', value: '12', caption: '8 active', tone: 'tone-teal', progress: 66 },
    { label: 'SDK Downloads', value: '1,240', caption: '28%', tone: 'tone-indigo' },
  ];

  readonly developerApi: ApiRow[] = [
    { method: 'POST', path: '/api/v1/portfolio', description: 'Create portfolio item', tone: 'tone-emerald' },
    { method: 'GET', path: '/api/v1/projects', description: 'List all projects', tone: 'tone-blue' },
    { method: 'PUT', path: '/api/v1/exchange/bids', description: 'Update a bid', tone: 'tone-amber' },
    { method: 'DELETE', path: '/api/v1/wallet/tokens', description: 'Remove token', tone: 'tone-rose' },
    { method: 'POST', path: '/api/v1/community/post', description: 'Create community post', tone: 'tone-purple' },
    { method: 'GET', path: '/api/v1/ai/context', description: 'Get AI context window', tone: 'tone-cyan' },
  ];

  readonly developerExtensions: CardItem[] = [
    { title: 'Webhooks', subtitle: 'Automation hooks', tone: 'tone-blue' },
    { title: 'OAuth 2.0', subtitle: 'Identity  access', tone: 'tone-emerald' },
    { title: 'Zapier', subtitle: 'Workflow', tone: 'tone-amber' },
    { title: 'n8n', subtitle: 'Pipelines', tone: 'tone-purple' },
    { title: 'Slack Bot', subtitle: 'Chat ops', tone: 'tone-teal' },
    { title: 'GitHub Action', subtitle: 'CI/CD', tone: 'tone-indigo' },
    { title: 'Chrome Ext', subtitle: 'Browser tools', tone: 'tone-rose' },
    { title: 'VS Code Ext', subtitle: 'Dev experience', tone: 'tone-cyan' },
  ];

  readonly developerKeys: CardItem[] = [
    { title: 'prod_k1_****', subtitle: 'Production', meta: 'Active', tone: 'tone-emerald' },
    { title: 'dev_k2_****', subtitle: 'Development', meta: 'Active', tone: 'tone-blue' },
    { title: 'test_k3_****', subtitle: 'Testing', meta: 'Active', tone: 'tone-amber' },
  ];

  readonly developerSdks: ChipItem[] = [
    { label: 'Node.js', tone: 'tone-emerald' },
    { label: 'Python', tone: 'tone-blue' },
    { label: 'Go', tone: 'tone-teal' },
    { label: 'Rust', tone: 'tone-amber' },
  ];

  readonly developerWebhooks: CardItem[] = [
    { title: 'portfolio.created', subtitle: 'Active', meta: 'Live', tone: 'tone-emerald' },
    { title: 'payment.received', subtitle: 'Active', meta: 'Live', tone: 'tone-blue' },
    { title: 'bid.accepted', subtitle: 'Active', meta: 'Live', tone: 'tone-amber' },
    { title: 'project.updated', subtitle: 'Active', meta: 'Live', tone: 'tone-purple' },
  ];

  readonly profileMetrics: Metric[] = [
    { label: 'Reputation', value: '94/100', caption: '3%', tone: 'tone-amber' },
    { label: 'Network', value: '1,240', caption: 'Connections', tone: 'tone-purple' },
    { label: 'Projects Done', value: '48', caption: '8%', tone: 'tone-blue' },
    { label: 'Earnings YTD', value: '$186K', caption: '22%', tone: 'tone-emerald' },
  ];

  readonly profilePersonas: CardItem[] = [
    { title: 'Developer', subtitle: 'Full-stack - Node - React', tone: 'tone-blue' },
    { title: 'Designer', subtitle: 'Brand - UX - Systems', tone: 'tone-purple' },
    { title: 'Strategist', subtitle: 'Product - Growth', tone: 'tone-emerald' },
  ];

  readonly profileSettings: CardItem[] = [
    { title: 'Preferences', tone: 'tone-amber' },
    { title: 'Notifications', tone: 'tone-purple' },
    { title: 'Privacy', tone: 'tone-rose' },
    { title: 'Security', tone: 'tone-emerald' },
    { title: 'Billing', tone: 'tone-amber' },
    { title: 'Integrations', tone: 'tone-teal' },
    { title: 'AI Config', tone: 'tone-indigo' },
    { title: 'Data Export', tone: 'tone-cyan' },
  ];

  readonly profileStats: Metric[] = [
    { label: 'Messages', value: '284', tone: 'tone-rose' },
    { label: 'Proposals', value: '18', tone: 'tone-blue' },
    { label: 'Contributions', value: '142', tone: 'tone-indigo' },
    { label: 'Reviews', value: '37', tone: 'tone-amber' },
    { label: 'Referrals', value: '12', tone: 'tone-emerald' },
  ];

  readonly orgMetrics: Metric[] = [
    { label: 'Organizations', value: '4', caption: 'Member of', tone: 'tone-rose' },
    { label: 'Collectives', value: '2', caption: 'Co-founded', tone: 'tone-blue' },
    { label: 'Teams', value: '3', caption: 'Active', tone: 'tone-teal' },
    { label: 'Total Members', value: '186', caption: 'Across all orgs', tone: 'tone-emerald' },
  ];

  readonly orgCards: CardItem[] = [
    { title: 'Delivery Coop', subtitle: 'Worker Cooperative', meta: '35 members', status: 'Active', tone: 'tone-emerald' },
    { title: 'Design Collective', subtitle: 'Creative Collective', meta: '12 members', status: 'Active', tone: 'tone-purple' },
    { title: 'Invest Club', subtitle: 'Investment Club', meta: '22 members', status: 'Active', tone: 'tone-blue' },
    { title: 'Dev DAO', subtitle: 'Autonomous Org', meta: '88 members', status: 'Active', tone: 'tone-indigo' },
    { title: 'Consulting Network', subtitle: 'Professional Network', meta: '18 members', status: 'Pending', tone: 'tone-amber' },
    { title: 'Art Cooperative', subtitle: 'Creative Cooperative', meta: '7 members', status: 'Forming', tone: 'tone-rose' },
  ];

  readonly orgProposals: TableRow[] = [
    { cells: ['New vehicle fleet', 'Delivery Coop', '32/35', 'Passed'], badge: 'Passed', badgeTone: 'tone-emerald' },
    { cells: ['Q2 Budget', 'Invest Club', '18/22', 'Active'], badge: 'Active', badgeTone: 'tone-blue' },
    { cells: ['Member Charter', 'Dev DAO', '72/88', 'Active'], badge: 'Active', badgeTone: 'tone-purple' },
    { cells: ['Rate increase', 'Design Coll.', '9/12', 'Draft'], badge: 'Draft', badgeTone: 'tone-amber' },
  ];

  readonly orgRoles: CardItem[] = [
    { title: 'Delivery Coop', subtitle: 'Co-Founder', meta: 'Lead', tone: 'tone-emerald' },
    { title: 'Invest Club', subtitle: 'Organizer', meta: 'Active', tone: 'tone-blue' },
    { title: 'Dev DAO', subtitle: 'Member', meta: 'Active', tone: 'tone-indigo' },
    { title: 'Design Coll.', subtitle: 'Member', meta: 'Active', tone: 'tone-purple' },
  ];

  readonly orgCapTables: CardItem[] = [
    { title: 'JC', subtitle: '18.5%', meta: 'Equity', tone: 'tone-blue' },
    { title: 'MS', subtitle: '14.2%', meta: 'Equity', tone: 'tone-emerald' },
    { title: 'DP', subtitle: '8.8%', meta: 'Equity', tone: 'tone-amber' },
    { title: 'PN', subtitle: '6.4%', meta: 'Equity', tone: 'tone-rose' },
  ];

  readonly legalMetrics: Metric[] = [
    { label: 'Active Contracts', value: '14', caption: '3 expiring soon', tone: 'tone-rose' },
    { label: 'IP Assets', value: '8', caption: 'Patents - TM - CR', tone: 'tone-purple' },
    { label: 'Compliance Items', value: '6', caption: '2 action needed', tone: 'tone-amber' },
    { label: 'Audit Status', value: 'Clean', caption: 'Last: Mar 2026', tone: 'tone-emerald' },
  ];

  readonly legalIp: CardItem[] = [
    { title: 'Trademark', subtitle: 'Kogi', status: 'Registered', tone: 'tone-rose' },
    { title: 'Patent', subtitle: 'Kogi OS', status: 'Pending', tone: 'tone-amber' },
    { title: 'Copyright', subtitle: 'Logo System', status: 'Active', tone: 'tone-emerald' },
    { title: 'Copyright', subtitle: 'API Spec', status: 'Active', tone: 'tone-rose' },
  ];

  readonly legalContracts: TableRow[] = [
    { cells: ['MSA - TechCorp', 'TechCorp Inc.', '$120K', 'Dec 2026', 'Active'], badge: 'Active', badgeTone: 'tone-emerald' },
    { cells: ['NDA - StartupX', 'StartupX LLC', '-', 'Jun 2026', 'Active'], badge: 'Active', badgeTone: 'tone-emerald' },
    { cells: ['Coop Charter', 'Members (35)', '-', '-', 'Active'], badge: 'Active', badgeTone: 'tone-emerald' },
    { cells: ['Dev Contract', 'DevDAO', '$48K', 'Mar 2026', 'Expiring'], badge: 'Expiring', badgeTone: 'tone-amber' },
    { cells: ['License Agmt.', '3rd Party', '$8K/yr', 'Dec 2026', 'Active'], badge: 'Active', badgeTone: 'tone-emerald' },
  ];

  readonly legalCompliance: CardItem[] = [
    { title: 'GDPR Compliance', subtitle: 'Complete', status: 'Complete', tone: 'tone-emerald' },
    { title: 'SOC 2 Type II', subtitle: 'In Progress', status: 'In Progress', tone: 'tone-amber' },
    { title: 'Tax Compliance', subtitle: 'Complete', status: 'Complete', tone: 'tone-emerald' },
    { title: 'AML / KYC', subtitle: 'Review Needed', status: 'Review', tone: 'tone-rose' },
    { title: 'Data Residency', subtitle: 'Complete', status: 'Complete', tone: 'tone-teal' },
    { title: 'IP Audit', subtitle: 'Scheduled', status: 'Scheduled', tone: 'tone-blue' },
  ];

  readonly legalUpcoming: CardItem[] = [
    { title: 'DevDAO contract expires', subtitle: 'In 7 days', meta: 'Due', tone: 'tone-rose' },
    { title: 'IP audit scheduled', subtitle: 'In 14 days', meta: 'Scheduled', tone: 'tone-blue' },
    { title: 'Q1 tax filing', subtitle: 'In 22 days', meta: 'Due', tone: 'tone-amber' },
    { title: 'NDA renewal', subtitle: 'In 45 days', meta: 'Upcoming', tone: 'tone-purple' },
  ];

  readonly legalActions: CardItem[] = [
    { title: 'Draft NDA', tone: 'tone-blue' },
    { title: 'Submit Patent', tone: 'tone-amber' },
    { title: 'Renew Contract', tone: 'tone-rose' },
    { title: 'File Compliance', tone: 'tone-emerald' },
  ];

  setScreen(id: ScreenId): void {
    this.activeScreen.set(id);
  }
}

bootstrapApplication(AppComponent).catch((err) => console.error(err));
  readonly workspaceMetrics: Metric[] = [
    { label: 'Open Tasks', value: '34', caption: '8 overdue', tone: 'tone-blue' },
    { label: 'Sprints Active', value: '3', caption: '2 on track', tone: 'tone-teal', progress: 60 },
    { label: 'Blocked', value: '5', caption: 'Needs action', tone: 'tone-amber' },
    { label: 'Done This Week', value: '21', caption: '+15%', tone: 'tone-emerald' },
  ];

  readonly workspaceKanban: KanbanColumn[] = [
    {
      title: 'Backlog',
      tone: 'tone-blue',
      items: ['Auth redesign', 'Payment flow', 'API docs', 'Onboard UX'],
    },
    {
      title: 'In Progress',
      tone: 'tone-amber',
      items: ['Mobile nav', 'Token calc', 'Legal review', 'Test suite'],
    },
    {
      title: 'Review',
      tone: 'tone-purple',
      items: ['Brand deck', 'Cop model', 'RFC-009', 'Data model'],
    },
    {
      title: 'Done',
      tone: 'tone-emerald',
      items: ['Login fix', 'CSV export', 'Error states', 'Docs v2'],
    },
  ];

  readonly workspaceCalendar: CardItem[] = [
    { title: 'Client sync 10am', subtitle: 'Mon Mar 10', meta: 'Today', tone: 'tone-blue' },
    { title: 'Sprint review 3pm', subtitle: 'Mon Mar 10', meta: 'Today', tone: 'tone-teal' },
    { title: 'Payment due', subtitle: 'Tue Mar 11', meta: 'Tomorrow', tone: 'tone-amber' },
    { title: 'Board meeting', subtitle: 'Wed Mar 12', meta: 'Upcoming', tone: 'tone-rose' },
    { title: 'Milestone delivery', subtitle: 'Thu Mar 13', meta: 'Upcoming', tone: 'tone-purple' },
  ];

  readonly workspaceRoadmaps: Array<CardItem & { progress: number }> = [
    { title: 'Q1 2026 - Foundation', meta: '65%', progress: 65, tone: 'tone-blue' },
    { title: 'Q2 2026 - Growth', meta: '33%', progress: 33, tone: 'tone-teal' },
    { title: 'Q3 2026 - Scale', meta: '12%', progress: 12, tone: 'tone-rose' },
  ];

  readonly workspaceGantt: GanttRow[] = [
    {
      label: 'Alpha Platform',
      bars: [
        { label: 'Alpha', tone: 'tone-blue', start: 10, width: 55 },
        { label: 'Alpha', tone: 'tone-teal', start: 70, width: 15 },
      ],
    },
    {
      label: 'Beta Campaign',
      bars: [{ label: 'Beta', tone: 'tone-amber', start: 25, width: 40 }],
    },
    {
      label: 'Gamma Research',
      bars: [{ label: 'Gamma', tone: 'tone-emerald', start: 5, width: 70 }],
    },
    {
      label: 'Delta Ops',
      bars: [{ label: 'Delta', tone: 'tone-purple', start: 40, width: 35 }],
    },
    {
      label: 'Epsilon Design',
      bars: [{ label: 'Epsilon', tone: 'tone-rose', start: 55, width: 30 }],
    },
  ];

  readonly timelineMetrics: Metric[] = [
    { label: 'Milestones', value: '24', caption: 'Q1 2026', tone: 'tone-blue' },
    { label: 'Scheduled Events', value: '18', caption: 'Next 30 days', tone: 'tone-teal' },
    { label: 'Overdue Items', value: '3', caption: 'Needs attention', tone: 'tone-rose' },
    { label: 'Completion Rate', value: '78%', caption: 'This quarter', tone: 'tone-emerald', progress: 78 },
  ];

  readonly timelineMaster: GanttRow[] = [
    {
      label: 'Platform MVP',
      bars: [
        { label: 'MVP', tone: 'tone-blue', start: 5, width: 55 },
        { label: 'MVP', tone: 'tone-purple', start: 62, width: 25 },
      ],
    },
    {
      label: 'Design System',
      bars: [{ label: 'Design', tone: 'tone-purple', start: 15, width: 70 }],
    },
    {
      label: 'API v1 Launch',
      bars: [{ label: 'API', tone: 'tone-emerald', start: 30, width: 40 }],
    },
    {
      label: 'Beta Onboarding',
      bars: [{ label: 'Beta', tone: 'tone-amber', start: 45, width: 35 }],
    },
    {
      label: 'Legal Review',
      bars: [{ label: 'Legal', tone: 'tone-rose', start: 20, width: 25 }],
    },
    {
      label: 'Exchange Module',
      bars: [{ label: 'Exchange', tone: 'tone-cyan', start: 60, width: 30 }],
    },
    {
      label: 'Community Beta',
      bars: [{ label: 'Community', tone: 'tone-rose', start: 68, width: 26 }],
    },
    {
      label: 'AI Agent v1',
      bars: [{ label: 'AI', tone: 'tone-indigo', start: 52, width: 30 }],
    },
  ];

  readonly timelineEvents: CardItem[] = [
    { title: 'Client sync', subtitle: 'Mon Mar 10', tone: 'tone-blue' },
    { title: 'Sprint review', subtitle: 'Mon Mar 10', tone: 'tone-purple' },
    { title: 'Board meeting', subtitle: 'Wed Mar 12', tone: 'tone-amber' },
    { title: 'Milestone gate', subtitle: 'Fri Mar 14', tone: 'tone-rose' },
    { title: 'Tax filing due', subtitle: 'Mar 22', tone: 'tone-emerald' },
    { title: 'Q1 close', subtitle: 'Mar 31', tone: 'tone-teal' },
  ];

  readonly timelineRoadmap: Array<CardItem & { progress: number }> = [
    { title: 'Q1 Foundation', meta: '65%', progress: 65, tone: 'tone-blue' },
    { title: 'Q2 Growth', meta: '33%', progress: 33, tone: 'tone-teal' },
    { title: 'Q3 Scale', meta: '12%', progress: 12, tone: 'tone-rose' },
    { title: 'Q4 Expand', meta: '4%', progress: 4, tone: 'tone-amber' },
  ];

  readonly timelineDeadlines: CardItem[] = [
    { title: 'Legal Review', subtitle: 'In 7 days', meta: 'Due', tone: 'tone-rose' },
    { title: 'API Launch', subtitle: 'In 12 days', meta: 'Due', tone: 'tone-teal' },
    { title: 'Board Deck', subtitle: 'In 18 days', meta: 'Due', tone: 'tone-amber' },
    { title: 'Q1 Close', subtitle: 'In 22 days', meta: 'Due', tone: 'tone-blue' },
  ];

  readonly strategyMetrics: Metric[] = [
    { label: 'Strategic OKRs', value: '5', caption: '3 on track', tone: 'tone-purple', progress: 60 },
    { label: 'Tactical Initiatives', value: '18', caption: '11 active', tone: 'tone-blue' },
    { label: 'Ops Processes', value: '24', caption: 'Documented', tone: 'tone-emerald' },
    { label: 'Governance Items', value: '7', caption: '2 pending vote', tone: 'tone-amber' },
  ];

  readonly strategyTree: CardItem[] = [
    {
      title: 'Vision',
      subtitle: 'Be the OS for independent work',
      meta: 'Strategic',
      tone: 'tone-purple',
    },
    {
      title: 'Mission',
      subtitle: 'Unify tools - empower workers',
      meta: 'Strategic',
      tone: 'tone-blue',
    },
    {
      title: 'OBJ 1',
      subtitle: 'Reach 50K users by Q4',
      meta: 'Objective',
      tone: 'tone-teal',
    },
    {
      title: 'OBJ 2',
      subtitle: '$1M ARR by Q3',
      meta: 'Objective',
      tone: 'tone-emerald',
    },
    {
      title: 'OBJ 3',
      subtitle: 'Community 10K members',
      meta: 'Objective',
      tone: 'tone-rose',
    },
    {
      title: 'KR 1',
      subtitle: 'NPS > 60',
      meta: 'Key Result',
      tone: 'tone-amber',
    },
  ];

  readonly strategyTactics: CardItem[] = [
    { title: 'Pricing Experiments', subtitle: 'Tactical - Active', tone: 'tone-blue' },
    { title: 'Onboarding Funnel', subtitle: 'Tactical - Active', tone: 'tone-teal' },
    { title: 'Partnership Program', subtitle: 'Tactical - Active', tone: 'tone-emerald' },
    { title: 'Content Marketing', subtitle: 'Tactical - Active', tone: 'tone-amber' },
    { title: 'API Integrations', subtitle: 'Tactical - Active', tone: 'tone-purple' },
    { title: 'Support Playbook', subtitle: 'Tactical - Active', tone: 'tone-rose' },
    { title: 'Legal Frameworks', subtitle: 'Tactical - Active', tone: 'tone-rose' },
    { title: 'Data Infrastructure', subtitle: 'Tactical - Active', tone: 'tone-amber' },
  ];

  readonly strategyGovernance: CardItem[] = [
    { title: 'RFC-009', subtitle: 'Under Review', meta: 'Pending', tone: 'tone-rose' },
    { title: 'Equity Policy', subtitle: 'Approved', meta: 'Approved', tone: 'tone-emerald' },
    { title: 'Data Retention', subtitle: 'Pending Vote', meta: 'Pending', tone: 'tone-amber' },
    { title: 'Member Charter', subtitle: 'Draft', meta: 'Draft', tone: 'tone-purple' },
  ];

  readonly strategyOkrs: Array<CardItem & { progress: number }> = [
    { title: 'User Growth', meta: '78%', progress: 78, tone: 'tone-blue' },
    { title: 'Revenue', meta: '45%', progress: 45, tone: 'tone-emerald' },
    { title: 'Community', meta: '62%', progress: 62, tone: 'tone-rose' },
  ];
  readonly bankMetrics: Metric[] = [
    { label: 'Total Balance', value: '$142,800', caption: '+8%', tone: 'tone-emerald' },
    { label: 'Operations', value: '$28,400', caption: 'Ops wallet', tone: 'tone-teal', progress: 55 },
    { label: 'Investments', value: '$89,200', caption: 'Portfolio', tone: 'tone-blue' },
    { label: 'Trading', value: '$18,300', caption: 'Exchange', tone: 'tone-amber' },
    { label: 'Personal', value: '$6,900', caption: 'Spending', tone: 'tone-cyan', progress: 40 },
  ];

  readonly bankWallets: CardItem[] = [
    { title: 'Personal Spending', subtitle: 'Daily - bills', value: '$6,900', tone: 'tone-cyan' },
    { title: 'Operations', subtitle: 'Payroll - tools', value: '$28,400', tone: 'tone-emerald' },
    { title: 'Investment', subtitle: 'Stocks - bonds', value: '$89,200', tone: 'tone-blue' },
    { title: 'Trading', subtitle: 'Crypto - tokens', value: '$18,300', tone: 'tone-amber' },
    { title: 'Marketplace', subtitle: 'Purchases', value: '$4,200', tone: 'tone-teal' },
    { title: 'Coop Pool', subtitle: 'Collective capital', value: '$124,500', tone: 'tone-purple' },
  ];

  readonly bankFundraising: CardItem[] = [
    { title: 'Seed Round', subtitle: '$450K raised', meta: '72% of goal', progress: 72, tone: 'tone-blue' },
    { title: 'Community Bond', subtitle: '$120K raised', meta: '48% of goal', progress: 48, tone: 'tone-emerald' },
    { title: 'Equipment Lease', subtitle: '$28K raised', meta: '96% of goal', progress: 96, tone: 'tone-amber' },
  ];

  readonly bankPlatforms: ChipItem[] = [
    { label: 'Stripe', tone: 'tone-blue' },
    { label: 'Wells Fargo', tone: 'tone-rose' },
    { label: 'Chase', tone: 'tone-teal' },
    { label: 'GoFundMe', tone: 'tone-emerald' },
    { label: 'Patreon', tone: 'tone-amber' },
  ];

  readonly bankTransactions: CardItem[] = [
    { title: 'Stripe payout', subtitle: '+$3,200', meta: 'Cleared', tone: 'tone-emerald' },
    { title: 'Tool subscription', subtitle: '-$49', meta: 'Posted', tone: 'tone-rose' },
    { title: 'Coop distribution', subtitle: '+$840', meta: 'Posted', tone: 'tone-teal' },
    { title: 'Tax payment', subtitle: '-$2,100', meta: 'Pending', tone: 'tone-amber' },
    { title: 'Invoice paid', subtitle: '+$8,400', meta: 'Cleared', tone: 'tone-blue' },
  ];

  readonly bankTaxes: CardItem[] = [
    { title: 'Income Tax', value: '$18,400', tone: 'tone-rose' },
    { title: 'Self-Employ', value: '$4,200', tone: 'tone-amber' },
    { title: 'Deductions', value: '-$6,800', tone: 'tone-emerald' },
    { title: 'Estimated', value: '$15,800', tone: 'tone-amber' },
  ];

  readonly exchangeMetrics: Metric[] = [
    { label: 'Open Bids', value: '7', caption: '$42K value', tone: 'tone-amber' },
    { label: 'Active Deals', value: '3', caption: '$18K escrow', tone: 'tone-emerald' },
    { label: 'Proposals Sent', value: '12', caption: '4 responded', tone: 'tone-blue' },
    { label: 'Trading Volume', value: '$284K', caption: 'Last 30 days', tone: 'tone-amber' },
  ];

  readonly exchangeBids: TableRow[] = [
    { cells: ['Brand Identity', 'Offer', '$8,400', 'Active', 'Studio Co'], badge: 'Active', badgeTone: 'tone-emerald' },
    { cells: ['API Integration', 'Bid', '$12,000', 'Pending', 'TechCorp'], badge: 'Pending', badgeTone: 'tone-amber' },
    { cells: ['UX Audit', 'Proposal', '$3,200', 'Countered', 'StartupX'], badge: 'Countered', badgeTone: 'tone-rose' },
    { cells: ['Data Pipeline', 'Offer', '$22,000', 'Due Diligence', 'Enterprise Y'], badge: 'Due Diligence', badgeTone: 'tone-blue' },
    { cells: ['Community Module', 'Bid', '$5,800', 'Active', 'DevDAO'], badge: 'Active', badgeTone: 'tone-emerald' },
  ];

  readonly exchangePipeline: CardItem[] = [
    { title: 'Enterprise Y Deal', subtitle: 'Due Diligence', value: '$22,000', progress: 70, tone: 'tone-amber' },
    { title: 'API Integration', subtitle: 'Escrow Active', value: '$12,000', progress: 55, tone: 'tone-emerald' },
    { title: 'DevDAO Proposal', subtitle: 'Negotiation', value: '$5,800', progress: 35, tone: 'tone-blue' },
  ];

  readonly exchangeRequests: CardItem[] = [
    { title: 'UX Design RFP', subtitle: 'Open - Bids: 3', meta: 'Open', tone: 'tone-amber' },
    { title: 'Backend Dev RFQ', subtitle: 'Open - Bids: 3', meta: 'Open', tone: 'tone-amber' },
    { title: 'Legal Review RFP', subtitle: 'Open - Bids: 3', meta: 'Open', tone: 'tone-amber' },
    { title: 'Data Science RFQ', subtitle: 'Open - Bids: 3', meta: 'Open', tone: 'tone-amber' },
  ];

  readonly exchangePlatforms: ChipItem[] = [
    { label: 'Robinhood', tone: 'tone-emerald' },
    { label: 'SoFi', tone: 'tone-purple' },
    { label: 'Coinbase', tone: 'tone-blue' },
    { label: 'Ethereum', tone: 'tone-indigo' },
  ];

  readonly marketplaceMetrics: Metric[] = [
    { label: 'Listed Items', value: '284', caption: 'Skills - Assets', tone: 'tone-teal' },
    { label: 'Active Orders', value: '18', caption: '$24K value', tone: 'tone-emerald' },
    { label: 'Barter Offers', value: '12', caption: 'Active trades', tone: 'tone-amber' },
    { label: 'My Sales', value: '$8,400', caption: '+18%', tone: 'tone-emerald' },
    { label: 'My Purchases', value: '$2,100', caption: 'This month', tone: 'tone-cyan' },
  ];

  readonly marketplaceItems: CardItem[] = [
    { title: 'Full-Stack Dev', subtitle: 'Labor', value: '$120/hr', tone: 'tone-blue' },
    { title: 'Logo Design', subtitle: 'Asset', value: '$250', tone: 'tone-purple' },
    { title: 'Brand Strategy', subtitle: 'Service', value: '$3,500', tone: 'tone-rose' },
    { title: 'React Template', subtitle: 'Artifact', value: '$89', tone: 'tone-cyan' },
    { title: 'Data Analysis', subtitle: 'Labor', value: '$85/hr', tone: 'tone-emerald' },
    { title: 'Legal Review', subtitle: 'Service', value: '$200/hr', tone: 'tone-rose' },
    { title: 'Office Chair', subtitle: 'Barter', value: 'Trade', tone: 'tone-amber' },
    { title: '3D Models Pack', subtitle: 'Asset', value: '$149', tone: 'tone-indigo' },
    { title: 'Content Writing', subtitle: 'Labor', value: '$0.12/wd', tone: 'tone-emerald' },
    { title: 'API Access', subtitle: 'Resource', value: '$29/mo', tone: 'tone-blue' },
    { title: 'Photography Kit', subtitle: 'Barter', value: 'Trade', tone: 'tone-amber' },
    { title: 'Copywriting', subtitle: 'Service', value: '$1,200', tone: 'tone-teal' },
  ];

  readonly marketplaceBarter: CardItem[] = [
    { title: 'Camera gear', subtitle: 'Want: Laptop', meta: 'Open', tone: 'tone-amber' },
    { title: 'Adobe License', subtitle: 'Want: Web dev', meta: 'Open', tone: 'tone-amber' },
    { title: 'Studio Time', subtitle: 'Want: Design work', meta: 'Open', tone: 'tone-amber' },
  ];

  readonly marketplaceOrders: CardItem[] = [
    { title: 'Brand Package', subtitle: 'Active', meta: '$3,500', tone: 'tone-emerald' },
    { title: 'Dev Hours', subtitle: 'Delivered', meta: '$1,200', tone: 'tone-teal' },
    { title: 'Legal Review', subtitle: 'Pending', meta: '$200', tone: 'tone-amber' },
  ];

  readonly marketplacePlatforms: ChipItem[] = [
    { label: 'Behance', tone: 'tone-blue' },
    { label: 'Upwork', tone: 'tone-emerald' },
    { label: 'Fiverr', tone: 'tone-teal' },
    { label: 'Etsy', tone: 'tone-amber' },
  ];
  readonly studioMetrics: Metric[] = [
    { label: 'Ideas', value: '42', caption: 'In development', tone: 'tone-purple' },
    { label: 'Prototypes', value: '8', caption: '3 testing', tone: 'tone-indigo', progress: 36 },
    { label: 'Published Assets', value: '24', caption: 'Available', tone: 'tone-teal' },
    { label: 'Notes / Binders', value: '138', caption: 'Organized', tone: 'tone-cyan' },
  ];

  readonly studioIdeas: CardItem[] = [
    { title: 'Mobile App Concept', subtitle: 'Prototype', status: 'Concept', tone: 'tone-purple' },
    { title: 'Brand System', subtitle: 'Asset', status: 'Ready', tone: 'tone-emerald' },
    { title: 'Tokenomics v3', subtitle: 'Blueprint', status: 'Draft', tone: 'tone-blue' },
    { title: 'Landing Page', subtitle: 'Mockup', status: 'Draft', tone: 'tone-rose' },
    { title: 'AI Agent UX', subtitle: 'Design', status: 'Active', tone: 'tone-amber' },
    { title: 'Community RFC', subtitle: 'Draft', status: 'Draft', tone: 'tone-amber' },
    { title: 'Data Schema', subtitle: 'Blueprint', status: 'Active', tone: 'tone-indigo' },
    { title: 'API Spec v2', subtitle: 'Document', status: 'Active', tone: 'tone-cyan' },
  ];

  readonly studioTestbeds: CardItem[] = [
    { title: 'A/B Test: Onboarding', subtitle: 'Testbed', tone: 'tone-purple' },
    { title: 'Perf Benchmark', subtitle: 'Testbed', tone: 'tone-indigo' },
    { title: 'API Load Test', subtitle: 'Testbed', tone: 'tone-cyan' },
    { title: 'UX Usability Study', subtitle: 'Testbed', tone: 'tone-purple' },
    { title: 'Payment Flow Test', subtitle: 'Testbed', tone: 'tone-emerald' },
    { title: 'AI Prompt Eval', subtitle: 'Testbed', tone: 'tone-rose' },
  ];

  readonly studioToolsets: CardItem[] = [
    { title: 'Design Tools', subtitle: 'Figma - Framer', meta: 'Active', tone: 'tone-purple' },
    { title: 'Dev Stack', subtitle: 'Node - React - PG', meta: 'Active', tone: 'tone-teal' },
    { title: 'AI Toolkit', subtitle: 'Claude - GPT - Grok', meta: 'Active', tone: 'tone-amber' },
    { title: 'Analytics', subtitle: 'Posthog - Mixpanel', meta: 'Active', tone: 'tone-cyan' },
  ];

  readonly studioFiles: CardItem[] = [
    { title: 'Binders', tone: 'tone-blue' },
    { title: 'Books', tone: 'tone-purple' },
    { title: 'Content', tone: 'tone-emerald' },
    { title: 'Files', tone: 'tone-cyan' },
  ];

  readonly studioPlatforms: ChipItem[] = [
    { label: 'Google Drive', tone: 'tone-blue' },
    { label: 'Figma', tone: 'tone-purple' },
    { label: 'Notion', tone: 'tone-teal' },
    { label: 'MS Teams', tone: 'tone-cyan' },
  ];

  readonly communityMetrics: Metric[] = [
    { label: 'Members', value: '8,420', caption: '+12%', tone: 'tone-rose' },
    { label: 'Active Spaces', value: '34', caption: '12 rooms open', tone: 'tone-teal' },
    { label: 'Posts Today', value: '284', caption: '+18% vs yesterday', tone: 'tone-purple' },
    { label: 'DMs Unread', value: '7', caption: 'Priority', tone: 'tone-amber' },
  ];

  readonly communityFeeds: FeedItem[] = [
    {
      author: 'Jordan C.',
      message: 'New project launched! Brand redesign for a VC-backed startup.',
      time: '2 min ago',
      stats: '12 replies | 48 reactions',
    },
    {
      author: 'Maria S.',
      message: 'Coop milestone: $1M ARR achieved. Proud of our 35-member team.',
      time: '8 min ago',
      stats: '48 replies | 96 reactions',
    },
    {
      author: 'Devon P.',
      message: 'Side project update: photography clients x3 this week.',
      time: '22 min ago',
      stats: '23 replies | 41 reactions',
    },
    {
      author: 'Priya N.',
      message: 'Investment club vote: 87% approve new property acquisition.',
      time: '1 hr ago',
      stats: '31 replies | 75 reactions',
    },
  ];

  readonly communitySpaces: CardItem[] = [
    { title: '#Office', subtitle: '21 online', meta: 'Live', tone: 'tone-blue' },
    { title: '#Finance', subtitle: '18 online', meta: 'Live', tone: 'tone-emerald' },
    { title: '#Studio', subtitle: '31 online', meta: 'Live', tone: 'tone-purple' },
    { title: '#General', subtitle: '12 online', meta: 'Live', tone: 'tone-cyan' },
    { title: '#Coops', subtitle: '9 online', meta: 'Live', tone: 'tone-rose' },
  ];

  readonly communityDms: CardItem[] = [
    { title: 'Jordan C.', subtitle: 'Hey, quick question...', tone: 'tone-blue' },
    { title: 'Maria S.', subtitle: 'Hey, quick question...', tone: 'tone-emerald' },
    { title: 'Devon P.', subtitle: 'Hey, quick question...', tone: 'tone-amber' },
  ];

  readonly communityPlatforms: ChipItem[] = [
    { label: 'Slack', tone: 'tone-purple' },
    { label: 'Zoom', tone: 'tone-emerald' },
    { label: 'Discord', tone: 'tone-blue' },
    { label: 'WhatsApp', tone: 'tone-amber' },
  ];

  readonly developerMetrics: Metric[] = [
    { label: 'API Calls (30d)', value: '284K', caption: '+34%', tone: 'tone-purple' },
    { label: 'Active Keys', value: '5', caption: '2 production', tone: 'tone-blue' },
    { label: 'Webhooks', value: '12', caption: '8 active', tone: 'tone-teal', progress: 66 },
    { label: 'SDK Downloads', value: '1,240', caption: '+28%', tone: 'tone-indigo' },
  ];

  readonly developerApi: ApiRow[] = [
    { method: 'POST', path: '/api/v1/portfolio', description: 'Create portfolio item', tone: 'tone-emerald' },
    { method: 'GET', path: '/api/v1/projects', description: 'List all projects', tone: 'tone-blue' },
    { method: 'PUT', path: '/api/v1/exchange/bids', description: 'Update a bid', tone: 'tone-amber' },
    { method: 'DELETE', path: '/api/v1/wallet/tokens', description: 'Remove token', tone: 'tone-rose' },
    { method: 'POST', path: '/api/v1/community/post', description: 'Create community post', tone: 'tone-purple' },
    { method: 'GET', path: '/api/v1/ai/context', description: 'Get AI context window', tone: 'tone-cyan' },
  ];

  readonly developerExtensions: CardItem[] = [
    { title: 'Webhooks', subtitle: 'Automation hooks', tone: 'tone-blue' },
    { title: 'OAuth 2.0', subtitle: 'Identity + access', tone: 'tone-emerald' },
    { title: 'Zapier', subtitle: 'Workflow', tone: 'tone-amber' },
    { title: 'n8n', subtitle: 'Pipelines', tone: 'tone-purple' },
    { title: 'Slack Bot', subtitle: 'Chat ops', tone: 'tone-teal' },
    { title: 'GitHub Action', subtitle: 'CI/CD', tone: 'tone-indigo' },
    { title: 'Chrome Ext', subtitle: 'Browser tools', tone: 'tone-rose' },
    { title: 'VS Code Ext', subtitle: 'Dev experience', tone: 'tone-cyan' },
  ];

  readonly developerKeys: CardItem[] = [
    { title: 'prod_k1_****', subtitle: 'Production', meta: 'Active', tone: 'tone-emerald' },
    { title: 'dev_k2_****', subtitle: 'Development', meta: 'Active', tone: 'tone-blue' },
    { title: 'test_k3_****', subtitle: 'Testing', meta: 'Active', tone: 'tone-amber' },
  ];

  readonly developerSdks: ChipItem[] = [
    { label: 'Node.js', tone: 'tone-emerald' },
    { label: 'Python', tone: 'tone-blue' },
    { label: 'Go', tone: 'tone-teal' },
    { label: 'Rust', tone: 'tone-amber' },
  ];

  readonly developerWebhooks: CardItem[] = [
    { title: 'portfolio.created', subtitle: 'Active', meta: 'Live', tone: 'tone-emerald' },
    { title: 'payment.received', subtitle: 'Active', meta: 'Live', tone: 'tone-blue' },
    { title: 'bid.accepted', subtitle: 'Active', meta: 'Live', tone: 'tone-amber' },
    { title: 'project.updated', subtitle: 'Active', meta: 'Live', tone: 'tone-purple' },
  ];

  readonly profileMetrics: Metric[] = [
    { label: 'Reputation', value: '94/100', caption: '+3%', tone: 'tone-amber' },
    { label: 'Network', value: '1,240', caption: 'Connections', tone: 'tone-purple' },
    { label: 'Projects Done', value: '48', caption: '+8%', tone: 'tone-blue' },
    { label: 'Earnings YTD', value: '$186K', caption: '+22%', tone: 'tone-emerald' },
  ];

  readonly profilePersonas: CardItem[] = [
    { title: 'Developer', subtitle: 'Full-stack - Node - React', tone: 'tone-blue' },
    { title: 'Designer', subtitle: 'Brand - UX - Systems', tone: 'tone-purple' },
    { title: 'Strategist', subtitle: 'Product - Growth', tone: 'tone-emerald' },
  ];

  readonly profileSettings: CardItem[] = [
    { title: 'Preferences', tone: 'tone-amber' },
    { title: 'Notifications', tone: 'tone-purple' },
    { title: 'Privacy', tone: 'tone-rose' },
    { title: 'Security', tone: 'tone-emerald' },
    { title: 'Billing', tone: 'tone-amber' },
    { title: 'Integrations', tone: 'tone-teal' },
    { title: 'AI Config', tone: 'tone-indigo' },
    { title: 'Data Export', tone: 'tone-cyan' },
  ];

  readonly profileStats: Metric[] = [
    { label: 'Messages', value: '284', tone: 'tone-rose' },
    { label: 'Proposals', value: '18', tone: 'tone-blue' },
    { label: 'Contributions', value: '142', tone: 'tone-indigo' },
    { label: 'Reviews', value: '37', tone: 'tone-amber' },
    { label: 'Referrals', value: '12', tone: 'tone-emerald' },
  ];
  readonly orgMetrics: Metric[] = [
    { label: 'Organizations', value: '4', caption: 'Member of', tone: 'tone-rose' },
    { label: 'Collectives', value: '2', caption: 'Co-founded', tone: 'tone-blue' },
    { label: 'Teams', value: '3', caption: 'Active', tone: 'tone-teal' },
    { label: 'Total Members', value: '186', caption: 'Across all orgs', tone: 'tone-emerald' },
  ];

  readonly orgCards: CardItem[] = [
    { title: 'Delivery Coop', subtitle: 'Worker Cooperative', meta: '35 members', status: 'Active', tone: 'tone-emerald' },
    { title: 'Design Collective', subtitle: 'Creative Collective', meta: '12 members', status: 'Active', tone: 'tone-purple' },
    { title: 'Invest Club', subtitle: 'Investment Club', meta: '22 members', status: 'Active', tone: 'tone-blue' },
    { title: 'Dev DAO', subtitle: 'Autonomous Org', meta: '88 members', status: 'Active', tone: 'tone-indigo' },
    { title: 'Consulting Network', subtitle: 'Professional Network', meta: '18 members', status: 'Pending', tone: 'tone-amber' },
    { title: 'Art Cooperative', subtitle: 'Creative Cooperative', meta: '7 members', status: 'Forming', tone: 'tone-rose' },
  ];

  readonly orgProposals: TableRow[] = [
    { cells: ['New vehicle fleet', 'Delivery Coop', '32/35', 'Passed'], badge: 'Passed', badgeTone: 'tone-emerald' },
    { cells: ['Q2 Budget', 'Invest Club', '18/22', 'Active'], badge: 'Active', badgeTone: 'tone-blue' },
    { cells: ['Member Charter', 'Dev DAO', '72/88', 'Active'], badge: 'Active', badgeTone: 'tone-purple' },
    { cells: ['Rate increase', 'Design Coll.', '9/12', 'Draft'], badge: 'Draft', badgeTone: 'tone-amber' },
  ];

  readonly orgRoles: CardItem[] = [
    { title: 'Delivery Coop', subtitle: 'Co-Founder', meta: 'Lead', tone: 'tone-emerald' },
    { title: 'Invest Club', subtitle: 'Organizer', meta: 'Active', tone: 'tone-blue' },
    { title: 'Dev DAO', subtitle: 'Member', meta: 'Active', tone: 'tone-indigo' },
    { title: 'Design Coll.', subtitle: 'Member', meta: 'Active', tone: 'tone-purple' },
  ];

  readonly orgCapTables: CardItem[] = [
    { title: 'JC', subtitle: '18.5%', meta: 'Equity', tone: 'tone-blue' },
    { title: 'MS', subtitle: '14.2%', meta: 'Equity', tone: 'tone-emerald' },
    { title: 'DP', subtitle: '8.8%', meta: 'Equity', tone: 'tone-amber' },
    { title: 'PN', subtitle: '6.4%', meta: 'Equity', tone: 'tone-rose' },
  ];

  readonly legalMetrics: Metric[] = [
    { label: 'Active Contracts', value: '14', caption: '3 expiring soon', tone: 'tone-rose' },
    { label: 'IP Assets', value: '8', caption: 'Patents - TM - CR', tone: 'tone-purple' },
    { label: 'Compliance Items', value: '6', caption: '2 action needed', tone: 'tone-amber' },
    { label: 'Audit Status', value: 'Clean', caption: 'Last: Mar 2026', tone: 'tone-emerald' },
  ];

  readonly legalIp: CardItem[] = [
    { title: 'Trademark', subtitle: 'Kogi', status: 'Registered', tone: 'tone-rose' },
    { title: 'Patent', subtitle: 'Kogi OS', status: 'Pending', tone: 'tone-amber' },
    { title: 'Copyright', subtitle: 'Logo System', status: 'Active', tone: 'tone-emerald' },
    { title: 'Copyright', subtitle: 'API Spec', status: 'Active', tone: 'tone-rose' },
  ];

  readonly legalContracts: TableRow[] = [
    { cells: ['MSA - TechCorp', 'TechCorp Inc.', '$120K', 'Dec 2026', 'Active'], badge: 'Active', badgeTone: 'tone-emerald' },
    { cells: ['NDA - StartupX', 'StartupX LLC', '-', 'Jun 2026', 'Active'], badge: 'Active', badgeTone: 'tone-emerald' },
    { cells: ['Coop Charter', 'Members (35)', '-', '-', 'Active'], badge: 'Active', badgeTone: 'tone-emerald' },
    { cells: ['Dev Contract', 'DevDAO', '$48K', 'Mar 2026', 'Expiring'], badge: 'Expiring', badgeTone: 'tone-amber' },
    { cells: ['License Agmt.', '3rd Party', '$8K/yr', 'Dec 2026', 'Active'], badge: 'Active', badgeTone: 'tone-emerald' },
  ];

  readonly legalCompliance: CardItem[] = [
    { title: 'GDPR Compliance', subtitle: 'Complete', status: 'Complete', tone: 'tone-emerald' },
    { title: 'SOC 2 Type II', subtitle: 'In Progress', status: 'In Progress', tone: 'tone-amber' },
    { title: 'Tax Compliance', subtitle: 'Complete', status: 'Complete', tone: 'tone-emerald' },
    { title: 'AML / KYC', subtitle: 'Review Needed', status: 'Review', tone: 'tone-rose' },
    { title: 'Data Residency', subtitle: 'Complete', status: 'Complete', tone: 'tone-teal' },
    { title: 'IP Audit', subtitle: 'Scheduled', status: 'Scheduled', tone: 'tone-blue' },
  ];

  readonly legalUpcoming: CardItem[] = [
    { title: 'DevDAO contract expires', subtitle: 'In 7 days', meta: 'Due', tone: 'tone-rose' },
    { title: 'IP audit scheduled', subtitle: 'In 14 days', meta: 'Scheduled', tone: 'tone-blue' },
    { title: 'Q1 tax filing', subtitle: 'In 22 days', meta: 'Due', tone: 'tone-amber' },
    { title: 'NDA renewal', subtitle: 'In 45 days', meta: 'Upcoming', tone: 'tone-purple' },
  ];

  readonly legalActions: CardItem[] = [
    { title: 'Draft NDA', tone: 'tone-blue' },
    { title: 'Submit Patent', tone: 'tone-amber' },
    { title: 'Renew Contract', tone: 'tone-rose' },
    { title: 'File Compliance', tone: 'tone-emerald' },
  ];

  setScreen(id: ScreenId): void {
    this.activeScreen.set(id);
  }
}

bootstrapApplication(AppComponent).catch((err) => console.error(err));
