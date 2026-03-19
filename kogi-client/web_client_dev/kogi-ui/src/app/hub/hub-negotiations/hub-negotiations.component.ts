import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-hub-negotiations',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './hub-negotiations.component.html',
  styleUrl: './hub-negotiations.component.css',
  host: {
    class: 'block w-full'
  }
})
export class HubNegotiationsComponent {
  summaryCards = [
    { label: 'Active Talks', value: '5', meta: 'Cross-org deals', tone: 'text-[#10b981]' },
    { label: 'Term Sheets', value: '3', meta: 'Drafting', tone: 'text-[#60a5fa]' },
    { label: 'Mediation', value: '1', meta: 'Needs review', tone: 'text-[#f59e0b]' },
    { label: 'Closings', value: '2', meta: 'This month', tone: 'text-[#a855f7]' }
  ];

  negotiations = [
    { title: 'Resource Sharing Agreement', parties: 'Collective + Coop Alpha', status: 'Drafting' },
    { title: 'IP Licensing Deal', parties: 'Open Source Guild + Studio', status: 'Review' },
    { title: 'Service Exchange', parties: 'Federation + Cell Sigma', status: 'Negotiation' }
  ];

  checklist = [
    { step: 'Confirm allocation terms', status: 'In progress' },
    { step: 'Finalize governance clauses', status: 'Pending' },
    { step: 'Approve signature authority', status: 'Mar 24' }
  ];
}
