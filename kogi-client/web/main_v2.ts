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
  icon: string;
}

interface ScreenMeta {
  title: string;
  icon: string;
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
  tag?: string;
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
  likes: string;
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
  icon?: string;
}

interface GanttRow {
  label: string;
  icon?: string;
  bars: TimelineBar[];
}

interface ApiRow {
  method: string;
  path: string;
  description: string;
  tone: Tone;
}

const NAV_ITEMS: NavItem[] = [
  { id: 'dashboard',     label: 'Dashboard',     icon: '▦' },
  { id: 'office',        label: 'Office',        icon: '⊞' },
  { id: 'portfolio',     label: 'Portfolio',     icon: '⊡' },
  { id: 'workspace',     label: 'Workspace',     icon: '⚙' },
  { id: 'timeline',      label: 'Timeline',      icon: '⊟' },
  { id: 'strategy',      label: 'Strategy',      icon: '◎' },
  { id: 'bank',          label: 'Bank',          icon: '⊠' },
  { id: 'exchange',      label: 'Exchange',      icon: '⇄' },
  { id: 'marketplace',   label: 'Marketplace',   icon: '⊕' },
  { id: 'studio',        label: 'Studio',        icon: '◈' },
  { id: 'community',     label: 'Community',     icon: '⊹' },
  { id: 'developer',     label: 'Developer',     icon: '‹›' },
  { id: 'profile',       label: 'Profile',       icon: '◯' },
  { id: 'organizations', label: 'Organizations', icon: '⊛' },
  { id: 'legal',         label: 'Legal',         icon: '⊖' },
];

const SCREEN_META: Record<ScreenId, ScreenMeta> = {
  dashboard:     { title: 'Dashboard',     icon: '▦',  tabs: ['Overview', 'Activity', 'AI'] },
  office:        { title: 'Office',        icon: '🏢', tabs: ['Projects', 'Programs', 'Portfolio'] },
  portfolio:     { title: 'Portfolio',     icon: '💼', tabs: ['Assets', 'Solutions', 'Artifacts'] },
  workspace:     { title: 'Workspace',     icon: '⚙️', tabs: ['Tasks', 'Kanban', 'Sprints'] },
  timeline:      { title: 'Timeline',      icon: '📅', tabs: ['Calendar', 'Roadmap', 'Gantt'] },
  strategy:      { title: 'Strategy',      icon: '🎯', tabs: ['Strategy', 'Tactics', 'Governance'] },
  bank:          { title: 'Bank',          icon: '🏦', tabs: ['Wallets', 'Finance', 'Fundraising'] },
  exchange:      { title: 'Exchange',      icon: '↔️', tabs: ['Bids', 'Deals', 'Due Diligence'] },
  marketplace:   { title: 'Marketplace',   icon: '🛍️', tabs: ['Buy', 'Sell', 'Barter'] },
  studio:        { title: 'Studio',        icon: '🎨', tabs: ['Ideas', 'Prototypes', 'Tools'] },
  community:     { title: 'Community',     icon: '🌐', tabs: ['Feeds', 'Spaces', 'Messages'] },
  developer:     { title: 'Developer',     icon: '⚡', tabs: ['API', 'SDK', 'Integrations'] },
  profile:       { title: 'Profile',       icon: '👤', tabs: ['Personas', 'Settings', 'Config'] },
  organizations: { title: 'Organizations', icon: '🏛️', tabs: ['Coops', 'Collectives', 'Teams'] },
  legal:         { title: 'Legal',         icon: '⚖️', tabs: ['IP', 'Contracts', 'Compliance'] },
};

@Component({
  selector: 'kogi-root',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="app-shell">
      <!-- ── SIDEBAR ── -->
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
            <span class="nav-icon">{{ item.icon }}</span>
            <span>{{ item.label }}</span>
          </button>
        </nav>
        <div class="sidebar-footer">
          <div class="card sidebar-card">
            <div class="card-title">System</div>
            <div class="sidebar-metric">All modules healthy</div>
            <div class="chip-row">
              <span class="chip tone-emerald">Online</span>
              <span class="chip tone-blue">Sync OK</span>
            </div>
          </div>
        </div>
      </aside>

      <!-- ── MAIN ── -->
      <main class="main">
        <!-- ── TOPBAR ── -->
        <header class="topbar">
          <div class="topbar-left">
            <span class="topbar-icon">{{ meta().icon }}</span>
            <div class="topbar-title-group">
              <div class="topbar-title">{{ meta().title }}</div>
              <div class="topbar-tabs">
                <ng-container *ngFor="let tab of meta().tabs; let i = index; let last = last">
                  <button class="topbar-tab" [class.active]="i === 0" type="button">{{ tab }}</button>
                  <span *ngIf="!last" class="topbar-sep">·</span>
                </ng-container>
              </div>
            </div>
          </div>
          <div class="topbar-actions">
            <div class="search"><input type="text" placeholder="Search..." /></div>
            <button class="pill" type="button">PRO</button>
            <button class="avatar" type="button">J</button>
          </div>
        </header>

        <!-- ── SCREENS ── -->
        <section class="content">
          <ng-container [ngSwitch]="activeScreen()">

            <!-- ════ DASHBOARD ════ -->
            <section *ngSwitchCase="'dashboard'" class="screen">
              <div class="stats-row">
                <div class="card metric-card" *ngFor="let s of dashboardMetrics" [ngClass]="s.tone">
                  <div class="metric-label">{{ s.label }}</div>
                  <div class="metric-value">{{ s.value }}</div>
                  <div class="metric-meta">
                    <span *ngIf="s.change" class="trend" [class.up]="s.trend==='up'" [class.down]="s.trend==='down'">{{ s.change }}</span>
                    <span class="metric-caption">{{ s.caption }}</span>
                  </div>
                  <div class="progress" *ngIf="s.progress !== undefined"><span [style.width.%]="s.progress"></span></div>
                </div>
              </div>

              <div class="layout-2">
                <div class="card">
                  <div class="card-title">Quick Access — Modules</div>
                  <div class="quick-grid">
                    <button *ngFor="let q of dashboardQuickLinks" class="quick-card" type="button" [ngClass]="q.tone">
                      <span class="quick-icon">{{ q.icon }}</span>
                      <span class="quick-label">{{ q.label }}</span>
                    </button>
                  </div>
                </div>

                <div class="card">
                  <div class="card-title">Recent Activity</div>
                  <div class="table-row table-header" style="--cols: 2.2fr 1fr 1fr 1fr;">
                    <span>Event</span><span>Module</span><span>Time</span><span>Status</span>
                  </div>
                  <div class="table-row" *ngFor="let a of dashboardActivity" style="--cols: 2.2fr 1fr 1fr 1fr;">
                    <span>{{ a.event }}</span>
                    <span class="status" [ngClass]="a.tone">{{ a.module }}</span>
                    <span class="muted">{{ a.time }}</span>
                    <span class="status tone-emerald">{{ a.status }}</span>
                  </div>
                </div>
              </div>
            </section>

            <!-- ════ OFFICE ════ -->
            <section *ngSwitchCase="'office'" class="screen">
              <div class="stats-row">
                <div class="card metric-card" *ngFor="let s of officeMetrics" [ngClass]="s.tone">
                  <div class="metric-label">{{ s.label }}</div>
                  <div class="metric-value">{{ s.value }}</div>
                  <div class="metric-meta"><span class="metric-caption">{{ s.caption }}</span></div>
                  <div class="progress" *ngIf="s.progress !== undefined"><span [style.width.%]="s.progress"></span></div>
                </div>
              </div>

              <div class="layout-2">
                <div class="stack">
                  <div class="card">
                    <div class="card-title">Programs & Projects</div>
                    <div class="program-grid">
                      <div class="card sub-card" *ngFor="let p of officePrograms" [ngClass]="p.tone">
                        <div class="card-title-sm">{{ p.title }}</div>
                        <div class="card-subtitle">{{ p.subtitle }}</div>
                        <div class="progress"><span [style.width.%]="p.progress"></span></div>
                        <div class="avatar-row">
                          <span class="avatar-chip" *ngFor="let m of p.members">{{ m }}</span>
                        </div>
                      </div>
                    </div>
                  </div>
                  <div class="card">
                    <div class="card-title">Portfolio Assets</div>
                    <div class="asset-grid">
                      <button *ngFor="let a of officeAssets" type="button" class="asset-card" [ngClass]="a.tone">
                        <span class="asset-icon">📁</span>
                        <span>{{ a.title }}</span>
                      </button>
                    </div>
                  </div>
                </div>

                <div class="stack">
                  <div class="card">
                    <div class="card-title">3rd Party</div>
                    <div class="chip-row">
                      <span class="chip" *ngFor="let c of officeThirdParty" [ngClass]="c.tone">→ {{ c.label }}</span>
                    </div>
                  </div>
                  <div class="card">
                    <div class="card-title">Team</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let t of officeTeam">
                        <div class="avatar-chip">{{ t.title }}</div>
                        <div class="list-body">
                          <div class="list-title">{{ t.subtitle }}</div>
                          <div class="list-meta">{{ t.meta }}</div>
                        </div>
                        <span class="status tone-emerald">Active</span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </section>

            <!-- ════ PORTFOLIO ════ -->
            <section *ngSwitchCase="'portfolio'" class="screen">
              <div class="stats-row">
                <div class="card metric-card" *ngFor="let s of portfolioMetrics" [ngClass]="s.tone">
                  <div class="metric-label">{{ s.label }}</div>
                  <div class="metric-value">{{ s.value }}</div>
                  <div class="metric-meta"><span class="metric-caption">{{ s.caption }}</span></div>
                </div>
              </div>

              <div class="card">
                <div class="card-title">Portfolio Grid — Modular Tile View</div>
                <div class="portfolio-grid">
                  <div class="portfolio-item" *ngFor="let p of portfolioItems" [ngClass]="p.tone">
                    <div class="portfolio-tag">{{ p.subtitle }}</div>
                    <div class="portfolio-name">{{ p.title }}</div>
                    <span class="status" [ngClass]="p.tone">{{ p.status }}</span>
                  </div>
                </div>
              </div>

              <div class="card">
                <div class="card-title">Linked Platforms</div>
                <div class="chip-row">
                  <span class="chip" *ngFor="let c of portfolioPlatforms" [ngClass]="c.tone">→ {{ c.label }}</span>
                </div>
              </div>
            </section>

            <!-- ════ WORKSPACE ════ -->
            <section *ngSwitchCase="'workspace'" class="screen">
              <div class="stats-row">
                <div class="card metric-card" *ngFor="let s of workspaceMetrics" [ngClass]="s.tone">
                  <div class="metric-label">{{ s.label }}</div>
                  <div class="metric-value">{{ s.value }}</div>
                  <div class="metric-meta"><span class="metric-caption">{{ s.caption }}</span></div>
                  <div class="progress" *ngIf="s.progress !== undefined"><span [style.width.%]="s.progress"></span></div>
                </div>
              </div>

              <div class="layout-2">
                <div class="stack">
                  <div class="card">
                    <div class="card-title">Kanban Board</div>
                    <div class="kanban">
                      <div class="kanban-col" *ngFor="let col of workspaceKanban" [ngClass]="col.tone">
                        <div class="kanban-title">{{ col.title }}</div>
                        <div class="kanban-item" *ngFor="let task of col.items">{{ task }}</div>
                      </div>
                    </div>
                  </div>
                  <div class="card">
                    <div class="card-title">Gantt Timeline</div>
                    <div class="gantt">
                      <div class="gantt-row" *ngFor="let row of workspaceGantt">
                        <div class="gantt-label">{{ row.label }}</div>
                        <div class="gantt-track">
                          <span *ngFor="let bar of row.bars" class="gantt-bar" [ngClass]="bar.tone" [style.left.%]="bar.start" [style.width.%]="bar.width"></span>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>

                <div class="stack">
                  <div class="card">
                    <div class="card-title">Calendar</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let e of workspaceCalendar">
                        <div class="avatar-chip" style="border-radius:4px;width:8px;padding:0;" [ngClass]="e.tone" style="background:var(--tone);width:3px;min-width:3px;border-radius:2px;"></div>
                        <div class="list-body">
                          <div class="list-title">{{ e.title }}</div>
                          <div class="list-meta">{{ e.subtitle }}</div>
                        </div>
                      </div>
                    </div>
                  </div>
                  <div class="card">
                    <div class="card-title">Roadmaps</div>
                    <div class="progress-list">
                      <div class="progress-item" *ngFor="let r of workspaceRoadmaps">
                        <div class="progress-meta"><span>{{ r.title }}</span><span class="muted">{{ r.meta }}</span></div>
                        <div class="progress" [ngClass]="r.tone"><span [style.width.%]="r.progress"></span></div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </section>

            <!-- ════ TIMELINE ════ -->
            <section *ngSwitchCase="'timeline'" class="screen">
              <div class="stats-row">
                <div class="card metric-card" *ngFor="let s of timelineMetrics" [ngClass]="s.tone">
                  <div class="metric-label">{{ s.label }}</div>
                  <div class="metric-value">{{ s.value }}</div>
                  <div class="metric-meta"><span class="metric-caption">{{ s.caption }}</span></div>
                  <div class="progress" *ngIf="s.progress !== undefined"><span [style.width.%]="s.progress"></span></div>
                </div>
              </div>

              <div class="card">
                <div class="card-title">Master Timeline — Q1 2026</div>
                <div class="gantt">
                  <div class="gantt-row" *ngFor="let row of timelineMaster">
                    <div class="gantt-label">
                      <span style="margin-right:5px;font-size:10px;color:var(--text-2)">{{ row.icon }}</span>{{ row.label }}
                    </div>
                    <div class="gantt-track">
                      <span *ngFor="let bar of row.bars" class="gantt-bar" [ngClass]="bar.tone" [style.left.%]="bar.start" [style.width.%]="bar.width"></span>
                    </div>
                  </div>
                </div>
              </div>

              <div class="layout-2">
                <div class="card">
                  <div class="card-title">Scheduled Events</div>
                  <div class="event-grid">
                    <div class="card sub-card" *ngFor="let e of timelineEvents" [ngClass]="e.tone">
                      <div class="card-subtitle">{{ e.meta }}</div>
                      <div class="card-title-sm">{{ e.title }}</div>
                    </div>
                  </div>
                </div>

                <div class="stack">
                  <div class="card">
                    <div class="card-title">Roadmap Progress</div>
                    <div class="progress-list">
                      <div class="progress-item" *ngFor="let r of timelineRoadmap">
                        <div class="progress-meta"><span>{{ r.title }}</span><span class="muted">{{ r.meta }}</span></div>
                        <div class="progress" [ngClass]="r.tone"><span [style.width.%]="r.progress"></span></div>
                      </div>
                    </div>
                  </div>
                  <div class="card">
                    <div class="card-title">Next Deadlines</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let d of timelineDeadlines">
                        <span class="status" [ngClass]="d.tone" style="width:6px;height:6px;border-radius:50%;background:var(--tone);display:inline-block;flex-shrink:0;padding:0;margin-right:2px;"></span>
                        <div class="list-body">
                          <div class="list-title">{{ d.title }}</div>
                          <div class="list-meta">{{ d.subtitle }}</div>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </section>

            <!-- ════ STRATEGY ════ -->
            <section *ngSwitchCase="'strategy'" class="screen">
              <div class="stats-row">
                <div class="card metric-card" *ngFor="let s of strategyMetrics" [ngClass]="s.tone">
                  <div class="metric-label">{{ s.label }}</div>
                  <div class="metric-value">{{ s.value }}</div>
                  <div class="metric-meta"><span class="metric-caption">{{ s.caption }}</span></div>
                  <div class="progress" *ngIf="s.progress !== undefined"><span [style.width.%]="s.progress"></span></div>
                </div>
              </div>

              <div class="card">
                <div class="card-title">Strategy Tree</div>
                <div class="strategy-grid">
                  <div class="card sub-card" *ngFor="let t of strategyTree" [ngClass]="t.tone">
                    <div class="card-subtitle">{{ t.subtitle }}</div>
                    <div class="card-title-sm">{{ t.title }}</div>
                    <div class="card-meta" *ngIf="t.meta">{{ t.meta }}</div>
                  </div>
                </div>
              </div>

              <div class="layout-2">
                <div class="card">
                  <div class="card-title">Tactics & Operations</div>
                  <div class="tactic-grid">
                    <div class="card sub-card" *ngFor="let t of strategyTactics" [ngClass]="t.tone">
                      <div class="card-title-sm">{{ t.title }}</div>
                      <div class="card-subtitle">{{ t.subtitle }}</div>
                    </div>
                  </div>
                </div>

                <div class="stack">
                  <div class="card">
                    <div class="card-title">Governance</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let g of strategyGovernance">
                        <div class="list-body">
                          <div class="list-title">{{ g.title }}</div>
                          <div class="list-meta">{{ g.subtitle }}</div>
                        </div>
                        <span class="status" [ngClass]="g.tone">{{ g.meta }}</span>
                      </div>
                    </div>
                  </div>
                  <div class="card">
                    <div class="card-title">OKR Progress</div>
                    <div class="progress-list">
                      <div class="progress-item" *ngFor="let o of strategyOkrs">
                        <div class="progress-meta"><span>{{ o.title }}</span><span class="muted">{{ o.meta }}</span></div>
                        <div class="progress" [ngClass]="o.tone"><span [style.width.%]="o.progress"></span></div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </section>

            <!-- ════ BANK ════ -->
            <section *ngSwitchCase="'bank'" class="screen">
              <div class="stats-row">
                <div class="card metric-card" *ngFor="let s of bankMetrics" [ngClass]="s.tone">
                  <div class="metric-label">{{ s.label }}</div>
                  <div class="metric-value">{{ s.value }}</div>
                  <div class="metric-meta">
                    <span *ngIf="s.change" class="trend" [class.up]="s.trend==='up'" [class.down]="s.trend==='down'">{{ s.change }}</span>
                    <span class="metric-caption">{{ s.caption }}</span>
                  </div>
                  <div class="progress" *ngIf="s.progress !== undefined"><span [style.width.%]="s.progress"></span></div>
                </div>
              </div>

              <div class="layout-2">
                <div class="stack">
                  <div class="card">
                    <div class="card-title">Wallet Types</div>
                    <div class="wallet-grid">
                      <div class="card sub-card" *ngFor="let w of bankWallets" [ngClass]="w.tone">
                        <div class="card-subtitle" style="font-size:18px;margin-bottom:6px;">{{ w.tag }}</div>
                        <div class="card-title-sm">{{ w.title }}</div>
                        <div class="metric-value" style="font-size:18px;margin:4px 0;">{{ w.value }}</div>
                        <div class="card-meta">{{ w.subtitle }}</div>
                      </div>
                    </div>
                  </div>
                  <div class="card">
                    <div class="card-title">Fundraising & Capital</div>
                    <div class="fund-grid">
                      <div class="card sub-card" *ngFor="let f of bankFundraising" [ngClass]="f.tone">
                        <div class="card-title-sm">{{ f.title }}</div>
                        <div class="metric-value" style="font-size:16px;margin:4px 0;">{{ f.value }}</div>
                        <div class="progress"><span [style.width.%]="f.progress"></span></div>
                        <div class="card-meta" style="margin-top:4px;">{{ f.meta }}</div>
                      </div>
                    </div>
                  </div>
                </div>

                <div class="stack">
                  <div class="card">
                    <div class="card-title">Linked Platforms</div>
                    <div class="chip-row">
                      <span class="chip" *ngFor="let c of bankPlatforms" [ngClass]="c.tone">→ {{ c.label }}</span>
                    </div>
                  </div>
                  <div class="card">
                    <div class="card-title">Transactions</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let t of bankTransactions">
                        <div class="list-body"><div class="list-title">{{ t.title }}</div></div>
                        <span class="status" [ngClass]="t.tone">{{ t.meta }}</span>
                      </div>
                    </div>
                  </div>
                  <div class="card">
                    <div class="card-title">Tax Summary</div>
                    <div class="tax-grid">
                      <div class="card sub-card" *ngFor="let t of bankTaxes" [ngClass]="t.tone">
                        <div class="card-subtitle" style="font-size:9px;text-transform:uppercase;letter-spacing:0.08em;">{{ t.subtitle }}</div>
                        <div class="metric-value" style="font-size:16px;margin-top:4px;">{{ t.value }}</div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </section>

            <!-- ════ EXCHANGE ════ -->
            <section *ngSwitchCase="'exchange'" class="screen">
              <div class="stats-row">
                <div class="card metric-card" *ngFor="let s of exchangeMetrics" [ngClass]="s.tone">
                  <div class="metric-label">{{ s.label }}</div>
                  <div class="metric-value">{{ s.value }}</div>
                  <div class="metric-meta"><span class="metric-caption">{{ s.caption }}</span></div>
                </div>
              </div>

              <div class="layout-2">
                <div class="stack">
                  <div class="card">
                    <div class="card-title">Bids & Offers</div>
                    <div class="table-row table-header" style="--cols: 2fr 1fr 1fr 1fr 1.5fr;">
                      <span>Item</span><span>Type</span><span>Value</span><span>Status</span><span>Counterparty</span>
                    </div>
                    <div class="table-row" *ngFor="let r of exchangeBids" style="--cols: 2fr 1fr 1fr 1fr 1.5fr;">
                      <span>{{ r.cells[0] }}</span>
                      <span class="muted">{{ r.cells[1] }}</span>
                      <span class="muted">{{ r.cells[2] }}</span>
                      <span class="status" [ngClass]="r.badgeTone">{{ r.badge }}</span>
                      <span class="muted">{{ r.cells[4] }}</span>
                    </div>
                  </div>
                  <div class="card">
                    <div class="card-title">Deal Pipeline</div>
                    <div class="deal-grid">
                      <div class="card sub-card" *ngFor="let d of exchangePipeline" [ngClass]="d.tone">
                        <div class="card-title-sm">{{ d.title }}</div>
                        <div class="card-subtitle">{{ d.subtitle }}</div>
                        <div class="metric-value" style="font-size:16px;margin:4px 0;">{{ d.value }}</div>
                        <div class="progress"><span [style.width.%]="d.progress"></span></div>
                      </div>
                    </div>
                  </div>
                </div>

                <div class="stack">
                  <div class="card">
                    <div class="card-title">Requests</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let r of exchangeRequests">
                        <div class="list-body">
                          <div class="list-title">{{ r.title }}</div>
                          <div class="list-meta">{{ r.subtitle }}</div>
                        </div>
                        <span class="status" [ngClass]="r.tone">{{ r.meta }}</span>
                      </div>
                    </div>
                  </div>
                  <div class="card">
                    <div class="card-title">Linked Platforms</div>
                    <div class="chip-row">
                      <span class="chip" *ngFor="let c of exchangePlatforms" [ngClass]="c.tone">→ {{ c.label }}</span>
                    </div>
                  </div>
                </div>
              </div>
            </section>

            <!-- ════ MARKETPLACE ════ -->
            <section *ngSwitchCase="'marketplace'" class="screen">
              <div class="stats-row">
                <div class="card metric-card" *ngFor="let s of marketplaceMetrics" [ngClass]="s.tone">
                  <div class="metric-label">{{ s.label }}</div>
                  <div class="metric-value">{{ s.value }}</div>
                  <div class="metric-meta">
                    <span *ngIf="s.change" class="trend" [class.up]="s.trend==='up'" [class.down]="s.trend==='down'">{{ s.change }}</span>
                    <span class="metric-caption">{{ s.caption }}</span>
                  </div>
                </div>
              </div>

              <div class="layout-2">
                <div class="card">
                  <div class="card-title">Marketplace — Modular Grid</div>
                  <div class="market-grid">
                    <div class="card sub-card" *ngFor="let m of marketplaceItems" [ngClass]="m.tone">
                      <div class="card-subtitle" style="font-size:9px;text-transform:uppercase;letter-spacing:0.08em;margin-bottom:4px;">{{ m.tag }}</div>
                      <div class="card-title-sm">{{ m.title }}</div>
                      <div style="margin-top:6px;font-weight:700;color:var(--tone,var(--text));">{{ m.value }}</div>
                    </div>
                  </div>
                </div>

                <div class="stack">
                  <div class="card">
                    <div class="card-title">Barter System</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let b of marketplaceBarter">
                        <div class="list-body">
                          <div class="list-title">{{ b.title }}</div>
                          <div class="list-meta">{{ b.subtitle }}</div>
                        </div>
                      </div>
                    </div>
                  </div>
                  <div class="card">
                    <div class="card-title">My Orders</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let o of marketplaceOrders">
                        <div class="list-body">
                          <div class="list-title">{{ o.title }}</div>
                          <div class="list-meta">{{ o.subtitle }}</div>
                        </div>
                        <span class="muted">{{ o.value }}</span>
                      </div>
                    </div>
                  </div>
                  <div class="card">
                    <div class="card-title">Linked Platforms</div>
                    <div class="chip-row">
                      <span class="chip" *ngFor="let c of marketplacePlatforms" [ngClass]="c.tone">→ {{ c.label }}</span>
                    </div>
                  </div>
                </div>
              </div>
            </section>

            <!-- ════ STUDIO ════ -->
            <section *ngSwitchCase="'studio'" class="screen">
              <div class="stats-row">
                <div class="card metric-card" *ngFor="let s of studioMetrics" [ngClass]="s.tone">
                  <div class="metric-label">{{ s.label }}</div>
                  <div class="metric-value">{{ s.value }}</div>
                  <div class="metric-meta"><span class="metric-caption">{{ s.caption }}</span></div>
                  <div class="progress" *ngIf="s.progress !== undefined"><span [style.width.%]="s.progress"></span></div>
                </div>
              </div>

              <div class="layout-2">
                <div class="stack">
                  <div class="card">
                    <div class="card-title">Ideas & Concepts — Modular Grid</div>
                    <div class="studio-grid">
                      <div class="card sub-card" *ngFor="let i of studioIdeas" [ngClass]="i.tone">
                        <div class="card-subtitle" style="font-size:9px;text-transform:uppercase;letter-spacing:0.08em;margin-bottom:4px;">{{ i.tag }}</div>
                        <div class="card-title-sm">{{ i.title }}</div>
                      </div>
                    </div>
                  </div>
                  <div class="card">
                    <div class="card-title">Testing & Testbeds</div>
                    <div class="studio-grid">
                      <div class="card sub-card" *ngFor="let t of studioTestbeds" [ngClass]="t.tone">
                        <div class="card-subtitle" style="font-size:9px;text-transform:uppercase;letter-spacing:0.08em;margin-bottom:4px;">Testbed</div>
                        <div class="card-title-sm">{{ t.title }}</div>
                      </div>
                    </div>
                  </div>
                </div>

                <div class="stack">
                  <div class="card">
                    <div class="card-title">Toolsets & Toolkits</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let t of studioToolsets">
                        <div class="list-body">
                          <div class="list-title">{{ t.title }}</div>
                          <div class="list-meta">{{ t.subtitle }}</div>
                        </div>
                      </div>
                    </div>
                  </div>
                  <div class="card">
                    <div class="card-title">Files & Notes</div>
                    <div class="asset-grid" style="grid-template-columns:repeat(2,1fr);">
                      <button *ngFor="let f of studioFiles" type="button" class="asset-card" [ngClass]="f.tone">
                        <span class="asset-icon">📒</span>
                        <span>{{ f.title }}</span>
                      </button>
                    </div>
                  </div>
                  <div class="card">
                    <div class="card-title">Linked Platforms</div>
                    <div class="chip-row">
                      <span class="chip" *ngFor="let c of studioPlatforms" [ngClass]="c.tone">→ {{ c.label }}</span>
                    </div>
                  </div>
                </div>
              </div>
            </section>

            <!-- ════ COMMUNITY ════ -->
            <section *ngSwitchCase="'community'" class="screen">
              <div class="stats-row">
                <div class="card metric-card" *ngFor="let s of communityMetrics" [ngClass]="s.tone">
                  <div class="metric-label">{{ s.label }}</div>
                  <div class="metric-value">{{ s.value }}</div>
                  <div class="metric-meta"><span class="metric-caption">{{ s.caption }}</span></div>
                </div>
              </div>

              <div class="layout-2">
                <div class="card">
                  <div class="card-title">Feeds & Timelines</div>
                  <div class="feed-list">
                    <div class="feed-item" *ngFor="let f of communityFeeds">
                      <div class="avatar-chip">{{ f.author.charAt(0) }}</div>
                      <div class="feed-body">
                        <div class="feed-author">{{ f.author }} <span class="feed-time" style="font-weight:400;">{{ f.time }}</span></div>
                        <div class="feed-message">{{ f.message }}</div>
                        <div class="feed-meta">♥ {{ f.likes }} · <span>Reply</span> · <span>Share</span></div>
                      </div>
                    </div>
                  </div>
                </div>

                <div class="stack">
                  <div class="card">
                    <div class="card-title">Spaces & Rooms</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let s of communitySpaces">
                        <div class="list-body">
                          <div class="list-title">{{ s.title }}</div>
                          <div class="list-meta">{{ s.subtitle }}</div>
                        </div>
                        <span style="font-size:11px;color:var(--text-2);">→</span>
                      </div>
                    </div>
                  </div>
                  <div class="card">
                    <div class="card-title">DMs</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let d of communityDms">
                        <div class="avatar-chip">{{ d.title.charAt(0) }}</div>
                        <div class="list-body">
                          <div class="list-title">{{ d.title }}</div>
                          <div class="list-meta">{{ d.subtitle }}</div>
                        </div>
                      </div>
                    </div>
                  </div>
                  <div class="card">
                    <div class="card-title">Linked Platforms</div>
                    <div class="chip-row">
                      <span class="chip" *ngFor="let c of communityPlatforms" [ngClass]="c.tone">→ {{ c.label }}</span>
                    </div>
                  </div>
                </div>
              </div>
            </section>

            <!-- ════ DEVELOPER ════ -->
            <section *ngSwitchCase="'developer'" class="screen">
              <div class="stats-row">
                <div class="card metric-card" *ngFor="let s of developerMetrics" [ngClass]="s.tone">
                  <div class="metric-label">{{ s.label }}</div>
                  <div class="metric-value">{{ s.value }}</div>
                  <div class="metric-meta"><span class="metric-caption">{{ s.caption }}</span></div>
                  <div class="progress" *ngIf="s.progress !== undefined"><span [style.width.%]="s.progress"></span></div>
                </div>
              </div>

              <div class="layout-2">
                <div class="stack">
                  <div class="card">
                    <div class="card-title">API Reference</div>
                    <div class="api-list">
                      <div class="api-row" *ngFor="let r of developerApi">
                        <span class="api-method" [ngClass]="r.tone">{{ r.method }}</span>
                        <span class="api-path">{{ r.path }}</span>
                        <span class="api-desc">{{ r.description }}</span>
                      </div>
                    </div>
                  </div>
                  <div class="card">
                    <div class="card-title">Extensions & Integrations</div>
                    <div class="extension-grid">
                      <div class="card sub-card" *ngFor="let e of developerExtensions" [ngClass]="e.tone">
                        <div class="card-title-sm">{{ e.title }}</div>
                        <div class="card-subtitle">{{ e.subtitle }}</div>
                      </div>
                    </div>
                  </div>
                </div>

                <div class="stack">
                  <div class="card">
                    <div class="card-title">API Keys</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let k of developerKeys">
                        <div class="list-body">
                          <div class="list-title">{{ k.title }}</div>
                          <div class="list-meta">{{ k.subtitle }}</div>
                        </div>
                        <span class="status" [ngClass]="k.tone">{{ k.meta }}</span>
                      </div>
                    </div>
                  </div>
                  <div class="card">
                    <div class="card-title">SDKs</div>
                    <div class="chip-row">
                      <span class="chip" *ngFor="let s of developerSdks" [ngClass]="s.tone">{{ s.label }}</span>
                    </div>
                  </div>
                  <div class="card">
                    <div class="card-title">Webhooks</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let w of developerWebhooks">
                        <div class="list-body">
                          <div class="list-title">{{ w.title }}</div>
                          <div class="list-meta">{{ w.subtitle }}</div>
                        </div>
                        <span class="status" [ngClass]="w.tone">{{ w.meta }}</span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </section>

            <!-- ════ PROFILE ════ -->
            <section *ngSwitchCase="'profile'" class="screen">
              <div class="profile-header">
                <div class="card profile-card">
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
                  <div class="card metric-card" *ngFor="let s of profileMetrics" [ngClass]="s.tone">
                    <div class="metric-label">{{ s.label }}</div>
                    <div class="metric-value">{{ s.value }}</div>
                    <div class="metric-meta"><span class="metric-caption">{{ s.caption }}</span></div>
                  </div>
                </div>
              </div>

              <div class="card">
                <div class="card-title">Personas & Roles</div>
                <div class="persona-grid">
                  <div class="card sub-card" *ngFor="let p of profilePersonas" [ngClass]="p.tone">
                    <div class="card-subtitle" style="font-size:18px;margin-bottom:4px;">{{ p.tag }}</div>
                    <div class="card-title-sm">{{ p.title }}</div>
                    <div class="card-meta">{{ p.subtitle }}</div>
                  </div>
                </div>
              </div>

              <div class="card">
                <div class="card-title">Settings & Configuration</div>
                <div class="settings-grid">
                  <button *ngFor="let s of profileSettings" type="button" class="asset-card" [ngClass]="s.tone">
                    {{ s.title }}
                  </button>
                </div>
              </div>

              <div class="card">
                <div class="card-title">Activity & Stats</div>
                <div class="stats-row">
                  <div class="card metric-card" *ngFor="let s of profileStats" [ngClass]="s.tone">
                    <div class="metric-label">{{ s.label }}</div>
                    <div class="metric-value">{{ s.value }}</div>
                  </div>
                </div>
              </div>
            </section>

            <!-- ════ ORGANIZATIONS ════ -->
            <section *ngSwitchCase="'organizations'" class="screen">
              <div class="stats-row">
                <div class="card metric-card" *ngFor="let s of orgMetrics" [ngClass]="s.tone">
                  <div class="metric-label">{{ s.label }}</div>
                  <div class="metric-value">{{ s.value }}</div>
                  <div class="metric-meta"><span class="metric-caption">{{ s.caption }}</span></div>
                </div>
              </div>

              <div class="layout-2">
                <div class="stack">
                  <div class="card">
                    <div class="card-title">Organizations — Modular Grid</div>
                    <div class="org-grid">
                      <div class="card sub-card" *ngFor="let o of orgCards" [ngClass]="o.tone">
                        <div class="card-subtitle" style="font-size:18px;margin-bottom:6px;">⊞</div>
                        <div class="card-title-sm">{{ o.title }}</div>
                        <div class="card-meta">{{ o.subtitle }}</div>
                        <div class="card-meta">{{ o.meta }}</div>
                        <span class="status" [ngClass]="o.tone" style="margin-top:4px;display:block;">{{ o.status }}</span>
                      </div>
                    </div>
                  </div>
                  <div class="card">
                    <div class="card-title">Governance & Proposals</div>
                    <div class="table-row table-header" style="--cols: 2fr 1fr 1fr 1fr;">
                      <span>Proposal</span><span>Org</span><span>Votes</span><span>Status</span>
                    </div>
                    <div class="table-row" *ngFor="let r of orgProposals" style="--cols: 2fr 1fr 1fr 1fr;">
                      <span>{{ r.cells[0] }}</span>
                      <span class="muted">{{ r.cells[1] }}</span>
                      <span class="muted">{{ r.cells[2] }}</span>
                      <span class="status" [ngClass]="r.badgeTone">{{ r.badge }}</span>
                    </div>
                  </div>
                </div>

                <div class="stack">
                  <div class="card">
                    <div class="card-title">My Roles</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let r of orgRoles">
                        <div class="list-body">
                          <div class="list-title">{{ r.title }}</div>
                          <div class="list-meta" style="color:var(--tone,var(--text-2));">{{ r.subtitle }}</div>
                        </div>
                      </div>
                    </div>
                  </div>
                  <div class="card">
                    <div class="card-title">Cap Tables</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let c of orgCapTables">
                        <span class="avatar-chip" [ngClass]="c.tone" style="background:var(--tone-bg);color:var(--tone);border-color:var(--tone-border);">{{ c.title }}</span>
                        <div class="list-body"><div class="list-title">{{ c.subtitle }}</div></div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </section>

            <!-- ════ LEGAL ════ -->
            <section *ngSwitchCase="'legal'" class="screen">
              <div class="stats-row">
                <div class="card metric-card" *ngFor="let s of legalMetrics" [ngClass]="s.tone">
                  <div class="metric-label">{{ s.label }}</div>
                  <div class="metric-value">{{ s.value }}</div>
                  <div class="metric-meta"><span class="metric-caption">{{ s.caption }}</span></div>
                </div>
              </div>

              <div class="layout-2">
                <div class="stack">
                  <div class="card">
                    <div class="card-title">IP & Trademarks</div>
                    <div class="legal-grid">
                      <div class="card sub-card" *ngFor="let ip of legalIp" [ngClass]="ip.tone">
                        <div class="card-subtitle" style="font-size:9px;text-transform:uppercase;letter-spacing:0.08em;margin-bottom:4px;">{{ ip.tag }}</div>
                        <div class="card-title-sm">{{ ip.title }}</div>
                        <span class="status" [ngClass]="ip.tone" style="margin-top:6px;display:inline-block;padding:2px 6px;background:var(--tone-bg);border:1px solid var(--tone-border);border-radius:3px;font-size:10px;">{{ ip.status }}</span>
                      </div>
                    </div>
                  </div>
                  <div class="card">
                    <div class="card-title">Contracts & Agreements</div>
                    <div class="table-row table-header" style="--cols: 2fr 1.5fr 0.8fr 1fr 1fr;">
                      <span>Contract</span><span>Party</span><span>Value</span><span>Expires</span><span>Status</span>
                    </div>
                    <div class="table-row" *ngFor="let r of legalContracts" style="--cols: 2fr 1.5fr 0.8fr 1fr 1fr;">
                      <span>{{ r.cells[0] }}</span>
                      <span class="muted">{{ r.cells[1] }}</span>
                      <span style="color:var(--amber);">{{ r.cells[2] }}</span>
                      <span class="muted">{{ r.cells[3] }}</span>
                      <span class="status" [ngClass]="r.badgeTone">{{ r.badge }}</span>
                    </div>
                  </div>
                  <div class="card">
                    <div class="card-title">Compliance & Audit</div>
                    <div class="legal-grid" style="grid-template-columns:repeat(3,1fr);">
                      <div class="card sub-card" *ngFor="let c of legalCompliance" [ngClass]="c.tone">
                        <div class="card-title-sm">{{ c.title }}</div>
                        <span class="status" [ngClass]="c.tone" style="margin-top:4px;display:block;font-size:10.5px;">{{ c.subtitle }}</span>
                      </div>
                    </div>
                  </div>
                </div>

                <div class="stack">
                  <div class="card">
                    <div class="card-title">Upcoming</div>
                    <div class="list">
                      <div class="list-item" *ngFor="let u of legalUpcoming">
                        <div class="list-body">
                          <div class="list-title">{{ u.title }}</div>
                          <div class="list-meta" style="color:var(--tone,var(--text-2));">{{ u.subtitle }}</div>
                        </div>
                      </div>
                    </div>
                  </div>
                  <div class="card">
                    <div class="card-title">Quick Actions</div>
                    <div class="action-list">
                      <button *ngFor="let a of legalActions" type="button" class="action-button" [ngClass]="a.tone">
                        {{ a.title }}
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
export class AppComponent {
  readonly navItems = NAV_ITEMS;
  readonly activeScreen = signal<ScreenId>('dashboard');
  readonly meta = computed(() => SCREEN_META[this.activeScreen()]);

  setScreen(id: ScreenId): void { this.activeScreen.set(id); }

  // ── DASHBOARD ───────────────────────────────────────────────────────────────
  readonly dashboardMetrics: Metric[] = [
    { label: 'Portfolio Health', value: '87%', change: '4%', trend: 'up', caption: '3 at risk', progress: 87, tone: 'tone-blue' },
    { label: 'Active Projects',  value: '12',  change: '2%', trend: 'up', caption: 'Concurrent', tone: 'tone-teal' },
    { label: 'Net Revenue',      value: '$24.8K', change: '11%', trend: 'up', caption: 'This month', tone: 'tone-emerald' },
    { label: 'AI Credits',       value: '8,420',  change: '2%', trend: 'down', caption: 'Remaining', progress: 66, tone: 'tone-purple' },
  ];

  readonly dashboardQuickLinks = [
    { label: 'Office',      icon: '🏢', tone: 'tone-blue'    },
    { label: 'Portfolio',   icon: '💼', tone: 'tone-purple'  },
    { label: 'Bank',        icon: '🏦', tone: 'tone-emerald' },
    { label: 'Exchange',    icon: '↔️', tone: 'tone-amber'   },
    { label: 'Marketplace', icon: '🛍️', tone: 'tone-teal'   },
    { label: 'Studio',      icon: '🎨', tone: 'tone-indigo'  },
    { label: 'Community',   icon: '🌐', tone: 'tone-rose'    },
    { label: 'Developer',   icon: '⚡', tone: 'tone-cyan'    },
  ];

  readonly dashboardActivity: ActivityItem[] = [
    { event: 'Payment received — $3,200', module: 'Exchange',  time: '2 min ago',  status: 'Active', tone: 'tone-emerald' },
    { event: 'New proposal: Brand Redesign', module: 'Studio',  time: '14 min ago', status: 'Active', tone: 'tone-blue'    },
    { event: 'Project milestone hit',     module: 'Office',    time: '1 hr ago',   status: 'Active', tone: 'tone-purple'  },
    { event: 'Community post: 24 reactions', module: 'Community', time: '2 hr ago', status: 'Active', tone: 'tone-rose'   },
    { event: 'Token distribution complete', module: 'Bank',    time: '5 hr ago',   status: 'Active', tone: 'tone-teal'    },
  ];

  // ── OFFICE ──────────────────────────────────────────────────────────────────
  readonly officeMetrics: Metric[] = [
    { label: 'Programs',      value: '4',   caption: '2 delayed',    tone: 'tone-blue' },
    { label: 'Projects',      value: '18',  caption: '12 active',    tone: 'tone-teal', progress: 70 },
    { label: 'Milestones Due',value: '6',   caption: 'Next 7 days',  tone: 'tone-amber' },
    { label: 'Team Capacity', value: '73%', caption: '-5%',          tone: 'tone-emerald', progress: 73 },
  ];

  readonly officePrograms: CardItem[] = [
    { title: 'Alpha Platform',  subtitle: 'In Progress · 8/12', tone: 'tone-blue',   progress: 67, members: ['A','A','A'] },
    { title: 'Beta Campaign',   subtitle: 'Planning · 2/6',     tone: 'tone-teal',   progress: 33, members: ['A','A','A'] },
    { title: 'Gamma Research',  subtitle: 'On Hold · 5/5',      tone: 'tone-emerald',progress: 100,members: ['A','A','A'] },
    { title: 'Delta Ops',       subtitle: 'In Progress · 11/15',tone: 'tone-amber',  progress: 73, members: ['A','A','A'] },
    { title: 'Epsilon Design',  subtitle: 'Active · 3/8',       tone: 'tone-purple', progress: 38, members: ['A','A','A'] },
    { title: 'Zeta Legal',      subtitle: 'Review · 7/9',       tone: 'tone-rose',   progress: 78, members: ['A','A','A'] },
  ];

  readonly officeAssets: CardItem[] = [
    { title: 'Projects',    tone: 'tone-blue'    },
    { title: 'Programs',    tone: 'tone-teal'    },
    { title: 'Assets',      tone: 'tone-purple'  },
    { title: 'Solutions',   tone: 'tone-emerald' },
    { title: 'Artifacts',   tone: 'tone-indigo'  },
    { title: 'Resources',   tone: 'tone-rose'    },
    { title: 'Blueprints',  tone: 'tone-amber'   },
    { title: 'APIs',        tone: 'tone-cyan'    },
    { title: 'Components',  tone: 'tone-blue'    },
    { title: 'Datasets',    tone: 'tone-teal'    },
  ];

  readonly officeThirdParty: ChipItem[] = [
    { label: 'Jira',   tone: 'tone-blue'   },
    { label: 'Monday', tone: 'tone-amber'  },
    { label: 'GitHub', tone: 'tone-indigo' },
    { label: 'GitLab', tone: 'tone-rose'   },
    { label: 'Notion', tone: 'tone-teal'   },
  ];

  readonly officeTeam: CardItem[] = [
    { title: 'J', subtitle: 'Jordan C.',  meta: '3 active projects', tone: 'tone-blue'    },
    { title: 'M', subtitle: 'Maria S.',   meta: '3 active projects', tone: 'tone-emerald' },
    { title: 'D', subtitle: 'Devon P.',   meta: '3 active projects', tone: 'tone-amber'   },
    { title: 'P', subtitle: 'Priya N.',   meta: '3 active projects', tone: 'tone-rose'    },
  ];

  // ── PORTFOLIO ────────────────────────────────────────────────────────────────
  readonly portfolioMetrics: Metric[] = [
    { label: 'Projects',  value: '12', caption: 'Active',   tone: 'tone-blue'    },
    { label: 'Programs',  value: '4',  caption: 'Operating',tone: 'tone-teal'    },
    { label: 'Assets',    value: '28', caption: 'Managed',  tone: 'tone-purple'  },
    { label: 'Solutions', value: '7',  caption: 'Live',     tone: 'tone-emerald' },
    { label: 'Artifacts', value: '15', caption: 'Ready',    tone: 'tone-indigo'  },
  ];

  readonly portfolioItems: CardItem[] = [
    { title: 'Brand Identity System', subtitle: 'Design',   status: 'Active',    tone: 'tone-purple'  },
    { title: 'API Gateway v2',        subtitle: 'Dev',      status: 'Released',  tone: 'tone-teal'    },
    { title: 'Market Research',       subtitle: 'Research', status: 'Draft',     tone: 'tone-amber'   },
    { title: 'CRM Integration',       subtitle: 'Solution', status: 'Active',    tone: 'tone-emerald' },
    { title: 'UI Component Lib',      subtitle: 'Asset',    status: 'Active',    tone: 'tone-rose'    },
    { title: 'Tokenomics Model',      subtitle: 'Finance',  status: 'Review',    tone: 'tone-cyan'    },
    { title: 'Legal Templates',       subtitle: 'Legal',    status: 'Active',    tone: 'tone-rose'    },
    { title: 'Analytics Dashboard',   subtitle: 'Dev',      status: 'Active',    tone: 'tone-indigo'  },
    { title: 'Product Roadmap',       subtitle: 'Strategy', status: 'Active',    tone: 'tone-blue'    },
    { title: 'Mobile App MVP',        subtitle: 'Dev',      status: 'Building',  tone: 'tone-purple'  },
    { title: 'Content Strategy',      subtitle: 'Marketing',status: 'Draft',     tone: 'tone-amber'   },
    { title: 'Partnership Deck',      subtitle: 'Sales',    status: 'Active',    tone: 'tone-amber'   },
  ];

  readonly portfolioPlatforms: ChipItem[] = [
    { label: 'Behance',      tone: 'tone-blue'   },
    { label: 'GitHub',       tone: 'tone-indigo' },
    { label: 'Dribbble',     tone: 'tone-rose'   },
    { label: 'Figma',        tone: 'tone-purple' },
    { label: 'Notion',       tone: 'tone-teal'   },
    { label: 'Google Drive', tone: 'tone-cyan'   },
  ];

  // ── WORKSPACE ───────────────────────────────────────────────────────────────
  readonly workspaceMetrics: Metric[] = [
    { label: 'Open Tasks',    value: '34', caption: '8 overdue',   tone: 'tone-blue'    },
    { label: 'Sprints Active',value: '3',  caption: '2 on track',  tone: 'tone-teal', progress: 60 },
    { label: 'Blocked',       value: '5',  caption: 'Needs action',tone: 'tone-amber'   },
    { label: 'Done This Week',value: '21', caption: '+15%',        tone: 'tone-emerald' },
  ];

  readonly workspaceKanban: KanbanColumn[] = [
    { title: 'Backlog',     tone: 'tone-blue',    items: ['Auth redesign','Payment flow','API docs','Onboard UX'] },
    { title: 'In Progress', tone: 'tone-amber',   items: ['Mobile nav','Token calc','Legal review','Test suite'] },
    { title: 'Review',      tone: 'tone-purple',  items: ['Brand deck','Coop model','RFC-009','Data model'] },
    { title: 'Done',        tone: 'tone-emerald', items: ['Login fix','CSV export','Error states','Docs v2'] },
  ];

  readonly workspaceCalendar: CardItem[] = [
    { title: 'Client sync 10am', subtitle: 'Mon Mar 10',  tone: 'tone-blue'   },
    { title: 'Sprint review 3pm',subtitle: 'Mon Mar 10',  tone: 'tone-teal'   },
    { title: 'Payment due',      subtitle: 'Tue Mar 11',  tone: 'tone-amber'  },
    { title: 'Board meeting',    subtitle: 'Wed Mar 12',  tone: 'tone-rose'   },
    { title: 'Milestone delivery',subtitle: 'Thu Mar 13', tone: 'tone-purple' },
  ];

  readonly workspaceRoadmaps: Array<{ title: string; meta: string; progress: number; tone: Tone }> = [
    { title: 'Q1 2026 — Foundation', meta: '65%', progress: 65, tone: 'tone-blue'  },
    { title: 'Q2 2026 — Growth',     meta: '33%', progress: 33, tone: 'tone-teal'  },
    { title: 'Q3 2026 — Scale',      meta: '12%', progress: 12, tone: 'tone-rose'  },
  ];

  readonly workspaceGantt: GanttRow[] = [
    { label: 'Alpha Platform',  bars: [{ label: 'A', tone: 'tone-blue',    start: 5,  width: 55 }] },
    { label: 'Beta Campaign',   bars: [{ label: 'B', tone: 'tone-amber',   start: 30, width: 40 }] },
    { label: 'Gamma Research',  bars: [{ label: 'G', tone: 'tone-emerald', start: 5,  width: 70 }] },
    { label: 'Delta Ops',       bars: [{ label: 'D', tone: 'tone-purple',  start: 40, width: 35 }] },
    { label: 'Epsilon Design',  bars: [{ label: 'E', tone: 'tone-rose',    start: 55, width: 30 }] },
  ];

  // ── TIMELINE ────────────────────────────────────────────────────────────────
  readonly timelineMetrics: Metric[] = [
    { label: 'Milestones',      value: '24',  caption: 'Q1 2026',        tone: 'tone-blue'    },
    { label: 'Scheduled Events',value: '18',  caption: 'Next 30 days',   tone: 'tone-teal'    },
    { label: 'Overdue Items',   value: '3',   caption: 'Needs attention', tone: 'tone-rose'    },
    { label: 'Completion Rate', value: '78%', caption: 'This quarter',   tone: 'tone-emerald', progress: 78 },
  ];

  readonly timelineMaster: GanttRow[] = [
    { label: 'Platform MVP',    icon: '✓', bars: [{ label: '', tone: 'tone-blue',    start: 2,  width: 48 }] },
    { label: 'Design System',   icon: '✓', bars: [{ label: '', tone: 'tone-purple',  start: 20, width: 52 }] },
    { label: 'API v1 Launch',   icon: '□', bars: [{ label: '', tone: 'tone-emerald', start: 35, width: 38 }] },
    { label: 'Beta Onboarding', icon: '□', bars: [{ label: '', tone: 'tone-amber',   start: 45, width: 45 }] },
    { label: 'Legal Review',    icon: '⚐', bars: [{ label: '', tone: 'tone-rose',    start: 28, width: 28 }] },
    { label: 'Exchange Module', icon: '□', bars: [{ label: '', tone: 'tone-cyan',    start: 58, width: 40 }] },
    { label: 'Community Beta',  icon: '□', bars: [{ label: '', tone: 'tone-rose',    start: 65, width: 33 }] },
    { label: 'AI Agent v1',     icon: '□', bars: [{ label: '', tone: 'tone-purple',  start: 52, width: 46 }] },
  ];

  readonly timelineEvents: CardItem[] = [
    { title: 'Client Sync',      meta: 'Mon Mar 10', tone: 'tone-rose'    },
    { title: 'Sprint Review',    meta: 'Mon Mar 10', tone: 'tone-blue'    },
    { title: 'Board Meeting',    meta: 'Wed Mar 12', tone: 'tone-amber'   },
    { title: 'Milestone Gate',   meta: 'Fri Mar 14', tone: 'tone-rose'    },
    { title: 'Tax Filing Due',   meta: 'Mar 22',     tone: 'tone-teal'    },
    { title: 'Q1 Close',         meta: 'Mar 31',     tone: 'tone-purple'  },
  ];

  readonly timelineRoadmap: Array<{ title: string; meta: string; progress: number; tone: Tone }> = [
    { title: 'Q1 Foundation', meta: '65%', progress: 65, tone: 'tone-blue'   },
    { title: 'Q2 Growth',     meta: '30%', progress: 30, tone: 'tone-teal'   },
    { title: 'Q3 Scale',      meta: '10%', progress: 10, tone: 'tone-amber'  },
    { title: 'Q4 Expand',     meta: '0%',  progress: 0,  tone: 'tone-purple' },
  ];

  readonly timelineDeadlines: CardItem[] = [
    { title: 'Legal Review', subtitle: 'In 7 days',  tone: 'tone-rose'   },
    { title: 'API Launch',   subtitle: 'In 11 days', tone: 'tone-emerald'},
    { title: 'Board Deck',   subtitle: 'In 18 days', tone: 'tone-amber'  },
    { title: 'Q1 Close',     subtitle: 'In 21 days', tone: 'tone-blue'   },
  ];

  // ── STRATEGY ────────────────────────────────────────────────────────────────
  readonly strategyMetrics: Metric[] = [
    { label: 'Strategic OKRs',       value: '5',  caption: '3 on track',       tone: 'tone-purple', progress: 60 },
    { label: 'Tactical Initiatives', value: '18', caption: '11 active',        tone: 'tone-blue'    },
    { label: 'Ops Processes',        value: '24', caption: 'Documented',       tone: 'tone-emerald' },
    { label: 'Governance Items',     value: '7',  caption: '2 pending vote',   tone: 'tone-amber'   },
  ];

  readonly strategyTree: CardItem[] = [
    { title: 'Be the OS for independent work',       subtitle: 'Vision',    tone: 'tone-blue'   },
    { title: 'Unify tools · empower workers',         subtitle: 'Mission',   tone: 'tone-teal'   },
    { title: 'Reach 50K users by Q4',                subtitle: 'Obj 1',     tone: 'tone-emerald'},
    { title: '$1M ARR by Q3',                        subtitle: 'Obj 2',     tone: 'tone-amber'  },
    { title: 'Community 10K members',                subtitle: 'Obj 3',     tone: 'tone-purple' },
    { title: 'NPS > 60',                             subtitle: 'Key Result', tone: 'tone-rose'   },
  ];

  readonly strategyTactics: CardItem[] = [
    { title: 'Pricing Experiments', subtitle: 'Tactical · Active', tone: 'tone-blue'    },
    { title: 'Onboarding Funnel',   subtitle: 'Tactical · Active', tone: 'tone-teal'    },
    { title: 'Partnership Program', subtitle: 'Tactical · Active', tone: 'tone-emerald' },
    { title: 'Content Marketing',   subtitle: 'Tactical · Active', tone: 'tone-purple'  },
    { title: 'API Integrations',    subtitle: 'Tactical · Active', tone: 'tone-blue'    },
    { title: 'Support Playbook',    subtitle: 'Tactical · Active', tone: 'tone-amber'   },
    { title: 'Legal Frameworks',    subtitle: 'Tactical · Active', tone: 'tone-rose'    },
    { title: 'Data Infrastructure', subtitle: 'Tactical · Active', tone: 'tone-cyan'    },
  ];

  readonly strategyGovernance: CardItem[] = [
    { title: 'RFC-009',        subtitle: 'Under Review',   meta: 'Review',  tone: 'tone-amber'  },
    { title: 'Equity Policy',  subtitle: 'Approved',       meta: 'Approved',tone: 'tone-emerald'},
    { title: 'Data Retention', subtitle: 'Pending Vote',   meta: 'Vote',    tone: 'tone-rose'   },
    { title: 'Member Charter', subtitle: 'Draft',          meta: 'Draft',   tone: 'tone-teal'   },
  ];

  readonly strategyOkrs: Array<{ title: string; meta: string; progress: number; tone: Tone }> = [
    { title: 'User Growth', meta: '78%', progress: 78, tone: 'tone-blue'    },
    { title: 'Revenue',     meta: '45%', progress: 45, tone: 'tone-emerald' },
    { title: 'Community',   meta: '62%', progress: 62, tone: 'tone-rose'    },
  ];

  // ── BANK ────────────────────────────────────────────────────────────────────
  readonly bankMetrics: Metric[] = [
    { label: 'Total Balance', value: '$142,800', change: '8%',  trend: 'up',   caption: 'All wallets',  tone: 'tone-emerald' },
    { label: 'Operations',    value: '$28,400',  caption: 'Ops Wallet',         tone: 'tone-blue',    progress: 55 },
    { label: 'Investments',   value: '$89,200',  change: '12%', trend: 'up',   caption: 'Portfolio',   tone: 'tone-purple'  },
    { label: 'Trading',       value: '$18,300',  change: '3%',  trend: 'down', caption: 'Exchange',    tone: 'tone-amber'   },
    { label: 'Personal',      value: '$6,900',   caption: 'Spending',           tone: 'tone-teal',    progress: 30 },
  ];

  readonly bankWallets: CardItem[] = [
    { tag: '💳', title: 'Personal Spending', value: '$6,900',   subtitle: 'Daily · bills',           tone: 'tone-teal'    },
    { tag: '💼', title: 'Operations',        value: '$28,400',  subtitle: 'Payroll · tools',         tone: 'tone-blue'    },
    { tag: '📈', title: 'Investment',        value: '$89,200',  subtitle: 'Stocks · bonds',          tone: 'tone-purple'  },
    { tag: '🔄', title: 'Trading',           value: '$18,300',  subtitle: 'Crypto · tokens',         tone: 'tone-amber'   },
    { tag: '🛍️', title: 'Marketplace',      value: '$4,200',   subtitle: 'Purchases',               tone: 'tone-rose'    },
    { tag: '🏛️', title: 'Coop Pool',        value: '$124,500', subtitle: 'Collective capital',      tone: 'tone-indigo'  },
  ];

  readonly bankFundraising: CardItem[] = [
    { title: 'Seed Round',      value: '$450K raised', progress: 72, meta: '72% of goal', tone: 'tone-blue'    },
    { title: 'Community Bond',  value: '$120K raised', progress: 48, meta: '48% of goal', tone: 'tone-emerald' },
    { title: 'Equipment Lease', value: '$28K raised',  progress: 90, meta: '90% of goal', tone: 'tone-amber'   },
  ];

  readonly bankPlatforms: ChipItem[] = [
    { label: 'Stripe',     tone: 'tone-blue'   },
    { label: 'Wells Fargo',tone: 'tone-rose'   },
    { label: 'Chase',      tone: 'tone-indigo' },
    { label: 'GoFundMe',   tone: 'tone-emerald'},
    { label: 'Patreon',    tone: 'tone-amber'  },
  ];

  readonly bankTransactions: CardItem[] = [
    { title: 'Stripe payout',      meta: '+$3,200', tone: 'tone-emerald' },
    { title: 'Tool subscription',  meta: '-$49',    tone: 'tone-rose'    },
    { title: 'Coop distribution',  meta: '+$840',   tone: 'tone-emerald' },
    { title: 'Tax payment',        meta: '-$2,100', tone: 'tone-rose'    },
    { title: 'Invoice paid',       meta: '+$8,400', tone: 'tone-emerald' },
  ];

  readonly bankTaxes: CardItem[] = [
    { title: 'Taxes', subtitle: 'Income Tax',  value: '$18,400', tone: 'tone-rose'    },
    { title: 'Self Employed', subtitle: 'Self Employed', value: '$4,200',  tone: 'tone-amber'   },
    { title: 'Deductions',    subtitle: 'Deductions',    value: '-$6,800', tone: 'tone-emerald' },
    { title: 'Estimated',     subtitle: 'Estimated',     value: '$15,800', tone: 'tone-blue'    },
  ];

  // ── EXCHANGE ────────────────────────────────────────────────────────────────
  readonly exchangeMetrics: Metric[] = [
    { label: 'Active Bids',  value: '12', caption: '3 expiring', tone: 'tone-blue'    },
    { label: 'Offers Out',   value: '8',  caption: '$24K value', tone: 'tone-purple'  },
    { label: 'Closed Deals', value: '5',  caption: 'This month', tone: 'tone-emerald' },
    { label: 'Pipeline',     value: '$142K', caption: 'Total value', tone: 'tone-amber' },
  ];

  readonly exchangeBids: TableRow[] = [
    { cells: ['Design System License', 'Offer',   '$12K',  'Active',  'Acme Corp'],      badge: 'Active',  badgeTone: 'tone-emerald' },
    { cells: ['API Access Token',       'Bid',    '$3,500','Pending', 'StartupXYZ'],      badge: 'Pending', badgeTone: 'tone-amber'   },
    { cells: ['Brand Strategy Pack',    'Offer',  '$8K',   'Active',  'DevDAO'],          badge: 'Active',  badgeTone: 'tone-emerald' },
    { cells: ['UX Audit Service',       'Bid',    '$5K',   'Review',  'Invest Club'],     badge: 'Review',  badgeTone: 'tone-purple'  },
    { cells: ['Full-Stack Dev Sprint',  'Offer',  '$15K',  'Expiring','TechCorp Inc.'],   badge: 'Expiring',badgeTone: 'tone-rose'    },
  ];

  readonly exchangePipeline: CardItem[] = [
    { title: 'Acme Contract',  subtitle: 'Design + Dev', value: '$28K', progress: 75, tone: 'tone-blue'    },
    { title: 'DevDAO Sprint',  subtitle: 'Engineering',  value: '$48K', progress: 50, tone: 'tone-purple'  },
    { title: 'Brand Package',  subtitle: 'Branding',     value: '$18K', progress: 90, tone: 'tone-emerald' },
    { title: 'Startup Deal',   subtitle: 'Strategy',     value: '$12K', progress: 30, tone: 'tone-amber'   },
  ];

  readonly exchangeRequests: CardItem[] = [
    { title: 'Due Diligence Review', subtitle: 'Acme Corp',     meta: 'Pending', tone: 'tone-amber'   },
    { title: 'Contract Signed',      subtitle: 'DevDAO',        meta: 'Done',    tone: 'tone-emerald' },
    { title: 'Proposal Sent',        subtitle: 'Invest Club',   meta: 'Sent',    tone: 'tone-blue'    },
  ];

  readonly exchangePlatforms: ChipItem[] = [
    { label: 'AngelList', tone: 'tone-blue'    },
    { label: 'Deel',      tone: 'tone-emerald' },
    { label: 'Upwork',    tone: 'tone-amber'   },
    { label: 'Toptal',    tone: 'tone-purple'  },
  ];

  // ── MARKETPLACE ─────────────────────────────────────────────────────────────
  readonly marketplaceMetrics: Metric[] = [
    { label: 'Listed Items',   value: '284', caption: 'Skills · Assets', tone: 'tone-blue'    },
    { label: 'Active Orders',  value: '18',  caption: '$24K value',      tone: 'tone-teal'    },
    { label: 'Barter Offers',  value: '12',  caption: 'Active trades',   tone: 'tone-amber'   },
    { label: 'My Sales',       value: '$8,400', change: '18%', trend: 'up', caption: '',      tone: 'tone-emerald' },
    { label: 'My Purchases',   value: '$2,100', caption: 'This month',   tone: 'tone-purple'  },
  ];

  readonly marketplaceItems: CardItem[] = [
    { tag: 'Labor',    title: 'Full-Stack Dev',  value: '$120/hr',  tone: 'tone-blue'    },
    { tag: 'Asset',    title: 'Logo Design',     value: '$250',     tone: 'tone-purple'  },
    { tag: 'Service',  title: 'Brand Strategy',  value: '$3,500',   tone: 'tone-teal'    },
    { tag: 'Artifact', title: 'React Template',  value: '$89',      tone: 'tone-amber'   },
    { tag: 'Labor',    title: 'Data Analysis',   value: '$85/hr',   tone: 'tone-blue'    },
    { tag: 'Service',  title: 'Legal Review',    value: '$200/hr',  tone: 'tone-rose'    },
    { tag: 'Barter',   title: 'Office Chair',    value: 'Trade',    tone: 'tone-emerald' },
    { tag: 'Asset',    title: '3D Models Pack',  value: '$149',     tone: 'tone-cyan'    },
    { tag: 'Labor',    title: 'Content Writing', value: '$0.12/wd', tone: 'tone-indigo'  },
    { tag: 'Resource', title: 'API Access',      value: '$29/mo',   tone: 'tone-teal'    },
    { tag: 'Barter',   title: 'Photography Kit', value: 'Trade',    tone: 'tone-purple'  },
    { tag: 'Service',  title: 'Copywriting',     value: '$1,200',   tone: 'tone-rose'    },
  ];

  readonly marketplaceBarter: CardItem[] = [
    { title: 'Camera gear',    subtitle: 'Want: Laptop',      tone: 'tone-amber'  },
    { title: 'Adobe License',  subtitle: 'Want: Web dev',     tone: 'tone-blue'   },
    { title: 'Studio Time',    subtitle: 'Want: Design work', tone: 'tone-purple' },
  ];

  readonly marketplaceOrders: CardItem[] = [
    { title: 'Brand Package', subtitle: 'Active',    value: '$3,500', tone: 'tone-blue'   },
    { title: 'Dev Hours',     subtitle: 'Delivered', value: '$1,200', tone: 'tone-emerald'},
    { title: 'Legal Review',  subtitle: 'Pending',   value: '$200',   tone: 'tone-amber'  },
  ];

  readonly marketplacePlatforms: ChipItem[] = [
    { label: 'Behance', tone: 'tone-blue'   },
    { label: 'Upwork',  tone: 'tone-teal'   },
    { label: 'Fiverr',  tone: 'tone-rose'   },
    { label: 'Etsy',    tone: 'tone-amber'  },
  ];

  // ── STUDIO ──────────────────────────────────────────────────────────────────
  readonly studioMetrics: Metric[] = [
    { label: 'Ideas',            value: '42',  caption: 'In development', tone: 'tone-purple'  },
    { label: 'Prototypes',       value: '8',   caption: '3 testing',      tone: 'tone-blue',    progress: 38 },
    { label: 'Published Assets', value: '24',  caption: 'Available',      tone: 'tone-emerald' },
    { label: 'Notes / Binders',  value: '138', caption: 'Organized',      tone: 'tone-amber'   },
  ];

  readonly studioIdeas: CardItem[] = [
    { tag: 'Prototype',  title: 'Mobile App Concept', tone: 'tone-blue'    },
    { tag: 'Blueprint',  title: 'Tokenomics v3',      tone: 'tone-purple'  },
    { tag: 'Mockup',     title: 'Landing Page',       tone: 'tone-rose'    },
    { tag: 'Design',     title: 'AI Agent UX',        tone: 'tone-amber'   },
    { tag: 'Document',   title: 'API Spec v2',        tone: 'tone-teal'    },
    { tag: 'Asset',      title: 'Brand System',       tone: 'tone-emerald' },
    { tag: 'Draft',      title: 'Community RFC',      tone: 'tone-cyan'    },
    { tag: 'Blueprint',  title: 'Data Schema',        tone: 'tone-indigo'  },
  ];

  readonly studioTestbeds: CardItem[] = [
    { title: 'A/B Test: Onboarding',tone: 'tone-blue'   },
    { title: 'Perf Benchmark',       tone: 'tone-teal'   },
    { title: 'API Load Test',        tone: 'tone-amber'  },
    { title: 'UX Usability Study',   tone: 'tone-purple' },
    { title: 'Payment Flow Test',    tone: 'tone-rose'   },
    { title: 'AI Prompt Eval',       tone: 'tone-cyan'   },
  ];

  readonly studioToolsets: CardItem[] = [
    { title: 'Design Tools', subtitle: 'Figma · Framer',          tone: 'tone-purple' },
    { title: 'Dev Stack',    subtitle: 'Node · React · PG',        tone: 'tone-blue'   },
    { title: 'AI Toolkit',   subtitle: 'Claude · GPT · Grok',      tone: 'tone-emerald'},
    { title: 'Analytics',    subtitle: 'Posthog · Mixpanel',       tone: 'tone-amber'  },
  ];

  readonly studioFiles: CardItem[] = [
    { title: 'Binders',  tone: 'tone-blue'    },
    { title: 'Books',    tone: 'tone-purple'  },
    { title: 'Content',  tone: 'tone-teal'    },
    { title: 'Files',    tone: 'tone-amber'   },
  ];

  readonly studioPlatforms: ChipItem[] = [
    { label: 'Google Drive', tone: 'tone-blue'   },
    { label: 'Figma',        tone: 'tone-purple' },
    { label: 'Notion',       tone: 'tone-teal'   },
    { label: 'MS Teams',     tone: 'tone-indigo' },
  ];

  // ── COMMUNITY ───────────────────────────────────────────────────────────────
  readonly communityMetrics: Metric[] = [
    { label: 'Members',       value: '8,420', caption: '+12%',          tone: 'tone-rose'   },
    { label: 'Active Spaces', value: '34',    caption: '12 rooms open', tone: 'tone-teal'   },
    { label: 'Posts Today',   value: '284',   caption: '+18% vs yesterday', tone: 'tone-purple' },
    { label: 'DMs Unread',    value: '7',     caption: 'Priority',      tone: 'tone-amber'  },
  ];

  readonly communityFeeds: FeedItem[] = [
    { author: 'Jordan C.', message: 'New project launched! Brand redesign for a VC-backed startup 🚀', time: '2min ago', likes: '12' },
    { author: 'Maria S.',  message: 'Coop milestone: $1M ARR achieved 🎉 Proud of our 35-member team!', time: '8min ago', likes: '48' },
    { author: 'Devon P.',  message: 'Side project update: photography clients ×3 this week!',           time: '22min ago',likes: '23' },
    { author: 'Priya N.',  message: 'Investment club vote: 87% approve new property acquisition',       time: '1hr ago',  likes: '31' },
  ];

  readonly communitySpaces: CardItem[] = [
    { title: '⊞ #Office',   subtitle: '21 online', tone: 'tone-blue'   },
    { title: '🎯 #Finance', subtitle: '18 online', tone: 'tone-emerald'},
    { title: '🎨 #Studio',  subtitle: '31 online', tone: 'tone-purple' },
    { title: '🌐 #General', subtitle: '12 online', tone: 'tone-cyan'   },
    { title: '🏛️ #Coops',  subtitle: '9 online',  tone: 'tone-rose'   },
  ];

  readonly communityDms: CardItem[] = [
    { title: 'Jordan C.', subtitle: 'Hey, quick question...', tone: 'tone-blue'    },
    { title: 'Maria S.',  subtitle: 'Hey, quick question...', tone: 'tone-emerald' },
    { title: 'Devon P.',  subtitle: 'Hey, quick question...', tone: 'tone-amber'   },
  ];

  readonly communityPlatforms: ChipItem[] = [
    { label: 'Slack',     tone: 'tone-purple'  },
    { label: 'Zoom',      tone: 'tone-emerald' },
    { label: 'Discord',   tone: 'tone-blue'    },
    { label: 'WhatsApp',  tone: 'tone-amber'   },
  ];

  // ── DEVELOPER ───────────────────────────────────────────────────────────────
  readonly developerMetrics: Metric[] = [
    { label: 'API Calls (30d)', value: '284K',  caption: '+34%',        tone: 'tone-purple' },
    { label: 'Active Keys',     value: '5',     caption: '2 production', tone: 'tone-blue'   },
    { label: 'Webhooks',        value: '12',    caption: '8 active',     tone: 'tone-teal',  progress: 66 },
    { label: 'SDK Downloads',   value: '1,240', caption: '+28%',        tone: 'tone-indigo' },
  ];

  readonly developerApi: ApiRow[] = [
    { method: 'POST',   path: '/api/v1/portfolio',      description: 'Create portfolio item',  tone: 'tone-emerald' },
    { method: 'GET',    path: '/api/v1/projects',        description: 'List all projects',      tone: 'tone-blue'    },
    { method: 'PUT',    path: '/api/v1/exchange/bids',   description: 'Update a bid',           tone: 'tone-amber'   },
    { method: 'DELETE', path: '/api/v1/wallet/tokens',   description: 'Remove token',           tone: 'tone-rose'    },
    { method: 'POST',   path: '/api/v1/community/post',  description: 'Create community post',  tone: 'tone-purple'  },
    { method: 'GET',    path: '/api/v1/ai/context',      description: 'Get AI context window',  tone: 'tone-cyan'    },
  ];

  readonly developerExtensions: CardItem[] = [
    { title: 'Webhooks',      subtitle: 'Automation hooks',   tone: 'tone-blue'   },
    { title: 'OAuth 2.0',     subtitle: 'Identity + access',  tone: 'tone-emerald'},
    { title: 'Zapier',        subtitle: 'Workflow',           tone: 'tone-amber'  },
    { title: 'n8n',           subtitle: 'Pipelines',          tone: 'tone-purple' },
    { title: 'Slack Bot',     subtitle: 'Chat ops',           tone: 'tone-teal'   },
    { title: 'GitHub Action', subtitle: 'CI/CD',              tone: 'tone-indigo' },
    { title: 'Chrome Ext',    subtitle: 'Browser tools',      tone: 'tone-rose'   },
    { title: 'VS Code Ext',   subtitle: 'Dev experience',     tone: 'tone-cyan'   },
  ];

  readonly developerKeys: CardItem[] = [
    { title: 'prod_k1_****', subtitle: 'Production', meta: 'Active', tone: 'tone-emerald' },
    { title: 'dev_k2_****',  subtitle: 'Development',meta: 'Active', tone: 'tone-blue'    },
    { title: 'test_k3_****', subtitle: 'Testing',    meta: 'Active', tone: 'tone-amber'   },
  ];

  readonly developerSdks: ChipItem[] = [
    { label: 'Node.js', tone: 'tone-emerald' },
    { label: 'Python',  tone: 'tone-blue'    },
    { label: 'Go',      tone: 'tone-teal'    },
    { label: 'Rust',    tone: 'tone-amber'   },
  ];

  readonly developerWebhooks: CardItem[] = [
    { title: 'portfolio.created', subtitle: 'Active', meta: 'Live', tone: 'tone-emerald' },
    { title: 'payment.received',  subtitle: 'Active', meta: 'Live', tone: 'tone-blue'    },
    { title: 'bid.accepted',      subtitle: 'Active', meta: 'Live', tone: 'tone-amber'   },
    { title: 'project.updated',   subtitle: 'Active', meta: 'Live', tone: 'tone-purple'  },
  ];

  // ── PROFILE ─────────────────────────────────────────────────────────────────
  readonly profileMetrics: Metric[] = [
    { label: 'Reputation',    value: '94/100', caption: '+3%',        tone: 'tone-amber'   },
    { label: 'Network',       value: '1,240',  caption: 'Connections',tone: 'tone-purple'  },
    { label: 'Projects Done', value: '48',     caption: '+8%',        tone: 'tone-blue'    },
    { label: 'Earnings YTD',  value: '$186K',  caption: '+22%',       tone: 'tone-emerald' },
  ];

  readonly profilePersonas: CardItem[] = [
    { tag: '🧑‍💻', title: 'Developer',  subtitle: 'Full-stack · Node · React', tone: 'tone-blue'    },
    { tag: '🎨',  title: 'Designer',   subtitle: 'Brand · UX · Systems',      tone: 'tone-purple'  },
    { tag: '📊',  title: 'Strategist', subtitle: 'Product · Growth',          tone: 'tone-emerald' },
  ];

  readonly profileSettings: CardItem[] = [
    { title: 'Preferences',   tone: 'tone-amber'   },
    { title: 'Notifications', tone: 'tone-purple'  },
    { title: 'Privacy',       tone: 'tone-rose'    },
    { title: 'Security',      tone: 'tone-emerald' },
    { title: 'Billing',       tone: 'tone-amber'   },
    { title: 'Integrations',  tone: 'tone-teal'    },
    { title: 'AI Config',     tone: 'tone-indigo'  },
    { title: 'Data Export',   tone: 'tone-cyan'    },
  ];

  readonly profileStats: Metric[] = [
    { label: 'Messages',      value: '284', tone: 'tone-rose'   },
    { label: 'Proposals',     value: '18',  tone: 'tone-blue'   },
    { label: 'Contributions', value: '142', tone: 'tone-indigo' },
    { label: 'Reviews',       value: '37',  tone: 'tone-amber'  },
    { label: 'Referrals',     value: '12',  tone: 'tone-emerald'},
  ];

  // ── ORGANIZATIONS ────────────────────────────────────────────────────────────
  readonly orgMetrics: Metric[] = [
    { label: 'Organizations', value: '4',   caption: 'Member of',      tone: 'tone-rose'    },
    { label: 'Collectives',   value: '2',   caption: 'Co-founded',     tone: 'tone-blue'    },
    { label: 'Teams',         value: '3',   caption: 'Active',         tone: 'tone-teal'    },
    { label: 'Total Members', value: '186', caption: 'Across all orgs',tone: 'tone-emerald' },
  ];

  readonly orgCards: CardItem[] = [
    { title: 'Delivery Coop',      subtitle: 'Worker Cooperative',  meta: '35 members', status: 'Active',  tone: 'tone-emerald' },
    { title: 'Design Collective',  subtitle: 'Creative Collective', meta: '12 members', status: 'Active',  tone: 'tone-purple'  },
    { title: 'Invest Club',        subtitle: 'Investment Club',     meta: '22 members', status: 'Active',  tone: 'tone-blue'    },
    { title: 'Dev DAO',            subtitle: 'Autonomous Org',      meta: '88 members', status: 'Active',  tone: 'tone-indigo'  },
    { title: 'Consulting Network', subtitle: 'Professional Network',meta: '18 members', status: 'Pending', tone: 'tone-amber'   },
    { title: 'Art Cooperative',    subtitle: 'Creative Cooperative',meta: '7 members',  status: 'Forming', tone: 'tone-rose'    },
  ];

  readonly orgProposals: TableRow[] = [
    { cells: ['New vehicle fleet','Delivery Coop','32/35','Passed'], badge: 'Passed', badgeTone: 'tone-emerald' },
    { cells: ['Q2 Budget',        'Invest Club',  '18/22','Active'], badge: 'Active', badgeTone: 'tone-blue'    },
    { cells: ['Member Charter',   'Dev DAO',      '72/88','Active'], badge: 'Active', badgeTone: 'tone-purple'  },
    { cells: ['Rate increase',    'Design Coll.', '9/12', 'Draft'],  badge: 'Draft',  badgeTone: 'tone-amber'   },
  ];

  readonly orgRoles: CardItem[] = [
    { title: 'Delivery Coop',  subtitle: 'Co-Founder', tone: 'tone-emerald' },
    { title: 'Invest Club',    subtitle: 'Organizer',   tone: 'tone-blue'    },
    { title: 'Dev DAO',        subtitle: 'Member',      tone: 'tone-indigo'  },
    { title: 'Design Coll.',   subtitle: 'Member',      tone: 'tone-purple'  },
  ];

  readonly orgCapTables: CardItem[] = [
    { title: 'JC', subtitle: '18.5%', tone: 'tone-blue'    },
    { title: 'MS', subtitle: '14.2%', tone: 'tone-emerald' },
    { title: 'DP', subtitle: '8.8%',  tone: 'tone-amber'   },
    { title: 'PN', subtitle: '6.4%',  tone: 'tone-rose'    },
  ];

  // ── LEGAL ────────────────────────────────────────────────────────────────────
  readonly legalMetrics: Metric[] = [
    { label: 'Active Contracts',value: '14',   caption: '3 expiring soon',  tone: 'tone-rose'    },
    { label: 'IP Assets',       value: '8',    caption: 'Patents · TM · CR',tone: 'tone-purple'  },
    { label: 'Compliance Items',value: '6',    caption: '2 action needed',  tone: 'tone-amber'   },
    { label: 'Audit Status',    value: 'Clean',caption: 'Last: Mar 2026',   tone: 'tone-emerald' },
  ];

  readonly legalIp: CardItem[] = [
    { tag: 'Trademark', title: 'Kogi™',      status: 'Registered', tone: 'tone-rose'    },
    { tag: 'Patent',    title: 'Kogi OS',    status: 'Pending',    tone: 'tone-amber'   },
    { tag: 'Copyright', title: 'Logo System',status: 'Active',     tone: 'tone-emerald' },
    { tag: 'Copyright', title: 'API Spec',   status: 'Active',     tone: 'tone-rose'    },
  ];

  readonly legalContracts: TableRow[] = [
    { cells: ['MSA — TechCorp', 'TechCorp Inc.', '$120K',  'Dec 2026','Active'],   badge: 'Active',   badgeTone: 'tone-emerald' },
    { cells: ['NDA — StartupX', 'StartupX LLC',  '—',      'Jun 2026','Active'],   badge: 'Active',   badgeTone: 'tone-emerald' },
    { cells: ['Coop Charter',   'Members (35)',   '—',      '—',       'Active'],   badge: 'Active',   badgeTone: 'tone-emerald' },
    { cells: ['Dev Contract',   'DevDAO',         '$48K',   'Mar 2026','Expiring'], badge: 'Expiring', badgeTone: 'tone-amber'   },
    { cells: ['License Agmt.',  '3rd Party',      '$8K/yr', 'Dec 2026','Active'],   badge: 'Active',   badgeTone: 'tone-emerald' },
  ];

  readonly legalCompliance: CardItem[] = [
    { title: 'GDPR Compliance', subtitle: 'Complete',     tone: 'tone-emerald' },
    { title: 'SOC 2 Type II',   subtitle: 'In Progress',  tone: 'tone-amber'   },
    { title: 'Tax Compliance',  subtitle: 'Complete',     tone: 'tone-emerald' },
    { title: 'AML / KYC',       subtitle: 'Review Needed',tone: 'tone-rose'    },
    { title: 'Data Residency',  subtitle: 'Complete',     tone: 'tone-teal'    },
    { title: 'IP Audit',        subtitle: 'Scheduled',    tone: 'tone-blue'    },
  ];

  readonly legalUpcoming: CardItem[] = [
    { title: 'DevDAO Contract expires', subtitle: 'In 7 days',  tone: 'tone-rose'   },
    { title: 'IP Audit scheduled',      subtitle: 'In 14 days', tone: 'tone-blue'   },
    { title: 'Q1 Tax filing',           subtitle: 'In 22 days', tone: 'tone-amber'  },
    { title: 'NDA renewal',             subtitle: 'In 45 days', tone: 'tone-purple' },
  ];

  readonly legalActions: CardItem[] = [
    { title: 'Draft NDA',       tone: 'tone-blue'   },
    { title: 'Submit Patent',   tone: 'tone-amber'  },
    { title: 'Renew Contract',  tone: 'tone-rose'   },
    { title: 'File Compliance', tone: 'tone-emerald'},
  ];
}

bootstrapApplication(AppComponent).catch((err) => console.error(err));
