import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-hub-restitution',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './hub-restitution.component.html',
  styleUrl: './hub-restitution.component.css',
  host: {
    class: 'block w-full'
  }
})
export class HubRestitutionComponent {
  summaryCards = [
    { label: 'Open Cases', value: '4', meta: 'Active mediation', tone: 'text-[#f59e0b]' },
    { label: 'Resolved', value: '12', meta: 'Last 12 months', tone: 'text-[#10b981]' },
    { label: 'Restitution Fund', value: '$14,600', meta: 'Available', tone: 'text-[#60a5fa]' },
    { label: 'Policies', value: '3', meta: 'Justice protocols', tone: 'text-[#a855f7]' }
  ];

  cases = [
    { title: 'Resource Access Dispute', status: 'Mediation', owner: 'Justice Council' },
    { title: 'Equity Allocation Appeal', status: 'Review', owner: 'Governance' },
    { title: 'Grant Compliance Issue', status: 'Resolution', owner: 'Treasury' }
  ];

  actions = [
    { step: 'Collect statements and evidence', status: 'In progress' },
    { step: 'Draft restitution plan', status: 'Pending review' },
    { step: 'Schedule community circle', status: 'Mar 23' }
  ];
}
