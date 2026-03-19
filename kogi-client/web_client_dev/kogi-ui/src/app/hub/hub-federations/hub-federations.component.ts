import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-hub-federations',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './hub-federations.component.html',
  styleUrl: './hub-federations.component.css',
  host: {
    class: 'block w-full'
  }
})
export class HubFederationsComponent {
  summaryCards = [
    { label: 'Federations', value: '4', meta: 'Multi-org networks', tone: 'text-[#10b981]' },
    { label: 'Member Orgs', value: '21', meta: 'Active nodes', tone: 'text-[#60a5fa]' },
    { label: 'Shared Assets', value: '$92,000', meta: 'Federated pools', tone: 'text-[#f59e0b]' },
    { label: 'Agreements', value: '9', meta: 'Active MOUs', tone: 'text-[#a855f7]' }
  ];

  federations = [
    { name: 'North Cluster', focus: 'Infrastructure sharing', status: 'Active' },
    { name: 'Open Knowledge Network', focus: 'Research exchange', status: 'Active' },
    { name: 'Care Alliance', focus: 'Mutual aid', status: 'Review' }
  ];

  agreements = [
    { title: 'Shared Services MOU', status: 'Signed' },
    { title: 'Resource Exchange Protocol', status: 'Draft' },
    { title: 'Data Governance Policy', status: 'Review' }
  ];
}
