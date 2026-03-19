import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-portfolio-workspace',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './portfolio-workspace.component.html',
  styleUrl: './portfolio-workspace.component.css',
  host: {
    class: 'block w-full'
  }
})
export class PortfolioWorkspaceComponent {
  workstreams = [
    { name: 'Strategy Sprint', status: 'Active', owner: 'J. Davis' },
    { name: 'Design Ops', status: 'Active', owner: 'M. Perez' },
    { name: 'Governance Review', status: 'Upcoming', owner: 'Ops Council' }
  ];

  tasks = [
    'Finalize Q2 portfolio roadmap',
    'Audit shared portfolio contributions',
    'Publish resource share policy update'
  ];
}
