import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-hub-dashboard',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './hub-dashboard.component.html',
  styleUrl: './hub-dashboard.component.css',
  host: {
    class: 'block w-full'
  }
})
export class HubDashboardComponent {
  summaryCards = [
    { label: 'Active Orgs', value: '18', meta: 'Federated teams', tone: 'text-[#60a5fa]' },
    { label: 'Open Votes', value: '6', meta: 'Governance ballots', tone: 'text-[#10b981]' },
    { label: 'Allocations', value: '$42,000', meta: 'Current cycle', tone: 'text-[#f59e0b]' },
    { label: 'Negotiations', value: '3', meta: 'In progress', tone: 'text-[#a855f7]' }
  ];

  agenda = [
    { title: 'Quarterly Budget Ratification', date: 'Mar 22', status: 'Voting live' },
    { title: 'Resource Allocation Review', date: 'Mar 24', status: 'Ready' },
    { title: 'Restitution Proposal 12', date: 'Mar 26', status: 'Draft' }
  ];

  spotlight = [
    { name: 'Open Source Guild', focus: 'Governance update', status: 'Submitted' },
    { name: 'Coop Design Collective', focus: 'New member intake', status: 'Active' },
    { name: 'Autonomous Cell Sigma', focus: 'Resource request', status: 'Review' }
  ];
}
