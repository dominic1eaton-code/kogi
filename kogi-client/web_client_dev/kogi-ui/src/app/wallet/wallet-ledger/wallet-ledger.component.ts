import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-wallet-ledger',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './wallet-ledger.component.html',
  styleUrl: './wallet-ledger.component.css',
  host: {
    class: 'block w-full'
  }
})
export class WalletLedgerComponent {
  summaryCards = [
    { label: 'Credits', value: '$42,800', meta: '30 days', tone: 'text-[#10b981]' },
    { label: 'Debits', value: '$11,460', meta: '30 days', tone: 'text-[#ef4444]' },
    { label: 'Net Flow', value: '$31,340', meta: 'Net positive', tone: 'text-[#60a5fa]' },
    { label: 'Reconciled', value: '92%', meta: '12 items open', tone: 'text-[#f59e0b]' }
  ];

  ledgerEntries = [
    { date: 'Mar 18', desc: 'Retainer Invoice #084', account: 'Checking', amount: '+$12,000', status: 'Cleared' },
    { date: 'Mar 17', desc: 'Cloud Hosting', account: 'Ops', amount: '-$420', status: 'Cleared' },
    { date: 'Mar 16', desc: 'Escrow Release - Deal 44', account: 'Escrow', amount: '+$4,200', status: 'Pending' },
    { date: 'Mar 16', desc: 'Payroll Run', account: 'Ops', amount: '-$8,900', status: 'Scheduled' },
    { date: 'Mar 15', desc: 'Dividend Deposit', account: 'Investment', amount: '+$1,100', status: 'Cleared' }
  ];

  reconciliationTasks = [
    { label: 'Match 3 receipts with ops charges', status: 'Due Mar 22' },
    { label: 'Confirm escrow release schedule', status: 'Due Mar 21' },
    { label: 'Sync bank feed - Capital Reserve', status: 'Waiting on bank' }
  ];
}
