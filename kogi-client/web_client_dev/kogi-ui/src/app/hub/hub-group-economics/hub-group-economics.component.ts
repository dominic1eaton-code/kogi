import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-hub-group-economics',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './hub-group-economics.component.html',
  styleUrl: './hub-group-economics.component.css',
  host: {
    class: 'block w-full'
  }
})
export class HubGroupEconomicsComponent {
  summaryCards = [
    { label: 'Group Treasury', value: '$148,200', meta: 'Federated funds', tone: 'text-[#10b981]' },
    { label: 'Economic Policies', value: '8', meta: 'Active rulesets', tone: 'text-[#60a5fa]' },
    { label: 'Distribution Cycle', value: 'Monthly', meta: 'Next run Apr 1', tone: 'text-[#f59e0b]' },
    { label: 'Mutual Aid', value: '$18,400', meta: 'Reserve', tone: 'text-[#a855f7]' }
  ];

  economics = [
    { item: 'Operations Allocation', amount: '$48,000', status: 'Active' },
    { item: 'Community Dividend', amount: '$26,000', status: 'Active' },
    { item: 'Innovation Pool', amount: '$18,500', status: 'Review' }
  ];

  policies = [
    { name: 'Solidarity Distribution', status: 'Approved' },
    { name: 'Mutual Aid Escrow', status: 'Active' },
    { name: 'Shared Asset Policy', status: 'Draft' }
  ];
}
