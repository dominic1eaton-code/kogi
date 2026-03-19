import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-hub-teams',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './hub-teams.component.html',
  styleUrl: './hub-teams.component.css',
  host: {
    class: 'block w-full'
  }
})
export class HubTeamsComponent {
  summaryCards = [
    { label: 'Teams', value: '26', meta: 'Across orgs', tone: 'text-[#10b981]' },
    { label: 'Active Members', value: '140', meta: 'Verified profiles', tone: 'text-[#60a5fa]' },
    { label: 'Open Roles', value: '18', meta: 'Seeking talent', tone: 'text-[#f59e0b]' },
    { label: 'Pods', value: '9', meta: 'Autonomous cells', tone: 'text-[#a855f7]' }
  ];

  teams = [
    { name: 'Design Collective', focus: 'Brand systems', status: 'Active' },
    { name: 'Operations Pod', focus: 'Finance + compliance', status: 'Active' },
    { name: 'Community Care', focus: 'Support + onboarding', status: 'Recruiting' }
  ];

  roles = [
    { role: 'Governance Facilitator', team: 'Steward Circle', status: 'Open' },
    { role: 'Data Steward', team: 'Infrastructure Guild', status: 'Review' },
    { role: 'Product Lead', team: 'Open Source Guild', status: 'Open' }
  ];
}
