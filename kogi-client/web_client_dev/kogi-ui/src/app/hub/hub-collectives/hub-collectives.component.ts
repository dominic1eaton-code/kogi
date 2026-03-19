import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-hub-collectives',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './hub-collectives.component.html',
  styleUrl: './hub-collectives.component.css',
  host: {
    class: 'block w-full'
  }
})
export class HubCollectivesComponent {
  summaryCards = [
    { label: 'Collectives', value: '11', meta: 'Active groups', tone: 'text-[#10b981]' },
    { label: 'Members', value: '84', meta: 'Cross-org', tone: 'text-[#60a5fa]' },
    { label: 'Shared Funds', value: '$22,600', meta: 'Treasury pools', tone: 'text-[#f59e0b]' },
    { label: 'Projects', value: '19', meta: 'Open initiatives', tone: 'text-[#a855f7]' }
  ];

  collectives = [
    { name: 'Design Commons', focus: 'Shared design systems', status: 'Active' },
    { name: 'Research Collective', focus: 'Policy and strategy', status: 'Active' },
    { name: 'Community Care', focus: 'Mutual aid', status: 'Recruiting' }
  ];

  charters = [
    { name: 'Design Commons Charter', status: 'Signed' },
    { name: 'Research Collective Charter', status: 'Draft' },
    { name: 'Care Circle Charter', status: 'Review' }
  ];
}
