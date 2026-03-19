import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-portfolio-content',
  standalone: true,
  imports: [CommonModule, RouterLink],
  templateUrl: './portfolio-content.component.html',
  styleUrl: './portfolio-content.component.css',
  host: {
    class: 'block w-full'
  }
})
export class PortfolioContentComponent {
  contentItems = [
    {
      name: 'Design System Library',
      type: 'Artifact',
      status: 'Published',
      meta: '120 components - 18 tokens',
      tone: 'text-[#8b5cf6]'
    },
    {
      name: 'Q3 Strategy Deck',
      type: 'Artifact',
      status: 'Approved',
      meta: '18 slides - exec summary',
      tone: 'text-[#60a5fa]'
    },
    {
      name: 'Brand Asset Pack',
      type: 'Asset',
      status: 'Active',
      meta: '52 exports - 4 formats',
      tone: 'text-[#f59e0b]'
    },
    {
      name: 'Research Dataset',
      type: 'Asset',
      status: 'Verified',
      meta: '3 data sources - 28k rows',
      tone: 'text-[#10b981]'
    }
  ];

  contentPipelines = [
    { label: 'Version Control', value: 'Semver, diff tracking, audit trail' },
    { label: 'Content Lifecycle', value: 'Draft -> Review -> Publish -> Archive' },
    { label: 'Distribution', value: 'Marketplace + subscriptions' },
    { label: 'Monetization', value: 'Free, paid, or subscription bundles' }
  ];
}
