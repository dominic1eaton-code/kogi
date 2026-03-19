import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-portfolio-subportfolio',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './portfolio-subportfolio.component.html',
  styleUrl: './portfolio-subportfolio.component.css',
  host: {
    class: 'block w-full'
  }
})
export class PortfolioSubportfolioComponent {
  summary = [
    { label: 'Parent Portfolio', value: 'Alpha Platform' },
    { label: 'Rollup Score', value: '78' },
    { label: 'Items', value: '16' },
    { label: 'Health', value: '82' }
  ];

  childItems = [
    { name: 'Design System Library', type: 'Program', status: 'Active' },
    { name: 'Brand Identity System', type: 'Project', status: 'Active' },
    { name: 'Research Hub', type: 'Resource', status: 'Active' }
  ];
}
