import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-itembook-schedule',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './itembook-schedule.component.html',
  styleUrl: './itembook-schedule.component.css',
  host: {
    class: 'block w-full'
  }
})
export class ItembookScheduleComponent {
  milestones = [
    { title: 'Stakeholder Review', date: 'Apr 02, 2026', status: 'Upcoming' },
    { title: 'Growth Sprint 4', date: 'Apr 12, 2026', status: 'Planned' },
    { title: 'Portfolio Health Audit', date: 'Apr 20, 2026', status: 'Scheduled' }
  ];

  reminders = [
    'Weekly governance sync - Mondays',
    'Resource share renewal - May 15',
    'Metrics reporting - Monthly close'
  ];
}
