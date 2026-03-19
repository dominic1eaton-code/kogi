import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-wallet-grants',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './wallet-grants.component.html',
  styleUrl: './wallet-grants.component.css',
  host: {
    class: 'block w-full'
  }
})
export class WalletGrantsComponent {
  summaryCards = [
    { label: 'Active Grants', value: '7', meta: '$82,000 total', tone: 'text-[#10b981]' },
    { label: 'Submitted', value: '4', meta: 'Awaiting review', tone: 'text-[#60a5fa]' },
    { label: 'Awarded', value: '$28,500', meta: 'Last 90 days', tone: 'text-[#f59e0b]' },
    { label: 'Compliance', value: '93%', meta: 'Reports on time', tone: 'text-[#a855f7]' }
  ];

  pipeline = [
    { name: 'Green Futures Fund', stage: 'Submitted', amount: '$18,000', due: 'Apr 2' },
    { name: 'Community Build Grant', stage: 'Negotiation', amount: '$12,500', due: 'Mar 28' },
    { name: 'Open Source Fellowship', stage: 'Awarded', amount: '$6,000', due: 'Mar 20' },
    { name: 'Education Access', stage: 'Draft', amount: '$4,500', due: 'Apr 8' }
  ];

  reporting = [
    { task: 'Impact report - Open Source Fellowship', status: 'Due Apr 10' },
    { task: 'Financial audit - Community Build', status: 'Due Apr 15' },
    { task: 'Milestone update - Green Futures', status: 'Due Apr 20' }
  ];
}
