import { bootstrapApplication } from '@angular/platform-browser';
import { Component, signal, computed } from '@angular/core';
import { CommonModule } from '@angular/common';

// ─── TYPES ─────────────────────────────────────────────────────────────────────

type ScreenId =
  | 'twofa' | 'login' | 'signup' | 'onboarding'
  | 'dashboard' | 'portfolio' | 'workboard'
  | 'marketplace' | 'exchange' | 'idea-studio'
  | 'community' | 'work-okrs' | 'funding'
  | 'governance' | 'ai-agent' | 'analytics'
  | 'search' | 'settings' | 'profile';

interface NavItem {
  id: ScreenId;
  label: string;
  icon: string;
  badge?: number;
}

interface StatCard {
  value: string;
  label: string;
  change: string;
  changeType: 'up' | 'down' | 'warn' | 'info';
}

interface ProjectRow {
  name: string;
  meta: string;
  status: string;
  statusType: string;
  dotColor: string;
}

interface ProgramCard {
  type: string;
  name: string;
  desc: string;
  stats: { value: string; label: string }[];
  health: number;
  healthColor: string;
  stripeColor: string;
  status: string;
  statusType: string;
}

interface StoryCard {
  title: string;
  tags: string[];
  pts: string;
  extra?: string;
  extraType?: string;
  progress?: number;
  avatar?: string;
  avatarColor?: string;
  blocked?: boolean;
  aiNote?: string;
}

interface ListingCard {
  title: string;
  seller: string;
  sellerInitials: string;
  sellerColor: string;
  verified: boolean;
  rating: string;
  reviews: number;
  price: string;
  bgColor: string;
  icon: string;
  aiMatch?: boolean;
}

interface ProposalCard {
  status: string;
  statusType: string;
  id: string;
  category: string;
  title: string;
  desc: string;
  forVotes?: number;
  againstVotes?: number;
  abstainVotes?: number;
  forPct?: number;
  quorumNote?: string;
  countdown?: string;
  execStep?: string;
  submitLabel?: string;
  lastEdit?: string;
}

interface MemberRow {
  initials: string;
  color: string;
  name: string;
  role: string;
  stake: string;
  voted: string;
  votedType: string;
}

interface TxRow {
  icon: string;
  name: string;
  meta: string;
  amount: string;
  income: boolean;
}

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [CommonModule],
  template: `
    <!-- ═══════════════════════════════ AUTH SCREENS ═══════════════════════════════ -->

    <!-- 2FA -->
    <ng-container *ngIf="screen() === 'twofa'">
      <div class="auth-center">
        <div class="twofa-icon">🔒</div>
        <h1 class="auth-title">Two-factor verification</h1>
        <p class="auth-subtitle">Enter the 6-digit code from your authenticator app</p>

        <div class="twofa-card">
          <div class="digit-row">
            <div class="digit-box filled">4</div>
            <div class="digit-box filled">8</div>
            <div class="digit-box filled">2</div>
            <div class="digit-box active">_</div>
            <div class="digit-box empty"></div>
            <div class="digit-box empty"></div>
          </div>

          <div class="dot-progress">
            <div class="dot past"></div>
            <div class="dot past"></div>
            <div class="dot filled"></div>
            <div class="dot"></div>
            <div class="dot"></div>
            <div class="dot"></div>
          </div>

          <button class="btn-primary" style="width:100%;justify-content:center;padding:12px;">
            Verify &amp; continue
          </button>

          <div class="twofa-footer">
            <button class="link-btn">Use backup code</button>
            <span class="expire-text">Code expires in 28s</span>
          </div>
        </div>

        <button class="back-link" (click)="navigate('login')">← Back to login</button>
      </div>
    </ng-container>

    <!-- LOGIN -->
    <ng-container *ngIf="screen() === 'login'">
      <div class="auth-split">
        <div class="auth-left">
          <div class="auth-left-logo">
            <div class="logo-icon">K</div>
            <span class="logo-name">kogi</span>
          </div>
          <div class="auth-left-content">
            <h1 class="auth-left-title">Your independent<br>work, unified.</h1>
            <p class="auth-left-desc">Portfolio, marketplace, finance, governance, and AI — all in one platform built for the independent worker.</p>
            <div class="auth-tags">
              <span class="auth-tag blue-tag">Portfolio OS</span>
              <span class="auth-tag green-tag">2.5% Marketplace</span>
              <span class="auth-tag purple-tag">AI Agent</span>
              <span class="auth-tag amber-tag">Co-op Governance</span>
            </div>
          </div>
        </div>
        <div class="auth-right">
          <h2 class="auth-form-title">Welcome back</h2>
          <p class="auth-form-sub">Sign in to your Kogi account</p>

          <div class="oauth-row">
            <button class="oauth-btn"><span>G</span> Continue with Google</button>
            <button class="oauth-btn"><span>◆</span> GitHub</button>
          </div>

          <div class="divider">or continue with email</div>

          <div class="field">
            <div class="field-label">Email address</div>
            <input class="field-input" type="email" value="jordan&#64;studio.io">
          </div>

          <div class="field">
            <div class="pw-actions">
              <span class="field-label">Password</span>
              <button class="link-btn" style="font-size:13px">Forgot password?</button>
            </div>
            <div class="password-wrapper">
              <input class="field-input" type="password" value="••••••••••••">
              <span style="position:absolute;right:12px;top:50%;transform:translateY(-50%);font-size:12px;color:var(--text-2);cursor:pointer">show</span>
            </div>
          </div>

          <div class="checkbox-row">
            <input type="checkbox" checked style="accent-color:var(--blue)">
            <span>Keep me signed in for 30 days</span>
          </div>

          <button class="btn-primary" style="width:100%;justify-content:center;padding:12px;font-size:15px;" (click)="navigate('dashboard')">
            Sign in
          </button>

          <p class="auth-switch">Don't have an account? <a (click)="navigate('signup')">Create one free</a></p>
        </div>
      </div>
    </ng-container>

    <!-- SIGNUP -->
    <ng-container *ngIf="screen() === 'signup'">
      <div class="auth-split">
        <div class="auth-left">
          <div class="auth-left-logo">
            <div class="logo-icon">K</div>
            <span class="logo-name">kogi</span>
          </div>
          <div class="auth-left-content">
            <h1 class="auth-left-title">Start your free<br>account today</h1>
            <p class="auth-left-desc">Join 18,000+ independent workers who've replaced 12 tools with one unified platform.</p>
            <div class="auth-features">
              <div class="auth-feature"><div class="check-icon">✓</div> Free plan with Portfolio + Work Board</div>
              <div class="auth-feature"><div class="check-icon">✓</div> No credit card required</div>
              <div class="auth-feature"><div class="check-icon">✓</div> Upgrade any time, cancel anytime</div>
            </div>
          </div>
        </div>
        <div class="auth-right">
          <h2 class="auth-form-title">Create your account</h2>
          <p class="auth-form-sub">You're 60 seconds from having a full work OS.</p>

          <div class="oauth-row" style="gap:8px">
            <button class="oauth-btn" style="font-size:12px"><span>G</span> Google</button>
            <button class="oauth-btn" style="font-size:12px"><span>◆</span> GitHub</button>
            <button class="oauth-btn" style="font-size:12px"><span>in</span> LinkedIn</button>
          </div>

          <div class="divider">or sign up with email</div>

          <div class="field-row" style="margin-bottom:16px">
            <div>
              <div class="field-label">First name</div>
              <input class="field-input" value="Jordan">
            </div>
            <div>
              <div class="field-label">Last name</div>
              <input class="field-input" value="Davis">
            </div>
          </div>

          <div class="field">
            <div class="field-label">Email address</div>
            <input class="field-input" type="email" value="jordan&#64;studio.io" style="border-color:var(--blue);box-shadow:0 0 0 3px var(--blue-glow)">
          </div>

          <div class="field">
            <div class="field-label">Password</div>
            <div class="password-wrapper">
              <input class="field-input" type="password" value="••••••••••••" style="padding-right:120px">
              <div class="pw-strength">
                <div class="pw-bar filled"></div>
                <div class="pw-bar filled"></div>
                <div class="pw-bar filled"></div>
                <div class="pw-bar filled"></div>
              </div>
            </div>
            <div class="pw-hint">Strong password</div>
          </div>

          <div class="field-label" style="margin-bottom:8px">Start with</div>
          <div class="plan-selector">
            <div class="plan-option selected">
              <div class="plan-name" style="color:var(--blue)">Free</div>
              <div class="plan-price">$0/mo — no card</div>
            </div>
            <div class="plan-option">
              <div class="plan-name">Pro</div>
              <div class="plan-price">$45/mo — 14 day trial</div>
            </div>
          </div>

          <button class="btn-primary" style="width:100%;justify-content:center;padding:12px;font-size:15px;" (click)="navigate('onboarding')">
            Create free account →
          </button>

          <p class="auth-terms">By signing up you agree to our <a>Terms</a> and <a>Privacy Policy</a></p>
          <p class="auth-switch">Already have an account? <a (click)="navigate('login')">Sign in</a></p>
        </div>
      </div>
    </ng-container>

    <!-- ONBOARDING -->
    <ng-container *ngIf="screen() === 'onboarding'">
      <div class="onboarding-screen">
        <div class="onboarding-logo">
          <div class="logo-icon">K</div>
          <span class="logo-name">kogi</span>
        </div>

        <h1 class="onboarding-title">Welcome, Jordan! 🖐️</h1>
        <p class="onboarding-sub">Let's personalize your workspace. What describes you best?</p>

        <div class="persona-list">
          <div class="persona-card selected" *ngFor="let p of personas; let i = index"
               [class.selected]="selectedPersona() === i"
               (click)="selectedPersona.set(i)">
            <div class="persona-icon">{{ p.icon }}</div>
            <div class="persona-name">{{ p.name }}</div>
            <div class="persona-desc">{{ p.desc }}</div>
            <span class="persona-badge" *ngIf="p.badge">{{ p.badge }}</span>
          </div>
        </div>

        <div class="onboarding-progress">
          <div class="step-indicator">
            <div class="step-bubble active">1</div>
            <div class="step-line"></div>
            <div class="step-bubble">2</div>
            <div class="step-line"></div>
            <div class="step-bubble">3</div>
            <div class="step-line"></div>
            <div class="step-bubble">4</div>
          </div>
          <div style="font-size:13px;color:var(--text-2)">Step 1 of 4 — about 2 minutes</div>
          <button class="btn-primary" (click)="navigate('dashboard')">Continue →</button>
        </div>
      </div>
    </ng-container>

    <!-- ═══════════════════════════════ APP SHELL ═══════════════════════════════ -->
    <ng-container *ngIf="isAppScreen()">
      <div class="app-shell">
        <!-- Sidebar -->
        <aside class="sidebar">
          <div class="sidebar-logo">
            <div class="logo-icon">K</div>
            <span class="logo-name">kogi</span>
          </div>

          <nav class="nav">
            <button *ngFor="let item of navItems"
                    class="nav-item"
                    [class.active]="screen() === item.id"
                    (click)="navigate(item.id)">
              <span class="nav-icon">{{ item.icon }}</span>
              {{ item.label }}
              <span class="nav-badge" *ngIf="item.badge">{{ item.badge }}</span>
            </button>
          </nav>

          <button class="sidebar-user" (click)="navigate('profile')">
            <div class="user-avatar" style="background:linear-gradient(135deg,#6d28d9,#4f46e5);color:#fff">JD</div>
            <div class="user-info">
              <div class="user-name">Jordan Davis</div>
              <div class="user-plan">Pro Plan</div>
            </div>
          </button>
        </aside>

        <!-- Main -->
        <div class="main">
          <!-- ── DASHBOARD ── -->
          <ng-container *ngIf="screen() === 'dashboard'">
            <div class="topbar">
              <div class="topbar-left">
                <span class="topbar-title">Dashboard</span>
              </div>
              <div class="topbar-actions">
                <div class="search-box">
                  <span>🔍</span> Search everything...
                  <span class="search-kbd">⌘K</span>
                </div>
                <div class="topbar-icon-btn">◑</div>
                <div class="topbar-icon-btn">?</div>
                <button class="btn-primary" style="padding:7px 14px;font-size:13px">+ New</button>
              </div>
            </div>
            <div class="content">
              <div class="inner-screen">
                <div class="greeting-card">
                  <div>
                    <div class="greeting-title">Good morning, Jordan 🖐️</div>
                    <div class="greeting-sub">You have 3 blocked stories and 2 invoices overdue. Your AI Agent has suggestions.</div>
                  </div>
                  <button class="btn-primary" (click)="navigate('ai-agent')">Ask AI Agent</button>
                </div>

                <div *ngFor="let s of dashStats" class="stat-card">
                  <div class="stat-value">{{ s.value }}</div>
                  <div class="stat-label">{{ s.label }}</div>
                  <span class="stat-change" [ngClass]="s.changeType">
                    {{ s.changeType === 'up' ? '▲' : s.changeType === 'down' ? '▼' : '' }} {{ s.change }}
                  </span>
                  <div class="stat-divider"></div>
                </div>

                <div class="card">
                  <div class="section-header">
                    <div>
                      <div class="section-title">Active Projects</div>
                      <div class="section-sub">6 projects · 3 at risk</div>
                    </div>
                    <button class="link-btn">View all</button>
                  </div>
                  <div *ngFor="let p of dashProjects" class="project-row">
                    <div class="project-dot" [style.background]="p.dotColor"></div>
                    <div>
                      <div class="project-name">{{ p.name }}</div>
                      <div class="project-meta">{{ p.meta }}</div>
                    </div>
                    <span class="badge" [ngClass]="'badge-' + p.statusType">{{ p.status }}</span>
                  </div>
                </div>
              </div>
            </div>
          </ng-container>

          <!-- ── PORTFOLIO ── -->
          <ng-container *ngIf="screen() === 'portfolio'">
            <div class="topbar">
              <div class="topbar-left">
                <span class="topbar-title">Portfolio</span>
              </div>
              <div class="topbar-actions">
                <div class="search-box">🔍 Search everything... <span class="search-kbd">⌘K</span></div>
                <div class="topbar-icon-btn">◑</div>
                <div class="topbar-icon-btn">?</div>
                <button class="btn-primary" style="padding:7px 14px;font-size:13px">+ New Item</button>
              </div>
            </div>
            <div class="content">
              <div class="inner-screen">
                <div class="section-header">
                  <div>
                    <div class="section-title" style="font-size:20px">My Portfolio</div>
                    <div class="section-sub" style="margin-top:4px">6 programs · 12 projects · 3 resources</div>
                  </div>
                  <div class="portfolio-toolbar">
                    <button class="view-btn">☰ List</button>
                    <button class="view-btn active">⊞ Grid</button>
                    <button class="view-btn">▲ Map</button>
                  </div>
                </div>

                <div *ngFor="let p of programs" class="program-card">
                  <div class="program-stripe" [style.background]="p.stripeColor"></div>
                  <div class="program-body">
                    <div style="display:flex;justify-content:space-between;align-items:flex-start">
                      <div>
                        <div class="program-tag">{{ p.type }}</div>
                        <div class="program-name">{{ p.name }}</div>
                        <div class="program-desc">{{ p.desc }}</div>
                      </div>
                      <span class="badge" [ngClass]="'badge-' + p.statusType">{{ p.status }}</span>
                    </div>
                    <div class="program-stats">
                      <div *ngFor="let s of p.stats">
                        <div class="pstat-value" [style.color]="s.label === 'This month' || s.label === 'Revenue' || s.label === 'Monthly' ? '#22c55e' : 'var(--text)'">{{ s.value }}</div>
                        <div class="pstat-label">{{ s.label }}</div>
                      </div>
                    </div>
                    <div class="health-bar-track">
                      <div class="health-bar" [style.width]="p.health + '%'" [style.background]="p.healthColor"></div>
                    </div>
                    <div class="health-text">{{ p.health }}% portfolio health</div>
                  </div>
                </div>

                <div class="resource-card">
                  <div class="resource-icon-box">📁</div>
                  <div style="flex:1">
                    <div class="card-label">RESOURCES</div>
                    <div class="resource-name">Assets &amp; Capital</div>
                    <div class="resource-lines">Software licenses<br>Investment accounts<br>Equipment (laptop, etc)</div>
                  </div>
                  <div class="resource-meta">
                    <div style="font-size:13px;color:var(--text-1)">6 tools</div>
                    <div class="resource-val">$12,400</div>
                    <div style="font-size:12px;color:var(--text-2)">3 items</div>
                  </div>
                </div>
              </div>
            </div>
          </ng-container>

          <!-- ── WORK BOARD ── -->
          <ng-container *ngIf="screen() === 'workboard'">
            <div class="topbar">
              <div class="topbar-left">
                <div>
                  <span class="topbar-breadcrumb">
                    <span>Portfolio</span> / <span>Design Studio 2025</span> / <span>E-commerce Build</span>
                  </span>
                  <div class="topbar-title">E-commerce Platform Build</div>
                </div>
              </div>
              <div class="topbar-actions">
                <div class="search-box">🔍 Search everything... <span class="search-kbd">⌘K</span></div>
                <div class="topbar-icon-btn">◑</div>
                <div class="topbar-icon-btn">?</div>
                <button class="btn-primary" style="padding:7px 14px;font-size:13px">+ Story</button>
              </div>
            </div>

            <div class="board-toolbar">
              <button class="board-tab active">Board</button>
              <button class="board-tab">List</button>
              <button class="board-tab">Sprint</button>
              <button class="board-tab">Timeline</button>
              <button class="board-tab">Backlog</button>
              <div class="board-meta">
                Sprint 4 of 6 · Mar 10–24
                <span class="sprint-badge badge-red">At risk</span>
                <button class="btn-ghost" style="padding:5px 12px;font-size:12px">Filter</button>
                <button class="btn-ghost" style="padding:5px 12px;font-size:12px">Group by</button>
              </div>
            </div>

            <div style="flex:1;overflow:hidden;display:flex;flex-direction:column">
              <div class="kanban-board">
                <!-- BACKLOG -->
                <div class="kanban-col">
                  <div class="col-header">
                    <span class="col-title" style="color:var(--text-1)">BACKLOG</span>
                    <span class="col-count">8</span>
                  </div>
                  <div *ngFor="let s of backlogStories" class="story-card">
                    <div class="story-title">{{ s.title }}</div>
                    <div class="story-tags"><span class="story-tag" [ngClass]="s.tags[0]">{{ s.tags[0] }}</span></div>
                    <div class="story-pts">{{ s.pts }}</div>
                  </div>
                  <div class="more-stories">+ 5 more stories</div>
                </div>

                <!-- IN PROGRESS -->
                <div class="kanban-col">
                  <div class="col-header">
                    <span class="col-title" style="color:var(--blue)">IN PROGRESS</span>
                    <span class="col-count">4</span>
                  </div>
                  <div *ngFor="let s of inProgressStories" class="story-card" [class.blocked]="s.blocked">
                    <div class="story-title">{{ s.title }}</div>
                    <div class="story-block-note" *ngIf="s.extra && s.extraType === 'block'">⚠ {{ s.extra }}</div>
                    <div class="story-tags"><span class="story-tag" [ngClass]="s.tags[0]">{{ s.tags[0] }}</span></div>
                    <div class="story-progress" *ngIf="s.progress">
                      <div class="story-progress-bar" [style.width]="s.progress + '%'"></div>
                    </div>
                    <div class="story-footer">
                      <span class="story-pts">{{ s.pts }}</span>
                      <div class="story-avatar" *ngIf="s.avatar" [style.background]="s.avatarColor">{{ s.avatar }}</div>
                    </div>
                  </div>
                </div>

                <!-- IN REVIEW -->
                <div class="kanban-col">
                  <div class="col-header">
                    <span class="col-title" style="color:var(--purple)">IN REVIEW</span>
                    <span class="col-count">2</span>
                  </div>
                  <div *ngFor="let s of reviewStories" class="story-card">
                    <div class="story-title">{{ s.title }}</div>
                    <div class="story-tags"><span class="story-tag" [ngClass]="s.tags[0]">{{ s.tags[0] }}</span></div>
                    <div class="story-pts">{{ s.pts }}</div>
                    <div style="font-size:11.5px;color:var(--text-2);margin-top:4px">{{ s.extra }}</div>
                  </div>
                </div>

                <!-- DONE -->
                <div class="kanban-col">
                  <div class="col-header">
                    <span class="col-title" style="color:var(--green)">DONE</span>
                    <span class="col-count">6</span>
                  </div>
                  <div *ngFor="let s of doneStories" class="story-card">
                    <div class="story-title">{{ s.title }}</div>
                    <div class="story-tags"><span class="story-tag done">✓ done</span></div>
                    <div class="story-pts">{{ s.pts }}</div>
                  </div>
                  <div class="more-stories">+ 3 more</div>
                </div>

                <!-- BLOCKED -->
                <div class="kanban-col">
                  <div class="col-header">
                    <span class="col-title" style="color:var(--red)">BLOCKED</span>
                    <span class="col-count" style="background:rgba(239,68,68,0.15);color:var(--red)">2</span>
                  </div>
                  <div *ngFor="let s of blockedStories" class="story-card blocked">
                    <div class="story-title">{{ s.title }}</div>
                    <div class="story-block-note">{{ s.extra }}</div>
                    <div class="story-tags"><span class="story-tag blocker">blocker</span></div>
                    <div class="story-ai-note" *ngIf="s.aiNote">✦ {{ s.aiNote }}</div>
                  </div>
                </div>
              </div>
            </div>
          </ng-container>

          <!-- ── MARKETPLACE ── -->
          <ng-container *ngIf="screen() === 'marketplace'">
            <div class="topbar">
              <div class="topbar-left">
                <span class="topbar-title">Marketplace</span>
              </div>
              <div class="topbar-actions">
                <div class="search-box">🔍 Search everything... <span class="search-kbd">⌘K</span></div>
                <div class="topbar-icon-btn">♡</div>
                <div class="topbar-icon-btn">?</div>
                <button class="btn-primary" style="padding:7px 14px;font-size:13px">+ List a Service</button>
              </div>
            </div>
            <div class="content">
              <div class="inner-screen">
                <div class="marketplace-hero">
                  <div class="marketplace-title">Find independent talent &amp; services</div>
                  <div class="marketplace-meta">2.5% platform fee · Escrow protected · 4,200+ active listings</div>
                  <div class="search-row">
                    <div class="market-search">🔍 React developer for SaaS startup...</div>
                    <div class="category-select">All Categories ▾</div>
                    <button class="btn-primary">Search</button>
                  </div>
                  <div class="popular-tags">
                    <span style="font-size:12px;color:var(--text-2)">Popular:</span>
                    <span class="pop-tag react">React developer</span>
                    <span class="pop-tag logo">Logo design</span>
                    <span class="pop-tag seo">SEO writing</span>
                    <span class="pop-tag">Data analysis</span>
                    <span class="pop-tag video">Video editing</span>
                  </div>
                </div>

                <div class="market-layout">
                  <div>
                    <div class="filter-panel">
                      <div class="filter-title">Category</div>
                      <div class="filter-item active"><span>All Services</span><span class="filter-count">4,200</span></div>
                      <div class="filter-item"><span>Development</span><span class="filter-count">1,820</span></div>
                      <div class="filter-item"><span>Design</span><span class="filter-count">960</span></div>
                      <div class="filter-item"><span>Writing &amp; Content</span><span class="filter-count">640</span></div>
                      <div class="filter-item"><span>Marketing</span><span class="filter-count">380</span></div>
                      <div class="filter-item"><span>Data &amp; Analytics</span><span class="filter-count">240</span></div>
                    </div>
                    <div class="filter-panel" style="margin-top:12px">
                      <div class="filter-title">Budget</div>
                      <div class="budget-row">
                        <input class="budget-input" value="$50">
                        <span style="color:var(--text-2);align-self:center">—</span>
                        <input class="budget-input" value="$500">
                      </div>
                      <div class="budget-track"><div class="budget-fill"></div></div>
                    </div>
                    <div class="filter-panel" style="margin-top:12px">
                      <div class="filter-title">Filters</div>
                      <div class="filter-check"><input type="checkbox" checked style="accent-color:var(--blue)"> Top Rated (4.8+)</div>
                      <div class="filter-check"><input type="checkbox"> Kogi Verified</div>
                      <div class="filter-check"><input type="checkbox"> Fast delivery (&lt; 48h)</div>
                      <div class="filter-check"><input type="checkbox" checked style="accent-color:var(--blue)"> Portfolio shown</div>
                    </div>
                  </div>

                  <div>
                    <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:12px">
                      <span style="font-size:13px;color:var(--text-1)">Showing 1–12 of 1,820 results</span>
                      <div class="category-select" style="font-size:12px">Best Match ▾</div>
                    </div>
                    <div *ngFor="let l of listings" class="listing-card">
                      <div class="listing-image" [style.background]="l.bgColor">
                        <span style="font-size:32px">{{ l.icon }}</span>
                        <span class="ai-match-badge" *ngIf="l.aiMatch" style="position:absolute;top:8px;right:8px">AI Match</span>
                      </div>
                      <div class="listing-body">
                        <div class="listing-title">{{ l.title }}</div>
                        <div class="seller-row">
                          <div class="seller-avatar" [style.background]="l.sellerColor">{{ l.sellerInitials }}</div>
                          <span class="seller-name">{{ l.seller }}</span>
                          <span class="verified-badge" *ngIf="l.verified">Verified</span>
                        </div>
                        <div class="rating-row">
                          <span class="stars">★ {{ l.rating }} ({{ l.reviews }})</span>
                          <span class="price">{{ l.price }}</span>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </ng-container>

          <!-- ── EXCHANGE ── -->
          <ng-container *ngIf="screen() === 'exchange'">
            <div class="topbar">
              <div class="topbar-left">
                <span class="topbar-title">Exchange</span>
              </div>
              <div class="topbar-actions">
                <div class="search-box">🔍 Search everything... <span class="search-kbd">⌘K</span></div>
                <div class="topbar-icon-btn">◑</div>
                <div class="topbar-icon-btn">?</div>
                <button class="btn-primary" style="padding:7px 14px;font-size:13px">+ Add Funds</button>
              </div>
            </div>
            <div class="content">
              <div class="inner-screen">
                <div class="balance-card">
                  <div class="balance-label">Available Balance</div>
                  <div class="balance-amount">$4,820.00</div>
                  <div class="balance-meta">KYC Level 2 · Wire transfers enabled</div>
                  <div class="action-buttons">
                    <button class="action-btn primary-action">↑ Withdraw</button>
                    <button class="action-btn">↓ Deposit</button>
                    <button class="action-btn">→ Send</button>
                    <button class="action-btn">⊟ Invoice</button>
                  </div>
                </div>

                <div *ngFor="let s of exchangeStats" class="stat-card">
                  <div class="stat-value">{{ s.value }}</div>
                  <div class="stat-label">{{ s.label }}</div>
                  <span class="stat-change" [ngClass]="s.changeType">
                    {{ s.changeType === 'up' ? '▲' : '' }} {{ s.change }}
                  </span>
                </div>

                <div class="card">
                  <div class="section-header" style="margin-bottom:12px">
                    <div class="section-title">Transaction History</div>
                    <div style="display:flex;gap:12px">
                      <button class="link-btn" style="color:var(--blue)">All</button>
                      <button class="link-btn">Income</button>
                      <button class="link-btn">Expenses</button>
                      <button class="link-btn">Escrow</button>
                    </div>
                  </div>
                  <div *ngFor="let t of transactions" class="tx-row">
                    <div class="tx-icon">{{ t.icon }}</div>
                    <div style="flex:1">
                      <div class="tx-name">{{ t.name }}</div>
                      <div class="tx-meta">{{ t.meta }}</div>
                    </div>
                    <span class="tx-amount" [class.income]="t.income" [class.expense]="!t.income">{{ t.amount }}</span>
                  </div>
                </div>
              </div>
            </div>
          </ng-container>

          <!-- ── IDEA STUDIO ── -->
          <ng-container *ngIf="screen() === 'idea-studio'">
            <div class="topbar">
              <div class="topbar-left">
                <span class="topbar-title">Idea Studio</span>
              </div>
              <div class="topbar-actions">
                <div class="search-box">🔍 Search everything... <span class="search-kbd">⌘K</span></div>
                <div class="topbar-icon-btn">♡</div>
                <div class="topbar-icon-btn">?</div>
                <button class="btn-primary" style="padding:7px 14px;font-size:13px">+ Capture Idea</button>
              </div>
            </div>
            <div class="content">
              <div class="studio-layout">
                <div class="studio-header">
                  <div>
                    <div class="studio-title">Idea Studio</div>
                    <div class="studio-meta">5-stage innovation pipeline · 14 active ideas</div>
                  </div>
                  <div style="display:flex;gap:16px">
                    <button class="link-btn" style="color:var(--blue);font-weight:600">Pipeline</button>
                    <button class="link-btn">Gallery</button>
                    <button class="link-btn">Archive</button>
                  </div>
                </div>

                <div *ngFor="let stage of ideaStages" class="pipeline-stage"
                     [class.active]="stage.active">
                  <div class="stage-icon">{{ stage.icon }}</div>
                  <div class="stage-name" [ngClass]="stage.colorClass">{{ stage.label }}</div>
                  <div class="stage-count" [ngClass]="stage.colorClass">{{ stage.count }}</div>
                  <div class="stage-ideas">ideas</div>
                </div>

                <div class="idea-preview">
                  <div class="idea-tags">
                    <span class="badge badge-blue">Stage 3 — Design</span>
                    <span class="badge badge-teal">SaaS</span>
                    <span class="badge" style="background:rgba(34,197,94,0.1);color:var(--green)">IP Timestamped ⚓</span>
                  </div>
                  <div class="idea-title-text">AI-powered resume scoring for freelancers</div>
                  <div class="validation-score">
                    <div class="score-label">Validation score</div>
                    <div class="score-value">84/100</div>
                  </div>
                </div>
              </div>
            </div>
          </ng-container>

          <!-- ── GOVERNANCE ── -->
          <ng-container *ngIf="screen() === 'governance'">
            <div class="topbar">
              <div class="topbar-left">
                <span class="topbar-title">Governance — TechWorkers Cooperative</span>
              </div>
              <div class="topbar-actions">
                <div class="search-box">🔍 Search everything... <span class="search-kbd">⌘K</span></div>
                <div class="topbar-icon-btn">♡</div>
                <div class="topbar-icon-btn">?</div>
                <button class="btn-primary" style="padding:7px 14px;font-size:13px">+ New Proposal</button>
              </div>
            </div>
            <div class="content">
              <div class="inner-screen">
                <div class="coop-header">
                  <div class="coop-icon-box">🏛️</div>
                  <div>
                    <div class="coop-name">TechWorkers Cooperative</div>
                    <div class="coop-meta">12 members · 1M1V voting · Est. 2023 · Charter v2.1</div>
                  </div>
                  <div class="coop-stats">
                    <div class="coop-stat">
                      <div class="coop-stat-value" style="color:var(--purple)">12</div>
                      <div class="coop-stat-label">Members</div>
                    </div>
                    <div class="coop-stat">
                      <div class="coop-stat-value" style="color:var(--blue)">3</div>
                      <div class="coop-stat-label">Active proposals</div>
                    </div>
                    <div class="coop-stat">
                      <div class="coop-stat-value" style="color:var(--green)">$84K</div>
                      <div class="coop-stat-label">Q4 Revenue</div>
                    </div>
                    <div class="coop-stat">
                      <div class="coop-stat-value">7/12</div>
                      <div class="coop-stat-label">Quorum met</div>
                    </div>
                  </div>
                </div>

                <div class="gov-tabs">
                  <button class="gov-tab active">Proposals</button>
                  <button class="gov-tab">Members</button>
                  <button class="gov-tab">Charter</button>
                  <button class="gov-tab">Treasury</button>
                  <button class="gov-tab">Audit Log</button>
                </div>

                <!-- Open proposal -->
                <div class="proposal-card open">
                  <div class="proposal-status-row">
                    <span class="badge badge-blue">Open</span>
                    <span class="badge badge-blue" style="background:rgba(59,130,246,0.08)">Voting</span>
                    <span class="proposal-id">PROP-2025-007 · Capital Allocation</span>
                    <div class="proposal-closes">
                      Closes in
                      <div class="proposal-timer">2 days 14h</div>
                    </div>
                  </div>
                  <div class="proposal-title">Allocate $8,000 from Q4 surplus for developer tooling</div>
                  <div class="proposal-desc">Purchase GitHub Copilot licenses, upgrade CI/CD infrastructure, and acquire 2 new dev laptops for incoming members. Full breakdown in attached budget document.</div>
                  <div class="vote-row">
                    <span class="vote-for">✓ For: 7 votes</span>
                    <span class="vote-abstain">Abstain: 2</span>
                    <span class="vote-against">✕ Against: 1</span>
                  </div>
                  <div class="vote-bar">
                    <div class="vote-bar-for" style="width:70%"></div>
                    <div class="vote-bar-against" style="width:10%"></div>
                  </div>
                  <div class="vote-quorum">
                    <span>7/10 required for passage (quorum met)</span>
                    <span>2 members haven't voted</span>
                  </div>
                  <div class="vote-actions">
                    <button class="vote-btn for">✓ Vote For</button>
                    <button class="vote-btn against">✕ Vote Against</button>
                    <button class="vote-btn abstain">Abstain</button>
                    <button class="view-proposal">View full proposal →</button>
                  </div>
                </div>

                <!-- Passed proposal -->
                <div class="proposal-card passed">
                  <div class="proposal-status-row">
                    <span class="badge badge-green">Passed</span>
                    <span class="badge badge-teal" style="background:rgba(20,184,166,0.1)">Executing</span>
                    <span class="proposal-id">PROP-2025-006 · Member Admission</span>
                  </div>
                  <div class="proposal-title">Admit Priya Nair as full member (developer)</div>
                  <div class="proposal-desc">Passed 10–0 on March 1, 2025. Onboarding documents sent. Equity stake allocated. Smart execution in progress.</div>
                  <div class="health-bar-track" style="margin-top:8px">
                    <div class="health-bar" style="width:75%;background:var(--teal)"></div>
                  </div>
                  <div class="exec-step">Step 3/4 — Wallet created, awaiting charter sign</div>
                </div>

                <!-- Draft proposal -->
                <div class="proposal-card draft">
                  <div class="proposal-status-row">
                    <span class="badge" style="background:var(--bg-3);color:var(--text-2)">Draft</span>
                    <span class="proposal-id">PROP-2025-008 · Profit Distribution</span>
                    <button class="btn-primary" style="margin-left:auto;padding:5px 12px;font-size:12px">Submit for vote</button>
                  </div>
                  <div class="proposal-title">Q4 2024 profit distribution — $62,400 to members</div>
                  <div class="proposal-desc">Last edited by you · 2 hours ago · Waiting on final accounting sign-off</div>
                </div>

                <!-- Members -->
                <div class="card">
                  <div class="section-title" style="margin-bottom:12px">Members (12)</div>
                  <div *ngFor="let m of govMembers" class="member-row">
                    <div class="member-avatar" [style.background]="m.color">{{ m.initials }}</div>
                    <div>
                      <div class="member-name">{{ m.name }}</div>
                      <div class="member-role">{{ m.role }} · {{ m.stake }}</div>
                    </div>
                    <span class="member-voted" [ngClass]="m.votedType === 'yes' ? 'voted-yes' : 'voted-pending'">{{ m.voted }}</span>
                  </div>
                </div>
              </div>
            </div>
          </ng-container>

          <!-- ── AI AGENT ── -->
          <ng-container *ngIf="screen() === 'ai-agent'">
            <div class="topbar">
              <div class="topbar-left">
                <span class="topbar-title">AI Agent</span>
              </div>
              <div class="topbar-actions">
                <div class="search-box">🔍 Search everything... <span class="search-kbd">⌘K</span></div>
                <div class="topbar-icon-btn">◑</div>
                <div class="topbar-icon-btn">?</div>
                <button class="btn-ghost">🕐 History</button>
              </div>
            </div>
            <div class="ai-layout" style="flex:1;overflow:hidden">
              <!-- AI Sidebar -->
              <div class="ai-sidebar">
                <div class="ai-sidebar-section">
                  <div class="ai-sidebar-label">Portfolio Context</div>
                  <div class="ai-sidebar-label" style="margin-top:8px;font-size:11px;color:var(--text-2)">ACTIVE PROJECTS</div>
                  <div class="context-card at-risk">
                    <div class="context-title">E-commerce Build</div>
                    <div class="context-meta">Sprint 4 · 2 blocked</div>
                    <span class="badge badge-red" style="font-size:10px">At risk</span>
                  </div>
                  <div class="context-card on-track">
                    <div class="context-title">Brand Identity</div>
                    <div class="context-meta">4 in progress</div>
                    <span class="badge badge-green" style="font-size:10px">On track</span>
                  </div>
                </div>

                <div class="ai-sidebar-section">
                  <div class="ai-sidebar-label">WALLET</div>
                  <div class="wallet-mini">
                    <div class="wallet-mini-amount">$4,820</div>
                    <div class="wallet-mini-meta">Available · 2 invoices overdue</div>
                  </div>
                </div>

                <div class="ai-sidebar-section">
                  <div class="ai-sidebar-label">OKR STATUS</div>
                  <div class="okr-row"><span>Revenue goal</span><span class="okr-val">62%</span></div>
                  <div class="health-bar-track"><div class="health-bar" style="width:62%;background:var(--blue)"></div></div>
                  <div class="okr-row" style="margin-top:8px"><span>New services</span><span class="okr-val">67%</span></div>
                  <div class="health-bar-track"><div class="health-bar" style="width:67%;background:var(--green)"></div></div>
                </div>

                <div class="ai-sidebar-section">
                  <div class="ai-sidebar-label">SUGGESTED ACTIONS</div>
                  <div class="suggestion-card blocked-s">⚡ Unblock 2 blocked stories</div>
                  <div class="suggestion-card overdue-s">⊟ Chase 2 overdue invoices</div>
                  <div class="suggestion-card okr-s">📊 OKR: revenue at risk</div>
                </div>
              </div>

              <!-- Chat -->
              <div class="ai-chat">
                <div class="chat-messages">
                  <!-- AI message 1 -->
                  <div class="message">
                    <div class="msg-avatar ai-avatar">✦</div>
                    <div>
                      <div class="msg-bubble">
                        <p style="margin-bottom:10px">Good morning, Jordan. I've reviewed your portfolio and have 3 actionable insights:</p>
                        <div class="insight-item blocked">
                          <span>⚠</span>
                          <div><span class="insight-tag blocked">Blocked:</span> "Set up Stripe payment gateway" has been blocked for 3 days. I found 2 Stripe-certified developers on the Marketplace. Want me to send inquiry requests?</div>
                        </div>
                        <div class="insight-item overdue">
                          <span>⊟</span>
                          <div><span class="insight-tag overdue">Overdue:</span> Invoice INV-2024-108 for $2,000 is 14 days overdue from DesignCo. I can draft a polite payment reminder email.</div>
                        </div>
                        <div class="insight-item okr">
                          <span>📊</span>
                          <div><span class="insight-tag okr">OKR:</span> Your Q3 revenue goal is at 62% with 28 days left. At current velocity you'll reach $12.4K vs $15K target. I suggest increasing your marketplace listing visibility.</div>
                        </div>
                        <div class="msg-actions">
                          <button class="msg-action-btn primary-action">Send marketplace inquiries</button>
                          <button class="msg-action-btn">Draft invoice reminder</button>
                          <button class="msg-action-btn">Show OKR analysis</button>
                        </div>
                      </div>
                      <div class="msg-meta">Kogi AI · 9:02 AM · 3 tools used</div>
                    </div>
                  </div>

                  <!-- User message -->
                  <div class="message user">
                    <div class="msg-avatar user-msg-avatar">JD</div>
                    <div>
                      <div class="msg-bubble">Yes, send inquiries to both developers. Also, create a sprint planning summary for the E-commerce project — I need to share it with the client this afternoon.</div>
                      <div class="msg-meta" style="text-align:right">Jordan · 9:05 AM</div>
                    </div>
                  </div>

                  <!-- AI message 2 -->
                  <div class="message">
                    <div class="msg-avatar ai-avatar">✦</div>
                    <div>
                      <div class="msg-tools">Using: search_marketplace, read_wbs, create_document</div>
                      <div class="msg-bubble">
                        <p style="margin-bottom:8px">✓ <strong>Inquiries sent</strong> to Alex Kim and Marcus Chen on Marketplace (Order requests created)</p>
                        <p style="color:var(--text-1)">⊟ Sprint Summary generating...</p>
                        <div class="ai-generating" style="margin-top:8px">
                          <div class="gen-dot"></div>
                          <div class="gen-dot"></div>
                          <div class="gen-dot"></div>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>

                <div class="chat-input-area">
                  <div class="chat-input-row">
                    <input class="chat-input" placeholder="Ask anything about your portfolio, projects, finances...">
                    <button class="at-btn">&#64;</button>
                    <button class="send-btn">▶</button>
                  </div>
                  <div class="quick-actions">
                    <span class="quick-action-label">Quick actions:</span>
                    <button class="quick-action-chip">Plan sprint</button>
                    <button class="quick-action-chip">Check OKRs</button>
                    <button class="quick-action-chip">Create invoice</button>
                    <button class="quick-action-chip">Search market</button>
                  </div>
                </div>
              </div>
            </div>
          </ng-container>

          <!-- ── ANALYTICS ── -->
          <ng-container *ngIf="screen() === 'analytics'">
            <div class="topbar">
              <div class="topbar-left">
                <span class="topbar-title">Analytics</span>
              </div>
              <div class="topbar-actions">
                <div class="search-box">🔍 Search everything... <span class="search-kbd">⌘K</span></div>
                <div class="topbar-icon-btn">◑</div>
                <div class="topbar-icon-btn">?</div>
                <div class="category-select" style="font-size:13px;background:var(--bg-2);border:1px solid var(--border-1);border-radius:7px;padding:7px 12px;color:var(--text-1)">Last 30 days ▾</div>
              </div>
            </div>
            <div class="content">
              <div class="inner-screen">
                <div class="section-header">
                  <div>
                    <div class="section-title" style="font-size:20px">Analytics Overview</div>
                    <div class="section-sub">Your platform performance · Feb 7 – Mar 7, 2025</div>
                  </div>
                  <button class="btn-ghost">⬇ Export CSV</button>
                </div>

                <div *ngFor="let s of analyticsStats" class="stat-card">
                  <div class="stat-value">{{ s.value }}</div>
                  <div class="stat-label">{{ s.label }}</div>
                  <span class="stat-change" [ngClass]="s.changeType">▲ {{ s.change }}</span>
                </div>

                <div class="card">
                  <div class="section-header" style="margin-bottom:16px">
                    <div class="section-title">Revenue Over Time</div>
                    <div class="chart-tabs">
                      <button class="chart-tab active">Daily</button>
                      <button class="chart-tab">Weekly</button>
                    </div>
                  </div>
                  <div class="chart-placeholder">
                    <div class="chart-bar" *ngFor="let b of chartBars" [style.height]="b + 'px'"></div>
                  </div>
                </div>
              </div>
            </div>
          </ng-container>

          <!-- ── PROFILE ── -->
          <ng-container *ngIf="screen() === 'profile'">
            <div class="topbar">
              <div class="topbar-left">
                <span class="topbar-title">My Profile</span>
              </div>
              <div class="topbar-actions">
                <div class="search-box">🔍 Search everything... <span class="search-kbd">⌘K</span></div>
                <div class="topbar-icon-btn">◑</div>
                <div class="topbar-icon-btn">?</div>
                <button class="btn-ghost">Preview public page</button>
                <button class="btn-primary" style="padding:7px 14px;font-size:13px">Save Changes</button>
              </div>
            </div>

            <div class="profile-tabs">
              <button class="profile-tab active">Profile &amp; Identity</button>
              <button class="profile-tab">Portfolio Showcase</button>
              <button class="profile-tab">Skills &amp; Certifications</button>
              <button class="profile-tab">Reviews</button>
              <button class="profile-tab">Connections</button>
            </div>

            <div class="content">
              <div class="inner-screen">
                <div class="profile-hero">
                  <div class="profile-avatar-lg">
                    JD
                    <div class="avatar-edit-btn">✏</div>
                  </div>
                  <div class="profile-name-lg">Jordan Davis</div>
                  <div class="profile-title-text">Brand Designer &amp; Product Strategist</div>
                  <div class="profile-avail-badge">Available for work</div>
                  <div class="profile-rating">★ 4.9 | 42 reviews | 3yr on Kogi</div>
                </div>

                <div class="availability-section">
                  <div class="section-title" style="margin-bottom:12px">Availability</div>
                  <div class="avail-row">
                    <span class="avail-label">Status</span>
                    <span class="avail-value green">Open to work</span>
                    <span class="avail-label">Capacity</span>
                    <span class="avail-value">20 hrs/week</span>
                    <span class="avail-label">Rate</span>
                    <span class="avail-value">$120–180/hr</span>
                    <span class="avail-label">Timezone</span>
                    <span class="avail-value">UTC-5 (EST)</span>
                  </div>
                </div>

                <div class="skills-section">
                  <div class="section-title" style="margin-bottom:12px">Top Skills</div>
                  <div class="skill-tags">
                    <span class="skill-tag blue">Brand Identity</span>
                    <span class="skill-tag purple">UI/UX Design</span>
                    <span class="skill-tag teal">Figma</span>
                    <span class="skill-tag green">Product Strategy</span>
                    <span class="skill-tag orange">Design Systems</span>
                    <span class="skill-tag">Webflow</span>
                  </div>
                </div>

                <div class="profile-form-section">
                  <div class="profile-form-title">Basic Information</div>
                  <div class="form-grid">
                    <div>
                      <div class="field-label">First name</div>
                      <input class="field-input" value="Jordan">
                    </div>
                    <div>
                      <div class="field-label">Last name</div>
                      <input class="field-input" value="Davis">
                    </div>
                  </div>
                  <div class="field">
                    <div class="field-label">Professional title</div>
                    <input class="field-input" value="Brand Designer &amp; Product Strategist" style="border-color:var(--blue)">
                  </div>
                  <div class="field">
                    <div class="field-label">Bio (visible on public profile)</div>
                    <textarea class="textarea-field">I help startups and independent businesses build brand identities that convert. 8 years designing for 120+ clients across SaaS, fintech, and e-commerce. I believe great design is invisible — and so is great strategy.</textarea>
                  </div>
                </div>
              </div>
            </div>
          </ng-container>

          <!-- ── SETTINGS ── -->
          <ng-container *ngIf="screen() === 'settings'">
            <div class="topbar">
              <div class="topbar-left">
                <span class="topbar-title">Settings</span>
              </div>
              <div class="topbar-actions">
                <div class="search-box">🔍 Search everything... <span class="search-kbd">⌘K</span></div>
                <div class="topbar-icon-btn">◑</div>
                <div class="topbar-icon-btn">?</div>
              </div>
            </div>
            <div class="content">
              <div class="settings-layout">
                <div class="settings-section-label">ACCOUNT</div>
                <div class="settings-group">
                  <button class="settings-item active">Profile &amp; Identity</button>
                  <button class="settings-item">Notifications</button>
                  <button class="settings-item">Privacy &amp; Visibility</button>
                </div>

                <div class="settings-section-label">SECURITY</div>
                <div class="settings-group">
                  <button class="settings-item">Password &amp; Auth</button>
                  <button class="settings-item">Two-Factor Auth</button>
                  <button class="settings-item">Sessions &amp; Devices</button>
                </div>

                <div class="settings-section-label">BILLING</div>
                <div class="settings-group">
                  <button class="settings-item">Plan &amp; Subscription</button>
                  <button class="settings-item">Payment Methods</button>
                </div>

                <div class="settings-section-label">PLATFORM</div>
                <div class="settings-group">
                  <button class="settings-item">Integrations</button>
                  <button class="settings-item">AI Agent Settings</button>
                  <button class="settings-item">Data Export</button>
                  <button class="settings-item danger">Delete Account</button>
                </div>

                <div class="current-plan-card">
                  <div class="plan-row">
                    <div>
                      <div class="plan-label">Current Plan</div>
                      <div class="plan-name-big">Pro · $45/month</div>
                      <div class="plan-sub" style="color:var(--blue)">Renews April 1, 2025 · 200 AI credits/mo</div>
                    </div>
                    <button class="btn-primary">Upgrade to Team</button>
                  </div>
                  <div style="margin-top:6px;font-size:12px;color:var(--text-2)">$120/mo · 3 seats</div>
                </div>

                <div class="notif-section">
                  <div class="notif-title">Notification Preferences</div>
                  <div *ngFor="let n of notifPrefs" class="notif-row">
                    <div>
                      <div class="notif-label">{{ n.label }}</div>
                      <div class="notif-desc">{{ n.desc }}</div>
                    </div>
                    <div class="notif-toggles">
                      <button class="toggle-chip" [class.on]="n.email">Email</button>
                      <button class="toggle-chip" [class.on]="n.push">Push</button>
                      <button class="toggle-chip" [class.on]="n.sms">SMS</button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </ng-container>

          <!-- ── COMMUNITY ── -->
          <ng-container *ngIf="screen() === 'community'">
            <div class="topbar">
              <div class="topbar-left"><span class="topbar-title">Community</span></div>
              <div class="topbar-actions">
                <div class="search-box">🔍 Search everything... <span class="search-kbd">⌘K</span></div>
                <div class="topbar-icon-btn">◑</div>
                <div class="topbar-icon-btn">?</div>
                <button class="btn-primary" style="padding:7px 14px;font-size:13px">+ New Post</button>
              </div>
            </div>
            <div class="content">
              <div class="inner-screen">
                <div class="card">
                  <div class="section-title" style="margin-bottom:16px">🌐 Discussions</div>
                  <div *ngFor="let post of communityPosts" class="project-row">
                    <div class="user-avatar" [style.background]="post.color" style="width:36px;height:36px;border-radius:50%;font-size:12px;font-weight:700;color:#fff;display:flex;align-items:center;justify-content:center;flex-shrink:0">{{ post.initials }}</div>
                    <div style="flex:1">
                      <div class="project-name">{{ post.title }}</div>
                      <div class="project-meta">{{ post.meta }}</div>
                    </div>
                    <div style="text-align:right">
                      <div style="font-size:12px;color:var(--text-2)">{{ post.time }}</div>
                      <div style="font-size:12px;color:var(--text-1);margin-top:2px">💬 {{ post.replies }} · ❤ {{ post.likes }}</div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </ng-container>

          <!-- ── WORK & OKRs ── -->
          <ng-container *ngIf="screen() === 'work-okrs'">
            <div class="topbar">
              <div class="topbar-left"><span class="topbar-title">Work &amp; OKRs</span></div>
              <div class="topbar-actions">
                <div class="search-box">🔍 Search everything... <span class="search-kbd">⌘K</span></div>
                <div class="topbar-icon-btn">◑</div>
                <div class="topbar-icon-btn">?</div>
                <button class="btn-primary" style="padding:7px 14px;font-size:13px">+ New OKR</button>
              </div>
            </div>
            <div class="content">
              <div class="inner-screen">
                <div class="greeting-card">
                  <div>
                    <div class="greeting-title">Q3 2025 Objectives</div>
                    <div class="greeting-sub">3 objectives · 9 key results · 28 days remaining</div>
                  </div>
                  <span class="badge badge-amber" style="font-size:13px;padding:6px 14px">62% complete</span>
                </div>
                <div *ngFor="let okr of okrItems" class="card">
                  <div class="section-header">
                    <div class="section-title">{{ okr.objective }}</div>
                    <span class="badge" [ngClass]="'badge-' + okr.statusType">{{ okr.pct }}%</span>
                  </div>
                  <div *ngFor="let kr of okr.keyResults" style="margin-top:10px">
                    <div style="display:flex;justify-content:space-between;font-size:13px;margin-bottom:4px">
                      <span style="color:var(--text-1)">{{ kr.label }}</span>
                      <span style="color:var(--text)">{{ kr.current }} / {{ kr.target }}</span>
                    </div>
                    <div class="health-bar-track">
                      <div class="health-bar" [style.width]="kr.pct + '%'" [style.background]="kr.color"></div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </ng-container>

          <!-- ── FUNDING ── -->
          <ng-container *ngIf="screen() === 'funding'">
            <div class="topbar">
              <div class="topbar-left"><span class="topbar-title">Funding</span></div>
              <div class="topbar-actions">
                <div class="search-box">🔍 Search everything... <span class="search-kbd">⌘K</span></div>
                <div class="topbar-icon-btn">◑</div>
                <div class="topbar-icon-btn">?</div>
                <button class="btn-primary" style="padding:7px 14px;font-size:13px">+ New Round</button>
              </div>
            </div>
            <div class="content">
              <div class="inner-screen">
                <div *ngFor="let f of fundingItems" class="card">
                  <div class="section-header">
                    <div>
                      <div class="section-title">{{ f.name }}</div>
                      <div class="section-sub">{{ f.type }} · {{ f.stage }}</div>
                    </div>
                    <span class="badge" [ngClass]="'badge-' + f.statusType">{{ f.status }}</span>
                  </div>
                  <div style="display:flex;gap:24px;margin:12px 0">
                    <div><div class="stat-value" style="font-size:24px">{{ f.raised }}</div><div style="font-size:12px;color:var(--text-2)">Raised</div></div>
                    <div><div class="stat-value" style="font-size:24px">{{ f.goal }}</div><div style="font-size:12px;color:var(--text-2)">Goal</div></div>
                    <div><div class="stat-value" style="font-size:24px">{{ f.investors }}</div><div style="font-size:12px;color:var(--text-2)">Investors</div></div>
                  </div>
                  <div class="health-bar-track">
                    <div class="health-bar" [style.width]="f.pct + '%'" [style.background]="f.color"></div>
                  </div>
                  <div class="health-text" style="margin-top:6px">{{ f.pct }}% of goal reached</div>
                </div>
              </div>
            </div>
          </ng-container>

          <!-- ── SEARCH ── -->
          <ng-container *ngIf="screen() === 'search'">
            <div class="topbar">
              <div class="topbar-left"><span class="topbar-title">Search</span></div>
              <div class="topbar-actions">
                <div class="topbar-icon-btn">◑</div>
                <div class="topbar-icon-btn">?</div>
              </div>
            </div>
            <div class="content">
              <div class="inner-screen">
                <div style="max-width:600px;margin:0 auto;padding-top:40px">
                  <input class="field-input" placeholder="Search everything — projects, people, documents..." style="font-size:16px;padding:14px 18px;border-radius:12px">
                  <div style="margin-top:24px;font-size:13px;color:var(--text-2)">Recent searches</div>
                  <div *ngFor="let s of recentSearches" style="padding:10px 0;border-bottom:1px solid var(--border);display:flex;align-items:center;gap:10px;cursor:pointer;color:var(--text-1)">
                    🕐 {{ s }}
                  </div>
                </div>
              </div>
            </div>
          </ng-container>

        </div><!-- /main -->
      </div><!-- /app-shell -->
    </ng-container>
  `
})
export class AppComponent {
  screen = signal<ScreenId>('login');

  selectedPersona = signal(0);

  isAppScreen = computed(() => {
    const appScreens: ScreenId[] = [
      'dashboard','portfolio','workboard','marketplace','exchange',
      'idea-studio','community','work-okrs','funding','governance',
      'ai-agent','analytics','search','settings','profile'
    ];
    return appScreens.includes(this.screen());
  });

  navigate(id: ScreenId) { this.screen.set(id); }

  // ─── NAV ───────────────────────────────────────────────────────────────
  navItems: NavItem[] = [
    { id: 'dashboard',  label: 'Dashboard',  icon: '◇' },
    { id: 'portfolio',  label: 'Portfolio',  icon: '⊞' },
    { id: 'workboard',  label: 'Work Board', icon: '⊟', badge: 3 },
    { id: 'marketplace',label: 'Marketplace',icon: '◆' },
    { id: 'exchange',   label: 'Exchange',   icon: '◉', badge: 2 },
    { id: 'idea-studio',label: 'Idea Studio',icon: '◎' },
    { id: 'community',  label: 'Community',  icon: '◌', badge: 7 },
    { id: 'work-okrs',  label: 'Work & OKRs',icon: '◈' },
    { id: 'funding',    label: 'Funding',    icon: '◆' },
    { id: 'governance', label: 'Governance', icon: '⊡', badge: 1 },
    { id: 'ai-agent',   label: 'AI Agent',   icon: '✦' },
    { id: 'analytics',  label: 'Analytics',  icon: '◆' },
    { id: 'search',     label: 'Search',     icon: '○' },
    { id: 'settings',   label: 'Settings',   icon: '○' },
  ];

  // ─── ONBOARDING ────────────────────────────────────────────────────────
  personas = [
    { icon: '🔱', name: 'Freelancer', desc: 'Designer, developer, writer, consultant', badge: 'Most popular' },
    { icon: '👥', name: 'Micro-Agency', desc: 'Small team of 2–15 collaborators', badge: '' },
    { icon: '🏢', name: 'Cooperative', desc: 'Worker co-op or collective with shared ownership', badge: '' },
    { icon: '💡', name: 'Explorer', desc: 'Just checking out the platform', badge: '' },
  ];

  // ─── DASHBOARD ─────────────────────────────────────────────────────────
  dashStats: StatCard[] = [
    { value: '6',       label: 'Active Projects', change: '2 this month',   changeType: 'up'   },
    { value: '73%',     label: 'Portfolio Health', change: '4pts',           changeType: 'down' },
    { value: '$4,820',  label: 'Wallet Balance',   change: '$1,200 received',changeType: 'up'   },
    { value: '2',       label: 'Pending Orders',   change: '1 needs response',changeType: 'info' },
    { value: '62%',     label: 'OKR Progress',     change: 'Q3 · 28 days left',changeType: 'warn'},
  ];

  dashProjects: ProjectRow[] = [
    { name: 'Brand Identity Redesign', meta: 'Design Studio 2025 · Sprint 2', status: 'On track', statusType: 'green', dotColor: '#22c55e' },
    { name: 'E-commerce Platform Build', meta: 'Design Studio 2025 · Sprint 4', status: 'At risk', statusType: 'red', dotColor: '#ef4444' },
    { name: 'KogiTask MVP', meta: 'SaaS Product Dev · Sprint 1', status: 'At risk', statusType: 'amber', dotColor: '#f59e0b' },
    { name: 'Newsletter Q2', meta: 'Content & Education', status: 'On track', statusType: 'green', dotColor: '#22c55e' },
    { name: 'YouTube Series', meta: 'Content & Education · Pilot', status: 'Paused', statusType: 'purple', dotColor: '#a78bfa' },
  ];

  // ─── PORTFOLIO ─────────────────────────────────────────────────────────
  programs: ProgramCard[] = [
    {
      type: 'PROGRAM', name: 'Design Studio 2025',
      desc: 'Full-service brand design and digital product studio. 3 active client projects.',
      stats: [{ value: '4', label: 'Projects' }, { value: '18', label: 'Stories' }, { value: '$8.2K', label: 'This month' }],
      health: 73, healthColor: '#3b82f6', stripeColor: 'linear-gradient(90deg,#3b82f6,#8b5cf6,#ec4899,#f59e0b,#22c55e)',
      status: 'Active', statusType: 'green',
    },
    {
      type: 'PROGRAM', name: 'SaaS Product Dev',
      desc: 'Building KogiTask — a niche task management app. Early MVP phase.',
      stats: [{ value: '1', label: 'Projects' }, { value: '34', label: 'Stories' }, { value: '$0', label: 'Revenue' }],
      health: 38, healthColor: '#ef4444', stripeColor: 'linear-gradient(90deg,#f59e0b,#ef4444)',
      status: 'At risk', statusType: 'red',
    },
    {
      type: 'SUB-PORTFOLIO', name: 'Content & Education',
      desc: 'Newsletter, YouTube channel, and course development. Q2 goals in progress.',
      stats: [{ value: '3', label: 'Projects' }, { value: '11', label: 'Stories' }, { value: '$1.1K', label: 'Monthly' }],
      health: 55, healthColor: '#f59e0b', stripeColor: 'linear-gradient(90deg,#f59e0b,#22c55e)',
      status: 'Active', statusType: 'green',
    },
  ];

  // ─── WORK BOARD ────────────────────────────────────────────────────────
  backlogStories: StoryCard[] = [
    { title: 'Set up email notifications system', tags: ['feature'], pts: '3 pts' },
    { title: 'Write API documentation', tags: ['docs'], pts: '2 pts' },
    { title: 'Integrate analytics dashboard', tags: ['feature'], pts: '5 pts' },
  ];

  inProgressStories: StoryCard[] = [
    { title: 'Set up Stripe payment gateway', tags: ['blocked'], pts: '5 pts', extra: 'Blocked — needs API key from client', extraType: 'block', blocked: true, avatar: 'JD', avatarColor: '#4f46e5' },
    { title: 'Build product catalog page', tags: ['feature'], pts: '8 pts · 3d left', progress: 60 },
    { title: 'Implement cart & checkout flow', tags: ['feature'], pts: '13 pts', progress: 30 },
  ];

  reviewStories: StoryCard[] = [
    { title: 'User authentication flows', tags: ['review'], pts: '8 pts', extra: 'Awaiting client sign-off' },
    { title: 'Design system component library', tags: ['review'], pts: '5 pts', extra: '' },
  ];

  doneStories: StoryCard[] = [
    { title: 'Project setup & repo structure', tags: ['done'], pts: '3 pts' },
    { title: 'Database schema design', tags: ['done'], pts: '8 pts' },
    { title: 'UI wireframes approved', tags: ['done'], pts: '5 pts' },
  ];

  blockedStories: StoryCard[] = [
    { title: 'Set up API authentication with client CRM', tags: ['blocker'], pts: '', extra: 'Blocked 3 days · No API key access', aiNote: 'AI: Found 2 solutions in marketplace →' },
    { title: 'Mobile responsive testing', tags: ['blocker'], pts: '', extra: 'Needs real devices — request submitted' },
  ];

  // ─── EXCHANGE ──────────────────────────────────────────────────────────
  exchangeStats: StatCard[] = [
    { value: '$12,340', label: 'Revenue (YTD)',     change: '$3,200 vs last year',changeType: 'up'   },
    { value: '$2,180',  label: 'In Escrow',         change: '2 active orders',    changeType: 'info' },
    { value: '$4,820',  label: 'Available',         change: 'Ready to withdraw',  changeType: 'up'   },
    { value: '2',       label: 'Overdue Invoices',  change: '$3,400 outstanding', changeType: 'warn' },
  ];

  transactions: TxRow[] = [
    { icon: '💳', name: 'DesignCo — Invoice #108', meta: 'Mar 5, 2025', amount: '+$2,000', income: true },
    { icon: '🔧', name: 'Figma subscription', meta: 'Mar 1, 2025', amount: '-$45', income: false },
    { icon: '💳', name: 'StartupX — Project deposit', meta: 'Feb 28, 2025', amount: '+$1,500', income: true },
    { icon: '☁️', name: 'AWS hosting', meta: 'Feb 25, 2025', amount: '-$82', income: false },
    { icon: '💳', name: 'TechCorp — Sprint payment', meta: 'Feb 20, 2025', amount: '+$3,200', income: true },
  ];

  // ─── MARKETPLACE ───────────────────────────────────────────────────────
  listings: ListingCard[] = [
    { title: 'React / Next.js Full-Stack Development', seller: 'Alex Kim', sellerInitials: 'AK', sellerColor: '#2563eb', verified: true, rating: '4.9', reviews: 124, price: 'from $120/hr', bgColor: 'linear-gradient(135deg,#1e3a8a,#1e40af)', icon: '🖥️' },
    { title: 'Brand Identity Design — Logo + Guidelines', seller: 'Sofia M.', sellerInitials: 'SM', sellerColor: '#9333ea', verified: true, rating: '5.0', reviews: 88, price: '$850 fixed', bgColor: 'linear-gradient(135deg,#4c1d95,#7c3aed)', icon: '✦' },
    { title: 'Data Analysis & Business Intelligence Reports', seller: 'Rania N.', sellerInitials: 'RN', sellerColor: '#0d9488', verified: true, rating: '4.8', reviews: 62, price: 'from $200/proj', bgColor: 'linear-gradient(135deg,#064e3b,#065f46)', icon: '📊', aiMatch: true },
  ];

  // ─── IDEA STUDIO ───────────────────────────────────────────────────────
  ideaStages = [
    { icon: '⭐', label: '1. Capture',   colorClass: 'capture',  count: 6, active: false },
    { icon: '🔍', label: '2. Validate',  colorClass: 'validate', count: 4, active: false },
    { icon: '👁️', label: '3. Design',    colorClass: 'design',   count: 2, active: true  },
    { icon: '🚀', label: '4. Prototype', colorClass: 'proto',    count: 1, active: false },
    { icon: '🎉', label: '5. Launch',    colorClass: 'launch',   count: 1, active: false },
  ];

  // ─── GOVERNANCE ────────────────────────────────────────────────────────
  govMembers: MemberRow[] = [
    { initials: 'JD', color: '#4f46e5', name: 'Jordan Davis (you)', role: 'Founder', stake: '15% stake', voted: 'Voted',   votedType: 'yes'     },
    { initials: 'AK', color: '#2563eb', name: 'Alex Kim',           role: 'Developer',  stake: '12% stake', voted: 'Voted',   votedType: 'yes'     },
    { initials: 'SM', color: '#9333ea', name: 'Sofia Moreira',      role: 'Designer',   stake: '10% stake', voted: 'Pending', votedType: 'pending' },
  ];

  // ─── ANALYTICS ─────────────────────────────────────────────────────────
  analyticsStats: StatCard[] = [
    { value: '$12,340', label: 'Total Revenue (30d)',  change: '23% vs last period', changeType: 'up'  },
    { value: '18',      label: 'Orders Completed',     change: '6 more',             changeType: 'up'  },
    { value: '147',     label: 'Profile Views',        change: '31% growth',         changeType: 'up'  },
    { value: '4.9★',    label: 'Average Rating',       change: 'Based on 42 reviews', changeType: 'up' },
    { value: '62%',     label: 'OKR Progress',         change: 'Revenue goal Q3',    changeType: 'warn'},
  ];

  chartBars = [40, 60, 45, 80, 55, 90, 70, 85, 65, 95, 75, 88, 50, 72, 60, 100, 80, 65, 90, 78, 55, 85, 70, 92, 68, 75, 88, 95, 82, 100];

  // ─── COMMUNITY ─────────────────────────────────────────────────────────
  communityPosts = [
    { initials: 'JD', color: '#4f46e5', title: 'How I scaled my freelance income to $200K/yr', meta: 'in #business · 42 upvotes', time: '2h ago', replies: 18, likes: 42 },
    { initials: 'AK', color: '#2563eb', title: 'The best tools for remote async collaboration in 2025', meta: 'in #tools · 31 upvotes', time: '4h ago', replies: 12, likes: 31 },
    { initials: 'SM', color: '#9333ea', title: 'Tips for negotiating better rates with enterprise clients', meta: 'in #freelancing · 28 upvotes', time: '6h ago', replies: 9, likes: 28 },
    { initials: 'RN', color: '#0d9488', title: 'Using AI to automate 30% of my business admin', meta: 'in #ai · 55 upvotes', time: '1d ago', replies: 22, likes: 55 },
  ];

  // ─── WORK & OKRs ───────────────────────────────────────────────────────
  okrItems = [
    {
      objective: '💰 Revenue: Reach $15K MRR by end of Q3',
      pct: 62, statusType: 'amber',
      keyResults: [
        { label: 'Marketplace revenue', current: '$8.2K', target: '$10K', pct: 82, color: '#3b82f6' },
        { label: 'New client projects', current: '3', target: '5', pct: 60, color: '#f59e0b' },
        { label: 'Average project size', current: '$2.7K', target: '$3K', pct: 90, color: '#22c55e' },
      ]
    },
    {
      objective: '🚀 Growth: Increase marketplace visibility',
      pct: 45, statusType: 'red',
      keyResults: [
        { label: 'Profile views / month', current: '147', target: '300', pct: 49, color: '#8b5cf6' },
        { label: 'New service listings', current: '2', target: '5', pct: 40, color: '#ef4444' },
      ]
    },
  ];

  // ─── FUNDING ───────────────────────────────────────────────────────────
  fundingItems = [
    { name: 'Pre-Seed Round', type: 'Equity', stage: 'KogiTask SaaS', raised: '$42K', goal: '$150K', investors: 8, pct: 28, color: '#3b82f6', status: 'Active', statusType: 'blue' },
    { name: 'Revenue-Based Financing', type: 'RBF', stage: 'Design Studio', raised: '$25K', goal: '$50K', investors: 1, pct: 50, color: '#22c55e', status: 'Funded', statusType: 'green' },
  ];

  // ─── SETTINGS ──────────────────────────────────────────────────────────
  notifPrefs = [
    { label: 'Marketplace orders', desc: 'New orders, messages, disputes', email: true, push: true, sms: false },
    { label: 'Governance votes', desc: 'New proposals, vote deadlines', email: true, push: true, sms: false },
    { label: 'AI Agent alerts', desc: 'Blocked items, OKR risks', email: false, push: true, sms: false },
    { label: 'Payments received', desc: 'Invoice paid, escrow released', email: true, push: true, sms: true },
  ];

  // ─── SEARCH ────────────────────────────────────────────────────────────
  recentSearches = [
    'Stripe payment gateway', 'E-commerce Build sprint', 'Alex Kim developer',
    'Invoice INV-2024-108', 'OKR Q3 revenue goal',
  ];
}

bootstrapApplication(AppComponent);
