import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-wallet-escrow',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './wallet-escrow.component.html',
  styleUrl: './wallet-escrow.component.css',
  host: {
    class: 'block w-full'
  }
})
export class WalletEscrowComponent {
  summaryCards = [
    { label: 'Active Escrows', value: '6', meta: '2 pending release', tone: 'text-[#f59e0b]' },
    { label: 'Locked Funds', value: '$24,600', meta: 'Across 4 deals', tone: 'text-[#f97316]' },
    { label: 'Pending Release', value: '$6,200', meta: 'Awaiting approval', tone: 'text-[#60a5fa]' },
    { label: 'Disputes', value: '1', meta: 'Needs mediation', tone: 'text-[#ef4444]' }
  ];

  escrowDeals = [
    { title: 'Brand Identity System', client: 'Acme Corp', amount: '$4,200', status: 'Locked', milestone: 'Design review' },
    { title: 'Data Pipeline Audit', client: 'DataStream Co', amount: '$2,800', status: 'Pending', milestone: 'Final report' },
    { title: 'Mobile App MVP', client: 'NovaTech', amount: '$7,500', status: 'Release Ready', milestone: 'QA signoff' },
    { title: 'DAO Tooling Sprint', client: 'Collective Studio', amount: '$10,100', status: 'Milestone 2', milestone: 'Sprint demo' }
  ];

  releaseSchedule = [
    { date: 'Mar 20', event: 'Release - Mobile App MVP', amount: '$3,000' },
    { date: 'Mar 22', event: 'Release - Brand Identity System', amount: '$1,200' },
    { date: 'Mar 25', event: 'Release - DAO Tooling Sprint', amount: '$4,000' }
  ];
}
