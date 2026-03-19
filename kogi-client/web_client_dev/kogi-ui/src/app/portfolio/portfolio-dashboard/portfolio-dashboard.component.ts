import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-portfolio-dashboard',
  standalone: true,
  imports: [CommonModule, RouterLink],
  templateUrl: './portfolio-dashboard.component.html',
  styleUrl: './portfolio-dashboard.component.css',
  host: {
    class: 'block w-full'
  }
})
export class PortfolioDashboardComponent {
  summaryCards = [
    { label: 'Total Components', value: '128', delta: '+12 this month', tone: 'text-[#60a5fa]' },
    { label: 'Active Items', value: '94', delta: '73% active ratio', tone: 'text-[#10b981]' },
    { label: 'Portfolio Health', value: '86', delta: 'Target > 80', tone: 'text-[#22c55e]' },
    { label: 'Budget Utilized', value: '$410k', delta: '68% used', tone: 'text-[#f59e0b]' },
    { label: 'At Risk', value: '7', delta: '2 escalations', tone: 'text-[#f472b6]' },
    { label: 'Pending Reviews', value: '14', delta: 'Governance queue', tone: 'text-[#8b5cf6]' }
  ];

  sheetRegistry = [
    { code: 'SHT-001', title: 'Master Registry Sheet', desc: 'Canonical component index & metadata.' },
    { code: 'SHT-002', title: 'Portfolio Hierarchy Sheet', desc: 'Parent-child ownership & rollups.' },
    { code: 'SHT-004', title: 'Projects Sheet', desc: 'Active projects, status, risks, dependencies.' },
    { code: 'SHT-005', title: 'Tasks & Backlog Sheet', desc: 'Execution backlog with priorities.' },
    { code: 'SHT-009', title: 'Finances Sheet', desc: 'Budgets, forecasts, capital flows.' },
    { code: 'SHT-015', title: 'Portable Benefits Sheet', desc: 'HSA, IRA, PTO, coverage status.' }
  ];

  healthSignals = [
    { label: 'Lifecycle Coverage', value: '92%', detail: 'Lifecycle completeness across items.' },
    { label: 'Governance Compliance', value: '88%', detail: 'Policies accepted, votes completed.' },
    { label: 'Resource Utilisation', value: '74%', detail: 'Capacity usage across resource pools.' },
    { label: 'Program Alignment', value: '81', detail: 'Alignment score across programs.' }
  ];

  collaborationHighlights = [
    { title: 'Team Portfolio', status: 'Admin-led', meta: '4 active contributors' },
    { title: 'Collective Portfolio', status: 'Open contribution', meta: '12 pending reviews' },
    { title: 'Federation Portfolio', status: 'Multi-org', meta: '2 federated nodes' },
    { title: 'Crowdresourced Portfolio', status: 'Campaign live', meta: '48 submissions' }
  ];

  benefitAccounts = [
    { label: 'Health (HSA)', balance: '$18,400', status: 'On track', tone: 'text-[#10b981]' },
    { label: 'Retirement (SEP-IRA)', balance: '$92,100', status: 'Ahead of target', tone: 'text-[#60a5fa]' },
    { label: 'Professional Dev', balance: '$3,600', status: '75% used', tone: 'text-[#f59e0b]' },
    { label: 'Portable Savings', balance: '$12,900', status: 'Stable', tone: 'text-[#8b5cf6]' }
  ];

  quickLinks = [
    { label: 'Open ItemBook', route: '/portfolio/itembook/charter' },
    { label: 'Run PQL Query', route: '/portfolio/query' },
    { label: 'View Collaboration', route: '/portfolio/collaboration' },
    { label: 'Review Registry', route: '/portfolio/registry' }
  ];
}

