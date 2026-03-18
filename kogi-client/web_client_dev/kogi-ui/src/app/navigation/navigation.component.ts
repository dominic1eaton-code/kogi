import { CommonModule } from '@angular/common';
import { Component, computed, signal } from '@angular/core';
import { RouterLink, RouterLinkActive } from '@angular/router';

type NavKey = 'dashboard' | 'portfolio' | 'wallet' | 'office' | 'spaces' | 'hub' | 'assistant';

type SecondaryItem = {
  label: string;
  route?: string;
  meta?: string;
};

type NavItem = {
  key: NavKey;
  label: string;
  icon: string;
  route: string;
  secondary: SecondaryItem[];
};

const NAV_ITEMS: NavItem[] = [
  {
    key: 'dashboard',
    label: 'Dashboard',
    icon: '&#9671;',
    route: '/dashboard',
    secondary: [
      { label: 'Overview', route: '/dashboard' },
      { label: 'Insights', meta: 'Health, OKRs, trends' },
      { label: 'Notifications', meta: 'Alerts and mentions' },
      { label: 'Quick Actions', meta: 'Shortcuts & macros' }
    ]
  },
  {
    key: 'portfolio',
    label: 'Portfolio',
    icon: '&#9776;',
    route: '/portfolio',
    secondary: [
      { label: 'Overview', route: '/portfolio' },
      { label: 'Create Portfolio', route: '/portfolio/new' },
      { label: 'Collections', meta: 'Case studies & showcases' },
      { label: 'Analytics', meta: 'Views, leads, conversions' }
    ]
  },
  {
    key: 'wallet',
    label: 'Wallet',
    icon: '&#8862;',
    route: '/wallet',
    secondary: [
      { label: 'Dashboard', route: '/wallet/dashboard' },
      { label: 'Wallets Overview', route: '/wallet/wallets/overview' },
      { label: 'Taxes', route: '/wallet/wallets/taxes' },
      { label: 'Auto-Split Rules', meta: 'Disbursements & rules' }
    ]
  },
  {
    key: 'office',
    label: 'Office',
    icon: '&#8862;',
    route: '/office',
    secondary: [
      { label: 'Overview', route: '/office' },
      { label: 'Workboard', meta: 'Stories, sprints, blockers' },
      { label: 'Calendar', meta: 'Meetings & deadlines' },
      { label: 'Team', meta: 'Collaborators & roles' }
    ]
  },
  {
    key: 'spaces',
    label: 'Spaces',
    icon: '&#9671;',
    route: '/spaces',
    secondary: [
      { label: 'Dashboard', route: '/spaces/dashboard' },
      { label: 'Spaces', route: '/spaces/spaces' },
      { label: 'Rooms', route: '/spaces/rooms' },
      { label: 'Feed', route: '/spaces/feed' },
      { label: 'Timeline', route: '/spaces/timeline' },
      { label: 'Channels', route: '/spaces/channels' },
      { label: 'Events', route: '/spaces/events' },
      { label: 'Network', route: '/spaces/network' }
    ]
  },
  {
    key: 'hub',
    label: 'Marketplace',
    icon: '&#9711;',
    route: '/marketplace',
    secondary: [
      { label: 'Overview', route: '/hub' },
      { label: 'Marketplace', meta: 'Offers, listings, bids' },
      { label: 'Exchange', meta: 'Transactions & escrow' },
      { label: 'Broadcasts', meta: 'Announcements & drops' }
    ]
  },
  {
    key: 'hub',
    label: 'Hub',
    icon: '&#9711;',
    route: '/hub',
    secondary: [
      { label: 'Overview', route: '/hub' },
      { label: 'Governance', meta: 'Offers, listings, bids' },
      { label: 'Teams', meta: 'Transactions & escrow' },
      { label: 'Organizations', meta: 'Announcements & drops' }
    ]
  },
  {
    key: 'assistant',
    label: 'Assistant',
    icon: '&#10022;',
    route: '/assistant',
    secondary: [
      { label: 'Overview', route: '/assistant' },
      { label: 'Workflows', meta: 'Automation & agents' },
      { label: 'Memory', meta: 'Context & preferences' },
      { label: 'Tools', meta: 'Connected services' }
    ]
  }
];

@Component({
  selector: 'app-navigation',
  standalone: true,
  imports: [CommonModule, RouterLink, RouterLinkActive],
  templateUrl: './navigation.component.html',
  styleUrl: './navigation.component.css'
})
export class NavigationComponent {
  navItems = NAV_ITEMS;
  expandedSection = signal<NavKey | null>(null);

  activeSection = computed(() => {
    const key = this.expandedSection();
    return this.navItems.find((item) => item.key === key) ?? null;
  });

  openSection(key: NavKey): void {
    this.expandedSection.set(key);
  }

  toggleSection(key: NavKey, event?: Event): void {
    if (event) {
      event.stopPropagation();
      event.preventDefault();
    }
    this.expandedSection.set(this.expandedSection() === key ? null : key);
  }

  closePanel(event?: Event): void {
    if (event) {
      event.stopPropagation();
      event.preventDefault();
    }
    this.expandedSection.set(null);
  }
}
