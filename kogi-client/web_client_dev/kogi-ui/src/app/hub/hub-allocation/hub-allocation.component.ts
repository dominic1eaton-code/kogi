import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-hub-allocation',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './hub-allocation.component.html',
  styleUrl: './hub-allocation.component.css',
  host: {
    class: 'block w-full'
  }
})
export class HubAllocationComponent {
  summaryCards = [
    { label: 'Allocation Pool', value: '$64,000', meta: 'Quarterly budget', tone: 'text-[#10b981]' },
    { label: 'Programs Funded', value: '12', meta: 'Across 5 orgs', tone: 'text-[#60a5fa]' },
    { label: 'Requests', value: '8', meta: 'Awaiting vote', tone: 'text-[#f59e0b]' },
    { label: 'Utilization', value: '71%', meta: 'Budget used', tone: 'text-[#a855f7]' }
  ];

  allocations = [
    { program: 'Community Infrastructure', amount: '$18,000', owner: 'Operations', status: 'Approved' },
    { program: 'Open Source Fund', amount: '$12,000', owner: 'Guild Council', status: 'Voting' },
    { program: 'Education Access', amount: '$9,500', owner: 'Collective Care', status: 'Draft' },
    { program: 'Mutual Aid', amount: '$6,800', owner: 'Treasury', status: 'Approved' }
  ];

  forecasts = [
    { label: 'Reserve Target', value: '$22,000', status: 'On track' },
    { label: 'Next Allocation Cycle', value: 'Apr 10', status: 'Scheduled' },
    { label: 'Unused Budget', value: '$18,400', status: 'Available' }
  ];
}
