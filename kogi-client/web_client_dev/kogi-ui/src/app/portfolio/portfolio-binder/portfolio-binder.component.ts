import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-portfolio-binder',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './portfolio-binder.component.html',
  styleUrl: './portfolio-binder.component.css',
  host: {
    class: 'block w-full'
  }
})
export class PortfolioBinderComponent {
  binderMeta = [
    { label: 'Binder Type', value: 'Portfolio Binder' },
    { label: 'Coverage', value: '91%' },
    { label: 'Owner', value: 'Platform Team' },
    { label: 'Last Review', value: 'Mar 14, 2026' }
  ];

  binderSections = [
    {
      title: 'Strategy & Vision',
      entries: ['Mission & Vision', 'Portfolio Roadmap', 'Quarterly OKRs']
    },
    {
      title: 'Programs & Projects',
      entries: ['Community Growth Program', 'Brand Identity System', 'Portfolio Engine v2']
    },
    {
      title: 'Assets & Artifacts',
      entries: ['Design System Library', 'Q3 Strategy Deck', 'Brand Asset Pack']
    }
  ];

  binderNotes = [
    'Binders group items across portfolios and provide coverage analytics.',
    'Binder coverage feeds the PortfolioHealth model and governance checks.',
    'Binder views support tags, filters, and per-section permissions.'
  ];
}
