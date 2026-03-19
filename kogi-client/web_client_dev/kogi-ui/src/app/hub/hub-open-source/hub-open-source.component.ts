import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-hub-open-source',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './hub-open-source.component.html',
  styleUrl: './hub-open-source.component.css',
  host: {
    class: 'block w-full'
  }
})
export class HubOpenSourceComponent {
  summaryCards = [
    { label: 'Projects', value: '22', meta: 'Open source repos', tone: 'text-[#10b981]' },
    { label: 'Contributors', value: '148', meta: 'Active this quarter', tone: 'text-[#60a5fa]' },
    { label: 'Releases', value: '7', meta: 'This month', tone: 'text-[#f59e0b]' },
    { label: 'Funding', value: '$18,200', meta: 'Sponsor pool', tone: 'text-[#a855f7]' }
  ];

  projects = [
    { name: 'Kogi UI System', status: 'Maintained', owner: 'Design Guild' },
    { name: 'Governance Toolkit', status: 'Release candidate', owner: 'Ops Guild' },
    { name: 'Community API', status: 'Active', owner: 'Engineering' }
  ];

  sponsors = [
    { name: 'Open Collective', amount: '$6,400', status: 'Active' },
    { name: 'Local Funders', amount: '$4,800', status: 'Pending' },
    { name: 'Community Grants', amount: '$2,200', status: 'Approved' }
  ];
}
