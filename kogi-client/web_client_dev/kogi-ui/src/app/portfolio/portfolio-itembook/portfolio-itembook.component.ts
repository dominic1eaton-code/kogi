import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';
import { RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';

type ItemBookNav = { label: string; route: string };

@Component({
  selector: 'app-portfolio-itembook',
  standalone: true,
  imports: [CommonModule, RouterLink, RouterLinkActive, RouterOutlet],
  templateUrl: './portfolio-itembook.component.html',
  styleUrl: './portfolio-itembook.component.css',
  host: {
    class: 'block w-full'
  }
})
export class PortfolioItembookComponent {
  navItems: ItemBookNav[] = [
    { label: 'Charter', route: '/portfolio/itembook/charter' },
    { label: 'Workspace', route: '/portfolio/itembook/workspace' },
    { label: 'Catalogue', route: '/portfolio/itembook/catalogue' },
    { label: 'Schedule', route: '/portfolio/itembook/schedule' },
    { label: 'Metrics', route: '/portfolio/itembook/metrics' },
    { label: 'Library', route: '/portfolio/itembook/library' },
    { label: 'Logs', route: '/portfolio/itembook/logs' }
  ];
}
