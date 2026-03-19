import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-itembook-workspace',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './itembook-workspace.component.html',
  styleUrl: './itembook-workspace.component.css',
  host: {
    class: 'block w-full'
  }
})
export class ItembookWorkspaceComponent {
  activeFiles = [
    'Brand Guidelines v2.pdf',
    'Growth Experiments.xlsx',
    'Community Messaging Framework.md'
  ];

  rooms = ['Strategy Room', 'Design Sync', 'Operations Sprint'];

  connectedItems = [
    { name: 'Community Growth Program', type: 'Program' },
    { name: 'Design Asset Pack', type: 'Asset' },
    { name: 'Portfolio Registry', type: 'Registry' }
  ];
}
