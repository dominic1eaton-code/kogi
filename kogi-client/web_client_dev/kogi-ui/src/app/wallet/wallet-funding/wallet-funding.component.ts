import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-wallet-funding',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './wallet-funding.component.html',
  styleUrl: './wallet-funding.component.css',
  host: {
    class: 'block w-full'
  }
})
export class WalletFundingComponent {
  summaryCards = [
    { label: 'Equity Issued', value: '72%', meta: 'Cap table utilization', tone: 'text-[#10b981]' },
    { label: 'Royalty Streams', value: '$6,400', meta: 'Monthly forecast', tone: 'text-[#60a5fa]' },
    { label: 'Dividend Pool', value: '$18,000', meta: 'Next payout', tone: 'text-[#f59e0b]' },
    { label: 'Liquidity Events', value: '2', meta: 'IPO / ICO tracking', tone: 'text-[#a855f7]' }
  ];

  capTable = [
    { holder: 'Founder Pool', shares: '420,000', stake: '42%', vesting: '4y cliff 1y' },
    { holder: 'Community Equity', shares: '220,000', stake: '22%', vesting: 'Mission based' },
    { holder: 'Investors Series A', shares: '180,000', stake: '18%', vesting: 'Preferred' },
    { holder: 'Advisors', shares: '60,000', stake: '6%', vesting: '18 mo' }
  ];

  distributions = [
    { stream: 'Product Royalties', amount: '$2,400', cadence: 'Monthly' },
    { stream: 'License Fees', amount: '$1,600', cadence: 'Quarterly' },
    { stream: 'Dividend Reserve', amount: '$18,000', cadence: 'Semiannual' }
  ];

  liquidity = [
    { event: 'IPO Prep', status: 'Due diligence', owner: 'Finance Council' },
    { event: 'Community ICO', status: 'Tokenomics review', owner: 'Governance' },
    { event: 'Secondary Sale', status: 'Draft terms', owner: 'Legal' }
  ];

  assets = [
    { name: 'Estate Holding Trust', type: 'Trust', value: '$140,000' },
    { name: 'Kogi Studio HQ', type: 'Real Estate', value: '$320,000' },
    { name: 'Heritage Reserve', type: 'Estate', value: '$90,000' }
  ];
}
