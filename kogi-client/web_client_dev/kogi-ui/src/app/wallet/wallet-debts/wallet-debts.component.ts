import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-wallet-debts',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './wallet-debts.component.html',
  styleUrl: './wallet-debts.component.css',
  host: {
    class: 'block w-full'
  }
})
export class WalletDebtsComponent {
  summaryCards = [
    { label: 'Total Debt', value: '$48,600', meta: 'Across 5 obligations', tone: 'text-[#ef4444]' },
    { label: 'Monthly Payments', value: '$3,420', meta: 'Auto-pay enabled', tone: 'text-[#f59e0b]' },
    { label: 'Interest Rate', value: '6.2%', meta: 'Weighted average', tone: 'text-[#60a5fa]' },
    { label: 'Payoff ETA', value: '18 mo', meta: 'Current schedule', tone: 'text-[#10b981]' }
  ];

  obligations = [
    { name: 'Studio Credit Line', balance: '$18,200', payment: '$1,200', rate: '5.4%', status: 'Current' },
    { name: 'Equipment Lease', balance: '$9,600', payment: '$620', rate: '7.1%', status: 'Current' },
    { name: 'Bridge Loan', balance: '$12,800', payment: '$940', rate: '6.8%', status: 'Auto-pay' },
    { name: 'Vendor Payable', balance: '$8,000', payment: '$660', rate: '0%', status: 'Negotiating' }
  ];

  schedule = [
    { date: 'Mar 20', item: 'Studio Credit Line', amount: '$1,200' },
    { date: 'Mar 22', item: 'Equipment Lease', amount: '$620' },
    { date: 'Mar 25', item: 'Bridge Loan', amount: '$940' },
    { date: 'Mar 28', item: 'Vendor Payable', amount: '$660' }
  ];

  strategies = [
    { label: 'Snowball Plan', desc: 'Prioritize smallest balances to reduce accounts quickly.' },
    { label: 'Avalanche Plan', desc: 'Target highest interest first to reduce total cost.' },
    { label: 'Refinance Review', desc: 'Evaluate lower rate offers for the credit line.' }
  ];
}
