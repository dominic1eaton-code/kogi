import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-portfolio-guidebook',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './portfolio-guidebook.component.html',
  styleUrl: './portfolio-guidebook.component.css',
  host: {
    class: 'block w-full'
  }
})
export class PortfolioGuidebookComponent {
  chapters = [
    { title: 'Portfolio Foundations', summary: 'Define scope, mission, and ownership.' },
    { title: 'ItemBook Operations', summary: 'Maintain charters, catalogues, and schedules.' },
    { title: 'Governance & Compliance', summary: 'Review cycles, approvals, and policy sets.' },
    { title: 'Collaboration Playbooks', summary: 'Shared portfolios and crowdresourcing.' }
  ];

  steps = [
    'Create the portfolio item and assign owners.',
    'Initialize ItemBook charter and workspace.',
    'Configure registry columns and views.',
    'Set governance rules and review cadence.'
  ];
}
