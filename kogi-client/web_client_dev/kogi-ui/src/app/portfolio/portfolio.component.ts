import { CommonModule } from '@angular/common';
import { Component, inject } from '@angular/core';
import { RouterLink, Router, RouterLinkActive, RouterOutlet } from '@angular/router';
import { NavigationComponent } from '../index/navigation/navigation.component';

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
  showItemsSection = true;
  showContainersSection = true;
  viewMode: 'grid' | 'list' | 'tree' | 'board' = 'grid';
  
  // Inject the Router service
  private router = inject(Router);

  readonly activeViewTabClass = 'bg-[#0f1f26] text-[#e6f1f4]';
  readonly inactiveViewTabClass = 'text-[#8ea6ad]';

  toggleSecondaryNav(): void {
    this.showSecondaryNav = !this.showSecondaryNav;
  }

  setViewMode(view: 'grid' | 'list' | 'tree' | 'board'): void {
    this.viewMode = view;
  }

  toggleItemsSection(): void {
    this.showItemsSection = !this.showItemsSection;
  }

  toggleContainersSection(): void {
    this.showContainersSection = !this.showContainersSection;
  }

  // Method to handle the navigation
  logout(): void {
    // Perform any necessary logic here
    console.log('Logging out of client session...');

    // Navigate to the specified route
    this.router.navigate(['/login']);
  }
}
