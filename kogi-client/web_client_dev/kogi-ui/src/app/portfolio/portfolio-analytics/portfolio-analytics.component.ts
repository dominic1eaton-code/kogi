import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-portfolio-analytics',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './portfolio-analytics.component.html',
  styleUrl: './portfolio-analytics.component.css',
  host: {
    class: 'block w-full'
  }
})
export class PortfolioAnalyticsComponent {
  modelScores = [
    { label: 'Portfolio Health', value: '86', trend: '+4', tone: 'text-[#10b981]' },
    { label: 'Program Alignment', value: '81', trend: '+2', tone: 'text-[#60a5fa]' },
    { label: 'Subportfolio Rollup', value: '78', trend: '-1', tone: 'text-[#f59e0b]' },
    { label: 'Resource Utilisation', value: '74%', trend: '+5%', tone: 'text-[#8b5cf6]' },
    { label: 'Asset Value', value: '$2.4M', trend: '+6%', tone: 'text-[#22c55e]' },
    { label: 'Binder Coverage', value: '92%', trend: '+1%', tone: 'text-[#f472b6]' }
  ];

  kpiRows = [
    { name: 'Active Components', value: '94', target: '100', status: 'On track' },
    { name: 'Budget Efficiency', value: '68%', target: '70%', status: 'Stable' },
    { name: 'Risk Mitigation', value: '81%', target: '85%', status: 'Needs review' },
    { name: 'Contribution Velocity', value: '3.2/wk', target: '4.0/wk', status: 'Lagging' }
  ];

  insightFeed = [
    '3 contributors have pending submissions awaiting steward review.',
    'Binder coverage improved by 6% after Q2 cleanup.',
    'Portable benefits utilization trending upward in design teams.',
    'Resource share expiry for Research Hub in 12 days.'
  ];

  modelCatalog = [
    'PortfolioHealth',
    'ProjectMetrics',
    'ProgramAlignment',
    'SubPortfolioRollup',
    'ResourceUtilisation',
    'AssetValue',
    'ArtifactMaturity',
    'BinderCoverage',
    'BookConsistency'
  ];
}
