import { CommonModule } from '@angular/common';
import { Component, inject } from '@angular/core';
import { Router, RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';
import { NavigationComponent } from '../index/navigation/navigation.component';

type PortfolioNavItem = {
  label: string;
  route: string;
  exact?: boolean;
  queryParams?: Record<string, string>;
};

type PortfolioNavSection = {
  label: string;
  items: PortfolioNavItem[];
};

@Component({
  selector: 'app-portfolio',
  standalone: true,
  imports: [RouterLink, RouterLinkActive, RouterOutlet, CommonModule, NavigationComponent],
  templateUrl: './portfolio.component.html',
  styleUrl: './portfolio.component.css',
  host: {
    class: 'block w-full min-h-screen'
  }
})
export class PortfolioComponent {
  showSecondaryNav = true;

  primaryTabs: PortfolioNavItem[] = [
    { label: 'Dashboard', route: '/portfolio/dashboard', exact: true },
    { label: 'Items', route: '/portfolio/items' },
    { label: 'Resources', route: '/portfolio/resources' },
    { label: 'Content', route: '/portfolio/content' },
    { label: 'Library', route: '/portfolio/itembook/library' },
    { label: 'Registry', route: '/portfolio/registry' },
    { label: 'Metrics', route: '/portfolio/analytics' },
    { label: 'Query', route: '/portfolio/query' }
  ];

  secondaryNav: PortfolioNavSection[] = [
    {
      label: 'Core Views',
      items: [
        { label: 'Overview', route: '/portfolio/dashboard', exact: true },
        { label: 'Grid View', route: '/portfolio/items', queryParams: { view: 'grid' } },
        { label: 'List View', route: '/portfolio/items', queryParams: { view: 'list' } },
        { label: 'Tree View', route: '/portfolio/items', queryParams: { view: 'tree' } },
        { label: 'Board View', route: '/portfolio/items', queryParams: { view: 'board' } }
      ]
    },
    {
      label: 'Structures',
      items: [
        { label: 'ItemBook', route: '/portfolio/itembook' },
        { label: 'Binder', route: '/portfolio/binder' },
        { label: 'Registry', route: '/portfolio/registry' },
        { label: 'Folder View', route: '/portfolio/folder' },
        { label: 'Graph View', route: '/portfolio/graph' },
        { label: 'Subportfolio', route: '/portfolio/subportfolio' }
      ]
    },
    {
      label: 'Knowledge',
      items: [
        { label: 'Notebook', route: '/portfolio/notebook' },
        { label: 'Guidebook', route: '/portfolio/guidebook' },
        { label: 'Workspace', route: '/portfolio/workspace' }
      ]
    },
    {
      label: 'Operations',
      items: [
        { label: 'Analytics', route: '/portfolio/analytics' },
        { label: 'PQL Query', route: '/portfolio/query' },
        { label: 'Collaboration', route: '/portfolio/collaboration' },
        { label: 'Component Detail', route: '/portfolio/detail/component' },
        { label: 'Program Detail', route: '/portfolio/detail/program' },
        { label: 'Resource Detail', route: '/portfolio/detail/resource' },
        { label: 'Asset Detail', route: '/portfolio/detail/asset' },
        { label: 'Artifact Detail', route: '/portfolio/detail/artifact' }
      ]
    },
    {
      label: 'Create',
      items: [
        { label: 'New Component Wizard', route: '/portfolio/new' },
        { label: 'Templates & Imports', route: '/portfolio/registry' }
      ]
    }
  ];

  // Inject the Router service
  private router = inject(Router);

  toggleSecondaryNav(): void {
    this.showSecondaryNav = !this.showSecondaryNav;
  }

  // Method to handle the navigation
  logout(): void {
    // Perform any necessary logic here
    console.log('Logging out of client session...');

    // Navigate to the specified route
    this.router.navigate(['/login']);
  }
}

