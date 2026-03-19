import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-portfolio-registry',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './portfolio-registry.component.html',
  styleUrl: './portfolio-registry.component.css',
  host: {
    class: 'block w-full'
  }
})
export class PortfolioRegistryComponent {
  columnGroups = [
    { title: 'Identity', desc: 'Name, type, owner, tags, IDs, classification.' },
    { title: 'Lifecycle', desc: 'State, stage, approvals, version history.' },
    { title: 'Financial & Budget', desc: 'Budget, spend, funding sources, ROI.' },
    { title: 'Timeline & Schedule', desc: 'Milestones, due dates, cadence.' },
    { title: 'Governance & Policy', desc: 'Permissions, votes, compliance.' },
    { title: 'Analytics & Metrics', desc: 'Health, KPIs, rollups, forecasts.' }
  ];

  sheetRegistry = [
    { code: 'SHT-001', title: 'Master Registry Sheet', purpose: 'Portfolio component index.' },
    { code: 'SHT-002', title: 'Hierarchy Sheet', purpose: 'Parent-child relationships & rollups.' },
    { code: 'SHT-004', title: 'Projects Sheet', purpose: 'Projects, programs, initiatives.' },
    { code: 'SHT-005', title: 'Tasks & Backlog', purpose: 'Execution backlog & priority.' },
    { code: 'SHT-009', title: 'Finances Sheet', purpose: 'Budgets, capital, grants.' },
    { code: 'SHT-011', title: 'Work & Gigs', purpose: 'Labor allocations & gigs.' },
    { code: 'SHT-015', title: 'Portable Benefits', purpose: 'HSA, IRA, PTO, coverage.' },
    { code: 'SHT-020', title: 'Shared Portfolios', purpose: 'Collaboration & attribution.' }
  ];

  viewEngine = [
    { title: 'View Definitions', desc: 'Saved filters, groupings, layout presets.' },
    { title: 'Filter System', desc: 'Column filters, smart predicates, PQL.' },
    { title: 'Sort, Group & Pivot', desc: 'Multi-axis grouping and rollups.' },
    { title: 'Board Modes', desc: 'Kanban, swimlanes, status columns.' }
  ];

  columnTypes = [
    'Text',
    'Number',
    'Currency',
    'Date',
    'State',
    'Tag',
    'Person',
    'Relation',
    'Formula',
    'Status',
    'Progress'
  ];

  accessControl = [
    { label: 'Permission Tiers', value: 'Owner - Admin - Editor - Commenter - Viewer' },
    { label: 'Row-Level Controls', value: 'Component-specific access policies.' },
    { label: 'Column-Level Controls', value: 'Sensitive columns gated by policy.' },
    { label: 'Audit Logs', value: 'Event log with CRDT merge history.' }
  ];
}
