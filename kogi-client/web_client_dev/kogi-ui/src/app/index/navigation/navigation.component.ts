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
      { label: 'Dashboard', route: '/portfolio/dashboard' },
      { label: 'Items', route: '/portfolio/items' },
      { label: 'Registry', route: '/portfolio/registry' },
      { label: 'Analytics', route: '/portfolio/analytics' },
      { label: 'ItemBook', route: '/portfolio/itembook/charter' },
      { label: 'Collaboration', route: '/portfolio/collaboration' },
      { label: 'Create Portfolio', route: '/portfolio/new' }
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
      { label: 'Banking', route: '/wallet/banking' },
      { label: 'Ledger', route: '/wallet/ledger' },
      { label: 'Escrow', route: '/wallet/escrow' },
      { label: 'Invoices', route: '/wallet/invoices' },
      { label: 'Investments', route: '/wallet/investments' },
      { label: 'Funding', route: '/wallet/funding' },
      { label: 'Benefits', route: '/wallet/benefits' },
      { label: 'Grants', route: '/wallet/grants' },
      { label: 'Group Economics', route: '/wallet/group-economics' },
      { label: 'Campaigns', route: '/wallet/campaigns' }
    ]
  },
  {
    key: 'office',
    label: 'Office',
    icon: '&#8862;',
    route: '/office',
    secondary: [
      { label: 'Overview', route: '/office/overview', meta: 'Office dashboard' },
      { label: 'Inbox', route: '/office/inbox', meta: 'Messages & requests' },
      { label: 'Schedule', route: '/office/schedule', meta: 'Timelines & timeboxes' },
      { label: 'Calendar', route: '/office/calendar', meta: 'Meetings & deadlines' },
      { label: 'Studio', route: '/office/studio', meta: 'Ideas, designs, assets' },
      { label: 'Contacts', route: '/office/contacts', meta: 'Contactbook & network' }
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
      { label: 'Dashboard', route: '/hub/dashboard' },
      { label: 'Governance', route: '/hub/governance' },
      { label: 'Voting', route: '/hub/voting' },
      { label: 'Allocation', route: '/hub/allocation' },
      { label: 'Distribution', route: '/hub/distribution' },
      { label: 'Collaboration', route: '/hub/collaboration' },
      { label: 'Negotiations', route: '/hub/negotiations' },
      { label: 'Teams', route: '/hub/teams' },
      { label: 'Organizations', route: '/hub/organizations' },
      { label: 'Collectives', route: '/hub/collectives' },
      { label: 'Open Source', route: '/hub/open-source' },
      { label: 'Group Economics', route: '/hub/group-economics' },
      { label: 'Community Showcase', route: '/hub/community-showcase' }
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
