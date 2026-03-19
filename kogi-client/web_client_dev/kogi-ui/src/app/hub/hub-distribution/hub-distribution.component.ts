import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-hub-distribution',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './hub-distribution.component.html',
  styleUrl: './hub-distribution.component.css',
  host: {
    class: 'block w-full'
  }
})
export class HubDistributionComponent {
  summaryCards = [
    { label: 'Total Distribution', value: '$36,200', meta: 'Current cycle', tone: 'text-[#10b981]' },
    { label: 'Recipients', value: '28', meta: 'Teams and members', tone: 'text-[#60a5fa]' },
    { label: 'Pending', value: '$6,800', meta: 'Awaiting approvals', tone: 'text-[#f59e0b]' },
    { label: 'Next Run', value: 'Mar 24', meta: 'Biweekly payout', tone: 'text-[#a855f7]' }
  ];

  payouts = [
    { name: 'Open Source Guild', amount: '$8,400', status: 'Approved' },
    { name: 'Coop Team Alpha', amount: '$5,200', status: 'Scheduled' },
    { name: 'Mutual Aid Pool', amount: '$3,600', status: 'Pending vote' },
    { name: 'Research Cell', amount: '$2,400', status: 'Approved' }
  ];

  rules = [
    { rule: '40% to Operations', detail: 'Covers core infrastructure' },
    { rule: '25% to Community Dividends', detail: 'Member payouts' },
    { rule: '20% to Innovation', detail: 'R and D funds' },
    { rule: '15% to Mutual Aid', detail: 'Care and emergency' }
  ];
}
