import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-hub-resource-crowdfund',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './hub-resource-crowdfund.component.html',
  styleUrl: './hub-resource-crowdfund.component.css',
  host: {
    class: 'block w-full'
  }
})
export class HubResourceCrowdfundComponent {
  summaryCards = [
    { label: 'Campaigns', value: '5', meta: 'Resource drives', tone: 'text-[#10b981]' },
    { label: 'Raised', value: '$42,600', meta: 'This quarter', tone: 'text-[#60a5fa]' },
    { label: 'Backers', value: '240', meta: 'Community support', tone: 'text-[#f59e0b]' },
    { label: 'Match Pool', value: '$8,000', meta: 'Available match', tone: 'text-[#a855f7]' }
  ];

  campaigns = [
    { name: 'Community Equipment Fund', goal: '$20,000', raised: '$14,200', status: 'Live' },
    { name: 'Open Source Maintenance', goal: '$12,000', raised: '$9,600', status: 'Live' },
    { name: 'Care Relief Sprint', goal: '$6,500', raised: '$5,300', status: 'Closing' }
  ];

  needs = [
    { item: 'Audio gear for media lab', status: 'Requested' },
    { item: 'Accessibility upgrades', status: 'Reviewed' },
    { item: 'Community travel fund', status: 'New' }
  ];
}
