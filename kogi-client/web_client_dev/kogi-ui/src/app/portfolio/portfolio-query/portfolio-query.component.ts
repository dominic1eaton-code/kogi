import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-portfolio-query',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './portfolio-query.component.html',
  styleUrl: './portfolio-query.component.css',
  host: {
    class: 'block w-full'
  }
})
export class PortfolioQueryComponent {
  sampleQuery = `SELECT items
WHERE type IN ('Project','Program')
  AND status = 'Active'
  AND budget_used > 50000
ORDER BY health_score DESC
LIMIT 25;`;

  queryResults = [
    { name: 'Alpha Platform', type: 'Portfolio', status: 'Active', health: 87, budget: '$45k' },
    { name: 'Community Growth', type: 'Program', status: 'Active', health: 76, budget: '$22k' },
    { name: 'Brand Identity System', type: 'Project', status: 'Active', health: 68, budget: '$18k' }
  ];

  queryMetrics = [
    { label: 'Rows Returned', value: '25' },
    { label: 'Execution Time', value: '38ms' },
    { label: 'Saved Views', value: '14' },
    { label: 'Active Filters', value: '6' }
  ];
}
