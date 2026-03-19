import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-wallet-group-economics',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './wallet-group-economics.component.html',
  styleUrl: './wallet-group-economics.component.css',
  host: {
    class: 'block w-full'
  }
})
export class WalletGroupEconomicsComponent {
  summaryCards = [
    { label: 'Group Treasury', value: '$128,000', meta: 'Shared funds', tone: 'text-[#10b981]' },
    { label: 'Distribution Cycle', value: 'Biweekly', meta: 'Next run Mar 22', tone: 'text-[#60a5fa]' },
    { label: 'Reserve Pool', value: '$24,500', meta: 'Stability fund', tone: 'text-[#f59e0b]' },
    { label: 'Allocation Rules', value: '6', meta: 'Active policies', tone: 'text-[#a855f7]' }
  ];

  pools = [
    { name: 'Operations Pool', value: '$48,200', policy: '40% of inflows' },
    { name: 'Community Dividend', value: '$22,400', policy: '20% of surplus' },
    { name: 'Innovation Fund', value: '$31,100', policy: '25% of revenue' },
    { name: 'Mutual Aid', value: '$9,800', policy: '15% of grants' }
  ];

  payouts = [
    { member: 'Studio Collective', amount: '$3,400', status: 'Scheduled' },
    { member: 'Open Source Guild', amount: '$2,100', status: 'Pending vote' },
    { member: 'Coop Team Alpha', amount: '$1,600', status: 'Approved' }
  ];
}
