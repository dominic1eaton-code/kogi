import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-hub-cooperatives',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './hub-cooperatives.component.html',
  styleUrl: './hub-cooperatives.component.css',
  host: {
    class: 'block w-full'
  }
})
export class HubCooperativesComponent {
  summaryCards = [
    { label: 'Cooperatives', value: '6', meta: 'Member owned', tone: 'text-[#10b981]' },
    { label: 'Member Owners', value: '72', meta: 'Active members', tone: 'text-[#60a5fa]' },
    { label: 'Dividends', value: '$12,400', meta: 'Next distribution', tone: 'text-[#f59e0b]' },
    { label: 'Governance', value: '100%', meta: 'Charter compliant', tone: 'text-[#a855f7]' }
  ];

  coops = [
    { name: 'Coop Alpha', focus: 'Production', status: 'Active' },
    { name: 'Coop Beta', focus: 'Research', status: 'Active' },
    { name: 'Coop Gamma', focus: 'Community services', status: 'Onboarding' }
  ];

  dividends = [
    { cycle: 'Q1 2026', amount: '$6,800', status: 'Scheduled' },
    { cycle: 'Q4 2025', amount: '$5,600', status: 'Paid' }
  ];
}
