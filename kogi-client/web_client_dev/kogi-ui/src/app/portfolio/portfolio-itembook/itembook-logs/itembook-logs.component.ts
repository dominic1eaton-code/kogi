import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-itembook-logs',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './itembook-logs.component.html',
  styleUrl: './itembook-logs.component.css',
  host: {
    class: 'block w-full'
  }
})
export class ItembookLogsComponent {
  logs = [
    { time: 'Today 09:42', entry: 'Charter updated with new stakeholder alignment.' },
    { time: 'Yesterday 16:10', entry: 'Workspace linked to Brand Asset Pack.' },
    { time: 'Mar 15', entry: 'Catalogue entry added for Research Brief.' },
    { time: 'Mar 12', entry: 'Metrics refresh: health score +3.' }
  ];
}
