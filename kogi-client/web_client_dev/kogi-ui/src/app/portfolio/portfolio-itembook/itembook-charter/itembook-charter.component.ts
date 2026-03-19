import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-itembook-charter',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './itembook-charter.component.html',
  styleUrl: './itembook-charter.component.css',
  host: {
    class: 'block w-full'
  }
})
export class ItembookCharterComponent {
  objectives = [
    'Define the portfolio mission and success criteria.',
    'Align stakeholders on scope, risks, and constraints.',
    'Approve governance policies and ownership structure.'
  ];

  stakeholders = [
    { role: 'Owner', name: 'Jordan Davis' },
    { role: 'Steward', name: 'Operations Council' },
    { role: 'Contributors', name: 'Program leads, project owners' }
  ];

  risks = [
    { label: 'Budget Overrun', mitigation: 'Monthly budget gates + alerts' },
    { label: 'Governance Drift', mitigation: 'Quarterly audit + vote' },
    { label: 'Resource Saturation', mitigation: 'Capacity planning + hiring' }
  ];

  approvals = [
    { label: 'Charter Approval', value: 'Approved - Mar 10, 2026' },
    { label: 'Policy Set', value: 'PortfolioCore-v2' },
    { label: 'Review Cadence', value: 'Quarterly' }
  ];
}
