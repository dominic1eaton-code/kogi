import { CommonModule } from '@angular/common';
import { Component, OnDestroy, OnInit, inject } from '@angular/core';
import { ActivatedRoute, RouterLink } from '@angular/router';
import { Subscription } from 'rxjs';

type ViewMode = 'grid' | 'list' | 'tree' | 'board';

type PortfolioCard = {
  name: string;
  type: string;
  status: string;
  summary: string;
  metricLabel: string;
  metricValue: string;
  secondaryLabel: string;
  secondaryValue: string;
  progress: number;
  tone: string;
  tags: string[];
  owners: string[];
};

type BoardColumn = {
  title: string;
  tone: string;
  items: { name: string; meta: string }[];
};

type TreeGroup = {
  title: string;
  items: { name: string; type: string; meta: string }[];
};

@Component({
  selector: 'app-portfolio-items',
  standalone: true,
  imports: [CommonModule, RouterLink],
  templateUrl: './portfolio-items.component.html',
  styleUrl: './portfolio-items.component.css',
  host: {
    class: 'block w-full'
  }
})
export class PortfolioItemsComponent implements OnInit, OnDestroy {
  viewMode: ViewMode = 'grid';
  showItemsSection = true;
  showContainersSection = true;

  items: PortfolioCard[] = [
    {
      name: 'Alpha Platform',
      type: 'Portfolio',
      status: 'Active',
      summary: 'Core independent worker OS driving all downstream systems.',
      metricLabel: 'Health',
      metricValue: '87',
      secondaryLabel: 'Budget',
      secondaryValue: '$45k',
      progress: 72,
      tone: 'text-[#10b981]',
      tags: ['platform', 'core'],
      owners: ['J', 'M', 'P']
    },
    {
      name: 'Community Growth',
      type: 'Program',
      status: 'Active',
      summary: 'Program to grow 50k active community members by Q4 2026.',
      metricLabel: 'Alignment',
      metricValue: '76',
      secondaryLabel: 'Projects',
      secondaryValue: '4',
      progress: 60,
      tone: 'text-[#f59e0b]',
      tags: ['community'],
      owners: ['A', 'K']
    },
    {
      name: 'Brand Identity System',
      type: 'Project',
      status: 'Active',
      summary: 'v2.0 rebrand: design system, motion language, partnership deck.',
      metricLabel: 'Completion',
      metricValue: '68%',
      secondaryLabel: 'Sprint',
      secondaryValue: '3 / 5',
      progress: 68,
      tone: 'text-[#8b5cf6]',
      tags: ['design', 'brand'],
      owners: ['J', 'L']
    },
    {
      name: 'Portfolio Engine v2',
      type: 'Project',
      status: 'Draft',
      summary: 'Next-gen portfolio engine with AI-powered recommendations.',
      metricLabel: 'Readiness',
      metricValue: '41%',
      secondaryLabel: 'Teams',
      secondaryValue: '2',
      progress: 41,
      tone: 'text-[#60a5fa]',
      tags: ['engine', 'ai'],
      owners: ['S', 'R']
    },
    {
      name: 'Research Hub',
      type: 'Resource',
      status: 'Active',
      summary: 'Knowledge base for research artifacts, experiments, and findings.',
      metricLabel: 'Coverage',
      metricValue: '82%',
      secondaryLabel: 'Docs',
      secondaryValue: '56',
      progress: 82,
      tone: 'text-[#22c55e]',
      tags: ['knowledge'],
      owners: ['N']
    }
  ];

  containers: PortfolioCard[] = [
    {
      name: 'Platform Binder',
      type: 'Binder',
      status: 'Active',
      summary: 'Logical collection grouping platform-related components by domain.',
      metricLabel: 'Coverage',
      metricValue: '91%',
      secondaryLabel: 'Items',
      secondaryValue: '14',
      progress: 91,
      tone: 'text-[#5ca0bf]',
      tags: ['binder'],
      owners: ['J']
    },
    {
      name: 'Project Dossier - Brand',
      type: 'ItemBook',
      status: 'Active',
      summary: 'ItemBook for Brand Identity System: charter, workspace, catalogue.',
      metricLabel: 'Consistency',
      metricValue: '88',
      secondaryLabel: 'Sections',
      secondaryValue: '9',
      progress: 88,
      tone: 'text-[#bf9a5c]',
      tags: ['charter', 'logs'],
      owners: ['M']
    },
    {
      name: 'Growth Playbook',
      type: 'Book',
      status: 'Active',
      summary: 'Trigger-based plays for acquisition, activation, retention loops.',
      metricLabel: 'Plays',
      metricValue: '12',
      secondaryLabel: 'Active',
      secondaryValue: '7',
      progress: 58,
      tone: 'text-[#5cbf7a]',
      tags: ['playbook'],
      owners: ['K']
    },
    {
      name: 'Design Assets Folder',
      type: 'Folder',
      status: 'Active',
      summary: 'File-system hierarchy for brand assets and exports.',
      metricLabel: 'Organisation',
      metricValue: '82',
      secondaryLabel: 'Files',
      secondaryValue: '124',
      progress: 82,
      tone: 'text-[#8b5cf6]',
      tags: ['files'],
      owners: ['L']
    }
  ];

  boardColumns: BoardColumn[] = [
    {
      title: 'Active',
      tone: 'text-[#10b981]',
      items: [
        { name: 'Alpha Platform', meta: 'Portfolio - Health 87' },
        { name: 'Community Growth', meta: 'Program - Alignment 76' },
        { name: 'Brand Identity System', meta: 'Project - Sprint 3/5' }
      ]
    },
    {
      title: 'Paused',
      tone: 'text-[#f59e0b]',
      items: [
        { name: 'Finance Infrastructure', meta: 'Project - Budget review' },
        { name: 'Investor Outreach', meta: 'Program - 2 stakeholders' }
      ]
    },
    {
      title: 'Draft',
      tone: 'text-[#8ea6ad]',
      items: [
        { name: 'Portfolio Engine v2', meta: 'Project - Drafting charter' }
      ]
    },
    {
      title: 'Complete',
      tone: 'text-[#3b82f6]',
      items: [
        { name: 'Q3 Strategy Deck', meta: 'Artifact - Published' }
      ]
    }
  ];

  containerBoardColumns: BoardColumn[] = [
    {
      title: 'Active',
      tone: 'text-[#10b981]',
      items: [
        { name: 'Platform Binder', meta: 'Binder - 14 items' },
        { name: 'Project Dossier - Brand', meta: 'ItemBook - 9 sections' }
      ]
    },
    {
      title: 'Draft',
      tone: 'text-[#8ea6ad]',
      items: [{ name: 'Portfolio Archive', meta: 'Archive - Snapshot in review' }]
    },
    {
      title: 'Reference',
      tone: 'text-[#60a5fa]',
      items: [{ name: 'Design Assets Folder', meta: 'Folder - 124 files' }]
    }
  ];

  treeGroups: TreeGroup[] = [
    {
      title: 'Programs',
      items: [
        { name: 'Community Growth', type: 'Program', meta: '4 projects' },
        { name: 'Design System Library', type: 'Program', meta: '2 projects' }
      ]
    },
    {
      title: 'Projects',
      items: [
        { name: 'Brand Identity System', type: 'Project', meta: 'Sprint 3/5' },
        { name: 'Portfolio Engine v2', type: 'Project', meta: 'Planning' }
      ]
    },
    {
      title: 'Resources',
      items: [
        { name: 'Research Hub', type: 'Resource', meta: '56 docs' },
        { name: 'Partner Contact Book', type: 'Record', meta: '38 contacts' }
      ]
    }
  ];

  containerTreeGroups: TreeGroup[] = [
    {
      title: 'Binders',
      items: [
        { name: 'Platform Binder', type: 'Binder', meta: '14 items' },
        { name: 'Community Binder', type: 'Binder', meta: '8 items' }
      ]
    },
    {
      title: 'Books & Dossiers',
      items: [
        { name: 'Project Dossier - Brand', type: 'ItemBook', meta: '9 sections' },
        { name: 'Growth Playbook', type: 'Book', meta: '12 plays' }
      ]
    },
    {
      title: 'Folders & Records',
      items: [
        { name: 'Design Assets Folder', type: 'Folder', meta: '124 files' },
        { name: 'Project Metadata Record', type: 'Record', meta: '48 entries' }
      ]
    }
  ];

  private readonly route = inject(ActivatedRoute);
  private querySub?: Subscription;

  ngOnInit(): void {
    this.querySub = this.route.queryParamMap.subscribe((params) => {
      const view = params.get('view') as ViewMode | null;
      if (view && ['grid', 'list', 'tree', 'board'].includes(view)) {
        this.viewMode = view;
      }
    });
  }

  ngOnDestroy(): void {
    this.querySub?.unsubscribe();
  }

  setViewMode(view: ViewMode): void {
    this.viewMode = view;
  }

  toggleItemsSection(): void {
    this.showItemsSection = !this.showItemsSection;
  }

  toggleContainersSection(): void {
    this.showContainersSection = !this.showContainersSection;
  }

  detailRoute(item: PortfolioCard): string[] {
    const type = item.type.toLowerCase();
    if (type.includes('program')) {
      return ['/portfolio/detail/program'];
    }
    if (type.includes('resource')) {
      return ['/portfolio/detail/resource'];
    }
    if (type.includes('asset')) {
      return ['/portfolio/detail/asset'];
    }
    if (type.includes('artifact')) {
      return ['/portfolio/detail/artifact'];
    }
    return ['/portfolio/detail/component'];
  }
}

