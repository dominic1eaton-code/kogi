import { bootstrapApplication } from '@angular/platform-browser';
import { provideRouter, RouterOutlet, Routes, RouterLink, RouterLinkActive } from '@angular/router';
import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { provideAnimations } from '@angular/platform-browser/animations';

// ─── TYPES ────────────────────────────────────────────────────────────────────

interface NavItem {
  label: string;
  icon: string;
  route: string;
  badge?: number;
}

interface KanbanCard {
  title: string;
  tags: string[];
  pts: number;
  blocked?: boolean;
  blockReason?: string;
  aiNote?: string;
  progress?: number;
}

interface KanbanColumn {
  id: string;
  title: string;
  count: number;
  cards: KanbanCard[];
  moreCount?: number;
}

interface AgentMessage {
  id: number;
  role: string;
  content: string;
  meta: string;
  toolsUsed?: string;
  actions?: string[];
}

// ─── SHARED: SIDEBAR ──────────────────────────────────────────────────────────

@Component({
  selector: 'app-sidebar',
  standalone: true,
  imports: [CommonModule, RouterLink, RouterLinkActive],
  template: `
    <nav class="sidebar">
      <div class="sidebar-logo">
        <div class="logo-mark">K</div>
        <span class="logo-text">kogi</span>
      </div>
      <ul class="nav-list">
        @for (item of navItems; track item.route) {
          <li>
            <a [routerLink]="item.route" routerLinkActive="active" class="nav-item">
              <span class="nav-icon">{{ item.icon }}</span>
              <span class="nav-label">{{ item.label }}</span>
              @if (item.badge) {
                <span class="nav-badge">{{ item.badge }}</span>
              }
            </a>
          </li>
        }
      </ul>
      <div class="sidebar-user">
        <div class="user-avatar">JD</div>
        <div class="user-info">
          <div class="user-name">Jordan Davis</div>
          <div class="user-plan">Pro Plan</div>
        </div>
      </div>
    </nav>
  `
})
export class SidebarComponent {
  navItems: NavItem[] = [
    { label: 'Dashboard',   icon: '◇', route: '/dashboard' },
    { label: 'Portfolio',   icon: '▤', route: '/portfolio' },
    { label: 'Work Board',  icon: '▦', route: '/work-board',  badge: 3 },
    { label: 'Marketplace', icon: '◈', route: '/marketplace' },
    { label: 'Exchange',    icon: '⊙', route: '/exchange',    badge: 2 },
    { label: 'Idea Studio', icon: '⊚', route: '/idea-studio' },
    { label: 'Community',   icon: '◎', route: '/community',   badge: 7 },
    { label: 'Work & OKRs', icon: '◆', route: '/okrs' },
    { label: 'Funding',     icon: '◉', route: '/funding' },
    { label: 'Governance',  icon: '▣', route: '/governance',  badge: 1 },
    { label: 'AI Agent',    icon: '◈', route: '/ai-agent' },
    { label: 'Analytics',   icon: '◈', route: '/analytics' },
    { label: 'Search',      icon: '⊙', route: '/search' },
    { label: 'Settings',    icon: '⊙', route: '/settings' },
  ];
}

// ─── DASHBOARD ────────────────────────────────────────────────────────────────

@Component({
  selector: 'app-dashboard',
  standalone: true,
  imports: [CommonModule, SidebarComponent],
  template: `
    <div class="page-layout">
      <app-sidebar></app-sidebar>
      <main class="main-content">
        <header class="topbar">
          <h1 class="topbar-title-text">Dashboard</h1>
          <div class="topbar-actions">
            <div class="search-box">
              <span>&#x2315;</span>
              <input type="text" placeholder="Search everything..." />
              <kbd>&#x2318;K</kbd>
            </div>
            <button class="icon-btn">&#x25D1;</button>
            <button class="icon-btn">?</button>
            <button class="btn-primary">+ New</button>
          </div>
        </header>
        <div class="page-body">
          <div class="greeting-banner">
            <div>
              <h2 class="greeting-text">Good morning, Jordan &#x1F44B;</h2>
              <p class="greeting-sub">You have 3 blocked stories and 2 invoices overdue. Your AI Agent has suggestions.</p>
            </div>
            <button class="btn-primary btn-lg">Ask AI Agent</button>
          </div>
          <div class="stats-grid">
            <div class="stat-card">
              <div class="stat-value">6</div>
              <div class="stat-label">Active Projects</div>
              <div class="stat-delta positive">&#x25B2; 2 this month</div>
            </div>
            <div class="stat-card">
              <div class="stat-value">73%</div>
              <div class="stat-label">Portfolio Health</div>
              <div class="stat-delta negative">&#x25BC; 4pts</div>
            </div>
            <div class="stat-card">
              <div class="stat-value">$4,820</div>
              <div class="stat-label">Wallet Balance</div>
              <div class="stat-delta positive">&#x25B2; $1,200 received</div>
            </div>
            <div class="stat-card">
              <div class="stat-value">2</div>
              <div class="stat-label">Pending Orders</div>
              <div class="stat-delta warning">1 needs response</div>
            </div>
            <div class="stat-card">
              <div class="stat-value">62%</div>
              <div class="stat-label">OKR Progress</div>
              <div class="stat-delta warning">Q3 · 28 days left</div>
            </div>
          </div>
          <div class="section-card">
            <div class="section-header">
              <div>
                <h3 class="section-title">Active Projects</h3>
                <span class="section-sub">6 projects · 3 at risk</span>
              </div>
              <a class="link-action" href="#">View all</a>
            </div>
            <div class="project-list">
              @for (project of activeProjects; track project.name) {
                <div class="project-row">
                  <div class="project-info">
                    <div class="project-dot" [class]="project.status"></div>
                    <span class="project-name">{{ project.name }}</span>
                    <span class="tag" [class]="'tag-' + project.status">{{ project.statusLabel }}</span>
                  </div>
                  <div class="project-meta">
                    <span class="project-client">{{ project.client }}</span>
                    <span class="project-due">{{ project.due }}</span>
                  </div>
                </div>
              }
            </div>
          </div>
        </div>
      </main>
    </div>
  `
})
export class DashboardComponent {
  activeProjects = [
    { name: 'Brand Identity Redesign',   client: 'Acme Corp',  due: 'Mar 20', status: 'on-track', statusLabel: 'On track' },
    { name: 'E-commerce Platform Build', client: 'ShopNow',    due: 'Mar 24', status: 'at-risk',  statusLabel: 'At risk'  },
    { name: 'Mobile App MVP',            client: 'StartupXYZ', due: 'Apr 5',  status: 'at-risk',  statusLabel: 'At risk'  },
    { name: 'Dashboard Analytics',       client: 'DataCo',     due: 'Apr 12', status: 'on-track', statusLabel: 'On track' },
  ];
}

// ─── WORK BOARD ───────────────────────────────────────────────────────────────

@Component({
  selector: 'app-work-board',
  standalone: true,
  imports: [CommonModule, SidebarComponent],
  template: `
    <div class="page-layout">
      <app-sidebar></app-sidebar>
      <main class="main-content">
        <header class="topbar">
          <div class="breadcrumb-nav">
            <span class="breadcrumb-text">Portfolio / Design Studio 2025 / E-commerce Build</span>
            <h1 class="topbar-title-text">E-commerce Platform Build</h1>
          </div>
          <div class="topbar-actions">
            <div class="search-box">
              <span>&#x2315;</span>
              <input type="text" placeholder="Search everything..." />
              <kbd>&#x2318;K</kbd>
            </div>
            <button class="icon-btn">&#x25D1;</button>
            <button class="icon-btn">?</button>
            <button class="btn-primary">+ Story</button>
          </div>
        </header>
        <div class="page-body">
          <div class="board-controls">
            <div class="tab-bar">
              @for (tab of tabs; track tab) {
                <button class="tab" [class.active]="tab === activeTab" (click)="activeTab = tab">{{ tab }}</button>
              }
            </div>
            <div class="board-meta">
              <span class="sprint-info">Sprint 4 of 6 · Mar 10&#x2013;24</span>
              <span class="tag tag-at-risk">At risk</span>
              <button class="btn-ghost">Filter</button>
              <button class="btn-ghost">Group by</button>
            </div>
          </div>
          <div class="kanban-board">
            @for (col of columns; track col.id) {
              <div class="kanban-col">
                <div class="col-header" [class]="'col-' + col.id">
                  <span class="col-title">{{ col.title }}</span>
                  <span class="col-count">{{ col.count }}</span>
                </div>
                <div class="col-cards">
                  @for (card of col.cards; track card.title) {
                    <div class="kanban-card" [class.blocked]="card.blocked">
                      <div class="card-title">{{ card.title }}</div>
                      @if (card.blockReason) {
                        <div class="card-block-msg">&#x26A0; {{ card.blockReason }}</div>
                      }
                      @if (card.aiNote) {
                        <div class="card-ai-note">&#x2736; {{ card.aiNote }}</div>
                      }
                      <div class="card-footer">
                        <div class="card-tags">
                          @for (tag of card.tags; track tag) {
                            <span class="tag" [class]="'tag-' + tag">{{ tag }}</span>
                          }
                        </div>
                        <span class="card-pts">{{ card.pts }} pts</span>
                      </div>
                      @if (card.progress !== undefined) {
                        <div class="progress-bar">
                          <div class="progress-fill" [style.width.%]="card.progress"></div>
                        </div>
                      }
                    </div>
                  }
                  @if (col.moreCount) {
                    <button class="more-btn">+ {{ col.moreCount }} more stories</button>
                  }
                </div>
              </div>
            }
          </div>
        </div>
      </main>
    </div>
  `
})
export class WorkBoardComponent {
  tabs = ['Board', 'List', 'Sprint', 'Timeline', 'Backlog'];
  activeTab = 'Board';

  columns: KanbanColumn[] = [
    {
      id: 'backlog', title: 'BACKLOG', count: 8,
      cards: [
        { title: 'Set up email notifications system', tags: ['feature'], pts: 3 },
        { title: 'Write API documentation',           tags: ['docs'],    pts: 2 },
        { title: 'Integrate analytics dashboard',     tags: ['feature'], pts: 5 },
      ],
      moreCount: 5
    },
    {
      id: 'in-progress', title: 'IN PROGRESS', count: 4,
      cards: [
        { title: 'Set up Stripe payment gateway',  tags: ['blocked'], pts: 5,  blocked: true, blockReason: 'Blocked — needs API key from client' },
        { title: 'Build product catalog page',     tags: ['feature'], pts: 8,  progress: 60 },
        { title: 'Implement cart & checkout flow', tags: ['feature'], pts: 13, progress: 30 },
      ]
    },
    {
      id: 'in-review', title: 'IN REVIEW', count: 2,
      cards: [
        { title: 'User authentication flows',       tags: ['review'], pts: 8 },
        { title: 'Design system component library', tags: ['review'], pts: 5 },
      ]
    },
    {
      id: 'done', title: 'DONE', count: 6,
      cards: [
        { title: 'Project setup & repo structure', tags: ['done'], pts: 3 },
        { title: 'Database schema design',         tags: ['done'], pts: 8 },
        { title: 'UI wireframes approved',         tags: ['done'], pts: 0 },
      ]
    },
    {
      id: 'blocked', title: 'BLOCKED', count: 2,
      cards: [
        { title: 'Set up API authentication with client CRM', tags: ['blocker'], pts: 0, blocked: true, blockReason: 'Blocked 3 days · No API key access',      aiNote: 'Found 2 solutions in marketplace →' },
        { title: 'Mobile responsive testing',                 tags: ['blocker'], pts: 0, blocked: true, blockReason: 'Needs real devices — request submitted' },
      ]
    },
  ];
}

// ─── PORTFOLIO ────────────────────────────────────────────────────────────────

@Component({
  selector: 'app-portfolio',
  standalone: true,
  imports: [CommonModule, SidebarComponent],
  template: `
    <div class="page-layout">
      <app-sidebar></app-sidebar>
      <main class="main-content">
        <header class="topbar">
          <h1 class="topbar-title-text">Portfolio</h1>
          <div class="topbar-actions">
            <div class="search-box"><span>&#x2315;</span><input placeholder="Search everything..." /><kbd>&#x2318;K</kbd></div>
            <button class="icon-btn">&#x25D1;</button>
            <button class="icon-btn">?</button>
            <button class="btn-primary">+ New Item</button>
          </div>
        </header>
        <div class="page-body">
          <div class="portfolio-header">
            <h2 class="section-title-lg">My Portfolio</h2>
            <div class="portfolio-meta">6 programs · 12 projects · 3 resources</div>
            <div class="view-toggles">
              <button class="btn-ghost">&#x2261; List</button>
              <button class="btn-ghost active">&#x25A6; Grid</button>
              <button class="btn-ghost">&#x25B3; Map</button>
            </div>
          </div>
          @for (program of programs; track program.name) {
            <div class="portfolio-card" [class]="'border-' + program.color">
              <div class="portfolio-card-header">
                <div>
                  <div class="portfolio-type">{{ program.type }}</div>
                  <h3 class="portfolio-name">{{ program.name }}</h3>
                  <p class="portfolio-desc">{{ program.description }}</p>
                </div>
                <span class="tag" [class]="'tag-' + (program.status === 'Active' ? 'on-track' : 'at-risk')">{{ program.status }}</span>
              </div>
              <div class="portfolio-stats">
                <div class="pstat"><span class="pstat-val">{{ program.projects }}</span><span class="pstat-lbl">Projects</span></div>
                <div class="pstat"><span class="pstat-val">{{ program.stories }}</span><span class="pstat-lbl">Stories</span></div>
                <div class="pstat"><span class="pstat-val green">{{ program.revenue }}</span><span class="pstat-lbl">{{ program.revenueLabel }}</span></div>
              </div>
              <div class="portfolio-progress">
                <div class="progress-bar">
                  <div class="progress-fill" [style.width]="program.health + '%'" [class]="'pf-' + program.color"></div>
                </div>
                <span class="progress-label">{{ program.health }}% portfolio health</span>
              </div>
            </div>
          }
          <div class="portfolio-card resources-card">
            <div class="resources-header">
              <div class="resource-icon">&#x25A6;</div>
              <div>
                <div class="portfolio-type">RESOURCES</div>
                <h3 class="portfolio-name">Assets &amp; Capital</h3>
              </div>
            </div>
            <div class="resource-items">
              <div class="resource-row"><span>Software licenses</span><span class="green">6 tools</span></div>
              <div class="resource-row"><span>Investment accounts</span><span class="green">$12,400</span></div>
              <div class="resource-row"><span>Equipment (laptop, etc)</span><span>3 items</span></div>
            </div>
          </div>
        </div>
      </main>
    </div>
  `
})
export class PortfolioComponent {
  programs = [
    { type: 'PROGRAM',       name: 'Design Studio 2025', description: 'Full-service brand design and digital product studio. 3 active client projects.', projects: 4, stories: 18, revenue: '$8.2K', revenueLabel: 'This month', health: 73, status: 'Active',  color: 'blue'   },
    { type: 'PROGRAM',       name: 'SaaS Product Dev',   description: 'Building KogiTask — a niche task management app. Early MVP phase.',               projects: 1, stories: 34, revenue: '$0',    revenueLabel: 'Revenue',    health: 38, status: 'At risk', color: 'red'    },
    { type: 'SUB-PORTFOLIO', name: 'Content & Education',description: 'Newsletter, YouTube channel, and course development. Q2 goals in progress.',       projects: 3, stories: 11, revenue: '$1.1K', revenueLabel: 'Monthly',    health: 55, status: 'Active',  color: 'yellow' },
  ];
}

// ─── EXCHANGE ─────────────────────────────────────────────────────────────────

@Component({
  selector: 'app-exchange',
  standalone: true,
  imports: [CommonModule, SidebarComponent],
  template: `
    <div class="page-layout">
      <app-sidebar></app-sidebar>
      <main class="main-content">
        <header class="topbar">
          <h1 class="topbar-title-text">Exchange</h1>
          <div class="topbar-actions">
            <div class="search-box"><span>&#x2315;</span><input placeholder="Search everything..." /><kbd>&#x2318;K</kbd></div>
            <button class="icon-btn">&#x25D1;</button>
            <button class="icon-btn">?</button>
            <button class="btn-primary">+ Add Funds</button>
          </div>
        </header>
        <div class="page-body">
          <div class="balance-hero">
            <div class="balance-label">Available Balance</div>
            <div class="balance-amount">$4,820.00</div>
            <div class="balance-meta">KYC Level 2 · Wire transfers enabled</div>
            <div class="balance-actions">
              <button class="btn-exchange active">&#x2191; Withdraw</button>
              <button class="btn-exchange">&#x2193; Deposit</button>
              <button class="btn-exchange">&#x2192; Send</button>
              <button class="btn-exchange">&#x25A6; Invoice</button>
            </div>
          </div>
          <div class="stats-grid">
            <div class="stat-card">
              <div class="stat-value">$12,340</div>
              <div class="stat-label">Revenue (YTD)</div>
              <div class="stat-delta positive">&#x25B2; $3,200 vs last year</div>
            </div>
            <div class="stat-card">
              <div class="stat-value">$2,180</div>
              <div class="stat-label">In Escrow</div>
              <div class="stat-delta warning">2 active orders</div>
            </div>
            <div class="stat-card">
              <div class="stat-value">$4,820</div>
              <div class="stat-label">Available</div>
              <div class="stat-delta positive">Ready to withdraw</div>
            </div>
            <div class="stat-card">
              <div class="stat-value">2</div>
              <div class="stat-label">Overdue Invoices</div>
              <div class="stat-delta negative">$3,400 outstanding</div>
            </div>
          </div>
          <div class="section-card">
            <div class="section-header">
              <h3 class="section-title">Transaction History</h3>
              <div class="tab-bar-sm">
                @for (t of txTabs; track t) {
                  <button class="tab-sm" [class.active]="t === activeTxTab" (click)="activeTxTab = t">{{ t }}</button>
                }
              </div>
            </div>
            <div class="tx-list">
              @for (tx of transactions; track tx.id) {
                <div class="tx-row">
                  <div class="tx-icon" [class]="'tx-' + tx.type">
                    {{ tx.type === 'income' ? '&#x2193;' : tx.type === 'expense' ? '&#x2191;' : '&#x2299;' }}
                  </div>
                  <div class="tx-info">
                    <div class="tx-title">{{ tx.title }}</div>
                    <div class="tx-meta">{{ tx.date }} · {{ tx.ref }}</div>
                  </div>
                  <div class="tx-amount" [class]="tx.type === 'income' ? 'positive' : 'negative'">
                    {{ tx.type === 'income' ? '+' : '-' }}{{ tx.amount }}
                  </div>
                </div>
              }
            </div>
          </div>
        </div>
      </main>
    </div>
  `
})
export class ExchangeComponent {
  txTabs = ['All', 'Income', 'Expenses', 'Escrow'];
  activeTxTab = 'All';
  transactions = [
    { id: 1, title: 'Brand Identity Project — Final payment', date: 'Mar 7, 2025',  ref: 'INV-2024-107', amount: '$1,200', type: 'income'  },
    { id: 2, title: 'Figma Professional license',            date: 'Mar 5, 2025',  ref: 'EXP-0042',     amount: '$15',    type: 'expense' },
    { id: 3, title: 'E-commerce Build — Milestone 2',        date: 'Mar 1, 2025',  ref: 'ESC-0019',     amount: '$2,180', type: 'escrow'  },
    { id: 4, title: 'Logo Design Package',                   date: 'Feb 28, 2025', ref: 'INV-2024-106', amount: '$850',   type: 'income'  },
    { id: 5, title: 'Adobe Creative Cloud',                  date: 'Feb 25, 2025', ref: 'EXP-0041',     amount: '$55',    type: 'expense' },
  ];
}

// ─── MARKETPLACE ──────────────────────────────────────────────────────────────

@Component({
  selector: 'app-marketplace',
  standalone: true,
  imports: [CommonModule, FormsModule, SidebarComponent],
  template: `
    <div class="page-layout">
      <app-sidebar></app-sidebar>
      <main class="main-content">
        <header class="topbar">
          <h1 class="topbar-title-text">Marketplace</h1>
          <div class="topbar-actions">
            <div class="search-box"><span>&#x2315;</span><input placeholder="Search everything..." /><kbd>&#x2318;K</kbd></div>
            <button class="icon-btn">&#x25D1;</button>
            <button class="icon-btn">?</button>
            <button class="btn-primary">+ List a Service</button>
          </div>
        </header>
        <div class="page-body">
          <div class="marketplace-hero">
            <h2>Find independent talent &amp; services</h2>
            <p class="hero-sub">2.5% platform fee · Escrow protected · 4,200+ active listings</p>
            <div class="market-search">
              <div class="market-search-input">
                <span>&#x2315;</span>
                <input [(ngModel)]="searchQuery" placeholder="React developer for SaaS startup..." />
              </div>
              <select class="category-select">
                <option>All Categories</option>
                <option>Development</option>
                <option>Design</option>
                <option>Writing &amp; Content</option>
                <option>Marketing</option>
                <option>Data &amp; Analytics</option>
              </select>
              <button class="btn-primary">Search</button>
            </div>
            <div class="popular-tags">
              <span class="tag-label">Popular:</span>
              @for (tag of popularTags; track tag.name) {
                <span class="tag-chip" [class]="'chip-' + tag.color">{{ tag.name }}</span>
              }
            </div>
          </div>
          <div class="marketplace-layout">
            <aside class="filters-panel">
              <div class="filter-group">
                <h4 class="filter-title">Category</h4>
                @for (cat of categories; track cat.name) {
                  <div class="filter-row" [class.active]="cat.name === 'All Services'">
                    <span>{{ cat.name }}</span>
                    <span class="filter-count">{{ cat.count }}</span>
                  </div>
                }
              </div>
              <div class="filter-group">
                <h4 class="filter-title">Budget</h4>
                <div class="budget-inputs">
                  <input type="text" value="$50" class="budget-input" />
                  <span>&#x2014;</span>
                  <input type="text" value="$500" class="budget-input" />
                </div>
                <div class="range-bar"><div class="range-fill"></div></div>
              </div>
              <div class="filter-group">
                <h4 class="filter-title">Filters</h4>
                <label class="checkbox-row"><input type="checkbox" checked /> Top Rated (4.8+)</label>
                <label class="checkbox-row"><input type="checkbox" /> Kogi Verified</label>
                <label class="checkbox-row"><input type="checkbox" /> Fast delivery (&lt; 48h)</label>
                <label class="checkbox-row"><input type="checkbox" checked /> Portfolio shown</label>
              </div>
            </aside>
            <div class="results-area">
              <div class="results-meta">
                <span>Showing 1&#x2013;12 of 1,820 results</span>
                <select class="sort-select">
                  <option>Best Match</option>
                  <option>Newest</option>
                  <option>Price: Low to High</option>
                </select>
              </div>
              <div class="listing-grid">
                @for (listing of listings; track listing.title) {
                  <div class="listing-card">
                    <div class="listing-thumb" [style.background]="listing.bg">
                      @if (listing.aiMatch) { <span class="ai-match-badge">AI Match</span> }
                      <span class="listing-icon">{{ listing.icon }}</span>
                    </div>
                    <div class="listing-body">
                      <h4 class="listing-title">{{ listing.title }}</h4>
                      <div class="listing-seller">
                        <span class="seller-avatar">{{ listing.sellerInitials }}</span>
                        <span class="seller-name">{{ listing.seller }}</span>
                        <span class="verified-badge">Verified</span>
                      </div>
                      <div class="listing-rating">
                        <span class="stars">&#x2605; {{ listing.rating }}</span>
                        <span class="review-count">({{ listing.reviews }})</span>
                        <span class="listing-price">{{ listing.price }}</span>
                      </div>
                    </div>
                  </div>
                }
              </div>
            </div>
          </div>
        </div>
      </main>
    </div>
  `
})
export class MarketplaceComponent {
  searchQuery = '';
  popularTags = [
    { name: 'React developer', color: 'blue'    },
    { name: 'Logo design',     color: 'purple'  },
    { name: 'SEO writing',     color: 'green'   },
    { name: 'Data analysis',   color: 'default' },
    { name: 'Video editing',   color: 'orange'  },
  ];
  categories = [
    { name: 'All Services',     count: 4200 },
    { name: 'Development',      count: 1820 },
    { name: 'Design',           count: 960  },
    { name: 'Writing & Content',count: 640  },
    { name: 'Marketing',        count: 380  },
    { name: 'Data & Analytics', count: 240  },
  ];
  listings = [
    { title: 'React / Next.js Full-Stack Development',        seller: 'Alex Kim', sellerInitials: 'AK', rating: 4.9, reviews: 124, price: 'from $120/hr',   bg: 'linear-gradient(135deg,#1a2a4a,#0d1b35)', icon: '🖥', aiMatch: false },
    { title: 'Brand Identity Design — Logo + Guidelines',     seller: 'Sofia M.', sellerInitials: 'SM', rating: 5.0, reviews: 88,  price: '$850 fixed',     bg: 'linear-gradient(135deg,#2d1a4a,#1a0d35)', icon: '✦', aiMatch: false },
    { title: 'Data Analysis & Business Intelligence Reports', seller: 'Rania N.', sellerInitials: 'RN', rating: 4.8, reviews: 62,  price: 'from $200/proj', bg: 'linear-gradient(135deg,#0d2b1a,#081a0f)', icon: '📊', aiMatch: true  },
    { title: 'UX Research & Usability Testing',              seller: 'Jess W.',  sellerInitials: 'JW', rating: 4.9, reviews: 47,  price: 'from $95/hr',    bg: 'linear-gradient(135deg,#2a1a0d,#1a0d08)', icon: '🔍', aiMatch: false },
    { title: 'SEO Content Strategy & Writing',               seller: 'Marc D.',  sellerInitials: 'MD', rating: 4.7, reviews: 93,  price: 'from $60/hr',    bg: 'linear-gradient(135deg,#1a2a1a,#0d1a0d)', icon: '✍', aiMatch: false },
    { title: 'Mobile App Development (iOS/Android)',          seller: 'Priya N.', sellerInitials: 'PN', rating: 5.0, reviews: 31,  price: 'from $150/hr',   bg: 'linear-gradient(135deg,#1a1a2a,#0d0d1a)', icon: '📱', aiMatch: true  },
  ];
}

// ─── GOVERNANCE ───────────────────────────────────────────────────────────────

@Component({
  selector: 'app-governance',
  standalone: true,
  imports: [CommonModule, SidebarComponent],
  template: `
    <div class="page-layout">
      <app-sidebar></app-sidebar>
      <main class="main-content">
        <header class="topbar">
          <h1 class="topbar-title-text">Governance &#x2014; TechWorkers Cooperative</h1>
          <div class="topbar-actions">
            <div class="search-box"><span>&#x2315;</span><input placeholder="Search everything..." /><kbd>&#x2318;K</kbd></div>
            <button class="icon-btn">&#x25D1;</button>
            <button class="icon-btn">?</button>
            <button class="btn-primary">+ New Proposal</button>
          </div>
        </header>
        <div class="page-body">
          <div class="org-card">
            <div class="org-avatar">&#x25A6;</div>
            <div class="org-info">
              <h2 class="org-name">TechWorkers Cooperative</h2>
              <p class="org-meta">12 members · 1M1V voting · Est. 2023 · Charter v2.1</p>
            </div>
            <div class="org-stats">
              <div class="ostat"><span class="ostat-val">12</span><span class="ostat-lbl">Members</span></div>
              <div class="ostat"><span class="ostat-val">3</span><span class="ostat-lbl">Active proposals</span></div>
              <div class="ostat"><span class="ostat-val green">$84K</span><span class="ostat-lbl">Q4 Revenue</span></div>
              <div class="ostat"><span class="ostat-val">7/12</span><span class="ostat-lbl">Quorum met</span></div>
            </div>
          </div>
          <div class="tab-bar">
            @for (tab of tabs; track tab) {
              <button class="tab" [class.active]="tab === activeTab" (click)="activeTab = tab">{{ tab }}</button>
            }
          </div>
          <div class="proposals-list">
            <div class="proposal-card proposal-active">
              <div class="proposal-header">
                <div class="proposal-badges">
                  <span class="badge badge-open">Open</span>
                  <span class="badge badge-voting">Voting</span>
                  <span class="badge badge-id">PROP-2025-007</span>
                  <span class="badge">Capital Allocation</span>
                </div>
                <div class="proposal-closes">Closes in <span class="countdown">2 days 14h</span></div>
              </div>
              <h3 class="proposal-title">Allocate $8,000 from Q4 surplus for developer tooling</h3>
              <p class="proposal-desc">Purchase GitHub Copilot licenses, upgrade CI/CD infrastructure, and acquire 2 new dev laptops for incoming members.</p>
              <div class="vote-bar-container">
                <div class="vote-label positive">&#x2713; For: 7 votes</div>
                <div class="vote-label">Abstain: 2</div>
                <div class="vote-label negative">&#x2717; Against: 1</div>
              </div>
              <div class="vote-bar">
                <div class="vote-fill" style="width: 70%"></div>
                <div class="vote-against" style="width: 10%"></div>
              </div>
              <div class="vote-meta">7/10 required for passage (quorum met)</div>
              <div class="vote-actions">
                <button class="btn-vote-for">&#x2713; Vote For</button>
                <button class="btn-vote-against">&#x2717; Vote Against</button>
                <button class="btn-ghost">Abstain</button>
                <a class="link-action" href="#">View full proposal &#x2192;</a>
              </div>
            </div>
            <div class="proposal-card">
              <div class="proposal-header">
                <div class="proposal-badges">
                  <span class="badge badge-passed">Passed</span>
                  <span class="badge badge-executing">Executing</span>
                  <span class="badge badge-id">PROP-2025-006</span>
                  <span class="badge">Member Admission</span>
                </div>
              </div>
              <h3 class="proposal-title">Admit Priya Nair as full member (developer)</h3>
              <p class="proposal-desc">Passed 10&#x2013;0 on March 1, 2025. Onboarding documents sent. Equity stake allocated.</p>
              <div class="progress-bar"><div class="progress-fill" style="width: 75%"></div></div>
              <div class="step-status">Step 3/4 &#x2014; Wallet created, awaiting charter sign</div>
            </div>
            <div class="proposal-card">
              <div class="proposal-header">
                <div class="proposal-badges">
                  <span class="badge badge-draft">Draft</span>
                  <span class="badge badge-id">PROP-2025-008</span>
                  <span class="badge">Profit Distribution</span>
                </div>
                <button class="btn-primary btn-sm">Submit for vote</button>
              </div>
              <h3 class="proposal-title">Q4 2024 profit distribution &#x2014; $62,400 to members</h3>
              <p class="proposal-desc">Last edited by you · 2 hours ago · Waiting on final accounting sign-off</p>
            </div>
          </div>
          <div class="section-card">
            <h3 class="section-title">Members (12)</h3>
            <div class="members-list">
              @for (m of members; track m.name) {
                <div class="member-row">
                  <div class="member-avatar">{{ m.initials }}</div>
                  <div class="member-info">
                    <div class="member-name">{{ m.name }}{{ m.isYou ? ' (you)' : '' }}</div>
                    <div class="member-role">{{ m.role }} · {{ m.stake }} stake</div>
                  </div>
                  <span class="vote-status" [class]="'vs-' + m.voteStatus.toLowerCase()">{{ m.voteStatus }}</span>
                </div>
              }
            </div>
          </div>
        </div>
      </main>
    </div>
  `
})
export class GovernanceComponent {
  tabs = ['Proposals', 'Members', 'Charter', 'Treasury', 'Audit Log'];
  activeTab = 'Proposals';
  members = [
    { name: 'Jordan Davis',  initials: 'JD', role: 'Founder',  stake: '15%', voteStatus: 'Voted',   isYou: true  },
    { name: 'Alex Kim',      initials: 'AK', role: 'Developer', stake: '12%', voteStatus: 'Voted',   isYou: false },
    { name: 'Sofia Moreira', initials: 'SM', role: 'Designer',  stake: '10%', voteStatus: 'Pending', isYou: false },
    { name: 'Rania N.',      initials: 'RN', role: 'Analyst',   stake: '8%',  voteStatus: 'Voted',   isYou: false },
  ];
}

// ─── IDEA STUDIO ──────────────────────────────────────────────────────────────

@Component({
  selector: 'app-idea-studio',
  standalone: true,
  imports: [CommonModule, SidebarComponent],
  template: `
    <div class="page-layout">
      <app-sidebar></app-sidebar>
      <main class="main-content">
        <header class="topbar">
          <h1 class="topbar-title-text">Idea Studio</h1>
          <div class="topbar-actions">
            <div class="search-box"><span>&#x2315;</span><input placeholder="Search everything..." /><kbd>&#x2318;K</kbd></div>
            <button class="icon-btn">&#x25D1;</button>
            <button class="icon-btn">?</button>
            <button class="btn-primary">+ Capture Idea</button>
          </div>
        </header>
        <div class="page-body">
          <div class="idea-header">
            <div>
              <h2 class="section-title-lg">Idea Studio</h2>
              <p class="section-sub">5-stage innovation pipeline · 14 active ideas</p>
            </div>
            <div class="tab-bar">
              @for (v of views; track v) {
                <button class="tab" [class.active]="v === activeView" (click)="activeView = v">{{ v }}</button>
              }
            </div>
          </div>
          <div class="pipeline">
            @for (stage of stages; track stage.name) {
              <div class="pipeline-stage">
                <div class="stage-icon">{{ stage.icon }}</div>
                <div class="stage-name" [class]="'stage-color-' + stage.color">{{ stage.index }}. {{ stage.name }}</div>
                <div class="stage-count" [class]="'count-' + stage.color">{{ stage.count }}</div>
                <div class="stage-label">ideas</div>
              </div>
            }
          </div>
          <div class="section-card idea-featured">
            <div class="idea-badges">
              <span class="badge badge-stage">Stage 3 &#x2014; Design</span>
              <span class="badge badge-saas">SaaS</span>
              <span class="badge">IP Timestamped &#x1F512;</span>
            </div>
            <div class="idea-content">
              <h3 class="idea-title">AI-powered resume scoring for freelancers</h3>
              <div class="validation-score">
                <span class="vs-label">Validation score</span>
                <span class="vs-value">84/100</span>
              </div>
            </div>
            <div class="idea-meta-row">
              <div class="idea-tags">
                <span class="tag tag-feature">Market research done</span>
                <span class="tag tag-docs">Competitor analysis</span>
                <span class="tag">3 mockups</span>
              </div>
              <button class="btn-primary btn-sm">Continue &#x2192;</button>
            </div>
          </div>
        </div>
      </main>
    </div>
  `
})
export class IdeaStudioComponent {
  views = ['Pipeline', 'Gallery', 'Archive'];
  activeView = 'Pipeline';
  stages = [
    { index: 1, name: 'Capture',   icon: '★',  count: 6, color: 'blue'   },
    { index: 2, name: 'Validate',  icon: '🔍', count: 4, color: 'purple' },
    { index: 3, name: 'Design',    icon: '👁', count: 2, color: 'cyan'   },
    { index: 4, name: 'Prototype', icon: '🚀', count: 1, color: 'yellow' },
    { index: 5, name: 'Launch',    icon: '🎉', count: 1, color: 'green'  },
  ];
}

// ─── AI AGENT ─────────────────────────────────────────────────────────────────

@Component({
  selector: 'app-ai-agent',
  standalone: true,
  imports: [CommonModule, FormsModule, SidebarComponent],
  template: `
    <div class="page-layout">
      <app-sidebar></app-sidebar>
      <main class="main-content">
        <header class="topbar">
          <h1 class="topbar-title-text">AI Agent</h1>
          <div class="topbar-actions">
            <div class="search-box"><span>&#x2315;</span><input placeholder="Search everything..." /><kbd>&#x2318;K</kbd></div>
            <button class="icon-btn">&#x25D1;</button>
            <button class="icon-btn">?</button>
            <button class="btn-ghost">&#x1F4CB; History</button>
          </div>
        </header>
        <div class="ai-layout">
          <aside class="ai-context">
            <h3 class="context-title">Portfolio Context</h3>
            <div class="context-section">
              <div class="context-label">ACTIVE PROJECTS</div>
              <div class="context-project at-risk">
                <div class="cp-name">E-commerce Build</div>
                <div class="cp-meta">Sprint 4 · 2 blocked</div>
                <span class="tag tag-at-risk">At risk</span>
              </div>
              <div class="context-project on-track">
                <div class="cp-name">Brand Identity</div>
                <div class="cp-meta">4 in progress</div>
                <span class="tag tag-on-track">On track</span>
              </div>
            </div>
            <div class="context-section">
              <div class="context-label">WALLET</div>
              <div class="wallet-display">
                <div class="wallet-amount">$4,820</div>
                <div class="wallet-meta">Available · 2 invoices overdue</div>
              </div>
            </div>
            <div class="context-section">
              <div class="context-label">OKR STATUS</div>
              <div class="okr-row"><span>Revenue goal</span><span class="okr-pct">62%</span></div>
              <div class="progress-bar sm"><div class="progress-fill" style="width:62%"></div></div>
              <div class="okr-row"><span>New services</span><span class="okr-pct green">67%</span></div>
              <div class="progress-bar sm"><div class="progress-fill pf-green" style="width:67%"></div></div>
            </div>
            <div class="context-section">
              <div class="context-label">SUGGESTED ACTIONS</div>
              <button class="suggestion-btn red">&#x26A1; Unblock 2 blocked stories</button>
              <button class="suggestion-btn yellow">&#x25A6; Chase 2 overdue invoices</button>
              <button class="suggestion-btn blue">&#x1F4CA; OKR: revenue at risk</button>
            </div>
          </aside>
          <div class="chat-area">
            <div class="messages">
              @for (msg of messages; track msg.id) {
                <div class="message" [class]="'msg-' + msg.role">
                  @if (msg.role === 'agent') {
                    <div class="agent-avatar">&#x2736;</div>
                  }
                  <div class="msg-bubble" [class]="'bubble-' + msg.role">
                    @if (msg.toolsUsed) {
                      <div class="tools-used">Using: {{ msg.toolsUsed }}</div>
                    }
                    <div [innerHTML]="msg.content"></div>
                    @if (msg.actions && msg.actions.length) {
                      <div class="msg-actions">
                        @for (action of msg.actions; track action) {
                          <button class="btn-msg-action">{{ action }}</button>
                        }
                      </div>
                    }
                    <div class="msg-meta">{{ msg.meta }}</div>
                  </div>
                  @if (msg.role === 'user') {
                    <div class="user-avatar-chat">JD</div>
                  }
                </div>
              }
            </div>
            <div class="chat-input-area">
              <div class="quick-actions">
                <span class="qa-label">Quick actions:</span>
                @for (qa of quickActions; track qa) {
                  <button class="quick-chip">{{ qa }}</button>
                }
              </div>
              <div class="chat-input-row">
                <input [(ngModel)]="inputText"
                       placeholder="Ask anything about your portfolio, projects, finances..."
                       class="chat-input"
                       (keydown.enter)="sendMessage()" />
                <button class="icon-btn">&#64;</button>
                <button class="btn-send" (click)="sendMessage()">&#x25B6;</button>
              </div>
            </div>
          </div>
        </div>
      </main>
    </div>
  `
})
export class AiAgentComponent {
  inputText = '';
  quickActions = ['Plan sprint', 'Check OKRs', 'Create invoice', 'Search market'];

  messages: AgentMessage[] = [
    {
      id: 1, role: 'agent',
      content: `<p><strong>Good morning, Jordan.</strong> I've reviewed your portfolio and have 3 actionable insights:</p>
        <div class="insight blocked"><strong>&#x26A0; Blocked:</strong> "Set up Stripe payment gateway" has been blocked for 3 days. I found 2 Stripe-certified developers on the Marketplace.</div>
        <div class="insight overdue"><strong>&#x25A6; Overdue:</strong> Invoice INV-2024-108 for $2,000 is 14 days overdue from DesignCo. I can draft a polite payment reminder email.</div>
        <div class="insight okr"><strong>&#x1F4CA; OKR:</strong> Your Q3 revenue goal is at 62% with 28 days left. At current velocity you'll reach $12.4K vs $15K target.</div>`,
      actions: ['Send marketplace inquiries', 'Draft invoice reminder', 'Show OKR analysis'],
      meta: 'Kogi AI · 9:02 AM · 3 tools used'
    },
    {
      id: 2, role: 'user',
      content: 'Yes, send inquiries to both developers. Also, create a sprint planning summary for the E-commerce project.',
      meta: 'Jordan · 9:05 AM'
    },
    {
      id: 3, role: 'agent',
      toolsUsed: 'search_marketplace, read_wbs, create_document',
      content: `<p>&#x2713; <strong>Inquiries sent</strong> to Alex Kim and Marcus Chen on Marketplace</p>
        <p>&#x25A6; <strong>Sprint Summary generating...</strong></p>
        <div class="typing-dots"><span></span><span></span><span></span></div>`,
      meta: ''
    }
  ];

  sendMessage(): void {
    if (!this.inputText.trim()) return;
    this.messages.push({
      id: this.messages.length + 1,
      role: 'user',
      content: this.inputText,
      meta: 'Jordan · Just now'
    });
    this.inputText = '';
  }
}

// ─── ANALYTICS ────────────────────────────────────────────────────────────────

@Component({
  selector: 'app-analytics',
  standalone: true,
  imports: [CommonModule, SidebarComponent],
  template: `
    <div class="page-layout">
      <app-sidebar></app-sidebar>
      <main class="main-content">
        <header class="topbar">
          <h1 class="topbar-title-text">Analytics</h1>
          <div class="topbar-actions">
            <div class="search-box"><span>&#x2315;</span><input placeholder="Search everything..." /><kbd>&#x2318;K</kbd></div>
            <button class="icon-btn">&#x25D1;</button>
            <button class="icon-btn">?</button>
            <select class="date-select">
              <option>Last 30 days</option>
              <option>Last 90 days</option>
              <option>YTD</option>
            </select>
          </div>
        </header>
        <div class="page-body">
          <div class="analytics-header">
            <div>
              <h2 class="section-title-lg">Analytics Overview</h2>
              <p class="section-sub">Your platform performance · Feb 7 &#x2013; Mar 7, 2025</p>
            </div>
            <button class="btn-ghost">&#x2193; Export CSV</button>
          </div>
          <div class="stats-grid">
            <div class="stat-card">
              <div class="stat-value">$12,340</div>
              <div class="stat-label">Total Revenue (30d)</div>
              <div class="stat-delta positive">&#x25B2; 23% vs last period</div>
            </div>
            <div class="stat-card">
              <div class="stat-value">18</div>
              <div class="stat-label">Orders Completed</div>
              <div class="stat-delta positive">&#x25B2; 6 more</div>
            </div>
            <div class="stat-card">
              <div class="stat-value">147</div>
              <div class="stat-label">Profile Views</div>
              <div class="stat-delta positive">&#x25B2; 31% growth</div>
            </div>
            <div class="stat-card">
              <div class="stat-value">4.9&#x2605;</div>
              <div class="stat-label">Average Rating</div>
              <div class="stat-delta positive">Based on 42 reviews</div>
            </div>
            <div class="stat-card">
              <div class="stat-value">62%</div>
              <div class="stat-label">OKR Progress</div>
              <div class="stat-delta warning">Revenue goal Q3</div>
            </div>
          </div>
          <div class="section-card">
            <div class="section-header">
              <h3 class="section-title">Revenue Over Time</h3>
              <div class="tab-bar-sm">
                <button class="tab-sm active">Daily</button>
                <button class="tab-sm">Weekly</button>
              </div>
            </div>
            <div class="chart-area">
              <svg viewBox="0 0 800 160" class="revenue-chart">
                <defs>
                  <linearGradient id="revGrad" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stop-color="#3b82f6" stop-opacity="0.3"/>
                    <stop offset="100%" stop-color="#3b82f6" stop-opacity="0"/>
                  </linearGradient>
                </defs>
                <path d="M0,120 C50,110 80,90 120,80 C160,70 180,85 220,65 C260,45 280,70 320,50 C360,30 380,60 420,40 C460,20 490,50 530,35 C570,20 600,45 640,30 C680,15 720,35 760,20 L800,20 L800,160 L0,160 Z" fill="url(#revGrad)"/>
                <path d="M0,120 C50,110 80,90 120,80 C160,70 180,85 220,65 C260,45 280,70 320,50 C360,30 380,60 420,40 C460,20 490,50 530,35 C570,20 600,45 640,30 C680,15 720,35 760,20" fill="none" stroke="#3b82f6" stroke-width="2"/>
              </svg>
            </div>
          </div>
          <div class="section-card">
            <h3 class="section-title">Top Performing Services</h3>
            <div class="service-table">
              @for (svc of topServices; track svc.name) {
                <div class="service-row">
                  <div class="svc-name">{{ svc.name }}</div>
                  <div class="svc-bar-wrap">
                    <div class="svc-bar"><div class="svc-bar-fill" [style.width.%]="svc.pct"></div></div>
                  </div>
                  <div class="svc-rev green">{{ svc.rev }}</div>
                  <div class="svc-orders">{{ svc.orders }} orders</div>
                </div>
              }
            </div>
          </div>
        </div>
      </main>
    </div>
  `
})
export class AnalyticsComponent {
  topServices = [
    { name: 'Brand Identity Design', pct: 85, rev: '$5,400', orders: 6 },
    { name: 'Product Strategy',      pct: 65, rev: '$3,800', orders: 4 },
    { name: 'UI/UX Design',          pct: 50, rev: '$2,200', orders: 5 },
    { name: 'Design Systems',        pct: 30, rev: '$940',   orders: 3 },
  ];
}

// ─── PROFILE ──────────────────────────────────────────────────────────────────

@Component({
  selector: 'app-profile',
  standalone: true,
  imports: [CommonModule, FormsModule, SidebarComponent],
  template: `
    <div class="page-layout">
      <app-sidebar></app-sidebar>
      <main class="main-content">
        <header class="topbar">
          <h1 class="topbar-title-text">My Profile</h1>
          <div class="topbar-actions">
            <div class="search-box"><span>&#x2315;</span><input placeholder="Search everything..." /><kbd>&#x2318;K</kbd></div>
            <button class="icon-btn">&#x25D1;</button>
            <button class="icon-btn">?</button>
            <button class="btn-ghost">Preview public page</button>
            <button class="btn-primary">Save Changes</button>
          </div>
        </header>
        <div class="page-body">
          <div class="tab-bar">
            @for (tab of tabs; track tab) {
              <button class="tab" [class.active]="tab === activeTab" (click)="activeTab = tab">{{ tab }}</button>
            }
          </div>
          <div class="profile-layout">
            <div class="avatar-card">
              <div class="profile-avatar">JD
                <div class="avatar-status"></div>
              </div>
              <h2 class="profile-name">Jordan Davis</h2>
              <p class="profile-title-text">Brand Designer &amp; Product Strategist</p>
              <span class="available-badge">Available for work</span>
              <p class="profile-rating">&#x2605; 4.9 | 42 reviews | 3yr on Kogi</p>
            </div>
            <div class="profile-forms">
              <div class="form-section">
                <h3 class="form-section-title">Availability</h3>
                <div class="form-row"><span class="form-label">Status</span><span class="green">Open to work</span></div>
                <div class="form-row"><span class="form-label">Capacity</span><span>20 hrs/week</span></div>
                <div class="form-row"><span class="form-label">Rate</span><span>$120&#x2013;180/hr</span></div>
                <div class="form-row"><span class="form-label">Timezone</span><span>UTC-5 (EST)</span></div>
              </div>
              <div class="form-section">
                <h3 class="form-section-title">Top Skills</h3>
                <div class="skills-tags">
                  @for (skill of skills; track skill.name) {
                    <span class="skill-tag" [class]="'skill-' + skill.color">{{ skill.name }}</span>
                  }
                </div>
              </div>
              <div class="form-section">
                <h3 class="form-section-title">Basic Information</h3>
                <div class="form-grid-2">
                  <div class="form-field">
                    <label>First name</label>
                    <input [(ngModel)]="firstName" class="form-input" />
                  </div>
                  <div class="form-field">
                    <label>Last name</label>
                    <input [(ngModel)]="lastName" class="form-input" />
                  </div>
                </div>
                <div class="form-field">
                  <label>Professional title</label>
                  <input [(ngModel)]="professionalTitle" class="form-input form-input-active" />
                </div>
                <div class="form-field">
                  <label>Bio (visible on public profile)</label>
                  <textarea [(ngModel)]="bio" class="form-textarea" rows="4"></textarea>
                </div>
              </div>
            </div>
          </div>
        </div>
      </main>
    </div>
  `
})
export class ProfileComponent {
  tabs = ['Profile & Identity', 'Portfolio Showcase', 'Skills & Certifications', 'Reviews', 'Connections'];
  activeTab = 'Profile & Identity';
  firstName = 'Jordan';
  lastName = 'Davis';
  professionalTitle = 'Brand Designer & Product Strategist';
  bio = 'I help startups and independent businesses build brand identities that convert. 8 years designing for 120+ clients across SaaS, fintech, and e-commerce. I believe great design is invisible — and so is great strategy.';
  skills = [
    { name: 'Brand Identity',   color: 'blue'    },
    { name: 'UI/UX Design',     color: 'purple'  },
    { name: 'Figma',            color: 'cyan'    },
    { name: 'Product Strategy', color: 'green'   },
    { name: 'Design Systems',   color: 'pink'    },
    { name: 'Webflow',          color: 'default' },
  ];
}

// ─── SETTINGS ─────────────────────────────────────────────────────────────────

@Component({
  selector: 'app-settings',
  standalone: true,
  imports: [CommonModule, SidebarComponent],
  template: `
    <div class="page-layout">
      <app-sidebar></app-sidebar>
      <main class="main-content">
        <header class="topbar">
          <h1 class="topbar-title-text">Settings</h1>
          <div class="topbar-actions">
            <div class="search-box"><span>&#x2315;</span><input placeholder="Search everything..." /><kbd>&#x2318;K</kbd></div>
            <button class="icon-btn">&#x25D1;</button>
            <button class="icon-btn">?</button>
          </div>
        </header>
        <div class="page-body">
          <div class="settings-layout">
            <aside class="settings-nav">
              <div class="settings-group">
                <div class="settings-group-title">ACCOUNT</div>
                @for (item of accountItems; track item) {
                  <div class="settings-item" [class.active]="item === activeItem" (click)="activeItem = item">{{ item }}</div>
                }
              </div>
              <div class="settings-group">
                <div class="settings-group-title">SECURITY</div>
                @for (item of securityItems; track item) {
                  <div class="settings-item" [class.active]="item === activeItem" (click)="activeItem = item">{{ item }}</div>
                }
              </div>
              <div class="settings-group">
                <div class="settings-group-title">BILLING</div>
                @for (item of billingItems; track item) {
                  <div class="settings-item" [class.active]="item === activeItem" (click)="activeItem = item">{{ item }}</div>
                }
              </div>
              <div class="settings-group">
                <div class="settings-group-title">PLATFORM</div>
                @for (item of platformItems; track item) {
                  <div class="settings-item" [class.active]="item === activeItem" (click)="activeItem = item">{{ item }}</div>
                }
              </div>
              <div class="settings-item danger">Delete Account</div>
            </aside>
            <div class="settings-content">
              <div class="section-card">
                <div class="plan-header">
                  <div>
                    <div class="plan-label">Current Plan</div>
                    <div class="plan-name">Pro · $45/month</div>
                    <div class="plan-meta">Renews April 1, 2025 · 200 AI credits/mo</div>
                  </div>
                  <button class="btn-primary">Upgrade to Team</button>
                </div>
                <div class="plan-upgrade-note">$120/mo · 3 seats</div>
              </div>
              <div class="section-card">
                <h3 class="section-title">Notification Preferences</h3>
                @for (notif of notifications; track notif.name) {
                  <div class="notif-row">
                    <div class="notif-info">
                      <div class="notif-name">{{ notif.name }}</div>
                      <div class="notif-sub">{{ notif.sub }}</div>
                    </div>
                    <div class="notif-channels">
                      @for (ch of notif.channels; track ch.name) {
                        <label class="channel-check">
                          <input type="checkbox" [checked]="ch.checked" />
                          {{ ch.name }}
                        </label>
                      }
                    </div>
                  </div>
                }
              </div>
            </div>
          </div>
        </div>
      </main>
    </div>
  `
})
export class SettingsComponent {
  accountItems  = ['Profile & Identity', 'Notifications', 'Privacy & Visibility'];
  securityItems = ['Password & Auth', 'Two-Factor Auth', 'Sessions & Devices'];
  billingItems  = ['Plan & Subscription', 'Payment Methods'];
  platformItems = ['Integrations', 'AI Agent Settings', 'Data Export'];
  activeItem = 'Profile & Identity';
  notifications = [
    { name: 'Marketplace orders', sub: 'New orders, messages, disputes',       channels: [{ name: 'Email', checked: true  }, { name: 'Push', checked: true  }, { name: 'SMS', checked: false }] },
    { name: 'Invoice updates',    sub: 'Payment received, overdue reminders',   channels: [{ name: 'Email', checked: true  }, { name: 'Push', checked: false }, { name: 'SMS', checked: true  }] },
    { name: 'Project activity',   sub: 'Comments, status changes, assignments', channels: [{ name: 'Email', checked: false }, { name: 'Push', checked: true  }, { name: 'SMS', checked: false }] },
    { name: 'Governance votes',   sub: 'New proposals, voting reminders',       channels: [{ name: 'Email', checked: true  }, { name: 'Push', checked: true  }, { name: 'SMS', checked: false }] },
  ];
}

// ─── APP SHELL ────────────────────────────────────────────────────────────────

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [RouterOutlet],
  template: `<router-outlet />`
})
export class AppComponent {}

// ─── ROUTES & BOOTSTRAP ───────────────────────────────────────────────────────

const routes: Routes = [
  { path: '',            redirectTo: '/dashboard', pathMatch: 'full' },
  { path: 'dashboard',   component: DashboardComponent   },
  { path: 'portfolio',   component: PortfolioComponent   },
  { path: 'work-board',  component: WorkBoardComponent   },
  { path: 'marketplace', component: MarketplaceComponent },
  { path: 'exchange',    component: ExchangeComponent    },
  { path: 'idea-studio', component: IdeaStudioComponent  },
  { path: 'governance',  component: GovernanceComponent  },
  { path: 'ai-agent',    component: AiAgentComponent     },
  { path: 'analytics',   component: AnalyticsComponent   },
  { path: 'profile',     component: ProfileComponent     },
  { path: 'settings',    component: SettingsComponent    },
  { path: '**',          redirectTo: '/dashboard'        },
];

bootstrapApplication(AppComponent, {
  providers: [
    provideRouter(routes),
    provideAnimations(),
  ]
}).catch(err => console.error(err));
