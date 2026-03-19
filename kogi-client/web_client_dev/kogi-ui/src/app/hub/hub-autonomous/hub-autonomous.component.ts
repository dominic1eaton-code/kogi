import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-hub-autonomous',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './hub-autonomous.component.html',
  styleUrl: './hub-autonomous.component.css',
  host: {
    class: 'block w-full'
  }
})
export class HubAutonomousComponent {
  summaryCards = [
    { label: 'Autonomous Cells', value: '9', meta: 'Independent teams', tone: 'text-[#10b981]' },
    { label: 'Active Missions', value: '14', meta: 'Self-managed', tone: 'text-[#60a5fa]' },
    { label: 'Cell Budgets', value: '$48,500', meta: 'Allocated funds', tone: 'text-[#f59e0b]' },
    { label: 'Compliance', value: '90%', meta: 'Charter alignment', tone: 'text-[#a855f7]' }
  ];

  cells = [
    { name: 'Cell Sigma', focus: 'Infrastructure tooling', status: 'Active' },
    { name: 'Cell Aurora', focus: 'Community onboarding', status: 'Active' },
    { name: 'Cell Delta', focus: 'Research and policy', status: 'Review' }
  ];

  charters = [
    { name: 'Sigma Charter', status: 'Signed' },
    { name: 'Aurora Charter', status: 'Signed' },
    { name: 'Delta Charter', status: 'Draft' }
  ];
}
