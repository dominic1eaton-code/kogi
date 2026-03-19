import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-hub-organizations',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './hub-organizations.component.html',
  styleUrl: './hub-organizations.component.css',
  host: {
    class: 'block w-full'
  }
})
export class HubOrganizationsComponent {
  summaryCards = [
    { label: 'Organizations', value: '32', meta: 'Verified entities', tone: 'text-[#10b981]' },
    { label: 'Federations', value: '4', meta: 'Shared governance', tone: 'text-[#60a5fa]' },
    { label: 'Charters', value: '28', meta: 'Active charters', tone: 'text-[#f59e0b]' },
    { label: 'Compliance', value: '94%', meta: 'On track', tone: 'text-[#a855f7]' }
  ];

  organizations = [
    { name: 'Kogi Studios', type: 'Studio', status: 'Active' },
    { name: 'Open Source Guild', type: 'Guild', status: 'Active' },
    { name: 'Community Trust', type: 'Trust', status: 'Review' },
    { name: 'Coop Alpha', type: 'Cooperative', status: 'Active' }
  ];

  compliance = [
    { item: 'Charter renewal - Community Trust', status: 'Due Apr 4' },
    { item: 'Policy audit - Kogi Studios', status: 'In progress' },
    { item: 'Federation review - North Cluster', status: 'Scheduled' }
  ];
}
