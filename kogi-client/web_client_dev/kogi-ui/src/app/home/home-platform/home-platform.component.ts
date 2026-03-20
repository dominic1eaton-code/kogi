import { Component } from '@angular/core';
import { NgFor } from '@angular/common';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-home-platform',
  standalone: true,
  imports: [NgFor, RouterLink],
  templateUrl: './home-platform.component.html',
  styleUrl: './home-platform.component.css'
})
export class HomePlatformComponent {
  modules = [
    {
      name: 'Portfolio OS',
      title: 'Work portfolios with living structure',
      description: 'Organize programs, projects, resources, and assets in one graph of work.',
      highlights: ['Programs + projects + assets', 'Itembooks, playbooks, notebooks', 'Portfolio analytics + health'],
      barClass: 'bg-gradient-to-r from-[#10b981] to-[#059669]',
      accent: 'text-[#10b981]'
    },
    {
      name: 'Office',
      title: 'Work boards, schedules, and studio space',
      description: 'Plan tasks, manage timelines, and keep production moving with shared workspaces.',
      highlights: ['Boards + calendars + gantt', 'Studio for ideas + prototypes', 'CRM + operations tools'],
      barClass: 'bg-gradient-to-r from-[#38bdf8] to-[#0ea5e9]',
      accent: 'text-[#38bdf8]'
    },
    {
      name: 'Marketplace',
      title: 'Find work, talent, and resources',
      description: 'Post listings, match collaborators, and fund campaigns without switching apps.',
      highlights: ['Gigs, contracts, campaigns', 'Talent matching + reviews', 'Built-in escrow'],
      barClass: 'bg-gradient-to-r from-[#f59e0b] to-[#f97316]',
      accent: 'text-[#f59e0b]'
    },
    {
      name: 'Exchange',
      title: 'Trade resources and assets',
      description: 'Manage bids, offers, and exchanges across portfolios and marketplaces.',
      highlights: ['Bids + deals + proposals', 'Asset + resource exchange', 'Settlement workflows'],
      barClass: 'bg-gradient-to-r from-[#8b5cf6] to-[#a855f7]',
      accent: 'text-[#8b5cf6]'
    },
    {
      name: 'Kogi Bank',
      title: 'Finance built for independent work',
      description: 'Smart accounts for taxes, benefits, savings, and cooperative treasuries.',
      highlights: ['Automated allocation rules', 'Portable benefits', 'Shared treasury pools'],
      barClass: 'bg-gradient-to-r from-[#facc15] to-[#f59e0b]',
      accent: 'text-[#facc15]'
    },
    {
      name: 'Community Spaces',
      title: 'Rooms, channels, and feeds',
      description: 'Build community spaces for collaboration, events, and resource sharing.',
      highlights: ['Spaces + rooms + chats', 'Feeds + timelines', 'Moderation + roles'],
      barClass: 'bg-gradient-to-r from-[#22c55e] to-[#10b981]',
      accent: 'text-[#22c55e]'
    },
    {
      name: 'Governance Hub',
      title: 'Decision making with real execution',
      description: 'Run proposals, voting, and distribution for cooperative organizations.',
      highlights: ['Proposal + voting flows', 'Treasury transparency', 'Allocation rules'],
      barClass: 'bg-gradient-to-r from-[#f97316] to-[#f43f5e]',
      accent: 'text-[#f97316]'
    },
    {
      name: 'Developer + Integrations',
      title: 'APIs, SDKs, and tool networks',
      description: 'Connect external tools, providers, and workflows into the platform.',
      highlights: ['API + SDK access', 'Provider registry', 'Tool orchestration'],
      barClass: 'bg-gradient-to-r from-[#94a3b8] to-[#64748b]',
      accent: 'text-[#94a3b8]'
    }
  ];
}
