import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-hub-governance',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './hub-governance.component.html',
  styleUrl: './hub-governance.component.css',
  host: {
    class: 'block w-full'
  }
})
export class HubGovernanceComponent {
  summaryCards = [
    { label: 'Policies', value: '24', meta: 'Active policies', tone: 'text-[#60a5fa]' },
    { label: 'Proposals', value: '9', meta: 'In review', tone: 'text-[#10b981]' },
    { label: 'Ratified', value: '17', meta: 'Last 90 days', tone: 'text-[#f59e0b]' },
    { label: 'Compliance', value: '96%', meta: 'Audit ready', tone: 'text-[#a855f7]' }
  ];

  proposals = [
    { title: 'Budget Allocation Q2', status: 'Voting', owner: 'Finance Council' },
    { title: 'Community Charter Update', status: 'Review', owner: 'Steward Circle' },
    { title: 'Resource Access Policy', status: 'Draft', owner: 'Operations' },
    { title: 'Restitution Protocol', status: 'Consensus', owner: 'Justice Council' }
  ];

  frameworks = [
    { name: 'Holonic Governance', desc: 'Nested councils with clear mandates.' },
    { name: 'Consensus + Delegate', desc: 'Hybrid voting with fallback delegates.' },
    { name: 'Cooperative Charter', desc: 'Member rights, obligations, and dividends.' }
  ];
}
