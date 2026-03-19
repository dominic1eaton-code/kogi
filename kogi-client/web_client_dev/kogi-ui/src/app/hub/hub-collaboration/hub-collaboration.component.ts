import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-hub-collaboration',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './hub-collaboration.component.html',
  styleUrl: './hub-collaboration.component.css',
  host: {
    class: 'block w-full'
  }
})
export class HubCollaborationComponent {
  summaryCards = [
    { label: 'Working Groups', value: '14', meta: 'Cross-org teams', tone: 'text-[#10b981]' },
    { label: 'Shared Projects', value: '9', meta: 'Active collaborations', tone: 'text-[#60a5fa]' },
    { label: 'Open Requests', value: '5', meta: 'Needs staffing', tone: 'text-[#f59e0b]' },
    { label: 'Partners', value: '22', meta: 'Verified orgs', tone: 'text-[#a855f7]' }
  ];

  groups = [
    { name: 'Open Source Council', focus: 'Release planning', status: 'Active' },
    { name: 'Community Care', focus: 'Mutual aid protocols', status: 'Active' },
    { name: 'Infrastructure Guild', focus: 'Shared tooling', status: 'Recruiting' }
  ];

  requests = [
    { title: 'Design System Sprint', need: '2 designers', status: 'Open' },
    { title: 'Governance Docs Audit', need: '1 researcher', status: 'Open' },
    { title: 'Data Migration', need: '1 engineer', status: 'Pending review' }
  ];
}
