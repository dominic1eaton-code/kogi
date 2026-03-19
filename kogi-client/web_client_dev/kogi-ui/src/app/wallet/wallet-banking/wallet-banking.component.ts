import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-wallet-banking',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './wallet-banking.component.html',
  styleUrl: './wallet-banking.component.css',
  host: {
    class: 'block w-full'
  }
})
export class WalletBankingComponent {
  summaryCards = [
    { label: 'Total Balance', value: '$214,880', meta: 'Across 6 accounts', tone: 'text-[#10b981]' },
    { label: 'Liquidity', value: '$92,400', meta: '30 day runway', tone: 'text-[#60a5fa]' },
    { label: 'Available Credit', value: '$48,000', meta: '3 active lines', tone: 'text-[#f59e0b]' },
    { label: 'Reserves', value: '$24,900', meta: 'Emergency fund', tone: 'text-[#a855f7]' }
  ];

  connections = [
    { name: 'Kogi Treasury', type: 'Primary Checking', status: 'Connected', balance: '$48,200' },
    { name: 'Ops Sweep', type: 'Sweep Account', status: 'Auto-sync', balance: '$21,900' },
    { name: 'Capital Reserve', type: 'Savings Vault', status: 'Connected', balance: '$74,800' },
    { name: 'Escrow Float', type: 'Escrow Holdings', status: 'Linked', balance: '$18,300' }
  ];

  transferQueue = [
    { name: 'Payroll Run', direction: 'Outbound', amount: '$18,200', status: 'Scheduled', date: 'Mar 22' },
    { name: 'Client Retainer', direction: 'Inbound', amount: '$12,500', status: 'Pending', date: 'Mar 21' },
    { name: 'Investment Sweep', direction: 'Outbound', amount: '$9,000', status: 'Queued', date: 'Mar 20' },
    { name: 'Tax Reserve', direction: 'Outbound', amount: '$4,200', status: 'Approved', date: 'Mar 20' }
  ];

  cashTools = [
    { label: 'Auto-Split Rules', desc: 'Distribute inflows across ops, tax, and investment accounts.' },
    { label: 'Liquidity Lanes', desc: 'Define minimum cash levels per portfolio and project.' },
    { label: 'Card Controls', desc: 'Set spend limits, merchant locks, and approval flows.' },
    { label: 'Treasury Forecast', desc: 'Plan 90 day inflows, outflows, and reserves.' }
  ];
}
