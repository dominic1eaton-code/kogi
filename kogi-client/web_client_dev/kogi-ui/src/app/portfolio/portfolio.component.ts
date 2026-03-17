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

  toggleSecondaryNav(): void {
    this.showSecondaryNav = !this.showSecondaryNav;
  }

  toggleItemsSection(): void {
    this.showItemsSection = !this.showItemsSection;
  }

  toggleContainersSection(): void {
    this.showContainersSection = !this.showContainersSection;
  }
}
