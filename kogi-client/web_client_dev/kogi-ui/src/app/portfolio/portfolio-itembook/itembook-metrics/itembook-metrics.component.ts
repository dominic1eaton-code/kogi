import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-itembook-metrics',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './itembook-metrics.component.html',
  styleUrl: './itembook-metrics.component.css',
  host: {
    class: 'block w-full'
  }
})
export class ItembookMetricsComponent {
  metrics = [
    { name: 'Engagement', value: '76%', target: '80%' },
    { name: 'Budget Efficiency', value: '68%', target: '70%' },
    { name: 'Completion', value: '58%', target: '65%' },
    { name: 'Governance Compliance', value: '88%', target: '90%' }
  ];

  scorecards = [
    { label: 'PortfolioHealth', value: '86' },
    { label: 'BinderCoverage', value: '92%' },
    { label: 'ProgramAlignment', value: '81' },
    { label: 'ResourceUtilisation', value: '74%' }
  ];
}
