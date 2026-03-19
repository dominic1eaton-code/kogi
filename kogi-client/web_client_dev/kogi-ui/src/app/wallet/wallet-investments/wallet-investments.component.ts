import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-wallet-investments',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './wallet-investments.component.html',
  styleUrl: './wallet-investments.component.css',
  host: {
    class: 'block w-full'
  }
})
export class WalletInvestmentsComponent {
  summaryCards = [
    { label: 'Portfolio Value', value: '$184,600', meta: 'Long-term holdings', tone: 'text-[#10b981]' },
    { label: 'Gain / Loss', value: '+$12,400', meta: 'Year to date', tone: 'text-[#60a5fa]' },
    { label: 'Allocation', value: '62% Equity', meta: 'Balanced mix', tone: 'text-[#a855f7]' },
    { label: 'Liquidity', value: '$32,000', meta: 'Available to deploy', tone: 'text-[#f59e0b]' }
  ];

  allocations = [
    { label: 'Equities', value: '42%', tone: 'bg-[#10b981]' },
    { label: 'Funds', value: '18%', tone: 'bg-[#60a5fa]' },
    { label: 'Real Estate', value: '16%', tone: 'bg-[#f59e0b]' },
    { label: 'REITs', value: '12%', tone: 'bg-[#a855f7]' },
    { label: 'Crypto', value: '6%', tone: 'bg-[#f97316]' },
    { label: 'Cash', value: '6%', tone: 'bg-[#22c55e]' }
  ];

  holdings = [
    { name: 'Kogi Equity Fund', type: 'Fund', value: '$48,000', change: '+4.2%' },
    { name: 'Atlas REIT', type: 'REIT', value: '$28,600', change: '+2.1%' },
    { name: 'Solar Grid Notes', type: 'Private', value: '$18,400', change: '+7.5%' },
    { name: 'Blue River Shares', type: 'Equity', value: '$24,300', change: '-1.2%' }
  ];

  watchlist = [
    { name: 'Community Land Trust', status: 'Due diligence' },
    { name: 'Green Logistics Bond', status: 'Term sheet' },
    { name: 'Impact Crowdfund', status: 'Monitoring' }
  ];
}
