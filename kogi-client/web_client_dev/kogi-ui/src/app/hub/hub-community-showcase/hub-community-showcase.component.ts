import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-hub-community-showcase',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './hub-community-showcase.component.html',
  styleUrl: './hub-community-showcase.component.css',
  host: {
    class: 'block w-full'
  }
})
export class HubCommunityShowcaseComponent {
  summaryCards = [
    { label: 'Showcases', value: '12', meta: 'Featured stories', tone: 'text-[#10b981]' },
    { label: 'Featured Orgs', value: '8', meta: 'Rotating highlights', tone: 'text-[#60a5fa]' },
    { label: 'Events', value: '5', meta: 'Upcoming', tone: 'text-[#f59e0b]' },
    { label: 'Contributions', value: '260', meta: 'Community wins', tone: 'text-[#a855f7]' }
  ];

  highlights = [
    { title: 'Community Media Lab', focus: 'Open access studio', status: 'Featured' },
    { title: 'Cooperative Launch Week', focus: 'Member onboarding', status: 'Live' },
    { title: 'Open Source Sprint', focus: 'Tooling release', status: 'Showcase' }
  ];

  events = [
    { name: 'Governance Town Hall', date: 'Mar 23', status: 'Scheduled' },
    { name: 'Community Demo Day', date: 'Mar 27', status: 'Open' },
    { name: 'Funding Showcase', date: 'Apr 2', status: 'Planned' }
  ];
}
