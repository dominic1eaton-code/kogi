import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-portfolio',
  standalone: true,
  imports: [CommonModule, RouterLink],
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
}
