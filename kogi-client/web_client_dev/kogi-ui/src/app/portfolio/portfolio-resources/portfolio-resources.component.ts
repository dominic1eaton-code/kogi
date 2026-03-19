import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-portfolio-resources',
  standalone: true,
  imports: [CommonModule, RouterLink],
  templateUrl: './portfolio-resources.component.html',
  styleUrl: './portfolio-resources.component.css',
  host: {
    class: 'block w-full'
  }
})
export class PortfolioResourcesComponent {
  resourcePools = [
    {
      name: 'Labor Pool',
      type: 'Talent & Roles',
      meta: '42 contributors - 8 open roles',
      tone: 'text-[#10b981]'
    },
    {
      name: 'Capital Stack',
      type: 'Funding',
      meta: '$1.2M allocated - 3 campaigns live',
      tone: 'text-[#f59e0b]'
    },
    {
      name: 'Asset Inventory',
      type: 'Assets',
      meta: '128 assets - 14 depreciation flags',
      tone: 'text-[#60a5fa]'
    },
    {
      name: 'Knowledge Base',
      type: 'Artifacts & Research',
      meta: '312 artifacts - 68 citations',
      tone: 'text-[#8b5cf6]'
    },
    {
      name: 'Toolbox Library',
      type: 'Toolchains',
      meta: '24 toolkits - 7 active workflows',
      tone: 'text-[#22c55e]'
    }
  ];

  resourceShares = [
    {
      resource: 'Research Hub',
      target: 'Community Space',
      access: 'Contribute',
      expiry: 'Jun 30, 2026'
    },
    {
      resource: 'Design Assets Folder',
      target: 'Brand Team',
      access: 'Manage',
      expiry: 'No expiry'
    },
    {
      resource: 'Partner Contact Book',
      target: 'Growth Collective',
      access: 'Read',
      expiry: 'May 15, 2026'
    }
  ];

  governancePolicies = [
    { label: 'Share Policy', value: 'Role-based + attribution required' },
    { label: 'Review Workflow', value: 'Steward review -> merge' },
    { label: 'Contribution Ledger', value: 'Capital + labor attribution tracking' },
    { label: 'Access Levels', value: 'Read - Fork - Contribute - Manage' }
  ];
}

