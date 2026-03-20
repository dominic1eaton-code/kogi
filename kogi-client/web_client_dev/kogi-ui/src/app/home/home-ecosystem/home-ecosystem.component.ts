import { Component } from '@angular/core';
import { NgFor } from '@angular/common';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-home-ecosystem',
  standalone: true,
  imports: [NgFor, RouterLink],
  templateUrl: './home-ecosystem.component.html',
  styleUrl: './home-ecosystem.component.css'
})
export class HomeEcosystemComponent {
  initiatives = [
    {
      title: 'Portable Benefits',
      description: 'Health, retirement, income protection, and professional development accounts tied to work.',
      barClass: 'bg-gradient-to-r from-[#10b981] to-[#059669]',
      accent: 'text-[#10b981]'
    },
    {
      title: 'Group Economics',
      description: 'Shared treasuries, revenue sharing pools, and cooperative finance tools.',
      barClass: 'bg-gradient-to-r from-[#f59e0b] to-[#f97316]',
      accent: 'text-[#f59e0b]'
    },
    {
      title: 'Grants + Microfinancing',
      description: 'Application workflows, funding dashboards, and impact reporting for member-led funding.',
      barClass: 'bg-gradient-to-r from-[#38bdf8] to-[#0ea5e9]',
      accent: 'text-[#38bdf8]'
    },
    {
      title: 'Equity Crowdfunding',
      description: 'Campaign management, investor tracking, and cap table tools for collective growth.',
      barClass: 'bg-gradient-to-r from-[#8b5cf6] to-[#a855f7]',
      accent: 'text-[#8b5cf6]'
    },
    {
      title: 'Crowdresourcing',
      description: 'Coordinate resources, labor, and contributions across shared initiatives.',
      barClass: 'bg-gradient-to-r from-[#22c55e] to-[#10b981]',
      accent: 'text-[#22c55e]'
    },
    {
      title: 'Shared Portfolios',
      description: 'Collaborative portfolios that track group programs, projects, and assets together.',
      barClass: 'bg-gradient-to-r from-[#f97316] to-[#f43f5e]',
      accent: 'text-[#f97316]'
    }
  ];

  orgTypes = ['Autonomous orgs', 'Collectives', 'Cooperatives', 'Federations', 'Teams', 'Communities'];

  instruments = [
    {
      title: 'Cooperative Treasury',
      description: 'Shared operating accounts with multi-signature governance controls.'
    },
    {
      title: 'Revenue Sharing Pool',
      description: 'Automatic distribution rules for cooperative earnings and royalties.'
    },
    {
      title: 'Group Investment Pool',
      description: 'Member contributions managed with transparent governance and reporting.'
    },
    {
      title: 'Mutual Aid Fund',
      description: 'Emergency support with documented contributions and approvals.'
    },
    {
      title: 'Collective Escrow',
      description: 'Hold and release funds based on team approvals and delivery checks.'
    },
    {
      title: 'Crowdresourcing Pool',
      description: 'Track contributions of labor, resources, and benefits across shared initiatives.'
    }
  ];
}
