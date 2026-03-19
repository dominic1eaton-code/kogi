import { CommonModule } from '@angular/common';
import { Component, OnDestroy, OnInit, inject } from '@angular/core';
import { ActivatedRoute, RouterLink } from '@angular/router';
import { Subscription } from 'rxjs';

type DetailMetric = { label: string; value: string; tone?: string };

type DetailBlock = {
  title: string;
  items: { label: string; value: string }[];
};

type DetailConfig = {
  name: string;
  kindLabel: string;
  status: string;
  summary: string;
  tone: string;
  metrics: DetailMetric[];
  blocks: DetailBlock[];
  activity: string[];
};

@Component({
  selector: 'app-portfolio-detail',
  standalone: true,
  imports: [CommonModule, RouterLink],
  templateUrl: './portfolio-detail.component.html',
  styleUrl: './portfolio-detail.component.css',
  host: {
    class: 'block w-full'
  }
})
export class PortfolioDetailComponent implements OnInit, OnDestroy {
  kind = 'component';
  detail?: DetailConfig;

  private readonly route = inject(ActivatedRoute);
  private sub?: Subscription;

  private readonly detailMap: Record<string, DetailConfig> = {
    component: {
      name: 'Alpha Platform',
      kindLabel: 'Portfolio Component',
      status: 'Active',
      summary: 'Core portfolio system coordinating items, containers, analytics, and collaboration.',
      tone: 'text-[#10b981]',
      metrics: [
        { label: 'Health', value: '87', tone: 'text-[#10b981]' },
        { label: 'Budget', value: '$45k', tone: 'text-[#f59e0b]' },
        { label: 'Lifecycle', value: '92%', tone: 'text-[#60a5fa]' },
        { label: 'Risk', value: 'Low', tone: 'text-[#22c55e]' }
      ],
      blocks: [
        {
          title: 'Governance',
          items: [
            { label: 'Owner', value: 'Jordan Davis' },
            { label: 'Approval Tier', value: 'Multi-sig' },
            { label: 'Policy Set', value: 'PortfolioCore-v2' }
          ]
        },
        {
          title: 'Timeline',
          items: [
            { label: 'Phase', value: 'Execution' },
            { label: 'Milestones', value: '5 active' },
            { label: 'Next Review', value: 'Apr 02, 2026' }
          ]
        },
        {
          title: 'Relationships',
          items: [
            { label: 'Programs', value: '3' },
            { label: 'Projects', value: '12' },
            { label: 'Binders', value: '4' }
          ]
        }
      ],
      activity: [
        'ItemBook charter updated by M. Perez',
        'Budget request submitted for Q2 expansion',
        'New resource share granted to Research Hub'
      ]
    },
    program: {
      name: 'Community Growth',
      kindLabel: 'Program Detail',
      status: 'Active',
      summary: 'Portfolio program focused on growth, community expansion, and engagement loops.',
      tone: 'text-[#f59e0b]',
      metrics: [
        { label: 'Alignment', value: '76', tone: 'text-[#f59e0b]' },
        { label: 'Projects', value: '4', tone: 'text-[#60a5fa]' },
        { label: 'Sponsors', value: '2', tone: 'text-[#10b981]' },
        { label: 'Budget', value: '$22k', tone: 'text-[#f59e0b]' }
      ],
      blocks: [
        {
          title: 'Charter',
          items: [
            { label: 'Objective', value: '50k active community members' },
            { label: 'Scope', value: 'Growth + retention initiatives' },
            { label: 'Success Criteria', value: 'Engagement + retention > 40%' }
          ]
        },
        {
          title: 'Stakeholders',
          items: [
            { label: 'Program Lead', value: 'A. Kim' },
            { label: 'Steward', value: 'K. Moss' },
            { label: 'Partners', value: '3 alliances' }
          ]
        },
        {
          title: 'Resources',
          items: [
            { label: 'Channels', value: '5 active' },
            { label: 'Events', value: '8 scheduled' },
            { label: 'Budget Reserve', value: '$9k' }
          ]
        }
      ],
      activity: [
        'Campaign brief approved by governance committee',
        'New community partnership added to registry',
        'Retention playbook updated in ItemBook'
      ]
    },
    resource: {
      name: 'Research Hub',
      kindLabel: 'Resource Detail',
      status: 'Active',
      summary: 'Knowledge base and resource repository with shared access policies.',
      tone: 'text-[#22c55e]',
      metrics: [
        { label: 'Utilisation', value: '74%', tone: 'text-[#22c55e]' },
        { label: 'Shares', value: '3', tone: 'text-[#60a5fa]' },
        { label: 'Artifacts', value: '312', tone: 'text-[#8b5cf6]' },
        { label: 'Coverage', value: '82%', tone: 'text-[#10b981]' }
      ],
      blocks: [
        {
          title: 'Access Policy',
          items: [
            { label: 'Access Level', value: 'Contribute' },
            { label: 'Expiry', value: 'Jun 30, 2026' },
            { label: 'Attribution', value: 'Required' }
          ]
        },
        {
          title: 'Linked Items',
          items: [
            { label: 'Programs', value: '2' },
            { label: 'Projects', value: '5' },
            { label: 'ItemBooks', value: '4' }
          ]
        },
        {
          title: 'Compliance',
          items: [
            { label: 'Governance Status', value: 'Approved' },
            { label: 'Data Residency', value: 'US-East' },
            { label: 'Review Cadence', value: 'Monthly' }
          ]
        }
      ],
      activity: [
        'New dataset added by research team',
        'Access renewed for Growth Collective',
        'Resource share expiring in 12 days'
      ]
    },
    asset: {
      name: 'Brand Asset Pack',
      kindLabel: 'Asset Detail',
      status: 'Active',
      summary: 'Reusable asset bundle with governance, licensing, and versioning.',
      tone: 'text-[#8b5cf6]',
      metrics: [
        { label: 'Files', value: '52', tone: 'text-[#8b5cf6]' },
        { label: 'Licenses', value: '3', tone: 'text-[#60a5fa]' },
        { label: 'Value', value: '$120k', tone: 'text-[#10b981]' },
        { label: 'Usage', value: '68%', tone: 'text-[#f59e0b]' }
      ],
      blocks: [
        {
          title: 'Ownership',
          items: [
            { label: 'Custodian', value: 'Design Guild' },
            { label: 'License Type', value: 'Shared internal' },
            { label: 'Review', value: 'Quarterly' }
          ]
        },
        {
          title: 'Distribution',
          items: [
            { label: 'Channels', value: 'Marketplace + Internal' },
            { label: 'Subscriptions', value: '4 active' },
            { label: 'Exports', value: 'Weekly' }
          ]
        },
        {
          title: 'Dependencies',
          items: [
            { label: 'Linked Artifacts', value: '6' },
            { label: 'Programs', value: '2' },
            { label: 'ItemBook', value: 'Brand Dossier' }
          ]
        }
      ],
      activity: [
        'Asset pack updated with new exports',
        'License audit completed',
        'Usage spike detected in Growth Program'
      ]
    },
    artifact: {
      name: 'Q3 Strategy Deck',
      kindLabel: 'Artifact Detail',
      status: 'Published',
      summary: 'Strategic artifact with governance approval and version history.',
      tone: 'text-[#60a5fa]',
      metrics: [
        { label: 'Version', value: 'v2.1', tone: 'text-[#60a5fa]' },
        { label: 'Stakeholders', value: '6', tone: 'text-[#10b981]' },
        { label: 'Views', value: '1,280', tone: 'text-[#f59e0b]' },
        { label: 'Impact', value: 'High', tone: 'text-[#22c55e]' }
      ],
      blocks: [
        {
          title: 'Approval Chain',
          items: [
            { label: 'Governance', value: 'Approved' },
            { label: 'Last Review', value: 'Mar 10, 2026' },
            { label: 'Next Review', value: 'Jun 10, 2026' }
          ]
        },
        {
          title: 'Distribution',
          items: [
            { label: 'Channels', value: 'Board + Investor' },
            { label: 'Subscription', value: 'Enterprise' },
            { label: 'External', value: 'Public summary' }
          ]
        },
        {
          title: 'Dependencies',
          items: [
            { label: 'Source Data', value: 'Analytics Suite' },
            { label: 'Linked Projects', value: '4' },
            { label: 'ItemBook', value: 'Strategy Playbook' }
          ]
        }
      ],
      activity: [
        'Deck shared with investor group',
        'Q3 metrics updated in analytics',
        'Revision requested by governance' 
      ]
    }
  };

  ngOnInit(): void {
    this.sub = this.route.paramMap.subscribe((params) => {
      const kindParam = params.get('kind') ?? 'component';
      this.kind = kindParam;
      this.detail = this.detailMap[kindParam] ?? this.detailMap['component'];
    });
  }

  ngOnDestroy(): void {
    this.sub?.unsubscribe();
  }
}
