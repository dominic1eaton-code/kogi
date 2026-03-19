import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-wallet-benefits',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './wallet-benefits.component.html',
  styleUrl: './wallet-benefits.component.css',
  host: {
    class: 'block w-full'
  }
})
export class WalletBenefitsComponent {
  summaryCards = [
    { label: 'HSA Balance', value: '$3,200', meta: 'Annual goal $4,150', tone: 'text-[#06b6d4]' },
    { label: 'IRA Balance', value: '$42,800', meta: 'Contribution 74%', tone: 'text-[#10b981]' },
    { label: 'Portable Benefits', value: 'Active', meta: '4 programs', tone: 'text-[#a855f7]' },
    { label: 'Coverage', value: '98%', meta: 'Insurance current', tone: 'text-[#f59e0b]' }
  ];

  benefitAccounts = [
    { label: 'Solo 401(k)', progress: '79%', value: '$18,200 / $23,000', tone: 'bg-[#10b981]' },
    { label: 'HSA', progress: '58%', value: '$2,400 / $4,150', tone: 'bg-[#06b6d4]' },
    { label: 'IRA', progress: '74%', value: '$4,800 / $6,500', tone: 'bg-[#a855f7]' },
    { label: 'Health Coverage', progress: '100%', value: 'Active through Mar 2027', tone: 'bg-[#f59e0b]' }
  ];

  benefitPrograms = [
    { name: 'Portable PTO', status: '24 hours accrued' },
    { name: 'Education Stipend', status: '$1,200 remaining' },
    { name: 'Wellness Fund', status: 'Quarterly reload' }
  ];
}
