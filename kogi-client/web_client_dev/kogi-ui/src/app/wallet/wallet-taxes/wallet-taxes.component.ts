import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-wallet-taxes',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './wallet-taxes.component.html',
  styleUrl: './wallet-taxes.component.css',
  host: {
    class: 'block w-full'
  }
})
export class WalletTaxesComponent {
  summaryCards = [
    { label: 'Tax Reserve', value: '$6,900', meta: '24% rate', tone: 'text-[#ef4444]' },
    { label: 'Estimated Due', value: '$2,100', meta: 'Next payment', tone: 'text-[#f59e0b]' },
    { label: 'Withholding', value: '$1,400', meta: 'Auto-sweep', tone: 'text-[#60a5fa]' },
    { label: 'Compliance', value: 'On track', meta: 'Q1 filings', tone: 'text-[#10b981]' }
  ];

  calendar = [
    { date: 'Apr 15', item: 'Estimated tax payment', status: 'Upcoming' },
    { date: 'May 15', item: 'Sales tax remittance', status: 'Scheduled' },
    { date: 'Jun 30', item: 'Quarterly filing', status: 'Draft' }
  ];

  filings = [
    { type: 'Federal', period: 'Q1 2026', status: 'In prep' },
    { type: 'State', period: 'Q1 2026', status: 'Not started' },
    { type: 'Local', period: 'Monthly', status: 'Current' }
  ];

  insights = [
    { label: 'Projected Liability', value: '$8,600', note: 'Based on YTD income' },
    { label: 'Safe Harbor', value: '92%', note: 'Target 100%' },
    { label: 'Expense Coverage', value: '$4,200', note: 'Eligible deductions' }
  ];
}
