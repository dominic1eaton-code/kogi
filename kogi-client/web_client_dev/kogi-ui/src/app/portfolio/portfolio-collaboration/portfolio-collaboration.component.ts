import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-portfolio-collaboration',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './portfolio-collaboration.component.html',
  styleUrl: './portfolio-collaboration.component.css',
  host: {
    class: 'block w-full'
  }
})
export class PortfolioCollaborationComponent {
  sharedTypes = [
    { type: 'Team Portfolio', governance: 'Admin-controlled', meta: 'Leader override' },
    { type: 'Organization Portfolio', governance: 'Multi-sig', meta: 'Governance proposals' },
    { type: 'Collective Portfolio', governance: 'Open contribution', meta: 'Light moderation' },
    { type: 'Cooperative Portfolio', governance: 'Member vote', meta: 'Full quorum' },
    { type: 'Federation Portfolio', governance: 'Inter-org protocol', meta: 'Federated sync' },
    { type: 'Crowdresourced Portfolio', governance: 'Steward review', meta: 'Contribution ledger' }
  ];

  workflowStates = [
    'OPEN_CONTRIBUTION -> campaign launched',
    'SUBMITTED -> contributor submits work',
    'STEWARD_REVIEW -> review or vote required',
    'MERGED / ACCEPTED -> attribution updated',
    'DISTRIBUTION_EVENT -> rewards distributed'
  ];

  contributions = [
    { contributor: 'A. Kim', type: 'Labor', value: '120 hrs', status: 'Accepted' },
    { contributor: 'K. Moss', type: 'Design', value: 'Brand kit', status: 'Merged' },
    { contributor: 'Studio Collective', type: 'Capital', value: '$24k', status: 'Pending' }
  ];

  campaigns = [
    { name: 'Community Growth Sprint', focus: 'Labor + Knowledge', status: 'Open', rewards: 'Attribution weight' },
    { name: 'Design Asset Refresh', focus: 'Artifact + Design', status: 'Review', rewards: 'Equity + payout' }
  ];

  engineInsights = [
    'CollaborationScore: 82 (Healthy diversity)',
    '3 contributors have pending submissions',
    'Merge conflict detected in shared binder',
    'Attribution weights rebalanced last week'
  ];
}
