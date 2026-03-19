import { CommonModule } from '@angular/common';
import { Component, inject } from '@angular/core';
import { RouterLink, RouterLinkActive, RouterOutlet, Router } from '@angular/router';
import { NavigationComponent } from '../index/navigation/navigation.component';

type HubNavItem = {
  label: string;
  route: string;
  exact?: boolean;
};

type HubNavSection = {
  label: string;
  items: HubNavItem[];
};

@Component({
  selector: 'app-hub',
  standalone: true,
  imports: [RouterLink, RouterLinkActive, RouterOutlet, CommonModule, NavigationComponent],
  templateUrl: './hub.component.html',
  styleUrl: './hub.component.css',
  host: {
    class: 'block w-full min-h-screen'
  }
})
export class HubComponent {
  showSecondaryNav = true;

  primaryTabs: HubNavItem[] = [
    { label: 'Dashboard', route: '/hub/dashboard', exact: true },
    { label: 'Governance', route: '/hub/governance' },
    { label: 'Voting', route: '/hub/voting' },
    { label: 'Allocation', route: '/hub/allocation' },
    { label: 'Distribution', route: '/hub/distribution' },
    { label: 'Collaboration', route: '/hub/collaboration' },
    { label: 'Restitution', route: '/hub/restitution' },
    { label: 'Negotiations', route: '/hub/negotiations' },
    { label: 'Teams', route: '/hub/teams' },
    { label: 'Organizations', route: '/hub/organizations' },
    { label: 'Collectives', route: '/hub/collectives' },
    { label: 'Cooperatives', route: '/hub/cooperatives' },
    { label: 'Federations', route: '/hub/federations' },
    { label: 'Autonomous', route: '/hub/autonomous' },
    { label: 'Open Source', route: '/hub/open-source' },
    { label: 'Group Economics', route: '/hub/group-economics' },
    { label: 'Crowdfund', route: '/hub/resource-crowdfund' },
    { label: 'Showcase', route: '/hub/community-showcase' }
  ];

  secondaryNav: HubNavSection[] = [
    {
      label: 'Governance',
      items: [
        { label: 'Overview', route: '/hub/governance', exact: true },
        { label: 'Voting', route: '/hub/voting' },
        { label: 'Allocation', route: '/hub/allocation' },
        { label: 'Distribution', route: '/hub/distribution' },
        { label: 'Restitution', route: '/hub/restitution' },
        { label: 'Negotiations', route: '/hub/negotiations' }
      ]
    },
    {
      label: 'Organizations',
      items: [
        { label: 'Teams', route: '/hub/teams' },
        { label: 'Organizations', route: '/hub/organizations' },
        { label: 'Collectives', route: '/hub/collectives' },
        { label: 'Cooperatives', route: '/hub/cooperatives' },
        { label: 'Federations', route: '/hub/federations' },
        { label: 'Autonomous Cells', route: '/hub/autonomous' }
      ]
    },
    {
      label: 'Community',
      items: [
        { label: 'Open Source', route: '/hub/open-source' },
        { label: 'Collaboration', route: '/hub/collaboration' },
        { label: 'Resource Crowdfund', route: '/hub/resource-crowdfund' },
        { label: 'Community Showcase', route: '/hub/community-showcase' }
      ]
    },
    {
      label: 'Economics',
      items: [
        { label: 'Group Economics', route: '/hub/group-economics' },
        { label: 'Distribution', route: '/hub/distribution' }
      ]
    }
  ];

  private router = inject(Router);

  toggleSecondaryNav(): void {
    this.showSecondaryNav = !this.showSecondaryNav;
  }

  logout(): void {
    console.log('Logging out of client session...');
    this.router.navigate(['/login']);
  }
}
