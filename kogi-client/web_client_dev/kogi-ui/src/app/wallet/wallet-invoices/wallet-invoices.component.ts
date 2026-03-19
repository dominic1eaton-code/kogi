import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-wallet-invoices',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './wallet-invoices.component.html',
  styleUrl: './wallet-invoices.component.css',
  host: {
    class: 'block w-full'
  }
})
export class WalletInvoicesComponent {
  summaryCards = [
    { label: 'Receivable', value: '$28,400', meta: '12 open invoices', tone: 'text-[#10b981]' },
    { label: 'Payable', value: '$6,900', meta: '5 bills', tone: 'text-[#ef4444]' },
    { label: 'Overdue', value: '$1,200', meta: '1 invoice', tone: 'text-[#f59e0b]' },
    { label: 'Drafts', value: '3', meta: 'Need approval', tone: 'text-[#60a5fa]' }
  ];

  invoices = [
    { id: '#091', client: 'Venture Partners', due: 'Mar 25', amount: '$12,000', status: 'Sent' },
    { id: '#090', client: 'Pixel Studio', due: 'Mar 22', amount: '$3,400', status: 'Viewed' },
    { id: '#089', client: 'Cloudflare', due: 'Mar 20', amount: '$200', status: 'Scheduled' },
    { id: '#088', client: 'Collective Studio', due: 'Mar 18', amount: '$1,800', status: 'Overdue' }
  ];

  templates = [
    { name: 'Retainer Invoice', cadence: 'Monthly', status: 'Active' },
    { name: 'Escrow Release', cadence: 'Milestone', status: 'Active' },
    { name: 'Grant Disbursement', cadence: 'Quarterly', status: 'Paused' }
  ];
}
