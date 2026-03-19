import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-wallet-campaigns',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './wallet-campaigns.component.html',
  styleUrl: './wallet-campaigns.component.css',
  host: {
    class: 'block w-full'
  }
})
export class WalletCampaignsComponent {
  summaryCards = [
    { label: 'Active Campaigns', value: '3', meta: 'Crowdfund + equity', tone: 'text-[#10b981]' },
    { label: 'Total Raised', value: '$92,400', meta: 'Across 180 backers', tone: 'text-[#60a5fa]' },
    { label: 'Open Pledges', value: '$12,600', meta: 'Pending close', tone: 'text-[#f59e0b]' },
    { label: 'Grant Match', value: '$18,000', meta: 'Match pipeline', tone: 'text-[#a855f7]' }
  ];

  campaigns = [
    { name: 'Community Studio Build', type: 'Crowdfund', goal: '$60,000', raised: '$42,000', status: 'Live' },
    { name: 'Equity Round - Seed', type: 'Equity', goal: '$120,000', raised: '$68,500', status: 'Open' },
    { name: 'Mutual Aid Pool', type: 'Donation', goal: '$15,000', raised: '$12,900', status: 'Final week' }
  ];

  investors = [
    { name: 'Lumen Ventures', amount: '$12,000', status: 'Committed' },
    { name: 'Northwind Collective', amount: '$6,500', status: 'Pending' },
    { name: 'Open Source Guild', amount: '$4,200', status: 'Matched' }
  ];
}
