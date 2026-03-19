import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-hub-voting',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './hub-voting.component.html',
  styleUrl: './hub-voting.component.css',
  host: {
    class: 'block w-full'
  }
})
export class HubVotingComponent {
  summaryCards = [
    { label: 'Active Votes', value: '6', meta: '3 closing soon', tone: 'text-[#10b981]' },
    { label: 'Participation', value: '78%', meta: '7d rolling', tone: 'text-[#60a5fa]' },
    { label: 'Quorum', value: '62%', meta: 'Target 55%', tone: 'text-[#f59e0b]' },
    { label: 'Delegations', value: '24', meta: 'Trusted delegates', tone: 'text-[#a855f7]' }
  ];

  ballots = [
    { title: 'Budget Allocation Q2', status: 'Open', close: 'Mar 22' },
    { title: 'Resource Access Policy', status: 'Open', close: 'Mar 24' },
    { title: 'Restitution Proposal 12', status: 'Consensus', close: 'Mar 25' },
    { title: 'Community Charter Update', status: 'Draft', close: 'Mar 28' }
  ];

  voters = [
    { group: 'Council Delegates', weight: '32%', status: 'Aligned' },
    { group: 'Member Assembly', weight: '48%', status: 'Voting' },
    { group: 'Advisory Board', weight: '20%', status: 'Pending' }
  ];
}
